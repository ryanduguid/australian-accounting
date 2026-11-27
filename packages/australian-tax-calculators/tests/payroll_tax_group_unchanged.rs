// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Working New South Wales, Tasmania and the Northern Territory's grouping
//! left every other jurisdiction's result as it was:
//! `tests/data/payroll_tax_group_unchanged.json` holds `payroll-tax-group`
//! results (or refusal codes) for Victoria, Queensland, Western Australia,
//! South Australia and the ACT as main gave them at 7f3c3eb1 before the
//! change. The `scope` and `sources` texts are left out of the comparison:
//! the change rewords them to name the 3 jurisdictions added.
//! Every person here is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// The result without the texts the change rewords.
fn worked(mut result: Value) -> Value {
    if let Some(fields) = result.get_mut("ok").and_then(Value::as_object_mut) {
        fields.remove("scope");
        fields.remove("sources");
    }
    result
}

#[test]
fn every_other_jurisdiction_keeps_its_group_result() {
    let text = include_str!("data/payroll_tax_group_unchanged.json");
    let cases: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(cases.len(), 49);
    for case in &cases {
        let got = match api::evaluate_coded("payroll-tax-group", &case["input"]) {
            Ok(result) => json!({"ok": result}),
            Err(e) => json!({"err": e.code}),
        };
        assert_eq!(
            worked(got),
            worked(case["expected"].clone()),
            "{}",
            case["input"]
        );
    }
}
