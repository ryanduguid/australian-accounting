// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The depreciation worksheet's years 2020-21 to 2024-25 against the Guide to
//! depreciating assets for each of 2021 to 2026, ITAA 1997 ss 40-70, 40-72 and
//! 40-75 and the temporary full expensing and backing business investment
//! rules of IT(TP)A 1997 Subdivisions 40-BA and 40-BB, all read on
//! 23 November 2026 (docs/calculation-evidence.md#depreciation).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const ADDED: [&str; 5] = ["2020-21", "2021-22", "2022-23", "2023-24", "2024-25"];

fn input(overrides: Value) -> Value {
    let mut input = json!({
        "cost": "1500.00", "effective_life": "5", "days": 365, "taxable_use": "1",
        "method": "diminishing_value", "scope_confirmed": true, "year": "2020-21",
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

/// The plain result, checked against `derive` byte for byte.
fn run(overrides: Value) -> Value {
    let input = input(overrides);
    let result = api::evaluate("depreciation", &input).unwrap();
    let doc = api::evaluate(
        "derive",
        &json!({"command": "depreciation", "input": input}),
    )
    .unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn amounts(result: &Value) -> [&str; 3] {
    ["decline_in_value", "deduction", "closing_adjustable_value"]
        .map(|key| result["amounts"][key].as_str().unwrap())
}

fn warned(result: &Value, text: &str) -> bool {
    result["warnings"].to_string().contains(text)
}

/// Guide to depreciating assets 2021: Laura's $1,500 photocopier, first used
/// on 1 July 2020 with a 5-year life, declines $600 by diminishing value and
/// $300 by prime cost in 2020-21, leaving $1,200 under prime cost.
#[test]
fn the_2021_guides_photocopier() {
    let dv = run(json!({}));
    assert_eq!(amounts(&dv), ["600.00", "600.00", "900.00"]);
    let pc = run(json!({"method": "prime_cost"}));
    assert_eq!(amounts(&pc), ["300.00", "300.00", "1200.00"]);
}

/// Guide to depreciating assets 2024: the same photocopier first used on
/// 1 July 2023 is held 366 days over 365: $1,500 x 366 / 365 x 200% / 5 is
/// $602 and leaves $898; by prime cost $301 and $1,199 (whole dollars).
#[test]
fn the_2024_guides_leap_year_photocopier() {
    let dv = run(json!({"year": "2023-24", "days": 366}));
    assert_eq!(amounts(&dv), ["601.64", "601.64", "898.36"]);
    let pc = run(json!({"year": "2023-24", "days": 366, "method": "prime_cost"}));
    assert_eq!(amounts(&pc), ["300.82", "300.82", "1199.18"]);
}

/// Guide to depreciating assets 2023: Paul's $2,000 refrigerator, 10-year
/// life, prime cost, used wholly privately from 1 July 2019: $200 a year, so
/// $1,600 opens 2021-22, which declines $200 and deducts nothing.
#[test]
fn the_2023_guides_private_refrigerator() {
    let r = run(json!({
        "year": "2021-22", "cost": "2000.00", "effective_life": "10", "taxable_use": "0",
        "method": "prime_cost", "opening_adjustable_value": "1600.00",
    }));
    assert_eq!(amounts(&r), ["200.00", "0.00", "1400.00"]);
}

/// The formulas are the same in every year: each added year's result is the
/// 2025-26 one but for its period and its warnings.
#[test]
fn each_added_year_matches_2025_26() {
    for method in ["prime_cost", "diminishing_value"] {
        for opening in [None, Some("1000.00")] {
            let mut facts = json!({"method": method, "days": 200, "taxable_use": "0.6"});
            if let Some(value) = opening {
                facts["opening_adjustable_value"] = json!(value);
            }
            let base = run(with_year(&facts, "2025-26"));
            for year in ADDED {
                let r = run(with_year(&facts, year));
                assert_eq!(r["period"], year);
                assert_eq!(r["amounts"], base["amounts"], "{year} {facts}");
                assert_eq!(r["scope"], base["scope"]);
            }
        }
    }
}

fn with_year(facts: &Value, year: &str) -> Value {
    let mut facts = facts.clone();
    facts["year"] = json!(year);
    facts
}

/// Only 2023-24, 1 July 2023 to 30 June 2024, holds 29 February.
#[test]
fn only_2023_24_takes_366_days() {
    for year in [
        "2020-21", "2021-22", "2022-23", "2024-25", "2025-26", "2026-27",
    ] {
        let refused = api::evaluate("depreciation", &input(json!({"year": year, "days": 366})));
        assert_eq!(
            refused.unwrap_err(),
            format!("days must be an integer from 0 to 365 for {year}.")
        );
    }
    let refused = api::evaluate_coded(
        "depreciation",
        &input(json!({"year": "2023-24", "days": 367})),
    );
    let refused = refused.unwrap_err();
    assert_eq!(refused.code, "input.day_count_invalid");
    assert_eq!(
        refused.message,
        "days must be an integer from 0 to 366 for 2023-24."
    );
}

#[test]
fn the_supported_years_run_from_2020_21_to_2026_27() {
    let periods = api::evaluate("periods", &json!({"kind": "depreciation"})).unwrap();
    let ids: Vec<&str> = periods
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27"
        ]
    );
    for year in ["2019-20", "2027-28"] {
        let refused = api::evaluate_coded("depreciation", &input(json!({"year": year})));
        assert_eq!(refused.unwrap_err().code, "period.unsupported", "{year}");
    }
    let refused = api::evaluate("depreciation", &input(json!({"year": "2027-28"})));
    assert!(refused.unwrap_err().ends_with(
        "supported: 2020-21, 2021-22, 2022-23, 2023-24, 2024-25, 2025-26, 2026-27. 2027-28 has 366 days, and this worksheet has been checked for a year of 366 days only in 2023-24."
    ));
}

/// 2020-21 to 2022-23 are temporary full expensing years, in the start year
/// and later years alike; backing business investment is flagged in a
/// 2020-21 start year only.
#[test]
fn the_full_expensing_and_backing_business_years_are_flagged() {
    for year in [
        "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
    ] {
        for opening in [None, Some("1000.00")] {
            let mut facts = json!({"year": year});
            if let Some(value) = opening {
                facts["opening_adjustable_value"] = json!(value);
            }
            let r = run(facts);
            assert_eq!(
                warned(&r, "temporary full expensing year"),
                year <= "2022-23",
                "{year}"
            );
            assert_eq!(
                warned(&r, "Backing business investment"),
                year == "2020-21" && opening.is_none(),
                "{year} {opening:?}"
            );
        }
    }
}

/// IT(TP)A ss 40-135(3) and 40-180(2): a later prime cost year after either
/// incentive needs s 40-75(3)'s adjusted formula, which is not applied.
#[test]
fn a_later_prime_cost_year_flags_the_adjusted_formula() {
    for year in ["2020-21", "2023-24", "2026-27"] {
        for (method, opening, flagged) in [
            ("prime_cost", Some("1000.00"), true),
            ("prime_cost", None, false),
            ("diminishing_value", Some("1000.00"), false),
        ] {
            let mut facts = json!({"year": year, "method": method});
            if let Some(value) = opening {
                facts["opening_adjustable_value"] = json!(value);
            }
            let r = run(facts);
            assert_eq!(warned(&r, "s 40-75(3)"), flagged, "{year} {method}");
        }
    }
}

#[test]
fn the_catalogue_lists_the_added_years_and_the_leap_day() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["depreciation"];
    assert_eq!(entry["supported_periods"].as_array().unwrap().len(), 7);
    assert_eq!(
        entry["required_inputs"]["days"],
        "integer from 0 to 365, or to 366 in 2023-24"
    );
}
