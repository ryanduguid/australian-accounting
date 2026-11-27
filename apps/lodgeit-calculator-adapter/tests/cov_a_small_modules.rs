// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Text escapes, configuration refusals and the account-classifier trial's
//! reading of a malformed result. Offline; every value is fabricated.
use lodgeit_calculator_adapter::client::{Outcome, Status};
use lodgeit_calculator_adapter::config;
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::pyjson::Json;
use lodgeit_calculator_adapter::pytext::{code_points, push_code_point, repr_code_points};
use lodgeit_calculator_adapter::pyvalue::points_lossy;
use lodgeit_calculator_adapter::trials::fano::{Line, interpret};

#[test]
fn os_text_carries_lone_surrogates_and_the_escape_itself_one_to_one() {
    let escape = 0x10_F7FF;
    let mut text = String::new();
    for cp in [u32::from('a'), 0xd800, escape, 0xdfff] {
        push_code_point(&mut text, cp);
    }
    assert_eq!(
        code_points(&text).collect::<Vec<_>>(),
        [u32::from('a'), 0xd800, escape, 0xdfff]
    );
    // An escape followed by an ordinary character stands for itself.
    let bare: String = [char::from_u32(escape).unwrap(), 'b'].into_iter().collect();
    assert_eq!(
        code_points(&bare).collect::<Vec<_>>(),
        [escape, u32::from('b')]
    );
}

#[test]
fn repr_escapes_backslashes_and_whitespace_controls() {
    let points: Vec<u32> = "\\\t\n\r".chars().map(u32::from).collect();
    assert_eq!(repr_code_points(&points), r"'\\\t\n\r'");
}

fn env_with(name: &'static str, value: &'static str) -> impl Fn(&str) -> Option<String> {
    move |key: &str| (key == name).then(|| value.to_owned())
}

#[test]
fn an_allowlist_of_only_separators_is_empty_and_refuses_every_host() {
    let config = config::from_environment(env_with("LODGEIT_ADAPTER_ALLOWED_HOSTS", " , "))
        .expect("an empty allowlist is a valid configuration");
    assert!(config.allowed_hosts.is_empty());
    let refusal = config
        .check_url("https://example.invalid/calculators", false)
        .expect_err("no host is allowed");
    assert!(
        points_lossy(&refusal.text)
            .ends_with("host example.invalid is not on the configured allowlist (empty)"),
        "{}",
        points_lossy(&refusal.text)
    );
}

#[test]
fn a_nan_attempt_count_raises_pythons_value_error() {
    let error = config::from_environment(env_with("LODGEIT_ADAPTER_MAX_ATTEMPTS", "nan"))
        .expect_err("int(nan) raises");
    assert_eq!(error.class, "ValueError");
    assert_eq!(
        points_lossy(&error.text),
        "cannot convert float NaN to integer"
    );
}

#[test]
fn the_process_environment_config_is_off_unless_switched_on() {
    let enabled = std::env::var_os("LODGEIT_ADAPTER_ENABLED").is_some_and(|v| v == "1");
    if let Ok(config) = config::from_process_environment() {
        assert_eq!(config.enabled, enabled);
    }
}

#[test]
fn a_computed_result_that_is_not_an_object_carries_no_results() {
    let zero = Decimal::parse_exact("0").expect("a literal");
    let lines = [Line {
        description: "Bank account, operating".to_owned(),
        source_topology: "current_assets".to_owned(),
        amount: Decimal::parse_exact("100.00").expect("a literal"),
        predicted_code: "sbrm_0000".to_owned(),
        confidence: zero,
    }];
    let outcome = Outcome {
        status: Status::Computed,
        calculator: None,
        period: None,
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: Some(200),
        result: Json::Array(vec![]),
        values: vec![],
        manifest: Json::Null,
        advisory: Json::Null,
        upstream_refusal_class: Json::Null,
        findings: vec![],
        observed_at: "2026-01-01T00:00:00+00:00".to_owned(),
        elapsed_ms: None,
        contract_snapshot: Json::Null,
        contract_sha256: String::new(),
        base_url: None,
    };
    let (status, suggestions, notes) = interpret(&lines, &outcome);
    assert_eq!(status, "CONTRACT_FAILURE");
    assert!(suggestions.is_empty());
    let notes: Vec<String> = notes.iter().map(|n| points_lossy(n)).collect();
    assert_eq!(notes, ["the response carries no results array"]);
}
