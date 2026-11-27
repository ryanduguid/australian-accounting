// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A checker report whose multibyte character straddles the CSV reader's
//! 8,192-byte chunk boundary still reads as a snapshot: the reader keeps the
//! incomplete sequence for the next chunk instead of refusing it.
//!
//! The expectation comes from the Python engine under CPython 3.14 on the
//! Linux test host: the same input gives a report with the character's first
//! byte at offset 8,191, which `parse_report_snapshot` reads as 1 ON_TIME row.

use std::path::Path;
use std::process::Command;

use payday_super_checker::practitioner_pack::parse_report_snapshot;

const HEADER: &str = "employee_id,payment_date,sg_amount,remitted_date,remitted_amount,\
                      fund_received_date,matched_amount\n";

/// The report the checker writes for one on-time line for `employee`.
fn report(employee: &str) -> Vec<u8> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("report-snapshot-chunks");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let line = format!("{employee},2026-08-06,120.00,2026-08-06,120.00,2026-08-10,120.00\n");
    std::fs::write(dir.join("pay.csv"), format!("{HEADER}{line}")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .args(["pay.csv", "--as-at", "2026-08-20", "-o", "report.csv"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let data = std::fs::read(dir.join("report.csv")).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    data
}

#[test]
fn a_character_split_across_the_chunk_boundary_reads_whole() {
    let short = report("E1");
    let prefix = short.windows(8).position(|w| w == b"E1,2026-").unwrap();
    let employee = format!("E{}\u{e9}", "x".repeat(8191 - prefix - 1));
    let data = report(&employee);
    assert_eq!(&data[8191..8193], "\u{e9}".as_bytes());
    let snapshot = parse_report_snapshot(&data, "report.csv", false).unwrap();
    assert_eq!(snapshot.rows.len(), 1);
    assert_eq!(snapshot.rows[0].verdict, "ON_TIME");
}
