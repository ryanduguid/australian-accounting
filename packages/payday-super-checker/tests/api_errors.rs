// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The JSON commands refuse input of the wrong shape with a message naming
//! the key, and decode base64 with its whole alphabet.

use payday_super_checker::api::evaluate;
use serde_json::{Value, json};

fn refused(command: &str, input: Value) -> String {
    evaluate(command, &input).expect_err(command)
}

#[test]
fn input_that_is_not_an_object_or_a_known_command_is_refused() {
    assert_eq!(
        refused("parse-amount", json!([])),
        "the JSON input must be an object"
    );
    assert_eq!(
        refused("no-such-command", json!({})),
        "unknown command no-such-command"
    );
}

#[test]
fn keys_of_the_wrong_type_are_named() {
    let cases = [
        (
            "load-profiles",
            json!({"role": 5}),
            "role must be a string or null",
        ),
        (
            "detect-profile",
            json!({"headers": "a", "role": "payroll"}),
            "headers must be a list of strings",
        ),
        (
            "detect-profile",
            json!({"headers": [1], "role": "payroll"}),
            "headers must be a list of strings",
        ),
        (
            "render-csv",
            json!({"lines": 5}),
            "lines must be a list of contribution lines",
        ),
        (
            "compute-due",
            json!({"line": 5}),
            "a contribution line must be an object",
        ),
        (
            "join",
            json!({"payroll_rows": 5, "super_rows": []}),
            "payroll_rows must be a list",
        ),
        (
            "parse-report-snapshot",
            json!({"data_b64": "@@@@"}),
            "data_b64 must be base64",
        ),
    ];
    for (command, input, message) in cases {
        assert_eq!(refused(command, input), message, "{command}");
    }
}

#[test]
fn a_line_with_events_that_are_not_a_list_is_refused() {
    let line = json!({
        "employee_id": "E1",
        "qe_day": "2026-07-28",
        "sg_amount": "100.00",
        "events": 5,
    });
    assert_eq!(
        refused("compute-due", json!({"line": line})),
        "events must be a list of events"
    );
}

#[test]
fn a_holiday_whose_provisional_flag_is_not_a_boolean_is_refused() {
    let holiday = json!({
        "day": "2026-01-01",
        "name": "New Year's Day",
        "jurisdictions": ["NAT"],
        "provisional": "no",
    });
    let input = json!({"calendar": {"holidays": [holiday]}});
    assert_eq!(
        refused("load-calendar", input),
        "provisional must be a boolean"
    );
}

#[test]
fn base64_with_plus_and_slash_decodes_like_the_text_it_encodes() {
    // "fn5+Pz8/" is base64 for "~~~???".
    let decoded = evaluate("parse-report-snapshot", &json!({"data_b64": "fn5+Pz8/"}));
    let direct = evaluate("parse-report-snapshot", &json!({"text": "~~~???"}));
    assert_eq!(decoded, direct);
}

#[test]
fn a_file_that_cannot_be_opened_reports_the_os_error() {
    let message = refused("read-payroll", json!({"path": "no-such-payroll.csv"}));
    assert!(message.contains("no-such-payroll.csv"), "{message}");
}
