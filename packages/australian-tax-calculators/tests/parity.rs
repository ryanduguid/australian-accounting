use australian_tax_calculators::api;
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

/// Worksheets added after Python 0.2.0, so absent from the frozen corpus;
/// `tests/fbt_car.rs` covers them.
const NEW_KINDS: [(&str, &str); 2] = [
    ("fbt-car-statutory", "fbt_car_statutory"),
    ("fbt-car-operating-cost", "fbt_car_operating_cost"),
];

/// The JSON boundary's answer in the corpus encoding: `{"ok": value}` or `{"error": message}`.
fn outcome(command: &str, input: &Value) -> Value {
    match api::evaluate(command, input) {
        Ok(v) => json!({ "ok": v }),
        Err(m) => json!({ "error": m }),
    }
}

/// The frozen catalogue predates the new worksheets: compare it without them.
fn without_new_kinds(command: &str, mut actual: Value) -> Value {
    if command == "worksheet-catalogue"
        && let Some(entries) = actual.get_mut("ok").and_then(Value::as_object_mut)
    {
        for (_, kind) in NEW_KINDS {
            assert!(entries.shift_remove(kind).is_some(), "{kind} missing");
        }
    }
    actual
}

/// Applies the source-record divergences CHANGELOG.md documents to a frozen
/// Python result; the frozen corpus itself is never edited.
fn adapt(command: &str, mut expected: Value) -> Value {
    // The ATO moved its lump sum in arrears page; the redirect was seen on 26 October 2026.
    let lump_sum_old = "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/lump-sum-payment-in-arrears-tax-offsets";
    let lump_sum_new = "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-you-must-declare/lump-sum-payment-in-arrears";
    let lump_sum = match command {
        "medicare-levy" => expected.pointer_mut("/ok/sources/3"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/medicare_levy/sources/3"),
        _ => None,
    };
    if let Some(source) = lump_sum {
        assert_eq!(*source, json!(lump_sum_old), "frozen lump sum source moved");
        *source = json!(lump_sum_new);
    }
    if let Some(Value::String(error)) = expected.get_mut("error") {
        *error = error.replace(lump_sum_old, lump_sum_new);
    }
    let study = match command {
        "study-loan-repayment" => expected.pointer_mut("/ok"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/study_loan_repayment"),
        _ => None,
    };
    let Some(study) = study else {
        return expected;
    };
    // The ATO moved and retitled its study loan page; read again on 20 October 2026.
    let url = "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates";
    study["sources"][0] = json!(url);
    study["source_checked"] = json!("2026-10-20");
    if command == "worksheet-catalogue" {
        study["source"] = json!(url);
        let passage = study["source_passage"].as_str().unwrap().replace(
            "repayment thresholds and rates",
            "minimum repayment income and rates",
        );
        study["source_passage"] = json!(passage);
        let evidence = &mut study["example_evidence"];
        evidence["document"] =
            json!("ATO study and training loan minimum repayment income and rates");
        evidence["reviewed"] = json!("2026-10-13");
        evidence["checked"] = json!("2026-10-20");
    }
    expected
}

/// Replays every call the Python 0.4.x tests and the differential generator
/// made, comparing serialised JSON so key order must match too.
#[test]
fn python_api_corpus_matches() {
    let corpus: Vec<Value> =
        serde_json::from_str(include_str!("data/python-api.json")).expect("corpus parses");
    assert!(corpus.len() > 3000, "corpus too small: {}", corpus.len());
    for command in api::commands() {
        if NEW_KINDS.iter().any(|(c, _)| *c == command) {
            continue;
        }
        assert!(
            corpus.iter().any(|row| row["command"] == command),
            "{command} has no case"
        );
    }
    let mut failures = Vec::new();
    for row in &corpus {
        let command = row["command"].as_str().unwrap();
        let actual = without_new_kinds(command, outcome(command, &row["input"])).to_string();
        let expected = adapt(command, row["expected"].clone()).to_string();
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
