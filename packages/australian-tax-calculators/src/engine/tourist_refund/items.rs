// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A tourist refund item's stated facts, checked.

use serde_json::{Map, Value};

use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{Arg, Result, int, refuse, stated};

const FIELDS: [&str; 7] = [
    "date",
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

/// The GST on a taxable supply is 10% of its value, the price less the GST
/// (GST Act ss 9-70 and 9-75), so at most one eleventh of the price.
fn gst_within_price(price: &Decimal, gst: &Decimal, item: &str) -> Result<()> {
    let share = price.div(&int(11))?.quantize(-2, Rounding::HalfUp)?;
    if gst.gt(&share)? {
        return refuse(
            "tourist_refund.gst_exceeds_price_share",
            format!(
                "{item}: gst {gst} is more than one eleventh of the price {price} ({share}), the most GST a taxable supply at that price bears (GST Act ss 9-70 and 9-75)."
            ),
        );
    }
    Ok(())
}

/// `wine_tax_borne`: required with wine, refused with other goods.
fn wine_tax_borne(fields: &Map<String, Value>, item: &str, goods: &str) -> Result<bool> {
    if goods == "wine" {
        return boolean(fields, item, "wine_tax_borne");
    }
    if fields.contains_key("wine_tax_borne") {
        return invalid(format!(
            "{item}: wine_tax_borne is taken only with goods wine."
        ));
    }
    Ok(false)
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
