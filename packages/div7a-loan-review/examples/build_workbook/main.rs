//! Builds workbooks/div7a-loan-review.xlsx, the accountant-facing Excel
//! front end. Run from the package directory on a machine with desktop Excel:
//!
//!     cargo run --example build_workbook
//!
//! rust_xlsxwriter writes the formulas; a single desktop Excel pass over COM
//! recalculates the workbook so the shipped file carries real cached values
//! (see `excel`). The repository's `tests/test_div7a_workbook.py` holds those
//! values to the engine.

mod builder;
mod excel;
mod model;

use std::io::{Cursor, Write};
use std::path::Path;

const SHIPPED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/workbooks/div7a-loan-review.xlsx"
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
