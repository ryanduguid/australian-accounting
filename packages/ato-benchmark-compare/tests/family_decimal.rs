// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The shared `decimal.rs` against CPython's default context (28 digits,
//! ROUND_HALF_EVEN), beside `decimal_edges.rs`. The same file runs in every
//! crate that holds that copy, so each crate's run measures its own copy.
//! Every expected value is what CPython 3.15's `decimal` returned or raised.
use ato_benchmark_compare::decimal::{Calc, Decimal, Rounding, Signal};
use std::cmp::Ordering;

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
fn abs_of_a_finite_value_drops_the_sign_and_rounds_to_the_context() {
    assert_eq!(s(d("-1.50").abs()), ok("1.50"));
    assert_eq!(
        s(d("-12345678901234567890123456789").abs()),
        ok("1.234567890123456789012345679E+28")
    );
}

#[test]
fn an_infinite_operand_decides_the_result() {
    assert_eq!(s(d("1").add(&d("-Infinity"))), ok("-Infinity"));
    assert_eq!(s(d("Infinity").sub(&d("1"))), ok("Infinity"));
    assert_eq!(s(d("Infinity").mul(&d("-2"))), ok("-Infinity"));
    assert_eq!(s(d("-Infinity").div(&d("2"))), ok("-Infinity"));
    assert_eq!(s(d("Infinity").div(&d("-2"))), ok("-Infinity"));
}

#[test]
fn a_nan_operand_propagates_and_a_signalling_one_raises() {
    assert_eq!(s(d("1").sub(&d("NaN5"))), ok("NaN5"));
    assert_eq!(s(d("sNaN").sub(&d("1"))), Err(Signal::InvalidOperation));
    assert_eq!(s(d("NaN").mul(&d("2"))), ok("NaN"));
}

#[test]
fn a_zero_quantizes_to_the_asked_exponent_with_its_sign() {
    assert_eq!(s(d("0").quantize(-2, Rounding::HalfEven)), ok("0.00"));
    assert_eq!(s(d("-0").quantize(-2, Rounding::HalfEven)), ok("-0.00"));
}

#[test]
fn a_remainder_whose_quotient_needs_more_than_28_digits_is_refused() {
    assert_eq!(
        s(d(&"9".repeat(29)).rem(&d("1"))),
        Err(Signal::InvalidOperation)
    );
    assert_eq!(s(d(&"9".repeat(28)).rem(&d("1"))), ok("0"));
}

#[test]
fn comparisons_order_signs_and_infinities_as_python_does() {
    assert_eq!(d("Infinity").compare(&d("1")), Ok(Ordering::Greater));
    assert_eq!(d("1").lt(&d("Infinity")), Ok(true));
    assert_eq!(d("-1").lt(&d("2")), Ok(true));
    assert_eq!(d("1").gt(&d("-2")), Ok(true));
    assert_eq!(d("-1").gt(&d("-2")), Ok(true));
    assert_eq!(d("-3").lt(&d("-2")), Ok(true));
}

#[test]
fn addition_aligns_the_operand_with_the_smaller_exponent() {
    assert_eq!(s(d("1.5").add(&d("2"))), ok("3.5"));
}

#[test]
fn display_is_the_scientific_string() {
    assert_eq!(d("1.50").to_string(), "1.50");
    assert_eq!(format!("{}", d("1E+3")), "1E+3");
}
