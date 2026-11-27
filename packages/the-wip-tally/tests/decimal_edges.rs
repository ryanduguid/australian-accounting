// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Decimal edge cases the schedule can meet: infinities, NaN payloads,
//! exponent limits and overflow. Each expected value or signal is what
//! CPython's `decimal` module gives under the default context.

use std::cmp::Ordering;
use the_wip_tally::decimal::{Calc, Decimal, Rounding, Signal};

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn s(result: Calc<Decimal>) -> Calc<String> {
    result.map(|v| v.to_string())
}

#[test]
fn infinities_add_and_subtract_as_python_does() {
    assert_eq!(s(d("Infinity").add(&d("1"))), Ok("Infinity".into()));
    assert_eq!(s(d("1").add(&d("Infinity"))), Ok("Infinity".into()));
    assert_eq!(s(d("Infinity").add(&d("Infinity"))), Ok("Infinity".into()));
    assert_eq!(
        s(d("Infinity").add(&d("-Infinity"))),
        Err(Signal::InvalidOperation)
    );
}

#[test]
fn a_quiet_nan_operand_carries_its_payload_through_each_operation() {
    assert_eq!(s(d("1").sub(&d("NaN12"))), Ok("NaN12".into()));
    assert_eq!(s(d("NaN3").mul(&d("2"))), Ok("NaN3".into()));
    assert_eq!(s(d("2").div(&d("-NaN5"))), Ok("-NaN5".into()));
}

#[test]
fn infinite_products_and_quotients_take_the_combined_sign() {
    assert_eq!(s(d("Infinity").mul(&d("-2"))), Ok("-Infinity".into()));
    assert_eq!(s(d("Infinity").div(&d("-2"))), Ok("-Infinity".into()));
    assert_eq!(s(d("2").div(&d("Infinity"))), Ok("0E-1000026".into()));
    assert_eq!(
        s(d("Infinity").div(&d("Infinity"))),
        Err(Signal::InvalidOperation)
    );
}

#[test]
fn a_dividend_longer_than_the_precision_rounds_to_28_digits() {
    let long = d("1234567890123456789012345678901");
    assert_eq!(
        s(long.div(&d("1"))),
        Ok("1.234567890123456789012345679E+30".into())
    );
    assert_eq!(
        s(long.div(&d("7"))),
        Ok("1.763668414462081127160493827E+29".into())
    );
}

#[test]
fn quantize_refuses_an_exponent_outside_the_context() {
    assert_eq!(
        d("1")
            .quantize(1_000_000, Rounding::HalfEven)
            .map(|v| v.to_string()),
        Err(Signal::InvalidOperation)
    );
    assert_eq!(
        d("1")
            .quantize(-1_000_028, Rounding::HalfUp)
            .map(|v| v.to_string()),
        Err(Signal::InvalidOperation)
    );
}

#[test]
fn fixed_format_prints_specials_and_drops_a_zeros_positive_exponent() {
    assert_eq!(d("Infinity").fixed_format(), "Infinity");
    assert_eq!(d("-NaN7").fixed_format(), "-NaN7");
    assert_eq!(d("0E+3").fixed_format(), "0");
    assert_eq!(d("-0E+2").fixed_format(), "-0");
}

#[test]
fn infinities_order_against_finite_values() {
    assert_eq!(d("Infinity").compare(&d("5")), Ok(Ordering::Greater));
    assert_eq!(d("5").compare(&d("Infinity")), Ok(Ordering::Less));
    assert_eq!(d("-Infinity").compare(&d("5")), Ok(Ordering::Less));
    assert_eq!(d("Infinity").equals(&d("Infinity")), Ok(true));
    assert_eq!(d("3").gt(&d("-3")), Ok(true));
}

#[test]
fn malformed_or_unbounded_exponents_are_refused() {
    let long = "1".repeat(26);
    for text in [
        "1E+".to_string(),
        "1Ex".to_string(),
        format!("1E{long}"),
        format!("1E-{long}"),
        // A Unicode space inside the number is not stripped.
        "1\u{2003}0".to_string(),
    ] {
        assert_eq!(
            Decimal::parse(&text).map(|v| v.to_string()),
            Err(Signal::InvalidOperation),
            "{text:?}"
        );
    }
    assert_eq!(s(Decimal::parse("\u{2003}1.5\u{2003}")), Ok("1.5".into()));
}

#[test]
fn results_above_emax_overflow_including_by_a_rounding_carry() {
    assert_eq!(s(d("9E999999").mul(&d("10"))), Err(Signal::Overflow));
    // 29 nines round up to 1E+28 digits, which carries past Etop.
    let nines = format!("{}E999971", "9".repeat(29));
    assert_eq!(s(d(&nines).mul(&d("1"))), Err(Signal::Overflow));
    assert_eq!(s(d("1").div(&d("0"))), Err(Signal::DivisionByZero));
}

#[test]
fn signals_are_named_as_cpython_raises_them() {
    assert_eq!(Signal::InvalidOperation.name(), "InvalidOperation");
    assert_eq!(Signal::DivisionByZero.name(), "DivisionByZero");
    assert_eq!(Signal::Overflow.name(), "Overflow");
}

#[test]
fn exponent_and_adjusted_read_as_python_does() {
    assert_eq!(d("123.45").exponent(), -2);
    assert_eq!(d("123.45").adjusted(), 2);
    assert_eq!(d("Infinity").adjusted(), 0);
}
