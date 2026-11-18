// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The housing worksheet (FBTAA Division 6): a housing benefit's taxable
//! value, which `fbt` then grosses up with the others. It cites the FBTAA as
//! compiled on 1 July 2026 (C2004A03280), the ATO's FBT rates and thresholds
//! page, 'Fringe benefits tax - a guide for employers', chapter 10, and the
//! FBT return 2026 instructions, item 23, read on 15 November 2026.

use serde_json::Value;

use super::super::{
    APPLIED, Arg, EXCESS, NET, Result, Step, cents, d, day_count, derived_result, fbt_year, flag,
    int, lookup, money, precise, py_min, rate, refuse, step, steps_from,
};
use super::{method, take};
use crate::decimal::Decimal;
use crate::metadata as meta;

pub(super) const HOUSING_METHODS: [&str; 4] = [
    "outside_australia",
    "accommodation_business",
    "base_year",
    "indexed",
];

const HOUSING: &str = "fbt-housing";

/// FBTAA Division 6, s 26: the market value of the recipient's current
/// housing right outside Australia (s 26(1)(a)) or in a business that lets
/// identical accommodation to outsiders (s 26(1)(b)), or otherwise its
/// statutory annual value for the tenancy period (s 26(1)(c)), worked out in a
/// base year (s 26(2)(a)) or indexed from the year before (s 26(2)(b)); less
/// the recipient's rent, never below nil.
pub fn fbt_housing(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_housing";
    let (year, period, days_in_year) = fbt_year(scope_confirmed, year_ended, kind)?;
    let method = method(method_arg, kind)?;
    let when = format!(" with method {method}");
    let (contribution, mut steps, rates) = match method {
        "outside_australia" | "accommodation_business" => market_value(given, &when, method)?,
        _ => {
            let year = year.parse().expect("FBT year");
            statutory_value(given, &when, method, year, days_in_year)?
        }
    };
    let contribution = money(contribution)?;
    let gross = steps.last().expect("gross step").value.clone();
    steps.extend(precise(|| less_rent(&contribution, &gross))?);
    derived_result(kind, &period, steps, rates)
}

/// s 26(1)(a) and (b): the market value, for an in-house right in an
/// accommodation business 75% of it.
fn market_value<'a>(given: &'a [(String, Arg)], when: &str, method: &str) -> Result<Housing<'a>> {
    if method == "outside_australia" {
        let names = ["market_value", "employee_contribution"];
        let [value, contribution] = take(given, HOUSING, when, names)?;
        let value = money(value)?;
        let gross = step(
            "gross_taxable_value",
            "FBTAA s 26(1)(a)",
            "market_value",
            &[&value],
            value.clone(),
        );
        return Ok((contribution, vec![gross], Vec::new()));
    }
    let names = ["market_value", "in_house", "employee_contribution"];
    let [value, in_house, contribution] = take(given, HOUSING, when, names)?;
    let value = money(value)?;
    let factor = d(if flag(in_house, "in_house")? {
        "0.75"
    } else {
        "1"
    });
    let gross = precise(|| Ok(cents(value.mul(&factor)?)?))?;
    let formula = "cents(market_value x in_house_factor)";
    let gross = step(
        "gross_taxable_value",
        "FBTAA s 26(1)(b)",
        formula,
        &[&value, &factor],
        gross,
    );
    Ok((
        contribution,
        vec![gross],
        vec![rate("in_house_factor", &factor)],
    ))
}

/// s 26(1): the recipient's rent applied, any excess, and the gross taxable
/// value less the rent applied, never below nil.
fn less_rent(contribution: &Decimal, gross: &Decimal) -> Result<Vec<Step>> {
    let authority = "FBTAA s 26(1), recipients rent";
    let applied = py_min(contribution.clone(), gross.clone())?;
    let table = [
        ("employee_contribution_applied", authority, APPLIED),
        ("excess_employee_contribution", authority, EXCESS),
        ("taxable_value", authority, NET),
    ];
    let rows: [(&[&Decimal], Decimal); 3] = [
        (&[contribution, gross], applied.clone()),
        (&[contribution, &applied], contribution.sub(&applied)?),
        (&[gross, &applied], gross.sub(&applied)?),
    ];
    Ok(steps_from(&table, rows))
}

/// A housing worksheet's recipient's rent input, its steps to the gross
/// taxable value and its rates.
type Housing<'a> = (&'a Arg, Vec<Step>, Vec<(String, String)>);

/// s 26(1)(c): the statutory annual value, worked out in a base year from the
/// market value for the tenancy period (s 26(2)(a)) or as the year before's
/// value times the State or Territory's indexation factor (s 26(2)(b)), for
/// the whole days of the tenancy period.
fn statutory_value<'a>(
    given: &'a [(String, Arg)],
    when: &str,
    method: &str,
    year: i64,
    days_in_year: i64,
) -> Result<Housing<'a>> {
    let days = int(days_in_year);
    let (contribution, tenancy, annual, mut rates) = if method == "base_year" {
        base_year_value(given, when, &days, days_in_year)?
    } else {
        indexed_value(given, when, year, days_in_year)?
    };
    let value = annual.value.clone();
    let gross = precise(|| Ok(cents(value.mul(&tenancy)?.div(&days)?)?))?;
    let formula = "cents(statutory_annual_value x tenancy_days / days_in_year)";
    let gross = step(
        "gross_taxable_value",
        "FBTAA s 26(1)(c)",
        formula,
        &[&value, &tenancy, &days],
        gross,
    );
    rates.push(rate("days_in_year", days_in_year));
    Ok((contribution, vec![annual, gross], rates))
}

/// A statutory annual value's recipient's rent input, tenancy days, step and
/// rates.
type Annual<'a> = (&'a Arg, Decimal, Step, Vec<(String, String)>);

/// s 26(2)(a): a base year's statutory annual value, from the market value for
/// the tenancy period.
fn base_year_value<'a>(
    given: &'a [(String, Arg)],
    when: &str,
    days: &Decimal,
    days_in_year: i64,
) -> Result<Annual<'a>> {
    let names = ["market_value", "tenancy_days", "employee_contribution"];
    let [value, tenancy, contribution] = take(given, HOUSING, when, names)?;
    let value = money(value)?;
    // The base year's value divides by the tenancy's days, so it needs one.
    let tenancy = match tenancy.int() {
        Some(Some(n)) if (1..=days_in_year).contains(&n) => int(n),
        _ => {
            return refuse(
                "input.day_count_invalid",
                format!(
                    "tenancy_days must be an integer from 1 to {days_in_year} with method base_year."
                ),
            );
        }
    };
    let annual = precise(|| Ok(cents(value.mul(days)?.div(&tenancy)?)?))?;
    let formula = "cents(market_value x days_in_year / tenancy_days)";
    let annual = step(
        "statutory_annual_value",
        "FBTAA s 26(2)(a)",
        formula,
        &[&value, days, &tenancy],
        annual,
    );
    Ok((contribution, tenancy, annual, Vec::new()))
}

/// s 26(2)(b): the year before's statutory annual value times the State or
/// Territory's indexation factor (s 28).
fn indexed_value<'a>(
    given: &'a [(String, Arg)],
    when: &str,
    year: i64,
    days_in_year: i64,
) -> Result<Annual<'a>> {
    let names = [
        "prior_statutory_annual_value",
        "state",
        "base_year_ended",
        "tenancy_days",
        "employee_contribution",
    ];
    let [prior, state, base, tenancy, contribution] = take(given, HOUSING, when, names)?;
    let prior = money(prior)?;
    let factor = indexation_factor(state, year)?;
    base_year_ended(base, year)?;
    let tenancy = int(day_count(tenancy, days_in_year, "tenancy_days")?);
    let annual = precise(|| Ok(cents(prior.mul(&factor)?)?))?;
    let formula = "cents(prior_statutory_annual_value x indexation_factor)";
    let annual = step(
        "statutory_annual_value",
        "FBTAA s 26(2)(b) and s 28; FBT rates and thresholds, housing indexation",
        formula,
        &[&prior, &factor],
        annual,
    );
    Ok((
        contribution,
        tenancy,
        annual,
        vec![rate("indexation_factor", &factor)],
    ))
}

/// The housing indexation factor for `state` in the FBT year ended 31 March
/// `year` (s 28), as the ATO's rates page publishes it.
fn indexation_factor(state: &Arg, year: i64) -> Result<Decimal> {
    let table = lookup(meta::FBT_HOUSING_INDEXATION, &year.to_string());
    if let Some((_, factor)) = state
        .text()
        .and_then(|s| table.iter().find(|(k, _)| *k == s))
    {
        return Ok(d(factor));
    }
    refuse(
        "input.state_invalid",
        "state must be NSW, VIC, QLD, SA, WA, TAS, NT or ACT, where the unit of accommodation is: the Jervis Bay Territory is part of NSW, and Christmas Island and the Cocos (Keeling) Islands are part of NT (FBTAA s 28(5)).",
    )
}

/// The base year must be one of the 9 years before `year`: after 9 years
/// without one, the year is a base year again (s 26(3)(c)).
fn base_year_ended(value: &Arg, year: i64) -> Result<()> {
    match value.int() {
        Some(Some(base)) if (year - 9..year).contains(&base) => Ok(()),
        _ => refuse(
            "fbt_housing.base_year_invalid",
            format!(
                "base_year_ended must be the FBT year, from {} to {}, last treated as a base year for this housing right: a year after 9 years without one is a base year (FBTAA s 26(3)(c)), so use method base_year.",
                year - 9,
                year - 1
            ),
        ),
    }
}
