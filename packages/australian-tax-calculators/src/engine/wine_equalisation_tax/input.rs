// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! A wine dealing's stated facts, checked.

use serde_json::{Map, Value};

use crate::decimal::Decimal;
use crate::engine::{Arg, Result, money, refuse};

/// The dealings the command values, each with the fields it takes.
const KINDS: [(&str, &[&str]); 5] = [
    (
        "wholesale_sale",
        &["kind", "price_excluding_wet_and_gst", "rebate_conditions"],
    ),
    (
        "retail_sale",
        &[
            "kind",
            "retail_price_including_wet_and_gst",
            "average_wholesale",
            "rebate_conditions",
        ],
    ),
    (
        "own_use",
        &[
            "kind",
            "retail_price_including_wet_and_gst",
            "average_wholesale",
            "rebate_conditions",
        ],
    ),
    (
        "quoted_sale",
        &[
            "kind",
            "price_excluding_wet_and_gst",
            "purchaser_intention",
            "rebate_conditions",
        ],
    ),
    (
        "own_use_purchased_under_quote",
        &["kind", "purchase_price_excluding_gst"],
    ),
];

/// What a purchaser's quote may state it intends to do with the wine: a
/// taxable dealing of its own, or one of the ways in WET Act s 5-50(1)(b).
const INTENTIONS: [&str; 4] = [
    "taxable_dealing",
    "manufacture",
    "gst_free_supply",
    "sale_under_quote",
];

const AVERAGE_FIELDS: [&str; 4] = [
    "grape_wine",
    "wholesale_share_at_least_10_percent",
    "quantity",
    "wholesale_sales",
];
const SALE_FIELDS: [&str; 2] = ["quantity", "price_excluding_wet_and_gst"];
pub(super) const CONDITIONS: [&str; 3] = [
    "producer",
    "source_product_owned_85_percent",
    "container_and_trade_mark",
];
const MAX_SALES: usize = 1000;

/// How a dealing's taxable value is worked out.
pub(super) enum Valuation {
    /// A price that is the taxable value as given.
    Price(Decimal),
    /// The half retail price method on a price including WET and GST.
    HalfRetail(Decimal),
    /// The average wholesale price method: the dealing's quantity, and the
    /// period's wholesale sales' total quantity and total price.
    AverageWholesale {
        quantity: Decimal,
        sold: Decimal,
        price: Decimal,
    },
}

/// A dealing's checked facts.
pub(super) struct Dealing {
    pub(super) kind: &'static str,
    pub(super) valuation: Valuation,
    /// A retail sale's price including WET and GST, for its GST.
    pub(super) retail_price: Option<Decimal>,
    pub(super) intention: Option<&'static str>,
    pub(super) conditions: Option<[bool; 3]>,
}

fn invalid<T>(message: String) -> Result<T> {
    refuse("wine_equalisation_tax.dealing_invalid", message)
}

/// `fields`, refusing a key outside `allowed`.
fn only<'v>(
    value: &'v Value,
    n: usize,
    what: &str,
    allowed: &[&str],
) -> Result<&'v Map<String, Value>> {
    let Value::Object(fields) = value else {
        return invalid(format!("dealing {n}: {what} must be an object."));
    };
    if let Some(key) = fields.keys().find(|k| !allowed.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("dealing {n}: {key} is not an input of {what}"),
        );
    }
    Ok(fields)
}

fn required<'v>(fields: &'v Map<String, Value>, n: usize, key: &str) -> Result<&'v Value> {
    match fields.get(key) {
        Some(value) => Ok(value),
        None => refuse(
            "input.missing_field",
            format!("dealing {n}: {key} is required"),
        ),
    }
}

/// An amount in AUD, a decimal string as every money input is.
fn amount(fields: &Map<String, Value>, n: usize, key: &str) -> Result<Decimal> {
    let parsed = match required(fields, n, key)? {
        Value::String(text) if text.len() <= 32 => Decimal::parse(text).ok(),
        _ => None,
    };
    match parsed.map(|v| money(&Arg::Decimal(v))) {
        Some(Ok(v)) => Ok(v),
        _ => refuse(
            "input.amount_invalid",
            format!(
                "dealing {n}: {key} must be a decimal string, finite and non-negative, at most 2dp and 1e12."
            ),
        ),
    }
}

/// A quantity above nil, at most 1e12 and 4 decimal places.
fn quantity(fields: &Map<String, Value>, n: usize, key: &str) -> Result<Decimal> {
    let parsed = match required(fields, n, key)? {
        Value::String(text) if text.len() <= 32 => Decimal::parse(text).ok(),
        _ => None,
    };
    let valid = |v: &Decimal| -> Result<bool> {
        Ok(v.is_finite()
            && v.gt(&crate::engine::int(0))?
            && !v.gt(&crate::engine::d("1e12"))?
            && v.exponent() >= -4)
    };
    match parsed {
        Some(v) if valid(&v)? => Ok(v),
        _ => refuse(
            "wine_equalisation_tax.quantity_invalid",
            format!(
                "dealing {n}: {key} must be a decimal string above 0, at most 1e12 and 4 decimal places."
            ),
        ),
    }
}

fn boolean(fields: &Map<String, Value>, n: usize, key: &str) -> Result<bool> {
    match required(fields, n, key)? {
        Value::Bool(b) => Ok(*b),
        _ => refuse(
            "input.flag_invalid",
            format!("dealing {n}: {key} must be a boolean."),
        ),
    }
}

/// The average wholesale price method's facts: WET Act ss 9-25(3) and 9-40.
fn average_wholesale(value: &Value, n: usize) -> Result<Valuation> {
    let fields = only(value, n, "average_wholesale", &AVERAGE_FIELDS)?;
    if !boolean(fields, n, "grape_wine")? {
        return refuse(
            "wine_equalisation_tax.method_unavailable",
            format!(
                "dealing {n}: wine that is not grape wine is valued by the half retail price method only (WET Act s 9-30); give retail_price_including_wet_and_gst instead."
            ),
        );
    }
    if !boolean(fields, n, "wholesale_share_at_least_10_percent")? {
        return refuse(
            "wine_equalisation_tax.method_unavailable",
            format!(
                "dealing {n}: the average wholesale price method may be chosen only if at least 10% by value of the tax period's sales of grape wine of that vintage and variety are wholesale sales (WET Act s 9-25(3)); use the half retail price method."
            ),
        );
    }
    let quantity_ = quantity(fields, n, "quantity")?;
    let Some(Value::Array(sales)) = fields.get("wholesale_sales") else {
        return invalid(format!(
            "dealing {n}: wholesale_sales must be a list of the tax period's wholesale sales of the same vintage and variety."
        ));
    };
    if sales.is_empty() || sales.len() > MAX_SALES {
        return invalid(format!(
            "dealing {n}: wholesale_sales must hold from 1 to {MAX_SALES} sales."
        ));
    }
    let (mut sold, mut price) = (crate::engine::int(0), crate::engine::int(0));
    for sale in sales {
        let sale = only(sale, n, "a wholesale sale", &SALE_FIELDS)?;
        sold = sold.add(&quantity(sale, n, "quantity")?)?;
        price = price.add(&amount(sale, n, "price_excluding_wet_and_gst")?)?;
    }
    Ok(Valuation::AverageWholesale {
        quantity: quantity_,
        sold,
        price,
    })
}

/// A retail sale or own use: the half retail price method, or the average
/// wholesale price method where `average_wholesale` is given.
fn retail_valuation(
    fields: &Map<String, Value>,
    n: usize,
    kind: &str,
) -> Result<(Valuation, Option<Decimal>)> {
    let retail = "retail_price_including_wet_and_gst";
    let price = match fields.get(retail) {
        Some(_) => Some(amount(fields, n, retail)?),
        None => None,
    };
    match (fields.get("average_wholesale"), price) {
        (Some(_), Some(_)) if kind == "own_use" => refuse(
            "input.field_unknown",
            format!(
                "dealing {n}: give own use either {retail} (the half retail price method) or average_wholesale, not both"
            ),
        ),
        (Some(average), Some(p)) => Ok((average_wholesale(average, n)?, Some(p))),
        (Some(average), None) if kind == "own_use" => Ok((average_wholesale(average, n)?, None)),
        (None, Some(p)) => Ok((Valuation::HalfRetail(p.clone()), Some(p))),
        _ => refuse(
            "input.missing_field",
            format!("dealing {n}: {retail} is required"),
        ),
    }
}

fn conditions(fields: &Map<String, Value>, n: usize) -> Result<Option<[bool; 3]>> {
    let Some(value) = fields.get("rebate_conditions") else {
        return Ok(None);
    };
    let given = only(value, n, "rebate_conditions", &CONDITIONS)?;
    let mut met = [false; 3];
    for (key, slot) in CONDITIONS.iter().zip(&mut met) {
        *slot = boolean(given, n, key)?;
    }
    Ok(Some(met))
}

fn intention(fields: &Map<String, Value>, n: usize) -> Result<&'static str> {
    let given = required(fields, n, "purchaser_intention")?;
    match INTENTIONS.iter().find(|i| given.as_str() == Some(**i)) {
        Some(found) => Ok(found),
        None => refuse(
            "wine_equalisation_tax.intention_invalid",
            format!(
                "dealing {n}: purchaser_intention must be taxable_dealing, manufacture, gst_free_supply or sale_under_quote."
            ),
        ),
    }
}

/// Dealing `n`'s facts.
pub(super) fn parse_dealing(value: &Value, n: usize) -> Result<Dealing> {
    let Value::Object(raw) = value else {
        return invalid(format!("dealing {n}: a dealing must be an object."));
    };
    let given = raw.get("kind").and_then(Value::as_str);
    let Some((kind, allowed)) = KINDS.iter().find(|(k, _)| Some(*k) == given) else {
        return refuse(
            "wine_equalisation_tax.kind_invalid",
            format!(
                "dealing {n}: kind must be wholesale_sale, retail_sale, own_use, quoted_sale or own_use_purchased_under_quote."
            ),
        );
    };
    let fields = only(value, n, kind, allowed)?;
    let (valuation, retail_price) = match *kind {
        "wholesale_sale" | "quoted_sale" => (
            Valuation::Price(amount(fields, n, "price_excluding_wet_and_gst")?),
            None,
        ),
        "own_use_purchased_under_quote" => (
            Valuation::Price(amount(fields, n, "purchase_price_excluding_gst")?),
            None,
        ),
        _ => retail_valuation(fields, n, kind)?,
    };
    let intention = match *kind {
        "quoted_sale" => Some(intention(fields, n)?),
        _ => None,
    };
    Ok(Dealing {
        kind,
        valuation,
        retail_price: retail_price.filter(|_| *kind == "retail_sale"),
        intention,
        conditions: conditions(fields, n)?,
    })
}
