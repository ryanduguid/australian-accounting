// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::{
    Context, POLICY,
    corpus::{self, Ranked},
    linked, page, splitlines, terms,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BinaryHeap,
    fs,
    path::{Component, Path, PathBuf},
};

const READ_ERROR: &str = "Cannot read that UTF-8 Markdown file in the configured library.";

fn path(root: &Path, name: &str) -> Result<PathBuf, String> {
    let relative = Path::new(name);
    if relative.is_absolute()
        || name.contains(':')
        || relative.components().any(|p| match p {
            Component::Normal(n) => n.to_string_lossy().starts_with('.'),
            Component::CurDir => false,
            _ => true,
        })
        || !relative
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("md"))
    {
        return Err("Use a relative .md path inside the configured library.".into());
    }
    // Python Path discards trailing separators and CurDir components before I/O.
    let candidate = root.join(relative.components().collect::<PathBuf>());
    for part in candidate.ancestors().take_while(|p| *p != root) {
        if linked(part).map_err(|_| READ_ERROR)? {
            return Err("Library links and junctions are not read.".into());
        }
    }
    if !fs::canonicalize(&candidate)
        .map_err(|_| READ_ERROR)?
        .starts_with(root)
    {
        return Err("The file must remain inside the configured library.".into());
    }
    Ok(candidate)
}

fn decode(raw: &[u8], context: &Context) -> Result<(Vec<String>, String), String> {
    if raw.len() > context.limit("MAX_FILE_BYTES", 8_000_000) {
        return Err("Markdown file exceeds the 8 MB retrieval limit.".into());
    }
    let text = std::str::from_utf8(raw).map_err(|_| READ_ERROR)?;
    Ok((
        splitlines(text.strip_prefix('\u{feff}').unwrap_or(text)),
        Sha256::digest(raw)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    ))
}

pub(super) fn excerpt(
    name: &str,
    lines: &[String],
    digest: &str,
    start: usize,
    count: usize,
) -> Result<Value, String> {
    if start < 1 {
        return Err("start_line must be at least 1.".into());
    }
    if start > lines.len() {
        return Err("start_line is beyond the end of the document.".into());
    }
    let (mut heading, mut pdf_page) = (None, Value::Null);
    for line in &lines[..start] {
        advance(line, &mut heading, &mut pdf_page)?;
    }
    shape(name, digest, lines, start, count, heading, pdf_page)
}

/// Moves the heading and PDF page in force before `line` to those in force after it.
/// The heading is kept whole so ranking sees every word; `shape` truncates it.
fn advance<'a>(
    line: &'a str,
    heading: &mut Option<&'a str>,
    pdf_page: &mut Value,
) -> Result<(), String> {
    if line.starts_with('#') {
        *heading = Some(line);
    }
    for (index, _) in line.match_indices("<!-- PDF page ") {
        let after = &line[index + "<!-- PDF page ".len()..];
        let digits: String = after
            .chars()
            .map_while(payday_super_checker::py::decimal_value)
            .map(|d| char::from(b'0' + d as u8))
            .collect();
        if !digits.is_empty() {
            let number = digits.trim_start_matches('0');
            *pdf_page = serde_json::from_str(if number.is_empty() { "0" } else { number })
                .map_err(|_| READ_ERROR)?;
            break;
        }
    }
    Ok(())
}

fn shape(
    name: &str,
    digest: &str,
    lines: &[String],
    start: usize,
    count: usize,
    heading: Option<&str>,
    pdf_page: Value,
) -> Result<Value, String> {
    // Truncate only at a line boundary so citation ranges remain exact.
    let mut end = (start - 1).saturating_add(count).min(lines.len());
    let mut length = lines[start - 1..end]
        .iter()
        .map(|s| s.chars().count())
        .sum::<usize>()
        + (end - start);
    while length > 12000 && end > start {
        end -= 1;
        length -= 1 + lines[end].chars().count();
    }
    if length > 12000 {
        return Err("The selected line exceeds 12000 characters; select another line.".into());
    }
    let heading = heading.map(|h| h.chars().take(300).collect::<String>());
    Ok(
        json!({"path":name,"sha256":digest,"start_line":start,"end_line":end,"total_lines":lines.len(),"heading":heading,"pdf_page":pdf_page,"text":lines[start - 1..end].join("\n"),"notice":POLICY["library"]["NOTICE"]}),
    )
}

pub(super) fn read(
    context: &Context,
    name: &str,
    start: usize,
    count: usize,
) -> Result<Value, String> {
    let root = context.root("AUS_ACCOUNTING_LIBRARY_ROOT", "Markdown folder")?;
    let raw = context
        .read(
            &path(&root, name)?,
            context.limit("MAX_FILE_BYTES", 8_000_000),
        )
        .map_err(|_| READ_ERROR)?;
    let (lines, digest) = decode(&raw, context)?;
    excerpt(name, &lines, &digest, start, count)
}

pub(super) fn search(
    context: &Context,
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<Value, String> {
    let terms = terms(query);
    if terms.is_empty() {
        return Err("Supply at least one word to search for.".into());
    }
    let root = context.root("AUS_ACCOUNTING_LIBRARY_ROOT", "Markdown folder")?;
    // shortcut: ranking scans the whole library on every search, as the Python server
    // does; add an index if a measured library query set needs one.
    let cap = offset + limit + 1;
    let mut best: BinaryHeap<Ranked<(usize, usize)>> = BinaryHeap::new();
    let (mut skipped, mut size, mut count, mut order) = (0, 0, 0, 0);
    let mut pending = vec![root.clone()];
    let finish = |matches, seen, skipped, more| {
        let mut value = page(
            matches,
            seen,
            more,
            POLICY["library"]["NOTICE"].as_str().unwrap(),
            context.limit("MAX_OFFSET", 10000),
        );
        value["skipped_files"] = json!(skipped);
        value
    };
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        let mut folders = Vec::new();
        let mut files = Vec::new();
        for entry in entries.flatten() {
            let filename = entry.file_name();
            if filename.as_encoded_bytes().first() == Some(&b'.') {
                continue;
            }
            let path = entry.path();
            if !path.is_dir()
                && !path
                    .extension()
                    .is_some_and(|s| s.eq_ignore_ascii_case("md"))
            {
                continue;
            }
            let name = filename
                .into_string()
                .map_err(|_| "Library paths must use Unicode names.")?;
            if path.is_dir() {
                if !linked(&path).unwrap_or(true) {
                    folders.push((name, path));
                }
            } else {
                files.push((name, path));
            }
        }
        folders.sort_by(|a, b| a.0.cmp(&b.0));
        pending.extend(folders.into_iter().rev().map(|(_, path)| path));
        files.sort_by(|a, b| a.0.cmp(&b.0));
        for (filename, candidate) in files {
            if !filename.to_lowercase().ends_with(".md") {
                continue;
            }
            count += 1;
            if count > context.limit("MAX_FILES", 1000) {
                return Err(
                    "Library exceeds 1000 Markdown files; configure a smaller folder.".into(),
                );
            }
            let name = candidate
                .strip_prefix(&root)
                .unwrap()
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let raw = path(&root, &name).and_then(|p| {
                context
                    .read(
                        &p,
                        context
                            .limit("MAX_FILE_BYTES", 8_000_000)
                            .min(context.limit("MAX_LIBRARY_BYTES", 64_000_000) - size),
                    )
                    .map_err(|_| READ_ERROR.into())
            });
            let Ok(raw) = raw else {
                skipped += 1;
                continue;
            };
            size += raw.len();
            if size > context.limit("MAX_LIBRARY_BYTES", 64_000_000) {
                return Err("Library exceeds 64 MB; configure a smaller folder.".into());
            }
            let Ok((lines, digest)) = decode(&raw, context) else {
                skipped += 1;
                continue;
            };
            let (mut heading, mut pdf_page) = (None, Value::Null);
            for (index, line) in lines.iter().enumerate() {
                advance(line, &mut heading, &mut pdf_page)?;
                let folded = payday_super_checker::py::casefold(line);
                if !terms.iter().all(|term| folded.contains(term)) {
                    continue;
                }
                let row = json!({"heading":heading,"line":line});
                let key = (corpus::tier(&terms, &["heading"], &["line"], &row), order);
                order += 1;
                if best.len() < cap || best.peek().is_some_and(|worst| key < worst.key) {
                    // A line too long to excerpt still takes its rank, as null, so the
                    // next offset lands after the last one this page consumed.
                    let value = shape(
                        &name,
                        &digest,
                        &lines,
                        index + 1,
                        3,
                        heading,
                        pdf_page.clone(),
                    )
                    .unwrap_or(Value::Null);
                    corpus::retain(&mut best, Ranked { key, value }, cap);
                }
            }
        }
    }
    let best = best.into_sorted_vec();
    let more = best.len() > offset + limit;
    let matches = best
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|ranked| ranked.value)
        .filter(|value| !value.is_null())
        .collect();
    Ok(finish(matches, offset + limit, skipped, more))
}
