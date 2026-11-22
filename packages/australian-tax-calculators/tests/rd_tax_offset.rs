// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The R&D tax incentive offset (ITAA 1997 s 355-100) and the s 355-450
//! clawback amount, read on 22 November 2026, against the ATO's 'Calculate the
//! clawback adjustment' and 'Clawback of feedstock adjustments' examples, the
//! R&D tax incentive schedule 2022 Part E method, each threshold, tier and cap
//! boundary, each year boundary and each refusal
//! (docs/calculation-evidence.md#rd-tax-offset).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "rd-tax-offset";
const YEARS: [&str; 6] = [
    "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
];

/// A base rate entity with turnover under $20 million and nothing else,
/// with `overrides`.
fn input(overrides: Value) -> Value {
    let mut input = json!({
        "notional_deductions": "0.00", "rsp_crc_expenditure": "0.00",
        "aggregated_turnover": "10000000.00", "exempt_entity_controlled": false,
        "base_rate_entity": true, "total_expenses": "0.00",
        "recoupment_clawback": "0.00", "feedstock_revenue": "0.00",
        "feedstock_expenditure": "0.00", "present_year_base_rate_entity": true,
        "year": "2024-25", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

/// The plain result, checked against `derive` byte for byte.
fn run(input: &Value) -> Value {
    let result = api::evaluate(COMMAND, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": COMMAND, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(COMMAND, input).unwrap_err().code
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

/// tax_offset, adjusted_offset, deduction_amount and clawback_income.
fn clawback(result: &Value) -> [String; 4] {
    [
        "tax_offset",
        "adjusted_offset",
        "deduction_amount",
        "clawback_income",
    ]
    .map(|key| amount(result, key))
}

/// The refundable and non-refundable parts.
fn split(result: &Value) -> [String; 2] {
    ["refundable_offset", "non_refundable_offset"].map(|key| amount(result, key))
}

/// Calculate the clawback adjustment, Company V: a grant of $2.3 million on a
/// $3.6 million project, total expenses $15 million, 30%, a non-refundable
/// offset of $115,500 + $1,534,500.
#[test]
fn company_v_grant() {
    let result = run(&input(json!({
        "year": "2021-22", "notional_deductions": "3600000.00",
        "total_expenses": "15000000.00", "aggregated_turnover": "60000000.00",
        "base_rate_entity": false, "present_year_base_rate_entity": false,
        "recoupment_clawback": "2300000.00",
    })));
    assert_eq!(
        clawback(&result),
        ["1650000.00", "580500.00", "690000.00", "1265000.00"]
    );
    assert_eq!(amount(&result, "base_offset"), "1080000.00");
    assert_eq!(amount(&result, "premium_tier_1"), "25500.00");
    assert_eq!(amount(&result, "premium_tier_2"), "544500.00");
    assert_eq!(split(&result), ["0.00", "1650000.00"]);
    assert_eq!(result["rates"]["corporate_tax_rate"], "0.30");
}

/// Company F (a $500,000 reimbursement of $1 million at 43.5%), Company Y's
/// 4 years, Company Q after the grant cap ($9,000), Company L and Company J
/// before and after its repayment: (year, notional deductions, clawback,
/// starting offset, adjusted offset, deduction amount, amount included).
#[test]
fn the_atos_refundable_clawback_examples() {
    let examples = [
        (
            "2022-23",
            "1000000.00",
            "500000.00",
            ["435000.00", "217500.00", "125000.00", "370000.00"],
        ),
        (
            "2023-24",
            "350000.00",
            "150000.00",
            ["152250.00", "87000.00", "37500.00", "111000.00"],
        ),
        (
            "2024-25",
            "150000.00",
            "50000.00",
            ["65250.00", "43500.00", "12500.00", "37000.00"],
        ),
        (
            "2023-24",
            "1000000.00",
            "300000.00",
            ["435000.00", "304500.00", "75000.00", "222000.00"],
        ),
        (
            "2024-25",
            "1500000.00",
            "100000.00",
            ["652500.00", "609000.00", "25000.00", "74000.00"],
        ),
        (
            "2024-25",
            "100000.00",
            "9000.00",
            ["43500.00", "39585.00", "2250.00", "6660.00"],
        ),
        (
            "2023-24",
            "700000.00",
            "250000.00",
            ["304500.00", "195750.00", "62500.00", "185000.00"],
        ),
        (
            "2023-24",
            "400000.00",
            "200000.00",
            ["174000.00", "87000.00", "50000.00", "148000.00"],
        ),
        (
            "2023-24",
            "400000.00",
            "50000.00",
            ["174000.00", "152250.00", "12500.00", "37000.00"],
        ),
    ];
    for (year, notional, recouped, want) in examples {
        let result = run(&input(json!({
            "year": year, "notional_deductions": notional, "recoupment_clawback": recouped,
        })));
        assert_eq!(clawback(&result), want, "{year} {notional}");
        assert_eq!(split(&result), [want[0].to_string(), "0.00".into()]);
    }
}

/// Feedstock (s 355-445(2)): the lesser of revenue and expenditure is the
/// clawback amount. Landscape Supplies' $9,000 and $10,000 on $122,000 rather
/// than its $22,000, which the threshold refusal holds back.
#[test]
fn feedstock_takes_the_lesser_amount() {
    let result = run(&input(json!({
        "notional_deductions": "122000.00", "feedstock_revenue": "9000.00",
        "feedstock_expenditure": "10000.00",
    })));
    assert_eq!(amount(&result, "clawback_amount"), "9000.00");
    assert_eq!(
        clawback(&result),
        ["53070.00", "49155.00", "2250.00", "6660.00"]
    );
    let lower_expenditure = input(json!({
        "notional_deductions": "122000.00", "feedstock_revenue": "9000.00",
        "feedstock_expenditure": "4000.00", "recoupment_clawback": "1000.00",
    }));
    assert_eq!(
        amount(&run(&lower_expenditure), "clawback_amount"),
        "5000.00"
    );
}

/// The ATO's feedstock example adjusts $22,000 less $9,000 at the full rate;
/// s 355-100(2) on its face would not. Refused, not guessed.
#[test]
fn a_clawback_under_the_threshold_is_refused() {
    let landscape = input(json!({
        "notional_deductions": "22000.00", "feedstock_revenue": "9000.00",
        "feedstock_expenditure": "10000.00",
    }));
    assert_eq!(code(&landscape), "rd_tax_offset.clawback_below_threshold");
    let below = input(json!({
        "notional_deductions": "15000.00", "rsp_crc_expenditure": "15000.00",
        "recoupment_clawback": "1.00",
    }));
    assert_eq!(code(&below), "rd_tax_offset.clawback_below_threshold");
    let at = input(json!({
        "notional_deductions": "30000.00", "recoupment_clawback": "10000.00",
    }));
    assert_eq!(clawback(&run(&at))[1], "8700.00");
    let under = input(json!({
        "notional_deductions": "30000.00", "recoupment_clawback": "10000.01",
    }));
    assert_eq!(code(&under), "rd_tax_offset.clawback_below_threshold");
}

/// The R&D tax incentive schedule 2022 Part E: $40 million at 38.5%, $110
/// million at 46.5% and the $50 million above the cap at 30%.
#[test]
fn a_non_refundable_offset_above_the_cap() {
    let result = run(&input(json!({
        "year": "2025-26", "notional_deductions": "200000000.00",
        "total_expenses": "2000000000.00", "aggregated_turnover": "500000000.00",
        "base_rate_entity": false,
    })));
    let parts = [
        "capped_base",
        "base_offset",
        "premium_tier_1",
        "premium_tier_2",
        "excess_offset",
        "tax_offset",
    ]
    .map(|key| amount(&result, key));
    assert_eq!(
        parts,
        [
            "150000000.00",
            "45000000.00",
            "3400000.00",
            "18150000.00",
            "15000000.00",
            "81550000.00"
        ]
    );
    assert_eq!(split(&result), ["0.00", "81550000.00"]);
}

/// Item 1 above the cap is still refundable (s 67-30(1) disregards s
/// 355-100(3)); at exactly $150 million nothing is excess.
#[test]
fn a_refundable_offset_at_and_above_the_cap() {
    let above = run(&input(json!({"notional_deductions": "160000000.00"})));
    assert_eq!(amount(&above, "excess_offset"), "2500000.00");
    assert_eq!(split(&above), ["67750000.00", "0.00"]);
    let at = run(&input(json!({"notional_deductions": "150000000.00"})));
    assert_eq!(amount(&at, "excess_offset"), "0.00");
    assert_eq!(amount(&at, "tax_offset"), "65250000.00");
}

/// s 355-100(2): under $20,000 only research service provider and CRC
/// expenditure counts; at $20,000 the whole total does.
#[test]
fn the_20000_threshold() {
    let under = run(&input(json!({
        "notional_deductions": "15000.00", "rsp_crc_expenditure": "12000.00",
    })));
    assert_eq!(amount(&under, "offset_base"), "12000.00");
    assert_eq!(amount(&under, "tax_offset"), "5220.00");
    let at = run(&input(json!({"notional_deductions": "20000.00"})));
    assert_eq!(amount(&at, "tax_offset"), "8700.00");
    let just_under = run(&input(json!({"notional_deductions": "19999.99"})));
    assert_eq!(amount(&just_under, "tax_offset"), "0.00");
    assert_eq!(amount(&just_under, "clawback_income"), "0.00");
}

/// Item 1 needs turnover under $20 million and no exempt entity control;
/// otherwise the tiers apply at the corporate tax rate.
#[test]
fn the_turnover_boundary_and_exempt_control() {
    let facts = |turnover: &str, exempt: bool| {
        input(json!({
            "notional_deductions": "100000.00", "total_expenses": "1000000.00",
            "aggregated_turnover": turnover, "exempt_entity_controlled": exempt,
        }))
    };
    let under = run(&facts("19999999.99", false));
    assert_eq!(split(&under), ["43500.00", "0.00"]);
    let at = run(&facts("20000000.00", false));
    assert_eq!(split(&at), ["0.00", "39900.00"]);
    assert_eq!(amount(&at, "premium_tier_1"), "1700.00");
    assert_eq!(amount(&at, "premium_tier_2"), "13200.00");
    let controlled = run(&input(json!({
        "notional_deductions": "100000.00", "total_expenses": "10000000.00",
        "aggregated_turnover": "5000000.00", "exempt_entity_controlled": true,
    })));
    assert_eq!(split(&controlled), ["0.00", "33500.00"]);
}

/// At exactly 2% of total expenses every dollar is in tier 1; a cent more
/// goes to tier 2.
#[test]
fn the_intensity_boundary() {
    let facts = |notional: &str| {
        input(json!({
            "notional_deductions": notional, "total_expenses": "10000000.00",
            "aggregated_turnover": "60000000.00", "base_rate_entity": false,
        }))
    };
    let at = run(&facts("200000.00"));
    assert_eq!(amount(&at, "tax_offset"), "77000.00");
    assert_eq!(amount(&at, "premium_tier_2"), "0.00");
    let above = run(&facts("200000.01"));
    assert_eq!(amount(&above, "premium_tier_1"), "17000.00");
    let well_above = run(&facts("200100.00"));
    assert_eq!(amount(&well_above, "premium_tier_2"), "16.50");
}

/// The present year's corporate tax rate divides the s 355-450 amount.
#[test]
fn the_present_years_rate_divides_the_clawback() {
    let result = run(&input(json!({
        "notional_deductions": "1000000.00", "recoupment_clawback": "500000.00",
        "present_year_base_rate_entity": false,
    })));
    assert_eq!(amount(&result, "clawback_income"), "308333.33");
    assert_eq!(result["rates"]["present_year_corporate_tax_rate"], "0.30");
}

#[test]
fn each_supported_year_and_its_boundaries() {
    for year in YEARS {
        let result = run(&input(
            json!({"year": year, "notional_deductions": "100000.00"}),
        ));
        assert_eq!(amount(&result, "tax_offset"), "43500.00", "{year}");
        assert_eq!(result["period"], year);
    }
    for year in ["2020-21", "2011-12"] {
        let refused = api::evaluate_coded(COMMAND, &input(json!({"year": year}))).unwrap_err();
        assert_eq!(refused.code, "period.unsupported");
        assert!(refused.message.contains("Act No. 92 of 2020"), "{year}");
    }
    let later = api::evaluate_coded(COMMAND, &input(json!({"year": "2027-28"}))).unwrap_err();
    assert_eq!(later.code, "period.unsupported");
    assert!(!later.message.contains("Act No. 92 of 2020"));
}

#[test]
fn each_refusal() {
    let cases = [
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (
            json!({"scope_confirmed": false, "year": "2020-21"}),
            "scope.not_confirmed",
        ),
        (
            json!({"notional_deductions": "-1.00"}),
            "input.amount_invalid",
        ),
        (json!({"total_expenses": "1.001"}), "input.amount_invalid"),
        (json!({"feedstock_revenue": 5}), "input.amount_invalid"),
        (json!({"base_rate_entity": "yes"}), "input.flag_invalid"),
        (json!({"exempt_entity_controlled": 1}), "input.flag_invalid"),
        (
            json!({"present_year_base_rate_entity": null}),
            "input.flag_invalid",
        ),
        (json!({"turnover": "1.00"}), "input.field_unknown"),
        (
            json!({"notional_deductions": "30000.00", "rsp_crc_expenditure": "30000.01"}),
            "rd_tax_offset.threshold_expenditure_above_total",
        ),
        (
            json!({"notional_deductions": "30000.00", "recoupment_clawback": "30000.01"}),
            "rd_tax_offset.clawback_above_total",
        ),
    ];
    for (overrides, want) in cases {
        assert_eq!(code(&input(overrides.clone())), want, "{overrides}");
    }
    let mut missing = input(json!({}));
    missing.as_object_mut().unwrap().remove("total_expenses");
    assert_eq!(code(&missing), "input.missing_field");
}

/// The catalogue's example is Company F, and it gives the ATO's $370,000.
#[test]
fn the_catalogue_example_is_company_f() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut example = catalogue["rd_tax_offset"]["example"]["facts"].clone();
    example.as_object_mut().unwrap().remove("kind");
    example["scope_confirmed"] = json!(true);
    assert_eq!(amount(&run(&example), "clawback_income"), "370000.00");
}
