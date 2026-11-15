// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! A study and training support loan on a Schedule 9 pay. Schedule 9 for
//! payments from 1 July 2026 and from 1 July 2024 to 30 June 2026 says to
//! 'refer to Study and training support loans weekly tax table', which adds its
//! column 2 component (tax-free threshold claimed, as every Schedule 9 payee
//! has at question 5 of the Withholding declaration) to the withholding amount
//! and points software to Schedule 8's formulas. All were read on the ATO's
//! site on 15 November 2026, with Schedule 8 of F2026L00716 on
//! legislation.gov.au.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "seniors-pensioners-withholding";
const STATUSES: [&str; 3] = ["single", "illness_separated", "couple"];

fn facts(earnings: &str, period: &str, status: &str, year: &str, loan: Value) -> Value {
    let mut facts = json!({
        "earnings": earnings, "pay_period": period, "status": status,
        "medicare_levy_exemption": "none", "year": year, "scope_confirmed": true,
    });
    for (key, value) in loan.as_object().unwrap() {
        facts[key] = value.clone();
    }
    facts
}

fn run(facts: &Value) -> Value {
    api::evaluate(COMMAND, facts).unwrap_or_else(|e| panic!("{facts}: {e}"))
}

fn dollars(result: &Value, key: &str) -> i64 {
    let text = result["amounts"][key]
        .as_str()
        .unwrap_or_else(|| panic!("no {key}: {result}"));
    text.strip_suffix(".00").unwrap().parse().unwrap()
}

/// The component, and the total against the same pay without a study loan.
fn component(earnings: &str, period: &str, status: &str, year: &str, loan: Value) -> i64 {
    let with = run(&facts(earnings, period, status, year, loan));
    let without = run(&facts(earnings, period, status, year, json!({})));
    let schedule_9 = dollars(&without, "withholding");
    assert_eq!(dollars(&with, "schedule_9_withholding"), schedule_9);
    let component = dollars(&with, "study_loan_component");
    assert_eq!(dollars(&with, "withholding"), schedule_9 + component);
    assert_eq!(
        with["amounts"]["weekly_earnings_used"],
        without["amounts"]["weekly_earnings_used"]
    );
    component
}

fn loan_2026() -> Value {
    json!({"study_loan": true})
}

/// 2025-26's coefficients apply to payments made from 24 September 2025.
fn loan_2025() -> Value {
    json!({"study_loan": true, "pay_date": "2025-09-24"})
}

#[test]
fn every_row_of_the_study_loan_weekly_table_from_july_2026() {
    // NAT 2173, published 17 June 2026: weekly earnings $985 to $3,600,
    // column 2, the same for each Schedule 9 status.
    let table = include_str!("data/study_loan_weekly_2026_27.csv");
    let mut lines = table.lines();
    assert_eq!(lines.next(), Some("weekly_earnings,tax_free_threshold"));
    let rows: Vec<(&str, &str)> = lines.map(|l| l.split_once(',').unwrap()).collect();
    assert_eq!(rows.len(), 2616);
    for (earnings, expected) in rows {
        let expected: i64 = expected.strip_suffix(".00").unwrap().parse().unwrap();
        for status in STATUSES {
            let got = component(earnings, "weekly", status, "2026-27", loan_2026());
            assert_eq!(got, expected, "{earnings} {status}");
        }
    }
}

/// 'A payee has claimed the tax-free threshold and has weekly earnings of
/// $1,816.23': $72 in the table from 1 July 2026, $79 in the one for payments
/// from 24 September 2025 to 30 June 2026 (last updated 9 September 2025).
#[test]
fn the_weekly_tables_examples() {
    for (year, loan, expected) in [("2026-27", loan_2026(), 72), ("2025-26", loan_2025(), 79)] {
        let result = run(&facts("1816.23", "weekly", "single", year, loan.clone()));
        assert_eq!(result["amounts"]["weekly_earnings_used"], "1816.99");
        assert_eq!(dollars(&result, "study_loan_component"), expected, "{year}");
        assert_eq!(
            component("1816.23", "weekly", "single", year, loan),
            expected
        );
    }
}

/// Schedule 8's examples 1 and 2, which claim the tax-free threshold: $2,608.36
/// a week and $4,409.75 a fortnight give $193 and $260 from 1 July 2026, and
/// $202 and $274 under F2025L01016 (as `tests/study_loan_withholding.rs`).
/// Example 3 is for a payee without the threshold, which Schedule 9 excludes;
/// its $10,627.88 a month with the threshold, worked by hand: 2026-27,
/// 10,627.88 x 3 / 13 = 2,452.59, x = 2,452.99, 0.15 x 2,452.99 - 200.5615 =
/// 167.3870, $167 a week, 167 x 13 / 3 = 723.67, $724; 2025-26, 0.17 x
/// 2,452.99 - 241.3462 = 175.6621, $176 a week, 176 x 13 / 3 = 762.67, $763.
#[test]
fn schedule_8_examples_through_schedule_9() {
    for (earnings, period, x, in_2026, in_2025) in [
        ("2608.36", "weekly", "2608.99", 193, 202),
        ("4409.75", "fortnightly", "2204.99", 260, 274),
        ("10627.88", "monthly", "2452.99", 724, 763),
    ] {
        for status in STATUSES {
            for (year, loan, expected) in [
                ("2026-27", loan_2026(), in_2026),
                ("2025-26", loan_2025(), in_2025),
            ] {
                let result = run(&facts(earnings, period, status, year, loan.clone()));
                assert_eq!(result["amounts"]["weekly_earnings_used"], x);
                let got = component(earnings, period, status, year, loan);
                assert_eq!(got, expected, "{year} {earnings} {status}");
            }
        }
    }
}

/// Each step of Schedule 8's tax-free threshold rates, on either side, by hand.
/// The earnings Schedule 8 says the component starts at ($1,337 a week, $2,674
/// a fortnight and $5,793.67 a month from 1 July 2026) enter its first rate.
#[test]
fn each_threshold_selects_its_coefficients() {
    for (year, loan, earnings, period, a, b, expected) in [
        ("2026-27", loan_2026(), "1336.99", "weekly", "0", "0", 0),
        (
            "2026-27",
            loan_2026(),
            "1337.00",
            "weekly",
            "0.15",
            "200.5615",
            0,
        ),
        (
            "2026-27",
            loan_2026(),
            "2493.00",
            "weekly",
            "0.15",
            "200.5615",
            174,
        ),
        (
            "2026-27",
            loan_2026(),
            "2494.00",
            "weekly",
            "0.17",
            "250.4527",
            174,
        ),
        (
            "2026-27",
            loan_2026(),
            "3576.00",
            "weekly",
            "0.17",
            "250.4527",
            358,
        ),
        (
            "2026-27",
            loan_2026(),
            "3577.00",
            "weekly",
            "0.10",
            "0",
            358,
        ),
        (
            "2026-27",
            loan_2026(),
            "2673.99",
            "fortnightly",
            "0",
            "0",
            0,
        ),
        (
            "2026-27",
            loan_2026(),
            "2674.00",
            "fortnightly",
            "0.15",
            "200.5615",
            0,
        ),
        ("2026-27", loan_2026(), "5793.66", "monthly", "0", "0", 0),
        (
            "2026-27",
            loan_2026(),
            "5793.67",
            "monthly",
            "0.15",
            "200.5615",
            0,
        ),
        ("2025-26", loan_2025(), "1287.99", "weekly", "0", "0", 0),
        (
            "2025-26",
            loan_2025(),
            "1288.00",
            "weekly",
            "0.15",
            "193.2692",
            0,
        ),
        (
            "2025-26",
            loan_2025(),
            "2402.00",
            "weekly",
            "0.15",
            "193.2692",
            167,
        ),
        (
            "2025-26",
            loan_2025(),
            "2403.00",
            "weekly",
            "0.17",
            "241.3462",
            167,
        ),
        (
            "2025-26",
            loan_2025(),
            "3446.00",
            "weekly",
            "0.17",
            "241.3462",
            345,
        ),
        (
            "2025-26",
            loan_2025(),
            "3447.00",
            "weekly",
            "0.10",
            "0",
            345,
        ),
    ] {
        let result = run(&facts(earnings, period, "couple", year, loan.clone()));
        let rates = &result["rates"];
        assert_eq!(
            (&rates["component_a"], &rates["component_b"]),
            (&json!(a), &json!(b)),
            "{year} {earnings} {period}"
        );
        assert_eq!(
            component(earnings, period, "couple", year, loan),
            expected,
            "{year} {earnings} {period}"
        );
    }
}

/// A full or half Medicare levy exemption changes the Schedule 9 scale, not the
/// component: Schedule 8 withholds none only for a Medicare levy variation
/// declaration claiming a reduction or exemption 'for having a spouse or
/// dependants and low family income', which is not an exemption category.
#[test]
fn medicare_levy_exemptions_keep_the_component() {
    for exemption in ["none", "full", "half"] {
        let mut with = facts("2608.36", "weekly", "single", "2026-27", loan_2026());
        with["medicare_levy_exemption"] = json!(exemption);
        let mut without = with.clone();
        without.as_object_mut().unwrap().remove("study_loan");
        let (with, without) = (run(&with), run(&without));
        assert_eq!(dollars(&with, "study_loan_component"), 193, "{exemption}");
        assert_eq!(
            dollars(&with, "withholding"),
            dollars(&without, "withholding") + 193
        );
        assert_eq!(with["rates"]["scale"], without["rates"]["scale"]);
    }
}

/// Without a study loan, or with `study_loan` false, the result is the one the
/// worksheet gave before it took a study loan: every weekly earnings in each
/// schedule's sample table, each status and exemption, each pay period.
#[test]
fn without_a_study_loan_the_result_is_unchanged() {
    for (table, years) in [
        (
            include_str!("data/seniors_pensioners_weekly_2026_27.csv"),
            &["2026-27"][..],
        ),
        (
            include_str!("data/seniors_pensioners_weekly_2024_26.csv"),
            &["2024-25", "2025-26"][..],
        ),
    ] {
        for line in table.lines().skip(1) {
            let earnings = line.split(',').next().unwrap();
            for year in years {
                for period in ["weekly", "fortnightly", "monthly"] {
                    for exemption in ["none", "full", "half"] {
                        let mut absent = facts(earnings, period, "single", year, json!({}));
                        absent["medicare_levy_exemption"] = json!(exemption);
                        let result = run(&absent);
                        let keys: Vec<&String> =
                            result["amounts"].as_object().unwrap().keys().collect();
                        assert_eq!(
                            keys,
                            ["weekly_earnings_used", "weekly_withholding", "withholding"]
                        );
                        let keys: Vec<&String> =
                            result["rates"].as_object().unwrap().keys().collect();
                        assert_eq!(keys, ["scale", "a", "b"]);
                        let mut off = absent.clone();
                        off["study_loan"] = json!(false);
                        assert_eq!(run(&off), result, "{year} {period} {earnings}");
                    }
                }
            }
        }
    }
}

fn refusal(facts: &Value) -> (&'static str, String) {
    let refusal = api::evaluate_coded(COMMAND, facts).unwrap_err();
    (refusal.code, refusal.message)
}

#[test]
fn what_schedule_8_does_not_cover_here_is_refused() {
    // Before 24 September 2025 Schedule 8 used a percentage of weekly
    // earnings, which this crate does not hold.
    let (code, message) = refusal(&facts(
        "2000.00",
        "weekly",
        "single",
        "2024-25",
        loan_2026(),
    ));
    assert_eq!(code, "period.unsupported");
    assert_eq!(
        message,
        "study_loan is supported for 2025-26, 2026-27 only: there is no Schedule 8 study and training support loan component for 2024-25."
    );
    let (code, message) = refusal(&facts(
        "2000.00",
        "weekly",
        "single",
        "2025-26",
        loan_2026(),
    ));
    assert_eq!(code, "input.missing_field");
    assert_eq!(
        message,
        "pay_date is required for 2025-26: its Schedule 8 coefficients apply to payments made on or after 2025-09-24."
    );
    let early = json!({"study_loan": true, "pay_date": "2025-09-23"});
    let (code, _) = refusal(&facts("2000.00", "weekly", "single", "2025-26", early));
    assert_eq!(code, "payg.pay_date_before_schedule");
    let outside = json!({"study_loan": true, "pay_date": "2026-07-01"});
    let (code, _) = refusal(&facts("2000.00", "weekly", "single", "2025-26", outside));
    assert_eq!(code, "payg.pay_date_outside_year");
    // Schedule 9 has no quarterly rule, with or without a study loan.
    let (code, _) = refusal(&facts(
        "2000.00",
        "quarterly",
        "single",
        "2026-27",
        loan_2026(),
    ));
    assert_eq!(code, "input.pay_period_invalid");
}

#[test]
fn malformed_study_loan_input_is_refused() {
    for loan in [
        json!({"pay_date": "2026-07-01"}),
        json!({"study_loan": false, "pay_date": "2026-07-01"}),
    ] {
        assert_eq!(
            refusal(&facts("2000.00", "weekly", "single", "2026-27", loan)),
            (
                "payg.pay_date_without_study_loan",
                "pay_date selects the Schedule 8 study and training support loan component; give it only with study_loan true.".into()
            )
        );
    }
    assert_eq!(
        refusal(&facts(
            "2000.00",
            "weekly",
            "single",
            "2026-27",
            json!({"study_loan": "yes"})
        )),
        ("input.flag_invalid", "study_loan must be a boolean.".into())
    );
}

/// The catalogue lists both optional inputs.
#[test]
fn the_catalogue_lists_the_optional_inputs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let optional = &catalogue["seniors_pensioners_withholding"]["optional_inputs"];
    let keys: Vec<&String> = optional.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["study_loan", "pay_date"]);
}
