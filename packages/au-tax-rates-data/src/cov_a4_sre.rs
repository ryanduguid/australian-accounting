// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `re.compile` and `re.search` against CPython 3.15.0: error texts and
//! positions, escapes, group names and references, conditional groups,
//! inline flags, ASCII and case-insensitive matching, sets, anchors and
//! greedy, lazy and possessive repeats. Each case is `(pattern, flags,
//! subject)` with CPython's answer: `str(re.error)` or `Class: message` when
//! compiling raises; then `groups N` with no subject, or `re.search`'s span
//! of every group (`-` where a group took no part) or `no match`.
//! `tests/data/cov_a4_sre.py` wrote `tests/data/cov_a4_sre.json`.

use super::*;
use crate::pyjson::{Json, loads_standard};

fn text(points: &[u32]) -> String {
    points
        .iter()
        .map(|&c| char::from_u32(c).unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

/// The answer as the oracle printed it.
fn answer(pattern: &str, flags: u32, subject: Option<&str>) -> String {
    let compiled = match compile(&pts(pattern), flags) {
        Ok(compiled) => compiled,
        Err(CompileError::Pattern(message)) => return text(&message),
        Err(CompileError::Other(class, message)) => return format!("{class}: {message}"),
    };
    let Some(subject) = subject else {
        return format!("groups {}", compiled.groups);
    };
    let Some(found) = compiled.search(&pts(subject)) else {
        return String::from("no match");
    };
    let span = |i| match found.span(i) {
        Some((a, b)) => format!("{a}-{b}"),
        None => String::from("-"),
    };
    let spans: Vec<String> = (0..=compiled.groups).map(span).collect();
    spans.join(" ")
}

/// The oracle's cases as `(pattern, flags, subject, answer)`.
fn cases() -> Vec<(String, u32, Option<String>, String)> {
    let doc = loads_standard(CASES).expect("case file");
    assert_eq!(doc.at("python").as_str(), Some("3.15.0"));
    let Json::Array(rows) = doc.at("cases") else {
        panic!("a list of cases");
    };
    let text = |value: &Json| value.as_str().map(str::to_string);
    let row = |case: &Json| match case {
        Json::Array(row) => match &row[..] {
            [pattern, Json::Int(flags), subject, answer] => (
                text(pattern).expect("a pattern"),
                u32::try_from(flags).expect("flags"),
                text(subject),
                text(answer).expect("an answer"),
            ),
            _ => panic!("a case of 4 items: {case:?}"),
        },
        _ => panic!("a case: {case:?}"),
    };
    rows.iter().map(row).collect()
}

#[test]
fn compile_and_search_answer_as_cpython_does() {
    let (got, want): (Vec<String>, Vec<String>) = cases()
        .iter()
        .map(|(pattern, flags, subject, want)| {
            let case = format!("{pattern:?} {flags} {subject:?}");
            let got = answer(pattern, *flags, subject.as_deref());
            (format!("{case} {got}"), format!("{case} {want}"))
        })
        .unzip();
    assert_eq!(got.len(), 125);
    assert_eq!(got, want);
}

/// The patterns too long for the table.
#[test]
fn long_patterns_answer_as_cpython_does() {
    let digits = format!("a{{{}}}", "1".repeat(5000));
    let limit = "ValueError: Exceeds the limit (4300 digits) for integer string conversion: \
                 value has 5000 digits; use sys.set_int_max_str_digits() to increase the limit";
    assert_eq!(answer(&digits, 0, None), limit);
    let deep = "RecursionError: maximum recursion depth exceeded";
    let groups = format!("{}{}", "(".repeat(500), ")".repeat(500));
    assert_eq!(answer(&groups, 0, None), deep);
    let plain = format!("{}{}", "(?:".repeat(500), ")".repeat(500));
    assert_eq!(answer(&plain, 0, None), deep);
    let deepest = format!("[a]{}a{}", "(?:".repeat(479), ")".repeat(479));
    assert_eq!(answer(&deepest, 0, Some("aa")), "0-2");
}

const CASES: &str = include_str!("../tests/data/cov_a4_sre.json");
