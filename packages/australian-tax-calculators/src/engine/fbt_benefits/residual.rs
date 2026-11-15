// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The residual worksheet (FBTAA Division 12): a residual benefit's taxable
//! value, which `fbt` then grosses up with the others. It cites the FBTAA as
//! compiled on 1 July 2026 (C2004A03280), 'Fringe benefits tax - a guide for
//! employers', chapter 18, and the FBT return 2026 instructions, item 23, read
//! on 15 November 2026.

use serde_json::Value;

use super::super::{Arg, Result, cents, d, derived_result, fbt_year, money, precise, step};
use super::{
    AVAILABLE, DEDUCTIBLE_INPUTS, DeductibleSteps, GROSS_DEDUCTION, ND, RD, REDUCTION,
    deductible_benefit, deductible_inputs, in_house_available, method, named,
};

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
/// The otherwise deductible rule's steps for a residual benefit (s 52).
const RESIDUAL_DEDUCTIBLE: DeductibleSteps = [
    ("gross_deduction", "FBTAA s 52(1)(b), GD", GROSS_DEDUCTION),
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
    let mut names = DEDUCTIBLE_INPUTS.to_vec();
    names.push(input);
    if reduces {
        names.push(AVAILABLE);
    }
    let given = named(given, "fbt-residual", &when, &names)?;
    let available = reduces
        .then(|| in_house_available(given(AVAILABLE)))
        .transpose()?;
    let value = money(given(input))?;
    let reductions = deductible_inputs(DEDUCTIBLE_INPUTS.map(&given), "s 52(1)(ba)")?;
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
    let authority = if available.is_some() {
        "FBTAA s 52(1) and s 62(1)"
    } else {
        "FBTAA s 52(1), TV - ND"
    };
    let rows = "FBTAA ss 48 to 51";
    let table = &RESIDUAL_DEDUCTIBLE;
    let steps = deductible_benefit(gross, &reductions, rows, table, available, authority)?;
    derived_result(kind, &period, steps, Vec::new())
}
