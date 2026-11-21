// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::{Context, terms};
use super::rank::{Ranked, priority, retain, section};
use super::rows::{
    READ_ERROR, Rows, bounded_size, holds, indexes, optional_words, policy, provenance,
    query_terms, root, string,
};
use payday_super_checker::{py, pyre::Regex};
use serde_json::{Value, json};
use std::{collections::BinaryHeap, path::Path, sync::LazyLock};

static DICTIONARY: LazyLock<Regex> =
    LazyLock::new(|| Regex::ignore_case(policy("DICTIONARY_HEADING")));
static DEFINITION: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("DEFINITION_HEAD")));
static SUBSECTION: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("SUBSECTION_LABEL")));

fn plain(text: &str) -> String {
    py::casefold(
        &text
            .replace('\u{2011}', "-")
            .replace('\u{a0}', " ")
            .replace('*', ""),
    )
    .split(py::is_space)
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}
pub(in crate::retrieval) fn definitions(text: &str) -> Vec<(String, String)> {
    let (mut head, mut body, mut result) = (None, Vec::new(), Vec::new());
    for paragraph in text.split("\n\n") {
        let mut found = DEFINITION
            .captures(paragraph)
            .and_then(|m| m.name("head"))
            .map(|s| s.trim_end_matches([' ', ',']).to_owned());
        if found.as_ref().is_some_and(|s| {
            let folded = py::casefold(s);
            folded.starts_with("note")
                || folded.starts_with("example")
                || folded.starts_with("in this ")
                || s.starts_with("- ")
        }) {
            found = None;
        }
        if found.is_some() || SUBSECTION.matches_start(paragraph) {
            if let Some(head) = head.take() {
                result.push((head, body.join("\n\n")));
            }
            head = found;
            body = vec![paragraph.to_owned()];
        } else if head.is_some() {
            body.push(paragraph.to_owned());
        }
    }
    if let Some(head) = head {
        result.push((head, body.join("\n\n")));
    }
    result
}

struct Found {
    exact: Vec<(usize, Value)>,
    partial: BinaryHeap<Ranked<(usize, usize, usize)>>,
    order: usize,
    maximum: usize,
}

struct Sought<'a> {
    context: &'a Context,
    input: &'a Value,
    act: &'a [String],
    words: &'a [String],
    wanted: &'a str,
}

fn skip_dictionary(input: &Value, row: &Value, act: &[String]) -> bool {
    !DICTIONARY.matches_start(string(row, "heading").unwrap_or(""))
        || (input["in_force_only"] != false && row["version_is_current"] == false)
        || !holds(row, &["act"], act)
}

fn definition_kind(head: &str, wanted: &str, words: &[String]) -> Option<&'static str> {
    let key = plain(head);
    if key.split(',').next().unwrap().trim_matches(py::is_space) == wanted {
        Some("exact")
    } else if words.iter().all(|word| terms(&key).contains(word)) {
        Some("partial")
    } else {
        None
    }
}

fn record(
    context: &Context,
    row: &Value,
    priority: usize,
    head: String,
    body: String,
    kind: &str,
    found: &mut Found,
) {
    let mut definition = row.clone();
    definition["text"] = json!(body);
    let mut value = section(&definition, context.limit("SEARCH_TEXT_CHARS", 1200));
    value["head"] = json!(head);
    value["match"] = json!(kind);
    if kind == "exact" {
        found.exact.push((priority, value));
    } else {
        retain(
            &mut found.partial,
            Ranked {
                key: (priority, found.order, 0),
                value,
            },
            found.maximum + 1,
        );
        found.order += 1;
    }
}

fn read_index(
    path: &Path,
    prefilter: &[String],
    sought: &Sought<'_>,
    found: &mut Found,
) -> Result<(), String> {
    for row in Rows::open(path, prefilter).map_err(|_| READ_ERROR)? {
        let row = row.map_err(|_| READ_ERROR)?;
        if skip_dictionary(sought.input, &row, sought.act) {
            continue;
        }
        let priority = priority(sought.context, &row);
        for (head, body) in definitions(string(&row, "text").unwrap_or("")) {
            let Some(kind) = definition_kind(&head, sought.wanted, sought.words) else {
                continue;
            };
            record(sought.context, &row, priority, head, body, kind, found);
        }
    }
    Ok(())
}

fn finish_definitions(term: &str, input: &Value, root: &Path, mut found: Found) -> Value {
    let maximum = found.maximum;
    found.exact.sort_by_key(|(p, _)| *p);
    let dropped = found.partial.len() > maximum;
    let mut definitions: Vec<_> = found.exact.into_iter().map(|(_, v)| v).collect();
    definitions.extend(
        found
            .partial
            .into_sorted_vec()
            .into_iter()
            .take(maximum)
            .map(|v| v.value),
    );
    let limit = input["limit"].as_u64().unwrap() as usize;
    let more = dropped || definitions.len() > limit;
    definitions.truncate(limit);
    json!({"term":term,"definitions":definitions,"has_more":more,"corpus":provenance(root),"notice":format!("{} {}",policy("NOTICE"),policy("DEFINITION_NOTICE"))})
}

pub(super) fn define(context: &Context, input: &Value) -> Result<Value, String> {
    let term = input["term"].as_str().unwrap();
    let words = query_terms(term, "term")?;
    let wanted = plain(term);
    let act = optional_words(input, "act")?;
    let root = root(context)?;
    let mut found = Found {
        exact: Vec::new(),
        partial: BinaryHeap::new(),
        order: 0,
        maximum: context.limit("MAX_DEFINITIONS", 20),
    };
    let mut size = 0;
    let prefilter = [words.clone(), act.clone()].concat();
    let sought = Sought {
        context,
        input,
        act: &act,
        words: &words,
        wanted: &wanted,
    };
    for path in indexes(context, &root)? {
        bounded_size(context, &path, &mut size)?;
        read_index(&path, &prefilter, &sought, &mut found)?;
    }
    Ok(finish_definitions(term, input, &root, found))
}
