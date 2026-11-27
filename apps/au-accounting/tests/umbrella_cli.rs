// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The umbrella's own flags, the tax engine's flags and how each engine's
//! commands report a stream they cannot read or write.
use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_au-accounting"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let bytes = stdin.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = input.write_all(&bytes);
    });
    let out = child.wait_with_output().unwrap();
    writer.join().unwrap();
    out
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap().replace("\r\n", "\n")
}

#[test]
fn the_umbrella_names_its_version_and_refuses_an_unknown_engine() {
    let version = run(&["--version"], b"");
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        text(&version.stdout),
        format!("au-accounting {}\n", env!("CARGO_PKG_VERSION"))
    );
    let unknown = run(&["nope"], b"");
    assert_eq!(unknown.status.code(), Some(2));
    assert!(unknown.stdout.is_empty());
    assert!(text(&unknown.stderr).starts_with("au-accounting: unknown engine 'nope'\n\n"));
    let bare = run(&[], b"");
    assert_eq!(bare.status.code(), Some(2));
    assert!(bare.stdout.is_empty());
    assert!(text(&bare.stderr).contains("tax"));
}

#[test]
fn the_tax_engine_names_its_version_and_gives_usage_for_an_unknown_command() {
    let version = run(&["tax", "--version"], b"");
    assert_eq!(version.status.code(), Some(0));
    assert!(text(&version.stdout).starts_with("australian-tax-calculators "));
    let usage = run(&["tax", "no-such-command", "extra"], b"");
    assert_eq!(usage.status.code(), Some(2));
    assert!(usage.stdout.is_empty());
    assert!(
        text(&usage.stderr)
            .starts_with("usage: australian-tax-calculators COMMAND < input.json\n")
    );
}

#[test]
fn refusal_json_prints_a_result_and_codes_an_oversized_input() {
    let ok = run(
        &["tax", "gst", "--refusal-json"],
        br#"{"amount": "1100", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"}"#,
    );
    assert_eq!(ok.status.code(), Some(0));
    assert!(ok.stderr.is_empty());
    let result: Value = serde_json::from_slice(&ok.stdout).unwrap();
    assert!(result.is_object());
    assert_ne!(result.get("ok"), Some(&Value::Bool(false)));
    let large = run(&["tax", "gst", "--refusal-json"], &vec![b' '; (8 << 20) + 1]);
    assert_eq!(large.status.code(), Some(2));
    let refusal: Value = serde_json::from_slice(&large.stdout).unwrap();
    assert_eq!(refusal["code"], "input.too_large");
    assert_eq!(refusal["message"], "JSON input exceeds 8 MiB");
}

#[cfg(unix)]
#[test]
fn an_unreadable_input_stream_is_coded_input_unreadable() {
    let out = Command::new(env!("CARGO_BIN_EXE_au-accounting"))
        .args(["tax", "gst", "--refusal-json"])
        .stdin(std::fs::File::open("/").unwrap())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let refusal: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(refusal["code"], "input.unreadable");
}

#[cfg(target_os = "linux")]
fn to_full(args: &[&str], full_stdout: bool) -> Output {
    let full = || {
        std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap()
    };
    let mut command = Command::new(env!("CARGO_BIN_EXE_au-accounting"));
    command.args(args).stdin(Stdio::null());
    if full_stdout {
        command.stdout(full()).stderr(Stdio::piped());
    } else {
        command.stdout(Stdio::piped()).stderr(full());
    }
    command.output().unwrap()
}

#[cfg(target_os = "linux")]
#[test]
fn a_failed_write_exits_120_with_pythons_message_or_1_elsewhere() {
    let python = to_full(&["benchmarks", "--api-help"], true);
    assert_eq!(python.status.code(), Some(120));
    assert_eq!(
        text(&python.stderr),
        "Exception ignored while flushing sys.stdout:\nOSError: [Errno 28] No space left on device\n"
    );
    assert_eq!(to_full(&["tax", "--api-help"], true).status.code(), Some(1));
    assert_eq!(
        to_full(&["benchmarks", "no-such-command"], false).status.code(),
        Some(120)
    );
    assert_eq!(
        to_full(&["tax", "no-such-command"], false).status.code(),
        Some(1)
    );
}
