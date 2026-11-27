// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The original Pydantic core descriptors preserve effective input behaviour.
//! They contain no executable Python and no statutory calculation.
mod integer;
mod kinds;

use integer::compare_integer;

use div7a_loan_review::raw::Raw;
use serde_json::Value;
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
    kinds::apply(kind, rule, value, path, config, errors)
}

fn boolean(
    rule: &Value,
    value: &Value,
    path: &str,
    config: &Value,
    errors: &mut Vec<Issue>,
) -> Value {
    // A field's own strict setting overrides its model's.
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

    fn check(
        rule: serde_json::Value,
        input: serde_json::Value,
    ) -> (serde_json::Value, Vec<String>) {
        let mut errors = vec![];
        let output = super::validate(&rule, Some(&input), "", &json!(null), &mut errors);
        let errors = errors
            .into_iter()
            .map(|e| format!("{}|{}|{}|{}", e.path, e.code, e.message, e.input))
            .collect();
        (output, errors)
    }

    /// Each expected issue is pydantic 2.13.5's for the same core schema
    /// (`TypeAdapter(...).validate_python`, recorded 27 November 2026).
    #[test]
    fn collection_rules_the_tools_do_not_use_yet_still_match_pydantic() {
        for (rule, input, expected) in [
            // TypeAdapter(list[int]).validate_python('x')
            (
                json!({"type": "list", "items_schema": {"type": "int"}}),
                json!("x"),
                vec![r#"|list_type|Input should be a valid list|"x""#],
            ),
            // TypeAdapter(list[int]).validate_python([1, 2]): no length bound.
            (
                json!({"type": "list", "items_schema": {"type": "int"}}),
                json!([1, 2]),
                vec![],
            ),
            // TypeAdapter(dict[str, int]).validate_python([])
            (
                json!({"type": "dict", "values_schema": {"type": "int"}}),
                json!([]),
                vec!["|dict_type|Input should be a valid dictionary|[]"],
            ),
            // TypeAdapter(dict[str, Any]).validate_python({'a': [1]}): values kept as given.
            (json!({"type": "dict"}), json!({"a": [1]}), vec![]),
            // A discriminated union of models given {}.
            (
                json!({"type": "tagged-union", "discriminator": "kind", "choices": {"a": {"type": "int"}}}),
                json!({}),
                vec!["|union_tag_not_found|Unable to extract tag using discriminator 'kind'|{}"],
            ),
        ] {
            expect(rule, input, expected);
        }
    }

    /// Scalar bounds and lax booleans, as pydantic 2.13.5 reports them.
    #[test]
    fn scalar_rules_the_tools_do_not_use_yet_still_match_pydantic() {
        for (rule, input, expected) in [
            // Annotated[int, Field(lt=5)] given 5.
            (
                json!({"type": "int", "lt": 5}),
                json!(5),
                vec!["|less_than|Input should be less than 5|5"],
            ),
            // Annotated[int, Field(ge=-1)]: 1 passes, -2 is refused.
            (json!({"type": "int", "ge": -1}), json!(1), vec![]),
            (
                json!({"type": "int", "ge": -1}),
                json!(-2),
                vec!["|greater_than_equal|Input should be greater than or equal to -1|-2"],
            ),
            // A lax bool given 'maybe' and 2.
            (
                json!({"type": "bool"}),
                json!("maybe"),
                vec![
                    r#"|bool_parsing|Input should be a valid boolean, unable to interpret input|"maybe""#,
                ],
            ),
            (
                json!({"type": "bool"}),
                json!(2),
                vec!["|bool_parsing|Input should be a valid boolean, unable to interpret input|2"],
            ),
        ] {
            expect(rule, input, expected);
        }
    }

    fn expect(rule: serde_json::Value, input: serde_json::Value, expected: Vec<&str>) {
        let (output, errors) = check(rule.clone(), input.clone());
        assert_eq!(errors, expected, "{rule} {input}");
        assert_eq!(output, input, "{rule}: the input is returned as given");
    }

    /// The descriptors are embedded, so a kind the validator does not know is a
    /// build defect: it stops instead of passing the input through unchecked.
    #[test]
    #[should_panic(expected = "unsupported embedded input descriptor set")]
    fn an_unknown_descriptor_kind_stops() {
        check(json!({"type": "set"}), json!([]));
    }

    /// Only model fields may be absent; they report it with the whole object.
    #[test]
    #[should_panic(expected = "missing fields are handled at their model boundary")]
    fn an_absent_value_outside_a_model_field_stops() {
        super::validate(&json!({"type": "int"}), None, "", &json!(null), &mut vec![]);
    }
}
