//! The subset of Python's `decimal` module the engine uses.
//!
//! Parsing follows CPython's `Decimal(str)` (the C `_decimal` module): exact,
//! with surrounding Python whitespace, underscores and any Unicode decimal
//! digit accepted. Arithmetic ports CPython's reference `_pydecimal`
//! algorithms (`__add__`, `__mul__`, `__truediv__`, `__neg__`, `quantize`,
//! `_fix`, `_rescale`, `_check_nans`) for the default context: 28 digits,
//! ROUND_HALF_EVEN, Emax 999999, Emin -999999, clamp 0 (precision settable per
//! thread, as `localcontext` sets it), and traps on
//! InvalidOperation, DivisionByZero and Overflow.

use std::cmp::Ordering;

use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::Zero;

use crate::pytext;

const EMAX: i128 = 999_999;
const EMIN: i128 = -999_999;

thread_local! {
    /// The context precision: 28 by default, as `decimal.getcontext().prec`.
    static PRECISION: std::cell::Cell<i128> = const { std::cell::Cell::new(28) };
}

/// The current context precision.
pub fn prec() -> i128 {
    PRECISION.with(std::cell::Cell::get)
}

fn etiny() -> i128 {
    EMIN - prec() + 1
}

fn etop() -> i128 {
    EMAX - prec() + 1
}

// Exact conversion runs under libmpdec's maximum context on 64-bit builds.
const MAX_EMAX: i128 = 999_999_999_999_999_999;
const MAX_PREC: i128 = 999_999_999_999_999_999;
const MAX_ETINY: i128 = -MAX_EMAX - (MAX_PREC - 1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    HalfEven,
    HalfUp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Finite,
    Infinite,
    /// Payload digits without leading zeros ("" for none) and whether signalling.
    NaN(String, bool),
}

/// A decimal value: `(-1)**negative * coefficient * 10**exp` for finite values.
#[derive(Clone, Debug)]
pub struct Decimal {
    negative: bool,
    /// Coefficient digits without leading zeros ("0" for zero), as `_pydecimal._int`.
    digits: String,
    exp: i128,
    kind: Kind,
}

/// A signal that the default context traps and Python raises as an exception.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    InvalidOperation,
    DivisionByZero,
    Overflow,
}

impl Signal {
    /// The exception class name CPython raises for the signal.
    pub fn name(self) -> &'static str {
        match self {
            Signal::InvalidOperation => "InvalidOperation",
            Signal::DivisionByZero => "DivisionByZero",
            Signal::Overflow => "Overflow",
        }
    }
}

pub type Calc<T> = Result<T, Signal>;

fn big(digits: &str) -> BigUint {
    BigUint::parse_bytes(digits.as_bytes(), 10).expect("coefficient digits are ASCII")
}

fn ten_pow(n: usize) -> BigUint {
    num_traits::pow(BigUint::from(10u32), n)
}

fn finite(negative: bool, digits: String, exp: i128) -> Decimal {
    let trimmed = digits.trim_start_matches('0');
    let digits = if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    };
    Decimal {
        negative,
        digits,
        exp,
        kind: Kind::Finite,
    }
}

fn infinity(negative: bool) -> Decimal {
    Decimal {
        negative,
        digits: "0".into(),
        exp: 0,
        kind: Kind::Infinite,
    }
}

impl Decimal {
    /// `Decimal(text)` with CPython's exact string conversion.
    pub fn parse(text: &str) -> Calc<Decimal> {
        let stripped = pytext::strip(text);
        let mut ascii = String::with_capacity(stripped.len());
        for ch in stripped.chars() {
            let cp = ch as u32;
            if ch == '_' {
                continue;
            }
            if cp > 0 && cp <= 127 {
                ascii.push(ch);
            } else if pytext::is_space(cp) {
                ascii.push(' ');
            } else if let Some(value) = pytext::decimal_value(cp) {
                ascii.push(char::from_digit(value, 10).expect("digit value below 10"));
            } else {
                return Err(Signal::InvalidOperation);
            }
        }
        parse_ascii(&ascii).ok_or(Signal::InvalidOperation)
    }

    pub fn is_finite(&self) -> bool {
        self.kind == Kind::Finite
    }

    fn is_nan(&self) -> bool {
        matches!(self.kind, Kind::NaN(..))
    }

    fn is_snan(&self) -> bool {
        matches!(self.kind, Kind::NaN(_, true))
    }

    fn is_infinite(&self) -> bool {
        self.kind == Kind::Infinite
    }

    /// Python's truth value is false only for a finite zero.
    pub fn is_zero(&self) -> bool {
        self.is_finite() && self.digits == "0"
    }

    pub fn adjusted(&self) -> i128 {
        if self.is_finite() {
            self.exp + self.digits.len() as i128 - 1
        } else {
            0
        }
    }

    /// `_check_nans`: a signalling NaN raises, a quiet NaN propagates.
    fn check_nans(&self, other: Option<&Decimal>) -> Calc<Option<Decimal>> {
        if self.is_snan() || other.is_some_and(Decimal::is_snan) {
            return Err(Signal::InvalidOperation);
        }
        Ok([Some(self), other].into_iter().flatten().find_map(Decimal::fix_nan))
    }

    /// `_fix_nan`: keep at most `prec` payload digits, dropping the leading
    /// ones; `None` when the value is not a NaN.
    fn fix_nan(&self) -> Option<Decimal> {
        let Kind::NaN(payload, signalling) = &self.kind else {
            return None;
        };
        let mut payload = payload.clone();
        if payload.len() as i128 > prec() {
            payload = payload[payload.len() - prec() as usize..]
                .trim_start_matches('0')
                .to_string();
        }
        Some(Decimal {
            kind: Kind::NaN(payload, *signalling),
            ..self.clone()
        })
    }

    /// `-self`: a zero becomes positive under ROUND_HALF_EVEN.
    pub fn neg(&self) -> Calc<Decimal> {
        if !self.is_finite() {
            if let Some(nan) = self.check_nans(None)? {
                return Ok(nan);
            }
            return Ok(infinity(!self.negative));
        }
        let negative = !self.is_zero() && !self.negative;
        fix(
            Decimal {
                negative,
                ..self.clone()
            },
            Rounding::HalfEven,
        )
    }

    pub fn add(&self, other: &Decimal) -> Calc<Decimal> {
        if !self.is_finite() || !other.is_finite() {
            if let Some(nan) = self.check_nans(Some(other))? {
                return Ok(nan);
            }
            if self.is_infinite() {
                if self.negative != other.negative && other.is_infinite() {
                    return Err(Signal::InvalidOperation);
                }
                return Ok(self.clone());
            }
            return Ok(other.clone());
        }
        add(self, other)
    }

    pub fn sub(&self, other: &Decimal) -> Calc<Decimal> {
        // NaNs are checked before the sign flip, as `__sub__` does.
        if (!self.is_finite() || !other.is_finite())
            && let Some(nan) = self.check_nans(Some(other))?
        {
            return Ok(nan);
        }
        // `__sub__` is `__add__(other.copy_negate())`.
        let negated = Decimal {
            negative: !other.negative,
            ..other.clone()
        };
        self.add(&negated)
    }

    pub fn mul(&self, other: &Decimal) -> Calc<Decimal> {
        let negative = self.negative ^ other.negative;
        if !self.is_finite() || !other.is_finite() {
            if let Some(nan) = self.check_nans(Some(other))? {
                return Ok(nan);
            }
            if (self.is_infinite() && other.is_zero()) || (other.is_infinite() && self.is_zero()) {
                return Err(Signal::InvalidOperation);
            }
            return Ok(infinity(negative));
        }
        let exp = self.exp + other.exp;
        if self.is_zero() || other.is_zero() {
            return fix(finite(negative, "0".into(), exp), Rounding::HalfEven);
        }
        let product = big(&self.digits) * big(&other.digits);
        fix(
            finite(negative, product.to_string(), exp),
            Rounding::HalfEven,
        )
    }

    pub fn div(&self, other: &Decimal) -> Calc<Decimal> {
        let negative = self.negative ^ other.negative;
        if !self.is_finite() || !other.is_finite() {
            if let Some(nan) = self.check_nans(Some(other))? {
                return Ok(nan);
            }
            if self.is_infinite() && other.is_infinite() {
                return Err(Signal::InvalidOperation);
            }
            if self.is_infinite() {
                return Ok(infinity(negative));
            }
            return Ok(finite(negative, "0".into(), etiny()));
        }
        if other.is_zero() {
            return Err(if self.is_zero() {
                Signal::InvalidOperation
            } else {
                Signal::DivisionByZero
            });
        }
        if self.is_zero() {
            return fix(
                finite(negative, "0".into(), self.exp - other.exp),
                Rounding::HalfEven,
            );
        }
        let shift = other.digits.len() as i128 - self.digits.len() as i128 + prec() + 1;
        let mut exp = self.exp - other.exp - shift;
        let (mut coeff, remainder) = if shift >= 0 {
            (big(&self.digits) * ten_pow(shift as usize)).div_rem(&big(&other.digits))
        } else {
            big(&self.digits).div_rem(&(big(&other.digits) * ten_pow((-shift) as usize)))
        };
        if !remainder.is_zero() {
            if (&coeff % 5u32).is_zero() {
                coeff += 1u32;
            }
        } else {
            let ideal = self.exp - other.exp;
            let ten = BigUint::from(10u32);
            while exp < ideal && (&coeff % &ten).is_zero() {
                coeff /= &ten;
                exp += 1;
            }
        }
        fix(finite(negative, coeff.to_string(), exp), Rounding::HalfEven)
    }

    /// `self.quantize(Decimal(f"1E{exp}"), rounding=rounding)`.
    pub fn quantize(&self, exp: i128, rounding: Rounding) -> Calc<Decimal> {
        if !self.is_finite() {
            if let Some(nan) = self.check_nans(None)? {
                return Ok(nan);
            }
            return Err(Signal::InvalidOperation);
        }
        if !(etiny()..=EMAX).contains(&exp) {
            return Err(Signal::InvalidOperation);
        }
        if self.is_zero() {
            return fix(finite(self.negative, "0".into(), exp), rounding);
        }
        if self.adjusted() > EMAX || self.adjusted() - exp + 1 > prec() {
            return Err(Signal::InvalidOperation);
        }
        let answer = rescale(self, exp, rounding);
        if answer.adjusted() > EMAX || answer.digits.len() as i128 > prec() {
            return Err(Signal::InvalidOperation);
        }
        fix(answer, rounding)
    }

    /// `Decimal(n)` for an integer.
    pub fn from_int(n: i64) -> Decimal {
        finite(n < 0, n.unsigned_abs().to_string(), 0)
    }

    /// `str(self)`, Python's to-sci-string with a capital `E`.
    pub fn to_sci_string(&self) -> String {
        let sign = if self.negative { "-" } else { "" };
        match self.kind {
            Kind::Infinite => return format!("{sign}Infinity"),
            Kind::NaN(ref payload, signalling) => {
                let name = if signalling { "sNaN" } else { "NaN" };
                return format!("{sign}{name}{payload}");
            }
            Kind::Finite => {}
        }
        let len = self.digits.len() as i128;
        let leftdigits = self.exp + len;
        let dotplace = if self.exp <= 0 && leftdigits > -6 {
            leftdigits
        } else {
            1
        };
        let (intpart, fracpart) = place_point(&self.digits, dotplace, true);
        let exp = if leftdigits == dotplace {
            String::new()
        } else {
            format!("E{:+}", leftdigits - dotplace)
        };
        format!("{sign}{intpart}{fracpart}{exp}")
    }

    /// `format(self, ",.{places}f")` (grouped) or `format(self, ".{places}f")`.
    /// Callers bound the exponent first, as the Python callers do.
    pub fn fixed(&self, places: u32, grouped: bool) -> String {
        if !self.is_finite() {
            return self.to_sci_string();
        }
        let rounded = rescale(self, -(places as i128), Rounding::HalfEven);
        let sign = if rounded.negative { "-" } else { "" };
        let dotplace = rounded.exp + rounded.digits.len() as i128;
        let (whole, frac) = place_point(&rounded.digits, dotplace, false);
        if !grouped {
            return format!("{sign}{whole}{frac}");
        }
        let mut out = String::from(sign);
        for (i, ch) in whole.chars().enumerate() {
            if i > 0 && (whole.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(ch);
        }
        out.push_str(&frac);
        out
    }

    /// `<`, `<=`, `>`, `>=`: any NaN raises InvalidOperation.
    pub fn compare(&self, other: &Decimal) -> Calc<Ordering> {
        if self.is_nan() || other.is_nan() {
            return Err(Signal::InvalidOperation);
        }
        Ok(self.order(other))
    }

    /// `==`: a signalling NaN raises, a quiet NaN equals nothing.
    pub fn equals(&self, other: &Decimal) -> Calc<bool> {
        if self.is_snan() || other.is_snan() {
            return Err(Signal::InvalidOperation);
        }
        if self.is_nan() || other.is_nan() {
            return Ok(false);
        }
        Ok(self.order(other) == Ordering::Equal)
    }

    pub fn lt(&self, other: &Decimal) -> Calc<bool> {
        Ok(self.compare(other)? == Ordering::Less)
    }

    pub fn le(&self, other: &Decimal) -> Calc<bool> {
        Ok(self.compare(other)? != Ordering::Greater)
    }

    pub fn gt(&self, other: &Decimal) -> Calc<bool> {
        Ok(self.compare(other)? == Ordering::Greater)
    }

    /// Ordering of two non-NaN values, infinities included.
    fn order(&self, other: &Decimal) -> Ordering {
        let rank = |d: &Decimal| match (d.is_infinite(), d.negative) {
            (true, true) => -1,
            (true, false) => 1,
            _ => 0,
        };
        match rank(self).cmp(&rank(other)) {
            Ordering::Equal if rank(self) != 0 => return Ordering::Equal,
            Ordering::Equal => {}
            ordering => return ordering,
        }
        let sign = |d: &Decimal| {
            if d.negative {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        };
        match (self.is_zero(), other.is_zero()) {
            (true, true) => return Ordering::Equal,
            (true, false) => return sign(other).reverse(),
            (false, true) => return sign(self),
            _ => {}
        }
        if self.negative != other.negative {
            return if self.negative {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
        let magnitude = match self.adjusted().cmp(&other.adjusted()) {
            Ordering::Equal => {
                let pad = |d: &Decimal, e: i128| {
                    d.digits.clone() + &"0".repeat((d.exp - e).max(0) as usize)
                };
                let low = self.exp.min(other.exp);
                pad(self, low).cmp(&pad(other, low))
            }
            ordering => ordering,
        };
        if self.negative {
            magnitude.reverse()
        } else {
            magnitude
        }
    }
}

fn place_point(digits: &str, dotplace: i128, sci: bool) -> (String, String) {
    let len = digits.len() as i128;
    if dotplace <= 0 && (sci || dotplace < 0) {
        (
            "0".into(),
            format!(".{}{digits}", "0".repeat((-dotplace) as usize)),
        )
    } else if dotplace >= len {
        (
            format!("{digits}{}", "0".repeat((dotplace - len) as usize)),
            String::new(),
        )
    } else {
        let (int, frac) = digits.split_at(dotplace as usize);
        let int = if int.is_empty() { "0" } else { int };
        (int.to_string(), format!(".{frac}"))
    }
}

/// libmpdec's string grammar after CPython has stripped and converted the text.
fn parse_ascii(text: &str) -> Option<Decimal> {
    let (negative, body) = match text.as_bytes().first()? {
        b'-' => (true, &text[1..]),
        b'+' => (false, &text[1..]),
        _ => (false, text),
    };
    if let Some(special) = special_value(negative, &body.to_ascii_lowercase()) {
        return special;
    }
    let (mantissa, exponent) = match body.find(['e', 'E']) {
        Some(at) => (&body[..at], Some(&body[at + 1..])),
        None => (body, None),
    };
    let (int, frac) = match mantissa.find('.') {
        Some(at) => (&mantissa[..at], &mantissa[at + 1..]),
        None => (mantissa, ""),
    };
    let all_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if int.len() + frac.len() == 0 || !all_digits(int) || !all_digits(frac) {
        return None;
    }
    let exp = match exponent {
        Some(exponent) => exponent_value(exponent)?,
        None => 0,
    } - frac.len() as i128;
    let value = finite(negative, format!("{int}{frac}"), exp);
    // Exact conversion: any rounding or clamping under the maximum context is InvalidOperation.
    if value.is_zero() {
        if !(MAX_ETINY..=MAX_EMAX).contains(&value.exp) {
            return None;
        }
    } else if value.adjusted() > MAX_EMAX || value.exp < MAX_ETINY {
        return None;
    }
    Some(value)
}

/// Infinity and the NaNs: `Some(None)` for a NaN with a malformed payload,
/// `None` when the text is not a special value at all.
fn special_value(negative: bool, lower: &str) -> Option<Option<Decimal>> {
    let special = |kind| Decimal {
        negative,
        digits: "0".into(),
        exp: 0,
        kind,
    };
    if lower == "inf" || lower == "infinity" {
        return Some(Some(special(Kind::Infinite)));
    }
    ["snan", "nan"].iter().find_map(|prefix| {
        let payload = lower.strip_prefix(prefix)?;
        Some(payload.bytes().all(|b| b.is_ascii_digit()).then(|| {
            special(Kind::NaN(
                payload.trim_start_matches('0').to_string(),
                *prefix == "snan",
            ))
        }))
    })
}

/// The exponent after `e`, saturated like libmpdec's strtoexp; anything this
/// large fails the bounds check that follows.
fn exponent_value(exponent: &str) -> Option<i128> {
    let (negative, digits) = match exponent.as_bytes().first()? {
        b'-' => (true, &exponent[1..]),
        b'+' => (false, &exponent[1..]),
        _ => (false, exponent),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let significant = digits.trim_start_matches('0');
    let value = if significant.len() > 25 {
        i128::MAX / 4
    } else {
        significant.parse::<i128>().unwrap_or(0)
    };
    Some(if negative { -value } else { value })
}

fn all_zeros(digits: &str, from: usize) -> bool {
    digits
        .as_bytes()
        .get(from..)
        .is_none_or(|rest| rest.iter().all(|&b| b == b'0'))
}

fn exact_half(digits: &str, at: usize) -> bool {
    digits.as_bytes().get(at) == Some(&b'5') && all_zeros(digits, at + 1)
}

/// `_pydecimal`'s rounding functions: 1 increments, -1 truncates inexactly, 0 is exact.
fn round_change(digits: &str, keep: usize, rounding: Rounding) -> i8 {
    let half_up = || {
        if matches!(digits.as_bytes().get(keep), Some(b'5'..=b'9')) {
            1
        } else if all_zeros(digits, keep) {
            0
        } else {
            -1
        }
    };
    match rounding {
        Rounding::HalfUp => half_up(),
        Rounding::HalfEven => {
            let even = keep == 0
                || matches!(
                    digits.as_bytes()[keep - 1],
                    b'0' | b'2' | b'4' | b'6' | b'8'
                );
            if exact_half(digits, keep) && even {
                -1
            } else {
                half_up()
            }
        }
    }
}

fn rescale(value: &Decimal, exp: i128, rounding: Rounding) -> Decimal {
    if value.is_zero() {
        return finite(value.negative, "0".into(), exp);
    }
    if value.exp >= exp {
        let padded = value.digits.clone() + &"0".repeat((value.exp - exp) as usize);
        return finite(value.negative, padded, exp);
    }
    let mut source = value.clone();
    let mut keep = value.digits.len() as i128 + value.exp - exp;
    if keep < 0 {
        source = finite(value.negative, "1".into(), exp - 1);
        keep = 0;
    }
    let keep = keep as usize;
    let changed = round_change(&source.digits, keep, rounding);
    let mut coeff = source.digits[..keep].to_string();
    if coeff.is_empty() {
        coeff = "0".into();
    }
    if changed == 1 {
        coeff = (big(&coeff) + 1u32).to_string();
    }
    finite(value.negative, coeff, exp)
}

/// `_pydecimal.Decimal._fix`: round to the context, clamp zero exponents and
/// raise Overflow above Emax.
fn fix(value: Decimal, rounding: Rounding) -> Calc<Decimal> {
    if value.is_zero() {
        let new_exp = value.exp.clamp(etiny(), EMAX);
        return Ok(finite(value.negative, "0".into(), new_exp));
    }
    let mut exp_min = value.digits.len() as i128 + value.exp - prec();
    if exp_min > etop() {
        return Err(Signal::Overflow);
    }
    exp_min = exp_min.max(etiny());
    if value.exp >= exp_min {
        return Ok(value);
    }
    let mut source = value.clone();
    let mut keep = value.digits.len() as i128 + value.exp - exp_min;
    if keep < 0 {
        source = finite(value.negative, "1".into(), exp_min - 1);
        keep = 0;
    }
    let keep = keep as usize;
    let changed = round_change(&source.digits, keep, rounding);
    let mut coeff = source.digits[..keep].to_string();
    if coeff.is_empty() {
        coeff = "0".into();
    }
    if changed > 0 {
        coeff = (big(&coeff) + 1u32).to_string();
        if coeff.len() as i128 > prec() {
            coeff.pop();
            exp_min += 1;
        }
    }
    if exp_min > etop() {
        return Err(Signal::Overflow);
    }
    Ok(finite(value.negative, coeff, exp_min))
}

fn add(a: &Decimal, b: &Decimal) -> Calc<Decimal> {
    let exp = a.exp.min(b.exp);
    if a.is_zero() && b.is_zero() {
        return fix(
            finite(a.negative && b.negative, "0".into(), exp),
            Rounding::HalfEven,
        );
    }
    if a.is_zero() {
        let exp = exp.max(b.exp - prec() - 1);
        return fix(rescale(b, exp, Rounding::HalfEven), Rounding::HalfEven);
    }
    if b.is_zero() {
        let exp = exp.max(a.exp - prec() - 1);
        return fix(rescale(a, exp, Rounding::HalfEven), Rounding::HalfEven);
    }
    // _normalize: a tiny operand is replaced by 10**exp, which rounds identically.
    let (mut tmp, mut other) = if a.exp < b.exp {
        (b.clone(), a.clone())
    } else {
        (a.clone(), b.clone())
    };
    let floor = tmp.exp + (-1i128).min(tmp.digits.len() as i128 - prec() - 2);
    if other.digits.len() as i128 + other.exp - 1 < floor {
        other = finite(other.negative, "1".into(), floor);
    }
    let tmp_int = big(&tmp.digits) * ten_pow((tmp.exp - other.exp) as usize);
    tmp.exp = other.exp;
    let other_int = big(&other.digits);
    let (big_int, small_int, big_negative) = if tmp.negative != other.negative {
        match tmp_int.cmp(&other_int) {
            Ordering::Equal => return fix(finite(false, "0".into(), exp), Rounding::HalfEven),
            Ordering::Less => (other_int, tmp_int, other.negative),
            Ordering::Greater => (tmp_int, other_int, tmp.negative),
        }
    } else {
        (tmp_int, other_int, tmp.negative)
    };
    let result = if tmp.negative != other.negative {
        big_int - small_int
    } else {
        big_int + small_int
    };
    fix(
        finite(big_negative, result.to_string(), tmp.exp),
        Rounding::HalfEven,
    )
}

impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_sci_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parses_like_cpython() {
        assert_eq!(d("1_000").to_sci_string(), "1000");
        assert_eq!(d("\u{1c}1.5\u{1f}").to_sci_string(), "1.5");
        assert_eq!(d("\u{663}\u{664}").to_sci_string(), "34");
        assert_eq!(d("1E+2").to_sci_string(), "1E+2");
        assert_eq!(d("-0.00").to_sci_string(), "-0.00");
        assert_eq!(d("+.5").to_sci_string(), "0.5");
        assert_eq!(d("1.").to_sci_string(), "1");
        assert_eq!(d("-sNaN005").to_sci_string(), "-sNaN5");
        assert_eq!(d("nan0").to_sci_string(), "NaN");
        assert_eq!(d("-inf").to_sci_string(), "-Infinity");
        for bad in [
            "1 0",
            ".",
            "-",
            "1e",
            "infinit",
            "1,5",
            "1\u{0}",
            "1e9223372036854775807",
            "0e99999999999999999999",
        ] {
            assert!(Decimal::parse(bad).is_err(), "{bad:?}");
        }
        assert_eq!(
            d("1e999999999999999999").adjusted(),
            999_999_999_999_999_999
        );
    }

    #[test]
    fn arithmetic_matches_cpython_probes() {
        let z = d("0E+999999999999999999");
        assert_eq!(s(z.add(&z)), ok("0E+999999"));
        assert_eq!(s(z.add(&d("1.00"))), ok("1.00"));
        let tiny = d("0E-999999999999999999");
        assert_eq!(s(tiny.add(&tiny)), ok("0E-1000026"));
        assert_eq!(s(d("0E-999999999").div(&d("100"))), ok("0E-1000026"));
        assert_eq!(s(d("-0.00").sub(&d("0.00"))), ok("-0.00"));
        assert_eq!(s(d("-0").add(&d("-0"))), ok("-0"));
        assert_eq!(s(d("1.00").sub(&d("1.00"))), ok("0.00"));
        assert_eq!(s(d("10").div(&d("4"))), ok("2.5"));
        assert_eq!(s(d("1").div(&d("3"))), ok("0.3333333333333333333333333333"));
        assert_eq!(s(d("100.00").div(&d("4"))), ok("25.00"));
        assert_eq!(s(d("12E+3").div(&d("4E-2"))), ok("3E+5"));
        assert_eq!(
            s(d("1E+18").sub(&d("1E-18"))),
            ok("1000000000000000000.000000000")
        );
        assert_eq!(
            s(d("9999999999999999999999999999.5").add(&d("0"))),
            ok("1.000000000000000000000000000E+28")
        );
        let percent = d("2").div(&d("3")).unwrap().mul(&d("100")).unwrap();
        assert_eq!(s(percent.quantize(-4, Rounding::HalfUp)), ok("66.6667"));
        assert_eq!(
            d("1E+25").quantize(-4, Rounding::HalfUp).map(|_| ()),
            Err(Signal::InvalidOperation)
        );
    }

    #[test]
    fn special_values_match_cpython_probes() {
        fn invalid<T>() -> Calc<T> {
            Err(Signal::InvalidOperation)
        }
        assert_eq!(s(d("0.00").neg()), ok("0.00"));
        assert_eq!(s(d("-0.00").neg()), ok("0.00"));
        assert_eq!(s(d("NaN5").neg()), ok("NaN5"));
        assert_eq!(s(d("sNaN").neg()), invalid());
        assert_eq!(s(d("Infinity").neg()), ok("-Infinity"));
        assert_eq!(s(d("1").div(&d("0"))), Err(Signal::DivisionByZero));
        assert_eq!(s(d("0").div(&d("0"))), invalid());
        assert_eq!(s(d("1").div(&d("Infinity"))), ok("0E-1000026"));
        assert_eq!(s(d("Infinity").mul(&d("0"))), invalid());
        assert_eq!(s(d("NaN").quantize(-2, Rounding::HalfUp)), ok("NaN"));
        assert_eq!(s(d("Infinity").quantize(-2, Rounding::HalfUp)), invalid());
        assert_eq!(
            s(d("9E+999999").add(&d("9E+999999"))),
            Err(Signal::Overflow)
        );
        assert_eq!(s(d("1E-999999999").add(&d("0"))), ok("0E-1000026"));
        let long_nan = d(&format!("NaN{}", "1".repeat(40)));
        assert_eq!(
            s(long_nan.add(&d("1"))),
            Ok(format!("NaN{}", "1".repeat(28)))
        );
        assert_eq!(d("sNaN").equals(&d("1")), invalid());
        assert_eq!(d("NaN").equals(&d("1")), Ok(false));
        assert_eq!(d("NaN").lt(&d("1")), invalid());
        assert_eq!(d("-Infinity").lt(&d("-1E+999999999")), Ok(true));
        assert_eq!(d("Infinity").fixed(2, true), "Infinity");
    }

    #[test]
    fn fixed_format_matches_cpython() {
        assert_eq!(d("1E+3").fixed(0, false), "1000");
        assert_eq!(d("-0.00").fixed(2, true), "-0.00");
        assert_eq!(d("1234567.005").fixed(2, true), "1,234,567.00");
        assert_eq!(d("1234567.015").fixed(2, true), "1,234,567.02");
        assert_eq!(d("1E-999999999").fixed(2, true), "0.00");
        assert_eq!(d("0E+3").fixed(2, true), "0.00");
        assert_eq!(d("-0.001").fixed(2, true), "-0.00");
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
        assert_eq!(s(long.div(&d("2"))), ok("6.172839450617283945061728395E+38"));
    }

    #[test]
    fn quantize_refuses_exponents_and_results_outside_the_context() {
        let invalid = || -> Calc<String> { Err(Signal::InvalidOperation) };
        assert_eq!(s(d("1").quantize(EMAX + 1, Rounding::HalfUp)), invalid());
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
        assert_eq!(Decimal::parse(&huge).map(|_| ()), Err(Signal::InvalidOperation));
        assert_eq!(d("\u{2003}1.5\u{2003}").to_sci_string(), "1.5");
    }

    #[test]
    fn display_is_the_sci_string() {
        assert_eq!(format!("{}", d("1.50")), "1.50");
        assert_eq!(d("-1E+3").to_string(), "-1E+3");
    }
}
