// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The non-commercial loss rules (ITAA 1997 Division 35, compilations 210,
//! 223 and 268, read on 22 November 2026) against the ATO's examples on its
//! 'Four tests', 'The income requirement' and 'How to defer your losses'
//! pages (each last updated 7 April 2025), the s 35-10 Jennifer example and
//! hand-worked cases at each threshold
//! (docs/calculation-evidence.md#non-commercial-losses).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("non-commercial-losses", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("non-commercial-losses", input)
        .unwrap_err()
        .code
}

fn with(mut base: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            base.as_object_mut().unwrap().remove(key);
        } else {
            base[key] = value.clone();
        }
    }
    base
}

/// A loss-making activity that passes no test, with $60,000 of taxable
/// income after its $1,000 loss.
fn base() -> Value {
    json!({
        "year": "2025-26", "scope_confirmed": true, "activity": "other",
        "assessable_income": "9000.00", "deductions": "10000.00", "brought_forward": "0.00",
        "part_year": false, "earlier_years": [null, null, null, null],
        "real_property_value": "0.00", "other_assets_value": "0.00",
        "taxable_income": "59000.00", "business_losses": "1000.00",
        "assessable_fhss_released_amount": "0.00", "reportable_fringe_benefits": "0.00",
        "reportable_super_contributions": "0.00", "total_net_investment_loss": "0.00",
        "net_exempt_income": "0.00",
    })
}

fn met(result: &Value, id: &str) -> bool {
    let tests = result["tests"].as_array().unwrap();
    let row = tests.iter().find(|t| t["test"] == id);
    row.unwrap_or_else(|| panic!("no {id}"))["met"]
        .as_bool()
        .unwrap()
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// Slaide: $16,000 for part of the year, a reasonable full-year estimate of
/// $22,000, so the assessable income test is met.
#[test]
fn slaide_passes_the_assessable_income_test_on_his_estimate() {
    let slaide = with(
        base(),
        json!({"assessable_income": "16000.00", "deductions": "18000.00", "part_year": true,
            "assessable_income_estimate": "22000.00", "taxable_income": "48000.00",
            "business_losses": "2000.00"}),
    );
    let result = run(&slaide);
    assert!(met(&result, "assessable_income_test"));
    assert_eq!(amount(&result, "assessable_income_measured"), "22000.00");
    assert_eq!(result["rule_applies"], false);
    assert_eq!(amount(&result, "excess"), "2000.00");
    assert_eq!(amount(&result, "deferred"), "0.00");
    assert_eq!(amount(&result, "deductible_this_year"), "2000.00");
    // Without the part year, the $16,000 is measured and fails.
    let full = run(&with(
        slaide,
        json!({"part_year": false, "assessable_income_estimate": "remove"}),
    ));
    assert!(!met(&full, "assessable_income_test"));
}

/// Nguyen: a $7,500 tax loss this year after profits of $17,500 and $15,000,
/// a $4,000 loss and a $16,000 profit: 3 of 5 years, so the profits test.
#[test]
fn nguyen_passes_the_profits_test() {
    let nguyen = with(
        base(),
        json!({"assessable_income": "10000.00", "deductions": "17500.00",
        "earlier_years": [
            {"assessable_income": "40000.00", "deductions": "22500.00"},
            {"assessable_income": "35000.00", "deductions": "20000.00"},
            {"assessable_income": "20000.00", "deductions": "24000.00"},
            {"assessable_income": "36000.00", "deductions": "20000.00"},
        ]}),
    );
    let result = run(&nguyen);
    assert!(met(&result, "profits_test"));
    assert_eq!(result["profit_years"], 3);
    assert_eq!(result["rule_applies"], false);
    // A break-even year is not a profit.
    let even = run(&with(
        nguyen,
        json!({"earlier_years": [
            {"assessable_income": "40000.00", "deductions": "22500.00"},
            {"assessable_income": "35000.00", "deductions": "35000.00"},
            {"assessable_income": "20000.00", "deductions": "24000.00"},
            {"assessable_income": "36000.00", "deductions": "20000.00"},
        ]}),
    ));
    assert_eq!(even["profit_years"], 2);
    assert!(!met(&even, "profits_test"));
}

/// The income requirement examples: Joe $165,000, Fred $50,000, Alex
/// $186,000, each met.
#[test]
fn the_ato_income_requirement_examples() {
    let joe = run(&with(
        base(),
        json!({"taxable_income": "155000.00", "business_losses": "0.00",
            "total_net_investment_loss": "10000.00"}),
    ));
    assert_eq!(amount(&joe, "income_requirement_total"), "165000.00");
    assert_eq!(joe["income_requirement"]["met"], true);
    let fred = run(&with(
        base(),
        json!({"taxable_income": "49000.00", "business_losses": "0.00",
            "reportable_super_contributions": "1000.00"}),
    ));
    assert_eq!(amount(&fred, "income_requirement_total"), "50000.00");
    let alex = run(&with(
        base(),
        json!({"taxable_income": "200000.00", "business_losses": "6000.00",
            "assessable_fhss_released_amount": "20000.00"}),
    ));
    assert_eq!(
        amount(&alex, "income_requirement_taxable_income"),
        "186000.00"
    );
    assert_eq!(alex["income_requirement"]["met"], true);
}

/// A loss bigger than the other income gives nil, not a negative, before
/// the other amounts are added (the ATO's 'use zero').
#[test]
fn a_negative_taxable_income_counts_as_nil() {
    let result = run(&with(
        base(),
        json!({"taxable_income": "-30000.00", "business_losses": "10000.00",
            "reportable_fringe_benefits": "5000.00"}),
    ));
    assert_eq!(amount(&result, "income_requirement_taxable_income"), "0.00");
    assert_eq!(amount(&result, "income_requirement_total"), "5000.00");
}

/// Jennifer (the s 35-10 example, moved into a supported year): $9,000 of
/// income, $10,000 of deductions and $2,000 brought forward, no test passed:
/// $3,000 is deferred.
#[test]
fn jennifer_defers_3000() {
    let result = run(&with(base(), json!({"brought_forward": "2000.00"})));
    assert_eq!(result["rule_applies"], true);
    assert_eq!(amount(&result, "excess"), "3000.00");
    assert_eq!(amount(&result, "deferred"), "3000.00");
    assert_eq!(amount(&result, "deductible_this_year"), "0.00");
    assert_eq!(result["commissioner_discretion"]["decided"], false);
    assert!(
        result["outcome"]
            .as_str()
            .unwrap()
            .contains("unless the Commissioner decides under s 35-55")
    );
}

/// Michael: $15,000 of income and $20,000 of deductions, $3,000 of net
/// exempt income: a $5,000 excess and $2,000 deferred (s 35-15).
#[test]
fn michael_defers_2000_after_net_exempt_income() {
    let result = run(&with(
        base(),
        json!({"assessable_income": "15000.00", "deductions": "20000.00",
            "net_exempt_income": "3000.00", "taxable_income": "55000.00",
            "business_losses": "5000.00"}),
    ));
    assert_eq!(amount(&result, "excess"), "5000.00");
    assert_eq!(amount(&result, "deferred"), "2000.00");
    // Net exempt income above the excess leaves nil, not a negative.
    let more = run(&with(base(), json!({"net_exempt_income": "5000.00"})));
    assert_eq!(amount(&more, "deferred"), "0.00");
}

#[test]
fn each_threshold_is_met_at_its_amount() {
    let cases = [
        (
            "assessable_income_test",
            "assessable_income",
            "19999.99",
            "20000.00",
        ),
        (
            "real_property_test",
            "real_property_value",
            "499999.99",
            "500000.00",
        ),
        (
            "other_assets_test",
            "other_assets_value",
            "99999.99",
            "100000.00",
        ),
    ];
    for (test, key, under, at) in cases {
        let below = run(&with(base(), json!({key: under, "deductions": "30000.00"})));
        assert!(!met(&below, test), "{test}");
        assert_eq!(below["rule_applies"], true, "{test}");
        let reached = run(&with(base(), json!({key: at, "deductions": "30000.00"})));
        assert!(met(&reached, test), "{test}");
        assert_eq!(reached["rule_applies"], false, "{test}");
    }
}

/// The income requirement is under $250,000; a test passed does not help
/// without it.
#[test]
fn the_income_requirement_is_under_250000() {
    let passing = with(base(), json!({"other_assets_value": "100000.00"}));
    let under = run(&with(
        passing.clone(),
        json!({"taxable_income": "248999.99", "business_losses": "1000.00"}),
    ));
    assert_eq!(amount(&under, "income_requirement_total"), "249999.99");
    assert_eq!(under["income_requirement"]["met"], true);
    assert_eq!(under["rule_applies"], false);
    let at = run(&with(
        passing,
        json!({"taxable_income": "249000.00", "business_losses": "1000.00"}),
    ));
    assert_eq!(at["income_requirement"]["met"], false);
    assert_eq!(at["rule_applies"], true);
    assert_eq!(amount(&at, "deferred"), "1000.00");
}

/// A primary production or professional arts business with other income
/// under $40,000 is excepted, even above the income requirement.
#[test]
fn the_excepted_activities() {
    for activity in ["primary_production", "professional_arts"] {
        let under = run(&with(
            base(),
            json!({"activity": activity, "other_income": "39999.99",
                "taxable_income": "300000.00"}),
        ));
        assert_eq!(under["exception"]["applies"], true, "{activity}");
        assert_eq!(under["rule_applies"], false, "{activity}");
        let at = run(&with(
            base(),
            json!({"activity": activity, "other_income": "40000.00"}),
        ));
        assert_eq!(at["exception"]["applies"], false, "{activity}");
        assert_eq!(at["rule_applies"], true, "{activity}");
    }
}

#[test]
fn a_profitable_year_defers_nothing() {
    let result = run(&with(
        base(),
        json!({"assessable_income": "12000.00", "deductions": "10000.00"}),
    ));
    assert_eq!(amount(&result, "excess"), "0.00");
    assert_eq!(amount(&result, "deferred"), "0.00");
    assert!(result["outcome"].as_str().unwrap().starts_with("no excess"));
}

#[test]
fn every_supported_year_and_no_other() {
    for year in [
        "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
    ] {
        assert_eq!(run(&with(base(), json!({"year": year})))["period"], year);
    }
    for year in ["2019-20", "2027-28"] {
        assert_eq!(
            code(&with(base(), json!({"year": year}))),
            "period.unsupported"
        );
    }
}

#[test]
fn refusals() {
    assert_eq!(
        code(&with(base(), json!({"activity": "hobby"}))),
        "non_commercial_losses.activity_invalid"
    );
    for years in [
        json!([null, null, null]),
        json!([null, null, null, null, null]),
        json!({"a": 1}),
        json!([null, null, null, 5]),
    ] {
        assert_eq!(
            code(&with(base(), json!({"earlier_years": years}))),
            "non_commercial_losses.earlier_years_invalid"
        );
    }
    assert_eq!(
        code(&with(
            base(),
            json!({"earlier_years": [{"assessable_income": "1.00"}, null, null, null]})
        )),
        "input.missing_field"
    );
    assert_eq!(
        code(&with(base(), json!({"part_year": true}))),
        "input.missing_field"
    );
    assert_eq!(
        code(&with(base(), json!({"assessable_income_estimate": "1.00"}))),
        "input.field_unknown"
    );
    assert_eq!(
        code(&with(base(), json!({"activity": "primary_production"}))),
        "input.missing_field"
    );
    assert_eq!(
        code(&with(base(), json!({"other_income": "1.00"}))),
        "input.field_unknown"
    );
    assert_eq!(
        code(&with(base(), json!({"deductions": "-1.00"}))),
        "input.amount_invalid"
    );
    assert_eq!(
        code(&with(base(), json!({"scope_confirmed": false}))),
        "scope.not_confirmed"
    );
    assert_eq!(
        code(&with(base(), json!({"not_an_input": 1}))),
        "input.field_unknown"
    );
}

/// s 35-30(a) or (b): for a part year, the actual assessable income or the
/// full-year estimate reaching $20,000 passes.
#[test]
fn a_part_year_passes_on_actual_income_or_the_estimate() {
    let part = |income: &str, estimate: &str| {
        run(&with(
            base(),
            json!({"assessable_income": income, "deductions": "21000.00", "part_year": true,
                "assessable_income_estimate": estimate}),
        ))
    };
    let actual = part("20000.00", "19999.99");
    assert!(met(&actual, "assessable_income_test"));
    assert_eq!(amount(&actual, "assessable_income_measured"), "20000.00");
    assert_eq!(amount(&actual, "deferred"), "0.00");
    assert_eq!(amount(&actual, "deductible_this_year"), "1000.00");
    let neither = part("19999.99", "19999.99");
    assert!(!met(&neither, "assessable_income_test"));
    assert_eq!(amount(&neither, "deferred"), "1000.01");
}

/// s 35-15: the outcome says when net exempt income leaves nothing, or
/// less than the excess, deferred.
#[test]
fn the_outcome_follows_the_amount_deferred() {
    let nil = run(&with(base(), json!({"net_exempt_income": "1000.00"})));
    assert_eq!(amount(&nil, "deferred"), "0.00");
    let text = nil["outcome"].as_str().unwrap();
    assert!(
        text.contains("reduces the amount deferred to nil"),
        "{text}"
    );
    let part = run(&with(base(), json!({"net_exempt_income": "400.00"})));
    assert_eq!(amount(&part, "deferred"), "600.00");
    let text = part["outcome"].as_str().unwrap();
    assert!(
        text.starts_with("the excess less net exempt income"),
        "{text}"
    );
}

/// s 35-15(2) reduces only an amount s 35-10(2)(b) applies to for the year:
/// when a test is passed, the brought-forward amount and the year's excess
/// are deductible in full.
#[test]
fn net_exempt_income_reduces_only_an_amount_deferred() {
    let passed = run(&with(
        base(),
        json!({"brought_forward": "2000.00", "net_exempt_income": "1000.00",
            "real_property_value": "500000.00"}),
    ));
    assert!(!passed["rule_applies"].as_bool().unwrap());
    assert_eq!(amount(&passed, "deductible_this_year"), "3000.00");
    assert_eq!(amount(&passed, "deferred"), "0.00");
}

/// s 35-10(2E): an FHSS amount over taxable income with the losses added
/// back leaves nil, not a negative amount.
#[test]
fn an_fhss_amount_over_taxable_income_leaves_nil() {
    let out = run(&with(
        base(),
        json!({"taxable_income": "5000.00", "business_losses": "1000.00",
            "assessable_fhss_released_amount": "8000.00", "reportable_fringe_benefits": "100.00"}),
    ));
    assert_eq!(amount(&out, "income_requirement_taxable_income"), "0.00");
    assert_eq!(amount(&out, "income_requirement_total"), "100.00");
}
