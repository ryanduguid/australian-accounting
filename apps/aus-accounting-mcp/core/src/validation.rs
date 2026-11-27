// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The original Pydantic core descriptors preserve effective input behaviour.
//! They contain no executable Python and no statutory calculation.
mod scalars;

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

/// Pydantic's message for a list or dictionary of `size` items outside a bound.
fn size_message(noun: &str, label: &str, bound: u64, size: usize) -> String {
    format!(
        "{noun} should have {label} {bound} item{} after validation, not {size}",
        if bound == 1 { "" } else { "s" }
    )
}

/// The too_short and too_long issues of a list or dictionary of `size` items.
fn size_issues(
    rule: &Value,
    noun: &str,
    size: usize,
    path: &str,
    value: &Value,
    errors: &mut Vec<Issue>,
) {
    for (key, code, label) in [
        ("min_length", "too_short", "at least"),
        ("max_length", "too_long", "at most"),
    ] {
        if let Some(bound) = rule[key].as_u64()
            && ((key == "min_length" && (size as u64) < bound)
                || (key == "max_length" && (size as u64) > bound))
        {
            issue(
                errors,
                path,
                code,
                size_message(noun, label, bound, size),
                value,
            );
        }
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
    let value = value.expect("missing fields are handled at their model boundary");
    match kind {
        "model-field" | "default" => validate(&rule["schema"], Some(value), path, config, errors),
        "nullable" if value.is_null() => Value::Null,
        "nullable" => validate(&rule["schema"], Some(value), path, config, errors),
        "model" => model(rule, value, path, errors),
        "model-fields" => model_fields(rule, value, path, config, errors),
        "str" => scalars::string(rule, value, path, config, errors),
        // A field's own strict setting overrides its model's.
        "bool" => scalars::boolean(rule, value, path, config, errors),
        "int" => scalars::int(rule, value, path, errors),
        "literal" => scalars::literal(rule, value, path, errors),
        "list" => list(rule, value, path, config, errors),
        "dict" => dict(rule, value, path, config, errors),
        "tagged-union" => tagged_union(rule, value, path, config, errors),
        _ => panic!("unsupported embedded input descriptor {kind}"),
    }
}

fn model(rule: &Value, value: &Value, path: &str, errors: &mut Vec<Issue>) -> Value {
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
    errors: &mut Vec<Issue>,
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

fn list(rule: &Value, value: &Value, path: &str, config: &Value, errors: &mut Vec<Issue>) -> Value {
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
        let message = size_message("List", "at most", bound, items.len());
        issue(errors, path, "too_long", message, value);
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
    size_issues(rule, "List", items.len(), path, value, errors);
    result
}

fn dict(rule: &Value, value: &Value, path: &str, config: &Value, errors: &mut Vec<Issue>) -> Value {
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
                let item = dict_item(rule, key, item, path, config, errors);
                (key.clone(), item)
            })
            .collect(),
    );
    if errors.len() == before {
        size_issues(rule, "Dictionary", object.len(), path, value, errors);
    }
    result
}

/// One dictionary entry: its key against `keys_schema`, then its value.
fn dict_item(
    rule: &Value,
    key: &str,
    item: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<Issue>,
) -> Value {
    let at = location(path, key);
    if !rule["keys_schema"].is_null() {
        validate(
            &rule["keys_schema"],
            Some(&Value::String(key.to_owned())),
            &location(&at, "[key]"),
            config,
            errors,
        );
    }
    if rule["values_schema"].is_null() {
        item.clone()
    } else {
        validate(&rule["values_schema"], Some(item), &at, config, errors)
    }
}

fn tagged_union(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<Issue>,
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
    let Some(tag) = value.get(key) else {
        let message = format!("Unable to extract tag using discriminator '{key}'");
        issue(errors, path, "union_tag_not_found", message, value);
        return value.clone();
    };
    if let Some(choice) = tag.as_str().and_then(|s| choices.get(s)) {
        let at = location(path, tag.as_str().unwrap());
        return validate(choice, Some(value), &at, config, errors);
    }
    let tags = choices
        .keys()
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let message = format!(
        "Input tag '{}' found using '{key}' does not match any of the expected tags: {tags}",
        Raw::from_value(tag).py_str()
    );
    issue(errors, path, "union_tag_invalid", message, value);
    value.clone()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(rule: &Value, value: Option<&Value>) -> (Value, Vec<Issue>) {
        let mut errors = vec![];
        let out = validate(rule, value, "p", &Value::Null, &mut errors);
        (out, errors)
    }

    #[test]
    fn list_descriptor_validates_type_bounds_and_items() {
        let rule = json!({"type": "list", "min_length": 2, "items_schema": {"type": "int"}});
        let (_, bad_type) = run(&rule, Some(&json!("bad")));
        let (_, bad_item) = run(&rule, Some(&json!(["bad"])));
        assert_eq!(bad_type[0].code, "list_type");
        assert_eq!(bad_item[0].path, "p.0");
        assert_eq!(bad_item[0].code, "int_type");
        assert_eq!(bad_item[1].code, "too_short");

        let max_one = json!({"type": "list", "max_length": 1, "items_schema": {"type": "int"}});
        let (_, too_long) = run(&max_one, Some(&json!([1, "x"])));
        assert_eq!(too_long.len(), 1);
        assert_eq!(too_long[0].code, "too_long");
        assert!(too_long[0].message.contains("at most 1 item "));

        let ok = json!({"type": "list", "min_length": 1, "max_length": 2, "items_schema": {"type": "int"}});
        let (out, errors) = run(&ok, Some(&json!([1, 2])));
        assert_eq!(out, json!([1, 2]));
        assert!(errors.is_empty());
    }

    #[test]
    fn dict_descriptor_validates_type_keys_values_and_size() {
        let dict_rule = json!({
            "type": "dict", "min_length": 2, "max_length": 2,
            "keys_schema": {"type": "str", "min_length": 2},
            "values_schema": {"type": "int"}
        });
        let (_, bad_type) = run(&dict_rule, Some(&json!([])));
        let (_, bad_item) = run(&dict_rule, Some(&json!({"a": "x"})));
        assert_eq!(bad_type[0].code, "dict_type");
        assert_eq!(bad_item.len(), 2);
        assert_eq!(bad_item[0].path, "p.a.[key]");
        assert_eq!(bad_item[1].path, "p.a");

        let pass = json!({"type": "dict", "min_length": 2, "max_length": 1, "keys_schema": null, "values_schema": null});
        let (out, short) = run(&pass, Some(&json!({"ab": 1})));
        assert_eq!(out, json!({"ab": 1}));
        assert_eq!(short[0].code, "too_short");
        let (_, long) = run(&pass, Some(&json!({"a": 1, "b": 2})));
        assert_eq!(long[0].code, "too_long");
        assert!(long[0].message.contains("at most 1 item "));
    }

    #[test]
    fn model_and_default_descriptors_cover_non_object_and_dotted_keys() {
        let model_rule = json!({
            "type": "model", "name": "Demo", "config": {},
            "schema": {"type": "model-fields", "fields": {"a.b": {"type": "model-field", "schema": {"type": "str"}}}}
        });
        assert_eq!(run(&model_rule, Some(&json!(1))).1[0].code, "model_type");
        assert_eq!(run(&model_rule, Some(&json!({}))).1[0].path, "p.`a.b`");
        let def = json!({"type": "default", "default": "x", "schema": {"type": "str"}});
        assert_eq!(run(&def, None).0, json!("x"));
    }

    #[test]
    #[should_panic(expected = "unsupported embedded input descriptor")]
    fn unsupported_descriptor_type_panics() {
        run(&json!({"type": "tuple"}), Some(&json!(1)));
    }
}
