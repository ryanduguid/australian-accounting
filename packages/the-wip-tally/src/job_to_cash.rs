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
    if !extra.is_finite() || extra.lt(&decimal("0"))? {
        return fail("Cost increase must be finite and non-negative");
    }
    let source = here().join("job-to-cash.csv");
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
    let assumptions_text = std::fs::read_to_string(here().join("job-cash-assumptions.json"))
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
    write_new(
        &output.join("job-to-cash.json").fs(),
        // write_text: text mode, so native line endings.
        text_mode(&format!("{json}\n")).as_bytes(),
    )?;
    Ok(json)
}
