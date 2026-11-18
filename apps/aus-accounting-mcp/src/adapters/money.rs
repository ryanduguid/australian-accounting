// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use div7a_loan_review::{decimal::Decimal, raw::Raw};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

static PLAIN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?[0-9]+(?:\.[0-9]+)?$").unwrap());
pub fn strip(text: &str) -> &str {
    text.trim_matches(payday_super_checker::py::is_space)
}

pub fn optional(value: Option<&Value>, field: &str) -> Result<Option<Decimal>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if strip(text).is_empty() => Err(format!(
            "{field}: omit an unknown amount or send null, not a blank string"
        )),
        Some(Value::String(text)) => amount(text, field).map(Some),
        _ => Err(format!("{field} must be a string or null")),
    }
}

pub fn amount(value: &str, field: &str) -> Result<Decimal, String> {
    let text = strip(value);
    if text.is_empty() {
        return Err(format!("{field} is required"));
    }
    let repr = Raw::text(value).py_repr();
    let invalid = || format!("{field}: {repr} is not a decimal amount");
    let amount = Decimal::parse(text).map_err(|_| invalid())?;
    if !amount.is_finite() {
        return Err(format!("{field}: {repr} is not a finite amount"));
    }
    if !PLAIN.is_match(text) {
        return Err(invalid());
    }
    let unsigned = text.trim_start_matches('-');
    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap().trim_start_matches('0');
    let fraction = parts.next().unwrap_or("");
    if whole.len() > 13
        || (whole.len() == 13
            && (whole > "1000000000000"
                || (whole == "1000000000000" && fraction.chars().any(|c| c != '0'))))
    {
        return Err(format!(
            "{field} absolute value must not exceed AUD 1000000000000.00"
        ));
    }
    if fraction.len() > 2 {
        return Err(format!("{field} must have no more than 2 decimal places"));
    }
    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_plain_amount_values_and_failure_precedence() {
        let original: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/native/money-relations.json"
        ))
        .unwrap();
        let accepted = original["test_parse_amount_accepts_plain_notation"]
            .as_array()
            .unwrap();
        assert_eq!(accepted.len(), 11);
        for raw in accepted {
            let raw = raw.as_str().unwrap();
            assert!(
                amount(raw, "amount")
                    .unwrap()
                    .equals(&Decimal::parse(raw.trim()).unwrap())
                    .unwrap(),
                "{raw:?}"
            );
        }
        let refused = original["test_parse_amount_keeps_one_message_per_failure"]
            .as_array()
            .unwrap();
        assert_eq!(refused.len(), 21);
        for case in refused {
            let raw = case[0].as_str().unwrap();
            let message = amount(raw, "amount").unwrap_err();
            assert!(
                message.contains(case[1].as_str().unwrap()),
                "{raw:?}: {message}"
            );
        }
        for absent in [None, Some(&Value::Null)] {
            assert!(optional(absent, "amount").unwrap().is_none());
        }
        for raw in ["", "   "] {
            assert!(
                optional(Some(&Value::String(raw.into())), "amount")
                    .unwrap_err()
                    .contains("omit an unknown amount or send null")
            );
        }
    }
}
