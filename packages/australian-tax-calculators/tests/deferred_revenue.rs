// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The deferred revenue schedule (AASB 15 paras 39, 40, 105 to 108, B14, B15
//! and B18), read on 24 November 2026. AASB 15 has no numeric example of a
//! straight-line schedule: D1 to D5 are made-up figures worked independently
//! in Python's `decimal` (docs/calculation-evidence.md#deferred-revenue).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "deferred-revenue";

fn run(input: &Value) -> Value {
    api::evaluate(CMD, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn input(amount: &str, start: &str, end: &str, pattern: &str, ends: Value, paid: Value) -> Value {
    json!({"contract_amount": amount, "start": start, "end": end, "pattern": pattern,
           "period_ends": ends, "billed": paid, "received": paid, "variable_consideration": false,
           "significant_financing_component": false, "estimate_changed": false,
           "year": "2025-26", "scope_confirmed": true})
}

fn paid(date: &str, amount: &str) -> Value {
    json!({"date": date, "amount": amount})
}

fn d1() -> Value {
    input(
        "12000.00",
        "2025-07-01",
        "2026-06-30",
        "daily",
        json!(["2025-09-30", "2025-12-31", "2026-03-31", "2026-06-30"]),
        json!([paid("2025-07-01", "12000.00")]),
    )
}

/// Each period's (revenue to date, contract liability, contract asset), where
/// billings and receipts match, so there is no receivable.
fn rows(r: &Value) -> Vec<(String, String, String)> {
    let s = |p: &Value, k: &str| p[k].as_str().unwrap().to_string();
    r["periods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                s(p, "revenue_to_date"),
                s(p, "contract_liability"),
                s(p, "contract_asset"),
            )
        })
        .collect()
}

fn expect(r: &Value, want: &[(&str, &str, &str)]) {
    let want: Vec<_> = want
        .iter()
        .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
        .collect();
    assert_eq!(rows(r), want);
}

#[test]
fn d1_daily_straight_line_paid_in_advance() {
    let r = run(&d1());
    expect(
        &r,
        &[
            ("3024.66", "8975.34", "0.00"),
            ("6049.32", "5950.68", "0.00"),
            ("9008.22", "2991.78", "0.00"),
            ("12000.00", "0.00", "0.00"),
        ],
    );
    let p = &r["periods"][1];
    assert_eq!(p["revenue_in_period"], "3024.66");
    assert_eq!(p["period_end"], "2025-12-31");
    assert_eq!(r["amounts"]["revenue_recognised"], "12000.00");
    assert_eq!(r["amounts"]["revenue_not_yet_recognised"], "0.00");
}

#[test]
fn d2_monthly_with_a_mid_month_period_end() {
    let mut i = input(
        "36000.00",
        "2025-07-01",
        "2028-06-30",
        "monthly",
        json!([
            "2025-12-31",
            "2026-06-30",
            "2026-09-15",
            "2027-12-31",
            "2028-06-30"
        ]),
        json!([
            paid("2025-07-01", "18000.00"),
            paid("2027-07-01", "18000.00")
        ]),
    );
    expect(
        &run(&i),
        &[
            ("6000.00", "12000.00", "0.00"),
            ("12000.00", "6000.00", "0.00"),
            ("14000.00", "4000.00", "0.00"),
            ("30000.00", "6000.00", "0.00"),
            ("36000.00", "0.00", "0.00"),
        ],
    );
    i["start"] = json!("2025-07-02");
    assert_eq!(code(&i), "deferred_revenue.month_boundary");
    i["start"] = json!("2025-07-01");
    i["end"] = json!("2028-06-29");
    assert_eq!(code(&i), "deferred_revenue.month_boundary");
}

fn d3() -> Value {
    let mut i = input(
        "50000.00",
        "2025-08-01",
        "2026-03-31",
        "milestones",
        json!(["2025-09-30", "2025-12-31", "2026-03-31"]),
        json!([
            paid("2025-08-01", "15000.00"),
            paid("2026-01-15", "20000.00")
        ]),
    );
    i["milestones"] = json!([
        {"label": "Design", "date": "2025-09-15", "amount": "10000.00"},
        {"label": "Build", "date": "2025-12-20", "amount": "30000.00"},
        {"label": "Handover", "date": "2026-03-31", "amount": "10000.00"}
    ]);
    i
}

#[test]
fn d3_milestones_with_an_asset_or_receivable() {
    let r = run(&d3());
    expect(
        &r,
        &[
            ("10000.00", "5000.00", "0.00"),
            ("40000.00", "0.00", "25000.00"),
            ("50000.00", "0.00", "15000.00"),
        ],
    );
    assert_eq!(r["amounts"]["contract_asset"], "15000.00");
    assert_eq!(r["amounts"]["receivable"], "0.00");
}

#[test]
fn d4_a_period_end_before_the_start() {
    let mut i = d1();
    i["period_ends"] = json!(["2025-06-30", "2025-07-01"]);
    expect(
        &run(&i),
        &[("0.00", "0.00", "0.00"), ("32.88", "11967.12", "0.00")],
    );
    i["billed"] = json!([paid("2025-06-01", "3000.00")]);
    i["received"] = json!([paid("2025-06-01", "3000.00")]);
    expect(
        &run(&i),
        &[("0.00", "3000.00", "0.00"), ("32.88", "2967.12", "0.00")],
    );
}

#[test]
fn d5_a_part_year_period_and_one_after_the_end() {
    let i = input(
        "1000.00",
        "2025-07-15",
        "2026-01-14",
        "daily",
        json!(["2025-07-31", "2025-12-31", "2026-06-30"]),
        json!([]),
    );
    let r = run(&i);
    expect(
        &r,
        &[
            ("92.39", "0.00", "92.39"),
            ("923.91", "0.00", "923.91"),
            ("1000.00", "0.00", "1000.00"),
        ],
    );
    let in_period: Vec<_> = r["periods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["revenue_in_period"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(in_period, ["92.39", "831.52", "76.09"]);
}

#[test]
fn every_supported_year_and_the_boundaries() {
    for (year, start, end) in [
        ("2020-21", "2020-07-01", "2021-06-30"),
        ("2026-27", "2027-06-30", "2027-06-30"),
    ] {
        let mut i = d1();
        i["year"] = json!(year);
        i["start"] = json!(start);
        i["end"] = json!(end);
        i["billed"] = json!([]);
        i["received"] = json!([]);
        assert!(api::evaluate(CMD, &i).is_ok(), "{year}");
    }
    let mut i = d1();
    i["year"] = json!("2019-20");
    assert_eq!(code(&i), "period.unsupported");
    i["year"] = json!("2027-28");
    assert_eq!(code(&i), "period.unsupported");
    i["year"] = json!("2024-25");
    assert_eq!(code(&i), "deferred_revenue.dates_invalid");
    let mut i = d1();
    i["scope_confirmed"] = json!(false);
    assert_eq!(code(&i), "scope.not_confirmed");
}

#[test]
fn the_performance_period() {
    let mut i = d1();
    i["end"] = json!("2025-06-30");
    assert_eq!(code(&i), "deferred_revenue.dates_invalid");
    i["end"] = json!("2035-07-01");
    assert_eq!(code(&i), "deferred_revenue.dates_invalid");
    i["end"] = json!("2035-06-30");
    i["period_ends"] = json!(["2035-06-30"]);
    expect(&run(&i), &[("12000.00", "0.00", "0.00")]);
    // 120 whole calendar months is the most a monthly schedule takes.
    i["pattern"] = json!("monthly");
    i["period_ends"] = json!(["2025-07-31", "2035-06-29", "2035-06-30"]);
    expect(
        &run(&i),
        &[
            ("100.00", "11900.00", "0.00"),
            ("11900.00", "100.00", "0.00"),
            ("12000.00", "0.00", "0.00"),
        ],
    );
    i["end"] = json!("2035-07-31");
    assert_eq!(code(&i), "deferred_revenue.dates_invalid");
    i["pattern"] = json!("daily");
    i["period_ends"] = json!(["2035-06-30"]);
    i["end"] = json!("2025-07-01");
    expect(&run(&i), &[("12000.00", "0.00", "0.00")]);
    i["end"] = json!("30/06/2026");
    assert_eq!(code(&i), "input.date_invalid");
}

#[test]
fn refusals() {
    let mut i = d1();
    i["variable_consideration"] = json!(true);
    assert_eq!(code(&i), "deferred_revenue.variable_consideration");
    let mut i = d1();
    i["significant_financing_component"] = json!(true);
    assert_eq!(code(&i), "deferred_revenue.financing_component");
    let mut i = d1();
    i["variable_consideration"] = json!("no");
    assert_eq!(code(&i), "input.flag_invalid");
    let mut i = d1();
    i["contract_amount"] = json!("0.00");
    assert_eq!(code(&i), "deferred_revenue.amount_invalid");
    i["contract_amount"] = json!("-1.00");
    assert_eq!(code(&i), "input.amount_invalid");
    let mut i = d1();
    i["not_an_input"] = json!(1);
    assert_eq!(code(&i), "input.field_unknown");
}

#[test]
fn the_pattern_and_milestones() {
    let mut i = d1();
    i["pattern"] = json!("weekly");
    assert_eq!(code(&i), "deferred_revenue.pattern_invalid");
    i["pattern"] = json!("milestones");
    assert_eq!(code(&i), "deferred_revenue.pattern_invalid");
    let mut i = d3();
    i["pattern"] = json!("daily");
    assert_eq!(code(&i), "deferred_revenue.pattern_invalid");
    let bad = [
        json!([]),
        json!([{"label": "x", "date": "2025-09-15", "amount": "50000.00", "extra": 1}]),
        json!([{"label": "x", "date": "2025-07-31", "amount": "50000.00"}]),
        json!([{"label": "x", "date": "2026-04-01", "amount": "50000.00"}]),
        json!([{"label": "x", "date": "2025-09-15", "amount": "49999.99"}]),
        json!([{"label": "x", "date": "2025-09-15", "amount": "0.00"},
               {"label": "y", "date": "2025-09-16", "amount": "50000.00"}]),
        json!("not a list"),
    ];
    for m in bad {
        let mut i = d3();
        i["milestones"] = m.clone();
        assert_eq!(code(&i), "deferred_revenue.milestones_invalid", "{m}");
    }
}

#[test]
fn period_ends_and_consideration() {
    for ends in [
        json!([]),
        json!(["2025-12-31", "2025-12-31"]),
        json!(["2025-12-31", "2025-09-30"]),
        json!(["31/12/2025"]),
        json!(vec!["2025-12-31"; 121]),
    ] {
        let mut i = d1();
        i["period_ends"] = ends.clone();
        assert_eq!(code(&i), "deferred_revenue.period_ends_invalid", "{ends}");
    }
    for key in ["billed", "received"] {
        for given in [
            json!([paid("2025-07-01", "12000.01")]),
            json!([paid("2025-07-01", "6000.00"), paid("2025-08-01", "6000.01")]),
            json!([paid("2025-07-01", "0.00")]),
            json!([paid("2025-07-01", "1.001")]),
            json!([{"date": "2025-07-01", "amount": "1.00", "label": "x"}]),
            json!([{"date": "2025-07-01"}]),
        ] {
            let mut i = d1();
            i[key] = given.clone();
            assert_eq!(
                code(&i),
                "deferred_revenue.consideration_invalid",
                "{key} {given}"
            );
        }
    }
}

/// Each period's (receivable, contract liability, contract asset).
fn balances(r: &Value) -> Vec<[String; 3]> {
    let s = |p: &Value, k: &str| p[k].as_str().unwrap().to_string();
    r["periods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            [
                s(p, "receivable"),
                s(p, "contract_liability"),
                s(p, "contract_asset"),
            ]
        })
        .collect()
}

#[test]
fn d6_d7_a_receivable_beside_the_contract_balance() {
    // $1,200 over 12 days is $100 a day; the first period end is day 1.
    let mut i = input(
        "1200.00",
        "2025-07-01",
        "2025-07-12",
        "daily",
        json!(["2025-07-01", "2025-07-05", "2025-07-12"]),
        json!([]),
    );
    i["billed"] = json!([paid("2025-07-01", "150.00"), paid("2025-07-05", "650.00")]);
    i["received"] = json!([paid("2025-07-05", "300.00"), paid("2025-07-20", "900.00")]);
    assert_eq!(
        balances(&run(&i)),
        [
            ["150.00", "50.00", "0.00"],
            ["500.00", "300.00", "0.00"],
            ["500.00", "0.00", "400.00"]
        ]
    );
    i["billed"] = json!([paid("2025-07-01", "100.00")]);
    i["received"] = json!([]);
    i["period_ends"] = json!(["2025-07-01"]);
    let r = run(&i);
    assert_eq!(balances(&r), [["100.00", "0.00", "0.00"]]);
    assert_eq!(r["periods"][0]["billed_to_date"], "100.00");
    assert_eq!(r["amounts"]["receivable"], "100.00");
}

#[test]
fn an_advance_received_before_any_billing_and_consideration_after_the_last_period_end() {
    let mut i = d1();
    i["billed"] = json!([paid("2026-01-01", "12000.00")]);
    i["received"] = json!([paid("2025-07-01", "3000.00"), paid("2026-07-31", "9000.00")]);
    let r = run(&i);
    assert_eq!(
        balances(&r),
        [
            ["0.00", "0.00", "24.66"],
            ["0.00", "0.00", "3049.32"],
            ["9000.00", "2991.78", "0.00"],
            ["9000.00", "0.00", "0.00"]
        ]
    );
    assert_eq!(r["periods"][3]["received_to_date"], "3000.00");
}

#[test]
fn milestone_labels() {
    for label in [
        json!(null),
        json!(5),
        json!({"x": 1}),
        json!(""),
        json!("x".repeat(201)),
    ] {
        let mut i = d3();
        i["milestones"][0]["label"] = label.clone();
        assert_eq!(code(&i), "deferred_revenue.milestones_invalid", "{label}");
    }
    let mut i = d3();
    i["milestones"][0].as_object_mut().unwrap().remove("label");
    assert_eq!(code(&i), "deferred_revenue.milestones_invalid");
}

#[test]
fn a_changed_estimate_is_refused_rather_than_rewriting_earlier_periods() {
    let mut initial = input(
        "12000.00",
        "2025-07-01",
        "2026-06-30",
        "monthly",
        json!(["2025-09-30"]),
        json!([]),
    );
    initial["billed"] = json!([paid("2025-07-01", "12000.00")]);
    initial["received"] = json!([]);
    let initial_out = run(&initial);
    assert_eq!(initial_out["periods"][0]["revenue_in_period"], "3000.00");
    assert_eq!(initial_out["periods"][0]["revenue_to_date"], "3000.00");

    // Duration revised from 12 to 18 months in December (AASB 15 para 43;
    // AASB 108 paras 36 to 38): prospective recognition takes $4,000
    // cumulative less $3,000 already recognised = $1,000 for the December
    // quarter; rerunning from the start with unchanged estimates reports
    // $2,000 for September and $2,000 for December instead.
    let mut revised = input(
        "12000.00",
        "2025-07-01",
        "2026-12-31",
        "monthly",
        json!(["2025-09-30", "2025-12-31"]),
        json!([]),
    );
    revised["billed"] = json!([paid("2025-07-01", "12000.00")]);
    revised["received"] = json!([]);
    let revised_out = run(&revised);
    assert_eq!(revised_out["periods"][1]["revenue_in_period"], "2000.00");
    assert_eq!(revised_out["periods"][1]["revenue_to_date"], "4000.00");
    let whole_dollars = |v: &Value| {
        v.as_str()
            .unwrap()
            .strip_suffix(".00")
            .unwrap()
            .parse::<i64>()
            .unwrap()
    };
    assert_eq!(
        whole_dollars(&revised_out["periods"][1]["revenue_to_date"])
            - whole_dollars(&initial_out["periods"][0]["revenue_to_date"]),
        1000
    );
    revised["estimate_changed"] = json!(true);
    assert_eq!(code(&revised), "deferred_revenue.estimate_changed");
    revised["estimate_changed"] = json!("yes");
    assert_eq!(code(&revised), "input.flag_invalid");
    revised.as_object_mut().unwrap().remove("estimate_changed");
    assert_eq!(code(&revised), "input.missing_field");

    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let scope = catalogue["deferred_revenue"]["scope"].as_str().unwrap();
    assert!(
        !scope.contains("rerun"),
        "scope still advises rerun: {scope}"
    );
    assert!(scope.contains("AASB 108"), "scope omits AASB 108: {scope}");
}
