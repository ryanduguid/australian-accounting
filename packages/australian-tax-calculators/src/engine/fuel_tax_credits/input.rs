// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A fuel tax credit line's stated facts, checked.

use serde_json::{Map, Value};

use super::conflict;
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, date_arg, money, refuse};

/// Each fuel a line may name, its ATO row and the unit its quantity is in.
pub(super) const FUELS: [(&str, usize, &str); 9] = [
    ("diesel", 0, "litres"),
    ("petrol", 0, "litres"),
    ("b5", 1, "litres"),
    ("b20", 1, "litres"),
    ("e10", 1, "litres"),
    ("e85", 2, "litres"),
    ("lpg", 3, "litres"),
    ("lng_cng", 4, "kilograms"),
    ("b100", 5, "litres"),
];

pub(super) const USES: [&str; 3] = [
    "heavy_vehicle_public_road",
    "other_business",
    "light_vehicle_public_road",
];

/// The diesel consumption rates for rigid trucks, in litres per kilometre, by
/// the upper bound of the gross weight class in tonnes (the last class has
/// none): the ATO's 'Basic method for heavy vehicles' (QC64655, last updated
/// 2 June 2021), Table 1, from PCG 2021/2 Schedule 1.
const RIGID_TRUCK_RATES: [(Option<&str>, &str); 5] = [
    (Some("12"), "0.18"),
    (Some("16.5"), "0.23"),
    (Some("24"), "0.28"),
    (Some("30"), "0.32"),
    (None, "0.35"),
];

pub(super) const DERIVED_KM_SHARE: &str = "0.984";

/// A non-negative decimal string field of line `n`, at most 1e12 and
/// `places` decimal places.
fn quantity(fields: &Map<String, Value>, n: usize, key: &str, places: i128) -> Result<Decimal> {
    let parsed = match fields.get(key) {
        None => {
            return refuse(
                "input.missing_field",
                format!("line {n}: {key} is required"),
            );
        }
        Some(Value::String(text)) if text.len() <= 32 => Decimal::parse_input(text).ok(),
        Some(_) => None,
    };
    let valid = |v: &Decimal| -> Result<bool> {
        Ok(v.is_finite()
            && !v.lt(&crate::engine::int(0))?
            && !v.gt(&crate::engine::d("1e12"))?
            && v.exponent() >= -places)
    };
    match parsed {
        Some(v) if valid(&v)? => Ok(v),
        _ => refuse(
            "fuel_tax_credits.quantity_invalid",
            format!(
                "line {n}: {key} must be a decimal string, non-negative, at most 1e12 and {places} decimal places."
            ),
        ),
    }
}

/// The value of line `n`'s `key`, if it is one of `allowed`; `None` if it is
/// another.
fn choice<'t>(
    fields: &Map<String, Value>,
    n: usize,
    key: &str,
    allowed: &[&'t str],
) -> Result<Option<&'t str>> {
    let Some(value) = fields.get(key) else {
        return refuse(
            "input.missing_field",
            format!("line {n}: {key} is required"),
        );
    };
    Ok(allowed
        .iter()
        .find(|a| value.as_str() == Some(**a))
        .copied())
}

/// The message for a value of `key` outside `allowed`.
fn expected(n: usize, key: &str, allowed: &[&str]) -> String {
    let (last, rest) = allowed.split_last().expect("choices");
    if rest.is_empty() {
        return format!("line {n}: {key} must be {last}.");
    }
    format!("line {n}: {key} must be {} or {last}.", rest.join(", "))
}

/// How a basic method line's public road kilometres and consumption rate were
/// given.
pub(super) struct Basic {
    pub(super) litres: Decimal,
    pub(super) km: Decimal,
    pub(super) km_basis: &'static str,
    pub(super) litres_per_km: Decimal,
    pub(super) rate_basis: &'static str,
}

/// A line's checked facts.
pub(super) struct Line {
    pub(super) fuel: &'static str,
    pub(super) category: usize,
    pub(super) unit: &'static str,
    pub(super) acquired: i64,
    pub(super) work: Work,
}

/// A line by quantity and use (the quantity given, or worked out from the
/// cost and the average price under PCG 2016/2 Method 1), or by the basic
/// method.
pub(super) enum Work {
    Quantity {
        quantity: Decimal,
        use_: &'static str,
        from_cost: Option<(Decimal, Decimal)>,
    },
    Basic(Basic),
}

const QUANTITY_FIELDS: [&str; 4] = ["fuel", "use", "acquired", "quantity"];
const FROM_COST_FIELDS: [&str; 6] = [
    "method",
    "fuel",
    "use",
    "acquired",
    "cost",
    "average_price_cents_per_litre",
];
const METHODS: [&str; 2] = ["basic_heavy_vehicle", "litres_from_cost"];
/// The liquid taxable fuels PCG 2016/2 covers (para 4(b)).
const LIQUID_FUELS: [&str; 7] = ["diesel", "petrol", "b5", "b20", "e10", "e85", "b100"];
const BASIC_FIELDS: [&str; 8] = [
    "method",
    "fuel",
    "acquired",
    "litres",
    "public_road_km",
    "total_km",
    "rigid_truck_gross_weight_tonnes",
    "litres_per_km",
];

/// Exactly one of 2 alternative fields of line `n`.
fn one_of<'f>(fields: &'f Map<String, Value>, n: usize, keys: [&'f str; 2]) -> Result<&'f str> {
    match keys.map(|k| fields.contains_key(k)) {
        [true, false] => Ok(keys[0]),
        [false, true] => Ok(keys[1]),
        _ => conflict(format!(
            "line {n}: give exactly one of {} and {}.",
            keys[0], keys[1]
        )),
    }
}

fn basic_facts(fields: &Map<String, Value>, n: usize) -> Result<Basic> {
    let litres = quantity(fields, n, "litres", 4)?;
    let (km, km_basis) = match one_of(fields, n, ["public_road_km", "total_km"])? {
        "public_road_km" => (quantity(fields, n, "public_road_km", 4)?, "actual"),
        _ => {
            let total = quantity(fields, n, "total_km", 4)?;
            (
                total.mul(&crate::engine::d(DERIVED_KM_SHARE))?,
                "derived: total_km x 98.4%",
            )
        }
    };
    let (litres_per_km, rate_basis) = consumption_rate(fields, n)?;
    Ok(Basic {
        litres,
        km,
        km_basis,
        litres_per_km,
        rate_basis,
    })
}

/// The diesel consumption rate in litres per kilometre and its basis.
fn consumption_rate(fields: &Map<String, Value>, n: usize) -> Result<(Decimal, &'static str)> {
    if one_of(
        fields,
        n,
        ["rigid_truck_gross_weight_tonnes", "litres_per_km"],
    )? == "litres_per_km"
    {
        let rate = quantity(fields, n, "litres_per_km", 6)?;
        if rate.is_zero() {
            return refuse(
                "fuel_tax_credits.quantity_invalid",
                format!("line {n}: litres_per_km must be above 0."),
            );
        }
        return Ok((
            rate,
            "engine diagnostic 'driving fuel' economy (PCG 2021/2 para 9)",
        ));
    }
    let weight = quantity(fields, n, "rigid_truck_gross_weight_tonnes", 3)?;
    if !weight.gt(&crate::engine::d("4.5"))? {
        return refuse(
            "fuel_tax_credits.simplified_method_unavailable",
            format!(
                "line {n}: a rigid truck of 4.5 tonnes or less is not a heavy vehicle, so the basic method does not apply (PCG 2021/2 para 4)."
            ),
        );
    }
    let mut found = RIGID_TRUCK_RATES[4].1;
    for (bound, rate) in RIGID_TRUCK_RATES {
        if let Some(b) = bound
            && !weight.gt(&crate::engine::d(b))?
        {
            found = rate;
            break;
        }
    }
    Ok((
        crate::engine::d(found),
        "rigid truck by gross weight: the ATO's 'Basic method for heavy vehicles' (QC64655) Table 1, from PCG 2021/2 Schedule 1",
    ))
}

/// PCG 2016/2 Method 1's total spend (AUD, GST inclusive) and the average
/// GST inclusive price for the fuel type in the area of operation, in cents
/// per litre (para 9).
fn cost_and_price(fields: &Map<String, Value>, n: usize) -> Result<(Decimal, Decimal)> {
    let cost = match fields.get("cost") {
        None => return refuse("input.missing_field", format!("line {n}: cost is required")),
        Some(Value::String(text)) if text.len() <= 32 => Decimal::parse_input(text)
            .ok()
            .and_then(|v| money(&Arg::Decimal(v)).ok()),
        Some(_) => None,
    };
    let Some(cost) = cost else {
        return refuse(
            "input.amount_invalid",
            format!(
                "line {n}: cost must be a decimal string, finite and non-negative, at most 2dp and 1e12: the GST inclusive spend on the fuel."
            ),
        );
    };
    let price = quantity(fields, n, "average_price_cents_per_litre", 4)?;
    if price.is_zero() {
        return refuse(
            "fuel_tax_credits.quantity_invalid",
            format!("line {n}: average_price_cents_per_litre must be above 0."),
        );
    }
    Ok((cost, price))
}

pub(super) fn parse_line(value: &Value, n: usize) -> Result<Line> {
    let Some(fields) = value.as_object() else {
        return refuse(
            "fuel_tax_credits.lines_invalid",
            format!("line {n} is not a JSON object."),
        );
    };
    let method = match fields.get("method") {
        None => None,
        Some(_) => match choice(fields, n, "method", &METHODS)? {
            Some(m) => Some(m),
            None => {
                return refuse(
                    "fuel_tax_credits.method_invalid",
                    expected(n, "method", &METHODS),
                );
            }
        },
    };
    let (fuel, category, unit) = line_fuel(fields, n, method)?;
    let acquired = match fields.get("acquired") {
        None => {
            return refuse(
                "input.missing_field",
                format!("line {n}: acquired is required"),
            );
        }
        Some(v) => date_arg(
            &Arg::Text(v.as_str().unwrap_or("").into()),
            &format!("line {n}: acquired"),
        )?,
    };
    let work = if method == Some("basic_heavy_vehicle") {
        Work::Basic(basic_facts(fields, n)?)
    } else {
        line_quantity(fields, n, method)?
    };
    Ok(Line {
        fuel,
        category,
        unit,
        acquired,
        work,
    })
}

/// The method's allowed fields checked, then its fuel, category and unit.
fn line_fuel(
    fields: &Map<String, Value>,
    n: usize,
    method: Option<&'static str>,
) -> Result<(&'static str, usize, &'static str)> {
    let allowed: &[&str] = match method {
        None => &QUANTITY_FIELDS,
        Some("basic_heavy_vehicle") => &BASIC_FIELDS,
        Some(_) => &FROM_COST_FIELDS,
    };
    if let Some(key) = fields.keys().find(|k| !allowed.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("line {n}: {key} is not an input of this line"),
        );
    }
    let names = FUELS.map(|(f, _, _)| f);
    let fuels: &[&str] = match method {
        None => &names,
        Some("basic_heavy_vehicle") => &["diesel"],
        Some(_) => &LIQUID_FUELS,
    };
    let Some(fuel) = choice(fields, n, "fuel", fuels)? else {
        let why = match method {
            None => return refuse("fuel_tax_credits.fuel_invalid", expected(n, "fuel", &names)),
            Some("basic_heavy_vehicle") => {
                "the basic method for heavy vehicles is for diesel only (PCG 2021/2 para 5)"
            }
            Some(_) => {
                "PCG 2016/2 Method 1 is for liquid taxable fuels only (para 4(b)), not LPG, LNG or CNG"
            }
        };
        return refuse(
            "fuel_tax_credits.simplified_method_unavailable",
            format!("line {n}: {why}."),
        );
    };
    Ok(*FUELS.iter().find(|(f, _, _)| *f == fuel).expect("a fuel"))
}

/// A quantity line's use and quantity, stated or (Method 1) worked out from
/// the cost.
fn line_quantity(
    fields: &Map<String, Value>,
    n: usize,
    method: Option<&'static str>,
) -> Result<Work> {
    let Some(use_) = choice(fields, n, "use", &USES)? else {
        return refuse("fuel_tax_credits.use_invalid", expected(n, "use", &USES));
    };
    let from_cost = match method {
        None => None,
        Some(_) => Some(cost_and_price(fields, n)?),
    };
    let quantity = match &from_cost {
        None => quantity(fields, n, "quantity", 4)?,
        Some((cost, price)) => cost.mul(&crate::engine::int(100))?.div(price)?,
    };
    Ok(Work::Quantity {
        quantity,
        use_,
        from_cost,
    })
}
