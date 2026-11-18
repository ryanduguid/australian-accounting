// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

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

/// Each non-empty line's fields with its line number. Fields are separated by
/// commas; one that holds a comma or a quote is quoted, with each quote in it
/// doubled (RFC 4180). A record ends at LF or CRLF and no field may hold
/// one, so every record is one line and a refusal can name it. A UTF-8 byte
/// order mark at the start is dropped.
pub(super) fn records(text: &str) -> Result<Vec<(usize, Vec<String>)>, Refusal> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    text.split('\n')
        .enumerate()
        .map(|(i, line)| (i + 1, line.strip_suffix('\r').unwrap_or(line)))
        .filter(|(_, line)| !line.is_empty())
        .map(|(at, line)| Ok((at, fields(at, line)?)))
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
