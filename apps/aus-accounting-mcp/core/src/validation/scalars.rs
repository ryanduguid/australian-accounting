// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The string, boolean, integer and literal descriptors.
use super::{Issue, issue, python};
use div7a_loan_review::raw::Raw;
use serde_json::Value;

fn integer(value: &Value) -> Option<String> {
    match Raw::from_value(value) {
        Raw::Int(text) => Some(text),
        _ => None,
    }
}
fn strict(rule: &Value, config: &Value) -> bool {
    rule["strict"].as_bool().unwrap_or(config["strict"] == true)
}
/// Pydantic's lax boolean: 0 and 1, and its true and false words in any ASCII case.
/// The error is pydantic's type for an input it could not read as a boolean.
fn lax_bool(value: &Value) -> Result<bool, &'static str> {
    match value {
        Value::Bool(held) => Ok(*held),
        Value::String(text) => match text.to_ascii_lowercase().as_str() {
            "0" | "off" | "f" | "false" | "n" | "no" => Ok(false),
            "1" | "on" | "t" | "true" | "y" | "yes" => Ok(true),
            _ => Err("bool_parsing"),
        },
        Value::Number(number) => {
            let whole = number.as_i64().or_else(|| {
                number
                    .as_f64()
                    .filter(|float| {
                        float.fract() == 0.0 && *float > i64::MIN as f64 && *float < i64::MAX as f64
                    })
                    .map(|float| float as i64)
            });
            match whole {
                Some(0) => Ok(false),
                Some(1) => Ok(true),
                Some(_) => Err("bool_parsing"),
                None => Err("bool_type"),
            }
        }
        _ => Err("bool_type"),
    }
}
fn compare_integer(left: &str, right: &str) -> std::cmp::Ordering {
    let magnitude = |text: &str| {
        text.trim_start_matches('-')
            .trim_start_matches('0')
            .to_string()
    };
    let a = magnitude(left);
    let b = magnitude(right);
    let negative_a = left.starts_with('-') && !a.is_empty();
    let negative_b = right.starts_with('-') && !b.is_empty();
    match (negative_a, negative_b) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => {
            let order = a.len().cmp(&b.len()).then_with(|| a.cmp(&b));
            if negative_a { order.reverse() } else { order }
        }
    }
}
fn expected(values: &[Value]) -> String {
    let labels: Vec<_> = values.iter().map(python).collect();
    match labels.len() {
        1 => labels[0].clone(),
        2 => format!("{} or {}", labels[0], labels[1]),
        _ => format!(
            "{} or {}",
            labels[..labels.len() - 1].join(", "),
            labels.last().unwrap()
        ),
    }
}

pub(super) fn string(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<Issue>,
) -> Value {
    let Some(text) = value.as_str() else {
        issue(
            errors,
            path,
            "string_type",
            "Input should be a valid string",
            value,
        );
        return value.clone();
    };
    let text = if rule["strip_whitespace"]
        .as_bool()
        .unwrap_or(config["str_strip_whitespace"].as_bool().unwrap_or(false))
    {
        text.trim()
    } else {
        text
    };
    if let Some((code, message)) = string_problem(rule, text) {
        issue(errors, path, code, message, value);
    }
    Value::String(text.into())
}

/// The first of the length and pattern checks that `text` fails, as pydantic orders them.
fn string_problem(rule: &Value, text: &str) -> Option<(&'static str, String)> {
    let size = text.chars().count() as u64;
    let plural = |bound: u64| if bound == 1 { "" } else { "s" };
    if let Some(min) = rule["min_length"].as_u64()
        && size < min
    {
        let message = format!("String should have at least {min} character{}", plural(min));
        return Some(("string_too_short", message));
    }
    if let Some(max) = rule["max_length"].as_u64()
        && size > max
    {
        let message = format!("String should have at most {max} character{}", plural(max));
        return Some(("string_too_long", message));
    }
    if let Some(pattern) = rule["pattern"].as_str()
        && !regex::Regex::new(pattern)
            .expect("embedded pattern")
            .is_match(text)
    {
        let message = format!("String should match pattern '{pattern}'");
        return Some(("string_pattern_mismatch", message));
    }
    None
}

pub(super) fn boolean(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<Issue>,
) -> Value {
    match lax_bool(value) {
        Ok(held) if value.is_boolean() || !strict(rule, config) => Value::Bool(held),
        Err("bool_parsing") if !strict(rule, config) => {
            issue(
                errors,
                path,
                "bool_parsing",
                "Input should be a valid boolean, unable to interpret input",
                value,
            );
            value.clone()
        }
        _ => {
            issue(
                errors,
                path,
                "bool_type",
                "Input should be a valid boolean",
                value,
            );
            value.clone()
        }
    }
}

pub(super) fn int(rule: &Value, value: &Value, path: &str, errors: &mut Vec<Issue>) -> Value {
    let Some(number) = integer(value) else {
        issue(
            errors,
            path,
            "int_type",
            "Input should be a valid integer",
            value,
        );
        return value.clone();
    };
    for (key, code, label) in [
        ("ge", "greater_than_equal", "greater than or equal to"),
        ("gt", "greater_than", "greater than"),
        ("le", "less_than_equal", "less than or equal to"),
        ("lt", "less_than", "less than"),
    ] {
        if let Some(bound) = integer(&rule[key]) {
            let order = compare_integer(&number, &bound);
            let bad = match key {
                "ge" => order.is_lt(),
                "gt" => order.is_le(),
                "le" => order.is_gt(),
                _ => order.is_ge(),
            };
            if bad {
                issue(
                    errors,
                    path,
                    code,
                    format!("Input should be {label} {}", rule[key]),
                    value,
                );
            }
        }
    }
    value.clone()
}

pub(super) fn literal(rule: &Value, value: &Value, path: &str, errors: &mut Vec<Issue>) -> Value {
    let values = rule["expected"].as_array().unwrap();
    let boolean_integer = |v: &Value| {
        if v.is_boolean() {
            Some(if v == true { 1.0 } else { 0.0 })
        } else {
            v.as_f64()
        }
    };
    if let Some(found) = values.iter().find(|v| {
        *v == value
            || (boolean_integer(v).is_some() && boolean_integer(v) == boolean_integer(value))
    }) {
        found.clone()
    } else {
        issue(
            errors,
            path,
            "literal_error",
            format!("Input should be {}", expected(values)),
            value,
        );
        value.clone()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// Pydantic 2.13.5's lax bool on each input (B-1 receipts, lax_bool.py).
    #[test]
    fn a_lax_boolean_reads_what_pydantic_reads() {
        for (input, expected) in [
            (json!(true), Ok(true)),
            (json!(0), Ok(false)),
            (json!(1), Ok(true)),
            (json!(2), Err("bool_parsing")),
            (json!(-1), Err("bool_parsing")),
            (json!(0.0), Ok(false)),
            (json!(1.0), Ok(true)),
            (json!(2.0), Err("bool_parsing")),
            (json!(1.5), Err("bool_type")),
            (json!("True"), Ok(true)),
            (json!("oN"), Ok(true)),
            (json!("Y"), Ok(true)),
            (json!("OFF"), Ok(false)),
            (json!("0"), Ok(false)),
            (json!(" true"), Err("bool_parsing")),
            (json!(""), Err("bool_parsing")),
            (json!("1.0"), Err("bool_parsing")),
            (json!(null), Err("bool_type")),
            (json!([]), Err("bool_type")),
        ] {
            assert_eq!(super::lax_bool(&input), expected, "{input}");
        }
        let huge: serde_json::Value =
            serde_json::from_str("1000000000000000000000000000000").unwrap();
        assert_eq!(super::lax_bool(&huge), Err("bool_type"));
    }

    /// Pydantic 2.13.5 reads a whole float strictly inside the i64 range as an integer
    /// (B-1 receipts, lax_bool_bound.py).
    #[test]
    fn a_whole_float_inside_the_i64_range_is_an_integer_to_pydantic() {
        for (input, expected) in [
            (9.21e18, "bool_parsing"),
            (9_223_372_036_854_774_784.0, "bool_parsing"),
            (9_223_372_036_854_775_808.0, "bool_type"),
            (-9.21e18, "bool_parsing"),
            (-9_223_372_036_854_774_784.0, "bool_parsing"),
            (-9_223_372_036_854_775_808.0, "bool_type"),
            (9.3e18, "bool_type"),
        ] {
            assert_eq!(super::lax_bool(&json!(input)), Err(expected), "{input}");
        }
    }
}
