// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Every state duty, foreign purchaser surcharge, land tax and first home
//! owner grant jurisdiction carries a passage quoted from its primary source,
//! with the provision, the URL and the day it was read, in the worksheet
//! catalogue and in each result.

use australian_tax_calculators::api;
use serde_json::{Value, json};

const ALL: [&str; 8] = ["nsw", "vic", "qld", "wa", "sa", "tas", "act", "nt"];

/// Each command, its worksheet, the jurisdictions it takes and those whose
/// catalogue example refuses (the ACT has paid no grant since 1 July 2019).
const COMMANDS: [(&str, &str, &[&str], &[&str]); 4] = [
    ("transfer-duty", "transfer_duty", &ALL, &[]),
    (
        "foreign-purchaser-surcharge",
        "foreign_purchaser_surcharge",
        &ALL,
        &[],
    ),
    (
        "land-tax",
        "land_tax",
        &["nsw", "vic", "qld", "wa", "sa", "tas", "act"],
        &[],
    ),
    (
        "first-home-owner-grant",
        "first_home_owner_grant",
        &ALL,
        &["act"],
    ),
];

fn catalogue_quotes(kind: &str) -> Vec<Value> {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    catalogue[kind]["source_quotes"]
        .as_array()
        .unwrap_or_else(|| panic!("{kind} has no source_quotes"))
        .clone()
}

fn of(quotes: &[Value], place: &str) -> Vec<Value> {
    quotes
        .iter()
        .filter(|q| q["jurisdiction"] == place)
        .cloned()
        .collect()
}

#[test]
fn every_jurisdiction_has_a_quoted_passage_and_its_date() {
    for (_, kind, places, _) in COMMANDS {
        let quotes = catalogue_quotes(kind);
        for place in places {
            assert!(!of(&quotes, place).is_empty(), "{kind} {place}");
        }
        for quote in &quotes {
            let text = |field: &str| quote[field].as_str().unwrap_or_default();
            assert!(places.contains(&text("jurisdiction")), "{quote}");
            for field in ["provision", "source", "passage"] {
                assert!(!text(field).trim().is_empty(), "{kind} {field}");
            }
            assert!(text("url").starts_with("https://"), "{}", text("url"));
            let host = text("url")[8..].split('/').next().unwrap();
            assert!(host.ends_with(".gov.au"), "{host}");
            assert_eq!(text("verified"), "2026-11-24", "{kind}");
            assert!(!text("passage").contains('\u{200b}'), "{kind}");
        }
    }
}

#[test]
fn other_worksheets_carry_no_quotes() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let quoted: Vec<&str> = COMMANDS.iter().map(|c| c.1).collect();
    for (kind, entry) in catalogue.as_object().unwrap() {
        assert_eq!(
            entry.get("source_quotes").is_some(),
            quoted.contains(&kind.as_str()),
            "{kind}"
        );
    }
}

/// The catalogue's example for `kind`, moved to `place`.
fn example(kind: &str, place: &str) -> Value {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue[kind]["example"]["facts"].clone();
    let fields = facts.as_object_mut().unwrap();
    fields.remove("kind");
    fields.insert("jurisdiction".into(), json!(place));
    if kind == "land_tax" {
        let year = if matches!(place, "nsw" | "vic") {
            "2026"
        } else {
            "2026-27"
        };
        fields.insert("year".into(), json!(year));
    }
    facts
}

#[test]
fn each_result_carries_its_jurisdictions_quotes() {
    for (command, kind, places, refused) in COMMANDS {
        let quotes = catalogue_quotes(kind);
        for place in places {
            let result = api::evaluate(command, &example(kind, place));
            if refused.contains(place) {
                assert!(result.is_err(), "{command} {place}");
                continue;
            }
            let result = result.unwrap_or_else(|e| panic!("{command} {place}: {e}"));
            assert_eq!(
                result["source_quotes"],
                Value::Array(of(&quotes, place)),
                "{command} {place}"
            );
        }
    }
}

/// A command outside the four, with a jurisdiction input, gains nothing.
#[test]
fn other_commands_gain_no_quotes() {
    let result =
        api::evaluate("motor-vehicle-duty", &example("motor_vehicle_duty", "vic")).unwrap();
    assert!(result.get("source_quotes").is_none());
}
