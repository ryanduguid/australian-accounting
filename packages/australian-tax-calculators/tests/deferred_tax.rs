// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The deferred tax worksheet (AASB 112, with Income Tax Rates Act 1986
//! s 23(2)), read on 22 November 2026, against vectors made up for these
//! tests that apply the paragraphs (the standard's own examples are not
//! reproduced), each year's rates, each refusal and the reconciliation
//! (docs/calculation-evidence.md#deferred-tax).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "deferred-tax";
const YEARS: [&str; 7] = [
    "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
];

fn with(base: Value, overrides: Value) -> Value {
    let mut input = base;
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

/// The plain result, checked against `derive` byte for byte.
fn run(input: &Value) -> Value {
    let result = api::evaluate(CMD, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": CMD, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

fn item(label: &str, class: &str, now: [&str; 2], before: [&str; 2]) -> Value {
    json!({"label": label, "class": class, "carrying_amount": now[0], "tax_base": now[1],
           "opening_carrying_amount": before[0], "opening_tax_base": before[1]})
}

/// V1: plant, a leave provision and a prepayment of a company that is not a
/// base rate entity, 2025-26.
fn v1(overrides: Value) -> Value {
    with(
        json!({
            "items": [
                item("Plant", "asset", ["70000.00", "50000.00"], ["80000.00", "65000.00"]),
                item("Leave provision", "liability", ["12000.00", "0.00"], ["10000.00", "0.00"]),
                item("Prepayment", "asset", ["3000.00", "0.00"], ["2000.00", "0.00"]),
            ],
            "unused_tax_losses": "0.00", "opening_unused_tax_losses": "0.00",
            "unrecognised_deductible": "0.00", "opening_unrecognised_deductible": "0.00",
            "accounting_profit": "100000.00", "non_deductible_expenses": "4000.00",
            "non_assessable_income": "0.00", "taxable_income": "100000.00",
            "tax_offsets": "0.00", "base_rate_entity": false,
            "expected_rate": "standard", "opening_expected_rate": "standard",
            "business_combination": false, "uncertain_tax_treatment": false,
            "tax_consolidated_group": false, "outside_profit_or_loss": false,
            "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

fn check(result: &Value, expected: &[(&str, &str)]) {
    for (key, want) in expected {
        assert_eq!(amount(result, key), *want, "{key}");
    }
}

/// V1: paras 15 and 24 at 30% throughout; the reconciliation's only
/// reconciling line is the non-deductible expenses.
#[test]
fn a_standard_company_with_no_rate_change() {
    let r = run(&v1(json!({})));
    check(
        &r,
        &[
            ("taxable_temporary_differences", "23000.00"),
            ("deductible_temporary_differences", "12000.00"),
            ("opening_taxable_temporary_differences", "17000.00"),
            ("opening_deductible_temporary_differences", "10000.00"),
            ("deferred_tax_liability", "6900.00"),
            ("deferred_tax_asset", "3600.00"),
            ("net_deferred_tax_liability", "3300.00"),
            ("opening_deferred_tax_liability", "5100.00"),
            ("opening_deferred_tax_asset", "3000.00"),
            ("deferred_tax_expense", "1200.00"),
            ("rate_change_effect", "0.00"),
            ("recognition_change_effect", "0.00"),
            ("origination_and_reversal", "1200.00"),
            ("temporary_difference_movement", "4000.00"),
            ("current_tax_before_offsets", "30000.00"),
            ("current_tax", "30000.00"),
            ("income_tax_expense", "31200.00"),
            ("tax_at_applicable_rate", "30000.00"),
            ("non_deductible_expenses_effect", "1200.00"),
            ("non_assessable_income_effect", "0.00"),
            ("deferred_rate_difference", "0.00"),
            ("tax_offsets_effect", "0.00"),
            ("reconciliation_difference", "0.00"),
            ("taxable_income_difference", "0.00"),
        ],
    );
    assert_eq!(r["rates"]["applicable_rate"], "0.30");
    assert_eq!(r["rates"]["effective_tax_rate"], "0.3120");
    let lines = r["items"].as_array().unwrap();
    assert_eq!(lines[1]["temporary_difference"], "-12000.00");
    assert_eq!(lines[1]["nature"], "deductible");
    assert_eq!(lines[0]["opening_temporary_difference"], "15000.00");
    assert_eq!(lines[2]["nature"], "taxable");
    assert_eq!(r["warnings"].as_array().unwrap().len(), 3);
}

/// V2: a base rate entity whose balances move from 30% to 25% (paras 60(a)
/// and 80(d)) and whose losses, unrecognised at the start, are recognised
/// and partly used (paras 34, 37 and 80(f)).
#[test]
fn a_rate_change_and_newly_recognised_losses() {
    let r = run(&v1(json!({
        "items": [item("Equipment", "asset", ["40000.00", "30000.00"], ["50000.00", "45000.00"])],
        "unused_tax_losses": "5000.00", "opening_unused_tax_losses": "20000.00",
        "opening_unrecognised_deductible": "20000.00",
        "accounting_profit": "30000.00", "non_deductible_expenses": "0.00",
        "non_assessable_income": "2000.00", "taxable_income": "8000.00",
        "base_rate_entity": true, "expected_rate": "base_rate_entity", "year": "2024-25",
    })));
    check(
        &r,
        &[
            ("deferred_tax_liability", "2500.00"),
            ("deferred_tax_asset", "1250.00"),
            ("net_deferred_tax_liability", "1250.00"),
            ("opening_deferred_tax_liability", "1500.00"),
            ("opening_deferred_tax_asset", "0.00"),
            ("deferred_tax_expense", "-250.00"),
            ("rate_change_effect", "-250.00"),
            ("recognition_change_effect", "-5000.00"),
            ("origination_and_reversal", "5000.00"),
            ("temporary_difference_movement", "20000.00"),
            ("current_tax", "2000.00"),
            ("income_tax_expense", "1750.00"),
            ("tax_at_applicable_rate", "7500.00"),
            ("non_assessable_income_effect", "-500.00"),
            ("deferred_rate_difference", "0.00"),
            ("reconciliation_difference", "0.00"),
            ("taxable_income_difference", "0.00"),
        ],
    );
    assert_eq!(r["rates"]["effective_tax_rate"], "0.0583");
    assert_eq!(r["rates"]["opening_rate"], "0.30");
}

/// V3: 2020-21 current tax at 26% for a base rate entity, while the year's
/// new difference is measured at the 25% that applies when it reverses.
#[test]
fn twenty_six_percent_current_and_twenty_five_percent_deferred_in_2020_21() {
    let r = run(&v1(json!({
        "items": [item("Software", "asset", ["10000.00", "0.00"], ["0.00", "0.00"])],
        "accounting_profit": "50000.00", "non_deductible_expenses": "0.00",
        "taxable_income": "40000.00", "base_rate_entity": true,
        "expected_rate": "base_rate_entity", "year": "2020-21",
    })));
    check(
        &r,
        &[
            ("deferred_tax_liability", "2500.00"),
            ("deferred_tax_expense", "2500.00"),
            ("current_tax", "10400.00"),
            ("income_tax_expense", "12900.00"),
            ("tax_at_applicable_rate", "13000.00"),
            ("deferred_rate_difference", "-100.00"),
            ("reconciliation_difference", "0.00"),
        ],
    );
    assert_eq!(r["rates"]["applicable_rate"], "0.26");
    assert_eq!(r["rates"]["closing_rate"], "0.25");
}

/// V4: a taxable income the inputs do not explain shows the gap and warns.
#[test]
fn an_unexplained_taxable_income_is_shown_and_warned() {
    let r = run(&v1(json!({"taxable_income": "99000.00"})));
    check(
        &r,
        &[
            ("taxable_income_difference", "-1000.00"),
            ("reconciliation_difference", "-300.00"),
            ("income_tax_expense", "30900.00"),
        ],
    );
    let warnings = r["warnings"].as_array().unwrap();
    assert!(
        warnings
            .last()
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("taxable_income does not equal")
    );
}

/// V5: non-refundable offsets reduce current tax and are a reconciling line;
/// offsets above the tax are refused.
#[test]
fn tax_offsets_reduce_current_tax() {
    let r = run(&v1(json!({"tax_offsets": "500.00"})));
    check(
        &r,
        &[
            ("current_tax", "29500.00"),
            ("income_tax_expense", "30700.00"),
            ("tax_offsets_effect", "-500.00"),
            ("reconciliation_difference", "0.00"),
        ],
    );
    assert_eq!(
        run(&v1(json!({"tax_offsets": "30000.00"})))["amounts"]["current_tax"],
        "0.00"
    );
    assert_eq!(
        code(&v1(json!({"tax_offsets": "30000.01"}))),
        "deferred_tax.offsets_above_tax"
    );
}

/// V6: a liability whose tax base is above its carrying amount is taxable
/// (paras 5 and 8), and an asset whose tax base is above it is deductible.
#[test]
fn the_sign_of_each_class() {
    let r = run(&v1(json!({
        "items": [
            item("Liability", "liability", ["1000.00", "1600.00"], ["0.00", "0.00"]),
            item("Asset", "asset", ["500.00", "800.00"], ["0.00", "0.00"]),
            item("Equal", "asset", ["900.00", "900.00"], ["0.00", "0.00"]),
        ],
        "non_deductible_expenses": "0.00", "taxable_income": "99700.00",
    })));
    check(
        &r,
        &[
            ("taxable_temporary_differences", "600.00"),
            ("deductible_temporary_differences", "300.00"),
            ("taxable_income_difference", "0.00"),
        ],
    );
    let lines = r["items"].as_array().unwrap();
    assert_eq!(lines[0]["nature"], "taxable");
    assert_eq!(lines[1]["temporary_difference"], "-300.00");
    assert_eq!(lines[2]["nature"], "none");
    assert_eq!(lines[2]["temporary_difference"], "0.00");
}

/// A loss before tax: the profit may be negative, and the effective rate is
/// omitted when the profit is nil.
#[test]
fn a_loss_before_tax_and_a_nil_profit() {
    let r = run(&v1(json!({
        "items": [item("Plant", "asset", ["1000.00", "1000.00"], ["1000.00", "1000.00"])],
        "accounting_profit": "-10000.00", "non_deductible_expenses": "0.00",
        "taxable_income": "0.00", "unused_tax_losses": "10000.00",
    })));
    check(
        &r,
        &[
            ("deferred_tax_asset", "3000.00"),
            ("income_tax_expense", "-3000.00"),
            ("tax_at_applicable_rate", "-3000.00"),
            ("reconciliation_difference", "0.00"),
        ],
    );
    assert_eq!(r["rates"]["effective_tax_rate"], "0.3000");
    let nil = run(&v1(
        json!({"accounting_profit": "0.00", "non_deductible_expenses": "0.00",
                              "taxable_income": "0.00"}),
    ));
    assert!(nil["rates"].get("effective_tax_rate").is_none());
}

/// The rate table: each year's s 23(2) rate for each status, and both
/// deferred rates.
#[test]
fn each_years_rates() {
    for year in YEARS {
        for (bre, want) in [
            (true, if year == "2020-21" { "0.26" } else { "0.25" }),
            (false, "0.30"),
        ] {
            let r = run(&v1(json!({"year": year, "base_rate_entity": bre})));
            assert_eq!(r["period"], year);
            assert_eq!(r["rates"]["applicable_rate"], want, "{year} {bre}");
        }
        let r = run(&v1(
            json!({"year": year, "expected_rate": "base_rate_entity"}),
        ));
        assert_eq!(r["rates"]["closing_rate"], "0.25");
        assert_eq!(amount(&r, "deferred_tax_liability"), "5750.00");
    }
    for year in ["2019-20", "2027-28"] {
        assert_eq!(
            code(&v1(json!({ "year": year }))),
            "period.unsupported",
            "{year}"
        );
    }
}

/// Each refusal the map and the spec name.
#[test]
fn the_refusals() {
    for (flag, refused) in [
        ("business_combination", "deferred_tax.business_combination"),
        (
            "uncertain_tax_treatment",
            "deferred_tax.uncertain_tax_treatment",
        ),
        (
            "tax_consolidated_group",
            "deferred_tax.tax_consolidated_group",
        ),
        (
            "outside_profit_or_loss",
            "deferred_tax.outside_profit_or_loss",
        ),
    ] {
        assert_eq!(code(&v1(json!({ flag: true }))), refused);
        assert_eq!(code(&v1(json!({ flag: "no" }))), "input.flag_invalid");
    }
    let opening = v1(json!({"year": "2020-21", "opening_expected_rate": "base_rate_entity"}));
    assert_eq!(code(&opening), "deferred_tax.opening_rate_unscheduled");
    run(&v1(
        json!({"year": "2021-22", "opening_expected_rate": "base_rate_entity"}),
    ));
    for key in ["expected_rate", "opening_expected_rate"] {
        assert_eq!(
            code(&v1(json!({ key: "0.25" }))),
            "deferred_tax.expected_rate_invalid"
        );
    }
    let above = v1(json!({"unrecognised_deductible": "12000.01"}));
    assert_eq!(code(&above), "deferred_tax.unrecognised_above_deductible");
    run(&v1(json!({"unrecognised_deductible": "12000.00"})));
    let opening_above = v1(json!({"opening_unrecognised_deductible": "10000.01"}));
    assert_eq!(
        code(&opening_above),
        "deferred_tax.unrecognised_above_deductible"
    );
    assert_eq!(
        code(&v1(json!({"scope_confirmed": false}))),
        "scope.not_confirmed"
    );
    assert_eq!(code(&v1(json!({"cost": "1.00"}))), "input.field_unknown");
}

/// A full unrecognised amount leaves no deferred tax asset, and its change
/// is the recognition effect.
#[test]
fn an_unrecognised_deductible_amount() {
    let r = run(&v1(
        json!({"unrecognised_deductible": "12000.00", "taxable_income": "100000.00"}),
    ));
    check(
        &r,
        &[
            ("deferred_tax_asset", "0.00"),
            ("recognition_change_effect", "3600.00"),
            ("origination_and_reversal", "1200.00"),
            ("deferred_tax_expense", "4800.00"),
            ("reconciliation_difference", "0.00"),
        ],
    );
}

#[test]
fn items_and_amounts_are_checked() {
    let good = item("Plant", "asset", ["1.00", "1.00"], ["1.00", "1.00"]);
    let mut extra = good.clone();
    extra["note"] = json!("x");
    let mut class = good.clone();
    class["class"] = json!("equity");
    let mut label = good.clone();
    label["label"] = json!(" ");
    let mut negative = good.clone();
    negative["tax_base"] = json!("-1.00");
    let mut missing = good.clone();
    missing.as_object_mut().unwrap().remove("opening_tax_base");
    let many: Vec<Value> = (0..501).map(|_| good.clone()).collect();
    for items in [
        json!([]),
        json!(many),
        json!({}),
        json!([1]),
        json!([extra]),
        json!([class]),
        json!([label]),
        json!([negative]),
        json!([missing]),
    ] {
        assert_eq!(
            code(&v1(json!({ "items": items }))),
            "deferred_tax.item_invalid"
        );
    }
    for bad in [json!("-1.00"), json!("1.001"), json!(5)] {
        assert_eq!(
            code(&v1(json!({ "taxable_income": bad }))),
            "input.amount_invalid",
            "{bad}"
        );
    }
    assert_eq!(
        code(&v1(json!({"accounting_profit": "1.001"}))),
        "input.signed_amount_invalid"
    );
}

/// The catalogue example is V1.
#[test]
fn the_catalogue_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["deferred_tax"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = run(&facts);
    assert_eq!(r["amounts"]["income_tax_expense"], "31200.00");
    let years = catalogue["deferred_tax"]["supported_periods"]
        .as_array()
        .unwrap();
    assert_eq!(years.len(), 7);
}
