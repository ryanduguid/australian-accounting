// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A company's accounting profit reconciled to taxable income (ITAA 1997
//! ss 4-15, 36-10, 36-17 and 36-20; AASB 112 paras 5 and 17), read on
//! 26 November 2026. B1 is the deferred-tax example's year; B2 to B7 are
//! made-up figures worked by hand (docs/calculation-evidence.md#book-to-tax).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "book-to-tax";

fn run(input: &Value) -> Value {
    let result = api::evaluate(CMD, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": CMD, "input": input})).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn line(difference: &str, effect: &str, amount: &str) -> Value {
    json!({"label": format!("{difference} {effect}"), "difference": difference,
           "effect": effect, "amount": amount})
}

fn input(profit: &str, lines: Value, exempt: &str, opening: &str, chosen: &str) -> Value {
    json!({"accounting_profit": profit, "adjustments": lines, "net_exempt_income": exempt,
           "opening_tax_losses": opening, "tax_losses_deducted": chosen,
           "loss_carry_back": false, "excess_franking_offsets": false,
           "subdivision_165_b": false, "tax_consolidated_group": false,
           "year": "2025-26", "scope_confirmed": true})
}

fn amount<'a>(r: &'a Value, key: &str) -> &'a str {
    r["amounts"][key].as_str().unwrap()
}

fn b1() -> Value {
    input(
        "100000.00",
        json!([
            line("permanent", "add", "4000.00"),
            line("temporary", "subtract", "5000.00"),
            line("temporary", "add", "2000.00"),
            line("temporary", "subtract", "1000.00")
        ]),
        "0.00",
        "0.00",
        "0.00",
    )
}

#[test]
fn b1_the_deferred_tax_example_reconciles_to_its_taxable_income() {
    let r = run(&b1());
    assert_eq!(amount(&r, "permanent_additions"), "4000.00");
    assert_eq!(amount(&r, "temporary_additions"), "2000.00");
    assert_eq!(amount(&r, "temporary_subtractions"), "6000.00");
    assert_eq!(amount(&r, "net_income_before_tax_losses"), "100000.00");
    assert_eq!(amount(&r, "taxable_income"), "100000.00");
    assert_eq!(amount(&r, "tax_loss_for_year"), "0.00");
    assert_eq!(
        amount(&r, "net_taxable_temporary_difference_change"),
        "4000.00"
    );
    assert_eq!(
        r["deferred_tax"],
        json!({"accounting_profit": "100000.00", "non_deductible_expenses": "4000.00",
               "non_assessable_income": "0.00", "taxable_income": "100000.00",
               "opening_unused_tax_losses": "0.00", "unused_tax_losses": "0.00"})
    );
    assert_eq!(r["adjustments"].as_array().unwrap().len(), 4);
}

/// The link: deferred-tax given these inputs and its own example's items
/// finds nothing unexplained.
#[test]
fn b1_feeds_deferred_tax_with_no_unexplained_difference() {
    let r = run(&b1());
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["deferred_tax"]["example"]["facts"].clone();
    let facts = facts.as_object_mut().unwrap();
    facts.remove("kind");
    for (k, v) in r["deferred_tax"].as_object().unwrap() {
        facts.insert(k.clone(), v.clone());
    }
    facts.insert("scope_confirmed".into(), json!(true));
    let dt = api::evaluate("deferred-tax", &Value::Object(facts.clone())).unwrap();
    assert_eq!(dt["amounts"]["taxable_income_difference"], "0.00");
    assert_eq!(dt["amounts"]["temporary_difference_movement"], "4000.00");
}

#[test]
fn b2_a_loss_year_carries_the_loss() {
    let r = run(&input(
        "-50000.00",
        json!([line("permanent", "add", "2000.00")]),
        "0.00",
        "1500.00",
        "0.00",
    ));
    assert_eq!(amount(&r, "net_income_before_tax_losses"), "-48000.00");
    assert_eq!(amount(&r, "excess_deductions"), "48000.00");
    assert_eq!(amount(&r, "taxable_income"), "0.00");
    assert_eq!(amount(&r, "tax_loss_for_year"), "48000.00");
    assert_eq!(amount(&r, "closing_tax_losses"), "49500.00");
    assert_eq!(r["deferred_tax"]["accounting_profit"], "-50000.00");
}

/// s 36-10(3): net exempt income reduces the year's tax loss; s 36-17(4):
/// the excess uses it up first, so no earlier loss is absorbed.
#[test]
fn b3_net_exempt_income_reduces_a_loss() {
    let r = run(&input("-10000.00", json!([]), "4000.00", "3000.00", "0.00"));
    assert_eq!(amount(&r, "net_exempt_income_left"), "0.00");
    assert_eq!(amount(&r, "tax_losses_against_exempt_income"), "0.00");
    assert_eq!(amount(&r, "tax_loss_for_year"), "6000.00");
    assert_eq!(amount(&r, "closing_tax_losses"), "9000.00");
}

/// s 36-17(4)(b): what is left of net exempt income after the excess
/// absorbs earlier losses even in a year with no taxable income.
#[test]
fn b4_net_exempt_income_left_absorbs_earlier_losses_in_a_loss_year() {
    let r = run(&input("-2000.00", json!([]), "5000.00", "10000.00", "0.00"));
    assert_eq!(amount(&r, "net_exempt_income_left"), "3000.00");
    assert_eq!(amount(&r, "tax_losses_against_exempt_income"), "3000.00");
    assert_eq!(amount(&r, "tax_loss_for_year"), "0.00");
    assert_eq!(amount(&r, "taxable_income"), "0.00");
    assert_eq!(amount(&r, "closing_tax_losses"), "7000.00");
}

/// Neither s 36-17(3) nor (4) applies when assessable income equals
/// deductions: net exempt income absorbs nothing.
#[test]
fn b8_nil_net_income_absorbs_no_losses() {
    let r = run(&input("0.00", json!([]), "5000.00", "10000.00", "0.00"));
    assert_eq!(amount(&r, "net_exempt_income_left"), "0.00");
    assert_eq!(amount(&r, "tax_losses_against_exempt_income"), "0.00");
    assert_eq!(amount(&r, "closing_tax_losses"), "10000.00");
}

/// s 36-17(3): net exempt income first, then the chosen amount, up to the
/// losses left.
#[test]
fn b5_losses_go_against_net_exempt_income_first_then_as_chosen() {
    let base = |chosen: &str| {
        input(
            "18000.00",
            json!([
                line("permanent", "subtract", "5000.00"),
                line("permanent", "add", "7000.00")
            ]),
            "5000.00",
            "8000.00",
            chosen,
        )
    };
    let r = run(&base("3000.00"));
    assert_eq!(amount(&r, "net_income_before_tax_losses"), "20000.00");
    assert_eq!(amount(&r, "tax_losses_against_exempt_income"), "5000.00");
    assert_eq!(amount(&r, "taxable_income"), "17000.00");
    assert_eq!(amount(&r, "closing_tax_losses"), "0.00");
    assert_eq!(r["deferred_tax"]["non_assessable_income"], "5000.00");
    assert_eq!(
        code(&base("3000.01")),
        "book_to_tax.losses_deducted_invalid"
    );
}

/// s 36-17(2): the company chooses any amount up to the excess, nil included.
#[test]
fn b6_the_choice_is_capped_at_the_excess() {
    let base = |chosen: &str| input("4000.00", json!([]), "0.00", "9000.00", chosen);
    let r = run(&base("0.00"));
    assert_eq!(amount(&r, "taxable_income"), "4000.00");
    assert_eq!(amount(&r, "closing_tax_losses"), "9000.00");
    let r = run(&base("4000.00"));
    assert_eq!(amount(&r, "taxable_income"), "0.00");
    assert_eq!(amount(&r, "closing_tax_losses"), "5000.00");
    assert_eq!(
        code(&base("4000.01")),
        "book_to_tax.losses_deducted_invalid"
    );
    let loss = input("-1.00", json!([]), "0.00", "9000.00", "0.01");
    assert_eq!(code(&loss), "book_to_tax.losses_deducted_invalid");
}

/// A nil reconciliation: profit equals taxable income.
#[test]
fn b7_no_lines_and_a_nil_profit() {
    let r = run(&input("0.00", json!([]), "0.00", "0.00", "0.00"));
    assert_eq!(amount(&r, "taxable_income"), "0.00");
    assert_eq!(amount(&r, "excess_deductions"), "0.00");
    assert_eq!(amount(&r, "permanent_subtractions"), "0.00");
}

#[test]
fn each_stated_exclusion_is_refused() {
    for (flag, expected) in [
        ("loss_carry_back", "book_to_tax.loss_carry_back"),
        (
            "excess_franking_offsets",
            "book_to_tax.excess_franking_offsets",
        ),
        ("subdivision_165_b", "book_to_tax.subdivision_165_b"),
        (
            "tax_consolidated_group",
            "book_to_tax.tax_consolidated_group",
        ),
    ] {
        let mut i = b1();
        i[flag] = json!(true);
        assert_eq!(code(&i), expected, "{flag}");
        i[flag] = json!("yes");
        assert_eq!(code(&i), "input.flag_invalid", "{flag}");
    }
}

#[test]
fn malformed_lines_are_refused() {
    let bad = [
        json!("lines"),
        json!([{"label": "x", "difference": "timing", "effect": "add", "amount": "1.00"}]),
        json!([{"label": "x", "difference": "permanent", "effect": "plus", "amount": "1.00"}]),
        json!([{"label": "", "difference": "permanent", "effect": "add", "amount": "1.00"}]),
        json!([{"label": "x", "difference": "permanent", "effect": "add", "amount": "-1.00"}]),
        json!([{"label": "x", "difference": "permanent", "effect": "add", "amount": "1.001"}]),
        json!([{"label": "x", "difference": "permanent", "effect": "add"}]),
        json!([{"label": "x", "difference": "permanent", "effect": "add", "amount": "1.00", "note": "y"}]),
        json!(["x"]),
        Value::Array(vec![line("permanent", "add", "1.00"); 501]),
    ];
    for lines in bad {
        let mut i = b1();
        i["adjustments"] = lines.clone();
        assert_eq!(code(&i), "book_to_tax.adjustment_invalid", "{lines}");
    }
    let mut i = b1();
    i["adjustments"] = Value::Array(vec![line("permanent", "add", "1.00"); 500]);
    assert_eq!(amount(&run(&i), "permanent_additions"), "500.00");
}

#[test]
fn amounts_scope_and_years_are_checked() {
    let mut i = b1();
    i["net_exempt_income"] = json!("-1.00");
    assert_eq!(code(&i), "input.amount_invalid");
    let mut i = b1();
    i["accounting_profit"] = json!("1.001");
    assert_eq!(code(&i), "input.signed_amount_invalid");
    let mut i = b1();
    i["scope_confirmed"] = json!(false);
    assert_eq!(code(&i), "scope.not_confirmed");
    for (year, ok) in [
        ("2019-20", false),
        ("2020-21", true),
        ("2026-27", true),
        ("2027-28", false),
    ] {
        let mut i = b1();
        i["year"] = json!(year);
        assert_eq!(api::evaluate_coded(CMD, &i).is_ok(), ok, "{year}");
    }
    let mut i = b1();
    i["extra"] = json!(1);
    assert_eq!(code(&i), "input.field_unknown");
}
