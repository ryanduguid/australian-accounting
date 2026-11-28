// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use gst_review::decimal::{Calc, Decimal, Rounding, Signal};
use std::cmp::Ordering;

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn s(result: Calc<Decimal>) -> Calc<String> {
    result.map(|v| v.to_sci_string())
}

fn ok(text: &str) -> Calc<String> {
    Ok(text.to_string())
}

#[test]
fn signal_names_display_and_parsing_edge_cases_match_cpython() {
    assert_eq!(Signal::InvalidOperation.name(), "InvalidOperation");
    assert_eq!(Signal::DivisionByZero.name(), "DivisionByZero");
    assert_eq!(Signal::Overflow.name(), "Overflow");
    assert_eq!(format!("{}", d("-12.30")), "-12.30");
    assert_eq!(d("Infinity").adjusted(), 0);
    assert_eq!(d("NaN5").adjusted(), 0);
    assert_eq!(d("1E-7").to_sci_string(), "1E-7");
    for bad in [
        "1\u{a0}0",
        "nanx",
        "1e+",
        "1e-",
        "1e+x",
        "0e-99999999999999999999",
        "1e-99999999999999999999",
        "1e10000000000000000000000000",
    ] {
        assert_eq!(Decimal::parse(bad).err(), Some(Signal::InvalidOperation));
    }
}

#[test]
fn abs_add_sub_mul_and_div_cover_specials_and_long_dividends() {
    assert_eq!(s(d("-5.25").abs()), ok("5.25"));
    assert_eq!(s(d("-Infinity").abs()), ok("Infinity"));
    assert_eq!(s(d("NaN5").abs()), ok("NaN5"));
    assert_eq!(s(d("sNaN").abs()), Err(Signal::InvalidOperation));
    assert_eq!(s(d("Infinity").add(&d("Infinity"))), ok("Infinity"));
    assert_eq!(
        s(d("Infinity").add(&d("-Infinity"))),
        Err(Signal::InvalidOperation)
    );
    assert_eq!(s(d("1").add(&d("Infinity"))), ok("Infinity"));
    assert_eq!(s(d("1").add(&d("NaN5"))), ok("NaN5"));
    assert_eq!(s(d("1").add(&d("sNaN"))), Err(Signal::InvalidOperation));
    assert_eq!(s(d("10.00").sub(&d("2"))), ok("8.00"));
    assert_eq!(s(d("NaN5").sub(&d("1"))), ok("NaN5"));
    assert_eq!(s(d("1").sub(&d("sNaN"))), Err(Signal::InvalidOperation));
    assert_eq!(s(d("NaN5").mul(&d("2"))), ok("NaN5"));
    assert_eq!(s(d("Infinity").mul(&d("-2"))), ok("-Infinity"));
    assert_eq!(s(d("NaN5").div(&d("2"))), ok("NaN5"));
    assert_eq!(
        s(d("Infinity").div(&d("-Infinity"))),
        Err(Signal::InvalidOperation)
    );
    assert_eq!(s(d("Infinity").div(&d("-2"))), ok("-Infinity"));
    assert_eq!(s(d("1").div(&d("7"))), ok("0.1428571428571428571428571429"));
    assert_eq!(
        s(d("12345678901234567890123456789012345").div(&d("1"))),
        ok("1.234567890123456789012345679E+34")
    );
}

#[test]
fn rem_matches_cpython_rules() {
    assert_eq!(s(d("NaN5").rem(&d("2"))), ok("NaN5"));
    assert_eq!(s(d("1").rem(&d("sNaN"))), Err(Signal::InvalidOperation));
    assert_eq!(s(d("Infinity").rem(&d("2"))), Err(Signal::InvalidOperation));
    assert_eq!(s(d("1").rem(&d("0"))), Err(Signal::InvalidOperation));
    assert_eq!(s(d("5").rem(&d("Infinity"))), ok("5"));
    assert_eq!(s(d("0").rem(&d("3"))), ok("0"));
    assert_eq!(s(d("1").rem(&d("100"))), ok("1"));
    assert_eq!(s(d("-7").rem(&d("3"))), ok("-1"));
    assert_eq!(s(d("7").rem(&d("-3"))), ok("1"));
    assert_eq!(s(d("5.5").rem(&d("2"))), ok("1.5"));
    assert_eq!(s(d("1E+30").rem(&d("1"))), Err(Signal::InvalidOperation));
    assert_eq!(
        s(d("99999999999999999999999999990").rem(&d("1"))),
        Err(Signal::InvalidOperation)
    );
}

#[test]
fn quantize_rounding_modes_and_overflows_match_cpython() {
    assert_eq!(s(d("0").quantize(-2, Rounding::HalfUp)), ok("0.00"));
    assert_eq!(
        s(d("1").quantize(1_000_000, Rounding::HalfUp)),
        Err(Signal::InvalidOperation)
    );
    assert_eq!(
        s(d("9.9999999999999999999999999999").quantize(-27, Rounding::HalfUp)),
        Err(Signal::InvalidOperation)
    );
    assert_eq!(s(d("1.1").quantize(0, Rounding::Ceiling)), ok("2"));
    assert_eq!(s(d("-1.1").quantize(0, Rounding::Ceiling)), ok("-1"));
    assert_eq!(s(d("1.00").quantize(0, Rounding::Ceiling)), ok("1"));
    assert_eq!(s(d("0.001").quantize(0, Rounding::Ceiling)), ok("1"));
    assert_eq!(s(d("1.4").quantize(0, Rounding::HalfUp)), ok("1"));
    assert_eq!(s(d("1.0").quantize(0, Rounding::HalfUp)), ok("1"));
    assert_eq!(s(d("0.5").quantize(0, Rounding::HalfEven)), ok("0"));
    assert_eq!(s(d("1E-1000030").abs()), ok("0E-1000026"));
    assert_eq!(
        s(d("9.9999999999999999999999999995E+999999").abs()),
        Err(Signal::Overflow)
    );
}

#[test]
fn fixed_format_to_integral_and_comparisons_match_cpython() {
    assert_eq!(d("Infinity").fixed_format(), "Infinity");
    assert_eq!(d("NaN5").fixed_format(), "NaN5");
    assert_eq!(d("0E+3").fixed_format(), "0");
    assert_eq!(d("-0E+3").fixed_format(), "-0");
    assert_eq!(d("0.00").fixed_format(), "0.00");
    assert_eq!(d("1E+3").fixed_format(), "1000");
    assert_eq!(d("-12.345").fixed_format(), "-12.345");
    assert_eq!(s(d("NaN5").to_integral_value()), ok("NaN5"));
    assert_eq!(
        s(d("sNaN").to_integral_value()),
        Err(Signal::InvalidOperation)
    );
    assert_eq!(
        s(d("Infinity").to_integral_rounded(Rounding::Down)),
        ok("Infinity")
    );
    assert_eq!(s(d("5").to_integral_rounded(Rounding::Down)), ok("5"));
    assert_eq!(d("Infinity").compare(&d("Infinity")), Ok(Ordering::Equal));
    assert_eq!(d("1").lt(&d("Infinity")), Ok(true));
    assert_eq!(d("0").lt(&d("1")), Ok(true));
    assert_eq!(d("1").gt(&d("0")), Ok(true));
    assert_eq!(d("-1").lt(&d("1")), Ok(true));
    assert_eq!(d("1").gt(&d("-1")), Ok(true));
    assert_eq!(d("1.0").equals(&d("1.00")), Ok(true));
    assert_eq!(d("-2").lt(&d("-1")), Ok(true));
    assert_eq!(d("1").gt(&d("NaN")), Err(Signal::InvalidOperation));
    assert_eq!(d("1").le(&d("NaN")), Err(Signal::InvalidOperation));
}
