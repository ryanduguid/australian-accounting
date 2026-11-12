// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! A corpus title's standing against one date, from a row's own fields, as the Python
//! server's as_at.py works it out. A status is about a title's compilation window,
//! never about the law on that date.
use super::POLICY;
use serde_json::{Value, json};

pub(super) const CAPTURED: &str = "captured";
pub(super) const STATUSES: [&str; 4] = [
    CAPTURED,
    "before_compilation",
    "superseded",
    "after_retrieval",
];

fn policy(name: &str) -> &'static Value {
    &POLICY["as_at"][name]
}

/// Whether value is a canonical YYYY-MM-DD date; other forms Python parses are not.
fn is_date(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        text.len() == 10
            && chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d")
                .is_ok_and(|day| day.format("%Y-%m-%d").to_string() == text && text > "0000-12-31")
    })
}

/// The requested date, refused unless it is a real date written YYYY-MM-DD.
pub(super) fn checked(value: &Value) -> Result<Option<String>, String> {
    if value.is_null() {
        return Ok(None);
    }
    if !is_date(value) {
        return Err(format!(
            "as_at must be a YYYY-MM-DD date; got {}",
            div7a_loan_review::raw::Raw::from_value(value).py_repr()
        ));
    }
    Ok(value.as_str().map(str::to_owned))
}

/// Python's `f"{row.get('register_id')!s:.32}"`.
pub(super) fn register_id(row: &Value) -> String {
    let id = match &row["register_id"] {
        Value::String(text) => text.clone(),
        Value::Null => "None".into(),
        other => div7a_loan_review::raw::Raw::from_value(other).py_str(),
    };
    id.chars().take(32).collect()
}

fn broken(row: &Value, problem: &str) -> String {
    format!(
        "{}: {problem}. {}",
        register_id(row),
        policy("UNCLASSIFIABLE").as_str().unwrap()
    )
}

fn date<'a>(row: &'a Value, field: &str) -> Result<&'a str, String> {
    if !is_date(&row[field]) {
        return Err(broken(row, &format!("{field} is not a YYYY-MM-DD date")));
    }
    Ok(row[field].as_str().unwrap())
}

/// The start of the later version, or None for a current version. A row without
/// version_is_current cannot be taken as current.
fn superseded<'a>(row: &'a Value, start: &str, retrieved: &str) -> Result<Option<&'a str>, String> {
    let Some(current) = row["version_is_current"].as_bool() else {
        return Err(broken(row, "version_is_current must be true or false"));
    };
    if current {
        if !row["superseded_from"].is_null() {
            return Err(broken(
                row,
                "superseded_from must be null when version_is_current is true",
            ));
        }
        return Ok(None);
    }
    let later = date(row, "superseded_from")?;
    if !(start < later && later <= retrieved) {
        return Err(broken(
            row,
            "superseded_from must fall after compilation_date and on or before retrieved",
        ));
    }
    Ok(Some(later))
}

/// The row's status against `on`, its window, what the status covers and the next step.
/// Canonical dates order as text.
pub(super) fn standing(row: &Value, on: &str) -> Result<Value, String> {
    let start = date(row, "compilation_date")?;
    let retrieved = date(row, "retrieved")?;
    if start > retrieved {
        return Err(broken(row, "compilation_date is after retrieved"));
    }
    let later = superseded(row, start, retrieved)?;
    let status = if on < start {
        "before_compilation"
    } else if let Some(later) = later {
        if on >= later { "superseded" } else { CAPTURED }
    } else if on > retrieved {
        "after_retrieval"
    } else {
        CAPTURED
    };
    Ok(
        json!({"date":on,"status":status,"compilation_date":start,"superseded_from":later,
        "retrieved":retrieved,"covers":policy("COVERS"),"next_step":policy("NEXT_STEPS")[status]}),
    )
}
