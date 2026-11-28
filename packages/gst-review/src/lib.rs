// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! GST coding and BAS reconciliation. Review aids, not determinations.
pub mod api;
pub mod bas;
pub mod cli;
pub mod coding;
pub mod decimal;
mod pytext;
mod tables;

use decimal::Decimal;
use jiff::civil::Date;
use serde_json::{Map, Value};
use std::sync::OnceLock;

pub type Result<T> = std::result::Result<T, String>;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn arithmetic<T>(result: decimal::Calc<T>) -> Result<T> {
    result.map_err(|e| e.name().to_string())
}

/// The rule sources and their check date, for a freshness report.
pub fn provenance() -> Value {
    let (coding, bas) = (&constants()["coding"], &constants()["bas"]);
    serde_json::json!({"sources":[coding["GST_ACT"],bas["ATO_BAS_METHOD"],bas["ATO_BAS_INSTRUCTIONS"]],
        "source_checked":coding["SOURCE_CHECKED"]})
}

fn constants() -> &'static Value {
    static CONSTANTS: OnceLock<Value> = OnceLock::new();
    CONSTANTS.get_or_init(|| serde_json::from_str(include_str!("constants.json")).unwrap())
}
fn object(value: &Value) -> Result<&Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| "input must be an object".into())
}
fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("{key} must be an array"))
}
fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).unwrap_or(&Value::Null)
}
fn date(value: &Value) -> Option<Date> {
    let text = value.as_str()?;
    let date = Date::strptime("%Y-%m-%d", text).ok()?;
    (date.strftime("%Y-%m-%d").to_string() == text && date.year() > 0).then_some(date)
}
fn optional_date(value: &Value) -> bool {
    value.is_null() || date(value).is_some()
}
fn money(value: &Value) -> Option<Decimal> {
    let text = value.as_str()?;
    if text.len() > 8192 {
        return None;
    }
    let d = Decimal::parse(text).ok()?;
    (d.is_finite()
        && d.exponent() >= -2
        && d.abs()
            .is_ok_and(|abs| abs.le(&Decimal::from_int(1_000_000_000_000)) == Ok(true)))
    .then_some(d)
}
fn repr(value: &Value) -> String {
    match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::String(s) => pytext::repr(s),
        Value::Number(n) => n.to_string(),
        Value::Array(a) => format!("[{}]", a.iter().map(repr).collect::<Vec<_>>().join(", ")),
        Value::Object(o) => format!(
            "{{{}}}",
            o.iter()
                .map(|(k, v)| format!("{}: {}", pytext::repr(k), repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
fn truth(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(v) => *v,
        Value::String(v) => !v.is_empty(),
        Value::Array(v) => !v.is_empty(),
        Value::Object(v) => !v.is_empty(),
        Value::Number(v) => v.as_f64().is_some_and(|n| n != 0.0),
    }
}
fn categories(codes: &Map<String, Value>, allowed: &[&str], label: &str) -> Result<()> {
    if codes.values().any(|v| !v.is_string()) {
        return Err("code table categories must be strings".into());
    }
    let mut unknown: Vec<_> = codes
        .values()
        .filter_map(Value::as_str)
        .filter(|s| !allowed.contains(s))
        .collect();
    unknown.sort();
    unknown.dedup();
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Unknown {label} in the code table: [{}].",
            unknown
                .into_iter()
                .map(pytext::repr)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}
fn codes<'a>(
    input: &'a Value,
    key: &str,
    family: &str,
    default: &str,
) -> Result<&'a Map<String, Value>> {
    match input.get(key) {
        None | Some(Value::Null) => Ok(constants()[family][default].as_object().unwrap()),
        Some(value) => value
            .as_object()
            .ok_or_else(|| format!("{key} must be an object or null")),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// The round trip and the positive year alone keep a date to 10
    /// characters: jiff writes `%Y` with at least 4 digits.
    #[test]
    fn a_date_is_ten_characters_without_a_length_check() {
        for text in ["2024-01-05", "0999-12-31", "9999-12-31"] {
            assert!(super::date(&json!(text)).is_some(), "{text}");
        }
        for text in [
            "999-12-31",
            "2024-1-05",
            "02024-01-05",
            "0000-01-01",
            "2024-01-05 ",
        ] {
            assert!(super::date(&json!(text)).is_none(), "{text}");
        }
    }

    #[test]
    fn arithmetic_helper_maps_all_decimal_signals() {
        use super::decimal::Signal;
        assert_eq!(super::arithmetic(Ok(42)), Ok(42));
        assert_eq!(
            super::arithmetic::<()>(Err(Signal::InvalidOperation)),
            Err("InvalidOperation".into())
        );
        assert_eq!(
            super::arithmetic::<()>(Err(Signal::DivisionByZero)),
            Err("DivisionByZero".into())
        );
        assert_eq!(
            super::arithmetic::<()>(Err(Signal::Overflow)),
            Err("Overflow".into())
        );
    }

    #[test]
    fn repr_and_truth_match_python_for_all_json_types() {
        assert_eq!(super::repr(&json!(true)), "True");
        assert_eq!(super::repr(&json!(false)), "False");
        assert_eq!(super::repr(&json!([1, null])), "[1, None]");
        assert_eq!(super::repr(&json!({"k": true})), "{'k': True}");
        assert!(super::truth(&json!(true)));
        assert!(!super::truth(&json!(false)));
        assert!(super::truth(&json!([1])));
        assert!(!super::truth(&json!([])));
        assert!(super::truth(&json!({"a": 1})));
        assert!(!super::truth(&json!({})));
        assert!(super::truth(&json!(2.5)));
        assert!(!super::truth(&json!(0)));
    }
}
