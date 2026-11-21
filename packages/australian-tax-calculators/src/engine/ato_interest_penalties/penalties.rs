// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The failure to lodge on time penalty and the statement penalties, each
//! before remission.

use serde_json::{Map, Value, json};

use super::{
    Facts, date_order, dollars, fact_invalid, flag, in_year, rates, remission_note, to_cents,
};
use crate::dates;
use crate::engine::{Result, rate, refuse, result, stated};

const KIND: &str = "ato_interest_penalties";

const FTL_BASE: &str = "TAA 1953 Sch 1 s 286-80(2)(a): 1 penalty unit for each 28 days or part of 28 days from the day the document is due to when it is given, at most 5 (counted from the due day to the day before it is given: PS LA 2011/19 para 16); s 286-80(5): each unit at the amount applying at the start of its 28-day period";
const FTL_SIZE: &str = "s 286-80(3): x2 for a medium entity (a medium withholder that month, or assessable income or current GST turnover more than $1 million and less than $20 million); s 286-80(4): x5 for a large entity ($20 million or more, or a large withholder); s 286-80(4A) and (4C): x500 for a significant global entity or a Pillar Two group entity, (3) and (4) then not applying; the size is a stated fact";
const UNIT: &str = "Crimes Act 1914 s 4AA: the penalty unit, $330 indexed from 1 July 2026; the ATO's 'Penalty units' (QC71196, last updated 26 June 2026): $364 from 1 July 2026, $330 from 7 November 2024, $313 from 1 July 2023, $275 from 1 January 2023, $222 from 1 July 2020, $210 from 1 July 2017, $180 from 31 July 2015, $170 from 28 December 2012";
const STATEMENT: &str = "TAA 1953 Sch 1 s 284-85: BPA + BPA x (increase % - reduction %); s 284-90(1) items 1 to 3: 75%, 50% or 25% of the shortfall amount for intentional disregard, recklessness or a failure to take reasonable care, items 3A to 3C: 60, 40 or 20 penalty units where there is no shortfall amount, item 4: 25% where a position not reasonably arguable gives a shortfall amount more than the reasonably arguable threshold (s 284-90(3)(a): the greater of $10,000 and 1% of the income tax payable on the return; each non-identical matter apart, MT 2008/2 paras 33 and 62); s 284-90(2): the greater item; s 284-224: less the part caused by treating the law in an accepted way (PS LA 2012/5 para 95); s 284-90(1A): doubled for a significant global entity; s 284-220: +20%; s 284-225: -80% (or to nil under $1,000, and to nil with no shortfall amount) for a voluntary disclosure before an examination is notified, the disclosures for the period added (MT 2012/3 para 35), or -20% for one after that saves significant time or resources";

const SIZES: [(&str, u128); 4] = [
    ("small", 1),
    ("medium", 2),
    ("large", 5),
    ("significant_global", 500),
];

pub(super) fn failure_to_lodge(f: &Facts, period: &str) -> Result<Value> {
    f.only(&["due_date", "given_on", "size"])?;
    let size = f.choice("size", &SIZES.map(|(s, _)| s))?;
    let multiplier = SIZES.iter().find(|(s, _)| *s == size).expect("a size").1;
    let (due, given) = (f.date("due_date")?, f.date("given_on")?);
    in_year(given, period, "given_on")?;
    if given < due {
        return date_order(format!(
            "{}: given_on must be on or after due_date.",
            f.item()
        ));
    }
    if rates::penalty_unit(due).is_none() {
        return refuse(
            "ato_interest_penalties.penalty_unit_unpublished",
            "The ATO's penalty unit table starts its values on 28 December 2012; an earlier due date is not worked.",
        );
    }
    if size == "significant_global" && due < dates::parse("2017-07-01").expect("date") {
        return refuse(
            "ato_interest_penalties.sge_before_2017",
            "The significant global entity penalty applies to documents due on or after 1 July 2017 (PS LA 2011/19 para 23).",
        );
    }
    let late = given - due;
    let count = ((late + 27) / 28).min(5);
    let units: Vec<(i64, u32)> = (0..count)
        .map(|k| {
            let start = due + 28 * k;
            (
                start,
                rates::penalty_unit(start).expect("a unit from the due date"),
            )
        })
        .collect();
    let base: u128 = units.iter().map(|(_, v)| u128::from(*v) * 100).sum();
    let mut out = result(
        KIND,
        period,
        vec![
            ("base_penalty_amount", dollars(base)),
            ("penalty", dollars(base * multiplier)),
        ],
        vec![rate("size_multiplier", multiplier)],
    )?;
    out["question"] = json!("failure_to_lodge");
    out["days_late"] = json!(late);
    out["penalty_units"] = json!(count);
    out["periods"] = Value::Array(
        units
            .iter()
            .map(|(start, value)| json!({"from": dates::format(*start), "penalty_unit": value.to_string()}))
            .collect(),
    );
    out["penalty_authority"] = json!(FTL_BASE);
    out["size_authority"] = json!(FTL_SIZE);
    out["unit_authority"] = json!(UNIT);
    remission_note(&mut out);
    Ok(out)
}

const BEHAVIOURS: [(&str, u128, u128); 4] = [
    ("intentional_disregard", 75, 60),
    ("recklessness", 50, 40),
    ("failure_to_take_reasonable_care", 25, 20),
    ("reasonable_care", 0, 0),
];
const DISCLOSURES: [&str; 3] = ["none", "before_examination", "during_examination"];
const PART_FIELDS: [&str; 6] = [
    "shortfall_amount",
    "behaviour",
    "not_reasonably_arguable",
    "accepted_way_amount",
    "increase",
    "disclosure",
];

/// A statement penalty part's stated facts.
struct Part {
    /// Cents, or `None` for a false or misleading statement with no shortfall.
    shortfall: Option<u64>,
    behaviour: &'static str,
    not_arguable: bool,
    accepted: u64,
    increase: bool,
    disclosure: &'static str,
}

fn optional_flag(fields: &Map<String, Value>, name: &str, key: &str) -> Result<bool> {
    match fields.get(key) {
        Some(value) => flag(value, name, key),
        None => Ok(false),
    }
}

fn part(item: &Value, name: &str) -> Result<Part> {
    let Value::Object(fields) = item else {
        return fact_invalid(format!("{name} must be an object."));
    };
    stated::known_keys(fields, name, "a part", &PART_FIELDS)?;
    let shortfall = match stated::required(fields, name, "shortfall_amount")? {
        Value::Null => None,
        _ => Some(to_cents(&stated::amount(
            fields,
            name,
            "shortfall_amount",
        )?)?),
    };
    let behaviour = super::choice(
        stated::required(fields, name, "behaviour")?,
        name,
        "behaviour",
        &BEHAVIOURS.map(|(b, _, _)| b),
    )?;
    let not_arguable = optional_flag(fields, name, "not_reasonably_arguable")?;
    let increase = optional_flag(fields, name, "increase")?;
    let disclosure = match fields.get("disclosure") {
        Some(value) => super::choice(value, name, "disclosure", &DISCLOSURES)?,
        None => "none",
    };
    let accepted = match fields.get("accepted_way_amount") {
        Some(_) => to_cents(&stated::amount(fields, name, "accepted_way_amount")?)?,
        None => 0,
    };
    match shortfall {
        None if not_arguable || fields.contains_key("accepted_way_amount") => {
            return fact_invalid(format!(
                "{name}: not_reasonably_arguable and accepted_way_amount need a shortfall_amount."
            ));
        }
        Some(cents) if accepted > cents => {
            return fact_invalid(format!(
                "{name}: accepted_way_amount cannot exceed shortfall_amount."
            ));
        }
        _ => {}
    }
    Ok(Part {
        shortfall,
        behaviour,
        not_arguable,
        accepted,
        increase,
        disclosure,
    })
}

/// The cents in `scaled` ten-thousandths of a cent, half up.
fn scaled_cents(scaled: u128) -> u128 {
    (scaled + 5_000) / 10_000
}

pub(super) fn statement_penalty(f: &Facts, period: &str) -> Result<Value> {
    f.only(&[
        "statement_date",
        "significant_global_entity",
        "trust_or_partnership",
        "income_tax_payable",
        "parts",
    ])?;
    let made = f.date("statement_date")?;
    in_year(made, period, "statement_date")?;
    let sge = f.flag("significant_global_entity")?;
    let parts: Vec<Part> = f
        .list("parts", 100)?
        .iter()
        .enumerate()
        .map(|(i, item)| part(item, &format!("part {}", i + 1)))
        .collect::<Result<_>>()?;
    let arguable = parts.iter().any(|p| p.not_arguable);
    let trust = f.given("trust_or_partnership") && f.flag("trust_or_partnership")?;
    if arguable && trust {
        return refuse(
            "ato_interest_penalties.item_not_worked",
            "A trust's or partnership's position that is not reasonably arguable (s 284-90(1) items 5 and 6, its threshold on net income) is not worked.",
        );
    }
    let tax = match (arguable, f.given("income_tax_payable")) {
        (true, _) => Some(f.cents("income_tax_payable")?),
        (false, true) => {
            return refuse(
                "input.field_unknown",
                format!(
                    "{}: income_tax_payable is an input only with a part that is not_reasonably_arguable.",
                    f.item()
                ),
            );
        }
        (false, false) => None,
    };
    // s 284-90(3)(a): the greater of $10,000 and 1% of the tax on the return.
    let threshold_cents = |tax: u64| u128::from(tax).max(100_000_000);
    let unit = rates::penalty_unit(made).expect("a unit from 2020-21");
    let disclosed_before: u64 = parts
        .iter()
        .filter(|p| p.disclosure == "before_examination")
        .filter_map(|p| p.shortfall)
        .sum();
    let (mut total, mut base_total, mut worked) = (0_u128, 0_u128, Vec::new());
    for (i, p) in parts.iter().enumerate() {
        let (_, percentage, units) = BEHAVIOURS
            .iter()
            .find(|(b, _, _)| *b == p.behaviour)
            .expect("a behaviour");
        let mut out = json!({"part": i + 1, "behaviour": p.behaviour});
        // BPA in ten-thousandths of a cent, so that a percentage of a cent and
        // then a percentage of that stay exact.
        let bpa = match p.shortfall {
            Some(cents) => {
                let over = tax.is_some_and(|t| u128::from(cents) * 100 > threshold_cents(t));
                let item4 = if p.not_arguable && over { 25 } else { 0 };
                let applied = (*percentage).max(item4);
                out["shortfall_amount"] = json!(dollars(u128::from(cents)).to_string());
                if p.accepted > 0 {
                    out["accepted_way_amount"] = json!(dollars(u128::from(p.accepted)).to_string());
                }
                if p.not_arguable {
                    out["above_reasonably_arguable_threshold"] = json!(over);
                }
                out["base_percentage"] = json!(applied.to_string());
                u128::from(cents - p.accepted) * applied * 100
            }
            None => {
                out["penalty_units"] = json!(units.to_string());
                out["penalty_unit"] = json!(unit.to_string());
                units * u128::from(unit) * 100 * 10_000
            }
        };
        let bpa = if sge { bpa * 2 } else { bpa };
        let increase: u128 = if p.increase { 20 } else { 0 };
        let reduction: Option<u128> = match (p.disclosure, p.shortfall) {
            ("before_examination", Some(_)) if disclosed_before >= 100_000 => Some(80),
            ("before_examination", _) => None,
            ("during_examination", _) => Some(20),
            _ => Some(0),
        };
        let penalty = match reduction {
            Some(cut) => bpa * (100 + increase - cut) / 100,
            None if increase > 0 => {
                return refuse(
                    "ato_interest_penalties.increase_with_nil",
                    format!(
                        "part {}: s 284-225(3)(b) and (4A) reduce the base penalty amount to nil, not by a percentage, and s 284-85 does not say how an increase then applies; this is not worked.",
                        i + 1
                    ),
                );
            }
            None => 0,
        };
        out["base_penalty_amount"] = json!(dollars(scaled_cents(bpa)).to_string());
        out["increase_percent"] = json!(increase.to_string());
        out["reduction"] = json!(match reduction {
            Some(cut) => format!("{cut}%"),
            None => "to nil".into(),
        });
        out["disclosure"] = json!(p.disclosure);
        out["penalty"] = json!(dollars(scaled_cents(penalty)).to_string());
        base_total += bpa;
        total += penalty;
        worked.push(out);
    }
    let mut rates_used = vec![rate("penalty_unit", unit)];
    if let Some(t) = tax {
        let threshold = threshold_cents(t);
        rates_used.push(rate(
            "reasonably_arguable_threshold",
            dollars(threshold.div_ceil(100)),
        ));
    }
    let mut out = result(
        KIND,
        period,
        vec![
            ("base_penalty_amount", dollars(scaled_cents(base_total))),
            ("penalty", dollars(scaled_cents(total))),
        ],
        rates_used,
    )?;
    out["question"] = json!("statement_penalty");
    out["parts"] = Value::Array(worked);
    out["penalty_authority"] = json!(STATEMENT);
    out["unit_authority"] = json!(UNIT);
    remission_note(&mut out);
    if sge {
        super::warn(
            &mut out,
            "Doubled for a significant global entity (s 284-90(1A)): stated, not tested against the trigger day.".into(),
        );
    }
    Ok(out)
}
