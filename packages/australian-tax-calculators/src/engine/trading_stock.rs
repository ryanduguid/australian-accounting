// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `trading_stock`: the change in value of trading stock under ITAA 1997
//! s 70-35 for 2020-21 to 2026-27, each closing item valued by its own
//! election (s 70-45 or s 70-50), and the small and medium business choice
//! of s 328-285 with the values of s 328-295. The citations are to ITAA 1997
//! compilation 268 (1 October 2026), read on 22 November 2026. Sections 70-35
//! to 70-50 read the same in compilations 210 (1 July 2020) and 223
//! (1 July 2021); s 328-285(2), the $50 million entity, applies to income
//! years starting on or after 1 July 2021 (Act No. 92 of 2020 Schedule 3
//! item 40(8)).

use serde_json::{Value, json};

use super::stated;
use super::{Arg, Result, flag, int, money, refuse, result, scope};
use crate::decimal::Decimal;

const KIND: &str = "trading_stock";

/// Section 328-285(1)(b).
const LIMIT: &str = "5000";
/// The largest number of items the command takes.
const MAX_ITEMS: usize = 500;
/// The 3 values of s 70-45(1), in its order.
const VALUES: [&str; 3] = ["cost", "market_selling_value", "replacement_value"];

/// The keys an item takes.
const ITEM_KEYS: [&str; 5] = [
    VALUES[0],
    VALUES[1],
    VALUES[2],
    "method",
    "obsolescence_value",
];

/// The facts as the command passes them; see the README for each.
pub struct TradingStockArgs<'a> {
    pub entity: &'a Arg,
    pub opening_value: &'a Arg,
    pub items: &'a Arg,
    pub estimated_closing_value: &'a Arg,
    pub simplified_choice: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One closing item: the method its value comes from and the value.
struct Valued {
    method: &'static str,
    value: Decimal,
}

/// Whether the entity may make the s 328-285 choice for the year: a small
/// business entity always, an entity covered by s 328-285(2) from 2021-22.
fn eligible(entity: &Arg, year: &str) -> Result<bool> {
    match entity.text() {
        Some("small_business_entity") => Ok(true),
        Some("medium_business_entity") => Ok(year != "2020-21"),
        Some("other") => Ok(false),
        _ => refuse(
            "trading_stock.entity_invalid",
            "entity must be small_business_entity, medium_business_entity or other.",
        ),
    }
}

/// An amount, or `null` for a value not given.
fn maybe(
    fields: &serde_json::Map<String, Value>,
    item: &str,
    key: &str,
) -> Result<Option<Decimal>> {
    match stated::required(fields, item, key)? {
        Value::Null => Ok(None),
        _ => stated::amount(fields, item, key).map(Some),
    }
}

fn missing<T>(item: &str, what: &str) -> Result<T> {
    refuse("trading_stock.value_missing", format!("{item}: {what}"))
}

/// The lowest of the 3 s 70-45 values, all of which must be given; the
/// first in s 70-45(1)'s order on a tie.
fn lowest(item: &str, values: &[Option<Decimal>; 3], why: &str) -> Result<(usize, Decimal)> {
    let mut best: Option<(usize, Decimal)> = None;
    for (i, value) in values.iter().enumerate() {
        let Some(value) = value else {
            return missing(
                item,
                &format!("{why} needs cost, market_selling_value and replacement_value."),
            );
        };
        let lower = match &best {
            None => true,
            Some((_, least)) => value.lt(least)?,
        };
        if lower {
            best = Some((i, value.clone()));
        }
    }
    Ok(best.expect("3 values"))
}

/// One item's elected value (s 70-45(1) or s 70-50).
fn value_item(i: usize, item: &Value) -> Result<Valued> {
    let name = format!("item {}", i + 1);
    let Value::Object(fields) = item else {
        return refuse(
            "trading_stock.items_invalid",
            format!("{name} must be an object."),
        );
    };
    stated::known_keys(fields, &name, "an item", &ITEM_KEYS)?;
    let values = [
        maybe(fields, &name, VALUES[0])?,
        maybe(fields, &name, VALUES[1])?,
        maybe(fields, &name, VALUES[2])?,
    ];
    let method = stated::required(fields, &name, "method")?
        .as_str()
        .unwrap_or("");
    if method != "obsolescence" && fields.contains_key("obsolescence_value") {
        return refuse(
            "input.field_unknown",
            format!("{name}: obsolescence_value is an input only for the obsolescence method"),
        );
    }
    match method {
        "lowest" => lowest(&name, &values, "lowest").map(|(at, value)| Valued {
            method: VALUES[at],
            value,
        }),
        "obsolescence" => obsolete(&name, fields, &values),
        chosen => elected(&name, chosen, &values),
    }
}

/// The value of the s 70-45(1) method named, which must be given.
fn elected(name: &str, chosen: &str, values: &[Option<Decimal>; 3]) -> Result<Valued> {
    let Some(at) = VALUES.iter().position(|m| *m == chosen) else {
        return refuse(
            "trading_stock.method_invalid",
            format!(
                "{name}: method must be cost, market_selling_value, replacement_value, lowest or obsolescence."
            ),
        );
    };
    match &values[at] {
        Some(value) => Ok(Valued {
            method: VALUES[at],
            value: value.clone(),
        }),
        None => missing(
            name,
            &format!("{chosen} is the elected method and is null."),
        ),
    }
}

/// Section 70-50: a value the operator elects below all 3 s 70-45 values.
fn obsolete(
    name: &str,
    fields: &serde_json::Map<String, Value>,
    values: &[Option<Decimal>; 3],
) -> Result<Valued> {
    let (_, least) = lowest(name, values, "obsolescence")?;
    let value = stated::amount(fields, name, "obsolescence_value")?;
    if !value.lt(&least)? {
        return refuse(
            "trading_stock.obsolescence_not_below",
            format!(
                "{name}: an obsolescence value must be below its cost, market selling value and replacement value (ITAA 1997 s 70-50)."
            ),
        );
    }
    Ok(Valued {
        method: "obsolescence",
        value,
    })
}

fn items(given: &Arg) -> Result<Vec<Valued>> {
    match given {
        Arg::Json(Value::Array(list)) if list.len() <= MAX_ITEMS => list
            .iter()
            .enumerate()
            .map(|(i, item)| value_item(i, item))
            .collect(),
        _ => refuse(
            "trading_stock.items_invalid",
            format!("items must be a list of at most {MAX_ITEMS} objects."),
        ),
    }
}

const STEPS: [(&str, &str); 4] = [
    (
        "stocktake_value",
        "ITAA 1997 s 70-45(1): each item at the cost, market selling value or replacement value elected for it, or s 70-50 below all 3 where obsolescence or other special circumstances warrant it",
    ),
    (
        "estimate_difference",
        "ITAA 1997 s 328-285(1)(b): the difference between the opening value and the reasonably estimated closing value, at most $5,000 for the choice; s 328-285(2) and Act No. 92 of 2020 Sch 3 item 40(8) for an entity under $50 million from 2021-22",
    ),
    (
        "closing_value",
        "ITAA 1997 s 70-45, or s 328-295(2) under the choice: the closing value equals the opening value; it is next year's opening value (ss 70-40 and 328-295(1))",
    ),
    (
        "assessable_increase",
        "ITAA 1997 s 70-35(2) and (3): an excess of the closing over the opening value is assessable; an excess of the opening over the closing value is deductible",
    ),
];

pub fn trading_stock(args: &TradingStockArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let eligible = eligible(args.entity, &year)?;
    let opening = money(args.opening_value)?;
    let valued = items(args.items)?;
    let stocktake = valued.iter().try_fold(int(0), |sum, v| sum.add(&v.value))?;
    let estimate = match args.estimated_closing_value {
        Arg::Other(none) if none == "None" => stocktake.clone(),
        given => money(given)?,
    };
    let chosen = flag(args.simplified_choice, "simplified_choice")?;
    let difference = gap(&estimate, &opening)?;
    let available = eligible && difference.le(&Decimal::parse(LIMIT)?)?;
    if chosen && !available {
        return refuse(
            "trading_stock.simplified_unavailable",
            unavailable(eligible, &year),
        );
    }
    let closing = if chosen {
        opening.clone()
    } else {
        stocktake.clone()
    };
    let (increase, decrease) = if closing.lt(&opening)? {
        (int(0), gap(&closing, &opening)?)
    } else {
        (gap(&closing, &opening)?, int(0))
    };
    let mut out = result(
        KIND,
        &year,
        vec![
            ("opening_value", opening),
            ("stocktake_value", stocktake),
            ("estimated_closing_value", estimate),
            ("estimate_difference", difference),
            ("closing_value", closing),
            ("assessable_increase", increase),
            ("deduction", decrease),
        ],
        vec![super::rate("simplified_limit", LIMIT)],
    )?;
    explain(&mut out, &valued, available, chosen)?;
    Ok(out)
}

/// The difference between 2 values, not below nil.
fn gap(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if a.lt(b)? { b.sub(a)? } else { a.sub(b)? })
}

fn unavailable(eligible: bool, year: &str) -> String {
    if eligible {
        "The choice not to account for changes in the value of trading stock needs the reasonably estimated closing value within $5,000 of the opening value (ITAA 1997 s 328-285(1)(b)).".into()
    } else {
        format!(
            "The choice not to account for changes in the value of trading stock is open in {year} only to a small business entity{} (ITAA 1997 s 328-285(1)(a)).",
            if year == "2020-21" {
                "; the $50 million entity of s 328-285(2) applies from 2021-22 (Act No. 92 of 2020 Sch 3 item 40(8))"
            } else {
                " or an entity covered by s 328-285(2)"
            }
        )
    }
}

/// Each item's elected method and value, whether the choice was open and
/// made, and the authorities.
fn explain(out: &mut Value, valued: &[Valued], available: bool, chosen: bool) -> Result<()> {
    let rows = valued
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let value = super::cents(v.value.clone())?.to_string();
            Ok(json!({"item": i + 1, "method": v.method, "value": value}))
        })
        .collect::<Result<Vec<Value>>>()?;
    out["items"] = json!(rows);
    out["simplified_available"] = json!(available);
    out["simplified_chosen"] = json!(chosen);
    out["authorities"] = STEPS
        .iter()
        .map(|(step, text)| ((*step).to_owned(), json!(text)))
        .collect::<serde_json::Map<_, _>>()
        .into();
    Ok(())
}
