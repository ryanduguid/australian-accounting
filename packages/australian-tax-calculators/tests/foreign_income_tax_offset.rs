// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The foreign income tax offset (ITAA 1997 Division 770, compilations 210
//! and 268) and the part-year tax-free threshold (Income Tax Rates Act 1986
//! ss 18 and 20, compilations 55 and 66), read on 22 November 2026, against
//! Examples 16 and 17 of the ATO's Guides to foreign income tax offset rules
//! 2024 to 2026 and its newcomer and departure examples
//! (docs/calculation-evidence.md#foreign-income-tax-offset).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn evaluate(input: &Value) -> Value {
    api::evaluate("foreign-income-tax-offset", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("foreign-income-tax-offset", input)
        .unwrap_err()
        .code
}

fn base(year: &str, method: &str, paid: &str) -> Value {
    json!({
        "year": year, "scope_confirmed": true, "foreign_tax_paid": paid,
        "nane_foreign_tax": "0.00", "limit_method": method,
    })
}

fn with(mut input: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            input.as_object_mut().unwrap().remove(key);
        } else {
            input[key] = value.clone();
        }
    }
    input
}

/// A worked limit: taxable income, the disregarded income and deductions and
/// the 2 Medicare amounts.
fn worked(year: &str, paid: &str, facts: [&str; 5]) -> Value {
    let [taxable, income, deductions, levy, levy_assumed] = facts;
    with(
        base(year, "worked", paid),
        json!({
            "taxable_income": taxable, "disregarded_income": income,
            "disregarded_deductions": deductions, "medicare_levy": levy,
            "medicare_levy_assumed": levy_assumed,
        }),
    )
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// Up to $1,000 needs no limit worked out; above it, the default limit is
/// $1,000 (s 770-75(2)(a) and note 1).
#[test]
fn default_limit_is_one_thousand() {
    let result = evaluate(&base("2025-26", "default", "800.00"));
    assert_eq!(amount(&result, "offset"), "800.00");
    let result = evaluate(&base("2025-26", "default", "1500.00"));
    assert_eq!(
        [
            amount(&result, "offset_limit"),
            amount(&result, "offset"),
            amount(&result, "offset_above_limit")
        ],
        ["1000.00", "1000.00", "500.00"]
    );
    let result = evaluate(&base("2020-21", "default", "1000.00"));
    assert_eq!(amount(&result, "offset_above_limit"), "0.00");
}

/// Tax on s 23AI or 23AK income counts (s 770-10(2)) and raises the limit by
/// itself (s 770-80).
#[test]
fn nane_tax_raises_the_limit() {
    let input = with(
        base("2025-26", "default", "1500.00"),
        json!({"nane_foreign_tax": "300.00"}),
    );
    let result = evaluate(&input);
    assert_eq!(
        [
            amount(&result, "offset_before_limit"),
            amount(&result, "offset_limit"),
            amount(&result, "offset")
        ],
        ["1800.00", "1300.00", "1300.00"]
    );
}

/// Example 16, 2025-26: Anna's $3,400 limited to $2,459.60 - $260.80.
#[test]
fn anna_2025_26() {
    let result = evaluate(&worked(
        "2025-26",
        "3400.00",
        ["31130", "12000.00", "700.00", "390.80", "0.00"],
    ));
    let keys = [
        "tax_free_threshold",
        "basic_income_tax",
        "tax_payable",
        "assumed_taxable_income",
        "basic_income_tax_assumed",
        "tax_payable_assumed",
        "offset_limit",
        "offset",
    ];
    let got: Vec<&str> = keys.iter().map(|k| amount(&result, k)).collect();
    assert_eq!(
        got,
        [
            "18200.00", "2068.80", "2459.60", "19830.00", "260.80", "260.80", "2198.80", "2198.80"
        ]
    );
}

/// Example 16, 2024-25: the same incomes, step 1 $2,581.80, limit $2,321.00.
#[test]
fn anna_2024_25() {
    let result = evaluate(&worked(
        "2024-25",
        "3400.00",
        ["31130", "12000.00", "700.00", "513.00", "0.00"],
    ));
    assert_eq!(amount(&result, "tax_payable"), "2581.80");
    assert_eq!(amount(&result, "offset"), "2321.00");
}

/// Example 17, 2025-26: Karen's limit of $16,388 - $9,988 = $6,400 leaves
/// her $2,000 whole.
#[test]
fn karen_2025_26() {
    let result = evaluate(&worked(
        "2025-26",
        "2000.00",
        ["80000", "20000.00", "0.00", "1600.00", "1200.00"],
    ));
    assert_eq!(
        [
            amount(&result, "tax_payable"),
            amount(&result, "tax_payable_assumed"),
            amount(&result, "offset_limit"),
            amount(&result, "offset")
        ],
        ["16388.00", "9988.00", "6400.00", "2000.00"]
    );
}

/// The 2024 guide's figures, stated: $3,079.30 - $309.70 and $18,067 -
/// $11,167. Its step 1 takes the full 2% levy at $31,130; the stated method
/// takes the figures as given.
#[test]
fn stated_figures_2023_24() {
    let stated = |paid: &str, actual: &str, assumed: &str| {
        let input = with(
            base("2023-24", "stated", paid),
            json!({"tax_payable": actual, "tax_payable_assumed": assumed}),
        );
        evaluate(&input)
    };
    let anna = stated("3400.00", "3079.30", "309.70");
    assert_eq!(
        [amount(&anna, "offset_limit"), amount(&anna, "offset")],
        ["2769.60", "2769.60"]
    );
    let karen = stated("2000.00", "18067.00", "11167.00");
    assert_eq!(
        [amount(&karen, "offset_limit"), amount(&karen, "offset")],
        ["6900.00", "2000.00"]
    );
    // Paragraph (b) below $1,000, or negative: the limit is $1,000.
    let low = stated("1500.00", "500.00", "900.00");
    assert_eq!(amount(&low, "offset_limit"), "1000.00");
}

/// John arrived on 17 April: 3 months, $14,648 (NEW); Britney left on
/// 22 December 2025: 6 months, $15,832 (LEAVE). The 16% band starts there.
#[test]
fn part_year_thresholds() {
    let periods = |from: &str, to: &str| json!([{"from": from, "to": to}]);
    let john = with(
        worked(
            "2025-26",
            "2000.00",
            ["30000", "10000.00", "0.00", "0.00", "0.00"],
        ),
        json!({"residency_periods": periods("2026-04-17", "2026-06-30")}),
    );
    let result = evaluate(&john);
    assert_eq!(amount(&result, "tax_free_threshold"), "14648.00");
    assert_eq!(result["rates"]["residency_months"], "3");
    // (30,000 - 14,648) x 16% and (20,000 - 14,648) x 16%.
    assert_eq!(amount(&result, "basic_income_tax"), "2456.32");
    assert_eq!(amount(&result, "basic_income_tax_assumed"), "856.32");
    let britney = with(
        worked(
            "2025-26",
            "500.00",
            ["60000", "0.00", "0.00", "0.00", "0.00"],
        ),
        json!({"residency_periods": periods("2025-07-01", "2025-12-22")}),
    );
    let result = evaluate(&britney);
    assert_eq!(amount(&result, "tax_free_threshold"), "15832.00");
    // The other thresholds do not move: (45,000 - 15,832) x 16% + 15,000 x 30%.
    assert_eq!(amount(&result, "basic_income_tax"), "9166.88");
}

/// Months are counted once, the month of each change included; 5 months
/// leave a threshold in cents; an eligible pensioner keeps $18,200.
#[test]
fn residency_months_and_pensioners() {
    let input = |periods: Value| {
        with(
            worked(
                "2025-26",
                "100.00",
                ["20000", "0.00", "0.00", "0.00", "0.00"],
            ),
            json!({"residency_periods": periods}),
        )
    };
    let two = json!([{"from": "2025-07-01", "to": "2025-08-03"}, {"from": "2025-08-20", "to": "2025-10-01"}]);
    let result = evaluate(&input(two));
    assert_eq!(result["rates"]["residency_months"], "4");
    assert_eq!(amount(&result, "tax_free_threshold"), "15042.67");
    let five = json!([{"from": "2026-02-01", "to": "2026-06-30"}]);
    assert_eq!(
        amount(&evaluate(&input(five)), "tax_free_threshold"),
        "15437.33"
    );
    let pensioner = with(
        input(json!([{"from": "2026-02-01", "to": "2026-06-30"}])),
        json!({"eligible_pensioner": true}),
    );
    let result = evaluate(&pensioner);
    assert_eq!(amount(&result, "tax_free_threshold"), "18200.00");
    assert!(result["rates"]["residency_months"].is_null());
}

/// 2026-27's first band is 15%.
#[test]
fn worked_2026_27_rate() {
    let result = evaluate(&worked(
        "2026-27",
        "100.00",
        ["28200", "0.00", "0.00", "0.00", "0.00"],
    ));
    assert_eq!(amount(&result, "basic_income_tax"), "1500.00");
}

/// Anna's 2025-26 worked input (Example 16).
fn anna() -> Value {
    let facts = ["31130", "12000.00", "700.00", "390.80", "0.00"];
    worked("2025-26", "3400.00", facts)
}

/// Each change to Anna's input, and the code it is refused with.
fn refused(cases: &[(Value, &str)]) {
    for (change, expected) in cases {
        assert_eq!(code(&with(anna(), change.clone())), *expected, "{change}");
    }
}

#[test]
fn worked_refusals() {
    let early = worked(
        "2023-24",
        "3400.00",
        ["31130", "12000.00", "700.00", "622.60", "0.00"],
    );
    assert_eq!(code(&early), "foreign_income_tax_offset.worked_year");
    refused(&[
        (
            json!({"limit_method": "computed"}),
            "foreign_income_tax_offset.limit_method_invalid",
        ),
        (json!({"medicare_levy": "remove"}), "input.missing_field"),
        (json!({"tax_payable": "100.00"}), "input.field_unknown"),
        (json!({"eligible_pensioner": true}), "input.field_unknown"),
        (
            json!({"taxable_income": "31130.50"}),
            "input.amount_not_whole_dollars",
        ),
    ]);
}

#[test]
fn residency_period_refusals() {
    let invalid = "foreign_income_tax_offset.residency_periods_invalid";
    let periods = |list: Value| json!({"residency_periods": list});
    refused(&[
        (periods(json!([])), invalid),
        (
            periods(json!([{"from": "2025-06-30", "to": "2025-12-31"}])),
            invalid,
        ),
        (
            periods(json!([{"from": "2026-01-31", "to": "2026-01-01"}])),
            invalid,
        ),
        (
            periods(json!([{"from": "2026-01-01"}])),
            "input.missing_field",
        ),
        (
            periods(json!([{"from": "2026-01-01", "to": "2026-02-30"}])),
            "input.date_invalid",
        ),
        (
            periods(json!([{"from": "2026-01-01", "to": "2026-02-01", "days": 3}])),
            "input.field_unknown",
        ),
    ]);
}

#[test]
fn default_and_stated_refusals() {
    let default = base("2025-26", "default", "800.00");
    let extra = with(default, json!({"taxable_income": "1000"}));
    assert_eq!(code(&extra), "input.field_unknown");
    let stated = with(
        base("2025-26", "stated", "800.00"),
        json!({"tax_payable": "100.00"}),
    );
    assert_eq!(code(&stated), "input.missing_field");
    assert_eq!(
        code(&base("2025-26", "default", "-1")),
        "input.amount_invalid"
    );
    assert_eq!(
        code(&base("2019-20", "default", "800.00")),
        "period.unsupported"
    );
}

/// The worked example's steps explain every amount.
#[test]
fn derive_explains_the_worked_limit() {
    let input = worked(
        "2025-26",
        "3400.00",
        ["31130", "12000.00", "700.00", "390.80", "0.00"],
    );
    let request = json!({"command": "foreign-income-tax-offset", "input": input});
    let doc = api::evaluate("derive", &request).unwrap();
    let ids: Vec<&str> = doc["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.first(), Some(&"tax_free_threshold"));
    assert_eq!(ids.last(), Some(&"offset_above_limit"));
}
