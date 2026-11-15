// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The loan fringe benefit worksheet against the ATO's and the Library's
//! worked examples (docs/calculation-evidence.md#fbt-loan), and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// FBT return 2026 instructions, item 23C example 8: $20,000 lent for the
/// whole year at no interest, so a daily balance of $20,000 on each of 365 days.
fn loan(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "daily_balance_total": "7300000.00", "interest_accrued": "0.00", "income_use": "0",
        "accrued_interest_deduction": "0.00", "year_ended": 2026, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("fbt-loan", &input)
}

fn amounts(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"].clone()
}

#[test]
fn matches_the_fbt_return_2026_example() {
    // $20,000 x 8.62% = $1,724, with no reductions.
    let result = loan(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_loan");
    assert_eq!(result["period"], "year ended 31 March 2026");
    assert_eq!(
        result["amounts"],
        json!({"notional_interest": "1724.00", "interest_accrued_applied": "0.00",
               "gross_taxable_value": "1724.00", "gross_deduction": "0.00",
               "accrued_interest_deduction_applied": "0.00", "notional_deduction": "0.00",
               "reduction_applied": "0.00", "taxable_value": "1724.00"})
    );
    assert_eq!(
        result["rates"],
        json!({"statutory_interest_rate": "0.0862", "days_in_year": "365"})
    );
}

#[test]
fn matches_the_library_loan_example_and_feeds_fbt_as_type_2() {
    // ¶3-100: $100,000 for 2025-26 at 3.45%, 40% spent on shares. Notional
    // interest $8,620 less $3,450 accrued is $5,170; GD $3,448 less RD $1,380
    // is the $2,068 notional deduction, for a reduced taxable value of $3,102.
    let a = amounts(loan(json!({
        "daily_balance_total": "36500000.00", "interest_accrued": "3450.00",
        "income_use": "0.4", "accrued_interest_deduction": "1380.00",
    })));
    assert_eq!(
        a,
        json!({"notional_interest": "8620.00", "interest_accrued_applied": "3450.00",
               "gross_taxable_value": "5170.00", "gross_deduction": "3448.00",
               "accrued_interest_deduction_applied": "1380.00", "notional_deduction": "2068.00",
               "reduction_applied": "2068.00", "taxable_value": "3102.00"})
    );
    // A loan is a type 2 benefit: $3,102 x 1.8868 is the Library's $5,853, and
    // the FBT on it $2,750.91, which the Library rounds to $2,751.
    let fbt = api::evaluate(
        "fbt",
        &json!({"type_one_value": "0.00", "type_two_value": a["taxable_value"],
                "year_ended": 2026, "scope_confirmed": true}),
    )
    .unwrap();
    assert_eq!(fbt["amounts"]["return_item_14b"], "5853.00");
    assert_eq!(fbt["amounts"]["return_item_16"], "2750.91");
}

#[test]
fn reruns_the_employers_guide_examples_at_the_supported_rates() {
    // Guide 8.5: $50,000 at 5% for the year. At 5.65% (2016-17) the guide
    // gets $325; at 8.62% the notional interest is $4,310, and at 8.27% $4,135.
    let a = amounts(loan(json!({
        "daily_balance_total": "18250000.00", "interest_accrued": "2500.00",
    })));
    assert_eq!(a["notional_interest"], "4310.00");
    assert_eq!(a["taxable_value"], "1810.00");
    let a = amounts(loan(json!({
        "daily_balance_total": "18250000.00", "interest_accrued": "2500.00", "year_ended": 2027,
    })));
    assert_eq!(a["notional_interest"], "4135.00");
    assert_eq!(a["taxable_value"], "1635.00");
    // Guide 8.8, rate set without regard to use: $50,000 at 4%, 60% applied to
    // income. TV 4310 - 2000 = 2310; GD 4310 x 60% = 2586; RD 2000 x 60% = 1200.
    let a = amounts(loan(json!({
        "daily_balance_total": "18250000.00", "interest_accrued": "2000.00",
        "income_use": "0.6", "accrued_interest_deduction": "1200.00",
    })));
    assert_eq!(a["gross_taxable_value"], "2310.00");
    assert_eq!(a["gross_deduction"], "2586.00");
    assert_eq!(a["notional_deduction"], "1386.00");
    assert_eq!(a["taxable_value"], "924.00");
    // Guide 8.8, rate set with regard to use: RD is limited to the deduction
    // without the allowance less the allowance, 2586 - 2155 = 431.
    let a = amounts(loan(json!({
        "daily_balance_total": "18250000.00", "interest_accrued": "2000.00",
        "income_use": "0.6", "accrued_interest_deduction": "431.00",
    })));
    assert_eq!(a["notional_deduction"], "2155.00");
    assert_eq!(a["taxable_value"], "155.00");
}

#[test]
fn the_otherwise_deductible_rule_stops_at_nil() {
    // A loan wholly used to produce income: ND $4,310 exceeds TV $2,310, and
    // the guide says there can be no negative figures.
    let a = amounts(loan(json!({
        "daily_balance_total": "18250000.00", "interest_accrued": "2000.00", "income_use": "1",
    })));
    assert_eq!(a["notional_deduction"], "4310.00");
    assert_eq!(a["reduction_applied"], "2310.00");
    assert_eq!(a["taxable_value"], "0.00");
    // An RD above GD leaves no notional deduction: s 19 needs one above nil.
    let a = amounts(loan(json!({
        "daily_balance_total": "18250000.00", "interest_accrued": "4000.00",
        "income_use": "0.1", "accrued_interest_deduction": "1000.00",
    })));
    assert_eq!(a["gross_deduction"], "431.00");
    assert_eq!(a["accrued_interest_deduction_applied"], "431.00");
    assert_eq!(a["notional_deduction"], "0.00");
    assert_eq!(a["taxable_value"], "310.00");
}

#[test]
fn interest_at_or_above_the_statutory_rate_gives_no_value() {
    // s 18: only the amount by which notional interest exceeds the interest
    // accrued.
    let a = amounts(loan(json!({"interest_accrued": "1800.00"})));
    assert_eq!(a["interest_accrued_applied"], "1724.00");
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn a_part_year_balance_accrues_by_the_day() {
    // $20,000 owed for 182 days: 3640000 x 0.0862 / 365 = 313768 / 365
    // = 859.6383...
    let a = amounts(loan(json!({"daily_balance_total": "3640000.00"})));
    assert_eq!(a["notional_interest"], "859.64");
    assert_eq!(a["taxable_value"], "859.64");
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let cases = [
        (
            loan(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
        (
            loan(json!({"year_ended": 2028})),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
        (
            loan(json!({"income_use": "1.01"})),
            "income_use must be a finite Decimal fraction from 0 to 1.",
        ),
        (
            loan(json!({"interest_accrued": "100.00", "accrued_interest_deduction": "100.01"})),
            "accrued_interest_deduction cannot exceed interest_accrued: it is the deduction for that interest (FBTAA s 19(1)(ba), RD).",
        ),
        (
            loan(json!({"daily_balance_total": "-1.00"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            loan(json!({"loan_balance": "20000.00"})),
            "loan_balance is not an input of fbt-loan",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result, Err(message.to_string()));
    }
    let mut input = json!({"daily_balance_total": "7300000.00", "income_use": "0",
                           "accrued_interest_deduction": "0.00", "year_ended": 2026,
                           "scope_confirmed": true});
    assert_eq!(
        api::evaluate_coded("fbt-loan", &input).unwrap_err().code,
        "input.missing_field"
    );
    input["interest_accrued"] = json!("0.00");
    input.as_object_mut().unwrap().remove("year_ended");
    assert_eq!(
        api::evaluate("fbt-loan", &input),
        Err("year_ended is required".to_string())
    );
    let code =
        |overrides| {
            api::evaluate_coded("fbt-loan", &{
        let mut input = json!({"daily_balance_total": "7300000.00", "interest_accrued": "1.00",
            "income_use": "0", "accrued_interest_deduction": "2.00", "year_ended": 2026,
            "scope_confirmed": true});
        for (k, v) in Value::as_object(&overrides).unwrap() {
            input[k] = v.clone();
        }
        input
    }).unwrap_err().code
        };
    assert_eq!(code(json!({})), "fbt_loan.deduction_exceeds_interest");
    assert_eq!(code(json!({"income_use": 1})), "input.fraction_invalid");
}

#[test]
fn the_catalogue_and_periods_describe_the_worksheet() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_loan"];
    assert_eq!(entry["required_inputs"]["year_ended"], "integer");
    assert_eq!(entry["source_checked"], "2026-11-15");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-loan", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "1724.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_loan"})).unwrap(),
        json!([
            {"id": "2026", "basis": "FBT year", "start": "2025-04-01", "end": "2026-03-31",
             "arguments": {"year_ended": 2026}},
            {"id": "2027", "basis": "FBT year", "start": "2026-04-01", "end": "2027-03-31",
             "arguments": {"year_ended": 2027}},
        ])
    );
    // A loan is always a type 2 benefit, and the result says so.
    let warnings = result["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.as_str().unwrap().contains("type_two_value"))
    );
}
