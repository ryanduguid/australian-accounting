// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The Excel workbook restates the checker's rules in worksheet formulas. Its
//! cached values are held to tests/data/workbook-sample.json by the
//! repository's tests/test_payday_workbook.py; this test holds the engine to
//! the same file, so a change on either side fails one of them.

use payday_super_checker::assess::{self, EXPOSED, Options, VERDICTS};
use payday_super_checker::csv_io::{self, Mapping, cents};
use payday_super_checker::decimal::Decimal;
use payday_super_checker::py::Date;
use payday_super_checker::sgc::scenario;
use payday_super_checker::{calendar, rates};
use serde_json::{Value, json};
use std::process::Command;

fn money(value: &Decimal) -> String {
    value.to_sci_string()
}

#[test]
fn the_engine_matches_the_frozen_workbook_sample() {
    let frozen: Value =
        serde_json::from_str(include_str!("data/workbook-sample.json")).expect("frozen sample");
    let as_at = Date::fromisoformat(frozen["as_at"].as_str().unwrap()).unwrap();
    let sample = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/sample_payrun.csv");
    let lines = csv_io::parse_rows(sample, &Mapping::default_mapping()).unwrap();
    let results = assess::assess(
        lines,
        &calendar::load_calendar(None).unwrap(),
        &rates::load_gic().unwrap(),
        as_at,
        Options {
            transition_allocation_confirmed: true,
            ..Options::default()
        },
    )
    .unwrap();
    let mut rows = Vec::new();
    let mut low_total = cents(None).unwrap();
    let mut high_total = cents(None).unwrap();
    for r in &results {
        let mut row = json!({
            "employee_id": r.line.employee_id,
            "verdict": r.verdict,
            "pathway": r.deadline.pathway,
            "final_due": r.deadline.due.map(|d| d.isoformat()),
            "unassessable_between": r.horizon_verdicts.as_ref().map(|(a, b)| format!("{a} or {b}")).unwrap_or_default(),
        });
        if EXPOSED.contains(&r.verdict.as_str()) {
            let uplift = r.uplift.as_ref().unwrap();
            let shortfall = cents(r.final_shortfall.as_ref()).unwrap();
            let nec = cents(r.nec.as_ref()).unwrap();
            let up_low = cents(Some(scenario(uplift, "clean_history", "vds_within_30d"))).unwrap();
            let up_high = cents(Some(scenario(uplift, "prior_history", "no_vds"))).unwrap();
            let low = shortfall.add(&nec).unwrap().add(&up_low).unwrap();
            let high = shortfall.add(&nec).unwrap().add(&up_high).unwrap();
            low_total = low_total.add(&low).unwrap();
            high_total = high_total.add(&high).unwrap();
            let fields = row.as_object_mut().unwrap();
            fields.insert("days_late".into(), json!(r.days_late));
            fields.insert("lateness_basis".into(), json!(r.lateness_basis));
            for (key, value) in [
                ("shortfall", &shortfall),
                ("nec", &nec),
                ("up_low", &up_low),
                ("up_high", &up_high),
                ("low", &low),
                ("high", &high),
            ] {
                fields.insert(key.into(), json!(money(value)));
            }
        }
        rows.push(row);
    }
    let counts: serde_json::Map<String, Value> = VERDICTS
        .iter()
        .map(|v| {
            (
                v.to_string(),
                json!(results.iter().filter(|r| r.verdict == *v).count()),
            )
        })
        .collect();
    let exposed = results.iter().filter(|r| r.is_exposed()).count();
    let actual = json!({
        "as_at": as_at.isoformat(),
        "transition_allocation_confirmed": true,
        "rows": rows,
        "summary": {
            "counts": counts,
            "exposed": exposed,
            "low": money(&low_total),
            "high": money(&high_total),
        },
    });
    assert_eq!(actual, frozen);
}

fn data(file: &str) -> String {
    format!("{}/tests/data/{file}", env!("CARGO_MANIFEST_DIR"))
}

/// A run of the checker over a fixture in tests/data, with its events file.
fn check_fixture(
    file: &str,
    events: Option<&str>,
    as_at: &str,
    scratch: &str,
) -> std::process::Output {
    let dir = std::env::temp_dir().join(format!("payday-{scratch}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let events: Vec<String> = events
        .map(|e| vec!["--events".to_string(), data(e)])
        .unwrap_or_default();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .arg(data(file))
        .args(["--as-at", as_at, "-o", "report.csv"])
        .args(&events)
        .current_dir(&dir)
        .output()
        .unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    out
}

/// The exit code the README gives the checker for each Review Checks status.
fn exit_code(overall: &Value) -> i32 {
    match overall.as_str().unwrap() {
        "PASS" => 0,
        "BLOCKED" => 1,
        "REVIEW" => 2,
        other => panic!("{other}"),
    }
}

/// The shipped sample has no returned row, so these fixtures carry them.
/// workbook-returned.json holds what Excel answered for them, and its
/// `produced_by` says how to reproduce that. These tests do not rerun the
/// workbook: they hold the engine to Excel's frozen answers, so a changed
/// formula fails nothing here until the file is reproduced.
fn frozen_returned() -> Value {
    serde_json::from_str(include_str!("data/workbook-returned.json")).expect("frozen rows")
}

#[test]
fn the_engine_matches_excel_on_returned_rows() {
    let frozen = frozen_returned();
    let as_at = frozen["as_at"].as_str().unwrap();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/workbook-returned.csv"
    );
    let results = assess::assess(
        csv_io::parse_rows(path, &Mapping::default_mapping()).unwrap(),
        &calendar::load_calendar(None).unwrap(),
        &rates::load_gic().unwrap(),
        Date::fromisoformat(as_at).unwrap(),
        Options::default(),
    )
    .unwrap();
    let rows: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "employee_id": r.line.employee_id,
                "verdict": r.verdict,
                "final_due": r.deadline.due.map(|d| d.isoformat()),
                "unassessable_between": r.horizon_verdicts.as_ref().map(|(a, b)| format!("{a} or {b}")).unwrap_or_default(),
                "exposed": u8::from(r.is_exposed()),
                "final_shortfall": r.final_shortfall.as_ref().map(money).unwrap_or_default(),
                // The checker read every row, so Excel must have too.
                "row_problem": "",
            })
        })
        .collect();
    assert_eq!(json!(rows), frozen["rows"]);
    let run = check_fixture("workbook-returned.csv", None, as_at, "returned");
    assert_eq!(run.status.code(), Some(exit_code(&frozen["overall"])));
}

#[test]
fn the_checker_refuses_the_rows_excel_blocks_for_the_same_reason() {
    let frozen = frozen_returned();
    let refused = &frozen["refused"];
    let run = check_fixture(
        "workbook-returned-refused.csv",
        None,
        frozen["as_at"].as_str().unwrap(),
        "refused",
    );
    assert_eq!(run.status.code(), Some(exit_code(&refused["overall"])));
    let reasons: Vec<String> = refused["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let reason = refused["reasons"][r["row_problem"].as_str().unwrap()]
                .as_str()
                .unwrap();
            format!("row {}: {reason}", r["row"])
        })
        .collect();
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        stderr.trim().strip_prefix("error: ").unwrap(),
        reasons.join("; ")
    );
}

/// workbook-events.json holds what Excel answered for lines that rows on the
/// Events sheet name; its `produced_by` says how to reproduce it.
fn frozen_events() -> Value {
    serde_json::from_str(include_str!("data/workbook-events.json")).expect("frozen rows")
}

/// The outcomes a row allows, best first: its verdict, or the pair an
/// undecided row lies between.
fn outcome_range(verdict: &str, between: &str) -> (usize, usize) {
    const ORDER: [&str; 3] = ["ON_TIME", "LATE", "UNPAID"];
    let rank = |v: &str| {
        ORDER
            .iter()
            .position(|o| *o == v)
            .unwrap_or_else(|| panic!("{v}"))
    };
    match between.split_once(" or ") {
        Some((worse, better)) => (rank(better), rank(worse)),
        None => (rank(verdict), rank(verdict)),
    }
}

/// The workbook does not weigh events, so a line an event names is UNKNOWN
/// there with no exposure, and its outer outcomes hold whatever the checker's
/// --events decides. A line no event names reads as the checker reads it.
#[test]
fn the_workbook_never_reads_an_event_line_better_than_the_checker() {
    let frozen = frozen_events();
    let as_at = frozen["as_at"].as_str().unwrap();
    let mut lines =
        csv_io::parse_rows(&data("workbook-events.csv"), &Mapping::default_mapping()).unwrap();
    csv_io::attach_events(&data("workbook-events-events.csv"), &mut lines).unwrap();
    let results = assess::assess(
        lines,
        &calendar::load_calendar(None).unwrap(),
        &rates::load_gic().unwrap(),
        Date::fromisoformat(as_at).unwrap(),
        Options::default(),
    )
    .unwrap();
    let excel = frozen["rows"].as_array().unwrap();
    assert_eq!(results.len(), excel.len());
    let mut checker_verdicts = Vec::new();
    for (index, (r, x)) in results.iter().zip(excel).enumerate() {
        assert_eq!(x["employee_id"], json!(r.line.employee_id));
        assert_eq!(x["final_due"], json!(r.deadline.due.map(|d| d.isoformat())));
        let between = r
            .horizon_verdicts
            .as_ref()
            .map(|(a, b)| format!("{a} or {b}"))
            .unwrap_or_default();
        if r.line.events.is_empty() {
            assert_eq!(x["verdict"], json!(r.verdict));
            assert_eq!(x["unassessable_between"], json!(between));
        } else {
            assert_eq!(x["verdict"], "UNKNOWN");
            assert_eq!(x["exposed"], 0);
            assert_eq!(x["final_shortfall"], "");
            let (best, worst) = outcome_range(
                x["verdict"].as_str().unwrap(),
                x["unassessable_between"].as_str().unwrap(),
            );
            let (b, w) = outcome_range(&r.verdict, &between);
            // A failure is printed. The employee identifier stays out of that
            // text; the frozen row's position identifies it.
            let row = index + 1;
            assert!(best <= b && w <= worst, "row {row}: {between}");
        }
        checker_verdicts.push(r.verdict.clone());
    }
    // One event line the checker decides and one it leaves open, so the
    // workbook's caution is tested both ways.
    assert_eq!(checker_verdicts, ["UNKNOWN", "ON_TIME", "ON_TIME"]);
    let run = check_fixture(
        "workbook-events.csv",
        Some("workbook-events-events.csv"),
        as_at,
        "events",
    );
    assert_eq!(run.status.code(), Some(2));
    assert_eq!(exit_code(&frozen["overall"]), 2);
}

#[test]
fn the_checker_refuses_the_events_excel_blocks_for_the_same_reason() {
    let frozen = frozen_events();
    let refused = &frozen["refused"];
    let run = check_fixture(
        "workbook-events-refused.csv",
        Some("workbook-events-refused-events.csv"),
        frozen["as_at"].as_str().unwrap(),
        "events-refused",
    );
    assert_eq!(run.status.code(), Some(exit_code(&refused["overall"])));
    let stderr = String::from_utf8_lossy(&run.stderr);
    let events = refused["events"].as_array().unwrap();
    assert!(
        stderr.contains(&format!("{} problem(s)", events.len())),
        "{stderr}"
    );
    for e in events {
        let reason = refused["reasons"][e["problem"].as_str().unwrap()]
            .as_str()
            .unwrap();
        let wanted = format!("event row {}: {reason}", e["row"]);
        assert!(stderr.contains(&wanted), "{wanted}: {stderr}");
    }
}
