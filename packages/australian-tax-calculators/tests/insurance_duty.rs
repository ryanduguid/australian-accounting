// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Insurance duty on one general insurance premium in each state and
//! territory for 2026-27: figures worked by hand from each Act's rate, and an
//! independent oracle's 644 cases (`data/insurance_duty_cases.json`: every
//! class at 23 premiums, including half-cent ties and the largest amount the
//! crate takes, on the first and last day of the year, New South Wales Type C
//! for policies taken out either side of 1 January 2018, each with its exact
//! duty), written from the spec, not the code. No revenue office publishes a worked general
//! insurance example with figures. Every source was read on 24 November 2026
//! (docs/calculation-evidence.md#insurance-duty). Every premium here is
//! fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(place: &str, class: &str, premium: &str) -> Value {
    let mut v = json!({
        "jurisdiction": place, "class": class, "premium": premium,
        "effected_date": "2026-09-15", "year": "2026-27", "scope_confirmed": true,
    });
    if place == "nt" {
        v["term_over_one_year"] = json!(false);
    }
    if class == "type_c" {
        v["policy_taken_out"] = json!("2018-03-01");
    }
    v
}

/// A decimal string without trailing zeros, so 90.00 and 90 compare alike.
fn plain(text: &str) -> &str {
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        text
    }
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
    let result = api::evaluate("insurance-duty", input).unwrap();
    let envelope = json!({"command": "insurance-duty", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn duty(place: &str, class: &str, premium: &str) -> String {
    let result = run(&input(place, class, premium));
    result["amounts"]["duty"].as_str().unwrap().to_string()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("insurance-duty", input)
        .unwrap_err()
        .code
}

/// Figures worked by hand from each rate, with the cent rounding at its
/// edges.
#[test]
fn each_rate_applies_to_the_premium() {
    let cases = [
        ("nsw", "type_a", "1000.00", "90.00"),
        ("nsw", "type_b", "1000.00", "50.00"),
        // A Type C policy taken out after 2017 is exempt.
        ("nsw", "type_c", "1000.00", "0.00"),
        // 123.456 and 86.4192.
        ("vic", "general", "1234.56", "123.46"),
        ("vic", "business", "1234.56", "86.42"),
        // 49.9995 rounds up; class 2 is charged alike.
        ("qld", "class_1", "555.55", "50.00"),
        ("qld", "class_2", "555.55", "50.00"),
        ("qld", "ctp", "400.00", "0.10"),
        ("qld", "ctp", "0", "0.10"),
        ("wa", "general", "1000.00", "100.00"),
        ("sa", "general", "1000.00", "110.00"),
        // 0.0055 rounds up to a cent; 0.0044 down to nil.
        ("sa", "general", "0.05", "0.01"),
        ("sa", "general", "0.04", "0.00"),
        // 9.999.
        ("tas", "general", "99.99", "10.00"),
        ("act", "general", "5000.00", "0.00"),
        ("nt", "general", "1000.00", "100.00"),
        // 0.045 is a tie and rounds up.
        ("wa", "general", "0.45", "0.05"),
    ];
    for (place, class, premium, want) in cases {
        assert_eq!(
            duty(place, class, premium),
            want,
            "{place} {class} {premium}"
        );
    }
}

/// A repealed duty is nil and says when it went; New South Wales Type C
/// turns on the day the policy was taken out.
#[test]
fn a_repealed_or_exempt_duty_gives_its_date() {
    let act = run(&input("act", "general", "5000.00"));
    assert_eq!(act["rates"]["abolished_from"], "2017-07-01");
    let nsw = run(&input("nsw", "type_c", "800.00"));
    assert_eq!(nsw["amounts"]["duty"], "0.00");
    assert_eq!(nsw["rates"]["exempt_from"], "2018-01-01");
    let taken = |day: &str| {
        let facts = with(
            input("nsw", "type_c", "1000.00"),
            json!({"policy_taken_out": day}),
        );
        run(&facts)["amounts"]["duty"].as_str().unwrap().to_string()
    };
    assert_eq!(taken("2017-12-31"), "25.00");
    assert_eq!(taken("2018-01-01"), "0.00");
    let wa = run(&input("wa", "general", "800.00"));
    assert!(wa["rates"].get("abolished_from").is_none());
}

/// The exact duty sits beside the cents, since no Act rounds one premium's
/// duty.
#[test]
fn the_exact_duty_is_given() {
    let exact = |place: &str, class: &str, premium: &str| {
        let result = run(&input(place, class, premium));
        result["rates"]["exact_duty"].as_str().unwrap().to_string()
    };
    assert_eq!(plain(&exact("vic", "business", "1234.56")), "86.4192");
    assert_eq!(plain(&exact("sa", "general", "0.05")), "0.0055");
    assert_eq!(plain(&exact("qld", "class_1", "555.55")), "49.9995");
    assert_eq!(exact("qld", "ctp", "400.00"), "0.10");
}

/// The oracle's cases, each through `derive` as well.
#[test]
fn every_oracle_case_agrees() {
    let text = include_str!("data/insurance_duty_cases.json");
    let rows: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(rows.len(), 644);
    for row in &rows {
        let facts = with(input("wa", "general", "0"), row["input"].clone());
        let result = run(&facts);
        assert_eq!(result["amounts"]["duty"], row["duty"], "{facts}");
        let exact = result["rates"]["exact_duty"].as_str().unwrap();
        assert_eq!(
            plain(exact),
            plain(row["exact"].as_str().unwrap()),
            "{facts}"
        );
    }
}

/// Each step names its authority and shows its working.
#[test]
fn the_steps_show_their_working() {
    let working = |place: &str, class: &str, premium: &str| {
        let envelope = json!({"command": "insurance-duty", "input": input(place, class, premium)});
        let doc = api::evaluate("derive", &envelope).unwrap();
        let step = &doc["steps"][0];
        assert_eq!(step["id"], "duty");
        let authority = step["authority"].as_str().unwrap().to_string();
        (authority, step["working"].as_str().unwrap().to_string())
    };
    let (authority, sum) = working("vic", "business", "1234.56");
    assert!(authority.contains("Duties Act 2000 (Vic) s 179(2) and (4)"));
    assert_eq!(
        sum,
        "cents(premium x rate_percent / 100) = cents(1234.56 x 7 / 100) = 86.42"
    );
    let (authority, sum) = working("qld", "ctp", "400.00");
    assert!(authority.contains("s 362(2)"));
    assert_eq!(sum, "duty_per_premium = 0.10 = 0.10");
    let (authority, sum) = working("act", "general", "5000.00");
    assert!(authority.contains("A2017-11"));
    assert_eq!(
        sum,
        "cents(premium x rate_percent / 100) = cents(5000.00 x 0 / 100) = 0.00"
    );
}

fn refused(cases: &[(&str, &str, Value, &str)]) {
    for (place, class, changes, want) in cases {
        let input = with(input(place, class, "1000.00"), changes.clone());
        assert_eq!(code(&input), *want, "{input}");
    }
}

#[test]
fn what_the_acts_do_not_settle_is_refused() {
    refused(&[
        (
            "nsw",
            "type_a",
            json!({"jurisdiction": "jbt"}),
            "insurance_duty.jurisdiction_invalid",
        ),
        ("nsw", "general", json!({}), "insurance_duty.class_invalid"),
        ("vic", "type_a", json!({}), "insurance_duty.class_invalid"),
        ("wa", "ctp", json!({}), "insurance_duty.class_invalid"),
        ("qld", "life", json!({}), "insurance_duty.class_invalid"),
        ("sa", "accident", json!({}), "insurance_duty.class_invalid"),
        (
            "vic",
            "general",
            json!({"effected_date": "2026-06-30"}),
            "insurance_duty.date_outside_year",
        ),
        (
            "vic",
            "business",
            json!({"effected_date": "2027-07-01"}),
            "insurance_duty.date_outside_year",
        ),
        (
            "wa",
            "general",
            json!({"effected_date": "15/09/2026"}),
            "input.date_invalid",
        ),
        (
            "wa",
            "general",
            json!({"year": "2025-26"}),
            "period.unsupported",
        ),
        (
            "wa",
            "general",
            json!({"premium": "10.005"}),
            "input.amount_invalid",
        ),
        (
            "wa",
            "general",
            json!({"premium": "-1"}),
            "input.amount_invalid",
        ),
        (
            "wa",
            "general",
            json!({"scope_confirmed": false}),
            "scope.not_confirmed",
        ),
        (
            "nt",
            "general",
            json!({"term_over_one_year": true}),
            "insurance_duty.term_unsettled",
        ),
        (
            "nt",
            "general",
            json!({"term_over_one_year": null}),
            "input.missing_field",
        ),
        (
            "nt",
            "general",
            json!({"term_over_one_year": "no"}),
            "input.flag_invalid",
        ),
        (
            "tas",
            "general",
            json!({"term_over_one_year": false}),
            "input.field_unknown",
        ),
        (
            "nsw",
            "type_c",
            json!({"policy_taken_out": null}),
            "input.missing_field",
        ),
        (
            "nsw",
            "type_c",
            json!({"policy_taken_out": "31/12/2017"}),
            "input.date_invalid",
        ),
        (
            "nsw",
            "type_a",
            json!({"policy_taken_out": "2017-12-31"}),
            "input.field_unknown",
        ),
    ]);
}
