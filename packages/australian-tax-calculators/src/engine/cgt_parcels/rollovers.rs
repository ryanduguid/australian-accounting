// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `cgt_rollover`: what a chosen CGT roll-over carries, from stated facts: a
//! marriage or relationship breakdown (ITAA 1997 Subdivision 126-A), the
//! small business roll-over (Subdivision 152-E, with CGT events J5 and J6 at
//! the end of the replacement asset period) and a disposal to a wholly-owned
//! company (Subdivision 122-A). It gives the acquirer's or the replacement
//! shares' cost base and reduced cost base, the date the discount tests use
//! (ss 115-30 and 115-34) and the capital gain or loss disregarded. Scrip for
//! scrip (Subdivision 124-M) is an event of `cgt_cost_base_adjustments`, which
//! carries a parcel's cost base to its replacement. The provisions were read
//! on the ATO's legal database on 26 November 2026; their history notes show
//! no change since 1 July 2020 other than the wording of an example to
//! s 126-15 (Act No. 13 of 2021). Eligibility is the caller's statement.

use super::input::Fields;
use super::*;
use crate::engine::result;

const KIND: &str = "cgt_rollover";

/// Shares and other assets acquired before this day are pre-CGT.
const PRE_CGT_FROM: &str = "1985-09-20";

/// The facts as the command passes them; see the README for each.
pub struct CgtRolloverArgs<'a> {
    pub rollover: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// What a roll-over gives: its amounts and the fields shown beside them.
type Outcome = (Vec<(&'static str, Decimal)>, Map<String, Value>);

fn no_rollover<T>(why: impl Into<String>) -> Result<T> {
    refuse("cgt_rollover.not_available", why)
}

fn pre_cgt(day: i64) -> bool {
    day < dates::parse(PRE_CGT_FROM).expect("a date")
}

/// A JSON boolean fact.
fn yes(f: &mut Fields, key: &'static str) -> Result<bool> {
    match f.required(key)? {
        Value::Bool(b) => Ok(*b),
        _ => f.invalid(key, "a JSON boolean"),
    }
}

/// One of `allowed`, as text.
fn choice(f: &mut Fields, key: &'static str, allowed: &[&'static str]) -> Result<&'static str> {
    let given = f.text(key)?;
    match allowed.iter().find(|a| **a == given) {
        Some(a) => Ok(*a),
        None => f.invalid(key, &format!("one of {}", allowed.join(", "))),
    }
}

/// A date in the income year being worked.
fn in_year(f: &mut Fields, key: &'static str, year: &str) -> Result<i64> {
    let day = f.date(key)?;
    let first = year_first(year).expect("a supported income year");
    let start = dates::parse(&format!("{first}-07-01")).expect("a date");
    let end = dates::add_months(start, 12) - 1;
    if !(start..=end).contains(&day) {
        return conflict(format!(
            "{key} must be in the {year} income year, from {} to {}.",
            dates::format(start),
            dates::format(end)
        ));
    }
    Ok(day)
}

fn max_nil(v: Decimal) -> Result<Decimal> {
    Ok(if v.gt(&int(0))? { v } else { int(0) })
}

/// The gain and loss a roll-over disregards for stated capital proceeds.
fn disregarded(proceeds: &Decimal, bases: [&Decimal; 2]) -> Result<[Decimal; 2]> {
    Ok([
        max_nil(proceeds.sub(bases[0])?)?,
        max_nil(bases[1].sub(proceeds)?)?,
    ])
}

const INDEXED: &str = "The asset was acquired at or before 21 September 1999: a cost base the transferor indexed carries under the roll-over rules of Division 114, which this worksheet does not apply; the amounts are the cost bases as stated.";

mod company;
mod marriage;
mod small_business;

/// A chosen roll-over's consequences from stated facts.
pub fn cgt_rollover(args: &CgtRolloverArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let Arg::Json(Value::Object(fields)) = args.facts else {
        return refuse(
            "cgt_rollover.facts_invalid",
            "facts must be a JSON object of the roll-over's facts.",
        );
    };
    let rollover = args.rollover.text().unwrap_or_default();
    let mut f = Fields::new("facts".to_string(), fields);
    let (amounts, extra) = precise(|| match rollover {
        "marriage_breakdown" => marriage::work(&mut f, &year),
        "small_business" => small_business::work(&mut f, &year),
        "wholly_owned_company" => company::work(&mut f, &year),
        _ => refuse(
            "cgt_rollover.rollover_invalid",
            "rollover must be marriage_breakdown, small_business or wholly_owned_company; a scrip for scrip roll-over is a scrip_for_scrip event of cgt-cost-base-adjustments.",
        ),
    })?;
    f.none_left()?;
    let mut out = result(KIND, &year, amounts, Vec::new())?;
    out["rollover"] = json!(rollover);
    let warnings: Vec<Value> = match extra.get("warnings") {
        Some(Value::Array(w)) => w.clone(),
        _ => Vec::new(),
    };
    for (k, v) in extra {
        if k != "warnings" {
            out[k] = v;
        }
    }
    if let Some(w) = out["warnings"].as_array_mut() {
        w.extend(warnings);
    }
    Ok(out)
}

/// The stated list of 1 to `MOST` objects under `key`, or none when absent.
fn optional_objects<'a>(
    f: &mut Fields<'a>,
    key: &'static str,
) -> Result<Option<Vec<&'a Map<String, Value>>>> {
    let Some(value) = f.get(key) else {
        return Ok(None);
    };
    let list: Option<Vec<&'a Map<String, Value>>> = value
        .as_array()
        .filter(|l| !l.is_empty() && l.len() <= MOST)
        .and_then(|l| l.iter().map(Value::as_object).collect());
    match list {
        Some(list) => Ok(Some(list)),
        None => f.invalid(key, &format!("a list of 1 to {MOST} objects")),
    }
}
