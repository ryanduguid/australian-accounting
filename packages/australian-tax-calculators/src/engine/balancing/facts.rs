// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `balancing_adjustment`'s facts: the asset, its cost, each income year held
//! and a car's s 40-325 facts, read from the input and checked against each
//! other before anything is worked out.

use serde_json::{Map, Value};

use crate::decimal::Decimal;
use crate::engine::business_capital::{year_first, year_text};
use crate::engine::{Arg, Result, money, refuse};

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
pub(super) enum Asset {
    /// A car (ITAA 1997 s 995-1): Division 28's methods, the car limit and
    /// s 118-5 apply.
    Car,
    /// A motor cycle or similar vehicle: s 118-5 applies.
    MotorCycle,
    Other,
}

/// One income year the asset was held.
pub(super) struct Held {
    first: i64,
    pub(super) decline: Decimal,
    /// The decline less the reductions s 40-25 required; `None` for a year the
    /// cents per kilometre method was chosen, when no decline was deducted.
    pub(super) deduction: Option<Decimal>,
    pub(super) days: Option<i64>,
}

/// The s 40-325 facts for a car whose cost s 40-230 reduced to the car limit.
pub(super) struct CarLimit {
    pub(super) limit: Decimal,
    pub(super) second_element: Decimal,
    pub(super) cost_ignoring_limit: Decimal,
}

/// The facts the worksheet works from, read and checked.
pub(super) struct Facts {
    pub(super) asset: Asset,
    pub(super) cost: Decimal,
    pub(super) years: Vec<Held>,
    pub(super) car_limit: Option<CarLimit>,
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

/// A car row's days held, from 1 to the days in its income year (366 when it
/// includes 29 February).
fn read_days(n: usize, fields: &Map<String, Value>, first: i64) -> Result<i64> {
    let most = if crate::engine::is_leap(first + 1) {
        366
    } else {
        365
    };
    match fields.get("days_held").map(Value::as_i64) {
        None => row_missing(n, "days_held"),
        Some(Some(days)) if (1..=most).contains(&days) => Ok(days),
        Some(_) => row_invalid(
            n,
            "days_held",
            &format!(
                "an integer from 1 to {most}, the days in {}",
                year_text(first)
            ),
        ),
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
        Some(read_days(n, fields, first)?)
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
pub(super) fn read_facts(
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
