// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The residual fringe benefit worksheet against the ATO's worked examples
//! (docs/calculation-evidence.md#fbt-residual), the otherwise deductible rule,
//! the in-house reduction and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// FBT return 2026 instructions, item 23M example 16: a one-tonne utility with
/// $8,000 of operating costs, used 25% privately, with no contribution. A null
/// override removes the input.
fn residual(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "external_notional_value", "notional_value": "8000.00",
        "employee_contribution": "0.00", "income_use": "0.75", "contribution_deduction": "0.00",
        "year_ended": 2026, "scope_confirmed": true,
    });
    let fields = input.as_object_mut().unwrap();
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            fields.remove(k);
        } else {
            fields.insert(k.clone(), v.clone());
        }
    }
    api::evaluate("fbt-residual", &input)
}

/// The employer's guide, 18.4: a television rental store's employee hires a
/// video recorder the public rents for $100, and pays $50.
fn rental(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "in_house_public_price", "lowest_price": "100.00", "notional_value": null,
        "employee_contribution": "50.00", "income_use": "0", "contribution_deduction": "0.00",
        "in_house_reduction_available": "0.00",
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    residual(input)
}

fn amounts(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"].clone()
}

#[test]
fn matches_the_fbt_return_2026_example() {
    // ($8,000 x 25%) - $0 = $2,000, the $6,000 business use a reduction.
    let result = residual(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_residual");
    assert_eq!(result["period"], "year ended 31 March 2026");
    assert_eq!(
        result["amounts"],
        json!({"gross_taxable_value": "8000.00", "employee_contribution_applied": "0.00",
               "excess_employee_contribution": "0.00", "value_after_contribution": "8000.00",
               "gross_deduction": "6000.00", "contribution_deduction_applied": "0.00",
               "notional_deduction": "6000.00", "reduction_applied": "6000.00",
               "taxable_value": "2000.00"})
    );
    assert_eq!(result["rates"], json!({}));
}

#[test]
fn an_in_house_benefit_is_75_percent_of_the_public_price_less_the_reduction() {
    // (75% x $100) - $50 = $25, before the employee's $1,000 reduction.
    let a = amounts(rental(json!({})));
    assert_eq!(a["gross_taxable_value"], "75.00");
    assert_eq!(a["value_after_contribution"], "25.00");
    assert_eq!(a["in_house_reduction_applied"], "0.00");
    assert_eq!(a["taxable_value"], "25.00");
    // s 62 takes the rest of the employee's $1,000, as far as it reaches.
    let a = amounts(rental(json!({"in_house_reduction_available": "1000.00"})));
    assert_eq!(a["in_house_reduction_applied"], "25.00");
    assert_eq!(a["taxable_value"], "0.00");
    let a = amounts(rental(json!({"in_house_reduction_available": "10.00"})));
    assert_eq!(a["taxable_value"], "15.00");
    // With no identical sale to the public, 75% of the notional value, to cents.
    let a = amounts(residual(json!({
        "method": "in_house_other", "notional_value": "0.05", "income_use": "0",
        "in_house_reduction_available": "0.00",
    })));
    assert_eq!(a["gross_taxable_value"], "0.04");
}

#[test]
fn a_salary_packaged_in_house_benefit_takes_its_whole_notional_value() {
    // s 48(aa): the notional value, and no s 62 reduction (s 62(2)).
    let a = amounts(residual(json!({
        "method": "in_house_salary_packaging", "notional_value": "1200.00", "income_use": "0",
    })));
    assert_eq!(a["gross_taxable_value"], "1200.00");
    assert_eq!(a["taxable_value"], "1200.00");
    assert!(a.get("in_house_reduction_applied").is_none());
    assert_eq!(
        residual(json!({"method": "in_house_salary_packaging", "notional_value": "1200.00",
                        "in_house_reduction_available": "1000.00"})),
        Err("in_house_reduction_available is not an input of fbt-residual with method in_house_salary_packaging".to_string())
    );
    // An external benefit bought by the employer, or paid for to another provider.
    let a = amounts(residual(json!({
        "method": "external_expenditure", "employer_expenditure": "300.00", "income_use": "0",
        "notional_value": null,
    })));
    assert_eq!(a["taxable_value"], "300.00");
}

#[test]
fn matches_the_employers_guide_otherwise_deductible_examples() {
    // 18.7: $500 of hired goods, a $250 contribution set without regard to use,
    // 80% used to produce income: GD $400 less RD $200 is $200, and $250 - $200.
    let hired = |deduction: &str| {
        amounts(residual(json!({
            "method": "external_cost_price", "cost_price": "500.00", "notional_value": null,
            "employee_contribution": "250.00", "income_use": "0.8",
            "contribution_deduction": deduction,
        })))
    };
    let a = hired("200.00");
    assert_eq!(a["value_after_contribution"], "250.00");
    assert_eq!(a["gross_deduction"], "400.00");
    assert_eq!(a["notional_deduction"], "200.00");
    assert_eq!(a["taxable_value"], "50.00");
    // The contribution set with regard to 50% intended use limits RD to
    // ($500 x 80%) - ($500 x 50%) = $150, so the reduction is $250, to nil.
    let a = hired("150.00");
    assert_eq!(a["notional_deduction"], "250.00");
    assert_eq!(a["reduction_applied"], "250.00");
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn reproduces_the_guides_vehicle_example_by_the_act() {
    // 18.6, operating cost method: $20,080 with 10% private use. The guide
    // prints $2,080; 10% of $20,080 is $2,008.
    let a = amounts(residual(
        json!({"notional_value": "20080.00", "income_use": "0.9"}),
    ));
    assert_eq!(a["taxable_value"], "2008.00");
}

#[test]
fn a_contribution_or_reduction_never_takes_the_value_below_nil() {
    let a = amounts(residual(json!({"employee_contribution": "9000.00"})));
    assert_eq!(a["employee_contribution_applied"], "8000.00");
    assert_eq!(a["excess_employee_contribution"], "1000.00");
    assert_eq!(a["value_after_contribution"], "0.00");
    assert_eq!(a["notional_deduction"], "6000.00");
    assert_eq!(a["reduction_applied"], "0.00");
    assert_eq!(a["taxable_value"], "0.00");
    // Wholly deductible: nil.
    let a = amounts(residual(json!({"income_use": "1"})));
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let cases = [
        (
            residual(json!({"method": "airline_transport"})),
            "method must be in_house_salary_packaging, in_house_public_price, in_house_other, external_cost_price, external_expenditure or external_notional_value.",
        ),
        (
            residual(
                json!({"employee_contribution": "100.00", "contribution_deduction": "100.01"}),
            ),
            "contribution_deduction cannot exceed employee_contribution: it is the deduction for that consideration (FBTAA s 52(1)(ba), RD).",
        ),
        (
            rental(json!({"in_house_reduction_available": "1000.01"})),
            "in_house_reduction_available must be from 0 to 1000.00: the part of the employee's $1,000 in-house reduction for the FBT year that their other in-house fringe benefits have not used (FBTAA s 62(1)).",
        ),
        (
            residual(json!({"income_use": "1.5"})),
            "income_use must be a finite Decimal fraction from 0 to 1.",
        ),
        (
            residual(
                json!({"method": "in_house_public_price", "in_house_reduction_available": "0.00"}),
            ),
            "notional_value is not an input of fbt-residual with method in_house_public_price",
        ),
        (
            residual(json!({"notional_value": "-1.00"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            residual(json!({"year_ended": 2028})),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
        (
            residual(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
        (
            residual(json!({"car_kilometres": 100})),
            "car_kilometres is not an input of fbt-residual",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result.map(|_| ()), Err(message.to_string()));
    }
    let code = |overrides: Value| {
        let mut input = json!({
            "method": "external_notional_value", "notional_value": "8000.00",
            "employee_contribution": "1.00", "income_use": "0.75", "contribution_deduction": "0.00",
            "year_ended": 2026, "scope_confirmed": true,
        });
        for (k, v) in overrides.as_object().unwrap() {
            input[k] = v.clone();
        }
        api::evaluate_coded("fbt-residual", &input)
            .unwrap_err()
            .code
    };
    assert_eq!(
        code(json!({"contribution_deduction": "2.00"})),
        "fbt.deduction_exceeds_contribution"
    );
    assert_eq!(
        code(json!({"method": "in_house_other", "in_house_reduction_available": "1500.00"})),
        "fbt.in_house_reduction_invalid"
    );
    assert_eq!(
        code(json!({"method": "lottery"})),
        "input.fbt_method_invalid"
    );
    assert_eq!(
        code(json!({"method": "in_house_other"})),
        "input.missing_field"
    );
}

#[test]
fn the_catalogue_names_the_methods_and_its_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_residual"];
    assert_eq!(
        entry["methods"],
        json!([
            "in_house_salary_packaging",
            "in_house_public_price",
            "in_house_other",
            "external_cost_price",
            "external_expenditure",
            "external_notional_value"
        ])
    );
    assert_eq!(entry["source_checked"], "2026-11-15");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-residual", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "2000.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_residual"})).unwrap()[1]["end"],
        "2027-03-31"
    );
}
