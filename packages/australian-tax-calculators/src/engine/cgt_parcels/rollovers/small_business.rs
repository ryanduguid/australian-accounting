// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Subdivision 152-E: the gain disregarded (ss 152-410 and 152-415), the
//! replacement asset period (s 104-190(1A)) and CGT event J5 or J6 at its end
//! (ss 104-197 and 104-198) for the replacement assets stated.

use super::super::super::sb_cgt_concessions::replacement_period;
use super::*;

const PENDING: &str = "No replacement_assets were stated: CGT event J5 or J6 is worked out at the end of the replacement asset period (ITAA 1997 ss 104-197(3) and 104-198(2)) from the replacement assets acquired and fourth element expenditure incurred by then.";
const KEPT: &str = "The small business roll-over does not reduce a replacement asset's cost base; a later change in its use can cause CGT event J2 (ITAA 1997 s 104-185), which this worksheet does not work out.";
const NOT_DISCOUNT: &str = "A capital gain from CGT event J5 or J6 is not a discount capital gain (ITAA 1997 s 115-25(3)(hb) and (hc)).";
const PERIOD: &str = "The Commissioner can extend the replacement asset period, and a later increase in capital proceeds under s 116-45(2) or 116-60(3) modifies it (ITAA 1997 s 104-190(1) and (2)); this worksheet uses the period s 104-190(1A) sets.";

/// One replacement asset or lot of fourth element expenditure.
struct Replacement {
    label: String,
    day: i64,
    incurred: Decimal,
    qualifies: bool,
}

fn replacement(n: usize, fields: &Map<String, Value>, period: (i64, i64)) -> Result<Replacement> {
    let mut f = Fields::new(format!("replacement asset {n}"), fields);
    let label = f.text("label")?;
    let day = f.date("date")?;
    let mut incurred = f.amount("first_element_cost_base", false)?;
    incurred = incurred.add(&f.amount("incidental_costs", true)?)?;
    incurred = incurred.add(&f.amount("fourth_element_expenditure", true)?)?;
    let qualifies = yes(&mut f, "qualifies")?;
    f.none_left()?;
    if !(period.0..=period.1).contains(&day) {
        return conflict(format!(
            "replacement asset {n}: date must be in the replacement asset period, {} to {} (ITAA 1997 s 104-190(1A)).",
            dates::format(period.0),
            dates::format(period.1)
        ));
    }
    Ok(Replacement {
        label,
        day,
        incurred,
        qualifies,
    })
}

/// CGT event J5 or J6 at the end of the period, or none, with the amount
/// incurred on the qualifying replacement assets and the gain.
fn at_end(
    all: &[Replacement],
    rolled: &Decimal,
) -> Result<(Option<&'static str>, Decimal, Decimal)> {
    let mut incurred = int(0);
    for r in all.iter().filter(|r| r.qualifies) {
        incurred = incurred.add(&r.incurred)?;
    }
    if !all.iter().any(|r| r.qualifies) {
        return Ok((Some("J5"), incurred, rolled.clone()));
    }
    if incurred.lt(rolled)? {
        return Ok((Some("J6"), incurred.clone(), rolled.sub(&incurred)?));
    }
    Ok((None, incurred, int(0)))
}

fn shown(all: &[Replacement]) -> Value {
    Value::Array(
        all.iter()
            .map(|r| {
                json!({
                    "label": r.label,
                    "date": dates::format(r.day),
                    "amount_incurred": r.incurred.to_string(),
                    "qualifies": r.qualifies,
                })
            })
            .collect(),
    )
}

/// The gain stated and the part chosen to be disregarded.
fn chosen(f: &mut Fields) -> Result<(Decimal, Decimal)> {
    if !yes(f, "basic_conditions_met")? {
        return no_rollover(
            "The small business roll-over needs the basic conditions in Subdivision 152-A for the gain (ITAA 1997 s 152-410).",
        );
    }
    let gain = f.amount("capital_gain", false)?;
    let rolled = f.amount("disregarded", false)?;
    if rolled.gt(&gain)? || rolled.is_zero() {
        return conflict(
            "disregarded must be above nil and at most capital_gain: the roll-over disregards all or part of the gain (ITAA 1997 s 152-415)."
                .into(),
        );
    }
    Ok((gain, rolled))
}

/// Subdivision 152-E on the stated facts.
pub(super) fn work(f: &mut Fields, year: &str) -> Result<Outcome> {
    let (gain, rolled) = chosen(f)?;
    let last = in_year(f, "last_cgt_event", year)?;
    let earnout = match f.get("earnout_latest_due") {
        None => None,
        Some(_) => Some(f.date("earnout_latest_due")?),
    };
    if earnout.is_some_and(|due| due < last) {
        return conflict("earnout_latest_due must be on or after last_cgt_event.".into());
    }
    let period = replacement_period(last, earnout);
    let mut amounts = vec![
        ("capital_gain", gain.clone()),
        ("disregarded_capital_gain", rolled.clone()),
        ("remaining_capital_gain", gain.sub(&rolled)?),
    ];
    let mut extra = Map::new();
    extra.insert(
        "replacement_asset_period".into(),
        json!({"from": dates::format(period.0), "to": dates::format(period.1)}),
    );
    let mut warnings = vec![json!(KEPT), json!(PERIOD)];
    match optional_objects(f, "replacement_assets")? {
        None => {
            warnings.push(json!(PENDING));
            extra.insert("cgt_event".into(), Value::Null);
        }
        Some(list) => {
            let all = list
                .iter()
                .enumerate()
                .map(|(i, o)| replacement(i + 1, o, period))
                .collect::<Result<Vec<_>>>()?;
            let (event, incurred, end_gain) = at_end(&all, &rolled)?;
            amounts.push(("amount_incurred", incurred));
            amounts.push(("capital_gain_at_period_end", end_gain));
            if event.is_some() {
                warnings.push(json!(NOT_DISCOUNT));
            }
            extra.insert("cgt_event".into(), json!(event));
            extra.insert("cgt_event_date".into(), json!(dates::format(period.1)));
            extra.insert("replacement_assets".into(), shown(&all));
        }
    }
    extra.insert("warnings".into(), Value::Array(warnings));
    Ok((amounts, extra))
}
