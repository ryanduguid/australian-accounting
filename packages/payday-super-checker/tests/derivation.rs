//! `derive` and `verify` on the command line: a derivation verifies, an edited
//! one exits 1 naming its step, and an unreadable file exits 2.

use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};

const INPUT: &str = r#"{"qe_day": "2026-07-09", "first_to_fund": true}"#;

fn run(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
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
    let dir = std::env::temp_dir().join(format!("payday-derivation-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("derivation.json");
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    let ok = run(&["verify", path.to_str().unwrap()], "");
    assert_eq!(ok.status.code(), Some(0));
    let ok: Value = serde_json::from_slice(&ok.stdout).unwrap();
    assert_eq!(ok["verified"], true);

    saved["steps"][2]["value"] = Value::from("2026-07-21");
    let edited = run(&["verify"], &saved.to_string());
    assert_eq!(edited.status.code(), Some(1));
    let edited: Value = serde_json::from_slice(&edited.stdout).unwrap();
    assert_eq!(edited["differs_at"]["id"], "usual-period");

    let missing = run(&["verify", dir.join("absent.json").to_str().unwrap()], "");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stderr).starts_with("error: "));
}
