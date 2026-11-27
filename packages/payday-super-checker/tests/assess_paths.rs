// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Assessment paths the sample pay run does not take. Each expected text is
//! what the pre-port Python engine (`paydaysuper` at 5874033^) printed or
//! wrote for the same input under CPython.

use std::path::{Path, PathBuf};

use payday_super_checker::cli::dispatch;

const HEADER: &str = "employee_id,payment_date,sg_amount,remitted_date,fund_received_date,\
first_contribution_to_fund,out_of_cycle,next_standard_payday,defined_benefit,\
remitted_amount,matched_amount\n";

/// Writes `rows` under the header in a fresh folder; returns the folder.
fn input(name: &str, rows: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("assess-paths-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("in.csv"), format!("{HEADER}{rows}")).unwrap();
    dir
}

/// (exit code, stdout, stderr) of a check of `in.csv` as at 27 Nov 2026.
fn check(dir: &Path) -> (i32, String, String) {
    let text = |p: PathBuf| p.to_string_lossy().into_owned();
    dispatch(&[
        text(dir.join("in.csv")),
        "--as-at".into(),
        "2026-11-27".into(),
        "-o".into(),
        text(dir.join("report.csv")),
    ])
}

#[test]
fn more_than_ten_listed_rows_are_counted_after_the_tenth() {
    let rows: String = (1..=11)
        .map(|d| format!("E{d:02},2026-06-{d:02},100.00,,,no,no,,no,,\n"))
        .collect();
    let dir = input("eleven", &rows);
    let (code, out, err) = check(&dir);
    assert_eq!((code, out.as_str()), (1, ""));
    assert_eq!(
        err,
        "error: 11 row(s) have a QE day before 1 Jul 2026 (rows 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 \
         and 1 more; earliest 2026-06-01): the old quarterly SG law applies to them and this \
         tool covers payday super only. Remove them and run again.\n"
    );
    assert!(!dir.join("report.csv").exists());
}

#[test]
fn lines_that_differ_only_in_amount_are_not_duplicates() {
    let dir = input(
        "amounts",
        "EMP001,2026-08-06,612.00,2026-08-10,2026-08-12,no,no,,no,,612.00\n\
         EMP001,2026-08-06,300.00,2026-08-10,2026-08-12,no,no,,no,,300.00\n",
    );
    let (code, out, err) = check(&dir);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("ON_TIME: 2  AT_RISK: 0  LATE: 0  UNPAID: 0  UNKNOWN: 0  SKIPPED: 0"));
    let report = std::fs::read_to_string(dir.join("report.csv")).unwrap();
    let lines: Vec<&str> = report.lines().skip(1).take(2).collect();
    assert_eq!(
        lines,
        [
            "2,EMP001,2026-08-06,USUAL_7BD,2026-08-17,ON_TIME,,,612.00,,,,,,,,,",
            "3,EMP001,2026-08-06,USUAL_7BD,2026-08-17,ON_TIME,,,300.00,,,,,,,,,",
        ]
    );
}
