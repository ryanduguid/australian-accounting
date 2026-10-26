//! GST coding and BAS reconciliation. Review aids, not determinations.
pub mod api;
pub mod bas;
pub mod cli;
pub mod coding;
pub mod decimal;
mod pytext;
mod tables;

use chrono::NaiveDate;
use decimal::Decimal;
use serde_json::{Map, Value};
use std::sync::OnceLock;

pub type Result<T> = std::result::Result<T, String>;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
fn date(value: &Value) -> Option<NaiveDate> {
    let text = value.as_str()?;
    let date = NaiveDate::parse_from_str(text, "%Y-%m-%d").ok()?;
    (date.format("%Y-%m-%d").to_string() == text && text.len() == 10 && !text.starts_with("0000"))
        .then_some(date)
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
            .ok()?
            .le(&Decimal::from_int(1_000_000_000_000))
            .ok()?)
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
