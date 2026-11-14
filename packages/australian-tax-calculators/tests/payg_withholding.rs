// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Schedule 1 beyond the base formula, through the public JSON boundary: the
//! Medicare levy adjustment against every row of the ATO's Medicare sample data,
//! quarterly pays, scale 4, tax offsets and the additional amounts for 53 or 27
//! pays, with the schedules' worked examples. Figures are from Schedule 1 for
//! payments from 1 July 2026 (LI 2026/18) and for payments from 1 July 2024 to
//! 30 June 2026, both read at ato.gov.au on 14 November 2026.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(earnings: &str, period: &str, scale: i64, year: &str, extra: Value) -> Value {
    let mut input = json!({"earnings": earnings, "pay_period": period, "scale": scale,
                           "year": year, "scope_confirmed": true});
    for (key, value) in extra.as_object().unwrap() {
        input[key] = value.clone();
    }
    input
}

fn run(earnings: &str, period: &str, scale: i64, year: &str, extra: Value) -> Value {
    let input = input(earnings, period, scale, year, extra);
    api::evaluate("payg-withholding", &input).unwrap_or_else(|e| panic!("{input}: {e}"))
}

fn refused(earnings: &str, period: &str, scale: i64, year: &str, extra: Value) -> (String, String) {
    let input = input(earnings, period, scale, year, extra);
    let refusal = api::evaluate_coded("payg-withholding", &input).unwrap_err();
    (refusal.code.to_string(), refusal.message)
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key]
        .as_str()
        .unwrap_or_else(|| panic!("no {key}: {result}"))
        .to_string()
}

fn dollars(result: &Value, key: &str) -> i64 {
    let text = amount(result, key);
    let whole = text
        .strip_suffix(".00")
        .unwrap_or_else(|| panic!("{key} {text}"));
    whole.parse().unwrap()
}

const BOTH_2024_YEARS: [&str; 2] = ["2024-25", "2025-26"];

/// Every adjustment in one ATO Medicare sample data file, worked out through
/// `payg-withholding` for `year`: 48 earnings points for each of 3 pay periods,
/// one column per family. The scale 2 file has a spouse-only column (0
/// children); the scale 6 file starts at 1 child. Each result must also net the
/// adjustment off the withholding before it.
fn matches_the_medicare_sample(sample: &str, scale: i64, year: &str) -> usize {
    let mut lines = sample.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    assert_eq!(header[..2], ["period", "earnings"]);
    let children: Vec<i64> = header[2..]
        .iter()
        .map(|column| match *column {
            "spouse_only" => 0,
            other => other[..1].parse().unwrap(),
        })
        .collect();
    let rows: Vec<Vec<&str>> = lines.map(|l| l.split(',').collect()).collect();
    assert_eq!(rows.len(), 144);
    for period in ["weekly", "fortnightly", "monthly"] {
        assert_eq!(rows.iter().filter(|r| r[0] == period).count(), 48);
    }
    let mut checked = 0;
    for row in &rows {
        assert_eq!(row.len(), 2 + children.len(), "{row:?}");
        for (column, count) in children.iter().enumerate() {
            let result = run(
                row[1],
                row[0],
                scale,
                year,
                json!({"medicare_adjustment_children": count}),
            );
            let expected: i64 = row[2 + column].parse().unwrap();
            assert_eq!(
                dollars(&result, "medicare_levy_adjustment"),
                expected,
                "{year} scale {scale} {row:?} children {count}"
            );
            assert_eq!(
                dollars(&result, "withholding"),
                dollars(&result, "withholding_before_adjustments") - expected,
                "{year} scale {scale} {row:?} children {count}"
            );
            checked += 1;
        }
    }
    checked
}

#[test]
fn every_medicare_adjustment_matches_the_ato_sample_for_payments_from_july_2026() {
    let scale_2 = include_str!("data/medicare_levy_adjustment_scale_2_sample_2026_27.csv");
    let scale_6 = include_str!("data/medicare_half_levy_adjustment_scale_6_sample_2026_27.csv");
    assert_eq!(matches_the_medicare_sample(scale_2, 2, "2026-27"), 864);
    assert_eq!(matches_the_medicare_sample(scale_6, 6, "2026-27"), 720);
}

#[test]
fn every_medicare_adjustment_matches_the_ato_sample_for_payments_from_july_2024_to_june_2026() {
    let scale_2 = include_str!("data/medicare_levy_adjustment_scale_2_sample_2024_26.csv");
    let scale_6 = include_str!("data/medicare_half_levy_adjustment_scale_6_sample_2024_26.csv");
    for year in BOTH_2024_YEARS {
        assert_eq!(matches_the_medicare_sample(scale_2, 2, year), 864);
        assert_eq!(matches_the_medicare_sample(scale_6, 6, year), 720);
    }
}

/// (earnings, period, scale, children, weekly family threshold, shading out
/// point, adjustment for the pay period), from each schedule's 'Medicare levy
/// adjustment' page.
type MedicareExample = (
    &'static str,
    &'static str,
    i64,
    i64,
    &'static str,
    &'static str,
    i64,
);

fn check_medicare_examples(year: &str, examples: &[MedicareExample]) {
    for (earnings, period, scale, children, wft, sop, adjustment) in examples {
        let result = run(
            earnings,
            period,
            *scale,
            year,
            json!({"medicare_adjustment_children": children}),
        );
        assert_eq!(
            result["rates"]["weekly_family_threshold"], *wft,
            "{earnings}"
        );
        assert_eq!(result["rates"]["shading_out_point"], *sop, "{earnings}");
        assert_eq!(
            dollars(&result, "medicare_levy_adjustment"),
            *adjustment,
            "{year} {earnings}"
        );
    }
}

#[test]
fn the_medicare_levy_adjustment_examples_for_payments_from_july_2026() {
    // Zail (formula 1), Abe (scale 6, formula 2), Wendy (formula 3), Kareem
    // (fortnightly), Scott (monthly, spouse only) and Stevi (6 children: WFT
    // $1,408.96, SOP $1,761). Zail's children are not stated; formula 1 needs none.
    check_medicare_examples(
        "2026-27",
        &[
            ("633.06", "weekly", 2, 0, "908.42", "1135", 10),
            ("1142.09", "weekly", 6, 3, "1158.69", "1448", 11),
            ("1319.21", "weekly", 2, 4, "1242.12", "1552", 19),
            ("2026.77", "fortnightly", 2, 1, "991.85", "1239", 36),
            ("3066.33", "monthly", 2, 0, "908.42", "1135", 61),
            ("2000.00", "weekly", 2, 6, "1408.96", "1761", 0),
        ],
    );
}

#[test]
fn the_medicare_levy_adjustment_examples_for_payments_from_july_2024_to_june_2026() {
    // The same examples in the 2024 schedule; Stevi's 6 children give WFT
    // $1,307.85 and SOP $1,634.
    for year in BOTH_2024_YEARS {
        check_medicare_examples(
            year,
            &[
                ("535.33", "weekly", 2, 0, "843.19", "1053", 4),
                ("1064.25", "weekly", 6, 3, "1075.52", "1344", 11),
                ("1200.47", "weekly", 2, 4, "1152.96", "1441", 19),
                ("1887.52", "fortnightly", 2, 1, "920.63", "1150", 34),
                ("2800.33", "monthly", 2, 0, "843.19", "1053", 56),
                ("2000.00", "weekly", 2, 6, "1307.85", "1634", 0),
            ],
        );
    }
}

/// The 'Examples of calculating the withholding amounts' page: Jabe (Medicare
/// levy adjustment, 5 children), Prue (scale 5, tax offsets of $1,645 a year,
/// fortnightly) and Tan (tax offsets of $1,365, monthly).
#[test]
fn the_general_examples_net_the_offsets_and_adjustment() {
    for (year, jabe, prue, tan) in [
        ("2026-27", (245, 26), (86, 63), (940, 113)),
        ("2024-25", (250, 16), (92, 63), (962, 113)),
        ("2025-26", (250, 16), (92, 63), (962, 113)),
    ] {
        let result = run(
            "1333.45",
            "weekly",
            2,
            year,
            json!({"medicare_adjustment_children": 5}),
        );
        assert_eq!(dollars(&result, "withholding_before_adjustments"), jabe.0);
        assert_eq!(dollars(&result, "medicare_levy_adjustment"), jabe.1);
        assert_eq!(dollars(&result, "withholding"), jabe.0 - jabe.1, "{year}");
        for ((earnings, period, scale, offsets), (before, reduction)) in [
            (("1299.30", "fortnightly", 5, "1645"), prue),
            (("5400.33", "monthly", 2, "1365"), tan),
        ] {
            let result = run(
                earnings,
                period,
                scale,
                year,
                json!({"tax_offsets": offsets}),
            );
            assert_eq!(dollars(&result, "withholding_before_adjustments"), before);
            assert_eq!(dollars(&result, "tax_offset_reduction"), reduction);
            assert_eq!(
                dollars(&result, "withholding"),
                before - reduction,
                "{year}"
            );
        }
    }
}

#[test]
fn the_medicare_adjustment_starts_at_the_weekly_earnings_threshold_and_ends_at_the_shading_out_point()
 {
    // 2026-27 scale 2: none below $538; formula 1 from $538 ((x - 538.67) x 0.1);
    // nil from the SOP. Scale 6 with a spouse only (no sample column): formula 1
    // from $908 ((x - 908.42) x 0.05) and nil from the SOP of $1,135.
    for (earnings, scale, children, adjustment) in [
        ("537.99", 2, 0, 0),
        ("538.00", 2, 0, 0),
        ("672.99", 2, 0, 13),
        ("673.00", 2, 0, 13),
        ("1134.99", 2, 0, 0),
        ("1135.00", 2, 0, 0),
        ("907.99", 6, 0, 0),
        ("1000.00", 6, 0, 5),
        ("1134.99", 6, 0, 11),
        ("1135.00", 6, 0, 0),
        ("1752.99", 2, 6, 1),
        ("1760.99", 2, 6, 0),
        ("1761.00", 2, 6, 0),
    ] {
        let result = run(
            earnings,
            "weekly",
            scale,
            "2026-27",
            json!({"medicare_adjustment_children": children}),
        );
        assert_eq!(
            dollars(&result, "medicare_levy_adjustment"),
            adjustment,
            "{earnings} scale {scale} children {children}"
        );
    }
}

/// The schedule has no quarterly worked example or sample data. Its rule: the
/// weekly equivalent is a thirteenth of the quarter's earnings, the amount and
/// the Medicare levy adjustment are 13 times weekly, and the tax offset
/// reduction is 25% of the offsets. So a quarter of 13 times each whole-dollar weekly sample point
/// withholds 13 times the ATO's weekly sample amount.
#[test]
fn a_quarterly_pay_withholds_13_times_the_weekly_sample() {
    for (sample, years) in [
        (
            include_str!("data/payg_withholding_sample_2026_27.csv"),
            &["2026-27"][..],
        ),
        (
            include_str!("data/payg_withholding_sample_2024_26.csv"),
            &BOTH_2024_YEARS[..],
        ),
    ] {
        let weekly = sample.lines().skip(1).filter(|l| l.starts_with("weekly,"));
        let mut checked = 0;
        for line in weekly {
            let row: Vec<&str> = line.split(',').collect();
            let quarter: i64 = row[1].parse::<i64>().unwrap() * 13;
            for (column, scale) in [1, 2, 3, 5, 6].into_iter().enumerate() {
                for year in years {
                    let result = run(
                        &format!("{quarter}.00"),
                        "quarterly",
                        scale,
                        year,
                        json!({}),
                    );
                    let expected: i64 = row[2 + column].parse().unwrap();
                    assert_eq!(
                        dollars(&result, "withholding"),
                        13 * expected,
                        "{year} {line}"
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 48 * 5 * years.len());
    }
    // $13,000 a quarter on scale 2 for 2026-27: x = 1,000.99, 0.3227 x 1,000.99 -
    // 185.1935 = 137.83, so $138 a week and $1,794 a quarter.
    let result = run("13000.00", "quarterly", 2, "2026-27", json!({}));
    assert_eq!(amount(&result, "weekly_earnings_used"), "1000.99");
    assert_eq!(dollars(&result, "withholding"), 1794);
    // Boundary at $538 a week: $6,993.99 a quarter is $537.99 a week (x = 537.99,
    // $26 a week), $6,994.00 is $538 (x = 538.99, $27 a week).
    let below = run("6993.99", "quarterly", 2, "2026-27", json!({}));
    let at = run("6994.00", "quarterly", 2, "2026-27", json!({}));
    assert_eq!(
        (dollars(&below, "withholding"), dollars(&at, "withholding")),
        (338, 351)
    );
    // Medicare and offsets per quarter: Kareem's $1,013.39 a week is $13,174.07 a
    // quarter (x = 1,013.99, WLA $18), so $234; offsets of $2 at 25% give $0.50,
    // rounded up to $1, and $1.98 gives $0.495, rounded to nil.
    let kareem = run(
        "13174.07",
        "quarterly",
        2,
        "2026-27",
        json!({"medicare_adjustment_children": 1}),
    );
    assert_eq!(dollars(&kareem, "medicare_levy_adjustment"), 234);
    for (offsets, reduction) in [("2.00", 1), ("1.98", 0)] {
        let result = run(
            "13000.00",
            "quarterly",
            2,
            "2026-27",
            json!({"tax_offsets": offsets}),
        );
        assert_eq!(
            dollars(&result, "tax_offset_reduction"),
            reduction,
            "{offsets}"
        );
    }
}

/// Scale 4: 47% for a resident and 45% for a foreign resident, applied to the
/// pay's earnings with the cents ignored, and the cents of the result ignored
/// (LI 2026/18 Schedule 1, note 3 to the coefficients; both schedules).
#[test]
fn scale_4_applies_the_no_tfn_rate_ignoring_cents() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        for (earnings, period, foreign, expected) in [
            ("1234.56", "weekly", false, 579),
            ("1234.56", "weekly", true, 555),
            ("1234.56", "fortnightly", false, 579),
            ("1234.56", "monthly", true, 555),
            ("1234.56", "quarterly", false, 579),
            ("0.99", "weekly", false, 0),
            ("3.00", "weekly", false, 1),
            ("100.99", "weekly", false, 47),
            ("2.99", "weekly", true, 0),
            ("3.00", "weekly", true, 1),
        ] {
            let result = run(
                earnings,
                period,
                4,
                year,
                json!({"foreign_resident": foreign}),
            );
            assert_eq!(
                dollars(&result, "withholding"),
                expected,
                "{year} {earnings}"
            );
            let rate = if foreign { "0.45" } else { "0.47" };
            assert_eq!(result["rates"]["tax_rate"], rate);
            assert!(result["amounts"].get("weekly_earnings_used").is_none());
        }
    }
}

#[test]
fn tax_offsets_reduce_by_the_pay_period_rate_rounded_to_the_dollar() {
    // 1.9% weekly, 3.8% fortnightly, 8.3% monthly, each rounded half up.
    for (period, earnings, offsets, reduction) in [
        ("weekly", "1000.00", "1000", 19),
        ("weekly", "1000.00", "50", 1),
        ("weekly", "1000.00", "26", 0),
        ("fortnightly", "2000.00", "1645", 63),
        ("monthly", "5000.00", "1365", 113),
    ] {
        let result = run(
            earnings,
            period,
            2,
            "2026-27",
            json!({"tax_offsets": offsets}),
        );
        assert_eq!(
            dollars(&result, "tax_offset_reduction"),
            reduction,
            "{period} {offsets}"
        );
    }
    // Scale 6 takes offsets and the adjustment together: Abe's $1,142.09 with 3
    // children and offsets of $1,000 is $172 less $19 less $11.
    let result = run(
        "1142.09",
        "weekly",
        6,
        "2026-27",
        json!({"tax_offsets": "1000", "medicare_adjustment_children": 3}),
    );
    assert_eq!(dollars(&result, "withholding_before_adjustments"), 172);
    assert_eq!(dollars(&result, "tax_offset_reduction"), 19);
    assert_eq!(dollars(&result, "medicare_levy_adjustment"), 11);
    assert_eq!(dollars(&result, "withholding"), 142);
}

#[test]
fn the_additional_amounts_for_53_or_27_pays_follow_each_schedules_table() {
    let extra = |earnings: &str, period: &str, year: &str| {
        let result = run(
            earnings,
            period,
            2,
            year,
            json!({"extra_pay_withholding": true}),
        );
        let plain = run(earnings, period, 2, year, json!({}));
        assert_eq!(
            dollars(&result, "withholding"),
            dollars(&plain, "withholding") + dollars(&result, "additional_withholding"),
            "{year} {period} {earnings}"
        );
        dollars(&result, "additional_withholding")
    };
    for (year, weekly, fortnightly) in [
        ("2026-27", [0, 3, 3, 7, 7, 12], [0, 12, 12, 27, 27, 48]),
        ("2024-25", [0, 3, 3, 6, 6, 12], [0, 12, 12, 26, 26, 47]),
        ("2025-26", [0, 3, 3, 6, 6, 12], [0, 12, 12, 26, 26, 47]),
    ] {
        let weekly_points = [
            "874.99", "875.00", "2574.99", "2575.00", "3649.99", "3650.00",
        ];
        for (earnings, expected) in weekly_points.into_iter().zip(weekly) {
            assert_eq!(
                extra(earnings, "weekly", year),
                expected,
                "{year} {earnings}"
            );
        }
        let (first, second) = if year == "2026-27" {
            (["1699.99", "1700.00"], ["5199.99", "5200.00"])
        } else {
            (["1749.99", "1750.00"], ["5149.99", "5150.00"])
        };
        let fortnightly_points = [
            first[0], first[1], second[0], second[1], "7249.99", "7250.00",
        ];
        for (earnings, expected) in fortnightly_points.into_iter().zip(fortnightly) {
            assert_eq!(
                extra(earnings, "fortnightly", year),
                expected,
                "{year} {earnings}"
            );
        }
    }
    // false asks for nothing, and the result is the plain one.
    let off = run(
        "3000.00",
        "weekly",
        2,
        "2026-27",
        json!({"extra_pay_withholding": false}),
    );
    assert_eq!(off, run("3000.00", "weekly", 2, "2026-27", json!({})));
    // Scale 4 takes it too, as the schedule excludes no scale: 47% of $1,000 is
    // $470, plus $3.
    let scale_4 = run(
        "1000.00",
        "weekly",
        4,
        "2026-27",
        json!({"foreign_resident": false, "extra_pay_withholding": true}),
    );
    assert_eq!(dollars(&scale_4, "withholding"), 473);
}

#[test]
fn inputs_the_schedule_does_not_allow_for_the_scale_or_pay_are_refused() {
    let code = |scale: i64, period: &str, extra: Value| {
        refused("1000.00", period, scale, "2026-27", extra).0
    };
    // Medicare levy adjustment: scales 2 and 6 only.
    for scale in [1, 3, 4, 5] {
        let extra = if scale == 4 {
            json!({"medicare_adjustment_children": 1, "foreign_resident": false})
        } else {
            json!({"medicare_adjustment_children": 1})
        };
        assert_eq!(
            code(scale, "weekly", extra),
            "payg.medicare_adjustment_not_allowed"
        );
    }
    // Tax offsets: scales 2, 5 and 6 only.
    for scale in [1, 3] {
        assert_eq!(
            code(scale, "weekly", json!({"tax_offsets": "100"})),
            "payg.tax_offsets_not_allowed"
        );
    }
    assert_eq!(
        code(
            4,
            "weekly",
            json!({"tax_offsets": "100", "foreign_resident": false})
        ),
        "payg.tax_offsets_not_allowed"
    );
    // Scale 4 needs residency; no other scale takes it.
    assert_eq!(code(4, "weekly", json!({})), "input.missing_field");
    assert_eq!(
        code(2, "weekly", json!({"foreign_resident": false})),
        "payg.foreign_resident_not_scale_4"
    );
    assert_eq!(
        code(4, "weekly", json!({"foreign_resident": "no"})),
        "input.flag_invalid"
    );
    // The 53 and 27 pay tables are weekly and fortnightly.
    for period in ["monthly", "quarterly"] {
        assert_eq!(
            code(2, period, json!({"extra_pay_withholding": true})),
            "payg.extra_pays_not_weekly_or_fortnightly"
        );
    }
    assert_eq!(
        code(2, "weekly", json!({"extra_pay_withholding": 1})),
        "input.flag_invalid"
    );
    for children in [json!(-1), json!(1.5), json!("2"), json!(true)] {
        assert_eq!(
            code(
                2,
                "weekly",
                json!({ "medicare_adjustment_children": children })
            ),
            "input.children_invalid"
        );
    }
    assert_eq!(
        code(2, "weekly", json!({"tax_offsets": "-1"})),
        "input.amount_invalid"
    );
    assert_eq!(
        code(2, "weekly", json!({"tax_offset": "100"})),
        "input.field_unknown"
    );
    // Bi-monthly and daily pays stay outside the worksheet.
    assert_eq!(code(2, "bi-monthly", json!({})), "input.pay_period_invalid");
}

/// Schedule 1 says the offsets and the adjustment reduce the amount withheld but
/// not what happens when they exceed it, so the worksheet refuses rather than
/// assume nil. $400.00 a week on scale 2 for 2026-27 withholds $6; offsets of
/// $1,173 a year reduce it by $22.
#[test]
fn reductions_beyond_the_amount_withheld_are_refused() {
    let (code, message) = refused(
        "400.00",
        "weekly",
        2,
        "2026-27",
        json!({"tax_offsets": "1173"}),
    );
    assert_eq!(code, "payg.reductions_exceed_withholding");
    assert!(
        message.contains("$22.00") && message.contains("$6.00"),
        "{message}"
    );
    // The refusal says what to do next.
    assert!(
        message.contains("Check the payee's declarations") && message.contains("ask the ATO"),
        "{message}"
    );
    // The 53 pay amount is added after the reductions, so it cannot cover an
    // excess: offsets whose 1.9% rounds to $1 more than the amount withheld on
    // $900.00 a week are refused although the additional amount is $3.
    let before = dollars(
        &run("900.00", "weekly", 2, "2026-27", json!({})),
        "withholding",
    );
    let offsets = (before * 1000 + 500 + 18) / 19;
    let (code, message) = refused(
        "900.00",
        "weekly",
        2,
        "2026-27",
        json!({"tax_offsets": offsets.to_string(), "extra_pay_withholding": true}),
    );
    assert_eq!(code, "payg.reductions_exceed_withholding");
    assert!(
        message.contains(&format!("(${}.00)", before + 1)),
        "{message}"
    );
    // Equal is allowed: offsets of $316 a year reduce $6 by $6.00 (1.9% = 6.004).
    let result = run(
        "400.00",
        "weekly",
        2,
        "2026-27",
        json!({"tax_offsets": "316"}),
    );
    assert_eq!(amount(&result, "withholding"), "0.00");
}
