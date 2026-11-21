// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::{Context, as_at, linked};
use super::dated::{agreeing, first_row, refusal};
use super::rank::section;
use super::rows::{Rows, policy, provenance, root, string};
use payday_super_checker::{py, pyre::Regex};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::LazyLock,
};

static ROW_ID: LazyLock<Regex> = LazyLock::new(|| Regex::python(policy("ROW_ID")));

/// The cited row, if any, with the rows before and after it.
type Around = (Option<Value>, Vec<Value>, Vec<Value>);

/// The cited row of a title index, if any, and up to `near` rows on each side.
fn around(path: &Path, id: &str, near: usize) -> Result<Around, String> {
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

fn section_path(context: &Context, id: &str) -> Result<(PathBuf, PathBuf), String> {
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
    Ok((root, path))
}

type AgreedRows = (Value, Vec<Value>, Vec<Value>, Option<Value>);

fn agreed_rows(
    path: &Path,
    id: &str,
    near: usize,
    on: &Option<String>,
) -> Result<AgreedRows, String> {
    let (found, before, after) = around(path, id, near)?;
    let found = found.ok_or("No section with that row_id in the configured corpus.")?;
    // With `on`, each row returned must agree with its title's first row, as `agreeing`
    // says: a read returns those rows or counts them as withheld.
    let standing = match on {
        None => None,
        Some(on) => {
            // An index emptied since `around` read it has no window, so no row agrees.
            let first = first_row(path, on)?.unwrap_or((json!({}), json!({})));
            let standing = agreeing(&found, on, &first)?;
            for row in before.iter().chain(&after) {
                agreeing(row, on, &first)?;
            }
            Some(standing)
        }
    };
    Ok((found, before, after, standing))
}

fn opened_text(
    found: &mut Value,
    start: usize,
    cap: usize,
) -> Result<(Value, usize, usize), String> {
    let whole = string(found, "text").unwrap_or("").to_owned();
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
    let mut selected = section(found, cap);
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
    Ok((selected, total, end))
}

// Each neighbour shares the cited row's standing, as checked above.
fn neighbour_sections(rows: &[Value], search_cap: usize, standing: &Option<Value>) -> Vec<Value> {
    rows.iter()
        .map(|r| {
            let mut value = section(r, search_cap);
            if let Some(standing) = standing {
                value["as_at"] = standing.clone();
            }
            value
        })
        .collect()
}

pub(super) fn read(context: &Context, input: &Value) -> Result<Value, String> {
    let on = as_at::checked(&input["as_at"])?;
    let id = input["row_id"].as_str().unwrap();
    let (root, path) = section_path(context, id)?;
    let near = input["neighbours"].as_u64().unwrap() as usize;
    let start = input["start"].as_u64().unwrap() as usize;
    let (mut found, before, after, standing) = agreed_rows(&path, id, near, &on)?;
    let cap = context.limit("READ_TEXT_CHARS", 12000);
    if let Some(standing) = standing.as_ref().filter(|s| s["status"] != as_at::CAPTURED) {
        let withheld = 1 + before.len() + after.len();
        return Ok(withheld_read(&root, &found, standing, start, withheld, cap));
    }
    let (mut selected, total, end) = opened_text(&mut found, start, cap)?;
    let search_cap = context.limit("SEARCH_TEXT_CHARS", 1200);
    let before = neighbour_sections(&before, search_cap, &standing);
    let after = neighbour_sections(&after, search_cap, &standing);
    let mut notice = policy("NOTICE").to_owned();
    if let Some(standing) = standing {
        selected["as_at"] = standing;
        notice = format!("{notice} {}", policy("AS_AT_NOTICE"));
    }
    Ok(
        json!({"section":selected,"start":start,"next_start":(end<total).then_some(end),"before":before,"after":after,"corpus":provenance(&root),"notice":notice}),
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
