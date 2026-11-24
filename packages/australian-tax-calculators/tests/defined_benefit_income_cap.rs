// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The defined benefit income cap (ITAA 1997 Subdivision 303-A, read on
//! 24 November 2026) against LCR 2017/1 Examples 2 to 6, the general
//! transfer balance cap for each year on the ATO's 'Transfer balance cap'
//! page and hand-worked cases at each boundary
//! (docs/calculation-evidence.md#defined-benefit-income-cap).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("defined-benefit-income-cap", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("defined-benefit-income-cap", input)
        .unwrap_err()
        .code
}

fn base(year: &str, taxed: &str, untaxed: &str, other: &str) -> Value {
    json!({
        "year": year, "scope_confirmed": true, "tax_free_and_taxed": taxed,
        "untaxed": untaxed, "other_defined_benefit_income": other,
    })
}

fn with(mut input: Value, key: &str, value: Value) -> Value {
    input[key] = value;
    input
}

fn amount(out: &Value, key: &str) -> String {
    out["amounts"][key].as_str().unwrap().to_owned()
}

#[test]
fn the_cap_is_a_sixteenth_of_each_years_general_transfer_balance_cap() {
    let rows = [
        ("2017-18", "100000.00"),
        ("2020-21", "100000.00"),
        ("2021-22", "106250.00"),
        ("2022-23", "106250.00"),
        ("2023-24", "118750.00"),
        ("2024-25", "118750.00"),
        ("2025-26", "125000.00"),
        ("2026-27", "131250.00"),
    ];
    for (year, cap) in rows {
        let out = run(&base(year, "0.00", "0.00", "0.00"));
        assert_eq!(amount(&out, "defined_benefit_income_cap"), cap, "{year}");
    }
}

#[test]
fn greg_turns_60_on_1_january_2018() {
    // LCR 2017/1 Example 2: $100,000 x 181 / 365 = $49,589.04, rounded up.
    let out = run(&with(
        base("2017-18", "0.00", "0.00", "0.00"),
        "first_day",
        json!("2018-01-01"),
    ));
    assert_eq!(amount(&out, "unreduced_cap"), "49590.00");
    assert_eq!(out["rates"]["days_after_first_day"], "180");
    assert_eq!(out["rates"]["days_in_year"], "365");
}

#[test]
fn sally_receives_150000_over_a_100000_cap() {
    // LCR 2017/1 Example 3: half the $50,000 excess is assessable.
    let out = run(&base("2017-18", "150000.00", "0.00", "0.00"));
    assert_eq!(amount(&out, "excess_over_cap"), "50000.00");
    assert_eq!(amount(&out, "assessable_income"), "25000.00");
    assert_eq!(amount(&out, "untaxed_offset"), "0.00");
}

#[test]
fn alastairs_untaxed_offset_is_reduced_by_the_excess() {
    // LCR 2017/1 Example 4: $60,000 tax free and $100,000 untaxed.
    let out = run(&base("2017-18", "60000.00", "100000.00", "0.00"));
    assert_eq!(amount(&out, "assessable_income"), "0.00");
    assert_eq!(amount(&out, "untaxed_offset_before_reduction"), "10000.00");
    assert_eq!(amount(&out, "offset_reduction"), "6000.00");
    assert_eq!(amount(&out, "untaxed_offset"), "4000.00");
}

#[test]
fn mary_turns_60_on_12_september_2017() {
    // LCR 2017/1 Example 5: 291 days after her birthday, so a cap of $80,000.
    let out = run(&with(
        base("2017-18", "120000.00", "0.00", "0.00"),
        "first_day",
        json!("2017-09-12"),
    ));
    assert_eq!(amount(&out, "defined_benefit_income_cap"), "80000.00");
    assert_eq!(amount(&out, "assessable_income"), "20000.00");
}

#[test]
fn anns_other_income_uses_up_the_cap() {
    // LCR 2017/1 Example 6: $104,000 of other defined benefit income takes the
    // $80,000 cap to nil; then a full year at $100,000.
    let first = run(&with(
        base("2017-18", "40000.00", "20000.00", "104000.00"),
        "first_day",
        json!("2017-09-12"),
    ));
    assert_eq!(amount(&first, "defined_benefit_income_cap"), "0.00");
    assert_eq!(amount(&first, "assessable_income"), "20000.00");
    assert_eq!(amount(&first, "untaxed_offset_before_reduction"), "2000.00");
    assert_eq!(amount(&first, "untaxed_offset"), "0.00");
    let second = run(&base("2018-19", "150000.00", "55000.00", "0.00"));
    assert_eq!(amount(&second, "assessable_income"), "25000.00");
    assert_eq!(amount(&second, "offset_reduction"), "10500.00");
    assert_eq!(amount(&second, "untaxed_offset"), "0.00");
}

#[test]
fn the_first_and_last_days_of_a_year() {
    let first = run(&with(
        base("2025-26", "0.00", "0.00", "0.00"),
        "first_day",
        json!("2025-07-01"),
    ));
    assert_eq!(amount(&first, "unreduced_cap"), "125000.00");
    let last = run(&with(
        base("2025-26", "0.00", "0.00", "0.00"),
        "first_day",
        json!("2026-06-30"),
    ));
    // $125,000 x 1 / 365 = $342.47, rounded up.
    assert_eq!(amount(&last, "unreduced_cap"), "343.00");
    // 2023-24 has 366 days: $118,750 x 1 / 366 = $324.45, rounded up.
    let leap = run(&with(
        base("2023-24", "0.00", "0.00", "0.00"),
        "first_day",
        json!("2024-06-30"),
    ));
    assert_eq!(leap["rates"]["days_in_year"], "366");
    assert_eq!(amount(&leap, "unreduced_cap"), "325.00");
}

#[test]
fn refusals() {
    let ok = base("2025-26", "1.00", "1.00", "0.00");
    for date in ["2025-06-30", "2026-07-01"] {
        assert_eq!(
            code(&with(ok.clone(), "first_day", json!(date))),
            "defined_benefit_income_cap.first_day_outside_year"
        );
    }
    for year in ["2016-17", "2027-28"] {
        assert_eq!(
            code(&with(ok.clone(), "year", json!(year))),
            "period.unsupported"
        );
    }
    assert_eq!(
        code(&with(ok.clone(), "scope_confirmed", json!(false))),
        "scope.not_confirmed"
    );
    assert_eq!(
        code(&with(ok.clone(), "untaxed", json!("-1.00"))),
        "input.amount_invalid"
    );
    assert_eq!(code(&with(ok, "extra", json!(1))), "input.field_unknown");
}
