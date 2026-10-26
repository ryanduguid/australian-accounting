//! Vendor imports checked end to end, from fabricated export files through
//! the written contributions CSV, outcome counts and warnings.

use payday_super_checker::importers::import_files;
use std::path::Path;

/// Beam's Reconciled means it has cleared the employer's money, before
/// Sent to fund, so only a Sent to fund row may carry a remittance date
/// and a Reconciled payday is written unfunded with a warning.
#[test]
fn a_reconciled_beam_batch_is_not_yet_sent_to_the_fund() {
    let dir = std::env::temp_dir().join(format!("payday-beam-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (payroll, super_csv, out) = (
        dir.join("payroll.csv"),
        dir.join("super.csv"),
        dir.join("out.csv"),
    );
    std::fs::write(
        &payroll,
        "Employee,Date Paid,Pay Period Ending,Super Guarantee\n\
         Test Employee One,09/07/2026,09/07/2026,612.00\n\
         Test Employee Two,09/07/2026,09/07/2026,540.00\n",
    )
    .unwrap();
    std::fs::write(
        &super_csv,
        "Employee,Super Contribution Type,Period Start,Period End,Payment Date,Amount,Status\n\
         Test Employee One,Super Guarantee,01/07/2026,09/07/2026,14/07/2026,612.00,Reconciled\n\
         Test Employee Two,Super Guarantee,01/07/2026,09/07/2026,14/07/2026,540.00,Sent to fund\n",
    )
    .unwrap();
    let path = |p: &Path| p.to_str().unwrap().to_string();
    let report = import_files(
        &path(&payroll),
        &path(&super_csv),
        &path(&out),
        Some("employment-hero"),
        false,
    )
    .unwrap();
    let written = std::fs::read_to_string(&out).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    // Employee, remitted_date, remitted_amount and matched_amount.
    let remittance: Vec<[&str; 4]> = written
        .lines()
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            [c[0], c[3], c[9], c[10]]
        })
        .collect();
    assert_eq!(
        remittance,
        [
            ["Test Employee One", "", "", "612.00"],
            ["Test Employee Two", "2026-07-14", "540.00", "540.00"]
        ]
    );
    assert_eq!(report.count("matched, no remittance date"), 1);
    assert_eq!(report.count("matched"), 1);
    let warning = "super row 2: status 'Reconciled' does not establish remittance to the \
                   fund, so its payment date is not evidence of remittance and was not used";
    assert!(report.warnings.iter().any(|w| w == warning));
}
