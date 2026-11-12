// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Recorded whole-of-state holidays and caller-supplied calendar coverage.
use crate::{data, date};
use chrono::{Datelike, NaiveDate};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub(crate) struct Calendar {
    holidays: BTreeMap<NaiveDate, String>,
    covers_to: NaiveDate,
}
type Roll = (Option<NaiveDate>, Vec<Value>, Option<String>);

pub(crate) fn parse_date(value: &Value, name: &str) -> Result<NaiveDate, String> {
    let parsed = value
        .as_str()
        .filter(|s| {
            s.len() == 10
                && s.as_bytes()[4] == b'-'
                && s.as_bytes()[7] == b'-'
                && s.bytes()
                    .enumerate()
                    .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        })
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .filter(|d| (1..=9999).contains(&d.year()));
    parsed.ok_or_else(|| format!("{name} must be a real date in YYYY-MM-DD form"))
}

pub(crate) fn printable_char(c: char) -> bool {
    static RANGES: OnceLock<Vec<(u32, u32)>> = OnceLock::new();
    let ranges = RANGES.get_or_init(|| {
        serde_json::from_str(include_str!("../data/nonprintable.json"))
            .expect("compiled Unicode ranges")
    });
    let n = c as u32;
    let i = ranges.partition_point(|&(_, end)| end < n);
    i == ranges.len() || ranges[i].0 > n
}

fn base_holidays() -> BTreeMap<NaiveDate, String> {
    data()["holidays"]
        .as_object()
        .expect("compiled holidays")
        .iter()
        .map(|(day, name)| {
            (
                NaiveDate::parse_from_str(day, "%Y-%m-%d").expect("recorded holiday"),
                name.as_str().expect("holiday name").into(),
            )
        })
        .collect()
}

pub(crate) fn calendar(extra: &Value, complete: &Value) -> Result<Calendar, String> {
    let mut holidays = base_holidays();
    if !extra.is_null() {
        let map = extra
            .as_object()
            .filter(|m| m.len() <= 50)
            .ok_or("extra_holidays must map at most 50 dates to holiday names")?;
        for (key, name) in map {
            let day = parse_date(&Value::String(key.clone()), "extra_holidays key")?;
            let name=name.as_str().filter(|s|!s.trim().is_empty() && s.chars().count()<=120 && s.chars().all(printable_char))
                .ok_or_else(||format!("extra_holidays[{day}] needs a holiday name of at most 120 printable characters"))?;
            if holidays.contains_key(&day) || data()["conflicts"].get(key).is_some() {
                return Err(format!("extra_holidays[{day}] is already in the ATO table"));
            }
            if day.weekday().num_days_from_monday() >= 5 {
                return Err(format!("extra_holidays[{day}] falls on a weekend"));
            }
            holidays.insert(day, format!("{} (caller-supplied)", name.trim()));
        }
    }
    let covers_to = if complete.is_null() {
        date(2027, 6, 30)
    } else {
        parse_date(complete, "holidays_complete_until")?.max(date(2027, 6, 30))
    };
    Ok(Calendar {
        holidays,
        covers_to,
    })
}

pub(crate) fn roll_day(mut day: NaiveDate, cal: &Calendar) -> Result<Roll, String> {
    let mut skipped = vec![];
    loop {
        if let Some(conflict) = data()["conflicts"].get(day.to_string()) {
            return Ok((
                None,
                skipped,
                Some(conflict.as_str().expect("recorded conflict").into()),
            ));
        }
        let reason = match day.weekday().num_days_from_monday() {
            5 => Some("Saturday"),
            6 => Some("Sunday"),
            _ => cal.holidays.get(&day).map(String::as_str),
        };
        let Some(reason) = reason else {
            if day < date(2026, 7, 1) || day > cal.covers_to {
                return Ok((
                    None,
                    skipped,
                    Some(format!(
                        "whether {day} is a public holiday for the whole of any state or territory: the holidays known cover 2026-07-01 to {}",
                        cal.covers_to
                    )),
                ));
            }
            return Ok((Some(day), skipped, None));
        };
        skipped.push(json!({"date":day.to_string(),"reason":reason}));
        day = day
            .succ_opt()
            .filter(|d| d.year() <= 9999)
            .ok_or("holiday roll exceeds supported date range")?;
    }
}

pub(crate) fn description(cal: &Calendar, complete: &Value) -> Value {
    let supplied: BTreeMap<_, _> = cal
        .holidays
        .iter()
        .filter(|(_, name)| name.ends_with(" (caller-supplied)"))
        .map(|(day, name)| (day.to_string(), name))
        .collect();
    json!({
        "definition":"Saturdays, Sundays and days that are a public holiday for the whole of any state or territory move a due date to the next business day, for taxpayers in every state.",
        "source_url":data()["holiday_source"],"source_provision":data()["holiday_provision"],
        "covers_from":"2026-07-01","covers_to":cal.covers_to.to_string(),
        "caller_holidays":supplied,"caller_complete_until":complete
    })
}

/// JSON equivalent of the original holiday helper. This does not extend coverage
/// automatically when a caller supplies a holiday mapping.
pub fn roll(request: &Value) -> Result<Value, String> {
    let request = request
        .as_object()
        .ok_or("roll request must be an object")?;
    for key in request.keys() {
        if !["day", "holidays", "covers_to"].contains(&key.as_str()) {
            return Err(format!("unknown roll field: {key:?}"));
        }
    }
    let day = parse_date(request.get("day").unwrap_or(&Value::Null), "day")?;
    let mut cal = Calendar {
        holidays: base_holidays(),
        covers_to: date(2027, 6, 30),
    };
    if let Some(holidays) = request.get("holidays").filter(|v| !v.is_null()) {
        let map = holidays.as_object().ok_or("holidays must be an object")?;
        cal.holidays = map
            .iter()
            .map(|(key, value)| {
                Ok((
                    parse_date(&Value::String(key.clone()), "holiday key")?,
                    value.as_str().ok_or("holiday name must be text")?.into(),
                ))
            })
            .collect::<Result<_, String>>()?;
    }
    if let Some(to) = request.get("covers_to") {
        cal.covers_to = parse_date(to, "covers_to")?;
    }
    let (due, skipped, reason) = roll_day(day, &cal)?;
    Ok(json!([due.map(|d| d.to_string()), skipped, reason]))
}
