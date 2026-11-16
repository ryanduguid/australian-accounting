// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! 2027-28: the worksheets whose every rate and threshold the law already sets
//! for that year accept it, each figure worked by hand in the comments, and
//! the rest refuse it saying what is missing
//! (docs/calculation-evidence.md#income-year-2027-28).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const YEAR: &str = "2027-28";

/// `command`'s result for `facts` in 2027-28; derive explains it byte for byte.
fn run(command: &str, facts: Value) -> Value {
    let mut input = facts;
    input["year"] = json!(YEAR);
    input["scope_confirmed"] = json!(true);
    let result = api::evaluate(command, &input).unwrap();
    assert_eq!(result["period"], YEAR, "{command}");
    let envelope = json!({"command": command, "input": input});
    match api::evaluate_coded("derive", &envelope) {
        Ok(doc) => assert_eq!(doc["result"].to_string(), result.to_string(), "{command}"),
        Err(refusal) => assert_eq!(refusal.code, "derivation.unsupported", "{command}"),
    }
    result
}

fn amount(command: &str, facts: Value, key: &str) -> String {
    run(command, facts)["amounts"][key]
        .as_str()
        .unwrap()
        .to_string()
}

/// The year's warning, the one after the 2 every result carries.
fn year_warning(command: &str, facts: Value) -> String {
    let result = run(command, facts);
    let warnings = result["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 3, "{command}");
    warnings[2].as_str().unwrap().to_string()
}

/// Income Tax Rates Act 1986 Sch 7 Part I clause 1(b), the table for 2027-28
/// and later years: 14% to $45,000, 30% to $135,000, 37% to $190,000 and 45%
/// above, from the $18,200 tax-free threshold.
#[test]
fn resident_tax_uses_the_2027_28_table() {
    for (income, tax) in [
        ("18200", "0.00"),
        ("45000", "3752.00"),   // 14% x 26,800
        ("100000", "20252.00"), // 3,752 + 30% x 55,000
        ("135000", "30752.00"), // 3,752 + 30% x 90,000
        ("190000", "51102.00"), // 30,752 + 37% x 55,000
        ("250000", "78102.00"), // 51,102 + 45% x 60,000
    ] {
        let facts = json!({"taxable_income": income});
        assert_eq!(
            amount("resident-tax", facts, "basic_income_tax"),
            tax,
            "{income}"
        );
    }
    let warning = year_warning("resident-tax", json!({"taxable_income": "100000"}));
    assert!(warning.starts_with("2027-28 uses the table"), "{warning}");
    assert!(warning.contains("s 119-10"), "{warning}");
}

/// Part II: 30% to $135,000, 37% to $190,000 and 45% above, no threshold.
#[test]
fn foreign_resident_tax_uses_part_ii() {
    for (income, tax) in [
        ("100000", "30000.00"), // 30% x 100,000
        ("200000", "65350.00"), // 40,500 + 37% x 55,000 + 45% x 10,000
    ] {
        let facts = json!({"taxable_income": income});
        assert_eq!(
            amount("foreign-resident-tax", facts, "basic_income_tax"),
            tax
        );
    }
    let warning = year_warning("foreign-resident-tax", json!({"taxable_income": "100000"}));
    assert!(warning.contains("16 November 2026"), "{warning}");
}

/// ITAA 1997 s 61-115: $700 less 5 cents a dollar above $37,500, and s 293-20
/// at $250,000 and 15%, neither tied to a year; GST at 10% (GST Act s 9-70).
#[test]
fn year_free_rules_give_their_usual_figures() {
    // 700 - 5% x 2,500 = 575
    let lito = json!({"taxable_income": "40000"});
    assert_eq!(
        amount("low-income-tax-offset", lito, "low_income_tax_offset"),
        "575.00"
    );
    // 15% x (240,000 + 20,000 - 250,000) = 1,500
    let div293 = json!({"division_293_income": "240000.00", "low_tax_contributions": "20000.00"});
    assert_eq!(
        amount("division-293", div293, "division_293_tax"),
        "1500.00"
    );
    let gst = json!({"amount": "1100.00", "gst_inclusive": true});
    assert_eq!(amount("gst", gst, "gst"), "100.00");
}

/// Medicare Levy Act 1986 s 3(1) keeps the 2025-26 threshold amount ($28,011)
/// and phase-in limit ($35,013) for later years.
#[test]
fn medicare_levy_keeps_the_thresholds_enacted_for_later_years() {
    let levy = |income: &str| {
        let facts = json!({"taxable_income": income, "section_9a_adjustment_applies": false});
        amount("medicare-levy", facts, "medicare_levy")
    };
    assert_eq!(levy("28011"), "0.00");
    assert_eq!(levy("30000"), "198.90"); // 10% x (30,000 - 28,011)
    assert_eq!(levy("50000"), "1000.00"); // 2% x 50,000
    let facts = json!({"taxable_income": "30000", "section_9a_adjustment_applies": false});
    let warning = year_warning("medicare-levy", facts);
    assert!(warning.starts_with("2027-28 uses"), "{warning}");
}

/// SISR Schedule 7: 2027-28 has 366 days, so a first year pro-rates over 366.
#[test]
fn pension_minimum_counts_the_leap_day() {
    let pension = |balance: &str, days: i64| {
        let facts = json!({"account_balance": balance, "age": 67, "days": days});
        amount("pension-minimum", facts, "minimum_payment")
    };
    assert_eq!(pension("600000.00", 366), "30000.00"); // 5% x 600,000
    // 5% x 250,000 x 181 / 366 = 6,181.42, to the nearest $10
    assert_eq!(pension("250000.00", 181), "6180.00");
}

/// Regulations s 9(6) with the 14% lowest rate: a single person's threshold
/// is (0.14 x 18,200 + 445 + 2,230 + 37,000 x (0.015 + 0.30 - 0.14)) / 0.315 =
/// 11,698 / 0.315 = 37,136.51, rounded up to 37,137, and the cut-out 37,137 +
/// 2,230 / 0.125 = 54,977.
#[test]
fn seniors_offset_thresholds_follow_the_14_percent_rate() {
    let facts = json!({"rebate_income": "40000", "status": "single"});
    let result = run("seniors-pensioners-tax-offset", facts.clone());
    assert_eq!(result["amounts"]["rebate_threshold"], "37137.00");
    assert_eq!(result["amounts"]["cut_out_threshold"], "54977.00");
    // 2,230 - 0.125 x (40,000 - 37,137) = 1,872.125, rounded up
    assert_eq!(
        result["amounts"]["seniors_pensioners_tax_offset"],
        "1873.00"
    );
    let warning = year_warning("seniors-pensioners-tax-offset", facts);
    assert!(
        warning.contains("14% lowest marginal tax rate"),
        "{warning}"
    );
}

/// Each worksheet that refuses 2027-28 names what is missing.
#[test]
fn the_rest_refuse_2027_28_and_say_why() {
    for (command, facts, why) in [
        (
            "payg-withholding",
            json!({"earnings": "1000", "pay_period": "weekly", "scale": 2}),
            "withholding schedules for payments from 1 July 2027",
        ),
        (
            "medicare-levy-surcharge",
            json!({"taxable_income": "100000", "reportable_fringe_benefits": "0",
                   "income_for_surcharge_purposes": "100000", "days_without_cover": 0}),
            "Medicare levy surcharge income thresholds",
        ),
        (
            "capital-gains",
            json!({"other_gains": "100.00", "discount_gains": "0", "current_losses": "0",
                   "prior_losses": "0"}),
            "Act No. 49 of 2026",
        ),
        (
            "depreciation",
            json!({"cost": "3000.00", "effective_life": "4", "days": 365,
                   "taxable_use": "1", "method": "prime_cost"}),
            "366 days",
        ),
    ] {
        let mut input = facts;
        input["year"] = json!(YEAR);
        input["scope_confirmed"] = json!(true);
        let refusal = api::evaluate_coded(command, &input).unwrap_err();
        assert_eq!(refusal.code, "period.unsupported", "{command}");
        assert!(
            refusal.message.contains(why),
            "{command}: {}",
            refusal.message
        );
    }
}

/// The estimate refuses a resident's 2027-28, whose working Australians tax
/// offset it does not work out, but composes a foreign resident's, who has no
/// such offset (ITAA 1997 s 61-155(1)(a)).
#[test]
fn the_estimate_refuses_a_residents_2027_28() {
    let resident = json!({"year": YEAR, "scope_confirmed": true,
        "residency": "full_year_resident", "taxable_income": "85000",
        "medicare-levy": {"section_9a_adjustment_applies": false,
                          "medicare_levy_exemption_full_days": 365}});
    let refusal = api::evaluate_coded("personal-tax-estimate", &resident).unwrap_err();
    assert_eq!(refusal.code, "period.unsupported");
    assert!(refusal.message.contains("s 61-155"), "{}", refusal.message);
    let foreign = json!({"year": YEAR, "scope_confirmed": true,
        "residency": "foreign_resident", "taxable_income": "100000"});
    let out = api::evaluate("personal-tax-estimate", &foreign).unwrap();
    assert_eq!(out["amounts"]["basic_income_tax"], "30000.00");
}
