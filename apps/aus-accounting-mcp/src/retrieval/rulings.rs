// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Validated, immutable operator runs. Metadata signatures avoid repeat validation.
mod manifest;
mod query;
mod runs;

use super::Lines;
use super::{Context, POLICY};
#[cfg(test)]
use jiff::civil::Date;
use num_bigint::BigInt;
use payday_super_checker::{py, pyre::Regex};
#[cfg(test)]
use query::provenance;
use query::{read, search};
use runs::catalogue;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{LazyLock, Mutex},
    time::SystemTime,
};

fn policy(name: &str) -> &'static str {
    POLICY["rulings"][name].as_str().unwrap()
}
static RUN_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("RUN_NAME")));
static DOCID: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("DOCID")));
static ROW_REF: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("ROW_REF")));
static ISO_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("ISO_DATE")));
static SHA256: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("SHA256")));

#[derive(Clone)]
struct Run {
    name: String,
    path: PathBuf,
    date: String,
    documents: HashSet<String>,
    excluded: HashSet<String>,
    notice: String,
    url: String,
}
struct Catalogue {
    runs: Vec<Run>,
    owner: HashMap<String, String>,
    withheld: HashMap<String, String>,
}
type Signature = (u64, SystemTime, u64, SystemTime);
static VALIDATED: LazyLock<Mutex<HashMap<PathBuf, (Signature, Run)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
fn canonical(id: &str) -> String {
    id.trim_matches(py::is_space)
        .trim_end_matches('/')
        .to_uppercase()
}
fn family_priority(row: &Value) -> usize {
    let families = POLICY["rulings"]["FAMILIES"].as_array().unwrap();
    families
        .iter()
        .position(|f| *f == row["family"])
        .unwrap_or(families.len())
}
fn count(value: &Value) -> Option<BigInt> {
    let n = value.as_number()?.to_string();
    if n.strip_prefix('-')
        .unwrap_or(&n)
        .bytes()
        .all(|b| b.is_ascii_digit())
    {
        n.parse().ok()
    } else {
        None
    }
}
fn withdrawal(row: &Value) -> Result<(String, String), String> {
    let status = row.get("withdrawal_status").unwrap_or(&Value::Null);
    let status = if row.get("withdrawal_status").is_none() {
        Some("UNKNOWN")
    } else {
        status.as_str()
    };
    let notice = if row.get("withdrawal_notice").is_none() {
        Some("")
    } else {
        row["withdrawal_notice"].as_str()
    };
    match (status, notice) {
        (Some(status @ ("UNKNOWN" | "NOTICE_PRESENT")), Some(notice))
            if (status == "NOTICE_PRESENT") == !notice.trim_matches(py::is_space).is_empty() =>
        {
            Ok((status.into(), notice.into()))
        }
        _ => Err("Rulings record has inconsistent withdrawal metadata.".into()),
    }
}

#[test]
fn provenance_matches_original_at_large_catalogue_sizes() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native/calvino-followups.json"
    ))
    .unwrap();
    for case in fixture["provenance_scale"].as_array().unwrap() {
        let count = case["document_count"].as_u64().unwrap() as usize;
        let catalogue = Catalogue {
            runs: (0..500)
                .map(|i| Run {
                    name: format!("run-{i:03}"),
                    path: PathBuf::new(),
                    date: Date::new(2025, 1, 1)
                        .unwrap()
                        .checked_add(jiff::SignedDuration::from_hours(24 * i))
                        .unwrap()
                        .to_string(),
                    documents: HashSet::new(),
                    excluded: HashSet::new(),
                    notice: format!("Synthetic notice {}", i % 3),
                    url: format!("https://example.invalid/notice/{}", i % 3),
                })
                .collect(),
            owner: (0..count)
                .map(|i| (format!("DOC{i:06}"), format!("run-{:03}", (i % 5) * 100)))
                .collect(),
            withheld: HashMap::from([("WITHHELD".into(), "run-499".into())]),
        };
        let start = std::time::Instant::now();
        let actual = provenance(&catalogue);
        println!(
            "provenance documents={count} nanoseconds={}",
            start.elapsed().as_nanos()
        );
        assert_eq!(actual, case["expected"]);
    }
}

/// What the configured runs serve, for `aus-accounting-mcp doctor`.
pub(super) fn census(context: &Context) -> Result<Value, String> {
    let catalogue = catalogue(context)?;
    let mut rows = 0;
    for run in &catalogue.runs {
        for raw in Lines::open(&run.path).map_err(|e| e.to_string())? {
            let row = super::parse_json(&raw.map_err(|e| e.to_string())?)?;
            if !row.is_object() {
                return Err("a rulings row is not a JSON object.".into());
            }
            let key = canonical(row["docid"].as_str().unwrap_or(""));
            rows += usize::from(catalogue.owner.get(&key) == Some(&run.name));
        }
    }
    Ok(
        json!({"runs":catalogue.runs.len(),"documents":catalogue.owner.len(),
        "withheld":catalogue.withheld.len(),"rows":rows}),
    )
}

pub(super) fn evaluate(context: &Context, name: &str, input: &Value) -> Result<Value, String> {
    if name == "search_ato_rulings" {
        search(context, input)
    } else {
        read(context, input)
    }
}
