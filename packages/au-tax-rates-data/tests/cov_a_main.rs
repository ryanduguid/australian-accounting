// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The `rates` binary passes its arguments to the command and exits with its code.

use std::process::Command;

fn rates(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rates"))
        .args(args)
        .output()
        .expect("run rates")
}

#[test]
fn help_prints_usage_and_exits_0() {
    let out = rates(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("usage: "));
}

#[test]
fn an_unknown_command_is_a_usage_error_with_exit_2() {
    let out = rates(&["no-such-command"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("usage: "), "{stderr}");
    assert!(
        stderr.contains("invalid choice: 'no-such-command'"),
        "{stderr}"
    );
}
