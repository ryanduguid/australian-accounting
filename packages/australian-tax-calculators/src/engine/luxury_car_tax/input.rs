// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! A luxury car tax line's stated facts, checked.

use serde_json::{Map, Value};

use super::thresholds;
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, money, refuse};

/// The lines the command works out, each with the fields it takes.
const KINDS: [(&str, &[&str]); 6] = [
    (
        "sale",
        &[
            "kind",
            "luxury_car_tax_value",
            "fuel_consumption_litres_per_100km",
            "used_before_1_july_2025",
            "lct_previously_payable",
        ],
    ),
    (
        "importation",
        &[
            "kind",
            "luxury_car_tax_value",
            "fuel_consumption_litres_per_100km",
            "used_before_1_july_2025",
        ],
    ),
    (
        "price_adjustment",
        &[
            "kind",
            "supplied_in",
            "corrected_luxury_car_tax_value",
            "fuel_consumption_litres_per_100km",
            "used_before_1_july_2025",
            "lct_previously_payable",
            "previously_attributed_lct",
        ],
    ),
    (
        "change_of_use_increasing",
        &[
            "kind",
            "supplied_in",
            "luxury_car_tax_value",
            "fuel_consumption_litres_per_100km",
            "used_before_1_july_2025",
            "lct_previously_payable",
        ],
    ),
    (
        "change_of_use_decreasing",
        &["kind", "lct_payable_on_supply"],
    ),
    (
        "refund",
        &[
            "kind",
            "supplied_in",
            "claimant",
            "refund_eligible_car",
            "lct_borne",
        ],
    ),
];

pub(super) const CLAIMANTS: [&str; 2] = ["primary_producer", "tourism_operator"];

/// A car's value and the facts that pick its threshold.
pub(super) struct Car {
    pub(super) year: &'static str,
    pub(super) value: Decimal,
    /// The combined fuel consumption rating, if the car has one.
    pub(super) consumption: Option<Decimal>,
    /// For a supply from 1 July 2025: an earlier supply or importation and a
    /// non-quotable use, both before that day (Act No. 29 of 2025 Sch 1 item 4(2)).
    pub(super) used_before: bool,
}

/// A line's checked facts.
pub(super) struct Line {
    pub(super) kind: &'static str,
    pub(super) car: Option<Car>,
    /// lct_previously_payable, lct_payable_on_supply or lct_borne, as the
    /// kind takes.
    pub(super) previous: Decimal,
    pub(super) attributed: Decimal,
    pub(super) refund: Option<(&'static str, bool)>,
}

fn invalid<T>(message: String) -> Result<T> {
    refuse("luxury_car_tax.line_invalid", message)
}

fn required<'v>(fields: &'v Map<String, Value>, n: usize, key: &str) -> Result<&'v Value> {
    match fields.get(key) {
        Some(value) => Ok(value),
        None => refuse(
            "input.missing_field",
            format!("line {n}: {key} is required"),
        ),
    }
}

/// An amount in AUD, a decimal string as every money input is.
fn amount(fields: &Map<String, Value>, n: usize, key: &str) -> Result<Decimal> {
    let parsed = match required(fields, n, key)? {
        Value::String(text) if text.len() <= 32 => Decimal::parse(text).ok(),
        _ => None,
    };
    match parsed.map(|v| money(&Arg::Decimal(v))) {
        Some(Ok(v)) => Ok(v),
        _ => refuse(
            "input.amount_invalid",
            format!(
                "line {n}: {key} must be a decimal string, finite and non-negative, at most 2dp and 1e12."
            ),
        ),
    }
}

fn optional_amount(fields: &Map<String, Value>, n: usize, key: &str) -> Result<Decimal> {
    if fields.contains_key(key) {
        amount(fields, n, key)
    } else {
        Ok(crate::engine::int(0))
    }
}

fn boolean(value: &Value, n: usize, key: &str) -> Result<bool> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => refuse(
            "input.flag_invalid",
            format!("line {n}: {key} must be a boolean."),
        ),
    }
}

/// The combined rating in litres per 100 km, or `None` for a car without one.
fn consumption(fields: &Map<String, Value>, n: usize) -> Result<Option<Decimal>> {
    let key = "fuel_consumption_litres_per_100km";
    let parsed = match required(fields, n, key)? {
        Value::Null => return Ok(None),
        Value::String(text) if text.len() <= 32 => Decimal::parse(text).ok(),
        _ => None,
    };
    let valid = |v: &Decimal| -> Result<bool> {
        Ok(v.is_finite()
            && !v.lt(&crate::engine::int(0))?
            && !v.gt(&crate::engine::int(100))?
            && v.exponent() >= -2)
    };
    match parsed {
        Some(v) if valid(&v)? => Ok(Some(v)),
        _ => refuse(
            "luxury_car_tax.consumption_invalid",
            format!(
                "line {n}: {key} must be null (no combined rating) or a decimal string from 0 to 100 with at most 2 decimal places."
            ),
        ),
    }
}

/// The financial year a line's supply or importation was in: the tax
/// period's year, or for an adjustment `supplied_in`, a year the table
/// holds and not after it.
fn supply_year(fields: &Map<String, Value>, n: usize, year: &'static str) -> Result<&'static str> {
    let Some(given) = fields.get("supplied_in") else {
        return Ok(year);
    };
    match thresholds::year(given.as_str().unwrap_or("")) {
        Some(found) if found <= year => Ok(found),
        _ => refuse(
            "luxury_car_tax.year_invalid",
            format!(
                "line {n}: supplied_in must be a financial year from {} to {year}, the years whose thresholds the ATO publishes, and not after the tax period's.",
                thresholds::first_year()
            ),
        ),
    }
}

/// The car's facts, `value_key` naming its luxury car tax value.
fn car(fields: &Map<String, Value>, n: usize, year: &'static str, value_key: &str) -> Result<Car> {
    let supplied = supply_year(fields, n, year)?;
    let value = amount(fields, n, value_key)?;
    let consumption = consumption(fields, n)?;
    let between = |c: &Decimal| -> Result<bool> {
        Ok(c.gt(&crate::engine::d("3.5"))? && !c.gt(&crate::engine::int(7))?)
    };
    let matters =
        supplied >= "2025-26" && consumption.as_ref().map(between).transpose()? == Some(true);
    let used_before = match (fields.get("used_before_1_july_2025"), matters) {
        (Some(v), true) => boolean(v, n, "used_before_1_july_2025")?,
        (None, true) => {
            return refuse(
                "input.missing_field",
                format!(
                    "line {n}: used_before_1_july_2025 is required for a car rated above 3.5 and at most 7 litres per 100 km supplied or imported from 1 July 2025: the 7 litre limit continues only if, before that day, the car was supplied or imported and used in Australia other than for a quotable purpose (Act No. 29 of 2025 Sch 1 item 4(2))"
                ),
            );
        }
        (Some(_), false) => {
            return refuse(
                "input.field_unknown",
                format!(
                    "line {n}: used_before_1_july_2025 is an input only for a car rated above 3.5 and at most 7 litres per 100 km supplied or imported from 1 July 2025"
                ),
            );
        }
        (None, false) => false,
    };
    Ok(Car {
        year: supplied,
        value,
        consumption,
        used_before,
    })
}

/// A refund line's claimant and whether the car is refund-eligible, for a
/// car supplied or imported from 1 July 2019 (in `year` or `supplied_in`).
fn refund(
    fields: &Map<String, Value>,
    n: usize,
    year: &'static str,
) -> Result<(&'static str, bool)> {
    if supply_year(fields, n, year)? < "2019-20" {
        return refuse(
            "luxury_car_tax.refund_invalid",
            format!(
                "line {n}: the refund of the lesser of the tax and $10,000 is for a refund-eligible car supplied or imported from 1 July 2019 (LCT Act ss 18-5(3) and 18-10(3) as substituted by Act No. 94 of 2019); the earlier refund of 8/33 of the tax, at most $3,000, is not worked."
            ),
        );
    }
    let claimant = required(fields, n, "claimant")?;
    let Some(found) = CLAIMANTS.iter().find(|c| claimant.as_str() == Some(**c)) else {
        return refuse(
            "luxury_car_tax.refund_invalid",
            format!("line {n}: refund claimant must be primary_producer or tourism_operator."),
        );
    };
    let eligible = boolean(
        required(fields, n, "refund_eligible_car")?,
        n,
        "refund_eligible_car",
    )?;
    Ok((found, eligible))
}

/// `value`'s fields, refusing a key outside `allowed`.
fn only<'v>(
    value: &'v Value,
    n: usize,
    what: &str,
    allowed: &[&str],
) -> Result<&'v Map<String, Value>> {
    let Value::Object(fields) = value else {
        return invalid(format!("line {n}: {what} must be an object."));
    };
    if let Some(key) = fields.keys().find(|k| !allowed.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("line {n}: {key} is not an input of {what}"),
        );
    }
    Ok(fields)
}

/// Line `n`'s facts, in the tax period's financial year `year`.
pub(super) fn parse_line(value: &Value, n: usize, year: &'static str) -> Result<Line> {
    let Value::Object(raw) = value else {
        return invalid(format!("line {n}: a line must be an object."));
    };
    let given = raw.get("kind").and_then(Value::as_str);
    let Some((kind, allowed)) = KINDS.iter().find(|(k, _)| Some(*k) == given) else {
        return refuse(
            "luxury_car_tax.kind_invalid",
            format!(
                "line {n}: kind must be sale, importation, price_adjustment, change_of_use_increasing, change_of_use_decreasing or refund."
            ),
        );
    };
    let fields = only(value, n, kind, allowed)?;
    let car = match *kind {
        "change_of_use_decreasing" | "refund" => None,
        "price_adjustment" => Some(car(fields, n, year, "corrected_luxury_car_tax_value")?),
        _ => Some(car(fields, n, year, "luxury_car_tax_value")?),
    };
    let (previous, attributed) = match *kind {
        "change_of_use_decreasing" => (
            amount(fields, n, "lct_payable_on_supply")?,
            crate::engine::int(0),
        ),
        "refund" => (amount(fields, n, "lct_borne")?, crate::engine::int(0)),
        "price_adjustment" => (
            optional_amount(fields, n, "lct_previously_payable")?,
            amount(fields, n, "previously_attributed_lct")?,
        ),
        _ => (
            optional_amount(fields, n, "lct_previously_payable")?,
            crate::engine::int(0),
        ),
    };
    Ok(Line {
        kind,
        car,
        previous,
        attributed,
        refund: match *kind {
            "refund" => Some(refund(fields, n, year)?),
            _ => None,
        },
    })
}
