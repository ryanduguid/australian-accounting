// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `personal-tax-estimate`: the ATO's worked figures for tax with the Medicare
//! levy (foreign income tax offset guides 2025 and 2026, examples 16 and 17),
//! the surcharge (Tom) and SAPTO (José), read on 15 November 2026; the order
//! and limit of the offsets (ITAA 1997 s 63-10(1)); and every refusal
//! (docs/calculation-evidence.md#personal-tax-estimate).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "personal-tax-estimate";

/// A single full-year resident with hospital cover all year, no study loan,
/// SAPTO or Division 293, with `extra` replacing or adding top-level inputs.
fn input(year: &str, taxable: &str, extra: Value) -> Value {
    let mut facts = json!({
        "year": year, "scope_confirmed": true, "residency": "full_year_resident",
        "taxable_income": taxable,
        "medicare-levy": {"section_9a_adjustment_applies": false},
        "medicare-levy-surcharge": {"reportable_fringe_benefits": "0",
            "income_for_surcharge_purposes": taxable, "days_without_cover": 0},
    });
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn amounts(year: &str, taxable: &str, extra: Value) -> Value {
    api::evaluate(COMMAND, &input(year, taxable, extra)).unwrap()["amounts"].clone()
}

fn refusal(facts: Value) -> api::Refusal {
    api::evaluate_coded(COMMAND, &facts).unwrap_err()
}

fn cents(value: &Value) -> i64 {
    let text = value.as_str().unwrap().replace('.', "");
    text.parse().unwrap()
}

/// The ATO's 'tax on taxable income (includes Medicare levy)', disregarding
/// offsets, from the foreign income tax offset rules guides for 2025 (2024-25)
/// and 2026 (2025-26): Karen's $80,000 and $60,000 and Anna's $19,830. The
/// estimate gives the same basic tax and levy, then applies the low income
/// tax offset, which the guide disregards.
#[test]
fn tax_and_medicare_levy_match_the_ato_foreign_income_tax_offset_examples() {
    // (taxable income, ATO tax with levy, LITO applied, total before credits)
    let cases = [
        ("80000", 1_638_800, 0, 1_638_800),
        ("60000", 998_800, 10_000, 988_800),
        // LITO is $700 but reduces tax only to nil (ATO, Low income tax offset).
        ("19830", 26_080, 26_080, 0),
    ];
    for year in ["2024-25", "2025-26"] {
        for (income, ato, offset, total) in cases {
            let a = amounts(year, income, json!({}));
            let with_levy = cents(&a["basic_income_tax"]) + cents(&a["medicare_levy"]);
            assert_eq!(with_levy, ato, "{year} {income}");
            assert_eq!(
                cents(&a["low_income_tax_offset_applied"]),
                offset,
                "{year} {income}"
            );
            assert_eq!(cents(&a["total_before_credits"]), total, "{year} {income}");
        }
    }
}

/// Anna's $31,130 for 2025-26: the guide's $2,459.60 is $2,068.80 tax and a
/// levy of 10% over $27,222, the 2024-25 low-income threshold. 2025-26's is
/// $28,011, so the levy is $311.90 and the sum $2,380.70.
#[test]
fn the_2025_26_guide_applies_the_2024_25_medicare_threshold_to_anna() {
    let a = amounts("2025-26", "31130", json!({}));
    assert_eq!(a["basic_income_tax"], "2068.80");
    assert_eq!(a["medicare_levy"], "311.90");
    // The guide's levy, in cents: $2,459.60 less $2,068.80 is 10% of $31,130 less $27,222.
    assert_eq!(245_960 - 206_880, (31_130 - 27_222) * 10);
}

/// Tom (ATO, Medicare levy surcharge income, thresholds and rates, 2026-27):
/// $90,000 taxable income, $27,000 reportable fringe benefits, no cover all
/// year: a $1,170 surcharge, beside $17,520 tax and a $1,800 levy.
#[test]
fn tom_pays_the_ato_surcharge_beside_tax_and_the_levy() {
    let surcharge = json!({"reportable_fringe_benefits": "27000",
        "income_for_surcharge_purposes": "117000", "days_without_cover": 365});
    let a = amounts(
        "2026-27",
        "90000",
        json!({"medicare-levy-surcharge": surcharge}),
    );
    assert_eq!(a["basic_income_tax"], "17520.00");
    assert_eq!(a["medicare_levy"], "1800.00");
    assert_eq!(a["medicare_levy_surcharge"], "1170.00");
    assert_eq!(a["total_before_credits"], "20490.00");
}

/// José (ATO, Seniors and pensioners tax offset, 2025-26): single, $39,000
/// rebate income, a $1,720 offset. Taken here as his taxable income too, SAPTO
/// applies before LITO ($625) and his levy is worked out at the SAPTO
/// thresholds, under which he pays none.
#[test]
fn jose_applies_sapto_first_and_takes_the_sapto_medicare_thresholds() {
    let sapto = json!({"rebate_income": "39000", "status": "single"});
    let a = amounts(
        "2025-26",
        "39000",
        json!({"seniors-pensioners-tax-offset": sapto}),
    );
    assert_eq!(a["basic_income_tax"], "3328.00");
    assert_eq!(a["seniors_pensioners_tax_offset_applied"], "1720.00");
    assert_eq!(a["low_income_tax_offset_applied"], "625.00");
    assert_eq!(a["income_tax_after_offsets"], "983.00");
    assert_eq!(a["medicare_levy"], "0.00");
    assert_eq!(a["total_before_credits"], "983.00");
    // Without SAPTO the same income pays the full 2% levy.
    assert_eq!(
        amounts("2025-26", "39000", json!({}))["medicare_levy"],
        "780.00"
    );
}

/// Each amount names the worksheet it came from, each part's own result is
/// kept whole, and the parts not given are listed.
#[test]
fn each_line_names_its_worksheet_and_each_part_keeps_its_sources() {
    let loan = json!({"repayment_income": "86380"});
    let facts = input("2026-27", "86380", json!({"study-loan-repayment": loan}));
    let out = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(
        out["working"]["basic_income_tax"],
        "resident-tax: basic_income_tax"
    );
    let keys: Vec<&String> = out["amounts"].as_object().unwrap().keys().collect();
    let working: Vec<&String> = out["working"].as_object().unwrap().keys().collect();
    assert_eq!(keys, working);
    assert_eq!(
        out["not_composed"],
        json!(["seniors-pensioners-tax-offset", "division-293"])
    );
    let alone = json!({"repayment_income": "86380", "year": "2026-27", "scope_confirmed": true});
    let part = api::evaluate("study-loan-repayment", &alone).unwrap();
    assert_eq!(out["components"]["study-loan-repayment"], part);
    assert_eq!(
        out["amounts"]["study_loan_repayment"],
        part["amounts"]["compulsory_repayment"]
    );
    for (_, result) in out["components"].as_object().unwrap() {
        assert!(!result["sources"].as_array().unwrap().is_empty());
    }
}

/// Division 293 tax is shown but kept out of the total: its own notice assesses it.
#[test]
fn division_293_is_shown_outside_the_total() {
    let div293 = json!({"division_293_income": "240000.00", "low_tax_contributions": "30000.00"});
    let with = amounts("2025-26", "240000", json!({"division-293": div293}));
    let without = amounts("2025-26", "240000", json!({}));
    assert_eq!(with["division_293_tax"], "3000.00");
    assert_eq!(without["division_293_tax"], "0.00");
    assert_eq!(
        with["total_before_credits"],
        without["total_before_credits"]
    );
}

#[test]
fn a_part_year_resident_or_working_holiday_maker_is_refused() {
    for residency in ["part_year_resident", "working_holiday_maker"] {
        let r = refusal(input("2025-26", "50000", json!({"residency": residency})));
        assert_eq!(r.code, "estimate.residency_unsupported", "{residency}");
    }
    let r = refusal(input("2025-26", "50000", json!({"residency": "resident"})));
    assert_eq!(r.code, "input.residency_invalid");
    let mut facts = input("2025-26", "50000", json!({}));
    facts.as_object_mut().unwrap().remove("residency");
    assert_eq!(refusal(facts).code, "input.missing_field");
}

/// A full-year foreign resident's inputs: the shared ones, with `extra` added.
fn foreign(year: &str, taxable: &str, extra: Value) -> Value {
    let mut facts = json!({"year": year, "scope_confirmed": true,
        "residency": "foreign_resident", "taxable_income": taxable});
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

/// A foreign resident for the full year (ATO, Tax rates - foreign resident,
/// last updated 1 June 2026, read on 15 November 2026: 30c for each $1 to
/// $135,000, $60,850 plus 45c for each $1 over $190,000 for 2024-25 and
/// 2025-26) pays basic tax on the foreign resident scale, has no low income
/// tax offset (ITAA 1997 s 61-110(1)(a)) and pays no Medicare levy or
/// surcharge (ITAA 1936 s 251S(1)(a)). Division 293 tax still applies, outside
/// the total: 15% of the lesser of $30,000 and the $30,000 over $250,000.
#[test]
fn a_foreign_resident_pays_the_foreign_resident_scale_without_lito_or_levies() {
    for (year, income, tax) in [
        ("2025-26", "60000", "18000.00"),
        ("2024-25", "19830", "5949.00"),
        ("2025-26", "250000", "87850.00"),
    ] {
        let out = api::evaluate(COMMAND, &foreign(year, income, json!({}))).unwrap();
        let a = &out["amounts"];
        assert_eq!(a["basic_income_tax"], tax, "{year} {income}");
        for key in [
            "low_income_tax_offset_applied",
            "medicare_levy",
            "medicare_levy_surcharge",
        ] {
            assert_eq!(a[key], "0.00", "{key}");
        }
        assert_eq!(a["total_before_credits"], tax, "{year} {income}");
        assert_eq!(out["residency"], "foreign_resident");
        let alone = json!({"taxable_income": income, "year": year, "scope_confirmed": true});
        assert_eq!(
            out["components"]["foreign-resident-tax"],
            api::evaluate("foreign-resident-tax", &alone).unwrap()
        );
        assert_eq!(
            out["working"]["basic_income_tax"],
            "foreign-resident-tax: basic_income_tax"
        );
    }
    let div293 = json!({"division-293": {"division_293_income": "250000",
        "low_tax_contributions": "30000"}});
    let a = &api::evaluate(COMMAND, &foreign("2025-26", "250000", div293)).unwrap()["amounts"];
    assert_eq!(a["division_293_tax"], "4500.00");
    assert_eq!(a["total_before_credits"], "87850.00");
}

/// A foreign resident gives no Medicare levy or surcharge input, which do not
/// apply, and no SAPTO or study loan, which the estimate composes for a
/// resident only: a foreign resident's study loan repayment includes the
/// overseas levy on worldwide income, which study-loan-repayment excludes.
#[test]
fn a_foreign_resident_part_that_does_not_apply_is_refused() {
    let surcharge = json!({"reportable_fringe_benefits": "0",
        "income_for_surcharge_purposes": "60000", "days_without_cover": 0});
    for (part, given, code) in [
        (
            "medicare-levy",
            json!({"section_9a_adjustment_applies": false}),
            "input.field_unknown",
        ),
        ("medicare-levy-surcharge", surcharge, "input.field_unknown"),
        (
            "seniors-pensioners-tax-offset",
            json!({"rebate_income": "60000", "status": "single"}),
            "estimate.residency_unsupported",
        ),
        (
            "study-loan-repayment",
            json!({"repayment_income": "60000"}),
            "estimate.residency_unsupported",
        ),
    ] {
        let mut facts = foreign("2025-26", "60000", json!({}));
        facts[part] = given;
        let r = refusal(facts);
        assert_eq!(r.code, code, "{part}");
        assert!(r.message.starts_with(part), "{}", r.message);
    }
}

/// A part's refusal is the part's own code under its name, never a partial total.
#[test]
fn a_part_refusal_keeps_its_code_and_names_the_part() {
    // A sole parent: the surcharge worksheet refuses s 8C (R2980).
    let sole_parent = json!({"reportable_fringe_benefits": "0",
        "income_for_surcharge_purposes": "50000", "days_without_cover": 0,
        "dependent_children": 1});
    let r = refusal(input(
        "2025-26",
        "50000",
        json!({"medicare-levy-surcharge": sole_parent}),
    ));
    assert_eq!(r.code, "input.field_unknown");
    assert!(
        r.message
            .starts_with("medicare-levy-surcharge: dependent_children"),
        "{}",
        r.message
    );
    // Study loan repayment has no 2024-25 rates.
    let loan = json!({"study-loan-repayment": {"repayment_income": "50000"}});
    let r = refusal(input("2024-25", "50000", loan));
    assert_eq!(r.code, "period.unsupported");
    assert!(
        r.message.starts_with("study-loan-repayment: "),
        "{}",
        r.message
    );
    let r = refusal(input("2025-26", "50000", json!({"scope_confirmed": false})));
    assert_eq!(r.code, "scope.not_confirmed");
    assert!(r.message.starts_with("resident-tax: "), "{}", r.message);
}

#[test]
fn a_missing_or_repeated_input_is_refused() {
    let mut facts = input("2025-26", "50000", json!({}));
    facts
        .as_object_mut()
        .unwrap()
        .remove("medicare-levy-surcharge");
    let r = refusal(facts);
    assert_eq!(
        (r.code, r.message.as_str()),
        ("input.missing_field", "medicare-levy-surcharge is required")
    );
    let r = refusal(input(
        "2025-26",
        "50000",
        json!({"low-income-tax-offset": {}}),
    ));
    assert_eq!(r.code, "input.field_unknown");
    // Shared inputs are given once, and SAPTO entitlement comes from the offset.
    for (key, value) in [("year", json!("2025-26")), ("sapto_entitled", json!(true))] {
        let mut levy = json!({"section_9a_adjustment_applies": false});
        levy[key] = value;
        let r = refusal(input("2025-26", "50000", json!({"medicare-levy": levy})));
        assert_eq!(r.code, "input.field_unknown", "{key}");
    }
}

/// The parts describe one family: a spouse in each or in none, one spouse
/// taxable income, and no dependants without a spouse, whom the surcharge
/// worksheet excludes (Medicare Levy Act 1986 s 8C).
#[test]
fn parts_that_describe_different_families_are_refused() {
    let levy = json!({"section_9a_adjustment_applies": false,
        "spouse_taxable_income": "20000", "spouse_sapto_entitled": true});
    let surcharge = json!({"reportable_fringe_benefits": "0",
        "income_for_surcharge_purposes": "30000", "days_without_cover": 0,
        "spouse_income_for_surcharge_purposes": "20000"});
    let sapto = json!({"rebate_income": "30000", "status": "couple",
        "spouse_rebate_income": "20000"});
    let mut other_spouse = sapto.clone();
    other_spouse["spouse_taxable_income"] = json!("25000");
    for extra in [
        json!({"medicare-levy": levy}),
        json!({"medicare-levy-surcharge": surcharge}),
        json!({"seniors-pensioners-tax-offset": sapto}),
        json!({"medicare-levy": {"section_9a_adjustment_applies": false,
            "dependant_offset_entitled": true}}),
        json!({"medicare-levy": {"section_9a_adjustment_applies": false,
            "dependent_children": 2}}),
        json!({"medicare-levy": levy, "medicare-levy-surcharge": surcharge,
            "seniors-pensioners-tax-offset": other_spouse}),
    ] {
        let r = refusal(input("2025-26", "30000", extra.clone()));
        assert_eq!(r.code, "estimate.family_mismatch", "{extra}");
    }
    let mut same_spouse = sapto.clone();
    same_spouse["spouse_taxable_income"] = json!("20000.00");
    let couple = json!({"medicare-levy": levy, "medicare-levy-surcharge": surcharge,
        "seniors-pensioners-tax-offset": same_spouse});
    let out = api::evaluate(COMMAND, &input("2025-26", "30000", couple)).unwrap();
    assert_eq!(out["ok"], true);
}

/// The surcharge worksheet's scope excludes anyone in an exemption category.
#[test]
fn exemption_days_beside_the_surcharge_are_refused() {
    let levy = json!({"section_9a_adjustment_applies": false,
        "medicare_levy_exemption_full_days": 10});
    let r = refusal(input("2025-26", "50000", json!({"medicare-levy": levy})));
    assert_eq!(r.code, "estimate.surcharge_with_exemption");
}
