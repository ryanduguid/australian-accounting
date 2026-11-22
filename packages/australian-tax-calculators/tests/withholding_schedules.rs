// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Schedule 2 (horticulture and shearing), Schedule 14 (an agreed additional
//! amount), the daily and casual table and no ABN withholding (TAA 1953 Sch 1
//! s 12-190), all read on 22 November 2026, against the published examples of
//! the Withholding Schedules Instruments 2024, 2025, (No. 2) 2025 and 2026, the
//! ATO's tax tables and each year boundary
//! (docs/calculation-evidence.md#horticulture-shearing-withholding).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const YEARS: [&str; 3] = ["2024-25", "2025-26", "2026-27"];

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

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

const HORTICULTURE: &str = "horticulture-shearing-withholding";

fn worker(overrides: Value) -> Value {
    with(
        json!({
            "earnings": "231.50", "tfn_quoted": true, "foreign_resident": false,
            "tax_free_threshold_claimed": true, "working_holiday_maker": false,
            "labour_mobility_scheme": false, "year": "2026-27", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Schedule 2's example, the same in each instrument: $231.50 gives $30
/// (13%), $108 (47%, no TFN), $69 (30%, foreign resident) and $103 (45%).
#[test]
fn schedule_2s_example_in_each_year() {
    for year in YEARS {
        for (tfn, foreign, rate, expected) in [
            (true, false, "0.13", "30.00"),
            (false, false, "0.47", "108.00"),
            (true, true, "0.30", "69.00"),
            (false, true, "0.45", "103.00"),
        ] {
            let r = run(
                HORTICULTURE,
                &worker(json!({"year": year, "tfn_quoted": tfn, "foreign_resident": foreign})),
            );
            assert_eq!(amount(&r, "earnings_used"), "231.00");
            assert_eq!(
                amount(&r, "withholding"),
                expected,
                "{year} {tfn} {foreign}"
            );
            assert_eq!(r["rates"]["rate"], rate);
        }
    }
}

/// With a TFN the result is rounded to the dollar, 50 cents up ($1,150 x 13%
/// = $149.50); cents in the earnings are ignored first.
#[test]
fn schedule_2_rounds_half_up_after_ignoring_cents() {
    let r = run(HORTICULTURE, &worker(json!({"earnings": "1150.00"})));
    assert_eq!(amount(&r, "withholding"), "150.00");
    let r = run(HORTICULTURE, &worker(json!({"earnings": "1149.99"})));
    assert_eq!(amount(&r, "withholding"), "149.00");
    // Without a TFN cents in the result are ignored too: 47% of $1,149 is $540.03.
    let r = run(
        HORTICULTURE,
        &worker(json!({"earnings": "1149.99", "tfn_quoted": false})),
    );
    assert_eq!(amount(&r, "withholding"), "540.00");
}

/// A worker without a TFN has no TFN declaration claiming the threshold, so
/// the threshold flag does not refuse them.
#[test]
fn schedule_2_without_a_tfn_needs_no_threshold_claim() {
    let r = run(
        HORTICULTURE,
        &worker(json!({"tfn_quoted": false, "tax_free_threshold_claimed": false})),
    );
    assert_eq!(amount(&r, "withholding"), "108.00");
}

#[test]
fn schedule_2_refusals() {
    for (overrides, expected) in [
        (
            json!({"working_holiday_maker": true}),
            "withholding.working_holiday_maker",
        ),
        (
            json!({"labour_mobility_scheme": true}),
            "horticulture_shearing_withholding.labour_mobility_scheme",
        ),
        (
            json!({"tax_free_threshold_claimed": false}),
            "horticulture_shearing_withholding.threshold_not_claimed",
        ),
        (json!({"tfn_quoted": "yes"}), "input.flag_invalid"),
        (json!({"year": "2023-24"}), "period.unsupported"),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
    ] {
        assert_eq!(
            code(HORTICULTURE, &worker(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}

const UPWARD: &str = "upward-variation-withholding";

fn upward(overrides: Value) -> Value {
    with(
        json!({
            "earnings": "563.00", "pay_period": "weekly", "scale": 2,
            "additional_amount": "50.00", "year": "2026-27", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Schedule 14's example: $563.00 a week on scale 2 and $50 more. The normal
/// amount is $33 for 2026-27 (F2026L00716) and $39 before (F2024L00664,
/// F2025L01016).
#[test]
fn schedule_14s_example_in_each_year() {
    for (year, normal, total) in [
        ("2024-25", "39.00", "89.00"),
        ("2025-26", "39.00", "89.00"),
        ("2026-27", "33.00", "83.00"),
    ] {
        let r = run(UPWARD, &upward(json!({"year": year})));
        assert_eq!(amount(&r, "normal_withholding"), normal, "{year}");
        assert_eq!(amount(&r, "additional_amount"), "50.00");
        assert_eq!(amount(&r, "withholding"), total, "{year}");
    }
}

/// The normal amount is payg-withholding's for the same inputs, its options
/// included, on every pay period and scale.
#[test]
fn schedule_14s_normal_amount_is_payg_withholdings() {
    let cases = [
        json!({"pay_period": "fortnightly", "scale": 1, "earnings": "2400.00"}),
        json!({"pay_period": "monthly", "scale": 6, "earnings": "5200.33"}),
        json!({"pay_period": "quarterly", "scale": 3, "earnings": "15000.00"}),
        json!({"scale": 4, "foreign_resident": true, "earnings": "900.40"}),
        json!({"tax_offsets": "1200.00", "medicare_adjustment_children": 1}),
        json!({"extra_pay_withholding": true, "earnings": "2600.00"}),
    ];
    for year in YEARS {
        for case in &cases {
            let input = upward(with(case.clone(), json!({"year": year})));
            let mut plain = input.clone();
            plain.as_object_mut().unwrap().remove("additional_amount");
            let payg = run("payg-withholding", &plain);
            let r = run(UPWARD, &input);
            let normal = amount(&payg, "withholding");
            assert_eq!(amount(&r, "normal_withholding"), normal, "{input}");
            let total: f64 = normal.parse::<f64>().unwrap() + 50.0;
            assert_eq!(amount(&r, "withholding"), format!("{total:.2}"), "{input}");
        }
    }
}

#[test]
fn schedule_14_refusals() {
    for (overrides, expected) in [
        (json!({"pay_period": "daily"}), "input.pay_period_invalid"),
        (json!({"scale": 7}), "input.tax_scale_unsupported"),
        (json!({"additional_amount": "-1"}), "input.amount_invalid"),
        (
            json!({"scale": 1, "tax_offsets": "100"}),
            "payg.tax_offsets_not_allowed",
        ),
        (json!({"year": "2023-24"}), "period.unsupported"),
    ] {
        assert_eq!(
            code(UPWARD, &upward(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}

const DAILY: &str = "daily-casual-withholding";

fn casual(overrides: Value) -> Value {
    with(
        json!({
            "earnings": "193.62", "tfn_quoted": true, "foreign_resident": false,
            "tax_free_threshold_claimed": true, "working_holiday_maker": false,
            "year": "2026-27", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// The tax table for daily and casual workers: $193.62 and $500 of tax
/// offsets give $25 less $2 from 1 July 2026 and $26 less $2 from 1 July 2024.
/// 2026-27: 193 x 5 + 0.99 = 965.99; 0.3227 x 965.99 - 185.1935 = 126.53, $127;
/// / 5 = 25.4, $25; $500 x 0.0038 = 1.9, $2.
#[test]
fn the_daily_tables_example_in_each_year() {
    for (year, daily, net) in [
        ("2024-25", "26.00", "24.00"),
        ("2025-26", "26.00", "24.00"),
        ("2026-27", "25.00", "23.00"),
    ] {
        let r = run(
            DAILY,
            &casual(json!({"year": year, "tax_offsets": "500.00"})),
        );
        assert_eq!(amount(&r, "weekly_earnings_used"), "965.99");
        assert_eq!(amount(&r, "daily_withholding"), daily, "{year}");
        assert_eq!(amount(&r, "tax_offset_reduction"), "2.00");
        assert_eq!(amount(&r, "withholding"), net, "{year}");
        assert_eq!(r["rates"]["scale"], "2");
    }
    let r = run(DAILY, &casual(json!({})));
    assert_eq!(amount(&r, "weekly_withholding"), "127.00");
    assert_eq!(amount(&r, "withholding"), "25.00");
    assert!(r["amounts"].get("tax_offset_reduction").is_none());
}

/// Below the scale 2 threshold ($362 a week in 2026-27) nothing is withheld.
#[test]
fn a_low_daily_wage_has_nil_withholding() {
    let r = run(DAILY, &casual(json!({"earnings": "70.00"})));
    assert_eq!(amount(&r, "weekly_earnings_used"), "350.99");
    assert_eq!(amount(&r, "withholding"), "0.00");
}

/// Without a TFN: 47% of the daily earnings with cents ignored ($90.71), or
/// 45% for a foreign resident ($86.85), cents in the result ignored.
#[test]
fn the_daily_table_without_a_tfn() {
    let r = run(DAILY, &casual(json!({"tfn_quoted": false})));
    assert_eq!(amount(&r, "withholding"), "90.00");
    let r = run(
        DAILY,
        &casual(json!({"tfn_quoted": false, "foreign_resident": true})),
    );
    assert_eq!(amount(&r, "withholding"), "86.00");
    assert_eq!(r["rates"]["rate"], "0.45");
}

#[test]
fn daily_table_refusals() {
    for (overrides, expected) in [
        (
            json!({"tax_free_threshold_claimed": false}),
            "daily_casual_withholding.threshold_not_claimed",
        ),
        (
            json!({"foreign_resident": true}),
            "daily_casual_withholding.foreign_resident_tfn",
        ),
        (
            json!({"working_holiday_maker": true}),
            "withholding.working_holiday_maker",
        ),
        (
            json!({"tfn_quoted": false, "tax_offsets": "100.00"}),
            "daily_casual_withholding.tax_offsets_not_allowed",
        ),
        (
            json!({"earnings": "100.00", "tax_offsets": "5000.00"}),
            "daily_casual_withholding.offsets_exceed_withholding",
        ),
        (json!({"year": "2023-24"}), "period.unsupported"),
    ] {
        assert_eq!(
            code(DAILY, &casual(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}

const NO_ABN: &str = "no-abn-withholding";

fn supply(overrides: Value) -> Value {
    with(
        json!({
            "payment": "1800.00", "gst_included": "0.00",
            "supply_total_excluding_gst": "1800.00", "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Rental expenses (Rental properties 2025): 47% of Derek's $1,800 is $846.
#[test]
fn the_rental_guides_no_abn_example() {
    for year in YEARS {
        let r = run(NO_ABN, &supply(json!({"year": year})));
        assert_eq!(amount(&r, "withholding"), "846.00");
        assert_eq!(r["rates"]["rate"], "0.47");
    }
}

/// s 12-190(4)(b) with GST Regulations s 29-80.01: nothing when the supply's
/// payments, disregarding GST, are $75 or less; a larger supply paid in parts
/// is withheld from each (Ana's $150 sign in two payments of $75).
#[test]
fn the_75_dollar_threshold() {
    let r = run(
        NO_ABN,
        &supply(json!({"payment": "75.00", "supply_total_excluding_gst": "75.00"})),
    );
    assert_eq!(amount(&r, "withholding"), "0.00");
    let r = run(
        NO_ABN,
        &supply(json!({"payment": "75.01", "supply_total_excluding_gst": "75.01"})),
    );
    assert_eq!(amount(&r, "withholding"), "35.00");
    let r = run(
        NO_ABN,
        &supply(json!({"payment": "75.00", "supply_total_excluding_gst": "150.00"})),
    );
    assert_eq!(amount(&r, "withholding"), "35.00");
    // GST is disregarded for the threshold, not for the amount withheld.
    let r = run(
        NO_ABN,
        &supply(json!({"payment": "82.50", "gst_included": "7.50",
                       "supply_total_excluding_gst": "75.00"})),
    );
    assert_eq!(amount(&r, "withholding"), "0.00");
    let r = run(
        NO_ABN,
        &supply(json!({"payment": "1100.00", "gst_included": "100.00",
                       "supply_total_excluding_gst": "1000.00"})),
    );
    assert_eq!(amount(&r, "withholding"), "517.00");
}

/// Regulations s 26: cents in the payment and in the result are disregarded.
#[test]
fn no_abn_cents_are_disregarded() {
    let r = run(
        NO_ABN,
        &supply(json!({"payment": "100.99", "supply_total_excluding_gst": "100.99"})),
    );
    assert_eq!(amount(&r, "payment_used"), "100.00");
    assert_eq!(amount(&r, "withholding"), "47.00");
    let r = run(
        NO_ABN,
        &supply(json!({"payment": "0.99", "supply_total_excluding_gst": "500.00"})),
    );
    assert_eq!(amount(&r, "withholding"), "0.00");
}

#[test]
fn no_abn_refusals() {
    for (overrides, expected) in [
        (
            json!({"gst_included": "1800.01"}),
            "no_abn_withholding.gst_above_payment",
        ),
        (
            json!({"supply_total_excluding_gst": "1799.99"}),
            "no_abn_withholding.total_below_payment",
        ),
        (json!({"year": "2023-24"}), "period.unsupported"),
    ] {
        assert_eq!(
            code(NO_ABN, &supply(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}
