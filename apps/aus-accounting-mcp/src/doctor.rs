// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `aus-accounting-mcp doctor`: what this install holds and whether its folders work,
//! as the Python server's doctor.py reports it.
use serde_json::{Value, json};
use std::fmt::Write;

pub const HELP: &str = "Print the installed version, the periods its bundled data records and each configured retrieval folder with its row count. Exits 1 if a configured folder is broken.";
const NOT_CURRENT: &str =
    "Bundled data periods (what the release records, not that a figure is current):";
const ENVIRONMENT: &str = "Retrieval folders (from this command's environment; an MCP client passes the ones in its own server configuration):";
const NOT_CONFIGURED: &str = "The folder is set in the MCP client's configuration for this server: tell the user, and do not retry until they have set it.";
const VARIABLES: [&str; 3] = [
    "AUS_ACCOUNTING_LIBRARY_ROOT",
    "AUS_ACCOUNTING_CORPUS_ROOT",
    "AUS_ACCOUNTING_RULINGS_ROOT",
];

fn periods(dataset: &Value) -> String {
    match &dataset["effective_periods"] {
        Value::Array(periods) => format!(
            "{} to {}",
            periods[0]["start"].as_str().unwrap(),
            periods[periods.len() - 1]["end"].as_str().unwrap()
        ),
        other => other.as_str().unwrap_or_default().to_owned(),
    }
}

/// What a working folder holds, or why it does not work.
fn describe(variable: &str, found: &Value) -> Result<String, String> {
    let count = |key: &str| found[key].to_string();
    match variable {
        "AUS_ACCOUNTING_LIBRARY_ROOT" => {
            let skipped = found["skipped"].as_array().unwrap();
            if let Some(first) = skipped.first() {
                return Err(format!(
                    "Markdown files search skips: {}, first {}.",
                    skipped.len(),
                    first.as_str().unwrap()
                ));
            }
            Ok(format!(
                "{} lines in {} Markdown files",
                count("rows"),
                count("files")
            ))
        }
        "AUS_ACCOUNTING_CORPUS_ROOT" => {
            if found["unreadable"] != 0 {
                return Err(format!(
                    "lines a search cannot return: {}.",
                    count("unreadable")
                ));
            }
            if found["skipped_entries"] != 0 {
                return Err(format!(
                    "entries under markdown holding no index a search reads: {}.",
                    count("skipped_entries")
                ));
            }
            match found["rate_rows"].as_u64() {
                None => {
                    Err("it has no usable rates/rates.jsonl, so search_tax_rates will fail.".into())
                }
                Some(0) => Err(
                    "its rates index holds no rows, so a rate search would come back empty.".into(),
                ),
                Some(rates) => Ok(format!(
                    "{} section rows in {} title indexes, {rates} rate rows",
                    count("rows"),
                    count("indexes")
                )),
            }
        }
        _ => Ok(format!(
            "{} paragraph rows from {} documents in {} runs, {} documents withheld",
            count("rows"),
            count("documents"),
            count("runs"),
            count("withheld")
        )),
    }
}

/// Whether the folder works, and a line saying what it holds or why it does not.
pub(crate) fn judged(variable: &str, census: Result<Value, String>) -> (bool, String) {
    let outcome = census.and_then(|found| describe(variable, &found).map(|held| (found, held)));
    match outcome {
        Err(error) => (false, error.replace(NOT_CONFIGURED, "").trim().to_owned()),
        Ok((found, _)) if found["rows"] == 0 => (
            false,
            "it holds no rows, so every search would come back empty.".into(),
        ),
        Ok((_, held)) => (true, held),
    }
}

/// The report and the exit code: 1 if a configured folder is broken.
pub fn report() -> (String, i32) {
    let freshness = crate::adapters::freshness::report(&json!({})).expect("bundled data freshness");
    let mut out = String::new();
    let _ = writeln!(out, "aus-accounting-mcp {}\n", crate::VERSION);
    let _ = writeln!(out, "{NOT_CURRENT}");
    for dataset in freshness["datasets"].as_array().unwrap() {
        let _ = writeln!(
            out,
            "  {}: {}",
            dataset["dataset"].as_str().unwrap(),
            periods(dataset)
        );
    }
    let _ = writeln!(out, "\n{ENVIRONMENT}");
    let mut broken = 0;
    for variable in VARIABLES {
        let root = std::env::var_os(variable).filter(|v| !v.is_empty());
        let Some(root) = root else {
            let _ = writeln!(out, "  {variable}: not set");
            continue;
        };
        let (ok, line) = judged(variable, crate::retrieval::census(variable));
        broken += usize::from(!ok);
        let state = if ok { "OK" } else { "BROKEN" };
        let _ = writeln!(
            out,
            "  {variable}: {state}: {}: {line}",
            root.to_string_lossy()
        );
    }
    if broken > 0 {
        let noun = if broken == 1 {
            "folder is"
        } else {
            "folders are"
        };
        let _ = writeln!(out, "\n{broken} configured {noun} broken.");
    }
    (out, i32::from(broken > 0))
}
