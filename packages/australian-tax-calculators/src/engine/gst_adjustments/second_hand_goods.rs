// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst_second_hand_goods`: the notional input tax credit for one acquisition
//! of second-hand goods under GST Act Subdivision 66-A (ss 66-5 and 66-10),
//! with the exclusions of s 195-1 and GSTD 2012/6.

use serde_json::{Value, json};

use super::super::{
    Arg, Result, Step, d, derived_result, flag, money, precise, rate, refuse, scope, step,
};
use super::{condition, failing};
use crate::decimal::Decimal;

const KIND: &str = "gst_second_hand_goods";

/// The consideration at or below which s 66-10(1A) applies.
const LIMIT: &str = "300";

/// The facts as the command passes them; see the README for each.
pub struct SecondHandGoodsArgs<'a> {
    pub consideration: &'a Arg,
    pub acquired_for_sale_or_exchange: &'a Arg,
    pub animals_or_plants: &'a Arg,
    pub supply_to_you: &'a Arg,
    pub imported: &'a Arg,
    pub by_way_of_hire: &'a Arg,
    pub resupply_taxable: &'a Arg,
    pub resupply_gst: Option<&'a Arg>,
    pub divided_for_resupply: &'a Arg,
    pub valuable_metal: &'a Arg,
    pub leased_before_sale: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const FOR_SALE: &str = "GST Act s 66-5(1): the second-hand goods are acquired for sale or exchange (but not for manufacture) in the ordinary course of business";
const GOODS: &str = "GST Act s 195-1 'second-hand goods': the goods are not animals or plants";
const SUPPLY_TO_YOU: &str = "GST Act s 66-5(2)(a): the supply of the goods to you was neither a taxable supply nor GST-free";
const IMPORTED: &str = "GST Act s 66-5(2)(b): you did not import the goods";
const HIRE: &str = "GST Act s 66-5(2)(c): the supply of the goods to you was not by way of hire";
const RESUPPLY: &str =
    "GST Act s 66-5(2)(e): you do not make a supply of the goods that is not a taxable supply";

/// The cases this command does not work.
fn excluded(args: &SecondHandGoodsArgs) -> Result<()> {
    if flag(args.divided_for_resupply, "divided_for_resupply")? {
        return refuse(
            "gst_second_hand_goods.divided_for_resupply",
            "Goods it is reasonable to expect you to divide before supplying them in 2 or more supplies: Subdivision 66-B may apply (s 66-40), and s 66-5(2)(d) then excludes Subdivision 66-A. This command does not work Subdivision 66-B.",
        );
    }
    if flag(args.valuable_metal, "valuable_metal")? {
        return refuse(
            "gst_second_hand_goods.valuable_metal",
            "Goods consisting partly or wholly of gold, silver or platinum: s 195-1 excludes them from second-hand goods to the extent of the valuable metal, unless they are incidental valuable metal goods, which this command does not decide.",
        );
    }
    if flag(args.leased_before_sale, "leased_before_sale")? {
        return refuse(
            "gst_second_hand_goods.leased_before_sale",
            "Goods leased before they are sold: GSTD 2012/6 counts both taxable supplies and attributes the credit to each in part, which this command does not work.",
        );
    }
    Ok(())
}

/// The conditions of ss 66-5 and 195-1, each met or not.
fn conditions(args: &SecondHandGoodsArgs) -> Result<Vec<Value>> {
    let supply = match args.supply_to_you.text() {
        Some(value @ ("neither" | "taxable" | "gst_free")) => value,
        _ => {
            return refuse(
                "gst_second_hand_goods.fact_invalid",
                "supply_to_you must be one of neither, taxable, gst_free.",
            );
        }
    };
    let for_sale = flag(
        args.acquired_for_sale_or_exchange,
        "acquired_for_sale_or_exchange",
    );
    Ok(vec![
        condition("acquired_for_sale_or_exchange", FOR_SALE, for_sale?),
        condition(
            "second_hand_goods",
            GOODS,
            !flag(args.animals_or_plants, "animals_or_plants")?,
        ),
        condition(
            "supply_to_you_not_taxable",
            SUPPLY_TO_YOU,
            supply == "neither",
        ),
        condition("not_imported", IMPORTED, !flag(args.imported, "imported")?),
        condition(
            "not_by_way_of_hire",
            HIRE,
            !flag(args.by_way_of_hire, "by_way_of_hire")?,
        ),
        condition(
            "resupply_taxable",
            RESUPPLY,
            flag(args.resupply_taxable, "resupply_taxable")?,
        ),
    ])
}

/// The s 66-10 credit: 1/11 of the consideration, capped above $300 at the
/// GST on your taxable supply of the goods.
fn credit(consideration: &Decimal, resupply_gst: Option<&Arg>) -> Result<Vec<Step>> {
    let eleventh = precise(|| Ok(consideration.div(&d("11"))?))?;
    let above = consideration.gt(&d(LIMIT))?;
    match (above, resupply_gst) {
        (false, Some(_)) => refuse(
            "input.field_unknown",
            "resupply_gst is an input only when the consideration is more than $300.",
        ),
        (false, None) => Ok(vec![step(
            "input_tax_credit",
            "GST Act s 66-10(1A): consideration of $300 or less",
            "consideration / 11",
            &[consideration],
            eleventh,
        )]),
        (true, None) => refuse(
            "input.missing_field",
            "resupply_gst is required when the consideration is more than $300 (s 66-10(1)(b)).",
        ),
        (true, Some(gst)) => {
            let gst = money(gst)?;
            let capped = if gst.lt(&eleventh)? {
                gst.clone()
            } else {
                eleventh.clone()
            };
            let first = step(
                "one_eleventh_of_consideration",
                "GST Act s 66-10(1)(a): consideration of more than $300",
                "consideration / 11",
                &[consideration],
                eleventh.clone(),
            );
            let second = step(
                "input_tax_credit",
                "GST Act s 66-10(1)(b): no more than the GST on your taxable supply of the goods",
                "min(one_eleventh_of_consideration, resupply_gst)",
                &[&eleventh, &gst],
                capped,
            );
            Ok(vec![first, second])
        }
    }
}

const APPLIES: [&str; 2] = [
    "The acquisition is a creditable acquisition despite the supply to you not being taxable (GST Act s 66-5(1)); keep the record s 66-17 requires: the supplier's name and address, the goods and their quantity, and the date and consideration of the acquisition.",
    "Where the consideration is more than $300, or you choose, the credit is attributable to the tax period in which any consideration for your taxable supply of the goods is received, or an earlier invoice is issued (s 66-15(1)); on a cash basis, to the extent the consideration is received (s 66-15(2)). Otherwise s 29-10 attributes it.",
];
const NOT_APPLIES: &str = "Subdivision 66-A gives no input tax credit for this acquisition; whether another provision does is not decided here.";

/// GST Act Subdivision 66-A: the conditions and the notional credit.
pub fn gst_second_hand_goods(args: &SecondHandGoodsArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    excluded(args)?;
    let consideration = money(args.consideration)?;
    let rows = conditions(args)?;
    let failing = failing(&rows);
    let applies = failing.is_empty();
    let steps = if applies {
        credit(&consideration, args.resupply_gst)?
    } else {
        vec![]
    };
    let rates = vec![rate("consideration_limit", LIMIT)];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["applies"] = json!(applies);
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["consequences"] = if applies {
        json!(APPLIES)
    } else {
        json!([NOT_APPLIES])
    };
    Ok(out)
}
