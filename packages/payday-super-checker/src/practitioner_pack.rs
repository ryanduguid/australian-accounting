//! Strict, privacy-conscious practitioner handoff for a checker report.
//!
//! The source CSV remains the row-level workpaper. This module produces a
//! deterministic Markdown index and checklist which refers to source row
//! numbers, never employee identifiers, and leaves every professional
//! judgement and consequential action to an appropriately authorised human.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::csvread::{self, Reader};
use crate::decimal::Decimal;
use crate::py::{Date, PyPath};
use crate::pytext;
use crate::rates::hex;
use crate::{Failure, Result};

/// Deliberately duplicated from `report::CSV_HEADER`: a producer change must
/// fail `the_contract_is_pinned_to_the_report_writer` until this consumer is
/// consciously reviewed and updated.
pub const EXPECTED_REPORT_HEADER: [&str; 18] = [
    "row",
    "employee_id",
    "qe_day",
    "pathway",
    "due_date",
    "verdict",
    "days_late",
    "lateness_measured_to",
    "sg_amount",
    "final_shortfall",
    "notional_earnings",
    "uplift_best_case",
    "uplift_worst_case",
    "sgc_estimate_low",
    "sgc_estimate_high",
    "caveats",
    "notes",
    "unassessable_between",
];
pub const VERDICTS: [&str; 6] = ["ON_TIME", "AT_RISK", "LATE", "UNPAID", "UNKNOWN", "SKIPPED"];
pub const EXPOSURE_FIELDS: [&str; 6] = [
    "final_shortfall",
    "notional_earnings",
    "uplift_best_case",
    "uplift_worst_case",
    "sgc_estimate_low",
    "sgc_estimate_high",
];

fn pack_error(message: String) -> Failure {
    Failure::Value(message)
}

/// A positive whole number as its canonical decimal digits, ordered by value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowNumber(pub String);

impl Ord for RowNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.cmp(&other.0))
    }
}

impl PartialOrd for RowNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
pub struct ReportRow {
    pub source_row: RowNumber,
    pub qe_day: Date,
    pub pathway: String,
    pub due_date: Option<Date>,
    pub verdict: String,
    pub days_late: Option<String>,
    pub lateness_measured_to: String,
    pub sg_amount: Decimal,
    /// The six exposure amounts, in `EXPOSURE_FIELDS` order.
    pub exposure: Vec<Option<Decimal>>,
    pub caveats: String,
    pub notes: String,
    pub unassessable_between: String,
}

impl ReportRow {
    pub fn amount(&self, field: &str) -> Option<&Decimal> {
        let i = EXPOSURE_FIELDS
            .iter()
            .position(|f| *f == field)
            .expect("an exposure field");
        self.exposure[i].as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct ReportSnapshot {
    /// `str(source_path)`.
    pub source_path: String,
    pub source_sha256: String,
    pub rows: Vec<ReportRow>,
    pub provenance: String,
    pub identifiers_omitted: bool,
}

impl ReportSnapshot {
    pub fn needs_attention(&self) -> bool {
        self.rows.iter().any(|r| r.verdict != "ON_TIME")
    }
}

fn iso_date(value: &str, field: &str, source_row: &str, optional: bool) -> Result<Option<Date>> {
    if value.is_empty() {
        if optional {
            return Ok(None);
        }
        return Err(pack_error(format!(
            "source row {source_row} has a blank {field}"
        )));
    }
    match Date::fromisoformat(value) {
        Some(d) if d.isoformat() == value => Ok(Some(d)),
        _ => Err(pack_error(format!(
            "source row {source_row} {field} must be an ISO date, got {}",
            pytext::repr(value)
        ))),
    }
}

fn amount(value: &str, field: &str, source_row: &str, optional: bool) -> Result<Option<Decimal>> {
    if value.is_empty() {
        if optional {
            return Ok(None);
        }
        return Err(pack_error(format!(
            "source row {source_row} has a blank {field}"
        )));
    }
    let Ok(amount) = Decimal::parse(value) else {
        return Err(pack_error(format!(
            "source row {source_row} {field} must be a decimal amount, got {}",
            pytext::repr(value)
        )));
    };
    if !amount.is_finite() {
        return Err(pack_error(format!(
            "source row {source_row} {field} must be a finite amount"
        )));
    }
    if amount.lt(&Decimal::from_int(0))? {
        return Err(pack_error(format!(
            "source row {source_row} {field} must not be negative"
        )));
    }
    if value != amount.fixed(2, false) {
        return Err(pack_error(format!(
            "source row {source_row} {field} must use two decimal places"
        )));
    }
    Ok(Some(amount))
}

/// `int(value)` that must also print back as `value`: canonical ASCII digits,
/// within Python's 4,300-digit conversion limit.
fn canonical_int(value: &str) -> bool {
    let canonical = value == "0"
        || (value
            .bytes()
            .next()
            .is_some_and(|b| (b'1'..=b'9').contains(&b))
            && value.bytes().all(|b| b.is_ascii_digit()));
    canonical && value.len() <= 4300
}

fn parse_data_row(values: &[String], seen: &mut BTreeSet<String>) -> Result<ReportRow> {
    let get = |field: &str| -> &str {
        let i = EXPECTED_REPORT_HEADER
            .iter()
            .position(|h| *h == field)
            .expect("a column");
        &values[i]
    };
    let row_text = get("row");
    if !canonical_int(row_text) || row_text == "0" {
        return Err(pack_error(format!(
            "source row {row_text} row must be a positive integer"
        )));
    }
    let source_row = row_text.to_string();
    if !seen.insert(source_row.clone()) {
        return Err(pack_error(format!("duplicate source row {source_row}")));
    }
    if get("employee_id").is_empty() {
        return Err(pack_error(format!(
            "source row {source_row} has a blank employee_id"
        )));
    }
    if get("pathway").is_empty() {
        return Err(pack_error(format!(
            "source row {source_row} has a blank pathway"
        )));
    }
    let verdict = get("verdict");
    if !VERDICTS.contains(&verdict) {
        return Err(pack_error(format!(
            "source row {source_row} has unsupported verdict {}",
            pytext::repr(verdict)
        )));
    }
    let qe_day = iso_date(get("qe_day"), "qe_day", &source_row, false)?.expect("required");
    let sg_amount = amount(get("sg_amount"), "sg_amount", &source_row, false)?.expect("required");
    let mut exposure = Vec::new();
    for field in EXPOSURE_FIELDS {
        exposure.push(amount(get(field), field, &source_row, true)?);
    }
    let exposed = verdict == "LATE" || verdict == "UNPAID";
    if exposed {
        let Some(shortfall) = &exposure[0] else {
            return Err(pack_error(format!(
                "source row {source_row} exposed verdict has no final shortfall"
            )));
        };
        let charge = &exposure[1..];
        if charge.iter().any(Option::is_none) {
            if charge.iter().any(Option::is_some) {
                return Err(pack_error(format!(
                    "source row {source_row} exposed verdict has incomplete exposure amounts"
                )));
            }
        } else {
            let c: Vec<&Decimal> = charge
                .iter()
                .map(|v| v.as_ref().expect("all present"))
                .collect();
            let (nec, up_low, up_high, low, high) = (c[0], c[1], c[2], c[3], c[4]);
            if !low.equals(&shortfall.add(nec)?.add(up_low)?)? {
                return Err(pack_error(format!(
                    "source row {source_row} low SG-charge estimate does not add up"
                )));
            }
            if !high.equals(&shortfall.add(nec)?.add(up_high)?)? {
                return Err(pack_error(format!(
                    "source row {source_row} high SG-charge estimate does not add up"
                )));
            }
            if up_low.gt(up_high)? || low.gt(high)? {
                return Err(pack_error(format!(
                    "source row {source_row} exposure range is reversed"
                )));
            }
        }
    } else if exposure.iter().any(Option::is_some) {
        return Err(pack_error(format!(
            "source row {source_row} non-exposed verdict carries exposure amounts"
        )));
    }
    let candidates = get("unassessable_between");
    if !candidates.is_empty() && verdict != "UNKNOWN" {
        return Err(pack_error(format!(
            "source row {source_row} carries candidate verdicts but is not UNKNOWN"
        )));
    }
    if !candidates.is_empty() {
        let parts: Vec<&str> = candidates.split(" or ").collect();
        let allowed = |p: &str| VERDICTS.contains(&p) || p == "NOT_YET_DUE";
        if parts.len() != 2 || !parts.iter().all(|p| allowed(p)) {
            return Err(pack_error(format!(
                "source row {source_row} has malformed candidate verdicts"
            )));
        }
    }
    let days_text = get("days_late");
    let days_late = if days_text.is_empty() {
        None
    } else if canonical_int(days_text) {
        Some(days_text.to_string())
    } else {
        return Err(pack_error(format!(
            "source row {source_row} days_late must be a non-negative integer"
        )));
    };
    if exposed && days_late.is_none() {
        return Err(pack_error(format!(
            "source row {source_row} exposed verdict has no days_late"
        )));
    }
    let due_date = iso_date(get("due_date"), "due_date", &source_row, true)?;
    Ok(ReportRow {
        source_row: RowNumber(source_row),
        qe_day,
        pathway: get("pathway").to_string(),
        due_date,
        verdict: verdict.to_string(),
        days_late,
        lateness_measured_to: get("lateness_measured_to").to_string(),
        sg_amount,
        exposure,
        caveats: get("caveats").to_string(),
        notes: get("notes").to_string(),
        unassessable_between: candidates.to_string(),
    })
}

/// `os.path.splitext(name)[1]`.
fn extension(name: &str) -> &str {
    match name.rfind('.') {
        Some(dot) if name[..dot].bytes().any(|b| b != b'.') => &name[dot..],
        _ => "",
    }
}

/// Read and validate one byte snapshot of a checker report named by a bare
/// filename in the working directory.
pub fn load_report_snapshot(raw: &str) -> Result<ReportSnapshot> {
    if raw.contains('\0') {
        return Err(pack_error("path contains a NUL byte".into()));
    }
    let name = PyPath::new(raw).name().to_string();
    if name != raw {
        return Err(pack_error(format!(
            "report path must be a .csv filename in the working directory, with no \
             directory part: cd to the directory holding {name}, or copy it here, \
             and pass the bare filename"
        )));
    }
    if !extension(&name).eq_ignore_ascii_case(".csv") {
        return Err(pack_error(format!("{raw} must be a .csv checker report")));
    }
    // `os.getcwd()` raised its OSError, naming no file.
    let root = std::env::current_dir()
        .map(|p| crate::cli::decode_os(p.as_os_str()))
        .map_err(|e| {
            let code = e.raw_os_error().unwrap_or(0);
            let (errno, strerror) =
                crate::py::errno_and_strerror(&e, code, crate::py::OsStyle::Errno);
            Failure::Os(crate::OsError {
                errno,
                text: format!("[Errno {errno}] {strerror}"),
                strerror,
                filename: None,
            })
        })?;
    let candidate = join(&root, &name);
    let data = std::fs::read(crate::py::fs_path(&candidate))
        .map_err(|e| crate::csv_io::os_failure(&e, &candidate))?;
    parse_report_snapshot(&data, &candidate, false)
}

/// `os.path.join(root, name)` for a bare name.
fn join(root: &str, name: &str) -> String {
    let sep = if cfg!(windows) { '\\' } else { '/' };
    if root.ends_with('/') || cfg!(windows) && root.ends_with('\\') {
        format!("{root}{name}")
    } else {
        format!("{root}{sep}{name}")
    }
}

/// Validate report bytes; the projected format is an explicit opt-in.
pub fn parse_report_snapshot(
    data: &[u8],
    source: &str,
    identifiers_omitted: bool,
) -> Result<ReportSnapshot> {
    // Python took a Path, so the source reads as pathlib prints it ("" is ".").
    let source = PyPath::new(source).display();
    let source = source.as_str();
    let digest = hex(&Sha256::digest(data));
    let body_bytes = data.strip_prefix(b"\xef\xbb\xbf").unwrap_or(data);
    if std::str::from_utf8(body_bytes).is_err() {
        return Err(pack_error(format!("{source} is not UTF-8 CSV")));
    }
    let mut table: Vec<Vec<String>> = Vec::new();
    let mut reader = Reader::strict(data);
    loop {
        match reader.next_row() {
            Ok(Some(row)) => table.push(row),
            Ok(None) => break,
            Err(csvread::ReadError::Csv(m)) => {
                return Err(pack_error(format!("{source} is malformed CSV: {m}")));
            }
            Err(csvread::ReadError::Decode { .. }) => {
                return Err(pack_error(format!("{source} is not UTF-8 CSV")));
            }
        }
    }
    if identifiers_omitted {
        let projected: Vec<&str> = EXPECTED_REPORT_HEADER
            .iter()
            .copied()
            .filter(|c| *c != "employee_id")
            .collect();
        if table.is_empty() || table[0] != projected {
            return Err(pack_error(
                "expected the 17-column report without identifiers".into(),
            ));
        }
        for values in table.iter_mut().skip(1) {
            if values.len() != projected.len() {
                return Err(pack_error(
                    "projected report row must have 17 fields".into(),
                ));
            }
            let marker = if values[0] == "NOTE" {
                values[0] = String::new();
                "NOTE"
            } else {
                "omitted"
            };
            values.insert(1, marker.into());
        }
        table[0] = EXPECTED_REPORT_HEADER
            .iter()
            .map(|s| s.to_string())
            .collect();
    }
    if table.is_empty() || table[0] != EXPECTED_REPORT_HEADER {
        return Err(pack_error(format!(
            "{source} must use the exact 18-column payday-super-checker report header"
        )));
    }
    let body = &table[1..];
    for (i, values) in body.iter().enumerate() {
        if values.len() != EXPECTED_REPORT_HEADER.len() {
            return Err(pack_error(format!(
                "CSV row {} has {} fields; expected 18",
                i + 2,
                values.len()
            )));
        }
    }
    let field = |values: &[String], name: &str| -> usize {
        let _ = values;
        EXPECTED_REPORT_HEADER
            .iter()
            .position(|h| *h == name)
            .expect("a column")
    };
    let is_terminal_note = |values: &Vec<String>| {
        values[field(values, "row")].is_empty()
            && values[field(values, "employee_id")] == "NOTE"
            && EXPECTED_REPORT_HEADER
                .iter()
                .zip(values)
                .all(|(h, v)| *h == "employee_id" || *h == "notes" || v.is_empty())
    };
    let note_rows: Vec<usize> = body
        .iter()
        .enumerate()
        .filter(|(_, v)| is_terminal_note(v))
        .map(|(i, _)| i)
        .collect();
    if note_rows.is_empty() {
        return Err(pack_error(format!(
            "{source} must end with one terminal NOTE row"
        )));
    }
    if note_rows.len() != 1 {
        return Err(pack_error(format!(
            "{source} must contain exactly one NOTE row"
        )));
    }
    if note_rows[0] != body.len() - 1 {
        return Err(pack_error(format!(
            "{source} NOTE row must be the terminal NOTE row"
        )));
    }
    let note = &body[body.len() - 1];
    let notes = &note[field(note, "notes")];
    let unexpected = EXPECTED_REPORT_HEADER
        .iter()
        .zip(note)
        .any(|(h, v)| *h != "employee_id" && *h != "notes" && !v.is_empty());
    if unexpected || pytext::strip(notes).is_empty() {
        return Err(pack_error(format!(
            "{source} has a malformed terminal NOTE row"
        )));
    }
    let data_rows = &body[..body.len() - 1];
    if data_rows.is_empty() {
        return Err(pack_error(format!(
            "{source} must contain at least one contribution row"
        )));
    }
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for values in data_rows {
        rows.push(parse_data_row(values, &mut seen)?);
    }
    Ok(ReportSnapshot {
        source_path: source.to_string(),
        source_sha256: digest,
        rows,
        provenance: notes.clone(),
        identifiers_omitted,
    })
}

fn markdown(value: &str) -> String {
    let mut text = value.replace(['\r', '\n'], " ");
    for c in ['\\', '`', '[', ']', '|', '<', '>'] {
        text = text.replace(c, &format!("\\{c}"));
    }
    text
}

fn dollars(value: &Decimal) -> String {
    format!("${}", value.fixed(2, true))
}

fn review_task(row: &ReportRow) -> String {
    match row.verdict.as_str() {
        "UNPAID" => "Confirm the payday, SG amount, receipt and remittance evidence, and any \
                     assessment facts; an authorised practitioner decides any remediation or lodgement."
            .into(),
        "LATE" => "Verify the receipt date, allocation and assessment facts, then have an \
                   authorised practitioner decide any correction, advice or lodgement."
            .into(),
        "AT_RISK" => "Obtain and verify the fund receipt date; remittance alone is not proof of \
                      on-time receipt."
            .into(),
        "UNKNOWN" if !row.unassessable_between.is_empty() => format!(
            "Resolve the missing calendar or allocation facts before deciding between {}.",
            row.unassessable_between
        ),
        "UNKNOWN" => "Confirm whether this is genuinely nil or not yet due; do not infer a pass \
                      from UNKNOWN."
            .into(),
        _ => "Assess the defined-benefit interest outside this tool.".into(),
    }
}

fn priority(verdict: &str) -> u8 {
    match verdict {
        "UNPAID" => 0,
        "LATE" => 1,
        "UNKNOWN" => 2,
        "AT_RISK" => 3,
        _ => 4,
    }
}

/// Render a deterministic review index without employee identifiers.
pub fn render_practitioner_pack(snapshot: &ReportSnapshot, decision_log: bool) -> Result<String> {
    let count = |v: &str| snapshot.rows.iter().filter(|r| r.verdict == v).count();
    let exposed: Vec<&ReportRow> = snapshot
        .rows
        .iter()
        .filter(|r| r.verdict == "LATE" || r.verdict == "UNPAID")
        .collect();
    let mut total_low = Decimal::parse("0.00").expect("a literal");
    let mut total_high = total_low.clone();
    for r in &exposed {
        if let Some(v) = r.amount("sgc_estimate_low") {
            total_low = total_low.add(v)?;
        }
    }
    for r in &exposed {
        if let Some(v) = r.amount("sgc_estimate_high") {
            total_high = total_high.add(v)?;
        }
    }
    let status = if snapshot.needs_attention() {
        "ATTENTION REQUIRED"
    } else {
        "NO EXCEPTION INDICATORS"
    };
    let name = PyPath::new(&snapshot.source_path).name().to_string();
    let mut lines: Vec<String> = vec![
        "# Payday Super Practitioner Review Pack".into(),
        String::new(),
        format!("**Status: {status}.**"),
        String::new(),
        "Private workpaper. This pack indexes the checker report; it does not lodge, \
         pay, post or make a compliance determination. An appropriately authorised \
         human remains responsible for every professional judgement and consequential action."
            .into(),
        String::new(),
        "## Source and provenance".into(),
        String::new(),
        format!("- Source report: {}", markdown(&name)),
        format!("- Source SHA-256: `{}`", snapshot.source_sha256),
        format!("- Contribution rows: {}", snapshot.rows.len()),
        format!("- Producer NOTE: {}", markdown(&snapshot.provenance)),
        String::new(),
        "## Verdict summary".into(),
        String::new(),
        "| Verdict | Rows |".into(),
        "| --- | ---: |".into(),
    ];
    lines.extend(VERDICTS.iter().map(|v| format!("| {v} | {} |", count(v))));
    lines.extend([
        String::new(),
        format!(
            "The displayed experimental SG-charge range across LATE and UNPAID rows is \
             **{} to {}**. Reconcile it to the source \
             report; it is not an ATO assessment.",
            dollars(&total_low),
            dollars(&total_high)
        ),
        String::new(),
        "## Review queue".into(),
        String::new(),
    ]);
    let mut attention: Vec<&ReportRow> = snapshot
        .rows
        .iter()
        .filter(|r| r.verdict != "ON_TIME")
        .collect();
    attention.sort_by(|a, b| {
        priority(&a.verdict)
            .cmp(&priority(&b.verdict))
            .then_with(|| a.source_row.cmp(&b.source_row))
    });
    if attention.is_empty() {
        lines.extend([
            "No non-ON_TIME rows were produced. Practitioner sign-off is still required; \
             verify the source evidence and the checker assumptions before relying on the report."
                .to_string(),
            String::new(),
        ]);
    } else {
        lines.extend([
            if snapshot.identifiers_omitted {
                "Identifiers are omitted from this pack. Use the original `row` value \
                 to reconcile against the private contribution input."
            } else {
                "The identifiers stay in the private source CSV. Use the original `row` value below \
                 to locate each employee record."
            }
            .to_string(),
            String::new(),
            "| Done | Source reference | QE day | Due | Verdict | Displayed range | Human review task |"
                .into(),
            "| --- | --- | --- | --- | --- | ---: | --- |".into(),
        ]);
        for row in attention {
            let due = row
                .due_date
                .map_or_else(|| "not established".to_string(), |d| d.isoformat());
            let mut verdict = row.verdict.clone();
            if !row.unassessable_between.is_empty() {
                verdict.push_str(&format!(" ({})", row.unassessable_between));
            }
            let range = match (
                row.amount("sgc_estimate_low"),
                row.amount("sgc_estimate_high"),
            ) {
                (Some(low), Some(high)) => format!("{} to {}", dollars(low), dollars(high)),
                _ => "not displayed".into(),
            };
            let mut task = review_task(row);
            if !row.caveats.is_empty() {
                task.push_str(&format!(" Checker caveat: {}", row.caveats));
            }
            lines.push(format!(
                "| [ ] | source row {} | {} | {} | {} | {} | {} |",
                row.source_row.0,
                row.qe_day.isoformat(),
                markdown(&due),
                markdown(&verdict),
                markdown(&range),
                markdown(&task)
            ));
        }
        lines.push(String::new());
    }
    lines.extend([
        if decision_log {
            "## Review checklist"
        } else {
            "## Practitioner sign-off"
        }
        .to_string(),
        String::new(),
        "- [ ] The source report SHA-256 above matches the file reviewed.".into(),
        "- [ ] The payroll, clearing-house and fund evidence has been reconciled for every queued row."
            .into(),
        "- [ ] Missing calendar, allocation, assessment and classification facts have been resolved or escalated."
            .into(),
        "- [ ] Any advice, correction, payment, lodgement or disclosure was decided and performed by an appropriately authorised human."
            .into(),
        String::new(),
    ]);
    if decision_log {
        lines.extend([
            "Record all decisions and practitioner sign-off in [decision-log.md](decision-log.md)."
                .to_string(),
            String::new(),
        ]);
    } else {
        lines.extend([
            "Reviewer: ______________________________".to_string(),
            String::new(),
            "Review date (Australia): ______________________________".into(),
            String::new(),
            "Conclusion and workpaper reference: ______________________________".into(),
            String::new(),
        ]);
    }
    Ok(lines.join("\n"))
}

/// Atomically write a Markdown pack at the caller-selected destination.
pub fn write_practitioner_pack(snapshot: &ReportSnapshot, path: &str) -> Result<()> {
    let text = render_practitioner_pack(snapshot, false)?;
    crate::atomic::write(path, text.as_bytes(), crate::atomic::markdown_destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_contract_is_pinned_to_the_report_writer() {
        assert_eq!(EXPECTED_REPORT_HEADER, crate::report::CSV_HEADER);
    }
}
