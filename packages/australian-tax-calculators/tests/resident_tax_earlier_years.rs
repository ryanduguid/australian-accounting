// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `resident-tax` for 2019-20 to 2023-24: Income Tax Rates Act 1986 Schedule 7
//! Part I clause 1's tables in compilations 54 (6 July 2019), 57 (15 October
//! 2020) and 62 (1 April 2024), read on 26 November 2026, and the ATO's myTax
//! 'Part-year tax-free threshold' pages for 2020 to 2024
//! (docs/calculation-evidence.md#resident-tax).
use australian_tax_calculators::{api, metadata};
use serde_json::{Value, json};

fn tax(income: u64, year: &str) -> Value {
    let r = api::evaluate(
        "resident-tax",
        &json!({"taxable_income": income.to_string(), "year": year, "scope_confirmed": true}),
    )
    .unwrap();
    r["amounts"]["basic_income_tax"].clone()
}

#[test]
fn the_years_run_from_2019_20() {
    let years = metadata::supported_periods("resident_tax");
    assert_eq!(
        years[..6],
        [
            "2019-20", "2020-21", "2021-22", "2022-23", "2023-24", "2024-25"
        ]
    );
    let refused = api::evaluate_coded(
        "resident-tax",
        &json!({"taxable_income": "50000", "year": "2018-19", "scope_confirmed": true}),
    )
    .unwrap_err();
    assert_eq!(refused.code, "period.unsupported");
}

#[test]
fn the_2019_20_table() {
    // 19% to $37,000, 32.5% to $90,000, 37% to $180,000, then 45%: $3,572,
    // $20,797 and $54,097 at the band tops.
    for (income, want) in [
        (18200, "0.00"),
        (18201, "0.19"),
        (37000, "3572.00"),
        (90000, "20797.00"),
        (180000, "54097.00"),
        (200000, "63097.00"),
    ] {
        assert_eq!(tax(income, "2019-20"), want, "{income}");
    }
}

#[test]
fn the_2020_21_to_2023_24_table() {
    // 19% to $45,000, 32.5% to $120,000, 37% to $180,000, then 45%; the 2
    // frozen 2023-24 inputs the Python release refused are the last 2 rows.
    for year in ["2020-21", "2021-22", "2022-23", "2023-24"] {
        for (income, want) in [
            (18200, "0.00"),
            (45000, "5092.00"),
            (120000, "29467.00"),
            (180000, "51667.00"),
            (200000, "60667.00"),
            (54830, "8286.75"),
            (392598, "147336.10"),
        ] {
            assert_eq!(tax(income, year), want, "{year} {income}");
        }
    }
}

#[test]
fn a_part_year_resident_in_an_earlier_year() {
    // s 20(1): $13,464 + $4,736 x 3 / 12 = $14,648.
    let r = api::evaluate(
        "resident-tax",
        &json!({"taxable_income": "20000", "year": "2021-22", "scope_confirmed": true,
                "part_year_residency_months": 3, "eligible_pensioner": false}),
    )
    .unwrap();
    assert_eq!(r["amounts"]["tax_free_threshold"], "14648.00");
    // (20,000 - 14,648) x 19%.
    assert_eq!(r["amounts"]["basic_income_tax"], "1016.88");
}
