// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The hire purchase worksheet beyond level monthly instalments: other
//! frequencies and dated segments (ITAA 1997 s 240-70), a deposit and
//! trade-in (s 240-25(3)), an early payout (Subdivision 240-G), journal lines
//! and the hirer's input tax credit (GST Act ss 9-75, 11-30 and 69-10), with
//! cases worked by hand at exactly 10% a month or week and the ATO's own
//! examples (docs/calculation-evidence.md#hire-purchase-schedule).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// $1,000 financed, 2 instalments of $100 a month in arrears and a $1,000
/// balloon, from 15 July 2026: exactly 10% a month. A `null` override drops
/// the input.
fn run(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "amount_financed": "1000.00", "instalment": "100.00", "instalments": 2,
        "balloon": "1000.00", "timing": "in_arrears", "start_date": "2026-07-15",
        "year": "2026-27", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            input.as_object_mut().unwrap().remove(k);
        } else {
            input[k] = v.clone();
        }
    }
    let result = api::evaluate("hire-purchase-schedule", &input);
    // derive gives the same result, or the same refusal.
    let envelope = json!({"command": "hire-purchase-schedule", "input": input});
    let doc = api::evaluate("derive", &envelope);
    assert_eq!(
        doc.as_ref().map(|doc| doc["result"].to_string()),
        result.as_ref().map(Value::to_string),
        "derive {input}"
    );
    result
}

fn code(overrides: Value) -> &'static str {
    let mut input = json!({
        "amount_financed": "1000.00", "instalment": "100.00", "instalments": 2,
        "balloon": "1000.00", "timing": "in_arrears", "start_date": "2026-07-15",
        "year": "2026-27", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            input.as_object_mut().unwrap().remove(k);
        } else {
            input[k] = v.clone();
        }
    }
    api::evaluate_coded("hire-purchase-schedule", &input)
        .unwrap_err()
        .code
}

fn interest(result: &Value) -> Vec<&str> {
    result["schedule"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["interest"].as_str().unwrap())
        .collect()
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap_or("absent")
}

/// An entry's lines as `account debit credit` strings.
fn lines(entry: &Value) -> Vec<String> {
    entry["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            format!(
                "{} {} {}",
                l["account"].as_str().unwrap(),
                l["debit"].as_str().unwrap(),
                l["credit"].as_str().unwrap()
            )
        })
        .collect()
}

fn segment(start: &str, count: i64, amount: &str, frequency: &str) -> Value {
    json!({"start": start, "count": count, "amount": amount, "frequency": frequency})
}

/// A quarter at 10% a month is 3 months compounded: 1,000 x 1.1^3 = 1,331,
/// so $331 a quarter. The rate stays a monthly one.
#[test]
fn a_quarterly_period_compounds_the_monthly_rate_over_its_3_months() {
    let r = run(json!({"instalment": "331.00", "frequency": "quarterly"})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.1000000000000000");
    assert_eq!(interest(&r), ["331.00", "331.00"]);
    assert_eq!(r["schedule"][0]["period_end"], "2026-10-15");
    assert_eq!(r["schedule"][1]["closing"], "0.00");
    assert_eq!(r["balloon_date"], "2027-01-15");
}

/// Checks the check: 3 months of simple interest at 10% would be $300, which
/// a quarter's compound interest is not.
#[test]
#[should_panic(expected = "331.00")]
fn a_wrong_quarterly_interest_fails() {
    let r = run(json!({"instalment": "331.00", "frequency": "quarterly"})).unwrap();
    assert_eq!(interest(&r)[0], "300.00", "{}", interest(&r)[0]);
}

/// Weekly at 10% a week, and fortnightly as 2 weeks: 1,000 x 1.1^2 = 1,210.
#[test]
fn weekly_and_fortnightly_periods_count_in_weeks() {
    let r = run(json!({"frequency": "weekly"})).unwrap();
    assert_eq!(r["rates"]["implicit_weekly_rate"], "0.1000000000000000");
    assert_eq!(r["rates"]["nominal_annual_rate"], "5.2000000000000000");
    assert!(r["rates"].get("implicit_monthly_rate").is_none());
    assert_eq!(interest(&r), ["100.00", "100.00"]);
    assert_eq!(r["schedule"][1]["period_start"], "2026-07-22");
    assert_eq!(r["balloon_date"], "2026-07-29");
    let r = run(
        json!({"instalment": "1210.00", "instalments": 1, "balloon": "0.00",
                       "frequency": "fortnightly"}),
    )
    .unwrap();
    assert_eq!(r["rates"]["implicit_weekly_rate"], "0.1000000000000000");
    assert_eq!(interest(&r), ["210.00"]);
    assert_eq!(r["schedule"][0]["period_end"], "2026-07-29");
}

/// An annual period is 2 arrangement payment periods of 6 months (s
/// 240-70(2)). At 50% a half-year, $1,000 grows to $1,500 and then $2,250, so
/// the halves carry $500 and $750, and only the second lists the payment.
#[test]
fn an_annual_period_is_2_periods_of_6_months() {
    let r = run(
        json!({"instalment": "2250.00", "instalments": 1, "balloon": "0.00",
                       "frequency": "annual"}),
    )
    .unwrap();
    assert_eq!(interest(&r), ["500.00", "750.00"]);
    let rows = r["schedule"].as_array().unwrap();
    assert_eq!(rows[0]["period_end"], "2027-01-15");
    assert_eq!(rows[0]["payment"], "0.00");
    assert_eq!(rows[0]["payment_date"], Value::Null);
    assert_eq!(rows[1]["payment_date"], "2027-07-15");
    assert_eq!(rows[1]["closing"], "0.00");
}

/// A 2-month payment holiday at 10% a month: $100 then $110 is added, and
/// $1,331 at the end of month 3 clears the $1,210 and its $121.
#[test]
fn a_nil_segment_is_a_payment_holiday() {
    let r = run(
        json!({"instalment": null, "instalments": null, "balloon": "0.00",
        "segments": [segment("2026-07-15", 2, "0.00", "monthly"),
                     segment("2026-09-15", 1, "1331.00", "monthly")]}),
    )
    .unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.1000000000000000");
    assert_eq!(interest(&r), ["100.00", "110.00", "121.00"]);
    assert_eq!(r["schedule"][0]["payment_date"], Value::Null);
    assert_eq!(r["schedule"][1]["closing"], "1210.00");
    assert_eq!(r["schedule"][2]["payment_date"], "2026-10-15");
    assert_eq!(amount(&r, "total_payments"), "1331.00");
}

/// A month then a quarter: $100 of interest paid by the first $100, then a
/// quarter's $331 on the $1,000.
#[test]
fn segments_run_on_from_each_other_at_their_own_frequency() {
    let segments = [
        segment("2026-07-15", 1, "100.00", "monthly"),
        segment("2026-08-15", 1, "1331.00", "quarterly"),
    ];
    let r = run(
        json!({"instalment": null, "instalments": null, "balloon": "0.00",
                       "segments": segments}),
    )
    .unwrap();
    assert_eq!(interest(&r), ["100.00", "331.00"]);
    assert_eq!(r["schedule"][1]["period_end"], "2026-11-15");
    let input = json!({"amount_financed": "1000.00", "balloon": "0.00", "timing": "in_arrears",
        "start_date": "2026-07-15", "year": "2026-27", "scope_confirmed": true,
        "segments": segments});
    let doc = api::evaluate(
        "derive",
        &json!({"command": "hire-purchase-schedule", "input": input}),
    )
    .unwrap();
    let total = doc["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "total_payments")
        .unwrap();
    assert_eq!(
        total["working"],
        "100.00 x 1 + 1331.00 x 1 + balloon = 100.00 x 1 + 1331.00 x 1 + 0.00 = 1431.00"
    );
}

/// A deposit and a trade-in sit beside the notional loan principal (s
/// 240-25(3)): the asset is booked at the consideration, $1,350, and the
/// liability at the $1,000 financed.
#[test]
fn a_deposit_and_trade_in_add_to_the_consideration_not_the_loan() {
    let r = run(json!({"deposit": "100.00", "trade_in": "250.00"})).unwrap();
    assert_eq!(amount(&r, "consideration"), "1350.00");
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.1000000000000000");
    let j = &r["journals"];
    assert_eq!(
        lines(&j["start"]),
        [
            "hire_purchase_asset 1350.00 0.00",
            "hire_purchase_liability 0.00 1000.00",
            "trade_in_allowance 0.00 250.00",
            "bank 0.00 100.00",
        ]
    );
    assert_eq!(
        lines(&j["periods"][0]),
        [
            "hire_purchase_liability 0.00 100.00",
            "interest_expense 100.00 0.00"
        ]
    );
    assert_eq!(
        lines(&j["payments"][1]),
        ["hire_purchase_liability 1100.00 0.00", "bank 0.00 1100.00"]
    );
    assert_eq!(
        lines(&j["by_income_year"][0]),
        [
            "hire_purchase_asset 1350.00 0.00",
            "hire_purchase_liability 1200.00 1200.00",
            "interest_expense 200.00 0.00",
            "trade_in_allowance 0.00 250.00",
            "bank 0.00 1300.00",
        ]
    );
    assert_eq!(r["gst"], json!({"computed": false, "missing": ["gst"]}));
}

/// Paid out at the end of month 2 of 3: $200 of interest, $100 paid, so
/// $1,100 outstanding. $1,150 exceeds it by $50 (s 240-105(2), s 240-110(1));
/// $1,050 falls $50 short (s 240-105(3), s 240-110(2)); $1,100 settles it
/// with no adjustment (s 240-105). Each says the payout itself is not
/// deductible where it acquires the property (s 240-85(b)).
#[test]
fn a_payout_settles_the_balance_and_its_difference_under_subdivision_240_g() {
    let base = json!({"instalments": 3, "payout": {"date": "2026-09-15", "amount": "1150.00", "known_at_start": false}});
    let r = run(base.clone()).unwrap();
    assert_eq!(interest(&r), ["100.00", "100.00"]);
    assert_eq!(r["schedule"][1]["payment"], "0.00");
    assert_eq!(r["schedule"][1]["closing"], "1100.00");
    for (key, value) in [
        ("total_payments", "1300.00"),
        ("total_interest", "300.00"),
        ("payout", "1150.00"),
        ("interest_to_payout", "200.00"),
        ("payments_before_payout", "100.00"),
        ("balance_at_payout", "1100.00"),
        ("payout_excess", "50.00"),
        ("payout_shortfall", "0.00"),
    ] {
        assert_eq!(amount(&r, key), value, "{key}");
    }
    let says = r["payout"]["consequence"].as_str().unwrap();
    assert!(
        says.contains("s 240-110(1)") && says.contains("s 240-85(b)"),
        "{says}"
    );
    let rows: Vec<&str> = interest(&r);
    assert_eq!(rows, ["100.00", "100.00"]);
    // A payout known at the start would have changed the implicit rate
    // (s 240-60(1) step 3 and (3)), which this worksheet does not estimate.
    let mut known = base.clone();
    known["payout"]["known_at_start"] = json!(true);
    assert_eq!(code(known), "hire_purchase.payout_known_at_start");
    let mut unstated = base.clone();
    unstated["payout"]
        .as_object_mut()
        .unwrap()
        .remove("known_at_start");
    assert_eq!(code(unstated), "hire_purchase.payout_invalid");
    assert!(
        r["payout"]["review"]
            .as_str()
            .unwrap()
            .starts_with("For review")
    );
    assert_eq!(
        lines(&r["journals"]["payout"]),
        [
            "hire_purchase_liability 1100.00 0.00",
            "payout_difference 50.00 0.00",
            "bank 0.00 1150.00",
        ]
    );
    let r = run(json!({"instalments": 3, "payout": {"date": "2026-09-15", "amount": "1050.00", "known_at_start": false}}))
        .unwrap();
    assert_eq!(amount(&r, "payout_shortfall"), "50.00");
    let says = r["payout"]["consequence"].as_str().unwrap();
    assert!(
        says.contains("s 240-110(2)") && says.contains("s 240-85(b)"),
        "{says}"
    );
    assert_eq!(
        lines(&r["journals"]["payout"])[1],
        "payout_difference 0.00 50.00"
    );
    let r = run(json!({"instalments": 3, "payout": {"date": "2026-09-15", "amount": "1100.00", "known_at_start": false}}))
        .unwrap();
    assert_eq!(amount(&r, "payout_excess"), "0.00");
    assert_eq!(amount(&r, "payout_shortfall"), "0.00");
    let says = r["payout"]["consequence"].as_str().unwrap();
    assert!(
        says.contains("s 240-105)") && says.contains("s 240-85(b)"),
        "{says}"
    );
}

/// GST on every component (the ATO's 'GST – Hire purchase and leasing'):
/// $1,100 price and $200 interest, so $1,300 and a $118.18 credit, $18.18 of
/// it on the interest. The asset is booked net of the $100.00 on the price.
#[test]
fn a_registered_hirer_s_credit_covers_the_price_and_the_interest() {
    let gst = json!({"registered": true, "accounting_basis": "accruals",
                     "creditable_purpose": "1", "asset": "other"});
    let r = run(json!({"deposit": "100.00", "gst": gst})).unwrap();
    for (key, value) in [
        ("total_consideration", "1300.00"),
        ("gst_payable", "118.18"),
        ("full_input_tax_credit", "118.18"),
        ("input_tax_credit", "118.18"),
        ("credit_on_interest", "18.18"),
    ] {
        assert_eq!(amount(&r, key), value, "{key}");
    }
    let j = &r["journals"];
    assert_eq!(j["gst_booked"], true);
    assert_eq!(
        lines(&j["start"]),
        [
            "hire_purchase_asset 1000.00 0.00",
            "gst_input_tax_credit 118.18 0.00",
            "hire_purchase_liability 0.00 1000.00",
            "deferred_interest_gst 0.00 18.18",
            "bank 0.00 100.00",
        ]
    );
    assert_eq!(
        lines(&j["periods"][0]),
        [
            "hire_purchase_liability 0.00 100.00",
            "interest_expense 90.91 0.00",
            "deferred_interest_gst 9.09 0.00",
        ]
    );
    assert_eq!(
        lines(&j["periods"][1])[2],
        "deferred_interest_gst 9.09 0.00"
    );
    assert_eq!(
        lines(&j["payments"][0]),
        ["hire_purchase_liability 100.00 0.00", "bank 0.00 100.00"]
    );
    assert_eq!(r["gst"]["first_consideration_date"], "2026-07-15");
    let attributable = r["gst"]["attributable"].as_str().unwrap();
    assert!(attributable.contains("s 29-10(1)") && !attributable.contains("158-5"));
    // On a cash basis the credit is still attributable up front (s 158-5).
    let gst = json!({"registered": true, "accounting_basis": "cash",
                     "creditable_purpose": "0.5", "asset": "other"});
    let r = run(json!({"gst": gst})).unwrap();
    assert!(r["gst"]["attributable"].as_str().unwrap().contains("158-5"));
    assert_eq!(r["gst"]["first_consideration_date"], "2026-08-15");
    // 1,200 / 11 x 0.5 = 54.55; the interest's share 200 / 11 x 0.5 = 9.09.
    assert_eq!(amount(&r, "input_tax_credit"), "54.55");
    assert_eq!(amount(&r, "credit_on_interest"), "9.09");
    assert_eq!(
        lines(&r["journals"]["start"])[0],
        "hire_purchase_asset 954.54 0.00"
    );
}

/// Albert's second freezer ('GST – Hire purchase and leasing'): $670 a month
/// for 60 months on a $33,000 freezer, so $40,200 in all and 1/11 of it as
/// the credit. The ATO prints $3,654.54; GST Act s 9-90(1) rounds to the
/// nearest cent, 0.5 up, which is $3,654.55. The full credit is that rounded
/// GST (s 11-25), so at 50% it is $1,827.275, $1,827.28, where 50% of the
/// unrounded $3,654.5454... would give $1,827.27.
#[test]
fn albert_s_second_freezer() {
    let gst = |purpose: &str| {
        json!({"registered": true, "accounting_basis": "cash",
                     "creditable_purpose": purpose, "asset": "other"})
    };
    let facts = json!({"amount_financed": "33000.00", "instalment": "670.00", "instalments": 60,
                       "balloon": "0.00", "start_date": "2026-07-20", "gst": gst("1")});
    let r = run(facts.clone()).unwrap();
    assert_eq!(amount(&r, "total_consideration"), "40200.00");
    assert_eq!(amount(&r, "input_tax_credit"), "3654.55");
    let mut half = facts;
    half["gst"] = gst("0.5");
    assert_eq!(amount(&run(half).unwrap(), "input_tax_credit"), "1827.28");
}

/// Darren's car ('Purchasing a motor vehicle'): $91,242.80, worth more than
/// the $69,883 car limit for 2026-27 and not quoted for, so s 69-10(1) caps
/// the credit on the car at 1/11 of that limit. Without an invoice that
/// establishes the split, the worksheet gives no GST figure.
#[test]
fn a_car_over_the_car_limit_gives_no_figure_and_is_flagged_for_review() {
    let gst = json!({"registered": true, "accounting_basis": "accruals",
                     "creditable_purpose": "1", "asset": "passenger_car",
                     "first_used_year": "2026-27", "gst_inclusive_market_value": "91242.80",
                     "luxury_car_tax": "2407.80", "quotes_abn_for_luxury_car_tax": false});
    let r = run(
        json!({"amount_financed": "91242.80", "instalment": "1800.00", "instalments": 60,
               "balloon": "0.00", "start_date": "2026-07-12", "gst": gst}),
    )
    .unwrap();
    for key in ["gst_payable", "full_input_tax_credit", "input_tax_credit"] {
        assert_eq!(amount(&r, key), "absent", "{key}");
    }
    assert_eq!(r["gst"]["computed"], false);
    assert!(r["gst"]["review"].as_str().unwrap().contains("s 69-10(1)"));
    assert_eq!(r["journals"]["gst_booked"], false);
}

#[test]
fn a_capped_hire_purchase_caps_only_the_verified_goods_credit() {
    let mut input = json!({
        "amount_financed": "91242.80", "instalment": "1800.00", "instalments": 60,
        "balloon": "0.00", "start_date": "2026-07-12",
        "gst": {"registered": true, "accounting_basis": "accruals",
                "creditable_purpose": "1", "asset": "passenger_car",
                "first_used_year": "2026-27", "gst_inclusive_market_value": "91242.80",
                "luxury_car_tax": "2407.80", "quotes_abn_for_luxury_car_tax": false,
                "separate_tax_invoice": {"tax_invoice_held": true,
                    "goods_consideration": "91242.80", "finance_consideration": "16757.20",
                    "goods_gst": "8075.91", "finance_gst": "1523.38"}}
    });
    let r = run(input.clone()).unwrap();
    // A $6,353 goods cap plus the separate $1,523.38 finance GST.
    assert_eq!(amount(&r, "gst_payable"), "9599.29");
    assert_eq!(amount(&r, "full_input_tax_credit"), "7876.38");
    assert_eq!(amount(&r, "input_tax_credit"), "7876.38");
    assert!(r["gst"].get("attributable").is_none());
    assert_eq!(r["journals"]["gst_booked"], false);

    input["gst"]["creditable_purpose"] = json!("0.5");
    assert_eq!(
        amount(&run(input.clone()).unwrap(), "input_tax_credit"),
        "3938.19"
    );
    input["gst"]["separate_tax_invoice"]["finance_gst"] = json!("0.00");
    assert!(run(input.clone()).unwrap_err().contains("do not reconcile"));
    input["gst"]["separate_tax_invoice"]["tax_invoice_held"] = json!(false);
    assert_eq!(run(input.clone()).unwrap()["gst"]["computed"], false);
    input["gst"]["separate_tax_invoice"]["tax_invoice_held"] = json!(true);
    input["gst"]["separate_tax_invoice"]
        .as_object_mut()
        .unwrap()
        .remove("finance_gst");
    assert_eq!(run(input).unwrap()["gst"]["computed"], false);
}

#[test]
fn a_capped_hire_purchase_uses_the_lesser_goods_credit_and_accepts_nil_finance() {
    let mut input = json!({
        "amount_financed": "60000.00", "instalment": "1300.00", "instalments": 60,
        "balloon": "0.00", "start_date": "2026-07-12",
        "gst": {"registered": true, "accounting_basis": "accruals",
                "creditable_purpose": "1", "asset": "passenger_car",
                "first_used_year": "2026-27", "gst_inclusive_market_value": "81000.00",
                "luxury_car_tax": "0.00", "quotes_abn_for_luxury_car_tax": false,
                "separate_tax_invoice": {"tax_invoice_held": true,
                    "goods_consideration": "60000.00", "finance_consideration": "18000.00",
                    "goods_gst": "5454.55", "finance_gst": "1636.36"}}
    });
    let result = run(input.clone()).unwrap();
    assert_eq!(amount(&result, "full_input_tax_credit"), "7090.91");
    let mut invalid = input.clone();
    invalid["gst"]["luxury_car_tax"] = json!("60000.01");
    invalid["gst"]["separate_tax_invoice"]["goods_gst"] = json!("0.00");
    assert!(
        run(invalid)
            .unwrap_err()
            .contains("luxury_car_tax cannot exceed the price")
    );

    input["amount_financed"] = json!("78000.00");
    input["gst"]["separate_tax_invoice"] = json!({
        "tax_invoice_held": true, "goods_consideration": "78000.00",
        "finance_consideration": "0.00", "goods_gst": "7090.91", "finance_gst": "0.00"
    });
    let result = run(input).unwrap();
    assert_eq!(amount(&result, "gst_payable"), "7090.91");
    assert_eq!(amount(&result, "full_input_tax_credit"), "6353.00");
}

/// s 69-10(1)(b) and (c): no cap for a car worth no more than the car limit
/// for the year it is first used, or for a hirer that may quote an ABN for
/// luxury car tax; a car the cap reaches is flagged for review.
#[test]
fn the_car_limit_reaches_only_a_car_worth_more_and_not_quoted_for() {
    let car = |value: &str, quotes: bool| {
        json!({"registered": true, "accounting_basis": "accruals", "creditable_purpose": "1",
               "asset": "passenger_car", "first_used_year": "2026-27",
               "gst_inclusive_market_value": value, "luxury_car_tax": "0.00",
               "quotes_abn_for_luxury_car_tax": quotes})
    };
    let facts = |gst: Value| {
        json!({"amount_financed": "60000.00", "instalment": "1300.00", "instalments": 60,
               "balloon": "0.00", "gst": gst})
    };
    // 78,000 / 11 = 7,090.91, above the $6,353 cap.
    let r = run(facts(car("60000.00", false))).unwrap();
    assert_eq!(amount(&r, "input_tax_credit"), "7090.91");
    assert_eq!(r["journals"]["gst_booked"], true);
    let r = run(facts(car("80000.00", true))).unwrap();
    assert_eq!(amount(&r, "input_tax_credit"), "7090.91");
    assert_eq!(r["rates"]["car_limit"], "69883");
    let r = run(facts(car("80000.00", false))).unwrap();
    assert_eq!(amount(&r, "input_tax_credit"), "absent");
    assert_eq!(r["gst"]["computed"], false);
    // $69,700 is under 2026-27's $69,883 but over 2024-25's $69,674, from
    // 'Assets and exclusions'.
    let r = run(facts(car("69700.00", false))).unwrap();
    assert_eq!(amount(&r, "input_tax_credit"), "7090.91");
    let mut early = facts(car("69700.00", false));
    early["gst"]["first_used_year"] = json!("2024-25");
    let r = run(early).unwrap();
    assert_eq!(amount(&r, "input_tax_credit"), "absent");
    assert!(r["gst"]["review"].as_str().is_some());
}

/// Each payment is journalled on the day it is paid and in that day's income
/// year, apart from the period's interest: in arrears from 1 June 2027 the
/// first instalment, paid on 1 July, is a 2027-28 payment, though June's
/// interest is 2026-27's; in advance it is paid on 1 June, in 2026-27.
#[test]
fn a_payment_is_journalled_on_its_day_and_in_its_income_year() {
    let r = run(json!({"start_date": "2027-06-01"})).unwrap();
    let j = &r["journals"];
    assert_eq!(j["periods"][0]["date"], "2027-07-01");
    assert_eq!(j["payments"][0]["date"], "2027-07-01");
    assert_eq!(
        lines(&j["by_income_year"][0]),
        [
            "hire_purchase_asset 1000.00 0.00",
            "hire_purchase_liability 0.00 1100.00",
            "interest_expense 100.00 0.00",
        ]
    );
    assert_eq!(
        lines(&j["by_income_year"][1]),
        [
            "hire_purchase_liability 1200.00 100.00",
            "interest_expense 100.00 0.00",
            "bank 0.00 1200.00",
        ]
    );
    let r = run(json!({"start_date": "2027-06-01", "timing": "in_advance"})).unwrap();
    let paid: Vec<(&str, &str)> = r["journals"]["payments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["date"].as_str().unwrap(),
                p["lines"][1]["credit"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        paid,
        [
            ("2027-06-01", "100.00"),
            ("2027-07-01", "100.00"),
            ("2027-08-01", "1000.00")
        ]
    );
    assert_eq!(
        lines(&r["journals"]["by_income_year"][0]).last().unwrap(),
        "bank 0.00 100.00"
    );
}

/// Without every fact a figure turns on, there is no GST figure, and the
/// facts missing are named; an unregistered hirer has a nil credit.
#[test]
fn missing_gst_facts_give_no_figure_and_name_what_is_missing() {
    let r = run(json!({"gst": {"registered": true}})).unwrap();
    assert_eq!(
        r["gst"],
        json!({"computed": false, "missing": ["accounting_basis", "creditable_purpose", "asset"]})
    );
    assert_eq!(amount(&r, "input_tax_credit"), "absent");
    let r = run(
        json!({"gst": {"registered": true, "accounting_basis": "cash",
                               "creditable_purpose": "1", "asset": "passenger_car",
                               "luxury_car_tax": "0.00"}}),
    )
    .unwrap();
    assert_eq!(
        r["gst"]["missing"],
        json!([
            "first_used_year",
            "gst_inclusive_market_value",
            "quotes_abn_for_luxury_car_tax"
        ])
    );
    let r = run(json!({"gst": {"registered": false}})).unwrap();
    assert_eq!(r["gst"], json!({"computed": true, "registered": false}));
    assert_eq!(amount(&r, "input_tax_credit"), "0.00");
    assert_eq!(r["journals"]["gst_booked"], false);
    let r = run(
        json!({"gst": {"registered": true, "accounting_basis": "cash",
                               "creditable_purpose": "0", "asset": "other"}}),
    )
    .unwrap();
    assert_eq!(amount(&r, "input_tax_credit"), "0.00");
    assert_eq!(r["journals"]["gst_booked"], false);
}

#[test]
fn an_empty_gst_object_names_registered_as_missing() {
    let r = run(json!({"gst": {}})).unwrap();
    assert_eq!(r["gst"], json!({"computed": false, "missing": ["registered"]}));
    assert_eq!(amount(&r, "input_tax_credit"), "absent");
}

#[test]
fn malformed_gst_facts_are_refused_with_what_they_must_be() {
    let refused = |overrides: Value| {
        let mut input = json!({
            "amount_financed": "1000.00", "instalment": "100.00", "instalments": 2,
            "balloon": "1000.00", "timing": "in_arrears", "start_date": "2026-07-15",
            "year": "2026-27", "scope_confirmed": true,
        });
        for (k, v) in overrides.as_object().unwrap() {
            input[k] = v.clone();
        }
        let error = api::evaluate_coded("hire-purchase-schedule", &input).unwrap_err();
        (error.code, error.message)
    };
    // Darren's car: capped by the car limit, so a split invoice may be given.
    let darren = |changes: Value| {
        let mut gst = json!({"registered": true, "accounting_basis": "accruals",
                             "creditable_purpose": "1", "asset": "passenger_car",
                             "first_used_year": "2026-27", "gst_inclusive_market_value": "91242.80",
                             "luxury_car_tax": "2407.80", "quotes_abn_for_luxury_car_tax": false});
        for (k, v) in changes.as_object().unwrap() {
            gst[k] = v.clone();
        }
        json!({"amount_financed": "91242.80", "instalment": "1800.00", "instalments": 60,
               "balloon": "0.00", "start_date": "2026-07-12", "gst": gst})
    };
    let invalid = "hire_purchase.gst_fact_invalid";
    let cases = [
        (
            json!({"gst": {"registered": "yes"}}),
            "input.flag_invalid",
            "gst: registered must be a boolean.",
        ),
        (
            json!({"gst": {"registered": true, "lease": true}}),
            "input.field_unknown",
            "gst: lease is not a GST fact this worksheet takes",
        ),
        (
            darren(json!({"first_used_year": 2026})),
            invalid,
            "gst: first_used_year must be an income year such as 2026-27.",
        ),
        (
            darren(json!({"luxury_car_tax": "lots"})),
            "input.amount_invalid",
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            darren(json!({"separate_tax_invoice": "held"})),
            invalid,
            "gst: separate_tax_invoice must be an object.",
        ),
        (
            darren(json!({"separate_tax_invoice": {"tax_invoice_held": true, "goods_price": "1.00"}})),
            invalid,
            "gst: separate_tax_invoice does not take goods_price.",
        ),
        (
            json!({"gst": {"registered": true, "accounting_basis": "cash", "creditable_purpose": "1",
                           "asset": "other", "separate_tax_invoice": {"tax_invoice_held": true}}}),
            invalid,
            "gst: separate_tax_invoice applies only to a passenger car reached by the hire purchase car limit.",
        ),
    ];
    for (overrides, code, message) in cases {
        assert_eq!(refused(overrides.clone()), (code, message.to_string()), "{overrides}");
    }
}

/// A week spanning 30 June is set apart in the interest and the journals.
#[test]
fn a_period_spanning_30_june_is_set_apart_in_the_journals_too() {
    let r = run(json!({"frequency": "weekly", "start_date": "2027-06-25"})).unwrap();
    let across = &r["interest_across_30_june"][0];
    assert_eq!(across["instalment"], 1);
    assert_eq!(across["days"], json!([6, 1]));
    let j = &r["journals"];
    assert_eq!(j["across_30_june"][0]["instalment"], 1);
    assert_eq!(j["by_income_year"][0]["income_year"], "2026-27");
    assert_eq!(j["by_income_year"][1]["income_year"], "2027-28");
    // Week 1's payment, on 2 July, is 2027-28's.
    assert_eq!(
        lines(&j["by_income_year"][1]),
        [
            "hire_purchase_liability 1200.00 100.00",
            "interest_expense 100.00 0.00",
            "bank 0.00 1200.00",
        ]
    );
}

/// The rate shown, discounting every payment back to the start, gives the
/// amount financed: 260 weekly payments of $230 and a $5,000 balloon on
/// $50,000.
#[test]
fn the_solved_rate_discounts_the_payments_to_the_amount_financed() {
    let r = run(
        json!({"amount_financed": "50000.00", "instalment": "230.00",
                       "instalments": 260, "balloon": "5000.00", "frequency": "weekly"}),
    )
    .unwrap();
    let rate: f64 = r["rates"]["implicit_weekly_rate"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let pv: f64 = (1..=260).map(|k| 230.0 / (1.0 + rate).powi(k)).sum::<f64>()
        + 5000.0 / (1.0 + rate).powi(260);
    assert!((pv - 50000.0).abs() < 0.01, "{pv}");
    assert_eq!(r["schedule"].as_array().unwrap().len(), 260);
    assert_eq!(r["schedule"][259]["closing"], "0.00");
}

#[test]
fn each_new_input_s_faults_are_refused_with_their_code() {
    let level_free =
        |segments: Value| json!({"instalment": null, "instalments": null, "segments": segments});
    assert_eq!(
        code(json!({"frequency": "daily"})),
        "hire_purchase.frequency_invalid"
    );
    assert_eq!(
        code(level_free(json!([]))),
        "hire_purchase.segments_invalid"
    );
    assert_eq!(
        code(level_free(
            json!([{"start": "2026-07-15", "amount": "1.00", "frequency": "monthly"}])
        )),
        "hire_purchase.segments_invalid"
    );
    let mut extra = segment("2026-07-15", 1, "1100.00", "monthly");
    extra["note"] = json!("x");
    assert_eq!(code(level_free(json!([extra]))), "input.field_unknown");
    assert_eq!(
        code(level_free(json!([segment(
            "2026-07-16",
            1,
            "1100.00",
            "monthly"
        )]))),
        "hire_purchase.segments_not_contiguous"
    );
    assert_eq!(
        code(level_free(json!([
            segment("2026-07-15", 1, "100.00", "monthly"),
            segment("2026-09-15", 1, "1100.00", "monthly")
        ]))),
        "hire_purchase.segments_not_contiguous"
    );
    assert_eq!(
        code(level_free(json!([
            segment("2026-07-15", 1, "100.00", "monthly"),
            segment("2026-08-15", 1, "1100.00", "weekly")
        ]))),
        "hire_purchase.frequencies_mixed"
    );
    assert_eq!(
        code(level_free(json!([segment(
            "2026-07-15",
            31,
            "100.00",
            "annual"
        )]))),
        "hire_purchase.instalments_unsupported"
    );
    assert_eq!(
        code(level_free(json!([segment(
            "2026-07-15",
            2,
            "0.00",
            "monthly"
        )]))),
        "hire_purchase.amount_nil"
    );
    assert_eq!(
        code(json!({"segments": [segment("2026-07-15", 1, "1100.00", "monthly")]})),
        "input.field_unknown"
    );
    assert_eq!(code(json!({"instalment": null})), "input.missing_field");
    for (frequency, most) in [
        ("weekly", 1565),
        ("fortnightly", 782),
        ("quarterly", 120),
        ("annual", 30),
    ] {
        assert_eq!(
            code(json!({"frequency": frequency, "instalments": most + 1})),
            "hire_purchase.instalments_unsupported",
            "{frequency}"
        );
    }
    let payout = |date: &str| json!({"instalments": 3, "payout": {"date": date, "amount": "1.00", "known_at_start": false}});
    assert_eq!(
        code(payout("2026-08-20")),
        "hire_purchase.payout_date_unsupported"
    );
    assert_eq!(
        code(payout("2026-10-15")),
        "hire_purchase.payout_date_unsupported"
    );
    assert_eq!(
        code(payout("2026-07-15")),
        "hire_purchase.payout_date_unsupported"
    );
    assert_eq!(
        code(json!({"payout": {"date": "2026-08-15"}})),
        "hire_purchase.payout_invalid"
    );
    assert_eq!(
        code(
            json!({"payout": {"date": "2026-08-15", "amount": "1.00", "known_at_start": false}, "gst": {"registered": false}})
        ),
        "hire_purchase.payout_gst_unsupported"
    );
    assert_eq!(
        code(json!({"gst": "yes"})),
        "hire_purchase.gst_fact_invalid"
    );
    assert_eq!(
        code(json!({"gst": {"registered": true, "accounting_basis": "hybrid"}})),
        "hire_purchase.gst_fact_invalid"
    );
    assert_eq!(
        code(json!({"gst": {"registered": true, "creditable_purpose": "1.5"}})),
        "input.fraction_invalid"
    );
    assert_eq!(
        code(json!({"gst": {"registered": false, "asset": "other"}})),
        "input.field_unknown"
    );
    assert_eq!(
        code(json!({"gst": {"registered": true, "asset": "other", "luxury_car_tax": "0.00"}})),
        "input.field_unknown"
    );
    assert_eq!(
        code(json!({"gst": {"registered": true, "first_used_year": "2027-28"}})),
        "hire_purchase.car_limit_unknown"
    );
    // A count whose term overflows is refused, not multiplied out.
    assert_eq!(
        code(level_free(json!([segment(
            "2026-07-15",
            1_i64 << 62,
            "100.00",
            "annual"
        )]))),
        "hire_purchase.instalments_unsupported"
    );
    let gst = json!({"registered": true, "accounting_basis": "cash", "creditable_purpose": "1",
                     "asset": "passenger_car", "first_used_year": "2026-27",
                     "gst_inclusive_market_value": "1000.00", "luxury_car_tax": "1000.01",
                     "quotes_abn_for_luxury_car_tax": false});
    assert_eq!(code(json!({"gst": gst})), "hire_purchase.gst_fact_conflict");
}
