// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Comparison edges a caller of the library can reach by building the
//! figures or the comparison itself.

use ato_benchmark_compare::dataset::{self, DataDir, Dataset};
use ato_benchmark_compare::decimal::Decimal;
use ato_benchmark_compare::ratios::{Figures, compute};
use ato_benchmark_compare::report::{compare, industry_page_source};

const BAKERY: &str = "Bakeries and hot bread shops";

fn shipped() -> Dataset {
    dataset::load(Some("2023-24"), &DataDir::Shipped).unwrap()
}

fn figures() -> Figures {
    let totals: Vec<(String, Decimal)> = [("turnover", "850000"), ("rent", "40000")]
        .iter()
        .map(|(k, v)| (k.to_string(), Decimal::parse(v).unwrap()))
        .collect();
    let supplied: Vec<String> = totals.iter().map(|(k, _)| k.clone()).collect();
    compute(&totals, &supplied, None).unwrap()
}

#[test]
fn a_ratio_the_figures_do_not_hold_gets_no_verdict() {
    let data = shipped();
    let bakery = data.get(BAKERY).unwrap();
    let mut without_rent = figures();
    without_rent.ratios.retain(|(k, _)| k != "rent_to_turnover");
    let keys = |f: &Figures| -> Vec<String> {
        compare(&data, bakery, f, None)
            .unwrap()
            .verdicts
            .into_iter()
            .map(|v| v.key)
            .collect()
    };
    let all = keys(&figures());
    assert!(all.iter().any(|k| k == "rent_to_turnover"));
    let expected: Vec<String> = all
        .into_iter()
        .filter(|k| k != "rent_to_turnover")
        .collect();
    assert_eq!(keys(&without_rent), expected);
}

#[test]
fn only_a_known_ratio_has_a_numerator() {
    let f = figures();
    assert_eq!(
        f.numerator("rent_to_turnover").map(Decimal::to_sci_string),
        Some("40000".into())
    );
    assert!(f.numerator("wages_to_turnover").is_none());
}

#[test]
fn a_band_without_page_ranges_has_no_page_source() {
    let data = shipped();
    let mut comparison = compare(&data, data.get(BAKERY).unwrap(), &figures(), None).unwrap();
    assert!(!industry_page_source(&comparison).is_null());
    comparison.band.as_mut().unwrap().page_ratios.clear();
    assert!(industry_page_source(&comparison).is_null());
}
