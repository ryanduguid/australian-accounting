//! The shipped ATO benchmark datasets and turnover band selection. Every ratio
//! is a decimal string, so no binary floating point enters the comparison.

use std::sync::LazyLock;

use sha2::{Digest, Sha256};

use crate::Failure;
use crate::decimal::{Calc, Decimal};
use crate::py::{self, PyPath};
use crate::pyjson::{self, Json};
use crate::pyre::Regex;
use crate::pytext;
use crate::raw::{Raw, float_repr};
use crate::unicode;

pub const SUPPORTED_SCHEMA_VERSION: i64 = 1;
pub const RATIO_KEYS: [&str; 2] = ["cost_of_sales_to_turnover", "total_expenses_to_turnover"];
pub const PAGE_RATIO_KEYS: [&str; 4] = [
    "cost_of_sales_to_turnover",
    "labour_to_turnover",
    "rent_to_turnover",
    "motor_vehicle_to_turnover",
];

/// The datasets compiled into the binary, by file name, as `data/` holds them.
pub const SHIPPED: [(&str, &str); 3] = [
    (
        "benchmarks-2022-23.json",
        include_str!("../data/benchmarks-2022-23.json"),
    ),
    (
        "benchmarks-2023-24.json",
        include_str!("../data/benchmarks-2023-24.json"),
    ),
    (
        "other-benchmarks-2023-24.json",
        include_str!("../data/other-benchmarks-2023-24.json"),
    ),
];

pub fn ratio_label(key: &str) -> &str {
    match key {
        "cost_of_sales_to_turnover" => "Cost of sales to turnover",
        "total_expenses_to_turnover" => "Total expenses to turnover",
        "labour_to_turnover" => "Labour to turnover",
        "rent_to_turnover" => "Rent to turnover",
        "motor_vehicle_to_turnover" => "Motor vehicle expenses to turnover",
        other => other,
    }
}

static FILENAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^benchmarks-(\d{4}-\d{2})\.json$"));
static PUNCTUATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("[-\u{2010}-\u{2015}\u{2018}\u{2019}\u{201c}\u{201d}'\"]+"));
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+"));

fn fail<T>(message: impl Into<String>) -> Result<T, Failure> {
    Err(Failure::Value(message.into()))
}

#[derive(Clone, Debug)]
pub struct Range {
    pub minimum: Decimal,
    pub maximum: Decimal,
}

impl Range {
    pub fn contains(&self, value: &Decimal) -> Calc<bool> {
        Ok(self.minimum.le(value)? && value.le(&self.maximum)?)
    }
}

#[derive(Clone, Debug)]
pub struct Band {
    pub band: String,
    pub label: String,
    pub turnover_from: Decimal,
    pub turnover_from_inclusive: bool,
    pub turnover_to: Option<Decimal>,
    /// Ratio ranges in the order the Python dict held them.
    pub ratios: Vec<(String, Option<Range>)>,
    /// The ratios that came from the ATO's industry page.
    pub page_ratios: Vec<String>,
}

impl Band {
    pub fn ratio(&self, key: &str) -> Option<&Range> {
        self.ratios
            .iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, r)| r.as_ref())
    }

    pub fn is_page_ratio(&self, key: &str) -> bool {
        self.page_ratios.iter().any(|k| k == key)
    }

    pub fn contains(&self, turnover: &Decimal) -> Calc<bool> {
        if self.turnover_from_inclusive {
            if turnover.lt(&self.turnover_from)? {
                return Ok(false);
            }
        } else if turnover.le(&self.turnover_from)? {
            return Ok(false);
        }
        if let Some(to) = &self.turnover_to
            && turnover.gt(to)?
        {
            return Ok(false);
        }
        Ok(true)
    }
}

#[derive(Clone, Debug)]
pub struct BusinessType {
    pub name: String,
    pub key_ratio: String,
    pub bands: Vec<Band>,
    /// The industry page the page ranges came from: its five fields.
    pub page: Option<Vec<(String, Raw)>>,
}

impl BusinessType {
    pub fn band_for(&self, turnover: &Decimal) -> Calc<Option<&Band>> {
        for band in &self.bands {
            if band.contains(turnover)? {
                return Ok(Some(band));
            }
        }
        Ok(None)
    }

    pub fn page_field(&self, key: &str) -> Raw {
        self.page
            .as_ref()
            .and_then(|p| p.iter().find(|(k, _)| k == key))
            .map_or(Raw::None, |(_, v)| v.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Dataset {
    pub year: String,
    pub source: Raw,
    pub business_types: Vec<BusinessType>,
    pub page_source: Option<Raw>,
}

/// Case fold and flatten punctuation so "Fish and chips shops" matches input.
pub fn normalise(text: &str) -> String {
    let folded = unicode::casefold(&unicode::nfkd(text));
    let flattened = PUNCTUATION.sub(" ", &folded);
    pytext::strip(&SPACES.sub(" ", &flattened)).to_string()
}

impl Dataset {
    pub fn search(&self, term: &str, exact_only: bool) -> Vec<&BusinessType> {
        let needle = normalise(term);
        let exact: Vec<&BusinessType> = self
            .business_types
            .iter()
            .filter(|bt| normalise(&bt.name) == needle)
            .collect();
        if !exact.is_empty() || exact_only {
            return exact;
        }
        self.business_types
            .iter()
            .filter(|bt| normalise(&bt.name).contains(&needle))
            .collect()
    }

    pub fn get(&self, name: &str) -> Result<&BusinessType, Failure> {
        let exact = self.search(name, true);
        if exact.len() == 1 {
            return Ok(exact[0]);
        }
        let partial = self.search(name, false);
        match partial.len() {
            1 => Ok(partial[0]),
            0 => fail(format!(
                "no ATO business type matches {}. Run 'ato-benchmark-compare industries' to \
                 list all {} of them.",
                pytext::repr(name),
                self.business_types.len()
            )),
            n => {
                let listed: Vec<&str> =
                    partial.iter().take(10).map(|bt| bt.name.as_str()).collect();
                let more = if n <= 10 {
                    String::new()
                } else {
                    format!("\n  ... and {} more", n - 10)
                };
                fail(format!(
                    "{} matches more than one business type:\n  {}{more}",
                    pytext::repr(name),
                    listed.join("\n  ")
                ))
            }
        }
    }
}

/// `Decimal(str(value))`, refusing what is not a finite number.
fn decimal(value: &Raw, where_: &str) -> Result<Decimal, Failure> {
    let Ok(dec) = Decimal::parse(&value.py_str()) else {
        return fail(format!("{where_}: {} is not a number", value.py_repr()));
    };
    if !dec.is_finite() {
        return fail(format!(
            "{where_}: {} is not a finite number",
            value.py_repr()
        ));
    }
    Ok(dec)
}

fn range(raw: Option<&Raw>, where_: &str) -> Result<Option<Range>, Failure> {
    let raw = match raw {
        None | Some(Raw::None) => return Ok(None),
        Some(raw) => raw,
    };
    let (Some(min), Some(max)) = (raw.get("min"), raw.get("max")) else {
        return fail(format!("{where_}: expected an object with min and max"));
    };
    let low = decimal(min, &format!("{where_}.min"))?;
    let high = decimal(max, &format!("{where_}.max"))?;
    if low.gt(&high).expect("finite") {
        return fail(format!("{where_}: minimum {low} exceeds maximum {high}"));
    }
    Ok(Some(Range {
        minimum: low,
        maximum: high,
    }))
}

/// `value == n` for a JSON value and a Python int.
fn equals_int(value: Option<&Raw>, n: i64) -> bool {
    match value {
        Some(Raw::Int(digits)) => digits.parse::<i64>() == Ok(n),
        Some(Raw::Float(f)) => *f == n as f64,
        Some(Raw::Bool(b)) => i64::from(*b) == n,
        _ => false,
    }
}

fn repr_of(value: Option<&Raw>) -> String {
    value.map_or("None".into(), Raw::py_repr)
}

/// The items `for x in value` visits, or the TypeError Python raised.
fn iterate(value: &Raw) -> Result<Vec<Raw>, Failure> {
    match value {
        Raw::List(items) => Ok(items.clone()),
        Raw::Dict(items) => Ok(items.iter().map(|(k, _)| Raw::Str(k.clone())).collect()),
        Raw::Str(s) => Ok(pytext::code_points(s)
            .map(|cp| {
                let mut one = String::new();
                pytext::push_code_point(&mut one, cp);
                Raw::Str(one)
            })
            .collect()),
        other => Err(Failure::Uncaught(format!(
            "TypeError: '{}' object is not iterable",
            other.type_name()
        ))),
    }
}

fn parse_json(text: &str, source_name: &str) -> Result<Raw, Failure> {
    match pyjson::loads(text) {
        Ok(value) => Ok(Raw::from_json(&value)),
        Err(e) => fail(format!("{source_name}: not valid JSON: {e}")),
    }
}

/// `loads(text, source_name)`: one benchmark year.
pub fn loads(text: &str, source_name: &str) -> Result<Dataset, Failure> {
    let raw = parse_json(text, source_name)?;
    if !matches!(raw, Raw::Dict(_)) {
        return fail(format!("{source_name}: expected a JSON object"));
    }
    let version = raw.get("schema_version");
    if !equals_int(version, SUPPORTED_SCHEMA_VERSION) {
        return fail(format!(
            "{source_name}: schema version {} is not supported by this release (expected {SUPPORTED_SCHEMA_VERSION})",
            repr_of(version)
        ));
    }
    let year = match raw.get("benchmark_year") {
        Some(Raw::Str(y)) if !y.is_empty() => y.clone(),
        _ => return fail(format!("{source_name}: benchmark_year is missing")),
    };
    let entries = match raw.get("business_types") {
        Some(Raw::List(items)) if !items.is_empty() => items.clone(),
        _ => {
            return fail(format!("{source_name}: business_types is missing or empty"));
        }
    };
    let mut business_types = Vec::new();
    for entry in &entries {
        if !matches!(entry, Raw::Dict(_)) {
            return fail(format!("{source_name}: a business type is not an object"));
        }
        let name = match entry.get("name") {
            Some(Raw::Str(n)) if !pytext::strip(n).is_empty() => n.clone(),
            _ => return fail(format!("{source_name}: a business type has no name")),
        };
        let key_ratio = match entry.get("key_ratio") {
            Some(Raw::Str(k)) if RATIO_KEYS.contains(&k.as_str()) => k.clone(),
            other => {
                return fail(format!(
                    "{source_name}: {name}: unknown key_ratio {}",
                    repr_of(other)
                ));
            }
        };
        let mut bands = Vec::new();
        let band_values = match entry.get("turnover_bands") {
            None => Vec::new(),
            Some(value) => iterate(value)?,
        };
        for band_raw in &band_values {
            if !matches!(band_raw, Raw::Dict(_)) {
                return fail(format!(
                    "{source_name}: {name}: a turnover band is not an object"
                ));
            }
            let band_value = band_raw.get("band");
            let where_ = format!(
                "{source_name}: {name}/{}",
                band_value.map_or("None".into(), Raw::py_str)
            );
            let band_id = match band_value {
                Some(Raw::Str(b)) if !pytext::strip(b).is_empty() => b.clone(),
                _ => return fail(format!("{where_}: band is missing or empty")),
            };
            let inclusive = match band_raw.get("turnover_from_inclusive") {
                Some(Raw::Bool(b)) => *b,
                _ => {
                    return fail(format!(
                        "{where_}: turnover_from_inclusive must be a boolean"
                    ));
                }
            };
            let label = band_raw.get("label").map_or(String::new(), Raw::py_str);
            let turnover_from = decimal(
                band_raw.get("turnover_from").unwrap_or(&Raw::None),
                &format!("{where_}.turnover_from"),
            )?;
            let turnover_to = match band_raw.get("turnover_to") {
                None | Some(Raw::None) => None,
                Some(value) => Some(decimal(value, &format!("{where_}.turnover_to"))?),
            };
            let mut ratios = Vec::new();
            for key in RATIO_KEYS {
                ratios.push((
                    key.to_string(),
                    range(band_raw.get(key), &format!("{where_}.{key}"))?,
                ));
            }
            bands.push(Band {
                band: band_id,
                label,
                turnover_from,
                turnover_from_inclusive: inclusive,
                turnover_to,
                ratios,
                page_ratios: Vec::new(),
            });
        }
        if bands.is_empty() {
            return fail(format!("{source_name}: {name}: no turnover bands"));
        }
        business_types.push(BusinessType {
            name: pytext::strip(&name).to_string(),
            key_ratio,
            bands,
            page: None,
        });
    }
    let source = match raw.get("source") {
        Some(source @ Raw::Dict(_)) => source.clone(),
        _ => return fail(format!("{source_name}: source metadata is missing")),
    };
    Ok(Dataset {
        year,
        source,
        business_types,
        page_source: None,
    })
}

/// `json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)`.
fn dumps_canonical(value: &Json, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Int(i) => out.push_str(&i.to_string()),
        Json::Float(f) => out.push_str(&if f.is_nan() {
            "NaN".to_string()
        } else if f.is_infinite() {
            if *f > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
        } else {
            float_repr(*f)
        }),
        Json::Str(s) => {
            out.push('"');
            for cp in pytext::code_points(s) {
                match cp {
                    0x22 => out.push_str("\\\""),
                    0x5C => out.push_str("\\\\"),
                    0x0A => out.push_str("\\n"),
                    0x0D => out.push_str("\\r"),
                    0x09 => out.push_str("\\t"),
                    0x08 => out.push_str("\\b"),
                    0x0C => out.push_str("\\f"),
                    0x20..=0x7E => out.push(char::from_u32(cp).unwrap()),
                    0x10000.. => {
                        let v = cp - 0x10000;
                        out.push_str(&format!(
                            "\\u{:04x}\\u{:04x}",
                            0xD800 + (v >> 10),
                            0xDC00 + (v & 0x3FF)
                        ));
                    }
                    _ => out.push_str(&format!("\\u{cp:04x}")),
                }
            }
            out.push('"');
        }
        Json::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                dumps_canonical(item, out);
            }
            out.push(']');
        }
        Json::Object(items) => {
            let mut sorted: Vec<&(String, Json)> = items.iter().collect();
            sorted.sort_by(|a, b| pytext::code_points(&a.0).cmp(pytext::code_points(&b.0)));
            out.push('{');
            for (i, (key, item)) in sorted.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                dumps_canonical(&Json::Str(key.clone()), out);
                out.push(':');
                dumps_canonical(item, out);
            }
            out.push('}');
        }
    }
}

/// SHA-256 of a JSON document's content, whatever its line endings or layout.
pub fn canonical_digest(text: &str) -> Result<String, Failure> {
    let value = pyjson::loads(text).map_err(|e| Failure::Value(format!("not valid JSON: {e}")))?;
    let mut canonical = String::new();
    dumps_canonical(&value, &mut canonical);
    Ok(Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Add the ranges published only on the ATO's industry pages to a loaded year.
pub fn merge_page_ranges(
    dataset: &Dataset,
    text: &str,
    source_name: &str,
    dataset_digest: Option<&Raw>,
) -> Result<Dataset, Failure> {
    let raw = parse_json(text, source_name)?;
    if !matches!(raw, Raw::Dict(_)) {
        return fail(format!("{source_name}: expected a JSON object"));
    }
    if !equals_int(raw.get("schema_version"), SUPPORTED_SCHEMA_VERSION) {
        return fail(format!(
            "{source_name}: schema version {} is not supported by this release (expected {SUPPORTED_SCHEMA_VERSION})",
            repr_of(raw.get("schema_version"))
        ));
    }
    if raw.get("benchmark_year") != Some(&Raw::Str(dataset.year.clone())) {
        return fail(format!(
            "{source_name}: benchmark year {} does not match the dataset's {}",
            repr_of(raw.get("benchmark_year")),
            pytext::repr(&dataset.year)
        ));
    }
    let source = match raw.get("source") {
        Some(source @ Raw::Dict(_)) => source.clone(),
        _ => return fail(format!("{source_name}: source metadata is missing")),
    };
    let none = Raw::None;
    let built_against = source.get("dataset_sha256").unwrap_or(&none);
    if !built_against.py_eq(dataset.source.get("sha256").unwrap_or(&none)) {
        return fail(format!(
            "{source_name}: built against a different dataset file (sha256 {})",
            repr_of(source.get("dataset_sha256"))
        ));
    }
    if let Some(digest) = dataset_digest.filter(|d| **d != Raw::None)
        && !source
            .get("dataset_json_sha256")
            .unwrap_or(&none)
            .py_eq(digest)
    {
        return fail(format!(
            "{source_name}: built against a different {} dataset file; rebuild it with \
             tools/build_other_benchmarks.py",
            dataset.year
        ));
    }
    let entries = match raw.get("business_types") {
        Some(Raw::List(items)) => items.clone(),
        _ => return fail(format!("{source_name}: business_types is missing")),
    };
    if !equals_int(raw.get("business_type_count"), entries.len() as i64) {
        return fail(format!(
            "{source_name}: business_type_count does not match business_types"
        ));
    }
    let excluded: Vec<String> = match raw.get("excluded_pages") {
        Some(Raw::List(items))
            if items.iter().all(|i| {
                matches!(i.get("name"), Some(Raw::Str(_))) && matches!(i, Raw::Dict(_))
            }) =>
        {
            items
                .iter()
                .map(|i| match i.get("name") {
                    Some(Raw::Str(n)) => n.clone(),
                    _ => unreachable!("checked"),
                })
                .collect()
        }
        _ => {
            return fail(format!(
                "{source_name}: excluded_pages is missing or malformed"
            ));
        }
    };
    let mut merged: Vec<BusinessType> = Vec::new();
    for entry in &entries {
        if !matches!(entry, Raw::Dict(_)) {
            return fail(format!("{source_name}: a business type is not an object"));
        }
        let name = entry.get("name");
        let business_type = match name {
            // `{bt.name: bt for bt in ...}`: the last of duplicate names wins.
            Some(Raw::Str(n)) => dataset.business_types.iter().rev().find(|bt| &bt.name == n),
            _ => None,
        };
        let Some(business_type) = business_type else {
            return fail(format!(
                "{source_name}: {} is not a business type in {}",
                repr_of(name),
                dataset.year
            ));
        };
        let shown = name.map_or("None".into(), Raw::py_str);
        if merged.iter().any(|bt| bt.name == business_type.name) {
            return fail(format!(
                "{source_name}: {} appears twice",
                business_type.name
            ));
        }
        let bands_raw = match entry.get("turnover_bands") {
            Some(Raw::List(items)) => Some(items.clone()),
            _ => None,
        };
        let matches_bands = bands_raw.as_ref().is_some_and(|items| {
            items.len() == business_type.bands.len()
                && items.iter().zip(&business_type.bands).all(|(b, band)| {
                    matches!(b, Raw::Dict(_)) && b.get("band") == Some(&Raw::Str(band.band.clone()))
                })
        });
        if !matches_bands {
            return fail(format!(
                "{source_name}: {shown}: turnover bands do not match the dataset"
            ));
        }
        let mut bands = Vec::new();
        for (band, band_raw) in business_type.bands.iter().zip(bands_raw.unwrap()) {
            let where_ = format!("{source_name}: {shown}/{}", band.band);
            let Raw::Dict(items) = &band_raw else {
                unreachable!("checked")
            };
            let mut unknown: Vec<&String> = items
                .iter()
                .map(|(k, _)| k)
                .filter(|k| *k != "band" && !PAGE_RATIO_KEYS.contains(&k.as_str()))
                .collect();
            if !unknown.is_empty() {
                unknown.sort_by(|a, b| pytext::code_points(a).cmp(pytext::code_points(b)));
                let listed: Vec<String> = unknown.iter().map(|k| pytext::repr(k)).collect();
                return fail(format!(
                    "{where_}: unknown ratio key(s) [{}]",
                    listed.join(", ")
                ));
            }
            let mut ratios = band.ratios.clone();
            let mut added = Vec::new();
            for key in PAGE_RATIO_KEYS {
                let Some(page_range) = range(band_raw.get(key), &format!("{where_}.{key}"))? else {
                    continue;
                };
                if band.ratio(key).is_some() {
                    return fail(format!(
                        "{where_}: {key} would replace the workbook's own range"
                    ));
                }
                match ratios.iter_mut().find(|(k, _)| k == key) {
                    Some(slot) => slot.1 = Some(page_range),
                    None => ratios.push((key.to_string(), Some(page_range))),
                }
                added.push(key.to_string());
            }
            bands.push(Band {
                ratios,
                page_ratios: added,
                ..band.clone()
            });
        }
        let page = [
            "page_title",
            "page_url",
            "page_last_updated",
            "page_reference",
            "text_sha256",
        ]
        .iter()
        .map(|k| (k.to_string(), entry.get(k).cloned().unwrap_or(Raw::None)))
        .collect();
        merged.push(BusinessType {
            name: business_type.name.clone(),
            key_ratio: business_type.key_ratio.clone(),
            bands,
            page: Some(page),
        });
    }
    let known: Vec<&String> = dataset.business_types.iter().map(|bt| &bt.name).collect();
    let mut excluded_unique = excluded.clone();
    excluded_unique.sort();
    excluded_unique.dedup();
    let overlap = excluded
        .iter()
        .any(|n| merged.iter().any(|bt| &bt.name == n));
    let union_ok = excluded
        .iter()
        .chain(merged.iter().map(|bt| &bt.name))
        .all(|n| known.contains(&n))
        && known
            .iter()
            .all(|n| excluded.contains(n) || merged.iter().any(|bt| &bt.name == *n));
    if excluded_unique.len() != excluded.len() || overlap || !union_ok {
        return fail(format!(
            "{source_name}: included and excluded business types do not account for the {} in {}",
            {
                let mut unique: Vec<&String> = known.clone();
                unique.sort();
                unique.dedup();
                unique.len()
            },
            dataset.year
        ));
    }
    Ok(Dataset {
        year: dataset.year.clone(),
        source: dataset.source.clone(),
        business_types: dataset
            .business_types
            .iter()
            .map(|bt| {
                merged
                    .iter()
                    .find(|m| m.name == bt.name)
                    .cloned()
                    .unwrap_or_else(|| bt.clone())
            })
            .collect(),
        page_source: Some(source),
    })
}

/// Where the datasets come from: the compiled-in copies or a directory.
pub enum DataDir<'a> {
    Shipped,
    Path(&'a PyPath),
}

/// The benchmark years a data directory holds, sorted.
pub fn available_years(dir: &DataDir) -> Vec<String> {
    let names: Vec<String> = match dir {
        DataDir::Shipped => SHIPPED.iter().map(|(name, _)| name.to_string()).collect(),
        DataDir::Path(path) => match std::fs::read_dir(path.fs()) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| crate::cli::decode_os(&e.file_name()))
                .collect(),
            Err(_) => Vec::new(),
        },
    };
    let mut years: Vec<String> = names
        .iter()
        .filter(|name| FILENAME.matches_start(name))
        .map(|name| {
            let inner: String = name.chars().skip("benchmarks-".len()).take(7).collect();
            inner
        })
        .collect();
    years.sort_by(|a, b| pytext::code_points(a).cmp(pytext::code_points(b)));
    years
}

/// A data file's text and the name its errors carry, or None when absent.
fn read_data(dir: &DataDir, name: &str) -> Result<Option<(String, String)>, Failure> {
    match dir {
        DataDir::Shipped => Ok(SHIPPED
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(n, text)| (py::universal_newlines(text), format!("data/{n}")))),
        DataDir::Path(path) => {
            let file = path.join(name);
            if !file.fs().is_file() {
                return Ok(None);
            }
            let shown = file.display();
            match py::read_text(&file.fs()) {
                Ok(text) => Ok(Some((text, shown))),
                Err(py::ReadError::Os(e)) => Err(Failure::Value(py::oserror_str(
                    &e,
                    &shown,
                    py::OsStyle::Errno,
                ))),
                Err(py::ReadError::Decode(e)) => {
                    Err(Failure::Uncaught(format!("UnicodeDecodeError: {e}")))
                }
            }
        }
    }
}

/// `load(year, data_dir)`: a benchmark year, by default the most recent.
pub fn load(year: Option<&str>, dir: &DataDir) -> Result<Dataset, Failure> {
    let available = available_years(dir);
    if available.is_empty() {
        let shown = match dir {
            DataDir::Shipped => "data".to_string(),
            DataDir::Path(p) => p.display(),
        };
        return fail(format!("no benchmark data files found in {shown}"));
    }
    let chosen = match year {
        Some(y) if !y.is_empty() => y.to_string(),
        _ => available.last().unwrap().clone(),
    };
    let Some((text, shown)) = read_data(dir, &format!("benchmarks-{chosen}.json"))? else {
        return fail(format!(
            "no dataset for benchmark year {}. Available: {}",
            pytext::repr(&chosen),
            available.join(", ")
        ));
    };
    let dataset = loads(&text, &shown)?;
    match read_data(dir, &format!("other-benchmarks-{chosen}.json"))? {
        Some((pages, pages_shown)) => merge_page_ranges(
            &dataset,
            &pages,
            &pages_shown,
            Some(&Raw::Str(canonical_digest(&text)?)),
        ),
        None => Ok(dataset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_years_load_with_their_page_ranges() {
        assert_eq!(available_years(&DataDir::Shipped), ["2022-23", "2023-24"]);
        let data = load(None, &DataDir::Shipped).unwrap();
        assert_eq!(data.year, "2023-24");
        assert_eq!(data.business_types.len(), 100);
        assert!(data.page_source.is_some());
        let bakeries = data.get("bakeries").unwrap();
        assert_eq!(bakeries.name, "Bakeries and hot bread shops");
        assert!(
            load(Some("2022-23"), &DataDir::Shipped)
                .unwrap()
                .page_source
                .is_none()
        );
    }

    #[test]
    fn normalise_flattens_dashes_and_quotes() {
        assert_eq!(
            normalise("  Fish \u{2013} and\u{2019}chips  "),
            "fish and chips"
        );
        assert_eq!(normalise("CAF\u{C9}"), "cafe\u{301}");
    }
}
