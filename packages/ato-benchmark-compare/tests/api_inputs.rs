// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The JSON command API refuses a request of the wrong shape with a message
//! naming the field, and carries routing notes into a comparison.

use ato_benchmark_compare::api::evaluate;
use serde_json::{Value, json};

fn refusal(command: &str, input: Value) -> String {
    evaluate(command, &input).expect_err("the request is refused")
}

fn bakery(extra: Value) -> Value {
    let mut input = json!({
        "dataset": {"year": "2023-24"},
        "industry": "Bakeries and hot bread shops",
        "totals": {"turnover": "850000", "rent": "40000"},
    });
    input
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    input
}

#[test]
fn a_request_that_is_not_an_object_is_refused() {
    assert_eq!(refusal("guard", json!([1])), "input must be a JSON object");
}

#[test]
fn an_unknown_command_is_refused() {
    assert_eq!(refusal("explode", json!({})), "unknown command explode");
}

#[test]
fn a_field_of_the_wrong_type_is_named() {
    assert_eq!(
        refusal("suggest", json!({"account": "Rent", "section": 3})),
        "section must be a string or null"
    );
    assert_eq!(
        refusal("suggest-mapping", json!({"rows": {}})),
        "rows must be a list"
    );
    assert_eq!(
        refusal("compute", json!({"totals": []})),
        "totals must be an object"
    );
    assert_eq!(
        refusal(
            "compute",
            json!({"totals": {}, "totals_supplied": "turnover"})
        ),
        "totals_supplied must be a list"
    );
    assert_eq!(
        refusal(
            "write-mapping",
            json!({"path": "m.csv", "mapping_rows": {}})
        ),
        "mapping_rows must be a list"
    );
    assert_eq!(
        refusal("route", json!({"rows": [], "mapping": {}})),
        "mapping must be a list"
    );
}

#[test]
fn an_unknown_comparison_output_is_refused() {
    assert_eq!(
        refusal("compare-figures", bakery(json!({"output": "pdf"}))),
        "unknown output pdf"
    );
}

#[test]
fn routing_notes_follow_the_comparison_notes() {
    let plain = evaluate("compare-figures", &bakery(json!({}))).unwrap();
    let routed = evaluate(
        "compare-figures",
        &bakery(json!({"routing_notes": ["Rent row read as rent", "Wages left unmapped"]})),
    )
    .unwrap();
    let mut expected = plain["notes"].as_array().unwrap().clone();
    expected.push(json!(["routing", "Rent row read as rent", []]));
    expected.push(json!(["routing", "Wages left unmapped", []]));
    assert_eq!(routed["notes"], Value::Array(expected));
    assert_eq!(routed["verdicts"], plain["verdicts"]);
}
