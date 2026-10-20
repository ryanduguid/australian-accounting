//! Amount parsing and formatting. Everything works in Decimal; no amount is
//! ever converted to floating point.

use std::sync::LazyLock;

use crate::decimal::{Decimal, Rounding};
use crate::pyre::Regex;
use crate::pytext;

/// Raised when a cell cannot be read as an amount (a `ValueError`).
#[derive(Clone, Debug, PartialEq)]
pub struct AmountError(pub String);

const NUMBER: &str = r"(?:[0-9]+|[0-9]{1,3}(?:,[0-9]{3})+)(?:\.[0-9]+)?|\.[0-9]+";

static ACCOUNTING_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::ignore_case(&format!(
        r"(?:[+-]?\$?\s*(?:{NUMBER})|\$[+-]\s*(?:{NUMBER})|\$?\s*\(\s*(?:{NUMBER})\s*\)|\(\s*\$?(?:{NUMBER})\s*\)|\$?\s*(?:{NUMBER})\s*(?:CR|DR))"
    ))
});
static CLEAN: LazyLock<Regex> = LazyLock::new(|| Regex::ignore_case(r"[\s$,()]|CR$|DR$"));

/// `parse_amount(raw, where)`: an accounting amount, or `None` for a missing cell.
pub fn parse_amount(raw: Option<&str>, where_: &str) -> Result<Decimal, AmountError> {
    parse_amount_shown(raw, raw.map(pytext::repr), where_)
}

/// `parse_amount` for `str(raw)` of a value whose `repr()` is `shown`.
pub fn parse_amount_shown(
    raw: Option<&str>,
    shown: Option<String>,
    where_: &str,
) -> Result<Decimal, AmountError> {
    let Some(raw) = raw else {
        return Err(AmountError(format!("{where_}: no amount given")));
    };
    let text = pytext::strip(raw);
    if text.is_empty() {
        return Err(AmountError(format!("{where_}: no amount given")));
    }
    if !ACCOUNTING_NUMBER.fullmatch(text) {
        return Err(AmountError(format!(
            "{where_}: {} is not an amount",
            shown.unwrap_or_else(|| pytext::repr(raw))
        )));
    }
    let negative = text.contains('(') || text.to_ascii_uppercase().ends_with("CR");
    let cleaned = CLEAN.sub("", text);
    let value = Decimal::parse(&cleaned).expect("the grammar leaves a plain number");
    if negative {
        // Python's `-value` raised the trapped signal itself, uncaught.
        value
            .neg()
            .map_err(|signal| AmountError(format!("decimal.{}", signal.name())))
    } else {
        Ok(value)
    }
}

/// Round to fixed places, refusing a value the decimal context cannot hold.
fn quantise(value: &Decimal, exp: i128) -> Result<Decimal, AmountError> {
    let refuse = || AmountError(format!("{value} has more digits than this tool can report"));
    let quantised = value
        .quantize(exp, Rounding::HalfUp)
        .map_err(|_| refuse())?;
    if quantised.is_zero() {
        // `abs()` of a zero: the sign goes.
        return quantised.abs().map_err(|_| refuse());
    }
    Ok(quantised)
}

/// An amount for display, with thousands separators and two decimals.
pub fn money(value: &Decimal) -> Result<String, AmountError> {
    Ok(quantise(value, -2)?.fixed(2, true))
}

fn scaled(value: &Decimal) -> Result<Decimal, AmountError> {
    let hundred = Decimal::from_int(100);
    // Python's `value * 100` raised the trapped signal itself, uncaught; its
    // traceback's last line and the AmountError line read the same.
    let product = value
        .mul(&hundred)
        .map_err(|signal| AmountError(format!("decimal.{}", signal.name())))?;
    quantise(&product, -2)
}

/// A ratio as a percentage of turnover, to 2 places.
pub fn percent(value: &Decimal) -> Result<String, AmountError> {
    Ok(format!("{}%", scaled(value)?))
}

/// A published bound without padding zeros: 29% rather than 29.00%.
pub fn percent_compact(value: &Decimal) -> Result<String, AmountError> {
    let text = scaled(value)?.fixed(2, false);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    Ok(format!("{}%", if text.is_empty() { "0" } else { text }))
}

pub fn percent_range(low: &Decimal, high: &Decimal) -> Result<String, AmountError> {
    // Python's `low == high` raised InvalidOperation for a signalling NaN
    // before either bound was formatted.
    let same = low
        .equals(high)
        .map_err(|signal| AmountError(format!("decimal.{}", signal.name())))?;
    if same {
        return percent_compact(low);
    }
    Ok(format!(
        "{} to {}",
        percent_compact(low)?,
        percent_compact(high)?
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(text: &str) -> Result<String, String> {
        parse_amount(Some(text), "amount")
            .map(|d| d.to_sci_string())
            .map_err(|e| e.0)
    }

    #[test]
    fn the_accounting_grammar_matches_python() {
        assert_eq!(amount("1,234.50"), Ok("1234.50".into()));
        assert_eq!(amount("(1,234)"), Ok("-1234".into()));
        assert_eq!(amount("$ 12 cr"), Ok("-12".into()));
        assert_eq!(amount("-$5"), Ok("-5".into()));
        assert_eq!(amount("$-5"), Ok("-5".into()));
        assert_eq!(amount(".5"), Ok("0.5".into()));
        assert_eq!(amount("(0)"), Ok("0".into()));
        assert_eq!(
            amount("1,2,3"),
            Err("amount: '1,2,3' is not an amount".into())
        );
        assert_eq!(amount("NaN"), Err("amount: 'NaN' is not an amount".into()));
        assert_eq!(
            amount("(5) CR"),
            Err("amount: '(5) CR' is not an amount".into())
        );
        assert_eq!(amount("  "), Err("amount: no amount given".into()));
    }

    #[test]
    fn formats_match_python() {
        let d = |t: &str| Decimal::parse(t).unwrap();
        assert_eq!(money(&d("-0.001")).unwrap(), "0.00");
        assert_eq!(money(&d("1234567.005")).unwrap(), "1,234,567.01");
        assert_eq!(percent(&d("0.3096")).unwrap(), "30.96%");
        assert_eq!(percent_compact(&d("0.29")).unwrap(), "29%");
        assert_eq!(percent_compact(&d("0")).unwrap(), "0%");
        assert_eq!(percent_range(&d("0.01"), &d("0.010")).unwrap(), "1%");
        assert_eq!(
            percent_range(&d("0.31"), &d("0.385")).unwrap(),
            "31% to 38.5%"
        );
    }
}
