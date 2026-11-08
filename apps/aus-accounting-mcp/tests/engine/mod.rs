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
// server advertises and reaches none of those periods: only the scope, sources, check
// date and passage of #551 and #567 reach a reply.
/// b4a70733 (#551): the PAYG scope, sources and check date.
const PAYG_OLD_SCOPE: &str = "made from 1 July 2026 to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations.";
const PAYG_NEW_SCOPE: &str = "made from 1 July 2024, in the income year given, to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. The year selects the Schedule 1 coefficients for payments made in it.";
const PAYG_PAGE: &str =
    "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026";
const PAYG_PASSAGE: &str = "Schedule 1 (NAT 1004) for payments from 1 July 2026, and for payments from 1 July 2024 to 30 June 2026: weekly coefficients, using a formula, working out the weekly earnings";
/// d8f34dc2 (#567): the FBT scope, sources and passage.
const FBT_OLD_SCOPE: &str = "for the year ended 31 March 2026. Excludes";
const FBT_NEW_SCOPE: &str = "for the year ended 31 March 2026 or 2027, both at the 47% FBT rate and the 2.0802 and 1.8868 gross-up rates. Excludes";
const FBT_OLD_END: &str = "before any optional rounding down to 5 cents.";
const FBT_NEW_END: &str = "before any optional rounding down to 5 cents. For 2027 the items follow the FBT return 2026 instructions until the ATO publishes the 2027 return.";
const FBT_PAGE: &str =
    "https://www.ato.gov.au/tax-rates-and-codes/fringe-benefits-tax-rates-and-thresholds";
const FBT_PASSAGE: &str = "; FBT rates and thresholds, the FBT rate and gross-up rates for years ending 31 March 2023 to 2027";
/// The date both #551 and #567 record for their source checks.
const CHECKED_2026_11_04: &str = "2026-11-04";

fn scope(kind: &str, record: &mut Value) {
    let text = record["scope"].as_str().unwrap();
    let text = match kind {
        "payg_withholding" => {
            assert!(text.contains(PAYG_OLD_SCOPE), "frozen PAYG scope changed");
            text.replace(PAYG_OLD_SCOPE, PAYG_NEW_SCOPE)
        }
        "fbt" => {
            assert!(text.contains(FBT_OLD_SCOPE) && text.ends_with(FBT_OLD_END));
            text.replace(FBT_OLD_SCOPE, FBT_NEW_SCOPE)
                .replace(FBT_OLD_END, FBT_NEW_END)
        }
        _ => return,
    };
    record["scope"] = json!(text);
}

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
        scope(kind, result);
        result["sources"].as_array_mut().unwrap().push(json!(page));
        result["source_checked"] = json!(CHECKED_2026_11_04);
    }
}

/// A frozen worksheet catalogue entry (the scope resource), as the linked engine gives it.
pub fn catalogue(kind: &str, entry: &mut Value) {
    let Some(page) = page(kind) else {
        return;
    };
    scope(kind, entry);
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
/// Python's: the scope line, each source line and its check date, changed table rows and
/// the catalogue passage. The note's layout is Python's.
pub fn file_note(kind: &str, note: &str, before: &Value, after: &Value) -> String {
    let mut note = note.to_owned();
    for (old, new) in [(&before["scope"], &after["scope"])] {
        note = note.replace(old.as_str().unwrap(), new.as_str().unwrap());
    }
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

/// The frozen disclaimer, with each worksheet scope line as the linked engine gives it.
pub fn disclaimer(text: &str) -> String {
    let lines = text.split('\n').map(|line| {
        for kind in ["payg_withholding", "fbt"] {
            if let Some(old) = line.strip_prefix(&format!("{kind}: ")) {
                let mut record = json!({ "scope": old });
                scope(kind, &mut record);
                return format!("{kind}: {}", record["scope"].as_str().unwrap());
            }
        }
        line.to_owned()
    });
    lines.collect::<Vec<_>>().join("\n")
}
