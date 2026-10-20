//! Operator-supplied facts, including the ones they cannot establish.
//! "unknown" is a value, carried through as UNKNOWN and never coerced to false.

use crate::decimal::Decimal;
use crate::money::{parse_money, parse_rate, parse_ratio};
use crate::pytext;
use crate::raw::Raw;
use crate::years::{YearOfIncome, parse_year};

/// Spellings of "I do not know"; a blank cell counts.
pub const UNKNOWN_TOKENS: [&str; 7] = ["", "unknown", "unk", "n/a", "na", "?", "-"];
const TRUE: [&str; 5] = ["true", "t", "yes", "y", "1"];
const FALSE: [&str; 5] = ["false", "f", "no", "n", "0"];

/// `str(raw).strip().lower()`.
fn token(raw: &Raw) -> String {
    pytext::strip(&raw.py_str()).to_lowercase()
}

pub fn is_unknown(raw: Option<&Raw>) -> bool {
    match raw {
        None | Some(Raw::None) => true,
        Some(raw) => UNKNOWN_TOKENS.contains(&token(raw).as_str()),
    }
}

/// true / false / unknown; anything unrecognised is an error, never a silent false.
pub fn parse_tristate(raw: Option<&Raw>, where_: &str) -> Result<Option<bool>, String> {
    let Some(raw) = raw.filter(|_| !is_unknown(raw)) else {
        return Ok(None);
    };
    let text = token(raw);
    if TRUE.contains(&text.as_str()) {
        return Ok(Some(true));
    }
    if FALSE.contains(&text.as_str()) {
        return Ok(Some(false));
    }
    Err(format!(
        "{where_} is {}; write true, false, or unknown. This engine will not read an \
         unrecognised value as false",
        raw.py_repr()
    ))
}

fn optional<T>(
    raw: Option<&Raw>,
    where_: &str,
    parse: impl Fn(&Raw, &str) -> Result<T, String>,
) -> Result<Option<T>, String> {
    match raw.filter(|_| !is_unknown(raw)) {
        None => Ok(None),
        Some(raw) => parse(raw, where_).map(Some),
    }
}

pub fn optional_money(raw: Option<&Raw>, where_: &str) -> Result<Option<Decimal>, String> {
    optional(raw, where_, parse_money)
}

pub fn optional_rate(raw: Option<&Raw>, where_: &str) -> Result<Option<Decimal>, String> {
    optional(raw, where_, parse_rate)
}

/// A term, remaining term or coverage ratio: non-negative, finite, may be fractional.
pub fn optional_years(raw: Option<&Raw>, where_: &str) -> Result<Option<Decimal>, String> {
    optional(raw, where_, parse_ratio)
}

pub fn optional_year_of_income(
    raw: Option<&Raw>,
    where_: &str,
) -> Result<Option<YearOfIncome>, String> {
    optional(raw, where_, parse_year)
}

/// `str(row.get(key, "") or "").strip()`.
pub fn text_cell(raw: Option<&Raw>) -> String {
    match raw {
        Some(raw) if raw.truthy() => pytext::strip(&raw.py_str()).to_string(),
        _ => String::new(),
    }
}
