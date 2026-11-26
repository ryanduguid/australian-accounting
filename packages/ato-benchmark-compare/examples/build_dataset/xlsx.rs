// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The first worksheet of an `.xlsx` file, read as openpyxl 3.1.5's
//! `load_workbook(..., data_only=True)` reads it.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use ato_benchmark_compare::py::repr_str;
use ato_benchmark_compare::raw::float_repr;
use num_bigint::BigInt;
use roxmltree::{Document, Node};
use zip::ZipArchive;
use zip::result::ZipError;

pub const COLS_READ: usize = 16;
const MAX_EXACT_F64_INT: f64 = 9_007_199_254_740_991.0;
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    Empty,
    Bool(bool),
    Int(String),
    Float(f64),
    Str(String),
}

impl CellValue {
    pub fn is_none(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    pub fn py_str(&self) -> String {
        match self {
            CellValue::Empty => "None".into(),
            CellValue::Bool(true) => "True".into(),
            CellValue::Bool(false) => "False".into(),
            CellValue::Int(digits) => digits.clone(),
            CellValue::Float(v) => float_repr(*v),
            CellValue::Str(s) => s.clone(),
        }
    }

    pub fn py_repr(&self) -> String {
        match self {
            CellValue::Str(s) => repr_str(s),
            other => other.py_str(),
        }
    }
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

pub fn parse_cell_coord(coord: &str) -> Option<(usize, usize)> {
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

pub fn parse_cell_node(
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
pub struct SheetRows {
    pub max_row: usize,
    pub max_col: usize,
    pub rows: BTreeMap<usize, [CellValue; COLS_READ]>,
}

pub fn read_first_sheet_rows(payload: &[u8]) -> Result<SheetRows, String> {
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
