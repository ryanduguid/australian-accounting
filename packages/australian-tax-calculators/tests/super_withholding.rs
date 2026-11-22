// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Schedule 12 withholding from a superannuation lump sum (the ATO's
//! Schedule 12 from 13 October 2020 to 30 June 2024 and Schedule 12 of
//! F2024L00664, F2025L00647 and F2026L00716, read on 22 November 2026)
//! against the schedules' Tom, Chloe, Heather and Dean examples, each rate in
//! Table A, the roll-over and no-TFN rules, rounding, each year's caps and
//! each refusal (docs/calculation-evidence.md#super-withholding).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A resident member with a TFN whose interest is the benefit itself.
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
        "tfn_provided": true, "foreign_resident": false,
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

/// A roll-over of an element untaxed in the fund.
fn rollover(year: &str, paid: &str, untaxed: &str) -> Value {
    json!({
        "benefit": untaxed, "interest_value": untaxed,
        "interest_tax_free_component": "0.00", "untaxed_element": untaxed,
        "recipient": "rollover", "payment_date": paid,
        "service_before_july_1983": false, "untaxed_plan_cap_used": "0.00",
        "tfn_provided": true, "foreign_resident": false,
        "year": year, "scope_confirmed": true,
    })
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

fn withheld(input: &Value) -> String {
    let r = api::evaluate("super-withholding", input).unwrap();
    r["amounts"]["withholding"]
        .as_str()
        .unwrap_or_else(|| panic!("withholding in {r}"))
        .to_string()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("super-withholding", input)
        .unwrap_err()
        .code
}

/// Tom's roll-overs: 47% of the element untaxed in the fund above each
/// year's untaxed plan cap, as each schedule's example works it.
#[test]
fn tom_rolls_over_above_the_untaxed_plan_cap() {
    for (year, paid, untaxed, expected) in [
        ("2020-21", "2020-11-01", "1585000.00", "9400.00"),
        ("2024-25", "2024-09-01", "1820000.00", "18800.00"),
        ("2025-26", "2025-09-01", "1900000.00", "16450.00"),
        ("2026-27", "2026-09-01", "1967000.00", "15040.00"),
    ] {
        assert_eq!(withheld(&rollover(year, paid, untaxed)), expected, "{year}");
    }
    // Within the cap nothing is withheld, and with no element untaxed in the
    // fund there is nothing to withhold from.
    assert_eq!(
        withheld(&rollover("2026-27", "2026-09-01", "1935000.00")),
        "0.00"
    );
    let mut taxed = rollover("2026-27", "2026-09-01", "1000.00");
    taxed["untaxed_element"] = json!("0.00");
    let o = taxed.as_object_mut().unwrap();
    o.remove("service_before_july_1983");
    o.remove("untaxed_plan_cap_used");
    assert_eq!(withheld(&taxed), "0.00");
}

/// Each year's untaxed plan cap, $1,000 above it: $470.
#[test]
fn each_years_untaxed_plan_cap() {
    for (year, cap) in [
        ("2020-21", 1_565_000),
        ("2021-22", 1_615_000),
        ("2022-23", 1_650_000),
        ("2023-24", 1_705_000),
        ("2024-25", 1_780_000),
        ("2025-26", 1_865_000),
        ("2026-27", 1_935_000),
    ] {
        let first: i32 = year[..4].parse().unwrap();
        let paid = format!("{}-03-01", first + 1);
        let input = rollover(year, &paid, &format!("{}.00", cap + 1000));
        assert_eq!(withheld(&input), "470.00", "{year}");
    }
}

/// Schedule 12 (2024, 2025 and 2026): Chloe, 62, has $13,600 withheld, 17%
/// of her $80,000 element untaxed in the fund.
#[test]
fn chloe_has_17_percent_withheld() {
    for (year, paid, born) in [
        ("2024-25", "2024-09-01", "1962-03-01"),
        ("2025-26", "2025-09-01", "1963-03-01"),
        ("2026-27", "2026-09-01", "1964-03-01"),
    ] {
        let chloe = member(year, paid, born, ["30000.00", "190000.00", "80000.00"]);
        assert_eq!(withheld(&chloe), "13600.00", "{year}");
    }
}

/// The roll-over rule states one rate whatever the TFN or residency: 47% of
/// the untaxed element above the untaxed plan cap left, cents ignored
/// without a TFN.
#[test]
fn rollovers_without_a_tfn_or_to_a_foreign_resident() {
    let tom = rollover("2026-27", "2026-09-01", "1967000.00");
    let mut no_tfn = tom.clone();
    no_tfn["tfn_provided"] = json!(false);
    assert_eq!(withheld(&no_tfn), "15040.00");
    let mut foreign = tom.clone();
    foreign["foreign_resident"] = json!(true);
    assert_eq!(withheld(&foreign), "15040.00");
    // $1,001.99 above the cap: 470.9353 rounds to $471 with a TFN; without
    // one, 47% of $1,001 is $470.47, cents ignored.
    let cents = rollover("2026-27", "2026-09-01", "1936001.99");
    assert_eq!(withheld(&cents), "471.00");
    let mut cents = cents;
    cents["tfn_provided"] = json!(false);
    assert_eq!(withheld(&cents), "470.00");
    // Within the cap, nothing either way.
    let mut within = rollover("2026-27", "2026-09-01", "1935000.00");
    within["tfn_provided"] = json!(false);
    assert_eq!(withheld(&within), "0.00");
}

/// Schedule 12 (2020): Heather, 59, has 17% withheld from the $5,000 above
/// the $215,000 low rate cap; Dean, 61, has nothing withheld.
#[test]
fn heather_and_dean_follow_schedule_12() {
    let mut heather = member(
        "2020-21",
        "2020-12-01",
        "1961-01-15",
        ["30000.00", "220000.00", "0.00"],
    );
    heather["low_rate_cap_used"] = json!("0.00");
    heather["other_lump_sums_this_year"] = json!(false);
    assert_eq!(withheld(&heather), "850.00");
    // Another lump sum that year with cap left is refused, as in
    // super-benefit-tax; without a TFN the input is not taken.
    heather["other_lump_sums_this_year"] = json!(true);
    assert_eq!(
        api::evaluate_coded("super-withholding", &heather)
            .unwrap_err()
            .code,
        "super_benefit_tax.other_lump_sums_this_year"
    );
    heather["tfn_provided"] = json!(false);
    heather.as_object_mut().unwrap().remove("low_rate_cap_used");
    assert_eq!(
        api::evaluate_coded("super-withholding", &heather)
            .unwrap_err()
            .code,
        "input.field_unknown"
    );
    let dean = member(
        "2020-21",
        "2020-12-01",
        "1959-06-01",
        ["30000.00", "220000.00", "0.00"],
    );
    assert_eq!(withheld(&dean), "0.00");
}

/// Table A's member rates, each the offset's ceiling rate plus 2%: 22% and
/// 32% under preservation age; nil, 17% and 32% from preservation age to 59;
/// 17% at 60; 47% above the untaxed plan cap.
#[test]
fn table_a_member_rates() {
    let under = member(
        "2022-23",
        "2022-10-01",
        "1972-05-01",
        ["0.00", "10000.00", "20000.00"],
    );
    assert_eq!(withheld(&under), "8600.00");
    // Preservation age 58, both caps nil left except $1,000 of the low rate
    // cap: taxed $1,000 nil, $9,000 at 17%; untaxed $10,000 at 32% to the
    // plan cap and $10,000 at 47% above it.
    let mut band = member(
        "2021-22",
        "2021-08-01",
        "1963-01-10",
        ["0.00", "10000.00", "20000.00"],
    );
    band["low_rate_cap_used"] = json!("224000.00");
    band["other_lump_sums_this_year"] = json!(false);
    band["untaxed_plan_cap_used"] = json!("1605000.00");
    assert_eq!(withheld(&band), "9430.00");
    // Low rate cap left after the taxed element goes to the untaxed element
    // at 17%: taxed $10,000 nil; untaxed $5,000 at 17% and $15,000 at 32%.
    band["low_rate_cap_used"] = json!("210000.00");
    band["other_lump_sums_this_year"] = json!(false);
    band["untaxed_plan_cap_used"] = json!("0.00");
    assert_eq!(withheld(&band), "5650.00");
    let mut aged = member(
        "2023-24",
        "2023-10-01",
        "1960-01-01",
        ["0.00", "10000.00", "20000.00"],
    );
    aged["untaxed_plan_cap_used"] = json!("1695000.00");
    assert_eq!(withheld(&aged), "6400.00");
}

/// The total is rounded once to the nearest dollar, 50 cents up: $1.43 and
/// $1.12 withhold $3, not $1 and $1.
#[test]
fn the_total_is_rounded_once_half_up() {
    let mut input = member(
        "2022-23",
        "2022-10-01",
        "1972-05-01",
        ["0.00", "6.50", "3.50"],
    );
    input["interest_value"] = json!("1000.00");
    assert_eq!(withheld(&input), "3.00");
    let mut input = member(
        "2022-23",
        "2022-10-01",
        "1972-05-01",
        ["0.00", "25.00", "0.00"],
    );
    input["interest_value"] = json!("1000.00");
    assert_eq!(withheld(&input), "6.00");
}

/// A foreign resident with a TFN: Table A less the 2% Medicare levy.
#[test]
fn a_foreign_resident_has_the_levy_taken_out() {
    let mut suzi = member(
        "2022-23",
        "2022-10-01",
        "1972-05-01",
        ["1000.00", "10000.00", "0.00"],
    );
    assert_eq!(withheld(&suzi), "2200.00");
    suzi["foreign_resident"] = json!(true);
    assert_eq!(withheld(&suzi), "2000.00");
}

/// Death benefits: 17% and 32% from a non-dependant's elements, nothing from
/// a dependant's or a deceased estate's.
#[test]
fn death_benefits() {
    let parts = ["100000.00", "300000.00", "100000.00"];
    assert_eq!(withheld(&death("non_dependant", parts)), "83000.00");
    assert_eq!(withheld(&death("death_benefits_dependant", parts)), "0.00");
    let mut estate = death("deceased_estate_trustee", parts);
    estate
        .as_object_mut()
        .unwrap()
        .remove("service_before_july_1983");
    assert_eq!(withheld(&estate), "0.00");
}

/// Nothing is withheld from a terminal medical condition payment or a
/// member's entire benefit under $200.
#[test]
fn terminal_medical_condition_and_small_balances() {
    let mut input = member(
        "2026-27",
        "2026-10-01",
        "1980-01-01",
        ["0.00", "50000.00", "0.00"],
    );
    input["terminal_medical_condition"] = json!(true);
    assert_eq!(withheld(&input), "0.00");
    let small = member(
        "2026-27",
        "2026-10-01",
        "1980-01-01",
        ["0.00", "199.99", "0.00"],
    );
    assert_eq!(withheld(&small), "0.00");
    // The untaxed plan cap does not bear on a nil amount, so it is not taken.
    let mut untaxed = member(
        "2026-27",
        "2026-10-01",
        "1980-01-01",
        ["0.00", "0.00", "1000.00"],
    );
    untaxed["terminal_medical_condition"] = json!(true);
    assert_eq!(code(&untaxed), "input.field_unknown");
    untaxed
        .as_object_mut()
        .unwrap()
        .remove("untaxed_plan_cap_used");
    assert_eq!(withheld(&untaxed), "0.00");
}

/// No TFN: 47% (45% foreign) of the taxable component under 60, of the
/// element untaxed in the fund at 60 or over, ignoring cents.
#[test]
fn no_tfn_rates_ignore_cents() {
    let mut young = member(
        "2026-27",
        "2026-10-01",
        "1980-01-01",
        ["500.00", "10000.99", "0.00"],
    );
    young["tfn_provided"] = json!(false);
    assert_eq!(withheld(&young), "4700.00");
    young["foreign_resident"] = json!(true);
    assert_eq!(withheld(&young), "4500.00");
    let mut old = member(
        "2026-27",
        "2026-10-01",
        "1950-01-01",
        ["0.00", "100000.00", "1000.99"],
    );
    old["tfn_provided"] = json!(false);
    old.as_object_mut().unwrap().remove("untaxed_plan_cap_used");
    assert_eq!(withheld(&old), "470.00");
    let mut dependant = death("death_benefits_dependant", ["0.00", "1000.00", "0.00"]);
    dependant["tfn_provided"] = json!(false);
    assert_eq!(withheld(&dependant), "0.00");
}

#[test]
fn bad_inputs_are_refused() {
    let base = member(
        "2020-21",
        "2020-10-13",
        "1950-01-01",
        ["1000.00", "9000.00", "0.00"],
    );
    assert_eq!(withheld(&base), "0.00");
    let with = |key: &str, value: Value| {
        let mut v = base.clone();
        v[key] = value;
        v
    };
    assert_eq!(
        code(&with("payment_date", json!("2020-10-12"))),
        "super_withholding.schedule_not_read"
    );
    assert_eq!(
        code(&with("recipient", json!("estate"))),
        "super_withholding.recipient_invalid"
    );
    assert_eq!(
        code(&with("tfn_provided", json!("yes"))),
        "input.flag_invalid"
    );
    assert_eq!(
        code(&with("foreign_resident", json!(1))),
        "input.flag_invalid"
    );
    for year in ["2019-20", "2027-28"] {
        assert_eq!(code(&with("year", json!(year))), "period.unsupported");
    }
    assert_eq!(code(&with("unexpected", json!(1))), "input.field_unknown");
    // No TFN: refused where the schedule states no rate, and the caps are
    // not taken.
    let mut nd = death("non_dependant", ["0.00", "1000.00", "0.00"]);
    nd["tfn_provided"] = json!(false);
    assert_eq!(code(&nd), "super_withholding.no_tfn_unstated");
    let mut small = member(
        "2026-27",
        "2026-10-01",
        "1980-01-01",
        ["0.00", "199.99", "0.00"],
    );
    small["tfn_provided"] = json!(false);
    assert_eq!(code(&small), "super_withholding.no_tfn_unstated");
    let mut capped = member(
        "2026-27",
        "2026-10-01",
        "1950-01-01",
        ["0.00", "1000.00", "1000.00"],
    );
    capped["tfn_provided"] = json!(false);
    assert_eq!(code(&capped), "input.field_unknown");
    // Roll-overs: member facts are not taken.
    let tom = rollover("2026-27", "2026-09-01", "1967000.00");
    let mut born = tom.clone();
    born["date_of_birth"] = json!("1960-01-01");
    assert_eq!(code(&born), "input.field_unknown");
    let mut missing = tom;
    missing
        .as_object_mut()
        .unwrap()
        .remove("untaxed_plan_cap_used");
    assert_eq!(code(&missing), "input.missing_field");
    // A deceased estate takes no member facts.
    let mut estate = death("deceased_estate_trustee", ["0.00", "1000.00", "0.00"]);
    estate["untaxed_plan_cap_used"] = json!("0.00");
    assert_eq!(code(&estate), "input.field_unknown");
    // super-benefit-tax's refusals carry over.
    assert_eq!(
        code(&with("disability_superannuation_benefit", json!(true))),
        "super_benefit_tax.disability_benefit"
    );
    let mut service = member(
        "2026-27",
        "2026-10-01",
        "1950-01-01",
        ["0.00", "1000.00", "1000.00"],
    );
    service["service_before_july_1983"] = json!(true);
    assert_eq!(code(&service), "super_benefit_tax.pre_july_1983_service");
}
