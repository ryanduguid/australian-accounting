// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A line's stated facts, checked and classified to its Excise Tariff Act
//! 1921 Schedule subitem.

use serde_json::{Map, Value};

use crate::decimal::Decimal;
use crate::engine::{Arg, Result, d, date_arg, int, refuse};

/// The products a line may name and be worked.
const PRODUCTS: [&str; 4] = ["beer", "brandy", "spirit", "other_excisable_beverage"];

/// Products the command names only to refuse, with the code and the reason.
const REFUSED: [(&str, &str, &str); 5] = [
    (
        "wine",
        "alcohol_excise.not_excisable",
        "wine is not an excisable beverage (Excise Tariff Act 1921 Schedule, 'other excisable beverage' (c)); it bears wine equalisation tax: use wine-equalisation-tax.",
    ),
    (
        "tobacco",
        "alcohol_excise.product_out_of_scope",
        "tobacco (Schedule item 5) is outside this worksheet, which covers items 1 to 3 only.",
    ),
    (
        "fuel",
        "alcohol_excise.product_out_of_scope",
        "fuel (Schedule item 10) is outside this worksheet, which covers items 1 to 3 only.",
    ),
    (
        "concessional_spirit",
        "alcohol_excise.product_out_of_scope",
        "spirit under subitems 3.5 to 3.8 is free only under an approval or determination (Excise Act 1901 ss 77FD to 77FG), a licensing question this worksheet does not decide.",
    ),
    (
        "non_commercial",
        "alcohol_excise.not_excisable",
        "liquor produced for non-commercial purposes using non-commercial facilities and equipment, other than liquor that is or contains distilled spirit, is outside Schedule items 1 to 3 (the note before the Schedule's table).",
    ),
];

const COMMON: [&str; 7] = [
    "date",
    "product",
    "volume_litres",
    "strength_percent",
    "labelled_strength_percent",
    "remission",
    "imported",
];
const BEER: [&str; 3] = ["container_litres", "pressurised_system", "brew_on_premises"];

/// A line's checked facts and its subitem.
pub(super) struct Line {
    pub(super) date: i64,
    pub(super) product: &'static str,
    pub(super) volume: Decimal,
    /// The strength duty is charged on: the labelled one where higher
    /// (Excise Act 1901 s 77FA).
    pub(super) strength: Decimal,
    pub(super) labelled: bool,
    pub(super) subitem: &'static str,
    pub(super) remission: bool,
}

/// A decimal string field of line `n` above 0, at most `max` and `places`
/// decimal places; `None` when absent.
fn number(
    fields: &Map<String, Value>,
    n: usize,
    key: &str,
    max: &str,
    places: i128,
) -> Result<Option<Decimal>> {
    let parsed = match fields.get(key) {
        None => return Ok(None),
        Some(Value::String(text)) if text.len() <= 32 => Decimal::parse_input(text).ok(),
        Some(_) => None,
    };
    let valid = |v: &Decimal| -> Result<bool> {
        Ok(v.is_finite() && v.gt(&int(0))? && !v.gt(&d(max))? && v.exponent() >= -places)
    };
    match parsed {
        Some(v) if valid(&v)? => Ok(Some(v)),
        _ => refuse(
            "alcohol_excise.quantity_invalid",
            format!(
                "line {n}: {key} must be a decimal string above 0, at most {max} and {places} decimal places."
            ),
        ),
    }
}

fn required(value: Option<Decimal>, n: usize, key: &str) -> Result<Decimal> {
    match value {
        Some(v) => Ok(v),
        None => refuse(
            "input.missing_field",
            format!("line {n}: {key} is required"),
        ),
    }
}

/// A boolean field of line `n`, `false` when absent.
fn boolean(fields: &Map<String, Value>, n: usize, key: &str) -> Result<bool> {
    match fields.get(key) {
        None => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => refuse(
            "input.flag_invalid",
            format!("line {n}: {key} must be a boolean."),
        ),
    }
}

/// The product named, refusing one the worksheet does not work.
fn product(fields: &Map<String, Value>, n: usize) -> Result<&'static str> {
    let given = fields.get("product").and_then(Value::as_str);
    if let Some(found) = PRODUCTS.iter().find(|p| Some(**p) == given) {
        return Ok(found);
    }
    if let Some((_, code, why)) = REFUSED.iter().find(|(p, ..)| Some(*p) == given) {
        return refuse(code, format!("line {n}: {why}"));
    }
    if fields.get("product").is_none() {
        return refuse(
            "input.missing_field",
            format!("line {n}: product is required"),
        );
    }
    refuse(
        "alcohol_excise.product_invalid",
        format!("line {n}: product must be beer, brandy, spirit or other_excisable_beverage."),
    )
}

/// Refuse any key the product does not take.
fn known_fields(fields: &Map<String, Value>, n: usize, product: &str) -> Result<()> {
    let beer = product == "beer";
    match fields
        .keys()
        .find(|k| !COMMON.contains(&k.as_str()) && !(beer && BEER.contains(&k.as_str())))
    {
        Some(key) => refuse(
            "input.field_unknown",
            format!("line {n}: {key} is not an input for {product}"),
        ),
        None => Ok(()),
    }
}

pub(super) fn parse_line(value: &Value, n: usize) -> Result<Line> {
    let Value::Object(fields) = value else {
        return refuse(
            "alcohol_excise.line_invalid",
            format!("line {n} must be an object."),
        );
    };
    let product = product(fields, n)?;
    known_fields(fields, n, product)?;
    if boolean(fields, n, "imported")? {
        return refuse(
            "alcohol_excise.product_out_of_scope",
            format!(
                "line {n}: imported goods bear customs duty under the Customs Tariff Act 1995, administered with the Department of Home Affairs, not excise duty under the Excise Tariff Act 1921."
            ),
        );
    }
    let date = date_arg(
        &Arg::Text(
            fields
                .get("date")
                .and_then(Value::as_str)
                .unwrap_or("")
                .into(),
        ),
        &format!("line {n}: date"),
    )?;
    let volume = required(
        number(fields, n, "volume_litres", "1e12", 4)?,
        n,
        "volume_litres",
    )?;
    let actual = required(
        number(fields, n, "strength_percent", "100", 3)?,
        n,
        "strength_percent",
    )?;
    let subitem = classify(fields, n, product, &actual)?;
    let (strength, labelled) = charged_strength(fields, n, product, actual, subitem)?;
    Ok(Line {
        date,
        product,
        volume,
        strength,
        labelled,
        subitem,
        remission: boolean(fields, n, "remission")?,
    })
}

/// The strength duty is charged on and whether it is the label's: a labelled
/// strength above the actual one (Excise Act 1901 s 77FA(1)). Refused where
/// the label would move the goods to another subitem, which s 77FA leaves
/// unsettled.
fn charged_strength(
    fields: &Map<String, Value>,
    n: usize,
    product: &str,
    actual: Decimal,
    subitem: &str,
) -> Result<(Decimal, bool)> {
    let Some(label) = number(fields, n, "labelled_strength_percent", "100", 3)? else {
        return Ok((actual, false));
    };
    if !label.gt(&actual)? {
        return Ok((actual, false));
    }
    let other = classify(fields, n, product, &label)?;
    if other != subitem {
        return refuse(
            "alcohol_excise.classification_unsettled",
            format!(
                "line {n}: the labelled strength puts the goods in subitem {other} and the actual strength in {subitem}; Excise Act 1901 s 77FA charges duty on the labelled strength but does not say which subitem's rate then applies."
            ),
        );
    }
    Ok((label, true))
}

/// The subitem for `product` at `strength`: Schedule items 1, 2 and 3.
fn classify(
    fields: &Map<String, Value>,
    n: usize,
    product: &str,
    strength: &Decimal,
) -> Result<&'static str> {
    let over = |limit: &str| strength.gt(&d(limit));
    if matches!(product, "beer" | "other_excisable_beverage") && !over("1.15")? {
        return refuse(
            "alcohol_excise.not_excisable",
            format!(
                "line {n}: {product} must contain more than 1.15% alcohol by volume to be excisable (Excise Tariff Act 1921 Schedule, 'beer' (g) and 'other excisable beverage')."
            ),
        );
    }
    Ok(match product {
        "brandy" => "3.1",
        "spirit" => "3.10",
        "other_excisable_beverage" if over("10")? => "3.2",
        "other_excisable_beverage" => "2",
        _ => beer_subitem(fields, n, over("3")?, over("3.5")?)?,
    })
}

/// Beer's subitem by strength band and container (Schedule item 1).
fn beer_subitem(
    fields: &Map<String, Value>,
    n: usize,
    over_3: bool,
    over_3_5: bool,
) -> Result<&'static str> {
    if boolean(fields, n, "brew_on_premises")? {
        if let Some(key) = ["container_litres", "pressurised_system"]
            .into_iter()
            .find(|k| fields.contains_key(*k))
        {
            return refuse(
                "input.field_unknown",
                format!(
                    "line {n}: {key} is not an input for beer brewed on premises (subitems 1.15 and 1.16 do not turn on the container)"
                ),
            );
        }
        return Ok(if over_3 { "1.16" } else { "1.15" });
    }
    let draught = draught(fields, n)?;
    Ok(match (over_3, over_3_5, draught) {
        (false, _, false) => "1.1",
        (false, _, true) => "1.2",
        (true, false, false) => "1.5",
        (true, false, true) => "1.6",
        (true, true, false) => "1.10",
        (true, true, true) => "1.11",
    })
}

/// Whether the container takes the draught rate: over 48 litres, or 8 to 48
/// litres designed to connect to a pressurised gas or pump delivery system.
fn draught(fields: &Map<String, Value>, n: usize) -> Result<bool> {
    let container = required(
        number(fields, n, "container_litres", "1e6", 3)?,
        n,
        "container_litres",
    )?;
    let middle = !container.lt(&int(8))? && !container.gt(&int(48))?;
    match (middle, fields.get("pressurised_system")) {
        (true, None) => refuse(
            "input.missing_field",
            format!("line {n}: pressurised_system is required for a container of 8 to 48 litres"),
        ),
        (true, Some(_)) => boolean(fields, n, "pressurised_system"),
        (false, None) => Ok(container.gt(&int(48))?),
        (false, Some(_)) => refuse(
            "input.field_unknown",
            format!(
                "line {n}: pressurised_system is an input only for a container of 8 to 48 litres"
            ),
        ),
    }
}
