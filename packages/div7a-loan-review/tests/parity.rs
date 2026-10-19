//! Replays every case the differential harness froze: the calls the Python
//! 0.3.2 tests made, generated API calls and generated command runs, with the
//! files each one names. Results are compared as text, key order included.

use div7a_loan_review::api;
use serde_json::{Value, json};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn corpus() -> Vec<Value> {
    serde_json::from_str(include_str!("data/python-cases.json")).expect("corpus parses")
}

/// FNV-1a 64, as the corpus stores outputs over its size limits.
fn fnv(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h = (h ^ u64::from(*b)).wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

/// The corpus was captured on Windows, where opening a directory is
/// `[Errno 13] Permission denied`; Python on Unix reports `[Errno 21]`.
fn native(text: &str) -> String {
    if cfg!(windows) {
        text.to_string()
    } else {
        text.replace("[Errno 13] Permission denied", "[Errno 21] Is a directory")
    }
}

fn matches_text(actual: &str, expected: &Value) -> bool {
    match expected {
        Value::String(text) => actual == native(text),
        Value::Object(o) => {
            o.get("fnv").and_then(Value::as_str) == Some(fnv(actual.as_bytes()).as_str())
                && o.get("len").and_then(Value::as_u64) == Some(actual.len() as u64)
        }
        _ => false,
    }
}

fn decode_b64(text: &str) -> Vec<u8> {
    let value = |c: u8| -> u32 {
        match c {
            b'A'..=b'Z' => u32::from(c - b'A'),
            b'a'..=b'z' => u32::from(c - b'a') + 26,
            b'0'..=b'9' => u32::from(c - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("base64 digit {c}"),
        }
    };
    let digits: Vec<u8> = text.bytes().filter(|b| *b != b'=').collect();
    let mut out = Vec::with_capacity(digits.len() * 3 / 4);
    for chunk in digits.chunks(4) {
        let mut acc = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            acc |= value(*c) << (18 - 6 * i);
        }
        let bytes = acc.to_be_bytes();
        out.extend_from_slice(&bytes[1..chunk.len()]);
    }
    out
}

static SCRATCH: AtomicUsize = AtomicUsize::new(0);

/// A fresh folder holding the case's files.
fn scratch(case: &Value) -> PathBuf {
    let n = SCRATCH.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("div7a-parity-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (name, data) in case["files"].as_object().unwrap() {
        let path = dir.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, decode_b64(data.as_str().unwrap())).unwrap();
    }
    dir
}

fn outcome(result: Result<Value, String>) -> Value {
    match result {
        Ok(v) => json!({ "ok": v }),
        Err(m) => json!({ "error": m }),
    }
}

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_div7a-loan-review"))
}

fn lf(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .expect("UTF-8 output")
        .replace("\r\n", "\n")
}

/// One case's failure description, or None when it matches.
fn check(case: &Value) -> Option<String> {
    let expected = &case["expected"];
    if case["kind"] == "api" {
        let command = case["command"].as_str().unwrap();
        let actual = if case["files"].as_object().unwrap().is_empty() {
            outcome(api::evaluate(command, &case["input"]))
        } else {
            let dir = scratch(case);
            let mut child = binary()
                .arg(command)
                .current_dir(&dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(case["input"].to_string().as_bytes())
                .unwrap();
            let out = child.wait_with_output().unwrap();
            let _ = std::fs::remove_dir_all(&dir);
            if out.status.code() == Some(0) {
                json!({"ok": serde_json::from_slice::<Value>(&out.stdout).unwrap()})
            } else {
                let err = lf(&out.stderr);
                json!({"error": err.strip_prefix("error: ").unwrap().strip_suffix('\n').unwrap()})
            }
        };
        let text = actual.to_string();
        // Compared as text: Value equality ignores key order.
        let want = expected.to_string();
        let ok = match expected.get("fnv") {
            Some(_) => matches_text(&text, expected),
            None => text == want,
        };
        return (!ok).then(|| {
            format!(
                "{} {}\n  expected {expected}\n  actual   {text}",
                case["test"], case["input"]
            )
        });
    }
    let dir = scratch(case);
    let mut command = binary();
    command
        .args(
            case["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap()),
        )
        .current_dir(&dir)
        .env_remove("FORCE_COLOR")
        .env_remove("NO_COLOR")
        .env("PYTHON_COLORS", "0")
        .env("COLUMNS", "80");
    for (key, value) in case["env"].as_object().unwrap() {
        command.env(key, value.as_str().unwrap());
        if key == "FORCE_COLOR" {
            command.env_remove("PYTHON_COLORS");
        }
    }
    let out = command.output().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let (stdout, stderr) = (lf(&out.stdout), lf(&out.stderr));
    let ok = out.status.code() == expected["code"].as_i64().map(|c| c as i32)
        && matches_text(&stdout, &expected["stdout"])
        && matches_text(&stderr, &expected["stderr"]);
    (!ok).then(|| {
        format!(
            "{} {:?}\n  expected {expected}\n  actual   {:?} {stdout:?} {stderr:?}",
            case["test"],
            case["args"],
            out.status.code()
        )
    })
}

fn replay(filter: impl Fn(&Value) -> bool) -> usize {
    let cases: Vec<Value> = corpus().into_iter().filter(|c| filter(c)).collect();
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
    let failures: Vec<String> = std::thread::scope(|s| {
        let handles: Vec<_> = cases
            .chunks(cases.len().div_ceil(workers).max(1))
            .map(|chunk| s.spawn(move || chunk.iter().filter_map(check).collect::<Vec<_>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    assert!(
        failures.is_empty(),
        "{} of {} differ:\n{}",
        failures.len(),
        cases.len(),
        failures[..failures.len().min(5)].join("\n")
    );
    cases.len()
}

#[test]
fn python_api_calls_match() {
    let n = replay(|c| c["kind"] == "api");
    assert!(n > 2000, "{n}");
}

#[test]
fn python_command_runs_match() {
    let n = replay(|c| c["kind"] == "cli");
    assert!(n > 800, "{n}");
}

#[test]
fn every_command_has_cases() {
    let cases = corpus();
    for command in api::COMMANDS {
        assert!(
            cases.iter().any(|c| c["command"] == command),
            "{command} has no case"
        );
    }
    for name in ["rate", "gate", "myr", "review"] {
        assert!(
            cases
                .iter()
                .any(|c| c["kind"] == "cli" && c["args"][0] == name),
            "{name} has no run"
        );
    }
}

/// The published reason-code vocabulary.
const REASON_CODES: [&str; 23] = [
    "REFUSED_GATE_RESULT_MISSING",
    "REFUSED_GATE_NOT_COMPLYING",
    "REFUSED_GATE_UNKNOWN",
    "REFUSED_GATE_BENCHMARK_YEAR_MISMATCH",
    "REFUSED_YEAR_IS_YEAR_OF_LOAN",
    "REFUSED_YEAR_BEFORE_LOAN",
    "REFUSED_REMAINING_TERM_NOT_POSITIVE",
    "REFUSED_BENCHMARK_RATE_NOT_POSITIVE",
    "GATE_NO_BENCHMARK_YEAR",
    "GATE_WRITTEN_AGREEMENT_FAIL",
    "GATE_WRITTEN_AGREEMENT_UNKNOWN",
    "GATE_LODGMENT_DAY_FAIL",
    "GATE_LODGMENT_DAY_UNKNOWN",
    "GATE_INTEREST_FAIL",
    "GATE_INTEREST_UNKNOWN",
    "GATE_TERM_FAIL",
    "GATE_TERM_UNKNOWN",
    "YEAR_OF_INCOME_UNKNOWN",
    "YEAR_LOAN_MADE_UNKNOWN",
    "BENCHMARK_RATE_UNKNOWN",
    "UNPAID_BALANCE_UNKNOWN",
    "PAYMENTS_APPLIED_UNKNOWN",
    "REMAINING_TERM_UNKNOWN",
];

fn collect_codes(value: &Value, out: &mut std::collections::BTreeSet<String>) {
    match value {
        Value::Object(o) => {
            for (key, item) in o {
                if key == "reason_codes" {
                    out.extend(
                        item.as_array()
                            .unwrap()
                            .iter()
                            .map(|c| c.as_str().unwrap().to_string()),
                    );
                } else {
                    collect_codes(item, out);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|i| collect_codes(i, out)),
        _ => {}
    }
}

/// Every published code is emitted by some corpus case, and nothing else is.
#[test]
fn the_reason_code_vocabulary_is_exactly_what_the_engine_emits() {
    let mut emitted = std::collections::BTreeSet::new();
    for case in corpus() {
        if case["kind"] == "api"
            && case["files"].as_object().unwrap().is_empty()
            && let Ok(value) = api::evaluate(case["command"].as_str().unwrap(), &case["input"])
        {
            collect_codes(&value, &mut emitted);
        }
    }
    // A nil benchmark rate needs a reviewed override file saying so.
    let dir = std::env::temp_dir().join(format!("div7a-nil-rate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let override_path = dir.join("nil.json");
    std::fs::write(
        &override_path,
        r#"{"verified_until": "2027-28", "citation": "test", "rates": [{"year_of_income": "2027-28", "rate": "0"}]}"#,
    )
    .unwrap();
    let nil = api::evaluate(
        "minimum-yearly-repayment",
        &json!({"facts": {"year_of_income": "2027-28", "year_loan_made": "2023-24",
            "amalgamated_loan_unpaid_at_end_of_previous_year": "1000.00", "remaining_term_years": "3",
            "payments_applied_during_the_year": "0.00", "loan_id": "L",
            "gate_result": {"verdict": "COMPLYING", "benchmark_year_used": "2023-24", "reasons": [], "caveats": []}},
            "override": override_path.to_str().unwrap()}),
    )
    .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    collect_codes(&nil, &mut emitted);
    let published: std::collections::BTreeSet<String> =
        REASON_CODES.iter().map(|c| c.to_string()).collect();
    assert_eq!(emitted, published);
}

/// argparse prints the version through the help formatter, so a narrow
/// terminal wraps it (Python 3.15 at COLUMNS=14 and 20 printed these layouts).
#[test]
fn the_version_wraps_like_help_text() {
    for (columns, expected) in [
        ("14", "div7a-loan-\nreview 1.0.0\n"),
        ("20", "div7a-loan-review\n1.0.0\n"),
        ("80", "div7a-loan-review 1.0.0\n"),
    ] {
        let out = binary()
            .arg("--version")
            .env("COLUMNS", columns)
            .output()
            .unwrap();
        assert_eq!(lf(&out.stdout), expected, "COLUMNS={columns}");
    }
}
