//! Builds workbooks/payday-super-checker.xlsx, the accountant-facing Excel
//! front end. Run from the package directory on a machine with desktop Excel:
//!
//!     cargo run --example build_workbook
//!
//! rust_xlsxwriter writes the formulas; a single desktop Excel pass over COM
//! recalculates the workbook so the shipped file carries real cached values
//! (see `excel`). The repository's `tests/test_payday_workbook.py` holds those
//! values to the engine.

mod builder;
mod excel;
mod model;

use std::io::{Cursor, Write};
use std::path::Path;

use model::{read_part, relationships};

/// The tblLines columns whose calculated formula Excel must treat as an array
/// formula when the table grows; see `repair_array_columns`.
pub const ARRAY_CALCS: [&str; 3] = ["Confirmed_latest", "Possible_latest", "Case_variant"];

const SHIPPED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/workbooks/payday-super-checker.xlsx"
);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let read = |path: &str| std::fs::read(path).unwrap_or_else(|e| fail(&format!("{path}: {e}")));
    let model_of =
        |path: &str| model::model(&read(path)).unwrap_or_else(|e| fail(&format!("{path}: {e}")));
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        // Print a workbook's semantic model, for freezing a reference.
        ["--model", path] => {
            for line in model_of(path) {
                println!("{line}");
            }
        }
        // Compare two workbooks' models; exit 1 on any difference.
        ["--diff", reference, candidate] => {
            let differences = model::differences(&model_of(reference), &model_of(candidate));
            for line in &differences {
                println!("{line}");
            }
            if !differences.is_empty() {
                std::process::exit(1);
            }
        }
        // Build to the shipped path or --out; --no-excel skips the Excel pass,
        // so the file has no cached values and the root parity test fails.
        [] => build(SHIPPED, true),
        ["--out", path] => build(path, true),
        ["--no-excel"] => build(SHIPPED, false),
        ["--no-excel", "--out", path] => build(path, false),
        _ => fail(
            "usage: build_workbook [--no-excel] [--out <xlsx>] | --model <xlsx> | --diff <reference> <candidate>",
        ),
    }
}

/// Build into a new staging file beside `path` and replace `path` only once
/// every step has succeeded, so a failed build leaves the previous workbook.
fn build(path: &str, recalc: bool) {
    if recalc && !cfg!(windows) {
        fail(excel::UNAVAILABLE);
    }
    let xlsx = builder::build().unwrap_or_else(|e| fail(&e.to_string()));
    let dest = Path::new(path);
    let dir = dest.parent().filter(|d| !d.as_os_str().is_empty());
    if let Some(dir) = dir {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| fail(&format!("{}: {e}", dir.display())));
    }
    let name = format!("build_workbook-staging-{}.xlsx", std::process::id());
    let staging = dir.map_or_else(|| Path::new(&name).to_path_buf(), |d| d.join(&name));
    let io = |e: std::io::Error| format!("{}: {e}", staging.display());
    // create_new: never overwrite, and so never remove, a file this run did
    // not create.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)
        .unwrap_or_else(|e| fail(&io(e)));
    let steps = || -> Result<Option<String>, String> {
        file.write_all(&xlsx).map_err(io)?;
        drop(file);
        let status = if recalc {
            Some(excel::recalc(&staging)?)
        } else {
            None
        };
        std::fs::rename(&staging, dest).map_err(|e| format!("{path}: {e}"))?;
        Ok(status)
    };
    match steps() {
        // The workbook is in place, so the report is best-effort: a closed
        // stdout must not turn a finished build into a failed one.
        Ok(status) => {
            let mut out = std::io::stdout().lock();
            let _ = status.map_or(Ok(()), |status| writeln!(out, "{status}"));
            let _ = writeln!(out, "{path}");
        }
        Err(e) => match std::fs::remove_file(&staging) {
            Err(r) if r.kind() != std::io::ErrorKind::NotFound => fail(&format!(
                "{e}; also could not remove {}: {r}",
                staging.display()
            )),
            _ => fail(&e),
        },
    }
}

fn fail(message: &str) -> ! {
    eprintln!("build_workbook: {message}");
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
    replace_part(xlsx, &part, repaired.as_bytes())
}

/// The package with `part` replaced by `body`, deflated, and every other
/// member raw-copied.
pub fn replace_part(xlsx: &[u8], part: &str, body: &[u8]) -> Result<Vec<u8>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(xlsx)).map_err(|e| e.to_string())?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.name().map_err(|e| e.to_string())? == part {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            out.start_file(part, options).map_err(|e| e.to_string())?;
            out.write_all(body).map_err(|e| e.to_string())?;
        } else {
            out.raw_copy_file(file).map_err(|e| e.to_string())?;
        }
    }
    Ok(out.finish().map_err(|e| e.to_string())?.into_inner())
}

/// Find the table part named `table` through workbook -> sheet -> table
/// relationships rather than assuming a part name.
fn table_part(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    sheet: &str,
    table: &str,
) -> Result<String, String> {
    let matching: Vec<String> = model::sheets(archive)?
        .into_iter()
        .filter(|(name, _, _)| name == sheet)
        .map(|(_, _, part)| part)
        .collect();
    let [sheet_part] = matching.as_slice() else {
        return Err(format!("expected exactly one sheet named {sheet}"));
    };
    let sheet_part = sheet_part.clone();
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
