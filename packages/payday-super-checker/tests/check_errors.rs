// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The checker's refusals after the contribution CSV has been read: an events
//! file it cannot use, a mapping value that is not a column heading, and a
//! report or evidence pack it cannot write.

use std::path::{Path, PathBuf};

use payday_super_checker::Failure;
use payday_super_checker::cli::dispatch;
use payday_super_checker::csv_io::{Mapping, parse_rows};

const EVENTS_HEADER: &str = "employee_id,payment_date,event,event_date,amount\n";

/// A folder for one test, under the build's own scratch directory, holding
/// the sample pay run as `in.csv`.
fn folder(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("check-errors-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample_payrun.csv");
    std::fs::copy(sample, dir.join("in.csv")).unwrap();
    dir
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// (exit code, stdout, stderr) of a check of `in.csv` with `extra` arguments.
fn check(dir: &Path, command: &[&str], extra: &[String]) -> (i32, String, String) {
    let mut args: Vec<String> = command.iter().map(|s| (*s).to_string()).collect();
    args.push(text(&dir.join("in.csv")));
    args.extend(
        ["--as-at", "2026-11-27", "--confirm-transition-allocation"]
            .iter()
            .map(|s| (*s).to_string()),
    );
    args.extend(extra.iter().cloned());
    dispatch(&args)
}

/// The error a check with this events file gives, with the report it did not write.
fn events_error(name: &str, events: &[u8]) -> (String, String) {
    let dir = folder(name);
    let path = dir.join("events.csv");
    std::fs::write(&path, events).unwrap();
    let output = dir.join("report.csv");
    let (code, out, err) = check(
        &dir,
        &[],
        &["--events".into(), text(&path), "-o".into(), text(&output)],
    );
    assert_eq!((code, out.as_str()), (1, ""), "{err}");
    assert!(!output.exists(), "a refused run writes no report");
    (err, text(&path))
}

#[test]
fn events_that_are_not_utf8_are_refused_with_the_byte() {
    let (err, path) = events_error("events-latin1", b"\xffemployee_id\n");
    assert_eq!(
        err,
        format!(
            "error: {path} is not UTF-8 text (byte 0xff at position 0). Excel's plain 'CSV' \
             export uses the Windows code page: re-save it as 'CSV UTF-8 (Comma delimited)' \
             and run again.\n"
        )
    );
}

#[test]
fn an_empty_events_file_has_no_header_row() {
    let (err, path) = events_error("events-empty", b"");
    assert_eq!(err, format!("error: {path} has no header row\n"));
}

#[test]
fn an_events_header_with_no_rows_is_refused() {
    let (err, path) = events_error("events-header-only", EVENTS_HEADER.as_bytes());
    assert_eq!(
        err,
        format!("error: {path} has a header but no data rows\n")
    );
}

#[test]
fn a_repeated_events_column_is_named_once_as_repeated() {
    let (err, path) = events_error("events-repeated", b"employee_id, employee_id\n");
    assert_eq!(
        err,
        format!(
            "error: {path} needs the columns ['employee_id', 'payment_date', 'event', \
             'event_date', 'amount'] with optional ['receipt_date', 'notified_date']. Unknown \
             or repeated: ['employee_id']; missing: ['payment_date', 'event', 'event_date', \
             'amount']\n"
        )
    );
}

#[test]
fn more_than_twenty_event_problems_are_counted_not_listed() {
    let mut events = EVENTS_HEADER.to_string();
    for _ in 0..22 {
        events.push_str("EMP001,2026-07-09,bogus,2026-07-10,1\n");
    }
    let (err, path) = events_error("events-many", events.as_bytes());
    assert!(
        err.starts_with(&format!("error: 22 problem(s) in {path}:\n")),
        "{err}"
    );
    assert!(err.contains("  - event row 21: event 'bogus' is not received, returned or allocated ... and 2 more problem(s).\n"), "{err}");
    assert!(!err.contains("event row 22"), "{err}");
}

#[test]
fn an_events_field_over_the_csv_limit_is_python_csv_error() {
    let events = format!(
        "{EVENTS_HEADER}EMP001,2026-07-09,{},2026-07-10,1\n",
        "x".repeat(131_073)
    );
    let (err, _) = events_error("events-field-limit", events.as_bytes());
    assert_eq!(
        err,
        "error: _csv.Error: field larger than field limit (131072)\n"
    );
}

#[test]
fn a_mapping_value_that_is_not_text_is_refused() {
    let dir = folder("mapping-number");
    let mapping = dir.join("mapping.json");
    std::fs::write(&mapping, r#"{"employee_id": 5, "sg_amount": " "}"#).unwrap();
    let shown = text(&mapping);
    let (code, _, err) = check(&dir, &[], &["--mapping-file".into(), shown.clone()]);
    assert_eq!(code, 1);
    assert_eq!(
        err,
        format!(
            "error: {shown} maps ['employee_id', 'sg_amount'] to something that is not a \
             column name. Each value must be the column heading as it appears in your CSV.\n"
        )
    );
}

#[test]
fn a_mapping_without_a_required_field_is_python_key_error() {
    let dir = folder("mapping-empty");
    let err = parse_rows(&text(&dir.join("in.csv")), &Mapping::default()).unwrap_err();
    assert!(
        matches!(&err, Failure::Uncaught(m) if m == "KeyError: 'employee_id'"),
        "{err:?}"
    );
}

#[test]
fn a_report_under_a_file_is_refused_as_unwritable() {
    let dir = folder("report-under-file");
    std::fs::write(dir.join("afile"), "").unwrap();
    let output = text(&dir.join("afile").join("report.csv"));
    let (code, out, err) = check(&dir, &[], &["-o".into(), output.clone()]);
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(
        err.starts_with(&format!("error: cannot write {output}: ")),
        "{err}"
    );
}

#[test]
fn an_evidence_pack_under_a_file_is_refused() {
    let dir = folder("pack-under-file");
    std::fs::write(dir.join("afile"), "").unwrap();
    let output = text(&dir.join("afile").join("pack"));
    let (code, out, err) = check(&dir, &["evidence-pack"], &["-o".into(), output.clone()]);
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(err.starts_with("error: "), "{err}");
    assert!(err.contains("pack"), "{err}");
    assert!(!err.contains("Traceback"), "{err}");
}

// Expected texts below are the pre-port Python engine's (paydaysuper at 5874033^).

#[test]
fn the_json_api_renders_the_review_pack_of_a_report() {
    let dir = folder("api-render-pack");
    let output = dir.join("report.csv");
    let (code, _, err) = check(&dir, &[], &["-o".into(), text(&output)]);
    assert_eq!(code, 2, "{err}");
    let report = std::fs::read_to_string(&output).unwrap();
    let input = serde_json::json!({
        "text": report,
        "source": "report.csv",
        "identifiers_omitted": false,
        "decision_log": false,
    });
    let pack = payday_super_checker::api::evaluate("render-practitioner-pack", &input).unwrap();
    let pack = pack.as_str().unwrap();
    assert!(
        pack.starts_with(
            "# Payday Super Practitioner Review Pack\n\n**Status: ATTENTION REQUIRED.**\n"
        ),
        "{pack}"
    );
    assert!(pack.contains(
        "| ON_TIME | 5 |\n| AT_RISK | 1 |\n| LATE | 2 |\n| UNPAID | 1 |\n| UNKNOWN | 0 |\n| SKIPPED | 1 |\n"
    ));
    assert!(pack.contains("range across LATE and UNPAID rows is **$815.43 to $1,304.69**."));
}

#[test]
fn a_review_pack_source_must_have_a_csv_extension() {
    use payday_super_checker::practitioner_pack::load_report_snapshot;
    // os.path.splitext: no dot, or only a leading one, is no extension.
    for (name, message) in [
        ("report", "report must be a .csv checker report"),
        (".csv", ".csv must be a .csv checker report"),
    ] {
        let failure = load_report_snapshot(name).err().unwrap();
        assert!(matches!(failure, Failure::Value(_)));
        assert_eq!(failure.message(), message);
    }
}

#[test]
fn report_bytes_that_are_not_utf8_are_refused() {
    use payday_super_checker::practitioner_pack::parse_report_snapshot;
    for data in [&b"\xff"[..], b"\xef\xbb\xbf\xff"] {
        let failure = parse_report_snapshot(data, "r.csv", false).err().unwrap();
        assert_eq!(failure.message(), "r.csv is not UTF-8 CSV");
    }
}

#[test]
fn source_rows_order_by_value_not_by_text() {
    use payday_super_checker::practitioner_pack::RowNumber;
    let row = |n: &str| RowNumber(n.into());
    assert!(row("9") < row("10"));
    assert!(row("12") > row("11"));
    assert_eq!(
        row("10").partial_cmp(&row("10")),
        Some(std::cmp::Ordering::Equal)
    );
}
