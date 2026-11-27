// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Subdivisions 820-B to 820-E: a financial entity (non-ADI) by its maximum
//! allowable debt (ss 820-85 to 820-115 and 820-185 to 820-220) or the third
//! party debt test it chose (ss 820-85(2C), 820-115(1), 820-185(2C) and
//! 820-220(1)), and an ADI by its minimum capital amount (ss 820-300 to
//! 820-325 and 820-395 to 820-415), from stated average values (Subdivision
//! 820-G). The arm's length capital amount (ss 820-315 and 820-410) is a
//! judgement the worksheet does not make: it is used only as stated, and a
//! shortfall it could remove is refused rather than estimated. Part-year
//! periods (ss 820-120, 820-225, 820-330 and 820-420) are refused. Read on
//! 27 November 2026 in ITAA 1997 compilation 268 (C2026C00471).

mod adi;
mod financial;

use super::inputs::{Class, Deduction, OLDER, Test, fact, not_used, required};
use super::{ALLOWED, ThinCapitalisationArgs, Worked, max0, sum_step, value_of};
use crate::decimal::{Calc, Decimal};
use crate::engine::{Result, Step, d, flag, precise, refuse, step};
use adi::adi;
use financial::financial;

const DD: &str = "s 820-40: the year's debt deductions the Subdivision tests, after any disallowed under Subdivision 820-EAA (s 820-31)";

fn stated_flag(a: &ThinCapitalisationArgs, key: &str, why: &str) -> Result<bool> {
    match a.get(key) {
        Some(v) => flag(v, key),
        None => refuse(
            "thin_capitalisation.input_missing",
            format!("{key} is needed: {why}."),
        ),
    }
}

/// Refuses every input of Subdivisions 820-B to 820-E outside `uses`.
fn only(a: &ThinCapitalisationArgs, uses: &[&str], why: &str) -> Result<()> {
    match OLDER
        .iter()
        .find(|k| !uses.contains(k) && a.get(k).is_some())
    {
        Some(k) => not_used(k, why),
        None => Ok(()),
    }
}

/// `None` when none of `keys` is given, every value when all are, and a
/// refusal naming the first missing one otherwise.
fn all_or_none(
    a: &ThinCapitalisationArgs,
    keys: &[&str],
    why: &str,
) -> Result<Option<Vec<Decimal>>> {
    if keys.iter().all(|k| a.get(k).is_none()) {
        return Ok(None);
    }
    keys.iter()
        .map(|k| required(a, k, why))
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

/// A step adding and subtracting stated terms: `+` or `-` before each name
/// after the first. With `floor`, a negative result is taken to be nil.
fn line(
    id: &'static str,
    authority: &'static str,
    terms: &[(char, &str, &Decimal)],
    floor: bool,
) -> Calc<Step> {
    let mut formula = String::new();
    let mut value = d("0");
    for (n, (sign, name, v)) in terms.iter().enumerate() {
        if n > 0 {
            formula.push_str(&format!(" {sign} "));
        }
        formula.push_str(name);
        value = if *sign == '-' {
            value.sub(v)?
        } else {
            value.add(v)?
        };
    }
    if floor {
        formula = format!("max({formula}, 0)");
        value = max0(value)?;
    }
    let operands: Vec<&Decimal> = terms.iter().map(|(_, _, v)| *v).collect();
    Ok(step(id, authority, &formula, &operands, value))
}

fn dd_step(items: &[Deduction]) -> Calc<Step> {
    let amounts: Vec<&Decimal> = items.iter().map(|x| &x.amount).collect();
    sum_step("debt_deductions", DD, &amounts)
}

/// The disallowed amount under ss 820-115(3), 820-220(3), 820-325 and
/// 820-415: each debt deduction times the excess (or shortfall) over average
/// debt, and no more than all of it (ss 820-85(1), 820-185(1), 820-300(1) and
/// 820-395(1) disallow 'all or a part' of each).
fn proportionate(
    steps: &mut Vec<Step>,
    excess_id: &'static str,
    authority: &'static str,
) -> Result<()> {
    let dd = value_of(steps, "debt_deductions").clone();
    let excess = value_of(steps, excess_id).clone();
    let avg = value_of(steps, "average_debt").clone();
    if excess.is_zero() {
        steps.push(step("total_disallowed_amount", authority, "0", &[], d("0")));
    } else {
        if !avg.gt(&d("0"))? {
            return fact(format!(
                "average debt is nil, so the {excess_id} cannot be spread over it ({authority})."
            ));
        }
        let formula = format!("min(debt_deductions x {excess_id} / average_debt, debt_deductions)");
        let raw = dd.mul(&excess)?.div(&avg)?;
        let value = if raw.gt(&dd)? { dd.clone() } else { raw };
        steps.push(step(
            "total_disallowed_amount",
            authority,
            &formula,
            &[&dd, &excess, &avg, &dd],
            value,
        ));
    }
    let disallowed = value_of(steps, "total_disallowed_amount").clone();
    steps.push(step(
        "allowed_debt_deductions",
        ALLOWED,
        "debt_deductions - total_disallowed_amount",
        &[&dd, &disallowed],
        dd.sub(&disallowed)?,
    ));
    Ok(())
}

fn not_worked_out<T>(what: &str) -> Result<T> {
    refuse(
        "thin_capitalisation.amount_not_worked_out",
        what.to_string(),
    )
}

/// The part-year rules are not applied.
pub(super) fn whole_year(a: &ThinCapitalisationArgs) -> Result<()> {
    if stated_flag(
        a,
        "whole_year",
        "Subdivisions 820-B to 820-E apply to an entity in its class for all of the income year (ss 820-85(1A), 820-185(1A), 820-300(1) and 820-395(1))",
    )? {
        return Ok(());
    }
    refuse(
        "thin_capitalisation.part_year_not_built",
        "An entity in its class for only part of the income year is tested period by period under ss 820-120, 820-225, 820-330 or 820-420, which this worksheet does not apply.",
    )
}

/// A financial entity's third party debt test, chosen under s 820-85(2C) or
/// 820-185(2C).
pub(super) fn third_party(
    a: &ThinCapitalisationArgs,
    c: Class,
    items: &[Deduction],
) -> Result<Worked> {
    only(
        a,
        &["whole_year"],
        "the third party debt test compares debt deductions with the third party earnings limit alone (ss 820-115(1) and 820-220(1))",
    )?;
    whole_year(a)?;
    let authority = if c == Class::OutwardFinancial {
        "s 820-115(1): debt deductions, other than those attributable to an overseas permanent establishment, above the third party earnings limit"
    } else {
        "s 820-220(1): debt deductions above the third party earnings limit"
    };
    precise(|| {
        let qualifying: Vec<&Decimal> = items
            .iter()
            .filter(|x| x.third_party == Some(true))
            .map(|x| &x.amount)
            .collect();
        let dd = dd_step(items)?;
        let limit = sum_step(
            "third_party_earnings_limit",
            "s 820-427A(1) and (2): the debt deductions attributable to debt interests satisfying the third party debt conditions",
            &qualifying,
        );
        let limit = limit?;
        let disallowed = max0(dd.value.sub(&limit.value)?)?;
        let total = step(
            "total_disallowed_amount",
            authority,
            "max(debt_deductions - third_party_earnings_limit, 0)",
            &[&dd.value, &limit.value],
            disallowed.clone(),
        );
        let allowed = step(
            "allowed_debt_deductions",
            ALLOWED,
            "debt_deductions - total_disallowed_amount",
            &[&dd.value, &disallowed],
            dd.value.sub(&disallowed)?,
        );
        Ok(Worked {
            steps: vec![dd, limit, total, allowed],
            rates: vec![],
            carried: None,
        })
    })
}

/// The test a financial entity or ADI is worked out by.
pub(super) fn work(
    a: &ThinCapitalisationArgs,
    c: Class,
    t: Test,
    items: &[Deduction],
) -> Result<Worked> {
    match t {
        Test::ThirdParty => third_party(a, c, items),
        _ if c.adi() => adi(a, c, items),
        _ => financial(a, c, items),
    }
}
