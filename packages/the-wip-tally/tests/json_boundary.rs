// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The binary's JSON stdin boundary: the command list, the 8 MiB bound and an
//! unreadable stdin.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const BINARY: &str = env!("CARGO_BIN_EXE_wip-tally");

fn run_with(args: &[&str], stdin: Stdio, input: Option<&[u8]>) -> Output {
    let mut child = Command::new(BINARY)
        .args(args)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(bytes) = input {
        // The binary may stop reading at the bound; a broken pipe is expected then.
        let _ = child.stdin.take().unwrap().write_all(bytes);
    }
    child.wait_with_output().unwrap()
}

#[test]
fn api_help_lists_the_json_commands() {
    let out = run_with(&["--api-help"], Stdio::null(), None);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(out.stdout).unwrap().replace("\r\n", "\n"),
        "JSON stdin commands: measure, parse-money\nAmounts are decimal strings. See README.md for schemas.\n"
    );
    assert!(out.stderr.is_empty());
}

#[test]
fn json_input_over_8_mib_is_refused_before_parsing() {
    let mut big = vec![b' '; (8 << 20) + 1];
    big[0] = b'{';
    let out = run_with(&["measure"], Stdio::piped(), Some(&big));
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert_eq!(
        String::from_utf8(out.stderr).unwrap().replace("\r\n", "\n"),
        "error: JSON input exceeds 8 MiB\n"
    );
}

#[test]
fn a_json_value_that_is_not_an_object_is_refused() {
    let out = run_with(&["measure"], Stdio::piped(), Some(b"[1]"));
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(out.stderr).unwrap().replace("\r\n", "\n"),
        "error: input must be a JSON object\n"
    );
}

#[cfg(unix)]
#[test]
fn an_unreadable_stdin_exits_2_with_the_os_error() {
    // Reading a directory fails with EISDIR.
    let dir = std::fs::File::open(env!("CARGO_MANIFEST_DIR")).unwrap();
    let out = run_with(&["measure"], Stdio::from(dir), None);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "error: Is a directory (os error 21)\n"
    );
}
