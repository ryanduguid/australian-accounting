// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `portable_lsl_levy`: Australian portable long service leave levies across
//! the national black coal mining industry scheme (Coal LSL, mirroring
//! `ryanduguid.github.io/tools/coal-lsl-levy`) and the state and territory
//! portable schemes for construction, community services, contract cleaning
//! and security. Every rate, threshold and statutory rounding rule was read
//! from the administering board's published pages and legislation on 24
//! November 2026; `docs/calculation-evidence.md#portable-lsl-levy` records
//! every source read and any unverified statute.

use serde_json::Value;

use super::{Arg, Result, d, int, money, refuse, scope};
use crate::decimal::{Calc, Decimal, Rounding};

mod coal;
mod states;

const KIND: &str = "portable_lsl_levy";

/// The facts the `portable-lsl-levy` command passes; see the README for each.
pub struct PortableLslLevyArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub scheme: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub ordinary_wages: Option<&'a Arg>,
    pub paid_on_time: Option<&'a Arg>,
    pub project_cost: Option<&'a Arg>,
    pub partial_exemption: Option<&'a Arg>,
    pub reporting_month: Option<&'a Arg>,
    pub branch: Option<&'a Arg>,
    pub eligible_employee: Option<&'a Arg>,
    pub base_rate_of_pay: Option<&'a Arg>,
    pub annual_salary_paid: Option<&'a Arg>,
    pub casual_loading: Option<&'a Arg>,
    pub ordinary_rate_of_pay: Option<&'a Arg>,
    pub salary_sacrificed: Option<&'a Arg>,
    pub overtime_and_penalty_rates: Option<&'a Arg>,
    pub allowances: Option<&'a Arg>,
    pub allowances_exclude_expense_reimbursements: Option<&'a Arg>,
    pub bonuses: Option<&'a Arg>,
    pub instrument_specifies_loading: Option<&'a Arg>,
    pub loading_quantifiable: Option<&'a Arg>,
}

/// Round a non-negative amount to the nearest whole dollar, with 50 cents
/// rounded downwards: Queensland BCI Act 1991 s 72(2) and Northern Territory
/// Construction Industry Long Service Leave and Benefits Act 2005 s 33(2).
pub(super) fn dollars_half_down(value: &Decimal) -> Calc<Decimal> {
    let down = value.quantize(0, Rounding::Down)?;
    let frac = value.sub(&down)?;
    if frac.gt(&d("0.5"))? {
        down.add(&int(1))
    } else {
        Ok(down)
    }
}

/// Refuse when any disallowed optional field is supplied for the chosen scheme
/// or branch, so an amount or flag the branch ignores is never silently dropped.
pub(super) fn reject_present(fields: &[(&str, Option<&Arg>)], context: &str) -> Result<()> {
    for &(name, value) in fields {
        if value.is_some() {
            return refuse(
                "portable_lsl_levy.branch_field_mismatch",
                format!("{name} does not apply to {context}."),
            );
        }
    }
    Ok(())
}

/// Read a required decimal money field.
pub(super) fn required_money(value: Option<&Arg>, name: &str, context: &str) -> Result<Decimal> {
    match value {
        Some(v) => money(v),
        None => refuse(
            "input.missing_field",
            format!("{name} is required for {context}."),
        ),
    }
}

/// Read an optional decimal money field, returning `(value, was_supplied)`.
pub(super) fn optional_money(value: Option<&Arg>) -> Result<(Decimal, bool)> {
    match value {
        Some(v) => Ok((money(v)?, true)),
        None => Ok((int(0), false)),
    }
}

pub fn portable_lsl_levy(args: &PortableLslLevyArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let jurisdiction = match args.jurisdiction.text() {
        Some(j @ ("cth" | "nsw" | "vic" | "qld" | "sa" | "wa" | "act" | "tas" | "nt")) => j,
        _ => {
            return refuse(
                "portable_lsl_levy.jurisdiction_invalid",
                "jurisdiction must be cth, nsw, vic, qld, sa, wa, act, tas or nt.",
            );
        }
    };
    let scheme = match args.scheme.text() {
        Some(
            s @ ("coal" | "construction" | "community_services" | "contract_cleaning" | "security"),
        ) => s,
        _ => {
            return refuse(
                "portable_lsl_levy.scheme_invalid",
                "scheme must be coal, construction, community_services, contract_cleaning or security.",
            );
        }
    };
    match (jurisdiction, scheme) {
        ("cth", "coal") => coal::calculate(args, &year),
        ("cth", _) | (_, "coal") => refuse(
            "portable_lsl_levy.scheme_unavailable",
            format!(
                "No portable long service leave scheme is covered for jurisdiction '{jurisdiction}' and scheme '{scheme}' (use jurisdiction 'cth' with scheme 'coal' for Coal LSL)."
            ),
        ),
        _ => states::calculate(jurisdiction, scheme, args, &year),
    }
}
