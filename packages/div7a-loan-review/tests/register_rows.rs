// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The `load-rows` command: a register's rows as DictReader builds them,
//! each as `[[column, value], ...]`. Expected values are CPython 3.15's
//! `csv.DictReader`. All data is fabricated.

use div7a_loan_review::api::evaluate;
use serde_json::{Value, json};

/// A fresh register file holding `data`, named for this process and test.
fn register(name: &str, data: &str) -> String {
    let dir = std::env::temp_dir().join(format!("div7a-rows-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch directory");
    let path = dir.join("register.csv");
    std::fs::write(&path, data).expect("scratch file");
    path.to_string_lossy().into_owned()
}

fn load_rows(name: &str, data: &str) -> Value {
    let input = json!({ "path": register(name, data), "needed": ["name"] });
    evaluate("load-rows", &input).expect("rows")
}

#[test]
fn a_repeated_column_keeps_its_first_place_and_its_last_value() {
    // dict(zip(fieldnames, row)): the later loan_id cell replaces the earlier one.
    assert_eq!(
        load_rows("repeated", "loan_id,name,loan_id\nA,n,B\n"),
        json!([[["loan_id", "B"], ["name", "n"]]])
    );
}

#[test]
fn a_short_row_fills_every_missing_column_with_none() {
    // restval None is written for each field past the row's end, so the
    // repeated loan_id ends as None even though the row's first cell was set.
    assert_eq!(
        load_rows("short", "loan_id,name,loan_id\nC\n"),
        json!([[["loan_id", null], ["name", null]]])
    );
}

const HEADER: &str = "loan_id,year_loan_made,written_agreement,terms_in_place_before_lodgment_day,\
maximum_term_years,secured_by_registered_mortgage_over_real_property,\
security_coverage_at_first_made,interest_rate_for_years_after_year_loan_made,\
amalgamated_loan_unpaid_at_end_of_previous_year,remaining_term_years,\
payments_applied_during_the_year";

#[test]
fn a_balance_too_large_to_work_in_cents_stops_the_register_review() {
    // A complying 7-year loan whose unpaid balance, 9E+999999, cannot be
    // quantised to cents: the decimal signal ends the review, as the Python
    // engine's uncaught decimal.InvalidOperation does, from rows or a file.
    let row = "F1,2023-24,true,true,7,false,unknown,0.0827,9E+999999,5,0.00";
    let mapping: serde_json::Map<String, Value> = HEADER
        .split(',')
        .zip(row.split(','))
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect();
    let rows = json!({ "rows": [mapping], "year_of_income": "2026-27" });
    assert_eq!(
        evaluate("review-register", &rows).unwrap_err(),
        "decimal.InvalidOperation"
    );
    let path = register("too-large", &format!("{HEADER}\n{row}\n"));
    let file = json!({ "path": path, "year_of_income": "2026-27" });
    assert_eq!(
        evaluate("review-register-file", &file).unwrap_err(),
        "decimal.InvalidOperation"
    );
}
