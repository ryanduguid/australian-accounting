// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use aus_accounting_mcp_core::{call, contracts};
use serde_json::Value;
use std::collections::BTreeSet;

#[test]
fn current_language_neutral_conformance_cases_execute_through_the_dispatcher() {
    let contract: Value =
        serde_json::from_str(include_str!("../../conformance/cases.json")).unwrap();
    assert_eq!(contract["schema_version"], 2);
    let cases = contract["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 32);
    let mut ids = BTreeSet::new();
    for case in cases {
        assert!(ids.insert(case["id"].as_str().unwrap()));
        let name = case["tool"].as_str().unwrap();
        assert!(contracts::tool(name).is_some());
        assert_ne!(
            case.get("expect").is_some(),
            case.get("expect_error_contains").is_some()
        );
        let reply = call(name, &case["arguments"]);
        if let Some(message) = case["expect_error_contains"].as_str() {
            assert_eq!(reply["isError"], true, "{}: {reply}", case["id"]);
            assert!(
                reply["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .contains(message),
                "{}: {reply}",
                case["id"]
            );
        } else {
            assert_eq!(reply["isError"], false, "{}: {reply}", case["id"]);
            for (pointer, expected) in case["expect"].as_object().unwrap() {
                assert!(pointer.starts_with('/'));
                assert_eq!(
                    reply["structuredContent"].pointer(pointer),
                    Some(expected),
                    "{} {pointer}",
                    case["id"]
                );
            }
        }
    }
}
