// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Input and arithmetic helpers shared by `partnership-net-income`,
//! `trustee-assessment` and `trust-loss-tests`. Added after the Python port.
use crate::api::{decimal, integer, is_absent};
use crate::decimal::{Decimal, Rounding, Signal};
use crate::engine::d;
use serde_json::{Value, json};

pub type Result<T> = std::result::Result<T, String>;

/// A trapped decimal signal, refused as `decimal.<Signal>` like the older commands.
impl From<Signal> for String {
    fn from(s: Signal) -> String {
        format!("decimal.{}", s.name())
    }
}

/// Income years 2024-25 to 2026-27, as years ending 30 June.
pub const YEARS: std::ops::RangeInclusive<i64> = 2025..=2027;

pub fn refuse(reason: impl std::fmt::Display, provision: &str) -> String {
    format!("refused: {reason} ({provision})")
}

pub fn year(v: &Value) -> Result<i64> {
    let fy = integer(v, "financial_year")?;
    if !YEARS.contains(&fy) {
        return Err(refuse(
            format!("financial_year {fy} is outside 2025 (2024-25) to 2027 (2026-27)"),
            "years this command covers",
        ));
    }
    Ok(fy)
}

/// A non-negative amount in dollars and cents below 1E+15; omitted or null is nil.
pub fn money(v: &Value, key: &str) -> Result<Decimal> {
    if is_absent(v, key) {
        return Ok(d("0"));
    }
    let x = decimal(v, key)?;
    let bad = !x.is_finite()
        || x.lt(&d("0"))?
        || (!x.is_zero() && x.adjusted() > 14)
        || !x.quantize(-2, Rounding::HalfUp)?.equals(&x)?;
    if bad {
        return Err(format!(
            "{key} must be a non-negative amount below 1E+15 with at most 2 decimal places"
        ));
    }
    Ok(x)
}

/// A percentage from 0 to 100 inclusive.
pub fn percentage(v: &Value, key: &str) -> Result<Decimal> {
    let x = decimal(v, key)?;
    if !x.is_finite() || x.lt(&d("0"))? || x.gt(&d("100"))? {
        return Err(format!("{key} must be a percentage from 0 to 100"));
    }
    Ok(x)
}

/// A yes-or-no input that defaults to false.
pub fn flag(v: &Value, key: &str) -> Result<bool> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        _ => Err(format!("{key} must be true or false")),
    }
}

/// A stated fact: true, false or unstated (null or omitted).
pub fn fact(v: &Value, key: &str) -> Result<Option<bool>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        _ => Err(format!("{key} must be true, false or null")),
    }
}

/// The first flag in `flags` that is true, refused with its reason and provision.
pub fn out_of_scope(v: &Value, flags: &[(&str, &str, &str)]) -> Result<()> {
    for (key, what, law) in flags {
        if flag(v, key)? {
            return Err(refuse(what, law));
        }
    }
    Ok(())
}

pub fn cents(x: &Decimal) -> Result<Decimal> {
    Ok(x.quantize(-2, Rounding::HalfUp)?)
}

pub fn amount(x: &Decimal) -> Value {
    json!(x.fixed(2, false))
}

pub fn least(a: Decimal, b: Decimal) -> Result<Decimal> {
    Ok(if b.lt(&a)? { b } else { a })
}

/// Splits `total` (whole cents) by `weights` (fractions summing to 1): each part is
/// rounded half up to the cent, and the cents left over or overdrawn go one at a
/// time to the largest parts first, so the parts add up to `total`.
pub fn share_out(total: &Decimal, weights: &[Decimal]) -> Result<Vec<Decimal>> {
    let exact = weights
        .iter()
        .map(|w| total.mul(w))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut parts = exact.iter().map(cents).collect::<Result<Vec<_>>>()?;
    let mut order: Vec<usize> = (0..parts.len()).collect();
    order.sort_by(|&a, &b| {
        exact[b]
            .compare(&exact[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut left = parts.iter().try_fold(total.clone(), |t, p| t.sub(p))?;
    let cent = if left.lt(&d("0"))? {
        d("-0.01")
    } else {
        d("0.01")
    };
    // Each rounded part is within half a cent, so fewer cents are left than parts.
    for &i in &order {
        if left.is_zero() {
            break;
        }
        parts[i] = parts[i].add(&cent)?;
        left = left.sub(&cent)?;
    }
    Ok(parts)
}
