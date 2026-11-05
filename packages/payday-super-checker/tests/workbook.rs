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

/// A run of the checker over a fixture in tests/data.
fn check_fixture(file: &str, as_at: &str, scratch: &str) -> std::process::Output {
    let dir = std::env::temp_dir().join(format!("payday-{scratch}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .arg(format!("{}/tests/data/{file}", env!("CARGO_MANIFEST_DIR")))
        .args(["--as-at", as_at, "-o", "report.csv"])
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
    let run = check_fixture("workbook-returned.csv", as_at, "returned");
    assert_eq!(run.status.code(), Some(exit_code(&frozen["overall"])));
}

#[test]
fn the_checker_refuses_the_rows_excel_blocks_for_the_same_reason() {
    let frozen = frozen_returned();
    let refused = &frozen["refused"];
    let run = check_fixture(
        "workbook-returned-refused.csv",
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
