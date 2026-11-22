// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `cgt_cost_base_adjustments`: the cost base and reduced cost base of
//! parcels of shares or units after a company's returns of capital (CGT
//! event G1, ITAA 1997 s 104-135), a non-AMIT trust's non-assessable
//! payments (CGT event E4, ss 104-70 and 104-71), an AMIT's cost base net
//! amounts (ss 104-107A to 104-107C, CGT event E10) and the demergers in
//! `DEMERGERS` (Subdivision 125-B), with the capital gains they make. The
//! provisions were read on the ATO's legal database on 22 November 2026; their
//! history notes show none changed since s 104-107A took its form for 2018-19
//! (Act No. 15 of 2019), the first year worked. The parcels it returns are
//! `cgt_parcels`' own, so they can be sold there.

use super::events::{Event, read_event};
use super::steps::total;
use super::work::{Row, Work, bases};
use super::*;

const KIND: &str = "cgt_cost_base_adjustments";

/// The first day an event is worked: s 104-107A applies in its current form
/// from the 2018-19 income year (No 15 of 2019, Sch 1 items 3 to 5).
const FIRST_DAY: &str = "2018-07-01";

const UNINDEXED: &str = "A parcel acquired at or before 21 September 1999 is worked without indexation, the cost base the discount method uses (the ATO's Guide to capital gains tax 2025, example 45); a taxpayer who will choose indexation (ITAA 1997 s 114-1) works the indexed cost base instead, and cgt-parcels refuses these parcels.";

const DISCOUNT_GAINS: &str = "ITAA 1997 ss 104-135(3), 104-70(4) and 104-107A(3) with s 115-25(1): the year's capital gains from the events on shares or units acquired at least 12 months before, by an entity that can make a discount capital gain (cgt-parcels' other_discount_capital_gains)";
const OTHER_GAINS: &str = "ITAA 1997 ss 104-135(3), 104-70(4) and 104-107A(3): the year's other capital gains from the events (cgt-parcels' other_capital_gains)";
const TOTAL_GAINS: &str = "ITAA 1997 s 102-5(1) step 1: the capital gains from the events, before losses and the discount";
const HELD_COST_BASE: &str =
    "ITAA 1997 s 110-25: the parcels' cost bases at the end of the year, after the events";
const HELD_REDUCED: &str =
    "ITAA 1997 s 110-55: the parcels' reduced cost bases at the end of the year, after the events";

/// The facts as the command passes them; see the README for each.
pub struct CgtCostBaseArgs<'a> {
    pub entity: &'a Arg,
    pub parcels: &'a Arg,
    pub events: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The events in time order, each from 2018-19 to the year's end, and no
/// security both a company's shares and a trust's units.
fn events_of(args: &CgtCostBaseArgs, year: &str, last: i64) -> Result<Vec<Event>> {
    let mut events = objects(args.events, "events")?
        .into_iter()
        .enumerate()
        .map(|(i, fields)| read_event(i + 1, fields))
        .collect::<Result<Vec<_>>>()?;
    for (k, e) in events.iter().enumerate() {
        if e.day < dates::parse(FIRST_DAY).expect("a date") {
            return refuse(
                "cgt_cost_base.event_year_unsupported",
                format!(
                    "event {}: it is before 1 July 2018; ITAA 1997 s 104-107A took its present form for 2018-19 (Act No. 15 of 2019) and earlier law was not read.",
                    e.n
                ),
            );
        }
        if e.day > last {
            return conflict(format!(
                "event {}: it is after the {year} income year ended.",
                e.n
            ));
        }
        if events[..k]
            .iter()
            .any(|o| o.security == e.security && o.change.company() != e.change.company())
        {
            return conflict(format!(
                "event {}: {} cannot be both a company's shares and a trust's units.",
                e.n, e.security
            ));
        }
    }
    events.sort_by_key(|e| (e.day, e.n));
    Ok(events)
}

fn work(args: &CgtCostBaseArgs, year: &str) -> Result<Work> {
    let start = year_first(year).expect("a supported income year");
    let first = dates::parse(&format!("{start}-07-01")).expect("a date");
    let last = dates::add_months(first, 12) - 1;
    let company = match ENTITIES
        .iter()
        .find(|(name, _)| Some(*name) == args.entity.text())
    {
        Some((name, _)) => *name == "company",
        None => {
            return refuse(
                "cgt_parcels.entity_invalid",
                "entity must be individual, trust, complying_superannuation_entity or company.",
            );
        }
    };
    let parcels = objects(args.parcels, "parcels")?
        .into_iter()
        .enumerate()
        .map(|(i, fields)| read_parcel(i + 1, fields))
        .collect::<Result<Vec<_>>>()?;
    for (i, parcel) in parcels.iter().enumerate() {
        if parcels[..i].iter().any(|p| p.id == parcel.id) {
            return conflict(format!(
                "parcel {}: the name {} is used twice.",
                i + 1,
                parcel.id
            ));
        }
    }
    let mut work = Work {
        parcels,
        rows: Vec::new(),
        company,
        first,
        last,
        unindexed: false,
        trust_years: Vec::new(),
    };
    for event in events_of(args, year, last)? {
        work.apply(&event)?;
    }
    Ok(work)
}

fn amount_steps(w: &Work) -> Result<Vec<Step>> {
    let gains = |discount: bool| -> Vec<&Decimal> {
        w.rows
            .iter()
            .filter(|r| r.counted && r.discount == discount && !r.gain.is_zero())
            .map(|r| &r.gain)
            .collect()
    };
    let held: Vec<[Decimal; 2]> = w.parcels.iter().map(bases).collect::<Result<_>>()?;
    let column = |k: usize| -> Vec<&Decimal> { held.iter().map(|b| &b[k]).collect() };
    let (fd, dg) = total(&gains(true))?;
    let (fo, og) = total(&gains(false))?;
    let (fc, cb) = total(&column(0))?;
    let (fr, rcb) = total(&column(1))?;
    Ok(vec![
        step(
            "discount_capital_gains",
            DISCOUNT_GAINS,
            &fd,
            &[],
            dg.clone(),
        ),
        step("other_capital_gains", OTHER_GAINS, &fo, &[], og.clone()),
        step(
            "total_capital_gains",
            TOTAL_GAINS,
            "discount_capital_gains + other_capital_gains",
            &[&dg, &og],
            dg.add(&og)?,
        ),
        step("cost_base_held", HELD_COST_BASE, &fc, &[], cb),
        step("reduced_cost_base_held", HELD_REDUCED, &fr, &[], rcb),
    ])
}

fn row_json(w: &Work, r: &Row) -> Result<Value> {
    let p = &w.parcels[r.parcel];
    let money = |v: &Decimal| -> Result<Value> { Ok(json!(cents(v.clone())?.to_string())) };
    let discount = if r.gain.is_zero() {
        Value::Null
    } else {
        json!(r.discount)
    };
    Ok(json!({
        "event": r.event,
        "change": r.change,
        "parcel": p.id,
        "security": p.security,
        "date": dates::format(r.day),
        "income_year": income_year(r.day),
        "counted": r.counted,
        "amount": match &r.amount { Some(a) => money(a)?, None => Value::Null },
        "cost_base_before": money(&r.before[0])?,
        "reduced_cost_base_before": money(&r.before[1])?,
        "cost_base_after": money(&r.after[0])?,
        "reduced_cost_base_after": money(&r.after[1])?,
        "capital_gain": money(&r.gain)?,
        "discount_capital_gain": discount,
        "authority": r.authority,
    }))
}

/// A parcel as `cgt_parcels` takes it: the reduced cost base as its
/// acquisition cost and the rest of the cost base as costs of owning.
fn parcel_json(p: &Parcel) -> Result<Value> {
    Ok(json!({
        "parcel": p.id,
        "security": p.security,
        "acquired": dates::format(p.acquired),
        "units": p.units.fixed_format(),
        "acquisition_cost": cents(p.base.clone())?.to_string(),
        "incidental_costs": "0.00",
        "ownership_costs": cents(p.owning.clone())?.to_string(),
    }))
}

pub fn cgt_cost_base_adjustments(args: &CgtCostBaseArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let (w, steps) = precise(|| {
        let w = work(args, &year)?;
        let steps = amount_steps(&w)?;
        Ok((w, steps))
    })?;
    let mut out = derived_result(KIND, &year, steps, Vec::new())?;
    out["adjustments"] = Value::Array(
        w.rows
            .iter()
            .map(|r| row_json(&w, r))
            .collect::<Result<_>>()?,
    );
    out["parcels"] = Value::Array(w.parcels.iter().map(parcel_json).collect::<Result<_>>()?);
    if w.unindexed {
        let warnings = out["warnings"].as_array_mut().expect("warnings");
        warnings.push(json!(UNINDEXED));
    }
    Ok(out)
}
