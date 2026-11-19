// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The borrowing expenses schedule (ITAA 1997 s 25-25, several loans, every
//! year) against the ATO's Peter and Fiona and Max examples, the Library's
//! Ben (¶4-160), a refinancing, the $100 rule across loans, use by year, the
//! s 26-155 flag and its refusals
//! (docs/calculation-evidence.md#borrowing-expenses-schedule). Every row
//! also matches `borrowing-expenses` for the same loan and year.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A loan wholly used to produce income, changed by `overrides`.
fn loan(overrides: Value) -> Value {
    let mut loan = json!({
        "borrowing_expenses": "1600.00", "loan_start": "2020-07-03",
        "loan_term_end": "2045-07-02", "repaid_on": null, "income_use": "1",
        "section_26_50_applies": false,
    });
    for (k, v) in overrides.as_object().unwrap() {
        loan[k] = v.clone();
    }
    loan
}

fn input(loans: Value, year: &str) -> Value {
    json!({"loans": loans, "year": year, "scope_confirmed": true})
}

fn run(input: &Value) -> Result<Value, String> {
    let result = api::evaluate("borrowing-expenses-schedule", input);
    // derive gives the same result, or the same refusal.
    let envelope = json!({"command": "borrowing-expenses-schedule", "input": input});
    let doc = api::evaluate("derive", &envelope);
    assert_eq!(
        doc.as_ref().map(|doc| doc["result"].to_string()),
        result.as_ref().map(Value::to_string),
        "derive {input}"
    );
    result
}

fn schedule(loans: Value, year: &str) -> Value {
    run(&input(loans, year)).unwrap()
}

fn refusal(loans: Value) -> String {
    run(&input(loans, "2026-27")).unwrap_err()
}

/// A loan's rows as (income year, remaining, maximum, deduction, days in the
/// year, days left).
fn rows(result: &Value, n: usize) -> Vec<[String; 6]> {
    result["loans"][n - 1]["schedule"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            [
                "income_year",
                "remaining_expenditure",
                "maximum_deduction",
                "deduction",
                "days_in_year",
                "remaining_loan_period_days",
            ]
            .map(|key| match &row[key] {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            })
        })
        .collect()
}

fn row(
    year: &str,
    remaining: &str,
    maximum: &str,
    deduction: &str,
    days: i64,
    left: i64,
) -> [String; 6] {
    [year, remaining, maximum, deduction]
        .map(str::to_string)
        .into_iter()
        .chain([days.to_string(), left.to_string()])
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

const FIONA_SHARE: &str = "0.8133971291866028708133971291866028708134";

/// Peter and Fiona and Max (ATO, Borrowing expenses) in one run: each loan's
/// 6 years, as the ATO's examples work them, and the totals by year. The
/// years put each one's Year 4 on 2023-24, which holds 29 February 2024.
#[test]
fn two_loans_give_every_year_of_each_as_the_ato_works_them() {
    let fiona = loan(
        json!({"borrowing_expenses": "1670.00", "loan_start": "2020-07-17",
                            "loan_term_end": "2040-07-16", "income_use": FIONA_SHARE}),
    );
    let r = schedule(json!([loan(json!({})), fiona]), "2024-25");
    assert_eq!(
        rows(&r, 1),
        [
            row("2020-21", "1600.00", "318.07", "318.07", 363, 1826),
            row("2021-22", "1281.93", "319.83", "319.83", 365, 1463),
            row("2022-23", "962.10", "319.82", "319.82", 365, 1098),
            row("2023-24", "642.28", "320.70", "320.70", 366, 733),
            row("2024-25", "321.58", "319.83", "319.83", 365, 367),
            row("2025-26", "1.75", "1.75", "1.75", 2, 2),
        ]
    );
    assert_eq!(
        rows(&r, 2),
        [
            row("2020-21", "1670.00", "319.18", "259.62", 349, 1826),
            row("2021-22", "1350.82", "333.82", "271.53", 365, 1477),
            row("2022-23", "1017.00", "333.82", "271.53", 365, 1112),
            row("2023-24", "683.18", "334.73", "272.27", 366, 747),
            row("2024-25", "348.45", "333.82", "271.53", 365, 381),
            row("2025-26", "14.63", "14.63", "11.90", 16, 16),
        ]
    );
    let amounts = &r["amounts"];
    assert_eq!(amounts["borrowing_expenses"], "3270.00");
    assert_eq!(amounts["maximum_deduction"], "653.65");
    assert_eq!(amounts["deduction"], "591.36");
    assert_eq!(amounts["remaining_expenditure_after_year"], "16.38");
    assert_eq!(
        r["schedule"][5],
        json!({"income_year": "2025-26", "maximum_deduction": "16.38",
        "deduction": "13.65", "section_26_155": false, "projected": false})
    );
    assert_eq!(r["loans"][0]["total_deduction"], "1600.00");
    assert_eq!(r["loans"][1]["total_deduction"], "1358.38");
    assert_eq!(r["loans"][0]["period_of_the_loan_to"], "2025-07-02");
    assert_eq!(
        r["loans"][0]["period_of_the_loan"],
        "s 25-25(5)(c): 5 years from the first borrowing"
    );
}

/// Ben (Library ¶4-160): $3,600 over a 4-year contract from 1 September
/// 2025. The Library's days (303 of 1,461, 365 of 1,158, 366 of 793, 365 of
/// 427, 62 of 62) and its whole dollars ($747, $899, $902, $899, $153) to
/// the cent; 2027-28 on is flagged for s 26-155, and 2028-29 on is projected.
#[test]
fn ben_runs_to_the_end_of_his_4_year_contract() {
    let ben = loan(
        json!({"borrowing_expenses": "3600.00", "loan_start": "2025-09-01",
                          "loan_term_end": "2029-08-31"}),
    );
    let r = schedule(json!([ben]), "2027-28");
    assert_eq!(
        rows(&r, 1),
        [
            row("2025-26", "3600.00", "746.61", "746.61", 303, 1461),
            row("2026-27", "2853.39", "899.38", "899.38", 365, 1158),
            row("2027-28", "1954.01", "901.85", "901.85", 366, 793),
            row("2028-29", "1052.16", "899.39", "899.39", 365, 427),
            row("2029-30", "152.77", "152.77", "152.77", 62, 62),
        ]
    );
    let flags: Vec<(bool, bool)> = r["loans"][0]["schedule"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["section_26_155"].as_bool().unwrap(),
                row["projected"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        flags,
        [
            (false, false),
            (false, false),
            (true, false),
            (true, true),
            (true, true)
        ]
    );
    let warnings = r["warnings"].to_string();
    assert!(warnings.contains("s 26-155(1) quarantines"), "{warnings}");
    assert!(warnings.contains("marked projected"), "{warnings}");
    assert_eq!(r["amounts"]["deduction"], "901.85");
    assert_eq!(
        r["loans"][0]["period_of_the_loan"],
        "s 25-25(5)(a): the original loan contract"
    );
}

/// A $1,500 loan on a 4-year contract from 1 September 2024 refinanced on
/// 31 December 2025 by an $800 loan: the first loan's period ends on
/// repayment (s 25-25(5)(b)), so 2025-26 takes its remaining $1,188.91; the
/// second spreads over its own 5 years to 30 December 2030, 1,826 days:
/// $800 × 182 ÷ 1,826 = $79.7371, so $79.74.
#[test]
fn refinancing_writes_off_the_old_loan_and_spreads_the_new_one() {
    let old = loan(
        json!({"borrowing_expenses": "1500.00", "loan_start": "2024-09-01",
                          "loan_term_end": "2028-08-31", "repaid_on": "2025-12-31"}),
    );
    let new = loan(
        json!({"borrowing_expenses": "800.00", "loan_start": "2025-12-31",
                          "loan_term_end": null, "refinances": 1}),
    );
    let r = schedule(json!([old, new]), "2025-26");
    assert_eq!(
        rows(&r, 1),
        [
            row("2024-25", "1500.00", "311.09", "311.09", 303, 1461),
            row("2025-26", "1188.91", "1188.91", "1188.91", 184, 184),
        ]
    );
    assert_eq!(
        rows(&r, 2),
        [
            row("2025-26", "800.00", "79.74", "79.74", 182, 1826),
            row("2026-27", "720.26", "159.91", "159.91", 365, 1644),
            row("2027-28", "560.35", "160.35", "160.35", 366, 1279),
            row("2028-29", "400.00", "159.91", "159.91", 365, 913),
            row("2029-30", "240.09", "159.91", "159.91", 365, 548),
            row("2030-31", "80.18", "80.18", "80.18", 183, 183),
        ]
    );
    assert_eq!(r["amounts"]["deduction"], "1268.65");
    assert_eq!(r["loans"][0]["refinanced_by"], 2);
    assert_eq!(r["loans"][1]["refinances"], 1);
    assert_eq!(
        r["loans"][0]["period_of_the_loan"],
        "s 25-25(5)(b): to the day the loan was repaid"
    );
    assert!(
        r["loans"][0]["notes"][0]
            .as_str()
            .unwrap()
            .starts_with("Refinanced by loan 2: repaid on 2025-12-31")
    );
    // Refinancing must repay the named loan on the new loan's first day.
    let mut late = new.clone();
    late["loan_start"] = json!("2026-01-01");
    assert_eq!(
        refusal(json!([
            loan(
                json!({"borrowing_expenses": "1500.00", "loan_start": "2024-09-01",
            "loan_term_end": "2028-08-31", "repaid_on": "2025-12-31"})
            ),
            late
        ])),
        "loan 2: refinances must be the number of an earlier loan in loans, repaid on this loan's loan_start and refinanced by no other loan."
    );
}

/// s 25-25(2) and (3): each year's deduction is that year's maximum times
/// that year's use. Peter's loan used half for income from 2022-23.
#[test]
fn use_can_change_from_year_to_year() {
    let uses = json!({"2020-21": "1", "2021-22": "1", "2022-23": "0.5", "2023-24": "0.5",
                      "2024-25": "0", "2025-26": "0.5"});
    let r = schedule(json!([loan(json!({"income_use": uses}))]), "2023-24");
    let deductions: Vec<String> = rows(&r, 1).into_iter().map(|row| row[3].clone()).collect();
    assert_eq!(
        deductions,
        ["318.07", "319.83", "159.91", "160.35", "0.00", "0.88"]
    );
    assert_eq!(r["amounts"]["deduction"], "160.35");
    assert_eq!(r["loans"][0]["schedule"][2]["income_use"], "0.5");
    // Every year of the loan needs its use.
    let mut short = uses.clone();
    short.as_object_mut().unwrap().remove("2025-26");
    assert_eq!(
        refusal(json!([loan(json!({"income_use": short}))])),
        "loan 1: income_use must be a decimal fraction from 0 to 1, or an object giving one for each income year from 2020-21 to 2025-26."
    );
}

/// s 25-25(6) adds the income-producing shares of everything incurred for
/// borrowing in the year: $60 and $40 make $100, deducted that year; one
/// cent of other borrowing expenses makes it $100.01, so both are spread.
#[test]
fn the_100_dollar_test_counts_every_loan_incurred_in_the_year() {
    let loans = json!([
        loan(
            json!({"borrowing_expenses": "60.00", "loan_start": "2025-08-01", "loan_term_end": "2028-07-31"})
        ),
        loan(
            json!({"borrowing_expenses": "40.00", "loan_start": "2026-02-01", "loan_term_end": null})
        ),
    ]);
    let r = schedule(loans.clone(), "2025-26");
    let whole = ["2025-26", "60.00", "60.00", "60.00", "null", "null"].map(String::from);
    assert_eq!(rows(&r, 1), [whole]);
    assert_eq!(
        r["loans"][1]["spread"],
        "not spread: $100 or less (s 25-25(6))"
    );
    assert_eq!(r["amounts"]["deduction"], "100.00");
    let mut over = input(loans, "2025-26");
    over["other_borrowing_expenses"] = json!({"2025-26": "0.01"});
    let spread = run(&over).unwrap();
    assert_eq!(
        spread["loans"][0]["spread"],
        "spread over the period of the loan (s 25-25(4))"
    );
    // $60 × 334 ÷ 1,096 = $18.2847, so $18.28; $40 × 150 ÷ 1,826 = $3.2859, so $3.29.
    assert_eq!(spread["amounts"]["deduction"], "21.57");
}

/// Every row of a loan used wholly or partly for income is the one-loan
/// worksheet's result for that year, to the cent.
#[test]
fn each_row_is_the_one_loan_worksheets_result() {
    let loans = [
        json!({"borrowing_expenses": "1600.00", "loan_start": "2020-07-03", "loan_term_end": "2045-07-02", "repaid_on": null}),
        json!({"borrowing_expenses": "3600.00", "loan_start": "2025-09-01", "loan_term_end": "2029-08-31", "repaid_on": null}),
        json!({"borrowing_expenses": "1500.00", "loan_start": "2024-09-01", "loan_term_end": "2028-08-31", "repaid_on": "2025-12-31"}),
        json!({"borrowing_expenses": "2000.00", "loan_start": "2024-02-29", "loan_term_end": null, "repaid_on": null}),
        json!({"borrowing_expenses": "987654.32", "loan_start": "2017-07-01", "loan_term_end": null, "repaid_on": "2019-03-31"}),
    ];
    for facts in loans {
        let mut one = facts.clone();
        one["income_use"] = json!("0.75");
        one["section_26_50_applies"] = json!(false);
        let r = schedule(json!([one]), "2027-28");
        for row in r["loans"][0]["schedule"].as_array().unwrap() {
            let year = row["income_year"].as_str().unwrap();
            if year > "2027-28" {
                continue;
            }
            let mut single = facts.clone();
            single["other_borrowing_expenses_in_year"] = json!("0.00");
            single["income_use"] = json!("0.75");
            single["section_26_50_applies"] = json!(false);
            single["year"] = json!(year);
            single["scope_confirmed"] = json!(true);
            let w = api::evaluate("borrowing-expenses", &single).unwrap();
            for key in [
                "remaining_expenditure",
                "maximum_deduction",
                "deduction",
                "remaining_expenditure_after_year",
            ] {
                assert_eq!(row[key], w["amounts"][key], "{facts} {year} {key}");
            }
        }
    }
}

#[test]
fn out_of_scope_and_malformed_loans_are_refused() {
    let cases = [
        (
            json!([]),
            "loans must be a non-empty list of objects, one for each loan.",
        ),
        (
            json!(["loan"]),
            "loan 1: loans must be a non-empty list of objects, one for each loan.",
        ),
        (
            json!([loan(json!({"interest": "10.00"}))]),
            "loan 1: interest is not a fact a loan takes",
        ),
        (
            json!([loan(json!({})), {"borrowing_expenses": "100.00"}]),
            "loan 2: loan_start is required.",
        ),
        (
            json!([loan(json!({"loan_start": "2017-06-30"}))]),
            "loan 1: a loan first drawn before 1 July 2017 is out of scope: s 25-25 was read as it applies from the 2017-18 income year.",
        ),
        (
            json!([loan(json!({"loan_start": "2027-07-01"}))]),
            "loan 1: loan_start must not be after the 2026-27 income year ended.",
        ),
        (
            json!([loan(json!({"section_26_50_applies": true}))]),
            "loan 1: Section 26-50 denies deductions for a leisure facility, such as a holiday home, unless s 26-50(3) applies at all times in the income year; part-year use under s 26-50(4) is out of scope.",
        ),
        (
            json!([loan(json!({"section_26_50_applies": "no"}))]),
            "loan 1: section_26_50_applies must be a boolean.",
        ),
        (
            json!([loan(json!({"borrowing_expenses": 1600}))]),
            "loan 1: Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            json!([loan(json!({"income_use": "1.5"}))]),
            "loan 1: income_use must be a decimal fraction from 0 to 1, or an object giving one for each income year from 2020-21 to 2025-26.",
        ),
        (
            json!([loan(json!({"repaid_on": "2020-07-02"}))]),
            "loan 1: repaid_on must be null or a YYYY-MM-DD date no earlier than loan_start.",
        ),
        (
            json!([loan(json!({"refinances": 1}))]),
            "loan 1: refinances must be the number of an earlier loan in loans, repaid on this loan's loan_start and refinanced by no other loan.",
        ),
    ];
    for (loans, message) in cases {
        assert_eq!(refusal(loans), message);
    }
    let mut other = input(json!([loan(json!({}))]), "2026-27");
    other["other_borrowing_expenses"] = json!({"2025": "1.00"});
    assert!(
        run(&other)
            .unwrap_err()
            .starts_with("other_borrowing_expenses must be an object")
    );
    let codes = [
        (json!([]), "borrowing.loans_invalid"),
        (
            json!([loan(json!({"income_use": {}}))]),
            "borrowing.income_use_invalid",
        ),
        (
            json!([loan(json!({"loan_start": "2017-06-30"}))]),
            "borrowing.loan_before_2017_18",
        ),
        (
            json!([{"loan_start": "2020-07-03"}]),
            "borrowing.loan_fact_missing",
        ),
        (
            json!([loan(json!({"refinances": 0}))]),
            "borrowing.refinance_invalid",
        ),
    ];
    for (loans, code) in codes {
        let refused = api::evaluate_coded("borrowing-expenses-schedule", &input(loans, "2026-27"));
        assert_eq!(refused.unwrap_err().code, code);
    }
}

#[test]
fn the_catalogue_lists_the_schedule_with_a_runnable_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["borrowing_expenses_schedule"];
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = run(&facts).unwrap();
    // Peter's loan from 3 July 2024, refinanced on 2 March 2026: 2025-26
    // wrote it off, so 2026-27 has only the new loan, half used for income.
    // $900 × 121 ÷ 1,826 = $59.64 in 2025-26; $840.36 × 365 ÷ 1,705 = $179.90.
    assert_eq!(r["amounts"]["maximum_deduction"], "179.90");
    assert_eq!(r["amounts"]["deduction"], "89.95");
    assert_eq!(entry["supported_periods"][0]["id"], "2017-18");
    assert_eq!(
        entry["example_evidence"]["note"],
        "docs/calculation-evidence.md#borrowing-expenses-schedule"
    );
}
