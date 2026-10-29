//! The Exchequer Tally's JSON results are plain UTF-8 here too: its CLI encoder
//! reads U+10F7FF pairs as OS-string escapes, which JSON text never carries.
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

fn exchequer(command: &str, input: &Value) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_au-accounting"))
        .args(["exchequer", command])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let bytes = serde_json::to_vec(input).unwrap();
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn exchequer_json_output_preserves_plane_16_private_use_text() {
    let statement = json!({"entity_name":"\u{10F7FF}\u{10F7FF}","abn_or_acn":"1","recipient_name":"\u{10F7FF}\u{10F800}","payment_date":"2026-10-18","total_distribution":"100.00","franking_percentage":"100.00","corporate_tax_rate":"0.25"});
    let out = exchequer("statement", &statement);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    let body = text.strip_suffix(if cfg!(windows) { "\r\n" } else { "\n" });
    let result: Value = serde_json::from_str(body.unwrap()).unwrap();
    assert_eq!(result["entity_name"], "\u{10F7FF}\u{10F7FF}");
    assert_eq!(result["recipient_name"], "\u{10F7FF}\u{10F800}");

    let mut refused = statement;
    refused["\u{10F7FF}\u{10F7FF}"] = "1".into();
    let out = exchequer("statement", &refused);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let newline = if cfg!(windows) { "\r\n" } else { "\n" };
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        format!(
            "error: generate_distribution_statement() got an unexpected keyword argument '\u{10F7FF}\u{10F7FF}'{newline}"
        )
    );
}
