// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The `rollforward` stdin command, the 8 MiB input bound and the encoding of
//! an undecodable command-line byte, run through the built binary.

use div7a_loan_review::rollforward::rollforward;
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], stdin: Vec<u8>) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_div7a-loan-review"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary starts");
    let mut pipe = child.stdin.take().expect("stdin");
    // The child may stop reading at its bound; a write it refuses is not a failure here.
    let writer = std::thread::spawn(move || {
        let _ = pipe.write_all(&stdin);
    });
    let out = child.wait_with_output().expect("binary runs");
    writer.join().expect("writer joins");
    (
        out.status.code().expect("exit code"),
        String::from_utf8(out.stdout).expect("UTF-8 stdout"),
        String::from_utf8(out.stderr).expect("UTF-8 stderr"),
    )
}

#[test]
fn rollforward_prints_the_library_result_as_pretty_json() {
    let input = json!({"year_of_income": "2026-27", "year_loan_made": "2025-26",
        "balance_at_start_of_year": "1000.00", "repayments": []});
    let want = rollforward(&input).expect("a valid rollforward");
    let (code, out, err) = run(&["rollforward"], input.to_string().into_bytes());
    assert_eq!((code, err.as_str()), (0, ""));
    let out = out.replace("\r\n", "\n");
    assert_eq!(
        out,
        format!("{}\n", serde_json::to_string_pretty(&want).unwrap())
    );
}

#[test]
fn rollforward_refuses_input_that_is_not_an_object() {
    let (code, out, err) = run(&["rollforward"], b"[]".to_vec());
    assert_eq!(code, 2);
    assert_eq!(out, "");
    assert_eq!(
        err.replace("\r\n", "\n"),
        "error: input must be a JSON object\n"
    );
}

#[test]
fn stdin_over_8_mib_is_refused_before_it_is_parsed() {
    let (code, out, err) = run(&["rollforward"], vec![b' '; (8 << 20) + 1]);
    assert_eq!(code, 2);
    assert_eq!(out, "");
    assert_eq!(
        err.replace("\r\n", "\n"),
        "error: JSON input exceeds 8 MiB\n"
    );
}

#[cfg(unix)]
#[test]
fn an_undecodable_argument_byte_is_written_as_its_escape() {
    use std::os::unix::ffi::OsStrExt;
    let text = div7a_loan_review::cli::decode_os(std::ffi::OsStr::from_bytes(b"a\xff"));
    // surrogateescape gives U+DCFF; backslashreplace writes it out.
    assert_eq!(div7a_loan_review::cli::output_bytes(&text), b"a\\udcff");
}
