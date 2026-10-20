//! Read a payroll export and a super payments export, join them, and write
//! the canonical contributions CSV.
//!
//! No vendor export carries a fund receipt date, so every vendor date lands
//! in `remitted` and the receipt column is left empty. Where a profile
//! classifies a status column, a vendor date lands in `remitted` only when
//! the row's status shows the payment actually left the employer.

use crate::csv_io::{
    self, Cell, DictReader, LATEST_SANE_YEAR, ReadFailure, cents, csv_safe, malformed_row_problem,
    money, not_utf8, parse_amount, parse_date_text, problems_error,
};
use crate::csvwrite;
use crate::dates;
use crate::deadlines::REGIME_START;
use crate::decimal::Decimal;
use crate::join::{self, Columns, JoinResult, MatchOutcome, OrphanReason, PayrollRow, SuperRow};
use crate::profiles::{self, Profile, normalise_header};
use crate::py::Date;
use crate::{Failure, Result, pytext, repr_list, sorted_strings};

fn csv_error(message: String) -> Failure {
    Failure::Value(message)
}

fn check_duplicate_headers(headers: &[String], path: &str) -> Result<()> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for h in headers {
        let normalised = normalise_header(h);
        let key = if normalised.is_empty() {
            crate::py::casefold(&crate::py::split_whitespace(h).join(" "))
        } else {
            normalised
        };
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1.push(h.clone()),
            None => groups.push((key, vec![h.clone()])),
        }
    }
    let mut duplicates: Vec<(String, Vec<String>)> =
        groups.into_iter().filter(|(_, v)| v.len() > 1).collect();
    if duplicates.is_empty() {
        return Ok(());
    }
    duplicates.sort_by(|a, b| pytext::code_points(&a.0).cmp(pytext::code_points(&b.0)));
    let detail: Vec<String> = duplicates
        .iter()
        .map(|(key, headings)| {
            format!(
                "{} all read as {}",
                repr_list(&sorted_strings(headings)),
                pytext::repr(key)
            )
        })
        .collect();
    Err(csv_error(format!(
        "{path} has two or more columns that normalise to the same heading: \
         {}. Only one of them would be read and there is no reliable way \
         to tell which, so no figure from this file can be trusted. Rename the \
         columns so each heading is unique.",
        detail.join("; ")
    )))
}

/// A vendor row: cell values by the file's own headings.
pub type Row = Vec<(String, String)>;

fn read_dicts(path: &str) -> Result<(Vec<String>, Vec<Row>)> {
    let data = csv_io::read_bytes(path)?;
    let read =
        || -> std::result::Result<(Option<Vec<String>>, Vec<csv_io::DictRow>), ReadFailure> {
            let mut reader = DictReader::new(&data)?;
            let fieldnames = reader.fieldnames().map(<[String]>::to_vec);
            let mut rows = Vec::new();
            if fieldnames.is_some() {
                while let Some(row) = reader.next_row()? {
                    rows.push(row);
                }
            }
            Ok((fieldnames, rows))
        };
    let (fieldnames, raw_rows) = match read() {
        Ok(read) => read,
        Err(ReadFailure::Decode { byte, start }) => return Err(not_utf8(path, byte, start)),
        Err(ReadFailure::Csv(m)) => return Err(csv_io::csv_module_error(&m)),
    };
    let Some(fieldnames) = fieldnames else {
        return Err(csv_error(format!("{path} has no header row")));
    };
    let headers: Vec<String> = fieldnames
        .into_iter()
        .filter(|h| !pytext::strip(h).is_empty())
        .collect();
    check_duplicate_headers(&headers, path)?;
    let mut problems = Vec::new();
    let mut rows = Vec::new();
    for (i, raw) in raw_rows.into_iter().enumerate() {
        let i = i as i64 + 2;
        if let Some(problem) = malformed_row_problem(&raw, i) {
            problems.push(problem);
            continue;
        }
        rows.push(
            raw.into_iter()
                .filter_map(|(k, v)| match (k, v) {
                    (Some(k), Cell::Value(v)) => Some((k, v)),
                    (Some(k), _) => Some((k, String::new())),
                    (None, _) => None,
                })
                .collect(),
        );
    }
    if !problems.is_empty() {
        return Err(problems_error(&problems, path));
    }
    if rows.is_empty() {
        return Err(csv_error(format!("{path} has a header but no data rows")));
    }
    Ok((headers, rows))
}

fn date(value: &str, field: &str, row: i64, formats: &[String]) -> Result<Option<Date>> {
    let text = pytext::strip(value);
    if text.is_empty() {
        return Ok(None);
    }
    let mut parsed = formats
        .iter()
        .find_map(|fmt| dates::strptime_date(text, fmt));
    if parsed.is_none() {
        parsed = parse_date_text(text)
            .map_err(|e| csv_error(format!("row {row}: {field} {}", e.message())))?;
    }
    let Some(parsed) = parsed else {
        return Err(csv_error(format!(
            "row {row}: cannot read {field} value {} as a date",
            pytext::repr(value)
        )));
    };
    if parsed.year > LATEST_SANE_YEAR {
        return Err(csv_error(format!(
            "row {row}: {field} value {} is not a real date. Leave placeholder \
             dates such as 9999-12-31 blank instead",
            pytext::repr(value)
        )));
    }
    Ok(Some(parsed))
}

fn amount(value: &str, field: &str, row: i64) -> Result<Decimal> {
    parse_amount(value, field, row, true)
}

fn resolved_get<'a>(resolved: &'a [(String, String)], field: &str) -> Option<&'a String> {
    resolved.iter().find(|(f, _)| f == field).map(|(_, h)| h)
}

fn cell(row: &Row, resolved: &[(String, String)], field: &str) -> String {
    let Some(heading) = resolved_get(resolved, field) else {
        return String::new();
    };
    row.iter()
        .find(|(k, _)| k == heading)
        .map(|(_, v)| pytext::strip(v).to_string())
        .unwrap_or_default()
}

fn optional(text: String) -> Option<String> {
    (!text.is_empty()).then_some(text)
}

pub type Resolved = Vec<(String, String)>;

/// Read a super payments export: its rows, profile and resolved columns.
pub fn read_super(path: &str, vendor: Option<&str>) -> Result<(Vec<SuperRow>, Profile, Resolved)> {
    let (headers, raw_rows) = read_dicts(path)?;
    let profile = profiles::detect(&headers, "super", vendor)?;
    let resolved = profiles::resolve_columns(&profile, &headers);
    if resolved_get(&resolved, "amount").is_none() {
        return Err(csv_error(format!(
            "{path}: no amount column found for profile {}",
            profile.key
        )));
    }
    if let Some(filter) = &profile.sg_filter
        && resolved_get(&resolved, &filter.column).is_none()
    {
        return Err(csv_error(format!(
            "{path} has no contribution type column, so salary sacrifice and \
             additional contributions cannot be told apart from super guarantee. \
             Re-run the report with that column included, or map the file by hand."
        )));
    }
    if let Some(rule) = &profile.remitted_status
        && resolved_get(&resolved, &rule.column).is_none()
    {
        return Err(csv_error(format!(
            "{path} has no payment status column, so a batch whose money never \
             left the employer cannot be told apart from one that was paid, and \
             every payment date in the file would be read as a remittance. \
             Re-run the report with that column included, or map the file by hand."
        )));
    }
    let wanted: Vec<String> = profile
        .sg_filter
        .iter()
        .flat_map(|f| f.include.iter().map(|v| normalise_header(v)))
        .collect();
    let sent: Vec<String> = profile
        .remitted_status
        .iter()
        .flat_map(|r| r.sent.iter().map(|v| normalise_header(v)))
        .collect();
    let not_sent: Vec<String> = profile
        .remitted_status
        .iter()
        .flat_map(|r| r.not_sent.iter().map(|v| normalise_header(v)))
        .collect();
    let mut rows = Vec::new();
    for (i, raw) in raw_rows.iter().enumerate() {
        let i = i as i64 + 2;
        if let Some(filter) = &profile.sg_filter {
            let kind = normalise_header(&cell(raw, &resolved, &filter.column));
            if !wanted.contains(&kind) {
                continue;
            }
        }
        let mut paid_date = date(
            &cell(raw, &resolved, "paid_date"),
            "paid date",
            i,
            &profile.date_formats,
        )?;
        let mut unpaid_status = None;
        if let Some(rule) = &profile.remitted_status {
            let status_text = cell(raw, &resolved, &rule.column);
            let status_key = normalise_header(&status_text);
            if not_sent.contains(&status_key) {
                unpaid_status = Some(crate::py::split_whitespace(&status_text).join(" "));
                paid_date = None;
            } else if !sent.contains(&status_key) {
                return Err(csv_error(format!(
                    "row {i}: status {} is not one this tool knows \
                     for profile {} (payment left the employer: \
                     {}; money not yet sent: \
                     {}), so there is no way to tell \
                     whether this payment was made. Correct the status column, \
                     or map the file by hand.",
                    pytext::repr(&status_text),
                    profile.key,
                    repr_list(&rule.sent),
                    repr_list(&rule.not_sent)
                )));
            }
        }
        let employee_id = optional(cell(raw, &resolved, "employee_id"));
        let employee_name = optional(cell(raw, &resolved, "employee_name"));
        let period_start = date(
            &cell(raw, &resolved, "period_start"),
            "period start",
            i,
            &profile.date_formats,
        )?;
        let period_end = date(
            &cell(raw, &resolved, "period_end"),
            "period end",
            i,
            &profile.date_formats,
        )?;
        let value = amount(&cell(raw, &resolved, "amount"), "amount", i)?;
        rows.push(SuperRow {
            employee_id,
            employee_name,
            period_start,
            period_end,
            paid_date,
            amount: value,
            row: i,
            unpaid_status,
        });
    }
    if rows.is_empty() {
        return Err(csv_error(match &profile.sg_filter {
            Some(filter) => format!(
                "{path} has rows but none of them is super guarantee. Check the \
                 contribution types against {}",
                repr_list(&filter.include)
            ),
            None => format!("{path} has no usable rows"),
        }));
    }
    Ok((rows, profile, resolved))
}

/// Read a payroll export: its rows, profile and resolved columns.
pub fn read_payroll(
    path: &str,
    vendor: Option<&str>,
) -> Result<(Vec<PayrollRow>, Profile, Resolved)> {
    let (headers, raw_rows) = read_dicts(path)?;
    let profile = profiles::detect(&headers, "payroll", vendor)?;
    let resolved = profiles::resolve_columns(&profile, &headers);
    for required in ["payday", "amount"] {
        if resolved_get(&resolved, required).is_none() {
            return Err(csv_error(format!(
                "{path}: no {required} column found for profile {}. Columns found: {}",
                profile.key,
                repr_list(&headers)
            )));
        }
    }
    let mut rows = Vec::new();
    for (i, raw) in raw_rows.iter().enumerate() {
        let i = i as i64 + 2;
        let Some(payday) = date(
            &cell(raw, &resolved, "payday"),
            "payday",
            i,
            &profile.date_formats,
        )?
        else {
            return Err(csv_error(format!("row {i}: payday is empty")));
        };
        let employee_id = optional(cell(raw, &resolved, "employee_id"));
        let employee_name = optional(cell(raw, &resolved, "employee_name"));
        let period_end = date(
            &cell(raw, &resolved, "period_end"),
            "period end",
            i,
            &profile.date_formats,
        )?;
        let sg_amount = amount(&cell(raw, &resolved, "amount"), "sg amount", i)?;
        rows.push(PayrollRow {
            employee_id,
            employee_name,
            payday,
            period_end,
            sg_amount,
            row: i,
        });
    }
    Ok((rows, profile, resolved))
}

pub const CANONICAL_HEADER: [&str; 12] = [
    "employee_id",
    "payment_date",
    "sg_amount",
    "remitted_date",
    "fund_received_date",
    "first_contribution_to_fund",
    "out_of_cycle",
    "next_standard_payday",
    "defined_benefit",
    "remitted_amount",
    "matched_amount",
    "join_caveats",
];

pub const OUTCOME_MATCHED: &str = "matched";
pub const OUTCOME_OWES_NOTHING: &str = "owes nothing";
pub const OUTCOME_UNDATED: &str = "matched, no remittance date";
pub const OUTCOME_PARTIAL: &str = "partial";
pub const OUTCOME_OVER: &str = "over";
pub const OUTCOME_UNMATCHED: &str = "unmatched";

/// Bucket one payroll row's join outcome; the more actionable bucket wins.
pub fn classify_outcome(outcome: &MatchOutcome, row: &PayrollRow) -> Result<&'static str> {
    if row.sg_amount.equals(&crate::csv_io::zero())? {
        return Ok(OUTCOME_OWES_NOTHING);
    }
    if outcome.flag == "no super payment found" {
        return Ok(OUTCOME_UNMATCHED);
    }
    if outcome.flag.starts_with("partial: ") {
        return Ok(OUTCOME_PARTIAL);
    }
    if outcome.flag.starts_with("over: ") {
        return Ok(OUTCOME_OVER);
    }
    if let Some(r) = &outcome.remitted_amount
        && cents(Some(r))?.lt(&cents(Some(&row.sg_amount))?)?
    {
        return Ok(OUTCOME_PARTIAL);
    }
    if outcome.remitted.is_none() {
        return Ok(OUTCOME_UNDATED);
    }
    Ok(OUTCOME_MATCHED)
}

/// Refuse an export whose formula guard would write two employees as one.
fn refuse_merged_identities(result: &JoinResult, labels: &[(String, String)]) -> Result<()> {
    let mut written: crate::Groups<String> = crate::Groups::default();
    for (key, label) in labels {
        written.push(csv_safe(label), key.clone());
    }
    let mut merged: Vec<(String, Vec<String>)> = written
        .entries
        .into_iter()
        .filter(|(_, keys)| keys.len() > 1)
        .collect();
    if merged.is_empty() {
        return Ok(());
    }
    let mut rows: crate::Groups<i64> = crate::Groups::default();
    for outcome in &result.outcomes {
        let row = &result.payroll_rows[outcome.payroll];
        rows.push(join::payroll_key(row, &result.key_mode), row.row);
    }
    merged.sort_by(|a, b| pytext::code_points(&a.0).cmp(pytext::code_points(&b.0)));
    let label_of = |key: &str| {
        labels
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, l)| l.clone())
            .unwrap_or_default()
    };
    let detail: Vec<String> = merged
        .iter()
        .map(|(value, keys)| {
            let people: Vec<String> = sorted_strings(keys)
                .iter()
                .map(|key| {
                    let mut numbers = rows.get(key).cloned().unwrap_or_default();
                    numbers.sort();
                    let numbers: Vec<String> = numbers.iter().map(i64::to_string).collect();
                    format!(
                        "\"{}\" (payroll row(s) {})",
                        label_of(key),
                        numbers.join(", ")
                    )
                })
                .collect();
            format!("\"{value}\" would be written for {}", people.join(" and "))
        })
        .collect();
    Err(csv_error(format!(
        "{} written employee identifier(s) would stand for more than one \
         person: {}. The usual cause is the spreadsheet formula guard, which \
         adds a leading quote to a code starting with =, +, - or @. Two employees \
         under one identifier share a deadline group in the \
         check, so the canonical file is not written. Give them distinct codes that \
         survive the guard unchanged, and import again.",
        merged.len(),
        detail.join("; ")
    )))
}

/// The canonical contributions CSV text, without the BOM the file carries.
/// Each join key's written label, refused where 2 people would share one.
fn labels_of(result: &JoinResult) -> Result<Vec<(String, String)>> {
    let mut labels: Vec<(String, String)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for outcome in &result.outcomes {
        let row = &result.payroll_rows[outcome.payroll];
        // Python's `a or b or c or ""`: an empty string falls through.
        let given = |s: &Option<String>| s.clone().filter(|s| !s.is_empty());
        let preferred = if result.key_mode == "id" {
            given(&row.employee_id)
        } else {
            given(&row.employee_name)
        }
        .or_else(|| given(&row.employee_id))
        .or_else(|| given(&row.employee_name))
        .unwrap_or_default();
        let k = join::payroll_key(row, &result.key_mode);
        if seen.insert(k.clone()) {
            labels.push((k, preferred));
        }
    }
    refuse_merged_identities(result, &labels)?;
    Ok(labels)
}

pub fn canonical_csv(result: &JoinResult) -> Result<String> {
    let labels = labels_of(result)?;
    let label_of: std::collections::HashMap<&str, &str> = labels
        .iter()
        .map(|(k, l)| (k.as_str(), l.as_str()))
        .collect();
    let join_note = result.warnings.join(" | ");
    let mut out = String::new();
    csvwrite::row(
        &CANONICAL_HEADER
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        &mut out,
    );
    for outcome in &result.outcomes {
        let row = &result.payroll_rows[outcome.payroll];
        let k = join::payroll_key(row, &result.key_mode);
        let label = label_of
            .get(k.as_str())
            .copied()
            .unwrap_or_default()
            .to_string();
        let mut credited = outcome.remitted_amount.clone();
        if let Some(c) = &credited
            && cents(Some(c))?.gt(&cents(Some(&row.sg_amount))?)?
        {
            credited = Some(row.sg_amount.clone());
        }
        let mut matched = outcome.matched_amount.clone();
        if let Some(m) = &matched
            && cents(Some(m))?.gt(&cents(Some(&row.sg_amount))?)?
        {
            matched = Some(row.sg_amount.clone());
        }
        let iso = |d: Option<Date>| d.map(|d| d.isoformat()).unwrap_or_default();
        let values = [
            label,
            row.payday.isoformat(),
            money(Some(&row.sg_amount))?,
            iso(outcome.remitted),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            money(credited.as_ref())?,
            money(matched.as_ref())?,
            join_note.clone(),
        ];
        let guarded: Vec<String> = values.iter().map(|v| csv_safe(v)).collect();
        csvwrite::row(&guarded, &mut out);
    }
    Ok(out)
}

/// Write the canonical contributions CSV with a UTF-8 BOM.
pub fn write_canonical(result: &JoinResult, path: &str) -> Result<()> {
    // The identity refusal, then the output-name guard, then the rows, in
    // Python's order; nothing is staged before all three pass.
    labels_of(result)?;
    crate::atomic::csv_destination(path)?;
    let text = canonical_csv(result)?;
    crate::atomic::write(
        path,
        format!("\u{feff}{text}").as_bytes(),
        crate::atomic::csv_destination,
    )
}

fn pre_regime_warnings(payroll_rows: &[PayrollRow]) -> Vec<String> {
    let early: Vec<&PayrollRow> = payroll_rows
        .iter()
        .filter(|r| r.payday < REGIME_START)
        .collect();
    if early.is_empty() {
        return Vec::new();
    }
    let shown: Vec<String> = early.iter().take(20).map(|r| r.row.to_string()).collect();
    let more = if early.len() > 20 {
        format!(" and {} more", early.len() - 20)
    } else {
        String::new()
    };
    let earliest = early.iter().map(|r| r.payday).min().expect("not empty");
    vec![format!(
        "{} payroll row(s) have a payday before \
         {} (row(s) {}{more}; earliest {}), and \
         the check refuses any file containing one: those paydays are governed by the \
         old quarterly SG law, which this tool does not model. They are written to the \
         output anyway, because they are real payroll rows -- delete them from it, or \
         re-export from the start of the financial year in which payday super applies, \
         before running the check.",
        early.len(),
        REGIME_START.isoformat(),
        shown.join(", "),
        earliest.isoformat()
    )]
}

/// What did and did not join, from one import run.
#[derive(Clone, Debug)]
pub struct ImportReport {
    pub payroll_profile: Profile,
    pub super_profile: Profile,
    /// Payroll-row counts by outcome bucket, in first-seen order.
    pub outcome_counts: Vec<(String, usize)>,
    pub orphan_reasons: Vec<OrphanReason>,
    /// The super rows `orphan_reasons` index, as Python's carried each row.
    pub super_rows: Vec<SuperRow>,
    pub key_mode: String,
    pub warnings: Vec<String>,
}

impl ImportReport {
    pub fn count(&self, bucket: &str) -> usize {
        self.outcome_counts
            .iter()
            .find(|(b, _)| b == bucket)
            .map_or(0, |(_, n)| *n)
    }

    /// `orphan_reasons` tallied by code, in first-seen order.
    pub fn orphan_counts(&self) -> Vec<(String, usize)> {
        let mut counts: Vec<(String, usize)> = Vec::new();
        for reason in &self.orphan_reasons {
            match counts.iter_mut().find(|(c, _)| *c == reason.code) {
                Some(entry) => entry.1 += 1,
                None => counts.push((reason.code.clone(), 1)),
            }
        }
        counts
    }

    pub fn clean(&self) -> bool {
        [
            OUTCOME_UNDATED,
            OUTCOME_PARTIAL,
            OUTCOME_OVER,
            OUTCOME_UNMATCHED,
        ]
        .iter()
        .all(|b| self.count(b) == 0)
            && self.orphan_reasons.is_empty()
    }
}

/// Refuse an output path that resolves to either export.
pub fn check_output(payroll_path: &str, super_path: &str, out_path: &str) -> Result<()> {
    let out = crate::py::resolve_text(&crate::py::fs_path(out_path));
    for source in [payroll_path, super_path] {
        if crate::py::same_resolved(&crate::py::resolve_text(&crate::py::fs_path(source)), &out) {
            return Err(csv_error(format!(
                "the output would overwrite {source}. Choose a different path with -o."
            )));
        }
    }
    Ok(())
}

/// Read both exports, join them, write the canonical CSV and summarise.
pub fn import_files(
    payroll_path: &str,
    super_path: &str,
    out_path: &str,
    vendor: Option<&str>,
    statutory_allocation_confirmed: bool,
) -> Result<ImportReport> {
    check_output(payroll_path, super_path, out_path)?;
    let (payroll_rows, payroll_profile, payroll_resolved) = read_payroll(payroll_path, vendor)?;
    let (super_rows, super_profile, super_resolved) = read_super(super_path, vendor)?;

    let key_mode = if join::both_have_ids(&payroll_rows, &super_rows) {
        "id"
    } else {
        "name"
    };
    let mut super_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    for row in &super_rows {
        if row.amount.gt(&csv_io::zero())? {
            super_keys.insert(join::super_key(row, key_mode));
        }
    }
    let mut grouped: crate::Groups<&PayrollRow> = crate::Groups::default();
    for row in &payroll_rows {
        let k = join::payroll_key(row, key_mode);
        if row.sg_amount.gt(&csv_io::zero())?
            && row.payday >= REGIME_START
            && super_keys.contains(&k)
        {
            grouped.push(k, row);
        }
    }
    let allocation_groups: Vec<&Vec<&PayrollRow>> = grouped
        .entries
        .iter()
        .map(|(_, rows)| rows)
        .filter(|rows| {
            let mut days: Vec<Date> = rows.iter().map(|r| r.payday).collect();
            days.sort();
            days.dedup();
            days.len() > 1
        })
        .collect();
    if !allocation_groups.is_empty() && !statutory_allocation_confirmed {
        let mut affected: Vec<i64> = allocation_groups
            .iter()
            .flat_map(|rows| rows.iter().map(|r| r.row))
            .collect();
        affected.sort();
        let shown: Vec<String> = affected.iter().take(20).map(i64::to_string).collect();
        let more = if affected.len() > 20 {
            format!(" and {} more", affected.len() - 20)
        } else {
            String::new()
        };
        return Err(csv_error(format!(
            "{} employee allocation group(s) contain more than \
             one positive payday (payroll rows {}{more}). LCR 2026/2 \
             paragraphs 31-33 apply contributions in fund-receipt order to the \
             earliest QE day with a base or final shortfall. These exports contain \
             employer payment dates and vendor periods, not fund-receipt order or \
             assessment facts, so they cannot establish the canonical allocation. \
             Reconcile every relevant payday, contribution receipt and assessment, \
             then rerun with --confirm-statutory-allocation; no output was written",
            allocation_groups.len(),
            shown.join(", ")
        )));
    }
    let confirmed = !allocation_groups.is_empty();
    let warnings_head = pre_regime_warnings(&payroll_rows);
    let columns = Columns {
        payroll_has_period_end: resolved_get(&payroll_resolved, "period_end").is_some(),
        super_has_period_start: resolved_get(&super_resolved, "period_start").is_some(),
        super_has_period_end: resolved_get(&super_resolved, "period_end").is_some(),
    };
    let result = join::join(payroll_rows, super_rows, columns)?;
    write_canonical(&result, out_path)?;

    let mut outcome_counts: Vec<(String, usize)> = Vec::new();
    for outcome in &result.outcomes {
        let bucket = classify_outcome(outcome, &result.payroll_rows[outcome.payroll])?;
        match outcome_counts.iter_mut().find(|(b, _)| b == bucket) {
            Some(entry) => entry.1 += 1,
            None => outcome_counts.push((bucket.to_string(), 1)),
        }
    }
    let mut warnings = warnings_head;
    warnings.extend(result.warnings.iter().cloned());
    if confirmed {
        warnings.insert(
            0,
            "operator confirmed the LCR 2026/2 statutory allocation: all relevant \
             paydays, fund receipts and assessments were reconciled, and the export \
             periods plus payment-date/row order reproduce the fund-receipt order and \
             earliest-shortfall application"
                .into(),
        );
    }
    warnings.extend(
        result
            .outcomes
            .iter()
            .filter(|o| !o.flag.is_empty())
            .map(|o| format!("row {}: {}", result.payroll_rows[o.payroll].row, o.flag)),
    );
    warnings.extend(result.orphan_reasons.iter().map(|r| {
        format!(
            "super row {}: {}",
            result.super_rows[r.super_row].row, r.message
        )
    }));
    warnings.extend(result.super_rows.iter().filter_map(|s| {
        s.unpaid_status.as_ref().map(|status| {
            format!(
                "super row {}: status {} does not establish \
                 remittance to the fund, so its payment date is not evidence of remittance \
                 and was not used",
                s.row,
                pytext::repr(status)
            )
        })
    }));
    Ok(ImportReport {
        payroll_profile,
        super_profile,
        outcome_counts,
        orphan_reasons: result.orphan_reasons.clone(),
        super_rows: result.super_rows.clone(),
        key_mode: result.key_mode.clone(),
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(value: &str, field: &str, row: i64) -> String {
        match amount(value, field, row) {
            Ok(d) => d.to_sci_string(),
            Err(f) => f.message(),
        }
    }

    #[test]
    fn the_canonical_header_is_the_checkers_default_mapping() {
        let mapped: Vec<&str> = crate::csv_io::DEFAULT_MAPPING
            .iter()
            .map(|(_, c)| *c)
            .collect();
        assert_eq!(CANONICAL_HEADER.to_vec(), mapped);
    }

    #[test]
    fn amounts_are_read_to_the_cent_half_up() {
        for (value, want) in [
            ("540.004", "540.00"),
            ("539.996", "540.00"),
            ("612.005", "612.01"),
            ("0.005", "0.01"),
            ("1e2", "100.00"),
            ("1,234.567", "1234.57"),
            ("612", "612.00"),
            ("0", "0.00"),
            ("0.00", "0.00"),
        ] {
            assert_eq!(read(value, "amount", 2), want, "{value}");
        }
    }

    #[test]
    fn an_amount_under_half_a_cent_or_negative_is_refused() {
        assert!(
            read("0.004", "amount", 4).contains("row 4: amount value '0.004' is under half a cent")
        );
        assert!(
            read("0.0049", "sg amount", 2)
                .contains("sg amount value '0.0049' is under half a cent")
        );
        for negative in ["-0.004", "-0.01", "($ 612.00)"] {
            assert!(
                read(negative, "sg_amount", 2).contains("negative"),
                "{negative}"
            );
        }
    }
}
