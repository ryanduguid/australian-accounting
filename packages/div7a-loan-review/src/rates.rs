//! The benchmark interest rate for a year of income: ITAA 1936 s 109N(2).
//!
//! The rate is the RBA indicator rate last published before 1 July, which is
//! the May figure. The table is frozen in `data/benchmark_rates.csv`,
//! reviewed by hand and compiled into the binary. A year outside the reviewed
//! coverage is UNKNOWN, never an extrapolation.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::Failure;
use crate::csvread;
use crate::decimal::Decimal;
use crate::money::parse_rate;
use crate::py::{self, PyPath};
use crate::pyjson;
use crate::pytext;
use crate::raw::Raw;
use crate::years::{YearOfIncome, parse_year};

/// The frozen table, as reviewed.
pub const FROZEN_TABLE: &str = include_str!("../data/benchmark_rates.csv");
/// Where the frozen table lives in the source tree, as its manifest names it.
pub const FROZEN_URI: &str = "data/benchmark_rates.csv";

pub const FROZEN_ORIGIN: &str = "frozen table";
pub const OVERRIDE_ORIGIN: &str = "reviewed override";

/// A rate table a result was computed from, and the digest of the text read.
#[derive(Clone, Debug, PartialEq)]
pub struct TableSource {
    pub uri: String,
    pub sha256: String,
    pub primary_url: String,
    pub retrieved_on: String,
    pub snapshot_sha256: String,
}

impl TableSource {
    pub fn to_json(&self) -> Value {
        let mut out = Map::new();
        out.insert("uri".into(), json!(self.uri));
        out.insert("sha256".into(), json!(self.sha256));
        for (key, value) in [
            ("primary_url", &self.primary_url),
            ("retrieved_on", &self.retrieved_on),
            ("snapshot_sha256", &self.snapshot_sha256),
        ] {
            if !value.is_empty() {
                out.insert(key.into(), json!(value));
            }
        }
        Value::Object(out)
    }
}

fn digest(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// One reviewed year in the table.
#[derive(Clone, Debug)]
pub struct BenchmarkEntry {
    pub year: YearOfIncome,
    pub rate: Decimal,
    pub rba_table: String,
    pub rba_series: String,
    pub rba_month: String,
    pub source: String,
    pub verify_at: String,
    pub seen: String,
    pub origin: &'static str,
    pub primary_url: String,
    pub retrieved_on: String,
    pub snapshot_sha256: String,
}

/// The s 109N(2) benchmark rate, with enough provenance to audit it later.
#[derive(Clone, Debug)]
pub struct RateResult {
    pub known: bool,
    pub year_of_income: String,
    pub rate: Option<Decimal>,
    pub entry: Option<BenchmarkEntry>,
    pub table_reviewed_until: String,
    pub table_reviewed_on: String,
    pub reason: Option<String>,
    pub statutory_trace: Vec<String>,
    pub sources: Vec<TableSource>,
}

impl RateResult {
    pub fn verdict(&self) -> &'static str {
        if self.known { "KNOWN" } else { "UNKNOWN" }
    }

    pub fn rate_text(&self) -> Option<String> {
        self.rate.as_ref().map(Decimal::to_sci_string)
    }

    fn field(&self, get: impl Fn(&BenchmarkEntry) -> &str) -> String {
        self.entry
            .as_ref()
            .map_or(String::new(), |e| get(e).to_string())
    }

    pub fn provenance(&self) -> Value {
        json!({
            "origin": self.entry.as_ref().map_or("", |e| e.origin),
            "rba_table": self.field(|e| &e.rba_table),
            "rba_series": self.field(|e| &e.rba_series),
            "rba_month": self.field(|e| &e.rba_month),
            "source": self.field(|e| &e.source),
            "verify_at": self.field(|e| &e.verify_at),
            "primary_url": self.field(|e| &e.primary_url),
            "retrieved_on": self.field(|e| &e.retrieved_on),
            "snapshot_sha256": self.field(|e| &e.snapshot_sha256),
            "entry_seen": self.field(|e| &e.seen),
            "table_reviewed_until": self.table_reviewed_until,
            "table_reviewed_on": self.table_reviewed_on,
        })
    }

    pub fn manifest(&self) -> Value {
        json!({"rate_table_uris": self.sources.iter().map(TableSource::to_json).collect::<Vec<_>>()})
    }

    pub fn to_json(&self) -> Value {
        json!({
            "verdict": self.verdict(),
            "year_of_income": self.year_of_income,
            "benchmark_rate": self.rate_text(),
            "provenance": self.provenance(),
            "reason": self.reason,
            "statutory_trace": self.statutory_trace,
            "manifest": self.manifest(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct BenchmarkTable {
    pub entries: BTreeMap<i64, BenchmarkEntry>,
    pub reviewed_until: YearOfIncome,
    pub reviewed_on: String,
    pub sources: Vec<TableSource>,
}

impl BenchmarkTable {
    fn coverage(&self) -> String {
        let first = self.entries.keys().next().expect("a table holds rates");
        let last = self
            .entries
            .keys()
            .next_back()
            .expect("a table holds rates");
        format!(
            "{} to {}",
            YearOfIncome { start_year: *first }.label(),
            YearOfIncome { start_year: *last }.label()
        )
    }

    pub fn with_override(&self, over: &RateOverride) -> BenchmarkTable {
        let mut entries = self.entries.clone();
        for entry in &over.entries {
            entries.insert(entry.year.start_year, entry.clone());
        }
        let mut sources = self.sources.clone();
        sources.push(over.source.clone());
        BenchmarkTable {
            entries,
            reviewed_until: self.reviewed_until.max(over.verified_until),
            reviewed_on: format!("{}; override {}", self.reviewed_on, over.citation),
            sources,
        }
    }

    pub fn lookup(&self, year: YearOfIncome) -> RateResult {
        let mut trace = vec![format!(
            "ITAA 1936 s 109N(2): benchmark interest rate is the RBA Indicator Lending \
             Rates - Bank variable housing loans rate last published before the start of \
             the year of income ({}).",
            year.starts_on()
        )];
        let Some(entry) = self.entries.get(&year.start_year) else {
            return RateResult {
                known: false,
                year_of_income: year.label(),
                rate: None,
                entry: None,
                table_reviewed_until: self.reviewed_until.label(),
                table_reviewed_on: self.reviewed_on.clone(),
                reason: Some(format!(
                    "No reviewed benchmark rate for {}. The frozen table covers {}. Supply \
                     a reviewed override file (--rates-override) carrying verified_until \
                     and a citation, or treat this year as UNKNOWN. This engine does not \
                     read the RBA at runtime and does not extrapolate from an adjacent year.",
                    year.label(),
                    self.coverage()
                )),
                statutory_trace: trace,
                sources: self.sources.clone(),
            };
        };
        trace.push(format!(
            "Rate taken from RBA table {} series {}, {} figure, as reviewed on {} ({}).",
            entry.rba_table, entry.rba_series, entry.rba_month, entry.seen, entry.origin
        ));
        RateResult {
            known: true,
            year_of_income: year.label(),
            rate: Some(entry.rate.clone()),
            entry: Some(entry.clone()),
            table_reviewed_until: self.reviewed_until.label(),
            table_reviewed_on: self.reviewed_on.clone(),
            reason: None,
            statutory_trace: trace,
            sources: self.sources.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RateOverride {
    pub entries: Vec<BenchmarkEntry>,
    pub verified_until: YearOfIncome,
    pub citation: String,
    pub source: TableSource,
}

/// `str.splitlines()`.
fn splitlines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let breaks = matches!(
            c,
            '\n' | '\r'
                | '\u{b}'
                | '\u{c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if !breaks {
            continue;
        }
        lines.push(&text[start..i]);
        let mut end = i + c.len_utf8();
        if c == '\r' && chars.peek().is_some_and(|(_, n)| *n == '\n') {
            chars.next();
            end += 1;
        }
        start = end;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// A '# key: value' line out of the table's comment header.
fn header_scalar(lines: &[&str], key: &str, shown: &str) -> Result<String, Failure> {
    let prefix = format!("# {key}:");
    lines
        .iter()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map(|rest| pytext::strip(rest).to_string())
        .ok_or_else(|| {
            Failure::Value(format!(
                "{shown} is missing its {} header line",
                pytext::repr(&prefix)
            ))
        })
}

/// `_required(row, key, where)`: a non-empty string field.
fn required(row: &Raw, key: &str, where_: &str) -> Result<String, Failure> {
    let raw = row.get(key).cloned().unwrap_or(Raw::None);
    let raw = if raw == Raw::None { Raw::text("") } else { raw };
    let Raw::Str(text) = &raw else {
        return Err(Failure::Value(format!(
            "{where_} gives {} as {}. Write it as a quoted string: a JSON number is read \
             as an IEEE double and loses precision",
            pytext::repr(key),
            raw.py_repr()
        )));
    };
    let value = pytext::strip(text);
    if value.is_empty() {
        return Err(Failure::Value(format!(
            "{where_} is missing {}",
            pytext::repr(key)
        )));
    }
    Ok(value.to_string())
}

/// `(row.get(key) or default).strip()`: a non-string value raised AttributeError.
fn optional_text(row: &Raw, key: &str, default: &str) -> Result<String, Failure> {
    match row.get(key) {
        Some(raw) if raw.truthy() => match raw {
            Raw::Str(text) => Ok(pytext::strip(text).to_string()),
            other => Err(Failure::Uncaught(format!(
                "AttributeError: '{}' object has no attribute 'strip'",
                other.type_name()
            ))),
        },
        _ => Ok(pytext::strip(default).to_string()),
    }
}

/// `dict(zip(fieldnames, row))` with DictReader's `None` for a short row.
pub fn dict_row(header: &[String], row: &[String]) -> Raw {
    let mut items: Vec<(String, Raw)> = Vec::new();
    for (i, key) in header.iter().enumerate() {
        let value = row.get(i).map_or(Raw::None, |v| Raw::Str(v.clone()));
        match items.iter_mut().find(|(k, _)| k == key) {
            Some(slot) => slot.1 = value,
            None => items.push((key.clone(), value)),
        }
    }
    Raw::Dict(items)
}

fn table_from_text(text: &str, shown: &str, uri: String) -> Result<BenchmarkTable, Failure> {
    let lines = splitlines(text);
    let reviewed_until_label = header_scalar(&lines, "reviewed_until", shown)?;
    let reviewed_on = header_scalar(&lines, "reviewed_on", shown)?;
    let reviewed_until = parse_year(
        &Raw::Str(reviewed_until_label),
        &format!("{shown} reviewed_until"),
    )
    .map_err(Failure::Value)?;
    let body: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| !line.starts_with('#'))
        .collect();
    // `csv.DictReader` over the lines themselves, read and checked one row at
    // a time, so a fault in an early row is reported before a later CSV error.
    let csv_error = |e: csvread::ReadError| Failure::Uncaught(format!("_csv.Error: {e}"));
    let mut reader = csvread::Reader::from_lines(&body);
    let header = reader.next_row().map_err(csv_error)?.unwrap_or_default();
    let mut entries: BTreeMap<i64, BenchmarkEntry> = BTreeMap::new();
    let mut n = 0;
    while let Some(cells) = reader.next_row().map_err(csv_error)? {
        if cells.is_empty() {
            continue;
        }
        let row = &dict_row(&header, &cells);
        let where_ = format!("benchmark rate row {} in {shown}", n + 1);
        let year = parse_year(
            &Raw::Str(required(row, "year_of_income", &where_)?),
            &where_,
        )
        .map_err(Failure::Value)?;
        let rate = parse_rate(
            &Raw::Str(required(row, "rate", &where_)?),
            &format!("{where_} rate"),
        )
        .map_err(Failure::Value)?;
        if entries.contains_key(&year.start_year) {
            return Err(Failure::Value(format!(
                "{where_}: {} appears twice in the table",
                year.label()
            )));
        }
        let entry = BenchmarkEntry {
            year,
            rate,
            rba_table: required(row, "rba_table", &where_)?,
            rba_series: required(row, "rba_series", &where_)?,
            rba_month: required(row, "rba_month", &where_)?,
            source: required(row, "source", &where_)?,
            verify_at: optional_text(row, "verify_at", "")?,
            seen: required(row, "seen", &where_)?,
            origin: FROZEN_ORIGIN,
            primary_url: optional_text(row, "primary_url", "")?,
            retrieved_on: optional_text(row, "retrieved_on", "")?,
            snapshot_sha256: optional_text(row, "snapshot_sha256", "")?,
        };
        entries.insert(year.start_year, entry);
        n += 1;
    }
    let Some(latest) = entries.keys().next_back().copied() else {
        return Err(Failure::Value(format!("{shown} holds no benchmark rates")));
    };
    let latest = YearOfIncome { start_year: latest };
    if reviewed_until < latest {
        return Err(Failure::Value(format!(
            "{shown} claims reviewed_until {} but carries a rate for {}; re-review the \
             table and update the header",
            reviewed_until.label(),
            latest.label()
        )));
    }
    // A table-level primary-source claim holds only when every row makes it.
    let mut claims: Vec<(&str, &str, &str)> = Vec::new();
    for entry in entries.values() {
        let claim = (
            entry.primary_url.as_str(),
            entry.retrieved_on.as_str(),
            entry.snapshot_sha256.as_str(),
        );
        if !claims.contains(&claim) {
            claims.push(claim);
        }
    }
    let (primary_url, retrieved_on, snapshot_sha256) = if claims.len() == 1 {
        claims[0]
    } else {
        ("", "", "")
    };
    let source = TableSource {
        uri,
        sha256: digest(text),
        primary_url: primary_url.into(),
        retrieved_on: retrieved_on.into(),
        snapshot_sha256: snapshot_sha256.into(),
    };
    Ok(BenchmarkTable {
        entries,
        reviewed_until,
        reviewed_on,
        sources: vec![source],
    })
}

/// The frozen table compiled into the binary.
pub fn frozen_table() -> BenchmarkTable {
    let text = py::universal_newlines(FROZEN_TABLE);
    table_from_text(&text, FROZEN_URI, FROZEN_URI.into()).expect("the frozen table is reviewed")
}

/// `Path.read_text(encoding="utf-8")`: an OSError is the caller's to word; a
/// decode error was uncaught.
fn read_text(path: &PyPath) -> Result<Result<String, std::io::Error>, Failure> {
    match py::read_text(&path.fs()) {
        Ok(text) => Ok(Ok(text)),
        Err(py::ReadError::Os(e)) => Ok(Err(e)),
        Err(py::ReadError::Decode(e)) => Err(Failure::Uncaught(format!("UnicodeDecodeError: {e}"))),
    }
}

/// `load_table(path)` for a table outside the binary.
pub fn load_table(path: &PyPath) -> Result<BenchmarkTable, Failure> {
    let shown = path.display();
    let text = read_text(path)?.map_err(|e| {
        Failure::Value(format!(
            "cannot read the benchmark rate table at {shown}: {}",
            py::oserror_str(&e, &shown, py::OsStyle::Errno)
        ))
    })?;
    table_from_text(&text, &shown, format!("file:{}", path.name()))
}

/// `load_override(path)`: refused unless it carries verified_until and a citation.
pub fn load_override(path: &PyPath) -> Result<RateOverride, Failure> {
    let shown = path.display();
    let text = read_text(path)?.map_err(|e| {
        Failure::Value(format!(
            "cannot read the rate override at {shown}: {}",
            py::oserror_str(&e, &shown, py::OsStyle::Errno)
        ))
    })?;
    let doc = pyjson::loads(&text)
        .map_err(|e| Failure::Value(format!("{shown} is not valid JSON: {e}")))?;
    let doc = Raw::from_json(&doc);
    if !matches!(doc, Raw::Dict(_)) {
        return Err(Failure::Value(format!(
            "{shown} must be a JSON object with 'verified_until', 'citation' and 'rates'; \
             it holds a {}",
            doc.type_name()
        )));
    }
    let citation = match doc.get("citation") {
        Some(raw) if raw.truthy() => pytext::strip(&raw.py_str()).to_string(),
        _ => String::new(),
    };
    if citation.is_empty() {
        return Err(Failure::Value(format!(
            "{shown} has no 'citation'. Record where the rate was read from -- RBA table F5 \
             series FILRHLBVS, the month, and the date you read it -- before this engine \
             will use it"
        )));
    }
    let Some(verified_raw) = doc.get("verified_until") else {
        return Err(Failure::Value(format!(
            "{shown} has no 'verified_until'. State the last year of income the override \
             has actually been reviewed to"
        )));
    };
    let verified_until =
        parse_year(verified_raw, &format!("{shown} verified_until")).map_err(Failure::Value)?;
    let rows = match doc.get("rates") {
        Some(Raw::List(rows)) if !rows.is_empty() => rows.clone(),
        _ => {
            return Err(Failure::Value(format!(
                "{shown}: 'rates' must be a non-empty list of rate objects"
            )));
        }
    };
    let mut entries: Vec<BenchmarkEntry> = Vec::new();
    for (n, row) in rows.iter().enumerate() {
        let where_ = format!("override rate {} in {shown}", n + 1);
        if !matches!(row, Raw::Dict(_)) {
            return Err(Failure::Value(format!("{where_} is not an object")));
        }
        let year = parse_year(
            &Raw::Str(required(row, "year_of_income", &where_)?),
            &where_,
        )
        .map_err(Failure::Value)?;
        let rate = parse_rate(
            &Raw::Str(required(row, "rate", &where_)?),
            &format!("{where_} rate"),
        )
        .map_err(Failure::Value)?;
        if entries.iter().any(|e| e.year == year) {
            return Err(Failure::Value(format!(
                "{where_}: {} occurs twice",
                year.label()
            )));
        }
        if year > verified_until {
            return Err(Failure::Value(format!(
                "{where_} supplies {}, past the file's own verified_until {}. Either review \
                 that year or remove it",
                year.label(),
                verified_until.label()
            )));
        }
        entries.push(BenchmarkEntry {
            year,
            rate,
            rba_table: optional_text(row, "rba_table", "F5")?,
            rba_series: optional_text(row, "rba_series", "FILRHLBVS")?,
            rba_month: optional_text(row, "rba_month", "")?,
            source: optional_text(row, "source", &citation)?,
            verify_at: optional_text(row, "verify_at", "")?,
            seen: optional_text(row, "seen", "")?,
            origin: OVERRIDE_ORIGIN,
            primary_url: String::new(),
            retrieved_on: String::new(),
            snapshot_sha256: String::new(),
        });
    }
    Ok(RateOverride {
        entries,
        verified_until,
        citation,
        source: TableSource {
            uri: format!("override:{}", path.name()),
            sha256: digest(&text),
            primary_url: String::new(),
            retrieved_on: String::new(),
            snapshot_sha256: String::new(),
        },
    })
}

/// The s 109N(2) benchmark rate for a year of income. An unreviewed year is
/// an UNKNOWN result, not an error.
pub fn benchmark_rate(
    year: YearOfIncome,
    table: Option<&BenchmarkTable>,
    over: Option<&RateOverride>,
) -> RateResult {
    let frozen;
    let table = match table {
        Some(table) => table,
        None => {
            frozen = frozen_table();
            &frozen
        }
    };
    match over {
        Some(over) => table.with_override(over).lookup(year),
        None => table.lookup(year),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frozen_table_reads_the_may_figure() {
        let table = frozen_table();
        let result = table.lookup(YearOfIncome::new(2025).unwrap());
        assert_eq!(result.rate_text().as_deref(), Some("0.0837"));
        assert_eq!(table.reviewed_until.label(), "2026-27");
        assert_eq!(table.sources[0].uri, FROZEN_URI);
    }

    #[test]
    fn the_frozen_table_carries_its_review_and_primary_sources() {
        let table = frozen_table();
        assert_eq!(table.reviewed_on, "2026-08-28");
        assert_eq!(table.coverage(), "2019-20 to 2026-27");
        let hosts = ["rba.gov.au", "ato.gov.au", "legislation.gov.au"];
        for entry in table.entries.values() {
            let rest = entry.primary_url.strip_prefix("https://").unwrap();
            let host = rest.split('/').next().unwrap();
            assert!(
                hosts
                    .iter()
                    .any(|h| host == *h || host.ends_with(&format!(".{h}"))),
                "{} cites {host}",
                entry.year.label()
            );
            assert!(!entry.retrieved_on.is_empty());
            assert_eq!(entry.snapshot_sha256.len(), 64);
        }
    }

    #[test]
    fn splitlines_matches_python() {
        assert_eq!(
            splitlines("a\nb\r\nc\rd\u{2028}e\n"),
            ["a", "b", "c", "d", "e"]
        );
        assert_eq!(splitlines(""), Vec::<&str>::new());
        assert_eq!(splitlines("a\n\nb"), ["a", "", "b"]);
    }
}
