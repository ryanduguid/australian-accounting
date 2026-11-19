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

use serde_json::{Value, json};

mod input;
mod rates;
use input::{Basic, Line, Work, parse_line};
use rates::{period_json, period_on, rates};

use super::{Arg, Result, date_arg, flag, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "fuel_tax_credits";
const MAX_LINES: usize = 1000;

const CREDIT: &str = "Fuel Tax Act 2006 ss 41-5 and 43-5(1), (2) and (2A) item 1: the credit is the effective fuel tax at the rate in force on the day the fuel was acquired; quantity x rate in cents / 100, rounded to the cent (the ATO's 'Working out your fuel tax credits', QC44511, step 3)";
const HEAVY: &str = "Fuel Tax Act 2006 s 43-10(3) and (6): for fuel used in a vehicle of more than 4.5 tonnes GVM travelling on a public road the credit is reduced by the road user charge, which the ATO's 'heavy vehicles for travelling on public roads' rate takes off";
const OTHER: &str = "Fuel Tax Act 2006 s 41-5: all other business uses, including the auxiliary equipment of a heavy vehicle, travel off public roads, and a heavy vehicle whose public road travel is incidental to its main use (s 43-10(4))";
const LIGHT: &str = "Fuel Tax Act 2006 s 41-20: no credit for fuel to be used in a vehicle with a gross vehicle mass of 4.5 tonnes or less travelling on a public road";
const BASIC: &str = "PCG 2021/2 paras 9 to 11: public road litres = public road kilometres (actual, or total kilometres x 98.4%) x the diesel consumption rate (Schedule 1, or the vehicle's engine diagnostic 'driving fuel' economy); other litres = total eligible diesel less public road litres; each at its rate, rounded to the cent";
const METHOD_1: &str = "PCG 2016/2 para 9 (Method 1): litres = the total GST inclusive spend on the fuel / the average GST inclusive price of the fuel type in the area of operation for the tax period, as the Australian Institute of Petroleum publishes it (https://aip.com.au/pricing/pump-prices-retail/); quantity is shown to 4 decimal places and the credit uses the exact quotient";
const LABEL_7D: &str = "The ATO's 'Working out your fuel tax credits' (QC44511) step 3: add the period's amounts and claim the whole dollar amount at label 7D; its 'Basic method for heavy vehicles' (QC64655) examples drop the cents ($1,885.95 is claimed as $1,885)";

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

/// PCG 2016/2 Method 2: the tax period whose last day sets every line's
/// rate, checked against the lines and the rate changes inside it.
fn period_end_day(given: &Arg, year: (i64, i64), lines: &[Line]) -> Result<i64> {
    let (start, end) = period_days(given)?;
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
    indexation_only(start, end)?;
    Ok(end)
}

/// `rate_at_period_end`'s start and end days.
fn period_days(given: &Arg) -> Result<(i64, i64)> {
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
    Ok((read("start")?, read("end")?))
}

/// Refuses Method 2 when a rate change after `start` and up to `end` is
/// other than indexation.
fn indexation_only(start: i64, end: i64) -> Result<()> {
    let mut changes = rates()
        .periods
        .iter()
        .filter(|p| p.from > start && p.from <= end);
    if let Some(p) = changes.find(|p| p.starts_with != "indexation") {
        return refuse(
            "fuel_tax_credits.simplified_method_unavailable",
            format!(
                "The rates change on {} for a reason other than indexation alone ({}), so PCG 2016/2 Method 2 (para 11: 'because of indexation') does not apply: give each line's own acquisition date instead.",
                dates::format(p.from),
                p.starts_with
            ),
        );
    }
    Ok(())
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
            quantity,
            use_,
            from_cost,
        } => {
            let (rate_cents, authority) = match *use_ {
                "heavy_vehicle_public_road" => (heavy.clone(), HEAVY),
                "other_business" => (other.clone(), OTHER),
                _ => (super::int(0), LIGHT),
            };
            out["use"] = json!(use_);
            if let Some((cost, price)) = from_cost {
                out["method"] = json!("litres_from_cost");
                out["cost"] = json!(cost.to_string());
                out["average_price_cents_per_litre"] = json!(shown(price));
                out["litres_from_cost_authority"] = json!(METHOD_1);
            }
            out["quantity"] = json!(if from_cost.is_some() {
                quantity.quantize(-4, Rounding::HalfUp)?.to_string()
            } else {
                shown(quantity)
            });
            out["unit"] = json!(line.unit);
            out["rate_cents"] = json!(shown(&rate_cents));
            out["authority"] = json!([CREDIT, authority]);
            credit(quantity, &rate_cents)?
        }
        Work::Basic(b) => work_basic(&mut out, n, b, heavy, other)?,
    };
    out["credit"] = json!(total.to_string());
    Ok((out, total))
}

/// PCG 2021/2's split of a heavy vehicle's diesel into public road and
/// other litres, each at its rate; adds the working to `out`.
fn work_basic(
    out: &mut Value,
    n: usize,
    b: &Basic,
    heavy: &Decimal,
    other: &Decimal,
) -> Result<Decimal> {
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
    Ok(on_credit.add(&off_credit)?)
}

pub fn fuel_tax_credits(args: &FuelArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let bounds = year_days(&year);
    let lines = read_lines(args.lines, bounds, &year)?;
    check_small_claimant(args, &lines)?;
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
    let last = rates().periods.last().expect("a period").to;
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
    add_warnings(&mut out, args, &lines, end_day)?;
    Ok(out)
}

/// The checked lines, each acquired inside the income year and on or
/// before the last day a rate is held for.
fn read_lines(given: &Arg, bounds: (i64, i64), year: &str) -> Result<Vec<Line>> {
    let Arg::Json(Value::Array(items)) = given else {
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
    Ok(lines)
}

/// `small_claimant` is required, and must be true, with a simplified
/// method, and is not an input otherwise.
fn check_small_claimant(args: &FuelArgs, lines: &[Line]) -> Result<()> {
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
    Ok(())
}

/// The Method 2, read date and s 47-5(2) warnings, and `time_limit_lines`.
fn add_warnings(
    out: &mut Value,
    args: &FuelArgs,
    lines: &[Line],
    end_day: Option<i64>,
) -> Result<()> {
    let read = dates::parse(&rates().read).expect("read date");
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
        let late = time_limit(lines, day)?;
        if !late.is_empty() {
            warnings.push(json!(format!(
                "Fuel Tax Act 2006 s 47-5(2): the entitlement for line(s) {} ceases unless a return taking it into account is given within 4 years after the day of acquisition, and the return is given on {}. That does not apply to a credit already taken into account in an assessment (Linfox Australia Pty Ltd v Commissioner of Taxation [2019] FCAFC 131; MT 2024/1) or to the s 47-10 exception; otherwise remove those lines.",
                late.join(", "),
                dates::format(day)
            )));
        }
        out["time_limit_lines"] = json!(late);
    }
    Ok(())
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
mod tests;
