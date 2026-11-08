// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The original Pydantic core descriptors preserve effective input behaviour.
//! They contain no executable Python and no statutory calculation.
use div7a_loan_review::raw::Raw;
use serde_json::{Map, Value};
use std::sync::LazyLock;

static RULES: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/effective-inputs.v1.json")).unwrap()
});

struct Issue {
    path: String,
    code: &'static str,
    message: String,
    input: Value,
}
fn issue(
    errors: &mut Vec<Issue>,
    path: &str,
    code: &'static str,
    message: impl Into<String>,
    input: &Value,
) {
    errors.push(Issue {
        path: path.into(),
        code,
        message: message.into(),
        input: input.clone(),
    });
}
fn location(path: &str, key: &str) -> String {
    let key = if key.contains('.') {
        format!("`{key}`")
    } else {
        key.to_owned()
    };
    if path.is_empty() {
        key
    } else {
        format!("{path}.{key}")
    }
}
fn integer(value: &Value) -> Option<String> {
    match Raw::from_value(value) {
        Raw::Int(text) => Some(text),
        _ => None,
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
fn python(value: &Value) -> String {
    Raw::from_value(value).py_repr()
}
fn displayed(value: &Value) -> String {
    let repr = python(value);
    let chars: Vec<_> = repr.chars().collect();
    if chars.len() > 50 {
        format!(
            "{}...{}",
            chars[..25].iter().collect::<String>(),
            chars[chars.len() - 24..].iter().collect::<String>()
        )
    } else {
        repr
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

fn validate(
    rule: &Value,
    value: Option<&Value>,
    path: &str,
    config: &Value,
    errors: &mut Vec<Issue>,
) -> Value {
    let kind = rule["type"].as_str().unwrap();
    if kind == "model-field" {
        return validate(&rule["schema"], value, path, config, errors);
    }
    if kind == "default" && value.is_none() {
        return rule["default"].clone();
    }
    let Some(value) = value else {
        // Model fields supply the full original object for missing diagnostics.
        unreachable!("missing fields are handled at their model boundary")
    };
    match kind {
        "model-field" | "default" => validate(&rule["schema"], Some(value), path, config, errors),
        "nullable" if value.is_null() => Value::Null,
        "nullable" => validate(&rule["schema"], Some(value), path, config, errors),
        "model" => {
            if !value.is_object() {
                issue(
                    errors,
                    path,
                    "model_type",
                    format!(
                        "Input should be a valid dictionary or instance of {}",
                        rule["name"].as_str().unwrap()
                    ),
                    value,
                );
                return value.clone();
            }
            validate(&rule["schema"], Some(value), path, &rule["config"], errors)
        }
        "model-fields" => {
            let object = value.as_object().unwrap();
            let fields = rule["fields"].as_object().unwrap();
            let mut result = Map::new();
            for (name, field) in fields {
                let at = location(path, name);
                if !object.contains_key(name) && field["schema"]["type"] != "default" {
                    issue(errors, &at, "missing", "Field required", value);
                } else {
                    result.insert(
                        name.clone(),
                        validate(field, object.get(name), &at, config, errors),
                    );
                }
            }
            if config["extra_fields_behavior"] == "forbid" {
                for (name, item) in object {
                    if !fields.contains_key(name) {
                        issue(
                            errors,
                            &location(path, name),
                            "extra_forbidden",
                            "Extra inputs are not permitted",
                            item,
                        );
                    }
                }
            }
            Value::Object(result)
        }
        "str" => {
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
            let size = text.chars().count() as u64;
            if let Some(min) = rule["min_length"].as_u64()
                && size < min
            {
                issue(
                    errors,
                    path,
                    "string_too_short",
                    format!(
                        "String should have at least {min} character{}",
                        if min == 1 { "" } else { "s" }
                    ),
                    value,
                );
            } else if let Some(max) = rule["max_length"].as_u64()
                && size > max
            {
                issue(
                    errors,
                    path,
                    "string_too_long",
                    format!(
                        "String should have at most {max} character{}",
                        if max == 1 { "" } else { "s" }
                    ),
                    value,
                );
            } else if let Some(pattern) = rule["pattern"].as_str()
                && !regex::Regex::new(pattern)
                    .expect("embedded pattern")
                    .is_match(text)
            {
                issue(
                    errors,
                    path,
                    "string_pattern_mismatch",
                    format!("String should match pattern '{pattern}'"),
                    value,
                );
            }
            Value::String(text.into())
        }
        "bool" => {
            if !value.is_boolean() {
                issue(
                    errors,
                    path,
                    "bool_type",
                    "Input should be a valid boolean",
                    value,
                );
            }
            value.clone()
        }
        "int" => {
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
        "literal" => {
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
                    || (boolean_integer(v).is_some()
                        && boolean_integer(v) == boolean_integer(value))
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
        "list" => {
            let Some(items) = value.as_array() else {
                issue(
                    errors,
                    path,
                    "list_type",
                    "Input should be a valid list",
                    value,
                );
                return value.clone();
            };
            if let Some(bound) = rule["max_length"].as_u64()
                && items.len() as u64 > bound
            {
                issue(
                    errors,
                    path,
                    "too_long",
                    format!(
                        "List should have at most {bound} item{} after validation, not {}",
                        if bound == 1 { "" } else { "s" },
                        items.len()
                    ),
                    value,
                );
                return value.clone();
            }
            let result = Value::Array(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        validate(
                            &rule["items_schema"],
                            Some(item),
                            &location(path, &i.to_string()),
                            config,
                            errors,
                        )
                    })
                    .collect(),
            );
            for (key, code, label) in [
                ("min_length", "too_short", "at least"),
                ("max_length", "too_long", "at most"),
            ] {
                if let Some(bound) = rule[key].as_u64()
                    && ((key == "min_length" && (items.len() as u64) < bound)
                        || (key == "max_length" && (items.len() as u64) > bound))
                {
                    issue(
                        errors,
                        path,
                        code,
                        format!(
                            "List should have {label} {bound} item{} after validation, not {}",
                            if bound == 1 { "" } else { "s" },
                            items.len()
                        ),
                        value,
                    );
                }
            }
            result
        }
        "dict" => {
            let Some(object) = value.as_object() else {
                issue(
                    errors,
                    path,
                    "dict_type",
                    "Input should be a valid dictionary",
                    value,
                );
                return value.clone();
            };
            let before = errors.len();
            let result = Value::Object(
                object
                    .iter()
                    .map(|(key, item)| {
                        let at = location(path, key);
                        if !rule["keys_schema"].is_null() {
                            validate(
                                &rule["keys_schema"],
                                Some(&Value::String(key.clone())),
                                &location(&at, "[key]"),
                                config,
                                errors,
                            );
                        }
                        let item = if rule["values_schema"].is_null() {
                            item.clone()
                        } else {
                            validate(&rule["values_schema"], Some(item), &at, config, errors)
                        };
                        (key.clone(), item)
                    })
                    .collect(),
            );
            if errors.len() == before {
                for (key, code, label) in [
                    ("min_length", "too_short", "at least"),
                    ("max_length", "too_long", "at most"),
                ] {
                    if let Some(bound) = rule[key].as_u64()
                        && ((key == "min_length" && (object.len() as u64) < bound)
                            || (key == "max_length" && (object.len() as u64) > bound))
                    {
                        issue(
                            errors,
                            path,
                            code,
                            format!(
                                "Dictionary should have {label} {bound} item{} after validation, not {}",
                                if bound == 1 { "" } else { "s" },
                                object.len()
                            ),
                            value,
                        );
                    }
                }
            }
            result
        }
        "tagged-union" => {
            if !value.is_object() {
                issue(
                    errors,
                    path,
                    "model_attributes_type",
                    "Input should be a valid dictionary or object to extract fields from",
                    value,
                );
                return value.clone();
            }
            let key = rule["discriminator"].as_str().unwrap();
            let choices = rule["choices"].as_object().unwrap();
            match value.get(key) {
                None => {
                    issue(
                        errors,
                        path,
                        "union_tag_not_found",
                        format!("Unable to extract tag using discriminator '{key}'"),
                        value,
                    );
                    value.clone()
                }
                Some(tag) => match tag.as_str().and_then(|s| choices.get(s)) {
                    Some(choice) => validate(
                        choice,
                        Some(value),
                        &location(path, tag.as_str().unwrap()),
                        config,
                        errors,
                    ),
                    None => {
                        issue(
                            errors,
                            path,
                            "union_tag_invalid",
                            format!(
                                "Input tag '{}' found using '{key}' does not match any of the expected tags: {}",
                                Raw::from_value(tag).py_str(),
                                choices
                                    .keys()
                                    .map(|s| format!("'{s}'"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            value,
                        );
                        value.clone()
                    }
                },
            }
        }
        _ => panic!("unsupported embedded input descriptor {kind}"),
    }
}

pub fn arguments(name: &str, input: &Value) -> Result<Value, String> {
    let rule = &RULES[name];
    let mut errors = vec![];
    let result = validate(rule, Some(input), "", &Value::Null, &mut errors);
    if errors.is_empty() {
        return Ok(result);
    }
    let mut message = format!(
        "{} validation error{} for {}",
        errors.len(),
        if errors.len() == 1 { "" } else { "s" },
        rule["name"].as_str().unwrap()
    );
    for error in errors {
        message += &format!(
            "\n{}\n  {} [type={}, input_value={}, input_type={}]\n    For further information visit https://errors.pydantic.dev/2.13/v/{}",
            error.path,
            error.message,
            error.code,
            displayed(&error.input),
            Raw::from_value(&error.input).type_name(),
            error.code
        );
    }
    Err(message)
}
