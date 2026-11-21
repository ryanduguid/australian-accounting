// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The low-value pool (ITAA 1997 ss 40-440 and 40-445), the general small
//! business pool (ss 328-190 to 328-215, with IT(TP)A 1997 ss 328-180(6) and
//! 328-181(5) and Act No. 71 of 2026 Sch 2) and capital works (ss 43-210 and
//! 43-215), all read on 22 November 2026, against the ATO's published
//! examples and each year boundary (docs/calculation-evidence.md#depreciation-low-value-pool).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const YEARS: [&str; 7] = [
    "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
];

fn with(base: Value, overrides: Value) -> Value {
    let mut input = base;
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

/// The plain result, checked against `derive` byte for byte.
fn run(command: &str, input: &Value) -> Value {
    let result = api::evaluate(command, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": command, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(command: &str, input: &Value) -> &'static str {
    api::evaluate_coded(command, input).unwrap_err().code
}

fn amounts<const N: usize>(result: &Value, keys: [&str; N]) -> [String; N] {
    keys.map(|key| result["amounts"][key].as_str().unwrap().to_string())
}

fn low_value(overrides: Value) -> Value {
    with(
        json!({
            "previous_closing_pool_balance": "0.00", "low_cost_assets": "0.00",
            "low_value_assets": "0.00", "second_element_costs": "0.00",
            "termination_values": "0.00", "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

const LVP: &str = "depreciation-low-value-pool";
const LVP_KEYS: [&str; 6] = [
    "new_assets_decline",
    "pool_decline",
    "decline_in_value",
    "closing_before_disposals",
    "closing_pool_balance",
    "assessable_excess",
];

/// Guide to depreciating assets 2026: John's $990 printer at 60% ($594) and a
/// $5,000 closing balance for 2024-25 give 18.75% x $594 = $111 and
/// 37.5% x $5,000 = $1,875, a $1,986 decline and a $3,608 closing balance, in
/// whole dollars; exactly $111.375, so $1,986.38 and $3,607.62.
#[test]
fn the_guides_low_value_pool_example() {
    let r = run(
        LVP,
        &low_value(
            json!({"previous_closing_pool_balance": "5000.00", "low_cost_assets": "594.00"}),
        ),
    );
    assert_eq!(
        amounts(&r, LVP_KEYS),
        ["111.38", "1875.00", "1986.38", "3607.62", "3607.62", "0.00"]
    );
    assert_eq!(r["rates"]["first_year_rate"], "0.1875");
    assert_eq!(r["rates"]["pool_rate"], "0.375");
    assert_eq!(r["calculation"], "depreciation_low_value_pool");
}

/// The ATO's Low-value pool page, 2021-22: 37.5% x $3,608 = $1,353, less $300
/// for the printer sold (60% of $500), closing at $1,955.
#[test]
fn a_disposal_reduces_the_closing_pool_balance() {
    let r = run(
        LVP,
        &low_value(json!({"previous_closing_pool_balance": "3608.00",
            "termination_values": "300.00", "year": "2021-22"})),
    );
    assert_eq!(
        amounts(&r, LVP_KEYS),
        ["0.00", "1353.00", "1353.00", "2255.00", "1955.00", "0.00"]
    );
}

/// Low-value assets go in at 37.5% and second element amounts at 18.75%.
#[test]
fn low_value_assets_and_second_element_costs_take_their_rates() {
    let r = run(
        LVP,
        &low_value(json!({"previous_closing_pool_balance": "1000.00",
            "low_value_assets": "800.00", "second_element_costs": "400.00", "year": "2026-27"})),
    );
    assert_eq!(
        amounts(&r, LVP_KEYS),
        ["75.00", "675.00", "750.00", "1450.00", "1450.00", "0.00"]
    );
}

/// The decline is rounded once, from the unrounded sum of its 2 parts.
#[test]
fn the_decline_is_rounded_once() {
    let r = run(
        LVP,
        &low_value(json!({"previous_closing_pool_balance": "0.01", "low_cost_assets": "0.02"})),
    );
    assert_eq!(
        amounts(&r, LVP_KEYS),
        ["0.00", "0.00", "0.01", "0.02", "0.02", "0.00"]
    );
}

/// s 40-445(2): termination values above the closing pool balance leave it at
/// zero and the excess is assessable.
#[test]
fn termination_values_above_the_balance_are_assessable() {
    let r = run(
        LVP,
        &low_value(json!({"previous_closing_pool_balance": "1000.00",
            "termination_values": "1000.00", "year": "2023-24"})),
    );
    assert_eq!(
        amounts(&r, LVP_KEYS),
        ["0.00", "375.00", "375.00", "625.00", "0.00", "375.00"]
    );
}

/// The temporary full expensing years carry a warning; later years do not.
#[test]
fn the_full_expensing_years_are_flagged() {
    for year in YEARS {
        let r = run(LVP, &low_value(json!({"year": year})));
        let flagged = r["warnings"]
            .to_string()
            .contains("temporary full expensing");
        assert_eq!(flagged, year <= "2022-23", "{year}");
    }
}

fn small(overrides: Value) -> Value {
    with(
        json!({
            "opening_pool_balance": "0.00", "additions": "0.00", "cost_additions": "0.00",
            "termination_values": "0.00", "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

const SBP: &str = "depreciation-small-business-pool";
const SBP_KEYS: [&str; 8] = [
    "low_pool_value_amount",
    "low_pool_value_deduction",
    "pool_deduction",
    "new_asset_deduction",
    "cost_addition_deduction",
    "deduction",
    "assessable_income",
    "closing_pool_balance",
];

/// The ATO's Small business pool calculations, example 2 (Loretta, 2018-19,
/// worked under the same rules in 2025-26): $100,000 opening, a $28,000
/// trailer and an $8,000 sale give $30,000 and $4,200, closing at $85,800.
#[test]
fn the_atos_pool_example() {
    let r = run(
        SBP,
        &small(
            json!({"opening_pool_balance": "100000.00", "additions": "28000.00",
            "termination_values": "8000.00"}),
        ),
    );
    assert_eq!(
        amounts(&r, SBP_KEYS),
        [
            "120000.00",
            "0.00",
            "30000.00",
            "4200.00",
            "0.00",
            "34200.00",
            "0.00",
            "85800.00"
        ]
    );
    assert_eq!(r["rates"]["pool_rate"], "0.30");
    assert_eq!(r["rates"]["new_asset_rate"], "0.15");
}

/// s 328-210(3)'s example, 2024-25: $28,500 + $16,800 - $29,600 = $15,700,
/// below $20,000, is deducted and the pool closes at zero.
#[test]
fn cassidys_gardens_writes_off_a_low_pool_value() {
    let r = run(
        SBP,
        &small(
            json!({"opening_pool_balance": "28500.00", "additions": "16800.00",
            "termination_values": "29600.00", "year": "2024-25"}),
        ),
    );
    assert_eq!(
        amounts(&r, SBP_KEYS),
        [
            "15700.00", "15700.00", "0.00", "0.00", "0.00", "15700.00", "0.00", "0.00"
        ]
    );
}

/// LCR 2021/3 example 13: a $200,000 pool at 30 June 2020 is deducted in full
/// for 2020-21 (IT(TP)A s 328-181(5)).
#[test]
fn a_full_expensing_year_writes_off_the_whole_pool() {
    let r = run(
        SBP,
        &small(json!({"opening_pool_balance": "200000.00", "year": "2020-21"})),
    );
    assert_eq!(r["amounts"]["deduction"], "200000.00");
    assert_eq!(r["amounts"]["closing_pool_balance"], "0.00");
    assert!(r["warnings"].to_string().contains("328-181(5)"));
}

/// The low pool value threshold for each year, and either side of it.
#[test]
fn each_years_low_pool_value_threshold() {
    for year in YEARS {
        let limited = year >= "2023-24";
        let r = run(
            SBP,
            &small(json!({"opening_pool_balance": "1000000.00", "year": year})),
        );
        let threshold = &r["rates"]["low_pool_value_threshold"];
        let rule = r["rates"]["low_pool_value_threshold_rule"]
            .as_str()
            .unwrap();
        if limited {
            assert_eq!(*threshold, "20000", "{year}");
            assert_eq!(r["amounts"]["pool_deduction"], "300000.00", "{year}");
            assert_eq!(r["amounts"]["closing_pool_balance"], "700000.00", "{year}");
            let provision = if year == "2026-27" {
                "Act No. 71 of 2026"
            } else {
                "328-180(6)(e)"
            };
            assert!(rule.contains(provision), "{year}: {rule}");
        } else {
            assert_eq!(*threshold, "none", "{year}");
            assert_eq!(r["amounts"]["low_pool_value_deduction"], "1000000.00");
            assert!(rule.contains("328-181(5)"), "{year}: {rule}");
        }
        // Either side of $20,000: below it is written off, at it is not.
        let at = run(
            SBP,
            &small(json!({"opening_pool_balance": "20000.00", "year": year})),
        );
        let under = run(
            SBP,
            &small(json!({"opening_pool_balance": "19999.99", "year": year})),
        );
        assert_eq!(under["amounts"]["deduction"], "19999.99", "{year}");
        let at_deduction = if limited { "6000.00" } else { "20000.00" };
        assert_eq!(at["amounts"]["deduction"], at_deduction, "{year}");
    }
}

/// s 328-215(2)(b): a s 328-210(2) amount below zero is assessable.
#[test]
fn a_negative_pool_amount_is_assessable() {
    let r = run(
        SBP,
        &small(
            json!({"opening_pool_balance": "10000.00", "termination_values": "15000.00",
            "year": "2026-27"}),
        ),
    );
    assert_eq!(
        amounts(&r, SBP_KEYS),
        [
            "-5000.00", "0.00", "0.00", "0.00", "0.00", "0.00", "5000.00", "0.00"
        ]
    );
}

/// s 328-215(2)(a): an amount at or above the threshold whose 30% deduction
/// exceeds it leaves a closing balance below zero, which is assessable.
#[test]
fn a_negative_closing_balance_is_assessable() {
    let r = run(
        SBP,
        &small(
            json!({"opening_pool_balance": "1000000.00", "termination_values": "950000.00",
            "year": "2026-27"}),
        ),
    );
    assert_eq!(
        amounts(&r, SBP_KEYS),
        [
            "50000.00",
            "0.00",
            "300000.00",
            "0.00",
            "0.00",
            "300000.00",
            "250000.00",
            "0.00"
        ]
    );
    // Below the threshold after a sale, the remainder is written off.
    let r = run(
        SBP,
        &small(json!({"opening_pool_balance": "100000.00", "termination_values": "95000.00"})),
    );
    assert_eq!(r["amounts"]["low_pool_value_deduction"], "5000.00");
}

/// Cost additions take 15%, each deduction in cents before the closing balance.
#[test]
fn cost_additions_take_fifteen_percent() {
    let r = run(
        SBP,
        &small(
            json!({"opening_pool_balance": "33333.33", "additions": "11111.11",
            "cost_additions": "2222.22"}),
        ),
    );
    assert_eq!(
        amounts(&r, SBP_KEYS),
        [
            "46666.66", "0.00", "10000.00", "1666.67", "333.33", "12000.00", "0.00", "34666.66"
        ]
    );
}

/// Only 2020-21's zero amount with additions could take the 57.5% rate.
#[test]
fn backing_business_investment_is_refused_where_it_could_apply() {
    let input = small(
        json!({"additions": "5000.00", "termination_values": "5000.00",
        "year": "2020-21"}),
    );
    assert_eq!(
        code(SBP, &input),
        "depreciation_small_business_pool.backing_business_investment"
    );
    let later = small(
        json!({"additions": "5000.00", "termination_values": "5000.00",
        "year": "2021-22"}),
    );
    assert_eq!(run(SBP, &later)["amounts"]["new_asset_deduction"], "750.00");
}

fn works(overrides: Value) -> Value {
    with(
        json!({
            "construction_expenditure": "115800.00", "construction_start": "1991-08-01",
            "four_percent_manner": false, "days_used": 346, "taxable_use": "1",
            "undeducted_construction_expenditure": "20000.00", "year": "2025-26",
            "scope_confirmed": true,
        }),
        overrides,
    )
}

const CW: &str = "capital-works";
const CW_KEYS: [&str; 3] = ["rate_amount", "reduced_amount", "deduction"];

/// Rental properties guide 2026, example 29: $115,800 x 2.5% x 346 / 365 =
/// $2,744 in whole dollars; exactly $2,744.3013...
#[test]
fn the_rental_guides_capital_works_example() {
    let r = run(CW, &works(json!({})));
    assert_eq!(amounts(&r, CW_KEYS), ["2744.30", "2744.30", "2744.30"]);
    assert_eq!(r["rates"]["rate"], "0.025");
}

/// 366 days in 2023-24 over a denominator of 365, and no more elsewhere.
#[test]
fn a_leap_year_counts_366_days_over_365() {
    let leap = works(json!({"construction_expenditure": "365000.00",
        "construction_start": "2010-01-01", "days_used": 366, "year": "2023-24",
        "undeducted_construction_expenditure": "300000.00"}));
    assert_eq!(amounts(&run(CW, &leap), CW_KEYS)[2], "9150.00");
    for year in YEARS.into_iter().filter(|y| *y != "2023-24") {
        let input = works(json!({"days_used": 366, "year": year}));
        assert_eq!(code(CW, &input), "input.day_count_invalid", "{year}");
        assert!(api::evaluate(CW, &works(json!({"days_used": 365, "year": year}))).is_ok());
    }
    assert_eq!(
        code(CW, &works(json!({"days_used": 367, "year": "2023-24"}))),
        "input.day_count_invalid"
    );
    assert_eq!(
        code(CW, &works(json!({"days_used": -1}))),
        "input.day_count_invalid"
    );
}

/// s 43-210: the days used are at most the days of the year from the later
/// of 1 July and the start of construction, so none for works begun after it.
#[test]
fn days_used_end_at_the_start_of_construction() {
    let begun = |start: &str, days: i64| {
        works(json!({"construction_start": start, "days_used": days, "year": "2025-26"}))
    };
    assert!(api::evaluate(CW, &begun("2025-07-01", 365)).is_ok());
    assert!(api::evaluate(CW, &begun("2026-06-01", 30)).is_ok());
    assert_eq!(
        code(CW, &begun("2026-06-01", 31)),
        "input.day_count_invalid"
    );
    assert!(api::evaluate(CW, &begun("2026-06-30", 1)).is_ok());
    assert_eq!(code(CW, &begun("2026-06-30", 2)), "input.day_count_invalid");
    assert_eq!(
        code(CW, &begun("2027-07-01", 346)),
        "input.day_count_invalid"
    );
    assert_eq!(
        amounts(&run(CW, &begun("2027-07-01", 0)), CW_KEYS)[2],
        "0.00"
    );
}

/// The 4% manner at half income use, and the undeducted limit.
#[test]
fn four_percent_partial_use_and_the_undeducted_limit() {
    let four = works(json!({"construction_expenditure": "1000000.00",
        "construction_start": "2024-01-01", "four_percent_manner": true, "days_used": 365,
        "taxable_use": "0.5", "undeducted_construction_expenditure": "900000.00",
        "year": "2026-27"}));
    let r = run(CW, &four);
    assert_eq!(amounts(&r, CW_KEYS), ["40000.00", "20000.00", "20000.00"]);
    assert_eq!(r["rates"]["rate"], "0.04");
    let limited = works(json!({"construction_expenditure": "100000.00",
        "construction_start": "1990-01-01", "days_used": 365,
        "undeducted_construction_expenditure": "1000.00"}));
    assert_eq!(
        amounts(&run(CW, &limited), CW_KEYS),
        ["2500.00", "2500.00", "1000.00"]
    );
}

/// The start date boundaries: 16 September 1987 for any deduction here, and
/// 27 February 1992 for the 4% manner.
#[test]
fn the_construction_start_boundaries() {
    let start = |date: &str, four: bool| {
        works(json!({"construction_start": date, "four_percent_manner": four}))
    };
    assert_eq!(
        code(CW, &start("1987-09-15", false)),
        "capital_works.begun_before_16_september_1987"
    );
    assert!(api::evaluate(CW, &start("1987-09-16", false)).is_ok());
    assert_eq!(
        code(CW, &start("1992-02-26", true)),
        "capital_works.four_percent_before_27_february_1992"
    );
    let r = run(CW, &start("1992-02-27", true));
    assert_eq!(r["rates"]["rate"], "0.04");
    assert_eq!(code(CW, &start("1991-13-01", false)), "input.date_invalid");
}

#[test]
fn capital_works_inputs_are_checked() {
    assert_eq!(
        code(
            CW,
            &works(json!({"undeducted_construction_expenditure": "115800.01"}))
        ),
        "capital_works.undeducted_above_expenditure"
    );
    assert_eq!(
        code(CW, &works(json!({"taxable_use": "1.01"}))),
        "input.fraction_invalid"
    );
    assert_eq!(
        code(
            CW,
            &works(json!({"taxable_use": "0.123456789012345678901"}))
        ),
        "capital_works.precision_exceeded"
    );
    assert_eq!(
        code(CW, &works(json!({"four_percent_manner": "yes"}))),
        "input.flag_invalid"
    );
}

/// Every year from 2020-21 to 2026-27 is taken, and the years either side are not.
#[test]
fn the_supported_years() {
    let inputs = [
        (LVP, low_value(json!({}))),
        (SBP, small(json!({}))),
        (CW, works(json!({}))),
    ];
    for (command, input) in inputs {
        for year in YEARS {
            let result = run(command, &with(input.clone(), json!({ "year": year })));
            assert_eq!(result["period"], year, "{command}");
        }
        for year in ["2019-20", "2027-28"] {
            let refused = with(input.clone(), json!({ "year": year }));
            assert_eq!(
                code(command, &refused),
                "period.unsupported",
                "{command} {year}"
            );
        }
        let unconfirmed = with(input.clone(), json!({"scope_confirmed": false}));
        assert_eq!(code(command, &unconfirmed), "scope.not_confirmed");
        let extra = with(input.clone(), json!({"cost": "1.00"}));
        assert_eq!(code(command, &extra), "input.field_unknown", "{command}");
    }
}

#[test]
fn pool_amounts_must_be_money() {
    for bad in [json!("-1.00"), json!("1.001"), json!(5), json!(null)] {
        let input = low_value(json!({ "low_cost_assets": bad }));
        assert_eq!(code(LVP, &input), "input.amount_invalid", "{bad}");
        let input = small(json!({ "additions": bad }));
        assert_eq!(code(SBP, &input), "input.amount_invalid", "{bad}");
    }
}

/// Each catalogue example is the published example it names.
#[test]
fn the_catalogue_examples_run() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    for (kind, key, expected) in [
        ("depreciation_low_value_pool", "decline_in_value", "1986.38"),
        ("depreciation_small_business_pool", "deduction", "34200.00"),
        ("capital_works", "deduction", "2744.30"),
    ] {
        let mut facts = catalogue[kind]["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        let r = run(&kind.replace('_', "-"), &facts);
        assert_eq!(r["amounts"][key], expected, "{kind}");
        assert_eq!(
            catalogue[kind]["supported_periods"]
                .as_array()
                .unwrap()
                .len(),
            7
        );
    }
}
