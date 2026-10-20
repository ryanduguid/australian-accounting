//! The transcribed tables checked against each other and the published figures,
//! and every value in the ATO's Schedule 1 sample data.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::{Decimal, Rounding};
use australian_tax_calculators::metadata::{
    CoefficientRow, LOW_INCOME_TAX_OFFSET, MEDICARE_LOW_INCOME_THRESHOLDS,
    PAYG_WITHHOLDING_COEFFICIENTS, RESIDENT_TAX_SCALES, STUDY_LOAN_REPAYMENT_THRESHOLDS,
    STUDY_LOAN_WITHHOLDING_COEFFICIENTS, supported_periods,
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
    let published = [
        ("2024-25", ("27222", "34027")),
        ("2025-26", ("28011", "35013")),
    ];
    assert_eq!(MEDICARE_LOW_INCOME_THRESHOLDS, published);
    for (_, (lower, upper)) in published {
        // 10% of the excess meets 2% of income at 1.25 x the threshold, rounded down.
        let limit = floor(&d(lower).mul(&d("1.25")).unwrap());
        assert!(d(upper).equals(&limit).unwrap());
    }
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

#[test]
fn every_scale_matches_the_ato_sample() {
    let sample = include_str!("data/payg_withholding_sample_2026_27.csv");
    let mut lines = sample.lines();
    assert_eq!(
        lines.next(),
        Some("period,earnings,scale1,scale2,scale3,scale5,scale6")
    );
    let rows: Vec<Vec<&str>> = lines.map(|l| l.split(',').collect()).collect();
    // 48 earnings points for each of 3 pay periods, as published on 17 June 2026.
    assert_eq!(rows.len(), 144);
    for period in ["weekly", "fortnightly", "monthly"] {
        assert_eq!(rows.iter().filter(|r| r[0] == period).count(), 48);
    }
    for row in &rows {
        for (column, scale) in [1, 2, 3, 5, 6].into_iter().enumerate() {
            let input = json!({"earnings": row[1], "pay_period": row[0], "scale": scale,
                               "year": "2026-27", "scope_confirmed": true});
            let result = api::evaluate("payg-withholding", &input).unwrap();
            let expected = d(row[2 + column]).fixed(2, false);
            assert_eq!(
                result["amounts"]["withholding"], expected,
                "{row:?} scale {scale}"
            );
        }
    }
}
