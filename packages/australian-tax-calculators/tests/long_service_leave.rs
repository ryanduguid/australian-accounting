// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Long service leave against the site's long service leave calculator
//! (ryanduguid.github.io at dcc7ecbe: its page cases, its unit tests' inputs
//! and a sweep of every place, status, reason and threshold, run through the
//! page's own `longServiceLeave` into `data/long_service_leave_site_cases.json`),
//! and against the regulators' worked examples. Every source was read on
//! 24 November 2026 (docs/calculation-evidence.md#long-service-leave). Every
//! employee here is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn input(place: &str, start: &str, end: &str, year: &str) -> Value {
    json!({
        "jurisdiction": place, "start": start, "end": end, "excluded_days": 0,
        "leave_taken": "0", "status": "employed", "weekly_pay": "1500",
        "year": year, "scope_confirmed": true,
    })
}

fn ends(v: Value, reason: &str) -> Value {
    with(v, json!({"status": "ends", "reason": reason}))
}

fn with(mut v: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value.is_null() {
            v.as_object_mut().unwrap().remove(key);
        } else {
            v[key] = value.clone();
        }
    }
    v
}

fn run(input: &Value) -> Value {
    api::evaluate("long-service-leave", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("long-service-leave", input)
        .unwrap_err()
        .code
}

/// The weeks, balance and value.
fn figures(result: &Value) -> [String; 3] {
    [
        result["rates"]["weeks"].as_str().unwrap().to_string(),
        result["rates"]["balance"].as_str().unwrap().to_string(),
        result["amounts"]["value"].as_str().unwrap().to_string(),
    ]
}

/// The financial year holding an ISO date, or a supported year when the
/// date does not parse.
fn year_of(date: &str) -> String {
    let parts: Vec<i64> = date.split('-').filter_map(|p| p.parse().ok()).collect();
    match parts[..] {
        [y, m, _] if date.len() == 10 => {
            let first = if m >= 7 { y } else { y - 1 };
            format!("{first}-{:02}", (first + 1) % 100)
        }
        _ => "2025-26".to_string(),
    }
}

/// The command's input for one of the site's rows.
fn site_input(row: &Value) -> Value {
    let x = &row["input"];
    let text = |key: &str| x[key].as_str().unwrap();
    let place = text("place").trim_start_matches("AU-").to_lowercase();
    let mut v = input(&place, text("start"), text("end"), &year_of(text("end")));
    let excluded = text("excluded");
    v["excluded_days"] = match excluded.parse::<i64>() {
        Ok(n) => json!(n),
        Err(_) => json!(excluded),
    };
    v["leave_taken"] = json!(text("taken"));
    v["weekly_pay"] = json!(text("pay"));
    if text("mode") == "ends" {
        v = ends(v, &text("reason").replace('-', "_"));
    }
    v
}

/// The site's figures in the command's form.
fn site_figures(site: &Value) -> [String; 3] {
    ["weeks", "balance", "value"].map(|k| site[k].as_str().unwrap().to_string())
}

/// How one row of the site's fixture came out: the same figures, the same
/// refusal, or a counted departure where a source decides otherwise.
fn checked(row: &Value) -> &'static str {
    let v = site_input(row);
    let site = &row["site"];
    let got = api::evaluate_coded("long-service-leave", &v);
    if site["refused"].is_string() {
        assert!(got.is_err(), "{row}");
        return "refused alike";
    }
    let result = match got {
        Ok(result) => result,
        Err(err) => {
            return match err.code {
                "period.unsupported" | "long_service_leave.before_current_text" => {
                    "outside the dates covered"
                }
                "long_service_leave.taken_above_weeks" => "leave taken above the exact weeks",
                other => panic!("{other}: {row}"),
            };
        }
    };
    let service =
        ["years", "months", "days"].map(|k| result["rates"][k].as_str().unwrap().to_string());
    let want = ["years", "months", "days"].map(|k| site[k].to_string());
    assert_eq!(service, want, "{row}");
    assert_eq!(
        result["rates"]["days_counted"].as_str().unwrap(),
        site["counted"].to_string(),
        "{row}"
    );
    if figures(&result) == site_figures(site) {
        return "identical";
    }
    assert_eq!(v["jurisdiction"], "qld", "{row}");
    "Queensland at 8.6667 weeks"
}

/// Every row of the site's fixture gives the page's figures or refusal, or
/// a counted departure.
#[test]
fn the_site_cases_agree_except_where_a_source_decides_otherwise() {
    let data: Value =
        serde_json::from_str(include_str!("data/long_service_leave_site_cases.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    let mut tally = BTreeMap::<&str, usize>::new();
    let mut page = BTreeMap::<&str, usize>::new();
    for row in rows {
        let outcome = checked(row);
        *tally.entry(outcome).or_default() += 1;
        if row["source"].as_str().unwrap().starts_with("page: ") {
            *page.entry(outcome).or_default() += 1;
        }
    }
    eprintln!(
        "long-service-leave site parity: {} rows {tally:?}; page cases {page:?}",
        rows.len()
    );
    assert_eq!(rows.len(), 3016);
    assert_eq!(page, BTreeMap::from(PAGE));
    assert_eq!(tally, BTreeMap::from(TALLY));
}

const PAGE: [(&str, usize); 4] = [
    ("identical", 28),
    ("refused alike", 6),
    ("Queensland at 8.6667 weeks", 3),
    ("outside the dates covered", 2),
];
const TALLY: [(&str, usize); 4] = [
    ("identical", 2521),
    ("refused alike", 69),
    ("Queensland at 8.6667 weeks", 241),
    ("outside the dates covered", 185),
];

/// The regulators' examples: place, start, end, reason (empty while
/// employed), weekly pay, leave taken, then weeks, balance and value.
type Example = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    [&'static str; 3],
);

const EXAMPLES: [Example; 8] = [
    // NSW Industrial Relations: redundancy after 12 years and 6 months, a
    // proportion on 3 months for 15 years.
    (
        "nsw",
        "2014-02-01",
        "2026-07-31",
        "redundancy",
        "1500",
        "0",
        ["10.8333", "10.8333", "16250.00"],
    ),
    // Victoria: 2,557 days / 420.
    (
        "vic",
        "2019-01-01",
        "2025-12-31",
        "resign",
        "1500",
        "0",
        ["6.0881", "6.0881", "9132.14"],
    ),
    // Wageline's Lee: redundancy after 12 years.
    (
        "wa",
        "2014-01-01",
        "2025-12-31",
        "redundancy",
        "1500",
        "0",
        ["10.4000", "10.4000", "15600.00"],
    ),
    // WorkSafe Tasmania's example 2: 13 years, 6 months and 14 days.
    (
        "tas",
        "2012-01-01",
        "2025-07-14",
        "resign",
        "1500",
        "0",
        ["11.7332", "11.7332", "17599.83"],
    ),
    // WorkSafe ACT: 7 years at $560 a week, 91/15 weeks (its 6.0667 x $560
    // is $3,397.35; the Act's exact period gives $3,397.33).
    (
        "act",
        "2016-01-01",
        "2022-12-31",
        "",
        "560",
        "0",
        ["6.0667", "6.0667", "3397.33"],
    ),
    // Business Queensland: 18 years, 3 months and 16 days; 8.6667 weeks for
    // 10 years gives $23,782.23 (8 2/3 weeks, the page, $23,782.14).
    (
        "qld",
        "2007-01-01",
        "2025-04-16",
        "",
        "1500",
        "0",
        ["15.8548", "15.8548", "23782.23"],
    ),
    // Queensland after 16 years and 6 months, on the Act's 8.6667 weeks.
    (
        "qld",
        "2010-01-01",
        "2026-06-30",
        "",
        "1500",
        "0",
        ["14.3001", "14.3001", "21450.08"],
    ),
    // SafeWork SA's Liam: 13 years, 10 weeks taken, 6.9 left.
    (
        "sa",
        "2013-01-01",
        "2025-12-31",
        "",
        "1500",
        "10",
        ["16.9000", "6.9000", "10350.00"],
    ),
];

#[test]
fn the_regulators_examples_agree() {
    for (place, start, end, reason, pay, taken, want) in EXAMPLES {
        let mut v = input(place, start, end, &year_of(end));
        v = with(v, json!({"weekly_pay": pay, "leave_taken": taken}));
        if !reason.is_empty() {
            v = ends(v, reason);
        }
        assert_eq!(figures(&run(&v)), want, "{v}");
    }
}

#[test]
fn hours_feed_leave_provisions() {
    // 10.8333... weeks at 38 hours is 411.666..., 411.67.
    let v = with(
        ends(
            input("nsw", "2014-02-01", "2026-07-31", "2026-27"),
            "redundancy",
        ),
        json!({"hours_per_week": "38"}),
    );
    let r = run(&v);
    assert_eq!(r["rates"]["hours"], "411.67");
    assert_eq!(
        r["rates"]["provision"],
        "Proportion on 3 months for 15 years, any reason, s 4(2)(a)(ii)"
    );
    let lines = json!([{
        "label": "Long service leave", "leave": "long_service", "hours": r["rates"]["hours"],
        "hourly_rate": "39.47", "probability": "1", "settled_within_12_months": true,
    }]);
    let provision = api::evaluate(
        "leave-provisions",
        &json!({"lines": lines, "defined_benefit_plan": false, "plan_assets": false, "year": "2026-27", "scope_confirmed": true}),
    );
    assert!(provision.is_ok(), "{provision:?}");
}

/// Endings and dates the worksheet refuses: place, start, end, year, reason
/// (empty while employed) and the code.
const REFUSED: [(&str, &str, &str, &str, &str, &str); 9] = [
    (
        "nsw",
        "2016-01-01",
        "2023-12-31",
        "2023-24",
        "fixed_term",
        "long_service_leave.unsettled",
    ),
    (
        "nt",
        "2016-01-01",
        "2024-12-31",
        "2024-25",
        "death",
        "long_service_leave.unsettled",
    ),
    (
        "nsw",
        "2011-01-01",
        "2021-03-24",
        "2020-21",
        "",
        "long_service_leave.before_current_text",
    ),
    (
        "vic",
        "2011-01-01",
        "2021-06-30",
        "2020-21",
        "",
        "long_service_leave.before_current_text",
    ),
    (
        "wa",
        "2011-01-01",
        "2022-06-19",
        "2021-22",
        "",
        "long_service_leave.before_current_text",
    ),
    (
        "sa",
        "2011-01-01",
        "2020-06-30",
        "2019-20",
        "",
        "period.unsupported",
    ),
    (
        "sa",
        "2011-01-01",
        "2025-06-30",
        "2025-26",
        "",
        "long_service_leave.year_mismatch",
    ),
    (
        "sa",
        "1990-06-22",
        "2025-06-30",
        "2024-25",
        "",
        "long_service_leave.start_too_early",
    ),
    (
        "sa",
        "2025-06-30",
        "2025-06-29",
        "2024-25",
        "",
        "long_service_leave.end_before_start",
    ),
];

#[test]
fn unsettled_endings_and_dates_outside_the_text_are_refused() {
    for (place, start, end, year, reason, want) in REFUSED {
        let mut v = input(place, start, end, year);
        if !reason.is_empty() {
            v = ends(v, reason);
        }
        assert_eq!(code(&v), want, "{v}");
    }
    // The first day each text covers is worked.
    for (place, end, year) in [
        ("nsw", "2021-03-25", "2020-21"),
        ("vic", "2021-07-01", "2021-22"),
        ("wa", "2022-06-20", "2021-22"),
    ] {
        let v = input(place, "2011-01-01", end, year);
        assert!(api::evaluate("long-service-leave", &v).is_ok(), "{place}");
    }
}

/// Changes to a valid input and the code each gives.
const INVALID: [(&str, &str); 16] = [
    (
        r#"{"jurisdiction": "au"}"#,
        "long_service_leave.jurisdiction_invalid",
    ),
    (r#"{"status": "left"}"#, "long_service_leave.status_invalid"),
    (r#"{"status": "ends"}"#, "input.missing_field"),
    (r#"{"reason": "resign"}"#, "input.field_unknown"),
    (
        r#"{"status": "ends", "reason": "quit"}"#,
        "long_service_leave.reason_invalid",
    ),
    (
        r#"{"excluded_days": 3653}"#,
        "long_service_leave.excluded_invalid",
    ),
    (
        r#"{"excluded_days": -1}"#,
        "long_service_leave.excluded_invalid",
    ),
    (
        r#"{"excluded_days": "0"}"#,
        "long_service_leave.excluded_invalid",
    ),
    (
        r#"{"leave_taken": "8.6667"}"#,
        "long_service_leave.taken_above_weeks",
    ),
    (
        r#"{"leave_taken": "1.00001"}"#,
        "long_service_leave.taken_invalid",
    ),
    (
        r#"{"leave_taken": "-1"}"#,
        "long_service_leave.taken_invalid",
    ),
    (
        r#"{"weekly_pay": "1000000.01"}"#,
        "long_service_leave.pay_invalid",
    ),
    (
        r#"{"hours_per_week": "0"}"#,
        "long_service_leave.hours_invalid",
    ),
    (
        r#"{"hours_per_week": "168.01"}"#,
        "long_service_leave.hours_invalid",
    ),
    (
        r#"{"hours_per_week": "38.125"}"#,
        "long_service_leave.hours_invalid",
    ),
    (r#"{"start": "2016-1-01"}"#, "input.date_invalid"),
];

#[test]
fn invalid_inputs_are_refused() {
    let base = input("nsw", "2016-01-01", "2025-12-31", "2025-26");
    for (change, want) in INVALID {
        let change: Value = serde_json::from_str(change).unwrap();
        assert_eq!(code(&with(base.clone(), change.clone())), want, "{change}");
    }
    // 8.6666 weeks taken of 8 2/3 leaves 0.0001 in the balance and
    // 0.0000666... weeks of pay, $0.10.
    let taken = run(&with(base, json!({"leave_taken": "8.6666"})));
    assert_eq!(figures(&taken), ["8.6667", "0.0001", "0.10"]);
}
