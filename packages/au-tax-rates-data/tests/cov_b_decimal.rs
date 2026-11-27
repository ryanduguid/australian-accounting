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
