//! CSV reading with loud failures.
//!
//! Payroll exports vary, so column names are mapped explicitly. Nothing is
//! guessed and nothing is coerced silently: a value the parser cannot read
//! raises with its row number rather than defaulting to zero or today.

use std::collections::BTreeSet;

use crate::csvread::{self, Reader};
use crate::dates;
use crate::deadlines::{ContribEvent, ContribLine};
use crate::decimal::{Decimal, Rounding};
use crate::py::Date;
use crate::pyjson;
use crate::pyre::Regex;
use crate::pytext;
use crate::raw::Raw;
use crate::returned;
use crate::{Failure, OsError, Result, repr_list, sorted_strings};

/// Characters Excel and Sheets can evaluate at the start of a cell.
pub const FORMULA_LEAD: [char; 4] = ['=', '+', '-', '@'];

fn csv_error(message: String) -> Failure {
    Failure::Value(message)
}

/// `money(value)`: cents, ROUND_HALF_UP, as `str()`; blank for None.
pub fn money(value: Option<&Decimal>) -> Result<String> {
    Ok(match value {
        None => String::new(),
        Some(v) => v.quantize(-2, Rounding::HalfUp)?.to_sci_string(),
    })
}

/// `cents(value)`: zero for None.
pub fn cents(value: Option<&Decimal>) -> Result<Decimal> {
    Ok(match value {
        None => Decimal::from_int(0),
        Some(v) => v.quantize(-2, Rounding::HalfUp)?,
    })
}

/// `min(a, b)`: the first argument unless the second is smaller.
pub fn dmin(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.lt(a)? { b.clone() } else { a.clone() })
}

/// `max(a, b)`: the first argument unless the second is larger.
pub fn dmax(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.gt(a)? { b.clone() } else { a.clone() })
}

pub fn zero() -> Decimal {
    Decimal::from_int(0)
}

/// How much of `sg_amount` is evidenced as remitted on this as-at date.
pub fn remitted_credit(line: &ContribLine, remitted_as_at: Option<Date>) -> Result<Decimal> {
    let owed = cents(Some(&line.sg_amount))?;
    if let Some(amount) = &line.remitted_amount {
        let credited = cents(Some(amount))?;
        if line.remitted.is_some() && remitted_as_at.is_some() {
            return dmin(&credited, &owed);
        }
        return Ok(zero());
    }
    if remitted_as_at.is_some() {
        return Ok(owed);
    }
    Ok(zero())
}

/// Stop a spreadsheet treating a cell as a formula.
pub fn csv_safe(text: &str) -> String {
    let stripped = text.trim_start_matches(|c: char| pytext::is_space(c as u32));
    if stripped.starts_with(FORMULA_LEAD) {
        format!("'{text}")
    } else {
        text.to_string()
    }
}

/// The canonical fields, in order, and whether each is required.
pub const CANONICAL: [(&str, bool); 13] = [
    ("employee_id", true),
    ("qe_day", true),
    ("sg_amount", true),
    ("remitted", false),
    ("received", false),
    ("first_to_fund", false),
    ("out_of_cycle", false),
    ("next_standard_qe_day", false),
    ("db_interest", false),
    ("remitted_amount", false),
    ("matched_amount", false),
    ("join_caveats", false),
    ("returned_by_fund", false),
];

pub const DEFAULT_MAPPING: [(&str, &str); 13] = [
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
    ("returned_by_fund", "returned_by_fund"),
];

pub const TRUE_WORDS: [&str; 5] = ["y", "yes", "true", "1", "t"];
pub const FALSE_WORDS: [&str; 6] = ["", "n", "no", "false", "0", "f"];

thread_local! {
    static PATTERNS: Patterns = Patterns::new();
}

struct Patterns {
    amount_text: Regex,
    fraction_overflow: Regex,
    fraction_pad: Regex,
    iso_shape: Regex,
    iso_offset_shape: Regex,
}

impl Patterns {
    fn new() -> Patterns {
        Patterns {
            amount_text: Regex::new(
                r"^[-+]?(?:\d{1,3}(?:[ ,]\d{3})+|\d+)(?:\.\d*)?(?:[eE][-+]?\d+)?$|^[-+]?\.\d+(?:[eE][-+]?\d+)?$",
            ),
            fraction_overflow: Regex::new(r"(:\d{2}\.\d{6})\d+"),
            fraction_pad: Regex::new(r"(:\d{2})\.(\d{1,5})(?!\d)"),
            iso_shape: Regex::new(
                r"\d{4}-\d{2}-\d{2}(?:[T ]\d{2}:\d{2}(?::\d{2}(?:\.\d{1,6})?)?)?$",
            ),
            iso_offset_shape: Regex::new(
                r"\d{4}-\d{2}-\d{2}[T ]\d{2}(?::\d{2}(?::\d{2}(?:\.\d{1,6})?)?)?(?:[Zz]|[+-]\d{2}:\d{2}(?::\d{2})?)$",
            ),
        }
    }
}

/// The column mapping, field by field in `DEFAULT_MAPPING` order with any
/// user fields after, and the fields the user mapped explicitly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mapping {
    pub columns: Vec<(String, String)>,
    pub explicit: BTreeSet<String>,
}

impl Mapping {
    pub fn default_mapping() -> Mapping {
        Mapping {
            columns: DEFAULT_MAPPING
                .iter()
                .map(|(f, c)| (f.to_string(), c.to_string()))
                .collect(),
            explicit: BTreeSet::new(),
        }
    }

    /// The column a field maps to; "" for a field the mapping lacks, which
    /// the reader refuses before it reads one.
    pub fn get(&self, field: &str) -> &str {
        self.columns
            .iter()
            .find(|(f, _)| f == field)
            .map_or("", |(_, c)| c.as_str())
    }

    pub fn has(&self, field: &str) -> bool {
        self.columns.iter().any(|(f, _)| f == field)
    }

    fn set(&mut self, field: &str, column: &str) {
        match self.columns.iter_mut().find(|(f, _)| f == field) {
            Some(entry) => entry.1 = column.to_string(),
            None => self.columns.push((field.to_string(), column.to_string())),
        }
    }
}

fn canonical_sorted() -> String {
    repr_list(&sorted_strings(CANONICAL.iter().map(|(f, _)| *f)))
}

fn is_canonical(field: &str) -> bool {
    CANONICAL.iter().any(|(f, _)| *f == field)
}

/// Read a text file as `open(path, encoding="utf-8").read()` does.
pub fn read_text(path: &str) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|e| os_failure(&e, path))?;
    let text = crate::py::decode_utf8(&bytes).map_err(|e| Failure::Value(e.to_string()))?;
    Ok(crate::py::universal_newlines(&text))
}

/// `json.load(f)` of a text file, the decode error as `on_decode` words it.
pub fn load_json(path: &str, on_decode: impl FnOnce(String) -> String) -> Result<Raw> {
    let text = read_text(path)?;
    match pyjson::loads(&text) {
        Ok(value) => Ok(Raw::from_json(&value)),
        // The integer limit is a ValueError of its own, not a JSONDecodeError.
        Err(e) if e.0.starts_with("Exceeds the limit") => Err(Failure::Value(e.0)),
        Err(e) => Err(Failure::Value(on_decode(e.to_string()))),
    }
}

/// `load_mapping(path, overrides)`.
pub fn load_mapping(path: Option<&str>, overrides: &[String]) -> Result<Mapping> {
    let mut mapping = Mapping::default_mapping();
    if let Some(path) = path {
        let user = load_json(path, |e| format!("{path} is not valid JSON: {e}"))?;
        let Raw::Dict(entries) = user else {
            return Err(csv_error(format!(
                "{path} must be a JSON object of field: column pairs"
            )));
        };
        let user: Vec<(String, Raw)> = entries
            .into_iter()
            .filter(|(k, _)| !k.starts_with('_'))
            .collect();
        let unknown: Vec<&str> = user
            .iter()
            .map(|(k, _)| k.as_str())
            .filter(|k| !is_canonical(k))
            .collect();
        if !unknown.is_empty() {
            return Err(csv_error(format!(
                "mapping file has unknown fields: {}; valid fields are {}",
                repr_list(&sorted_strings(unknown)),
                canonical_sorted()
            )));
        }
        let bad: Vec<&str> = user
            .iter()
            .filter(|(_, v)| match v {
                Raw::Str(s) => pytext::strip(s).is_empty(),
                _ => true,
            })
            .map(|(k, _)| k.as_str())
            .collect();
        if !bad.is_empty() {
            return Err(csv_error(format!(
                "{path} maps {} to something that is not a column name. Each value \
                 must be the column heading as it appears in your CSV.",
                repr_list(&sorted_strings(bad))
            )));
        }
        for (field, value) in &user {
            if let Raw::Str(column) = value {
                mapping.set(field, column);
                mapping.explicit.insert(field.clone());
            }
        }
    }
    for item in overrides {
        let Some((field, column)) = item.split_once('=') else {
            return Err(csv_error(format!(
                "--map expects field=column, got {}",
                pytext::repr_os(item)
            )));
        };
        if !is_canonical(field) {
            return Err(csv_error(format!(
                "--map field {} is not one of {}",
                pytext::repr_os(field),
                canonical_sorted()
            )));
        }
        mapping.set(field, column);
        mapping.explicit.insert(field.to_string());
    }
    Ok(mapping)
}

pub const DATE_FORMATS: [&str; 6] = [
    "%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%d/%m/%y", "%d %b %Y", "%d %B %Y",
];
pub const TIME_FORMATS: [&str; 5] = [
    "%H:%M",
    "%H:%M:%S",
    "%H:%M:%S.%f",
    "%I:%M %p",
    "%I:%M:%S %p",
];

/// No payroll date is beyond this.
pub const LATEST_SANE_YEAR: i32 = 2200;

/// Read a date in any format this tool accepts, or None. A date-time with a
/// zone marker is refused.
pub fn parse_date_text(text: &str) -> Result<Option<Date>> {
    let text = pytext::strip(text);
    if text.is_empty() {
        return Ok(None);
    }
    let (offset, iso) = PATTERNS.with(|p| {
        let iso_text = p
            .fraction_overflow
            .sub_with(text, |m| m.group(1).unwrap_or_default());
        let iso_text = p.fraction_pad.sub_with(&iso_text, |m| {
            let digits = m.group(2).unwrap_or_default();
            let pad = 6usize.saturating_sub(pytext::py_len(&digits));
            format!(
                "{}.{digits}{}",
                m.group(1).unwrap_or_default(),
                "0".repeat(pad)
            )
        });
        if p.iso_offset_shape.matches_start(&iso_text) {
            return (true, None);
        }
        let iso = if p.iso_shape.matches_start(&iso_text) {
            dates::fromisoformat_date(&iso_text)
        } else {
            None
        };
        (false, iso)
    });
    if offset {
        return Err(csv_error(format!(
            "value {} carries a UTC or timezone offset marker, and its \
             as-written calendar day belongs to that zone, not necessarily to \
             the Australian day the law tests -- 2026-07-21T20:00:00Z is \
             already 22 July 2026 in AEST, so keeping the written day would \
             read a receipt one day early. Convert it to the Australian local \
             calendar date and supply that instead",
            pytext::repr_os(text)
        )));
    }
    if iso.is_some() {
        return Ok(iso);
    }
    for fmt in DATE_FORMATS {
        if let Some(date) = dates::strptime_date(text, fmt) {
            return Ok(Some(date));
        }
        for time_fmt in TIME_FORMATS {
            if let Some(date) = dates::strptime_date(text, &format!("{fmt} {time_fmt}")) {
                return Ok(Some(date));
            }
        }
    }
    Ok(None)
}

fn parse_date(value: &str, field: &str, row: i64) -> Result<Date> {
    let parsed = match parse_date_text(value) {
        Ok(parsed) => parsed,
        Err(e) => return Err(csv_error(format!("row {row}: {field} {}", e.message()))),
    };
    let Some(parsed) = parsed else {
        return Err(csv_error(format!(
            "row {row}: cannot read {field} value {}: use YYYY-MM-DD or \
             DD/MM/YYYY (day first, Australian order)",
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
    Ok(parsed)
}

/// Read one amount cell, to the cent: the one reader for both the checker's
/// CSV and the importer's vendor exports.
pub fn parse_amount(value: &str, field: &str, row: i64, empty_is_error: bool) -> Result<Decimal> {
    let mut text = pytext::strip(&pytext::strip(value).replace('$', "")).to_string();
    if text.starts_with('(') && text.ends_with(')') {
        let inner: Vec<char> = text.chars().collect();
        let body: String = if inner.len() >= 2 {
            inner[1..inner.len() - 1].iter().collect()
        } else {
            String::new()
        };
        text = format!("-{}", pytext::strip(&body));
    }
    if empty_is_error && text.is_empty() {
        return Err(csv_error(format!("row {row}: {field} is empty")));
    }
    let shown = pytext::repr(value);
    let unreadable = || {
        csv_error(format!(
            "row {row}: cannot read {field} value {shown} as an amount"
        ))
    };
    let loose = text.replace([',', ' '], "");
    let amount = Decimal::parse(&loose).map_err(|_| unreadable())?;
    if !amount.is_finite() {
        return Err(unreadable());
    }
    if amount.adjusted() > 15 {
        return Err(csv_error(format!(
            "row {row}: {field} value {shown} is too large to be a real amount"
        )));
    }
    if !PATTERNS.with(|p| p.amount_text.matches_start(&text)) {
        return Err(csv_error(format!(
            "row {row}: cannot read {field} value {shown} as an amount. A comma or \
             space is only read as a thousands separator, so 612,00 is refused rather \
             than read as 61200."
        )));
    }
    if amount.lt(&zero())? {
        return Err(csv_error(format!(
            "row {row}: {field} is negative ({shown})"
        )));
    }
    let rounded = cents(Some(&amount))?;
    if rounded.is_zero() && !amount.is_zero() {
        return Err(csv_error(format!(
            "row {row}: {field} value {shown} is under half a cent, so reading it \
             to the cent leaves the row carrying no money at all. Every figure this \
             tool matches, writes and reports is a cent figure. Round it yourself, or \
             take the row out."
        )));
    }
    if rounded.is_zero() {
        return Ok(Decimal::parse("0.00").expect("a literal"));
    }
    Ok(rounded)
}

fn parse_bool(value: &str, field: &str, row: i64) -> Result<bool> {
    let text = pytext::strip(value);
    // `str.lower()` of a non-ASCII text never gives one of these ASCII words.
    let lowered = text.is_ascii().then(|| text.to_ascii_lowercase());
    match lowered.as_deref() {
        Some(t) if TRUE_WORDS.contains(&t) => Ok(true),
        Some(t) if FALSE_WORDS.contains(&t) => Ok(false),
        _ => Err(csv_error(format!(
            "row {row}: cannot read {field} value {} as yes/no",
            pytext::repr(value)
        ))),
    }
}

/// A `DictReader` cell: a value, the `restval` sentinel for a column the row
/// never supplied, or the surplus values under the `None` key.
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Value(String),
    Missing,
    Surplus(Vec<String>),
}

/// One `DictReader` row: keys in insertion order, `None` for the rest key.
pub type DictRow = Vec<(Option<String>, Cell)>;

fn dict_set(row: &mut DictRow, key: Option<String>, cell: Cell) {
    match row.iter_mut().find(|(k, _)| *k == key) {
        Some(entry) => entry.1 = cell,
        None => row.push((key, cell)),
    }
}

/// `csv.DictReader(open(path, newline="", encoding="utf-8-sig"), restval=MISSING)`.
pub struct DictReader<'a> {
    reader: Reader<'a>,
    fieldnames: Option<Vec<String>>,
}

/// Why the file could not be read as CSV.
pub enum ReadFailure {
    /// UnicodeDecodeError: the byte and its position within the decode call.
    Decode {
        byte: u8,
        start: usize,
    },
    Csv(String),
}

impl From<csvread::ReadError> for ReadFailure {
    fn from(e: csvread::ReadError) -> ReadFailure {
        match e {
            csvread::ReadError::Decode { start, byte, .. } => ReadFailure::Decode { byte, start },
            csvread::ReadError::Csv(m) => ReadFailure::Csv(m),
        }
    }
}

impl<'a> DictReader<'a> {
    /// Reads the header row, as the `fieldnames` property does.
    pub fn new(data: &'a [u8]) -> std::result::Result<DictReader<'a>, ReadFailure> {
        let mut reader = Reader::new(data);
        let fieldnames = reader.next_row()?;
        Ok(DictReader { reader, fieldnames })
    }

    pub fn fieldnames(&self) -> Option<&[String]> {
        self.fieldnames.as_deref()
    }

    /// `next(reader)`, or None at the end.
    pub fn next_row(&mut self) -> std::result::Result<Option<DictRow>, ReadFailure> {
        let names = self.fieldnames.clone().unwrap_or_default();
        let row = loop {
            match self.reader.next_row()? {
                None => return Ok(None),
                Some(row) if row.is_empty() => continue,
                Some(row) => break row,
            }
        };
        let mut out: DictRow = Vec::new();
        for (name, value) in names.iter().zip(&row) {
            dict_set(&mut out, Some(name.clone()), Cell::Value(value.clone()));
        }
        if names.len() < row.len() {
            dict_set(&mut out, None, Cell::Surplus(row[names.len()..].to_vec()));
        } else {
            for name in &names[row.len()..] {
                dict_set(&mut out, Some(name.clone()), Cell::Missing);
            }
        }
        Ok(Some(out))
    }
}

/// The message for a file that is not UTF-8.
pub fn not_utf8(path: &str, byte: u8, start: usize) -> Failure {
    csv_error(format!(
        "{path} is not UTF-8 text (byte {byte:#04x} at position {start}). Excel's plain \
         'CSV' export uses the Windows code page: re-save it as 'CSV UTF-8 (Comma \
         delimited)' and run again."
    ))
}

/// A `csv.Error` Python left uncaught.
pub fn csv_module_error(message: &str) -> Failure {
    Failure::Uncaught(format!("_csv.Error: {message}"))
}

/// How many row problems an error message lists in full.
pub const MAX_PROBLEMS_SHOWN: usize = 20;

/// The problem with a row whose field count does not match the header.
pub fn malformed_row_problem(row: &DictRow, i: i64) -> Option<String> {
    let short = sorted_strings(row.iter().filter_map(|(k, v)| match (k, v) {
        (Some(k), Cell::Missing) if !k.is_empty() => Some(k.clone()),
        _ => None,
    }));
    if !short.is_empty() {
        return Some(format!(
            "row {i} stops early and supplies no value for {}. A truncated \
             row is not the same as a blank field, so it is not assumed empty.",
            repr_list(&short)
        ));
    }
    let surplus: Vec<String> = row
        .iter()
        .find(|(k, _)| k.is_none())
        .map(|(_, v)| match v {
            Cell::Surplus(values) => values
                .iter()
                .filter(|v| !pytext::strip(v).is_empty())
                .cloned()
                .collect(),
            _ => Vec::new(),
        })
        .unwrap_or_default();
    if !surplus.is_empty() {
        return Some(format!(
            "row {i} carries more values than the header has columns: \
             {}. They would be dropped, so the row is refused instead.",
            repr_list(&surplus)
        ));
    }
    None
}

/// The collected row problems as one error.
pub fn problems_error(problems: &[String], path: &str) -> Failure {
    let shown = &problems[..problems.len().min(MAX_PROBLEMS_SHOWN)];
    let more = if problems.len() > MAX_PROBLEMS_SHOWN {
        format!(
            " ... and {} more problem(s).",
            problems.len() - MAX_PROBLEMS_SHOWN
        )
    } else {
        String::new()
    };
    csv_error(format!(
        "{} problem(s) in {path}:\n  - {}{more}",
        problems.len(),
        shown.join("\n  - ")
    ))
}

/// Read a file's bytes as `open()` would before any decoding.
pub fn read_bytes(path: &str) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| os_failure(&e, path))
}

/// `parse_rows(path, mapping, explicit)`.
pub fn parse_rows(path: &str, mapping: &Mapping) -> Result<Vec<ContribLine>> {
    let data = read_bytes(path)?;
    match parse_rows_bytes(path, &data, mapping) {
        Err(Failed::Read(ReadFailure::Decode { byte, start })) => Err(not_utf8(path, byte, start)),
        Err(Failed::Read(ReadFailure::Csv(m))) => Err(csv_module_error(&m)),
        Err(Failed::Engine(f)) => Err(f),
        Ok(lines) => Ok(lines),
    }
}

enum Failed {
    Read(ReadFailure),
    Engine(Failure),
}

impl From<ReadFailure> for Failed {
    fn from(e: ReadFailure) -> Failed {
        Failed::Read(e)
    }
}

impl From<Failure> for Failed {
    fn from(e: Failure) -> Failed {
        Failed::Engine(e)
    }
}

fn parse_rows_bytes(
    path: &str,
    data: &[u8],
    mapping: &Mapping,
) -> std::result::Result<Vec<ContribLine>, Failed> {
    let mut reader = DictReader::new(data)?;
    let Some(fieldnames) = reader.fieldnames().map(<[String]>::to_vec) else {
        return Err(csv_error(format!("{path} has no header row")).into());
    };
    let named: Vec<String> = fieldnames
        .iter()
        .map(|h| pytext::strip(h).to_string())
        .filter(|h| !h.is_empty())
        .collect();
    let duplicates = sorted_strings(
        named
            .iter()
            .filter(|h| named.iter().filter(|n| n == h).count() > 1)
            .collect::<BTreeSet<_>>(),
    );
    if !duplicates.is_empty() {
        return Err(csv_error(format!(
            "{path} has duplicate column name(s): {}. Only the last \
             one would be read, so the figures cannot be trusted. Rename them.",
            repr_list(&duplicates)
        ))
        .into());
    }
    let headers: BTreeSet<&str> = named.iter().map(String::as_str).collect();
    let found = repr_list(&sorted_strings(headers.iter()));
    // A mapping without a canonical field raised KeyError where it was read.
    let key_error = |field: &str| Failure::Uncaught(format!("KeyError: {}", pytext::repr(field)));
    for (field, required) in CANONICAL {
        if required && !mapping.has(field) {
            return Err(key_error(field).into());
        }
    }
    let missing: Vec<&str> = CANONICAL
        .iter()
        .filter(|(field, required)| *required && !headers.contains(mapping.get(field)))
        .map(|(field, _)| mapping.get(field))
        .collect();
    if !missing.is_empty() {
        return Err(csv_error(format!(
            "{path} is missing required column(s): {}. Columns found: \
             {found}. Map your own names with --map field=column. The \
             amount column must hold super guarantee only, not salary sacrifice or \
             additional contributions.",
            repr_list(&missing)
        ))
        .into());
    }
    // Python looked each explicit field up while listing the mismapped ones.
    for field in &mapping.explicit {
        if !mapping.has(field) {
            return Err(key_error(field).into());
        }
    }
    let mismapped = sorted_strings(
        mapping
            .explicit
            .iter()
            .filter(|field| !headers.contains(mapping.get(field)))
            .map(|field| format!("{field} -> {}", mapping.get(field))),
    );
    if !mismapped.is_empty() {
        return Err(csv_error(format!(
            "mapped column(s) not found in {path}: {}. Columns found: \
             {found}. Remove the mapping for any field your export \
             does not have.",
            repr_list(&mismapped)
        ))
        .into());
    }

    // A return column the mapping cannot read would turn a returned contribution
    // back into an ordinary one, so it stops the file. A file without it reads
    // as no return, which is the only case the default covers.
    let unread: Vec<&String> = named
        .iter()
        .filter(|h| {
            h.chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .eq("returnedbyfund".chars())
                && !(mapping.has("returned_by_fund") && mapping.get("returned_by_fund") == *h)
        })
        .collect();
    if !unread.is_empty() {
        return Err(csv_error(format!(
            "{path} has a return column the mapping does not read: {}. Ignoring it could \
             read a returned contribution as received. Map it with \
             --map returned_by_fund=COLUMN, or remove it.",
            repr_list(&sorted_strings(unread))
        ))
        .into());
    }

    let mut lines = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    let mut i: i64 = 1;
    while let Some(raw) = reader.next_row()? {
        i += 1;
        let mut row: DictRow = Vec::new();
        for (k, v) in raw {
            let key = k.map(|k| pytext::strip(&k).to_string());
            dict_set(&mut row, key, v);
        }
        if let Some(problem) = malformed_row_problem(&row, i) {
            problems.push(problem);
            continue;
        }
        let cell = |column: &str| -> String {
            row.iter()
                .find(|(k, _)| k.as_deref() == Some(column))
                .map(|(_, v)| match v {
                    Cell::Value(s) => s.clone(),
                    _ => String::new(),
                })
                .unwrap_or_default()
        };
        // An optional field the mapping lacks raises KeyError where it is read.
        let optional = |field: &str| -> Result<String> {
            if mapping.has(field) {
                Ok(cell(mapping.get(field)))
            } else {
                Err(key_error(field))
            }
        };
        let employee = pytext::strip(&cell(mapping.get("employee_id"))).to_string();
        if employee.is_empty() {
            problems.push(format!("row {i}: employee_id is empty"));
            continue;
        }
        match read_line(
            i,
            employee,
            &cell(mapping.get("sg_amount")),
            &cell(mapping.get("qe_day")),
            &optional,
        ) {
            Ok(line) => lines.push(line),
            Err(Failure::Value(m)) => problems.push(m),
            Err(other) => return Err(other.into()),
        }
    }
    if !problems.is_empty() {
        return Err(problems_error(&problems, path).into());
    }
    if lines.is_empty() {
        return Err(csv_error(format!("{path} has a header but no data rows")).into());
    }
    Ok(lines)
}

fn read_line(
    i: i64,
    employee: String,
    sg_raw: &str,
    qe_raw: &str,
    optional: &dyn Fn(&str) -> Result<String>,
) -> Result<ContribLine> {
    let remitted_raw = pytext::strip(&optional("remitted")?).to_string();
    let received_raw = pytext::strip(&optional("received")?).to_string();
    let next_raw = pytext::strip(&optional("next_standard_qe_day")?).to_string();
    let remitted_amount_raw = pytext::strip(&optional("remitted_amount")?).to_string();
    let matched_amount_raw = pytext::strip(&optional("matched_amount")?).to_string();
    let join_caveats_raw = pytext::strip(&optional("join_caveats")?).to_string();
    let join_caveats: Vec<String> = join_caveats_raw
        .split(" | ")
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();

    let sg_amount = parse_amount(sg_raw, "sg_amount", i, false)?;
    let remitted_amount = if remitted_amount_raw.is_empty() {
        None
    } else {
        Some(parse_amount(
            &remitted_amount_raw,
            "remitted_amount",
            i,
            false,
        )?)
    };
    let matched_amount = if matched_amount_raw.is_empty() {
        None
    } else {
        Some(parse_amount(
            &matched_amount_raw,
            "matched_amount",
            i,
            false,
        )?)
    };
    if remitted_amount.is_some() && remitted_raw.is_empty() {
        return Err(csv_error(format!(
            "row {i}: remitted_amount requires remitted_date so an \
             as-at report cannot credit the payment before it occurred"
        )));
    }
    if let Some(r) = &remitted_amount
        && r.gt(&sg_amount)?
    {
        return Err(csv_error(format!(
            "row {i}: remitted_amount {} is greater than sg_amount {}",
            money(Some(r))?,
            money(Some(&sg_amount))?
        )));
    }
    if let Some(m) = &matched_amount
        && m.gt(&sg_amount)?
    {
        return Err(csv_error(format!(
            "row {i}: matched_amount {} is greater than sg_amount {}",
            money(Some(m))?,
            money(Some(&sg_amount))?
        )));
    }
    if let (Some(m), Some(r)) = (&matched_amount, &remitted_amount)
        && r.gt(m)?
    {
        return Err(csv_error(format!(
            "row {i}: remitted_amount {} is greater than matched_amount {}",
            money(Some(r))?,
            money(Some(m))?
        )));
    }
    if let Some(m) = &matched_amount
        && m.lt(&sg_amount)?
        && !remitted_raw.is_empty()
        && remitted_amount.is_none()
    {
        return Err(csv_error(format!(
            "row {i}: matched_amount below sg_amount requires \
             remitted_amount when remitted_date is present; otherwise \
             the legacy blank-amount fallback would treat the whole \
             liability as remitted"
        )));
    }
    let qe_day = parse_date(qe_raw, "qe_day", i)?;
    let remitted = if remitted_raw.is_empty() {
        None
    } else {
        Some(parse_date(&remitted_raw, "remitted", i)?)
    };
    let received = if received_raw.is_empty() {
        None
    } else {
        Some(parse_date(&received_raw, "received", i)?)
    };
    let first_to_fund = parse_bool(&optional("first_to_fund")?, "first_to_fund", i)?;
    let out_of_cycle = parse_bool(&optional("out_of_cycle")?, "out_of_cycle", i)?;
    let next_standard_qe_day = if next_raw.is_empty() {
        None
    } else {
        Some(parse_date(&next_raw, "next_standard_qe_day", i)?)
    };
    let db_interest = parse_bool(&optional("db_interest")?, "db_interest", i)?;
    // Mappings saved before this field exist. `parse_rows_bytes` has already
    // refused a file that carries the column such a mapping cannot read.
    let returned_by_fund = parse_bool(
        &optional("returned_by_fund").unwrap_or_default(),
        "returned_by_fund",
        i,
    )?;
    Ok(ContribLine {
        employee_id: employee,
        qe_day,
        sg_amount,
        remitted,
        remitted_amount,
        received,
        first_to_fund,
        out_of_cycle,
        next_standard_qe_day,
        db_interest,
        row: i,
        duplicate_note: String::new(),
        matched_amount,
        join_caveats,
        returned_by_fund,
        events: Vec::new(),
        return_not_shown: false,
    })
}

/// The event file's columns, each with whether a row must fill it.
const EVENT_COLUMNS: [(&str, bool); 7] = [
    ("employee_id", true),
    ("payment_date", true),
    ("event", true),
    ("event_date", true),
    ("amount", true),
    ("receipt_date", false),
    ("notified_date", false),
];

/// Read the events file and attach each event to the one line it names. An
/// event that names no line, or more than one, is refused rather than dropped.
pub fn attach_events(path: &str, lines: &mut [ContribLine]) -> Result<()> {
    let data = read_bytes(path)?;
    match attach_events_bytes(path, &data, lines) {
        Err(Failed::Read(ReadFailure::Decode { byte, start })) => Err(not_utf8(path, byte, start)),
        Err(Failed::Read(ReadFailure::Csv(m))) => Err(csv_module_error(&m)),
        Err(Failed::Engine(f)) => Err(f),
        Ok(()) => Ok(()),
    }
}

fn attach_events_bytes(
    path: &str,
    data: &[u8],
    lines: &mut [ContribLine],
) -> std::result::Result<(), Failed> {
    let mut reader = DictReader::new(data)?;
    let Some(fieldnames) = reader.fieldnames().map(<[String]>::to_vec) else {
        return Err(csv_error(format!("{path} has no header row")).into());
    };
    let named: Vec<String> = fieldnames
        .iter()
        .map(|h| pytext::strip(h).to_string())
        .collect();
    let known = |h: &str| EVENT_COLUMNS.iter().any(|(c, _)| *c == h);
    let mut bad: Vec<String> = named.iter().filter(|h| !known(h)).cloned().collect();
    for (i, h) in named.iter().enumerate() {
        if named[..i].contains(h) {
            bad.push(h.clone());
        }
    }
    let absent: Vec<&str> = EVENT_COLUMNS
        .iter()
        .filter(|(c, required)| *required && !named.iter().any(|h| h == c))
        .map(|(c, _)| *c)
        .collect();
    if !bad.is_empty() || !absent.is_empty() {
        return Err(csv_error(format!(
            "{path} needs the columns {} with optional {}. Unknown or repeated: {}; \
             missing: {}",
            repr_list(&[
                "employee_id",
                "payment_date",
                "event",
                "event_date",
                "amount"
            ]),
            repr_list(&["receipt_date", "notified_date"]),
            repr_list(&bad),
            repr_list(&absent)
        ))
        .into());
    }
    let mut problems: Vec<String> = Vec::new();
    let mut i: i64 = 1;
    let mut any = false;
    while let Some(raw) = reader.next_row()? {
        i += 1;
        any = true;
        let mut row: DictRow = Vec::new();
        for (k, v) in raw {
            dict_set(&mut row, k.map(|k| pytext::strip(&k).to_string()), v);
        }
        if let Some(problem) = malformed_row_problem(&row, i) {
            problems.push(problem);
            continue;
        }
        match read_event(&row, i) {
            Err(Failure::Value(m)) => problems.push(m),
            Err(other) => return Err(other.into()),
            Ok((employee, qe_day, event)) => {
                let matching: Vec<usize> = (0..lines.len())
                    .filter(|&l| lines[l].employee_id == employee && lines[l].qe_day == qe_day)
                    .collect();
                match matching[..] {
                    [only] => lines[only].events.push(event),
                    [] => problems.push(format!(
                        "event row {i}: no contribution row has employee_id {} and payment \
                         date {}, and an event is never dropped",
                        pytext::repr(&employee),
                        qe_day.isoformat()
                    )),
                    _ => problems.push(format!(
                        "event row {i}: {} contribution rows have employee_id {} and payment \
                         date {}, so the event cannot be applied to one of them",
                        matching.len(),
                        pytext::repr(&employee),
                        qe_day.isoformat()
                    )),
                }
            }
        }
    }
    if !problems.is_empty() {
        return Err(problems_error(&problems, path).into());
    }
    if !any {
        return Err(csv_error(format!("{path} has a header but no data rows")).into());
    }
    Ok(())
}

fn read_event(row: &DictRow, i: i64) -> Result<(String, Date, ContribEvent)> {
    let cell = |column: &str| -> String {
        row.iter()
            .find(|(k, _)| k.as_deref() == Some(column))
            .map(|(_, v)| match v {
                Cell::Value(s) => pytext::strip(s).to_string(),
                _ => String::new(),
            })
            .unwrap_or_default()
    };
    let at = format!("event row {i}");
    let required = |column: &str| -> Result<String> {
        let value = cell(column);
        if value.is_empty() {
            return Err(csv_error(format!("{at}: {column} is empty")));
        }
        Ok(value)
    };
    let employee = required("employee_id")?;
    let qe_day = parse_date(&required("payment_date")?, "payment_date", i)?;
    let kind_text = required("event")?;
    let date = parse_date(&required("event_date")?, "event_date", i)?;
    let amount = parse_amount(&required("amount")?, "amount", i, true)?;
    let optional_date = |column: &str| -> Result<Option<Date>> {
        let value = cell(column);
        if value.is_empty() {
            Ok(None)
        } else {
            parse_date(&value, column, i).map(Some)
        }
    };
    let receipt = optional_date("receipt_date")?;
    let notified = optional_date("notified_date")?;
    let kind = returned::event_kind(&kind_text, receipt, notified)
        .map_err(|m| csv_error(format!("{at}: {m}")))?;
    Ok((
        employee,
        qe_day,
        ContribEvent {
            kind,
            date,
            amount,
            row: i,
        },
    ))
}

/// An `OSError` from reading `path`.
pub fn os_failure(e: &std::io::Error, path: &str) -> Failure {
    Failure::Os(OsError::from_io(e, path, crate::py::OsStyle::Errno))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> Option<String> {
        parse_date_text(text).unwrap().map(|d| d.isoformat())
    }

    #[test]
    fn dates_read_as_python_reads_them() {
        assert_eq!(date("2026-07-09").as_deref(), Some("2026-07-09"));
        assert_eq!(date(" 9 Jul 2026 ").as_deref(), Some("2026-07-09"));
        assert_eq!(
            date("2026-07-09T00:00:00.0000000").as_deref(),
            Some("2026-07-09")
        );
        assert_eq!(date("2026-07-09 12:30:00.5").as_deref(), Some("2026-07-09"));
        assert_eq!(date("2026-07-31T24:00").as_deref(), Some("2026-08-01"));
        assert_eq!(date("09/07/2026 2:30 pm").as_deref(), Some("2026-07-09"));
        assert_eq!(date("2026-07"), None);
        assert_eq!(date("20260709"), None);
        assert_eq!(date(""), None);
        assert!(parse_date_text("2026-07-21T20:00:00Z").is_err());
        assert!(parse_date_text("2026-07-21 20:00+10:00").is_err());
    }

    #[test]
    fn amounts_read_to_the_cent() {
        let amount = |t: &str| parse_amount(t, "sg_amount", 2, false).map(|d| d.to_sci_string());
        assert_eq!(amount("$ 1,234.567").unwrap(), "1234.57");
        assert_eq!(amount("(0.00)").unwrap(), "0.00");
        assert_eq!(amount("1e2").unwrap(), "100.00");
        assert!(amount("612,00").is_err());
        assert!(amount("-5").is_err());
        assert!(amount("0.004").is_err());
        assert!(amount("1e16").is_err());
        assert!(amount("nan").is_err());
    }
}
