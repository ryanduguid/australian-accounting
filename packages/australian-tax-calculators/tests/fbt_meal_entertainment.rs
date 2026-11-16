// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The meal entertainment worksheet's 50/50 split and 12 week register methods
//! against the ATO's and the Library's worked examples
//! (docs/calculation-evidence.md#fbt-meal-entertainment), and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn meal(input: Value) -> Result<Value, String> {
    let mut full = json!({"year_ended": 2026, "scope_confirmed": true});
    for (k, v) in input.as_object().unwrap() {
        full[k] = v.clone();
    }
    api::evaluate("fbt-meal-entertainment", &full)
}

fn split(expenditure: &str) -> Result<Value, String> {
    meal(json!({"method": "50_50_split", "meal_entertainment_expenditure": expenditure}))
}

fn register(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "12_week_register", "meal_entertainment_expenditure": "177500.00",
        "register_fringe_benefits_value": "400.00", "register_meal_entertainment_value": "1000.00",
        "register_first_year": 2026, "register_years_expenditure": [],
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    meal(input)
}

#[test]
fn the_50_50_split_matches_the_fbt_return_2026_example() {
    // Item 23P example 18: $4,000 x 50% = $2,000.
    let result = split("4000.00").unwrap();
    assert_eq!(result["calculation"], "fbt_meal_entertainment");
    assert_eq!(result["amounts"], json!({"taxable_value": "2000.00"}));
    assert_eq!(result["rates"], json!({"split_share": "0.50"}));
}

#[test]
fn both_methods_match_the_library_example() {
    // ¶3-220: $177,500 of meal entertainment for 2025-26. The 50/50 split gives
    // $88,750; a register showing 40% gives $71,000.
    assert_eq!(
        split("177500.00").unwrap()["amounts"]["taxable_value"],
        "88750.00"
    );
    let result = register(json!({})).unwrap();
    assert_eq!(
        result["amounts"],
        json!({"register_percentage": "40.00", "taxable_value": "71000.00"})
    );
    // The same register in its fifth valid year, 2027 under a 2023 register.
    let years = json!(["177500.00", "177500.00", "177500.00", "177500.00"]);
    let a = register(json!({"year_ended": 2027, "register_first_year": 2023,
                            "register_years_expenditure": years}))
    .unwrap();
    assert_eq!(a["amounts"]["taxable_value"], "71000.00");
}

#[test]
fn the_register_percentage_stays_exact() {
    // 1000 / 3000 is 33.33...%, shown to 2 places but applied unrounded:
    // $900 x 33.333...% = $300, where 33.33% would give $299.97.
    let a = register(json!({
        "meal_entertainment_expenditure": "900.00",
        "register_fringe_benefits_value": "1000.00", "register_meal_entertainment_value": "3000.00",
    }))
    .unwrap();
    assert_eq!(a["amounts"]["register_percentage"], "33.33");
    assert_eq!(a["amounts"]["taxable_value"], "300.00");
    // $21,000.07 x 5,000 / 14,000 is exactly $7,500.025, which rounds half up
    // to $7,500.03; a percentage cut to working precision first gives $7,500.02.
    let a = register(json!({
        "meal_entertainment_expenditure": "21000.07",
        "register_fringe_benefits_value": "5000.00", "register_meal_entertainment_value": "14000.00",
    }))
    .unwrap();
    assert_eq!(a["amounts"]["taxable_value"], "7500.03");
}

#[test]
fn a_register_ceases_after_a_year_more_than_20_percent_above_its_first() {
    // s 37CD(3): a register first valid for 2023, with $1,000 of expenses that
    // year. $1,200 in 2024 is 20% higher, not more, so it stays valid; $1,200.01
    // in 2025 ends its validity at the end of 2025, so not for 2026 or 2027.
    let run = |year: i64, earlier: Value| {
        register(json!({"year_ended": year, "register_first_year": 2023,
                        "register_years_expenditure": earlier}))
    };
    let a = run(2027, json!(["1000.00", "1200.00", "1200.00", "1200.00"])).unwrap();
    assert_eq!(a["amounts"]["taxable_value"], "71000.00");
    let ceased = "The register ceased to be valid at the end of the FBT year ended 31 March 2025: that year's meal entertainment expenses, 1200.01, were more than 20% higher than 1000.00 for its first valid year, 2023 (FBTAA s 37CD(3)). Keep a new register.";
    assert_eq!(
        run(2026, json!(["1000.00", "1200.00", "1200.01"])),
        Err(ceased.to_string())
    );
    assert_eq!(
        run(2027, json!(["1000.00", "1200.00", "1200.01", "900.00"])),
        Err(ceased.to_string())
    );
    // The year being valued may rise by any amount: the register ceases only
    // at the end of that year. $177,500 against $1,000 in its first year.
    let a = register(json!({"register_first_year": 2025,
                            "register_years_expenditure": ["1000.00"]}))
    .unwrap();
    assert_eq!(a["amounts"]["taxable_value"], "71000.00");
    // A first valid year with no expenses makes any later expenses more than
    // 20% higher.
    let nil = run(2026, json!(["0.00", "0.00", "0.01"])).unwrap_err();
    assert!(nil.contains("ended 31 March 2025"), "{nil}");
    let shape = "register_years_expenditure must be a list of decimal strings, the meal entertainment expenses (FBTAA s 37CB(3)) of each FBT year from register_first_year to the year before year_ended, oldest first: 3 here.";
    for earlier in [
        json!(["1000.00", "1200.00"]),
        json!(["1000.00", "1200.00", "1200.00", "1200.00"]),
        json!(["1000.00", "1200.00", 1200]),
        json!("1000.00"),
    ] {
        assert_eq!(run(2026, earlier), Err(shape.to_string()));
    }
    assert_eq!(
        run(2026, json!(["1000.00", "1200.00", "1200.001"])),
        Err("Amounts must be finite non-negative Decimals, at most 2dp and 1e12.".to_string())
    );
    let code = |earlier: Value| {
        let input = json!({"method": "12_week_register", "year_ended": 2026,
            "scope_confirmed": true, "meal_entertainment_expenditure": "1.00",
            "register_fringe_benefits_value": "1.00", "register_meal_entertainment_value": "2.00",
            "register_first_year": 2024, "register_years_expenditure": earlier});
        api::evaluate_coded("fbt-meal-entertainment", &input)
            .unwrap_err()
            .code
    };
    assert_eq!(
        code(json!(["100.00", "120.01"])),
        "fbt_meal_entertainment.register_ceased"
    );
    assert_eq!(
        code(json!([])),
        "fbt_meal_entertainment.register_years_invalid"
    );
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let cases = [
        (
            meal(json!({"method": "actual", "meal_entertainment_expenditure": "1.00"})),
            "method must be 50_50_split or 12_week_register.",
        ),
        (
            meal(
                json!({"method": "50_50_split", "meal_entertainment_expenditure": "1.00",
                        "register_first_year": 2026}),
            ),
            "register_first_year is not an input of fbt-meal-entertainment with method 50_50_split",
        ),
        (
            register(json!({"register_first_year": 2021})),
            "register_first_year must be the FBT year, from 2022 to 2026, the register was first valid for: a register is valid for that year and the 4 after it (FBTAA s 37CD).",
        ),
        (
            register(json!({"register_first_year": 2027})),
            "register_first_year must be the FBT year, from 2022 to 2026, the register was first valid for: a register is valid for that year and the 4 after it (FBTAA s 37CD).",
        ),
        (
            register(json!({"register_fringe_benefits_value": "1000.01"})),
            "register_meal_entertainment_value must be more than nil and at least register_fringe_benefits_value: the register percentage is the share of the register's meal entertainment that is fringe benefits (FBTAA s 37CB(2)).",
        ),
        (
            register(
                json!({"register_fringe_benefits_value": "0.00", "register_meal_entertainment_value": "0.00"}),
            ),
            "register_meal_entertainment_value must be more than nil and at least register_fringe_benefits_value: the register percentage is the share of the register's meal entertainment that is fringe benefits (FBTAA s 37CB(2)).",
        ),
        (
            meal(
                json!({"method": "50_50_split", "meal_entertainment_expenditure": "1.00", "year_ended": 2028}),
            ),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
        (
            meal(json!({"method": "12_week_register", "meal_entertainment_expenditure": "1.00"})),
            "register_fringe_benefits_value is required with method 12_week_register",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result.map(|_| ()), Err(message.to_string()));
    }
    let code = |input| {
        let mut full = json!({"year_ended": 2026, "scope_confirmed": true});
        for (k, v) in Value::as_object(&input).unwrap() {
            full[k] = v.clone();
        }
        api::evaluate_coded("fbt-meal-entertainment", &full)
            .unwrap_err()
            .code
    };
    assert_eq!(
        code(json!({"method": 5, "meal_entertainment_expenditure": "1.00"})),
        "input.fbt_method_invalid"
    );
    assert_eq!(
        code(
            json!({"method": "12_week_register", "meal_entertainment_expenditure": "1.00",
                    "register_fringe_benefits_value": "1.00", "register_meal_entertainment_value": "2.00",
                    "register_first_year": "2026", "register_years_expenditure": []})
        ),
        "fbt.register_not_valid"
    );
    assert_eq!(
        code(
            json!({"method": "12_week_register", "meal_entertainment_expenditure": "1.00",
                    "register_fringe_benefits_value": "3.00", "register_meal_entertainment_value": "2.00",
                    "register_first_year": 2026, "register_years_expenditure": []})
        ),
        "fbt_meal_entertainment.register_values_invalid"
    );
    assert_eq!(
        code(json!({"method": "50_50_split"})),
        "input.missing_field"
    );
}

#[test]
fn the_catalogue_names_both_methods_and_its_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_meal_entertainment"];
    assert_eq!(entry["methods"], json!(["50_50_split", "12_week_register"]));
    assert_eq!(entry["source_checked"], "2026-11-16");
    assert!(entry["optional_inputs"]["register_first_year"].is_string());
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-meal-entertainment", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "2000.00");
}
