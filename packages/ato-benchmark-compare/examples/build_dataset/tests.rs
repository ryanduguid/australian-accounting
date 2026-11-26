// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use std::fs;
use std::io::{Cursor, Write};

use roxmltree::Document;
use serde_json::Value;
use zip::write::SimpleFileOptions;

use crate::bands::ratio;
use crate::shared::{parity, to_json_indented};
use crate::xlsx::{CellValue, parse_cell_coord, parse_cell_node, read_first_sheet_rows};

const RECORDED: &str = include_str!("../../tests/data/builders/build_dataset-python.json");
const WORKSHEET: &str =
    r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#;

fn ooxml_bytes(rows_xml: &str, shared_xml: Option<&str>) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
    let opts = SimpleFileOptions::default();
    let sheet = format!("{WORKSHEET}{rows_xml}</sheetData></worksheet>");
    let parts = [
        (
            "xl/workbook.xml",
            r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
        ),
        ("xl/worksheets/sheet1.xml", &sheet),
    ];
    for (name, text) in parts
        .into_iter()
        .chain(shared_xml.map(|s| ("xl/sharedStrings.xml", s)))
    {
        z.start_file(name, opts).unwrap();
        z.write_all(text.as_bytes()).unwrap();
    }
    z.finish().unwrap();
    buf
}

/// Every recorded answer of the Python `build_dataset.py`, replayed.
#[test]
fn port_answers_as_the_python_script_did() {
    let cases = parity::cases(RECORDED);
    assert!(cases.len() >= 20, "recorded cases");
    for case in &cases {
        let dir = parity::scratch("build-dataset", case["name"].as_str().unwrap());
        if let Some(rel) = case["xlsx"].as_str() {
            let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/builders/");
            fs::copy(format!("{fixture}{rel}"), dir.join("input.xlsx")).unwrap();
        }
        parity::write_files(&dir, case);
        let got = crate::run(&parity::args(&dir, case));
        parity::check(&dir, case, &got);
    }
}

#[test]
fn shipped_datasets_round_trip_byte_for_byte_with_indent_one() {
    for raw in [
        include_str!("../../data/benchmarks-2022-23.json"),
        include_str!("../../data/benchmarks-2023-24.json"),
    ] {
        let parsed: Value = serde_json::from_str(raw).unwrap();
        assert_eq!(to_json_indented(&parsed), raw);
    }
}

#[test]
fn raw_ooxml_preserves_openpyxl_cast_number_and_inline_string_rules() {
    let buf = ooxml_bytes(
        r#"<row r="1"><c r="A1" t="inlineStr"><is><t>a_x000D_b _x005F_c x005F_d</t></is></c><c r="B1"><v>0.45000000000000001</v></c><c r="C1"><v>007</v></c><c r="D1" t="s"><v>0</v></c></row>"#,
        Some(
            r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><si><t>a_x000D_b _x005F_c x005F_d</t></si></sst>"#,
        ),
    );
    let sheet = read_first_sheet_rows(&buf).unwrap();
    assert_eq!((sheet.max_row, sheet.max_col), (1, 4));
    let row1 = &sheet.rows[&1];
    assert_eq!(row1[0], CellValue::Str("a_x000D_b _x005F_c x005F_d".into()));
    assert_eq!(row1[3], CellValue::Str("a_x000D_b _c d".into()));
    assert_eq!(ratio(&row1[1], "x").unwrap().as_deref(), Some("0.45"));
    assert_eq!(ratio(&row1[2], "y").unwrap().as_deref(), Some("7"));
}

#[test]
fn sparse_and_unbounded_ooxml_coordinates_do_not_oom_or_overflow() {
    let sparse = ooxml_bytes(
        r#"<row r="1"><c r="A1" t="inlineStr"><is><t>Hdr</t></is></c></row><row r="100000000"><c r="A100000000"><v>1</v></c><c r="XFD1048576"><v>1</v></c></row>"#,
        None,
    );
    let sheet = read_first_sheet_rows(&sparse).unwrap();
    assert_eq!((sheet.max_row, sheet.max_col), (100_000_000, 16_384));
    assert_eq!(sheet.rows.len(), 2);
    for (rows, expected) in [
        (
            r#"<row r="1e25"><c r="A1"><v>1</v></c></row>"#,
            "not a valid row number",
        ),
        (r#"<row r="0"/>"#, "not a valid row number"),
        (
            r#"<row r="1"><c r="A0"><v>1</v></c></row>"#,
            "invalid cell coordinate",
        ),
    ] {
        let err = read_first_sheet_rows(&ooxml_bytes(rows, None)).unwrap_err();
        assert!(err.contains(expected), "{err}");
    }
    let doc = Document::parse(r#"<c><v>1</v></c>"#).unwrap();
    let mut col = usize::MAX;
    assert_eq!(
        parse_cell_node(doc.root_element(), 1, &mut col, &[]).unwrap_err(),
        "column index overflow"
    );
    assert!(parse_cell_coord("XFD1048576").is_some());
    assert_eq!(parse_cell_coord("A0"), None);
}
