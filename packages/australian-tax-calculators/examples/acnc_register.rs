// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Rebuild `data/acnc-register.jsonl`, which `charity-check` reads, from the
//! ACNC Register of Australian charities CSV
//! (https://data.gov.au/data/dataset/acnc-register). Give it the resource's
//! last-modified time from the dataset page and the download's SHA-256, and
//! the CSV on stdin. The new file replaces the old only when the build
//! succeeds:
//!
//! ```sh
//! cargo run --release --example acnc_register -- \
//!   --last-modified 2026-11-22T17:39:55 --sha256 "$(sha256sum datadotgov_main.csv | cut -c1-64)" \
//!   < datadotgov_main.csv > data/acnc-register.jsonl.new &&
//!   mv data/acnc-register.jsonl.new data/acnc-register.jsonl
//! ```
//!
//! It keeps the ABN, legal name, registration date and the 14 subtype columns
//! of each charity with an ABN, sorted by ABN, and counts and drops a row
//! whose ABN is withheld (blank) or fails the check digit test (the dataset
//! of 22 November 2026 gives 2 ACNC groups an 11-digit number that is not an
//! ABN), since `charity-check` refuses such a number before any lookup. It keeps no address, website, size,
//! responsible persons, financial year, operating location or beneficiary
//! column. It stops on: a header without the expected columns, a row with
//! another number of fields or an unclosed quote, a duplicate ABN, a registration date other
//! than blank or DD/MM/YYYY, or a subtype cell other than blank or Y.
use serde_json::json;
use std::io::{Read, Write};

/// The CSV's subtype columns in ACNC Act 2012 s 25-5(5) item order, 1 to 14.
const SUBTYPE_COLUMNS: [&str; 14] = [
    "Advancing_Health",
    "Advancing_Education",
    "Advancing_social_or_public_welfare",
    "Advancing_Religion",
    "Advancing_Culture",
    "Promoting_reconciliation__mutual_respect_and_tolerance",
    "Promoting_or_protecting_human_rights",
    "Advancing_security_or_safety_of_Australia_or_Australian_public",
    "Preventing_or_relieving_suffering_of_animals",
    "Advancing_natual_environment",
    "Purposes_beneficial_to_ther_general_public_and_other_analogous",
    "Promote_or_oppose_a_change_to_law__government_poll_or_prac",
    "HPC",
    "PBI",
];

type Result<T> = std::result::Result<T, String>;

/// One CSV line's fields: commas separate them, and a quoted field may hold
/// commas and doubled quotes.
fn fields(line: &str, at: usize) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut chars = line.chars().peekable();
    loop {
        let mut field = String::new();
        if chars.peek() == Some(&'"') {
            chars.next();
            loop {
                match chars.next() {
                    Some('"') if chars.peek() == Some(&'"') => {
                        chars.next();
                        field.push('"');
                    }
                    Some('"') => break,
                    Some(c) => field.push(c),
                    None => return Err(format!("line {at}: an unclosed quote")),
                }
            }
        }
        while let Some(&c) = chars.peek() {
            if c == ',' {
                break;
            }
            field.push(c);
            chars.next();
        }
        out.push(field);
        if chars.next().is_none() {
            return Ok(out);
        }
    }
}

fn abn_ok(abn: &str) -> bool {
    const WEIGHTS: [u32; 11] = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    let Some(digits) = abn
        .chars()
        .map(|c| c.to_digit(10))
        .collect::<Option<Vec<u32>>>()
    else {
        return false;
    };
    if digits.len() != 11 || digits[0] == 0 {
        return false;
    }
    let total: u32 = digits.iter().zip(WEIGHTS).map(|(d, w)| d * w).sum();
    (total - WEIGHTS[0]).is_multiple_of(89)
}

/// DD/MM/YYYY as YYYY-MM-DD, or None for a blank cell.
fn date(cell: &str, at: usize) -> Result<Option<String>> {
    if cell.is_empty() {
        return Ok(None);
    }
    let parts: Vec<&str> = cell.split('/').collect();
    let shaped = parts.len() == 3
        && [2, 2, 4] == [parts[0].len(), parts[1].len(), parts[2].len()]
        && parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit()));
    if !shaped {
        return Err(format!(
            "line {at}: registration date {cell:?} is not DD/MM/YYYY"
        ));
    }
    Ok(Some(format!("{}-{}-{}", parts[2], parts[1], parts[0])))
}

fn column(header: &[String], name: &str) -> Result<usize> {
    header
        .iter()
        .position(|h| h == name)
        .ok_or_else(|| format!("the header has no {name} column"))
}

type Row = (String, String, Option<String>, Vec<usize>);

/// What the build read: the rows kept, sorted by ABN, and the rows dropped.
#[derive(Debug, PartialEq)]
struct Counts {
    kept: Vec<Row>,
    withheld: usize,
    invalid: usize,
    read: usize,
}

/// The s 25-5(5) items of the subtype columns marked 'Y'.
fn subtype_items(cells: &[String], subtypes: &[usize], at: usize) -> Result<Vec<usize>> {
    let mut items = Vec::new();
    for (item, &col) in subtypes.iter().enumerate() {
        match cells[col].as_str() {
            "Y" => items.push(item + 1),
            "" => {}
            other => {
                return Err(format!(
                    "line {at}: subtype cell {other:?} is not Y or blank"
                ));
            }
        }
    }
    Ok(items)
}

fn rows(text: &str) -> Result<Counts> {
    let mut lines = text.strip_prefix('\u{feff}').unwrap_or(text).lines();
    let header = fields(lines.next().ok_or("an empty file")?, 1)?;
    let [abn, name, registered] =
        ["ABN", "Charity_Legal_Name", "Registration_Date"].map(|c| column(&header, c));
    let (abn, name, registered) = (abn?, name?, registered?);
    let subtypes: Vec<usize> = SUBTYPE_COLUMNS
        .iter()
        .map(|c| column(&header, c))
        .collect::<Result<_>>()?;
    let (mut kept, mut withheld, mut invalid, mut read) = (Vec::new(), 0, 0, 0);
    for (i, line) in lines.enumerate() {
        let at = i + 2;
        let cells = fields(line, at)?;
        if cells.len() != header.len() {
            return Err(format!(
                "line {at}: {} fields, not {}",
                cells.len(),
                header.len()
            ));
        }
        read += 1;
        if cells[abn].is_empty() {
            withheld += 1;
            continue;
        }
        if !abn_ok(&cells[abn]) {
            invalid += 1;
            continue;
        }
        kept.push((
            cells[abn].clone(),
            cells[name].clone(),
            date(&cells[registered], at)?,
            subtype_items(&cells, &subtypes, at)?,
        ));
    }
    kept.sort_by(|a, b| a.0.cmp(&b.0));
    if let Some(pair) = kept.windows(2).find(|p| p[0].0 == p[1].0) {
        return Err(format!("ABN {} appears twice", pair[0].0));
    }
    Ok(Counts {
        kept,
        withheld,
        invalid,
        read,
    })
}

fn header(last_modified: &str, sha256: &str, read: &Counts) -> serde_json::Value {
    json!({
        "source": "ACNC Register of Australian charities, Australian Charities and Not-for-profits Commission",
        "source_url": "https://data.gov.au/data/dataset/acnc-register",
        "resource_url": "https://data.gov.au/data/dataset/b050b242-4487-4306-abf5-07ca073e5594/resource/8fb32972-24e9-4c95-885e-7140be51be8a/download/datadotgov_main.csv",
        "licence": "Creative Commons Attribution 3.0 Australia",
        "licence_url": "https://creativecommons.org/licenses/by/3.0/au/",
        "copyright": "(c) Commonwealth of Australia (Australian Charities and Not-for-profits Commission)",
        "attribution": "Contains data from the ACNC Register of Australian charities, Australian Charities and Not-for-profits Commission, licensed under CC BY 3.0 AU (https://creativecommons.org/licenses/by/3.0/au/).",
        "changes": "Only the ABN, legal name, registration date and subtype columns of charities with an ABN are kept; dates are written YYYY-MM-DD and subtypes as ACNC Act 2012 s 25-5(5) item numbers.",
        "last_modified": last_modified,
        "snapshot_date": &last_modified[..10],
        "csv_sha256": sha256,
        "rows_read": read.read,
        "abn_withheld_dropped": read.withheld,
        "abn_invalid_dropped": read.invalid,
        "charities": read.kept.len(),
    })
}

fn arguments() -> Result<(String, String)> {
    // The operator's own build flags, read as data and checked below.
    // nosemgrep: rust.lang.security.args.args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        let at = args.iter().position(|a| a == flag);
        at.and_then(|i| args.get(i + 1))
            .cloned()
            .ok_or(format!("{flag} is required"))
    };
    let last_modified = value("--last-modified")?;
    let sha256 = value("--sha256")?;
    let dated = last_modified.len() >= 10
        && last_modified.get(4..5) == Some("-")
        && last_modified.get(7..8) == Some("-");
    if !dated || sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("--last-modified takes YYYY-MM-DDTHH:MM:SS and --sha256 64 hex digits".into());
    }
    Ok((last_modified, sha256.to_ascii_lowercase()))
}

fn main() -> Result<()> {
    let (last_modified, sha256) = arguments()?;
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let read = rows(&text)?;
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let first = header(&last_modified, &sha256, &read);
    writeln!(out, "{first}").map_err(|e| e.to_string())?;
    for row in &read.kept {
        writeln!(out, "{}", json!(row)).map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "acnc_register/tests.rs"]
mod tests;
