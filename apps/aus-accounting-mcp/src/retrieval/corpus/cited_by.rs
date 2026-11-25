// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::Context;
use super::rank::{priority, section};
use super::rows::{READ_ERROR, Rows, bounded_size, indexes, policy, provenance, string};
use payday_super_checker::{py, pyre::Regex};
use serde_json::{Value, json};
use std::{collections::HashSet, path::Path, sync::LazyLock};

static LABEL_ONLY: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("SECTION_LABEL")));
static LABEL_SEARCH: LazyLock<Regex> =
    LazyLock::new(|| Regex::python(&format!("({})", policy("SECTION_LABEL"))));
static SUBSECTIONS: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("SUBSECTIONS")));
static OTHER_TITLE: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("OTHER_TITLE")));
static CITATION: LazyLock<Regex> = LazyLock::new(|| {
    let prefix = concat!(
        r"(?:[Ss\u017f][Uu][Bb])?",
        r"(?:[Ss\u017f][Ee][Cc][Tt][Ii\u0130\u0131][Oo][Nn][Ss\u017f]?|[Ss\u017f][Ss\u017f]?\.?)"
    );
    let expanded = policy("CITATION").replacen(r"(?i:(?:sub)?(?:sections?|ss?\.?))", prefix, 1);
    Regex::python(&format!("(?P<whole>{expanded})"))
});

struct Target<'a> {
    context: &'a Context,
    label: &'a str,
    act: &'a str,
    register_id: &'a Value,
    variants: &'a [String],
    in_force_only: bool,
    cap: usize,
}

fn cite_plain(text: &str) -> String {
    text.replace('\u{2011}', "-").replace('\u{a0}', " ")
}

fn act_name(text: &str) -> String {
    let folded = super::define::plain(text);
    folded.strip_prefix("the ").unwrap_or(&folded).to_owned()
}

fn char_to_byte(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map_or(text.len(), |(byte, _)| byte)
}

fn labels_contain(labels: &str, wanted: &str) -> bool {
    let stripped = SUBSECTIONS.sub(" ", labels);
    let found = std::cell::Cell::new(false);
    LABEL_SEARCH.sub_with(&stripped, |item| {
        if item.group(1).as_deref() == Some(wanted) {
            found.set(true);
        }
        String::new()
    });
    found.get()
}

fn cited_labels(text: &str, label: &str, act: &str, internal: bool) -> Vec<(usize, usize)> {
    let plain = cite_plain(text);
    let hits = std::cell::RefCell::new(Vec::new());
    CITATION.sub_with(&plain, |found| {
        if let Some(labels) = found.name("labels")
            && labels_contain(&labels, label)
        {
            let keep = match found.name("act") {
                Some(named) => act_name(&named) == act,
                None if internal => {
                    found.name("this").is_some()
                        || !OTHER_TITLE.matches_start(&plain[char_to_byte(&plain, found.end)..])
                }
                None => false,
            };
            if keep {
                let len = found.name("whole").unwrap().chars().count();
                hits.borrow_mut().push((found.end - len, found.end));
            }
        }
        String::new()
    });
    hits.into_inner()
}

fn citing_entry(row: &Value, hits: &[(usize, usize)], cap: usize) -> Value {
    let text = string(row, "text").unwrap_or("");
    let first_start = char_to_byte(text, hits[0].0);
    let first_end = char_to_byte(text, hits[0].1);
    let start = text[..first_start].rfind("\n\n").map_or(0, |i| i + 2);
    let end = text[first_end..]
        .find("\n\n")
        .map_or(text.len(), |i| first_end + i);
    let paragraph = text[start..end].trim_matches(py::is_space);
    let mut copy = row.clone();
    copy["text"] = json!(paragraph);
    let mut entry = section(&copy, cap);
    let mut seen = HashSet::new();
    let mut citations = Vec::new();
    for &(s, e) in hits {
        let slice = &text[char_to_byte(text, s)..char_to_byte(text, e)];
        let normalized: Vec<_> = slice
            .split(py::is_space)
            .filter(|w| !w.is_empty())
            .collect();
        let written = normalized.join(" ");
        if seen.insert(written.clone()) {
            citations.push(written);
        }
    }
    entry["citations"] = json!(citations);
    entry
}

fn scan_index(
    path: &Path,
    target: &Target<'_>,
    found: &mut Vec<(usize, usize, Value)>,
) -> Result<(), String> {
    for row in Rows::open_some(path, target.variants, Some(1)).map_err(|_| READ_ERROR)? {
        let citing = row.map_err(|_| READ_ERROR)?;
        if target.in_force_only && citing["version_is_current"] == false {
            continue;
        }
        let internal = citing.get("register_id").unwrap_or(&Value::Null) == target.register_id;
        let own = cite_plain(string(&citing, "section").unwrap_or(""));
        if internal && own.trim_matches(py::is_space) == target.label {
            continue;
        }
        let hits = cited_labels(
            string(&citing, "text").unwrap_or(""),
            target.label,
            target.act,
            internal,
        );
        if !hits.is_empty() {
            let key = priority(target.context, &citing);
            let order = found.len();
            found.push((key, order, citing_entry(&citing, &hits, target.cap)));
        }
    }
    Ok(())
}

pub(super) fn cited_by(context: &Context, input: &Value) -> Result<Value, String> {
    let row_id = input["row_id"].as_str().unwrap();
    let (root, row) = super::read::read_row(context, row_id)?;
    let label = cite_plain(string(&row, "section").unwrap_or(""))
        .trim_matches(py::is_space)
        .to_owned();
    if !LABEL_ONLY.fullmatch(&label) {
        return Err("That row has no section number that another provision can cite.".into());
    }
    let act = act_name(string(&row, "act").unwrap_or(""));
    let variants = [
        py::casefold(&label),
        py::casefold(&label.replace('-', "\u{2011}")),
    ];
    let target = Target {
        context,
        label: &label,
        act: &act,
        register_id: row.get("register_id").unwrap_or(&Value::Null),
        variants: &variants,
        in_force_only: input["in_force_only"] != false,
        cap: context.limit("CITING_TEXT_CHARS", 600),
    };
    let mut found = Vec::new();
    let mut size = 0;
    for path in indexes(context, &root)? {
        bounded_size(context, &path, &mut size)?;
        scan_index(&path, &target, &mut found)?;
    }
    found.sort_by_key(|(p, order, _)| (*p, *order));
    let limit = input["limit"].as_u64().unwrap() as usize;
    let has_more = found.len() > limit;
    let cited_by: Vec<_> = found.into_iter().take(limit).map(|(_, _, e)| e).collect();
    Ok(json!({
        "provision": {
            "row_id": string(&row, "row_id"),
            "register_id": string(&row, "register_id"),
            "act": string(&row, "act"),
            "section": string(&row, "section"),
            "heading": string(&row, "heading"),
        },
        "cited_by": cited_by,
        "has_more": has_more,
        "corpus": provenance(&root),
        "notice": format!("{} {}", policy("NOTICE"), policy("CITED_BY_NOTICE")),
    }))
}
