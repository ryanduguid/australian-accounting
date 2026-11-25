// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `break-even`: the whole units, and the sales, that cover a period's fixed
//! costs at one price and one variable cost a unit, as the site's break-even
//! page works it (tools/business-calculators/break-even, read on
//! 26 November 2026): contribution = price - variable cost; units = fixed
//! costs / contribution, rounded up to a whole unit; sales = units x price.
//! The definitions follow Business Queensland's break-even page. No statute
//! sets a rate or threshold here.

use super::pricing::up_to;
use super::{Arg, Result, d, derived_result, money, precise, rate, refuse, scope, step};
use crate::decimal::Decimal;

const KIND: &str = "break_even";
const SOURCE: &str = "Business Queensland, 'Break-even and profit': contribution margin and break-even point; the site's break-even page rounds part of a unit up";

/// The worksheet's inputs, as the caller gave them.
pub struct BreakEvenArgs<'a> {
    pub fixed_costs: &'a Arg,
    pub price: &'a Arg,
    pub variable_cost: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// Contribution, whole units and sales, refused where none breaks even.
fn figures(fixed: &Decimal, price: &Decimal, variable: &Decimal) -> Result<[Decimal; 3]> {
    if !price.gt(variable)? {
        return refuse(
            "break_even.no_contribution",
            "price must exceed variable_cost: a unit that adds nothing to fixed costs never breaks even.",
        );
    }
    let [contribution, units, sales] = precise(|| {
        let contribution = price.sub(variable)?;
        let units = up_to(&fixed.div(&contribution)?, &d("1"))?;
        let sales = units.mul(price)?;
        Ok([contribution, units, sales])
    })?;
    if sales.gt(&d("1e12"))? {
        return refuse(
            "break_even.sales_out_of_range",
            "The sales needed exceed 1000000000000.00: use smaller amounts.",
        );
    }
    Ok([contribution, units, sales])
}

/// Contribution a unit, whole units to break even and the sales they make.
pub fn break_even(a: &BreakEvenArgs) -> Result<serde_json::Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let fixed = money(a.fixed_costs)?;
    let price = money(a.price)?;
    let variable = money(a.variable_cost)?;
    let [contribution, units, sales] = figures(&fixed, &price, &variable)?;
    let steps = vec![
        step(
            "contribution",
            SOURCE,
            "price - variable_cost",
            &[&price, &variable],
            contribution.clone(),
        ),
        step(
            "units",
            SOURCE,
            "dollars_up(fixed_costs / contribution)",
            &[&fixed, &contribution],
            units.clone(),
        ),
        step("sales", SOURCE, "units x price", &[&units, &price], sales),
    ];
    derived_result(KIND, &year, steps, vec![rate("whole_units", units)])
}
