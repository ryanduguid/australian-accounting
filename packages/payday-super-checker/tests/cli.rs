//! Command runs the frozen corpus cannot replay: those that read the machine's
//! clock, which differs on every replay, and those that name a directory or a
//! symlink, which a frozen file set cannot hold.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const HEADER: &str = "employee_id,payment_date,sg_amount,remitted_date,fund_received_date,\
                      first_contribution_to_fund,out_of_cycle,next_standard_payday,defined_benefit\n";

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("payday-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out: Output = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn write(dir: &Path, name: &str, rows: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, format!("{HEADER}{rows}")).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn an_input_that_cannot_be_read_is_named() {
    let dir = scratch("unreadable");
    let folder = dir.join("pay.csv");
    std::fs::create_dir(&folder).unwrap();
    let shown = folder.to_string_lossy().into_owned();
    let (code, _, err) = run(&dir, &[&shown, "-o", "report.csv", "--as-at", "2026-08-10"]);
    assert_eq!(code, 1);
    assert!(
        err.starts_with(&format!("error: cannot read {shown}: ")),
        "{err}"
    );

    std::fs::create_dir(dir.join("folder.csv")).unwrap();
    let (code, _, err) = run(&dir, &["review-pack", "folder.csv", "-o", "pack.md"]);
    assert_eq!(code, 1);
    assert!(err.starts_with("error: cannot read "), "{err}");

    let super_file = write(&dir, "super.csv", "");
    let (code, _, err) = run(
        &dir,
        &[
            "import",
            "--payroll",
            &shown,
            "--super",
            &super_file,
            "-o",
            "out.csv",
        ],
    );
    assert_eq!(code, 1);
    assert!(
        err.starts_with("error: cannot read ") && err.contains(&shown),
        "{err}"
    );
    assert!(!err.contains("Traceback"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn runs_without_an_as_at_date_keep_their_refusals() {
    let dir = scratch("clock");
    let target = write(&dir, "pay.csv", "E1,2026-07-09,600.00,,,no,no,,no\n");
    let (code, _, err) = run(&dir, &[&target, "-o", &target]);
    assert_eq!(code, 1);
    assert!(err.contains("overwrite the input"), "{err}");
    assert!(
        std::fs::read_to_string(&target)
            .unwrap()
            .starts_with("employee_id,payment_date")
    );

    let (code, _, err) = run(&dir, &["nope.csv"]);
    assert_eq!(code, 1);
    assert!(err.contains("error:"), "{err}");

    let old = write(&dir, "old.csv", "E1,2026-06-30,600.00,,,no,no,,no\n");
    let (code, _, err) = run(&dir, &[&old, "-o", "r.csv"]);
    assert_eq!(code, 1);
    assert!(err.contains("old quarterly SG law"), "{err}");

    let sentinel = write(
        &dir,
        "sentinel.csv",
        "E9,2026-07-09,100.00,,9999-12-31,no,no,,no\n",
    );
    let (code, _, err) = run(&dir, &[&sentinel, "-o", "r.csv"]);
    assert_eq!(code, 1);
    assert!(err.contains("not a real date"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_defaulted_as_at_is_named_in_a_notice() {
    let dir = scratch("as-at");
    let input = write(
        &dir,
        "contributions.csv",
        "E1,2026-08-06,600.00,2026-08-07,2026-08-10,no,no,,no\n",
    );
    let before = chrono::Local::now().date_naive().to_string();
    let (code, _, err) = run(&dir, &[&input, "-o", "report.csv"]);
    let after = chrono::Local::now().date_naive().to_string();
    assert!(code == 0 || code == 2, "{code}: {err}");
    assert!(
        err.contains("note: --as-at not supplied; assuming"),
        "{err}"
    );
    assert!(err.contains(&before) || err.contains(&after), "{err}");
    assert!(err.contains("machine's clock"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_existing_evidence_pack_directory_is_preserved() {
    let dir = scratch("pack");
    let output = dir.join("pack");
    std::fs::create_dir(&output).unwrap();
    let sentinel = output.join("decision-log.md");
    std::fs::write(&sentinel, "previous human decisions").unwrap();
    let fixture = Path::new(ROOT)
        .join("evaluation/payday_super_evidence/fixtures/timely_remittance_no_receipt.csv");
    let (code, _, err) = run(
        &dir,
        &[
            "evidence-pack",
            &fixture.to_string_lossy(),
            "--as-at",
            "2026-08-10",
            "-o",
            "pack",
        ],
    );
    assert_eq!(code, 1);
    assert!(err.contains("already exists"), "{err}");
    assert_eq!(
        std::fs::read_to_string(&sentinel).unwrap(),
        "previous human decisions"
    );
    assert_eq!(std::fs::read_dir(&output).unwrap().count(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_practitioner_pack_replaces_a_symlink_rather_than_its_target() {
    let dir = scratch("link");
    let report = run(
        &dir,
        &[
            &Path::new(ROOT)
                .join("examples/sample_payrun_no_transition.csv")
                .to_string_lossy(),
            "--as-at",
            "2026-08-10",
            "-o",
            "report.csv",
        ],
    );
    assert!(report.0 == 0 || report.0 == 2, "{report:?}");
    let target = dir.join("elsewhere.md");
    std::fs::write(&target, "leave me").unwrap();
    let output = dir.join("practitioner-review.md");
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(&target, &output);
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_file(&target, &output);
    if linked.is_err() {
        // Creating a symlink needs a privilege Windows does not always grant.
        let _ = std::fs::remove_dir_all(&dir);
        return;
    }
    let (code, _, err) = run(
        &dir,
        &["review-pack", "report.csv", "-o", "practitioner-review.md"],
    );
    assert!(code == 0 || code == 2, "{code}: {err}");
    let meta = std::fs::symlink_metadata(&output).unwrap();
    assert!(meta.is_file() && !meta.file_type().is_symlink());
    assert!(
        std::fs::read_to_string(&output)
            .unwrap()
            .contains("# Payday Super Practitioner Review Pack")
    );
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "leave me");
    let _ = std::fs::remove_dir_all(&dir);
}
