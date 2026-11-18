// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::super::{Context, page, words};
use super::rank::{Ranked, priority, retain, tier};
use super::rows::{READ_ERROR, Rows, bounded_size, joined, policy};
use serde_json::{Value, json};
use std::{
    collections::BinaryHeap,
    path::{Path, PathBuf},
};

/// How a search reads one kind of row: what it matches, ranks and returns.
pub(super) struct Kind<'a> {
    fields: &'a [&'a str],
    heading: &'a [&'a str],
    body: &'a [&'a str],
}
const SECTION_FIELDS: &[&str] = &["act", "section", "heading", "container", "text"];
const RATE_FIELDS: &[&str] = &["act", "section", "heading", "topic", "content"];
pub(super) const SECTIONS: Kind = Kind {
    fields: SECTION_FIELDS,
    heading: &["heading"],
    body: &["text"],
};
pub(super) const RATES: Kind = Kind {
    fields: RATE_FIELDS,
    heading: &["heading", "topic"],
    body: &["content"],
};
pub(super) type Key = (usize, usize, usize);
type Assessed = Result<Option<(Key, Value)>, String>;

/// The best `count` rows `assess` accepts, best first; ties keep corpus order. `assess`
/// is given the index each row came from.
fn scan(
    context: &Context,
    files: &[PathBuf],
    prefilter: &[String],
    least: Option<usize>,
    count: usize,
    assess: &mut dyn FnMut(&Path, &Value) -> Assessed,
) -> Result<Vec<Value>, String> {
    let (mut best, mut size, mut order) = (BinaryHeap::new(), 0, 0);
    for path in files {
        bounded_size(context, path, &mut size)?;
        for row in Rows::open_some(path, prefilter, least).map_err(|_| READ_ERROR)? {
            let row = row.map_err(|_| READ_ERROR)?;
            if let Some((key, value)) = assess(path, &row)? {
                retain(
                    &mut best,
                    Ranked {
                        key: (key, order),
                        value,
                    },
                    count,
                );
                order += 1;
            }
        }
    }
    Ok(best
        .into_sorted_vec()
        .into_iter()
        .map(|ranked| ranked.value)
        .collect())
}

/// The query's words and how a row is tested against them, as corpus.py's `_search`
/// and `_withheld_counts` read them.
pub(super) struct Query {
    terms: Vec<String>,
    words: Vec<String>,
    opposites: Vec<(usize, String)>,
    pub(super) parts: Vec<String>,
}
impl Query {
    pub(super) fn new(terms: &[String]) -> Self {
        let words = words::search_words(terms);
        let opposites = words
            .iter()
            .enumerate()
            .filter(|(_, w)| words::is_modal(w.split(' ').next().unwrap()))
            .map(|(i, w)| (i, words::opposite(w)))
            .collect();
        let parts = words::parts(&words);
        Self {
            terms: terms.to_vec(),
            words,
            opposites,
            parts,
        }
    }
    pub(super) fn is_match(&self, row: &Value, kind: &Kind) -> bool {
        words::holds_all(&self.words, &joined(row, kind.fields))
    }
    /// Whether any row can be partly held: a shorter query cannot be, unless a row
    /// holds a modal word's opposite, and stop words alone name nothing.
    pub(super) fn has_partials(&self) -> bool {
        self.words.len() + usize::from(!self.opposites.is_empty()) > MIN_PARTIAL_WORDS
            && !words::is_stop(&self.words[0])
    }
    /// The sort key and matched words of a row whose heading and body hold at least
    /// MIN_PARTIAL_WORDS of the words, a modal word's opposite counting but reported as
    /// missing. Every word holding a digit must be there as well.
    pub(super) fn partial(
        &self,
        context: &Context,
        row: &Value,
        kind: &Kind,
    ) -> Option<(Key, Vec<bool>)> {
        let text = joined(row, &[kind.heading, kind.body].concat());
        let found: Vec<bool> = self.words.iter().map(|w| words::has(w, &text)).collect();
        let held = found.iter().filter(|f| **f).count();
        let opposed = self
            .opposites
            .iter()
            .filter(|(i, opposite)| !found[*i] && words::has(opposite, &text))
            .count();
        let numbers = self
            .words
            .iter()
            .zip(&found)
            .all(|(w, f)| *f || !words::has_digit(w));
        if held + opposed < MIN_PARTIAL_WORDS || !numbers {
            return None;
        }
        let title = joined(row, kind.heading);
        let titled = self.words.iter().filter(|w| words::has(w, &title)).count();
        Some((
            (
                usize::MAX - held,
                usize::MAX - titled,
                priority(context, row),
            ),
            found,
        ))
    }
}
pub(super) const MIN_PARTIAL_WORDS: usize = 2;

/// One page of the rows holding every word of the query, then of those holding some.
/// A partial hit always ranks below every match and is never reported as one.
pub(super) struct Search<'a> {
    pub(super) context: &'a Context,
    pub(super) files: &'a [PathBuf],
    pub(super) query: &'a Query,
    pub(super) scope_terms: &'a [String],
    pub(super) scope: &'a dyn Fn(&Value) -> bool,
    pub(super) kind: &'a Kind<'static>,
    // The result for one row, given the index it came from.
    pub(super) build: &'a dyn Fn(&Path, &Value) -> Result<Value, String>,
    pub(super) limit: usize,
    pub(super) offset: usize,
    pub(super) corpus: Value,
}

pub(super) fn search_rows(search: Search<'_>) -> Result<Value, String> {
    let end = search.offset + search.limit;
    let prefilter = [search.query.parts.clone(), search.scope_terms.to_vec()].concat();
    let full = full_matches(&search, &prefilter, end)?;
    let partial = partial_matches(&search, &full, end)?;
    Ok(finish_search(search, full, partial, end))
}

fn full_matches(
    search: &Search<'_>,
    prefilter: &[String],
    end: usize,
) -> Result<Vec<Value>, String> {
    scan(
        search.context,
        search.files,
        prefilter,
        None,
        end + 1,
        &mut |path, row| accept_match(search, path, row),
    )
}

fn accept_match(search: &Search<'_>, path: &Path, row: &Value) -> Assessed {
    if !(search.scope)(row) || !search.query.is_match(row, search.kind) {
        return Ok(None);
    }
    let tier = tier(
        &search.query.words,
        &search.query.terms,
        search.kind.heading,
        search.kind.body,
        row,
    );
    Ok(Some((
        (tier, priority(search.context, row), 0),
        (search.build)(path, row)?,
    )))
}

fn partial_matches(search: &Search<'_>, full: &[Value], end: usize) -> Result<Vec<Value>, String> {
    if full.len() > end || !search.query.has_partials() {
        return Ok(Vec::new());
    }
    let count = end + 1 - full.len();
    scan(
        search.context,
        search.files,
        &search.query.parts,
        Some(MIN_PARTIAL_WORDS),
        count,
        &mut |path, row| accept_partial(search, path, row),
    )
}

fn accept_partial(search: &Search<'_>, path: &Path, row: &Value) -> Assessed {
    let Some((key, found)) = search.query.partial(search.context, row, search.kind) else {
        return Ok(None);
    };
    if search.query.is_match(row, search.kind) || !(search.scope)(row) {
        return Ok(None);
    }
    let mut value = (search.build)(path, row)?;
    value["match"] = json!("partial");
    value["matched_words"] = json!(listed(&search.query.words, &found, true));
    value["missing_words"] = json!(listed(&search.query.words, &found, false));
    Ok(Some((key, value)))
}

fn listed<'a>(words: &'a [String], found: &[bool], held: bool) -> Vec<&'a String> {
    words
        .iter()
        .zip(found)
        .filter(|(_, flag)| **flag == held)
        .map(|(word, _)| word)
        .collect()
}

fn finish_search(search: Search<'_>, full: Vec<Value>, partial: Vec<Value>, end: usize) -> Value {
    let (matched, more) = (full.len(), full.len() + partial.len() > end);
    let mut hits: Vec<Value> = full
        .into_iter()
        .chain(partial)
        .skip(search.offset)
        .take(search.limit)
        .collect();
    let partial_hits = hits.split_off(matched.saturating_sub(search.offset).min(hits.len()));
    let mut result = page(
        hits,
        end,
        more,
        policy("NOTICE"),
        search.context.limit("MAX_OFFSET", 10000),
    );
    result["corpus"] = search.corpus;
    let mut dropped: Vec<&String> = Vec::new();
    for term in &search.query.terms {
        if words::is_stop(term) && !search.query.parts.contains(term) && !dropped.contains(&term) {
            dropped.push(term);
        }
    }
    if !partial_hits.is_empty() {
        result["notice"] = json!(format!(
            "{} {}",
            result["notice"].as_str().unwrap(),
            policy("PARTIAL_NOTICE")
        ));
    }
    result["partial_hits"] = json!(partial_hits);
    result["dropped_words"] = json!(dropped);
    result
}
