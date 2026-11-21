// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `fhss` inputs: dates, financial years, the contributions list and the
//! personal deductions, each checked as the command reads it.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::dates;
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, money, refuse};

/// A contribution as given, or the part of one treated as concessional or
/// non-concessional.
#[derive(Clone)]
pub(super) struct Piece {
    /// The contribution's number in the list, from 1.
    pub(super) n: usize,
    pub(super) day: i64,
    pub(super) fy: i64,
    pub(super) kind: &'static str,
    pub(super) concessional: bool,
    pub(super) amount: Decimal,
    pub(super) counted: Decimal,
}

pub(super) fn day(value: &Arg, name: &str) -> Result<i64> {
    match value.text().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => refuse(
            "input.date_invalid",
            format!("{name} must be a YYYY-MM-DD date from 1900 to 2100."),
        ),
    }
}

/// The year a financial year starts in: 2024 for 1 July 2024 to 30 June 2025.
pub(super) fn fy_of(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    if m >= 7 { y } else { y - 1 }
}

pub(super) fn fy_label(start: i64) -> String {
    format!("{start}-{:02}", (start + 1) % 100)
}

/// A decimal string field of a list item, as `money` reads one.
pub(super) fn item_money(fields: &Map<String, Value>, key: &str, what: &str) -> Result<Decimal> {
    let parsed = match fields.get(key) {
        Some(Value::String(text)) if text.len() <= 64 => Decimal::parse_input(text).ok(),
        _ => None,
    };
    match parsed.map(|v| money(&Arg::Decimal(v))) {
        Some(Ok(v)) => Ok(v),
        _ => refuse(
            "input.amount_invalid",
            format!("{what}: {key} must be a non-negative decimal string, at most 2dp and 1e12."),
        ),
    }
}

pub(super) fn only_fields(fields: &Map<String, Value>, allowed: &[&str], what: &str) -> Result<()> {
    match fields.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(key) => refuse(
            "input.field_unknown",
            format!("{what}: {key} is not an input of fhss."),
        ),
        None => Ok(()),
    }
}

const NOT_ELIGIBLE: &str = "Only voluntary contributions are eligible (TAA 1953 Sch 1 s 138-35(2)): employer_voluntary (salary sacrifice or another employer contribution that is not mandated) or personal (made by you). Leave out super guarantee and other mandated or required contributions, spouse and family contributions, government co-contributions, contribution splits, downsizer, small business CGT, structured settlement and COVID-19 re-contributions (GN 2024/1 section 1); KiwiSaver and other foreign transfers are outside this command.";

/// The contributions list: each dated from 1 July 2017 to the determination
/// day, with an amount above nil and an eligible kind.
pub(super) fn read_contributions(list: &Arg, determination: i64) -> Result<Vec<Piece>> {
    let Arg::Json(Value::Array(items)) = list else {
        return refuse(
            "fhss.contributions_invalid",
            "contributions must be a list of at most 10000 objects with date, amount and kind.",
        );
    };
    if items.is_empty() || items.len() > 10_000 {
        return refuse(
            "fhss.contributions_invalid",
            "contributions must be a list of 1 to 10000 objects with date, amount and kind.",
        );
    }
    items
        .iter()
        .enumerate()
        .map(|(i, item)| contribution(i + 1, item, determination))
        .collect()
}

/// Contribution `n` of the list.
fn contribution(n: usize, item: &Value, determination: i64) -> Result<Piece> {
    let what = format!("contribution {n}");
    let Some(fields) = item.as_object() else {
        return refuse(
            "fhss.contributions_invalid",
            format!("{what} must be an object."),
        );
    };
    only_fields(fields, &["date", "amount", "kind"], &what)?;
    let day = contribution_day(fields, &what, determination)?;
    let amount = item_money(fields, "amount", &what)?;
    if amount.is_zero() {
        return refuse(
            "input.amount_invalid",
            format!("{what}: amount must be above nil."),
        );
    }
    let kind = match fields.get("kind").and_then(Value::as_str) {
        Some("employer_voluntary") => "employer_voluntary",
        Some("personal") => "personal",
        _ => {
            return refuse(
                "fhss.contribution_kind_invalid",
                format!("{what}: kind must be employer_voluntary or personal. {NOT_ELIGIBLE}"),
            );
        }
    };
    Ok(Piece {
        n,
        day,
        fy: fy_of(day),
        kind,
        concessional: kind == "employer_voluntary",
        amount,
        counted: Decimal::from_int(0),
    })
}

/// A contribution's date, from 1 July 2017 to the determination day.
fn contribution_day(fields: &Map<String, Value>, what: &str, determination: i64) -> Result<i64> {
    let Some(day) = fields
        .get("date")
        .and_then(Value::as_str)
        .and_then(dates::parse)
    else {
        return refuse(
            "input.date_invalid",
            format!("{what}: date must be a YYYY-MM-DD date."),
        );
    };
    if day < dates::parse("2017-07-01").expect("date") || day > determination {
        return refuse(
            "fhss.contribution_out_of_period",
            format!(
                "{what}: only contributions made from 1 July 2017 (s 138-30(1)) to the determination day can count in it."
            ),
        );
    }
    Ok(day)
}

/// The personal contributions deducted for each financial year (ITAA 1997
/// s 290-150), which makes them concessional.
pub(super) fn read_deductions(list: Option<&Arg>) -> Result<BTreeMap<i64, Decimal>> {
    let mut out = BTreeMap::new();
    let Some(list) = list else {
        return Ok(out);
    };
    let Arg::Json(Value::Array(items)) = list else {
        return refuse(
            "fhss.deductions_invalid",
            "personal_deductions must be a list of objects with financial_year and amount.",
        );
    };
    for (i, item) in items.iter().enumerate() {
        let what = format!("personal deduction {}", i + 1);
        let Some(fields) = item.as_object() else {
            return refuse(
                "fhss.deductions_invalid",
                format!("{what} must be an object."),
            );
        };
        only_fields(fields, &["financial_year", "amount"], &what)?;
        let fy = fields
            .get("financial_year")
            .and_then(Value::as_str)
            .and_then(|label| {
                let start: i64 = label.get(..4)?.parse().ok()?;
                (fy_label(start) == label).then_some(start)
            });
        let Some(fy) = fy else {
            return refuse(
                "fhss.deductions_invalid",
                format!("{what}: financial_year must be a year such as \"2024-25\"."),
            );
        };
        if out
            .insert(fy, item_money(fields, "amount", &what)?)
            .is_some()
        {
            return refuse(
                "fhss.deductions_invalid",
                format!("{what}: {} is given twice.", fy_label(fy)),
            );
        }
    }
    Ok(out)
}
