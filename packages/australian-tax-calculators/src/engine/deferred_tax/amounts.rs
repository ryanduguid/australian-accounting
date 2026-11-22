// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `deferred_tax`'s movement, current tax, expense and para 81(c)(i)
//! reconciliation, each a step.

use super::super::{Result, Step, refuse};
use super::{Calc, Decimal, Facts, Rounding, Rule, cents, plain, rule_steps};

/// The value of the step named `id`, from any of `groups`.
fn value<'a>(groups: &[&'a [Step]], id: &str) -> &'a Decimal {
    &groups
        .iter()
        .flat_map(|g| g.iter())
        .find(|s| s.id == id)
        .expect("an earlier step")
        .value
}

const MOVEMENT: [Rule; 5] = [
    (
        "deferred_tax_expense",
        "AASB 112 paras 58 and 80: the year's movement in the net deferred tax balance, deferred tax expense (income) in profit or loss",
        "net_deferred_tax_liability - (opening_deferred_tax_liability - opening_deferred_tax_asset)",
    ),
    (
        "rate_change_effect",
        "AASB 112 paras 60(a) and 80(d): the opening recognised net taxable amount remeasured from the opening to the closing rate",
        "cents((opening_taxable_temporary_differences - opening_deductible_temporary_differences - opening_unused_tax_losses + opening_unrecognised_deductible) x (closing_rate - opening_rate))",
    ),
    (
        "recognition_change_effect",
        "AASB 112 paras 56, 60(b), 80(f) and 80(g): the change in deductible amounts with no deferred tax asset, at the closing rate",
        "cents((unrecognised_deductible - opening_unrecognised_deductible) x closing_rate)",
    ),
    (
        "origination_and_reversal",
        "AASB 112 para 80(c): the rest of the movement, from temporary differences arising and reversing",
        "deferred_tax_expense - rate_change_effect - recognition_change_effect",
    ),
    (
        "temporary_difference_movement",
        "AASB 112 para 5: the change in net taxable temporary differences less unused tax losses, before recognition",
        "taxable_temporary_differences - deductible_temporary_differences - unused_tax_losses - (opening_taxable_temporary_differences - opening_deductible_temporary_differences - opening_unused_tax_losses)",
    ),
];

/// The rate change on the opening recognised net amount and the change in
/// unrecognised amounts, each at the cent.
fn effects(f: &Facts, opening_net: &Decimal) -> Calc<(Decimal, Decimal)> {
    let recognised = opening_net.add(&f.opening_unrecognised)?;
    let rate_change = cents(recognised.mul(&f.closing_rate.sub(&f.opening_rate)?)?)?;
    let unrecognised = f.unrecognised.sub(&f.opening_unrecognised)?;
    Ok((rate_change, cents(unrecognised.mul(&f.closing_rate)?)?))
}

/// The movement steps, from the 4 difference totals and the balance steps.
pub(super) fn movement(f: &Facts, t: [&Decimal; 4], built: &[Step]) -> Result<Vec<Step>> {
    let [taxable, deductible, opening_taxable, opening_deductible] = t;
    let net = value(&[built], "net_deferred_tax_liability");
    let opening_liability = value(&[built], "opening_deferred_tax_liability");
    let opening_asset = value(&[built], "opening_deferred_tax_asset");
    let expense = plain(net.sub(&opening_liability.sub(opening_asset)?)?)?;
    let opening_net = opening_taxable
        .sub(opening_deductible)?
        .sub(&f.opening_losses)?;
    let (rate_change, recognition) = effects(f, &opening_net)?;
    let origination = plain(expense.sub(&rate_change)?.sub(&recognition)?)?;
    let closing_net = taxable.sub(deductible)?.sub(&f.losses)?;
    let change = plain(closing_net.sub(&opening_net)?)?;
    let rows = [
        (vec![net, opening_liability, opening_asset], expense.clone()),
        (
            vec![
                opening_taxable,
                opening_deductible,
                &f.opening_losses,
                &f.opening_unrecognised,
                &f.closing_rate,
                &f.opening_rate,
            ],
            rate_change.clone(),
        ),
        (
            vec![&f.unrecognised, &f.opening_unrecognised, &f.closing_rate],
            recognition.clone(),
        ),
        (vec![&expense, &rate_change, &recognition], origination),
        (
            vec![
                taxable,
                deductible,
                &f.losses,
                opening_taxable,
                opening_deductible,
                &f.opening_losses,
            ],
            change,
        ),
    ];
    Ok(rule_steps(&MOVEMENT, rows))
}

const CURRENT: [Rule; 3] = [
    (
        "current_tax_before_offsets",
        "AASB 112 paras 12 and 46; Income Tax Rates Act 1986 s 23(2): taxable income at the year's company rate, 25% (26% in 2020-21) for a base rate entity (s 23AA) and 30% otherwise",
        "cents(taxable_income x applicable_rate)",
    ),
    (
        "current_tax",
        "AASB 112 paras 5 and 46: less the non-refundable tax offsets the preparer has worked out",
        "current_tax_before_offsets - tax_offsets",
    ),
    (
        "income_tax_expense",
        "AASB 112 paras 5, 6 and 77: current tax plus deferred tax expense (income)",
        "current_tax + deferred_tax_expense",
    ),
];

/// Current tax and the income tax expense.
fn current(f: &Facts, steps: &[Step]) -> Result<Vec<Step>> {
    let before = cents(f.taxable_income.mul(&f.applicable)?)?;
    if f.offsets.gt(&before)? {
        return refuse(
            "deferred_tax.offsets_above_tax",
            "tax_offsets is more than current tax before offsets; refundable offsets and offsets carried forward are outside this worksheet.",
        );
    }
    let current = plain(before.sub(&f.offsets)?)?;
    let deferred = value(&[steps], "deferred_tax_expense");
    let expense = plain(current.add(deferred)?)?;
    let rows = [
        (vec![&f.taxable_income, &f.applicable], before.clone()),
        (vec![&before, &f.offsets], current.clone()),
        (vec![&current, deferred], expense),
    ];
    Ok(rule_steps(&CURRENT, rows))
}

const RECONCILIATION: [Rule; 7] = [
    (
        "tax_at_applicable_rate",
        "AASB 112 paras 81(c)(i) and 85: accounting profit at the applicable rate (the year's s 23(2) company rate)",
        "cents(accounting_profit x applicable_rate)",
    ),
    (
        "non_deductible_expenses_effect",
        "AASB 112 paras 81(c)(i) and 84: expenses not deductible in working out taxable income, at the applicable rate",
        "cents(non_deductible_expenses x applicable_rate)",
    ),
    (
        "non_assessable_income_effect",
        "AASB 112 paras 81(c)(i) and 84: income exempt from tax or otherwise not assessable, at the applicable rate",
        "0 - cents(non_assessable_income x applicable_rate)",
    ),
    (
        "deferred_rate_difference",
        "AASB 112 paras 47 and 81(c)(i): the year's temporary difference movement is measured at the closing rate, not the applicable rate",
        "cents(temporary_difference_movement x (closing_rate - applicable_rate))",
    ),
    (
        "tax_offsets_effect",
        "AASB 112 para 81(c)(i): tax offsets reduce current tax",
        "0 - tax_offsets",
    ),
    (
        "reconciliation_difference",
        "AASB 112 para 81(c)(i): income tax expense less the reconciling lines above, the rate change effect and the recognition change effect; nil, or a cent of rounding, when the inputs explain taxable income",
        "income_tax_expense - (tax_at_applicable_rate + non_deductible_expenses_effect + non_assessable_income_effect + deferred_rate_difference + rate_change_effect + recognition_change_effect + tax_offsets_effect)",
    ),
    (
        "taxable_income_difference",
        "AASB 112 para 5: taxable income less accounting profit adjusted for non-deductible expenses, non-assessable income and the temporary difference movement",
        "taxable_income - (accounting_profit + non_deductible_expenses - non_assessable_income - temporary_difference_movement)",
    ),
];

/// The expense less the sum of the reconciling lines.
fn left_over(expense: &Decimal, lines: &[&Decimal]) -> Calc<Decimal> {
    let mut explained = Decimal::from_int(0);
    for line in lines {
        explained = explained.add(line)?;
    }
    plain(expense.sub(&explained)?)
}

/// Taxable income less profit adjusted for the permanent differences and the
/// temporary difference movement.
fn unexplained(f: &Facts, change: &Decimal) -> Calc<Decimal> {
    let adjusted = f.profit.add(&f.non_deductible)?.sub(&f.non_assessable)?;
    plain(f.taxable_income.sub(&adjusted.sub(change)?)?)
}

/// The reconciliation lines, from the profit and the earlier steps.
fn reconciliation(f: &Facts, steps: &[&[Step]]) -> Result<Vec<Step>> {
    let r = &f.applicable;
    let at_rate = cents(f.profit.mul(r)?)?;
    let non_deductible = cents(f.non_deductible.mul(r)?)?;
    let non_assessable = plain(cents(f.non_assessable.mul(r)?)?.neg()?)?;
    let change = value(steps, "temporary_difference_movement");
    let rate_gap = cents(change.mul(&f.closing_rate.sub(r)?)?)?;
    let offsets = plain(f.offsets.neg()?)?;
    let rate_change = value(steps, "rate_change_effect");
    let recognition = value(steps, "recognition_change_effect");
    let expense = value(steps, "income_tax_expense");
    let lines = [
        &at_rate,
        &non_deductible,
        &non_assessable,
        &rate_gap,
        rate_change,
        recognition,
        &offsets,
    ];
    let difference = left_over(expense, &lines)?;
    let unexplained = unexplained(f, change)?;
    let mut operands = vec![expense];
    operands.extend(lines);
    let rows = [
        (vec![&f.profit, r], at_rate.clone()),
        (vec![&f.non_deductible, r], non_deductible.clone()),
        (vec![&f.non_assessable, r], non_assessable.clone()),
        (vec![change, &f.closing_rate, r], rate_gap.clone()),
        (vec![&f.offsets], offsets.clone()),
        (operands, difference),
        (
            vec![
                &f.taxable_income,
                &f.profit,
                &f.non_deductible,
                &f.non_assessable,
                change,
            ],
            unexplained,
        ),
    ];
    Ok(rule_steps(&RECONCILIATION, rows))
}

/// Current tax, the expense and the reconciliation, after the movement.
pub(super) fn expense(f: &Facts, steps: &[Step]) -> Result<Vec<Step>> {
    let mut out = current(f, steps)?;
    let lines = reconciliation(f, &[steps, &out])?;
    out.extend(lines);
    Ok(out)
}

/// AASB 112 para 86: the average effective tax rate, the income tax expense
/// over accounting profit to 4 decimal places half up; none for a nil profit.
pub(super) fn effective_rate(f: &Facts, steps: &[Step]) -> Result<Option<String>> {
    if f.profit.is_zero() {
        return Ok(None);
    }
    let expense = value(&[steps], "income_tax_expense");
    let ratio =
        super::super::precise(|| Ok(expense.div(&f.profit)?.quantize(-4, Rounding::HalfUp)?))?;
    Ok(Some(ratio.to_string()))
}
