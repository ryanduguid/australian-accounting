// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The tax-exempt body entertainment worksheet against the ATO's worked
//! examples (docs/calculation-evidence.md#fbt-tax-exempt-body-entertainment)
//! and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "fbt-tax-exempt-body-entertainment";

/// FBT return 2026 instructions, item 23L example 15: a council's Christmas
/// function for employees and their spouses, valued at $5,000.
fn entertainment(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "actual", "entertainment_expenditure": "5000.00",
        "year_ended": 2026, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate(COMMAND, &input)
}

/// The employer's guide, 15.9: Mary entertains 3 clients at a restaurant and
/// is reimbursed; 1 head of 4 is hers.
fn per_head(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "per_head", "entertainment_expenditure": "400.00",
        "employee_heads": 1, "total_heads": 4,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    entertainment(input)
}

#[test]
fn matches_the_fbt_return_2026_example() {
    // s 39: the expenditure attributable to the entertainment, with no
    // contribution or reduction (item 23L).
    let result = entertainment(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_tax_exempt_body_entertainment");
    assert_eq!(result["amounts"], json!({"taxable_value": "5000.00"}));
    assert_eq!(result["rates"], json!({}));
}

#[test]
fn the_per_head_basis_matches_the_employers_guide_example() {
    // 25% of the $400 reimbursed.
    let result = per_head(json!({})).unwrap();
    assert_eq!(result["amounts"], json!({"taxable_value": "100.00"}));
    // A share that does not divide evenly is rounded to cents.
    let a = per_head(json!({"entertainment_expenditure": "100.00", "total_heads": 3})).unwrap();
    assert_eq!(a["amounts"]["taxable_value"], "33.33");
    let a = per_head(json!({"employee_heads": 0})).unwrap();
    assert_eq!(a["amounts"]["taxable_value"], "0.00");
    let a = per_head(json!({"employee_heads": 4})).unwrap();
    assert_eq!(a["amounts"]["taxable_value"], "400.00");
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let heads = "employee_heads and total_heads must be JSON integers, total_heads from 1 to 1000000 and employee_heads from 0 to total_heads: the employees and associates entertained, out of everyone entertained.";
    let cases = [
        (per_head(json!({"employee_heads": 5})), heads),
        (
            per_head(json!({"total_heads": 0, "employee_heads": 0})),
            heads,
        ),
        (per_head(json!({"employee_heads": -1})), heads),
        (per_head(json!({"total_heads": 1_000_001})), heads),
        (per_head(json!({"total_heads": "4"})), heads),
        (
            entertainment(json!({"employee_heads": 1})),
            "employee_heads is not an input of fbt-tax-exempt-body-entertainment with method actual",
        ),
        (
            entertainment(json!({"employee_contribution": "1.00"})),
            "employee_contribution is not an input of fbt-tax-exempt-body-entertainment",
        ),
        (
            entertainment(json!({"method": "meal_entertainment"})),
            "method must be actual or per_head.",
        ),
        (
            entertainment(json!({"entertainment_expenditure": "-5.00"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            entertainment(json!({"year_ended": 2028})),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
        (
            entertainment(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result.map(|_| ()), Err(message.to_string()));
    }
    let input = json!({"method": "per_head", "entertainment_expenditure": "1.00",
        "employee_heads": 2, "total_heads": 1, "year_ended": 2027, "scope_confirmed": true});
    let code = api::evaluate_coded(COMMAND, &input).unwrap_err().code;
    assert_eq!(code, "fbt_tax_exempt_body_entertainment.heads_invalid");
}

#[test]
fn the_catalogue_names_the_methods_and_its_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_tax_exempt_body_entertainment"];
    assert_eq!(entry["methods"], json!(["actual", "per_head"]));
    assert_eq!(entry["source_checked"], "2026-11-15");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "5000.00");
    assert_eq!(
        api::evaluate(
            "periods",
            &json!({"kind": "fbt_tax_exempt_body_entertainment"})
        )
        .unwrap()[1]["end"],
        "2027-03-31"
    );
}
