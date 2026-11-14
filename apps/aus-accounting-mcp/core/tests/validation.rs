// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Argument checks against the Python server on `main`, recorded in
//! fixtures/python-refusals.json: the hosted tools' refusals in draft PR 599's
//! native fixtures, then inputs for the checks those do not reach.
use serde_json::Value;

#[test]
fn each_argument_refusal_is_the_python_servers() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/python-refusals.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 80);
    for case in cases {
        let origin = &case["origin"];
        let name = case["name"].as_str().unwrap();
        let reply = aus_accounting_mcp_core::call(name, &case["arguments"]);
        assert_eq!(reply["isError"], case["isError"], "{origin}: {reply}");
        let Some(expected) = case["text"].as_str() else {
            continue;
        };
        let text = reply["content"][0]["text"].as_str().unwrap();
        if expected.contains(" validation error") {
            assert_eq!(text, expected, "{origin}");
        } else {
            // An input the checks admit reaches the adapter, whose wording is the
            // draft's: compare up to its first sentence.
            let first = expected.split(". ").next().unwrap();
            assert!(text.starts_with(first), "{origin}: {text}");
        }
    }
}
