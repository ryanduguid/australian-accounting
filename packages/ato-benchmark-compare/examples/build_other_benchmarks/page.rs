// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! One saved industry page parsed into its benchmark tables.

use std::collections::BTreeMap;

use ato_benchmark_compare::py::repr_str;

use crate::text::{
    Error, collapse_spaces, decimal_gt, find_financial_year, heading_key, is_average_pct,
    is_band_label, is_qc, parse_heading_name, parse_pct_range, parse_updated_line,
    parse_year_heading, py_str_splitlines, ratio,
};

pub struct ParsedPage {
    pub sections: BTreeMap<String, ParsedSection>,
    pub financial_year: String,
    pub last_updated: String,
    pub qc: String,
}

pub struct ParsedSection {
    pub year: String,
    pub labels: Vec<String>,
    pub ranges: BTreeMap<String, Vec<Option<(String, String)>>>,
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

fn build_error(where_: &str, what: &str) -> Error {
    Error::Build(format!("{where_}: {what}"))
}

/// The index just past the table's `Annual turnover range` line.
fn find_band_header(lines: &[String], start: usize, where_: &str) -> Result<usize, Error> {
    let mut i = start + 1;
    while i < lines.len() && lines[i] != "Annual turnover range" {
        if parse_year_heading(&lines[i]).is_some() {
            return Err(build_error(
                where_,
                "the next table starts before this one's bands",
            ));
        }
        i += 1;
    }
    if i == lines.len() {
        return Err(build_error(
            where_,
            "no 'Annual turnover range' after the year heading",
        ));
    }
    Ok(i + 1)
}

/// A line that is not a benchmark heading: `Some(next)` skips an `Average`
/// block, `None` ends the table, and a line followed by a row of ranges is an
/// unrecognised heading.
fn skip_non_heading(
    lines: &[String],
    i: usize,
    bands: usize,
    where_: &str,
) -> Result<Option<usize>, Error> {
    let tail = &lines[i + 1..];
    if tail.len() < bands {
        return Ok(None);
    }
    if lines[i].starts_with("Average ") && tail[..bands].iter().all(|v| is_average_pct(v)) {
        return Ok(Some(i + 1 + bands));
    }
    if tail[..bands].iter().all(|v| parse_pct_range(v).is_some()) {
        return Err(build_error(
            where_,
            &format!("unrecognised benchmark heading {}", repr_str(&lines[i])),
        ));
    }
    Ok(None)
}

/// The band labels and each heading's ranges of the table whose year heading
/// is `lines[start]`.
fn parse_section(
    lines: &[String],
    start: usize,
    where_: &str,
) -> Result<(Vec<String>, RangeMap), Error> {
    let mut i = find_band_header(lines, start, where_)?;
    let mut labels = Vec::new();
    while i < lines.len() && is_band_label(&lines[i]) {
        labels.push(lines[i].clone());
        i += 1;
    }
    if labels.is_empty() {
        return Err(build_error(where_, "no turnover band labels"));
    }
    let mut ranges = RangeMap::new();
    while i < lines.len() {
        let Some(name) = parse_heading_name(&lines[i]) else {
            match skip_non_heading(lines, i, labels.len(), where_)? {
                Some(next) => {
                    i = next;
                    continue;
                }
                None => break,
            }
        };
        let heading = repr_str(&lines[i]);
        let Some(key) = heading_key(&name) else {
            return Err(build_error(
                where_,
                &format!("unrecognised benchmark heading {heading}"),
            ));
        };
        if ranges.contains_key(key) {
            return Err(build_error(where_, &format!("{heading} appears twice")));
        }
        i += 1;
        let values = parse_ratio_block(lines, &mut i, labels.len(), key, where_)?;
        ranges.insert(key.to_string(), values);
    }
    if ranges.is_empty() {
        return Err(build_error(
            where_,
            "no benchmark ranges under the year heading",
        ));
    }
    Ok((labels, ranges))
}

/// What one pass over the page's lines found.
#[derive(Default)]
struct Found {
    sections: BTreeMap<String, ParsedSection>,
    updated: Option<String>,
    qc: Option<String>,
    financial_year: Option<String>,
}

fn scan_line(lines: &[String], index: usize, found: &mut Found, where_: &str) -> Result<(), Error> {
    let line = &lines[index];
    if found.financial_year.is_none() {
        found.financial_year = find_financial_year(line);
    }
    if let Some((kind, year)) = parse_year_heading(line) {
        if found.sections.contains_key(kind) {
            return Err(build_error(where_, &format!("two {kind} benchmark tables")));
        }
        let (labels, ranges) = parse_section(lines, index, &format!("{where_} ({kind})"))?;
        let section = ParsedSection {
            year,
            labels,
            ranges,
        };
        found.sections.insert(kind.to_string(), section);
    } else if found.updated.is_none()
        && let Some(u) = parse_updated_line(line)
    {
        found.updated = Some(u.to_string());
    } else if is_qc(line) {
        found.qc = Some(line.clone());
    }
    Ok(())
}

pub fn parse_page(raw_text: &str, where_: &str) -> Result<ParsedPage, Error> {
    let text = raw_text.replace(['\u{2018}', '\u{2019}', '\u{00b4}', '\u{0060}'], "'");
    let lines: Vec<String> = py_str_splitlines(&text)
        .into_iter()
        .map(collapse_spaces)
        .filter(|line| !line.is_empty())
        .collect();
    let mut found = Found::default();
    for index in 0..lines.len() {
        scan_line(&lines, index, &mut found, where_)?;
    }
    if !found.sections.contains_key("key") {
        return Err(build_error(where_, "no key benchmark table"));
    }
    let Some(financial_year) = found.financial_year else {
        return Err(build_error(
            where_,
            "the page does not state which financial year it covers",
        ));
    };
    let (Some(last_updated), Some(qc)) = (found.updated, found.qc) else {
        return Err(build_error(
            where_,
            "the page carries no last-updated date or QC reference",
        ));
    };
    Ok(ParsedPage {
        sections: found.sections,
        financial_year,
        last_updated,
        qc,
    })
}
