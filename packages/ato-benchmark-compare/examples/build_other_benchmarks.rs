// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Build the other benchmark ranges from the ATO's industry pages.
//!
//! Development-time script. The engine only reads its output. Save each
//! industry's "in detail" page as text into a directory holding `index.json`
//! and one text file per page, then run:
//!
//!     cargo run --example build_other_benchmarks -- \
//!         --pages-dir <saved pages> \
//!         --dataset data/benchmarks-2023-24.json \
//!         --retrieved 2026-09-29 \
//!         --out data/other-benchmarks-2023-24.json

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use ato_benchmark_compare::dataset::{canonical_digest, normalise};
use ato_benchmark_compare::decimal::Decimal;
use ato_benchmark_compare::py::{decimal_value, is_space, repr_str, split_whitespace};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

const OTHER_KEYS: [&str; 4] = [
    "cost_of_sales_to_turnover",
    "labour_to_turnover",
    "rent_to_turnover",
    "motor_vehicle_to_turnover",
];
const TITLE_SUFFIX: &str = " | Australian Taxation Office";
const MAX_EXCLUDED: usize = 5;
const ADDRESS_ALIASES: [(&str, &str); 2] = [
    (
        "motor-vehicle-retail-new-and-used",
        "Motor vehicle retail - new and used car",
    ),
    (
        "roofing-services-includes-roof-tiling-guttering-and-metal-roofing",
        "Roofing services, including roof tiling, guttering and metal roofing",
    ),
];

#[derive(Debug)]
enum Error {
    Build(String),
    Mismatch(String),
}

impl Error {
    fn message(&self) -> &str {
        match self {
            Error::Build(m) | Error::Mismatch(m) => m,
        }
    }
}

struct CliArgs {
    pages_dir: PathBuf,
    dataset: PathBuf,
    retrieved: String,
    out: PathBuf,
    expect_excluded: BTreeSet<String>,
}

enum CliAction {
    Help,
    Run(CliArgs),
}

struct ParsedPage {
    sections: BTreeMap<String, ParsedSection>,
    financial_year: String,
    last_updated: String,
    qc: String,
}

struct ParsedSection {
    year: String,
    labels: Vec<String>,
    ranges: BTreeMap<String, Vec<Option<(String, String)>>>,
}

fn usage() -> &'static str {
    "usage: build_other_benchmarks [-h] --pages-dir PAGES_DIR --dataset DATASET \
     --retrieved RETRIEVED --out OUT [--expect-excluded BUSINESS_TYPE]"
}

fn help_text() -> String {
    format!(
        "{}\n\nBuild the other benchmark ranges from the ATO's industry pages.\n\noptions:\n  \
         -h, --help            show this help message and exit\n  \
         --pages-dir PAGES_DIR\n  \
         --dataset DATASET\n  \
         --retrieved RETRIEVED\n                        ISO date the pages were saved\n  \
         --out OUT\n  \
         --expect-excluded BUSINESS_TYPE\n                        \
         a business type whose page is known to disagree; repeat for each one",
        usage()
    )
}

fn cli_err(msg: &str) -> String {
    format!("{}\nbuild_other_benchmarks: error: {msg}", usage())
}

fn next_flag_value(
    flag: &str,
    inline: Option<&str>,
    args: &[String],
    index: &mut usize,
) -> Result<String, String> {
    match inline {
        Some(v) => Ok(v.to_string()),
        None => {
            *index += 1;
            args.get(*index)
                .cloned()
                .ok_or_else(|| cli_err(&format!("argument {flag}: expected one argument")))
        }
    }
}

fn parse_cli(args: &[String]) -> Result<CliAction, String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        return Ok(CliAction::Help);
    }
    let (mut pages_dir, mut dataset, mut retrieved, mut out) = (None, None, None, None);
    let mut expect_excluded = BTreeSet::new();
    let mut i = 0;
    while i < args.len() {
        let raw = &args[i];
        let (flag, inline) = raw
            .split_once('=')
            .map_or((raw.as_str(), None), |(f, v)| (f, Some(v)));
        if matches!(
            flag,
            "--pages-dir" | "--dataset" | "--retrieved" | "--out" | "--expect-excluded"
        ) {
            if inline == Some("") && flag != "--retrieved" && flag != "--expect-excluded" {
                return Err(cli_err(&format!("argument {flag}: expected one argument")));
            }
            let val = next_flag_value(flag, inline, args, &mut i)?;
            match flag {
                "--pages-dir" => pages_dir = Some(PathBuf::from(val)),
                "--dataset" => dataset = Some(PathBuf::from(val)),
                "--retrieved" => retrieved = Some(val),
                "--out" => out = Some(PathBuf::from(val)),
                "--expect-excluded" => {
                    expect_excluded.insert(val);
                }
                _ => {}
            }
        } else {
            return Err(cli_err(&format!("unrecognized arguments: {raw}")));
        }
        i += 1;
    }
    let mut missing = Vec::new();
    if pages_dir.is_none() {
        missing.push("--pages-dir");
    }
    if dataset.is_none() {
        missing.push("--dataset");
    }
    if retrieved.is_none() {
        missing.push("--retrieved");
    }
    if out.is_none() {
        missing.push("--out");
    }
    if !missing.is_empty() {
        return Err(cli_err(&format!(
            "the following arguments are required: {}",
            missing.join(", ")
        )));
    }
    Ok(CliAction::Run(CliArgs {
        pages_dir: pages_dir.unwrap(),
        dataset: dataset.unwrap(),
        retrieved: retrieved.unwrap(),
        out: out.unwrap(),
        expect_excluded,
    }))
}

fn write_json_indented(value: &Value, level: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => out.push_str(&serde_json::to_string(s).expect("JSON string")),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.extend(std::iter::repeat_n(' ', level + 1));
                write_json_indented(item, level + 1, out);
            }
            out.push('\n');
            out.extend(std::iter::repeat_n(' ', level));
            out.push(']');
        }
        Value::Object(map) if map.is_empty() => out.push_str("{}"),
        Value::Object(map) => {
            out.push('{');
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.extend(std::iter::repeat_n(' ', level + 1));
                out.push_str(&serde_json::to_string(k).expect("JSON key"));
                out.push_str(": ");
                write_json_indented(v, level + 1, out);
            }
            out.push('\n');
            out.extend(std::iter::repeat_n(' ', level));
            out.push('}');
        }
    }
}

fn to_json_indented(value: &Value) -> String {
    let mut out = String::new();
    write_json_indented(value, 0, &mut out);
    out.push('\n');
    out
}

fn py_str_splitlines(text: &str) -> Vec<&str> {
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

fn collapse_spaces(line: &str) -> String {
    split_whitespace(line).join(" ")
}

fn py_repr_list(items: &[String]) -> String {
    let joined = items
        .iter()
        .map(|s| repr_str(s))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

fn py_repr_pair(a: &str, b: &str) -> String {
    format!("({}, {})", repr_str(a), repr_str(b))
}

fn py_repr_obj(val: &Value) -> String {
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

fn slug(text: &str) -> String {
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

fn label_key(label: &str) -> String {
    label
        .replace(['\u{2013}', '\u{2014}'], "-")
        .chars()
        .filter(|c| !is_space(*c))
        .collect::<String>()
        .to_lowercase()
}

fn ratio(percent: &str) -> Result<String, Error> {
    let dec = Decimal::parse(percent)
        .map_err(|e| Error::Build(format!("invalid decimal {percent:?}: {}", e.name())))?;
    let divided = dec
        .div(&Decimal::from_int(100))
        .map_err(|e| Error::Build(format!("decimal error: {}", e.name())))?;
    Ok(divided.to_sci_string())
}

fn decimal_eq(a: &str, b: &str) -> Result<bool, Error> {
    let da = Decimal::parse(a)
        .map_err(|e| Error::Build(format!("invalid decimal {a:?}: {}", e.name())))?;
    let db = Decimal::parse(b)
        .map_err(|e| Error::Build(format!("invalid decimal {b:?}: {}", e.name())))?;
    let lt = da
        .lt(&db)
        .map_err(|e| Error::Build(format!("decimal error: {}", e.name())))?;
    let gt = da
        .gt(&db)
        .map_err(|e| Error::Build(format!("decimal error: {}", e.name())))?;
    Ok(!lt && !gt)
}

fn decimal_gt(a: &str, b: &str) -> Result<bool, Error> {
    let da = Decimal::parse(a)
        .map_err(|e| Error::Build(format!("invalid decimal {a:?}: {}", e.name())))?;
    let db = Decimal::parse(b)
        .map_err(|e| Error::Build(format!("invalid decimal {b:?}: {}", e.name())))?;
    da.gt(&db)
        .map_err(|e| Error::Build(format!("decimal error: {}", e.name())))
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

fn parse_heading_name(line: &str) -> Option<String> {
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

fn heading_key(name: &str) -> Option<&'static str> {
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

fn is_average_pct(s: &str) -> bool {
    scan_decimal_token(s).is_some_and(|end| &s[end..] == "%")
}

fn parse_pct_range(s: &str) -> Option<(&str, &str)> {
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

fn is_band_label(s: &str) -> bool {
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

fn parse_year_heading(s: &str) -> Option<(&'static str, String)> {
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

fn parse_updated_line(s: &str) -> Option<&str> {
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

fn find_financial_year(s: &str) -> Option<String> {
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

fn is_qc(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("QC") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| decimal_value(c).is_some())
}

fn parse_ratio_block(
    lines: &[String],
    i: &mut usize,
    band_count: usize,
    key: &str,
    where_: &str,
) -> Result<Vec<Option<(String, String)>>, Error> {
    let mut values = Vec::new();
    while *i < lines.len() && values.len() < band_count {
        let line = &lines[*i];
        let Some((p_low, p_high)) = parse_pct_range(line) else {
            return Err(Error::Build(format!(
                "{where_}: expected a range such as '22% to 38%' under {key}, found {}",
                repr_str(line)
            )));
        };
        let low = ratio(p_low)?;
        let high = ratio(p_high)?;
        if decimal_gt(&low, &high)? {
            return Err(Error::Build(format!(
                "{where_}: {key} range {} runs backwards",
                repr_str(line)
            )));
        }
        values.push(Some((low, high)));
        *i += 1;
    }
    if values.len() != band_count {
        return Err(Error::Build(format!(
            "{where_}: {key} has {} ranges for {band_count} bands",
            values.len()
        )));
    }
    Ok(values)
}

type RangeMap = BTreeMap<String, Vec<Option<(String, String)>>>;
type SectionData = (Vec<String>, RangeMap);

fn parse_section(lines: &[String], start: usize, where_: &str) -> Result<SectionData, Error> {
    let mut i = start + 1;
    while i < lines.len() && lines[i] != "Annual turnover range" {
        if parse_year_heading(&lines[i]).is_some() {
            return Err(Error::Build(format!(
                "{where_}: the next table starts before this one's bands"
            )));
        }
        i += 1;
    }
    if i == lines.len() {
        return Err(Error::Build(format!(
            "{where_}: no 'Annual turnover range' after the year heading"
        )));
    }
    i += 1;
    let mut labels = Vec::new();
    while i < lines.len() && is_band_label(&lines[i]) {
        labels.push(lines[i].clone());
        i += 1;
    }
    if labels.is_empty() {
        return Err(Error::Build(format!("{where_}: no turnover band labels")));
    }
    let mut ranges: BTreeMap<String, Vec<Option<(String, String)>>> = BTreeMap::new();
    while i < lines.len() {
        let Some(name) = parse_heading_name(&lines[i]) else {
            let tail = &lines[i + 1..];
            if lines[i].starts_with("Average ")
                && tail.len() >= labels.len()
                && tail[..labels.len()].iter().all(|v| is_average_pct(v))
            {
                i += 1 + labels.len();
                continue;
            }
            if tail.len() >= labels.len()
                && tail[..labels.len()]
                    .iter()
                    .all(|v| parse_pct_range(v).is_some())
            {
                return Err(Error::Build(format!(
                    "{where_}: unrecognised benchmark heading {}",
                    repr_str(&lines[i])
                )));
            }
            break;
        };
        let Some(key) = heading_key(&name) else {
            return Err(Error::Build(format!(
                "{where_}: unrecognised benchmark heading {}",
                repr_str(&lines[i])
            )));
        };
        if ranges.contains_key(key) {
            return Err(Error::Build(format!(
                "{where_}: {} appears twice",
                repr_str(&lines[i])
            )));
        }
        i += 1;
        let values = parse_ratio_block(lines, &mut i, labels.len(), key, where_)?;
        ranges.insert(key.to_string(), values);
    }
    if ranges.is_empty() {
        return Err(Error::Build(format!(
            "{where_}: no benchmark ranges under the year heading"
        )));
    }
    Ok((labels, ranges))
}

fn parse_page(raw_text: &str, where_: &str) -> Result<ParsedPage, Error> {
    let text = raw_text.replace(['\u{2018}', '\u{2019}', '\u{00b4}', '\u{0060}'], "'");
    let lines: Vec<String> = py_str_splitlines(&text)
        .into_iter()
        .map(collapse_spaces)
        .filter(|line| !line.is_empty())
        .collect();
    let mut sections = BTreeMap::new();
    let (mut updated, mut qc, mut financial_year) = (None, None, None);
    for (index, line) in lines.iter().enumerate() {
        if financial_year.is_none()
            && let Some(fy) = find_financial_year(line)
        {
            financial_year = Some(fy);
        }
        if let Some((kind, value)) = parse_year_heading(line) {
            if sections.contains_key(kind) {
                return Err(Error::Build(format!(
                    "{where_}: two {kind} benchmark tables"
                )));
            }
            let (labels, ranges) = parse_section(&lines, index, &format!("{where_} ({kind})"))?;
            sections.insert(
                kind.to_string(),
                ParsedSection {
                    year: value,
                    labels,
                    ranges,
                },
            );
        } else if updated.is_none()
            && let Some(u) = parse_updated_line(line)
        {
            updated = Some(u.to_string());
        } else if is_qc(line) {
            qc = Some(line.clone());
        }
    }
    if !sections.contains_key("key") {
        return Err(Error::Build(format!("{where_}: no key benchmark table")));
    }
    let Some(financial_year) = financial_year else {
        return Err(Error::Build(format!(
            "{where_}: the page does not state which financial year it covers"
        )));
    };
    let (Some(last_updated), Some(qc)) = (updated, qc) else {
        return Err(Error::Build(format!(
            "{where_}: the page carries no last-updated date or QC reference"
        )));
    };
    Ok(ParsedPage {
        sections,
        financial_year,
        last_updated,
        qc,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn check_section_agreements(
    page: &ParsedPage,
    bands: &[Value],
    year: &str,
    where_: &str,
) -> Result<Vec<String>, Error> {
    let mut anomalies = Vec::new();
    let expected: Vec<String> = bands
        .iter()
        .map(|b| label_key(b["label"].as_str().unwrap_or_default()))
        .collect();
    let expected_labels: Vec<String> = bands
        .iter()
        .map(|b| b["label"].as_str().unwrap_or_default().to_string())
        .collect();
    for (kind, section) in &page.sections {
        if section.year != year {
            if kind != "key" {
                return Err(Error::Mismatch(format!(
                    "{where_}: {kind} benchmarks are for {}, dataset is {year}",
                    section.year
                )));
            }
            anomalies.push(format!(
                "key table heading says {}; the page and its ranges are {year}",
                section.year
            ));
        }
        let printed: Vec<String> = section.labels.iter().map(|l| label_key(l)).collect();
        if printed != expected {
            return Err(Error::Mismatch(format!(
                "{where_}: {kind} bands {} do not match the dataset's {}",
                py_repr_list(&section.labels),
                py_repr_list(&expected_labels)
            )));
        }
    }
    Ok(anomalies)
}

fn check_dataset_ranges(
    page: &ParsedPage,
    business_type: &Value,
    where_: &str,
) -> Result<(), Error> {
    let bands = business_type["turnover_bands"]
        .as_array()
        .ok_or_else(|| Error::Build("missing turnover_bands".into()))?;
    let mut printed_keys = BTreeSet::new();
    for section in page.sections.values() {
        for key in section.ranges.keys() {
            printed_keys.insert(key.as_str());
        }
    }
    for band in bands {
        for key in ["cost_of_sales_to_turnover", "total_expenses_to_turnover"] {
            if !band[key].is_null() && !printed_keys.contains(key) {
                return Err(Error::Mismatch(format!(
                    "{where_}: the page does not print the dataset's {key} range"
                )));
            }
        }
    }
    for section in page.sections.values() {
        for (key, values) in &section.ranges {
            for (band, value) in bands.iter().zip(values.iter()) {
                let published = &band[key.as_str()];
                let Some((low, high)) = value else {
                    continue;
                };
                if published.is_null() {
                    continue;
                }
                let pub_min = published["min"].as_str().unwrap_or_default();
                let pub_max = published["max"].as_str().unwrap_or_default();
                if !decimal_eq(pub_min, low)? || !decimal_eq(pub_max, high)? {
                    return Err(Error::Mismatch(format!(
                        "{where_}: {key} for {} is {} on the page but {} in the dataset",
                        band["band"].as_str().unwrap_or_default(),
                        py_repr_pair(low, high),
                        py_repr_obj(published)
                    )));
                }
            }
        }
    }
    let key_ratio = business_type["key_ratio"].as_str().unwrap_or_default();
    if !page.sections["key"].ranges.contains_key(key_ratio) {
        return Err(Error::Mismatch(format!(
            "{where_}: the key table does not carry {key_ratio}"
        )));
    }
    Ok(())
}

fn page_entry(
    item: &Value,
    business_type: &Value,
    page_name: &str,
    raw: &[u8],
    year: &str,
) -> Result<Value, Error> {
    let where_ = item["final_url"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| item["url"].as_str())
        .ok_or_else(|| Error::Build("index item missing url".into()))?;
    let text = std::str::from_utf8(raw)
        .map_err(|e| Error::Build(format!("{where_}: invalid UTF-8: {e}")))?;
    let page = parse_page(text, where_)?;
    if page.financial_year != year {
        return Err(Error::Mismatch(format!(
            "{where_}: the page covers {}, dataset is {year}",
            page.financial_year
        )));
    }
    let bands = business_type["turnover_bands"]
        .as_array()
        .ok_or_else(|| Error::Build("missing turnover_bands".into()))?;
    let anomalies = check_section_agreements(&page, bands, year, where_)?;
    check_dataset_ranges(&page, business_type, where_)?;

    let empty_other = BTreeMap::new();
    let other = page
        .sections
        .get("other")
        .map_or(&empty_other, |s| &s.ranges);
    let mut out_bands = Vec::with_capacity(bands.len());
    for (position, band) in bands.iter().enumerate() {
        let mut entry = Map::new();
        entry.insert("band".into(), band["band"].clone());
        for key in OTHER_KEYS {
            let cell = other
                .get(key)
                .and_then(|vals| vals.get(position))
                .and_then(Option::as_ref);
            let value = match cell {
                Some((low, high)) if band[key].is_null() => json!({"min": low, "max": high}),
                _ => Value::Null,
            };
            entry.insert(key.into(), value);
        }
        out_bands.push(Value::Object(entry));
    }
    Ok(json!({
        "name": business_type["name"],
        "page_title": page_name,
        "page_url": where_,
        "page_last_updated": page.last_updated,
        "page_reference": page.qc,
        "text_sha256": sha256_hex(raw),
        "page_anomalies": anomalies,
        "turnover_bands": out_bands,
    }))
}

fn json_truthy(val: &Value) -> bool {
    match val {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn item_where(item: &Value) -> Result<&str, Error> {
    item["final_url"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| item["url"].as_str())
        .ok_or_else(|| Error::Build("index item missing url".into()))
}

fn resolve_business_type(
    item: &Value,
    year: &str,
    by_name: &HashMap<String, usize>,
    by_slug: &HashMap<String, usize>,
) -> Result<(usize, String), Error> {
    let where_ = item_where(item)?;
    let status = &item["status"];
    if status.as_u64() != Some(200)
        || json_truthy(&item["error"])
        || json_truthy(&item["truncated"])
    {
        let shown = if status.is_null() {
            "None".to_string()
        } else {
            status.to_string()
        };
        return Err(Error::Build(format!(
            "{where_}: page was not saved whole (status {shown})"
        )));
    }
    let title = item["title"].as_str().unwrap_or("");
    let Some(stripped) = title.strip_suffix(TITLE_SUFFIX) else {
        return Err(Error::Build(format!(
            "{where_}: unexpected page title {}",
            repr_str(title)
        )));
    };
    let page_name = stripped.trim_matches(is_space).to_string();
    let address = where_
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("");
    let alias_norm = ADDRESS_ALIASES
        .iter()
        .find(|(k, _)| *k == address)
        .map(|(_, v)| normalise(v))
        .unwrap_or_default();
    let from_address = by_slug
        .get(address)
        .or_else(|| by_name.get(&alias_norm))
        .copied();
    let from_title = by_name.get(&normalise(&page_name)).copied();
    if let (Some(a), Some(t)) = (from_address, from_title)
        && a != t
    {
        return Err(Error::Build(format!(
            "{where_}: address and title name different business types"
        )));
    }
    let idx = from_address.or(from_title).ok_or_else(|| {
        Error::Build(format!(
            "{where_}: {} is not a business type in {year}",
            repr_str(&page_name)
        ))
    })?;
    Ok((idx, page_name))
}

fn validate_exclusions(
    excluded: &[Value],
    expect_excluded: &BTreeSet<String>,
) -> Result<(), Error> {
    let actual: BTreeSet<String> = excluded
        .iter()
        .map(|item| item["name"].as_str().unwrap_or_default().to_string())
        .collect();
    if &actual != expect_excluded {
        let reasons: Vec<&str> = excluded
            .iter()
            .filter(|item| !expect_excluded.contains(item["name"].as_str().unwrap_or_default()))
            .map(|item| item["reason"].as_str().unwrap_or_default())
            .collect();
        let actual_vec: Vec<String> = actual.into_iter().collect();
        let expect_vec: Vec<String> = expect_excluded.iter().cloned().collect();
        return Err(Error::Build(format!(
            "excluded pages {} differ from --expect-excluded {}. {}",
            py_repr_list(&actual_vec),
            py_repr_list(&expect_vec),
            reasons.join("; ")
        )));
    }
    if excluded.len() > MAX_EXCLUDED {
        let reasons: Vec<&str> = excluded
            .iter()
            .take(3)
            .map(|item| item["reason"].as_str().unwrap_or_default())
            .collect();
        return Err(Error::Build(format!(
            "{} pages disagree with the dataset: {}",
            excluded.len(),
            reasons.join("; ")
        )));
    }
    Ok(())
}

fn build(
    pages_dir: &std::path::Path,
    dataset_path: &std::path::Path,
    retrieved: &str,
    expect_excluded: &BTreeSet<String>,
) -> Result<Value, Error> {
    let dataset_text = fs::read_to_string(dataset_path)
        .map_err(|e| Error::Build(format!("{}: {e}", dataset_path.display())))?;
    let dataset: Value = serde_json::from_str(&dataset_text)
        .map_err(|e| Error::Build(format!("{}: {e}", dataset_path.display())))?;
    let year = dataset["benchmark_year"]
        .as_str()
        .ok_or_else(|| Error::Build("dataset missing benchmark_year".into()))?;
    let business_types = dataset["business_types"]
        .as_array()
        .ok_or_else(|| Error::Build("dataset missing business_types".into()))?;
    let mut by_name = HashMap::new();
    let mut by_slug = HashMap::new();
    for (idx, bt) in business_types.iter().enumerate() {
        let name = bt["name"].as_str().unwrap_or_default();
        by_name.insert(normalise(name), idx);
        by_slug.insert(slug(name), idx);
    }
    if by_slug.len() != business_types.len() {
        return Err(Error::Build(
            "two business types share a page address".into(),
        ));
    }
    let index_path = pages_dir.join("index.json");
    let index_text = fs::read_to_string(&index_path)
        .map_err(|e| Error::Build(format!("{}: {e}", index_path.display())))?;
    let index: Vec<Value> = serde_json::from_str(&index_text)
        .map_err(|e| Error::Build(format!("{}: {e}", index_path.display())))?;

    let (mut entries, mut excluded, mut seen) = (Vec::new(), Vec::new(), BTreeSet::new());
    for item in &index {
        let (idx, page_name) = resolve_business_type(item, year, &by_name, &by_slug)?;
        let bt = &business_types[idx];
        let name = bt["name"].as_str().unwrap_or_default().to_string();
        let where_ = item_where(item)?;
        if !seen.insert(name.clone()) {
            return Err(Error::Build(format!(
                "{where_}: {} has a second page",
                repr_str(&name)
            )));
        }
        let file_rel = item["file"]
            .as_str()
            .ok_or_else(|| Error::Build(format!("{where_}: index item missing file")))?;
        let raw = fs::read(pages_dir.join(file_rel))
            .map_err(|e| Error::Build(format!("{}: {e}", pages_dir.join(file_rel).display())))?;
        match page_entry(item, bt, &page_name, &raw, year) {
            Ok(entry) => entries.push(entry),
            Err(Error::Mismatch(reason)) => excluded.push(json!({
                "name": name,
                "page_url": where_,
                "reason": reason,
            })),
            Err(err @ Error::Build(_)) => return Err(err),
        }
    }
    let missing: Vec<String> = business_types
        .iter()
        .filter_map(|bt| bt["name"].as_str())
        .filter(|name| !seen.contains(*name))
        .map(str::to_string)
        .collect();
    if !missing.is_empty() {
        let mut sorted = missing;
        sorted.sort();
        let preview: Vec<String> = sorted.iter().take(5).cloned().collect();
        return Err(Error::Build(format!(
            "no page for {} business types: {}",
            sorted.len(),
            py_repr_list(&preview)
        )));
    }
    validate_exclusions(&excluded, expect_excluded)?;
    entries.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    excluded.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    let json_digest =
        canonical_digest(&dataset_text).map_err(|e| Error::Build(format!("{e:?}")))?;
    Ok(json!({
        "schema_version": 1,
        "benchmark_year": year,
        "business_type_count": entries.len(),
        "source": {
            "publisher": "Australian Taxation Office",
            "pages": "Small business benchmarks, industry pages (in detail)",
            "index_url": "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/small-business-benchmarks/benchmarks-a-z",
            "retrieved": retrieved,
            "dataset_sha256": dataset["source"]["sha256"],
            "dataset_json_sha256": json_digest,
            "licence": "ATO copyright notice: free to copy, adapt, modify, transmit and distribute, but not in any way that suggests the ATO or the Commonwealth endorses the user or its products",
            "licence_url": "https://www.ato.gov.au/about-ato/using-our-website/copyright-notice",
            "caution": "Not all expenses are reported by every business. The ATO says to use these ranges only as a guide where they apply to the business.",
        },
        "business_types": entries,
        "excluded_pages": excluded,
    }))
}

fn count_filled_ranges(built: &Value) -> usize {
    built["business_types"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|e| e["turnover_bands"].as_array().into_iter().flatten())
        .flat_map(|band| OTHER_KEYS.iter().map(move |k| &band[*k]))
        .filter(|v| !v.is_null())
        .count()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match parse_cli(&args) {
        Ok(CliAction::Help) => {
            println!("{}", help_text());
            return ExitCode::SUCCESS;
        }
        Ok(CliAction::Run(cli)) => cli,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    let built = match build(
        &cli.pages_dir,
        &cli.dataset,
        &cli.retrieved,
        &cli.expect_excluded,
    ) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: {}", err.message());
            return ExitCode::from(1);
        }
    };
    let text = to_json_indented(&built);
    if let Err(e) = fs::write(&cli.out, text.as_bytes()) {
        eprintln!("error: {}: {e}", cli.out.display());
        return ExitCode::from(1);
    }
    let count = built["business_type_count"].as_u64().unwrap_or(0);
    println!(
        "wrote {}: {count} business types, {} ranges",
        cli.out.display(),
        count_filled_ranges(&built)
    );
    for item in built["excluded_pages"].as_array().into_iter().flatten() {
        eprintln!("excluded: {}", item["reason"].as_str().unwrap_or_default());
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const ADDRESS: &str = "https://www.ato.gov.au/businesses-and-organisations/\
        income-deductions-and-concessions/small-business-benchmarks/in-detail/fabricated-services";

    fn sample_page() -> String {
        [
            "Performance benchmarks use information reported on tax returns for the 2023\u{2013}24 financial year and are updated each year.",
            "Last updated 16 March 2026",
            "Key benchmarks for 2023\u{2013}24",
            "Annual turnover range",
            "$50,000 \u{2013} $150,000",
            "More than $150,000",
            "'Total expenses' divided 'Annual turnover'",
            "54%\u{00a0}to\u{00a0}73%",
            "79%\u{00a0}to\u{00a0}89%",
            "Average total expenses",
            "63%",
            "Other benchmarks for 2023\u{2013}24",
            "Annual turnover range",
            "$50,000 \u{2013} $150,000",
            "More than $150,000",
            "\u{2018}Labour\u{2019} divided by \u{2018}Annual turnover\u{2019}",
            "20% to 30%",
            "25% to 35%",
            "Rent/turnover",
            "0% to 1%",
            "1%",
            "QC12345",
        ]
        .join("\n")
    }

    fn fab_dataset() -> Value {
        json!({
            "schema_version": 1,
            "benchmark_year": "2023-24",
            "business_type_count": 1,
            "source": {"sha256": "ab".repeat(32)},
            "business_types": [{
                "name": "Fabricated services",
                "key_ratio": "total_expenses_to_turnover",
                "turnover_bands": [
                    {
                        "band": "low", "label": "$50,000 - $150,000", "turnover_from": "50000",
                        "turnover_from_inclusive": true, "turnover_to": "150000",
                        "total_expenses_to_turnover": {"min": "0.54", "max": "0.73"},
                        "cost_of_sales_to_turnover": null,
                    },
                    {
                        "band": "high", "label": "More than $150,000", "turnover_from": "150000",
                        "turnover_from_inclusive": false, "turnover_to": null,
                        "total_expenses_to_turnover": {"min": "0.79", "max": "0.89"},
                        "cost_of_sales_to_turnover": null,
                    },
                ],
            }],
        })
    }

    fn temp_dir(label: &str) -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "ato-other-builder-{label}-{}-{n}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn run_build(label: &str, page: &str, expect: &[&str]) -> (PathBuf, Result<Value, Error>) {
        let dir = temp_dir(label);
        let pages = dir.join("pages");
        fs::create_dir_all(&pages).unwrap();
        fs::write(pages.join("001.txt"), page.as_bytes()).unwrap();
        fs::write(
            pages.join("index.json"),
            serde_json::to_string(&json!([{
                "url": ADDRESS,
                "final_url": ADDRESS,
                "file": "001.txt",
                "status": 200,
                "truncated": false,
                "title": "Fabricated services | Australian Taxation Office"
            }]))
            .unwrap(),
        )
        .unwrap();
        let dataset_path = dir.join("benchmarks-2023-24.json");
        fs::write(
            &dataset_path,
            serde_json::to_string(&fab_dataset()).unwrap(),
        )
        .unwrap();
        let expected: BTreeSet<String> = expect.iter().map(|s| (*s).to_string()).collect();
        let res = build(&pages, &dataset_path, "2026-09-29", &expected);
        (dir, res)
    }

    #[test]
    fn builder_reads_every_layout_the_pages_use() {
        let page = parse_page(&sample_page(), "fabricated").unwrap();
        assert_eq!(page.financial_year, "2023-24");
        assert_eq!(page.last_updated, "16 March 2026");
        assert_eq!(page.qc, "QC12345");
        assert_eq!(
            page.sections["key"].ranges["total_expenses_to_turnover"],
            vec![
                Some(("0.54".into(), "0.73".into())),
                Some(("0.79".into(), "0.89".into())),
            ]
        );
        let other = &page.sections["other"].ranges;
        assert_eq!(
            other["labour_to_turnover"],
            vec![
                Some(("0.2".into(), "0.3".into())),
                Some(("0.25".into(), "0.35".into())),
            ]
        );
        assert_eq!(
            other["rent_to_turnover"],
            vec![
                Some(("0".into(), "0.01".into())),
                Some(("0.01".into(), "0.01".into())),
            ]
        );
    }

    #[test]
    fn builder_refuses_a_range_it_cannot_read() {
        let bad = sample_page().replace("20% to 30%", "20 to 30 per cent");
        let err = parse_page(&bad, "fabricated").err().unwrap();
        assert!(
            err.message().contains("expected a range"),
            "{}",
            err.message()
        );
    }

    #[test]
    fn builder_includes_a_page_that_agrees() {
        let (dir, res) = run_build("agree", &sample_page(), &[]);
        let built = res.unwrap();
        assert_eq!(built["excluded_pages"], json!([]));
        let entry = &built["business_types"][0];
        assert_eq!(entry["page_reference"], "QC12345");
        assert_eq!(
            entry["turnover_bands"][1]["rent_to_turnover"],
            json!({"min": "0.01", "max": "0.01"})
        );
        assert_eq!(
            built["source"]["dataset_json_sha256"],
            canonical_digest(&serde_json::to_string(&fab_dataset()).unwrap()).unwrap()
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn builder_leaves_out_a_page_that_disagrees() {
        let cases = [
            (
                "54%\u{00a0}to\u{00a0}73%",
                "55%\u{00a0}to\u{00a0}73%",
                "in the dataset",
            ),
            (
                "'Total expenses' divided 'Annual turnover'",
                "'Cost of sales' divided by 'Annual turnover'",
                "does not print the dataset's total_expenses_to_turnover",
            ),
            (
                "Other benchmarks for 2023\u{2013}24",
                "Other benchmarks for 2022\u{2013}23",
                "other benchmarks are for 2022-23",
            ),
            (
                "the 2023\u{2013}24 financial year",
                "the 2022\u{2013}23 financial year",
                "the page covers 2022-23",
            ),
        ];
        for (idx, (from, to, expected_reason)) in cases.into_iter().enumerate() {
            let (dir, res) = run_build(
                &format!("disagree-{idx}"),
                &sample_page().replace(from, to),
                &["Fabricated services"],
            );
            let built = res.unwrap();
            assert_eq!(built["business_types"], json!([]));
            let reason = built["excluded_pages"][0]["reason"].as_str().unwrap();
            assert!(reason.contains(expected_reason), "{reason}");
            let _ = fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn unexpected_exclusion_stops_the_build() {
        let (dir, res) = run_build(
            "unexpected-excl",
            &sample_page().replace("54%\u{00a0}to\u{00a0}73%", "55%\u{00a0}to\u{00a0}73%"),
            &[],
        );
        let msg = res.unwrap_err().message().to_string();
        assert!(msg.contains("differ from --expect-excluded"), "{msg}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn mistyped_key_heading_is_recorded_when_its_ranges_agree() {
        let (dir, res) = run_build(
            "mistyped-key",
            &sample_page().replace(
                "Key benchmarks for 2023\u{2013}24",
                "Key benchmarks for 2023\u{2013}23",
            ),
            &[],
        );
        let built = res.unwrap();
        assert_eq!(
            built["business_types"][0]["page_anomalies"],
            json!(["key table heading says 2023-23; the page and its ranges are 2023-24"])
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn builder_reads_a_second_key_range_after_its_averages() {
        let page = sample_page().replace(
            "Average total expenses\n63%\n",
            "Average total expenses\n63%\n70%\n'Cost of sales' divided by 'Annual turnover'\n10% to 20%\n15% to 25%\n",
        );
        let parsed = parse_page(&page, "fabricated").unwrap();
        assert_eq!(
            parsed.sections["key"].ranges["cost_of_sales_to_turnover"],
            vec![
                Some(("0.1".into(), "0.2".into())),
                Some(("0.15".into(), "0.25".into())),
            ]
        );
    }

    #[test]
    fn page_without_reference_and_unrecognised_headings_are_refused() {
        let no_qc = parse_page(&sample_page().replace("QC12345", ""), "fabricated")
            .err()
            .unwrap();
        assert!(
            no_qc.message().contains("QC reference"),
            "{}",
            no_qc.message()
        );

        let bad_heading = parse_page(
            &sample_page().replace("Rent/turnover", "Rent as a share of turnover"),
            "fabricated",
        )
        .err()
        .unwrap();
        assert!(
            bad_heading
                .message()
                .contains("unrecognised benchmark heading"),
            "{}",
            bad_heading.message()
        );
    }

    #[test]
    fn shipped_page_file_digest_and_layout_match_canonical_and_indent_one() {
        let dataset = include_str!("../data/benchmarks-2023-24.json");
        let pages_raw = include_str!("../data/other-benchmarks-2023-24.json");
        let pages: Value = serde_json::from_str(pages_raw).unwrap();
        assert_eq!(
            canonical_digest(dataset).unwrap(),
            pages["source"]["dataset_json_sha256"].as_str().unwrap()
        );
        assert_eq!(to_json_indented(&pages), pages_raw);
    }

    #[test]
    fn builder_folds_names_as_engine_does() {
        let cases = [
            ("Fish and chips shops", "fish and chips shops"),
            (
                "Bakeries \u{2013} and \u{2018}hot\u{2019} bread",
                "bakeries and hot bread",
            ),
            ("Caf\u{00e9}s", "cafe\u{0301}s"),
            ("STRASSE \u{00df}", "strasse ss"),
            ("  spaced\tout  ", "spaced out"),
        ];
        for (text, folded) in cases {
            assert_eq!(normalise(text), folded);
        }
    }
}
