// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `charity-check`: whether each ABN is on one week's copy of the ACNC
//! Register of Australian charities, with its registration date and subtypes.
//! The data is `data/acnc-register.jsonl`, built from the dataset by
//! `examples/acnc_register.rs`; its first line records the copy it came from.
//! A copy more than `MAX_AGE_DAYS` older than the check date is refused.
use crate::api::Refusal;
use crate::dates;
use crate::dgr::abn_digits;
use crate::engine;
use serde_json::{Value, json};
use std::sync::OnceLock;

const COMMAND: &str = "charity-check";
const MAX_ABNS: usize = 10_000;
/// The dataset is updated weekly (user notes para 4), so a copy older than
/// this has missed at least 4 updates.
const MAX_AGE_DAYS: i64 = 31;

const DATASET: &str = "https://data.gov.au/data/dataset/acnc-register";
const USER_NOTES: &str = "https://data.gov.au/data/dataset/b050b242-4487-4306-abf5-07ca073e5594/resource/8687e783-a9de-4431-a8fc-d677bc475c03/download/acnc-registered-charities-dataset-user-notes-dec-2024.pdf";
const ACNC_ACT: &str = "https://www.legislation.gov.au/C2012A00168/latest/text";
const LICENCE_URL: &str = "https://creativecommons.org/licenses/by/3.0/au/";
const SOURCES: [&str; 3] = [DATASET, USER_NOTES, ACNC_ACT];

/// ACNC Act 2012 s 25-5(5), items 1 to 14, in table order.
pub const SUBTYPES: [&str; 14] = [
    "advancing health",
    "advancing education",
    "advancing social or public welfare",
    "advancing religion",
    "advancing culture",
    "promoting reconciliation, mutual respect and tolerance",
    "promoting or protecting human rights",
    "advancing the security or safety of Australia or the Australian public",
    "preventing or relieving the suffering of animals",
    "advancing the natural environment",
    "purposes beneficial to the general public and analogous to the others",
    "advancing public debate",
    "promoting the prevention or control of diseases in human beings (health promotion charity)",
    "public benevolent institution",
];

const SCOPE: &str = "Whether each ABN is on one week's copy of the ACNC Register of Australian charities, with the legal name, registration date and subtypes (ACNC Act 2012 s 25-5(5)) the copy shows. It does not read the live register, settle income tax exemption or DGR status, or show a registration's history.";

/// What one copy of the data cannot show; every result carries these.
const LIMITS: [&str; 5] = [
    "The data is a copy of the ACNC's weekly dataset, which lists only charities registered on its date (user notes para 4): a registration revoked before then, or made after it, is not shown.",
    "An ABN not in the data is not shown as a registered charity on that date; that does not establish that the entity is not registered. A charity may have its ABN withheld from the dataset (para 7), and a withheld subtype shows as absent (para 32). The ACNC Charity Register shows the current position.",
    "Registration with the ACNC is one condition of the income tax exemption for a charity; ITAA 1997 Division 50 sets the others, so this result does not settle exemption.",
    "Registration is not DGR endorsement: use dgr-check for a recipient's DGR entries.",
    "The registration date is the effective date of registration with the ACNC; it is 3 December 2012 for a charity registered for tax concessions before the ACNC began (paras 16 and 17).",
];

pub(crate) struct Register {
    header: Value,
    snapshot_date: String,
    entries: Vec<(String, String, Option<String>, Vec<usize>)>,
}

fn register() -> &'static Register {
    static REGISTER: OnceLock<Register> = OnceLock::new();
    REGISTER.get_or_init(|| parse_register(include_str!("../data/acnc-register.jsonl")))
}

/// The register from its JSONL text; panics on a malformed file, which the
/// tests read in full.
pub(crate) fn parse_register(text: &str) -> Register {
    let mut lines = text.lines();
    let header: Value = serde_json::from_str(lines.next().expect("a header")).expect("JSON header");
    let snapshot_date = header["snapshot_date"].as_str().expect("a snapshot date");
    assert!(
        dates::parse(snapshot_date).is_some(),
        "a YYYY-MM-DD snapshot date"
    );
    let mut entries: Vec<(String, String, Option<String>, Vec<usize>)> = Vec::new();
    for line in lines {
        let row: (String, String, Option<String>, Vec<usize>) =
            serde_json::from_str(line).expect("a register row");
        assert!(
            abn_digits(&row.0).as_deref() == Some(row.0.as_str()),
            "{}: an ABN",
            row.0
        );
        assert!(
            entries.last().is_none_or(|e| e.0 < row.0),
            "{}: sorted",
            row.0
        );
        assert!(
            row.2.as_deref().is_none_or(|d| dates::parse(d).is_some()),
            "{}: date",
            row.0
        );
        assert!(
            row.3.iter().all(|i| (1..=14).contains(i)),
            "{}: subtypes",
            row.0
        );
        assert!(row.3.is_sorted(), "{}: subtype order", row.0);
        entries.push(row);
    }
    assert_eq!(
        header["charities"].as_u64(),
        Some(entries.len() as u64),
        "charity count"
    );
    Register {
        snapshot_date: snapshot_date.to_string(),
        header,
        entries,
    }
}

fn refusal<T>(code: &'static str, message: String) -> Result<T, Refusal> {
    Err(Refusal::new(code, message))
}

/// The input's ABNs, checked, and its check date.
fn read_input(v: &Value) -> Result<(Vec<String>, String), Refusal> {
    if let Some(key) = v
        .as_object()
        .into_iter()
        .flat_map(|o| o.keys())
        .find(|k| *k != "abns" && *k != "as_at")
    {
        return refusal(
            "input.field_unknown",
            format!("{key} is not an input of {COMMAND}"),
        );
    }
    let Some(given) = v.get("abns") else {
        return refusal("input.missing_field", "abns is required".into());
    };
    let list = match given.as_array() {
        Some(list) if (1..=MAX_ABNS).contains(&list.len()) => list,
        _ => {
            return refusal(
                "charity_check.abns_invalid",
                format!("abns must be a list of 1 to {MAX_ABNS} ABNs"),
            );
        }
    };
    let mut abns = Vec::with_capacity(list.len());
    for (at, abn) in list.iter().enumerate() {
        match abn.as_str().and_then(abn_digits) {
            Some(digits) => abns.push(digits),
            None => {
                return refusal(
                    "input.abn_invalid",
                    format!(
                        "abns[{at}] {abn} is not 11 digits, optionally spaced, that pass the ABN check digit test ABN Lookup publishes"
                    ),
                );
            }
        }
    }
    let Some(as_at) = v.get("as_at") else {
        return refusal("input.missing_field", "as_at is required".into());
    };
    match as_at.as_str().filter(|d| dates::parse(d).is_some()) {
        Some(day) => Ok((abns, day.to_string())),
        None => refusal(
            "input.date_invalid",
            "as_at must be a YYYY-MM-DD date from 1900 to 2100".into(),
        ),
    }
}

/// The last check date the copy answers for.
fn stale_after(register: &Register) -> String {
    dates::format(dates::parse(&register.snapshot_date).expect("a checked date") + MAX_AGE_DAYS)
}

/// Refuses a check date the copy cannot answer for.
fn age_check(register: &Register, as_at: &str) -> Result<(), Refusal> {
    let snapshot = &register.snapshot_date;
    if as_at < snapshot.as_str() {
        return refusal(
            "charity_check.as_at_before_snapshot",
            format!(
                "as_at {as_at} is before the data date, {snapshot}: the copy shows the register on {snapshot} and cannot show it on an earlier date"
            ),
        );
    }
    let last = stale_after(register);
    if as_at > last.as_str() {
        return refusal(
            "charity_check.snapshot_stale",
            format!(
                "the data date, {snapshot}, is more than {MAX_AGE_DAYS} days before as_at {as_at} (the last date it answers for is {last}); rebuild data/acnc-register.jsonl from the current dataset with examples/acnc_register.rs, or check the ACNC Charity Register"
            ),
        );
    }
    Ok(())
}

fn subtypes(items: &[usize]) -> Value {
    items
        .iter()
        .map(|&item| json!({"item": item, "subtype": SUBTYPES[item - 1]}))
        .collect()
}

/// One ABN's reading of the copy.
fn lookup(register: &Register, at: usize, abn: &str) -> Value {
    let date = &register.snapshot_date;
    let spaced = format!("{} {} {} {}", &abn[..2], &abn[2..5], &abn[5..8], &abn[8..]);
    let mut out = match register.entries.binary_search_by(|e| e.0.as_str().cmp(abn)) {
        Ok(found) => {
            let (_, name, registered, items) = &register.entries[found];
            json!({
                "on_register_snapshot": true,
                "finding": format!("ABN {spaced} ({name}) is on the ACNC register in the data of {date}."),
                "legal_name": name,
                "registration_date": registered,
                "subtypes": subtypes(items),
            })
        }
        Err(_) => json!({
            "on_register_snapshot": false,
            "finding": format!("ABN {spaced} is not on the ACNC register in the data of {date}. That does not establish that the entity is not a registered charity: check the ACNC Charity Register."),
        }),
    };
    let object = out.as_object_mut().expect("an object");
    object.insert("index".into(), json!(at));
    object.insert("abn".into(), json!(abn));
    object.insert("snapshot_date".into(), json!(date));
    object.insert("limits".into(), json!(LIMITS));
    out
}

/// `charity-check`: `{"abns": [...], "as_at": "YYYY-MM-DD"}` gives each ABN's
/// reading of the register copy, in order.
pub fn check(v: &Value) -> Result<Value, Refusal> {
    check_with(register(), v)
}

pub(crate) fn check_with(register: &Register, v: &Value) -> Result<Value, Refusal> {
    let (abns, as_at) = read_input(v)?;
    age_check(register, &as_at)?;
    let results: Vec<Value> = abns
        .iter()
        .enumerate()
        .map(|(at, abn)| lookup(register, at, abn))
        .collect();
    let header = &register.header;
    Ok(json!({
        "ok": true,
        "engine": "australian-tax-calculators",
        "engine_version": engine::VERSION,
        "calculation": "charity_check",
        "as_at": as_at,
        "data": {
            "source": header["source"],
            "source_url": DATASET,
            "resource_url": header["resource_url"],
            "last_modified": header["last_modified"],
            "snapshot_date": register.snapshot_date,
            "answers_until": stale_after(register),
            "csv_sha256": header["csv_sha256"],
            "charities": register.entries.len(),
            "licence": header["licence"],
            "licence_url": LICENCE_URL,
            "copyright": header["copyright"],
            "attribution": header["attribution"],
            "changes": header["changes"],
        },
        "charities": results,
        "scope": SCOPE,
        "limits": LIMITS,
        "sources": SOURCES,
        "source_checked": "2026-11-24",
        "warnings": [
            "ACNC registration does not by itself settle income tax exemption or DGR status; this result never calls an entity exempt or eligible.",
            "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
        ],
    }))
}

#[cfg(test)]
mod tests;
