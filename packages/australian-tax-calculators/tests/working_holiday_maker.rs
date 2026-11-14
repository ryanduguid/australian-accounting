// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Schedule 15 withholding for working holiday makers against the ATO's
//! lookup tool example and examples 1 and 2
//! (docs/calculation-evidence.md#working-holiday-makers), each Table A band
//! boundary, the rounding rules, the unregistered employer's foreign resident
//! rates and the refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "working-holiday-maker-withholding";

fn input(payment: &str, before: &str, pay_period: &str, year: &str) -> Value {
    json!({
        "payment": payment, "payments_before": before, "employer_registered": true,
        "tfn_quoted": true, "pay_period": pay_period, "year": year, "scope_confirmed": true,
    })
}

fn with(overrides: Value) -> Value {
    let mut facts = input("680.70", "0.00", "weekly", "2026-27");
    for (k, v) in overrides.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn evaluate(facts: &Value) -> Value {
    api::evaluate(COMMAND, facts).unwrap()
}

/// The payment used (cents ignored), the withholding and the rate.
fn run(facts: &Value) -> (String, String, String) {
    let result = evaluate(facts);
    let text = |v: &Value| v.as_str().unwrap().to_string();
    (
        text(&result["amounts"]["payment_used"]),
        text(&result["amounts"]["withholding"]),
        text(&result["rates"]["rate"]),
    )
}

fn withholding(payment: &str, before: &str) -> String {
    run(&input(payment, before, "monthly", "2026-27")).1
}

fn refusal_code(facts: &Value) -> &'static str {
    api::evaluate_coded(COMMAND, facts).unwrap_err().code
}

fn triple(x: &str, y: &str, a: &str) -> (String, String, String) {
    (x.into(), y.into(), a.into())
}

/// The 3 published examples, which read the same in the schedule for
/// 1 July 2024 to 30 June 2026 and in the one from 1 July 2026.
#[test]
fn the_ato_examples() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        // The lookup tool: $680.70 for the week, payments not over $45,000.
        assert_eq!(
            run(&input("680.70", "0.00", "weekly", year)),
            triple("680.00", "102.00", "0.15"),
            "{year}"
        );
        // Example 1: 0.15 x 2,825 = 423.75, or $424.
        assert_eq!(
            run(&input("2825.75", "0.00", "monthly", year)),
            triple("2825.00", "424.00", "0.15"),
            "{year}"
        );
        // Example 2: $47,000 paid before, so 0.30 x 8,173 = 2,451.90, or $2,452.
        assert_eq!(
            run(&input("8173.20", "47000.00", "monthly", year)),
            triple("8173.00", "2452.00", "0.30"),
            "{year}"
        );
    }
    let result = evaluate(&input("8173.20", "47000.00", "monthly", "2026-27"));
    assert_eq!(result["period"], "2026-27 monthly");
    assert_eq!(
        result["withholding_basis"],
        "registered employer, TFN quoted: column 3, rate 0.30 on the payment with cents ignored"
    );
    assert_eq!(result["source_checked"], "2026-11-14");
}

/// Columns 2 to 5 take the payments made before this one: less than $45,001,
/// $45,001 to $135,000, $135,001 to $190,000 and more than $190,000.
#[test]
fn each_band_boundary() {
    for (before, rate) in [
        ("0.00", "0.15"),
        ("44999.99", "0.15"),
        ("45000.00", "0.15"),
        ("45000.50", "0.15"),
        ("45000.99", "0.15"),
        ("45001.00", "0.30"),
        ("135000.00", "0.30"),
        ("135001.00", "0.37"),
        ("190000.00", "0.37"),
        ("190000.01", "0.45"),
        ("190000.99", "0.45"),
        ("190001.00", "0.45"),
        ("5000000.00", "0.45"),
    ] {
        assert_eq!(
            run(&input("100.00", before, "weekly", "2026-27")).2,
            rate,
            "{before}"
        );
    }
}

/// No column covers $135,000.01 to $135,000.99, so they are refused, and the
/// refusal names both columns' rates and Table A's 37% for the payer to confirm.
#[test]
fn payments_before_between_columns_3_and_4_are_refused() {
    for before in ["135000.01", "135000.50", "135000.99"] {
        assert_eq!(
            refusal_code(&with(json!({"payments_before": before}))),
            "working_holiday_maker.band_unsettled",
            "{before}"
        );
    }
    let refused =
        api::evaluate_coded(COMMAND, &with(json!({"payments_before": "135000.50"}))).unwrap_err();
    assert_eq!(
        refused.message,
        "payments_before of $135,000.01 to $135,000.99 is in none of Schedule 15's columns: column 3 (30%) ends at $135,000 and column 4 (37%) starts at $135,001. Table A's rate column gives 37% on each $1 over $135,000, but its income bands leave the same gap, so the worksheet does not choose; confirm the rate with the ATO before withholding."
    );
}

/// The result is rounded to the nearest dollar, 50 cents up, directly from
/// the unrounded product; at the top rate and without a TFN, cents are ignored.
#[test]
fn rounding() {
    for (payment, before, y) in [
        // 0.15 x 10 = 1.50, rounded up.
        ("10.99", "0.00", "2.00"),
        // 0.15 x 3 = 0.45, rounded down.
        ("3.00", "0.00", "0.00"),
        // Cents ignored before the rate applies: 0.15 x 3.99 would be 0.60.
        ("3.99", "0.00", "0.00"),
        // 0.30 x 5 = 1.50.
        ("5.00", "45001.00", "2.00"),
        // 0.37 x 50 = 18.50.
        ("50.00", "135001.00", "19.00"),
        // 0.45 x 10 = 4.50, cents ignored at the top rate.
        ("10.00", "190001.00", "4.00"),
        ("0.99", "0.00", "0.00"),
        // So a small payment can withhold less once column 5 applies:
        // 0.37 x 2 = 0.74, rounded up, against 0.45 x 2 = 0.90, cents ignored.
        ("2.00", "190000.00", "1.00"),
        ("2.00", "190000.01", "0.00"),
    ] {
        assert_eq!(withholding(payment, before), y, "{payment} {before}");
    }
}

/// Without a TFN any employer withholds 45%, cents ignored, whatever was paid
/// before: column 6 for a registered employer, and for an unregistered one the
/// foreign resident rate without a TFN, which Schedule 1 scale 4 also gives.
#[test]
fn no_tfn_is_45_percent_with_cents_ignored() {
    for (registered, basis) in [
        (
            true,
            "registered employer, no TFN: column 6, rate 0.45 on the payment with cents ignored",
        ),
        (
            false,
            "unregistered employer, no TFN: the foreign resident rate without a TFN (Schedule 1 scale 4), 0.45 on the payment with cents ignored",
        ),
    ] {
        for (payment, before, x, y) in [
            ("10.00", "0.00", "10.00", "4.00"),
            ("10.99", "0.00", "10.00", "4.00"),
            ("1000.99", "0.00", "1000.00", "450.00"),
            ("2825.75", "135000.50", "2825.00", "1271.00"),
        ] {
            for year in ["2024-25", "2025-26", "2026-27"] {
                let facts = with(json!({"payment": payment, "payments_before": before,
                                        "tfn_quoted": false, "employer_registered": registered,
                                        "year": year}));
                assert_eq!(run(&facts), triple(x, y, "0.45"), "{payment} {registered}");
                assert_eq!(evaluate(&facts)["withholding_basis"], basis);
                let scale_4 = api::evaluate(
                    "payg-withholding",
                    &json!({"earnings": payment, "pay_period": "weekly", "scale": 4,
                            "foreign_resident": true, "year": year, "scope_confirmed": true}),
                )
                .unwrap();
                assert_eq!(scale_4["amounts"]["withholding"], y, "{payment} {year}");
            }
        }
    }
}

/// An unregistered employer withholds at foreign resident rates: the
/// Schedule 1 scale 3 amount that payg-withholding gives for the pay period.
#[test]
fn an_unregistered_employer_uses_foreign_resident_rates() {
    for (payment, period, year) in [
        ("680.70", "weekly", "2026-27"),
        ("2825.75", "monthly", "2025-26"),
        ("8173.20", "fortnightly", "2024-25"),
    ] {
        let facts = with(
            json!({"payment": payment, "pay_period": period, "year": year,
                                "employer_registered": false, "payments_before": "47000.00"}),
        );
        let result = evaluate(&facts);
        let payg = api::evaluate(
            "payg-withholding",
            &json!({"earnings": payment, "pay_period": period, "scale": 3, "year": year,
                    "scope_confirmed": true}),
        )
        .unwrap();
        assert_eq!(
            result["amounts"]["withholding"], payg["amounts"]["withholding"],
            "{payment} {period} {year}"
        );
        assert_eq!(result["amounts"]["payment_used"], payment);
        assert_eq!(result["rates"]["scale"], "3");
        assert_eq!(result["period"], format!("{year} {period}"));
    }
    // 2026-27 scale 3, weekly: 0.30 x 680.99 - 0.30 = 203.997, or $204.
    assert_eq!(
        evaluate(&with(json!({"employer_registered": false})))["amounts"]["withholding"],
        "204.00"
    );
}

#[test]
fn out_of_scope_and_malformed_input_is_refused() {
    for (overrides, code) in [
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"year": "2023-24"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (
            json!({"pay_period": "quarterly"}),
            "input.pay_period_invalid",
        ),
        (json!({"pay_period": "annual"}), "input.pay_period_invalid"),
        (json!({"pay_period": 7}), "input.pay_period_invalid"),
        (
            json!({"pay_period": "quarterly", "employer_registered": false}),
            "input.pay_period_invalid",
        ),
        (json!({"payment": "-0.01"}), "input.amount_invalid"),
        (json!({"payment": "680.701"}), "input.amount_invalid"),
        (json!({"payment": 680}), "input.amount_invalid"),
        (json!({"payments_before": "-1.00"}), "input.amount_invalid"),
        (json!({"employer_registered": "yes"}), "input.flag_invalid"),
        (json!({"tfn_quoted": null}), "input.flag_invalid"),
        // A field the worksheet does not take is refused, never ignored.
        (
            json!({"tfn_exemption_claimed": true}),
            "input.field_unknown",
        ),
        (json!({"study_loan": true}), "input.field_unknown"),
    ] {
        assert_eq!(refusal_code(&with(overrides.clone())), code, "{overrides}");
    }
    for field in [
        "payment",
        "payments_before",
        "employer_registered",
        "tfn_quoted",
        "pay_period",
        "year",
        "scope_confirmed",
    ] {
        let mut facts = with(json!({}));
        facts.as_object_mut().unwrap().remove(field);
        assert_eq!(refusal_code(&facts), "input.missing_field", "{field}");
    }
}
