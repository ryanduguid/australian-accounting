// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The employer's kind and the not-for-profit inputs of `fbt-return`,
//! checked or refused before any worksheet runs: FBTAA ss 57A and 65J,
//! compilation 97 (1 July 2026, C2026C00332), read on 22 November 2026.
use super::COMMAND;
use crate::api::Refusal;
use crate::decimal::Decimal;
use serde_json::{Value, json};

/// How the return treats the employer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Taxable,
    Rebatable,
    /// Every benefit is exempt under s 57A; the cap per employee applies.
    Exempt(i64),
}

/// Each `employer_kind`, with the provision that makes it one.
const KINDS: [(&str, Kind, &str); 7] = [
    ("taxable", Kind::Taxable, "FBTAA s 5B(1A)"),
    ("rebatable", Kind::Rebatable, "FBTAA s 65J(1)"),
    (
        "public_benevolent_institution",
        Kind::Exempt(30_000),
        "FBTAA s 57A(1); $30,000 cap, s 5B(1E) step 3",
    ),
    (
        "health_promotion_charity",
        Kind::Exempt(30_000),
        "FBTAA s 57A(5); $30,000 cap, s 5B(1E) step 3",
    ),
    (
        "public_hospital",
        Kind::Exempt(17_000),
        "FBTAA s 57A(3)(a); $17,000 cap, s 5B(1E) step 2(c)",
    ),
    (
        "not_for_profit_hospital",
        Kind::Exempt(17_000),
        "FBTAA s 57A(4); $17,000 cap, s 5B(1E) step 2(d)",
    ),
    (
        "public_ambulance_service",
        Kind::Exempt(17_000),
        "FBTAA s 57A(3)(b); $17,000 cap, s 5B(1E) step 2(ca)",
    ),
];

/// What a benefit is, for s 5B(1L) and (1M) and s 65J(2H) and (2J).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Entertainment {
    Unstated,
    /// Meal entertainment or entertainment facility leasing provided under a
    /// salary packaging arrangement (s 5B(1M), s 65J(2J)).
    SalaryPackaged,
    /// Meal entertainment or entertainment facility leasing not under a
    /// salary packaging arrangement, which the caps disregard.
    NotSalaryPackaged,
}

const NOT_COVERED: &str = "not covered: a government body whose employees work in or for a hospital (s 57A(2)), an organisation partly a public benevolent institution and partly rebatable, live-in residential care workers (s 58), and one employee's benefits from more than one employer";

/// The employer's kind; absent is a taxable employer, as before.
pub(super) fn check_kind(v: &Value) -> Result<(&'static str, Kind), Refusal> {
    let Some(given) = v.get("employer_kind") else {
        return Ok(("taxable", Kind::Taxable));
    };
    match KINDS
        .iter()
        .find(|(name, ..)| Some(*name) == given.as_str())
    {
        Some((name, kind, _)) => Ok((name, *kind)),
        None => Err(Refusal::new(
            "fbt_return.employer_kind_unsupported",
            format!(
                "employer_kind must be one of {} ({NOT_COVERED}); it is {given}",
                KINDS.map(|(name, ..)| name).join(", ")
            ),
        )),
    }
}

fn invalid_days(year: i64, days: i64) -> Refusal {
    Refusal::new(
        "fbt_return.rebatable_days_invalid",
        format!(
            "rebatable_days and employer_days are given together as JSON integers: employer_days from 1 to {days}, the days of the FBT year ended 31 March {year}, on which the employer was an employer, and rebatable_days from 0 to employer_days, the whole days on which it was a rebatable employer (FBTAA s 65J(2A))"
        ),
    )
}

/// The rebatable days and the days the employer was an employer in the year
/// (s 65J(2A)), given together for a rebatable employer only; absent, every
/// day.
pub(super) fn check_days(v: &Value, kind: Kind, year: i64) -> Result<Option<[i64; 2]>, Refusal> {
    let given = ["rebatable_days", "employer_days"].map(|key| v.get(key));
    let Some(key) = ["rebatable_days", "employer_days"]
        .into_iter()
        .zip(given)
        .find_map(|(key, value)| value.map(|_| key))
    else {
        return Ok(None);
    };
    if kind != Kind::Rebatable {
        return Err(Refusal::new(
            "input.field_unknown",
            format!("{key} is an input of {COMMAND} only with employer_kind rebatable"),
        ));
    }
    // The FBT year ended 31 March `year` holds February of `year`.
    let days = if year % 4 == 0 { 366 } else { 365 };
    match given.map(|value| value.and_then(Value::as_i64)) {
        [Some(rebatable), Some(employer)]
            if (1..=days).contains(&employer) && (0..=employer).contains(&rebatable) =>
        {
            Ok(Some([rebatable, employer]))
        }
        _ => Err(invalid_days(year, days)),
    }
}

/// The FBT instalments reported for the year, less any credits claimed
/// (s 105(2)), when given.
pub(super) fn check_instalments(v: &Value) -> Result<Option<Decimal>, Refusal> {
    let Some(given) = v.get("instalments_reported") else {
        return Ok(None);
    };
    let parsed = given.as_str().and_then(|text| Decimal::parse(text).ok());
    match parsed {
        Some(amount)
            if amount.is_finite()
                && amount.exponent() >= -2
                && !amount.lt(&Decimal::from_int(0)).unwrap_or(true) =>
        {
            Ok(Some(amount))
        }
        _ => Err(Refusal::new(
            "input.amount_invalid",
            "instalments_reported must be a string of a non-negative amount with at most 2 decimal places: the FBT instalments for the year reported on the activity statements, less any credits claimed at 6B (FBTAA s 105(2))",
        )),
    }
}

fn entertainment_invalid(at: &str, why: &str) -> Refusal {
    Refusal::new(
        "fbt_return.entertainment_invalid",
        format!("{at}.entertainment {why}"),
    )
}

/// A benefit's entertainment field, which only a rebatable or s 57A employer
/// takes, checked against its worksheet and `excluded`.
pub(super) fn check_entertainment(
    at: &str,
    b: &Value,
    command: &str,
    excluded: bool,
    kind: Kind,
) -> Result<Entertainment, Refusal> {
    let Some(given) = b.get("entertainment") else {
        return Ok(Entertainment::Unstated);
    };
    if kind == Kind::Taxable {
        return Err(Refusal::new(
            "input.field_unknown",
            format!(
                "entertainment is an input of a benefit within {COMMAND} only with a rebatable or s 57A employer_kind"
            ),
        ));
    }
    match given.as_str() {
        Some("salary_packaged")
            if matches!(command, "fbt-car-parking" | "fbt-meal-entertainment") =>
        {
            Err(entertainment_invalid(
                at,
                &format!(
                    "cannot be salary_packaged for {command}, which values only benefits outside a salary packaging arrangement or car parking"
                ),
            ))
        }
        Some("salary_packaged") if excluded => Err(entertainment_invalid(
            at,
            "salary_packaged needs excluded false: salary packaged meal entertainment and entertainment facility leasing are not excluded fringe benefits (FBTAA s 5E(3))",
        )),
        Some("salary_packaged") => Ok(Entertainment::SalaryPackaged),
        Some("not_salary_packaged") if !excluded => Err(entertainment_invalid(
            at,
            "not_salary_packaged needs excluded true: meal entertainment and entertainment facility leasing outside a salary packaging arrangement are excluded fringe benefits (FBTAA s 5E(3))",
        )),
        Some("not_salary_packaged") => Ok(Entertainment::NotSalaryPackaged),
        _ => Err(entertainment_invalid(
            at,
            "must be salary_packaged or not_salary_packaged: meal entertainment (s 37AD) or a benefit attributable to entertainment facility leasing expenses, provided under a salary packaging arrangement or not",
        )),
    }
}

/// An exempt electric car under a rebatable or s 57A employer is refused.
pub(super) fn check_exempt_car(at: &str, input: &Value, kind: Kind) -> Result<(), Refusal> {
    if kind == Kind::Taxable || input["electric_car_exempt"] != json!(true) {
        return Ok(());
    }
    Err(Refusal::new(
        "fbt_return.nfp_exempt_car_unsupported",
        format!(
            "{at}: an exempt electric car is not composed for a rebatable or s 57A employer: s 135Q(1)(c), which s 5B(1K) step 1 reads, counts a car exempt under s 8A as if it were a fringe benefit, and no ATO example shows how it meets the cap"
        ),
    ))
}
