// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Schedule 12 withholding from a superannuation lump sum: the components,
//! treatment and caps of the tax worksheet, at Table A's rates, plus the
//! roll-over and no-TFN rules.

use serde_json::{Value, json};

use super::income_stream::{STREAM_OPTIONAL, income_stream};
use super::parts::{self, Parts};
use super::{
    Components, SUPER_BENEFIT_OPTIONAL, SuperBenefitTaxArgs, Treatment, components, payment_day,
    refuse_pre_1983_service, treatment, when,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{
    Arg, Result, Step, d, derived_result, flag, int, py_min, rate, refuse, scope, step,
};

const KIND: &str = "super_withholding";

/// The facts as the command passes them: the tax worksheet's, with the
/// payee's TFN and residency.
pub struct SuperWithholdingArgs<'a> {
    pub base: SuperBenefitTaxArgs<'a>,
    pub tfn_provided: &'a Arg,
    pub foreign_resident: &'a Arg,
    /// [`STREAM_OPTIONAL`] in order: given only for an income stream payment.
    pub stream: [Option<&'a Arg>; 4],
}

const TABLE_A: &str = "Withholding Schedules, Schedule 12, Table A: the rate for the part, which includes the 2% Medicare levy (a foreign resident's rate excludes it); nil where the tax offset's ceiling rate is nil (the ATO's Schedule 12 from 13 October 2020 to 30 June 2024; Schedule 12 of F2024L00664, F2025L00647 and F2026L00716)";
const ROUNDED: &str = "Schedule 12, 'Rounding of withholding amounts': the amount to withhold to the nearest dollar, 50 cents up, with no preliminary rounding to the cent";
const NIL: &str = "Schedule 12, 'Payments not subject to PAYG withholding' and Table A: nothing is withheld from a terminal medical condition payment, a member benefit under $200 that is the payee's entire benefit, a death benefit to a dependant or a death benefit to the trustee of a deceased estate";
const ROLLOVER: &str = "Schedule 12, 'Rollovers': nothing is withheld from the element untaxed in the fund up to the untaxed plan cap, and 47% from the amount above it";
const ROLLOVER_NO_TFN: &str = "Schedule 12, 'Rollovers' and 'Rounding of withholding amounts': 47% from the element untaxed in the fund above the untaxed plan cap, ignoring cents where no TFN was provided";
const NO_TFN_BASE: &str = "Schedule 12, 'When a TFN has not been provided': the taxable component under 60, or the element untaxed in the fund at 60 or over, ignoring cents";
const NO_TFN: &str = "Schedule 12, 'When a TFN has not been provided': 47% for an Australian resident or 45% for a foreign resident, ignoring cents, with no tax offsets or Medicare levy adjustments";
const ABOVE_PLAN: &str = "Schedule 12, 'Rollovers', with ITAA 1997 s 307-350: the element untaxed in the fund above the untaxed plan cap amount left";
const AGED_60_NIL: &str = "Schedule 12, Table A: nothing is withheld from the element taxed in the fund of a member benefit at 60 and above, or from the tax free component";
const FIRST_DAY: &str = "2020-10-13";
const RECIPIENTS: &str = "recipient must be member, death_benefits_dependant, non_dependant, rollover or deceased_estate_trustee.";

/// A result's steps after the components, and its rates.
type Tail = (Vec<Step>, Vec<(String, String)>);

/// A treatment, the member's age and preservation age, and the result's tail.
type Worked = (Treatment, Option<(i64, i64)>, Tail);

/// The single step for a payment nothing is withheld from.
fn nil() -> Tail {
    (vec![step("withholding", NIL, "0", &[], int(0))], vec![])
}

/// Each inapplicable optional input refused, with `why` naming who takes it.
fn none_of(args: &SuperBenefitTaxArgs, keys: &[usize], why: &str) -> Result<()> {
    for &i in keys {
        when(false, args.optional[i], SUPER_BENEFIT_OPTIONAL[i], why)?;
    }
    Ok(())
}

/// The amount to withhold, rounded, from the parts' withholding steps.
fn total(ids: &[String], values: &[Decimal]) -> Result<Step> {
    let mut sum = int(0);
    for value in values {
        sum = sum.add(value)?;
    }
    let formula = format!("dollars({})", ids.join(" + "));
    let operands: Vec<&Decimal> = values.iter().collect();
    let rounded = sum.quantize(0, Rounding::HalfUp)?;
    Ok(step("withholding", ROUNDED, &formula, &operands, rounded))
}

/// A member's roll-over: 47% of the element untaxed in the fund above the
/// untaxed plan cap left, the rest nil.
fn rollover(args: &SuperBenefitTaxArgs, c: &Components, year: &str, tfn: bool) -> Result<Tail> {
    // The 'Rollovers' rule states one rate whatever the TFN or residency;
    // without a TFN, cents are ignored ('Rounding of withholding amounts').
    none_of(
        args,
        &[0, 1, 2, 4, 6],
        "for a member benefit (recipient member)",
    )?;
    refuse_pre_1983_service(args, c)?;
    let mut p = Parts::default();
    let Some(left) = parts::plan_left(c, year, args.optional[5], &mut p)? else {
        return Ok(nil_rollover());
    };
    let untaxed = &c.untaxed.value;
    let above = untaxed.sub(&py_min(untaxed.clone(), left.clone())?)?;
    let formula = "untaxed_element - min(untaxed_element, untaxed_plan_cap_left)";
    p.steps.push(step(
        "untaxed_above_plan_cap",
        ABOVE_PLAN,
        formula,
        &[untaxed, untaxed, &left],
        above.clone(),
    ));
    let r = d("0.47");
    p.rates.push(rate("rollover_rate", &r));
    let (formula, authority, withheld) = if tfn {
        let withheld = above.mul(&r)?.quantize(0, Rounding::HalfUp)?;
        (
            "dollars(untaxed_above_plan_cap x rollover_rate)",
            ROLLOVER,
            withheld,
        )
    } else {
        let used = above.quantize(0, Rounding::Down)?;
        let withheld = used.mul(&r)?.quantize(0, Rounding::Down)?;
        let formula = "floor(floor(untaxed_above_plan_cap) x rollover_rate)";
        (formula, ROLLOVER_NO_TFN, withheld)
    };
    p.steps.push(step(
        "withholding",
        authority,
        formula,
        &[&above, &r],
        withheld,
    ));
    Ok((p.steps, p.rates))
}

/// A roll-over with no element untaxed in the fund.
fn nil_rollover() -> Tail {
    (
        vec![step("withholding", ROLLOVER, "0", &[], int(0))],
        vec![],
    )
}

/// No TFN: 47% or 45% of the amount the payee's age sets, cents ignored.
fn no_tfn(t: Treatment, c: &Components, foreign: bool) -> Result<Tail> {
    let (name, base) = match t {
        Treatment::Aged60 => ("untaxed_element", &c.untaxed.value),
        Treatment::PreservationTo59 | Treatment::UnderPreservation => {
            ("taxable_component", &c.taxable.value)
        }
        Treatment::TerminalMedicalCondition | Treatment::Dependant => return Ok(nil()),
        Treatment::SmallClosingBalance | Treatment::NonDependant => {
            return refuse(
                "super_withholding.no_tfn_unstated",
                "Schedule 12's rates without a TFN turn on the payee's age, which the command does not take for a death benefit, and it states none for a member benefit under $200.",
            );
        }
    };
    let used = base.quantize(0, Rounding::Down)?;
    let r = d(if foreign { "0.45" } else { "0.47" });
    let withheld = used.mul(&r)?.quantize(0, Rounding::Down)?;
    let steps = vec![
        step(
            "withheld_from",
            NO_TFN_BASE,
            &format!("floor({name})"),
            &[base],
            used.clone(),
        ),
        step(
            "withholding",
            NO_TFN,
            "floor(withheld_from x no_tfn_rate)",
            &[&used, &r],
            withheld,
        ),
    ];
    Ok((steps, vec![rate("no_tfn_rate", &r)]))
}

/// With a TFN: each part of the tax worksheet at Table A's rate, the tax
/// offset's ceiling rate plus the Medicare levy for a resident.
fn table_a(p: Parts, foreign: bool) -> Result<Tail> {
    if p.shares.is_empty() {
        return Ok((
            vec![step("withholding", AGED_60_NIL, "0", &[], int(0))],
            vec![],
        ));
    }
    let levy = d(if foreign { "0" } else { "0.02" });
    let mut steps: Vec<Step> = p
        .steps
        .into_iter()
        .filter(|s| !s.id.ends_with("_tax"))
        .collect();
    let mut rates: Vec<(String, String)> = p
        .rates
        .into_iter()
        .filter(|(k, _)| !k.ends_with("_rate"))
        .collect();
    let (mut ids, mut values) = (vec![], vec![]);
    for (id, amount, ceiling) in &p.shares {
        let ceiling = d(ceiling);
        let r = if ceiling.is_zero() {
            ceiling
        } else {
            ceiling.add(&levy)?
        };
        let name = format!("{id}_withholding");
        rates.push(rate(&format!("{name}_rate"), &r));
        let value = amount.mul(&r)?;
        let formula = format!("{id} x {name}_rate");
        steps.push(step(
            name.clone(),
            TABLE_A,
            &formula,
            &[amount, &r],
            value.clone(),
        ));
        ids.push(name);
        values.push(value);
    }
    steps.push(total(&ids, &values)?);
    Ok((steps, rates))
}

/// A member's, dependant's or non-dependant's lump sum.
fn by_treatment(
    args: &SuperBenefitTaxArgs,
    c: &Components,
    year: &str,
    paid: i64,
    (tfn, foreign): (bool, bool),
) -> Result<Worked> {
    let (t, ages) = treatment(args, c, paid)?;
    refuse_pre_1983_service(args, c)?;
    let tail = if !tfn {
        none_of(args, &[4, 5, 6], "with a TFN (tfn_provided true)")?;
        no_tfn(t, c, foreign)?
    } else if t.wholly_free().is_some() {
        // Nil whatever the caps: the cap inputs do not bear on it.
        none_of(args, &[4, 5, 6], "for a benefit taxed by age")?;
        nil()
    } else {
        table_a(parts::parts(t, c, year, args)?, foreign)?
    };
    Ok((t, ages, tail))
}

pub fn super_withholding(args: &SuperWithholdingArgs) -> Result<Value> {
    let b = &args.base;
    let year = scope(b.scope_confirmed, b.year, KIND)?;
    let paid = payment_day(b, &year)?;
    if paid < dates::parse(FIRST_DAY).expect("date") {
        return refuse(
            "super_withholding.schedule_not_read",
            "Schedule 12 as it stood before 13 October 2020 was not read for this command; payment_date must be on or after 13 October 2020.",
        );
    }
    let tfn = flag(args.tfn_provided, "tfn_provided")?;
    let foreign = flag(args.foreign_resident, "foreign_resident")?;
    if args.stream[0].is_some() {
        return income_stream(b, &args.stream, (&year, paid), (tfn, foreign));
    }
    for (key, given) in STREAM_OPTIONAL.iter().zip(args.stream) {
        when(
            false,
            given,
            key,
            "for an income stream (with income_stream_period)",
        )?;
    }
    let recipient = b.recipient.text();
    let known = [
        "member",
        "death_benefits_dependant",
        "non_dependant",
        "rollover",
        "deceased_estate_trustee",
    ];
    if !recipient.is_some_and(|r| known.contains(&r)) {
        return refuse("super_withholding.recipient_invalid", RECIPIENTS);
    }
    let c = components(b, false)?;
    let (name, ages, (tail, mut rates)) = match recipient {
        Some("rollover") => ("rollover", None, rollover(b, &c, &year, tfn)?),
        Some("deceased_estate_trustee") => {
            none_of(
                b,
                &[0, 1, 2, 3, 4, 5, 6],
                "for a member benefit or a roll-over",
            )?;
            ("deceased_estate_trustee", None, nil())
        }
        _ => {
            let (t, ages, tail) = by_treatment(b, &c, &year, paid, (tfn, foreign))?;
            (t.name(), ages, tail)
        }
    };
    let mut steps = vec![c.tax_free, c.taxable, c.taxed, c.untaxed];
    steps.extend(tail);
    if let Some((age, preservation)) = ages {
        rates.push(rate("age", age));
        rates.push(rate("preservation_age", preservation));
    }
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["treatment"] = json!(name);
    Ok(out)
}
