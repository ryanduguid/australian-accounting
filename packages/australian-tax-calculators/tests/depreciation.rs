// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Later income years of the depreciation worksheet against the ATO's
//! 'Prime cost (straight line) and diminishing value methods' page (last
//! updated 28 August 2026) and ITAA 1997 ss 40-70, 40-72 and 40-75 in the
//! ATO legal database, all read on 13 November 2026 at 18:19 AEDT. A later
//! year's diminishing value base is its opening adjustable value (s 40-70(1),
//! applied by s 40-72(1)); prime cost keeps cost (s 40-75(1)); either decline
//! is limited to the opening adjustable value (ss 40-72(3) and 40-75(7)(b)).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(overrides: Value) -> Value {
    let mut input = json!({
        "cost": "80000.00", "effective_life": "5", "days": 365, "taxable_use": "1",
        "method": "diminishing_value", "scope_confirmed": true, "year": "2026-27",
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

fn run(overrides: Value) -> Value {
    api::evaluate("depreciation", &input(overrides)).unwrap()
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded("depreciation", &input(overrides))
        .unwrap_err()
        .code
}

/// Decline in value, deduction and closing adjustable value.
fn amounts(result: &Value) -> [&str; 3] {
    ["decline_in_value", "deduction", "closing_adjustable_value"]
        .map(|key| result["amounts"][key].as_str().unwrap())
}

/// The ATO's diminishing value example: $80,000, 5-year life, held all year.
/// Year 1 $32,000; year 2 base $48,000, claim $19,200; year 3 base $28,800,
/// claim $11,520; year 4 base $17,280, claim $6,912.
#[test]
fn diminishing_value_follows_the_atos_later_years() {
    let first = run(json!({}));
    assert_eq!(amounts(&first), ["32000.00", "32000.00", "48000.00"]);
    let mut opening = "48000.00".to_string();
    for expected in [
        ["19200.00", "19200.00", "28800.00"],
        ["11520.00", "11520.00", "17280.00"],
        ["6912.00", "6912.00", "10368.00"],
    ] {
        let year = run(json!({"opening_adjustable_value": opening}));
        assert_eq!(amounts(&year), expected);
        opening = expected[2].to_string();
    }
}

/// The same page: at 40% private use 'all figures above would be multiplied
/// by a factor of 0.6', so year 2's $19,200 decline gives a $11,520 deduction.
/// The adjustable value falls by the whole decline, not the deduction.
#[test]
fn a_later_year_deduction_is_reduced_for_private_use() {
    let year = run(json!({"opening_adjustable_value": "48000.00", "taxable_use": "0.6"}));
    assert_eq!(amounts(&year), ["19200.00", "11520.00", "28800.00"]);
}

/// The ATO's prime cost example: 20% of the $80,000 cost, $16,000, in each of
/// the 5 years, so year 2 opens at $64,000 and closes at $48,000.
#[test]
fn prime_cost_applies_its_rate_to_cost_in_a_later_year() {
    let year = run(json!({"method": "prime_cost", "opening_adjustable_value": "64000.00"}));
    assert_eq!(amounts(&year), ["16000.00", "16000.00", "48000.00"]);
}

/// The ATO's note: an asset acquired part way through the year takes its final
/// prime cost claim in the sixth year. Held 146 days in year 1:
/// $80,000 × 146 ÷ 365 × 20% = $6,400, then $16,000 in each of years 2 to 5,
/// so year 6 opens at $80,000 − $6,400 − $64,000 = $9,600. The formula gives
/// $16,000, which s 40-75(7)(b) limits to that $9,600.
#[test]
fn prime_cost_is_limited_to_the_opening_value_in_the_final_year() {
    let prime = |overrides: Value| {
        let mut overrides = overrides;
        overrides["method"] = json!("prime_cost");
        run(overrides)
    };
    assert_eq!(
        amounts(&prime(json!({"days": 146}))),
        ["6400.00", "6400.00", "73600.00"]
    );
    let year_6 = prime(json!({"opening_adjustable_value": "9600.00"}));
    assert_eq!(amounts(&year_6), ["9600.00", "9600.00", "0.00"]);
}

/// s 40-72(3): diminishing value never takes more than the base value either.
/// $1,000 opening over a 1-year life: $1,000 × 200% ÷ 1 = $2,000, limited to $1,000.
#[test]
fn diminishing_value_is_limited_to_the_opening_value() {
    let year = run(json!({"effective_life": "1", "opening_adjustable_value": "1000.00"}));
    assert_eq!(amounts(&year), ["1000.00", "1000.00", "0.00"]);
}

/// An asset already written off declines no further by either method.
#[test]
fn a_written_off_asset_declines_no_further() {
    for method in ["prime_cost", "diminishing_value"] {
        let year = run(json!({"method": method, "opening_adjustable_value": "0.00"}));
        assert_eq!(amounts(&year), ["0.00", "0.00", "0.00"], "{method}");
    }
}

#[test]
fn an_opening_value_above_cost_or_not_money_is_refused() {
    assert_eq!(
        code(json!({"opening_adjustable_value": "80000.01"})),
        "input.opening_value_above_cost"
    );
    for bad in [
        json!("-1.00"),
        json!("1.001"),
        json!("NaN"),
        json!(48000),
        json!(null),
        json!(true),
    ] {
        assert_eq!(
            code(json!({ "opening_adjustable_value": bad })),
            "input.amount_invalid",
            "{bad}"
        );
    }
    assert_eq!(
        code(json!({"opening_adjustable_value": "12,50"})),
        "input.decimal_unreadable"
    );
    // A misspelt opening value is refused, not ignored into a start-year result.
    assert_eq!(
        code(json!({"opening_value": "48000.00"})),
        "input.field_unknown"
    );
}

/// An opening value equal to cost is the start year's answer.
#[test]
fn an_opening_value_equal_to_cost_matches_the_start_year() {
    for method in ["prime_cost", "diminishing_value"] {
        let first = run(json!({"method": method, "days": 200}));
        let same =
            run(json!({"method": method, "days": 200, "opening_adjustable_value": "80000.00"}));
        assert_eq!(first, same, "{method}");
    }
}

#[test]
fn the_catalogue_lists_the_opening_value_as_optional() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["depreciation"];
    assert!(entry["optional_inputs"]["opening_adjustable_value"].is_string());
    assert!(
        entry["required_inputs"]
            .get("opening_adjustable_value")
            .is_none()
    );
    assert!(catalogue["gst"].get("optional_inputs").is_none());
}

/// A deduction whose exact value is half a cent rounds up. $1,000.50 over 14
/// years by diminishing value, or 7 by prime cost, is a decline of exactly
/// $2,001 ÷ 14 = $142.928571…, and at 21% taxable use exactly $30.015, so
/// $30.02. Dividing before applying the use gave $30.01499… and $30.01.
#[test]
fn an_exact_half_cent_deduction_rounds_up() {
    for overrides in [
        json!({"cost": "1000.50", "effective_life": "14"}),
        json!({"cost": "1000.50", "effective_life": "7", "method": "prime_cost"}),
        json!({"cost": "2000.00", "effective_life": "14", "opening_adjustable_value": "1000.50"}),
    ] {
        let mut overrides = overrides;
        overrides["taxable_use"] = json!("0.21");
        let year = run(overrides.clone());
        assert_eq!(amounts(&year), ["142.93", "30.02", "857.57"], "{overrides}");
    }
}
