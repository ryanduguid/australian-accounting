// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Point-in-time legislation indexes supplied by the operator. No downloads.
use super::{Context, POLICY, as_at, linked, page, terms, words};
use payday_super_checker::{py, pyre::Regex};
use serde_json::{Value, json};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, VecDeque},
    fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

const READ_ERROR: &str = "Cannot read a UTF-8 JSONL index in the configured corpus.";
const SECTION_FIELDS: &[&str] = &["act", "section", "heading", "container", "text"];
const RATE_FIELDS: &[&str] = &["act", "section", "heading", "topic", "content"];
static ROW_ID: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("ROW_ID")));
static DICTIONARY: LazyLock<Regex> =
    LazyLock::new(|| Regex::ignore_case(policy("DICTIONARY_HEADING")));
static DEFINITION: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("DEFINITION_HEAD")));
static SUBSECTION: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("SUBSECTION_LABEL")));

fn policy(name: &str) -> &'static str {
    POLICY["corpus"][name].as_str().unwrap()
}
fn string<'a>(row: &'a Value, field: &str) -> Option<&'a str> {
    row[field].as_str()
}
fn py_string(value: &Value) -> String {
    div7a_loan_review::raw::Raw::from_value(value).py_str()
}
fn root(context: &Context) -> Result<PathBuf, String> {
    context.root(
        "AUS_ACCOUNTING_CORPUS_ROOT",
        "legislation corpus folder",
        policy("QUICK_BUILD"),
    )
}
fn query_terms(query: &str, field: &str) -> Result<Vec<String>, String> {
    let result = terms(query);
    if result.is_empty() {
        Err(format!("Supply at least one word in {field}."))
    } else {
        Ok(result)
    }
}
fn optional_words(input: &Value, field: &str) -> Result<Vec<String>, String> {
    match input[field].as_str().filter(|s| !s.is_empty()) {
        Some(text) => query_terms(text, field),
        None => Ok(Vec::new()),
    }
}
pub(super) fn joined(row: &Value, fields: &[&str]) -> String {
    py::casefold(
        &fields
            .iter()
            .filter_map(|f| string(row, f))
            .collect::<Vec<_>>()
            .join(" "),
    )
}
pub(super) fn holds(row: &Value, fields: &[&str], words: &[String]) -> bool {
    let joined = joined(row, fields);
    words.iter().all(|w| joined.contains(w))
}

fn indexes(context: &Context, root: &Path) -> Result<Vec<PathBuf>, String> {
    let titles = root.join("markdown");
    if !titles.is_dir() || linked(&titles).unwrap_or(true) {
        return Err("The corpus folder must contain a markdown directory of title indexes.".into());
    }
    let mut entries = fs::read_dir(titles)
        .map_err(|_| READ_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| READ_ERROR)?;
    entries.sort_by_key(|e| e.file_name().to_string_lossy().into_owned());
    let files: Vec<_> = entries
        .into_iter()
        .filter_map(|e| {
            let directory = e.path();
            if e.file_name().to_string_lossy().starts_with('.')
                || linked(&directory).unwrap_or(true)
                || !directory.is_dir()
            {
                return None;
            }
            let index = directory.join("sections.jsonl");
            (index.is_file() && !linked(&index).unwrap_or(true)).then_some(index)
        })
        .collect();
    if files.is_empty() {
        return Err("No title indexes found under the configured corpus folder.".into());
    }
    let maximum = context.limit("MAX_INDEX_FILES", 5000);
    if files.len() > maximum {
        return Err(format!(
            "Corpus exceeds {maximum} title indexes; configure a smaller corpus."
        ));
    }
    Ok(files)
}

/// Stream universal-newline UTF-8 JSONL. Decode every scanned line, even when
/// a raw prefilter subsequently excludes it. Malformed JSON rows are skipped.
/// `least` asks a raw line for that many of the filter's terms, as a partial hit
/// needs; None asks for every term.
struct Rows {
    lines: super::Lines,
    filter: Vec<String>,
    least: Option<usize>,
}
impl Rows {
    fn open(path: &Path, filter: &[String]) -> io::Result<Self> {
        Self::open_some(path, filter, None)
    }
    fn open_some(path: &Path, filter: &[String], least: Option<usize>) -> io::Result<Self> {
        Ok(Self {
            lines: super::Lines::open(path)?,
            filter: filter.to_vec(),
            least,
        })
    }
}
impl Iterator for Rows {
    type Item = io::Result<Value>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = match self.lines.next()? {
                Ok(line) => line,
                Err(e) => return Some(Err(e)),
            };
            if !line.contains("\\u") && !line.contains("\\/") {
                let folded = py::casefold(&line);
                let held = self
                    .filter
                    .iter()
                    .filter(|t| folded.contains(t.as_str()))
                    .count();
                if held < self.least.unwrap_or(self.filter.len()) {
                    continue;
                }
            }
            match super::parse_json(&line) {
                Ok(row) if row.is_object() => return Some(Ok(row)),
                Err(error) if error == "JSON nesting exceeds 1000 containers." => {
                    return Some(Err(io::Error::new(io::ErrorKind::InvalidData, error)));
                }
                _ => {}
            }
        }
    }
}

fn provenance(root: &Path) -> Value {
    let path = root.join("sources.json");
    let mut result = json!({});
    if !path.is_file() || linked(&path).unwrap_or(true) {
        return result;
    }
    let Ok(raw) = fs::read(&path) else {
        return result;
    };
    let Ok(text) = std::str::from_utf8(&raw) else {
        return result;
    };
    let Ok(record) = super::parse_json(text) else {
        return result;
    };
    for field in [
        "corpus",
        "source",
        "retrieved",
        "licence",
        "licence_url",
        "licence_note",
    ] {
        if let Some(value) = record[field].as_str() {
            result[field] = json!(value);
        }
    }
    if let Some(value) = record["attribution_markdown"].as_str() {
        result["attribution"] = json!(value);
    }
    result
}

/// Where the words sit in a matching row, lower first: the phrase in one heading field,
/// every word in one heading field, the phrase in one body field, then the rest.
/// `phrase` is the whole query, stop words included, so "cost of a car" stays a phrase.
pub(super) fn tier(
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
fn priority(context: &Context, row: &Value) -> usize {
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
fn section(row: &Value, cap: usize) -> Value {
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
fn rate(row: &Value, cap: usize) -> Value {
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

pub(super) struct Ranked<K> {
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
pub(super) fn retain<K: Ord>(best: &mut BinaryHeap<Ranked<K>>, item: Ranked<K>, cap: usize) {
    if best.len() < cap {
        best.push(item);
    } else if best.peek().is_some_and(|worst| item.key < worst.key) {
        best.pop();
        best.push(item);
    }
}
fn bounded_size(context: &Context, path: &Path, size: &mut u64) -> Result<(), String> {
    *size += fs::metadata(path).map_err(|_| READ_ERROR)?.len();
    let maximum = context.limit("MAX_CORPUS_BYTES", 320_000_000);
    if *size > maximum as u64 {
        return Err(format!(
            "Corpus exceeds {} MB; configure a smaller corpus.",
            maximum / 1_000_000
        ));
    }
    Ok(())
}

/// How a search reads one kind of row: what it matches, ranks and returns.
struct Kind<'a> {
    fields: &'a [&'a str],
    heading: &'a [&'a str],
    body: &'a [&'a str],
}
const SECTIONS: Kind = Kind {
    fields: SECTION_FIELDS,
    heading: &["heading"],
    body: &["text"],
};
const RATES: Kind = Kind {
    fields: RATE_FIELDS,
    heading: &["heading", "topic"],
    body: &["content"],
};
type Key = (usize, usize, usize);
type Assessed = Result<Option<(Key, Value)>, String>;

/// The best `count` rows `assess` accepts, best first; ties keep corpus order.
fn scan(
    context: &Context,
    files: &[PathBuf],
    prefilter: &[String],
    least: Option<usize>,
    count: usize,
    assess: &mut dyn FnMut(&Value) -> Assessed,
) -> Result<Vec<Value>, String> {
    let (mut best, mut size, mut order) = (BinaryHeap::new(), 0, 0);
    for path in files {
        bounded_size(context, path, &mut size)?;
        for row in Rows::open_some(path, prefilter, least).map_err(|_| READ_ERROR)? {
            let row = row.map_err(|_| READ_ERROR)?;
            if let Some((key, value)) = assess(&row)? {
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
        .map(|r| r.value)
        .collect())
}

/// The query's words and how a row is tested against them, as corpus.py's `_search`
/// and `_withheld_counts` read them.
struct Query {
    terms: Vec<String>,
    words: Vec<String>,
    opposites: Vec<(usize, String)>,
    parts: Vec<String>,
}
impl Query {
    fn new(terms: &[String]) -> Self {
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
    fn is_match(&self, row: &Value, kind: &Kind) -> bool {
        words::holds_all(&self.words, &joined(row, kind.fields))
    }
    /// Whether any row can be partly held: a shorter query cannot be, unless a row
    /// holds a modal word's opposite, and stop words alone name nothing.
    fn has_partials(&self) -> bool {
        self.words.len() + usize::from(!self.opposites.is_empty()) > MIN_PARTIAL_WORDS
            && !words::is_stop(&self.words[0])
    }
    /// The sort key and matched words of a row whose heading and body hold at least
    /// MIN_PARTIAL_WORDS of the words, a modal word's opposite counting but reported as
    /// missing. Every word holding a digit must be there as well.
    fn partial(&self, context: &Context, row: &Value, kind: &Kind) -> Option<(Key, Vec<bool>)> {
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
const MIN_PARTIAL_WORDS: usize = 2;

/// One page of the rows holding every word of the query, then of those holding some.
/// A partial hit always ranks below every match and is never reported as one.
#[allow(clippy::too_many_arguments)]
fn search_rows(
    context: &Context,
    files: &[PathBuf],
    query: &Query,
    scope_terms: &[String],
    scope: &dyn Fn(&Value) -> bool,
    kind: &Kind,
    build: &dyn Fn(&Value) -> Result<Value, String>,
    limit: usize,
    offset: usize,
    corpus: Value,
) -> Result<Value, String> {
    let end = offset + limit;
    let prefilter = [query.parts.clone(), scope_terms.to_vec()].concat();
    let full = scan(context, files, &prefilter, None, end + 1, &mut |row| {
        if !scope(row) || !query.is_match(row, kind) {
            return Ok(None);
        }
        let tier = tier(&query.words, &query.terms, kind.heading, kind.body, row);
        Ok(Some(((tier, priority(context, row), 0), build(row)?)))
    })?;
    let partial = if full.len() <= end && query.has_partials() {
        let count = end + 1 - full.len();
        let least = Some(MIN_PARTIAL_WORDS);
        scan(context, files, &query.parts, least, count, &mut |row| {
            let Some((key, found)) = query.partial(context, row, kind) else {
                return Ok(None);
            };
            if query.is_match(row, kind) || !scope(row) {
                return Ok(None);
            }
            let mut value = build(row)?;
            let pick = |held: bool| -> Vec<&String> {
                query
                    .words
                    .iter()
                    .zip(&found)
                    .filter(|(_, f)| **f == held)
                    .map(|(w, _)| w)
                    .collect()
            };
            value["match"] = json!("partial");
            value["matched_words"] = json!(pick(true));
            value["missing_words"] = json!(pick(false));
            Ok(Some((key, value)))
        })?
    } else {
        Vec::new()
    };
    let (matched, more) = (full.len(), full.len() + partial.len() > end);
    let mut hits: Vec<Value> = full
        .into_iter()
        .chain(partial)
        .skip(offset)
        .take(limit)
        .collect();
    let partial_hits = hits.split_off(matched.saturating_sub(offset).min(hits.len()));
    let mut result = page(
        hits,
        end,
        more,
        policy("NOTICE"),
        context.limit("MAX_OFFSET", 10000),
    );
    result["corpus"] = corpus;
    let mut dropped: Vec<&String> = Vec::new();
    for term in &query.terms {
        if words::is_stop(term) && !query.parts.contains(term) && !dropped.contains(&term) {
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
    Ok(result)
}

fn in_force(input: &Value, row: &Value) -> bool {
    input["in_force_only"] == false || row["version_is_current"] != false
}

fn search(context: &Context, input: &Value, rates: bool) -> Result<Value, String> {
    let on = as_at::checked(&input["as_at"])?;
    let terms = query_terms(input["query"].as_str().unwrap(), "query")?;
    let filter = optional_words(input, if rates { "topic" } else { "act" })?;
    let wanted_year = input["year"]
        .as_str()
        .map(|s| s.trim_matches(py::is_space))
        .filter(|s| !s.is_empty())
        .map(py::casefold);
    let root = root(context)?;
    let (offset, limit) = (
        input["offset"].as_u64().unwrap() as usize,
        input["limit"].as_u64().unwrap() as usize,
    );
    let query = Query::new(&terms);
    let cap = context.limit("SEARCH_TEXT_CHARS", 1200);
    if rates {
        let path = root.join("rates/rates.jsonl");
        if !path.is_file()
            || linked(&path).unwrap_or(true)
            || linked(path.parent().unwrap()).unwrap_or(true)
        {
            return Err("The configured corpus has no rates/rates.jsonl index.".into());
        }
        let scope = |row: &Value| {
            in_force(input, row)
                && holds(row, &["topic"], &filter)
                && wanted_year.as_ref().is_none_or(|wanted| {
                    row["years"].as_array().is_some_and(|years| {
                        years.iter().any(|v| py::casefold(&py_string(v)) == *wanted)
                    })
                })
        };
        let build = |row: &Value| Ok(rate(row, cap));
        let corpus = provenance(&root);
        return search_rows(
            context,
            &[path],
            &query,
            &filter,
            &scope,
            &RATES,
            &build,
            limit,
            offset,
            corpus,
        );
    }
    let files = indexes(context, &root)?;
    if let Some(on) = on {
        return dated_search(context, &files, &on, &query, &filter, limit, offset, &root);
    }
    let scope = |row: &Value| in_force(input, row) && holds(row, &["act"], &filter);
    let build = |row: &Value| Ok(section(row, cap));
    let corpus = provenance(&root);
    let mut result = search_rows(
        context, &files, &query, &filter, &scope, &SECTIONS, &build, limit, offset, corpus,
    )?;
    if result["matches"].as_array().unwrap().is_empty() && offset == 0 {
        result["notice"] = json!(format!(
            "{} {}",
            result["notice"].as_str().unwrap(),
            policy("NO_MATCH_HINT")
        ));
    }
    Ok(result)
}

fn window_error(row: &Value) -> String {
    format!(
        "Rows of {} disagree about their compilation window; the title's index is broken: tell the user, and do not retry.",
        as_at::register_id(row)
    )
}

/// The first row, after verifying every row shares its title and compilation window.
fn first_row(path: &Path, on: &str) -> Result<Option<Value>, String> {
    let mut first: Option<(Value, Value)> = None;
    for row in Rows::open(path, &[]).map_err(|_| READ_ERROR)? {
        let row = row.map_err(|_| READ_ERROR)?;
        let found = as_at::standing(&row, on)?;
        match &first {
            None => first = Some((row, found)),
            Some((head, window)) => {
                if found != *window || row["register_id"] != head["register_id"] {
                    return Err(window_error(&row));
                }
            }
        }
    }
    Ok(first.map(|(row, _)| row))
}

/// A row of a title held for `on`, with its standing. A row disagreeing with its title
/// is a broken index: a dated result holds only captured rows.
fn dated_section(row: &Value, cap: usize, on: &str) -> Result<Value, String> {
    let found = as_at::standing(row, on)?;
    if found["status"] != as_at::CAPTURED {
        return Err(window_error(row));
    }
    let mut value = section(row, cap);
    value["as_at"] = found;
    Ok(value)
}

/// What a date withheld from one title, in the corpus builder's field names.
fn refusal(row: &Value, found: &Value, matching: usize, partial: usize) -> Value {
    json!({"refused":true,"register_id":string(row,"register_id"),"act":string(row,"act"),
        "matching_rows":matching,"partial_rows":partial,"as_at":found})
}

/// Count the same full and partial matches as a search, without retaining their rows.
fn withheld_counts(
    context: &Context,
    path: &Path,
    query: &Query,
    act_terms: &[String],
    scope: &dyn Fn(&Value) -> bool,
) -> Result<(usize, usize), String> {
    let prefilter = [query.parts.clone(), act_terms.to_vec()].concat();
    let mut matching = 0;
    for row in Rows::open(path, &prefilter).map_err(|_| READ_ERROR)? {
        let row = row.map_err(|_| READ_ERROR)?;
        matching += usize::from(scope(&row) && query.is_match(&row, &SECTIONS));
    }
    let mut partial = 0;
    if query.has_partials() {
        let rows = Rows::open_some(path, &query.parts, Some(MIN_PARTIAL_WORDS));
        for row in rows.map_err(|_| READ_ERROR)? {
            let row = row.map_err(|_| READ_ERROR)?;
            partial += usize::from(
                query.partial(context, &row, &SECTIONS).is_some()
                    && !query.is_match(&row, &SECTIONS)
                    && scope(&row),
            );
        }
    }
    Ok((matching, partial))
}

/// One page from titles held for `on`, and a refusal for each title that holds no
/// answer: one holding rows the search would have returned, or one `act` names. If
/// the whole search has no eligible rows, every title in scope that is not captured
/// is refused. The status replaces in_force_only.
#[allow(clippy::too_many_arguments)]
fn dated_search(
    context: &Context,
    files: &[PathBuf],
    on: &str,
    query: &Query,
    act_terms: &[String],
    limit: usize,
    offset: usize,
    root: &Path,
) -> Result<Value, String> {
    let in_act =
        |row: &Value| act_terms.is_empty() || holds(row, &["act", "register_id"], act_terms);
    let mut titles = Vec::new();
    let mut size = 0;
    for path in files {
        bounded_size(context, path, &mut size)?;
        if let Some(row) = first_row(path, on)? {
            let found = as_at::standing(&row, on)?;
            let head = json!({"register_id":row["register_id"],"act":row["act"]});
            titles.push((path.clone(), head, found));
        }
    }
    let held: Vec<PathBuf> = titles
        .iter()
        .filter(|(_, _, found)| found["status"] == as_at::CAPTURED)
        .map(|(path, _, _)| path.clone())
        .collect();
    let cap = context.limit("SEARCH_TEXT_CHARS", 1200);
    let build = |row: &Value| dated_section(row, cap, on);
    let corpus = provenance(root);
    let mut result = search_rows(
        context, &held, query, act_terms, &in_act, &SECTIONS, &build, limit, offset, corpus,
    )?;
    let empty = |page: &Value| {
        page["matches"].as_array().unwrap().is_empty()
            && page["partial_hits"].as_array().unwrap().is_empty()
    };
    let mut no_rows = empty(&result);
    if no_rows && offset > 0 {
        let first = search_rows(
            context,
            &held,
            query,
            act_terms,
            &in_act,
            &SECTIONS,
            &|_| Ok(json!({})),
            1,
            0,
            json!({}),
        )?;
        no_rows = empty(&first);
    }
    let mut refusals = Vec::new();
    for (path, head, found) in &titles {
        if found["status"] == as_at::CAPTURED {
            continue;
        }
        let (matching, partial) = withheld_counts(context, path, query, act_terms, &in_act)?;
        if matching + partial > 0 || (in_act(head) && (no_rows || !act_terms.is_empty())) {
            refusals.push((
                priority(context, head),
                refusal(head, found, matching, partial),
            ));
        }
    }
    refusals.sort_by_key(|(priority, _)| *priority);
    let refusals: Vec<Value> = refusals.into_iter().map(|(_, r)| r).collect();
    let statuses: Vec<&Value> = titles
        .iter()
        .map(|(_, _, found)| &found["status"])
        .collect();
    Ok(dated_page(
        context,
        result.take(),
        refusals,
        &statuses,
        on,
        offset,
    ))
}

/// A dated search page with its refusals, its summary and the limits of a status.
fn dated_page(
    context: &Context,
    mut result: Value,
    refusals: Vec<Value>,
    statuses: &[&Value],
    on: &str,
    offset: usize,
) -> Value {
    let listed = context.limit("MAX_REFUSALS", 20);
    let mut notice = vec![result["notice"].as_str().unwrap().to_owned()];
    if !refusals.is_empty() {
        notice.push(policy("REFUSED_NOTICE").into());
    } else if result["matches"].as_array().unwrap().is_empty() && offset == 0 {
        notice.push(policy("NO_MATCH_HINT").into());
    }
    if refusals.len() > listed {
        notice.push(
            policy("OMITTED_REFUSALS")
                .replace("{listed}", &listed.to_string())
                .replace("{total}", &refusals.len().to_string()),
        );
    }
    notice.push(policy("AS_AT_NOTICE").into());
    let mut unlisted = serde_json::Map::new();
    for item in refusals.iter().skip(listed) {
        let status = item["as_at"]["status"].as_str().unwrap();
        let ids = unlisted.entry(status).or_insert_with(|| json!([]));
        ids.as_array_mut()
            .unwrap()
            .push(item["register_id"].clone());
    }
    let rows = result["matches"].as_array().unwrap().len()
        + result["partial_hits"].as_array().unwrap().len();
    let withheld: u64 = refusals
        .iter()
        .map(|r| r["matching_rows"].as_u64().unwrap() + r["partial_rows"].as_u64().unwrap())
        .sum();
    let mut titles = serde_json::Map::new();
    for status in as_at::STATUSES {
        titles.insert(
            status.into(),
            json!(statuses.iter().filter(|s| **s == status).count()),
        );
    }
    result["notice"] = json!(notice.join(" "));
    result["as_at_summary"] = json!({"date":on,"rows":rows,"withheld_rows":withheld,
        "refused_titles":refusals.len(),"unlisted_titles":unlisted,"titles":titles,
        "covers":policy("COVERS")});
    result["refusals"] = json!(refusals.into_iter().take(listed).collect::<Vec<_>>());
    result
}

/// The cited row of a title index, if any, and up to `near` rows on each side.
fn around(
    path: &Path,
    id: &str,
    near: usize,
) -> Result<(Option<Value>, Vec<Value>, Vec<Value>), String> {
    let needle = if near > 0 || id.contains(['"', '\\']) {
        vec![]
    } else {
        vec![py::casefold(id)]
    };
    let (mut before, mut after, mut found) = (VecDeque::new(), Vec::new(), None);
    let error = "Cannot read that UTF-8 JSONL index in the configured corpus.";
    for row in Rows::open(path, &needle).map_err(|_| error)? {
        let row = row.map_err(|_| error)?;
        if found.is_none() {
            if row["row_id"] == id {
                found = Some(row);
                if near == 0 {
                    break;
                }
            } else if near > 0 {
                before.push_back(row);
                if before.len() > near {
                    before.pop_front();
                }
            }
        } else {
            after.push(row);
            if after.len() >= near {
                break;
            }
        }
    }
    Ok((found, before.into(), after))
}

fn read(context: &Context, input: &Value) -> Result<Value, String> {
    let on = as_at::checked(&input["as_at"])?;
    let id = input["row_id"].as_str().unwrap();
    if !ROW_ID.matches_start(id) {
        return Err("Use a row_id returned by search_tax_legislation.".into());
    }
    let root = root(context)?;
    let path = root
        .join("markdown")
        .join(id.split(':').next().unwrap())
        .join("sections.jsonl");
    if !path.is_file()
        || [
            &path,
            path.parent().unwrap(),
            path.parent().unwrap().parent().unwrap(),
        ]
        .iter()
        .any(|p| linked(p).unwrap_or(true))
    {
        return Err("No title index for that row_id in the configured corpus.".into());
    }
    if let Some(on) = &on {
        first_row(&path, on)?;
    }
    let near = input["neighbours"].as_u64().unwrap() as usize;
    let start = input["start"].as_u64().unwrap() as usize;
    let (found, before, after) = around(&path, id, near)?;
    let mut found = found.ok_or("No section with that row_id in the configured corpus.")?;
    let standing = on
        .as_deref()
        .map(|on| as_at::standing(&found, on))
        .transpose()?;
    let cap = context.limit("READ_TEXT_CHARS", 12000);
    if let Some(standing) = standing.as_ref().filter(|s| s["status"] != as_at::CAPTURED) {
        let withheld = 1 + before.len() + after.len();
        return Ok(withheld_read(&root, &found, standing, start, withheld, cap));
    }
    let whole = string(&found, "text").unwrap_or("").to_owned();
    let total = whole.chars().count();
    if start > 0 && start >= total {
        return Err(format!(
            "start is past the end of this provision, which has {total} characters."
        ));
    }
    let end = start.saturating_add(cap).min(total);
    found["text"] = json!(
        whole
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>()
    );
    let mut selected = section(&found, cap);
    selected["total_chars"] = json!(total);
    if start > 0 || end < total {
        selected["caveats"].as_array_mut().unwrap().insert(
            0,
            json!(
                policy("PART_CAVEAT")
                    .replace("{start}", &start.to_string())
                    .replace("{end}", &end.to_string())
                    .replace("{total}", &total.to_string())
            ),
        );
    }
    let search_cap = context.limit("SEARCH_TEXT_CHARS", 1200);
    let near = |rows: &[Value]| -> Result<Vec<Value>, String> {
        rows.iter()
            .map(|r| match &on {
                None => Ok(section(r, search_cap)),
                Some(on) => dated_section(r, search_cap, on),
            })
            .collect()
    };
    let mut notice = policy("NOTICE").to_owned();
    if let Some(standing) = standing {
        selected["as_at"] = standing;
        notice = format!("{notice} {}", policy("AS_AT_NOTICE"));
    }
    Ok(
        json!({"section":selected,"start":start,"next_start":(end<total).then_some(end),"before":near(&before)?,"after":near(&after)?,"corpus":provenance(&root),"notice":notice}),
    )
}

/// A read refused for a date: the cited row's citation and window, without its text.
fn withheld_read(
    root: &Path,
    row: &Value,
    found: &Value,
    start: usize,
    withheld: usize,
    cap: usize,
) -> Value {
    let mut empty = row.clone();
    empty["text"] = json!("");
    let mut selected = section(&empty, cap);
    selected["total_chars"] = json!(string(row, "text").unwrap_or("").chars().count());
    selected["caveats"].as_array_mut().unwrap().insert(
        0,
        json!(
            policy("WITHHELD_CAVEAT")
                .replace("{date}", found["date"].as_str().unwrap())
                .replace("{status}", found["status"].as_str().unwrap())
        ),
    );
    selected["as_at"] = found.clone();
    let notice = format!(
        "{} {} {}",
        policy("NOTICE"),
        policy("REFUSED_NOTICE"),
        policy("AS_AT_NOTICE")
    );
    json!({"section":selected,"start":start,"next_start":null,"before":[],"after":[],
        "refusals":[refusal(row, found, withheld, 0)],"corpus":provenance(root),"notice":notice})
}

/// How many lines are rows a search can return, and how many are not.
fn count_rows(path: &Path, required: &[&str]) -> Result<(usize, usize), String> {
    let (mut rows, mut unreadable) = (0, 0);
    for line in super::Lines::open(path).map_err(|e| e.to_string())? {
        let line = line.map_err(|e| e.to_string())?;
        let row = super::parse_json(&line).ok().filter(Value::is_object);
        if row.is_some_and(|row| required.iter().all(|key| row[*key].is_string())) {
            rows += 1;
        } else if !line.trim_matches(py::is_space).is_empty() {
            unreadable += 1;
        }
    }
    Ok((rows, unreadable))
}

/// What the configured corpus holds, for `aus-accounting-mcp doctor`.
pub(super) fn census(context: &Context) -> Result<Value, String> {
    let root = root(context)?;
    let files = indexes(context, &root)?;
    let (mut rows, mut unreadable, mut size) = (0, 0, 0);
    for path in &files {
        bounded_size(context, path, &mut size)?;
        let (held, bad) = count_rows(path, &["row_id", "text"])?;
        rows += held;
        unreadable += bad;
    }
    let path = root.join("rates/rates.jsonl");
    let mut rates = Value::Null;
    if path.is_file()
        && !linked(&path).unwrap_or(true)
        && !linked(path.parent().unwrap()).unwrap_or(true)
    {
        let (held, bad) = count_rows(&path, &["rate_id", "content"])?;
        rates = json!(held);
        unreadable += bad;
    }
    let entries = fs::read_dir(root.join("markdown"))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .count();
    Ok(
        json!({"indexes":files.len(),"rows":rows,"rate_rows":rates,"unreadable":unreadable,
        "skipped_entries":entries - files.len()}),
    )
}

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
pub(super) fn definitions(text: &str) -> Vec<(String, String)> {
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
fn define(context: &Context, input: &Value) -> Result<Value, String> {
    let term = input["term"].as_str().unwrap();
    let words = query_terms(term, "term")?;
    let wanted = plain(term);
    let act = optional_words(input, "act")?;
    let root = root(context)?;
    let (mut exact, mut partial, mut size, mut order) = (Vec::new(), BinaryHeap::new(), 0, 0);
    let maximum = context.limit("MAX_DEFINITIONS", 20);
    let prefilter = [words.clone(), act.clone()].concat();
    for path in indexes(context, &root)? {
        bounded_size(context, &path, &mut size)?;
        for row in Rows::open(&path, &prefilter).map_err(|_| READ_ERROR)? {
            let row = row.map_err(|_| READ_ERROR)?;
            if !DICTIONARY.matches_start(string(&row, "heading").unwrap_or(""))
                || (input["in_force_only"] != false && row["version_is_current"] == false)
                || !holds(&row, &["act"], &act)
            {
                continue;
            }
            let priority = priority(context, &row);
            for (head, body) in definitions(string(&row, "text").unwrap_or("")) {
                let key = plain(&head);
                let kind = if key.split(',').next().unwrap().trim_matches(py::is_space) == wanted {
                    "exact"
                } else if words.iter().all(|word| terms(&key).contains(word)) {
                    "partial"
                } else {
                    continue;
                };
                let mut definition = row.clone();
                definition["text"] = json!(body);
                let mut value = section(&definition, context.limit("SEARCH_TEXT_CHARS", 1200));
                value["head"] = json!(head);
                value["match"] = json!(kind);
                if kind == "exact" {
                    exact.push((priority, value));
                } else {
                    retain(
                        &mut partial,
                        Ranked {
                            key: (priority, order, 0),
                            value,
                        },
                        maximum + 1,
                    );
                    order += 1;
                }
            }
        }
    }
    exact.sort_by_key(|(p, _)| *p);
    let dropped = partial.len() > maximum;
    let mut found: Vec<_> = exact.into_iter().map(|(_, v)| v).collect();
    found.extend(
        partial
            .into_sorted_vec()
            .into_iter()
            .take(maximum)
            .map(|v| v.value),
    );
    let limit = input["limit"].as_u64().unwrap() as usize;
    let more = dropped || found.len() > limit;
    found.truncate(limit);
    Ok(
        json!({"term":term,"definitions":found,"has_more":more,"corpus":provenance(&root),"notice":format!("{} {}",policy("NOTICE"),policy("DEFINITION_NOTICE"))}),
    )
}
pub(super) fn evaluate(context: &Context, name: &str, input: &Value) -> Result<Value, String> {
    match name {
        "search_tax_legislation" => search(context, input, false),
        "search_tax_rates" => search(context, input, true),
        "read_tax_legislation_section" => read(context, input),
        "define_tax_term" => define(context, input),
        _ => unreachable!("validated corpus tool"),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// Python's test_a_title_changed_after_validation_cannot_return_uncaptured_text rewrites
    /// a title between validating it and reading its rows, which a frozen replay cannot do.
    /// The guard it proves is here: a row read for a date must itself be captured.
    #[test]
    fn a_row_changed_after_validation_is_refused_not_returned() {
        let changed = json!({"register_id": "C9999A10001", "compilation_date": "2098-12-01",
            "retrieved": "2099-01-01", "version_is_current": true, "superseded_from": null,
            "text": "The synthetic levy applies to a dated receipt."});
        assert_eq!(
            super::dated_section(&changed, 100, "2098-08-01").unwrap_err(),
            "Rows of C9999A10001 disagree about their compilation window; the title's index is broken: tell the user, and do not retry."
        );
        let mut held = changed.clone();
        held["compilation_date"] = json!("2098-07-01");
        let section = super::dated_section(&held, 100, "2098-08-01").unwrap();
        assert_eq!(section["as_at"]["status"], "captured");
    }

    /// Python's test_a_read_failure_while_counting_refusals_stops_the_search replaces the
    /// row reader with one that fails, which a frozen replay cannot do. A title that
    /// cannot be read while its withheld rows are counted stops the search.
    #[test]
    fn a_read_failure_while_counting_refusals_stops_the_search() {
        let context = super::Context {
            root: None,
            limits: Default::default(),
            denied: false,
            principal_acts: None,
            fault: None,
        };
        let gone = std::env::temp_dir()
            .join(format!("native-mcp-unreadable-{}", std::process::id()))
            .join("sections.jsonl");
        assert!(!gone.exists());
        let query = super::Query::new(&["synthetic".into(), "levy".into()]);
        assert_eq!(
            super::withheld_counts(&context, &gone, &query, &[], &|_| true).unwrap_err(),
            super::READ_ERROR
        );
    }
}
