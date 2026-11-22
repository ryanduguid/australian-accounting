// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `foreign-resident-withholding` against the ATO's foreign resident capital
//! gains withholding examples (Jane and Toni; Willow and Stanley; Maisie and
//! Max; Zac and Jen; Kim), the rate and threshold boundaries of TAA 1953
//! Sch 1 ss 14-200(3) and 14-215(1)(a) under Acts No. 57 of 2017 and No. 135
//! of 2024, and the refusals, all read on 22 November 2026
//! (docs/calculation-evidence.md#foreign-resident-withholding).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn none() -> Value {
    json!({"interest": "1", "document": "none"})
}

fn clearance() -> Value {
    json!({"interest": "1", "document": "clearance_certificate"})
}

/// One foreign resident vendor selling real property for $1,200,000 under a
/// contract of 16 December 2024 (the ATO's Jane and Toni example).
fn base() -> Value {
    json!({
        "payment": "capital_gains", "acquisition_date": "2024-12-16",
        "asset": "taxable_australian_real_property",
        "market_value": "1200000.00", "cost_base_first_element": "1200000.00",
        "option_payments": "0.00", "vendors": [none()], "exclusion": null,
        "class_variation_to_nil": false, "year": "2024-25", "scope_confirmed": true,
    })
}

fn with(changes: Value) -> Value {
    let mut input = base();
    for (key, value) in changes.as_object().unwrap() {
        input[key] = value.clone();
    }
    input
}

fn run(input: &Value) -> Value {
    api::evaluate("foreign-resident-withholding", input).unwrap()
}

fn payable(input: &Value) -> String {
    run(input)["amounts"]["amount_payable"]
        .as_str()
        .unwrap()
        .to_string()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("foreign-resident-withholding", input)
        .unwrap_err()
        .code
}

fn price(date: &str, year: &str, amount: &str) -> Value {
    with(json!({
        "acquisition_date": date, "year": year,
        "market_value": amount, "cost_base_first_element": amount,
    }))
}

/// Jane and Toni: a contract signed on 16 December 2024 takes 12.5% of
/// $1.2 million, $150,000; signed after 1 January 2025, 15%, $180,000.
#[test]
fn the_contract_date_sets_the_rate() {
    let r = run(&base());
    assert_eq!(r["amounts"]["amount_payable"], "150000.00");
    assert_eq!(r["rates"]["rate"], "0.125");
    assert_eq!(r["rates"]["threshold"], "750000");
    let later = price("2025-01-02", "2024-25", "1200000.00");
    assert_eq!(payable(&later), "180000.00");
    assert_eq!(run(&later)["rates"]["threshold"], "none");
}

/// Willow and Stanley: a $600,000 home, contract of 8 January 2025, Willow's
/// clearance certificate given and Stanley's not: 15% of his half, $45,000.
/// Below the old $750,000 threshold, an Australian resident without a
/// certificate is still withheld from (s 14-210(1)(e)).
#[test]
fn willow_and_stanley_one_clearance_certificate() {
    let mut input = price("2025-01-08", "2024-25", "600000.00");
    input["vendors"] = json!([clearance(), none()]);
    let r = run(&input);
    assert_eq!(r["amounts"]["amount_payable"], "45000.00");
    assert_eq!(r["vendors"][0]["amount"], "0.00");
    assert_eq!(r["vendors"][1]["amount"], "45000.00");
    assert_eq!(r["vendors"][1]["share_of_base"], "300000.00");
}

/// Maisie and Max: both certificates, nothing; neither, $97,500, $48,750 each.
#[test]
fn maisie_and_max_both_or_neither() {
    let mut input = price("2025-01-14", "2024-25", "650000.00");
    input["vendors"] = json!([clearance(), clearance()]);
    let r = run(&input);
    assert_eq!(r["amounts"]["amount_payable"], "0.00");
    assert!(r["nil_reason"].as_str().unwrap().contains("s 14-210(2)"));
    input["vendors"] = json!([none(), none()]);
    let r = run(&input);
    assert_eq!(r["amounts"]["amount_payable"], "97500.00");
    assert_eq!(r["vendors"][0]["amount"], "48750.00");
}

/// Zac and Jen: Jen's certificate, Zac's half at 15%, $75,000; with Zac's 5%
/// notice of variation, $25,000; above the notice's maximum price, the full
/// rate again.
#[test]
fn zac_and_jen_with_and_without_a_variation() {
    let mut input = price("2025-02-01", "2024-25", "1000000.00");
    input["vendors"] = json!([none(), clearance()]);
    assert_eq!(payable(&input), "75000.00");
    let notice = |maximum: &str| json!({"interest": "1", "document": "variation", "rate": "0.05", "maximum_price": maximum});
    input["vendors"] = json!([notice("1000000.00"), clearance()]);
    assert_eq!(payable(&input), "25000.00");
    input["vendors"] = json!([notice("999999.99"), clearance()]);
    assert_eq!(payable(&input), "75000.00");
}

/// Kim: 3 equal vendors, no certificates, $1.3 million: $195,000, $65,000
/// each.
#[test]
fn kim_three_vendors() {
    let mut input = price("2025-01-11", "2024-25", "1300000.00");
    input["vendors"] = json!([none(), none(), none()]);
    let r = run(&input);
    assert_eq!(r["amounts"]["amount_payable"], "195000.00");
    for i in 0..3 {
        assert_eq!(r["vendors"][i]["amount"], "65000.00");
    }
}

/// Unequal interests: a 3:1 split with the larger vendor certified.
#[test]
fn interests_weight_each_vendor() {
    let mut input = price("2026-03-01", "2025-26", "800000.00");
    input["vendors"] = json!([
        {"interest": "3", "document": "clearance_certificate"},
        {"interest": "1", "document": "none"},
    ]);
    assert_eq!(payable(&input), "30000.00");
}

/// s 14-215(1)(a): before 1 January 2025 a market value under $750,000
/// excludes the transaction; $750,000 does not; from 1 January 2025 there is
/// no threshold.
#[test]
fn threshold_boundaries() {
    let under = price("2024-12-31", "2024-25", "749999.99");
    let r = run(&under);
    assert_eq!(r["amounts"]["amount_payable"], "0.00");
    assert!(r["nil_reason"].as_str().unwrap().contains("s 14-215(1)(a)"));
    assert_eq!(r["vendors"][0]["amount"], "0.00");
    assert_eq!(
        payable(&price("2024-12-31", "2024-25", "750000.00")),
        "93750.00"
    );
    assert_eq!(
        payable(&price("2025-01-01", "2024-25", "500000.00")),
        "75000.00"
    );
    assert_eq!(
        payable(&price("2017-07-01", "2017-18", "750000.00")),
        "93750.00"
    );
    assert_eq!(
        payable(&price("2027-06-30", "2026-27", "100000.00")),
        "15000.00"
    );
}

/// The threshold reads market value, not the price paid.
#[test]
fn threshold_reads_market_value() {
    let input = with(json!({"market_value": "700000.00", "cost_base_first_element": "800000.00"}));
    assert_eq!(payable(&input), "0.00");
}

/// s 14-200(3)(a)(ii): the option fee comes off the base.
#[test]
fn option_payments_reduce_the_base() {
    let mut input = price("2026-05-01", "2025-26", "1000000.00");
    input["option_payments"] = json!("10000.00");
    let r = run(&input);
    assert_eq!(r["amounts"]["withholding_base"], "990000.00");
    assert_eq!(r["amounts"]["amount_payable"], "148500.00");
    // A notice's maximum price caps the price, not the base after the fee.
    let notice = json!({"interest": "1", "document": "variation", "rate": "0.05", "maximum_price": "995000.00"});
    input["vendors"] = json!([notice]);
    assert_eq!(payable(&input), "148500.00");
    input["option_payments"] = json!("1000000.01");
    assert_eq!(
        code(&input),
        "foreign_resident_withholding.option_exceeds_price"
    );
}

/// Class variations to nil and s 14-215 exclusions.
#[test]
fn class_variation_and_exclusions_give_nil() {
    let varied = with(json!({"class_variation_to_nil": true}));
    let r = run(&varied);
    assert_eq!(r["amounts"]["amount_payable"], "0.00");
    assert_eq!(r["amounts"]["amount_at_full_rate"], "150000.00");
    for (exclusion, paragraph) in [
        ("approved_stock_exchange", "(b)"),
        ("crossing_system", "(c)"),
        ("already_withheld", "(d)"),
        ("securities_lending", "(e)"),
        ("insolvent_company", "(f)"),
        ("bankruptcy", "(g)"),
    ] {
        let r = run(&with(json!({"exclusion": exclusion})));
        assert_eq!(r["amounts"]["amount_payable"], "0.00");
        assert!(r["nil_reason"].as_str().unwrap().contains(paragraph));
    }
    assert_eq!(
        code(&with(json!({"exclusion": "under_threshold"}))),
        "foreign_resident_withholding.exclusion_invalid"
    );
}

#[test]
fn refuses_dates_outside_the_scope_or_year() {
    let early = price("2017-06-30", "2016-17", "1000000.00");
    assert_eq!(code(&early), "period.unsupported");
    let late = price("2027-07-01", "2027-28", "1000000.00");
    assert_eq!(code(&late), "period.unsupported");
    let outside = price("2025-07-01", "2024-25", "1000000.00");
    assert_eq!(
        code(&outside),
        "foreign_resident_withholding.date_outside_year"
    );
    assert_eq!(
        code(&with(json!({"acquisition_date": "16/12/2024"}))),
        "input.date_invalid"
    );
}

#[test]
fn refuses_payments_and_assets_not_built() {
    for payment in [
        "interest",
        "dividend",
        "royalty",
        "mit_fund_payment",
        "s12_315",
    ] {
        assert_eq!(
            code(&with(json!({"payment": payment}))),
            "foreign_resident_withholding.payment_unsupported"
        );
    }
    for asset in [
        "indirect_australian_real_property_interest",
        "option_or_right",
        "look_through_earnout",
    ] {
        assert_eq!(
            code(&with(json!({"asset": asset}))),
            "foreign_resident_withholding.asset_unsupported"
        );
    }
    let company_title = with(json!({"asset": "company_title_interest"}));
    assert_eq!(payable(&company_title), "150000.00");
}

#[test]
fn refuses_bad_vendors_and_variations() {
    let bad = "foreign_resident_withholding.vendors_invalid";
    assert_eq!(code(&with(json!({"vendors": []}))), bad);
    assert_eq!(code(&with(json!({"vendors": "none"}))), bad);
    assert_eq!(code(&with(json!({"vendors": vec![none(); 51]}))), bad);
    assert_eq!(
        code(&with(
            json!({"vendors": [{"interest": "0", "document": "none"}]})
        )),
        bad
    );
    assert_eq!(
        code(&with(
            json!({"vendors": [{"interest": "1", "document": "declaration"}]})
        )),
        bad
    );
    assert_eq!(
        code(&with(
            json!({"vendors": [{"interest": "1", "document": "variation", "maximum_price": "1"}]})
        )),
        bad
    );
    assert_eq!(
        code(&with(
            json!({"vendors": [{"interest": "1", "document": "none", "rate": "0.05"}]})
        )),
        "input.field_unknown"
    );
    assert_eq!(
        code(&with(
            json!({"vendors": [{"interest": "1", "document": "variation", "rate": "0.13", "maximum_price": "2000000"}]})
        )),
        "foreign_resident_withholding.variation_invalid"
    );
    let at_full = with(
        json!({"vendors": [{"interest": "1", "document": "variation", "rate": "0.125", "maximum_price": "2000000"}]}),
    );
    assert_eq!(payable(&at_full), "150000.00");
}

#[test]
fn refuses_unconfirmed_scope() {
    assert_eq!(
        code(&with(json!({"scope_confirmed": false}))),
        "scope.not_confirmed"
    );
}
