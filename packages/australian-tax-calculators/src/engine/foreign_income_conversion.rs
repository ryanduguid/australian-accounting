// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `foreign_income_conversion`: foreign currency amounts translated into
//! Australian dollars under ITAA 1997 s 960-50 for 2020-21 to 2026-27, at a
//! rate the operator states with its source. The special translation rules
//! are s 960-50(6) items 2, 5, 6, 7, 8 (as substituted) and 11, the same in
//! compilations 210 (1 July 2020) and 268 (1 October 2026); the average rate
//! is item 12 and Schedule 2 of the Income Tax Assessment Regulations 1997
//! (2020-21) or the Income Tax Assessment (1997 Act) Regulations 2021 (from
//! 2021-22, s 1000-1.02). All were read on 22 November 2026.

use serde_json::{Map, Value, json};

use super::{Arg, Result, d, int, refuse, result, scope};
use crate::dates;
use crate::decimal::Decimal;

mod item;

use item::{Item, Kind};

const KIND: &str = "foreign_income_conversion";

/// The largest number of amounts the command takes.
const MAX_ITEMS: usize = 200;

/// Each kind's total, in the order the result gives them.
const TOTALS: [(Kind, &str); 6] = [
    (Kind::OrdinaryIncome, "ordinary_income"),
    (Kind::StatutoryIncome, "statutory_income"),
    (Kind::Deduction, "deductions"),
    (Kind::DepreciatingAssetCost, "depreciating_asset_costs"),
    (Kind::CgtEvent, "cgt_amounts"),
    (Kind::ForeignTaxPaid, "foreign_tax_paid"),
];

const AVERAGE_CAPITAL: &str = "An average rate is used for a CGT amount or a depreciating asset's cost: the ATO's 'General information on average rates' (examples 5 and 7) says an average would not reasonably approximate the rate for a one-off purchase or sale of a large capital asset, which Schedule 2 requires before an average is used.";

pub fn foreign_income_conversion(items: &Arg, year: &Arg, scope_confirmed: &Arg) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let Arg::Json(Value::Array(given)) = items else {
        return refuse(
            "foreign_income_conversion.items_invalid",
            "items must be a list of objects.",
        );
    };
    if given.is_empty() || given.len() > MAX_ITEMS {
        return refuse(
            "foreign_income_conversion.items_invalid",
            format!("items must hold 1 to {MAX_ITEMS} amounts."),
        );
    }
    let read = given
        .iter()
        .enumerate()
        .map(|(i, value)| Item::read(value, &format!("item {}", i + 1)))
        .collect::<Result<Vec<_>>>()?;
    let mut totals = Vec::new();
    for (kind, key) in TOTALS {
        let total = read
            .iter()
            .filter(|i| i.kind == kind)
            .try_fold(int(0), |sum, i| sum.add(&i.aud))?;
        totals.push((key, total));
    }
    let mut out = result(KIND, &year, totals, Vec::new())?;
    out["items"] = json!(read.iter().map(Item::render).collect::<Vec<_>>());
    out["average_rate_rule"] = if read.iter().any(Item::is_average) {
        json!(average_rule(&year))
    } else {
        Value::Null
    };
    if read.iter().any(Item::average_capital) {
        warn(&mut out, AVERAGE_CAPITAL);
    }
    Ok(out)
}

/// Item 12 and the average rate clause for the year's regulations: the 1997
/// Regulations for 2020-21, which began before the 2021 Regulations commenced
/// on 1 April 2021 (2021 Regulations s 1000-1.02).
fn average_rule(year: &str) -> &'static str {
    if year == "2020-21" {
        "ITAA 1997 s 960-50(6) item 12, inserted by the Income Tax Assessment Regulations 1997 reg 960-50.01(1), and Schedule 2 clause 1.3: an average of the rates during a period chosen, not exceeding 12 months, used for amounts relating to that period, only where it appears on reasonable grounds to approximate the rate the other item gives, from a source that is not an associate or the entity itself"
    } else {
        "ITAA 1997 s 960-50(6) item 12, inserted by the Income Tax Assessment (1997 Act) Regulations 2021 s 960-50.01(2), and Schedule 2 clause 3: an average of the rates during a period chosen, not exceeding 12 months, used for amounts relating to that period, only where it appears on reasonable grounds to approximate the rate the other item gives, from a source that is not an associate or the entity itself"
    }
}

fn warn(out: &mut Value, note: &str) {
    out["warnings"]
        .as_array_mut()
        .expect("warnings")
        .push(json!(note));
}

/// A decimal string of `fields[key]`, finite, at most `places` decimal places
/// and at most 1e12; above zero when `positive`.
fn decimal(
    fields: &Map<String, Value>,
    name: &str,
    key: &str,
    places: i128,
    positive: bool,
) -> Result<Decimal> {
    let parsed = match fields.get(key) {
        Some(Value::String(text)) if text.len() <= 32 => Decimal::parse_input(text).ok(),
        None => return refuse("input.missing_field", format!("{name}: {key} is required")),
        _ => None,
    };
    let valid = |v: &Decimal| -> Result<bool> {
        let floor = if positive {
            v.gt(&int(0))?
        } else {
            !v.lt(&int(0))?
        };
        Ok(floor && v.le(&d("1e12"))? && v.exponent() >= -places)
    };
    match parsed {
        Some(v) if valid(&v)? => Ok(v),
        _ => refuse(
            "input.amount_invalid",
            format!(
                "{name}: {key} must be a decimal string{}, at most {places}dp and 1e12.",
                if positive {
                    " above zero"
                } else {
                    ", not negative"
                }
            ),
        ),
    }
}

/// A `YYYY-MM-DD` date in `fields[key]`, or `None` when absent.
fn date(fields: &Map<String, Value>, name: &str, key: &str) -> Result<Option<i64>> {
    match fields.get(key) {
        None => Ok(None),
        Some(Value::String(text)) if dates::parse(text).is_some() => Ok(dates::parse(text)),
        Some(_) => refuse(
            "input.date_invalid",
            format!("{name}: {key} must be a YYYY-MM-DD date from 1900 to 2100."),
        ),
    }
}

/// A non-empty string of at most `limit` characters in `fields[key]`.
fn text<'v>(
    fields: &'v Map<String, Value>,
    name: &str,
    key: &str,
    limit: usize,
) -> Result<&'v str> {
    match fields.get(key) {
        Some(Value::String(t)) if !t.trim().is_empty() && t.chars().count() <= limit => Ok(t),
        None => refuse("input.missing_field", format!("{name}: {key} is required")),
        Some(_) => refuse(
            "foreign_income_conversion.text_invalid",
            format!("{name}: {key} must be text of 1 to {limit} characters."),
        ),
    }
}
