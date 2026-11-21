// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! What the catalogue and the steps cite: every worksheet's evidence note opens
//! a heading of docs/calculation-evidence.md, and resident tax cites the table
//! for the year it applied.
use australian_tax_calculators::{api, engine::worksheet_catalogue, metadata};
use serde_json::json;
use std::collections::BTreeSet;

/// The anchors GitHub gives the document's headings: lower case, punctuation
/// other than hyphens and underscores dropped, spaces as hyphens.
fn anchors() -> BTreeSet<String> {
    include_str!("../docs/calculation-evidence.md")
        .lines()
        .filter(|line| line.starts_with('#'))
        .map(|line| {
            line.trim_start_matches('#')
                .trim()
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | ' '))
                .map(|c| if c == ' ' { '-' } else { c })
                .collect()
        })
        .collect()
}

#[test]
fn every_evidence_note_opens_a_heading() {
    let anchors = anchors();
    let catalogue = worksheet_catalogue();
    let entries = catalogue.as_object().unwrap();
    assert!(entries.len() > 40);
    let broken: Vec<String> = entries
        .iter()
        .filter_map(|(kind, entry)| {
            let note = entry["example_evidence"]["note"].as_str().unwrap();
            let anchor = note.strip_prefix("docs/calculation-evidence.md#").unwrap();
            (!anchors.contains(anchor)).then(|| format!("{kind}: {note}"))
        })
        .collect();
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn resident_tax_cites_the_table_for_its_year() {
    for year in metadata::supported_periods("resident_tax") {
        let facts = json!({"taxable_income": "50000", "year": year, "scope_confirmed": true});
        let doc = api::evaluate(
            "derive",
            &json!({"command": "resident-tax", "input": facts}),
        )
        .unwrap();
        let step = doc["steps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == "basic_income_tax")
            .unwrap();
        let authority = step["authority"].as_str().unwrap();
        assert!(authority.contains(year), "{year}: {authority}");
    }
}
