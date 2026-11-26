// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `decimal`'s special values, signals and edge rounding, each expected value
//! as CPython's `decimal` module gives it under the default context.

use australian_tax_calculators::decimal::{Decimal, Rounding, Signal};

fn d(text: &str) -> Decimal {
    Decimal::parse(text).expect("valid decimal text")
}

fn s(value: Result<Decimal, Signal>) -> String {
    value.expect("no signal").to_sci_string()
}

#[test]
fn signals_name_their_python_exception_and_refusal_code() {
    let zero_division = d("1").div(&d("0")).unwrap_err();
    assert_eq!(zero_division, Signal::DivisionByZero);
    assert_eq!(zero_division.name(), "DivisionByZero");
    assert_eq!(zero_division.code(), "decimal.division_by_zero");

    let overflow = d("9E999999").mul(&d("10")).unwrap_err();
    assert_eq!(overflow, Signal::Overflow);
    assert_eq!(overflow.name(), "Overflow");
    assert_eq!(overflow.code(), "decimal.overflow");
}

#[test]
fn parse_rejects_interior_unicode_space_and_malformed_exponents() {
    // A non-ASCII space inside the text becomes a space, which the grammar refuses.
    assert_eq!(
        Decimal::parse("1\u{a0}2").unwrap_err(),
        Signal::InvalidOperation
    );
    for text in ["1E", "1E+", "1E-", "1Ex"] {
        assert_eq!(
            Decimal::parse(text).unwrap_err(),
            Signal::InvalidOperation,
            "{text}"
        );
    }
    // Exponents beyond 25 significant digits saturate and fail the bounds check.
    let huge = format!("1E{}", "1".repeat(26));
    assert_eq!(
        Decimal::parse(&huge).unwrap_err(),
        Signal::InvalidOperation
    );
    let tiny = format!("1E-{}", "1".repeat(26));
    assert_eq!(
        Decimal::parse(&tiny).unwrap_err(),
        Signal::InvalidOperation
    );
    // Leading zeros in the exponent do not count towards that limit.
    assert_eq!(d(&format!("1E{}5", "0".repeat(30))).to_sci_string(), "1E+5");
}

#[test]
fn infinity_has_adjusted_exponent_zero_and_orders_above_finite_values() {
    let inf = d("Infinity");
    assert_eq!(inf.adjusted(), 0);
    assert_eq!(inf.gt(&d("1E999999")), Ok(true));
    assert_eq!(d("-Infinity").lt(&inf), Ok(true));
    assert_eq!(inf.compare(&d("NaN")), Err(Signal::InvalidOperation));
}

#[test]
fn addition_and_subtraction_with_special_operands() {
    assert_eq!(
        d("Infinity").add(&d("-Infinity")).unwrap_err(),
        Signal::InvalidOperation
    );
    assert_eq!(s(d("Infinity").add(&d("Infinity"))), "Infinity");
    assert_eq!(s(d("-Infinity").add(&d("5"))), "-Infinity");
    assert_eq!(s(d("5").add(&d("-Infinity"))), "-Infinity");
    assert_eq!(s(d("NaN7").add(&d("1"))), "NaN7");
    assert_eq!(
        d("sNaN").add(&d("1")).unwrap_err(),
        Signal::InvalidOperation
    );

    // NaNs propagate before the sign flip: `1 - NaN5` is `NaN5`, not `-NaN5`.
    assert_eq!(s(d("1").sub(&d("NaN5"))), "NaN5");
    assert_eq!(s(d("-NaN").sub(&d("1"))), "-NaN");
    assert_eq!(s(d("1").sub(&d("Infinity"))), "-Infinity");
    assert_eq!(
        d("Infinity").sub(&d("Infinity")).unwrap_err(),
        Signal::InvalidOperation
    );
}

#[test]
fn multiplication_with_special_operands() {
    assert_eq!(s(d("NaN").mul(&d("2"))), "NaN");
    assert_eq!(
        d("Infinity").mul(&d("0")).unwrap_err(),
        Signal::InvalidOperation
    );
    assert_eq!(
        d("0").mul(&d("-Infinity")).unwrap_err(),
        Signal::InvalidOperation
    );
    assert_eq!(s(d("Infinity").mul(&d("-2"))), "-Infinity");
    assert_eq!(s(d("-3").mul(&d("-Infinity"))), "Infinity");
}

#[test]
fn division_with_special_operands_and_long_dividends() {
    assert_eq!(s(d("2").div(&d("NaN3"))), "NaN3");
    assert_eq!(
        d("Infinity").div(&d("-Infinity")).unwrap_err(),
        Signal::InvalidOperation
    );
    assert_eq!(s(d("Infinity").div(&d("-2"))), "-Infinity");
    assert_eq!(s(d("-2").div(&d("Infinity"))), "-0E-1000026");
    // A dividend longer than the divisor plus the precision rounds to 28 digits.
    assert_eq!(
        s(d("1234567890123456789012345678901").div(&d("1"))),
        "1.234567890123456789012345679E+30"
    );
}

#[test]
fn quantize_refuses_exponents_and_results_outside_the_context() {
    assert_eq!(
        d("1").quantize(1_000_000, Rounding::HalfEven).unwrap_err(),
        Signal::InvalidOperation
    );
    // Rounding carries the coefficient to 29 digits, beyond the precision.
    assert_eq!(
        d("99999999999999999999999999.995")
            .quantize(-2, Rounding::HalfEven)
            .unwrap_err(),
        Signal::InvalidOperation
    );
    assert_eq!(
        s(d("99999999999999999999999999.985").quantize(-2, Rounding::HalfEven)),
        "99999999999999999999999999.98"
    );
}

#[test]
fn remainder_with_special_operands_and_impossible_divisions() {
    assert_eq!(s(d("NaN").rem(&d("1"))), "NaN");
    assert_eq!(
        d("Infinity").rem(&d("1")).unwrap_err(),
        Signal::InvalidOperation
    );
    assert_eq!(d("1").rem(&d("0")).unwrap_err(), Signal::InvalidOperation);
    // A finite dividend over infinity is its own remainder.
    assert_eq!(s(d("-1.50").rem(&d("Infinity"))), "-1.50");
    // DivisionImpossible: the quotient needs more than 28 digits.
    assert_eq!(
        d("1E30").rem(&d("1")).unwrap_err(),
        Signal::InvalidOperation
    );
    assert_eq!(
        d(&"9".repeat(29)).rem(&d("1")).unwrap_err(),
        Signal::InvalidOperation
    );
}

#[test]
fn to_integral_value_keeps_special_values() {
    assert_eq!(s(d("NaN4").to_integral_value()), "NaN4");
    assert_eq!(s(d("-Infinity").to_integral_value()), "-Infinity");
    assert_eq!(
        d("sNaN").to_integral_value().unwrap_err(),
        Signal::InvalidOperation
    );
}

#[test]
fn fixed_format_of_special_values_and_zero_with_positive_exponent() {
    assert_eq!(d("Infinity").fixed_format(), "Infinity");
    assert_eq!(d("-NaN").fixed_format(), "-NaN");
    assert_eq!(d("0E+3").fixed_format(), "0");
    assert_eq!(d("-0E+2").fixed_format(), "-0");
}

#[test]
fn rounding_carry_at_emax_overflows() {
    // 29 nines at the largest adjusted exponent: rounding to 28 digits carries
    // the exponent past Etop.
    let edge = d(&format!("9.{}E+999999", "9".repeat(28)));
    assert_eq!(edge.mul(&d("1")).unwrap_err(), Signal::Overflow);
    // One nine fewer fits exactly.
    let fits = d(&format!("9.{}E+999999", "9".repeat(27)));
    assert_eq!(
        s(fits.mul(&d("1"))),
        format!("9.{}E+999999", "9".repeat(27))
    );
}
