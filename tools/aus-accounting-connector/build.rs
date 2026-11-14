// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Fix the release's serve-until date: the first day every tool call is refused.
//! It is the earliest of the day before the next 1 July after the build, the end
//! of the GIC table's last quarter and the business-day calendar's verified_until,
//! the payday tables `.github/ci/check_rate_expiry.py` watches. The build date is
//! SOURCE_DATE_EPOCH when set, so a release build is reproducible.
//! AUS_ACCOUNTING_CONNECTOR_SERVE_UNTIL may bring the date earlier, never later.
use chrono::{Datelike, NaiveDate};
use std::{env, fs};

const DATA: &str = "../../packages/payday-super-checker/data";

fn table(name: &str) -> serde_json::Value {
    let path = format!("{DATA}/{name}");
    println!("cargo::rerun-if-changed={path}");
    serde_json::from_str(&fs::read_to_string(&path).expect(&path)).expect(&path)
}

fn day(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").expect(text)
}

fn main() {
    for name in [
        "SOURCE_DATE_EPOCH",
        "AUS_ACCOUNTING_CONNECTOR_SERVE_UNTIL",
        "AUS_ACCOUNTING_CONNECTOR_COMMIT",
    ] {
        println!("cargo::rerun-if-env-changed={name}");
    }
    let built = match env::var("SOURCE_DATE_EPOCH") {
        Ok(seconds) => {
            chrono::DateTime::from_timestamp(seconds.parse().expect("SOURCE_DATE_EPOCH"), 0)
                .expect("SOURCE_DATE_EPOCH")
                .date_naive()
        }
        Err(_) => chrono::Utc::now().date_naive(),
    };
    let july = NaiveDate::from_ymd_opt(built.year(), 7, 1).unwrap();
    let next_july = if built < july {
        july
    } else {
        july.with_year(built.year() + 1).unwrap()
    };
    let gic_end = table("gic_rates.json")["quarters"]
        .as_array()
        .expect("quarters")
        .iter()
        .map(|quarter| day(quarter["to"].as_str().expect("to")))
        .max()
        .expect("a GIC quarter");
    let verified_until = day(table("business_days.json")["verified_until"]
        .as_str()
        .expect("verified_until"));
    let mut serve_until = [next_july.pred_opt().unwrap(), gic_end, verified_until]
        .into_iter()
        .min()
        .unwrap();
    if let Ok(earlier) = env::var("AUS_ACCOUNTING_CONNECTOR_SERVE_UNTIL") {
        let earlier = day(&earlier);
        assert!(
            earlier <= serve_until,
            "the serve-until override can only bring the date earlier"
        );
        serve_until = earlier;
    }
    println!("cargo::rustc-env=CONNECTOR_SERVE_UNTIL={serve_until}");
    let commit = env::var("AUS_ACCOUNTING_CONNECTOR_COMMIT").unwrap_or_else(|_| "unknown".into());
    println!("cargo::rustc-env=CONNECTOR_COMMIT={commit}");
}
