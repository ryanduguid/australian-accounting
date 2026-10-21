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
