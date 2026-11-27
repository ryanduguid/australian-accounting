// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The decimal port against CPython 3.15's `decimal` in its default context:
//! each expected string is what `str()` of the Python result prints, or the
//! name of the exception it raises.
use std::cmp::Ordering;

use lodgeit_calculator_adapter::decimal::{self, Calc, Conversion, Decimal, Rounding, Signal};

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap_or_else(|_| panic!("{text} parses"))
}

fn show(result: Calc<Decimal>) -> String {
    match result {
        Ok(value) => value.to_string(),
        Err(signal) => signal.name().to_string(),
    }
}

#[test]
fn a_local_precision_applies_inside_and_is_restored_after() {
    let third = decimal::with_precision(3, || d("1").div(&d("3")));
    assert_eq!(show(third), "0.333");
    assert_eq!(decimal::prec(), 28);
    assert_eq!(show(d("1").div(&d("7"))), "0.1428571428571428571428571429");
}

#[test]
fn conversion_failures_name_their_python_exception() {
    for bad in ["1E", "1Ex", "1\u{3000}0"] {
        let failure = Decimal::parse_exact(bad).expect_err(bad);
        assert_eq!(failure, Conversion::Syntax);
        assert_eq!(failure.text(), "[<class 'decimal.ConversionSyntax'>]");
    }
    let range = Decimal::parse_exact("1E1000000000000000000").expect_err("beyond the maximum context");
    assert_eq!(range.text(), "[<class 'decimal.InvalidOperation'>]");
    assert_eq!(Signal::DivisionByZero.name(), "DivisionByZero");
    assert_eq!(Signal::Overflow.name(), "Overflow");
}

#[test]
fn a_libmpdec_string_converts_exactly() {
    assert_eq!(Decimal::from_mpd_string("-12E3").expect("valid").to_string(), "-1.2E+4");
}

#[test]
fn a_float_converts_to_its_exact_binary_value() {
    assert_eq!(
        Decimal::from_float(0.1).to_string(),
        "0.1000000000000000055511151231257827021181583404541015625"
    );
    assert_eq!(Decimal::from_float(-0.0).to_string(), "-0");
    assert_eq!(Decimal::from_float(1e20).to_string(), "100000000000000000000");
    assert_eq!(Decimal::from_float(2.5).to_string(), "2.5");
    assert_eq!(Decimal::from_float(f64::NAN).to_string(), "NaN");
    assert!(Decimal::from_float(f64::NAN).is_nan_value());
    let minus_infinity = Decimal::from_float(f64::NEG_INFINITY);
    assert_eq!(minus_infinity.to_string(), "-Infinity");
    assert!(minus_infinity.is_infinite_value() && minus_infinity.is_negative());
    // The smallest subnormal, whose exponent field is zero.
    let tiny = Decimal::from_float(5e-324);
    assert_eq!(tiny.adjusted(), -324);
    assert!(tiny.coefficient().starts_with("4940656458412465"));
}

#[test]
fn normalize_drops_trailing_zeros_and_keeps_specials() {
    for (text, expected) in [
        ("1.2300", "1.23"),
        ("0.00", "0"),
        ("-0.0", "-0"),
        ("120", "1.2E+2"),
        ("NaN", "NaN"),
        ("Infinity", "Infinity"),
        ("sNaN", "InvalidOperation"),
    ] {
        assert_eq!(show(d(text).normalize()), expected, "{text}");
    }
}

#[test]
fn abs_rounds_a_magnitude_and_keeps_specials() {
    assert_eq!(show(d("-1.5").abs()), "1.5");
    assert_eq!(show(d("-Infinity").abs()), "Infinity");
    assert_eq!(show(d("NaN").abs()), "NaN");
    assert_eq!(d("-Infinity").adjusted(), 0);
}

#[test]
fn infinities_add_and_subtract_as_python_does() {
    assert_eq!(show(d("Infinity").add(&d("-Infinity"))), "InvalidOperation");
    assert_eq!(show(d("Infinity").add(&d("1"))), "Infinity");
    assert_eq!(show(d("1").add(&d("-Infinity"))), "-Infinity");
    assert_eq!(show(d("1").sub(&d("Infinity"))), "-Infinity");
    assert_eq!(show(d("0.1").add(&d("1"))), "1.1");
}

#[test]
fn subtraction_propagates_a_nan_before_flipping_its_sign() {
    assert_eq!(show(d("1").sub(&d("-NaN"))), "-NaN");
    assert_eq!(show(d("1").sub(&d("NaN5"))), "NaN5");
}

#[test]
fn multiplication_of_specials_and_zeros() {
    assert_eq!(show(d("NaN").mul(&d("2"))), "NaN");
    assert_eq!(show(d("Infinity").mul(&d("0"))), "InvalidOperation");
    assert_eq!(show(d("Infinity").mul(&d("-2"))), "-Infinity");
    assert_eq!(show(d("0.00").mul(&d("-5"))), "-0.00");
}

#[test]
fn division_of_specials_and_long_dividends() {
    assert_eq!(show(d("NaN").div(&d("2"))), "NaN");
    assert_eq!(show(d("Infinity").div(&d("Infinity"))), "InvalidOperation");
    assert_eq!(show(d("-Infinity").div(&d("2"))), "-Infinity");
    let ones = "1".repeat(40);
    assert_eq!(show(d(&ones).div(&d("3"))), "3.703703703703703703703703704E+38");
}

#[test]
fn rounding_past_the_largest_exponent_overflows() {
    let edge = d("9.9999999999999999999999999999E999999");
    assert_eq!(show(edge.mul(&d("1"))), "Overflow");
}

#[test]
fn quantize_refuses_what_python_refuses() {
    assert_eq!(show(d("1").quantize(1_000_000, Rounding::HalfEven)), "InvalidOperation");
    assert_eq!(show(d("0.0").quantize(-3, Rounding::HalfEven)), "0.000");
    // Rounding up carries into a 29th digit.
    let nines = d("9.9999999999999999999999999999");
    assert_eq!(show(nines.quantize(-27, Rounding::HalfEven)), "InvalidOperation");
}

#[test]
fn quantize_rounds_down_and_towards_positive_infinity() {
    for (text, rounding, expected) in [
        ("1.29", Rounding::Down, "1.2"),
        ("1.20", Rounding::Down, "1.2"),
        ("1.21", Rounding::Ceiling, "1.3"),
        ("-1.29", Rounding::Ceiling, "-1.2"),
        ("1.20", Rounding::Ceiling, "1.2"),
    ] {
        assert_eq!(show(d(text).quantize(-1, rounding)), expected, "{text} {rounding:?}");
    }
}

#[test]
fn remainder_follows_the_dividends_sign_and_pythons_refusals() {
    for (a, b, expected) in [
        ("7", "3", "1"),
        ("-7", "3", "-1"),
        ("7", "-3", "1"),
        ("5.5", "2", "1.5"),
        ("Infinity", "1", "InvalidOperation"),
        ("1", "0", "InvalidOperation"),
        ("0", "0", "InvalidOperation"),
        ("1", "Infinity", "1"),
        ("NaN", "1", "NaN"),
        ("0.001", "5", "0.001"),
        ("0.00", "3", "0.00"),
        ("1E30", "7", "InvalidOperation"),
        ("99999999999999999999999999999", "1", "InvalidOperation"),
    ] {
        assert_eq!(show(d(a).rem(&d(b))), expected, "{a} % {b}");
    }
}

#[test]
fn to_integral_value_rounds_half_even_or_as_asked() {
    for (text, expected) in [
        ("2.5", "2"),
        ("3.5", "4"),
        ("120", "120"),
        ("1E+2", "1E+2"),
        ("NaN", "NaN"),
        ("Infinity", "Infinity"),
        ("sNaN", "InvalidOperation"),
    ] {
        assert_eq!(show(d(text).to_integral_value()), expected, "{text}");
    }
    assert_eq!(show(d("-2.5").to_integral_rounded(Rounding::HalfUp)), "-3");
}

#[test]
fn fixed_format_writes_specials_and_zeros_with_a_positive_exponent() {
    assert_eq!(d("Infinity").fixed_format(), "Infinity");
    assert_eq!(d("0E+3").fixed_format(), "0");
    assert_eq!(d("-0E+3").fixed_format(), "-0");
}

#[test]
fn ordering_covers_signs_zeros_and_infinities() {
    let cmp = |a: &str, b: &str| d(a).compare(&d(b)).expect("no NaN");
    assert_eq!(cmp("1", "Infinity"), Ordering::Less);
    assert_eq!(cmp("0", "-1"), Ordering::Greater);
    assert_eq!(cmp("-1", "2"), Ordering::Less);
    assert_eq!(cmp("2", "-1"), Ordering::Greater);
    assert_eq!(cmp("-1", "-2"), Ordering::Greater);
    assert_eq!(d("1.0").le(&d("1")), Ok(true));
    assert_eq!(d("1.1").le(&d("1")), Ok(false));
    assert_eq!(d("NaN").le(&d("1")), Err(Signal::InvalidOperation));
}
