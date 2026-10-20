//! JSON stdin boundary for the removed Python import API. Decimals are
//! strings, files are paths, and a dataset is a year (with an optional data
//! directory) or JSON text. Results mirror the Python objects' fields.

use serde_json::{Map, Value, json};

use crate::Failure;
use crate::csvsafe::guard;
use crate::dataset::{self, DataDir, Dataset, canonical_digest, merge_page_ranges, normalise};
use crate::decimal::Decimal;
use crate::mapping::{self, MappingRow, account_key, normalise_account, suggest};
use crate::money;
use crate::pnl::{self, PnlRow};
use crate::py::PyPath;
use crate::ratios::{Figures, compute};
use crate::raw::Raw;
use crate::report::{self, Comparison, raw_json};

type Result<T> = std::result::Result<T, String>;

pub const COMMANDS: [&str; 26] = [
    "parse-amount",
    "money",
    "percent",
    "percent-compact",
    "percent-range",
    "guard",
    "normalise",
    "normalise-account",
    "account-key",
    "suggest",
    "is-total-row",
    "section-for",
    "canonical-digest",
    "available-years",
    "load-dataset",
    "loads",
    "merge-page-ranges",
    "search",
    "get",
    "read-pnl",
    "suggest-mapping",
    "read-mapping",
    "write-mapping",
    "route",
    "compute",
    "compare-figures",
];

fn failure(f: Failure) -> String {
    f.to_string()
}

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))
}

fn optional_text<'a>(v: &'a Value, key: &str) -> Result<Option<&'a str>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(format!("{key} must be a string or null")),
    }
}

/// A keyword argument with a default: the default when the key is absent,
/// otherwise `str()` of the value given, `None` included, as an f-string
/// prints it.
fn named<'a>(v: &Value, key: &str, default: &'a str) -> std::borrow::Cow<'a, str> {
    match v.get(key) {
        None => std::borrow::Cow::Borrowed(default),
        Some(value) => std::borrow::Cow::Owned(Raw::from_value(value).py_str()),
    }
}

fn decimal(text: &str) -> Result<Decimal> {
    Decimal::parse(text).map_err(|s| format!("decimal.{}", s.name()))
}

fn strings(v: &Value, key: &str) -> Result<Option<Vec<String>>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => items
            .iter()
            .map(|i| {
                i.as_str()
                    .map(String::from)
                    .ok_or(format!("{key} must hold strings"))
            })
            .collect::<Result<_>>()
            .map(Some),
        Some(_) => Err(format!("{key} must be a list")),
    }
}

/// A dataset named by `year`/`data_dir`, or built from `text` and `pages_text`.
fn dataset_of(v: &Value) -> Result<Dataset> {
    if let Some(text) = optional_text(v, "text")? {
        let name = named(v, "source_name", "<string>");
        let data = dataset::loads(text, &name).map_err(failure)?;
        return match optional_text(v, "pages_text")? {
            None => Ok(data),
            Some(pages) => merge_page_ranges(
                &data,
                pages,
                &named(v, "pages_name", "<string>"),
                v.get("dataset_digest").map(Raw::from_value).as_ref(),
            )
            .map_err(failure),
        };
    }
    let dir_text = optional_text(v, "data_dir")?;
    let path;
    let dir = match dir_text {
        Some(p) => {
            path = PyPath::new(p);
            DataDir::Path(&path)
        }
        None => DataDir::Shipped,
    };
    dataset::load(optional_text(v, "year")?, &dir).map_err(failure)
}

fn sorted(items: &[String]) -> Vec<String> {
    let mut items = items.to_vec();
    items.sort();
    items
}

fn decimal_json(value: &Decimal) -> Value {
    json!(value.to_sci_string())
}

/// A dataset's contents, for comparing loads.
fn dataset_json(data: &Dataset) -> Value {
    json!({
        "year": data.year,
        "source": raw_json(&data.source),
        "page_source": data.page_source.as_ref().map(raw_json),
        "business_types": data.business_types.iter().map(|bt| json!({
            "name": bt.name,
            "key_ratio": bt.key_ratio,
            "page": bt.page.as_ref().map(|p| p.iter().map(|(k, v)| json!([k, raw_json(v)])).collect::<Vec<_>>()),
            "bands": bt.bands.iter().map(|b| json!({
                "band": b.band,
                "label": b.label,
                "turnover_from": decimal_json(&b.turnover_from),
                "turnover_from_inclusive": b.turnover_from_inclusive,
                "turnover_to": b.turnover_to.as_ref().map(decimal_json),
                "ratios": b.ratios.iter().map(|(k, r)| json!([k, r.as_ref().map(|r| json!([decimal_json(&r.minimum), decimal_json(&r.maximum)]))])).collect::<Vec<_>>(),
                "page_ratios": sorted(&b.page_ratios),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

fn pnl_rows(v: &Value) -> Result<Vec<PnlRow>> {
    let Some(Value::Array(items)) = v.get("rows") else {
        return Err("rows must be a list".into());
    };
    items
        .iter()
        .map(|row| {
            Ok(PnlRow {
                account: text(row, "account")?.to_string(),
                amount: decimal(text(row, "amount")?)?,
                line_number: row["line_number"].as_u64().unwrap_or(0) as usize,
                section: optional_text(row, "section")?.map(String::from),
                is_total: row["is_total"].as_bool().unwrap_or(false),
            })
        })
        .collect()
}

fn pnl_rows_json(rows: &[PnlRow]) -> Value {
    json!(
        rows.iter()
            .map(|r| json!({
                "account": r.account,
                "amount": decimal_json(&r.amount),
                "line_number": r.line_number,
                "section": r.section,
                "is_total": r.is_total,
            }))
            .collect::<Vec<_>>()
    )
}

fn mapping_row_json(row: &MappingRow) -> Value {
    json!({"account": row.account, "bucket": row.bucket, "source": row.source,
           "note": row.note, "amount": row.amount})
}

fn mapping_row(v: &Value) -> Result<MappingRow> {
    let field =
        |key: &str| -> Result<String> { Ok(optional_text(v, key)?.unwrap_or("").to_string()) };
    Ok(MappingRow {
        account: field("account")?,
        bucket: field("bucket")?,
        source: field("source")?,
        note: field("note")?,
        amount: field("amount")?,
    })
}

/// Bucket totals as given, in key order.
fn totals_of(v: &Value) -> Result<Vec<(String, Decimal)>> {
    let Some(Value::Object(items)) = v.get("totals") else {
        return Err("totals must be an object".into());
    };
    items
        .iter()
        .map(|(k, value)| {
            let text = value.as_str().ok_or("totals hold decimal strings")?;
            Ok((k.clone(), decimal(text)?))
        })
        .collect()
}

fn figures_json(f: &Figures) -> Value {
    let mut supplied = f.supplied_fields.clone();
    supplied.sort();
    json!({
        "totals": f.totals.iter().map(|(k, v)| json!([k, decimal_json(v)])).collect::<Vec<_>>(),
        "turnover": decimal_json(&f.turnover),
        "turnover_basis": f.turnover_basis,
        "trading_sales": decimal_json(&f.trading_sales),
        "other_income": decimal_json(&f.other_income),
        "total_business_income": decimal_json(&f.total_business_income),
        "total_expenses_reported": decimal_json(&f.total_expenses_reported),
        "total_expenses_for_ratio": decimal_json(&f.total_expenses_for_ratio),
        "cost_of_sales_for_ratio": decimal_json(&f.cost_of_sales_for_ratio),
        "labour": decimal_json(&f.labour),
        "ratios": f.ratios.iter().map(|(k, v)| json!([k, decimal_json(v)])).collect::<Vec<_>>(),
        "warnings": f.warning_details.iter().map(|d| {
            let mut fields = d.required_fields.clone();
            fields.sort();
            json!([d.code, d.text, fields])
        }).collect::<Vec<_>>(),
        "supplied_fields": supplied,
    })
}

/// `compute` from a request: totals, the keys it supplied, and W1.
fn figures_of(v: &Value) -> Result<Figures> {
    let totals = totals_of(v)?;
    let supplied = match strings(v, "totals_supplied")? {
        Some(s) => s,
        None => totals.iter().map(|(k, _)| k.clone()).collect(),
    };
    let w1 = optional_text(v, "w1")?.map(decimal).transpose()?;
    compute(&totals, &supplied, w1.as_ref()).map_err(failure)
}

fn comparison_json(c: &Comparison) -> Value {
    json!({
        "key_ratio": c.key_ratio,
        "band": c.band.as_ref().map(|b| b.band.clone()),
        "notes": c.note_details.iter().map(|d| {
            let mut fields = d.required_fields.clone();
            fields.sort();
            json!([d.code, d.text, fields])
        }).collect::<Vec<_>>(),
        "verdicts": c.verdicts.iter().map(|v| json!({
            "key": v.key, "label": v.label, "ratio": decimal_json(&v.ratio),
            "benchmark": v.benchmark.as_ref().map(|r| json!([decimal_json(&r.minimum), decimal_json(&r.maximum)])),
            "status": v.status, "is_key": v.is_key, "benchmark_source": v.benchmark_source,
        })).collect::<Vec<_>>(),
    })
}

fn map_value(m: Map<String, Value>) -> Value {
    Value::Object(m)
}

/// Run one JSON stdin command: the result value, or the refusal message.
/// `json.loads` refuses an integer token of more than 4,300 digits, as
/// `int(text)` does under Python's default limit.
fn int_limit(value: &Value) -> Result<()> {
    match value {
        Value::Number(n) => {
            let text = n.to_string();
            let digits = text.strip_prefix('-').unwrap_or(&text);
            if digits.len() > 4300 && digits.bytes().all(|b| b.is_ascii_digit()) {
                return Err(format!(
                    "ValueError: Exceeds the limit (4300 digits) for integer string conversion: \
                     value has {} digits; use sys.set_int_max_str_digits() to increase the limit",
                    digits.len()
                ));
            }
            Ok(())
        }
        Value::Array(items) => items.iter().try_for_each(int_limit),
        Value::Object(items) => items.values().try_for_each(int_limit),
        _ => Ok(()),
    }
}

pub fn evaluate(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return Err("input must be a JSON object".into());
    }
    int_limit(v)?;
    let amount_err = |e: money::AmountError| e.0;
    match command {
        "parse-amount" => {
            let raw = match v.get("raw") {
                None | Some(Value::Null) => None,
                Some(other) => Some(Raw::from_value(other)),
            };
            let where_ = named(v, "where", "amount");
            let text = raw.as_ref().map(Raw::py_str);
            money::parse_amount_shown(text.as_deref(), raw.as_ref().map(Raw::py_repr), &where_)
                .map(|d| decimal_json(&d))
                .map_err(amount_err)
        }
        "money" | "percent" | "percent-compact" => {
            let value = decimal(text(v, "value")?)?;
            match command {
                "money" => money::money(&value),
                "percent" => money::percent(&value),
                _ => money::percent_compact(&value),
            }
            .map(|t| json!(t))
            .map_err(amount_err)
        }
        "percent-range" => {
            let (low, high) = (decimal(text(v, "low")?)?, decimal(text(v, "high")?)?);
            money::percent_range(&low, &high)
                .map(|t| json!(t))
                .map_err(amount_err)
        }
        // `guard(value)` takes any object: `str(value)`, or "" for None.
        "guard" => {
            let value = v.get("value").map(Raw::from_value).unwrap_or(Raw::None);
            let text = (value != Raw::None).then(|| value.py_str());
            Ok(json!(guard(text.as_deref())))
        }
        "normalise" => Ok(json!(normalise(text(v, "text")?))),
        "normalise-account" => Ok(json!(normalise_account(text(v, "text")?))),
        "account-key" => Ok(json!(account_key(text(v, "text")?))),
        "suggest" => {
            let (bucket, reason) = suggest(text(v, "account")?, optional_text(v, "section")?);
            Ok(json!([bucket, reason]))
        }
        "is-total-row" => Ok(json!(pnl::is_total_row(text(v, "label")?))),
        "section-for" => Ok(json!(pnl::section_for(text(v, "label")?))),
        "canonical-digest" => canonical_digest(text(v, "text")?)
            .map(|d| json!(d))
            .map_err(failure),
        "available-years" => {
            let path = optional_text(v, "data_dir")?.map(PyPath::new);
            let dir = match &path {
                Some(p) => DataDir::Path(p),
                None => DataDir::Shipped,
            };
            Ok(json!(dataset::available_years(&dir)))
        }
        "load-dataset" | "loads" | "merge-page-ranges" => dataset_of(v).map(|d| dataset_json(&d)),
        "search" => {
            let data = dataset_of(v)?;
            let exact = v
                .get("exact_only")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            Ok(json!(
                data.search(text(v, "term")?, exact)
                    .iter()
                    .map(|bt| bt.name.clone())
                    .collect::<Vec<_>>()
            ))
        }
        "get" => {
            let data = dataset_of(v)?;
            data.get(text(v, "name")?)
                .map(|bt| json!(bt.name))
                .map_err(failure)
        }
        "read-pnl" => {
            let file = pnl::read(
                &PyPath::new(text(v, "path")?),
                optional_text(v, "amount_column")?,
            )
            .map_err(failure)?;
            Ok(
                json!({"rows": pnl_rows_json(&file.rows), "layout": file.layout,
                      "amount_column": file.amount_column, "skipped": file.skipped}),
            )
        }
        "suggest-mapping" => {
            let draft = mapping::suggest_mapping(&pnl_rows(v)?).map_err(failure)?;
            Ok(
                json!({"rows": draft.rows.iter().map(mapping_row_json).collect::<Vec<_>>(),
                      "duplicates": draft.duplicates, "needs_review": draft.needs_review()}),
            )
        }
        "read-mapping" => {
            let rows = mapping::read_mapping(&PyPath::new(text(v, "path")?)).map_err(failure)?;
            Ok(json!(
                rows.iter()
                    .map(|(k, r)| json!([k, mapping_row_json(r)]))
                    .collect::<Vec<_>>()
            ))
        }
        "write-mapping" => {
            let Some(Value::Array(items)) = v.get("mapping_rows") else {
                return Err("mapping_rows must be a list".into());
            };
            let rows: Vec<MappingRow> = items.iter().map(mapping_row).collect::<Result<_>>()?;
            mapping::write_mapping(&PyPath::new(text(v, "path")?), &rows).map_err(failure)?;
            Ok(Value::Null)
        }
        "route" => {
            let rows = pnl_rows(v)?;
            let Some(Value::Array(items)) = v.get("mapping") else {
                return Err("mapping must be a list".into());
            };
            // A dict: a repeated key keeps its first place and its last value.
            let mut mapping: mapping::Mapping = Vec::new();
            for item in items {
                let row = mapping_row(&item[1])?;
                let key = item[0].as_str().ok_or("mapping keys are strings")?;
                match mapping.iter_mut().find(|(k, _)| k == key) {
                    Some(slot) => slot.1 = row,
                    None => mapping.push((key.to_string(), row)),
                }
            }
            let flip = v.get("flip").and_then(Value::as_bool).unwrap_or(false);
            let routed = mapping::route(&rows, &mapping, flip).map_err(failure)?;
            let mut supplied = routed.supplied_buckets.clone();
            supplied.sort();
            Ok(json!({
                "totals": routed.totals.iter().map(|(k, d)| json!([k, decimal_json(d)])).collect::<Vec<_>>(),
                "unreviewed": routed.unreviewed,
                "notes": routed.notes,
                "supplied_buckets": supplied,
            }))
        }
        "compute" => figures_of(v).map(|f| figures_json(&f)),
        "compare-figures" => {
            let data = dataset_of(&v["dataset"])?;
            let bt = data.get(text(v, "industry")?).map_err(failure)?;
            let figures = figures_of(v)?;
            let supplied = strings(v, "supplied")?;
            let mut comparison =
                report::compare(&data, bt, &figures, supplied.as_deref()).map_err(failure)?;
            if let Some(notes) = strings(v, "routing_notes")? {
                comparison.note_details.extend(
                    notes
                        .into_iter()
                        .map(|n| crate::ratios::EvidenceMessage::new("routing", n, &[])),
                );
            }
            // Each serialiser's own default when absent; the value as given otherwise.
            // As `json.loads` made it. JSON outputs write it back with Python's
            // float spelling; the text report uses the value itself.
            let unreviewed = v.get("unreviewed").map(Raw::from_value);
            let unreviewed_json = unreviewed.as_ref().map(report::raw_json);
            let output_supplied = strings(v, "output_supplied")?;
            let evidenced_output = matches!(
                optional_text(v, "output")?,
                Some("evidenced" | "evidenced_ratio")
            );
            if evidenced_output && matches!(v.get("output_supplied"), Some(Value::Null)) {
                // `frozenset(None)` inside the evidenced serialisers.
                return Err("TypeError: 'NoneType' object is not iterable".into());
            }
            match optional_text(v, "output")?.unwrap_or("comparison") {
                "comparison" => Ok(comparison_json(&comparison)),
                "to_dict" => report::to_dict(&comparison, unreviewed_json.unwrap_or(json!(0)))
                    .map(map_value)
                    .map_err(failure),
                "evidenced" => report::to_evidenced_dict(
                    &comparison,
                    &output_supplied.unwrap_or_default(),
                    unreviewed_json.unwrap_or(Value::Null),
                )
                .map(map_value)
                .map_err(failure),
                "text" => report::render_text(
                    &comparison,
                    &unreviewed.unwrap_or(Raw::Int("0".into())),
                    output_supplied.as_deref(),
                )
                .map(|t| json!(t))
                .map_err(failure),
                "evidenced_ratio" => report::evidenced_ratio(
                    &comparison,
                    &output_supplied.unwrap_or_default(),
                    text(v, "ratio")?,
                )
                .map(|b| json!(b))
                .map_err(failure),
                other => Err(format!("unknown output {other}")),
            }
        }
        _ => Err(format!("unknown command {command}")),
    }
}
