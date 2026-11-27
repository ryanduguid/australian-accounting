// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Records given as rows: reading their fields, and the hours diary that
//! stands in for `hours` from 2022-23 (PCG 2023/1 paras 26(a) and 56 to 59).

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::{Arg, Decimal, Result, Step, conflict, int, invalid, money, refuse, step};
use crate::dates;

/// The most rows a diary or log book may hold.
pub(super) const MAX_ROWS: usize = 3000;

const DIARY: &str = "PCG 2023/1 paras 26(a) and 58: the hours worked from home in the income year, from the record of hours kept for the whole year, as the diary's entries total";

const HOURS_RECORD: &str = "PCG 2023/1 para 58: confirm the hours come from a record kept at the time for the whole income year (timesheets, rosters, logs of time on employer or business systems, time-tracking apps or a diary); an estimate for the year, or one period's hours applied to the rest of it, is not accepted.";
const HOURS_RECORD_2022: &str = "PCG 2023/1 para 59 (2022-23 only): confirm there is a record representative of the hours worked from home from 1 July 2022 to 28 February 2023 and a record of the actual hours worked from home from 1 March 2023 to 30 June 2023.";
const HOURS_ACTUAL: &str = "PCG 2023/1 para 28: confirm the hours do not exceed the hours actually worked at home, and that no expense the rate covers is claimed separately.";
const RUNNING_RECORD: &str = "PCG 2023/1 paras 56 and 60 to 61: confirm there is one document (an invoice, bill or credit card statement) for each additional running expense the rate covers that was incurred, at least one monthly or quarterly bill for energy, phone and internet, and evidence of incurring an expense whose bill is not in the taxpayer's name.";
const BESIDE_RECORD: &str = "PCG 2023/1 para 57: for each amount claimed beside the rate, confirm there are documents meeting Division 900 (an employee) or ITAA 1936 s 262A (a business) and records showing its income-producing use in the year.";

/// The object `value` with exactly `keys`, or the refusal naming `what`.
pub(super) fn fields<'a>(
    value: &'a Value,
    what: &str,
    keys: &[&str],
) -> Result<&'a Map<String, Value>> {
    let Value::Object(map) = value else {
        return invalid(format!("{what} must be an object."));
    };
    if let Some(key) = map.keys().find(|k| !keys.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("{key} is not a field of {what}"),
        );
    }
    if let Some(key) = keys.iter().find(|k| !map.contains_key(**k)) {
        return refuse("input.missing_field", format!("{what}: {key} is required"));
    }
    Ok(map)
}

/// A strict `YYYY-MM-DD` field as its day number.
pub(super) fn date(map: &Map<String, Value>, key: &str, what: &str) -> Result<i64> {
    match map[key].as_str().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => invalid(format!("{what}: {key} must be a date YYYY-MM-DD.")),
    }
}

/// A decimal string field read as every amount is: finite, not negative, at
/// most 2 decimal places and 1e12.
pub(super) fn amount(map: &Map<String, Value>, key: &str) -> Result<Decimal> {
    let parsed = match &map[key] {
        Value::String(text) if text.len() <= 32 => Decimal::parse_input(text).ok(),
        _ => None,
    };
    money(&parsed.map_or(Arg::Bool(false), Arg::Decimal))
}

/// The list `value` holds, 1 to [`MAX_ROWS`] long.
pub(super) fn list<'a>(value: &'a Value, what: &str) -> Result<&'a [Value]> {
    match value {
        Value::Array(rows) if !rows.is_empty() && rows.len() <= MAX_ROWS => Ok(rows),
        _ => invalid(format!("{what} must be a list of 1 to {MAX_ROWS} objects.")),
    }
}

/// The income year's first and last days.
pub(super) fn year_days(first: i64) -> (i64, i64) {
    let day = |text: String| dates::parse(&text).expect("income year");
    (
        day(format!("{first}-07-01")),
        day(format!("{}-06-30", first + 1)),
    )
}

/// An hours diary's total, its step and what it covered.
pub(super) struct Diary {
    pub(super) hours: Decimal,
    days: usize,
    from: i64,
    to: i64,
    terms: Vec<String>,
}

impl Diary {
    /// The diary's rows, each dated inside the year, with hours above 0 and
    /// at most 24 on any one date.
    pub(super) fn read(given: &Arg, year: &str, first: i64) -> Result<Self> {
        let Arg::Json(value) = given else {
            return invalid(format!(
                "hours_diary must be a list of 1 to {MAX_ROWS} objects."
            ));
        };
        let (start, end) = year_days(first);
        let mut by_date: BTreeMap<i64, Decimal> = BTreeMap::new();
        let mut terms = Vec::new();
        for (i, row) in list(value, "hours_diary")?.iter().enumerate() {
            let what = format!("hours_diary row {}", i + 1);
            let map = fields(row, &what, &["date", "hours"])?;
            let day = date(map, "date", &what)?;
            let hours = amount(map, "hours")?;
            if hours.is_zero() {
                return invalid(format!("{what}: hours must be above 0."));
            }
            if day < start || day > end {
                return conflict(format!("{what}: {} is outside {year}.", dates::format(day)));
            }
            let total = by_date.entry(day).or_insert_with(|| int(0));
            *total = total.add(&hours)?;
            if total.gt(&int(24))? {
                return conflict(format!(
                    "hours_diary: {} has more than 24 hours.",
                    dates::format(day)
                ));
            }
            terms.push(hours.to_string());
        }
        let hours = by_date.values().try_fold(int(0), |sum, h| sum.add(h))?;
        let (from, to) = (
            *by_date.keys().next().unwrap(),
            *by_date.keys().last().unwrap(),
        );
        let days = by_date.len();
        Ok(Self {
            hours,
            days,
            from,
            to,
            terms,
        })
    }

    pub(super) fn step(&self) -> Step {
        step(
            "diary_hours",
            DIARY,
            &self.terms.join(" + "),
            &[],
            self.hours.clone(),
        )
    }

    pub(super) fn report(&self, out: &mut Value) {
        out["diary_days"] = json!(self.days);
        out["diary_first_date"] = json!(dates::format(self.from));
        out["diary_last_date"] = json!(dates::format(self.to));
    }
}

/// The records PCG 2023/1 makes a condition of the fixed rate, for the
/// operator to confirm; `beside` when an amount is claimed beside the rate.
pub(super) fn fixed_rate_records(first: i64, beside: bool) -> Value {
    let hours = if first == 2022 {
        HOURS_RECORD_2022
    } else {
        HOURS_RECORD
    };
    let mut records = vec![hours, HOURS_ACTUAL, RUNNING_RECORD];
    if beside {
        records.push(BESIDE_RECORD);
    }
    json!(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_hours_diary_that_is_not_json_is_refused() {
        let error = Diary::read(&Arg::Text("kept".into()), "2025-26", 2025)
            .err()
            .unwrap();
        assert!(
            error
                .to_string()
                .starts_with("hours_diary must be a list of 1 to ")
        );
    }
}
