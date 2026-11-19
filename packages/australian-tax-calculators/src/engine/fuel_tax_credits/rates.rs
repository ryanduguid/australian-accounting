// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The bundled ATO rate periods, `data/fuel-tax-credit-rates.json`, read
//! once and checked.

use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::dates;
use crate::decimal::Decimal;

/// The ATO's 6 rows of eligible fuel types, in the data file's keys.
pub(super) const CATEGORIES: [&str; 6] = ["liquid", "blend", "e85", "lpg", "lng_cng", "b100"];

/// One published rate period.
pub(super) struct Period {
    pub(super) from: i64,
    pub(super) to: i64,
    pub(super) starts_with: String,
    pub(super) source: Value,
    pub(super) heavy: Vec<Decimal>,
    pub(super) other: Vec<Decimal>,
}

pub(super) struct Rates {
    pub(super) read: String,
    pub(super) periods: Vec<Period>,
}

pub(super) fn rates() -> &'static Rates {
    static RATES: OnceLock<Rates> = OnceLock::new();
    RATES.get_or_init(|| parse_rates(include_str!("../../../data/fuel-tax-credit-rates.json")))
}

/// The bundled rates: contiguous periods from 1 July 2022, 6 fuel rows in 2
/// uses each. A malformed file is a build defect, so it panics.
fn parse_rates(text: &str) -> Rates {
    let data: Value = serde_json::from_str(text).expect("JSON rates");
    let day = |v: &Value| dates::parse(v.as_str().expect("a date")).expect("YYYY-MM-DD");
    let row = |v: &Value| -> Vec<Decimal> {
        CATEGORIES
            .iter()
            .map(|c| Decimal::parse(v[*c].as_str().expect("a rate")).expect("a decimal rate"))
            .collect()
    };
    let known = data["starts_with"]
        .as_object()
        .expect("period start reasons");
    let mut periods: Vec<Period> = Vec::new();
    for p in data["periods"].as_array().expect("periods") {
        let (from, to) = (day(&p["from"]), day(&p["to"]));
        assert!(from <= to, "{from}: ordered");
        assert!(
            periods.last().is_none_or(|last| last.to + 1 == from),
            "{from}: contiguous"
        );
        let starts_with = p["starts_with"].as_str().expect("a reason").to_string();
        assert!(
            known.contains_key(&starts_with),
            "{starts_with}: a known reason"
        );
        for key in ["page", "table", "url", "updated", "read"] {
            assert!(p["source"][key].is_string(), "{from}: source {key}");
        }
        periods.push(Period {
            from,
            to,
            starts_with,
            source: p["source"].clone(),
            heavy: row(&p["cents"]["heavy_vehicle_public_road"]),
            other: row(&p["cents"]["other_business"]),
        });
    }
    assert_eq!(periods[0].from, dates::parse("2022-07-01").expect("date"));
    let read = data["index"]["read"]
        .as_str()
        .expect("read date")
        .to_string();
    Rates { read, periods }
}

/// The period whose rates apply on `day`, which lies in the published range.
pub(super) fn period_on(day: i64) -> &'static Period {
    rates()
        .periods
        .iter()
        .find(|p| p.from <= day && day <= p.to)
        .expect("a day in the published range")
}

pub(super) fn period_json(p: &Period) -> Value {
    json!({"from": dates::format(p.from), "to": dates::format(p.to), "source": p.source})
}
