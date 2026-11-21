// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The unused leave worksheet against the ATO's Schedule 7 for payments from
//! 1 July 2026 (published 17 June 2026) and its long service leave example,
//! the ATO's examples in 'Withholding from unused leave payments on termination
//! of employment' (last updated 1 May 2020) where they do not depend on that
//! year's tax tables, and ITAA 1997 Subdivisions 83-A and 83-B (compilation
//! 268), all read on 16 November 2026 (docs/calculation-evidence.md#unused-leave-payment);
//! then the $300 limit, the rounding, the accrual parts and the refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// Nil leave accrued from 1 January 2000, a normal termination, $1,155 a week
/// on scale 2 (the tax-free threshold claimed) in 2026-27.
fn input(overrides: Value) -> Value {
    let mut facts = json!({
        "annual_leave": "0.00", "leave_loading": "0.00", "long_service_leave": "0.00",
        "leave_accrual_start": "2000-01-01", "termination_reason": "normal",
        "working_holiday_maker": false,
        "earnings": "1155.00", "pay_period": "weekly", "scale": 2,
        "year": "2026-27", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn run(overrides: Value) -> Value {
    let facts = input(overrides);
    let result = api::evaluate("unused-leave-payment", &facts).unwrap();
    // derive gives the same result, byte for byte.
    let envelope = json!({"command": "unused-leave-payment", "input": facts});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(
        doc["result"].to_string(),
        result.to_string(),
        "derive {facts}"
    );
    result
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded("unused-leave-payment", &input(overrides))
        .unwrap_err()
        .code
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

fn amounts<'a, const N: usize>(result: &'a Value, keys: [&str; N]) -> [&'a str; N] {
    keys.map(|key| amount(result, key))
}

/// Schedule 1's weekly scale 2 withholding on `earnings` in 2026-27.
fn weekly(earnings: &str) -> i64 {
    weekly_on(earnings, 2)
}

/// Schedule 1's weekly withholding on `earnings` on `scale` in 2026-27.
fn weekly_on(earnings: &str, scale: i64) -> i64 {
    let facts = json!({"earnings": earnings, "pay_period": "weekly", "scale": scale,
                       "year": "2026-27", "scope_confirmed": true});
    let result = api::evaluate("payg-withholding", &facts).unwrap();
    let withheld = amount(&result, "withholding");
    withheld.strip_suffix(".00").unwrap().parse().unwrap()
}

/// Norman, Schedule 7's example: he retires (a normal termination) on $1,155 a
/// week with the tax-free threshold, and is paid $780 of pre-16 August 1978,
/// $6,420 of 16 August 1978 to 17 August 1993 and $14,820 of post-17 August
/// 1993 long service leave. 5% of $780 is $39; $6,420 x 32% is $2,054.40;
/// $39 + $14,820 = $14,859, / 52 is $285.75, so $285; $1,155 withholds $188 and
/// $1,440 $279, so ($279 - $188) x 52 = $4,732; $2,054 + $4,732 = $6,786. The
/// example gives no accrual start, so any day before 16 August 1978 serves.
#[test]
fn norman_has_6786_withheld_as_schedule_7_example_says() {
    let norman = run(json!({
        "long_service_leave": "22020.00",
        "long_service_leave_before_16_august_1978": "780.00",
        "long_service_leave_16_august_1978_to_17_august_1993": "6420.00",
        "leave_accrual_start": "1977-01-01",
    }));
    assert_eq!(
        amounts(
            &norman,
            [
                "long_service_leave_pre_1978_assessable",
                "flat_rate_withholding",
                "marginal_rate_amount",
                "withholding_on_earnings",
                "marginal_amount_per_period",
                "withholding_on_earnings_and_share",
                "marginal_withholding",
                "withholding",
            ]
        ),
        [
            "39.00", "2054.00", "14859.00", "188.00", "285.00", "279.00", "4732.00", "6786.00"
        ]
    );
    // Assessable: 5% of the pre-16 August 1978 part and all the rest; labels
    // A (the 30% capped part), B (the whole pre-1978 part) and gross pay.
    assert_eq!(
        amounts(
            &norman,
            [
                "long_service_leave_pre_1978_not_assessable",
                "assessable_income",
                "offset_capped_income",
                "lump_sum_a",
                "lump_sum_b",
                "unused_leave_on_termination",
                "qualifying_earnings",
            ]
        ),
        [
            "741.00", "21279.00", "6420.00", "6420.00", "780.00", "14820.00", "0.00"
        ]
    );
    assert_eq!(norman["rates"]["lump_sum_a_type"], "T");
    assert_eq!(norman["calculation"], "unused_leave_payment");
    assert_eq!(norman["period"], "2026-27");
}

/// The worksheet's catalogue example is Norman's.
#[test]
fn the_catalogue_example_is_schedule_7_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["unused_leave_payment"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("unused-leave-payment", &facts).unwrap();
    assert_eq!(amount(&result, "withholding"), "6786.00");
    assert_eq!(
        catalogue["unused_leave_payment"]["example_evidence"]["note"],
        "docs/calculation-evidence.md#unused-leave-payment"
    );
}

/// Greg, the ATO's unused annual leave example 1: invalidity, $2,260 of annual
/// leave and $420 of loading, $2,680 at 32%, $857.60. The 2020 page disregards
/// the cents for $857; Schedule 7 for payments from 1 July 2026 rounds to the
/// nearest dollar, 50 cents up, so $858. All of it is lump sum A, type R.
#[test]
fn greg_has_858_withheld_where_the_2020_page_disregarded_cents() {
    let greg = run(json!({
        "annual_leave": "2260.00", "leave_loading": "420.00",
        "termination_reason": "invalidity", "leave_accrual_start": "1990-01-01",
    }));
    assert_eq!(
        amounts(
            &greg,
            [
                "annual_leave_capped",
                "annual_leave_marginal",
                "flat_rate_withholding",
                "marginal_withholding",
                "withholding",
                "lump_sum_a",
                "unused_leave_on_termination",
            ]
        ),
        [
            "2680.00", "0.00", "858.00", "0.00", "858.00", "2680.00", "0.00"
        ]
    );
    assert_eq!(greg["rates"]["lump_sum_a_type"], "R");
}

/// Robyn, the ATO's long service leave example 7: genuine redundancy, $8,000,
/// all after 15 August 1978; $8,000 x 32% = $2,560, all lump sum A, type R,
/// and capped at 30% by the s 83-85 offset.
#[test]
fn robyn_has_2560_withheld_as_the_ato_example_says() {
    let robyn = run(json!({
        "long_service_leave": "8000.00", "termination_reason": "genuine_redundancy",
        "leave_accrual_start": "2005-01-01", "earnings": "1000.00",
    }));
    assert_eq!(
        amounts(
            &robyn,
            [
                "long_service_leave_capped",
                "long_service_leave_marginal",
                "withholding",
                "lump_sum_a",
                "assessable_income",
            ]
        ),
        ["8000.00", "0.00", "2560.00", "8000.00", "8000.00"]
    );
    assert_eq!(robyn["rates"]["lump_sum_a_type"], "R");
}

/// Matt, the ATO's example 8: retirement, $80,000 attributed $3,852 before
/// 16 August 1978, $31,407 to 17 August 1993 and $44,741 after. 5% of $3,852 is
/// $192.60; $31,407 x 32% is $10,050; $192.60 + $44,741 = $44,933.60 goes to
/// the marginal rate calculation. Its $16,744 comes from the 2014 tax tables,
/// so only the parts that do not depend on them are checked.
#[test]
fn matt_has_the_parts_the_ato_example_gives() {
    let matt = run(json!({
        "long_service_leave": "80000.00",
        "long_service_leave_before_16_august_1978": "3852.00",
        "long_service_leave_16_august_1978_to_17_august_1993": "31407.00",
        "leave_accrual_start": "1977-01-01", "earnings": "1200.00",
    }));
    assert_eq!(
        amounts(
            &matt,
            [
                "long_service_leave_pre_1978_assessable",
                "flat_rate_withholding",
                "marginal_rate_amount",
                "marginal_amount_per_period",
                "lump_sum_b",
            ]
        ),
        ["192.60", "10050.00", "44933.60", "864.00", "3852.00"]
    );
}

/// Sali, the ATO's example 9: all $16,000 is pre-16 August 1978 leave, 5% of
/// it ($800) goes to the marginal rate calculation, $15 a week after cents.
#[test]
fn sali_has_5_percent_assessable_as_the_ato_example_says() {
    let sali = run(json!({
        "long_service_leave": "16000.00",
        "long_service_leave_before_16_august_1978": "16000.00",
        "long_service_leave_16_august_1978_to_17_august_1993": "0.00",
        "leave_accrual_start": "1972-01-01", "earnings": "2400.00",
    }));
    assert_eq!(
        amounts(
            &sali,
            [
                "long_service_leave_pre_1978_assessable",
                "long_service_leave_pre_1978_not_assessable",
                "assessable_income",
                "flat_rate_withholding",
                "marginal_amount_per_period",
                "lump_sum_a",
                "lump_sum_b",
            ]
        ),
        [
            "800.00", "15200.00", "800.00", "0.00", "15.00", "0.00", "16000.00"
        ]
    );
}

/// Helen, the ATO's annual leave example 2: she resigns with $85,000 of which
/// $3,533 accrued before 18 August 1993. That part is lump sum A, type T, at
/// 32% ($1,130.56, so $1,131); the other $81,467 is at marginal rates.
/// Mark, example 4: $4,000 after 17 August 1993 over 52 weeks is $76 a week.
#[test]
fn helen_and_mark_split_annual_leave_as_the_ato_examples_say() {
    let helen = run(json!({
        "annual_leave": "85000.00", "annual_leave_before_18_august_1993": "3533.00",
        "leave_loading_before_18_august_1993": "0.00", "leave_accrual_start": "1992-09-18",
    }));
    assert_eq!(
        amounts(
            &helen,
            [
                "annual_leave_capped",
                "annual_leave_marginal",
                "flat_rate_withholding",
                "lump_sum_a",
                "unused_leave_on_termination",
            ]
        ),
        ["3533.00", "81467.00", "1131.00", "3533.00", "81467.00"]
    );
    assert_eq!(helen["rates"]["lump_sum_a_type"], "T");
    let mark = run(json!({"annual_leave": "4000.00", "earnings": "1000.00"}));
    assert_eq!(amount(&mark, "marginal_amount_per_period"), "76.00");
    let extra = weekly("1076.00") - weekly("1000.00");
    assert_eq!(amount(&mark, "withholding"), format!("{}.00", extra * 52));
}

/// Leave loading follows its annual leave: before 18 August 1993 at 32%,
/// after it at marginal rates, added to the annual leave (s 83-10(3)(b)).
#[test]
fn leave_loading_is_taxed_with_its_annual_leave() {
    let result = run(json!({
        "annual_leave": "1000.00", "leave_loading": "500.00",
        "annual_leave_before_18_august_1993": "100.00",
        "leave_loading_before_18_august_1993": "50.00",
        "leave_accrual_start": "1990-01-01",
    }));
    assert_eq!(
        amounts(&result, ["annual_leave_capped", "annual_leave_marginal"]),
        ["150.00", "1350.00"]
    );
}

/// A post-17 August 1993 normal payment under $300 has the lesser of the
/// marginal rate amount and 32% of it withheld (Schedule 7, 'When a TFN is
/// provided'); at $300 the limit no longer applies.
#[test]
fn under_300_the_lesser_of_marginal_and_32_percent_is_withheld() {
    for (leave, earnings) in [
        ("250.00", "3000.00"),
        ("299.99", "1155.00"),
        ("300.00", "3000.00"),
    ] {
        let result = run(json!({"annual_leave": leave, "earnings": earnings}));
        let share = (leave.parse::<f64>().unwrap() / 52.0).floor() as i64;
        let base: i64 = earnings.strip_suffix(".00").unwrap().parse().unwrap();
        let marginal = (weekly(&format!("{}.00", base + share)) - weekly(earnings)) * 52;
        assert_eq!(
            amount(&result, "marginal_withholding_before_limit"),
            format!("{marginal}.00")
        );
        let cents: i64 = leave.replace('.', "").parse().unwrap();
        // 32% of the payment in cents, to the nearest dollar, 50 cents up.
        let limit = (cents * 32 + 5000) / 10000;
        let expected = if leave == "300.00" {
            marginal
        } else {
            marginal.min(limit)
        };
        assert_eq!(
            amount(&result, "withholding"),
            format!("{expected}.00"),
            "{leave}"
        );
    }
}

/// 32% rounds to the nearest dollar, 50 cents up: $1.57 is $0.5024, so $1;
/// $1.56 is $0.4992, so nil.
#[test]
fn withholding_at_32_percent_rounds_to_the_dollar() {
    for (leave, withheld) in [("1.57", "1.00"), ("1.56", "0.00")] {
        let result = run(json!({
            "annual_leave": leave, "termination_reason": "genuine_redundancy",
        }));
        assert_eq!(
            amount(&result, "flat_rate_withholding"),
            withheld,
            "{leave}"
        );
    }
}

/// For a genuine redundancy, an early retirement scheme or invalidity, all
/// annual leave and all long service leave from 16 August 1978 are capped,
/// whenever they accrued, so their 1993 parts are not needed; the pre-16
/// August 1978 part still is.
#[test]
fn a_concessional_reason_needs_only_the_1978_part() {
    for reason in [
        "genuine_redundancy",
        "early_retirement_scheme",
        "invalidity",
    ] {
        let result = run(json!({
            "annual_leave": "1000.00", "long_service_leave": "5000.00",
            "long_service_leave_before_16_august_1978": "1000.00",
            "termination_reason": reason, "leave_accrual_start": "1976-03-01",
        }));
        assert_eq!(
            amounts(
                &result,
                [
                    "annual_leave_capped",
                    "long_service_leave_capped",
                    "marginal_rate_amount",
                    "lump_sum_a",
                    "lump_sum_b",
                ]
            ),
            ["1000.00", "4000.00", "50.00", "5000.00", "1000.00"],
            "{reason}"
        );
        assert_eq!(result["rates"]["lump_sum_a_type"], "R", "{reason}");
    }
}

/// The parts are needed from the day before each date and not after it.
#[test]
fn the_accrual_dates_decide_which_parts_are_needed() {
    let annual = json!({"annual_leave": "100.00"});
    let mut at_1993 = annual.clone();
    at_1993["leave_accrual_start"] = json!("1993-08-18");
    assert_eq!(amount(&run(at_1993), "annual_leave_marginal"), "100.00");
    let mut before_1993 = annual.clone();
    before_1993["leave_accrual_start"] = json!("1993-08-17");
    assert_eq!(code(before_1993), "leave.accrual_split_missing");
    let lsl = json!({
        "long_service_leave": "100.00",
        "long_service_leave_16_august_1978_to_17_august_1993": "0.00",
    });
    let mut at_1978 = lsl.clone();
    at_1978["leave_accrual_start"] = json!("1978-08-16");
    assert_eq!(
        amount(&run(at_1978), "long_service_leave_marginal"),
        "100.00"
    );
    let mut before_1978 = lsl.clone();
    before_1978["leave_accrual_start"] = json!("1978-08-15");
    assert_eq!(code(before_1978), "leave.accrual_split_missing");
}

/// The ATO's tax tables page (updated 17 June 2026, read 17 November 2026)
/// lists Schedules 1 and 7 for payments from 1 July 2026 and none from 1 July
/// 2027, so a 2027-28 payment is refused with that reason.
#[test]
fn a_2027_28_payment_is_refused_with_the_reason() {
    let facts = input(json!({"year": "2027-28"}));
    let refusal = api::evaluate_coded("unused-leave-payment", &facts).unwrap_err();
    assert_eq!(refusal.code, "period.unsupported");
    assert!(
        refusal
            .message
            .contains("withholding schedules for payments from 1 July 2027"),
        "{}",
        refusal.message
    );
}

#[test]
fn each_refusal_has_its_code() {
    let old = "1976-01-01";
    for (overrides, expected) in [
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"year": "2025-26"}), "period.unsupported"),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"annual_leave": "-1.00"}), "input.amount_invalid"),
        (json!({"leave_loading": "NaN"}), "input.amount_invalid"),
        (
            json!({"long_service_leave": "Infinity"}),
            "input.amount_invalid",
        ),
        (json!({"annual_leave": "1.001"}), "input.amount_invalid"),
        (
            json!({"annual_leave": "10.00", "annual_leave_before_18_august_1993": "-1.00", "leave_accrual_start": old}),
            "input.amount_invalid",
        ),
        (json!({"termination_reason": "death"}), "leave.death"),
        (
            json!({"termination_reason": "dismissal"}),
            "input.termination_reason_invalid",
        ),
        (
            json!({"working_holiday_maker": true}),
            "leave.working_holiday_maker",
        ),
        // Residency is scale 4's alone, and required there.
        (
            json!({"foreign_resident": true}),
            "payg.foreign_resident_not_scale_4",
        ),
        (json!({"scale": 4}), "input.missing_field"),
        (
            json!({"scale": 4, "foreign_resident": "no"}),
            "input.flag_invalid",
        ),
        (json!({"scale": 7}), "input.tax_scale_unsupported"),
        (
            json!({"leave_accrual_start": "1993-02-30"}),
            "input.date_invalid",
        ),
        (
            json!({"pay_period": "quarterly"}),
            "input.pay_period_invalid",
        ),
        // Leave accrued from 1976 on a normal termination: each part needed.
        (
            json!({"annual_leave": "10.00", "leave_accrual_start": old}),
            "leave.accrual_split_missing",
        ),
        (
            json!({"leave_loading": "10.00", "leave_accrual_start": old}),
            "leave.accrual_split_missing",
        ),
        (
            json!({"long_service_leave": "10.00", "long_service_leave_before_16_august_1978": "1.00", "leave_accrual_start": old}),
            "leave.accrual_split_missing",
        ),
        (
            json!({"long_service_leave": "10.00", "long_service_leave_16_august_1978_to_17_august_1993": "1.00", "leave_accrual_start": old}),
            "leave.accrual_split_missing",
        ),
        (
            json!({"long_service_leave": "10.00", "termination_reason": "genuine_redundancy", "leave_accrual_start": old}),
            "leave.accrual_split_missing",
        ),
        // A part above its total, or accrued before the accrual start allows.
        (
            json!({"annual_leave": "10.00", "annual_leave_before_18_august_1993": "10.01", "leave_accrual_start": old}),
            "leave.accrual_split_invalid",
        ),
        (
            json!({"long_service_leave": "10.00", "long_service_leave_before_16_august_1978": "6.00", "long_service_leave_16_august_1978_to_17_august_1993": "5.00", "leave_accrual_start": old}),
            "leave.accrual_split_invalid",
        ),
        (
            json!({"annual_leave": "10.00", "annual_leave_before_18_august_1993": "1.00"}),
            "leave.accrual_split_invalid",
        ),
        (
            json!({"long_service_leave": "10.00", "long_service_leave_before_16_august_1978": "1.00", "long_service_leave_16_august_1978_to_17_august_1993": "0.00", "leave_accrual_start": "1980-01-01"}),
            "leave.accrual_split_invalid",
        ),
        // Under $300 after 17 August 1993 beside pre-1978 leave.
        (
            json!({"long_service_leave": "1000.00", "long_service_leave_before_16_august_1978": "900.00", "long_service_leave_16_august_1978_to_17_august_1993": "0.00", "leave_accrual_start": old}),
            "leave.under_300_with_pre_1978",
        ),
        (json!({"sick_leave": "10.00"}), "input.field_unknown"),
    ] {
        assert_eq!(code(overrides.clone()), expected, "{overrides}");
    }
    let mut missing = input(json!({}));
    missing
        .as_object_mut()
        .unwrap()
        .remove("leave_accrual_start");
    let refusal = api::evaluate_coded("unused-leave-payment", &missing).unwrap_err();
    assert_eq!(refusal.code, "input.missing_field");
}

/// A part given as nil is accepted whatever the accrual start.
#[test]
fn a_nil_part_is_accepted_after_its_date() {
    let result = run(json!({
        "annual_leave": "10.00", "annual_leave_before_18_august_1993": "0.00",
        "long_service_leave": "10.00", "long_service_leave_before_16_august_1978": "0.00",
    }));
    assert_eq!(amount(&result, "unused_leave_on_termination"), "20.00");
}

#[test]
fn every_result_says_it_is_withholding_and_how_super_is_read() {
    let result = run(json!({"annual_leave": "1000.00"}));
    let warnings: Vec<&str> = result["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap())
        .collect();
    assert!(
        warnings
            .iter()
            .any(|w| w.starts_with("Withholding, not the tax payable")),
        "{warnings:?}"
    );
    assert!(
        warnings
            .iter()
            .any(|w| w.starts_with("qualifying_earnings"))
    );
    assert!(
        result["scope"]
            .as_str()
            .unwrap()
            .contains("leave paid after")
    );
}

/// A foreign resident who gave a TFN: Schedule 7's method when a TFN is
/// provided does not depend on residency, and its marginal rate calculation
/// uses 'the relevant PAYG withholding tax table', Schedule 1 scale 3. Norman's
/// leave on that scale: $2,054 at 32%, then ($1,440 less $1,155 withheld) x 52.
#[test]
fn a_foreign_resident_with_a_tfn_uses_scale_3() {
    let result = run(json!({
        "long_service_leave": "22020.00",
        "long_service_leave_before_16_august_1978": "780.00",
        "long_service_leave_16_august_1978_to_17_august_1993": "6420.00",
        "leave_accrual_start": "1977-01-01", "scale": 3,
    }));
    let marginal = (weekly_on("1440.00", 3) - weekly_on("1155.00", 3)) * 52;
    assert_eq!(
        amounts(
            &result,
            [
                "flat_rate_withholding",
                "marginal_withholding",
                "withholding"
            ]
        ),
        [
            "2054.00".to_string(),
            format!("{marginal}.00"),
            format!("{}.00", 2054 + marginal)
        ]
    );
}

/// Without a TFN, Schedule 7 withholds 47% from the payment, 45% for a foreign
/// resident, and its rounding ignores the cents of the result: $1,002 is
/// $470.94, so $470, not the $471 rounding would give; $1,002.99 is $471.4053,
/// so $471, where ignoring the payment's cents too would give $470. The whole
/// payment counts, the 95% of pre-16 August 1978 leave that is not assessable
/// included, and no $300 limit applies; the shares and labels do not change.
#[test]
fn without_a_tfn_47_or_45_percent_of_the_payment_is_withheld() {
    for (leave, foreign, withheld) in [
        ("1002.00", false, "470.00"),
        ("1002.99", false, "471.00"),
        ("1002.99", true, "451.00"),
    ] {
        let result = run(json!({
            "long_service_leave": leave,
            "long_service_leave_before_16_august_1978": "900.00",
            "long_service_leave_16_august_1978_to_17_august_1993": "0.00",
            "leave_accrual_start": "1976-03-01", "scale": 4, "foreign_resident": foreign,
        }));
        assert_eq!(
            amount(&result, "withholding"),
            withheld,
            "{leave} {foreign}"
        );
        assert_eq!(
            amounts(
                &result,
                ["long_service_leave_pre_1978_assessable", "lump_sum_b"]
            ),
            ["45.00", "900.00"]
        );
        assert!(result["amounts"].get("flat_rate_withholding").is_none());
    }
}
