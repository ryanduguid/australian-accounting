// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Convert an ATO small business benchmarks workbook into the shipped JSON dataset.
//!
//! Development-time script using `[dev-dependencies]` (`zip`, `roxmltree`), so
//! the release binary links none of it. Run when the ATO publishes a new year:
//!
//!     cargo run --example build_dataset -- \
//!         --xlsx small-business-benchmarks-2023-24-data.xlsx \
//!         --year 2023-24 \
//!         --resource-url https://data.gov.au/... \
//!         --resource-name "2023-24 Benchmarks" \
//!         --resource-last-modified 2026-03-15T22:07:39.563377 \
//!         --retrieved 2026-08-13 \
//!         --out data/benchmarks-2023-24.json

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use ato_benchmark_compare::cli::{decode_os, output_bytes};
use ato_benchmark_compare::decimal::Decimal;
use ato_benchmark_compare::py::{PyPath, decimal_value, is_space, repr_str};
use ato_benchmark_compare::raw::float_repr;
use num_bigint::BigInt;
use roxmltree::{Document, Node};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use zip::ZipArchive;
use zip::result::ZipError;

const COLS_READ: usize = 16;
const MAX_EXACT_F64_INT: f64 = 9_007_199_254_740_991.0;
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const BAND_SPEC: [(&str, usize, usize, usize, usize, usize); 3] = [
    ("low", 1, 2, 7, 8, 13),
    ("medium", 3, 4, 9, 10, 14),
    ("high", 5, 6, 11, 12, 15),
];

#[derive(Clone, Debug, PartialEq)]
enum CellValue {
    Empty,
    Bool(bool),
    Int(String),
    Float(f64),
    Str(String),
}

impl CellValue {
    fn is_none(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    fn py_str(&self) -> String {
        match self {
            CellValue::Empty => "None".into(),
            CellValue::Bool(true) => "True".into(),
            CellValue::Bool(false) => "False".into(),
            CellValue::Int(digits) => digits.clone(),
            CellValue::Float(v) => float_repr(*v),
            CellValue::Str(s) => s.clone(),
        }
    }

    fn py_repr(&self) -> String {
        match self {
            CellValue::Str(s) => repr_str(s),
            other => other.py_str(),
        }
    }
}

struct CliArgs {
    xlsx: PyPath,
    year: String,
    resource_name: String,
    resource_url: String,
    resource_last_modified: String,
    retrieved: String,
    out: PyPath,
}

fn usage() -> &'static str {
    "usage: build_dataset [-h] --xlsx XLSX --year YEAR --resource-name RESOURCE_NAME \
     --resource-url RESOURCE_URL --resource-last-modified RESOURCE_LAST_MODIFIED \
     --retrieved RETRIEVED --out OUT"
}

fn help_text() -> String {
    format!(
        "{}\n\nConvert an ATO small business benchmarks workbook into the shipped JSON dataset.\n\noptions:\n  \
         -h, --help            show this help message and exit\n  \
         --xlsx XLSX\n  \
         --year YEAR           benchmark year, for example \"2023-24\"\n  \
         --resource-name RESOURCE_NAME\n  \
         --resource-url RESOURCE_URL\n  \
         --resource-last-modified RESOURCE_LAST_MODIFIED\n  \
         --retrieved RETRIEVED\n                        ISO date the file was downloaded\n  \
         --out OUT",
        usage()
    )
}

fn cli_err(msg: &str) -> String {
    format!("{}\nbuild_dataset: error: {msg}", usage())
}

fn parse_cli(args: &[String]) -> Result<Option<CliArgs>, String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        return Ok(None);
    }
    let mut vals: HashMap<&'static str, String> = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let raw = &args[i];
        let (flag, inline) = raw
            .split_once('=')
            .map_or((raw.as_str(), None), |(f, v)| (f, Some(v)));
        let key = match flag {
            "--xlsx" => "--xlsx",
            "--year" => "--year",
            "--resource-name" => "--resource-name",
            "--resource-url" => "--resource-url",
            "--resource-last-modified" => "--resource-last-modified",
            "--retrieved" => "--retrieved",
            "--out" => "--out",
            _ => return Err(cli_err(&format!("unrecognized arguments: {raw}"))),
        };
        let value = match inline {
            Some(v) => v.to_string(),
            None => {
                i += 1;
                args.get(i)
                    .cloned()
                    .ok_or_else(|| cli_err(&format!("argument {key}: expected one argument")))?
            }
        };
        vals.insert(key, value);
        i += 1;
    }
    let required = [
        "--xlsx",
        "--year",
        "--resource-name",
        "--resource-url",
        "--resource-last-modified",
        "--retrieved",
        "--out",
    ];
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|k| !vals.contains_key(k))
        .collect();
    if !missing.is_empty() {
        return Err(cli_err(&format!(
            "the following arguments are required: {}",
            missing.join(", ")
        )));
    }
    Ok(Some(CliArgs {
        xlsx: PyPath::new(&vals["--xlsx"]),
        year: vals["--year"].clone(),
        resource_name: vals["--resource-name"].clone(),
        resource_url: vals["--resource-url"].clone(),
        resource_last_modified: vals["--resource-last-modified"].clone(),
        retrieved: vals["--retrieved"].clone(),
        out: PyPath::new(&vals["--out"]),
    }))
}

fn write_json_indented(value: &Value, level: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => out.push_str(&serde_json::to_string(s).expect("JSON string")),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.extend(std::iter::repeat_n(' ', level + 1));
                write_json_indented(item, level + 1, out);
            }
            out.push('\n');
            out.extend(std::iter::repeat_n(' ', level));
            out.push(']');
        }
        Value::Object(map) if map.is_empty() => out.push_str("{}"),
        Value::Object(map) => {
            out.push('{');
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.extend(std::iter::repeat_n(' ', level + 1));
                out.push_str(&serde_json::to_string(k).expect("JSON key"));
                out.push_str(": ");
                write_json_indented(v, level + 1, out);
            }
            out.push('\n');
            out.extend(std::iter::repeat_n(' ', level));
            out.push('}');
        }
    }
}

fn to_json_indented(value: &Value) -> String {
    let mut out = String::new();
    write_json_indented(value, 0, &mut out);
    out.push('\n');
    out
}

fn read_zip_part(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<Option<String>, String> {
    let mut file = match archive.by_name(name) {
        Ok(file) => file,
        Err(ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(format!("{name}: {e}")),
    };
    let mut text = String::new();
    file.read_to_string(&mut text)
        .map_err(|e| format!("{name}: {e}"))?;
    Ok(Some(text))
}

fn resolve_rel(source: &str, target: &str) -> String {
    if let Some(abs) = target.strip_prefix('/') {
        return abs.to_string();
    }
    let mut parts: Vec<&str> = source.split('/').collect();
    parts.pop();
    for seg in target.split('/') {
        match seg {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            _ => parts.push(seg),
        }
    }
    parts.join("/")
}

fn extract_ooxml_text(node: Node<'_, '_>) -> String {
    let mut out = String::new();
    for child in node.children() {
        if child.has_tag_name("t") {
            if let Some(txt) = child.text() {
                out.push_str(txt);
            }
        } else if child.has_tag_name("r") {
            for sub in child.children().filter(|n| n.has_tag_name("t")) {
                if let Some(txt) = sub.text() {
                    out.push_str(txt);
                }
            }
        }
    }
    out
}

// openpyxl 3.1.5 reader/strings.py replaces only literal `x005F_` in shared
// strings; WorkSheetParser.parse_cell keeps inlineStr verbatim and neither
// path calls openpyxl.utils.escape.unescape on cell values.
fn read_shared_strings(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    part: &str,
) -> Result<Vec<String>, String> {
    let Some(xml) = read_zip_part(archive, part)? else {
        return Ok(Vec::new());
    };
    let doc = Document::parse(&xml).map_err(|e| format!("{part}: {e}"))?;
    Ok(doc
        .descendants()
        .filter(|n| n.has_tag_name("si"))
        .map(|si| extract_ooxml_text(si).replace("x005F_", ""))
        .collect())
}

fn parse_cell_coord(coord: &str) -> Option<(usize, usize)> {
    let split = coord.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = coord.split_at(split);
    if letters.is_empty() {
        return None;
    }
    let mut col = 0usize;
    for b in letters.bytes() {
        if !b.is_ascii_alphabetic() {
            return None;
        }
        col = col
            .checked_mul(26)?
            .checked_add((b.to_ascii_uppercase() - b'A') as usize + 1)?;
    }
    let row: usize = digits.parse().ok()?;
    (row >= 1 && col >= 1).then_some((row, col))
}

fn cast_number(raw: &str) -> Result<CellValue, String> {
    if raw.contains(['.', 'e', 'E']) {
        let v: f64 = raw
            .trim()
            .parse()
            .map_err(|_| format!("invalid float cell value {raw:?}"))?;
        Ok(CellValue::Float(v))
    } else {
        let int: BigInt = raw
            .trim()
            .parse()
            .map_err(|_| format!("invalid integer cell value {raw:?}"))?;
        Ok(CellValue::Int(int.to_string()))
    }
}

fn parse_cell_node(
    cell: Node<'_, '_>,
    row_counter: usize,
    col_counter: &mut usize,
    shared: &[String],
) -> Result<(usize, usize, CellValue), String> {
    let data_type = cell.attribute("t").unwrap_or("n");
    let (row, col) = match cell.attribute("r") {
        Some(coord) => {
            let (r, c) = parse_cell_coord(coord)
                .ok_or_else(|| format!("invalid cell coordinate {coord:?}"))?;
            *col_counter = c;
            (r, c)
        }
        None => {
            *col_counter = col_counter
                .checked_add(1)
                .ok_or_else(|| "column index overflow".to_string())?;
            (row_counter, *col_counter)
        }
    };
    if data_type == "inlineStr" {
        let val = cell
            .children()
            .find(|n| n.has_tag_name("is"))
            .map_or(CellValue::Empty, |is_node| {
                CellValue::Str(extract_ooxml_text(is_node))
            });
        return Ok((row, col, val));
    }
    let raw = cell
        .children()
        .find(|n| n.has_tag_name("v"))
        .and_then(|n| n.text())
        .filter(|s| !s.is_empty());
    let Some(raw) = raw else {
        return Ok((row, col, CellValue::Empty));
    };
    let val = match data_type {
        "n" => cast_number(raw)?,
        "s" => {
            let idx: usize = raw
                .trim()
                .parse()
                .map_err(|_| format!("invalid shared string index {raw:?}"))?;
            let text = shared
                .get(idx)
                .ok_or_else(|| format!("shared string index {idx} out of bounds"))?;
            CellValue::Str(text.clone())
        }
        "b" => {
            let b = raw
                .trim()
                .parse::<i64>()
                .map_err(|_| format!("invalid bool cell value {raw:?}"))?
                != 0;
            CellValue::Bool(b)
        }
        "str" | "e" | "d" => CellValue::Str(raw.to_string()),
        other => return Err(format!("unsupported cell type {other:?}")),
    };
    Ok((row, col, val))
}

fn parse_row_number(attr: &str) -> Result<usize, String> {
    if let Ok(n) = attr.parse::<usize>()
        && n >= 1
    {
        return Ok(n);
    }
    if let Ok(f) = attr.parse::<f64>()
        && f.is_finite()
        && f >= 1.0
        && f.fract() == 0.0
        && f <= MAX_EXACT_F64_INT
        && f <= (usize::MAX as f64)
    {
        return Ok(f as usize);
    }
    Err(format!("{attr} is not a valid row number"))
}

fn workbook_parts(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Result<(String, String), String> {
    let wb_xml = read_zip_part(archive, "xl/workbook.xml")?
        .ok_or_else(|| "workbook missing xl/workbook.xml".to_string())?;
    let rels_xml = read_zip_part(archive, "xl/_rels/workbook.xml.rels")?
        .ok_or_else(|| "workbook missing xl/_rels/workbook.xml.rels".to_string())?;
    let wb_doc = Document::parse(&wb_xml).map_err(|e| format!("xl/workbook.xml: {e}"))?;
    let rels_doc =
        Document::parse(&rels_xml).map_err(|e| format!("xl/_rels/workbook.xml.rels: {e}"))?;
    let first_rid = wb_doc
        .descendants()
        .find(|n| n.has_tag_name("sheet"))
        .and_then(|n| n.attribute((REL_NS, "id")))
        .ok_or_else(|| "workbook has no worksheets".to_string())?;
    let mut sheet_part = None;
    let mut shared_part = None;
    for rel in rels_doc
        .descendants()
        .filter(|n| n.has_tag_name("Relationship"))
    {
        let id = rel.attribute("Id").unwrap_or_default();
        let kind = rel.attribute("Type").unwrap_or_default();
        let target = rel.attribute("Target").unwrap_or_default();
        if id == first_rid {
            sheet_part = Some(resolve_rel("xl/workbook.xml", target));
        }
        if kind.ends_with("/sharedStrings") {
            shared_part = Some(resolve_rel("xl/workbook.xml", target));
        }
    }
    let sheet_part = sheet_part.ok_or_else(|| format!("missing relationship {first_rid}"))?;
    Ok((
        sheet_part,
        shared_part.unwrap_or_else(|| "xl/sharedStrings.xml".to_string()),
    ))
}

#[derive(Debug)]
struct SheetRows {
    max_row: usize,
    max_col: usize,
    rows: BTreeMap<usize, [CellValue; COLS_READ]>,
}

fn read_first_sheet_rows(payload: &[u8]) -> Result<SheetRows, String> {
    let mut archive =
        ZipArchive::new(Cursor::new(payload)).map_err(|e| format!("invalid xlsx archive: {e}"))?;
    let (sheet_part, shared_part) = workbook_parts(&mut archive)?;
    let shared = read_shared_strings(&mut archive, &shared_part)?;
    let sheet_xml = read_zip_part(&mut archive, &sheet_part)?
        .ok_or_else(|| format!("missing worksheet part {sheet_part}"))?;
    let doc = Document::parse(&sheet_xml).map_err(|e| format!("{sheet_part}: {e}"))?;

    let mut rows: BTreeMap<usize, [CellValue; COLS_READ]> = BTreeMap::new();
    let (mut row_counter, mut max_row, mut max_col) = (0usize, 0usize, 0usize);
    if let Some(data) = doc.descendants().find(|n| n.has_tag_name("sheetData")) {
        for row in data.children().filter(|n| n.has_tag_name("row")) {
            row_counter = match row.attribute("r") {
                Some(r) => parse_row_number(r)?,
                None => row_counter
                    .checked_add(1)
                    .ok_or_else(|| "row index overflow".to_string())?,
            };
            let mut col_counter = 0usize;
            for cell in row.children().filter(|n| n.has_tag_name("c")) {
                let (r, c, val) = parse_cell_node(cell, row_counter, &mut col_counter, &shared)?;
                max_row = max_row.max(r);
                max_col = max_col.max(c);
                if c <= COLS_READ {
                    let entry = rows
                        .entry(r)
                        .or_insert_with(|| std::array::from_fn(|_| CellValue::Empty));
                    entry[c - 1] = val;
                }
            }
        }
    }
    Ok(SheetRows {
        max_row,
        max_col,
        rows,
    })
}

fn money(text: &str) -> Result<Decimal, String> {
    Decimal::parse(&text.replace(',', ""))
        .map_err(|e| format!("invalid money amount {text:?}: {}", e.name()))
}

fn dec_gt(a: &Decimal, b: &Decimal) -> Result<bool, String> {
    a.gt(b).map_err(|e| format!("decimal error: {}", e.name()))
}

fn dec_lt(a: &Decimal, b: &Decimal) -> Result<bool, String> {
    a.lt(b).map_err(|e| format!("decimal error: {}", e.name()))
}

fn dec_eq(a: &Decimal, b: &Decimal) -> Result<bool, String> {
    Ok(!dec_lt(a, b)? && !dec_gt(a, b)?)
}

fn ratio(value: &CellValue, where_: &str) -> Result<Option<String>, String> {
    if value.is_none() {
        return Ok(None);
    }
    if let CellValue::Str(s) = value
        && s.trim_matches(is_space).is_empty()
    {
        return Ok(None);
    }
    let as_str = match value {
        CellValue::Str(s) => s.trim_matches(is_space).to_string(),
        other => other.py_str(),
    };
    let dec = Decimal::parse(&as_str).map_err(|e| {
        format!(
            "{where_}: invalid decimal {}: {}",
            value.py_repr(),
            e.name()
        )
    })?;
    if !dec.is_finite() {
        return Err(format!("{where_}: non finite ratio {}", value.py_repr()));
    }
    if dec_lt(&dec, &Decimal::from_int(0))? || dec_gt(&dec, &Decimal::from_int(10))? {
        return Err(format!(
            "{where_}: ratio {} is outside any plausible range",
            dec.to_sci_string()
        ));
    }
    Ok(Some(dec.to_sci_string()))
}

fn scan_money_token(s: &str) -> Option<(&str, &str)> {
    let rest = s.strip_prefix('$')?;
    let mut end = 0;
    for (idx, ch) in rest.char_indices() {
        if ch == ',' || decimal_value(ch).is_some() {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    (end > 0).then(|| (&rest[..end], &rest[end..]))
}

fn parse_range_label(
    label: &CellValue,
    where_: &str,
) -> Result<Option<(Decimal, Option<Decimal>)>, String> {
    if label.is_none() {
        return Ok(None);
    }
    let raw = label.py_str();
    let text = raw.trim_matches(is_space);
    if text.is_empty() || text.to_uppercase() == "N/A" {
        return Ok(None);
    }
    if let Some((first, after_first)) = scan_money_token(text) {
        let mid = after_first.trim_start_matches(is_space);
        let mut chars = mid.chars();
        if let Some(dash) = chars.next()
            && matches!(dash, '-' | '\u{2013}' | '\u{2014}')
        {
            let after_dash = chars.as_str().trim_start_matches(is_space);
            if let Some((second, tail)) = scan_money_token(after_dash)
                && tail.is_empty()
            {
                let (low, high) = (money(first)?, money(second)?);
                if dec_gt(&low, &high)? {
                    return Err(format!(
                        "{where_}: range label {} runs backwards",
                        repr_str(text)
                    ));
                }
                return Ok(Some((low, Some(high))));
            }
        }
    }
    let lower = text.to_ascii_lowercase();
    if lower.starts_with("more than ") {
        let rest = &text["more than ".len()..];
        if let Some((amount, tail)) = scan_money_token(rest)
            && tail.is_empty()
        {
            return Ok(Some((money(amount)?, None)));
        }
    }
    Err(format!(
        "{where_}: unrecognised turnover range label {}",
        repr_str(text)
    ))
}

fn pair(
    minimum: Option<String>,
    maximum: Option<String>,
    where_: &str,
) -> Result<Option<Value>, String> {
    match (minimum, maximum) {
        (None, None) => Ok(None),
        (Some(min), Some(max)) => {
            let dmin = Decimal::parse(&min)
                .map_err(|e| format!("{where_}: invalid decimal {min}: {}", e.name()))?;
            let dmax = Decimal::parse(&max)
                .map_err(|e| format!("{where_}: invalid decimal {max}: {}", e.name()))?;
            if dec_gt(&dmin, &dmax)? {
                return Err(format!("{where_}: minimum {min} exceeds maximum {max}"));
            }
            Ok(Some(json!({"min": min, "max": max})))
        }
        _ => Err(format!("{where_}: only one end of the range is published")),
    }
}

fn build_bands(
    row: &[CellValue],
    name: &str,
    anomalies: &mut Vec<String>,
) -> Result<Vec<Value>, String> {
    let mut bands = Vec::new();
    let mut previous_to: Option<Decimal> = None;
    for &(band, te_min, te_max, cos_min, cos_max, label_col) in &BAND_SPEC {
        let label = &row[label_col];
        let Some((printed_from, printed_to)) = parse_range_label(label, &format!("{name}/{band}"))?
        else {
            continue;
        };
        let (turnover_from, from_inclusive) = match &previous_to {
            None => (printed_from.clone(), true),
            Some(prev) => {
                let diff = printed_from
                    .sub(prev)
                    .map_err(|e| format!("decimal error: {}", e.name()))?;
                if !dec_eq(&diff, &Decimal::from_int(0))? && !dec_eq(&diff, &Decimal::from_int(1))?
                {
                    anomalies.push(format!(
                        "{name}/{band}: band starts at {} but the previous band ends at {}",
                        printed_from.to_sci_string(),
                        prev.to_sci_string()
                    ));
                }
                (prev.clone(), false)
            }
        };
        let te = pair(
            ratio(&row[te_min], &format!("{name}/{band}/total_expenses_min"))?,
            ratio(&row[te_max], &format!("{name}/{band}/total_expenses_max"))?,
            &format!("{name}/{band}/total_expenses"),
        )?;
        let cos = pair(
            ratio(&row[cos_min], &format!("{name}/{band}/cost_of_sales_min"))?,
            ratio(&row[cos_max], &format!("{name}/{band}/cost_of_sales_max"))?,
            &format!("{name}/{band}/cost_of_sales"),
        )?;
        bands.push(json!({
            "band": band,
            "label": label.py_str().trim_matches(is_space),
            "turnover_from": turnover_from.to_sci_string(),
            "turnover_from_inclusive": from_inclusive,
            "turnover_to": printed_to.as_ref().map(Decimal::to_sci_string),
            "total_expenses_to_turnover": te,
            "cost_of_sales_to_turnover": cos,
        }));
        previous_to = printed_to;
    }
    if bands.is_empty() {
        return Err(format!("{name}: no turnover bands published"));
    }
    Ok(bands)
}

fn validate_header(header: &[CellValue; COLS_READ], max_col: usize) -> Result<(), String> {
    if header[0].is_none() || !header[0].py_str().contains("BusinessType") {
        return Err(format!(
            "unexpected first header cell: {}",
            header[0].py_repr()
        ));
    }
    if max_col < COLS_READ {
        return Err(format!("expected 16 columns, found {max_col}"));
    }
    for &(_, lo, hi, _, _, _) in &BAND_SPEC {
        for col in [lo, hi] {
            if !header[col].py_str().contains("Total Expenses") {
                return Err(format!(
                    "column {col} is not a Total Expenses column: {}",
                    header[col].py_repr()
                ));
            }
        }
    }
    for &(_, _, _, lo, hi, _) in &BAND_SPEC {
        for col in [lo, hi] {
            if !header[col].py_str().contains("Cost of Sales") {
                return Err(format!(
                    "column {col} is not a Cost of Sales column: {}",
                    header[col].py_repr()
                ));
            }
        }
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn build(xlsx: &Path, args: &CliArgs) -> Result<(Value, Vec<String>), String> {
    let payload = fs::read(xlsx).map_err(|e| format!("{}: {e}", xlsx.display()))?;
    let sheet = read_first_sheet_rows(&payload)?;
    if sheet.max_row == 0 || sheet.max_col == 0 {
        return Err("workbook is empty".to_string());
    }
    let empty_header: [CellValue; COLS_READ] = std::array::from_fn(|_| CellValue::Empty);
    let header = sheet.rows.get(&1).unwrap_or(&empty_header);
    validate_header(header, sheet.max_col)?;

    let mut anomalies = Vec::new();
    let mut business_types = Vec::new();
    let mut seen = BTreeSet::new();
    for row in sheet.rows.range(2..).map(|(_, row)| row) {
        if row[0].is_none() {
            continue;
        }
        let raw_name = row[0].py_str();
        let name = raw_name.trim_matches(is_space).to_string();
        if name.is_empty() {
            continue;
        }
        if !seen.insert(name.clone()) {
            return Err(format!("duplicate business type {}", repr_str(&name)));
        }
        let bands = build_bands(row, &name, &mut anomalies)?;
        let has_cos = bands
            .iter()
            .any(|b| !b["cost_of_sales_to_turnover"].is_null());
        let key_ratio = if has_cos {
            "cost_of_sales_to_turnover"
        } else {
            "total_expenses_to_turnover"
        };
        business_types.push(json!({
            "name": name,
            "key_ratio": key_ratio,
            "turnover_bands": bands,
        }));
    }
    let dataset = json!({
        "schema_version": 1,
        "benchmark_year": args.year,
        "business_type_count": business_types.len(),
        "source": {
            "publisher": "Australian Taxation Office",
            "dataset": "Small Business Benchmarks",
            "dataset_page": "https://data.gov.au/data/dataset/small-business-benchmarks",
            "resource_name": args.resource_name,
            "resource_url": args.resource_url,
            "resource_last_modified": args.resource_last_modified,
            "retrieved": args.retrieved,
            "sha256": sha256_hex(&payload),
            "bytes": payload.len(),
            "licence": "Creative Commons Attribution 2.5 Australia",
            "licence_url": "https://creativecommons.org/licenses/by/2.5/au/",
        },
        "business_types": business_types,
    });
    Ok((dataset, anomalies))
}

fn write_stdout(text: &str) {
    let _ = std::io::stdout().write_all(&output_bytes(text));
}

fn write_stderr(text: &str) {
    let _ = std::io::stderr().write_all(&output_bytes(text));
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args_os().skip(1).map(|a| decode_os(&a)).collect();
    let cli = match parse_cli(&args) {
        Ok(None) => {
            write_stdout(&format!("{}\n", help_text()));
            return ExitCode::SUCCESS;
        }
        Ok(Some(cli)) => cli,
        Err(msg) => {
            write_stderr(&format!("{msg}\n"));
            return ExitCode::from(2);
        }
    };
    let (dataset, anomalies) = match build(&cli.xlsx.fs(), &cli) {
        Ok(res) => res,
        Err(msg) => {
            write_stderr(&format!("error: {msg}\n"));
            return ExitCode::from(1);
        }
    };
    for line in &anomalies {
        write_stderr(&format!("anomaly: {line}\n"));
    }
    let out_fs = cli.out.fs();
    if let Some(parent) = out_fs.parent()
        && !parent.as_os_str().is_empty()
        && let Err(e) = fs::create_dir_all(parent)
    {
        write_stderr(&format!("error: {}: {e}\n", parent.display()));
        return ExitCode::from(1);
    }
    let text = to_json_indented(&dataset);
    if let Err(e) = fs::write(&out_fs, text.as_bytes()) {
        write_stderr(&format!("error: {}: {e}\n", cli.out.display()));
        return ExitCode::from(1);
    }
    let count = dataset["business_type_count"].as_u64().unwrap_or(0);
    let sha = dataset["source"]["sha256"].as_str().unwrap_or_default();
    write_stdout(&format!(
        "wrote {}: {count} business types, sha256 {}\n",
        cli.out.display(),
        &sha[..16]
    ));
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::Workbook;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use zip::write::SimpleFileOptions;

    fn ooxml_bytes(sheet_xml: &str, shared_xml: Option<&str>) -> Vec<u8> {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opts = SimpleFileOptions::default();
            z.start_file("xl/workbook.xml", opts).unwrap();
            z.write_all(br#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets></workbook>"#).unwrap();
            z.start_file("xl/_rels/workbook.xml.rels", opts).unwrap();
            z.write_all(br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#).unwrap();
            z.start_file("xl/worksheets/sheet1.xml", opts).unwrap();
            z.write_all(sheet_xml.as_bytes()).unwrap();
            if let Some(ss) = shared_xml {
                z.start_file("xl/sharedStrings.xml", opts).unwrap();
                z.write_all(ss.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        buf
    }

    fn temp_path(name: &str) -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ato-build-dataset-{}-{n}-{name}",
            std::process::id()
        ))
    }

    fn standard_headers() -> [&'static str; 16] {
        [
            "BusinessType",
            "Low Total Expenses min",
            "Low Total Expenses max",
            "Med Total Expenses min",
            "Med Total Expenses max",
            "High Total Expenses min",
            "High Total Expenses max",
            "Low Cost of Sales min",
            "Low Cost of Sales max",
            "Med Cost of Sales min",
            "Med Cost of Sales max",
            "High Cost of Sales min",
            "High Cost of Sales max",
            "Low Range",
            "Medium Range",
            "High Range",
        ]
    }

    fn write_workbook_bytes(rows: &[Vec<CellValue>]) -> Vec<u8> {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        for (r, row) in rows.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                match cell {
                    CellValue::Empty => {}
                    CellValue::Bool(b) => {
                        ws.write_boolean(r as u32, c as u16, *b).unwrap();
                    }
                    CellValue::Int(digits) => {
                        let v: i64 = digits.parse().unwrap();
                        ws.write_number(r as u32, c as u16, v as f64).unwrap();
                    }
                    CellValue::Float(v) => {
                        ws.write_number(r as u32, c as u16, *v).unwrap();
                    }
                    CellValue::Str(s) => {
                        ws.write_string(r as u32, c as u16, s).unwrap();
                    }
                }
            }
        }
        wb.save_to_buffer().unwrap()
    }

    fn sample_args(xlsx: &Path, out: &Path) -> CliArgs {
        CliArgs {
            xlsx: PyPath::new(&xlsx.to_string_lossy()),
            year: "2023-24".into(),
            resource_name: "2023-24 Benchmarks".into(),
            resource_url: "https://data.gov.au/example.xlsx".into(),
            resource_last_modified: "2026-03-15T22:07:39.563377".into(),
            retrieved: "2026-08-13".into(),
            out: PyPath::new(&out.to_string_lossy()),
        }
    }

    #[test]
    fn shipped_datasets_round_trip_byte_for_byte_with_indent_one() {
        for raw in [
            include_str!("../data/benchmarks-2022-23.json"),
            include_str!("../data/benchmarks-2023-24.json"),
        ] {
            let parsed: Value = serde_json::from_str(raw).unwrap();
            assert_eq!(to_json_indented(&parsed), raw);
        }
    }

    #[test]
    fn build_handles_full_workbook_and_band_gap_anomaly() {
        let xlsx = temp_path("valid.xlsx");
        let out = temp_path("out.json");
        let header: Vec<CellValue> = standard_headers()
            .into_iter()
            .map(|s| CellValue::Str(s.into()))
            .collect();
        let row1 = vec![
            CellValue::Str("Bakeries".into()),
            CellValue::Float(0.54),
            CellValue::Float(0.73),
            CellValue::Float(0.65),
            CellValue::Float(0.78),
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Float(0.31),
            CellValue::Float(0.38),
            CellValue::Float(0.32),
            CellValue::Float(0.41),
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Str("$50,000 - $150,000".into()),
            CellValue::Str("$150,005 \u{2013} $600,000".into()),
            CellValue::Str("N/A".into()),
        ];
        let row2 = vec![
            CellValue::Str("Tax agents".into()),
            CellValue::Float(0.1),
            CellValue::Float(0.25),
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Str("More than $50,000".into()),
            CellValue::Empty,
            CellValue::Empty,
        ];
        fs::write(&xlsx, write_workbook_bytes(&[header, row1, row2])).unwrap();
        let args = sample_args(&xlsx, &out);
        let (dataset, anomalies) = build(&xlsx, &args).unwrap();
        assert_eq!(dataset["business_type_count"], 2);
        assert_eq!(
            dataset["business_types"][0]["key_ratio"],
            "cost_of_sales_to_turnover"
        );
        assert_eq!(
            dataset["business_types"][0]["turnover_bands"][1]["turnover_from"],
            "150000"
        );
        assert_eq!(
            dataset["business_types"][0]["turnover_bands"][1]["turnover_from_inclusive"],
            false
        );
        assert_eq!(
            dataset["business_types"][1]["key_ratio"],
            "total_expenses_to_turnover"
        );
        assert_eq!(
            anomalies,
            vec!["Bakeries/medium: band starts at 150005 but the previous band ends at 150000"]
        );
        let _ = fs::remove_file(&xlsx);
    }

    #[test]
    fn raw_ooxml_preserves_openpyxl_cast_number_and_inline_string_rules() {
        let buf = ooxml_bytes(
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>a_x000D_b _x005F_c x005F_d</t></is></c><c r="B1"><v>0.45000000000000001</v></c><c r="C1"><v>007</v></c><c r="D1" t="s"><v>0</v></c></row></sheetData></worksheet>"#,
            Some(
                r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><si><t>a_x000D_b _x005F_c x005F_d</t></si></sst>"#,
            ),
        );
        let sheet = read_first_sheet_rows(&buf).unwrap();
        assert_eq!(sheet.max_row, 1);
        assert_eq!(sheet.max_col, 4);
        let row1 = &sheet.rows[&1];
        assert_eq!(row1[0], CellValue::Str("a_x000D_b _x005F_c x005F_d".into()));
        assert_eq!(row1[3], CellValue::Str("a_x000D_b _c d".into()));
        assert_eq!(ratio(&row1[1], "x").unwrap().as_deref(), Some("0.45"));
        assert_eq!(ratio(&row1[2], "y").unwrap().as_deref(), Some("7"));
    }

    #[test]
    fn sparse_and_unbounded_ooxml_coordinates_do_not_oom_or_overflow() {
        let sparse = ooxml_bytes(
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>Hdr</t></is></c></row><row r="100000000"><c r="A100000000"><v>1</v></c><c r="XFD1048576"><v>1</v></c></row></sheetData></worksheet>"#,
            None,
        );
        let sheet = read_first_sheet_rows(&sparse).unwrap();
        assert_eq!(sheet.max_row, 100_000_000);
        assert_eq!(sheet.max_col, 16_384);
        assert_eq!(sheet.rows.len(), 2);

        let bad_float = ooxml_bytes(
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1e25"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#,
            None,
        );
        assert!(
            read_first_sheet_rows(&bad_float)
                .unwrap_err()
                .contains("not a valid row number")
        );

        let zero_row = ooxml_bytes(
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="0"/></sheetData></worksheet>"#,
            None,
        );
        assert!(
            read_first_sheet_rows(&zero_row)
                .unwrap_err()
                .contains("not a valid row number")
        );

        let bad_coord = ooxml_bytes(
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A0"><v>1</v></c></row></sheetData></worksheet>"#,
            None,
        );
        assert!(
            read_first_sheet_rows(&bad_coord)
                .unwrap_err()
                .contains("invalid cell coordinate")
        );

        let doc = Document::parse(r#"<c><v>1</v></c>"#).unwrap();
        let mut col = usize::MAX;
        assert_eq!(
            parse_cell_node(doc.root_element(), 1, &mut col, &[]).unwrap_err(),
            "column index overflow"
        );
        assert!(parse_cell_coord("XFD1048576").is_some());
        assert_eq!(parse_cell_coord("A0"), None);
    }

    #[test]
    fn header_and_value_refusals_match_build_dataset_messages() {
        let xlsx = temp_path("bad.xlsx");
        let out = temp_path("o.json");
        let empty = write_workbook_bytes(&[]);
        fs::write(&xlsx, empty).unwrap();
        let args = sample_args(&xlsx, &out);
        assert_eq!(build(&xlsx, &args).unwrap_err(), "workbook is empty");

        let mut bad_hdr: Vec<CellValue> = standard_headers()
            .into_iter()
            .map(|s| CellValue::Str(s.into()))
            .collect();
        bad_hdr[1] = CellValue::Str("Wrong".into());
        fs::write(&xlsx, write_workbook_bytes(&[bad_hdr])).unwrap();
        assert_eq!(
            build(&xlsx, &args).unwrap_err(),
            "column 1 is not a Total Expenses column: 'Wrong'"
        );

        assert!(
            ratio(&CellValue::Float(10.5), "t/low")
                .unwrap_err()
                .contains("outside any plausible range")
        );
        assert!(
            pair(Some("0.5".into()), Some("0.4".into()), "t/low")
                .unwrap_err()
                .contains("minimum 0.5 exceeds maximum 0.4")
        );
        assert!(
            pair(Some("0.5".into()), None, "t/low")
                .unwrap_err()
                .contains("only one end")
        );
        let _ = fs::remove_file(&xlsx);
    }
}
