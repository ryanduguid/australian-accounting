// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The scholarship exemption (ITAA 1997 s 51-10 item 2.1A and s 51-35, read
//! in compilations 170 and 268) against the ATO's 'Scholarship payments'
//! examples: Jenny, Henry, Kirra, AAA University, Megan and Diego
//! (docs/calculation-evidence.md#scholarship-exemption).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("scholarship-exemption", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("scholarship-exemption", input)
        .unwrap_err()
        .code
}

fn facts(full_time: bool, employment: bool, labour: bool, educational: bool) -> Value {
    json!({
        "year": "2025-26", "scope_confirmed": true, "payment_type": "scholarship",
        "full_time_student": full_time, "employment_condition": employment,
        "labour_contract_condition": labour, "principally_educational": educational,
        "commonwealth_payment": "none", "friendly_society_plan": false,
    })
}

fn with(mut input: Value, key: &str, value: Value) -> Value {
    input[key] = value;
    input
}

#[test]
fn the_atos_examples() {
    let rows = [
        // Jenny: a full-time journalism student's award, no conditions.
        ("Jenny", facts(true, false, false, true), json!([])),
        // Henry: must work for the payer after graduating.
        (
            "Henry",
            facts(true, true, false, true),
            json!(["no_employment_condition"]),
        ),
        // Kirra: studies part time.
        (
            "Kirra",
            facts(false, false, false, true),
            json!(["full_time_student"]),
        ),
        // AAA University's vet students: work experience is part of the course.
        ("AAA University", facts(true, false, false, true), json!([])),
        // Megan: work experience for her education, no employment requirement.
        ("Megan", facts(true, false, false, true), json!([])),
        // Diego: the payer's output, not his education, is the main factor.
        (
            "Diego",
            facts(true, false, false, false),
            json!(["principally_educational"]),
        ),
    ];
    for (name, input, failed) in rows {
        let out = run(&input);
        assert_eq!(out["exempt"], json!(failed == json!([])), "{name}");
        assert_eq!(out["conditions_not_met"], failed, "{name}");
        assert_eq!(out["conditions"].as_array().unwrap().len(), 4, "{name}");
    }
}

#[test]
fn a_labour_contract_condition_and_every_payment_type() {
    let out = run(&facts(true, false, true, true));
    assert_eq!(
        out["conditions_not_met"],
        json!(["no_labour_contract_condition"])
    );
    assert!(out["treatment"].as_str().unwrap().starts_with("Not exempt"));
    for kind in [
        "scholarship",
        "bursary",
        "educational_allowance",
        "educational_assistance",
    ] {
        let out = run(&with(
            facts(true, false, false, true),
            "payment_type",
            json!(kind),
        ));
        assert_eq!(out["payment_type"], kind);
        assert_eq!(out["exempt"], true);
    }
}

#[test]
fn refusals() {
    let ok = facts(true, false, false, true);
    for payment in [
        "secondary_or_isolated_children",
        "education_or_training_payment",
        "education_entry_payment",
    ] {
        assert_eq!(
            code(&with(ok.clone(), "commonwealth_payment", json!(payment))),
            "scholarship_exemption.commonwealth_payment"
        );
    }
    let cases = [
        (
            with(ok.clone(), "commonwealth_payment", json!("other")),
            "scholarship_exemption.commonwealth_payment_invalid",
        ),
        (
            with(ok.clone(), "friendly_society_plan", json!(true)),
            "scholarship_exemption.friendly_society_plan",
        ),
        (
            with(ok.clone(), "payment_type", json!("prize")),
            "scholarship_exemption.payment_type_invalid",
        ),
        (
            with(ok.clone(), "full_time_student", json!("yes")),
            "input.flag_invalid",
        ),
        (
            with(ok.clone(), "year", json!("2016-17")),
            "period.unsupported",
        ),
        (
            with(ok.clone(), "year", json!("2027-28")),
            "period.unsupported",
        ),
        (
            with(ok, "scope_confirmed", json!(false)),
            "scope.not_confirmed",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(code(&input), expected, "{input}");
    }
}
