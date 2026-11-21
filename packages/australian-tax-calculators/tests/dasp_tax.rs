// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The tax on a departing Australia superannuation payment (Superannuation
//! (Departing Australia Superannuation Payments Tax) Act 2007 s 5, read on 19
//! November 2026) against the Library's Superannuation Benefits, 8-420, and
//! the edges of each rate (docs/calculation-evidence.md#dasp-tax).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn payment(tax_free: &str, taxed: &str, untaxed: &str, whm: bool) -> Value {
    json!({
        "tax_free_component": tax_free, "taxed_element": taxed,
        "untaxed_element": untaxed, "working_holiday_maker_contributions": whm,
        "unclaimed_money_rollover": false, "year": "2026-27", "scope_confirmed": true,
    })
}

fn run(input: &Value) -> Value {
    api::evaluate("dasp-tax", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("dasp-tax", input).unwrap_err().code
}

/// The tax on each component, the total and the net payment.
fn taxes(input: &Value) -> [String; 5] {
    let a = run(input)["amounts"].clone();
    [
        "tax_free_component_tax",
        "taxed_element_tax",
        "untaxed_element_tax",
        "dasp_tax",
        "net_payment",
    ]
    .map(|key| a[key].as_str().unwrap().to_string())
}

/// Superannuation Benefits, 8-420: a $42,000 benefit of a $25,000 tax free
/// component and a $17,000 element taxed in the fund bears nil on the first
/// and $5,950 (35% x $17,000) on the second. It is the catalogue's example.
#[test]
fn the_librarys_example_bears_5950() {
    let input = payment("25000.00", "17000.00", "0.00", false);
    let r = run(&input);
    assert_eq!(r["amounts"]["payment"], "42000.00");
    assert_eq!(
        taxes(&input),
        ["0.00", "5950.00", "0.00", "5950.00", "36050.00"]
    );
    assert_eq!(r["rate_basis"], "ordinary");
    assert_eq!(r["rates"]["taxed_element_rate"], "0.35");
    assert_eq!(r["rates"]["untaxed_element_rate"], "0.45");
    assert_eq!(r["period"], "2026-27");
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["dasp_tax"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_eq!(run(&facts)["amounts"]["dasp_tax"], "5950.00");
}

/// The same payment to a former working holiday maker bears 65% on the
/// element taxed in the fund (s 5(3)): $11,050.
#[test]
fn a_working_holiday_makers_payment_bears_65_percent() {
    let input = payment("25000.00", "17000.00", "0.00", true);
    assert_eq!(
        taxes(&input),
        ["0.00", "11050.00", "0.00", "11050.00", "30950.00"]
    );
    let r = run(&input);
    assert_eq!(r["rate_basis"], "working_holiday_maker");
    assert_eq!(r["rates"]["taxed_element_rate"], "0.65");
    assert_eq!(r["rates"]["untaxed_element_rate"], "0.65");
}

/// The element untaxed in the fund bears 45% (s 5(1)(c)), and 65% too for a
/// working holiday maker (s 5(3)); the tax free component bears nil either
/// way.
#[test]
fn the_untaxed_element_bears_45_or_65_percent() {
    let ordinary = payment("1000.00", "10000.00", "20000.00", false);
    assert_eq!(
        taxes(&ordinary),
        ["0.00", "3500.00", "9000.00", "12500.00", "18500.00"]
    );
    let whm = payment("1000.00", "10000.00", "20000.00", true);
    assert_eq!(
        taxes(&whm),
        ["0.00", "6500.00", "13000.00", "19500.00", "11500.00"]
    );
}

/// Each element's tax is rounded to the cent, 0.5 cents up, before the
/// total, so the tax and the net payment add back to the payment; a payment
/// under $200 is taxed like any other (s 301-225 does not reach it).
#[test]
fn each_elements_tax_is_rounded_to_the_cent() {
    let input = payment("0.00", "8400.37", "0.01", false);
    // 8,400.37 x 35% = 2,940.1295; 0.01 x 45% = 0.0045.
    assert_eq!(
        taxes(&input),
        ["0.00", "2940.13", "0.00", "2940.13", "5460.25"]
    );
    let input = payment("0.00", "0.10", "0.10", true);
    // 0.10 x 65% = 0.065 each.
    assert_eq!(taxes(&input), ["0.00", "0.07", "0.07", "0.14", "0.06"]);
    let small = payment("20.00", "150.00", "0.00", false);
    assert_eq!(taxes(&small)[3], "52.50");
    // 0.01 x 35% = 0.0035 and 0.01 x 45% = 0.0045 each round to nil, so the
    // tax is nil where rounding their sum, 0.008, once would give 0.01.
    let cent_each = payment("0.00", "0.01", "0.01", false);
    assert_eq!(taxes(&cent_each), ["0.00", "0.00", "0.00", "0.00", "0.02"]);
}

/// A roll-over paid under s 20H(2), (2AA) or (2A) of the Superannuation
/// (Unclaimed Money and Lost Members) Act 1999 with an element untaxed in the
/// fund is refused, as s 5(2) taxes any excess untaxed roll-over amount in it
/// at nil; without that element s 5(1) applies as usual.
#[test]
fn an_unclaimed_money_rollover_with_an_untaxed_element_is_refused() {
    let mut input = payment("0.00", "100.00", "50.00", false);
    input["unclaimed_money_rollover"] = json!(true);
    assert_eq!(code(&input), "dasp_tax.unclaimed_money_rollover");
    let mut input = payment("10.00", "100.00", "0.00", false);
    input["unclaimed_money_rollover"] = json!(true);
    assert_eq!(taxes(&input), ["0.00", "35.00", "0.00", "35.00", "75.00"]);
    input["unclaimed_money_rollover"] = json!("no");
    assert_eq!(code(&input), "input.flag_invalid");
    input
        .as_object_mut()
        .unwrap()
        .remove("unclaimed_money_rollover");
    assert_eq!(code(&input), "input.missing_field");
}

/// A payment wholly of tax free component bears no tax.
#[test]
fn a_tax_free_payment_bears_nil() {
    assert_eq!(
        taxes(&payment("3000.00", "0.00", "0.00", true)),
        ["0.00", "0.00", "0.00", "0.00", "3000.00"]
    );
}

/// A nil payment, an amount outside money, a flag that is not a boolean, a
/// year outside those read and unconfirmed scope are each refused.
#[test]
fn bad_inputs_are_refused() {
    assert_eq!(
        code(&payment("0.00", "0.00", "0.00", false)),
        "dasp_tax.payment_nil"
    );
    for key in ["tax_free_component", "taxed_element", "untaxed_element"] {
        let mut input = payment("1.00", "1.00", "1.00", false);
        input[key] = json!("-1");
        assert_eq!(code(&input), "input.amount_invalid", "{key}");
        input[key] = json!("1.001");
        assert_eq!(code(&input), "input.amount_invalid", "{key}");
    }
    let mut input = payment("1.00", "1.00", "1.00", false);
    input["working_holiday_maker_contributions"] = json!("yes");
    assert_eq!(code(&input), "input.flag_invalid");
    for year in ["2016-17", "2024-25", "2028-29"] {
        let mut input = payment("1.00", "1.00", "1.00", false);
        input["year"] = json!(year);
        assert_eq!(code(&input), "period.unsupported", "{year}");
    }
    let mut input = payment("1.00", "1.00", "1.00", false);
    input["scope_confirmed"] = json!(false);
    assert_eq!(code(&input), "scope.not_confirmed");
    let mut input = payment("1.00", "1.00", "1.00", false);
    input["excess_untaxed_rollover_amount"] = json!("1.00");
    assert_eq!(code(&input), "input.field_unknown");
}
