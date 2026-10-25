//! Rust port of tools/build_workbook.py, in progress. The Python builder stays
//! the source of the shipped workbook until this one passes the differential
//! gates recorded for the port.

use std::io::{Cursor, Read, Write};

/// The tblLines columns whose calculated formula Excel must treat as an array
/// formula when the table grows; see `repair_array_columns`.
pub const ARRAY_CALCS: [&str; 3] = ["Confirmed_latest", "Possible_latest", "Case_variant"];

fn main() {
    eprintln!("build_workbook: the Rust builder is not complete; use tools/build_workbook.py");
    std::process::exit(1);
}

/// rust_xlsxwriter 0.101.0 has no way to mark a table's calculated column
/// formula as an array formula, which Excel needs to fill a new row without
/// rewriting a same-table column inside EXACT to this-row. Add `array="1"` to
/// exactly the named columns of `table`, refusing anything unexpected.
/// `columns` pairs each column name with the formula it must already carry.
pub fn repair_array_columns(
    xlsx: &[u8],
    sheet: &str,
    table: &str,
    columns: &[(&str, &str)],
) -> Result<Vec<u8>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(xlsx)).map_err(|e| e.to_string())?;
    let part = table_part(&mut archive, sheet, table)?;
    let xml = read_part(&mut archive, &part)?;
    let repaired = mark_array_columns(&xml, columns)?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.name().map_err(|e| e.to_string())? == part.as_str() {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            out.start_file(part.as_str(), options)
                .map_err(|e| e.to_string())?;
            out.write_all(repaired.as_bytes())
                .map_err(|e| e.to_string())?;
        } else {
            out.raw_copy_file(file).map_err(|e| e.to_string())?;
        }
    }
    Ok(out.finish().map_err(|e| e.to_string())?.into_inner())
}

fn read_part(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<String, String> {
    let mut text = String::new();
    archive
        .by_name(name)
        .map_err(|e| format!("{name}: {e}"))?
        .read_to_string(&mut text)
        .map_err(|e| format!("{name}: {e}"))?;
    Ok(text)
}

/// Resolve a relationship target against the directory of its source part.
fn resolve(source: &str, target: &str) -> String {
    if let Some(absolute) = target.strip_prefix('/') {
        return absolute.to_string();
    }
    let mut parts: Vec<&str> = source.split('/').collect();
    parts.pop();
    for segment in target.split('/') {
        match segment {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            _ => parts.push(segment),
        }
    }
    parts.join("/")
}

fn rels_of(part: &str) -> String {
    let (dir, file) = part.rsplit_once('/').unwrap_or(("", part));
    format!("{dir}/_rels/{file}.rels")
}

/// Relationship targets of `part`, resolved, with their type URIs.
fn relationships(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    part: &str,
) -> Result<Vec<(String, String, String)>, String> {
    let rels = rels_of(part);
    let xml = read_part(archive, &rels)?;
    let doc = roxmltree::Document::parse(&xml).map_err(|e| format!("{rels}: {e}"))?;
    Ok(doc
        .descendants()
        .filter(|n| n.has_tag_name("Relationship"))
        .map(|n| {
            let id = n.attribute("Id").unwrap_or_default().to_string();
            let kind = n.attribute("Type").unwrap_or_default().to_string();
            (
                id,
                kind,
                resolve(part, n.attribute("Target").unwrap_or_default()),
            )
        })
        .collect())
}

/// Find the table part named `table` through workbook -> sheet -> table
/// relationships rather than assuming a part name.
fn table_part(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    sheet: &str,
    table: &str,
) -> Result<String, String> {
    const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let workbook = read_part(archive, "xl/workbook.xml")?;
    let doc = roxmltree::Document::parse(&workbook).map_err(|e| e.to_string())?;
    let mut ids = doc
        .descendants()
        .filter(|n| n.has_tag_name("sheet") && n.attribute("name") == Some(sheet))
        .filter_map(|n| n.attribute((R, "id")));
    let id = match (ids.next(), ids.next()) {
        (Some(id), None) => id.to_string(),
        _ => return Err(format!("expected exactly one sheet named {sheet}")),
    };
    let sheet_part = relationships(archive, "xl/workbook.xml")?
        .into_iter()
        .find(|(rid, _, _)| *rid == id)
        .map(|(_, _, target)| target)
        .ok_or_else(|| format!("no relationship {id} for sheet {sheet}"))?;
    let mut found = Vec::new();
    for (_, kind, target) in relationships(archive, &sheet_part)? {
        if !kind.ends_with("/table") {
            continue;
        }
        let xml = read_part(archive, &target)?;
        let doc = roxmltree::Document::parse(&xml).map_err(|e| format!("{target}: {e}"))?;
        if doc.root_element().attribute("name") == Some(table) {
            found.push(target);
        }
    }
    match found.as_slice() {
        [part] => Ok(part.clone()),
        _ => Err(format!(
            "expected exactly one table {table} on {sheet}, found {}",
            found.len()
        )),
    }
}

/// Insert `array="1"` into each named column's `<calculatedColumnFormula>`,
/// which rust_xlsxwriter writes with no attributes.
fn mark_array_columns(xml: &str, columns: &[(&str, &str)]) -> Result<String, String> {
    const TAG: &str = "<calculatedColumnFormula>";
    let doc = roxmltree::Document::parse(xml).map_err(|e| e.to_string())?;
    let mut inserts = Vec::new();
    for (name, formula) in columns {
        let mut matches = doc
            .descendants()
            .filter(|n| n.has_tag_name("tableColumn") && n.attribute("name") == Some(name));
        let column = match (matches.next(), matches.next()) {
            (Some(column), None) => column,
            _ => return Err(format!("expected exactly one table column {name}")),
        };
        let mut formulas = column
            .children()
            .filter(|n| n.has_tag_name("calculatedColumnFormula"));
        let element = match (formulas.next(), formulas.next()) {
            (Some(element), None) => element,
            _ => {
                return Err(format!(
                    "{name}: expected exactly one calculatedColumnFormula"
                ));
            }
        };
        if element.text() != Some(formula) {
            return Err(format!(
                "{name}: calculated formula differs from the expected formula"
            ));
        }
        let start = element.range().start;
        if !xml[start..].starts_with(TAG) {
            return Err(format!(
                "{name}: calculatedColumnFormula already has attributes"
            ));
        }
        inserts.push(start + TAG.len() - 1);
    }
    inserts.sort_unstable();
    let mut repaired = xml.to_string();
    for at in inserts.into_iter().rev() {
        repaired.insert_str(at, r#" array="1""#);
    }
    Ok(repaired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::{Formula, Table, TableColumn, TableStyle, Workbook};

    const CASE: &str = "IF(COUNTIF(tblLines[employee_id],tblLines[[#This Row],[employee_id]])>SUMPRODUCT(--EXACT(tblLines[employee_id],tblLines[[#This Row],[employee_id]])),1,0)";

    /// A Register sheet after a decoy sheet, so the table is not table1 on
    /// sheet1, with one array calculated column and one ordinary one.
    fn sample() -> Vec<u8> {
        let mut wb = Workbook::new();
        let decoy = wb.add_worksheet().set_name("Start Here").unwrap();
        decoy.write(0, 0, "x").unwrap();
        decoy
            .add_table(0, 0, 1, 0, &Table::new().set_name("tblOther"))
            .unwrap();
        let ws = wb.add_worksheet().set_name("Register").unwrap();
        for (c, title) in ["employee_id", "Case_variant", "Plain"].iter().enumerate() {
            ws.write(0, c as u16, *title).unwrap();
        }
        ws.write(1, 0, "JMCGILL").unwrap();
        let columns = [
            TableColumn::new().set_header("employee_id"),
            TableColumn::new()
                .set_header("Case_variant")
                .set_formula(Formula::new(CASE)),
            TableColumn::new()
                .set_header("Plain")
                .set_formula(Formula::new("1+1")),
        ];
        let table = Table::new()
            .set_name("tblLines")
            .set_columns(&columns)
            .set_autofilter(false)
            .set_style(TableStyle::Light1);
        ws.add_table(0, 0, 1, 2, &table).unwrap();
        ws.write_array_formula(1, 1, 1, 1, Formula::new(CASE))
            .unwrap();
        wb.save_to_buffer().unwrap()
    }

    fn part(xlsx: &[u8], name: &str) -> String {
        let mut archive = zip::ZipArchive::new(Cursor::new(xlsx)).unwrap();
        read_part(&mut archive, name).unwrap()
    }

    #[test]
    fn marks_only_the_named_column_and_keeps_every_other_part() {
        let before = sample();
        let after =
            repair_array_columns(&before, "Register", "tblLines", &[("Case_variant", CASE)])
                .unwrap();
        let table = part(&after, "xl/tables/table2.xml");
        assert_eq!(
            table
                .matches(r#"<calculatedColumnFormula array="1">"#)
                .count(),
            1
        );
        assert!(table.contains("<calculatedColumnFormula>1+1</calculatedColumnFormula>"));
        assert!(!table.contains("<autoFilter"));
        assert_eq!(
            table.replacen(r#" array="1""#, "", 1),
            part(&before, "xl/tables/table2.xml")
        );
        let mut a = zip::ZipArchive::new(Cursor::new(before.as_slice())).unwrap();
        let mut b = zip::ZipArchive::new(Cursor::new(after.as_slice())).unwrap();
        let names = |z: &zip::ZipArchive<_>| {
            z.file_names()
                .map(|n| n.unwrap().into_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&a), names(&b));
        for name in names(&a) {
            if name != "xl/tables/table2.xml" {
                assert_eq!(
                    read_part(&mut a, &name).unwrap(),
                    read_part(&mut b, &name).unwrap()
                );
            }
        }
        assert!(part(&after, "xl/worksheets/sheet2.xml").contains(r#"<f t="array" ref="B2">"#));
    }

    #[test]
    fn refuses_anything_unexpected() {
        let xlsx = sample();
        let refuse = |sheet, table, columns: &[(&str, &str)]| {
            repair_array_columns(&xlsx, sheet, table, columns).unwrap_err()
        };
        assert!(refuse("Missing", "tblLines", &[("Case_variant", CASE)]).contains("sheet"));
        assert!(refuse("Register", "tblOther", &[("Case_variant", CASE)]).contains("table"));
        assert!(refuse("Register", "tblLines", &[("Nope", CASE)]).contains("Nope"));
        assert!(refuse("Register", "tblLines", &[("employee_id", CASE)]).contains("exactly one"));
        assert!(refuse("Register", "tblLines", &[("Case_variant", "1+1")]).contains("differs"));
        let once =
            repair_array_columns(&xlsx, "Register", "tblLines", &[("Case_variant", CASE)]).unwrap();
        let twice = repair_array_columns(&once, "Register", "tblLines", &[("Case_variant", CASE)]);
        assert!(twice.unwrap_err().contains("already has attributes"));
    }
}
