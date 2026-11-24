// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `personal-tax-estimate` for a part-year resident: the tax-free threshold
//! pro-rated by `resident-tax` (Income Tax Rates Act 1986 s 20(1)), the low
//! income tax offset in full (ITAA 1997 s 61-110(1)(a)) and the Medicare levy
//! less the non-resident days' share (ITAA 1936 s 251U(1)(d), Medicare Levy
//! Act 1986 s 9(1)), with every refusal
//! (docs/calculation-evidence.md#personal-tax-estimate). Every figure is
//! worked by hand in the comments.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "personal-tax-estimate";

/// A single part-year resident who arrived on 15 October 2025: resident in 9
/// months of 2025-26 (October to June, s 18(1)(b)) and a non-resident with no
/// dependants on the 106 days from 1 July to 14 October 2025. `extra`
/// replaces or adds top-level inputs.
fn input(year: &str, taxable: &str, extra: Value) -> Value {
    let mut facts = json!({
        "year": year, "scope_confirmed": true, "residency": "part_year_resident",
        "part_year_residency_months": 9, "eligible_pensioner": false,
        "taxable_income": taxable,
        "medicare-levy": {"section_9a_adjustment_applies": false, "non_resident_days": 106},
    });
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn amounts(facts: &Value) -> Value {
    api::evaluate(COMMAND, facts).unwrap()["amounts"].clone()
}

fn refusal(facts: &Value) -> api::Refusal {
    api::evaluate_coded(COMMAND, facts).unwrap_err()
}

fn check(a: &Value, expected: &[(&str, &str)]) {
    for (key, value) in expected {
        assert_eq!(a[*key], *value, "{key}");
    }
}

/// Threshold 13,464 + 4,736 x 9 / 12 = 17,016; basic tax (45,000 - 17,016) x
/// 16% + 15,000 x 30% = 4,477.44 + 4,500 = 8,977.44; the low income tax
/// offset 325 - 1.5% x 15,000 = 100 in full; the levy 1,200 x (730 - 212) /
/// 730 = 851.506...; total 8,977.44 - 100 + 851.51 = 9,728.95.
#[test]
fn a_part_year_resident_gets_the_pro_rated_threshold_and_the_reduced_levy() {
    let a = amounts(&input("2025-26", "60000", json!({})));
    check(
        &a,
        &[
            ("basic_income_tax", "8977.44"),
            ("low_income_tax_offset_applied", "100.00"),
            ("income_tax_after_offsets", "8877.44"),
            ("medicare_levy", "851.51"),
            ("medicare_levy_surcharge", "0.00"),
            ("total_before_credits", "9728.95"),
        ],
    );
    let out = api::evaluate(COMMAND, &input("2025-26", "60000", json!({}))).unwrap();
    assert!(
        out["working"]["medicare_levy_surcharge"]
            .as_str()
            .unwrap()
            .starts_with("nil: not composed for a part-year resident")
    );
    assert!(
        out["not_composed"]
            .as_array()
            .unwrap()
            .contains(&json!("medicare-levy-surcharge"))
    );
    assert_eq!(
        out["components"]["resident-tax"]["amounts"]["tax_free_threshold"],
        "17016.00"
    );
}

/// An eligible pensioner keeps the $18,200 threshold (s 18(2)(a)): (45,000 -
/// 18,200) x 16% + 4,500 = 8,788.
#[test]
fn an_eligible_pensioner_keeps_the_full_threshold() {
    let a = amounts(&input(
        "2025-26",
        "60000",
        json!({"eligible_pensioner": true}),
    ));
    assert_eq!(a["basic_income_tax"], "8788.00");
}

/// 2026-27, with the standard deduction for a person resident at some time in
/// the year (s 25-130(1)(b)): $60,000 of salary less $1,000 is $59,000;
/// (45,000 - 17,016) x 15% + 14,000 x 30% = 4,197.60 + 4,200 = 8,397.60; the
/// offset 325 - 1.5% x 14,000 = 115; the levy 1,180 x 518 / 730 = 837.315...;
/// total 8,397.60 - 115 + 837.32 = 9,119.92, against $9,000 withheld.
#[test]
fn salary_works_out_taxable_income_with_the_standard_deduction() {
    let mut facts = input(
        "2026-27",
        "0",
        json!({"assessable_labour_income": "60000", "tax_withheld": "9000"}),
    );
    facts.as_object_mut().unwrap().remove("taxable_income");
    let a = amounts(&facts);
    check(
        &a,
        &[
            ("standard_deduction", "1000.00"),
            ("taxable_income", "59000.00"),
            ("basic_income_tax", "8397.60"),
            ("low_income_tax_offset_applied", "115.00"),
            ("medicare_levy", "837.32"),
            ("total_before_credits", "9119.92"),
            ("amount_payable", "119.92"),
        ],
    );
}

/// With a dependant who was not a prescribed person, no non-resident day
/// counts (s 251U(2)): the whole levy, 1,200; total 8,877.44 + 1,200.
#[test]
fn with_no_prescribed_non_resident_days_the_whole_levy_is_due() {
    let a = amounts(&input(
        "2025-26",
        "60000",
        json!({"medicare-levy": {"section_9a_adjustment_applies": false}}),
    ));
    check(
        &a,
        &[
            ("medicare_levy", "1200.00"),
            ("total_before_credits", "10077.44"),
        ],
    );
}

#[test]
fn parts_not_composed_for_a_part_year_resident_are_refused() {
    for part in [
        "medicare-levy-surcharge",
        "seniors-pensioners-tax-offset",
        "private-health-insurance-rebate",
        "study-loan-repayment",
        "study-loan-withholding",
        "seniors-pensioners-withholding",
        "payg-withholding",
        "schedule-5-method-a",
    ] {
        let r = refusal(&input("2025-26", "60000", json!({part: {}})));
        assert_eq!(r.code, "estimate.residency_unsupported", "{part}");
        assert!(r.message.contains(part), "{part}");
    }
    // From 2027-28 the working Australians tax offset applies (s 61-155).
    let r = refusal(&input("2027-28", "60000", json!({})));
    assert_eq!(r.code, "estimate.residency_unsupported");
    assert!(r.message.contains("s 61-160"));
    let r = refusal(&input(
        "2025-26",
        "60000",
        json!({"residency": "working_holiday_maker"}),
    ));
    assert_eq!(r.code, "estimate.residency_unsupported");
}

#[test]
fn part_year_inputs_are_refused_where_they_do_not_fit() {
    let mut facts = input("2025-26", "60000", json!({}));
    facts
        .as_object_mut()
        .unwrap()
        .remove("part_year_residency_months");
    assert_eq!(refusal(&facts).code, "input.missing_field");
    let mut facts = input("2025-26", "60000", json!({}));
    facts.as_object_mut().unwrap().remove("eligible_pensioner");
    assert_eq!(refusal(&facts).code, "input.missing_field");
    for residency in ["full_year_resident", "foreign_resident"] {
        for key in ["part_year_residency_months", "eligible_pensioner"] {
            let mut facts = input(
                "2025-26",
                "60000",
                json!({"residency": residency,
                    "medicare-levy": {"section_9a_adjustment_applies": false}}),
            );
            let object = facts.as_object_mut().unwrap();
            let other = if key == "eligible_pensioner" {
                "part_year_residency_months"
            } else {
                "eligible_pensioner"
            };
            object.remove(other);
            if residency == "foreign_resident" {
                object.remove("medicare-levy");
            }
            let r = refusal(&facts);
            assert_eq!(r.code, "input.field_unknown", "{residency} {key}");
            assert!(r.message.contains(key), "{residency} {key}");
        }
    }
    // A full-year resident has no non-resident days.
    let mut facts = input(
        "2025-26",
        "60000",
        json!({"residency": "full_year_resident"}),
    );
    let object = facts.as_object_mut().unwrap();
    object.remove("part_year_residency_months");
    object.remove("eligible_pensioner");
    let r = refusal(&facts);
    assert_eq!(r.code, "input.field_unknown");
    assert!(r.message.contains("non_resident_days"));
    // Resident on at least 1 day in each of the 9 months: at most 356
    // non-resident days in 365.
    let many = json!({"medicare-levy": {"section_9a_adjustment_applies": false,
        "non_resident_days": 357}});
    let r = refusal(&input("2025-26", "60000", many));
    assert_eq!(r.code, "input.day_count_invalid");
    let most = json!({"medicare-levy": {"section_9a_adjustment_applies": false,
        "non_resident_days": 356}});
    // 1,200 x (730 - 712) / 730 = 29.589... .
    assert_eq!(
        amounts(&input("2025-26", "60000", most))["medicare_levy"],
        "29.59"
    );
}

/// One residency month (arrived 20 June 2026): threshold 13,464 + 4,736 / 12
/// = 13,858.666...; basic tax (30,000 - 13,858.666...) x 16% = 2,582.613...;
/// the low income tax offset in full, $700, for a resident at any time in the
/// year (s 61-110(1)(a)); the levy 198.90 x (730 - 708) / 730 = 5.994...
/// on 354 non-resident days; total 2,582.61 - 700 + 5.99 = 1,888.60.
#[test]
fn one_residency_month_keeps_the_full_low_income_offset() {
    let facts = input(
        "2025-26",
        "30000",
        json!({"part_year_residency_months": 1,
            "medicare-levy": {"section_9a_adjustment_applies": false, "non_resident_days": 354}}),
    );
    check(
        &amounts(&facts),
        &[
            ("basic_income_tax", "2582.61"),
            ("low_income_tax_offset_applied", "700.00"),
            ("medicare_levy", "5.99"),
            ("total_before_credits", "1888.60"),
        ],
    );
    // 364 is the most: resident on at least 1 day of the month.
    let most = input(
        "2025-26",
        "30000",
        json!({"part_year_residency_months": 1,
            "medicare-levy": {"section_9a_adjustment_applies": false, "non_resident_days": 365}}),
    );
    assert_eq!(refusal(&most).code, "input.day_count_invalid");
}

/// 12 residency months (arrived 3 July 2025, or 2 periods that between them
/// touch every month): the threshold is the full $18,200, (45,000 - 18,200) x
/// 16% + 4,500 = 8,788; 2 non-resident days, 1,200 x 726 / 730 = 1,193.424...;
/// total 8,788 - 100 + 1,193.42 = 9,881.42. At most 365 - 12 = 353 days.
#[test]
fn twelve_residency_months_keep_the_full_threshold() {
    let levy = |days: i64| {
        json!({"part_year_residency_months": 12,
            "medicare-levy": {"section_9a_adjustment_applies": false, "non_resident_days": days}})
    };
    check(
        &amounts(&input("2025-26", "60000", levy(2))),
        &[
            ("basic_income_tax", "8788.00"),
            ("medicare_levy", "1193.42"),
            ("total_before_credits", "9881.42"),
        ],
    );
    assert!(api::evaluate(COMMAND, &input("2025-26", "60000", levy(353))).is_ok());
    assert_eq!(
        refusal(&input("2025-26", "60000", levy(354))).code,
        "input.day_count_invalid"
    );
}

/// The sentences of a scope that start 'Excludes', which must not name a case
/// the command takes.
fn exclusions(scope: &str) -> Vec<&str> {
    scope
        .split(". ")
        .filter(|sentence| sentence.starts_with("Excludes"))
        .collect()
}

/// Every worksheet that takes a part-year resident's input, and the estimate
/// that composes one, states no exclusion of part-year residence: a scope that
/// still said 'Excludes ... part-year residence' beside `non_resident_days`
/// would tell a caller the case is unsupported.
#[test]
fn no_scope_excludes_the_part_year_residence_it_takes() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut checked = Vec::new();
    for (kind, entry) in catalogue.as_object().unwrap() {
        let optional = &entry["optional_inputs"];
        if ["non_resident_days", "part_year_residency_months"]
            .iter()
            .any(|name| optional.get(name).is_some())
        {
            let scope = entry["scope"].as_str().unwrap();
            assert!(scope.contains("part-year resident"), "{kind}: {scope}");
            for sentence in exclusions(scope) {
                assert!(!sentence.contains("part-year"), "{kind}: {sentence}");
            }
            checked.push(kind.as_str());
        }
    }
    assert!(checked.contains(&"medicare_levy"), "{checked:?}");
    let levy = api::evaluate(
        "medicare-levy",
        &json!({"taxable_income": "60000", "section_9a_adjustment_applies": false,
                "year": "2025-26", "scope_confirmed": true, "non_resident_days": 106}),
    )
    .unwrap();
    let estimate = api::evaluate(COMMAND, &input("2025-26", "60000", json!({}))).unwrap();
    for scope in [&levy["scope"], &estimate["scope"]] {
        for sentence in exclusions(scope.as_str().unwrap()) {
            assert!(!sentence.contains("part-year"), "{sentence}");
        }
    }
}
