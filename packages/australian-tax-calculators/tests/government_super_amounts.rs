// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The super co-contribution, LISTO and the spouse contribution tax offset:
//! the ATO's Angelo, Julie and spouse examples, each year's thresholds, each
//! condition and the refusals
//! (docs/calculation-evidence.md#government-super-amounts). Every figure is the
//! ATO's or worked by hand from the provision in the comment.
use australian_tax_calculators::api;
use australian_tax_calculators::engine::worksheet_catalogue;
use serde_json::{Value, json};

const COMMAND: &str = "government-super-amounts";

/// The ATO's Table 25 (last updated 27 April 2026): lower and higher income
/// thresholds.
const THRESHOLDS: [(&str, i64, i64); 7] = [
    ("2020-21", 39837, 54837),
    ("2021-22", 41112, 56112),
    ("2022-23", 42016, 57016),
    ("2023-24", 43445, 58445),
    ("2024-25", 45400, 60400),
    ("2025-26", 47488, 62488),
    ("2026-27", 49293, 64293),
];

/// A full-year resident in `year` with every component and contribution nil
/// and every other test met, with `given`.
fn input(year: &str, given: Value) -> Value {
    let mut facts = json!({
        "residency": "full_year_resident", "year": year, "scope_confirmed": true,
        "return_lodged": true, "under_71": true, "temporary_visa_held": false,
        "non_concessional_within_cap": true, "total_super_balance_below_cap": true,
    });
    for key in [
        "taxable_income",
        "assessable_fhss_released_amount",
        "family_trust_distribution_amounts",
        "reportable_fringe_benefits_section_57a",
        "reportable_fringe_benefits_other",
        "reportable_employer_super_contributions",
        "deductible_personal_super_contributions",
        "excess_concessional_contributions",
        "net_financial_investment_loss",
        "net_rental_property_loss",
        "tax_free_government_pensions",
        "target_foreign_income",
        "exempt_foreign_employment_income",
        "child_support_paid",
        "assessable_income",
        "business_deductions",
        "eligible_income",
        "personal_contributions",
        "concessional_contributions",
    ] {
        facts[key] = json!("0");
    }
    for (k, v) in given.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

/// Salary of `income` dollars, all of it eligible income.
fn salary(income: i64, more: Value) -> Value {
    let mut facts = json!({"taxable_income": income.to_string(),
        "assessable_income": income.to_string(), "eligible_income": income.to_string()});
    for (k, v) in more.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn evaluate(year: &str, given: Value) -> Value {
    let input = input(year, given);
    let result = api::evaluate(COMMAND, &input).unwrap();
    // derive explains this command: its result is the plain one, byte for byte.
    let doc = api::evaluate("derive", &json!({"command": COMMAND, "input": input})).unwrap();
    assert_eq!(
        doc["result"].to_string(),
        result.to_string(),
        "derive {input}"
    );
    result
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

/// Whether `condition` of `amount_id` was met, not met or not given.
fn met(result: &Value, amount_id: &str, condition: &str) -> Value {
    result["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["amount"] == amount_id && c["condition"] == condition)
        .unwrap_or_else(|| panic!("{amount_id} {condition}"))["met"]
        .clone()
}

fn refusal(year: &str, given: Value) -> (&'static str, String) {
    let refused = api::evaluate_coded(COMMAND, &input(year, given)).unwrap_err();
    (refused.code, refused.message)
}

/// ATO, 'Super co-contribution' (last updated 2 August 2023), read on
/// 22 November 2026: Angelo earns $35,000 and pays $1,040 of personal
/// contributions; he is eligible for the maximum co-contribution of $500. His
/// income is under the lower threshold in every supported year.
#[test]
fn ato_angelo_gets_the_maximum_co_contribution_in_every_year() {
    for (year, lower, higher) in THRESHOLDS {
        let result = evaluate(
            year,
            salary(35000, json!({"personal_contributions": "1040"})),
        );
        assert_eq!(amount(&result, "co_contribution_total_income"), "35000.00");
        // 50% of $1,040 (s 9(1)(d)), above the $500 maximum (s 10(1C)).
        assert_eq!(amount(&result, "co_contribution_matched"), "520.00");
        assert_eq!(amount(&result, "co_contribution_maximum"), "500.00");
        assert_eq!(amount(&result, "co_contribution"), "500.00", "{year}");
        assert_eq!(amount(&result, "co_contribution_paid"), "500.00");
        assert_eq!(amount(&result, "listo"), "0.00");
        assert_eq!(result["rates"]["lower_income_threshold"], lower.to_string());
        assert_eq!(
            result["rates"]["higher_income_threshold"],
            higher.to_string()
        );
        // s 10A(3)(d): the higher threshold is $15,000 above the lower.
        assert_eq!(higher - lower, 15000);
        assert!(
            result["amounts"]
                .get("spouse_contribution_offset")
                .is_none()
        );
    }
}

/// ATO, 'Low income super tax offset' (last updated 2 August 2023), read on
/// 22 November 2026: Julie earns $32,000 in 2022-23, has $1,000 of
/// deductions, so adjusted taxable income of $31,000, and her employer pays
/// $3,360 of super guarantee; 15% of $3,360 is more than $500, so her LISTO is
/// $500.
#[test]
fn ato_julie_gets_the_maximum_listo() {
    let result = evaluate(
        "2022-23",
        json!({"taxable_income": "31000", "assessable_income": "32000",
               "eligible_income": "32000", "concessional_contributions": "3360"}),
    );
    assert_eq!(amount(&result, "adjusted_taxable_income"), "31000.00");
    assert_eq!(amount(&result, "listo_15_percent"), "504.00");
    assert_eq!(amount(&result, "listo"), "500.00");
    assert_eq!(amount(&result, "listo_paid"), "500.00");
    // No personal contributions: no co-contribution (s 6(1)(a)).
    assert_eq!(amount(&result, "co_contribution"), "0.00");
    assert_eq!(
        met(&result, "co_contribution", "personal_contributions_made"),
        false
    );
    assert_eq!(met(&result, "listo", "adjusted_taxable_income_37000"), true);
}

/// The spouse's income in `year` and the contributions for the spouse.
fn spouse(income: &str, contributions: &str) -> Value {
    json!({"spouse_contributions": contributions, "spouse_assessable_income": income,
           "spouse_assessable_fhss_released_amount": "0",
           "spouse_reportable_fringe_benefits_total": "0",
           "spouse_reportable_employer_super_contributions": "0",
           "spouse_excess_concessional_contributions": "0",
           "spouse_non_concessional_within_cap": true,
           "spouse_total_super_balance_below_cap": true})
}

/// ATO, 'Superannuation-related tax offsets' (last updated 8 June 2026),
/// read on 22 November 2026, Examples 1 to 3: Robert earns $19,000 in 2021-22
/// and Judy contributes $3,500, $540; Carmel earns $38,000 in 2021-22 and Adam
/// contributes $3,000, 18% of $2,000, $360; Aleah earns $35,000 in 2022-23 and
/// Lewis contributes $2,000, $360.
#[test]
fn ato_spouse_examples() {
    for (year, income, contributions, limit, offset) in [
        ("2021-22", "19000", "3500", "3000.00", "540.00"),
        ("2021-22", "38000", "3000", "2000.00", "360.00"),
        ("2022-23", "35000", "2000", "3000.00", "360.00"),
    ] {
        let result = evaluate(year, spouse(income, contributions));
        assert_eq!(amount(&result, "spouse_income"), format!("{income}.00"));
        assert_eq!(amount(&result, "spouse_contribution_limit"), limit);
        assert_eq!(
            amount(&result, "spouse_contribution_offset"),
            offset,
            "{income}"
        );
    }
}

/// s 10(1C): the maximum falls by 3.333 cents a dollar above the lower
/// threshold; s 6(1)(c): nothing at the higher; s 11: at least $20; reg 15(2):
/// paid up to a multiple of 5 cents. Each year's thresholds, at $1,000 of
/// contributions.
#[test]
fn each_years_thresholds() {
    for (year, lower, higher) in THRESHOLDS {
        let at = |income: i64| {
            evaluate(
                year,
                salary(income, json!({"personal_contributions": "1000"})),
            )
        };
        let result = at(lower);
        assert_eq!(amount(&result, "co_contribution"), "500.00", "{year}");
        // 500 - 0.03333 = 499.96667.
        let result = at(lower + 1);
        assert_eq!(amount(&result, "co_contribution_maximum"), "499.97");
        assert_eq!(amount(&result, "co_contribution"), "499.97");
        assert_eq!(amount(&result, "co_contribution_paid"), "500.00");
        // 500 - 14,999 x 0.03333 = 0.08333, raised to $20.
        let result = at(higher - 1);
        assert_eq!(amount(&result, "co_contribution_maximum"), "0.08");
        assert_eq!(amount(&result, "co_contribution"), "20.00");
        let result = at(higher);
        assert_eq!(
            met(&result, "co_contribution", "below_higher_income_threshold"),
            false
        );
        assert_eq!(amount(&result, "co_contribution"), "0.00");
        assert_eq!(amount(&result, "co_contribution_paid"), "0.00");
    }
}

/// 2025-26, total income $50,000: $500 - 2,512 x 0.03333 = $416.27504, paid
/// as $416.30.
#[test]
fn taper_and_payment_rounding() {
    let result = evaluate(
        "2025-26",
        salary(50000, json!({"personal_contributions": "1000"})),
    );
    assert_eq!(amount(&result, "co_contribution_taper"), "83.72");
    assert_eq!(amount(&result, "co_contribution"), "416.28");
    assert_eq!(amount(&result, "co_contribution_paid"), "416.30");
    // s 11: 50% of $10 is $5, raised to $20.
    let result = evaluate(
        "2025-26",
        salary(30000, json!({"personal_contributions": "10"})),
    );
    assert_eq!(amount(&result, "co_contribution_matched"), "5.00");
    assert_eq!(amount(&result, "co_contribution"), "20.00");
}

/// s 6(1)(b): 10% or more of total income from employment or business; s 8(2)
/// takes business deductions off total income, and s 8(3) leaves them on for
/// the 10% test.
#[test]
fn ten_percent_eligible_income_test() {
    for (eligible, ok) in [("4999", false), ("5000", true)] {
        let result = evaluate(
            "2025-26",
            json!({"taxable_income": "50000", "assessable_income": "50000",
                   "eligible_income": eligible, "personal_contributions": "1000"}),
        );
        assert_eq!(
            met(&result, "co_contribution", "eligible_income_10_percent"),
            ok
        );
        assert_eq!(met(&result, "listo", "eligible_income_10_percent"), ok);
        let expected = if ok { "416.28" } else { "0.00" };
        assert_eq!(amount(&result, "co_contribution"), expected);
    }
    let result = evaluate(
        "2025-26",
        json!({"taxable_income": "40000", "assessable_income": "60000",
               "business_deductions": "20000", "eligible_income": "6000",
               "personal_contributions": "1000"}),
    );
    assert_eq!(amount(&result, "co_contribution_total_income"), "40000.00");
    assert_eq!(amount(&result, "eligible_income_test_total"), "60000.00");
    assert_eq!(amount(&result, "co_contribution"), "500.00");
    let mut short = input("2025-26", json!({}));
    short["taxable_income"] = json!("40000");
    short["assessable_income"] = json!("60000");
    short["business_deductions"] = json!("20000");
    short["eligible_income"] = json!("5999");
    short["personal_contributions"] = json!("1000");
    let result = api::evaluate(COMMAND, &short).unwrap();
    assert_eq!(
        met(&result, "co_contribution", "eligible_income_10_percent"),
        false
    );
}

/// The stated tests: each not met gives no co-contribution.
#[test]
fn each_stated_test() {
    for key in [
        "return_lodged",
        "under_71",
        "non_concessional_within_cap",
        "total_super_balance_below_cap",
    ] {
        let mut given = salary(30000, json!({"personal_contributions": "1000"}));
        given[key] = json!(false);
        let result = evaluate("2024-25", given);
        assert_eq!(met(&result, "co_contribution", key), false);
        assert_eq!(amount(&result, "co_contribution"), "0.00", "{key}");
    }
}

/// s 12C(1)(b) and s 12E(2): adjusted taxable income of $37,000 or less; 15%
/// to the cent, at least $10, paid up to 5 cents.
#[test]
fn listo_limit_minimum_and_rounding() {
    for (income, ok) in [(37000, true), (37001, false)] {
        let result = evaluate(
            "2025-26",
            salary(income, json!({"concessional_contributions": "4000"})),
        );
        assert_eq!(met(&result, "listo", "adjusted_taxable_income_37000"), ok);
        assert_eq!(amount(&result, "listo"), if ok { "500.00" } else { "0.00" });
    }
    let result = evaluate(
        "2025-26",
        salary(20000, json!({"concessional_contributions": "50"})),
    );
    assert_eq!(amount(&result, "listo_15_percent"), "7.50");
    assert_eq!(amount(&result, "listo"), "10.00");
    let result = evaluate(
        "2025-26",
        salary(20000, json!({"concessional_contributions": "1234.57"})),
    );
    // 15% is 185.1855: 185.19 to the cent, paid as 185.20.
    assert_eq!(amount(&result, "listo"), "185.19");
    assert_eq!(amount(&result, "listo_paid"), "185.20");
    // No concessional contributions: no LISTO.
    let result = evaluate("2025-26", salary(20000, json!({})));
    assert_eq!(
        met(&result, "listo", "concessional_contributions_made"),
        false
    );
    assert_eq!(amount(&result, "listo"), "0.00");
}

/// ss 290-230(2)(c), (4A) and (5) and 290-235(1)(a).
#[test]
fn spouse_limits() {
    let result = evaluate("2025-26", spouse("39999", "3000"));
    assert_eq!(amount(&result, "spouse_contribution_limit"), "1.00");
    assert_eq!(amount(&result, "spouse_contribution_offset"), "0.18");
    let result = evaluate("2025-26", spouse("40000", "3000"));
    assert_eq!(
        met(
            &result,
            "spouse_contribution_offset",
            "spouse_income_below_40000"
        ),
        false
    );
    assert_eq!(amount(&result, "spouse_contribution_offset"), "0.00");
    // Employer contributions less excess concessional contributions, less the
    // FHSS amount, plus fringe benefits: 30,000 - 2,000 + 3,000 + (5,000 - 1,000).
    let mut given = spouse("30000", "3000");
    given["spouse_assessable_fhss_released_amount"] = json!("2000");
    given["spouse_reportable_fringe_benefits_total"] = json!("3000");
    given["spouse_reportable_employer_super_contributions"] = json!("5000");
    given["spouse_excess_concessional_contributions"] = json!("1000");
    let result = evaluate("2025-26", given);
    assert_eq!(amount(&result, "spouse_income"), "35000.00");
    for key in [
        "spouse_non_concessional_within_cap",
        "spouse_total_super_balance_below_cap",
    ] {
        let mut given = spouse("20000", "3000");
        given[key] = json!(false);
        let result = evaluate("2025-26", given);
        assert_eq!(
            amount(&result, "spouse_contribution_offset"),
            "0.00",
            "{key}"
        );
    }
}

#[test]
fn refusals() {
    let base = || salary(30000, json!({"personal_contributions": "1000"}));
    let mut visa = base();
    visa["temporary_visa_held"] = json!(true);
    assert_eq!(
        refusal("2025-26", visa).0,
        "government_super_amounts.temporary_visa"
    );
    // A balance fact not established, where it decides the amount.
    let mut unknown = base();
    unknown["total_super_balance_below_cap"] = Value::Null;
    let (code, message) = refusal("2025-26", unknown);
    assert_eq!(code, "government_super_amounts.fact_not_supplied");
    assert!(message.starts_with("total_super_balance_below_cap decides the co_contribution"));
    // Not where another test already decides it.
    let mut decided = salary(30000, json!({}));
    decided["total_super_balance_below_cap"] = Value::Null;
    decided["non_concessional_within_cap"] = Value::Null;
    let result = evaluate("2025-26", decided);
    assert_eq!(
        met(&result, "co_contribution", "total_super_balance_below_cap"),
        Value::Null
    );
    let mut spouse_unknown = spouse("20000", "3000");
    spouse_unknown["spouse_total_super_balance_below_cap"] = Value::Null;
    assert_eq!(
        refusal("2025-26", spouse_unknown).0,
        "government_super_amounts.fact_not_supplied"
    );
    let mut too_much = base();
    too_much["eligible_income"] = json!("30001");
    assert_eq!(
        refusal("2025-26", too_much).0,
        "government_super_amounts.fact_conflict"
    );
    let mut fhss = spouse("1000", "3000");
    fhss["spouse_assessable_fhss_released_amount"] = json!("1001");
    assert_eq!(
        refusal("2025-26", fhss).0,
        "government_super_amounts.fact_conflict"
    );
    assert_eq!(
        refusal("2025-26", json!({"personal_contributions": "100"})).0,
        "government_super_amounts.no_total_income"
    );
    // Nothing to decide: no refusal.
    assert_eq!(
        amount(&evaluate("2025-26", json!({})), "co_contribution"),
        "0.00"
    );
    let mut partial = spouse("20000", "3000");
    partial
        .as_object_mut()
        .unwrap()
        .remove("spouse_excess_concessional_contributions");
    let (code, message) = refusal("2025-26", partial);
    assert_eq!(code, "input.missing_field");
    assert!(message.starts_with("spouse_excess_concessional_contributions is required"));
    let mut part_year = base();
    part_year["residency"] = json!("part_year_resident");
    assert_eq!(
        refusal("2025-26", part_year).0,
        "income_tests.residency_unsupported"
    );
    for year in ["2019-20", "2027-28"] {
        let (code, message) = refusal(year, base());
        assert_eq!(code, "period.unsupported", "{year}");
        if year == "2027-28" {
            assert!(message.contains("Act No. 8 of 2026"), "{message}");
        }
    }
    let mut unconfirmed = base();
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(refusal("2025-26", unconfirmed).0, "scope.not_confirmed");
    let mut unknown_field = base();
    unknown_field["spouse_age"] = json!("40");
    assert_eq!(refusal("2025-26", unknown_field).0, "input.field_unknown");
    let mut flag = base();
    flag["under_71"] = json!("yes");
    assert_eq!(refusal("2025-26", flag).0, "input.flag_invalid");
}

#[test]
fn catalogue_lists_the_worksheet_and_its_example_runs() {
    let catalogue = worksheet_catalogue();
    let entry = &catalogue["government_super_amounts"];
    let years: Vec<&str> = entry["supported_periods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(years, THRESHOLDS.map(|(y, ..)| y));
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(amount(&result, "co_contribution"), "500.00");
}
