// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lodgment-calendar"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let bytes = input.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&bytes);
    });
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    output
}

#[test]
fn success_writes_one_json_result_and_errors_only_use_stderr() {
    let input=br#"{"obligation":"income_tax_return","entity_type":"partnership","financial_year":"2025-26","channel":"self"}"#;
    let output = run(&["due-dates"], input);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["engine_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["obligations"][0]["payment"], "none");
    for (args, input) in [
        (vec!["due-dates"], b"{".as_slice()),
        (vec!["due-dates"], b"[]".as_slice()),
        (vec!["unknown"], b"".as_slice()),
    ] {
        let output = run(&args, input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(output.stderr.starts_with(b"error: "));
    }
}

#[test]
fn bounded_input_and_command_help() {
    let output = run(&["due-dates"], &vec![b' '; (8 << 20) + 1]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: JSON input exceeds 8 MiB\n"
    );
    let help = run(&["--help"], b"");
    assert!(help.status.success());
    assert!(
        String::from_utf8_lossy(&help.stdout)
            .contains("AGPL-3.0-or-later with additional terms (see NOTICE).")
    );
    for flag in ["--version", "--api-help"] {
        let output = run(&[flag], b"");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn duplicate_fields_at_every_depth_are_refused() {
    for input in [
        br#"{"obligation":"bas","obligation":"ias"}"#.as_slice(),
        br#"{"facts":{"lodges_electronically":true,"lodges_electronically":false}}"#.as_slice(),
        br#"{"extra_holidays":{"2027-07-28":"One","2027-07-28":"Two"}}"#.as_slice(),
        br#"{"facts":[{"a":1,"a":2}]}"#.as_slice(),
    ] {
        let output = run(&["due-dates"], input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("duplicate JSON field")
        );
    }
}

#[test]
fn hostile_refusal_values_do_not_panic_or_emit_controls() {
    for input in [
        br#"{"obligation":1e99999}"#.as_slice(),
        br#"{"obligation":"a\u0000\u001b"}"#.as_slice(),
        br#"{"a\u0000\u001b":true}"#.as_slice(),
        br#"{"a\u0000\u001b":true,"a\u0000\u001b":false}"#.as_slice(),
    ] {
        let output = run(&["due-dates"], input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.contains(&0));
        assert!(!output.stderr.contains(&27));
    }
}

#[test]
fn duplicate_fields_are_found_past_negative_fractional_and_null_values() {
    let output = run(
        &["due-dates"],
        br#"{"a":-1,"b":1.5,"c":null,"d":[-2,0.25,null],"a":2}"#,
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: invalid JSON: duplicate JSON field: \"a\" at line 1 column 47\n"
    );
}

#[test]
fn a_negative_overflowing_number_is_named_as_negative_infinity() {
    let output = run(&["due-dates"], br#"{"obligation":-1e99999}"#);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("-inf"), "{err}");
}

#[cfg(unix)]
#[test]
fn an_unreadable_input_stream_is_reported_and_refused() {
    let output = Command::new(env!("CARGO_BIN_EXE_lodgment-calendar"))
        .arg("due-dates")
        .stdin(std::fs::File::open("/").unwrap())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(
        err.starts_with("error: ") && err.contains("directory"),
        "{err}"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_failed_output_write_exits_1() {
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_lodgment-calendar"))
        .arg("--help")
        .stdout(full)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}
