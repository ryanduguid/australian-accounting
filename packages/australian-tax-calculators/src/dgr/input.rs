// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! One donation's inputs, checked or refused.
use super::register::abn_digits;
use super::{COMMAND, FIELDS};
use crate::api::{Refusal, optional_decimal};
use crate::dates;
use crate::decimal::Decimal;
use crate::engine;
use serde_json::{Map, Value};

const NAME_RULE: &str = "must be a string of 1 to 200 characters with an ASCII letter or digit";
const DATE_RULE: &str = "must be a YYYY-MM-DD date from 1900 to 2100";

/// One donation's inputs, checked.
pub(super) struct Donation {
    pub(super) abn: Option<String>,
    pub(super) name: Option<String>,
    pub(super) date: Option<String>,
    pub(super) amount: Option<Decimal>,
}

pub(super) fn read_donation(at: usize, value: &Value) -> Result<Donation, Refusal> {
    let field = |key: &str| format!("donations[{at}].{key}");
    let object = donation_object(at, value)?;
    let text = |key: &str, code: &'static str, why: &str| match object.get(key) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(Refusal::new(code, format!("{} {why}", field(key)))),
    };
    let abn = text("abn", "input.abn_invalid", "must be a string")?;
    let abn = abn
        .map(|given| checked_abn(&given, &field("abn")))
        .transpose()?;
    let name = text("name", "input.name_invalid", NAME_RULE)?;
    if name.as_deref().is_some_and(|n| !valid_name(n)) {
        return Err(Refusal::new(
            "input.name_invalid",
            format!("{} {NAME_RULE}", field("name")),
        ));
    }
    if abn.is_none() && name.is_none() {
        return Err(Refusal::new(
            "input.missing_field",
            format!("donations[{at}] needs an abn or a name"),
        ));
    }
    let date = text("date", "input.date_invalid", DATE_RULE)?;
    if date.as_deref().is_some_and(|d| dates::parse(d).is_none()) {
        return Err(Refusal::new(
            "input.date_invalid",
            format!("{} {DATE_RULE}", field("date")),
        ));
    }
    Ok(Donation {
        abn,
        name,
        date,
        amount: read_amount(value, &field("amount"))?,
    })
}

/// The donation as an object with only the fields `dgr-check` reads.
fn donation_object(at: usize, value: &Value) -> Result<&Map<String, Value>, Refusal> {
    let Some(object) = value.as_object() else {
        return Err(Refusal::new(
            "input.donations_invalid",
            format!("donations[{at}] must be an object"),
        ));
    };
    if let Some(key) = object.keys().find(|k| !FIELDS.contains(&k.as_str())) {
        return Err(Refusal::new(
            "input.field_unknown",
            format!("donations[{at}].{key} is not an input of {COMMAND}"),
        ));
    }
    Ok(object)
}

fn checked_abn(given: &str, field: &str) -> Result<String, Refusal> {
    abn_digits(given).ok_or_else(|| {
        Refusal::new(
            "input.abn_invalid",
            format!(
                "{field} {given:?} is not 11 digits, optionally spaced, that pass the ABN check digit test ABN Lookup publishes"
            ),
        )
    })
}

/// 1 to 200 characters, with an ASCII letter or digit in the name itself:
/// `name_words` reads `&` as AND, so it cannot be the test.
fn valid_name(name: &str) -> bool {
    name.chars().count() <= 200 && name.chars().any(|c| c.is_ascii_alphanumeric())
}

fn read_amount(value: &Value, field: &str) -> Result<Option<Decimal>, Refusal> {
    let prefix = |mut refusal: Refusal| {
        refusal.message = format!("{field}: {}", refusal.message);
        refusal
    };
    match optional_decimal(value, "amount").map_err(prefix)? {
        None => Ok(None),
        Some(arg) => engine::money(&arg)
            .map(Some)
            .map_err(|e| prefix(Refusal::new(e.code(), e.to_string()))),
    }
}
