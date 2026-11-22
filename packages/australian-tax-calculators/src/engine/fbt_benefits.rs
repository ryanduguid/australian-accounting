// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The loan, meal entertainment and car parking worksheets: each benefit
//! category's taxable value, which `fbt` then grosses up with the others. They
//! take their inputs by name, since which inputs apply turns on the method, and
//! cite the FBTAA as compiled on 1 July 2026 (C2004A03280) and ATO pages, read
//! on 15 November 2026; the market value and spaces car parking methods and the
//! meal entertainment register's 20% rule were read on 16 November 2026, the
//! FBTAA again as compilation 97, the register's current compilation that day.

use serde_json::Value;

use super::{
    APPLIED, Arg, EXCESS, NET, Result, Step, cents, d, day_count, derived_result, fbt_year,
    finite_between, int, lookup, money, precise, py_min, rate, refuse, step, steps_from,
};
use crate::decimal::Decimal;
use crate::metadata as meta;

mod expense_property;
pub use expense_property::{fbt_debt_waiver, fbt_expense_payment, fbt_property};
mod lafha_board;
pub use lafha_board::{fbt_board, fbt_lafha, fbt_tax_exempt_body_entertainment};
mod housing;
pub use housing::fbt_housing;
mod residual;
pub use residual::fbt_residual;
mod entertainment;
mod register;
mod spaces;
pub use entertainment::fbt_entertainment_classifier;
pub use register::fbt_meal_entertainment_register;

/// The methods `method` accepts for `kind`, in the catalogue's order.
pub fn methods(kind: &str) -> &'static [&'static str] {
    match kind {
        "fbt_meal_entertainment" => &["50_50_split", "12_week_register"],
        "fbt_car_parking" => &[
            "commercial_parking_station",
            "market_value",
            "average_cost",
            "spaces",
            "12_week_register",
        ],
        "fbt_expense_payment" => &expense_property::EXPENSE_PAYMENT_METHODS,
        "fbt_property" => &expense_property::PROPERTY_METHODS,
        "fbt_residual" => &residual::RESIDUAL_METHODS,
        "fbt_housing" => &housing::HOUSING_METHODS,
        "fbt_lafha" => &lafha_board::LAFHA_METHODS,
        "fbt_tax_exempt_body_entertainment" => &lafha_board::ENTERTAINMENT_METHODS,
        "fbt_entertainment_classifier" => &["actual", "50_50_split", "12_week_register"],
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
    let input = named(given, command, when, &names)?;
    Ok(names.map(input))
}

/// [`take`] for a list of `names` known only at run time: a lookup of each.
pub(super) fn named<'a>(
    given: &'a [(String, Arg)],
    command: &str,
    when: &str,
    names: &[&str],
) -> Result<impl Fn(&str) -> &'a Arg> {
    if let Some((name, _)) = given.iter().find(|(n, _)| !names.contains(&n.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("{name} is not an input of {command}{when}"),
        );
    }
    if let Some(name) = names
        .iter()
        .find(|name| !given.iter().any(|(n, _)| n == *name))
    {
        return refuse("input.missing_field", format!("{name} is required{when}"));
    }
    Ok(move |name: &str| {
        let found = given.iter().find(|(n, _)| n == name);
        &found.expect("checked present").1
    })
}

/// `income_use`: the share of an amount that would have been a once-only
/// income tax deduction to the employee, for the otherwise deductible rule.
fn income_use(value: &Arg) -> Result<Decimal> {
    match finite_between(value, "0", "1")? {
        Some(share) => Ok(share),
        None => refuse(
            "input.fraction_invalid",
            "income_use must be a finite Decimal fraction from 0 to 1.",
        ),
    }
}

/// The input naming the part of an employee's $1,000 in-house reduction for
/// the year (s 62) their other in-house fringe benefits have left.
const AVAILABLE: &str = "in_house_reduction_available";

/// `in_house_reduction_available`: from 0 to $1,000.
fn in_house_available(value: &Arg) -> Result<Decimal> {
    let available = money(value)?;
    if available.gt(&int(1000))? {
        return refuse(
            "fbt.in_house_reduction_invalid",
            "in_house_reduction_available must be from 0 to 1000.00: the part of the employee's $1,000 in-house reduction for the FBT year that their other in-house fringe benefits have not used (FBTAA s 62(1)).",
        );
    }
    Ok(available)
}

/// The inputs every property, residual and board benefit takes for the
/// contribution and the otherwise deductible rule.
const DEDUCTIBLE_INPUTS: [&str; 3] = [
    "employee_contribution",
    "income_use",
    "contribution_deduction",
];

/// The contribution, income use share and deduction for the contribution, the
/// last at most the contribution (FBTAA `section`, RD).
fn deductible_inputs(inputs: [&Arg; 3], section: &str) -> Result<[Decimal; 3]> {
    let contribution = money(inputs[0])?;
    let share = income_use(inputs[1])?;
    let deduction = money(inputs[2])?;
    if deduction.gt(&contribution)? {
        return refuse(
            "fbt.deduction_exceeds_contribution",
            format!(
                "contribution_deduction cannot exceed employee_contribution: it is the deduction for that consideration (FBTAA {section}, RD)."
            ),
        );
    }
    Ok([contribution, share, deduction])
}

/// The recipient's contribution applied, its excess and the value after it,
/// under `authority`.
fn payment_rows(authority: &'static str) -> [(&'static str, &'static str, &'static str); 3] {
    [
        ("employee_contribution_applied", authority, APPLIED),
        ("excess_employee_contribution", authority, EXCESS),
        ("value_after_contribution", authority, NET),
    ]
}

/// The steps from `gross`'s step: `contribution` applied to it, never taking
/// it below nil, any excess, and the value after it, as the `rows` name them.
fn contribution_steps(
    gross: Step,
    contribution: &Decimal,
    rows: &[(&'static str, &'static str, &str); 3],
) -> Result<Vec<Step>> {
    let value = gross.value.clone();
    precise(|| {
        let applied = py_min(contribution.clone(), value.clone())?;
        let excess = contribution.sub(&applied)?;
        let net = value.sub(&applied)?;
        let mut steps = vec![gross];
        steps.extend(steps_from(
            rows,
            [
                (&[contribution, &value], applied.clone()),
                (&[contribution, &applied], excess),
                (&[&value, &applied], net),
            ],
        ));
        Ok(steps)
    })
}

/// The otherwise deductible rule's steps, (id, authority, formula): the gross
/// deduction on `base`, the deduction the recipient has (RD), the notional
/// deduction and the reduction it allows, never more than the value.
type DeductibleSteps = [(&'static str, &'static str, &'static str); 4];

/// The formulas of a property, residual or board benefit's [`DeductibleSteps`].
const GROSS_DEDUCTION: &str = "cents(gross_taxable_value x income_use)";
const RD: &str = "min(contribution_deduction, gross_deduction)";
const ND: &str = "gross_deduction - contribution_deduction_applied";
const REDUCTION: &str = "min(notional_deduction, value_after_contribution)";

/// [`DeductibleSteps`] on `base` at `share`, with the recipient's deduction
/// `rd`, against `value`, the value after the contribution.
fn deductible_steps(
    table: &DeductibleSteps,
    base: &Decimal,
    share: &Decimal,
    rd: &Decimal,
    value: &Decimal,
) -> Result<Vec<Step>> {
    precise(|| {
        let gd = cents(base.mul(share)?)?;
        let applied = py_min(rd.clone(), gd.clone())?;
        let nd = gd.sub(&applied)?;
        let reduction = py_min(nd.clone(), value.clone())?;
        Ok(steps_from(
            table,
            [
                (&[base, share], gd.clone()),
                (&[rd, &gd], applied.clone()),
                (&[&gd, &applied], nd.clone()),
                (&[&nd, value], reduction),
            ],
        ))
    })
}

/// The value after the contribution less each of `reductions`: its formula,
/// operands and amount.
fn less(
    value: &Decimal,
    reductions: &[(&str, Decimal)],
) -> Result<(String, Vec<Decimal>, Decimal)> {
    let mut formula = "value_after_contribution".to_string();
    let (mut operands, mut net) = (vec![value.clone()], value.clone());
    for (name, amount) in reductions {
        formula = format!("{formula} - {name}");
        operands.push(amount.clone());
        net = precise(|| Ok(net.sub(amount)?))?;
    }
    Ok((formula, operands, net))
}

/// The last steps: s 62's reduction where `available` is given, then the
/// taxable value, the value after the contribution less every reduction.
fn taxable_steps(
    mut steps: Vec<Step>,
    available: Option<Decimal>,
    authority: &'static str,
) -> Result<Vec<Step>> {
    let value = steps
        .iter()
        .find(|s| s.id == "value_after_contribution")
        .expect("a value after the contribution")
        .value
        .clone();
    let mut reductions: Vec<(&str, Decimal)> = steps
        .iter()
        .filter(|s| s.id == "reduction_applied")
        .map(|s| (s.static_id(), s.value.clone()))
        .collect();
    if let Some(available) = available {
        let (formula, operands, net) = less(&value, &reductions)?;
        let applied = precise(|| Ok(py_min(available.clone(), net)?))?;
        let mut all = vec![&available];
        all.extend(&operands);
        let formula = format!("min({AVAILABLE}, {formula})");
        let id = "in_house_reduction_applied";
        steps.push(step(id, "FBTAA s 62(1)", &formula, &all, applied.clone()));
        reductions.push((id, applied));
    }
    let (formula, operands, net) = less(&value, &reductions)?;
    let operands: Vec<&Decimal> = operands.iter().collect();
    steps.push(step("taxable_value", authority, &formula, &operands, net));
    Ok(steps)
}

/// A property, residual or board benefit's steps after `gross`: the
/// contribution (`rows_authority`), the otherwise deductible rule on the gross
/// value (`table`), s 62 where `available` is given, and the taxable value
/// (`authority`).
fn deductible_benefit(
    gross: Step,
    [contribution, share, deduction]: &[Decimal; 3],
    rows_authority: &'static str,
    table: &DeductibleSteps,
    available: Option<Decimal>,
    authority: &'static str,
) -> Result<Vec<Step>> {
    let base = gross.value.clone();
    let rows = payment_rows(rows_authority);
    let mut steps = contribution_steps(gross, contribution, &rows)?;
    let value = steps
        .last()
        .expect("value after contribution")
        .value
        .clone();
    steps.extend(deductible_steps(table, &base, share, deduction, &value)?);
    taxable_steps(steps, available, authority)
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
    let income_use = self::income_use(income_use)?;
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
        "register_years_expenditure",
    ];
    let [expenditure, benefits, total, first, earlier] = take(given, command, &when, names)?;
    let (expenditure, benefits, total) = (money(expenditure)?, money(benefits)?, money(total)?);
    let year: i64 = year.parse().expect("FBT year");
    register_year(first, year, "s 37CD")?;
    let first = first.int().flatten().expect("checked first year");
    register_still_valid(earlier, first, year)?;
    if total.is_zero() || benefits.gt(&total)? {
        return refuse(
            "fbt_meal_entertainment.register_values_invalid",
            "register_meal_entertainment_value must be more than nil and at least register_fringe_benefits_value: the register percentage is the share of the register's meal entertainment that is fringe benefits (FBTAA s 37CB(2)).",
        );
    }
    let steps = precise(|| register_steps(&expenditure, &benefits, &total))?;
    derived_result(kind, &period, steps, Vec::new())
}

/// s 37CB: the register percentage, and the expenditure times the register's
/// ratio, divided once so that a value on a half cent stays exact.
fn register_steps(expenditure: &Decimal, benefits: &Decimal, total: &Decimal) -> Result<Vec<Step>> {
    let percentage = benefits.div(total)?.mul(&int(100))?;
    let taxable = expenditure.mul(benefits)?.div(total)?;
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
            "meal_entertainment_expenditure x register_fringe_benefits_value / register_meal_entertainment_value",
            &[expenditure, benefits, total],
            taxable,
        ),
    ])
}

/// s 37CD(3): a register valid for an FBT year ceases to be valid at the end of
/// it if that year's meal entertainment expenses are more than 20% higher than
/// those of the register's first valid year. `earlier` holds each year's
/// expenses from `first` to the year before `year`, oldest first.
fn register_still_valid(earlier: &Arg, first: i64, year: i64) -> Result<()> {
    fn shape<T>(years: i64) -> Result<T> {
        refuse(
            "fbt_meal_entertainment.register_years_invalid",
            format!(
                "register_years_expenditure must be a list of decimal strings, the meal entertainment expenses (FBTAA s 37CB(3)) of each FBT year from register_first_year to the year before year_ended, oldest first: {years} here."
            ),
        )
    }
    let years = year - first;
    let Arg::Json(Value::Array(items)) = earlier else {
        return shape(years);
    };
    if items.len() as i64 != years {
        return shape(years);
    }
    let amounts = items
        .iter()
        .map(|item| match item.as_str().map(Decimal::parse_input) {
            Some(Ok(amount)) => money(&Arg::Decimal(amount)),
            _ => shape(years),
        })
        .collect::<Result<Vec<_>>>()?;
    let Some((base, later)) = amounts.split_first() else {
        return Ok(());
    };
    let limit = precise(|| Ok(base.mul(&d("1.2"))?))?;
    for (ended, amount) in (first + 1..).zip(later) {
        if amount.gt(&limit)? {
            return refuse(
                "fbt_meal_entertainment.register_ceased",
                format!(
                    "The register ceased to be valid at the end of the FBT year ended 31 March {ended}: that year's meal entertainment expenses, {amount}, were more than 20% higher than {base} for its first valid year, {first} (FBTAA s 37CD(3)). Keep a new register."
                ),
            );
        }
    }
    Ok(())
}

const PARKING: &str = "fbt-car-parking";

/// The taxable value when no station's fee on the first business day is above
/// the car parking threshold.
const NO_BENEFIT: &str = "FBTAA s 39A(1)(a)(iii): no commercial parking station within 1 km charged a lowest all-day fee of more than the car parking threshold on the first business day of the FBT year, so no car parking benefit arises";

/// FBTAA Division 10A: the commercial parking station method (s 39C), the
/// market value method (s 39D), the average cost method (s 39DA), the spaces
/// method (s 39FA) or the 12 week register method (s 39GB). No car parking
/// benefit arises unless a station's lowest fee on the first business day is
/// more than the year's car parking threshold (s 39A(1)(a)(iii)).
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
        // Each group of spaces is checked against the threshold on its own.
        "spaces" => spaces::spaces(given, &when, &threshold, days_in_year)?,
        _ => fee_parking(given, &when, method, &threshold)?,
    };
    // Without a benefit there is no gross value or payment to report, only the
    // nil taxable value; the method's inputs were still checked above.
    if fee.gt(&threshold)? {
        steps.push(taxable);
    } else {
        steps = vec![step("taxable_value", NO_BENEFIT, "0", &[], int(0))];
    }
    let rates = vec![rate("car_parking_threshold", &threshold)];
    derived_result(kind, &period, steps, rates)
}

/// A car parking result before its taxable value: the first business day's
/// fee, the steps and the taxable value should a benefit arise.
type Parking = (Decimal, Vec<Step>, Step);

/// A fee method's first business day fee, recipients' payments, steps to its
/// gross taxable value and authority.
type FeeMethod = (Decimal, Decimal, Vec<Step>, &'static str);

/// The commercial parking station, market value or average cost method: each
/// benefit's value, reduced by the recipients' payments, never below nil.
fn fee_parking(
    given: &[(String, Arg)],
    when: &str,
    method: &str,
    threshold: &Decimal,
) -> Result<Parking> {
    let (fee, contribution, steps, authority) = match method {
        "average_cost" => average_cost(given, when, threshold)?,
        "market_value" => each_benefit(given, when, "market_value", "FBTAA s 39D(2)", threshold)?,
        _ => each_benefit(given, when, "lowest_fee", "FBTAA s 39C", threshold)?,
    };
    let (steps, taxable) = with_payments(steps, &contribution, authority)?;
    Ok((fee, steps, taxable))
}

/// `steps` and the recipients' payments `contribution` applied to their last
/// step, the gross taxable value, never taking it below nil, any excess, and
/// the taxable value, all under `authority`.
fn with_payments(
    mut steps: Vec<Step>,
    contribution: &Decimal,
    authority: &'static str,
) -> Result<(Vec<Step>, Step)> {
    let gross = steps.last().expect("gross step").value.clone();
    let (payment, taxable) = precise(|| {
        let applied = py_min(contribution.clone(), gross.clone())?;
        let net = gross.sub(&applied)?;
        let rows: [(&[&Decimal], Decimal); 2] = [
            (&[contribution, &gross], applied.clone()),
            (&[contribution, &applied], contribution.sub(&applied)?),
        ];
        let table = [
            ("employee_contribution_applied", authority, APPLIED),
            ("excess_employee_contribution", authority, EXCESS),
        ];
        let taxable = step("taxable_value", authority, NET, &[&gross, &applied], net);
        Ok((steps_from(&table, rows), taxable))
    })?;
    steps.extend(payment);
    Ok((steps, taxable))
}

/// s 39C or s 39D: each benefit at the same value, the lowest all-day fee on
/// the day it is provided (`lowest_fee`) or the arm's length amount in the
/// valuer's report (`market_value`).
fn each_benefit(
    given: &[(String, Arg)],
    when: &str,
    value: &'static str,
    authority: &'static str,
    threshold: &Decimal,
) -> Result<FeeMethod> {
    let names = [
        "first_business_day_fee",
        "benefits",
        value,
        "employee_contribution",
    ];
    let [fee, benefits, each, contribution] = take(given, PARKING, when, names)?;
    let (count, each, fee) = (benefit_count(benefits)?, money(each)?, money(fee)?);
    let gross = precise(|| Ok(count.mul(&each)?))?;
    let contribution = fee_contribution(contribution, fee.gt(threshold)?, &count, &each, &gross)?;
    let formula = format!("benefits x {value}");
    let steps = vec![step(
        "gross_taxable_value",
        authority,
        &formula,
        &[&count, &each],
        gross,
    )];
    Ok((fee, contribution, steps, authority))
}

/// s 39DA: each benefit at the average of the lowest fees on the days a
/// benefit is first and last provided in the year.
fn average_cost(given: &[(String, Arg)], when: &str, threshold: &Decimal) -> Result<FeeMethod> {
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
    let fee = money(fee)?;
    let (average, gross) = precise(|| {
        let average = first.add(&last)?.div(&int(2))?;
        let gross = count.mul(&average)?;
        Ok((average, gross))
    })?;
    let contribution =
        fee_contribution(contribution, fee.gt(threshold)?, &count, &average, &gross)?;
    let steps = vec![
        step(
            "average_cost",
            "FBTAA s 39DA(3)",
            "(first_day_fee + last_day_fee) / 2",
            &[&first, &last],
            average.clone(),
        ),
        step(
            "gross_taxable_value",
            authority,
            "benefits x average_cost",
            &[&count, &average],
            gross,
        ),
    ];
    Ok((fee, contribution, steps, authority))
}

/// Under ss 39C, 39D and 39DA a payment can reduce only the benefit it is for.
/// With several benefits, only a raw total up to one benefit's value can be
/// used without knowing how it was allocated.
fn fee_contribution(
    value: &Arg,
    benefit_arises: bool,
    count: &Decimal,
    each: &Decimal,
    gross: &Decimal,
) -> Result<Decimal> {
    if let Arg::Json(Value::Object(fields)) = value {
        let amount = fields
            .get("capped_total")
            .and_then(Value::as_str)
            .filter(|text| text.len() <= 8192)
            .and_then(|text| Decimal::parse_input(text).ok());
        let Some(amount) = amount.filter(|_| fields.len() == 1) else {
            return refuse(
                "fbt_car_parking.capped_contribution_invalid",
                "employee_contribution must contain only capped_total, a decimal string.",
            );
        };
        if !amount.is_finite()
            || amount.lt(&int(0))?
            || amount.gt(&d("1e12"))?
            || amount.exponent() < -3
        {
            return refuse(
                "fbt_car_parking.capped_contribution_invalid",
                "employee_contribution capped_total must be finite, non-negative, at most 3 decimal places and at most 1e12.",
            );
        }
        if amount.gt(gross)? {
            return refuse(
                "fbt_car_parking.capped_contribution_invalid",
                "employee_contribution capped_total cannot exceed gross taxable value.",
            );
        }
        return Ok(amount);
    }
    let total = money(value)?;
    if benefit_arises && count.gt(&int(1))? && total.gt(each)? {
        return refuse(
            "fbt_car_parking.contribution_allocation_needed",
            "employee_contribution exceeds one benefit's value. Work each benefit separately or give capped_total after capping each payment at its own benefit's value (FBTAA ss 39C, 39D and 39DA).",
        );
    }
    Ok(total)
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
        let gross = cents(value.mul(&int(52))?.mul(&days)?.div(&int(12 * 366))?)?;
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
                "cents(register_value x 52 x availability_days / (12 x 366))",
                &[&value, &days],
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
