// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Named divergences between the Python oracle and the engine crates this server links.
//!
//! The frozen fixtures hold the Python server's replies, which come from the published
//! engine releases (australian-tax-calculators 0.2.0 and the other PyPI pins). The
//! native server links the engine crates as they are on main. Each function below
//! applies one engine commit on main to a frozen Python value, mirroring the engine's
//! own `tests/parity.rs` adaptation, so every other byte must still match. The figures changed here
//! follow the carry-forward cap in s 291-20(4) and the FBTAA gross-up formulas.
#![allow(dead_code)]

use australian_tax_calculators::decimal::{Decimal, Rounding};
use serde_json::{Value, json};

// fd0b593e (#543) opens 2026-27 for gst, capital gains and depreciation, b4a70733 (#551)
// opens 2024-25 and 2025-26 for payg_withholding and d8f34dc2 (#567) opens the FBT year
// ending 31 March 2027. Later commits widen study loan withholding, the Medicare levy,
// depreciation and contribution caps. The input schema fixes or bounds the new periods
// and omits the new inputs, so the server advertises and reaches none of them and serves
// the 0.2.0 scope words. The sources, check dates and passages below are what still
// reaches a reply.
/// b4a70733 (#551), re-read on 14 November 2026: the PAYG source page and passage.
const PAYG_PAGE: &str =
    "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026";
const PAYG_PASSAGE: &str = "Schedule 1 (NAT 1004) for payments from 1 July 2026, and for payments from 1 July 2024 to 30 June 2026: weekly coefficients and scale 4, using a formula (rounding and the 53 and 27 pay tables), working out the weekly earnings (quarterly pays and tax offsets), Medicare levy adjustment, examples of calculating the withholding amounts, sample data";
/// d8f34dc2 (#567): the FBT source page and passage.
const FBT_PAGE: &str =
    "https://www.ato.gov.au/tax-rates-and-codes/fringe-benefits-tax-rates-and-thresholds";
const FBT_PASSAGE: &str = "; FBT rates and thresholds, the FBT rate and gross-up rates for years ending 31 March 2023 to 2027";
const PAYG_PAGES: [&str; 1] = [PAYG_PAGE];
const FBT_PAGES: [&str; 1] = [FBT_PAGE];
const STUDY_PAGES: [&str; 2] = [
    "https://www.legislation.gov.au/F2025L01016/asmade/text",
    "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026",
];
const STUDY_PASSAGE: &str = "Schedule 8 (NAT 3539) for payments from 1 July 2026, and Schedule 8 of the Taxation Administration (Withholding Schedules) Instrument (No. 2) 2025 (F2025L01016) for payments from 24 September 2025: study and training support loans component rates, calculating fortnightly, monthly and quarterly amounts, examples 1 to 3, rounding";
const MEDICARE_PAGES: [&str; 8] = [
    "https://www.legislation.gov.au/C2026A00058/asmade/text",
    "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy/medicare-levy-reduction/medicare-levy-reduction-family-income",
    "https://www.legislation.gov.au/C2004A03351/2025-03-28/2025-03-28/text/original/epub/OEBPS/document_1/document_1.html",
    "https://www.legislation.gov.au/C2025A00028/asmade/text",
    "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy/medicare-levy-exemption/medical-exemption-from-medicare-levy",
    "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy/medicare-levy-exemption/not-entitled-to-medicare-benefits",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19360027/251U",
    "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/medicare-levy-questions-m1-m2-individual-tax-return-2026/m1-medicare-levy-reduction-or-exemption-2026",
];
const MEDICARE_PASSAGE: &str = "Medicare Levy Act 1986 ss 3(1) and (3), 6(1), 7(1) and (2) and 8, compilations 52 (2024-25) and 53; Treasury Laws Amendment (More Cost of Living Relief) Act 2025 Sch 2 item 14 (2024-25 and later years); Treasury Laws Amendment (Delivering an Efficient and Trusted Tax System) Act 2026 Sch 5 item 14 (2025-26 and later years); ATO Medicare levy reduction for low-income earners and Medicare levy reduction - family income, with the Ashton example (2025-26); myTax 2025 tables 2 and 3 and worksheet 2 (2024-25); exemptions: Medicare Levy Act 1986 s 9(1), ITAA 1936 ss 251T and 251U(1) to (3), ATO Medical exemption from Medicare levy (the Paul example), Not entitled to Medicare benefits (the Priya example) and M1 Medicare levy reduction or exemption 2026, part B; non-resident days (read 24 November 2026): ITAA 1936 ss 251S(1)(a) and 251U(1)(d), (2) and (3), and M1 2026, category 2";
const DEPRECIATION_PAGES: [&str; 3] = [
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-70",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-72",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-75",
];
const DEPRECIATION_PASSAGE: &str = "Decline in value: diminishing value and prime cost methods, with the later-year diminishing value example; taxable use; ITAA 1997 ss 40-70(1), 40-72 and 40-75(1) and (7), base value and the limit on decline in a later year; Guides to depreciating assets 2021 to 2026, the methods and days held in a leap year, with Laura's photocopier in the 2021 and 2024 guides and Paul's refrigerator in the 2023 guide; IT(TP)A 1997 Subdivisions 40-BA and 40-BB, flagged";
const CAPS_PAGES: [&str; 2] = [
    "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/caps-limits-and-tax-on-super-contributions/concessional-contributions-cap",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/291-20",
];
const CAPS_PASSAGE: &str = "Contributions caps, tables 1.1, 2 and 4 and unused concessional cap carry forward; Concessional contributions cap, carry forward unused contribution cap amounts and the Sam example; ITAA 1997 s 291-20(1) to (7), compilation 268; Non-concessional contributions cap, bring-forward arrangement";

struct Review {
    pages: &'static [&'static str],
    checked: &'static str,
    passage: &'static str,
}

fn review(kind: &str) -> Option<Review> {
    Some(match kind {
        "payg_withholding" => Review {
            pages: &PAYG_PAGES,
            checked: "2026-11-14",
            passage: PAYG_PASSAGE,
        },
        "fbt" => Review {
            pages: &FBT_PAGES,
            checked: "2026-11-04",
            passage: "",
        },
        "study_loan_withholding" => Review {
            pages: &STUDY_PAGES,
            checked: "2026-11-14",
            passage: STUDY_PASSAGE,
        },
        "medicare_levy" => Review {
            pages: &MEDICARE_PAGES,
            checked: "2026-11-14",
            passage: MEDICARE_PASSAGE,
        },
        "depreciation" => Review {
            pages: &DEPRECIATION_PAGES,
            checked: "2026-11-23",
            passage: DEPRECIATION_PASSAGE,
        },
        "contribution_caps" => Review {
            pages: &CAPS_PAGES,
            checked: "2026-11-16",
            passage: CAPS_PASSAGE,
        },
        _ => return None,
    })
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

/// Exact results for the three successful FBT inputs in the frozen MCP corpus,
/// computed separately with the s 5B(1B) and (1C) ratios 57,000/27,401 and 100/53.
fn statutory_fbt(facts: &Value, amounts: &mut Value) {
    let values = match (
        facts["type_one_value"].as_str(),
        facts["type_two_value"].as_str(),
    ) {
        (Some("16500"), Some("6000")) => [
            "34323.56", "11320.75", "21452.83", "34324.00", "11321.00", "45645.00", "21453.15",
        ],
        (Some("1000.00"), Some("2000.00")) => [
            "2080.22", "3773.58", "2751.29", "2080.00", "3774.00", "5854.00", "2751.38",
        ],
        (Some("1000.00"), Some("1000.00")) => [
            "2080.22", "1886.79", "1864.49", "2080.00", "1887.00", "3967.00", "1864.49",
        ],
        _ => panic!("independently verify the statutory FBT amounts for {facts}"),
    };
    let fields = [
        "type_one_grossed_up",
        "type_two_grossed_up",
        "fbt_estimate",
        "return_item_14a",
        "return_item_14b",
        "return_item_15",
        "return_item_16",
    ];
    for (field, value) in fields.into_iter().zip(values) {
        amounts[field] = json!(value);
    }
}

/// A frozen disclaimer, with the worksheet scopes the linked engine states: the
/// statutory FBT wording and #837's resident_tax scope.
pub fn disclaimer(text: &str) -> String {
    let mut text = statutory_fbt_text(text);
    let label = "\nresident_tax: ";
    if let Some(start) = text.find(label) {
        let from = start + label.len();
        let end = from + text[from..].find('\n').unwrap();
        let mut scope = json!({});
        resident_tax(&mut scope, &["scope"]);
        text.replace_range(from..end, scope["scope"].as_str().unwrap());
    }
    text
}

pub fn statutory_fbt_text(text: &str) -> String {
    text.replace(
        "for the year ended 31 March 2026. Excludes",
        "for the year ended 31 March 2026 at the 47% FBT rate. Uses the formulas in FBTAA s 5B(1B) and (1C) for gross-up. Excludes",
    )
}

/// #837: resident_tax also pro-rates a part-year resident's tax-free threshold, so its
/// scope, check date and source passage are the linked engine's, as the engine's own
/// `adapt_resident_tax` in tests/parity.rs applies them. Without the new inputs every
/// amount is the frozen one.
fn resident_tax(value: &mut Value, fields: &[&str]) {
    let now = &australian_tax_calculators::engine::worksheet_catalogue()["resident_tax"];
    for field in fields {
        value[*field] = now[*field].clone();
    }
}

/// A frozen successful calculate_tax_worksheet result, as the linked engine gives it.
pub fn worksheet(kind: &str, facts: &Value, result: &mut Value) {
    if kind == "resident_tax" {
        resident_tax(result, &["scope", "source_checked"]);
    }
    if kind == "contribution_caps" && result["amounts"].get("carry_forward_applied").is_some() {
        carry_forward(facts, result);
    }
    if kind == "fbt" {
        statutory_fbt(facts, &mut result["amounts"]);
        result["scope"] = json!(statutory_fbt_text(result["scope"].as_str().unwrap()));
    }
    let Some(review) = review(kind) else {
        return;
    };
    let sources = result["sources"].as_array_mut().unwrap();
    sources.extend(review.pages.iter().map(|page| json!(page)));
    result["source_checked"] = json!(review.checked);
}

/// A frozen worksheet catalogue entry (the scope resource), as the linked engine gives it.
pub fn catalogue(kind: &str, entry: &mut Value) {
    if kind == "resident_tax" {
        resident_tax(entry, &["scope", "source_checked", "source_passage"]);
    }
    let Some(review) = review(kind) else {
        return;
    };
    entry["sources"]
        .as_array_mut()
        .unwrap()
        .extend(review.pages.iter().map(|page| json!(page)));
    entry["source_checked"] = json!(review.checked);
    let passage = entry["source_passage"].as_str().unwrap();
    entry["source_passage"] = json!(if kind == "fbt" {
        format!("{passage}{FBT_PASSAGE}")
    } else {
        review.passage.to_owned()
    });
    if kind == "fbt" {
        entry["scope"] = json!(statutory_fbt_text(entry["scope"].as_str().unwrap()));
    }
    let evidence = &mut entry["example_evidence"];
    match kind {
        "payg_withholding" => {
            evidence["document"] = json!("ATO Schedule 1 sample data");
            evidence["paragraphs"] = json!(
                "withholding amounts, Medicare levy adjustment scale 2 and Medicare half-levy adjustment scale 6: weekly, fortnightly and monthly tables"
            );
            evidence["checked"] = json!(review.checked);
            entry["required_inputs"]["pay_period"] =
                json!("weekly, fortnightly, monthly or quarterly");
            entry["required_inputs"]["scale"] =
                json!("integer 1 to 6; scale 4 also needs foreign_resident");
        }
        "study_loan_withholding" => {
            evidence["document"] = json!(
                "ATO Schedule 8 statement of formulas for STSL components, and F2025L01016 Schedule 8 for 2025-26"
            );
            evidence["paragraphs"] = json!("component rates, examples 1 to 3 in each");
            evidence["checked"] = json!(review.checked);
            entry["required_inputs"]["pay_period"] =
                json!("weekly, fortnightly, monthly or quarterly");
        }
        "medicare_levy" => {
            evidence["document"] = json!(
                "ATO Medicare levy reduction for low-income earners; Medicare levy reduction - family income; Not entitled to Medicare benefits"
            );
            evidence["paragraphs"] = json!(
                "table of single thresholds, example (Angie); family taxable income thresholds, example (Ashton); claiming an exemption, example (Priya)"
            );
            evidence["checked"] = json!(review.checked);
        }
        "depreciation" => {
            entry["required_inputs"]["days"] = json!("integer from 0 to 365, or to 366 in 2023-24");
        }
        _ => {}
    }
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
    if kind == "resident_tax" {
        resident_tax(row, &["source_checked"]);
        resident_tax(&mut row["provenance"], &["source_passage"]);
    }
    let Some(review) = review(&kind) else {
        return;
    };
    row["sources"]
        .as_array_mut()
        .unwrap()
        .extend(review.pages.iter().map(|page| json!(page)));
    row["source_checked"] = json!(review.checked);
    let passage = row["provenance"]["source_passage"].as_str().unwrap();
    row["provenance"]["source_passage"] = json!(if kind == "fbt" {
        format!("{passage}{FBT_PASSAGE}")
    } else {
        review.passage.to_owned()
    });
}

/// A frozen Python file note, restating the adapted result `after` where `before` held
/// Python's: each source line and its check date, changed table rows and the catalogue
/// passage. The note's layout is Python's.
pub fn file_note(kind: &str, note: &str, before: &Value, after: &Value) -> String {
    let mut note = if kind == "fbt" {
        statutory_fbt_text(note)
    } else {
        note.to_owned()
    };
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
    if kind == "resident_tax" {
        let now = &australian_tax_calculators::engine::worksheet_catalogue()["resident_tax"];
        for (old, new) in [
            (
                before["scope"].as_str().unwrap(),
                now["scope"].as_str().unwrap(),
            ),
            (
                "Income Tax Rates Act 1986, Schedule 7 Part I clause 1.",
                &format!("{}.", now["source_passage"].as_str().unwrap()),
            ),
        ] {
            note = note.replace(old, new);
        }
    }
    if let Some(review) = review(kind)
        && let Some(start) = note.find(covered)
    {
        let from = start + covered.len();
        let end = from + note[from..].find(".\n").unwrap();
        let passage = &note[from..end];
        let replacement = if kind == "fbt" {
            format!("{passage}{FBT_PASSAGE}")
        } else {
            review.passage.to_owned()
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

/// The bundled business-day note after the 14 November 2026 reading of the WA, TAS and ACT pages.
const CALENDAR_BEFORE: &str = "The WA Government says 2028 dates will be published when confirmed, and the Public and Bank Holidays Amendment Bill 2025 remains at Legislative Council second reading.";
const CALENDAR_AFTER: &str = "On 14 November 2026 the WA page (last updated 5 November 2026) gave the 2028 dates the Public and Bank Holiday Amendment Act 2026 sets: Labour Day on 13 March, Easter Saturday, King's Birthday on 12 June and Western Australia Day on 13 November; Show Day is left out because some regions observe another day instead. The same day the TAS page (last updated 23 January 2026) listed Easter Sunday statewide for 2027 and 2028, and the ACT page listed ANZAC Day 2027 on Sunday 25 and Monday 26 April.";

pub fn calendar_note(text: &str) -> String {
    text.replace(CALENDAR_BEFORE, CALENDAR_AFTER)
}

/// Every string of a frozen payday result, as the linked engine words it.
pub fn payday(value: &mut Value) {
    match value {
        Value::String(text) => *text = calendar_note(&payday_hint(text)),
        Value::Array(items) => items.iter_mut().for_each(payday),
        Value::Object(object) => object.values_mut().for_each(payday),
        _ => {}
    }
}

/// A computed native repayment records when the agreed loan term was not supplied.
const TERM_NOT_CHECKED: &str = "maximum_term_years was not supplied, so this engine has not checked the remaining term against the loan's term or that term less the years since the loan year (s 109E(6)). A remaining term that is too long gives a lower minimum yearly repayment than the Act requires.";

pub fn div7a(arguments: &Value, result: &mut Value) {
    let repayment = &mut result["minimum_yearly_repayment"];
    if arguments["maximum_term_years"].is_null() && repayment["myr_required"].is_string() {
        repayment["caveats"]
            .as_array_mut()
            .unwrap()
            .push(json!(TERM_NOT_CHECKED));
    }
}

/// The coding review counts an invalid supplier ABN. Frozen lines have none, so the count is nil.
pub fn gst(result: &mut Value) {
    let Some(counts) = result.get_mut("counts").and_then(Value::as_object_mut) else {
        return;
    };
    if counts.contains_key("supplier_abn_invalid") {
        return;
    }
    let mut next = serde_json::Map::new();
    for (key, value) in counts.iter() {
        next.insert(key.clone(), value.clone());
        if key == "gst_differs_from_registration" {
            next.insert("supplier_abn_invalid".into(), json!(0));
        }
    }
    *counts = next;
}

/// The two Division 296 rate records added after the frozen list.
const ADDED_RATE_IDS: [&str; 2] = [
    "division-296-tax-rate",
    "division-296-very-large-balance-tax-rate",
];

pub fn reviewed_rates(value: &mut Value) {
    match value {
        Value::Object(object) => {
            if let Some(ids) = object.get_mut("known_ids").and_then(Value::as_array_mut)
                && ids.len() == 72
            {
                for id in ADDED_RATE_IDS {
                    let at = ids
                        .iter()
                        .position(|item| item.as_str().unwrap() > id)
                        .unwrap_or(ids.len());
                    ids.insert(at, json!(id));
                }
            }
            for (key, child) in object.iter_mut() {
                if key != "known_ids" {
                    reviewed_rates(child);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(reviewed_rates),
        _ => {}
    }
}

pub fn rate_records(row: &mut Value) {
    if row["dataset"] == "reviewed_ato_rates" && row["provenance"]["records"] == 72 {
        row["provenance"]["records"] = json!(74);
    }
}
