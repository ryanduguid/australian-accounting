// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::credit;
use super::input::DERIVED_KM_SHARE;
use super::rates::rates;
use crate::dates;
use crate::decimal::Rounding;

/// PCG 2021/2's own example (paras 19 to 24) and the ATO's Jim (QC64655),
/// at December 2020 rates the data file does not hold: the method's
/// arithmetic on the guideline's figures.
#[test]
fn the_basic_method_reproduces_the_guideline_examples() {
    let split = |litres: &str, km: &str, rate: &str| {
        let public = crate::engine::d(km).mul(&crate::engine::d(rate)).unwrap();
        let off = crate::engine::d(litres).sub(&public).unwrap();
        let on = credit(&public, &crate::engine::d("16.5")).unwrap();
        let off = credit(&off, &crate::engine::d("42.3")).unwrap();
        (
            on.to_string(),
            off.to_string(),
            on.add(&off).unwrap().to_string(),
        )
    };
    // We Deliver: 30,550 actual public road km at 0.35 L/km.
    assert_eq!(
        split("11430", "30550", "0.35"),
        ("1764.26".into(), "311.96".into(), "2076.22".into())
    );
    // Jim: 32,000 total km x 98.4% = 31,488 at 0.35 L/km; $1,991 at 7D.
    let derived = crate::engine::d("32000")
        .mul(&crate::engine::d(DERIVED_KM_SHARE))
        .unwrap();
    let (on, off, total) = split("11430", &derived.to_string(), "0.35");
    assert_eq!((on.as_str(), off.as_str()), ("1818.43", "173.09"));
    let label = crate::engine::d(&total)
        .quantize(0, Rounding::Down)
        .unwrap();
    assert_eq!(label.to_string(), "1991");
}

#[test]
fn the_bundled_rates_are_whole_and_contiguous() {
    let r = rates();
    assert_eq!(r.periods.len(), 16);
    assert_eq!(dates::format(r.periods[15].to), "2027-01-31");
    assert!(
        r.periods
            .iter()
            .all(|p| p.heavy.len() == 6 && p.other.len() == 6)
    );
}
