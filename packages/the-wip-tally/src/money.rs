//! Amount parsing and formatting. Everything downstream works in Decimal; no
//! amount is ever converted to binary floating point.

use crate::decimal::{Calc, Decimal, Rounding, Signal};
use crate::pytext;

/// Why a cell could not be read: Python's `AmountError` message, or a trapped
/// decimal signal, which the original raised uncaught.
#[derive(Debug, PartialEq, Eq)]
pub enum AmountError {
    Message(String),
    Decimal(Signal),
}

impl From<Signal> for AmountError {
    fn from(signal: Signal) -> Self {
        AmountError::Decimal(signal)
    }
}

pub type Result<T> = std::result::Result<T, AmountError>;

fn refuse<T>(message: String) -> Result<T> {
    Err(AmountError::Message(message))
}

pub fn d(text: &str) -> Decimal {
    Decimal::parse(text).expect("decimal literal")
}

fn is_ascii_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_space(c: char) -> bool {
    pytext::is_space(c as u32)
}

/// `_NUMBER`: `(?:[0-9]+|[0-9]{1,3}(?:,[0-9]{3})+)(?:\.[0-9]+)?|\.[0-9]+`.
fn is_number(s: &str) -> bool {
    let (whole, frac) = match s.find('.') {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };
    if let Some(frac) = frac
        && (frac.is_empty() || !frac.chars().all(is_ascii_digit))
    {
        return false;
    }
    if whole.is_empty() {
        return frac.is_some();
    }
    if whole.chars().all(is_ascii_digit) {
        return true;
    }
    let mut groups = whole.split(',');
    let first = groups.next().unwrap_or("");
    !first.is_empty()
        && first.len() <= 3
        && first.chars().all(is_ascii_digit)
        && groups.all(|g| g.len() == 3 && g.chars().all(is_ascii_digit))
}

fn trim_start_space(s: &str) -> &str {
    s.trim_start_matches(is_space)
}

fn trim_end_space(s: &str) -> &str {
    s.trim_end_matches(is_space)
}

fn strip_ci_suffix<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
    let n = s.len().checked_sub(suffix.len())?;
    (s.is_char_boundary(n) && s[n..].eq_ignore_ascii_case(suffix)).then(|| &s[..n])
}

/// `_ACCOUNTING_NUMBER.fullmatch(text)` (re.IGNORECASE, Unicode `\s`).
fn is_accounting_number(text: &str) -> bool {
    // [+-]?\$?\s*NUMBER
    let a = {
        let s = text.strip_prefix(['+', '-']).unwrap_or(text);
        let s = s.strip_prefix('$').unwrap_or(s);
        is_number(trim_start_space(s))
    };
    // \$[+-]\s*NUMBER
    let b = text
        .strip_prefix('$')
        .and_then(|s| s.strip_prefix(['+', '-']))
        .is_some_and(|s| is_number(trim_start_space(s)));
    // \$?\s*\(\s*NUMBER\s*\)
    let c = {
        let s = text.strip_prefix('$').unwrap_or(text);
        trim_start_space(s)
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .is_some_and(|s| is_number(trim_end_space(trim_start_space(s))))
    };
    // \(\s*\$?NUMBER\s*\)
    let d = text
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .is_some_and(|s| {
            let s = trim_end_space(trim_start_space(s));
            is_number(s.strip_prefix('$').unwrap_or(s))
        });
    // \$?\s*NUMBER\s*(?:CR|DR)
    let e = strip_ci_suffix(text, "CR")
        .or_else(|| strip_ci_suffix(text, "DR"))
        .is_some_and(|s| {
            let s = s.strip_prefix('$').unwrap_or(s);
            is_number(trim_end_space(trim_start_space(s)))
        });
    a || b || c || d || e
}

/// `parse_amount`: an accounting amount as an unquantised Decimal.
pub fn parse_amount(raw: &str, where_: &str) -> Result<Decimal> {
    let text = pytext::strip(raw);
    if text.is_empty() {
        return refuse(format!("{where_}: no amount given"));
    }
    if !is_accounting_number(text) {
        return refuse(format!("{where_}: {} is not an amount", pytext::repr(raw)));
    }
    let negative = text.contains('(') || pytext_upper_ends_cr(text);
    // re.sub(r"[\s$,()]|CR$|DR$", "", text, flags=re.IGNORECASE)
    let body = strip_ci_suffix(text, "CR")
        .or_else(|| strip_ci_suffix(text, "DR"))
        .unwrap_or(text);
    let cleaned: String = body
        .chars()
        .filter(|&c| !is_space(c) && !matches!(c, '$' | ',' | '(' | ')'))
        .collect();
    let value = Decimal::parse(&cleaned).expect("the grammar admits only finite decimals");
    Ok(if negative { value.neg()? } else { value })
}

/// `text.upper().endswith("CR")`: only an ASCII c and r can produce it.
fn pytext_upper_ends_cr(text: &str) -> bool {
    strip_ci_suffix(text, "CR").is_some()
}

/// `as_money`: cents, half up, with a signed zero collapsed to 0.00.
pub fn as_money(value: &Decimal) -> Calc<Decimal> {
    let quantised = value.quantize(-2, Rounding::HalfUp)?;
    Ok(if quantised.equals(&d("0"))? {
        d("0.00")
    } else {
        quantised
    })
}

/// `parse_money`: an amount quantised to cents; a non-zero sub-half-cent value is refused.
pub fn parse_money(raw: &str, where_: &str) -> Result<Decimal> {
    let out_of_range = || {
        AmountError::Message(format!(
            "{where_}: amount is outside the supported range for cents"
        ))
    };
    let value = parse_amount(raw, where_).map_err(|error| match error {
        AmountError::Decimal(_) => out_of_range(),
        other => other,
    })?;
    let quantised = as_money(&value).map_err(|_| out_of_range())?;
    if !value.equals(&d("0"))? && quantised.equals(&d("0"))? {
        return refuse(format!(
            "{where_}: {} is smaller than half a cent and would round to nil",
            pytext::repr(raw)
        ));
    }
    Ok(quantised)
}

/// `_PERCENT_RE.fullmatch`: the number before a `%`, with Unicode `\d`.
fn percent_group(text: &str) -> Option<&str> {
    let body = trim_end_space(text.strip_suffix('%')?);
    let digits =
        |s: &str| !s.is_empty() && s.chars().all(|c| pytext::decimal_value(c as u32).is_some());
    let unsigned = body.strip_prefix(['+', '-']).unwrap_or(body);
    let ok = match unsigned.split_once('.') {
        Some((whole, frac)) => (whole.is_empty() || digits(whole)) && digits(frac),
        None => digits(unsigned),
    };
    ok.then_some(body)
}

/// `parse_ratio`: 0 to 1, or a percentage with a `%` suffix.
pub fn parse_ratio(raw: &str, where_: &str) -> Result<Decimal> {
    let text = pytext::strip(raw);
    if text.is_empty() {
        return refuse(format!("{where_}: no ratio given"));
    }
    let value = match percent_group(text) {
        Some(group) => parse_amount(group, where_)?.div(&d("100"))?,
        None => {
            let value = parse_amount(text, where_)?;
            if value.gt(&d("1"))? {
                return refuse(format!(
                    "{where_}: {} is greater than 1. Write it as a percentage \
                     with a % suffix, or as a ratio between 0 and 1.",
                    pytext::repr(raw)
                ));
            }
            value
        }
    };
    if value.lt(&d("0"))? || value.gt(&d("1"))? {
        return refuse(format!(
            "{where_}: {} is not between 0 and 1",
            pytext::repr(raw)
        ));
    }
    Ok(value)
}

/// `parse_bool`: yes/no words after `strip().casefold()`.
pub fn parse_bool(raw: &str, where_: &str) -> Result<bool> {
    let text = crate::py::casefold(pytext::strip(raw));
    match text.as_str() {
        "yes" | "y" | "true" | "1" => Ok(true),
        "no" | "n" | "false" | "0" => Ok(false),
        _ => refuse(format!("{where_}: {} is not yes or no", pytext::repr(raw))),
    }
}

/// `money`: thousands separators and two decimals.
pub fn money(value: &Decimal) -> Calc<String> {
    Ok(as_money(value)?.fixed(2, true))
}

/// `percent`: a ratio as a percentage to 2 places.
pub fn percent(value: &Decimal) -> Calc<String> {
    let scaled = value.mul(&d("100"))?.quantize(-2, Rounding::HalfUp)?;
    Ok(format!("{scaled}%"))
}

/// `as_points`: a margin movement at the 2 places it is displayed at.
pub fn as_points(value: &Decimal) -> Calc<Decimal> {
    let quantised = value.quantize(-2, Rounding::HalfUp)?;
    Ok(if quantised.equals(&d("0"))? {
        d("0.00")
    } else {
        quantised
    })
}

pub fn points(value: &Decimal) -> Calc<String> {
    Ok(format!("{} points", as_points(value)?))
}
