// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The dataset rules of the Python `build_dataset.py`: ratios, turnover band
//! labels and the bands built from one workbook row.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use ato_benchmark_compare::decimal::Decimal;
use ato_benchmark_compare::py::{decimal_value, is_space, repr_str};
use serde_json::{Value, json};

use crate::shared::sha256_hex;
use crate::xlsx::{COLS_READ, CellValue, SheetRows, read_first_sheet_rows};

const BAND_SPEC: [(&str, usize, usize, usize, usize, usize); 3] = [
    ("low", 1, 2, 7, 8, 13),
    ("medium", 3, 4, 9, 10, 14),
    ("high", 5, 6, 11, 12, 15),
];

fn money(text: &str) -> Result<Decimal, String> {
    Decimal::parse(&text.replace(',', ""))
        .map_err(|e| format!("invalid money amount {text:?}: {}", e.name()))
}

fn dec_gt(a: &Decimal, b: &Decimal) -> Result<bool, String> {
    a.gt(b).map_err(|e| format!("decimal error: {}", e.name()))
}

fn dec_lt(a: &Decimal, b: &Decimal) -> Result<bool, String> {
    a.lt(b).map_err(|e| format!("decimal error: {}", e.name()))
}

fn dec_eq(a: &Decimal, b: &Decimal) -> Result<bool, String> {
    Ok(!dec_lt(a, b)? && !dec_gt(a, b)?)
}

pub fn ratio(value: &CellValue, where_: &str) -> Result<Option<String>, String> {
    if value.is_none() {
        return Ok(None);
    }
    if let CellValue::Str(s) = value
        && s.trim_matches(is_space).is_empty()
    {
        return Ok(None);
    }
    let as_str = match value {
        CellValue::Str(s) => s.trim_matches(is_space).to_string(),
        other => other.py_str(),
    };
    let dec = Decimal::parse(&as_str).map_err(|e| {
        format!(
            "{where_}: invalid decimal {}: {}",
            value.py_repr(),
            e.name()
        )
    })?;
    if !dec.is_finite() {
        return Err(format!("{where_}: non finite ratio {}", value.py_repr()));
    }
    if dec_lt(&dec, &Decimal::from_int(0))? || dec_gt(&dec, &Decimal::from_int(10))? {
        return Err(format!(
            "{where_}: ratio {} is outside any plausible range",
            dec.to_sci_string()
        ));
    }
    Ok(Some(dec.to_sci_string()))
}

fn scan_money_token(s: &str) -> Option<(&str, &str)> {
    let rest = s.strip_prefix('$')?;
    let mut end = 0;
    for (idx, ch) in rest.char_indices() {
        if ch == ',' || decimal_value(ch).is_some() {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    (end > 0).then(|| (&rest[..end], &rest[end..]))
}

pub fn parse_range_label(
    label: &CellValue,
    where_: &str,
) -> Result<Option<(Decimal, Option<Decimal>)>, String> {
    if label.is_none() {
        return Ok(None);
    }
    let raw = label.py_str();
    let text = raw.trim_matches(is_space);
    if text.is_empty() || text.to_uppercase() == "N/A" {
        return Ok(None);
    }
    if let Some((first, after_first)) = scan_money_token(text) {
        let mid = after_first.trim_start_matches(is_space);
        let mut chars = mid.chars();
        if let Some(dash) = chars.next()
            && matches!(dash, '-' | '\u{2013}' | '\u{2014}')
        {
            let after_dash = chars.as_str().trim_start_matches(is_space);
            if let Some((second, tail)) = scan_money_token(after_dash)
                && tail.is_empty()
            {
                let (low, high) = (money(first)?, money(second)?);
                if dec_gt(&low, &high)? {
                    return Err(format!(
                        "{where_}: range label {} runs backwards",
                        repr_str(text)
                    ));
                }
                return Ok(Some((low, Some(high))));
            }
        }
    }
    let lower = text.to_ascii_lowercase();
    if lower.starts_with("more than ") {
        let rest = &text["more than ".len()..];
        if let Some((amount, tail)) = scan_money_token(rest)
            && tail.is_empty()
        {
            return Ok(Some((money(amount)?, None)));
        }
    }
    Err(format!(
        "{where_}: unrecognised turnover range label {}",
        repr_str(text)
    ))
}

fn pair(
    minimum: Option<String>,
    maximum: Option<String>,
    where_: &str,
) -> Result<Option<Value>, String> {
    match (minimum, maximum) {
        (None, None) => Ok(None),
        (Some(min), Some(max)) => {
            let dmin = Decimal::parse(&min)
                .map_err(|e| format!("{where_}: invalid decimal {min}: {}", e.name()))?;
            let dmax = Decimal::parse(&max)
                .map_err(|e| format!("{where_}: invalid decimal {max}: {}", e.name()))?;
            if dec_gt(&dmin, &dmax)? {
                return Err(format!("{where_}: minimum {min} exceeds maximum {max}"));
            }
            Ok(Some(json!({"min": min, "max": max})))
        }
        _ => Err(format!("{where_}: only one end of the range is published")),
    }
}

/// One band's entry: its bounds and both ratio pairs, read from `row`.
fn band_entry(
    row: &[CellValue],
    spec: (&str, usize, usize, usize, usize, usize),
    name: &str,
    from: (Decimal, bool),
    printed_to: Option<&Decimal>,
) -> Result<Value, String> {
    let (band, te_min, te_max, cos_min, cos_max, label_col) = spec;
    let ratio_pair = |lo: usize, hi: usize, what: &str| {
        pair(
            ratio(&row[lo], &format!("{name}/{band}/{what}_min"))?,
            ratio(&row[hi], &format!("{name}/{band}/{what}_max"))?,
            &format!("{name}/{band}/{what}"),
        )
    };
    let te = ratio_pair(te_min, te_max, "total_expenses")?;
    let cos = ratio_pair(cos_min, cos_max, "cost_of_sales")?;
    Ok(json!({
        "band": band,
        "label": row[label_col].py_str().trim_matches(is_space),
        "turnover_from": from.0.to_sci_string(),
        "turnover_from_inclusive": from.1,
        "turnover_to": printed_to.map(Decimal::to_sci_string),
        "total_expenses_to_turnover": te,
        "cost_of_sales_to_turnover": cos,
    }))
}

/// The ATO prints adjoining bands as "$50,000 - $150,000" then "$150,001 -
/// $600,000", so every band after the first starts, exclusive, where the one
/// before it ends; a gap other than 0 or 1 dollar is reported as an anomaly.
fn band_start(
    printed_from: &Decimal,
    previous_to: Option<&Decimal>,
    where_: &str,
    anomalies: &mut Vec<String>,
) -> Result<(Decimal, bool), String> {
    let Some(prev) = previous_to else {
        return Ok((printed_from.clone(), true));
    };
    let diff = printed_from
        .sub(prev)
        .map_err(|e| format!("decimal error: {}", e.name()))?;
    if !dec_eq(&diff, &Decimal::from_int(0))? && !dec_eq(&diff, &Decimal::from_int(1))? {
        anomalies.push(format!(
            "{where_}: band starts at {} but the previous band ends at {}",
            printed_from.to_sci_string(),
            prev.to_sci_string()
        ));
    }
    Ok((prev.clone(), false))
}

pub fn build_bands(
    row: &[CellValue],
    name: &str,
    anomalies: &mut Vec<String>,
) -> Result<Vec<Value>, String> {
    let mut bands = Vec::new();
    let mut previous_to: Option<Decimal> = None;
    for spec in BAND_SPEC {
        let where_ = format!("{name}/{}", spec.0);
        let Some((printed_from, printed_to)) = parse_range_label(&row[spec.5], &where_)? else {
            continue;
        };
        let from = band_start(&printed_from, previous_to.as_ref(), &where_, anomalies)?;
        bands.push(band_entry(row, spec, name, from, printed_to.as_ref())?);
        previous_to = printed_to;
    }
    if bands.is_empty() {
        return Err(format!("{name}: no turnover bands published"));
    }
    Ok(bands)
}

pub fn validate_header(header: &[CellValue; COLS_READ], max_col: usize) -> Result<(), String> {
    if header[0].is_none() || !header[0].py_str().contains("BusinessType") {
        return Err(format!(
            "unexpected first header cell: {}",
            header[0].py_repr()
        ));
    }
    if max_col < COLS_READ {
        return Err(format!("expected 16 columns, found {max_col}"));
    }
    for &(_, lo, hi, _, _, _) in &BAND_SPEC {
        for col in [lo, hi] {
            if !header[col].py_str().contains("Total Expenses") {
                return Err(format!(
                    "column {col} is not a Total Expenses column: {}",
                    header[col].py_repr()
                ));
            }
        }
    }
    for &(_, _, _, lo, hi, _) in &BAND_SPEC {
        for col in [lo, hi] {
            if !header[col].py_str().contains("Cost of Sales") {
                return Err(format!(
                    "column {col} is not a Cost of Sales column: {}",
                    header[col].py_repr()
                ));
            }
        }
    }
    Ok(())
}

/// The `--year` and resource flags `build_dataset.py` copied into `source`.
pub struct Resource {
    pub year: String,
    pub name: String,
    pub url: String,
    pub last_modified: String,
    pub retrieved: String,
}

/// One entry per named row after the header, in workbook order.
fn business_types(sheet: &SheetRows, anomalies: &mut Vec<String>) -> Result<Vec<Value>, String> {
    let mut business_types = Vec::new();
    let mut seen = BTreeSet::new();
    for row in sheet.rows.range(2..).map(|(_, row)| row) {
        let name = row[0].py_str().trim_matches(is_space).to_string();
        if row[0].is_none() || name.is_empty() {
            continue;
        }
        if !seen.insert(name.clone()) {
            return Err(format!("duplicate business type {}", repr_str(&name)));
        }
        let bands = build_bands(row, &name, anomalies)?;
        let key_ratio = if bands
            .iter()
            .any(|b| !b["cost_of_sales_to_turnover"].is_null())
        {
            "cost_of_sales_to_turnover"
        } else {
            "total_expenses_to_turnover"
        };
        business_types.push(json!({
            "name": name,
            "key_ratio": key_ratio,
            "turnover_bands": bands,
        }));
    }
    Ok(business_types)
}

/// The dataset and its anomaly lines, as `build_dataset.py`'s `build` made them.
pub fn build(xlsx: &Path, resource: &Resource) -> Result<(Value, Vec<String>), String> {
    let payload = fs::read(xlsx).map_err(|e| format!("{}: {e}", xlsx.display()))?;
    let sheet = read_first_sheet_rows(&payload)?;
    if sheet.max_row == 0 || sheet.max_col == 0 {
        return Err("workbook is empty".to_string());
    }
    let empty_header: [CellValue; COLS_READ] = std::array::from_fn(|_| CellValue::Empty);
    validate_header(sheet.rows.get(&1).unwrap_or(&empty_header), sheet.max_col)?;
    let mut anomalies = Vec::new();
    let business_types = business_types(&sheet, &mut anomalies)?;
    let dataset = json!({
        "schema_version": 1,
        "benchmark_year": resource.year,
        "business_type_count": business_types.len(),
        "source": {
            "publisher": "Australian Taxation Office",
            "dataset": "Small Business Benchmarks",
            "dataset_page": "https://data.gov.au/data/dataset/small-business-benchmarks",
            "resource_name": resource.name,
            "resource_url": resource.url,
            "resource_last_modified": resource.last_modified,
            "retrieved": resource.retrieved,
            "sha256": sha256_hex(&payload),
            "bytes": payload.len(),
            "licence": "Creative Commons Attribution 2.5 Australia",
            "licence_url": "https://creativecommons.org/licenses/by/2.5/au/",
        },
        "business_types": business_types,
    });
    Ok((dataset, anomalies))
}
