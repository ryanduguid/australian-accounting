// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Schedule 3 (entertainers), Schedule 4 (return to work payments), Schedule 6
//! (annuities) and Schedule 10 (voluntary agreements), read on 22 November
//! 2026, against the published examples of the Withholding Schedules
//! Instruments 2024, 2025, (No. 2) 2025 and 2026 and each year boundary
//! (docs/calculation-evidence.md#entertainer-withholding).
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

fn refusals(command: &str, base: fn(Value) -> Value, cases: &[(Value, &str)]) {
    for (overrides, expected) in cases {
        assert_eq!(
            code(command, &base(overrides.clone())),
            *expected,
            "{overrides}"
        );
    }
}

const ENTERTAINER: &str = "entertainer-withholding";

fn performer(overrides: Value) -> Value {
    with(
        json!({
            "earnings": "500.35", "tfn_quoted": true, "foreign_resident": false,
            "tax_free_threshold_claimed": true, "working_holiday_maker": false,
            "year": "2026-27", "scope_confirmed": true,
            "performances_per_week": 2, "performances_per_day": 1,
        }),
        overrides,
    )
}

/// Schedule 3's formula example: Sandra's 2 performances at $500.35, one a
/// day. 2026-27: 0.1432 x 1,000.99 - 60.8377 = 82.5041, $83, / 2 = 41.50, $42.
/// Before: 0.1512 x 1,000.99 - 64.3365 = 87.0132, $87, $43.50, $44.
#[test]
fn sandras_example_in_each_year() {
    for (year, weekly, daily) in [
        ("2024-25", "87.00", "44.00"),
        ("2025-26", "87.00", "44.00"),
        ("2026-27", "83.00", "42.00"),
    ] {
        let r = run(ENTERTAINER, &performer(json!({"year": year})));
        assert_eq!(amount(&r, "weekly_earnings_used"), "1000.99");
        assert_eq!(amount(&r, "weekly_withholding"), weekly, "{year}");
        assert_eq!(amount(&r, "withholding"), daily, "{year}");
        assert_eq!(r["rates"]["table"], "A");
    }
}

/// Schedule 3's lookup tool example, by the formula: $279.35 a day, 3
/// performances, $500 of tax offsets ($500 / 52 / 3 = 3.21, $3). 2026-27:
/// 0.2000 x 837.99 - 108.2135 = 59.38, $59, / 3 = 19.67, $20, less $3, $17.
/// Before: 0.1440 x 837.99 - 57.8462 = 62.82, $63, $21, less $3, $18.
#[test]
fn the_lookup_example_by_the_formula() {
    for (year, net) in [
        ("2024-25", "18.00"),
        ("2025-26", "18.00"),
        ("2026-27", "17.00"),
    ] {
        let input = performer(json!({
            "earnings": "279.35", "performances_per_week": 3, "tax_offsets": "500.00", "year": year,
        }));
        let r = run(ENTERTAINER, &input);
        assert_eq!(amount(&r, "tax_offset_reduction"), "3.00");
        assert_eq!(amount(&r, "withholding"), net, "{year}");
    }
}

/// Table B when the threshold is not claimed: 0.2582 x 1,000.99 less 74.1674
/// (2026-27) or 68.2367 (before) is $184 or $190, halved to $92 or $95. Two
/// of 3 performances in a day: $59 / 3 x 2 = 39.33, $39. Below Table A's
/// first limit nothing is withheld.
#[test]
fn table_b_several_performances_and_nil() {
    for (year, daily) in [("2024-25", "95.00"), ("2026-27", "92.00")] {
        let input = performer(json!({"tax_free_threshold_claimed": false, "year": year}));
        let r = run(ENTERTAINER, &input);
        assert_eq!(amount(&r, "withholding"), daily, "{year}");
        assert_eq!(r["rates"]["table"], "B");
    }
    let input = performer(json!({
        "earnings": "279.35", "performances_per_week": 3, "performances_per_day": 2,
    }));
    assert_eq!(amount(&run(ENTERTAINER, &input), "withholding"), "39.00");
    let input = performer(json!({"earnings": "100.00", "performances_per_week": 3}));
    assert_eq!(amount(&run(ENTERTAINER, &input), "withholding"), "0.00");
}

/// Without a TFN: 47%, or 45% for a foreign resident, of the day's earnings
/// with cents ignored; nil tax offsets are no claim.
#[test]
fn schedule_3_without_a_tfn() {
    let r = run(ENTERTAINER, &performer(json!({"tfn_quoted": false})));
    assert_eq!(amount(&r, "day_earnings_used"), "500.00");
    assert_eq!(amount(&r, "withholding"), "235.00");
    let input = performer(json!({"tfn_quoted": false, "foreign_resident": true}));
    assert_eq!(amount(&run(ENTERTAINER, &input), "withholding"), "225.00");
    let input = performer(json!({
        "tfn_quoted": false, "earnings": "100.50", "performances_per_day": 2, "tax_offsets": "0.00",
    }));
    let r = run(ENTERTAINER, &input);
    assert_eq!(amount(&r, "day_earnings"), "201.00");
    assert_eq!(amount(&r, "withholding"), "94.00");
    // One payment for 3 performances at $3: 47% of $9 is $4, where 3
    // separate payments would give $1 each.
    let input = performer(json!({
        "tfn_quoted": false, "earnings": "3.00", "performances_per_week": 3, "performances_per_day": 3,
    }));
    assert_eq!(amount(&run(ENTERTAINER, &input), "withholding"), "4.00");
}

#[test]
fn schedule_3_refusals() {
    refusals(
        ENTERTAINER,
        performer,
        &[
            (
                json!({"foreign_resident": true}),
                "entertainer_withholding.foreign_resident_tfn",
            ),
            (
                json!({"tfn_quoted": false, "tax_offsets": "100.00"}),
                "entertainer_withholding.tax_offsets_not_allowed",
            ),
            (
                json!({"tax_free_threshold_claimed": false, "tax_offsets": "100.00"}),
                "entertainer_withholding.tax_offsets_not_allowed",
            ),
            (
                json!({"performances_per_day": 2, "tax_offsets": "100.00"}),
                "entertainer_withholding.offsets_several_performances",
            ),
            (
                json!({"earnings": "279.35", "performances_per_week": 3, "tax_offsets": "50000.00"}),
                "entertainer_withholding.offsets_exceed_withholding",
            ),
            (
                json!({"working_holiday_maker": true}),
                "withholding.working_holiday_maker",
            ),
            (json!({"performances_per_day": 3}), "input.count_invalid"),
            (json!({"performances_per_week": 0}), "input.count_invalid"),
            (json!({"year": "2023-24"}), "period.unsupported"),
            (json!({"year": "2027-28"}), "period.unsupported"),
        ],
    );
}

const RETURN_TO_WORK: &str = "return-to-work-withholding";

fn returner(overrides: Value) -> Value {
    with(
        json!({
            "payment": "18000.00", "tfn_quoted": true, "foreign_resident": false,
            "working_holiday_maker": false, "year": "2026-27", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Schedule 4's example in each instrument: George's $18,000 at 32% is
/// $5,760. With a TFN the cents count ($101.57 x 0.32 = 32.50, $33); without
/// one they are ignored (47% or 45% of $18,000).
#[test]
fn georges_example_and_rounding() {
    for year in YEARS {
        let r = run(RETURN_TO_WORK, &returner(json!({"year": year})));
        assert_eq!(amount(&r, "withholding"), "5760.00", "{year}");
    }
    let r = run(RETURN_TO_WORK, &returner(json!({"payment": "101.57"})));
    assert_eq!(amount(&r, "withholding"), "33.00");
    let input = returner(json!({"payment": "18000.99", "tfn_quoted": false}));
    assert_eq!(
        amount(&run(RETURN_TO_WORK, &input), "withholding"),
        "8460.00"
    );
    let input = returner(json!({"tfn_quoted": false, "foreign_resident": true}));
    assert_eq!(
        amount(&run(RETURN_TO_WORK, &input), "withholding"),
        "8100.00"
    );
}

#[test]
fn schedule_4_refusals() {
    refusals(
        RETURN_TO_WORK,
        returner,
        &[
            (
                json!({"working_holiday_maker": true}),
                "withholding.working_holiday_maker",
            ),
            (json!({"payment": "-1"}), "input.amount_invalid"),
            (json!({"year": "2023-24"}), "period.unsupported"),
        ],
    );
}

const ANNUITY: &str = "annuity-withholding";

fn annuitant(overrides: Value) -> Value {
    with(
        json!({
            "payment": "1000.00", "deductible_amount": "5200.00", "instalments": 52,
            "pay_period": "weekly", "scale": 2, "year": "2026-27", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Schedule 6's examples: Barbara's $1,000 a week less $5,200 / 52 is $900,
/// $106 on the weekly table from 1 July 2026 and $111 before; Kenneth's $850 a
/// fortnight less $1,000 / 10 is $750, $4 in each year.
#[test]
fn barbaras_and_kenneths_examples() {
    for (year, barbara) in [
        ("2024-25", "111.00"),
        ("2025-26", "111.00"),
        ("2026-27", "106.00"),
    ] {
        let r = run(ANNUITY, &annuitant(json!({"year": year})));
        assert_eq!(amount(&r, "earnings"), "900.00");
        assert_eq!(amount(&r, "withholding"), barbara, "{year}");
        let kenneth = annuitant(json!({
            "payment": "850.00", "deductible_amount": "1000.00", "instalments": 10,
            "pay_period": "fortnightly", "year": year,
        }));
        let r = run(ANNUITY, &kenneth);
        assert_eq!(amount(&r, "earnings"), "750.00");
        assert_eq!(amount(&r, "withholding"), "4.00", "{year}");
    }
}

/// After the income amount, the result is payg-withholding's on it, for each
/// pay period, scale and option.
#[test]
fn schedule_6_is_schedule_1_on_the_income() {
    let cases = [
        json!({"pay_period": "monthly", "scale": 1, "payment": "4000.00", "deductible_amount": "6000.00", "instalments": 12}),
        json!({"pay_period": "quarterly", "scale": 6, "payment": "9000.00", "instalments": 4}),
        json!({"scale": 3}),
        json!({"scale": 5, "tax_offsets": "300.00"}),
        json!({"deductible_amount": "5226.00"}),
        json!({"tax_offsets": "1200.00", "medicare_adjustment_children": 1}),
        json!({"extra_pay_withholding": true}),
    ];
    for year in YEARS {
        for case in &cases {
            let input = annuitant(with(case.clone(), json!({"year": year})));
            let r = run(ANNUITY, &input);
            let mut plain = input.clone();
            let fields = plain.as_object_mut().unwrap();
            fields.remove("deductible_amount");
            fields.remove("instalments");
            fields.remove("payment");
            fields.insert("earnings".into(), json!(amount(&r, "earnings")));
            let payg = run("payg-withholding", &plain);
            assert_eq!(
                amount(&r, "withholding"),
                amount(&payg, "withholding"),
                "{input}"
            );
        }
    }
}

/// Without a TFN (scale 4): 47%, or 45% for a foreign resident, of the
/// annuity payment with cents ignored.
#[test]
fn schedule_6_without_a_tfn() {
    let input = annuitant(json!({"scale": 4, "foreign_resident": false, "payment": "1000.99"}));
    assert_eq!(amount(&run(ANNUITY, &input), "withholding"), "470.00");
    let input = annuitant(json!({"scale": 4, "foreign_resident": true}));
    assert_eq!(amount(&run(ANNUITY, &input), "withholding"), "450.00");
}

#[test]
fn schedule_6_refusals() {
    refusals(
        ANNUITY,
        annuitant,
        &[
            (
                json!({"deductible_amount": "52052.00"}),
                "annuity_withholding.deductible_above_payment",
            ),
            (
                json!({"deductible_amount": "100.00", "instalments": 3}),
                "annuity_withholding.income_not_whole_cents",
            ),
            (
                json!({"scale": 4, "foreign_resident": false, "extra_pay_withholding": true}),
                "annuity_withholding.extra_pay_without_tfn",
            ),
            (json!({"instalments": 0}), "input.count_invalid"),
            (json!({"pay_period": "daily"}), "input.pay_period_invalid"),
            (json!({"scale": 7}), "input.tax_scale_unsupported"),
            (json!({"year": "2023-24"}), "period.unsupported"),
        ],
    );
}

const VOLUNTARY: &str = "voluntary-agreement-withholding";

fn agreement(overrides: Value) -> Value {
    with(
        json!({
            "payment": "1000.00", "gst_included": "0.00", "agreement_in_force": true,
            "abn_quoted": true, "agreement_applies_instalment_rate": false,
            "year": "2026-27", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Schedule 10's example: Francesca's CIR of 16.44% applies under the
/// agreement ($164.40, $164); without that term 20% ($200). A CIR of 20% or
/// more applies anyway; GST is excluded first; 50 cents round up.
#[test]
fn francescas_example_and_the_rate() {
    for year in YEARS {
        let r = run(VOLUNTARY, &agreement(json!({"year": year})));
        assert_eq!(amount(&r, "withholding"), "200.00", "{year}");
    }
    for (overrides, expected) in [
        (
            json!({"instalment_rate": "16.44", "agreement_applies_instalment_rate": true}),
            "164.00",
        ),
        (json!({"instalment_rate": "16.44"}), "200.00"),
        (json!({"instalment_rate": "25"}), "250.00"),
        (json!({"instalment_rate": "20.00"}), "200.00"),
        (
            json!({"payment": "1100.00", "gst_included": "100.00"}),
            "200.00",
        ),
        (json!({"payment": "1002.50"}), "201.00"),
    ] {
        let r = run(VOLUNTARY, &agreement(overrides.clone()));
        assert_eq!(amount(&r, "withholding"), expected, "{overrides}");
    }
}

#[test]
fn schedule_10_refusals() {
    refusals(
        VOLUNTARY,
        agreement,
        &[
            (
                json!({"agreement_in_force": false}),
                "voluntary_agreement_withholding.agreement_not_in_force",
            ),
            (
                json!({"abn_quoted": false}),
                "voluntary_agreement_withholding.abn_not_quoted",
            ),
            (
                json!({"agreement_applies_instalment_rate": true}),
                "voluntary_agreement_withholding.instalment_rate_missing",
            ),
            (
                json!({"instalment_rate": "101"}),
                "input.percentage_invalid",
            ),
            (
                json!({"gst_included": "1000.01"}),
                "voluntary_agreement_withholding.gst_above_payment",
            ),
            (json!({"year": "2023-24"}), "period.unsupported"),
        ],
    );
}
