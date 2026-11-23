// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst_going_concern`: whether a supply of a going concern is GST-free under
//! GST Act s 38-325, condition by condition, on facts the caller states, with
//! GSTR 2002/5. It never infers a fact: each one is a required input.

use serde_json::{Value, json};

use super::super::{Arg, Result, flag, rate, refuse, result, scope};
use super::{condition, failing};

const KIND: &str = "gst_going_concern";

/// The facts as the command passes them; see the README for each.
pub struct GoingConcernArgs<'a> {
    pub for_consideration: &'a Arg,
    pub recipient_registration: &'a Arg,
    pub written_agreement: &'a Arg,
    pub all_things_necessary: &'a Arg,
    pub carried_on_until_supply: &'a Arg,
    pub farm_land: &'a Arg,
    pub part_outside_arrangement: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const CONSIDERATION: &str = "GST Act s 38-325(1)(a): the supply is for consideration";
const REGISTERED: &str =
    "GST Act s 38-325(1)(b): the recipient is registered or required to be registered";
const AGREED: &str = "GST Act s 38-325(1)(c): the supplier and the recipient have agreed in writing that the supply is of a going concern, on or before the day of the supply (GSTR 2002/5 para 182)";
const ALL_THINGS: &str = "GST Act s 38-325(2)(a): under the arrangement the supplier supplies to the recipient all of the things that are necessary for the continued operation of an enterprise";
const CARRIED_ON: &str = "GST Act s 38-325(2)(b): the supplier carries on, or will carry on, the enterprise until the day of the supply (whether or not as part of a larger enterprise)";

/// One of `allowed`, or a refusal naming them.
fn choice<'a>(arg: &'a Arg, key: &str, allowed: &[&str]) -> Result<&'a str> {
    match arg.text() {
        Some(value) if allowed.contains(&value) => Ok(value),
        _ => refuse(
            "gst_going_concern.fact_invalid",
            format!("{key} must be one of {}.", allowed.join(", ")),
        ),
    }
}

/// The supplies outside the s 38-325 test this command works.
fn excluded(args: &GoingConcernArgs) -> Result<()> {
    if flag(args.farm_land, "farm_land")? {
        return refuse(
            "gst_going_concern.farm_land",
            "A supply of farm land for farming: Subdivision 38-O has its own conditions, which this command does not test.",
        );
    }
    if flag(args.part_outside_arrangement, "part_outside_arrangement")? {
        return refuse(
            "gst_going_concern.part_outside_arrangement",
            "Part of the supply is not under the arrangement: that part is not GST-free under s 38-325, and s 9-80 apportions the supply (GSTR 2002/5 paras 168 and 169), which this command does not work.",
        );
    }
    Ok(())
}

/// The 5 conditions of s 38-325, each met or not on the stated facts.
fn conditions(args: &GoingConcernArgs) -> Result<Vec<Value>> {
    let registration = choice(
        args.recipient_registration,
        "recipient_registration",
        &["registered", "required_to_be_registered", "neither"],
    )?;
    let agreement = choice(
        args.written_agreement,
        "written_agreement",
        &["on_or_before_supply_day", "after_supply_day", "none"],
    )?;
    Ok(vec![
        condition(
            "for_consideration",
            CONSIDERATION,
            flag(args.for_consideration, "for_consideration")?,
        ),
        condition(
            "recipient_registered",
            REGISTERED,
            registration != "neither",
        ),
        condition(
            "agreed_in_writing",
            AGREED,
            agreement == "on_or_before_supply_day",
        ),
        condition(
            "all_things_necessary",
            ALL_THINGS,
            flag(args.all_things_necessary, "all_things_necessary")?,
        ),
        condition(
            "carried_on_until_supply",
            CARRIED_ON,
            flag(args.carried_on_until_supply, "carried_on_until_supply")?,
        ),
    ])
}

const GST_FREE: [&str; 2] = [
    "The supply of the going concern is GST-free (GST Act s 38-325(1)).",
    "A recipient that intends some supplies made through the enterprise to be neither taxable nor GST-free has an increasing adjustment under s 135-5, and Division 129 applies to later changes in that proportion under s 135-10 (GSTR 2002/5 paras 13 and 14).",
];
const NOT_GST_FREE: &str = "The supply is not GST-free under s 38-325; whether it, or each part of it, is taxable, input taxed or GST-free under another provision is not decided here.";

/// GST Act s 38-325: the conditions and whether the supply is GST-free.
pub fn gst_going_concern(args: &GoingConcernArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    excluded(args)?;
    let rows = conditions(args)?;
    let failing = failing(&rows);
    let gst_free = failing.is_empty();
    let mut out = result(KIND, &year, vec![], vec![rate("gst_rate", "0.10")])?;
    out["gst_free"] = json!(gst_free);
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["consequences"] = if gst_free {
        json!(GST_FREE)
    } else {
        json!([NOT_GST_FREE])
    };
    Ok(out)
}
