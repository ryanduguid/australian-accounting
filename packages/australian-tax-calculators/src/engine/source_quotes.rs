// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The passage of each Act, determination or revenue office page that states
//! the rate, scale or grant a state duty, surcharge, land tax, first home
//! owner grant, motor vehicle duty, insurance duty or Victorian property tax
//! command applies, held in data/source-quotes.json and quoted
//! word for word as read at the official site on [`VERIFIED`]
//! (docs/calculation-evidence.md#quoted-source-passages). Line and table row
//! breaks are kept as newlines and table cells are joined by a tab;
//! non-breaking spaces are written as spaces and zero-width characters are
//! left out.

use std::sync::OnceLock;

use serde_json::{Value, json};

/// The day every passage was read at its URL.
const VERIFIED: &str = "2026-11-24";

/// The commands whose results carry `source_quotes`, by API command name,
/// with the input that picks a result's passages: the jurisdiction, or the
/// tax for the Victorian property taxes.
const COMMANDS: [(&str, &str, &str); 7] = [
    ("transfer-duty", "transfer_duty", "jurisdiction"),
    (
        "foreign-purchaser-surcharge",
        "foreign_purchaser_surcharge",
        "jurisdiction",
    ),
    ("land-tax", "land_tax", "jurisdiction"),
    ("first-home-owner-grant", "first_home_owner_grant", "jurisdiction"),
    ("motor-vehicle-duty", "motor_vehicle_duty", "jurisdiction"),
    ("insurance-duty", "insurance_duty", "jurisdiction"),
    ("vic-property-taxes", "vic_property_taxes", "tax"),
];

/// Each passage as JSON, with its read date.
fn quotes() -> &'static [Value] {
    static QUOTES: OnceLock<Vec<Value>> = OnceLock::new();
    QUOTES.get_or_init(|| {
        let mut all: Vec<Value> =
            serde_json::from_str(include_str!("../../data/source-quotes.json"))
                .expect("data/source-quotes.json");
        for quote in &mut all {
            quote["verified"] = json!(VERIFIED);
        }
        all
    })
}

/// The passages of worksheet `kind` whose field `key` is `value`, or all of
/// them, without the worksheet name.
fn select(kind: &str, pick: Option<(&str, &str)>) -> Vec<Value> {
    quotes()
        .iter()
        .filter(|q| q["kind"] == kind && pick.is_none_or(|(key, value)| q[key] == value))
        .map(|q| {
            let mut quote = q.clone();
            quote.as_object_mut().expect("a passage").remove("kind");
            quote
        })
        .collect()
}

/// Every quoted passage of worksheet `kind`, for its catalogue entry, or
/// `None` for a worksheet without them.
pub(crate) fn catalogue(kind: &str) -> Option<Value> {
    let all = select(kind, None);
    (!all.is_empty()).then_some(Value::Array(all))
}

/// Adds to a successful `command` result the passages for the jurisdiction,
/// or tax, its input names.
pub(crate) fn attach(command: &str, input: &Value, result: &mut Value) {
    let Some((_, kind, key)) = COMMANDS.iter().find(|(c, _, _)| *c == command) else {
        return;
    };
    if let Some(value) = input[key].as_str() {
        result["source_quotes"] = Value::Array(select(kind, Some((key, value))));
    }
}
