// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::{compare_integer, expected, integer, issue, location, validate};
use div7a_loan_review::raw::Raw;
use serde_json::{Map, Value};

pub(super) fn apply(
    kind: &str,
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<super::Issue>,
) -> Value {
    match kind {
        "model-field" | "default" => validate(&rule["schema"], Some(value), path, config, errors),
        "nullable" if value.is_null() => Value::Null,
        "nullable" => validate(&rule["schema"], Some(value), path, config, errors),
        "model" => model(rule, value, path, errors),
        "model-fields" => model_fields(rule, value, path, config, errors),
        "str" => string(rule, value, path, config, errors),
        "bool" => super::boolean(rule, value, path, config, errors),
        "int" => integer_value(rule, value, path, errors),
        "literal" => literal(rule, value, path, errors),
        "list" => list(rule, value, path, config, errors),
        "dict" => dictionary(rule, value, path, config, errors),
        "tagged-union" => tagged(rule, value, path, config, errors),
        _ => panic!("unsupported embedded input descriptor {kind}"),
    }
}

fn model(rule: &Value, value: &Value, path: &str, errors: &mut Vec<super::Issue>) -> Value {
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

fn model_fields(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<super::Issue>,
) -> Value {
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
    forbid_extra(config, fields, object, path, errors);
    Value::Object(result)
}

fn forbid_extra(
    config: &Value,
    fields: &Map<String, Value>,
    object: &Map<String, Value>,
    path: &str,
    errors: &mut Vec<super::Issue>,
) {
    if config["extra_fields_behavior"] != "forbid" {
        return;
    }
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

fn string(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<super::Issue>,
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
    reject_string(rule, value, path, text, errors);
    Value::String(text.into())
}

fn reject_string(
    rule: &Value,
    value: &Value,
    path: &str,
    text: &str,
    errors: &mut Vec<super::Issue>,
) {
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
}

fn integer_value(rule: &Value, value: &Value, path: &str, errors: &mut Vec<super::Issue>) -> Value {
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
    check_integer_bounds(rule, value, path, &number, errors);
    value.clone()
}

fn check_integer_bounds(
    rule: &Value,
    value: &Value,
    path: &str,
    number: &str,
    errors: &mut Vec<super::Issue>,
) {
    for (key, code, label) in [
        ("ge", "greater_than_equal", "greater than or equal to"),
        ("gt", "greater_than", "greater than"),
        ("le", "less_than_equal", "less than or equal to"),
        ("lt", "less_than", "less than"),
    ] {
        if let Some(bound) = integer(&rule[key]) {
            let order = compare_integer(number, &bound);
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
}

fn as_number(value: &Value) -> Option<f64> {
    if value.is_boolean() {
        Some(if value == true { 1.0 } else { 0.0 })
    } else {
        value.as_f64()
    }
}

fn literal(rule: &Value, value: &Value, path: &str, errors: &mut Vec<super::Issue>) -> Value {
    let values = rule["expected"].as_array().unwrap();
    if let Some(found) = values.iter().find(|candidate| {
        *candidate == value
            || (as_number(candidate).is_some() && as_number(candidate) == as_number(value))
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

fn list(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<super::Issue>,
) -> Value {
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
    if list_too_long(rule, value, path, items, errors) {
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
    collection_size("List", rule, value, path, items.len(), errors);
    result
}

fn list_too_long(
    rule: &Value,
    value: &Value,
    path: &str,
    items: &[Value],
    errors: &mut Vec<super::Issue>,
) -> bool {
    let Some(bound) = rule["max_length"].as_u64() else {
        return false;
    };
    if items.len() as u64 <= bound {
        return false;
    }
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
    true
}

fn collection_size(
    name: &str,
    rule: &Value,
    value: &Value,
    path: &str,
    length: usize,
    errors: &mut Vec<super::Issue>,
) {
    for (key, code, label) in [
        ("min_length", "too_short", "at least"),
        ("max_length", "too_long", "at most"),
    ] {
        if let Some(bound) = rule[key].as_u64()
            && ((key == "min_length" && (length as u64) < bound)
                || (key == "max_length" && (length as u64) > bound))
        {
            issue(
                errors,
                path,
                code,
                format!(
                    "{name} should have {label} {bound} item{} after validation, not {length}",
                    if bound == 1 { "" } else { "s" }
                ),
                value,
            );
        }
    }
}

fn dictionary(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<super::Issue>,
) -> Value {
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
    let result = Value::Object(dict_entries(rule, object, path, config, errors));
    if errors.len() == before {
        collection_size("Dictionary", rule, value, path, object.len(), errors);
    }
    result
}

fn dict_entries(
    rule: &Value,
    object: &Map<String, Value>,
    path: &str,
    config: &Value,
    errors: &mut Vec<super::Issue>,
) -> Map<String, Value> {
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
        .collect()
}

fn tagged(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<super::Issue>,
) -> Value {
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
        None => missing_tag(key, value, path, errors),
        Some(tag) => match tag.as_str().and_then(|name| choices.get(name)) {
            Some(choice) => validate(
                choice,
                Some(value),
                &location(path, tag.as_str().unwrap()),
                config,
                errors,
            ),
            None => invalid_tag(key, tag, choices, value, path, errors),
        },
    }
}

fn missing_tag(key: &str, value: &Value, path: &str, errors: &mut Vec<super::Issue>) -> Value {
    issue(
        errors,
        path,
        "union_tag_not_found",
        format!("Unable to extract tag using discriminator '{key}'"),
        value,
    );
    value.clone()
}

fn invalid_tag(
    key: &str,
    tag: &Value,
    choices: &Map<String, Value>,
    value: &Value,
    path: &str,
    errors: &mut Vec<super::Issue>,
) -> Value {
    let names = choices
        .keys()
        .map(|name| format!("'{name}'"))
        .collect::<Vec<_>>()
        .join(", ");
    issue(
        errors,
        path,
        "union_tag_invalid",
        format!(
            "Input tag '{}' found using '{key}' does not match any of the expected tags: {names}",
            Raw::from_value(tag).py_str()
        ),
        value,
    );
    value.clone()
}
