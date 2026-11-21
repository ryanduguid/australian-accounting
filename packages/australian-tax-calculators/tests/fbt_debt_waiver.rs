// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The debt waiver fringe benefit worksheet against the ATO's worked examples
//! (docs/calculation-evidence.md#fbt-debt-waiver), and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// FBT return 2026 instructions, item 23D example 9: a $500 debt, principal
/// and interest, waived.
fn waiver(overrides: Value) -> Result<Value, String> {
    let mut input = json!({"amount_waived": "500.00", "year_ended": 2026, "scope_confirmed": true});
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("fbt-debt-waiver", &input)
}

#[test]
fn matches_the_fbt_return_2026_example_and_feeds_fbt_as_type_2() {
    let result = waiver(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_debt_waiver");
    assert_eq!(result["period"], "year ended 31 March 2026");
    assert_eq!(result["amounts"], json!({"taxable_value": "500.00"}));
    // A debt waiver is always type 2: 500 x 1.8868 = 943.40, item 14B $943.
    let fbt = api::evaluate(
        "fbt",
        &json!({"type_one_value": "0.00", "type_two_value": "500.00",
                "year_ended": 2026, "scope_confirmed": true}),
    )
    .unwrap();
    assert_eq!(fbt["amounts"]["return_item_14b"], "943.00");
    assert_eq!(fbt["amounts"]["return_item_16"], "443.21");
    let warnings = result["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.as_str().unwrap().contains("type_two_value"))
    );
}

#[test]
fn matches_the_employers_guide_examples() {
    // Guide 8.2: a $1,000 loan released is $1,000; with $100 of accrued
    // interest also waived, $1,100. The year ended 31 March 2027 alike.
    let one = waiver(json!({"amount_waived": "1000.00"})).unwrap();
    assert_eq!(one["amounts"]["taxable_value"], "1000.00");
    let both = waiver(json!({"amount_waived": "1100.00", "year_ended": 2027})).unwrap();
    assert_eq!(both["amounts"]["taxable_value"], "1100.00");
    assert_eq!(both["period"], "year ended 31 March 2027");
    let nil = waiver(json!({"amount_waived": "0.00"})).unwrap();
    assert_eq!(nil["amounts"]["taxable_value"], "0.00");
}

#[test]
fn refuses_what_s_15_does_not_allow() {
    let cases = [
        (
            waiver(json!({"employee_contribution": "100.00"})),
            "employee_contribution is not an input of fbt-debt-waiver",
        ),
        (
            waiver(json!({"income_use": "1"})),
            "income_use is not an input of fbt-debt-waiver",
        ),
        (
            waiver(json!({"amount_waived": "-1.00"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            waiver(json!({"amount_waived": "1.005"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            waiver(json!({"year_ended": 2025})),
            "Unsupported period 2025; supported: 2026, 2027.",
        ),
        (
            waiver(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result, Err(message.to_string()));
    }
    let input = json!({"year_ended": 2026, "scope_confirmed": true});
    let refused = api::evaluate_coded("fbt-debt-waiver", &input).unwrap_err();
    assert_eq!(refused.code, "input.missing_field");
}

#[test]
fn the_catalogue_describes_the_worksheet() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_debt_waiver"];
    assert_eq!(entry["source_checked"], "2026-11-15");
    assert_eq!(entry["methods"], json!([]));
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-debt-waiver", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "500.00");
}
