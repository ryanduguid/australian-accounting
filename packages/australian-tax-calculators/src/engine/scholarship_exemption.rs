// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `scholarship_exemption`: whether a scholarship, bursary, educational
//! allowance or educational assistance is exempt under ITAA 1997 s 51-10
//! item 2.1A, subject to the exceptions in s 51-35, for an income year from
//! 2017-18 to 2026-27. Both read the same in compilations 170 (1 July 2017)
//! and 268 (1 October 2026), read on 24 November 2026 with the ATO's
//! 'Scholarship payments' page and its 'Is my scholarship taxable?' tool page
//! (last updated 1 July 2026). Commonwealth payments, which the tool does not
//! deal with, and friendly society scholarship plans are refused.

use serde_json::{Value, json};

use super::{Arg, Result, flag, refuse, result, scope};

const KIND: &str = "scholarship_exemption";

pub struct ScholarshipExemptionArgs<'a> {
    pub payment_type: &'a Arg,
    pub full_time_student: &'a Arg,
    pub employment_condition: &'a Arg,
    pub labour_contract_condition: &'a Arg,
    pub principally_educational: &'a Arg,
    pub commonwealth_payment: &'a Arg,
    pub friendly_society_plan: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// Item 2.1A's payments: a scholarship, bursary, educational allowance or
/// educational assistance.
fn payment_type(value: &Arg) -> Result<&str> {
    match value.text() {
        Some(
            kind @ ("scholarship" | "bursary" | "educational_allowance" | "educational_assistance"),
        ) => Ok(kind),
        _ => refuse(
            "scholarship_exemption.payment_type_invalid",
            "payment_type must be scholarship, bursary, educational_allowance or educational_assistance: the payments ITAA 1997 s 51-10 item 2.1A covers. A prize, award or other payment is not tested here.",
        ),
    }
}

/// s 51-35(a), (b) and (f): Commonwealth payments are outside item 2.1A, and
/// the ATO's tool does not deal with them.
fn no_commonwealth_payment(value: &Arg) -> Result<()> {
    let why = match value.text() {
        Some("none") => return Ok(()),
        Some("secondary_or_isolated_children") => {
            "A Commonwealth payment for secondary education or the education of isolated children is not exempt under item 2.1A (s 51-35(a)); item 2.1B of the s 51-10 table, subject to s 51-40, may exempt it."
        }
        Some("education_or_training_payment") => {
            "A Commonwealth education or training payment, such as Austudy, ABSTUDY or Youth Allowance, is not exempt under item 2.1A (s 51-35(b)); Subdivision 52-E or 52-F may exempt all or part of it."
        }
        Some("education_entry_payment") => {
            "An education entry payment under Part 2.13A of the Social Security Act 1991 is not exempt under item 2.1A (s 51-35(f))."
        }
        _ => {
            return refuse(
                "scholarship_exemption.commonwealth_payment_invalid",
                "commonwealth_payment must be none, secondary_or_isolated_children, education_or_training_payment or education_entry_payment.",
            );
        }
    };
    refuse(
        "scholarship_exemption.commonwealth_payment",
        format!(
            "{why} The ATO's 'Is my scholarship taxable?' tool does not deal with Commonwealth payments, and neither does this worksheet."
        ),
    )
}

/// The 4 conditions, each `(id, provision, met)`.
fn conditions(args: &ScholarshipExemptionArgs) -> Result<[(&'static str, &'static str, bool); 4]> {
    let full_time = flag(args.full_time_student, "full_time_student")?;
    let employment = flag(args.employment_condition, "employment_condition")?;
    let labour = flag(args.labour_contract_condition, "labour_contract_condition")?;
    let educational = flag(args.principally_educational, "principally_educational")?;
    Ok([
        (
            "full_time_student",
            "ITAA 1997 s 51-10 item 2.1A: paid to a full-time student at a school, college or university",
            full_time,
        ),
        (
            "no_employment_condition",
            "ITAA 1997 s 51-35(c): not paid on the condition that the student will (or will if required) become or continue to be the payer's employee",
            !employment,
        ),
        (
            "no_labour_contract_condition",
            "ITAA 1997 s 51-35(d): not paid on the condition that the student will (or will if required) enter into or continue a contract with the payer wholly or principally for the student's labour",
            !labour,
        ),
        (
            "principally_educational",
            "ITAA 1997 s 51-35(e): the scholarship is provided principally for educational purposes",
            educational,
        ),
    ])
}

pub fn scholarship_exemption(args: &ScholarshipExemptionArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let kind = payment_type(args.payment_type)?;
    no_commonwealth_payment(args.commonwealth_payment)?;
    if flag(args.friendly_society_plan, "friendly_society_plan")? {
        return refuse(
            "scholarship_exemption.friendly_society_plan",
            "Education benefits under a friendly society scholarship plan can be taxable (the ATO's 'Is my scholarship taxable?' page); this worksheet does not test them.",
        );
    }
    let tests = conditions(args)?;
    let exempt = tests.iter().all(|(_, _, met)| *met);
    let failed: Vec<&str> = tests
        .iter()
        .filter(|(_, _, met)| !met)
        .map(|(id, _, _)| *id)
        .collect();
    let steps: Vec<Value> = tests
        .iter()
        .map(|(id, provision, met)| json!({"condition": id, "provision": provision, "met": met}))
        .collect();
    let mut out = result(KIND, &year, Vec::new(), Vec::new())?;
    out["payment_type"] = json!(kind);
    out["exempt"] = json!(exempt);
    out["conditions_not_met"] = json!(failed);
    out["conditions"] = json!(steps);
    out["treatment"] = json!(if exempt {
        "Exempt income under ITAA 1997 s 51-10 item 2.1A: tell the provider it is exempt and do not include it in the tax return."
    } else {
        "Not exempt under item 2.1A: include it in the tax return as assessable income and tell the provider, who may need to withhold from periodic payments (the ATO's 'Scholarship payments' page)."
    });
    Ok(out)
}
