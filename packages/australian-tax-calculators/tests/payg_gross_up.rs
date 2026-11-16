// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `payg-gross-up` against every row of the ATO's Schedule 1 sample data
//! (docs/calculation-evidence.md#payg-gross-up): each row's net, its earnings
//! less its withholding, grosses up to the row's earnings, or is refused as a
//! tie that names them. Then the coefficient bounds the search relies on, its
//! refusals and its catalogue example. With a study loan, the net of each of
//! Schedule 8's examples 1 to 3 grosses up to its earnings, worked by hand
//! from the ATO's Schedule 1 and Schedule 8 for payments from 1 July 2026,
//! read on 16 November 2026, and F2025L01016's Schedule 8 for 2025-26.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::Decimal;
use australian_tax_calculators::metadata::{
    PAYG_WITHHOLDING_COEFFICIENTS, PAYG_WITHHOLDING_RULES, STUDY_LOAN_WITHHOLDING_COEFFICIENTS,
};
use serde_json::{Value, json};

const COMMAND: &str = "payg-gross-up";

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn input(net: &str, period: &str, scale: i64, year: &str) -> Value {
    json!({"net_pay": net, "pay_period": period, "scale": scale, "year": year,
           "scope_confirmed": true})
}

/// Every row and scale of one sample file for `year`: (grossed up, tied).
fn grosses_up_the_ato_sample(sample: &str, year: &str) -> (usize, usize) {
    let mut lines = sample.lines();
    assert_eq!(
        lines.next(),
        Some("period,earnings,scale1,scale2,scale3,scale5,scale6")
    );
    let (mut unique, mut tied) = (0, 0);
    for line in lines {
        let row: Vec<&str> = line.split(',').collect();
        let gross = d(row[1]).fixed(2, false);
        for (column, scale) in [1, 2, 3, 5, 6].into_iter().enumerate() {
            let net = d(row[1]).sub(&d(row[2 + column])).unwrap().fixed(2, false);
            let case = format!("{year} {} {gross} scale {scale}: net {net}", row[0]);
            match api::evaluate_coded(COMMAND, &input(&net, row[0], scale, year)) {
                Ok(result) => {
                    assert_eq!(result["amounts"]["gross_pay"], gross, "{case}");
                    assert_eq!(result["amounts"]["net_pay"], net, "{case}");
                    unique += 1;
                }
                Err(refusal) => {
                    assert_eq!(refusal.code, "payg.gross_up_not_unique", "{case}");
                    assert!(
                        refusal.message.contains(&format!("${gross}")),
                        "{case}: {}",
                        refusal.message
                    );
                    tied += 1;
                }
            }
        }
    }
    (unique, tied)
}

#[test]
fn every_sample_row_grosses_up_to_its_earnings_or_a_tie_naming_them() {
    let current = include_str!("data/payg_withholding_sample_2026_27.csv");
    let earlier = include_str!("data/payg_withholding_sample_2024_26.csv");
    for (sample, year) in [
        (current, "2026-27"),
        (earlier, "2024-25"),
        (earlier, "2025-26"),
    ] {
        let (unique, tied) = grosses_up_the_ato_sample(sample, year);
        assert_eq!(unique + tied, 144 * 5, "{year}");
        assert!(
            unique > 0 && tied > 0,
            "{year}: {unique} unique, {tied} tied"
        );
    }
}

/// The search's bound: with every `a` at most 0.47 and every `b` at least -1,
/// and the scale 4 rates at most 47%, no pay withholds more than half its
/// gross plus $26 (src/engine/payg_gross_up.rs).
#[test]
fn every_coefficient_keeps_withholding_within_the_search_bound() {
    let most = d("0.47");
    for (year, scales) in PAYG_WITHHOLDING_COEFFICIENTS {
        for (scale, rows) in *scales {
            for (limit, a, b) in *rows {
                let row = format!("{year} scale {scale} below {limit:?}");
                assert!(d(a).le(&most).unwrap(), "{row}: a {a}");
                assert!(!d(b).lt(&d("-1")).unwrap(), "{row}: b {b}");
            }
        }
    }
    for (year, rules) in PAYG_WITHHOLDING_RULES {
        let (resident, foreign) = rules.no_tfn_rates;
        for rate in [resident, foreign] {
            assert!(d(rate).le(&most).unwrap(), "{year} scale 4 {rate}");
        }
    }
}

/// The study loan search's bound: with every Schedule 8 component `a` at most
/// 0.17 and every `b` at least 0, no pay withholds more than 64% of its gross
/// plus $35 (src/engine/payg_gross_up.rs).
#[test]
fn every_study_loan_coefficient_keeps_withholding_within_the_search_bound() {
    for (year, tables) in STUDY_LOAN_WITHHOLDING_COEFFICIENTS {
        for (table, rows) in *tables {
            for (limit, a, b) in *rows {
                let row = format!("{year} {table} below {limit:?}");
                assert!(d(a).le(&d("0.17")).unwrap(), "{row}: a {a}");
                assert!(!d(b).lt(&d("0")).unwrap(), "{row}: b {b}");
            }
        }
    }
}

fn with_loan(net: &str, period: &str, scale: i64, year: &str) -> Value {
    let mut facts = input(net, period, scale, year);
    facts["study_loan"] = json!(true);
    facts
}

/// Schedule 8's examples 1 to 3 for payments from 1 July 2026, each pay's net
/// after its Schedule 1 amount and component. The ATO's combined coefficients
/// with the component give the same totals: 0.56 x 2,608.99 - 613.9154 is
/// $847; 0.47 x 2,204.99 - 382.2935 is $654, so $1,308; and on scale 1,
/// 0.56 x 2,452.99 - 419.8343 is $954, so $4,134 a month.
#[test]
fn the_schedule_8_examples_gross_up_with_the_study_loan_component() {
    // Example 1, weekly $2,608.36 on scale 2: x = 2,608.99; Schedule 1
    // 0.39 x - 363.4627 = 654.0434, so $654; component 0.17 x - 250.4527 =
    // 193.0756, so $193; $847 withheld leaves $1,761.36.
    // Example 2, fortnightly $4,409.75 on scale 2: x = 2,204.99; Schedule 1
    // 0.32 x - 181.7319 = 523.8649, so $524 and $1,048 a fortnight; component
    // $130, so $260; $1,308 withheld leaves $3,101.75.
    for (net, period, gross, tax, component) in [
        ("1761.36", "weekly", "2608.36", "654.00", "193.00"),
        ("3101.75", "fortnightly", "4409.75", "1048.00", "260.00"),
    ] {
        let result = api::evaluate(COMMAND, &with_loan(net, period, 2, "2026-27")).unwrap();
        let amounts = &result["amounts"];
        assert_eq!(amounts["net_pay"], net, "{period}");
        assert_eq!(amounts["gross_pay"], gross, "{period}");
        assert_eq!(amounts["schedule_1_withholding"], tax, "{period}");
        assert_eq!(amounts["study_loan_component"], component, "{period}");
    }
    // Example 3, monthly $10,627.88 on scale 1: x = 2,452.99; Schedule 1
    // 0.39 x - 228.8816 = 727.7845, so $728 and $3,154.67, or $3,155, a month;
    // component $226, so $979; $4,134 withheld leaves $6,493.88. $10,622.88
    // leaves it too: x = 2,451.99 gives $727 and $226, so $3,150 + $979 = $4,129.
    let refusal =
        api::evaluate_coded(COMMAND, &with_loan("6493.88", "monthly", 1, "2026-27")).unwrap_err();
    assert_eq!(refusal.code, "payg.gross_up_not_unique");
    assert_eq!(
        refusal.message,
        "2 gross pays leave a net of $6493.88 for a 2026-27 monthly pay on scale 1: $10622.88 and $10627.88. Schedule 1 withholds whole dollars, so a gross that withholds a dollar more for each dollar more of pay leaves the same net. Take the gross from the pay agreement."
    );
}

/// Schedule 1's 'Working out the weekly earnings' and Schedule 8's
/// 'Calculating fortnightly, monthly or quarterly withholding amounts', for
/// payments from 1 July 2026, read on 16 November 2026: a monthly amount
/// ending in 33 cents gains a cent before it is multiplied by 3 and divided
/// by 13.
#[test]
fn a_monthly_study_loan_pay_ending_in_33_cents_gains_a_cent() {
    // $5,997.33 on scale 2: (5,997.33 + 0.01) x 3 / 13 = 1,384.0015, so
    // x = 1,384.99 (1,383.99 without the cent). Schedule 1 0.32 x - 181.7319
    // = 261.4649, so $261 and $1,131 a month; component 0.15 x - 200.5615 =
    // 7.1870, so $7 and $30; $1,161 withheld leaves $4,836.33.
    let result = api::evaluate(COMMAND, &with_loan("4836.33", "monthly", 2, "2026-27")).unwrap();
    let amounts = &result["amounts"];
    assert_eq!(amounts["gross_pay"], "5997.33");
    assert_eq!(amounts["weekly_earnings_used"], "1384.99");
    assert_eq!(amounts["schedule_1_withholding"], "1131.00");
    assert_eq!(amounts["study_loan_component"], "30.00");
    // $5,893.33 gives x = 1,360.99 (1,359.99 without the cent): Schedule 1
    // 253.7849, so $254 and $1,101; component 3.5870, so $4 and $17; $1,118
    // withheld leaves $4,775.33. $5,884.33 gives x = 1,357.99: $253 and
    // $1,096, component $3 and $13, so $1,109 withheld leaves $4,775.33 too.
    // Without the cent, $5,893.33 would withhold $1,109 and leave $4,784.33.
    let refusal =
        api::evaluate_coded(COMMAND, &with_loan("4775.33", "monthly", 2, "2026-27")).unwrap_err();
    assert_eq!(refusal.code, "payg.gross_up_not_unique");
    assert!(
        refusal.message.starts_with(
            "2 gross pays leave a net of $4775.33 for a 2026-27 monthly pay on scale 2: $5884.33 and $5893.33."
        ),
        "{}",
        refusal.message
    );
}

#[test]
fn a_2025_26_study_loan_takes_the_pay_date_and_its_coefficients() {
    // F2025L01016's example 1, weekly $2,608.36 on scale 2 from 24 September
    // 2025: x = 2,608.99; Schedule 1 0.39 x - 358.3077 = 659.1984, so $659;
    // component 0.17 x - 241.3462 = 202.1821, so $202; $861 withheld leaves
    // $1,747.36. $2,609.36 withholds $660 + $202 = $862 and $2,610.36
    // withholds $660 + $203 = $863, leaving the same net.
    let mut facts = with_loan("1747.36", "weekly", 2, "2025-26");
    let refusal = api::evaluate_coded(COMMAND, &facts).unwrap_err();
    assert_eq!(refusal.code, "input.missing_field");
    facts["pay_date"] = json!("2025-09-24");
    let refusal = api::evaluate_coded(COMMAND, &facts).unwrap_err();
    assert_eq!(refusal.code, "payg.gross_up_not_unique");
    assert!(
        refusal.message.starts_with(
            "3 gross pays leave a net of $1747.36 for a 2025-26 weekly pay on scale 2: $2608.36, $2609.36 and $2610.36."
        ),
        "{}",
        refusal.message
    );
    facts["pay_date"] = json!("2025-09-23");
    let refusal = api::evaluate_coded(COMMAND, &facts).unwrap_err();
    assert_eq!(refusal.code, "payg.pay_date_before_schedule");
}

#[test]
fn a_study_loan_outside_schedule_8_is_refused() {
    let code = |facts: &Value| api::evaluate_coded(COMMAND, facts).unwrap_err().code;
    assert_eq!(
        code(&with_loan("1761.36", "weekly", 2, "2024-25")),
        "period.unsupported"
    );
    assert_eq!(
        code(&with_loan("501.00", "weekly", 4, "2026-27")),
        "input.tax_scale_unsupported"
    );
    let mut foreign = with_loan("1761.36", "weekly", 4, "2026-27");
    foreign["foreign_resident"] = json!(true);
    assert_eq!(code(&foreign), "payg.foreign_resident_not_scale_4");
    let mut dated = input("862.00", "weekly", 2, "2026-27");
    dated["pay_date"] = json!("2026-07-01");
    assert_eq!(code(&dated), "payg.pay_date_without_study_loan");
    let mut text = input("862.00", "weekly", 2, "2026-27");
    text["study_loan"] = json!("yes");
    assert_eq!(code(&text), "input.flag_invalid");
    assert_eq!(
        code(&with_loan("333333333300.01", "weekly", 2, "2026-27")),
        "payg.gross_up_net_too_large"
    );
    // study_loan false is the plain search.
    let mut plain = input("862.00", "weekly", 2, "2026-27");
    plain["study_loan"] = json!(false);
    assert_eq!(
        api::evaluate(COMMAND, &plain).unwrap(),
        api::evaluate(COMMAND, &input("862.00", "weekly", 2, "2026-27")).unwrap()
    );
}

#[test]
fn the_largest_net_searched_with_a_study_loan_stays_within_the_amount_limit() {
    let net = "333333333300.00";
    match api::evaluate_coded(COMMAND, &with_loan(net, "quarterly", 2, "2026-27")) {
        Ok(result) => {
            let gross = d(result["amounts"]["gross_pay"].as_str().unwrap());
            assert!(gross.le(&d("1000000000000")).unwrap());
        }
        Err(refusal) => assert_eq!(
            refusal.code, "payg.gross_up_not_unique",
            "{}",
            refusal.message
        ),
    }
}

#[test]
fn a_net_with_one_gross_shows_the_withholding_that_leaves_it() {
    // Weekly scale 2 for 2026-27: $1,000.00 gives x = 1,000.99 and
    // y = 0.3227 x - 185.1935 = 137.8260, so $138 is withheld, leaving $862.00;
    // $999 and $1,001 also withhold $138, leaving $861 and $863.
    let result = api::evaluate(COMMAND, &input("862.00", "weekly", 2, "2026-27")).unwrap();
    let amounts = &result["amounts"];
    assert_eq!(amounts["net_pay"], "862.00");
    assert_eq!(amounts["gross_pay"], "1000.00");
    assert_eq!(amounts["weekly_earnings_used"], "1000.99");
    assert_eq!(amounts["withholding"], "138.00");
    assert_eq!(result["rates"]["a"], "0.3227");
    assert_eq!(result["period"], "2026-27 weekly");
}

#[test]
fn a_net_two_grosses_leave_is_refused_naming_both() {
    // Weekly scale 2 for 2026-27: $364 withholds nothing (x = 364.99 gives
    // y = 0.15 x - 54.3462 = 0.4023, rounded to $0) and $365 withholds $1
    // (y = 0.5523), so a net of $364.00 has 2 grosses.
    let refusal =
        api::evaluate_coded(COMMAND, &input("364.00", "weekly", 2, "2026-27")).unwrap_err();
    assert_eq!(refusal.code, "payg.gross_up_not_unique");
    assert_eq!(
        refusal.message,
        "2 gross pays leave a net of $364.00 for a 2026-27 weekly pay on scale 2: $364.00 and $365.00. Schedule 1 withholds whole dollars, so a gross that withholds a dollar more for each dollar more of pay leaves the same net. Take the gross from the pay agreement."
    );
}

#[test]
fn scale_4_takes_residency_as_payg_withholding_does() {
    let mut facts = input("501.00", "weekly", 4, "2026-27");
    let refusal = api::evaluate_coded(COMMAND, &facts).unwrap_err();
    assert_eq!(
        (refusal.code, refusal.message.as_str()),
        (
            "input.missing_field",
            "foreign_resident is required for scale 4."
        )
    );
    // 45% of $910 is $409.50, cents ignored, so $409 is withheld; $909 and
    // $911 also withhold $409.
    facts["foreign_resident"] = json!(true);
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(result["amounts"]["gross_pay"], "910.00");
    assert_eq!(result["amounts"]["withholding"], "409.00");
}

#[test]
fn inputs_outside_the_search_are_refused() {
    let code = |facts: &Value| api::evaluate_coded(COMMAND, facts).unwrap_err().code;
    let mut offsets = input("862.00", "weekly", 2, "2026-27");
    offsets["tax_offsets"] = json!("1000.00");
    assert_eq!(code(&offsets), "input.field_unknown");
    assert_eq!(
        code(&input("499999999974.01", "weekly", 2, "2026-27")),
        "payg.gross_up_net_too_large"
    );
    let mut unconfirmed = input("862.00", "weekly", 2, "2026-27");
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(code(&unconfirmed), "scope.not_confirmed");
    assert_eq!(
        code(&input("862.00", "weekly", 2, "2027-28")),
        "period.unsupported"
    );
    assert_eq!(
        code(&input("862.00", "daily", 2, "2026-27")),
        "input.pay_period_invalid"
    );
}

#[test]
fn the_largest_net_searched_still_grosses_up_within_the_amount_limit() {
    let net = "499999999974.00";
    let result = api::evaluate_coded(COMMAND, &input(net, "quarterly", 2, "2026-27"));
    let gross = match result {
        Ok(result) => d(result["amounts"]["gross_pay"].as_str().unwrap()),
        Err(refusal) => {
            assert_eq!(
                refusal.code, "payg.gross_up_not_unique",
                "{}",
                refusal.message
            );
            return;
        }
    };
    assert!(gross.le(&d("1000000000000")).unwrap());
}

#[test]
fn the_catalogue_example_grosses_up_the_payg_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["payg_gross_up"];
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(result["amounts"]["gross_pay"], "1000.00");
    assert_eq!(
        entry["optional_inputs"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["foreign_resident", "study_loan", "pay_date"]
    );
    let periods = api::evaluate("periods", &json!({"kind": "payg_gross_up"})).unwrap();
    assert_eq!(periods.as_array().unwrap().len(), 3);
}
