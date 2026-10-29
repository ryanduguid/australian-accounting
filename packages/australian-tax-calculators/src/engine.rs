//! Explicitly bounded worksheets using Decimal and published official methods,
//! ported from `austaxcalc.calculations` without changing a rate, rounding
//! step, scope statement or message. Every worksheet computes in a 40-digit
//! context, as the original's `localcontext` did.

use serde_json::{Map, Value, json};

use crate::decimal::{Calc, Decimal, Rounding, Signal, with_precision};
use crate::metadata::{self as meta, CoefficientRow, Example};
use crate::pytext;
use crate::texts;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const WORKING_PRECISION: i128 = 40;

/// A refusal: Python's ValueError with its message, or a trapped decimal signal.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Value(String),
    Decimal(Signal),
}

impl From<Signal> for Error {
    fn from(signal: Signal) -> Error {
        Error::Decimal(signal)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Value(m) => f.write_str(m),
            Error::Decimal(s) => write!(f, "decimal.{}", s.name()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn refuse<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::Value(message.into()))
}

pub fn d(text: &str) -> Decimal {
    Decimal::parse(text).expect("decimal literal")
}

fn int(n: i64) -> Decimal {
    Decimal::from_int(n)
}

/// An argument as a caller passed it: the original checks types itself.
#[derive(Clone, Debug)]
pub enum Arg {
    Decimal(Decimal),
    Bool(bool),
    Int(i64),
    /// An integer outside the 64-bit range: still an `int`, never in range.
    BigInt(String),
    Text(String),
    /// Any other value: a float, None, a list or a mapping.
    Other(String),
}

impl Arg {
    /// `repr(value)`.
    fn repr(&self) -> String {
        match self {
            Arg::Decimal(d) => format!("Decimal('{d}')"),
            Arg::Bool(true) => "True".into(),
            Arg::Bool(false) => "False".into(),
            Arg::Int(n) => n.to_string(),
            Arg::BigInt(s) | Arg::Other(s) => s.clone(),
            Arg::Text(s) => pytext::repr(s),
        }
    }

    /// `str(value)`.
    fn py_str(&self) -> String {
        match self {
            Arg::Text(s) => s.clone(),
            Arg::Decimal(d) => d.to_string(),
            other => other.repr(),
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            Arg::Text(s) => Some(s),
            _ => None,
        }
    }

    /// `type(value) is int`: a bool is not an int here.
    fn int(&self) -> Option<Option<i64>> {
        match self {
            Arg::Int(n) => Some(Some(*n)),
            Arg::BigInt(_) => Some(None),
            _ => None,
        }
    }
}

fn py_min(a: Decimal, b: Decimal) -> Calc<Decimal> {
    Ok(if b.lt(&a)? { b } else { a })
}

fn py_max(a: Decimal, b: Decimal) -> Calc<Decimal> {
    Ok(if b.gt(&a)? { b } else { a })
}

fn lookup<'a, T>(table: &'a [(&str, T)], key: &str) -> &'a T {
    &table
        .iter()
        .find(|(k, _)| *k == key)
        .expect("validated key")
        .1
}

fn sources(kind: &str) -> Vec<&'static str> {
    let mut out = vec![*lookup(texts::SOURCES, kind)];
    if let Some((_, extra)) = texts::EXTRA_SOURCES.iter().find(|(k, _)| *k == kind) {
        out.extend(extra.iter().copied());
    }
    out
}

/// `_money`: a finite non-negative Decimal at most 2dp and 1e12.
fn money(value: &Arg) -> Result<Decimal> {
    let bad = || refuse("Amounts must be finite non-negative Decimals, at most 2dp and 1e12.");
    let Arg::Decimal(v) = value else {
        return bad();
    };
    if !v.is_finite() || v.lt(&int(0))? || v.gt(&d("1e12"))? || v.exponent() < -2 {
        return bad();
    }
    Ok(v.clone())
}

/// `_scope`: confirmation, then a supported period.
fn scope(confirmed: &Arg, year: &Arg, kind: &str) -> Result<String> {
    if !matches!(confirmed, Arg::Bool(true)) {
        return refuse("Establish all stated scope conditions before calculating.");
    }
    let supported = meta::supported_periods(kind);
    match year.text() {
        Some(y) if supported.contains(&y) => Ok(y.to_string()),
        _ => refuse(format!(
            "Unsupported period {}; supported: {}.",
            year.repr(),
            supported.join(", ")
        )),
    }
}

fn whole_dollars(income: &Decimal, what: &str) -> Result<()> {
    if !income.equals(&income.to_integral_value()?)? {
        return refuse(format!("Supply established whole-dollar {what}."));
    }
    Ok(())
}

/// `_result`: amounts rendered to cents half up, in a 40-digit context.
fn result(
    kind: &str,
    period: &str,
    amounts: Vec<(&str, Decimal)>,
    rates: Vec<(String, String)>,
) -> Result<Value> {
    let mut rendered = Map::new();
    for (key, value) in amounts {
        let cents = with_precision(WORKING_PRECISION, || value.quantize(-2, Rounding::HalfUp))?;
        rendered.insert(key.to_string(), json!(cents.to_string()));
    }
    let rates: Map<String, Value> = rates.into_iter().map(|(k, v)| (k, json!(v))).collect();
    let mut out = Map::new();
    out.insert("ok".into(), json!(true));
    out.insert("engine".into(), json!("australian-tax-calculators"));
    out.insert("engine_version".into(), json!(VERSION));
    out.insert("calculation".into(), json!(kind));
    out.insert("period".into(), json!(period));
    out.insert("amounts".into(), Value::Object(rendered));
    out.insert("rates".into(), Value::Object(rates));
    out.insert("scope".into(), json!(lookup(texts::SCOPES, kind)));
    out.insert("sources".into(), json!(sources(kind)));
    out.insert("source_checked".into(), json!(meta::source_review(kind).0));
    out.insert(
        "warnings".into(),
        json!([
            "Operator-confirmed facts and scope; eligibility is not independently verified.",
            "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
        ]),
    );
    Ok(Value::Object(out))
}

fn rate(key: &str, value: impl ToString) -> (String, String) {
    (key.to_string(), value.to_string())
}

/// Run `f` in the worksheets' 40-digit context.
fn precise<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    with_precision(WORKING_PRECISION, f)
}

pub fn gst(amount: &Arg, gst_inclusive: &Arg, scope_confirmed: &Arg, year: &Arg) -> Result<Value> {
    let year = scope(scope_confirmed, year, "gst")?;
    let amount = money(amount)?;
    let Arg::Bool(inclusive) = gst_inclusive else {
        return refuse("gst_inclusive must be a boolean.");
    };
    let (tax, exclusive) = precise(|| {
        if *inclusive {
            let tax = amount.div(&int(11))?;
            Ok((tax.clone(), amount.sub(&tax)?))
        } else {
            Ok((amount.div(&int(10))?, amount.clone()))
        }
    })?;
    let total = precise(|| Ok(exclusive.add(&tax)?))?;
    result(
        "gst",
        &year,
        vec![("gst", tax), ("exclusive", exclusive), ("inclusive", total)],
        vec![rate("gst_rate", "0.10")],
    )
}

pub fn resident_tax(taxable_income: &Arg, year: &Arg, scope_confirmed: &Arg) -> Result<Value> {
    let year = scope(scope_confirmed, year, "resident_tax")?;
    let income = money(taxable_income)?;
    whole_dollars(&income, "taxable income")?;
    let scale = lookup(meta::RESIDENT_TAX_SCALES, &year);
    let tax = precise(|| {
        let mut tax = int(0);
        for (low, high, rate) in scale.iter() {
            let ceiling = match high {
                None => income.clone(),
                Some(high) => py_min(income.clone(), int(*high))?,
            };
            let band = py_max(int(0), ceiling.sub(&int(*low))?)?;
            tax = tax.add(&band.mul(&d(rate))?)?;
        }
        Ok(tax)
    })?;
    result(
        "resident_tax",
        &year,
        vec![("taxable_income_used", income), ("basic_income_tax", tax)],
        scale
            .iter()
            .map(|(low, _, r)| rate(&format!("over_{low}"), r))
            .collect(),
    )
}

pub fn capital_gains(
    other_gains: &Arg,
    discount_gains: &Arg,
    current_losses: &Arg,
    prior_losses: &Arg,
    scope_confirmed: &Arg,
    year: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "capital_gains")?;
    let other = money(other_gains)?;
    let discount = money(discount_gains)?;
    let current = money(current_losses)?;
    let prior = money(prior_losses)?;
    let amounts = precise(|| {
        let losses = current.add(&prior)?;
        let other_offset = py_min(other.clone(), losses.clone())?;
        let discount_offset = py_min(discount.clone(), losses.sub(&other_offset)?)?;
        let discounted = discount.sub(&discount_offset)?.div(&int(2))?;
        Ok(vec![
            ("losses_used", other_offset.add(&discount_offset)?),
            (
                "losses_remaining",
                losses.sub(&other_offset)?.sub(&discount_offset)?,
            ),
            (
                "net_capital_gain",
                other.sub(&other_offset)?.add(&discounted)?,
            ),
            ("discount_applied", discounted),
        ])
    })?;
    result(
        "capital_gains",
        &year,
        amounts,
        vec![rate("discount", "0.50")],
    )
}

pub fn fbt(
    type_one_value: &Arg,
    type_two_value: &Arg,
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    scope(scope_confirmed, &Arg::Text(year_ended.py_str()), "fbt").map_err(|e| match e {
        // `_scope` reports the period as the original's `str(year_ended)`.
        Error::Value(m) => Error::Value(m),
        other => other,
    })?;
    let amounts = precise(|| {
        let first = money(type_one_value)?.mul(&d("2.0802"))?;
        let second = money(type_two_value)?.mul(&d("1.8868"))?;
        let item_14a = first.quantize(0, Rounding::HalfUp)?;
        let item_14b = second.quantize(0, Rounding::HalfUp)?;
        let item_15 = item_14a.add(&item_14b)?;
        Ok(vec![
            ("type_one_grossed_up", first.clone()),
            ("type_two_grossed_up", second.clone()),
            ("fbt_estimate", first.add(&second)?.mul(&d("0.47"))?),
            ("return_item_14a", item_14a),
            ("return_item_14b", item_14b),
            ("return_item_15", item_15.clone()),
            ("return_item_16", item_15.mul(&d("0.47"))?),
        ])
    })?;
    result(
        "fbt",
        "year ended 31 March 2026",
        amounts,
        vec![
            rate("type_one", "2.0802"),
            rate("type_two", "1.8868"),
            rate("fbt_rate", "0.47"),
        ],
    )
}

/// A car worksheet's FBT year: `year_ended` must be an integer. Returns the
/// period id, its label and the days in the year (366 when it holds 29 February).
fn fbt_year(confirmed: &Arg, year_ended: &Arg, kind: &str) -> Result<(String, String, i64)> {
    // Only a supported integer passes; anything else is quoted as given.
    let period = match year_ended.int() {
        Some(Some(n)) if meta::supported_periods(kind).contains(&n.to_string().as_str()) => {
            Arg::Text(n.to_string())
        }
        _ => Arg::Other(year_ended.repr()),
    };
    let year = scope(confirmed, &period, kind)?;
    let ended: i64 = year.parse().expect("FBT year");
    let days = if is_leap(ended) { 366 } else { 365 };
    Ok((year, format!("year ended 31 March {ended}"), days))
}

fn day_count(value: &Arg, days_in_year: i64, name: &str) -> Result<i64> {
    match value.int() {
        Some(Some(n)) if (0..=days_in_year).contains(&n) => Ok(n),
        _ => refuse(format!(
            "{name} must be an integer from 0 to {days_in_year}."
        )),
    }
}

fn flag(value: &Arg, name: &str) -> Result<bool> {
    match value {
        Arg::Bool(b) => Ok(*b),
        _ => refuse(format!("{name} must be a boolean.")),
    }
}

fn cents(value: Decimal) -> Calc<Decimal> {
    value.quantize(-2, Rounding::HalfUp)
}

/// The steps after a car's gross taxable value. The gross value is rounded to
/// cents first: the recipient's payment is in cents, so this moves no taxable
/// value and keeps the applied and excess parts adding up. The payment cannot
/// take the value below nil. Under the operating cost method, s 10(5) then
/// substitutes the statutory formula value when it is lower. An exempt electric
/// car's taxable value is nil; the value before the exemption is the notional
/// value its reportable fringe benefits amount is worked out from.
fn car_taxable_values(
    gross: Decimal,
    contribution: &Decimal,
    statutory: Option<&Decimal>,
    exempt: bool,
) -> Calc<Vec<(&'static str, Decimal)>> {
    let gross = cents(gross)?;
    let applied = py_min(contribution.clone(), gross.clone())?;
    let mut value = gross.sub(&applied)?;
    let mut out = vec![
        ("gross_taxable_value", gross),
        ("employee_contribution_applied", applied.clone()),
        ("excess_employee_contribution", contribution.sub(&applied)?),
    ];
    if let Some(statutory) = statutory {
        out.push(("operating_cost_taxable_value", value.clone()));
        out.push(("statutory_formula_taxable_value", statutory.clone()));
        value = py_min(value, statutory.clone())?;
    }
    out.push(("taxable_value_before_exemption", value.clone()));
    out.push(("taxable_value", if exempt { int(0) } else { value }));
    Ok(out)
}

/// FBTAA s 9: `(A x B x C) / D - E` on the operator's established base value.
#[allow(clippy::too_many_arguments)]
pub fn fbt_car_statutory(
    cost_price: &Arg,
    later_accessories: &Arg,
    one_third_reduction: &Arg,
    days_available: &Arg,
    employee_contribution: &Arg,
    electric_car_exempt: &Arg,
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_car_statutory";
    let (_, period, days_in_year) = fbt_year(scope_confirmed, year_ended, kind)?;
    let cost = money(cost_price)?;
    let accessories = money(later_accessories)?;
    let reduce = flag(one_third_reduction, "one_third_reduction")?;
    let days = day_count(days_available, days_in_year, "days_available")?;
    let contribution = money(employee_contribution)?;
    let exempt = flag(electric_car_exempt, "electric_car_exempt")?;
    let amounts = precise(|| {
        // The one-third reduction never reaches accessories added after acquisition.
        let reduced = if reduce {
            cost.sub(&cost.div(&int(3))?)?
        } else {
            cost.clone()
        };
        let base = reduced.add(&accessories)?;
        let gross = base
            .mul(&d("0.20"))?
            .mul(&int(days))?
            .div(&int(days_in_year))?;
        let mut out = vec![("base_value", base)];
        out.extend(car_taxable_values(gross, &contribution, None, exempt)?);
        Ok(out)
    })?;
    result(
        kind,
        &period,
        amounts,
        vec![
            rate("statutory_percentage", "0.20"),
            rate("days_available", days),
            rate("days_in_year", days_in_year),
        ],
    )
}

/// FBTAA s 10: `C x (100% - BP) - R`, with deemed depreciation and interest on
/// an owned car's depreciated value (ss 11 and 12), apportioned to the days in
/// the holding period it was owned, and the s 10(5) statutory comparison.
#[allow(clippy::too_many_arguments)]
pub fn fbt_car_operating_cost(
    running_costs: &Arg,
    lease_charges: &Arg,
    depreciated_value: &Arg,
    days_held: &Arg,
    business_use: &Arg,
    employee_contribution: &Arg,
    statutory_formula_taxable_value: &Arg,
    electric_car_exempt: &Arg,
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_car_operating_cost";
    let (year, period, days_in_year) = fbt_year(scope_confirmed, year_ended, kind)?;
    let running = money(running_costs)?;
    let lease = money(lease_charges)?;
    let value = money(depreciated_value)?;
    if lease.gt(&int(0))? && value.gt(&int(0))? {
        return refuse(
            "A car is either owned, with a depreciated_value, or leased, with lease_charges; set the other to 0.",
        );
    }
    let days = day_count(days_held, days_in_year, "days_held")?;
    let Some(business) = finite_between(business_use, "0", "1")? else {
        return refuse("business_use must be a finite Decimal fraction from 0 to 1.");
    };
    let contribution = money(employee_contribution)?;
    let statutory = money(statutory_formula_taxable_value)?;
    let exempt = flag(electric_car_exempt, "electric_car_exempt")?;
    let interest_rate = d(lookup(meta::FBT_BENCHMARK_INTEREST_RATES, &year));
    let private = precise(|| Ok(int(1).sub(&business)?))?;
    let amounts = precise(|| {
        let deemed = |rate: &Decimal| value.mul(rate)?.mul(&int(days))?.div(&int(days_in_year));
        let depreciation = deemed(&d("0.25"))?;
        let interest = deemed(&interest_rate)?;
        let costs = running.add(&lease)?.add(&depreciation)?.add(&interest)?;
        let gross = costs.mul(&private)?;
        let mut out = vec![
            ("deemed_depreciation", depreciation),
            ("deemed_interest", interest),
            ("operating_costs", costs),
        ];
        out.extend(car_taxable_values(
            gross,
            &contribution,
            Some(&statutory),
            exempt,
        )?);
        Ok(out)
    })?;
    result(
        kind,
        &period,
        amounts,
        vec![
            rate("depreciation_rate", "0.25"),
            rate("benchmark_interest_rate", &interest_rate),
            rate("days_held", days),
            rate("days_in_year", days_in_year),
            rate("private_use", &private),
        ],
    )
}

fn finite_between(value: &Arg, low: &str, high: &str) -> Calc<Option<Decimal>> {
    let Arg::Decimal(v) = value else {
        return Ok(None);
    };
    if !v.is_finite() || !(d(low).le(v)? && v.le(&d(high))?) {
        return Ok(None);
    }
    Ok(Some(v.clone()))
}

#[allow(clippy::too_many_arguments)]
pub fn depreciation(
    cost: &Arg,
    effective_life: &Arg,
    days: &Arg,
    taxable_use: &Arg,
    method: &Arg,
    scope_confirmed: &Arg,
    year: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "depreciation")?;
    let cost = money(cost)?;
    let Some(life) = finite_between(effective_life, "0.01", "1000")? else {
        return refuse("effective_life must be a finite Decimal between 0.01 and 1000 years.");
    };
    let Some(use_) = finite_between(taxable_use, "0", "1")? else {
        return refuse("taxable_use must be a finite Decimal fraction from 0 to 1.");
    };
    let Some(Some(days)) = days
        .int()
        .filter(|n| n.is_some_and(|n| (0..=365).contains(&n)))
    else {
        return refuse("days must be an integer from 0 to 365 for 2025-26.");
    };
    let factor = match method.text() {
        Some("prime_cost") => int(1),
        Some("diminishing_value") => int(2),
        _ => return refuse("method must be prime_cost or diminishing_value."),
    };
    let amounts = precise(|| {
        let raw = cost
            .mul(&int(days))?
            .div(&int(365))?
            .mul(&factor)?
            .div(&life)?;
        let decline = py_min(cost.clone(), raw)?;
        Ok(vec![
            ("decline_in_value", decline.clone()),
            ("deduction", decline.mul(&use_)?),
            ("closing_adjustable_value", cost.sub(&decline)?),
        ])
    })?;
    result("depreciation", &year, amounts, Vec::new())
}

pub fn quarterly_sg(
    ordinary_time_earnings: &Arg,
    qualifying_contributions: &Arg,
    year: &Arg,
    quarter: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "quarterly_sg")?;
    let Some(Some(quarter)) = quarter
        .int()
        .filter(|n| n.is_some_and(|n| (1..=4).contains(&n)))
    else {
        return refuse("quarter must be 1 (Jul-Sep), 2, 3 or 4 (Apr-Jun).");
    };
    let amounts = precise(|| {
        let earnings = py_min(money(ordinary_time_earnings)?, d("62500"))?;
        let paid = money(qualifying_contributions)?;
        let minimum = earnings.mul(&d("0.12"))?;
        Ok(vec![
            ("earnings_used", earnings),
            ("minimum_sg", minimum.clone()),
            (
                "additional_contribution",
                py_max(int(0), minimum.sub(&paid)?)?,
            ),
        ])
    })?;
    result(
        "quarterly_sg",
        &format!("{year} Q{quarter}"),
        amounts,
        vec![
            rate("sg_rate", "0.12"),
            rate("maximum_quarterly_base", "62500.00"),
        ],
    )
}

fn payg_rows(year: &str, scale: i64) -> Option<&'static [CoefficientRow]> {
    let scales = lookup(meta::PAYG_WITHHOLDING_COEFFICIENTS, year);
    scales
        .iter()
        .find(|(s, _)| *s == scale)
        .map(|(_, rows)| *rows)
}

/// `_payg_checks`: scope, earnings, pay period and scale, in that order.
fn payg_checks(
    kind: &str,
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<(String, Decimal, String, i64)> {
    let year = scope(scope_confirmed, year, kind)?;
    let earnings = money(earnings)?;
    let period = match pay_period.text() {
        Some(p @ ("weekly" | "fortnightly" | "monthly")) => p.to_string(),
        _ => return refuse("pay_period must be weekly, fortnightly or monthly."),
    };
    let Some(Some(scale)) = scale.int() else {
        return refuse("scale must be 1, 2, 3, 5 or 6; scale 4 is not supported.");
    };
    if payg_rows(&year, scale).is_none() {
        return refuse("scale must be 1, 2, 3, 5 or 6; scale 4 is not supported.");
    }
    Ok((year, earnings, period, scale))
}

/// Schedule 1 and 8: the weekly equivalent in whole dollars plus 99 cents.
fn weekly_earnings(earnings: &Decimal, pay_period: &str) -> Calc<Decimal> {
    let weekly = match pay_period {
        "weekly" => earnings.clone(),
        "fortnightly" => earnings.div(&int(2))?,
        _ => {
            // Schedule 1: a monthly amount ending in 33 cents gains a cent first.
            let mut earnings = earnings.clone();
            if earnings.rem(&int(1))?.equals(&d("0.33"))? {
                earnings = earnings.add(&d("0.01"))?;
            }
            earnings.mul(&int(3))?.div(&int(13))?
        }
    };
    weekly.quantize(0, Rounding::Down)?.add(&d("0.99"))
}

/// y = a * x - b on the first applicable row, rounded straight to the dollar.
fn weekly_formula(x: &Decimal, rows: &[CoefficientRow]) -> Calc<(Decimal, Decimal, Decimal)> {
    let mut chosen = None;
    for (limit, a, b) in rows {
        let applies = match limit {
            None => true,
            Some(limit) => x.lt(&int(*limit))?,
        };
        if applies {
            chosen = Some((d(a), d(b)));
            break;
        }
    }
    let (a, b) = chosen.expect("the last row has no limit");
    let amount = py_max(int(0), a.mul(x)?.sub(&b)?.quantize(0, Rounding::HalfUp)?)?;
    Ok((a, b, amount))
}

fn for_pay_period(weekly_amount: &Decimal, pay_period: &str) -> Calc<Decimal> {
    match pay_period {
        "weekly" => Ok(weekly_amount.clone()),
        "fortnightly" => weekly_amount.mul(&int(2)),
        _ => weekly_amount
            .mul(&int(13))?
            .div(&int(3))?
            .quantize(0, Rounding::HalfUp),
    }
}

pub fn payg_withholding(
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let (year, earnings, period, scale) = payg_checks(
        "payg_withholding",
        earnings,
        pay_period,
        scale,
        year,
        scope_confirmed,
    )?;
    let rows = payg_rows(&year, scale).expect("checked scale");
    let (amounts, a, b) = precise(|| {
        let x = weekly_earnings(&earnings, &period)?;
        let (a, b, weekly) = weekly_formula(&x, rows)?;
        let withholding = for_pay_period(&weekly, &period)?;
        Ok((
            vec![
                ("weekly_earnings_used", x),
                ("weekly_withholding", weekly),
                ("withholding", withholding),
            ],
            a,
            b,
        ))
    })?;
    result(
        "payg_withholding",
        &format!("{year} {period}"),
        amounts,
        vec![rate("scale", scale), rate("a", a), rate("b", b)],
    )
}

pub fn study_loan_withholding(
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let (year, earnings, period, scale) = payg_checks(
        "study_loan_withholding",
        earnings,
        pay_period,
        scale,
        year,
        scope_confirmed,
    )?;
    let table = if scale == 1 {
        "no_threshold"
    } else {
        "threshold_or_foreign"
    };
    let tax_rows = payg_rows(&year, scale).expect("checked scale");
    let loan_rows = lookup(
        lookup(meta::STUDY_LOAN_WITHHOLDING_COEFFICIENTS, &year),
        table,
    );
    let (amounts, rates) = precise(|| {
        let x = weekly_earnings(&earnings, &period)?;
        let (a, b, weekly_tax) = weekly_formula(&x, tax_rows)?;
        let (loan_a, loan_b, weekly_loan) = weekly_formula(&x, loan_rows)?;
        let tax = for_pay_period(&weekly_tax, &period)?;
        let component = for_pay_period(&weekly_loan, &period)?;
        let total = tax.add(&component)?;
        Ok((
            vec![
                ("weekly_earnings_used", x),
                ("schedule_1_withholding", tax),
                ("study_loan_component", component),
                ("withholding", total),
            ],
            vec![
                rate("scale", scale),
                rate("a", a),
                rate("b", b),
                rate("component_a", loan_a),
                rate("component_b", loan_b),
            ],
        ))
    })?;
    result(
        "study_loan_withholding",
        &format!("{year} {period}"),
        amounts,
        rates,
    )
}

pub fn medicare_levy(
    taxable_income: &Arg,
    section_9a_adjustment_applies: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "medicare_levy")?;
    let income = money(taxable_income)?;
    whole_dollars(&income, "taxable income")?;
    let Arg::Bool(adjusted) = section_9a_adjustment_applies else {
        return refuse("section_9a_adjustment_applies must be a boolean.");
    };
    if *adjusted {
        let extra = texts::EXTRA_SOURCES
            .iter()
            .find(|(k, _)| *k == "medicare_levy")
            .expect("medicare sources")
            .1[2];
        return refuse(format!(
            "Medicare Levy Act 1986 s 9A (lump sum payments in arrears) applies, and this \
             worksheet does not make that adjustment. See {extra}"
        ));
    }
    let (threshold, limit) = lookup(meta::MEDICARE_LOW_INCOME_THRESHOLDS, &year);
    let (threshold, limit) = (d(threshold), d(limit));
    let levy_rate = d(lookup(&meta::MEDICARE_LEVY_RATES, "levy"));
    let shade_in = d(lookup(&meta::MEDICARE_LEVY_RATES, "shade_in"));
    let amounts = precise(|| {
        let full = income.mul(&levy_rate)?;
        let levy = if income.le(&threshold)? {
            int(0)
        } else if income.le(&limit)? {
            py_min(full.clone(), income.sub(&threshold)?.mul(&shade_in)?)?
        } else {
            full.clone()
        };
        Ok(vec![
            ("taxable_income_used", income.clone()),
            ("levy_at_full_rate", full.clone()),
            ("low_income_reduction", full.sub(&levy)?),
            ("medicare_levy", levy),
        ])
    })?;
    result(
        "medicare_levy",
        &year,
        amounts,
        vec![
            rate("levy_rate", &levy_rate),
            rate("shade_in_rate", &shade_in),
            rate("threshold_amount", &threshold),
            rate("phase_in_limit", &limit),
        ],
    )
}

pub fn low_income_tax_offset(
    taxable_income: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "low_income_tax_offset")?;
    let income = money(taxable_income)?;
    whole_dollars(&income, "taxable income")?;
    let rule = |key: &str| d(lookup(&meta::LOW_INCOME_TAX_OFFSET, key));
    let offset = precise(|| {
        Ok(if income.le(&rule("first_step"))? {
            rule("maximum")
        } else if income.le(&rule("second_step"))? {
            rule("maximum").sub(&income.sub(&rule("first_step"))?.mul(&rule("first_rate"))?)?
        } else if income.le(&rule("cut_out"))? {
            // At the cut-out the table gives minus half a cent; the offset is nil.
            py_max(
                int(0),
                rule("second_amount").sub(
                    &income
                        .sub(&rule("second_step"))?
                        .mul(&rule("second_rate"))?,
                )?,
            )?
        } else {
            int(0)
        })
    })?;
    result(
        "low_income_tax_offset",
        &year,
        vec![
            ("taxable_income_used", income),
            ("low_income_tax_offset", offset),
        ],
        meta::LOW_INCOME_TAX_OFFSET
            .iter()
            .map(|(k, v)| rate(k, v))
            .collect(),
    )
}

pub fn division_293(
    division_293_income: &Arg,
    low_tax_contributions: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "division_293")?;
    let income = money(division_293_income)?;
    let contributions = money(low_tax_contributions)?;
    let threshold = d(lookup(&meta::DIVISION_293, "threshold"));
    let rate_ = d(lookup(&meta::DIVISION_293, "rate"));
    let amounts = precise(|| {
        let combined = income.add(&contributions)?;
        let excess = py_max(int(0), combined.sub(&threshold)?)?;
        let taxable = py_min(contributions.clone(), excess.clone())?;
        Ok(vec![
            ("combined_income_and_contributions", combined),
            ("excess_over_threshold", excess),
            ("taxable_contributions", taxable.clone()),
            ("division_293_tax", taxable.mul(&rate_)?),
        ])
    })?;
    result(
        "division_293",
        &year,
        amounts,
        vec![rate("threshold", &threshold), rate("rate", &rate_)],
    )
}

pub fn study_loan_repayment(
    repayment_income: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "study_loan_repayment")?;
    let income = money(repayment_income)?;
    whole_dollars(&income, "repayment income")?;
    let thresholds = lookup(meta::STUDY_LOAN_REPAYMENT_THRESHOLDS, &year);
    let get = |key: &str| *lookup(thresholds, key);
    let (minimum, step) = (d(get("minimum")), d(get("step")));
    let rates = |key: &str| d(lookup(&meta::STUDY_LOAN_REPAYMENT_RATES, key));
    let (lower, upper, flat) = (rates("lower"), rates("upper"), rates("flat"));
    let (amounts, base, rate_applied) = precise(|| {
        let base = step.sub(&minimum)?.mul(&lower)?;
        let marginal = py_max(int(0), py_min(income.clone(), step.clone())?.sub(&minimum)?)?
            .mul(&lower)?
            .add(&py_max(int(0), income.sub(&step)?)?.mul(&upper)?)?;
        let whole = income.mul(&flat)?;
        let (rate_applied, repayment) = if income.le(&minimum)? {
            (int(0), int(0))
        } else if whole.le(&marginal)? {
            (flat.clone(), whole.clone())
        } else if income.le(&step)? {
            (lower.clone(), marginal.clone())
        } else {
            (upper.clone(), marginal.clone())
        };
        Ok((
            vec![
                ("repayment_income_used", income.clone()),
                ("marginal_amount", marginal),
                ("ten_percent_of_income", whole),
                ("compulsory_repayment", repayment),
            ],
            base,
            rate_applied,
        ))
    })?;
    result(
        "study_loan_repayment",
        &year,
        amounts,
        vec![
            rate("rate_applied", &rate_applied),
            rate("minimum_repayment_income", &minimum),
            rate("lower_rate", &lower),
            rate("lower_band_top", &step),
            rate("upper_band_base", &base),
            rate("upper_band_base_as_printed", get("printed_base")),
            rate("upper_rate", &upper),
            rate("flat_rate", &flat),
            rate("flat_from_as_printed", get("flat_from")),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn contribution_caps(
    total_super_balance: &Arg,
    concessional_contributions: &Arg,
    unused_concessional_cap: &Arg,
    non_concessional_contributions: &Arg,
    under_75_in_year: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "contribution_caps")?;
    let balance = money(total_super_balance)?;
    let concessional = money(concessional_contributions)?;
    let unused = money(unused_concessional_cap)?;
    let non_concessional = money(non_concessional_contributions)?;
    let Arg::Bool(under_75) = under_75_in_year else {
        return refuse("under_75_in_year must be a boolean.");
    };
    let (cap, transfer) = lookup(meta::CONTRIBUTION_CAPS, &year);
    let (amounts, rates) = precise(|| {
        let (cap, transfer) = (d(cap), d(transfer));
        let annual = cap.mul(&int(4))?;
        let carried = if balance.lt(&d(meta::CARRY_FORWARD_BALANCE_LIMIT))? {
            unused.clone()
        } else {
            int(0)
        };
        let available_concessional = cap.add(&carried)?;
        // s 292-85: nil at or above the transfer balance cap; otherwise the
        // balance band sets how many annual caps the first year can bring forward.
        let multiple: i64 = if balance.compare(&transfer)?.is_ge() {
            0
        } else if !under_75 || balance.compare(&transfer.sub(&annual)?)?.is_ge() {
            1
        } else if balance
            .compare(&transfer.sub(&annual.mul(&int(2))?)?)?
            .is_ge()
        {
            2
        } else {
            3
        };
        let available = annual.mul(&int(multiple))?;
        let triggered = multiple > 1 && non_concessional.gt(&annual)?;
        Ok((
            vec![
                ("carry_forward_applied", carried),
                ("concessional_available", available_concessional.clone()),
                (
                    "concessional_remaining",
                    py_max(int(0), available_concessional.sub(&concessional)?)?,
                ),
                (
                    "excess_concessional",
                    py_max(int(0), concessional.sub(&available_concessional)?)?,
                ),
                ("non_concessional_available", available.clone()),
                (
                    "non_concessional_remaining",
                    py_max(int(0), available.sub(&non_concessional)?)?,
                ),
                (
                    "excess_non_concessional",
                    py_max(int(0), non_concessional.sub(&available)?)?,
                ),
            ],
            vec![
                rate("concessional_cap", &cap),
                rate("non_concessional_cap", &annual),
                rate("general_transfer_balance_cap", &transfer),
                rate(
                    "carry_forward_balance_limit",
                    meta::CARRY_FORWARD_BALANCE_LIMIT,
                ),
                rate("non_concessional_cap_multiple", multiple),
                rate(
                    "bring_forward_period_years",
                    if triggered { multiple } else { 0 },
                ),
            ],
        ))
    })?;
    result("contribution_caps", &year, amounts, rates)
}

fn is_leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub fn pension_minimum(
    account_balance: &Arg,
    age: &Arg,
    days: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "pension_minimum")?;
    let balance = money(account_balance)?;
    let Some(Some(age)) = age
        .int()
        .filter(|n| n.is_some_and(|n| (0..=150).contains(&n)))
    else {
        return refuse("age must be an integer number of years from 0 to 150.");
    };
    let start: i64 = year[..4].parse().expect("supported period");
    let days_in_year: i64 = if is_leap(start + 1) { 366 } else { 365 };
    let Some(Some(days)) = days
        .int()
        .filter(|n| n.is_some_and(|n| (1..=days_in_year).contains(&n)))
    else {
        return refuse(format!(
            "days must be an integer from 1 to {days_in_year} for {year}."
        ));
    };
    let factor = d(meta::PENSION_MINIMUM_FACTORS
        .iter()
        .find(|(lowest, _)| age >= *lowest)
        .expect("age 0 band")
        .1);
    let amounts = precise(|| {
        // cl 4: nothing is required for a pension commencing on or after 1 June,
        // which is within the last 30 days of the income year.
        let exact = if days <= 30 {
            int(0)
        } else {
            balance
                .mul(&factor)?
                .mul(&int(days))?
                .div(&int(days_in_year))?
        };
        let minimum = exact
            .div(&int(10))?
            .quantize(0, Rounding::HalfUp)?
            .mul(&int(10))?;
        Ok(vec![
            ("account_balance_used", balance.clone()),
            ("minimum_before_rounding", exact),
            ("minimum_payment", minimum),
        ])
    })?;
    result(
        "pension_minimum",
        &year,
        amounts,
        vec![
            rate("percentage_factor", &factor),
            rate("days", days),
            rate("days_in_year", days_in_year),
        ],
    )
}

const KINDS: [&str; 16] = [
    "gst",
    "resident_tax",
    "capital_gains",
    "fbt",
    "fbt_car_statutory",
    "fbt_car_operating_cost",
    "depreciation",
    "quarterly_sg",
    "payg_withholding",
    "contribution_caps",
    "pension_minimum",
    "study_loan_repayment",
    "study_loan_withholding",
    "medicare_levy",
    "low_income_tax_offset",
    "division_293",
];

/// `periods(kind)`: accepted period arguments and inclusive calendar boundaries.
pub fn periods(kind: &str) -> Value {
    let mut out = Vec::new();
    for year in meta::supported_periods(kind) {
        let mut period = Map::new();
        if kind.starts_with("fbt") {
            let ended: i64 = year.parse().expect("FBT year");
            period.insert("id".into(), json!(year));
            period.insert("basis".into(), json!("FBT year"));
            period.insert("start".into(), json!(format!("{}-04-01", ended - 1)));
            period.insert("end".into(), json!(format!("{year}-03-31")));
            period.insert("arguments".into(), json!({ "year_ended": ended }));
        } else {
            let start: i64 = year[..4].parse().expect("income year");
            period.insert("id".into(), json!(year));
            period.insert("basis".into(), json!("income year"));
            period.insert("start".into(), json!(format!("{start}-07-01")));
            period.insert("end".into(), json!(format!("{}-06-30", start + 1)));
            period.insert("arguments".into(), json!({ "year": year }));
            if kind == "quarterly_sg" {
                period.insert("basis".into(), json!("complete quarter within income year"));
                let next = start + 1;
                period.insert(
                    "quarters".into(),
                    json!([
                        {"quarter": 1, "start": format!("{start}-07-01"), "end": format!("{start}-09-30")},
                        {"quarter": 2, "start": format!("{start}-10-01"), "end": format!("{start}-12-31")},
                        {"quarter": 3, "start": format!("{next}-01-01"), "end": format!("{next}-03-31")},
                        {"quarter": 4, "start": format!("{next}-04-01"), "end": format!("{next}-06-30")},
                    ]),
                );
            }
        }
        out.push(Value::Object(period));
    }
    Value::Array(out)
}

fn example_value(value: Example) -> Value {
    match value {
        Example::Text(s) => json!(s),
        Example::Flag(b) => json!(b),
        Example::Whole(n) => json!(n),
    }
}

/// `worksheet_catalogue()`: JSON-ready discovery data for every worksheet.
pub fn worksheet_catalogue() -> Value {
    let mut out = Map::new();
    for kind in KINDS {
        let supported = periods(kind);
        let fbt = kind.starts_with("fbt");
        let (checked, passage) = meta::source_review(kind);
        let mut evidence: Map<String, Value> = meta::library_evidence(kind)
            .iter()
            .map(|(k, v)| (k.to_string(), json!(v)))
            .collect();
        evidence.insert(
            "note".into(),
            json!(format!(
                "docs/calculation-evidence.md#{}",
                kind.replace('_', "-")
            )),
        );
        let mut inputs = Map::new();
        inputs.insert("kind".into(), json!(kind));
        inputs.insert(
            "scope_confirmed".into(),
            json!("boolean true; establish every scope condition"),
        );
        inputs.insert(
            if fbt { "year_ended" } else { "year" }.into(),
            json!(if fbt { "integer" } else { "income-year string" }),
        );
        for (k, v) in meta::input_units(kind) {
            inputs.insert(k.into(), json!(v));
        }
        let mut facts = Map::new();
        facts.insert("kind".into(), json!(kind));
        facts.insert("scope_confirmed".into(), json!(true));
        if let Some(Value::Object(arguments)) = supported[0].get("arguments") {
            for (k, v) in arguments {
                facts.insert(k.clone(), v.clone());
            }
        }
        for (k, v) in meta::examples(kind) {
            facts.insert(k.into(), example_value(v));
        }
        let mut entry = Map::new();
        entry.insert("scope".into(), json!(lookup(texts::SCOPES, kind)));
        entry.insert("source".into(), json!(lookup(texts::SOURCES, kind)));
        entry.insert("sources".into(), json!(sources(kind)));
        entry.insert("source_checked".into(), json!(checked));
        entry.insert("source_passage".into(), json!(passage));
        entry.insert("example_evidence".into(), Value::Object(evidence));
        entry.insert("supported_periods".into(), supported.clone());
        entry.insert("required_inputs".into(), Value::Object(inputs));
        entry.insert(
            "money_format".into(),
            json!(
                "Non-negative finite AUD decimal strings, at most 2 decimal places and 1000000000000.00. Resident taxable income must be whole dollars."
            ),
        );
        entry.insert(
            "methods".into(),
            if kind == "depreciation" {
                json!(["prime_cost", "diminishing_value"])
            } else {
                json!([])
            },
        );
        entry.insert(
            "example".into(),
            json!({ "synthetic": true, "facts": Value::Object(facts) }),
        );
        out.insert(kind.into(), Value::Object(entry));
    }
    Value::Object(out)
}
