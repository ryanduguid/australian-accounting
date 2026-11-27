//! `Decimal`'s special values (NaN, sNaN, infinities), float conversion and conversion errors
//! against CPython 3.15's `decimal` in the default context. Expected values are what
//! `tests/data/cov_b_decimal.py` prints.
use au_tax_rates_data::decimal::{Calc, Conversion, Decimal, Rounding, Signal};

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn s(r: Calc<Decimal>) -> Calc<String> {
    r.map(|v| v.to_sci_string())
}

fn ok(text: &str) -> Calc<String> {
    Ok(text.into())
}

const INVALID: Calc<String> = Err(Signal::InvalidOperation);

#[test]
fn addition_propagates_nans_and_refuses_opposite_infinities() {
    assert_eq!(s(d("NaN").add(&d("1"))), ok("NaN"));
    assert_eq!(s(d("1").add(&d("-NaN7"))), ok("-NaN7"));
    assert_eq!(s(d("sNaN").add(&d("1"))), INVALID);
    assert_eq!(s(d("1").add(&d("sNaN"))), INVALID);
    assert_eq!(s(d("Infinity").add(&d("-Infinity"))), INVALID);
    assert_eq!(s(d("Infinity").add(&d("Infinity"))), ok("Infinity"));
    assert_eq!(s(d("-Infinity").add(&d("5"))), ok("-Infinity"));
    assert_eq!(s(d("5").add(&d("-Infinity"))), ok("-Infinity"));
    // Equal magnitudes of opposite sign cancel to a positive zero at the smaller exponent.
    assert_eq!(s(d("2.50").add(&d("-2.5"))), ok("0.00"));
}

#[test]
fn a_nan_payload_keeps_its_last_28_digits_without_leading_zeros() {
    let nan = d(&format!("NaN1{}{}", "0".repeat(5), "9".repeat(25)));
    assert_eq!(s(nan.add(&d("1"))), ok(&format!("NaN{}", "9".repeat(25))));
}

#[test]
fn multiplication_and_division_of_special_values() {
    assert_eq!(s(d("NaN").mul(&d("2"))), ok("NaN"));
    assert_eq!(s(d("Infinity").mul(&d("0"))), INVALID);
    assert_eq!(s(d("0").mul(&d("-Infinity"))), INVALID);
    assert_eq!(s(d("-Infinity").mul(&d("2"))), ok("-Infinity"));
    assert_eq!(s(d("3").mul(&d("-Infinity"))), ok("-Infinity"));
    assert_eq!(s(d("NaN").div(&d("2"))), ok("NaN"));
    assert_eq!(s(d("Infinity").div(&d("-Infinity"))), INVALID);
    assert_eq!(s(d("-Infinity").div(&d("2"))), ok("-Infinity"));
    // A finite value over an infinity is a zero at Etiny, signed by the operands.
    assert_eq!(s(d("2").div(&d("-Infinity"))), ok("-0E-1000026"));
}

#[test]
fn normalize_and_integral_value_keep_special_values() {
    assert_eq!(s(d("NaN12").normalize()), ok("NaN12"));
    assert_eq!(s(d("sNaN").normalize()), INVALID);
    assert_eq!(s(d("-Infinity").normalize()), ok("-Infinity"));
    assert_eq!(s(d("-0.000").normalize()), ok("-0"));
    assert_eq!(s(d("NaN").to_integral_value()), ok("NaN"));
    assert_eq!(s(d("Infinity").to_integral_value()), ok("Infinity"));
    assert_eq!(
        s(d("-Infinity").to_integral_rounded(Rounding::Down)),
        ok("-Infinity")
    );
    assert_eq!(d("Infinity").adjusted(), 0);
    assert_eq!(d("-Infinity").fixed_format(), "-Infinity");
    assert_eq!(d("NaN").fixed_format(), "NaN");
}

#[test]
fn floats_convert_to_their_exact_binary_value() {
    let f = |x: f64| Decimal::from_float(x).to_sci_string();
    // CPython always makes a positive NaN from a float, whatever its sign bit.
    assert_eq!(f(f64::NAN), "NaN");
    assert_eq!(f(-f64::NAN), "NaN");
    assert_eq!(f(f64::INFINITY), "Infinity");
    assert_eq!(f(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(f(-0.0), "-0");
    assert_eq!(
        f(0.1),
        "0.1000000000000000055511151231257827021181583404541015625"
    );
    assert_eq!(f(3.0), "3");
    assert_eq!(f(2f64.powi(60)), "1152921504606846976");
    // The smallest subnormal, 2**-1074.
    assert_eq!(
        f(5e-324),
        "4.940656458412465441765687928682213723650598026143247644255856825006755072702087518652998363616359923797965646954457177309266567103559397963987747960107818781263007131903114045278458171678489821036887186360569987307230500063874091535649843873124733972731696151400317153853980741262385655911710266585566867681870395603106249319452715914924553293054565444011274801297099995419319894090804165633245247571478690147267801593552386115501348035264934720193790268107107491703332226844753335720832431936092382893458368060106011506169809753078342277318329247904982524730776375927247874656084778203734469699533647017972677717585125660551199131504891101451037862738167250955837389733598993664809941164205702637090279242767544565229087538682506419718265533447265625E-324"
    );
}

#[test]
fn conversion_errors_print_as_python_prints_them() {
    // Decimal("1E999999999999999999999"): InvalidOperation [<class 'decimal.InvalidOperation'>].
    let range = Decimal::parse_exact("1E999999999999999999999").unwrap_err();
    assert_eq!(range.text(), "[<class 'decimal.InvalidOperation'>]");
    // Decimal("x"): InvalidOperation [<class 'decimal.ConversionSyntax'>].
    assert_eq!(
        Decimal::parse_exact("x").unwrap_err().text(),
        Conversion::Syntax.text()
    );
    assert_eq!(
        Conversion::Syntax.text(),
        "[<class 'decimal.ConversionSyntax'>]"
    );
    // Decimal((1, (4,), 'F')), which libmpdec reads as "-Infinity".
    assert_eq!(
        Decimal::from_mpd_string("-Infinity").map(|v| v.to_sci_string()),
        Ok("-Infinity".to_string())
    );
}

#[test]
fn finite_arithmetic_edges_match_python() {
    // Unicode whitespace inside the number becomes a space, which is bad syntax.
    assert_eq!(
        Decimal::parse_exact("1\u{2003}0").unwrap_err(),
        Conversion::Syntax
    );
    assert_eq!(Decimal::parse_exact("1E").unwrap_err(), Conversion::Syntax);
    // An exponent beyond libmpdec's limits; leading zeros do not count towards its length.
    let long = format!("1E{}", "1".repeat(30));
    assert_eq!(Decimal::parse_exact(&long).unwrap_err(), Conversion::Range);
    assert_eq!(
        s(Decimal::parse(&format!("1E{}5", "0".repeat(30)))),
        ok("1E+5")
    );
    assert_eq!(
        s(d("1E-40").div(&d("3"))),
        ok("3.333333333333333333333333333E-41")
    );
    assert_eq!(
        s(d("2").div(&d("3E+40"))),
        ok("6.666666666666666666666666667E-41")
    );
    // 1/7's 29th digit is a 5 with a remainder: _pydecimal bumps it before rounding.
    assert_eq!(s(d("1").div(&d("7"))), ok("0.1428571428571428571428571429"));
    assert_eq!(s(d("1").add(&d("-2.5"))), ok("-1.5"));
    assert_eq!(s(d("-1").add(&d("2.5"))), ok("1.5"));
    assert_eq!(s(d("9E+999999").mul(&d("10"))), Err(Signal::Overflow));
    assert_eq!(s(d("1E+2").to_integral_value()), ok("1E+2"));
    assert_eq!(s(d("2.000").to_integral_value()), ok("2"));
    assert_eq!(s(d("2.00").to_integral_rounded(Rounding::Down)), ok("2"));
    assert_eq!(s(d("2.50").to_integral_rounded(Rounding::Down)), ok("2"));
    assert_eq!(d("0E+3").fixed_format(), "0");
    assert_eq!(d("-0E+3").fixed_format(), "-0");
}

#[test]
fn comparison_orders_infinities_and_signs() {
    use std::cmp::Ordering::{Greater, Less};
    for (a, b, expected) in [
        ("-Infinity", "Infinity", Less),
        ("Infinity", "5", Greater),
        ("-1", "2", Less),
        ("1", "-2", Greater),
        ("-1", "-2", Greater),
        ("-2", "-1", Less),
    ] {
        assert_eq!(d(a).compare(&d(b)), Ok(expected), "{a} vs {b}");
    }
}

#[test]
fn an_exponent_with_no_digits_or_a_stray_character_is_a_syntax_error() {
    for text in ["1E+", "1E1x"] {
        assert_eq!(
            Decimal::parse_exact(text).unwrap_err(),
            Conversion::Syntax,
            "{text}"
        );
    }
}

#[test]
fn a_dividend_longer_than_the_precision_is_rounded_to_28_digits() {
    let ones = d(&"1".repeat(40));
    assert_eq!(
        s(ones.div(&d("1"))),
        ok("1.111111111111111111111111111E+39")
    );
    assert_eq!(
        s(ones.div(&d("3"))),
        ok("3.703703703703703703703703704E+38")
    );
}

#[test]
fn rounding_that_carries_past_emax_overflows() {
    let nines = |exp: &str| d(&format!("{}E+{exp}", "9".repeat(29)));
    assert_eq!(s(nines("999971").add(&d("0"))), Err(Signal::Overflow));
    assert_eq!(
        s(nines("999970").add(&d("0"))),
        ok("1.000000000000000000000000000E+999999")
    );
}
