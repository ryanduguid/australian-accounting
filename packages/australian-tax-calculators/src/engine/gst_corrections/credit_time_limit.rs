// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst_credit_time_limit`: whether one input tax credit can be taken into
//! account in a GST return: held tax invoice (GST Act s 29-10(3), with
//! s 29-80(1) and reg 29-80.01's $75), attribution to that return (s 29-10(1)
//! to (5)), the 4 years of s 93-5(1) and the supplier's period of review in
//! s 93-15.

use serde_json::{Value, json};

use super::super::{Arg, Result, d, date_arg, flag, money, refuse, result, scope};
use super::{condition, failing, in_year};
use crate::dates;

const KIND: &str = "gst_credit_time_limit";

/// The facts as the command passes them; see the README for each.
pub struct CreditTimeLimitArgs<'a> {
    pub supply_value: &'a Arg,
    pub return_due: &'a Arg,
    pub tax_invoice_held_from: &'a Arg,
    pub return_given: &'a Arg,
    pub assessment_date: &'a Arg,
    pub earlier_attributable_return_given: &'a Arg,
    pub section_29_10_5_election: &'a Arg,
    pub supplier_review_ended_without_invoice: &'a Arg,
    pub supplier_gst_accounted_for: &'a Arg,
    pub special_attribution: &'a Arg,
    pub section_93_10_exception: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const INVOICE: &str = "GST Act s 29-10(3): a tax invoice is held when the GST return is given, unless the supply's value is $75 or less (s 29-80(1) and GST Regulations reg 29-80.01); without one the credit is not attributable to that period and moves to the first later return given whilst holding it";
const ATTRIBUTION: &str = "GST Act s 29-10(1) to (5): this return is the earliest tax period to which the credit (with a tax invoice where required) is attributable, not preceded by an earlier attributable return or an intervening return given whilst holding the tax invoice, unless a valid s 29-10(5) election moved attribution to it";
const FOUR_YEARS: &str = "GST Act s 93-5(1): the credit is taken into account in an assessment within 4 years after the day the GST return was required for the tax period it would be attributable to, apart from s 29-10(3)";
const SUPPLIER: &str = "GST Act s 93-15: entitlement is not barred after the supplier's period of review ended without you holding a tax invoice, because that period has not ended without one or GST on the related supply was accounted for in full in the supplier's assessment";

#[derive(Clone, Copy, PartialEq, Eq)]
enum SupplierGst {
    None,
    Partial,
    Full,
}

fn supplier_gst(arg: &Arg) -> Result<SupplierGst> {
    match arg.text() {
        Some("none") => Ok(SupplierGst::None),
        Some("partial") => Ok(SupplierGst::Partial),
        Some("full") => Ok(SupplierGst::Full),
        _ => refuse(
            "gst_credit_time_limit.fact_invalid",
            "supplier_gst_accounted_for must be one of none, partial, full.",
        ),
    }
}

/// The date a tax invoice was first held, or `None` for `not_held`.
fn held_from(arg: &Arg) -> Result<Option<i64>> {
    if arg.text() == Some("not_held") {
        return Ok(None);
    }
    date_arg(arg, "tax_invoice_held_from").map(Some)
}

/// The return's due day, the day it is given and the assessment day, checked.
fn dates_of(args: &CreditTimeLimitArgs, year: &str) -> Result<(i64, i64, i64)> {
    let due = date_arg(args.return_due, "return_due")?;
    let given = date_arg(args.return_given, "return_given")?;
    let assessed = date_arg(args.assessment_date, "assessment_date")?;
    if !in_year(given, year) {
        return refuse(
            "gst_credit_time_limit.return_given_outside_year",
            format!("return_given must fall in {year}."),
        );
    }
    if assessed < given {
        return refuse(
            "gst_credit_time_limit.dates_invalid",
            "assessment_date must not be before return_given.",
        );
    }
    Ok((due, given, assessed))
}

fn check_exclusions(args: &CreditTimeLimitArgs) -> Result<()> {
    if flag(args.special_attribution, "special_attribution")? {
        return refuse(
            "gst_credit_time_limit.special_attribution",
            "A Commissioner's determination under s 29-25, or one under s 29-10(3) that a tax invoice is not required, sets other rules (s 93-10(1)), which this command does not apply.",
        );
    }
    if flag(args.section_93_10_exception, "section_93_10_exception")? {
        return refuse(
            "gst_credit_time_limit.section_93_10_exception",
            "An exception under GST Act s 93-10(4) (amended assessment reversing input-taxed supply treatment) or s 93-10(5) (s 29-70(1B) tax invoice request agreed to after 4 years) preserves entitlement outside the general 4-year rule, which this command does not apply.",
        );
    }
    Ok(())
}

fn credit_conditions(
    args: &CreditTimeLimitArgs,
    needed: bool,
    held: Option<i64>,
    given: i64,
    assessed: i64,
    cutoff: i64,
) -> Result<Vec<Value>> {
    let earlier_return = flag(
        args.earlier_attributable_return_given,
        "earlier_attributable_return_given",
    )?;
    let election_29_10_5 = flag(args.section_29_10_5_election, "section_29_10_5_election")?;
    let review_ended = flag(
        args.supplier_review_ended_without_invoice,
        "supplier_review_ended_without_invoice",
    )?;
    let accounted = supplier_gst(args.supplier_gst_accounted_for)?;
    if review_ended && accounted == SupplierGst::Partial {
        return refuse(
            "gst_credit_time_limit.partial_supplier_gst",
            "Where the supplier's period of review ended without a tax invoice and only part of the GST on the related supply was taken into account in the supplier's assessment, GST Act s 93-15 ceases entitlement only to the extent not taken into account; partial shortfall amounts are not worked by this command.",
        );
    }
    Ok(vec![
        condition(
            "tax_invoice_held",
            INVOICE,
            !needed || held.is_some_and(|day| day <= given),
        ),
        condition(
            "attributable_to_return",
            ATTRIBUTION,
            !earlier_return || election_29_10_5,
        ),
        condition("within_4_years", FOUR_YEARS, assessed <= cutoff),
        condition(
            "supplier_entitlement_not_barred",
            SUPPLIER,
            !review_ended || accounted == SupplierGst::Full,
        ),
    ])
}

/// GST Act ss 29-10(3) and 93-5: whether the credit can be claimed in the
/// return given on `return_given`, and the day the 4 years end.
pub fn gst_credit_time_limit(args: &CreditTimeLimitArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    check_exclusions(args)?;
    let (due, given, assessed) = dates_of(args, &year)?;
    let needed = money(args.supply_value)?.gt(&d("75"))?;
    let held = held_from(args.tax_invoice_held_from)?;
    let cutoff = dates::add_months(due, 48);
    let rows = credit_conditions(args, needed, held, given, assessed, cutoff)?;
    let failing = failing(&rows);
    let mut out = result(KIND, &year, vec![], vec![])?;
    out["claimable"] = json!(failing.is_empty());
    out["tax_invoice_needed"] = json!(needed);
    out["entitlement_ends"] = json!(dates::format(cutoff));
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["note"] = json!(
        "An election under GST Act s 29-10(5) cannot be revoked or amended (s 29-10(6)) and remains subject to Division 93. Where only part of the GST on the related supply was not taken into account in the supplier's assessment after the supplier's period of review ended without a tax invoice, GST Act s 93-15 ceases entitlement only to that extent."
    );
    Ok(out)
}
