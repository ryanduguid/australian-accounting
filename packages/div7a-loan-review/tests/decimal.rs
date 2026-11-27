// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `decimal`'s special values, signals and edge rounding, each expected value
//! as CPython's `decimal` module gives it under the default context.

use div7a_loan_review::decimal::{Decimal, Rounding, Signal};

fn d(text: &str) -> Decimal {
    Decimal::parse(text).expect("valid decimal text")
}

fn s(value: Result<Decimal, Signal>) -> String {
    value.expect("no signal").to_sci_string()
}

#[test]
fn signals_name_their_python_exception() {
    assert_eq!(d("1").div(&d("0")).unwrap_err().name(), "DivisionByZero");
    let overflow = d("9E999999").mul(&d("10")).unwrap_err();
    assert_eq!(overflow, Signal::Overflow);
    assert_eq!(overflow.name(), "Overflow");
    assert_eq!(d("0").div(&d("0")).unwrap_err().name(), "InvalidOperation");
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
    assert_eq!(Decimal::parse(&huge).unwrap_err(), Signal::InvalidOperation);
    let tiny = format!("1E-{}", "1".repeat(26));
    assert_eq!(Decimal::parse(&tiny).unwrap_err(), Signal::InvalidOperation);
    // Leading zeros in the exponent do not count towards that limit.
    assert_eq!(d(&format!("1E{}5", "0".repeat(30))).to_sci_string(), "1E+5");
}

#[test]
fn exponent_is_the_coefficient_scale() {
    assert_eq!(d("7").exponent(), 0);
    assert_eq!(d("1.250").exponent(), -3);
    assert_eq!(d("1E+2").exponent(), 2);
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
fn signs_order_finite_values() {
    assert_eq!(d("-1").lt(&d("1")), Ok(true));
    assert_eq!(d("1").lt(&d("-1")), Ok(false));
    assert_eq!(d("-2").lt(&d("-1")), Ok(true));
    assert_eq!(d("-1").lt(&d("-2")), Ok(false));
}

#[test]
fn addition_and_subtraction_with_special_operands() {
    assert_eq!(
        d("Infinity").add(&d("-Infinity")).unwrap_err(),
        Signal::InvalidOperation
    );
    assert_eq!(s(d("Infinity").add(&d("Infinity"))), "Infinity");
    assert_eq!(s(d("Infinity").add(&d("1"))), "Infinity");
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
    assert_eq!(s(d("-Infinity").sub(&d("1"))), "-Infinity");
    assert_eq!(
        d("Infinity").sub(&d("Infinity")).unwrap_err(),
        Signal::InvalidOperation
    );
}

#[test]
fn multiplication_with_special_operands() {
    assert_eq!(s(d("NaN").mul(&d("2"))), "NaN");
    assert_eq!(s(d("NaN").mul(&d("Infinity"))), "NaN");
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
fn round_down_truncates_towards_zero() {
    assert_eq!(s(d("1.29").quantize(-1, Rounding::Down)), "1.2");
    assert_eq!(s(d("-1.29").quantize(-1, Rounding::Down)), "-1.2");
    assert_eq!(s(d("1.20").quantize(-1, Rounding::Down)), "1.2");
}

#[test]
fn remainder_keeps_the_dividend_sign() {
    assert_eq!(s(d("7.5").rem(&d("2"))), "1.5");
    assert_eq!(s(d("-7.5").rem(&d("2"))), "-1.5");
    // A dividend far smaller than the divisor is its own remainder.
    assert_eq!(s(d("0.001").rem(&d("7"))), "0.001");
    assert_eq!(s(d("0").rem(&d("3.0"))), "0.0");
}

#[test]
fn remainder_with_special_operands_and_impossible_divisions() {
    assert_eq!(s(d("NaN").rem(&d("1"))), "NaN");
    assert_eq!(s(d("1").rem(&d("NaN2"))), "NaN2");
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
fn integral_powers_follow_libmpdec() {
    assert_eq!(s(d("0.001").pow_integral(&d("2"))), "0.000001");
    // A base of one keeps its places times the exponent, capped at prec - 1.
    assert_eq!(s(d("1.000").pow_integral(&d("3"))), "1.000000000");
    assert_eq!(
        s(d("1.0").pow_integral(&d("100"))),
        "1.000000000000000000000000000"
    );
    assert_eq!(s(d("1.5").pow_integral(&d("Infinity"))), "Infinity");
    assert_eq!(s(d("0.5").pow_integral(&d("Infinity"))), "0");
    assert_eq!(
        d("1.5").pow_integral(&d("1E10")).unwrap_err(),
        Signal::Overflow
    );
}

#[test]
fn fixed_format_writes_every_digit() {
    assert_eq!(d("1.250").fixed_format(), "1.250");
    assert_eq!(d("1E+2").fixed_format(), "100");
    assert_eq!(d("-0.0012").fixed_format(), "-0.0012");
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
    let fits = d(&format!("9.{}E+999999", "9".repeat(27)));
    assert_eq!(
        s(fits.mul(&d("1"))),
        format!("9.{}E+999999", "9".repeat(27))
    );
}

#[test]
fn a_trapped_signal_as_a_failure_reads_as_python_names_it() {
    use div7a_loan_review::Failure;
    let failure = Failure::from(Signal::Overflow);
    assert_eq!(failure, Failure::Signal(Signal::Overflow));
    assert_eq!(failure.to_string(), "decimal.Overflow");
}
