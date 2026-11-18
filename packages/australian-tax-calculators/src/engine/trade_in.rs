// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `trade_in`: trading in a depreciating asset for a replacement. The old
//! asset's termination value is the trade-in allowance (ITAA 1997 s 40-305,
//! the liability to pay the replacement's price terminated) less the GST on
//! the trade-in plus any GST Act s 132-5 decreasing adjustment (Subdivision
//! 27-B); its balancing adjustment is `balancing_adjustment`'s. The
//! replacement's cost is its price (s 40-185) less its input tax credit
//! (s 27-80), capped for a car by GST Act s 69-10 and ITAA 1997 s 40-230 at
//! the car limit. The citations are to ITAA 1997 compilation 268 (1 October
//! 2026) and the GST Act compilation 96 (1 January 2026), read on 18 November
//! 2026.

use serde_json::Value;

use super::balancing::{Facts, Given, Worked, read_facts, worked};
use super::business_capital::year_first;
use super::{
    Arg, Result, Step, d, derived_result, finite_between, flag, int, money, precise, py_min, rate,
    refuse, scope, step,
};
use crate::decimal::Decimal;

const KIND: &str = "trade_in";

/// The car limit for the financial years this worksheet takes: $69,674 for
/// 2025-26 (the ATO's Guide to depreciating assets 2026) and $69,883 for
/// 2026-27 (the ATO's 'Purchasing a motor vehicle', last updated 3 June
/// 2026), both read on 18 November 2026.
const CAR_LIMITS: [(&str, &str); 2] = [("2025-26", "69674"), ("2026-27", "69883")];

/// The facts as the command passes them; see the README for each.
pub struct TradeInArgs<'a> {
    pub cost: &'a Arg,
    pub asset: &'a Arg,
    pub years: &'a Arg,
    pub car_limit: Option<&'a Arg>,
    pub trade_in_allowance: &'a Arg,
    pub trade_in_taxable: &'a Arg,
    pub adjusted_input_tax_credit: Option<&'a Arg>,
    pub full_input_tax_credit: Option<&'a Arg>,
    pub replacement_price: &'a Arg,
    pub replacement_asset: &'a Arg,
    pub replacement_luxury_car_tax: Option<&'a Arg>,
    pub replacement_creditable_acquisition: &'a Arg,
    pub creditable_purpose: Option<&'a Arg>,
    pub arms_length: &'a Arg,
    pub involuntary_disposal: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// An optional input that `needed` decides: required when needed, refused
/// when not.
fn when<'a>(needed: bool, given: Option<&'a Arg>, key: &str, why: &str) -> Result<Option<&'a Arg>> {
    match (needed, given) {
        (true, None) => refuse("input.missing_field", format!("{key} is required {why}.")),
        (false, Some(_)) => refuse(
            "input.field_unknown",
            format!("{key} is an input only {why}"),
        ),
        (_, given) => Ok(given),
    }
}

fn conflict<T>(message: &str) -> Result<T> {
    refuse("trade_in.fact_conflict", message)
}

/// The traded-in asset's GST facts: the input tax credit claimed (as
/// adjusted) and the full one, for a taxable trade-in.
fn old_credits(args: &TradeInArgs, taxable: bool) -> Result<Option<[Decimal; 2]>> {
    let why = "when the trade-in is a taxable supply (s 132-5)";
    let adjusted = when(
        taxable,
        args.adjusted_input_tax_credit,
        "adjusted_input_tax_credit",
        why,
    )?;
    let full = when(
        taxable,
        args.full_input_tax_credit,
        "full_input_tax_credit",
        why,
    )?;
    let (Some(adjusted), Some(full)) = (adjusted, full) else {
        return Ok(None);
    };
    let (adjusted, full) = (money(adjusted)?, money(full)?);
    if adjusted.gt(&full)? {
        return conflict(
            "adjusted_input_tax_credit cannot exceed full_input_tax_credit (GST Act s 132-5(2)).",
        );
    }
    Ok(Some([adjusted, full]))
}

/// The replacement: its price, luxury car tax, whether it is a passenger car
/// and the extent of creditable purpose of a creditable acquisition.
struct Replacement {
    price: Decimal,
    luxury_car_tax: Decimal,
    car: bool,
    creditable_purpose: Option<Decimal>,
}

/// The extent of creditable purpose, for a creditable acquisition only.
fn creditable_purpose(args: &TradeInArgs) -> Result<Option<Decimal>> {
    let creditable = flag(
        args.replacement_creditable_acquisition,
        "replacement_creditable_acquisition",
    )?;
    let why = "for a creditable acquisition (GST Act s 11-30)";
    let Some(extent) = when(
        creditable,
        args.creditable_purpose,
        "creditable_purpose",
        why,
    )?
    else {
        return Ok(None);
    };
    match finite_between(extent, "0", "1")? {
        Some(extent) if !extent.is_zero() => Ok(Some(extent)),
        Some(_) => conflict(
            "creditable_purpose is nil: an acquisition made for no creditable purpose is not a creditable acquisition (GST Act s 11-5).",
        ),
        None => refuse(
            "input.fraction_invalid",
            "creditable_purpose must be a finite decimal fraction from 0 to 1.",
        ),
    }
}

fn replacement(args: &TradeInArgs) -> Result<Replacement> {
    let price = money(args.replacement_price)?;
    let car = match args.replacement_asset.text() {
        Some("passenger_car") => true,
        Some("other") => false,
        _ => {
            return refuse(
                "trade_in.asset_invalid",
                "replacement_asset must be passenger_car or other.",
            );
        }
    };
    let key = "replacement_luxury_car_tax";
    let tax = when(
        car,
        args.replacement_luxury_car_tax,
        key,
        "for a passenger car",
    )?;
    let luxury_car_tax = tax.map(money).transpose()?.unwrap_or_else(|| int(0));
    if luxury_car_tax.gt(&price)? {
        return conflict(
            "replacement_luxury_car_tax cannot exceed replacement_price, which includes it.",
        );
    }
    let creditable_purpose = creditable_purpose(args)?;
    Ok(Replacement {
        price,
        luxury_car_tax,
        car,
        creditable_purpose,
    })
}

/// Refuses a trade-in this worksheet does not work: one not at arm's length,
/// or an involuntary disposal whose s 40-365 choice it does not make.
fn refuse_unsupported(args: &TradeInArgs) -> Result<()> {
    if !flag(args.arms_length, "arms_length")? {
        return refuse(
            "trade_in.not_arms_length",
            "A trade-in not at arm's length is not worked: ITAA 1997 s 40-300(2) item 6 makes the termination value the market value where it would otherwise be less, s 27-95(2) then takes no GST off it, the first element of the replacement's cost can be its market value (the ATO's Guide to depreciating assets 2026), and s 40-225 increases a car's cost by any discount portion referable to selling the old asset for less than its market value.",
        );
    }
    if flag(args.involuntary_disposal, "involuntary_disposal")? {
        return refuse(
            "trade_in.involuntary_disposal",
            "An involuntary disposal is not worked: where the old asset was lost, destroyed or compulsorily acquired (ITAA 1997 s 40-365(2)) and the replacement is acquired from one year before the event to one year after the end of its income year and used wholly for a taxable purpose ((3) and (4)), s 40-365 lets you exclude some or all of the assessable balancing adjustment amount and reduce the replacement's cost by it instead ((5) and (6)); this worksheet does not make that choice.",
        );
    }
    Ok(())
}

const GST_SALE: &str = "GST Act ss 9-70 and 9-75(1): GST on the trade-in, a taxable supply, is 10% of its value, the price x 10/11, so 1/11 of the trade-in allowance";
const NOT_TAXABLE: &str = "GST Act s 9-5: the trade-in is not a taxable supply, as the operator established, so no GST is payable and there is no s 132-5 adjustment";
const DECREASING: &str = "GST Act s 132-5(2) and (3): 1/11 x price x (1 - adjusted input tax credit / full input tax credit), capped at the full input tax credit less the adjusted one";
const NO_FULL_CREDIT: &str = "GST Act s 132-5(3): the decreasing adjustment is at most the full input tax credit less the adjusted one, which is nil when the full input tax credit is nil (such as for an asset bought without GST)";
const TV_TRADE_IN: &str = "ITAA 1997 s 40-305(1)(b) item 2, s 27-95(1) and (3): the trade-in allowance (the liability to pay the replacement's price terminated), less the GST payable on the trade-in, plus the decreasing adjustment";

/// The trade-in's GST and decreasing adjustment, with their steps.
fn sale_gst(allowance: &Decimal, credits: Option<&[Decimal; 2]>) -> Result<[Step; 2]> {
    let Some([adjusted, full]) = credits else {
        let nil = |id| step(id, NOT_TAXABLE, "0", &[], int(0));
        return Ok([nil("trade_in_gst"), nil("decreasing_adjustment")]);
    };
    let gst = allowance.div(&int(11))?;
    let gst_step = step(
        "trade_in_gst",
        GST_SALE,
        "trade_in_allowance / 11",
        &[allowance],
        gst.clone(),
    );
    // adjusted <= full, so a nil full credit leaves a nil difference.
    if full.is_zero() {
        let nil = step(
            "decreasing_adjustment",
            NO_FULL_CREDIT,
            "full_input_tax_credit - adjusted_input_tax_credit",
            &[full, adjusted],
            int(0),
        );
        return Ok([gst_step, nil]);
    }
    let share = int(1).sub(&adjusted.div(full)?)?;
    let adjustment = py_min(gst.mul(&share)?, full.sub(adjusted)?)?;
    Ok([
        gst_step,
        step(
            "decreasing_adjustment",
            DECREASING,
            "min(trade_in_gst x (1 - adjusted_input_tax_credit / full_input_tax_credit), full_input_tax_credit - adjusted_input_tax_credit)",
            &[&gst, adjusted, full, full, adjusted],
            adjustment,
        ),
    ])
}

const NO_CREDIT: &str = "GST Act s 11-5: the replacement is not a creditable acquisition, as the operator established, so there is no input tax credit";
const FULL_CREDIT: &str = "GST Act ss 9-70, 9-75(1), 11-25 and 11-30(3): the full input tax credit is the GST payable on the replacement, 1/11 of its price";
const CAR_CREDIT: &str = "GST Act ss 9-75(2), 11-25 and 69-10(1), with the ATO's 'Purchasing a motor vehicle' (Darren's example): the GST payable on a car, 1/11 of its price less luxury car tax, up to 1/11 of the car limit for the financial year it is first used";
const PART_CREDIT: &str = "GST Act ss 11-30(3) and 69-10(3): the full input tax credit x the extent of creditable purpose";
const COST: &str = "ITAA 1997 ss 40-180, 40-185(1)(b) item 2 and 27-80(1): the price the replacement is bought for, the liability incurred, less its input tax credit";
const CAR_COST: &str = "ITAA 1997 s 40-230(1): a car designed mainly for carrying passengers has its cost, after Subdivision 27-B, reduced to the car limit for the financial year it is first held";
const NOT_CAR_COST: &str = "ITAA 1997 s 40-230: the car limit does not apply to an asset that is not a car designed mainly for carrying passengers";

/// The replacement's full input tax credit: the GST payable on it, capped for
/// a passenger car at 1/11 of the car limit.
fn full_credit(new: &Replacement, car_limit: &Decimal) -> Result<Step> {
    let id = "replacement_full_input_tax_credit";
    Ok(match (&new.creditable_purpose, new.car) {
        (None, _) => step(id, NO_CREDIT, "0", &[], int(0)),
        (Some(_), true) => {
            let gst = new.price.sub(&new.luxury_car_tax)?.div(&int(11))?;
            let capped = py_min(gst, car_limit.div(&int(11))?)?;
            let formula =
                "min((replacement_price - replacement_luxury_car_tax) / 11, car_limit / 11)";
            let operands = [&new.price, &new.luxury_car_tax, car_limit];
            step(id, CAR_CREDIT, formula, &operands, capped)
        }
        (Some(_), false) => {
            let gst = new.price.div(&int(11))?;
            step(
                id,
                FULL_CREDIT,
                "replacement_price / 11",
                &[&new.price],
                gst,
            )
        }
    })
}

/// The replacement's cost ignoring the car limit and its cost, after the
/// input tax credit `credit`.
fn cost_steps(new: &Replacement, credit: &Decimal, car_limit: &Decimal) -> Result<[Step; 2]> {
    let ignoring = new.price.sub(credit)?;
    let cost = if new.car {
        let formula = "min(replacement_cost_ignoring_car_limit, car_limit)";
        let value = py_min(ignoring.clone(), car_limit.clone())?;
        step(
            "replacement_cost",
            CAR_COST,
            formula,
            &[&ignoring, car_limit],
            value,
        )
    } else {
        let formula = "replacement_cost_ignoring_car_limit";
        step(
            "replacement_cost",
            NOT_CAR_COST,
            formula,
            &[&ignoring],
            ignoring.clone(),
        )
    };
    let formula = "replacement_price - replacement_input_tax_credit";
    let id = "replacement_cost_ignoring_car_limit";
    let ignoring = step(id, COST, formula, &[&new.price, credit], ignoring);
    Ok([ignoring, cost])
}

/// The replacement's input tax credit and cost, with their steps.
fn replacement_steps(new: &Replacement, car_limit: &Decimal) -> Result<[Step; 4]> {
    let full = full_credit(new, car_limit)?;
    let credit = match &new.creditable_purpose {
        None => step("replacement_input_tax_credit", NO_CREDIT, "0", &[], int(0)),
        Some(extent) => step(
            "replacement_input_tax_credit",
            PART_CREDIT,
            "replacement_full_input_tax_credit x creditable_purpose",
            &[&full.value, extent],
            full.value.mul(extent)?,
        ),
    };
    let [ignoring, cost] = cost_steps(new, &credit.value, car_limit)?;
    Ok([full, credit, ignoring, cost])
}

/// The old asset's balancing adjustment on the termination value the trade-in
/// gives, then the replacement's steps.
fn trade_in_steps(
    facts: &Facts,
    allowance: &Decimal,
    credits: Option<&[Decimal; 2]>,
    new: &Replacement,
    car_limit: &Decimal,
) -> Result<Worked> {
    precise(|| {
        let [gst, adjustment] = sale_gst(allowance, credits)?;
        let value = allowance.sub(&gst.value)?.add(&adjustment.value)?;
        let given = Given {
            authority: TV_TRADE_IN,
            formula: "trade_in_allowance - trade_in_gst + decreasing_adjustment",
            operands: &[allowance, &gst.value, &adjustment.value],
            value,
        };
        let (old, mut rates) = worked(facts, &given)?;
        let mut steps = vec![gst, adjustment];
        steps.extend(old);
        steps.extend(replacement_steps(new, car_limit)?);
        if new.car {
            rates.push(rate("car_limit", car_limit));
        }
        Ok((steps, rates))
    })
}

pub fn trade_in(args: &TradeInArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    refuse_unsupported(args)?;
    let event = year_first(&year).expect("a supported income year");
    let facts = read_facts(args.cost, args.asset, args.years, args.car_limit, event)?;
    let allowance = money(args.trade_in_allowance)?;
    let taxable = flag(args.trade_in_taxable, "trade_in_taxable")?;
    let credits = old_credits(args, taxable)?;
    let new = replacement(args)?;
    let (_, limit) = CAR_LIMITS
        .iter()
        .find(|(y, _)| *y == year)
        .expect("a supported income year");
    let (steps, rates) = trade_in_steps(&facts, &allowance, credits.as_ref(), &new, &d(limit))?;
    derived_result(KIND, &year, steps, rates)
}
