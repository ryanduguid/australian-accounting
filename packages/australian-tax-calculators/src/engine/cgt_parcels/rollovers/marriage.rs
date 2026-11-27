// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Subdivision 126-A: a CGT event between spouses or former spouses because
//! of a relationship breakdown (ss 126-5, 126-15 and 126-25).

use super::*;

/// The grounds of s 126-5(1) and 126-15(1); the last 3 need the conditions
/// of s 126-25 (ss 126-5(3A) and 126-15(5)).
const BASES: [&str; 6] = [
    "court_order",
    "maintenance_agreement",
    "arbitral_award",
    "financial_agreement",
    "part_viiiab_financial_agreement",
    "state_law_agreement",
];

const EVENTS: [&str; 6] = ["A1", "B1", "D1", "D2", "D3", "F1"];

const TRANSFEREE: &str = "ITAA 1997 s 126-5(5) (and s 126-15(1)): the first element of the transferee's cost base and reduced cost base is the transferor's cost base and reduced cost base when the transferee acquired the asset";
const CREATED: &str = "ITAA 1997 s 126-5(8) (and s 126-15(1)): the first element of the created asset's cost base and reduced cost base is the transferor's expenditure or incidental costs the table names for the event";
const COMPANY: &str = "The transferor is a company or trustee: s 126-15(2) to (4) reduce the cost base and reduced cost base of shares, units, loans and indirect interests in it by the fall in their market value the trigger event causes, which this worksheet does not work out.";
const RESIDENCE: &str = "Where the asset is a dwelling, ss 118-178 and 118-180 decide how the transferor's use counts towards the transferee's main residence exemption; this worksheet does not apply them.";

/// The checks of ss 126-5(1), (3) and (3A) and 126-25 on the stated facts.
fn available(f: &mut Fields, event: &str) -> Result<()> {
    let basis = choice(f, "basis", &BASES)?;
    if BASES[3..].contains(&basis) && !yes(f, "separated_and_breakdown_connected")? {
        return no_rollover(
            "There is no roll-over for something done under a financial agreement or a binding written agreement unless the spouses were separated with no reasonable likelihood of resuming cohabitation and the event happened because of reasons directly connected with the breakdown (ITAA 1997 ss 126-5(3A), 126-15(5) and 126-25).",
        );
    }
    if matches!(event, "A1" | "B1") && yes(f, "trading_stock")? {
        return no_rollover(
            "There is no roll-over for the transferor's trading stock (ITAA 1997 s 126-5(3)(a)).",
        );
    }
    if event == "B1" && !yes(f, "title_passes")? {
        return no_rollover(
            "There is no roll-over for CGT event B1 where title does not pass to the transferee at or before the end of the agreement (ITAA 1997 s 126-5(3)(b)).",
        );
    }
    Ok(())
}

/// A disposal case (A1 or B1): the transferor's bases carried.
fn disposal(f: &mut Fields, day: i64, extra: &mut Map<String, Value>) -> Result<Outcome> {
    let acquired = f.date("transferor_acquired")?;
    if acquired > day {
        return conflict("transferor_acquired must be on or before event_date.".into());
    }
    if pre_cgt(acquired) {
        return refuse(
            "cgt_rollover.pre_cgt",
            "The transferor acquired the asset before 20 September 1985, so the transferee is taken to have acquired it before that day (ITAA 1997 s 126-5(6)) and no cost base carries; this worksheet has nothing to work out.",
        );
    }
    let cost = f.amount("cost_base", false)?;
    let reduced = f.amount("reduced_cost_base", false)?;
    let class = choice(
        f,
        "asset_class",
        &["ordinary", "collectable", "personal_use"],
    );
    let class = class?;
    let mut amounts = vec![
        ("cost_base", cost.clone()),
        ("reduced_cost_base", reduced.clone()),
    ];
    if f.get("capital_proceeds").is_some() {
        let proceeds = f.amount("capital_proceeds", false)?;
        let [gain, loss] = disregarded(&proceeds, [&cost, &reduced])?;
        amounts.push(("disregarded_capital_gain", gain));
        amounts.push(("disregarded_capital_loss", loss));
    }
    extra.insert("authority".into(), json!(TRANSFEREE));
    extra.insert(
        "acquired_for_discount".into(),
        json!(dates::format(acquired)),
    );
    extra.insert("asset_class".into(), json!(class));
    let mut warnings = vec![json!(RESIDENCE)];
    if acquired <= dates::parse(INDEXATION_LAST_DAY).expect("a date") {
        warnings.push(json!(INDEXED));
    }
    extra.insert("warnings".into(), Value::Array(warnings));
    Ok((amounts, std::mem::take(extra)))
}

/// A creation case (D1, D2, D3 or F1): the transferor's outlay.
fn creation(f: &mut Fields, day: i64, extra: &mut Map<String, Value>) -> Result<Outcome> {
    let outlay = f.amount("expenditure", false)?;
    extra.insert("authority".into(), json!(CREATED));
    extra.insert("acquired_for_discount".into(), json!(dates::format(day)));
    extra.insert("asset_class".into(), json!("ordinary"));
    extra.insert("warnings".into(), json!([]));
    let amounts = vec![("cost_base", outlay.clone()), ("reduced_cost_base", outlay)];
    Ok((amounts, std::mem::take(extra)))
}

/// Subdivision 126-A on the stated facts.
pub(super) fn work(f: &mut Fields, year: &str) -> Result<Outcome> {
    let transferor = choice(f, "transferor", &["individual", "company", "trustee"])?;
    let event = choice(f, "event", &EVENTS)?;
    let day = in_year(f, "event_date", year)?;
    available(f, event)?;
    let mut extra = Map::new();
    extra.insert("transferor".into(), json!(transferor));
    extra.insert("event".into(), json!(event));
    extra.insert("acquired".into(), json!(dates::format(day)));
    let (amounts, mut extra) = if matches!(event, "A1" | "B1") {
        disposal(f, day, &mut extra)?
    } else {
        creation(f, day, &mut extra)?
    };
    if transferor != "individual"
        && let Some(Value::Array(w)) = extra.get_mut("warnings")
    {
        w.push(json!(COMPANY));
    }
    Ok((amounts, extra))
}
