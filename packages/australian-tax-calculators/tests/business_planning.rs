// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `break-even`, `cash-forecast`, `hourly-rate`, `fixed-fee` and `staff-cost`
//! agree with the site's calculators on the site's own vectors, read from
//! ryanduguid.github.io main on 26 November 2026: `_data/calculator_cases.json`
//! (each page's boundary cases and the break-even peer case),
//! `scripts/business-calculators.test.mjs` and `scripts/webmcp-tools.test.mjs`.
//! The super guarantee cases beyond the site use the ATO's 'Super guarantee'
//! tables 21, 23 and 24, read the same day.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(command: &str, input: &Value) -> Value {
    api::evaluate(command, input).unwrap()
}

fn code(command: &str, input: &Value) -> &'static str {
    api::evaluate_coded(command, input).unwrap_err().code
}

fn amount<'a>(out: &'a Value, key: &str) -> &'a str {
    out["amounts"][key].as_str().unwrap()
}

fn break_even(fixed: &str, price: &str, variable: &str) -> Value {
    json!({"fixed_costs": fixed, "price": price, "variable_cost": variable,
           "year": "2025-26", "scope_confirmed": true})
}

#[test]
fn break_even_matches_the_site() {
    // calculator_cases.json row 1 and business-calculators.test.mjs.
    let out = run("break-even", &break_even("1000", "30", "18"));
    assert_eq!(amount(&out, "contribution"), "12.00");
    assert_eq!(amount(&out, "units"), "84.00");
    assert_eq!(amount(&out, "sales"), "2520.00");
    assert_eq!(out["rates"]["whole_units"], "84");
    // The page's peer case: whole dollars that break even on a whole unit.
    let peer = run("break-even", &break_even("1000", "25", "15"));
    assert_eq!(amount(&peer, "units"), "100.00");
    assert_eq!(amount(&peer, "sales"), "2500.00");
    for (price, variable) in [("18", "18"), ("17", "18")] {
        let refused = break_even("1000", price, variable);
        assert_eq!(code("break-even", &refused), "break_even.no_contribution");
    }
    let huge = break_even("1000000000", "1000000000", "999999999.99");
    assert_eq!(code("break-even", &huge), "break_even.sales_out_of_range");
    assert_eq!(
        amount(&run("break-even", &break_even("0", "2", "1")), "units"),
        "0.00"
    );
}

fn hourly(costs: &str, profit: &str, hours: &str) -> Value {
    json!({"annual_costs": costs, "target_profit": profit, "billable_hours": hours,
           "year": "2025-26", "scope_confirmed": true})
}

#[test]
fn hourly_rate_matches_the_site() {
    for (costs, profit, hours, rate) in [
        ("20.15", "0", "2", "10.08"),
        ("1000000000", "1000000000", "0.01", "200000000000.00"),
        ("80000", "0", "1600", "50.00"),
        ("80000", "20000", "1000", "100.00"),
        ("100", "0", "3", "33.34"),
        ("100000", "0", "2999", "33.35"),
    ] {
        let out = run("hourly-rate", &hourly(costs, profit, hours));
        assert_eq!(
            amount(&out, "hourly_rate"),
            rate,
            "{costs} {profit} {hours}"
        );
    }
    for hours in ["0", "8784.01", "1.005"] {
        assert_eq!(
            code("hourly-rate", &hourly("1", "0", hours)),
            "pricing.hours_invalid"
        );
    }
    assert_eq!(
        code("hourly-rate", &hourly("100.005", "0", "10")),
        "input.amount_invalid"
    );
    let mut extra = hourly("20.15", "0", "2");
    extra["gst"] = json!(true);
    assert_eq!(code("hourly-rate", &extra), "input.field_unknown");
}

fn fee(hours: &str, rate: &str, margin: &str, step: &str, gst: bool, count: Value) -> Value {
    json!({"hours": hours, "rate": rate, "margin_percent": margin, "round_to": step,
           "add_gst": gst, "instalments": count, "year": "2025-26", "scope_confirmed": true})
}

fn fee_amounts(out: &Value) -> [&str; 6] {
    [
        "fee",
        "gst",
        "total",
        "first_instalment",
        "instalment",
        "effective_rate",
    ]
    .map(|key| amount(out, key))
}

#[test]
fn fixed_fee_matches_the_site() {
    for (input, expected) in [
        (
            fee("18.5", "95", "40", "10", true, json!(12)),
            ["2930.00", "293.00", "3223.00", "268.62", "268.58", "158.38"],
        ),
        (
            fee("12", "180", "0", "50", true, json!(12)),
            ["2200.00", "220.00", "2420.00", "201.74", "201.66", "183.33"],
        ),
        (
            fee("3", "100", "0", "0.01", true, json!(7)),
            ["300.00", "30.00", "330.00", "47.16", "47.14", "100.00"],
        ),
        (
            fee("1", "120", "45", "0.01", false, json!(1)),
            ["218.19", "0.00", "218.19", "218.19", "218.19", "218.19"],
        ),
    ] {
        assert_eq!(fee_amounts(&run("fixed-fee", &input)), expected, "{input}");
    }
    let fee_of = |rate: &str| {
        amount(
            &run("fixed-fee", &fee("10", rate, "0", "100", false, json!(1))),
            "fee",
        )
        .to_string()
    };
    assert_eq!(fee_of("100"), "1000.00");
    assert_eq!(fee_of("100.01"), "1100.00");
    let gst = run("fixed-fee", &fee("1", "120", "45", "0.01", true, json!(1)));
    assert_eq!(amount(&gst, "gst"), "21.82");
    let edge = run(
        "fixed-fee",
        &fee("1", "909090909.09", "0", "0.01", true, json!(1)),
    );
    assert_eq!(amount(&edge, "total"), "1000000000.00");
}

/// The site's instalment property: the instalments add up to the total, the
/// first carries under one cent per instalment more, and after any number of
/// months the client has paid at least that many months' share.
#[test]
fn fixed_fee_instalments_add_up_as_on_the_site() {
    for total in ["0.01", "1", "10.13", "330", "999.99", "3223", "1234567.89"] {
        for count in [1_i64, 2, 3, 7, 11, 12, 13, 23, 24] {
            let out = run(
                "fixed-fee",
                &fee("1", total, "0", "0.01", false, json!(count)),
            );
            let cents = |key: &str| -> i64 { amount(&out, key).replace('.', "").parse().unwrap() };
            let (whole, first, each) = (
                cents("total"),
                cents("first_instalment"),
                cents("instalment"),
            );
            assert_eq!(first + each * (count - 1), whole, "{total} over {count}");
            assert!(
                first - each >= 0 && first - each < count,
                "{total} over {count}"
            );
            for paid in 1..=count {
                assert!((first + each * (paid - 1)) * count >= whole * paid);
            }
        }
    }
}

#[test]
fn fixed_fee_refuses_as_the_site_does() {
    let refused = |input: Value| code("fixed-fee", &input);
    assert_eq!(
        refused(fee("10", "100", "100", "10", true, json!(12))),
        "pricing.margin_invalid"
    );
    assert_eq!(
        refused(fee("10", "100", "0", "5", true, json!(12))),
        "pricing.round_to_invalid"
    );
    for count in [json!(0), json!(25), json!(1.5), json!("12")] {
        assert_eq!(
            refused(fee("10", "100", "0", "10", true, count)),
            "input.count_invalid"
        );
    }
    assert_eq!(
        refused(fee("8784", "1000000", "99.99", "10", false, json!(1))),
        "pricing.fee_out_of_range"
    );
    assert_eq!(
        refused(fee("1", "909090909.1", "0", "0.01", true, json!(1))),
        "pricing.fee_out_of_range"
    );
    let mut not_flag = fee("1", "1", "0", "10", true, json!(1));
    not_flag["add_gst"] = json!("yes");
    assert_eq!(refused(not_flag), "input.flag_invalid");
}

fn staff(wages: &str, other: &str, year: &str) -> Value {
    json!({"wages": wages, "other_costs": other, "year": year, "scope_confirmed": true})
}

fn with(mut input: Value, key: &str, value: &str) -> Value {
    input[key] = json!(value);
    input
}

#[test]
fn staff_cost_matches_the_site() {
    for (wages, sup, other, annual, monthly) in [
        ("51.30", "0", "0", "51.30", "4.28"),
        ("80000", "9600", "1000", "90600.00", "7550.00"),
        ("80000", "9600", "4000", "93600.00", "7800.00"),
    ] {
        let out = run(
            "staff-cost",
            &with(staff(wages, other, "2025-26"), "super_amount", sup),
        );
        assert_eq!(amount(&out, "annual_cost"), annual);
        assert_eq!(amount(&out, "monthly_average"), monthly);
    }
    let negative = with(staff("-51.30", "0", "2025-26"), "super_amount", "0");
    assert_eq!(code("staff-cost", &negative), "input.amount_invalid");
}

#[test]
fn staff_cost_works_the_super_guarantee_and_stated_on_costs() {
    let sg = |base: &str, year: &str| {
        let out = run(
            "staff-cost",
            &with(staff("1", "0", year), "super_base", base),
        );
        amount(&out, "superannuation").to_string()
    };
    // ATO table 21's percentages; before 2026-27 a base up to one quarter's
    // maximum contribution base (table 24).
    assert_eq!(sg("10000", "2021-22"), "1000.00");
    assert_eq!(sg("10000", "2022-23"), "1050.00");
    assert_eq!(sg("10000", "2023-24"), "1100.00");
    assert_eq!(sg("10000.10", "2024-25"), "1150.01");
    assert_eq!(sg("62500", "2025-26"), "7500.00");
    // Table 23: the 2026-27 annual base of $270,830 caps qualifying earnings.
    assert_eq!(sg("270830", "2026-27"), "32499.60");
    assert_eq!(sg("300000", "2026-27"), "32499.60");
    // $200,000.00 earned as $100,000.00 in each of 2 quarters owes $15,000.00,
    // not 12% of the whole: the split decides it, so an annual base above one
    // quarter's cap is refused.
    for base in ["62500.01", "200000", "250000"] {
        let above = with(staff("1", "0", "2025-26"), "super_base", base);
        assert_eq!(
            code("staff-cost", &above),
            "staff_cost.base_above_quarterly_cap",
            "{base}"
        );
    }
    // Stated rates on wages plus super: $89,600 at 5.45% and 1.5%.
    let mut input = with(staff("80000", "1000", "2026-27"), "super_base", "80000");
    input = with(
        with(input, "payroll_tax_rate", "5.45"),
        "workers_compensation_rate",
        "1.5",
    );
    let out = run("staff-cost", &input);
    assert_eq!(amount(&out, "superannuation"), "9600.00");
    assert_eq!(amount(&out, "payroll_tax"), "4883.20");
    assert_eq!(amount(&out, "workers_compensation"), "1344.00");
    assert_eq!(amount(&out, "annual_cost"), "96827.20");
    assert_eq!(amount(&out, "monthly_average"), "8068.93");
    assert_eq!(out["rates"]["sg_rate"], "0.12");
}

#[test]
fn staff_cost_refuses_an_unclear_super_basis_and_bad_rates() {
    let neither = staff("1", "0", "2025-26");
    assert_eq!(
        code("staff-cost", &neither),
        "staff_cost.super_basis_invalid"
    );
    let both = with(
        with(neither.clone(), "super_amount", "1"),
        "super_base",
        "1",
    );
    assert_eq!(code("staff-cost", &both), "staff_cost.super_basis_invalid");
    let stated = with(neither.clone(), "super_amount", "0");
    for rate in ["100.01", "1.00001", "-1"] {
        let bad = with(stated.clone(), "payroll_tax_rate", rate);
        assert_eq!(
            code("staff-cost", &bad),
            "staff_cost.rate_invalid",
            "{rate}"
        );
    }
    let early = with(staff("1", "0", "2020-21"), "super_amount", "0");
    assert_eq!(code("staff-cost", &early), "period.unsupported");
}

fn weeks(receipts: &[(usize, &str)], payments: &str, fill: &str) -> Value {
    let mut rows: Vec<Value> = (0..13)
        .map(|_| json!({"receipts": fill, "payments": payments}))
        .collect();
    for (week, amount) in receipts {
        rows[*week]["receipts"] = json!(amount);
    }
    Value::Array(rows)
}

fn cash(opening: &str, rows: Value, buffer: &str, delayed: Option<(u64, &str, u64)>) -> Value {
    let mut input = json!({"opening": opening, "buffer": buffer, "weeks": rows,
                           "year": "2025-26", "scope_confirmed": true});
    if let Some((week, amount, delay)) = delayed {
        input["delayed_receipt"] = json!({"week": week, "amount": amount, "delay_weeks": delay});
    }
    input
}

fn totals(out: &Value) -> [&str; 4] {
    [
        "closing_cash",
        "lowest_balance",
        "funding_gap",
        "deferred_beyond_week_13",
    ]
    .map(|key| amount(out, key))
}

#[test]
fn cash_forecast_matches_the_site() {
    let moved = run(
        "cash-forecast",
        &cash(
            "200",
            weeks(&[(0, "1000")], "100", "0"),
            "100",
            Some((1, "1000", 2)),
        ),
    );
    assert_eq!(moved["weeks"][0]["closing"], "100.00");
    assert_eq!(moved["weeks"][1]["closing"], "0.00");
    assert_eq!(moved["weeks"][2]["receipts"], "1000.00");
    assert_eq!(totals(&moved), ["-100.00", "-100.00", "200.00", "0.00"]);
    let rising = run(
        "cash-forecast",
        &cash("20", weeks(&[], "0", "50"), "100", None),
    );
    assert_eq!(totals(&rising)[1..3], ["20.00", "80.00"]);
    let last = weeks(&[(12, "1000")], "0", "0");
    let past = run(
        "cash-forecast",
        &cash("-50", last.clone(), "100", Some((13, "600", 1))),
    );
    assert_eq!(totals(&past), ["350.00", "-50.00", "150.00", "600.00"]);
    let none = run(
        "cash-forecast",
        &cash("0", last.clone(), "0", Some((13, "0", 13))),
    );
    assert_eq!(totals(&none)[0], "1000.00");
    let landing = run(
        "cash-forecast",
        &cash("0", last.clone(), "0", Some((12, "0", 1))),
    );
    assert_eq!(totals(&landing)[3], "0.00");
    let twelve = weeks(&[(11, "700"), (12, "1000")], "0", "0");
    let into_13 = run(
        "cash-forecast",
        &cash("0", twelve, "0", Some((12, "700", 1))),
    );
    assert_eq!(
        [totals(&into_13)[0], totals(&into_13)[3]],
        ["1700.00", "0.00"]
    );
    // calculator_cases.json row 2: delayed into week 13, so not deferred.
    let row2 = run(
        "cash-forecast",
        &cash(
            "-50",
            weeks(&[(11, "1000")], "0", "0"),
            "100",
            Some((12, "600", 1)),
        ),
    );
    assert_eq!(totals(&row2), ["950.00", "-50.00", "150.00", "0.00"]);
}

#[test]
fn cash_forecast_dates_weeks_and_refuses_as_the_site_does() {
    let last = weeks(&[(12, "1000")], "0", "0");
    let mut dated = cash("-50", last.clone(), "100", Some((13, "600", 1)));
    dated["start_date"] = json!("2026-07-06");
    let out = run("cash-forecast", &dated);
    assert_eq!(out["weeks"][0]["start_date"], "2026-07-06");
    assert_eq!(out["weeks"][0]["end_date"], "2026-07-12");
    assert_eq!(out["weeks"][12]["closing"], "350.00");
    dated["start_date"] = json!("2100-12-01");
    assert_eq!(code("cash-forecast", &dated), "input.date_invalid");
    let over = cash("0", last.clone(), "0", Some((13, "1001", 1)));
    assert_eq!(
        code("cash-forecast", &over),
        "cash_forecast.delay_above_receipts"
    );
    let mut short = cash("0", last.clone(), "0", None);
    short["weeks"].as_array_mut().unwrap().pop();
    assert_eq!(code("cash-forecast", &short), "cash_forecast.weeks_invalid");
    let mut extra_key = cash("0", last.clone(), "0", None);
    extra_key["weeks"][0]["note"] = json!("x");
    assert_eq!(
        code("cash-forecast", &extra_key),
        "cash_forecast.weeks_invalid"
    );
    for delayed in [(0, "1", 1), (14, "1", 1), (1, "1", 14), (1, "-1", 1)] {
        let bad = cash("0", last.clone(), "0", Some(delayed));
        assert_eq!(
            code("cash-forecast", &bad),
            "cash_forecast.delayed_receipt_invalid",
            "{delayed:?}"
        );
    }
    let mut listed = cash("0", last.clone(), "0", None);
    listed["delayed_receipt"] = json!([1, "1", 1]);
    assert_eq!(
        code("cash-forecast", &listed),
        "cash_forecast.delayed_receipt_invalid"
    );
    let mut extra = cash("0", last, "0", None);
    extra["extra"] = json!(1);
    assert_eq!(code("cash-forecast", &extra), "input.field_unknown");
}
