// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The living-away-from-home allowance, board and tax-exempt body
//! entertainment worksheets (FBTAA Divisions 7, 9 and 10): each benefit's
//! taxable value, which `fbt` then grosses up with the others. They cite the
//! FBTAA as compiled on 1 July 2026 (C2004A03280), 'Fringe benefits tax - a
//! guide for employers', chapters 11, 13 and 15, and the FBT return 2026
//! instructions, item 23, read on 15 November 2026.

use serde_json::Value;

use super::super::{
    Arg, Result, Step, cents, d, derived_result, fbt_year, int, money, precise, py_max, py_min,
    rate, refuse, step, steps_from,
};
use super::{
    DEDUCTIBLE_INPUTS, DeductibleSteps, GROSS_DEDUCTION, ND, RD, REDUCTION, deductible_benefit,
    deductible_inputs, method, take,
};
use crate::decimal::Decimal;

pub(super) const LAFHA_METHODS: [&str; 3] = ["maintains_home", "fly_in_fly_out", "other"];
pub(super) const ENTERTAINMENT_METHODS: [&str; 2] = ["actual", "per_head"];

/// The otherwise deductible rule's steps for board (s 37).
const BOARD_DEDUCTIBLE: DeductibleSteps = [
    (
        "gross_deduction",
        "FBTAA s 37(b), gross deduction",
        GROSS_DEDUCTION,
    ),
    ("contribution_deduction_applied", "FBTAA s 37(c), RD", RD),
    ("notional_deduction", "FBTAA s 37(c), GD - RD", ND),
    (
        "reduction_applied",
        "FBTAA s 37; FBT return 2026 instructions, item 23J",
        REDUCTION,
    ),
];

/// The living-away-from-home allowance worksheet's steps once ss 31 or 31A
/// apply: (id, authority, formula).
const LAFHA_STEPS: [(&str, &str, &str); 5] = [
    (
        "exempt_accommodation_component",
        "FBTAA s 136(1), exempt accommodation component; s 31G",
        "min(accommodation_component, accommodation_expenses)",
    ),
    (
        "statutory_food_total",
        "FBTAA s 136(1), statutory food amount ($42 a week from age 12, $21 under)",
        "(adult_days x 42 + child_days x 21) / 7",
    ),
    (
        "applicable_statutory_food_total",
        "FBTAA s 31H(2), applicable statutory food total",
        "max(statutory_food_total - normal_food_amount, 0)",
    ),
    (
        "exempt_food_component",
        "FBTAA s 31H(1) and (2)",
        "min(max(food_component - applicable_statutory_food_total, 0), food_expenses)",
    ),
    (
        "taxable_value",
        "FBTAA s 31(2) and s 31A(2)",
        "allowance - exempt_accommodation_component - exempt_food_component",
    ),
];

const LAFHA: &str = "fbt-lafha";

/// FBTAA Division 7: the allowance less its exempt accommodation and food
/// components where s 31 (an Australian home kept, the first 12 months) or
/// s 31A (fly-in fly-out or drive-in drive-out) applies; otherwise, under
/// s 31B, the allowance.
pub fn fbt_lafha(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_lafha";
    let (_, period, _) = fbt_year(scope_confirmed, year_ended, kind)?;
    let method = method(method_arg, kind)?;
    let when = format!(" with method {method}");
    if method == "other" {
        let [allowance] = take(given, LAFHA, &when, ["allowance"])?;
        let allowance = money(allowance)?;
        let taxable = step(
            "taxable_value",
            "FBTAA s 31B(2)",
            "allowance",
            &[&allowance],
            allowance.clone(),
        );
        return derived_result(kind, &period, vec![taxable], Vec::new());
    }
    let names = [
        "allowance",
        "accommodation_component",
        "accommodation_expenses",
        "food_component",
        "food_expenses",
        "normal_food_amount",
        "adult_days",
        "child_days",
    ];
    let inputs = take(given, LAFHA, &when, names)?;
    let amounts: Vec<Decimal> = inputs[..6]
        .iter()
        .map(|arg| money(arg))
        .collect::<Result<_>>()?;
    let adults = person_days(inputs[6], "adult_days")?;
    let children = person_days(inputs[7], "child_days")?;
    if precise(|| Ok(amounts[1].add(&amounts[3])?.gt(&amounts[0])?))? {
        return refuse(
            "fbt_lafha.components_exceed_allowance",
            "accommodation_component and food_component together cannot exceed allowance: each is the part of the allowance paid for those expenses (FBTAA s 136(1)).",
        );
    }
    let steps = precise(|| lafha_steps(&amounts, &adults, &children))?;
    derived_result(kind, &period, steps, Vec::new())
}

/// [`LAFHA_STEPS`] on the allowance, its accommodation component and the
/// expenses for it, its food component and the expenses for food, the normal
/// food amount taken into account, and the days of eligible family members.
fn lafha_steps(amounts: &[Decimal], adults: &Decimal, children: &Decimal) -> Result<Vec<Step>> {
    let [allowance, accommodation, rent, food, spent, normal] = amounts else {
        unreachable!("the 6 amounts LAFHA takes")
    };
    let zero = int(0);
    let exempt_accommodation = py_min(accommodation.clone(), rent.clone())?;
    let statutory = adults
        .mul(&int(42))?
        .add(&children.mul(&int(21))?)?
        .div(&int(7))?;
    let applicable = py_max(statutory.sub(normal)?, zero.clone())?;
    let over = py_max(food.sub(&applicable)?, zero)?;
    let exempt_food = py_min(over, spent.clone())?;
    let taxable = allowance.sub(&exempt_accommodation)?.sub(&exempt_food)?;
    Ok(steps_from(
        &LAFHA_STEPS,
        [
            (&[accommodation, rent], exempt_accommodation.clone()),
            (&[adults, children], statutory.clone()),
            (&[&statutory, normal], applicable.clone()),
            (&[food, &applicable, spent], exempt_food.clone()),
            (&[allowance, &exempt_accommodation, &exempt_food], taxable),
        ],
    ))
}

/// Days of eligible family members, added together: a JSON integer from 0.
fn person_days(value: &Arg, name: &str) -> Result<Decimal> {
    match value.int() {
        Some(Some(n)) if (0..=1_000_000).contains(&n) => Ok(int(n)),
        _ => refuse(
            "input.day_count_invalid",
            format!("{name} must be an integer from 0 to 1000000."),
        ),
    }
}

/// FBTAA Division 9: $2.00 a meal for a recipient aged 12 or more before the
/// FBT year and $1.00 for another (s 36), less the recipients' contributions,
/// reduced by the otherwise deductible rule (s 37).
pub fn fbt_board(
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_board";
    let (_, period, _) = fbt_year(scope_confirmed, year_ended, kind)?;
    let names = [
        "adult_meals",
        "child_meals",
        DEDUCTIBLE_INPUTS[0],
        DEDUCTIBLE_INPUTS[1],
        DEDUCTIBLE_INPUTS[2],
    ];
    let [adults, children, c, s, r] = take(given, "fbt-board", "", names)?;
    let adults = meal_count(adults, "adult_meals")?;
    let children = meal_count(children, "child_meals")?;
    let reductions = deductible_inputs([c, s, r], "s 37(c)")?;
    let (adult_value, child_value) = (d("2.00"), d("1.00"));
    let gross = precise(|| {
        Ok(adults
            .mul(&adult_value)?
            .add(&children.mul(&child_value)?)?)
    })?;
    let formula = "adult_meals x adult_meal_value + child_meals x child_meal_value";
    let operands = [&adults, &adult_value, &children, &child_value];
    let gross = step(
        "gross_taxable_value",
        "FBTAA s 36",
        formula,
        &operands,
        gross,
    );
    let authority = "FBTAA s 36 and s 37";
    let steps = deductible_benefit(
        gross,
        &reductions,
        "FBTAA s 36",
        &BOARD_DEDUCTIBLE,
        None,
        authority,
    );
    let steps = steps?;
    let rates = vec![
        rate("adult_meal_value", &adult_value),
        rate("child_meal_value", &child_value),
    ];
    derived_result(kind, &period, steps, rates)
}

/// A number of board meals: a JSON integer from 0.
fn meal_count(value: &Arg, name: &str) -> Result<Decimal> {
    match value.int() {
        Some(Some(n)) if (0..=100_000_000).contains(&n) => Ok(int(n)),
        _ => refuse(
            "input.benefit_count_invalid",
            format!("{name} must be a JSON integer from 0 to 100000000."),
        ),
    }
}

const ENTERTAINMENT: &str = "fbt-tax-exempt-body-entertainment";

/// FBTAA Division 10, s 39: so much of the non-deductible exempt entertainment
/// expenditure as is attributable to entertaining employees and their
/// associates, found as the actual amount or, where that cannot easily be
/// worked out, on a per head basis (the employer's guide, 15.9). No
/// contribution or other reduction applies.
pub fn fbt_tax_exempt_body_entertainment(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_tax_exempt_body_entertainment";
    let (_, period, _) = fbt_year(scope_confirmed, year_ended, kind)?;
    let method = method(method_arg, kind)?;
    let when = format!(" with method {method}");
    let name = "entertainment_expenditure";
    if method == "actual" {
        let [expenditure] = take(given, ENTERTAINMENT, &when, [name])?;
        let expenditure = money(expenditure)?;
        let taxable = step(
            "taxable_value",
            "FBTAA s 39",
            name,
            &[&expenditure],
            expenditure.clone(),
        );
        return derived_result(kind, &period, vec![taxable], Vec::new());
    }
    let names = [name, "employee_heads", "total_heads"];
    let [expenditure, employees, total] = take(given, ENTERTAINMENT, &when, names)?;
    let expenditure = money(expenditure)?;
    let (employees, total) = match (employees.int(), total.int()) {
        (Some(Some(e)), Some(Some(t))) if (0..=t).contains(&e) && (1..=1_000_000).contains(&t) => {
            (int(e), int(t))
        }
        _ => {
            return refuse(
                "fbt_tax_exempt_body_entertainment.heads_invalid",
                "employee_heads and total_heads must be JSON integers, total_heads from 1 to 1000000 and employee_heads from 0 to total_heads: the employees and associates entertained, out of everyone entertained.",
            );
        }
    };
    let taxable = precise(|| Ok(cents(expenditure.mul(&employees)?.div(&total)?)?))?;
    let taxable = step(
        "taxable_value",
        "FBTAA s 39; Fringe benefits tax - a guide for employers, 15.9, per head basis",
        "cents(entertainment_expenditure x employee_heads / total_heads)",
        &[&expenditure, &employees, &total],
        taxable,
    );
    derived_result(kind, &period, vec![taxable], Vec::new())
}

#[cfg(test)]
mod tests {
    use super::lafha_steps;
    use crate::decimal::Decimal;

    #[test]
    #[should_panic(expected = "the 6 amounts LAFHA takes")]
    fn lafha_steps_take_exactly_6_amounts() {
        let zero = Decimal::from_int(0);
        let _ = lafha_steps(&[], &zero, &zero);
    }
}
