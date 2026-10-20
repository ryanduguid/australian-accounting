//! The fabricated evaluation packs: every scenario's exit code, due date,
//! verdict and final shortfall, and the refusal the receipt-amount pack names.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn contract(pack: &str) -> Value {
    let path = Path::new(ROOT)
        .join("evaluation")
        .join(pack)
        .join("expected_results.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("payday-evaluation-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run the checker on one fixture: (exit code, report rows by header, stderr).
fn run(
    pack: &str,
    fixture: &str,
    as_at: &str,
    extra: &[&str],
) -> (i32, Vec<Vec<(String, String)>>, String) {
    let dir = scratch(&format!("{pack}-{fixture}-{}", extra.len()));
    let input = Path::new(ROOT)
        .join("evaluation")
        .join(pack)
        .join("fixtures")
        .join(fixture);
    let report = dir.join("report.csv");
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .arg(&input)
        .args(["--as-at", as_at])
        .args(extra)
        .arg("-o")
        .arg(&report)
        .output()
        .unwrap();
    let mut rows = Vec::new();
    if let Ok(text) = std::fs::read_to_string(&report) {
        let text = text.trim_start_matches('\u{feff}');
        let mut lines = text.split("\r\n").filter(|l| !l.is_empty());
        let header: Vec<String> = lines
            .next()
            .unwrap()
            .split(',')
            .map(str::to_string)
            .collect();
        for line in lines {
            // The fixtures' rows hold no quoted commas before the verdict columns.
            let cells: Vec<String> = line.split(',').map(str::to_string).collect();
            rows.push(header.iter().cloned().zip(cells).collect());
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    (
        out.status.code().unwrap(),
        rows,
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn field<'a>(row: &'a [(String, String)], name: &str) -> &'a str {
    row.iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
        .unwrap()
}

#[test]
fn the_payday_super_evidence_scenarios_hold() {
    let contract = contract("payday_super_evidence");
    let as_at = contract["as_at"].as_str().unwrap();
    for scenario in contract["scenarios"].as_array().unwrap() {
        let fixture = scenario["fixture"].as_str().unwrap();
        let (code, rows, _) = run("payday_super_evidence", fixture, as_at, &[]);
        assert_eq!(
            i64::from(code),
            scenario["expected_exit"].as_i64().unwrap(),
            "{fixture}"
        );
        assert_eq!(rows.len(), 2, "{fixture}");
        assert_eq!(field(&rows[1], "employee_id"), "NOTE");
        assert_eq!(
            field(&rows[0], "due_date"),
            scenario["expected_due_date"].as_str().unwrap()
        );
        assert_eq!(
            field(&rows[0], "verdict"),
            scenario["expected_verdict"].as_str().unwrap()
        );
    }
}

#[test]
fn confirming_remittance_only_changes_the_exit_code_not_the_verdict() {
    let as_at = contract("payday_super_evidence")["as_at"]
        .as_str()
        .unwrap()
        .to_string();
    let fixture = "timely_remittance_no_receipt.csv";
    let (plain, rows, _) = run("payday_super_evidence", fixture, &as_at, &[]);
    let (confirmed, confirmed_rows, _) = run(
        "payday_super_evidence",
        fixture,
        &as_at,
        &["--confirm-remittance-only"],
    );
    assert_eq!((plain, confirmed), (2, 0));
    assert_eq!(field(&rows[0], "verdict"), "AT_RISK");
    assert_eq!(field(&confirmed_rows[0], "verdict"), "AT_RISK");
}

#[test]
fn the_receipt_amount_scenarios_and_refusal_hold() {
    let contract = contract("receipt_amount");
    let as_at = contract["as_at"].as_str().unwrap();
    for scenario in contract["scenarios"].as_array().unwrap() {
        let fixture = scenario["fixture"].as_str().unwrap();
        let (code, rows, _) = run("receipt_amount", fixture, as_at, &[]);
        assert_eq!(
            i64::from(code),
            scenario["expected_exit"].as_i64().unwrap(),
            "{fixture}"
        );
        assert_eq!(
            field(&rows[0], "due_date"),
            scenario["expected_due_date"].as_str().unwrap()
        );
        assert_eq!(
            field(&rows[0], "verdict"),
            scenario["expected_verdict"].as_str().unwrap()
        );
        assert_eq!(
            field(&rows[0], "final_shortfall"),
            scenario["expected_final_shortfall"].as_str().unwrap(),
            "{fixture}"
        );
    }
    for refusal in contract["refusals"].as_array().unwrap() {
        let fixture = refusal["fixture"].as_str().unwrap();
        let (code, rows, stderr) = run("receipt_amount", fixture, as_at, &[]);
        assert_eq!(i64::from(code), refusal["expected_exit"].as_i64().unwrap());
        assert!(rows.is_empty(), "{fixture} wrote a report");
        assert!(
            stderr.contains(refusal["expected_error"].as_str().unwrap()),
            "{stderr}"
        );
    }
}

#[test]
fn only_the_declared_evaluation_csvs_are_allowlisted() {
    let rules = std::fs::read_to_string(Path::new(ROOT).join(".gitignore")).unwrap();
    for pack in ["payday_super_evidence", "receipt_amount"] {
        let contract = contract(pack);
        let mut declared: Vec<String> = contract["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .chain(contract["refusals"].as_array().into_iter().flatten())
            .map(|s| {
                format!(
                    "evaluation/{pack}/fixtures/{}",
                    s["fixture"].as_str().unwrap()
                )
            })
            .collect();
        declared.sort();
        let prefix = format!("!evaluation/{pack}/fixtures/");
        let mut allowed: Vec<String> = rules
            .lines()
            .filter(|r| r.starts_with(&prefix))
            .map(|r| r.trim_start_matches('!').to_string())
            .collect();
        allowed.sort();
        assert_eq!(allowed, declared, "{pack}");
    }
}

#[test]
fn every_fixture_is_declared_once_and_fabricated() {
    for pack in ["payday_super_evidence", "receipt_amount"] {
        let contract = contract(pack);
        let mut declared: Vec<String> = contract["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .chain(contract["refusals"].as_array().into_iter().flatten())
            .map(|s| s["fixture"].as_str().unwrap().to_string())
            .collect();
        declared.sort();
        let folder = Path::new(ROOT)
            .join("evaluation")
            .join(pack)
            .join("fixtures");
        let mut present: Vec<String> = std::fs::read_dir(folder)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        present.sort();
        assert_eq!(present, declared, "{pack}");
    }
    let contract = contract("payday_super_evidence");
    let sources = contract["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2);
    assert!(
        sources
            .iter()
            .all(|s| s["url"].as_str().unwrap().starts_with("https://"))
    );
    let header = "employee_id,payment_date,sg_amount,remitted_date,fund_received_date,\
                  first_contribution_to_fund,out_of_cycle,next_standard_payday,defined_benefit,\
                  remitted_amount,matched_amount";
    for scenario in contract["scenarios"].as_array().unwrap() {
        let path = Path::new(ROOT)
            .join("evaluation/payday_super_evidence/fixtures")
            .join(scenario["fixture"].as_str().unwrap());
        let text = std::fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = text.trim_start_matches('\u{feff}').lines().collect();
        assert_eq!(lines[0], header);
        assert_eq!(lines.len(), 2);
        assert!(
            lines[1].starts_with("SYN001,2026-08-06,120.00,"),
            "{}",
            lines[1]
        );
    }
}
