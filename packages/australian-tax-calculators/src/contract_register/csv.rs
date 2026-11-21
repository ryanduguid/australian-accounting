// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The contracts file's records, and a field written back as CSV.
use crate::api::Refusal;
use std::iter::Peekable;
use std::str::Chars;

fn invalid(line: usize, why: &str) -> Refusal {
    Refusal::new(
        "contract_register.csv_invalid",
        format!("line {line}: {why}"),
    )
}

/// Each line's fields with its line number. Fields are separated by commas;
/// one that holds a comma or a quote is quoted, with each quote in it doubled
/// (RFC 4180). A record ends at LF or CRLF and no field may hold one, so every
/// record is one line and a refusal can name it. Empty lines may only end the
/// file: one before the last record is refused, not skipped. A UTF-8 byte
/// order mark at the start is dropped.
pub(super) fn records(text: &str) -> Result<Vec<(usize, Vec<String>)>, Refusal> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let lines: Vec<&str> = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let used = lines
        .iter()
        .rposition(|line| !line.is_empty())
        .map_or(0, |i| i + 1);
    lines[..used]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let at = i + 1;
            if line.is_empty() {
                return Err(invalid(at, "an empty line may only end the file"));
            }
            Ok((at, fields(at, line)?))
        })
        .collect()
}

fn fields(at: usize, line: &str) -> Result<Vec<String>, Refusal> {
    let (mut chars, mut out) = (line.chars().peekable(), Vec::new());
    loop {
        let (field, ended) = if chars.next_if_eq(&'"').is_some() {
            quoted(at, &mut chars)?
        } else {
            plain(at, &mut chars)?
        };
        out.push(field);
        if ended {
            return Ok(out);
        }
    }
}

/// A quoted field after its opening quote, and whether the line ends after it.
fn quoted(at: usize, chars: &mut Peekable<Chars>) -> Result<(String, bool), Refusal> {
    let mut field = String::new();
    loop {
        match chars.next() {
            Some('"') if chars.next_if_eq(&'"').is_some() => field.push('"'),
            Some('"') => break,
            Some(c) => field.push(c),
            None => return Err(invalid(at, "a quoted field is not closed")),
        }
    }
    match chars.next() {
        None => Ok((field, true)),
        Some(',') => Ok((field, false)),
        Some(_) => Err(invalid(
            at,
            "a quoted field must end at a comma or the end of the line",
        )),
    }
}

/// An unquoted field, and whether the line ends after it.
fn plain(at: usize, chars: &mut Peekable<Chars>) -> Result<(String, bool), Refusal> {
    let mut field = String::new();
    loop {
        match chars.next() {
            None => return Ok((field, true)),
            Some(',') => return Ok((field, false)),
            Some('"') => {
                return Err(invalid(
                    at,
                    "a quote may appear only in a quoted field, doubled",
                ));
            }
            Some(c) => field.push(c),
        }
    }
}

/// `text` as a CSV field: quoted, with its quotes doubled, where it holds a
/// comma, a quote or a line break.
pub(super) fn field(text: &str) -> String {
    if text.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}
