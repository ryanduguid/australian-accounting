// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Schedule 8 by pay date, through the public JSON boundary. For 2025-26 the
//! component coefficients and examples are those of Schedule 8 to the Taxation
//! Administration (Withholding Schedules) Instrument (No. 2) 2025 (F2025L01016),
//! which commenced on 24 September 2025; for 2026-27, the ATO's Schedule 8 for
//! payments from 1 July 2026. Both were read on 14 November 2026.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(earnings: &str, period: &str, scale: i64, year: &str, extra: Value) -> Value {
    let mut input = json!({"earnings": earnings, "pay_period": period, "scale": scale,
                           "year": year, "scope_confirmed": true});
    for (key, value) in extra.as_object().unwrap() {
        input[key] = value.clone();
    }
    input
}

fn run(earnings: &str, period: &str, scale: i64, year: &str, extra: Value) -> Value {
    let input = input(earnings, period, scale, year, extra);
    api::evaluate("study-loan-withholding", &input).unwrap_or_else(|e| panic!("{input}: {e}"))
}

fn refused(year: &str, extra: Value) -> (String, String) {
    let input = input("2000.00", "weekly", 2, year, extra);
    let refusal = api::evaluate_coded("study-loan-withholding", &input).unwrap_err();
    (refusal.code.to_string(), refusal.message)
}

fn dollars(result: &Value, key: &str) -> i64 {
    let text = result["amounts"][key]
        .as_str()
        .unwrap_or_else(|| panic!("no {key}: {result}"));
    text.strip_suffix(".00").unwrap().parse().unwrap()
}

/// The Schedule 1 amount the payg-withholding worksheet gives the same pay.
fn schedule_1(earnings: &str, period: &str, scale: i64, year: &str) -> i64 {
    let input = input(earnings, period, scale, year, json!({}));
    dollars(
        &api::evaluate("payg-withholding", &input).unwrap(),
        "withholding",
    )
}

#[test]
fn the_2025_26_examples_use_the_coefficients_from_24_september_2025() {
    // Examples 1 to 3: weekly $2,608.36 and fortnightly $4,409.75 with the
    // tax-free threshold, monthly $10,627.88 without it.
    for (earnings, period, scale, x, component) in [
        ("2608.36", "weekly", 2, "2608.99", 202),
        ("4409.75", "fortnightly", 2, "2204.99", 274),
        ("10627.88", "monthly", 1, "2452.99", 1018),
    ] {
        for pay_date in ["2025-09-24", "2026-06-30"] {
            let result = run(
                earnings,
                period,
                scale,
                "2025-26",
                json!({"pay_date": pay_date}),
            );
            assert_eq!(result["amounts"]["weekly_earnings_used"], json!(x));
            assert_eq!(
                dollars(&result, "study_loan_component"),
                component,
                "{earnings}"
            );
            let tax = schedule_1(earnings, period, scale, "2025-26");
            assert_eq!(dollars(&result, "schedule_1_withholding"), tax);
            assert_eq!(dollars(&result, "withholding"), tax + component);
        }
    }
    let result = run(
        "2608.36",
        "weekly",
        2,
        "2025-26",
        json!({"pay_date": "2025-09-24"}),
    );
    assert_eq!(result["rates"]["component_a"], json!("0.17"));
    assert_eq!(result["rates"]["component_b"], json!("241.3462"));
}

#[test]
fn the_2026_27_examples_keep_their_own_coefficients() {
    // The same earnings under the 2026 schedule: 0.17 x 2,608.99 - 250.4527 is
    // $193; 0.15 x 2,204.99 - 200.5615 is $130, so $260; 0.17 x 2,452.99 -
    // 190.9527 is $226, so $979.
    for (earnings, period, scale, component) in [
        ("2608.36", "weekly", 2, 193),
        ("4409.75", "fortnightly", 2, 260),
        ("10627.88", "monthly", 1, 979),
    ] {
        let plain = run(earnings, period, scale, "2026-27", json!({}));
        assert_eq!(
            dollars(&plain, "study_loan_component"),
            component,
            "{earnings}"
        );
        for pay_date in ["2026-07-01", "2027-06-30"] {
            let dated = run(
                earnings,
                period,
                scale,
                "2026-27",
                json!({"pay_date": pay_date}),
            );
            assert_eq!(dated, plain);
        }
    }
}

#[test]
fn a_2025_26_pay_before_24_september_2025_is_refused() {
    let (code, message) = refused("2025-26", json!({"pay_date": "2025-09-23"}));
    assert_eq!(code, "payg.pay_date_before_schedule");
    assert!(message.contains("on or after 2025-09-24"), "{message}");
    assert_eq!(
        refused("2025-26", json!({"pay_date": "2025-07-01"})).0,
        code
    );
    run(
        "2000.00",
        "weekly",
        2,
        "2025-26",
        json!({"pay_date": "2025-09-24"}),
    );
}

#[test]
fn a_2025_26_pay_needs_its_date() {
    let (code, message) = refused("2025-26", json!({}));
    assert_eq!(code, "input.missing_field");
    assert_eq!(
        message,
        "pay_date is required for 2025-26: its Schedule 8 coefficients apply to payments made on or after 2025-09-24."
    );
}

#[test]
fn a_pay_date_outside_the_income_year_or_malformed_is_refused() {
    for (year, pay_date) in [
        ("2025-26", "2025-06-30"),
        ("2025-26", "2026-07-01"),
        ("2026-27", "2026-06-30"),
        ("2026-27", "2027-07-01"),
    ] {
        let (code, message) = refused(year, json!({"pay_date": pay_date}));
        assert_eq!(code, "payg.pay_date_outside_year", "{year} {pay_date}");
        assert_eq!(
            message,
            format!("pay_date {pay_date} is outside the {year} income year.")
        );
    }
    for pay_date in [
        json!("2025-9-24"),
        json!("2025-09-31"),
        json!(20250924),
        json!(null),
    ] {
        let (code, _) = refused("2025-26", json!({"pay_date": pay_date}));
        assert_eq!(code, "input.date_invalid", "{pay_date}");
    }
    let (code, message) = refused("2026-27", json!({"paydate": "2026-07-01"}));
    assert_eq!(code, "input.field_unknown");
    assert_eq!(message, "paydate is not an input of study-loan-withholding");
}

#[test]
fn the_component_starts_at_each_2025_26_threshold() {
    // F2025L01016's starting earnings: $1,288 weekly, $2,576 fortnightly and
    // $5,581.33 monthly with the tax-free threshold; $938, $1,876 and $4,064.67
    // without it. A cent less stays on the nil row.
    let component = |earnings: &str, period: &str, scale: i64| {
        let result = run(
            earnings,
            period,
            scale,
            "2025-26",
            json!({"pay_date": "2025-09-24"}),
        );
        (
            dollars(&result, "study_loan_component"),
            result["rates"]["component_a"].as_str().unwrap().to_string(),
        )
    };
    for (scale, period, below, from) in [
        (2, "weekly", "1287.99", "1288.00"),
        (2, "fortnightly", "2575.99", "2576.00"),
        (2, "monthly", "5581.32", "5581.33"),
        (1, "weekly", "937.99", "938.00"),
        (1, "fortnightly", "1875.99", "1876.00"),
        (1, "monthly", "4064.66", "4064.67"),
    ] {
        assert_eq!(component(below, period, scale), (0, "0".into()), "{below}");
        assert_eq!(component(from, period, scale).1, "0.15", "{from}");
    }
    // 0.15 x 1,288.99 less 193.2692 is under 50 cents; the first dollar is at
    // x = 1,291.99. Then each later row edge on scale 1.
    assert_eq!(component("1288.00", "weekly", 2), (0, "0.15".into()));
    assert_eq!(component("1291.00", "weekly", 2), (1, "0.15".into()));
    assert_eq!(component("2052.99", "weekly", 1), (167, "0.15".into()));
    assert_eq!(component("2053.00", "weekly", 1), (167, "0.17".into()));
    assert_eq!(component("2596.99", "weekly", 1), (260, "0.17".into()));
    assert_eq!(component("2597.00", "weekly", 1), (260, "0.10".into()));
}

#[test]
fn periods_start_2025_26_on_24_september_2025() {
    let periods = api::evaluate("periods", &json!({"kind": "study_loan_withholding"})).unwrap();
    assert_eq!(
        periods,
        json!([
            {"id": "2025-26", "basis": "income year", "start": "2025-09-24",
             "end": "2026-06-30", "arguments": {"year": "2025-26"}},
            {"id": "2026-27", "basis": "income year", "start": "2026-07-01",
             "end": "2027-06-30", "arguments": {"year": "2026-27"}},
        ])
    );
}
