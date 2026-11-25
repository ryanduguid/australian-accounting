// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! An SMSF's earnings and tax shared across member accounts, with each
//! pension's minimum checked (docs/calculation-evidence.md#smsf-member-allocation).
//! The funds are made up; the expected figures were worked independently with
//! exact fractions in Python, and the minimums are pension-minimum's own.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "smsf-member-allocation";

fn run(input: &Value) -> Value {
    api::evaluate(CMD, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn tx(date: &str, kind: &str, amount: &str) -> Value {
    json!({"date": date, "type": kind, "amount": amount})
}

fn input(accounts: Value, earnings: &str, tax: &str) -> Value {
    json!({"accounts": accounts, "net_earnings": earnings, "income_tax": tax,
           "allocation_method": "daily_weighted_balance", "year": "2025-26",
           "scope_confirmed": true})
}

/// The catalogue's example: an accumulation account with 2 contributions and
/// a pension paying exactly its minimum.
fn example() -> Value {
    input(
        json!([
            {"label": "A", "kind": "accumulation", "opening_balance": "300000.00",
             "transactions": [tx("2025-10-01", "concessional_contribution", "10000.00"),
                              tx("2026-01-15", "non_concessional_contribution", "20000.00")]},
            {"label": "B", "kind": "account_based_pension", "opening_balance": "500000.00",
             "age": 68,
             "transactions": [tx("2025-12-31", "pension_payment", "15000.00"),
                              tx("2026-06-15", "pension_payment", "10000.00")]}
        ]),
        "60000.00",
        "6000.00",
    )
}

/// A transition to retirement pension over its maximum and an account-based
/// pension $500 short of its minimum.
fn shortfalls(exception: Option<bool>) -> Value {
    let mut d = json!({"label": "D", "kind": "account_based_pension",
        "opening_balance": "100000.00", "age": 80,
        "transactions": [tx("2026-06-30", "pension_payment", "6500.00")]});
    if let Some(e) = exception {
        d["minimum_exception"] = json!(e);
    }
    input(
        json!([
            {"label": "C", "kind": "transition_to_retirement", "opening_balance": "200000.00",
             "age": 60, "transactions": [tx("2025-07-31", "pension_payment", "25000.00")]},
            d
        ]),
        "30000.00",
        "3000.00",
    )
}

fn field(r: &Value, i: usize, name: &str) -> Value {
    r["accounts"][i][name].clone()
}

#[test]
fn the_example_shares_earnings_by_days_held() {
    let r = run(&example());
    // A: 300,000 + 10,000 x 273/365 + 20,000 x 167/365; B: 500,000 less
    // 15,000 x 182/365 and 10,000 x 16/365.
    assert_eq!(field(&r, 0, "weighted_balance"), "316630.14");
    assert_eq!(field(&r, 1, "weighted_balance"), "492082.19");
    assert_eq!(field(&r, 0, "earnings"), "23491.43");
    assert_eq!(field(&r, 1, "earnings"), "36508.57");
    // 15% of A's $10,000 concessional contribution; the other $4,500 is
    // earnings tax, all A's as B's pension is in the retirement phase.
    assert_eq!(field(&r, 0, "contributions_tax"), "1500.00");
    assert_eq!(field(&r, 0, "earnings_tax"), "4500.00");
    assert_eq!(field(&r, 1, "earnings_tax"), "0.00");
    assert_eq!(field(&r, 1, "earnings_taxed"), false);
    assert_eq!(field(&r, 0, "closing_balance"), "347491.43");
    assert_eq!(field(&r, 1, "closing_balance"), "511508.57");
    assert_eq!(r["amounts"]["contributions_tax"], "1500.00");
    assert_eq!(r["amounts"]["earnings_tax"], "4500.00");
    assert_eq!(r["amounts"]["closing_balances"], "859000.00");
    let p = &r["accounts"][1]["pension"];
    assert_eq!(p["minimum_payment"], "25000.00");
    assert_eq!(p["minimum_met"], true);
    assert_eq!(p["shortfall"], "0.00");
    assert!(p.get("maximum_payment").is_none());
    assert_eq!(r["accounts"][0].get("pension"), None);
}

#[test]
fn the_minimum_is_pension_minimums() {
    for (age, balance) in [(64, "123456.78"), (65, "500000.00"), (95, "87654.32")] {
        let mut i = example();
        i["accounts"][1]["age"] = json!(age);
        i["accounts"][1]["opening_balance"] = json!(balance);
        let ours = &run(&i)["accounts"][1]["pension"];
        let theirs = api::evaluate(
            "pension-minimum",
            &json!({"account_balance": balance, "age": age, "days": 365,
                    "year": "2025-26", "scope_confirmed": true}),
        )
        .unwrap();
        assert_eq!(
            ours["minimum_payment"], theirs["amounts"]["minimum_payment"],
            "{age}"
        );
        assert_eq!(
            ours["percentage_factor"], theirs["rates"]["percentage_factor"],
            "{age}"
        );
    }
}

#[test]
fn a_missed_standard_taxes_the_pension() {
    let r = run(&shortfalls(None));
    assert_eq!(field(&r, 0, "weighted_balance"), "177054.79");
    assert_eq!(field(&r, 1, "weighted_balance"), "99982.19");
    assert_eq!(field(&r, 0, "earnings"), "19173.05");
    assert_eq!(field(&r, 1, "earnings"), "10826.95");
    // C is not in the retirement phase; D missed its minimum.
    assert_eq!(field(&r, 0, "earnings_tax"), "1917.30");
    assert_eq!(field(&r, 1, "earnings_tax"), "1082.70");
    let c = &r["accounts"][0]["pension"];
    assert_eq!(c["minimum_payment"], "8000.00");
    assert_eq!(c["maximum_payment"], "20000.00");
    assert_eq!(c["maximum_exceeded"], true);
    let d = &r["accounts"][1]["pension"];
    assert_eq!(d["minimum_payment"], "7000.00");
    assert_eq!(d["shortfall"], "500.00");
    assert_eq!(d["one_twelfth_of_minimum"], "583.33");
    assert_eq!(d["minimum_met"], false);
    let warnings = r["warnings"].to_string();
    assert!(warnings.contains("C did not meet the pension standards"));
    assert!(warnings.contains("D did not meet the pension standards"));
}

#[test]
fn a_stated_exception_keeps_the_pension_exempt() {
    let r = run(&shortfalls(Some(true)));
    assert_eq!(field(&r, 1, "earnings_taxed"), false);
    assert_eq!(field(&r, 0, "earnings_tax"), "3000.00");
    assert_eq!(field(&r, 1, "earnings_tax"), "0.00");
    assert_eq!(
        r["accounts"][1]["pension"]["minimum_exception_stated"],
        true
    );
    // false is the same as leaving it out.
    assert_eq!(run(&shortfalls(Some(false))), run(&shortfalls(None)));
}

#[test]
fn shares_round_cumulatively_to_the_total() {
    let acc =
        |label: &str| json!({"label": label, "kind": "accumulation", "opening_balance": "1000.00"});
    let r = run(&input(
        json!([acc("x"), acc("y"), acc("z")]),
        "100.00",
        "-30.00",
    ));
    let earnings: Vec<Value> = (0..3).map(|i| field(&r, i, "earnings")).collect();
    assert_eq!(earnings, ["33.33", "33.34", "33.33"]);
    // An earnings tax refund with no retirement phase pension is shared too.
    let tax: Vec<Value> = (0..3).map(|i| field(&r, i, "earnings_tax")).collect();
    assert_eq!(tax, ["-10.00", "-10.00", "-10.00"]);
    let r = run(&input(
        json!([acc("x"), acc("y"), acc("z")]),
        "-100.00",
        "0",
    ));
    let earnings: Vec<Value> = (0..3).map(|i| field(&r, i, "earnings")).collect();
    assert_eq!(earnings, ["-33.33", "-33.34", "-33.33"]);
}

#[test]
fn withdrawals_and_roll_overs() {
    let acc = json!({"label": "x", "kind": "accumulation", "opening_balance": "1000.00",
        "transactions": [tx("2026-06-30", "rollover_in", "365.00"),
                         tx("2026-06-30", "lump_sum", "1365.00")]});
    // Same-day moves net before the balance is checked: nil, not below it;
    // each weighs 1 day of 365.
    let r = run(&input(json!([acc]), "0", "0"));
    assert_eq!(field(&r, 0, "weighted_balance"), "997.26");
    assert_eq!(field(&r, 0, "rollover_in"), "365.00");
    assert_eq!(field(&r, 0, "closing_balance"), "0.00");
    // A pension can be partly commuted, which does not count to its minimum.
    let mut i = example();
    i["accounts"][1]["transactions"][1] = tx("2026-06-15", "lump_sum", "10000.00");
    let p = &run(&i)["accounts"][1]["pension"];
    assert_eq!(p["pension_payments"], "15000.00");
    assert_eq!(p["shortfall"], "10000.00");
}

fn with(path: &str, v: Value) -> Value {
    let mut i = example();
    let (parent, key) = path.rsplit_once('/').unwrap();
    match i.pointer_mut(parent).unwrap() {
        Value::Object(o) => {
            o.insert(key.to_string(), v);
        }
        list => list[key.parse::<usize>().unwrap()] = v,
    }
    i
}

#[test]
fn account_refusals() {
    let cases = [
        (
            with("/allocation_method", json!("opening_balance")),
            "smsf_member_allocation.method_unsupported",
        ),
        (
            with("/accounts", json!([])),
            "smsf_member_allocation.account_invalid",
        ),
        (
            with("/accounts/0/kind", json!("reserve")),
            "smsf_member_allocation.account_invalid",
        ),
        (
            with("/accounts/0/label", json!(" ")),
            "smsf_member_allocation.account_invalid",
        ),
        (
            with("/accounts/0/opening_balance", json!("-1")),
            "smsf_member_allocation.account_invalid",
        ),
        (
            with("/accounts/0/age", json!(70)),
            "smsf_member_allocation.account_invalid",
        ),
        (
            with("/accounts/1/age", json!(151)),
            "smsf_member_allocation.account_invalid",
        ),
        (
            with("/accounts/1/minimum_exception", json!("yes")),
            "smsf_member_allocation.account_invalid",
        ),
        (
            with("/accounts/0/extra", json!(1)),
            "smsf_member_allocation.account_invalid",
        ),
    ];
    for (i, want) in cases {
        assert_eq!(code(&i), want, "{i}");
    }
    let many: Vec<Value> = (0..31).map(|_| example()["accounts"][0].clone()).collect();
    assert_eq!(
        code(&with("/accounts", json!(many))),
        "smsf_member_allocation.account_invalid"
    );
}

#[test]
fn transaction_refusals() {
    let cases = [
        (
            with("/accounts/0/transactions", json!({})),
            "smsf_member_allocation.transaction_invalid",
        ),
        (
            with("/accounts/0/transactions/0/date", json!("2026-07-01")),
            "smsf_member_allocation.transaction_invalid",
        ),
        (
            with("/accounts/0/transactions/0/amount", json!("0")),
            "smsf_member_allocation.transaction_invalid",
        ),
        (
            with("/accounts/0/transactions/0/type", json!("gift")),
            "smsf_member_allocation.transaction_invalid",
        ),
        (
            with("/accounts/0/transactions/0", json!([])),
            "smsf_member_allocation.transaction_invalid",
        ),
        (
            with("/accounts/0/transactions/0/type", json!("pension_payment")),
            "smsf_member_allocation.transaction_not_allowed",
        ),
        (
            with("/accounts/1/transactions/0/type", json!("rollover_in")),
            "smsf_member_allocation.transaction_not_allowed",
        ),
        (
            with(
                "/accounts/0/transactions/1",
                tx("2026-01-15", "lump_sum", "310000.01"),
            ),
            "smsf_member_allocation.balance_negative",
        ),
        (
            with("/accounts/1/opening_balance", json!("0")),
            "smsf_member_allocation.pension_not_whole_year",
        ),
        (
            with("/accounts/1/transactions/1/amount", json!("485000.00")),
            "smsf_member_allocation.pension_not_whole_year",
        ),
        (
            with("/net_earnings", json!("0.001")),
            "input.signed_amount_invalid",
        ),
        (with("/year", json!("2023-24")), "period.unsupported"),
        (
            with("/scope_confirmed", json!(false)),
            "scope.not_confirmed",
        ),
    ];
    for (i, want) in cases {
        assert_eq!(code(&i), want, "{i}");
    }
    let txs: Vec<Value> = (0..1001)
        .map(|_| tx("2026-01-01", "rollover_in", "1.00"))
        .collect();
    assert_eq!(
        code(&with("/accounts/0/transactions", json!(txs))),
        "smsf_member_allocation.transaction_invalid"
    );
}

#[test]
fn tax_and_earnings_with_nowhere_to_go() {
    let nil = json!([{"label": "x", "kind": "accumulation", "opening_balance": "0"}]);
    assert_eq!(
        code(&input(nil.clone(), "1.00", "0")),
        "smsf_member_allocation.no_balance"
    );
    // Nil earnings on nil balances share nothing.
    assert_eq!(field(&run(&input(nil, "0", "0")), 0, "earnings"), "0.00");
    // Only a retirement phase pension: no account bears earnings tax.
    let mut i = example();
    i["accounts"].as_array_mut().unwrap().remove(0);
    assert_eq!(
        code(&{
            let mut j = i.clone();
            j["income_tax"] = json!("1.00");
            j
        }),
        "smsf_member_allocation.tax_unallocable"
    );
    // A refund beside a retirement phase pension is the trustee's to share.
    let mut j = example();
    j["income_tax"] = json!("1000.00");
    assert_eq!(
        code(&j),
        "smsf_member_allocation.refund_with_retirement_phase"
    );
    i["income_tax"] = json!("0");
    assert_eq!(field(&run(&i), 0, "earnings"), "60000.00");
}

#[test]
fn a_transition_to_retirement_pension_reaching_65_is_refused() {
    // ITAA 1997 s 307-80(3)(b): attaining 65 ends a transition to retirement
    // pension's non-retirement phase status, at 64 on 1 July within the year.
    for age in [64, 65, 80] {
        let mut i = shortfalls(None);
        i["accounts"][0]["age"] = json!(age);
        assert_eq!(
            code(&i),
            "smsf_member_allocation.transition_reaches_65",
            "{age}"
        );
    }
    let mut i = shortfalls(None);
    i["accounts"][0]["age"] = json!(63);
    assert_eq!(field(&run(&i), 0, "earnings_taxed"), true);
}

#[test]
fn losses_and_tax_below_nil_are_refused() {
    let acc = json!([{"label": "x", "kind": "accumulation", "opening_balance": "100.00"}]);
    assert_eq!(
        code(&input(acc.clone(), "-200.00", "0")),
        "smsf_member_allocation.balance_negative"
    );
    assert_eq!(
        code(&input(acc.clone(), "0", "100.01")),
        "smsf_member_allocation.balance_negative"
    );
    let r = run(&input(acc, "-100.00", "0"));
    assert_eq!(field(&r, 0, "closing_balance"), "0.00");
    assert_eq!(r["amounts"]["closing_balances"], "0.00");
}
