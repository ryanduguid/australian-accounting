// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The worked examples on the ATO's 'Payment deadlines for Payday Super' page
//! (last updated 29 September 2026), read on 5 November 2026. Each date below is
//! the ATO's; the amounts are fabricated. Each count is also worked here by
//! hand under SGAA 1992 s 6(1). Monday 3 August 2026, NT Picnic Day, is the only
//! day other than a weekend that any count below skips.

use payday_super_checker::api;
use serde_json::{Value, json};

fn due(line: Value) -> String {
    let deadline = api::evaluate("compute-due", &json!({ "line": line })).unwrap();
    deadline["due"].as_str().unwrap().to_string()
}

fn payday(qe_day: &str) -> Value {
    json!({"employee_id": "E1", "qe_day": qe_day, "sg_amount": "120.00"})
}

fn first_contribution(qe_day: &str) -> Value {
    let mut line = payday(qe_day);
    line["first_to_fund"] = json!(true);
    line
}

fn out_of_cycle(qe_day: &str, next: &str) -> Value {
    let mut line = payday(qe_day);
    line["out_of_cycle"] = json!(true);
    line["next_standard_qe_day"] = json!(next);
    line
}

fn assessed(lines: Value, as_at: &str) -> Vec<Value> {
    let input = json!({"lines": lines, "as_at": as_at, "allow_stale_gic": true,
                       "transition_allocation_confirmed": true});
    let results = api::evaluate("assess", &input).unwrap();
    results.as_array().unwrap().clone()
}

/// `line` with the whole amount received on `day`.
fn received(mut line: Value, day: &str) -> Value {
    line["remitted"] = json!(day);
    line["remitted_amount"] = json!("120.00");
    line["received"] = json!(day);
    line["matched_amount"] = json!("120.00");
    line
}

/// Example 1, QE day 1. 20 business days from Thursday 9 July: 10 July is the
/// 1st, 17 July the 6th, 24 July the 11th, 31 July the 16th, then 4, 5, 6 and
/// 7 August, past Picnic Day.
#[test]
fn example_1_first_payment_is_due_7_august_2026() {
    assert_eq!(due(first_contribution("2026-07-09")), "2026-08-07");
}

/// Example 1, QE day 2, 'within 7 business days of 30 July 2026, which is
/// 11 August 2026': 31 July, then 4, 5, 6, 7, 10 and 11 August.
#[test]
fn example_1_second_payment_is_due_11_august_2026() {
    assert_eq!(due(payday("2026-07-30")), "2026-08-11");
}

#[test]
fn example_1_second_payment_decides_on_11_august_2026() {
    let on_the_day = received(payday("2026-07-30"), "2026-08-11");
    let results = assessed(json!([on_the_day]), "2026-08-12");
    assert_eq!(results[0]["verdict"], "ON_TIME");
    let next_day = received(payday("2026-07-30"), "2026-08-12");
    let results = assessed(json!([next_day]), "2026-08-12");
    assert_eq!(results[0]["verdict"], "LATE");
    assert_eq!(results[0]["days_late"], 1);
    let results = assessed(json!([payday("2026-07-30")]), "2026-08-12");
    assert_eq!(results[0]["verdict"], "UNPAID");
}

/// Example 2, QE day 1. 20 business days from Friday 7 August: 14 August is the
/// 5th, 21 August the 10th, 28 August the 15th and 4 September the 20th.
#[test]
fn example_2_first_contribution_to_a_new_fund_is_due_4_september_2026() {
    assert_eq!(due(first_contribution("2026-08-07")), "2026-09-04");
}

/// Example 2, QE day 2: 7, 8, 9, 10, 11, 14 and 15 September.
#[test]
fn example_2_next_payment_is_due_15_september_2026() {
    assert_eq!(due(payday("2026-09-04")), "2026-09-15");
}

/// Example 3: 7 business days from the regular 10 December payday: 11, 14, 15,
/// 16, 17, 18 and 21 December.
#[test]
fn example_3_a_bonus_is_due_with_the_next_regular_payday() {
    assert_eq!(due(out_of_cycle("2026-12-07", "2026-12-10")), "2026-12-21");
}

/// Example 4: 7 business days from the regular 3 September payday: 4, 7, 8, 9,
/// 10, 11 and 14 September.
#[test]
fn example_4_a_back_payment_is_due_with_the_next_regular_payday() {
    assert_eq!(due(out_of_cycle("2026-08-29", "2026-09-03")), "2026-09-14");
}

/// Example 6: 23 July is ordinarily due 4 August, past Picnic Day, but takes
/// the 7 August due date of the first contribution it overlaps (s 18C(2) item 4).
#[test]
fn example_6_a_bunched_payday_takes_the_later_due_date() {
    assert_eq!(due(payday("2026-07-23")), "2026-08-04");
    let first = received(first_contribution("2026-07-09"), "2026-08-05");
    let results = assessed(json!([first, payday("2026-07-23")]), "2026-08-05");
    assert_eq!(results[0]["deadline"]["due"], "2026-08-07");
    assert_eq!(results[1]["deadline"]["due"], "2026-08-07");
    assert_eq!(results[1]["deadline"]["pathway"], "ITEM4_ALIGNED");
}
