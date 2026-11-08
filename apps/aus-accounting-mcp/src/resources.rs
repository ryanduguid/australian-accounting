// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Policy is embedded; mutable engine metadata comes from the linked engines.
use ato_benchmark_compare::{
    dataset::{self, DataDir},
    report,
};
use payday_super_checker::{calendar, rates};
use serde_json::{Value, json};
use std::sync::LazyLock;

static POLICY: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/resource-policy.v1.json")).unwrap()
});
pub const RETRIEVAL_FOLDERS: [&str; 3] = [
    "AUS_ACCOUNTING_LIBRARY_ROOT",
    "AUS_ACCOUNTING_CORPUS_ROOT",
    "AUS_ACCOUNTING_RULINGS_ROOT",
];

pub fn read(uri: &str) -> Result<Value, String> {
    let mut value = POLICY
        .get(uri)
        .ok_or_else(|| format!("Unknown resource: {uri}"))?
        .clone();
    match uri {
        "aus-accounting://scope" => {
            value["calculation_worksheets"] = crate::adapters::tax::advertised_worksheets()?;
            for name in RETRIEVAL_FOLDERS {
                let configured = std::env::var_os(name)
                    .filter(|s| !s.is_empty())
                    .is_some_and(|p| std::path::Path::new(&p).is_dir());
                value["retrieval_folders_configured"][name] = json!(configured);
            }
        }
        "aus-accounting://component-versions" => {
            value["server"]["version"] = json!(crate::VERSION);
            value["engines"] = json!([
                {"distribution":"ato-benchmark-compare","version":ato_benchmark_compare::VERSION},
                {"distribution":"div7a-loan-review","version":div7a_loan_review::VERSION},
                {"distribution":"payday-super-checker","version":payday_super_checker::VERSION},
                {"distribution":"australian-tax-calculators","version":australian_tax_calculators::engine::VERSION},
                {"distribution":"gst-review","version":gst_review::VERSION},
                {"distribution":"lodgment-calendar","version":lodgment_calendar::VERSION}]);
        }
        "aus-accounting://benchmark-dataset-years" => {
            let years = dataset::available_years(&DataDir::Shipped);
            value["engine_version"] = json!(ato_benchmark_compare::VERSION);
            value["latest_benchmark_year"] = json!(years.last());
            value["datasets"]=Value::Array(years.iter().map(|year| {
                let data=dataset::load(Some(year),&DataDir::Shipped).map_err(|e|e.to_string())?;
                Ok(json!({"benchmark_year":data.year,"business_types":data.business_types.len(),"source":report::raw_json(&data.source)}))
            }).collect::<Result<_,String>>()?);
        }
        "aus-accounting://payday-coverage" => {
            let calendar = calendar::load_calendar(None).map_err(|e| e.message())?;
            let gic = rates::load_gic().map_err(|e| e.message())?;
            value["engine_version"] = json!(payday_super_checker::VERSION);
            value["law_content_date"] = json!(payday_super_checker::LAW_CONTENT_DATE);
            value["calendar"] = json!({"verified_from":calendar.verified_from.isoformat(),"verified_until":calendar.verified_until.isoformat(),"coverage_until":calendar.coverage_until.isoformat()});
            value["gic"]["known_until"] = json!(gic.last_known().isoformat());
            value["gic"]["provenance"] = json!(gic.provenance());
        }
        _ => {}
    }
    Ok(value)
}

pub fn result(uri: &str) -> Result<Value, String> {
    let value = read(uri)?;
    let metadata = crate::contracts::CATALOGUE["resources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["uri"] == uri)
        .unwrap();
    let text = if value.is_string() {
        value.as_str().unwrap().to_string()
    } else {
        serde_json::to_string_pretty(&value).unwrap()
    };
    Ok(json!({"contents":[{"uri":uri,"mimeType":metadata["mimeType"],"text":text}]}))
}
