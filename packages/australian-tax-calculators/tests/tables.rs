// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The transcribed tables checked against each other and the published figures,
//! and every value in the ATO's Schedule 1 sample data.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::{Decimal, Rounding};
use australian_tax_calculators::metadata::{
    CoefficientRow, FOREIGN_RESIDENT_TAX_SCALES, LOW_INCOME_TAX_OFFSET,
    MEDICARE_LOW_INCOME_THRESHOLDS, PAYG_WITHHOLDING_COEFFICIENTS, RESIDENT_TAX_SCALES,
    STUDY_LOAN_REPAYMENT_THRESHOLDS, STUDY_LOAN_WITHHOLDING_COEFFICIENTS, supported_periods,
};
use serde_json::json;

mod common;

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn n(value: i64) -> Decimal {
    Decimal::from_int(value)
}

fn get<'a, T>(table: &'a [(&str, T)], key: &str) -> &'a T {
    &table.iter().find(|(k, _)| *k == key).unwrap().1
}

/// `value.to_integral_value(ROUND_FLOOR)` for a non-negative value.
fn floor(value: &Decimal) -> Decimal {
    value.quantize(0, Rounding::Down).unwrap()
}

#[test]
fn every_supported_resident_period_has_its_own_contiguous_scale() {
    let years: Vec<&str> = RESIDENT_TAX_SCALES.iter().map(|(y, _)| *y).collect();
    assert_eq!(supported_periods("resident_tax"), years);
    for (year, scale) in RESIDENT_TAX_SCALES {
        let lows: Vec<i64> = scale.iter().map(|r| r.0).collect();
        assert!(lows.is_sorted(), "{year}");
        for pair in scale.windows(2) {
            assert_eq!(pair[0].1, Some(pair[1].0), "{year}: brackets must abut");
        }
        assert_eq!(
            scale.last().unwrap().1,
            None,
            "{year}: the top bracket is open"
        );
        for (_, _, rate) in *scale {
            assert!(
                d("0").lt(&d(rate)).unwrap() && d(rate).lt(&d("1")).unwrap(),
                "{year}"
            );
        }
    }
}

/// Income Tax Rates Act 1986 Schedule 7 Part II, compilations 63, 65 and 66
/// (read on 14 November 2026): from $0, the second resident personal tax rate
/// (Part I item 2 for the year) to $135,000, the third (item 3) to $190,000 and
/// 45% above, for each year the resident scale covers.
#[test]
fn the_foreign_resident_scale_takes_the_years_resident_rates() {
    let years: Vec<&str> = FOREIGN_RESIDENT_TAX_SCALES
        .iter()
        .map(|(y, _)| *y)
        .collect();
    assert_eq!(supported_periods("foreign_resident_tax"), years);
    assert_eq!(supported_periods("resident_tax"), years);
    for (year, scale) in FOREIGN_RESIDENT_TAX_SCALES {
        let resident = get(RESIDENT_TAX_SCALES, year);
        assert_eq!(
            *scale,
            [
                (0, Some(135000), resident[1].2),
                (135000, Some(190000), resident[2].2),
                (190000, None, "0.45"),
            ],
            "{year}"
        );
        assert_eq!((resident[1].0, resident[2].0), (45000, 135000), "{year}");
    }
}

/// The law these worksheets apply is the same in 2026-27, so each result
/// differs from the 2025-26 one only in its period.
#[test]
fn worksheets_opened_for_2026_27_match_2025_26() {
    for (command, input) in [
        ("gst", json!({"amount": "1100.00", "gst_inclusive": true})),
        (
            "capital-gains",
            json!({"other_gains": "100.00", "discount_gains": "1000.00",
                   "current_losses": "200.00", "prior_losses": "300.00"}),
        ),
        (
            "depreciation",
            json!({"cost": "3000.00", "effective_life": "4", "days": 365,
                   "taxable_use": "0.4", "method": "diminishing_value"}),
        ),
    ] {
        let run = |year: &str| {
            let mut input = input.clone();
            input["year"] = json!(year);
            input["scope_confirmed"] = json!(true);
            api::evaluate(command, &input).unwrap()
        };
        let mut later = run("2026-27");
        assert_eq!(later["period"], "2026-27");
        later["period"] = json!("2025-26");
        assert_eq!(later, run("2025-26"), "{command}");
    }
    let input = json!({"cost": "3000.00", "effective_life": "4", "days": 366,
        "taxable_use": "0.4", "method": "prime_cost", "scope_confirmed": true, "year": "2026-27"});
    assert_eq!(
        api::evaluate("depreciation", &input).unwrap_err(),
        "days must be an integer from 0 to 365 for 2026-27."
    );
}

/// The FBT year ending 31 March 2027 keeps the 47% FBT rate and the 2.0802 and
/// 1.8868 gross-up rates (ATO FBT rates and thresholds, read 4 November 2026),
/// so the ATO's MantCo example (Calculating your FBT) gives the same figures:
/// type 1 $16,500 × 2.0802 = $34,323.30, item 14A $34,323; type 2 $6,000 ×
/// 1.8868 = $11,320.80, item 14B $11,321; item 15 $45,644; item 16 $45,644 ×
/// 47% = $21,452.68. The estimate keeps the gross-up precision:
/// ($34,323.30 + $11,320.80) × 47% = $21,452.727, so $21,452.73.
#[test]
fn fbt_opened_for_2027_matches_mantco() {
    let run = |year_ended: i64| {
        api::evaluate(
            "fbt",
            &json!({"type_one_value": "16500.00", "type_two_value": "6000.00",
                    "year_ended": year_ended, "scope_confirmed": true}),
        )
    };
    let later = run(2027).unwrap();
    assert_eq!(later["period"], "year ended 31 March 2027");
    let amounts = &later["amounts"];
    assert_eq!(amounts["type_one_grossed_up"], "34323.30");
    assert_eq!(amounts["type_two_grossed_up"], "11320.80");
    assert_eq!(amounts["return_item_14a"], "34323.00");
    assert_eq!(amounts["return_item_14b"], "11321.00");
    assert_eq!(amounts["return_item_15"], "45644.00");
    assert_eq!(amounts["return_item_16"], "21452.68");
    assert_eq!(amounts["fbt_estimate"], "21452.73");
    let mut earlier = run(2026).unwrap();
    assert_eq!(earlier["period"], "year ended 31 March 2026");
    earlier["period"] = later["period"].clone();
    assert_eq!(later, earlier);
    assert_eq!(
        run(2028).unwrap_err(),
        "Unsupported period '2028'; supported: 2026, 2027."
    );
}

/// The ATO says CGT reforms take effect from 1 July 2027:
/// https://softwaredevelopers.ato.gov.au/reforming-negative-gearing-dsp-special-purpose-working-group.
/// Later income years remain outside this worksheet until it handles the reform.
#[test]
fn capital_gains_stops_before_the_income_year_the_reform_affects() {
    for year in supported_periods("capital_gains") {
        let start: i32 = year[..4].parse().unwrap();
        assert!(
            start < 2027,
            "capital_gains {year}: implement the 1 July 2027 CGT reform before adding this year"
        );
    }
}

#[test]
fn lito_second_step_amount_and_cut_out_follow_from_the_table() {
    let rule = |key: &str| d(get(&LOW_INCOME_TAX_OFFSET, key));
    let first = rule("second_step").sub(&rule("first_step")).unwrap();
    let expected = rule("maximum")
        .sub(&rule("first_rate").mul(&first).unwrap())
        .unwrap();
    assert!(rule("second_amount").equals(&expected).unwrap());
    // s 61-110(1)(b): $66,667 is the first whole dollar at which item 3 reaches nil.
    let reaches_nil = rule("second_step")
        .add(&rule("second_amount").div(&rule("second_rate")).unwrap())
        .unwrap();
    let ceiling = floor(&reaches_nil).add(&n(1)).unwrap();
    assert!(!floor(&reaches_nil).equals(&reaches_nil).unwrap());
    assert!(rule("cut_out").equals(&ceiling).unwrap());
}

#[test]
fn medicare_thresholds_match_the_published_table_and_phase_in() {
    // 2026-27: Medicare Levy Act 1986 s 3(1), compilation 53, with Act No. 58,
    // 2026, Sch 5 item 14 (2025-26 and later years), both read on 14 November 2026.
    let published = [
        ("2024-25", ("27222", "34027")),
        ("2025-26", ("28011", "35013")),
        ("2026-27", ("28011", "35013")),
    ];
    assert_eq!(MEDICARE_LOW_INCOME_THRESHOLDS, published);
    for (_, (lower, upper)) in published {
        // 10% of the excess meets 2% of income at 1.25 x the threshold, rounded down.
        let limit = floor(&d(lower).mul(&d("1.25")).unwrap());
        assert!(d(upper).equals(&limit).unwrap());
    }
}

/// 2026-27 at the enacted threshold amount ($28,011) and phase-in limit
/// ($35,013): s 7(1) gives nil at or below the threshold, s 7(2) caps the levy
/// at 10% of the excess up to the limit, and s 6(1) charges 2% of the whole above it.
#[test]
fn medicare_levy_2026_27_applies_the_thresholds_enacted_for_later_years() {
    let run = |income: &str, year: &str| {
        api::evaluate(
            "medicare-levy",
            &json!({"taxable_income": income, "section_9a_adjustment_applies": false,
                    "year": year, "scope_confirmed": true}),
        )
    };
    // (taxable income, 2% of it, levy); the reduction is their difference.
    for (income, full, levy) in [
        ("0", "0.00", "0.00"),
        ("28010", "560.20", "0.00"),     // below the threshold: nil
        ("28011", "560.22", "0.00"),     // at the threshold: nil
        ("28012", "560.24", "0.10"),     // 10% x (28,012 - 28,011)
        ("29000", "580.00", "98.90"),    // 10% x 989, as in the ATO's 2025-26 Angie example
        ("35013", "700.26", "700.20"),   // at the limit: 10% x 7,002 < 2% x 35,013
        ("35014", "700.28", "700.28"),   // above the limit: 2% x 35,014
        ("50000", "1000.00", "1000.00"), // 2% x 50,000
    ] {
        let result = run(income, "2026-27").unwrap();
        let amounts = &result["amounts"];
        assert_eq!(result["period"], "2026-27");
        assert_eq!(amounts["levy_at_full_rate"], full, "{income}");
        assert_eq!(amounts["medicare_levy"], levy, "{income}");
        let reduction = d(full).sub(&d(levy)).unwrap();
        let reduction = reduction.quantize(-2, Rounding::HalfUp).unwrap();
        assert_eq!(
            amounts["low_income_reduction"],
            reduction.to_string(),
            "{income}"
        );
        assert_eq!(result["rates"]["threshold_amount"], "28011");
        assert_eq!(result["rates"]["phase_in_limit"], "35013");
        let warnings = result["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 3, "{income}");
        assert_eq!(
            warnings[2],
            "2026-27 uses the threshold amount and phase-in limit Medicare Levy Act 1986 s 3(1) \
             sets for 2025-26 and later years (Act No. 58, 2026, Sch 5 item 14). The ATO had not \
             published 2026-27 thresholds when this was checked on 14 November 2026. That Act, \
             assented on 30 June 2026, raised the figures for 2025-26, so a later Act can raise \
             them for 2026-27 and change this result."
        );
        let sources = result["sources"].as_array().unwrap();
        assert!(sources.contains(&json!(
            "https://www.legislation.gov.au/C2026A00058/asmade/text"
        )));
    }
    // The ATO published the 2025-26 thresholds, so that year carries no such warning.
    let earlier = run("29000", "2025-26").unwrap();
    assert_eq!(earlier["warnings"].as_array().unwrap().len(), 2);
    assert_eq!(
        run("29000", "2027-28").unwrap_err(),
        "Unsupported period '2027-28'; supported: 2024-25, 2025-26, 2026-27."
    );
}

#[test]
fn study_loan_crossover_is_derived_from_the_exact_base() {
    let published = [
        ("2025-26", ["67000", "125000", "8700", "179286"]),
        ("2026-27", ["69528", "129717", "9028", "186051"]),
    ];
    for (year, figures) in published {
        assert_eq!(
            get(STUDY_LOAN_REPAYMENT_THRESHOLDS, year).map(|(_, v)| v),
            figures
        );
        let [minimum, step, _, flat_from] = figures;
        let base = d(step).sub(&d(minimum)).unwrap().mul(&d("0.15")).unwrap();
        let step_value: i64 = step.parse().unwrap();
        let first = (step_value..)
            .find(|income| {
                let flat = n(*income).mul(&d("0.10")).unwrap();
                let marginal = base
                    .add(&n(income - step_value).mul(&d("0.17")).unwrap())
                    .unwrap();
                flat.le(&marginal).unwrap()
            })
            .unwrap();
        assert_eq!(first.to_string(), flat_from, "{year}");
    }
}

fn pick(rows: &[CoefficientRow], x: &Decimal) -> (Decimal, Decimal) {
    let (_, a, b) = rows
        .iter()
        .find(|(top, _, _)| top.is_none_or(|top| x.lt(&n(top)).unwrap()))
        .unwrap();
    (d(a), d(b))
}

#[test]
fn combined_rows_are_schedule_1_plus_the_component_rows() {
    // The schedule's combined coefficients for scales 1 and 2, to 4 decimal places.
    let schedule_1 = get(PAYG_WITHHOLDING_COEFFICIENTS, "2026-27");
    let component = get(STUDY_LOAN_WITHHOLDING_COEFFICIENTS, "2026-27");
    for (scale, table, rows) in common::COMBINED {
        for (limit, a, b) in rows {
            let x = limit.map_or(n(5000), |l| n(l - 1));
            let s1_rows = schedule_1.iter().find(|(s, _)| *s == scale).unwrap().1;
            let s1 = pick(s1_rows, &x);
            let c = pick(get(component, table), &x);
            assert!(
                s1.0.add(&c.0).unwrap().equals(&d(a)).unwrap(),
                "{scale} {limit:?}"
            );
            let gap = s1.1.add(&c.1).unwrap().sub(&d(b)).unwrap();
            assert!(gap.le(&d("0.0001")).unwrap() && d("-0.0001").le(&gap).unwrap());
        }
    }
}

#[test]
fn every_component_year_has_schedule_1_coefficients() {
    for (year, _) in STUDY_LOAN_WITHHOLDING_COEFFICIENTS {
        assert!(
            PAYG_WITHHOLDING_COEFFICIENTS.iter().any(|(y, _)| y == year),
            "{year}"
        );
    }
}

/// Every amount in one ATO Schedule 1 sample data file, worked out for `year`:
/// 48 earnings points for each of 3 pay periods on scales 1, 2, 3, 5 and 6.
fn matches_the_ato_sample(sample: &str, year: &str) {
    let mut lines = sample.lines();
    assert_eq!(
        lines.next(),
        Some("period,earnings,scale1,scale2,scale3,scale5,scale6")
    );
    let rows: Vec<Vec<&str>> = lines.map(|l| l.split(',').collect()).collect();
    assert_eq!(rows.len(), 144);
    for period in ["weekly", "fortnightly", "monthly"] {
        assert_eq!(rows.iter().filter(|r| r[0] == period).count(), 48);
    }
    for row in &rows {
        for (column, scale) in [1, 2, 3, 5, 6].into_iter().enumerate() {
            let input = json!({"earnings": row[1], "pay_period": row[0], "scale": scale,
                               "year": year, "scope_confirmed": true});
            let result = api::evaluate("payg-withholding", &input).unwrap();
            let expected = d(row[2 + column]).fixed(2, false);
            assert_eq!(
                result["amounts"]["withholding"], expected,
                "{year} {row:?} scale {scale}"
            );
        }
    }
}

#[test]
fn every_scale_matches_the_ato_sample() {
    // Payments from 1 July 2026; published on 17 June 2026.
    let sample = include_str!("data/payg_withholding_sample_2026_27.csv");
    matches_the_ato_sample(sample, "2026-27");
}

#[test]
fn every_scale_matches_the_ato_sample_for_payments_from_july_2024_to_june_2026() {
    // Published on 17 June 2024 for payments from 1 July 2024 to 30 June 2026.
    let sample = include_str!("data/payg_withholding_sample_2024_26.csv");
    for year in ["2024-25", "2025-26"] {
        matches_the_ato_sample(sample, year);
    }
}

#[test]
fn a_year_without_its_own_coefficients_is_refused() {
    for year in ["2023-24", "2027-28"] {
        let input = json!({"earnings": "1000", "pay_period": "weekly", "scale": 2,
                           "year": year, "scope_confirmed": true});
        assert_eq!(
            api::evaluate("payg-withholding", &input).unwrap_err(),
            format!("Unsupported period '{year}'; supported: 2024-25, 2025-26, 2026-27.")
        );
    }
}
