// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! GST corrections (F2023L01284 ss 6 and 7) and credits pending a tax invoice
//! (GST Act ss 29-10(3), 29-80 and 93-5 and reg 29-80.01), read on
//! 24 November 2026, with each boundary
//! (docs/calculation-evidence.md#gst-error-correction).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const EC: &str = "gst-error-correction";
const CT: &str = "gst-credit-time-limit";

fn with(mut input: Value, overrides: Value) -> Value {
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

fn correction(overrides: Value) -> Value {
    with(
        json!({
            "error_kind": "debit", "earlier_period_start": "2024-10-01",
            "earlier_return_due": "2025-02-28", "lodged": "2025-12-28",
            "current_gst_turnover": "1500000", "net_debit_errors": "4000.00",
            "within_period_of_review": true, "compliance_activity": false,
            "commissioner_permits": false, "already_corrected": false,
            "reckless_or_intentional": false, "credit_ceased_under_division_93": false,
            "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

fn credit(overrides: Value) -> Value {
    with(
        json!({
            "supply_value": "500.00", "return_due": "2025-10-28",
            "tax_invoice_held_from": "2025-09-15", "return_given": "2025-10-20",
            "assessment_date": "2025-10-20", "supplier_review_ended_without_invoice": false,
            "special_attribution": false, "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

fn run(command: &str, input: &Value) -> Value {
    api::evaluate(command, input).unwrap()
}

fn code(command: &str, input: &Value) -> &'static str {
    api::evaluate_coded(command, input).unwrap_err().code
}

/// A small business's debit error inside 18 months and under $12,500.
#[test]
fn a_debit_error_within_the_limits_may_be_corrected() {
    let r = run(EC, &correction(json!({})));
    assert_eq!(r["may_correct"], true);
    assert_eq!(r["conditions"].as_array().unwrap().len(), 8);
    assert_eq!(r["debit_error_time_limit"], "2026-08-28");
    assert_eq!(r["debit_error_value_limit"], "12500");
}

/// s 7 table: each turnover band's months and value limit, at its boundary.
#[test]
fn each_turnover_band_sets_its_limits() {
    for (turnover, months_end, limit) in [
        ("19999999.99", "2026-08-28", "12500"),
        ("20000000", "2026-02-28", "25000"),
        ("99999999.99", "2026-02-28", "25000"),
        ("100000000", "2026-02-28", "50000"),
        ("500000000.00", "2026-02-28", "100000"),
        ("1000000000", "2026-02-28", "560000"),
    ] {
        let r = run(
            EC,
            &correction(json!({"current_gst_turnover": turnover, "lodged": "2026-01-01"})),
        );
        assert_eq!(r["debit_error_time_limit"], months_end, "{turnover}");
        assert_eq!(r["debit_error_value_limit"], limit, "{turnover}");
    }
}

/// The time limit includes its last day; the value limit is 'less than'.
#[test]
fn the_debit_limits_boundaries() {
    let last = run(
        EC,
        &correction(json!({"lodged": "2026-08-28", "year": "2026-27"})),
    );
    assert_eq!(last["may_correct"], true);
    let late = run(
        EC,
        &correction(json!({"lodged": "2026-08-29", "year": "2026-27"})),
    );
    assert_eq!(late["failing_conditions"], json!(["within_time_limit"]));
    let at = run(EC, &correction(json!({"net_debit_errors": "12500.00"})));
    assert_eq!(at["failing_conditions"], json!(["below_value_limit"]));
    let under = run(EC, &correction(json!({"net_debit_errors": "12499.99"})));
    assert_eq!(under["may_correct"], true);
}

/// A credit error has no s 7 limits.
#[test]
fn a_credit_error_has_no_time_or_value_limit() {
    let r = run(
        EC,
        &correction(
            json!({"error_kind": "credit", "net_debit_errors": "900000.00",
                           "lodged": "2027-01-01", "year": "2026-27"}),
        ),
    );
    assert_eq!(r["may_correct"], true);
    assert_eq!(r["conditions"].as_array().unwrap().len(), 5);
    assert!(r.get("debit_error_time_limit").is_none());
}

/// Each s 6 and s 7 condition failing in turn.
#[test]
fn each_correction_condition_can_fail() {
    for (overrides, failing) in [
        (
            json!({"credit_ceased_under_division_93": true}),
            "relates_to_gst",
        ),
        (
            json!({"earlier_period_start": "2012-06-30", "earlier_return_due": "2012-07-28",
                "lodged": "2025-12-28"}),
            "period_started_from_1_july_2012",
        ),
        (
            json!({"within_period_of_review": false}),
            "within_period_of_review",
        ),
        (
            json!({"compliance_activity": true}),
            "no_compliance_activity",
        ),
        (json!({"already_corrected": true}), "not_already_corrected"),
        (json!({"reckless_or_intentional": true}), "not_reckless"),
    ] {
        let r = run(EC, &correction(overrides.clone()));
        let failed = r["failing_conditions"].as_array().unwrap();
        assert!(failed.contains(&json!(failing)), "{overrides}: {failed:?}");
    }
    let permitted = correction(json!({"compliance_activity": true, "commissioner_permits": true}));
    assert_eq!(run(EC, &permitted)["may_correct"], true);
}

#[test]
fn the_correction_refusals() {
    for (overrides, expected) in [
        (
            json!({"error_kind": "both"}),
            "gst_error_correction.fact_invalid",
        ),
        (
            json!({"earlier_return_due": "2024-09-30"}),
            "gst_error_correction.dates_invalid",
        ),
        (
            json!({"lodged": "2025-02-27", "year": "2024-25"}),
            "gst_error_correction.dates_invalid",
        ),
        (
            json!({"lodged": "2026-07-01"}),
            "gst_error_correction.lodged_outside_year",
        ),
        (json!({"lodged": "28/12/2025"}), "input.date_invalid"),
        (
            json!({"current_gst_turnover": "-1"}),
            "input.amount_invalid",
        ),
        (json!({"already_corrected": "no"}), "input.flag_invalid"),
        (json!({"year": "2023-24"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"extra": true}), "input.field_unknown"),
    ] {
        assert_eq!(
            code(EC, &correction(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}

/// s 29-10(3): with the tax invoice held when the return is given, the credit
/// is claimable; s 93-5 ends the entitlement 4 years after the return's due day.
#[test]
fn a_credit_with_its_tax_invoice_is_claimable() {
    let r = run(CT, &credit(json!({})));
    assert_eq!(r["claimable"], true);
    assert_eq!(r["tax_invoice_needed"], true);
    assert_eq!(r["entitlement_ends"], "2029-10-28");
}

/// Without the invoice the credit waits; $75 or less needs none.
#[test]
fn a_tax_invoice_is_needed_above_75_dollars() {
    for (overrides, claimable) in [
        (json!({"tax_invoice_held_from": "not_held"}), false),
        (json!({"tax_invoice_held_from": "2025-10-21"}), false),
        (json!({"tax_invoice_held_from": "2025-10-20"}), true),
        (
            json!({"tax_invoice_held_from": "not_held", "supply_value": "75.00"}),
            true,
        ),
        (
            json!({"tax_invoice_held_from": "not_held", "supply_value": "75.01"}),
            false,
        ),
    ] {
        let r = run(CT, &credit(overrides.clone()));
        assert_eq!(r["claimable"], claimable, "{overrides}");
    }
}

/// s 93-5(1): the last day of the 4 years counts; the next does not.
#[test]
fn the_entitlement_lapses_after_4_years() {
    let due = json!({"return_due": "2021-10-28", "return_given": "2025-10-20"});
    let last = run(
        CT,
        &credit(with(due.clone(), json!({"assessment_date": "2025-10-28"}))),
    );
    assert_eq!(last["claimable"], true);
    let late = run(
        CT,
        &credit(with(due, json!({"assessment_date": "2025-10-29"}))),
    );
    assert_eq!(late["failing_conditions"], json!(["within_4_years"]));
    let supplier = run(
        CT,
        &credit(json!({"supplier_review_ended_without_invoice": true})),
    );
    assert_eq!(
        supplier["failing_conditions"],
        json!(["supplier_review_open"])
    );
}

#[test]
fn the_credit_refusals() {
    for (overrides, expected) in [
        (
            json!({"special_attribution": true}),
            "gst_credit_time_limit.special_attribution",
        ),
        (
            json!({"assessment_date": "2025-10-19"}),
            "gst_credit_time_limit.dates_invalid",
        ),
        (
            json!({"return_given": "2026-07-01", "assessment_date": "2026-07-01"}),
            "gst_credit_time_limit.return_given_outside_year",
        ),
        (
            json!({"tax_invoice_held_from": "held"}),
            "input.date_invalid",
        ),
        (json!({"supply_value": "1.001"}), "input.amount_invalid"),
        (json!({"year": "2023-24"}), "period.unsupported"),
        (json!({"extra": true}), "input.field_unknown"),
    ] {
        assert_eq!(
            code(CT, &credit(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}

/// The catalogue's examples run.
#[test]
fn the_catalogue_examples_run() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    for (command, kind) in [(EC, "gst_error_correction"), (CT, "gst_credit_time_limit")] {
        let mut facts = catalogue[kind]["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        facts["scope_confirmed"] = json!(true);
        assert!(api::evaluate(command, &facts).is_ok(), "{command}");
    }
}
