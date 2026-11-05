// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The edge cases in a published reviewer verdict, as fabricated cases: each
//! one has a case or a stated reason (`tests/data/lodgeit-verdict-cases.json`,
//! attributed in NOTICE).
use div7a_loan_review::api;
use serde_json::{Value, json};

fn cases() -> Value {
    serde_json::from_str(include_str!("data/lodgeit-verdict-cases.json")).unwrap()
}

/// Run the gate first where a case names its facts, as a reviewer would.
fn run(case: &Value) -> Value {
    let command = case["command"].as_str().unwrap();
    let mut input = case["input"].clone();
    if let Some(facts) = case.get("gate_facts") {
        let gate = api::evaluate("complying-loan-gate", &json!({"facts": facts})).unwrap();
        input["facts"]["gate_result"] = gate;
    }
    api::evaluate(command, &input).unwrap()
}

#[test]
fn every_edge_case_has_a_case_or_a_stated_reason() {
    let file = cases();
    assert_eq!(file["synthetic"], true);
    let edge_cases = file["edge_cases"].as_array().unwrap();
    let ids: Vec<&str> = edge_cases
        .iter()
        .map(|e| e["id"].as_str().expect("an edge case id is a string"))
        .collect();
    assert_eq!(
        ids,
        [
            "V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8", "V9", "V10", "V11", "V12", "V13",
            "V14",
        ]
    );
    for edge in edge_cases {
        let has_case = edge["cases"].as_array().is_some_and(|c| !c.is_empty());
        let has_reason = edge["reason"].as_str().is_some_and(|r| !r.is_empty());
        assert!(has_case || has_reason, "{}", edge["id"]);
        assert!(edge["provisions"].is_string(), "{}", edge["id"]);
    }
}

#[test]
fn every_case_reaches_its_stated_result() {
    let file = cases();
    let all = file["edge_cases"].as_array().unwrap();
    for case in all.iter().filter_map(|e| e["cases"].as_array()).flatten() {
        let name = &case["name"];
        let result = run(case);
        for (key, want) in case["expect"].as_object().unwrap() {
            if key == "caveats_mention" {
                let needle = want.as_str().unwrap();
                let caveats = result["caveats"].as_array().unwrap();
                assert!(
                    caveats.iter().any(|c| c.as_str().unwrap().contains(needle)),
                    "{name}: {caveats:?}"
                );
            } else {
                // get, not indexing: an expected null must not pass on an absent key.
                assert_eq!(result.get(key), Some(want), "{name}: {key}");
            }
        }
    }
}

#[test]
fn the_notice_credits_the_reviewer_and_the_licence() {
    let file = cases();
    let notice = include_str!("../NOTICE");
    for field in ["reviewer", "credential", "url", "sha256"] {
        let value = file["verdict"][field].as_str().unwrap();
        assert!(notice.contains(value), "{field}");
    }
    assert!(notice.contains("https://creativecommons.org/licenses/by/4.0/"));
}
