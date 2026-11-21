// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The chattel mortgage schedule: a stated or implicit rate, the schedule and
//! its printable table, interest by income year and the purchaser's input tax
//! credit on the price, worked by hand at 1% a month or week, against the
//! closed-form annuity payment and the ATO's Darren example
//! (docs/calculation-evidence.md#chattel-mortgage-schedule).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// $1,000 financed at 12% a year (1% a month), 3 monthly instalments in
/// arrears, no balloon, from 15 July 2026. A `null` override drops the input.
fn input(overrides: Value) -> Value {
    let mut input = json!({
        "amount_financed": "1000.00", "annual_rate": "0.12", "instalments": 3,
        "balloon": "0.00", "timing": "in_arrears", "start_date": "2026-07-15",
        "year": "2026-27", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            input.as_object_mut().unwrap().remove(k);
        } else {
            input[k] = v.clone();
        }
    }
    input
}

fn run(overrides: Value) -> Value {
    let input = input(overrides);
    let result = api::evaluate("chattel-mortgage-schedule", &input).unwrap();
    // derive gives the same result.
    let envelope = json!({"command": "chattel-mortgage-schedule", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded("chattel-mortgage-schedule", &input(overrides))
        .unwrap_err()
        .code
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

fn column<'a>(result: &'a Value, key: &str) -> Vec<&'a str> {
    result["schedule"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row[key].as_str().unwrap())
        .collect()
}

/// By hand: 1.01^3 = 1.030301, so the level instalment is
/// 10 x 1.030301 / 0.030301 = $340.0221..., $340.02. Month 1 charges $10.00
/// and leaves $669.98; month 2 $6.6998, leaving $336.6598; month 3 $3.366598,
/// leaving $0.006398 after $340.02, so the last instalment is $340.03. The
/// interest to date shows $10.00, $16.70 and $20.07.
#[test]
fn a_stated_rate_sets_the_instalment_and_the_last_absorbs_the_rounding() {
    let r = run(json!({}));
    for (key, value) in [
        ("amount_financed", "1000.00"),
        ("instalment", "340.02"),
        ("final_adjustment", "0.01"),
        ("total_payments", "1020.07"),
        ("total_interest", "20.07"),
    ] {
        assert_eq!(amount(&r, key), value, "{key}");
    }
    assert_eq!(column(&r, "interest"), ["10.00", "6.70", "3.37"]);
    assert_eq!(column(&r, "payment"), ["340.02", "340.02", "340.03"]);
    assert_eq!(column(&r, "principal"), ["330.02", "333.32", "336.66"]);
    assert_eq!(column(&r, "closing"), ["669.98", "336.66", "0.00"]);
    assert_eq!(r["rates"]["monthly_rate"], "0.0100000000000000");
    assert_eq!(r["rates"]["nominal_annual_rate"], "0.1200000000000000");
    let printed = r["statement"].as_str().unwrap();
    assert!(
        printed.starts_with("| Instalment | Period end |"),
        "{printed}"
    );
    assert!(
        printed.contains(
            "| 3 | 2026-10-15 | 2026-10-15 | 336.66 | 3.37 | 340.03 | 0.00 | 336.66 | 0.00 |"
        ),
        "{printed}"
    );
}

/// A balloon equal to the amount financed leaves an interest-only loan: at
/// 1% a month on $10,000 each instalment is the month's $100.00 of interest.
/// Weekly at 52% a year is 1% a week, so $10.00 on $1,000.
#[test]
fn an_interest_only_loan_pays_the_interest_and_the_balloon_repays_it() {
    let r = run(json!({"amount_financed": "10000.00", "balloon": "10000.00"}));
    assert_eq!(amount(&r, "instalment"), "100.00");
    assert_eq!(amount(&r, "final_adjustment"), "0.00");
    assert_eq!(column(&r, "interest"), ["100.00", "100.00", "100.00"]);
    assert_eq!(column(&r, "balloon"), ["0.00", "0.00", "10000.00"]);
    assert_eq!(r["balloon_date"], "2026-10-15");
    let r = run(
        json!({"annual_rate": "0.52", "frequency": "weekly", "instalments": 2,
                       "balloon": "1000.00"}),
    );
    assert_eq!(amount(&r, "instalment"), "10.00");
    assert_eq!(column(&r, "period_end"), ["2026-07-22", "2026-07-29"]);
    assert_eq!(r["rates"]["weekly_rate"], "0.0100000000000000");
}

/// The closed-form annuity payment, A = (P - B v^n) r / (1 - v^n) with
/// v = 1 / (1 + r), as a spreadsheet's PMT(0.0799/12, 60, -45000, 5000)
/// gives it, worked separately in exact fractions: $844.1560..., so $844.16,
/// with the balance left at the end -$0.29 and the interest $10,649.31. Given
/// that instalment instead, the rate the payments imply is 7.99% again.
#[test]
fn the_instalment_matches_the_closed_form_annuity_payment() {
    let facts = json!({"amount_financed": "45000.00", "annual_rate": "0.0799",
                       "instalments": 60, "balloon": "5000.00"});
    let r = run(facts);
    assert_eq!(amount(&r, "instalment"), "844.16");
    assert_eq!(amount(&r, "final_adjustment"), "-0.29");
    assert_eq!(amount(&r, "total_interest"), "10649.31");
    assert_eq!(column(&r, "payment").last(), Some(&"843.87"));
    let implicit = run(json!({"amount_financed": "45000.00", "annual_rate": null,
                              "instalment": "844.16", "instalments": 60, "balloon": "5000.00"}));
    let nominal: f64 = implicit["rates"]["nominal_annual_rate"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((nominal - 0.0799).abs() < 0.00001, "{nominal}");
    assert!(implicit["rates"].get("implicit_monthly_rate").is_some());
    // 104 weekly instalments of $20,000 at 6.5%: $205.1984..., so $205.20.
    let weekly = run(
        json!({"amount_financed": "20000.00", "annual_rate": "0.065",
                            "frequency": "weekly", "instalments": 104}),
    );
    assert_eq!(amount(&weekly, "instalment"), "205.20");
    assert_eq!(amount(&weekly, "final_adjustment"), "-0.17");
    assert_eq!(amount(&weekly, "total_interest"), "1340.63");
}

/// Given the instalment, the rate is found as hire-purchase-schedule finds
/// it: $100 twice and a $1,000 balloon on $1,000 is exactly 10% a month, in
/// arrears; in advance the first instalment pays no interest.
#[test]
fn an_instalment_given_implies_the_rate() {
    let r = run(
        json!({"annual_rate": null, "instalment": "100.00", "instalments": 2,
                       "balloon": "1000.00"}),
    );
    assert_eq!(column(&r, "interest"), ["100.00", "100.00"]);
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.1000000000000000");
    assert!(r["amounts"].get("instalment").is_none());
    let r = run(
        json!({"annual_rate": null, "instalment": "100.00", "instalments": 2,
                       "balloon": "979.00", "timing": "in_advance"}),
    );
    assert_eq!(column(&r, "interest"), ["90.00", "89.00"]);
}

/// 12 monthly instalments from 15 July 2026: periods 1 to 11 fall in
/// 2026-27, and period 12, 15 June to 15 July 2027, spans 30 June with 16
/// days before 1 July and 14 from it.
#[test]
fn interest_is_totalled_by_income_year_with_the_period_across_30_june_apart() {
    let r = run(json!({"amount_financed": "12000.00", "instalments": 12}));
    let years = r["interest_by_income_year"].as_array().unwrap();
    assert_eq!(years.len(), 1);
    assert_eq!(years[0]["income_year"], "2026-27");
    assert_eq!(years[0]["last_instalment"], 11);
    let across = &r["interest_across_30_june"][0];
    assert_eq!(across["instalment"], 12);
    assert_eq!(across["income_years"], json!(["2026-27", "2027-28"]));
    assert_eq!(across["days"], json!([16, 14]));
    let interest: Vec<f64> = column(&r, "interest")
        .iter()
        .map(|v| v.parse().unwrap())
        .collect();
    let year: f64 = years[0]["interest"].as_str().unwrap().parse().unwrap();
    let sum: f64 = interest[..11].iter().sum();
    assert!((year - sum).abs() < 0.001);
}

/// Darren's example ('Purchasing a motor vehicle'): a car for $91,242.80
/// including $8,075.91 GST and $2,407.80 luxury car tax, first used in
/// 2026-27 and worth more than the $69,883 car limit, so the credit is at most
/// 1/11 of it, $6,353, and $3,176.50 at 50%. Bought with a $10,000 deposit
/// and $81,242.80 lent, the loan's interest gives no credit.
#[test]
fn darren_s_car_credit_is_capped_at_one_eleventh_of_the_car_limit() {
    let gst = |purpose: &str, basis: &str| {
        json!({"registered": true, "accounting_basis": basis, "creditable_purpose": purpose,
               "asset": "passenger_car", "first_used_year": "2026-27",
               "gst_inclusive_market_value": "91242.80", "luxury_car_tax": "2407.80",
               "quotes_abn_for_luxury_car_tax": false})
    };
    let facts = |purpose: &str, basis: &str| {
        json!({"amount_financed": "81242.80", "deposit": "10000.00", "annual_rate": "0.0799",
               "instalments": 60, "gst": gst(purpose, basis)})
    };
    let r = run(facts("1", "accruals"));
    for (key, value) in [
        ("price", "91242.80"),
        ("gst_payable", "8075.91"),
        ("full_input_tax_credit", "6353.00"),
        ("input_tax_credit", "6353.00"),
    ] {
        assert_eq!(amount(&r, key), value, "{key}");
    }
    assert!(r["amounts"].get("credit_on_interest").is_none());
    assert_eq!(r["rates"]["car_limit"], "69883");
    let said = r["gst"]["attributable"].as_str().unwrap();
    assert!(
        said.contains("s 29-10(1)") && said.contains("issue 1.21"),
        "{said}"
    );
    let notes = r["gst"]["notes"].to_string();
    assert!(notes.contains("s 40-5(1)"), "{notes}");
    let half = run(facts("0.5", "cash"));
    assert_eq!(amount(&half, "input_tax_credit"), "3176.50");
    let said = half["gst"]["attributable"].as_str().unwrap();
    assert!(said.contains("s 29-10(2)(a)"), "{said}");
    assert_eq!(half["gst"]["purchase_date"], "2026-07-15");
    // Quoting an ABN, the cap does not apply (s 69-10(1)(b)).
    let mut quoting = facts("1", "accruals");
    quoting["gst"]["quotes_abn_for_luxury_car_tax"] = json!(true);
    assert_eq!(amount(&run(quoting), "input_tax_credit"), "8075.91");
}

/// Goods that are not a car: 1/11 of the $33,000 price, $3,000.00, and
/// $1,500.00 at 50%; nothing on the interest. Unregistered, the credit is nil.
#[test]
fn other_goods_give_one_eleventh_of_the_price() {
    let gst = json!({"registered": true, "accounting_basis": "accruals",
                     "creditable_purpose": "0.5", "asset": "other"});
    let r = run(json!({"amount_financed": "30000.00", "trade_in": "3000.00",
                       "instalments": 36, "gst": gst}));
    assert_eq!(amount(&r, "price"), "33000.00");
    assert_eq!(amount(&r, "gst_payable"), "3000.00");
    assert_eq!(amount(&r, "input_tax_credit"), "1500.00");
    let r = run(json!({"gst": {"registered": false}}));
    assert_eq!(amount(&r, "input_tax_credit"), "0.00");
    let r = run(json!({}));
    assert_eq!(r["gst"], json!({"computed": false, "missing": ["gst"]}));
}

#[test]
fn input_the_schedule_cannot_represent_is_refused() {
    let instalment = json!({"instalment": "340.02"});
    assert_eq!(code(instalment), "chattel_mortgage.terms_conflict");
    assert_eq!(
        code(json!({"annual_rate": null})),
        "chattel_mortgage.terms_conflict"
    );
    assert_eq!(
        code(json!({"timing": "in_advance"})),
        "chattel_mortgage.stated_rate_unsupported"
    );
    assert_eq!(
        code(json!({"frequency": "fortnightly"})),
        "chattel_mortgage.stated_rate_unsupported"
    );
    assert_eq!(
        code(json!({"instalments": 1, "balloon": "2000.00"})),
        "chattel_mortgage.balloon_too_large"
    );
    // $1.46 over 30 years at 1% a month: $0.0150... rounds up to $0.02,
    // overpaying by $17.41 by the end.
    assert_eq!(
        code(json!({"amount_financed": "1.46", "instalments": 360})),
        "chattel_mortgage.final_instalment_unsupported"
    );
    // $50,000 over 30 years at 50% a year: $2,083.3333... rounds down to
    // $2,083.33, and the 1/3 cent unpaid each month compounds to $242,950.50
    // by the end, more than a whole instalment.
    let compounded = json!({"amount_financed": "50000.00", "annual_rate": "0.50",
                            "instalments": 360});
    assert_eq!(
        code(compounded),
        "chattel_mortgage.final_instalment_unsupported"
    );
    // At 30% the $678.01 left is less than the $1,250.17 instalment.
    let r = run(json!({"amount_financed": "50000.00", "annual_rate": "0.30",
                       "instalments": 360}));
    assert_eq!(amount(&r, "final_adjustment"), "678.01");
    assert_eq!(
        code(json!({"annual_rate": "1.5"})),
        "input.fraction_invalid"
    );
    assert_eq!(
        code(json!({"amount_financed": "0.00"})),
        "hire_purchase.amount_nil"
    );
    assert_eq!(
        code(json!({"start_date": "2026-06-30"})),
        "hire_purchase.start_outside_year"
    );
    assert_eq!(
        code(json!({"instalments": 361})),
        "hire_purchase.instalments_unsupported"
    );
    assert_eq!(code(json!({"segments": []})), "input.field_unknown");
    assert_eq!(code(json!({"payout": {}})), "input.field_unknown");
    let car = json!({"registered": true, "accounting_basis": "cash", "creditable_purpose": "1",
                     "asset": "passenger_car", "first_used_year": "2027-28",
                     "gst_inclusive_market_value": "1000.00", "luxury_car_tax": "0.00",
                     "quotes_abn_for_luxury_car_tax": false});
    assert_eq!(code(json!({"gst": car})), "hire_purchase.car_limit_unknown");
}
