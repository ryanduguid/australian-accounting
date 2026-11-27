// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The job-to-cash bridge behind `examples/job_to_cash.rs`: one fabricated
//! contract through the WIP engine to cash inputs, writing the same files as
//! the original `examples/job_to_cash.py`.

use std::path::{Path, PathBuf};

use crate::csv_io::{self, Failure};
use crate::csvread::Reader;
use crate::decimal::Decimal;
use crate::engine::{self, Schedule};
use crate::py::{self, OsStyle, PyPath};
use crate::report;
use serde_json::Value;

fn here() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")
}

fn fail<T>(message: impl Into<String>) -> Result<T, Failure> {
    Err(Failure::Message(message.into()))
}

/// `json.dumps(value, indent=2)` for a flat object of strings (ensure_ascii).
fn dumps(pairs: &[(&str, String)]) -> String {
    let quote = |s: &str| {
        let mut out = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\x08' => out.push_str("\\b"),
                '\x0c' => out.push_str("\\f"),
                c if (' '..='~').contains(&c) => out.push(c),
                c => {
                    let mut units = [0u16; 2];
                    for unit in c.encode_utf16(&mut units) {
                        out.push_str(&format!("\\u{unit:04x}"));
                    }
                }
            }
        }
        out.push('"');
        out
    };
    let body: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("  {}: {}", quote(k), quote(v)))
        .collect();
    format!("{{\n{}\n}}", body.join(",\n"))
}

/// Text-mode output: native line endings.
pub fn text_mode(text: &str) -> String {
    if cfg!(windows) {
        text.replace('\n', "\r\n")
    } else {
        text.to_string()
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Failure> {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut f| f.write_all(bytes))
        .map_err(|e| {
            Failure::Message(py::oserror_str(
                &e,
                &path.display().to_string(),
                OsStyle::Errno,
            ))
        })
}

/// `output.mkdir(parents=True, exist_ok=False)`.
fn make_folder(output: &PyPath) -> Result<(), Failure> {
    let path = output.fs();
    let first = std::fs::create_dir(&path);
    let result = match first {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let _ = std::fs::create_dir_all(output.parent().fs());
            std::fs::create_dir(&path)
        }
        other => other,
    };
    result.map_err(|e| Failure::Message(py::oserror_str(&e, &output.display(), OsStyle::Native)))
}

fn decimal(text: &str) -> Decimal {
    Decimal::parse(text).expect("fabricated assumption is a decimal")
}

/// Write the scenario pack under `output` (a new folder); returns the JSON summary.
pub fn run(output: &PyPath, extra: &Decimal) -> Result<String, Failure> {
    run_from(&here(), output, extra)
}

/// `run` with the scenario source and assumptions read from `examples`.
fn run_from(examples: &Path, output: &PyPath, extra: &Decimal) -> Result<String, Failure> {
    if !extra.is_finite() || extra.lt(&decimal("0"))? {
        return fail("Cost increase must be finite and non-negative");
    }
    let source = examples.join("job-to-cash.csv");
    let source_bytes = std::fs::read(&source).map_err(|e| {
        Failure::Message(py::oserror_str(
            &e,
            &source.display().to_string(),
            OsStyle::Errno,
        ))
    })?;
    let mapping = csv_io::load_mapping(None)?;
    let contracts = csv_io::read_contracts(&source.display().to_string(), &source_bytes, &mapping)?;
    if contracts.len() != 1 {
        return fail("The example requires one contract");
    }
    let mut contract = contracts[0].clone();
    contract.estimated_cost_to_complete = contract.estimated_cost_to_complete.add(extra)?;
    let position = engine::measure(&contract)?;
    let assumptions_text = std::fs::read_to_string(examples.join("job-cash-assumptions.json"))
        .map_err(|e| Failure::Message(e.to_string()))?;
    let assumptions: Value =
        serde_json::from_str(&assumptions_text).map_err(|e| Failure::Message(e.to_string()))?;
    let field = |key: &str| assumptions[key].as_str().unwrap_or_default().to_string();
    let receipts = decimal(&field("receipts_to_date"));
    let paid = decimal(&field("costs_paid_to_date"));
    let certified_gross = contract
        .certified_billings
        .add(&position.gst_on_certified_billings)?;
    let retention_gross = contract
        .retention_withheld
        .add(&position.gst_on_retention)?;
    let collectible = certified_gross.sub(&retention_gross)?.sub(&receipts)?;
    if collectible.lt(&decimal("0"))?
        || contract
            .committed_outstanding
            .gt(&contract.estimated_cost_to_complete)?
    {
        return fail("Cash or commitment assumptions do not reconcile");
    }
    // The committed amount is already inside remaining cost; never add it twice.
    let future_cost = contract
        .estimated_cost_to_complete
        .mul(&decimal(&field("future_cost_cash_multiplier")))?;
    make_folder(output)?;

    // Persist the exact scenario source so the existing pack verifier can rebuild it.
    let mut reader = Reader::new(&source_bytes);
    let fields = reader
        .next_row()
        .map_err(|e| Failure::Message(e.to_string()))?
        .unwrap_or_default();
    let mut rows = Vec::new();
    while let Some(row) = reader
        .next_row()
        .map_err(|e| Failure::Message(e.to_string()))?
    {
        if !row.is_empty() {
            rows.push(row);
        }
    }
    let etc_column = fields
        .iter()
        .rposition(|f| f == "estimated_cost_to_complete")
        .expect("the example names the column");
    rows[0][etc_column] = contract.estimated_cost_to_complete.to_string();
    let mut scenario_text = String::new();
    report::csv_line_ending(&fields, &mut scenario_text, "\r\n");
    for row in &rows {
        report::csv_line_ending(row, &mut scenario_text, "\r\n");
    }
    let scenario = output.join("contracts.csv");
    write_new(&scenario.fs(), scenario_text.as_bytes())?;

    let schedule = Schedule {
        as_at: "2026-09-30".into(),
        positions: vec![position.clone()],
    };
    let schedule_path = output.join("wip-schedule.csv");
    report::atomic_write(&schedule_path, &report::schedule_csv(&schedule))?;
    let schedule_bytes = report::read_bytes(&schedule_path)?;
    let scenario_bytes = report::read_bytes(&scenario)?;
    let pack = report::build_review_pack(
        &schedule_path,
        &schedule_bytes,
        scenario.name(),
        &scenario_bytes,
        &schedule,
    )?;
    report::atomic_write(&output.join("wip-review.md"), &pack)?;

    let cash_fields: Vec<String> = ["id", "date", "receipt", "payment", "evidence"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let cash_rows = [
        [
            "CERTIFIED-RECEIVABLE".to_string(),
            field("certified_collection_date"),
            collectible.to_string(),
            "0".into(),
            field("certified_collection_evidence"),
        ],
        [
            "RETENTION".into(),
            field("retention_collection_date"),
            retention_gross.to_string(),
            "0".into(),
            field("retention_release_assumption"),
        ],
        [
            "REMAINING-COST".into(),
            field("remaining_cost_payment_date"),
            "0".into(),
            future_cost.to_string(),
            field("remaining_cost_evidence"),
        ],
    ];
    let mut cash_text = String::new();
    report::csv_line_ending(&cash_fields, &mut cash_text, "\r\n");
    for row in &cash_rows {
        report::csv_line_ending(row, &mut cash_text, "\r\n");
    }
    let cash = output.join("cash-assumptions.csv");
    write_new(&cash.fs(), cash_text.as_bytes())?;

    let result = [
        ("contract_id", contract.contract_id.clone()),
        ("currency", "AUD".into()),
        ("as_at", "2026-09-30".into()),
        (
            "cash_assumptions_sha256",
            report::sha256_hex(cash_text.as_bytes()),
        ),
        ("source_sha256", report::sha256_hex(&source_bytes)),
        ("scenario_source_sha256", report::sha256_hex(&scenario_bytes)),
        (
            "gross_profit_at_completion",
            position.gross_profit_at_completion.to_string(),
        ),
        ("revenue_to_date", position.revenue_to_date.to_string()),
        ("certified_billings", contract.certified_billings.to_string()),
        ("contract_asset", position.contract_asset.to_string()),
        ("uncertified_claims", contract.uncertified_claims.to_string()),
        ("retention_gross", retention_gross.to_string()),
        ("receipts_to_date", receipts.to_string()),
        ("costs_paid_to_date", paid.to_string()),
        ("cash_to_date", receipts.sub(&paid)?.to_string()),
        ("collectible_certified_balance", collectible.to_string()),
        ("future_cost_cash", future_cost.to_string()),
        (
            "committed_in_remaining_cost",
            contract.committed_outstanding.to_string(),
        ),
        (
            "scope",
            "Fabricated bridge under supplied assumptions. Uncertified and future unbilled claims are excluded from cash. Retention date is a planning assumption.".into(),
        ),
    ];
    let json = dumps(&result);
    // The folder was created empty above, so this name cannot already exist.
    let written = write_new(
        &output.join("job-to-cash.json").fs(),
        // write_text: text mode, so native line endings.
        text_mode(&format!("{json}\n")).as_bytes(),
    );
    written?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wip-j2c-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn zero() -> Decimal {
        decimal("0")
    }

    fn message<T: std::fmt::Debug>(result: Result<T, Failure>) -> String {
        match result {
            Err(Failure::Message(m)) => m,
            other => panic!("not a message: {other:?}"),
        }
    }

    /// An examples folder holding the shipped scenario with `csv` and `assumptions` edited.
    fn examples(
        name: &str,
        csv: impl Fn(String) -> String,
        assumptions: impl Fn(String) -> String,
    ) -> PathBuf {
        let dir = scratch(name);
        let read = |file: &str| std::fs::read_to_string(here().join(file)).unwrap();
        std::fs::write(dir.join("job-to-cash.csv"), csv(read("job-to-cash.csv"))).unwrap();
        std::fs::write(
            dir.join("job-cash-assumptions.json"),
            assumptions(read("job-cash-assumptions.json")),
        )
        .unwrap();
        dir
    }

    #[test]
    fn dumps_escapes_as_json_dumps_with_ensure_ascii() {
        // json.dumps({"k\"\\": "a\n\r\t\b\fé\U0001f600~ "}, indent=2)
        assert_eq!(
            dumps(&[("k\"\\", "a\n\r\t\u{8}\u{c}\u{e9}\u{1f600}~ ".into())]),
            "{\n  \"k\\\"\\\\\": \"a\\n\\r\\t\\b\\f\\u00e9\\ud83d\\ude00~ \"\n}"
        );
    }

    #[test]
    fn a_negative_or_infinite_cost_increase_is_refused() {
        let out = PyPath::new(&scratch("neg").join("out").to_string_lossy());
        for extra in ["-1", "Infinity"] {
            assert_eq!(
                message(run(&out, &decimal(extra))),
                "Cost increase must be finite and non-negative"
            );
        }
    }

    #[test]
    fn a_missing_scenario_source_is_reported_as_python_raised_it() {
        let dir = scratch("missing");
        let out = PyPath::new(&dir.join("out").to_string_lossy());
        let source = dir.join("job-to-cash.csv");
        let got = message(run_from(&dir, &out, &zero()));
        assert!(
            got.starts_with("[Errno 2] ") && got.ends_with(&format!("{}'", source.display())),
            "{got}"
        );
    }

    #[test]
    fn a_scenario_with_two_contracts_is_refused() {
        let dir = examples(
            "two",
            |csv| {
                let row = csv
                    .lines()
                    .nth(1)
                    .unwrap()
                    .replace("HUNTER-CIVIL-01", "HUNTER-CIVIL-02");
                format!("{csv}{row}\n")
            },
            |json| json,
        );
        let out = PyPath::new(&dir.join("out").to_string_lossy());
        assert_eq!(
            message(run_from(&dir, &out, &zero())),
            "The example requires one contract"
        );
    }

    #[test]
    fn receipts_beyond_the_certified_balance_do_not_reconcile() {
        let dir = examples(
            "receipts",
            |csv| csv,
            |json| json.replace("\"300000\"", "\"900000\""),
        );
        let out = PyPath::new(&dir.join("out").to_string_lossy());
        assert_eq!(
            message(run_from(&dir, &out, &zero())),
            "Cash or commitment assumptions do not reconcile"
        );
        assert!(!dir.join("out").exists(), "nothing is written");
    }

    #[test]
    fn missing_parent_folders_are_created_as_mkdir_parents_does() {
        let dir = scratch("parents");
        let out = PyPath::new(&dir.join("a").join("b").to_string_lossy());
        let json = run(&out, &zero()).unwrap();
        assert!(json.contains("\"contract_id\": \"HUNTER-CIVIL-01\""));
        for name in [
            "contracts.csv",
            "wip-schedule.csv",
            "wip-review.md",
            "cash-assumptions.csv",
            "job-to-cash.json",
        ] {
            assert!(dir.join("a").join("b").join(name).is_file(), "{name}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn an_existing_file_is_not_overwritten() {
        let dir = scratch("exists");
        let path = dir.join("taken.txt");
        std::fs::write(&path, "kept").unwrap();
        assert_eq!(
            message(write_new(&path, b"new")),
            format!("[Errno 17] File exists: '{}'", path.display())
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "kept");
    }
}
