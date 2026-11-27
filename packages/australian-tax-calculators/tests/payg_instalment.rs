// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! One quarterly PAYG instalment (Schedule 1 to the TAA 1953, Division 45,
//! compilations 219 and 227, read on 22 and 23 November 2026) against the ATO's
//! examples on 'How to vary your PAYG instalments' and 'PAYG instalments: how
//! to complete your activity statement', the site's 11 PAYG instalment cases
//! (`_data/calculator_cases.json`, checked 15 November 2026) and hand-worked
//! cases at each boundary (docs/calculation-evidence.md#payg-instalment).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("payg-instalment", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("payg-instalment", input)
        .unwrap_err()
        .code
}

/// The method's facts with the year and scope.
fn facts(method: &str, given: Value) -> Value {
    let mut out = json!({"year": "2026-27", "scope_confirmed": true, "method": method});
    for (key, value) in given.as_object().unwrap() {
        out[key] = value.clone();
    }
    out
}

fn rate(income: &str, t2: &str) -> Value {
    facts(
        "rate",
        json!({"instalment_income": income, "instalment_rate": t2}),
    )
}

fn varied_amount(number: i64, tax: &str, earlier: &str, credits: &str) -> Value {
    facts(
        "varied_amount",
        json!({"instalment_number": number, "estimated_tax": tax,
               "earlier_instalments": earlier, "credits_entitled": credits}),
    )
}

/// T1, the previous quarter's rate, estimated tax and income, earlier
/// instalments, credits and earlier income.
fn varied_rate(figures: [&str; 7]) -> Value {
    let [
        income,
        previous,
        tax,
        year_income,
        earlier,
        credits,
        earlier_income,
    ] = figures;
    facts(
        "varied_rate",
        json!({"instalment_income": income, "previous_rate": previous, "estimated_tax": tax,
               "estimated_instalment_income": year_income, "earlier_instalments": earlier,
               "credits_claimed": credits, "earlier_instalment_income": earlier_income}),
    )
}

fn amounts(result: &Value) -> Vec<&str> {
    let a = &result["amounts"];
    ["due", "amount", "instalment", "credit"]
        .iter()
        .map(|k| a[*k].as_str().unwrap_or("-"))
        .collect()
}

fn applied(result: &Value) -> &str {
    result["rates"]["instalment_rate"].as_str().unwrap()
}

/// The ATO's Sophia: $20,100 at 1.7% is $341.70, entered as 341 at T11 and 5A.
#[test]
fn sophia_t1_times_t2_drops_the_cents() {
    let out = run(&rate("20100", "1.7"));
    assert_eq!(amounts(&out), ["-", "341.70", "341.00", "0.00"]);
    assert_eq!(applied(&out), "1.70");
    assert_eq!(out["method"], "rate");
    assert_eq!(out["calculation"], "payg_instalment");
}

#[test]
fn the_site_rate_case() {
    let out = run(&rate("106000", "11"));
    assert_eq!(amounts(&out), ["-", "11660.00", "11660.00", "0.00"]);
    assert_eq!(applied(&out), "11.00");
}

#[test]
fn the_notified_amount_stands() {
    let out = run(&facts("amount", json!({"notified_amount": "5000"})));
    assert_eq!(amounts(&out), ["-", "5000.00", "5000.00", "0.00"]);
    let cents = run(&facts("amount", json!({"notified_amount": "5000.99"})));
    assert_eq!(amounts(&cents), ["-", "5000.99", "5000.00", "0.00"]);
}

/// The ATO's Cari (example 1): $14,000 x 75% less $10,000, then the rest.
#[test]
fn cari_varies_the_amount_in_the_third_and_fourth_quarters() {
    let third = run(&varied_amount(3, "14000", "10000", "0"));
    assert_eq!(amounts(&third), ["10500.00", "500.00", "500.00", "0.00"]);
    assert_eq!(third["rates"]["share_of_estimated_tax"], "75%");
    let fourth = run(&varied_amount(4, "14000", "10500", "0"));
    assert_eq!(amounts(&fourth), ["14000.00", "3500.00", "3500.00", "0.00"]);
}

#[test]
fn a_varied_amount_below_nil_is_a_credit_and_is_added_back_later() {
    let third = run(&varied_amount(3, "8000", "10000", "0"));
    assert_eq!(amounts(&third), ["6000.00", "-4000.00", "0.00", "4000.00"]);
    let fourth = run(&varied_amount(4, "8000", "10000", "4000"));
    assert_eq!(amounts(&fourth), ["8000.00", "2000.00", "2000.00", "0.00"]);
}

#[test]
fn each_quarter_takes_its_share_unrounded() {
    // $1,000.02 x 25% = $250.005, shown as $250.00; the instalment drops the cents.
    let first = run(&varied_amount(1, "1000.02", "0", "0"));
    assert_eq!(amounts(&first), ["250.00", "250.00", "250.00", "0.00"]);
    let second = run(&varied_amount(2, "1000.02", "250", "0"));
    assert_eq!(amounts(&second), ["500.01", "250.01", "250.00", "0.00"]);
    // A credit of $0.99 drops to nil at 5B.
    let small = run(&varied_amount(3, "1000", "750.99", "0"));
    assert_eq!(amounts(&small), ["750.00", "-0.99", "0.00", "0.00"]);
}

/// The ATO's Harmander (example 2): $10,125 / $82,480 is 12.2757..%, which
/// the ATO gives as 12.27%; the site adds his quarterly figures.
#[test]
fn harmander_varied_rate_is_cut_to_2_places_with_a_credit() {
    let out = run(&varied_rate([
        "20620", "16.84", "10125", "82480", "6944", "0", "41240",
    ]));
    assert_eq!(applied(&out), "12.27");
    assert_eq!(out["rates"]["previous_rate"], "16.84");
    assert!(out["rates"].get("t2_rate").is_none());
    // $41,240 x 12.27% = $5,060.148; $6,944 less that is $1,883.852.
    assert_eq!(amounts(&out), ["-", "2530.07", "2530.00", "1883.00"]);
}

/// The ATO's Bill: 10% then 5%, with a $500 credit at 5B.
#[test]
fn bill_claims_a_credit_at_the_lower_rate() {
    let out = run(&varied_rate([
        "10000", "10", "2000", "40000", "1000", "0", "10000",
    ]));
    assert_eq!(applied(&out), "5.00");
    assert_eq!(amounts(&out), ["-", "500.00", "500.00", "500.00"]);
}

#[test]
fn a_rate_no_lower_than_the_previous_rate_gives_no_credit() {
    let higher = run(&varied_rate([
        "10000", "10", "1200", "10000", "2000", "0", "10000",
    ]));
    assert_eq!(applied(&higher), "12.00");
    assert_eq!(amounts(&higher), ["-", "1200.00", "1200.00", "0.00"]);
    let equal = run(&varied_rate([
        "10000", "10", "1000", "10000", "5000", "0", "10000",
    ]));
    assert_eq!(amounts(&equal), ["-", "1000.00", "1000.00", "0.00"]);
}

#[test]
fn earlier_credits_reduce_a_lower_rate_credit() {
    // $3,000 less $500 claimed, less $20,000 x 5% = $1,500 credit.
    let out = run(&varied_rate([
        "10000", "10", "2000", "40000", "3000", "500", "20000",
    ]));
    assert_eq!(amounts(&out), ["-", "500.00", "500.00", "1500.00"]);
}

#[test]
fn nil_estimated_income_varies_the_rate_to_nil_and_all_income_to_100() {
    let nil = run(&varied_rate(["5000", "8", "0", "0", "0", "0", "0"]));
    assert_eq!(applied(&nil), "0.00");
    assert_eq!(amounts(&nil), ["-", "0.00", "0.00", "0.00"]);
    let full = run(&varied_rate(["5000", "8", "100", "100", "0", "0", "0"]));
    assert_eq!(applied(&full), "100.00");
    assert_eq!(amounts(&full), ["-", "5000.00", "5000.00", "0.00"]);
}

#[test]
fn the_rate_amount_is_shown_to_the_cent_toward_nil() {
    // $0.50 x 1% = $0.005, shown as $0.00.
    assert_eq!(
        amounts(&run(&rate("0.50", "1"))),
        ["-", "0.00", "0.00", "0.00"]
    );
    assert_eq!(
        amounts(&run(&rate("1000000", "100"))),
        ["-", "1000000.00", "1000000.00", "0.00"]
    );
}

#[test]
fn the_steps_cite_the_division() {
    let out = run(&varied_amount(3, "14000", "10000", "0"));
    let steps = out["steps"].as_array().unwrap();
    assert!(
        steps
            .iter()
            .any(|s| s["authority"].as_str().unwrap().contains("s 45-410"))
    );
    assert!(
        steps
            .iter()
            .any(|s| s["authority"].as_str().unwrap().contains("s 45-420"))
    );
    let rated = run(&varied_rate([
        "10000", "10", "2000", "40000", "1000", "0", "10000",
    ]));
    let ids: Vec<&str> = rated["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["step"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "varied_rate",
            "earlier_income_at_varied_rate",
            "credit",
            "amount",
            "instalment"
        ]
    );
}

#[test]
fn the_site_refusals_hold() {
    assert_eq!(
        code(&varied_amount(1, "8000", "1000", "0")),
        "payg_instalment.first_has_earlier"
    );
    assert_eq!(
        code(&varied_amount(4, "8000", "1000", "1000.01")),
        "payg_instalment.credits_exceed_earlier"
    );
    assert_eq!(
        code(&varied_amount(1, "8000", "0", "1")),
        "payg_instalment.credits_exceed_earlier"
    );
    assert_eq!(
        code(&varied_rate(["1", "10", "100.01", "100", "0", "0", "0"])),
        "payg_instalment.tax_exceeds_income"
    );
    assert_eq!(
        code(&varied_rate(["1", "10", "1", "100", "10", "10.01", "0"])),
        "payg_instalment.credits_exceed_earlier"
    );
    assert_eq!(
        code(&facts("monthly", json!({}))),
        "payg_instalment.method_invalid"
    );
    for number in [json!(0), json!(5), json!("3"), json!(2.0)] {
        let mut input = varied_amount(3, "1", "0", "0");
        input["instalment_number"] = number;
        assert_eq!(code(&input), "payg_instalment.instalment_number_invalid");
    }
    for t2 in ["100.01", "1.234", "-1"] {
        assert_eq!(code(&rate("1", t2)), "payg_instalment.rate_invalid");
        let previous = varied_rate(["1", t2, "1", "100", "0", "0", "0"]);
        assert_eq!(code(&previous), "payg_instalment.rate_invalid");
    }
}

/// No credit can exist before the second instalment: s 45-410(2) never
/// gives a negative first amount, and s 45-215(1)(b) needs an earlier quarter.
#[test]
fn credits_before_the_third_instalment_are_refused() {
    assert_eq!(
        code(&varied_amount(2, "8000", "2000", "1")),
        "payg_instalment.credits_before_third"
    );
    assert!(api::evaluate("payg-instalment", &varied_amount(3, "8000", "2000", "1")).is_ok());
}

#[test]
fn each_method_takes_only_its_own_facts() {
    let mut extra = rate("1", "1");
    extra["notified_amount"] = json!("1");
    assert_eq!(code(&extra), "input.field_unknown");
    let mut missing = varied_amount(3, "1", "0", "0");
    missing.as_object_mut().unwrap().remove("credits_entitled");
    assert_eq!(code(&missing), "input.missing_field");
    let mut unknown = rate("1", "1");
    unknown["quarter"] = json!(1);
    assert_eq!(code(&unknown), "input.field_unknown");
    // The varied rate compares with the previous quarter's rate, not T2.
    let mut t2 = varied_rate(["1", "1", "1", "100", "0", "0", "0"]);
    t2["instalment_rate"] = json!("1");
    assert_eq!(code(&t2), "input.field_unknown");
    let mut previous = rate("1", "1");
    previous["previous_rate"] = json!("1");
    assert_eq!(code(&previous), "input.field_unknown");
    assert_eq!(code(&rate("-1", "1")), "input.amount_invalid");
}

#[test]
fn only_2025_26_and_2026_27_are_supported() {
    for (year, ok) in [
        ("2024-25", false),
        ("2025-26", true),
        ("2026-27", true),
        ("2027-28", false),
    ] {
        let mut input = rate("20100", "1.7");
        input["year"] = json!(year);
        let result = api::evaluate_coded("payg-instalment", &input);
        assert_eq!(result.is_ok(), ok, "{year}");
        if !ok {
            assert_eq!(result.unwrap_err().code, "period.unsupported");
        }
    }
    let mut unconfirmed = rate("1", "1");
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(code(&unconfirmed), "scope.not_confirmed");
}

/// s 45-410(5) step 2 adds back each s 45-420 credit the payer was entitled
/// to claim, claimed at 5B or not: Q3 was nil with a $4,000 credit, so Q4 is
/// $8,000 less $10,000 plus $4,000.
#[test]
fn an_unclaimed_credit_is_still_added_back() {
    let fourth = run(&varied_amount(4, "8000", "10000", "4000"));
    assert_eq!(amounts(&fourth), ["8000.00", "2000.00", "2000.00", "0.00"]);
    let mut claimed = varied_amount(4, "8000", "10000", "4000");
    let credits = claimed
        .as_object_mut()
        .unwrap()
        .remove("credits_entitled")
        .unwrap();
    claimed["credits_claimed"] = credits;
    assert_eq!(code(&claimed), "input.field_unknown");
}

/// ATO, How to vary, step 3: nil estimated instalment income varies the rate
/// to nil without an estimate of tax.
#[test]
fn nil_estimated_income_needs_no_tax_estimate() {
    let nil = run(&varied_rate(["5000", "8", "100", "0", "0", "0", "0"]));
    assert_eq!(applied(&nil), "0.00");
    assert_eq!(amounts(&nil), ["-", "0.00", "0.00", "0.00"]);
}

/// Alcuin r2 S1: $2 less $10,001 x 0.01% ($1.0001) is $0.9999, which drops to
/// a nil credit; rounding the product to $1.00 first gave a $1 credit.
#[test]
fn the_lower_rate_credit_rounds_only_the_result() {
    let out = run(&varied_rate(["1", "0.02", "1", "6000", "2", "0", "10001"]));
    assert_eq!(applied(&out), "0.01");
    assert_eq!(amounts(&out)[3], "0.00");
    let steps = out["steps"].as_array().unwrap();
    assert_eq!(steps[1]["step"], "earlier_income_at_varied_rate");
    assert_eq!(steps[1]["value"], "1.0001");
}

/// Alcuin r2 G1: s 45-215(1)(b) compares the varied rate with the rate used
/// for the previous quarter. Q1 at 12% on $20,000 and Q2 at 10% on $10,000 make
/// $3,400; the Commissioner's new T2 is 14% and the payer varies to 11%.
#[test]
fn the_credit_compares_with_the_previous_quarters_rate() {
    // 11% is not lower than Q2's 10%: no credit, though $30,000 x 11% is $3,300.
    let out = run(&varied_rate([
        "10000", "10", "1100", "10000", "3400", "0", "30000",
    ]));
    assert_eq!(applied(&out), "11.00");
    assert_eq!(out["rates"]["previous_rate"], "10.00");
    assert_eq!(amounts(&out), ["-", "1100.00", "1100.00", "0.00"]);
    // Had Q2 used 12%, 11% is lower: $3,400 less $3,300.
    let lower = run(&varied_rate([
        "10000", "12", "1100", "10000", "3400", "0", "30000",
    ]));
    assert_eq!(amounts(&lower), ["-", "1100.00", "1100.00", "100.00"]);
}

/// No step rounds up into a whole dollar: 25% of $399.99 is $99.9975 and
/// $3,333.33 x 3% is $99.9999, each an instalment of $99.
#[test]
fn no_step_rounds_up_into_the_next_dollar() {
    let share = run(&varied_amount(1, "399.99", "0", "0"));
    assert_eq!(amounts(&share), ["99.99", "99.99", "99.00", "0.00"]);
    let product = run(&rate("3333.33", "3"));
    assert_eq!(amounts(&product), ["-", "99.99", "99.00", "0.00"]);
}

/// Alcuin r2 I1: a negative second instalment is a $1,000 credit, added back
/// in the third and fourth (s 45-410(4) and (5)).
#[test]
fn a_negative_second_instalment_is_added_back_in_the_third() {
    let second = run(&varied_amount(2, "8000", "5000", "0"));
    assert_eq!(amounts(&second), ["4000.00", "-1000.00", "0.00", "1000.00"]);
    let third = run(&varied_amount(3, "8000", "5000", "1000"));
    assert_eq!(amounts(&third), ["6000.00", "2000.00", "2000.00", "0.00"]);
    let fourth = run(&varied_amount(4, "8000", "7000", "1000"));
    assert_eq!(amounts(&fourth), ["8000.00", "2000.00", "2000.00", "0.00"]);
}

/// Alcuin r3 I4: a negative sub-cent amount shows toward nil and its credit
/// drops the cents: 50% of $200.01 is $100.005, less $200 is -$99.995.
#[test]
fn a_negative_sub_cent_amount_shows_toward_nil() {
    let out = run(&varied_amount(2, "200.01", "200", "0"));
    assert_eq!(amounts(&out), ["100.00", "-99.99", "0.00", "99.00"]);
}

/// Alcuin r2 I2: nil estimated income gives a nil rate, so the credit is the
/// earlier instalments less credits claimed: $3,000 less $500.
#[test]
fn a_nil_rate_credits_the_earlier_instalments_net_of_credits() {
    let out = run(&varied_rate([
        "5000", "8", "100", "0", "3000", "500", "20000",
    ]));
    assert_eq!(applied(&out), "0.00");
    assert_eq!(amounts(&out), ["-", "0.00", "0.00", "2500.00"]);
}

/// An instalment rate given as a JSON number or a boolean, not a decimal
/// string, is refused as an invalid rate.
#[test]
fn a_rate_that_is_not_a_decimal_is_refused() {
    for t2 in [json!(true), json!(["1.7"])] {
        let mut input = rate("20100", "1.7");
        input["instalment_rate"] = t2;
        assert_eq!(code(&input), "payg_instalment.rate_invalid");
    }
}
