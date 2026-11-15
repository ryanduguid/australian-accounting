// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `payg-gross-up` against every row of the ATO's Schedule 1 sample data
//! (docs/calculation-evidence.md#payg-gross-up): each row's net, its earnings
//! less its withholding, grosses up to the row's earnings, or is refused as a
//! tie that names them. Then the coefficient bounds the search relies on, its
//! refusals and its catalogue example.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::Decimal;
use australian_tax_calculators::metadata::{PAYG_WITHHOLDING_COEFFICIENTS, PAYG_WITHHOLDING_RULES};
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
        ["foreign_resident"]
    );
    let periods = api::evaluate("periods", &json!({"kind": "payg_gross_up"})).unwrap();
    assert_eq!(periods.as_array().unwrap().len(), 3);
}
