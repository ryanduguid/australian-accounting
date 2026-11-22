// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The account day by day, worked exactly: the balance is `num / den` cents,
//! and `den` grows only on a day with excess, when that day's earnings are
//! worked at the day's GIC rate.

use num_bigint::BigInt;
use serde_json::{Value, json};

use super::events::Event;
use super::{KIND, dollars, general_cap, label, year_of};
use crate::dates;
use crate::engine::ato_interest_penalties::rates;
use crate::engine::{Result, d, rate, refuse, result};

const EARNINGS: &str = "ITAA 1997 s 294-235: the rate for the day (the TAA 1953 s 8AAD(1) GIC rate, the base interest rate plus 7 points over the days in the calendar year; no lower rate has been determined under s 294-235(3)) x the excess at the end of the day, credited at the start of the next day (s 294-25(1) item 3), so it compounds daily";
const TAX_15: &str = "Superannuation (Excess Transfer Balance Tax) Imposition Act 2016 s 5(1)(b): 15% of the notional earnings for the period (ITAA 1997 s 294-230(3)(a))";
const TAX_30: &str = "Superannuation (Excess Transfer Balance Tax) Imposition Act 2016 s 5(1)(a) and (2): 30% of the notional earnings, the period starting on or after 1 July 2018 after liability for an earlier period that started on or after 1 July 2018";
/// 1 July 2018, from which a second period's tax is 30%.
const SECOND_RATE_FROM: i64 = 17_713;

/// One excess transfer balance period (s 294-230(2)).
struct Period {
    from: i64,
    to: i64,
    earnings: BigInt,
}

/// The highest end-of-day balance in cents, the earliest day it was reached
/// and the personal cap on that day (s 294-40(2)(a) to (c)).
struct Highest {
    cents: BigInt,
    day: i64,
    cap: i64,
}

pub(super) struct Account {
    num: BigInt,
    den: BigInt,
    pending: BigInt,
    year: i64,
    cap: i64,
    exceeded: bool,
    highest: Option<Highest>,
    previously_liable: bool,
    caps: Vec<Value>,
    ledger: Vec<Value>,
    periods: Vec<Period>,
    open: bool,
}

impl Account {
    pub(super) fn new(events: &[Event], previously_liable: bool) -> Self {
        let year = year_of(events[0].day);
        let cap = general_cap(year);
        Account {
            num: BigInt::ZERO,
            den: BigInt::from(1),
            pending: BigInt::ZERO,
            year,
            cap,
            exceeded: false,
            highest: None,
            previously_liable,
            caps: vec![json!({
                "year": label(year),
                "general_transfer_balance_cap": dollars_of(cap),
                "transfer_balance_cap": dollars_of(cap),
                "authority": "ITAA 1997 s 294-35(1): the general transfer balance cap for the financial year the account starts",
            })],
            ledger: Vec::new(),
            periods: Vec::new(),
            open: false,
        }
    }

    /// s 294-40: a later year's cap, raised by the unused cap percentage of
    /// the general cap's indexation increase where the balance has never
    /// exceeded the cap at the end of a day.
    fn index(&mut self, year: i64) {
        let increase = general_cap(year) - general_cap(year - 1);
        let mut row = json!({
            "year": label(year),
            "general_transfer_balance_cap": dollars_of(general_cap(year)),
        });
        let raised = if increase <= 0 {
            row["authority"] = json!("ITAA 1997 s 294-35(2): the previous year's cap; the general cap was not increased by indexation (s 294-40(1)(a))");
            0
        } else if self.exceeded {
            row["authority"] = json!("ITAA 1997 s 294-40(1)(b): no proportional indexation, the balance having exceeded the cap at the end of a day before the year");
            0
        } else {
            let percent = self.unused_percent();
            if let Some(h) = &self.highest {
                row["highest_transfer_balance"] = json!(dollars(&h.cents, &BigInt::from(1)).to_string());
                row["highest_on"] = json!(dates::format(h.day));
                row["cap_on_that_day"] = dollars_of(h.cap);
            }
            row["unused_cap_percentage"] = json!(percent);
            row["indexation_increase"] = dollars_of(increase);
            row["authority"] = json!("ITAA 1997 s 294-40(2): unused cap percentage (100% less the highest end-of-day balance up to the end of the previous year as a percentage, rounded down, of the cap on the earliest day it was reached) x the general cap's indexation increase");
            increase * percent / 100
        };
        self.cap += raised;
        self.year = year;
        row["transfer_balance_cap"] = dollars_of(self.cap);
        self.caps.push(row);
    }

    fn unused_percent(&self) -> i64 {
        let Some(h) = &self.highest else { return 100 };
        if h.cents < BigInt::ZERO {
            return 100; // s 294-40(3)
        }
        let used: BigInt = &h.cents * 100 / (BigInt::from(h.cap) * 100);
        100 - i64::try_from(used).expect("a percentage")
    }

    /// One day: the earlier day's earnings credit, the day's events, the
    /// excess at its end and its earnings.
    pub(super) fn day(&mut self, day: i64, today: &[&Event], disregarded: bool) -> Result<()> {
        if year_of(day) != self.year {
            self.index(year_of(day));
        }
        self.num += std::mem::take(&mut self.pending);
        for e in today {
            self.record(e);
        }
        let over: BigInt = &self.num - BigInt::from(self.cap * 100) * &self.den;
        if over > BigInt::ZERO {
            self.exceeded = true;
        } else if !self.exceeded {
            self.note_highest(day);
        }
        if over > BigInt::ZERO && !disregarded {
            self.earn(day, over)?;
        } else {
            self.open = false;
        }
        Ok(())
    }

    fn record(&mut self, e: &Event) {
        let amount = BigInt::from(e.cents) * &self.den;
        if e.credit {
            self.num += amount;
        } else {
            self.num -= amount;
        }
        let (debit, credit) = if e.credit { (0, e.cents) } else { (e.cents, 0) };
        self.ledger.push(json!({
            "date": dates::format(e.day),
            "kind": e.kind,
            "authority": e.authority,
            "debit": dollars(&BigInt::from(debit), &BigInt::from(1)).to_string(),
            "credit": dollars(&BigInt::from(credit), &BigInt::from(1)).to_string(),
            "transfer_balance": dollars(&self.num, &self.den).to_string(),
        }));
    }

    fn note_highest(&mut self, day: i64) {
        let higher = self.highest.as_ref().is_none_or(|h| self.num > h.cents);
        if higher {
            self.highest = Some(Highest {
                cents: self.num.clone(),
                day,
                cap: self.cap,
            });
        }
    }

    /// The day's earnings on `over`, the excess in `num`'s units.
    fn earn(&mut self, day: i64, over: BigInt) -> Result<()> {
        let Some(gic) = rates::gic(day) else {
            return refuse(
                "transfer_balance.rate_unpublished",
                format!(
                    "The excess at the end of {} needs that day's GIC rate, which the ATO has published only to {}.",
                    dates::format(day),
                    rates::TABLE_END
                ),
            );
        };
        let scale = BigInt::from(10_000 * year_length(day));
        self.num *= &scale;
        self.den *= &scale;
        for p in &mut self.periods {
            p.earnings *= &scale;
        }
        let earned = over * gic;
        if !self.open {
            self.periods.push(Period {
                from: day,
                to: day,
                earnings: BigInt::ZERO,
            });
            self.open = true;
        }
        let period = self.periods.last_mut().expect("an open period");
        period.to = day;
        period.earnings += &earned;
        self.pending = earned;
        Ok(())
    }

    /// Each period's notional earnings and tax, in order.
    fn period_rows(&self) -> (Vec<Value>, BigInt, BigInt) {
        let (mut earnings, mut tax) = (BigInt::ZERO, BigInt::ZERO);
        let mut liable = self.previously_liable;
        let rows = self
            .periods
            .iter()
            .map(|p| {
                let second = liable && p.from >= SECOND_RATE_FROM;
                liable |= p.from >= SECOND_RATE_FROM;
                let percent = if second { 30 } else { 15 };
                let owed = &p.earnings * percent;
                earnings += &p.earnings;
                tax += &owed;
                json!({
                    "from": dates::format(p.from),
                    "to": dates::format(p.to),
                    "days": p.to - p.from + 1,
                    "notional_earnings": dollars(&p.earnings, &self.den).to_string(),
                    "tax_rate_percent": percent,
                    "excess_transfer_balance_tax": dollars(&owed, &(&self.den * 100)).to_string(),
                    "authority": if second { TAX_30 } else { TAX_15 },
                })
            })
            .collect();
        (rows, earnings, tax)
    }

    pub(super) fn finish(mut self, period: &str, to: i64) -> Result<Value> {
        let (rows, earnings, tax) = self.period_rows();
        let cap: BigInt = BigInt::from(self.cap * 100) * &self.den;
        let excess: BigInt = (&self.num - &cap).max(BigInt::ZERO);
        let space: BigInt = (&cap - &self.num).max(BigInt::ZERO);
        let mut out = result(
            KIND,
            period,
            vec![
                ("transfer_balance", dollars(&self.num, &self.den)),
                ("transfer_balance_cap", d(&format!("{}.00", self.cap))),
                ("cap_space", dollars(&space, &self.den)),
                ("excess_transfer_balance", dollars(&excess, &self.den)),
                ("notional_earnings", dollars(&earnings, &self.den)),
                ("excess_transfer_balance_tax", dollars(&tax, &(&self.den * 100))),
            ],
            vec![rate("general_transfer_balance_cap", general_cap(year_of(to)))],
        )?;
        out["to"] = json!(dates::format(to));
        out["caps"] = Value::Array(std::mem::take(&mut self.caps));
        out["ledger"] = Value::Array(std::mem::take(&mut self.ledger));
        out["excess_periods"] = Value::Array(rows);
        out["in_excess_at_to"] = json!(self.open);
        out["earnings_authority"] = json!(EARNINGS);
        Ok(out)
    }
}

fn dollars_of(whole: i64) -> Value {
    json!(format!("{whole}.00"))
}

fn year_length(day: i64) -> i64 {
    let (y, _, _) = dates::to_civil(day);
    if crate::engine::is_leap(y) { 366 } else { 365 }
}
