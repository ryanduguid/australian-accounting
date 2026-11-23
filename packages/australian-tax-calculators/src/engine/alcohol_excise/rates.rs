// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The bundled substituted rates, `data/alcohol-excise-rates.json`, read once
//! and checked.

use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::dates;
use crate::decimal::Decimal;

/// The Excise Tariff Act 1921 Schedule subitems with a rate, in the data
/// file's order.
pub(super) const SUBITEMS: [&str; 12] = [
    "1.1", "1.2", "1.5", "1.6", "1.10", "1.11", "1.15", "1.16", "2", "3.1", "3.2", "3.10",
];

/// One notice's rates and the days they apply to here.
pub(super) struct Period {
    pub(super) from: i64,
    pub(super) to: i64,
    pub(super) notice: Value,
    pub(super) rates: Vec<Decimal>,
}

pub(super) struct Rates {
    pub(super) read: String,
    pub(super) periods: Vec<Period>,
}

pub(super) fn rates() -> &'static Rates {
    static RATES: OnceLock<Rates> = OnceLock::new();
    RATES.get_or_init(|| parse_rates(include_str!("../../../data/alcohol-excise-rates.json")))
}

/// Contiguous periods from 1 July 2022, a rate for every subitem in each. A
/// malformed file is a build defect, so it panics.
fn parse_rates(text: &str) -> Rates {
    let data: Value = serde_json::from_str(text).expect("JSON rates");
    let day = |v: &Value| dates::parse(v.as_str().expect("a date")).expect("YYYY-MM-DD");
    let mut periods: Vec<Period> = Vec::new();
    for p in data["periods"].as_array().expect("periods") {
        let (from, to) = (day(&p["from"]), day(&p["to"]));
        assert!(from <= to, "{from}: ordered");
        assert!(
            periods.last().is_none_or(|last| last.to + 1 == from),
            "{from}: contiguous"
        );
        for key in ["id", "title", "in_force_from", "url", "read"] {
            assert!(p["notice"][key].is_string(), "{from}: notice {key}");
        }
        let row = &p["dollars_per_litre_of_alcohol"];
        let rates = SUBITEMS
            .iter()
            .map(|s| Decimal::parse(row[*s].as_str().expect("a rate")).expect("a decimal rate"))
            .collect();
        periods.push(Period {
            from,
            to,
            notice: p["notice"].clone(),
            rates,
        });
    }
    assert_eq!(periods[0].from, dates::parse("2022-07-01").expect("date"));
    let read = data["index"]["read"]
        .as_str()
        .expect("read date")
        .to_string();
    Rates { read, periods }
}

/// The period whose rates apply on `day`, which lies in the held range.
pub(super) fn period_on(day: i64) -> &'static Period {
    rates()
        .periods
        .iter()
        .find(|p| p.from <= day && day <= p.to)
        .expect("a day in the held range")
}

/// The rate for `subitem` in `period`.
pub(super) fn rate_of<'p>(period: &'p Period, subitem: &str) -> &'p Decimal {
    let i = SUBITEMS
        .iter()
        .position(|s| *s == subitem)
        .expect("a rated subitem");
    &period.rates[i]
}

pub(super) fn period_json(p: &Period) -> Value {
    json!({"from": dates::format(p.from), "to": dates::format(p.to), "notice": p.notice})
}
