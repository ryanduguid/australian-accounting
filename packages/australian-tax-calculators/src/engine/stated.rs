// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The stated facts of one item in a list of dealings or lines, read alike by
//! the commands that take such lists. `item` names it in a refusal, such as
//! `dealing 3` or `line 3`.

use serde_json::{Map, Value};

use crate::decimal::Decimal;
use crate::engine::{Arg, Result, money, refuse};

pub(super) fn required<'v>(
    fields: &'v Map<String, Value>,
    item: &str,
    key: &str,
) -> Result<&'v Value> {
    match fields.get(key) {
        Some(value) => Ok(value),
        None => refuse("input.missing_field", format!("{item}: {key} is required")),
    }
}

/// An amount in AUD, a decimal string as every money input is.
pub(super) fn amount(fields: &Map<String, Value>, item: &str, key: &str) -> Result<Decimal> {
    let parsed = match required(fields, item, key)? {
        Value::String(text) if text.len() <= 32 => Decimal::parse(text).ok(),
        _ => None,
    };
    match parsed.map(|v| money(&Arg::Decimal(v))) {
        Some(Ok(v)) => Ok(v),
        _ => refuse(
            "input.amount_invalid",
            format!(
                "{item}: {key} must be a decimal string, finite and non-negative, at most 2dp and 1e12."
            ),
        ),
    }
}

/// Refuses a key of `fields` outside `allowed`, `what` naming the object.
pub(super) fn known_keys(
    fields: &Map<String, Value>,
    item: &str,
    what: &str,
    allowed: &[&str],
) -> Result<()> {
    match fields.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(key) => refuse(
            "input.field_unknown",
            format!("{item}: {key} is not an input of {what}"),
        ),
        None => Ok(()),
    }
}
