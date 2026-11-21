// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `government_super_amounts`: the super co-contribution and the low income
//! superannuation tax offset (LISTO) under the Superannuation (Government
//! Co-contribution for Low Income Earners) Act 2003, and the spouse
//! contribution tax offset under ITAA 1997 Subdivision 290-D, each with the
//! conditions met or not and their provisions.
//!
//! The steps cite the Act's compilation 28, in force from 1 July 2018 to
//! 30 June 2027, ITAA 1997 compilation 268 (1 October 2026), the
//! co-contribution Regulations 2004 and 2022, the ATO's co-contribution income
//! thresholds (Table 25, last updated 27 April 2026), all read on 22 November
//! 2026. Total income and adjusted taxable income are `income-tests`'
//! measures; their definitions read the same in the compilations in force on
//! 1 July 2020 (docs/calculation-evidence.md#government-super-amounts).

use serde_json::{Value, json};

use super::income_tests::{SECTION_57A_FACTOR, measures, named, value};
use super::{
    Arg, Result, Step, d, derived_result, flag, int, money, precise, rate, refuse, scope, step,
};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "government_super_amounts";

/// The ATO's co-contribution income thresholds (Table 25): the lower income
/// threshold (s 10A(2)) and the higher, $15,000 above it (s 10A(3)(d)).
const THRESHOLDS: [(&str, &str, &str); 7] = [
    ("2020-21", "39837", "54837"),
    ("2021-22", "41112", "56112"),
    ("2022-23", "42016", "57016"),
    ("2023-24", "43445", "58445"),
    ("2024-25", "45400", "60400"),
    ("2025-26", "47488", "62488"),
    ("2026-27", "49293", "64293"),
];

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

/// The `income-tests` steps the amounts are worked from.
const INCOME_STEPS: [&str; 8] = [
    "taxable_income_without_fhss",
    "reportable_fringe_benefits_total",
    "adjusted_fringe_benefits_total",
    "total_net_investment_loss",
    "reportable_superannuation_contributions",
    "adjusted_taxable_income",
    "co_contribution_employer_contributions",
    "co_contribution_total_income",
];

const ACT: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003";

const TEST_TOTAL: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 8(3): for the 10% eligible income test, total income without the reduction for business deductions in s 8(2)";
const MATCHED: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 9(1)(d): 50% of the eligible personal superannuation contributions made in the income year";
const TAPER: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 10(1C) item 2: 3.333 cents for each dollar of total income above the lower income threshold, which the Commissioner publishes (s 10A(8); the ATO's Table 25)";
const MAXIMUM: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 10(1C): $500, less the taper between the thresholds";
const CO_CONTRIBUTION: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 ss 9, 10(1C) and 11: the matched amount up to the maximum, and at least $20, where every s 6(1) condition is met";
const PAID: &str = "Superannuation (Government Co-contribution for Low Income Earners) Regulations 2022 reg 15(2) (2004 Regulations reg 24(2) before 1 April 2022): an amount paid that is not a multiple of 5 cents is increased to the next multiple of 5 cents";
const LISTO_15: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 12E(2)(a) and (3): 15% of the concessional contributions for the financial year, without constitutionally protected fund amounts and other s 291-165(1)(c) amounts";
const LISTO: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 12E(2): 15% to the nearest cent (the ATO's 'Low income super tax offset'), at most $500 and at least $10, where every s 12C(1) condition is met";
const SPOUSE_INCOME: &str = "ITAA 1997 s 290-230(2)(c) and (5): the spouse's assessable income without the assessable FHSS released amount, reportable fringe benefits total and reportable employer superannuation contributions less (not below nil) excess concessional contributions";
const SPOUSE_LIMIT: &str =
    "ITAA 1997 s 290-235(1)(a): $3,000 reduced by the spouse's income above $37,000";
const SPOUSE_OFFSET: &str = "ITAA 1997 s 290-235: 18% of the lesser of the limit and the spouse contributions, at most $540, where every s 290-230 condition is met";

/// One condition of one amount, met, not met or not given.
struct Condition {
    amount: &'static str,
    id: &'static str,
    authority: &'static str,
    met: Option<bool>,
}

fn condition(
    amount: &'static str,
    id: &'static str,
    authority: &'static str,
    met: Option<bool>,
) -> Condition {
    Condition {
        amount,
        id,
        authority,
        met,
    }
}

/// Whether `amount` is payable: every condition met; not, if one is not met;
/// and refused if a condition not given would decide it.
fn payable(conditions: &[Condition], amount: &str) -> Result<bool> {
    let of_amount = || conditions.iter().filter(|c| c.amount == amount);
    if of_amount().any(|c| c.met == Some(false)) {
        return Ok(false);
    }
    match of_amount().find(|c| c.met.is_none()) {
        Some(c) => refuse(
            "government_super_amounts.fact_not_supplied",
            format!(
                "{} decides the {amount} and was not given ({}).",
                c.id, c.authority
            ),
        ),
        None => Ok(true),
    }
}

/// A balance or cap fact: true, false, or `null` for not established.
fn stated(value: &Arg, name: &str) -> Result<Option<bool>> {
    match value {
        Arg::Other(none) if none == "None" => Ok(None),
        other => flag(other, name).map(Some),
    }
}

/// `amount` increased to the next multiple of 5 cents.
fn up_to_5_cents(amount: &Decimal) -> Result<Decimal> {
    let twentieths = amount.mul(&int(20))?;
    let whole = twentieths.quantize(0, Rounding::Down)?;
    let whole = if whole.lt(&twentieths)? {
        whole.add(&int(1))?
    } else {
        whole
    };
    Ok(whole.div(&int(20))?)
}

fn lesser(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.lt(a)? { b.clone() } else { a.clone() })
}

fn greater(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.gt(a)? { b.clone() } else { a.clone() })
}

/// The co-contribution's steps and conditions.
fn co_contribution(
    values: &[(&'static str, Decimal)],
    facts: &Facts,
    conditions: &mut Vec<Condition>,
) -> Result<Vec<Step>> {
    let (lower, higher) = (&facts.lower, &facts.higher);
    let total = value(values, "co_contribution_total_income");
    let personal = &facts.personal;
    let amount = "co_contribution";
    conditions.extend([
        condition(amount, "personal_contributions_made", "Co-contribution Act s 6(1)(a): one or more eligible personal superannuation contributions in the income year (s 7)", Some(!personal.is_zero())),
        condition(amount, "eligible_income_10_percent", "Co-contribution Act s 6(1)(b) and s 8(3): 10% or more of total income, before business deductions, from activities that make the person an employee (s 6(2)) or from carrying on a business", Some(facts.ten_percent)),
        condition(amount, "below_higher_income_threshold", "Co-contribution Act s 6(1)(c): total income less than the higher income threshold", Some(total.lt(higher)?)),
        condition(amount, "return_lodged", "Co-contribution Act s 6(1)(d): an income tax return for the year is lodged", Some(facts.return_lodged)),
        condition(amount, "non_concessional_within_cap", "Co-contribution Act s 6(1)(da): non-concessional contributions for the financial year do not exceed the non-concessional contributions cap", facts.non_concessional),
        condition(amount, "total_super_balance_below_cap", "Co-contribution Act s 6(1)(db): total superannuation balance immediately before the start of the financial year less than the general transfer balance cap", facts.balance),
        condition(amount, "under_71", "Co-contribution Act s 6(1)(e): less than 71 years old at the end of the income year", Some(facts.under_71)),
        condition(amount, "no_temporary_visa", "Co-contribution Act s 6(1)(f): no temporary visa at any time in the income year", Some(true)),
    ]);
    let (matching, taper_rate, maximum, minimum) = (d("0.5"), d("0.03333"), int(500), int(20));
    let matched = personal.mul(&matching)?;
    let excess = greater(&int(0), &total.sub(lower)?)?;
    let taper = excess.mul(&taper_rate)?;
    let most = greater(&int(0), &maximum.sub(&taper)?)?;
    let mut steps = vec![
        step(
            "co_contribution_matched",
            MATCHED,
            "personal_contributions x matching_rate",
            &[personal, &matching],
            matched.clone(),
        ),
        step(
            "co_contribution_taper",
            TAPER,
            "max(0, co_contribution_total_income - lower_income_threshold) x taper_rate",
            &[total, lower, &taper_rate],
            taper.clone(),
        ),
        step(
            "co_contribution_maximum",
            MAXIMUM,
            "max(0, maximum_co_contribution - co_contribution_taper)",
            &[&maximum, &taper],
            most.clone(),
        ),
    ];
    let paid = if payable(conditions, amount)? {
        let got = greater(&minimum, &lesser(&matched, &most)?)?;
        steps.push(step(
            amount,
            CO_CONTRIBUTION,
            "max(minimum_co_contribution, min(co_contribution_matched, co_contribution_maximum))",
            &[&minimum, &matched, &most],
            got.clone(),
        ));
        got
    } else {
        steps.push(step(amount, CO_CONTRIBUTION, "0", &[], int(0)));
        int(0)
    };
    steps.push(step(
        "co_contribution_paid",
        PAID,
        "dollars_up(co_contribution x 20) / 20",
        &[&paid],
        up_to_5_cents(&paid)?,
    ));
    Ok(steps)
}

/// The low income superannuation tax offset's steps and conditions.
fn listo(
    values: &[(&'static str, Decimal)],
    facts: &Facts,
    conditions: &mut Vec<Condition>,
) -> Result<Vec<Step>> {
    let amount = "listo";
    let ati = value(values, "adjusted_taxable_income");
    let concessional = &facts.concessional;
    let limit = int(37000);
    conditions.extend([
        condition(amount, "concessional_contributions_made", "Co-contribution Act s 12C(1)(a): concessional contributions for the financial year", Some(!concessional.is_zero())),
        condition(amount, "adjusted_taxable_income_37000", "Co-contribution Act s 12C(1)(b): adjusted taxable income (A New Tax System (Family Assistance) Act 1999 Sch 3, disregarding cls 3 and 3A) does not exceed $37,000", Some(!ati.gt(&limit)?)),
        condition(amount, "eligible_income_10_percent", "Co-contribution Act s 12C(1)(c) and (3): the 10% eligible income test of s 6(1)(b)", Some(facts.ten_percent)),
        condition(amount, "no_temporary_visa", "Co-contribution Act s 12C(1)(d): the temporary visa test of s 6(1)(f)", Some(true)),
    ]);
    let (listo_rate, maximum, minimum) = (d("0.15"), int(500), int(10));
    let fifteen = concessional.mul(&listo_rate)?;
    let mut steps = vec![step(
        "listo_15_percent",
        LISTO_15,
        "concessional_contributions x listo_rate",
        &[concessional, &listo_rate],
        fifteen.clone(),
    )];
    let got = if payable(conditions, amount)? {
        let cents = fifteen.quantize(-2, Rounding::HalfUp)?;
        let got = lesser(&maximum, &greater(&minimum, &cents)?)?;
        steps.push(step(
            amount,
            LISTO,
            "min(listo_maximum, max(listo_minimum, cents(listo_15_percent)))",
            &[&maximum, &minimum, &fifteen],
            got.clone(),
        ));
        got
    } else {
        steps.push(step(amount, LISTO, "0", &[], int(0)));
        int(0)
    };
    steps.push(step(
        "listo_paid",
        PAID,
        "dollars_up(listo x 20) / 20",
        &[&got],
        up_to_5_cents(&got)?,
    ));
    Ok(steps)
}

/// The spouse contribution tax offset's steps and conditions.
fn spouse(inputs: &[(&str, &Arg)], conditions: &mut Vec<Condition>) -> Result<Vec<Step>> {
    let amount = "spouse_contribution_offset";
    let get = |key: &str| named(inputs, key);
    let [contributions, assessable, fhss, fringe, employer, excess] = [
        "spouse_contributions",
        "spouse_assessable_income",
        "spouse_assessable_fhss_released_amount",
        "spouse_reportable_fringe_benefits_total",
        "spouse_reportable_employer_super_contributions",
        "spouse_excess_concessional_contributions",
    ]
    .map(|key| money(get(key)));
    let (contributions, assessable, fhss, fringe, employer, excess) = (
        contributions?,
        assessable?,
        fhss?,
        fringe?,
        employer?,
        excess?,
    );
    for (part, why) in [
        (
            &fhss,
            "spouse_assessable_fhss_released_amount exceeds spouse_assessable_income: it is included in assessable income (ITAA 1997 s 313-20)",
        ),
        (
            &excess,
            "spouse_excess_concessional_contributions exceeds spouse_assessable_income: they are included in assessable income (ITAA 1997 s 291-15(a))",
        ),
    ] {
        if part.gt(&assessable)? {
            return refuse("government_super_amounts.fact_conflict", format!("{why}."));
        }
    }
    let income = assessable
        .sub(&fhss)?
        .add(&fringe)?
        .add(&greater(&int(0), &employer.sub(&excess)?)?)?;
    let (base, start, below, offset_rate) = (int(3000), int(37000), int(40000), d("0.18"));
    let limit = greater(
        &int(0),
        &base.sub(&greater(&int(0), &income.sub(&start)?)?)?,
    )?;
    conditions.extend([
        condition(amount, "spouse_contributions_made", "ITAA 1997 s 290-230(1): a contribution in the income year to a superannuation fund or RSA for your spouse's superannuation benefits", Some(!contributions.is_zero())),
        condition(amount, "spouse_income_below_40000", "ITAA 1997 s 290-230(2)(c): the spouse's income less than $40,000", Some(income.lt(&below)?)),
        condition(amount, "spouse_non_concessional_within_cap", "ITAA 1997 s 290-230(4A)(a): the spouse's non-concessional contributions for the financial year do not exceed the spouse's non-concessional contributions cap", stated(get("spouse_non_concessional_within_cap"), "spouse_non_concessional_within_cap")?),
        condition(amount, "spouse_total_super_balance_below_cap", "ITAA 1997 s 290-230(4A)(b): the spouse's total superannuation balance immediately before the start of the financial year less than the general transfer balance cap", stated(get("spouse_total_super_balance_below_cap"), "spouse_total_super_balance_below_cap")?),
    ]);
    let mut steps = vec![
        step(
            "spouse_income",
            SPOUSE_INCOME,
            "spouse_assessable_income - spouse_assessable_fhss_released_amount + spouse_reportable_fringe_benefits_total + max(0, spouse_reportable_employer_super_contributions - spouse_excess_concessional_contributions)",
            &[&assessable, &fhss, &fringe, &employer, &excess],
            income.clone(),
        ),
        step(
            "spouse_contribution_limit",
            SPOUSE_LIMIT,
            "max(0, spouse_contribution_base - max(0, spouse_income - spouse_income_taper_start))",
            &[&base, &income, &start],
            limit.clone(),
        ),
    ];
    if payable(conditions, amount)? {
        let got = lesser(&limit, &contributions)?.mul(&offset_rate)?;
        steps.push(step(
            amount,
            SPOUSE_OFFSET,
            "min(spouse_contribution_limit, spouse_contributions) x spouse_offset_rate",
            &[&limit, &contributions, &offset_rate],
            got,
        ));
    } else {
        steps.push(step(amount, SPOUSE_OFFSET, "0", &[], int(0)));
    }
    Ok(steps)
}

/// The person's facts every amount reads.
struct Facts {
    lower: Decimal,
    higher: Decimal,
    personal: Decimal,
    concessional: Decimal,
    ten_percent: bool,
    return_lodged: bool,
    under_71: bool,
    non_concessional: Option<bool>,
    balance: Option<bool>,
}

/// The super co-contribution, LISTO and, where a spouse contribution is given,
/// the spouse contribution tax offset for one full-year resident and income
/// year, from `inputs` by name: `income-tests`' components and `residency`,
/// the person's contributions and facts, `year`, `scope_confirmed` and the
/// [`SPOUSE_INPUTS`] if any.
pub fn government_super_amounts(inputs: &[(&str, &Arg)]) -> Result<Value> {
    let get = |key: &str| named(inputs, key);
    let year = scope(get("scope_confirmed"), get("year"), KIND)?;
    let given: Vec<&str> = SPOUSE_INPUTS
        .into_iter()
        .filter(|key| inputs.iter().any(|(k, _)| k == key))
        .collect();
    if let Some(missing) = SPOUSE_INPUTS
        .iter()
        .find(|key| !given.is_empty() && !given.contains(key))
    {
        return refuse(
            "input.missing_field",
            format!("{missing} is required with the other spouse inputs."),
        );
    }
    if flag(get("temporary_visa_held"), "temporary_visa_held")? {
        return refuse(
            "government_super_amounts.temporary_visa",
            format!(
                "{ACT} s 6(1)(f) and s 12C(1)(d): the exceptions for New Zealand citizens and visas prescribed under s 20AA(2) of the Superannuation (Unclaimed Money and Lost Members) Act 1999 are not worked out here."
            ),
        );
    }
    let (values, income_steps) = measures(inputs)?;
    let (_, lower, higher) = THRESHOLDS
        .iter()
        .find(|(y, ..)| *y == year)
        .expect("a supported year");
    let facts_in = (
        money(get("eligible_income"))?,
        money(get("personal_contributions"))?,
        money(get("concessional_contributions"))?,
    );
    let (eligible, personal, concessional) = facts_in;
    let return_lodged = flag(get("return_lodged"), "return_lodged")?;
    let under_71 = flag(get("under_71"), "under_71")?;
    let non_concessional = stated(
        get("non_concessional_within_cap"),
        "non_concessional_within_cap",
    )?;
    let balance = stated(
        get("total_super_balance_below_cap"),
        "total_super_balance_below_cap",
    )?;
    precise(|| {
        let total = value(&values, "co_contribution_total_income");
        let test_total = total.add(value(&values, "business_deductions"))?;
        if eligible.gt(&test_total)? {
            return refuse(
                "government_super_amounts.fact_conflict",
                "eligible_income exceeds total income before business deductions, of which it is a part (s 8(3)).",
            );
        }
        if test_total.is_zero() && !(personal.is_zero() && concessional.is_zero()) {
            return refuse(
                "government_super_amounts.no_total_income",
                "Total income is nil, so the 10% eligible income test (s 6(1)(b)) has no share to measure.",
            );
        }
        let facts = Facts {
            lower: d(lower),
            higher: d(higher),
            personal,
            concessional,
            ten_percent: !eligible.mul(&int(10))?.lt(&test_total)?,
            return_lodged,
            under_71,
            non_concessional,
            balance,
        };
        let mut steps: Vec<Step> = income_steps
            .into_iter()
            .filter(|s| INCOME_STEPS.contains(&s.static_id()))
            .collect();
        let business = value(&values, "business_deductions");
        steps.push(step(
            "eligible_income_test_total",
            TEST_TOTAL,
            "co_contribution_total_income + business_deductions",
            &[total, business],
            test_total.clone(),
        ));
        let mut conditions = Vec::new();
        steps.extend(co_contribution(&values, &facts, &mut conditions)?);
        steps.extend(listo(&values, &facts, &mut conditions)?);
        if !given.is_empty() {
            steps.extend(spouse(inputs, &mut conditions)?);
        }
        let rates = vec![
            rate("lower_income_threshold", lower),
            rate("higher_income_threshold", higher),
            rate("matching_rate", "0.5"),
            rate("taper_rate", "0.03333"),
            rate("maximum_co_contribution", "500"),
            rate("minimum_co_contribution", "20"),
            rate("listo_rate", "0.15"),
            rate("listo_income_limit", "37000"),
            rate("listo_maximum", "500"),
            rate("listo_minimum", "10"),
            rate("spouse_offset_rate", "0.18"),
            rate("spouse_contribution_base", "3000"),
            rate("spouse_income_taper_start", "37000"),
            rate("spouse_income_limit", "40000"),
            rate("section_57a_factor", SECTION_57A_FACTOR),
        ];
        let mut out = derived_result(KIND, &year, steps, rates)?;
        out["conditions"] = json!(
            conditions
                .iter()
                .map(|c| json!({"amount": c.amount, "condition": c.id, "authority": c.authority, "met": c.met}))
                .collect::<Vec<_>>()
        );
        Ok(out)
    })
}
