// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! ITAA 1997 Division 230's reach (s 230-455) and accruals method
//! (ss 230-100, 230-130, 230-135 and 230-170), read on 26 November 2026. T1 and
//! T2 are the ATO guide's loan (Chris Co) and zero coupon bond (Iron Co)
//! examples, moved into the supported years; the income-year splits, the
//! straight line and T5 were worked independently in Python's `decimal`
//! (docs/calculation-evidence.md#tofa-accruals).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "tofa-accruals";

fn run(input: &Value) -> Value {
    api::evaluate(CMD, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn flow(date: &str, amount: &str) -> Value {
    json!({"date": date, "amount": amount})
}

/// A large company outside s 230-455's thresholds, choosing accruals.
fn input(year: &str, start: &str, end: &str, months: i64, flows: Value) -> Value {
    json!({"start": start, "end": end, "interval_months": months, "financial_benefits": flows,
           "method": "compounding", "approximation_confirmed": false, "accruals_chosen": true,
           "entity_type": "other", "aggregated_turnover": "150000000", "assets": "400000000",
           "financial_assets": "120000000", "qualifying_security": true,
           "threshold_failed_earlier": false, "division_230_election": false, "elections": [],
           "year": year, "scope_confirmed": true})
}

fn chris() -> Value {
    input(
        "2021-22",
        "2021-07-01",
        "2025-06-30",
        12,
        json!([
            flow("2021-07-01", "-10000.00"),
            flow("2025-06-30", "20736.00")
        ]),
    )
}

fn iron() -> Value {
    input(
        "2020-21",
        "2021-01-01",
        "2023-12-31",
        12,
        json!([flow("2021-01-01", "-100.00"), flow("2023-12-31", "133.10")]),
    )
}

fn column(r: &Value, list: &str, key: &str) -> Vec<String> {
    r[list]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row[key].as_str().unwrap().to_string())
        .collect()
}

fn with(mut v: Value, key: &str, value: Value) -> Value {
    v[key] = value;
    v
}

#[test]
fn t1_chris_co_loan_compounds_at_20_percent() {
    let r = run(&chris());
    assert_eq!(
        column(&r, "intervals", "accrual"),
        ["2000.00", "2400.00", "2880.00", "3456.00"]
    );
    assert_eq!(
        column(&r, "intervals", "opening_balance"),
        ["10000.00", "12000.00", "14400.00", "17280.00"]
    );
    assert_eq!(column(&r, "intervals", "closing_balance")[3], "0.00");
    assert_eq!(
        column(&r, "income_years", "income_year"),
        ["2021-22", "2022-23", "2023-24", "2024-25"]
    );
    assert_eq!(
        column(&r, "income_years", "compounding"),
        ["2000.00", "2400.00", "2880.00", "3456.00"]
    );
    assert_eq!(r["rates"]["rate_of_return_per_interval"], "0.200000000000");
    assert_eq!(r["amounts"]["overall_gain_or_loss"], "10736.00");
    assert_eq!(r["amounts"]["first_income_year_amount"], "2000.00");
    assert_eq!(r["division_230_applies"], true);
}

#[test]
fn t2_iron_co_zero_coupon_bond_splits_straddling_intervals_by_days() {
    let r = run(&iron());
    assert_eq!(
        column(&r, "intervals", "accrual"),
        ["10.00", "11.00", "12.10"]
    );
    assert_eq!(r["rates"]["rate_of_return_per_interval"], "0.100000000000");
    assert_eq!(
        column(&r, "income_years", "income_year"),
        ["2020-21", "2021-22", "2022-23", "2023-24"]
    );
    assert_eq!(
        column(&r, "income_years", "compounding"),
        ["4.96", "10.49", "11.55", "6.10"]
    );
}

#[test]
fn t3_a_borrower_accrues_the_mirror_loss() {
    let v = input(
        "2021-22",
        "2021-07-01",
        "2025-06-30",
        12,
        json!([
            flow("2021-07-01", "10000.00"),
            flow("2025-06-30", "-20736.00")
        ]),
    );
    let r = run(&v);
    assert_eq!(
        column(&r, "intervals", "accrual"),
        ["-2000.00", "-2400.00", "-2880.00", "-3456.00"]
    );
    assert_eq!(r["amounts"]["overall_gain_or_loss"], "-10736.00");
}

#[test]
fn t4_straight_line_spreads_by_days_beside_compounding() {
    let v = with(
        with(chris(), "method", json!("straight_line")),
        "approximation_confirmed",
        json!(true),
    );
    let r = run(&v);
    assert_eq!(
        column(&r, "income_years", "straight_line"),
        ["2682.16", "2682.17", "2689.51", "2682.16"]
    );
    assert_eq!(
        column(&r, "income_years", "compounding"),
        ["2000.00", "2400.00", "2880.00", "3456.00"]
    );
    assert_eq!(r["amounts"]["first_income_year_amount"], "2682.16");
    assert_eq!(r["method"], "straight_line");
}

#[test]
fn t5_a_discount_bond_with_coupons_finds_its_rate() {
    let v = input(
        "2021-22",
        "2021-07-01",
        "2024-06-30",
        12,
        json!([
            flow("2021-07-01", "-950.00"),
            flow("2022-06-30", "50.00"),
            flow("2023-06-30", "50.00"),
            flow("2024-06-30", "1050.00")
        ]),
    );
    let r = run(&v);
    assert_eq!(r["rates"]["rate_of_return_per_interval"], "0.069018424518");
    assert_eq!(
        column(&r, "intervals", "accrual"),
        ["65.57", "66.64", "67.79"]
    );
    assert_eq!(r["amounts"]["overall_gain_or_loss"], "200.00");
}

#[test]
fn t6_a_par_bond_with_quarterly_coupons_accrues_each_coupon() {
    let mut flows = vec![flow("2021-07-01", "-1000.00")];
    for end in ["2021-09-30", "2021-12-31", "2022-03-31"] {
        flows.push(flow(end, "25.00"));
    }
    flows.push(flow("2022-06-30", "1025.00"));
    let r = run(&input(
        "2021-22",
        "2021-07-01",
        "2022-06-30",
        3,
        json!(flows),
    ));
    assert_eq!(
        column(&r, "intervals", "accrual"),
        ["25.00", "25.00", "25.00", "25.00"]
    );
    assert_eq!(
        column(&r, "intervals", "end"),
        ["2021-09-30", "2021-12-31", "2022-03-31", "2022-06-30"]
    );
    assert_eq!(r["rates"]["rate_of_return_per_interval"], "0.025000000000");
}

/// A small company: (aggregated turnover, assets, financial assets).
fn sized(kind: &str, figures: (&str, &str, &str), qualifying: bool) -> Value {
    let v = with(
        with(chris(), "entity_type", json!(kind)),
        "qualifying_security",
        json!(qualifying),
    );
    let v = with(
        with(v, "aggregated_turnover", json!(figures.0)),
        "assets",
        json!(figures.1),
    );
    with(v, "financial_assets", json!(figures.2))
}

fn applies(v: &Value) -> bool {
    run(v)["division_230_applies"].as_bool().unwrap()
}

#[test]
fn section_230_455_thresholds_on_each_boundary() {
    let small = ("99999999.99", "299999999.99", "99999999.99");
    assert!(!applies(&sized("other", small, false)));
    assert!(applies(&sized("other", ("100000000", "1", "1"), false)));
    assert!(applies(&sized("other", ("1", "300000000", "1"), false)));
    assert!(applies(&sized("other", ("1", "1", "100000000"), false)));
    assert!(!applies(&sized(
        "super_or_scheme",
        ("510000000", "99999999.99", "1"),
        false
    )));
    assert!(applies(&sized(
        "super_or_scheme",
        ("1", "100000000", "1"),
        false
    )));
    assert!(!applies(&sized(
        "adi_or_securitisation",
        ("19999999.99", "900000000", "1"),
        false
    )));
    assert!(applies(&sized(
        "adi_or_securitisation",
        ("20000000", "1", "1"),
        false
    )));
    // A qualifying security longer than 12 months is reached (s 230-455(1)(e)).
    assert!(applies(&sized("other", small, true)));
}

#[test]
fn an_election_or_an_earlier_failure_brings_a_small_entity_in() {
    let small = sized("other", ("1", "1", "1"), false);
    assert!(applies(&with(
        small.clone(),
        "division_230_election",
        json!(true)
    )));
    assert!(applies(&with(
        small.clone(),
        "threshold_failed_earlier",
        json!(true)
    )));
    let r = run(&small);
    assert!(r["reason"].as_str().unwrap().contains("s 230-455(1)"));
    assert!(r.get("intervals").is_none());
}

fn individual(qualifying: bool) -> Value {
    let mut v = with(chris(), "entity_type", json!("individual"));
    for key in ["aggregated_turnover", "assets", "financial_assets"] {
        v.as_object_mut().unwrap().remove(key);
    }
    with(v, "qualifying_security", json!(qualifying))
}

#[test]
fn an_individual_is_reached_only_by_a_long_qualifying_security() {
    assert!(!applies(&individual(false)));
    // Section 230-100(4) then applies, so accruals apply without a choice.
    let v = with(individual(true), "accruals_chosen", json!(false));
    assert_eq!(column(&run(&v), "intervals", "accrual")[0], "2000.00");
    // An earlier failure does not concern an individual (s 230-455(9)).
    assert!(!applies(&with(
        individual(false),
        "threshold_failed_earlier",
        json!(true)
    )));
}

#[test]
fn a_qualifying_security_ending_within_12_months_is_excluded() {
    let v = input(
        "2021-22",
        "2021-07-01",
        "2022-06-30",
        12,
        json!([flow("2021-07-01", "-100.00"), flow("2022-06-30", "110.00")]),
    );
    let v = with(
        with(
            with(v, "aggregated_turnover", json!("1")),
            "assets",
            json!("1"),
        ),
        "financial_assets",
        json!("1"),
    );
    assert!(!applies(&v));
}

fn check(cases: Vec<(Value, &str)>) {
    for (v, want) in cases {
        assert_eq!(code(&v), want, "{v}");
    }
}

#[test]
fn refusals_of_methods_and_elections() {
    check(vec![
        (
            with(chris(), "accruals_chosen", json!(false)),
            "tofa_accruals.accruals_not_chosen",
        ),
        (
            with(chris(), "method", json!("straight_line")),
            "tofa_accruals.approximation_not_confirmed",
        ),
        (
            with(chris(), "method", json!("effective_interest")),
            "tofa_accruals.method_not_built",
        ),
        (
            with(chris(), "method", json!("daily")),
            "tofa_accruals.method_invalid",
        ),
        (
            with(chris(), "elections", json!(["hedging"])),
            "tofa_accruals.election_not_built",
        ),
        (
            with(chris(), "elections", json!(["fair_value"])),
            "tofa_accruals.election_not_built",
        ),
        (
            with(chris(), "elections", json!(["other"])),
            "tofa_accruals.elections_invalid",
        ),
        (
            with(chris(), "elections", json!("none")),
            "tofa_accruals.elections_invalid",
        ),
    ]);
}

#[test]
fn refusals_of_terms_and_benefits() {
    check(vec![
        (
            with(chris(), "entity_type", json!("company")),
            "tofa_accruals.entity_type_invalid",
        ),
        (
            with(chris(), "interval_months", json!(13)),
            "tofa_accruals.interval_invalid",
        ),
        (
            with(chris(), "interval_months", json!(5)),
            "tofa_accruals.term_invalid",
        ),
        (
            with(chris(), "start", json!("2022-07-01")),
            "tofa_accruals.term_invalid",
        ),
        (
            with(
                chris(),
                "financial_benefits",
                json!([flow("2021-07-01", "-1.00")]),
            ),
            "tofa_accruals.benefits_invalid",
        ),
        (
            with(
                chris(),
                "financial_benefits",
                json!([flow("2021-07-01", "0"), flow("2025-06-30", "1.00")]),
            ),
            "tofa_accruals.benefits_invalid",
        ),
        (
            with(
                chris(),
                "financial_benefits",
                json!([flow("2021-07-01", "-1.00"), flow("2023-01-15", "2.00")]),
            ),
            "tofa_accruals.benefit_within_interval",
        ),
        (
            with(
                chris(),
                "financial_benefits",
                json!([
                    flow("2021-07-01", "-1.00"),
                    flow("2022-06-30", "2.00"),
                    flow("2025-06-30", "-2.00")
                ]),
            ),
            "tofa_accruals.rate_not_unique",
        ),
    ]);
}

#[test]
fn refusals_of_threshold_inputs() {
    check(vec![
        (
            with(
                chris(),
                "financial_benefits",
                json!([flow("2021-07-01", "1.00"), flow("2025-06-30", "2.00")]),
            ),
            "tofa_accruals.rate_not_unique",
        ),
        (
            with(
                chris(),
                "financial_benefits",
                json!([flow("2021-07-01", "-1.00"), flow("2025-06-30", "1000.00")]),
            ),
            "tofa_accruals.rate_out_of_range",
        ),
        (
            individual(false)
                .as_object()
                .map(|o| {
                    let mut o = o.clone();
                    o.insert("assets".into(), json!("1"));
                    Value::Object(o)
                })
                .unwrap(),
            "tofa_accruals.threshold_input",
        ),
        (
            chris()
                .as_object()
                .map(|o| {
                    let mut o = o.clone();
                    o.remove("assets");
                    Value::Object(o)
                })
                .unwrap(),
            "tofa_accruals.threshold_input",
        ),
        (
            with(chris(), "year", json!("2019-20")),
            "period.unsupported",
        ),
        (with(chris(), "unknown", json!(1)), "input.field_unknown"),
    ]);
}

#[test]
fn refusals_of_malformed_benefits() {
    check(vec![
        (
            with(chris(), "start", json!("1 July 2021")),
            "input.date_invalid",
        ),
        (
            with(chris(), "financial_benefits", json!("none")),
            "tofa_accruals.benefits_invalid",
        ),
        (
            with(
                chris(),
                "financial_benefits",
                json!([flow("2021-07-01", "-1.00"), {"date": "2025-06-30", "amount": "2.00", "note": "x"}]),
            ),
            "tofa_accruals.benefits_invalid",
        ),
        (
            with(
                chris(),
                "financial_benefits",
                json!([flow("2021-07-01", "-1000.00"), flow("2025-06-30", "1.00")]),
            ),
            "tofa_accruals.rate_out_of_range",
        ),
    ]);
}

/// Benefits that net to nil have no overall gain or loss (Alcuin S2).
#[test]
fn benefits_netting_to_nil_are_refused() {
    let v = with(
        chris(),
        "financial_benefits",
        json!([
            flow("2021-07-01", "-10000.00"),
            flow("2025-06-30", "10000.00")
        ]),
    );
    assert_eq!(code(&v), "tofa_accruals.no_gain_or_loss");
}

/// Intervals run from the start under Acts Interpretation Act s 2G(2): from
/// 31 August, 6 months end on 29 February in a leap year.
#[test]
fn intervals_from_a_month_end_follow_the_calendar() {
    let v = input(
        "2023-24",
        "2023-08-31",
        "2024-08-30",
        6,
        json!([flow("2023-08-31", "-100.00"), flow("2024-08-30", "121.00")]),
    );
    let r = run(&v);
    assert_eq!(column(&r, "intervals", "end"), ["2024-02-29", "2024-08-30"]);
    assert_eq!(column(&r, "intervals", "accrual"), ["10.00", "11.00"]);
}

#[test]
fn every_year_boundary_of_the_start() {
    for (year, start, end) in [
        ("2020-21", "2020-07-01", "2021-06-30"),
        ("2026-27", "2027-06-30", "2028-06-29"),
    ] {
        let v = input(
            year,
            start,
            end,
            12,
            json!([flow(start, "-100.00"), flow(end, "110.00")]),
        );
        assert_eq!(run(&v)["amounts"]["overall_gain_or_loss"], "10.00");
    }
}
