// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Look up a reviewed ATO rate for a date, or refuse. Never a nearest-year figure.
//!
//! The records are the Python server's `rate_records/`, a copy of
//! `packages/au-tax-rates-data/data` that a root test holds equal. Each is a
//! point-in-time copy of an ATO page, never a statement of current law.
use chrono::NaiveDate;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::LazyLock};

const STATEMENT: &str = "A point-in-time copy of an ATO page, captured at retrieved_at, not a live lookup or a confirmation of current law. A record with no effective_to says only that the figure applied when it was retrieved. Retain source_url, quote and notes, and confirm the figure at the source before relying on it.";
const FILES: &[&str] = include!(concat!(env!("OUT_DIR"), "/rate_records.rs"));

/// Every reviewed record by id.
pub(super) static RECORDS: LazyLock<BTreeMap<String, Value>> = LazyLock::new(|| {
    FILES
        .iter()
        .map(|text| {
            let record: Value = serde_json::from_str(text).expect("bundled rate record");
            (record["id"].as_str().expect("record id").to_owned(), record)
        })
        .collect()
});

/// The income years the records name, which bound every lookup.
pub(super) fn covered_years() -> Vec<String> {
    let mut years: Vec<String> = RECORDS
        .values()
        .filter_map(|r| r["period"]["income_year"].as_str().map(str::to_owned))
        .collect();
    years.sort();
    years.dedup();
    years
}

fn year_start(label: &str) -> NaiveDate {
    NaiveDate::from_ymd_opt(label[..4].parse().unwrap(), 7, 1).unwrap()
}

fn year_end(label: &str) -> NaiveDate {
    NaiveDate::from_ymd_opt(label[..4].parse::<i32>().unwrap() + 1, 6, 30).unwrap()
}

fn day(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("bundled record date")
}

/// Whether the record's own period holds `on`; None when it records no period.
fn covers(record: &Value, on: NaiveDate) -> Option<bool> {
    let period = &record["period"];
    if let Some(year) = period["income_year"].as_str() {
        return Some(year_start(year) <= on && on <= year_end(year));
    }
    let from = period["effective_from"].as_str()?;
    // An open-ended record is bounded by covered_years before it reaches this test.
    let to = period["effective_to"].as_str().map_or(NaiveDate::MAX, day);
    Some(day(from) <= on && on <= to)
}

fn result(record_id: &str, on: NaiveDate, status: &str) -> Value {
    json!({"ok":true,"status":status,"record_id":record_id,"as_at":on.to_string(),
        "covered_years":covered_years(),"record":null,"refusal":null,"reason":null,
        "record_period":null,"covering_ids":[],"known_ids":[],"after_last_read":false,
        "after_last_read_note":null,"currency":"NOT_ESTABLISHED","statement":STATEMENT})
}

fn refused(
    record_id: &str,
    on: NaiveDate,
    code: &str,
    reason: String,
    record: Option<&Value>,
    covering_ids: Vec<String>,
    known_ids: Vec<String>,
) -> Value {
    let mut out = result(record_id, on, "REFUSED");
    out["refusal"] = json!(code);
    out["reason"] = json!(format!("{reason} No figure is returned."));
    out["record_period"] = record.map_or(Value::Null, |r| r["period"].clone());
    out["covering_ids"] = json!(covering_ids);
    out["known_ids"] = json!(known_ids);
    out
}

/// The named record if as_at falls in the covered years and its period, else a refusal.
pub(super) fn lookup(input: &Value) -> Result<Value, String> {
    let record_id = input["record_id"].as_str().unwrap();
    let as_at = input["as_at"].as_str().unwrap();
    super::valid_day(as_at, "as_at").map_err(|_| {
        format!(
            "as_at must be a YYYY-MM-DD date; got {}",
            div7a_loan_review::raw::Raw::text(as_at).py_repr()
        )
    })?;
    let on = day(as_at);
    let repr = div7a_loan_review::raw::Raw::text(record_id).py_repr();
    let Some(record) = RECORDS.get(record_id) else {
        let reason = format!("No reviewed record has the id {repr}.");
        let known = RECORDS.keys().cloned().collect();
        return Ok(refused(
            record_id,
            on,
            "UNKNOWN_ID",
            reason,
            None,
            vec![],
            known,
        ));
    };
    let years = covered_years();
    let (first, last) = (&years[0], &years[years.len() - 1]);
    if !(year_start(first) <= on && on <= year_end(last)) {
        let reason = format!(
            "{on} is outside the reviewed records, which cover {} only.",
            years.join(" and ")
        );
        let code = "OUTSIDE_COVERED_YEARS";
        return Ok(refused(
            record_id,
            on,
            code,
            reason,
            Some(record),
            vec![],
            vec![],
        ));
    }
    match covers(record, on) {
        None => {
            let reason = format!(
                "{record_id} records no effective period, only the date its page was read, so it cannot be placed at any date."
            );
            let code = "NO_RECORDED_PERIOD";
            Ok(refused(
                record_id,
                on,
                code,
                reason,
                Some(record),
                vec![],
                vec![],
            ))
        }
        Some(false) => {
            let covering = RECORDS
                .values()
                .filter(|other| {
                    other["label"] == record["label"] && covers(other, on) == Some(true)
                })
                .map(|other| other["id"].as_str().unwrap().to_owned())
                .collect();
            let reason = format!("{on} is outside the period {record_id} holds.");
            let code = "OUTSIDE_RECORD_PERIOD";
            Ok(refused(
                record_id,
                on,
                code,
                reason,
                Some(record),
                covering,
                vec![],
            ))
        }
        Some(true) => {
            let mut found = result(record_id, on, "FOUND");
            found["record"] = record.clone();
            let period = &record["period"];
            let read = day(&record["retrieved_at"].as_str().unwrap()[..10]);
            if on > read
                && period.get("effective_from").is_some()
                && period.get("effective_to").is_none()
            {
                found["after_last_read"] = json!(true);
                found["after_last_read_note"] = json!(format!(
                    "This record was last read on {read} and does not show that the figure still applied on {on}."
                ));
            }
            Ok(found)
        }
    }
}
