// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Consolidation eliminations (AASB 10 paras 22, B86 and B94; AASB 3 paras 19
//! and 32), read on 26 November 2026. AASB 10 has no numeric example: C1 and
//! C2 are made-up groups worked by hand and checked in Python's `decimal`
//! (docs/calculation-evidence.md#consolidation-eliminations).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "consolidation-eliminations";

fn run(input: &Value) -> Value {
    api::evaluate(CMD, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn l(account: &str, class: &str, amount: &str) -> Value {
    json!({"account": account, "class": class, "amount": amount})
}

fn at(entity: &str, account: &str) -> Value {
    json!({"entity": entity, "account": account})
}

fn parent() -> Value {
    json!({"name": "P Ltd", "lines": [
        l("Cash", "asset", "46000.00"), l("Investment in S Ltd", "asset", "80000.00"),
        l("Receivable from S Ltd", "asset", "10000.00"), l("Inventory", "asset", "30000.00"),
        l("Share capital", "equity", "100000.00"), l("Retained earnings", "equity", "50000.00"),
        l("Sales", "income", "160000.00"), l("Cost of sales", "expense", "120000.00"),
        l("Income tax expense", "expense", "24000.00")]})
}

fn subsidiary(ownership: &str, nci: &str) -> Value {
    json!({"name": "S Ltd", "ownership": ownership, "investment_account": "Investment in S Ltd",
        "consideration": "80000.00", "nci_at_acquisition": nci,
        "equity_at_acquisition": [{"account": "Share capital", "amount": "60000.00"},
                                  {"account": "Retained earnings", "amount": "20000.00"}],
        "lines": [
        l("Cash", "asset", "33000.00"), l("Inventory", "asset", "15000.00"),
        l("Plant", "asset", "80000.00"), l("Payable to P Ltd", "liability", "10000.00"),
        l("Share capital", "equity", "60000.00"), l("Retained earnings", "equity", "30000.00"),
        l("Sales", "income", "100000.00"), l("Cost of sales", "expense", "60000.00"),
        l("Income tax expense", "expense", "12000.00")]})
}

fn unrealised(profit: &str) -> Value {
    json!([{"seller": "S Ltd", "buyer": "P Ltd", "inventory_account": "Inventory",
            "cost_of_sales_account": "Cost of sales", "profit": profit,
            "income_tax_account": "Income tax expense", "deferred_tax_account": "Deferred tax asset"}])
}

fn c1() -> Value {
    json!({"parent": parent(), "subsidiaries": [subsidiary("0.8", "20000.00")],
        "intra_group_balances": [{"receivable": at("P Ltd", "Receivable from S Ltd"),
                                  "payable": at("S Ltd", "Payable to P Ltd"), "amount": "10000.00"}],
        "intra_group_transactions": [{"income": at("S Ltd", "Sales"),
                                      "expense": at("P Ltd", "Cost of sales"), "amount": "40000.00"}],
        "unrealised_profits": unrealised("5000.00"), "tax_rate": "0.3",
        "acquired_in_period": false, "fair_value_adjustments": false,
        "previously_held_interest": false, "ownership_changed": false,
        "intra_group_dividends": false, "year": "2025-26", "scope_confirmed": true})
}

fn with(mut v: Value, key: &str, value: Value) -> Value {
    v[key] = value;
    v
}

fn line<'a>(r: &'a Value, account: &str) -> &'a Value {
    r["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["account"] == account)
        .unwrap()
}

#[test]
fn c1_partly_owned_subsidiary_with_upstream_unrealised_profit() {
    let r = run(&c1());
    let a = &r["amounts"];
    assert_eq!(a["goodwill"], "20000.00");
    assert_eq!(a["consolidated_profit"], "40500.00");
    assert_eq!(
        a["profit_attributable_to_non_controlling_interests"],
        "4900.00"
    );
    assert_eq!(a["profit_attributable_to_owners"], "35600.00");
    assert_eq!(a["non_controlling_interests"], "26900.00");
    assert_eq!(a["equity_attributable_to_owners"], "193600.00");
    assert_eq!(a["total_assets"], "220500.00");
    assert_eq!(a["total_liabilities"], "0.00");
    for (row, consolidated) in [
        ("Cash", "79000.00"),
        ("Investment in S Ltd", "0.00"),
        ("Inventory", "40000.00"),
        ("Goodwill", "20000.00"),
        ("Deferred tax asset", "1500.00"),
        ("Share capital", "100000.00"),
        ("Retained earnings", "58000.00"),
        ("Non-controlling interests", "22000.00"),
        ("Sales", "220000.00"),
        ("Cost of sales", "145000.00"),
        ("Income tax expense", "34500.00"),
    ] {
        assert_eq!(line(&r, row)["consolidated"], consolidated);
    }
}

#[test]
fn c1_journals_balance_and_cite_their_paragraphs() {
    let r = run(&c1());
    let journals = r["journals"].as_array().unwrap();
    assert_eq!(journals.len(), 5);
    for j in journals {
        let total = |side: &str| -> f64 {
            j["lines"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| l[side].as_str().unwrap().parse::<f64>().unwrap())
                .sum()
        };
        assert!((total("debit") - total("credit")).abs() < 1e-9, "{j}");
    }
    assert_eq!(
        journals[0]["authority"],
        "AASB 10 para B86(b); AASB 3 paras 19 and 32"
    );
    assert_eq!(journals[1]["authority"], "AASB 10 paras 22 and B94");
    let nci = &journals[1]["lines"];
    assert_eq!(
        nci[0],
        json!({"account": "Retained earnings", "class": "equity", "debit": "2000.00", "credit": "0.00"})
    );
}

/// C2: wholly owned, no unrealised profit, goodwill nil.
#[test]
fn c2_wholly_owned_without_goodwill() {
    let mut v = c1();
    v["subsidiaries"] = json!([subsidiary("1", "0.00")]);
    v["unrealised_profits"] = json!([]);
    let r = run(&v);
    let a = &r["amounts"];
    assert_eq!(a["goodwill"], "0.00");
    assert_eq!(a["non_controlling_interests"], "0.00");
    assert_eq!(a["consolidated_profit"], "44000.00");
    assert_eq!(a["profit_attributable_to_owners"], "44000.00");
    assert_eq!(line(&r, "Retained earnings")["consolidated"], "60000.00");
    assert_eq!(r["journals"].as_array().unwrap().len(), 3);
}

#[test]
fn a_nil_tax_rate_posts_no_deferred_tax() {
    let r = run(&with(c1(), "tax_rate", json!("0")));
    assert!(
        r["lines"]
            .as_array()
            .unwrap()
            .iter()
            .all(|l| l["account"] != "Deferred tax asset")
    );
    assert_eq!(
        r["amounts"]["profit_attributable_to_non_controlling_interests"],
        "4600.00"
    );
}

#[test]
fn downstream_unrealised_profit_does_not_reach_the_non_controlling_interest() {
    let mut v = c1();
    v["unrealised_profits"] = json!([{"seller": "P Ltd", "buyer": "S Ltd", "inventory_account": "Inventory",
        "cost_of_sales_account": "Cost of sales", "profit": "5000.00",
        "income_tax_account": "Income tax expense", "deferred_tax_account": "Deferred tax asset"}]);
    let r = run(&v);
    assert_eq!(
        r["amounts"]["profit_attributable_to_non_controlling_interests"],
        "5600.00"
    );
    assert_eq!(r["amounts"]["consolidated_profit"], "40500.00");
}

fn check(cases: Vec<(Value, &str)>) {
    for (v, want) in cases {
        assert_eq!(code(&v), want, "{v}");
    }
}

/// C1 changed by `f`.
fn edited(f: impl FnOnce(&mut Value)) -> Value {
    let mut v = c1();
    f(&mut v);
    v
}

#[test]
fn refusals_of_stated_facts_outside_the_worksheet() {
    check(vec![
        (
            with(c1(), "acquired_in_period", json!(true)),
            "consolidation_eliminations.acquired_in_period",
        ),
        (
            with(c1(), "fair_value_adjustments", json!(true)),
            "consolidation_eliminations.fair_value_adjustments",
        ),
        (
            with(c1(), "previously_held_interest", json!(true)),
            "consolidation_eliminations.previously_held_interest",
        ),
        (
            with(c1(), "ownership_changed", json!(true)),
            "consolidation_eliminations.ownership_changed",
        ),
        (
            with(c1(), "intra_group_dividends", json!(true)),
            "consolidation_eliminations.intra_group_dividends",
        ),
        (
            with(c1(), "tax_rate", json!("1")),
            "consolidation_eliminations.tax_rate_invalid",
        ),
        (
            with(c1(), "subsidiaries", json!([])),
            "consolidation_eliminations.item_invalid",
        ),
        (with(c1(), "year", json!("2019-20")), "period.unsupported"),
        (with(c1(), "unknown", json!(1)), "input.field_unknown"),
    ]);
}

#[test]
fn refusals_of_trial_balances_and_acquisitions() {
    check(vec![
        (
            edited(|v| v["subsidiaries"][0]["consideration"] = json!("50000.00")),
            "consolidation_eliminations.bargain_purchase",
        ),
        (
            edited(|v| v["parent"]["lines"][0]["amount"] = json!("46000.01")),
            "consolidation_eliminations.unbalanced",
        ),
        (
            edited(|v| v["parent"]["lines"][1]["account"] = json!("Cash")),
            "consolidation_eliminations.account_invalid",
        ),
        (
            edited(|v| v["parent"]["lines"][0]["account"] = json!("Goodwill")),
            "consolidation_eliminations.account_invalid",
        ),
        (
            edited(|v| v["subsidiaries"][0]["equity_at_acquisition"][0]["account"] = json!("Cash")),
            "consolidation_eliminations.account_invalid",
        ),
        (
            edited(|v| {
                v["subsidiaries"][0]["equity_at_acquisition"][1]["account"] = json!("Share capital")
            }),
            "consolidation_eliminations.account_invalid",
        ),
        (
            edited(|v| v["subsidiaries"][0]["ownership"] = json!("1")),
            "consolidation_eliminations.nci_invalid",
        ),
        (
            edited(|v| v["subsidiaries"][0]["ownership"] = json!("1.2")),
            "consolidation_eliminations.ownership_invalid",
        ),
        (
            edited(|v| v["subsidiaries"][0]["extra"] = json!(1)),
            "consolidation_eliminations.item_invalid",
        ),
    ]);
}

#[test]
fn refusals_of_malformed_items() {
    let item = "consolidation_eliminations.item_invalid";
    check(vec![
        (with(c1(), "parent", json!("P Ltd")), item),
        (edited(|v| v["parent"]["lines"] = json!([])), item),
        (
            edited(|v| v["parent"]["lines"][0]["class"] = json!("cash")),
            item,
        ),
        (
            edited(|v| v["parent"]["lines"][0]["amount"] = json!("1.001")),
            item,
        ),
        (edited(|v| v["parent"]["name"] = json!(" ")), item),
        (
            edited(|v| v["subsidiaries"][0]["equity_at_acquisition"] = json!([])),
            item,
        ),
        (
            edited(|v| v["subsidiaries"][0]["nci_at_acquisition"] = json!("-1.00")),
            item,
        ),
        (
            edited(|v| v["unrealised_profits"][0]["buyer"] = json!("S Ltd")),
            item,
        ),
        (
            edited(|v| v["intra_group_balances"][0]["amount"] = json!("0")),
            item,
        ),
        (
            edited(|v| v["subsidiaries"][0]["name"] = json!("P Ltd")),
            item,
        ),
    ]);
}

/// Assets less liabilities equal the owners' equity and the non-controlling
/// interests, current profit included.
fn balances(r: &Value) {
    let a = |k: &str| r["amounts"][k].as_str().unwrap().parse::<f64>().unwrap();
    let net = a("total_assets") - a("total_liabilities");
    let equity = a("equity_attributable_to_owners") + a("non_controlling_interests");
    assert!((net - equity).abs() < 0.005, "{net} {equity}");
}

#[test]
fn every_accepted_case_balances() {
    let mut downstream = c1();
    downstream["unrealised_profits"][0]["seller"] = json!("P Ltd");
    downstream["unrealised_profits"][0]["buyer"] = json!("S Ltd");
    let mut whole = c1();
    whole["subsidiaries"] = json!([subsidiary("1", "0.00")]);
    let mut loss = c1();
    loss["subsidiaries"][0]["lines"][5]["amount"] = json!("10000.00");
    loss["subsidiaries"][0]["lines"][0]["amount"] = json!("13000.00");
    for v in [
        c1(),
        with(c1(), "tax_rate", json!("0")),
        downstream,
        whole,
        loss,
    ] {
        balances(&run(&v));
    }
}

/// An investment line carrying more than the consideration would leave part
/// of it in the consolidated assets (Alcuin S1).
#[test]
fn an_investment_line_above_the_consideration_is_refused() {
    let v = edited(|v| {
        v["parent"]["lines"][1]["amount"] = json!("90000.00");
        v["parent"]["lines"][0]["amount"] = json!("36000.00");
    });
    assert_eq!(
        code(&v),
        "consolidation_eliminations.investment_not_eliminated"
    );
}

#[test]
fn refusals_of_eliminations() {
    let conflict = edited(|v| {
        v["subsidiaries"][0]["lines"][3]["account"] = json!("Receivable from S Ltd");
        v["intra_group_balances"][0]["payable"]["account"] = json!("Receivable from S Ltd");
    });
    check(vec![
        (conflict, "consolidation_eliminations.class_conflict"),
        (
            edited(|v| v["intra_group_balances"][0]["payable"]["entity"] = json!("Q Ltd")),
            "consolidation_eliminations.entity_unknown",
        ),
        (
            edited(|v| v["intra_group_balances"][0]["amount"] = json!("10000.01")),
            "consolidation_eliminations.exceeds_line",
        ),
        (
            edited(|v| v["intra_group_transactions"][0]["income"]["entity"] = json!("P Ltd")),
            "consolidation_eliminations.item_invalid",
        ),
        (
            with(c1(), "unrealised_profits", unrealised("30000.01")),
            "consolidation_eliminations.exceeds_line",
        ),
        (
            edited(|v| v["unrealised_profits"][0]["deferred_tax_account"] = json!("Goodwill")),
            "consolidation_eliminations.item_invalid",
        ),
        (
            edited(|v| v["unrealised_profits"][0]["cost_of_sales_account"] = json!("Sales")),
            "consolidation_eliminations.account_invalid",
        ),
    ]);
}

#[test]
fn every_year_boundary() {
    for year in ["2020-21", "2026-27"] {
        assert_eq!(
            run(&with(c1(), "year", json!(year)))["amounts"]["goodwill"],
            "20000.00"
        );
    }
}
