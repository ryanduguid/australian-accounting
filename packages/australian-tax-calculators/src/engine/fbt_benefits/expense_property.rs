// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The debt waiver, expense payment and property worksheets (FBTAA Divisions 3,
//! 5 and 11): each benefit's taxable value, which `fbt` then grosses up with the
//! others. They cite the FBTAA as compiled on 1 July 2026 (C2004A03280) and the
//! ATO's 'Fringe benefits tax - a guide for employers', chapters 8, 9, 17 and
//! 19, and FBT return 2026 instructions, item 23, read on 15 November 2026.

use serde_json::Value;

use super::super::{
    Arg, Result, Step, d, derived_result, fbt_year, money, precise, py_min, refuse, step,
};
use super::{
    AVAILABLE, DEDUCTIBLE_INPUTS, DeductibleSteps, GROSS_DEDUCTION, ND, RD, REDUCTION,
    contribution_steps, deductible_benefit, deductible_inputs, deductible_steps,
    in_house_available, income_use, method, named, payment_rows, take, taxable_steps,
};
use crate::decimal::Decimal;

/// How a property valuation rule turns its inputs into a value.
#[derive(Clone, Copy)]
enum Valuation {
    /// The input as it is.
    Whole,
    /// 75% of the input.
    ThreeQuarters,
    /// The lesser of the 2 inputs.
    Lesser,
}
use Valuation::{Lesser, ThreeQuarters, Whole};

/// A property valuation rule: (method, authority, valuation, inputs). The
/// first 6 are the in-house rules (s 42(1)), the last 3 the external (s 43).
type Rule = (
    &'static str,
    &'static str,
    Valuation,
    &'static [&'static str],
);
const RULES: [Rule; 9] = [
    (
        "in_house_salary_packaging",
        "FBTAA s 42(1)(aa)",
        Whole,
        &["notional_value"],
    ),
    (
        "in_house_trade_price",
        "FBTAA s 42(1)(a)(i)",
        Whole,
        &["lowest_price"],
    ),
    (
        "in_house_public_price",
        "FBTAA s 42(1)(a)(ii)",
        ThreeQuarters,
        &["lowest_price"],
    ),
    (
        "in_house_produced_other",
        "FBTAA s 42(1)(a)(iii)",
        ThreeQuarters,
        &["notional_value"],
    ),
    (
        "in_house_acquired",
        "FBTAA s 42(1)(b) and (2)",
        Lesser,
        &["arms_length_price", "notional_value"],
    ),
    (
        "in_house_other",
        "FBTAA s 42(1)(c)",
        ThreeQuarters,
        &["notional_value"],
    ),
    (
        "external_cost_price",
        "FBTAA s 43(a)",
        Whole,
        &["cost_price"],
    ),
    (
        "external_expenditure",
        "FBTAA s 43(b)",
        Whole,
        &["employer_expenditure"],
    ),
    (
        "external_notional_value",
        "FBTAA s 43(c)",
        Whole,
        &["notional_value"],
    ),
];

/// The property worksheet's methods: one for each rule.
pub(super) const PROPERTY_METHODS: [&str; 9] = {
    let mut out = [""; 9];
    let mut i = 0;
    while i < 9 {
        out[i] = RULES[i].0;
        i += 1;
    }
    out
};

/// The expense payment worksheet's methods: an external payment (s 23), or an
/// in-house property expense payment (s 22A(1)) valued by an in-house rule.
pub(super) const EXPENSE_PAYMENT_METHODS: [&str; 7] = {
    let mut out = ["external"; 7];
    let mut i = 0;
    while i < 6 {
        out[i + 1] = RULES[i].0;
        i += 1;
    }
    out
};

/// The debt waiver worksheet's only input.
const WAIVED: &str = "amount_waived";

/// FBTAA s 15: a debt waiver fringe benefit's taxable value is the amount
/// waived. No recipient's contribution or otherwise deductible rule applies.
pub fn fbt_debt_waiver(
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_debt_waiver";
    let (_, period, _) = fbt_year(scope_confirmed, year_ended, kind)?;
    let [waived] = take(given, "fbt-debt-waiver", "", [WAIVED])?;
    let waived = money(waived)?;
    let authority = "FBTAA s 15; FBT return 2026 instructions, item 23D";
    let steps = vec![step(
        "taxable_value",
        authority,
        WAIVED,
        &[&waived],
        waived.clone(),
    )];
    derived_result(kind, &period, steps, Vec::new())
}

/// The rule `method` names.
fn rule(method: &str) -> Rule {
    *RULES
        .iter()
        .find(|r| r.0 == method)
        .expect("a rule per method")
}

/// Whether s 62's in-house reduction applies: an in-house rule, but not to a
/// benefit under a salary packaging arrangement (s 62(2)).
fn reduces(rule: Rule) -> bool {
    rule.0.starts_with("in_house_") && rule.0 != "in_house_salary_packaging"
}

/// The inputs `rule` takes beside those every method of a worksheet takes.
fn rule_inputs(rule: Rule) -> Vec<&'static str> {
    let mut names = rule.3.to_vec();
    if reduces(rule) {
        names.push(AVAILABLE);
    }
    names
}

/// The value `rule` gives the property before any contribution.
fn value_step<'a>(rule: Rule, input: impl Fn(&str) -> &'a Arg) -> Result<Step> {
    let (_, authority, valuation, names) = rule;
    let amounts = names
        .iter()
        .map(|name| money(input(name)))
        .collect::<Result<Vec<_>>>()?;
    let id = "gross_taxable_value";
    precise(|| {
        Ok(match valuation {
            Whole => step(id, authority, names[0], &[&amounts[0]], amounts[0].clone()),
            ThreeQuarters => {
                let value = amounts[0].mul(&d("0.75"))?;
                let formula = format!("{} x 0.75", names[0]);
                step(id, authority, &formula, &[&amounts[0]], value)
            }
            Lesser => {
                let value = py_min(amounts[0].clone(), amounts[1].clone())?;
                let formula = format!("min({}, {})", names[0], names[1]);
                step(id, authority, &formula, &[&amounts[0], &amounts[1]], value)
            }
        })
    })
}

/// The s 62 reduction left for this benefit, where `rule` takes one: at most
/// $1,000.
fn available<'a>(rule: Rule, input: impl Fn(&str) -> &'a Arg) -> Result<Option<Decimal>> {
    if !reduces(rule) {
        return Ok(None);
    }
    Ok(Some(in_house_available(input(AVAILABLE))?))
}

/// The otherwise deductible rule's steps for a property benefit (s 44).
const PROPERTY_DEDUCTIBLE: DeductibleSteps = [
    ("gross_deduction", "FBTAA s 44(1)(b), GD", GROSS_DEDUCTION),
    (
        "contribution_deduction_applied",
        "FBTAA s 44(1)(ba), RD",
        RD,
    ),
    ("notional_deduction", "FBTAA s 44(1)(ba), ND = GD - RD", ND),
    (
        "reduction_applied",
        "FBTAA s 44(1); FBT return 2026 instructions, item 23K",
        REDUCTION,
    ),
];

/// The otherwise deductible rule's steps for an external expense payment
/// (s 24), on the recipient's expenditure.
const EXPENSE_DEDUCTIBLE: DeductibleSteps = [
    (
        "gross_deduction",
        "FBTAA s 24(1)(b)(ii), GD",
        "recipients_expenditure x income_use",
    ),
    (
        "recipients_portion_deduction_applied",
        "FBTAA s 24(1)(ba), RD",
        "min(recipients_portion_deduction, gross_deduction)",
    ),
    (
        "notional_deduction",
        "FBTAA s 24(1)(ba), ND = GD - RD",
        "gross_deduction - recipients_portion_deduction_applied",
    ),
    (
        "reduction_applied",
        "FBTAA s 24(1); FBT return 2026 instructions, item 23E",
        REDUCTION,
    ),
];

const PROPERTY: &str = "fbt-property";

/// FBTAA Division 11: the in-house (s 42) or external (s 43) value less the
/// recipient's contribution, reduced by the otherwise deductible rule (s 44)
/// and, for an in-house benefit, the employee's $1,000 reduction (s 62).
pub fn fbt_property(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_property";
    let (_, period, _) = fbt_year(scope_confirmed, year_ended, kind)?;
    let rule = rule(method(method_arg, kind)?);
    let when = format!(" with method {}", rule.0);
    let mut names = DEDUCTIBLE_INPUTS.to_vec();
    names.extend(rule_inputs(rule));
    let input = named(given, PROPERTY, &when, &names)?;
    let gross = value_step(rule, &input)?;
    let reductions = deductible_inputs(DEDUCTIBLE_INPUTS.map(&input), "s 44(1)(ba)")?;
    let available = available(rule, &input)?;
    let rows = if rule.0.starts_with("in_house_") {
        "FBTAA s 42(1)"
    } else {
        "FBTAA s 43"
    };
    let authority = if available.is_some() {
        "FBTAA s 44(1), TV - ND, and s 62(1); FBT return 2026 instructions, item 23K"
    } else {
        "FBTAA s 44(1), TV - ND; FBT return 2026 instructions, item 23K"
    };
    let table = &PROPERTY_DEDUCTIBLE;
    let steps = deductible_benefit(gross, &reductions, rows, table, available, authority)?;
    derived_result(kind, &period, steps, Vec::new())
}

const EXPENSE: &str = "fbt-expense-payment";

/// FBTAA Division 5: an external expense payment's value (s 23), reduced by
/// the otherwise deductible rule (s 24), or an in-house property expense
/// payment's (s 22A(1)), the in-house property value less the recipient's
/// portion, reduced by the employee's $1,000 in-house reduction (s 62).
pub fn fbt_expense_payment(
    method_arg: &Arg,
    given: &[(String, Arg)],
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_expense_payment";
    let (_, period, _) = fbt_year(scope_confirmed, year_ended, kind)?;
    let method = method(method_arg, kind)?;
    let when = format!(" with method {method}");
    let rule = (method != "external").then(|| rule(method));
    let deductible = ["income_use", "recipients_portion_deduction"];
    if let Some((name, _)) = given
        .iter()
        .find(|(n, _)| rule.is_some() && deductible.contains(&n.as_str()))
    {
        return refuse(
            "fbt_expense_payment.in_house_deductible_not_covered",
            format!(
                "{name} is not an input of {EXPENSE}{when}: the otherwise deductible rule for an in-house expense payment (FBTAA s 24(1)(b)(i)) is not covered, so the taxable value is before any reduction it allows."
            ),
        );
    }
    let mut names = vec![
        "recipients_expenditure",
        "amount_paid",
        "employee_contribution",
    ];
    match rule {
        None => names.extend(deductible),
        Some(rule) => names.extend(rule_inputs(rule)),
    }
    let input = named(given, EXPENSE, &when, &names)?;
    let expenditure = money(input("recipients_expenditure"))?;
    let paid = money(input("amount_paid"))?;
    let contribution = money(input("employee_contribution"))?;
    if paid.gt(&expenditure)? {
        return refuse(
            "fbt_expense_payment.payment_exceeds_expenditure",
            "amount_paid cannot exceed recipients_expenditure: it pays or reimburses that expenditure in whole or in part (FBTAA s 20).",
        );
    }
    let amounts = [&expenditure, &paid, &contribution];
    let steps = match rule {
        None => external_payment(amounts, &input)?,
        Some(rule) => in_house_payment(rule, amounts, &input)?,
    };
    derived_result(kind, &period, steps, Vec::new())
}

/// s 23 and s 24: the payment less the recipient's contribution, reduced by
/// the notional deduction on the recipient's expenditure.
fn external_payment<'a>(
    amounts: [&Decimal; 3],
    input: impl Fn(&str) -> &'a Arg,
) -> Result<Vec<Step>> {
    let [expenditure, paid, contribution] = amounts;
    let share = income_use(input("income_use"))?;
    let deduction = money(input("recipients_portion_deduction"))?;
    // s 136(1): the recipient's portion is the expenditure less the
    // provider's portion, the payment less the recipient's contribution.
    let portion = precise(|| Ok(expenditure.sub(&paid.sub(contribution)?)?))?;
    if deduction.gt(&portion)? {
        return refuse(
            "fbt_expense_payment.deduction_exceeds_portion",
            "recipients_portion_deduction cannot exceed the recipient's portion, recipients_expenditure less amount_paid net of employee_contribution: it is the deduction for the expenditure the recipient bore (FBTAA s 24(1)(ba), RD, and s 136(1)).",
        );
    }
    let gross = step(
        "gross_taxable_value",
        "FBTAA s 23",
        "amount_paid",
        &[paid],
        paid.clone(),
    );
    let mut steps = contribution_steps(gross, contribution, &payment_rows("FBTAA s 23"))?;
    let value = steps
        .last()
        .expect("value after contribution")
        .value
        .clone();
    let table = &EXPENSE_DEDUCTIBLE;
    steps.extend(deductible_steps(
        table,
        expenditure,
        &share,
        &deduction,
        &value,
    )?);
    let authority = "FBTAA s 24(1), TV - ND; FBT return 2026 instructions, item 23E";
    taxable_steps(steps, None, authority)
}

/// The notional contribution of an in-house property expense payment, the
/// recipient's portion, applied to the in-house value, its excess and the
/// value after it (s 22A(1)(b)).
const PORTION_ROWS: [(&str, &str, &str); 3] = [
    (
        "recipients_portion_applied",
        "FBTAA s 22A(1)(b)",
        "min(recipients_portion, gross_taxable_value)",
    ),
    (
        "excess_recipients_portion",
        "FBTAA s 22A(1)(b)",
        "recipients_portion - recipients_portion_applied",
    ),
    (
        "value_after_contribution",
        "FBTAA s 22A(1)",
        "gross_taxable_value - recipients_portion_applied",
    ),
];

/// s 22A(1): the in-house property value `rule` gives, less the recipient's
/// portion as the contribution, then s 62. The otherwise deductible rule for
/// an in-house expense payment (s 24(1)(b)(i)) is not covered.
fn in_house_payment<'a>(
    rule: Rule,
    amounts: [&Decimal; 3],
    input: impl Fn(&str) -> &'a Arg,
) -> Result<Vec<Step>> {
    let [expenditure, paid, contribution] = amounts;
    let gross = value_step(rule, &input)?;
    let available = available(rule, &input)?;
    let portion = precise(|| Ok(expenditure.sub(paid)?.add(contribution)?))?;
    let formula = "recipients_expenditure - amount_paid + employee_contribution";
    let authority = "FBTAA s 22A(1)(b)(i) and s 136(1), recipients portion";
    let portion_step = step(
        "recipients_portion",
        authority,
        formula,
        &[expenditure, paid, contribution],
        portion.clone(),
    );
    let mut steps = contribution_steps(gross, &portion, &PORTION_ROWS)?;
    steps.insert(1, portion_step);
    let authority = if available.is_some() {
        "FBTAA s 22A(1) and s 62(1)"
    } else {
        "FBTAA s 22A(1)"
    };
    taxable_steps(steps, available, authority)
}
