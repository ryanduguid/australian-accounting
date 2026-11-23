// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `resident-tax` for a part-year resident: the tax-free threshold pro-rated
//! under Income Tax Rates Act 1986 s 20(1), with ss 16 and 18 (compilation 66,
//! read on 23 November 2026), against the ATO's John and Britney examples,
//! each band, the eligible pensioner rule and each refusal
//! (docs/calculation-evidence.md#part-year-tax-free-threshold).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn facts(year: &str, income: &str, months: Value) -> Value {
    json!({
        "taxable_income": income, "year": year, "scope_confirmed": true,
        "part_year_residency_months": months, "eligible_pensioner": false,
    })
}

fn run(input: &Value) -> Value {
    api::evaluate("resident-tax", input).unwrap()
}

fn amount(input: &Value, id: &str) -> String {
    let r = run(input);
    r["amounts"][id]
        .as_str()
        .unwrap_or_else(|| panic!("{id} in {r}"))
        .to_string()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("resident-tax", input).unwrap_err().code
}

/// The ATO's examples: John, a resident from 17 April, has 3 months and a
/// $14,648 threshold; Britney, a foreign resident from 22 December 2025,
/// has 6 months and $15,832.
#[test]
fn john_and_britney() {
    let john = facts("2025-26", "20000", json!(3));
    assert_eq!(amount(&john, "tax_free_threshold"), "14648.00");
    assert_eq!(amount(&john, "basic_income_tax"), "856.32");
    let britney = facts("2025-26", "100000", json!(6));
    assert_eq!(amount(&britney, "tax_free_threshold"), "15832.00");
    // Only the first band's lower limit moves: 16% to $45,000, then 30%.
    assert_eq!(amount(&britney, "basic_income_tax"), "21166.88");
    let r = run(&britney);
    assert_eq!(r["rates"]["over_tax_free_threshold"], "0.16");
    assert_eq!(r["rates"]["over_45000"], "0.30");
}

/// The Act states no rounding: one month gives $13,858.666..., kept exact.
#[test]
fn an_uneven_month_is_kept_exact() {
    let v = facts("2026-27", "20000", json!(1));
    assert_eq!(amount(&v, "tax_free_threshold"), "13858.67");
    assert_eq!(amount(&v, "basic_income_tax"), "921.20");
    // 2 months: (20,000 - 14,253.333...) x 16% = 919.4666..., shown to the cent.
    let two = facts("2025-26", "20000", json!(2));
    assert_eq!(amount(&two, "basic_income_tax"), "919.47");
    // Income at or below the pro-rated threshold bears no tax.
    let below = facts("2027-28", "16000", json!(8));
    assert_eq!(amount(&below, "tax_free_threshold"), "16621.33");
    assert_eq!(amount(&below, "basic_income_tax"), "0.00");
    // Each month count from 1 to 12, ending at the full threshold.
    for (months, threshold) in [(2, "14253.33"), (11, "17805.33"), (12, "18200.00")] {
        let v = facts("2024-25", "50000", json!(months));
        assert_eq!(amount(&v, "tax_free_threshold"), threshold, "{months}");
    }
}

/// An eligible pensioner has no part-year residency period (s 18(2)(a)), and
/// 12 months is the full year: both give the full-year tax.
#[test]
fn eligible_pensioners_and_a_full_year() {
    let full = json!({"taxable_income": "50000", "year": "2025-26", "scope_confirmed": true});
    let full_tax = amount(&full, "basic_income_tax");
    let mut pensioner = facts("2025-26", "50000", json!(3));
    pensioner["eligible_pensioner"] = json!(true);
    assert_eq!(amount(&pensioner, "tax_free_threshold"), "18200.00");
    assert_eq!(amount(&pensioner, "basic_income_tax"), full_tax);
    let twelve = facts("2025-26", "50000", json!(12));
    assert_eq!(amount(&twelve, "basic_income_tax"), full_tax);
    // Without the new inputs the result keeps its full-year form.
    let r = run(&full);
    assert!(r["amounts"].get("tax_free_threshold").is_none());
    assert_eq!(r["rates"]["over_18200"], "0.16");
}

#[test]
fn refusals() {
    for months in [
        json!(0),
        json!(13),
        json!(2.5),
        json!(3.0),
        json!("3"),
        json!(true),
        Value::Null,
    ] {
        assert_eq!(
            code(&facts("2025-26", "20000", months.clone())),
            "input.count_invalid",
            "{months}"
        );
    }
    let mut missing = facts("2025-26", "20000", json!(3));
    missing
        .as_object_mut()
        .unwrap()
        .remove("eligible_pensioner");
    assert_eq!(code(&missing), "input.missing_field");
    let mut alone = facts("2025-26", "20000", json!(3));
    alone
        .as_object_mut()
        .unwrap()
        .remove("part_year_residency_months");
    assert_eq!(code(&alone), "input.field_unknown");
    let mut flag = facts("2025-26", "20000", json!(3));
    for given in [json!("no"), Value::Null] {
        flag["eligible_pensioner"] = given;
        assert_eq!(code(&flag), "input.flag_invalid");
    }
    // A near miss of either input is refused, not ignored.
    for key in [
        "part_year_resident_months",
        "part_year",
        "is_eligible_pensioner",
    ] {
        let v = json!({"taxable_income": "20000", "year": "2025-26",
            "scope_confirmed": true, key: 3});
        assert_eq!(code(&v), "input.field_unknown", "{key}");
    }
    // Other unknown fields keep Python's lenience.
    let lenient = json!({"taxable_income": "20000", "year": "2025-26",
        "scope_confirmed": true, "note": 1});
    assert_eq!(amount(&lenient, "basic_income_tax"), "288.00");
    // The scope and year checks come first.
    let mut unconfirmed = facts("2025-26", "20000", json!(3));
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(code(&unconfirmed), "scope.not_confirmed");
    assert_eq!(
        code(&facts("2023-24", "20000", json!(3))),
        "period.unsupported"
    );
    assert_eq!(
        code(&facts("2025-26", "20000.50", json!(3))),
        "input.amount_not_whole_dollars"
    );
}
