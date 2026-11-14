// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use gst_review::{api, bas::Period, coding::Registration, decimal};
use serde_json::{Value, json};

#[test]
fn every_original_behaviour_case_matches_the_python_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/python-contracts.json")).unwrap();
    let tests: std::collections::BTreeSet<_> =
        cases.iter().map(|c| c["test"].as_str().unwrap()).collect();
    assert_eq!(tests.len(), 77);
    for case in cases {
        decimal::with_precision(case["caller_precision"].as_i64().unwrap().into(), || {
            let before = decimal::prec();
            let actual = api::evaluate(case["command"].as_str().unwrap(), &case["input"]);
            if let Some(error) = case.get("error") {
                assert_eq!(
                    actual.unwrap_err(),
                    error.as_str().unwrap(),
                    "{}",
                    case["test"]
                );
            } else {
                let actual = actual.unwrap();
                assert_eq!(actual, case["expected"], "{}", case["test"]);
                if let Some(counts) = actual.get("counts") {
                    assert_eq!(
                        counts
                            .as_object()
                            .unwrap()
                            .keys()
                            .map(String::as_str)
                            .collect::<Vec<_>>(),
                        [
                            "unknown_gst_code",
                            "gst_differs_from_registration",
                            "supplier_abn_invalid",
                            "between_entities",
                            "coded",
                            "suggested",
                            "nothing_suggested"
                        ]
                    );
                }
                if let Some(periods) = actual.get("periods").and_then(Value::as_array) {
                    for period in periods {
                        assert_eq!(
                            period["labels"]
                                .as_object()
                                .unwrap()
                                .keys()
                                .map(String::as_str)
                                .collect::<Vec<_>>(),
                            ["G1", "G2", "G3", "G10", "G11", "1A", "1B"]
                        );
                    }
                }
            }
            assert_eq!(
                decimal::prec(),
                before,
                "caller context changed: {}",
                case["test"]
            );
        });
    }
}

#[test]
fn public_date_helpers_include_both_boundaries() {
    let start = "2026-07-01".parse().unwrap();
    let end = "2026-09-30".parse().unwrap();
    let period = Period {
        label: "Q1".into(),
        start,
        end,
    };
    let registration = Registration {
        registered_from: Some(start),
        registered_to: Some(end),
    };
    for day in [
        start.pred_opt().unwrap(),
        start,
        end,
        end.succ_opt().unwrap(),
    ] {
        let expected = day >= start && day <= end;
        assert_eq!(period.holds(day), expected);
        assert_eq!(registration.covers(day), expected);
    }
    assert!(
        !Registration {
            registered_from: None,
            registered_to: None
        }
        .covers(start)
    );
}

#[test]
fn money_scale_pairing_ties_signed_zero_and_custom_table_order() {
    let input = json!({"registrations":{"A":{"registered_from":null},"B":{"registered_from":null}},"lines":[
        {"id":"out","entity":"A","date":"2026-07-10","amount":"-110.0"},
        {"id":"first","entity":"B","date":"2026-07-09","amount":"110.00"},
        {"id":"second","entity":"B","date":"2026-07-11","amount":"110"},
        {"id":"zero","entity":"A","date":"2026-07-10","amount":"-0.00"},
        {"id":"otherzero","entity":"B","date":"2026-07-10","amount":"0"}]});
    let result = api::evaluate("review-coding", &input).unwrap();
    assert_eq!(result["lines"][0]["counterpart"], "first");
    assert_eq!(result["lines"][2]["counterpart"], Value::Null);
    assert_eq!(result["lines"][3]["counterpart"], Value::Null);
    assert_eq!(result["lines"][3]["amount"], "-0.00");
    let error = api::evaluate(
        "review-coding",
        &json!({"gst_codes":{"a":"z","b":"a'"},"registrations":{},"lines":[]}),
    )
    .unwrap_err();
    assert_eq!(
        error,
        "Unknown GST treatments in the code table: [\"a'\", 'z']."
    );
}

#[test]
fn empty_lodged_period_and_input_period_order_are_retained() {
    let result = api::evaluate(
        "reconcile-bas",
        &json!({"basis":"accruals","lines":[],"periods":[
        {"label":"Q2","start":"2026-10-01","end":"2026-12-31"},
        {"label":"Q1","start":"2026-07-01","end":"2026-09-30"}],"lodged":{"Q2":{}}}),
    )
    .unwrap();
    assert_eq!(result["periods"][0]["period"], "Q2");
    assert_eq!(result["periods"][0]["lodged"], true);
    assert_eq!(result["periods"][1]["lodged"], false);
    assert_eq!(result["no_ledger_lines"], json!(["Q2"]));
}

#[test]
fn malformed_json_types_are_refused_without_panics() {
    for input in [Value::Null, json!([]), json!(false)] {
        assert!(api::evaluate("review-coding", &input).is_err());
    }
    for input in [
        json!({}),
        json!({"lines":null,"registrations":{}}),
        json!({"lines":[],"registrations":false}),
        json!({"lines":[null],"registrations":{}}),
    ] {
        assert!(api::evaluate("review-coding", &input).is_err());
    }
    assert!(api::evaluate("unknown", &json!({})).is_err());
}

// ABN Lookup's worked example 51 824 753 556 and a copy with a wrong last digit,
// from https://abr.business.gov.au/Help/AbnFormat (version 9.9.7, read 14 November 2026).
#[test]
fn supplier_abns_failing_the_check_digit_are_findings() {
    let line = |id: &str, abn: Value| {
        json!({"id":id,"entity":"A","date":"2026-08-04","amount":"-110.00","account":"400",
        "gst_code":"GST","confirmed":true,"supplier_abn":abn})
    };
    let input = json!({"registrations":{"A":{"registered_from":"2024-07-01"}},"lines":[
        line("valid", json!("51 824 753 556")),
        line("wrong", json!("51 824 753 557")),
        line("short", json!("5182475355")),
        line("none", Value::Null)]});
    let result = api::evaluate("review-coding", &input).unwrap();
    let states: Vec<_> = result["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["state"].as_str().unwrap())
        .collect();
    assert_eq!(
        states,
        [
            "coded",
            "supplier_abn_invalid",
            "supplier_abn_invalid",
            "coded"
        ]
    );
    assert_eq!(
        result["lines"][1]["reason"],
        "Supplier ABN '51 824 753 557' fails the ABN format and check digit test ABN Lookup publishes, so it is not a valid ABN as written. Check it against its source."
    );
    assert_eq!(result["counts"]["supplier_abn_invalid"], 2);
    assert!(
        result["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().starts_with(
                "A valid check digit shows only that the digits pass the arithmetic ABN Lookup publishes."
            ))
    );
    let without = api::evaluate(
        "review-coding",
        &json!({"registrations":{"A":{"registered_from":null}},"lines":[line("none", Value::Null)]}),
    )
    .unwrap();
    assert_eq!(without["warnings"].as_array().unwrap().len(), 2);
    let error = api::evaluate(
        "review-coding",
        &json!({"registrations":{"A":{"registered_from":null}},"lines":[line("n", json!(51824753556_u64))]}),
    )
    .unwrap_err();
    assert_eq!(error, "Line 'n' supplier_abn must be a string or None.");
}

/// ABN Lookup's steps as written: subtract 1 from the first digit, weight, sum and
/// divide by 89. None when the text is not 11 ASCII digits with optional spaces.
fn abr_steps(text: &str) -> Option<bool> {
    if !text.chars().all(|c| c.is_ascii_digit() || c == ' ') {
        return None;
    }
    let mut digits: Vec<i64> = text
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(i64::from)
        .collect();
    if digits.len() != 11 {
        return None;
    }
    digits[0] -= 1;
    let weights = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    Some(digits.iter().zip(weights).map(|(d, w)| d * w).sum::<i64>() % 89 == 0)
}

#[test]
fn the_abn_check_agrees_with_abn_lookups_steps_as_written() {
    let passes = |abn: &str| {
        let input = json!({"registrations":{"A":{"registered_from":"2024-07-01"}},"lines":[
            {"id":"1","entity":"A","date":"2026-08-04","amount":"-110.00","account":"400",
            "gst_code":"GST","confirmed":true,"supplier_abn":abn}]});
        let result = api::evaluate("review-coding", &input).unwrap();
        match result["lines"][0]["state"].as_str().unwrap() {
            "coded" => true,
            "supplier_abn_invalid" => false,
            other => panic!("{abn:?}: {other}"),
        }
    };
    // ABN Lookup's example as written in several ways, and texts the format refuses.
    for (abn, valid) in [
        ("51824753556", true),
        ("51 824 753 556", true),
        (" 5 1824753556 ", true),
        ("51 824 753 557", false),
        ("51\t824 753 556", false),
        ("51\u{a0}824 753 556", false),
        ("51-824-753-556", false),
        ("\u{ff15}1 824 753 556", false),
        ("\u{665}1 824 753 556", false),
        ("", false),
        ("   ", false),
        ("5182475355", false),
        ("518247535560", false),
        ("051824753556", false),
    ] {
        assert_eq!(passes(abn), valid, "{abn:?}");
        assert_eq!(abr_steps(abn) == Some(true), valid, "{abn:?}");
    }
    // A leading zero cannot have 1 subtracted, though -1 in its place would divide by 89.
    assert_eq!(abr_steps("00000090000"), Some(true));
    assert!(!passes("00000090000"));
    // Every 2-digit prefix of pseudo-random 9-digit bodies, with and without spaces.
    let mut seed: u64 = 20_261_114;
    let mut valid = 0;
    for _ in 0..40 {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let body = format!("{:09}", (seed >> 33) % 1_000_000_000);
        for prefix in 0..100 {
            let abn = format!("{prefix:02}{body}");
            let expected = prefix >= 10 && abr_steps(&abn) == Some(true);
            valid += usize::from(expected);
            assert_eq!(passes(&abn), expected, "{abn}");
            let spaced = format!("{} {} {} {}", &abn[..2], &abn[2..5], &abn[5..8], &abn[8..]);
            assert_eq!(passes(&spaced), expected, "{spaced}");
        }
    }
    // Check digits 10 to 99 reach every remainder, so each body has a valid ABN.
    assert!(valid >= 40, "{valid}");
}

#[test]
fn an_invalid_supplier_abn_ranks_below_registration_conflicts_and_above_the_rest() {
    let bad = "51 824 753 557";
    let input = json!({"registrations":{"A":{"registered_from":"2024-07-01"},"B":{"registered_from":null}},"lines":[
        {"id":"unregistered","entity":"B","date":"2026-08-04","amount":"-55.00","gst_code":"GST","supplier_abn":bad},
        {"id":"supplier-unregistered","entity":"A","date":"2026-08-04","amount":"-66.00","gst_code":"GST","supplier_registered":false,"supplier_abn":bad},
        {"id":"suggested","entity":"A","date":"2026-08-04","amount":"-77.00","suggested_gst_code":"GST","supplier_abn":bad},
        {"id":"out","entity":"A","date":"2026-08-04","amount":"-88.00","supplier_abn":bad},
        {"id":"in","entity":"B","date":"2026-08-05","amount":"88.00"}]});
    let result = api::evaluate("review-coding", &input).unwrap();
    let states: Vec<_> = result["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| (l["state"].as_str().unwrap(), l["counterpart"].clone()))
        .collect();
    assert_eq!(
        states,
        [
            ("gst_differs_from_registration", Value::Null),
            ("gst_differs_from_registration", Value::Null),
            ("supplier_abn_invalid", Value::Null),
            ("supplier_abn_invalid", json!("in")),
            ("between_entities", json!("out")),
        ]
    );
}
