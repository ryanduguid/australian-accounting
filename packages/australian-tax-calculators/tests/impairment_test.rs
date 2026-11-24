// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The impairment test (AASB 136 paras 18, 30 to 33, 55, 59, 104 and 105),
//! read on 24 November 2026. E2 is Illustrative Example 2 (IE23 to IE32); I2
//! to I6 are made-up figures worked in Python's `decimal`
//! (docs/calculation-evidence.md#impairment-test).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "impairment-test";

fn run(input: &Value) -> Value {
    api::evaluate(CMD, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn asset(label: &str, carrying: &str, goodwill: bool) -> Value {
    json!({"label": label, "carrying_amount": carrying, "goodwill": goodwill})
}

fn floored(label: &str, carrying: &str, floor: &str) -> Value {
    json!({"label": label, "carrying_amount": carrying, "goodwill": false, "floor": floor})
}

fn input(flows: Value, rate: &str, assets: Value) -> Value {
    json!({"cash_flows": flows, "discount_rate": rate, "assets": assets, "revalued": false,
           "non_controlling_interest": false, "year": "2025-26", "scope_confirmed": true})
}

fn e2() -> Value {
    input(
        json!([
            "230", "253", "273", "290", "304", "313", "307", "289", "245", "184", "61"
        ]),
        "0.15",
        json!([
            asset("Goodwill", "1000", true),
            asset("Identifiable assets", "1833", false)
        ]),
    )
}

fn amount(r: &Value, key: &str) -> String {
    r["amounts"][key].as_str().unwrap().to_string()
}

fn losses(r: &Value) -> Vec<String> {
    r["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["impairment_loss"].as_str().unwrap().to_string())
        .collect()
}

/// A cents string rounded half up to whole units.
fn whole(cents: &str) -> i64 {
    let (units, part) = cents.split_once('.').unwrap();
    let units: i64 = units.parse().unwrap();
    units + i64::from(part >= "50")
}

#[test]
fn e2_illustrative_example_2() {
    let r = run(&e2());
    let years: Vec<i64> = r["years"]
        .as_array()
        .unwrap()
        .iter()
        .map(|y| whole(y["present_value"].as_str().unwrap()))
        .collect();
    assert_eq!(years, [200, 191, 180, 166, 151, 135, 115, 94, 70, 45, 13]);
    assert_eq!(amount(&r, "value_in_use"), "1361.20");
    assert_eq!(amount(&r, "recoverable_amount"), "1361.20");
    assert_eq!(amount(&r, "carrying_amount"), "2833.00");
    assert_eq!(amount(&r, "impairment_loss"), "1471.80");
    assert_eq!(amount(&r, "goodwill_impairment"), "1000.00");
    assert_eq!(amount(&r, "other_assets_impairment"), "471.80");
    assert_eq!(amount(&r, "unallocated"), "0.00");
    assert_eq!(losses(&r), ["1000.00", "471.80"]);
    assert_eq!(r["assets"][1]["carrying_amount_after"], "1361.20");
    assert!(
        r["amounts"]
            .get("fair_value_less_costs_of_disposal")
            .is_none()
    );
}

#[test]
fn i2_the_floor_passes_the_excess_on() {
    let mut i = input(
        json!(["100"]),
        "0.1",
        json!([
            asset("Goodwill", "200", true),
            floored("Plant", "600", "550"),
            asset("Building", "400", false)
        ]),
    );
    i["fair_value_less_costs_of_disposal"] = json!("700");
    let r = run(&i);
    assert_eq!(amount(&r, "recoverable_amount"), "700.00");
    assert_eq!(amount(&r, "impairment_loss"), "500.00");
    assert_eq!(losses(&r), ["200.00", "50.00", "250.00"]);
    assert_eq!(r["assets"][1]["carrying_amount_after"], "550.00");
}

#[test]
fn i3_a_loss_left_once_every_asset_is_at_its_floor() {
    let i = input(
        json!(["0"]),
        "0.1",
        json!([
            asset("Goodwill", "100", true),
            floored("A", "500", "450"),
            floored("B", "300", "280")
        ]),
    );
    let r = run(&i);
    assert_eq!(amount(&r, "impairment_loss"), "900.00");
    assert_eq!(losses(&r), ["100.00", "50.00", "20.00"]);
    assert_eq!(amount(&r, "unallocated"), "730.00");
    assert_eq!(amount(&r, "other_assets_impairment"), "70.00");
}

#[test]
fn i4_not_impaired() {
    let mut i = e2();
    i["fair_value_less_costs_of_disposal"] = json!("2833.00");
    let r = run(&i);
    assert_eq!(amount(&r, "recoverable_amount"), "2833.00");
    assert_eq!(amount(&r, "impairment_loss"), "0.00");
    assert_eq!(losses(&r), ["0.00", "0.00"]);
    i["fair_value_less_costs_of_disposal"] = json!("1000.00");
    assert_eq!(amount(&run(&i), "recoverable_amount"), "1361.20");
}

#[test]
fn i5_a_terminal_value() {
    let mut i = input(
        json!(["12000.00", "12500.00", "13000.00", "13500.00", "14000.00"]),
        "0.0925",
        json!([asset("Asset", "80000.00", false)]),
    );
    i["terminal_value"] = json!("40000.00");
    let r = run(&i);
    assert_eq!(amount(&r, "value_in_use"), "75599.58");
    assert_eq!(amount(&r, "impairment_loss"), "4400.42");
    assert_eq!(losses(&r), ["4400.42"]);
}

#[test]
fn i6_cumulative_rounding() {
    let i = input(
        json!(["230"]),
        "0.15",
        json!([
            asset("A", "100.00", false),
            asset("B", "100.00", false),
            asset("C", "100.00", false)
        ]),
    );
    let r = run(&i);
    assert_eq!(amount(&r, "impairment_loss"), "100.00");
    assert_eq!(losses(&r), ["33.33", "33.34", "33.33"]);
}

#[test]
fn negative_flows_and_a_floor_above_carrying_amount() {
    let i = input(
        json!(["-100.00", "300.00"]),
        "0.1",
        json!([
            floored("A", "100.00", "150.00"),
            asset("B", "200.00", false)
        ]),
    );
    let r = run(&i);
    assert_eq!(amount(&r, "value_in_use"), "157.02");
    assert_eq!(losses(&r), ["0.00", "142.98"]);
    let mut i = input(
        json!(["-500.00"]),
        "0.1",
        json!([asset("A", "100.00", false)]),
    );
    i["terminal_value"] = json!("-10.00");
    let r = run(&i);
    assert_eq!(amount(&r, "value_in_use"), "-463.64");
    assert_eq!(losses(&r), ["100.00"]);
    assert_eq!(amount(&r, "unallocated"), "463.64");
}

#[test]
fn refusals() {
    let mut i = e2();
    i["revalued"] = json!(true);
    assert_eq!(code(&i), "impairment_test.revalued");
    let mut i = e2();
    i["non_controlling_interest"] = json!(true);
    assert_eq!(code(&i), "impairment_test.non_controlling_interest");
    let mut i = e2();
    i["revalued"] = json!("no");
    assert_eq!(code(&i), "input.flag_invalid");
    for flows in [
        json!([]),
        json!(vec!["1"; 51]),
        json!(["1.001"]),
        json!([5]),
        json!("230"),
    ] {
        let mut i = e2();
        i["cash_flows"] = flows.clone();
        assert_eq!(code(&i), "impairment_test.cash_flows_invalid", "{flows}");
    }
    let mut i = e2();
    i["cash_flows"] = json!(vec!["1"; 50]);
    assert!(api::evaluate(CMD, &i).is_ok());
    for rate in ["0", "-0.1", "1.01", "0.1234567"] {
        let mut i = e2();
        i["discount_rate"] = json!(rate);
        assert_eq!(code(&i), "impairment_test.rate_invalid", "{rate}");
    }
    let mut i = e2();
    i["discount_rate"] = json!("1");
    assert!(api::evaluate(CMD, &i).is_ok());
    let mut i = e2();
    i["terminal_value"] = json!("1.001");
    assert_eq!(code(&i), "input.signed_amount_invalid");
    let mut i = e2();
    i["fair_value_less_costs_of_disposal"] = json!("-1.00");
    assert_eq!(code(&i), "input.amount_invalid");
}

#[test]
fn assets() {
    for assets in [
        json!([]),
        json!([asset("G1", "1", true), asset("G2", "1", true)]),
        json!([{"label": "G", "carrying_amount": "1", "goodwill": true, "floor": "0"}]),
        json!([{"label": "A", "carrying_amount": "1", "goodwill": false, "class": "x"}]),
        json!([{"label": "A", "carrying_amount": "1"}]),
        json!([{"label": " ", "carrying_amount": "1", "goodwill": false}]),
        json!([asset("A", "-1.00", false)]),
        json!([floored("A", "1.00", "-1.00")]),
        json!([asset("A", "0.00", false), asset("B", "0", false)]),
        json!(vec![asset("A", "1", false); 201]),
        json!("not a list"),
    ] {
        let mut i = e2();
        i["assets"] = assets.clone();
        assert_eq!(code(&i), "impairment_test.asset_invalid", "{assets}");
    }
}

#[test]
fn years_and_scope() {
    for year in ["2020-21", "2026-27"] {
        let mut i = e2();
        i["year"] = json!(year);
        assert!(api::evaluate(CMD, &i).is_ok(), "{year}");
    }
    let mut i = e2();
    i["year"] = json!("2019-20");
    assert_eq!(code(&i), "period.unsupported");
    i["year"] = json!("2027-28");
    assert_eq!(code(&i), "period.unsupported");
    let mut i = e2();
    i["scope_confirmed"] = json!(false);
    assert_eq!(code(&i), "scope.not_confirmed");
    let mut i = e2();
    i["not_an_input"] = json!(1);
    assert_eq!(code(&i), "input.field_unknown");
}
