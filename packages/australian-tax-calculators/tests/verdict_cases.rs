// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The edge cases in a published reviewer verdict, as fabricated cases: each
//! one has a case or a stated reason (`tests/data/lodgeit-verdict-cases.json`,
//! attributed in NOTICE).
use australian_tax_calculators::api;
use serde_json::Value;

fn cases() -> Value {
    serde_json::from_str(include_str!("data/lodgeit-verdict-cases.json")).unwrap()
}

/// `a.b` read out of nested objects; `None` for an absent key, so that an
/// expected null cannot pass on a key the result never had.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    value.pointer(&format!("/{}", path.replace('.', "/")))
}

#[test]
fn every_edge_case_has_a_case_or_a_stated_reason() {
    let file = cases();
    assert_eq!(file["synthetic"], true);
    let edge_cases = file["edge_cases"].as_array().unwrap();
    // The 19 in section 4 of the verdict and 6 from its other sections.
    assert_eq!(edge_cases.len(), 25);
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
        let (name, command) = (&case["name"], case["command"].as_str().unwrap());
        let mut input = file["defaults"][command].clone();
        for (key, value) in case["input"].as_object().unwrap() {
            input[key] = value.clone();
        }
        let result = api::evaluate(command, &input);
        if let Some(message) = case["refused"].as_str() {
            assert_eq!(result, Err(message.to_string()), "{name}");
            continue;
        }
        let result = result.unwrap_or_else(|e| panic!("{name}: {e}"));
        for (path, want) in case["expect"].as_object().unwrap() {
            assert_eq!(at(&result, path), Some(want), "{name}: {path}");
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
