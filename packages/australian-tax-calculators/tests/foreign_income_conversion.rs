// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Foreign currency translation (ITAA 1997 s 960-50, compilations 210 and
//! 268, and the 1997 and 2021 Regulations' item 12 and Schedule 2, read on
//! 22 November 2026) against the ATO's rental expense and rental income
//! examples and the rate it publishes for 2025-26
//! (docs/calculation-evidence.md#foreign-income-conversion).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(year: &str, items: Value) -> Value {
    json!({"year": year, "scope_confirmed": true, "items": items})
}

fn run(items: Value) -> Value {
    api::evaluate("foreign-income-conversion", &input("2025-26", items)).unwrap()
}

fn code(items: Value) -> &'static str {
    let given = input("2025-26", items);
    api::evaluate_coded("foreign-income-conversion", &given)
        .unwrap_err()
        .code
}

/// `base` with the fields of `extra` added or replaced.
fn merged(mut base: Value, extra: Value) -> Value {
    for (key, value) in extra.as_object().unwrap() {
        base[key] = value.clone();
    }
    base
}

/// An item of `kind` in NZD at a specific rate on `day`, with `dates`.
fn item(kind: &str, amount: &str, rate: &str, day: &str, dates: Value) -> Value {
    let base = json!({
        "kind": kind, "currency": "NZD", "amount": amount, "rate": rate,
        "rate_source": "ATO example", "basis": "specific", "rate_date": day,
    });
    merged(base, dates)
}

/// A deduction paid on the day it became deductible.
fn paid(amount: &str, rate: &str, day: &str) -> Value {
    item(
        "deduction",
        amount,
        rate,
        day,
        json!({"deductible": day, "paid": day}),
    )
}

/// USD6,000 at the ATO's average for the year ended 30 June 2026 (0.6785).
fn average(kind: &str, dates: Value, from: &str, to: &str) -> Value {
    let base = json!({
        "kind": kind, "currency": "USD", "amount": "6000.00", "rate": "0.6785",
        "rate_source": "ATO, average for the year ended 30 June 2026",
        "basis": "average", "average_from": from, "average_to": to,
    });
    merged(base, dates)
}

fn aud(result: &Value, i: usize) -> &str {
    result["items"][i]["aud"].as_str().unwrap()
}

fn rule(result: &Value, i: usize) -> &str {
    result["items"][i]["translation_rule"].as_str().unwrap()
}

/// The ATO's 'Translation of foreign currency rental expenses': NZD1,000 of
/// borrowing expenses and NZD1,500 of interest at 1.10 and NZD600 of council
/// rates at 1.20, each paid when deductible (AUD909, AUD1,364 and AUD500).
#[test]
fn ato_rental_expenses() {
    let result = run(json!([
        paid("1000", "1.10", "2025-07-01"),
        paid("1500", "1.10", "2025-07-01"),
        paid("600", "1.20", "2025-08-10"),
    ]));
    let got = [aud(&result, 0), aud(&result, 1), aud(&result, 2)];
    assert_eq!(got, ["909.09", "1363.64", "500.00"]);
    assert_eq!(result["amounts"]["deductions"], "2772.73");
    assert_eq!(result["amounts"]["ordinary_income"], "0.00");
    assert!(rule(&result, 0).contains("item 8(a)"));
    assert_eq!(result["average_rate_rule"], Value::Null);
}

/// The same example's air conditioner, held and paid for on the day of 1.05
/// (AUD1,905 there).
#[test]
fn ato_air_conditioner() {
    let dates = json!({"held": "2025-10-22", "satisfied": "2025-10-22"});
    let unit = item(
        "depreciating_asset_cost",
        "2000",
        "1.05",
        "2025-10-22",
        dates,
    );
    let result = run(json!([unit]));
    assert_eq!(aud(&result, 0), "1904.76");
    assert_eq!(result["amounts"]["depreciating_asset_costs"], "1904.76");
    assert!(rule(&result, 0).contains("item 2(a)"));
}

/// The ATO's 'Translation of foreign currency rental income': rent derived
/// when the agent receives it, so item 6 translates at that day's rate; a
/// receipt before derivation takes the receipt's day, one after does not.
#[test]
fn ordinary_income_at_the_earlier_of_receipt_and_derivation() {
    let rent = |day: &str, dates: Value| item("ordinary_income", "1100", "1.10", day, dates);
    let result = run(json!([rent(
        "2025-07-28",
        json!({"derived": "2025-07-28"})
    )]));
    assert_eq!(aud(&result, 0), "1000.00");
    assert!(rule(&result, 0).contains("item 6(b)"));
    let early = json!({"derived": "2025-07-28", "received": "2025-07-01"});
    let result = run(json!([rent("2025-07-01", early)]));
    assert_eq!(result["items"][0]["translated_on"], "2025-07-01");
    assert!(rule(&result, 0).contains("item 6(a)"));
    let late = json!({"derived": "2025-07-28", "received": "2025-08-01"});
    let result = run(json!([rent("2025-07-28", late)]));
    assert_eq!(result["items"][0]["translated_on"], "2025-07-28");
}

/// `kind` with `dates` translates on `day` under `item`, at 2 to the dollar.
fn translates(kind: &str, dates: Value, day: &str, item_rule: &str) {
    let result = run(json!([item(kind, "100", "2", day, dates)]));
    let row = &result["items"][0];
    assert_eq!(row["translated_on"], day, "{kind}");
    assert!(rule(&result, 0).contains(item_rule), "{kind} {item_rule}");
    assert_eq!(row["aud"], "50.00");
}

/// Statutory income and deductions at the earlier time.
#[test]
fn statutory_income_and_deductions() {
    let early = json!({"included": "2025-09-30", "received": "2025-09-01"});
    translates("statutory_income", early, "2025-09-01", "item 7(a)");
    let unpaid = json!({"included": "2025-09-30"});
    translates("statutory_income", unpaid, "2025-09-30", "item 7(b)");
    let later = json!({"deductible": "2025-09-30", "paid": "2025-10-15"});
    translates("deduction", later, "2025-09-30", "item 8(b)");
}

/// An asset's obligation satisfied first, or not; CGT amounts and foreign
/// tax at their one time.
#[test]
fn assets_cgt_and_foreign_tax() {
    let first = json!({"held": "2025-09-30", "satisfied": "2025-09-01"});
    translates("depreciating_asset_cost", first, "2025-09-01", "item 2(b)");
    let same = json!({"held": "2025-09-30", "satisfied": "2025-09-30"});
    translates("depreciating_asset_cost", same, "2025-09-30", "item 2(a)");
    translates(
        "cgt_event",
        json!({"event": "2025-11-05"}),
        "2025-11-05",
        "item 5",
    );
    let tax = json!({"paid": "2026-03-31"});
    translates("foreign_tax_paid", tax.clone(), "2026-03-31", "item 11");
    let result = run(json!([item(
        "foreign_tax_paid",
        "100",
        "2",
        "2026-03-31",
        tax
    )]));
    assert_eq!(result["amounts"]["foreign_tax_paid"], "50.00");
}

/// A specific rate must be the rate for the day the rule names: a rate for
/// the day before is refused, not used.
#[test]
fn a_specific_rate_for_another_day_is_refused() {
    let dates = json!({"deductible": "2025-08-10"});
    let wrong = item("deduction", "600", "1.20", "2025-08-09", dates);
    assert_eq!(code(json!([wrong])), "foreign_income_conversion.rate_date");
}

/// USD6,000 at 0.6785; the 2021 Regulations' clause from 2021-22.
#[test]
fn an_average_for_the_year() {
    let derived = json!({"derived": "2026-03-31"});
    let result = run(json!([average(
        "ordinary_income",
        derived,
        "2025-07-01",
        "2026-06-30"
    )]));
    assert_eq!(aud(&result, 0), "8843.04");
    let rule = result["average_rate_rule"].as_str().unwrap();
    assert!(rule.contains("Regulations 2021"));
    let derived = json!({"derived": "2021-03-31"});
    let old = average("ordinary_income", derived, "2020-07-01", "2021-06-30");
    let result = api::evaluate("foreign-income-conversion", &input("2020-21", json!([old])));
    let rule = result.unwrap()["average_rate_rule"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(rule.contains("Regulations 1997"));
}

/// The period may be up to 12 months and must contain the day.
#[test]
fn an_average_period_is_refused_outside_its_bounds() {
    let derived = json!({"derived": "2026-03-31"});
    for (from, to) in [
        ("2025-07-01", "2026-07-01"),
        ("2026-06-30", "2025-07-01"),
        ("2025-07-01", "2025-12-31"),
    ] {
        let given = average("ordinary_income", derived.clone(), from, to);
        assert_eq!(
            code(json!([given])),
            "foreign_income_conversion.average_period"
        );
    }
}

/// An average for a one-off capital amount is warned about (the ATO's
/// average rate examples 5 and 7), not refused.
#[test]
fn an_average_for_a_capital_amount_is_warned() {
    let event = json!({"event": "2026-01-15"});
    let result = run(json!([average(
        "cgt_event",
        event,
        "2025-07-01",
        "2026-06-30"
    )]));
    assert!(result["warnings"].to_string().contains("examples 5 and 7"));
    let derived = json!({"derived": "2026-01-15"});
    let result = run(json!([average(
        "ordinary_income",
        derived,
        "2025-07-01",
        "2026-06-30"
    )]));
    assert!(!result["warnings"].to_string().contains("examples 5 and 7"));
}

/// Each refused field of a good item, and the code.
#[test]
fn item_field_refusals() {
    let good = paid("600", "1.20", "2025-08-10");
    let cases = [
        (
            json!({"kind": "trading_stock"}),
            "foreign_income_conversion.kind_invalid",
        ),
        (
            json!({"currency": "AUD"}),
            "foreign_income_conversion.currency_invalid",
        ),
        (
            json!({"currency": "nzd"}),
            "foreign_income_conversion.currency_invalid",
        ),
        (
            json!({"basis": "spot"}),
            "foreign_income_conversion.basis_invalid",
        ),
        (json!({"rate": "0"}), "input.amount_invalid"),
        (json!({"rate": "1.1234567"}), "input.amount_invalid"),
        (json!({"amount": "-1"}), "input.amount_invalid"),
        (json!({"amount": "1.23456"}), "input.amount_invalid"),
        (
            json!({"rate_source": ""}),
            "foreign_income_conversion.text_invalid",
        ),
        (json!({"received": "2025-08-10"}), "input.field_unknown"),
        (json!({"deductible": "2025-02-30"}), "input.date_invalid"),
    ];
    for (change, expected) in cases {
        assert_eq!(
            code(json!([merged(good.clone(), change.clone())])),
            expected,
            "{change}"
        );
    }
    let mut missing = good;
    missing.as_object_mut().unwrap().remove("rate_source");
    assert_eq!(code(json!([missing])), "input.missing_field");
}

#[test]
fn list_and_year_refusals() {
    let good = paid("600", "1.20", "2025-08-10");
    for items in [json!([]), json!("x"), json!([1])] {
        assert_eq!(code(items), "foreign_income_conversion.items_invalid");
    }
    let many: Vec<Value> = (0..201).map(|_| good.clone()).collect();
    assert_eq!(code(json!(many)), "foreign_income_conversion.items_invalid");
    let given = input("2019-20", json!([good]));
    let refused = api::evaluate_coded("foreign-income-conversion", &given).unwrap_err();
    assert_eq!(refused.code, "period.unsupported");
}
