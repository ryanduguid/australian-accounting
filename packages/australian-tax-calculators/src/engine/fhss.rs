// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `fhss`: the First Home Super Saver scheme's maximum release amount and the
//! tax on its release. From a list of contributions it counts the eligible
//! ones against the limits (TAA 1953 Sch 1 ss 138-30 and 138-35), works out the
//! releasable contributions amount and the associated earnings (s 138-40) at
//! the shortfall interest charge rate (s 280-105(2)); from a determination's
//! stated components it starts there. Either way it gives the assessable FHSS
//! released amount (ITAA 1997 s 313-20), the withholding (TAA Regulations 2017
//! reg 53A), the offset (s 313-25) and the FHSS tax if the money is neither
//! used nor recontributed (s 313-60). Provisions, the Commissioner's guidance
//! note GN 2024/1 and the SIC rate table were read on 20 November 2026.

mod count;
mod earnings;
mod input;

use serde_json::{Value, json};

use super::{Arg, Result, d, finite_between, flag, money, precise, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use input::{day, fy_label, fy_of, item_money, only_fields};

const KIND: &str = "fhss";

/// The arguments as the command passes them; see the README for each.
pub struct FhssArgs<'a> {
    pub determination_date: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub contributions: Option<&'a Arg>,
    pub determination_requested: Option<&'a Arg>,
    pub personal_deductions: Option<&'a Arg>,
    pub determination: Option<&'a Arg>,
    pub amount_requested: Option<&'a Arg>,
    pub estimated_marginal_rate: Option<&'a Arg>,
    pub commissioner_cannot_estimate: Option<&'a Arg>,
}

struct Components {
    concessional: Decimal,
    non_concessional: Decimal,
    earnings: Decimal,
}

fn from_determination(given: &Arg) -> Result<Components> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "fhss.determination_invalid",
            "determination must be an object with concessional, non_concessional and associated_earnings.",
        );
    };
    let keys = ["concessional", "non_concessional", "associated_earnings"];
    only_fields(fields, &keys, "determination")?;
    let [concessional, non_concessional, earnings] =
        keys.map(|key| item_money(fields, key, "determination"));
    Ok(Components {
        concessional: concessional?,
        non_concessional: non_concessional?,
        earnings: earnings?,
    })
}

const ESTIMATE: &str = "estimate: associated_earnings follows s 138-40 on the counted amount (85% of a concessional contribution, as s 138-30(1)(b) counts it) at the SIC rates the ATO publishes, from the start of the contribution's month to the determination day, rounded down for each contribution. The Commissioner's determination is the figure that counts: GN 2024/1 example 6 (Jill) states $5,190, which neither this reading nor the others tried reproduces, so this is an estimate.";
const STATED: &str = "stated: the components are the determination's own, as given.";

pub fn fhss(args: &FhssArgs) -> Result<Value> {
    let period = scope(args.scope_confirmed, args.year, KIND)?;
    let determined = day(args.determination_date, "determination_date")?;
    if fy_label(fy_of(determined)) != period {
        return refuse(
            "fhss.year_mismatch",
            format!("determination_date must fall in year, the financial year {period}."),
        );
    }
    // The supported years start with 2018-19, when determinations began.
    if determined > dates::parse("2026-12-31").expect("date") {
        return refuse(
            "fhss.rate_unpublished",
            "determination_date must be no later than 31 December 2026, the last quarter whose SIC rate the ATO had published when this was built.",
        );
    }
    let (components, detail, basis) = match (args.contributions, args.determination) {
        (Some(_), None) => {
            let (c, detail) = precise(|| earnings::from_contributions(args, determined))?;
            (c, Some(detail), ESTIMATE)
        }
        (None, Some(given)) => {
            for (name, extra) in [
                ("determination_requested", args.determination_requested),
                ("personal_deductions", args.personal_deductions),
            ] {
                if extra.is_some() {
                    return refuse(
                        "input.field_unknown",
                        format!("{name} is an input only with contributions."),
                    );
                }
            }
            (from_determination(given)?, None, STATED)
        }
        _ => {
            return refuse(
                "fhss.basis_invalid",
                "Give either contributions (to estimate the determination) or determination (its stated components), not both or neither.",
            );
        }
    };
    precise(|| release(args, determined, &period, components, detail, basis))
}

const WITHHELD_ESTIMATE: &str = "estimate: reg 53A(1)(a) withholds the tax the Commissioner estimates on the assessable FHSS released amount; this applies GN 2024/1 section 7's account of it, the estimated marginal rate plus 2% Medicare levy less the 30% offset, not below nil, rounded down to the dollar.";
const WITHHELD_17: &str = "reg_53A_1_b: the Commissioner cannot estimate the tax, as stated, so 17% of the assessable FHSS released amount is withheld (reg 53A(1)(b)), rounded down to the dollar.";

/// The new regime's first day: Treasury Laws Amendment (2023 Measures No. 3)
/// Act 2023 Sch 4 commenced on 15 September 2024.
const SCHEDULE_4_START: &str = "2024-09-15";
const TRANSITIONAL: &str = "The determination was made before 15 September 2024. A release after an action under item 30 of Schedule 4 to the Treasury Laws Amendment (2023 Measures No. 3) Act 2023 (amending that determination, its request or its release authority by 14 September 2027 after an unsuccessful attempt) is outside Subdivisions 313-C to 313-E, so fhss_tax_if_not_used and recontribution_required do not apply to it; item 28(4) also gives 14 days, not 90, in s 313-35(1)(c)(i).";

/// reg 53A, with the basis: the Commissioner's estimate, worked as GN 2024/1
/// section 7 gives it, or 17% where he cannot estimate (reg 53A(1)(b)). One
/// of the 2 must be given: a missing marginal rate does not show that he
/// cannot estimate.
fn withholding_rate(
    marginal: Option<&Arg>,
    cannot_estimate: Option<&Arg>,
) -> Result<(Decimal, &'static str)> {
    let cannot = match cannot_estimate {
        Some(arg) => flag(arg, "commissioner_cannot_estimate")?,
        None => false,
    };
    let arg = match (marginal, cannot) {
        (Some(_), true) => {
            return refuse(
                "input.field_unknown",
                "Give estimated_marginal_rate or commissioner_cannot_estimate true, not both.",
            );
        }
        (None, true) => return Ok((d("0.17"), WITHHELD_17)),
        (Some(arg), false) => arg,
        (None, false) => {
            return refuse(
                "input.missing_field",
                "estimated_marginal_rate is required unless commissioner_cannot_estimate is true: reg 53A(1)(b)'s 17% applies only where the Commissioner cannot estimate the tax.",
            );
        }
    };
    let Some(marginal) = finite_between(arg, "0", "0.45")? else {
        return refuse(
            "input.fraction_invalid",
            "estimated_marginal_rate must be a finite Decimal fraction from 0 to 0.45.",
        );
    };
    let r = marginal.add(&d("0.02"))?.sub(&d("0.30"))?;
    let r = if r.lt(&Decimal::from_int(0))? {
        Decimal::from_int(0)
    } else {
        r
    };
    Ok((r, WITHHELD_ESTIMATE))
}

/// s 313-20: the concessional and earnings components, less any shortfall
/// of the amount released below the maximum, not below nil.
fn assessable(c: &Components, maximum: &Decimal, requested: &Decimal) -> Result<Decimal> {
    let taxed = c.concessional.add(&c.earnings)?;
    let assessable = taxed.sub(&maximum.sub(requested)?)?;
    Ok(if assessable.lt(&Decimal::from_int(0))? {
        Decimal::from_int(0)
    } else {
        assessable
    })
}

/// The amount requested: the maximum release amount unless given, and never
/// more (TAA 1953 Sch 1 s 131-10(1), table item 4).
fn requested(given: Option<&Arg>, maximum: &Decimal) -> Result<Decimal> {
    let requested = match given {
        Some(arg) => money(arg)?,
        None => maximum.clone(),
    };
    if requested.gt(maximum)? {
        return refuse(
            "fhss.request_exceeds_maximum",
            "amount_requested is more than the FHSS maximum release amount (TAA 1953 Sch 1 s 131-10(1), table item 4).",
        );
    }
    Ok(requested)
}

fn rates(withholding_rate: &Decimal) -> Vec<(String, String)> {
    vec![
        rate("annual_limit", "15000"),
        rate("concessional_share", "0.85"),
        rate("withholding_rate", withholding_rate.to_string()),
        rate("fhss_offset_rate", "0.30"),
        rate("fhss_tax_rate", "0.20"),
    ]
}

fn release(
    args: &FhssArgs,
    determined: i64,
    period: &str,
    c: Components,
    detail: Option<Value>,
    basis: &str,
) -> Result<Value> {
    let maximum = c.concessional.add(&c.non_concessional)?.add(&c.earnings)?;
    let requested = requested(args.amount_requested, &maximum)?;
    let assessable = assessable(&c, &maximum, &requested)?;
    let (withholding_rate, withholding_basis) = withholding_rate(
        args.estimated_marginal_rate,
        args.commissioner_cannot_estimate,
    )?;
    let withheld = withholding_rate
        .mul(&assessable)?
        .quantize(0, Rounding::Down)?;
    let paid = requested.sub(&withheld)?;
    let offset = d("0.30").mul(&assessable)?;
    let fhss_tax = d("0.20").mul(&assessable)?;
    let recontribute = assessable.sub(&withheld)?;
    let amounts = vec![
        ("concessional_component", c.concessional),
        ("non_concessional_component", c.non_concessional),
        ("associated_earnings", c.earnings),
        ("maximum_release_amount", maximum),
        ("amount_requested", requested),
        ("assessable_fhss_released_amount", assessable),
        ("tax_withheld", withheld),
        ("paid_to_you", paid),
        ("fhss_tax_offset", offset),
        ("fhss_tax_if_not_used", fhss_tax),
        ("recontribution_required", recontribute),
    ];
    let mut out = result(KIND, period, amounts, rates(&withholding_rate))?;
    out["associated_earnings_basis"] = json!(basis.split(':').next().expect("a basis"));
    out["basis"] = json!(basis);
    out["withholding_basis"] = json!(withholding_basis.split(':').next().expect("a basis"));
    out["withholding_note"] = json!(withholding_basis);
    if determined < dates::parse(SCHEDULE_4_START).expect("date") {
        out["transitional_note"] = json!(TRANSITIONAL);
    }
    if let Some(Value::Object(fields)) = detail {
        for (key, value) in fields {
            out[key] = value;
        }
    }
    Ok(out)
}
