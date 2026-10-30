//! Vendor imports checked end to end, from fabricated export files through
//! the written contributions CSV, outcome counts and warnings.

use payday_super_checker::importers::{ImportReport, import_files};

const PAYROLL_HEADER: &str = "Employee,Date Paid,Pay Period Ending,Super Guarantee\n";
const BEAM_HEADER: &str =
    "Employee,Super Contribution Type,Period Start,Period End,Payment Date,Amount,Status\n";

/// Imports fabricated Employment Hero files and returns the result with the
/// written CSV's employee, remitted_date, remitted_amount and matched_amount.
fn import_beam(
    name: &str,
    payroll: &str,
    super_csv: &str,
) -> (payday_super_checker::Result<ImportReport>, Vec<[String; 4]>) {
    let dir = std::env::temp_dir().join(format!("payday-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = |file: &str| dir.join(file).to_str().unwrap().to_string();
    let (payroll_path, super_path, out) = (path("payroll.csv"), path("super.csv"), path("out.csv"));
    std::fs::write(&payroll_path, payroll).unwrap();
    std::fs::write(&super_path, super_csv).unwrap();
    let report = import_files(
        &payroll_path,
        &super_path,
        &out,
        Some("employment-hero"),
        false,
    );
    let written = match report {
        Ok(_) => std::fs::read_to_string(&out).unwrap(),
        Err(_) => String::new(),
    };
    let _ = std::fs::remove_dir_all(&dir);
    // The fabricated values hold no commas or quotes, so a plain split reads
    // each field; the columns are found by their header names.
    assert!(!written.contains('"'), "{written}");
    // The file opens with a UTF-8 BOM for Excel.
    let mut lines = written.strip_prefix('\u{feff}').unwrap_or(&written).lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split(',').collect();
    let at = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let remittance = lines
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            [
                "employee_id",
                "remitted_date",
                "remitted_amount",
                "matched_amount",
            ]
            .map(|name| c[at(name)].to_string())
        })
        .collect();
    (report, remittance)
}

fn not_remitted_warning(row: usize, status: &str) -> String {
    format!(
        "super row {row}: status '{status}' does not establish remittance to the fund, \
         so its payment date is not evidence of remittance and was not used"
    )
}

/// Beam's Reconciled means it has cleared the employer's money, before
/// Sent to fund, so only a Sent to fund row may carry a remittance date
/// and a Reconciled payday is written unfunded with a warning.
#[test]
fn a_reconciled_beam_batch_is_not_yet_sent_to_the_fund() {
    let (report, remittance) = import_beam(
        "beam-reconciled",
        &format!(
            "{PAYROLL_HEADER}\
             Test Employee One,09/07/2026,09/07/2026,612.00\n\
             Test Employee Two,09/07/2026,09/07/2026,540.00\n"
        ),
        &format!(
            "{BEAM_HEADER}\
             Test Employee One,Super Guarantee,01/07/2026,09/07/2026,14/07/2026,612.00,Reconciled\n\
             Test Employee Two,Super Guarantee,01/07/2026,09/07/2026,14/07/2026,540.00,Sent to fund\n"
        ),
    );
    let report = report.unwrap();
    assert_eq!(
        remittance,
        [
            ["Test Employee One", "", "", "612.00"],
            ["Test Employee Two", "2026-07-14", "540.00", "540.00"]
        ]
        .map(|row| row.map(str::to_string))
    );
    assert_eq!(report.count("matched, no remittance date"), 1);
    assert_eq!(report.count("matched"), 1);
    assert!(
        report
            .warnings
            .contains(&not_remitted_warning(2, "Reconciled"))
    );
}

/// Every status the vendor lists, one fabricated row per status: the 11 for
/// automated batches, plus 'Not actioned', 'Paid' and 'Payment failed'
/// ('Created' is also the first manual status). Only a sent status keeps its
/// payment date as a remittance, and that date is never a fund receipt.
#[test]
fn every_vendor_beam_status_is_classified_and_only_sent_ones_are_a_remittance() {
    let not_sent = [
        "Created",
        "Submission accepted",
        "Awaiting payment",
        "Awaiting clearance",
        "Reconciled",
        "Sent to fund with refund",
        "Cancelled",
        "Failed",
        "Refunded",
        "Not actioned",
        "Payment failed",
    ];
    let sent = ["Sent to fund", "Sent to fund with response", "Paid"];
    let statuses: Vec<&str> = not_sent.iter().chain(&sent).copied().collect();
    // A status the profile gains must be added to these lists and tested.
    let profiles = payday_super_checker::profiles::load_profiles(Some("super")).unwrap();
    let rule = profiles
        .iter()
        .find(|p| p.key == "employment-hero-super")
        .and_then(|p| p.remitted_status.as_ref())
        .unwrap();
    let sorted = |list: Vec<String>| {
        let mut list = list;
        list.sort_unstable();
        list
    };
    let expected = |list: &[&str]| sorted(list.iter().map(|s| s.to_string()).collect());
    assert_eq!(sorted(rule.sent.clone()), expected(&sent));
    assert_eq!(sorted(rule.not_sent.clone()), expected(&not_sent));
    let mut payroll = PAYROLL_HEADER.to_string();
    let mut super_csv = BEAM_HEADER.to_string();
    for (n, status) in statuses.iter().enumerate() {
        payroll += &format!("Test Employee {n},09/07/2026,09/07/2026,100.00\n");
        super_csv += &format!(
            "Test Employee {n},Super Guarantee,01/07/2026,09/07/2026,14/07/2026,100.00,{status}\n"
        );
    }
    let (report, remittance) = import_beam("beam-ladder", &payroll, &super_csv);
    let report = report.unwrap();
    for (n, status) in statuses.iter().enumerate() {
        let is_sent = sent.contains(status);
        let (date, amount) = if is_sent {
            ("2026-07-14", "100.00")
        } else {
            ("", "")
        };
        assert_eq!(
            remittance[n],
            [&format!("Test Employee {n}"), date, amount, "100.00"].map(str::to_string),
            "{status}"
        );
        assert_eq!(
            report
                .warnings
                .contains(&not_remitted_warning(n + 2, status)),
            !is_sent,
            "{status}"
        );
    }
    assert_eq!(report.count("matched, no remittance date"), not_sent.len());
    assert_eq!(report.count("matched"), sent.len());
}

/// A batch where a fund rejected and refunded some contributions does not say
/// which, so no row in it reads as remitted. A reviewer settles each
/// contribution from the batch history and enters that row's date by hand.
#[test]
fn a_sent_to_fund_with_refund_row_gets_no_remittance_date() {
    let (report, remittance) = import_beam(
        "beam-with-refund",
        &format!("{PAYROLL_HEADER}Test Employee One,09/07/2026,09/07/2026,612.00
"),
        &format!(
            "{BEAM_HEADER}             Test Employee One,Super Guarantee,01/07/2026,09/07/2026,14/07/2026,612.00,Sent to fund with refund
"
        ),
    );
    let report = report.unwrap();
    assert_eq!(
        remittance,
        [["Test Employee One", "", "", "612.00"].map(str::to_string)]
    );
    assert!(
        report
            .warnings
            .contains(&not_remitted_warning(2, "Sent to fund with refund"))
    );
}

/// A status off the ladder stops the import rather than guessing, including
/// one that only starts like a known status.
#[test]
fn a_near_miss_beam_status_stops_the_import() {
    let (report, _) = import_beam(
        "beam-near-miss",
        &format!("{PAYROLL_HEADER}Test Employee One,09/07/2026,09/07/2026,100.00
"),
        &format!(
            "{BEAM_HEADER}             Test Employee One,Super Guarantee,01/07/2026,09/07/2026,14/07/2026,100.00,Sent to fund with reversal
"
        ),
    );
    let message = report.err().unwrap().message();
    assert!(
        message.contains("row 2: status 'Sent to fund with reversal' is not one this tool knows"),
        "{message}"
    );
    // A withheld date is missing evidence, not proof that no money moved.
    assert!(
        message.contains("does not establish remittance: ['Awaiting clearance'"),
        "{message}"
    );
    assert!(!message.contains("not yet sent"), "{message}");
}

/// A status off the ladder stops the import rather than guessing.
#[test]
fn an_unknown_beam_status_stops_the_import() {
    let (report, _) = import_beam(
        "beam-unknown",
        &format!("{PAYROLL_HEADER}Test Employee One,09/07/2026,09/07/2026,100.00\n"),
        &format!(
            "{BEAM_HEADER}\
             Test Employee One,Super Guarantee,01/07/2026,09/07/2026,14/07/2026,100.00,Pending\n"
        ),
    );
    let message = report.err().unwrap().message();
    assert!(
        message.contains("row 2: status 'Pending' is not one this tool knows"),
        "{message}"
    );
}
