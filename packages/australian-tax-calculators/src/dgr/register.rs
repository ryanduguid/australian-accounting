// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The bundled DGR data, read once, with the ABN check and the name words
//! `dgr-check` matches on.
use crate::dates;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

/// The register: the extract's header and every ABN with a DGR entry, sorted by ABN.
pub(super) struct Register {
    pub(super) header: Value,
    pub(super) data_date: String,
    pub(super) entries: Vec<Entry>,
    /// Each name word and the (entry, name) pairs it appears in.
    pub(super) words: HashMap<String, Vec<(usize, usize)>>,
}

/// A data row: ABN, status, status date, name, DGR entries and other names.
pub(super) type Row = (
    String,
    String,
    String,
    String,
    Vec<(String, Option<String>)>,
    Vec<String>,
);

pub(super) struct Entry {
    pub(super) abn: String,
    pub(super) cancelled: bool,
    pub(super) status_from: String,
    /// The entity's name, its other names, then its fund names.
    pub(super) names: Vec<String>,
    pub(super) other_names: usize,
    /// Each DGR entry's start date and, when the endorsement is only for the
    /// operation of a fund, authority or institution, its name (empty where
    /// the extract gives none).
    pub(super) dgr: Vec<(String, Option<String>)>,
}

pub(super) fn register() -> &'static Register {
    static REGISTER: OnceLock<Register> = OnceLock::new();
    REGISTER.get_or_init(|| parse_register(include_str!("../../data/dgr-register.jsonl")))
}

/// The bundled data, which `tests/dgr_check.rs` reads in full.
pub(super) fn parse_register(text: &str) -> Register {
    let mut lines = text.lines();
    let header: Value = serde_json::from_str(lines.next().expect("a header")).expect("JSON header");
    let time = header["extract_time"].as_str().expect("an extract time");
    let data_date = time.get(..10).filter(|d| dates::parse(d).is_some());
    let data_date = data_date.expect("a YYYY-MM-DD extract date").to_string();
    let mut entries: Vec<Entry> = Vec::new();
    let mut words: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for line in lines {
        let row: Row = serde_json::from_str(line).expect("a register row");
        let (abn, status, status_from, name, dgr, others) = row;
        assert!(abn_digits(&abn).is_some(), "{abn}: a valid ABN");
        assert!(entries.last().is_none_or(|e| e.abn < abn), "{abn}: sorted");
        assert!(matches!(status.as_str(), "ACT" | "CAN"), "{abn}: {status}");
        let dated = std::iter::once(&status_from).chain(dgr.iter().map(|(from, _)| from));
        assert!(
            dated.clone().all(|d| dates::parse(d).is_some()),
            "{abn}: dates"
        );
        assert!(!dgr.is_empty(), "{abn}: a DGR entry");
        let other_names = others.len();
        let mut names = vec![name];
        names.extend(others);
        names.extend(dgr.iter().filter_map(|(_, fund)| fund.clone()));
        for (at, name) in names.iter().enumerate() {
            for word in name_words(name) {
                let postings = words.entry(word).or_default();
                if postings.last() != Some(&(entries.len(), at)) {
                    postings.push((entries.len(), at));
                }
            }
        }
        entries.push(Entry {
            abn,
            cancelled: status == "CAN",
            status_from,
            names,
            other_names,
            dgr,
        });
    }
    Register {
        header,
        data_date,
        entries,
        words,
    }
}

/// The ABN's 11 digits, if it passes ABN Lookup's 'Format of the ABN' test
/// (https://abr.business.gov.au/Help/AbnFormat, read 16 November 2026):
/// subtract 1 from the first digit, weight the 11 digits 10, 1, 3, 5, 7, 9, 11,
/// 13, 15, 17 and 19, and the sum must divide by 89. As in gst-review and the
/// MCP's `validate_abn`, only ASCII digits and spaces are accepted: stripping
/// anything else could turn a malformed ABN into a different, valid one.
pub(super) fn abn_digits(text: &str) -> Option<String> {
    const WEIGHTS: [u32; 11] = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    if !text.bytes().all(|b| b.is_ascii_digit() || b == b' ') {
        return None;
    }
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    let values: Vec<u32> = digits.bytes().map(|b| u32::from(b - b'0')).collect();
    if values.len() != 11 || values[0] == 0 {
        return None;
    }
    let total: u32 = values.iter().zip(WEIGHTS).map(|(d, w)| d * w).sum();
    // Subtracting 1 from the first digit takes its weight off the sum once.
    (total - WEIGHTS[0]).is_multiple_of(89).then_some(digits)
}

/// A name's words for matching: upper case, split at anything not a letter or
/// digit, with `&` read as AND and LTD, INC and PTY as the words they shorten.
// shortcut: whole-word matching after these 4 rewrites, no fuzzy or phonetic
// match; a misspelt or reworded name finds nothing. A ranked fuzzy search would
// find more, at the cost of more candidates to rule out by hand.
pub(super) fn name_words(name: &str) -> Vec<String> {
    let spaced = name.to_ascii_uppercase().replace('&', " AND ");
    let mut words: Vec<String> = spaced
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| match w {
            "LTD" => "LIMITED",
            "INC" => "INCORPORATED",
            "PTY" => "PROPRIETARY",
            other => other,
        })
        .map(str::to_string)
        .collect();
    words.sort();
    words.dedup();
    words
}
