// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The binary's JSON stdin boundary: the command list, the 8 MiB bound and a
//! stdin that cannot be read.
use std::io::Write;
use std::process::{Command, Stdio};

const BINARY: &str = env!("CARGO_BIN_EXE_ato-benchmark-compare");

fn lf(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .unwrap()
        .replace("\r\n", "\n")
}

#[test]
fn api_help_lists_every_json_command() {
    let out = Command::new(BINARY).arg("--api-help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = lf(&out.stdout);
    let commands = ato_benchmark_compare::api::COMMANDS.join(", ");
    assert_eq!(
        text,
        format!("JSON stdin commands: {commands}\nSee README.md for their schemas.\n")
    );
}

#[test]
fn json_input_over_8_mib_is_refused_before_parsing() {
    let command = ato_benchmark_compare::api::COMMANDS[0];
    let mut child = Command::new(BINARY)
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    // Write from a thread: the binary stops reading at 8 MiB + 1 byte.
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&vec![b' '; (8 << 20) + 16]);
    });
    let out = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(lf(&out.stderr), "error: JSON input exceeds 8 MiB\n");
}

#[cfg(target_os = "linux")]
#[test]
fn a_stdin_that_cannot_be_read_is_reported() {
    let command = ato_benchmark_compare::api::COMMANDS[0];
    let directory = std::fs::File::open(env!("CARGO_MANIFEST_DIR")).unwrap();
    let out = Command::new(BINARY)
        .arg(command)
        .stdin(directory)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(lf(&out.stderr), "error: Is a directory (os error 21)\n");
}
