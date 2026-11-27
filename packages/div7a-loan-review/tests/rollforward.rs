// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `rollforward` inputs it must refuse or disclose: a key printed raw could
//! break the one-line error, a nil repayment splits a period and can move the
//! rounded interest, and a balance floored at nil is an assumption the output
//! has to state. The first-year readings' rounding exception is pinned too, as
//! are hand-worked anchors and the 1 July to 30 June year.

use div7a_loan_review::rollforward::rollforward;
use serde_json::{Value, json};

fn loan(repayments: Value) -> Value {
    json!({"year_of_income": "2026-27", "year_loan_made": "2025-26",
        "balance_at_start_of_year": "1000.00", "repayments": repayments})
}

#[test]
fn an_unknown_key_is_shown_escaped() {
    let mut input = loan(json!([]));
    input["bad\nkey"] = json!("x");
    let refused = rollforward(&input).unwrap_err();
    assert!(!refused.contains('\n'), "{refused}");
    assert!(
        refused.starts_with(r"'bad\nkey' is not a rollforward input"),
        "{refused}"
    );
}

#[test]
fn a_nil_repayment_is_refused() {
    for amount in ["0", "0.00"] {
        let refused =
            rollforward(&loan(json!([{"date": "2026-10-01", "amount": amount}]))).unwrap_err();
        assert!(
            refused.starts_with("repayments[0].amount is 0.00; a repayment of nil"),
            "{refused}"
        );
    }
}

#[test]
fn a_balance_floored_at_nil_is_disclosed() {
    let floored = |result: &Value| {
        result["caveats"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c.as_str().unwrap().contains("taken as nil"))
            .map(|c| c.as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    // 1,022.11 on 1 October leaves the daily balance at -22.11 until 30 June.
    let past = rollforward(&loan(json!([{"date": "2026-10-01", "amount": "1022.11"}]))).unwrap();
    let said = floored(&past);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(
        said[0].starts_with("From 2026-10-01 the repayments exceed the balance"),
        "{said:?}"
    );
    // Repaying the balance exactly, or not at all, floors nothing.
    for repayments in [
        json!([{"date": "2026-10-01", "amount": "1000.00"}]),
        json!([]),
    ] {
        assert!(floored(&rollforward(&loan(repayments)).unwrap()).is_empty());
    }
}

/// The worked example's first-year reading (the 2 July repayment, made before
/// the lodgment day, counted in the year) usually closes higher than the other
/// reading (it counted in the loan year), but rounding each period to cents
/// can leave it a cent lower, as the caveat and README say.
#[test]
fn the_first_year_readings_differ_by_rounding_alone_here() {
    let closing = |opening: &str, repayments: Value| {
        let mut input = loan(repayments);
        input["balance_at_start_of_year"] = json!(opening);
        rollforward(&input).unwrap()["amalgamated_loan_unpaid_at_end_of_year"].clone()
    };
    let later = json!([
        {"date": "2026-09-20", "amount": "61.22"},
        {"date": "2027-01-31", "amount": "178.93"},
    ]);
    let mut with_early = later.as_array().unwrap().clone();
    with_early.insert(0, json!({"date": "2026-07-02", "amount": "8.13"}));
    assert_eq!(closing("436.48", Value::Array(with_early)), "215.09");
    assert_eq!(closing("428.35", later), "215.10");
}

/// A $70,000 loan worked by hand from s 109E(7), the 2024-25 and 2025-26
/// benchmark rates and the ATO's daily-balance day count: every day of the
/// year counts, and a repayment reduces the balance from its own date.
#[test]
fn the_hand_worked_anchors_hold() {
    let file: Value = serde_json::from_str(include_str!("data/rollforward-anchors.json")).unwrap();
    assert_eq!(file["synthetic"], true);
    let cases = file["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    for case in cases {
        let id = &case["case_id"];
        let result = rollforward(&case["input"]).unwrap_or_else(|e| panic!("{id}: {e}"));
        assert_eq!(result["days_in_year"], 365, "{id}");
        for key in ["interest", "amalgamated_loan_unpaid_at_end_of_year"] {
            assert_eq!(result[key], case[key], "{id} {key}");
        }
    }
}

/// The year of income is a label, so a substituted accounting period cannot be
/// entered: a start date in its place is refused, the year runs 1 July to
/// 30 June, and the README says such a period is not modelled.
#[test]
fn a_substituted_accounting_period_cannot_be_entered() {
    let mut input = loan(json!([]));
    input["year_of_income"] = json!("2026-11-01");
    let refused = rollforward(&input).unwrap_err();
    assert!(
        refused.starts_with("year_of_income '2026-11-01' must be written like 2026-27"),
        "{refused}"
    );
    for (date, inside) in [
        ("2026-06-30", false),
        ("2026-07-01", true),
        ("2027-06-30", true),
        ("2027-07-01", false),
    ] {
        let result = rollforward(&loan(json!([{"date": date, "amount": "1.00"}])));
        assert_eq!(result.is_ok(), inside, "{date}: {result:?}");
    }
    let readme = include_str!("../README.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(readme.contains("(a substituted accounting period is not modelled, and the ATO calculator does not take one either)"));
}

#[test]
fn a_repayment_missing_a_field_or_with_a_number_is_named() {
    let refused = rollforward(&loan(json!([{"date": "2026-10-01"}]))).unwrap_err();
    assert_eq!(refused, "repayments[0] amount is required");
    let refused = rollforward(&loan(json!([{"date": 20261001, "amount": "5.00"}]))).unwrap_err();
    assert_eq!(refused, "repayments[0] date must be a string");
}

#[test]
fn a_balance_repaid_before_a_later_repayment_is_floored_from_that_date() {
    let result = rollforward(&loan(json!([
        {"date": "2026-08-01", "amount": "1500.00"},
        {"date": "2026-09-01", "amount": "10.00"},
    ])))
    .unwrap();
    let caveats: Vec<&str> = result["caveats"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        caveats.iter().any(|c| c.starts_with(
            "From 2026-08-01 the repayments exceed the balance, so the balance is taken as nil"
        )),
        "{caveats:?}"
    );
}
