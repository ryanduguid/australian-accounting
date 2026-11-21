// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::super::{Context, Lines, POLICY, linked, parse_json, terms};
use payday_super_checker::py;
use serde_json::{Value, json};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(super) const READ_ERROR: &str = "Cannot read a UTF-8 JSONL index in the configured corpus.";

pub(super) fn policy(name: &str) -> &'static str {
    POLICY["corpus"][name].as_str().unwrap()
}
pub(super) fn string<'a>(row: &'a Value, field: &str) -> Option<&'a str> {
    row[field].as_str()
}
pub(super) fn py_string(value: &Value) -> String {
    div7a_loan_review::raw::Raw::from_value(value).py_str()
}
pub(super) fn root(context: &Context) -> Result<PathBuf, String> {
    context.root(
        "AUS_ACCOUNTING_CORPUS_ROOT",
        "legislation corpus folder",
        policy("SETUP_ROUTES"),
    )
}
pub(super) fn query_terms(query: &str, field: &str) -> Result<Vec<String>, String> {
    let result = terms(query);
    if result.is_empty() {
        Err(format!("Supply at least one word in {field}."))
    } else {
        Ok(result)
    }
}
pub(super) fn optional_words(input: &Value, field: &str) -> Result<Vec<String>, String> {
    match input[field].as_str().filter(|s| !s.is_empty()) {
        Some(text) => query_terms(text, field),
        None => Ok(Vec::new()),
    }
}
pub(in crate::retrieval) fn joined(row: &Value, fields: &[&str]) -> String {
    py::casefold(
        &fields
            .iter()
            .filter_map(|f| string(row, f))
            .collect::<Vec<_>>()
            .join(" "),
    )
}
pub(in crate::retrieval) fn holds(row: &Value, fields: &[&str], words: &[String]) -> bool {
    let joined = joined(row, fields);
    words.iter().all(|w| joined.contains(w))
}

pub(super) fn indexes(context: &Context, root: &Path) -> Result<Vec<PathBuf>, String> {
    let titles = root.join("markdown");
    if !titles.is_dir() || linked(&titles).unwrap_or(true) {
        return Err("The corpus folder must contain a markdown directory of title indexes.".into());
    }
    let mut entries = fs::read_dir(titles)
        .map_err(|_| READ_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| READ_ERROR)?;
    entries.sort_by_key(|e| e.file_name().to_string_lossy().into_owned());
    let files: Vec<_> = entries
        .into_iter()
        .filter_map(|e| {
            let directory = e.path();
            if e.file_name().to_string_lossy().starts_with('.')
                || linked(&directory).unwrap_or(true)
                || !directory.is_dir()
            {
                return None;
            }
            let index = directory.join("sections.jsonl");
            (index.is_file() && !linked(&index).unwrap_or(true)).then_some(index)
        })
        .collect();
    if files.is_empty() {
        return Err("No title indexes found under the configured corpus folder.".into());
    }
    let maximum = context.limit("MAX_INDEX_FILES", 5000);
    if files.len() > maximum {
        return Err(format!(
            "Corpus exceeds {maximum} title indexes; configure a smaller corpus."
        ));
    }
    Ok(files)
}

/// Stream universal-newline UTF-8 JSONL. Decode every scanned line, even when
/// a raw prefilter subsequently excludes it. Malformed JSON rows are skipped.
/// `least` asks a raw line for that many of the filter's terms, as a partial hit
/// needs; None asks for every term.
pub(super) struct Rows {
    lines: Lines,
    filter: Vec<String>,
    least: Option<usize>,
}
impl Rows {
    pub(super) fn open(path: &Path, filter: &[String]) -> io::Result<Self> {
        Self::open_some(path, filter, None)
    }
    pub(super) fn open_some(
        path: &Path,
        filter: &[String],
        least: Option<usize>,
    ) -> io::Result<Self> {
        Ok(Self {
            lines: Lines::open(path)?,
            filter: filter.to_vec(),
            least,
        })
    }
}
impl Iterator for Rows {
    type Item = io::Result<Value>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = match self.lines.next()? {
                Ok(line) => line,
                Err(e) => return Some(Err(e)),
            };
            if !line.contains("\\u") && !line.contains("\\/") {
                let folded = py::casefold(&line);
                let held = self
                    .filter
                    .iter()
                    .filter(|t| folded.contains(t.as_str()))
                    .count();
                if held < self.least.unwrap_or(self.filter.len()) {
                    continue;
                }
            }
            match parse_json(&line) {
                Ok(row) if row.is_object() => return Some(Ok(row)),
                Err(error) if error == "JSON nesting exceeds 1000 containers." => {
                    return Some(Err(io::Error::new(io::ErrorKind::InvalidData, error)));
                }
                _ => {}
            }
        }
    }
}

pub(super) fn provenance(root: &Path) -> Value {
    let path = root.join("sources.json");
    let mut result = json!({});
    if !path.is_file() || linked(&path).unwrap_or(true) {
        return result;
    }
    let Ok(raw) = fs::read(&path) else {
        return result;
    };
    let Ok(text) = std::str::from_utf8(&raw) else {
        return result;
    };
    let Ok(record) = parse_json(text) else {
        return result;
    };
    for field in [
        "corpus",
        "source",
        "retrieved",
        "licence",
        "licence_url",
        "licence_note",
    ] {
        if let Some(value) = record[field].as_str() {
            result[field] = json!(value);
        }
    }
    if let Some(value) = record["attribution_markdown"].as_str() {
        result["attribution"] = json!(value);
    }
    result
}

pub(super) fn bounded_size(context: &Context, path: &Path, size: &mut u64) -> Result<(), String> {
    *size += fs::metadata(path).map_err(|_| READ_ERROR)?.len();
    let maximum = context.limit("MAX_CORPUS_BYTES", 320_000_000);
    if *size > maximum as u64 {
        return Err(format!(
            "Corpus exceeds {} MB; configure a smaller corpus.",
            maximum / 1_000_000
        ));
    }
    Ok(())
}

/// How many lines are rows a search can return, and how many are not.
fn count_rows(path: &Path, required: &[&str]) -> Result<(usize, usize), String> {
    let (mut rows, mut unreadable) = (0, 0);
    for line in Lines::open(path).map_err(|e| e.to_string())? {
        let line = line.map_err(|e| e.to_string())?;
        let row = parse_json(&line).ok().filter(Value::is_object);
        if row.is_some_and(|row| required.iter().all(|key| row[*key].is_string())) {
            rows += 1;
        } else if !line.trim_matches(py::is_space).is_empty() {
            unreadable += 1;
        }
    }
    Ok((rows, unreadable))
}

/// What the configured corpus holds, for `aus-accounting-mcp doctor`.
pub(in crate::retrieval) fn census(context: &Context) -> Result<Value, String> {
    let root = root(context)?;
    let files = indexes(context, &root)?;
    let (mut rows, mut unreadable, mut size) = (0, 0, 0);
    for path in &files {
        bounded_size(context, path, &mut size)?;
        let (held, bad) = count_rows(path, &["row_id", "text"])?;
        rows += held;
        unreadable += bad;
    }
    let path = root.join("rates/rates.jsonl");
    let mut rates = Value::Null;
    if path.is_file()
        && !linked(&path).unwrap_or(true)
        && !linked(path.parent().unwrap()).unwrap_or(true)
    {
        let (held, bad) = count_rows(&path, &["rate_id", "content"])?;
        rates = json!(held);
        unreadable += bad;
    }
    let entries = fs::read_dir(root.join("markdown"))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .count();
    Ok(
        json!({"indexes":files.len(),"rows":rows,"rate_rows":rates,"unreadable":unreadable,
        "skipped_entries":entries - files.len()}),
    )
}
