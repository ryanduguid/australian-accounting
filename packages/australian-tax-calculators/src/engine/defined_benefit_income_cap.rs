// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `defined_benefit_income_cap`: the defined benefit income cap (ITAA 1997
//! s 303-4), the 50% of the excess that is assessable (s 303-2) and the
//! reduced untaxed element offset (s 303-3, with ss 301-100(2) and
//! 302-85(2)), for income years 2017-18 to 2026-27. Read on the ATO's legal
//! database on 24 November 2026 with LCR 2017/1, whose Examples 2 to 6 the
//! tests reproduce. Which benefits get concessional treatment (age 60, or a
//! death benefit from someone who died at 60 or over) is the operator's
//! statement in the 3 amounts.

use serde_json::Value;

use super::{
    Arg, Result, Step, date_arg, derived_result, dollars_up, int, money, precise, rate, refuse,
    scope, step,
};
use crate::dates;
use crate::decimal::Decimal;
use crate::metadata as meta;

const KIND: &str = "defined_benefit_income_cap";

pub struct DefinedBenefitIncomeCapArgs<'a> {
    pub tax_free_and_taxed: &'a Arg,
    pub untaxed: &'a Arg,
    pub other_defined_benefit_income: &'a Arg,
    pub first_day: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// s 294-35(3): the general transfer balance cap for the income year.
fn general_cap(year: &str) -> Decimal {
    let earlier = meta::EARLIER_TRANSFER_BALANCE_CAPS
        .iter()
        .find(|(y, _)| *y == year)
        .map(|(_, cap)| *cap);
    let later = || {
        meta::CONTRIBUTION_CAPS
            .iter()
            .find(|(y, _)| *y == year)
            .map(|(_, (_, cap))| *cap)
    };
    super::d(earlier.or_else(later).expect("a supported year"))
}

/// The income year's first and last days.
fn year_bounds(year: &str) -> (i64, i64) {
    let start: i64 = year[..4].parse().expect("supported year");
    let day = |y: i64, md: &str| dates::parse(&format!("{y}-{md}")).expect("valid date");
    (day(start, "07-01"), day(start + 1, "06-30"))
}

/// s 303-4(2): the days remaining in the year after `first_day`, or `None`
/// for a full year's cap.
fn days_after(first_day: Option<&Arg>, first: i64, last: i64) -> Result<Option<i64>> {
    let Some(given) = first_day else {
        return Ok(None);
    };
    let day = date_arg(given, "first_day")?;
    if day < first || day > last {
        return refuse(
            "defined_benefit_income_cap.first_day_outside_year",
            "first_day must be in the income year: it is the day in this year on which s 301-10, 301-100, 302-65 or 302-85 first applied to your defined benefit income (ITAA 1997 s 303-4(2)). Leave it out when that day was in an earlier year.",
        );
    }
    Ok(Some(last - day))
}

/// s 303-4(1) or (2), then (3): the cap before and after the reduction for
/// defined benefit income without concessional treatment.
fn cap_steps(
    general: &Decimal,
    after: Option<i64>,
    year_days: i64,
    other: &Decimal,
) -> Result<[Step; 2]> {
    let unreduced = match after {
        None => step(
            "unreduced_cap",
            "ITAA 1997 s 303-4(1): the general transfer balance cap / 16, rounded up to the dollar",
            "dollars_up(general_transfer_balance_cap / 16)",
            &[general],
            dollars_up(&general.div(&int(16))?)?,
        ),
        Some(after) => {
            let (after, days) = (int(after), int(year_days));
            let share = int(1).add(&after)?.div(&days)?;
            step(
                "unreduced_cap",
                "ITAA 1997 s 303-4(2): pro-rated from the first day, rounded up to the dollar",
                "dollars_up(general_transfer_balance_cap / 16 x (1 + days_after_first_day) / days_in_year)",
                &[general, &after, &days],
                dollars_up(&general.div(&int(16))?.mul(&share)?)?,
            )
        }
    };
    let reduced = super::py_max(int(0), unreduced.value.sub(other)?)?;
    let cap = step(
        "defined_benefit_income_cap",
        "ITAA 1997 s 303-4(3): reduced by defined benefit income to which none of ss 301-10, 301-100, 302-65 and 302-85 applies, not below nil (LCR 2017/1 paragraph 61)",
        "max(0, unreduced_cap - other_defined_benefit_income)",
        &[&unreduced.value, other],
        reduced,
    );
    Ok([unreduced, cap])
}

/// ss 303-2 and 303-3: the assessable half of the excess and the offset.
fn excess_steps(cap: &Decimal, taxed: &Decimal, untaxed: &Decimal) -> Result<[Step; 5]> {
    let half = super::d("0.5");
    let tenth = super::d("0.1");
    let excess = super::py_max(int(0), taxed.sub(cap)?)?;
    let assessable = excess.mul(&half)?;
    let offset = untaxed.mul(&tenth)?;
    let all_excess = super::py_max(int(0), taxed.add(untaxed)?.sub(cap)?)?;
    let reduction = all_excess.mul(&tenth)?;
    let left = super::py_max(int(0), offset.sub(&reduction)?)?;
    Ok([
        step(
            "excess_over_cap",
            "ITAA 1997 s 303-2(1)(b): the tax free components and taxed elements over the cap",
            "max(0, tax_free_and_taxed - defined_benefit_income_cap)",
            &[taxed, cap],
            excess.clone(),
        ),
        step(
            "assessable_income",
            "ITAA 1997 s 303-2(1): 50% of the excess is assessable income, with no offset (LCR 2017/1 paragraph 34)",
            "excess_over_cap x 0.5",
            &[&excess],
            assessable,
        ),
        step(
            "untaxed_offset_before_reduction",
            "ITAA 1997 ss 301-100(2) and 302-85(2): 10% of the untaxed elements",
            "untaxed x 0.1",
            &[untaxed],
            offset.clone(),
        ),
        step(
            "offset_reduction",
            "ITAA 1997 s 303-3: 10% of the concessional defined benefit income, untaxed elements included, over the cap",
            "max(0, tax_free_and_taxed + untaxed - defined_benefit_income_cap) x 0.1",
            &[taxed, untaxed, cap],
            reduction.clone(),
        ),
        step(
            "untaxed_offset",
            "ITAA 1997 s 303-3: the offsets less the reduction, not below nil",
            "max(0, untaxed_offset_before_reduction - offset_reduction)",
            &[&offset, &reduction],
            left,
        ),
    ])
}

pub fn defined_benefit_income_cap(args: &DefinedBenefitIncomeCapArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let taxed = money(args.tax_free_and_taxed)?;
    let untaxed = money(args.untaxed)?;
    let other = money(args.other_defined_benefit_income)?;
    let (first, last) = year_bounds(&year);
    let year_days = last - first + 1;
    let after = days_after(args.first_day, first, last)?;
    let general = general_cap(&year);
    let steps = precise(|| {
        let [unreduced, cap] = cap_steps(&general, after, year_days, &other)?;
        let rest = excess_steps(&cap.value, &taxed, &untaxed)?;
        let mut steps = vec![unreduced, cap];
        steps.extend(rest);
        Ok(steps)
    })?;
    let mut rates = vec![
        rate("general_transfer_balance_cap", &general),
        rate("days_in_year", year_days),
    ];
    if let Some(after) = after {
        rates.push(rate("days_after_first_day", after));
    }
    derived_result(KIND, &year, steps, rates)
}
