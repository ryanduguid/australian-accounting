// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Minors' unearned income (ITAA 1936 Division 6AA; Income Tax Rates Act 1986
//! ss 13 and 15 and Schedule 11, read on 22 November 2026) against the ATO's
//! 'Tax rates if you're under 18 years old' tables (last updated 1 June 2026),
//! TR 92/12 Example 5A's s 13(2) arithmetic and hand-worked cases at each edge
//! (docs/calculation-evidence.md#minors-income).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("minors-income", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("minors-income", input)
        .unwrap_err()
        .code
}

fn with(mut base: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            base.as_object_mut().unwrap().remove(key);
        } else {
            base[key] = value.clone();
        }
    }
    base
}

/// A prescribed person with `wages` of employment income and `unearned` of
/// other assessable income, no deductions, so taxable income is their sum.
fn minor(year: &str, residency: &str, wages: u64, unearned: u64) -> Value {
    let excepted = if wages == 0 {
        json!([])
    } else {
        json!([{"ground": "employment", "amount": format!("{wages}.00")}])
    };
    json!({
        "year": year, "scope_confirmed": true,
        "taxable_income": (wages + unearned).to_string(),
        "residency": residency, "under_18_on_30_june": true, "excepted_person": "none",
        "excepted_income": excepted, "trust_shares": [],
        "other_assessable_income": format!("{unearned}.00"),
        "deductions_exclusive": "0.00", "deductions_related": "0.00",
        "apportionable_deductions": "0.00", "special_income_component": false,
    })
}

fn amount(out: &Value, key: &str) -> String {
    out["amounts"][key].as_str().unwrap().to_string()
}

/// (tax on other income, tax on eligible income, basic income tax).
fn taxes(out: &Value) -> [String; 3] {
    [
        "tax_on_other_income",
        "tax_on_eligible_income",
        "basic_income_tax",
    ]
    .map(|k| amount(out, k))
}

/// ATO table 1 (residents, 2025-26) at each edge, and TR 92/12 Example 5A's
/// $1,000 giving $385.44.
#[test]
fn resident_table_edges() {
    for (unearned, tax) in [
        (0, "0.00"),
        (416, "0.00"),
        (417, "0.66"),
        (1000, "385.44"),
        (1307, "588.06"),
        (1308, "588.60"),
        (2000, "900.00"),
    ] {
        let out = run(&minor("2025-26", "resident", 0, unearned));
        assert_eq!(amount(&out, "basic_income_tax"), tax, "{unearned}");
        assert_eq!(out["prescribed_person"], true);
    }
}

/// ATO tables 3 and 4 (foreign residents): 30% of the whole to $416, $124.80
/// plus 66% of the excess to $713, then 45% of the whole; 2026-27 from the Act.
#[test]
fn foreign_resident_table_edges() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        for (unearned, tax) in [
            (100, "30.00"),
            (416, "124.80"),
            (417, "125.46"),
            (500, "180.24"),
            (713, "320.82"),
            (714, "321.30"),
        ] {
            let out = run(&minor(year, "foreign_resident", 0, unearned));
            assert_eq!(amount(&out, "basic_income_tax"), tax, "{year} {unearned}");
        }
    }
}

/// s 13(10) and s 15(8): the phase-out limits from the year's tables.
#[test]
fn phase_out_limits_and_second_rate() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        let resident = run(&minor(year, "resident", 0, 2000));
        assert_eq!(resident["rates"]["phase_out_limit"], "1307", "{year}");
        assert_eq!(
            resident["rates"]["second_resident_personal_tax_rate"],
            "0.30"
        );
        let foreign = run(&minor(year, "foreign_resident", 0, 2000));
        assert_eq!(foreign["rates"]["phase_out_limit"], "713", "{year}");
    }
}

/// ATO table 1 row 3 with table 5: wages at ordinary rates, the dividends at
/// 45%; and each year's first band (16%, 16%, 15%).
#[test]
fn wages_and_unearned_income_by_year() {
    let out = run(&minor("2025-26", "resident", 28000, 2000));
    assert_eq!(taxes(&out), ["1568.00", "900.00", "2468.00"]);
    assert_eq!(amount(&out, "excepted_assessable_income"), "28000.00");
    assert_eq!(amount(&out, "other_taxable_income"), "28000.00");
    for (year, other) in [
        ("2024-25", "1888.00"),
        ("2025-26", "1888.00"),
        ("2026-27", "1770.00"),
    ] {
        let out = run(&minor(year, "resident", 30000, 2000));
        assert_eq!(taxes(&out)[0], other, "{year}");
        assert_eq!(taxes(&out)[1], "900.00", "{year}");
    }
}

/// The ATO's note: the top-slice tax at ordinary rates when it is greater
/// than 66% of the excess (s 13(2)(b)), and 45% when the slice is at 45%.
#[test]
fn top_slice_beats_the_phase_in() {
    let out = run(&minor("2025-26", "resident", 45000, 500));
    assert_eq!(taxes(&out), ["4288.00", "150.00", "4438.00"]);
    let out = run(&minor("2025-26", "resident", 199000, 1000));
    assert_eq!(taxes(&out)[1], "450.00");
    // Eligible taxable income of $416 or less: ordinary rates on the whole.
    let out = run(&minor("2025-26", "resident", 45000, 400));
    assert_eq!(taxes(&out), ["4288.00", "120.00", "4408.00"]);
}

/// s 102AD(c): apportionable deductions in proportion.
#[test]
fn apportionable_deductions() {
    let gift = json!({"apportionable_deductions": "100.00", "taxable_income": "1900"});
    let out = run(&with(minor("2025-26", "resident", 0, 2000), gift));
    assert_eq!(amount(&out, "apportioned_deductions"), "100.00");
    assert_eq!(amount(&out, "eligible_taxable_income"), "1900.00");
    assert_eq!(amount(&out, "basic_income_tax"), "855.00");
    let gift = json!({"apportionable_deductions": "100.00", "taxable_income": "3900"});
    let out = run(&with(minor("2025-26", "resident", 2000, 2000), gift));
    assert_eq!(amount(&out, "apportioned_deductions"), "50.00");
    assert_eq!(amount(&out, "eligible_taxable_income"), "1950.00");
    assert_eq!(taxes(&out), ["0.00", "877.50", "877.50"]);
}

/// s 102AD(a) and (b); nil when the deductions exceed eligible income.
#[test]
fn related_deductions() {
    let out = run(&with(
        minor("2025-26", "resident", 1000, 1500),
        json!({"deductions_exclusive": "300.00", "deductions_related": "100.00", "taxable_income": "2100"}),
    ));
    assert_eq!(amount(&out, "eligible_taxable_income"), "1100.00");
    assert_eq!(amount(&out, "tax_on_eligible_income"), "451.44");
    let out = run(&with(
        minor("2025-26", "resident", 1000, 500),
        json!({"deductions_exclusive": "600.00", "taxable_income": "900"}),
    ));
    assert_eq!(amount(&out, "eligible_taxable_income"), "0.00");
}

/// s 102AE(2)(e) and (4), s 102AG: only the stated part of a trust share.
#[test]
fn trust_share_part() {
    let out = run(&with(
        minor("2025-26", "resident", 0, 0),
        json!({
            "trust_shares": [{"amount": "5000.00", "division_6aa_part": "3000.00"}],
            "deductions_exclusive": "200.00", "taxable_income": "4800",
        }),
    ));
    assert_eq!(amount(&out, "excepted_assessable_income"), "2000.00");
    assert_eq!(amount(&out, "eligible_assessable_income"), "3000.00");
    assert_eq!(amount(&out, "eligible_taxable_income"), "2800.00");
    assert_eq!(taxes(&out), ["0.00", "1260.00", "1260.00"]);
}

/// s 102AC: not under 18, or an excepted person, pays ordinary rates.
#[test]
fn not_a_prescribed_person() {
    let adult = run(&with(
        minor("2025-26", "resident", 0, 2000),
        json!({"under_18_on_30_june": false}),
    ));
    assert_eq!(adult["prescribed_person"], false);
    assert_eq!(amount(&adult, "basic_income_tax"), "0.00");
    let facts = |education: bool| {
        json!({"excepted_person": "full_time_occupation", "full_time_occupation": {
            "engaged_on_last_day": false, "three_months_in_year": true,
            "intends_full_time_occupation_next_year": true,
            "intends_full_time_education_next_year": education,
        }})
    };
    let worker = run(&with(minor("2025-26", "resident", 0, 2000), facts(false)));
    assert_eq!(worker["prescribed_person"], false);
    assert_eq!(amount(&worker, "basic_income_tax"), "0.00");
    // s 102AC(8)(b): intending full-time study next year.
    let student = run(&with(minor("2025-26", "resident", 0, 2000), facts(true)));
    assert_eq!(student["prescribed_person"], true);
    assert_eq!(amount(&student, "basic_income_tax"), "900.00");
    let pension = run(&with(
        minor("2025-26", "resident", 0, 2000),
        json!({"excepted_person": "disability_support_pension"}),
    ));
    assert_eq!(pension["prescribed_person"], false);
}

/// s 102AC(3) to (5): a double orphan dependent on relatives is not excepted.
#[test]
fn dependence_on_relatives() {
    for (dependent, prescribed) in [(true, true), (false, false)] {
        let out = run(&with(
            minor("2025-26", "resident", 0, 2000),
            json!({"excepted_person": "double_orphan_pension", "dependent_on_relatives": dependent}),
        ));
        assert_eq!(out["prescribed_person"], prescribed);
    }
}

/// Each refusal with its code.
#[test]
fn refusals() {
    let base = minor("2025-26", "resident", 0, 2000);
    for (changes, expected) in [
        (json!({"year": "2023-24"}), "period.unsupported"),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (
            json!({"special_income_component": true}),
            "minors_income.special_income_component",
        ),
        (
            json!({"residency": "part_year"}),
            "minors_income.residency_invalid",
        ),
        (
            json!({"excepted_person": "student"}),
            "minors_income.excepted_person_invalid",
        ),
        (
            json!({"excepted_person": "full_time_occupation"}),
            "input.missing_field",
        ),
        (
            json!({"excepted_person": "full_time_occupation", "full_time_occupation": {"engaged_on_last_day": true}}),
            "input.missing_field",
        ),
        (
            json!({"excepted_person": "double_orphan_pension"}),
            "input.missing_field",
        ),
        (
            json!({"dependent_on_relatives": false}),
            "input.field_unknown",
        ),
        (
            json!({"excepted_person": "carer_allowance", "dependent_on_relatives": false}),
            "input.field_unknown",
        ),
        (
            json!({"full_time_occupation": {}, "excepted_person": "carer_allowance"}),
            "input.field_unknown",
        ),
        (
            json!({"excepted_income": [{"ground": "pocket_money", "amount": "1.00"}]}),
            "minors_income.excepted_income_invalid",
        ),
        (
            json!({"excepted_income": {}}),
            "minors_income.excepted_income_invalid",
        ),
        (
            json!({"trust_shares": [{"amount": "1.00", "division_6aa_part": "2.00"}]}),
            "minors_income.trust_shares_invalid",
        ),
        (
            json!({"taxable_income": "2001"}),
            "minors_income.facts_inconsistent",
        ),
        (
            json!({"taxable_income": "1999.50"}),
            "input.amount_not_whole_dollars",
        ),
        (json!({"under_18_on_30_june": "yes"}), "input.flag_invalid"),
    ] {
        assert_eq!(
            code(&with(base.clone(), changes.clone())),
            expected,
            "{changes}"
        );
    }
    // A loss against excepted income leaves eligible above taxable income.
    let loss = with(
        minor("2025-26", "resident", 1000, 2000),
        json!({"taxable_income": "1500"}),
    );
    assert_eq!(code(&loss), "minors_income.facts_inconsistent");
}

/// 50 items are taken and 51 refused.
#[test]
fn item_limit() {
    let item = json!({"ground": "employment", "amount": "1.00"});
    let fifty = with(
        minor("2025-26", "resident", 0, 2000),
        json!({"excepted_income": vec![item.clone(); 50], "taxable_income": "2050"}),
    );
    assert_eq!(amount(&run(&fifty), "excepted_assessable_income"), "50.00");
    let fifty_one = with(fifty, json!({"excepted_income": vec![item; 51]}));
    assert_eq!(code(&fifty_one), "minors_income.excepted_income_invalid");
}

/// Review finding S1: s 102AG(1) applies only to a prescribed beneficiary,
/// and the excepted person ground is not tested for a person 18 or over.
#[test]
fn trust_share_and_ground_of_a_non_prescribed_person() {
    let share = json!({
        "trust_shares": [{"amount": "5000.00", "division_6aa_part": "3000.00"}],
        "taxable_income": "5000",
    });
    let minor_share = run(&with(minor("2025-26", "resident", 0, 0), share.clone()));
    assert_eq!(
        amount(&minor_share, "eligible_assessable_income"),
        "3000.00"
    );
    let adult = with(
        minor("2025-26", "resident", 0, 0),
        json!({"under_18_on_30_june": false}),
    );
    let out = run(&with(adult.clone(), share));
    assert_eq!(amount(&out, "excepted_assessable_income"), "5000.00");
    assert_eq!(amount(&out, "eligible_assessable_income"), "0.00");
    assert_eq!(amount(&out, "basic_income_tax"), "0.00");
    let out = run(&with(
        adult,
        json!({"excepted_person": "full_time_occupation", "taxable_income": "0"}),
    ));
    assert_eq!(out["prescribed_person"], false);
    let worker = with(
        minor("2025-26", "resident", 0, 2000),
        json!({"excepted_person": "full_time_occupation"}),
    );
    assert_eq!(code(&worker), "input.missing_field");
}

/// s 102AD(c)'s other branches: a nil numerator with apportionable
/// deductions, a nil denominator, and negative deductions refused.
#[test]
fn apportionment_edges() {
    let out = run(&with(
        minor("2025-26", "resident", 1000, 500),
        json!({"deductions_exclusive": "600.00", "apportionable_deductions": "100.00", "taxable_income": "800"}),
    ));
    assert_eq!(amount(&out, "apportioned_deductions"), "0.00");
    assert_eq!(amount(&out, "eligible_taxable_income"), "0.00");
    let nil = run(&minor("2025-26", "resident", 0, 0));
    assert_eq!(taxes(&nil), ["0.00", "0.00", "0.00"]);
    let no_taxable_income = with(
        minor("2025-26", "resident", 0, 100),
        json!({"taxable_income": "0"}),
    );
    assert_eq!(code(&no_taxable_income), "minors_income.facts_inconsistent");
    for key in [
        "deductions_exclusive",
        "deductions_related",
        "apportionable_deductions",
    ] {
        let negative = with(minor("2025-26", "resident", 0, 2000), json!({key: "-1.00"}));
        assert_eq!(code(&negative), "input.amount_invalid", "{key}");
    }
}

/// s 15(2): the foreign resident top-slice tax at 37% beats both phase-in
/// amounts.
#[test]
fn foreign_top_slice() {
    let out = run(&minor("2025-26", "foreign_resident", 135000, 500));
    assert_eq!(taxes(&out), ["40500.00", "185.00", "40685.00"]);
    let out = run(&minor("2025-26", "foreign_resident", 135000, 400));
    assert_eq!(taxes(&out)[1], "148.00");
}

/// A full-time occupation fact that is not a boolean is refused, naming it.
#[test]
fn a_full_time_occupation_fact_must_be_a_boolean() {
    let input = with(
        minor("2025-26", "resident", 0, 2000),
        json!({"excepted_person": "full_time_occupation", "full_time_occupation": {
            "engaged_on_last_day": "yes", "three_months_in_year": true,
            "intends_full_time_occupation_next_year": true,
            "intends_full_time_education_next_year": false,
        }}),
    );
    let error = api::evaluate_coded("minors-income", &input).unwrap_err();
    assert_eq!(error.code, "input.flag_invalid");
    assert_eq!(
        error.message,
        "full_time_occupation: engaged_on_last_day must be a boolean."
    );
}
