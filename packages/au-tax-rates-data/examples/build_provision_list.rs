// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Write legislation-provisions.json from a local build of the legislation corpus.
//!
//! CI has no corpus, so the root `tests/test_dated_rates.py` holds each
//! record's `legislation` citations to this checked-in list, which records
//! where each provision came from: the corpus retrieval date, each title's
//! compilation, the one corpus row whose heading is that provision and the
//! digest of the `sections.jsonl` it was read from. Run it after changing a
//! citation or the corpus:
//!
//!     cargo run --example build_provision_list -- <corpus folder>

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use au_tax_rates_data::py::is_space;
use au_tax_rates_data::pyjson::{Json, PyStr, loads_bytes, loads_standard};
use au_tax_rates_data::pyos::{OsError, Style};
use au_tax_rates_data::pypath::PyPath;
use au_tax_rates_data::pyvalue::{dumps_with, utf8_strict_error};
use sha2::{Digest, Sha256};

fn package_root() -> PathBuf {
    let root = if cfg!(feature = "differential") {
        std::env::var("RATES_ROOT").ok()
    } else {
        None
    }
    .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").to_string());
    let path = PyPath::new(&root);
    path.resolve().unwrap_or(path).fs()
}

fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path)
        .map_err(|e| OsError::new(&e, Some(&path.display().to_string()), Style::Errno).text)
}

fn read_json_utf8(path: &Path) -> Result<Json, String> {
    let bytes = read_bytes(path)?;
    let text =
        String::from_utf8(bytes).map_err(|e| format!("{}: invalid UTF-8: {e}", path.display()))?;
    loads_standard(&text).map_err(|e| format!("{}: {e:?}", path.display()))
}

fn field<'a>(obj: &'a Json, key: &str, where_: &str) -> Result<&'a Json, String> {
    obj.get(key)
        .ok_or_else(|| format!("{where_}: missing field {key:?}"))
}

fn str_field<'a>(obj: &'a Json, key: &str, where_: &str) -> Result<&'a str, String> {
    field(obj, key, where_)?
        .as_str()
        .ok_or_else(|| format!("{where_}: field {key:?} is not a string"))
}

/// The distinct `(title_id, provision)` pairs the records in `package/data` cite.
pub fn citations(package: &Path) -> Result<Vec<(String, String)>, String> {
    let data_dir = package.join("data");
    let entries = fs::read_dir(&data_dir)
        .map_err(|e| OsError::new(&e, Some(&data_dir.display().to_string()), Style::Native).text)?;
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();

    let mut pairs = BTreeSet::new();
    for path in paths {
        let where_ = path.display().to_string();
        let record = read_json_utf8(&path)?;
        let Some(items) = record.get("legislation") else {
            continue;
        };
        let Json::Array(list) = items else {
            return Err(format!("{where_}: 'legislation' is not a list"));
        };
        for item in list {
            let title_id = str_field(item, "title_id", &where_)?.to_string();
            let provision = str_field(item, "provision", &where_)?.to_string();
            pairs.insert((title_id, provision));
        }
    }
    Ok(pairs.into_iter().collect())
}

/// The provision a row's heading starts with, with an ASCII hyphen.
pub fn heading_number(row: &Json) -> Result<Option<String>, String> {
    let heading = match row.get("heading") {
        None => "",
        Some(value) => value
            .as_str()
            .ok_or_else(|| "corpus row field 'heading' is not a string".to_string())?,
    };
    let normalised = heading.replace('\u{2011}', "-");
    Ok(normalised
        .split(is_space)
        .find(|word| !word.is_empty())
        .map(str::to_string))
}

fn split_bytes_lines(data: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'\r' {
            lines.push(&data[start..i]);
            i += if data.get(i + 1) == Some(&b'\n') {
                2
            } else {
                1
            };
            start = i;
        } else if data[i] == b'\n' {
            lines.push(&data[start..i]);
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }
    if start < data.len() {
        lines.push(&data[start..]);
    }
    lines
}

fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn encode_json_indented(value: &Json) -> Result<Vec<u8>, String> {
    let mut points = dumps_with(value, Some(2), false, false);
    points.push(u32::from('\n'));
    if let Some(err) = utf8_strict_error(&points) {
        return Err(err);
    }
    let mut out = String::with_capacity(points.len());
    for cp in points {
        out.push(char::from_u32(cp).expect("validated Unicode scalar"));
    }
    Ok(out.into_bytes())
}

fn build_provision(
    title_id: &str,
    provision: &str,
    rows: &[Json],
    digest: &str,
) -> Result<Json, String> {
    let mut found = Vec::new();
    for row in rows {
        if heading_number(row)?.as_deref() == Some(provision) {
            found.push(row);
        }
    }
    if found.len() != 1 {
        return Err(format!(
            "{title_id} has {} rows for {provision} in this corpus",
            found.len()
        ));
    }
    let row = found[0];
    let where_ = format!("{title_id}/{provision}");
    Ok(Json::obj([
        ("title_id", Json::str(title_id)),
        ("provision", Json::str(provision)),
        ("act", field(row, "act", &where_)?.clone()),
        (
            "compilation_number",
            field(row, "compilation_number", &where_)?.clone(),
        ),
        (
            "compilation_date",
            field(row, "compilation_date", &where_)?.clone(),
        ),
        ("row_id", field(row, "row_id", &where_)?.clone()),
        ("heading", field(row, "heading", &where_)?.clone()),
        ("sections_sha256", Json::str(digest)),
    ]))
}

pub fn build(package: &Path, corpus: &Path) -> Result<Vec<u8>, String> {
    let sources_path = corpus.join("sources.json");
    let sources = read_json_utf8(&sources_path)?;
    let source_where = sources_path.display().to_string();
    let corpus_name = field(&sources, "corpus", &source_where)?.clone();
    let retrieved = field(&sources, "retrieved", &source_where)?.clone();

    let mut provisions = Vec::new();
    for (title_id, provision) in citations(package)? {
        let sections_path = corpus
            .join("markdown")
            .join(&title_id)
            .join("sections.jsonl");
        let data = read_bytes(&sections_path)?;
        let digest = sha256_hex(&data);
        let mut rows = Vec::new();
        for (line_no, line) in split_bytes_lines(&data).into_iter().enumerate() {
            let row = loads_bytes(line).map_err(|e| {
                format!(
                    "{}:{}: invalid JSON line: {e:?}",
                    sections_path.display(),
                    line_no + 1
                )
            })?;
            rows.push(row);
        }
        provisions.push(build_provision(&title_id, &provision, &rows, &digest)?);
    }

    let listing = Json::obj([
        (
            "corpus",
            Json::Object(vec![
                (PyStr::Text("name".into()), corpus_name),
                (PyStr::Text("retrieved".into()), retrieved),
            ]),
        ),
        ("provisions", Json::Array(provisions)),
    ]);
    let output = encode_json_indented(&listing)?;
    let out_path = package.join("legislation-provisions.json");
    fs::write(&out_path, &output)
        .map_err(|e| OsError::new(&e, Some(&out_path.display().to_string()), Style::Errno).text)?;
    Ok(output)
}

const USAGE: &str = "usage: build_provision_list <corpus folder>";

/// The exit code and stderr of one run against `package`.
fn run(package: &Path, args: &[OsString]) -> (u8, String) {
    let [corpus] = args else {
        return (1, format!("{USAGE}\n"));
    };
    match build(package, Path::new(corpus)) {
        Ok(_) => (0, String::new()),
        Err(message) => (1, format!("{message}\n")),
    }
}

fn output_bytes(text: &str) -> Vec<u8> {
    if !cfg!(windows) {
        return text.as_bytes().to_vec();
    }
    let mut out = Vec::with_capacity(text.len());
    for &b in text.as_bytes() {
        if b == b'\n' {
            out.push(b'\r');
        }
        out.push(b);
    }
    out
}

fn main() -> ExitCode {
    // The corpus folder is the operator's own local path.
    let raw = std::env::args_os(); // nosemgrep: rust.lang.security.args-os.args-os
    let args: Vec<OsString> = raw.skip(1).collect();
    let (code, stderr) = run(&package_root(), &args);
    let _ = std::io::stderr().write_all(&output_bytes(&stderr));
    ExitCode::from(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECORDED: &str = include_str!("../tests/data/build_provision_list-python.json");

    /// A fresh scratch folder for one case; test-only, holding nothing secret.
    fn scratch(case: &str) -> PathBuf {
        let base = std::env::temp_dir(); // nosemgrep: rust.lang.security.temp-dir.temp-dir
        let dir = base.join(format!(
            "build-provision-list-{}-{case}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("pkg/data")).unwrap();
        dir
    }

    /// Lays out one recorded case: the package's own records and the case's corpus.
    fn write_case(dir: &Path, case: &Json) {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        for entry in fs::read_dir(data).unwrap() {
            let path = entry.unwrap().path();
            fs::copy(&path, dir.join("pkg/data").join(path.file_name().unwrap())).unwrap();
        }
        let Json::Object(files) = case.at("files") else {
            panic!("files must be an object");
        };
        for (rel, text) in files {
            let path = dir.join(rel.as_str().unwrap());
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text.as_str().unwrap()).unwrap();
        }
    }

    fn text(value: &Json) -> String {
        value.as_str().unwrap().to_string()
    }

    /// Every recorded answer of the Python `build_provision_list.py`, replayed.
    #[test]
    fn port_answers_as_the_python_script_did() {
        let doc = loads_standard(RECORDED).unwrap();
        let Json::Array(cases) = doc.at("cases") else {
            panic!("cases must be an array");
        };
        assert!(cases.len() >= 7, "recorded cases");
        for case in cases {
            let name = text(case.at("name"));
            let dir = scratch(&name);
            write_case(&dir, case);
            let root = dir.to_string_lossy().replace('\\', "/");
            let Json::Array(raw_args) = case.at("args") else {
                panic!("args must be an array");
            };
            let args: Vec<OsString> = raw_args
                .iter()
                .map(|a| text(a).replace("{dir}", &root).into())
                .collect();
            let (code, stderr) = run(&dir.join("pkg"), &args);
            let Json::Int(exit) = case.at("exit") else {
                panic!("exit must be an integer");
            };
            assert_eq!(code.to_string(), exit.to_string(), "{name}: {stderr}");
            if text(case.at("check")) == "exact" {
                assert_eq!(stderr, text(case.at("stderr")), "{name}");
            }
            if let Json::Object(outputs) = case.at("outputs") {
                for (rel, expected) in outputs {
                    let path = dir.join("pkg").join(rel.as_str().unwrap());
                    assert_eq!(fs::read_to_string(path).unwrap(), text(expected), "{name}");
                }
            }
            let _ = fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn heading_number_normalises_non_breaking_hyphen_and_unicode_space() {
        let row = loads_standard(r#"{"heading": "45\u201115\u00a0Liability"}"#).unwrap();
        assert_eq!(heading_number(&row).unwrap().as_deref(), Some("45-15"));
        let empty = loads_standard(r#"{"heading": "   "}"#).unwrap();
        assert_eq!(heading_number(&empty).unwrap(), None);
        let missing = loads_standard(r#"{}"#).unwrap();
        assert_eq!(heading_number(&missing).unwrap(), None);
    }

    #[test]
    fn shipped_legislation_provisions_round_trips_byte_for_byte() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let raw = fs::read(manifest.join("legislation-provisions.json")).unwrap();
        let text = String::from_utf8(raw.clone()).unwrap();
        let parsed = loads_standard(&text).unwrap();
        let reencoded = encode_json_indented(&parsed).unwrap();
        assert_eq!(reencoded, raw);
    }

    #[test]
    fn shipped_citations_match_legislation_provisions_list() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let pairs = citations(manifest).unwrap();
        let listed = read_json_utf8(&manifest.join("legislation-provisions.json")).unwrap();
        let Json::Array(items) = listed.at("provisions") else {
            panic!("provisions must be an array");
        };
        let expected: Vec<(String, String)> = items
            .iter()
            .map(|item| {
                (
                    item.at("title_id").as_str().unwrap().to_string(),
                    item.at("provision").as_str().unwrap().to_string(),
                )
            })
            .collect();
        assert_eq!(pairs, expected);
    }
}
