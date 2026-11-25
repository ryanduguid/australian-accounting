// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Commercial debt forgiveness (ITAA 1997 Division 245), read on
//! 26 November 2026. D1 builds on the example under s 245-50 (Samara accepts
//! $60 for Daniel's $100 debt); the rest are made-up figures worked by hand
//! (docs/calculation-evidence.md#debt-forgiveness).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "debt-forgiveness";

fn run(input: &Value) -> Value {
    api::evaluate(CMD, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn debt(value: &str, offset: &str) -> Value {
    json!({"label": "Loan", "commercial_debt": true, "excluded": false, "value": value, "amount_offset": offset})
}

fn item(label: &str, amount: &str) -> Value {
    json!({"label": label, "amount": amount})
}

fn input(debts: Value) -> Value {
    json!({"debts": debts, "tax_losses": [], "net_capital_losses": [], "expenditures": [],
           "cost_bases": [], "partnership": false, "attributable_income": false,
           "year": "2025-26", "scope_confirmed": true})
}

fn d1() -> Value {
    let mut i = input(json!([debt("100.00", "60.00")]));
    i["tax_losses"] = json!([item("2023-24 tax loss", "25.00")]);
    i["net_capital_losses"] = json!([item("2022-23 net capital loss", "10.00")]);
    i["expenditures"] = json!([{"label": "Plant", "item": 1, "amount": "3.00"}]);
    i["cost_bases"] = json!([item("Shares", "50.00")]);
    i
}

fn amount(r: &Value, key: &str) -> String {
    r["amounts"][key].as_str().unwrap().to_string()
}

fn column(r: &Value, list: &str, key: &str) -> Vec<String> {
    r[list]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x[key].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn d1_the_section_245_50_example_applied_in_order() {
    let r = run(&d1());
    assert_eq!(amount(&r, "gross_forgiven_amount"), "40.00");
    assert_eq!(amount(&r, "total_net_forgiven_amount"), "40.00");
    assert_eq!(amount(&r, "tax_losses_reduced"), "25.00");
    assert_eq!(amount(&r, "net_capital_losses_reduced"), "10.00");
    assert_eq!(amount(&r, "expenditure_reduced"), "3.00");
    assert_eq!(amount(&r, "cost_bases_reduced"), "2.00");
    assert_eq!(amount(&r, "unapplied_net_forgiven_amount"), "0.00");
    assert_eq!(column(&r, "cost_bases", "remaining"), ["48.00"]);
    assert_eq!(r["expenditures"][0]["item"], 1);
    assert_eq!(r["debts"][0]["gross_forgiven_amount"], "40.00");
    assert_eq!(r["debts"][0]["division_245_applies"], true);
}

#[test]
fn d2_joint_debtors_share_equally() {
    let mut d = debt("1000.00", "0.00");
    d["debtors"] = json!(3);
    let r = run(&input(json!([d])));
    assert_eq!(r["debts"][0]["gross_forgiven_amount"], "1000.00");
    assert_eq!(r["debts"][0]["gross_forgiven_amount_share"], "333.33");
    assert_eq!(amount(&r, "total_net_forgiven_amount"), "333.33");
    assert_eq!(amount(&r, "unapplied_net_forgiven_amount"), "333.33");
}

#[test]
fn d3_other_provisions_and_a_creditors_agreement_come_off() {
    let mut d = debt("10000.00", "2000.00");
    d["assessable_income"] = json!("1000.00");
    d["deductions_reduced"] = json!("300.00");
    d["cost_bases_reduced"] = json!("200.00");
    d["forgone_by_creditor"] = json!("1500.00");
    let r = run(&input(json!([d.clone()])));
    assert_eq!(amount(&r, "gross_forgiven_amount"), "8000.00");
    assert_eq!(amount(&r, "brought_to_account"), "1500.00");
    assert_eq!(amount(&r, "forgone_by_creditors"), "1500.00");
    assert_eq!(amount(&r, "total_net_forgiven_amount"), "5000.00");
    // The provisional amount is 6,500.00: a cent more cannot be forgone.
    d["forgone_by_creditor"] = json!("6500.00");
    assert_eq!(
        amount(
            &run(&input(json!([d.clone()]))),
            "total_net_forgiven_amount"
        ),
        "0.00"
    );
    d["forgone_by_creditor"] = json!("6500.01");
    assert_eq!(
        code(&input(json!([d]))),
        "debt_forgiveness.forgone_exceeds_provisional"
    );
}

#[test]
fn d4_debts_outside_the_division_and_offsets_at_or_above_value() {
    let mut excluded = debt("500.00", "0.00");
    excluded["excluded"] = json!(true);
    let mut private = debt("700.00", "0.00");
    private["commercial_debt"] = json!(false);
    let r = run(&input(json!([
        excluded,
        private,
        debt("100.00", "100.00"),
        debt("100.00", "100.01"),
        debt("250.00", "50.00")
    ])));
    assert_eq!(
        column(&r, "debts", "net_forgiven_amount"),
        ["0.00", "0.00", "0.00", "0.00", "200.00"]
    );
    assert_eq!(r["debts"][0]["division_245_applies"], false);
    assert_eq!(amount(&r, "total_net_forgiven_amount"), "200.00");
}

#[test]
fn d5_reductions_beyond_the_share_leave_nil() {
    let mut d = debt("100.00", "0.00");
    d["assessable_income"] = json!("150.00");
    let r = run(&input(json!([d, debt("40.00", "0.00")])));
    assert_eq!(
        column(&r, "debts", "net_forgiven_amount"),
        ["0.00", "40.00"]
    );
    assert_eq!(amount(&r, "total_net_forgiven_amount"), "40.00");
}

#[test]
fn d6_order_within_and_between_classes() {
    let mut i = input(json!([debt("1000.00", "0.00")]));
    i["tax_losses"] = json!([item("2021-22", "300.00"), item("2022-23", "200.00")]);
    i["net_capital_losses"] = json!([item("2020-21", "100.00")]);
    i["expenditures"] = json!([{"label": "Borrowing costs", "item": 2, "amount": "150.00"}]);
    i["cost_bases"] = json!([
        {"label": "Shares in an associate", "amount": "500.00", "associate": true},
        {"label": "Land", "amount": "100.00", "associate": false}
    ]);
    let r = run(&i);
    assert_eq!(amount(&r, "tax_losses_reduced"), "500.00");
    assert_eq!(column(&r, "tax_losses", "reduced_by"), ["300.00", "200.00"]);
    assert_eq!(amount(&r, "net_capital_losses_reduced"), "100.00");
    assert_eq!(amount(&r, "expenditure_reduced"), "150.00");
    assert_eq!(amount(&r, "cost_bases_reduced"), "250.00");
    // Section 245-185: the associate's shares are reduced last.
    assert_eq!(
        column(&r, "cost_bases", "label"),
        ["Land", "Shares in an associate"]
    );
    assert_eq!(column(&r, "cost_bases", "reduced_by"), ["100.00", "150.00"]);
    assert_eq!(r["cost_bases"][1]["associate"], true);
    assert_eq!(amount(&r, "unapplied_net_forgiven_amount"), "0.00");
    // A smaller amount stops part way through the first class.
    i["debts"] = json!([debt("350.00", "0.00")]);
    let r = run(&i);
    assert_eq!(column(&r, "tax_losses", "reduced_by"), ["300.00", "50.00"]);
    assert_eq!(column(&r, "tax_losses", "remaining"), ["0.00", "150.00"]);
    assert_eq!(amount(&r, "net_capital_losses_reduced"), "0.00");
    assert_eq!(column(&r, "cost_bases", "reduced_by"), ["0.00", "0.00"]);
}

#[test]
fn d7_what_cannot_be_applied_is_disregarded() {
    let r = run(&input(json!([debt("1000.00", "0.00")])));
    assert_eq!(amount(&r, "tax_losses_reduced"), "0.00");
    assert_eq!(amount(&r, "unapplied_net_forgiven_amount"), "1000.00");
}

#[test]
fn refusals() {
    let mut i = d1();
    i["partnership"] = json!(true);
    assert_eq!(code(&i), "debt_forgiveness.partnership");
    let mut i = d1();
    i["attributable_income"] = json!(true);
    assert_eq!(code(&i), "debt_forgiveness.attributable_income");
    let mut no_value = debt("1.00", "0.00");
    no_value.as_object_mut().unwrap().remove("value");
    let mut zero_debtors = debt("1.00", "0.00");
    zero_debtors["debtors"] = json!(0);
    let mut extra = debt("1.00", "0.00");
    extra["creditor"] = json!("Samara");
    let mut flagless = debt("1.00", "0.00");
    flagless.as_object_mut().unwrap().remove("excluded");
    for bad in [
        json!([]),
        json!("Loan"),
        json!([no_value]),
        json!([zero_debtors]),
        json!([extra]),
        json!([flagless]),
        json!([debt("1.001", "0.00")]),
        json!(["Loan"]),
        json!(vec![debt("1.00", "0.00"); 201]),
    ] {
        let i = input(bad.clone());
        assert_eq!(code(&i), "debt_forgiveness.debt_invalid", "{bad}");
    }
    for (list, bad) in [
        ("tax_losses", json!([{"label": "A"}])),
        (
            "tax_losses",
            json!([{"label": "A", "amount": "1.00", "item": 1}]),
        ),
        ("net_capital_losses", json!("A")),
        (
            "expenditures",
            json!([{"label": "A", "amount": "1.00", "item": 9}]),
        ),
        ("expenditures", json!([{"label": "A", "amount": "1.00"}])),
        (
            "cost_bases",
            json!([{"label": "A", "amount": "1.00", "associate": "yes"}]),
        ),
        ("cost_bases", json!([{"label": "", "amount": "1.00"}])),
        ("cost_bases", json!(vec![item("A", "1.00"); 201])),
    ] {
        let mut i = d1();
        i[list] = bad.clone();
        assert_eq!(code(&i), "debt_forgiveness.item_invalid", "{list} {bad}");
    }
}

#[test]
fn years_and_scope() {
    for year in ["2020-21", "2026-27"] {
        let mut i = d1();
        i["year"] = json!(year);
        assert!(api::evaluate(CMD, &i).is_ok(), "{year}");
    }
    for year in ["2019-20", "2027-28"] {
        let mut i = d1();
        i["year"] = json!(year);
        assert_eq!(code(&i), "period.unsupported", "{year}");
    }
    let mut i = d1();
    i["scope_confirmed"] = json!(false);
    assert_eq!(code(&i), "scope.not_confirmed");
    let mut i = d1();
    i["not_an_input"] = json!(1);
    assert_eq!(code(&i), "input.field_unknown");
    let mut i = d1();
    i["partnership"] = json!("no");
    assert_eq!(code(&i), "input.flag_invalid");
}
