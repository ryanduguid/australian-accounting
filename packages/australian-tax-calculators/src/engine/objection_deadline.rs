// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `objection-deadline`: the last day to lodge a taxation objection under
//! s 14ZW of the Taxation Administration Act 1953 (compilation no. 227 of
//! 1 October 2026), as the site's objection deadline page works it
//! (ryanduguid.github.io `assets/business-calculators.mjs`, checked
//! 26 November 2026), for an income tax or FBT assessment, an amended one, or
//! a decision for which s 14ZW sets no other period. Which item of the table
//! in ITAA 1936 s 170(1) applies (compilation no. 193 of 1 October 2026) is the
//! operator's stated fact. Counting follows the Acts Interpretation Act 1901
//! (compilation no. 39 of 28 March 2026): s 36(1) item 6 leaves out the day
//! notice is given, a period of years ends as a period of 12 months a year does
//! under s 2G(2) (the Act does not define a year), and s 36(2) moves a last day
//! on a Saturday, Sunday or stated holiday to the next day that is none of
//! these. All read on 26 November 2026.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::{Arg, Result, date_arg, flag, refuse, result, scope};
use crate::dates;

const KIND: &str = "objection_deadline";
const SIXTY_DAYS: i64 = 60;
const MOST_HOLIDAYS: usize = 100;

/// The facts as the command passes them; see the README for each.
pub struct ObjectionDeadlineArgs<'a> {
    pub decision: &'a Arg,
    pub notice_date: &'a Arg,
    pub s170_item: Option<&'a Arg>,
    pub original_notice_date: Option<&'a Arg>,
    pub amended_before: Option<&'a Arg>,
    pub holidays: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Decision {
    Assessment,
    AmendedAssessment,
    Fbt,
    AmendedFbt,
    Other,
}

fn fact_invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("objection_deadline.fact_invalid", message)
}

fn decision(value: &Arg) -> Result<Decision> {
    match value {
        Arg::Text(t) if t == "assessment" => Ok(Decision::Assessment),
        Arg::Text(t) if t == "amended_assessment" => Ok(Decision::AmendedAssessment),
        Arg::Text(t) if t == "fbt_assessment" => Ok(Decision::Fbt),
        Arg::Text(t) if t == "amended_fbt_assessment" => Ok(Decision::AmendedFbt),
        Arg::Text(t) if t == "other" => Ok(Decision::Other),
        _ => fact_invalid(
            "decision must be one of assessment, amended_assessment, fbt_assessment, amended_fbt_assessment, other.",
        ),
    }
}

/// The years the period runs: 2 where item 1, 2, 3 or 3A of the s 170(1) table
/// applies to an income tax assessment, else 4 (s 14ZW(1)(aa)); 4 for FBT
/// (s 14ZW(1)(aaa)); none for any other decision.
fn years(kind: Decision, item: Option<&Arg>) -> Result<Option<i64>> {
    let income_tax = matches!(kind, Decision::Assessment | Decision::AmendedAssessment);
    match (income_tax, item) {
        (true, Some(Arg::Text(t))) if ["1", "2", "3", "3A"].contains(&t.as_str()) => Ok(Some(2)),
        (true, Some(Arg::Text(t))) if t == "none" => Ok(Some(4)),
        (true, _) => fact_invalid(
            "s170_item must be 1, 2, 3, 3A or none for an income tax assessment: the item of the ITAA 1936 s 170(1) table that applies to it.",
        ),
        (false, Some(_)) => fact_invalid("s170_item applies only to an income tax assessment."),
        (false, None) if kind == Decision::Other => Ok(None),
        (false, None) => Ok(Some(4)),
    }
}

/// The last day of `years` years after `notice`: from the next day (AIA
/// s 36(1) item 6) to the day before the same day 12 x `years` months on, or
/// the end of that month where it has no such day (AIA s 2G(2)).
fn years_after(notice: i64, years: i64) -> i64 {
    dates::months_end(notice + 1, 12 * years)
}

/// AIA s 36(2): the next day that is not a Saturday, Sunday or stated holiday.
fn lodgeable(mut day: i64, holidays: &BTreeSet<i64>) -> i64 {
    while (day + 3).rem_euclid(7) >= 5 || holidays.contains(&day) {
        day += 1;
    }
    day
}

fn holidays(given: Option<&Arg>) -> Result<BTreeSet<i64>> {
    let Some(given) = given else {
        return Ok(BTreeSet::new());
    };
    let read: Option<BTreeSet<i64>> = match given {
        Arg::Json(Value::Array(items)) if items.len() <= MOST_HOLIDAYS => items
            .iter()
            .map(|v| v.as_str().and_then(dates::parse))
            .collect(),
        _ => None,
    };
    read.map_or_else(
        || {
            fact_invalid(format!(
                "holidays must be a list of at most {MOST_HOLIDAYS} YYYY-MM-DD dates."
            ))
        },
        Ok,
    )
}

/// The notice of the assessment amended, required and checked for an amended
/// assessment and refused otherwise.
fn original(a: &ObjectionDeadlineArgs, amended: bool, notice: i64) -> Result<Option<(i64, bool)>> {
    match (amended, a.original_notice_date, a.amended_before) {
        (true, Some(first), Some(before)) => {
            let first = date_arg(first, "original_notice_date")?;
            if first > notice {
                return fact_invalid(
                    "original_notice_date cannot be later than notice_date, the amended assessment's notice.",
                );
            }
            Ok(Some((first, flag(before, "amended_before")?)))
        }
        (true, _, _) => {
            fact_invalid("An amended assessment needs original_notice_date and amended_before.")
        }
        (false, None, None) => Ok(None),
        (false, _, _) => fact_invalid(
            "original_notice_date and amended_before apply only to an amended assessment.",
        ),
    }
}

/// The provision and the period's end, with the amended assessment's 2 ends.
fn period(
    kind: Decision,
    years: Option<i64>,
    notice: i64,
    first: Option<(i64, bool)>,
) -> Result<(&'static str, i64, Option<[i64; 2]>)> {
    let Some(n) = years else {
        return Ok(("paragraph 14ZW(1)(c)", notice + SIXTY_DAYS, None));
    };
    let Some((first, before)) = first else {
        let rule = match (kind, n) {
            (Decision::Fbt, _) => "paragraph 14ZW(1)(aaa)",
            (_, 2) => "subparagraph 14ZW(1)(aa)(i)",
            _ => "subparagraph 14ZW(1)(aa)(ii)",
        };
        return Ok((rule, years_after(notice, n), None));
    };
    if n == 2 && before {
        return refuse(
            "objection_deadline.unsettled",
            "The assessment had been amended before: s 14ZW(1C) makes the first assessment's notice the start only for subsection (1B), not (1BA), and this command does not settle which notice (1BA)(c) counts from.",
        );
    }
    let rule = if n == 2 {
        "subsection 14ZW(1BA)"
    } else {
        "subsection 14ZW(1B)"
    };
    let ends = [notice + SIXTY_DAYS, years_after(first, n)];
    Ok((rule, ends[0].max(ends[1]), Some(ends)))
}

/// The last day to lodge the objection.
pub fn objection_deadline(a: &ObjectionDeadlineArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let kind = decision(a.decision)?;
    let notice = date_arg(a.notice_date, "notice_date")?;
    let n = years(kind, a.s170_item)?;
    let amended = matches!(kind, Decision::AmendedAssessment | Decision::AmendedFbt);
    let first = original(a, amended, notice)?;
    let listed = holidays(a.holidays)?;
    let (rule, ends, both) = period(kind, n, notice, first)?;
    let mut out = result(KIND, &year, vec![], vec![])?;
    out["provision"] = json!(rule);
    out["period_ends"] = json!(dates::format(ends));
    out["last_day"] = json!(dates::format(lodgeable(ends, &listed)));
    if let Some([after_amended, after_original]) = both {
        out["amended_notice_period_ends"] = json!(dates::format(after_amended));
        out["original_notice_period_ends"] = json!(dates::format(after_original));
    }
    out["late_objection"] = json!(
        "After the last day an objection may still be lodged with a written request stating fully and in detail the circumstances and reasons for the delay (TAA 1953 s 14ZW(2) and (3)); the Commissioner decides it under s 14ZX."
    );
    Ok(out)
}
