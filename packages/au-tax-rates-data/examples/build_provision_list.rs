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

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let corpus = args
        .next()
        .ok_or_else(|| "usage: build_provision_list <corpus folder>".to_string())?;
    if args.next().is_some() {
        return Err("usage: build_provision_list <corpus folder>".to_string());
    }
    build(&package_root(), Path::new(&corpus))?;
    Ok(())
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
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = std::io::stderr().write_all(&output_bytes(&format!("{message}\n")));
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn temp_dir(label: &str) -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "au-tax-rates-build-provision-list-{label}-{}-{n}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_record(data_dir: &Path, name: &str, body: &str) {
        fs::write(data_dir.join(name), body.as_bytes()).unwrap();
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

    #[test]
    fn build_writes_expected_json_and_rejects_ambiguous_rows() {
        let root = temp_dir("build");
        let pkg = root.join("pkg");
        let corpus = root.join("corpus");
        fs::create_dir_all(pkg.join("data")).unwrap();
        fs::create_dir_all(corpus.join("markdown/C1")).unwrap();
        write_record(
            &pkg.join("data"),
            "b.json",
            r#"{"id":"b","legislation":[{"title_id":"C1","provision":"5B"}]}"#,
        );
        write_record(
            &pkg.join("data"),
            "a.json",
            r#"{"id":"a","legislation":[{"title_id":"C1","provision":"5B"}]}"#,
        );
        fs::write(
            corpus.join("sources.json"),
            br#"{"corpus":"Test Corpus","retrieved":"2026-08-04"}"#,
        )
        .unwrap();
        let jsonl = concat!(
            "{\"act\":\"Test Act\",\"compilation_number\":\"1\",",
            "\"compilation_date\":\"2026-07-01\",\"row_id\":\"C1:0001:5B\",",
            "\"heading\":\"5B Employer\u{2019}s amount\"}\n"
        );
        fs::write(corpus.join("markdown/C1/sections.jsonl"), jsonl.as_bytes()).unwrap();

        let written = build(&pkg, &corpus).unwrap();
        assert_eq!(
            written,
            fs::read(pkg.join("legislation-provisions.json")).unwrap()
        );
        let doc = read_json_utf8(&pkg.join("legislation-provisions.json")).unwrap();
        assert_eq!(doc.at("corpus").at("name").as_str(), Some("Test Corpus"));
        let Json::Array(items) = doc.at("provisions") else {
            panic!("provisions must be an array");
        };
        assert_eq!(items.len(), 1);

        let doubled = format!("{jsonl}{jsonl}");
        fs::write(
            corpus.join("markdown/C1/sections.jsonl"),
            doubled.as_bytes(),
        )
        .unwrap();
        assert_eq!(
            build(&pkg, &corpus).unwrap_err(),
            "C1 has 2 rows for 5B in this corpus"
        );
        fs::write(corpus.join("markdown/C1/sections.jsonl"), b"").unwrap();
        assert_eq!(
            build(&pkg, &corpus).unwrap_err(),
            "C1 has 0 rows for 5B in this corpus"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
