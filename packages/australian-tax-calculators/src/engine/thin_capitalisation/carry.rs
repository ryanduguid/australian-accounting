// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The special deduction for fixed ratio test disallowed amounts of the 15
//! income years before this one (ITAA 1997 ss 820-56 and 820-57), earliest
//! first and only up to the year's excess of the fixed ratio earnings limit
//! over net debt deductions. Whether an amount is available, that is not
//! treated as nil by s 820-58 (a later choice of another test) or s 820-59
//! (the modified continuity of ownership or business continuity test), is the
//! caller's stated fact.

use serde_json::{Value, json};

use super::{max0, total, value_of};
use crate::decimal::{Calc, Decimal, Rounding};
use crate::engine::{Arg, Result, Step, d, money, refuse, step};

const MOST: usize = 15;
/// The first income year the fixed ratio test applies to (2023-24).
const FIRST: i64 = 2023;

/// One earlier year's fixed ratio test disallowed amount, not yet applied.
pub(super) struct Carried {
    year: String,
    start: i64,
    amount: Decimal,
    available: bool,
}

/// The first calendar year of `year` written as 2024-25.
fn start_of(year: &str) -> Option<i64> {
    let (a, b) = year.split_once('-')?;
    if a.len() != 4 || b.len() != 2 {
        return None;
    }
    let (a, b): (i64, i64) = (a.parse().ok()?, b.parse().ok()?);
    ((a + 1) % 100 == b).then_some(a)
}

fn wrong<T>(why: String) -> Result<T> {
    refuse("thin_capitalisation.carry_forward_invalid", why)
}

fn entry(v: &Value, n: usize) -> Result<Carried> {
    let o = v.as_object();
    let known = o.is_some_and(|o| {
        o.keys()
            .all(|k| matches!(k.as_str(), "year" | "amount" | "available"))
    });
    let field = |key| o.and_then(|o| o.get(key));
    let year = field("year").and_then(Value::as_str);
    let amount = field("amount")
        .and_then(Value::as_str)
        .and_then(|s| Decimal::parse_input(s).ok())
        .and_then(|v| money(&Arg::Decimal(v)).ok());
    let available = field("available").and_then(Value::as_bool);
    match (year, year.and_then(start_of), amount, available) {
        (Some(y), Some(start), Some(amount), Some(available)) if known => Ok(Carried {
            year: y.to_string(),
            start,
            amount,
            available,
        }),
        _ => wrong(format!(
            "entry {n}: needs year (as 2024-25), amount in AUD from 0 and available true or false, and nothing else."
        )),
    }
}

/// The stated amounts, earliest year first (s 820-56(3)(a)).
pub(super) fn read(given: Option<&Arg>, year: &str) -> Result<Vec<Carried>> {
    let now = start_of(year).expect("a supported year");
    let items = match given {
        None => return Ok(vec![]),
        Some(Arg::Json(Value::Array(items))) if items.len() <= MOST => items,
        Some(_) => {
            return wrong(format!(
                "frt_disallowed_amounts must be a list of at most {MOST} objects."
            ));
        }
    };
    let mut out: Vec<Carried> = Vec::with_capacity(items.len());
    for (i, v) in items.iter().enumerate() {
        let c = entry(v, i + 1)?;
        if c.start < FIRST || c.start >= now || now - c.start > 15 {
            return wrong(format!(
                "entry {}: {} is not one of the 15 income years before {year} from 2023-24, the first year of the fixed ratio test (s 820-56(2)(b)).",
                i + 1,
                c.year
            ));
        }
        if out.iter().any(|o| o.start == c.start) {
            return wrong(format!("entry {}: {} is given twice.", i + 1, c.year));
        }
        out.push(c);
    }
    out.sort_by_key(|c| c.start);
    Ok(out)
}

/// The special deduction's step and each amount's application.
pub(super) fn apply(steps: &[Step], carried: &[Carried]) -> Calc<(Step, Value)> {
    let limit = value_of(steps, "fixed_ratio_earnings_limit");
    let ndd = value_of(steps, "net_debt_deductions");
    let room = max0(limit.sub(ndd)?)?;
    let pool: Vec<&Decimal> = carried
        .iter()
        .filter(|c| c.available)
        .map(|c| &c.amount)
        .collect();
    let sum = total(pool.iter().copied())?;
    let special = if room.lt(&sum)? { room } else { sum };
    let terms = if pool.is_empty() {
        "0".to_string()
    } else {
        pool.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" + ")
    };
    let formula = format!("min(max(fixed_ratio_earnings_limit - net_debt_deductions, 0), {terms})");
    let mut left = special.clone();
    let mut rows = Vec::with_capacity(carried.len());
    for c in carried {
        let applied = match c.available {
            true if left.lt(&c.amount)? => left.clone(),
            true => c.amount.clone(),
            false => d("0"),
        };
        left = left.sub(&applied)?;
        let cents = |v: &Decimal| v.quantize(-2, Rounding::HalfUp).map(|v| v.to_string());
        rows.push(json!({
            "year": c.year,
            "amount": cents(&c.amount)?,
            "available": c.available,
            "applied": cents(&applied)?,
            "remaining": cents(&c.amount.sub(&applied)?)?,
        }));
    }
    let special = step(
        "special_deduction",
        "s 820-56: the excess of the fixed ratio earnings limit over net debt deductions, applied to the available fixed ratio test disallowed amounts of the previous 15 income years, earliest first",
        &formula,
        &[limit, ndd],
        special,
    );
    Ok((special, Value::Array(rows)))
}
