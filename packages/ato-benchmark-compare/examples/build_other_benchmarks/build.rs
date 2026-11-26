// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Checks each parsed page against the workbook dataset and assembles the
//! other-benchmarks file, as `build_other_benchmarks.py`'s `build` did.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use ato_benchmark_compare::dataset::{canonical_digest, normalise};
use ato_benchmark_compare::py::{is_space, repr_str};
use serde_json::{Map, Value, json};

use crate::page::{ParsedPage, parse_page};
use crate::shared::sha256_hex;
use crate::text::{Error, decimal_eq, label_key, py_repr_list, py_repr_obj, py_repr_pair, slug};

const OTHER_KEYS: [&str; 4] = [
    "cost_of_sales_to_turnover",
    "labour_to_turnover",
    "rent_to_turnover",
    "motor_vehicle_to_turnover",
];
const TITLE_SUFFIX: &str = " | Australian Taxation Office";
const MAX_EXCLUDED: usize = 5;
const ADDRESS_ALIASES: [(&str, &str); 2] = [
    (
        "motor-vehicle-retail-new-and-used",
        "Motor vehicle retail - new and used car",
    ),
    (
        "roofing-services-includes-roof-tiling-guttering-and-metal-roofing",
        "Roofing services, including roof tiling, guttering and metal roofing",
    ),
];

fn check_section_agreements(
    page: &ParsedPage,
    bands: &[Value],
    year: &str,
    where_: &str,
) -> Result<Vec<String>, Error> {
    let mut anomalies = Vec::new();
    let expected: Vec<String> = bands
        .iter()
        .map(|b| label_key(b["label"].as_str().unwrap_or_default()))
        .collect();
    for (kind, section) in &page.sections {
        if section.year != year {
            if kind != "key" {
                return Err(Error::Mismatch(format!(
                    "{where_}: {kind} benchmarks are for {}, dataset is {year}",
                    section.year
                )));
            }
            anomalies.push(format!(
                "key table heading says {}; the page and its ranges are {year}",
                section.year
            ));
        }
        let printed: Vec<String> = section.labels.iter().map(|l| label_key(l)).collect();
        if printed != expected {
            let expected_labels: Vec<String> = bands
                .iter()
                .map(|b| b["label"].as_str().unwrap_or_default().to_string())
                .collect();
            return Err(Error::Mismatch(format!(
                "{where_}: {kind} bands {} do not match the dataset's {}",
                py_repr_list(&section.labels),
                py_repr_list(&expected_labels)
            )));
        }
    }
    Ok(anomalies)
}

fn bands_of(business_type: &Value) -> Result<&Vec<Value>, Error> {
    business_type["turnover_bands"]
        .as_array()
        .ok_or_else(|| Error::Build("missing turnover_bands".into()))
}

/// Every range the dataset publishes for a band must be printed on the page.
fn check_printed_keys(page: &ParsedPage, bands: &[Value], where_: &str) -> Result<(), Error> {
    let printed: BTreeSet<&str> = page
        .sections
        .values()
        .flat_map(|section| section.ranges.keys().map(String::as_str))
        .collect();
    for band in bands {
        for key in ["cost_of_sales_to_turnover", "total_expenses_to_turnover"] {
            if !band[key].is_null() && !printed.contains(key) {
                return Err(Error::Mismatch(format!(
                    "{where_}: the page does not print the dataset's {key} range"
                )));
            }
        }
    }
    Ok(())
}

/// Every range the page prints and the dataset publishes must agree.
fn check_range_values(page: &ParsedPage, bands: &[Value], where_: &str) -> Result<(), Error> {
    for (key, values) in page.sections.values().flat_map(|s| s.ranges.iter()) {
        for (band, value) in bands.iter().zip(values.iter()) {
            let published = &band[key.as_str()];
            let (Some((low, high)), false) = (value, published.is_null()) else {
                continue;
            };
            let pub_min = published["min"].as_str().unwrap_or_default();
            let pub_max = published["max"].as_str().unwrap_or_default();
            if !decimal_eq(pub_min, low)? || !decimal_eq(pub_max, high)? {
                return Err(Error::Mismatch(format!(
                    "{where_}: {key} for {} is {} on the page but {} in the dataset",
                    band["band"].as_str().unwrap_or_default(),
                    py_repr_pair(low, high),
                    py_repr_obj(published)
                )));
            }
        }
    }
    Ok(())
}

fn check_dataset_ranges(
    page: &ParsedPage,
    business_type: &Value,
    where_: &str,
) -> Result<(), Error> {
    let bands = bands_of(business_type)?;
    check_printed_keys(page, bands, where_)?;
    check_range_values(page, bands, where_)?;
    let key_ratio = business_type["key_ratio"].as_str().unwrap_or_default();
    if !page.sections["key"].ranges.contains_key(key_ratio) {
        return Err(Error::Mismatch(format!(
            "{where_}: the key table does not carry {key_ratio}"
        )));
    }
    Ok(())
}

/// Each band's other ranges, kept only where the workbook dataset has none.
fn other_bands(page: &ParsedPage, bands: &[Value]) -> Vec<Value> {
    let empty_other = BTreeMap::new();
    let other = page
        .sections
        .get("other")
        .map_or(&empty_other, |s| &s.ranges);
    let mut out_bands = Vec::with_capacity(bands.len());
    for (position, band) in bands.iter().enumerate() {
        let mut entry = Map::new();
        entry.insert("band".into(), band["band"].clone());
        for key in OTHER_KEYS {
            let cell = other
                .get(key)
                .and_then(|vals| vals.get(position))
                .and_then(Option::as_ref);
            let value = match cell {
                Some((low, high)) if band[key].is_null() => json!({"min": low, "max": high}),
                _ => Value::Null,
            };
            entry.insert(key.into(), value);
        }
        out_bands.push(Value::Object(entry));
    }
    out_bands
}

fn page_entry(
    item: &Value,
    business_type: &Value,
    page_name: &str,
    raw: &[u8],
    year: &str,
) -> Result<Value, Error> {
    let where_ = item_where(item)?;
    let text = std::str::from_utf8(raw)
        .map_err(|e| Error::Build(format!("{where_}: invalid UTF-8: {e}")))?;
    let page = parse_page(text, where_)?;
    if page.financial_year != year {
        return Err(Error::Mismatch(format!(
            "{where_}: the page covers {}, dataset is {year}",
            page.financial_year
        )));
    }
    let bands = bands_of(business_type)?;
    let anomalies = check_section_agreements(&page, bands, year, where_)?;
    check_dataset_ranges(&page, business_type, where_)?;
    Ok(json!({
        "name": business_type["name"],
        "page_title": page_name,
        "page_url": where_,
        "page_last_updated": page.last_updated,
        "page_reference": page.qc,
        "text_sha256": sha256_hex(raw),
        "page_anomalies": anomalies,
        "turnover_bands": other_bands(&page, bands),
    }))
}

fn json_truthy(val: &Value) -> bool {
    match val {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn item_where(item: &Value) -> Result<&str, Error> {
    item["final_url"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| item["url"].as_str())
        .ok_or_else(|| Error::Build("index item missing url".into()))
}

/// The page's name from its saved title, once the save is known to be whole.
fn saved_page_name(item: &Value, where_: &str) -> Result<String, Error> {
    let status = &item["status"];
    if status.as_u64() != Some(200)
        || json_truthy(&item["error"])
        || json_truthy(&item["truncated"])
    {
        let shown = if status.is_null() {
            "None".to_string()
        } else {
            status.to_string()
        };
        return Err(Error::Build(format!(
            "{where_}: page was not saved whole (status {shown})"
        )));
    }
    let title = item["title"].as_str().unwrap_or("");
    let Some(stripped) = title.strip_suffix(TITLE_SUFFIX) else {
        return Err(Error::Build(format!(
            "{where_}: unexpected page title {}",
            repr_str(title)
        )));
    };
    Ok(stripped.trim_matches(is_space).to_string())
}

/// Business types by folded name and by page address.
struct Lookup {
    by_name: HashMap<String, usize>,
    by_slug: HashMap<String, usize>,
}

impl Lookup {
    fn new(business_types: &[Value]) -> Result<Self, Error> {
        let (mut by_name, mut by_slug) = (HashMap::new(), HashMap::new());
        for (idx, bt) in business_types.iter().enumerate() {
            let name = bt["name"].as_str().unwrap_or_default();
            by_name.insert(normalise(name), idx);
            by_slug.insert(slug(name), idx);
        }
        if by_slug.len() != business_types.len() {
            return Err(Error::Build(
                "two business types share a page address".into(),
            ));
        }
        Ok(Self { by_name, by_slug })
    }

    /// The business type a saved page is for, by its address and its title.
    fn resolve(&self, item: &Value, year: &str) -> Result<(usize, String), Error> {
        let where_ = item_where(item)?;
        let page_name = saved_page_name(item, where_)?;
        let address = where_
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("");
        let alias_norm = ADDRESS_ALIASES
            .iter()
            .find(|(k, _)| *k == address)
            .map(|(_, v)| normalise(v))
            .unwrap_or_default();
        let from_address = self
            .by_slug
            .get(address)
            .or_else(|| self.by_name.get(&alias_norm))
            .copied();
        let from_title = self.by_name.get(&normalise(&page_name)).copied();
        if let (Some(a), Some(t)) = (from_address, from_title)
            && a != t
        {
            return Err(Error::Build(format!(
                "{where_}: address and title name different business types"
            )));
        }
        let idx = from_address.or(from_title).ok_or_else(|| {
            Error::Build(format!(
                "{where_}: {} is not a business type in {year}",
                repr_str(&page_name)
            ))
        })?;
        Ok((idx, page_name))
    }
}

fn validate_exclusions(
    excluded: &[Value],
    expect_excluded: &BTreeSet<String>,
) -> Result<(), Error> {
    let reason = |item: &Value| item["reason"].as_str().unwrap_or_default().to_string();
    let actual: BTreeSet<String> = excluded
        .iter()
        .map(|item| item["name"].as_str().unwrap_or_default().to_string())
        .collect();
    if &actual != expect_excluded {
        let reasons: Vec<String> = excluded
            .iter()
            .filter(|item| !expect_excluded.contains(item["name"].as_str().unwrap_or_default()))
            .map(reason)
            .collect();
        let actual_vec: Vec<String> = actual.into_iter().collect();
        let expect_vec: Vec<String> = expect_excluded.iter().cloned().collect();
        return Err(Error::Build(format!(
            "excluded pages {} differ from --expect-excluded {}. {}",
            py_repr_list(&actual_vec),
            py_repr_list(&expect_vec),
            reasons.join("; ")
        )));
    }
    if excluded.len() > MAX_EXCLUDED {
        let reasons: Vec<String> = excluded.iter().take(3).map(reason).collect();
        return Err(Error::Build(format!(
            "{} pages disagree with the dataset: {}",
            excluded.len(),
            reasons.join("; ")
        )));
    }
    Ok(())
}

fn read_text(path: &Path) -> Result<String, Error> {
    fs::read_to_string(path).map_err(|e| path_error(path, &e))
}

fn path_error(path: &Path, e: &dyn std::fmt::Display) -> Error {
    Error::Build(format!("{}: {e}", path.display()))
}

/// The included entries, the excluded pages and the names that had a page.
#[derive(Default)]
struct Pages {
    entries: Vec<Value>,
    excluded: Vec<Value>,
    seen: BTreeSet<String>,
}

impl Pages {
    fn add(
        &mut self,
        item: &Value,
        business_types: &[Value],
        lookup: &Lookup,
        year: &str,
        pages_dir: &Path,
    ) -> Result<(), Error> {
        let (idx, page_name) = lookup.resolve(item, year)?;
        let bt = &business_types[idx];
        let name = bt["name"].as_str().unwrap_or_default().to_string();
        let where_ = item_where(item)?;
        if !self.seen.insert(name.clone()) {
            return Err(Error::Build(format!(
                "{where_}: {} has a second page",
                repr_str(&name)
            )));
        }
        let file_rel = item["file"]
            .as_str()
            .ok_or_else(|| Error::Build(format!("{where_}: index item missing file")))?;
        let path = pages_dir.join(file_rel);
        let raw = fs::read(&path).map_err(|e| path_error(&path, &e))?;
        match page_entry(item, bt, &page_name, &raw, year) {
            Ok(entry) => self.entries.push(entry),
            Err(Error::Mismatch(reason)) => self.excluded.push(json!({
                "name": name,
                "page_url": where_,
                "reason": reason,
            })),
            Err(err @ Error::Build(_)) => return Err(err),
        }
        Ok(())
    }

    fn check_complete(&self, business_types: &[Value]) -> Result<(), Error> {
        let mut missing: Vec<String> = business_types
            .iter()
            .filter_map(|bt| bt["name"].as_str())
            .filter(|name| !self.seen.contains(*name))
            .map(str::to_string)
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        missing.sort();
        let preview: Vec<String> = missing.iter().take(5).cloned().collect();
        Err(Error::Build(format!(
            "no page for {} business types: {}",
            missing.len(),
            py_repr_list(&preview)
        )))
    }
}

fn source(retrieved: &str, dataset: &Value, json_digest: String) -> Value {
    json!({
        "publisher": "Australian Taxation Office",
        "pages": "Small business benchmarks, industry pages (in detail)",
        "index_url": "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/small-business-benchmarks/benchmarks-a-z",
        "retrieved": retrieved,
        "dataset_sha256": dataset["source"]["sha256"],
        "dataset_json_sha256": json_digest,
        "licence": "ATO copyright notice: free to copy, adapt, modify, transmit and distribute, but not in any way that suggests the ATO or the Commonwealth endorses the user or its products",
        "licence_url": "https://www.ato.gov.au/about-ato/using-our-website/copyright-notice",
        "caution": "Not all expenses are reported by every business. The ATO says to use these ranges only as a guide where they apply to the business.",
    })
}

pub fn build(
    pages_dir: &Path,
    dataset_path: &Path,
    retrieved: &str,
    expect_excluded: &BTreeSet<String>,
) -> Result<Value, Error> {
    let dataset_text = read_text(dataset_path)?;
    let dataset: Value =
        serde_json::from_str(&dataset_text).map_err(|e| path_error(dataset_path, &e))?;
    let year = dataset["benchmark_year"]
        .as_str()
        .ok_or_else(|| Error::Build("dataset missing benchmark_year".into()))?;
    let business_types = dataset["business_types"]
        .as_array()
        .ok_or_else(|| Error::Build("dataset missing business_types".into()))?;
    let lookup = Lookup::new(business_types)?;
    let index_path = pages_dir.join("index.json");
    let index: Vec<Value> =
        serde_json::from_str(&read_text(&index_path)?).map_err(|e| path_error(&index_path, &e))?;
    let mut pages = Pages::default();
    for item in &index {
        pages.add(item, business_types, &lookup, year, pages_dir)?;
    }
    pages.check_complete(business_types)?;
    validate_exclusions(&pages.excluded, expect_excluded)?;
    let by_name = |a: &Value, b: &Value| a["name"].as_str().cmp(&b["name"].as_str());
    pages.entries.sort_by(by_name);
    pages.excluded.sort_by(by_name);
    let json_digest =
        canonical_digest(&dataset_text).map_err(|e| Error::Build(format!("{e:?}")))?;
    Ok(json!({
        "schema_version": 1,
        "benchmark_year": year,
        "business_type_count": pages.entries.len(),
        "source": source(retrieved, &dataset, json_digest),
        "business_types": pages.entries,
        "excluded_pages": pages.excluded,
    }))
}

pub fn count_filled_ranges(built: &Value) -> usize {
    built["business_types"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|e| e["turnover_bands"].as_array().into_iter().flatten())
        .flat_map(|band| OTHER_KEYS.iter().map(move |k| &band[*k]))
        .filter(|v| !v.is_null())
        .count()
}
