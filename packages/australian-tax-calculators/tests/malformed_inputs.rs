// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Malformed facts each command refuses with its own code: a wrong type, a
//! missing or unknown key, dates out of order and names used twice.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn code(command: &str, input: &Value) -> &'static str {
    match api::evaluate_coded(command, input) {
        Ok(_) => "accepted",
        Err(refusal) => refusal.code,
    }
}

/// Each case's code, so one run shows every mismatch.
fn codes(command: &str, cases: &[Value]) -> Vec<&'static str> {
    cases.iter().map(|c| code(command, c)).collect()
}

fn ato(question: &str, year: &str, facts: Value) -> Value {
    let mut out = facts;
    out["question"] = json!(question);
    out["year"] = json!(year);
    out["scope_confirmed"] = json!(true);
    out
}

fn gic(extra: Value) -> Value {
    let mut facts = json!({"amount": "100.00", "due_date": "2025-07-01", "to": "2025-07-10"});
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    ato("gic", "2025-26", facts)
}

fn overpaid(payments: Value, refunded: &str, on: &str, notice: &str) -> Value {
    json!({"payments": payments, "refunded": refunded, "refunded_on": on, "notice_issued": notice, "relevant_tax": true, "decision_to_which_act_applies": true})
}

fn early(extra: Value) -> Value {
    let mut facts = json!({"amount": "100.00", "liability": "income_tax", "paid_on": "2025-11-06", "due_date": "2025-11-21", "full_self_assessment": true});
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    ato("early_payment_interest", "2025-26", facts)
}

fn statement(parts: Value) -> Value {
    json!({"statement_date": "2026-03-15", "significant_global_entity": false, "parts": parts})
}

#[test]
fn ato_interest_and_penalties_refuse_malformed_facts() {
    let reckless = json!({"shortfall_amount": "100.00", "behaviour": "recklessness"});
    let mut stray_tax = statement(json!([reckless]));
    stray_tax["income_tax_payable"] = json!("1000.00");
    let cases = [
        gic(json!({"payments": "none"})),
        gic(json!({"payments": [1]})),
        gic(json!({"due_date": "1 July 2025"})),
        gic(json!({"due_date": "2025-07-10", "to": "2025-07-05"})),
        gic(json!({"assumed_annual_rate": "0.00"})),
        gic(json!({"assumed_annual_rate": "100.01"})),
        early(json!({"full_self_assessment": "yes"})),
        ato(
            "statement_penalty",
            "2025-26",
            statement(json!([{"shortfall_amount": "100.00", "behaviour": "carelessness"}])),
        ),
        ato("statement_penalty", "2025-26", statement(json!(["part"]))),
        ato("statement_penalty", "2025-26", stray_tax),
        ato(
            "overpayment_interest",
            "2024-25",
            overpaid(
                json!([{"paid_on": "2024-09-01", "amount": "100.00"}]),
                "100.00",
                "2024-10-01",
                "2024-10-15",
            ),
        ),
        ato(
            "overpayment_interest",
            "2026-27",
            overpaid(
                json!([{"paid_on": "2026-07-01", "amount": "100.00"}]),
                "100.00",
                "2027-02-01",
                "2026-07-15",
            ),
        ),
    ];
    assert_eq!(
        codes("ato-interest-penalties", &cases),
        [
            "ato_interest_penalties.fact_invalid",
            "ato_interest_penalties.fact_invalid",
            "ato_interest_penalties.fact_invalid",
            "ato_interest_penalties.date_order",
            "ato_interest_penalties.fact_invalid",
            "ato_interest_penalties.fact_invalid",
            "input.flag_invalid",
            "ato_interest_penalties.fact_invalid",
            "ato_interest_penalties.fact_invalid",
            "input.field_unknown",
            "ato_interest_penalties.date_order",
            "ato_interest_penalties.rate_unpublished",
        ]
    );
}

/// A refund smaller than the last instalment is attributed to it alone
/// (Overpayments Act s 9(3)); the earlier instalment earns nothing.
#[test]
fn a_refund_within_the_last_instalment_stops_there() {
    let two = json!([{"paid_on": "2024-12-01", "amount": "3000.00"}, {"paid_on": "2025-03-01", "amount": "3000.00"}]);
    let r = api::evaluate(
        "ato-interest-penalties",
        &ato(
            "overpayment_interest",
            "2025-26",
            overpaid(two, "2000.00", "2025-08-15", "2024-10-15"),
        ),
    )
    .unwrap();
    let parts = r["attribution"].as_array().unwrap();
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0]["paid_on"], "2025-03-01");
    assert_eq!(parts[0]["attributed"], "2000.00");
}

fn balancing(asset: &str, years: Value) -> Value {
    json!({
        "cost": "1000.00", "termination_value": "500.00", "asset": asset, "years": years,
        "year": "2025-26", "scope_confirmed": true,
    })
}

#[test]
fn balancing_adjustment_refuses_malformed_rows() {
    let row = json!({"income_year": "2025-26", "decline_in_value": "100.00", "deduction": "50.00"});
    let with = |key: &str, value: Value| {
        let mut r = row.clone();
        r[key] = value;
        r
    };
    let without = |key: &str| {
        let mut r = row.clone();
        r.as_object_mut().unwrap().remove(key);
        r
    };
    let car_row = |method: Value| json!({"income_year": "2025-26", "decline_in_value": "100.00", "method": method, "deduction": "50.00", "days_held": 365});
    let mut car_limit = balancing("car", json!([car_row(json!("log_book"))]));
    car_limit["car_limit"] = json!({"limit": "69674.00", "second_element_cost": "0.00", "cost_ignoring_car_limit": "1000.00", "colour": "red"});
    let mut car_days = car_row(json!("log_book"));
    car_days.as_object_mut().unwrap().remove("days_held");
    let car = |row: Value| {
        let mut facts = balancing("car", json!([row]));
        facts["car_limit"] = Value::Null;
        facts
    };
    let cases = [
        balancing("other", json!([with("decline_in_value", json!(100))])),
        balancing("other", json!([without("income_year")])),
        balancing("other", json!(["row"])),
        balancing("other", json!("rows")),
        car(car_row(json!("odometer"))),
        car(car_days),
        car_limit,
    ];
    assert_eq!(
        codes("balancing-adjustment", &cases),
        [
            "input.amount_invalid",
            "balancing.fact_missing",
            "balancing.years_invalid",
            "balancing.years_invalid",
            "balancing.fact_invalid",
            "balancing.fact_missing",
            "input.field_unknown",
        ]
    );
}

fn caps(extra: Value) -> Value {
    let mut facts = json!({
        "total_super_balance": "450000", "concessional_contributions": "97500",
        "non_concessional_contributions": "0", "under_75_in_year": true,
        "earlier_concessional_contributions": [{"year": "2019-20", "concessional_contributions": "11000"}],
        "defined_benefit_interest": false, "year": "2024-25", "scope_confirmed": true,
    });
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

#[test]
fn contribution_caps_by_year_refuses_malformed_years_and_flags() {
    let cases = [
        caps(json!({"earlier_concessional_contributions": [5]})),
        caps(
            json!({"earlier_concessional_contributions": [{"year": "2019-20", "concessional_contributions": "eleven"}]}),
        ),
        caps(json!({"under_75_in_year": "yes"})),
    ];
    assert_eq!(
        codes("contribution-caps", &cases),
        [
            "contribution_caps.earlier_years_invalid",
            "contribution_caps.earlier_years_invalid",
            "input.flag_invalid",
        ]
    );
}

const HEADER: &str = "contract,client,financier,kind,start_date,amount_financed,instalment,instalments,frequency,timing,balloon";

fn register(csv: &str) -> (i32, String) {
    let args = ["--balance-date", "2026-06-30", "--scope-confirmed"].map(String::from);
    let (status, _, err) = api::contract_register(&args, &mut csv.as_bytes());
    (status, err)
}

#[test]
fn the_contract_register_reads_a_quoted_last_field_and_refuses_text_after_a_quote() {
    let row = "HP-1,Client,Bank,hire_purchase,2025-08-01,24000.00,1000.00,24,monthly,in_arrears";
    let (status, err) = register(&format!("{HEADER}\n{row},\"0.00\"\n"));
    assert_eq!(status, 0, "{err}");
    let (status, err) = register(&format!("{HEADER}\n{row},\"0.00\"x\n"));
    assert_eq!(status, 2);
    assert!(err.contains("a quoted field must end at a comma or the end of the line"));
}

#[test]
fn the_contract_register_refuses_a_file_over_8_mib() {
    let big = format!("{HEADER}\n{}", "x".repeat(8 * 1024 * 1024));
    let (status, err) = register(&big);
    assert_eq!(status, 2);
    assert!(err.contains("the contracts file exceeds 8 MiB"));
}

/// `base` with `extra`'s fields added or replaced, a `null` dropping one.
fn merged(mut base: Value, extra: Value) -> Value {
    for (key, value) in extra.as_object().unwrap() {
        if value.is_null() {
            base.as_object_mut().unwrap().remove(key);
        } else {
            base[key] = value.clone();
        }
    }
    base
}

fn hire_purchase(extra: Value) -> Value {
    let base = json!({
        "amount_financed": "1000.00", "instalment": "100.00", "instalments": 2,
        "balloon": "1000.00", "timing": "in_arrears", "start_date": "2026-07-15",
        "year": "2026-27", "scope_confirmed": true,
    });
    merged(base, extra)
}

fn segment(start: Value, count: i64, amount: Value, frequency: &str) -> Value {
    json!({"start": start, "count": count, "amount": amount, "frequency": frequency})
}

#[test]
fn hire_purchase_refuses_malformed_segments_and_payouts() {
    let level = |segments: Value| {
        hire_purchase(
            json!({"instalment": null, "instalments": null, "balloon": "0.00", "segments": segments}),
        )
    };
    let one = |s: Value| level(json!([s]));
    let monthly = segment(json!("2026-07-15"), 1, json!("1100.00"), "monthly");
    let cases = [
        merged(level(json!([monthly])), json!({"amount_financed": "0.00"})),
        level(json!("monthly")),
        one(json!("segment")),
        one(segment(
            json!("15 July 2026"),
            1,
            json!("1100.00"),
            "monthly",
        )),
        one(segment(json!("2026-07-15"), 1, json!(1100), "monthly")),
        one(segment(json!("2026-07-15"), 1600, json!("10.00"), "weekly")),
        hire_purchase(json!({"instalments": 3, "payout": "2026-09-15"})),
        hire_purchase(
            json!({"instalments": 3, "payout": {"date": "2026-09-15", "amount": "1150.00", "known_at_start": false, "fee": "1.00"}}),
        ),
    ];
    assert_eq!(
        codes("hire-purchase-schedule", &cases),
        [
            "hire_purchase.amount_nil",
            "hire_purchase.segments_invalid",
            "hire_purchase.segments_invalid",
            "hire_purchase.segments_invalid",
            "hire_purchase.segments_invalid",
            "hire_purchase.instalments_unsupported",
            "hire_purchase.payout_invalid",
            "input.field_unknown",
        ]
    );
}

fn chattel(extra: Value) -> Value {
    let base = json!({
        "amount_financed": "1000.00", "annual_rate": "0.12", "instalments": 3,
        "balloon": "0.00", "timing": "in_arrears", "start_date": "2026-07-15",
        "year": "2026-27", "scope_confirmed": true,
    });
    merged(base, extra)
}

fn car_gst(purpose: &str, luxury_car_tax: &str) -> Value {
    json!({"registered": true, "accounting_basis": "accruals", "creditable_purpose": purpose,
           "asset": "passenger_car", "first_used_year": "2026-27",
           "gst_inclusive_market_value": "91242.80", "luxury_car_tax": luxury_car_tax,
           "quotes_abn_for_luxury_car_tax": false})
}

#[test]
fn a_chattel_mortgage_refuses_a_nil_instalment_and_lct_above_the_price() {
    let cases = [
        chattel(json!({"annual_rate": null, "instalment": "0.00"})),
        chattel(
            json!({"amount_financed": "81242.80", "deposit": "10000.00", "gst": car_gst("1", "100000.00")}),
        ),
    ];
    assert_eq!(
        codes("chattel-mortgage-schedule", &cases),
        [
            "hire_purchase.amount_nil",
            "hire_purchase.gst_fact_conflict"
        ]
    );
}

/// Goods used for no creditable purpose give no input tax credit on the
/// price, and missing GST facts leave the credit unworked rather than nil.
#[test]
fn a_chattel_mortgage_with_no_creditable_purpose_gives_no_credit() {
    let facts = chattel(
        json!({"amount_financed": "81242.80", "deposit": "10000.00", "gst": car_gst("0", "2407.80")}),
    );
    let r = api::evaluate("chattel-mortgage-schedule", &facts).unwrap();
    assert_eq!(r["amounts"]["input_tax_credit"], "0.00");
    let missing = chattel(json!({"gst": {"registered": true}}));
    let r = api::evaluate("chattel-mortgage-schedule", &missing).unwrap();
    assert!(r["amounts"].get("input_tax_credit").is_none());
}

fn loan(extra: Value) -> Value {
    let base = json!({
        "opening_balance": "0.00", "annual_rate": "0.073",
        "start_date": "2026-07-01", "end_date": "2026-09-30",
        "transactions": [{"date": "2026-07-01", "kind": "drawing", "amount": "10000.00"}],
        "year": "2026-27", "scope_confirmed": true,
    });
    merged(base, extra)
}

#[test]
fn loan_transactions_refuse_malformed_entries() {
    let cases = [
        loan(json!({"transactions": ["drawing"]})),
        loan(
            json!({"transactions": [{"date": "1 July 2026", "kind": "drawing", "amount": "1.00"}]}),
        ),
        loan(json!({"transactions": [{"date": "2026-07-01", "kind": "drawing", "amount": 1}]})),
        loan(json!({"rate_changes": [{"date": "2026-08-01", "annual_rate": 0.05}]})),
    ];
    assert_eq!(
        codes("loan-transactions", &cases),
        [
            "loan_transactions.entry_invalid",
            "loan_transactions.entry_invalid",
            "loan_transactions.entry_invalid",
            "loan_transactions.entry_invalid",
        ]
    );
}

#[test]
fn fuel_tax_credits_refuse_missing_and_malformed_line_facts() {
    let line = json!({"fuel": "diesel", "use": "other_business", "acquired": "2026-10-01", "quantity": "100"});
    let from_cost = json!({"method": "litres_from_cost", "fuel": "diesel", "use": "other_business",
                           "acquired": "2026-09-30", "cost": 23, "average_price_cents_per_litre": "245.9"});
    let fuel = |l: Value| json!({"lines": [l], "year": "2026-27", "scope_confirmed": true});
    let cases = [
        fuel(merged(line.clone(), json!({"fuel": null}))),
        fuel(merged(line.clone(), json!({"acquired": null}))),
        fuel(merged(line, json!({"method": "litres_from_receipts"}))),
        fuel(from_cost),
    ];
    assert_eq!(
        codes("fuel-tax-credits", &cases),
        [
            "input.missing_field",
            "input.missing_field",
            "fuel_tax_credits.method_invalid",
            "input.amount_invalid",
        ]
    );
}

#[test]
fn luxury_car_tax_refuses_a_flag_that_is_not_a_boolean() {
    let sale = json!({"kind": "sale", "luxury_car_tax_value": "100000.00", "fuel_consumption_litres_per_100km": "5.0", "used_before_1_july_2025": "yes"});
    let facts = json!({"lines": [sale], "year": "2025-26", "scope_confirmed": true});
    assert_eq!(code("luxury-car-tax", &facts), "input.flag_invalid");
}

#[test]
fn foreign_income_conversion_refuses_missing_and_malformed_facts() {
    let deduction = json!({
        "kind": "deduction", "currency": "NZD", "amount": "100.00", "rate": "0.9",
        "rate_source": "ATO example", "basis": "specific", "rate_date": "2025-09-01",
        "deductible": "2025-09-01", "paid": "2025-09-01",
    });
    let fic = |item: Value| json!({"year": "2025-26", "scope_confirmed": true, "items": [item]});
    let cases = [
        fic(merged(deduction.clone(), json!({"amount": null}))),
        fic(merged(deduction.clone(), json!({"amount": 100}))),
        fic(merged(deduction.clone(), json!({"deductible": null}))),
        fic(merged(deduction.clone(), json!({"rate_date": null}))),
        fic(merged(deduction, json!({"currency": null}))),
    ];
    assert_eq!(
        codes("foreign-income-conversion", &cases),
        [
            "input.missing_field",
            "input.amount_invalid",
            "input.missing_field",
            "input.missing_field",
            "input.missing_field",
        ]
    );
}

#[test]
fn impairment_and_consolidation_refuse_malformed_items() {
    let impairment = json!({"cash_flows": ["100.00"], "discount_rate": "0.1", "assets": ["plant"], "revalued": false,
                            "non_controlling_interest": false, "year": "2025-26", "scope_confirmed": true});
    assert_eq!(
        code("impairment-test", &impairment),
        "impairment_test.asset_invalid"
    );
    let consolidation = json!({"parent": "P Ltd", "subsidiaries": [], "intra_group_balances": [],
        "intra_group_transactions": [], "unrealised_profits": [], "tax_rate": "0.3",
        "acquired_in_period": false, "fair_value_adjustments": false,
        "previously_held_interest": false, "ownership_changed": false,
        "intra_group_dividends": false, "year": "2025-26", "scope_confirmed": true});
    assert_eq!(
        code("consolidation-eliminations", &consolidation),
        "consolidation_eliminations.item_invalid"
    );
}

#[test]
fn the_foreign_income_tax_offset_refuses_malformed_residency_periods() {
    let worked = json!({
        "year": "2025-26", "scope_confirmed": true, "foreign_tax_paid": "2000.00",
        "nane_foreign_tax": "0.00", "limit_method": "worked",
        "taxable_income": "30000", "disregarded_income": "10000.00",
        "disregarded_deductions": "0.00", "medicare_levy": "0.00", "medicare_levy_assumed": "0.00",
    });
    let cases = [
        merged(worked.clone(), json!({"residency_periods": "2025-26"})),
        merged(worked, json!({"residency_periods": ["2025-07-01"]})),
    ];
    assert_eq!(
        codes("foreign-income-tax-offset", &cases),
        [
            "foreign_income_tax_offset.residency_periods_invalid",
            "foreign_income_tax_offset.residency_periods_invalid",
        ]
    );
}

#[test]
fn fbt_worksheets_refuse_malformed_contributions_attendees_and_entries() {
    let station = |contribution: Value| {
        json!({"method": "commercial_parking_station", "first_business_day_fee": "18.00",
               "benefits": 230, "lowest_fee": "18.00", "employee_contribution": contribution,
               "year_ended": 2026, "scope_confirmed": true})
    };
    assert_eq!(
        codes(
            "fbt-car-parking",
            &[
                station(json!({"capped_total": "1000.00", "cap": "1"})),
                station(json!({"capped_total": "1000.0001"})),
            ]
        ),
        [
            "fbt_car_parking.capped_contribution_invalid",
            "fbt_car_parking.capped_contribution_invalid",
        ]
    );
    let line = json!({
        "cost": "1100.00", "gst_status": "includes_gst", "provided": "food_or_drink",
        "occasion": "social_function", "venue": "elsewhere", "paid_by": "employer",
        "attendees": {"employees": 10},
    });
    let classify = |l: Value| {
        json!({"lines": [l], "income_tax_exempt": false, "method": "actual",
               "year_ended": 2027, "scope_confirmed": true})
    };
    assert_eq!(
        codes(
            "fbt-entertainment-classifier",
            &[
                classify(merged(line.clone(), json!({"cost": "eleven"}))),
                classify(merged(line, json!({"attendees": 10}))),
            ]
        ),
        [
            "input.amount_invalid",
            "fbt_entertainment_classifier.fact_invalid",
        ]
    );
    let entry = json!({"date": "2025-05-14", "recipients": {"employees": 3, "associates": 0, "others": 2},
                       "cost": "500.00", "kind": "client dinner", "location": "restaurant",
                       "on_premises": true, "fringe_benefits_value": "300.00"});
    let register = |e: Value| {
        json!({"register_start": "2025-05-01", "register_end": "2025-07-23",
               "entries": [e], "year_ended": 2026, "scope_confirmed": true})
    };
    assert_eq!(
        codes(
            "fbt-meal-entertainment-register",
            &[
                register(merged(entry.clone(), json!({"kind": 5}))),
                register(merged(entry, json!({"in_house_dining_facility": "no"}))),
            ]
        ),
        [
            "fbt_meal_entertainment_register.entry_invalid",
            "fbt_meal_entertainment_register.entry_invalid",
        ]
    );
}
