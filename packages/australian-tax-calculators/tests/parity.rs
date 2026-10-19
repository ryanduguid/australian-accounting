use australian_tax_calculators::api;
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

/// The JSON boundary's answer in the corpus encoding: `{"ok": value}` or `{"error": message}`.
fn outcome(command: &str, input: &Value) -> Value {
    match api::evaluate(command, input) {
        Ok(v) => json!({ "ok": v }),
        Err(m) => json!({ "error": m }),
    }
}

/// Replays every call the Python 0.4.x tests and the differential generator
/// made, comparing serialised JSON so key order must match too.
#[test]
fn python_api_corpus_matches() {
    let corpus: Vec<Value> =
        serde_json::from_str(include_str!("data/python-api.json")).expect("corpus parses");
    assert!(corpus.len() > 3000, "corpus too small: {}", corpus.len());
    for command in api::commands() {
        assert!(
            corpus.iter().any(|row| row["command"] == command),
            "{command} has no case"
        );
    }
    let mut failures = Vec::new();
    for row in &corpus {
        let command = row["command"].as_str().unwrap();
        let actual = outcome(command, &row["input"]).to_string();
        let expected = row["expected"].to_string();
        if actual != expected {
            failures.push(format!(
                "{} {command} {}\n  expected {expected}\n  actual   {actual}",
                row["test"], row["input"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} differ:\n{}",
        failures.len(),
        corpus.len(),
        failures[..failures.len().min(5)].join("\n")
    );
}

#[test]
fn unparseable_decimal_strings_raise_invalid_operation() {
    let input = json!({"amount": "12,50", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"});
    assert_eq!(
        outcome("gst", &input),
        json!({"error": "decimal.InvalidOperation"})
    );
    let long = json!({"amount": "1".repeat(8193), "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"});
    assert_eq!(
        outcome("gst", &long),
        json!({"error": "amount: decimal input exceeds 8192 bytes"})
    );
}

#[test]
fn missing_arguments_and_non_objects_are_refused() {
    assert_eq!(
        outcome("resident-tax", &json!({"taxable_income": "1"})),
        json!({"error": "year is required"})
    );
    assert_eq!(
        outcome("gst", &json!([])),
        json!({"error": "input must be a JSON object"})
    );
    assert_eq!(
        outcome("periods", &json!({"kind": "nope"})),
        json!({"error": "unknown worksheet 'nope'"})
    );
}

fn run(args: &[&str], stdin: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_australian-tax-calculators"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap().replace("\r\n", "\n");
    (
        out.status.code().unwrap(),
        text(out.stdout),
        text(out.stderr),
    )
}

#[test]
fn binary_runs_commands_and_refuses_bad_input() {
    let (code, out, err) = run(
        &["gst"],
        r#"{"amount": "1100", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"}"#,
    );
    assert_eq!((code, err.as_str()), (0, ""));
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["amounts"]["gst"], "100.00");

    let (code, out, err) = run(&["gst"], "{");
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(err.starts_with("error: invalid JSON:"), "{err}");

    let (code, out, err) = run(&[], "");
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(
        err.starts_with("usage: australian-tax-calculators COMMAND"),
        "{err}"
    );

    let (code, out, _) = run(&["--api-help"], "");
    assert_eq!(code, 0);
    assert!(out.contains("worksheet-catalogue, periods"), "{out}");
}
