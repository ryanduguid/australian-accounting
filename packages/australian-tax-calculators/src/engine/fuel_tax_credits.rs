// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `fuel_tax_credits`: the fuel tax credits for business fuel acquired from
//! 1 July 2022, line by line at the ATO's published rate for the day of
//! acquisition, with the basic method for heavy vehicles (PCG 2021/2) and the
//! period-end rate of PCG 2016/2 Method 2. The rates are
//! `data/fuel-tax-credit-rates.json`, each period with its ATO page and the
//! day it was read. The citations are to the Fuel Tax Act 2006 at the ATO's
//! legal database, PCG 2016/2, PCG 2021/2 and the ATO's pages QC44494,
//! QC44511, QC48520 and QC64655, all read on 19 November 2026.

use std::sync::OnceLock;

use serde_json::{Map, Value, json};

use super::{Arg, Result, date_arg, flag, money, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "fuel_tax_credits";
const MAX_LINES: usize = 1000;

/// The ATO's 6 rows of eligible fuel types, in the data file's keys.
const CATEGORIES: [&str; 6] = ["liquid", "blend", "e85", "lpg", "lng_cng", "b100"];

/// Each fuel a line may name, its ATO row and the unit its quantity is in.
const FUELS: [(&str, usize, &str); 9] = [
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

const USES: [&str; 3] = [
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

const DERIVED_KM_SHARE: &str = "0.984";

const CREDIT: &str = "Fuel Tax Act 2006 ss 41-5 and 43-5(1), (2) and (2A) item 1: the credit is the effective fuel tax at the rate in force on the day the fuel was acquired; quantity x rate in cents / 100, rounded to the cent (the ATO's 'Working out your fuel tax credits', QC44511, step 3)";
const HEAVY: &str = "Fuel Tax Act 2006 s 43-10(3) and (6): for fuel used in a vehicle of more than 4.5 tonnes GVM travelling on a public road the credit is reduced by the road user charge, which the ATO's 'heavy vehicles for travelling on public roads' rate takes off";
const OTHER: &str = "Fuel Tax Act 2006 s 41-5: all other business uses, including the auxiliary equipment of a heavy vehicle, travel off public roads, and a heavy vehicle whose public road travel is incidental to its main use (s 43-10(4))";
const LIGHT: &str = "Fuel Tax Act 2006 s 41-20: no credit for fuel to be used in a vehicle with a gross vehicle mass of 4.5 tonnes or less travelling on a public road";
const BASIC: &str = "PCG 2021/2 paras 9 to 11: public road litres = public road kilometres (actual, or total kilometres x 98.4%) x the diesel consumption rate (Schedule 1, or the vehicle's engine diagnostic 'driving fuel' economy); other litres = total eligible diesel less public road litres; each at its rate, rounded to the cent";
const METHOD_1: &str = "PCG 2016/2 para 9 (Method 1): litres = the total GST inclusive spend on the fuel / the average GST inclusive price of the fuel type in the area of operation for the tax period, as the Australian Institute of Petroleum publishes it (https://aip.com.au/pricing/pump-prices-retail/); quantity is shown to 4 decimal places and the credit uses the exact quotient";
const LABEL_7D: &str = "The ATO's 'Working out your fuel tax credits' (QC44511) step 3: add the period's amounts and claim the whole dollar amount at label 7D; its 'Basic method for heavy vehicles' (QC64655) examples drop the cents ($1,885.95 is claimed as $1,885)";

/// One published rate period.
struct Period {
    from: i64,
    to: i64,
    starts_with: String,
    source: Value,
    heavy: Vec<Decimal>,
    other: Vec<Decimal>,
}

struct Rates {
    read: String,
    periods: Vec<Period>,
}

fn rates() -> &'static Rates {
    static RATES: OnceLock<Rates> = OnceLock::new();
    RATES.get_or_init(|| parse_rates(include_str!("../../data/fuel-tax-credit-rates.json")))
}

/// The bundled rates: contiguous periods from 1 July 2022, 6 fuel rows in 2
/// uses each. A malformed file is a build defect, so it panics.
fn parse_rates(text: &str) -> Rates {
    let data: Value = serde_json::from_str(text).expect("JSON rates");
    let day = |v: &Value| dates::parse(v.as_str().expect("a date")).expect("YYYY-MM-DD");
    let row = |v: &Value| -> Vec<Decimal> {
        CATEGORIES
            .iter()
            .map(|c| Decimal::parse(v[*c].as_str().expect("a rate")).expect("a decimal rate"))
            .collect()
    };
    let known = data["starts_with"]
        .as_object()
        .expect("period start reasons");
    let mut periods: Vec<Period> = Vec::new();
    for p in data["periods"].as_array().expect("periods") {
        let (from, to) = (day(&p["from"]), day(&p["to"]));
        assert!(from <= to, "{from}: ordered");
        assert!(
            periods.last().is_none_or(|last| last.to + 1 == from),
            "{from}: contiguous"
        );
        let starts_with = p["starts_with"].as_str().expect("a reason").to_string();
        assert!(
            known.contains_key(&starts_with),
            "{starts_with}: a known reason"
        );
        for key in ["page", "table", "url", "updated", "read"] {
            assert!(p["source"][key].is_string(), "{from}: source {key}");
        }
        periods.push(Period {
            from,
            to,
            starts_with,
            source: p["source"].clone(),
            heavy: row(&p["cents"]["heavy_vehicle_public_road"]),
            other: row(&p["cents"]["other_business"]),
        });
    }
    assert_eq!(periods[0].from, dates::parse("2022-07-01").expect("date"));
    let read = data["index"]["read"]
        .as_str()
        .expect("read date")
        .to_string();
    Rates { read, periods }
}

/// The facts as the command passes them; see the README for each.
pub struct FuelArgs<'a> {
    pub lines: &'a Arg,
    pub small_claimant: Option<&'a Arg>,
    pub rate_at_period_end: Option<&'a Arg>,
    pub return_date: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn conflict<T>(message: String) -> Result<T> {
    refuse("fuel_tax_credits.fact_conflict", message)
}

fn cents(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(-2, Rounding::HalfUp)?)
}

/// `quantity x rate / 100`, the credit in dollars to the cent.
fn credit(quantity: &Decimal, rate_cents: &Decimal) -> Result<Decimal> {
    cents(&quantity.mul(rate_cents)?.div(&super::int(100))?)
}

/// An exact value without trailing zeros after the point.
fn shown(value: &Decimal) -> String {
    let text = value.fixed_format();
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text
    }
}

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
        Some(Value::String(text)) if text.len() <= 32 => Decimal::parse(text).ok(),
        Some(_) => None,
    };
    let valid = |v: &Decimal| -> Result<bool> {
        Ok(v.is_finite()
            && !v.lt(&super::int(0))?
            && !v.gt(&super::d("1e12"))?
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
struct Basic {
    litres: Decimal,
    km: Decimal,
    km_basis: &'static str,
    litres_per_km: Decimal,
    rate_basis: &'static str,
}

/// A line's checked facts.
struct Line {
    fuel: &'static str,
    category: usize,
    unit: &'static str,
    acquired: i64,
    work: Work,
}

/// A line by quantity and use (the quantity given, or worked out from the
/// cost and the average price under PCG 2016/2 Method 1), or by the basic
/// method.
enum Work {
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
                total.mul(&super::d(DERIVED_KM_SHARE))?,
                "derived: total_km x 98.4%",
            )
        }
    };
    let (litres_per_km, rate_basis) = match one_of(
        fields,
        n,
        ["rigid_truck_gross_weight_tonnes", "litres_per_km"],
    )? {
        "litres_per_km" => {
            let rate = quantity(fields, n, "litres_per_km", 6)?;
            if rate.is_zero() {
                return refuse(
                    "fuel_tax_credits.quantity_invalid",
                    format!("line {n}: litres_per_km must be above 0."),
                );
            }
            (
                rate,
                "engine diagnostic 'driving fuel' economy (PCG 2021/2 para 9)",
            )
        }
        _ => {
            let weight = quantity(fields, n, "rigid_truck_gross_weight_tonnes", 3)?;
            if !weight.gt(&super::d("4.5"))? {
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
                    && !weight.gt(&super::d(b))?
                {
                    found = rate;
                    break;
                }
            }
            (
                super::d(found),
                "rigid truck by gross weight: the ATO's 'Basic method for heavy vehicles' (QC64655) Table 1, from PCG 2021/2 Schedule 1",
            )
        }
    };
    Ok(Basic {
        litres,
        km,
        km_basis,
        litres_per_km,
        rate_basis,
    })
}

/// PCG 2016/2 Method 1's total spend (AUD, GST inclusive) and the average
/// GST inclusive price for the fuel type in the area of operation, in cents
/// per litre (para 9).
fn cost_and_price(fields: &Map<String, Value>, n: usize) -> Result<(Decimal, Decimal)> {
    let cost = match fields.get("cost") {
        None => return refuse("input.missing_field", format!("line {n}: cost is required")),
        Some(Value::String(text)) if text.len() <= 32 => Decimal::parse(text)
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

fn parse_line(value: &Value, n: usize) -> Result<Line> {
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
    let (_, category, unit) = *FUELS.iter().find(|(f, _, _)| *f == fuel).expect("a fuel");
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
        let Some(use_) = choice(fields, n, "use", &USES)? else {
            return refuse("fuel_tax_credits.use_invalid", expected(n, "use", &USES));
        };
        let from_cost = match method {
            None => None,
            Some(_) => Some(cost_and_price(fields, n)?),
        };
        let quantity = match &from_cost {
            None => quantity(fields, n, "quantity", 4)?,
            Some((cost, price)) => cost.mul(&super::int(100))?.div(price)?,
        };
        Work::Quantity {
            quantity,
            use_,
            from_cost,
        }
    };
    Ok(Line {
        fuel,
        category,
        unit,
        acquired,
        work,
    })
}

/// The period whose rates apply on `day`, which lies in the published range.
fn period_on(day: i64) -> &'static Period {
    rates()
        .periods
        .iter()
        .find(|p| p.from <= day && day <= p.to)
        .expect("a day in the published range")
}

fn period_json(p: &Period) -> Value {
    json!({"from": dates::format(p.from), "to": dates::format(p.to), "source": p.source})
}

/// PCG 2016/2 Method 2: the tax period whose last day sets every line's
/// rate, checked against the lines and the rate changes inside it.
fn period_end_day(given: &Arg, year: (i64, i64), lines: &[Line]) -> Result<i64> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "fuel_tax_credits.period_invalid",
            "rate_at_period_end must be an object with start and end, the tax period's first and last days.",
        );
    };
    if let Some(key) = fields
        .keys()
        .find(|k| !matches!(k.as_str(), "start" | "end"))
    {
        return refuse(
            "input.field_unknown",
            format!("rate_at_period_end: {key} is not an input"),
        );
    }
    let read = |key: &str| -> Result<i64> {
        let text = fields.get(key).and_then(Value::as_str).unwrap_or("");
        date_arg(
            &Arg::Text(text.into()),
            &format!("rate_at_period_end: {key}"),
        )
    };
    let (start, end) = (read("start")?, read("end")?);
    if start > end || start < year.0 || end > year.1 {
        return conflict(
            "rate_at_period_end: the tax period must start on or before it ends and lie inside the income year.".into(),
        );
    }
    for (n, line) in lines.iter().enumerate() {
        if line.acquired < start || line.acquired > end {
            return conflict(format!(
                "line {}: acquired is outside the tax period of rate_at_period_end; PCG 2016/2 Method 2 takes the fuel acquired in the period.",
                n + 1
            ));
        }
        if matches!(line.fuel, "lpg" | "lng_cng") {
            return refuse(
                "fuel_tax_credits.simplified_method_unavailable",
                format!(
                    "line {}: PCG 2016/2 covers liquid taxable fuels only (para 4(b)), not {}.",
                    n + 1,
                    line.fuel
                ),
            );
        }
    }
    let changes = rates()
        .periods
        .iter()
        .filter(|p| p.from > start && p.from <= end);
    if let Some(p) = changes.clone().find(|p| p.starts_with != "indexation") {
        return refuse(
            "fuel_tax_credits.simplified_method_unavailable",
            format!(
                "The rates change on {} for a reason other than indexation alone ({}), so PCG 2016/2 Method 2 (para 11: 'because of indexation') does not apply: give each line's own acquisition date instead.",
                dates::format(p.from),
                p.starts_with
            ),
        );
    }
    Ok(end)
}

/// The income year's first and last days for `year`, `YYYY-YY`.
fn year_days(year: &str) -> (i64, i64) {
    let first: i64 = year[..4].parse().expect("income year");
    let day = |y: i64, md: &str| dates::parse(&format!("{y}-{md}")).expect("date");
    (day(first, "07-01"), day(first + 1, "06-30"))
}

fn work_line(line: &Line, n: usize, rate_day: i64) -> Result<(Value, Decimal)> {
    let p = period_on(rate_day);
    let (heavy, other) = (&p.heavy[line.category], &p.other[line.category]);
    let mut out = json!({
        "line": n,
        "fuel": line.fuel,
        "acquired": dates::format(line.acquired),
        "rate_day": dates::format(rate_day),
        "rate_period": period_json(p),
    });
    let total = match &line.work {
        Work::Quantity {
            quantity: qty,
            use_,
            from_cost,
        } => {
            let (rate_cents, authority) = match *use_ {
                "heavy_vehicle_public_road" => (heavy.clone(), HEAVY),
                "other_business" => (other.clone(), OTHER),
                _ => (super::int(0), LIGHT),
            };
            let amount = credit(qty, &rate_cents)?;
            out["use"] = json!(use_);
            if let Some((cost, price)) = from_cost {
                out["method"] = json!("litres_from_cost");
                out["cost"] = json!(cost.to_string());
                out["average_price_cents_per_litre"] = json!(shown(price));
                out["litres_from_cost_authority"] = json!(METHOD_1);
            }
            out["quantity"] = json!(if from_cost.is_some() {
                qty.quantize(-4, Rounding::HalfUp)?.to_string()
            } else {
                shown(qty)
            });
            out["unit"] = json!(line.unit);
            out["rate_cents"] = json!(shown(&rate_cents));
            out["authority"] = json!([CREDIT, authority]);
            amount
        }
        Work::Basic(b) => {
            let public = b.km.mul(&b.litres_per_km)?;
            if public.gt(&b.litres)? {
                return conflict(format!(
                    "line {n}: the public road litres ({} km x {} L/km = {}) exceed the {} litres of eligible diesel; check the kilometres and the consumption rate.",
                    shown(&b.km),
                    shown(&b.litres_per_km),
                    shown(&public),
                    shown(&b.litres)
                ));
            }
            let off = b.litres.sub(&public)?;
            let (on_credit, off_credit) = (credit(&public, heavy)?, credit(&off, other)?);
            let amount = on_credit.add(&off_credit)?;
            out["method"] = json!("basic_heavy_vehicle");
            out["litres"] = json!(shown(&b.litres));
            out["public_road_km"] = json!(shown(&b.km));
            out["public_road_km_basis"] = json!(b.km_basis);
            out["litres_per_km"] = json!(shown(&b.litres_per_km));
            out["litres_per_km_basis"] = json!(b.rate_basis);
            out["public_road_litres"] = json!(shown(&public));
            out["other_litres"] = json!(shown(&off));
            out["public_road_rate_cents"] = json!(shown(heavy));
            out["other_rate_cents"] = json!(shown(other));
            out["public_road_credit"] = json!(on_credit.to_string());
            out["other_credit"] = json!(off_credit.to_string());
            out["authority"] = json!([CREDIT, HEAVY, BASIC]);
            amount
        }
    };
    out["credit"] = json!(total.to_string());
    Ok((out, total))
}

pub fn fuel_tax_credits(args: &FuelArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let Arg::Json(Value::Array(items)) = args.lines else {
        return refuse(
            "fuel_tax_credits.lines_invalid",
            "lines must be a list of line objects.",
        );
    };
    if items.is_empty() || items.len() > MAX_LINES {
        return refuse(
            "fuel_tax_credits.lines_invalid",
            format!("lines must hold from 1 to {MAX_LINES} lines."),
        );
    }
    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, v)| parse_line(v, i + 1))
        .collect::<Result<_>>()?;
    let bounds = year_days(&year);
    let last = rates().periods.last().expect("a period").to;
    for (i, line) in lines.iter().enumerate() {
        if line.acquired < bounds.0 || line.acquired > bounds.1 {
            return refuse(
                "fuel_tax_credits.date_outside_year",
                format!(
                    "line {}: acquired is outside the income year {year}; run each income year apart.",
                    i + 1
                ),
            );
        }
        if line.acquired > last {
            return refuse(
                "fuel_tax_credits.rate_unpublished",
                format!(
                    "line {}: no published rate is held for fuel acquired after {}; the rates change at the February 2027 indexation and were not published when this was checked on {}.",
                    i + 1,
                    dates::format(last),
                    rates().read
                ),
            );
        }
    }
    let simplified = lines.iter().any(|l| {
        matches!(
            l.work,
            Work::Basic(_)
                | Work::Quantity {
                    from_cost: Some(_),
                    ..
                }
        )
    }) || args.rate_at_period_end.is_some();
    match (simplified, args.small_claimant) {
        (true, None) => {
            return refuse(
                "input.missing_field",
                "small_claimant is required with a basic_heavy_vehicle or litres_from_cost line or rate_at_period_end.",
            );
        }
        (true, Some(given)) if !flag(given, "small_claimant")? => {
            return refuse(
                "fuel_tax_credits.simplified_method_unavailable",
                "The simplified methods are for an entity whose fuel tax credits were $10,000 or less in the previous 12 months and are expected to be in the current 12 months (PCG 2016/2 para 4; PCG 2021/2 para 6).",
            );
        }
        (false, Some(_)) => {
            return refuse(
                "input.field_unknown",
                "small_claimant is an input only with a basic_heavy_vehicle or litres_from_cost line or rate_at_period_end",
            );
        }
        _ => {}
    }
    let end_day = match args.rate_at_period_end {
        Some(given) => Some(period_end_day(given, bounds, &lines)?),
        None => None,
    };
    let mut worked = Vec::new();
    let mut total = super::int(0);
    for (i, line) in lines.iter().enumerate() {
        let (out, amount) = work_line(line, i + 1, end_day.unwrap_or(line.acquired))?;
        total = total.add(&amount)?;
        worked.push(out);
    }
    let label = total.quantize(0, Rounding::Down)?;
    let read = dates::parse(&rates().read).expect("read date");
    let mut out = result(
        KIND,
        &year,
        vec![("fuel_tax_credits", total), ("label_7d", label)],
        vec![
            rate("rates_read", &rates().read),
            rate("rates_from", "2022-07-01"),
            rate("rates_to", dates::format(last)),
        ],
    )?;
    out["lines"] = Value::Array(worked);
    out["label_7d_authority"] = json!(LABEL_7D);
    let warnings = out["warnings"].as_array_mut().expect("warnings");
    if let Some(day) = end_day {
        warnings.push(json!(format!(
            "PCG 2016/2 Method 2: every line takes the rate in force on {}, the tax period's last day (para 11).",
            dates::format(day)
        )));
    }
    if lines.iter().any(|l| l.acquired > read) {
        warnings.push(json!(format!(
            "Some fuel was acquired after {}, when the rates were last read: check the ATO's 'Rates - business' page for a change since.",
            rates().read
        )));
    }
    if let Some(given) = args.return_date {
        let day = date_arg(given, "return_date")?;
        let late = time_limit(&lines, day)?;
        if !late.is_empty() {
            warnings.push(json!(format!(
                "Fuel Tax Act 2006 s 47-5(2): the entitlement for line(s) {} ceases unless a return taking it into account is given within 4 years after the day of acquisition, and the return is given on {}. That does not apply to a credit already taken into account in an assessment (Linfox Australia Pty Ltd v Commissioner of Taxation [2019] FCAFC 131; MT 2024/1) or to the s 47-10 exception; otherwise remove those lines.",
                late.join(", "),
                dates::format(day)
            )));
        }
        out["time_limit_lines"] = json!(late);
    }
    Ok(out)
}

/// The numbers of the lines acquired more than 4 years before `return_day`,
/// refusing a return before an acquisition.
fn time_limit(lines: &[Line], return_day: i64) -> Result<Vec<String>> {
    let mut late = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if return_day < line.acquired {
            return conflict(format!(
                "line {}: return_date is before the fuel was acquired.",
                i + 1
            ));
        }
        if return_day > dates::add_months(line.acquired, 48) {
            late.push((i + 1).to_string());
        }
    }
    Ok(late)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PCG 2021/2's own example (paras 19 to 24) and the ATO's Jim (QC64655),
    /// at December 2020 rates the data file does not hold: the method's
    /// arithmetic on the guideline's figures.
    #[test]
    fn the_basic_method_reproduces_the_guideline_examples() {
        let split = |litres: &str, km: &str, rate: &str| {
            let public = crate::engine::d(km).mul(&crate::engine::d(rate)).unwrap();
            let off = crate::engine::d(litres).sub(&public).unwrap();
            let on = credit(&public, &crate::engine::d("16.5")).unwrap();
            let off = credit(&off, &crate::engine::d("42.3")).unwrap();
            (
                on.to_string(),
                off.to_string(),
                on.add(&off).unwrap().to_string(),
            )
        };
        // We Deliver: 30,550 actual public road km at 0.35 L/km.
        assert_eq!(
            split("11430", "30550", "0.35"),
            ("1764.26".into(), "311.96".into(), "2076.22".into())
        );
        // Jim: 32,000 total km x 98.4% = 31,488 at 0.35 L/km; $1,991 at 7D.
        let derived = crate::engine::d("32000")
            .mul(&crate::engine::d(DERIVED_KM_SHARE))
            .unwrap();
        let (on, off, total) = split("11430", &derived.to_string(), "0.35");
        assert_eq!((on.as_str(), off.as_str()), ("1818.43", "173.09"));
        let label = crate::engine::d(&total)
            .quantize(0, Rounding::Down)
            .unwrap();
        assert_eq!(label.to_string(), "1991");
    }

    #[test]
    fn the_bundled_rates_are_whole_and_contiguous() {
        let r = rates();
        assert_eq!(r.periods.len(), 16);
        assert_eq!(dates::format(r.periods[15].to), "2027-01-31");
        assert!(
            r.periods
                .iter()
                .all(|p| p.heavy.len() == 6 && p.other.len() == 6)
        );
    }
}
