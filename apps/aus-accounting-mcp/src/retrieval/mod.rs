// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Opt-in, bounded reads of operator-configured reference folders.
mod corpus;
mod library;
mod rulings;
#[cfg(test)]
mod tests;

use serde_json::{Value, json};
use std::{
    fs,
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
    sync::LazyLock,
};

static POLICY: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/retrieval-policy.v1.json"))
        .expect("embedded retrieval policy")
});
const NOT_CONFIGURED: &str = "The folder is set in the MCP client's configuration for this server: tell the user, and do not retry until they have set it.";
const BOUNDARY_NOTICE: &str = "More matches remain, but continuing past 10000 results is not supported: narrow the query and search again.";

struct Context {
    root: Option<PathBuf>,
    #[cfg(test)]
    limits: std::collections::HashMap<String, usize>,
    #[cfg(test)]
    denied: bool,
    #[cfg(test)]
    principal_acts: Option<Vec<String>>,
    #[cfg(test)]
    fault: Option<String>,
}

impl Context {
    fn from_env(variable: &str) -> Self {
        Self {
            root: std::env::var_os(variable)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from),
            #[cfg(test)]
            limits: Default::default(),
            #[cfg(test)]
            denied: false,
            #[cfg(test)]
            principal_acts: None,
            #[cfg(test)]
            fault: None,
        }
    }
    fn root(&self, variable: &str, description: &str) -> Result<PathBuf, String> {
        let configured = self.root.as_ref().ok_or_else(|| {
            format!("Set {variable} to an authorised {description}. {NOT_CONFIGURED}")
        })?;
        fs::canonicalize(configured)
            .ok()
            .filter(|p| p.is_dir())
            .ok_or_else(|| format!("{variable} must name an existing folder. {NOT_CONFIGURED}"))
    }
    fn limit(&self, name: &str, default: usize) -> usize {
        #[cfg(test)]
        if let Some(value) = self.limits.get(name) {
            return default.min(*value);
        }
        let _ = name;
        default
    }
    fn read(&self, path: &Path, limit: usize) -> io::Result<Vec<u8>> {
        #[cfg(test)]
        if self.denied && path.file_name().is_some_and(|n| n == "locked.md") {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        let mut raw = Vec::new();
        fs::File::open(path)?
            .take(limit as u64 + 1)
            .read_to_end(&mut raw)?;
        Ok(raw)
    }
}

fn linked(path: &Path) -> io::Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        Ok(metadata.file_type().is_symlink() || metadata.file_attributes() & 1024 != 0)
    }
    #[cfg(not(windows))]
    {
        Ok(metadata.file_type().is_symlink())
    }
}

fn terms(query: &str) -> Vec<String> {
    payday_super_checker::py::casefold(query)
        .split(|c: char| !payday_super_checker::pyre::is_word(c as u32))
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Python str.splitlines(), including Unicode separators and a single CRLF break.
fn splitlines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if matches!(
            ch,
            '\n' | '\r'
                | '\u{b}'
                | '\u{c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        ) {
            lines.push(text[start..index].to_owned());
            start = index + ch.len_utf8();
            if ch == '\r' && chars.peek().is_some_and(|(_, c)| *c == '\n') {
                start = chars.next().unwrap().0 + 1;
            }
        }
    }
    if start < text.len() {
        lines.push(text[start..].to_owned());
    }
    lines
}

/// Incremental universal-newline UTF-8 input; CR-only files remain streaming.
struct Lines {
    reader: BufReader<fs::File>,
    after_cr: bool,
}
impl Lines {
    fn open(path: &Path) -> io::Result<Self> {
        Ok(Self {
            reader: BufReader::new(fs::File::open(path)?),
            after_cr: false,
        })
    }
}
impl Iterator for Lines {
    type Item = io::Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut raw = Vec::new();
        loop {
            let buffer = match self.reader.fill_buf() {
                Ok(buffer) => buffer,
                Err(e) => return Some(Err(e)),
            };
            if buffer.is_empty() {
                return if raw.is_empty() {
                    None
                } else {
                    Some(
                        String::from_utf8(raw)
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
                    )
                };
            }
            if self.after_cr {
                self.after_cr = false;
                if buffer[0] == b'\n' {
                    self.reader.consume(1);
                    continue;
                }
            }
            let buffer = self.reader.fill_buf().unwrap();
            if let Some(end) = buffer.iter().position(|b| matches!(b, b'\r' | b'\n')) {
                raw.extend_from_slice(&buffer[..end]);
                self.after_cr = buffer[end] == b'\r';
                self.reader.consume(end + 1);
                return Some(
                    String::from_utf8(raw)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
                );
            }
            raw.extend_from_slice(buffer);
            let consumed = buffer.len();
            self.reader.consume(consumed);
        }
    }
}

/// Keep serde's fast normal path. Rare deeper JSON uses a bounded larger stack
/// so valid operator metadata is not silently dropped at serde's default 128.
fn parse_json(text: &str) -> Result<Value, String> {
    use serde::Deserialize;
    match serde_json::from_str(text) {
        Ok(value) => Ok(value),
        Err(error) if error.to_string().contains("recursion limit exceeded") => {
            let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
            for byte in text.bytes() {
                if quoted {
                    if escaped {
                        escaped = false;
                    } else if byte == b'\\' {
                        escaped = true;
                    } else if byte == b'"' {
                        quoted = false;
                    }
                } else {
                    match byte {
                        b'"' => quoted = true,
                        b'[' | b'{' => {
                            depth += 1;
                            if depth > 1000 {
                                return Err("JSON nesting exceeds 1000 containers.".into());
                            }
                        }
                        b']' | b'}' => depth = depth.saturating_sub(1),
                        _ => {}
                    }
                }
            }
            stacker::grow(16 * 1024 * 1024, || {
                let mut decoder = serde_json::Deserializer::from_str(text);
                decoder.disable_recursion_limit();
                let value = Value::deserialize(&mut decoder).map_err(|e| e.to_string())?;
                decoder.end().map_err(|e| e.to_string())?;
                Ok(value)
            })
        }
        Err(error) => Err(error.to_string()),
    }
}

fn page(matches: Vec<Value>, seen: usize, more: bool, notice: &str, max_offset: usize) -> Value {
    let mut result = json!({"matches":matches,"has_more":more,"next_offset":null,"notice":notice});
    if more {
        if seen > max_offset {
            result["notice"] = json!(format!("{notice} {BOUNDARY_NOTICE}"));
        } else {
            result["next_offset"] = json!(seen);
        }
    }
    result
}

// Pydantic permits an arbitrary positive start_line. A position beyond native
// address space is necessarily beyond an in-memory document, not a panic.
fn position(value: &Value) -> usize {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(usize::MAX)
}

pub(crate) fn evaluate(name: &str, input: &Value) -> Result<Value, String> {
    let variable = match name {
        "read_accounting_library" | "search_accounting_library" => "AUS_ACCOUNTING_LIBRARY_ROOT",
        "search_ato_rulings" | "read_ato_ruling" => "AUS_ACCOUNTING_RULINGS_ROOT",
        _ => "AUS_ACCOUNTING_CORPUS_ROOT",
    };
    let context = Context::from_env(variable);
    match name {
        "read_accounting_library" => library::read(
            &context,
            input["path"].as_str().unwrap(),
            position(&input["start_line"]),
            position(&input["line_count"]),
        ),
        "search_accounting_library" => library::search(
            &context,
            input["query"].as_str().unwrap(),
            input["limit"].as_u64().unwrap() as usize,
            input["offset"].as_u64().unwrap() as usize,
        ),
        "search_tax_legislation"
        | "read_tax_legislation_section"
        | "define_tax_term"
        | "search_tax_rates" => corpus::evaluate(&context, name, input),
        "search_ato_rulings" | "read_ato_ruling" => rulings::evaluate(&context, name, input),
        _ => Err(format!(
            "{name}: native migration is incomplete; use the existing Python server"
        )),
    }
}
