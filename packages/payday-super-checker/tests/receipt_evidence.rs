// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The evidence record on a line (`receipt_evidence`, `mvr_status`,
//! `fund_error_ref`) and the first-year note. They annotate a line and never
//! decide it. All facts are fabricated.

use std::process::Command;

use payday_super_checker::api;
use serde_json::{Value, json};

/// A 6 August 2026 payday is due 17 August 2026.
fn base(received: Option<&str>, remitted: Option<&str>) -> Value {
    let mut line = json!({"employee_id": "E1", "qe_day": "2026-08-06", "sg_amount": "120.00"});
    if let Some(day) = remitted {
        line["remitted"] = json!(day);
        line["remitted_amount"] = json!("120.00");
    }
    if let Some(day) = received {
        line["received"] = json!(day);
        line["matched_amount"] = json!("120.00");
    }
    line
}

fn assess(line: &Value, as_at: &str) -> Result<Value, String> {
    api::evaluate(
        "assess",
        &json!({"lines": [line], "as_at": as_at, "allow_stale_gic": true}),
    )
    .map(|results| results[0].clone())
}

fn with(line: &Value, fields: &[(&str, &str)]) -> Value {
    let mut out = line.clone();
    for (field, value) in fields {
        out[*field] = json!(value);
    }
    out
}

/// What a verdict rests on: everything but the notes, caveats and echoed line.
fn decision(result: &Value) -> Value {
    let mut out = result.clone();
    for key in ["notes", "caveats", "line"] {
        out.as_object_mut().unwrap().remove(key);
    }
    out
}

fn texts(result: &Value, key: &str) -> String {
    result[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect::<Vec<_>>()
        .join(" | ")
}

#[test]
fn no_evidence_value_changes_a_verdict_or_a_figure() {
    let mut returned = base(Some("2026-08-10"), Some("2026-08-07"));
    returned["returned_by_fund"] = json!(true);
    let cases = [
        (
            base(Some("2026-08-10"), Some("2026-08-07")),
            "2026-08-20",
            "ON_TIME",
        ),
        (base(None, Some("2026-08-06")), "2026-08-20", "AT_RISK"),
        (
            base(Some("2026-08-25"), Some("2026-08-24")),
            "2026-09-01",
            "LATE",
        ),
        (base(None, None), "2026-09-01", "UNPAID"),
        (returned, "2026-08-20", "UNKNOWN"),
    ];
    let mut compared = 0;
    for (line, as_at, verdict) in &cases {
        let plain = assess(line, as_at).unwrap();
        assert_eq!(plain["verdict"], *verdict);
        let has_receipt = !line["received"].is_null();
        for receipt in ["", "fund", "clearing_house", "sent"] {
            // The contradictions the next test refuses.
            if (has_receipt && receipt == "sent")
                || (!has_receipt && (receipt == "fund" || receipt == "clearing_house"))
                || (receipt == "sent" && line["remitted"].is_null())
            {
                continue;
            }
            for mvr in ["", "matched", "not_matched", "not_sent"] {
                for error in ["", "none", "CTER-0001"] {
                    let fields = [
                        ("receipt_evidence", receipt),
                        ("mvr_status", mvr),
                        ("fund_error_ref", error),
                    ];
                    let annotated = assess(&with(line, &fields), as_at).unwrap();
                    assert_eq!(decision(&annotated), decision(&plain), "{fields:?}");
                    compared += 1;
                }
            }
        }
    }
    // The 3 lines with a receipt allow 3 receipt values, the remitted line 2 and
    // the unpaid line 1, each with 12 pairs of the other 2 fields.
    assert_eq!(compared, (3 * 3 + 2 + 1) * 12);
}

#[test]
fn evidence_the_line_contradicts_is_refused() {
    let received = base(Some("2026-08-10"), Some("2026-08-07"));
    let sent_only = base(None, Some("2026-08-06"));
    let nothing = base(None, None);
    for (line, value, wanted) in [
        (
            &sent_only,
            "fund",
            "row 0: receipt_evidence=fund names where a fund receipt",
        ),
        (
            &sent_only,
            "clearing_house",
            "row 0: receipt_evidence=clearing_house names where a fund receipt",
        ),
        (
            &received,
            "sent",
            "row 0: receipt_evidence=sent says the fund's receipt is not confirmed",
        ),
        (
            &nothing,
            "sent",
            "row 0: receipt_evidence=sent needs remitted_date",
        ),
    ] {
        let err = assess(&with(line, &[("receipt_evidence", value)]), "2026-08-20").unwrap_err();
        assert!(err.contains(wanted), "{err}");
    }
    // A received event is a recorded receipt too.
    let mut evented = with(&sent_only, &[("receipt_evidence", "fund")]);
    evented["events"] = json!([
        {"event": "received", "event_date": "2026-08-10", "amount": "120.00"},
        {"event": "allocated", "event_date": "2026-08-10", "amount": "120.00", "receipt_date": "2026-08-10"},
    ]);
    assess(&evented, "2026-08-20").unwrap();
}

#[test]
fn an_unknown_evidence_word_is_refused() {
    let line = base(None, Some("2026-08-06"));
    let err = assess(&with(&line, &[("receipt_evidence", "bank")]), "2026-08-20").unwrap_err();
    assert!(
        err.contains("receipt_evidence 'bank' is not fund, clearing_house, sent or blank"),
        "{err}"
    );
    let err = assess(&with(&line, &[("mvr_status", "ok")]), "2026-08-20").unwrap_err();
    assert!(
        err.contains("mvr_status 'ok' is not matched, not_matched, not_sent or blank"),
        "{err}"
    );
}

#[test]
fn the_record_is_noted_a_known_error_is_a_caveat_and_the_line_echoes_it() {
    let line = with(
        &base(Some("2026-08-10"), Some("2026-08-07")),
        &[
            ("receipt_evidence", "Fund"),
            ("mvr_status", "matched"),
            ("fund_error_ref", " none "),
        ],
    );
    let result = assess(&line, "2026-08-20").unwrap();
    assert_eq!(result["verdict"], "ON_TIME");
    let notes = texts(&result, "notes");
    assert!(
        notes.contains("which does not change the verdict"),
        "{notes}"
    );
    assert!(notes.contains("from the fund's own record"), "{notes}");
    assert!(notes.contains("the fund matched the member"), "{notes}");
    assert!(notes.contains("no fund error was received"), "{notes}");
    assert_eq!(texts(&result, "caveats"), "");
    assert_eq!(result["line"]["receipt_evidence"], "fund");
    assert_eq!(result["line"]["fund_error_ref"], "none");

    let line = with(
        &base(Some("2026-08-10"), Some("2026-08-07")),
        &[
            ("mvr_status", "not_matched"),
            ("fund_error_ref", "CTER-0001"),
        ],
    );
    let result = assess(&line, "2026-08-20").unwrap();
    assert_eq!(result["verdict"], "ON_TIME");
    let caveats = texts(&result, "caveats");
    assert!(
        caveats.contains("a fund error (CTER-0001) is recorded"),
        "{caveats}"
    );
    assert!(caveats.contains("could not match the member"), "{caveats}");
    assert!(
        caveats.matches("returned_by_fund or --events").count() == 2,
        "{caveats}"
    );
    assert_eq!(result["line"]["fund_error_ref"], "CTER-0001");

    let plain = assess(&base(Some("2026-08-10"), Some("2026-08-07")), "2026-08-20").unwrap();
    assert!(plain["line"].get("receipt_evidence").is_none());
    assert_eq!(texts(&plain, "notes"), "");
}

/// A failed member verification on an ON_TIME line is a caveat of its own: not
/// refused, not a fund error, and not a return.
#[test]
fn a_failed_member_verification_on_an_on_time_line_is_only_a_caveat() {
    let line = base(Some("2026-08-10"), Some("2026-08-07"));
    let plain = assess(&line, "2026-08-20").unwrap();
    let result = assess(&with(&line, &[("mvr_status", "not_matched")]), "2026-08-20").unwrap();
    assert_eq!(decision(&result), decision(&plain));
    assert_eq!(result["verdict"], "ON_TIME");
    let caveats = texts(&result, "caveats");
    assert!(caveats.contains("could not match the member"), "{caveats}");
    assert!(caveats.contains("is not a fund error"), "{caveats}");
    assert!(
        caveats.contains("does not say an amount came back"),
        "{caveats}"
    );
    assert!(!caveats.contains("a fund error ("), "{caveats}");
    assert!(result["line"].get("returned_by_fund").is_none(), "{result}");
}

const FIRST_YEAR: &str = "first-year approach";

/// Paid on the QE day, no fund error received.
fn first_year_line(qe_day: &str, remitted: &str) -> Value {
    json!({
        "employee_id": "E1",
        "qe_day": qe_day,
        "sg_amount": "120.00",
        "remitted": remitted,
        "fund_error_ref": "none",
    })
}

#[test]
fn the_first_year_note_needs_every_fact_and_leaves_at_risk() {
    let line = first_year_line("2026-08-06", "2026-08-06");
    let result = assess(&line, "2026-08-20").unwrap();
    assert_eq!(result["verdict"], "AT_RISK");
    let notes = texts(&result, "notes");
    assert!(notes.contains(FIRST_YEAR), "{notes}");
    assert!(notes.contains("stays AT_RISK"), "{notes}");
    assert!(notes.contains("PCG 2026/1 paragraph 11"), "{notes}");
    // A record that the money was sent, or a matched member, changes nothing.
    for (field, value) in [("receipt_evidence", "sent"), ("mvr_status", "matched")] {
        let also = assess(&with(&line, &[(field, value)]), "2026-08-20").unwrap();
        assert_eq!(decision(&also), decision(&result), "{field}");
        assert!(texts(&also, "notes").contains(FIRST_YEAR), "{field}");
    }
    // The last first-year QE day, 30 June 2027, is due 9 July 2027.
    let last = assess(&first_year_line("2027-06-30", "2027-06-30"), "2027-07-20").unwrap();
    assert_eq!(last["verdict"], "AT_RISK");
    assert!(texts(&last, "notes").contains(FIRST_YEAR));

    let mut returned = line.clone();
    returned["returned_by_fund"] = json!(true);
    let missing = [
        (
            "after the first year",
            first_year_line("2027-07-01", "2027-07-01"),
            "2027-07-20",
        ),
        (
            "paid after payday",
            first_year_line("2026-08-06", "2026-08-07"),
            "2026-08-20",
        ),
        (
            "paid before payday",
            first_year_line("2026-08-06", "2026-08-05"),
            "2026-08-20",
        ),
        (
            "errors not checked",
            with(&line, &[("fund_error_ref", "")]),
            "2026-08-20",
        ),
        (
            "a fund error",
            with(&line, &[("fund_error_ref", "CTER-0001")]),
            "2026-08-20",
        ),
        (
            "member not matched",
            with(&line, &[("mvr_status", "not_matched")]),
            "2026-08-20",
        ),
        ("a return", returned, "2026-08-20"),
        (
            "a receipt after the as-at date, 8 days late",
            with(
                &line,
                &[("received", "2026-08-25"), ("matched_amount", "120.00")],
            ),
            "2026-08-20",
        ),
    ];
    for (why, line, as_at) in missing {
        let result = assess(&line, as_at).unwrap();
        assert!(
            !texts(&result, "notes").contains(FIRST_YEAR),
            "{why}: {result}"
        );
        // A return leaves the line undecided; every other line stays AT_RISK.
        if why != "a return" {
            assert_eq!(result["verdict"], "AT_RISK", "{why}");
        }
    }
    // Every other condition holding, a verdict other than AT_RISK gets no note.
    let late = assess(&first_year_line("2026-08-06", "2026-08-20"), "2026-09-01").unwrap();
    assert_eq!(late["verdict"], "LATE");
    assert!(!texts(&late, "notes").contains(FIRST_YEAR));
}

/// Run the checker on `csv` in a fresh folder: exit code, report and stderr.
fn check_file(csv: &str) -> (Option<i32>, String, String) {
    static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "payday-evidence-{}-{}",
        std::process::id(),
        RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pay.csv"), csv).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["pay.csv", "--as-at", "2026-08-20", "-o", "report.csv"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let report = std::fs::read_to_string(dir.join("report.csv")).unwrap_or_default();
    std::fs::remove_dir_all(&dir).unwrap();
    (
        out.status.code(),
        report,
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn the_checker_reads_the_columns_and_the_report_carries_them() {
    let header = "employee_id,payment_date,sg_amount,remitted_date,remitted_amount,\
                  fund_received_date,matched_amount";
    let rows = [
        ("E1,2026-08-06,120.00,2026-08-06,120.00,,", "sent,,none"),
        (
            "E2,2026-08-06,120.00,2026-08-06,120.00,2026-08-10,120.00",
            "clearing_house,not_matched,CTER-0002",
        ),
        // Remitted and received after the 17 August deadline: LATE, so both runs
        // exit 2.
        (
            "E3,2026-08-06,120.00,2026-08-18,120.00,2026-08-19,120.00",
            "fund,matched,none",
        ),
    ];
    let with_evidence: String = rows.iter().map(|(r, e)| format!("{r},{e}\n")).collect();
    let without: String = rows.iter().map(|(r, _)| format!("{r}\n")).collect();
    let (code, report, stderr) = check_file(&format!(
        "{header},receipt_evidence,mvr_status,fund_error_ref\n{with_evidence}"
    ));
    let (plain_code, ..) = check_file(&format!("{header}\n{without}"));
    assert_eq!(plain_code, Some(2));
    assert_eq!(code, plain_code, "{stderr}");
    // The report keeps its 18 columns; the evidence goes in notes and caveats.
    assert_eq!(
        report.trim_start_matches('\u{feff}').lines().next(),
        Some(payday_super_checker::report::CSV_HEADER.join(",").as_str())
    );
    assert!(report.contains(FIRST_YEAR), "{report}");
    assert!(
        report.contains("a fund error (CTER-0002) is recorded"),
        "{report}"
    );

    let (code, report, stderr) = check_file(
        "employee_id,payment_date,sg_amount,remitted_date,receipt_evidence\n\
         E1,2026-08-06,120.00,2026-08-06,fund\n",
    );
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("row 2: receipt_evidence=fund"), "{stderr}");
    assert_eq!(report, "");
}

/// Build an evidence pack from `csv` in a fresh folder: exit code and the 4
/// files, read back as bytes.
fn pack(csv: &str) -> (Option<i32>, Vec<Vec<u8>>) {
    static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "payday-evidence-pack-{}-{}",
        std::process::id(),
        RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pay.csv"), csv).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["evidence-pack", "pay.csv", "--as-at", "2026-08-20"])
        .args(["--assessment-date", "2026-08-20", "-o", "pack"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let files = payday_super_checker::evidence_pack::FILES
        .iter()
        .map(|f| std::fs::read(dir.join("pack").join(f)).unwrap())
        .collect();
    std::fs::remove_dir_all(&dir).unwrap();
    (out.status.code(), files)
}

/// The pack keeps its report header and its bytes when no evidence is given,
/// and carries evidence through the report it is rebuilt from.
#[test]
fn the_evidence_pack_is_unchanged_without_evidence_and_carries_it() {
    let header = "employee_id,payment_date,sg_amount,remitted_date,remitted_amount,\
                  fund_received_date,matched_amount";
    let rows = [
        "E1,2026-08-06,120.00,2026-08-06,120.00,,",
        "E2,2026-08-06,120.00,2026-08-06,120.00,2026-08-10,120.00",
    ];
    let plain = pack(&format!("{header}\n{}\n", rows.join("\n")));
    let blank = pack(&format!(
        "{header},receipt_evidence,mvr_status,fund_error_ref\n{}\n",
        rows.map(|r| format!("{r},,,")).join("\n")
    ));
    assert_eq!(plain, blank);
    let report = String::from_utf8(plain.1[0].clone()).unwrap();
    let first = report
        .trim_start_matches('\u{feff}')
        .lines()
        .next()
        .unwrap();
    // The pack's report is the 18-column report without employee_id.
    let pack_header: Vec<&str> = payday_super_checker::report::CSV_HEADER
        .into_iter()
        .filter(|c| *c != "employee_id")
        .collect();
    assert_eq!(first, pack_header.join(","));

    let evidenced = pack(&format!(
        "{header},receipt_evidence,mvr_status,fund_error_ref\n{},sent,,none\n{},fund,matched,CTER-0003\n",
        rows[0], rows[1]
    ));
    assert_eq!(evidenced.0, plain.0);
    let report = String::from_utf8(evidenced.1[0].clone()).unwrap();
    assert!(
        report.trim_start_matches('\u{feff}').starts_with(first),
        "{report}"
    );
    assert!(
        report.contains("evidence recorded with this row"),
        "{report}"
    );
    assert!(report.contains(FIRST_YEAR), "{report}");
    assert!(
        report.contains("a fund error (CTER-0003) is recorded"),
        "{report}"
    );
    let exceptions = String::from_utf8(evidenced.1[2].clone()).unwrap();
    assert!(exceptions.contains(FIRST_YEAR), "{exceptions}");
}
