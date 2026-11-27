// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::{Context, as_at};
use super::query::{MIN_PARTIAL_WORDS, Query, SECTIONS, Search, search_rows};
use super::rank::{priority, section};
use super::rows::{READ_ERROR, Rows, bounded_size, holds, policy, provenance, string};
use serde_json::{Map, Value, json};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

fn window_error(row: &Value) -> String {
    format!(
        "Rows of {} disagree about their compilation window; the title's index is broken: tell the user, and do not retry.",
        as_at::register_id(row)
    )
}

/// An index's first row's register_id and act, and that row's standing on one date.
pub(super) type First = (Value, Value);

/// The register_id and act of an index's first row, and that row's standing on `on`.
/// A title is classified from this row alone, so reading stops there; `agreeing`
/// checks each later row of the same index that a result uses against it.
pub(super) fn first_row(path: &Path, on: &str) -> Result<Option<First>, String> {
    let Some(row) = Rows::open(path, &[]).map_err(|_| READ_ERROR)?.next() else {
        return Ok(None);
    };
    let row = row.map_err(|_| READ_ERROR)?;
    let head = json!({"register_id":row["register_id"],"act":row["act"]});
    Ok(Some((head, as_at::standing(&row, on)?)))
}

/// The row's standing on `on`, refused unless it has `first`'s register_id and standing.
/// `first` is from the index the row came from. A row a result returns or counts that
/// disagrees with it is a broken index: using it would break the rule that a dated
/// result holds only rows its title's classification covers.
pub(super) fn agreeing(row: &Value, on: &str, (head, window): &First) -> Result<Value, String> {
    let found = as_at::standing(row, on)?;
    if row["register_id"] != head["register_id"] || found != *window {
        return Err(window_error(row));
    }
    Ok(found)
}

/// What a date withheld from one title, in the corpus builder's field names.
pub(super) fn refusal(row: &Value, found: &Value, matching: usize, partial: usize) -> Value {
    json!({"refused":true,"register_id":string(row,"register_id"),"act":string(row,"act"),
        "matching_rows":matching,"partial_rows":partial,"as_at":found})
}

/// Count the same full and partial matches as a search, without retaining their rows.
/// Each counted row must agree with `first`, the index's first row, as `agreeing` says.
pub(super) fn withheld_counts(
    context: &Context,
    path: &Path,
    query: &Query,
    act_terms: &[String],
    scope: &dyn Fn(&Value) -> bool,
    on: &str,
    first: &First,
) -> Result<(usize, usize), String> {
    let prefilter = [query.parts.clone(), act_terms.to_vec()].concat();
    let mut matching = 0;
    for row in Rows::open(path, &prefilter).map_err(|_| READ_ERROR)? {
        let row = row.map_err(|_| READ_ERROR)?;
        if scope(&row) && query.is_match(&row, &SECTIONS) {
            agreeing(&row, on, first)?;
            matching += 1;
        }
    }
    let mut partial = 0;
    if query.has_partials() {
        let rows = Rows::open_some(path, &query.parts, Some(MIN_PARTIAL_WORDS));
        for row in rows.map_err(|_| READ_ERROR)? {
            let row = row.map_err(|_| READ_ERROR)?;
            if query.partial(context, &row, &SECTIONS).is_some()
                && !query.is_match(&row, &SECTIONS)
                && scope(&row)
            {
                agreeing(&row, on, first)?;
                partial += 1;
            }
        }
    }
    Ok((matching, partial))
}

fn titles_for(
    context: &Context,
    files: &[PathBuf],
    on: &str,
) -> Result<Vec<(PathBuf, First)>, String> {
    let mut titles = Vec::new();
    let mut size = 0;
    for path in files {
        bounded_size(context, path, &mut size)?;
        if let Some(first) = first_row(path, on)? {
            titles.push((path.clone(), first));
        }
    }
    Ok(titles)
}

fn captured(titles: &[(PathBuf, First)]) -> (HashMap<&Path, &First>, Vec<PathBuf>) {
    let firsts: HashMap<&Path, &First> = titles
        .iter()
        .filter(|(_, (_, found))| found["status"] == as_at::CAPTURED)
        .map(|(path, first)| (path.as_path(), first))
        .collect();
    let held: Vec<PathBuf> = titles
        .iter()
        .filter(|(_, (_, found))| found["status"] == as_at::CAPTURED)
        .map(|(path, _)| path.clone())
        .collect();
    (firsts, held)
}

struct Scope<'a> {
    context: &'a Context,
    query: &'a Query,
    act_terms: &'a [String],
    in_act: &'a dyn Fn(&Value) -> bool,
    on: &'a str,
}

fn held_page(
    scope: &Scope<'_>,
    held: &[PathBuf],
    firsts: &HashMap<&Path, &First>,
    limit: usize,
    offset: usize,
    root: &Path,
) -> Result<Value, String> {
    let cap = scope.context.limit("SEARCH_TEXT_CHARS", 1200);
    let build = |path: &Path, row: &Value| -> Result<Value, String> {
        let mut value = section(row, cap);
        value["as_at"] = agreeing(row, scope.on, firsts[path])?;
        Ok(value)
    };
    search_rows(Search {
        context: scope.context,
        files: held,
        query: scope.query,
        scope_terms: scope.act_terms,
        scope: scope.in_act,
        kind: &SECTIONS,
        build: &build,
        limit,
        offset,
        corpus: provenance(root),
    })
}

fn page_empty(page: &Value) -> bool {
    page["matches"].as_array().unwrap().is_empty()
        && page["partial_hits"].as_array().unwrap().is_empty()
}

fn no_eligible_rows(
    scope: &Scope<'_>,
    held: &[PathBuf],
    result: &Value,
    offset: usize,
) -> Result<bool, String> {
    let mut no_rows = page_empty(result);
    if no_rows && offset > 0 {
        let first = search_rows(Search {
            context: scope.context,
            files: held,
            query: scope.query,
            scope_terms: scope.act_terms,
            scope: scope.in_act,
            kind: &SECTIONS,
            build: &|_, _| Ok(json!({})),
            limit: 1,
            offset: 0,
            corpus: json!({}),
        })?;
        no_rows = page_empty(&first);
    }
    Ok(no_rows)
}

fn collect_refusals(
    scope: &Scope<'_>,
    titles: &[(PathBuf, First)],
    no_rows: bool,
) -> Result<Vec<Value>, String> {
    let mut refusals = Vec::new();
    for (path, first) in titles {
        let (head, found) = first;
        if found["status"] == as_at::CAPTURED {
            continue;
        }
        let (matching, partial) = withheld_counts(
            scope.context,
            path,
            scope.query,
            scope.act_terms,
            scope.in_act,
            scope.on,
            first,
        )?;
        if matching + partial > 0
            || ((scope.in_act)(head) && (no_rows || !scope.act_terms.is_empty()))
        {
            refusals.push((
                priority(scope.context, head),
                refusal(head, found, matching, partial),
            ));
        }
    }
    refusals.sort_by_key(|(priority, _)| *priority);
    Ok(refusals.into_iter().map(|(_, refusal)| refusal).collect())
}

fn title_statuses(titles: &[(PathBuf, First)]) -> Vec<&Value> {
    titles
        .iter()
        .map(|(_, (_, found))| &found["status"])
        .collect()
}

fn finish_dated(
    scope: &Scope<'_>,
    titles: &[(PathBuf, First)],
    held: &[PathBuf],
    offset: usize,
    mut result: Value,
) -> Result<Value, String> {
    let no_rows = no_eligible_rows(scope, held, &result, offset)?;
    let refusals = collect_refusals(scope, titles, no_rows)?;
    Ok(dated_page(
        scope.context,
        result.take(),
        refusals,
        &title_statuses(titles),
        scope.on,
        offset,
    ))
}

/// One page from titles held for `on`, and a refusal for each title that holds no
/// answer: one holding rows the search would have returned, or one `act` names. If
/// the whole search has no eligible rows, every title in scope that is not captured
/// is refused. The status replaces in_force_only. Each title is classified from its first
/// row, and `agreeing` checks only the rows a page returns or a refusal counts against the
/// first row of their own index.
#[allow(clippy::too_many_arguments)]
pub(super) fn dated_search(
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
    let scope = Scope {
        context,
        query,
        act_terms,
        in_act: &in_act,
        on,
    };
    let titles = titles_for(context, files, on)?;
    let (firsts, held) = captured(&titles);
    let result = held_page(&scope, &held, &firsts, limit, offset, root)?;
    finish_dated(&scope, &titles, &held, offset, result)
}

fn dated_notice(result: &Value, refusals: &[Value], listed: usize, offset: usize) -> String {
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
    notice.join(" ")
}

fn unlisted_titles(refusals: &[Value], listed: usize) -> Map<String, Value> {
    let mut unlisted = Map::new();
    for item in refusals.iter().skip(listed) {
        let status = item["as_at"]["status"].as_str().unwrap();
        let ids = unlisted.entry(status).or_insert_with(|| json!([]));
        ids.as_array_mut()
            .unwrap()
            .push(item["register_id"].clone());
    }
    unlisted
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
    let notice = dated_notice(&result, &refusals, listed, offset);
    let unlisted = unlisted_titles(&refusals, listed);
    let rows = result["matches"].as_array().unwrap().len()
        + result["partial_hits"].as_array().unwrap().len();
    let withheld: u64 = refusals
        .iter()
        .map(|r| r["matching_rows"].as_u64().unwrap() + r["partial_rows"].as_u64().unwrap())
        .sum();
    let mut titles = Map::new();
    for status in as_at::STATUSES {
        titles.insert(
            status.into(),
            json!(statuses.iter().filter(|s| **s == status).count()),
        );
    }
    result["notice"] = json!(notice);
    result["as_at_summary"] = json!({"date":on,"rows":rows,"withheld_rows":withheld,
        "refused_titles":refusals.len(),"unlisted_titles":unlisted,"titles":titles,
        "covers":policy("COVERS")});
    result["refusals"] = json!(refusals.into_iter().take(listed).collect::<Vec<_>>());
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(label: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let dir = std::env::temp_dir().join(format!(
                "aus-accounting-mcp-dated-{label}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn test_context() -> Context {
        Context {
            root: None,
            limits: Default::default(),
            denied: false,
            principal_acts: None,
            fault: None,
        }
    }

    #[test]
    fn withheld_counts_distinguishes_full_and_partial_rows() {
        let dir = TempDir::new("withheld");
        let index = dir.0.join("sections.jsonl");
        let base = |n: u32, text: &str| {
            json!({
                "register_id": "C9999A10002",
                "act": "Synthetic Act 2098",
                "row_id": format!("C9999A10002:{n:04}:{n}"),
                "section": n.to_string(),
                "heading": format!("{n} Rule"),
                "text": text,
                "compilation_date": "2098-07-01",
                "retrieved": "2099-01-01",
                "version_is_current": true,
                "superseded_from": null
            })
        };
        let rows = [
            base(1, "Synthetic levy receipt rule."),
            base(2, "Synthetic levy partial row."),
        ];
        fs::write(&index, format!("{}\n{}\n", rows[0], rows[1])).unwrap();
        let on = "2098-01-01";
        let first = first_row(&index, on).unwrap().unwrap();
        let query = Query::new(&["synthetic".into(), "levy".into(), "receipt".into()]);
        let counts =
            withheld_counts(&test_context(), &index, &query, &[], &|_| true, on, &first).unwrap();
        assert_eq!(counts, (1, 1));
    }

    #[test]
    fn no_eligible_rows_propagates_read_error_on_offset_recheck() {
        let context = test_context();
        let query = Query::new(&["synthetic".into(), "levy".into()]);
        let scope = Scope {
            context: &context,
            query: &query,
            act_terms: &[],
            in_act: &|_| true,
            on: "2098-08-01",
        };
        let gone = TempDir::new("missing").0.join("missing.jsonl");
        let empty = json!({"matches": [], "partial_hits": []});
        let err = no_eligible_rows(&scope, &[gone], &empty, 1).unwrap_err();
        assert_eq!(err, READ_ERROR);
    }
}
