// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The residual, housing, living-away-from-home allowance, board and
//! tax-exempt body entertainment worksheets (FBTAA Divisions 12, 6, 7, 9 and
//! 10): each benefit's taxable value, which `fbt` then grosses up with the
//! others. They cite the FBTAA as compiled on 1 July 2026 (C2004A03280), the
//! ATO's FBT rates and thresholds page, 'Fringe benefits tax - a guide for
//! employers', chapters 10, 11, 13, 15 and 18, and the FBT return 2026
//! instructions, item 23, read on 15 November 2026.

use serde_json::Value;

use super::super::{
    APPLIED, Arg, EXCESS, NET, Result, Step, cents, d, day_count, derived_result, fbt_year,
    finite_between, flag, int, lookup, money, precise, py_max, py_min, rate, refuse, step,
    steps_from,
};
use super::{method, take};
use crate::decimal::Decimal;
use crate::metadata as meta;

/// A residual valuation rule: (method, authority, whether 75% of the input
/// applies, the input valued, whether the s 62 reduction applies).
type Rule = (&'static str, &'static str, bool, &'static str, bool);
const RULES: [Rule; 6] = [
    (
        "in_house_salary_packaging",
        "FBTAA s 48(aa) and s 49(aa)",
        false,
        "notional_value",
        false,
    ),
    (
        "in_house_public_price",
        "FBTAA s 48(a) and s 49(a)",
        true,
        "lowest_price",
        true,
    ),
    (
        "in_house_other",
        "FBTAA s 48(b) and s 49(b)",
        true,
        "notional_value",
        true,
    ),
    (
        "external_cost_price",
        "FBTAA s 50(a) and s 51(a)",
        false,
        "cost_price",
        false,
    ),
    (
        "external_expenditure",
        "FBTAA s 50(b) and s 51(b)",
        false,
        "employer_expenditure",
        false,
    ),
    (
        "external_notional_value",
        "FBTAA s 50(c) and s 51(c)",
        false,
        "notional_value",
        false,
    ),
];

/// The residual worksheet's methods: one for each rule.
pub(super) const RESIDUAL_METHODS: [&str; 6] = {
    let mut out = [""; 6];
    let mut i = 0;
    while i < 6 {
        out[i] = RULES[i].0;
        i += 1;
    }
    out
};
pub(super) const HOUSING_METHODS: [&str; 4] = [
    "outside_australia",
    "accommodation_business",
    "base_year",
    "indexed",
];
pub(super) const LAFHA_METHODS: [&str; 3] = ["maintains_home", "fly_in_fly_out", "other"];
pub(super) const ENTERTAINMENT_METHODS: [&str; 2] = ["actual", "per_head"];

/// The steps after a gross taxable value that a recipient's contribution and
/// the otherwise deductible rule reduce: (id, authority, formula).
type DeductibleRows = [(&'static str, &'static str, &'static str); 7];
const RESIDUAL_ROWS: DeductibleRows = [
    (
        "employee_contribution_applied",
        "FBTAA ss 48 to 51",
        APPLIED,
    ),
    ("excess_employee_contribution", "FBTAA ss 48 to 51", EXCESS),
    ("value_after_contribution", "FBTAA ss 48 to 51", VALUE_AFTER),
    (
        "gross_deduction",
        "FBTAA s 52(1)(b), GD",
        "cents(gross_taxable_value x income_use)",
    ),
    (
        "contribution_deduction_applied",
        "FBTAA s 52(1)(ba), RD",
        RD,
    ),
    ("notional_deduction", "FBTAA s 52(1)(ba), ND = GD - RD", ND),
    (
        "reduction_applied",
        "FBTAA s 52(1), TV - ND; FBT return 2026 instructions, item 23M",
        REDUCTION,
    ),
];
const BOARD_ROWS: DeductibleRows = [
    ("employee_contribution_applied", "FBTAA s 36", APPLIED),
    ("excess_employee_contribution", "FBTAA s 36", EXCESS),
    ("value_after_contribution", "FBTAA s 36", VALUE_AFTER),
    (
        "gross_deduction",
        "FBTAA s 37(b), gross deduction",
        "cents(gross_taxable_value x income_use)",
    ),
    ("contribution_deduction_applied", "FBTAA s 37(c), RD", RD),
    ("notional_deduction", "FBTAA s 37(c), GD - RD", ND),
    (
        "reduction_applied",
        "FBTAA s 37; FBT return 2026 instructions, item 23J",
        REDUCTION,
    ),
];
const VALUE_AFTER: &str = NET;
const RD: &str = "min(contribution_deduction, gross_deduction)";
const ND: &str = "gross_deduction - contribution_deduction_applied";
const REDUCTION: &str = "min(notional_deduction, value_after_contribution)";

/// The inputs every residual and board benefit takes for the contribution and
/// the otherwise deductible rule.
const DEDUCTIBLE_INPUTS: [&str; 3] = [
    "employee_contribution",
    "income_use",
    "contribution_deduction",
];

/// The contribution, income use share and deduction for the contribution, the
/// last at most the contribution (`section`, RD).
fn deductible_inputs(inputs: [&Arg; 3], section: &str) -> Result<[Decimal; 3]> {
    let contribution = money(inputs[0])?;
    let Some(share) = finite_between(inputs[1], "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "income_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
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

/// `rows` on `gross`: the contribution applied, never taking the value below
/// nil, then the notional deduction on the gross value at `share` less the
/// deduction for the contribution, and the reduction it allows, never more
/// than the value after the contribution.
fn deductible_steps(
    rows: &DeductibleRows,
    gross: &Decimal,
    [contribution, share, deduction]: &[Decimal; 3],
) -> Result<Vec<Step>> {
    precise(|| {
        let applied = py_min(contribution.clone(), gross.clone())?;
        let value = gross.sub(&applied)?;
        let gd = cents(gross.mul(share)?)?;
        let rd = py_min(deduction.clone(), gd.clone())?;
        let nd = gd.sub(&rd)?;
        let reduction = py_min(nd.clone(), value.clone())?;
        Ok(steps_from(
            rows,
            [
                (&[contribution, gross], applied.clone()),
                (&[contribution, &applied], contribution.sub(&applied)?),
                (&[gross, &applied], value.clone()),
                (&[gross, share], gd.clone()),
                (&[deduction, &gd], rd.clone()),
                (&[&gd, &rd], nd.clone()),
                (&[&nd, &value], reduction),
            ],
        ))
    })
}

/// The value of the step `id` in `steps`.
fn value_of(steps: &[Step], id: &str) -> Decimal {
    let found = steps.iter().find(|s| s.id == id);
    found.expect("step present").value.clone()
}

/// The taxable value: the value after the contribution less the otherwise
/// deductible reduction.
fn taxable_step(steps: &[Step], authority: &'static str) -> Result<Step> {
    let value = value_of(steps, "value_after_contribution");
    let reduction = value_of(steps, "reduction_applied");
    let taxable = precise(|| Ok(value.sub(&reduction)?))?;
    let formula = "value_after_contribution - reduction_applied";
    Ok(step(
        "taxable_value",
        authority,
        formula,
        &[&value, &reduction],
        taxable,
    ))
}

/// The input naming the part of an employee's $1,000 in-house reduction for
/// the year (s 62) their other in-house fringe benefits have left.
const AVAILABLE: &str = "in_house_reduction_available";

/// FBTAA Division 12: the in-house (ss 48 and 49) or external (ss 50 and 51)
/// value less the recipient's contribution, reduced by the otherwise
/// deductible rule (s 52) and, for an in-house benefit outside a salary
/// packaging arrangement, the employee's $1,000 reduction (s 62).
pub fn fbt_residual(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_residual";
    let (_, period, _) = fbt_year(scope_confirmed, year_ended, kind)?;
    let method = method(method_arg, kind)?;
    let rule = *RULES.iter().find(|r| r.0 == method).expect("a rule");
    let (_, authority, three_quarters, input, reduces) = rule;
    let when = format!(" with method {method}");
    let (inputs, available) = if reduces {
        let names = [
            DEDUCTIBLE_INPUTS[0],
            DEDUCTIBLE_INPUTS[1],
            DEDUCTIBLE_INPUTS[2],
            input,
            AVAILABLE,
        ];
        let [c, s, r, value, available] = take(given, "fbt-residual", &when, names)?;
        ([c, s, r, value], Some(in_house_available(available)?))
    } else {
        let names = [
            DEDUCTIBLE_INPUTS[0],
            DEDUCTIBLE_INPUTS[1],
            DEDUCTIBLE_INPUTS[2],
            input,
        ];
        (take(given, "fbt-residual", &when, names)?, None)
    };
    let value = money(inputs[3])?;
    let reductions = deductible_inputs([inputs[0], inputs[1], inputs[2]], "s 52(1)(ba)")?;
    let gross = precise(|| {
        Ok(if three_quarters {
            let formula = format!("cents(0.75 x {input})");
            let gross = cents(value.mul(&d("0.75"))?)?;
            step("gross_taxable_value", authority, &formula, &[&value], gross)
        } else {
            step(
                "gross_taxable_value",
                authority,
                input,
                &[&value],
                value.clone(),
            )
        })
    })?;
    let mut steps = deductible_steps(&RESIDUAL_ROWS, &gross.value, &reductions)?;
    steps.insert(0, gross);
    let taxable = match available {
        None => taxable_step(&steps, "FBTAA s 52(1), TV - ND")?,
        Some(available) => {
            let value = value_of(&steps, "value_after_contribution");
            let reduction = value_of(&steps, "reduction_applied");
            let (applied, taxable) = precise(|| {
                let left = value.sub(&reduction)?;
                let applied = py_min(available.clone(), left)?;
                Ok((applied.clone(), value.sub(&reduction)?.sub(&applied)?))
            })?;
            steps.push(step(
                "in_house_reduction_applied",
                "FBTAA s 62(1)",
                "min(in_house_reduction_available, value_after_contribution - reduction_applied)",
                &[&available, &value, &reduction],
                applied.clone(),
            ));
            step(
                "taxable_value",
                "FBTAA s 52(1) and s 62(1)",
                "value_after_contribution - reduction_applied - in_house_reduction_applied",
                &[&value, &reduction, &applied],
                taxable,
            )
        }
    };
    steps.push(taxable);
    derived_result(kind, &period, steps, Vec::new())
}

/// The part of the employee's $1,000 in-house reduction left: from 0 to 1,000.
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

const HOUSING: &str = "fbt-housing";

/// FBTAA Division 6, s 26: the market value of the recipient's current
/// housing right outside Australia (s 26(1)(a)) or in a business that lets
/// identical accommodation to outsiders (s 26(1)(b)), or otherwise its
/// statutory annual value for the tenancy period (s 26(1)(c)), worked out in a
/// base year (s 26(2)(a)) or indexed from the year before (s 26(2)(b)); less
/// the recipient's rent, never below nil.
pub fn fbt_housing(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_housing";
    let (year, period, days_in_year) = fbt_year(scope_confirmed, year_ended, kind)?;
    let method = method(method_arg, kind)?;
    let when = format!(" with method {method}");
    let contribution_name = "employee_contribution";
    let (contribution, mut steps, rates) = match method {
        "outside_australia" => {
            let names = ["market_value", contribution_name];
            let [value, contribution] = take(given, HOUSING, &when, names)?;
            let value = money(value)?;
            let gross = step(
                "gross_taxable_value",
                "FBTAA s 26(1)(a)",
                "market_value",
                &[&value],
                value.clone(),
            );
            (contribution, vec![gross], Vec::new())
        }
        "accommodation_business" => {
            let names = ["market_value", "in_house", contribution_name];
            let [value, in_house, contribution] = take(given, HOUSING, &when, names)?;
            let value = money(value)?;
            let factor = d(if flag(in_house, "in_house")? {
                "0.75"
            } else {
                "1"
            });
            let gross = precise(|| Ok(cents(value.mul(&factor)?)?))?;
            let formula = "cents(market_value x in_house_factor)";
            let gross = step(
                "gross_taxable_value",
                "FBTAA s 26(1)(b)",
                formula,
                &[&value, &factor],
                gross,
            );
            (
                contribution,
                vec![gross],
                vec![rate("in_house_factor", &factor)],
            )
        }
        _ => {
            let year = year.parse().expect("FBT year");
            statutory_value(given, &when, method, year, days_in_year)?
        }
    };
    let contribution = money(contribution)?;
    let gross = steps.last().expect("gross step").value.clone();
    let authority = "FBTAA s 26(1), recipients rent";
    precise(|| {
        let applied = py_min(contribution.clone(), gross.clone())?;
        let table = [
            ("employee_contribution_applied", authority, APPLIED),
            ("excess_employee_contribution", authority, EXCESS),
            ("taxable_value", authority, NET),
        ];
        let rows: [(&[&Decimal], Decimal); 3] = [
            (&[&contribution, &gross], applied.clone()),
            (&[&contribution, &applied], contribution.sub(&applied)?),
            (&[&gross, &applied], gross.sub(&applied)?),
        ];
        steps.extend(steps_from(&table, rows));
        Ok(())
    })?;
    derived_result(kind, &period, steps, rates)
}

/// A housing worksheet's recipient's rent input, its steps to the gross
/// taxable value and its rates.
type Housing<'a> = (&'a Arg, Vec<Step>, Vec<(String, String)>);

/// s 26(1)(c): the statutory annual value, worked out in a base year from the
/// market value for the tenancy period (s 26(2)(a)) or as the year before's
/// value times the State or Territory's indexation factor (s 26(2)(b)), for
/// the whole days of the tenancy period.
fn statutory_value<'a>(
    given: &'a [(String, Arg)],
    when: &str,
    method: &str,
    year: i64,
    days_in_year: i64,
) -> Result<Housing<'a>> {
    let days = int(days_in_year);
    let (contribution, tenancy, annual, mut rates) = if method == "base_year" {
        let names = ["market_value", "tenancy_days", "employee_contribution"];
        let [value, tenancy, contribution] = take(given, HOUSING, when, names)?;
        let value = money(value)?;
        // The base year's value divides by the tenancy's days, so it needs one.
        let tenancy = match tenancy.int() {
            Some(Some(n)) if (1..=days_in_year).contains(&n) => int(n),
            _ => {
                return refuse(
                    "input.day_count_invalid",
                    format!(
                        "tenancy_days must be an integer from 1 to {days_in_year} with method base_year."
                    ),
                );
            }
        };
        let annual = precise(|| Ok(cents(value.mul(&days)?.div(&tenancy)?)?))?;
        let formula = "cents(market_value x days_in_year / tenancy_days)";
        let annual = step(
            "statutory_annual_value",
            "FBTAA s 26(2)(a)",
            formula,
            &[&value, &days, &tenancy],
            annual,
        );
        (contribution, tenancy, annual, Vec::new())
    } else {
        let names = [
            "prior_statutory_annual_value",
            "state",
            "base_year_ended",
            "tenancy_days",
            "employee_contribution",
        ];
        let [prior, state, base, tenancy, contribution] = take(given, HOUSING, when, names)?;
        let prior = money(prior)?;
        let factor = indexation_factor(state, year)?;
        base_year_ended(base, year)?;
        let tenancy = int(day_count(tenancy, days_in_year, "tenancy_days")?);
        let annual = precise(|| Ok(cents(prior.mul(&factor)?)?))?;
        let formula = "cents(prior_statutory_annual_value x indexation_factor)";
        let annual = step(
            "statutory_annual_value",
            "FBTAA s 26(2)(b) and s 28; FBT rates and thresholds, housing indexation",
            formula,
            &[&prior, &factor],
            annual,
        );
        (
            contribution,
            tenancy,
            annual,
            vec![rate("indexation_factor", &factor)],
        )
    };
    let value = annual.value.clone();
    let gross = precise(|| Ok(cents(value.mul(&tenancy)?.div(&days)?)?))?;
    let formula = "cents(statutory_annual_value x tenancy_days / days_in_year)";
    let gross = step(
        "gross_taxable_value",
        "FBTAA s 26(1)(c)",
        formula,
        &[&value, &tenancy, &days],
        gross,
    );
    rates.push(rate("days_in_year", days_in_year));
    Ok((contribution, vec![annual, gross], rates))
}

/// The housing indexation factor for `state` in the FBT year ended 31 March
/// `year` (s 28), as the ATO's rates page publishes it.
fn indexation_factor(state: &Arg, year: i64) -> Result<Decimal> {
    let table = lookup(meta::FBT_HOUSING_INDEXATION, &year.to_string());
    if let Some((_, factor)) = state
        .text()
        .and_then(|s| table.iter().find(|(k, _)| *k == s))
    {
        return Ok(d(factor));
    }
    refuse(
        "input.state_invalid",
        "state must be NSW, VIC, QLD, SA, WA, TAS, NT or ACT, where the unit of accommodation is: the Jervis Bay Territory is part of NSW, and Christmas Island and the Cocos (Keeling) Islands are part of NT (FBTAA s 28(5)).",
    )
}

/// The base year must be one of the 9 years before `year`: after 9 years
/// without one, the year is a base year again (s 26(3)(c)).
fn base_year_ended(value: &Arg, year: i64) -> Result<()> {
    match value.int() {
        Some(Some(base)) if (year - 9..year).contains(&base) => Ok(()),
        _ => refuse(
            "fbt_housing.base_year_invalid",
            format!(
                "base_year_ended must be the FBT year, from {} to {}, last treated as a base year for this housing right: a year after 9 years without one is a base year (FBTAA s 26(3)(c)), so use method base_year.",
                year - 9,
                year - 1
            ),
        ),
    }
}

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
    let mut steps = deductible_steps(&BOARD_ROWS, &gross.value, &reductions)?;
    steps.insert(0, gross);
    steps.push(taxable_step(&steps, "FBTAA s 36 and s 37")?);
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
