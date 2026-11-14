// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Foreign resident basic tax (Income Tax Rates Act 1986 Schedule 7 Part II)
//! against the ATO's published table for 2024-25 and 2025-26, each bracket
//! boundary in every supported year, and the refusals
//! (docs/calculation-evidence.md#foreign-resident-tax).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "foreign-resident-tax";

fn input(income: &str, year: &str) -> Value {
    json!({"taxable_income": income, "year": year, "scope_confirmed": true})
}

fn evaluate(income: &str, year: &str) -> Value {
    api::evaluate(COMMAND, &input(income, year)).unwrap()
}

fn tax(income: i64, year: &str) -> String {
    let result = evaluate(&income.to_string(), year);
    result["amounts"]["basic_income_tax"]
        .as_str()
        .unwrap()
        .to_string()
}

/// `cents` as dollars and cents.
fn money(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

/// ATO, Tax rates - foreign resident (last updated 1 June 2026), read on
/// 14 November 2026, for 2024-25 and 2025-26: 30c for each $1 to $135,000;
/// $40,500 plus 37c for each $1 over $135,000 to $190,000; $60,850 plus 45c
/// for each $1 over $190,000. The table is checked at each boundary, a dollar
/// either side and above the top threshold.
#[test]
fn the_ato_table_for_2024_25_and_2025_26() {
    let table = |income: i64| -> i64 {
        // In cents: the ATO's base tax plus its rate on the excess.
        if income <= 135_000 {
            income * 30
        } else if income <= 190_000 {
            4_050_000 + (income - 135_000) * 37
        } else {
            6_085_000 + (income - 190_000) * 45
        }
    };
    for year in ["2024-25", "2025-26"] {
        for income in [
            0, 1, 18_200, 134_999, 135_000, 135_001, 189_999, 190_000, 190_001, 250_000,
        ] {
            assert_eq!(tax(income, year), money(table(income)), "{year} {income}");
        }
        let result = evaluate("45000", year);
        assert_eq!(result["warnings"].as_array().unwrap().len(), 2, "{year}");
    }
}

/// Every bracket boundary in every supported year, worked from Schedule 7
/// Part II: 2026-27 has the same second and third resident personal tax rates
/// (30% and 37%), so the same amounts, and a warning that the ATO had not yet
/// published its 2026-27 table.
#[test]
fn every_bracket_boundary_in_each_year() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        for (income, expected) in [
            (0, "0.00"),
            (1, "0.30"),
            (134_999, "40499.70"),
            (135_000, "40500.00"),
            (135_001, "40500.37"),
            (189_999, "60849.63"),
            (190_000, "60850.00"),
            (190_001, "60850.45"),
        ] {
            assert_eq!(tax(income, year), expected, "{year} {income}");
        }
        let result = evaluate("200000", year);
        assert_eq!(
            result["rates"],
            json!({"over_0": "0.30", "over_135000": "0.37", "over_190000": "0.45"})
        );
        assert_eq!(result["calculation"], "foreign_resident_tax");
        assert_eq!(result["period"], year);
    }
    let later = evaluate("200000", "2026-27");
    let warnings = later["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 3);
    assert_eq!(
        warnings[2],
        "2026-27 uses the rates Income Tax Rates Act 1986 Schedule 7 Part II sets for the \
         2024-25 and later years, with the second and third resident personal tax rates for \
         2026-27 (30% and 37%), as compiled at 1 July 2026 (compilation 66). The ATO had not \
         published 2026-27 foreign resident rates when this was checked on 14 November 2026, \
         and a later Act can change them for 2026-27 and change this result."
    );
}

/// No tax-free threshold: the first dollar is taxed at 30%, where the resident
/// scale taxes nothing to $18,200.
#[test]
fn a_foreign_resident_has_no_tax_free_threshold() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        assert_eq!(tax(18_200, year), "5460.00", "{year}");
        let resident = api::evaluate("resident-tax", &input("18200", year)).unwrap();
        assert_eq!(resident["amounts"]["basic_income_tax"], "0.00", "{year}");
    }
}

/// Basic tax only: no Medicare levy or offset amount, and the scope says why
/// neither applies to a foreign resident.
#[test]
fn the_result_holds_basic_tax_only_and_says_why() {
    let result = evaluate("100000", "2025-26");
    assert_eq!(
        result["amounts"],
        json!({"taxable_income_used": "100000.00", "basic_income_tax": "30000.00"})
    );
    let scope = result["scope"].as_str().unwrap();
    for needle in [
        "There is no tax-free threshold",
        "pays no Medicare levy, which ITAA 1936 s 251S(1) charges on a person resident",
        "ITAA 1997 s 61-110(1) gives only to a resident",
        "Excludes part-year residents, who are taxed as residents",
        // Each payment that makes a non-resident pay resident rates, on the
        // conditions s 3(1) sets: payable at any time in the year and taxed.
        "not one to whom, at any time in the year, compensation or a pension, allowance or \
         benefit is payable under the Veterans' Entitlements Act 1986, subsection 4(6) of the \
         Veterans' Entitlements (Transitional Provisions and Consequential Amendments) Act 1986, \
         the Military Rehabilitation and Compensation Act 2004 or the Social Security Act 1991 \
         on which the person is liable to pay income tax in Australia",
    ] {
        assert!(scope.contains(needle), "{needle}");
    }
    let sources = result["sources"].as_array().unwrap();
    assert!(sources.contains(&json!(
        "https://www.ato.gov.au/tax-rates-and-codes/tax-rates-foreign-residents"
    )));
}

#[test]
fn unsupported_and_invalid_inputs_are_refused() {
    let refused = |facts: &Value| api::evaluate_coded(COMMAND, facts).unwrap_err();
    // 2027-28 is refused, as resident-tax refuses it.
    let later = refused(&input("100000", "2027-28"));
    assert_eq!(later.code, "period.unsupported");
    assert_eq!(
        later.message,
        "Unsupported period '2027-28'; supported: 2024-25, 2025-26, 2026-27."
    );
    assert_eq!(
        refused(&input("100000", "2023-24")).code,
        "period.unsupported"
    );
    assert_eq!(
        refused(&input("100000.50", "2025-26")).code,
        "input.amount_not_whole_dollars"
    );
    assert_eq!(
        refused(&input("-1", "2025-26")).code,
        "input.amount_invalid"
    );
    let mut unconfirmed = input("100000", "2025-26");
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(refused(&unconfirmed).code, "scope.not_confirmed");
    let mut extra = input("100000", "2025-26");
    extra["tax_free_threshold"] = json!(true);
    let field = refused(&extra);
    assert_eq!(field.code, "input.field_unknown");
    assert_eq!(
        field.message,
        "tax_free_threshold is not an input of foreign-resident-tax"
    );
}

#[test]
fn the_catalogue_lists_it() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["foreign_resident_tax"];
    assert_eq!(
        entry["required_inputs"],
        json!({"kind": "foreign_resident_tax",
               "scope_confirmed": "boolean true; establish every scope condition",
               "year": "income-year string", "taxable_income": "AUD, whole dollars"})
    );
    let mut example = entry["example"]["facts"].clone();
    example.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate(COMMAND, &example).unwrap();
    assert_eq!(result["amounts"]["basic_income_tax"], "13500.00");
    let periods = api::evaluate("periods", &json!({"kind": "foreign_resident_tax"})).unwrap();
    let ids: Vec<&str> = periods
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["2024-25", "2025-26", "2026-27"]);
}
