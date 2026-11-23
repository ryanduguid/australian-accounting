// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Reading `deferred_tax`'s items and signing each one's temporary
//! differences (AASB 112 paras 5, 7 and 8).

use serde_json::{Map, Value, json};

use super::super::{Arg, Result, int, money, refuse};
use super::{Calc, Decimal, cents};

/// The most items one worksheet takes.
const MOST: usize = 500;
const FIELDS: [&str; 6] = [
    "label",
    "class",
    "carrying_amount",
    "tax_base",
    "opening_carrying_amount",
    "opening_tax_base",
];

/// One asset or liability: its carrying amounts and tax bases at the end and
/// start of the year.
pub(super) struct Item {
    label: String,
    liability: bool,
    amounts: [Decimal; 4],
}

fn invalid<T>(why: String) -> Result<T> {
    refuse("deferred_tax.item_invalid", why)
}

pub(super) fn read(given: &Arg) -> Result<Vec<Item>> {
    let Arg::Json(Value::Array(list)) = given else {
        return invalid(format!("items must be a list of 1 to {MOST} objects."));
    };
    if list.is_empty() || list.len() > MOST {
        return invalid(format!("items must be a list of 1 to {MOST} objects."));
    }
    list.iter()
        .enumerate()
        .map(|(i, v)| item(i + 1, v))
        .collect()
}

fn item(n: usize, value: &Value) -> Result<Item> {
    let Some(fields) = value.as_object() else {
        return invalid(format!("item {n} must be an object."));
    };
    if let Some(key) = fields.keys().find(|k| !FIELDS.contains(&k.as_str())) {
        return invalid(format!("item {n}: {key} is not a field of an item."));
    }
    let label = match fields.get("label") {
        Some(Value::String(s)) if !s.trim().is_empty() && s.chars().count() <= 200 => s.clone(),
        _ => {
            return invalid(format!(
                "item {n}: label must be a non-empty string of at most 200 characters."
            ));
        }
    };
    let liability = match fields.get("class").and_then(Value::as_str) {
        Some("asset") => false,
        Some("liability") => true,
        _ => return invalid(format!("item {n}: class must be asset or liability.")),
    };
    let mut amounts = Vec::with_capacity(4);
    for key in &FIELDS[2..] {
        let parsed = fields
            .get(*key)
            .and_then(Value::as_str)
            .and_then(|t| Decimal::parse_input(t).ok());
        match parsed.map(|v| money(&Arg::Decimal(v))) {
            Some(Ok(v)) => amounts.push(v),
            _ => {
                return invalid(format!(
                    "item {n}: {key} must be a finite non-negative decimal string, at most 2dp and 1e12."
                ));
            }
        }
    }
    let amounts: [Decimal; 4] = amounts.try_into().expect("4 amounts");
    Ok(Item {
        label,
        liability,
        amounts,
    })
}

/// The signed temporary difference: positive taxable, negative deductible.
fn signed(liability: bool, carrying: &Decimal, base: &Decimal) -> Calc<Decimal> {
    if liability {
        base.sub(carrying)
    } else {
        carrying.sub(base)
    }
}

/// (taxable, deductible) parts of a signed difference, each not below nil.
fn parts(v: &Decimal) -> Calc<(Decimal, Decimal)> {
    Ok(if v.lt(&int(0))? {
        (int(0), v.neg()?)
    } else {
        (v.clone(), int(0))
    })
}

fn nature(v: &Decimal) -> Calc<&'static str> {
    Ok(if v.is_zero() {
        "none"
    } else if v.lt(&int(0))? {
        "deductible"
    } else {
        "taxable"
    })
}

/// The parts at each date.
pub(super) type Rows = (Vec<(Decimal, Decimal)>, Vec<(Decimal, Decimal)>);

/// Each item's taxable and deductible parts at the end and start of the year,
/// and a line for each item in the result.
pub(super) fn differences(items: &[Item]) -> Result<(Rows, Vec<Map<String, Value>>)> {
    let mut rows: Rows = (Vec::new(), Vec::new());
    let mut lines = Vec::new();
    for it in items {
        let [carrying, base, opening_carrying, opening_base] = &it.amounts;
        let closing = signed(it.liability, carrying, base)?;
        let opening = signed(it.liability, opening_carrying, opening_base)?;
        rows.0.push(parts(&closing)?);
        rows.1.push(parts(&opening)?);
        let mut line = Map::new();
        line.insert("label".into(), json!(it.label));
        line.insert(
            "class".into(),
            json!(if it.liability { "liability" } else { "asset" }),
        );
        line.insert(
            "temporary_difference".into(),
            json!(cents(closing.clone())?.to_string()),
        );
        line.insert("nature".into(), json!(nature(&closing)?));
        line.insert(
            "opening_temporary_difference".into(),
            json!(cents(opening.clone())?.to_string()),
        );
        line.insert("opening_nature".into(), json!(nature(&opening)?));
        lines.push(line);
    }
    Ok((rows, lines))
}
