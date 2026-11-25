// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `super-projection`: one accumulation super account projected a whole year
//! at a time from 2026-27 to a retirement age, in future dollars and as a
//! present value, as the site's super projection page does
//! (ryanduguid.github.io `assets/super-projection.mjs`, checked 26 November
//! 2026).
//!
//! The assumed retirement age and the present value follow s 4 of ASIC
//! Corporations (Superannuation Calculators and Retirement Estimates)
//! Instrument 2022/603 (F2022L00895, compilation no. 1 of 11 September 2024):
//! the greater of the age entered and the preservation age, 60 for anyone born
//! after 30 June 1964 (SISR reg 6.01(2), as the ATO's 'When you can withdraw
//! your super' tabulates it), or 67 when none is entered; and C / (1 + r)^n
//! with r = 0.037 unless another rate is entered. Concessional contributions
//! are the fund's assessable income (ITAA 1997 s 295-160 item 1), taxed at 15%
//! (Income Tax Rates Act 1986 s 26(1)(a), compilation no. 66). The employer
//! pays its rate on salary up to the 2026-27 maximum contribution base of
//! $270,830 (ATO 'Super guarantee', Table 23, last updated 17 April 2026), and
//! the 2026-27 caps are $32,500 concessional and $130,000 non-concessional
//! (ATO 'Contributions caps', last updated 11 September 2026). All read on
//! 26 November 2026. Every other figure is the operator's assumption.

use num_bigint::BigInt;
use serde_json::{Value, json};

use super::{Arg, Result, d, money, refuse, result, scope, whole_dollars};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "super_projection";
const PRESERVATION_AGE: i64 = 60;
const DEFAULT_RETIREMENT_AGE: i64 = 67;
const LAST_AGE: i64 = 75;
/// 2026-27, in cents.
const MAXIMUM_CONTRIBUTION_BASE: i128 = 27_083_000;
const CONCESSIONAL_CAP: i128 = 3_250_000;
const NON_CONCESSIONAL_CAP: i128 = 13_000_000;
/// 15%, and the Instrument's default 3.7%, in hundredths of a per cent.
const CONTRIBUTIONS_TAX: i128 = 1500;
const DEFAULT_DISCOUNT: i128 = 370;

/// The worksheet's inputs, as the caller gave them.
pub struct SuperProjectionArgs<'a> {
    pub age: &'a Arg,
    pub retirement_age: Option<&'a Arg>,
    pub balance: &'a Arg,
    pub salary: &'a Arg,
    pub employer_rate: &'a Arg,
    pub salary_sacrifice: &'a Arg,
    pub after_tax_contributions: &'a Arg,
    pub return_rate: &'a Arg,
    pub percent_fee: &'a Arg,
    pub admin_fee: &'a Arg,
    pub insurance: &'a Arg,
    pub wage_growth: &'a Arg,
    pub discount_rate: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One year's flows in cents: they grow with wages after the first year.
#[derive(Clone)]
struct Flows {
    salary: i128,
    base: i128,
    sacrifice: i128,
    after_tax: i128,
    fee: i128,
    insurance: i128,
}

impl Flows {
    fn grown(&self, growth: i128) -> Flows {
        let g = |v: i128| share(v, 10_000 + growth);
        Flows {
            salary: g(self.salary),
            base: g(self.base),
            sacrifice: g(self.sacrifice),
            after_tax: g(self.after_tax),
            fee: g(self.fee),
            insurance: g(self.insurance),
        }
    }
}

/// The employer, return and percentage fee rates, in hundredths of a per cent.
struct Rates {
    employer: i128,
    earn: i128,
    fee: i128,
}

struct Year {
    closing: i128,
    employer: i128,
    concessional: i128,
    contributed: i128,
    earnings: i128,
    fees: i128,
}

/// `num / den` cents, half up, for a non-negative numerator.
fn ratio(num: i128, den: i128) -> i128 {
    (2 * num + den) / (2 * den)
}

/// `amount` cents times `rate` hundredths of a per cent, half up.
fn share(amount: i128, rate: i128) -> i128 {
    ratio(amount * rate, 10_000)
}

/// One year, every flow paid evenly through it and so earning for half the
/// year: earnings and the percentage fee are on the opening balance plus half
/// the year's net flows, and the fund deducts the fees and premiums against
/// its 15% tax. Fees that would overdraw the account take what it holds.
fn project_year(balance: i128, y: &Flows, r: &Rates) -> Year {
    let employer = share(y.salary.min(y.base), r.employer);
    let concessional = employer + y.sacrifice;
    let contributed = concessional - share(concessional, CONTRIBUTIONS_TAX) + y.after_tax;
    let flow = contributed - y.fee - y.insurance;
    let twice_average = (2 * balance + flow).max(0);
    let earnings = ratio(twice_average * r.earn, 20_000);
    let percent_fee = ratio(twice_average * r.fee, 20_000);
    let fees = y.fee + y.insurance + percent_fee;
    let net = fees - share(fees, CONTRIBUTIONS_TAX);
    let available = balance + contributed + earnings;
    let taken = net.min(available);
    Year {
        closing: available - taken,
        employer,
        concessional,
        contributed,
        earnings,
        fees: taken,
    }
}

fn input_invalid<T>(message: String) -> Result<T> {
    refuse("super_projection.input_invalid", message)
}

/// A whole number of years from `low` to `high`.
fn age(value: &Arg, name: &str, low: i64, high: i64) -> Result<i64> {
    match value {
        Arg::Int(n) if (low..=high).contains(n) => Ok(*n),
        _ => input_invalid(format!(
            "{name} must be a whole number from {low} to {high}."
        )),
    }
}

/// A decimal fraction from 0 to `most`, at most 4 places, in hundredths of a
/// per cent.
fn rate(value: &Arg, name: &str, most: &str) -> Result<i128> {
    if let Arg::Decimal(v) = value
        && v.is_finite()
        && !v.lt(&d("0"))?
        && !v.gt(&d(most))?
        && v.exponent() >= -4
    {
        return Ok(digits(&v.quantize(-4, Rounding::HalfUp)?));
    }
    input_invalid(format!(
        "{name} must be a decimal fraction from 0 to {most}, at most 4 places."
    ))
}

/// A checked amount in cents.
fn cents(value: &Arg) -> Result<i128> {
    Ok(digits(&money(value)?.quantize(-2, Rounding::HalfUp)?))
}

/// A non-negative fixed-point Decimal's digits as an integer.
fn digits(v: &Decimal) -> i128 {
    v.to_string()
        .replace('.', "")
        .parse()
        .expect("a checked non-negative amount")
}

fn dollars(cents: i128) -> Decimal {
    d(&format!("{}.{:02}", cents / 100, cents % 100))
}

/// The Instrument's assumed retirement age, refused unless after `now`.
fn retirement(now: i64, given: Option<&Arg>) -> Result<i64> {
    let retire = match given {
        Some(v) => age(v, "retirement_age", 15, LAST_AGE)?.max(PRESERVATION_AGE),
        None if now < DEFAULT_RETIREMENT_AGE => DEFAULT_RETIREMENT_AGE,
        None => now + 1,
    };
    if retire <= now {
        return refuse(
            "super_projection.retirement_not_after_age",
            "The assumed retirement age is not after the age now; this projection does not cover an account already paying a pension.",
        );
    }
    Ok(retire)
}

fn first_flows(a: &SuperProjectionArgs) -> Result<Flows> {
    let salary = money(a.salary)?;
    whole_dollars(&salary, "salary")?;
    Ok(Flows {
        salary: cents(a.salary)?,
        base: MAXIMUM_CONTRIBUTION_BASE,
        sacrifice: cents(a.salary_sacrifice)?,
        after_tax: cents(a.after_tax_contributions)?,
        fee: cents(a.admin_fee)?,
        insurance: cents(a.insurance)?,
    })
}

/// The caps for 2026-27 only: the projection assumes later caps rise with wages.
fn within_caps(first: &Year, y: &Flows) -> Result<()> {
    if first.concessional > CONCESSIONAL_CAP {
        return refuse(
            "super_projection.cap_exceeded",
            "Concessional contributions for 2026-27 are above the $32,500 cap; this projection does not compute excess contributions.",
        );
    }
    if y.after_tax > NON_CONCESSIONAL_CAP {
        return refuse(
            "super_projection.cap_exceeded",
            "After-tax contributions for 2026-27 are above the $130,000 cap; this projection does not compute excess contributions.",
        );
    }
    Ok(())
}

/// `balance` cents / (1 + discount)^years, half up.
fn present_value(balance: i128, discount: i128, years: u32) -> Decimal {
    let num = BigInt::from(balance) * BigInt::from(10_000).pow(years);
    let den = BigInt::from(10_000 + discount).pow(years);
    let two = BigInt::from(2);
    let pv: BigInt = (&two * num + &den) / (two * den);
    dollars(i128::try_from(pv).expect("no more than the balance"))
}

/// The account at the assumed retirement age.
pub fn super_projection(a: &SuperProjectionArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let now = age(a.age, "age", 15, LAST_AGE - 1)?;
    let retire = retirement(now, a.retirement_age)?;
    let rates = Rates {
        employer: rate(a.employer_rate, "employer_rate", "0.3")?,
        earn: rate(a.return_rate, "return_rate", "0.2")?,
        fee: rate(a.percent_fee, "percent_fee", "0.05")?,
    };
    let growth = rate(a.wage_growth, "wage_growth", "0.1")?;
    let discount = match a.discount_rate {
        Some(v) => rate(v, "discount_rate", "0.1")?,
        None => DEFAULT_DISCOUNT,
    };
    let mut balance = cents(a.balance)?;
    let mut y = first_flows(a)?;
    let first = project_year(balance, &y, &rates);
    within_caps(&first, &y)?;
    let (mut contributed, mut earnings, mut fees) = (0, 0, 0);
    for n in 0..retire - now {
        if n > 0 {
            y = y.grown(growth);
        }
        let r = project_year(balance, &y, &rates);
        balance = r.closing;
        contributed += r.contributed;
        earnings += r.earnings;
        fees += r.fees;
    }
    let years = u32::try_from(retire - now).expect("1 to 60 years");
    let amounts = vec![
        ("employer_contributions_2026_27", dollars(first.employer)),
        (
            "concessional_contributions_2026_27",
            dollars(first.concessional),
        ),
        ("balance_at_retirement", dollars(balance)),
        ("present_value", present_value(balance, discount, years)),
        ("contributions_less_tax", dollars(contributed)),
        ("earnings", dollars(earnings)),
        ("fees_and_premiums", dollars(fees)),
    ];
    let mut out = result(KIND, &year, amounts, vec![])?;
    out["years"] = json!(years);
    out["retirement_age"] = json!(retire);
    Ok(out)
}
