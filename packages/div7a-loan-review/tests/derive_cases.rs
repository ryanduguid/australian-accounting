// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `derive` with facts left out, and the input it refuses, called in process.

use div7a_loan_review::derivation::derive;
use serde_json::{Value, json};

fn working(derivation: &Value, id: &str) -> String {
    derivation["steps"]
        .as_array()
        .expect("steps")
        .iter()
        .find(|s| s["id"] == id)
        .and_then(|s| s["working"].as_str())
        .unwrap_or_else(|| panic!("no {id} step"))
        .to_string()
}

#[test]
fn a_derivation_says_which_facts_were_not_supplied() {
    let bare = derive(&json!({ "facts": { "year_of_income": "2026-27" } })).expect("derives");
    assert_eq!(
        working(&bare, "year-loan-made"),
        "Not supplied, so an earlier year of making is not established."
    );
    assert_eq!(
        working(&bare, "complying-terms"),
        "No s 109N gate result was supplied."
    );
    assert_eq!(working(&bare, "remaining-term"), "Not supplied.");
}

#[test]
fn a_gate_result_without_its_benchmark_year_reads_as_unstated() {
    let gate =
        json!({ "verdict": "COMPLYING", "benchmark_year_used": "", "reasons": [], "caveats": [] });
    let derived = derive(&json!({ "facts": {
        "year_of_income": "2026-27",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
        "remaining_term_years": "5",
        "gate_result": gate,
    } }))
    .expect("derives");
    assert_eq!(
        working(&derived, "complying-terms"),
        "The s 109N gate verdict supplied, read against the unstated benchmark. Only a loan \
         on complying terms forms part of an amalgamated loan."
    );
}

#[test]
fn derive_refuses_input_that_is_not_an_object_or_a_decimal_it_cannot_hold() {
    assert_eq!(
        derive(&json!([1])).unwrap_err(),
        "the JSON input must be an object"
    );
    let gate = json!({ "verdict": "COMPLYING", "benchmark_year_used": "2023-24", "reasons": [], "caveats": [] });
    let overflow = derive(&json!({ "facts": {
        "year_of_income": "2026-27",
        "year_loan_made": "2023-24",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "9E+999999",
        "remaining_term_years": "5",
        "payments_applied_during_the_year": "0",
        "gate_result": gate,
    } }));
    assert_eq!(overflow.unwrap_err(), "decimal.InvalidOperation");
    // Refused before the formula runs (a nil remaining term), the same balance
    // still cannot be shown in cents, so writing the derivation's steps fails.
    let refused = derive(&json!({ "facts": {
        "year_of_income": "2026-27",
        "year_loan_made": "2023-24",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "9E+999999",
        "remaining_term_years": "0",
        "payments_applied_during_the_year": "0",
        "gate_result": gate,
    } }));
    assert_eq!(refused.unwrap_err(), "decimal.InvalidOperation");
}
