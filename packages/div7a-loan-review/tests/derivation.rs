//! `derive` and `verify` on the command line: a derivation verifies, an edited
//! one exits 1 naming its step, and an unreadable file exits 2.

use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};

const INPUT: &str = r#"{"facts": {"year_of_income": "2026-27", "year_loan_made": "2023-24",
    "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00", "remaining_term_years": "5",
    "payments_applied_during_the_year": "25556.00", "gate_result": {"verdict": "COMPLYING",
    "benchmark_year_used": "2023-24", "reasons": [], "caveats": []}}}"#;

fn run(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_div7a-loan-review"))
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
    child.wait_with_output().unwrap()
}

#[test]
fn a_saved_derivation_verifies_and_an_edit_is_named() {
    let derived = run(&["derive"], INPUT);
    assert_eq!(derived.status.code(), Some(0));
    let mut saved: Value = serde_json::from_slice(&derived.stdout).unwrap();
    let dir = std::env::temp_dir().join(format!("div7a-derivation-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("derivation.json");
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    let ok = run(&["verify", path.to_str().unwrap()], "");
    assert_eq!(ok.status.code(), Some(0));
    let ok: Value = serde_json::from_slice(&ok.stdout).unwrap();
    assert_eq!(ok["verified"], true);

    saved["steps"][3]["value"] = Value::from("0.0837");
    let edited = run(&["verify"], &saved.to_string());
    assert_eq!(edited.status.code(), Some(1));
    let edited: Value = serde_json::from_slice(&edited.stdout).unwrap();
    assert_eq!(edited["differs_at"]["id"], "benchmark-rate");

    let missing = run(&["verify", dir.join("absent.json").to_str().unwrap()], "");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stderr).starts_with("error: "));
}

#[test]
fn verification_checks_identity_and_reports_comparison_progress() {
    let original: Value = serde_json::from_slice(&run(&["derive"], INPUT).stdout).unwrap();
    for key in ["engine", "calculation"] {
        for value in [Some(Value::from("wrong")), None] {
            let mut saved = original.clone();
            if let Some(value) = value {
                saved[key] = value;
            } else {
                saved.as_object_mut().unwrap().remove(key);
            }
            let result = run(&["verify", "-"], &saved.to_string());
            assert_eq!(result.status.code(), Some(1));
            let report: Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(report["differs_at"]["field"], key);
            assert_eq!(report["steps_checked"], 0);
        }
    }
    let n = original["steps"].as_array().unwrap().len();
    for i in [0, n / 2, n - 1] {
        let mut saved = original.clone();
        saved["steps"][i]["value"] = Value::from("edited");
        let report: Value =
            serde_json::from_slice(&run(&["verify"], &saved.to_string()).stdout).unwrap();
        assert_eq!(report["steps_checked"], i + 1);
    }
    for variant in [0, 1, 2] {
        let mut saved = original.clone();
        let steps = saved["steps"].as_array_mut().unwrap();
        match variant {
            0 => {
                steps.pop();
            }
            1 => steps.push(steps[0].clone()),
            _ => steps.push(Value::Null),
        }
        let report: Value =
            serde_json::from_slice(&run(&["verify"], &saved.to_string()).stdout).unwrap();
        assert_eq!(report["verified"], false);
        assert_eq!(
            report["steps_checked"],
            if variant == 0 { n } else { n + 1 }
        );
    }
    let mut saved = original.clone();
    saved["result"] = Value::Null;
    let report: Value =
        serde_json::from_slice(&run(&["verify"], &saved.to_string()).stdout).unwrap();
    assert_eq!(report["steps_checked"], n);
    saved = original;
    saved["engine_version"] = Value::from("historical-untrusted");
    let report: Value =
        serde_json::from_slice(&run(&["verify"], &saved.to_string()).stdout).unwrap();
    assert_eq!(report["verified"], true);
    assert_eq!(report["steps_checked"], n);
    assert_eq!(report["saved_engine_version"], "historical-untrusted");
}
