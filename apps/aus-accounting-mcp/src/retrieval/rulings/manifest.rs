// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::{Context, Lines, parse_json};
use super::{DOCID, ISO_DATE, POLICY, SHA256, canonical, count, family_priority, withdrawal};
use jiff::civil::Date;
use num_bigint::BigInt;
use payday_super_checker::py;
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::Path,
};

// Validate duplicate members recursively before decoding to Value. Reuse serde's
// JSON parser and arbitrary-precision support; no second JSON grammar lives here.
struct Unique;
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Check;
        impl<'de> Visitor<'de> for Check {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON without duplicate object keys")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Unique, M::Error> {
                let mut keys = HashSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key) {
                        return Err(de::Error::custom("duplicate key"));
                    }
                    map.next_value::<Unique>()?;
                }
                Ok(Unique)
            }
            fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Unique, S::Error> {
                while seq.next_element::<Unique>()?.is_some() {}
                Ok(Unique)
            }
            fn visit_bool<E: de::Error>(self, _: bool) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_i64<E: de::Error>(self, _: i64) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_u64<E: de::Error>(self, _: u64) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_str<E: de::Error>(self, _: &str) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
                Ok(Unique)
            }
        }
        deserializer.deserialize_any(Check)
    }
}
pub(super) struct Manifest {
    pub(super) date: String,
    pub(super) listed: serde_json::Map<String, Value>,
    pub(super) excluded: HashSet<String>,
    pub(super) notice: String,
    pub(super) url: String,
}

fn invalid_manifest(run: &str, reason: &str) -> String {
    format!("Rulings run {run} has an invalid manifest.json: {reason}.")
}
fn read_manifest(context: &Context, path: &Path, run: &str) -> Result<Value, String> {
    let readable = || invalid_manifest(run, "it is not readable UTF-8 JSON without duplicate keys");
    let raw = context
        .read(path, context.limit("MAX_MANIFEST_BYTES", 4_000_000))
        .map_err(|_| readable())?;
    let text = std::str::from_utf8(&raw).map_err(|_| readable())?;
    let data = parse_json(text).map_err(|_| readable())?;
    match serde_json::from_str::<Unique>(text) {
        Ok(_) => {}
        Err(error) if error.to_string().contains("recursion limit exceeded") => {
            stacker::grow(16 * 1024 * 1024, || {
                let mut decoder = serde_json::Deserializer::from_str(text);
                decoder.disable_recursion_limit();
                Unique::deserialize(&mut decoder)
            })
            .map_err(|_| readable())?;
        }
        Err(_) => return Err(readable()),
    }
    Ok(data)
}
fn manifest_preamble<'a>(
    data: &'a Value,
    run: &str,
) -> Result<(&'a str, &'a str, &'a str), String> {
    if !data.is_object() || data["stage"] != "rulings" {
        return Err(invalid_manifest(run, "stage must be 'rulings'"));
    }
    let date = data["fetched_on"]
        .as_str()
        .filter(|s| {
            ISO_DATE.fullmatch(s)
                && !s.starts_with("0000-")
                && Date::strptime("%Y-%m-%d", s).is_ok()
        })
        .ok_or_else(|| invalid_manifest(run, "fetched_on must be a YYYY-MM-DD date"))?;
    let notice = data["reuse_notice"]
        .as_str()
        .filter(|s| !s.trim_matches(py::is_space).is_empty());
    let url = data["reuse_notice_url"]
        .as_str()
        .filter(|s| s.starts_with("https://"));
    let (Some(notice), Some(url)) = (notice, url) else {
        return Err(invalid_manifest(
            run,
            "the reuse notice and its https address are required",
        ));
    };
    Ok((date, notice, url))
}
fn manifest_lists<'a>(
    context: &Context,
    data: &'a Value,
    run: &str,
) -> Result<(&'a Vec<Value>, &'a Vec<Value>, BigInt), String> {
    let maximum = context.limit("MAX_DOCUMENTS", 1000);
    let documents = data["documents"]
        .as_array()
        .filter(|a| a.len() <= maximum)
        .ok_or_else(|| {
            invalid_manifest(
                run,
                &format!("documents must be a list of at most {maximum}"),
            )
        })?;
    let excluded = data["excluded"]
        .as_array()
        .ok_or_else(|| invalid_manifest(run, "excluded must be a list"))?;
    let rows = count(&data["rows"])
        .filter(|n| *n >= BigInt::from(0))
        .ok_or_else(|| invalid_manifest(run, "rows must be a non-negative integer"))?;
    Ok((documents, excluded, rows))
}
fn accept_docid(
    document: &Value,
    index: usize,
    run: &str,
    listed: &serde_json::Map<String, Value>,
) -> Result<String, String> {
    let id = document["docid"]
        .as_str()
        .map(canonical)
        .filter(|id| DOCID.fullmatch(id))
        .ok_or_else(|| {
            invalid_manifest(run, &format!("document {index} has no supported docid"))
        })?;
    if listed.contains_key(&id) {
        return Err(invalid_manifest(
            run,
            &format!("document {index} repeats an earlier docid"),
        ));
    }
    if count(&document["paragraphs"]).is_none_or(|n| n < BigInt::from(1)) {
        return Err(invalid_manifest(
            run,
            &format!("document {index} must count at least one paragraph"),
        ));
    }
    if family_priority(document) == POLICY["rulings"]["FAMILIES"].as_array().unwrap().len() {
        return Err(invalid_manifest(
            run,
            &format!("document {index} names an unsupported family"),
        ));
    }
    Ok(id)
}
fn accept_document_fields(
    document: &Value,
    index: usize,
    date: &str,
    run: &str,
) -> Result<(), String> {
    if document["fetched_on"] != date {
        return Err(invalid_manifest(
            run,
            &format!("document {index} carries a different date from its run"),
        ));
    }
    if !document["sha256"]
        .as_str()
        .is_some_and(|s| SHA256.fullmatch(s))
    {
        return Err(invalid_manifest(
            run,
            &format!("document {index} needs the SHA-256 of its page"),
        ));
    }
    if !matches!(
        document["licence_basis"].as_str(),
        Some("document-notice" | "site-notice-via-dc-rights")
    ) {
        return Err(invalid_manifest(
            run,
            &format!("document {index} records no supported reuse basis"),
        ));
    }
    if !["title", "source_url"]
        .iter()
        .all(|f| document[*f].is_string())
    {
        return Err(invalid_manifest(
            run,
            &format!("document {index} needs a title and a source_url"),
        ));
    }
    withdrawal(document)?;
    Ok(())
}
fn listed_documents(
    documents: &[Value],
    date: &str,
    run: &str,
) -> Result<serde_json::Map<String, Value>, String> {
    let mut listed = serde_json::Map::new();
    for (index, document) in documents.iter().enumerate() {
        let index = index + 1;
        let id = accept_docid(document, index, run, &listed)?;
        accept_document_fields(document, index, date, run)?;
        listed.insert(id, document.clone());
    }
    Ok(listed)
}
fn withheld_ids(
    excluded: &[Value],
    listed: &serde_json::Map<String, Value>,
    run: &str,
) -> Result<HashSet<String>, String> {
    let mut withheld = HashSet::new();
    for (index, entry) in excluded.iter().enumerate() {
        let id = entry["docid"]
            .as_str()
            .filter(|s| !s.trim_matches(py::is_space).is_empty())
            .ok_or_else(|| {
                invalid_manifest(run, &format!("excluded entry {} needs a docid", index + 1))
            })?;
        let id = canonical(id);
        if listed.contains_key(&id) {
            return Err(invalid_manifest(
                run,
                &format!(
                    "excluded entry {} names a document the run also accepted",
                    index + 1
                ),
            ));
        }
        withheld.insert(id);
    }
    Ok(withheld)
}
fn rows_match(
    rows: &BigInt,
    listed: &serde_json::Map<String, Value>,
    run: &str,
) -> Result<(), String> {
    if *rows
        != listed
            .values()
            .map(|d| count(&d["paragraphs"]).unwrap())
            .sum::<BigInt>()
    {
        return Err(invalid_manifest(
            run,
            "rows does not equal the sum of the documents' paragraph counts",
        ));
    }
    Ok(())
}
pub(super) fn manifest(context: &Context, path: &Path, run: &str) -> Result<Manifest, String> {
    let data = read_manifest(context, path, run)?;
    let (date, notice, url) = manifest_preamble(&data, run)?;
    let (documents, excluded, rows) = manifest_lists(context, &data, run)?;
    let listed = listed_documents(documents, date, run)?;
    let excluded = withheld_ids(excluded, &listed, run)?;
    rows_match(&rows, &listed, run)?;
    Ok(Manifest {
        date: date.into(),
        listed,
        excluded,
        notice: notice.into(),
        url: url.into(),
    })
}
fn counted_rows(
    name: &str,
    manifest: &Manifest,
    counts: &HashMap<String, usize>,
) -> Result<(), String> {
    for (key, document) in &manifest.listed {
        if BigInt::from(counts[key]) != count(&document["paragraphs"]).unwrap() {
            return Err(format!(
                "Rulings run {name} holds {} rows for a document its manifest counts as {}.",
                counts[key], document["paragraphs"]
            ));
        }
    }
    Ok(())
}

pub(super) fn validate(path: &Path, name: &str, manifest: &Manifest) -> Result<(), String> {
    let error = format!("Cannot read rulings run {name} as UTF-8 JSONL.");
    let mut counts: HashMap<_, usize> = manifest.listed.keys().map(|k| (k.clone(), 0)).collect();
    for (line, raw) in Lines::open(path).map_err(|_| &error)?.enumerate() {
        let line = line + 1;
        let raw = raw.map_err(|_| &error)?;
        let row = parse_json(&raw)
            .map_err(|_| format!("Rulings run {name} has a malformed row at line {line}."))?;
        if !row.is_object()
            || !POLICY["rulings"]["ROW_FIELDS"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| row[f.as_str().unwrap()].is_string())
        {
            return Err(format!(
                "Rulings run {name} row {line} lacks a required field."
            ));
        }
        let key = canonical(row["docid"].as_str().unwrap());
        let document = manifest.listed.get(&key).ok_or_else(|| {
            format!(
                "Rulings run {name} row {line} belongs to a document its manifest does not list."
            )
        })?;
        let disagree = [
            ("family", "family"),
            ("title", "title"),
            ("source_url", "source_url"),
            ("source_sha256", "sha256"),
            ("licence_basis", "licence_basis"),
        ]
        .iter()
        .any(|(r, d)| row[*r] != document[*d]);
        if disagree
            || row["fetched_on"] != manifest.date
            || !matches!(row["numbering"].as_str(), Some("document" | "sequential"))
            || withdrawal(&row)? != withdrawal(document)?
        {
            return Err(format!(
                "Rulings run {name} row {line} disagrees with its manifest record."
            ));
        }
        *counts.get_mut(&key).unwrap() += 1;
    }
    counted_rows(name, manifest, &counts)
}

#[cfg(test)]
mod tests {
    use super::Unique;
    use serde::Deserialize;

    /// With serde_json's `arbitrary_precision` (this crate's build) a float reaches the
    /// check as a number token inside a map, so it needs no float arm.
    #[test]
    fn floats_pass_and_duplicate_keys_are_refused() {
        assert!(serde_json::from_str::<Unique>(r#"[1.5, {"a": 2.5e3, "b": -0.0}]"#).is_ok());
        let error = serde_json::from_str::<Unique>(r#"{"a": 1, "a": 2}"#)
            .err()
            .unwrap();
        assert!(error.to_string().starts_with("duplicate key"), "{error}");
    }

    #[test]
    fn a_value_json_cannot_hold_is_described_as_the_check_expects() {
        let bytes = serde::de::value::BytesDeserializer::<serde::de::value::Error>::new(b"x");
        let error = Unique::deserialize(bytes).err().unwrap();
        assert_eq!(
            error.to_string(),
            "invalid type: byte array, expected JSON without duplicate object keys"
        );
    }
}
