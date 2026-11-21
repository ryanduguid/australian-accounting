// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `wine_equalisation_tax`: the wine tax on a tax period's assessable dealings
//! in wine, and the producer rebate on them under its yearly cap, with the
//! amounts for BAS labels 1C and 1D, for dealings in the financial years
//! 2023-24 to 2027-28. The citations are to the A New Tax System (Wine
//! Equalisation Tax) Act 1999 at the ATO's legal database, equal on the
//! provisions used to compilation 32 (1 April 2026) on the Federal Register
//! of Legislation; the Treasury Laws Amendment (Supporting Choice in
//! Superannuation and Other Measures) Act 2026 (No. 12, 2026) Schedule 6;
//! WETR 2009/1; WETR 2009/2; and the ATO's pages QC22750, QC22751, QC22756,
//! QC22758, QC22762, QC33703 and QC17458, all read on 19 November 2026.

use serde_json::{Value, json};

mod input;
use input::{CONDITIONS, Dealing, Valuation, parse_dealing};

use super::{Arg, Result, d, flag, int, money, rate, refuse, result, scope};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "wine_equalisation_tax";
const MAX_DEALINGS: usize = 1000;
const WINE_TAX_RATE: &str = "0.29";

const TAX: &str = "WET Act s 5-5(3): the wine tax is the taxable value of the dealing (Division 9) x 29%, rounded half up to the cent";
const WHOLESALE: &str = "WET Act s 5-5, Assessable Dealings Table items AD1a, AD1b and AD11b: a wholesale sale's taxable value is the price (excluding wine tax and GST) for which the wine was sold";
const HALF_RETAIL: &str = "WET Act ss 9-25(2), 9-30 and 9-35: the notional wholesale selling price by the half retail price method is 50% of the price of the retail sale, or for an AOU connected with retail sales 50% of the price for which the wine would normally be sold by retail; the price includes wine tax and GST (WETR 2009/1 para 143), rounded half up to the cent";
const AVERAGE: &str = "WET Act ss 9-25(3) and 9-40: the notional wholesale selling price by the average wholesale price method is the weighted average of the prices (excluding wine tax and GST) of the tax period's wholesale sales of grape wine of the same vintage and grape varieties, here their total price / their total quantity x this dealing's quantity, rounded half up to the cent";
const PURCHASE: &str = "WET Act s 5-5, Assessable Dealings Table items AD3c(a) and AD13c(a): an AOU of wine purchased under quote is valued at the purchase price (excluding GST)";
const QUOTED: &str = "WET Act s 7-10(1): a sale is not taxable if the purchaser quotes for it at or before the time of the sale; the producer rebate is worked out on the price (excluding wine tax and GST) as if it were a wholesale sale (s 19-5(1)(b)(ii) and s 19-15(1)(a))";
const GST_ON_WET: &str = "GST Act ss 9-70 and 9-75; the ATO's 'Wholesale sales' (QC22750): GST is 10% of the WET inclusive price, assuming a taxable supply, rounded half up to the cent";
const GST_IN_RETAIL: &str = "GST Act ss 9-70 and 9-75; WETR 2009/1 paras 145 and 151: the GST in a GST inclusive retail price is 1/11 of it, assuming a taxable supply, rounded half up to the cent";
const REBATE: &str = "WET Act s 19-15(1): the producer rebate is 29% of the price (excluding wine tax and GST) for a wholesale sale, and 29% of the notional wholesale selling price for a retail sale or AOU; rounded half up to the cent";
const CAP: &str = "WET Act s 19-15(2) and (3), as amended by the Treasury Laws Amendment (Supporting Choice in Superannuation and Other Measures) Act 2026 Schedule 6 for assessable dealings from 1 July 2026: the producer rebates for a financial year are at most $400,000 ($350,000 for dealings before 1 July 2026), for a producer or for a group of associated producers (s 19-20)";
const PROGRESSIVE: &str = "WETR 2009/2 para 67 and note 37: the Commissioner accepts producer rebates claimed in the activity statement for the tax period to which the dealing's wine tax is attributed, so rebate_claimed_earlier_in_year reduces the cap left";
const LABELS: &str = "The ATO's 'Reporting and paying WET' (QC22762) and 'Wine equalisation tax (WET)' (QC33703): wine tax payable at 1C and the producer rebate at 1D, never netted off; 'Instructions for completing your BAS' (QC17458): round down to whole dollars";

/// The conditions of WET Act s 19-5(1) for a producer rebate, in order: the
/// operator's 3 statements and the liability (or quote) the dealing shows.
const CONDITION_AUTHORITIES: [&str; 3] = [
    "WET Act s 19-5(1)(a) and s 33-1 'producer': the entity manufactures the wine, or supplies the source product to another entity that manufactures the wine on its behalf",
    "WET Act s 19-5(1)(d) and (3) to (5): the producer owns the source product (the fresh grapes, other fruit or vegetables, apples or pears, honey or rice) for at least 85% of the wine by volume, from immediately before crushing (or initial fermentation) until the wine is placed in its container",
    "WET Act s 19-5(1)(e) and (7): at the dealing the wine is in a container suitable for retail sale of at most 5 litres (51 litres for cider or perry), branded with a trade mark identifying the producer, owned by it or an associated producer, and registered, pending or used since 1 July 2015",
];
const LIABLE: &str = "WET Act s 19-5(1)(b) and (c): the producer is liable to wine tax on the dealing, or would have been had the purchaser not quoted, and the quote did not state an intention to use the wine in manufacture, make a GST-free supply of it or sell it under quote (s 5-50(1)(b))";

/// The facts as the command passes them; see the README for each.
pub struct WetArgs<'a> {
    pub dealings: &'a Arg,
    pub rebate_claimed_earlier_in_year: Option<&'a Arg>,
    pub associated_producer: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn cents(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(-2, Rounding::HalfUp)?)
}

/// 29% of `value`, to the cent.
fn at_rate(value: &Decimal) -> Result<Decimal> {
    cents(&value.mul(&d(WINE_TAX_RATE))?)
}

/// The yearly cap on a producer's (or group's) producer rebates: $400,000
/// for dealings from 1 July 2026, $350,000 before.
fn rebate_cap(year: &str) -> Decimal {
    int(if year >= "2026-27" { 400_000 } else { 350_000 })
}

/// A dealing's taxable value, the method's name and its authority.
fn taxable_value(dealing: &Dealing) -> Result<(Decimal, &'static str, &'static str)> {
    Ok(match &dealing.valuation {
        Valuation::Price(price) => match dealing.kind {
            "wholesale_sale" => (price.clone(), "price", WHOLESALE),
            "quoted_sale" => (price.clone(), "price", QUOTED),
            _ => (price.clone(), "purchase_price", PURCHASE),
        },
        Valuation::HalfRetail(price) => (
            cents(&price.mul(&d("0.5"))?)?,
            "half_retail_price",
            HALF_RETAIL,
        ),
        Valuation::AverageWholesale {
            quantity,
            sold,
            price,
        } => (
            cents(&price.div(sold)?.mul(quantity)?)?,
            "average_wholesale_price",
            AVERAGE,
        ),
    })
}

/// The GST on a sale and the price including WET and GST, where the dealing
/// is a sale.
fn gst(
    dealing: &Dealing,
    value: &Decimal,
    tax: &Decimal,
) -> Result<Option<(Decimal, Decimal, &'static str)>> {
    Ok(match (dealing.kind, &dealing.retail_price) {
        ("wholesale_sale" | "quoted_sale", _) => {
            let before = value.add(tax)?;
            let gst = cents(&before.mul(&d("0.1"))?)?;
            Some((gst.clone(), before.add(&gst)?, GST_ON_WET))
        }
        ("retail_sale", Some(price)) => {
            Some((cents(&price.div(&int(11))?)?, price.clone(), GST_IN_RETAIL))
        }
        _ => None,
    })
}

/// The rebate conditions as rows, and the rebate the dealing earns.
fn rebate(dealing: &Dealing, met: [bool; 3], value: &Decimal) -> Result<(Value, Decimal)> {
    let liable = dealing.intention.is_none_or(|i| i == "taxable_dealing");
    let mut rows: Vec<Value> = CONDITIONS
        .iter()
        .zip(CONDITION_AUTHORITIES)
        .zip(met)
        .map(
            |((key, authority), met)| json!({"condition": key, "met": met, "authority": authority}),
        )
        .collect();
    rows.insert(
        1,
        json!({"condition": "liable_to_wine_tax_or_quoted_for_a_taxable_dealing", "met": liable, "authority": LIABLE}),
    );
    let all = liable && met.iter().all(|m| *m);
    let amount = if all { at_rate(value)? } else { int(0) };
    let out = json!({
        "conditions": rows,
        "all_met": all,
        "rebate": cents(&amount)?.to_string(),
        "authority": REBATE,
    });
    Ok((out, amount))
}

/// Dealing `n`'s working, its wine tax and the rebate it earns.
fn work_dealing(dealing: &Dealing, n: usize) -> Result<(Value, Decimal, Decimal)> {
    let (value, method, authority) = taxable_value(dealing)?;
    let tax = if dealing.kind == "quoted_sale" {
        int(0)
    } else {
        at_rate(&value)?
    };
    let mut out = json!({
        "dealing": n,
        "kind": dealing.kind,
        "taxable_value": value.to_string(),
        "taxable_value_method": method,
        "taxable_value_authority": authority,
        "wine_tax": cents(&tax)?.to_string(),
        "wine_tax_authority": if dealing.kind == "quoted_sale" { QUOTED } else { TAX },
    });
    if let Some(intention) = dealing.intention {
        out["purchaser_intention"] = json!(intention);
    }
    if let Some((amount, total, why)) = gst(dealing, &value, &tax)? {
        out["gst"] = json!(amount.to_string());
        out["price_including_wet_and_gst"] = json!(total.to_string());
        out["gst_authority"] = json!(why);
    }
    let mut earned = int(0);
    if let Some(met) = dealing.conditions {
        let (working, amount) = rebate(dealing, met, &value)?;
        out["producer_rebate"] = working;
        earned = amount;
    }
    Ok((out, tax, earned))
}

fn read_dealings(given: &Arg) -> Result<Vec<Dealing>> {
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            "wine_equalisation_tax.dealings_invalid",
            "dealings must be a list of dealing objects.",
        );
    };
    if items.is_empty() || items.len() > MAX_DEALINGS {
        return refuse(
            "wine_equalisation_tax.dealings_invalid",
            format!("dealings must hold from 1 to {MAX_DEALINGS} dealings."),
        );
    }
    items
        .iter()
        .enumerate()
        .map(|(i, v)| parse_dealing(v, i + 1))
        .collect()
}

/// The rebate inputs: required where a dealing claims a rebate and not
/// inputs otherwise. Returns the rebates claimed earlier in the year and
/// whether the producer is in an associated group.
fn rebate_inputs(args: &WetArgs, claimed: bool) -> Result<Option<(Decimal, bool)>> {
    match (
        claimed,
        args.rebate_claimed_earlier_in_year,
        args.associated_producer,
    ) {
        (true, Some(earlier), Some(group)) => {
            Ok(Some((money(earlier)?, flag(group, "associated_producer")?)))
        }
        (true, earlier, _) => refuse(
            "input.missing_field",
            format!(
                "{} is required with a dealing's rebate_conditions",
                if earlier.is_none() {
                    "rebate_claimed_earlier_in_year"
                } else {
                    "associated_producer"
                }
            ),
        ),
        (false, None, None) => Ok(None),
        (false, ..) => refuse(
            "input.field_unknown",
            "rebate_claimed_earlier_in_year and associated_producer are inputs only with a dealing's rebate_conditions",
        ),
    }
}

pub fn wine_equalisation_tax(args: &WetArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let dealings = read_dealings(args.dealings)?;
    let claimed = dealings.iter().any(|x| x.conditions.is_some());
    let rebate_facts = rebate_inputs(args, claimed)?;
    let (mut worked, mut tax, mut earned) = (Vec::new(), int(0), int(0));
    for (i, dealing) in dealings.iter().enumerate() {
        let (out, t, e) = work_dealing(dealing, i + 1)?;
        tax = tax.add(&t)?;
        earned = earned.add(&e)?;
        worked.push(out);
    }
    let cap = rebate_cap(&year);
    let (earlier, group) = rebate_facts.clone().unwrap_or((int(0), false));
    let left = if earlier.lt(&cap)? {
        cap.sub(&earlier)?
    } else {
        int(0)
    };
    let rebate = if earned.lt(&left)? {
        earned.clone()
    } else {
        left
    };
    let mut out = result(
        KIND,
        &year,
        vec![
            ("wine_tax", tax.clone()),
            ("label_1c", tax.quantize(0, Rounding::Down)?),
            ("producer_rebate_before_cap", earned.clone()),
            ("producer_rebate", rebate.clone()),
            ("rebate_over_cap", earned.sub(&rebate)?),
            ("label_1d", rebate.quantize(0, Rounding::Down)?),
        ],
        vec![
            rate("wine_tax_rate", WINE_TAX_RATE),
            rate("producer_rebate_cap", cap.to_string()),
        ],
    )?;
    out["dealings"] = Value::Array(worked);
    out["producer_rebate_cap_authority"] = json!(CAP);
    out["labels_authority"] = json!(LABELS);
    if rebate_facts.is_some() {
        add_rebate_warnings(&mut out, &earlier, &cap, group)?;
    }
    Ok(out)
}

/// The progressive claim, group cap and excess warnings.
fn add_rebate_warnings(
    out: &mut Value,
    earlier: &Decimal,
    cap: &Decimal,
    group: bool,
) -> Result<()> {
    let warnings = out["warnings"].as_array_mut().expect("warnings");
    warnings.push(json!(PROGRESSIVE));
    if group {
        warnings.push(json!(format!(
            "An associated producer shares the ${} cap with its group (WET Act ss 19-15(3) and 19-20): rebate_claimed_earlier_in_year must be the group's claims for the year, and the group splits what is left.",
            cap.fixed(0, true)
        )));
    }
    if earlier.gt(cap)? {
        warnings.push(json!(format!(
            "The rebates claimed earlier in the year exceed the ${} cap; WET Act s 19-25 makes the excess payable as wine tax at the end of the financial year.",
            cap.fixed(0, true)
        )));
    }
    Ok(())
}
