// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `review_div7a_loan` against the Python server on `main`, recorded in
//! fixtures/python-div7a.json: the Division 7A conformance cases and remaining
//! terms that cannot hold on a loan the gate passed.
use serde_json::Value;

fn runs() -> Vec<(String, Value, Value)> {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/python-div7a.json")).unwrap();
    fixture["runs"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, run)| {
            (
                name.clone(),
                run["arguments"].clone(),
                run["result"].clone(),
            )
        })
        .collect()
}

fn core(arguments: &Value) -> Value {
    let result = aus_accounting_mcp_core::call("review_div7a_loan", arguments);
    assert_eq!(result["isError"], false, "{result}");
    result["structuredContent"].clone()
}

#[test]
fn a_remaining_term_longer_than_the_maximum_term_is_refused() {
    let (_, arguments, _) = runs()
        .into_iter()
        .find(|(name, ..)| name == "probe_remaining_above_maximum_term")
        .unwrap();
    let repayment = &core(&arguments)["minimum_yearly_repayment"];
    assert_eq!(repayment["verdict"], "REFUSED");
    assert_eq!(
        repayment["reason_codes"],
        serde_json::json!(["REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM"])
    );
    assert_eq!(repayment["myr_required"], Value::Null);
    assert_eq!(repayment["shortfall"], Value::Null);
}

#[test]
fn the_gate_and_repayment_verdicts_codes_and_reasons_match_the_python_server() {
    let runs = runs();
    assert_eq!(runs.len(), 11);
    for (name, arguments, python) in runs {
        let rust = core(&arguments);
        assert_eq!(rust["gate"]["verdict"], python["gate"]["verdict"], "{name}");
        for key in [
            "verdict",
            "reason_codes",
            "reasons",
            "myr_required",
            "shortfall",
        ] {
            assert_eq!(
                rust["minimum_yearly_repayment"][key], python["minimum_yearly_repayment"][key],
                "{name} {key}"
            );
        }
    }
}
