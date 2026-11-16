// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The income measures: the ATO's Christina and Tom examples, each measure
//! fed to the command that takes it, the adjustments each definition makes,
//! the components each measure leaves out and the refusals
//! (docs/calculation-evidence.md#income-tests). Every figure is the ATO's or
//! worked by hand from the provision in the comment.
use australian_tax_calculators::api;
use australian_tax_calculators::engine::worksheet_catalogue;
use serde_json::{Value, json};

const COMMAND: &str = "income-tests";

const YEARS: [&str; 3] = ["2024-25", "2025-26", "2026-27"];

/// Every component nil, for a full-year resident in `year`, with `given`.
fn input(year: &str, given: Value) -> Value {
    let mut facts =
        json!({"residency": "full_year_resident", "year": year, "scope_confirmed": true});
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
    ] {
        facts[key] = json!("0");
    }
    for (k, v) in given.as_object().unwrap() {
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

fn refusal(facts: Value) -> (&'static str, String) {
    let refused = api::evaluate_coded(COMMAND, &facts).unwrap_err();
    (refused.code, refused.message)
}

/// ATO, 'When you must repay your loan' (last updated 13 October 2026), read
/// on 16 November 2026: Christina's taxable income of $50,420, total net
/// investment loss of $1,250, reportable fringe benefits of $4,560, exempt
/// foreign employment income of $2,580 and reportable super contributions of
/// $15,000 give repayment income of $73,810. The split of the loss, the
/// fringe benefits and the contributions, her target foreign income (her
/// exempt foreign employment income alone) and her assessable income are
/// fabricated. The definitions are the same in every supported year.
fn christina() -> Value {
    json!({"taxable_income": "50420", "net_financial_investment_loss": "1250",
           "reportable_fringe_benefits_other": "4560",
           "exempt_foreign_employment_income": "2580", "target_foreign_income": "2580",
           "reportable_employer_super_contributions": "15000",
           "assessable_income": "50420"})
}

#[test]
fn ato_christina_repayment_income_in_every_year() {
    for year in YEARS {
        let result = evaluate(year, christina());
        assert_eq!(
            result["amounts"],
            json!({
                "taxable_income_without_fhss": "50420.00",
                "reportable_fringe_benefits_total": "4560.00",
                "adjusted_fringe_benefits_total": "4560.00",
                "total_net_investment_loss": "1250.00",
                "reportable_superannuation_contributions": "15000.00",
                // 50,420 + 4,560 + 15,000 + 1,250.
                "income_for_surcharge_purposes": "71230.00",
                // Less the 15,000 contributions (s 293-20(1)(a)).
                "division_293_income": "56230.00",
                // 50,420 + 15,000 + 1,250 + 4,560.
                "rebate_income": "71230.00",
                "repayment_income": "73810.00",
                // 50,420 + 4,560 + 2,580 + 1,250 + 15,000.
                "adjusted_taxable_income": "73810.00",
                "co_contribution_employer_contributions": "15000.00",
                // 50,420 + 4,560 + 15,000.
                "co_contribution_total_income": "69980.00",
            }),
            "{year}"
        );
        assert_eq!(result["calculation"], "income_tests");
        assert_eq!(result["period"], year);
        assert_eq!(result["rates"], json!({"section_57a_factor": "0.53"}));
    }
}

/// The repayment income feeds `study-loan-repayment`: for 2025-26, 15% of
/// the $6,810 above the $67,000 minimum repayment income (HESA s 154-10 and
/// s 154-20(2)), $1,021.50.
#[test]
fn christinas_repayment_income_feeds_study_loan_repayment() {
    let result = evaluate("2025-26", christina());
    let repayment = api::evaluate(
        "study-loan-repayment",
        &json!({"repayment_income": result["amounts"]["repayment_income"],
                "year": "2025-26", "scope_confirmed": true}),
    )
    .unwrap();
    assert_eq!(repayment["amounts"]["compulsory_repayment"], "1021.50");
}

/// ATO, 'Medicare levy surcharge income, thresholds and rates' (last updated
/// 22 June 2026), read on 16 November 2026: Tom's $90,000 taxable income and
/// $27,000 total reportable fringe benefits give income for MLS purposes of
/// $117,000 for 2026-27, and `medicare-levy-surcharge` then gives the page's
/// $1,170 at the 1% tier. Which employer gave the benefits is fabricated.
#[test]
fn ato_tom_income_for_surcharge_purposes_feeds_the_surcharge() {
    let result = evaluate(
        "2026-27",
        json!({"taxable_income": "90000", "reportable_fringe_benefits_other": "27000",
               "assessable_income": "90000"}),
    );
    assert_eq!(
        amount(&result, "income_for_surcharge_purposes"),
        "117000.00"
    );
    let surcharge = api::evaluate(
        "medicare-levy-surcharge",
        &json!({"taxable_income": "90000",
                "reportable_fringe_benefits": result["amounts"]["reportable_fringe_benefits_total"],
                "income_for_surcharge_purposes": result["amounts"]["income_for_surcharge_purposes"],
                "days_without_cover": 365, "year": "2026-27", "scope_confirmed": true}),
    )
    .unwrap();
    assert_eq!(surcharge["amounts"]["medicare_levy_surcharge"], "1170.00");
}

/// ATO, 'Division 293 tax on concessional contributions by high-income
/// earners' (last updated 24 August 2026): Jan's Division 293 income of
/// $240,000 and $15,000 of contributions give $750. The components are
/// fabricated: $200,000 taxable income, $25,000 of fringe benefits, a $15,000
/// net financial investment loss and $20,000 of reportable employer
/// contributions, which income for surcharge purposes counts and Division 293
/// income disregards (s 293-20(1)(a)).
#[test]
fn division_293_income_feeds_division_293() {
    let result = evaluate(
        "2025-26",
        json!({"taxable_income": "200000", "reportable_fringe_benefits_other": "25000",
               "net_financial_investment_loss": "15000",
               "reportable_employer_super_contributions": "20000",
               "assessable_income": "230000"}),
    );
    assert_eq!(
        amount(&result, "income_for_surcharge_purposes"),
        "260000.00"
    );
    assert_eq!(amount(&result, "division_293_income"), "240000.00");
    let tax = api::evaluate(
        "division-293",
        &json!({"division_293_income": result["amounts"]["division_293_income"],
                "low_tax_contributions": "15000", "year": "2025-26", "scope_confirmed": true}),
    )
    .unwrap();
    assert_eq!(tax["amounts"]["division_293_tax"], "750.00");
}

/// Family Assistance Act Sch 3 cl 4 and the ATO's rebate income and adjusted
/// taxable income worksheets (2025 and 2026): a section 57A employer's
/// $4,567 counts as $4,567 x 0.53 = $2,420.51, rounded down to $2,420, and
/// another employer's $1,000 in full, $3,420 in rebate income and adjusted
/// taxable income. The surcharge, repayment and co-contribution measures take
/// the reportable fringe benefits total, $5,567 (FBTAA s 135N).
#[test]
fn section_57a_benefits_count_at_53_percent_rounded_down_where_adjusted() {
    let result = evaluate(
        "2025-26",
        json!({"taxable_income": "40000", "assessable_income": "40000",
               "reportable_fringe_benefits_section_57a": "4567",
               "reportable_fringe_benefits_other": "1000"}),
    );
    assert_eq!(
        amount(&result, "reportable_fringe_benefits_total"),
        "5567.00"
    );
    assert_eq!(amount(&result, "adjusted_fringe_benefits_total"), "3420.00");
    assert_eq!(amount(&result, "rebate_income"), "43420.00");
    assert_eq!(amount(&result, "adjusted_taxable_income"), "43420.00");
    assert_eq!(amount(&result, "income_for_surcharge_purposes"), "45567.00");
    assert_eq!(amount(&result, "repayment_income"), "45567.00");
    assert_eq!(amount(&result, "co_contribution_total_income"), "45567.00");
}

/// ITAA 1997 s 995-1 reduces reportable super contributions, and the
/// co-contribution Act s 8(1A) reportable employer contributions, by excess
/// concessional contributions, but not below zero.
#[test]
fn excess_concessional_contributions_reduce_the_contributions_to_nil_at_most() {
    let given = |excess: &str| {
        json!({"taxable_income": "60000", "assessable_income": "63000",
               "reportable_employer_super_contributions": "10000",
               "deductible_personal_super_contributions": "5000",
               "excess_concessional_contributions": excess})
    };
    let some = evaluate("2025-26", given("3000"));
    assert_eq!(
        amount(&some, "reportable_superannuation_contributions"),
        "12000.00"
    );
    assert_eq!(
        amount(&some, "co_contribution_employer_contributions"),
        "7000.00"
    );
    // 63,000 + 7,000.
    assert_eq!(amount(&some, "co_contribution_total_income"), "70000.00");
    let all = evaluate("2025-26", given("20000"));
    assert_eq!(
        amount(&all, "reportable_superannuation_contributions"),
        "0.00"
    );
    assert_eq!(
        amount(&all, "co_contribution_employer_contributions"),
        "0.00"
    );
    assert_eq!(amount(&all, "rebate_income"), "60000.00");
}

/// Each measure disregards the assessable FHSS released amount; where it is
/// more than taxable income, no taxable income is left (ITAA 1997 s 4-15(1)).
/// The co-contribution takes it from assessable income (s 8(1)(a)).
#[test]
fn the_fhss_released_amount_is_disregarded() {
    let part = evaluate(
        "2025-26",
        json!({"taxable_income": "30000", "assessable_income": "45000",
               "assessable_fhss_released_amount": "10000"}),
    );
    assert_eq!(amount(&part, "taxable_income_without_fhss"), "20000.00");
    assert_eq!(amount(&part, "rebate_income"), "20000.00");
    assert_eq!(amount(&part, "co_contribution_total_income"), "35000.00");
    let all = evaluate(
        "2025-26",
        json!({"taxable_income": "30000", "assessable_income": "50000",
               "assessable_fhss_released_amount": "40000"}),
    );
    assert_eq!(amount(&all, "taxable_income_without_fhss"), "0.00");
    assert_eq!(amount(&all, "repayment_income"), "0.00");
}

/// Only income for surcharge purposes, and so Division 293 income, adds the
/// amounts on which family trust distribution tax was paid (s 995-1 (a)).
#[test]
fn family_trust_distribution_amounts_count_for_the_surcharge_only() {
    let result = evaluate(
        "2026-27",
        json!({"taxable_income": "100000", "assessable_income": "100000",
               "family_trust_distribution_amounts": "20000"}),
    );
    assert_eq!(
        amount(&result, "income_for_surcharge_purposes"),
        "120000.00"
    );
    assert_eq!(amount(&result, "division_293_income"), "120000.00");
    for measure in [
        "rebate_income",
        "repayment_income",
        "adjusted_taxable_income",
    ] {
        assert_eq!(amount(&result, measure), "100000.00", "{measure}");
    }
}

/// Adjusted taxable income adds tax free pensions and target foreign income
/// and takes off child support paid (Family Assistance Act Sch 3 cl 2(1)), to
/// below nil where the child support is more, as the clause and the ATO's
/// worksheet 1b row k set no floor.
#[test]
fn adjusted_taxable_income_adds_pensions_and_foreign_income_less_child_support() {
    let result = evaluate(
        "2025-26",
        json!({"taxable_income": "20000", "assessable_income": "20000",
               "tax_free_government_pensions": "5000", "target_foreign_income": "3000",
               "child_support_paid": "8000"}),
    );
    // 20,000 + 3,000 + 5,000 - 8,000.
    assert_eq!(amount(&result, "adjusted_taxable_income"), "20000.00");
    assert_eq!(amount(&result, "rebate_income"), "20000.00");
    let below = evaluate("2025-26", json!({"child_support_paid": "1000"}));
    assert_eq!(amount(&below, "adjusted_taxable_income"), "-1000.00");
}

/// The co-contribution's total income (s 8(1) and (2)): assessable income,
/// fringe benefits and employer contributions, less business deductions.
#[test]
fn co_contribution_total_income_takes_off_business_deductions() {
    let result = evaluate(
        "2025-26",
        json!({"taxable_income": "35000", "assessable_income": "40000",
               "reportable_fringe_benefits_other": "2000",
               "reportable_employer_super_contributions": "1000",
               "business_deductions": "5000"}),
    );
    // 40,000 + 2,000 + 1,000 - 5,000.
    assert_eq!(amount(&result, "co_contribution_total_income"), "38000.00");
}

/// Each measure lists, as not used, the components it leaves out, through the
/// steps its formula names.
#[test]
fn each_measure_lists_the_components_it_does_not_use() {
    let result = evaluate("2025-26", christina());
    assert_eq!(
        result["not_used"],
        json!({
            "income_for_surcharge_purposes": [
                "tax_free_government_pensions", "target_foreign_income",
                "exempt_foreign_employment_income", "child_support_paid", "assessable_income",
                "business_deductions"],
            "division_293_income": [
                "tax_free_government_pensions", "target_foreign_income",
                "exempt_foreign_employment_income", "child_support_paid", "assessable_income",
                "business_deductions"],
            "rebate_income": [
                "family_trust_distribution_amounts", "tax_free_government_pensions",
                "target_foreign_income", "exempt_foreign_employment_income",
                "child_support_paid", "assessable_income", "business_deductions"],
            "repayment_income": [
                "family_trust_distribution_amounts", "tax_free_government_pensions",
                "target_foreign_income", "child_support_paid", "assessable_income",
                "business_deductions"],
            "adjusted_taxable_income": [
                "family_trust_distribution_amounts", "exempt_foreign_employment_income",
                "assessable_income", "business_deductions"],
            "co_contribution_total_income": [
                "taxable_income", "family_trust_distribution_amounts",
                "deductible_personal_super_contributions", "net_financial_investment_loss",
                "net_rental_property_loss", "tax_free_government_pensions",
                "target_foreign_income", "exempt_foreign_employment_income",
                "child_support_paid"],
        })
    );
}

/// Each measure's step cites its definition.
#[test]
fn each_measure_cites_its_definition() {
    let doc = api::evaluate(
        "derive",
        &json!({"command": COMMAND, "input": input("2025-26", christina())}),
    )
    .unwrap();
    let authority = |id: &str| {
        let steps = doc["steps"].as_array().unwrap();
        let step = steps.iter().find(|s| s["id"] == id).unwrap();
        step["authority"].as_str().unwrap().to_string()
    };
    for (measure, provision) in [
        (
            "income_for_surcharge_purposes",
            "ITAA 1997 s 995-1 'income for surcharge purposes'",
        ),
        ("division_293_income", "ITAA 1997 s 293-20(1)(a)"),
        ("rebate_income", "ITAA 1936 s 6(1) 'rebate income'"),
        ("repayment_income", "HESA s 154-5(1)"),
        (
            "adjusted_taxable_income",
            "Family Assistance Act Sch 3 cl 2(1)",
        ),
        (
            "co_contribution_total_income",
            "Low Income Earners) Act 2003 s 8(1)",
        ),
    ] {
        assert!(authority(measure).contains(provision), "{measure}");
    }
    let steps = doc["steps"].as_array().unwrap();
    let repayment = steps
        .iter()
        .find(|s| s["id"] == "repayment_income")
        .unwrap();
    assert_eq!(
        repayment["working"],
        "taxable_income_without_fhss + total_net_investment_loss + reportable_fringe_benefits_total + exempt_foreign_employment_income + reportable_superannuation_contributions = 50420 + 1250 + 4560 + 2580 + 15000 = 73810"
    );
}

/// Exempt foreign employment income with taxable income of $1 or more: the
/// ATO's surcharge page adds it, s 995-1 does not, and the result says so.
#[test]
fn exempt_foreign_employment_income_warns_about_the_surcharge_page() {
    let warns = |given: Value| {
        evaluate("2025-26", given)["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| {
                w.as_str()
                    .unwrap()
                    .contains("adds it where taxable income is $1 or more")
            })
    };
    assert!(warns(christina()));
    // Both surcharge Acts were read: s 3(1) of each takes s 995-1's meaning.
    let warning = evaluate("2025-26", christina())["warnings"].to_string();
    assert!(warning.contains("Medicare Levy Act 1986 (compilation 53)"));
    assert!(warning.contains("s 61-580(1)(d)"));
    let no_income = json!({"exempt_foreign_employment_income": "2580",
                           "target_foreign_income": "2580"});
    assert!(!warns(no_income));
    assert!(!warns(
        json!({"taxable_income": "50420", "assessable_income": "50420"})
    ));
}

#[test]
fn refusals() {
    let base = || input("2025-26", christina());
    let mut missing = base();
    missing
        .as_object_mut()
        .unwrap()
        .remove("child_support_paid");
    assert_eq!(
        refusal(missing),
        (
            "input.missing_field",
            "child_support_paid is required".to_string()
        )
    );
    let mut negative = base();
    negative["net_rental_property_loss"] = json!("-1");
    assert_eq!(refusal(negative).0, "input.amount_invalid");
    let mut number = base();
    number["business_deductions"] = json!(0);
    assert_eq!(refusal(number).0, "input.amount_invalid");
    let mut cents = base();
    cents["taxable_income"] = json!("50420.50");
    cents["assessable_income"] = json!("50420.50");
    assert_eq!(refusal(cents).0, "input.amount_not_whole_dollars");
    let mut unknown = base();
    unknown["low_tax_contributions"] = json!("1000");
    assert_eq!(
        refusal(unknown),
        (
            "input.field_unknown",
            "low_tax_contributions is not an input of income-tests".to_string()
        )
    );
    for residency in [
        "part_year_resident",
        "working_holiday_maker",
        "foreign_resident",
    ] {
        let mut other = base();
        other["residency"] = json!(residency);
        let (code, message) = refusal(other);
        assert_eq!(code, "income_tests.residency_unsupported", "{residency}");
        assert!(
            message
                .starts_with("income-tests works out the measures for a full-year resident only")
        );
    }
    let mut invalid = base();
    invalid["residency"] = json!("resident");
    assert_eq!(refusal(invalid).0, "input.residency_invalid");
    for year in ["2023-24", "2027-28"] {
        let mut other = base();
        other["year"] = json!(year);
        assert_eq!(refusal(other).0, "period.unsupported", "{year}");
    }
    let mut unconfirmed = base();
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(refusal(unconfirmed).0, "scope.not_confirmed");
}

/// A component larger than the amount that includes it cannot be true.
#[test]
fn components_larger_than_their_whole_are_refused() {
    for (key, value) in [
        ("taxable_income", "50421"),
        ("assessable_fhss_released_amount", "50421"),
        ("exempt_foreign_employment_income", "2581"),
    ] {
        let mut facts = input("2025-26", christina());
        facts[key] = json!(value);
        let (code, message) = refusal(facts);
        assert_eq!(code, "income_tests.components_inconsistent", "{key}");
        assert!(message.starts_with(&format!("{key} exceeds ")), "{message}");
    }
}

#[test]
fn catalogue_lists_the_worksheet_and_its_example_runs() {
    let catalogue = worksheet_catalogue();
    let entry = &catalogue["income_tests"];
    let years: Vec<&str> = entry["supported_periods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(years, YEARS);
    let required = entry["required_inputs"].as_object().unwrap();
    assert_eq!(
        required.len(),
        20,
        "kind, scope, year, residency and 16 components"
    );
    assert!(entry["scope"].as_str().unwrap().contains(
        "https://www.servicesaustralia.gov.au/income-test-for-commonwealth-seniors-health-card"
    ));
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(amount(&result, "repayment_income"), "73810.00");
}
