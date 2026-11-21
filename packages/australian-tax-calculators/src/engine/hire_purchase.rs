// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `hire_purchase_schedule`: a hire purchase agreement's notional interest
//! for each arrangement payment period at its implicit interest rate (ITAA
//! 1997 s 240-60), with each payment split into interest and principal on a
//! dated schedule, journal lines, an optional early payout and an optional
//! input tax credit. Its submodules also hold the chattel mortgage schedule
//! and loan statement, which share its payment schedule.

mod chattel_mortgage;
mod gst;
mod journals;
mod loan_transactions;
mod regime;
mod statement;

pub use chattel_mortgage::{ChattelMortgageArgs, chattel_mortgage_schedule};
pub use loan_transactions::{LoanTransactionsArgs, loan_transactions};

use serde_json::{Value, json};

use super::{
    Arg, Result, Step, d, date_arg, derived_result, int, money, precise, py_max, rate, refuse,
    scope, step,
};
use crate::dates;
use crate::decimal::{Calc, Decimal, Rounding};
use regime::{Regime, Unit};

const KIND: &str = "hire_purchase_schedule";

/// The amounts: (id, authority, formula). They cite ITAA 1997 compilation 268
/// (1 October 2026), read on 16 November 2026 and again on 18 November 2026.
/// The rows, their totals by income year and the implicit rate behind them are
/// not amounts, so no step works them out.
const STEPS: [(&str, &str, &str); 3] = [
    (
        "amount_financed",
        "ITAA 1997 s 240-25(3): the notional loan principal, as established",
        "amount_financed",
    ),
    (
        "total_payments",
        "ITAA 1997 s 240-25(6) and s 240-60(1) step 3: the arrangement payments, each instalment and the balloon, as the agreement schedules them to the end of the term; with a payout, payments_before_payout and the payout are what is paid",
        "instalment x instalments + balloon",
    ),
    (
        "total_interest",
        "ITAA 1997 s 240-25(6) and s 240-60(1): the payments repay the notional loan principal and pay its notional interest at the implicit interest rate, so the notional interest over the term is the payments less the principal, as scheduled to the end of the term; with a payout, interest_to_payout is the interest the schedule shows to its date",
        "total_payments - amount_financed",
    ),
];

const DEPOSIT: &str = "ITAA 1997 s 240-25(3): an amount paid by the notional buyer to the notional seller at or before the start of the arrangement for the cost of the property, as established";
const TRADE_IN: &str = "ITAA 1997 s 240-25(3): an amount credited by the notional seller as having been paid by the notional buyer at or before the start of the arrangement for the cost of the property, such as a trade-in, as established";
const CONSIDERATION: &str = "ITAA 1997 s 240-25(3) and (5): the consideration for the notional sale, which is the notional loan principal plus the amounts paid or credited at or before the start for the cost of the property";
const PAYOUT: &str = "ITAA 1997 s 240-75(1)(b) and s 995-1 'termination amount': the amount payable because the arrangement ends on the payout date, as established";
const INTEREST_TO_PAYOUT: &str = "ITAA 1997 s 240-60(1): the notional interest for each arrangement payment period up to the payout date, as the schedule shows it";
const PAYMENTS_BEFORE_PAYOUT: &str =
    "ITAA 1997 s 240-65: the arrangement payments due before the payout date, taken as made";
const BALANCE_AT_PAYOUT: &str = "ITAA 1997 s 240-60(1) steps 1 and 2: the notional loan principal plus the notional interest to the payout date, less the arrangement payments made";
const PAYOUT_EXCESS: &str = "ITAA 1997 s 240-105(2) and (4): at the end of the arrangement, the excess of the payments and termination amounts over the notional loan principal plus the assessed notional interest, which is the payout's excess over the balance outstanding";
const PAYOUT_SHORTFALL: &str = "ITAA 1997 s 240-105(3) and (4): at the end of the arrangement, the excess of the notional loan principal plus the assessed notional interest over the payments and termination amounts, which is the balance outstanding's excess over the payout";

/// Halvings of the 0 to 100% a unit interval the implicit rate is sought in,
/// leaving it within 2^-65 a unit; `implicit_rate` then checks that the rate
/// found leaves a nil balance to the cent.
const HALVINGS: usize = 64;

/// The facts as the command passes them; see the README for each.
pub struct HirePurchaseArgs<'a> {
    pub amount_financed: &'a Arg,
    pub instalment: Option<&'a Arg>,
    pub instalments: Option<&'a Arg>,
    pub frequency: Option<&'a Arg>,
    pub segments: Option<&'a Arg>,
    pub balloon: &'a Arg,
    pub timing: &'a Arg,
    pub start_date: &'a Arg,
    pub deposit: Option<&'a Arg>,
    pub trade_in: Option<&'a Arg>,
    pub payout: Option<&'a Arg>,
    pub gst: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The agreement's payments: its instalments on the boundaries of its
/// arrangement payment periods and a balloon at the end of the term.
struct Agreement {
    financed: Decimal,
    balloon: Decimal,
    in_advance: bool,
    regime: Regime,
}

/// `(1 + r)^n - 1`, the rate for a period of `n` units, built up without
/// forming `1 + r` so a 1-unit period's rate is `r` exactly.
fn growth(r: &Decimal, n: i64) -> Calc<Decimal> {
    let mut g = r.clone();
    for _ in 1..n {
        g = g.add(r)?.add(&r.mul(&g)?)?;
    }
    Ok(g)
}

impl Agreement {
    fn term(&self) -> i64 {
        self.regime.term()
    }

    /// The instalment the period from `from` to `to` lists: the one due at
    /// its start in advance, at its end in arrears; nil on or after a payout
    /// at boundary `until`, which replaces it.
    fn listed(&self, from: i64, to: i64, until: Option<i64>) -> Decimal {
        let at = if self.in_advance { from } else { to };
        match until {
            Some(b) if at >= b => int(0),
            _ => self.regime.due[at as usize].clone(),
        }
    }

    /// The balloon, in the last period when no payout comes first.
    fn balloon_in(&self, to: i64, until: Option<i64>) -> Decimal {
        if to == self.term() && until.is_none() {
            self.balloon.clone()
        } else {
            int(0)
        }
    }

    /// The unrounded balance after the last payment at the rate `r` a unit:
    /// nil at the implicit rate, below nil under it and above nil over it,
    /// since it is the amount financed less the payments' present value at
    /// `r`, carried to the end of the term.
    fn residual(&self, r: &Decimal) -> Calc<Decimal> {
        let mut balance = self.financed.clone();
        for &(from, to) in &self.regime.periods {
            let listed = self.listed(from, to, None);
            let base = if self.in_advance {
                balance.sub(&listed)?
            } else {
                balance.clone()
            };
            let interest = base.mul(&growth(r, to - from)?)?;
            let payment = listed.add(&self.balloon_in(to, None))?;
            balance = balance.add(&interest)?.sub(&payment)?;
        }
        Ok(balance)
    }

    /// s 240-60(1) step 3: the compound rate a unit at which the payments
    /// repay the amount financed, by halving the interval it lies in.
    fn implicit_rate(&self) -> Result<Decimal> {
        let (mut low, mut high) = (int(0), int(1));
        let positive = |r: &Decimal| -> Result<bool> { Ok(int(0).lt(&self.residual(r)?)?) };
        // At nil the residual is the amount financed less the payments.
        if self.residual(&low)?.is_zero() {
            return Ok(low);
        }
        if positive(&low)? || !positive(&high)? {
            return refuse(
                "hire_purchase.rate_unsupported",
                "The payments imply an interest rate below nil (they total less than the amount financed) or of 100% a month (or week) or more; this worksheet takes rates from nil to below 100% a month (or week).",
            );
        }
        for _ in 0..HALVINGS {
            let middle = low.add(&high)?.div(&int(2))?;
            if positive(&middle)? {
                high = middle;
            } else {
                low = middle;
            }
        }
        let r = low.add(&high)?.div(&int(2))?;
        // The balance this rate leaves must round to nil, or the schedule's
        // cents cannot be trusted: at a high rate over a long term the last
        // halving still moves it by more.
        let left = self.residual(&r)?;
        let half_cent = d("0.005");
        if !(half_cent.neg()?.lt(&left)? && left.lt(&half_cent)?) {
            return refuse(
                "hire_purchase.rate_imprecise",
                "At this rate and term the implicit rate cannot be found closely enough to leave a nil balance to the cent.",
            );
        }
        Ok(r)
    }

    /// One row an arrangement payment period in cents, to the payout at
    /// boundary `until` or the end of the term. Each period's interest is the
    /// notional interest to date at `r` rounded half up to cents, less what
    /// the periods before show, so a cent of rounding never compounds: over
    /// the whole term the periods add up to the total charges, each is within
    /// a cent of its amount at `r` and the last balance is nil. The balloon
    /// has its own column, as in advance it is paid a period after the last
    /// instalment. Beside the rows, each period's dates, interest and payment
    /// in cents.
    fn schedule(
        &self,
        r: &Decimal,
        start: i64,
        charges: &Decimal,
        until: Option<i64>,
    ) -> Result<(Vec<Value>, Vec<Period>)> {
        let (mut rows, mut periods) = (Vec::new(), Vec::new());
        let (mut exact, mut to_date, mut shown) = (self.financed.clone(), int(0), int(0));
        let mut balance = self.financed.clone();
        let unit = self.regime.unit;
        let last = until.unwrap_or_else(|| self.term());
        for (i, &(from, to)) in self.regime.periods.iter().enumerate() {
            if to > last {
                break;
            }
            let k = i as i64 + 1;
            let listed = self.listed(from, to, until);
            let balloon = self.balloon_in(to, until);
            let payment = listed.add(&balloon)?;
            let base = if self.in_advance {
                exact.sub(&listed)?
            } else {
                exact.clone()
            };
            let interest_exact = base.mul(&growth(r, to - from)?)?;
            to_date = to_date.add(&interest_exact)?;
            exact = exact.add(&interest_exact)?.sub(&payment)?;
            let through = if to == self.term() && until.is_none() {
                charges.clone()
            } else {
                to_date.quantize(-2, Rounding::HalfUp)?
            };
            let interest = through.sub(&shown)?;
            shown = through;
            let principal = payment.sub(&interest)?;
            let closing = balance.sub(&principal)?;
            let (from_day, to_day) = (unit.date(start, from), unit.date(start, to));
            let mut payments = Vec::new();
            if !listed.is_zero() {
                payments.push((
                    if self.in_advance { from_day } else { to_day },
                    listed.clone(),
                ));
            }
            if !balloon.is_zero() {
                match payments.last_mut() {
                    Some((day, paid)) if *day == to_day => *paid = paid.add(&balloon)?,
                    _ => payments.push((to_day, balloon.clone())),
                }
            }
            let paid_on = payments.first().map(|p| p.0);
            rows.push(json!({
                "instalment": k,
                "period_start": dates::format(from_day),
                "period_end": dates::format(to_day),
                "payment_date": paid_on.map(dates::format),
                "opening": balance.fixed(2, false),
                "interest": interest.fixed(2, false),
                "payment": listed.fixed(2, false),
                "balloon": balloon.fixed(2, false),
                "principal": principal.fixed(2, false),
                "closing": closing.fixed(2, false),
            }));
            periods.push(Period {
                instalment: k,
                from: from_day,
                to: to_day,
                interest,
                payments,
            });
            balance = closing;
        }
        Ok((rows, periods))
    }
}

/// One schedule period: its number, its days from `from` up to `to`, its
/// interest as shown and what is paid in it, as (day, amount), in date order.
struct Period {
    instalment: i64,
    from: i64,
    to: i64,
    interest: Decimal,
    payments: Vec<(i64, Decimal)>,
}

/// The income year `day` falls in, as `2026-27`, and the 1 July that ends it.
fn income_year(day: i64) -> (String, i64) {
    let (y, m, _) = dates::to_civil(day);
    let first = if m >= 7 { y } else { y - 1 };
    let ends = dates::parse(&format!("{}-07-01", first + 1)).expect("1 July");
    (format!("{first}-{:02}", (first + 1) % 100), ends)
}

/// The periods' interest by income year (ITAA 1997 s 240-60(2)). A period
/// wholly in one income year counts in it. A period spanning 30 June is
/// listed apart, with its days before and from 1 July: s 240-60(2) relates its
/// interest to each year in accordance with generally accepted accounting
/// principles, which this worksheet leaves to the caller.
fn by_income_year(periods: &[Period]) -> Result<(Value, Value)> {
    // (income year, first instalment, last instalment, interest)
    type Year = (String, i64, i64, Decimal);
    let (mut years, mut across): (Vec<Year>, Vec<Value>) = (Vec::new(), Vec::new());
    for period in periods {
        let (year, ends) = income_year(period.from);
        if period.to > ends {
            across.push(json!({
                "instalment": period.instalment,
                "interest": period.interest.fixed(2, false),
                "income_years": [year, income_year(ends).0],
                "days": [ends - period.from, period.to - ends],
            }));
        } else if let Some(last) = years.last_mut().filter(|last| last.0 == year) {
            last.2 = period.instalment;
            last.3 = last.3.add(&period.interest)?;
        } else {
            years.push((
                year,
                period.instalment,
                period.instalment,
                period.interest.clone(),
            ));
        }
    }
    let years = years
        .into_iter()
        .map(|(year, first, last, interest)| {
            json!({
                "income_year": year,
                "first_instalment": first,
                "last_instalment": last,
                "interest": interest.fixed(2, false),
            })
        })
        .collect();
    Ok((Value::Array(years), Value::Array(across)))
}

/// `start_date` as a day number, refused unless it falls in the income year.
fn start_in_year(start_date: &Arg, year: &str) -> Result<i64> {
    let start = date_arg(start_date, "start_date")?;
    let first: i64 = year[..4].parse().expect("income year");
    let opens = dates::parse(&format!("{first}-07-01")).expect("income year");
    let closes = dates::parse(&format!("{}-06-30", first + 1)).expect("income year");
    if start < opens || start > closes {
        return refuse(
            "hire_purchase.start_outside_year",
            format!(
                "start_date must fall in the {year} income year, 1 July {first} to 30 June {}.",
                first + 1
            ),
        );
    }
    Ok(start)
}

/// The rate `r` a unit, implicit or stated, and the nominal annual rate (12 or 52 times it), to
/// 16 places, so the rates reproduce a period's interest to the cent at the
/// largest amount, and fixed-point, so a nil rate reads 0.0000000000000000;
/// then any car limit the GST facts were worked with.
fn rates(
    r: &Decimal,
    unit: Unit,
    implicit: bool,
    car_limit: Option<&Decimal>,
) -> Result<Vec<(String, String)>> {
    let (key, per_year) = match (unit, implicit) {
        (Unit::Month, true) => ("implicit_monthly_rate", 12),
        (Unit::Week, true) => ("implicit_weekly_rate", 52),
        (Unit::Month, false) => ("monthly_rate", 12),
        (Unit::Week, false) => ("weekly_rate", 52),
    };
    precise(|| {
        let mut rates = vec![
            rate(key, r.quantize(-16, Rounding::HalfUp)?.fixed(16, false)),
            rate(
                "nominal_annual_rate",
                r.mul(&int(per_year))?
                    .quantize(-16, Rounding::HalfUp)?
                    .fixed(16, false),
            ),
        ];
        rates.extend(car_limit.map(|limit| rate("car_limit", limit)));
        Ok(rates)
    })
}

fn timing(value: &Arg) -> Result<bool> {
    match value.text() {
        Some("in_arrears") => Ok(false),
        Some("in_advance") => Ok(true),
        _ => refuse(
            "input.timing_invalid",
            "timing must be in_arrears or in_advance.",
        ),
    }
}

/// An input only one regime takes: required for it, refused for the other.
fn regime_input<'a>(given: Option<&'a Arg>, key: &str, level: bool) -> Result<Option<&'a Arg>> {
    match (level, given) {
        (true, None) if key != "frequency" => {
            refuse("input.missing_field", format!("{key} is required"))
        }
        (false, Some(_)) => refuse(
            "input.field_unknown",
            format!("{key} is an input only without segments"),
        ),
        (_, given) => Ok(given),
    }
}

/// The agreement, its start day and, for a level regime, its instalment and
/// their number.
type Agreed = (Agreement, i64, Option<(Decimal, i64)>);

/// The agreement and its start day, refusing a nil amount, a term outside
/// 30 years, an unknown timing or frequency, a start outside the income year
/// and segments that do not run on from it.
fn agreement(args: &HirePurchaseArgs, year: &str) -> Result<Agreed> {
    let financed = money(args.amount_financed)?;
    let level = args.segments.is_none();
    let instalment = regime_input(args.instalment, "instalment", level)?;
    let instalments = regime_input(args.instalments, "instalments", level)?;
    let frequency = regime_input(args.frequency, "frequency", level)?;
    let (Some(instalment), Some(instalments)) = (instalment, instalments) else {
        // Segments.
        if financed.is_zero() {
            return refuse(
                "hire_purchase.amount_nil",
                "amount_financed must be more than nil.",
            );
        }
        let balloon = money(args.balloon)?;
        let in_advance = timing(args.timing)?;
        let start = start_in_year(args.start_date, year)?;
        let segments = args.segments.expect("segments without instalments");
        let plan = regime::segments(segments, start)?;
        let agreement = Agreement {
            financed,
            balloon,
            in_advance,
            regime: regime::build(plan, in_advance, true)?,
        };
        return Ok((agreement, start, None));
    };
    let amount = money(instalment)?;
    if financed.is_zero() || amount.is_zero() {
        return refuse(
            "hire_purchase.amount_nil",
            "amount_financed and instalment must be more than nil.",
        );
    }
    let balloon = money(args.balloon)?;
    let plan = regime::level(amount.clone(), instalments, frequency)?;
    let in_advance = timing(args.timing)?;
    let start = start_in_year(args.start_date, year)?;
    let count = instalments.int().flatten().expect("a count in range");
    let agreement = Agreement {
        financed,
        balloon,
        in_advance,
        regime: regime::build(plan, in_advance, true)?,
    };
    Ok((agreement, start, Some((amount, count))))
}

/// An optional amount, nil when not given.
fn optional_money(value: Option<&Arg>) -> Result<Decimal> {
    value.map_or_else(|| Ok(int(0)), money)
}

/// The payout's date and amount.
struct Payout {
    day: i64,
    amount: Decimal,
    /// The boundary it falls on, in units from the start.
    at: i64,
}

fn payout_invalid<T>() -> Result<T> {
    refuse(
        "hire_purchase.payout_invalid",
        "payout must be an object with date (YYYY-MM-DD), amount (a decimal string) and known_at_start (a boolean).",
    )
}

/// The payout, which must fall on the end of an arrangement payment period
/// before the end of the term.
fn read_payout(value: &Arg, agreement: &Agreement, start: i64) -> Result<Payout> {
    let Arg::Json(Value::Object(fields)) = value else {
        return payout_invalid();
    };
    if let Some(key) = fields
        .keys()
        .find(|k| !matches!(k.as_str(), "date" | "amount" | "known_at_start"))
    {
        return refuse(
            "input.field_unknown",
            format!("payout: {key} is not a fact a payout takes"),
        );
    }
    let text = |key: &str| fields.get(key).and_then(Value::as_str);
    let (Some(day), Some(amount), Some(known)) = (
        text("date").and_then(dates::parse),
        text("amount").and_then(|t| Decimal::parse_input(t).ok()),
        fields.get("known_at_start").and_then(Value::as_bool),
    ) else {
        return payout_invalid();
    };
    if known {
        return refuse(
            "hire_purchase.payout_known_at_start",
            "A payout known or expected at the start of the arrangement, or one no reasonable estimate could then be made of, changes the implicit interest rate (ITAA 1997 s 240-60(1) step 3, (3) and (4)); this worksheet finds the rate from the scheduled payments only, as where the reasonable estimate at the start was no payout.",
        );
    }
    let amount = money(&Arg::Decimal(amount))?;
    let unit = agreement.regime.unit;
    let at = agreement
        .regime
        .periods
        .iter()
        .map(|p| p.1)
        .find(|&to| to < agreement.term() && unit.date(start, to) == day);
    match at {
        Some(at) => Ok(Payout { day, amount, at }),
        None => refuse(
            "hire_purchase.payout_date_unsupported",
            "payout date must be the end of an arrangement payment period after the start and before the end of the term: Division 240, as read, gives no notional interest for part of a period that a payout cuts short.",
        ),
    }
}

/// The Division 240 consequence of a payout's `excess` over the balance or
/// `shortfall` below it.
fn consequence(excess: &Decimal, shortfall: &Decimal) -> &'static str {
    if excess.is_zero() && shortfall.is_zero() {
        "The payout equals the balance outstanding, so the payments and termination amounts equal the notional loan principal plus the assessed notional interest and Subdivision 240-G makes no adjustment (ITAA 1997 s 240-105). The payout itself is not deductible where it is paid to acquire the property (s 240-85(b))."
    } else if !excess.is_zero() {
        "The payout exceeds the balance outstanding: at the end of the arrangement (s 240-75(1)(b)) the payments and termination amounts exceed the notional loan principal plus the assessed notional interest, so s 240-105(2) includes the excess in the notional seller's assessable income and s 240-110(1) entitles the notional buyer to deduct a corresponding amount in its income year, to the extent s 240-110(3) allows. The payout itself is not deductible where it is paid to acquire the property (s 240-85(b))."
    } else {
        "The payout is less than the balance outstanding: at the end of the arrangement (s 240-75(1)(b)) the notional loan principal plus the assessed notional interest exceed the payments and termination amounts, so s 240-105(3) entitles the notional seller to deduct the excess and s 240-110(2) includes a corresponding amount in the notional buyer's assessable income for its income year, to the extent s 240-110(3) allows. The payout itself is not deductible where it is paid to acquire the property (s 240-85(b))."
    }
}

/// What a reviewer must establish before relying on a payout's consequence.
const PAYOUT_REVIEW: &str = "For review: this takes the payout as a termination amount payable because the arrangement ends on the payout date, every arrangement payment due before it as paid, and, as the payout was neither known nor expected at the start (known_at_start false), the implicit rate as found from the payments the agreement scheduled: the reasonable estimate of termination amounts at the start was nil (s 240-60(3)), and no estimate under s 240-60(4) is made. Whether s 240-110(3) limits the notional buyer's deduction or inclusion, and which of its income years it falls in, are not worked out.";

/// `values` written as a sum, `0` for none, and their total.
fn sum_terms(values: Vec<&Decimal>) -> Result<(String, Decimal)> {
    let total = values.iter().try_fold(int(0), |sum, v| sum.add(v))?;
    let text = values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" + ");
    Ok((if text.is_empty() { "0".into() } else { text }, total))
}

/// The payout's steps: the payout, the interest and payments to its date, the
/// balance it discharges and the difference.
fn payout_steps(payout: &Payout, financed: &Decimal, periods: &[Period]) -> Result<Vec<Step>> {
    let (interest_terms, interest) = sum_terms(periods.iter().map(|p| &p.interest).collect())?;
    let paid = periods
        .iter()
        .flat_map(|p| p.payments.iter().map(|(_, amount)| amount));
    let (paid_terms, paid_total) = sum_terms(paid.collect())?;
    let mut steps = vec![
        step(
            "payout",
            PAYOUT,
            "payout",
            &[&payout.amount],
            payout.amount.clone(),
        ),
        step(
            "interest_to_payout",
            INTEREST_TO_PAYOUT,
            &interest_terms,
            &[],
            interest.clone(),
        ),
        step(
            "payments_before_payout",
            PAYMENTS_BEFORE_PAYOUT,
            &paid_terms,
            &[],
            paid_total.clone(),
        ),
    ];
    steps.extend(settlement_steps(
        &payout.amount,
        financed,
        &interest,
        &paid_total,
    )?);
    Ok(steps)
}

/// The balance a payout of `amount` discharges and its excess or shortfall.
fn settlement_steps(
    amount: &Decimal,
    financed: &Decimal,
    interest: &Decimal,
    paid: &Decimal,
) -> Result<[Step; 3]> {
    let balance = financed.add(interest)?.sub(paid)?;
    let difference = amount.sub(&balance)?;
    Ok([
        step(
            "balance_at_payout",
            BALANCE_AT_PAYOUT,
            "amount_financed + interest_to_payout - payments_before_payout",
            &[financed, interest, paid],
            balance.clone(),
        ),
        step(
            "payout_excess",
            PAYOUT_EXCESS,
            "max(0, payout - balance_at_payout)",
            &[amount, &balance],
            py_max(int(0), difference.clone())?,
        ),
        step(
            "payout_shortfall",
            PAYOUT_SHORTFALL,
            "max(0, balance_at_payout - payout)",
            &[&balance, amount],
            py_max(int(0), difference.neg()?)?,
        ),
    ])
}

/// The deposit, trade-in and consideration steps.
fn consideration_steps(
    financed: &Decimal,
    deposit: &Decimal,
    trade_in: &Decimal,
) -> Result<[Step; 3]> {
    let consideration = financed.add(deposit)?.add(trade_in)?;
    Ok([
        step("deposit", DEPOSIT, "deposit", &[deposit], deposit.clone()),
        step(
            "trade_in",
            TRADE_IN,
            "trade_in",
            &[trade_in],
            trade_in.clone(),
        ),
        step(
            "consideration",
            CONSIDERATION,
            "amount_financed + deposit + trade_in",
            &[financed, deposit, trade_in],
            consideration,
        ),
    ])
}

/// The first day any consideration is provided: the start, where a deposit
/// or trade-in is given then, or the first payment's day.
fn first_consideration(periods: &[Period], start: i64, upfront: bool) -> i64 {
    if upfront {
        return start;
    }
    periods
        .iter()
        .flat_map(|p| &p.payments)
        .next()
        .map_or(start, |(day, _)| *day)
}

/// The rendered amount `key` as a decimal.
fn amount(out: &Value, key: &str) -> Decimal {
    d(out["amounts"][key].as_str().expect("a rendered amount"))
}

/// The amount financed, total payments and total interest steps.
fn base_steps(
    agreement: &Agreement,
    level: Option<&(Decimal, i64)>,
    total: &Decimal,
    charges: &Decimal,
) -> Vec<Step> {
    let [
        (id, financed_authority, formula),
        (total_id, total_authority, level_formula),
        (interest_id, interest_authority, interest_formula),
    ] = STEPS;
    let financed = &agreement.financed;
    let total_step = match (&agreement.regime.terms, level) {
        (Some(terms), _) => step(
            total_id,
            total_authority,
            &format!("{terms} + balloon"),
            &[&agreement.balloon],
            total.clone(),
        ),
        (None, Some((instalment, count))) => step(
            total_id,
            total_authority,
            level_formula,
            &[instalment, &int(*count), &agreement.balloon],
            total.clone(),
        ),
        (None, None) => unreachable!("a level regime has its instalment"),
    };
    vec![
        step(
            id,
            financed_authority,
            formula,
            &[financed],
            financed.clone(),
        ),
        total_step,
        step(
            interest_id,
            interest_authority,
            interest_formula,
            &[total, financed],
            charges.clone(),
        ),
    ]
}

/// The credit the journals book, from the rendered result.
fn booked(out: &Value, worked: Option<&gst::Worked>) -> Option<journals::Booked> {
    let purpose = worked?.purpose_on_interest.clone()?;
    Some(journals::Booked {
        credit: amount(out, "input_tax_credit"),
        on_interest: amount(out, "credit_on_interest"),
        purpose,
    })
}

/// The payout's date, its Division 240 consequence and what to review.
fn payout_summary(out: &Value, payout: &Payout) -> Value {
    let (excess, shortfall) = (
        amount(out, "payout_excess"),
        amount(out, "payout_shortfall"),
    );
    json!({
        "date": dates::format(payout.day),
        "consequence": consequence(&excess, &shortfall),
        "review": PAYOUT_REVIEW,
    })
}

/// The rate, schedule rows, periods, steps and worked credit.
type Amounts = (
    Decimal,
    Vec<Value>,
    Vec<Period>,
    Vec<Step>,
    Option<gst::Worked>,
);

/// The worksheet's amounts from its facts: `upfront` is the deposit and
/// trade-in.
fn amounts(
    args: &HirePurchaseArgs,
    agreement: &Agreement,
    start: i64,
    level: Option<&(Decimal, i64)>,
    upfront: [&Decimal; 2],
    gst_facts: &gst::Facts,
    payout: Option<&Payout>,
) -> Result<Amounts> {
    let ([deposit, trade_in], financed) = (upfront, &agreement.financed);
    let r = agreement.implicit_rate()?;
    let total = agreement.regime.total.add(&agreement.balloon)?;
    let charges = total.sub(financed)?;
    let (rows, periods) = agreement.schedule(&r, start, &charges, payout.map(|p| p.at))?;
    let mut steps = base_steps(agreement, level, &total, &charges);
    if args.deposit.is_some() || args.trade_in.is_some() || args.gst.is_some() {
        steps.extend(consideration_steps(financed, deposit, trade_in)?);
    }
    let consideration = financed.add(deposit)?.add(trade_in)?;
    let mut gst_worked = gst::worked(gst_facts, &consideration, &charges)?;
    if let Some(worked) = &mut gst_worked {
        steps.append(&mut worked.steps);
    }
    if let Some(payout) = payout {
        steps.extend(payout_steps(payout, financed, &periods)?);
    }
    Ok((r, rows, periods, steps, gst_worked))
}

/// The payout, refused beside GST facts.
fn payout_without_gst(
    args: &HirePurchaseArgs,
    agreement: &Agreement,
    start: i64,
) -> Result<Option<Payout>> {
    let payout = args
        .payout
        .map(|p| read_payout(p, agreement, start))
        .transpose()?;
    if payout.is_some() && args.gst.is_some() {
        return refuse(
            "hire_purchase.payout_gst_unsupported",
            "A payout is worked without gst facts: it changes the consideration for an agreement whose input tax credit is attributable when it starts, and this worksheet does not work out any adjustment that causes.",
        );
    }
    Ok(payout)
}

pub fn hire_purchase_schedule(args: &HirePurchaseArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let (agreement, start, level) = agreement(args, &year)?;
    let deposit = optional_money(args.deposit)?;
    let trade_in = optional_money(args.trade_in)?;
    let payout = payout_without_gst(args, &agreement, start)?;
    let gst_facts = gst::read(args.gst, true)?;
    let financed = &agreement.financed;
    let (r, rows, periods, steps, gst_worked) = precise(|| {
        amounts(
            args,
            &agreement,
            start,
            level.as_ref(),
            [&deposit, &trade_in],
            &gst_facts,
            payout.as_ref(),
        )
    })?;
    let unit = agreement.regime.unit;
    let car_limit = gst_worked.as_ref().and_then(|w| w.car_limit.as_ref());
    let mut out = derived_result(KIND, &year, steps, rates(&r, unit, true, car_limit)?)?;
    let facts = journals::Facts {
        start,
        financed,
        deposit: &deposit,
        trade_in: &trade_in,
        gst: booked(&out, gst_worked.as_ref()),
        periods: &periods,
        payout: payout
            .as_ref()
            .map(|p| (p.day, amount(&out, "balance_at_payout"), p.amount.clone())),
        whole_term: payout.is_none(),
    };
    let journals = precise(|| journals::journals(&facts))?;
    let (years, across) = precise(|| by_income_year(&periods))?;
    let upfront = !deposit.is_zero() || !trade_in.is_zero();
    let first = first_consideration(&periods, start, upfront);
    // The balloon falls due at the end of the term, after an instalment in advance.
    out["balloon_date"] = json!(dates::format(unit.date(start, agreement.term())));
    out["schedule"] = Value::Array(rows);
    out["interest_by_income_year"] = years;
    out["interest_across_30_june"] = across;
    out["journals"] = journals;
    out["gst"] = gst::summary(&gst_facts, &dates::format(first));
    if let Some(payout) = &payout {
        out["payout"] = payout_summary(&out, payout);
    }
    Ok(out)
}
