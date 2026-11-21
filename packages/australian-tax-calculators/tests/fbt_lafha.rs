// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The living-away-from-home allowance worksheet against the ATO's worked
//! examples (docs/calculation-evidence.md#fbt-lafha), the statutory food
//! amounts and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// FBT return 2026 instructions, item 23G example 12, for the first 40 weeks
/// of the year: $591 a week, $350 for accommodation spent and substantiated,
/// $241 for total food costs, within the Commissioner's reasonable amount.
fn lafha(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "maintains_home", "allowance": "23640.00",
        "accommodation_component": "14000.00", "accommodation_expenses": "14000.00",
        "food_component": "9640.00", "food_expenses": "9640.00", "normal_food_amount": "0.00",
        "adult_days": 280, "child_days": 0, "year_ended": 2026, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("fbt-lafha", &input)
}

/// One week of an allowance the employer's guide, 11.6, describes, for one
/// adult: (allowance, accommodation, food, normal food amount, food expenses).
fn week(facts: [&str; 5]) -> Value {
    let [allowance, accommodation, food, normal, spent] = facts;
    let result = lafha(json!({
        "allowance": allowance, "accommodation_component": accommodation,
        "accommodation_expenses": accommodation, "food_component": food,
        "food_expenses": spent, "normal_food_amount": normal, "adult_days": 7,
    }));
    result.unwrap()["amounts"].clone()
}

#[test]
fn matches_the_fbt_return_2026_example() {
    // 40 weeks: $23,640 - $14,000 - ($9,640 - $1,680) = $1,680.
    let result = lafha(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_lafha");
    assert_eq!(
        result["amounts"],
        json!({"exempt_accommodation_component": "14000.00", "statutory_food_total": "1680.00",
               "applicable_statutory_food_total": "1680.00",
               "exempt_food_component": "7960.00", "taxable_value": "1680.00"})
    );
    // The other 12 weeks are past the 12 month limit, so s 31B values them at
    // the allowance: $7,092, and the year's $8,772 with the 40 weeks.
    let rest = api::evaluate(
        "fbt-lafha",
        &json!({"method": "other", "allowance": "7092.00", "year_ended": 2026,
                "scope_confirmed": true}),
    )
    .unwrap();
    assert_eq!(rest["amounts"], json!({"taxable_value": "7092.00"}));
}

#[test]
fn matches_the_employers_guide_food_examples() {
    // 11.6: an allowance net of normal food costs, $233 spent: nil.
    let a = week(["933.00", "700.00", "233.00", "42.00", "233.00"]);
    assert_eq!(a["applicable_statutory_food_total"], "0.00");
    assert_eq!(a["taxable_value"], "0.00");
    // Total food costs of $320, $260 spent: $320 - $42 = $278, limited to $260,
    // leaving $140.
    let a = week(["900.00", "500.00", "320.00", "0.00", "260.00"]);
    assert_eq!(a["exempt_food_component"], "260.00");
    assert_eq!(a["taxable_value"], "140.00");
    // Additional food costs of $200 within the reasonable amount: $80 left.
    let a = week(["630.00", "350.00", "200.00", "42.00", "200.00"]);
    assert_eq!(a["taxable_value"], "80.00");
    // Total food costs of $300, taken as the $233 reasonable amount spent:
    // $258 limited to $233, leaving $147.
    let a = week(["730.00", "350.00", "300.00", "0.00", "233.00"]);
    assert_eq!(a["exempt_food_component"], "233.00");
    assert_eq!(a["taxable_value"], "147.00");
}

#[test]
fn the_exempt_accommodation_component_is_limited_to_the_expenses() {
    // 11.5: Steve and Helen each receive $450 for accommodation and pay half
    // of a $450 rent, so each exempt component is $225; the $225 over is taxed.
    // Where Steve alone receives it and Helen pays the $450, all of it is exempt.
    let a = week(["450.00", "450.00", "0.00", "0.00", "0.00"]);
    assert_eq!(a["exempt_accommodation_component"], "450.00");
    assert_eq!(a["taxable_value"], "0.00");
    let result = lafha(json!({
        "allowance": "450.00", "accommodation_component": "450.00",
        "accommodation_expenses": "225.00", "food_component": "0.00", "food_expenses": "0.00",
        "adult_days": 7,
    }))
    .unwrap();
    assert_eq!(
        result["amounts"]["exempt_accommodation_component"],
        "225.00"
    );
    assert_eq!(result["amounts"]["exempt_food_component"], "0.00");
    assert_eq!(result["amounts"]["taxable_value"], "225.00");
}

#[test]
fn the_statutory_food_amount_turns_on_age() {
    // s 136(1): $42 a week from 12, $21 under. One adult and 2 children for a
    // week: $42 + $42 = $84; a part week keeps its exact share.
    let a = amounts(lafha(json!({"adult_days": 7, "child_days": 14})));
    assert_eq!(a["statutory_food_total"], "84.00");
    let a = amounts(lafha(json!({"adult_days": 1, "child_days": 0})));
    assert_eq!(a["statutory_food_total"], "6.00");
    let a = amounts(lafha(json!({"adult_days": 0, "child_days": 1})));
    assert_eq!(a["statutory_food_total"], "3.00");
    // A normal food amount above the statutory total leaves nothing to deduct.
    let a = amounts(lafha(json!({"normal_food_amount": "2000.00"})));
    assert_eq!(a["applicable_statutory_food_total"], "0.00");
    assert_eq!(a["exempt_food_component"], "9640.00");
    // No eligible family member's days: the whole food component, as spent.
    let a = amounts(lafha(json!({"adult_days": 0, "food_expenses": "100.00"})));
    assert_eq!(a["exempt_food_component"], "100.00");
}

#[test]
fn a_fly_in_fly_out_employee_takes_the_same_reductions() {
    let a = amounts(lafha(json!({"method": "fly_in_fly_out"})));
    assert_eq!(a["taxable_value"], "1680.00");
    // The Library's ¶3-180: $700 a week, $500 spent on accommodation and $200
    // for total food costs: $200 - $42 = $158 exempt, leaving $42. Its ¶3-160,
    // an employee who keeps a home, is $850 with $650 and $200: also $42.
    let a = amounts(lafha(json!({
        "method": "fly_in_fly_out", "allowance": "700.00",
        "accommodation_component": "500.00", "accommodation_expenses": "500.00",
        "food_component": "200.00", "food_expenses": "200.00", "adult_days": 7,
    })));
    assert_eq!(a["exempt_food_component"], "158.00");
    assert_eq!(a["taxable_value"], "42.00");
    let a = week(["850.00", "650.00", "200.00", "0.00", "200.00"]);
    assert_eq!(a["taxable_value"], "42.00");
}

fn amounts(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"].clone()
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let cases = [
        (
            lafha(json!({"food_component": "9640.01"})),
            "accommodation_component and food_component together cannot exceed allowance: each is the part of the allowance paid for those expenses (FBTAA s 136(1)).",
        ),
        (
            lafha(json!({"adult_days": -1})),
            "adult_days must be an integer from 0 to 1000000.",
        ),
        (
            lafha(json!({"child_days": 1_000_001})),
            "child_days must be an integer from 0 to 1000000.",
        ),
        (
            lafha(json!({"method": "overseas"})),
            "method must be maintains_home, fly_in_fly_out or other.",
        ),
        (
            lafha(json!({"method": "other"})),
            "accommodation_component is not an input of fbt-lafha with method other",
        ),
        (
            lafha(json!({"food_expenses": "1.001"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            lafha(json!({"year_ended": 2028})),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
        (
            lafha(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result.map(|_| ()), Err(message.to_string()));
    }
    let mut input = json!({"method": "maintains_home", "allowance": "1.00",
        "accommodation_component": "1.00", "accommodation_expenses": "0.00",
        "food_component": "1.00", "food_expenses": "0.00", "normal_food_amount": "0.00",
        "adult_days": 0, "child_days": 0, "year_ended": 2027, "scope_confirmed": true});
    let code = |input: &Value| api::evaluate_coded("fbt-lafha", input).unwrap_err().code;
    assert_eq!(code(&input), "fbt_lafha.components_exceed_allowance");
    input["allowance"] = json!("2.00");
    input["adult_days"] = json!(1.5);
    assert_eq!(code(&input), "input.day_count_invalid");
}

#[test]
fn the_catalogue_names_the_methods_and_its_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_lafha"];
    assert_eq!(
        entry["methods"],
        json!(["maintains_home", "fly_in_fly_out", "other"])
    );
    assert_eq!(entry["source_checked"], "2026-11-15");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-lafha", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "1680.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_lafha"})).unwrap()[1]["end"],
        "2027-03-31"
    );
}
