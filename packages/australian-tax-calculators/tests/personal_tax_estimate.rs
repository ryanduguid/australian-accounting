// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `personal-tax-estimate`: the ATO's worked figures for tax with the Medicare
//! levy (foreign income tax offset guides 2025 and 2026, examples 16 and 17),
//! the surcharge (Tom) and SAPTO (José), read on 15 November 2026; the order
//! and limit of the offsets (ITAA 1997 s 63-10(1)); taxable income worked out
//! from labour income with the standard deduction and compared with the tax
//! withheld, by hand; and every refusal
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
    // The reason names the part not taken, the Medicare levy's
    // non-resident days, not the threshold resident-tax now pro-rates.
    let r = refusal(input(
        "2025-26",
        "50000",
        json!({"residency": "part_year_resident"}),
    ));
    assert!(r.message.contains("s 251U(1)(d)"), "{}", r.message);
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
    // A field a part does not take is refused, also by a part that alone
    // ignores one, so a contradictory residency cannot hide inside a part.
    for (part, given) in [
        (
            "study-loan-repayment",
            json!({"repayment_income": "50000", "residency": "foreign_resident"}),
        ),
        (
            "division-293",
            json!({"division_293_income": "50000", "low_tax_contributions": "0",
                "low_tax_contribution": "9000"}),
        ),
    ] {
        let mut facts = input("2025-26", "50000", json!({}));
        facts[part] = given;
        let r = refusal(facts);
        assert_eq!(r.code, "input.field_unknown", "{part}");
        assert!(
            r.message.ends_with(&format!("is not an input of {part}")),
            "{}",
            r.message
        );
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

/// A single full-year resident with hospital cover all year, whose taxable
/// income is worked out from `income`: the labour income and the inputs beside
/// it, and `tax_withheld` when given.
fn from_income(year: &str, surcharge_income: &str, income: Value) -> Value {
    let mut facts = input(year, "0", income);
    facts.as_object_mut().unwrap().remove("taxable_income");
    facts["medicare-levy-surcharge"]["income_for_surcharge_purposes"] = json!(surcharge_income);
    facts
}

/// Taxable income from salary, read on 16 November 2026: ITAA 1997 s 4-15(1)
/// (assessable income less deductions, nil where they equal or exceed it),
/// s 25-130 as Act No. 49 of 2026 Sch 4 inserted it for 2026-27 and later
/// years (the lesser of $1,000 and assessable labour income, reduced by the
/// work-related deductions it lists), and TAA 1953 Sch 1 s 18-15(1) (a credit
/// for the amounts withheld). Worked by hand: $85,000 salary, $300 of work
/// expenses, $500 interest and $200 of union fees in 2026-27. The standard
/// deduction is $1,000 - $300 = $700, so taxable income is $85,500 - $1,200 =
/// $84,300. Tax is 15% of $26,800 plus 30% of $39,300, $15,810; there is no
/// low income tax offset above $66,667; the levy is 2%, $1,686; the total is
/// $17,496, and $16,000 withheld leaves $1,496 to pay.
#[test]
fn salary_works_out_taxable_income_with_the_2026_27_standard_deduction() {
    let income = json!({"assessable_labour_income": "85000", "other_assessable_income": "500",
        "work_related_deductions": "300", "other_deductions": "200", "tax_withheld": "16000"});
    let out = api::evaluate(COMMAND, &from_income("2026-27", "84300", income)).unwrap();
    let a = &out["amounts"];
    for (key, expected) in [
        ("total_assessable_income", "85500.00"),
        ("standard_deduction", "700.00"),
        ("total_deductions", "1200.00"),
        ("taxable_income", "84300.00"),
        ("basic_income_tax", "15810.00"),
        ("low_income_tax_offset_applied", "0.00"),
        ("medicare_levy", "1686.00"),
        ("medicare_levy_surcharge", "0.00"),
        ("total_before_credits", "17496.00"),
        ("tax_withheld", "16000.00"),
        ("refund", "0.00"),
        ("amount_payable", "1496.00"),
    ] {
        assert_eq!(a[key], expected, "{key}");
    }
    let keys: Vec<&String> = a.as_object().unwrap().keys().collect();
    let working: Vec<&String> = out["working"].as_object().unwrap().keys().collect();
    assert_eq!(keys, working);
    assert_eq!(
        keys[..4],
        [
            "total_assessable_income",
            "standard_deduction",
            "total_deductions",
            "taxable_income"
        ]
    );
    // The parts run on the taxable income worked out, as if it were given.
    let given = api::evaluate(COMMAND, &input("2026-27", "84300", json!({}))).unwrap();
    assert_eq!(out["components"], given["components"]);
    let sources = out["sources"].as_array().unwrap();
    for section in [
        "19970038/4-15",
        "19970038/25-130",
        "C2026A00049",
        "19530001/Sch1-18-15",
    ] {
        assert!(
            sources
                .iter()
                .any(|s| s.as_str().unwrap().contains(section)),
            "{section}"
        );
    }
}

/// s 25-130(2): the lesser of $1,000 and assessable labour income, reduced,
/// but not below zero, by the work-related deductions; other deductions do
/// not reduce it.
#[test]
fn the_standard_deduction_is_limited_by_labour_income_and_work_deductions() {
    for (labour, other_income, work, other, standard, taxable) in [
        ("85000", "0", "0", "0", "1000.00", "84000.00"),
        ("85000", "0", "1000", "0", "0.00", "84000.00"),
        ("85000", "0", "1500", "0", "0.00", "83500.00"),
        ("85000", "0", "0", "5000", "1000.00", "79000.00"),
        ("600", "30000", "0", "0", "600.00", "30000.00"),
    ] {
        let income = json!({"assessable_labour_income": labour,
            "other_assessable_income": other_income,
            "work_related_deductions": work, "other_deductions": other});
        let out = api::evaluate(COMMAND, &from_income("2026-27", taxable, income)).unwrap();
        assert_eq!(
            out["amounts"]["standard_deduction"], standard,
            "{labour} {work}"
        );
        assert_eq!(out["amounts"]["taxable_income"], taxable, "{labour} {work}");
    }
}

/// Before 2026-27 there is no standard deduction (Act No. 49 of 2026 Sch 4
/// item 17(1)). $60,000 salary and $300 of work expenses in 2025-26 give
/// $59,700: tax 16% of $26,800 plus 30% of $14,700, $8,698; the low income tax
/// offset $325 - 1.5% of $14,700, $104.50; the levy $1,194; a total of
/// $9,787.50, so $10,000 withheld gives a $212.50 refund. A foreign resident
/// is not resident at any time in the year (s 25-130(1)(b)), so has none either.
#[test]
fn no_standard_deduction_before_2026_27_or_for_a_foreign_resident() {
    let income = json!({"assessable_labour_income": "60000",
        "work_related_deductions": "300", "tax_withheld": "10000"});
    let out = api::evaluate(COMMAND, &from_income("2025-26", "59700", income)).unwrap();
    let a = &out["amounts"];
    for (key, expected) in [
        ("standard_deduction", "0.00"),
        ("taxable_income", "59700.00"),
        ("basic_income_tax", "8698.00"),
        ("low_income_tax_offset_applied", "104.50"),
        ("medicare_levy", "1194.00"),
        ("total_before_credits", "9787.50"),
        ("refund", "212.50"),
        ("amount_payable", "0.00"),
    ] {
        assert_eq!(a[key], expected, "{key}");
    }
    let foreign = json!({"year": "2026-27", "scope_confirmed": true,
        "residency": "foreign_resident", "assessable_labour_income": "60000"});
    let out = api::evaluate(COMMAND, &foreign).unwrap();
    assert_eq!(out["amounts"]["standard_deduction"], "0.00");
    assert_eq!(out["amounts"]["taxable_income"], "60000.00");
}

/// s 4-15(1) step 3: deductions that equal or exceed assessable income leave
/// no taxable income. $500 of salary gives a $500 standard deduction, and
/// $2,000 of gifts leaves $2,500 of deductions against $500.
#[test]
fn deductions_at_or_above_income_leave_no_taxable_income() {
    let income = json!({"assessable_labour_income": "500", "other_deductions": "2000",
        "tax_withheld": "0"});
    let out = api::evaluate(COMMAND, &from_income("2026-27", "0", income)).unwrap();
    let a = &out["amounts"];
    assert_eq!(a["standard_deduction"], "500.00");
    assert_eq!(a["total_deductions"], "2500.00");
    assert_eq!(a["taxable_income"], "0.00");
    assert_eq!(a["total_before_credits"], "0.00");
    assert_eq!(
        (&a["refund"], &a["amount_payable"]),
        (&json!("0.00"), &json!("0.00"))
    );
}

#[test]
fn tax_withheld_also_compares_with_an_established_taxable_income() {
    let out = api::evaluate(
        COMMAND,
        &input("2025-26", "60000", json!({"tax_withheld": "10000.50"})),
    )
    .unwrap();
    // $8,788 tax less a $100 offset plus a $1,200 levy is $9,888.
    assert_eq!(out["amounts"]["total_before_credits"], "9888.00");
    assert_eq!(out["amounts"]["refund"], "112.50");
    assert!(out["amounts"].get("taxable_income").is_none());
}

#[test]
fn income_inputs_that_conflict_or_are_malformed_are_refused() {
    let salary = |extra: Value| {
        let mut income = json!({"assessable_labour_income": "85000"});
        for (k, v) in extra.as_object().unwrap() {
            income[k] = v.clone();
        }
        from_income("2026-27", "85000", income)
    };
    let mut both = salary(json!({}));
    both["taxable_income"] = json!("85000");
    let r = refusal(both);
    assert_eq!(r.code, "input.field_unknown");
    assert!(
        r.message
            .starts_with("assessable_labour_income is not an input"),
        "{}",
        r.message
    );
    let mut alone = input("2026-27", "85000", json!({"other_deductions": "200"}));
    alone.as_object_mut().unwrap().remove("taxable_income");
    let r = refusal(alone.clone());
    assert_eq!(
        (r.code, r.message.as_str()),
        (
            "input.missing_field",
            "assessable_labour_income is required with other_deductions"
        )
    );
    alone.as_object_mut().unwrap().remove("other_deductions");
    let r = refusal(alone);
    assert_eq!(
        (r.code, r.message.as_str()),
        (
            "input.missing_field",
            "taxable_income or assessable_labour_income is required"
        )
    );
    for (extra, code) in [
        (
            json!({"assessable_labour_income": "85000.50"}),
            "input.amount_not_whole_dollars",
        ),
        (
            json!({"work_related_deductions": "-1"}),
            "input.amount_invalid",
        ),
        (
            json!({"other_assessable_income": 500}),
            "input.amount_invalid",
        ),
        (json!({"tax_withheld": "1.234"}), "input.amount_invalid"),
        (json!({"tax_withheld": "ten"}), "input.decimal_unreadable"),
    ] {
        assert_eq!(refusal(salary(extra.clone())).code, code, "{extra}");
    }
}

/// A fortnightly pay of $3,269.23 on scale 2, as `payg-withholding` takes it.
fn fortnightly(scale: i64) -> Value {
    json!({"earnings": "3269.23", "pay_period": "fortnightly", "scale": scale})
}

/// The tax withheld from one employer's regular pays, worked by hand from
/// Schedule 1 for payments from 1 July 2026 (scale 2: a = 0.3200 and
/// b = 181.7319 below $2,596 a week) and Schedule 5 Method A, read on
/// 16 November 2026, and credited under TAA 1953 Sch 1 s 18-15(1). The salary
/// is that of `salary_works_out_taxable_income_with_the_2026_27_standard_deduction`,
/// whose total before credits is $17,496. Weekly earnings are
/// floor($3,269.23 / 2) + 0.99 = $1,634.99, so $341 a week (0.32 x 1,634.99,
/// less 181.7319, is 341.4649, to the dollar) and $682 a fortnight; 26 pays
/// withhold $17,732, a refund of $236.
#[test]
fn the_tax_withheld_is_worked_out_from_one_employers_regular_pay() {
    let income = json!({"assessable_labour_income": "85000", "other_assessable_income": "500",
        "work_related_deductions": "300", "other_deductions": "200",
        "payg-withholding": fortnightly(2), "pays_in_year": 26});
    let out = api::evaluate(COMMAND, &from_income("2026-27", "84300", income)).unwrap();
    let a = &out["amounts"];
    for (key, expected) in [
        ("total_before_credits", "17496.00"),
        ("withholding_per_pay", "682.00"),
        ("tax_withheld", "17732.00"),
        ("refund", "236.00"),
        ("amount_payable", "0.00"),
    ] {
        assert_eq!(a[key], expected, "{key}");
    }
    let keys: Vec<&String> = a.as_object().unwrap().keys().collect();
    let working: Vec<&String> = out["working"].as_object().unwrap().keys().collect();
    assert_eq!(keys, working);
    assert_eq!(
        keys[keys.len() - 4..],
        [
            "withholding_per_pay",
            "tax_withheld",
            "refund",
            "amount_payable"
        ]
    );
    assert!(
        out["working"]["tax_withheld"]
            .as_str()
            .unwrap()
            .starts_with("pays_in_year x withholding_per_pay: "),
    );
    // The pay's part keeps its own result, as when called alone.
    let mut alone = fortnightly(2);
    alone["year"] = json!("2026-27");
    alone["scope_confirmed"] = json!(true);
    let pay = api::evaluate("payg-withholding", &alone).unwrap();
    assert_eq!(out["components"]["payg-withholding"], pay);
    let sources = out["sources"].as_array().unwrap();
    assert!(
        sources
            .iter()
            .any(|s| s.as_str().unwrap().contains("19530001/Sch1-12-35"))
    );
}

/// A $5,000 bonus paid with one of the 26 pays, by Schedule 5 Method A: $192
/// a pay (floor($5,000 / 26)) lifts a fortnight's earnings to $3,461, whose
/// weekly $1,730.99 withholds $372 (0.32 x 1,730.99 - 181.7319 = 372.1849),
/// $744 a fortnight; ($744 - $682) x 26 = $1,612, under the 47% limit of
/// $2,350. The year withholds $17,732 + $1,612 = $19,344. The bonus raises
/// labour income to $90,000 and taxable income to $89,300: tax is $4,020 plus
/// 30% of $44,300, $17,310, and the levy $1,786, a total of $19,096 and a
/// refund of $248.
#[test]
fn a_bonus_paid_with_a_regular_pay_adds_method_a_withholding() {
    let income = json!({"assessable_labour_income": "90000", "other_assessable_income": "500",
        "work_related_deductions": "300", "other_deductions": "200",
        "payg-withholding": fortnightly(2), "pays_in_year": 26,
        "schedule-5-method-a": {"additional_payment": "5000", "pay_periods": 26}});
    let out = api::evaluate(COMMAND, &from_income("2026-27", "89300", income)).unwrap();
    let a = &out["amounts"];
    for (key, expected) in [
        ("total_before_credits", "19096.00"),
        ("withholding_per_pay", "682.00"),
        ("additional_payment_withholding", "1612.00"),
        ("tax_withheld", "19344.00"),
        ("refund", "248.00"),
    ] {
        assert_eq!(a[key], expected, "{key}");
    }
    // Method A takes the regular pay's earnings, period and scale.
    let method_a = json!({"earnings": "3269.23", "additional_payment": "5000",
        "pay_periods": 26, "pay_period": "fortnightly", "scale": 2, "study_loan": false,
        "year": "2026-27", "scope_confirmed": true});
    let alone = api::evaluate("schedule-5-method-a", &method_a).unwrap();
    assert_eq!(out["components"]["schedule-5-method-a"], alone);
}

/// A foreign resident on scale 3 (30% a week to $2,596, less 30 cents):
/// $1,634.99 withholds $490 a week (490.197 to the dollar), $980 a fortnight
/// and $25,480 over 26 pays, against basic tax of 30% of $85,000, $25,500.
#[test]
fn a_foreign_resident_on_scale_3_has_the_tax_withheld_worked_out() {
    let mut facts = foreign(
        "2026-27",
        "85000",
        json!({"payg-withholding": fortnightly(3), "pays_in_year": 26}),
    );
    facts.as_object_mut().unwrap().remove("taxable_income");
    facts["assessable_labour_income"] = json!("85000");
    let a = &api::evaluate(COMMAND, &facts).unwrap()["amounts"];
    assert_eq!(a["basic_income_tax"], "25500.00");
    assert_eq!(a["tax_withheld"], "25480.00");
    assert_eq!(a["amount_payable"], "20.00");
}

/// A study loan or a senior's pay is the withholding its own command gives,
/// times the pays: the estimate adds no rate of its own.
#[test]
fn a_study_loan_or_seniors_pay_is_its_own_commands_withholding_times_the_pays() {
    let loan = json!({"repayment_income": "84300"});
    let senior = json!({"earnings": "5000", "pay_period": "monthly", "status": "single",
        "medicare_levy_exemption": "none"});
    for (command, part, pays) in [
        ("study-loan-withholding", fortnightly(2), 27),
        ("seniors-pensioners-withholding", senior, 12),
    ] {
        let facts = input(
            "2026-27",
            "84300",
            json!({command: part, "pays_in_year": pays, "study-loan-repayment": loan}),
        );
        let out = api::evaluate(COMMAND, &facts).unwrap();
        let mut alone = part.clone();
        alone["year"] = json!("2026-27");
        alone["scope_confirmed"] = json!(true);
        let pay = api::evaluate(command, &alone).unwrap();
        let per_pay = cents(&pay["amounts"]["withholding"]);
        assert!(per_pay > 0, "{command}");
        assert_eq!(
            cents(&out["amounts"]["tax_withheld"]),
            per_pay * pays,
            "{command}"
        );
        assert_eq!(out["components"][command], pay);
    }
}

/// At most 53 weekly, 27 fortnightly, 12 monthly or 4 quarterly pays fall in
/// an income year (Schedule 1, 'When there are 53 pays in a financial year'
/// and 'When there are 27 pays in a financial year').
#[test]
fn pays_in_year_is_limited_by_the_pay_period() {
    for (period, most) in [
        ("weekly", 53),
        ("fortnightly", 27),
        ("monthly", 12),
        ("quarterly", 4),
    ] {
        let pay = json!({"earnings": "1000", "pay_period": period, "scale": 2});
        let facts = |pays: Value| {
            input(
                "2026-27",
                "50000",
                json!({"payg-withholding": pay, "pays_in_year": pays}),
            )
        };
        assert!(
            api::evaluate(COMMAND, &facts(json!(most))).is_ok(),
            "{period}"
        );
        assert!(api::evaluate(COMMAND, &facts(json!(1))).is_ok(), "{period}");
        for pays in [
            json!(most + 1),
            json!(0),
            json!("26"),
            json!(1.5),
            json!(true),
        ] {
            let r = refusal(facts(pays.clone()));
            assert_eq!(r.code, "input.pay_periods_invalid", "{period} {pays}");
            assert_eq!(
                r.message,
                format!("pays_in_year must be an integer from 1 to {most} for {period} pays.")
            );
        }
    }
}

#[test]
fn withholding_inputs_that_conflict_or_are_missing_are_refused() {
    let pay = || json!({"payg-withholding": fortnightly(2), "pays_in_year": 26});
    let with = |extra: Value| {
        let mut facts = input("2026-27", "84300", pay());
        for (k, v) in extra.as_object().unwrap() {
            facts[k] = v.clone();
        }
        facts
    };
    let without = |key: &str| {
        let mut facts = with(json!({}));
        facts.as_object_mut().unwrap().remove(key);
        facts
    };
    let loan = json!({"repayment_income": "84300"});
    let senior = json!({"earnings": "2000", "pay_period": "monthly", "status": "single",
        "medicare_levy_exemption": "none"});
    let bonus = json!({"additional_payment": "5000", "pay_periods": 26});
    for (facts, code, message) in [
        (
            with(json!({"tax_withheld": "17000"})),
            "input.field_unknown",
            "payg-withholding is not an input of personal-tax-estimate beside tax_withheld: give tax_withheld, or a regular pay to work it out",
        ),
        (
            with(json!({"study-loan-withholding": fortnightly(2), "study-loan-repayment": loan})),
            "input.field_unknown",
            "study-loan-withholding is not an input of personal-tax-estimate beside payg-withholding: give the one regular pay",
        ),
        (
            without("payg-withholding"),
            "input.missing_field",
            "payg-withholding, study-loan-withholding or seniors-pensioners-withholding is required with pays_in_year",
        ),
        (
            input("2026-27", "84300", json!({"schedule-5-method-a": bonus})),
            "input.missing_field",
            "payg-withholding, study-loan-withholding or seniors-pensioners-withholding is required with schedule-5-method-a",
        ),
        (
            without("pays_in_year"),
            "input.missing_field",
            "pays_in_year is required with payg-withholding",
        ),
        (
            input(
                "2026-27",
                "84300",
                json!({"study-loan-withholding": fortnightly(2), "pays_in_year": 26}),
            ),
            "input.missing_field",
            "study-loan-repayment is required with study-loan-withholding: the withholding includes a study and training support loan component",
        ),
        (
            input(
                "2026-27",
                "84300",
                json!({"seniors-pensioners-withholding": senior, "pays_in_year": 12,
                    "schedule-5-method-a": bonus}),
            ),
            "input.field_unknown",
            "schedule-5-method-a is not an input of personal-tax-estimate beside seniors-pensioners-withholding: Method A here works from Schedule 1 amounts, not Schedule 9's",
        ),
        (
            with(json!({"schedule-5-method-a": {"additional_payment": "5000",
                "pay_periods": 26, "scale": 2}})),
            "input.field_unknown",
            "scale is not an input of schedule-5-method-a within personal-tax-estimate: it comes from the regular pay's part",
        ),
        (
            with(json!({"payg-withholding": fortnightly(3)})),
            "estimate.scale_mismatch",
            "payg-withholding scale 3 is for a foreign resident payee, and residency is full_year_resident",
        ),
        (
            foreign(
                "2026-27",
                "85000",
                json!({"payg-withholding": fortnightly(2), "pays_in_year": 26}),
            ),
            "estimate.scale_mismatch",
            "payg-withholding scale 2 is for an Australian resident payee, and residency is foreign_resident",
        ),
        (
            foreign(
                "2026-27",
                "85000",
                json!({"seniors-pensioners-withholding": senior, "pays_in_year": 12}),
            ),
            "estimate.residency_unsupported",
            "seniors-pensioners-withholding is not composed for a foreign resident: Schedule 9 is for payees who declare that they are Australian residents for tax purposes",
        ),
        (
            with(json!({"payg-withholding": {"earnings": "3269.23",
                "pay_period": "daily", "scale": 2}})),
            "input.pay_period_invalid",
            "payg-withholding: pay_period must be weekly, fortnightly, monthly or quarterly.",
        ),
    ] {
        let r = refusal(facts);
        assert_eq!((r.code, r.message.as_str()), (code, message));
    }
    // Scale 4 is a foreign resident's with foreign_resident true.
    let no_tfn = json!({"earnings": "3269.23", "pay_period": "fortnightly", "scale": 4,
        "foreign_resident": true});
    let r = refusal(with(json!({"payg-withholding": no_tfn.clone()})));
    assert_eq!(r.code, "estimate.scale_mismatch");
    let matching = foreign(
        "2026-27",
        "85000",
        json!({"payg-withholding": no_tfn, "pays_in_year": 26}),
    );
    assert!(api::evaluate(COMMAND, &matching).is_ok());
    let study = foreign(
        "2026-27",
        "85000",
        json!({"study-loan-withholding": fortnightly(3), "pays_in_year": 26}),
    );
    assert_eq!(refusal(study).code, "estimate.residency_unsupported");
}

/// One pay stands for the year's pays only when the year's Schedule 8
/// coefficients applied from 1 July: 2025-26's changed on 24 September 2025,
/// so a study loan pay in 2025-26 is refused rather than multiplied out.
#[test]
fn a_study_loan_pay_in_a_year_whose_schedule_8_changed_is_refused() {
    let loan = json!({"repayment_income": "84300"});
    let senior = json!({"earnings": "5000", "pay_period": "monthly", "status": "single",
        "medicare_levy_exemption": "none", "study_loan": true, "pay_date": "2025-10-15"});
    let mut study = fortnightly(2);
    study["pay_date"] = json!("2025-10-15");
    for (command, part, pays) in [
        ("study-loan-withholding", study.clone(), 26),
        ("study-loan-withholding", study, 1),
        ("seniors-pensioners-withholding", senior, 12),
    ] {
        let facts = input(
            "2025-26",
            "84300",
            json!({command: part, "pays_in_year": pays, "study-loan-repayment": loan}),
        );
        let r = refusal(facts);
        assert_eq!(r.code, "period.unsupported", "{command}");
        assert_eq!(
            r.message,
            format!(
                "{command} with a study loan does not give the tax withheld for 2025-26: its Schedule 8 coefficients changed on 2025-09-24, and this estimate takes no year whose coefficients changed after 1 July; give tax_withheld"
            )
        );
    }
}

/// Schedule 1's additional amount for 53 weekly or 27 fortnightly pays is
/// asked for only in such a year, so it needs that many pays.
#[test]
fn the_extra_pay_amount_needs_a_year_of_53_or_27_pays() {
    for (period, most) in [("weekly", 53), ("fortnightly", 27)] {
        let pay = json!({"earnings": "2000", "pay_period": period, "scale": 2,
            "extra_pay_withholding": true});
        let facts = |pays: i64| {
            input(
                "2026-27",
                "100000",
                json!({"payg-withholding": pay, "pays_in_year": pays}),
            )
        };
        let r = refusal(facts(most - 1));
        assert_eq!(r.code, "input.pay_periods_invalid", "{period}");
        assert_eq!(
            r.message,
            format!(
                "pays_in_year must be {most} with extra_pay_withholding, the additional amount for a year of {most} {period} pays."
            )
        );
        let out = api::evaluate(COMMAND, &facts(most)).unwrap();
        let per_pay = cents(&out["amounts"]["withholding_per_pay"]);
        assert_eq!(cents(&out["amounts"]["tax_withheld"]), per_pay * most);
    }
}

/// Method A here takes none of the regular pay's declared tax offsets,
/// Medicare levy adjustment or extra pay amount, so its steps 2 and 5 would
/// not use the regular pay's amounts: a bonus beside any of them is refused,
/// and one declared false is not a declaration.
#[test]
fn a_bonus_beside_a_declared_adjustment_is_refused() {
    let bonus = json!({"additional_payment": "5000", "pay_periods": 26});
    let with = |key: &str, value: Value, pays: i64| {
        let mut pay = fortnightly(2);
        pay[key] = value;
        input(
            "2026-27",
            "84300",
            json!({"payg-withholding": pay, "pays_in_year": pays,
                "schedule-5-method-a": bonus}),
        )
    };
    for (key, value, pays) in [
        ("tax_offsets", json!("1445"), 26),
        ("medicare_adjustment_children", json!(1), 26),
        ("extra_pay_withholding", json!(true), 27),
    ] {
        let r = refusal(with(key, value, pays));
        assert_eq!(r.code, "input.field_unknown", "{key}");
        assert_eq!(
            r.message,
            format!(
                "schedule-5-method-a is not an input of personal-tax-estimate beside payg-withholding with {key}: Method A here works the bonus out without the tax offsets, Medicare levy adjustment and extra pay amount a payee declares"
            )
        );
    }
    assert!(api::evaluate(COMMAND, &with("extra_pay_withholding", json!(false), 26)).is_ok());
    // false is no amount or count: the pay's own part refuses it.
    for key in ["tax_offsets", "medicare_adjustment_children"] {
        let r = refusal(with(key, json!(false), 26));
        assert!(
            r.message.starts_with("payg-withholding: "),
            "{key}: {}",
            r.message
        );
    }
}
