// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `super_benefit_tax`: one superannuation lump sum from a complying plan,
//! its components by the proportioning rule and the most income tax, before
//! Medicare levy, each part can bear. The citations are to ITAA 1997
//! Divisions 301, 302, 303 and 307 from the ATO's legal database, the ATO's
//! 'Payments from super' rates and caps and Schedule 12 of the withholding
//! schedules, all read on 22 November 2026.

use serde_json::{Value, json};

use super::{
    Arg, Result, Step, cents, d, date_arg, derived_result, flag, int, lookup, money,
    preservation_age, py_min, rate, refuse, scope, step,
};
use crate::dates;
use crate::decimal::Decimal;
use crate::metadata as meta;

const KIND: &str = "super_benefit_tax";

/// The inputs only a member benefit, or only some, call for.
pub const SUPER_BENEFIT_OPTIONAL: [&str; 6] = [
    "date_of_birth",
    "terminal_medical_condition",
    "disability_superannuation_benefit",
    "service_before_july_1983",
    "low_rate_cap_used",
    "untaxed_plan_cap_used",
];

/// The facts as the command passes them; see the README for each.
pub struct SuperBenefitTaxArgs<'a> {
    pub benefit: &'a Arg,
    pub interest_value: &'a Arg,
    pub interest_tax_free_component: &'a Arg,
    pub untaxed_element: &'a Arg,
    pub recipient: &'a Arg,
    pub payment_date: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    /// [`SUPER_BENEFIT_OPTIONAL`] in order.
    pub optional: [Option<&'a Arg>; 6],
}

const TAX_FREE: &str = "ITAA 1997 s 307-125(2) and (3)(b): the tax free component bears the same proportion to the benefit as the interest's tax free component bears to its value just before the benefit is paid; the Act states no rounding, so this worksheet rounds it to the cent (0.5 cents up)";
const TAXABLE: &str =
    "ITAA 1997 ss 307-120(2) and 307-125(2): the taxable component is the rest of the benefit";
const TAXED: &str = "ITAA 1997 s 307-275(1) and (2): the taxable component is an element taxed in the fund except as ss 307-280 to 307-290 make part of it untaxed, as the fund states it";
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
const NANE: &str = "the parts the Act makes not assessable income and not exempt income";
const ASSESSABLE: &str = "ITAA 1997 ss 301-20(1), 301-35(1), 301-95(1), 301-105(1), 301-115(1) and 302-145(1): the parts included in assessable income";
const MAXIMUM: &str = "The sum of each part's tax at the rate its offset ensures is not exceeded: the most income tax the benefit can bear before Medicare levy, which the person's marginal rates can make lower; the ATO's tables add the 2% Medicare levy to each rate";
const LOW_COUNTED: &str = "ITAA 1997 s 307-345(2): the amounts counted towards the s 301-20(2) and 301-105(4) offsets, which reduce the low rate cap amount for later lump sums";
const PLAN_COUNTED: &str = "ITAA 1997 s 307-350(2): the element untaxed in the fund, up to the untaxed plan cap amount left, which reduces that amount for later lump sums from the plan";

/// How the benefit is taxed: the recipient, and for a member the age band
/// or the rule that makes the whole benefit tax free.
#[derive(Clone, Copy, PartialEq)]
enum Treatment {
    Aged60,
    PreservationTo59,
    UnderPreservation,
    Dependant,
    NonDependant,
    TerminalMedicalCondition,
    SmallClosingBalance,
}

impl Treatment {
    fn name(self) -> &'static str {
        match self {
            Self::Aged60 => "aged_60_or_over",
            Self::PreservationTo59 => "preservation_age_to_59",
            Self::UnderPreservation => "under_preservation_age",
            Self::Dependant => "death_benefits_dependant",
            Self::NonDependant => "non_dependant",
            Self::TerminalMedicalCondition => "terminal_medical_condition",
            Self::SmallClosingBalance => "under_200_closing_the_interest",
        }
    }

    /// The provision that makes the whole benefit not assessable income and
    /// not exempt income, if one does.
    fn wholly_free(self) -> Option<&'static str> {
        match self {
            Self::Dependant => Some(
                "ITAA 1997 s 302-60: a lump sum to a death benefits dependant is not assessable income and not exempt income",
            ),
            Self::TerminalMedicalCondition => Some(
                "ITAA 1997 s 303-10(2): a member lump sum received with a terminal medical condition existing then or within 90 days after is not assessable income and not exempt income",
            ),
            Self::SmallClosingBalance => Some(
                "ITAA 1997 s 301-225(1): a member lump sum under $200 that leaves the interest's value nil is not assessable income and not exempt income",
            ),
            _ => None,
        }
    }
}

fn conflict<T>(message: &str) -> Result<T> {
    refuse("super_benefit_tax.fact_conflict", message)
}

/// `given` when `needed`, refused when given and not needed.
fn when<'a>(needed: bool, given: Option<&'a Arg>, key: &str, why: &str) -> Result<Option<&'a Arg>> {
    match (needed, given) {
        (true, None) => refuse("input.missing_field", format!("{key} is required {why}.")),
        (false, Some(_)) => refuse(
            "input.field_unknown",
            format!("{key} is an input only {why}."),
        ),
        (_, given) => Ok(given),
    }
}

/// Completed years from `born` to `day`; a 29 February birthday falls on
/// 1 March in a common year.
fn age_on(born: i64, day: i64) -> i64 {
    let (by, bm, bd) = dates::to_civil(born);
    let (y, m, dd) = dates::to_civil(day);
    y - by - i64::from((m, dd) < (bm, bd))
}

/// The components: benefit, tax free, taxable, taxed and untaxed elements.
struct Components {
    benefit: Decimal,
    interest: Decimal,
    tax_free: Step,
    taxable: Step,
    taxed: Step,
    untaxed: Step,
}

fn components(args: &SuperBenefitTaxArgs) -> Result<Components> {
    let benefit = money(args.benefit)?;
    let interest = money(args.interest_value)?;
    let interest_free = money(args.interest_tax_free_component)?;
    let untaxed = money(args.untaxed_element)?;
    if benefit.is_zero() || interest.is_zero() {
        return refuse(
            "super_benefit_tax.benefit_nil",
            "The benefit and the value of the superannuation interest must each be more than nil.",
        );
    }
    if benefit.gt(&interest)? {
        return conflict(
            "benefit cannot exceed interest_value, the value of the interest it is paid from.",
        );
    }
    if interest_free.gt(&interest)? {
        return conflict("interest_tax_free_component cannot exceed interest_value.");
    }
    let free = cents(benefit.mul(&interest_free)?.div(&interest)?)?;
    let tax_free = step(
        "tax_free_component",
        TAX_FREE,
        "cents(benefit x interest_tax_free_component / interest_value)",
        &[&benefit, &interest_free, &interest],
        free,
    );
    let taxable = step(
        "taxable_component",
        TAXABLE,
        "benefit - tax_free_component",
        &[&benefit, &tax_free.value],
        benefit.sub(&tax_free.value)?,
    );
    if untaxed.gt(&taxable.value)? {
        return conflict(
            "untaxed_element cannot exceed the taxable component the proportioning rule gives.",
        );
    }
    let taxed = step(
        "taxed_element",
        TAXED,
        "taxable_component - untaxed_element",
        &[&taxable.value, &untaxed],
        taxable.value.sub(&untaxed)?,
    );
    let untaxed = step(
        "untaxed_element",
        TAXED,
        "untaxed_element",
        &[&untaxed],
        untaxed.clone(),
    );
    Ok(Components {
        benefit,
        interest,
        tax_free,
        taxable,
        taxed,
        untaxed,
    })
}

/// The recipient's treatment, with the member's age and preservation age.
fn treatment(
    args: &SuperBenefitTaxArgs,
    c: &Components,
    paid: i64,
) -> Result<(Treatment, Option<(i64, i64)>)> {
    let [born, terminal, disability, ..] = args.optional;
    let member = match args.recipient.text() {
        Some("member") => true,
        Some("death_benefits_dependant" | "non_dependant") => false,
        _ => {
            return refuse(
                "super_benefit_tax.recipient_invalid",
                "recipient must be member, death_benefits_dependant or non_dependant.",
            );
        }
    };
    let why = "for a member benefit (recipient member)";
    let born = when(member, born, "date_of_birth", why)?;
    let terminal = when(member, terminal, "terminal_medical_condition", why)?;
    let disability = when(member, disability, "disability_superannuation_benefit", why)?;
    let (Some(born), Some(terminal), Some(disability)) = (born, terminal, disability) else {
        let t = if args.recipient.text() == Some("non_dependant") {
            Treatment::NonDependant
        } else {
            Treatment::Dependant
        };
        return Ok((t, None));
    };
    if flag(disability, "disability_superannuation_benefit")? {
        return refuse(
            "super_benefit_tax.disability_benefit",
            "A disability superannuation benefit's tax free component is increased under ITAA 1997 s 307-145 from the service days and days to retirement, which this command does not take.",
        );
    }
    let born = date_arg(born, "date_of_birth")?;
    if born >= paid {
        return conflict("date_of_birth must be before payment_date.");
    }
    let age = age_on(born, paid);
    let preservation = preservation_age(dates::to_civil(born));
    let t = if flag(terminal, "terminal_medical_condition")? {
        Treatment::TerminalMedicalCondition
    } else if c.benefit.lt(&int(200))? && c.benefit.equals(&c.interest)? {
        Treatment::SmallClosingBalance
    } else if age >= 60 {
        Treatment::Aged60
    } else if age >= preservation {
        Treatment::PreservationTo59
    } else {
        Treatment::UnderPreservation
    };
    Ok((t, Some((age, preservation))))
}

/// A cap left: the year's cap less what earlier lump sums counted.
fn cap_left(
    id: &'static str,
    authority: &'static str,
    cap: &str,
    used: &Arg,
    key: &str,
) -> Result<Step> {
    let cap = d(cap);
    let used = money(used)?;
    if used.gt(&cap)? {
        return conflict(&format!("{key} cannot exceed the year's cap of {cap}."));
    }
    let formula = format!("{} - {key}", key.trim_end_matches("_used"));
    Ok(step(
        id,
        authority,
        &formula,
        &[&cap, &used],
        cap.sub(&used)?,
    ))
}

/// The worksheet's parts so far, with their tax and the rates.
#[derive(Default)]
struct Parts {
    steps: Vec<Step>,
    tax: Vec<Decimal>,
    rates: Vec<(String, String)>,
}

impl Parts {
    /// One part at a rate: its amount's step, then its tax's.
    fn part(&mut self, amount: Step, rate_text: &str) -> Result<()> {
        let id = amount.id.to_string();
        let r = d(rate_text);
        let tax = cents(amount.value.mul(&r)?)?;
        self.tax.push(tax.clone());
        self.rates.push(rate(&format!("{id}_rate"), rate_text));
        let tax_step = step(
            format!("{id}_tax"),
            amount.authority,
            &format!("cents({id} x {id}_rate)"),
            &[&amount.value, &r],
            tax,
        );
        self.steps.push(amount);
        self.steps.push(tax_step);
        Ok(())
    }
}

/// The untaxed plan cap left, when the benefit has an element untaxed in
/// the fund, pushed with its cap rate.
fn plan_left(
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
    p.rates.push(rate("untaxed_plan_cap", cap));
    let left = cap_left(
        "untaxed_plan_cap_left",
        PLAN_CAP_LEFT,
        cap,
        used,
        "untaxed_plan_cap_used",
    )?;
    let value = left.value.clone();
    p.steps.push(left);
    Ok(Some(value))
}

/// The element untaxed in the fund within the plan cap at `rate_text` (or,
/// for preservation age to 59, split at the low rate cap left), then above
/// it at the top rate. Returns the part within the plan cap and the part
/// counted against the low rate cap.
fn untaxed_parts(
    c: &Components,
    left: &Decimal,
    within_rule: (&'static str, &str),
    low_left: Option<(&Decimal, &Decimal)>,
    p: &mut Parts,
) -> Result<(Decimal, Decimal)> {
    let untaxed = &c.untaxed.value;
    let within = py_min(untaxed.clone(), left.clone())?;
    let mut counted = int(0);
    if let Some((low_cap_left, taxed_low)) = low_left {
        let room = low_cap_left.sub(taxed_low)?;
        counted = py_min(within.clone(), room)?;
        let s = step(
            "untaxed_within_low_rate_cap",
            UNTAXED_LOW,
            "min(min(untaxed_element, untaxed_plan_cap_left), low_rate_cap_left - taxed_within_low_rate_cap)",
            &[untaxed, left, low_cap_left, taxed_low],
            counted.clone(),
        );
        p.part(s, "0.15")?;
    }
    let s = step(
        "untaxed_within_plan_cap",
        within_rule.0,
        if low_left.is_some() {
            "min(untaxed_element, untaxed_plan_cap_left) - untaxed_within_low_rate_cap"
        } else {
            "min(untaxed_element, untaxed_plan_cap_left) - 0"
        },
        &[untaxed, left, &counted][..if low_left.is_some() { 3 } else { 2 }],
        within.sub(&counted)?,
    );
    p.part(s, within_rule.1)?;
    let s = step(
        "untaxed_above_plan_cap",
        UNTAXED_ABOVE_PLAN,
        "untaxed_element - min(untaxed_element, untaxed_plan_cap_left)",
        &[untaxed, untaxed, left],
        untaxed.sub(&within)?,
    );
    p.part(s, "0.45")?;
    Ok((within, counted))
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
    let low_used = when(
        t == Treatment::PreservationTo59,
        low_used,
        "low_rate_cap_used",
        why,
    )?;
    let plan = plan_left(c, year, plan_used, p)?;
    let taxed = &c.taxed.value;
    let mut low_counted = None;
    let within = match t {
        Treatment::Aged60 => match &plan {
            Some(left) => Some(untaxed_parts(c, left, (UNTAXED_60, "0.15"), None, p)?.0),
            None => None,
        },
        Treatment::UnderPreservation => {
            let s = step(
                "taxed_element_part",
                TAXED_UNDER,
                "taxed_element",
                &[taxed],
                taxed.clone(),
            );
            p.part(s, "0.20")?;
            match &plan {
                Some(left) => Some(untaxed_parts(c, left, (UNTAXED_UNDER, "0.30"), None, p)?.0),
                None => None,
            }
        }
        Treatment::PreservationTo59 => {
            // From 1 July 2024 the preservation age is 60 for everyone born
            // after 30 June 1964, and everyone born before is 60 or over, so
            // this band arises only in the years the table holds.
            let cap = lookup(&meta::SUPER_LOW_RATE_CAP, year);
            p.rates.push(rate("low_rate_cap", cap));
            let used = low_used.expect("required above");
            let left = cap_left(
                "low_rate_cap_left",
                LOW_CAP_LEFT,
                cap,
                used,
                "low_rate_cap_used",
            )?;
            let low_left = left.value.clone();
            p.steps.push(left);
            let low = py_min(taxed.clone(), low_left.clone())?;
            let s = step(
                "taxed_within_low_rate_cap",
                TAXED_LOW,
                "min(taxed_element, low_rate_cap_left)",
                &[taxed, &low_left],
                low.clone(),
            );
            p.part(s, "0")?;
            let s = step(
                "taxed_above_low_rate_cap",
                TAXED_ABOVE_LOW,
                "taxed_element - taxed_within_low_rate_cap",
                &[taxed, &low],
                taxed.sub(&low)?,
            );
            p.part(s, "0.15")?;
            let (within, untaxed_low) = match &plan {
                Some(left) => {
                    let (w, u) = untaxed_parts(
                        c,
                        left,
                        (UNTAXED_ABOVE_LOW, "0.30"),
                        Some((&low_left, &low)),
                        p,
                    )?;
                    (Some(w), u)
                }
                None => (None, int(0)),
            };
            low_counted = Some((low, untaxed_low));
            within
        }
        _ => unreachable!("member bands only"),
    };
    if let Some((taxed_low, untaxed_low)) = low_counted {
        let s = step(
            "low_rate_cap_counted",
            LOW_COUNTED,
            if plan.is_some() {
                "taxed_within_low_rate_cap + untaxed_within_low_rate_cap"
            } else {
                "taxed_within_low_rate_cap + 0"
            },
            &[&taxed_low, &untaxed_low][..if plan.is_some() { 2 } else { 1 }],
            taxed_low.add(&untaxed_low)?,
        );
        p.steps.push(s);
    }
    if let (Some(within), Some(left)) = (within, plan) {
        let s = step(
            "untaxed_plan_cap_counted",
            PLAN_COUNTED,
            "min(untaxed_element, untaxed_plan_cap_left)",
            &[&c.untaxed.value, &left],
            within,
        );
        p.steps.push(s);
    }
    Ok(())
}

/// The parts and their tax, by treatment.
fn parts(t: Treatment, c: &Components, year: &str, args: &SuperBenefitTaxArgs) -> Result<Parts> {
    let mut p = Parts::default();
    let [.., low_used, plan_used] = args.optional;
    let caps_why = "for a member benefit taxed by age (not wholly tax free)";
    let member_taxed = matches!(
        t,
        Treatment::Aged60 | Treatment::PreservationTo59 | Treatment::UnderPreservation
    );
    if !member_taxed {
        when(false, low_used, "low_rate_cap_used", caps_why)?;
        when(false, plan_used, "untaxed_plan_cap_used", caps_why)?;
    }
    match t {
        Treatment::NonDependant => {
            let s = step(
                "taxed_element_part",
                TAXED_NON_DEPENDANT,
                "taxed_element",
                &[&c.taxed.value],
                c.taxed.value.clone(),
            );
            p.part(s, "0.15")?;
            let s = step(
                "untaxed_element_part",
                UNTAXED_NON_DEPENDANT,
                "untaxed_element",
                &[&c.untaxed.value],
                c.untaxed.value.clone(),
            );
            p.part(s, "0.30")?;
        }
        t if t.wholly_free().is_some() => {}
        _ => member_parts(t, c, year, args, &mut p)?,
    }
    Ok(p)
}

/// The amounts that are assessable: the taxable component, or for a member
/// 60 or over only the element untaxed in the fund, or nothing.
fn assessable(t: Treatment, c: &Components) -> Decimal {
    match t {
        Treatment::Aged60 => c.untaxed.value.clone(),
        Treatment::PreservationTo59 | Treatment::UnderPreservation | Treatment::NonDependant => {
            c.taxable.value.clone()
        }
        _ => int(0),
    }
}

pub fn super_benefit_tax(args: &SuperBenefitTaxArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let paid = date_arg(args.payment_date, "payment_date")?;
    let first: i64 = year[..4].parse().expect("income year");
    let start = dates::parse(&format!("{first}-07-01")).expect("date");
    let end = dates::parse(&format!("{}-06-30", first + 1)).expect("date");
    if paid < start || paid > end {
        return conflict(&format!(
            "payment_date must fall in the {year} income year."
        ));
    }
    let c = components(args)?;
    let (t, ages) = treatment(args, &c, paid)?;
    let [.., service, _, _] = args.optional;
    let has_untaxed = !c.untaxed.value.is_zero();
    let why = "with an element untaxed in the fund";
    if let Some(service) = when(has_untaxed, service, "service_before_july_1983", why)?
        && flag(service, "service_before_july_1983")?
    {
        return refuse(
            "super_benefit_tax.pre_july_1983_service",
            "A lump sum with an element untaxed in the fund attributable to an interest that existed before 1 July 2007 and service before 1 July 1983 moves part of that element to the tax free component under ITAA 1997 s 307-150, which this command does not take.",
        );
    }
    let p = parts(t, &c, &year, args)?;
    let assessable = assessable(t, &c);
    let free = c.benefit.sub(&assessable)?;
    let mut maximum = int(0);
    for tax in &p.tax {
        maximum = maximum.add(tax)?;
    }
    let tax_ids: Vec<String> = p
        .steps
        .iter()
        .map(|s| s.id.to_string())
        .filter(|id| id.ends_with("_tax"))
        .collect();
    let formula = if tax_ids.is_empty() {
        "0".to_string()
    } else {
        tax_ids.join(" + ")
    };
    let nane_authority = t.wholly_free().unwrap_or(NANE);
    let mut steps = vec![c.tax_free, c.taxable, c.taxed, c.untaxed];
    steps.extend(p.steps);
    steps.push(step(
        "assessable_amount",
        ASSESSABLE,
        match t {
            Treatment::Aged60 => "untaxed_element",
            Treatment::PreservationTo59
            | Treatment::UnderPreservation
            | Treatment::NonDependant => "taxable_component",
            _ => "0",
        },
        &if t.wholly_free().is_some() {
            vec![]
        } else {
            vec![&assessable]
        },
        assessable.clone(),
    ));
    steps.push(step(
        "non_assessable_non_exempt",
        nane_authority,
        "benefit - assessable_amount",
        &[&c.benefit, &assessable],
        free,
    ));
    let tax_refs: Vec<&Decimal> = p.tax.iter().collect();
    steps.push(step(
        "maximum_tax",
        MAXIMUM,
        &formula,
        &tax_refs,
        maximum.clone(),
    ));
    let mut rates = p.rates;
    if let Some((age, preservation)) = ages {
        rates.push(rate("age", age));
        rates.push(rate("preservation_age", preservation));
    }
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["treatment"] = json!(t.name());
    Ok(out)
}
