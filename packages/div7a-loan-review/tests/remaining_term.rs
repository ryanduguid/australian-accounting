// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The `minimum-yearly-repayment` command and `derive` apply the register
//! review's remaining-term limits (s 109E(6)) when they are given the loan's
//! term, `maximum_term_years`, and a figure worked without it says that no
//! limit was checked. All facts are fabricated.

use div7a_loan_review::{api, derivation};
use serde_json::{Value, json};

const COMPLYING: &str = "COMPLYING";
const NOT_CHECKED: &str = "maximum_term_years was not supplied";

/// A 7-year-style loan made in 2024-25 that the gate passed.
fn facts(term: Option<&str>, year: &str, remaining: &str) -> Value {
    let mut facts = json!({"year_of_income": year, "year_loan_made": "2024-25",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
        "remaining_term_years": remaining, "payments_applied_during_the_year": "0.00",
        "loan_id": "L", "gate_result": {"verdict": COMPLYING,
            "benchmark_year_used": "2024-25", "reasons": [], "caveats": []}});
    if let Some(term) = term {
        facts["maximum_term_years"] = json!(term);
    }
    json!({ "facts": facts })
}

fn repayment(input: &Value) -> Value {
    api::evaluate("minimum-yearly-repayment", input).unwrap()
}

fn not_checked(answer: &Value) -> bool {
    answer["caveats"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c.as_str().unwrap().starts_with(NOT_CHECKED))
}

#[test]
fn a_seven_year_loan_has_six_years_left_in_its_second_repayment_year() {
    assert_eq!(
        repayment(&facts(Some("7"), "2025-26", "7"))["verdict"],
        "MYR_SHORT"
    );
    assert_eq!(
        repayment(&facts(Some("7"), "2026-27", "6"))["verdict"],
        "MYR_SHORT"
    );
    for remaining in ["7", "6.5"] {
        let refused = repayment(&facts(Some("7"), "2026-27", remaining));
        assert_eq!(refused["verdict"], "REFUSED", "{remaining}");
        assert_eq!(
            refused["reason_codes"],
            json!(["REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED"])
        );
        assert_eq!(refused["myr_required"], Value::Null);
    }
}

#[test]
fn a_remaining_term_past_the_loans_own_term_is_refused() {
    let refused = repayment(&facts(Some("5"), "2025-26", "7"));
    assert_eq!(
        refused["reason_codes"],
        json!(["REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM"])
    );
}

#[test]
fn a_register_row_given_as_mapping_carries_its_term() {
    let row = |term: &str| {
        json!({"year_of_income": "2026-27",
            "gate_result": {"verdict": COMPLYING, "benchmark_year_used": "2024-25",
                "reasons": [], "caveats": []},
            "mapping": {"loan_id": "L", "maximum_term_years": term, "year_loan_made": "2024-25",
                "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
                "remaining_term_years": "7", "payments_applied_during_the_year": "0.00"}})
    };
    assert_eq!(
        repayment(&row("7"))["reason_codes"],
        json!(["REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED"])
    );
    assert_eq!(repayment(&row("8"))["verdict"], "MYR_SHORT");
    assert!(
        api::evaluate("minimum-yearly-repayment", &row("seven"))
            .unwrap_err()
            .contains("row maximum_term_years")
    );
}

#[test]
fn a_figure_worked_without_the_loans_term_says_no_limit_was_checked() {
    let bare = repayment(&facts(None, "2026-27", "7"));
    assert_eq!(bare["verdict"], "MYR_SHORT");
    assert!(not_checked(&bare));
    assert!(!not_checked(&repayment(&facts(Some("7"), "2026-27", "6"))));
    let mut unpaid = facts(None, "2026-27", "7");
    unpaid["facts"]["payments_applied_during_the_year"] = Value::Null;
    assert!(!not_checked(&repayment(&unpaid)), "no figure, no caveat");
}

#[test]
fn a_derivation_gives_the_commands_answer() {
    for input in [
        facts(Some("7"), "2026-27", "7"),
        facts(None, "2026-27", "7"),
    ] {
        let derived = derivation::derive(&input).unwrap();
        assert_eq!(derived["result"], repayment(&input));
    }
}

#[test]
fn a_refusal_without_the_loans_term_carries_no_caveat() {
    let refused = repayment(&facts(None, "2024-25", "7"));
    assert_eq!(refused["verdict"], "REFUSED");
    assert!(!not_checked(&refused));
}

#[test]
fn a_term_in_facts_that_is_not_a_decimal_is_an_error() {
    assert!(
        api::evaluate(
            "minimum-yearly-repayment",
            &facts(Some("seven"), "2026-27", "6")
        )
        .is_err()
    );
}

#[test]
fn a_derivation_with_the_loans_term_verifies() {
    let derived = derivation::derive(&facts(Some("7"), "2026-27", "6")).unwrap();
    assert_eq!(derivation::verify(&derived).unwrap()["verified"], true);
}
