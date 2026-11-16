// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `hire_purchase_schedule`: a hire purchase agreement's monthly notional
//! interest at its implicit interest rate (ITAA 1997 s 240-60), with each
//! payment split into interest and principal on a dated schedule.

use serde_json::{Value, json};

use super::{
    Arg, Result, d, date_arg, derived_result, int, money, precise, rate, refuse, scope, steps_from,
};
use crate::dates;
use crate::decimal::{Calc, Decimal, Rounding};

const KIND: &str = "hire_purchase_schedule";

/// The amounts: (id, authority, formula). They cite ITAA 1997 compilation 268
/// (1 October 2026), read on 16 November 2026. The monthly rows, their
/// totals by income year and the implicit rate behind them are not amounts, so
/// no step works them out.
const STEPS: [(&str, &str, &str); 3] = [
    (
        "amount_financed",
        "ITAA 1997 s 240-25(3): the notional loan principal, as established",
        "amount_financed",
    ),
    (
        "total_payments",
        "ITAA 1997 s 240-25(6) and s 240-60(1) step 3: the arrangement payments, each instalment and the balloon",
        "instalment x instalments + balloon",
    ),
    (
        "total_interest",
        "ITAA 1997 s 240-25(6) and s 240-60(1): the payments repay the notional loan principal and pay its notional interest at the implicit interest rate, so the notional interest over the term is the payments less the principal",
        "total_payments - amount_financed",
    ),
];

/// 30 years of monthly instalments.
const MOST_INSTALMENTS: i64 = 360;

/// Halvings of the 0 to 100% a month interval the implicit rate is sought in,
/// leaving it within 2^-65 a month; `implicit_rate` then checks that the rate
/// found leaves a nil balance to the cent.
const HALVINGS: usize = 64;

/// The agreement's payments: a level monthly instalment, in arrears or in
/// advance, and a balloon at the end of the term, the last month's end in
/// either timing.
struct Agreement {
    financed: Decimal,
    instalment: Decimal,
    count: i64,
    balloon: Decimal,
    in_advance: bool,
}

impl Agreement {
    /// The payments in month `k`: the instalment, with the balloon in the last.
    fn payment(&self, k: i64) -> Calc<Decimal> {
        if k == self.count {
            self.instalment.add(&self.balloon)
        } else {
            Ok(self.instalment.clone())
        }
    }

    /// What the month's interest accrues on: the opening balance, less the
    /// month's instalment when it is paid at the start of the month.
    fn interest_base(&self, opening: &Decimal) -> Calc<Decimal> {
        if self.in_advance {
            opening.sub(&self.instalment)
        } else {
            Ok(opening.clone())
        }
    }

    /// The unrounded balance after the last payment at the monthly rate `r`:
    /// nil at the implicit rate, below nil under it and above nil over it,
    /// since it is the amount financed less the payments' present value at
    /// `r`, carried to the end of the term.
    fn residual(&self, r: &Decimal) -> Calc<Decimal> {
        let mut balance = self.financed.clone();
        for k in 1..=self.count {
            let interest = self.interest_base(&balance)?.mul(r)?;
            balance = balance.add(&interest)?.sub(&self.payment(k)?)?;
        }
        Ok(balance)
    }

    /// s 240-60(1) step 3: the monthly compound rate at which the payments
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
                "The payments imply an interest rate below nil (they total less than the amount financed) or of 100% a month or more; this worksheet takes rates from nil to below 100% a month.",
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

    /// One row a month in cents. Each month's interest is the notional
    /// interest to date at `r` rounded half up to cents, less what the months
    /// before show, so a cent of rounding never compounds: the months add up to
    /// the total charges and each is within a cent of its amount at `r`. Every
    /// balance shown is then the balance at `r` rounded, and the last is nil.
    /// The balloon has its own column, as in advance it is paid a month after
    /// the last instalment.
    /// Beside the rows, each month's period and interest in cents.
    fn schedule(
        &self,
        r: &Decimal,
        start: i64,
        charges: &Decimal,
    ) -> Result<(Vec<Value>, Vec<Month>)> {
        let (mut rows, mut months) = (Vec::new(), Vec::new());
        let (mut exact, mut to_date, mut shown) = (self.financed.clone(), int(0), int(0));
        let mut balance = self.financed.clone();
        for k in 1..=self.count {
            let payment = self.payment(k)?;
            let balloon = payment.sub(&self.instalment)?;
            let interest_exact = self.interest_base(&exact)?.mul(r)?;
            to_date = to_date.add(&interest_exact)?;
            exact = exact.add(&interest_exact)?.sub(&payment)?;
            let through = if k == self.count {
                charges.clone()
            } else {
                to_date.quantize(-2, Rounding::HalfUp)?
            };
            let interest = through.sub(&shown)?;
            shown = through;
            let principal = payment.sub(&interest)?;
            let closing = balance.sub(&principal)?;
            let (from, to) = (dates::add_months(start, k - 1), dates::add_months(start, k));
            let paid = if self.in_advance { from } else { to };
            rows.push(json!({
                "instalment": k,
                "period_start": dates::format(from),
                "period_end": dates::format(to),
                "payment_date": dates::format(paid),
                "opening": balance.fixed(2, false),
                "interest": interest.fixed(2, false),
                "payment": self.instalment.fixed(2, false),
                "balloon": balloon.fixed(2, false),
                "principal": principal.fixed(2, false),
                "closing": closing.fixed(2, false),
            }));
            months.push(Month {
                instalment: k,
                from,
                to,
                interest,
            });
            balance = closing;
        }
        Ok((rows, months))
    }

    /// The agreement's terms from the inputs, refusing a nil amount, a term
    /// outside 1 to 360 months or an unknown timing.
    fn from_inputs(
        amount_financed: &Arg,
        instalment: &Arg,
        instalments: &Arg,
        balloon: &Arg,
        timing: &Arg,
    ) -> Result<Self> {
        let (financed, instalment) = (money(amount_financed)?, money(instalment)?);
        if financed.is_zero() || instalment.is_zero() {
            return refuse(
                "hire_purchase.amount_nil",
                "amount_financed and instalment must be more than nil.",
            );
        }
        let balloon = money(balloon)?;
        let count = match instalments.int() {
            Some(Some(n)) if (1..=MOST_INSTALMENTS).contains(&n) => n,
            _ => {
                return refuse(
                    "hire_purchase.instalments_unsupported",
                    format!(
                        "instalments must be an integer from 1 to {MOST_INSTALMENTS}, monthly."
                    ),
                );
            }
        };
        let in_advance = match timing.text() {
            Some("in_arrears") => false,
            Some("in_advance") => true,
            _ => {
                return refuse(
                    "input.timing_invalid",
                    "timing must be in_arrears or in_advance.",
                );
            }
        };
        Ok(Agreement {
            financed,
            instalment,
            count,
            balloon,
            in_advance,
        })
    }
}

/// One schedule month: its instalment number, its period from `from` up to
/// `to` as day numbers, and its interest as shown.
struct Month {
    instalment: i64,
    from: i64,
    to: i64,
    interest: Decimal,
}

/// The income year `day` falls in, as `2026-27`, and the 1 July that ends it.
fn income_year(day: i64) -> (String, i64) {
    let (y, m, _) = dates::to_civil(day);
    let first = if m >= 7 { y } else { y - 1 };
    let ends = dates::parse(&format!("{}-07-01", first + 1)).expect("1 July");
    (format!("{first}-{:02}", (first + 1) % 100), ends)
}

/// The months' interest by income year (ITAA 1997 s 240-60(2)). A month
/// wholly in one income year counts in it. A month spanning 30 June is listed
/// apart, with its days before and from 1 July: s 240-60(2) relates its
/// interest to each year in accordance with generally accepted accounting
/// principles, which this worksheet leaves to the caller.
fn by_income_year(months: &[Month]) -> Result<(Value, Value)> {
    // (income year, first instalment, last instalment, interest)
    type Year = (String, i64, i64, Decimal);
    let (mut years, mut across): (Vec<Year>, Vec<Value>) = (Vec::new(), Vec::new());
    for month in months {
        let (year, ends) = income_year(month.from);
        if month.to > ends {
            across.push(json!({
                "instalment": month.instalment,
                "interest": month.interest.fixed(2, false),
                "income_years": [year, income_year(ends).0],
                "days": [ends - month.from, month.to - ends],
            }));
        } else if let Some(last) = years.last_mut().filter(|last| last.0 == year) {
            last.2 = month.instalment;
            last.3 = last.3.add(&month.interest)?;
        } else {
            years.push((
                year,
                month.instalment,
                month.instalment,
                month.interest.clone(),
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

/// The monthly rate `r` and 12 times it, to 16 places, so the rates
/// reproduce a month's interest to the cent at the largest amount, and
/// fixed-point, so a nil rate reads 0.0000000000000000.
fn rates(r: &Decimal) -> Result<Vec<(String, String)>> {
    precise(|| {
        Ok(vec![
            rate(
                "implicit_monthly_rate",
                r.quantize(-16, Rounding::HalfUp)?.fixed(16, false),
            ),
            rate(
                "nominal_annual_rate",
                r.mul(&int(12))?
                    .quantize(-16, Rounding::HalfUp)?
                    .fixed(16, false),
            ),
        ])
    })
}

#[allow(clippy::too_many_arguments)]
pub fn hire_purchase_schedule(
    amount_financed: &Arg,
    instalment: &Arg,
    instalments: &Arg,
    balloon: &Arg,
    timing: &Arg,
    start_date: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let agreement =
        Agreement::from_inputs(amount_financed, instalment, instalments, balloon, timing)?;
    let start = start_in_year(start_date, &year)?;
    let (r, rows, steps, (years, across)) = precise(|| {
        let r = agreement.implicit_rate()?;
        let count = int(agreement.count);
        let total = agreement.instalment.mul(&count)?.add(&agreement.balloon)?;
        let charges = total.sub(&agreement.financed)?;
        let (rows, months) = agreement.schedule(&r, start, &charges)?;
        let financed = &agreement.financed;
        let steps = steps_from(
            &STEPS,
            [
                (&[financed], financed.clone()),
                (
                    &[&agreement.instalment, &count, &agreement.balloon],
                    total.clone(),
                ),
                (&[&total, financed], charges),
            ],
        );
        Ok((r, rows, steps, by_income_year(&months)?))
    })?;
    let mut out = derived_result(KIND, &year, steps, rates(&r)?)?;
    // The balloon falls due at the end of the term, after an instalment in advance.
    out["balloon_date"] = json!(dates::format(dates::add_months(start, agreement.count)));
    out["schedule"] = Value::Array(rows);
    out["interest_by_income_year"] = years;
    out["interest_across_30_june"] = across;
    Ok(out)
}
