// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `usi-check` through the binary, on the fabricated extract in the fuzz corpus.
//! It writes one report and reports through its exit code; nothing else runs.

use std::path::{Path, PathBuf};
use std::process::Command;

fn extract() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/usi_extract")
}

/// A folder for one test, under the build's own scratch directory.
fn folder(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// (exit code, report text, stdout, stderr).
fn run(dir: &Path, input: &str, extract: &Path, extra: &[&str]) -> (i32, String, String, String) {
    let csv = dir.join("input.csv");
    let report = dir.join("usi-check.csv");
    std::fs::write(&csv, input).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .arg("usi-check")
        .arg(&csv)
        .arg("--usi-extract")
        .arg(extract)
        .args(["--as-at", "2026-11-05", "-o"])
        .arg(&report)
        .args(extra)
        .output()
        .unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap().replace("\r\n", "\n");
    (
        out.status.code().unwrap(),
        std::fs::read_to_string(&report)
            .unwrap_or_default()
            .replace("\r\n", "\n"),
        text(out.stdout),
        text(out.stderr),
    )
}

#[test]
fn all_listed_exits_0_and_says_what_listed_does_not_mean() {
    let dir = folder("all-listed");
    let (code, report, out, err) = run(&dir, "employee,usi\nE1,FAB0001\n", &extract(), &[]);
    assert_eq!((code, err.as_str()), (0, ""));
    assert_eq!(
        report,
        "row,usi,status,reason,extract_run_on\n2,FAB0001,LISTED,,2026-11-05\n"
    );
    assert!(out.contains("1 rows: 1 LISTED, 0 UNVERIFIED."), "{out}");
    assert!(out.contains("does not establish"), "{out}");
}

#[test]
fn a_row_that_is_not_listed_exits_2_with_its_reason() {
    let dir = folder("mixed");
    let input = "usi\nFAB0001\nFAB0002\nFAB0003\nFAB9999\n\n";
    let (code, report, ..) = run(&dir, input, &extract(), &[]);
    assert_eq!(code, 2);
    assert_eq!(
        report,
        "row,usi,status,reason,extract_run_on\n\
         2,FAB0001,LISTED,,2026-11-05\n\
         3,FAB0002,UNVERIFIED,restricted,2026-11-05\n\
         4,FAB0003,UNVERIFIED,end_dated,2026-11-05\n\
         5,FAB9999,UNVERIFIED,not_listed,2026-11-05\n"
    );
}

#[test]
fn a_stale_extract_is_a_finding_and_the_limit_is_configurable() {
    let dir = folder("stale");
    let later = ["--as-at", "2026-12-05"];
    let (code, report, ..) = run(&dir, "usi\nFAB0001\n", &extract(), &later);
    assert_eq!(code, 2);
    assert!(report.contains("UNVERIFIED,extract_stale"), "{report}");
    let longer = ["--as-at", "2026-12-05", "--max-extract-age-days", "30"];
    let (code, report, ..) = run(&dir, "usi\nFAB0001\n", &extract(), &longer);
    assert_eq!(code, 0, "{report}");
}

#[test]
fn a_missing_or_undated_extract_exits_1_and_passes_no_row() {
    let dir = folder("unusable");
    let (code, report, _, err) = run(&dir, "usi\nFAB0001\n", &dir.join("absent.txt"), &[]);
    assert_eq!(code, 1);
    assert!(
        report.contains("2,FAB0001,UNVERIFIED,extract_unreadable,\n"),
        "{report}"
    );
    assert!(err.contains("extract_unreadable"), "{err}");

    let undated = dir.join("undated.txt");
    let data = std::fs::read(extract()).unwrap();
    let cut = data.len() - data.rsplit(|b| *b == b'(').next().unwrap().len();
    let without_footer = [&data[..cut], b"1 rows affected)\r\n"].concat();
    let text = String::from_utf8(without_footer).unwrap();
    let rows = text.matches("FAB000").count();
    std::fs::write(&undated, text.replace("(1 rows", &format!("({rows} rows"))).unwrap();
    let (code, report, _, err) = run(&dir, "usi\nFAB0001\n", &undated, &[]);
    assert_eq!(code, 1, "{err}");
    assert!(report.contains("UNVERIFIED,extract_undated"), "{report}");
}

#[test]
fn an_input_without_a_usi_column_writes_no_report() {
    let dir = folder("no-column");
    let (code, report, _, err) = run(&dir, "employee\nE1\n", &extract(), &[]);
    assert_eq!((code, report.as_str()), (1, ""));
    assert!(err.contains("exactly one usi column"), "{err}");
}

#[test]
fn a_bad_age_limit_is_refused() {
    let dir = folder("bad-age");
    let (code, _, _, err) = run(
        &dir,
        "usi\nFAB0001\n",
        &extract(),
        &["--max-extract-age-days", "soon"],
    );
    assert_eq!(code, 1);
    assert!(err.contains("--max-extract-age-days expects"), "{err}");
}

#[test]
fn the_report_cannot_replace_the_extract() {
    let dir = folder("overwrite");
    let copy = dir.join("usi-check.csv");
    std::fs::copy(extract(), &copy).unwrap();
    let (code, _, _, err) = run(&dir, "usi\nFAB0001\n", &copy, &[]);
    assert_eq!(code, 1);
    assert!(err.contains("would overwrite"), "{err}");
    assert_eq!(
        std::fs::read(&copy).unwrap(),
        std::fs::read(extract()).unwrap()
    );
}
