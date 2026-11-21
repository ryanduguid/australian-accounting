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
pub const WORK_EXPENSES_OPTIONAL: [(&str, bool); 27] = [
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
];

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
    precise(|| match *method {
        "home_office" => home_office::home_office(given, &when, &year, first),
        "car_cents_per_kilometre" => car::cents_per_kilometre(given, &when, &year, first),
        "car_log_book" => car::log_book(given, &when, &year, first),
        _ => self_education::self_education(given, &when, &year, first),
    })
}
