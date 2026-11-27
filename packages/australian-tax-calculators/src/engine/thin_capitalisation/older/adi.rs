// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Subdivisions 820-D and 820-E: ADIs by their minimum capital amount.

use super::super::inputs::{Class, Deduction, fact, not_used, optional_amount, required};
use super::super::{ThinCapitalisationArgs, Worked, max0};
use super::{all_or_none, dd_step, not_worked_out, only, proportionate, stated_flag, whole_year};
use crate::decimal::Decimal;
use crate::engine::{Result, Step, d, precise, rate, step};

const WORLD: [&str; 4] = [
    "worldwide_capital_risk_weighted_assets",
    "worldwide_capital_tier_1_prudential_capital_deductions",
    "group_average_tier_1_capital",
    "group_average_risk_weighted_assets",
];

/// Whether an outward ADI is foreign controlled, and its worldwide figures
/// when stated.
type World = (bool, Option<Vec<Decimal>>);

fn uses(outward: bool) -> Vec<&'static str> {
    let mut uses = vec![
        "whole_year",
        "average_debt_capital",
        "average_adi_equity_capital",
        "average_risk_weighted_assets",
        "arm_length_capital_amount",
    ];
    if outward {
        uses.extend([
            "average_controlled_foreign_entity_equity",
            "average_tier_1_prudential_capital_deductions",
            "foreign_controlled_australian_entity",
        ]);
        uses.extend(WORLD);
    } else {
        uses.push("average_interest_free_pe_loans");
    }
    uses
}

fn world(a: &ThinCapitalisationArgs) -> Result<World> {
    let controlled = stated_flag(
        a,
        "foreign_controlled_australian_entity",
        "the worldwide capital amount is not available to a foreign controlled Australian entity (s 820-320(1))",
    );
    let controlled = controlled?;
    let w = all_or_none(a, &WORLD, "the worldwide capital amount uses all of them")?;
    if controlled && w.is_some() {
        return not_used(
            "worldwide_capital_risk_weighted_assets",
            "the worldwide capital amount does not apply to a foreign controlled Australian entity (s 820-320(1))",
        );
    }
    if let Some(w) = &w
        && !w[3].gt(&d("0"))?
    {
        return fact("group_average_risk_weighted_assets must be above nil (s 820-320(3)).");
    }
    Ok((controlled, w))
}

/// An ADI's minimum capital amount test (Subdivision 820-D or 820-E).
pub(super) fn adi(a: &ThinCapitalisationArgs, c: Class, items: &[Deduction]) -> Result<Worked> {
    let outward = c == Class::OutwardAdi;
    only(
        a,
        &uses(outward),
        "it is not in the provisions for the ADI's class (ss 820-300 to 820-325 or 820-395 to 820-415)",
    )?;
    whole_year(a)?;
    let need = |key: &str| required(a, key, "the provision uses it; state 0.00 if none");
    let debt_capital = need("average_debt_capital")?;
    let equity = need("average_adi_equity_capital")?;
    let rwa = need("average_risk_weighted_assets")?;
    let arm = optional_amount(a, "arm_length_capital_amount")?;
    let (second, t1) = if outward {
        (
            need("average_controlled_foreign_entity_equity")?,
            Some(need("average_tier_1_prudential_capital_deductions")?),
        )
    } else {
        (need("average_interest_free_pe_loans")?, None)
    };
    let worldwide = if outward { Some(world(a)?) } else { None };
    precise(|| {
        let mut steps = vec![dd_step(items)?];
        let (eq_id, eq) = equity_capital(&mut steps, outward, &equity, &second)?;
        let sh = safe_harbour(&mut steps, &rwa, t1.as_ref())?;
        let (least, missing) = minimum(&mut steps, outward, sh, worldwide.as_ref(), arm.as_ref())?;
        let shortfall = max0(least.sub(&eq)?)?;
        if !shortfall.is_zero() && !missing.is_empty() {
            return not_worked_out(&format!(
                "The ADI's equity capital is below the amounts worked out, but the minimum capital amount is the least of them all and {} was not stated: a judgement this worksheet does not make, so the disallowed amount is not worked out.",
                missing.join(" and ")
            ));
        }
        let stepped = shortfall_steps(
            &mut steps,
            outward,
            eq_id,
            [&least, &eq, &shortfall],
            &debt_capital,
        );
        stepped?;
        Ok(Worked {
            steps,
            rates: vec![rate("safe_harbour_capital_rate", "0.06")],
            carried: None,
        })
    })
}

/// Adjusted average equity capital (outward) or average equity capital
/// (inward), with its step id.
fn equity_capital(
    steps: &mut Vec<Step>,
    outward: bool,
    equity: &Decimal,
    second: &Decimal,
) -> Result<(&'static str, Decimal)> {
    let (eq_id, eq_auth, eq_formula) = if outward {
        (
            "adjusted_average_equity_capital",
            "s 820-300(3): ADI equity capital less controlled foreign entity equity, neither attributable to overseas permanent establishments",
            "average_adi_equity_capital - average_controlled_foreign_entity_equity",
        )
    } else {
        (
            "average_equity_capital",
            "s 820-395(3): ADI equity capital of the Australian permanent establishments not allocated to OB activities, plus the loans to them that give rise to no debt deductions",
            "average_adi_equity_capital + average_interest_free_pe_loans",
        )
    };
    let eq = if outward {
        equity.sub(second)?
    } else {
        equity.add(second)?
    };
    steps.push(step(
        eq_id,
        eq_auth,
        eq_formula,
        &[equity, second],
        eq.clone(),
    ));
    Ok((eq_id, eq))
}

/// The safe harbour capital amount (ss 820-310(1) and 820-405).
fn safe_harbour(steps: &mut Vec<Step>, rwa: &Decimal, t1: Option<&Decimal>) -> Result<Decimal> {
    let six = d("0.06");
    let v = match t1 {
        Some(t1) => {
            let v = six.mul(rwa)?.add(t1)?;
            steps.push(step(
                "safe_harbour_capital_amount",
                "s 820-310(1): 6% of risk-weighted assets and capitalised software intangibles (step 1's exclusions applied), plus tier 1 prudential capital deductions (step 3's exclusions applied)",
                "safe_harbour_capital_rate x average_risk_weighted_assets + average_tier_1_prudential_capital_deductions",
                &[&six, rwa, t1],
                v.clone(),
            ));
            v
        }
        None => {
            let v = six.mul(rwa)?;
            steps.push(step(
                "safe_harbour_capital_amount",
                "s 820-405: 6% of the risk-weighted assets of the Australian permanent establishments not attributable to OB activities",
                "safe_harbour_capital_rate x average_risk_weighted_assets",
                &[&six, rwa],
                v.clone(),
            ));
            v
        }
    };
    Ok(v)
}

/// The worldwide capital amount (s 820-320).
fn worldwide_capital(steps: &mut Vec<Step>, w: &[Decimal]) -> Result<Decimal> {
    let ratio = w[2].div(&w[3])?;
    steps.push(step(
        "worldwide_group_capital_ratio",
        "s 820-320(3): the group's average tier 1 capital / its average risk-weighted assets, under the prudential standards",
        "group_average_tier_1_capital / group_average_risk_weighted_assets",
        &[&w[2], &w[3]],
        ratio.clone(),
    ));
    let v = w[0].mul(&ratio)?.add(&w[1])?;
    steps.push(step(
        "worldwide_capital_amount",
        "s 820-320(2): risk-weighted assets (step 1's exclusions applied) x the worldwide group capital ratio, plus tier 1 prudential capital deductions (step 4's exclusions applied)",
        "worldwide_capital_risk_weighted_assets x worldwide_group_capital_ratio + worldwide_capital_tier_1_prudential_capital_deductions",
        &[&w[0], &ratio, &w[1]],
        v.clone(),
    ));
    Ok(v)
}

/// The minimum capital amount, the least of the amounts worked out, and the
/// amounts it is the least of that were not stated.
fn minimum(
    steps: &mut Vec<Step>,
    outward: bool,
    sh: Decimal,
    worldwide: Option<&World>,
    arm: Option<&Decimal>,
) -> Result<(Decimal, Vec<&'static str>)> {
    let mut amounts = vec![("safe_harbour_capital_amount", sh)];
    let mut missing: Vec<&str> = Vec::new();
    match worldwide {
        Some((_, Some(w))) => {
            amounts.push(("worldwide_capital_amount", worldwide_capital(steps, w)?))
        }
        Some((false, None)) => missing.push("the worldwide capital amount (s 820-320)"),
        _ => {}
    }
    match arm {
        Some(v) => amounts.push(("arm_length_capital_amount", v.clone())),
        None if outward => missing.push("the arm's length capital amount (s 820-315)"),
        None => missing.push("the arm's length capital amount (s 820-410)"),
    }
    let mut least = amounts[0].1.clone();
    for (_, v) in &amounts[1..] {
        if v.lt(&least)? {
            least = v.clone();
        }
    }
    let formula = amounts
        .iter()
        .skip(1)
        .fold(amounts[0].0.to_string(), |f, (n, _)| {
            format!("min({f}, {n})")
        });
    let refs: Vec<&Decimal> = amounts.iter().map(|(_, v)| v).collect();
    steps.push(step(
        "minimum_capital_amount",
        if outward {
            "s 820-305: the least of the safe harbour, arm's length and (unless a foreign controlled Australian entity) worldwide capital amounts"
        } else {
            "s 820-400: the lesser of the safe harbour and arm's length capital amounts"
        },
        &formula,
        &refs,
        least.clone(),
    ));
    Ok((least, missing))
}

/// The capital shortfall, average debt and the disallowed amount (ss 820-325
/// and 820-415).
fn shortfall_steps(
    steps: &mut Vec<Step>,
    outward: bool,
    eq_id: &str,
    amounts: [&Decimal; 3],
    debt_capital: &Decimal,
) -> Result<()> {
    let [least, eq, shortfall] = amounts;
    steps.push(step(
        "capital_shortfall",
        if outward {
            "s 820-325: the amount by which adjusted average equity capital is less than the minimum capital amount"
        } else {
            "s 820-415: the amount by which average equity capital is less than the minimum capital amount"
        },
        &format!("max(minimum_capital_amount - {eq_id}, 0)"),
        &[least, eq],
        shortfall.clone(),
    ));
    steps.push(step(
        "average_debt",
        if outward {
            "s 820-325: average debt capital giving rise to debt deductions, other than that attributable to overseas permanent establishments"
        } else {
            "s 820-415: average debt capital giving rise to debt deductions other than allowable OB deductions"
        },
        "average_debt_capital",
        &[debt_capital],
        debt_capital.clone(),
    ));
    proportionate(
        steps,
        "capital_shortfall",
        if outward {
            "s 820-325: each debt deduction x capital shortfall / average debt"
        } else {
            "s 820-415: each debt deduction (other than an allowable OB deduction) x capital shortfall / average debt"
        },
    )
}
