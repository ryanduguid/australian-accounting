// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Reading `leave_provisions`' lines: each employee's or group's accumulated
//! leave with the caller's stated assumptions.

use serde_json::{Map, Value};

use super::super::{Arg, Result, d, money, refuse};
use crate::decimal::Decimal;

/// The most lines one worksheet takes.
const MOST: usize = 500;
/// The most whole years to settlement a long-term line takes. The least is 1:
/// a line expected to be settled within 12 months is short-term (AASB 119
/// para 8).
const MOST_YEARS: u64 = 50;
const SHORT: [&str; 6] = [
    "label",
    "leave",
    "hours",
    "hourly_rate",
    "probability",
    "settled_within_12_months",
];
const LONG: [&str; 4] = [
    "years_to_settlement",
    "salary_growth",
    "discount_rate",
    "can_defer_settlement",
];

/// A long-term line's assumptions (AASB 119 paras 83 to 87 and 155).
pub(super) struct LongTerm {
    pub years: u64,
    pub growth: Decimal,
    pub discount: Decimal,
    pub can_defer: bool,
}

/// One line: annual or long service leave, its accumulated hours at the
/// reporting date, the rate, the probability of payment and, when it is not
/// expected to be settled wholly within 12 months, its long-term assumptions.
pub(super) struct Line {
    pub label: String,
    pub long_service: bool,
    pub hours: Decimal,
    pub rate: Decimal,
    pub probability: Decimal,
    pub long_term: Option<LongTerm>,
}

fn invalid<T>(why: String) -> Result<T> {
    refuse("leave_provisions.line_invalid", why)
}

pub(super) fn read(given: &Arg) -> Result<Vec<Line>> {
    let Arg::Json(Value::Array(list)) = given else {
        return invalid(format!("lines must be a list of 1 to {MOST} objects."));
    };
    if list.is_empty() || list.len() > MOST {
        return invalid(format!("lines must be a list of 1 to {MOST} objects."));
    }
    list.iter()
        .enumerate()
        .map(|(i, v)| line(i + 1, v))
        .collect()
}

/// A decimal string from 0 to 1 with at most 6 places.
fn unit(fields: &Map<String, Value>, key: &str, n: usize) -> Result<Decimal> {
    let parsed = fields
        .get(key)
        .and_then(Value::as_str)
        .and_then(|t| Decimal::parse_input(t).ok())
        .filter(|v| v.is_finite() && v.exponent() >= -6);
    match parsed {
        Some(v) if !v.lt(&d("0"))? && !v.gt(&d("1"))? => Ok(v),
        _ => invalid(format!(
            "line {n}: {key} must be a decimal string from 0 to 1, at most 6 places."
        )),
    }
}

fn amount(fields: &Map<String, Value>, key: &str, n: usize) -> Result<Decimal> {
    let parsed = fields
        .get(key)
        .and_then(Value::as_str)
        .and_then(|t| Decimal::parse_input(t).ok());
    match parsed.map(|v| money(&Arg::Decimal(v))) {
        Some(Ok(v)) => Ok(v),
        _ => invalid(format!(
            "line {n}: {key} must be a finite non-negative decimal string, at most 2dp and 1e12."
        )),
    }
}

fn boolean(fields: &Map<String, Value>, key: &str, n: usize) -> Result<bool> {
    match fields.get(key) {
        Some(Value::Bool(b)) => Ok(*b),
        _ => invalid(format!("line {n}: {key} must be a boolean.")),
    }
}

fn label(fields: &Map<String, Value>, n: usize) -> Result<String> {
    match fields.get("label") {
        Some(Value::String(s)) if !s.trim().is_empty() && s.chars().count() <= 200 => Ok(s.clone()),
        _ => invalid(format!(
            "line {n}: label must be a non-empty string of at most 200 characters."
        )),
    }
}

fn kind(fields: &Map<String, Value>, n: usize) -> Result<bool> {
    match fields.get("leave").and_then(Value::as_str) {
        Some("annual") => Ok(false),
        Some("long_service") => Ok(true),
        _ => invalid(format!(
            "line {n}: leave must be annual or long_service; sick, parental, jury and military service leave, long-term disability and termination benefits are outside this worksheet (AASB 119 paras 17, 18, 157 and 159)."
        )),
    }
}

fn long_term(fields: &Map<String, Value>, n: usize) -> Result<LongTerm> {
    let years = match fields.get("years_to_settlement").and_then(Value::as_u64) {
        Some(y) if (1..=MOST_YEARS).contains(&y) => y,
        _ => {
            return invalid(format!(
                "line {n}: years_to_settlement must be a whole number from 1 to {MOST_YEARS}; a portion expected to be settled within 12 months is a short-term line (AASB 119 para 8)."
            ));
        }
    };
    Ok(LongTerm {
        years,
        growth: unit(fields, "salary_growth", n)?,
        discount: unit(fields, "discount_rate", n)?,
        can_defer: boolean(fields, "can_defer_settlement", n)?,
    })
}

fn line(n: usize, value: &Value) -> Result<Line> {
    let Some(fields) = value.as_object() else {
        return invalid(format!("line {n} must be an object."));
    };
    let known = |k: &str| SHORT.contains(&k) || LONG.contains(&k);
    if let Some(key) = fields.keys().find(|k| !known(k)) {
        return invalid(format!("line {n}: {key} is not a field of a line."));
    }
    let short_term = boolean(fields, "settled_within_12_months", n)?;
    if short_term && let Some(key) = LONG.iter().find(|k| fields.contains_key(**k)) {
        return invalid(format!(
            "line {n}: {key} is for a line not expected to be settled wholly within 12 months; a short-term line is undiscounted (AASB 119 para 11)."
        ));
    }
    Ok(Line {
        label: label(fields, n)?,
        long_service: kind(fields, n)?,
        hours: amount(fields, "hours", n)?,
        rate: amount(fields, "hourly_rate", n)?,
        probability: unit(fields, "probability", n)?,
        long_term: if short_term {
            None
        } else {
            Some(long_term(fields, n)?)
        },
    })
}
