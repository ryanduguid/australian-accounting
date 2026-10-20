//! Decimal money, rate and ratio handling. No float anywhere: money is
//! quantised to cents with ROUND_HALF_UP, the engine's own documented choice.

use crate::decimal::{Calc, Decimal, Rounding, with_precision};
use crate::pytext;
use crate::raw::Raw;

pub const MAX_MONEY_MAGNITUDE: &str = "1000000000000.00";
pub const MAX_MONEY_DECIMAL_PLACES: u32 = 2;
pub const ROUNDING: &str = "ROUND_HALF_UP";
/// Intermediate precision for the s 109E(6) formula.
pub const FORMULA_PRECISION: i128 = 60;
/// A benchmark rate above this is a typo, not a rate.
pub const RATE_CEILING: &str = "1";

pub fn d(text: &str) -> Decimal {
    Decimal::parse(text).expect("decimal literal")
}

/// Build a Decimal from operator input, refusing the values that poison a report.
fn decimal(raw: &Raw, where_: &str) -> Result<Decimal, String> {
    let shown = raw.py_str();
    let text = pytext::strip(&shown);
    if text.is_empty() {
        return Err(format!(
            "{where_} is empty; supply a number or the word 'unknown'"
        ));
    }
    let Ok(value) = Decimal::parse(text) else {
        return Err(format!(
            "{where_} is {}, which is not a number",
            raw.py_repr()
        ));
    };
    if !value.is_finite() {
        return Err(format!(
            "{where_} is {}; nan and infinity compare false against every threshold in \
             this engine and would silently produce a clean verdict",
            raw.py_repr()
        ));
    }
    Ok(value)
}

fn below_zero(value: &Decimal) -> bool {
    value.lt(&d("0")).expect("finite")
}

/// A non-negative amount of money, in dollars and cents.
pub fn parse_money(raw: &Raw, where_: &str) -> Result<Decimal, String> {
    let value = decimal(raw, where_)?;
    if below_zero(&value) {
        return Err(format!(
            "{where_} is {}; a negative amount here inverts the shortfall and reports \
             exposure as money owed back to the borrower",
            raw.py_repr()
        ));
    }
    if value.gt(&d(MAX_MONEY_MAGNITUDE)).expect("finite") {
        return Err(format!(
            "{where_} is {}; amounts cannot exceed {MAX_MONEY_MAGNITUDE} AUD",
            raw.py_repr()
        ));
    }
    let cents = to_cents(&value).expect("bounded amounts quantise");
    if !value.equals(&cents).expect("finite") {
        return Err(format!(
            "{where_} is {}; amounts cannot have more than {MAX_MONEY_DECIMAL_PLACES} \
             decimal places",
            raw.py_repr()
        ));
    }
    Ok(value)
}

/// An annual interest rate as a decimal fraction: 0.0877, not 8.77.
pub fn parse_rate(raw: &Raw, where_: &str) -> Result<Decimal, String> {
    let value = decimal(raw, where_)?;
    if below_zero(&value) {
        return Err(format!(
            "{where_} is {}; an interest rate cannot be negative",
            raw.py_repr()
        ));
    }
    if value.gt(&d(RATE_CEILING)).expect("finite") {
        return Err(format!(
            "{where_} is {}, above {RATE_CEILING} (100% a year). Rates are decimal \
             fractions in this engine: write 8.77 per cent as 0.0877",
            raw.py_repr()
        ));
    }
    Ok(value)
}

/// A non-negative ratio, such as security cover expressed against the loan.
pub fn parse_ratio(raw: &Raw, where_: &str) -> Result<Decimal, String> {
    let value = decimal(raw, where_)?;
    if below_zero(&value) {
        return Err(format!(
            "{where_} is {}; a coverage ratio cannot be negative",
            raw.py_repr()
        ));
    }
    Ok(value)
}

/// Quantise to cents with ROUND_HALF_UP at the formula precision.
pub fn to_cents(value: &Decimal) -> Calc<Decimal> {
    with_precision(FORMULA_PRECISION, || value.quantize(-2, Rounding::HalfUp))
}

/// The JSON and CSV form of an amount: always 2 places.
pub fn cents_str(value: &Decimal) -> Calc<String> {
    Ok(to_cents(value)?.to_sci_string())
}
