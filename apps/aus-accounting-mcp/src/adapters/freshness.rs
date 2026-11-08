// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! What each bundled dataset records about its period, version and sources.
//! Every value is read from the linked engines; a fact an engine does not record
//! is reported as UNKNOWN, and nothing here says that a figure is current.
use ato_benchmark_compare::{
    dataset::{self, DataDir},
    report,
};
use chrono::NaiveDate;
use payday_super_checker::{calendar as payday_calendar, rates as payday_rates};
use serde_json::{Map, Value, json};

const UNKNOWN: &str = "UNKNOWN";
const PAYDAY_TOOLS: [&str; 3] = [
    "calc_payday_super_deadline",
    "review_payday_super_contributions",
    "build_payday_super_evidence_pack",
];
const STATEMENT: &str = "These are the periods, versions and source dates recorded in the installed release. They do not establish that any figure is current. uvx and other package caches can keep serving an older release, and official figures can change after a source was checked. This tool performs no lookup and cannot tell whether a newer release or newer official figures exist. UNKNOWN means the installed engine records no such fact. An as_at_status of INSIDE_RECORDED_PERIOD means the release holds data for that date, not that the data is current or unchanged. Verify every mutable fact against current official sources before relying on a result.";
const NOT_LISTED: &str = "The library, legislation corpus and rulings tools read folders the operator configures; their rows carry their own capture dates and are not listed here. generate_synthetic_sbr_fixture uses no dated data.";

fn iso_or_unknown(value: &Value) -> Value {
    value
        .as_str()
        .and_then(|text| NaiveDate::parse_from_str(text, "%Y-%m-%d").ok())
        .map_or(json!(UNKNOWN), |day| json!(day.to_string()))
}

fn dates<'a>(values: impl IntoIterator<Item = &'a Value>) -> Value {
    let mut known: Vec<String> = values
        .into_iter()
        .filter_map(|value| iso_or_unknown(value).as_str().map(str::to_owned))
        .filter(|day| day != UNKNOWN)
        .collect();
    known.sort();
    known.dedup();
    if known.is_empty() {
        json!(UNKNOWN)
    } else {
        json!(known)
    }
}

fn urls<'a>(values: impl IntoIterator<Item = &'a Value>) -> Value {
    let mut known: Vec<&str> = vec![];
    for url in values.into_iter().filter_map(Value::as_str) {
        if url.starts_with("https://") && !known.contains(&url) {
            known.push(url);
        }
    }
    if known.is_empty() {
        json!(UNKNOWN)
    } else {
        json!(known)
    }
}

fn financial_year(label: &str, basis: &str) -> Result<Value, String> {
    let start = label
        .get(..4)
        .and_then(|year| year.parse::<i32>().ok())
        .filter(|year| label.len() == 7 && *label == format!("{year}-{:02}", (year + 1) % 100))
        .ok_or_else(|| format!("not a financial year label: {label:?}"))?;
    Ok(
        json!({"id":label,"basis":basis,"start":format!("{start:04}-07-01"),"end":format!("{:04}-06-30",start+1)}),
    )
}

/// The fixed key order of one dataset row; `facts` is
/// (effective_periods, sources, retrieved, source_checked, provenance).
fn row(
    dataset: String,
    description: String,
    tools: &[&str],
    engine: (&str, &str),
    facts: (Value, Value, Value, Value, Value),
) -> Value {
    json!({"dataset":dataset,"description":description,"tools":tools,"engine":engine.0,
        "engine_version":engine.1,"effective_periods":facts.0,"sources":facts.1,"retrieved":facts.2,
        "source_checked":facts.3,"provenance":facts.4})
}

fn benchmarks() -> Result<Vec<Value>, String> {
    dataset::available_years(&DataDir::Shipped)
        .iter()
        .map(|year| {
            let data = dataset::load(Some(year), &DataDir::Shipped).map_err(|e| e.to_string())?;
            let source = report::raw_json(&data.source);
            Ok(row(
                format!("ato_small_business_benchmarks:{year}"),
                format!("ATO small business benchmarks for {year}."),
                &["list_ato_benchmark_industries", "get_ato_benchmarks"],
                ("ato-benchmark-compare", ato_benchmark_compare::VERSION),
                (
                    json!([financial_year(year, "benchmark financial year")?]),
                    urls([&source["resource_url"], &source["dataset_page"]]),
                    dates([&source["retrieved"]]),
                    json!(UNKNOWN),
                    source.clone(),
                ),
            ))
        })
        .collect()
}

fn div7a() -> Result<Value, String> {
    use div7a_loan_review as engine;
    let table = engine::rates::frozen_table();
    let entries: Vec<_> = table.entries.values().collect();
    let links: Vec<Value> = entries
        .iter()
        .flat_map(|entry| [json!(entry.primary_url), json!(entry.verify_at)])
        .collect();
    let retrieved: Vec<Value> = entries.iter().map(|e| json!(e.retrieved_on)).collect();
    let periods = entries
        .iter()
        .map(|entry| financial_year(&entry.year.label(), "year of income"))
        .collect::<Result<Vec<_>, _>>()?;
    let tables: Vec<Value> = table
        .sources
        .iter()
        .map(|source| source.to_json())
        .collect();
    Ok(row(
        "div7a_benchmark_rates".into(),
        "Division 7A s 109N(2) benchmark interest rates by year of income.".into(),
        &["get_div7a_benchmark_rate", "review_div7a_loan"],
        ("div7a-loan-review", engine::VERSION),
        (
            json!(periods),
            urls(&links),
            dates(&retrieved),
            iso_or_unknown(&json!(table.reviewed_on)),
            json!({"reviewed_until":table.reviewed_until.label(),"reviewed_on":table.reviewed_on,
                "law_content_date":engine::LAW_CONTENT_DATE,"tables":tables}),
        ),
    ))
}

fn document(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

fn gic() -> Result<Value, String> {
    let table = payday_rates::load_gic().map_err(|e| e.message())?;
    let doc = document(payday_rates::GIC_RATES)?;
    let quarters = doc["quarters"].as_array().ok_or("gic quarters")?;
    let periods: Vec<Value> = quarters
        .iter()
        .map(|q| {
            let (from, to) = (
                q["from"].as_str().unwrap_or(""),
                q["to"].as_str().unwrap_or(""),
            );
            json!({"id":format!("{from}/{to}"),"basis":"GIC quarter","start":from,"end":to})
        })
        .collect();
    let source = table
        .source
        .as_ref()
        .map_or(json!(UNKNOWN), |s| json!({"uri":s.uri,"sha256":s.sha256}));
    Ok(row(
        "payday_gic_rates".into(),
        "General interest charge quarters used for Payday Super notional earnings.".into(),
        &PAYDAY_TOOLS,
        ("payday-super-checker", payday_super_checker::VERSION),
        (
            json!(periods),
            urls(quarters.iter().map(|q| &q["verify_at"])),
            dates(quarters.iter().map(|q| &q["seen"])),
            json!(UNKNOWN),
            json!({"summary":table.provenance(),"table":source,
                "law_content_date":payday_super_checker::LAW_CONTENT_DATE}),
        ),
    ))
}

fn calendar() -> Result<Value, String> {
    let calendar = payday_calendar::load_calendar(None).map_err(|e| e.message())?;
    let doc = document(payday_calendar::BUNDLED)?;
    let official = &doc["official_sources"];
    let links: Vec<&Value> = official
        .as_object()
        .map_or(vec![], |o| o.values().collect());
    Ok(row(
        "payday_business_day_calendar".into(),
        "Whole-of-state public holidays under SGAA 1992 s 6(1) for Payday Super deadlines.".into(),
        &PAYDAY_TOOLS,
        ("payday-super-checker", payday_super_checker::VERSION),
        (
            json!([{"id":"verified","basis":"calendar verification span",
                "start":calendar.verified_from.isoformat(),"end":calendar.verified_until.isoformat()}]),
            urls(links),
            json!(UNKNOWN),
            iso_or_unknown(&official["checked"]),
            json!({"coverage_until":calendar.coverage_until.isoformat(),
                "coverage_note":official.get("coverage_note").cloned().unwrap_or(json!(UNKNOWN)),
                "law_content_date":payday_super_checker::LAW_CONTENT_DATE}),
        ),
    ))
}

fn sg_parameters() -> Result<Value, String> {
    let doc = document(payday_rates::RATES)?;
    let years = doc["financial_years"]
        .as_object()
        .ok_or("financial_years")?;
    let mut labels: Vec<&String> = years.keys().collect();
    labels.sort();
    let periods = labels
        .iter()
        .map(|label| financial_year(label, "financial year"))
        .collect::<Result<Vec<_>, _>>()?;
    let sources: Map<String, Value> = years
        .iter()
        .map(|(label, entry)| {
            let source = entry.get("source").cloned().unwrap_or(json!(UNKNOWN));
            (label.clone(), source)
        })
        .collect();
    Ok(row(
        "payday_super_parameters".into(),
        "SG charge percentage, concessional cap and maximum contributions base by financial year."
            .into(),
        &PAYDAY_TOOLS,
        ("payday-super-checker", payday_super_checker::VERSION),
        (
            json!(periods),
            urls(
                years
                    .values()
                    .flat_map(|e| [&e["verify_at"], &e["cross_check"]]),
            ),
            dates(years.values().map(|e| &e["seen"])),
            json!(UNKNOWN),
            json!({"sources":sources,"law_content_date":payday_super_checker::LAW_CONTENT_DATE}),
        ),
    ))
}

fn worksheets() -> Result<Vec<Value>, String> {
    let catalogue = super::tax::advertised_worksheets()?;
    let kinds = catalogue.as_object().ok_or("worksheet catalogue")?;
    Ok(kinds
        .iter()
        .map(|(kind, entry)| {
            let periods: Vec<Value> = entry["supported_periods"]
                .as_array()
                .map_or(vec![], |periods| {
                    periods
                        .iter()
                        .map(|p| json!({"id":p["id"],"basis":p["basis"],"start":p["start"],"end":p["end"]}))
                        .collect()
                });
            let sources = match entry["sources"].as_array().filter(|s| !s.is_empty()) {
                Some(sources) => urls(sources),
                None => urls([&entry["source"]]),
            };
            row(
                format!("tax_worksheet:{kind}"),
                format!("Rates and thresholds for the {kind} worksheet."),
                &["calculate_tax_worksheet"],
                ("australian-tax-calculators", australian_tax_calculators::engine::VERSION),
                (
                    if periods.is_empty() { json!(UNKNOWN) } else { json!(periods) },
                    sources,
                    json!(UNKNOWN),
                    iso_or_unknown(&entry["source_checked"]),
                    json!({"source_passage":entry.get("source_passage").cloned().unwrap_or(json!(UNKNOWN))}),
                ),
            )
        })
        .collect())
}

fn lodgment() -> Value {
    let record = lodgment_calendar::provenance();
    let links: Vec<&Value> = record["sources"]
        .as_object()
        .map_or(vec![], |sources| sources.values().map(|s| &s[0]).collect());
    row(
        "lodgment_program".into(),
        format!(
            "ATO {} lodgment program due dates and its public holiday table.",
            record["program_year"].as_str().unwrap_or(UNKNOWN)
        ),
        &["get_lodgment_due_dates"],
        ("lodgment-calendar", lodgment_calendar::VERSION),
        (
            json!([{"id":record["program_year"],"basis":"lodgment program year and holiday table",
                "start":record["covers_from"],"end":record["covers_to"]}]),
            urls(links),
            json!(UNKNOWN),
            iso_or_unknown(&record["source_checked"]),
            json!({"activity_statement_year":record["activity_year"],
                "income_tax_return_year":record["return_year"],
                "holiday_table":record["holiday_provision"]}),
        ),
    )
}

fn gst() -> Value {
    let record = gst_review::provenance();
    row(
        "gst_review_rules".into(),
        "GST coding and BAS reconciliation rules.".into(),
        &["review_gst_coding", "reconcile_bas"],
        ("gst-review", gst_review::VERSION),
        (
            json!(UNKNOWN),
            urls(record["sources"].as_array().into_iter().flatten()),
            json!(UNKNOWN),
            iso_or_unknown(&record["source_checked"]),
            json!({}),
        ),
    )
}

fn as_at_status(periods: &Value, day: &str) -> &'static str {
    let Some(periods) = periods.as_array() else {
        return UNKNOWN;
    };
    // ISO dates of equal width order as the days they name.
    let inside = periods.iter().any(|period| {
        period["start"].as_str().is_some_and(|start| start <= day)
            && period["end"].as_str().is_some_and(|end| day <= end)
    });
    if inside {
        "INSIDE_RECORDED_PERIOD"
    } else {
        "OUTSIDE_RECORDED_PERIOD"
    }
}

pub fn report(input: &Value) -> Result<Value, String> {
    let (as_at, tool) = (input["as_at"].as_str(), input["tool"].as_str());
    if let Some(day) = as_at {
        super::valid_day(day, "as_at").map_err(|_| {
            format!(
                "as_at must be a YYYY-MM-DD date; got {}",
                div7a_loan_review::raw::Raw::text(day).py_repr()
            )
        })?;
    }
    let mut datasets = benchmarks()?;
    datasets.extend([div7a()?, gic()?, calendar()?, sg_parameters()?]);
    datasets.extend(worksheets()?);
    datasets.extend([lodgment(), gst()]);
    if let Some(tool) = tool {
        datasets.retain(|dataset| {
            dataset["tools"]
                .as_array()
                .is_some_and(|tools| tools.iter().any(|t| t == tool))
        });
    }
    if let Some(day) = as_at {
        for dataset in &mut datasets {
            dataset["as_at_status"] = json!(as_at_status(&dataset["effective_periods"], day));
        }
    }
    Ok(
        json!({"ok":true,"server":{"distribution":"aus-accounting-mcp","version":crate::VERSION},
        "bundled_data":true,"live_lookup":false,"currency":"NOT_ESTABLISHED","statement":STATEMENT,
        "as_at":as_at,"tool":tool,"datasets":datasets,"not_listed":NOT_LISTED}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_iso_date_is_reported_as_a_date() {
        for value in [json!(null), json!(""), json!("2026-13-01"), json!(20261028)] {
            assert_eq!(iso_or_unknown(&value), UNKNOWN);
        }
        assert_eq!(iso_or_unknown(&json!("2026-10-28")), "2026-10-28");
        assert_eq!(dates([&json!(null)]), UNKNOWN);
        assert_eq!(
            dates([
                &json!("2026-10-28"),
                &json!(null),
                &json!("2026-01-02"),
                &json!("2026-10-28")
            ]),
            json!(["2026-01-02", "2026-10-28"])
        );
        assert_eq!(urls([&json!("http://example.com")]), UNKNOWN);
    }

    #[test]
    fn a_label_that_is_not_one_financial_year_is_refused() {
        assert_eq!(
            financial_year("2025-26", "x").unwrap()["start"],
            "2025-07-01"
        );
        assert_eq!(financial_year("1999-00", "x").unwrap()["end"], "2000-06-30");
        for label in ["2025-27", "2025", "2025-2026", "abcd-26", "2025/26"] {
            assert!(financial_year(label, "x").is_err(), "{label}");
        }
    }

    #[test]
    fn a_date_in_a_gap_between_recorded_periods_is_outside() {
        let periods = json!([{"start":"2024-01-01","end":"2024-03-31"},{"start":"2024-07-01","end":"2024-09-30"}]);
        assert_eq!(
            as_at_status(&periods, "2024-05-01"),
            "OUTSIDE_RECORDED_PERIOD"
        );
        assert_eq!(
            as_at_status(&periods, "2024-07-01"),
            "INSIDE_RECORDED_PERIOD"
        );
        assert_eq!(as_at_status(&json!(UNKNOWN), "2024-07-01"), UNKNOWN);
    }
}
