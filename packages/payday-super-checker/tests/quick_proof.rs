//! The README's fabricated proof: the committed transcript is the console
//! output of the run it names, and the README shows it before installation.

use std::process::Command;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const SAMPLE: &str = "examples/sample_payrun_no_transition.csv";

fn transcript(output: &str) -> String {
    format!(
        "# Fabricated Payday Super review\n\nCommand:\n\n```bash\npayday-super-check {SAMPLE} --as-at 2026-09-10\n```\n\n\
         Exit code: 2, review required.\n\n```text\n{}\n```\n\nThe input is fabricated. No employer or employee data are used.\n",
        output.trim_end()
    )
}

#[test]
fn the_committed_transcript_is_the_current_run() {
    let dir = std::env::temp_dir().join(format!("payday-proof-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let report = dir.join("report.csv");
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .current_dir(ROOT)
        .args([SAMPLE, "--as-at", "2026-09-10", "-o"])
        .arg(&report)
        .env("COLUMNS", "80")
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(out.status.code(), Some(2));
    let output = format!(
        "{}{}",
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap()
    )
    .replace(&report.to_string_lossy().into_owned(), "report.csv")
    .replace("\r\n", "\n");
    for required in [
        "ON_TIME: 3  AT_RISK: 1  LATE: 1  UNPAID: 1  UNKNOWN: 0  SKIPPED: 1",
        "shortfall $780.00",
        "experimental estimated SG charge $788.76 - $1262.02",
        "Educational tool, not advice",
    ] {
        assert!(output.contains(required), "{required}");
    }
    let committed = std::fs::read_to_string(format!("{ROOT}/assets/quick-proof.md")).unwrap();
    assert_eq!(committed.replace("\r\n", "\n"), transcript(&output));
}

#[test]
fn the_readme_places_the_proof_and_its_check_before_the_build() {
    let readme = std::fs::read_to_string(format!("{ROOT}/README.md")).unwrap();
    let proof =
        "[![Fabricated Payday Super review](assets/quick-proof.svg)](assets/quick-proof.md)";
    let check = "cargo test --locked --test quick_proof";
    assert!(readme.contains(proof) && readme.contains(check));
    assert!(readme.find(proof).unwrap() < readme.find("## Build and check").unwrap());
}
