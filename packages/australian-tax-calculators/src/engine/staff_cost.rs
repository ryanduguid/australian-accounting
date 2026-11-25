// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `staff-cost`: one staff member's annual cost and monthly average, as the
//! site's staff cost page works it (tools/business-calculators/staff, read on
//! 26 November 2026): wages + super + other costs, with the month a twelfth
//! rounded half up. Beyond the page, the super guarantee can be worked from a
//! stated base at the year's charge percentage, capped by the maximum
//! contribution base, and payroll tax and workers compensation added at rates
//! the operator states. The percentages and bases are the ATO's 'Super
//! guarantee' tables 21, 23 and 24 (QC18123, updated 17 April 2026) and SGAA
//! s 17A(2) as compiled on 1 July 2026 (compilation 78), read on
//! 26 November 2026.

use serde_json::Value;

use super::{Arg, Result, Step, d, derived_result, money, precise, rate, refuse, scope, step};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "staff_cost";
const PAGE: &str = "The site's staff cost page: annual cost = wages + super + other annual costs; monthly average = annual cost / 12, half up";
const SG_2026: &str = "SGAA s 17A(2) (charge percentage 12, from 1 July 2026) and ATO 'Super guarantee' table 23: the annual maximum contribution base caps qualifying earnings";
const SG_QUARTERLY: &str = "ATO 'Super guarantee' tables 21 and 24: the year's super guarantee percentage on ordinary time earnings, a base no higher than one quarter's maximum contribution base, so no quarter can pass it";
const STATED: &str = "The operator's stated rate on wages plus super; the scheme sets its own base, threshold and exemptions";

/// (year, super guarantee percentage, maximum contribution base, annual).
const SG: [(&str, &str, &str, bool); 6] = [
    ("2021-22", "0.10", "58920", false),
    ("2022-23", "0.105", "60220", false),
    ("2023-24", "0.11", "62270", false),
    ("2024-25", "0.115", "65070", false),
    ("2025-26", "0.12", "62500", false),
    ("2026-27", "0.12", "270830", true),
];

pub struct StaffCostArgs<'a> {
    pub wages: &'a Arg,
    pub other_costs: &'a Arg,
    pub super_amount: Option<&'a Arg>,
    pub super_base: Option<&'a Arg>,
    pub payroll_tax_rate: Option<&'a Arg>,
    pub workers_compensation_rate: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// A stated rate in per cent, 0 to 100 to at most 4 decimal places; 0 when absent.
fn percent(given: Option<&Arg>, name: &str) -> Result<Decimal> {
    match given {
        None => Ok(d("0")),
        Some(Arg::Decimal(r))
            if r.is_finite() && r.exponent() >= -4 && !r.lt(&d("0"))? && !r.gt(&d("100"))? =>
        {
            Ok(r.clone())
        }
        Some(_) => refuse(
            "staff_cost.rate_invalid",
            format!("{name} must be a per cent from 0 to 100, at most 4 decimal places."),
        ),
    }
}

/// The super step: the stated amount, or the super guarantee on the base.
fn superannuation(
    a: &StaffCostArgs,
    year: &str,
    rates: &mut Vec<(String, String)>,
) -> Result<Step> {
    match (a.super_amount, a.super_base) {
        (Some(amount), None) => {
            let amount = money(amount)?;
            Ok(step(
                "superannuation",
                PAGE,
                "super_amount",
                &[&amount],
                amount.clone(),
            ))
        }
        (None, Some(base)) => guarantee(&money(base)?, year, rates),
        _ => refuse(
            "staff_cost.super_basis_invalid",
            "Give exactly one of super_amount (the super you pay) and super_base (the earnings the super guarantee is worked on).",
        ),
    }
}

fn guarantee(base: &Decimal, year: &str, rates: &mut Vec<(String, String)>) -> Result<Step> {
    let (_, percentage, cap, annual) = SG
        .iter()
        .find(|row| row.0 == year)
        .expect("a supported year");
    let (sg_rate, cap) = (d(percentage), d(cap));
    rates.push(rate("sg_rate", percentage));
    if *annual {
        rates.push(rate("contribution_base", &cap));
        let value = precise(|| {
            let used = if base.gt(&cap)? { &cap } else { base };
            Ok(sg_rate.mul(used)?.quantize(-2, Rounding::HalfUp)?)
        })?;
        return Ok(step(
            "superannuation",
            SG_2026,
            "cents(sg_rate x min(super_base, contribution_base))",
            &[&sg_rate, base, &cap],
            value,
        ));
    }
    rates.push(rate("quarterly_contribution_base", &cap));
    // An annual base above one quarter's cap may have crossed it in a quarter,
    // which the annual figure cannot show.
    if base.gt(&cap)? {
        return refuse(
            "staff_cost.base_above_quarterly_cap",
            format!(
                "super_base is above the quarterly maximum contribution base ({cap}) for {year}: each quarter is capped alone, so give super_amount instead."
            ),
        );
    }
    let value = precise(|| Ok(sg_rate.mul(base)?.quantize(-2, Rounding::HalfUp)?))?;
    Ok(step(
        "superannuation",
        SG_QUARTERLY,
        "cents(sg_rate x super_base)",
        &[&sg_rate, base],
        value,
    ))
}

/// An on-cost at a stated rate on wages plus super.
fn on_cost(
    id: &'static str,
    name: &str,
    wages: &Decimal,
    sup: &Decimal,
    r: &Decimal,
) -> Result<Step> {
    let value = precise(|| {
        Ok(wages
            .add(sup)?
            .mul(r)?
            .div(&d("100"))?
            .quantize(-2, Rounding::HalfUp)?)
    })?;
    let formula = format!("cents((wages + superannuation) x {name} / 100)");
    Ok(step(id, STATED, &formula, &[wages, sup, r], value))
}

/// The annual cost of [wages, super, payroll tax, workers compensation,
/// other costs] and its monthly average.
fn totals(parts: [&Decimal; 5]) -> Result<[Step; 2]> {
    let annual = precise(|| {
        let mut sum = d("0");
        for part in parts {
            sum = sum.add(part)?;
        }
        Ok(sum)
    })?;
    let monthly = precise(|| Ok(annual.div(&d("12"))?.quantize(-2, Rounding::HalfUp)?))?;
    Ok([
        step(
            "annual_cost",
            PAGE,
            "wages + superannuation + payroll_tax + workers_compensation + other_costs",
            &parts,
            annual.clone(),
        ),
        step(
            "monthly_average",
            PAGE,
            "cents(annual_cost / 12)",
            &[&annual],
            monthly,
        ),
    ])
}

/// Annual cost and monthly average of one staff member.
pub fn staff_cost(a: &StaffCostArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let wages = money(a.wages)?;
    let other = money(a.other_costs)?;
    let mut rates = Vec::new();
    let sup = superannuation(a, &year, &mut rates)?;
    let payroll = percent(a.payroll_tax_rate, "payroll_tax_rate")?;
    let workers = percent(a.workers_compensation_rate, "workers_compensation_rate")?;
    rates.push(rate("payroll_tax_rate", &payroll));
    rates.push(rate("workers_compensation_rate", &workers));
    let tax = on_cost(
        "payroll_tax",
        "payroll_tax_rate",
        &wages,
        &sup.value,
        &payroll,
    )?;
    let comp = on_cost(
        "workers_compensation",
        "workers_compensation_rate",
        &wages,
        &sup.value,
        &workers,
    )?;
    let [annual, monthly] = totals([&wages, &sup.value, &tax.value, &comp.value, &other])?;
    derived_result(KIND, &year, vec![sup, tax, comp, annual, monthly], rates)
}
