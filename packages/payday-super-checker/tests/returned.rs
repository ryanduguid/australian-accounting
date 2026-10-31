//! Contributions a fund returned: SIS reg 7.07G(4) and LCR 2026/2 paragraphs
//! 24 and 25 say a returned amount is not a contribution, and the ATO says the
//! timeframe does not change. All fixtures are fabricated.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

const HEADER: &str = "employee_id,payment_date,sg_amount,remitted_date,fund_received_date,\
                      first_contribution_to_fund,out_of_cycle,next_standard_payday,defined_benefit,\
                      remitted_amount,matched_amount,join_caveats,returned_by_fund\n";

fn scratch() -> PathBuf {
    static RUNS: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "payday-returned-{}-{}",
        std::process::id(),
        RUNS.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

struct Run {
    code: i32,
    stderr: String,
    rows: Vec<Vec<(String, String)>>,
}

impl Run {
    fn field(&self, row: usize, name: &str) -> &str {
        self.rows[row]
            .iter()
            .find(|(k, _)| k == name)
            .map_or("", |(_, v)| v.as_str())
    }
}

fn split_csv(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => cells.push(std::mem::take(&mut cell)),
            other => cell.push(other),
        }
    }
    cells.push(cell);
    cells
}

/// Check `rows` as at `as_at`; the report's data rows, the note row dropped.
fn check_in(dir: &Path, rows: &str, as_at: &str, extra: &[&str]) -> Run {
    std::fs::write(dir.join("pay.csv"), format!("{HEADER}{rows}")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["pay.csv", "--as-at", as_at, "-o", "report.csv"])
        .args(["--confirm-transition-allocation"])
        .args(extra)
        .current_dir(dir)
        .output()
        .unwrap();
    let mut parsed = Vec::new();
    if let Ok(text) = std::fs::read_to_string(dir.join("report.csv")) {
        let text = text.trim_start_matches('\u{feff}');
        let mut lines = text.split("\r\n").filter(|l| !l.is_empty());
        let header = split_csv(lines.next().unwrap());
        for line in lines {
            let row: Vec<(String, String)> = header.iter().cloned().zip(split_csv(line)).collect();
            if row.iter().any(|(k, v)| k == "verdict" && !v.is_empty()) {
                parsed.push(row);
            }
        }
    }
    Run {
        code: out.status.code().unwrap(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        rows: parsed,
    }
}

fn check(rows: &str, as_at: &str, extra: &[&str]) -> Run {
    check_in(&scratch(), rows, as_at, extra)
}

/// A 6 August 2026 payday is due 17 August 2026.
const TIMELY_RECEIPT: &str = "E1,2026-08-06,120.00,2026-08-07,2026-08-10,no,no,,no,120.00,120.00,,";

fn flagged(row: &str) -> String {
    format!("{row}yes\n")
}

fn assert_unresolved(run: &Run, row: usize, between: &str) {
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert_eq!(run.field(row, "verdict"), "UNKNOWN");
    assert_eq!(run.field(row, "unassessable_between"), between);
    assert_eq!(run.field(row, "final_shortfall"), "");
    assert_eq!(run.field(row, "sgc_estimate_low"), "");
}

#[test]
fn a_timely_receipt_the_fund_returned_is_not_on_time() {
    let run = check(&flagged(TIMELY_RECEIPT), "2026-08-20", &[]);
    assert_unresolved(&run, 0, "UNPAID or ON_TIME");
    assert!(run.field(0, "caveats").contains("reg 7.07G(4)"));
    assert!(run.field(0, "caveats").contains("does not restart"));

    let unflagged = check(&format!("{TIMELY_RECEIPT}no\n"), "2026-08-20", &[]);
    assert_eq!(unflagged.field(0, "verdict"), "ON_TIME");
    assert_eq!(unflagged.code, 0);
}

#[test]
fn a_return_with_no_date_after_the_deadline_is_unresolved_rather_than_late() {
    let run = check(
        &flagged("E1,2026-08-06,120.00,2026-08-07,,no,no,,no,120.00,120.00,,"),
        "2026-09-20",
        &[],
    );
    assert_unresolved(&run, 0, "UNPAID or LATE");
}

#[test]
fn a_returned_earlier_row_does_not_confirm_an_item_4_extension() {
    // The first payday has 20 business days. Its receipt would extend the
    // second payday's deadline, but that receipt came back.
    let first = "E1,2026-08-06,120.00,2026-08-07,2026-08-10,yes,no,,no,120.00,120.00,,";
    let second = "E1,2026-08-13,120.00,2026-08-14,2026-08-28,no,no,,no,120.00,120.00,,no\n";
    let kept = check(&format!("{first}no\n{second}"), "2026-09-20", &[]);
    assert_eq!(kept.field(1, "verdict"), "ON_TIME");
    assert_eq!(kept.field(1, "pathway"), "ITEM4_ALIGNED");

    let returned = check(&format!("{}{second}", flagged(first)), "2026-09-20", &[]);
    assert_ne!(returned.field(1, "verdict"), "ON_TIME");
    assert_ne!(returned.field(1, "pathway"), "ITEM4_ALIGNED");
}

#[test]
fn a_returned_flag_on_a_row_with_nothing_to_test_is_refused() {
    let db = check(
        &flagged("E1,2026-08-06,120.00,,,no,no,,yes,,,,"),
        "2026-08-20",
        &[],
    );
    assert_eq!(db.code, 1);
    assert!(db.stderr.contains("returned_by_fund=yes"), "{}", db.stderr);
    let nil = check(
        &flagged("E1,2026-08-06,0.00,,,no,no,,no,,,,"),
        "2026-08-20",
        &[],
    );
    assert_eq!(nil.code, 1);

    let unreadable = check(
        "E1,2026-08-06,120.00,,,no,no,,no,,,,maybe\n",
        "2026-08-20",
        &[],
    );
    assert_eq!(unreadable.code, 1);
    assert!(
        unreadable.stderr.contains("returned_by_fund"),
        "{}",
        unreadable.stderr
    );
}

/// Beam's status says a fund refunded some of the batch, not which contributions.
#[test]
fn a_beam_refund_status_travels_to_an_unresolved_row() {
    let dir = scratch();
    std::fs::write(
        dir.join("payroll.csv"),
        "Employee,Date Paid,Pay Period Ending,Super Guarantee\n\
         Test Employee One,06/08/2026,06/08/2026,120.00\n\
         Test Employee Two,06/08/2026,06/08/2026,90.00\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("super.csv"),
        "Employee,Contribution Type,Period Start,Period End,Payment Date,Amount,Status\n\
         Test Employee One,Super Guarantee,01/08/2026,06/08/2026,07/08/2026,120.00,Sent to fund with refund\n\
         Test Employee Two,Super Guarantee,01/08/2026,06/08/2026,07/08/2026,90.00,Sent to fund\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["import", "--payroll", "payroll.csv", "--super", "super.csv"])
        .args(["--vendor", "employment-hero", "-o", "pay.csv"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("returned_by_fund=yes"), "{stdout}");
    assert!(stdout.contains("No supported payroll or clearing-house export"));
    let written = std::fs::read_to_string(dir.join("pay.csv")).unwrap();
    assert!(
        written
            .lines()
            .next()
            .unwrap()
            .ends_with(",returned_by_fund")
    );

    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["pay.csv", "--as-at", "2026-09-20", "-o", "report.csv"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let report = std::fs::read_to_string(dir.join("report.csv")).unwrap();
    let rows: Vec<&str> = report.lines().skip(1).collect();
    assert!(rows[0].contains("UNKNOWN") && rows[0].contains("returned_by_fund=yes"));
    assert!(rows[0].contains("UNPAID or LATE"));
    assert!(rows[1].contains("AT_RISK"), "{}", rows[1]);
}

#[test]
fn the_api_accepts_the_returned_flag() {
    let call = |line: Value| -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
            .arg("compute-due")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(serde_json::json!({ "line": line }).to_string().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        serde_json::from_slice(&out.stdout).unwrap()
    };
    let line = serde_json::json!({
        "employee_id": "E1", "qe_day": "2026-08-06", "sg_amount": "120.00",
        "first_to_fund": false, "out_of_cycle": false, "db_interest": false,
        "returned_by_fund": true,
    });
    assert_eq!(call(line)["due"], "2026-08-17");
}
