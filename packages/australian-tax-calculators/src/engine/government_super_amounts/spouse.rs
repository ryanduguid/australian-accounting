// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The spouse contribution tax offset of `government_super_amounts`: ITAA
//! 1997 ss 290-230 and 290-235, compilation 268, read on 22 November 2026.

use super::super::income_tests::named;
use super::super::{Arg, Result, Step, int, money, refuse, steps_from};
use super::{Condition, amount_step, fixed, greater, lesser, payable, stated};
use crate::decimal::Decimal;

/// The spouse inputs: all given, for a spouse contribution, or none.
pub const SPOUSE_INPUTS: [&str; 8] = [
    "spouse_contributions",
    "spouse_assessable_income",
    "spouse_assessable_fhss_released_amount",
    "spouse_reportable_fringe_benefits_total",
    "spouse_reportable_employer_super_contributions",
    "spouse_excess_concessional_contributions",
    "spouse_non_concessional_within_cap",
    "spouse_total_super_balance_below_cap",
];

const SPOUSE_INCOME: &str = "ITAA 1997 s 290-230(2)(c) and (5): the spouse's assessable income without the assessable FHSS released amount, reportable fringe benefits total and reportable employer superannuation contributions less (not below nil) excess concessional contributions";
const SPOUSE_LIMIT: &str =
    "ITAA 1997 s 290-235(1)(a): $3,000 reduced by the spouse's income above $37,000";
const SPOUSE_OFFSET: &str = "ITAA 1997 s 290-235: 18% of the lesser of the limit and the spouse contributions, at most $540, where every s 290-230 condition is met";

const SPOUSE_CONDITIONS: [(&str, &str); 4] = [
    (
        "spouse_contributions_made",
        "ITAA 1997 s 290-230(1): a contribution in the income year to a superannuation fund or RSA for your spouse's superannuation benefits",
    ),
    (
        "spouse_income_below_40000",
        "ITAA 1997 s 290-230(2)(c): the spouse's income less than $40,000",
    ),
    (
        "spouse_non_concessional_within_cap",
        "ITAA 1997 s 290-230(4A)(a): the spouse's non-concessional contributions for the financial year do not exceed the spouse's non-concessional contributions cap",
    ),
    (
        "spouse_total_super_balance_below_cap",
        "ITAA 1997 s 290-230(4A)(b): the spouse's total superannuation balance immediately before the start of the financial year less than the general transfer balance cap",
    ),
];

/// The spouse's contributions and income components, each checked against
/// the others, then the spouse's income (s 290-230(2)(c)).
fn spouse_income(inputs: &[(&str, &Arg)]) -> Result<[Decimal; 7]> {
    let parts = [
        "spouse_contributions",
        "spouse_assessable_income",
        "spouse_assessable_fhss_released_amount",
        "spouse_reportable_fringe_benefits_total",
        "spouse_reportable_employer_super_contributions",
        "spouse_excess_concessional_contributions",
    ]
    .map(|key| money(named(inputs, key)));
    let [a, b, c, e, f, g] = parts;
    let parts = [a?, b?, c?, e?, f?, g?];
    for (part, why) in [
        (
            &parts[2],
            "spouse_assessable_fhss_released_amount exceeds spouse_assessable_income: it is included in assessable income (ITAA 1997 s 313-20)",
        ),
        (
            &parts[5],
            "spouse_excess_concessional_contributions exceeds spouse_assessable_income: they are included in assessable income (ITAA 1997 s 291-15(a))",
        ),
    ] {
        if part.gt(&parts[1])? {
            return refuse("government_super_amounts.fact_conflict", format!("{why}."));
        }
    }
    let [made, assessable, fhss, fringe, employer, excess] = parts;
    let reportable = greater(&int(0), &employer.sub(&excess)?)?;
    let income = assessable.sub(&fhss)?.add(&fringe)?.add(&reportable)?;
    Ok([made, assessable, fhss, fringe, employer, excess, income])
}

/// The spouse offset's working before the amount: (id, authority, formula).
const SPOUSE_STEPS: [(&str, &str, &str); 2] = [
    (
        "spouse_income",
        SPOUSE_INCOME,
        "spouse_assessable_income - spouse_assessable_fhss_released_amount + spouse_reportable_fringe_benefits_total + max(0, spouse_reportable_employer_super_contributions - spouse_excess_concessional_contributions)",
    ),
    (
        "spouse_contribution_limit",
        SPOUSE_LIMIT,
        "max(0, spouse_contribution_base - max(0, spouse_income - spouse_income_taper_start))",
    ),
];
const SPOUSE_FORMULA: &str =
    "min(spouse_contribution_limit, spouse_contributions) x spouse_offset_rate";

const OFFSET: &str = "spouse_contribution_offset";

/// The spouse contribution tax offset's steps.
pub(super) fn spouse(
    inputs: &[(&str, &Arg)],
    conditions: &mut Vec<Condition>,
) -> Result<Vec<Step>> {
    let [made, assessable, fhss, fringe, employer, excess, income] = spouse_income(inputs)?;
    let [base, start, below, offset_rate] = [
        "spouse_contribution_base",
        "spouse_income_taper_start",
        "spouse_income_limit",
        "spouse_offset_rate",
    ]
    .map(fixed);
    let limit = greater(
        &int(0),
        &base.sub(&greater(&int(0), &income.sub(&start)?)?)?,
    );
    let limit = limit?;
    let met = [
        Some(!made.is_zero()),
        Some(income.lt(&below)?),
        stated(inputs, "spouse_non_concessional_within_cap")?,
        stated(inputs, "spouse_total_super_balance_below_cap")?,
    ];
    let ok = payable(conditions, OFFSET, &SPOUSE_CONDITIONS, met)?;
    let got = lesser(&limit, &made)?.mul(&offset_rate)?;
    let (amount, _) = amount_step(
        ok,
        OFFSET,
        SPOUSE_OFFSET,
        SPOUSE_FORMULA,
        &[&limit, &made, &offset_rate],
        got,
    );
    let mut steps = steps_from(
        &SPOUSE_STEPS,
        [
            (
                &[&assessable, &fhss, &fringe, &employer, &excess],
                income.clone(),
            ),
            (&[&base, &income, &start], limit),
        ],
    );
    steps.push(amount);
    Ok(steps)
}

/// The spouse inputs given: all or none.
pub(super) fn spouse_given(inputs: &[(&str, &Arg)]) -> Result<bool> {
    let given = SPOUSE_INPUTS.map(|key| inputs.iter().any(|(k, _)| *k == key));
    match SPOUSE_INPUTS.iter().zip(given).find(|(_, g)| !g) {
        Some((missing, _)) if given.contains(&true) => refuse(
            "input.missing_field",
            format!("{missing} is required with the other spouse inputs."),
        ),
        _ => Ok(given[0]),
    }
}
