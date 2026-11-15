// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The property fringe benefit worksheet against the ATO's and the Library's
//! worked examples (docs/calculation-evidence.md#fbt-property), and its
//! refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// The employer's guide, 17.3: a desk the employer makes and sells the public
/// for at least $900, bought by an employee for $500.
fn property(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "in_house_public_price", "lowest_price": "900.00",
        "employee_contribution": "500.00", "income_use": "0", "contribution_deduction": "0.00",
        "in_house_reduction_available": "0.00", "year_ended": 2026, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            input.as_object_mut().unwrap().remove(k);
        } else {
            input[k] = v.clone();
        }
    }
    api::evaluate("fbt-property", &input)
}

fn taxable(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"]["taxable_value"].clone()
}

#[test]
fn matches_the_employers_guide_in_house_examples() {
    // Retail goods: (75% x $900) - $500 = $175.
    let result = property(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_property");
    assert_eq!(
        result["amounts"],
        json!({"gross_taxable_value": "675.00", "employee_contribution_applied": "500.00",
               "excess_employee_contribution": "0.00", "value_after_contribution": "175.00",
               "gross_deduction": "0.00", "contribution_deduction_applied": "0.00",
               "notional_deduction": "0.00", "reduction_applied": "0.00",
               "in_house_reduction_applied": "0.00", "taxable_value": "175.00"})
    );
    // Non-retail goods sold to wholesalers for $1,000 less 5% for early
    // payment: the lowest arm's length price, $950.
    let trade = json!({"method": "in_house_trade_price", "lowest_price": "950.00",
                       "employee_contribution": "0.00"});
    assert_eq!(taxable(property(trade)), "950.00");
    // A damaged racquet worth $30, sold to an employee for $5:
    // (75% x $30) - $5 = $17.50.
    let seconds = json!({"method": "in_house_produced_other", "lowest_price": null,
                         "notional_value": "30.00", "employee_contribution": "5.00"});
    assert_eq!(taxable(property(seconds)), "17.50");
    // A television bought for $500 and retailed at $750, sold to an employee
    // for $400: the lesser of $500 and $750, less $400, is $100.
    let bought = json!({"method": "in_house_acquired", "lowest_price": null,
                        "arms_length_price": "500.00", "notional_value": "750.00",
                        "employee_contribution": "400.00"});
    assert_eq!(taxable(property(bought)), "100.00");
}

#[test]
fn matches_the_fbt_return_2026_example_with_the_in_house_reduction() {
    // Item 23K example 14: a $2,000 television and a $1,600 air conditioner,
    // valued at 75% of the selling price as the ATO's example does: $2,700,
    // less $300 paid and the $1,000 in-house reduction, $1,400.
    let result = property(json!({
        "lowest_price": "3600.00", "employee_contribution": "300.00",
        "in_house_reduction_available": "1000.00",
    }))
    .unwrap();
    let a = &result["amounts"];
    assert_eq!(a["gross_taxable_value"], "2700.00");
    assert_eq!(a["in_house_reduction_applied"], "1000.00");
    assert_eq!(a["taxable_value"], "1400.00");
    // Goods with a GST credit are type 1: 1400 x 2.0802 = 2912.28, item 14A $2,912.
    let fbt = api::evaluate(
        "fbt",
        &json!({"type_one_value": "1400.00", "type_two_value": "0.00",
                "year_ended": 2026, "scope_confirmed": true}),
    )
    .unwrap();
    assert_eq!(fbt["amounts"]["return_item_14a"], "2912.00");
    assert_eq!(fbt["amounts"]["return_item_16"], "1368.64");
}

#[test]
fn matches_the_library_in_house_example() {
    // ¶3-260: a $2,000 television bought for $1,800 is (75% x $2,000) - $1,800,
    // nil, with $300 of excess contribution that reduces nothing else.
    let tv = property(
        json!({"lowest_price": "2000.00", "employee_contribution": "1800.00",
                             "in_house_reduction_available": "1000.00"}),
    )
    .unwrap();
    assert_eq!(tv["amounts"]["excess_employee_contribution"], "300.00");
    assert_eq!(tv["amounts"]["in_house_reduction_applied"], "0.00");
    assert_eq!(tv["amounts"]["taxable_value"], "0.00");
    // A $1,200 sound bar bought for $600: $900 - $600 = $300, which the
    // employee's $1,000 reduction takes to nil (Joshua).
    let bar = |available: &str| {
        property(
            json!({"lowest_price": "1200.00", "employee_contribution": "600.00",
                        "in_house_reduction_available": available}),
        )
        .unwrap()["amounts"]
            .clone()
    };
    assert_eq!(bar("0.00")["taxable_value"], "300.00");
    assert_eq!(bar("1000.00")["taxable_value"], "0.00");
    // Jenny's $4,000 desktop, 75% = $3,000, no contribution and no once-only
    // deduction, with the $700 of her reduction the sound bar left: $2,300.
    let desktop = json!({"lowest_price": "4000.00", "employee_contribution": "0.00",
                         "in_house_reduction_available": "700.00"});
    assert_eq!(taxable(property(desktop)), "2300.00");
}

#[test]
fn matches_the_employers_guide_otherwise_deductible_examples() {
    // Guide 17.5: $500 of goods, $250 paid, used 80% for work. The
    // contribution set without regard to use: GD $400 less RD $200, $50.
    let external = |deduction: &str| {
        property(json!({
            "method": "external_notional_value", "lowest_price": null,
            "in_house_reduction_available": null, "notional_value": "500.00",
            "employee_contribution": "250.00", "income_use": "0.8",
            "contribution_deduction": deduction,
        }))
        .unwrap()["amounts"]
            .clone()
    };
    let a = external("200.00");
    assert_eq!(a["value_after_contribution"], "250.00");
    assert_eq!(a["gross_deduction"], "400.00");
    assert_eq!(a["notional_deduction"], "200.00");
    assert_eq!(a["taxable_value"], "50.00");
    assert!(a.get("in_house_reduction_applied").is_none());
    // Set allowing for 50% use: RD ($500 x 80%) - ($500 x 50%) = $150, ND $250.
    let a = external("150.00");
    assert_eq!(a["reduction_applied"], "250.00");
    assert_eq!(a["taxable_value"], "0.00");
    // s 44 does not round GD: $100.03 x 50% = $50.015 off $100.03 leaves
    // $50.015, so $50.02 (a GD rounded first, $50.02, would leave $50.01).
    let a = property(json!({
        "method": "external_notional_value", "lowest_price": null,
        "in_house_reduction_available": null, "notional_value": "100.03",
        "employee_contribution": "0.00", "income_use": "0.5",
    }))
    .unwrap()["amounts"]
        .clone();
    assert_eq!(a["taxable_value"], "50.02");
    // RD above GD but within the contribution: RD applies only up to GD, so
    // ND is nil and the value after the contribution stands.
    let a = property(json!({
        "method": "external_notional_value", "lowest_price": null,
        "in_house_reduction_available": null, "notional_value": "500.00",
        "employee_contribution": "250.00", "income_use": "0.2",
        "contribution_deduction": "200.00",
    }))
    .unwrap()["amounts"]
        .clone();
    assert_eq!(a["gross_deduction"], "100.00");
    assert_eq!(a["contribution_deduction_applied"], "100.00");
    assert_eq!(a["notional_deduction"], "0.00");
    assert_eq!(a["taxable_value"], "250.00");
}

#[test]
fn the_otherwise_deductible_rule_comes_before_the_in_house_reduction() {
    // $500 at cost, half deductible: ND $250 leaves $250, then s 62's $100.
    let a = property(json!({
        "method": "in_house_acquired", "lowest_price": null, "arms_length_price": "500.00",
        "notional_value": "800.00", "employee_contribution": "0.00", "income_use": "0.5",
        "in_house_reduction_available": "100.00", "year_ended": 2027,
    }))
    .unwrap()["amounts"]
        .clone();
    assert_eq!(a["reduction_applied"], "250.00");
    assert_eq!(a["in_house_reduction_applied"], "100.00");
    assert_eq!(a["taxable_value"], "150.00");
    // 75% is shown rounded half up to cents: 33.33 x 0.75 = 24.9975.
    let a = property(json!({"lowest_price": "33.33", "employee_contribution": "0.00"}));
    assert_eq!(a.unwrap()["amounts"]["gross_taxable_value"], "25.00");
    // The external rules take the amount as it is.
    for (method, input) in [
        ("external_cost_price", "cost_price"),
        ("external_expenditure", "employer_expenditure"),
    ] {
        let a = property(json!({"method": method, "lowest_price": null,
                                "in_house_reduction_available": null, input: "120.00",
                                "employee_contribution": "20.00"}));
        assert_eq!(taxable(a), "100.00", "{method}");
    }
    // Salary packaged: the notional value, and no s 62 reduction (s 62(2)).
    let a = property(
        json!({"method": "in_house_salary_packaging", "lowest_price": null,
                            "in_house_reduction_available": null,
                            "notional_value": "300.00", "employee_contribution": "0.00"}),
    );
    assert_eq!(taxable(a), "300.00");
}

#[test]
fn refusals_name_the_input_or_rule() {
    let cases = [
        (
            property(json!({"contribution_deduction": "500.01"})),
            "contribution_deduction cannot exceed employee_contribution: it is the deduction for that consideration (FBTAA s 44(1)(ba), RD).",
        ),
        (
            property(json!({"method": "in_house_airline_transport"})),
            "method must be in_house_salary_packaging, in_house_trade_price, in_house_public_price, in_house_produced_other, in_house_acquired, in_house_other, external_cost_price, external_expenditure or external_notional_value.",
        ),
        (
            property(
                json!({"method": "in_house_salary_packaging", "lowest_price": null,
                            "notional_value": "300.00"}),
            ),
            "in_house_reduction_available is not an input of fbt-property with method in_house_salary_packaging",
        ),
        (
            property(json!({"method": "external_cost_price"})),
            "lowest_price is not an input of fbt-property with method external_cost_price",
        ),
        (
            property(json!({"lowest_price": null})),
            "lowest_price is required with method in_house_public_price",
        ),
        (
            property(json!({"in_house_reduction_available": "1000.01"})),
            "in_house_reduction_available must be from 0 to 1000.00: the part of the employee's $1,000 in-house reduction for the FBT year that their other in-house fringe benefits have not used (FBTAA s 62(1)).",
        ),
        (
            property(json!({"lowest_price": "900.001"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            property(json!({"income_use": "-0.1"})),
            "income_use must be a finite Decimal fraction from 0 to 1.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result, Err(message.to_string()));
    }
    let mut input = json!({"method": "in_house_public_price", "lowest_price": "900.00",
                           "employee_contribution": "10.00", "income_use": "0",
                           "contribution_deduction": "10.01",
                           "in_house_reduction_available": "0.00", "year_ended": 2026,
                           "scope_confirmed": true});
    let code = |input: &Value| api::evaluate_coded("fbt-property", input).unwrap_err().code;
    assert_eq!(code(&input), "fbt.deduction_exceeds_contribution");
    input["contribution_deduction"] = json!("0.00");
    input["in_house_reduction_available"] = json!("1000.01");
    assert_eq!(code(&input), "fbt.in_house_reduction_invalid");
}

#[test]
fn the_catalogue_describes_the_worksheet() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_property"];
    assert_eq!(entry["source_checked"], "2026-11-15");
    assert_eq!(entry["methods"].as_array().unwrap().len(), 9);
    assert_eq!(entry["methods"][8], "external_notional_value");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-property", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "300.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_property"})).unwrap()[1]["id"],
        "2027"
    );
}
