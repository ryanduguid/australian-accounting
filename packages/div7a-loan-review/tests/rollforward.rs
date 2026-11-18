// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `rollforward` inputs it must refuse or disclose: a key printed raw could
//! break the one-line error, a nil repayment splits a period and can move the
//! rounded interest, and a balance floored at nil is an assumption the output
//! has to state. The first-year readings' rounding exception is pinned too.

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
