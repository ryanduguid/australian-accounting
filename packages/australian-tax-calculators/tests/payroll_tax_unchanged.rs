// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Working New South Wales's monthly returns and reconciliation left every
//! other result as it was: `tests/data/payroll_tax_unchanged.json` holds
//! `payroll-tax-monthly` and `payroll-tax` results (or refusal codes) for each
//! other jurisdiction, and New South Wales's annual results, as main gave them
//! at 4df634bb before the change. Every employer here is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

#[test]
fn every_other_jurisdiction_keeps_its_result() {
    let text = include_str!("data/payroll_tax_unchanged.json");
    let cases: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(cases.len(), 43);
    for case in &cases {
        let command = case["command"].as_str().unwrap();
        let got = match api::evaluate_coded(command, &case["input"]) {
            Ok(result) => json!({"ok": result}),
            Err(e) => json!({"err": e.code}),
        };
        assert_eq!(got, case["expected"], "{command} {}", case["input"]);
    }
}
