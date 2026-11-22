// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `trading-stock` (ITAA 1997 ss 70-35 to 70-50 and Subdivision 328-E) and
//! `prepayments` (ITAA 1936 ss 82KZL to 82KZMF), read on 22 November 2026,
//! against the ATO's examples (Simplified trading stock rules, 27 May 2026;
//! Deductions for prepaid expenses 2021 and 2026), the s 328-295 example and
//! hand-worked cases at each edge
//! (docs/calculation-evidence.md#trading-stock and #prepayments).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(command: &str, input: &Value) -> Value {
    api::evaluate(command, input).unwrap()
}

fn code(command: &str, input: &Value) -> &'static str {
    api::evaluate_coded(command, input).unwrap_err().code
}

fn with(mut base: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        base[key] = value.clone();
    }
    base
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// A small business entity's trading stock for 2025-26, with no items.
fn stock(opening: &str, estimate: Value, choice: bool) -> Value {
    json!({
        "year": "2025-26", "scope_confirmed": true, "entity": "small_business_entity",
        "opening_value": opening, "items": [], "estimated_closing_value": estimate,
        "simplified_choice": choice,
    })
}

fn item(cost: &str, msv: &str, rv: &str, method: &str) -> Value {
    json!({"cost": cost, "market_selling_value": msv, "replacement_value": rv, "method": method})
}

/// The s 328-295 example: Angela's $30,000 opening value and $34,000
/// estimate; under the choice the closing value is $30,000.
#[test]
fn angela_keeps_her_opening_value() {
    let out = run("trading-stock", &stock("30000.00", json!("34000.00"), true));
    assert_eq!(amount(&out, "closing_value"), "30000.00");
    assert_eq!(amount(&out, "estimate_difference"), "4000.00");
    assert_eq!(amount(&out, "assessable_increase"), "0.00");
    assert_eq!(amount(&out, "deduction"), "0.00");
    assert_eq!(out["simplified_chosen"], true);
}

/// The ATO's Colin ($6,800 to $7,820) and Joel ($5,600 to $8,000 and to
/// $12,000).
#[test]
fn colin_and_joel() {
    let colin = run("trading-stock", &stock("6800.00", json!("7820.00"), true));
    assert_eq!(amount(&colin, "closing_value"), "6800.00");
    assert_eq!(colin["simplified_available"], true);
    let joel = run("trading-stock", &stock("5600.00", json!("8000.00"), true));
    assert_eq!(amount(&joel, "closing_value"), "5600.00");
    let over = stock("5600.00", json!("12000.00"), true);
    assert_eq!(
        code("trading-stock", &over),
        "trading_stock.simplified_unavailable"
    );
    // Joel's stocktake at $12,000 without the choice: $6,400 assessable.
    let counted = with(
        stock("5600.00", Value::Null, false),
        json!({"items": [item("12000.00", "15000.00", "12500.00", "cost")]}),
    );
    let out = run("trading-stock", &counted);
    assert_eq!(amount(&out, "assessable_increase"), "6400.00");
    assert_eq!(out["simplified_available"], false);
}

#[test]
fn the_5000_limit_is_inclusive_both_ways() {
    for (estimate, ok) in [
        ("15000.00", true),
        ("15000.01", false),
        ("5000.00", true),
        ("4999.99", false),
    ] {
        let input = stock("10000.00", json!(estimate), true);
        let out = api::evaluate_coded("trading-stock", &input);
        assert_eq!(out.is_ok(), ok, "{estimate}");
    }
}

/// s 328-285(2) applies from 2021-22 (Act No. 92 of 2020 Sch 3 item 40(8)).
#[test]
fn a_medium_business_may_choose_from_2021_22() {
    let medium = |year: &str| {
        with(
            stock("10000.00", json!("11000.00"), true),
            json!({"year": year, "entity": "medium_business_entity"}),
        )
    };
    assert_eq!(
        code("trading-stock", &medium("2020-21")),
        "trading_stock.simplified_unavailable"
    );
    for year in ["2021-22", "2026-27"] {
        assert_eq!(
            amount(&run("trading-stock", &medium(year)), "closing_value"),
            "10000.00"
        );
    }
    let other = with(medium("2025-26"), json!({"entity": "other"}));
    assert_eq!(
        code("trading-stock", &other),
        "trading_stock.simplified_unavailable"
    );
    let small_2020 = with(
        medium("2020-21"),
        json!({"entity": "small_business_entity"}),
    );
    assert_eq!(
        run("trading-stock", &small_2020)["simplified_available"],
        true
    );
}

/// Each item by its own election: lowest, market selling value and
/// obsolescence (ss 70-45 and 70-50), worked by hand.
#[test]
fn each_item_takes_its_own_election() {
    let input = with(
        stock("10000.00", Value::Null, false),
        json!({"items": [
            item("4000.00", "5000.00", "4500.00", "lowest"),
            item("3000.00", "2500.00", "2800.00", "market_selling_value"),
            {"cost": "600.00", "market_selling_value": "700.00", "replacement_value": "800.00",
             "method": "obsolescence", "obsolescence_value": "100.00"},
        ]}),
    );
    let out = run("trading-stock", &input);
    assert_eq!(amount(&out, "stocktake_value"), "6600.00");
    assert_eq!(amount(&out, "closing_value"), "6600.00");
    assert_eq!(amount(&out, "deduction"), "3400.00");
    assert_eq!(amount(&out, "assessable_increase"), "0.00");
    let methods: Vec<&str> = out["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["method"].as_str().unwrap())
        .collect();
    assert_eq!(methods, ["cost", "market_selling_value", "obsolescence"]);
    // Without an estimate the items' total is the estimate: $3,400 apart.
    assert_eq!(amount(&out, "estimated_closing_value"), "6600.00");
    assert_eq!(out["simplified_available"], true);
}

#[test]
fn lowest_takes_the_first_of_equal_values() {
    let input = with(
        stock("0.00", Value::Null, false),
        json!({"items": [item("100.00", "90.00", "90.00", "lowest")]}),
    );
    let out = run("trading-stock", &input);
    assert_eq!(out["items"][0]["method"], "market_selling_value");
    assert_eq!(out["items"][0]["value"], "90.00");
}

#[test]
fn trading_stock_refusals() {
    let base = stock("1000.00", Value::Null, false);
    let items = |list: Value| with(base.clone(), json!({"items": list}));
    let cases = [
        (
            with(base.clone(), json!({"entity": "large"})),
            "trading_stock.entity_invalid",
        ),
        (
            items(json!({"cost": "1.00"})),
            "trading_stock.items_invalid",
        ),
        (items(json!(["x"])), "trading_stock.items_invalid"),
        (
            items(json!([item("1.00", "2.00", "3.00", "average")])),
            "trading_stock.method_invalid",
        ),
        (
            items(
                json!([{"cost": null, "market_selling_value": "2.00", "replacement_value": "3.00", "method": "cost"}]),
            ),
            "trading_stock.value_missing",
        ),
        (
            items(
                json!([{"cost": "1.00", "market_selling_value": null, "replacement_value": "3.00", "method": "lowest"}]),
            ),
            "trading_stock.value_missing",
        ),
        (
            items(
                json!([{"cost": "600.00", "market_selling_value": "700.00", "replacement_value": "800.00",
                "method": "obsolescence", "obsolescence_value": "600.00"}]),
            ),
            "trading_stock.obsolescence_not_below",
        ),
        (
            items(
                json!([{"cost": "1.00", "market_selling_value": "2.00", "replacement_value": "3.00",
                "method": "cost", "obsolescence_value": "0.50"}]),
            ),
            "input.field_unknown",
        ),
        (
            items(
                json!([{"cost": "1.00", "market_selling_value": "2.00", "replacement_value": "3.00", "method": "cost", "note": "x"}]),
            ),
            "input.field_unknown",
        ),
        (
            with(base.clone(), json!({"opening_value": "-1"})),
            "input.amount_invalid",
        ),
        (
            with(base.clone(), json!({"year": "2019-20"})),
            "period.unsupported",
        ),
        (
            with(base.clone(), json!({"year": "2027-28"})),
            "period.unsupported",
        ),
        (
            with(base.clone(), json!({"scope_confirmed": false})),
            "scope.not_confirmed",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(code("trading-stock", &input), expected, "{input}");
    }
    let many: Vec<Value> = (0..501)
        .map(|_| item("1.00", "1.00", "1.00", "cost"))
        .collect();
    assert_eq!(
        code("trading-stock", &items(json!(many))),
        "trading_stock.items_invalid"
    );
}

#[test]
fn every_year_from_2020_21_to_2026_27() {
    for year in [
        "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
    ] {
        let out = run(
            "trading-stock",
            &with(stock("1.00", json!("2.00"), true), json!({"year": year})),
        );
        assert_eq!(out["period"], year);
        assert_eq!(out["rates"]["simplified_limit"], "5000");
        let pre = run(
            "prepayments",
            &with(
                tom(),
                json!({"year": year, "incurred": format!("{}-05-31", &year[..4].parse::<i32>().unwrap() + 1), "agreement_date": "2020-07-01"}),
            ),
        );
        assert_eq!(pre["period"], year);
    }
}

/// Tom Pty Ltd (ATO Deductions for prepaid expenses 2026): a small business
/// entity's $15,000 on 31 May 2026 for 1 June 2026 to 30 June 2027.
fn tom() -> Value {
    json!({
        "year": "2025-26", "scope_confirmed": true, "taxpayer": "small_business_entity",
        "individual": false, "business_expenditure": true, "chose_82kzmd": false,
        "expenditure": "15000.00", "incurred": "2026-05-31", "service_start": "2026-06-01",
        "service_end": "2027-06-30", "agreement_date": "2026-05-31", "exclusion": "none",
        "tax_shelter": false,
    })
}

fn schedule(out: &Value) -> Vec<(String, Value, String)> {
    out["schedule"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["year"].as_str().unwrap().to_owned(),
                r["days"].clone(),
                r["deduction"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn rows(list: &[(&str, i64, &str)]) -> Vec<(String, Value, String)> {
    list.iter()
        .map(|(y, d, a)| ((*y).to_owned(), json!(d), (*a).to_owned()))
        .collect()
}

/// The ATO rounds to whole dollars: $1,139 and $13,861.
#[test]
fn tom_pty_ltd_apportions_over_395_days() {
    let out = run("prepayments", &tom());
    assert_eq!(out["treatment"], "apportioned_82kzm");
    assert_eq!(out["eligible_service_period"]["days"], 395);
    assert_eq!(
        schedule(&out),
        rows(&[("2025-26", 30, "1139.24"), ("2026-27", 365, "13860.76")])
    );
    assert_eq!(amount(&out, "deduction_expenditure_year"), "1139.24");
    assert_eq!(amount(&out, "deduction_later_years"), "13860.76");
}

/// Noel Pty Ltd: 12 months that end after the next income year; $9,809 and
/// $391.
#[test]
fn noel_pty_ltd_ends_after_the_next_year() {
    let input = with(
        tom(),
        json!({"expenditure": "10200.00", "incurred": "2026-06-30", "service_start": "2026-07-15",
               "service_end": "2027-07-14", "agreement_date": "2026-06-30"}),
    );
    let out = run("prepayments", &input);
    assert_eq!(out["treatment"], "apportioned_82kzm");
    assert_eq!(
        schedule(&out),
        rows(&[
            ("2025-26", 0, "0.00"),
            ("2026-27", 351, "9808.77"),
            ("2027-28", 14, "391.23")
        ])
    );
}

/// The Jacobs Trust: $24,000 on 1 June 2026 for 1 July 2026 to 30 June 2027.
#[test]
fn the_jacobs_trust_takes_the_12_month_rule() {
    let input = with(
        tom(),
        json!({"expenditure": "24000.00", "incurred": "2026-06-01", "service_start": "2026-07-01",
               "service_end": "2027-06-30", "agreement_date": "2026-06-01"}),
    );
    let out = run("prepayments", &input);
    assert_eq!(out["treatment"], "twelve_month_rule");
    assert_eq!(amount(&out, "deduction_expenditure_year"), "24000.00");
    assert_eq!(
        out["schedule"],
        json!([{"year": "2025-26", "days": null, "deduction": "24000.00"}])
    );
    // The same lease for a medium business in 2020-21 (Act No. 92 of 2020
    // Sch 3 item 40(5)): the 12-month rule too.
    let medium = with(
        input.clone(),
        json!({"year": "2020-21", "taxpayer": "medium_business_entity", "incurred": "2021-06-01",
               "service_start": "2021-07-01", "service_end": "2022-06-30", "agreement_date": "2021-06-01"}),
    );
    assert_eq!(
        run("prepayments", &medium)["treatment"],
        "twelve_month_rule"
    );
    // Choosing s 82KZMD apportions it instead.
    let chose = with(input, json!({"chose_82kzmd": true}));
    let out = run("prepayments", &chose);
    assert_eq!(out["treatment"], "apportioned_82kzmd");
    assert_eq!(
        schedule(&out),
        rows(&[("2025-26", 0, "0.00"), ("2026-27", 365, "24000.00")])
    );
}

/// An individual's non-business subscriptions: Martin (2021 guide, $571 and
/// $679 over 181 and 215 of 396 days) and Jasmin (2026 guide).
#[test]
fn martin_and_jasmin() {
    let martin = with(
        tom(),
        json!({"year": "2020-21", "taxpayer": "other", "individual": true, "business_expenditure": false,
               "expenditure": "1250.00", "incurred": "2021-01-01", "service_start": "2021-01-01",
               "service_end": "2022-01-31", "agreement_date": "2021-01-01"}),
    );
    let out = run("prepayments", &martin);
    assert_eq!(out["treatment"], "apportioned_82kzm");
    assert_eq!(
        schedule(&out),
        rows(&[("2020-21", 181, "571.34"), ("2021-22", 215, "678.66")])
    );
    let jasmin = with(
        martin,
        json!({"year": "2025-26", "expenditure": "1750.00", "incurred": "2026-06-01",
               "service_start": "2026-06-01", "service_end": "2027-05-31", "agreement_date": "2026-06-01"}),
    );
    let out = run("prepayments", &jasmin);
    assert_eq!(out["treatment"], "twelve_month_rule");
    assert_eq!(amount(&out, "deduction_expenditure_year"), "1750.00");
}

/// Marion's olive grove (2026 guide): a stated tax shelter; $1,671 and $8,329.
/// The same 12 months would meet the 12-month rule without it.
#[test]
fn marion_apportions_under_82kzmf() {
    let marion = with(
        tom(),
        json!({"taxpayer": "other", "individual": true, "business_expenditure": false,
               "expenditure": "10000.00", "incurred": "2026-05-01", "service_start": "2026-05-01",
               "service_end": "2027-04-30", "agreement_date": "2026-04-30", "tax_shelter": true}),
    );
    let out = run("prepayments", &marion);
    assert_eq!(out["treatment"], "apportioned_82kzmf");
    assert_eq!(
        schedule(&out),
        rows(&[("2025-26", 61, "1671.23"), ("2026-27", 304, "8328.77")])
    );
    let plain = with(marion, json!({"tax_shelter": false}));
    assert_eq!(run("prepayments", &plain)["treatment"], "twelve_month_rule");
}

/// John's truck registration (a state law), Maree's $950 net of GST and the
/// $1,000 edge: excluded expenditure is deductible when incurred.
#[test]
fn excluded_expenditure() {
    let john = with(
        tom(),
        json!({"taxpayer": "other", "individual": true, "expenditure": "1200.00", "incurred": "2025-12-31",
               "service_start": "2026-01-01", "service_end": "2026-12-31", "agreement_date": "2025-12-31",
               "exclusion": "law_or_court_order"}),
    );
    let out = run("prepayments", &john);
    assert_eq!(out["treatment"], "excluded_expenditure");
    assert_eq!(out["eligible_service_period"], Value::Null);
    assert_eq!(amount(&out, "deduction_expenditure_year"), "1200.00");
    let base = with(
        john,
        json!({"exclusion": "none", "service_end": "2027-12-31"}),
    );
    for (expenditure, treatment) in [
        ("950.00", "excluded_expenditure"),
        ("999.99", "excluded_expenditure"),
        ("1000.00", "apportioned_82kzmd"),
    ] {
        let out = run(
            "prepayments",
            &with(base.clone(), json!({"expenditure": expenditure})),
        );
        assert_eq!(out["treatment"], treatment, "{expenditure}");
    }
    let wages = with(base, json!({"exclusion": "contract_of_service"}));
    assert_eq!(
        run("prepayments", &wages)["treatment"],
        "excluded_expenditure"
    );
}

/// A large business's $12,000 for 1 April 2026 to 31 March 2027 under
/// s 82KZMD, worked by hand: 91 and 274 of 365 days.
#[test]
fn a_large_business_apportions_under_82kzmd() {
    let input = with(
        tom(),
        json!({"taxpayer": "other", "expenditure": "12000.00", "incurred": "2026-04-01",
               "service_start": "2026-04-01", "service_end": "2027-03-31", "agreement_date": "2026-03-01"}),
    );
    let out = run("prepayments", &input);
    assert_eq!(out["treatment"], "apportioned_82kzmd");
    assert_eq!(
        schedule(&out),
        rows(&[("2025-26", 91, "2991.78"), ("2026-27", 274, "9008.22")])
    );
    // A non-individual not carrying on a business: s 82KZMD as well.
    let idle = with(input, json!({"business_expenditure": false}));
    assert_eq!(run("prepayments", &idle)["treatment"], "apportioned_82kzmd");
}

/// The 10-year cap (s 82KZL(1)(d)) and the 12 months of s 82KZM(1)(ba)
/// under Acts Interpretation Act s 2G(2).
#[test]
fn period_edges() {
    let long = with(
        tom(),
        json!({"expenditure": "100000.00", "incurred": "2026-06-30", "service_start": "2026-07-01",
               "service_end": "2041-06-30", "agreement_date": "2026-06-30"}),
    );
    let out = run("prepayments", &long);
    assert_eq!(
        out["eligible_service_period"],
        json!({"start": "2026-07-01", "end": "2036-06-30", "days": 3653})
    );
    let years = schedule(&out);
    assert_eq!(years.len(), 11);
    assert_eq!(
        years[0],
        ("2025-26".to_owned(), json!(0), "0.00".to_owned())
    );
    assert_eq!(years.last().unwrap().0, "2035-36");
    let total: f64 = years.iter().map(|r| r.2.parse::<f64>().unwrap()).sum();
    assert!((total - 100000.0).abs() < 0.001);
    let twelve = |end: &str, incurred: &str| {
        with(
            tom(),
            json!({"incurred": incurred, "agreement_date": incurred, "service_start": "2026-02-15", "service_end": end}),
        )
    };
    // From 15 February 2026: 12 months end on 14 February 2027.
    assert_eq!(
        run("prepayments", &twelve("2027-02-14", "2026-02-15"))["treatment"],
        "twelve_month_rule"
    );
    assert_eq!(
        run("prepayments", &twelve("2027-02-15", "2026-02-15"))["treatment"],
        "apportioned_82kzm"
    );
    // Within 12 months but past the next year's end: 2 July 2026 to 1 July 2027.
    let late = with(
        tom(),
        json!({"service_start": "2026-07-02", "service_end": "2027-07-01"}),
    );
    assert_eq!(run("prepayments", &late)["treatment"], "apportioned_82kzm");
    let ends = with(
        tom(),
        json!({"service_start": "2026-07-01", "service_end": "2027-06-30"}),
    );
    assert_eq!(run("prepayments", &ends)["treatment"], "twelve_month_rule");
}

/// A thing wholly done in the expenditure year, or before the expenditure
/// was incurred, is not prepaid.
#[test]
fn not_prepaid() {
    let in_year = with(
        tom(),
        json!({"service_end": "2026-06-30", "taxpayer": "other"}),
    );
    let out = run("prepayments", &in_year);
    assert_eq!(out["treatment"], "done_in_year");
    assert_eq!(amount(&out, "deduction_expenditure_year"), "15000.00");
    let arrears = with(
        tom(),
        json!({"taxpayer": "other", "service_start": "2025-07-01", "service_end": "2026-03-31"}),
    );
    let out = run("prepayments", &arrears);
    assert_eq!(out["treatment"], "not_prepaid");
    assert_eq!(out["eligible_service_period"], Value::Null);
    // Service begun before the payment: the period starts when incurred.
    let begun = with(
        tom(),
        json!({"taxpayer": "other", "service_start": "2026-01-01"}),
    );
    let out = run("prepayments", &begun);
    assert_eq!(out["eligible_service_period"]["start"], "2026-05-31");
    assert_eq!(out["eligible_service_period"]["days"], 396);
}

#[test]
fn prepayment_refusals() {
    let cases = [
        (json!({"taxpayer": "large"}), "prepayments.taxpayer_invalid"),
        (
            json!({"exclusion": "capital"}),
            "prepayments.exclusion_invalid",
        ),
        (
            json!({"incurred": "2026-07-01", "agreement_date": "2026-07-01"}),
            "prepayments.dates_invalid",
        ),
        (
            json!({"incurred": "2025-06-30", "agreement_date": "2025-06-30"}),
            "prepayments.dates_invalid",
        ),
        (
            json!({"service_end": "2026-05-31"}),
            "prepayments.dates_invalid",
        ),
        (
            json!({"agreement_date": "2026-06-01"}),
            "prepayments.dates_invalid",
        ),
        (
            json!({"agreement_date": "1999-11-11"}),
            "prepayments.agreement_before_cutoff",
        ),
        (
            json!({"taxpayer": "other", "chose_82kzmd": true}),
            "prepayments.choice_unavailable",
        ),
        (json!({"incurred": "2026-02-30"}), "input.date_invalid"),
        (json!({"tax_shelter": "yes"}), "input.flag_invalid"),
        (json!({"expenditure": "1.001"}), "input.amount_invalid"),
        (json!({"year": "2019-20"}), "period.unsupported"),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"note": "x"}), "input.field_unknown"),
    ];
    for (changes, expected) in cases {
        assert_eq!(
            code("prepayments", &with(tom(), changes.clone())),
            expected,
            "{changes}"
        );
    }
    let after = with(tom(), json!({"agreement_date": "1999-11-12"}));
    assert!(api::evaluate("prepayments", &after).is_ok());
}

/// Round 1 S1: a s 82KZMD choice for expenditure it cannot apply to is
/// refused: an individual's non-business expenditure (s 82KZMA(3)(a)),
/// excluded expenditure (s 82KZMA(4)) and a thing done in the expenditure year
/// (s 82KZMA(3)(c)). S2: a stated tax shelter that s 82KZME cannot cover.
#[test]
fn contradicted_choices_and_shelters_are_refused() {
    let chose = with(tom(), json!({"chose_82kzmd": true}));
    let cases = [
        (
            json!({"individual": true, "business_expenditure": false}),
            "prepayments.choice_unavailable",
        ),
        (
            json!({"expenditure": "999.99"}),
            "prepayments.choice_unavailable",
        ),
        (
            json!({"service_end": "2026-06-30"}),
            "prepayments.choice_unavailable",
        ),
        (
            json!({"tax_shelter": true}),
            "prepayments.choice_unavailable",
        ),
    ];
    for (changes, expected) in cases {
        assert_eq!(
            code("prepayments", &with(chose.clone(), changes.clone())),
            expected,
            "{changes}"
        );
    }
    assert_eq!(
        run("prepayments", &chose)["treatment"],
        "apportioned_82kzmd"
    );
    let shelter = with(tom(), json!({"tax_shelter": true}));
    for changes in [
        json!({"exclusion": "law_or_court_order"}),
        json!({"expenditure": "950.00"}),
        json!({"service_end": "2026-06-30"}),
        json!({"service_start": "2025-07-01", "service_end": "2026-03-31"}),
    ] {
        assert_eq!(
            code("prepayments", &with(shelter.clone(), changes.clone())),
            "prepayments.tax_shelter_conflict",
            "{changes}"
        );
    }
    assert_eq!(
        run("prepayments", &shelter)["treatment"],
        "apportioned_82kzmf"
    );
}

/// Round 1 evidence gaps: Mike's period, 396 days as the 2021 guide counts it
/// (the 2026 guide says 397; the amount is fabricated), Jasmin's $1,500 for 2025-26 (2026 guide), Martin on the 2026
/// guide's dates and the 10-year edge.
#[test]
fn more_published_periods_and_the_10_year_edge() {
    let mike = with(
        tom(),
        json!({"expenditure": "3960.00", "incurred": "2025-12-01", "service_start": "2025-12-01",
               "service_end": "2026-12-31", "agreement_date": "2025-12-01"}),
    );
    let out = run("prepayments", &mike);
    assert_eq!(out["eligible_service_period"]["days"], 396);
    assert_eq!(
        schedule(&out),
        rows(&[("2025-26", 212, "2120.00"), ("2026-27", 184, "1840.00")])
    );
    let jasmin = with(
        tom(),
        json!({"taxpayer": "other", "individual": true, "business_expenditure": false,
               "expenditure": "1500.00", "incurred": "2025-07-01", "service_start": "2025-07-01",
               "service_end": "2026-06-30", "agreement_date": "2025-07-01"}),
    );
    assert_eq!(run("prepayments", &jasmin)["treatment"], "done_in_year");
    let martin = with(
        jasmin,
        json!({"expenditure": "1250.00", "incurred": "2026-01-01", "service_start": "2026-01-01",
               "service_end": "2027-01-31", "agreement_date": "2026-01-01"}),
    );
    let out = run("prepayments", &martin);
    assert_eq!(
        schedule(&out),
        rows(&[("2025-26", 181, "571.34"), ("2026-27", 215, "678.66")])
    );
    let ten = |end: &str| {
        let input = with(
            tom(),
            json!({"incurred": "2026-06-30", "service_start": "2026-07-01", "service_end": end,
                   "agreement_date": "2026-06-30"}),
        );
        run("prepayments", &input)["eligible_service_period"]["end"].clone()
    };
    assert_eq!(ten("2036-06-30"), "2036-06-30");
    assert_eq!(ten("2036-07-01"), "2036-06-30");
    assert_eq!(ten("2036-06-29"), "2036-06-29");
}

/// Round 1 evidence gap: no opening stock, the choice made on an estimate
/// that differs from the stocktake.
#[test]
fn the_choice_rests_on_the_estimate_not_the_stocktake() {
    let input = with(
        stock("0.00", json!("4000.00"), true),
        json!({"items": [item("9000.00", "9500.00", "9200.00", "cost")]}),
    );
    let out = run("trading-stock", &input);
    assert_eq!(amount(&out, "stocktake_value"), "9000.00");
    assert_eq!(amount(&out, "estimate_difference"), "4000.00");
    assert_eq!(amount(&out, "closing_value"), "0.00");
    assert_eq!(amount(&out, "assessable_increase"), "0.00");
    let counted = with(input.clone(), json!({"estimated_closing_value": null}));
    assert_eq!(
        code("trading-stock", &counted),
        "trading_stock.simplified_unavailable"
    );
    let without = with(input, json!({"simplified_choice": false}));
    let out = run("trading-stock", &without);
    assert_eq!(amount(&out, "assessable_increase"), "9000.00");
    assert_eq!(out["simplified_available"], true);
}
