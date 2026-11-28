//! Command-line edges. Every expected value is what CPython printed running the
//! original Python package (0.2.0) with the same arguments, NO_COLOR=1 and COLUMNS=80.

use std::path::PathBuf;
use std::process::{Command, Output};

const USAGE: &str = "usage: wip-tally [-h] [--version] {schedule,review-pack} ...\n";
const HEADER: &str = "contract_id,original_contract_sum,costs_incurred,estimated_cost_to_complete,\
                      certified_billings,outcome_reasonably_measurable\n";

fn folder(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-edges-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_in(dir: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wip-tally"))
        .args(args)
        .current_dir(dir)
        .env("COLUMNS", "80")
        .env("NO_COLOR", "1")
        .env_remove("PYTHON_COLORS")
        .env_remove("FORCE_COLOR")
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

#[test]
fn argument_errors_read_as_argparse_words_them() {
    let dir = folder("errors");
    std::fs::write(
        dir.join("c.csv"),
        format!("{HEADER}J1,1000,100,400,0,yes\n"),
    )
    .unwrap();
    for (args, message) in [
        (
            &["-"][..],
            "argument command: invalid choice: '-' (choose from 'schedule', 'review-pack')",
        ),
        (
            &["--version=1"],
            "argument --version: ignored explicit argument '1'",
        ),
        (
            &["--help=1"],
            "argument -h/--help: ignored explicit argument '1'",
        ),
        (
            &["--bogus", "schedule", "c.csv"],
            "unrecognized arguments: --bogus",
        ),
        (
            &["schedule", "c.csv", "b.csv", "-o", "x.csv"],
            "unrecognized arguments: b.csv",
        ),
    ] {
        let out = run_in(&dir, args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert_eq!(text(&out.stdout), "", "{args:?}");
        assert_eq!(
            text(&out.stderr),
            format!("{USAGE}wip-tally: error: {message}\n"),
            "{args:?}"
        );
    }
    assert!(!dir.join("x.csv").exists());
}

#[test]
fn letters_after_h_are_further_flags_and_help_runs_first() {
    let dir = folder("help");
    let joined = run_in(&dir, &["-hx"]);
    let plain = run_in(&dir, &["-h"]);
    assert_eq!(joined.status.code(), Some(0));
    assert!(text(&joined.stdout).starts_with(USAGE));
    assert_eq!(joined.stdout, plain.stdout);
    assert_eq!(text(&joined.stderr), "");
}

#[test]
fn a_contract_without_a_price_has_no_margin_in_either_report() {
    let dir = folder("no-price");
    std::fs::write(dir.join("c.csv"), format!("{HEADER}Z1,0,100,400,0,yes\n")).unwrap();
    let flags = "negative_margin_at_completion, onerous_contract_review_aasb_137, \
                 etc_has_no_commitments";
    let out = run_in(
        &dir,
        &["schedule", "c.csv", "--as-at", "2026-10-31", "-o", "s.csv"],
    );
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        text(&out.stdout),
        format!(
            "TheWIPTally: 1 contract(s), as at 2026-10-31\n\n  Contract assets:      0.00\n  \
             Contract liabilities: 0.00\n  Revenue to date:      0.00\n\n  Assets and \
             liabilities are per contract. They are not netted.\n\n  Review flags on 1 \
             contract(s):\n    Z1  margin n/a  fade n/a\n      {flags}\nWrote s.csv\n"
        )
    );
    let out = run_in(
        &dir,
        &["review-pack", "s.csv", "--source", "c.csv", "-o", "p.md"],
    );
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(text(&out.stdout), "Wrote p.md\n");
    let pack = text(&std::fs::read(dir.join("p.md")).unwrap());
    let queue = &pack[pack.find("## Review queue").unwrap()..];
    assert_eq!(
        queue,
        format!(
            "## Review queue\n\n| Contract | Margin | Fade | Flags |\n| --- | --- | --- | --- |\n\
             | Z1 |  |  | {flags} |\n\nKeep this pack beside the schedule CSV. The CSV is the \
             row-level evidence.\n"
        )
    );
}

#[test]
fn an_undated_schedule_is_checked_against_a_rebuild_as_at_today() {
    let dir = folder("undated");
    std::fs::write(
        dir.join("c.csv"),
        format!("{HEADER}J1,1000,100,400,0,yes\n"),
    )
    .unwrap();
    std::fs::write(dir.join("plain.csv"), "contract_id\nJ1\n").unwrap();
    let out = run_in(
        &dir,
        &[
            "review-pack",
            "plain.csv",
            "--source",
            "c.csv",
            "-o",
            "p.md",
        ],
    );
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(text(&out.stdout), "");
    assert_eq!(
        text(&out.stderr),
        "error: plain.csv does not match the rebuilt schedule; the source file, the mapping \
         and --as-at must be the ones that produced it\n"
    );
    assert!(!dir.join("p.md").exists());
}

/// CPython 3.14 on Linux, run from a working directory deleted before the call.
#[cfg(unix)]
#[test]
fn a_deleted_working_directory_refuses_before_reading_the_inputs() {
    let dir = folder("cwd-gone");
    let gone = dir.join("gone");
    let out = dir.join("o.csv");
    let run = |args: &[&str]| {
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(r#"cd "$1" && rmdir "$1" && shift && exec "$@""#)
            .arg("sh")
            .arg(&gone)
            .arg(env!("CARGO_BIN_EXE_wip-tally"))
            .args(args)
            .env("COLUMNS", "80")
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr),
        )
    };
    let out_text = out.to_str().unwrap();
    for (args, expected) in [
        (
            vec!["schedule", "c.csv", "--out", "o.csv"],
            "error: refusing to write o.csv: cannot resolve c.csv \
             ([Errno 2] No such file or directory)\n"
                .to_string(),
        ),
        (
            vec!["schedule", "c.csv", "--out", out_text],
            format!(
                "error: refusing to write {out_text}: cannot resolve c.csv \
                 ([Errno 2] No such file or directory)\n"
            ),
        ),
    ] {
        std::fs::create_dir_all(&gone).unwrap();
        assert_eq!(run(&args), (Some(1), String::new(), expected));
    }
    assert!(!out.exists());
}
