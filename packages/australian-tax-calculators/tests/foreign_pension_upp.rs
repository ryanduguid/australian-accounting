// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The deductible amount of a foreign pension's undeducted purchase price
//! (ITAA 1936 s 27H, read on 24 November 2026) against the ATO's 2026 D11
//! shortcuts for UK and Dutch pensions and hand-worked cases of the s 27H(2)
//! formula, as the ATO publishes no numeric foreign pension example
//! (docs/calculation-evidence.md#foreign-pension-upp).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("foreign-pension-upp", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("foreign-pension-upp", input)
        .unwrap_err()
        .code
}

fn formula(annuity: &str, price: &str, residual: &str, basis: &str, years: &str) -> Value {
    json!({
        "year": "2025-26", "scope_confirmed": true, "method": "formula",
        "annuity_derived": annuity, "undeducted_purchase_price": price,
        "residual_capital_value": residual,
        "relevant_number": {"basis": basis, "years": years},
        "commuted": false, "whole_year": true,
    })
}

fn shortcut(method: &str, annuity: &str) -> Value {
    json!({"year": "2025-26", "scope_confirmed": true, "method": method, "annuity_derived": annuity})
}

fn with(mut input: Value, key: &str, value: Value) -> Value {
    input[key] = value;
    input
}

fn without(mut input: Value, key: &str) -> Value {
    input.as_object_mut().unwrap().remove(key);
    input
}

fn amount(out: &Value, key: &str) -> String {
    out["amounts"][key].as_str().unwrap().to_owned()
}

#[test]
fn a_life_pension_over_a_life_expectation_factor() {
    // $100,000 / 20 = $5,000 a year, as in the ATO's premise of a $100,000
    // purchase price giving a $5,000 deductible amount.
    let out = run(&formula(
        "12000.00",
        "100000.00",
        "0.00",
        "life_expectation_factor",
        "20",
    ));
    assert_eq!(amount(&out, "deductible_amount"), "5000.00");
    assert_eq!(amount(&out, "excluded_amount"), "5000.00");
    assert_eq!(amount(&out, "assessable_amount"), "7000.00");
    assert_eq!(
        out["rates"]["relevant_number_basis"],
        "life_expectation_factor"
    );
}

#[test]
fn a_term_certain_with_a_residual_capital_value() {
    // ($50,000 - $10,000) / 10 years.
    let out = run(&formula(
        "6000.00",
        "50000.00",
        "10000.00",
        "term_years",
        "10",
    ));
    assert_eq!(amount(&out, "deductible_amount"), "4000.00");
    assert_eq!(amount(&out, "assessable_amount"), "2000.00");
}

#[test]
fn a_shared_annuity_takes_the_whole_dollar_share() {
    // A = $6,000 / $12,001 (whole dollars), so $5,000 x A = $2,499.79.
    let input = with(
        formula(
            "6000.80",
            "100000.00",
            "0.00",
            "life_expectation_factor",
            "20",
        ),
        "total_annuity_derived",
        json!("12001.60"),
    );
    let out = run(&input);
    assert_eq!(amount(&out, "deductible_amount"), "2499.79");
    assert_eq!(amount(&out, "assessable_amount"), "3501.01");
}

#[test]
fn the_deductible_amount_cannot_exclude_more_than_the_annuity() {
    let out = run(&formula(
        "15000.00",
        "100000.00",
        "0.00",
        "life_expectation_factor",
        "5",
    ));
    assert_eq!(amount(&out, "deductible_amount"), "20000.00");
    assert_eq!(amount(&out, "excluded_amount"), "15000.00");
    assert_eq!(amount(&out, "assessable_amount"), "0.00");
}

#[test]
fn the_atos_uk_and_dutch_shortcuts() {
    // D11 (2026): UK category A or B widow's 8%, C or D nil; Dutch SVB 25%.
    for (category, deductible) in [
        ("a", "800.00"),
        ("b_widow", "800.00"),
        ("c", "0.00"),
        ("d", "0.00"),
    ] {
        let input = with(
            shortcut("uk_state_pension", "10000.00"),
            "uk_category",
            json!(category),
        );
        assert_eq!(
            amount(&run(&input), "deductible_amount"),
            deductible,
            "{category}"
        );
    }
    let dutch = run(&shortcut("dutch_svb_fallback", "8000.00"));
    assert_eq!(amount(&dutch, "deductible_amount"), "2000.00");
    assert_eq!(amount(&dutch, "assessable_amount"), "6000.00");
}

#[test]
fn the_shortcuts_are_for_2025_26_only() {
    let uk = with(
        shortcut("uk_state_pension", "100.00"),
        "uk_category",
        json!("a"),
    );
    for year in ["2024-25", "2026-27"] {
        assert_eq!(
            code(&with(uk.clone(), "year", json!(year))),
            "foreign_pension_upp.method_year_unsupported"
        );
    }
    let early = with(
        formula("100.00", "1000.00", "0.00", "term_years", "10"),
        "year",
        json!("2007-08"),
    );
    assert_eq!(amount(&run(&early), "deductible_amount"), "100.00");
    assert_eq!(
        code(&with(early, "year", json!("2006-07"))),
        "period.unsupported"
    );
}

#[test]
fn refusals() {
    let ok = formula("100.00", "1000.00", "0.00", "term_years", "10");
    let cases = [
        (
            with(ok.clone(), "method", json!("other")),
            "foreign_pension_upp.method_invalid",
        ),
        (
            with(ok.clone(), "commuted", json!(true)),
            "foreign_pension_upp.commuted",
        ),
        (
            with(ok.clone(), "whole_year", json!(false)),
            "foreign_pension_upp.part_year",
        ),
        (
            with(ok.clone(), "residual_capital_value", json!("1000.01")),
            "foreign_pension_upp.residual_exceeds_price",
        ),
        (
            with(ok.clone(), "total_annuity_derived", json!("99.00")),
            "foreign_pension_upp.share_invalid",
        ),
        (
            without(ok.clone(), "relevant_number"),
            "input.missing_field",
        ),
        (
            with(ok.clone(), "uk_category", json!("a")),
            "input.field_unknown",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(code(&input), expected, "{input}");
    }
}

#[test]
fn shortcut_refusals() {
    let uk = shortcut("uk_state_pension", "1.00");
    let dutch = shortcut("dutch_svb_fallback", "1.00");
    let cases = [
        (
            with(uk.clone(), "uk_category", json!("e")),
            "foreign_pension_upp.uk_category_invalid",
        ),
        (uk, "input.missing_field"),
        (with(dutch, "commuted", json!(false)), "input.field_unknown"),
    ];
    for (input, expected) in cases {
        assert_eq!(code(&input), expected, "{input}");
    }
}

#[test]
fn only_a_term_of_years_or_one_life_is_a_relevant_number() {
    let ok = formula("100.00", "1000.00", "0.00", "term_years", "10");
    for number in [
        json!({"basis": "commissioner", "years": "10"}),
        json!({"basis": "term_years", "years": "0"}),
        json!({"basis": "term_years", "years": "-1"}),
        json!({"basis": "term_years", "years": "Infinity"}),
        json!({"basis": "term_years", "years": 10}),
        json!({"basis": "term_years", "years": "10", "extra": 1}),
        json!("10"),
    ] {
        assert_eq!(
            code(&with(ok.clone(), "relevant_number", number)),
            "foreign_pension_upp.relevant_number_invalid"
        );
    }
    // s 27H(4)(a) sets no longest term: 151 years certain is a relevant number.
    let long = with(
        formula("100.00", "15100.00", "0.00", "term_years", "151"),
        "relevant_number",
        json!({"basis": "term_years", "years": "151"}),
    );
    assert_eq!(amount(&run(&long), "deductible_amount"), "100.00");
}
