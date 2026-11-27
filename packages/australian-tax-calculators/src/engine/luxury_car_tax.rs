// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `luxury_car_tax`: the luxury car tax on a tax period's taxable supplies
//! and importations of luxury cars, its adjustments and the primary producer
//! and tourism operator refund, with the amounts for BAS labels 1E and 1F,
//! for financial years 2012-13 to 2026-27. The citations are to the A New Tax
//! System (Luxury Car Tax) Act 1999 (compilation 27, 1 April 2025, at the
//! Federal Register of Legislation, and the ATO's legal database), its
//! Imposition (General and Customs) Acts s 4, the Treasury Laws Amendment
//! (Tax Incentives and Integrity) Act 2025 (No. 29, 2025) Schedule 1, and
//! the ATO's pages QC38161, QC22107, QC22108, QC22109, QC39467, QC39921,
//! QC33694 and QC17458, all read on 19 November 2026.

use serde_json::{Value, json};

mod input;
use input::{Car, Line, parse_line};

use super::{Arg, Result, d, int, rate, refuse, result, scope};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "luxury_car_tax";
const MAX_LINES: usize = 1000;
const LCT_RATE: &str = "0.33";
const REFUND_CAP: i64 = 10_000;

/// The luxury car tax thresholds the ATO publishes, by the financial year
/// of the supply or importation.
pub(crate) mod thresholds {
    /// (financial year, fuel-efficient car limit, luxury car tax threshold),
    /// from the ATO's 'Luxury car tax rate and thresholds' (QC38161, last
    /// updated 1 June 2026), read on 19 November 2026. The 2026-27 figures
    /// are LCT Act s 25-1(3) and (5) indexed by the ATO's factor of 1.003:
    /// $80,567 x 1.003 = $80,808.70 and $91,387 x 1.003 = $91,661.16, to
    /// whole dollars.
    pub(crate) const TABLE: [(&str, &str, &str); 15] = [
        ("2012-13", "75375", "59133"),
        ("2013-14", "75375", "60316"),
        ("2014-15", "75375", "61884"),
        ("2015-16", "75375", "63184"),
        ("2016-17", "75526", "64132"),
        ("2017-18", "75526", "65094"),
        ("2018-19", "75526", "66331"),
        ("2019-20", "75526", "67525"),
        ("2020-21", "77565", "68740"),
        ("2021-22", "79659", "69152"),
        ("2022-23", "84916", "71849"),
        ("2023-24", "89332", "76950"),
        ("2024-25", "91387", "80567"),
        ("2025-26", "91387", "80567"),
        ("2026-27", "91661", "80809"),
    ];

    /// `given` as a year the table holds.
    pub(crate) fn year(given: &str) -> Option<&'static str> {
        TABLE
            .iter()
            .find(|(y, _, _)| *y == given)
            .map(|(y, _, _)| *y)
    }

    pub(crate) fn first_year() -> &'static str {
        TABLE[0].0
    }

    /// The fuel-efficient car limit and the general threshold for `year`.
    pub(crate) fn of(year: &str) -> (&'static str, &'static str) {
        let (_, efficient, general) = TABLE
            .iter()
            .find(|(y, _, _)| *y == year)
            .expect("a year the table holds");
        (efficient, general)
    }
}

const FORMULA: &str = "LCT Act s 5-15(1) (supplies) and s 7-15(1) (importations): rate x 10/11 x (luxury car tax value - luxury car tax threshold), the rate 33% (Luxury Car Tax Imposition (General) Act 1999 s 4 and (Customs) Act 1999 s 4); nil where the value does not exceed the threshold, the car then not being a luxury car (s 25-1(1)); rounded half up to the cent";
const GENERAL: &str = "LCT Act s 25-1(3): the luxury car tax threshold for the financial year of the supply or importation, as the ATO publishes it (QC38161)";
const EFFICIENT_NEW: &str = "LCT Act s 25-1(4) and (5): a car with a combined fuel consumption rating of at most 3.5 litres per 100 km takes the fuel-efficient car limit for the year, as the ATO publishes it (QC38161); Act No. 29 of 2025 Sch 1 items 2 and 4(1) for supplies and importations from 1 July 2025";
const EFFICIENT_OLD: &str = "LCT Act s 25-1(4) and (5) as in force before 1 July 2025: a car with a fuel consumption of at most 7 litres per 100 km takes the fuel-efficient car limit for the year, as the ATO publishes it (QC38161); from 1 July 2025 only where, before that day, the car was supplied or imported and used in Australia other than for a quotable purpose (Act No. 29 of 2025 Sch 1 item 4(2))";
const PREVIOUS: &str = "LCT Act s 5-15(2) and (3): less all luxury car tax payable on a previous importation or supply of the car (taking account of adjustments other than bad debts), not below nil";
const IMPORT: &str = "LCT Act ss 7-15 and 13-20: luxury car tax on a taxable importation is paid with customs duty to the Department of Home Affairs, not at label 1E (the ATO's QC22108)";
const PRICE_ADJUSTMENT: &str = "LCT Act ss 15-20 and 15-25: the corrected luxury car tax amount on the supply as it now stands, against the amount previously attributed: an increasing adjustment if more, a decreasing one if less";
const USE_UP: &str = "LCT Act s 15-30(3) and (4): a car supplied under quote (s 15-30(3)(b)(i)), or one on whose supply a s 15-30(1) decreasing adjustment arose (s 15-30(3)(b)(ii)), then used other than for a quotable purpose: an increasing adjustment of the luxury car tax payable, or that would have been payable, on the supply (s 5-15, less earlier tax on the car), which for s 15-30(3)(b)(ii) is the decreasing adjustment (s 15-30(2) and (4)(b)) when the supply's value and year are given";
const USE_DOWN: &str = "LCT Act s 15-30(1) and (2): a registered recipient who did not quote but intends to use, and has only used, the car for a quotable purpose: a decreasing adjustment of the luxury car tax that was payable";
const REFUND_PP: &str = "LCT Act s 18-5: a registered primary producer carrying on a primary production business at the time of the supply or importation, who bore luxury car tax on a refund-eligible car (a 4 wheel drive or all wheel drive car of the kind the regulations specify), is refunded the lesser of that tax and $10,000 (s 18-5(3) as substituted by Act No. 94 of 2019 for cars supplied or imported from 1 July 2019), for one car a financial year (s 18-5(4)), by the approved form, not on the BAS (QC39467)";
const REFUND_TO: &str = "LCT Act s 18-10: a registered tourism operator who bore luxury car tax on a refund-eligible car, which the Commissioner is satisfied it will use solely in a business whose principal purpose is carrying tourists for tourist activities, is refunded the lesser of that tax and $10,000 for each car (s 18-10(3) as substituted by Act No. 94 of 2019 for cars supplied or imported from 1 July 2019), by the approved form, not on the BAS (QC39467)";
const REFUND_YEAR: &str = "LCT Act s 18-5(4): a primary producer has a refund entitlement for at most one car a financial year; this command sees only this tax period's lines, so a refund for another car in an earlier period of the year means none is due here";
const LABELS: &str = "The ATO's 'Luxury car tax (LCT)' BAS labels (QC33694): the luxury car tax on sales and increasing adjustments at 1E, decreasing adjustments at 1F; 'Instructions for completing your BAS' (QC17458): round down to whole dollars";

/// The facts as the command passes them; see the README for each.
pub struct LctArgs<'a> {
    pub lines: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn cents(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(-2, Rounding::HalfUp)?)
}

fn shown(value: &Decimal) -> Result<String> {
    Ok(cents(value)?.to_string())
}

/// `value`, or nil to the cent where it is below nil.
fn at_least_nil(value: Decimal) -> Result<Decimal> {
    Ok(if value.lt(&int(0))? { d("0.00") } else { value })
}

/// The car's threshold, which of the 2 it is and its authority.
fn threshold(car: &Car) -> Result<(Decimal, &'static str, &'static str)> {
    let (efficient, general) = thresholds::of(car.year);
    let new_rule = car.year >= "2025-26" && !car.used_before;
    let limit = d(if new_rule { "3.5" } else { "7" });
    let is_efficient = match &car.consumption {
        Some(rating) => !rating.gt(&limit)?,
        None => false,
    };
    Ok(match (is_efficient, new_rule) {
        (true, true) => (d(efficient), "fuel_efficient", EFFICIENT_NEW),
        (true, false) => (d(efficient), "fuel_efficient", EFFICIENT_OLD),
        (false, _) => (d(general), "general", GENERAL),
    })
}

/// The s 5-15(1) amount on `car`, with its working added to `out`.
fn tax_on(car: &Car, out: &mut Value) -> Result<Decimal> {
    let (limit, basis, authority) = threshold(car)?;
    let excess = at_least_nil(car.value.sub(&limit)?)?;
    let tax = cents(&excess.mul(&int(10))?.mul(&d(LCT_RATE))?.div(&int(11))?)?;
    out["financial_year"] = json!(car.year);
    out["luxury_car_tax_value"] = json!(car.value.to_string());
    if let Some(rating) = &car.consumption {
        out["fuel_consumption_litres_per_100km"] = json!(rating.to_string());
    }
    out["threshold"] = json!(limit.to_string());
    out["threshold_kind"] = json!(basis);
    out["threshold_authority"] = json!(authority);
    out["value_over_threshold"] = json!(shown(&excess)?);
    out["luxury_car_tax"] = json!(tax.to_string());
    out["luxury_car_tax_authority"] = json!(FORMULA);
    Ok(tax)
}

/// The totals a line adds to: sales, importations, increasing and
/// decreasing adjustments and refunds.
#[derive(Default)]
struct Totals {
    sales: Option<Decimal>,
    imports: Option<Decimal>,
    up: Option<Decimal>,
    down: Option<Decimal>,
    refunds: Option<Decimal>,
}

fn add(slot: &mut Option<Decimal>, amount: &Decimal) -> Result<()> {
    *slot = Some(slot.take().unwrap_or_else(|| int(0)).add(amount)?);
    Ok(())
}

/// A sale's or importation's tax payable, less earlier tax on the car for a
/// sale.
fn work_supply(line: &Line, car: &Car, out: &mut Value, totals: &mut Totals) -> Result<()> {
    let tax = tax_on(car, out)?;
    if line.kind == "importation" {
        out["payable_to"] = json!("home_affairs");
        out["payable_authority"] = json!(IMPORT);
        return add(&mut totals.imports, &tax);
    }
    let payable = at_least_nil(tax.sub(&line.previous)?)?;
    if !line.previous.is_zero() {
        out["lct_previously_payable"] = json!(line.previous.to_string());
        out["previous_authority"] = json!(PREVIOUS);
    }
    out["luxury_car_tax_payable"] = json!(payable.to_string());
    out["price_including_lct"] = json!(car.value.add(&payable)?.to_string());
    add(&mut totals.sales, &payable)
}

/// A refund: the lesser of the tax borne and $10,000 on a refund-eligible car.
fn work_refund(line: &Line, out: &mut Value, totals: &mut Totals) -> Result<()> {
    let (claimant, eligible) = line.refund.expect("a refund line's facts");
    let cap = int(REFUND_CAP);
    let amount = match eligible {
        true if line.previous.gt(&cap)? => cap,
        true => line.previous.clone(),
        false => int(0),
    };
    out["claimant"] = json!(claimant);
    out["refund_eligible_car"] = json!(eligible);
    out["lct_borne"] = json!(line.previous.to_string());
    out["refund"] = json!(shown(&amount)?);
    out["refund_authority"] = json!(if claimant == "primary_producer" {
        REFUND_PP
    } else {
        REFUND_TO
    });
    add(&mut totals.refunds, &amount)
}

fn work_line(line: &Line, n: usize, totals: &mut Totals) -> Result<Value> {
    let mut out = json!({"line": n, "kind": line.kind});
    match (line.kind, &line.car) {
        ("sale" | "importation", Some(car)) => work_supply(line, car, &mut out, totals)?,
        ("price_adjustment", Some(car)) => {
            let corrected = at_least_nil(tax_on(car, &mut out)?.sub(&line.previous)?)?;
            let change = corrected.sub(&line.attributed)?;
            out["corrected_luxury_car_tax"] = json!(corrected.to_string());
            out["previously_attributed_lct"] = json!(line.attributed.to_string());
            out["adjustment_authority"] = json!(PRICE_ADJUSTMENT);
            if change.lt(&int(0))? {
                out["decreasing_adjustment"] = json!(shown(&change.neg()?)?);
                add(&mut totals.down, &change.neg()?)?;
            } else {
                out["increasing_adjustment"] = json!(shown(&change)?);
                add(&mut totals.up, &change)?;
            }
        }
        ("change_of_use_increasing", Some(car)) => {
            let tax = at_least_nil(tax_on(car, &mut out)?.sub(&line.previous)?)?;
            out["increasing_adjustment"] = json!(tax.to_string());
            out["adjustment_authority"] = json!(USE_UP);
            add(&mut totals.up, &tax)?;
        }
        ("refund", _) => work_refund(line, &mut out, totals)?,
        _ => {
            out["decreasing_adjustment"] = json!(line.previous.to_string());
            out["adjustment_authority"] = json!(USE_DOWN);
            add(&mut totals.down, &line.previous)?;
        }
    }
    Ok(out)
}

fn read_lines(given: &Arg, year: &'static str) -> Result<Vec<Line>> {
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            "luxury_car_tax.lines_invalid",
            "lines must be a list of line objects.",
        );
    };
    if items.is_empty() || items.len() > MAX_LINES {
        return refuse(
            "luxury_car_tax.lines_invalid",
            format!("lines must hold from 1 to {MAX_LINES} lines."),
        );
    }
    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, v)| parse_line(v, i + 1, year))
        .collect::<Result<_>>()?;
    let primary = lines
        .iter()
        .filter(|l| matches!(l.refund, Some(("primary_producer", true))))
        .count();
    if primary > 1 {
        return refuse(
            "luxury_car_tax.refund_limit",
            "A primary producer's refund entitlement is for one car a financial year (LCT Act s 18-5(4)); give at most one line with a primary_producer refund.",
        );
    }
    Ok(lines)
}

pub fn luxury_car_tax(args: &LctArgs) -> Result<Value> {
    let given = scope(args.scope_confirmed, args.year, KIND)?;
    let year = thresholds::year(&given).expect("a supported year");
    let lines = read_lines(args.lines, year)?;
    let mut totals = Totals::default();
    let worked = lines
        .iter()
        .enumerate()
        .map(|(i, line)| work_line(line, i + 1, &mut totals))
        .collect::<Result<Vec<_>>>()?;
    let zero = int(0);
    let value = |slot: &Option<Decimal>| slot.clone().unwrap_or_else(|| zero.clone());
    let (sales, up, down) = (value(&totals.sales), value(&totals.up), value(&totals.down));
    let label_1e = sales.add(&up)?.quantize(0, Rounding::Down)?;
    let (efficient, general) = thresholds::of(year);
    let out = result(
        KIND,
        year,
        vec![
            ("lct_on_sales", sales),
            ("increasing_adjustments", up),
            ("decreasing_adjustments", down.clone()),
            ("label_1e", label_1e),
            ("label_1f", down.quantize(0, Rounding::Down)?),
            ("lct_on_importations", value(&totals.imports)),
            ("refunds", value(&totals.refunds)),
        ],
        vec![
            rate("lct_rate", LCT_RATE),
            rate("luxury_car_tax_threshold", general),
            rate("fuel_efficient_car_limit", efficient),
        ],
    );
    let mut out = out?;
    out["lines"] = Value::Array(worked);
    out["labels_authority"] = json!(LABELS);
    if lines
        .iter()
        .any(|l| matches!(l.refund, Some(("primary_producer", true))))
    {
        let warnings = out["warnings"].as_array_mut().expect("warnings");
        warnings.push(json!(REFUND_YEAR));
    }
    Ok(out)
}
