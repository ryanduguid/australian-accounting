// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `aus-accounting-mcp doctor` as the Python server's doctor.py reports it.
use serde_json::Value;
use std::process::{Command, Output, Stdio};

const ROOTS: [&str; 3] = [
    "AUS_ACCOUNTING_LIBRARY_ROOT",
    "AUS_ACCOUNTING_CORPUS_ROOT",
    "AUS_ACCOUNTING_RULINGS_ROOT",
];

fn doctor(roots: [Option<&std::ffi::OsStr>; 3]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aus-accounting-mcp"));
    command.arg("doctor").stdin(Stdio::null());
    for (variable, root) in ROOTS.iter().zip(roots) {
        match root {
            Some(root) => command.env(variable, root),
            None => command.env_remove(variable),
        };
    }
    command.output().unwrap()
}

/// Python's complete report with no folder set; only the version line is the native release's.
fn python_unconfigured() -> (String, i32, String) {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/native/retrieval-contracts.json")).unwrap();
    let expected = &fixture["doctor_unconfigured"];
    let stdout = expected["stdout"].as_str().unwrap().replacen(
        "aus-accounting-mcp 0.7.1\n",
        "aus-accounting-mcp 1.0.0\n",
        1,
    );
    (
        stdout,
        expected["exit"].as_i64().unwrap() as i32,
        expected["help"].as_str().unwrap().to_owned(),
    )
}

#[test]
fn doctor_reports_an_unconfigured_install_as_python_does() {
    let (stdout, code, _) = python_unconfigured();
    let output = doctor([None, None, None]);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), stdout);
    assert_eq!(output.status.code(), Some(code));
    // An empty variable counts as not set.
    let empty = Some(std::ffi::OsStr::new(""));
    let output = doctor([empty, empty, empty]);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), stdout);
    assert_eq!(output.status.code(), Some(code));
}

#[test]
fn doctor_exits_one_and_counts_each_broken_folder() {
    let missing = std::env::temp_dir().join(format!("native-doctor-gone-{}", std::process::id()));
    assert!(!missing.exists());
    let output = doctor([Some(missing.as_os_str()), Some(missing.as_os_str()), None]);
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1));
    let shown = missing.to_string_lossy();
    assert!(text.contains(&format!(
        "  AUS_ACCOUNTING_LIBRARY_ROOT: BROKEN: {shown}: AUS_ACCOUNTING_LIBRARY_ROOT must name an existing folder.\n"
    )));
    assert!(text.contains(&format!("  AUS_ACCOUNTING_CORPUS_ROOT: BROKEN: {shown}: ")));
    assert!(text.contains("  AUS_ACCOUNTING_RULINGS_ROOT: not set\n"));
    assert!(text.ends_with("\n2 configured folders are broken.\n"));
    // The MCP tools' words to a model read wrongly on a person's terminal.
    assert!(!text.contains("do not retry"));
}

#[test]
fn the_help_names_the_doctor_command() {
    let (_, _, help) = python_unconfigured();
    let output = Command::new(env!("CARGO_BIN_EXE_aus-accounting-mcp"))
        .arg("--help")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("  doctor  ") && text.contains(&help));
}
