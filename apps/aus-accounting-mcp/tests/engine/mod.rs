// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Named divergences between the Python oracle and the engine crates this server links.
//!
//! The frozen fixtures hold the Python server's replies, which come from the published
//! engine releases (australian-tax-calculators 0.2.0 and the other PyPI pins). The
//! native server links the engine crates as they are on main. Each function below
//! applies one engine commit on main to a frozen Python value, mirroring the engine's
//! own `tests/parity.rs` adaptation, so every other byte must still match. No figure is
//! normalised except the carry-forward cap, which follows s 291-20(4) as #542 applies it.
#![allow(dead_code)]

use australian_tax_calculators::decimal::{Decimal, Rounding};
use serde_json::{Value, json};

// fd0b593e (#543) opens 2026-27 for gst, capital gains and depreciation, b4a70733 (#551)
// opens 2024-25 and 2025-26 for payg_withholding and d8f34dc2 (#567) opens the FBT year
// ending 31 March 2027. The input schema fixes or bounds each of those years, so the
// server advertises and reaches none of those periods, and serves the 0.2.0 scope words
// for the periods it admits: only the sources, check date and passage of #551 and #567
// reach a reply.
/// b4a70733 (#551): the PAYG source page and passage.
const PAYG_PAGE: &str =
    "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026";
const PAYG_PASSAGE: &str = "Schedule 1 (NAT 1004) for payments from 1 July 2026, and for payments from 1 July 2024 to 30 June 2026: weekly coefficients, using a formula, working out the weekly earnings";
/// d8f34dc2 (#567): the FBT source page and passage.
const FBT_PAGE: &str =
    "https://www.ato.gov.au/tax-rates-and-codes/fringe-benefits-tax-rates-and-thresholds";
const FBT_PASSAGE: &str = "; FBT rates and thresholds, the FBT rate and gross-up rates for years ending 31 March 2023 to 2027";
/// The date both #551 and #567 record for their source checks.
const CHECKED_2026_11_04: &str = "2026-11-04";

fn page(kind: &str) -> Option<&'static str> {
    match kind {
        "payg_withholding" => Some(PAYG_PAGE),
        "fbt" => Some(FBT_PAGE),
        _ => None,
    }
}

/// 04d493f7 (#542): only the unused cap that covers the excess is applied. The frozen
/// value is all the eligible unused cap, so it is the upper bound.
fn carry_forward(facts: &Value, result: &mut Value) {
    let dec = |v: &Value| Decimal::parse(v.as_str().unwrap().trim()).unwrap();
    let zero = Decimal::from_int(0);
    let carried = dec(&result["amounts"]["carry_forward_applied"]);
    let excess = dec(&facts["concessional_contributions"])
        .sub(&dec(&result["rates"]["concessional_cap"]))
        .unwrap();
    let needed = if excess.gt(&zero).unwrap() {
        excess
    } else {
        zero
    };
    let applied = if needed.lt(&carried).unwrap() {
        needed
    } else {
        carried
    };
    let cents = applied.quantize(-2, Rounding::HalfUp).unwrap();
    result["amounts"]["carry_forward_applied"] = json!(cents.to_string());
}

/// A frozen successful calculate_tax_worksheet result, as the linked engine gives it.
pub fn worksheet(kind: &str, facts: &Value, result: &mut Value) {
    if kind == "contribution_caps" && result["amounts"].get("carry_forward_applied").is_some() {
        carry_forward(facts, result);
    }
    if let Some(page) = page(kind) {
        result["sources"].as_array_mut().unwrap().push(json!(page));
        result["source_checked"] = json!(CHECKED_2026_11_04);
    }
}

/// A frozen worksheet catalogue entry (the scope resource), as the linked engine gives it.
pub fn catalogue(kind: &str, entry: &mut Value) {
    let Some(page) = page(kind) else {
        return;
    };
    entry["sources"].as_array_mut().unwrap().push(json!(page));
    entry["source_checked"] = json!(CHECKED_2026_11_04);
    let passage = entry["source_passage"].as_str().unwrap();
    entry["source_passage"] = json!(if kind == "fbt" {
        format!("{passage}{FBT_PASSAGE}")
    } else {
        PAYG_PASSAGE.to_owned()
    });
}

/// A frozen tax_worksheet freshness dataset, as the linked engine's catalogue gives it.
pub fn freshness(row: &mut Value) {
    let Some(kind) = row["dataset"]
        .as_str()
        .and_then(|d| d.strip_prefix("tax_worksheet:"))
    else {
        return;
    };
    let kind = kind.to_owned();
    let Some(page) = page(&kind) else {
        return;
    };
    row["sources"].as_array_mut().unwrap().push(json!(page));
    row["source_checked"] = json!(CHECKED_2026_11_04);
    let passage = row["provenance"]["source_passage"].as_str().unwrap();
    row["provenance"]["source_passage"] = json!(if kind == "fbt" {
        format!("{passage}{FBT_PASSAGE}")
    } else {
        PAYG_PASSAGE.to_owned()
    });
}

/// A frozen Python file note, restating the adapted result `after` where `before` held
/// Python's: each source line and its check date, changed table rows and the catalogue
/// passage. The note's layout is Python's.
pub fn file_note(kind: &str, note: &str, before: &Value, after: &Value) -> String {
    let mut note = note.to_owned();
    let lines = |result: &Value| -> String {
        result["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|url| {
                format!(
                    "- {} (checked {})",
                    url.as_str().unwrap(),
                    result["source_checked"].as_str().unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    note = note.replace(&lines(before), &lines(after));
    for section in ["amounts", "rates"] {
        if let Some(rows) = before[section].as_object() {
            for (key, old) in rows {
                let new = &after[section][key];
                if old != new {
                    note = note.replace(
                        &format!("| `{key}` | {} |", old.as_str().unwrap()),
                        &format!("| `{key}` | {} |", new.as_str().unwrap()),
                    );
                }
            }
        }
    }
    let covered = "The engine's source check covered: ";
    if page(kind).is_some()
        && let Some(start) = note.find(covered)
    {
        let from = start + covered.len();
        let end = from + note[from..].find(".\n").unwrap();
        let passage = &note[from..end];
        let replacement = if kind == "fbt" {
            format!("{passage}{FBT_PASSAGE}")
        } else {
            PAYG_PASSAGE.to_owned()
        };
        note.replace_range(from..end, &replacement);
    }
    note
}

/// 56afe0c (#602): the payday first-contribution hint names recommenced employment, as
/// the engine's `tests/data/amendments.json` applies it to the engine's own frozen cases.
/// Only these words change; no deadline, verdict or figure.
const HINT_BEFORE: &str =
    "If it is a new starter or a fund switch, set first_contribution_to_fund=yes";
const HINT_AFTER: &str = "If it is the first after employment starts or recommences (including rehiring a worker under a new arrangement), or after a fund switch, set first_contribution_to_fund=yes";

/// Frozen payday text, as the linked engine words it.
pub fn payday_hint(text: &str) -> String {
    text.replace(HINT_BEFORE, HINT_AFTER)
}

/// Every string of a frozen payday result, as the linked engine words it.
pub fn payday(value: &mut Value) {
    match value {
        Value::String(text) => *text = payday_hint(text),
        Value::Array(items) => items.iter_mut().for_each(payday),
        Value::Object(object) => object.values_mut().for_each(payday),
        _ => {}
    }
}
