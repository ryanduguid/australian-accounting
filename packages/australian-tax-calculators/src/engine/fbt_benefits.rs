// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The loan, meal entertainment and car parking worksheets: each benefit
//! category's taxable value, which `fbt` then grosses up with the others. They
//! take their inputs by name, since which inputs apply turns on the method, and
//! cite the FBTAA as compiled on 1 July 2026 (C2004A03280) and ATO pages, read
//! on 15 November 2026.

use serde_json::Value;

use super::{
    APPLIED, Arg, EXCESS, NET, Result, Step, cents, d, day_count, derived_result, fbt_year,
    finite_between, int, lookup, money, precise, py_min, rate, refuse, step, steps_from,
};
use crate::decimal::Decimal;
use crate::metadata as meta;

/// The methods `method` accepts for `kind`, in the catalogue's order.
pub fn methods(kind: &str) -> &'static [&'static str] {
    match kind {
        "fbt_meal_entertainment" => &["50_50_split", "12_week_register"],
        "fbt_car_parking" => &[
            "commercial_parking_station",
            "average_cost",
            "12_week_register",
        ],
        "fbt_residual" => &residual_housing::RESIDUAL_METHODS,
        "fbt_housing" => &residual_housing::HOUSING_METHODS,
        "fbt_lafha" => &residual_housing::LAFHA_METHODS,
        "fbt_tax_exempt_body_entertainment" => &residual_housing::ENTERTAINMENT_METHODS,
        _ => &[],
    }
}

/// `kind`'s method named by `value`.
fn method(value: &Arg, kind: &str) -> Result<&'static str> {
    let methods = methods(kind);
    if let Some(found) = value.text().and_then(|m| methods.iter().find(|x| **x == m)) {
        return Ok(found);
    }
    let (last, rest) = methods.split_last().expect("methods");
    refuse(
        "input.fbt_method_invalid",
        format!("method must be {} or {last}.", rest.join(", ")),
    )
}

/// The inputs `names` from those `given`, refusing any other input given and
/// any of `names` absent. `when` names the method the inputs depend on.
fn take<'a, const N: usize>(
    given: &'a [(String, Arg)],
    command: &str,
    when: &str,
    names: [&str; N],
) -> Result<[&'a Arg; N]> {
    if let Some((name, _)) = given.iter().find(|(n, _)| !names.contains(&n.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("{name} is not an input of {command}{when}"),
        );
    }
    let found = names.map(|name| given.iter().find(|(n, _)| n == name).map(|(_, arg)| arg));
    if let Some((name, _)) = names.iter().zip(&found).find(|(_, arg)| arg.is_none()) {
        return refuse("input.missing_field", format!("{name} is required{when}"));
    }
    Ok(found.map(|arg| arg.expect("checked present")))
}

/// A 12 week register is valid for the FBT year it was first valid for and
/// the 4 after it (`section`). Whether it has since ceased to be valid is the
/// operator's to establish.
fn register_year(value: &Arg, year: i64, section: &str) -> Result<()> {
    match value.int() {
        Some(Some(first)) if (year - 4..=year).contains(&first) => Ok(()),
        _ => refuse(
            "fbt.register_not_valid",
            format!(
                "register_first_year must be the FBT year, from {} to {year}, the register was first valid for: a register is valid for that year and the 4 after it (FBTAA {section}).",
                year - 4
            ),
        ),
    }
}

/// The loan worksheet's amounts: (id, authority, formula).
const LOAN_STEPS: [(&str, &str, &str); 8] = [
    (
        "notional_interest",
        "FBTAA s 18(1) and s 136(1), notional amount of interest; FBT rates and thresholds, statutory or benchmark interest rate",
        "cents(daily_balance_total x statutory_interest_rate / days_in_year)",
    ),
    (
        "interest_accrued_applied",
        "FBTAA s 18(1)",
        "min(interest_accrued, notional_interest)",
    ),
    (
        "gross_taxable_value",
        "FBTAA s 18(1)",
        "notional_interest - interest_accrued_applied",
    ),
    (
        "gross_deduction",
        "FBTAA s 19(1)(b), GD",
        "cents(notional_interest x income_use)",
    ),
    (
        "accrued_interest_deduction_applied",
        "FBTAA s 19(1)(ba), RD",
        "min(accrued_interest_deduction, gross_deduction)",
    ),
    (
        "notional_deduction",
        "FBTAA s 19(1)(ba), ND = GD - RD",
        "gross_deduction - accrued_interest_deduction_applied",
    ),
    (
        "reduction_applied",
        "FBTAA s 19(1); FBT return 2026 instructions, item 23C",
        "min(notional_deduction, gross_taxable_value)",
    ),
    (
        "taxable_value",
        "FBTAA s 19(1), TV - ND; FBT return 2026 instructions, item 23C",
        "gross_taxable_value - reduction_applied",
    ),
];

/// FBTAA ss 18 and 19: notional interest at the statutory interest rate on the
/// daily balances less the interest accrued, reduced by the notional deduction
/// the otherwise deductible rule allows, never below nil.
pub fn fbt_loan(given: &[(String, Arg)], year_ended: &Arg, scope_confirmed: &Arg) -> Result<Value> {
    let kind = "fbt_loan";
    let (year, period, days_in_year) = fbt_year(scope_confirmed, year_ended, kind)?;
    let names = [
        "daily_balance_total",
        "interest_accrued",
        "income_use",
        "accrued_interest_deduction",
    ];
    let [balance, accrued, income_use, deduction] = take(given, "fbt-loan", "", names)?;
    let (balance, accrued) = (money(balance)?, money(accrued)?);
    let Some(income_use) = finite_between(income_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "income_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    let deduction = money(deduction)?;
    if deduction.gt(&accrued)? {
        return refuse(
            "fbt_loan.deduction_exceeds_interest",
            "accrued_interest_deduction cannot exceed interest_accrued: it is the deduction for that interest (FBTAA s 19(1)(ba), RD).",
        );
    }
    let statutory = d(lookup(meta::FBT_BENCHMARK_INTEREST_RATES, &year));
    let inputs = [&balance, &accrued, &income_use, &deduction];
    let steps = precise(|| loan_steps(inputs, &statutory, &int(days_in_year)))?;
    let rates = vec![
        rate("statutory_interest_rate", &statutory),
        rate("days_in_year", days_in_year),
    ];
    derived_result(kind, &period, steps, rates)
}

/// [`LOAN_STEPS`] on the loan's daily balances, interest accrued, income use
/// and deduction for that interest, at `statutory` over `days`.
fn loan_steps(inputs: [&Decimal; 4], statutory: &Decimal, days: &Decimal) -> Result<Vec<Step>> {
    let [balance, accrued, income_use, deduction] = inputs;
    let notional = cents(balance.mul(statutory)?.div(days)?)?;
    let applied = py_min(accrued.clone(), notional.clone())?;
    let gross = notional.sub(&applied)?;
    let gd = cents(notional.mul(income_use)?)?;
    let rd = py_min(deduction.clone(), gd.clone())?;
    let nd = gd.sub(&rd)?;
    let reduction = py_min(nd.clone(), gross.clone())?;
    let taxable = gross.sub(&reduction)?;
    Ok(steps_from(
        &LOAN_STEPS,
        [
            (&[balance, statutory, days], notional.clone()),
            (&[accrued, &notional], applied.clone()),
            (&[&notional, &applied], gross.clone()),
            (&[&notional, income_use], gd.clone()),
            (&[deduction, &gd], rd.clone()),
            (&[&gd, &rd], nd.clone()),
            (&[&nd, &gross], reduction.clone()),
            (&[&gross, &reduction], taxable),
        ],
    ))
}

/// FBTAA Division 9A: the 50/50 split method (s 37BA) or the 12 week register
/// method (s 37CB) on the year's meal entertainment expenditure.
pub fn fbt_meal_entertainment(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_meal_entertainment";
    let (year, period, _) = fbt_year(scope_confirmed, year_ended, kind)?;
    let method = method(method_arg, kind)?;
    let (command, when) = ("fbt-meal-entertainment", format!(" with method {method}"));
    if method == "50_50_split" {
        let [expenditure] = take(given, command, &when, ["meal_entertainment_expenditure"])?;
        let expenditure = money(expenditure)?;
        let share = d("0.50");
        let formula = "meal_entertainment_expenditure x split_share";
        let taxable = precise(|| Ok(expenditure.mul(&share)?))?;
        let steps = vec![step(
            "taxable_value",
            "FBTAA s 37BA",
            formula,
            &[&expenditure, &share],
            taxable,
        )];
        return derived_result(kind, &period, steps, vec![rate("split_share", &share)]);
    }
    let names = [
        "meal_entertainment_expenditure",
        "register_fringe_benefits_value",
        "register_meal_entertainment_value",
        "register_first_year",
    ];
    let [expenditure, benefits, total, first] = take(given, command, &when, names)?;
    let (expenditure, benefits, total) = (money(expenditure)?, money(benefits)?, money(total)?);
    register_year(first, year.parse().expect("FBT year"), "s 37CD")?;
    if total.is_zero() || benefits.gt(&total)? {
        return refuse(
            "fbt_meal_entertainment.register_values_invalid",
            "register_meal_entertainment_value must be more than nil and at least register_fringe_benefits_value: the register percentage is the share of the register's meal entertainment that is fringe benefits (FBTAA s 37CB(2)).",
        );
    }
    let steps = precise(|| register_steps(&expenditure, &benefits, &total))?;
    derived_result(kind, &period, steps, Vec::new())
}

/// s 37CB: the register percentage, kept exact, applied to the expenditure.
fn register_steps(expenditure: &Decimal, benefits: &Decimal, total: &Decimal) -> Result<Vec<Step>> {
    let hundred = int(100);
    let percentage = benefits.div(total)?.mul(&hundred)?;
    let taxable = expenditure.mul(&percentage)?.div(&hundred)?;
    Ok(vec![
        step(
            "register_percentage",
            "FBTAA s 37CB(2)",
            "register_fringe_benefits_value / register_meal_entertainment_value x 100",
            &[benefits, total],
            percentage.clone(),
        ),
        step(
            "taxable_value",
            "FBTAA s 37CB(1) and (3)",
            "meal_entertainment_expenditure x register_percentage / 100",
            &[expenditure, &percentage],
            taxable,
        ),
    ])
}

const PARKING: &str = "fbt-car-parking";

/// The taxable value when no station's fee on the first business day is above
/// the car parking threshold.
const NO_BENEFIT: &str = "FBTAA s 39A(1)(a)(iii): no commercial parking station within 1 km charged a lowest all-day fee of more than the car parking threshold on the first business day of the FBT year, so no car parking benefit arises";

/// FBTAA Division 10A: the commercial parking station method (s 39C), the
/// average cost method (s 39DA) or the 12 week register method (s 39GB). No
/// car parking benefit arises unless a station's lowest fee on the first
/// business day is more than the year's car parking threshold (s 39A(1)(a)(iii)).
pub fn fbt_car_parking(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_car_parking";
    let (year, period, days_in_year) = fbt_year(scope_confirmed, year_ended, kind)?;
    let method = method(method_arg, kind)?;
    let when = format!(" with method {method}");
    let threshold = d(lookup(meta::FBT_CAR_PARKING_THRESHOLDS, &year));
    let (fee, mut steps, taxable) = match method {
        "12_week_register" => {
            let year = year.parse().expect("FBT year");
            register_parking(given, &when, year, days_in_year)?
        }
        _ => fee_parking(given, &when, method == "average_cost")?,
    };
    steps.push(if fee.gt(&threshold)? {
        taxable
    } else {
        step("taxable_value", NO_BENEFIT, "0", &[], int(0))
    });
    let rates = vec![rate("car_parking_threshold", &threshold)];
    derived_result(kind, &period, steps, rates)
}

/// A car parking result before its taxable value: the first business day's
/// fee, the steps and the taxable value should a benefit arise.
type Parking = (Decimal, Vec<Step>, Step);

/// A fee method's first business day fee, recipients' payments, steps to its
/// gross taxable value and authority.
type FeeMethod = (Decimal, Decimal, Vec<Step>, &'static str);

/// The commercial parking station or average cost method: each benefit's
/// value, reduced by the recipients' payments, never below nil.
fn fee_parking(given: &[(String, Arg)], when: &str, average: bool) -> Result<Parking> {
    let (fee, contribution, mut steps, authority) = if average {
        average_cost(given, when)?
    } else {
        parking_station(given, when)?
    };
    let gross = steps.last().expect("gross step").value.clone();
    let (payment, taxable) = precise(|| {
        let applied = py_min(contribution.clone(), gross.clone())?;
        let net = gross.sub(&applied)?;
        let rows: [(&[&Decimal], Decimal); 2] = [
            (&[&contribution, &gross], applied.clone()),
            (&[&contribution, &applied], contribution.sub(&applied)?),
        ];
        let table = [
            ("employee_contribution_applied", authority, APPLIED),
            ("excess_employee_contribution", authority, EXCESS),
        ];
        let taxable = step("taxable_value", authority, NET, &[&gross, &applied], net);
        Ok((steps_from(&table, rows), taxable))
    })?;
    steps.extend(payment);
    Ok((fee, steps, taxable))
}

/// s 39C: each benefit at the lowest all-day fee on the day it is provided.
fn parking_station(given: &[(String, Arg)], when: &str) -> Result<FeeMethod> {
    let names = [
        "first_business_day_fee",
        "benefits",
        "lowest_fee",
        "employee_contribution",
    ];
    let [fee, benefits, lowest, contribution] = take(given, PARKING, when, names)?;
    let (count, lowest) = (benefit_count(benefits)?, money(lowest)?);
    let gross = precise(|| Ok(count.mul(&lowest)?))?;
    let authority = "FBTAA s 39C";
    let formula = "benefits x lowest_fee";
    let steps = vec![step(
        "gross_taxable_value",
        authority,
        formula,
        &[&count, &lowest],
        gross,
    )];
    Ok((money(fee)?, money(contribution)?, steps, authority))
}

/// s 39DA: each benefit at the average of the lowest fees on the days a
/// benefit is first and last provided in the year.
fn average_cost(given: &[(String, Arg)], when: &str) -> Result<FeeMethod> {
    let names = [
        "first_business_day_fee",
        "benefits",
        "first_day_fee",
        "last_day_fee",
        "employee_contribution",
    ];
    let [fee, benefits, first, last, contribution] = take(given, PARKING, when, names)?;
    let (count, first, last) = (benefit_count(benefits)?, money(first)?, money(last)?);
    let authority = "FBTAA s 39DA(2)";
    let steps = precise(|| {
        let average = first.add(&last)?.div(&int(2))?;
        let gross = cents(count.mul(&average)?)?;
        let formula = "(first_day_fee + last_day_fee) / 2";
        Ok(vec![
            step(
                "average_cost",
                "FBTAA s 39DA(3)",
                formula,
                &[&first, &last],
                average.clone(),
            ),
            step(
                "gross_taxable_value",
                authority,
                "cents(benefits x average_cost)",
                &[&count, &average],
                gross,
            ),
        ])
    })?;
    Ok((money(fee)?, money(contribution)?, steps, authority))
}

/// The number of car parking benefits: a JSON integer from 0.
fn benefit_count(value: &Arg) -> Result<Decimal> {
    match value.int() {
        Some(Some(n)) if n >= 0 => Ok(int(n)),
        _ => refuse(
            "input.benefit_count_invalid",
            "benefits must be a JSON integer from 0.",
        ),
    }
}

/// s 39GB: the register's total value of car parking benefits (s 39GC, already
/// net of the recipients' payments) for 52 weeks, for the availability period's
/// days out of 366.
fn register_parking(
    given: &[(String, Arg)],
    when: &str,
    year: i64,
    days_in_year: i64,
) -> Result<Parking> {
    let names = [
        "first_business_day_fee",
        "register_value",
        "availability_days",
        "register_first_year",
    ];
    let [fee, value, days, first] = take(given, PARKING, when, names)?;
    let (fee, value) = (money(fee)?, money(value)?);
    let days = int(day_count(days, days_in_year, "availability_days")?);
    register_year(first, year, "s 39GF")?;
    let authority = "FBTAA s 39GB";
    precise(|| {
        let annual = value.mul(&int(52))?.div(&int(12))?;
        let gross = cents(annual.mul(&days)?.div(&int(366))?)?;
        let steps = vec![
            step(
                "annual_value",
                authority,
                "register_value x 52 / 12",
                &[&value],
                annual.clone(),
            ),
            step(
                "gross_taxable_value",
                authority,
                "cents(annual_value x availability_days / 366)",
                &[&annual, &days],
                gross.clone(),
            ),
        ];
        let taxable = step(
            "taxable_value",
            "FBTAA s 39GB and s 39GC",
            "gross_taxable_value",
            &[&gross],
            gross.clone(),
        );
        Ok((fee, steps, taxable))
    })
}

mod residual_housing;
pub use residual_housing::{
    fbt_board, fbt_housing, fbt_lafha, fbt_residual, fbt_tax_exempt_body_entertainment,
};
