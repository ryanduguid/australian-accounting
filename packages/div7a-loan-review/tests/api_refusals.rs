// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! What `api::evaluate` refuses: input of the wrong shape or type, named by
//! the field at fault, before any rule runs.

use div7a_loan_review::api::evaluate;
use serde_json::{Value, json};

fn refusal(command: &str, input: Value) -> String {
    evaluate(command, &input).expect_err(command)
}

#[test]
fn input_that_is_not_an_object_is_refused() {
    for input in [json!([1]), json!("x"), json!(null)] {
        assert_eq!(
            refusal("parse-year", input.clone()),
            "input must be a JSON object",
            "{input}"
        );
    }
}

#[test]
fn an_unknown_command_is_named() {
    assert_eq!(
        refusal("no-such-command", json!({})),
        "unknown command no-such-command"
    );
}

#[test]
fn gate_facts_of_the_wrong_type_name_their_field() {
    let gate = |facts: Value| refusal("complying-loan-gate", json!({ "facts": facts }));
    assert_eq!(
        gate(json!({ "secured_by_registered_mortgage_over_real_property": "yes" })),
        "secured_by_registered_mortgage_over_real_property must be a boolean or null"
    );
    assert_eq!(
        gate(json!({ "interest_rate_for_years_after_year_loan_made": 8.77 })),
        "interest_rate_for_years_after_year_loan_made must be a string or null"
    );
}

#[test]
fn repayment_facts_of_the_wrong_type_name_their_field() {
    let myr = |facts: Value| refusal("minimum-yearly-repayment", json!({ "facts": facts }));
    assert_eq!(
        myr(json!({ "amalgamated_loan_unpaid_at_end_of_previous_year": 1000 })),
        "amalgamated_loan_unpaid_at_end_of_previous_year must be a string or null"
    );
    assert_eq!(
        myr(json!({ "gate_result": { "verdict": "COMPLYING", "reasons": "none" } })),
        "reasons must be a list of strings"
    );
}

#[test]
fn register_input_of_the_wrong_shape_is_refused() {
    assert_eq!(
        refusal(
            "review-register",
            json!({ "year_of_income": "2024-25", "rows": { "loan_id": "L1" } })
        ),
        "rows must be a list"
    );
    assert_eq!(
        refusal("load-rows", json!({ "path": "register.csv" })),
        "needed is required"
    );
}

#[test]
fn a_register_row_with_a_bad_year_loan_made_names_the_row_and_column() {
    let refused = refusal(
        "minimum-yearly-repayment",
        json!({ "year_of_income": "2026-27", "mapping": { "year_loan_made": "soon" } }),
    );
    assert!(refused.starts_with("row year_loan_made"), "{refused}");
}
