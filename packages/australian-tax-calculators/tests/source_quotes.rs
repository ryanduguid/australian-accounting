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
        if place == "wa" {
            fields.insert("metropolitan_region".into(), json!("none"));
        }
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

/// The figure each jurisdiction's passages must state, as the command's own
/// rate table, surcharge or grant applies it (the top band for a scale).
const FIGURES: [(&str, &str, &str); 31] = [
    (
        "transfer_duty",
        "nsw",
        "$52,237 plus $5.50 for every $100 over $1,290,000",
    ),
    ("transfer_duty", "vic", "$110,000 plus 6.5%"),
    ("transfer_duty", "qld", "$38,025 plus $5.75"),
    ("transfer_duty", "wa", "$28,453 + $5.15"),
    ("transfer_duty", "sa", "$21,330 plus $5.50"),
    ("transfer_duty", "tas", "$27 810 plus $4.50"),
    ("transfer_duty", "act", "$4.54 per $100"),
    ("transfer_duty", "nt", "0.06571441"),
    ("foreign_purchaser_surcharge", "nsw", "9%"),
    ("foreign_purchaser_surcharge", "vic", "8%"),
    ("foreign_purchaser_surcharge", "qld", "8%"),
    ("foreign_purchaser_surcharge", "wa", "7%"),
    ("foreign_purchaser_surcharge", "sa", "7%"),
    ("foreign_purchaser_surcharge", "tas", "8 percent"),
    ("foreign_purchaser_surcharge", "act", "0.75%"),
    ("foreign_purchaser_surcharge", "nt", "no land tax"),
    ("land_tax", "nsw", "$88,036 + 2%"),
    ("land_tax", "vic", "$31,650 plus 2.65%"),
    ("land_tax", "qld", "$150,000 plus 2.25 cents"),
    ("land_tax", "wa", "$186,550 + 0.0267"),
    ("land_tax", "sa", "$36,000 plus $2.40"),
    ("land_tax", "tas", "$1 737.50 plus 1.5%"),
    ("land_tax", "act", "$1,778"),
    ("first_home_owner_grant", "nsw", "$10,000"),
    ("first_home_owner_grant", "vic", "$10,000"),
    ("first_home_owner_grant", "qld", "$30,000"),
    ("first_home_owner_grant", "wa", "$10,000"),
    ("first_home_owner_grant", "sa", "$15,000"),
    ("first_home_owner_grant", "tas", "$20 000"),
    ("first_home_owner_grant", "act", "from 1 July 2019"),
    ("first_home_owner_grant", "nt", "$50 000"),
];

#[test]
fn each_passage_states_the_figure_the_command_applies() {
    for (kind, place, figure) in FIGURES {
        let quotes = of(&catalogue_quotes(kind), place);
        assert!(
            quotes
                .iter()
                .any(|q| q["passage"].as_str().unwrap().contains(figure)),
            "{kind} {place}: {figure}"
        );
    }
}

/// The commands take lower-case codes only, so a result's passages cannot
/// belong to another spelling of its jurisdiction.
#[test]
fn an_upper_case_jurisdiction_is_refused() {
    for (command, kind, _, _) in COMMANDS {
        assert!(
            api::evaluate(command, &example(kind, "NSW")).is_err(),
            "{command}"
        );
    }
}

/// A command outside the four, with a jurisdiction input, gains nothing.
#[test]
fn other_commands_gain_no_quotes() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["motor_vehicle_duty"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert!(facts.get("jurisdiction").is_some());
    let result = api::evaluate("motor-vehicle-duty", &facts).unwrap();
    assert!(result.get("source_quotes").is_none());
}
