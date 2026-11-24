// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A tourist refund item's stated facts, checked.

use serde_json::{Map, Value};

use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{Arg, Result, int, refuse, stated};

const FIELDS: [&str; 9] = [
    "date",
    "taxable_supply",
    "wine_tax_refunded_or_credited",
    "price",
    "gst",
    "goods",
    "wine_tax_borne",
    "tax_invoice",
    "partly_consumed",
];
const GOODS: [&str; 4] = ["general", "wine", "other_alcohol", "tobacco"];
const MAX_ITEMS: usize = 1000;

/// One item's checked facts.
pub(super) struct Item {
    pub(super) date: i64,
    pub(super) price: Decimal,
    pub(super) gst: Decimal,
    pub(super) taxable_supply: bool,
    pub(super) goods: &'static str,
    pub(super) wine_tax_borne: bool,
    pub(super) tax_invoice: bool,
    pub(super) partly_consumed: bool,
}

fn invalid<T>(message: String) -> Result<T> {
    refuse("tourist_refund.item_invalid", message)
}

fn boolean(fields: &Map<String, Value>, item: &str, key: &str) -> Result<bool> {
    match stated::required(fields, item, key)? {
        Value::Bool(b) => Ok(*b),
        _ => refuse(
            "input.flag_invalid",
            format!("{item}: {key} must be a boolean."),
        ),
    }
}

fn goods(fields: &Map<String, Value>, item: &str) -> Result<&'static str> {
    let given = stated::required(fields, item, "goods")?.as_str();
    match GOODS.iter().find(|g| given == Some(**g)) {
        Some(g) => Ok(g),
        None => refuse(
            "tourist_refund.fact_invalid",
            format!("{item}: goods must be one of {}.", GOODS.join(", ")),
        ),
    }
}

fn date(fields: &Map<String, Value>, item: &str) -> Result<i64> {
    match stated::required(fields, item, "date")?
        .as_str()
        .and_then(dates::parse)
    {
        Some(day) => Ok(day),
        None => refuse(
            "input.date_invalid",
            format!("{item}: date must be a YYYY-MM-DD date from 1900 to 2100."),
        ),
    }
}

/// On the supported basis, the GST payable on an ordinary taxable supply is
/// 10% of its value, the price less the GST (GST Act ss 9-70 and 9-75),
/// rounded to the cent (s 9-90), so at most one eleventh of the price.
fn gst_within_price(price: &Decimal, gst: &Decimal, item: &str) -> Result<()> {
    let share = price.div(&int(11))?.quantize(-2, Rounding::HalfUp)?;
    if gst.gt(&share)? {
        return refuse(
            "tourist_refund.gst_exceeds_price_share",
            format!(
                "{item}: gst {gst} is more than one eleventh of the price {price} ({share}), outside the supported basis: an ordinary taxable supply at that price bears at most that (GST Act ss 9-70, 9-75 and 9-90); special valuation rules (s 9-99) and invoice-level allocations are not worked."
            ),
        );
    }
    Ok(())
}

/// `wine_tax_borne`: required with wine, with no wine tax refunded or
/// credited (WET Act s 31-10(2) and (3) would reduce it); both refused with
/// other goods.
fn wine_tax_borne(fields: &Map<String, Value>, item: &str, goods: &str) -> Result<bool> {
    const WINE: [&str; 2] = ["wine_tax_borne", "wine_tax_refunded_or_credited"];
    if goods != "wine" {
        if let Some(key) = WINE.iter().find(|k| fields.contains_key(**k)) {
            return invalid(format!("{item}: {key} is taken only with goods wine."));
        }
        return Ok(false);
    }
    if boolean(fields, item, "wine_tax_refunded_or_credited")? {
        return refuse(
            "tourist_refund.wine_tax_reduced",
            format!(
                "{item}: wine tax refunded or credited, or the basis of a wine tax credit entitlement, reduces the wine tax borne (WET Act s 31-10(2) and (3)); how that meets reg 25-5.03's amount is not settled by the sources read."
            ),
        );
    }
    boolean(fields, item, "wine_tax_borne")
}

fn item(value: &Value, n: usize, departure: i64) -> Result<Item> {
    let name = format!("item {n}");
    let Value::Object(fields) = value else {
        return invalid(format!("{name} must be an object."));
    };
    stated::known_keys(fields, &name, "an item", &FIELDS)?;
    let goods = goods(fields, &name)?;
    let item = Item {
        date: date(fields, &name)?,
        price: stated::amount(fields, &name, "price")?,
        gst: stated::amount(fields, &name, "gst")?,
        taxable_supply: boolean(fields, &name, "taxable_supply")?,
        goods,
        wine_tax_borne: wine_tax_borne(fields, &name, goods)?,
        tax_invoice: boolean(fields, &name, "tax_invoice")?,
        partly_consumed: boolean(fields, &name, "partly_consumed")?,
    };
    if item.date > departure {
        return refuse(
            "tourist_refund.date_invalid",
            format!("{name}: date is after departure_date."),
        );
    }
    if !item.taxable_supply && !item.gst.is_zero() {
        return invalid(format!(
            "{name}: gst is above nil on a supply stated not to be taxable."
        ));
    }
    gst_within_price(&item.price, &item.gst, &name)?;
    Ok(item)
}

/// The items, each checked, none acquired after `departure`.
pub(super) fn items(value: &Arg, departure: i64) -> Result<Vec<Item>> {
    let list = match value {
        Arg::Json(Value::Array(list)) if (1..=MAX_ITEMS).contains(&list.len()) => list,
        _ => {
            return invalid(format!("items must be a list of 1 to {MAX_ITEMS} objects."));
        }
    };
    list.iter()
        .enumerate()
        .map(|(i, v)| item(v, i + 1, departure))
        .collect()
}
