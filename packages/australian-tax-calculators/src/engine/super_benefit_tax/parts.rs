// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The parts of a member's or a non-dependant's lump sum, each at the rate
//! its offset ensures is not exceeded, with the caps they count against.

use super::{Components, SuperBenefitTaxArgs, Treatment, conflict, when};
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, Step, cents, d, int, lookup, money, py_min, rate, step};
use crate::metadata as meta;

const LOW_CAP_LEFT: &str = "ITAA 1997 s 307-345: the year's indexed low rate cap amount less the amounts counted towards the s 301-20(2) and 301-105(4) offsets for earlier lump sums (s 307-345(2) for earlier years; ss 301-20(3) and 301-105(5) within the year)";
const PLAN_CAP_LEFT: &str = "ITAA 1997 s 307-350: the year's indexed untaxed plan cap amount for the plan less the elements untaxed in the fund of earlier lump sums from it counted under s 307-350(2)";
const TAXED_LOW: &str = "ITAA 1997 s 301-20(2) and (3), with s 301-90(b): preservation age to under 60, the element taxed in the fund up to the low rate cap amount, at a rate not above 0%";
const TAXED_ABOVE_LOW: &str = "ITAA 1997 s 301-20(4) and (5), with s 301-90(b): preservation age to under 60, the element taxed in the fund above the low rate cap amount, at a rate not above 15%";
const TAXED_UNDER: &str = "ITAA 1997 s 301-35, with s 301-90(b): under preservation age, the element taxed in the fund at a rate not above 20%";
const TAXED_NON_DEPENDANT: &str = "ITAA 1997 s 302-145(1) and (2): a death benefit to a non-dependant, the element taxed in the fund at a rate not above 15%";
const UNTAXED_NON_DEPENDANT: &str = "ITAA 1997 s 302-145(1) and (3): a death benefit to a non-dependant, the element untaxed in the fund at a rate not above 30%";
const UNTAXED_60: &str = "ITAA 1997 s 301-95(2) and (3): 60 or over, the element untaxed in the fund up to the untaxed plan cap amount, at a rate not above 15%";
const UNTAXED_LOW: &str = "ITAA 1997 s 301-105(4) to (6): preservation age to under 60, the element untaxed in the fund within the untaxed plan cap and up to the low rate cap amount the element taxed in the fund left, at a rate not above 15%";
const UNTAXED_ABOVE_LOW: &str = "ITAA 1997 s 301-105(2) and (3): preservation age to under 60, the rest of the element untaxed in the fund within the untaxed plan cap amount, at a rate not above 30%";
const UNTAXED_UNDER: &str = "ITAA 1997 s 301-115(2) and (3): under preservation age, the element untaxed in the fund up to the untaxed plan cap amount, at a rate not above 30%";
const UNTAXED_ABOVE_PLAN: &str = "ITAA 1997 ss 301-95, 301-105 and 301-115, each section's note: the element untaxed in the fund above the untaxed plan cap amount has no offset and is taxed at the top marginal rate (45% in each year from 2020-21 to 2026-27, the ATO's resident tax rates)";
const LOW_COUNTED: &str = "ITAA 1997 s 307-345(2): the amounts counted towards the s 301-20(2) and 301-105(4) offsets, which reduce the low rate cap amount for later lump sums";
const PLAN_COUNTED: &str = "ITAA 1997 s 307-350(2): the element untaxed in the fund, up to the untaxed plan cap amount left, which reduces that amount for later lump sums from the plan";

/// The worksheet's parts so far, with their tax and the rates.
#[derive(Default)]
pub(super) struct Parts {
    pub(super) steps: Vec<Step>,
    pub(super) tax: Vec<Decimal>,
    /// Each part's id, amount and ceiling rate, for withholding.
    pub(super) shares: Vec<(String, Decimal, String)>,
    pub(super) rates: Vec<(String, String)>,
}

impl Parts {
    /// One part at a rate: its amount's step, then its tax's.
    fn part(&mut self, amount: Step, rate_text: &str) -> Result<()> {
        let id = amount.id.to_string();
        let r = d(rate_text);
        let tax = cents(amount.value.mul(&r)?)?;
        self.tax.push(tax.clone());
        let share = (id.clone(), amount.value.clone(), rate_text.to_string());
        self.shares.push(share);
        self.rates.push(rate(&format!("{id}_rate"), rate_text));
        let formula = format!("cents({id} x {id}_rate)");
        let tax_step = step(
            format!("{id}_tax"),
            amount.authority,
            &formula,
            &[&amount.value, &r],
            tax,
        );
        self.steps.push(amount);
        self.steps.push(tax_step);
        Ok(())
    }

    /// One part at a rate from its id, authority and formula.
    fn add(
        &mut self,
        head: (&'static str, &'static str, &str),
        operands: &[&Decimal],
        value: Decimal,
        rate_text: &str,
    ) -> Result<()> {
        let (id, authority, formula) = head;
        self.part(step(id, authority, formula, operands, value), rate_text)
    }

    /// A cap left, the year's cap less what earlier lump sums counted,
    /// pushed with the cap among the rates.
    fn cap_left(
        &mut self,
        (id, authority): (&'static str, &'static str),
        cap: &str,
        used: &Arg,
        key: &str,
    ) -> Result<Decimal> {
        let name = key.trim_end_matches("_used");
        self.rates.push(rate(name, cap));
        let cap = d(cap);
        let used = money(used)?;
        if used.gt(&cap)? {
            return conflict(&format!("{key} cannot exceed the year's cap of {cap}."));
        }
        let left = cap.sub(&used)?;
        self.steps.push(step(
            id,
            authority,
            &format!("{name} - {key}"),
            &[&cap, &used],
            left.clone(),
        ));
        Ok(left)
    }
}

/// The untaxed plan cap left, when the benefit has an element untaxed in the
/// fund.
pub(super) fn plan_left(
    c: &Components,
    year: &str,
    used: Option<&Arg>,
    p: &mut Parts,
) -> Result<Option<Decimal>> {
    let why = "for a member benefit with an element untaxed in the fund";
    let Some(used) = when(
        !c.untaxed.value.is_zero(),
        used,
        "untaxed_plan_cap_used",
        why,
    )?
    else {
        return Ok(None);
    };
    let cap = lookup(&meta::SUPER_UNTAXED_PLAN_CAP, year);
    p.cap_left(
        ("untaxed_plan_cap_left", PLAN_CAP_LEFT),
        cap,
        used,
        "untaxed_plan_cap_used",
    )
    .map(Some)
}

/// The element untaxed in the fund within the plan cap at `within_rule`'s
/// rate (first, for preservation age to 59, 15% on the low rate cap the
/// element taxed in the fund left), then above it at the top rate. Returns
/// the part within the plan cap and the part counted against the low rate cap.
fn untaxed_parts(
    c: &Components,
    left: &Decimal,
    within_rule: (&'static str, &str),
    low: Option<(&Decimal, &Decimal)>,
    p: &mut Parts,
) -> Result<(Decimal, Decimal)> {
    let untaxed = &c.untaxed.value;
    let within = py_min(untaxed.clone(), left.clone())?;
    let mut counted = int(0);
    let mut rest = "min(untaxed_element, untaxed_plan_cap_left) - 0";
    if let Some((low_left, taxed_low)) = low {
        counted = py_min(within.clone(), low_left.sub(taxed_low)?)?;
        let formula = "min(min(untaxed_element, untaxed_plan_cap_left), low_rate_cap_left - taxed_within_low_rate_cap)";
        let id = ("untaxed_within_low_rate_cap", UNTAXED_LOW, formula);
        p.add(
            id,
            &[untaxed, left, low_left, taxed_low],
            counted.clone(),
            "0.15",
        )?;
        rest = "min(untaxed_element, untaxed_plan_cap_left) - untaxed_within_low_rate_cap";
    }
    let operands = [untaxed, left, &counted];
    let operands = &operands[..if low.is_some() { 3 } else { 2 }];
    let id = ("untaxed_within_plan_cap", within_rule.0, rest);
    p.add(id, operands, within.sub(&counted)?, within_rule.1)?;
    let formula = "untaxed_element - min(untaxed_element, untaxed_plan_cap_left)";
    let id = ("untaxed_above_plan_cap", UNTAXED_ABOVE_PLAN, formula);
    p.add(id, &[untaxed, untaxed, left], untaxed.sub(&within)?, "0.45")?;
    Ok((within, counted))
}

/// The element taxed in the fund within and above the low rate cap left.
/// Returns the part within it.
fn taxed_low_parts(c: &Components, low_left: &Decimal, p: &mut Parts) -> Result<Decimal> {
    let taxed = &c.taxed.value;
    let low = py_min(taxed.clone(), low_left.clone())?;
    let id = (
        "taxed_within_low_rate_cap",
        TAXED_LOW,
        "min(taxed_element, low_rate_cap_left)",
    );
    p.add(id, &[taxed, low_left], low.clone(), "0")?;
    let formula = "taxed_element - taxed_within_low_rate_cap";
    let id = ("taxed_above_low_rate_cap", TAXED_ABOVE_LOW, formula);
    p.add(id, &[taxed, &low], taxed.sub(&low)?, "0.15")?;
    Ok(low)
}

/// Preservation age to under 60: the element taxed in the fund within and
/// above the low rate cap left, then the element untaxed in the fund.
/// Returns the untaxed part within the plan cap, if any.
fn preservation_parts(
    c: &Components,
    year: &str,
    used: &Arg,
    plan: Option<&Decimal>,
    p: &mut Parts,
) -> Result<Option<Decimal>> {
    // From 1 July 2024 the preservation age is 60 for everyone born after
    // 30 June 1964, and everyone born before is 60 or over, so this band
    // arises only in the years the table holds.
    let cap = lookup(&meta::SUPER_LOW_RATE_CAP, year);
    let id = ("low_rate_cap_left", LOW_CAP_LEFT);
    let low_left = p.cap_left(id, cap, used, "low_rate_cap_used")?;
    let low = taxed_low_parts(c, &low_left, p)?;
    let rule = (UNTAXED_ABOVE_LOW, "0.30");
    let untaxed = plan
        .map(|left| untaxed_parts(c, left, rule, Some((&low_left, &low)), p))
        .transpose()?;
    let counted = match &untaxed {
        Some((_, u)) => {
            let formula = "taxed_within_low_rate_cap + untaxed_within_low_rate_cap";
            step(
                "low_rate_cap_counted",
                LOW_COUNTED,
                formula,
                &[&low, u],
                low.add(u)?,
            )
        }
        None => {
            let formula = "taxed_within_low_rate_cap + 0";
            step(
                "low_rate_cap_counted",
                LOW_COUNTED,
                formula,
                &[&low],
                low.clone(),
            )
        }
    };
    p.steps.push(counted);
    Ok(untaxed.map(|(within, _)| within))
}

/// Under preservation age: the element taxed in the fund at 20%, then the
/// element untaxed in the fund. Returns the untaxed part within the plan cap.
fn under_preservation_parts(
    c: &Components,
    plan: Option<&Decimal>,
    p: &mut Parts,
) -> Result<Option<Decimal>> {
    let taxed = &c.taxed.value;
    let id = ("taxed_element_part", TAXED_UNDER, "taxed_element");
    p.add(id, &[taxed], taxed.clone(), "0.20")?;
    let rule = (UNTAXED_UNDER, "0.30");
    plan.map(|left| untaxed_parts(c, left, rule, None, p).map(|(w, _)| w))
        .transpose()
}

/// The member parts for the age band, with what they count against each cap.
fn member_parts(
    t: Treatment,
    c: &Components,
    year: &str,
    args: &SuperBenefitTaxArgs,
    p: &mut Parts,
) -> Result<()> {
    let [.., low_used, plan_used] = args.optional;
    let why = "for a member at or above preservation age and under 60";
    let needed = t == Treatment::PreservationTo59;
    let low_used = when(needed, low_used, "low_rate_cap_used", why)?;
    let plan = plan_left(c, year, plan_used, p)?;
    let within = match (t, &plan, low_used) {
        (Treatment::PreservationTo59, _, Some(used)) => {
            preservation_parts(c, year, used, plan.as_ref(), p)?
        }
        (Treatment::UnderPreservation, ..) => under_preservation_parts(c, plan.as_ref(), p)?,
        (_, Some(left), _) => Some(untaxed_parts(c, left, (UNTAXED_60, "0.15"), None, p)?.0),
        _ => None,
    };
    if let (Some(within), Some(left)) = (within, plan) {
        let formula = "min(untaxed_element, untaxed_plan_cap_left)";
        let operands = [&c.untaxed.value, &left];
        p.steps.push(step(
            "untaxed_plan_cap_counted",
            PLAN_COUNTED,
            formula,
            &operands,
            within,
        ));
    }
    Ok(())
}

/// The parts and their tax, by treatment.
pub(super) fn parts(
    t: Treatment,
    c: &Components,
    year: &str,
    args: &SuperBenefitTaxArgs,
) -> Result<Parts> {
    let mut p = Parts::default();
    match t {
        Treatment::NonDependant => {
            let (taxed, untaxed) = (&c.taxed.value, &c.untaxed.value);
            let id = ("taxed_element_part", TAXED_NON_DEPENDANT, "taxed_element");
            p.add(id, &[taxed], taxed.clone(), "0.15")?;
            let id = (
                "untaxed_element_part",
                UNTAXED_NON_DEPENDANT,
                "untaxed_element",
            );
            p.add(id, &[untaxed], untaxed.clone(), "0.30")?;
        }
        Treatment::Aged60 | Treatment::PreservationTo59 | Treatment::UnderPreservation => {
            member_parts(t, c, year, args, &mut p)?
        }
        _ => {}
    }
    if !matches!(
        t,
        Treatment::Aged60 | Treatment::PreservationTo59 | Treatment::UnderPreservation
    ) {
        let why = "for a member benefit taxed by age (not wholly tax free)";
        let [.., low_used, plan_used] = args.optional;
        when(false, low_used, "low_rate_cap_used", why)?;
        when(false, plan_used, "untaxed_plan_cap_used", why)?;
    }
    Ok(p)
}
