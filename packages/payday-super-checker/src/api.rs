//! JSON stdin boundary for the removed Python import API. Decimals are
//! strings, dates are `YYYY-MM-DD` strings, files are paths. A calendar is
//! the bundled table with an optional override file, or explicit holidays; a
//! GIC table is the bundled one or explicit quarters. Results mirror the
//! Python objects' fields.

use serde_json::{Map, Value, json};

use crate::Failure;
use crate::assess::{self, Assessed, Options};
use crate::calendar::{self, BusinessCalendar, Holiday};
use crate::csv_io::{self, Mapping};
use crate::deadlines::{self, ContribLine, Deadline};
use crate::decimal::Decimal;
use crate::evidence_pack::{self, PackFacts};
use crate::importers;
use crate::join::{self, Columns, JoinResult, PayrollRow, SuperRow};
use crate::practitioner_pack::{self, ReportSnapshot};
use crate::profiles::{self, Profile};
use crate::py::Date;
use crate::rates::{self, GicQuarter, GicTable, TableSource};
use crate::report::{self, Figures, NoteFacts, SummaryFacts};
use crate::sgc;

type Result<T> = std::result::Result<T, String>;

pub const COMMANDS: [&str; 30] = [
    "parse-date-text",
    "parse-amount",
    "csv-safe",
    "money",
    "load-mapping",
    "parse-rows",
    "load-calendar",
    "add-business-days",
    "load-gic",
    "daily-rate",
    "notional-earnings",
    "uplift-scenarios",
    "exposure-range",
    "compute-due",
    "assess",
    "render-csv",
    "console-summary",
    "needs-attention",
    "financial-year",
    "build-evidence-pack",
    "parse-report-snapshot",
    "render-practitioner-pack",
    "normalise-header",
    "normalise-name",
    "load-profiles",
    "detect-profile",
    "read-payroll",
    "read-super",
    "join",
    "import-files",
];

fn failure(f: Failure) -> String {
    match f {
        Failure::Value(m) | Failure::StaleGic(m) => m,
        Failure::Os(e) => e.display(),
        other => other.traceback_line(),
    }
}

/// Standard base64, as the harness writes file bytes.
fn base64_decode(text: &str) -> Result<Vec<u8>> {
    let value = |c: u8| -> Result<u32> {
        Ok(match c {
            b'A'..=b'Z' => u32::from(c - b'A'),
            b'a'..=b'z' => u32::from(c - b'a') + 26,
            b'0'..=b'9' => u32::from(c - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err("data_b64 must be base64".into()),
        })
    };
    let digits: Vec<u8> = text.bytes().filter(|b| *b != b'=').collect();
    let mut out = Vec::with_capacity(digits.len() * 3 / 4);
    for chunk in digits.chunks(4) {
        let mut acc = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            acc |= value(*c)? << (18 - 6 * i);
        }
        let bytes = acc.to_be_bytes();
        out.extend_from_slice(&bytes[1..chunk.len()]);
    }
    Ok(out)
}

fn get<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.get(key).filter(|x| !x.is_null())
}

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))
}

fn optional_text<'a>(v: &'a Value, key: &str) -> Result<Option<&'a str>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(format!("{key} must be a string or null")),
    }
}

/// A field whose Python default is a string: absent takes it; `null` and
/// other types are refused, since the API passes no Python object through.
fn text_or<'a>(v: &'a Value, key: &str, default: &'a str) -> Result<&'a str> {
    match v.get(key) {
        None => Ok(default),
        Some(_) => text(v, key),
    }
}

/// A boolean whose Python default is False: absent takes it; `null` and
/// other types are refused.
fn flag(v: &Value, key: &str) -> Result<bool> {
    match v.get(key) {
        None => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("{key} must be a boolean")),
    }
}

/// A boolean the Python function or dataclass required.
fn required_flag(v: &Value, key: &str) -> Result<bool> {
    match v.get(key) {
        Some(Value::Bool(b)) => Ok(*b),
        _ => Err(format!("{key} must be a boolean")),
    }
}

/// A control the Python API required to be a literal boolean: absent takes
/// `default`; anything else, `null` included, is refused with its message.
fn control(v: &Value, key: &str, default: bool) -> Result<bool> {
    match v.get(key) {
        None => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("{key} must be a boolean")),
    }
}

/// An integer with a Python default: absent takes it; anything but a 64-bit
/// JSON integer is refused.
fn int(v: &Value, key: &str, default: i64) -> Result<i64> {
    match v.get(key) {
        None => Ok(default),
        Some(_) => required_int(v, key),
    }
}

fn required_int(v: &Value, key: &str) -> Result<i64> {
    v.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{key} must be an integer"))
}

fn strings(v: &Value, key: &str) -> Result<Vec<String>> {
    match v.get(key) {
        None => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|x| {
                x.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("{key} must be a list of strings"))
            })
            .collect(),
        Some(_) => Err(format!("{key} must be a list of strings")),
    }
}

/// A list Python accepted as `None` too (`explicit`, `overrides`).
fn optional_strings(v: &Value, key: &str) -> Result<Vec<String>> {
    match v.get(key) {
        Some(Value::Null) => Ok(Vec::new()),
        _ => strings(v, key),
    }
}

/// A list the Python function or dataclass required.
fn required_strings(v: &Value, key: &str) -> Result<Vec<String>> {
    match v.get(key) {
        Some(Value::Array(_)) => strings(v, key),
        _ => Err(format!("{key} must be a list of strings")),
    }
}

fn date_of(text: &str, key: &str) -> Result<Date> {
    Date::fromisoformat(text)
        .filter(|d| d.isoformat() == text)
        .ok_or_else(|| format!("{key} must be a date in YYYY-MM-DD form"))
}

fn date(v: &Value, key: &str) -> Result<Date> {
    date_of(text(v, key)?, key)
}

fn optional_date(v: &Value, key: &str) -> Result<Option<Date>> {
    optional_text(v, key)?.map(|t| date_of(t, key)).transpose()
}

fn decimal_of(text: &str) -> Result<Decimal> {
    Decimal::parse(text).map_err(|s| format!("decimal.{}", s.name()))
}

fn decimal(v: &Value, key: &str) -> Result<Decimal> {
    decimal_of(text(v, key)?)
}

fn optional_decimal(v: &Value, key: &str) -> Result<Option<Decimal>> {
    optional_text(v, key)?.map(decimal_of).transpose()
}

fn iso(d: Option<Date>) -> Value {
    d.map_or(Value::Null, |d| Value::String(d.isoformat()))
}

fn dec(d: Option<&Decimal>) -> Value {
    d.map_or(Value::Null, |d| Value::String(d.to_sci_string()))
}

// --- contribution lines, calendars and GIC tables ----------------------------

fn line_of(v: &Value) -> Result<ContribLine> {
    if !v.is_object() {
        return Err("a contribution line must be an object".into());
    }
    Ok(ContribLine {
        employee_id: text(v, "employee_id")?.to_string(),
        qe_day: date(v, "qe_day")?,
        sg_amount: decimal(v, "sg_amount")?,
        remitted: optional_date(v, "remitted")?,
        remitted_amount: optional_decimal(v, "remitted_amount")?,
        received: optional_date(v, "received")?,
        first_to_fund: flag(v, "first_to_fund")?,
        out_of_cycle: flag(v, "out_of_cycle")?,
        next_standard_qe_day: optional_date(v, "next_standard_qe_day")?,
        db_interest: flag(v, "db_interest")?,
        row: int(v, "row", 0)?,
        duplicate_note: text_or(v, "duplicate_note", "")?.to_string(),
        matched_amount: optional_decimal(v, "matched_amount")?,
        join_caveats: strings(v, "join_caveats")?,
    })
}

fn lines_of(v: &Value) -> Result<Vec<ContribLine>> {
    match v.get("lines") {
        Some(Value::Array(items)) => items.iter().map(line_of).collect(),
        _ => Err("lines must be a list of contribution lines".into()),
    }
}

fn line_json(line: &ContribLine) -> Value {
    json!({
        "employee_id": line.employee_id,
        "qe_day": line.qe_day.isoformat(),
        "sg_amount": line.sg_amount.to_sci_string(),
        "remitted": iso(line.remitted),
        "remitted_amount": dec(line.remitted_amount.as_ref()),
        "received": iso(line.received),
        "first_to_fund": line.first_to_fund,
        "out_of_cycle": line.out_of_cycle,
        "next_standard_qe_day": iso(line.next_standard_qe_day),
        "db_interest": line.db_interest,
        "row": line.row,
        "duplicate_note": line.duplicate_note,
        "matched_amount": dec(line.matched_amount.as_ref()),
        "join_caveats": line.join_caveats,
    })
}

fn calendar_of(v: &Value) -> Result<BusinessCalendar> {
    let Some(spec) = get(v, "calendar") else {
        return calendar::load_calendar(None).map_err(failure);
    };
    if let Some(path) = optional_text(spec, "override_path")? {
        return calendar::load_calendar(Some(path)).map_err(failure);
    }
    let Some(Value::Array(items)) = spec.get("holidays") else {
        return calendar::load_calendar(None).map_err(failure);
    };
    let mut holidays = Vec::new();
    for item in items {
        holidays.push(Holiday {
            day: date(item, "day")?,
            name: text(item, "name")?.to_string(),
            jurisdictions: required_strings(item, "jurisdictions")?,
            provisional: required_flag(item, "provisional")?,
        });
    }
    Ok(BusinessCalendar::new(
        holidays,
        date(spec, "verified_from")?,
        date(spec, "verified_until")?,
        optional_date(spec, "coverage_until")?,
    ))
}

fn calendar_json(cal: &BusinessCalendar) -> Value {
    json!({
        "verified_from": cal.verified_from.isoformat(),
        "verified_until": cal.verified_until.isoformat(),
        "coverage_until": cal.coverage_until.isoformat(),
        "holidays": cal.holidays().map(|h| json!({
            "day": h.day.isoformat(),
            "name": h.name,
            "jurisdictions": h.jurisdictions,
            "provisional": h.provisional,
        })).collect::<Vec<_>>(),
    })
}

fn gic_of(v: &Value) -> Result<GicTable> {
    let Some(Value::Array(items)) = get(v, "quarters") else {
        return rates::load_gic().map_err(failure);
    };
    let mut quarters = Vec::new();
    for item in items {
        quarters.push(GicQuarter {
            start: date(item, "from")?,
            end: date(item, "to")?,
            annual_pct: decimal(item, "annual_pct")?,
            seen: text_or(item, "seen", "")?.to_string(),
        });
    }
    GicTable::new(quarters, None).map_err(failure)
}

fn gic_json(gic: &GicTable) -> Value {
    json!({
        "source": gic.source.as_ref().map(|s| json!({"uri": s.uri, "sha256": s.sha256})),
        "provenance": gic.provenance(),
        "quarters": gic.quarters().iter().map(|q| json!({
            "from": q.start.isoformat(),
            "to": q.end.isoformat(),
            "annual_pct": q.annual_pct.to_sci_string(),
            "seen": q.seen,
        })).collect::<Vec<_>>(),
    })
}

fn deadline_json(dl: &Deadline) -> Value {
    json!({
        "due": iso(dl.due),
        "pathway": dl.pathway,
        "notes": dl.notes,
        "caveats": dl.caveats,
        "own_due": iso(dl.own_due),
        "item2_due": iso(dl.item2_due),
        "possible_item4_due": iso(dl.possible_item4_due),
    })
}

fn uplift_json(uplift: &sgc::Uplift) -> Value {
    let mut out = Map::new();
    for (history, row) in uplift {
        let mut scenarios = Map::new();
        for (scenario, value) in row {
            scenarios.insert(scenario.clone(), Value::String(value.to_sci_string()));
        }
        out.insert(history.clone(), Value::Object(scenarios));
    }
    Value::Object(out)
}

fn figures_json(f: &Figures) -> Value {
    json!({
        "shortfall": dec(f.shortfall.as_ref()),
        "nec": dec(f.nec.as_ref()),
        "up_low": dec(f.up_low.as_ref()),
        "up_high": dec(f.up_high.as_ref()),
        "low": dec(f.low.as_ref()),
        "high": dec(f.high.as_ref()),
    })
}

fn result_json(r: &Assessed) -> Result<Value> {
    Ok(json!({
        "line": line_json(&r.line),
        "deadline": deadline_json(&r.deadline),
        "verdict": r.verdict,
        "days_late": r.days_late,
        "lateness_basis": r.lateness_basis,
        "base_shortfall": dec(r.base_shortfall.as_ref()),
        "final_shortfall": dec(r.final_shortfall.as_ref()),
        "offset_s18d": r.offset_s18d,
        "nec": dec(r.nec.as_ref()),
        "sgc_low": dec(r.sgc_low.as_ref()),
        "sgc_high": dec(r.sgc_high.as_ref()),
        "uplift": r.uplift.as_ref().map(uplift_json),
        "notes": r.notes,
        "caveats": r.caveats,
        "horizon_verdicts": r.horizon_verdicts.as_ref().map(|(a, b)| json!([a, b])),
        "receipt_established": r.receipt_established,
        // Python's assess() did no display rounding, so a figure that cannot
        // be rounded to cents leaves `figures` null rather than failing the call.
        "figures": report::rounded_figures(r).ok().map(|f| figures_json(&f)),
    }))
}

/// The assessment every report command starts from.
fn assessed(v: &Value) -> Result<(Vec<Assessed>, Date, Option<Date>, GicTable)> {
    let lines = lines_of(v)?;
    let cal = calendar_of(v)?;
    let gic = gic_of(v)?;
    let as_at = date(v, "as_at")?;
    let assessment_date = optional_date(v, "assessment_date")?;
    // The amount and date problems come first, then the controls, as Python checked them.
    assess::check_inputs(&lines).map_err(failure)?;
    let transition_allocation_confirmed = control(v, "transition_allocation_confirmed", false)?;
    let allow_stale_gic = control(v, "allow_stale_gic", false)?;
    let results = assess::assess(
        lines,
        &cal,
        &gic,
        as_at,
        Options {
            assessment_date,
            transition_allocation_confirmed,
            allow_stale_gic,
        },
    )
    .map_err(failure)?;
    Ok((results, as_at, assessment_date, gic))
}

// --- reports, packs and profiles ----------------------------------------------

fn snapshot_of(v: &Value) -> Result<ReportSnapshot> {
    if let Some(path) = optional_text(v, "path")? {
        return practitioner_pack::load_report_snapshot(path).map_err(failure);
    }
    let data = match optional_text(v, "data_b64")? {
        Some(encoded) => base64_decode(encoded)?,
        None => text(v, "text")?.as_bytes().to_vec(),
    };
    practitioner_pack::parse_report_snapshot(
        &data,
        text(v, "source")?,
        flag(v, "identifiers_omitted")?,
    )
    .map_err(failure)
}

fn snapshot_json(s: &ReportSnapshot) -> Value {
    json!({
        "source_path": s.source_path,
        "source_sha256": s.source_sha256,
        "provenance": s.provenance,
        "identifiers_omitted": s.identifiers_omitted,
        "needs_attention": s.needs_attention(),
        "rows": s.rows.iter().map(|r| {
            let mut row = json!({
                "source_row": r.source_row.0,
                "qe_day": r.qe_day.isoformat(),
                "pathway": r.pathway,
                "due_date": iso(r.due_date),
                "verdict": r.verdict,
                "days_late": r.days_late,
                "lateness_measured_to": r.lateness_measured_to,
                "sg_amount": r.sg_amount.to_sci_string(),
                "caveats": r.caveats,
                "notes": r.notes,
                "unassessable_between": r.unassessable_between,
            });
            for field in practitioner_pack::EXPOSURE_FIELDS {
                row[field] = dec(r.amount(field));
            }
            row
        }).collect::<Vec<_>>(),
    })
}

fn profile_json(p: &Profile) -> Value {
    json!({
        "key": p.key,
        "name": p.name,
        "role": p.role,
        "verified": p.verified,
        "signature": p.signature,
        "columns": p.columns.iter().map(|(f, h)| json!([f, h])).collect::<Vec<_>>(),
        "date_formats": p.date_formats,
        "sg_filter": p.sg_filter.as_ref().map(|f| json!({"column": f.column, "include": f.include})),
        "remitted_status": p.remitted_status.as_ref().map(|s| json!({
            "column": s.column, "sent": s.sent, "not_sent": s.not_sent,
        })),
        "notes": p.notes,
    })
}

fn resolved_json(resolved: &[(String, String)]) -> Value {
    Value::Array(resolved.iter().map(|(f, h)| json!([f, h])).collect())
}

fn payroll_json(r: &PayrollRow) -> Value {
    json!({
        "employee_id": r.employee_id,
        "employee_name": r.employee_name,
        "payday": r.payday.isoformat(),
        "period_end": iso(r.period_end),
        "sg_amount": r.sg_amount.to_sci_string(),
        "row": r.row,
    })
}

fn super_json(r: &SuperRow) -> Value {
    json!({
        "employee_id": r.employee_id,
        "employee_name": r.employee_name,
        "period_start": iso(r.period_start),
        "period_end": iso(r.period_end),
        "paid_date": iso(r.paid_date),
        "amount": r.amount.to_sci_string(),
        "row": r.row,
        "unpaid_status": r.unpaid_status,
    })
}

fn payroll_of(v: &Value) -> Result<PayrollRow> {
    Ok(PayrollRow {
        employee_id: optional_text(v, "employee_id")?.map(str::to_string),
        employee_name: optional_text(v, "employee_name")?.map(str::to_string),
        payday: date(v, "payday")?,
        period_end: optional_date(v, "period_end")?,
        sg_amount: decimal(v, "sg_amount")?,
        row: required_int(v, "row")?,
    })
}

fn super_of(v: &Value) -> Result<SuperRow> {
    Ok(SuperRow {
        employee_id: optional_text(v, "employee_id")?.map(str::to_string),
        employee_name: optional_text(v, "employee_name")?.map(str::to_string),
        period_start: optional_date(v, "period_start")?,
        period_end: optional_date(v, "period_end")?,
        paid_date: optional_date(v, "paid_date")?,
        amount: decimal(v, "amount")?,
        row: required_int(v, "row")?,
        unpaid_status: optional_text(v, "unpaid_status")?.map(str::to_string),
    })
}

fn list_of<T>(v: &Value, key: &str, f: fn(&Value) -> Result<T>) -> Result<Vec<T>> {
    match v.get(key) {
        Some(Value::Array(items)) => items.iter().map(f).collect(),
        _ => Err(format!("{key} must be a list")),
    }
}

fn join_json(result: &JoinResult) -> Result<Value> {
    let mut outcomes = Vec::new();
    for o in &result.outcomes {
        let row = &result.payroll_rows[o.payroll];
        outcomes.push(json!({
            "payroll": payroll_json(row),
            "remitted": iso(o.remitted),
            "flag": o.flag,
            "last_known_paid_date": iso(o.last_known_paid_date),
            "remitted_amount": dec(o.remitted_amount.as_ref()),
            "matched_amount": dec(o.matched_amount.as_ref()),
            "outcome": importers::classify_outcome(o, row).map_err(failure)?,
        }));
    }
    Ok(json!({
        "outcomes": outcomes,
        "orphans": result.orphans.iter().map(|&i| super_json(&result.super_rows[i])).collect::<Vec<_>>(),
        "key_mode": result.key_mode,
        "warnings": result.warnings,
        "orphan_reasons": result.orphan_reasons.iter().map(|r| json!({
            "super_row": super_json(&result.super_rows[r.super_row]),
            "code": r.code,
            "message": r.message,
        })).collect::<Vec<_>>(),
        "canonical_csv": importers::canonical_csv(result).map_err(failure)?,
    }))
}

fn mapping_of(v: &Value) -> Result<Mapping> {
    if let Some(Value::Array(pairs)) = v.get("mapping") {
        let mut columns = Vec::new();
        for pair in pairs {
            let (Some(field), Some(column)) = (
                pair.get(0).and_then(Value::as_str),
                pair.get(1).and_then(Value::as_str),
            ) else {
                return Err("mapping must be a list of [field, column] pairs".into());
            };
            columns.push((field.to_string(), column.to_string()));
        }
        return Ok(Mapping {
            columns,
            explicit: optional_strings(v, "explicit")?.into_iter().collect(),
        });
    }
    csv_io::load_mapping(
        optional_text(v, "mapping_file")?,
        &optional_strings(v, "overrides")?,
    )
    .map_err(failure)
}

/// Run one JSON stdin command.
pub fn evaluate(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return Err("the JSON input must be an object".into());
    }
    match command {
        "parse-date-text" => Ok(iso(
            csv_io::parse_date_text(text(v, "text")?).map_err(failure)?
        )),
        "parse-amount" => Ok(dec(Some(
            &csv_io::parse_amount(
                text(v, "value")?,
                text(v, "field")?,
                required_int(v, "row")?,
                flag(v, "empty_is_error")?,
            )
            .map_err(failure)?,
        ))),
        "csv-safe" => Ok(json!(csv_io::csv_safe(text(v, "text")?))),
        "money" => Ok(json!(
            csv_io::money(optional_decimal(v, "value")?.as_ref()).map_err(failure)?
        )),
        "load-mapping" => {
            let mapping = mapping_of(v)?;
            Ok(json!({
                "mapping": mapping.columns.iter().map(|(f, c)| json!([f, c])).collect::<Vec<_>>(),
                "explicit": mapping.explicit.iter().collect::<Vec<_>>(),
            }))
        }
        "parse-rows" => {
            let mapping = mapping_of(v)?;
            let lines = csv_io::parse_rows(text(v, "path")?, &mapping).map_err(failure)?;
            Ok(Value::Array(lines.iter().map(line_json).collect()))
        }
        "load-calendar" => Ok(calendar_json(&calendar_of(v)?)),
        "add-business-days" => {
            let cal = calendar_of(v)?;
            let day = cal
                .add_business_days(date(v, "date")?, required_int(v, "n")?)
                .map_err(failure)?;
            Ok(json!(day.isoformat()))
        }
        "load-gic" => Ok(gic_json(&gic_of(v)?)),
        "daily-rate" => {
            let gic = gic_of(v)?;
            let day = date(v, "date")?;
            Ok(dec(Some(
                &gic.daily_rate(day, control(v, "allow_stale", false)?)
                    .map_err(failure)?,
            )))
        }
        "notional-earnings" => {
            let gic = gic_of(v)?;
            let shortfall = decimal(v, "shortfall")?;
            if shortfall
                .lt(&Decimal::from_int(0))
                .map_err(|s| failure(s.into()))?
            {
                return Err("shortfall cannot be negative".into());
            }
            Ok(dec(Some(
                &sgc::notional_earnings(
                    &shortfall,
                    date(v, "due")?,
                    date(v, "end")?,
                    &gic,
                    control(v, "allow_stale", false)?,
                )
                .map_err(failure)?,
            )))
        }
        "uplift-scenarios" => Ok(uplift_json(
            &sgc::uplift_scenarios(&decimal(v, "final_shortfall")?, &decimal(v, "nec")?)
                .map_err(failure)?,
        )),
        "exposure-range" => {
            let (low, high) =
                sgc::exposure_range(&decimal(v, "final_shortfall")?, &decimal(v, "nec")?)
                    .map_err(failure)?;
            Ok(json!([low.to_sci_string(), high.to_sci_string()]))
        }
        "compute-due" => {
            let cal = calendar_of(v)?;
            let line = line_of(v.get("line").ok_or("line must be an object")?)?;
            Ok(deadline_json(
                &deadlines::compute_due(&line, &cal).map_err(failure)?,
            ))
        }
        "assess" => {
            let (results, ..) = assessed(v)?;
            Ok(Value::Array(
                results.iter().map(result_json).collect::<Result<_>>()?,
            ))
        }
        "render-csv" => {
            let (results, as_at, assessment_date, _) = assessed(v)?;
            let include_employee_ids = control(v, "include_employee_ids", true)?;
            Ok(json!(
                report::render_csv(
                    &results,
                    &NoteFacts {
                        as_at,
                        law_date: text(v, "law_date")?,
                        assessment_date,
                        source: optional_text(v, "source")?,
                        gic_provenance: text_or(v, "gic_provenance", "")?,
                    },
                    include_employee_ids,
                )
                .map_err(failure)?
            ))
        }
        "console-summary" => {
            let (results, as_at, assessment_date, _) = assessed(v)?;
            let remittance_only_confirmed = control(v, "remittance_only_confirmed", false)?;
            let rates = crate::rates::load_rates().map_err(failure)?;
            Ok(json!(
                report::console_summary(
                    &results,
                    &SummaryFacts {
                        as_at,
                        csv_path: text(v, "csv_path")?,
                        law_date: text(v, "law_date")?,
                        rates: &rates,
                        assessment_date,
                        remittance_only_confirmed,
                    },
                )
                .map_err(failure)?
            ))
        }
        "needs-attention" => {
            let (results, ..) = assessed(v)?;
            Ok(json!(
                report::needs_attention(&results, control(v, "remittance_only_confirmed", false)?)
                    .map_err(failure)?
            ))
        }
        "financial-year" => Ok(json!(assess::financial_year(date(v, "date")?))),
        "build-evidence-pack" => {
            let (results, as_at, assessment_date, _) = assessed(v)?;
            let remittance_only_confirmed = control(v, "remittance_only_confirmed", false)?;
            // Python's defaults: no rate tables and no provenance line.
            let tables: Vec<TableSource> = match v.get("rate_tables") {
                None => Vec::new(),
                Some(Value::Array(items)) => items
                    .iter()
                    .map(|t| {
                        Ok(TableSource {
                            uri: text(t, "uri")?.to_string(),
                            sha256: text(t, "sha256")?.to_string(),
                        })
                    })
                    .collect::<Result<_>>()?,
                Some(_) => return Err("rate_tables must be a list".into()),
            };
            let provenance = text_or(v, "gic_provenance", "")?.to_string();
            let files = evidence_pack::build_evidence_pack(
                &results,
                &PackFacts {
                    as_at,
                    assessment_date,
                    gic_provenance: &provenance,
                    remittance_only_confirmed,
                    rate_tables: &tables,
                },
            )
            .map_err(failure)?;
            Ok(Value::Array(
                files.iter().map(|(n, t)| json!([n, t])).collect(),
            ))
        }
        "parse-report-snapshot" => Ok(snapshot_json(&snapshot_of(v)?)),
        "render-practitioner-pack" => Ok(json!(
            practitioner_pack::render_practitioner_pack(&snapshot_of(v)?, flag(v, "decision_log")?)
                .map_err(failure)?
        )),
        "normalise-header" => Ok(json!(profiles::normalise_header(text(v, "text")?))),
        "normalise-name" => Ok(json!(profiles::normalise_name(text(v, "text")?))),
        "load-profiles" => Ok(Value::Array(
            profiles::load_profiles(optional_text(v, "role")?)
                .map_err(failure)?
                .iter()
                .map(profile_json)
                .collect(),
        )),
        "detect-profile" => {
            let headers = required_strings(v, "headers")?;
            let profile = profiles::detect(&headers, text(v, "role")?, optional_text(v, "vendor")?)
                .map_err(failure)?;
            Ok(json!({
                "key": profile.key,
                "score": profiles::score(&profile, &headers),
                "resolved": resolved_json(&profiles::resolve_columns(&profile, &headers)),
            }))
        }
        "read-payroll" => {
            let (rows, profile, resolved) =
                importers::read_payroll(text(v, "path")?, optional_text(v, "vendor")?)
                    .map_err(failure)?;
            Ok(json!({
                "rows": rows.iter().map(payroll_json).collect::<Vec<_>>(),
                "profile": profile.key,
                "resolved": resolved_json(&resolved),
            }))
        }
        "read-super" => {
            let (rows, profile, resolved) =
                importers::read_super(text(v, "path")?, optional_text(v, "vendor")?)
                    .map_err(failure)?;
            Ok(json!({
                "rows": rows.iter().map(super_json).collect::<Vec<_>>(),
                "profile": profile.key,
                "resolved": resolved_json(&resolved),
            }))
        }
        "join" => {
            let defaults = Columns::default();
            // Python read these flags by truthiness.
            let present = |key: &str, default: bool| match v.get(key) {
                Some(value) => crate::raw::Raw::from_value(value).truthy(),
                None => default,
            };
            let result = join::join(
                list_of(v, "payroll_rows", payroll_of)?,
                list_of(v, "super_rows", super_of)?,
                Columns {
                    payroll_has_period_end: present(
                        "payroll_has_period_end",
                        defaults.payroll_has_period_end,
                    ),
                    super_has_period_start: present(
                        "super_has_period_start",
                        defaults.super_has_period_start,
                    ),
                    super_has_period_end: present(
                        "super_has_period_end",
                        defaults.super_has_period_end,
                    ),
                },
            )
            .map_err(failure)?;
            join_json(&result)
        }
        "import-files" => {
            let (payroll, super_path, output) =
                (text(v, "payroll")?, text(v, "super")?, text(v, "output")?);
            importers::check_output(payroll, super_path, output).map_err(failure)?;
            let confirmed = control(v, "statutory_allocation_confirmed", false)
                .map_err(|_| "statutory_allocation_confirmed must be a boolean".to_string())?;
            let report = importers::import_files(
                payroll,
                super_path,
                output,
                optional_text(v, "vendor")?,
                confirmed,
            )
            .map_err(failure)?;
            Ok(json!({
                "payroll_profile": profile_json(&report.payroll_profile),
                "super_profile": profile_json(&report.super_profile),
                "outcome_counts": report.outcome_counts.iter().map(|(b, n)| json!([b, n])).collect::<Vec<_>>(),
                "orphan_reasons": report.orphan_reasons.iter().map(|r| json!({
                    "super_row": super_json(&report.super_rows[r.super_row]),
                    "code": r.code,
                    "message": r.message,
                })).collect::<Vec<_>>(),
                "key_mode": report.key_mode,
                "warnings": report.warnings,
                "clean": report.clean(),
            }))
        }
        _ => Err(format!("unknown command {command}")),
    }
}
