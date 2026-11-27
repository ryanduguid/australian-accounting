// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `small_business_income_offset`: an individual's small business income tax
//! offset under ITAA 1997 Subdivision 328-F for 2020-21 to 2026-27. The
//! citations are to ITAA 1997 compilation 268 (1 October 2026), read on
//! 22 November 2026. Subdivision 328-F reads the same in compilation 210
//! (from 1 July 2020) except the rate in s 328-360(1), 13% there and 16% from
//! compilation 223 (1 July 2021), as the ATO's 'Small business income tax
//! offset' page (last updated 7 May 2025) gives for 2020-21 and for 2021-22
//! onwards. Net small business income, the basic income tax liability and
//! the offsets applied first are the operator's figures.

use serde_json::{Map, Value, json};

use super::stated;
use super::{Arg, Result, flag, int, money, precise, py_min, rate, refuse, result, scope};
use crate::decimal::Decimal;

const KIND: &str = "small_business_income_offset";

/// Section 328-357: each $10 million in s 328-110 read as $5 million.
const TURNOVER: &str = "5000000";
/// Section 328-360(2).
const CAP: &str = "1000";
/// The largest number of s 328-355(b) or (c) amounts the command takes.
const MAX_SHARES: usize = 50;

/// The facts the command takes only for an individual with net small
/// business income of their own, in [`SmallBusinessIncomeOffsetArgs::optional`].
pub const SBITO_OPTIONAL: [&str; 6] = [
    "business",
    "small_business_when_stopped",
    "previous_year_turnover",
    "year_before_turnover",
    "likely_turnover",
    "year_turnover",
];

/// The facts as the command passes them; see the README for each.
pub struct SmallBusinessIncomeOffsetArgs<'a> {
    pub taxable_income: &'a Arg,
    pub basic_income_tax_liability: &'a Arg,
    pub offsets_applied_first: &'a Arg,
    pub net_small_business_income: &'a Arg,
    pub shares: &'a Arg,
    pub prescribed_person: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 6],
}

/// Section 328-360(1): 13% for 2020-21 (compilation 210), 16% from 2021-22.
fn offset_rate(year: &str) -> &'static str {
    if year == "2020-21" { "0.13" } else { "0.16" }
}

/// A turnover, or `null` for a year no business was carried on.
fn turnover(value: &Arg) -> Result<Option<Decimal>> {
    match value {
        Arg::Other(none) if none == "None" => Ok(None),
        other => money(other).map(Some),
    }
}

/// Whether the individual is a small business entity for the year (s 328-110
/// with $5 million, s 328-357), and by which rule.
fn small_business_entity(given: [Option<&Arg>; 6]) -> Result<Option<&'static str>> {
    let [business, stopped, previous, before, likely, at_end] = given;
    let business = business.expect("required with net small business income");
    let carried = carried_on(business, stopped)?;
    let [Some(previous), Some(before), Some(likely), Some(at_end)] =
        [previous, before, likely, at_end]
    else {
        unreachable!("required with net small business income");
    };
    let previous = turnover(previous)?;
    let before = turnover(before)?;
    let likely = money(likely)?;
    let at_end = money(at_end)?;
    let limit = Decimal::parse(TURNOVER)?;
    let under = |amount: &Decimal| amount.lt(&limit);
    // s 328-110(3): the likely turnover does not count after 2 years at or
    // over the threshold.
    let barred = match (&previous, &before) {
        (Some(p), Some(b)) => !under(p)? && !under(b)?,
        _ => false,
    };
    Ok(if !carried {
        None
    } else if previous.as_ref().map(under).transpose()? == Some(true) {
        Some("s 328-110(1)(b)(i)")
    } else if under(&likely)? && !barred {
        Some("s 328-110(1)(b)(ii)")
    } else if under(&at_end)? {
        Some("s 328-110(4)")
    } else {
        None
    })
}

/// Whether the business counts as carried on: yes while it runs, and while
/// it is wound up only if it was a small business entity when it stopped
/// (s 328-110(5)).
fn carried_on(business: &Arg, stopped: Option<&Arg>) -> Result<bool> {
    match (business.text(), stopped) {
        (Some("carried_on"), None) => Ok(true),
        (Some("winding_up"), Some(stopped)) => flag(stopped, "small_business_when_stopped"),
        (Some("winding_up"), None) => refuse(
            "input.missing_field",
            "small_business_when_stopped is required for a business being wound up.",
        ),
        (Some("carried_on"), Some(_)) => refuse(
            "input.field_unknown",
            "small_business_when_stopped is an input only for a business being wound up",
        ),
        _ => refuse(
            "small_business_income_offset.business_invalid",
            "business must be carried_on or winding_up.",
        ),
    }
}

/// Each s 328-355(b) or (c) amount less its s 328-370 deductions, not below
/// zero (s 328-360(1)(b)).
fn shares(given: &Arg) -> Result<Vec<Decimal>> {
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            "small_business_income_offset.shares_invalid",
            "shares must be a list of objects with amount and deductions.",
        );
    };
    if items.len() > MAX_SHARES {
        return refuse(
            "small_business_income_offset.shares_invalid",
            format!("shares must hold at most {MAX_SHARES} entries."),
        );
    }
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let name = format!("share {}", i + 1);
            let Value::Object(fields) = item else {
                return refuse(
                    "small_business_income_offset.shares_invalid",
                    format!("{name} must be an object with amount and deductions."),
                );
            };
            stated::known_keys(fields, &name, "a share", &["amount", "deductions"])?;
            let amount = stated::amount(fields, &name, "amount")?;
            let deductions = stated::amount(fields, &name, "deductions")?;
            let net = amount.sub(&deductions)?;
            Ok(if net.lt(&int(0))? { int(0) } else { net })
        })
        .collect()
}

/// The optional facts: required with net small business income of the
/// individual's own and refused without it; `small_business_when_stopped`
/// is checked with `business`.
fn optional_facts<'a>(
    args: &SmallBusinessIncomeOffsetArgs<'a>,
    own: bool,
) -> Result<[Option<&'a Arg>; 6]> {
    for (key, given) in SBITO_OPTIONAL.iter().zip(args.optional) {
        match (own, given) {
            (true, None) if *key != "small_business_when_stopped" => {
                return refuse(
                    "input.missing_field",
                    format!("{key} is required with net small business income of your own."),
                );
            }
            (false, Some(_)) => {
                return refuse(
                    "input.field_unknown",
                    format!("{key} is an input only with net small business income of your own"),
                );
            }
            _ => {}
        }
    }
    Ok(args.optional)
}

const PRESCRIBED: &str = "A prescribed person under s 102AC of the ITAA 1936 (broadly, under 18 at the end of the year) works out total net small business income from business income under s 102AE (ITAA 1997 s 328-375), which this worksheet does not take.";

const ENTITY_RULE: &str = "ITAA 1997 ss 328-355(a), 328-357 and 328-110: you carry on a business in the income year (s 328-110(5) for one being wound up) and your aggregated turnover is under $5 million for the previous year, likely for the year (not after 2 years at $5 million or more, s 328-110(3)), or for the year at its end (s 328-110(4))";

const STEPS: [(&str, &str); 4] = [
    (
        "total_net_small_business_income",
        "ITAA 1997 s 328-360(1): your net small business income if you are a small business entity, plus each s 328-355(b) or (c) amount less its s 328-370 deductions (not below zero), no more than your taxable income",
    ),
    (
        "offset_before_cap",
        "ITAA 1997 s 328-360(1): the rate x total net small business income / taxable income x basic income tax liability (13% for 2020-21, 16% from 2021-22)",
    ),
    (
        "offset",
        "ITAA 1997 s 328-360(2): $1,000 if the amount under s 328-360(1) exceeds $1,000",
    ),
    (
        "offset_applied",
        "ITAA 1997 s 63-10(1) item 20: applied against the basic income tax liability left after offsets_applied_first; the rest cannot be refunded, transferred or carried forward",
    ),
];

fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

pub fn small_business_income_offset(args: &SmallBusinessIncomeOffsetArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    if flag(args.prescribed_person, "prescribed_person")? {
        return refuse("small_business_income_offset.prescribed_person", PRESCRIBED);
    }
    let taxable = money(args.taxable_income)?;
    super::whole_dollars(&taxable, "taxable income")?;
    let liability = money(args.basic_income_tax_liability)?;
    let first = money(args.offsets_applied_first)?;
    let own = money(args.net_small_business_income)?;
    let shares = shares(args.shares)?;
    let optional = optional_facts(args, !own.is_zero())?;

    // None without own income; Some(None) when not a small business entity.
    let entity = if own.is_zero() {
        None
    } else {
        Some(small_business_entity(optional)?)
    };
    let counted = match entity {
        Some(Some(_)) => own.clone(),
        _ => int(0),
    };
    let shares_net = shares
        .iter()
        .try_fold(int(0), |sum, share| sum.add(share))?;
    let total = py_min(counted.add(&shares_net)?, taxable.clone())?;
    let (before_cap, offset, applied) =
        offset_amounts(&year, &taxable, &liability, &first, &total)?;
    let unused = offset.sub(&applied)?;

    let out = result(
        KIND,
        &year,
        vec![
            ("net_small_business_income_counted", counted),
            ("shares_net", shares_net),
            ("total_net_small_business_income", total),
            ("offset_before_cap", before_cap),
            ("offset", offset),
            ("offset_applied", applied),
            ("offset_unused", unused),
        ],
        vec![
            rate("offset_rate", offset_rate(&year)),
            rate("cap", CAP),
            rate("turnover_threshold", TURNOVER),
        ],
    );
    let mut out = out?;
    explain(&mut out, entity);
    Ok(out)
}

/// The small business entity condition and rule, when the individual has
/// net small business income of their own, and the authorities.
fn explain(out: &mut Value, entity: Option<Option<&'static str>>) {
    let conditions: Vec<Value> = entity
        .iter()
        .map(|rule| condition("small_business_entity", ENTITY_RULE, rule.is_some()))
        .collect();
    out["conditions"] = json!(conditions);
    out["small_business_entity_rule"] = match entity {
        Some(Some(rule)) => json!(rule),
        _ => Value::Null,
    };
    out["authorities"] = Value::Object(
        STEPS
            .iter()
            .map(|(step, text)| ((*step).to_owned(), json!(text)))
            .collect::<Map<_, _>>(),
    );
}

/// The offset before the cap, the offset and the part applied (s 328-360 and
/// s 63-10(1) item 20).
fn offset_amounts(
    year: &str,
    taxable: &Decimal,
    liability: &Decimal,
    first: &Decimal,
    total: &Decimal,
) -> Result<(Decimal, Decimal, Decimal)> {
    let cap = Decimal::parse(CAP)?;
    precise(|| {
        let before_cap = if taxable.is_zero() {
            int(0)
        } else {
            Decimal::parse(offset_rate(year))?
                .mul(liability)?
                .mul(total)?
                .div(taxable)?
        };
        let offset = py_min(before_cap.clone(), cap.clone())?;
        let left = liability.sub(first)?;
        let left = if left.lt(&int(0))? { int(0) } else { left };
        let applied = py_min(offset.clone(), left)?;
        Ok((before_cap, offset, applied))
    })
}
