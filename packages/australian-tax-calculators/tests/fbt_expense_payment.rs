// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The expense payment fringe benefit worksheet against the ATO's and the
//! Library's worked examples (docs/calculation-evidence.md#fbt-expense-payment),
//! and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// FBT return 2026 instructions, item 23E example 10: a $1,200 home telephone
/// bill paid for an employee, 60% for business.
fn payment(overrides: Value) -> Result<Value, String> {
    let input = json!({
        "method": "external", "recipients_expenditure": "1200.00", "amount_paid": "1200.00",
        "employee_contribution": "0.00", "income_use": "0.6",
        "recipients_portion_deduction": "0.00", "year_ended": 2026, "scope_confirmed": true,
    });
    evaluate(input, overrides)
}

/// The employer's guide, 9.3: petrol the employer made and sold a retailer for
/// $800, bought by an employee at the $1,000 retail price, $250 reimbursed.
fn in_house(overrides: Value) -> Result<Value, String> {
    let input = json!({
        "method": "in_house_acquired", "recipients_expenditure": "1000.00",
        "amount_paid": "250.00", "employee_contribution": "0.00",
        "arms_length_price": "800.00", "notional_value": "1000.00",
        "in_house_reduction_available": "0.00", "year_ended": 2026, "scope_confirmed": true,
    });
    evaluate(input, overrides)
}

/// `input` with `overrides` applied, an override of null dropping the input.
fn evaluate(mut input: Value, overrides: Value) -> Result<Value, String> {
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            input.as_object_mut().unwrap().remove(k);
        } else {
            input[k] = v.clone();
        }
    }
    api::evaluate("fbt-expense-payment", &input)
}

fn amounts(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"].clone()
}

#[test]
fn matches_the_fbt_return_2026_example() {
    // $1,200 x 60% = $720 otherwise deductible; $1,200 - $720 = $480.
    let result = payment(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_expense_payment");
    assert_eq!(result["period"], "year ended 31 March 2026");
    assert_eq!(
        result["amounts"],
        json!({"gross_taxable_value": "1200.00", "employee_contribution_applied": "0.00",
               "excess_employee_contribution": "0.00", "value_after_contribution": "1200.00",
               "gross_deduction": "720.00", "recipients_portion_deduction_applied": "0.00",
               "notional_deduction": "720.00", "reduction_applied": "720.00",
               "taxable_value": "480.00"})
    );
}

#[test]
fn matches_the_employers_guide_otherwise_deductible_examples() {
    // Guide 9.4: $500 spent, 80% deductible. $250 reimbursed without regard to
    // use: GD $400 less RD ($500 - $250) x 80% = $200 leaves $50.
    let a = amounts(payment(json!({
        "recipients_expenditure": "500.00", "amount_paid": "250.00", "income_use": "0.8",
        "recipients_portion_deduction": "200.00",
    })));
    assert_eq!(a["gross_taxable_value"], "250.00");
    assert_eq!(a["gross_deduction"], "400.00");
    assert_eq!(a["notional_deduction"], "200.00");
    assert_eq!(a["taxable_value"], "50.00");
    // $350 reimbursed for the business part: RD ($500 x 80%) - $350 = $50, ND
    // $350, and the taxable value $350 - $350 = nil.
    let a = amounts(payment(json!({
        "recipients_expenditure": "500.00", "amount_paid": "350.00", "income_use": "0.8",
        "recipients_portion_deduction": "50.00", "year_ended": 2027,
    })));
    assert_eq!(a["notional_deduction"], "350.00");
    assert_eq!(a["reduction_applied"], "350.00");
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn matches_the_library_examples_and_feeds_fbt_as_type_2() {
    // ¶3-240: an associate's $10,000 HELP debt reimbursed. The rule does not
    // reach an associate, so income_use is 0, and the debt has no GST.
    let a = amounts(payment(json!({
        "recipients_expenditure": "10000.00", "amount_paid": "10000.00", "income_use": "0",
    })));
    assert_eq!(a["taxable_value"], "10000.00");
    let fbt = |two: &str| {
        let input = json!({"type_one_value": "0.00", "type_two_value": two,
                           "year_ended": 2026, "scope_confirmed": true});
        api::evaluate("fbt", &input).unwrap()["amounts"].clone()
    };
    // Apply FBTAA s 5B(1C) to the Library's $10,000 taxable value.
    assert_eq!(fbt("10000.00")["type_two_grossed_up"], "18867.92");
    // ¶3-340: $35,000 of mortgage payments and $24,000 of childcare fees
    // reimbursed, $59,000 at the type 2 rate.
    let mortgage = amounts(payment(json!({
        "recipients_expenditure": "35000.00", "amount_paid": "35000.00", "income_use": "0",
    })));
    let childcare = amounts(payment(json!({
        "recipients_expenditure": "24000.00", "amount_paid": "24000.00", "income_use": "0",
    })));
    assert_eq!(mortgage["taxable_value"], "35000.00");
    assert_eq!(childcare["taxable_value"], "24000.00");
    assert_eq!(fbt("59000.00")["return_item_14b"], "111321.00");
}

#[test]
fn matches_the_employers_guide_in_house_example() {
    // Guide 9.3: $800 - ($1,000 - $250) = $50.
    let result = in_house(json!({})).unwrap();
    assert_eq!(
        result["amounts"],
        json!({"gross_taxable_value": "800.00", "recipients_portion": "750.00",
               "recipients_portion_applied": "750.00", "excess_recipients_portion": "0.00",
               "value_after_contribution": "50.00", "in_house_reduction_applied": "0.00",
               "taxable_value": "50.00"})
    );
    // With the employee's $1,000 in-house reduction unused, s 62 takes it to nil.
    let a = amounts(in_house(json!({"in_house_reduction_available": "1000.00"})));
    assert_eq!(a["in_house_reduction_applied"], "50.00");
    assert_eq!(a["taxable_value"], "0.00");
    // A payment to the retailer with a $100 contribution: the recipient's
    // portion is $1,000 - ($250 - $100) = $850, more than the $800 value.
    let a = amounts(in_house(json!({"employee_contribution": "100.00"})));
    assert_eq!(a["recipients_portion"], "850.00");
    assert_eq!(a["excess_recipients_portion"], "50.00");
    assert_eq!(a["taxable_value"], "0.00");
    // Salary packaged: the notional value, and no s 62 reduction.
    let a = amounts(in_house(json!({
        "method": "in_house_salary_packaging", "arms_length_price": null,
        "in_house_reduction_available": null,
    })));
    assert_eq!(a["gross_taxable_value"], "1000.00");
    assert_eq!(a["taxable_value"], "250.00");
    assert!(a.get("in_house_reduction_applied").is_none());
}

#[test]
fn an_in_house_residual_expense_payment_takes_the_residual_rules() {
    // s 22A(2): the guide's 18.4 rental, $100 to the public, the employee
    // charged $50, gives (75% x $100) - $50 = $25. Paid as an expense instead
    // (the employee spends $100, $50 reimbursed), the recipient's portion is
    // $50 and the value is the same $25, as 9.3 says it should be.
    let rental = json!({
        "method": "in_house_residual_public_price", "recipients_expenditure": "100.00",
        "amount_paid": "50.00", "employee_contribution": "0.00", "lowest_price": "100.00",
        "in_house_reduction_available": "0.00", "year_ended": 2026, "scope_confirmed": true,
    });
    let result = evaluate(rental.clone(), json!({})).unwrap();
    assert_eq!(
        result["amounts"],
        json!({"gross_taxable_value": "75.00", "recipients_portion": "50.00",
               "recipients_portion_applied": "50.00", "excess_recipients_portion": "0.00",
               "value_after_contribution": "25.00", "in_house_reduction_applied": "0.00",
               "taxable_value": "25.00"})
    );
    let residual = api::evaluate(
        "fbt-residual",
        &json!({"method": "in_house_public_price", "lowest_price": "100.00",
                "employee_contribution": "50.00", "income_use": "0",
                "contribution_deduction": "0.00", "in_house_reduction_available": "0.00",
                "year_ended": 2026, "scope_confirmed": true}),
    )
    .unwrap();
    assert_eq!(
        residual["amounts"]["taxable_value"],
        result["amounts"]["taxable_value"]
    );
    // The $1,000 in-house reduction (s 62) takes it to nil.
    let a = amounts(evaluate(
        rental.clone(),
        json!({"in_house_reduction_available": "1000.00"}),
    ));
    assert_eq!(a["in_house_reduction_applied"], "25.00");
    assert_eq!(a["taxable_value"], "0.00");
    // No identical benefit sold to the public: 75% of a $90.01 notional value
    // is $67.5075, rounded to $67.51, less the $50 portion.
    let a = amounts(evaluate(
        rental.clone(),
        json!({"method": "in_house_residual_other", "lowest_price": null,
               "notional_value": "90.01"}),
    ));
    assert_eq!(a["gross_taxable_value"], "67.51");
    assert_eq!(a["taxable_value"], "17.51");
    // Salary packaged: the notional value, and no s 62 reduction.
    let a = amounts(evaluate(
        rental,
        json!({"method": "in_house_residual_salary_packaging", "lowest_price": null,
               "notional_value": "100.00", "in_house_reduction_available": null,
               "year_ended": 2027}),
    ));
    assert_eq!(a["gross_taxable_value"], "100.00");
    assert_eq!(a["taxable_value"], "50.00");
    assert!(a.get("in_house_reduction_applied").is_none());
}

#[test]
fn an_in_house_residual_expense_payment_cites_s_22a_2() {
    let input = json!({
        "method": "in_house_residual_public_price", "recipients_expenditure": "100.00",
        "amount_paid": "50.00", "employee_contribution": "0.00", "lowest_price": "100.00",
        "in_house_reduction_available": "10.00", "year_ended": 2026, "scope_confirmed": true,
    });
    let doc = api::evaluate(
        "derive",
        &json!({"command": "fbt-expense-payment", "input": input}),
    )
    .unwrap();
    let authorities: Vec<&str> = doc["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["authority"].as_str().unwrap())
        .collect();
    assert_eq!(
        authorities,
        [
            "FBTAA s 22A(2) and (3)(a), s 48(a) and s 49(a)",
            "FBTAA s 22A(2)(b)(i) for a payment or (ii) for a reimbursement, and s 136(1), recipients portion",
            "FBTAA s 22A(2)(b)",
            "FBTAA s 22A(2)(b)",
            "FBTAA s 22A(2)",
            "FBTAA s 62(1)",
            "FBTAA s 22A(2) and s 62(1)",
        ]
    );
    // The otherwise deductible rule for in-house payments stays uncovered.
    let mut with_use = input;
    with_use["income_use"] = json!("0.5");
    let refusal = api::evaluate_coded("fbt-expense-payment", &with_use).unwrap_err();
    assert_eq!(
        refusal.code,
        "fbt_expense_payment.in_house_deductible_not_covered"
    );
}

/// FBTAA s 136(1), compilation 97, read on 16 November 2026: an in-house
/// property or residual expense payment's provider may be a third party that
/// bought from the employer or an associate, as the guide's 9.3 retailer did.
/// Each provider must carry on a business selling principally to outsiders at
/// or about the provision time, or for a residual benefit the comparison time.
#[test]
fn the_scope_covers_a_provider_who_bought_from_the_employer() {
    let result = in_house(json!({})).unwrap();
    let scope = result["scope"].as_str().unwrap();
    for needle in [
        "for goods from the employer or an associate that, at or about the time it provides \
         them, carries on a business that includes providing identical or similar goods \
         principally to outsiders, or from a third party that acquired them from the employer \
         or an associate when, at or about that time, both carry on such a business (s 136(1), \
         paragraphs (c) and (d) of the definition)",
        "from the employer or an associate that, at or about the time that would be its \
         comparison time were it a residual fringe benefit, carries on a business that includes \
         providing identical or similar benefits principally to outsiders, or from a third party \
         that purchased it from the employer or an associate when, at or about that time, both \
         carry on such a business (s 136(1), paragraphs (b) and (c) of the definition)",
    ] {
        assert!(scope.contains(needle), "{needle}");
    }
}

#[test]
fn a_contribution_above_the_payment_leaves_nil_and_an_excess() {
    // s 23 reduces the payment by the contribution; item 23E shows zero, never
    // a negative value, and an excess reduces no other benefit.
    let a = amounts(payment(json!({
        "recipients_expenditure": "100.00", "amount_paid": "100.00",
        "employee_contribution": "150.00", "income_use": "0",
    })));
    assert_eq!(a["employee_contribution_applied"], "100.00");
    assert_eq!(a["excess_employee_contribution"], "50.00");
    assert_eq!(a["taxable_value"], "0.00");
    // A wholly deductible expense paid in full: ND equals the value, nil.
    let a = amounts(payment(json!({"income_use": "1"})));
    assert_eq!(a["taxable_value"], "0.00");
    // The gross deduction is rounded to cents: 1200.01 x 0.3333 = 399.963333.
    let a = amounts(payment(json!({
        "recipients_expenditure": "1200.01", "amount_paid": "1200.01", "income_use": "0.3333",
    })));
    assert_eq!(a["gross_deduction"], "399.96");
    assert_eq!(a["taxable_value"], "800.05");
}

#[test]
fn refusals_name_the_input_or_rule() {
    let cases = [
        (
            payment(json!({"amount_paid": "1200.01"})),
            "amount_paid cannot exceed recipients_expenditure: it pays or reimburses that expenditure in whole or in part (FBTAA s 20).",
        ),
        (
            payment(json!({"amount_paid": "1000.00", "recipients_portion_deduction": "200.01"})),
            "recipients_portion_deduction cannot exceed the recipient's portion, recipients_expenditure less amount_paid net of employee_contribution: it is the deduction for the expenditure the recipient bore (FBTAA s 24(1)(ba), RD, and s 136(1)).",
        ),
        (
            payment(json!({"method": "in_house_residual"})),
            "method must be external, in_house_salary_packaging, in_house_trade_price, in_house_public_price, in_house_produced_other, in_house_acquired, in_house_other, in_house_residual_salary_packaging, in_house_residual_public_price or in_house_residual_other.",
        ),
        (
            in_house(json!({"income_use": "0.5"})),
            "income_use is not an input of fbt-expense-payment with method in_house_acquired: the otherwise deductible rule for an in-house expense payment (FBTAA s 24(1)(b)(i)) is not covered, so the taxable value is before any reduction it allows.",
        ),
        (
            payment(json!({"in_house_reduction_available": "1000.00"})),
            "in_house_reduction_available is not an input of fbt-expense-payment with method external",
        ),
        (
            in_house(json!({"method": "in_house_public_price"})),
            "arms_length_price is not an input of fbt-expense-payment with method in_house_public_price",
        ),
        (
            in_house(json!({"in_house_reduction_available": "1000.01"})),
            "in_house_reduction_available must be from 0 to 1000.00: the part of the employee's $1,000 in-house reduction for the FBT year that their other in-house fringe benefits have not used (FBTAA s 62(1)).",
        ),
        (
            payment(json!({"income_use": "1.5"})),
            "income_use must be a finite Decimal fraction from 0 to 1.",
        ),
        (
            payment(json!({"year_ended": 2028})),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result, Err(message.to_string()));
    }
    let code = |result: Result<Value, api::Refusal>| result.unwrap_err().code;
    let mut input = json!({"method": "in_house_public_price", "recipients_expenditure": "10.00",
                           "amount_paid": "5.00", "employee_contribution": "0.00",
                           "in_house_reduction_available": "0.00", "year_ended": 2026,
                           "scope_confirmed": true});
    assert_eq!(
        code(api::evaluate_coded("fbt-expense-payment", &input)),
        "input.missing_field"
    );
    input["lowest_price"] = json!("12.00");
    input["in_house_reduction_available"] = json!("1000.01");
    assert_eq!(
        code(api::evaluate_coded("fbt-expense-payment", &input)),
        "fbt.in_house_reduction_invalid"
    );
    input["in_house_reduction_available"] = json!("0.00");
    input["recipients_portion_deduction"] = json!("0.00");
    assert_eq!(
        code(api::evaluate_coded("fbt-expense-payment", &input)),
        "fbt_expense_payment.in_house_deductible_not_covered"
    );
    let input = json!({"method": "external", "recipients_expenditure": "1.00",
                       "amount_paid": "2.00", "employee_contribution": "0.00",
                       "income_use": "0", "recipients_portion_deduction": "0.00",
                       "year_ended": 2026, "scope_confirmed": true});
    assert_eq!(
        code(api::evaluate_coded("fbt-expense-payment", &input)),
        "fbt_expense_payment.payment_exceeds_expenditure"
    );
}

#[test]
fn the_catalogue_describes_the_worksheet() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_expense_payment"];
    assert_eq!(entry["source_checked"], "2026-11-16");
    assert_eq!(entry["methods"][0], "external");
    assert_eq!(entry["methods"].as_array().unwrap().len(), 10);
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-expense-payment", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "480.00");
    // Which gross-up applies turns on the GST credit, and the result says so.
    let warnings = result["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.as_str().unwrap().contains("type_one_value"))
    );
}
