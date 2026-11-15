// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The board worksheet against the ATO's worked examples
//! (docs/calculation-evidence.md#fbt-board), the otherwise deductible rule and
//! its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// FBT return 2026 instructions, item 23J example 13: board meals valued at
/// $21,900, 10,950 meals at $2, with no contribution or deduction.
fn board(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "adult_meals": 10950, "child_meals": 0, "employee_contribution": "0.00",
        "income_use": "0", "contribution_deduction": "0.00",
        "year_ended": 2026, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("fbt-board", &input)
}

fn amounts(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"].clone()
}

#[test]
fn matches_the_fbt_return_2026_example() {
    let result = board(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_board");
    assert_eq!(result["period"], "year ended 31 March 2026");
    assert_eq!(
        result["amounts"],
        json!({"gross_taxable_value": "21900.00", "employee_contribution_applied": "0.00",
               "excess_employee_contribution": "0.00", "value_after_contribution": "21900.00",
               "gross_deduction": "0.00", "contribution_deduction_applied": "0.00",
               "notional_deduction": "0.00", "reduction_applied": "0.00",
               "taxable_value": "21900.00"})
    );
    assert_eq!(
        result["rates"],
        json!({"adult_meal_value": "2.00", "child_meal_value": "1.00"})
    );
}

#[test]
fn matches_the_employers_guide_contribution_example() {
    // 13.2: $6 a day towards 3 meals a day takes each day's board to nil.
    let a = amounts(board(
        json!({"adult_meals": 3, "employee_contribution": "6.00"}),
    ));
    assert_eq!(a["gross_taxable_value"], "6.00");
    assert_eq!(a["employee_contribution_applied"], "6.00");
    assert_eq!(a["taxable_value"], "0.00");
    // A week's $182 towards board and lodging, $42 of it for the 21 meals,
    // leaves nothing; the $140 for accommodation is not a board contribution.
    let a = amounts(board(
        json!({"adult_meals": 21, "employee_contribution": "182.00"}),
    ));
    assert_eq!(a["excess_employee_contribution"], "140.00");
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn a_meal_for_a_child_under_12_is_1_dollar() {
    // s 36: $1.00 for a person not 12 before the FBT year began.
    let a = amounts(board(json!({"adult_meals": 2, "child_meals": 3,
                                 "employee_contribution": "1.50"})));
    assert_eq!(a["gross_taxable_value"], "7.00");
    assert_eq!(a["taxable_value"], "5.50");
    let a = amounts(board(json!({"adult_meals": 0, "child_meals": 0})));
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn the_otherwise_deductible_rule_reduces_board() {
    // s 37: had the employee paid for the meals, half deductible, with $10 of
    // the $20 paid deducted: GD $10,950, RD $10, ND $10,940.
    let a = amounts(board(
        json!({"employee_contribution": "20.00", "income_use": "0.5",
                                 "contribution_deduction": "10.00"}),
    ));
    assert_eq!(a["value_after_contribution"], "21880.00");
    assert_eq!(a["gross_deduction"], "10950.00");
    assert_eq!(a["notional_deduction"], "10940.00");
    assert_eq!(a["taxable_value"], "10940.00");
    let a = amounts(board(json!({"income_use": "1"})));
    assert_eq!(a["taxable_value"], "0.00");
    // 2027's values are the same.
    let a = amounts(board(json!({"year_ended": 2027})));
    assert_eq!(a["taxable_value"], "21900.00");
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let cases = [
        (
            board(json!({"adult_meals": -1})),
            "adult_meals must be a JSON integer from 0 to 100000000.",
        ),
        (
            board(json!({"child_meals": 1.5})),
            "child_meals must be a JSON integer from 0 to 100000000.",
        ),
        (
            board(json!({"adult_meals": 100_000_001})),
            "adult_meals must be a JSON integer from 0 to 100000000.",
        ),
        (
            board(json!({"employee_contribution": "1.00", "contribution_deduction": "1.01"})),
            "contribution_deduction cannot exceed employee_contribution: it is the deduction for that consideration (FBTAA s 37(c), RD).",
        ),
        (
            board(json!({"income_use": "-0.1"})),
            "income_use must be a finite Decimal fraction from 0 to 1.",
        ),
        (
            board(json!({"method": "per_meal"})),
            "method is not an input of fbt-board",
        ),
        (
            board(json!({"year_ended": 2028})),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
        (
            board(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result.map(|_| ()), Err(message.to_string()));
    }
    let input = json!({"adult_meals": "3", "child_meals": 0, "employee_contribution": "0.00",
        "income_use": "0", "contribution_deduction": "0.00",
        "year_ended": 2026, "scope_confirmed": true});
    let code = api::evaluate_coded("fbt-board", &input).unwrap_err().code;
    assert_eq!(code, "input.benefit_count_invalid");
}

#[test]
fn the_catalogue_lists_it_and_its_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_board"];
    assert_eq!(entry["source_checked"], "2026-11-15");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-board", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "21900.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_board"})).unwrap()[1]["end"],
        "2027-03-31"
    );
}
