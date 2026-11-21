// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::{Context, as_at, linked};
use super::dated::dated_search;
use super::query::{Query, RATES, SECTIONS, Search, search_rows};
use super::rank::{rate, section};
use super::rows::{
    holds, indexes, optional_words, policy, provenance, py_string, query_terms, root,
};
use payday_super_checker::py;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn in_force(input: &Value, row: &Value) -> bool {
    input["in_force_only"] == false || row["version_is_current"] != false
}

fn rate_scope(input: &Value, row: &Value, filter: &[String], year: &Option<String>) -> bool {
    in_force(input, row)
        && holds(row, &["topic"], filter)
        && year.as_ref().is_none_or(|wanted| {
            row["years"]
                .as_array()
                .is_some_and(|years| years.iter().any(|v| py::casefold(&py_string(v)) == *wanted))
        })
}

struct Prepared<'a> {
    context: &'a Context,
    input: &'a Value,
    on: Option<String>,
    filter: Vec<String>,
    year: Option<String>,
    root: PathBuf,
    offset: usize,
    limit: usize,
    query: Query,
    cap: usize,
}

fn prepare<'a>(
    context: &'a Context,
    input: &'a Value,
    field: &str,
) -> Result<Prepared<'a>, String> {
    let on = as_at::checked(&input["as_at"])?;
    let terms = query_terms(input["query"].as_str().unwrap(), "query")?;
    let filter = optional_words(input, field)?;
    let year = input["year"]
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
    Ok(Prepared {
        context,
        input,
        on,
        filter,
        year,
        root,
        offset,
        limit,
        query,
        cap,
    })
}

fn search_rates(prepared: Prepared<'_>) -> Result<Value, String> {
    let path = prepared.root.join("rates/rates.jsonl");
    if !path.is_file()
        || linked(&path).unwrap_or(true)
        || linked(path.parent().unwrap()).unwrap_or(true)
    {
        return Err("The configured corpus has no rates/rates.jsonl index.".into());
    }
    let scope = |row: &Value| rate_scope(prepared.input, row, &prepared.filter, &prepared.year);
    let build = |_: &Path, row: &Value| Ok(rate(row, prepared.cap));
    search_rows(Search {
        context: prepared.context,
        files: &[path],
        query: &prepared.query,
        scope_terms: &prepared.filter,
        scope: &scope,
        kind: &RATES,
        build: &build,
        limit: prepared.limit,
        offset: prepared.offset,
        corpus: provenance(&prepared.root),
    })
}

fn search_sections(prepared: Prepared<'_>) -> Result<Value, String> {
    let files = indexes(prepared.context, &prepared.root)?;
    if let Some(on) = &prepared.on {
        return dated_search(
            prepared.context,
            &files,
            on,
            &prepared.query,
            &prepared.filter,
            prepared.limit,
            prepared.offset,
            &prepared.root,
        );
    }
    let scope =
        |row: &Value| in_force(prepared.input, row) && holds(row, &["act"], &prepared.filter);
    let build = |_: &Path, row: &Value| Ok(section(row, prepared.cap));
    let mut result = search_rows(Search {
        context: prepared.context,
        files: &files,
        query: &prepared.query,
        scope_terms: &prepared.filter,
        scope: &scope,
        kind: &SECTIONS,
        build: &build,
        limit: prepared.limit,
        offset: prepared.offset,
        corpus: provenance(&prepared.root),
    })?;
    if result["matches"].as_array().unwrap().is_empty() && prepared.offset == 0 {
        result["notice"] = json!(format!(
            "{} {}",
            result["notice"].as_str().unwrap(),
            policy("NO_MATCH_HINT")
        ));
    }
    Ok(result)
}

pub(super) fn search(context: &Context, input: &Value, rates: bool) -> Result<Value, String> {
    let prepared = prepare(context, input, if rates { "topic" } else { "act" })?;
    if rates {
        search_rates(prepared)
    } else {
        search_sections(prepared)
    }
}
