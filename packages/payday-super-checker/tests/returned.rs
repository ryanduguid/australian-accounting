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

// --- dated, amount-specific events -------------------------------------------

const EVENTS_HEADER: &str =
    "employee_id,payment_date,event,event_date,amount,receipt_date,notified_date\n";

/// Received on time, all of it returned: nothing is eligible yet.
const ORIGINAL: &str = "E1,2026-08-06,120.00,2026-08-07,2026-08-10,no,no,,no,120.00,120.00,,no\n";
const FIRST_ORIGINAL: &str =
    "E1,2026-08-06,120.00,2026-08-07,2026-08-10,yes,no,,no,120.00,120.00,,no\n";
const RETURN_ALL: &str = "E1,2026-08-06,returned,2026-08-12,120.00,2026-08-10,\n";
const RETURN_PART: &str = "E1,2026-08-06,returned,2026-08-12,40.00,2026-08-10,\n";

fn with_events(rows: &str, events: &str, as_at: &str) -> Run {
    let dir = scratch();
    std::fs::write(dir.join("events.csv"), format!("{EVENTS_HEADER}{events}")).unwrap();
    check_in(&dir, rows, as_at, &["--events", "events.csv"])
}

fn resubmitted(on: &str, amount: &str) -> String {
    format!("E1,2026-08-06,received,{on},{amount},,\n")
}

/// A resubmission and the fund's record that it allocated all of it that day.
fn resent(on: &str, amount: &str) -> String {
    format!(
        "{}E1,2026-08-06,allocated,{on},{amount},{on},\n",
        resubmitted(on, amount)
    )
}

/// The fund's record that it allocated the $80 left of the 10 August receipt.
const KEPT_PART: &str = "E1,2026-08-06,allocated,2026-08-11,80.00,2026-08-10,\n";

fn refused(rows: &str, events: &str, wanted: &str) {
    let run = with_events(rows, events, "2026-09-20");
    assert_eq!(run.code, 1, "{events}: {}", run.stderr);
    assert!(run.stderr.contains(wanted), "{wanted}: {}", run.stderr);
    assert!(run.rows.is_empty());
}

#[test]
fn a_resubmission_received_inside_the_original_deadline_is_on_time() {
    let run = with_events(
        ORIGINAL,
        &format!("{RETURN_ALL}{}", resent("2026-08-14", "120.00")),
        "2026-09-20",
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.field(0, "verdict"), "ON_TIME");
    assert_eq!(run.field(0, "due_date"), "2026-08-17");
    let caveats = run.field(0, "caveats");
    assert!(
        caveats.contains(
            "contribution events: 2026-08-10 received $120.00; 2026-08-12 returned $120.00 of \
             the 2026-08-10 receipt; 2026-08-14 received $120.00"
        ),
        "{caveats}"
    );
}

#[test]
fn a_rejection_does_not_restart_the_deadline() {
    // A receipt on 13 August rejected on 18 August, the third business day. The
    // 20 August resubmission would be in time if the clock restarted on the
    // rejection; the deadline stays 17 August.
    let run = with_events(
        "E1,2026-08-06,120.00,2026-08-07,2026-08-13,no,no,,no,120.00,120.00,,no\n",
        &format!(
            "E1,2026-08-06,returned,2026-08-18,120.00,2026-08-13,\n{}",
            resent("2026-08-20", "120.00")
        ),
        "2026-09-20",
    );
    assert_eq!(run.code, 2);
    assert_eq!(run.field(0, "verdict"), "LATE");
    assert_eq!(run.field(0, "due_date"), "2026-08-17");
    assert_eq!(run.field(0, "days_late"), "3");
    assert_eq!(run.field(0, "lateness_measured_to"), "fund receipt");
    assert_eq!(run.field(0, "final_shortfall"), "0.00");
    // $120 at the 11.43% GIC rate for 18, 19 and 20 August.
    assert_eq!(run.field(0, "notional_earnings"), "0.11");
}

#[test]
fn a_return_with_no_resubmission_is_unresolved_on_both_sides_of_the_deadline() {
    let open = with_events(ORIGINAL, RETURN_ALL, "2026-08-13");
    assert_unresolved(&open, 0, "UNPAID or ON_TIME");
    let passed = with_events(ORIGINAL, RETURN_ALL, "2026-09-20");
    assert_unresolved(&passed, 0, "UNPAID or LATE");
    assert!(
        passed
            .field(0, "caveats")
            .contains("$120.00 of the $120.00")
    );
    assert!(passed.field(0, "caveats").contains("does not restart"));
}

#[test]
fn a_partial_return_counts_only_what_the_fund_kept() {
    let unresolved = with_events(ORIGINAL, RETURN_PART, "2026-09-20");
    assert_unresolved(&unresolved, 0, "UNPAID or LATE");
    assert!(
        unresolved
            .field(0, "caveats")
            .contains("$40.00 of the $120.00")
    );

    let timely = with_events(
        ORIGINAL,
        &format!("{RETURN_PART}{KEPT_PART}{}", resent("2026-08-14", "40.00")),
        "2026-09-20",
    );
    assert_eq!(timely.field(0, "verdict"), "ON_TIME");

    let late = with_events(
        ORIGINAL,
        &format!("{RETURN_PART}{KEPT_PART}{}", resent("2026-08-20", "40.00")),
        "2026-09-20",
    );
    assert_eq!(late.field(0, "verdict"), "LATE");
    assert_eq!(late.field(0, "final_shortfall"), "0.00");
    // Interest runs on the $40 shortfall alone, not the $120 liability.
    assert_eq!(late.field(0, "notional_earnings"), "0.04");
    assert!(
        late.field(0, "notes")
            .contains("reduces the base shortfall to $40.00")
    );

    let short = with_events(
        ORIGINAL,
        &format!("{RETURN_PART}{}", resubmitted("2026-08-14", "30.00")),
        "2026-09-20",
    );
    assert_unresolved(&short, 0, "UNPAID or LATE");
    assert!(short.field(0, "caveats").contains("$10.00 of the $120.00"));
}

#[test]
fn money_cannot_be_counted_twice() {
    refused(
        ORIGINAL,
        &resubmitted("2026-08-14", "120.00"),
        "count the money twice",
    );
    refused(
        ORIGINAL,
        &resubmitted("2026-08-10", "120.00"),
        "already recorded",
    );
    refused(
        ORIGINAL,
        &format!("{RETURN_PART}{}", resubmitted("2026-08-14", "40.01")),
        "count the money twice",
    );
    refused(
        ORIGINAL,
        &format!("{RETURN_PART}{RETURN_PART}"),
        "already has a return",
    );
    refused(
        ORIGINAL,
        "E1,2026-08-06,returned,2026-08-12,120.01,2026-08-10,\n",
        "more than the",
    );
}

#[test]
fn malformed_events_are_refused_and_never_dropped() {
    let refuse = |events: &str, wanted: &str| refused(ORIGINAL, events, wanted);
    refuse(
        "E2,2026-08-06,received,2026-08-14,5.00,,\n",
        "no contribution row",
    );
    refuse(
        "e1,2026-08-06,received,2026-08-14,5.00,,\n",
        "no contribution row",
    );
    refuse(
        "E1,2026-08-07,received,2026-08-14,5.00,,\n",
        "no contribution row",
    );
    refuse(
        "E1,2026-08-06,reversed,2026-08-14,5.00,,\n",
        "not received, returned or allocated",
    );
    refuse("E1,2026-08-06,received,,5.00,,\n", "event_date is empty");
    refuse(
        "E1,2026-08-06,received,2026-13-40,5.00,,\n",
        "cannot read event_date",
    );
    refuse("E1,2026-08-06,received,2026-08-14,,,\n", "amount is empty");
    refuse(
        "E1,2026-08-06,received,2026-08-14,-5.00,,\n",
        "amount is negative",
    );
    refuse(
        "E1,2026-08-06,received,2026-08-14,0.00,,\n",
        "more than nil",
    );
    refuse("E1,2026-08-06,received,2026-08-14,five,,\n", "as an amount");
    refuse(
        "E1,2026-08-06,returned,2026-08-12,5.00,2026-08-11,\n",
        "does not record",
    );
    refuse(
        "E1,2026-08-06,returned,2026-08-09,5.00,2026-08-10,\n",
        "before the receipt",
    );
    refuse(
        "E1,2026-08-06,returned,2026-08-12,5.00,,\n",
        "Fill receipt_date",
    );
    refuse(
        "E1,2026-08-06,returned,2026-08-12,5.00,2026-08-10,2026-08-09\n",
        "cannot be told",
    );
    refuse(
        "E1,2026-08-06,received,2026-08-14,5.00,2026-08-10,\n",
        "takes no receipt_date or notified_date",
    );
    refuse("E1,2026-08-06,received,2026-08-14\n", "stops early");
    refuse(
        "E1,2026-08-06,received,2026-08-14,5.00,,,extra\n",
        "more values",
    );
    refused(
        &format!("{ORIGINAL}{ORIGINAL}"),
        RETURN_ALL,
        "2 contribution rows",
    );
    refused(
        "E1,2026-08-06,120.00,,,no,no,,yes,,,,no\n",
        "E1,2026-08-06,received,2026-08-14,120.00,,\n",
        "tested for lateness",
    );

    let dir = scratch();
    std::fs::write(
        dir.join("events.csv"),
        "employee_id,payment_date,event,event_date,amount,why\n",
    )
    .unwrap();
    let run = check_in(&dir, ORIGINAL, "2026-09-20", &["--events", "events.csv"]);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("'why'"), "{}", run.stderr);

    let dir = scratch();
    std::fs::write(dir.join("events.csv"), EVENTS_HEADER).unwrap();
    let run = check_in(&dir, ORIGINAL, "2026-09-20", &["--events", "events.csv"]);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("no data rows"), "{}", run.stderr);

    let dir = scratch();
    let run = check_in(&dir, ORIGINAL, "2026-09-20", &["--events", "absent.csv"]);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("absent.csv"), "{}", run.stderr);
}

/// 20 business days after 6 August 2026 is 3 September 2026. A wholly returned
/// amount was never a contribution, so the resubmission is still the first
/// eligible one; a partly kept amount is the first, and takes the extension.
#[test]
fn only_the_first_eligible_contribution_has_the_20_business_days() {
    let whole = with_events(
        FIRST_ORIGINAL,
        &format!("{RETURN_ALL}{}", resent("2026-09-01", "120.00")),
        "2026-09-20",
    );
    assert_eq!(whole.field(0, "due_date"), "2026-09-03");
    assert_eq!(whole.field(0, "verdict"), "ON_TIME");

    let whole_late = with_events(
        FIRST_ORIGINAL,
        &format!("{RETURN_ALL}{}", resent("2026-09-04", "120.00")),
        "2026-09-20",
    );
    assert_eq!(whole_late.field(0, "verdict"), "LATE");
    assert_eq!(whole_late.field(0, "days_late"), "1");

    let inside_usual = with_events(
        FIRST_ORIGINAL,
        &format!("{RETURN_PART}{KEPT_PART}{}", resent("2026-08-17", "40.00")),
        "2026-09-20",
    );
    assert_eq!(inside_usual.field(0, "verdict"), "ON_TIME");

    let gap = with_events(
        FIRST_ORIGINAL,
        &format!("{RETURN_PART}{KEPT_PART}{}", resent("2026-08-18", "40.00")),
        "2026-09-20",
    );
    assert_unresolved(&gap, 0, "LATE or ON_TIME");
    assert!(gap.field(0, "caveats").contains("paragraph 54"));

    let after = with_events(
        FIRST_ORIGINAL,
        &format!("{RETURN_PART}{KEPT_PART}{}", resent("2026-09-04", "40.00")),
        "2026-09-20",
    );
    assert_eq!(after.field(0, "verdict"), "LATE");
    assert_eq!(after.field(0, "days_late"), "1");
}

#[test]
fn late_notification_changes_no_figure() {
    let events = |told: &str| {
        format!(
            "E1,2026-08-06,returned,2026-08-12,120.00,2026-08-10,{told}\n{}",
            resent("2026-08-25", "120.00")
        )
    };
    let prompt = with_events(ORIGINAL, &events("2026-08-12"), "2026-09-20");
    let late = with_events(ORIGINAL, &events("2026-08-24"), "2026-09-20");
    for field in [
        "verdict",
        "due_date",
        "days_late",
        "final_shortfall",
        "notional_earnings",
    ] {
        assert_eq!(prompt.field(0, field), late.field(0, field), "{field}");
    }
    assert_eq!(late.field(0, "verdict"), "LATE");
    assert!(!prompt.field(0, "caveats").contains("Late notice"));
    let caveats = late.field(0, "caveats");
    assert!(
        caveats.contains("Late notice does not move the deadline"),
        "{caveats}"
    );
    assert!(caveats.contains("PCG 2026/1"), "{caveats}");

    // PCG 2026/1 stops at QE days to 30 June 2027.
    let later = with_events(
        "E1,2027-07-06,120.00,2027-07-07,2027-07-08,no,no,,no,120.00,120.00,,no\n",
        "E1,2027-07-06,returned,2027-07-09,120.00,2027-07-08,2027-08-30\n",
        "2027-08-31",
    );
    assert!(later.field(0, "caveats").contains("Late notice"));
    assert!(!later.field(0, "caveats").contains("PCG 2026/1"));
}

#[test]
fn the_as_at_date_cannot_bring_a_returned_amount_back() {
    let before_return = with_events(ORIGINAL, RETURN_ALL, "2026-08-11");
    assert_unresolved(&before_return, 0, "UNPAID or ON_TIME");
    assert!(
        before_return
            .field(0, "caveats")
            .contains("after the as-at date")
    );

    let before_resubmission = with_events(
        ORIGINAL,
        &format!("{RETURN_ALL}{}", resubmitted("2026-08-14", "120.00")),
        "2026-08-13",
    );
    assert_unresolved(&before_resubmission, 0, "UNPAID or ON_TIME");
    assert!(
        before_resubmission
            .field(0, "caveats")
            .contains("ignored for this")
    );
}

#[test]
fn a_late_receipt_after_the_assessment_does_not_reduce_the_final_shortfall() {
    let dir = scratch();
    std::fs::write(
        dir.join("events.csv"),
        format!(
            "{EVENTS_HEADER}{RETURN_ALL}{}{}",
            resent("2026-08-20", "70.00"),
            resent("2026-09-10", "50.00")
        ),
    )
    .unwrap();
    let run = check_in(
        &dir,
        ORIGINAL,
        "2026-09-20",
        &["--events", "events.csv", "--assessment-date", "2026-09-01"],
    );
    assert_eq!(run.field(0, "verdict"), "LATE");
    assert_eq!(run.field(0, "final_shortfall"), "50.00");
    // $120 at 11.43% a year: 18 to 31 August, 14 days; the $70 of 20 August
    // cannot stop the shortfall while $50 stays unpaid at the assessment.
    assert_eq!(run.field(0, "notional_earnings"), "0.53");

    // With no assessment yet, both receipts count and earnings run to 10 September.
    let unassessed = check_in(&dir, ORIGINAL, "2026-09-20", &["--events", "events.csv"]);
    assert_eq!(unassessed.field(0, "final_shortfall"), "0.00");
    assert_eq!(unassessed.field(0, "notional_earnings"), "0.91");
}

#[test]
fn a_return_of_money_only_ever_seen_as_a_remittance_never_reads_at_risk() {
    let remitted = "E1,2026-08-06,120.00,2026-08-07,,no,no,,no,120.00,120.00,,no\n";
    let no_receipt = "E1,2026-08-06,returned,2026-08-12,120.00,,2026-08-12\n";
    let alone = with_events(remitted, no_receipt, "2026-08-13");
    assert_unresolved(&alone, 0, "UNPAID or ON_TIME");
    let resent = with_events(
        remitted,
        &format!("{no_receipt}{}", resent("2026-08-14", "120.00")),
        "2026-09-20",
    );
    assert_eq!(resent.field(0, "verdict"), "ON_TIME");
    // The event is the receipt, so the row needs no further confirmation.
    assert_eq!(resent.code, 0, "{}", resent.stderr);
}

#[test]
fn the_flag_is_answered_only_by_a_dated_return() {
    let row = flagged(TIMELY_RECEIPT);
    let answered = with_events(
        &row,
        &format!("{RETURN_ALL}{}", resent("2026-08-14", "120.00")),
        "2026-09-20",
    );
    assert_eq!(answered.field(0, "verdict"), "ON_TIME");

    let only_a_receipt = with_events(
        "E1,2026-08-06,120.00,2026-08-07,,no,no,,no,120.00,120.00,,yes\n",
        &resubmitted("2026-08-14", "120.00"),
        "2026-09-20",
    );
    assert_unresolved(&only_a_receipt, 0, "UNPAID or ON_TIME");
}

#[test]
fn a_receipt_with_no_amount_cannot_be_reconciled() {
    let run = with_events(
        "E1,2026-08-06,120.00,2026-08-07,2026-08-10,no,no,,no,,,,no\n",
        RETURN_ALL,
        "2026-09-20",
    );
    assert_unresolved(&run, 0, "UNPAID or ON_TIME");
    assert!(run.field(0, "caveats").contains("Supply matched_amount"));
}

#[test]
fn only_what_the_fund_kept_confirms_item_4() {
    let second = "E1,2026-08-13,120.00,2026-08-14,2026-08-28,no,no,,no,120.00,120.00,,no\n";
    let rows = format!("{FIRST_ORIGINAL}{second}");
    let returned = with_events(&rows, RETURN_ALL, "2026-09-20");
    assert_ne!(returned.field(1, "pathway"), "ITEM4_ALIGNED");
    assert_ne!(returned.field(1, "verdict"), "ON_TIME");

    // A resubmission alone shows no eligible contribution for item 4 to rest on.
    let unallocated = with_events(
        &rows,
        &format!("{RETURN_ALL}{}", resubmitted("2026-08-20", "120.00")),
        "2026-09-20",
    );
    assert_ne!(unallocated.field(1, "verdict"), "ON_TIME");

    let allocated_late = with_events(
        &rows,
        &format!(
            "{RETURN_ALL}{}E1,2026-08-06,allocated,2026-09-10,120.00,2026-08-20,\n",
            resubmitted("2026-08-20", "120.00")
        ),
        "2026-09-20",
    );
    assert_ne!(allocated_late.field(1, "verdict"), "ON_TIME");

    let allocated = with_events(
        &rows,
        &format!("{RETURN_ALL}{}", resent("2026-08-20", "120.00")),
        "2026-09-20",
    );
    assert_eq!(allocated.field(1, "pathway"), "ITEM4_ALIGNED");
    assert_eq!(allocated.field(1, "verdict"), "ON_TIME");
}

#[test]
fn an_event_inside_the_transition_needs_the_confirmation() {
    let dir = scratch();
    std::fs::write(
        dir.join("pay.csv"),
        format!("{HEADER}E1,2026-07-09,120.00,,,no,no,,no,,,,no\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("events.csv"),
        format!("{EVENTS_HEADER}E1,2026-07-09,received,2026-07-15,120.00,,\n"),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["pay.csv", "--events", "events.csv", "--as-at", "2026-09-20"])
        .args(["-o", "report.csv"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("LCR 2026/1"));
}

#[test]
fn the_evidence_pack_carries_the_ordered_trail_without_the_employee() {
    let dir = scratch();
    std::fs::write(dir.join("pay.csv"), format!("{HEADER}{ORIGINAL}")).unwrap();
    std::fs::write(
        dir.join("events.csv"),
        format!(
            "{EVENTS_HEADER}{RETURN_ALL}{}",
            resent("2026-08-20", "120.00")
        ),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["evidence-pack", "pay.csv", "--events", "events.csv"])
        .args(["--as-at", "2026-09-20", "-o", "pack"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let exceptions = std::fs::read_to_string(dir.join("pack/exceptions.json")).unwrap();
    assert!(exceptions.contains("contribution events: 2026-08-10 received $120.00"));
    assert!(exceptions.contains("\"verdict\": \"LATE\""));
    assert!(!exceptions.contains("E1"));
}

#[test]
fn the_report_refuses_to_overwrite_the_events_file() {
    let dir = scratch();
    std::fs::write(
        dir.join("events.csv"),
        format!("{EVENTS_HEADER}{RETURN_ALL}"),
    )
    .unwrap();
    std::fs::write(dir.join("pay.csv"), format!("{HEADER}{ORIGINAL}")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["pay.csv", "--events", "events.csv", "--as-at", "2026-09-20"])
        .args(["-o", "events.csv"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--events input"));
}

#[test]
fn the_api_takes_events_and_applies_the_same_rules() {
    use std::io::Write;
    let call = |events: Value| -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
            .arg("assess")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let input = serde_json::json!({
            "as_at": "2026-09-20",
            "transition_allocation_confirmed": true,
            "lines": [{
                "employee_id": "E1", "qe_day": "2026-08-06", "sg_amount": "120.00",
                "received": "2026-08-10", "matched_amount": "120.00", "row": 2,
                "first_to_fund": false, "out_of_cycle": false, "db_interest": false,
                "events": events,
            }],
        });
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.to_string().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&out.stderr).into_owned()))
    };
    let returned = serde_json::json!({
        "event": "returned", "event_date": "2026-08-12", "amount": "120.00",
        "receipt_date": "2026-08-10",
    });
    let answer = call(serde_json::json!([returned]));
    assert_eq!(answer[0]["verdict"], "UNKNOWN", "{answer}");
    assert_eq!(answer[0]["line"]["events"][0]["event"], "returned");

    let doubled = call(serde_json::json!([
        {"event": "received", "event_date": "2026-08-14", "amount": "120.00"}
    ]));
    assert!(
        doubled.to_string().contains("count the money twice"),
        "{doubled}"
    );
}

// --- findings from the first independent review -----------------------------

/// Run an API command and return its answer, or its error text as a string.
fn api(command: &str, input: &Value) -> Value {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .arg(command)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&out.stderr).into_owned()))
}

const LEGACY_MAPPING: [(&str, &str); 12] = [
    ("employee_id", "employee_id"),
    ("qe_day", "payment_date"),
    ("sg_amount", "sg_amount"),
    ("remitted", "remitted_date"),
    ("received", "fund_received_date"),
    ("first_to_fund", "first_contribution_to_fund"),
    ("out_of_cycle", "out_of_cycle"),
    ("next_standard_qe_day", "next_standard_payday"),
    ("db_interest", "defined_benefit"),
    ("remitted_amount", "remitted_amount"),
    ("matched_amount", "matched_amount"),
    ("join_caveats", "join_caveats"),
];

fn legacy_parse(path: &Path) -> Value {
    api(
        "parse-rows",
        &serde_json::json!({ "path": path, "mapping": LEGACY_MAPPING }),
    )
}

#[test]
fn a_mapping_that_cannot_read_the_return_column_refuses_the_file() {
    let dir = scratch();
    let carrying = dir.join("carrying.csv");
    std::fs::write(&carrying, format!("{HEADER}{}", flagged(TIMELY_RECEIPT))).unwrap();
    let answer = legacy_parse(&carrying);
    assert!(
        answer.to_string().contains("return column"),
        "a returned row must not parse as unflagged: {answer}"
    );

    let plain = dir.join("plain.csv");
    let without = HEADER.replace(",returned_by_fund", "");
    let row = TIMELY_RECEIPT.strip_suffix(',').unwrap();
    std::fs::write(&plain, format!("{without}{row}\n")).unwrap();
    let answer = legacy_parse(&plain);
    assert_eq!(answer[0]["employee_id"], "E1", "{answer}");
    assert!(answer[0].get("returned_by_fund").is_none(), "{answer}");
}

#[test]
fn a_differently_written_return_column_is_refused_until_mapped() {
    for column in ["Returned By Fund", "Returned  By Fund", "returned-by-fund"] {
        let dir = scratch();
        std::fs::write(
            dir.join("pay.csv"),
            format!(
                "{}{}",
                HEADER.replace("returned_by_fund", column),
                flagged(TIMELY_RECEIPT)
            ),
        )
        .unwrap();
        let run = |extra: &[&str]| {
            Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
                .args(["pay.csv", "--as-at", "2026-09-20", "-o", "report.csv"])
                .args(extra)
                .current_dir(&dir)
                .output()
                .unwrap()
        };
        let refused = run(&[]);
        assert_eq!(refused.status.code(), Some(1), "{column}");
        assert!(String::from_utf8_lossy(&refused.stderr).contains(column));
        let mapped = run(&["--map", &format!("returned_by_fund={column}")]);
        assert_eq!(
            mapped.status.code(),
            Some(2),
            "{}",
            String::from_utf8_lossy(&mapped.stderr)
        );
    }
}

#[test]
fn json_events_are_validated_like_csv_events() {
    let line = |event: Value| {
        serde_json::json!({
            "as_at": "2026-09-20",
            "transition_allocation_confirmed": true,
            "lines": [{
                "employee_id": "E1", "qe_day": "2026-08-06", "sg_amount": "120.00",
                "row": 2, "first_to_fund": false, "out_of_cycle": false,
                "db_interest": false, "events": [event],
            }],
        })
    };
    for field in ["notified_date", "receipt_date"] {
        let answer = api(
            "assess",
            &line(serde_json::json!({
                "event": "received", "event_date": "2026-08-14", "amount": "120.00",
                field: "2026-08-12",
            })),
        );
        assert!(
            answer
                .to_string()
                .contains("takes no receipt_date or notified_date"),
            "{field}: {answer}"
        );
    }
    let answer = api(
        "assess",
        &line(serde_json::json!({
            "event": "reversed", "event_date": "2026-08-14", "amount": "120.00",
        })),
    );
    assert!(
        answer
            .to_string()
            .contains("not received, returned or allocated"),
        "{answer}"
    );
    let answer = api(
        "assess",
        &line(serde_json::json!({
            "event": "allocated", "event_date": "2026-08-14", "amount": "120.00",
        })),
    );
    assert!(
        answer.to_string().contains("must name its receipt"),
        "{answer}"
    );
}

/// Received on time, returned, and resent on Friday 30 October 2026. A Victorian
/// fund's third business day is 5 November, because of Melbourne Cup Day on the
/// 3rd; the national calendar's is the 4th. An SMSF has longer again.
const BEFORE_THE_CUP: &str =
    "E2,2026-10-23,120.00,2026-10-24,2026-10-26,no,no,,no,120.00,120.00,,no\n";
const RESENT_BEFORE_THE_CUP: &str = "E2,2026-10-23,returned,2026-10-27,120.00,2026-10-26,\n\
                                     E2,2026-10-23,received,2026-10-30,120.00,,\n";

#[test]
fn a_receipt_on_a_line_with_a_return_needs_the_funds_allocation() {
    for as_at in ["2026-10-30", "2026-11-05", "2026-12-20", "2028-03-01"] {
        let run = with_events(BEFORE_THE_CUP, RESENT_BEFORE_THE_CUP, as_at);
        assert_unresolved(&run, 0, "UNPAID or ON_TIME");
        let caveats = run.field(0, "caveats");
        assert!(caveats.contains("Supply an allocated event"), "{caveats}");
        assert!(caveats.contains("2026-10-30"), "{caveats}");
    }

    let allocated = with_events(
        BEFORE_THE_CUP,
        &format!("{RESENT_BEFORE_THE_CUP}E2,2026-10-23,allocated,2026-11-02,120.00,2026-10-30,\n"),
        "2026-11-03",
    );
    assert_eq!(allocated.code, 0, "{}", allocated.stderr);
    assert_eq!(allocated.field(0, "verdict"), "ON_TIME");
    assert!(
        allocated
            .field(0, "caveats")
            .contains("allocated $120.00 of the 2026-10-30 receipt")
    );
    assert!(!allocated.field(0, "caveats").contains("assumes"));

    let not_yet = with_events(
        BEFORE_THE_CUP,
        &format!("{RESENT_BEFORE_THE_CUP}E2,2026-10-23,allocated,2026-11-02,120.00,2026-10-30,\n"),
        "2026-11-01",
    );
    assert_unresolved(&not_yet, 0, "UNPAID or ON_TIME");

    let part = with_events(
        BEFORE_THE_CUP,
        &format!("{RESENT_BEFORE_THE_CUP}E2,2026-10-23,allocated,2026-11-02,100.00,2026-10-30,\n"),
        "2026-12-20",
    );
    assert_unresolved(&part, 0, "UNPAID or ON_TIME");
}

#[test]
fn a_partly_returned_receipt_needs_its_kept_part_allocated_too() {
    let resend_only = with_events(
        ORIGINAL,
        &format!("{RETURN_PART}{}", resent("2026-08-14", "40.00")),
        "2026-09-20",
    );
    assert_unresolved(&resend_only, 0, "UNPAID or ON_TIME");
    assert!(resend_only.field(0, "caveats").contains("2026-08-10"));
}

#[test]
fn an_allocation_after_the_deadline_does_not_show_a_timely_receipt_was_on_time() {
    let run = with_events(
        ORIGINAL,
        &format!(
            "{RETURN_ALL}{}E1,2026-08-06,allocated,2026-08-18,120.00,2026-08-14,\n",
            resubmitted("2026-08-14", "120.00")
        ),
        "2026-09-20",
    );
    assert_unresolved(&run, 0, "LATE or ON_TIME");
    assert!(run.field(0, "caveats").contains("paragraph 20"));
}

#[test]
fn a_late_receipt_with_no_allocation_is_unresolved_and_with_one_is_late() {
    let bare = with_events(
        ORIGINAL,
        &format!("{RETURN_ALL}{}", resubmitted("2026-08-20", "120.00")),
        "2026-09-20",
    );
    assert_unresolved(&bare, 0, "UNPAID or LATE");

    let allocated_later = with_events(
        ORIGINAL,
        &format!(
            "{RETURN_ALL}{}E1,2026-08-06,allocated,2026-08-21,120.00,2026-08-20,\n",
            resubmitted("2026-08-20", "120.00")
        ),
        "2026-09-20",
    );
    assert_eq!(allocated_later.field(0, "verdict"), "LATE");
    assert!(
        allocated_later
            .field(0, "caveats")
            .contains("measured to the receipt")
    );
}

#[test]
fn a_line_with_no_return_keeps_its_reading_with_no_allocation() {
    let extra = "E1,2026-08-06,received,2026-08-11,40.00,,\n";
    let row = "E1,2026-08-06,120.00,2026-08-07,2026-08-10,no,no,,no,80.00,80.00,,no\n";
    for as_at in ["2026-08-11", "2026-09-20"] {
        let run = with_events(row, extra, as_at);
        assert_eq!(run.code, 0, "{}", run.stderr);
        assert_eq!(run.field(0, "verdict"), "ON_TIME");
    }
}

#[test]
fn allocated_events_are_validated() {
    let row = |rest: &str| format!("E1,2026-08-06,allocated,{rest}\n");
    refused(
        ORIGINAL,
        &row("2026-08-11,120.00,2026-08-09,"),
        "An allocation needs a matching kept receipt",
    );
    refused(
        ORIGINAL,
        &row("2026-08-09,120.00,2026-08-10,"),
        "before the receipt it names",
    );
    refused(
        ORIGINAL,
        &row("2026-08-11,120.01,2026-08-10,"),
        "more than the $120.00 the fund kept",
    );
    refused(
        ORIGINAL,
        &format!("{RETURN_ALL}{}", row("2026-08-11,120.00,2026-08-10,")),
        "more than the $0.00 the fund kept",
    );
    refused(
        ORIGINAL,
        &format!("{RETURN_PART}{}", row("2026-08-11,120.00,2026-08-10,")),
        "more than the $80.00 the fund kept",
    );
    refused(
        ORIGINAL,
        &format!(
            "{}{}",
            row("2026-08-11,70.00,2026-08-10,"),
            row("2026-08-11,70.00,2026-08-10,")
        ),
        "more than the $120.00 the fund kept",
    );
    refused(
        ORIGINAL,
        &row("2026-08-11,120.00,,"),
        "must name its receipt in receipt_date",
    );
    refused(
        ORIGINAL,
        &row("2026-08-11,120.00,2026-08-10,2026-08-11"),
        "takes no notified_date",
    );
    refused(
        ORIGINAL,
        &row("2026-08-11,0.00,2026-08-10,"),
        "more than nil",
    );
    // A row receipt with no amount has nothing kept to match.
    refused(
        "E1,2026-08-06,120.00,2026-08-07,2026-08-10,no,no,,no,,,,no\n",
        &row("2026-08-11,120.00,2026-08-10,"),
        "An allocation needs a matching kept receipt",
    );
}

#[test]
fn a_shortfall_after_the_deadline_states_the_base_shortfall_and_no_estimate() {
    let all = with_events(ORIGINAL, RETURN_ALL, "2026-09-20");
    assert_unresolved(&all, 0, "UNPAID or LATE");
    let caveats = all.field(0, "caveats");
    assert!(caveats.contains("base shortfall of $120.00"), "{caveats}");
    assert!(caveats.contains("s 18C"), "{caveats}");
    assert!(caveats.contains("s 19A"), "{caveats}");
    assert_eq!(all.field(0, "notional_earnings"), "");

    let part = with_events(ORIGINAL, RETURN_PART, "2026-09-20");
    assert_unresolved(&part, 0, "UNPAID or LATE");
    assert!(
        part.field(0, "caveats")
            .contains("base shortfall of $40.00")
    );

    let open = with_events(ORIGINAL, RETURN_ALL, "2026-08-13");
    assert!(!open.field(0, "caveats").contains("base shortfall"));
}

#[test]
fn a_return_after_the_funds_window_is_not_read_as_a_rejection() {
    // 10 August plus 3 business days is 13 August, so 14 August is outside.
    let late_return = "E1,2026-08-06,returned,2026-08-14,120.00,2026-08-10,\n";
    let on_time_resend = with_events(
        ORIGINAL,
        &format!("{late_return}{}", resubmitted("2026-08-17", "120.00")),
        "2026-09-20",
    );
    assert_unresolved(&on_time_resend, 0, "UNPAID or ON_TIME");
    let caveats = on_time_resend.field(0, "caveats");
    assert!(caveats.contains("more than 3 business days"), "{caveats}");
    assert!(
        caveats.contains("does not say the credit is gone"),
        "{caveats}"
    );

    let late_resend = with_events(
        ORIGINAL,
        &format!("{late_return}{}", resubmitted("2026-08-20", "120.00")),
        "2026-09-20",
    );
    assert_unresolved(&late_resend, 0, "LATE or ON_TIME");

    let alone = with_events(ORIGINAL, late_return, "2026-09-20");
    assert_unresolved(&alone, 0, "UNPAID or ON_TIME");
    let caveats = alone.field(0, "caveats");
    assert!(!caveats.contains("no eligible fund receipt"), "{caveats}");
    assert!(!caveats.contains("base shortfall"), "{caveats}");

    // The third business day is still inside.
    let inside = with_events(
        ORIGINAL,
        &format!(
            "E1,2026-08-06,returned,2026-08-13,120.00,2026-08-10,\n{}",
            resent("2026-08-17", "120.00")
        ),
        "2026-09-20",
    );
    assert_eq!(inside.field(0, "verdict"), "ON_TIME");
}

#[test]
fn an_allocation_at_or_after_the_assessment_withholds_the_late_figures() {
    let dir = scratch();
    let run = |allocated_on: &str, as_at: &str| {
        std::fs::write(
            dir.join("events.csv"),
            format!(
                "{EVENTS_HEADER}{RETURN_ALL}{}E1,2026-08-06,allocated,{allocated_on},120.00,\
                 2026-08-20,\n",
                resubmitted("2026-08-20", "120.00")
            ),
        )
        .unwrap();
        check_in(
            &dir,
            ORIGINAL,
            as_at,
            &["--events", "events.csv", "--assessment-date", "2026-08-21"],
        )
    };
    for allocated_on in ["2026-08-24", "2026-08-21"] {
        let held = run(allocated_on, "2026-08-24");
        assert_unresolved(&held, 0, "UNPAID or LATE");
        let caveats = held.field(0, "caveats");
        assert!(caveats.contains("s 18D"), "{caveats}");
        assert!(caveats.contains("are withheld"), "{caveats}");
        assert_eq!(held.field(0, "notional_earnings"), "");
    }

    let before = run("2026-08-20", "2026-08-24");
    assert_eq!(before.field(0, "verdict"), "LATE");
    assert_eq!(before.field(0, "final_shortfall"), "0.00");
}

#[test]
fn unequal_same_day_allocations_of_one_receipt_add_up() {
    let part = |amount: &str| format!("E1,2026-08-06,allocated,2026-08-11,{amount},2026-08-10,\n");
    let run = with_events(
        ORIGINAL,
        &format!("{}{}", part("70.00"), part("50.00")),
        "2026-09-20",
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.field(0, "verdict"), "ON_TIME");
    let added = "several allocated entries dated 2026-08-11 for the 2026-08-10 receipt";
    let caveats = run.field(0, "caveats");
    assert!(caveats.contains(added), "{caveats}");
    assert!(caveats.contains("not duplicates"), "{caveats}");

    let on_different_days = with_events(
        ORIGINAL,
        &format!(
            "{}E1,2026-08-06,allocated,2026-08-12,60.00,2026-08-10,\n",
            part("60.00")
        ),
        "2026-09-20",
    );
    assert_eq!(on_different_days.field(0, "verdict"), "ON_TIME");
    assert!(
        !on_different_days
            .field(0, "caveats")
            .contains("several allocated")
    );

    let on_a_return = with_events(
        ORIGINAL,
        &format!(
            "{RETURN_PART}{}{}{}",
            part("30.00"),
            part("50.00"),
            resent("2026-08-14", "40.00")
        ),
        "2026-09-20",
    );
    assert_eq!(on_a_return.code, 0, "{}", on_a_return.stderr);
    assert_eq!(on_a_return.field(0, "verdict"), "ON_TIME");
}

#[test]
fn identical_allocation_entries_are_refused() {
    let part = |amount: &str| format!("E1,2026-08-06,allocated,2026-08-11,{amount},2026-08-10,\n");
    refused(
        ORIGINAL,
        &format!(
            "{RETURN_PART}{}{}{}",
            part("40.00"),
            part("40.00"),
            resent("2026-08-14", "40.00")
        ),
        "consolidate genuine equal allocations into one row",
    );
    // The same amount written differently is still the same entry.
    refused(
        ORIGINAL,
        &format!("{}{}", part("60"), part("60.00")),
        "repeats an allocation of $60.00",
    );

    // A late receipt: without the refusal these would offset the whole shortfall.
    let dir = scratch();
    let late = "E1,2026-08-06,allocated,2026-08-20,60.00,2026-08-20,\n";
    std::fs::write(
        dir.join("events.csv"),
        format!(
            "{EVENTS_HEADER}{RETURN_ALL}{}{late}{late}",
            resubmitted("2026-08-20", "120.00")
        ),
    )
    .unwrap();
    let run = check_in(
        &dir,
        ORIGINAL,
        "2026-09-20",
        &["--events", "events.csv", "--assessment-date", "2026-09-01"],
    );
    assert_eq!(run.code, 1, "{}", run.stderr);
    assert!(
        run.stderr.contains("repeats an allocation"),
        "{}",
        run.stderr
    );
    assert!(run.rows.is_empty());
}
