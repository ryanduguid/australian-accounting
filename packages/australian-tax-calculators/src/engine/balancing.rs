// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `balancing_adjustment`: the balancing adjustment when a balancing
//! adjustment event occurs for a depreciating asset used partly for a purpose
//! other than a taxable purpose, under ITAA 1997 Subdivision 40-D: the amount
//! assessable or deductible under s 40-285, reduced under s 40-290 (or, for a
//! car whose car expenses were claimed by both the log book and the cents per
//! kilometre methods, worked out under s 40-370 instead), the car limit
//! adjustment of s 40-325, and the capital gain or loss of CGT event K7
//! (ss 104-235 and 104-240), disregarded for a car or motor cycle (s 118-5).
//! The citations are to ITAA 1997 compilation 268 (1 October 2026), read on
//! 18 November 2026.

use serde_json::{Map, Value};

use super::business_capital::{year_first, year_text};
use super::{Arg, Result, Step, derived_result, int, money, precise, rate, refuse, scope, step};
use crate::decimal::Decimal;

const KIND: &str = "balancing_adjustment";

/// Division 40 was added by Act No 76 of 2001 and applies from 1 July 2001
/// (the compilation's endnotes): a year before 2001-02 is not one this works.
const FIRST_YEAR: i64 = 2001;

/// Act No 162 of 2015 left the cents per kilometre and log book methods as the
/// only car expense methods from the 2015-16 income year; s 40-370 names only
/// those, so an earlier car year (when the one-third of actual expenses and
/// 12% of original value methods could also apply) is refused.
const FIRST_CAR_YEAR: i64 = 2015;

/// What the asset is, for the provisions only some assets have.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Asset {
    /// A car (ITAA 1997 s 995-1): Division 28's methods, the car limit and
    /// s 118-5 apply.
    Car,
    /// A motor cycle or similar vehicle: s 118-5 applies.
    MotorCycle,
    Other,
}

/// One income year the asset was held.
struct Held {
    first: i64,
    decline: Decimal,
    /// The decline less the reductions s 40-25 required; `None` for a year the
    /// cents per kilometre method was chosen, when no decline was deducted.
    deduction: Option<Decimal>,
    days: Option<i64>,
}

/// The s 40-325 facts for a car whose cost s 40-230 reduced to the car limit.
struct CarLimit {
    limit: Decimal,
    second_element: Decimal,
    cost_ignoring_limit: Decimal,
}

/// The facts the worksheet works from, read and checked.
struct Facts {
    asset: Asset,
    cost: Decimal,
    years: Vec<Held>,
    car_limit: Option<CarLimit>,
}

fn missing<T>(at: &str, key: &str) -> Result<T> {
    refuse(
        "balancing.fact_missing",
        format!("{at}: {key} is required."),
    )
}

fn row_missing<T>(n: usize, key: &str) -> Result<T> {
    missing(&format!("years row {n}"), key)
}

fn row_invalid<T>(n: usize, key: &str, expected: &str) -> Result<T> {
    refuse(
        "balancing.fact_invalid",
        format!("years row {n}: {key} must be {expected}."),
    )
}

/// A money fact of a row (`at` names it) or of `car_limit`: a decimal string
/// that is a valid amount.
fn amount(fields: &Map<String, Value>, key: &str, at: &str) -> Result<Decimal> {
    let parsed = match fields.get(key) {
        None => return missing(at, key),
        Some(Value::String(text)) => Decimal::parse(text).ok(),
        Some(_) => None,
    };
    match parsed.map(|value| money(&Arg::Decimal(value))) {
        Some(Ok(value)) => Ok(value),
        _ => refuse(
            "input.amount_invalid",
            format!(
                "{at}: {key} must be a finite non-negative decimal string, at most 2dp and 1e12."
            ),
        ),
    }
}

/// The keys a row of this asset takes, by its method.
fn row_keys(asset: Asset, cents: bool) -> &'static [&'static str] {
    match (asset, cents) {
        (Asset::Car, true) => &["income_year", "decline_in_value", "method", "days_held"],
        (Asset::Car, false) => &[
            "income_year",
            "decline_in_value",
            "method",
            "days_held",
            "deduction",
        ],
        _ => &["income_year", "decline_in_value", "deduction"],
    }
}

/// A car row's method: true for the cents per kilometre method.
fn car_method(n: usize, fields: &Map<String, Value>) -> Result<bool> {
    match fields.get("method").map(Value::as_str) {
        None => row_missing(n, "method"),
        Some(Some("cents_per_kilometre")) => Ok(true),
        Some(Some("log_book")) => Ok(false),
        Some(_) => row_invalid(n, "method", "log_book or cents_per_kilometre"),
    }
}

fn read_year(n: usize, fields: &Map<String, Value>, asset: Asset, event: i64) -> Result<i64> {
    let Some(given) = fields.get("income_year") else {
        return row_missing(n, "income_year");
    };
    let Some(first) = given.as_str().and_then(year_first) else {
        return row_invalid(n, "income_year", "an income year such as \"2025-26\"");
    };
    if first < FIRST_YEAR || (asset == Asset::Car && first < FIRST_CAR_YEAR) {
        return refuse(
            "balancing.year_before_scope",
            format!(
                "years row {n}: {} is before this worksheet's scope: Division 40 applies from 2001-02, and for a car the worksheet starts at 2015-16, when Act No 162 of 2015 left only the cents per kilometre and log book methods that s 40-370 names; a year under the one-third of actual expenses or 12% of original value method is not one s 40-370, as compiled now, says how to count.",
                year_text(first)
            ),
        );
    }
    if first > event {
        return refuse(
            "balancing.year_after_event",
            format!(
                "years row {n}: {} is after the {} income year in which the balancing adjustment event occurred.",
                year_text(first),
                year_text(event)
            ),
        );
    }
    Ok(first)
}

fn read_days(n: usize, fields: &Map<String, Value>) -> Result<i64> {
    match fields.get("days_held").map(Value::as_i64) {
        None => row_missing(n, "days_held"),
        Some(Some(days)) if (1..=366).contains(&days) => Ok(days),
        Some(_) => row_invalid(n, "days_held", "an integer from 1 to 366"),
    }
}

/// One row of `years`, refusing a fact missing, invalid or not taken.
fn read_row(n: usize, value: &Value, asset: Asset, event: i64) -> Result<Held> {
    let Value::Object(fields) = value else {
        return years_invalid();
    };
    let cents = asset == Asset::Car && car_method(n, fields)?;
    if let Some(key) = fields
        .keys()
        .find(|k| !row_keys(asset, cents).contains(&k.as_str()))
    {
        return refuse(
            "input.field_unknown",
            format!("years row {n}: {key} is not a fact this row takes"),
        );
    }
    let first = read_year(n, fields, asset, event)?;
    let at = format!("years row {n}");
    let decline = amount(fields, "decline_in_value", &at)?;
    let deduction = if cents {
        None
    } else {
        let deduction = amount(fields, "deduction", &at)?;
        if deduction.gt(&decline)? {
            return refuse(
                "balancing.deduction_above_decline",
                format!(
                    "years row {n}: the deduction cannot exceed the year's decline in value (s 40-25)."
                ),
            );
        }
        Some(deduction)
    };
    let days = if asset == Asset::Car {
        Some(read_days(n, fields)?)
    } else {
        None
    };
    Ok(Held {
        first,
        decline,
        deduction,
        days,
    })
}

fn years_invalid<T>() -> Result<T> {
    refuse(
        "balancing.years_invalid",
        "years must be a non-empty list of objects, one for each income year the asset was held.",
    )
}

fn read_asset(asset: &Arg) -> Result<Asset> {
    match asset.text() {
        Some("car") => Ok(Asset::Car),
        Some("motor_cycle_or_similar") => Ok(Asset::MotorCycle),
        Some("other") => Ok(Asset::Other),
        _ => refuse(
            "balancing.asset_invalid",
            "asset must be car, motor_cycle_or_similar or other.",
        ),
    }
}

/// `car_limit`: required for a car, as an object or null, and refused for
/// any other asset.
fn read_car_limit(asset: Asset, given: Option<&Arg>) -> Result<Option<CarLimit>> {
    let fields = match (asset, given) {
        (Asset::Car, None) => {
            return refuse(
                "input.missing_field",
                "car_limit is required for a car: the s 40-325 facts, or null when s 40-230 did not reduce its cost to the car limit.",
            );
        }
        (Asset::Car, Some(Arg::Json(Value::Null))) | (_, None) => return Ok(None),
        (Asset::Car, Some(Arg::Json(Value::Object(fields)))) => fields,
        (Asset::Car, Some(_)) => {
            return refuse(
                "balancing.car_limit_invalid",
                "car_limit must be null or an object with limit, second_element_cost and cost_ignoring_car_limit.",
            );
        }
        (_, Some(_)) => {
            return refuse(
                "input.field_unknown",
                "car_limit is an input only for a car",
            );
        }
    };
    let keys = ["limit", "second_element_cost", "cost_ignoring_car_limit"];
    if let Some(key) = fields.keys().find(|k| !keys.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("car_limit: {key} is not a fact it takes"),
        );
    }
    let facts = CarLimit {
        limit: amount(fields, keys[0], "car_limit")?,
        second_element: amount(fields, keys[1], "car_limit")?,
        cost_ignoring_limit: amount(fields, keys[2], "car_limit")?,
    };
    let reduced = facts.limit.add(&facts.second_element)?;
    if !reduced.lt(&facts.cost_ignoring_limit)? {
        return refuse(
            "balancing.car_limit_not_reduced",
            "car_limit: the car limit plus the second element of cost must be less than the cost ignoring the car limit; otherwise s 40-230 did not reduce the cost and s 40-325 does not apply (give null).",
        );
    }
    Ok(Some(facts))
}

/// The asset's facts, checked against each other.
fn read_facts(
    cost: &Arg,
    asset: &Arg,
    years: &Arg,
    car_limit: Option<&Arg>,
    event: i64,
) -> Result<Facts> {
    let cost = money(cost)?;
    let asset = read_asset(asset)?;
    let car_limit = read_car_limit(asset, car_limit)?;
    let Arg::Json(Value::Array(list)) = years else {
        return years_invalid();
    };
    if list.is_empty() {
        return years_invalid();
    }
    let years = list
        .iter()
        .enumerate()
        .map(|(i, value)| read_row(i + 1, value, asset, event))
        .collect::<Result<Vec<_>>>()?;
    for (i, row) in years.iter().enumerate() {
        if years[..i].iter().any(|earlier| earlier.first == row.first) {
            return refuse(
                "balancing.year_repeated",
                format!(
                    "years row {}: {} is given twice.",
                    i + 1,
                    year_text(row.first)
                ),
            );
        }
    }
    Ok(Facts {
        asset,
        cost,
        years,
        car_limit,
    })
}

/// The decline since the start time, refusing one above cost or nil.
fn total_decline(facts: &Facts) -> Result<(Decimal, String)> {
    let mut total = int(0);
    for row in &facts.years {
        total = total.add(&row.decline)?;
    }
    if total.gt(&facts.cost)? {
        return refuse(
            "balancing.decline_above_cost",
            "The years' declines in value total more than cost, which would leave an adjustable value below nil (s 40-85).",
        );
    }
    if total.is_zero() {
        return refuse(
            "balancing.no_decline",
            "The years' declines in value total nil, so the fraction in s 40-290(2) and s 104-240 (sum of reductions over total decline) is undefined.",
        );
    }
    let terms: Vec<String> = facts
        .years
        .iter()
        .map(|row| row.decline.to_string())
        .collect();
    Ok((total, terms.join(" + ")))
}

/// s 40-370(4)(c): a cents per kilometre year counts its notional decline as
/// 20% for a taxable purpose, so 80% of it is a reduction.
fn reduction(row: &Held) -> Result<(Decimal, String)> {
    Ok(match &row.deduction {
        Some(deduction) => (
            row.decline.sub(deduction)?,
            format!("({} - {deduction})", row.decline),
        ),
        None => (
            row.decline.mul(&super::d("0.8"))?,
            format!("({} x (1 - 0.2))", row.decline),
        ),
    })
}

/// The sum of the s 40-25 reductions, with s 40-370(4)'s for cents per
/// kilometre years.
fn sum_of_reductions(facts: &Facts) -> Result<(Decimal, String)> {
    let mut sum = int(0);
    let mut terms = Vec::new();
    for row in &facts.years {
        let (value, term) = reduction(row)?;
        sum = sum.add(&value)?;
        terms.push(term);
    }
    Ok((sum, terms.join(" + ")))
}

const TV_GIVEN: &str = "ITAA 1997 s 40-300: the termination value, as the operator established it under ss 40-300 to 40-320 after Subdivision 27-B";
const TV_CAR_LIMIT: &str = "ITAA 1997 s 40-325: a car whose cost s 40-230 reduced to the car limit has its termination value times (car limit for the financial year first used + second element of cost) / total cost ignoring the car limit, after Subdivision 27-B";
const ADJUSTABLE: &str = "ITAA 1997 s 40-85(1)(b) and (c): cost less the decline in value since the start time, just before the event";
const TOTAL_DECLINE: &str = "ITAA 1997 s 40-290(2), total decline: the decline in value since you started to hold the asset, each year's";
const SUM_OF_REDUCTIONS: &str = "ITAA 1997 s 40-290(2), sum of reductions: each year's reduction of the deduction under s 40-25(2), the decline less the deduction; for a cents per kilometre year, s 40-370(4) assumes 20% taxable use of its notional decline";
const AMOUNT: &str = "ITAA 1997 s 40-285(1) and (2): the difference between the termination value and the adjustable value just before the event, assessable where termination value is more and deductible where it is less";
const REDUCED: &str = "ITAA 1997 s 40-290(2): the balancing adjustment amount x sum of reductions / total decline, which comes off the amount assessable or deductible";

/// The termination value the balancing adjustment uses: s 40-325's for a car
/// whose cost s 40-230 reduced.
fn termination(facts: &Facts, given: &Decimal) -> Result<Step> {
    Ok(match &facts.car_limit {
        None => step(
            "termination_value",
            TV_GIVEN,
            "termination_value",
            &[given],
            given.clone(),
        ),
        Some(car) => {
            let numerator = car.limit.add(&car.second_element)?;
            let value = given.mul(&numerator)?.div(&car.cost_ignoring_limit)?;
            step(
                "termination_value",
                TV_CAR_LIMIT,
                "termination_value x (car_limit + second_element_cost) / cost_ignoring_car_limit",
                &[
                    given,
                    &car.limit,
                    &car.second_element,
                    &car.cost_ignoring_limit,
                ],
                value,
            )
        }
    })
}

/// How the amount is worked out once reduced: s 40-285 with s 40-290, or for
/// a car whose expenses were claimed by both car expense methods s 40-370.
enum Path {
    General,
    /// s 40-370: the days the decline was deducted and the days held.
    Switched(i64, i64),
    /// s 40-370(1) Note 2: only the cents per kilometre method was chosen.
    CentsOnly,
}

fn path(facts: &Facts) -> Result<Path> {
    if facts.asset != Asset::Car {
        return Ok(Path::General);
    }
    let held: i64 = facts.years.iter().filter_map(|row| row.days).sum();
    let logged: Vec<&Held> = facts
        .years
        .iter()
        .filter(|row| row.deduction.is_some())
        .collect();
    if logged.len() == facts.years.len() {
        return Ok(Path::General);
    }
    if logged.is_empty() {
        return Ok(Path::CentsOnly);
    }
    if logged
        .iter()
        .any(|row| row.deduction.as_ref().is_some_and(Decimal::is_zero))
    {
        return refuse(
            "balancing.days_deducted_unclear",
            "A log book year with a nil deduction: s 40-370 step 3 counts the days for which you deducted the decline in value, and a nil deduction leaves unclear whether that year's days count.",
        );
    }
    let deducted: i64 = logged.iter().filter_map(|row| row.days).sum();
    Ok(Path::Switched(deducted, held))
}

const SWITCHED: &str = "ITAA 1997 s 40-370(2) steps 1 to 5: (termination value - adjustable value), reduced by the s 40-290(2) formula, x days the decline was deducted / days held";
const CENTS_ONLY: &str = "ITAA 1997 s 40-370(1) Note 2: only the cents per kilometre method was chosen, so no amount is assessable or deductible under s 40-370 or s 40-285";

/// The assessable or deductible amount, never below nil: `from - to`, reduced
/// for non-taxable use and, under s 40-370, scaled by the days, worked with
/// one division, last.
fn reduced_amount(
    id: &'static str,
    path: &Path,
    from: &Decimal,
    to: &Decimal,
    totals: [&Decimal; 2],
) -> Result<Step> {
    let [total, sum] = totals;
    let numerator = from.sub(to)?.mul(&total.sub(sum)?)?;
    let names = if id == "assessable_amount" {
        "termination_value - adjustable_value"
    } else {
        "adjustable_value - termination_value"
    };
    let formula = format!("max(0, ({names}) x (total_decline - sum_of_reductions) / total_decline");
    Ok(match path {
        Path::CentsOnly => step(id, CENTS_ONLY, "0", &[], int(0)),
        Path::General => {
            let value = super::py_max(int(0), numerator.div(total)?)?;
            step(
                id,
                AMOUNT,
                &format!("{formula})"),
                &[from, to, total, sum, total],
                value,
            )
        }
        Path::Switched(deducted, held) => {
            let (deducted, held) = (int(*deducted), int(*held));
            let value = numerator.mul(&deducted)?.div(&total.mul(&held)?)?;
            let value = super::py_max(int(0), value)?;
            let formula = format!("{formula} x days_deducted / days_held)");
            step(
                id,
                SWITCHED,
                &formula,
                &[from, to, total, sum, total, &deducted, &held],
                value,
            )
        }
    })
}

const K7_GAIN: &str = "ITAA 1997 s 104-240(1): CGT event K7, (termination value - cost) x sum of reductions / total decline, where termination value exceeds cost; before any exemption, the discount and the CGT changes from 1 July 2027";
const K7_LOSS: &str = "ITAA 1997 s 104-240(2): CGT event K7, (cost - termination value) x sum of reductions / total decline, where cost exceeds termination value; before any exemption";
const K7_DISREGARDED: &str = "ITAA 1997 s 118-5(a): a capital gain or loss from a car, motor cycle or similar vehicle is disregarded";

fn k7(facts: &Facts, tv: &Decimal, total: &Decimal, sum: &Decimal) -> Result<[Step; 2]> {
    if facts.asset != Asset::Other {
        let nil = |id| step(id, K7_DISREGARDED, "0", &[], int(0));
        return Ok([nil("capital_gain"), nil("capital_loss")]);
    }
    let cost = &facts.cost;
    let gain = super::py_max(int(0), tv.sub(cost)?)?.mul(sum)?.div(total)?;
    let loss = super::py_max(int(0), cost.sub(tv)?)?.mul(sum)?.div(total)?;
    Ok([
        step(
            "capital_gain",
            K7_GAIN,
            "max(0, termination_value - cost) x sum_of_reductions / total_decline",
            &[tv, cost, sum, total],
            gain,
        ),
        step(
            "capital_loss",
            K7_LOSS,
            "max(0, cost - termination_value) x sum_of_reductions / total_decline",
            &[cost, tv, sum, total],
            loss,
        ),
    ])
}

/// The worksheet's steps and rates from read facts and the termination value
/// worked out under ss 40-300 to 40-320.
fn worked(facts: &Facts, given_tv: &Decimal) -> Result<(Vec<Step>, Vec<(String, String)>)> {
    let path = path(facts)?;
    precise(|| {
        let tv_step = termination(facts, given_tv)?;
        let tv = tv_step.value.clone();
        let (total, total_terms) = total_decline(facts)?;
        let (sum, sum_terms) = sum_of_reductions(facts)?;
        let adjustable = facts.cost.sub(&total)?;
        let amount = super::py_max(tv.sub(&adjustable)?, adjustable.sub(&tv)?)?;
        let reduced = amount.mul(&sum)?.div(&total)?;
        let assessable =
            reduced_amount("assessable_amount", &path, &tv, &adjustable, [&total, &sum])?;
        let deductible =
            reduced_amount("deductible_amount", &path, &adjustable, &tv, [&total, &sum])?;
        let [gain, loss] = k7(facts, &tv, &total, &sum)?;
        let steps = vec![
            tv_step,
            step(
                "total_decline",
                TOTAL_DECLINE,
                &total_terms,
                &[],
                total.clone(),
            ),
            step(
                "adjustable_value",
                ADJUSTABLE,
                "cost - total_decline",
                &[&facts.cost, &total],
                adjustable.clone(),
            ),
            step(
                "sum_of_reductions",
                SUM_OF_REDUCTIONS,
                &sum_terms,
                &[],
                sum.clone(),
            ),
            step(
                "balancing_adjustment_amount",
                AMOUNT,
                "max(termination_value - adjustable_value, adjustable_value - termination_value)",
                &[&tv, &adjustable, &adjustable, &tv],
                amount.clone(),
            ),
            step(
                "reduction_for_non_taxable_use",
                REDUCED,
                "balancing_adjustment_amount x sum_of_reductions / total_decline",
                &[&amount, &sum, &total],
                reduced,
            ),
            assessable,
            deductible,
            gain,
            loss,
        ];
        let mut rates = vec![rate("years_held", facts.years.len())];
        let provision = match path {
            Path::General => "ITAA 1997 s 40-285 and s 40-290",
            Path::Switched(deducted, held) => {
                rates.push(rate("days_deducted", deducted));
                rates.push(rate("days_held", held));
                "ITAA 1997 s 40-370"
            }
            Path::CentsOnly => "ITAA 1997 s 40-370(1) Note 2",
        };
        rates.push(rate("provision", provision));
        Ok((steps, rates))
    })
}

#[allow(clippy::too_many_arguments)]
pub fn balancing_adjustment(
    cost: &Arg,
    termination_value: &Arg,
    asset: &Arg,
    years: &Arg,
    car_limit: Option<&Arg>,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let event = year_first(&year).expect("a supported income year");
    let given_tv = money(termination_value)?;
    let facts = read_facts(cost, asset, years, car_limit, event)?;
    let (steps, rates) = worked(&facts, &given_tv)?;
    derived_result(KIND, &year, steps, rates)
}
