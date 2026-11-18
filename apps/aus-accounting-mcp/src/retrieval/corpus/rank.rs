// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::super::{Context, POLICY, terms, words};
use super::rows::{policy, py_string, string};
use payday_super_checker::py;
use serde_json::{Value, json};
use std::{cmp::Ordering, collections::BinaryHeap};

/// Where the words sit in a matching row, lower first: the phrase in one heading field,
/// every word in one heading field, the phrase in one body field, then the rest.
/// `phrase` is the whole query, stop words included, so "cost of a car" stays a phrase.
pub(in crate::retrieval) fn tier(
    words: &[String],
    phrase: &[String],
    headings: &[&str],
    bodies: &[&str],
    row: &Value,
) -> usize {
    let titles: Vec<&str> = headings.iter().filter_map(|f| string(row, f)).collect();
    let in_phrase = |parts: &[String]| {
        !phrase.is_empty() && parts.windows(phrase.len()).any(|window| window == phrase)
    };
    if titles.iter().any(|title| in_phrase(&terms(title))) {
        return 0;
    }
    if titles
        .iter()
        .any(|title| words::holds_all(words, &py::casefold(title)))
    {
        return 1;
    }
    if bodies
        .iter()
        .filter_map(|field| string(row, field))
        .any(|body| in_phrase(&terms(body)))
    {
        return 2;
    }
    3
}
pub(super) fn priority(context: &Context, row: &Value) -> usize {
    let act = py::casefold(string(row, "act").unwrap_or(""));
    #[cfg(test)]
    if let Some(acts) = &context.principal_acts {
        return acts.iter().position(|a| *a == act).unwrap_or(acts.len());
    }
    let _ = context;
    let acts = POLICY["corpus"]["PRINCIPAL_ACTS"].as_array().unwrap();
    acts.iter().position(|a| a == &act).unwrap_or(acts.len())
}
fn truncated(text: &str, cap: usize) -> (String, usize, Vec<String>) {
    let length = text.chars().count();
    let caveats = if length > cap {
        vec![
            policy("TRUNCATED_CAVEAT")
                .replace("{kept}", &cap.to_string())
                .replace("{total}", &length.to_string()),
        ]
    } else {
        Vec::new()
    };
    (text.chars().take(cap).collect(), length, caveats)
}
pub(super) fn section(row: &Value, cap: usize) -> Value {
    let (text, total, mut caveats) = truncated(string(row, "text").unwrap_or(""), cap);
    if row["version_is_current"] == false {
        caveats.push(policy("NOT_CURRENT_CAVEAT").into());
    }
    let mut value = json!({"text":text,"total_chars":total,"caveats":caveats,"version_is_current":row["version_is_current"].as_bool()});
    for field in [
        "row_id",
        "register_id",
        "act",
        "collection",
        "section",
        "heading",
        "container",
        "kind",
        "compilation_number",
        "compilation_date",
        "retrieved",
        "superseded_from",
        "register_page",
        "source_url",
        "licence",
        "attribution",
    ] {
        value[field] = json!(string(row, field));
    }
    value
}
pub(super) fn rate(row: &Value, cap: usize) -> Value {
    let (text, total, mut caveats) = truncated(string(row, "content").unwrap_or(""), cap);
    if row["version_is_current"] == false {
        caveats.push(policy("NOT_CURRENT_CAVEAT").into());
    }
    let mut value = json!({"content":text,"total_chars":total,"caveats":caveats,"version_is_current":row["version_is_current"].as_bool()});
    for field in [
        "rate_id",
        "register_id",
        "act",
        "collection",
        "section",
        "heading",
        "topic",
        "kind",
        "compilation_number",
        "compilation_date",
        "register_page",
    ] {
        value[field] = json!(string(row, field));
    }
    for field in ["amounts", "years"] {
        value[field] = json!(
            row[field]
                .as_array()
                .map(|items| items.iter().map(py_string).collect::<Vec<_>>())
                .unwrap_or_default()
        );
    }
    value
}

pub(in crate::retrieval) struct Ranked<K> {
    pub key: K,
    pub value: Value,
}
impl<K: Ord> PartialEq for Ranked<K> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<K: Ord> Eq for Ranked<K> {}
impl<K: Ord> Ord for Ranked<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}
impl<K: Ord> PartialOrd for Ranked<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
pub(in crate::retrieval) fn retain<K: Ord>(
    best: &mut BinaryHeap<Ranked<K>>,
    item: Ranked<K>,
    cap: usize,
) {
    if best.len() < cap {
        best.push(item);
    } else if best.peek().is_some_and(|worst| item.key < worst.key) {
        best.pop();
        best.push(item);
    }
}
