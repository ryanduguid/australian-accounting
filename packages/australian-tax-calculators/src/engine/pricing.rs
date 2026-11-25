// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `hourly-rate` and `fixed-fee`: pricing as the site's hourly and fixed fee
//! pages work it (tools/business-calculators/hourly and fixed-fee, read on
//! 26 November 2026). A required rate and a fee round up, so the hours cover
//! the costs and the fee keeps its margin. GST on a fee is 10% of its value
//! (GST Act s 9-70, read on the ATO's legal database on 26 November 2026).

use serde_json::Value;

use super::{
    Arg, Result, Step, d, derived_result, flag, money, precise, rate, refuse, scope, step,
};
use crate::decimal::{Decimal, Rounding};

const HOURLY: &str = "hourly_rate";
const FEE: &str = "fixed_fee";
const QLD: &str = "Business Queensland, 'Pricing products and services': hourly rate = (costs + profit) / billable hours; the site's hourly page rounds up to the cent";
const FEE_SOURCE: &str =
    "The site's fixed fee page: hours x rate / (1 - margin), rounded up to the step";
const SPLIT: &str = "The site's fixed fee page: each instalment is the total over the count, cut to the cent; the first carries the cents left over";
const GST: &str = "A New Tax System (Goods and Services Tax) Act 1999 s 9-70: GST is 10% of the value of a taxable supply";
const MOST_TOTAL: &str = "1000000000";

pub struct HourlyRateArgs<'a> {
    pub annual_costs: &'a Arg,
    pub target_profit: &'a Arg,
    pub billable_hours: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

pub struct FixedFeeArgs<'a> {
    pub hours: &'a Arg,
    pub rate: &'a Arg,
    pub margin_percent: &'a Arg,
    pub round_to: &'a Arg,
    pub add_gst: &'a Arg,
    pub instalments: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// `value` rounded up to a whole multiple of `unit`.
pub(super) fn up_to(value: &Decimal, unit: &Decimal) -> Result<Decimal> {
    let units = value.div(unit)?;
    let mut whole = units.quantize(0, Rounding::Down)?;
    if whole.lt(&units)? {
        whole = whole.add(&d("1"))?;
    }
    Ok(whole.mul(unit)?)
}

/// Hours from 0.01 to 8,784 (a leap year's), at most 2 decimal places.
fn hours(given: &Arg, name: &str) -> Result<Decimal> {
    match given {
        Arg::Decimal(h)
            if h.is_finite()
                && h.exponent() >= -2
                && !h.lt(&d("0.01"))?
                && !h.gt(&d("8784"))? =>
        {
            Ok(h.clone())
        }
        _ => refuse(
            "pricing.hours_invalid",
            format!("{name} must be hours from 0.01 to 8784, at most 2 decimal places."),
        ),
    }
}

/// The hourly rate before GST that covers annual costs and target profit.
pub fn hourly_rate(a: &HourlyRateArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, HOURLY)?;
    let costs = money(a.annual_costs)?;
    let profit = money(a.target_profit)?;
    let time = hours(a.billable_hours, "billable_hours")?;
    let value = precise(|| {
        let exact = costs.add(&profit)?.div(&time)?;
        Ok(up_to(&exact.mul(&d("100"))?, &d("1"))?.div(&d("100"))?)
    })?;
    let steps = vec![step(
        "hourly_rate",
        QLD,
        "dollars_up((annual_costs + target_profit) / billable_hours x 100) / 100",
        &[&costs, &profit, &time],
        value,
    )];
    derived_result(HOURLY, &year, steps, vec![])
}

/// The margin, a share of the fee: 0 to 99.99 per cent, at most 2 decimal places.
fn margin(given: &Arg) -> Result<Decimal> {
    match given {
        Arg::Decimal(m)
            if m.is_finite() && m.exponent() >= -2 && !m.lt(&d("0"))? && !m.gt(&d("99.99"))? =>
        {
            Ok(m.clone())
        }
        _ => refuse(
            "pricing.margin_invalid",
            "margin_percent must be a per cent from 0 to 99.99, at most 2 decimal places: a 100% margin has no fee.",
        ),
    }
}

/// The step a fee rounds up to: a cent, or $10, $50 or $100.
fn round_to(given: &Arg) -> Result<Decimal> {
    match given.text() {
        Some(t @ ("0.01" | "10" | "50" | "100")) => Ok(d(t)),
        _ => refuse(
            "pricing.round_to_invalid",
            "round_to must be \"0.01\", \"10\", \"50\" or \"100\": the step the fee rounds up to.",
        ),
    }
}

fn instalments(given: &Arg) -> Result<i64> {
    match given {
        Arg::Int(n) if (1..=24).contains(n) => Ok(*n),
        _ => refuse(
            "input.count_invalid",
            "instalments must be a whole number from 1 to 24.",
        ),
    }
}

/// The fee and its GST from [hours, rate, margin, step, GST rate].
fn fee_and_gst(given: [&Decimal; 5]) -> Result<[Decimal; 2]> {
    let [time, charge, margin, unit, gst_rate] = given;
    precise(|| {
        let share = d("1").sub(&margin.div(&d("100"))?)?;
        let fee = up_to(&time.mul(charge)?.div(&share)?, unit)?;
        let gst = fee.mul(gst_rate)?.quantize(-2, Rounding::HalfUp)?;
        Ok([fee, gst])
    })
}

/// The fee, GST, total, instalment, first instalment and effective rate from
/// [hours, rate, margin, step, GST rate, count]; refused above $1 billion.
fn fee_figures(given: [&Decimal; 6]) -> Result<[Decimal; 6]> {
    let [time, charge, margin, unit, gst_rate, count] = given;
    let [fee, gst] = fee_and_gst([time, charge, margin, unit, gst_rate])?;
    let total = precise(|| Ok(fee.add(&gst)?))?;
    if total.gt(&d(MOST_TOTAL))? {
        return refuse(
            "pricing.fee_out_of_range",
            "The fee with GST is over 1000000000.00: use smaller amounts.",
        );
    }
    let [each, first, effective] = precise(|| {
        let each = total.div(count)?.quantize(-2, Rounding::Down)?;
        let first = total.sub(&each.mul(&count.sub(&d("1"))?)?)?;
        let effective = fee.div(time)?.quantize(-2, Rounding::HalfUp)?;
        Ok([each, first, effective])
    })?;
    Ok([fee, gst, total, each, first, effective])
}

fn fee_steps(given: [&Decimal; 6], figures: [Decimal; 6]) -> Vec<Step> {
    let [time, charge, margin, unit, gst_rate, count] = given;
    let [fee, gst, total, each, first, effective] = figures;
    vec![
        step(
            "fee",
            FEE_SOURCE,
            "dollars_up(hours x rate / (1 - margin_percent / 100) / round_to) x round_to",
            &[time, charge, margin, unit, unit],
            fee.clone(),
        ),
        step(
            "gst",
            GST,
            "cents(fee x gst_rate)",
            &[&fee, gst_rate],
            gst.clone(),
        ),
        step("total", SPLIT, "fee + gst", &[&fee, &gst], total.clone()),
        step(
            "instalment",
            SPLIT,
            "cents_down(total / instalments)",
            &[&total, count],
            each.clone(),
        ),
        step(
            "first_instalment",
            SPLIT,
            "total - instalment x (instalments - 1)",
            &[&total, &each, count],
            first,
        ),
        step(
            "effective_rate",
            FEE_SOURCE,
            "cents(fee / hours)",
            &[&fee, time],
            effective,
        ),
    ]
}

/// A fixed fee from hours at a rate or a cost and margin, with GST and
/// monthly instalments that add up to the total.
pub fn fixed_fee(a: &FixedFeeArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, FEE)?;
    let time = hours(a.hours, "hours")?;
    let charge = money(a.rate)?;
    let margin = margin(a.margin_percent)?;
    let unit = round_to(a.round_to)?;
    let add_gst = flag(a.add_gst, "add_gst")?;
    let count = Decimal::from_int(instalments(a.instalments)?);
    let gst_rate = d(if add_gst { "0.1" } else { "0" });
    let given = [&time, &charge, &margin, &unit, &gst_rate, &count];
    let steps = fee_steps(given, fee_figures(given)?);
    derived_result(FEE, &year, steps, vec![rate("gst_rate", &gst_rate)])
}
