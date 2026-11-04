// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: FSL-1.1-ALv2

//! The borrowing expenses worksheet (ITAA 1997 s 25-25) against the ATO's and
//! the Library's worked examples (docs/calculation-evidence.md#borrowing-expenses),
//! the $100 rule, the write-off in the year of repayment and its refusals.
//! The ATO's examples give a day and a month only. The years used put each
//! one's Year 4 on 2023-24, which holds 29 February 2024, as each says Year 4
//! is a leap year, so its Years 5 and 6 are 2024-25 and 2025-26.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "borrowing_expenses": "1600.00", "other_borrowing_expenses_in_year": "0.00",
        "income_use": "1", "loan_start": "2024-07-03", "loan_term_end": null,
        "repaid_on": null, "section_26_50_applies": false, "year": "2024-25",
        "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("borrowing-expenses", &input)
}

fn refusal(overrides: Value) -> String {
    run(overrides).unwrap_err()
}

/// The amounts in order: remaining expenditure, maximum deduction, deduction
/// and the remaining expenditure after the year.
fn amounts(result: &Value) -> [&str; 4] {
    [
        "remaining_expenditure",
        "maximum_deduction",
        "deduction",
        "remaining_expenditure_after_year",
    ]
    .map(|key| result["amounts"][key].as_str().unwrap())
}

fn days(result: &Value) -> (&str, &str) {
    let rates = &result["rates"];
    (
        rates["days_in_year"].as_str().unwrap(),
        rates["remaining_loan_period_days"].as_str().unwrap(),
    )
}

/// Peter (ATO, Borrowing expenses): $1,600 on a 25-year loan from 3 July
/// 2020, so 5 years, 3 July 2020 to 2 July 2025, 1,826 days. Each year's
/// maximum is the remaining expenditure × days in the year ÷ days left, to
/// cents, carried forward at that amount:
/// - 2020-21: $1,600 × 363 ÷ 1,826 = $318.0723, so $318.07 (the ATO's figure)
/// - 2021-22: $1,281.93 × 365 ÷ 1,463 = $319.8253, so $319.83
/// - 2022-23: $962.10 × 365 ÷ 1,098 = $319.8238, so $319.82
/// - 2023-24: $642.28 × 366 ÷ 733 = $320.7019, so $320.70
/// - 2024-25: $321.58 × 365 ÷ 367 = $319.8275, so $319.83, leaving $1.75:
///   $1,598.25 claimed by the end of Year 5, as the ATO says
/// - 2025-26: $1.75 × 2 ÷ 2 = $1.75 (the ATO's figure)
#[test]
fn peter_claims_years_5_and_6_as_the_ato_does() {
    let peter = |year: &str| {
        run(json!({"loan_start": "2020-07-03", "loan_term_end": "2045-07-02", "year": year}))
            .unwrap()
    };
    let year_5 = peter("2024-25");
    assert_eq!(amounts(&year_5), ["321.58", "319.83", "319.83", "1.75"]);
    assert_eq!(days(&year_5), ("365", "367"));
    assert_eq!(year_5["rates"]["period_of_the_loan_from"], "2020-07-03");
    assert_eq!(year_5["rates"]["period_of_the_loan_to"], "2025-07-02");
    assert_eq!(
        year_5["rates"]["period_of_the_loan"],
        "s 25-25(5)(c): 5 years from the first borrowing"
    );
    let year_6 = peter("2025-26");
    assert_eq!(amounts(&year_6), ["1.75", "1.75", "1.75", "0.00"]);
    assert_eq!(days(&year_6), ("2", "2"));
}

/// Fiona and Max (ATO, Borrowing expenses): $1,670 on a 20-year loan of
/// $209,000 from 17 July 2020, $39,000 of it for a private car, so 5 years to
/// 16 July 2025 and an income-producing share of 170,000 ÷ 209,000, supplied
/// to 40 significant digits. The ATO's table: $1,670.00 × 349 ÷ 1,826 =
/// $319.18; $1,350.82 × 365 ÷ 1,477 = $333.82; $1,017.00 × 365 ÷ 1,112 =
/// $333.82; $683.18 × 366 ÷ 747 = $334.73; then
/// - 2024-25: $348.45 × 365 ÷ 381 = $333.8169, so $333.82; × the share =
///   $271.5282, so $271.53 (the ATO's figure)
/// - 2025-26: $14.63 × 16 ÷ 16 = $14.63; × the share = $11.90 (the ATO's figure)
///
/// With no income-producing use the maximum is the same and nothing is
/// deductible (s 25-25(3), note).
#[test]
fn fiona_and_max_deduct_the_rental_share_of_each_maximum() {
    let share = "0.8133971291866028708133971291866028708134";
    let fiona = |year: &str, share: &str| {
        run(json!({"borrowing_expenses": "1670.00", "income_use": share,
                   "loan_start": "2020-07-17", "loan_term_end": "2040-07-16", "year": year}))
        .unwrap()
    };
    let year_5 = fiona("2024-25", share);
    assert_eq!(amounts(&year_5), ["348.45", "333.82", "271.53", "14.63"]);
    assert_eq!(days(&year_5), ("365", "381"));
    assert_eq!(year_5["rates"]["income_use"], share);
    let year_6 = fiona("2025-26", share);
    assert_eq!(amounts(&year_6), ["14.63", "14.63", "11.90", "0.00"]);
    assert_eq!(days(&year_6), ("16", "16"));
    let private = fiona("2024-25", "0");
    assert_eq!(amounts(&private), ["348.45", "333.82", "0.00", "14.63"]);
}

/// Ben (Library ¶4-160): $3,600 on a 4-year business loan from 1 September
/// 2025, so the contract's period, 1 September 2025 to 31 August 2029, is
/// 1,461 days. The Library rounds to dollars ($747, then $2,853 × 365 ÷ 1,158
/// = $899); to cents:
/// - 2025-26: $3,600 × 303 ÷ 1,461 = $746.6119, so $746.61
/// - 2026-27: $2,853.39 × 365 ÷ 1,158 = $899.3846, so $899.38
#[test]
fn ben_spreads_over_the_shorter_contract_period() {
    let ben = |year: &str| {
        run(
            json!({"borrowing_expenses": "3600.00", "loan_start": "2025-09-01",
                   "loan_term_end": "2029-08-31", "year": year}),
        )
        .unwrap()
    };
    let first = ben("2025-26");
    assert_eq!(amounts(&first), ["3600.00", "746.61", "746.61", "2853.39"]);
    assert_eq!(days(&first), ("303", "1461"));
    assert_eq!(
        first["rates"]["period_of_the_loan"],
        "s 25-25(5)(a): the original loan contract"
    );
    let second = ben("2026-27");
    assert_eq!(amounts(&second), ["2853.39", "899.38", "899.38", "1954.01"]);
    assert_eq!(days(&second), ("365", "1158"));
}

/// The s 25-25(4) example's shape: $1,500 on a 4-year loan from 1 September
/// 2024 (1,461 days), repaid on 31 December 2025. The period is determined at
/// the end of each income year, so 2024-25 ignores the later repayment:
/// $1,500 × 303 ÷ 1,461 = $311.0883, so $311.09. In 2025-26 the period ends on
/// repayment, 184 of 184 days are in the year, and the remaining $1,188.91 is
/// written off. (The Act's example counts 302 and 183 days, one fewer than
/// every day both ends included, as the ATO's examples and the Library count.)
#[test]
fn the_year_of_repayment_takes_what_remains() {
    let loan = |year: &str| {
        run(
            json!({"borrowing_expenses": "1500.00", "loan_start": "2024-09-01",
                   "loan_term_end": "2028-08-31", "repaid_on": "2025-12-31", "year": year}),
        )
        .unwrap()
    };
    let first = loan("2024-25");
    assert_eq!(amounts(&first), ["1500.00", "311.09", "311.09", "1188.91"]);
    assert_eq!(first["rates"]["period_of_the_loan_to"], "2028-08-31");
    let repaid = loan("2025-26");
    assert_eq!(amounts(&repaid), ["1188.91", "1188.91", "1188.91", "0.00"]);
    assert_eq!(days(&repaid), ("184", "184"));
    assert_eq!(
        repaid["rates"]["period_of_the_loan"],
        "s 25-25(5)(b): to the day the loan was repaid"
    );
    assert_eq!(
        run(
            json!({"borrowing_expenses": "1500.00", "loan_start": "2024-09-01",
                   "loan_term_end": "2028-08-31", "repaid_on": "2025-12-31", "year": "2026-27"})
        )
        .unwrap_err(),
        "The period of the loan ended on 2025-12-31, before the 2026-27 income year began: nothing remains to deduct."
    );
}

/// s 25-25(6): this loan's income-producing share plus the other borrowing
/// expenses that year, at $100 or less, is deducted in the year incurred.
/// $150 used 60% for income is $90, deducted in full; $10.01 of other
/// expenses makes $100.01, so it is spread: on a 3-year loan from 1 August
/// 2025 (1,096 days), 2025-26 has 334 of them, $150 × 334 ÷ 1,096 =
/// $45.7117, so $45.71, × 60% = $27.426, so $27.43.
#[test]
fn a_total_of_100_dollars_or_less_is_not_spread() {
    let loan = |amount: &str, share: &str, other: &str| {
        run(json!({"borrowing_expenses": amount, "income_use": share,
                   "other_borrowing_expenses_in_year": other, "loan_start": "2025-08-01",
                   "loan_term_end": "2028-07-31", "year": "2025-26"}))
        .unwrap()
    };
    let whole = loan("100.00", "1", "0.00");
    assert_eq!(amounts(&whole), ["100.00", "100.00", "100.00", "0.00"]);
    assert_eq!(
        whole["rates"]["spread"],
        "not spread: $100 or less (s 25-25(6))"
    );
    assert!(whole["rates"].get("remaining_loan_period_days").is_none());
    let share = loan("150.00", "0.6", "0.00");
    assert_eq!(amounts(&share), ["150.00", "150.00", "90.00", "0.00"]);
    let spread = loan("150.00", "0.6", "10.01");
    assert_eq!(amounts(&spread), ["150.00", "45.71", "27.43", "104.29"]);
    assert_eq!(
        spread["rates"]["spread"],
        "spread over the period of the loan (s 25-25(4))"
    );
    assert_eq!(
        refusal(
            json!({"borrowing_expenses": "100.00", "loan_start": "2024-08-01",
                       "year": "2025-26"})
        ),
        "s 25-25(6) applied in the 2024-25 income year the expenditure was incurred, so its deduction is not spread to 2025-26."
    );
}

#[test]
fn out_of_scope_facts_are_refused() {
    assert!(refusal(json!({"section_26_50_applies": true})).starts_with("Section 26-50 denies"));
    assert_eq!(
        refusal(json!({"loan_start": "2024-02-29", "year": "2024-25"})),
        "A loan first drawn on 29 February is out of scope: no source settles when its 5 years end."
    );
    assert_eq!(
        refusal(json!({"loan_term_end": "2024-07-02"})),
        "loan_term_end must be null or a YYYY-MM-DD date no earlier than loan_start."
    );
    assert_eq!(
        refusal(json!({"repaid_on": "2025-13-01"})),
        "repaid_on must be null or a YYYY-MM-DD date no earlier than loan_start."
    );
    assert_eq!(
        refusal(json!({"loan_start": "2025-07-01"})),
        "loan_start must not be after the 2024-25 income year ended."
    );
    assert_eq!(
        refusal(json!({"loan_start": "2019-07-01", "year": "2024-25"})),
        "The period of the loan ended on 2024-06-30, before the 2024-25 income year began: nothing remains to deduct."
    );
    assert_eq!(
        refusal(json!({"year": "2027-28"})),
        "Unsupported period '2027-28'; supported: 2024-25, 2025-26, 2026-27."
    );
    assert_eq!(
        refusal(json!({"income_use": "1.01"})),
        "income_use must be a finite Decimal fraction from 0 to 1."
    );
    assert_eq!(
        refusal(json!({"borrowing_expenses": "1.001"})),
        "Amounts must be finite non-negative Decimals, at most 2dp and 1e12."
    );
    assert_eq!(
        refusal(json!({"scope_confirmed": false})),
        "Establish all stated scope conditions before calculating."
    );
}

#[test]
fn the_catalogue_lists_the_worksheet_with_a_runnable_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["borrowing_expenses"];
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = api::evaluate("borrowing-expenses", &facts).unwrap();
    // Peter's first year from 3 July 2024: $1,600 × 363 ÷ 1,826 = $318.07.
    assert_eq!(r["amounts"]["deduction"], "318.07");
    assert_eq!(entry["supported_periods"][0]["id"], "2024-25");
    assert_eq!(
        entry["example_evidence"]["note"],
        "docs/calculation-evidence.md#borrowing-expenses"
    );
}
