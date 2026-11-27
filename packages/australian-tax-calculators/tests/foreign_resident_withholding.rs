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
        "option_payments": "0.00", "sale_price": "1200000.00",
        "vendors": [none()], "exclusion": null,
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
        "sale_price": amount,
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
    input["sale_price"] = json!("990000.00");
    let r = run(&input);
    assert_eq!(r["amounts"]["withholding_base"], "990000.00");
    assert_eq!(r["amounts"]["amount_payable"], "148500.00");
    // The first element includes the option fee (ITAA 1997 s 134-1(1) item
    // 1); a notice's maximum caps the sale price, which does not.
    let notice = json!({"interest": "1", "document": "variation", "rate": "0.05", "maximum_price": "995000.00"});
    input["vendors"] = json!([notice]);
    assert_eq!(payable(&input), "49500.00");
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
        "look_through_earnout",
    ] {
        assert_eq!(
            code(&with(json!({"payment": payment}))),
            "foreign_resident_withholding.payment_unsupported"
        );
    }
    for asset in [
        "indirect_australian_real_property_interest",
        "option_or_right",
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

/// With no vendor holding a document the class variations do not apply: the
/// amount is one s 14-200(3)(a) figure, rounded once (15.009 to 15.01, not
/// 2 x 7.50).
#[test]
fn no_documents_round_the_single_amount_once() {
    let mut input = price("2025-03-01", "2024-25", "100.06");
    input["vendors"] = json!([none(), none()]);
    let r = run(&input);
    assert_eq!(r["amounts"]["amount_payable"], "15.01");
    assert_eq!(r["amounts"]["amount_at_full_rate"], "15.01");
}

/// The vendor rows add up to the amount payable: with no document the one
/// rounding's left-over cent goes to the row with the largest remainder, and
/// the row says so.
#[test]
fn no_document_rows_reconcile_to_the_amount_payable() {
    let mut input = price("2025-03-01", "2024-25", "100.06");
    input["vendors"] = json!([none(), none()]);
    let r = run(&input);
    assert_eq!(r["vendors"][0]["amount"], "7.51");
    assert_eq!(r["vendors"][1]["amount"], "7.50");
    assert!(
        r["vendors"][0]["basis"]
            .as_str()
            .unwrap()
            .contains("rounding cent")
    );
    // 3 equal vendors of 100.01 at 15%: 5.0005 each, 15.0015 in all, 15.00.
    let mut three = price("2025-03-01", "2024-25", "100.01");
    three["vendors"] = json!([none(), none(), none()]);
    let r = run(&three);
    assert_eq!(r["amounts"]["amount_payable"], "15.00");
    for i in 0..3 {
        assert_eq!(r["vendors"][i]["amount"], "5.00");
    }
}

/// s 14-215(2) adds similar interests only where the asset is an interest in
/// real property; a company title interest is the shares, valued alone.
#[test]
fn company_title_threshold_reads_the_interest_alone() {
    let mut under = price("2024-12-31", "2024-25", "749999.99");
    under["asset"] = json!("company_title_interest");
    assert_eq!(payable(&under), "0.00");
    let mut at = price("2024-12-31", "2024-25", "750000.00");
    at["asset"] = json!("company_title_interest");
    assert_eq!(payable(&at), "93750.00");
}

/// A notice's maximum sale price is compared with the sale price, not the
/// first element of the cost base net of an input tax credit: $1.1 million
/// GST inclusive and a $100,000 credit. Below the base the notice fails on
/// either reading (15% of $1 million); at or above the sale price it applies
/// on either (5%); in between the answer turns on the maximum's GST basis,
/// which the ATO does not state, so it is refused.
#[test]
fn variation_maximum_reads_the_sale_price() {
    let mut input = price("2026-05-01", "2025-26", "1000000.00");
    input["sale_price"] = json!("1100000.00");
    let notice = |maximum: &str| json!({"interest": "1", "document": "variation", "rate": "0.05", "maximum_price": maximum});
    input["vendors"] = json!([notice("999999.99")]);
    assert_eq!(payable(&input), "150000.00");
    input["vendors"] = json!([notice("1100000.00")]);
    assert_eq!(payable(&input), "50000.00");
    for maximum in ["1000000.00", "1050000.00", "1099999.99"] {
        input["vendors"] = json!([notice(maximum)]);
        assert_eq!(
            code(&input),
            "foreign_resident_withholding.variation_maximum_unsettled"
        );
    }
    // An excluded transaction is nil whatever the notice says.
    input["vendors"] = json!([notice("1050000.00")]);
    input["exclusion"] = json!("bankruptcy");
    assert_eq!(payable(&input), "0.00");
}

/// A negative interest is refused, as every stated amount is.
#[test]
fn refuses_a_negative_interest() {
    let input = with(json!({"vendors": [{"interest": "-1", "document": "none"}]}));
    assert_eq!(code(&input), "input.amount_invalid");
}

/// Unequal interests 1:2 at $100.06 and 15%: 5.003 and 10.006 round down to
/// 5.00 and 10.00; the cent the single 15.01 leaves goes to the larger
/// remainder, the second vendor.
#[test]
fn the_rounding_cent_goes_to_the_largest_remainder() {
    let mut input = price("2025-03-01", "2024-25", "100.06");
    input["vendors"] = json!([
        {"interest": "1", "document": "none"},
        {"interest": "2", "document": "none"},
    ]);
    let r = run(&input);
    assert_eq!(r["vendors"][0]["amount"], "5.00");
    assert_eq!(r["vendors"][1]["amount"], "10.01");
    assert_eq!(r["amounts"]["amount_payable"], "15.01");
}

/// Every vendor row adds up to the amount payable, with and without
/// documents, over a sweep of bases and interest splits.
#[test]
fn vendor_rows_always_add_up_to_the_amount_payable() {
    let cents = |v: &Value| -> i64 {
        let text = v.as_str().unwrap().replace('.', "");
        text.parse().unwrap()
    };
    let splits: [&[&str]; 5] = [
        &["1", "1"],
        &["1", "2"],
        &["1", "1", "1"],
        &["3", "5", "7", "11"],
        &["0.33", "0.33", "0.34", "1", "2", "9"],
    ];
    for base in (0..400).map(|i| format!("{}.{:02}", 1000 + i * 7, (i * 37) % 100)) {
        for split in splits {
            for mixed in [false, true] {
                let mut input = price("2026-01-15", "2025-26", &base);
                let vendors: Vec<Value> = split
                    .iter()
                    .enumerate()
                    .map(|(i, interest)| {
                        let document = if mixed && i == 0 {
                            "clearance_certificate"
                        } else {
                            "none"
                        };
                        json!({"interest": interest, "document": document})
                    })
                    .collect();
                input["vendors"] = json!(vendors);
                let r = run(&input);
                let rows: i64 = r["vendors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| cents(&v["amount"]))
                    .sum();
                assert_eq!(
                    rows,
                    cents(&r["amounts"]["amount_payable"]),
                    "{base} {split:?} {mixed}"
                );
            }
        }
    }
}

/// The wording the round 2 review asked for: the authority calls rounding
/// the worksheet's convention, a company title interest is valued alone and
/// option renewals and extensions count.
#[test]
fn inputs_and_authorities_state_the_rules() {
    let r = run(&base());
    assert!(
        r["authorities"]["amount_payable"]
            .as_str()
            .unwrap()
            .contains("The law sets no rounding")
    );
    let units = australian_tax_calculators::metadata::input_units("foreign_resident_withholding");
    let text = |name: &str| units.iter().find(|(key, _)| *key == name).unwrap().1;
    assert!(text("market_value").contains("a company title interest is the shares, valued alone"));
    assert!(text("option_payments").contains("or to renew or extend it"));
    assert!(text("sale_price").contains("GST inclusive"));
}

#[test]
fn a_vendor_that_is_not_an_object_is_refused() {
    let input = with(json!({"vendors": [none(), "Jane"]}));
    assert_eq!(code(&input), "foreign_resident_withholding.vendors_invalid");
}
