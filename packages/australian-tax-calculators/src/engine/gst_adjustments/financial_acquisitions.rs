// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst_financial_acquisitions_threshold`: GST Act Division 189, whether an
//! entity exceeds the financial acquisitions threshold on the current
//! (s 189-5) and the future (s 189-10) acquisitions tests, with GSTR 2003/9's
//! examples.

use serde_json::{Value, json};

use super::super::{
    Arg, Result, d, derived_result, flag, money, precise, rate, refuse, scope, step,
};
use super::{condition, failing};
use crate::decimal::Decimal;

const KIND: &str = "gst_financial_acquisitions_threshold";

/// The amount limb, $150,000 (no other amount is specified in the GST
/// Regulations 2019, read on 22 November 2026).
const AMOUNT_LIMIT: &str = "150000";

/// The facts as the command passes them; see the README for each.
pub struct FinancialAcquisitionsArgs<'a> {
    pub current_financial_credits: &'a Arg,
    pub current_total_credits: &'a Arg,
    pub future_financial_credits: &'a Arg,
    pub future_total_credits: &'a Arg,
    pub gst_group: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The provisions for one test, alone or as a GST group member.
fn provisions(future: bool, group: bool) -> (&'static str, &'static str) {
    match (future, group) {
        (false, false) => (
            "GST Act s 189-5(1)(a): the input tax credits for the financial acquisitions of the 12 months ending at the end of the month, treated as solely for a creditable purpose, do not exceed $150,000",
            "GST Act s 189-5(1)(b): those credits are not more than 10% of the input tax credits for all acquisitions and importations of those 12 months, the financial acquisitions included",
        ),
        (false, true) => (
            "GST Act s 189-5(2)(a): the input tax credits for every GST group member's financial acquisitions of the 12 months ending at the end of the month, treated as solely for a creditable purpose, do not exceed $150,000",
            "GST Act s 189-5(2)(b): those credits are not more than 10% of the input tax credits for all the members' acquisitions and importations of those 12 months, the financial acquisitions included",
        ),
        (true, false) => (
            "GST Act s 189-10(1)(a): the input tax credits for the financial acquisitions of the month and the next 11 months, treated as solely for a creditable purpose, do not exceed $150,000",
            "GST Act s 189-10(1)(b): those credits are not more than 10% of the input tax credits for all acquisitions and importations of those months, the financial acquisitions included",
        ),
        (true, true) => (
            "GST Act s 189-10(2)(a): the input tax credits for every GST group member's financial acquisitions of the month and the next 11 months, treated as solely for a creditable purpose, do not exceed $150,000",
            "GST Act s 189-10(2)(b): those credits are not more than 10% of the input tax credits for all the members' acquisitions and importations of those months, the financial acquisitions included",
        ),
    }
}

/// One test's 2 condition rows, each met when its limb is not exceeded, and
/// its 10% step.
fn test(
    name: &'static str,
    financial: &Decimal,
    total: &Decimal,
    future: bool,
    group: bool,
) -> Result<(Vec<Value>, super::super::Step)> {
    if financial.gt(total)? {
        return refuse(
            "gst_financial_acquisitions_threshold.fact_conflict",
            format!(
                "{name}_financial_credits cannot exceed {name}_total_credits, which include them."
            ),
        );
    }
    let (amount_rule, share_rule) = provisions(future, group);
    let tenth = precise(|| Ok(total.mul(&d("0.10"))?))?;
    let (id, formula) = if future {
        ("future_ten_percent", "0.10 x future_total_credits")
    } else {
        ("current_ten_percent", "0.10 x current_total_credits")
    };
    let rows = vec![
        condition(
            &format!("{name}_within_amount"),
            amount_rule,
            !financial.gt(&d(AMOUNT_LIMIT))?,
        ),
        condition(
            &format!("{name}_within_ten_percent"),
            share_rule,
            !financial.gt(&tenth)?,
        ),
    ];
    let share = step(id, share_rule, formula, &[total], tenth);
    Ok((rows, share))
}

const EXCEEDS: &str = "The entity exceeds the financial acquisitions threshold (GSTR 2003/9 para 13: either test is enough): an acquisition is not for a creditable purpose to the extent it relates to making input taxed financial supplies (GST Act s 11-15(2)(a)), apart from reduced credit acquisitions (Division 70), and changes in applying things to financial supplies count for Division 129 (s 129-5(2)).";
const WITHIN: &str = "The entity does not exceed the financial acquisitions threshold: an acquisition relating to making financial supplies is not, for that reason alone, outside a creditable purpose (GST Act s 11-15(4)), and Division 129 disregards changes in applying things to financial supplies (s 129-5(2)).";

/// GST Act Division 189: the current and the future acquisitions tests.
pub fn gst_financial_acquisitions_threshold(args: &FinancialAcquisitionsArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let group = flag(args.gst_group, "gst_group")?;
    let [
        current_financial,
        current_total,
        future_financial,
        future_total,
    ] = [
        args.current_financial_credits,
        args.current_total_credits,
        args.future_financial_credits,
        args.future_total_credits,
    ]
    .map(money);
    let (mut rows, current) = test(
        "current",
        &current_financial?,
        &current_total?,
        false,
        group,
    )?;
    let (future_rows, future) = test("future", &future_financial?, &future_total?, true, group)?;
    rows.extend(future_rows);
    let failing = failing(&rows);
    let exceeds = !failing.is_empty();
    let rates = vec![
        rate("amount_limit", AMOUNT_LIMIT),
        rate("proportion_limit", "0.10"),
    ];
    let mut out = derived_result(KIND, &year, vec![current, future], rates)?;
    out["exceeds_threshold"] = json!(exceeds);
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["consequences"] = json!([if exceeds { EXCEEDS } else { WITHIN }]);
    Ok(out)
}
