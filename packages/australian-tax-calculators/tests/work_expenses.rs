// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Work expenses against the ATO's and PCG 2023/1's working from home
//! examples, the ATO's car examples, the self-education reduction examples of
//! the ATO and TR 98/9, every rate for every year, and each refusal, all read
//! on 22 November 2026 (docs/calculation-evidence.md#work-expenses).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const YEARS: [&str; 7] = [
    "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
];

fn run(input: &Value) -> Value {
    api::evaluate("work-expenses", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("work-expenses", input)
        .unwrap_err()
        .code
}

fn with(mut base: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            base.as_object_mut().unwrap().remove(key);
        } else {
            base[key] = value.clone();
        }
    }
    base
}

fn amount<'a>(result: &'a Value, id: &str) -> &'a str {
    result["amounts"][id]
        .as_str()
        .unwrap_or_else(|| panic!("no {id} in {}", result["amounts"]))
}

/// A home office claim with `hours` and no other expenses.
fn home(year: &str, hours: &str) -> Value {
    let mut input = json!({
        "method": "home_office", "reimbursed": false, "allowance_received": false,
        "hours": hours, "energy": "0.00", "phone": "0.00", "internet": "0.00",
        "stationery": "0.00", "cleaning": "0.00", "furniture_decline": "0.00",
        "asset_decline": "0.00", "repairs": "0.00",
        "year": year, "scope_confirmed": true,
    });
    if year < "2022-23" {
        input["dedicated_home_office"] = json!(true);
    }
    input
}

#[test]
fn the_ato_fixed_rate_examples_match() {
    // Keisha: 843 hours x 70c = $590 in 2025-26.
    let keisha = run(&home("2025-26", "843"));
    assert_eq!(keisha["calculation"], "work_expenses");
    assert_eq!(amount(&keisha, "fixed_rate_amount"), "590.10");
    assert_eq!(amount(&keisha, "fixed_rate_return_amount"), "590.00");
    assert_eq!(keisha["rates"]["fixed_rate"], "0.70");
    // Yang: 567 hours and a $299 chair and $250 desk, $945.90, $945 claimed.
    let yang = run(&with(
        home("2025-26", "567"),
        json!({"furniture_decline": "549.00"}),
    ));
    assert_eq!(amount(&yang, "fixed_rate_amount"), "396.90");
    assert_eq!(amount(&yang, "fixed_rate_method"), "945.90");
    assert_eq!(amount(&yang, "fixed_rate_return_amount"), "945.00");
    // Wanda: only the 136 recorded hours, $95.
    let wanda = run(&home("2025-26", "136"));
    assert_eq!(amount(&wanda, "fixed_rate_return_amount"), "95.00");
    // PCG 2023/1 example 1, Gerry: 768 hours in 2024-25, $537.
    let gerry = run(&home("2024-25", "768"));
    assert_eq!(amount(&gerry, "fixed_rate_return_amount"), "537.00");
    // PCG 2023/1 example 8, Piruntha: $392 + $200 + $239.20 + $696.52, $1,527.
    let piruntha = run(&with(
        home("2024-25", "560"),
        json!({"furniture_decline": "439.20", "asset_decline": "696.52"}),
    ));
    assert_eq!(amount(&piruntha, "fixed_rate_amount"), "392.00");
    assert_eq!(amount(&piruntha, "fixed_rate_separate_expenses"), "1135.72");
    assert_eq!(amount(&piruntha, "fixed_rate_method"), "1527.72");
    assert_eq!(amount(&piruntha, "fixed_rate_return_amount"), "1527.00");
}

#[test]
fn the_fixed_rate_covers_its_expenses_and_the_actual_cost_counts_them() {
    let input = with(
        home("2025-26", "843"),
        json!({"energy": "180.00", "phone": "240.00", "internet": "300.00",
            "stationery": "20.00", "cleaning": "50.00", "repairs": "10.00"}),
    );
    let result = run(&input);
    // Energy, phone, internet and stationery are in the rate (PCG 2023/1
    // para 14); cleaning and repairs are claimed beside it (para 15).
    assert_eq!(amount(&result, "fixed_rate_separate_expenses"), "60.00");
    assert_eq!(amount(&result, "fixed_rate_method"), "650.10");
    assert_eq!(amount(&result, "actual_cost_method"), "800.00");
    assert_eq!(result["higher_method"], "actual_cost");
    assert_eq!(
        result["methods_available"],
        json!(["fixed_rate", "actual_cost"])
    );
    assert!(result["amounts"].get("shortcut_method").is_none());
    // A tie names the fixed rate.
    let tie = run(&with(home("2025-26", "100"), json!({"energy": "70.00"})));
    assert_eq!(tie["higher_method"], "fixed_rate");
}

#[test]
fn each_year_has_its_hourly_rates() {
    let rates = [
        ("2020-21", "0.52", Some("0.80")),
        ("2021-22", "0.52", Some("0.80")),
        ("2022-23", "0.67", None),
        ("2023-24", "0.67", None),
        ("2024-25", "0.70", None),
        ("2025-26", "0.70", None),
        ("2026-27", "0.70", None),
    ];
    for (year, fixed, shortcut) in rates {
        let result = run(&home(year, "100"));
        assert_eq!(result["period"], year);
        assert_eq!(result["rates"]["fixed_rate"], fixed, "{year}");
        let expected = format!("{:.2}", fixed.parse::<f64>().unwrap() * 100.0);
        assert_eq!(amount(&result, "fixed_rate_amount"), expected, "{year}");
        match shortcut {
            Some(rate) => {
                assert_eq!(result["rates"]["shortcut_rate"], rate);
                assert_eq!(amount(&result, "shortcut_method"), "80.00");
            }
            None => assert!(result["rates"].get("shortcut_rate").is_none(), "{year}"),
        }
    }
}

#[test]
fn before_2022_23_the_52c_rate_needs_a_dedicated_home_office() {
    let input = with(
        home("2021-22", "1000"),
        json!({"energy": "100.00", "phone": "120.00", "internet": "200.00",
            "stationery": "30.00", "cleaning": "40.00", "furniture_decline": "60.00",
            "asset_decline": "300.00", "repairs": "0.00"}),
    );
    let result = run(&input);
    // Energy, cleaning and furniture are in the 52c; phone, internet,
    // stationery and other assets are added.
    assert_eq!(amount(&result, "fixed_rate_amount"), "520.00");
    assert_eq!(amount(&result, "fixed_rate_separate_expenses"), "650.00");
    assert_eq!(amount(&result, "fixed_rate_method"), "1170.00");
    assert_eq!(amount(&result, "shortcut_method"), "800.00");
    assert_eq!(amount(&result, "actual_cost_method"), "850.00");
    assert_eq!(result["higher_method"], "fixed_rate");
    assert_eq!(
        result["methods_available"],
        json!(["fixed_rate", "shortcut", "actual_cost"])
    );
    let shared = run(&with(input, json!({"dedicated_home_office": false})));
    assert!(shared["amounts"].get("fixed_rate_amount").is_none());
    assert_eq!(
        shared["methods_available"],
        json!(["shortcut", "actual_cost"])
    );
    assert_eq!(shared["higher_method"], "actual_cost");
    assert!(shared["fixed_rate_unavailable"].is_string());
    // From 2022-23 the question is not asked.
    let later = with(
        home("2022-23", "10"),
        json!({"dedicated_home_office": true}),
    );
    assert_eq!(code(&later), "input.field_unknown");
    assert_eq!(
        code(&with(
            home("2021-22", "10"),
            json!({"dedicated_home_office": "remove"})
        )),
        "input.missing_field"
    );
}

#[test]
fn hours_cannot_exceed_the_year() {
    // 2023-24 includes 29 February 2024: 8,784 hours.
    assert!(api::evaluate("work-expenses", &home("2023-24", "8784")).is_ok());
    assert_eq!(
        code(&home("2023-24", "8784.01")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        code(&home("2024-25", "8760.01")),
        "work_expenses.fact_conflict"
    );
}

fn cents(year: &str, kilometres: &str) -> Value {
    json!({
        "method": "car_cents_per_kilometre", "reimbursed": false,
        "allowance_received": false, "owned_or_leased": true,
        "business_kilometres": kilometres, "year": year, "scope_confirmed": true,
    })
}

#[test]
fn johans_cents_per_kilometre_claim_matches() {
    // 2,514 km x 88c = $2,212 in 2025-26.
    let johan = run(&cents("2025-26", "2514"));
    assert_eq!(amount(&johan, "claimed_kilometres"), "2514.00");
    assert_eq!(amount(&johan, "deduction"), "2212.32");
    assert_eq!(amount(&johan, "return_amount"), "2212.00");
    assert_eq!(johan["kilometres_discarded"], false);
}

#[test]
fn each_year_has_its_cents_per_kilometre_rate() {
    let rates = ["0.72", "0.72", "0.78", "0.85", "0.88", "0.88", "0.91"];
    for (year, rate) in YEARS.iter().zip(rates) {
        let result = run(&cents(year, "1000"));
        assert_eq!(result["rates"]["rate_per_kilometre"], rate, "{year}");
        let expected = format!("{:.2}", rate.parse::<f64>().unwrap() * 1000.0);
        assert_eq!(amount(&result, "deduction"), expected, "{year}");
    }
}

#[test]
fn only_the_first_5000_business_kilometres_count() {
    // s 28-25(2)'s example: 5,085 business kilometres, 5,000 claimed.
    let over = run(&cents("2026-27", "5085"));
    assert_eq!(amount(&over, "claimed_kilometres"), "5000.00");
    assert_eq!(amount(&over, "deduction"), "4550.00");
    assert_eq!(over["kilometres_discarded"], true);
    let at = run(&cents("2026-27", "5000"));
    assert_eq!(at["kilometres_discarded"], false);
    assert_eq!(amount(&at, "deduction"), "4550.00");
}

fn log_book(changes: Value) -> Value {
    with(
        json!({
            "method": "car_log_book", "reimbursed": false, "allowance_received": false,
            "owned_or_leased": true, "business_kilometres": "69", "total_kilometres": "100",
            "car_expenses": "17275.33", "log_book_year": "2025-26", "log_book_weeks": 12,
            "year": "2025-26", "scope_confirmed": true,
        }),
        changes,
    )
}

#[test]
fn lanas_log_book_decline_in_value_matches() {
    // $17,275.33 x 69% = $11,919.98.
    let lana = run(&log_book(json!({})));
    assert_eq!(amount(&lana, "business_use_percentage"), "69.00");
    assert_eq!(amount(&lana, "deduction"), "11919.98");
    assert_eq!(amount(&lana, "return_amount"), "11919.00");
    assert_eq!(lana["log_book_valid_to"], "2029-30");
}

#[test]
fn a_log_book_serves_5_years_and_needs_12_weeks() {
    // Kept for 2021-22, it serves 2021-22 to 2025-26 (s 28-115(2)).
    assert!(
        api::evaluate(
            "work-expenses",
            &log_book(json!({"log_book_year": "2021-22"}))
        )
        .is_ok()
    );
    assert_eq!(
        code(&log_book(json!({"log_book_year": "2020-21"}))),
        "work_expenses.log_book_expired"
    );
    assert_eq!(
        code(&log_book(json!({"log_book_year": "2026-27"}))),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        code(&log_book(json!({"log_book_weeks": 11}))),
        "work_expenses.log_book_too_short"
    );
    for bad in [json!("2025-27"), json!("2025"), json!(2025)] {
        assert_eq!(
            code(&log_book(json!({"log_book_year": bad}))),
            "work_expenses.fact_invalid"
        );
    }
    assert_eq!(
        code(&log_book(json!({"log_book_weeks": "12"}))),
        "work_expenses.fact_invalid"
    );
    assert_eq!(
        code(&log_book(json!({"business_kilometres": "100.01"}))),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        code(&log_book(
            json!({"business_kilometres": "0", "total_kilometres": "0"})
        )),
        "work_expenses.fact_conflict"
    );
}

fn education(year: &str, changes: Value) -> Value {
    let mut input = json!({
        "method": "self_education", "reimbursed": false, "allowance_received": false,
        "connection": "maintains_or_improves_skills", "employed_when_incurred": true,
        "category_a": "0.00", "category_b": "0.00", "category_c": "0.00", "category_d": "0.00",
        "year": year, "scope_confirmed": true,
    });
    if year < "2022-23" {
        input["prescribed_course"] = json!(true);
        input["category_e"] = json!("0.00");
    }
    with(input, changes)
}

#[test]
fn the_250_reduction_examples_match_before_2022_23() {
    // Maureen: $290 of category A only, $40 after the $250.
    let maureen = run(&education("2021-22", json!({"category_a": "290.00"})));
    assert_eq!(amount(&maureen, "reduction"), "250.00");
    assert_eq!(amount(&maureen, "deduction"), "40.00");
    assert_eq!(maureen["reduction_applies"], true);
    // TR 98/9 paras 145 to 150, Con: $500 of child minding, $100 of
    // repairs and $100 of cents per kilometre cover the $250; $750.
    let con = run(&education(
        "2020-21",
        json!({"category_a": "350.00", "category_b": "200.00", "category_c": "100.00",
            "category_d": "100.00", "category_e": "500.00"}),
    ));
    assert_eq!(amount(&con, "reduction"), "0.00");
    assert_eq!(amount(&con, "deduction"), "750.00");
    // Paras 151 to 155: a $2,200 computer covers it; $500 plus $400.
    let con = run(&education(
        "2020-21",
        json!({"category_a": "500.00", "category_b": "400.00", "category_e": "2200.00"}),
    ));
    assert_eq!(amount(&con, "deduction"), "900.00");
    // Part covered: $250 - $100 leaves $150 off category A, never below nil.
    let part = run(&education(
        "2021-22",
        json!({"category_a": "120.00", "category_c": "100.00"}),
    ));
    assert_eq!(amount(&part, "reduction"), "150.00");
    assert_eq!(amount(&part, "category_a_after_reduction"), "0.00");
    assert_eq!(amount(&part, "deduction"), "100.00");
}

#[test]
fn the_reduction_ends_for_2022_23_and_never_reaches_a_seminar() {
    let later = run(&education("2022-23", json!({"category_a": "290.00"})));
    assert_eq!(amount(&later, "deduction"), "290.00");
    assert!(later["amounts"].get("reduction").is_none());
    assert_eq!(later["reduction_applies"], false);
    for name in ["prescribed_course", "category_e"] {
        let mut changes = json!({});
        changes[name] = json!(true);
        let asked = education("2022-23", changes);
        assert_eq!(code(&asked), "input.field_unknown", "{name}");
    }
    let seminar = education(
        "2021-22",
        json!({"prescribed_course": false, "category_e": "remove", "category_a": "290.00"}),
    );
    assert_eq!(amount(&run(&seminar), "deduction"), "290.00");
    let with_e = with(seminar, json!({"category_e": "10.00"}));
    assert_eq!(code(&with_e), "input.field_unknown");
}

#[test]
fn self_education_needs_a_connection_to_current_employment() {
    for year in YEARS {
        let deductible = run(&education(
            year,
            json!({"connection": "increases_income", "category_a": "1000.00"}),
        ));
        assert_eq!(deductible["deductible"], true, "{year}");
        // Wendy (new employment), Bert (general only) and Callum's second
        // semester (no longer employed): nothing is deductible.
        for changes in [
            json!({"connection": "new_employment"}),
            json!({"connection": "general_only"}),
            json!({"employed_when_incurred": false}),
        ] {
            let mut changes = changes;
            changes["category_a"] = json!("1000.00");
            let result = run(&education(year, changes.clone()));
            assert_eq!(result["deductible"], false, "{year} {changes}");
            assert_eq!(amount(&result, "deduction"), "0.00");
            assert_eq!(amount(&result, "return_amount"), "0.00");
        }
    }
    assert_eq!(
        code(&education("2025-26", json!({"connection": "yes"}))),
        "work_expenses.fact_invalid"
    );
}

#[test]
fn refusals_name_what_is_out_of_scope_or_wrong() {
    let base = home("2025-26", "100");
    let cases = [
        (json!({"reimbursed": true}), "work_expenses.reimbursed"),
        (json!({"reimbursed": "remove"}), "input.missing_field"),
        (json!({"reimbursed": "unknown"}), "input.flag_invalid"),
        (
            json!({"allowance_received": true}),
            "work_expenses.allowance_received",
        ),
        (
            json!({"method": "occupancy"}),
            "work_expenses.method_invalid",
        ),
        (json!({"occupancy": "100.00"}), "input.field_unknown"),
        (json!({"business_kilometres": "10"}), "input.field_unknown"),
        (json!({"energy": "remove"}), "input.missing_field"),
        (json!({"hours": "-1"}), "input.amount_invalid"),
        (json!({"year": "2019-20"}), "period.unsupported"),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
    ];
    for (changes, expected) in cases {
        assert_eq!(
            code(&with(base.clone(), changes.clone())),
            expected,
            "{changes}"
        );
    }
    let refused =
        api::evaluate_coded("work-expenses", &with(base, json!({"year": "2027-28"}))).unwrap_err();
    assert!(
        refused
            .message
            .contains("No cents per kilometre rate for 2027-28")
    );
    for method in [cents("2025-26", "10"), log_book(json!({}))] {
        let hired = with(method, json!({"owned_or_leased": false}));
        assert_eq!(code(&hired), "work_expenses.car_not_held");
    }
}

#[test]
fn derive_explains_each_method() {
    for input in [
        home("2021-22", "10"),
        home("2025-26", "10"),
        cents("2025-26", "6000"),
        log_book(json!({})),
        education("2020-21", json!({"category_a": "300.00"})),
        education("2025-26", json!({"connection": "general_only"})),
    ] {
        let doc = api::evaluate(
            "derive",
            &json!({"command": "work-expenses", "input": input}),
        )
        .unwrap();
        let steps = doc["steps"].as_array().unwrap();
        assert!(!steps.is_empty());
        assert!(
            steps
                .iter()
                .all(|s| s["authority"].as_str().is_some_and(|a| !a.is_empty()))
        );
    }
}
