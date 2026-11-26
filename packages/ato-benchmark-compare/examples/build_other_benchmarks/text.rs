// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Line-level readers for the ATO's industry pages saved as text, with the
//! Python spellings `build_other_benchmarks.py`'s messages print.

use ato_benchmark_compare::dataset::normalise;
use ato_benchmark_compare::decimal::{Decimal, Signal};
use ato_benchmark_compare::py::{decimal_value, is_space, repr_str, split_whitespace};
use serde_json::Value;

#[derive(Debug)]
pub enum Error {
    Build(String),
    Mismatch(String),
}

impl Error {
    pub fn message(&self) -> &str {
        match self {
            Error::Build(m) | Error::Mismatch(m) => m,
        }
    }
}

pub fn py_str_splitlines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        let is_break = matches!(
            ch,
            '\n' | '\r'
                | '\x0b'
                | '\x0c'
                | '\x1c'
                | '\x1d'
                | '\x1e'
                | '\u{0085}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if is_break {
            out.push(&text[start..idx]);
            let mut next_start = idx + ch.len_utf8();
            if ch == '\r'
                && let Some(&(next_idx, '\n')) = chars.peek()
            {
                chars.next();
                next_start = next_idx + 1;
            }
            start = next_start;
        }
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

pub fn collapse_spaces(line: &str) -> String {
    split_whitespace(line).join(" ")
}

pub fn py_repr_list(items: &[String]) -> String {
    let joined = items
        .iter()
        .map(|s| repr_str(s))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

pub fn py_repr_pair(a: &str, b: &str) -> String {
    format!("({}, {})", repr_str(a), repr_str(b))
}

pub fn py_repr_obj(val: &Value) -> String {
    let Some(map) = val.as_object() else {
        return val.to_string();
    };
    let parts: Vec<String> = map
        .iter()
        .map(|(k, v)| {
            let v_repr = v.as_str().map(repr_str).unwrap_or_else(|| v.to_string());
            format!("{}: {v_repr}", repr_str(k))
        })
        .collect();
    format!("{{{}}}", parts.join(", "))
}

pub fn slug(text: &str) -> String {
    let folded = normalise(text).replace('\'', "");
    let mut out = String::new();
    let mut dash = false;
    for ch in folded.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

pub fn label_key(label: &str) -> String {
    label
        .replace(['\u{2013}', '\u{2014}'], "-")
        .chars()
        .filter(|c| !is_space(*c))
        .collect::<String>()
        .to_lowercase()
}

fn parse_decimal(text: &str) -> Result<Decimal, Error> {
    Decimal::parse(text)
        .map_err(|e| Error::Build(format!("invalid decimal {text:?}: {}", e.name())))
}

fn decimal_error(e: Signal) -> Error {
    Error::Build(format!("decimal error: {}", e.name()))
}

pub fn ratio(percent: &str) -> Result<String, Error> {
    let divided = parse_decimal(percent)?
        .div(&Decimal::from_int(100))
        .map_err(decimal_error)?;
    Ok(divided.to_sci_string())
}

pub fn decimal_eq(a: &str, b: &str) -> Result<bool, Error> {
    let (da, db) = (parse_decimal(a)?, parse_decimal(b)?);
    Ok(!da.lt(&db).map_err(decimal_error)? && !da.gt(&db).map_err(decimal_error)?)
}

pub fn decimal_gt(a: &str, b: &str) -> Result<bool, Error> {
    parse_decimal(a)?
        .gt(&parse_decimal(b)?)
        .map_err(decimal_error)
}

fn is_ascii_alpha_space(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphabetic() || b == b' ')
}

fn strip_single_quotes(mut s: &str) -> &str {
    s = s.trim_matches(' ');
    if let Some(rest) = s.strip_prefix('\'') {
        s = rest;
    }
    if let Some(rest) = s.strip_suffix('\'') {
        s = rest;
    }
    s.trim_matches(' ')
}

fn parse_divided_heading(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let mut offset = 0;
    while let Some(pos) = lower[offset..].find("divided") {
        let idx = offset + pos;
        offset = idx + 1;
        let before = &line[..idx];
        if !before.ends_with(' ') {
            continue;
        }
        let left = strip_single_quotes(before);
        if !is_ascii_alpha_space(left) {
            continue;
        }
        let after = &lower[idx + 7..];
        if !after.starts_with(' ') {
            continue;
        }
        let mut rest = after.trim_start_matches(' ');
        if let Some(after_by) = rest.strip_prefix("by")
            && after_by.starts_with(' ')
        {
            rest = after_by.trim_start_matches(' ');
        }
        if strip_single_quotes(rest) == "annual turnover" {
            return Some(left.to_ascii_lowercase());
        }
    }
    None
}

pub fn parse_heading_name(line: &str) -> Option<String> {
    if let Some(name) = parse_divided_heading(line) {
        return Some(name);
    }
    let (left, right) = line.split_once('/')?;
    let left_trimmed = left.trim_end_matches(' ');
    if !is_ascii_alpha_space(left_trimmed) {
        return None;
    }
    let right_lower = right.trim_start_matches(' ').to_ascii_lowercase();
    if right_lower == "turnover" || right_lower == "annual turnover" {
        return Some(left_trimmed.to_ascii_lowercase());
    }
    None
}

pub fn heading_key(name: &str) -> Option<&'static str> {
    match name {
        "total expenses" => Some("total_expenses_to_turnover"),
        "cost of sales" => Some("cost_of_sales_to_turnover"),
        "labour" => Some("labour_to_turnover"),
        "rent" => Some("rent_to_turnover"),
        "motor vehicle expenses" => Some("motor_vehicle_to_turnover"),
        _ => None,
    }
}

fn scan_decimal_token(s: &str) -> Option<usize> {
    let mut end = 0;
    let mut has_int = false;
    for (idx, ch) in s.char_indices() {
        if decimal_value(ch).is_some() {
            has_int = true;
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    if !has_int {
        return None;
    }
    if let Some(rest) = s[end..].strip_prefix('.') {
        let mut frac_end = 0;
        for (idx, ch) in rest.char_indices() {
            if decimal_value(ch).is_some() {
                frac_end = idx + ch.len_utf8();
            } else {
                break;
            }
        }
        if frac_end > 0 {
            end += 1 + frac_end;
        }
    }
    Some(end)
}

pub fn is_average_pct(s: &str) -> bool {
    scan_decimal_token(s).is_some_and(|end| &s[end..] == "%")
}

pub fn parse_pct_range(s: &str) -> Option<(&str, &str)> {
    let end1 = scan_decimal_token(s)?;
    let num1 = &s[..end1];
    let rest = s[end1..].strip_prefix('%')?;
    if rest.is_empty() {
        return Some((num1, num1));
    }
    let after_to = rest.strip_prefix(" to ")?;
    let end2 = scan_decimal_token(after_to)?;
    if &after_to[end2..] != "%" {
        return None;
    }
    Some((num1, &after_to[..end2]))
}

fn scan_comma_digits(s: &str) -> Option<usize> {
    let mut end = 0;
    for (idx, ch) in s.char_indices() {
        if ch == ',' || decimal_value(ch).is_some() {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    (end > 0).then_some(end)
}

pub fn is_band_label(s: &str) -> bool {
    if let Some(rest) = s.strip_prefix("More than $") {
        return scan_comma_digits(rest).is_some_and(|end| end == rest.len());
    }
    let Some(rest) = s.strip_prefix('$') else {
        return false;
    };
    let Some(end1) = scan_comma_digits(rest) else {
        return false;
    };
    let mid = rest[end1..].trim_start_matches(is_space);
    let mut chars = mid.chars();
    let Some(dash) = chars.next() else {
        return false;
    };
    if !matches!(dash, '-' | '\u{2013}' | '\u{2014}') {
        return false;
    }
    let after_dash = chars.as_str().trim_start_matches(is_space);
    let Some(rest2) = after_dash.strip_prefix('$') else {
        return false;
    };
    scan_comma_digits(rest2).is_some_and(|end| end == rest2.len())
}

fn count_digits(s: &str, count: usize) -> Option<usize> {
    let mut seen = 0;
    for (idx, ch) in s.char_indices() {
        if decimal_value(ch).is_none() {
            break;
        }
        seen += 1;
        if seen == count {
            return Some(idx + ch.len_utf8());
        }
    }
    None
}

pub fn parse_year_heading(s: &str) -> Option<(&'static str, String)> {
    let (kind, rest) = if let Some(r) = s.strip_prefix("Key benchmarks for ") {
        ("key", r)
    } else {
        ("other", s.strip_prefix("Other benchmarks for ")?)
    };
    let y1_end = count_digits(rest, 4)?;
    let mut chars = rest[y1_end..].chars();
    let sep = chars.next()?;
    if !matches!(sep, '-' | '\u{2013}') {
        return None;
    }
    let after_sep = chars.as_str();
    let y2_end = count_digits(after_sep, 2)?;
    if y2_end != after_sep.len() {
        return None;
    }
    Some((
        kind,
        format!("{}-{}", &rest[..y1_end], &after_sep[..y2_end]),
    ))
}

pub fn parse_updated_line(s: &str) -> Option<&str> {
    let body = s.strip_prefix("Last updated ")?;
    let mut chars = body.chars();
    let d1 = chars.next()?;
    decimal_value(d1)?;
    let d2 = chars.next()?;
    let after_day = if decimal_value(d2).is_some() {
        if chars.next()? != ' ' {
            return None;
        }
        chars.as_str()
    } else if d2 == ' ' {
        chars.as_str()
    } else {
        return None;
    };
    let (month, year) = after_day.split_once(' ')?;
    let mut m_chars = month.chars();
    if !m_chars.next()?.is_ascii_uppercase() {
        return None;
    }
    let rest_month = m_chars.as_str();
    if rest_month.is_empty() || !rest_month.bytes().all(|b| b.is_ascii_lowercase()) {
        return None;
    }
    count_digits(year, 4).filter(|&end| end == year.len())?;
    Some(body)
}

pub fn find_financial_year(s: &str) -> Option<String> {
    const PREFIX: &str = "tax returns for the ";
    let mut offset = 0;
    while let Some(pos) = s[offset..].find(PREFIX) {
        let start = offset + pos + PREFIX.len();
        offset = offset + pos + 1;
        let tail = &s[start..];
        let Some(y1_end) = count_digits(tail, 4) else {
            continue;
        };
        let mut after_y1 = tail[y1_end..].chars();
        let Some(sep) = after_y1.next() else {
            continue;
        };
        if decimal_value(sep).is_some() {
            continue;
        }
        let tail2 = after_y1.as_str();
        let Some(y2_end) = count_digits(tail2, 2) else {
            continue;
        };
        if tail2[y2_end..].starts_with(" financial year") {
            return Some(format!("{}-{}", &tail[..y1_end], &tail2[..y2_end]));
        }
    }
    None
}

pub fn is_qc(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("QC") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| decimal_value(c).is_some())
}
