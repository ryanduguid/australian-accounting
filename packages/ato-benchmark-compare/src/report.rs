//! Compare computed ratios against a benchmark and render the result.

use serde_json::{Map, Value, json};

use crate::Failure;
use crate::dataset::{Band, BusinessType, Dataset, Range, ratio_label};
use crate::decimal::Decimal;
use crate::mapping::{BUCKETS, EXPENSE_BUCKETS, is_bucket};
use crate::money::{money, percent, percent_range};
use crate::ratios::{EvidenceMessage, Figures};
use crate::raw::Raw;

pub const WITHIN: &str = "within";
pub const BELOW: &str = "below";
pub const ABOVE: &str = "above";
pub const NO_BENCHMARK: &str = "no benchmark in this dataset";
pub const NO_BAND: &str = "no turnover band applies";
pub const NOT_SUPPLIED: &str = "not_supplied";
pub const SOURCE_DATASET: &str = "ato_dataset";
pub const SOURCE_INDUSTRY_PAGE: &str = "ato_industry_page";

pub const DISCLAIMER: &str = "The ATO publishes these benchmarks as ranges, and sitting outside \
a range is not of itself a finding that anything is wrong. This tool reports a comparison. It is \
not tax advice and it does not decide how any amount should be reported.";

pub const RATIO_ORDER: [&str; 5] = [
    "cost_of_sales_to_turnover",
    "total_expenses_to_turnover",
    "labour_to_turnover",
    "rent_to_turnover",
    "motor_vehicle_to_turnover",
];

const OTHER_INCOME_NOTE: &str = "other_business_income was omitted. The ATO rule divides by \
sales, or by total business income once other income exceeds sales, so every ratio is \
not_supplied until that figure is established. The turnover figure, the basis that selected it, \
the turnover band, the published ranges and any note quoting that turnover are withheld for the \
same reason. Pass 0 only when the operator established the business had no other income.";

/// Every bucket but `excluded`, in `BUCKETS` order.
fn calculation_fields() -> Vec<&'static str> {
    BUCKETS
        .iter()
        .map(|(b, _)| *b)
        .filter(|b| *b != "excluded")
        .collect()
}

fn amount_failure(e: crate::money::AmountError) -> Failure {
    Failure::Value(e.0)
}

#[derive(Clone, Debug)]
pub struct Verdict {
    pub key: String,
    pub label: String,
    pub ratio: Decimal,
    pub benchmark: Option<Range>,
    pub status: &'static str,
    pub is_key: bool,
    pub benchmark_source: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct Comparison {
    pub dataset: Dataset,
    pub business_type: BusinessType,
    pub figures: Figures,
    pub band: Option<Band>,
    pub verdicts: Vec<Verdict>,
    pub key_ratio: String,
    pub notes: Vec<String>,
    pub note_details: Vec<EvidenceMessage>,
}

impl Comparison {
    pub fn key_verdict(&self) -> Option<&Verdict> {
        self.verdicts.iter().find(|v| v.is_key)
    }

    pub fn outside_key_range(&self) -> bool {
        self.key_verdict()
            .is_some_and(|v| v.status == BELOW || v.status == ABOVE)
    }
}

/// Refuse a supplied field that is neither a bucket nor `w1`.
pub fn known_fields(supplied: &[String]) -> Result<Vec<String>, Failure> {
    let mut known: Vec<String> = Vec::new();
    for f in supplied {
        if !known.contains(f) {
            known.push(f.clone());
        }
    }
    let mut unknown: Vec<&String> = known
        .iter()
        .filter(|f| !is_bucket(f) && *f != "w1")
        .collect();
    if !unknown.is_empty() {
        unknown.sort();
        return Err(Failure::Uncaught(format!(
            "ValueError: unknown supplied field(s): {}",
            unknown
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    Ok(known)
}

fn has(set: &[String], name: &str) -> bool {
    set.iter().any(|s| s == name)
}

/// Whether income, the expense set and labour are evidenced.
fn evidence_flags(known: &[String]) -> (bool, bool, bool) {
    let supplied: Vec<String> = known.iter().filter(|f| *f != "w1").cloned().collect();
    (
        has(&supplied, "turnover") && has(&supplied, "other_income"),
        EXPENSE_BUCKETS.iter().all(|b| has(&supplied, b)),
        [
            "salary_wages",
            "contractor_commission",
            "cost_of_sales_labour",
        ]
        .iter()
        .all(|b| has(&supplied, b))
            && (!has(known, "w1") || has(&supplied, "associated_persons")),
    )
}

fn own_buckets(name: &str) -> Vec<&'static str> {
    match name {
        "total_expenses_to_turnover" => EXPENSE_BUCKETS.to_vec(),
        "cost_of_sales_to_turnover" => vec!["cost_of_sales"],
        "rent_to_turnover" => vec!["rent"],
        "motor_vehicle_to_turnover" => vec!["motor_vehicle"],
        _ => vec![],
    }
}

/// Which ratios the supplied set can state, in `RATIO_ORDER`.
fn evidenced_ratios(known: &[String]) -> Vec<(&'static str, bool)> {
    let (income, _, labour) = evidence_flags(known);
    RATIO_ORDER
        .iter()
        .map(|name| {
            let own = if *name == "labour_to_turnover" {
                labour
            } else {
                own_buckets(name)
                    .iter()
                    .all(|b| known.iter().any(|k| k == b && k != "w1"))
            };
            (*name, own && income)
        })
        .collect()
}

struct Presence {
    income_evidenced: bool,
    expense_complete: bool,
    labour_evidenced: bool,
    key_ratio: String,
    evidenced: Vec<(&'static str, bool)>,
}

impl Presence {
    fn evidenced(&self, name: &str) -> bool {
        self.evidenced
            .iter()
            .find(|(k, _)| *k == name)
            .is_some_and(|(_, v)| *v)
    }
}

fn presence(comparison: &Comparison, supplied: &[String]) -> Result<Presence, Failure> {
    let known = known_fields(supplied)?;
    let (income, expense, labour) = evidence_flags(&known);
    let key_ratio = if has(&known, "cost_of_sales") {
        comparison.key_ratio.clone()
    } else {
        comparison.business_type.key_ratio.clone()
    };
    Ok(Presence {
        income_evidenced: income,
        expense_complete: expense,
        labour_evidenced: labour,
        key_ratio,
        evidenced: evidenced_ratios(&known),
    })
}

pub fn omitted_buckets_note(omitted: &[&str]) -> String {
    format!(
        "These buckets were omitted, not evidenced as zero, so their ratios are not_supplied: {}.",
        omitted.join(", ")
    )
}

/// Whether one ratio is evidenced under the same gate as the outputs.
pub fn evidenced_ratio(
    comparison: &Comparison,
    supplied: &[String],
    ratio: &str,
) -> Result<bool, Failure> {
    Ok(presence(comparison, supplied)?.evidenced(ratio))
}

/// Compare the computed ratios against the dataset.
pub fn compare(
    dataset: &Dataset,
    business_type: &BusinessType,
    figures: &Figures,
    supplied: Option<&[String]>,
) -> Result<Comparison, Failure> {
    let evidenced = evidenced_ratios(&known_fields(supplied.unwrap_or(&figures.supplied_fields))?);
    let mut notes = Vec::new();
    let mut details = Vec::new();
    let mut add_note = |code: &str, text: String, fields: &[&str]| {
        notes.push(text.clone());
        details.push(EvidenceMessage::new(code, text, fields));
    };
    let band = business_type.band_for(&figures.turnover)?.cloned();
    if band.is_none() {
        let lowest = &business_type.bands[0];
        let shown = money(&figures.turnover).map_err(amount_failure)?;
        if figures.turnover.lt(&lowest.turnover_from)? {
            add_note(
                "turnover_below_range",
                format!(
                    "Turnover of ${shown} is below the lowest published range for this industry \
                     ({}), so the ATO benchmarks do not apply.",
                    lowest.label
                ),
                &["turnover", "other_income"],
            );
        } else {
            add_note(
                "turnover_outside_ranges",
                format!(
                    "Turnover of ${shown} does not fall in any published range for this industry."
                ),
                &["turnover", "other_income"],
            );
        }
    }
    let mut key_ratio = business_type.key_ratio.clone();
    if key_ratio == "cost_of_sales_to_turnover" && figures.cost_of_sales_for_ratio.is_zero() {
        key_ratio = "total_expenses_to_turnover".into();
        add_note(
            "cost_of_sales_key_fallback",
            "Cost of sales to turnover is the ATO key range for this industry, but no cost of \
             sales was mapped. The ATO says to use total expenses to turnover instead where cost \
             of sales is not reported, so that is the key range used here."
                .into(),
            &["cost_of_sales"],
        );
    } else if key_ratio == "cost_of_sales_to_turnover" {
        add_note(
            "cost_of_sales_small_judgement",
            "The ATO says to use total expenses to turnover as the key range instead where cost \
             of sales is only a small amount. It does not put a figure on small, so both ranges \
             are reported and that judgement is yours."
                .into(),
            &["cost_of_sales"],
        );
    }
    let mut verdicts = Vec::new();
    for name in RATIO_ORDER {
        let Some(ratio) = figures.ratio(name) else {
            continue;
        };
        let benchmark = band.as_ref().and_then(|b| b.ratio(name)).cloned();
        let is_evidenced = evidenced
            .iter()
            .find(|(k, _)| *k == name)
            .is_some_and(|(_, v)| *v);
        let status = if !is_evidenced {
            NOT_SUPPLIED
        } else if band.is_none() {
            NO_BAND
        } else if let Some(range) = &benchmark {
            if range.contains(ratio)? {
                WITHIN
            } else if ratio.lt(&range.minimum)? {
                BELOW
            } else {
                ABOVE
            }
        } else {
            NO_BENCHMARK
        };
        let source = benchmark.as_ref().map(|_| {
            if band.as_ref().is_some_and(|b| b.is_page_ratio(name)) {
                SOURCE_INDUSTRY_PAGE
            } else {
                SOURCE_DATASET
            }
        });
        verdicts.push(Verdict {
            key: name.into(),
            label: ratio_label(name).into(),
            ratio: ratio.clone(),
            benchmark,
            status,
            is_key: name == key_ratio,
            benchmark_source: source,
        });
    }
    let compared = |v: &Verdict| [WITHIN, BELOW, ABOVE].contains(&v.status);
    let page_used: Vec<String> = verdicts
        .iter()
        .filter(|v| band.as_ref().is_some_and(|b| b.is_page_ratio(&v.key)) && compared(v))
        .map(|v| v.label.to_lowercase())
        .collect();
    if !page_used.is_empty() {
        let plural = page_used.len() > 1;
        let listed = if plural {
            format!(
                "{} and {}",
                page_used[..page_used.len() - 1].join(", "),
                page_used.last().unwrap()
            )
        } else {
            page_used[0].clone()
        };
        add_note(
            "industry_page_ranges",
            format!(
                "The {listed} range{} come{} from the ATO's page for this industry, not the \
                 data.gov.au dataset. The ATO says not every business reports these expenses and \
                 to use these ranges only as a guide where they apply.",
                if plural { "s" } else { "" },
                if plural { "" } else { "s" }
            ),
            &["turnover", "other_income"],
        );
    }
    let single: Vec<String> = verdicts
        .iter()
        .filter(|v| {
            v.benchmark_source == Some(SOURCE_INDUSTRY_PAGE)
                && v.benchmark
                    .as_ref()
                    .is_some_and(|r| r.minimum.equals(&r.maximum).unwrap_or(false))
                && compared(v)
        })
        .map(|v| v.label.to_lowercase())
        .collect();
    if !single.is_empty() {
        add_note(
            "industry_page_single_figure",
            format!(
                "The ATO printed a single figure, not a range, for {}. The tool compares against \
                 that figure and does not infer wider bounds.",
                single.join(" and ")
            ),
            &["turnover", "other_income"],
        );
    }
    Ok(Comparison {
        dataset: dataset.clone(),
        business_type: business_type.clone(),
        figures: figures.clone(),
        band,
        verdicts,
        key_ratio,
        notes,
        note_details: details,
    })
}

/// The industry page behind any range in the selected band, or None.
pub fn industry_page_source(comparison: &Comparison) -> Value {
    let (Some(band), Some(page), Some(source)) = (
        comparison.band.as_ref(),
        comparison.business_type.page.as_ref(),
        comparison.dataset.page_source.as_ref(),
    ) else {
        return Value::Null;
    };
    if band.page_ratios.is_empty() {
        return Value::Null;
    }
    let mut out = Map::new();
    for (key, value) in page {
        out.insert(key.clone(), raw_json(value));
    }
    for key in ["retrieved", "licence", "licence_url", "caution"] {
        out.insert(key.into(), source.get(key).map_or(Value::Null, raw_json));
    }
    out.insert(
        "ratios".into(),
        json!(
            RATIO_ORDER
                .iter()
                .filter(|k| band.is_page_ratio(k))
                .collect::<Vec<_>>()
        ),
    );
    Value::Object(out)
}

/// A JSON value from the dataset, as `json.dumps` writes it back.
pub fn raw_json(value: &Raw) -> Value {
    match value {
        Raw::None => Value::Null,
        Raw::Bool(b) => json!(b),
        Raw::Int(digits) => serde_json::from_str(digits).unwrap_or(Value::Null),
        // `json.dumps` writes repr(float); JSON has no spelling for NaN or
        // infinity, which come back as null.
        Raw::Float(f) if f.is_finite() => Value::Number(
            crate::raw::float_repr(*f)
                .parse()
                .expect("a finite float's repr is a JSON number"),
        ),
        Raw::Float(_) => Value::Null,
        Raw::Str(s) => json!(s),
        Raw::List(items) => Value::Array(items.iter().map(raw_json).collect()),
        Raw::Dict(items) => Value::Object(
            items
                .iter()
                .map(|(k, v)| (k.clone(), raw_json(v)))
                .collect(),
        ),
    }
}

fn source_text(source: &Raw, key: &str) -> String {
    source.get(key).map_or("None".into(), Raw::py_str)
}

/// Render the comparison as text; with `supplied`, withhold what nobody supplied.
pub fn render_text(
    comparison: &Comparison,
    unreviewed: &Raw,
    supplied: Option<&[String]>,
) -> Result<String, Failure> {
    let figures = &comparison.figures;
    let source = &comparison.dataset.source;
    let presence = supplied.map(|s| presence(comparison, s)).transpose()?;
    let supplied_names: Option<Vec<String>> =
        supplied.map(|s| s.iter().filter(|f| *f != "w1").cloned().collect());
    let mut lines: Vec<String> = Vec::new();
    let money_text = |v: &Decimal| money(v).map_err(amount_failure);
    lines.push("ATO small business benchmark comparison".into());
    lines.push("=".repeat(39));
    lines.push(format!("Business type:  {}", comparison.business_type.name));
    lines.push(format!("Benchmark year: {}", comparison.dataset.year));
    if presence.as_ref().is_none_or(|p| p.income_evidenced) {
        lines.push(format!(
            "Turnover:       ${} ({})",
            money_text(&figures.turnover)?,
            figures.turnover_basis
        ));
        lines.push(format!(
            "Turnover band:  {}",
            comparison
                .band
                .as_ref()
                .map_or("none applies".to_string(), |b| b.label.clone())
        ));
    } else {
        lines.push("Turnover:       not supplied (other business income was not evidenced)".into());
        lines.push("Turnover band:  withheld until turnover is established".into());
    }
    lines.push(String::new());
    let width = comparison
        .verdicts
        .iter()
        .map(|v| v.label.chars().count())
        .max()
        .unwrap_or(0)
        + 6;
    let pad = |text: &str, width: usize| format!("{text:<width$}");
    lines.push(format!(
        "{}{}{}Result",
        pad("Ratio", width),
        pad("This business", 15),
        pad("ATO range", 18)
    ));
    lines.push("-".repeat(width + 15 + 18 + 8));
    for verdict in &comparison.verdicts {
        let evidenced = presence.as_ref().is_none_or(|p| p.evidenced(&verdict.key));
        let is_key = presence
            .as_ref()
            .map_or(verdict.is_key, |p| p.key_ratio == verdict.key);
        let label = format!("{}{}", verdict.label, if is_key { " (key)" } else { "" });
        let show_range = presence.as_ref().is_none_or(|p| p.income_evidenced);
        let benchmark = match (&verdict.benchmark, show_range) {
            (Some(r), true) => percent_range(&r.minimum, &r.maximum).map_err(amount_failure)?,
            _ => "-".into(),
        };
        let this_business = if evidenced {
            percent(&verdict.ratio).map_err(amount_failure)?
        } else {
            "not supplied".into()
        };
        let status = if evidenced {
            verdict.status
        } else {
            NOT_SUPPLIED
        };
        lines.push(format!(
            "{}{}{}{status}",
            pad(&label, width),
            pad(&this_business, 15),
            pad(&benchmark, 18)
        ));
    }
    lines.push(String::new());
    let figure = |value: &Decimal, evidenced: bool| -> Result<String, Failure> {
        Ok(if evidenced {
            format!("${}", money_text(value)?)
        } else {
            "not supplied".into()
        })
    };
    let named =
        |name: &str| presence.is_none() || supplied_names.as_ref().is_some_and(|s| has(s, name));
    let income = presence.as_ref().is_none_or(|p| p.income_evidenced);
    let expense = presence.as_ref().is_none_or(|p| p.expense_complete);
    let labour = presence.as_ref().is_none_or(|p| p.labour_evidenced);
    lines.push("Figures used".into());
    lines.push(format!(
        "  Sales of goods and services   {}",
        figure(&figures.trading_sales, named("turnover"))?
    ));
    lines.push(format!(
        "  Other business income         {}",
        figure(&figures.other_income, income)?
    ));
    lines.push(format!(
        "  Total business income         {}",
        figure(&figures.total_business_income, income)?
    ));
    lines.push(format!(
        "  Total expenses                {}",
        figure(&figures.total_expenses_reported, expense)?
    ));
    lines.push(format!(
        "  Less payments to associates   {}",
        figure(
            figures.amount("associated_persons"),
            named("associated_persons")
        )?
    ));
    lines.push(format!(
        "  Total expenses for the ratio  {}",
        figure(&figures.total_expenses_for_ratio, expense)?
    ));
    lines.push(format!(
        "  Cost of sales excluding wages {}",
        figure(&figures.cost_of_sales_for_ratio, named("cost_of_sales"))?
    ));
    lines.push(format!(
        "  Labour                        {}",
        figure(&figures.labour, labour)?
    ));
    lines.push(String::new());
    if unreviewed.truthy() {
        lines.push("Review outstanding".into());
        lines.push(format!(
            "  {} account(s) still carry the bucket this tool suggested. Suggestions \
             are made from account names alone and are not a substitute for reading the ledger.",
            unreviewed.py_str()
        ));
        lines.push(String::new());
    }
    let (rendered_notes, checks) = match supplied {
        Some(supplied) => {
            let known: Vec<String> = supplied.to_vec();
            let names: Vec<String> = known.iter().filter(|f| *f != "w1").cloned().collect();
            let mut notes: Vec<String> = comparison
                .note_details
                .iter()
                .filter(|d| d.stated_by(&known))
                .map(|d| d.text.clone())
                .collect();
            let checks: Vec<String> = figures
                .warning_details
                .iter()
                .filter(|d| d.stated_by(&known))
                .map(|d| d.text.clone())
                .collect();
            let withheld = figures.warning_details.len() - checks.len();
            let omitted: Vec<&str> = calculation_fields()
                .into_iter()
                .filter(|f| !has(&names, f))
                .collect();
            if !omitted.is_empty() {
                notes.push(omitted_buckets_note(&omitted));
            }
            if !has(&names, "other_income") {
                notes.push(OTHER_INCOME_NOTE.into());
            }
            if withheld > 0 {
                notes.push(format!(
                    "{withheld} check(s) to make were withheld because one or more figures \
                     needed to state them were omitted rather than evidenced as zero."
                ));
            }
            (notes, checks)
        }
        None => (comparison.notes.clone(), figures.warnings.clone()),
    };
    if !rendered_notes.is_empty() {
        lines.push("Notes".into());
        lines.extend(rendered_notes.iter().map(|n| format!("  - {n}")));
        lines.push(String::new());
    }
    if !checks.is_empty() {
        lines.push("Checks to make".into());
        lines.extend(checks.iter().map(|c| format!("  - {c}")));
        lines.push(String::new());
    }
    lines.push("Source".into());
    lines.push(format!(
        "  {}, {}, {}",
        source_text(source, "publisher"),
        source_text(source, "dataset"),
        source_text(source, "resource_name")
    ));
    lines.push(format!("  {}", source_text(source, "resource_url")));
    lines.push(format!(
        "  Retrieved {}, sha256 {}",
        source_text(source, "retrieved"),
        source_text(source, "sha256")
    ));
    lines.push(format!(
        "  Licensed {} ({})",
        source_text(source, "licence"),
        source_text(source, "licence_url")
    ));
    let page = industry_page_source(comparison);
    if !page.is_null() && income {
        // An f-string prints `str()` of each value, whatever its type, read
        // from the loaded values rather than their JSON form.
        let page_raw = comparison.business_type.page.as_ref();
        let source_raw = comparison.dataset.page_source.as_ref();
        // `{**page, "retrieved": source.get("retrieved"), ...}`: those four
        // keys come from the page source, the rest from the page.
        let text = |key: &str| {
            let value = if ["retrieved", "licence", "licence_url", "caution"].contains(&key) {
                source_raw.and_then(|r| r.get(key))
            } else {
                page_raw
                    .and_then(|items| items.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v))
            };
            value.map_or("None".into(), Raw::py_str)
        };
        let labels: Vec<&str> = page["ratios"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| ratio_label(k.as_str().unwrap()))
            .collect();
        lines.push(format!("  {}: ATO industry page", labels.join(", ")));
        lines.push(format!("  {}", text("page_url")));
        lines.push(format!(
            "  Page last updated {}, {}, retrieved {}, text sha256 {}",
            text("page_last_updated"),
            text("page_reference"),
            text("retrieved"),
            text("text_sha256")
        ));
        lines.push(format!(
            "  Reused under the ATO copyright notice ({})",
            text("licence_url")
        ));
    }
    lines.push(String::new());
    lines.push(DISCLAIMER.into());
    Ok(lines.join("\n"))
}

fn text_of(value: &Decimal) -> Value {
    json!(value.to_sci_string())
}

pub fn to_dict(comparison: &Comparison, unreviewed: Value) -> Result<Map<String, Value>, Failure> {
    let figures = &comparison.figures;
    let mut ratios = Vec::new();
    for v in &comparison.verdicts {
        ratios.push(json!({
            "ratio": v.key,
            "label": v.label,
            "value": text_of(&v.ratio),
            "percent": percent(&v.ratio).map_err(amount_failure)?,
            "benchmark_min": v.benchmark.as_ref().map(|r| r.minimum.to_sci_string()),
            "benchmark_max": v.benchmark.as_ref().map(|r| r.maximum.to_sci_string()),
            "benchmark_source": v.benchmark_source,
            "status": v.status,
            "is_key_ratio": v.is_key,
        }));
    }
    let mut totals: Vec<&(String, Decimal)> = figures.totals.iter().collect();
    totals.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = Map::new();
    out.insert("benchmark_year".into(), json!(comparison.dataset.year));
    out.insert("business_type".into(), json!(comparison.business_type.name));
    out.insert("key_ratio".into(), json!(comparison.key_ratio));
    out.insert("turnover".into(), text_of(&figures.turnover));
    out.insert("turnover_basis".into(), json!(figures.turnover_basis));
    out.insert(
        "turnover_band".into(),
        comparison
            .band
            .as_ref()
            .map_or(Value::Null, |b| json!({"band": b.band, "label": b.label})),
    );
    out.insert(
        "figures".into(),
        json!({
            "sales_of_goods_and_services": text_of(&figures.trading_sales),
            "other_business_income": text_of(&figures.other_income),
            "total_business_income": text_of(&figures.total_business_income),
            "total_expenses": text_of(&figures.total_expenses_reported),
            "payments_to_associated_persons": text_of(figures.amount("associated_persons")),
            "total_expenses_for_ratio": text_of(&figures.total_expenses_for_ratio),
            "cost_of_sales_for_ratio": text_of(&figures.cost_of_sales_for_ratio),
            "labour": text_of(&figures.labour),
        }),
    );
    out.insert(
        "bucket_totals".into(),
        Value::Object(
            totals
                .into_iter()
                .map(|(k, v)| (k.clone(), text_of(v)))
                .collect(),
        ),
    );
    out.insert("ratios".into(), Value::Array(ratios));
    out.insert("unreviewed_accounts".into(), unreviewed);
    out.insert("notes".into(), json!(comparison.notes));
    out.insert("checks_to_make".into(), json!(figures.warnings));
    out.insert("source".into(), raw_json(&comparison.dataset.source));
    out.insert(
        "industry_page_source".into(),
        industry_page_source(comparison),
    );
    out.insert("disclaimer".into(), json!(DISCLAIMER));
    Ok(out)
}

/// Serialise a comparison without presenting unsupplied amounts as zero.
pub fn to_evidenced_dict(
    comparison: &Comparison,
    supplied_fields: &[String],
    unreviewed: Value,
) -> Result<Map<String, Value>, Failure> {
    let presence = presence(comparison, supplied_fields)?;
    // `to_dict(comparison, unreviewed=unreviewed or 0)`, then the value as given.
    let counted = if Raw::from_value(&unreviewed).truthy() {
        unreviewed.clone()
    } else {
        json!(0)
    };
    let mut payload = to_dict(comparison, counted)?;
    payload.insert("unreviewed_accounts".into(), unreviewed);
    let figures = &comparison.figures;
    let supplied: Vec<String> = supplied_fields
        .iter()
        .filter(|f| *f != "w1")
        .cloned()
        .collect();
    let key_ratio = presence.key_ratio.clone();
    let mut ratios = Vec::new();
    for row in payload["ratios"].as_array().unwrap() {
        let name = row["ratio"].as_str().unwrap();
        let is_key = name == key_ratio;
        if presence.evidenced(name) {
            let mut row = row.clone();
            row["is_key_ratio"] = json!(is_key);
            ratios.push(row);
            continue;
        }
        let income = presence.income_evidenced;
        ratios.push(json!({
            "ratio": row["ratio"],
            "label": row["label"],
            "value": null,
            "percent": null,
            "benchmark_min": if income { row["benchmark_min"].clone() } else { Value::Null },
            "benchmark_max": if income { row["benchmark_max"].clone() } else { Value::Null },
            "benchmark_source": if income { row["benchmark_source"].clone() } else { Value::Null },
            "status": NOT_SUPPLIED,
            "is_key_ratio": is_key,
        }));
    }
    let totals = payload["bucket_totals"].as_object_mut().unwrap();
    for name in calculation_fields() {
        if !has(&supplied, name) {
            totals.insert(name.into(), Value::Null);
        }
    }
    if !has(&supplied, "excluded") {
        totals.insert("excluded".into(), Value::Null);
    }
    let figs = payload["figures"].as_object_mut().unwrap();
    if !has(&supplied, "turnover") {
        figs.insert("sales_of_goods_and_services".into(), Value::Null);
    }
    if !presence.income_evidenced {
        figs.insert("other_business_income".into(), Value::Null);
        figs.insert("total_business_income".into(), Value::Null);
    }
    if !presence.expense_complete {
        figs.insert("total_expenses".into(), Value::Null);
        figs.insert("total_expenses_for_ratio".into(), Value::Null);
    }
    if !has(&supplied, "associated_persons") {
        figs.insert("payments_to_associated_persons".into(), Value::Null);
    }
    if !has(&supplied, "cost_of_sales") {
        figs.insert("cost_of_sales_for_ratio".into(), Value::Null);
    }
    if !presence.labour_evidenced {
        figs.insert("labour".into(), Value::Null);
    }
    if !presence.income_evidenced {
        for key in [
            "turnover",
            "turnover_basis",
            "turnover_band",
            "industry_page_source",
        ] {
            payload.insert(key.into(), Value::Null);
        }
    }
    payload.insert("key_ratio".into(), json!(key_ratio));
    let known: Vec<String> = supplied_fields.to_vec();
    let mut notes: Vec<String> = comparison
        .note_details
        .iter()
        .filter(|d| d.stated_by(&known))
        .map(|d| d.text.clone())
        .collect();
    let checks: Vec<String> = figures
        .warning_details
        .iter()
        .filter(|d| d.stated_by(&known))
        .map(|d| d.text.clone())
        .collect();
    let withheld = figures.warning_details.len() - checks.len();
    let mut omitted: Vec<String> = calculation_fields()
        .into_iter()
        .filter(|f| !has(&supplied, f))
        .map(String::from)
        .collect();
    if !omitted.is_empty() {
        notes.push(omitted_buckets_note(
            &omitted.iter().map(String::as_str).collect::<Vec<_>>(),
        ));
    }
    if !has(&known, "w1") {
        omitted.push("w1".into());
    }
    if !has(&supplied, "other_income") {
        notes.push(OTHER_INCOME_NOTE.into());
    }
    if withheld > 0 {
        notes.push(format!(
            "{withheld} check(s) to make were withheld because one or more figures needed to \
             state them were omitted rather than evidenced as zero."
        ));
    }
    let mut sorted_supplied = supplied.clone();
    sorted_supplied.sort();
    sorted_supplied.dedup();
    payload.insert("ratios".into(), Value::Array(ratios));
    payload.insert("notes".into(), json!(notes));
    payload.insert("checks_to_make".into(), json!(checks));
    payload.insert("supplied_buckets".into(), json!(sorted_supplied));
    payload.insert("omitted_buckets".into(), json!(omitted));
    payload.insert("complete_buckets".into(), json!(presence.expense_complete));
    Ok(payload)
}
