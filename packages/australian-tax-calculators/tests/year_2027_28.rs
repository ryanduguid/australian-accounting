// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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
                   "income_for_surcharge_purposes": "100000", "days_without_cover": 30}),
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

/// Hire purchase (ITAA 1997 Division 240) sets no figure by year, and Act No.
/// 49 of 2026 does not change it, so its 2027-28 refusal names no rental rule.
#[test]
fn hire_purchase_refuses_2027_28_without_a_rental_reason() {
    let input = json!({"amount_financed": "45000.00", "instalment": "850.00",
        "instalments": 60, "balloon": "5000.00", "timing": "in_arrears",
        "start_date": "2027-07-15", "year": YEAR, "scope_confirmed": true});
    let refusal = api::evaluate_coded("hire-purchase-schedule", &input).unwrap_err();
    assert_eq!(refusal.code, "period.unsupported");
    assert_eq!(
        refusal.message,
        "Unsupported period '2027-28'; supported: 2024-25, 2025-26, 2026-27."
    );
}

/// Medicare Levy Act 1986 ss 8B(1)(c), 8C(1)(c) and 8D(1)(b), and the Fringe
/// Benefits Act ss 12 to 14, apply only for a period without private patient
/// hospital cover: with no such day the surcharge is nil whatever 2027-28's
/// unpublished thresholds, and with one the year is refused.
#[test]
fn the_surcharge_is_nil_in_2027_28_with_no_day_without_cover() {
    let facts = |days: i64| {
        json!({"taxable_income": "200000", "reportable_fringe_benefits": "5000",
            "income_for_surcharge_purposes": "205000", "days_without_cover": days})
    };
    assert_eq!(
        amount(
            "medicare-levy-surcharge",
            facts(0),
            "medicare_levy_surcharge"
        ),
        "0.00"
    );
    // days_without_cover counts a day any of a family went without cover.
    let mut family = facts(0);
    family["spouse_income_for_surcharge_purposes"] = json!("150000");
    family["dependent_children"] = json!(2);
    assert_eq!(
        amount("medicare-levy-surcharge", family, "medicare_levy_surcharge"),
        "0.00"
    );
    let mut input = facts(1);
    input["year"] = json!(YEAR);
    input["scope_confirmed"] = json!(true);
    let refusal = api::evaluate_coded("medicare-levy-surcharge", &input).unwrap_err();
    assert_eq!(refusal.code, "period.unsupported");
    assert_eq!(
        refusal.message,
        "2027-28 is supported only with days_without_cover 0, when no surcharge applies. The 2027-28 Medicare levy surcharge income thresholds had not been published when this was checked on 16 November 2026."
    );
}

/// A full-year resident with hospital cover all year, as the estimate takes one.
fn resident(extra: Value) -> Value {
    let mut facts = json!({"year": YEAR, "scope_confirmed": true,
        "residency": "full_year_resident",
        "medicare-levy": {"section_9a_adjustment_applies": false},
        "medicare-levy-surcharge": {"reportable_fringe_benefits": "0",
            "income_for_surcharge_purposes": "0", "days_without_cover": 0}});
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn estimate(extra: Value) -> Value {
    api::evaluate("personal-tax-estimate", &resident(extra)).unwrap()
}

/// ITAA 1997 ss 61-155 and 61-160 (Act No. 49 of 2026 Sch 3, for 2027-28 and
/// later years), applied first (s 63-10(1) item 3). $85,000 of salary, $500
/// of other income, $300 of work expenses and $200 of other deductions: the
/// standard deduction is $1,000 - $300 = $700, taxable income $84,300 and net
/// labour income $85,000 - $300 - $700 = $84,000, whose tax, $3,752 + 30% of
/// $39,000 = $15,452, exceeds $250. Basic tax is $3,752 + 30% of $39,300 =
/// $15,542, less $250 is $15,292; no LITO above $66,667; the levy is 2% of
/// $84,300, $1,686; the total $16,978, against $17,000 withheld a $22 refund.
#[test]
fn a_residents_2027_28_estimate_applies_the_working_australians_tax_offset() {
    let out = estimate(json!({"assessable_labour_income": "85000",
        "other_assessable_income": "500", "work_related_deductions": "300",
        "other_deductions": "200", "tax_withheld": "17000"}));
    let a = &out["amounts"];
    for (key, expected) in [
        ("standard_deduction", "700.00"),
        ("taxable_income", "84300.00"),
        ("basic_income_tax", "15542.00"),
        ("net_labour_income", "84000.00"),
        ("working_australians_tax_offset", "250.00"),
        ("working_australians_tax_offset_applied", "250.00"),
        ("seniors_pensioners_tax_offset_applied", "0.00"),
        ("low_income_tax_offset_applied", "0.00"),
        ("income_tax_after_offsets", "15292.00"),
        ("medicare_levy", "1686.00"),
        ("medicare_levy_surcharge", "0.00"),
        ("total_before_credits", "16978.00"),
        ("refund", "22.00"),
    ] {
        assert_eq!(a[key], expected, "{key}");
    }
    let keys: Vec<&String> = a.as_object().unwrap().keys().collect();
    let working: Vec<&String> = out["working"].as_object().unwrap().keys().collect();
    assert_eq!(keys, working);
    let at = keys.iter().position(|k| *k == "basic_income_tax").unwrap();
    assert_eq!(
        keys[at + 1..at + 4],
        [
            "net_labour_income",
            "working_australians_tax_offset",
            "working_australians_tax_offset_applied"
        ]
    );
    let line = out["working"]["net_labour_income"].as_str().unwrap();
    assert!(
        line.starts_with(
            "assessable_labour_income - work_related_deductions - standard_deduction:"
        )
    );
    let sources = out["sources"].as_array().unwrap();
    assert!(
        sources
            .iter()
            .any(|s| s.as_str().unwrap().ends_with("/61-160"))
    );
}

/// The offset is the tax on net labour income where that is under $250: on
/// $19,000, 14% of $800 = $112, and on $18,201, 14 cents (s 61-160(b)); none
/// at the $18,200 tax-free threshold (s 61-155(1)(b)). It comes off $30,000's
/// basic tax of 14% of $11,800 = $1,652 before LITO's $700: $840 remains, and
/// the levy is 10% of $1,989, $198.90.
#[test]
fn below_250_the_offset_is_the_tax_on_net_labour_income() {
    for (net, offset) in [("19000", "112.00"), ("18201", "0.14"), ("18200", "0.00")] {
        let out = estimate(json!({"taxable_income": "30000", "net_labour_income": net}));
        let a = &out["amounts"];
        assert_eq!(a["working_australians_tax_offset"], offset, "{net}");
        assert_eq!(a["working_australians_tax_offset_applied"], offset, "{net}");
        assert_eq!(a["low_income_tax_offset_applied"], "700.00", "{net}");
        let line = out["working"]["net_labour_income"].as_str().unwrap();
        assert!(line.ends_with("as established"), "{line}");
    }
    let out = estimate(json!({"taxable_income": "30000", "net_labour_income": "19000"}));
    let a = &out["amounts"];
    assert_eq!(a["income_tax_after_offsets"], "840.00");
    assert_eq!(a["medicare_levy"], "198.90");
    assert_eq!(a["total_before_credits"], "1038.90");
}

/// Applied first, the offset is limited by basic tax and leaves SAPTO what is
/// left: taxable income $30,000 and net labour income $20,000 (14% of $1,800 =
/// $252, so $250); basic tax $1,652 less $250 leaves $1,402, all SAPTO's
/// $2,230 can take (rebate income $30,000, under the $37,137 threshold), and
/// nothing for LITO. With taxable income $19,000 and net labour income
/// $30,000, the offset is limited to the basic tax of $112.
#[test]
fn the_offset_comes_before_sapto_and_is_limited_by_basic_tax() {
    let sapto = json!({"rebate_income": "30000", "status": "single"});
    let out = estimate(
        json!({"taxable_income": "30000", "net_labour_income": "20000",
        "seniors-pensioners-tax-offset": sapto}),
    );
    let a = &out["amounts"];
    assert_eq!(a["working_australians_tax_offset_applied"], "250.00");
    assert_eq!(a["seniors_pensioners_tax_offset_applied"], "1402.00");
    assert_eq!(a["low_income_tax_offset_applied"], "0.00");
    assert_eq!(a["income_tax_after_offsets"], "0.00");
    let out = estimate(json!({"taxable_income": "19000", "net_labour_income": "30000"}));
    let a = &out["amounts"];
    assert_eq!(a["working_australians_tax_offset"], "250.00");
    assert_eq!(a["working_australians_tax_offset_applied"], "112.00");
    assert_eq!(a["low_income_tax_offset_applied"], "0.00");
    assert_eq!(a["total_before_credits"], "0.00");
}

/// net_labour_income is required beside taxable_income for a resident from
/// 2027-28 and refused where the offset does not apply; a day without cover
/// is refused through the surcharge part; a foreign resident is composed
/// without the offset (ITAA 1997 s 61-155(1)(a)).
#[test]
fn the_estimates_2027_28_refusals() {
    let refusal = |facts: Value| api::evaluate_coded("personal-tax-estimate", &facts).unwrap_err();
    let r = refusal(resident(json!({"taxable_income": "85000"})));
    assert_eq!(r.code, "input.missing_field");
    assert_eq!(
        r.message,
        "net_labour_income is required with taxable_income for a resident from 2027-28: the working Australians tax offset (ITAA 1997 s 61-160) is worked out on it"
    );
    let mut earlier = resident(json!({"taxable_income": "85000", "net_labour_income": "85000"}));
    earlier["year"] = json!("2026-27");
    let foreign = json!({"year": YEAR, "scope_confirmed": true,
        "residency": "foreign_resident", "taxable_income": "100000",
        "net_labour_income": "100000"});
    for facts in [earlier, foreign] {
        let r = refusal(facts);
        assert_eq!(r.code, "input.field_unknown");
        assert_eq!(
            r.message,
            "net_labour_income is not an input of personal-tax-estimate here: the working Australians tax offset (ITAA 1997 s 61-155) is for a resident from 2027-28"
        );
    }
    let surcharge = json!({"reportable_fringe_benefits": "0",
        "income_for_surcharge_purposes": "85000", "days_without_cover": 10});
    let r = refusal(resident(
        json!({"taxable_income": "85000", "net_labour_income": "85000",
        "medicare-levy-surcharge": surcharge}),
    ));
    assert_eq!(r.code, "period.unsupported");
    assert!(
        r.message
            .starts_with("medicare-levy-surcharge: 2027-28 is supported only"),
        "{}",
        r.message
    );
    let foreign = json!({"year": YEAR, "scope_confirmed": true,
        "residency": "foreign_resident", "taxable_income": "100000"});
    let out = api::evaluate("personal-tax-estimate", &foreign).unwrap();
    assert_eq!(out["amounts"]["basic_income_tax"], "30000.00");
    assert!(
        out["amounts"]
            .get("working_australians_tax_offset")
            .is_none()
    );
}

/// Net labour income is labour amounts less labour deductions with no floor
/// (ITAA 1997 s 61-155(2)): $20,000 of salary and $21,000 of work expenses
/// leave -$1,000 and no offset. A given net_labour_income stands for labour
/// income held in other_assessable_income: $20,000 gives 14% of $1,800, $252,
/// so the full $250, where the labour inputs alone would give 14% of $800.
#[test]
fn net_labour_income_is_signed_and_a_given_one_is_taken() {
    let out = estimate(json!({"assessable_labour_income": "20000",
        "work_related_deductions": "21000", "other_assessable_income": "50000"}));
    let a = &out["amounts"];
    assert_eq!(a["net_labour_income"], "-1000.00");
    assert_eq!(a["working_australians_tax_offset"], "0.00");
    assert_eq!(a["working_australians_tax_offset_applied"], "0.00");
    let labour = json!({"assessable_labour_income": "20000", "other_assessable_income": "1000"});
    let derived = estimate(labour.clone());
    assert_eq!(
        derived["amounts"]["working_australians_tax_offset"],
        "112.00"
    );
    let mut given = labour;
    given["net_labour_income"] = json!("20000");
    let out = estimate(given);
    assert_eq!(out["amounts"]["net_labour_income"], "20000.00");
    assert_eq!(out["amounts"]["working_australians_tax_offset"], "250.00");
    // A given labour loss is reported as it is, with no offset.
    let loss = json!({"taxable_income": "49000", "net_labour_income": "-1000"});
    let a = &estimate(loss)["amounts"];
    assert_eq!(a["net_labour_income"], "-1000.00");
    assert_eq!(a["working_australians_tax_offset"], "0.00");
    let cents = resident(json!({"taxable_income": "49000", "net_labour_income": "-1000.50"}));
    let r = api::evaluate_coded("personal-tax-estimate", &cents).unwrap_err();
    assert_eq!(r.code, "input.amount_not_whole_dollars");
}

/// A regular pay cannot stand for 2027-28's withholding: the ATO had not
/// published the schedules, so the estimate refuses with no partial result.
#[test]
fn a_regular_pay_in_2027_28_is_refused() {
    let facts = resident(json!({"assessable_labour_income": "85000",
        "payg-withholding": {"earnings": "3269.23", "pay_period": "fortnightly", "scale": 2},
        "pays_in_year": 26}));
    let r = api::evaluate_coded("personal-tax-estimate", &facts).unwrap_err();
    assert_eq!(r.code, "period.unsupported");
    assert!(
        r.message
            .starts_with("payg-withholding: Unsupported period '2027-28'"),
        "{}",
        r.message
    );
}
