//! Point-in-time legislation indexes supplied by the operator. No downloads.
use super::{Context, POLICY, linked, page, terms};
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
static ROW_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(policy("ROW_ID")));
static DICTIONARY: LazyLock<Regex> =
    LazyLock::new(|| Regex::ignore_case(policy("DICTIONARY_HEADING")));
static DEFINITION: LazyLock<Regex> = LazyLock::new(|| Regex::new(policy("DEFINITION_HEAD")));
static SUBSECTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(policy("SUBSECTION_LABEL")));

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
    context.root("AUS_ACCOUNTING_CORPUS_ROOT", "legislation corpus folder")
}
fn words(query: &str, field: &str) -> Result<Vec<String>, String> {
    let result = terms(query);
    if result.is_empty() {
        Err(format!("Supply at least one word in {field}."))
    } else {
        Ok(result)
    }
}
fn optional_words(input: &Value, field: &str) -> Result<Vec<String>, String> {
    match input[field].as_str().filter(|s| !s.is_empty()) {
        Some(text) => words(text, field),
        None => Ok(Vec::new()),
    }
}
pub(super) fn holds(row: &Value, fields: &[&str], words: &[String]) -> bool {
    let joined = py::casefold(
        &fields
            .iter()
            .filter_map(|f| string(row, f))
            .collect::<Vec<_>>()
            .join(" "),
    );
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
struct Rows {
    lines: super::Lines,
    filter: Vec<String>,
}
impl Rows {
    fn open(path: &Path, filter: &[String]) -> io::Result<Self> {
        Ok(Self {
            lines: super::Lines::open(path)?,
            filter: filter.to_vec(),
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
                if !self.filter.iter().all(|term| folded.contains(term)) {
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

pub(super) fn tier(words: &[String], headings: &[&str], bodies: &[&str], row: &Value) -> usize {
    let heading_words: Vec<_> = headings
        .iter()
        .filter_map(|field| string(row, field))
        .map(terms)
        .collect();
    let phrase = |parts: &[String]| {
        !words.is_empty() && parts.windows(words.len()).any(|window| window == words)
    };
    if heading_words.iter().any(|parts| phrase(parts)) {
        return 0;
    }
    if heading_words
        .iter()
        .any(|parts| words.iter().all(|w| parts.contains(w)))
    {
        return 1;
    }
    if bodies
        .iter()
        .filter_map(|field| string(row, field))
        .any(|body| phrase(&terms(body)))
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
    let (text, total, caveats) = truncated(string(row, "content").unwrap_or(""), cap);
    let mut value = json!({"content":text,"total_chars":total,"caveats":caveats});
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

pub(super) struct Ranked {
    pub key: (usize, usize, usize),
    pub value: Value,
}
impl PartialEq for Ranked {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for Ranked {}
impl Ord for Ranked {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}
impl PartialOrd for Ranked {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
pub(super) fn retain(best: &mut BinaryHeap<Ranked>, item: Ranked, cap: usize) {
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

fn search(context: &Context, input: &Value, rates: bool) -> Result<Value, String> {
    let words = words(input["query"].as_str().unwrap(), "query")?;
    let filter = optional_words(input, if rates { "topic" } else { "act" })?;
    let wanted_year = input["year"]
        .as_str()
        .map(|s| s.trim_matches(py::is_space))
        .filter(|s| !s.is_empty())
        .map(py::casefold);
    let root = root(context)?;
    let files = if rates {
        let path = root.join("rates/rates.jsonl");
        if !path.is_file()
            || linked(&path).unwrap_or(true)
            || linked(path.parent().unwrap()).unwrap_or(true)
        {
            return Err("The configured corpus has no rates/rates.jsonl index.".into());
        }
        vec![path]
    } else {
        indexes(context, &root)?
    };
    let offset = input["offset"].as_u64().unwrap() as usize;
    let limit = input["limit"].as_u64().unwrap() as usize;
    let mut best = BinaryHeap::new();
    let (mut size, mut order) = (0, 0);
    let prefilter = [words.clone(), filter.clone()].concat();
    for path in files {
        bounded_size(context, &path, &mut size)?;
        for row in Rows::open(&path, &prefilter).map_err(|_| READ_ERROR)? {
            let row = row.map_err(|_| READ_ERROR)?;
            if !rates && input["in_force_only"] != false && row["version_is_current"] == false {
                continue;
            }
            if !holds(
                &row,
                if rates { RATE_FIELDS } else { SECTION_FIELDS },
                &words,
            ) || !holds(&row, if rates { &["topic"] } else { &["act"] }, &filter)
            {
                continue;
            }
            if rates
                && wanted_year.as_ref().is_some_and(|wanted| {
                    !row["years"].as_array().is_some_and(|years| {
                        years.iter().any(|v| py::casefold(&py_string(v)) == *wanted)
                    })
                })
            {
                continue;
            }
            let key = (
                tier(
                    &words,
                    if rates {
                        &["heading", "topic"]
                    } else {
                        &["heading"]
                    },
                    if rates { &["content"] } else { &["text"] },
                    &row,
                ),
                priority(context, &row),
                order,
            );
            let cap = context.limit("SEARCH_TEXT_CHARS", 1200);
            retain(
                &mut best,
                Ranked {
                    key,
                    value: if rates {
                        rate(&row, cap)
                    } else {
                        section(&row, cap)
                    },
                },
                offset + limit + 1,
            );
            order += 1;
        }
    }
    let best = best.into_sorted_vec();
    let more = best.len() > offset + limit;
    let matches = best
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|v| v.value)
        .collect::<Vec<_>>();
    let empty = matches.is_empty();
    let mut result = page(
        matches,
        offset + limit,
        more,
        policy("NOTICE"),
        context.limit("MAX_OFFSET", 10000),
    );
    result["corpus"] = provenance(&root);
    if !rates && empty && offset == 0 {
        result["notice"] = json!(format!(
            "{} {}",
            result["notice"].as_str().unwrap(),
            policy("NO_MATCH_HINT")
        ));
    }
    Ok(result)
}

fn read(context: &Context, input: &Value) -> Result<Value, String> {
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
    let near = input["neighbours"].as_u64().unwrap() as usize;
    let start = input["start"].as_u64().unwrap() as usize;
    let needle = if near > 0 || id.contains(['"', '\\']) {
        vec![]
    } else {
        vec![py::casefold(id)]
    };
    let (mut before, mut after, mut found) = (VecDeque::new(), Vec::new(), None);
    let error = "Cannot read that UTF-8 JSONL index in the configured corpus.";
    for row in Rows::open(&path, &needle).map_err(|_| error)? {
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
    let mut found = found.ok_or("No section with that row_id in the configured corpus.")?;
    let whole = string(&found, "text").unwrap_or("").to_owned();
    let total = whole.chars().count();
    if start > 0 && start >= total {
        return Err(format!(
            "start is past the end of this provision, which has {total} characters."
        ));
    }
    let cap = context.limit("READ_TEXT_CHARS", 12000);
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
    Ok(
        json!({"section":selected,"start":start,"next_start":(end<total).then_some(end),"before":before.iter().map(|r|section(r,search_cap)).collect::<Vec<_>>(),"after":after.iter().map(|r|section(r,search_cap)).collect::<Vec<_>>(),"corpus":provenance(&root),"notice":policy("NOTICE")}),
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
    let words = words(term, "term")?;
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
