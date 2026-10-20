//! CSV schedule, console summary and practitioner review pack, as
//! `wiptally.report`, `wiptally.csvsafe` and `wiptally.atomic_io` wrote them.

use std::io::Write;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::csv_io::Failure;
use crate::decimal::{Calc, Decimal};
use crate::engine::{ContractPosition, Schedule};
use crate::money::{money, percent, points};
use crate::py::{self, OsStyle, PyPath};
use crate::pytext;

pub const OUTPUT_COLUMNS: [&str; 35] = [
    "as_at",
    "contract_id",
    "customer",
    "description",
    "progress_method",
    "original_contract_sum",
    "approved_variations",
    "unapproved_variations_estimate",
    "variable_consideration_included",
    "variable_consideration_excluded",
    "transaction_price",
    "costs_incurred",
    "inefficiency_rework_wastage",
    "uninstalled_materials",
    "progress_cost",
    "estimated_cost_to_complete",
    "estimated_cost_at_completion",
    "progress_eac",
    "percent_complete",
    "revenue_to_date",
    "certified_billings",
    "uncertified_claims",
    "contract_asset",
    "contract_liability",
    "gross_profit_at_completion",
    "margin_at_completion",
    "prior_margin_at_completion",
    "profit_fade_points",
    "period_revenue",
    "retention_withheld",
    "retention_classification",
    "gst_on_certified_billings",
    "gst_on_retention",
    "committed_outstanding",
    "flags",
];

pub const SCHEMA_VERSION: &str = "wip-tally-schedule-v2";

/// `csvsafe.guard`: neutralise a value a spreadsheet would read as a formula.
pub fn guard(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let stripped = text.trim_start_matches(|c: char| pytext::is_space(c as u32));
    let first = stripped.chars().next();
    if matches!(first, Some('=' | '@')) {
        return format!("'{text}");
    }
    if matches!(first, Some('+' | '-')) && !looks_numeric(&stripped[1..]) {
        return format!("'{text}");
    }
    if matches!(text.chars().next(), Some('\t' | '\r' | '\n')) {
        return format!("'{text}");
    }
    text.to_string()
}

/// `_NUMBER_RE.match`: `^[\d,]*+(?:\.\d++|(?<=\d))$` with Unicode `\d`.
fn looks_numeric(s: &str) -> bool {
    let digit = |c: char| pytext::decimal_value(c as u32).is_some();
    let s = s.strip_suffix('\n').unwrap_or(s);
    let run = s
        .char_indices()
        .find(|(_, c)| !digit(*c) && *c != ',')
        .map_or(s.len(), |(i, _)| i);
    let (prefix, rest) = s.split_at(run);
    match rest.strip_prefix('.') {
        Some(frac) => !frac.is_empty() && frac.chars().all(digit),
        None => rest.is_empty() && prefix.chars().last().is_some_and(digit),
    }
}

/// `format(value, "f")`, or blank for None.
fn dec(value: Option<&Decimal>) -> String {
    value.map_or_else(String::new, Decimal::fixed_format)
}

/// `" ".join(value.split())`.
fn one_line(value: &str) -> String {
    py::split_whitespace(value).join(" ")
}

/// `_md_cell`: backslashes first, then pipes, so the identifier round-trips.
pub fn md_cell(value: &str) -> String {
    one_line(value).replace('\\', "\\\\").replace('|', "\\|")
}

fn row(position: &ContractPosition, as_at: &str) -> Vec<String> {
    let c = &position.contract;
    let p = position;
    vec![
        guard(as_at),
        guard(&c.contract_id),
        guard(&c.customer),
        guard(&c.description),
        c.progress_method.clone(),
        dec(Some(&c.original_contract_sum)),
        dec(Some(&c.approved_variations)),
        dec(Some(&c.unapproved_variations_estimate)),
        dec(Some(&p.variable_consideration_included)),
        dec(Some(&p.variable_consideration_excluded)),
        dec(Some(&p.transaction_price)),
        dec(Some(&c.costs_incurred)),
        dec(Some(&c.inefficiency_rework_wastage)),
        dec(Some(&c.uninstalled_materials)),
        dec(Some(&p.progress_cost)),
        dec(Some(&c.estimated_cost_to_complete)),
        dec(Some(&p.estimated_cost_at_completion)),
        dec(Some(&p.progress_eac)),
        dec(Some(&p.percent_complete)),
        dec(Some(&p.revenue_to_date)),
        dec(Some(&c.certified_billings)),
        dec(Some(&c.uncertified_claims)),
        dec(Some(&p.contract_asset)),
        dec(Some(&p.contract_liability)),
        dec(Some(&p.gross_profit_at_completion)),
        dec(p.margin_at_completion.as_ref()),
        dec(p.prior_margin_at_completion.as_ref()),
        dec(p.profit_fade_points.as_ref()),
        dec(p.period_revenue.as_ref()),
        dec(Some(&c.retention_withheld)),
        c.retention_classification.clone(),
        dec(Some(&p.gst_on_certified_billings)),
        dec(Some(&p.gst_on_retention)),
        dec(Some(&c.committed_outstanding)),
        p.flags.join("|"),
    ]
}

/// `csv.writer(lineterminator="\n")` with QUOTE_MINIMAL for one row.
pub fn csv_line(fields: &[String], out: &mut String) {
    csv_line_ending(fields, out, "\n");
}

/// One QUOTE_MINIMAL row with the given `lineterminator`.
pub fn csv_line_ending(fields: &[String], out: &mut String, terminator: &str) {
    if fields.len() == 1 && fields[0].is_empty() {
        out.push_str("\"\"");
        out.push_str(terminator);
        return;
    }
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        if field.contains([',', '"', '\r', '\n']) {
            out.push('"');
            out.push_str(&field.replace('"', "\"\""));
            out.push('"');
        } else {
            out.push_str(field);
        }
    }
    out.push_str(terminator);
}

/// The schedule CSV text, exactly as written to disk.
pub fn schedule_csv(schedule: &Schedule) -> String {
    let mut out = String::new();
    let header: Vec<String> = OUTPUT_COLUMNS.iter().map(|c| c.to_string()).collect();
    csv_line(&header, &mut out);
    for position in &schedule.positions {
        csv_line(&row(position, &schedule.as_at), &mut out);
    }
    out
}

pub fn render_console(schedule: &Schedule) -> Calc<String> {
    let mut lines = vec![
        format!(
            "TheWIPTally: {} contract(s), as at {}",
            schedule.positions.len(),
            schedule.as_at
        ),
        String::new(),
        format!(
            "  Contract assets:      {}",
            money(&schedule.total_contract_assets()?)?
        ),
        format!(
            "  Contract liabilities: {}",
            money(&schedule.total_contract_liabilities()?)?
        ),
        format!(
            "  Revenue to date:      {}",
            money(&schedule.total_revenue_to_date()?)?
        ),
        String::new(),
        "  Assets and liabilities are per contract. They are not netted.".into(),
    ];
    let review = schedule.review_rows();
    if review.is_empty() {
        lines.push(String::new());
        lines.push("  No review flags. Practitioner sign-off is still required.".into());
        return Ok(lines.join("\n") + "\n");
    }
    lines.push(String::new());
    lines.push(format!("  Review flags on {} contract(s):", review.len()));
    // sorted(..., key=fade or 0, reverse=True): stable, largest first.
    let zero = crate::money::d("0");
    let key = |row: &ContractPosition| row.profit_fade_points.clone().unwrap_or(zero.clone());
    let mut ranked = review.clone();
    let mut failure = None;
    ranked.sort_by(|a, b| match key(b).compare(&key(a)) {
        Ok(ordering) => ordering,
        Err(signal) => {
            failure = Some(signal);
            std::cmp::Ordering::Equal
        }
    });
    if let Some(signal) = failure {
        return Err(signal);
    }
    for position in ranked {
        let fade = match &position.profit_fade_points {
            Some(p) => points(p)?,
            None => "n/a".into(),
        };
        let margin = match &position.margin_at_completion {
            Some(m) => percent(m)?,
            None => "n/a".into(),
        };
        lines.push(format!(
            "    {}  margin {margin}  fade {fade}",
            one_line(&position.contract.contract_id)
        ));
        lines.push(format!("      {}", position.flags.join(", ")));
    }
    Ok(lines.join("\n") + "\n")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `build_review_pack`: refuses a schedule whose bytes differ from the rebuild.
pub fn build_review_pack(
    schedule_path: &PyPath,
    schedule_bytes: &[u8],
    source_name: &str,
    source_bytes: &[u8],
    schedule: &Schedule,
) -> Result<String, Failure> {
    if schedule_bytes != schedule_csv(schedule).as_bytes() {
        return Err(Failure::Message(format!(
            "{schedule_path} does not match the rebuilt schedule; the source \
             file, the mapping and --as-at must be the ones that produced it"
        )));
    }
    let assets = money(&schedule.total_contract_assets()?)?;
    let liabilities = money(&schedule.total_contract_liabilities()?)?;
    let revenue = money(&schedule.total_revenue_to_date()?)?;
    let mut lines: Vec<String> = vec![
        "# Construction WIP review pack".into(),
        String::new(),
        "Review aid only. Not accounting, tax or legal advice. Not a journal,".into(),
        "not a lodgment, and not a determination under AASB 15 or AASB 137.".into(),
        String::new(),
        format!("- Schema: `{SCHEMA_VERSION}`"),
        format!("- As at: {}", schedule.as_at),
        format!("- Source file: `{source_name}`"),
        format!("- Source SHA-256: `{}`", sha256_hex(source_bytes)),
        format!("- Schedule file: `{}`", schedule_path.name()),
        format!("- Schedule SHA-256: `{}`", sha256_hex(schedule_bytes)),
        format!("- Contracts: {}", schedule.positions.len()),
        format!("- Contract assets (sum, not net): {assets}"),
        format!("- Contract liabilities (sum, not net): {liabilities}"),
        String::new(),
        "## Sign-off".into(),
        String::new(),
        "- [ ] Unit of account (AASB 15 para 17 combination) is accepted".into(),
        "- [ ] Over-time vs point-in-time conclusion sits with the engagement lead".into(),
        "- [ ] Cost to complete is current against programme and commitments".into(),
        "- [ ] Constrained variable consideration is accepted or excluded".into(),
        "- [ ] Retention classification under paras 105-108 is accepted".into(),
        "- [ ] Onerous-contract flags have been run through AASB 137, not booked from this file"
            .into(),
        "- [ ] GST on certified claims, including retention, has been considered for the BAS"
            .into(),
        "- [ ] Tax does not follow this schedule; the ATO long-term construction position is separate"
            .into(),
        String::new(),
        "## Portfolio".into(),
        String::new(),
        "| Side | Amount |".into(),
        "| --- | ---: |".into(),
        format!("| Contract assets | {assets} |"),
        format!("| Contract liabilities | {liabilities} |"),
        format!("| Revenue to date | {revenue} |"),
        String::new(),
        "Do not offset the two sides.".into(),
        String::new(),
        "## Review queue".into(),
        String::new(),
    ];
    let review = schedule.review_rows();
    if review.is_empty() {
        lines.push("No review flags were raised. Sign-off is still required.".into());
        lines.push(String::new());
    } else {
        lines.push("| Contract | Margin | Fade | Flags |".into());
        lines.push("| --- | --- | --- | --- |".into());
        for position in review {
            let margin = match &position.margin_at_completion {
                Some(m) => percent(m)?,
                None => String::new(),
            };
            let fade = match &position.profit_fade_points {
                Some(p) => points(p)?,
                None => String::new(),
            };
            lines.push(format!(
                "| {} | {margin} | {fade} | {} |",
                md_cell(&position.contract.contract_id),
                position.flags.join(", ")
            ));
        }
        lines.push(String::new());
    }
    lines.push("Keep this pack beside the schedule CSV. The CSV is the row-level evidence.".into());
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// `atomic_write_text`: stage beside the destination, flush, then replace it
/// in one filesystem operation so a failed rewrite cannot truncate it.
pub fn atomic_write(path: &PyPath, text: &str) -> Result<(), Failure> {
    let parent = path.parent();
    let os_error = |e: std::io::Error, name: &str| {
        Failure::Message(py::oserror_str(&e, name, OsStyle::Native))
    };
    std::fs::create_dir_all(parent.fs()).map_err(|e| os_error(e, &parent.display()))?;
    let (temporary, mut handle) = create_temporary(&parent, path.name())?;
    let written = handle
        .write_all(text.as_bytes())
        .and_then(|()| handle.sync_all());
    drop(handle);
    let result = written
        .map_err(|e| os_error(e, &temporary.display().to_string()))
        .and_then(|()| {
            std::fs::rename(&temporary, path.fs())
                .map_err(|e| os_error(e, &temporary.display().to_string()))
        });
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

/// `tempfile.mkstemp(dir=parent, prefix=f".{name}.", suffix=".tmp")`.
fn create_temporary(
    parent: &PyPath,
    name: &str,
) -> Result<(std::path::PathBuf, std::fs::File), Failure> {
    const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789_";
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_nanos() as u64)
        ^ u64::from(std::process::id()).rotate_left(32);
    for _ in 0..100 {
        let suffix: String = (0..8)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                LETTERS[(seed % LETTERS.len() as u64) as usize] as char
            })
            .collect();
        let candidate = parent.fs().join(format!(".{name}.{suffix}.tmp"));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        // `mkstemp` creates its file readable by the owner only.
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        match options.open(&candidate) {
            Ok(file) => return Ok((candidate, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                let shown = parent.join(&format!(".{name}.{suffix}.tmp")).display();
                return Err(Failure::Message(py::oserror_str(
                    &e,
                    &shown,
                    OsStyle::Errno,
                )));
            }
        }
    }
    Err(Failure::Message(format!(
        "[Errno 17] No usable temporary file name found: {}",
        pytext::repr(&parent.display())
    )))
}

/// Read a file whole, rendering a failure as `str(OSError)`.
pub fn read_bytes(path: &PyPath) -> Result<Vec<u8>, Failure> {
    std::fs::read(path.fs())
        .map_err(|e| Failure::Message(py::oserror_str(&e, &path.display(), OsStyle::Errno)))
}

pub fn exists(path: &Path) -> bool {
    path.exists()
}
