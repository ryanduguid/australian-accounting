// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The leave provisions worksheet (AASB 119 and AASB 101 para 69), read on
//! 22 November 2026. AASB 119 has no numeric example for annual or long
//! service leave: V1 to V6 are made-up figures worked independently, and
//! P68 applies the para 68 illustrative example
//! (docs/calculation-evidence.md#leave-provisions).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "leave-provisions";
const YEARS: [&str; 7] = [
    "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
];

/// The plain result, checked against `derive` byte for byte.
fn run(input: &Value) -> Value {
    let result = api::evaluate(CMD, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": CMD, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

fn input(lines: Value) -> Value {
    json!({"lines": lines, "defined_benefit_plan": false, "plan_assets": false,
           "year": "2025-26", "scope_confirmed": true})
}

fn short(leave: &str, hours: &str, rate: &str, p: &str) -> Value {
    json!({"label": "x", "leave": leave, "hours": hours, "hourly_rate": rate,
           "probability": p, "settled_within_12_months": true})
}

/// A long-term line: `a` is [hours, rate, growth, discount, probability].
fn long(leave: &str, a: [&str; 5], years: u64, can_defer: bool) -> Value {
    json!({"label": "x", "leave": leave, "hours": a[0], "hourly_rate": a[1],
           "probability": a[4], "settled_within_12_months": false,
           "years_to_settlement": years, "salary_growth": a[2], "discount_rate": a[3],
           "can_defer_settlement": can_defer})
}

fn v1() -> Value {
    short("annual", "152.00", "45.50", "1")
}
fn v2() -> Value {
    long(
        "long_service",
        ["400.00", "52.00", "0.03", "0.045", "1"],
        2,
        false,
    )
}
fn v3() -> Value {
    long(
        "long_service",
        ["120.00", "38.00", "0.035", "0.0425", "0.4"],
        5,
        true,
    )
}
fn v4() -> Value {
    long("annual", ["300.00", "40.00", "0.03", "0.04", "1"], 1, false)
}
fn v6() -> Value {
    short("annual", "80.00", "30.00", "0.85")
}

fn line(r: &Value, n: usize, key: &str) -> String {
    r["lines"][n][key].as_str().unwrap().to_string()
}

/// Each made-up vector alone: paras 11 and 16 undiscounted, paras 155 and
/// 83 to 87 grown and discounted, AASB 101 para 69 for the class.
#[test]
fn each_vector_alone() {
    let cases = [
        (v1(), "6916.00", "current"),
        (v2(), "20207.16", "current"),
        (v3(), "1759.33", "non_current"),
        (v4(), "11884.62", "current"),
        (v6(), "2040.00", "current"),
    ];
    for (l, want, class) in cases {
        let r = run(&input(json!([l])));
        assert_eq!(line(&r, 0, "provision"), want);
        assert_eq!(line(&r, 0, "classification"), class);
        assert_eq!(amount(&r, "total_provision"), want);
    }
    let r = run(&input(json!([v2()])));
    assert_eq!(line(&r, 0, "value_at_current_rate"), "20800.00");
    assert_eq!(line(&r, 0, "projected_value"), "22066.72");
    assert_eq!(line(&r, 0, "category"), "other_long_term");
    let r = run(&input(json!([v3()])));
    assert_eq!(line(&r, 0, "projected_value"), "5415.85");
    assert!(
        run(&input(json!([v1()])))["lines"][0]
            .get("projected_value")
            .is_none()
    );
}

/// All five together: the totals by kind and the current split.
#[test]
fn totals_and_the_current_split() {
    let r = run(&input(json!([v1(), v2(), v3(), v4(), v6()])));
    for (key, want) in [
        ("annual_leave_provision", "20840.62"),
        ("long_service_leave_provision", "21966.49"),
        ("total_provision", "42807.11"),
        ("current_provision", "41047.78"),
        ("non_current_provision", "1759.33"),
    ] {
        assert_eq!(amount(&r, key), want, "{key}");
    }
    assert_eq!(r["period"], "2025-26");
}

/// Para 68's example (1% of final salary a year, 7% growth, 10% discount):
/// its obligations of 89, 196, 324 and 476 are these with the cents dropped.
#[test]
fn the_para_68_example() {
    let cases = [
        ("1", "100.00", 4, "89.53"),
        ("2", "107.00", 3, "196.96"),
        ("3", "114.49", 2, "324.99"),
        ("4", "122.50", 1, "476.64"),
    ];
    for (hours, rate, years, want) in cases {
        let l = long(
            "long_service",
            [hours, rate, "0.07", "0.10", "1"],
            years,
            true,
        );
        let r = run(&input(json!([l])));
        assert_eq!(amount(&r, "total_provision"), want);
        assert_eq!(amount(&r, "non_current_provision"), want);
    }
}

#[test]
fn every_supported_year_and_the_boundaries() {
    for year in YEARS {
        let mut i = input(json!([v1()]));
        i["year"] = json!(year);
        assert_eq!(amount(&run(&i), "total_provision"), "6916.00");
    }
    for year in ["2019-20", "2027-28"] {
        let mut i = input(json!([v1()]));
        i["year"] = json!(year);
        assert_eq!(code(&i), "period.unsupported");
    }
    let mut i = input(json!([v1()]));
    i["scope_confirmed"] = json!(false);
    assert_eq!(code(&i), "scope.not_confirmed");
}

#[test]
fn refusals() {
    let mut i = input(json!([v1()]));
    i["defined_benefit_plan"] = json!(true);
    assert_eq!(code(&i), "leave_provisions.defined_benefit_plan");
    let mut i = input(json!([v1()]));
    i["plan_assets"] = json!(true);
    assert_eq!(code(&i), "leave_provisions.plan_assets");
    let unsure = long("long_service", ["1.00", "1.00", "0", "0", "0.9"], 1, false);
    assert_eq!(
        code(&input(json!([unsure]))),
        "leave_provisions.fact_conflict"
    );
    let mut i = input(json!([v1()]));
    i["extra"] = json!(1);
    assert_eq!(code(&i), "input.field_unknown");
}

#[test]
fn invalid_lines() {
    let mut cases = vec![json!([]), json!("x"), json!([1])];
    let edits: [(&str, Value); 9] = [
        ("leave", json!("sick")),
        ("probability", json!("1.01")),
        ("probability", json!("0.0000001")),
        ("discount_rate", json!("-0.01")),
        ("salary_growth", json!("1.5")),
        ("years_to_settlement", json!(-1)),
        ("years_to_settlement", json!(51)),
        ("hours", json!("-1")),
        ("label", json!("")),
    ];
    for (key, value) in edits {
        let mut l = v3();
        l[key] = value;
        cases.push(json!([l]));
    }
    let mut extra = v1();
    extra["discount_rate"] = json!("0.04");
    cases.push(json!([extra]));
    let mut missing = v3();
    missing.as_object_mut().unwrap().remove("discount_rate");
    cases.push(json!([missing]));
    let mut unknown = v1();
    unknown["on_costs"] = json!("0.1");
    cases.push(json!([unknown]));
    for lines in cases {
        assert_eq!(
            code(&input(lines.clone())),
            "leave_provisions.line_invalid",
            "{lines}"
        );
    }
}

/// The boundaries of each assumption are accepted: 0 and 1, 0 and 50 years.
/// A long-term portion expected within 6 months rounds to 0 years: grown and
/// discounted for no year, it is undiscounted (fresh-context finding 1).
#[test]
fn a_long_term_portion_due_within_months_takes_0_years() {
    let l = long(
        "long_service",
        ["100.00", "50.00", "0.03", "0.05", "1"],
        0,
        false,
    );
    let r = run(&input(json!([l])));
    assert_eq!(amount(&r, "total_provision"), "5000.00");
    assert_eq!(amount(&r, "current_provision"), "5000.00");
    assert_eq!(line(&r, 0, "projected_value"), "5000.00");
}

#[test]
fn assumption_boundaries() {
    let l = long("long_service", ["10.00", "10.00", "0", "0", "0"], 50, true);
    assert_eq!(amount(&run(&input(json!([l]))), "total_provision"), "0.00");
    let l = long("long_service", ["10.00", "10.00", "1", "1", "1"], 1, true);
    assert_eq!(
        amount(&run(&input(json!([l]))), "total_provision"),
        "100.00"
    );
    let l = long("annual", ["10.00", "10.00", "0", "0", "1"], 1, false);
    assert_eq!(
        amount(&run(&input(json!([l]))), "current_provision"),
        "100.00"
    );
}
