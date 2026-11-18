// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `balancing_adjustment`: the balancing adjustment when a balancing
//! adjustment event occurs for a depreciating asset used partly for a purpose
//! other than a taxable purpose, under ITAA 1997 Subdivision 40-D: the amount
//! assessable or deductible under s 40-285, reduced under s 40-290 (or, for a
//! car whose car expenses were claimed by both the log book and the cents per
//! kilometre methods, worked out under s 40-370 instead), the car limit
//! adjustment of s 40-325, and the capital gain or loss of CGT event K7
//! (ss 104-235 and 104-240), disregarded for a car or motor cycle (s 118-5).
//! The citations are to ITAA 1997 compilation 268 (1 October 2026), read on
//! 18 November 2026.

use serde_json::Value;

use super::business_capital::year_first;
use super::{Arg, Result, Step, derived_result, int, money, precise, rate, refuse, scope, step};
use crate::decimal::Decimal;

mod facts;

use facts::{Asset, Facts, Held, read_facts};

const KIND: &str = "balancing_adjustment";

/// A worksheet's steps and rates.
type Worked = (Vec<Step>, Vec<(String, String)>);

/// The decline since the start time, refusing one above cost or nil.
fn total_decline(facts: &Facts) -> Result<(Decimal, String)> {
    let mut total = int(0);
    for row in &facts.years {
        total = total.add(&row.decline)?;
    }
    if total.gt(&facts.cost)? {
        return refuse(
            "balancing.decline_above_cost",
            "The years' declines in value total more than cost, which would leave an adjustable value below nil (s 40-85).",
        );
    }
    if total.is_zero() {
        return refuse(
            "balancing.no_decline",
            "The years' declines in value total nil, so the fraction in s 40-290(2) and s 104-240 (sum of reductions over total decline) is undefined.",
        );
    }
    let terms: Vec<String> = facts
        .years
        .iter()
        .map(|row| row.decline.to_string())
        .collect();
    Ok((total, terms.join(" + ")))
}

/// s 40-370(4)(c): a cents per kilometre year counts its notional decline as
/// 20% for a taxable purpose, so 80% of it is a reduction.
fn reduction(row: &Held) -> Result<(Decimal, String)> {
    Ok(match &row.deduction {
        Some(deduction) => (
            row.decline.sub(deduction)?,
            format!("({} - {deduction})", row.decline),
        ),
        None => (
            row.decline.mul(&super::d("0.8"))?,
            format!("({} x (1 - 0.2))", row.decline),
        ),
    })
}

/// The sum of the s 40-25 reductions, with s 40-370(4)'s for cents per
/// kilometre years.
fn sum_of_reductions(facts: &Facts) -> Result<(Decimal, String)> {
    let mut sum = int(0);
    let mut terms = Vec::new();
    for row in &facts.years {
        let (value, term) = reduction(row)?;
        sum = sum.add(&value)?;
        terms.push(term);
    }
    Ok((sum, terms.join(" + ")))
}

const TV_GIVEN: &str = "ITAA 1997 s 40-300: the termination value, as the operator established it under ss 40-300 to 40-320 after Subdivision 27-B";
const TV_CAR_LIMIT: &str = "ITAA 1997 s 40-325: a car whose cost s 40-230 reduced to the car limit has its termination value times (car limit for the financial year first used + second element of cost) / total cost ignoring the car limit, after Subdivision 27-B";
const ADJUSTABLE: &str = "ITAA 1997 s 40-85(1)(b) and (c): cost less the decline in value since the start time, just before the event";
const TOTAL_DECLINE: &str = "ITAA 1997 s 40-290(2), total decline: the decline in value since you started to hold the asset, each year's";
const SUM_OF_REDUCTIONS: &str = "ITAA 1997 s 40-290(2), sum of reductions: each year's reduction of the deduction under s 40-25(2), the decline less the deduction; for a cents per kilometre year, s 40-370(4) assumes 20% taxable use of its notional decline";
const AMOUNT: &str = "ITAA 1997 s 40-285(1) and (2): the difference between the termination value and the adjustable value just before the event, assessable where termination value is more and deductible where it is less";
const REDUCED: &str = "ITAA 1997 s 40-290(2): the balancing adjustment amount x sum of reductions / total decline, which comes off the amount assessable or deductible";

/// The termination value the balancing adjustment uses: s 40-325's for a car
/// whose cost s 40-230 reduced.
fn termination(facts: &Facts, given: &Decimal) -> Result<Step> {
    Ok(match &facts.car_limit {
        None => step(
            "termination_value",
            TV_GIVEN,
            "termination_value",
            &[given],
            given.clone(),
        ),
        Some(car) => {
            let numerator = car.limit.add(&car.second_element)?;
            let value = given.mul(&numerator)?.div(&car.cost_ignoring_limit)?;
            // car_limit's facts are written in: a formula names only
            // top-level inputs.
            let formula = format!(
                "termination_value x ({} + {}) / {}",
                car.limit, car.second_element, car.cost_ignoring_limit
            );
            step("termination_value", TV_CAR_LIMIT, &formula, &[given], value)
        }
    })
}

/// How the amount is worked out once reduced: s 40-285 with s 40-290, or for
/// a car whose expenses were claimed by both car expense methods s 40-370.
enum Path {
    General,
    /// s 40-370: the days the decline was deducted and the days held.
    Switched(i64, i64),
    /// s 40-370(1) Note 2: only the cents per kilometre method was chosen.
    CentsOnly,
}

fn path(facts: &Facts) -> Result<Path> {
    if facts.asset != Asset::Car {
        return Ok(Path::General);
    }
    let held: i64 = facts.years.iter().filter_map(|row| row.days).sum();
    let logged: Vec<&Held> = facts
        .years
        .iter()
        .filter(|row| row.deduction.is_some())
        .collect();
    if logged.len() == facts.years.len() {
        return Ok(Path::General);
    }
    if logged.is_empty() {
        return Ok(Path::CentsOnly);
    }
    // Step 3 counts the days for which the decline was deducted, so a log
    // book year with a nil deduction adds none.
    let deducted: i64 = logged
        .iter()
        .filter(|row| row.deduction.as_ref().is_some_and(|d| !d.is_zero()))
        .filter_map(|row| row.days)
        .sum();
    Ok(Path::Switched(deducted, held))
}

const SWITCHED: &str = "ITAA 1997 s 40-370(2) steps 1 to 5: (termination value - adjustable value), reduced by the s 40-290(2) formula, x days the decline was deducted / days held";
const CENTS_ONLY: &str = "ITAA 1997 s 40-370(1) Note 2: only the cents per kilometre method was chosen, so no amount is assessable or deductible under s 40-370 or s 40-285";

/// The assessable or deductible amount, never below nil: `from - to`, reduced
/// for non-taxable use and, under s 40-370, scaled by the days, worked with
/// one division, last.
fn reduced_amount(
    id: &'static str,
    path: &Path,
    from: &Decimal,
    to: &Decimal,
    totals: [&Decimal; 2],
) -> Result<Step> {
    let [total, sum] = totals;
    let numerator = from.sub(to)?.mul(&total.sub(sum)?)?;
    let names = if id == "assessable_amount" {
        "termination_value - adjustable_value"
    } else {
        "adjustable_value - termination_value"
    };
    let formula = format!("max(0, ({names}) x (total_decline - sum_of_reductions)");
    Ok(match path {
        Path::CentsOnly => step(id, CENTS_ONLY, "0", &[], int(0)),
        Path::General => {
            let value = super::py_max(int(0), numerator.div(total)?)?;
            step(
                id,
                AMOUNT,
                &format!("{formula} / total_decline)"),
                &[from, to, total, sum, total],
                value,
            )
        }
        Path::Switched(deducted, held) => {
            let (deducted, held) = (int(*deducted), int(*held));
            let value = numerator.mul(&deducted)?.div(&total.mul(&held)?)?;
            let value = super::py_max(int(0), value)?;
            let formula = format!("{formula} x days_deducted / (total_decline x days_held))");
            step(
                id,
                SWITCHED,
                &formula,
                &[from, to, total, sum, &deducted, total, &held],
                value,
            )
        }
    })
}

const K7_GAIN: &str = "ITAA 1997 s 104-240(1): CGT event K7, (termination value - cost) x sum of reductions / total decline, where termination value exceeds cost; before any exemption, the discount and the CGT changes from 1 July 2027";
const K7_LOSS: &str = "ITAA 1997 s 104-240(2): CGT event K7, (cost - termination value) x sum of reductions / total decline, where cost exceeds termination value; before any exemption";
const K7_DISREGARDED: &str = "ITAA 1997 s 118-5(a): a capital gain or loss from a car, motor cycle or similar vehicle is disregarded";

fn k7(facts: &Facts, tv: &Decimal, total: &Decimal, sum: &Decimal) -> Result<[Step; 2]> {
    if facts.asset != Asset::Other {
        let nil = |id| step(id, K7_DISREGARDED, "0", &[], int(0));
        return Ok([nil("capital_gain"), nil("capital_loss")]);
    }
    let cost = &facts.cost;
    let gain = super::py_max(int(0), tv.sub(cost)?)?.mul(sum)?.div(total)?;
    let loss = super::py_max(int(0), cost.sub(tv)?)?.mul(sum)?.div(total)?;
    Ok([
        step(
            "capital_gain",
            K7_GAIN,
            "max(0, termination_value - cost) x sum_of_reductions / total_decline",
            &[tv, cost, sum, total],
            gain,
        ),
        step(
            "capital_loss",
            K7_LOSS,
            "max(0, cost - termination_value) x sum_of_reductions / total_decline",
            &[cost, tv, sum, total],
            loss,
        ),
    ])
}

/// The total decline, adjustable value and sum of reductions, with their
/// steps.
fn basis(facts: &Facts) -> Result<([Decimal; 3], [Step; 3])> {
    let (total, total_terms) = total_decline(facts)?;
    let (sum, sum_terms) = sum_of_reductions(facts)?;
    let adjustable = facts.cost.sub(&total)?;
    let steps = [
        step(
            "total_decline",
            TOTAL_DECLINE,
            &total_terms,
            &[],
            total.clone(),
        ),
        step(
            "adjustable_value",
            ADJUSTABLE,
            "cost - total_decline",
            &[&facts.cost, &total],
            adjustable.clone(),
        ),
        step(
            "sum_of_reductions",
            SUM_OF_REDUCTIONS,
            &sum_terms,
            &[],
            sum.clone(),
        ),
    ];
    Ok(([total, adjustable, sum], steps))
}

/// The rates: the years held, the provision applied and, under s 40-370, the
/// days.
fn rates(path: &Path, years_held: usize) -> Vec<(String, String)> {
    let mut rates = vec![rate("years_held", years_held)];
    let provision = match path {
        Path::General => "ITAA 1997 s 40-285 and s 40-290",
        Path::Switched(deducted, held) => {
            rates.push(rate("days_deducted", deducted));
            rates.push(rate("days_held", held));
            "ITAA 1997 s 40-370"
        }
        Path::CentsOnly => "ITAA 1997 s 40-370(1) Note 2",
    };
    rates.push(rate("provision", provision));
    rates
}

/// The worksheet's steps and rates from read facts and the termination value
/// worked out under ss 40-300 to 40-320.
fn worked(facts: &Facts, given_tv: &Decimal) -> Result<Worked> {
    let path = path(facts)?;
    precise(|| {
        let tv_step = termination(facts, given_tv)?;
        let tv = tv_step.value.clone();
        let ([total, adjustable, sum], basis_steps) = basis(facts)?;
        let amount = super::py_max(tv.sub(&adjustable)?, adjustable.sub(&tv)?)?;
        let reduced = amount.mul(&sum)?.div(&total)?;
        let totals = [&total, &sum];
        let mut steps = vec![tv_step];
        steps.extend(basis_steps);
        steps.extend([
            step(
                "balancing_adjustment_amount",
                AMOUNT,
                "max(termination_value - adjustable_value, adjustable_value - termination_value)",
                &[&tv, &adjustable, &adjustable, &tv],
                amount.clone(),
            ),
            step(
                "reduction_for_non_taxable_use",
                REDUCED,
                "balancing_adjustment_amount x sum_of_reductions / total_decline",
                &[&amount, &sum, &total],
                reduced,
            ),
            reduced_amount("assessable_amount", &path, &tv, &adjustable, totals)?,
            reduced_amount("deductible_amount", &path, &adjustable, &tv, totals)?,
        ]);
        steps.extend(k7(facts, &tv, &total, &sum)?);
        Ok((steps, rates(&path, facts.years.len())))
    })
}

#[allow(clippy::too_many_arguments)]
pub fn balancing_adjustment(
    cost: &Arg,
    termination_value: &Arg,
    asset: &Arg,
    years: &Arg,
    car_limit: Option<&Arg>,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let event = year_first(&year).expect("a supported income year");
    let given_tv = money(termination_value)?;
    let facts = read_facts(cost, asset, years, car_limit, event)?;
    let (steps, rates) = worked(&facts, &given_tv)?;
    derived_result(KIND, &year, steps, rates)
}
