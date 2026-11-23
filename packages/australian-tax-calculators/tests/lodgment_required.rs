// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Whether an individual must lodge, against each year's lodgment instrument
//! read on the Federal Register of Legislation on 22 November 2026 (the
//! Notices for 2020-21 to 2022-23 and the Taxation Laws instruments for
//! 2023-24 to 2025-26): each threshold on both sides, each exception and
//! each refusal (docs/calculation-evidence.md#lodgment-required). The
//! instruments publish no worked examples, so the facts are fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const YEARS: [&str; 6] = [
    "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26",
];

fn run(input: &Value) -> Value {
    api::evaluate("lodgment-required", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("lodgment-required", input)
        .unwrap_err()
        .code
}

/// A full-year resident adult with nothing else.
fn base(year: &str, taxable: &str) -> Value {
    json!({
        "year": year, "scope_confirmed": true, "residency": {"status": "whole_year"},
        "taxable_income": taxable, "dividends_and_franking_credits": "0.00",
        "under_18": null, "triggers": [], "study_loan": null,
        "working_holiday_maker": null, "listed_payment": null, "sapto": null,
        "child_support_parent": false,
    })
}

fn with(mut input: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            input.as_object_mut().unwrap().remove(key);
        } else {
            input[key] = value.clone();
        }
    }
    input
}

/// Whether a return is required, and the provisions requiring it.
fn decided(input: &Value) -> (bool, Vec<String>) {
    let got = run(input);
    let by = got["required_by"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    (got["lodgment_required"].as_bool().unwrap(), by)
}

fn notice(year: &str) -> bool {
    year < "2023-24"
}

#[test]
fn a_full_year_resident_lodges_over_18200() {
    for year in YEARS {
        let cite = if notice(year) {
            "Table B item (1)(a)"
        } else {
            "subparagraph 6(3)(a)(i)"
        };
        assert_eq!(decided(&base(year, "18200.00")), (false, vec![]), "{year}");
        assert_eq!(
            decided(&base(year, "18200.01")),
            (true, vec![cite.to_owned()]),
            "{year}"
        );
    }
}

/// The lesser of $18,200 and $13,464 plus $395 for each month resident.
#[test]
fn a_part_year_resident_lodges_over_the_lesser_amount() {
    for (months, limit) in [(1, 13_859), (5, 15_439), (11, 17_809), (12, 18_200)] {
        let input = |taxable: i64| {
            with(
                base("2025-26", &format!("{taxable}.00")),
                json!({"residency": {"status": "part_year", "months": months}}),
            )
        };
        let got = run(&input(limit));
        assert_eq!(got["lodgment_required"], json!(false), "{months}");
        assert_eq!(got["tests"]["part_year_limit"], json!(limit.to_string()));
        assert_eq!(
            decided(&input(limit + 1)),
            (true, vec!["subparagraph 6(3)(a)(ii)".to_owned()]),
            "{months}"
        );
    }
}

#[test]
fn a_non_resident_lodges_with_income_taxable_here() {
    let none = |taxable: bool| {
        with(
            base("2025-26", "1.00"),
            json!({"residency": {"status": "none", "taxable_australian_income": taxable}}),
        )
    };
    assert_eq!(
        decided(&none(true)),
        (true, vec!["paragraph 6(3)(b)".to_owned()])
    );
    assert_eq!(decided(&none(false)), (false, vec![]));
}

#[test]
fn a_minor_lodges_over_416() {
    let minor = |own: &str, dividends: &str| {
        with(
            base("2025-26", "0.00"),
            json!({"under_18": {"income_excluding_work": own},
                "dividends_and_franking_credits": dividends}),
        )
    };
    let cite = vec!["paragraph 6(2)(i)".to_owned()];
    assert_eq!(decided(&minor("416.00", "416.00")), (false, vec![]));
    assert_eq!(decided(&minor("416.01", "0.00")), (true, cite.clone()));
    assert_eq!(decided(&minor("0.00", "416.01")), (true, cite));
    let old = with(minor("417.00", "0.00"), json!({"year": "2022-23"}));
    assert_eq!(decided(&old), (true, vec!["Table A item (9)".to_owned()]));
}

#[test]
fn dividends_over_18200_need_a_return() {
    let input = |amount: &str| {
        with(
            base("2025-26", "0.00"),
            json!({"dividends_and_franking_credits": amount}),
        )
    };
    assert_eq!(decided(&input("18200.00")), (false, vec![]));
    assert_eq!(
        decided(&input("18200.01")),
        (true, vec!["paragraph 6(2)(o)".to_owned()])
    );
}

#[test]
fn each_stated_condition_needs_a_return() {
    let keys = [
        ("payg_withheld", "(a)", "(1)"),
        ("tax_loss", "(b)", "(2)"),
        ("net_capital_loss", "(c)", "(3)"),
        ("business", "(f)", "(6)"),
        ("primary_production_trust", "(g)", "(7)"),
        ("primary_production_partnership", "(h)", "(8)"),
        ("exempt_overseas_income", "(j)", "(10)"),
        ("payg_instalment", "(k)", "(11)"),
        ("special_professional", "(l)", "(12)"),
        ("private_health_offset", "(m)", "(13)"),
        ("reportable_amounts", "(n)", "(14)"),
        ("co_contribution", "(p)", "(16)"),
        ("contributions_cap", "(q)", "(17)"),
        ("super_lump_sum", "(r)", "(18)"),
    ];
    for (key, new, old) in keys {
        let input = with(base("2025-26", "0.00"), json!({"triggers": [key, key]}));
        assert_eq!(
            decided(&input),
            (true, vec![format!("paragraph 6(2){new}")]),
            "{key}"
        );
        let input = with(input, json!({"year": "2021-22"}));
        assert_eq!(
            decided(&input),
            (true, vec![format!("Table A item {old}")]),
            "{key}"
        );
    }
}

/// s 6(10) [Table J]: each year's income amount, for a foreign resident.
#[test]
fn a_foreign_resident_with_a_study_debt_lodges_over_the_years_amount() {
    let amounts = [11_655, 11_753, 12_090, 12_887, 13_609, 16_750];
    for (year, amount) in YEARS.iter().zip(amounts) {
        let input = |income: i64| {
            with(
                base(year, "0.00"),
                json!({"residency": {"status": "none", "taxable_australian_income": false},
                    "study_loan": {"income": format!("{income}.00")}}),
            )
        };
        let cite = if notice(year) {
            "Table J"
        } else {
            "subsection 6(10)"
        };
        assert_eq!(decided(&input(amount)), (false, vec![]), "{year}");
        assert_eq!(
            decided(&input(amount + 1)),
            (true, vec![cite.to_owned()]),
            "{year}"
        );
    }
    let resident = with(
        base("2025-26", "0.00"),
        json!({"study_loan": {"income": "90000.00"}}),
    );
    assert_eq!(decided(&resident), (false, vec![]));
}

/// s 7(1)(f) [Table P] relieves every requirement.
#[test]
fn a_working_holiday_maker_under_45001_is_relieved() {
    let input = |taxable: &str, other: bool| {
        with(
            base("2025-26", taxable),
            json!({"triggers": ["payg_withheld"], "working_holiday_maker": {
                "other_assessable_income": other,
                "taxable_income_excluding_remainders": taxable}}),
        )
    };
    let got = run(&input("45000.00", false));
    assert_eq!(got["lodgment_required"], json!(false));
    assert_eq!(got["exceptions_applied"], json!(["paragraph 7(1)(f)"]));
    assert_eq!(got["due_date"], Value::Null);
    assert!(decided(&input("45001.00", false)).0);
    assert!(decided(&input("30000.00", true)).0);
    let old = with(input("45000.00", false), json!({"year": "2020-21"}));
    assert_eq!(run(&old)["exceptions_applied"], json!(["Table P"]));
}

/// s 7(2) [Table K(1)]: 'less than' $21,885 to 2023-24 and $22,575 in
/// 2024-25; 'less than or equal to' $22,575 in 2025-26.
#[test]
fn a_listed_payment_relieves_under_the_years_amount() {
    let cases = [
        ("2020-21", 21_884, false),
        ("2020-21", 21_885, true),
        ("2023-24", 21_884, false),
        ("2023-24", 21_885, true),
        ("2024-25", 22_574, false),
        ("2024-25", 22_575, true),
        ("2025-26", 22_575, false),
        ("2025-26", 22_576, true),
    ];
    for (year, taxable, required) in cases {
        let input = with(
            base(year, &format!("{taxable}.00")),
            json!({"listed_payment": {"other_assessable_income": true}}),
        );
        assert_eq!(
            run(&input)["lodgment_required"],
            json!(required),
            "{year} {taxable}"
        );
    }
    let only = with(
        base("2025-26", "30000.00"),
        json!({"listed_payment": {"other_assessable_income": false}}),
    );
    assert_eq!(decided(&only), (false, vec![]));
}

/// s 7(3) [Table K(2)]: rebate income under the category's amount.
#[test]
fn a_seniors_offset_relieves_under_the_years_amount() {
    let amounts = [
        ("2020-21", [32_280, 31_280, 28_975]),
        ("2023-24", [32_280, 31_280, 28_975]),
        ("2024-25", [34_919, 33_732, 30_994]),
        ("2025-26", [34_919, 33_732, 30_994]),
    ];
    for (year, limits) in amounts {
        for (category, limit) in ["single", "apart", "together"].into_iter().zip(limits) {
            let input = |rebate: i64| {
                with(
                    base(year, "30000.00"),
                    json!({"sapto": {"rebate_income": format!("{rebate}.00"), "category": category}}),
                )
            };
            assert!(!decided(&input(limit - 1)).0, "{year} {category}");
            assert!(decided(&input(limit)).0, "{year} {category}");
        }
    }
}

/// An exception that holds for a person covered by a provision it names
/// leaves nothing required under the instrument, whatever else they meet;
/// for anyone else it relieves nothing.
#[test]
fn an_exception_relieves_the_instrument_only_for_a_person_it_covers() {
    for year in ["2022-23", "2025-26"] {
        let both = run(&with(
            base(year, "25000.00"),
            json!({"triggers": ["payg_withheld"],
                "sapto": {"rebate_income": "25000.00", "category": "single"}}),
        ));
        assert_eq!(both["lodgment_required"], false, "{year}");
        assert_eq!(both["exceptions_applied"].as_array().unwrap().len(), 1);
    }
    let withheld_only = run(&with(
        base("2025-26", "15000.00"),
        json!({"triggers": ["payg_withheld"],
            "sapto": {"rebate_income": "15000.00", "category": "single"}}),
    ));
    assert_eq!(withheld_only["lodgment_required"], true);
    assert_eq!(withheld_only["exceptions_applied"], json!([]));
    // s 7(2) names s 6(2)(i); section 4 of the Notices reaches Table B only.
    let minor = |year: &str| {
        with(
            base(year, "10000.00"),
            json!({"under_18": {"income_excluding_work": "500.00"},
                "listed_payment": {"other_assessable_income": false}}),
        )
    };
    assert_eq!(decided(&minor("2024-25")), (false, vec![]));
    assert_eq!(
        decided(&minor("2022-23")),
        (true, vec!["Table A item (9)".to_owned()])
    );
}

#[test]
fn the_due_date_is_31_october_after_the_year() {
    for (year, due) in YEARS.iter().zip([
        "2021-10-31",
        "2022-10-31",
        "2023-10-31",
        "2024-10-31",
        "2025-10-31",
        "2026-10-31",
    ]) {
        let got = run(&base(year, "50000.00"));
        assert_eq!(got["due_date"], json!(due), "{year}");
        let id = got["instrument"]["id"].as_str().unwrap();
        assert_eq!(&id[1..5], &due[..4], "{year}");
    }
    let got = run(&base("2025-26", "50000.00"));
    assert_eq!(
        got["instrument"]["title"],
        json!("Taxation Laws (Requirement to Lodge a Return for the 2026 Year) Instrument 2026")
    );
    let got = run(&base("2020-21", "50000.00"));
    assert_eq!(
        got["instrument"]["title"],
        json!("Notice of Requirement to Lodge a Return for the Income Year Ended 30 June 2021")
    );
}

#[test]
fn a_child_support_parent_not_otherwise_required_is_refused() {
    let parent = with(
        base("2025-26", "10000.00"),
        json!({"child_support_parent": true}),
    );
    assert_eq!(code(&parent), "lodgment_required.child_support_parent");
    let required = with(parent, json!({"taxable_income": "50000.00"}));
    assert!(decided(&required).0);
}

#[test]
fn bad_inputs_are_refused() {
    let cases = [
        (
            json!({"year": "2026-27"}),
            "lodgment_required.instrument_not_made",
        ),
        (json!({"year": "2019-20"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (
            json!({"scope_confirmed": false, "year": "2026-27"}),
            "scope.not_confirmed",
        ),
        (
            json!({"residency": {"status": "half"}}),
            "lodgment_required.residency_invalid",
        ),
        (
            json!({"residency": {"status": "part_year", "months": 13}}),
            "lodgment_required.residency_invalid",
        ),
        (
            json!({"residency": {"status": "part_year"}}),
            "lodgment_required.residency_invalid",
        ),
        (
            json!({"residency": {"status": "whole_year", "months": 3}}),
            "input.field_unknown",
        ),
        (
            json!({"residency": {"status": "none"}}),
            "input.missing_field",
        ),
        (
            json!({"residency": "resident"}),
            "lodgment_required.residency_invalid",
        ),
        (
            json!({"triggers": ["lottery_win"]}),
            "lodgment_required.triggers_invalid",
        ),
        (
            json!({"triggers": "payg_withheld"}),
            "lodgment_required.triggers_invalid",
        ),
        (json!({"under_18": true}), "lodgment_required.group_invalid"),
        (json!({"under_18": {}}), "input.missing_field"),
        (json!({"study_loan": {"income": 5}}), "input.amount_invalid"),
        (
            json!({"sapto": {"rebate_income": "1.00", "category": "couple"}}),
            "lodgment_required.group_invalid",
        ),
        (
            json!({"listed_payment": {"other_assessable_income": "no"}}),
            "input.flag_invalid",
        ),
        (json!({"child_support_parent": "no"}), "input.flag_invalid"),
        (json!({"taxable_income": "-1"}), "input.amount_invalid"),
        (json!({"sapto": "remove"}), "input.missing_field"),
        (
            json!({"residency_status": "whole_year"}),
            "input.field_unknown",
        ),
    ];
    for (change, expected) in cases {
        assert_eq!(
            code(&with(base("2025-26", "0.00"), change.clone())),
            expected,
            "{change}"
        );
    }
}

/// The catalogue's example: tax withheld from a resident employee under the
/// tax-free threshold still needs a return (s 6(2)(a)).
#[test]
fn the_catalogue_example_needs_a_return() {
    let input = with(
        base("2025-26", "15000.00"),
        json!({"triggers": ["payg_withheld"]}),
    );
    assert_eq!(
        decided(&input),
        (true, vec!["paragraph 6(2)(a)".to_owned()])
    );
    assert_eq!(run(&input)["due_date"], json!("2026-10-31"));
}
