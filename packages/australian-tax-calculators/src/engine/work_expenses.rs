// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `work_expenses`: an employee's working from home, car and self-education
//! deductions for an income year from 2020-21 to 2026-27, one method a call.
//! The car rules are ITAA 1997 Division 28 (ss 28-12, 28-25, 28-90, 28-115
//! and 28-120) and the Commissioner's cents per kilometre instruments under
//! s 28-25(4); the working from home rates are PCG 2023/1 Table 1 from
//! 2022-23 and, before it, the ATO's 52 cent fixed rate and 80 cent shortcut
//! (PCG 2020/3); the self-education reduction is ITAA 1936 s 82A, repealed
//! for 2022-23 onwards by Act No. 84, 2022, Schedule 3 items 26 and 33. All
//! were read on 22 November 2026. Hours, kilometres and each expense's
//! work-related amount are established facts: the worksheet applies the
//! rates, cap, coverage and reduction to them.

use serde_json::Value;

use super::fbt_benefits::named;
use super::{
    Arg, Result, Step, d, derived_result, flag, int, money, precise, py_max, py_min, rate, refuse,
    scope, step,
};
use crate::decimal::{Decimal, Rounding};

mod car;
mod home_office;
mod journeys;
mod rows;
mod self_education;

const KIND: &str = "work_expenses";
const COMMAND: &str = "work-expenses";

/// The methods `method` accepts, in the catalogue's order.
pub const METHODS: [&str; 4] = [
    "home_office",
    "car_cents_per_kilometre",
    "car_log_book",
    "self_education",
];

/// Each input only some methods or years take, and whether it is a decimal.
pub const WORK_EXPENSES_OPTIONAL: [(&str, bool); 32] = [
    ("hours", true),
    ("energy", true),
    ("phone", true),
    ("internet", true),
    ("stationery", true),
    ("cleaning", true),
    ("furniture_decline", true),
    ("asset_decline", true),
    ("repairs", true),
    ("running_expenses_incurred", false),
    ("dedicated_home_office", false),
    ("owned_or_leased", false),
    ("business_kilometres", true),
    ("total_kilometres", true),
    ("car_expenses", true),
    ("log_book_year", false),
    ("log_book_weeks", false),
    ("log_book_representative", false),
    ("connection", false),
    ("employed_when_incurred", false),
    ("prescribed_course", false),
    ("log_book_car_in_category_a", false),
    ("category_a", true),
    ("category_b", true),
    ("category_c", true),
    ("category_d", true),
    ("category_e", true),
    ("assessable_labour_income", true),
    ("other_work_related_deductions", true),
    ("australian_resident", false),
    ("hours_diary", false),
    ("log_book", false),
];

/// The optional inputs given as JSON lists or objects: the records rows.
pub const WORK_EXPENSES_RECORDS: [&str; 2] = ["hours_diary", "log_book"];

const WHOLE_DOLLARS: &str = "TAA 1953 Sch 1 s 388-85 as PCG 2023/1's footnotes and the ATO's examples apply it: the return amount is the total with cents disregarded, not rounded";

/// The income year's first calendar year.
fn start(year: &str) -> i64 {
    year[..4].parse().expect("income year")
}

fn invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("work_expenses.fact_invalid", message.into())
}

fn conflict<T>(message: impl Into<String>) -> Result<T> {
    refuse("work_expenses.fact_conflict", message.into())
}

fn sum(values: &[&Decimal]) -> Result<Decimal> {
    values.iter().try_fold(int(0), |total, v| Ok(total.add(v)?))
}

/// A method's total and, as `id`, its return amount with cents disregarded.
fn with_return(total: Step, id: &'static str) -> Result<[Step; 2]> {
    let whole = total.value.quantize(0, Rounding::Down)?;
    let formula = format!("floor({})", total.id);
    let back = step(id, WHOLE_DOLLARS, &formula, &[&total.value], whole);
    Ok([total, back])
}

/// The standard deduction's facts (ITAA 1997 s 25-130), all 3 or none, from
/// 2026-27 (Act No. 49, 2026, Schedule 4 item 17(1)).
const STANDARD_FACTS: [&str; 3] = [
    "assessable_labour_income",
    "other_work_related_deductions",
    "australian_resident",
];

const STANDARD_CAP: &str = "1000";
const LIMIT: &str = "ITAA 1997 s 25-130(1) and (2)(a) and (b): the lesser of $1,000 and the year's assessable labour income (s 25-130(4)), for an individual who is an Australian resident at any time in the year and derives assessable labour income";
const NOT_RESIDENT: &str = "ITAA 1997 s 25-130(1)(b): no standard deduction for an individual who is not an Australian resident at any time in the year";
const CLAIMED: &str = "ITAA 1997 s 25-130(2)(c) to (g): this method's return amount, a deduction incurred in gaining or producing assessable labour income (s 8-1, Division 28, s 25-10 or Subdivision 40-B), and the other work-related deductions those paragraphs cover";
const REDUCED: &str = "ITAA 1997 s 25-130(2): the limit reduced, but not below zero, by the covered deductions claimed";
const WITH_STANDARD: &str = "ITAA 1997 ss 8-1 and 25-130: the covered deductions claimed and the standard deduction left beside them";
const CHOICE: &str = "Explanatory memorandum to the Treasury Laws Amendment (Tax Reform No. 1) Bill 2026, paras 4.14 to 4.17: the taxpayer chooses whether to itemise a covered deduction, and an itemised claim needs its records; this is what claiming this method adds to the year's deductions";

/// The standard deduction's facts, once shown to be all given and the year
/// to be one s 25-130 applies to.
struct Standard {
    labour: Decimal,
    other: Decimal,
    resident: bool,
}

/// Inputs by name, as the command passes them.
type Given = Vec<(String, Arg)>;

/// `given` split into the method's inputs and the standard deduction's facts.
fn standard_facts(
    given: &[(String, Arg)],
    when: &str,
    first: i64,
) -> Result<(Given, Option<Standard>)> {
    let (facts, method): (Vec<_>, Vec<_>) = given
        .iter()
        .cloned()
        .partition(|(n, _)| STANDARD_FACTS.contains(&n.as_str()));
    if facts.is_empty() {
        return Ok((method, None));
    }
    if first < 2026 {
        return refuse(
            "work_expenses.standard_deduction_unavailable",
            "The standard deduction for work-related expenses (ITAA 1997 s 25-130) applies to assessments for 2026-27 and later years only (Act No. 49, 2026, Schedule 4 item 17(1)).",
        );
    }
    let get = named(&facts, COMMAND, when, &STANDARD_FACTS)?;
    let standard = Standard {
        labour: money(get("assessable_labour_income"))?,
        other: money(get("other_work_related_deductions"))?,
        resident: flag(get("australian_resident"), "australian_resident")?,
    };
    Ok((method, Some(standard)))
}

/// The standard deduction's limit: the lesser of $1,000 and the assessable
/// labour income for a resident, and nil for anyone else.
fn standard_limit(s: &Standard) -> Result<Step> {
    if !s.resident {
        return Ok(step(
            "standard_deduction_limit",
            NOT_RESIDENT,
            "0",
            &[],
            int(0),
        ));
    }
    let cap = d(STANDARD_CAP);
    let value = py_min(cap.clone(), s.labour.clone())?;
    Ok(step(
        "standard_deduction_limit",
        LIMIT,
        "min(standard_deduction_cap, assessable_labour_income)",
        &[&cap, &s.labour],
        value,
    ))
}

/// The standard deduction beside `claimed`, this method's return amount, and
/// without it.
fn standard_steps(s: &Standard, claimed: &Decimal) -> Result<Vec<Step>> {
    let limit = standard_limit(s)?;
    let lim = limit.value.clone();
    let all = claimed.add(&s.other)?;
    let left = py_max(int(0), lim.sub(&all)?)?;
    let with_it = all.add(&left)?;
    let mut steps = vec![
        limit,
        step(
            "work_related_deductions_claimed",
            CLAIMED,
            "claimed_method_amount + other_work_related_deductions",
            &[claimed, &s.other],
            all.clone(),
        ),
        step(
            "standard_deduction",
            REDUCED,
            "max(0, standard_deduction_limit - work_related_deductions_claimed)",
            &[&lim, &all],
            left.clone(),
        ),
        step(
            "deductions_if_claimed",
            WITH_STANDARD,
            "work_related_deductions_claimed + standard_deduction",
            &[&all, &left],
            with_it.clone(),
        ),
    ];
    steps.extend(without_claim(&lim, &s.other, &with_it)?);
    Ok(steps)
}

/// The standard deduction and the deductions without this method's claim,
/// and what the claim adds to `with_it`, the deductions with it.
fn without_claim(lim: &Decimal, other: &Decimal, with_it: &Decimal) -> Result<[Step; 3]> {
    let left_without = py_max(int(0), lim.sub(other)?)?;
    let without = other.add(&left_without)?;
    let adds = with_it.sub(&without)?;
    Ok([
        step(
            "standard_deduction_if_not_claimed",
            REDUCED,
            "max(0, standard_deduction_limit - other_work_related_deductions)",
            &[lim, other],
            left_without.clone(),
        ),
        step(
            "deductions_if_not_claimed",
            WITH_STANDARD,
            "other_work_related_deductions + standard_deduction_if_not_claimed",
            &[other, &left_without],
            without.clone(),
        ),
        step(
            "claiming_adds",
            CHOICE,
            "deductions_if_claimed - deductions_if_not_claimed",
            &[with_it, &without],
            adds,
        ),
    ])
}

/// A method's result, with the standard deduction steps after its own when
/// the facts are given; `claimed` names the step holding the amount claimed.
fn finish(
    year: &str,
    mut steps: Vec<Step>,
    rates: Vec<(String, String)>,
    standard: Option<&Standard>,
    claimed: &str,
) -> Result<Value> {
    if let Some(s) = standard {
        let amount = steps
            .iter()
            .find(|x| x.id == claimed)
            .expect("the claimed step")
            .value
            .clone();
        steps.extend(standard_steps(s, &amount)?);
    }
    derived_result(KIND, year, steps, rates)
}

/// `work-expenses`.
pub fn work_expenses(
    method: &Arg,
    reimbursed: &Arg,
    allowance_received: &Arg,
    given: &[(String, Arg)],
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let Some(method) = method
        .text()
        .and_then(|m| METHODS.iter().find(|x| **x == m))
    else {
        return refuse(
            "work_expenses.method_invalid",
            format!("method must be {}.", METHODS.join(", ")),
        );
    };
    if flag(reimbursed, "reimbursed")? {
        return refuse(
            "work_expenses.reimbursed",
            "An expense reimbursed or paid for by an employer or anyone else is outside the worksheet: give only expenses the taxpayer incurred and bore (PCG 2023/1 para 50).",
        );
    }
    if flag(allowance_received, "allowance_received")? {
        return refuse(
            "work_expenses.allowance_received",
            "An allowance for these expenses brings in the reasonable allowance substantiation exceptions and occupation-specific guidance, which the worksheet does not apply.",
        );
    }
    let when = format!(" with method {method} in {year}");
    let first = start(&year);
    let (given, standard) = standard_facts(given, &when, first)?;
    let (given, s) = (given.as_slice(), standard.as_ref());
    precise(|| match *method {
        "home_office" => home_office::home_office(given, &when, &year, first, s),
        "car_cents_per_kilometre" => car::cents_per_kilometre(given, &when, &year, first, s),
        "car_log_book" => car::log_book(given, &when, &year, first, s),
        _ => self_education::self_education(given, &when, &year, first, s),
    })
}
