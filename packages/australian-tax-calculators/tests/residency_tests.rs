// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The residency tests of ITAA 1936 s 6(1), resident (a) (compilations 169
//! and 193, read on 22 November 2026), against TR 2023/1: examples 5, 7, 10,
//! 11, 13, 14, 15, 16 and 18, with dates fabricated to fit each narrative,
//! the day count (paragraphs 86 and 87, footnote 51) at each year's
//! boundary, and the refusals (docs/calculation-evidence.md#residency-tests).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("residency-tests", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("residency-tests", input)
        .unwrap_err()
        .code
}

fn span(from: &str, to: &str) -> Value {
    json!({"from": from, "to": to})
}

/// No test met: no residing, no domicile, no days, no fund.
fn base(year: &str) -> Value {
    json!({
        "year": year, "scope_confirmed": true, "resides": false,
        "domicile_in_australia": false, "permanent_place_of_abode_outside": false,
        "presence": [], "usual_place_of_abode_outside": false,
        "intends_to_take_up_residence": false, "commonwealth_super": "none",
        "resident_elsewhere": false,
    })
}

fn with(mut input: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            input.as_object_mut().unwrap().remove(key);
        } else {
            input[key] = value.clone();
        }
    }
    input
}

fn met_by(result: &Value) -> Vec<&str> {
    result["met_by"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect()
}

/// `days` consecutive days in Australia from 1 July of the year's first
/// calendar year.
fn from_july(year: &str, days: i64) -> Value {
    let start = format!("{}-07-01", &year[..4]);
    let end = add_days(&start, days - 1);
    json!([span(&start, &end)])
}

fn add_days(date: &str, days: i64) -> String {
    let mut y: i64 = date[..4].parse().unwrap();
    let mut m: i64 = date[5..7].parse().unwrap();
    let mut d = date[8..].parse::<i64>().unwrap() + days;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let len = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ][(m - 1) as usize];
        if d <= len {
            return format!("{y:04}-{m:02}-{d:02}");
        }
        d -= len;
        m += 1;
        if m == 13 {
            m = 1;
            y += 1;
        }
    }
}

/// TR 2023/1's examples: the facts, then whether resident and the tests met.
#[test]
fn the_ruling_examples_decide_alike() {
    let visitor = json!({"usual_place_of_abode_outside": true});
    let cases: Vec<(&str, Value, bool, Vec<&str>, i64)> = vec![
        (
            "example 5, Conchita: 9 months on holiday, abode in Mexico",
            with(
                base("2025-26"),
                with(
                    visitor.clone(),
                    json!({"presence": [span("2025-08-01", "2026-04-30")]}),
                ),
            ),
            false,
            vec![],
            273,
        ),
        (
            "example 11, Stuart: domiciled here, no permanent abode abroad",
            with(
                base("2025-26"),
                json!({"domicile_in_australia": true, "presence": [
                    span("2025-09-01", "2025-09-07"), span("2025-12-20", "2025-12-26"),
                    span("2026-04-01", "2026-04-07")]}),
            ),
            true,
            vec!["domicile"],
            21,
        ),
        (
            "example 10, Brian: domiciled here, permanent abode in Vietnam",
            with(
                base("2025-26"),
                json!({"domicile_in_australia": true, "permanent_place_of_abode_outside": true,
                    "presence": [span("2025-07-01", "2025-07-20"), span("2025-09-10", "2025-09-16"),
                    span("2025-11-10", "2025-11-16")]}),
            ),
            false,
            vec![],
            34,
        ),
        (
            "example 16, Megan: stays from July, intends to live here",
            with(
                base("2025-26"),
                json!({"resides": true, "intends_to_take_up_residence": true,
                    "presence": [span("2025-07-01", "2026-06-30")]}),
            ),
            true,
            vec!["ordinary_concepts", "days_183"],
            365,
        ),
        (
            "example 13, Aiko: resides from arrival, under 183 days",
            with(
                base("2025-26"),
                json!({"resides": true, "presence": [span("2026-01-05", "2026-02-10"),
                    span("2026-03-01", "2026-03-07"), span("2026-04-20", "2026-04-26"),
                    span("2026-06-10", "2026-06-30")]}),
            ),
            true,
            vec!["ordinary_concepts"],
            72,
        ),
        (
            "example 14, Alani: 9 months of seasonal work, abode in Papua New Guinea",
            with(
                base("2025-26"),
                with(
                    visitor.clone(),
                    json!({"presence": [span("2025-09-01", "2026-05-31")]}),
                ),
            ),
            false,
            vec![],
            273,
        ),
        (
            "example 15, Grace: a 12-month working holiday from August, abode in the UK",
            with(
                base("2025-26"),
                with(
                    visitor.clone(),
                    json!({"presence": [span("2025-08-15", "2026-08-14")]}),
                ),
            ),
            false,
            vec![],
            320,
        ),
        (
            "example 18, Ryan: first year of 2 working holidays, abode in Ireland",
            with(
                base("2024-25"),
                with(
                    visitor.clone(),
                    json!({"presence": [span("2024-07-01", "2026-06-30")]}),
                ),
            ),
            false,
            vec![],
            365,
        ),
        (
            "example 18, Ryan: second year, a share house on a 12-month lease",
            with(
                base("2025-26"),
                with(
                    visitor.clone(),
                    json!({"presence": [span("2024-07-01", "2026-06-30")]}),
                ),
            ),
            false,
            vec![],
            365,
        ),
        (
            "example 7, Mark: in Brazil for 2 years, family home here",
            with(
                base("2025-26"),
                json!({"resides": true, "domicile_in_australia": true,
                    "presence": [span("2025-12-20", "2026-01-10")]}),
            ),
            true,
            vec!["ordinary_concepts", "domicile"],
            22,
        ),
    ];
    for (name, input, resident, tests, days) in cases {
        let got = run(&input);
        assert_eq!(got["resident"], json!(resident), "{name}");
        assert_eq!(met_by(&got), tests, "{name}");
        assert_eq!(got["tests"]["days_in_australia"], json!(days), "{name}");
    }
}

/// More than one-half of the year: 183 days, and 184 in 2023-24, which
/// holds 29 February 2024 (TR 2023/1 footnote 51).
#[test]
fn each_year_needs_more_than_half_its_days() {
    let years = [
        ("2020-21", 365),
        ("2021-22", 365),
        ("2022-23", 365),
        ("2023-24", 366),
        ("2024-25", 365),
        ("2025-26", 365),
        ("2026-27", 365),
    ];
    for (year, length) in years {
        let needed = length / 2 + 1;
        let short = run(&with(
            base(year),
            json!({"presence": from_july(year, needed - 1)}),
        ));
        let enough = run(&with(
            base(year),
            json!({"presence": from_july(year, needed)}),
        ));
        assert_eq!(short["income_year"]["days"], json!(length), "{year}");
        assert_eq!(short["tests"]["days_needed"], json!(needed), "{year}");
        assert_eq!(short["resident"], json!(false), "{year}");
        assert_eq!(enough["resident"], json!(true), "{year}");
        assert_eq!(met_by(&enough), ["days_183"], "{year}");
    }
}

/// The exception needs both a usual place of abode outside Australia and no
/// intention to take up residence.
#[test]
fn the_183_day_exception_needs_both_answers() {
    let year = from_july("2025-26", 183);
    let cases = [
        (true, false, false),
        (true, true, true),
        (false, false, true),
        (false, true, true),
    ];
    for (outside, intends, resident) in cases {
        let got = run(&with(
            base("2025-26"),
            json!({"presence": year, "usual_place_of_abode_outside": outside,
                "intends_to_take_up_residence": intends}),
        ));
        assert_eq!(got["resident"], json!(resident), "{outside} {intends}");
    }
}

#[test]
fn days_are_counted_once_and_inside_the_year() {
    let cases = [
        (json!([span("2025-10-01", "2025-10-01")]), 1),
        (json!([span("2025-06-01", "2025-07-10")]), 10),
        (json!([span("2026-06-25", "2026-07-05")]), 6),
        (
            json!([
                span("2025-08-01", "2025-08-10"),
                span("2025-08-10", "2025-08-20")
            ]),
            20,
        ),
        (
            json!([
                span("2025-08-01", "2025-08-31"),
                span("2025-08-05", "2025-08-06")
            ]),
            31,
        ),
        (json!([span("2024-01-01", "2028-12-31")]), 365),
        (json!([span("2024-01-01", "2025-06-30")]), 0),
    ];
    for (presence, days) in cases {
        let got = run(&with(base("2025-26"), json!({"presence": presence})));
        assert_eq!(got["tests"]["days_in_australia"], json!(days), "{presence}");
    }
}

#[test]
fn each_commonwealth_fund_link_makes_a_resident() {
    for link in [
        "pss_member",
        "css_eligible_employee",
        "partner_of_member",
        "child_under_16_of_member",
    ] {
        let got = run(&with(base("2025-26"), json!({"commonwealth_super": link})));
        assert_eq!(got["resident"], json!(true), "{link}");
        assert_eq!(met_by(&got), ["commonwealth_super"], "{link}");
    }
    assert_eq!(run(&base("2025-26"))["resident"], json!(false));
}

/// The permanent place of abode is read only with an Australian domicile.
#[test]
fn a_foreign_domicile_ignores_the_permanent_abode() {
    let got = run(&with(
        base("2025-26"),
        json!({"permanent_place_of_abode_outside": true}),
    ));
    assert_eq!(got["tests"]["domicile"], json!(false));
    let steps = got["steps"].to_string();
    assert!(!steps.contains("permanent place of abode outside"));
}

/// Treaty tie-breakers are refused for a resident, not for a non-resident.
#[test]
fn a_treaty_resident_is_refused() {
    let resident = with(
        base("2025-26"),
        json!({"resides": true, "resident_elsewhere": true}),
    );
    assert_eq!(code(&resident), "residency_tests.treaty_tie_breaker");
    let visitor = with(base("2025-26"), json!({"resident_elsewhere": true}));
    assert_eq!(run(&visitor)["resident"], json!(false));
}

#[test]
fn bad_inputs_are_refused() {
    let cases = [
        (json!({"year": "2019-20"}), "period.unsupported"),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"resides": "yes"}), "input.flag_invalid"),
        (
            json!({"resident_elsewhere": "remove"}),
            "input.missing_field",
        ),
        (
            json!({"commonwealth_super": "dfat"}),
            "residency_tests.super_invalid",
        ),
        (json!({"presence": {}}), "residency_tests.presence_invalid"),
        (json!({"presence": [1]}), "residency_tests.presence_invalid"),
        (
            json!({"presence": [span("2025-08-10", "2025-08-01")]}),
            "residency_tests.presence_invalid",
        ),
        (
            json!({"presence": [span("2025-02-30", "2025-03-01")]}),
            "residency_tests.presence_invalid",
        ),
        (
            json!({"presence": [{"from": "2025-08-01"}]}),
            "input.missing_field",
        ),
        (
            json!({"presence": [{"from": "2025-08-01", "to": "2025-08-02", "visa": "417"}]}),
            "input.field_unknown",
        ),
        (json!({"days": 200}), "input.field_unknown"),
    ];
    for (change, expected) in cases {
        assert_eq!(
            code(&with(base("2025-26"), change.clone())),
            expected,
            "{change}"
        );
    }
    let many: Vec<Value> = (0..401).map(|_| span("2025-08-01", "2025-08-01")).collect();
    assert_eq!(
        code(&with(base("2025-26"), json!({"presence": many}))),
        "residency_tests.presence_invalid"
    );
}

/// The catalogue's example is TR 2023/1 example 5.
#[test]
fn the_catalogue_example_is_conchita() {
    let got = run(&with(
        base("2025-26"),
        json!({"usual_place_of_abode_outside": true,
            "presence": [span("2025-08-01", "2026-04-30")]}),
    ));
    assert_eq!(got["resident"], json!(false));
    assert_eq!(got["tests"]["days_needed"], json!(183));
}
