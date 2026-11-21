// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Rebuild `data/dgr-register.jsonl`, which `dgr-check` reads, from the ABN
//! Bulk Extract (https://data.gov.au/data/dataset/abn-bulk-extract). Feed it
//! the 20 XML files of one week's extract, one after another, on stdin. The
//! new file replaces the old only when the build succeeds:
//!
//! ```sh
//! for zip in public_split_1_10.zip public_split_11_20.zip; do
//!   for xml in $(unzip -Z1 "$zip"); do unzip -p "$zip" "$xml"; done
//! done | cargo run --release --example dgr_register > data/dgr-register.jsonl.new &&
//!   mv data/dgr-register.jsonl.new data/dgr-register.jsonl
//! ```
//!
//! It streams the 13 GB of XML and keeps only ABN records with a `DGR` element:
//! the ABN, its status and status date, the entity's name, its other names and
//! each DGR entry's start date and fund name. A `DGR` element with a
//! `NonIndividualName` is for a fund, authority or institution, kept with its
//! name, or with an empty name where the extract gives none. It keeps nothing
//! about an individual (an `IND` record's `LegalEntity`): such a record is
//! counted and dropped. It reads only the XML the schema uses, without
//! validating against the schema, and stops on: XML it cannot read (a comment,
//! CDATA, an unknown entity, a misnested tag or a truncated file), an ABN that
//! is not 11 digits, a status or DGR date that is not `yyyymmdd`, a `DGR`
//! status other than ACT, a kept record whose ABN status is not ACT or CAN or
//! which has no entity name, a file whose record count differs from its
//! `RecordCount`, files from different extracts, or a file sequence other than
//! 1 to 20 once each.

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{self, BufReader, Bytes, Read, Write};

type Result<T> = std::result::Result<T, String>;

enum Event {
    Start(String, Vec<(String, String)>, bool),
    End(String),
    Text(String),
    Eof,
}

struct Xml<R: Read> {
    bytes: Bytes<BufReader<R>>,
    back: Option<u8>,
}

impl<R: Read> Xml<R> {
    fn byte(&mut self) -> Result<Option<u8>> {
        match self.back.take() {
            Some(b) => Ok(Some(b)),
            None => self.bytes.next().transpose().map_err(|e| e.to_string()),
        }
    }

    fn until(&mut self, stop: u8) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        loop {
            match self.byte()? {
                None => return Err("the XML ends inside a tag".into()),
                Some(b) if b == stop => return Ok(out),
                Some(b) => out.push(b),
            }
        }
    }

    fn event(&mut self) -> Result<Event> {
        let Some(first) = self.byte()? else {
            return Ok(Event::Eof);
        };
        if first != b'<' {
            let mut raw = vec![first];
            while let Some(b) = self.byte()? {
                if b == b'<' {
                    self.back = Some(b);
                    break;
                }
                raw.push(b);
            }
            return text(raw).map(Event::Text);
        }
        match self.byte()? {
            Some(b'?') => {
                // The XML declaration that opens each file.
                let body = self.until(b'>')?;
                if !body.starts_with(b"xml ") || body.last() != Some(&b'?') {
                    return Err("an unexpected processing instruction".into());
                }
                self.event()
            }
            Some(b'!') => {
                Err("a comment, CDATA section or DOCTYPE, which the schema has none of".into())
            }
            Some(b'/') => {
                let name = self.until(b'>')?;
                Ok(Event::End(text(name)?.trim_end().to_string()))
            }
            Some(b) => {
                self.back = Some(b);
                self.start_tag()
            }
            None => Err("the XML ends after '<'".into()),
        }
    }

    /// A start tag after its '<', up to the '>' outside any quoted value.
    fn start_tag(&mut self) -> Result<Event> {
        let mut raw = Vec::new();
        let mut quote = None;
        loop {
            let Some(b) = self.byte()? else {
                return Err("the XML ends inside a tag".into());
            };
            match (quote, b) {
                (None, b'>') => break,
                (None, b'"' | b'\'') => quote = Some(b),
                (Some(q), _) if q == b => quote = None,
                _ => {}
            }
            raw.push(b);
        }
        let empty = raw.last() == Some(&b'/');
        if empty {
            raw.pop();
        }
        let (name, attributes) = tag(&String::from_utf8(raw).map_err(|e| e.to_string())?)?;
        Ok(Event::Start(name, attributes, empty))
    }
}

/// Character data with its 5 predefined entities and character references
/// decoded; any other reference is refused rather than kept as written.
fn text(raw: Vec<u8>) -> Result<String> {
    let raw = String::from_utf8(raw).map_err(|e| format!("text is not UTF-8: {e}"))?;
    let mut out = String::new();
    let mut rest = raw.as_str();
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let end = rest[at..]
            .find(';')
            .ok_or("an unterminated entity reference")?
            + at;
        let entity = &rest[at + 1..end];
        let decoded = match entity {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let code = match entity.strip_prefix("#x") {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => entity.strip_prefix('#').and_then(|d| d.parse().ok()),
                };
                code.and_then(char::from_u32)
                    .ok_or_else(|| format!("an unknown entity &{entity};"))?
            }
        };
        out.push(decoded);
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// A start tag's name and its attributes, values decoded.
fn tag(body: &str) -> Result<(String, Vec<(String, String)>)> {
    let body = body.trim();
    let (name, mut rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
    let mut attributes = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            return Ok((name.to_string(), attributes));
        }
        let (key, after) = rest
            .split_once('=')
            .ok_or_else(|| format!("<{name}>: an attribute with no value"))?;
        let after = after.trim_start();
        let quote = after.chars().next().filter(|c| matches!(c, '"' | '\''));
        let quote = quote.ok_or_else(|| format!("<{name}>: an unquoted attribute"))?;
        let (value, tail) = after[1..]
            .split_once(quote)
            .ok_or_else(|| format!("<{name}>: an unterminated attribute"))?;
        attributes.push((key.trim().to_string(), text(value.as_bytes().to_vec())?));
        rest = tail;
    }
}

fn attribute<'a>(attributes: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// The extract's `yyyymmdd` as an ISO date.
fn date(raw: Option<&str>, what: &str, abn: &str) -> Result<String> {
    match raw {
        Some(d) if d.len() == 8 && d.bytes().all(|b| b.is_ascii_digit()) => {
            Ok(format!("{}-{}-{}", &d[..4], &d[4..6], &d[6..]))
        }
        other => Err(format!(
            "ABN record after {abn}: {what} {other:?} is not yyyymmdd"
        )),
    }
}

#[derive(Default)]
struct Record {
    abn: String,
    status: String,
    status_from: String,
    individual: bool,
    name: Option<String>,
    other_names: Vec<String>,
    dgr: Vec<(String, Option<String>)>,
}

impl Record {
    fn line(mut self) -> (String, Value) {
        let name = self.name.expect("checked before the line is made");
        let mut others: Vec<String> = Vec::new();
        for other in self.other_names.drain(..) {
            let seen = |n: &String| n.eq_ignore_ascii_case(&other);
            if !other.eq_ignore_ascii_case(&name) && !others.iter().any(seen) {
                others.push(other);
            }
        }
        self.dgr.sort();
        let dgr: Vec<Value> = self
            .dgr
            .iter()
            .map(|(from, fund)| json!([from, fund]))
            .collect();
        let line = json!([self.abn, self.status, self.status_from, name, dgr, others]);
        (self.abn, line)
    }
}

#[derive(Default)]
struct Totals {
    extract_time: Option<String>,
    files: BTreeMap<u32, u64>,
    records: u64,
    individuals_with_dgr: u64,
    lines: BTreeMap<String, Value>,
}

/// Where the reader is: the open elements, the record being read and the
/// current file's sequence number, declared count and records so far.
#[derive(Default)]
struct State {
    path: Vec<String>,
    record: Record,
    sequence: Option<u32>,
    declared: Option<u64>,
    in_file: u64,
}

impl State {
    /// Character data, kept only where the record or the file header uses it.
    fn text(&mut self, value: String, totals: &mut Totals) -> Result<()> {
        let at: Vec<&str> = self.path.iter().rev().take(3).map(String::as_str).collect();
        let record = &mut self.record;
        match at.as_slice() {
            ["ABN", ..] => record.abn = value,
            ["FileSequenceNumber", ..] => self.sequence = value.parse::<u32>().ok(),
            ["RecordCount", ..] => self.declared = value.parse::<u64>().ok(),
            ["ExtractTime", ..] => match &totals.extract_time {
                Some(time) if *time != value => {
                    return Err(format!("files from 2 extracts: {time} and {value}"));
                }
                _ => totals.extract_time = Some(value),
            },
            ["NonIndividualNameText", "NonIndividualName", "MainEntity"] => {
                record.name = Some(value)
            }
            ["NonIndividualNameText", "NonIndividualName", "OtherEntity"] => {
                record.other_names.push(value)
            }
            ["NonIndividualNameText", "NonIndividualName", "DGR"] => {
                let last = record.dgr.last_mut().ok_or("a DGR name outside a DGR")?;
                last.1 = Some(value);
            }
            // Address, ASIC and GST details and individuals' names: not kept.
            _ => {}
        }
        Ok(())
    }

    /// A start tag, before it joins the path.
    fn start(&mut self, name: &str, attributes: &[(String, String)]) -> Result<()> {
        let record = &mut self.record;
        match name {
            "ABR" => *record = Record::default(),
            "ABN" => {
                record.status = attribute(attributes, "status").unwrap_or_default().into();
                record.status_from = date(
                    attribute(attributes, "ABNStatusFromDate"),
                    "ABNStatusFromDate",
                    &record.abn,
                )?;
            }
            "LegalEntity" => record.individual = true,
            // A DGR entry with a name element is for a fund, authority
            // or institution even when its name text is empty: ABN
            // Lookup lists such an entry as a fund with no name.
            "NonIndividualName" if self.path.last().is_some_and(|p| p == "DGR") => {
                let last = record.dgr.last_mut().ok_or("a DGR name outside a DGR")?;
                last.1 = Some(String::new());
            }
            "DGR" => {
                if let Some(status) = attribute(attributes, "status").filter(|s| *s != "ACT") {
                    return Err(format!(
                        "ABN record after {}: DGR status {status:?}",
                        record.abn
                    ));
                }
                let from = date(
                    attribute(attributes, "DGRStatusFromDate"),
                    "DGRStatusFromDate",
                    &record.abn,
                )?;
                record.dgr.push((from, None));
            }
            _ => {}
        }
        Ok(())
    }

    /// The end of an ABN record: counted, and kept when it has DGR entries.
    fn end_record(&mut self, totals: &mut Totals) -> Result<()> {
        self.in_file += 1;
        totals.records += 1;
        let record = &self.record;
        if !(record.abn.len() == 11 && record.abn.bytes().all(|b| b.is_ascii_digit())) {
            return Err(format!("an ABN of {:?}", record.abn));
        }
        if record.dgr.is_empty() {
            return Ok(());
        }
        if record.individual {
            totals.individuals_with_dgr += 1;
            return Ok(());
        }
        if !matches!(record.status.as_str(), "ACT" | "CAN") {
            return Err(format!("ABN {}: status {:?}", record.abn, record.status));
        }
        if record.name.as_deref().is_none_or(|n| n.trim().is_empty()) {
            return Err(format!("ABN {}: no entity name", record.abn));
        }
        let (abn, line) = std::mem::take(&mut self.record).line();
        if totals.lines.insert(abn.clone(), line).is_some() {
            return Err(format!("ABN {abn} appears twice"));
        }
        Ok(())
    }

    /// The end of a file: its record count must match what it declares.
    fn end_file(&mut self, totals: &mut Totals) -> Result<()> {
        let (Some(sequence), Some(declared)) = (self.sequence.take(), self.declared.take()) else {
            return Err("a file with no FileSequenceNumber or RecordCount".into());
        };
        let in_file = std::mem::take(&mut self.in_file);
        if declared != in_file {
            return Err(format!(
                "file {sequence} declares {declared} records but holds {in_file}"
            ));
        }
        if totals.files.insert(sequence, in_file).is_some() {
            return Err(format!("file {sequence} given twice"));
        }
        Ok(())
    }
}

/// Reads every file on stdin into `totals`.
fn read(xml: &mut Xml<impl Read>, totals: &mut Totals) -> Result<()> {
    let mut state = State::default();
    loop {
        let (start, end) = match xml.event()? {
            Event::Eof if state.path.is_empty() => return Ok(()),
            Event::Eof => return Err("the XML ends inside an element".into()),
            Event::Text(value) => {
                state.text(value, totals)?;
                continue;
            }
            Event::Start(name, attributes, empty) => {
                state.start(&name, &attributes)?;
                (Some(name.clone()), empty.then_some(name))
            }
            Event::End(name) => (None, Some(name)),
        };
        if let Some(name) = start {
            state.path.push(name);
        }
        let Some(name) = end else { continue };
        if state.path.pop().as_deref() != Some(name.as_str()) {
            return Err(format!("</{name}> closes another element"));
        }
        match name.as_str() {
            "ABR" => state.end_record(totals)?,
            "Transfer" => state.end_file(totals)?,
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let mut xml = Xml {
        bytes: BufReader::with_capacity(1 << 20, io::stdin().lock()).bytes(),
        back: None,
    };
    let mut totals = Totals::default();
    read(&mut xml, &mut totals)?;
    if totals.files.keys().copied().ne(1..=20) {
        return Err(format!("files {:?}, not 1 to 20", totals.files.keys()));
    }
    let header = json!({
        "source": "ABN Bulk Extract, Australian Business Register",
        "source_url": "https://data.gov.au/data/dataset/abn-bulk-extract",
        "licence": "Creative Commons Attribution 3.0 Australia",
        "extract_time": totals.extract_time,
        "abn_records_read": totals.records,
        "individuals_with_dgr_dropped": totals.individuals_with_dgr,
        "abns": totals.lines.len(),
    });
    let mut out = io::BufWriter::new(io::stdout().lock());
    let mut write = || -> io::Result<()> {
        writeln!(out, "{header}")?;
        for line in totals.lines.values() {
            writeln!(out, "{line}")?;
        }
        out.flush()
    };
    write().map_err(|e| e.to_string())?;
    eprintln!("{header}");
    Ok(())
}

#[cfg(test)]
#[path = "dgr_register/tests.rs"]
mod tests;
