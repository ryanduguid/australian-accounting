// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `payroll-tax-group`'s refusals of malformed persons, id lists and
//! controlling-interest facts, and the groups that need no tracing or wages.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A Victorian group of A, B and C through common employees, `changes`
/// replacing or adding facts.
fn group(changes: Value) -> Value {
    let mut input = json!({
        "jurisdiction": "vic", "employer": "A", "employed_all_year": true,
        "year": "2026-27", "scope_confirmed": true,
        "persons": [
            {"id": "A", "taxable_wages": "500000.00"},
            {"id": "B", "taxable_wages": "400000.00"},
            {"id": "C", "taxable_wages": "300000.00"},
        ],
        "common_employees": [["A", "B"], ["A", "C"]],
    });
    for (key, value) in changes.as_object().unwrap() {
        input[key] = value.clone();
    }
    input
}

fn run(input: &Value) -> Value {
    api::evaluate("payroll-tax-group", input).unwrap()
}

fn refused(input: &Value) -> (&'static str, String) {
    let error = api::evaluate_coded("payroll-tax-group", input).unwrap_err();
    (error.code, error.message)
}

/// Each malformed person is refused as an invalid fact, saying what is wrong.
#[test]
fn malformed_persons_are_refused() {
    for (persons, message) in [
        (json!(["A"]), "Each of persons must be an object."),
        (json!([{"taxable_wages": "1.00"}]), "Each person needs an id string."),
        (
            json!([{"id": "A", "taxable_wages": 500000}]),
            "taxable_wages must be an amount as a string.",
        ),
        (
            json!([{"id": "A", "taxable_wages": "1.00", "carries_on_business": "yes"}]),
            "carries_on_business must be true or false.",
        ),
    ] {
        let (code, text) = refused(&group(json!({"persons": persons})));
        assert_eq!(code, "payroll_tax_group.fact_invalid");
        assert_eq!(text, message);
    }
}

/// An exclusions list given as text or an object, or naming a person twice,
/// and controlling-interest holders naming no one, are refused.
#[test]
fn malformed_id_lists_are_refused() {
    for (changes, message) in [
        (json!({"exclusions": "C"}), "exclusions must be a list of person ids."),
        (json!({"exclusions": {"C": true}}), "exclusions must be a list of person ids."),
        (json!({"exclusions": ["C", "C"]}), "exclusions names \"C\" twice."),
        (
            json!({"controlling_interests": [
                {"holders": [], "business": "B", "test": "sole_owner"},
            ]}),
            "holders must name at least one person.",
        ),
    ] {
        let (code, text) = refused(&group(changes));
        assert_eq!(code, "payroll_tax_group.fact_invalid");
        assert_eq!(text, message);
    }
}

/// A test the Act states as a fact, such as being the sole owner, has no
/// percentage to meet, so one given with it is refused.
#[test]
fn a_fact_test_takes_no_percent() {
    let facts = json!([
        {"holders": ["A"], "business": "B", "test": "sole_owner", "percent": "100"},
    ]);
    let (code, text) = refused(&group(json!({"controlling_interests": facts})));
    assert_eq!(code, "payroll_tax_group.fact_invalid");
    assert_eq!(text, "The sole_owner test takes no percent.");
}

/// A related pair that does not include the excluded member does not bar
/// its exclusion.
#[test]
fn an_unrelated_pair_does_not_bar_an_exclusion() {
    let input = group(json!({
        "related_bodies_corporate": [["A", "B"]],
        "exclusions": ["C"],
    }));
    let result = run(&input);
    assert_eq!(result["group"], json!(["A", "B"]));
    assert_eq!(result["excluded"], json!(["C"]));
}

/// A group paying no wages in Australia has no deduction to share.
#[test]
fn a_group_without_wages_has_a_nil_deduction() {
    let persons = json!([
        {"id": "A", "taxable_wages": "0.00"},
        {"id": "B", "taxable_wages": "0.00"},
        {"id": "C", "taxable_wages": "0.00"},
    ]);
    let result = run(&group(json!({"persons": persons})));
    assert_eq!(result["amounts"]["group_deduction"], "0.00");
}

/// Queensland's excess deduction order given as text, not a list of ids, is
/// refused.
#[test]
fn an_excess_order_must_be_a_list() {
    let input = group(json!({
        "jurisdiction": "qld",
        "related_bodies_corporate": [["A", "B"]],
        "designated_group_employers": ["A"],
        "excess_deduction_order": "B",
    }));
    let (code, _) = refused(&input);
    assert_eq!(code, "payroll_tax_group.fact_invalid");
}
