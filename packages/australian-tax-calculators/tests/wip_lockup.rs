// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! WIP ageing and lock-up days, mirroring Ozzit's `oz.WIPDaysλ` and
//! `oz.ReceivableDaysλ` (function reference v3.7.0, read on 24 November 2026):
//! O1 to O4 are its published examples, W1 a made-up ageing worked in
//! Python's `decimal` (docs/calculation-evidence.md#wip-lock-up).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "wip-lockup";

fn run(input: &Value) -> Value {
    api::evaluate(CMD, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn entry(date: &str, amount: &str) -> Value {
    json!({"label": "x", "date": date, "amount": amount})
}

fn input(entries: Value, fees: &str, receivables: &str, billings: &str) -> Value {
    json!({"entries": entries, "as_at": "2026-06-30", "fee_revenue": fees,
           "receivables": receivables, "billings": billings, "year": "2025-26",
           "scope_confirmed": true})
}

/// One WIP balance of `wip`, dated at `as_at`.
fn ozzit(wip: &str, fees: &str, receivables: &str, billings: &str) -> Value {
    input(
        json!([entry("2026-06-30", wip)]),
        fees,
        receivables,
        billings,
    )
}

fn w1() -> Value {
    input(
        json!([
            entry("2026-06-30", "5000.00"),
            entry("2026-05-31", "7500.00"),
            entry("2026-05-30", "2500.00"),
            entry("2026-04-01", "4000.00"),
            entry("2026-03-31", "3000.00"),
            entry("2025-12-01", "1250.50")
        ]),
        "300000.00",
        "60000.00",
        "320000.00",
    )
}

#[test]
fn the_ozzit_examples() {
    // =oz.WIPDaysλ(42000, 510000) and =oz.ReceivableDaysλ(125000, 1200000).
    let r = run(&ozzit("42000", "510000", "125000", "1200000"));
    assert_eq!(r["rates"]["wip_days"], "30.0588");
    assert_eq!(r["rates"]["receivable_days"], "38.0208");
    assert_eq!(r["rates"]["lock_up_days"], "68.0797");
    assert_eq!(r["amounts"]["wip_days"], "30.06");
    assert_eq!(r["amounts"]["receivable_days"], "38.02");
    assert_eq!(r["amounts"]["lock_up_days"], "68.08");
    // =oz.ReceivableDaysλ(125000, 330000, 90).
    let mut i = ozzit("42000", "510000", "125000", "330000");
    i["days"] = json!(90);
    assert_eq!(run(&i)["rates"]["receivable_days"], "34.0909");
}

#[test]
fn w1_ageing_at_each_bucket_edge() {
    let r = run(&w1());
    let got: Vec<_> = r["ageing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["from_days"].clone(),
                b["to_days"].clone(),
                b["amount"].clone(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            (json!(0), json!(30), json!("12500.00")),
            (json!(31), json!(60), json!("2500.00")),
            (json!(61), json!(90), json!("4000.00")),
            (json!(91), Value::Null, json!("4250.50")),
        ]
    );
    assert_eq!(r["amounts"]["wip"], "23250.50");
    assert_eq!(r["amounts"]["wip_days"], "28.29");
    assert_eq!(r["amounts"]["receivable_days"], "68.44");
    assert_eq!(r["amounts"]["lock_up_days"], "96.73");
}

#[test]
fn stated_buckets() {
    let mut i = w1();
    i["bucket_days"] = json!([7, 100]);
    let r = run(&i);
    let amounts: Vec<_> = r["ageing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["amount"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(amounts, ["5000.00", "17000.00", "1250.50"]);
    assert_eq!(r["ageing"][1]["from_days"], 8);
    for bad in [
        json!([]),
        json!([0, 30]),
        json!([30, 30]),
        json!([60, 30]),
        json!([1, 2, 3, 4, 5, 6, 7]),
        json!([3651]),
        json!([30.5]),
        json!("30"),
    ] {
        let mut i = w1();
        i["bucket_days"] = bad.clone();
        assert_eq!(code(&i), "wip_lockup.buckets_invalid", "{bad}");
    }
    let mut i = w1();
    i["bucket_days"] = json!([1, 2, 3, 4, 5, 3650]);
    assert!(api::evaluate(CMD, &i).is_ok());
}

#[test]
fn days_and_flows() {
    for (days, ok) in [
        (json!(1), true),
        (json!(3660), true),
        (json!(0), false),
        (json!(3661), false),
        (json!(-5), false),
        (json!("365"), false),
    ] {
        let mut i = w1();
        i["days"] = days.clone();
        if ok {
            assert!(api::evaluate(CMD, &i).is_ok(), "{days}");
        } else {
            assert_eq!(code(&i), "wip_lockup.days_invalid", "{days}");
        }
    }
    let mut i = w1();
    i["fee_revenue"] = json!("0.00");
    assert_eq!(code(&i), "wip_lockup.flow_nil");
    let mut i = w1();
    i["billings"] = json!("0");
    assert_eq!(code(&i), "wip_lockup.flow_nil");
    let mut i = w1();
    i["receivables"] = json!("0.00");
    assert_eq!(run(&i)["amounts"]["receivable_days"], "0.00");
    i["receivables"] = json!("-1.00");
    assert_eq!(code(&i), "input.amount_invalid");
}

#[test]
fn entries() {
    for bad in [
        json!([]),
        json!([entry("2026-07-01", "1.00")]),
        json!([entry("2026-06-30", "-1.00")]),
        json!([entry("2026-06-30", "1.001")]),
        json!([{"label": " ", "date": "2026-06-30", "amount": "1.00"}]),
        json!([{"label": "x", "date": "2026-06-30", "amount": "1.00", "client": "y"}]),
        json!([{"label": "x", "date": "30/06/2026", "amount": "1.00"}]),
        json!("not a list"),
    ] {
        let mut i = w1();
        i["entries"] = bad.clone();
        assert_eq!(code(&i), "wip_lockup.entry_invalid", "{bad}");
    }
    let mut i = w1();
    i["entries"] = json!(vec![entry("2026-06-30", "1.00"); 5001]);
    assert_eq!(code(&i), "wip_lockup.entry_invalid");
    i["entries"] = json!(vec![entry("2026-06-30", "1.00"); 5000]);
    assert_eq!(run(&i)["amounts"]["wip"], "5000.00");
}

#[test]
fn years_and_scope() {
    for (year, as_at) in [("2020-21", "2020-07-01"), ("2026-27", "2027-06-30")] {
        let mut i = w1();
        i["year"] = json!(year);
        i["as_at"] = json!(as_at);
        i["entries"] = json!([entry(as_at, "1.00")]);
        assert!(api::evaluate(CMD, &i).is_ok(), "{year}");
    }
    let mut i = w1();
    i["as_at"] = json!("2026-07-01");
    assert_eq!(code(&i), "wip_lockup.as_at_outside_year");
    i["as_at"] = json!("2025-06-30");
    assert_eq!(code(&i), "wip_lockup.as_at_outside_year");
    let mut i = w1();
    i["year"] = json!("2019-20");
    assert_eq!(code(&i), "period.unsupported");
    i["year"] = json!("2027-28");
    assert_eq!(code(&i), "period.unsupported");
    let mut i = w1();
    i["scope_confirmed"] = json!(false);
    assert_eq!(code(&i), "scope.not_confirmed");
    let mut i = w1();
    i["not_an_input"] = json!(1);
    assert_eq!(code(&i), "input.field_unknown");
}

/// Each bucket's amount, in order.
fn bucket_amounts(r: &Value) -> Vec<String> {
    r["ageing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["amount"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn ages_60_and_61_and_a_leap_year() {
    // 1 May to 30 Jun 2026 is 60 days; 30 Apr is 61.
    let mut i = w1();
    i["entries"] = json!([entry("2026-05-01", "1.00"), entry("2026-04-30", "2.00")]);
    assert_eq!(bucket_amounts(&run(&i)), ["0.00", "1.00", "2.00", "0.00"]);
    // 31 Dec 2023 to 29 Feb 2024 is 60 days (31 + 29); 30 Dec 2023 is 61.
    i["year"] = json!("2023-24");
    i["as_at"] = json!("2024-02-29");
    i["entries"] = json!([entry("2023-12-31", "1.00"), entry("2023-12-30", "2.00")]);
    assert_eq!(bucket_amounts(&run(&i)), ["0.00", "1.00", "2.00", "0.00"]);
    for (as_at, ok) in [
        ("2023-07-01", true),
        ("2024-06-30", true),
        ("2023-06-30", false),
        ("2024-07-01", false),
    ] {
        i["as_at"] = json!(as_at);
        i["entries"] = json!([entry("2023-06-30", "1.00")]);
        if ok {
            assert!(api::evaluate(CMD, &i).is_ok(), "{as_at}");
        } else {
            assert_eq!(code(&i), "wip_lockup.as_at_outside_year", "{as_at}");
        }
    }
    for bad in ["2026-02-30", "2026-06-31", "30/06/2026", ""] {
        let mut i = w1();
        i["as_at"] = json!(bad);
        assert_eq!(code(&i), "input.date_invalid", "{bad}");
    }
}

#[test]
fn old_entries_single_buckets_and_the_money_limit() {
    // 30 Jun 2016 is 3,652 days before 30 Jun 2026: entries have no age limit.
    let mut i = w1();
    i["entries"] = json!([entry("2016-06-30", "1.00"), entry("2026-06-29", "2.00")]);
    i["bucket_days"] = json!([3650]);
    assert_eq!(bucket_amounts(&run(&i)), ["2.00", "1.00"]);
    i["bucket_days"] = json!([1]);
    let r = run(&i);
    assert_eq!(bucket_amounts(&r), ["2.00", "1.00"]);
    assert_eq!(r["ageing"][1]["from_days"], 2);
    let mut i = w1();
    i["entries"] = json!([entry("2026-06-30", "1000000000000.00")]);
    assert_eq!(run(&i)["amounts"]["wip"], "1000000000000.00");
    i["entries"] = json!([entry("2026-06-30", "1000000000000.01")]);
    assert_eq!(code(&i), "wip_lockup.entry_invalid");
    for key in ["fee_revenue", "receivables", "billings"] {
        let mut i = w1();
        i[key] = json!("1000000000000.00");
        assert!(api::evaluate(CMD, &i).is_ok(), "{key}");
        i[key] = json!("1000000000000.01");
        assert_eq!(code(&i), "input.amount_invalid", "{key}");
    }
}

#[test]
fn lock_up_rounds_the_unrounded_sum() {
    // Each of 0.04 / 100,000 x 365 = 0.000146 rounds to 0.0001; their
    // unrounded sum, 0.000292, rounds to 0.0003, not 0.0002.
    let r = run(&ozzit("0.04", "100000.00", "0.04", "100000.00"));
    assert_eq!(r["rates"]["wip_days"], "0.0001");
    assert_eq!(r["rates"]["receivable_days"], "0.0001");
    assert_eq!(r["rates"]["lock_up_days"], "0.0003");
}
