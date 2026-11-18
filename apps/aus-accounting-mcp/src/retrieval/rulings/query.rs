// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::super::{
    Context, Lines,
    corpus::{self, Ranked},
    page, parse_json, position, terms, words,
};
use super::runs::catalogue;
use super::{Catalogue, ROW_REF, Run, canonical, family_priority, policy, withdrawal};
use serde_json::{Value, json};
use std::collections::{BinaryHeap, HashSet, VecDeque};
#[cfg(test)]
use std::fs;

fn text(row: &Value, field: &str, cap: usize, caveat: &str) -> (String, usize, Vec<String>) {
    let text = row[field].as_str().unwrap_or("");
    let total = text.chars().count();
    (
        text.chars().take(cap).collect(),
        total,
        if total > cap {
            vec![
                caveat
                    .replace("{kept}", &cap.to_string())
                    .replace("{total}", &total.to_string()),
            ]
        } else {
            vec![]
        },
    )
}
fn paragraph(
    row: &Value,
    run: &Run,
    key: &str,
    line: usize,
    catalogue: &Catalogue,
    cap: usize,
) -> Result<Value, String> {
    let (text, total, mut caveats) = text(row, "text", cap, policy("TRUNCATED_CAVEAT"));
    let (status, _) = withdrawal(row)?;
    let (notice, _, extra) = self::text(
        row,
        "withdrawal_notice",
        cap,
        "Withdrawal notice is truncated at {kept} of {total} characters. Read source_url.",
    );
    caveats.extend(extra);
    if status == "NOTICE_PRESENT" {
        caveats.push(format!("A withdrawal notice was present in the page captured on {}. Check its effect and applicable period on the ATO Legal Database before reliance.",row["fetched_on"].as_str().unwrap()));
    }
    let serving = catalogue.owner.get(key) == Some(&run.name);
    if !serving {
        caveats.push(policy("REPLACED_CAVEAT").into());
    }
    if row["family"] == policy("EDITED_FAMILY") {
        caveats.push(policy("EDITED_CAVEAT").into());
    }
    if row["numbering"] == "sequential" {
        caveats.push(policy("SEQUENTIAL_CAVEAT").into());
    }
    let mut value = json!({"row_ref":format!("{}|{key}|{line}",run.name),"text":text,"total_chars":total,"withdrawal_status":status,"withdrawal_notice":notice,"run":run.name,"serving":serving,"caveats":caveats});
    for field in [
        "docid",
        "family",
        "title",
        "paragraph",
        "numbering",
        "heading",
        "source_url",
        "source_sha256",
        "fetched_on",
        "licence_basis",
    ] {
        value[field] = json!(row[field].as_str());
    }
    Ok(value)
}
fn after_catalogue(context: &Context, catalogue: &Catalogue, read: bool) {
    #[cfg(test)]
    if context.fault.as_deref() == Some("rulings-changed-after-validation") {
        let path = &catalogue.runs.last().unwrap().path;
        if read {
            fs::write(path, "{not json\n".repeat(3)).unwrap();
        } else {
            fs::remove_file(path).unwrap();
        }
    }
    let _ = (context, catalogue, read);
}
fn words(query: &str, field: &str) -> Result<Vec<String>, String> {
    let t = terms(query);
    if t.is_empty() {
        Err(format!("Supply at least one word in {field}."))
    } else {
        Ok(t)
    }
}

type RankKey = (usize, bool, usize, usize);
struct Query<'a> {
    catalogue: &'a Catalogue,
    context: &'a Context,
    terms: &'a [String],
    words: &'a [String],
    family: &'a [String],
    offset: usize,
    limit: usize,
}
struct Hits {
    best: BinaryHeap<Ranked<RankKey>>,
    order: usize,
}
struct Window {
    before: VecDeque<(usize, Value)>,
    after: Vec<(usize, Value)>,
    found: Option<Value>,
}
type WordLists = (Vec<String>, Vec<String>, Vec<String>);

fn requested(input: &Value) -> Result<WordLists, String> {
    let terms = self::words(input["query"].as_str().unwrap(), "query")?;
    let words = words::query_words(&terms);
    let family = input["family"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| self::words(s, "family"))
        .transpose()?
        .unwrap_or_default();
    Ok((terms, words, family))
}
fn consider(
    hits: &mut Hits,
    query: &Query<'_>,
    row: &Value,
    run: &Run,
    line: usize,
) -> Result<(), String> {
    let key = canonical(row["docid"].as_str().unwrap_or(""));
    if query.catalogue.owner.get(&key) != Some(&run.name)
        || !corpus::holds(row, &["family"], query.family)
        || !words::holds_all(
            query.words,
            &corpus::joined(row, &["title", "heading", "text"]),
        )
    {
        return Ok(());
    }
    let value = paragraph(
        row,
        run,
        &key,
        line,
        query.catalogue,
        query.context.limit("SEARCH_TEXT_CHARS", 1200),
    )?;
    corpus::retain(
        &mut hits.best,
        Ranked {
            key: (
                corpus::tier(
                    query.words,
                    query.terms,
                    &["title", "heading"],
                    &["text"],
                    row,
                ),
                value["withdrawal_status"] == "NOTICE_PRESENT",
                family_priority(row),
                hits.order,
            ),
            value,
        },
        query.offset + query.limit + 1,
    );
    hits.order += 1;
    Ok(())
}
fn collect_matches(query: &Query<'_>) -> Result<Hits, String> {
    let mut hits = Hits {
        best: BinaryHeap::new(),
        order: 0,
    };
    let error = "Cannot read a UTF-8 rulings run in the configured folder.";
    for run in &query.catalogue.runs {
        for (line, raw) in Lines::open(&run.path).map_err(|_| error)?.enumerate() {
            let raw = raw.map_err(|_| error)?;
            let Ok(row) = parse_json(&raw) else {
                continue;
            };
            if !row.is_object() {
                continue;
            }
            consider(&mut hits, query, &row, run, line + 1)?;
        }
    }
    Ok(hits)
}
fn finish(
    best: BinaryHeap<Ranked<RankKey>>,
    offset: usize,
    limit: usize,
    catalogue: &Catalogue,
) -> Result<Value, String> {
    let best = best.into_sorted_vec();
    let more = best.len() > offset + limit;
    let matches = best
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|v| v.value)
        .collect::<Vec<_>>();
    let empty = matches.is_empty();
    let mut result = page(matches, offset + limit, more, policy("NOTICE"), 10000);
    result["corpus"] = provenance(catalogue);
    if empty && offset == 0 {
        result["notice"] = json!(format!("{} {}", policy("NOTICE"), policy("NO_MATCH_HINT")));
    }
    Ok(result)
}
pub(super) fn search(context: &Context, input: &Value) -> Result<Value, String> {
    let (terms, words, family) = requested(input)?;
    let catalogue = catalogue(context)?;
    after_catalogue(context, &catalogue, false);
    let (offset, limit) = (position(&input["offset"]), position(&input["limit"]));
    let hits = collect_matches(&Query {
        catalogue: &catalogue,
        context,
        terms: &terms,
        words: &words,
        family: &family,
        offset,
        limit,
    })?;
    finish(hits.best, offset, limit, &catalogue)
}
fn row_reference(reference: &str) -> Result<(String, String, usize), String> {
    if !ROW_REF.fullmatch(reference) {
        return Err("Use a row_ref returned by search_ato_rulings.".into());
    }
    let captures = ROW_REF.captures(reference).unwrap();
    Ok((
        captures.name("run").unwrap(),
        captures.name("docid").unwrap(),
        captures.name("line").unwrap().parse::<usize>().unwrap(),
    ))
}
fn serving_run<'a>(catalogue: &'a Catalogue, name: &str, key: &str) -> Result<&'a Run, String> {
    let run = catalogue
        .runs
        .iter()
        .find(|r| r.name == name)
        .ok_or("No rulings run with that name in the configured folder; search again.")?;
    if !run.documents.contains(key) {
        return Err("That rulings run holds no document with that docid; search again.".into());
    }
    if catalogue.withheld.contains_key(key) {
        return Err(
            "A later rulings run excluded that document, so no copy of it can be read.".into(),
        );
    }
    Ok(run)
}
fn place_row(
    window: &mut Window,
    row: Value,
    same: bool,
    line: usize,
    target: usize,
    near: usize,
) -> Result<bool, String> {
    if line < target {
        if same && near > 0 {
            window.before.push_back((line, row));
            if window.before.len() > near {
                window.before.pop_front();
            }
        }
        return Ok(false);
    }
    if line == target {
        if !same {
            return Err(
                "The row at that row_ref belongs to another document; search again.".into(),
            );
        }
        window.found = Some(row);
        return Ok(near == 0);
    }
    if same {
        window.after.push((line, row));
        if window.after.len() >= near {
            return Ok(true);
        }
    }
    Ok(false)
}
type Neighbours = (VecDeque<(usize, Value)>, Vec<(usize, Value)>, Value);

fn scan_document(run: &Run, key: &str, target: usize, near: usize) -> Result<Neighbours, String> {
    let mut window = Window {
        before: VecDeque::new(),
        after: Vec::new(),
        found: None,
    };
    let error = "Cannot read that rulings run as UTF-8 JSONL.";
    for (line, raw) in Lines::open(&run.path).map_err(|_| error)?.enumerate() {
        let raw = raw.map_err(|_| error)?;
        let line = line + 1;
        if line < target && near == 0 {
            continue;
        }
        let row = parse_json(&raw).map_err(|_| error)?;
        let same = canonical(row["docid"].as_str().ok_or(error)?) == key;
        if place_row(&mut window, row, same, line, target, near)? {
            break;
        }
    }
    let Window {
        before,
        after,
        found,
    } = window;
    let found = found
        .ok_or("No paragraph at that row_ref in the configured rulings runs; search again.")?;
    Ok((before, after, found))
}
fn slice_text(
    found: &mut Value,
    start: usize,
    context: &Context,
) -> Result<(usize, usize), String> {
    let whole = found["text"].as_str().unwrap_or("");
    let total = whole.chars().count();
    if start > 0 && start >= total {
        return Err(format!(
            "start is past the end of this paragraph, which has {total} characters."
        ));
    }
    let end = start
        .saturating_add(context.limit("READ_TEXT_CHARS", 12000))
        .min(total);
    found["text"] = json!(
        whole
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>()
    );
    Ok((end, total))
}
fn note_part(selected: &mut Value, start: usize, end: usize, total: usize) {
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
}
pub(super) fn read(context: &Context, input: &Value) -> Result<Value, String> {
    let (name, key, target) = row_reference(input["row_ref"].as_str().unwrap())?;
    let catalogue = catalogue(context)?;
    after_catalogue(context, &catalogue, true);
    let run = serving_run(&catalogue, &name, &key)?;
    let (near, start) = (position(&input["neighbours"]), position(&input["start"]));
    let (before, after, mut found) = scan_document(run, &key, target, near)?;
    let (end, total) = slice_text(&mut found, start, context)?;
    let mut selected = paragraph(
        &found,
        run,
        &key,
        target,
        &catalogue,
        context.limit("READ_TEXT_CHARS", 12000),
    )?;
    selected["total_chars"] = json!(total);
    note_part(&mut selected, start, end, total);
    let build = |(line, row): &(usize, Value)| {
        paragraph(
            row,
            run,
            &key,
            *line,
            &catalogue,
            context.limit("SEARCH_TEXT_CHARS", 1200),
        )
    };
    Ok(
        json!({"paragraph":selected,"start":start,"next_start":(end<total).then_some(end),"before":before.iter().map(build).collect::<Result<Vec<_>,_>>()?,"after":after.iter().map(build).collect::<Result<Vec<_>,_>>()?,"corpus":provenance(&catalogue),"notice":policy("NOTICE")}),
    )
}

pub(super) fn provenance(catalogue: &Catalogue) -> Value {
    let serving: HashSet<&str> = catalogue.owner.values().map(String::as_str).collect();
    let runs = catalogue
        .runs
        .iter()
        .filter(|run| serving.contains(run.name.as_str()))
        .collect::<Vec<_>>();
    let mut notices: Vec<Value> = Vec::new();
    for run in &runs {
        if let Some(notice) = notices
            .iter_mut()
            .find(|n| n["notice"] == run.notice && n["notice_url"] == run.url)
        {
            notice["runs"].as_array_mut().unwrap().push(json!(run.name));
        } else {
            notices.push(json!({"notice":run.notice,"notice_url":run.url,"runs":[run.name]}));
        }
    }
    json!({"runs_configured":catalogue.runs.len(),"runs_serving":runs.len(),"documents_serving":catalogue.owner.len(),"documents_withheld":catalogue.withheld.len(),"fetched_on_earliest":runs.iter().map(|r|&r.date).min(),"fetched_on_latest":runs.iter().map(|r|&r.date).max(),"reuse_notices":notices,"non_endorsement":policy("NON_ENDORSEMENT"),"ranking":policy("RANKING_NOTE")})
}
