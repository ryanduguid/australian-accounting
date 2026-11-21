// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The tax on a superannuation lump sum (ITAA 1997 Divisions 301, 302 and
//! 307, read on 22 November 2026) against the ATO's 'Tax on super benefits'
//! and Schedule 12 examples, the s 307-125 example, each age band, cap and
//! year boundary, and each refusal (docs/calculation-evidence.md#super-benefit-tax).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A member's benefit whose interest is the benefit itself, so the
/// components are as stated.
fn member(year: &str, paid: &str, born: &str, parts: [&str; 3]) -> Value {
    let [free, taxed, untaxed] = parts;
    let benefit = cents(free) + cents(taxed) + cents(untaxed);
    let benefit = format!("{}.{:02}", benefit / 100, benefit % 100);
    let mut v = json!({
        "benefit": benefit, "interest_value": benefit,
        "interest_tax_free_component": free, "untaxed_element": untaxed,
        "recipient": "member", "payment_date": paid, "date_of_birth": born,
        "terminal_medical_condition": false,
        "disability_superannuation_benefit": false,
        "year": year, "scope_confirmed": true,
    });
    if untaxed != "0.00" {
        v["service_before_july_1983"] = json!(false);
        v["untaxed_plan_cap_used"] = json!("0.00");
    }
    v
}

fn cents(text: &str) -> u64 {
    let (whole, part) = text.split_once('.').unwrap();
    whole.parse::<u64>().unwrap() * 100 + part.parse::<u64>().unwrap()
}

/// A death benefit of the given components in 2026-27.
fn death(recipient: &str, parts: [&str; 3]) -> Value {
    let mut v = member("2026-27", "2026-10-01", "1980-01-01", parts);
    let o = v.as_object_mut().unwrap();
    for key in [
        "date_of_birth",
        "terminal_medical_condition",
        "disability_superannuation_benefit",
        "untaxed_plan_cap_used",
    ] {
        o.remove(key);
    }
    v["recipient"] = json!(recipient);
    v
}

fn run(input: &Value) -> Value {
    api::evaluate("super-benefit-tax", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("super-benefit-tax", input)
        .unwrap_err()
        .code
}

fn amount(r: &Value, key: &str) -> String {
    r["amounts"][key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} in {r}"))
        .to_string()
}

/// s 307-125(2)'s example: $100 from a $1,000 interest with a $200 tax free
/// component has a $20 tax free component and an $80 taxable component.
#[test]
fn the_proportioning_rule_follows_the_acts_example() {
    let mut input = member("2026-27", "2026-10-01", "1960-01-01", ["0.00"; 3]);
    input["benefit"] = json!("100.00");
    input["interest_value"] = json!("1000.00");
    input["interest_tax_free_component"] = json!("200.00");
    let r = run(&input);
    assert_eq!(amount(&r, "tax_free_component"), "20.00");
    assert_eq!(amount(&r, "taxable_component"), "80.00");
    assert_eq!(amount(&r, "taxed_element"), "80.00");
    assert_eq!(r["treatment"], "aged_60_or_over");
}

/// The tax free component is rounded to the cent, 0.5 cents up, and the
/// taxable component is the rest.
#[test]
fn the_tax_free_component_is_rounded_half_up() {
    let mut input = member(
        "2026-27",
        "2026-10-01",
        "1960-01-01",
        ["0.00", "0.00", "0.00"],
    );
    input["benefit"] = json!("1.00");
    input["interest_value"] = json!("8.00");
    input["interest_tax_free_component"] = json!("1.00");
    let r = run(&input);
    assert_eq!(amount(&r, "tax_free_component"), "0.13");
    assert_eq!(amount(&r, "taxable_component"), "0.87");
}

/// The ATO's Tony: 58, $350,000 on 25 July 2021, $100,000 tax free and
/// $250,000 taxed. $225,000 is within the 2021-22 low rate cap at 0%,
/// $25,000 above it at 15% (17% with Medicare levy). It is the catalogue's
/// example.
#[test]
fn tony_pays_15_percent_above_the_low_rate_cap() {
    let mut input = member(
        "2021-22",
        "2021-07-25",
        "1963-03-01",
        ["100000.00", "250000.00", "0.00"],
    );
    input["low_rate_cap_used"] = json!("0.00");
    let r = run(&input);
    assert_eq!(r["treatment"], "preservation_age_to_59");
    assert_eq!(r["rates"]["age"], "58");
    assert_eq!(r["rates"]["preservation_age"], "58");
    assert_eq!(r["rates"]["low_rate_cap"], "225000");
    assert_eq!(amount(&r, "taxed_within_low_rate_cap"), "225000.00");
    assert_eq!(amount(&r, "taxed_within_low_rate_cap_tax"), "0.00");
    assert_eq!(amount(&r, "taxed_above_low_rate_cap"), "25000.00");
    assert_eq!(amount(&r, "taxed_above_low_rate_cap_tax"), "3750.00");
    assert_eq!(amount(&r, "maximum_tax"), "3750.00");
    assert_eq!(amount(&r, "assessable_amount"), "250000.00");
    assert_eq!(amount(&r, "non_assessable_non_exempt"), "100000.00");
    assert_eq!(amount(&r, "low_rate_cap_counted"), "225000.00");
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["super_benefit_tax"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_eq!(run(&facts)["amounts"]["maximum_tax"], "3750.00");
}

/// Schedule 12 (from 13 October 2020): Heather, 59, and Dean, 61, each take
/// $250,000 with $30,000 tax free in 2020-21. Heather has $5,000 above the
/// $215,000 low rate cap (withholding $850 at 17%); Dean's taxed element is
/// tax free.
#[test]
fn heather_and_dean_follow_schedule_12() {
    let mut heather = member(
        "2020-21",
        "2020-12-01",
        "1961-01-15",
        ["30000.00", "220000.00", "0.00"],
    );
    heather["low_rate_cap_used"] = json!("0.00");
    let r = run(&heather);
    assert_eq!(r["rates"]["preservation_age"], "56");
    assert_eq!(amount(&r, "taxed_above_low_rate_cap"), "5000.00");
    assert_eq!(amount(&r, "maximum_tax"), "750.00");
    let dean = member(
        "2020-21",
        "2020-12-01",
        "1959-06-01",
        ["30000.00", "220000.00", "0.00"],
    );
    let r = run(&dean);
    assert_eq!(r["treatment"], "aged_60_or_over");
    assert_eq!(amount(&r, "maximum_tax"), "0.00");
    assert_eq!(amount(&r, "assessable_amount"), "0.00");
    assert_eq!(amount(&r, "non_assessable_non_exempt"), "250000.00");
}

/// The ATO's Mei, 60 on 25 September 2021: $205,000 untaxed within the
/// $1,615,000 untaxed plan cap at 15% (17% with Medicare levy).
#[test]
fn mei_pays_15_percent_on_the_untaxed_element() {
    let input = member(
        "2021-22",
        "2021-09-25",
        "1961-06-01",
        ["45000.00", "130000.00", "205000.00"],
    );
    let r = run(&input);
    assert_eq!(r["rates"]["untaxed_plan_cap"], "1615000");
    assert_eq!(amount(&r, "untaxed_within_plan_cap"), "205000.00");
    assert_eq!(amount(&r, "untaxed_within_plan_cap_tax"), "30750.00");
    assert_eq!(amount(&r, "untaxed_above_plan_cap"), "0.00");
    assert_eq!(amount(&r, "maximum_tax"), "30750.00");
    assert_eq!(amount(&r, "assessable_amount"), "205000.00");
    assert_eq!(amount(&r, "untaxed_plan_cap_counted"), "205000.00");
}

/// The ATO's Suzi, 50: $10,000 taxed at 20% (22% with Medicare levy); and
/// Frankie, 45: $45,000 taxed at 20% and $20,000 untaxed at 30%.
#[test]
fn under_preservation_age_pays_20_and_30_percent() {
    let suzi = member(
        "2022-23",
        "2022-10-01",
        "1972-05-01",
        ["1000.00", "10000.00", "0.00"],
    );
    let r = run(&suzi);
    assert_eq!(r["treatment"], "under_preservation_age");
    assert_eq!(amount(&r, "taxed_element_part_tax"), "2000.00");
    assert_eq!(amount(&r, "maximum_tax"), "2000.00");
    let frankie = member(
        "2022-23",
        "2022-10-01",
        "1977-05-01",
        ["5000.00", "45000.00", "20000.00"],
    );
    let r = run(&frankie);
    assert_eq!(amount(&r, "taxed_element_part_tax"), "9000.00");
    assert_eq!(amount(&r, "untaxed_within_plan_cap_tax"), "6000.00");
    assert_eq!(amount(&r, "maximum_tax"), "15000.00");
    assert_eq!(amount(&r, "assessable_amount"), "65000.00");
}

/// The ATO's Peta, 59 in 2020-21, past both caps: $40,000 taxed at 15% and
/// $45,000 untaxed at the top rate; Howard, 72, past the plan cap: $45,000
/// untaxed at the top rate.
#[test]
fn past_both_caps_the_untaxed_element_bears_the_top_rate() {
    let mut peta = member(
        "2020-21",
        "2021-02-01",
        "1961-03-01",
        ["20000.00", "40000.00", "45000.00"],
    );
    peta["low_rate_cap_used"] = json!("215000.00");
    peta["untaxed_plan_cap_used"] = json!("1565000.00");
    let r = run(&peta);
    assert_eq!(amount(&r, "low_rate_cap_left"), "0.00");
    assert_eq!(amount(&r, "taxed_above_low_rate_cap_tax"), "6000.00");
    assert_eq!(amount(&r, "untaxed_above_plan_cap_tax"), "20250.00");
    assert_eq!(amount(&r, "maximum_tax"), "26250.00");
    assert_eq!(amount(&r, "low_rate_cap_counted"), "0.00");
    assert_eq!(amount(&r, "untaxed_plan_cap_counted"), "0.00");
    let mut howard = member(
        "2022-23",
        "2022-10-01",
        "1950-05-01",
        ["20000.00", "40000.00", "45000.00"],
    );
    howard["untaxed_plan_cap_used"] = json!("1650000.00");
    let r = run(&howard);
    assert_eq!(amount(&r, "untaxed_above_plan_cap_tax"), "20250.00");
    assert_eq!(amount(&r, "maximum_tax"), "20250.00");
}

/// The ATO's Anh: in 2020-21 the $1,565,000 untaxed plan cap less the
/// $165,000 she had received leaves $1,400,000.
#[test]
fn anhs_untaxed_plan_cap_left_is_1_4_million() {
    let mut anh = member(
        "2020-21",
        "2020-10-01",
        "1955-01-01",
        ["0.00", "0.00", "90000.00"],
    );
    anh["untaxed_plan_cap_used"] = json!("165000.00");
    let r = run(&anh);
    assert_eq!(amount(&r, "untaxed_plan_cap_left"), "1400000.00");
    assert_eq!(amount(&r, "maximum_tax"), "13500.00");
}

/// Schedule 12 (2026)'s Chloe, 62: $80,000 untaxed at 15% ($13,600
/// withheld at 17%).
#[test]
fn chloe_pays_15_percent_in_2026_27() {
    let chloe = member(
        "2026-27",
        "2026-10-01",
        "1964-01-01",
        ["30000.00", "190000.00", "80000.00"],
    );
    let r = run(&chloe);
    assert_eq!(r["rates"]["untaxed_plan_cap"], "1935000");
    assert_eq!(amount(&r, "maximum_tax"), "12000.00");
}

/// ss 301-20 and 301-105(6): the low rate cap goes to the element taxed in
/// the fund first; the untaxed element gets 15% on what is left, 30% to the
/// untaxed plan cap and 45% above it.
#[test]
fn the_low_rate_cap_goes_to_the_taxed_element_first() {
    let mut input = member(
        "2021-22",
        "2021-08-01",
        "1963-01-10",
        ["0.00", "200000.00", "100000.00"],
    );
    input["low_rate_cap_used"] = json!("0.00");
    let r = run(&input);
    assert_eq!(amount(&r, "taxed_within_low_rate_cap"), "200000.00");
    assert_eq!(amount(&r, "untaxed_within_low_rate_cap"), "25000.00");
    assert_eq!(amount(&r, "untaxed_within_low_rate_cap_tax"), "3750.00");
    assert_eq!(amount(&r, "untaxed_within_plan_cap"), "75000.00");
    assert_eq!(amount(&r, "untaxed_within_plan_cap_tax"), "22500.00");
    assert_eq!(amount(&r, "maximum_tax"), "26250.00");
    assert_eq!(amount(&r, "low_rate_cap_counted"), "225000.00");
    // The untaxed plan cap splits at its edge: $1,615,000 less $1,600,000
    // leaves $15,000 at 15% and 30%, the rest at 45%.
    input["untaxed_plan_cap_used"] = json!("1600000.00");
    let r = run(&input);
    assert_eq!(amount(&r, "untaxed_within_low_rate_cap"), "15000.00");
    assert_eq!(amount(&r, "untaxed_within_plan_cap"), "0.00");
    assert_eq!(amount(&r, "untaxed_above_plan_cap"), "85000.00");
    assert_eq!(amount(&r, "untaxed_above_plan_cap_tax"), "38250.00");
}

/// Each year's caps, as the ATO's Tables 9 and 10 give them.
#[test]
fn each_years_caps() {
    let low = [
        ("2020-21", "215000"),
        ("2021-22", "225000"),
        ("2022-23", "230000"),
        ("2023-24", "235000"),
    ];
    for (year, cap) in low {
        let first: i32 = year[..4].parse().unwrap();
        let born = format!("{}-06-10", first - 59);
        let mut input = member(
            year,
            &format!("{}-06-01", first + 1),
            &born,
            ["0.00", "1000.00", "0.00"],
        );
        input["low_rate_cap_used"] = json!("0.00");
        let r = run(&input);
        assert_eq!(r["treatment"], "preservation_age_to_59", "{year}");
        assert_eq!(r["rates"]["low_rate_cap"], cap, "{year}");
    }
    let plan = [
        ("2020-21", "1565000"),
        ("2021-22", "1615000"),
        ("2022-23", "1650000"),
        ("2023-24", "1705000"),
        ("2024-25", "1780000"),
        ("2025-26", "1865000"),
        ("2026-27", "1935000"),
    ];
    for (year, cap) in plan {
        let first: i32 = year[..4].parse().unwrap();
        let input = member(
            year,
            &format!("{first}-07-01"),
            "1950-01-01",
            ["0.00", "0.00", "1000.00"],
        );
        assert_eq!(run(&input)["rates"]["untaxed_plan_cap"], cap, "{year}");
    }
}

/// Age turns on the birthday, 29 February on 1 March in a common year, and
/// from 1 July 2024 nobody is between preservation age and 60.
#[test]
fn age_and_preservation_age_boundaries() {
    let treat = |year: &str, paid: &str, born: &str| {
        let mut input = member(year, paid, born, ["0.00", "1000.00", "0.00"]);
        let r = api::evaluate("super-benefit-tax", &input);
        if r.is_err() {
            input["low_rate_cap_used"] = json!("0.00");
        }
        run(&input)["treatment"].as_str().unwrap().to_string()
    };
    assert_eq!(
        treat("2023-24", "2024-06-30", "1964-06-30"),
        "aged_60_or_over"
    );
    assert_eq!(
        treat("2023-24", "2024-06-29", "1964-06-30"),
        "preservation_age_to_59"
    );
    assert_eq!(
        treat("2023-24", "2024-06-30", "1964-07-01"),
        "under_preservation_age"
    );
    assert_eq!(
        treat("2023-24", "2023-12-30", "1963-12-31"),
        "preservation_age_to_59"
    );
    assert_eq!(
        treat("2023-24", "2023-12-30", "1964-12-31"),
        "under_preservation_age"
    );
    assert_eq!(
        treat("2023-24", "2024-02-28", "1964-02-29"),
        "preservation_age_to_59"
    );
    assert_eq!(
        treat("2023-24", "2024-02-29", "1964-02-29"),
        "aged_60_or_over"
    );
    assert_eq!(
        treat("2022-23", "2023-02-28", "1964-02-29"),
        "under_preservation_age"
    );
    assert_eq!(
        treat("2022-23", "2023-03-01", "1964-02-29"),
        "preservation_age_to_59"
    );
    assert_eq!(
        treat("2020-21", "2020-07-01", "1961-07-01"),
        "preservation_age_to_59"
    );
    assert_eq!(
        treat("2020-21", "2020-07-01", "1964-07-01"),
        "under_preservation_age"
    );
    assert_eq!(
        treat("2024-25", "2024-07-01", "1964-07-02"),
        "under_preservation_age"
    );
    assert_eq!(
        treat("2024-25", "2024-07-01", "1964-06-30"),
        "aged_60_or_over"
    );
    assert_eq!(
        treat("2026-27", "2027-06-30", "1967-06-30"),
        "aged_60_or_over"
    );
    assert_eq!(
        treat("2026-27", "2027-06-29", "1967-06-30"),
        "under_preservation_age"
    );
}

/// s 302-60: a dependant's lump sum is wholly tax free; s 302-145: a
/// non-dependant's taxed element bears 15% and untaxed element 30%.
#[test]
fn death_benefits_to_dependants_and_non_dependants() {
    let parts = ["100000.00", "300000.00", "100000.00"];
    let r = run(&death("death_benefits_dependant", parts));
    assert_eq!(r["treatment"], "death_benefits_dependant");
    assert_eq!(amount(&r, "maximum_tax"), "0.00");
    assert_eq!(amount(&r, "non_assessable_non_exempt"), "500000.00");
    let r = run(&death("non_dependant", parts));
    assert_eq!(amount(&r, "taxed_element_part_tax"), "45000.00");
    assert_eq!(amount(&r, "untaxed_element_part_tax"), "30000.00");
    assert_eq!(amount(&r, "maximum_tax"), "75000.00");
    assert_eq!(amount(&r, "assessable_amount"), "400000.00");
    assert_eq!(amount(&r, "non_assessable_non_exempt"), "100000.00");
    assert!(r["rates"].get("age").is_none());
}

/// s 303-10: a terminal medical condition makes a member's lump sum tax
/// free; s 301-225(1): so does one under $200 that closes the interest.
#[test]
fn terminal_medical_condition_and_small_closing_balance() {
    let mut input = member(
        "2026-27",
        "2026-10-01",
        "1981-01-01",
        ["0.00", "50000.00", "1000.00"],
    );
    input["terminal_medical_condition"] = json!(true);
    input
        .as_object_mut()
        .unwrap()
        .remove("untaxed_plan_cap_used");
    let r = run(&input);
    assert_eq!(r["treatment"], "terminal_medical_condition");
    assert_eq!(amount(&r, "maximum_tax"), "0.00");
    let small = member(
        "2026-27",
        "2026-10-01",
        "1981-01-01",
        ["0.00", "199.99", "0.00"],
    );
    let r = run(&small);
    assert_eq!(r["treatment"], "under_200_closing_the_interest");
    assert_eq!(amount(&r, "maximum_tax"), "0.00");
    let mut open = small.clone();
    open["interest_value"] = json!("500.00");
    let r = run(&open);
    assert_eq!(r["treatment"], "under_preservation_age");
    assert_eq!(amount(&r, "maximum_tax"), "40.00");
    let whole = member(
        "2026-27",
        "2026-10-01",
        "1981-01-01",
        ["0.00", "200.00", "0.00"],
    );
    assert_eq!(amount(&run(&whole), "maximum_tax"), "40.00");
}

#[test]
fn bad_inputs_are_refused() {
    let base = member(
        "2021-22",
        "2021-08-01",
        "1950-01-01",
        ["1000.00", "9000.00", "0.00"],
    );
    let with = |key: &str, value: Value| {
        let mut v = base.clone();
        v[key] = value;
        v
    };
    assert_eq!(
        code(&with("benefit", json!("0.00"))),
        "super_benefit_tax.benefit_nil"
    );
    assert_eq!(
        code(&with("interest_value", json!("0.00"))),
        "super_benefit_tax.benefit_nil"
    );
    assert_eq!(
        code(&with("benefit", json!("10000.01"))),
        "super_benefit_tax.fact_conflict"
    );
    assert_eq!(
        code(&with("interest_tax_free_component", json!("10000.01"))),
        "super_benefit_tax.fact_conflict"
    );
    assert_eq!(
        code(&with("untaxed_element", json!("9000.01"))),
        "super_benefit_tax.fact_conflict"
    );
    assert_eq!(
        code(&with("payment_date", json!("2022-07-01"))),
        "super_benefit_tax.fact_conflict"
    );
    assert_eq!(
        code(&with("payment_date", json!("2021-06-30"))),
        "super_benefit_tax.fact_conflict"
    );
    assert_eq!(
        code(&with("payment_date", json!("2021-13-01"))),
        "input.date_invalid"
    );
    assert_eq!(
        code(&with("date_of_birth", json!("2021-08-01"))),
        "super_benefit_tax.fact_conflict"
    );
    assert_eq!(
        code(&with("recipient", json!("estate"))),
        "super_benefit_tax.recipient_invalid"
    );
    assert_eq!(
        code(&with("disability_superannuation_benefit", json!(true))),
        "super_benefit_tax.disability_benefit"
    );
    assert_eq!(
        code(&with("terminal_medical_condition", json!("yes"))),
        "input.flag_invalid"
    );
    assert_eq!(
        code(&with("benefit", json!("-1.00"))),
        "input.amount_invalid"
    );
    for year in ["2019-20", "2027-28"] {
        assert_eq!(
            code(&with("year", json!(year))),
            "period.unsupported",
            "{year}"
        );
    }
    assert_eq!(
        code(&with("scope_confirmed", json!(false))),
        "scope.not_confirmed"
    );
    assert_eq!(code(&with("unexpected", json!(1))), "input.field_unknown");
    // Member facts are refused for a death benefit and required for a member.
    let mut dead = base.clone();
    dead["recipient"] = json!("non_dependant");
    assert_eq!(code(&dead), "input.field_unknown");
    let mut no_birth = base.clone();
    no_birth.as_object_mut().unwrap().remove("date_of_birth");
    assert_eq!(code(&no_birth), "input.missing_field");
    // The caps: required where they apply, refused where they cannot, and
    // never above the year's cap.
    let mut late = base.clone();
    late["low_rate_cap_used"] = json!("0.00");
    assert_eq!(code(&late), "input.field_unknown");
    let mut untaxed = with("untaxed_element", json!("100.00"));
    assert_eq!(code(&untaxed), "input.missing_field");
    untaxed["service_before_july_1983"] = json!(true);
    untaxed["untaxed_plan_cap_used"] = json!("0.00");
    assert_eq!(code(&untaxed), "super_benefit_tax.pre_july_1983_service");
    untaxed["service_before_july_1983"] = json!(false);
    untaxed["untaxed_plan_cap_used"] = json!("1615000.01");
    assert_eq!(code(&untaxed), "super_benefit_tax.fact_conflict");
    let mut low = member(
        "2021-22",
        "2021-08-01",
        "1963-01-10",
        ["0.00", "1000.00", "0.00"],
    );
    assert_eq!(code(&low), "input.missing_field");
    low["low_rate_cap_used"] = json!("225000.01");
    assert_eq!(code(&low), "super_benefit_tax.fact_conflict");
    let mut dependant = death("death_benefits_dependant", ["0.00", "1000.00", "0.00"]);
    dependant["untaxed_plan_cap_used"] = json!("0.00");
    assert_eq!(code(&dependant), "input.field_unknown");
}
