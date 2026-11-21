// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The loan transactions statement: daily interest at a rate that changes on
//! dated events, debited at each month's end, dated drawings and repayments,
//! interest by income year and the printable statement, worked by hand at
//! 7.3% a year, which is exactly 0.02% a day on 365 days
//! (docs/calculation-evidence.md#loan-transactions).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// $10,000 drawn on 1 July 2026 at 7.3%, $1,000 repaid on 31 August, the
/// statement to 30 September 2026. A `null` override drops the input.
fn input(overrides: Value) -> Value {
    let mut input = json!({
        "opening_balance": "0.00", "annual_rate": "0.073",
        "start_date": "2026-07-01", "end_date": "2026-09-30",
        "transactions": [
            {"date": "2026-07-01", "kind": "drawing", "amount": "10000.00"},
            {"date": "2026-08-31", "kind": "repayment", "amount": "1000.00"},
        ],
        "year": "2026-27", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            input.as_object_mut().unwrap().remove(k);
        } else {
            input[k] = v.clone();
        }
    }
    input
}

fn run(overrides: Value) -> Value {
    let input = input(overrides);
    let result = api::evaluate("loan-transactions", &input).unwrap();
    let envelope = json!({"command": "loan-transactions", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded("loan-transactions", &input(overrides))
        .unwrap_err()
        .code
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// The interest lines: (date, debit).
fn interest(result: &Value) -> Vec<(String, String)> {
    result["statement_rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["details"] == "Interest")
        .map(|row| {
            (
                row["date"].as_str().unwrap().to_string(),
                row["debit"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// By hand: July is 31 days at $2.00, $62.00, so $10,062.00; August 30 days
/// at $2.0124 and 31 August, after the repayment, at $1.8124, $62.1844, so
/// $62.18 and $9,124.18; September 30 days at $1.824836, $54.74508, so $54.75
/// and $9,178.93.
#[test]
fn interest_accrues_daily_and_is_debited_at_each_month_end() {
    let r = run(json!({}));
    assert_eq!(
        interest(&r),
        [
            ("2026-07-31".into(), "62.00".into()),
            ("2026-08-31".into(), "62.18".into()),
            ("2026-09-30".into(), "54.75".into()),
        ]
    );
    for (key, value) in [
        ("opening_balance", "0.00"),
        ("total_drawings", "10000.00"),
        ("total_repayments", "1000.00"),
        ("total_interest", "178.93"),
        ("closing_balance", "9178.93"),
    ] {
        assert_eq!(amount(&r, key), value, "{key}");
    }
    assert_eq!(
        r["interest_by_income_year"],
        json!([{"income_year": "2026-27", "interest": "178.93"}])
    );
    let printed = r["statement"].as_str().unwrap();
    assert!(
        printed.starts_with(
            "| Date | Details | Debit | Credit | Balance |\n| --- | --- | ---: | ---: | ---: |\n"
        ),
        "{printed}"
    );
    assert!(
        printed.contains("| 2026-08-31 | Repayment |  | 1000.00 | 9062.00 |"),
        "{printed}"
    );
    assert!(
        printed.contains("| 2026-09-30 | Interest | 54.75 |  | 9178.93 |"),
        "{printed}"
    );
}

/// From 1 August at 3.65%, $0.0001 a dollar a day: August 30 days at
/// $1.0062 and 31 August at $0.9062, $31.0922, so $31.09.
#[test]
fn a_rate_change_applies_from_its_date() {
    let r = run(json!({"rate_changes": [{"date": "2026-08-01", "annual_rate": "0.0365"}]}));
    let lines = interest(&r);
    assert_eq!(lines[0].1, "62.00");
    assert_eq!(lines[1].1, "31.09");
    let row = r["statement_rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["date"] == "2026-08-01")
        .unwrap();
    assert_eq!(row["details"], "Annual rate 0.0365 from this day");
}

/// Across 30 June: June's 30 days at $2.00 are 2025-26's $60.00; July's 31
/// days at $2.012 are 2026-27's $62.372, so $62.37.
#[test]
fn each_income_year_has_the_interest_debited_in_it() {
    let r = run(
        json!({"start_date": "2026-06-01", "end_date": "2026-07-31", "year": "2025-26",
                       "transactions": [{"date": "2026-06-01", "kind": "drawing", "amount": "10000.00"}]}),
    );
    assert_eq!(
        r["interest_by_income_year"],
        json!([
            {"income_year": "2025-26", "interest": "60.00"},
            {"income_year": "2026-27", "interest": "62.37"},
        ])
    );
}

/// Paid out on 10 July: 9 days at $2.00 are $18.00, so $10,018.00 clears it.
/// The payout day itself ends with nothing owing and accrues nothing.
#[test]
fn a_payout_with_the_interest_to_date_ends_at_nil() {
    let paid_out = |amount: &str| {
        json!({"end_date": "2026-07-10", "transactions": [
            {"date": "2026-07-01", "kind": "drawing", "amount": "10000.00"},
            {"date": "2026-07-10", "kind": "repayment", "amount": amount},
        ]})
    };
    let r = run(paid_out("10018.00"));
    assert_eq!(amount(&r, "total_interest"), "18.00");
    assert_eq!(amount(&r, "closing_balance"), "0.00");
    assert_eq!(
        code(paid_out("10018.01")),
        "loan_transactions.balance_below_nil"
    );
}

/// An opening balance with no transactions accrues from the first day.
#[test]
fn an_opening_balance_accrues_from_the_start() {
    let r =
        run(json!({"opening_balance": "10000.00", "transactions": [], "end_date": "2026-07-31"}));
    assert_eq!(amount(&r, "total_interest"), "62.00");
    assert_eq!(amount(&r, "total_drawings"), "0.00");
}

#[test]
fn input_the_statement_cannot_represent_is_refused() {
    let unordered = json!([
        {"date": "2026-08-01", "kind": "drawing", "amount": "1.00"},
        {"date": "2026-07-01", "kind": "drawing", "amount": "1.00"},
    ]);
    for (overrides, expected) in [
        (
            json!({"transactions": unordered}),
            "loan_transactions.not_in_date_order",
        ),
        (
            json!({"transactions": [{"date": "2026-10-01", "kind": "drawing", "amount": "1.00"}]}),
            "loan_transactions.date_outside_statement",
        ),
        (
            json!({"transactions": [{"date": "2026-07-01", "kind": "fee", "amount": "1.00"}]}),
            "loan_transactions.entry_invalid",
        ),
        (
            json!({"transactions": [{"date": "2026-07-01", "kind": "drawing", "amount": "1.00", "memo": "x"}]}),
            "input.field_unknown",
        ),
        (
            json!({"transactions": {"date": "2026-07-01"}}),
            "loan_transactions.entry_invalid",
        ),
        (
            json!({"rate_changes": [{"date": "2026-07-01", "annual_rate": "0.05"}]}),
            "loan_transactions.date_outside_statement",
        ),
        (
            json!({"rate_changes": [{"date": "2026-08-01", "annual_rate": "0.05"},
                                    {"date": "2026-08-01", "annual_rate": "0.06"}]}),
            "loan_transactions.not_in_date_order",
        ),
        (
            json!({"rate_changes": [{"date": "2026-08-01", "annual_rate": "1.05"}]}),
            "input.fraction_invalid",
        ),
        (json!({"annual_rate": "-0.01"}), "input.fraction_invalid"),
        (
            json!({"start_date": "2026-06-30"}),
            "loan_transactions.start_outside_year",
        ),
        (
            json!({"end_date": "2026-06-30"}),
            "loan_transactions.end_date_invalid",
        ),
        (
            json!({"end_date": "2056-07-01"}),
            "loan_transactions.end_date_invalid",
        ),
        (
            json!({"transactions": [{"date": "2026-07-01", "kind": "repayment", "amount": "1.00"}]}),
            "loan_transactions.balance_below_nil",
        ),
        (json!({"transactions": null}), "input.missing_field"),
        (json!({"year": "2027-28"}), "period.unsupported"),
    ] {
        assert_eq!(code(overrides.clone()), expected, "{overrides}");
    }
}
