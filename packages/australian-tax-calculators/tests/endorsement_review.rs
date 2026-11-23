// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The ATO's endorsement review worksheets (NAT 74110, NAT 11685 and NAT
//! 11686, February 2021), each question's outcome as the worksheet gives it
//! and as ITAA 1997 Divisions 30 and 50 and TAA 1953 Sch 1 Division 426 read
//! on 24 November 2026 (docs/calculation-evidence.md#endorsement-review).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("endorsement-review", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("endorsement-review", input)
        .unwrap_err()
        .code
}

const WORKSHEETS: [(&str, &[&str]); 3] = [
    (
        "income_tax_exempt_charity",
        &[
            "abn_current",
            "registered_charity",
            "meets_a_test",
            "meets_both_conditions",
        ],
    ),
    (
        "dgr_whole",
        &[
            "abn_current",
            "within_category",
            "winding_up_clause",
            "in_australia",
            "receipts_correct",
        ],
    ),
    (
        "dgr_fund",
        &[
            "abn_current",
            "within_category",
            "gift_fund_maintained",
            "in_australia",
            "receipts_correct",
        ],
    ),
];

fn review(worksheet: &str, keys: &[&str], no: Option<&str>) -> Value {
    let answers: serde_json::Map<String, Value> = keys
        .iter()
        .map(|k| ((*k).to_owned(), json!(Some(*k) != no)))
        .collect();
    json!({"year": "2026-27", "scope_confirmed": true, "worksheet": worksheet, "answers": answers})
}

#[test]
fn all_yes_continues() {
    for (worksheet, keys) in WORKSHEETS {
        let out = run(&review(worksheet, keys, None));
        assert_eq!(out["status"], "continues", "{worksheet}");
        let findings = out["findings"].as_array().unwrap();
        assert_eq!(findings.len(), keys.len());
        assert!(findings.iter().all(|f| f.get("outcome").is_none()));
        assert!(!out["not_asked"].as_array().unwrap().is_empty());
    }
}

#[test]
fn each_no_gives_its_outcome() {
    let action_needed = ["winding_up_clause", "receipts_correct"];
    for (worksheet, keys) in WORKSHEETS {
        for key in keys {
            let out = run(&review(worksheet, keys, Some(key)));
            let expected = if action_needed.contains(key) {
                "action_needed"
            } else {
                "not_entitled"
            };
            assert_eq!(out["status"], expected, "{worksheet} {key}");
            let finding = out["findings"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["question"] == *key)
                .unwrap();
            assert_eq!(finding["answer"], false);
            assert_eq!(finding["outcome"], expected);
            let action = finding["action"].as_str().unwrap();
            // A cancelled ABN needs no notice from the entity (s 426-45(3)).
            assert_eq!(
                action.contains("426-45(3)"),
                *key == "abn_current",
                "{worksheet} {key}"
            );
        }
    }
}

#[test]
fn not_entitled_outranks_action_needed() {
    let (worksheet, keys) = WORKSHEETS[1];
    let mut input = review(worksheet, keys, Some("receipts_correct"));
    input["answers"]["in_australia"] = json!(false);
    assert_eq!(run(&input)["status"], "not_entitled");
}

#[test]
fn refusals() {
    let (worksheet, keys) = WORKSHEETS[0];
    let ok = review(worksheet, keys, None);
    let mut wrong_set = ok.clone();
    wrong_set["worksheet"] = json!("dgr_whole");
    let mut extra = ok.clone();
    extra["answers"]["more"] = json!(true);
    let mut not_bool = ok.clone();
    not_bool["answers"]["abn_current"] = json!("yes");
    let mut not_object = ok.clone();
    not_object["answers"] = json!([true]);
    let mut bad_worksheet = ok.clone();
    bad_worksheet["worksheet"] = json!("dgr");
    let mut old = ok.clone();
    old["year"] = json!("2025-26");
    for (input, expected) in [
        (wrong_set, "endorsement_review.answers_invalid"),
        (extra, "endorsement_review.answers_invalid"),
        (not_bool, "endorsement_review.answers_invalid"),
        (not_object, "endorsement_review.answers_invalid"),
        (bad_worksheet, "endorsement_review.worksheet_invalid"),
        (old, "period.unsupported"),
    ] {
        assert_eq!(code(&input), expected, "{input}");
    }
}
