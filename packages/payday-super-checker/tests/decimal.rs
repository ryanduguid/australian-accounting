// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `decimal` against CPython's `decimal` module: special values through
//! arithmetic, long division, quantize limits, overflow and refused input.

use payday_super_checker::decimal::{Calc, Decimal, Rounding, Signal};

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn s(r: Calc<Decimal>) -> Calc<String> {
    r.map(|v| v.to_sci_string())
}

fn ok(text: &str) -> Calc<String> {
    Ok(text.to_string())
}

#[test]
fn infinities_and_nans_propagate_through_arithmetic() {
    let invalid = || -> Calc<String> { Err(Signal::InvalidOperation) };
    assert_eq!(s(d("Infinity").add(&d("-Infinity"))), invalid());
    assert_eq!(s(d("Infinity").add(&d("1"))), ok("Infinity"));
    assert_eq!(s(d("1").add(&d("-Infinity"))), ok("-Infinity"));
    assert_eq!(s(d("NaN").sub(&d("1"))), ok("NaN"));
    assert_eq!(s(d("1").sub(&d("NaN3"))), ok("NaN3"));
    assert_eq!(s(d("NaN").mul(&d("1"))), ok("NaN"));
    assert_eq!(s(d("Infinity").mul(&d("-2"))), ok("-Infinity"));
    assert_eq!(s(d("NaN").div(&d("1"))), ok("NaN"));
    assert_eq!(s(d("Infinity").div(&d("Infinity"))), invalid());
    assert_eq!(s(d("-Infinity").div(&d("2"))), ok("-Infinity"));
    assert_eq!(d("Infinity").adjusted(), 0);
    assert_eq!(d("Infinity").gt(&d("1")), Ok(true));
    assert_eq!(Signal::DivisionByZero.name(), "DivisionByZero");
}

#[test]
fn long_dividends_round_to_the_context() {
    let long = d("1234567890123456789012345678901234567890");
    assert_eq!(
        s(long.div(&d("2"))),
        ok("6.172839450617283945061728395E+38")
    );
}

#[test]
fn quantize_refuses_exponents_and_results_outside_the_context() {
    let invalid = || -> Calc<String> { Err(Signal::InvalidOperation) };
    assert_eq!(s(d("1").quantize(1_000_000, Rounding::HalfUp)), invalid());
    let carries = d("9999999999999999999999999999.5");
    assert_eq!(s(carries.quantize(0, Rounding::HalfUp)), invalid());
}

#[test]
fn rounding_carry_past_emax_overflows() {
    let edge = d(&format!("9.{}E+999999", "9".repeat(28)));
    assert_eq!(s(edge.mul(&d("1"))), Err(Signal::Overflow));
}

#[test]
fn malformed_exponents_and_inner_spaces_are_refused() {
    for bad in ["1e+", "1e5x", "1\u{2003}0"] {
        assert!(Decimal::parse(bad).is_err(), "{bad:?}");
    }
    let huge = format!("1e{}", "1".repeat(26));
    assert_eq!(
        Decimal::parse(&huge).map(|_| ()),
        Err(Signal::InvalidOperation)
    );
    assert_eq!(d("\u{2003}1.5\u{2003}").to_sci_string(), "1.5");
}

#[test]
fn display_is_the_sci_string() {
    assert_eq!(format!("{}", d("1.50")), "1.50");
    assert_eq!(d("-1E+3").to_string(), "-1E+3");
}
