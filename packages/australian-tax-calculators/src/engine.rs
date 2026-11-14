// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Explicitly bounded worksheets using Decimal and published official methods,
//! ported from `austaxcalc.calculations` without changing a rate, rounding
//! step, scope statement or message. Every worksheet computes in a 40-digit
//! context, as the original's `localcontext` did.

use serde_json::{Map, Value, json};

use crate::dates;
use crate::decimal::{Calc, Decimal, Rounding, Signal, with_precision};
use crate::metadata::{self as meta, CoefficientRow, Example};
use crate::pytext;
use crate::texts;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const WORKING_PRECISION: i128 = 40;

/// A refusal: Python's ValueError with its message, or a trapped decimal signal.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Value {
        /// The refused rule's stable code, from `texts::REFUSAL_CODES`.
        code: &'static str,
        message: String,
    },
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
            Error::Value { message, .. } => f.write_str(message),
            Error::Decimal(s) => write!(f, "decimal.{}", s.name()),
        }
    }
}

impl Error {
    /// The refusal's stable code; the message beside it may be reworded.
    pub fn code(&self) -> &'static str {
        match self {
            Error::Value { code, .. } => code,
            Error::Decimal(signal) => signal.code(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn refuse<T>(code: &'static str, message: impl Into<String>) -> Result<T> {
    Err(Error::Value {
        code,
        message: message.into(),
    })
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
    /// A JSON value for a parameter the original never had (main residence periods).
    Json(Value),
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
            Arg::Json(v) => v.to_string(),
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
    let bad = || {
        refuse(
            "input.amount_invalid",
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        )
    };
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
        return refuse(
            "scope.not_confirmed",
            "Establish all stated scope conditions before calculating.",
        );
    }
    let supported = meta::supported_periods(kind);
    match year.text() {
        Some(y) if supported.contains(&y) => Ok(y.to_string()),
        _ => refuse(
            "period.unsupported",
            format!(
                "Unsupported period {}; supported: {}.",
                year.repr(),
                supported.join(", ")
            ),
        ),
    }
}

fn whole_dollars(income: &Decimal, what: &str) -> Result<()> {
    if !income.equals(&income.to_integral_value()?)? {
        return refuse(
            "input.amount_not_whole_dollars",
            format!("Supply established whole-dollar {what}."),
        );
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
    let mut warnings = vec![
        "Operator-confirmed facts and scope; eligibility is not independently verified.",
        "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
    ];
    if let Some((_, extra)) = texts::EXTRA_WARNINGS.iter().find(|(k, _)| *k == kind) {
        warnings.extend(extra.iter().copied());
    }
    out.insert("warnings".into(), json!(warnings));
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
        return refuse("input.flag_invalid", "gst_inclusive must be a boolean.");
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
    // `_scope` reports the period as the original's `str(year_ended)`.
    let year = scope(scope_confirmed, &Arg::Text(year_ended.py_str()), "fbt")?;
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
        &format!("year ended 31 March {year}"),
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
        _ => refuse(
            "input.day_count_invalid",
            format!("{name} must be an integer from 0 to {days_in_year}."),
        ),
    }
}

fn flag(value: &Arg, name: &str) -> Result<bool> {
    match value {
        Arg::Bool(b) => Ok(*b),
        _ => refuse("input.flag_invalid", format!("{name} must be a boolean.")),
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
/// value its reportable fringe benefits amount is worked out from, so it is
/// also the car's reportable taxable value, before the employee's $2,000
/// threshold and the type 2 gross-up, which apply to all their benefits.
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
    out.push(("taxable_value", if exempt { int(0) } else { value.clone() }));
    out.push(("reportable_taxable_value", value));
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
/// the holding period it was owned, and the s 10(5) statutory comparison. The
/// deemed depreciation rate is 200% over the car's effective life (s 11(1AA)).
#[allow(clippy::too_many_arguments)]
pub fn fbt_car_operating_cost(
    running_costs: &Arg,
    lease_charges: &Arg,
    depreciated_value: &Arg,
    effective_life_years: &Arg,
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
            "fbt_car.cost_basis_conflict",
            "A car is either owned, with a depreciated_value, or leased, with lease_charges; set the other to 0.",
        );
    }
    // An owned car can reach a nil depreciated value, so a nil value does not
    // mean a leased car; either way it deems no depreciation.
    let depreciable = value.gt(&int(0))?;
    // s 11 has no cost cap, so a life under 2 would deem more than the car's value.
    let usable = finite_between(effective_life_years, "2", "1000")?;
    let nil = finite_between(effective_life_years, "0", "0")?;
    let Some(life) = usable.or(if depreciable { None } else { nil }) else {
        return refuse(
            "input.effective_life_invalid",
            "effective_life_years must be a finite Decimal from 2 to 1000 years, or 0 for a leased car or an owned car with no depreciated value left.",
        );
    };
    let days = day_count(days_held, days_in_year, "days_held")?;
    let Some(business) = finite_between(business_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "business_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    let contribution = money(employee_contribution)?;
    let statutory = money(statutory_formula_taxable_value)?;
    let exempt = flag(electric_car_exempt, "electric_car_exempt")?;
    let interest_rate = d(lookup(meta::FBT_BENCHMARK_INTEREST_RATES, &year));
    let private = precise(|| Ok(int(1).sub(&business)?))?;
    let depreciation_rate = precise(|| {
        Ok(if depreciable {
            int(2).div(&life)?
        } else {
            int(0)
        })
    })?;
    let amounts = precise(|| {
        let deemed = |rate: &Decimal| value.mul(rate)?.mul(&int(days))?.div(&int(days_in_year));
        let depreciation = deemed(&depreciation_rate)?;
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
            rate("depreciation_rate", &depreciation_rate),
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
    opening_adjustable_value: Option<&Arg>,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "depreciation")?;
    let cost = money(cost)?;
    let Some(life) = finite_between(effective_life, "0.01", "1000")? else {
        return refuse(
            "input.effective_life_invalid",
            "effective_life must be a finite Decimal between 0.01 and 1000 years.",
        );
    };
    let Some(use_) = finite_between(taxable_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "taxable_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    // Exact to the cent only within this bound. For a life of l significant
    // digits and a use of u decimal places, a deduction that is not exactly a
    // half cent is at least 1 / (730 * l-digit integer * 10^u) cents from one;
    // with l + u <= 22 the 40-digit products are exact and the one division errs
    // by under 1e-25 cents on a cost of at most 1e12, which is closer. (A life
    // written with a positive exponent widens that denominator by its trailing
    // zeros, and shrinks the deduction, and so the error, by the same factor.)
    let life_digits = life.adjusted() - life.exponent() + 1;
    if life_digits + (-use_.exponent()).max(0) > 22 {
        return refuse(
            "depreciation.precision_exceeded",
            "effective_life's significant digits and taxable_use's decimal places, as written, must total 22 or fewer: beyond that the 40-digit working precision can miss the exact cent.",
        );
    }
    let Some(Some(days)) = days
        .int()
        .filter(|n| n.is_some_and(|n| (0..=365).contains(&n)))
    else {
        return refuse(
            "input.day_count_invalid",
            format!("days must be an integer from 0 to 365 for {year}."),
        );
    };
    let factor = match method.text() {
        Some("prime_cost") => int(1),
        Some("diminishing_value") => int(2),
        _ => {
            return refuse(
                "input.depreciation_method_invalid",
                "method must be prime_cost or diminishing_value.",
            );
        }
    };
    // A later year starts from its opening adjustable value: the diminishing
    // value base and the limit on either method's decline (ss 40-70(1), 40-72(3)
    // and 40-75(7)); prime cost still applies its rate to cost (s 40-75(1)).
    let opening = match opening_adjustable_value {
        None => cost.clone(),
        Some(value) => {
            let opening = money(value)?;
            if opening.gt(&cost)? {
                return refuse(
                    "input.opening_value_above_cost",
                    "opening_adjustable_value cannot exceed cost.",
                );
            }
            opening
        }
    };
    let base = if method.text() == Some("prime_cost") {
        &cost
    } else {
        &opening
    };
    let amounts = precise(|| {
        // Divide once, last: a quotient taken first is rounded to the working
        // precision, which can move an exact half cent below it.
        let numerator = base.mul(&int(days))?.mul(&factor)?;
        let denominator = int(365).mul(&life)?;
        let raw = numerator.div(&denominator)?;
        // py_min(opening, raw): the opening value unless raw is lower.
        let (decline, deduction) = if raw.lt(&opening)? {
            (raw, numerator.mul(&use_)?.div(&denominator)?)
        } else {
            (opening.clone(), opening.mul(&use_)?)
        };
        // Less the decline as shown, so a closing value carried forward as the
        // next opening value never writes off more than cost.
        let closing = opening.sub(&cents(decline.clone())?)?;
        Ok(vec![
            ("decline_in_value", decline),
            ("deduction", deduction),
            ("closing_adjustable_value", closing),
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
        return refuse(
            "input.quarter_invalid",
            "quarter must be 1 (Jul-Sep), 2, 3 or 4 (Apr-Jun).",
        );
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
        _ => {
            return refuse(
                "input.pay_period_invalid",
                "pay_period must be weekly, fortnightly or monthly.",
            );
        }
    };
    let Some(Some(scale)) = scale.int() else {
        return refuse(
            "input.tax_scale_unsupported",
            "scale must be 1, 2, 3, 5 or 6; scale 4 is not supported.",
        );
    };
    if payg_rows(&year, scale).is_none() {
        return refuse(
            "input.tax_scale_unsupported",
            "scale must be 1, 2, 3, 5 or 6; scale 4 is not supported.",
        );
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
        return refuse(
            "input.flag_invalid",
            "section_9a_adjustment_applies must be a boolean.",
        );
    };
    if *adjusted {
        let extra = texts::EXTRA_SOURCES
            .iter()
            .find(|(k, _)| *k == "medicare_levy")
            .expect("medicare sources")
            .1[2];
        return refuse(
            "medicare.s9a_adjustment_out_of_scope",
            format!(
                "Medicare Levy Act 1986 s 9A (lump sum payments in arrears) applies, and this \
             worksheet does not make that adjustment. See {extra}"
            ),
        );
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
    let mut out = result(
        "medicare_levy",
        &year,
        amounts,
        vec![
            rate("levy_rate", &levy_rate),
            rate("shade_in_rate", &shade_in),
            rate("threshold_amount", &threshold),
            rate("phase_in_limit", &limit),
        ],
    )?;
    if let Some((_, note)) = texts::MEDICARE_YEAR_WARNINGS
        .iter()
        .find(|(y, _)| *y == year)
    {
        out["warnings"]
            .as_array_mut()
            .expect("warnings")
            .push(json!(note));
    }
    Ok(out)
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

/// A finite Decimal of at most 2dp and magnitude 1e12 that may be negative:
/// total superannuation earnings can be (Division 296 regulations, reg 296-65.02 note).
fn signed_money(value: &Arg) -> Result<Decimal> {
    let bad = || {
        refuse(
            "input.signed_amount_invalid",
            "Amounts that may be negative must be finite Decimals, at most 2dp and 1e12 either side of nil.",
        )
    };
    let Arg::Decimal(v) = value else {
        return bad();
    };
    if !v.is_finite() || v.lt(&d("-1e12"))? || v.gt(&d("1e12"))? || v.exponent() < -2 {
        return bad();
    }
    Ok(v.clone())
}

/// The share of `balance` above `threshold` as a percentage rounded to 2 decimal
/// places, half up (ITAA 1997 ss 296-40(2) and (3), 296-45(2) and (3)); nil when
/// the balance is not greater than the threshold.
fn percentage_above(balance: &Decimal, threshold: &Decimal) -> Result<Decimal> {
    if !balance.gt(threshold)? {
        return Ok(d("0.00"));
    }
    precise(|| {
        let share = balance.sub(threshold)?.div(balance)?.mul(&int(100))?;
        Ok(share.quantize(-2, Rounding::HalfUp)?)
    })
}

/// Exceptions in order, each with the reason given when it applies.
const DIVISION_296_EXCEPTIONS: [(&str, &str); 3] = [
    (
        "child_recipient",
        "not liable: a child recipient of a superannuation income stream in the year (s 296-20)",
    ),
    (
        "structured_settlement_contribution",
        "not liable: a structured settlement contribution was made in respect of the individual in or before the year (s 296-25)",
    ),
    (
        "died_in_year",
        "not liable: died on or before the last day of the 2026-27 income year (Income Tax (Transitional Provisions) Act 1997 s 296-1(3))",
    ),
];

/// The reason of the first exception that applies, if any, and whether the
/// individual died by the end of the year (s 296-50), once every flag is valid.
fn division_296_exceptions(exceptions: [&Arg; 3]) -> Result<(Option<&'static str>, bool)> {
    let mut excepted = None;
    let mut died = false;
    for (value, (name, reason)) in exceptions.into_iter().zip(DIVISION_296_EXCEPTIONS) {
        let set = flag(value, name)?;
        if set && excepted.is_none() {
            excepted = Some(reason);
        }
        if name == "died_in_year" {
            died = set;
        }
    }
    Ok((excepted, died))
}

/// Whether s 296-15 makes the individual liable, with the reason given.
fn division_296_liability(
    excepted: Option<&'static str>,
    reference: &Decimal,
    large: &Decimal,
    earnings: &Decimal,
    percentage: &Decimal,
) -> Result<(bool, &'static str)> {
    Ok(if let Some(reason) = excepted {
        (false, reason)
    } else if !reference.gt(large)? {
        (
            false,
            "not liable: the total superannuation balance at the end of the year is not greater than the large superannuation balance threshold (s 296-40(1)(a); Income Tax (Transitional Provisions) Act 1997 s 296-1(2))",
        )
    } else if !earnings.gt(&int(0))? {
        (
            false,
            "not liable: total superannuation earnings for the year are not greater than nil (s 296-40(1)(b))",
        )
    } else if percentage.is_zero() {
        (
            true,
            "liable (s 296-15), but the s 296-40(3) percentage rounds to 0.00, so taxable superannuation earnings are nil",
        )
    } else {
        (true, "liable (s 296-15)")
    })
}

/// The earnings amounts ss 296-40 and 296-45 define, whether or not an exception
/// removes the liability, and the 2 taxes and their total, which only a liability
/// gives; also whether the very large component is negative. A share is nil when
/// the balance is not above its threshold, so taxable superannuation earnings
/// arise only from a nonzero first share and earnings greater than nil
/// (s 296-40(1)), and s 296-45 has no 'greater than nil' condition.
fn division_296_amounts(
    liable: bool,
    earnings: &Decimal,
    [percentage, very_large_percentage]: [&Decimal; 2],
    [rate_, very_large_rate]: [&Decimal; 2],
) -> Result<(Vec<(&'static str, Decimal)>, bool)> {
    precise(|| {
        let hundred = int(100);
        let taxable = if earnings.gt(&int(0))? && !percentage.is_zero() {
            earnings.mul(percentage)?.div(&hundred)?
        } else {
            int(0)
        };
        let component = if !very_large_percentage.is_zero() {
            earnings.mul(very_large_percentage)?.div(&hundred)?
        } else {
            int(0)
        };
        let (tax, further) = if liable {
            (taxable.mul(rate_)?, component.mul(very_large_rate)?)
        } else {
            (int(0), int(0))
        };
        let total = tax.add(&further)?;
        // Results carry no negative amounts, so a negative component is reported as null,
        // and a zero one, which earnings of -0.00 would sign, as 0.00.
        let negative = component.lt(&int(0))?;
        let shown = if negative || component.is_zero() {
            int(0)
        } else {
            component
        };
        Ok((
            vec![
                ("taxable_superannuation_earnings", taxable),
                ("very_large_balance_earnings_component", shown),
                ("tax_on_taxable_superannuation_earnings", tax),
                ("tax_on_very_large_balance_earnings_component", further),
                ("division_296_tax", total),
            ],
            negative,
        ))
    })
}

pub fn division_296(
    total_super_balance_end_of_year: &Arg,
    total_super_earnings: &Arg,
    exceptions: [&Arg; 3],
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "division_296")?;
    let balance = money(total_super_balance_end_of_year)?;
    let earnings = signed_money(total_super_earnings)?;
    let (excepted, died) = division_296_exceptions(exceptions)?;
    let [large, very_large, rate_, very_large_rate] = [
        "large_threshold",
        "very_large_threshold",
        "rate",
        "very_large_rate",
    ]
    .map(|key| d(lookup(&meta::DIVISION_296, key)));
    // ITTPA s 296-1(2): for 2026-27 the reference amount is the balance at the end of the
    // year, which s 296-50 takes to be nil for someone who has died by then.
    let reference = if died { int(0) } else { balance };
    let percentage = percentage_above(&reference, &large)?;
    let very_large_percentage = percentage_above(&reference, &very_large)?;
    let (liable, liability) =
        division_296_liability(excepted, &reference, &large, &earnings, &percentage)?;
    let (amounts, negative_component) = division_296_amounts(
        liable,
        &earnings,
        [&percentage, &very_large_percentage],
        [&rate_, &very_large_rate],
    )?;
    let mut out = result(
        "division_296",
        &year,
        amounts,
        vec![
            rate("large_superannuation_balance_threshold", &large),
            rate("very_large_superannuation_balance_threshold", &very_large),
            rate("rate", &rate_),
            rate("very_large_rate", &very_large_rate),
            rate("taxable_percentage", &percentage),
            rate("very_large_percentage", &very_large_percentage),
        ],
    )?;
    if negative_component {
        out["amounts"]["very_large_balance_earnings_component"] = Value::Null;
    }
    out["liability"] = json!(liability);
    Ok(out)
}

/// SISR reg 6.01(2): 55 for a person born before 1 July 1960, one more for
/// each later year of birth from 1 July, and 60 for one born after 30 June 1964.
fn preservation_age(born: (i64, i64, i64)) -> i64 {
    let (year, month, _) = born;
    let birth_year_from_july = if month >= 7 { year } else { year - 1 };
    (birth_year_from_july - 1960 + 56).clamp(55, 60)
}

/// Schedule 11 for one ETP part: the part within its cap, the part above it,
/// and the amount withheld from both, rounded to the dollar with 50 cents up.
fn etp_part(
    component: &Decimal,
    cap: &Decimal,
    concessional: &Decimal,
    top: &Decimal,
) -> Calc<[Decimal; 3]> {
    let within = py_min(component.clone(), cap.clone())?;
    let above = component.sub(&within)?;
    let withheld = within
        .mul(concessional)?
        .add(&above.mul(top)?)?
        .quantize(0, Rounding::HalfUp)?;
    Ok([within, above, withheld])
}

/// Schedule 11 for one ETP: the tax-free part of the redundancy payment, then
/// the excluded part (code R) first, under the ETP cap alone, and the
/// non-excluded part (code O), under the smaller of the ETP cap left and the
/// whole-of-income cap (s 82-10(4), (6) and (7); Schedule 11).
fn etp_amounts(
    redundancy: &Decimal,
    years: i64,
    excluded: &Decimal,
    non_excluded: &Decimal,
    other: &Decimal,
    concessional: &Decimal,
) -> Result<Vec<(&'static str, Decimal)>> {
    let [etp_cap, whole_of_income, base, service, top] = [
        "etp_cap",
        "whole_of_income_cap",
        "base_amount",
        "service_amount",
        "above_cap_rate",
    ]
    .map(|key| d(lookup(&meta::TERMINATION_PAYMENT, key)));
    precise(|| {
        // s 83-170(3): base amount + service amount x years of service.
        let limit = base.add(&service.mul(&int(years))?)?;
        let tax_free = py_min(redundancy.clone(), limit.clone())?;
        let excluded_part = redundancy.sub(&tax_free)?.add(excluded)?;
        let [r_within, r_above, r_withheld] =
            etp_part(&excluded_part, &etp_cap, concessional, &top)?;
        let remaining = etp_cap.sub(&r_within)?;
        let calculated = py_max(int(0), whole_of_income.sub(other)?)?;
        let non_excluded_cap = py_min(remaining.clone(), calculated.clone())?;
        let [o_within, o_above, o_withheld] =
            etp_part(non_excluded, &non_excluded_cap, concessional, &top)?;
        let total = r_withheld.add(&o_withheld)?;
        Ok(vec![
            ("tax_free_limit", limit),
            ("tax_free_redundancy_amount", tax_free),
            ("excluded_taxable_component", excluded_part),
            ("excluded_within_cap", r_within),
            ("excluded_above_cap", r_above),
            ("excluded_withholding", r_withheld),
            ("remaining_etp_cap", remaining),
            ("whole_of_income_cap", calculated),
            ("non_excluded_cap", non_excluded_cap),
            ("non_excluded_taxable_component", non_excluded.clone()),
            ("non_excluded_within_cap", o_within),
            ("non_excluded_above_cap", o_above),
            ("non_excluded_withholding", o_withheld),
            ("total_withholding", total),
        ])
    })
}

/// The whole years of service, from 0 to 150, that a redundancy payment relates
/// to (s 83-170(3)).
fn years_of_service(value: &Arg) -> Result<i64> {
    let Some(Some(years)) = value
        .int()
        .filter(|n| n.is_some_and(|n| (0..=150).contains(&n)))
    else {
        return refuse(
            "input.years_of_service_invalid",
            "completed_years_of_service must be a whole number from 0 to 150.",
        );
    };
    Ok(years)
}

#[allow(clippy::too_many_arguments)]
pub fn termination_payment(
    redundancy_payment: &Arg,
    completed_years_of_service: &Arg,
    excluded_payment: &Arg,
    non_excluded_payment: &Arg,
    other_taxable_payments: &Arg,
    date_of_birth: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "termination_payment")?;
    let redundancy = money(redundancy_payment)?;
    let years = years_of_service(completed_years_of_service)?;
    let excluded = money(excluded_payment)?;
    let non_excluded = money(non_excluded_payment)?;
    let other = money(other_taxable_payments)?;
    let born = dates::to_civil(date_arg(date_of_birth, "date_of_birth")?);
    let figure = |key| lookup(&meta::TERMINATION_PAYMENT, key);
    // s 82-10(3)(a): preservation age or older on the last day of the income year.
    let age = preservation_age(born);
    let year_end: i64 = year[..4].parse::<i64>().expect("income year") + 1;
    let reached = (born.0 + age, born.1, born.2) <= (year_end, 6, 30);
    let within_rate = if reached {
        "preservation_age_rate"
    } else {
        "under_preservation_age_rate"
    };
    let concessional = d(figure(within_rate));
    let amounts = etp_amounts(
        &redundancy,
        years,
        &excluded,
        &non_excluded,
        &other,
        &concessional,
    )?;
    let rates = vec![
        rate("etp_cap", figure("etp_cap")),
        rate("whole_of_income_cap_amount", figure("whole_of_income_cap")),
        rate("redundancy_base_amount", figure("base_amount")),
        rate("redundancy_service_amount", figure("service_amount")),
        rate("preservation_age", age),
        rate("preservation_age_reached", reached),
        rate("withholding_rate_within_cap", &concessional),
        rate("withholding_rate_above_cap", figure("above_cap_rate")),
    ];
    result("termination_payment", &year, amounts, rates)
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
        return refuse("input.flag_invalid", "under_75_in_year must be a boolean.");
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
        // s 291-20(3)-(4): the unused cap raises this year's cap only by the
        // excess it covers; unapplied amounts remain subject to 5-year expiry.
        let applied = py_min(carried.clone(), py_max(int(0), concessional.sub(&cap)?)?)?;
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
                ("carry_forward_applied", applied),
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
        return refuse(
            "input.age_invalid",
            "age must be an integer number of years from 0 to 150.",
        );
    };
    let start: i64 = year[..4].parse().expect("supported period");
    let days_in_year: i64 = if is_leap(start + 1) { 366 } else { 365 };
    let Some(Some(days)) = days
        .int()
        .filter(|n| n.is_some_and(|n| (1..=days_in_year).contains(&n)))
    else {
        return refuse(
            "input.day_count_invalid",
            format!("days must be an integer from 1 to {days_in_year} for {year}."),
        );
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

/// A period of days, both ends included, and for income use the part of the
/// dwelling's floor area used.
struct Span {
    start: i64,
    end: i64,
    part: Option<Decimal>,
}

fn date_arg(value: &Arg, name: &str) -> Result<i64> {
    match value.text().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => refuse(
            "input.date_invalid",
            format!("{name} must be a YYYY-MM-DD date from 1900 to 2100."),
        ),
    }
}

/// A list of periods inside `first..=last`, sorted, refused if two overlap.
fn spans(value: &Arg, name: &str, with_part: bool, first: i64, last: i64) -> Result<Vec<Span>> {
    let bad = |code, why: &str| refuse(code, format!("{name}: {why}"));
    let Arg::Json(Value::Array(items)) = value else {
        return bad(
            "main_residence.periods_not_a_list",
            "must be a list of periods.",
        );
    };
    if items.len() > 1000 {
        return bad("main_residence.periods_too_many", "at most 1000 periods.");
    }
    let keys: &[&str] = if with_part {
        &["start", "end", "floor_area"]
    } else {
        &["start", "end"]
    };
    let mut out = Vec::new();
    for item in items {
        let fields = match item {
            Value::Object(f)
                if f.len() == keys.len() && keys.iter().all(|k| f.contains_key(*k)) =>
            {
                f
            }
            _ => {
                return bad(
                    "main_residence.period_shape_invalid",
                    &format!(
                        "each period must be an object with exactly {}.",
                        keys.join(", ")
                    ),
                );
            }
        };
        let date = |key: &str| fields[key].as_str().and_then(dates::parse);
        let (Some(start), Some(end)) = (date("start"), date("end")) else {
            return bad(
                "main_residence.period_date_invalid",
                "start and end must be YYYY-MM-DD dates from 1900 to 2100.",
            );
        };
        if start > end || start < first || end > last {
            return bad(
                "main_residence.period_outside_ownership",
                "each period must run forwards inside the ownership period.",
            );
        }
        let part = if with_part {
            let fraction = fields["floor_area"]
                .as_str()
                .and_then(|t| Decimal::parse(t).ok());
            match fraction {
                Some(f)
                    if f.is_finite()
                        && f.exponent() >= -4
                        && f.gt(&int(0))?
                        && f.le(&int(1))? =>
                {
                    Some(f)
                }
                _ => {
                    return bad(
                        "main_residence.floor_area_invalid",
                        "floor_area must be a decimal string above 0 and at most 1, to 4 places.",
                    );
                }
            }
        } else {
            None
        };
        out.push(Span { start, end, part });
    }
    out.sort_by_key(|s| s.start);
    if out.windows(2).any(|w| w[1].start <= w[0].end) {
        return bad(
            "main_residence.periods_overlap",
            "periods must not overlap.",
        );
    }
    Ok(out)
}

/// ITAA 1997 Subdiv 118-B: the part of a dwelling's capital gain or loss that
/// is not exempt, from a taxable weight for each day of the ownership period.
#[allow(clippy::too_many_arguments)]
pub fn main_residence(
    capital_gain: &Arg,
    capital_loss: &Arg,
    market_value_basis: &Arg,
    ownership_start: &Arg,
    ownership_end: &Arg,
    lived_periods: &Arg,
    income_use_periods: &Arg,
    absence_choice_periods: &Arg,
    new_home_acquired: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "main_residence";
    let year = scope(scope_confirmed, year, kind)?;
    let gain = money(capital_gain)?;
    let loss = money(capital_loss)?;
    if !gain.is_zero() && !loss.is_zero() {
        return refuse(
            "main_residence.gain_and_loss_both_supplied",
            "Supply a capital gain or a capital loss, not both.",
        );
    }
    let market_value = flag(market_value_basis, "market_value_basis")?;
    let first = date_arg(ownership_start, "ownership_start")?;
    let last = date_arg(ownership_end, "ownership_end")?;
    if last < first {
        return refuse(
            "main_residence.ownership_dates_reversed",
            "ownership_end must not be before ownership_start.",
        );
    }
    let year_start = dates::parse(&format!("{}-07-01", &year[..4])).expect("income year");
    if last < year_start {
        return refuse(
            "main_residence.ownership_ends_before_year",
            format!(
                "ownership_end must not be before the {year} income year began: the ownership period ends on or after the CGT event."
            ),
        );
    }
    let year_end = dates::add_months(year_start, 12) - 1;
    if first > year_end {
        return refuse(
            "main_residence.ownership_starts_after_year",
            format!(
                "ownership_start must not be after the {year} income year ended: the dwelling is acquired before the CGT event."
            ),
        );
    }
    if first < dates::parse("1985-09-20").expect("date") {
        return refuse(
            "main_residence.pre_cgt_dwelling",
            "A dwelling acquired before 20 September 1985 is out of scope.",
        );
    }
    let lived = spans(lived_periods, "lived_periods", false, first, last)?;
    let income = spans(income_use_periods, "income_use_periods", true, first, last)?;
    let chosen = spans(
        absence_choice_periods,
        "absence_choice_periods",
        false,
        first,
        last,
    )?;
    let new_home = match new_home_acquired {
        Arg::Json(Value::Null) => None,
        Arg::Json(Value::String(t)) => match dates::parse(t) {
            Some(day) if day <= last => Some(day),
            _ => {
                return refuse(
                    "main_residence.new_home_date_invalid",
                    "new_home_acquired must be null or a YYYY-MM-DD date no later than ownership_end.",
                );
            }
        },
        _ => {
            return refuse(
                "main_residence.new_home_date_invalid",
                "new_home_acquired must be null or a YYYY-MM-DD date no later than ownership_end.",
            );
        }
    };

    let days = (last - first + 1) as usize;
    let at = |day: i64| (day - first) as usize;
    let mut is_lived = vec![false; days];
    let mut part: Vec<Option<Decimal>> = vec![None; days];
    let mut is_chosen = vec![false; days];
    for s in &lived {
        is_lived[at(s.start)..=at(s.end)].fill(true);
    }
    for s in &income {
        part[at(s.start)..=at(s.end)].fill(s.part.clone());
    }
    for s in &chosen {
        if s.start == first
            || !is_lived[at(s.start - 1)]
            || is_lived[at(s.start)..=at(s.end)].contains(&true)
        {
            return refuse(
                "main_residence.absence_choice_period_invalid",
                "absence_choice_periods: each period must start the day after a lived period ends and contain no lived day.",
            );
        }
        is_chosen[at(s.start)..=at(s.end)].fill(true);
    }

    // s 118-145: a part used to produce income before the move stays taxable
    // throughout the absence (ATO QC66030, Helen); income use is covered for
    // 6 years per absence, to the sixth anniversary of its first day inclusive
    // (ATO QC66030, Roya), counted across breaks in the same absence. A
    // covered day's weight is that part; a day past the 6 years is taxable.
    let mut chosen_weight: Vec<Option<Decimal>> = vec![None; days];
    let mut past_cap = vec![false; days];
    for s in &chosen {
        let mut run = at(s.start - 1);
        while run > 0 && is_lived[run - 1] {
            run -= 1;
        }
        let before = &part[run..at(s.start)];
        let kept = before.last().cloned().flatten();
        // Once part use begins it must continue, at one floor area, to the move.
        let mut consistent = true;
        let mut began = false;
        for p in before {
            match (p, &kept) {
                (Some(p), Some(k)) => {
                    began = true;
                    consistent &= k.equals(p)?;
                }
                (Some(_), None) => consistent = false,
                (None, _) => consistent &= !began,
            }
        }
        if !consistent {
            return refuse(
                "main_residence.income_use_changed_before_absence",
                "Income use of part of the dwelling changed or stopped before the move that started an absence choice period: out of scope.",
            );
        }
        let kept = kept.unwrap_or_else(|| int(0));
        let mut cap: Option<i64> = None;
        let mut used = 0;
        for day in s.start..=s.end {
            let i = at(day);
            chosen_weight[i] = Some(kept.clone());
            if part[i].is_some() {
                let limit = *cap.get_or_insert_with(|| dates::add_months(day, 72) - day + 1);
                used += 1;
                if used > limit {
                    chosen_weight[i] = Some(int(1));
                    past_cap[i] = true;
                }
            }
        }
    }

    // s 118-140: both homes are main residences for up to 6 months ending
    // when ownership ends, from the 6-month date itself (Library ¶2-075).
    let overlap = match new_home {
        None => Err("not claimed"),
        Some(acquired) => {
            // The 12 months ending on the last day start the day after the
            // same date a year earlier.
            let window = (dates::add_months(last, -12) + 1).max(first);
            let mut three_months = false;
            let mut run_start = None;
            for day in window..=last {
                if is_lived[at(day)] {
                    let a = *run_start.get_or_insert(day);
                    three_months |= day >= dates::add_months(a, 3) - 1;
                } else {
                    run_start = None;
                }
            }
            // Not the main residence: neither lived in nor covered by a choice.
            let income_while_not_home = (window..=last).any(|d| {
                let i = at(d);
                part[i].is_some() && !is_lived[i] && (!is_chosen[i] || past_cap[i])
            });
            if !three_months {
                Err(
                    "not applied: no continuous 3 months as main residence in the 12 months before ownership ended",
                )
            } else if income_while_not_home {
                Err(
                    "not applied: used to produce income in those 12 months while not the main residence",
                )
            } else {
                Ok(acquired.max(dates::months_before(last, 6)).max(first))
            }
        }
    };

    // Each day's taxable weight: 0 exempt, 1 taxable, or the part of the
    // floor area used to produce income.
    let mut weight = vec![int(0); days];
    for (i, w) in weight.iter_mut().enumerate() {
        let day = first + i as i64;
        *w = if is_lived[i] {
            part[i].clone().unwrap_or_else(|| int(0))
        } else if let Some(chosen) = &chosen_weight[i] {
            chosen.clone()
        } else if matches!(overlap, Ok(from) if day >= from) {
            int(0)
        } else {
            int(1)
        };
    }

    // s 118-192: first used to produce income after 20 August 1996 when a
    // full exemption would have applied just before, and only partly exempt
    // because of that use: acquired at market value on that day.
    let first_income = income.first().map(|s| s.start);
    let cutoff = dates::parse("1996-08-20").expect("date");
    let mut from = first;
    if let Some(start) = first_income.filter(|d| *d > first) {
        let exempt_before = weight[..at(start)]
            .iter()
            .try_fold(true, |ok, w| Ok::<bool, Signal>(ok && w.is_zero()))?;
        let taxable_after = weight[at(start)..].iter().any(|w| !w.is_zero());
        if exempt_before && taxable_after && start >= cutoff {
            if start == cutoff {
                return refuse(
                    "main_residence.income_use_started_20_august_1996",
                    "Income use first started on 20 August 1996, when the market value rule's 7.30 pm start applies: out of scope.",
                );
            }
            let other_cause = (at(start)..days)
                .any(|i| !weight[i].is_zero() && part[i].is_none() && !is_chosen[i]);
            if other_cause {
                return refuse(
                    "main_residence.partial_exemption_other_cause",
                    "The dwelling is only partly exempt for a reason besides income use as well as that use: out of scope.",
                );
            }
            from = start;
        }
    }
    let applies = from != first;
    if applies && !market_value {
        return refuse(
            "main_residence.first_income_use_rule_applies",
            format!(
                "The home first used to produce income rule applies (s 118-192): work out the gain or loss from the dwelling's market value on {}, then set market_value_basis.",
                dates::format(from)
            ),
        );
    }
    if !applies && market_value {
        return refuse(
            "main_residence.first_income_use_rule_does_not_apply",
            "The home first used to produce income rule does not apply: work out the gain from the actual cost base, or the loss from the actual reduced cost base, and set market_value_basis to false.",
        );
    }

    let period = &weight[at(from)..];
    let counted = period.len() as i64;
    let (taxable_days, amounts) = precise(|| {
        let mut sum = int(0);
        for w in period {
            sum = sum.add(w)?;
        }
        let taxable = |amount: &Decimal| amount.mul(&sum)?.div(&int(counted));
        // Round the taxable share first so the exempt share adds back to it.
        let (g, l) = (cents(taxable(&gain)?)?, cents(taxable(&loss)?)?);
        Ok((
            sum.clone(),
            vec![
                ("assessable_capital_gain", g.clone()),
                ("exempt_capital_gain", gain.sub(&g)?),
                ("assessable_capital_loss", l.clone()),
                ("exempt_capital_loss", loss.sub(&l)?),
            ],
        ))
    })?;
    result(
        kind,
        &year,
        amounts,
        vec![
            rate("ownership_period_from", dates::format(from)),
            rate("ownership_period_to", dates::format(last)),
            rate("ownership_period_days", counted),
            rate("non_main_residence_days", taxable_days),
            rate(
                "home_first_used_to_produce_income",
                if applies { "applied" } else { "not applied" },
            ),
            rate(
                "changing_main_residence_overlap",
                match overlap {
                    Ok(day) => format!("from {}", dates::format(day)),
                    Err(why) => why.to_string(),
                },
            ),
        ],
    )
}

/// The dates that fix the period of a loan under s 25-25(5): the first day
/// it was borrowed, the end of its original contract and the day it was
/// repaid.
pub struct LoanDates<'a> {
    pub start: &'a Arg,
    pub term_end: &'a Arg,
    pub repaid_on: &'a Arg,
}

/// A loan's validated dates as day numbers, read against the income year
/// claimed.
struct Loan {
    start: i64,
    term_end: Option<i64>,
    repaid: Option<i64>,
    /// The income year the loan began in, by the calendar year it starts in.
    first_year: i64,
    /// The income year claimed, by the calendar year it starts in.
    claim_year: i64,
    /// The first day of the income year claimed.
    year_start: i64,
    /// The last day of the income year claimed.
    year_end: i64,
}

impl LoanDates<'_> {
    /// Each date in turn, then `loan_start` against the income year claimed.
    fn parse(&self, year: &str) -> Result<Loan> {
        let start = date_arg(self.start, "loan_start")?;
        let (first_year, month, day) = dates::to_civil(start);
        if (month, day) == (2, 29) {
            return refuse(
                "borrowing.loan_start_29_february",
                "A loan first drawn on 29 February is out of scope: no source settles when its 5 years end.",
            );
        }
        let later = |value: &Arg, name: &str| match value {
            Arg::Json(Value::Null) => Ok(None),
            Arg::Json(Value::String(t)) if dates::parse(t).is_some_and(|day| day >= start) => {
                Ok(dates::parse(t))
            }
            _ => refuse(
                "borrowing.loan_date_invalid",
                format!("{name} must be null or a YYYY-MM-DD date no earlier than loan_start."),
            ),
        };
        let term_end = later(self.term_end, "loan_term_end")?;
        let repaid = later(self.repaid_on, "repaid_on")?;
        let claim_year: i64 = year[..4].parse().expect("income year");
        let year_start = dates::parse(&format!("{claim_year}-07-01")).expect("income year");
        let year_end = dates::add_months(year_start, 12) - 1;
        if start > year_end {
            return refuse(
                "borrowing.loan_starts_after_year",
                format!("loan_start must not be after the {year} income year ended."),
            );
        }
        let first_year = if month >= 7 {
            first_year
        } else {
            first_year - 1
        };
        Ok(Loan {
            start,
            term_end,
            repaid,
            first_year,
            claim_year,
            year_start,
            year_end,
        })
    }
}

impl Loan {
    /// s 25-25(5): the shortest of the contract's period, the period to the
    /// day the loan is repaid and 5 years from the first day it was borrowed,
    /// each day counted (the ATO's 3 July to 2 July 5 years later), as
    /// determined at the end of the income year ending on `last`.
    fn end_at(&self, last: i64) -> i64 {
        let five_years = dates::add_months(self.start, 60) - 1;
        let planned = self.term_end.map_or(five_years, |end| end.min(five_years));
        // Determined at the end of each income year: a repayment counts from then.
        match self.repaid {
            Some(day) if day <= last => planned.min(day),
            _ => planned,
        }
    }

    /// The last day of the period of the loan at the end of the income year
    /// claimed, and the paragraph of s 25-25(5) that fixes it.
    fn period(&self, year: &str) -> Result<(i64, &'static str)> {
        let end = self.end_at(self.year_end);
        if end < self.year_start {
            return refuse(
                "borrowing.loan_period_ended",
                format!(
                    "The period of the loan ended on {}, before the {year} income year began: nothing remains to deduct.",
                    dates::format(end)
                ),
            );
        }
        let basis = if self.term_end == Some(end) {
            "s 25-25(5)(a): the original loan contract"
        } else if self.repaid == Some(end) {
            "s 25-25(5)(b): to the day the loan was repaid"
        } else {
            "s 25-25(5)(c): 5 years from the first borrowing"
        };
        Ok((end, basis))
    }

    /// For the income year claimed: the remaining expenditure, the maximum
    /// deduction, the days left in the period of the loan and its days in
    /// the year.
    fn maximum_deduction(&self, expenditure: &Decimal) -> Result<(Decimal, Decimal, i64, i64)> {
        // s 25-25(4), year by year from the one the loan began: the remaining
        // expenditure over the days left, times those days in the year. Each
        // maximum is rounded to cents and carried forward at that amount, as the
        // ATO's examples are.
        precise(|| {
            let mut remaining = expenditure.clone();
            let mut from =
                dates::add_months(self.year_start, 12 * (self.first_year - self.claim_year));
            loop {
                let last = dates::add_months(from, 12) - 1;
                let end = self.end_at(last);
                let begin = self.start.max(from);
                let (left, days) = (end - begin + 1, end.min(last) - begin + 1);
                let maximum = cents(remaining.mul(&int(days))?.div(&int(left))?)?;
                if from == self.year_start {
                    return Ok((remaining, maximum, left, days));
                }
                remaining = remaining.sub(&maximum)?;
                from = last + 1;
            }
        })
    }
}

/// s 25-25(6): the deduction, not spread, in the income year the expenditure
/// was incurred.
fn not_spread(year: &str, loan: &Loan, expenditure: &Decimal, share: &Decimal) -> Result<Value> {
    let first_year = loan.first_year;
    let incurred_in = format!("{first_year}-{:02}", (first_year + 1) % 100);
    if incurred_in != year {
        return refuse(
            "borrowing.s25_25_6_not_spread",
            format!(
                "s 25-25(6) applied in the {incurred_in} income year the expenditure was incurred, so its deduction is not spread to {year}."
            ),
        );
    }
    let amounts = precise(|| {
        Ok(vec![
            ("borrowing_expenses", expenditure.clone()),
            ("remaining_expenditure", expenditure.clone()),
            ("maximum_deduction", expenditure.clone()),
            ("deduction", cents(expenditure.mul(share)?)?),
            ("remaining_expenditure_after_year", int(0)),
        ])
    })?;
    result(
        "borrowing_expenses",
        year,
        amounts,
        vec![
            rate("spread", "not spread: $100 or less (s 25-25(6))"),
            rate("income_use", share),
        ],
    )
}

/// s 25-25(4): the expenditure spread by days over the period of the loan.
fn spread(year: &str, loan: &Loan, expenditure: &Decimal, share: &Decimal) -> Result<Value> {
    let (end, basis) = loan.period(year)?;
    let (remaining, maximum, days_left, days_in_year) = loan.maximum_deduction(expenditure)?;
    let amounts = precise(|| {
        Ok(vec![
            ("borrowing_expenses", expenditure.clone()),
            ("remaining_expenditure", remaining.clone()),
            ("maximum_deduction", maximum.clone()),
            ("deduction", cents(maximum.mul(share)?)?),
            ("remaining_expenditure_after_year", remaining.sub(&maximum)?),
        ])
    })?;
    result(
        "borrowing_expenses",
        year,
        amounts,
        vec![
            rate("spread", "spread over the period of the loan (s 25-25(4))"),
            rate("income_use", share),
            rate("period_of_the_loan_from", dates::format(loan.start)),
            rate("period_of_the_loan_to", dates::format(end)),
            rate("period_of_the_loan", basis),
            rate("remaining_loan_period_days", days_left),
            rate("days_in_year", days_in_year),
        ],
    )
}

/// ITAA 1997 s 25-25: one income year's deduction for one loan's borrowing
/// expenses. Under s 25-25(6) a total of $100 or less is deducted when
/// incurred; otherwise s 25-25(4) spreads it by days over the period of the
/// loan, as determined at the end of each income year, so the year the loan
/// is repaid takes what remains.
pub fn borrowing_expenses(
    borrowing_expenses: &Arg,
    other_borrowing_expenses_in_year: &Arg,
    income_use: &Arg,
    loan: LoanDates<'_>,
    section_26_50_applies: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "borrowing_expenses")?;
    let expenditure = money(borrowing_expenses)?;
    let other = money(other_borrowing_expenses_in_year)?;
    let Some(share) = finite_between(income_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "income_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    if flag(section_26_50_applies, "section_26_50_applies")? {
        return refuse(
            "borrowing.s26_50_denied",
            "Section 26-50 denies deductions for a leisure facility, such as a holiday home, unless s 26-50(3) applies at all times in the income year; part-year use under s 26-50(4) is out of scope.",
        );
    }
    let loan = loan.parse(&year)?;

    // s 25-25(6): this loan's income-producing share plus the taxpayer's
    // other borrowing expenditure that year, at $100 or less, is not spread.
    // Money not used to produce income at all is outside s 25-25(6)(a) and
    // (b), so it is spread and nothing is deducted (s 25-25(3), note).
    let total = precise(|| Ok(expenditure.mul(&share)?.add(&other)?))?;
    if !share.is_zero() && total.le(&int(100))? {
        return not_spread(&year, &loan, &expenditure, &share);
    }
    spread(&year, &loan, &expenditure, &share)
}

/// One pay that includes an additional payment, as Schedule 5 (NAT 3348) reads it.
struct AdditionalPay {
    kind: &'static str,
    year: String,
    earnings: Decimal,
    period: String,
    scale: i64,
    study_loan: bool,
    /// Pay periods in a financial year: 52, 26 or 12, with no adjustment for a
    /// year of 53 weekly or 27 fortnightly pays.
    per_year: i64,
    /// The most pays a financial year can hold: 53, 27 or 12.
    most_pays: i64,
}

fn additional_pay(
    kind: &'static str,
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    study_loan: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<AdditionalPay> {
    let (year, earnings, period, scale) =
        payg_checks(kind, earnings, pay_period, scale, year, scope_confirmed)?;
    let study_loan = flag(study_loan, "study_loan")?;
    let loan_years = meta::supported_periods("study_loan_withholding");
    if study_loan && !loan_years.contains(&year.as_str()) {
        return refuse(
            "period.unsupported",
            format!(
                "study_loan is supported for {} only: there is no Schedule 8 study and training support loan component for {year}.",
                loan_years.join(", ")
            ),
        );
    }
    let (per_year, most_pays) = match period.as_str() {
        "weekly" => (52, 53),
        "fortnightly" => (26, 27),
        _ => (12, 12),
    };
    Ok(AdditionalPay {
        kind,
        year,
        earnings,
        period,
        scale,
        study_loan,
        per_year,
        most_pays,
    })
}

impl AdditionalPay {
    /// What the relevant tax table withholds from `earnings` for one pay: the
    /// Schedule 1 amount, plus the Schedule 8 component for a study and
    /// training support loan, which Schedule 5 allows in place of separate
    /// calculations.
    fn table(&self, earnings: &Decimal) -> Result<Decimal> {
        let args = [
            Arg::Decimal(earnings.clone()),
            Arg::Text(self.period.clone()),
            Arg::Int(self.scale),
            Arg::Text(self.year.clone()),
            Arg::Bool(true),
        ];
        let [e, p, s, y, c] = &args;
        let out = if self.study_loan {
            study_loan_withholding(e, p, s, y, c)?
        } else {
            payg_withholding(e, p, s, y, c)?
        };
        Ok(d(out["amounts"]["withholding"]
            .as_str()
            .expect("withholding amount")))
    }

    fn finish(&self, amounts: Vec<(&str, Decimal)>, extra: Vec<(String, String)>) -> Result<Value> {
        let mut rates = vec![
            rate("scale", self.scale),
            rate("study_loan", self.study_loan),
        ];
        rates.extend(extra);
        let period = format!("{} {}", self.year, self.period);
        result(self.kind, &period, amounts, rates)
    }
}

fn pay_count(value: &Arg, name: &str, low: i64, high: i64) -> Result<i64> {
    match value.int() {
        Some(Some(n)) if (low..=high).contains(&n) => Ok(n),
        _ => refuse(
            "input.pay_periods_invalid",
            format!("{name} must be an integer from {low} to {high}."),
        ),
    }
}

/// Methods A and B(ii): 47% of the additional payment, and the lesser of that
/// and `extra` as the withholding on it, cents ignored and a negative result nil.
fn within_limit(extra: Decimal, additional: &Decimal) -> Result<(Decimal, Decimal)> {
    let limit = additional.mul(&d(meta::SCHEDULE_5_LIMIT_RATE))?;
    let lesser = py_min(extra, limit.clone())?;
    let on_additional = py_max(int(0), lesser)?.quantize(0, Rounding::Down)?;
    Ok((limit, on_additional))
}

/// Method A and B(ii) steps 3 to 7: the additional payments spread over
/// `periods` pay periods, cents ignored, added to `base`, and the extra the tax
/// table then withholds for one pay, times `periods`.
fn spread_withholding(
    pay: &AdditionalPay,
    base: &Decimal,
    base_withholding: &Decimal,
    additional: &Decimal,
    periods: i64,
) -> Result<(Decimal, Decimal, Decimal)> {
    let share = additional.div(&int(periods))?.quantize(0, Rounding::Down)?;
    let with_share = pay.table(&base.add(&share)?)?;
    let extra = with_share.sub(base_withholding)?.mul(&int(periods))?;
    Ok((share, with_share, extra))
}

/// Method A steps 1 to 10.
fn method_a_amounts(
    pay: &AdditionalPay,
    additional: &Decimal,
    periods: i64,
) -> Result<Vec<(&'static str, Decimal)>> {
    precise(|| {
        let earnings = pay.earnings.quantize(0, Rounding::Down)?;
        let on_earnings = pay.table(&earnings)?;
        let (share, with_share, extra) =
            spread_withholding(pay, &earnings, &on_earnings, additional, periods)?;
        let (limit, on_additional) = within_limit(extra.clone(), additional)?;
        let total = on_earnings.add(&on_additional)?;
        Ok(vec![
            ("earnings_used", earnings),
            ("withholding_on_earnings", on_earnings),
            ("additional_payment_per_period", share),
            ("withholding_on_earnings_and_share", with_share),
            ("withholding_before_limit", extra),
            ("withholding_limit", limit),
            ("withholding_on_additional_payment", on_additional),
            ("withholding", total),
        ])
    })
}

/// Schedule 5 Method A: withholding from a pay that includes back payments,
/// commissions, bonuses or similar payments, spread over the pay periods.
#[allow(clippy::too_many_arguments)]
pub fn schedule_5_method_a(
    earnings: &Arg,
    additional_payment: &Arg,
    pay_periods: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    study_loan: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let pay = additional_pay(
        "schedule_5_method_a",
        earnings,
        pay_period,
        scale,
        study_loan,
        year,
        scope_confirmed,
    )?;
    let additional = money(additional_payment)?;
    // A payment for a single pay period is added to that pay under Schedule 1.
    let periods = pay_count(pay_periods, "pay_periods", 2, pay.per_year)?;
    let amounts = method_a_amounts(&pay, &additional, periods)?;
    pay.finish(
        amounts,
        vec![
            rate("pay_periods", periods),
            rate("limit_rate", meta::SCHEDULE_5_LIMIT_RATE),
        ],
    )
}

/// An earlier pay period a Method B(i) back payment applies to.
struct BackPaidPeriod {
    earnings: Decimal,
    back_payment: Decimal,
    withheld: Decimal,
}

fn back_payment_periods_invalid<T>(most: i64) -> Result<T> {
    refuse(
        "input.back_payment_periods_invalid",
        format!(
            "back_payment_periods must be a list of 1 to {most} objects, each with exactly earnings, back_payment and withheld as decimal strings."
        ),
    )
}

fn back_payment_periods(value: &Arg, most: i64) -> Result<Vec<BackPaidPeriod>> {
    let Arg::Json(Value::Array(items)) = value else {
        return back_payment_periods_invalid(most);
    };
    if items.is_empty() || items.len() as i64 > most {
        return back_payment_periods_invalid(most);
    }
    items
        .iter()
        .map(|item| {
            let Value::Object(fields) = item else {
                return back_payment_periods_invalid(most);
            };
            if fields.len() != 3 {
                return back_payment_periods_invalid(most);
            }
            let amount = |key: &str| match fields.get(key).and_then(Value::as_str) {
                Some(text) => match Decimal::parse(text) {
                    Ok(v) => money(&Arg::Decimal(v)),
                    Err(_) => back_payment_periods_invalid(most),
                },
                None => back_payment_periods_invalid(most),
            };
            Ok(BackPaidPeriod {
                earnings: amount("earnings")?,
                back_payment: amount("back_payment")?,
                withheld: amount("withheld")?,
            })
        })
        .collect()
}

/// Method B(i) steps 2 to 7: each earlier period recalculated, less what was
/// withheld from it, the total nil if negative, with no 47% limit.
fn method_b_i_amounts(
    pay: &AdditionalPay,
    periods: &[BackPaidPeriod],
) -> Result<Vec<(&'static str, Decimal)>> {
    precise(|| {
        let mut back_payment = int(0);
        let mut difference = int(0);
        for period in periods {
            let recalculated = pay.table(&period.earnings.add(&period.back_payment)?)?;
            difference = difference.add(&recalculated.sub(&period.withheld)?)?;
            back_payment = back_payment.add(&period.back_payment)?;
        }
        let on_back_payment = py_max(int(0), difference)?;
        let on_earnings = pay.table(&pay.earnings)?;
        let total = on_earnings.add(&on_back_payment)?;
        Ok(vec![
            ("back_payment", back_payment),
            ("withholding_on_back_payment", on_back_payment),
            ("withholding_on_earnings", on_earnings),
            ("withholding", total),
        ])
    })
}

/// Schedule 5 Method B(i): withholding from a pay that includes a back payment
/// for earlier pay periods in the same financial year, recalculated period by period.
pub fn schedule_5_method_b_i(
    earnings: &Arg,
    back_payments: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    study_loan: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let pay = additional_pay(
        "schedule_5_method_b_i",
        earnings,
        pay_period,
        scale,
        study_loan,
        year,
        scope_confirmed,
    )?;
    let periods = back_payment_periods(back_payments, pay.most_pays - 1)?;
    let amounts = method_b_i_amounts(&pay, &periods)?;
    pay.finish(amounts, vec![rate("back_payment_periods", periods.len())])
}

/// The year to date a Method B(ii) payment is averaged against.
struct YearToDate {
    earnings: Decimal,
    pays: i64,
    /// Earlier additional payments in the year worked by Method B(ii), and
    /// what was withheld from them.
    earlier: Decimal,
    earlier_withheld: Decimal,
}

/// Method B(ii) steps 1 to 12.
fn method_b_ii_amounts(
    pay: &AdditionalPay,
    additional: &Decimal,
    to_date: &YearToDate,
) -> Result<Vec<(&'static str, Decimal)>> {
    precise(|| {
        let average = to_date.earnings.div(&int(to_date.pays))?;
        let average = average.quantize(0, Rounding::Down)?;
        let on_average = pay.table(&average)?;
        let all_additional = to_date.earlier.add(additional)?;
        let (share, with_share, extra) =
            spread_withholding(pay, &average, &on_average, &all_additional, pay.per_year)?;
        // Step 8 subtracts what earlier Method B(ii) payments had withheld, and
        // can be negative; results carry no negative amounts, so it is not shown.
        let less_earlier = extra.sub(&to_date.earlier_withheld)?;
        let (limit, on_additional) = within_limit(less_earlier, additional)?;
        let on_earnings = pay.table(&pay.earnings)?;
        let total = on_earnings.add(&on_additional)?;
        Ok(vec![
            ("average_total_earnings", average),
            ("withholding_on_average_earnings", on_average),
            ("additional_payments_per_period", share),
            ("withholding_on_average_and_share", with_share),
            ("withholding_for_the_year", extra),
            ("withholding_limit", limit),
            ("withholding_on_additional_payment", on_additional),
            ("withholding_on_earnings", on_earnings),
            ("withholding", total),
        ])
    })
}

/// Schedule 5 Method B(ii): withholding from a pay that includes an additional
/// payment for a prior financial year or for no single pay period, averaged
/// over the financial year. `to_date` is the year to date's earnings, pay
/// periods, earlier Method B(ii) additional payments and their withholding.
#[allow(clippy::too_many_arguments)]
pub fn schedule_5_method_b_ii(
    earnings: &Arg,
    additional_payment: &Arg,
    to_date: [&Arg; 4],
    pay_period: &Arg,
    scale: &Arg,
    study_loan: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let pay = additional_pay(
        "schedule_5_method_b_ii",
        earnings,
        pay_period,
        scale,
        study_loan,
        year,
        scope_confirmed,
    )?;
    let additional = money(additional_payment)?;
    let [to_date_earnings, pays_to_date, earlier, earlier_withheld] = to_date;
    let to_date = YearToDate {
        earnings: money(to_date_earnings)?,
        pays: pay_count(pays_to_date, "pay_periods_to_date", 1, pay.most_pays)?,
        earlier: money(earlier)?,
        earlier_withheld: money(earlier_withheld)?,
    };
    let amounts = method_b_ii_amounts(&pay, &additional, &to_date)?;
    pay.finish(
        amounts,
        vec![
            rate("pay_periods_to_date", to_date.pays),
            rate("pay_periods", pay.per_year),
            rate("limit_rate", meta::SCHEDULE_5_LIMIT_RATE),
        ],
    )
}

const KINDS: [&str; 23] = [
    "gst",
    "resident_tax",
    "capital_gains",
    "main_residence",
    "borrowing_expenses",
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
    "division_296",
    "termination_payment",
    "schedule_5_method_a",
    "schedule_5_method_b_i",
    "schedule_5_method_b_ii",
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
        Example::Json(text) => serde_json::from_str(text).expect("JSON example"),
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
        let money_format = "Non-negative finite AUD decimal strings, at most 2 decimal places and 1000000000000.00. Resident taxable income must be whole dollars.";
        entry.insert(
            "money_format".into(),
            json!(if kind == "division_296" {
                format!("{money_format} Total superannuation earnings may be negative, down to -1000000000000.00.")
            } else {
                money_format.to_string()
            }),
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
        let optional: Map<String, Value> = meta::optional_inputs(kind)
            .into_iter()
            .map(|(k, v)| (k.into(), json!(v)))
            .collect();
        if !optional.is_empty() {
            entry.insert("optional_inputs".into(), Value::Object(optional));
        }
        out.insert(kind.into(), Value::Object(entry));
    }
    Value::Object(out)
}
