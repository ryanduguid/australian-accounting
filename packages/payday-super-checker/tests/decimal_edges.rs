// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `decimal` edges against CPython's default context (28 digits, Emax
//! 999999, Emin -999999, so Etiny -1000026 and Etop 999972). `decimal.rs` is
//! a byte-for-byte copy shared with other crates, so its tests live here.
//! Every expected value is what CPython's `decimal` returned or raised.
use payday_super_checker::decimal::{Calc, Decimal, Rounding, Signal, with_precision};

const INVALID: Calc<String> = Err(Signal::InvalidOperation);

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
fn signals_carry_cpython_class_names() {
    assert_eq!(Signal::InvalidOperation.name(), "InvalidOperation");
    assert_eq!(Signal::DivisionByZero.name(), "DivisionByZero");
    assert_eq!(Signal::Overflow.name(), "Overflow");
}

#[test]
fn parse_refuses_inner_whitespace_and_bad_exponents() {
    // An ideographic space inside the text is a space, which the grammar refuses.
    assert_eq!(s(Decimal::parse("1\u{3000}2")), INVALID);
    assert_eq!(s(Decimal::parse("1e+")), INVALID);
    assert_eq!(s(Decimal::parse("1e5x")), INVALID);
    // An exponent of more than 25 significant digits saturates and fails the bounds.
    let huge = "9".repeat(30);
    assert_eq!(s(Decimal::parse(&format!("1e{huge}"))), INVALID);
    assert_eq!(s(Decimal::parse(&format!("0e-{huge}"))), INVALID);
    assert_eq!(
        s(Decimal::parse(&format!("1e000{}", "0".repeat(30)))),
        ok("1")
    );
}

#[test]
fn specials_follow_pydecimal_arithmetic() {
    assert_eq!(d("Infinity").adjusted(), 0);
    assert_eq!(d("NaN").adjusted(), 0);
    assert_eq!(d("12.5").exponent(), -1);
    assert_eq!(s(d("-NaN7").abs()), ok("-NaN7"));
    assert_eq!(s(d("-Infinity").abs()), ok("Infinity"));
    assert_eq!(s(d("sNaN").abs()), INVALID);
    assert_eq!(s(d("Infinity").add(&d("-Infinity"))), INVALID);
    assert_eq!(s(d("Infinity").add(&d("Infinity"))), ok("Infinity"));
    assert_eq!(s(d("1").add(&d("NaN3"))), ok("NaN3"));
    assert_eq!(s(d("NaN2").div(&d("1"))), ok("NaN2"));
    assert_eq!(s(d("Infinity").div(&d("-Infinity"))), INVALID);
}

#[test]
fn division_with_a_longer_dividend_rounds_to_28_digits() {
    // 34 digits over 3: the dividend is scaled down, not the divisor up.
    let dividend = d("1234567890123456789012345678901234");
    assert_eq!(
        s(dividend.div(&d("3"))),
        ok("4.115226300411522630041152263E+32")
    );
}

#[test]
fn quantize_refuses_out_of_range_exponents_and_carries() {
    assert_eq!(s(d("1").quantize(1_000_000, Rounding::HalfEven)), INVALID);
    assert_eq!(s(d("1").quantize(-1_000_027, Rounding::HalfEven)), INVALID);
    // 28 nines and a half round up to 29 digits, more than the precision.
    let nines = format!("{}.5", "9".repeat(28));
    assert_eq!(s(d(&nines).quantize(0, Rounding::HalfUp)), INVALID);
}

#[test]
fn rounding_that_carries_past_etop_overflows() {
    // 29 nines at exponent 999971 round to 28 digits at 999973, above Etop.
    let top = format!("{}E+999971", "9".repeat(29));
    assert_eq!(s(d(&top).neg()), Err(Signal::Overflow));
}

#[test]
fn remainders_and_integral_values_match_cpython() {
    assert_eq!(s(d("7").rem(&d("-3"))), ok("1"));
    assert_eq!(s(d("-7.5").rem(&d("2"))), ok("-1.5"));
    assert_eq!(s(d("0.01").rem(&d("7"))), ok("0.01"));
    assert_eq!(s(d("5").rem(&d("Infinity"))), ok("5"));
    assert_eq!(s(d("Infinity").rem(&d("1"))), INVALID);
    assert_eq!(s(d("1").rem(&d("0"))), INVALID);
    assert_eq!(s(d("NaN").rem(&d("1"))), ok("NaN"));
    assert_eq!(s(d("1E+40").rem(&d("3"))), INVALID);
    assert_eq!(s(d("2.5").to_integral_value()), ok("2"));
    assert_eq!(s(d("3.5").to_integral_value()), ok("4"));
    assert_eq!(s(d("12E+1").to_integral_value()), ok("1.2E+2"));
    assert_eq!(s(d("-Infinity").to_integral_value()), ok("-Infinity"));
    assert_eq!(s(d("NaN4").to_integral_value()), ok("NaN4"));
    assert_eq!(s(d("2.1").to_integral_rounded(Rounding::Ceiling)), ok("3"));
    assert_eq!(
        s(d("-2.1").to_integral_rounded(Rounding::Ceiling)),
        ok("-2")
    );
    assert_eq!(s(d("2.0").to_integral_rounded(Rounding::Ceiling)), ok("2"));
    assert_eq!(s(d("2.9").to_integral_rounded(Rounding::Down)), ok("2"));
    assert_eq!(s(d("2.0").to_integral_rounded(Rounding::Down)), ok("2"));
}

#[test]
fn fixed_format_writes_every_digit() {
    assert_eq!(d("1.2E+3").fixed_format(), "1200");
    assert_eq!(d("-0E+3").fixed_format(), "-0");
    assert_eq!(d("1E-3").fixed_format(), "0.001");
    assert_eq!(d("NaN").fixed_format(), "NaN");
}

#[test]
fn precision_applies_inside_with_precision_only() {
    let third = || s(d("1").div(&d("3")));
    assert_eq!(with_precision(5, third), ok("0.33333"));
    assert_eq!(third(), ok("0.3333333333333333333333333333"));
}
