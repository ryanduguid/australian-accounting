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
    Arg, Result, Step, cents, date_arg, derived_result, flag, int, money, preservation_age, rate,
    refuse, scope, step,
};
use crate::dates;
use crate::decimal::Decimal;

mod income_stream;
mod parts;
mod withholding;

pub use income_stream::STREAM_OPTIONAL;
pub use withholding::{SuperWithholdingArgs, super_withholding};

const KIND: &str = "super_benefit_tax";

/// The inputs only a member benefit, or only some, call for.
pub const SUPER_BENEFIT_OPTIONAL: [&str; 7] = [
    "date_of_birth",
    "terminal_medical_condition",
    "disability_superannuation_benefit",
    "service_before_july_1983",
    "low_rate_cap_used",
    "untaxed_plan_cap_used",
    "other_lump_sums_this_year",
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
    pub optional: [Option<&'a Arg>; 7],
}

const TAX_FREE: &str = "ITAA 1997 s 307-125(2) and (3)(b): the tax free component bears the same proportion to the benefit as the interest's tax free component bears to its value just before the benefit is paid; the Act states no rounding, so this worksheet rounds it to the cent (0.5 cents up)";
const TAX_FREE_STREAM: &str = "ITAA 1997 s 307-125(2) and (3)(a): the tax free component of an income stream payment bears the same proportion to the payment as the interest's tax free component bore to its value when the stream commenced; the Act states no rounding, so this worksheet rounds it to the cent (0.5 cents up)";
const TAXABLE: &str =
    "ITAA 1997 ss 307-120(2) and 307-125(2): the taxable component is the rest of the benefit";
const TAXED: &str = "ITAA 1997 s 307-275(1) and (2): the taxable component is an element taxed in the fund except as ss 307-280 to 307-290 make part of it untaxed, as the fund states it";
const ASSESSABLE: &str = "ITAA 1997 ss 301-20(1), 301-35(1), 301-95(1), 301-105(1), 301-115(1) and 302-145(1): the parts included in assessable income";
const MAXIMUM: &str = "The sum of each part's tax at the rate its offset ensures is not exceeded: the most income tax the benefit can bear before Medicare levy, which the person's marginal rates can make lower; the ATO's tables add the 2% Medicare levy to each rate";

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

    /// The provision that makes the parts other than the assessable amount
    /// not assessable income and not exempt income.
    fn rest_free(self) -> &'static str {
        match self {
            Self::Aged60 => {
                "ITAA 1997 s 301-10: at 60 or over, the lump sum other than the element untaxed in the fund (s 301-90(a)) is not assessable income and not exempt income"
            }
            Self::PreservationTo59 => {
                "ITAA 1997 s 301-15: the tax free component is not assessable income and not exempt income"
            }
            Self::UnderPreservation => {
                "ITAA 1997 s 301-30: the tax free component is not assessable income and not exempt income"
            }
            Self::NonDependant => {
                "ITAA 1997 s 302-140: the tax free component is not assessable income and not exempt income"
            }
            _ => self.wholly_free().expect("wholly free"),
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

/// The amounts given, refused when nil or inconsistent: benefit, interest
/// value, the interest's tax free component and the element untaxed in the
/// fund. An income stream payment is not limited by the interest's value
/// when the stream commenced.
fn amounts(args: &SuperBenefitTaxArgs, stream: bool) -> Result<[Decimal; 4]> {
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
    if !stream && benefit.gt(&interest)? {
        return conflict(
            "benefit cannot exceed interest_value, the value of the interest it is paid from.",
        );
    }
    if interest_free.gt(&interest)? {
        return conflict("interest_tax_free_component cannot exceed interest_value.");
    }
    Ok([benefit, interest, interest_free, untaxed])
}

/// The components of a lump sum, or with `stream` of an income stream payment.
fn components(args: &SuperBenefitTaxArgs, stream: bool) -> Result<Components> {
    let [benefit, interest, interest_free, untaxed] = amounts(args, stream)?;
    let free = cents(benefit.mul(&interest_free)?.div(&interest)?)?;
    let formula = "cents(benefit x interest_tax_free_component / interest_value)";
    let operands = [&benefit, &interest_free, &interest];
    let authority = if stream { TAX_FREE_STREAM } else { TAX_FREE };
    let tax_free = step("tax_free_component", authority, formula, &operands, free);
    let taxable_value = benefit.sub(&tax_free.value)?;
    let formula = "benefit - tax_free_component";
    let operands = [&benefit, &tax_free.value];
    let taxable = step(
        "taxable_component",
        TAXABLE,
        formula,
        &operands,
        taxable_value,
    );
    if untaxed.gt(&taxable.value)? {
        return conflict(
            "untaxed_element cannot exceed the taxable component the proportioning rule gives.",
        );
    }
    let formula = "taxable_component - untaxed_element";
    let taxed_value = taxable.value.sub(&untaxed)?;
    let taxed = step(
        "taxed_element",
        TAXED,
        formula,
        &[&taxable.value, &untaxed],
        taxed_value,
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

/// Whether the recipient is the member, refused when not a known recipient.
fn is_member(args: &SuperBenefitTaxArgs) -> Result<bool> {
    match args.recipient.text() {
        Some("member") => Ok(true),
        Some("death_benefits_dependant" | "non_dependant") => Ok(false),
        _ => refuse(
            "super_benefit_tax.recipient_invalid",
            "recipient must be member, death_benefits_dependant or non_dependant.",
        ),
    }
}

/// A member's treatment from the facts, age and preservation age.
fn band(c: &Components, terminal: bool, age: i64, preservation: i64) -> Result<Treatment> {
    Ok(if terminal {
        Treatment::TerminalMedicalCondition
    } else if c.benefit.lt(&int(200))? && c.benefit.equals(&c.interest)? {
        Treatment::SmallClosingBalance
    } else if age >= 60 {
        Treatment::Aged60
    } else if age >= preservation {
        Treatment::PreservationTo59
    } else {
        Treatment::UnderPreservation
    })
}

/// The recipient's treatment, with the member's age and preservation age.
fn treatment(
    args: &SuperBenefitTaxArgs,
    c: &Components,
    paid: i64,
) -> Result<(Treatment, Option<(i64, i64)>)> {
    let [born, terminal, disability, ..] = args.optional;
    let member = is_member(args)?;
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
    let t = band(
        c,
        flag(terminal, "terminal_medical_condition")?,
        age,
        preservation,
    );
    let t = t?;
    Ok((t, Some((age, preservation))))
}

/// The amount that is assessable and the formula naming it: the taxable
/// component, or for a member 60 or over only the element untaxed in the
/// fund, or nothing.
fn assessable(t: Treatment, c: &Components) -> (&'static str, Decimal) {
    match t {
        Treatment::Aged60 => ("untaxed_element", c.untaxed.value.clone()),
        Treatment::PreservationTo59 | Treatment::UnderPreservation | Treatment::NonDependant => {
            ("taxable_component", c.taxable.value.clone())
        }
        _ => ("0", int(0)),
    }
}

/// The payment date, refused outside the income year.
fn payment_day(args: &SuperBenefitTaxArgs, year: &str) -> Result<i64> {
    let paid = date_arg(args.payment_date, "payment_date")?;
    let first: i64 = year[..4].parse().expect("income year");
    let start = dates::parse(&format!("{first}-07-01")).expect("date");
    let end = dates::parse(&format!("{}-06-30", first + 1)).expect("date");
    if paid < start || paid > end {
        return conflict(&format!(
            "payment_date must fall in the {year} income year."
        ));
    }
    Ok(paid)
}

/// s 307-150's modification, which the command does not take, refused.
fn refuse_pre_1983_service(args: &SuperBenefitTaxArgs, c: &Components) -> Result<()> {
    let service = args.optional[3];
    let why = "with an element untaxed in the fund";
    if let Some(service) = when(
        !c.untaxed.value.is_zero(),
        service,
        "service_before_july_1983",
        why,
    )? && flag(service, "service_before_july_1983")?
    {
        return refuse(
            "super_benefit_tax.pre_july_1983_service",
            "A lump sum with an element untaxed in the fund attributable to an interest that existed before 1 July 2007 and service before 1 July 1983 moves part of that element to the tax free component under ITAA 1997 s 307-150, which this command does not take.",
        );
    }
    Ok(())
}

/// The closing steps: the assessable amount, the rest and the most tax.
fn totals(t: Treatment, c: &Components, ids: &[String], tax: &[Decimal]) -> Result<[Step; 3]> {
    let (name, assessable) = assessable(t, c);
    let operands = if t.wholly_free().is_some() {
        vec![]
    } else {
        vec![&assessable]
    };
    let assessable_step = step(
        "assessable_amount",
        ASSESSABLE,
        name,
        &operands,
        assessable.clone(),
    );
    let free = c.benefit.sub(&assessable)?;
    let authority = t.rest_free();
    let rest = step(
        "non_assessable_non_exempt",
        authority,
        "benefit - assessable_amount",
        &[&c.benefit, &assessable],
        free,
    );
    let mut maximum = int(0);
    for value in tax {
        maximum = maximum.add(value)?;
    }
    let formula = if ids.is_empty() {
        "0".to_string()
    } else {
        ids.join(" + ")
    };
    let values: Vec<&Decimal> = tax.iter().collect();
    Ok([
        assessable_step,
        rest,
        step("maximum_tax", MAXIMUM, &formula, &values, maximum),
    ])
}

pub fn super_benefit_tax(args: &SuperBenefitTaxArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let paid = payment_day(args, &year)?;
    let c = components(args, false)?;
    let (t, ages) = treatment(args, &c, paid)?;
    refuse_pre_1983_service(args, &c)?;
    let p = parts::parts(t, &c, &year, args)?;
    let ids: Vec<String> = p
        .steps
        .iter()
        .map(|s| s.id.to_string())
        .filter(|id| id.ends_with("_tax"))
        .collect();
    let totals = totals(t, &c, &ids, &p.tax)?;
    let mut steps = vec![c.tax_free, c.taxable, c.taxed, c.untaxed];
    steps.extend(p.steps);
    steps.extend(totals);
    let mut rates = p.rates;
    if let Some((age, preservation)) = ages {
        rates.push(rate("age", age));
        rates.push(rate("preservation_age", preservation));
    }
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["treatment"] = json!(t.name());
    Ok(out)
}
