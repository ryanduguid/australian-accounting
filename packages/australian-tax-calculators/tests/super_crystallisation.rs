// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Crystallisation of a taxed superannuation interest at 30 June 2007 (ITTPA
//! 1997 s 307-225 and former ITAA 1936 s 27AA(1)(d), read in the February
//! 2007 compilation) against the day count of the ATO's 'Calculating
//! components of a super benefit' example 9 and hand-worked cases of the
//! formula, as the ATO publishes no numeric crystallisation example
//! (docs/calculation-evidence.md#super-crystallisation).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("super-crystallisation", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("super-crystallisation", input)
        .unwrap_err()
        .code
}

fn interest(start: &str, value: &str, parts: [&str; 4]) -> Value {
    let [concessional, invalidity, cgt, undeducted] = parts;
    json!({
        "year": "2006-07", "scope_confirmed": true, "eligible_service_start": start,
        "interest_value": value, "concessional_component": concessional,
        "post_june_1994_invalidity_component": invalidity, "cgt_exempt_component": cgt,
        "undeducted_contributions": undeducted, "untaxed_source": false,
        "income_stream_before_july_2007": false,
    })
}

const NIL: [&str; 4] = ["0.00", "0.00", "0.00", "0.00"];

fn with(mut input: Value, key: &str, value: Value) -> Value {
    input[key] = value;
    input
}

fn amount(out: &Value, key: &str) -> String {
    out["amounts"][key].as_str().unwrap().to_owned()
}

#[test]
fn service_from_1_july_1977() {
    // As in example 9, 2,191 days fall before 1 July 1983; here the period
    // ends on 30 June 2007: $230,000 x 2,191 / 10,957 = $45,991.60.
    let out = run(&interest("1977-07-01", "230000.00", NIL));
    assert_eq!(out["rates"]["pre_july_1983_days"], "2191");
    assert_eq!(out["rates"]["service_days"], "10957");
    assert_eq!(amount(&out, "pre_july_83_component"), "45991.60");
    assert_eq!(amount(&out, "crystallised_segment"), "45991.60");
    assert_eq!(amount(&out, "taxable_component"), "184008.40");
}

#[test]
fn the_lesser_amount_after_undeducted_contributions() {
    let out = run(&interest(
        "1977-07-01",
        "230000.00",
        ["0.00", "0.00", "0.00", "200000.00"],
    ));
    assert_eq!(amount(&out, "pre_july_83_component"), "30000.00");
    assert_eq!(amount(&out, "crystallised_segment"), "230000.00");
    assert_eq!(amount(&out, "taxable_component"), "0.00");
}

#[test]
fn every_component_and_service_from_1980() {
    // Base $465,000; 1,095 of 9,861 days.
    let out = run(&interest(
        "1980-07-01",
        "500000.00",
        ["10000.00", "5000.00", "20000.00", "50000.00"],
    ));
    assert_eq!(amount(&out, "pre_july_83_base"), "465000.00");
    assert_eq!(amount(&out, "pre_july_83_component"), "51635.23");
    assert_eq!(amount(&out, "crystallised_segment"), "136635.23");
    assert_eq!(amount(&out, "taxable_component"), "363364.77");
}

#[test]
fn no_service_before_1_july_1983() {
    for start in ["1983-07-01", "1990-07-01", "2007-06-30"] {
        let out = run(&interest(
            start,
            "50000.00",
            ["1000.00", "0.00", "0.00", "5000.00"],
        ));
        assert_eq!(out["rates"]["pre_july_1983_days"], "0", "{start}");
        assert_eq!(amount(&out, "pre_july_83_component"), "0.00", "{start}");
        assert_eq!(amount(&out, "crystallised_segment"), "6000.00", "{start}");
        assert_eq!(amount(&out, "taxable_component"), "44000.00", "{start}");
    }
    let one_day = run(&interest("2007-06-30", "1.00", NIL));
    assert_eq!(one_day["rates"]["service_days"], "1");
    // The last day before 1 July 1983: 1 of 8,767 days.
    let edge = run(&interest("1983-06-30", "8767.00", NIL));
    assert_eq!(edge["rates"]["pre_july_1983_days"], "1");
    assert_eq!(amount(&edge, "pre_july_83_component"), "1.00");
}

#[test]
fn refusals() {
    let ok = interest("1980-07-01", "1000.00", NIL);
    let cases = [
        (
            with(ok.clone(), "untaxed_source", json!(true)),
            "super_crystallisation.untaxed_source",
        ),
        (
            with(ok.clone(), "income_stream_before_july_2007", json!(true)),
            "super_crystallisation.income_stream_before_july_2007",
        ),
        (
            with(ok.clone(), "eligible_service_start", json!("2007-07-01")),
            "super_crystallisation.dates_invalid",
        ),
        (
            with(ok.clone(), "undeducted_contributions", json!("1000.01")),
            "super_crystallisation.components_exceed_value",
        ),
        (
            with(ok.clone(), "income_stream_before_july_2007", json!("no")),
            "input.flag_invalid",
        ),
        (
            with(ok.clone(), "year", json!("2007-08")),
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
