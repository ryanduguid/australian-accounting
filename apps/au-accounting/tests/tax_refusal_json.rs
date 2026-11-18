// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `au-accounting tax COMMAND --refusal-json` prints the engine's refusal code.
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

fn tax_gst(stdin: &[u8]) -> (Option<i32>, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_au-accounting"))
        .args(["tax", "gst", "--refusal-json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    let out = child.wait_with_output().unwrap();
    let text = |bytes| String::from_utf8(bytes).unwrap().replace("\r\n", "\n");
    (out.status.code(), text(out.stdout), text(out.stderr))
}

#[test]
fn a_refusal_prints_its_code_beside_the_error_line() {
    let scope = "Establish all stated scope conditions before calculating.";
    let refused =
        br#"{"amount": "1100", "gst_inclusive": true, "scope_confirmed": false, "year": "2025-26"}"#;
    let (code, out, err) = tax_gst(refused);
    assert_eq!((code, err), (Some(2), format!("error: {scope}\n")));
    let refusal: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        refusal,
        json!({"ok": false, "code": "scope.not_confirmed", "message": scope})
    );
    for (stdin, want) in [
        (b"{".as_slice(), "input.invalid_json"),
        (br#"{"a":1,"a":2}"#, "input.invalid_json"),
        (b"[]", "input.not_object"),
    ] {
        let (code, out, _) = tax_gst(stdin);
        assert_eq!(code, Some(2));
        let refusal: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(refusal["code"], want);
    }
}
