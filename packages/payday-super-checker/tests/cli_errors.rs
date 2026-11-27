// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Failures the command line reports instead of a result: a CSV field over
//! Python's `csv` field limit (which Python left uncaught), JSON input over the
//! 8 MiB cap, unreadable inputs and unwritable reports, and argparse's
//! handling of tokens that look like a dash or a negative number.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const HEADER: &str = "employee_id,payment_date,sg_amount,remitted_date,fund_received_date,\
                      first_contribution_to_fund,out_of_cycle,next_standard_payday,defined_benefit\n";
const FIELD_LIMIT_ERROR: &str = "error: _csv.Error: field larger than field limit (131072)\n";

/// A folder for one test, under the build's own scratch directory.
fn folder(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-errors-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// (exit code, stdout, stderr) of the binary run in `dir`, with `stdin` piped in.
fn run(dir: &Path, args: &[&str], stdin: &[u8]) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // The binary may stop reading once it has seen enough; a closed pipe is fine.
    let _ = child.stdin.take().unwrap().write_all(stdin);
    let out = child.wait_with_output().unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap().replace("\r\n", "\n");
    (
        out.status.code().unwrap(),
        text(out.stdout),
        text(out.stderr),
    )
}

/// A CSV whose one data row holds a field one character over the limit.
fn oversized(dir: &Path, name: &str, header: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, format!("{header}{}\n", "x".repeat(131_073))).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn api_help_lists_the_json_commands() {
    let dir = folder("api-help");
    let (code, out, err) = run(&dir, &["--api-help"], b"");
    assert_eq!(code, 0);
    assert!(
        out.starts_with("JSON stdin commands: parse-date-text, "),
        "{out}"
    );
    assert!(
        out.ends_with("\nSee README.md for their schemas.\n"),
        "{out}"
    );
    assert_eq!(err, "");
}

#[test]
fn json_input_over_8_mib_is_refused() {
    let dir = folder("json-cap");
    let big = vec![b' '; (8 << 20) + 1];
    let (code, out, err) = run(&dir, &["parse-amount"], &big);
    assert_eq!((code, out.as_str()), (2, ""));
    assert_eq!(err, "error: JSON input exceeds 8 MiB\n");

    std::fs::write(dir.join("big.json"), &big).unwrap();
    let (code, _, err) = run(&dir, &["verify", "big.json"], b"");
    assert_eq!(code, 2);
    assert_eq!(err, "error: JSON input exceeds 8 MiB\n");

    // Exactly 8 MiB is read, and then judged as JSON.
    let (code, _, err) = run(&dir, &["parse-amount"], &big[1..]);
    assert_eq!(code, 2);
    assert!(err.starts_with("error: invalid JSON: "), "{err}");
}

#[test]
fn a_field_over_the_csv_limit_fails_check() {
    let dir = folder("check-field-limit");
    let csv = oversized(&dir, "pay.csv", HEADER);
    let (code, out, err) = run(
        &dir,
        &[&csv, "-o", "report.csv", "--as-at", "2026-08-10"],
        b"",
    );
    assert_eq!((code, out.as_str()), (1, ""));
    assert_eq!(err, FIELD_LIMIT_ERROR);
    assert!(!dir.join("report.csv").exists());
}

#[test]
fn a_field_over_the_csv_limit_fails_usi_check() {
    let dir = folder("usi-field-limit");
    let csv = oversized(&dir, "usis.csv", "usi\n");
    let extract = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/usi_extract");
    let extract = extract.to_string_lossy().into_owned();
    let args = [
        "usi-check",
        &csv,
        "--usi-extract",
        &extract,
        "--as-at",
        "2026-11-05",
    ];
    let (code, _, err) = run(&dir, &args, b"");
    assert_eq!(code, 1);
    assert_eq!(err, FIELD_LIMIT_ERROR);
}

#[test]
fn usi_check_names_an_input_it_cannot_read_and_a_report_it_cannot_write() {
    let dir = folder("usi-os");
    let extract = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/usi_extract");
    let extract = extract.to_string_lossy().into_owned();
    let base = [
        "usi-check",
        "--usi-extract",
        &extract,
        "--as-at",
        "2026-11-05",
    ];

    let (code, _, err) = run(&dir, &[&base[..], &["missing.csv"]].concat(), b"");
    assert_eq!(code, 1);
    assert!(err.starts_with("error: cannot read missing.csv: "), "{err}");

    std::fs::write(dir.join("usis.csv"), "usi\n12345678901\n").unwrap();
    let report = dir.join("no-such-folder").join("report.csv");
    let report = report.to_string_lossy().into_owned();
    let (code, _, err) = run(
        &dir,
        &[&base[..], &["usis.csv", "-o", &report]].concat(),
        b"",
    );
    assert_eq!(code, 1);
    assert!(err.starts_with("error: cannot read "), "{err}");
    assert!(err.contains("no-such-folder"), "{err}");
}

#[test]
fn usi_check_without_its_arguments_prints_usage() {
    let dir = folder("usi-usage");
    let (code, out, err) = run(&dir, &["usi-check"], b"");
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(err.starts_with("usage: "), "{err}");
    assert!(
        err.contains("the following arguments are required"),
        "{err}"
    );
}

#[test]
fn a_lone_dash_and_a_negative_decimal_are_positional_paths() {
    let dir = folder("dash-paths");
    for path in ["-", "-.5"] {
        let (code, _, err) = run(
            &dir,
            &[path, "-o", "report.csv", "--as-at", "2026-08-10"],
            b"",
        );
        assert_eq!(code, 1, "{path}");
        assert_eq!(err, format!("error: file not found: {path}\n"));
    }
}

#[test]
fn a_field_over_the_csv_limit_fails_import() {
    let dir = folder("import-field-limit");
    let payroll = oversized(&dir, "payroll.csv", "Employee ID,Pay Date,Super\n");
    std::fs::write(dir.join("super.csv"), "Employee ID,Date Paid,Amount\n").unwrap();
    let args = [
        "import",
        "--payroll",
        &payroll,
        "--super",
        "super.csv",
        "-o",
        "out.csv",
    ];
    let (code, _, err) = run(&dir, &args, b"");
    assert_eq!(code, 1);
    assert_eq!(err, FIELD_LIMIT_ERROR);
}

/// Employment Hero exports with one matched SG payday, for the import command.
fn import_inputs(dir: &Path) {
    std::fs::write(
        dir.join("payroll.csv"),
        "Employee,Date Paid,Pay Period Ending,Super Guarantee\nAnn,2026-07-10,2026-07-07,100.00\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("super.csv"),
        "Employee,Super Contribution Type,Period Start,Period End,Payment Date,Amount,Status\n\
         Ann,SG,2026-07-01,2026-07-07,2026-07-12,100.00,Sent to Fund\n",
    )
    .unwrap();
}

// Expected texts are the pre-port Python CLI's (paydaysuper at 5874033^).

#[test]
fn import_names_an_export_it_cannot_read() {
    let dir = folder("import-unreadable");
    import_inputs(&dir);
    std::fs::create_dir(dir.join("pdir")).unwrap();
    let args = ["import", "--payroll", "pdir", "--super", "super.csv"];
    let (code, out, err) = run(&dir, &[&args[..], &["-o", "out.csv"]].concat(), b"");
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(err.starts_with("error: cannot read pdir: "), "{err}");
    #[cfg(unix)]
    assert_eq!(err, "error: cannot read pdir: Is a directory\n");
    // Python checked the output name inside its OSError handler, so a bad
    // name there escaped as an uncaught ValueError.
    let (code, _, err) = run(&dir, &[&args[..], &["-o", "out.txt"]].concat(), b"");
    assert_eq!(code, 1);
    assert_eq!(
        err,
        "error: ValueError: generated output must use a .csv filename: out.txt\n"
    );
}

#[test]
fn import_names_an_output_it_cannot_write() {
    let dir = folder("import-unwritable");
    import_inputs(&dir);
    std::fs::create_dir(dir.join("outdir.csv")).unwrap();
    let args = [
        "import",
        "--payroll",
        "payroll.csv",
        "--super",
        "super.csv",
        "--vendor",
        "employment-hero",
        "-o",
        "outdir.csv",
    ];
    let (code, out, err) = run(&dir, &args, b"");
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(err.starts_with("error: cannot write outdir.csv: "), "{err}");
    #[cfg(unix)]
    assert_eq!(err, "error: cannot write outdir.csv: Is a directory\n");
}

/// A folder holding the sample pay run's checker report as `report.csv`.
fn report_folder(name: &str) -> PathBuf {
    let dir = folder(name);
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample_payrun.csv");
    std::fs::copy(sample, dir.join("in.csv")).unwrap();
    let args = [
        "in.csv",
        "--as-at",
        "2026-11-27",
        "--confirm-transition-allocation",
        "-o",
        "report.csv",
    ];
    let (code, _, err) = run(&dir, &args, b"");
    assert_eq!(code, 2, "{err}");
    dir
}

#[test]
fn review_pack_names_a_pack_it_cannot_write() {
    let dir = report_folder("pack-unwritable");
    std::fs::create_dir(dir.join("pack.md")).unwrap();
    let (code, out, err) = run(&dir, &["review-pack", "report.csv", "-o", "pack.md"], b"");
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(err.starts_with("error: cannot write pack.md: "), "{err}");
    #[cfg(unix)]
    assert_eq!(err, "error: cannot write pack.md: Is a directory\n");
}

#[test]
fn a_double_dash_after_the_positional_leaves_the_rest_unrecognised() {
    let dir = report_folder("pack-marker");
    let (code, out, err) = run(
        &dir,
        &["review-pack", "report.csv", "--", "-o", "x.md"],
        b"",
    );
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(
        err.ends_with("review-pack: error: unrecognized arguments: -o x.md\n"),
        "{err}"
    );
    assert!(!dir.join("x.md").exists());
}

/// `os.getcwd()` fails in a folder removed whilst the command runs in it; its
/// FileNotFoundError names no file, which Python printed as None.
#[cfg(unix)]
#[test]
fn review_pack_in_a_removed_folder_names_no_file() {
    let dir = report_folder("pack-cwd-gone");
    std::fs::create_dir(dir.join("gone")).unwrap();
    let out = Command::new("sh")
        .args([
            "-c",
            "cd gone && rmdir ../gone && exec \"$0\" review-pack report.csv",
            env!("CARGO_BIN_EXE_payday-super-check"),
        ])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "error: file not found: None\n"
    );
}
