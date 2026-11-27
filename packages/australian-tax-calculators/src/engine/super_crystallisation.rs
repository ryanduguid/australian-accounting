// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `super_crystallisation`: the crystallised segment of a superannuation
//! interest's tax free component at 30 June 2007 (ITAA 1997 s 307-225), with
//! its pre-July 83 component under former ITAA 1936 s 27AA(1)(d), read on
//! 24 November 2026 in the compilation of 19 February 2007 (C2007C00085,
//! the formula by OCR of its image) and on the ATO's legal database. The
//! non-qualifying and excessive components are nil, as the ATO's
//! crystallisation calculator takes neither; the eligible service period's
//! days count both its first day and 30 June 2007, as the ATO's 'Calculating
//! components of a super benefit' example 9 counts them.

use serde_json::Value;

use super::{
    Arg, Result, Step, cents, date_arg, derived_result, flag, int, money, precise, rate, refuse,
    scope, step,
};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "super_crystallisation";

pub struct SuperCrystallisationArgs<'a> {
    pub eligible_service_start: &'a Arg,
    pub interest_value: &'a Arg,
    pub concessional_component: &'a Arg,
    pub post_june_1994_invalidity_component: &'a Arg,
    pub cgt_exempt_component: &'a Arg,
    pub undeducted_contributions: &'a Arg,
    pub untaxed_source: &'a Arg,
    pub income_stream_before_july_2007: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The refused interests: an untaxed source and an income stream that began
/// to be paid before 1 July 2007.
fn refused_facts(args: &SuperCrystallisationArgs) -> Result<()> {
    if flag(args.untaxed_source, "untaxed_source")? {
        return refuse(
            "super_crystallisation.untaxed_source",
            "The interest is from an untaxed source: ITAA 1997 s 307-225(3) disregards the element untaxed in the fund for the pre-July 83 component, and the ATO's crystallisation calculator does not take such an interest.",
        );
    }
    if flag(
        args.income_stream_before_july_2007,
        "income_stream_before_july_2007",
    )? {
        return refuse(
            "super_crystallisation.income_stream_before_july_2007",
            "The interest supports an income stream first paid before 1 July 2007: ITTPA 1997 s 307-125 works out its tax free component from the s 27H deductible amount instead.",
        );
    }
    Ok(())
}

/// Days in the eligible service period to 30 June 2007 and those before
/// 1 July 1983, the first and last days counted.
fn service_days(start: i64) -> Result<(i64, i64)> {
    let end = dates::parse("2007-06-30").expect("30 June 2007");
    if start > end {
        return refuse(
            "super_crystallisation.dates_invalid",
            "eligible_service_start must be on or before 30 June 2007: the eligible service period runs to the notional payment just before 1 July 2007 (ITAA 1997 s 307-225(1)).",
        );
    }
    let before = dates::parse("1983-06-30").expect("30 June 1983");
    Ok((end - start + 1, (before - start + 1).max(0)))
}

/// The 4 components s 307-225(2) adds as they are, in order.
struct Components {
    value: Decimal,
    concessional: Decimal,
    invalidity: Decimal,
    cgt: Decimal,
    undeducted: Decimal,
}

fn components(args: &SuperCrystallisationArgs) -> Result<Components> {
    let c = Components {
        value: money(args.interest_value)?,
        concessional: money(args.concessional_component)?,
        invalidity: money(args.post_june_1994_invalidity_component)?,
        cgt: money(args.cgt_exempt_component)?,
        undeducted: money(args.undeducted_contributions)?,
    };
    let sum = c
        .concessional
        .add(&c.invalidity)?
        .add(&c.cgt)?
        .add(&c.undeducted)?;
    if sum.gt(&c.value)? {
        return refuse(
            "super_crystallisation.components_exceed_value",
            "The concessional, post-June 1994 invalidity and CGT exempt components and the undeducted contributions add to more than interest_value: they are parts of the notional payment, whose post-June 83 component is what remains (former ITAA 1936 s 27AA(1)(e)).",
        );
    }
    Ok(c)
}

/// Former s 27AA(1)(d): the lesser of the base x pre-July 83 days / total
/// days and the base less undeducted contributions, to the cent.
fn pre_july_83_steps(c: &Components, [total, before]: &[Decimal; 2]) -> Result<[Step; 2]> {
    let base = c
        .value
        .sub(&c.concessional)?
        .sub(&c.invalidity)?
        .sub(&c.cgt)?;
    let by_days = base.mul(before)?.div(total)?;
    let less_undeducted = base.sub(&c.undeducted)?;
    // Both are at least nil: the components cannot exceed the value.
    let pre = cents(super::py_min(by_days, less_undeducted)?)?;
    Ok([
        step(
            "pre_july_83_base",
            "Former ITAA 1936 s 27AA(1)(d)(i): ETP - C - IC - NQ - EC - CGT, with NQ and EC nil",
            "interest_value - concessional_component - post_june_1994_invalidity_component - cgt_exempt_component",
            &[&c.value, &c.concessional, &c.invalidity, &c.cgt],
            base.clone(),
        ),
        step(
            "pre_july_83_component",
            "Former ITAA 1936 s 27AA(1)(d): the lesser of (i) the base x pre-July 83 days / total days and (ii) the base reduced by the undeducted contributions, to the cent",
            "cents(min(pre_july_83_base x pre_july_1983_days / service_days, pre_july_83_base - undeducted_contributions))",
            &[&base, before, total, &base, &c.undeducted],
            pre,
        ),
    ])
}

/// s 307-225(2): the crystallised segment, and the taxable component left.
fn segment_steps(c: &Components, pre: &Decimal) -> Result<[Step; 2]> {
    let segment = c
        .concessional
        .add(&c.invalidity)?
        .add(&c.undeducted)?
        .add(&c.cgt)?
        .add(pre)?;
    let taxable = c.value.sub(&segment)?;
    Ok([
        step(
            "crystallised_segment",
            "ITAA 1997 s 307-225(2): the concessional component, post-June 1994 invalidity component, undeducted contributions, CGT exempt component and pre-July 83 component",
            "concessional_component + post_june_1994_invalidity_component + undeducted_contributions + cgt_exempt_component + pre_july_83_component",
            &[&c.concessional, &c.invalidity, &c.undeducted, &c.cgt, pre],
            segment.clone(),
        ),
        step(
            "taxable_component",
            "ITAA 1997 ss 307-210 and 307-215: the value less the tax free component, which at 1 July 2007 is the crystallised segment",
            "interest_value - crystallised_segment",
            &[&c.value, &segment],
            taxable,
        ),
    ])
}

pub fn super_crystallisation(args: &SuperCrystallisationArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    refused_facts(args)?;
    let start = date_arg(args.eligible_service_start, "eligible_service_start")?;
    let (total, before) = service_days(start)?;
    let c = components(args)?;
    let days = [int(total), int(before)];
    let steps = precise(|| {
        let [base, pre] = pre_july_83_steps(&c, &days)?;
        let rest = segment_steps(&c, &pre.value)?;
        let mut steps = vec![base, pre];
        steps.extend(rest);
        Ok(steps)
    })?;
    let rates = vec![
        rate("service_days", total),
        rate("pre_july_1983_days", before),
        rate("post_june_1983_days", total - before),
    ];
    derived_result(KIND, &year, steps, rates)
}
