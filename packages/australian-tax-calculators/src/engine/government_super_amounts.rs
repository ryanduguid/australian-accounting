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

use super::income_tests::{SECTION_57A_FACTOR, Values, measures, named, value};
use super::{
    Arg, Result, Step, d, derived_result, flag, int, money, precise, rate, refuse, scope, step,
    steps_from,
};
use crate::decimal::{Decimal, Rounding};

mod spouse;
pub use spouse::SPOUSE_INPUTS;
use spouse::{spouse, spouse_given};

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
const LISTO_15: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 12E(2)(a) and (3): 15% of the concessional contributions for the financial year; a person with contributions or amounts s 12E(3) disregards (constitutionally protected funds, ITAA 1997 s 291-370(1)(a), and other s 291-165(1)(c) amounts) is outside scope";
const LISTO: &str = "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 12E(2): 15% to the nearest cent (the ATO's 'Low income super tax offset'), at most $500 and at least $10, where every s 12C(1) condition is met";

/// The 10% eligible income test's condition, which nil total income leaves
/// unmeasured.
const TEN_PERCENT: &str = "eligible_income_10_percent";

/// Each amount's conditions: (id, authority), in the order of their facts.
const CO_CONDITIONS: [(&str, &str); 8] = [
    (
        "personal_contributions_made",
        "Co-contribution Act s 6(1)(a): one or more eligible personal superannuation contributions in the income year (s 7)",
    ),
    (
        TEN_PERCENT,
        "Co-contribution Act s 6(1)(b) and s 8(3): 10% or more of total income, before business deductions, from activities that make the person an employee (s 6(2)) or from carrying on a business",
    ),
    (
        "below_higher_income_threshold",
        "Co-contribution Act s 6(1)(c): total income less than the higher income threshold",
    ),
    (
        "return_lodged",
        "Co-contribution Act s 6(1)(d): an income tax return for the year is lodged",
    ),
    (
        "non_concessional_within_cap",
        "Co-contribution Act s 6(1)(da): non-concessional contributions for the financial year do not exceed the non-concessional contributions cap",
    ),
    (
        "total_super_balance_below_cap",
        "Co-contribution Act s 6(1)(db): total superannuation balance immediately before the start of the financial year less than the general transfer balance cap",
    ),
    (
        "under_71",
        "Co-contribution Act s 6(1)(e): less than 71 years old at the end of the income year",
    ),
    (
        "no_temporary_visa",
        "Co-contribution Act s 6(1)(f): no temporary visa at any time in the income year",
    ),
];
const LISTO_CONDITIONS: [(&str, &str); 4] = [
    (
        "concessional_contributions_made",
        "Co-contribution Act s 12C(1)(a): concessional contributions for the financial year",
    ),
    (
        "adjusted_taxable_income_37000",
        "Co-contribution Act s 12C(1)(b): adjusted taxable income (A New Tax System (Family Assistance) Act 1999 Sch 3, disregarding cls 3 and 3A) does not exceed $37,000",
    ),
    (
        TEN_PERCENT,
        "Co-contribution Act s 12C(1)(c) and (3): the 10% eligible income test of s 6(1)(b)",
    ),
    (
        "no_temporary_visa",
        "Co-contribution Act s 12C(1)(d): the temporary visa test of s 6(1)(f)",
    ),
];

/// The rates every result shows besides the year's thresholds.
const RATES: [(&str, &str); 13] = [
    ("matching_rate", "0.5"),
    ("taper_rate", "0.03333"),
    ("maximum_co_contribution", "500"),
    ("minimum_co_contribution", "20"),
    ("listo_rate", "0.15"),
    ("listo_income_limit", "37000"),
    ("listo_maximum", "500"),
    ("listo_minimum", "10"),
    ("spouse_offset_rate", "0.18"),
    ("spouse_contribution_base", "3000"),
    ("spouse_income_taper_start", "37000"),
    ("spouse_income_limit", "40000"),
    ("section_57a_factor", SECTION_57A_FACTOR),
];

/// The value of the rate `key` in [`RATES`].
fn fixed(key: &str) -> Decimal {
    d(RATES.iter().find(|(k, _)| *k == key).expect("a rate").1)
}

/// One condition of one amount, met, not met or not given.
struct Condition {
    amount: &'static str,
    id: &'static str,
    authority: &'static str,
    met: Option<bool>,
}

/// Records `amount`'s conditions from `table` and whether each is met, and
/// says whether the amount is payable: every condition met; not, if one is
/// not met; and refused if a condition not given would decide it.
fn payable<const N: usize>(
    conditions: &mut Vec<Condition>,
    amount: &'static str,
    table: &[(&'static str, &'static str); N],
    met: [Option<bool>; N],
) -> Result<bool> {
    conditions.extend(
        table
            .iter()
            .zip(met)
            .map(|((id, authority), met)| Condition {
                amount,
                id,
                authority,
                met,
            }),
    );
    if met.contains(&Some(false)) {
        return Ok(false);
    }
    match table.iter().zip(met).find(|(_, met)| met.is_none()) {
        Some(((TEN_PERCENT, _), _)) => refuse(
            "government_super_amounts.no_total_income",
            format!(
                "Total income is nil, so the 10% eligible income test (s 6(1)(b)) has no share to measure, and it decides the {amount}."
            ),
        ),
        Some(((id, authority), _)) => refuse(
            "government_super_amounts.fact_not_supplied",
            format!("{id} decides the {amount} and was not given ({authority})."),
        ),
        None => Ok(true),
    }
}

/// The amount's step: `formula` on `operands` giving `value` when payable,
/// and nil otherwise.
fn amount_step(
    payable: bool,
    id: &'static str,
    authority: &'static str,
    formula: &str,
    operands: &[&Decimal],
    value: Decimal,
) -> (Step, Decimal) {
    if payable {
        (step(id, authority, formula, operands, value.clone()), value)
    } else {
        (step(id, authority, "0", &[], int(0)), int(0))
    }
}

/// The step `{of}_paid`: `amount`, the step `of`, increased to the next
/// multiple of 5 cents.
fn paid_step(of: &str, amount: &Decimal) -> Result<Step> {
    let (id, formula) = (format!("{of}_paid"), format!("dollars_up({of} x 20) / 20"));
    let twentieths = amount.mul(&int(20))?;
    let whole = twentieths.quantize(0, Rounding::Down)?;
    let whole = if whole.lt(&twentieths)? {
        whole.add(&int(1))?
    } else {
        whole
    };
    Ok(step(id, PAID, &formula, &[amount], whole.div(&int(20))?))
}

/// A balance or cap fact: true, false, or `null` for not established.
fn stated(inputs: &[(&str, &Arg)], name: &str) -> Result<Option<bool>> {
    match named(inputs, name) {
        Arg::Other(none) if none == "None" => Ok(None),
        other => flag(other, name).map(Some),
    }
}

fn lesser(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.lt(a)? { b.clone() } else { a.clone() })
}

fn greater(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.gt(a)? { b.clone() } else { a.clone() })
}

/// The co-contribution's working before the amount: (id, authority, formula).
const CO_STEPS: [(&str, &str, &str); 3] = [
    (
        "co_contribution_matched",
        MATCHED,
        "personal_contributions x matching_rate",
    ),
    (
        "co_contribution_taper",
        TAPER,
        "max(0, co_contribution_total_income - lower_income_threshold) x taper_rate",
    ),
    (
        "co_contribution_maximum",
        MAXIMUM,
        "max(0, maximum_co_contribution - co_contribution_taper)",
    ),
];
const CO_FORMULA: &str =
    "max(minimum_co_contribution, min(co_contribution_matched, co_contribution_maximum))";

/// The co-contribution's steps.
fn co_contribution(
    values: &Values,
    facts: &Facts,
    conditions: &mut Vec<Condition>,
) -> Result<Vec<Step>> {
    let total = value(values, "co_contribution_total_income");
    let (lower, personal) = (&facts.lower, &facts.personal);
    let met = [
        Some(!personal.is_zero()),
        facts.ten_percent,
        Some(total.lt(&facts.higher)?),
        Some(facts.return_lodged),
        facts.non_concessional,
        facts.balance,
        Some(facts.under_71),
        Some(true),
    ];
    let ok = payable(conditions, "co_contribution", &CO_CONDITIONS, met)?;
    let [matching, taper_rate, maximum, minimum] = [
        "matching_rate",
        "taper_rate",
        "maximum_co_contribution",
        "minimum_co_contribution",
    ]
    .map(fixed);
    let matched = personal.mul(&matching)?;
    let taper = greater(&int(0), &total.sub(lower)?)?.mul(&taper_rate)?;
    let most = greater(&int(0), &maximum.sub(&taper)?)?;
    let got = greater(&minimum, &lesser(&matched, &most)?)?;
    let (amount, paid) = amount_step(
        ok,
        "co_contribution",
        CO_CONTRIBUTION,
        CO_FORMULA,
        &[&minimum, &matched, &most],
        got,
    );
    let mut steps = steps_from(
        &CO_STEPS,
        [
            (&[personal, &matching], matched.clone()),
            (&[total, lower, &taper_rate], taper.clone()),
            (&[&maximum, &taper], most.clone()),
        ],
    );
    steps.extend([amount, paid_step("co_contribution", &paid)?]);
    Ok(steps)
}

/// The low income superannuation tax offset's steps.
fn listo(values: &Values, facts: &Facts, conditions: &mut Vec<Condition>) -> Result<Vec<Step>> {
    let ati = value(values, "adjusted_taxable_income");
    let concessional = &facts.concessional;
    let [listo_rate, limit, maximum, minimum] = [
        "listo_rate",
        "listo_income_limit",
        "listo_maximum",
        "listo_minimum",
    ]
    .map(fixed);
    let met = [
        Some(!concessional.is_zero()),
        Some(!ati.gt(&limit)?),
        facts.ten_percent,
        Some(true),
    ];
    let ok = payable(conditions, "listo", &LISTO_CONDITIONS, met)?;
    let fifteen = concessional.mul(&listo_rate)?;
    let got = lesser(
        &maximum,
        &greater(&minimum, &fifteen.quantize(-2, Rounding::HalfUp)?)?,
    );
    let got = got?;
    let (amount, paid) = amount_step(
        ok,
        "listo",
        LISTO,
        "min(listo_maximum, max(listo_minimum, cents(listo_15_percent)))",
        &[&maximum, &minimum, &fifteen],
        got,
    );
    Ok(vec![
        step(
            "listo_15_percent",
            LISTO_15,
            "concessional_contributions x listo_rate",
            &[concessional, &listo_rate],
            fifteen.clone(),
        ),
        amount,
        paid_step("listo", &paid)?,
    ])
}

/// The person's facts every amount reads.
struct Facts {
    lower: Decimal,
    higher: Decimal,
    personal: Decimal,
    concessional: Decimal,
    ten_percent: Option<bool>,
    return_lodged: bool,
    under_71: bool,
    non_concessional: Option<bool>,
    balance: Option<bool>,
}

/// The 10% eligible income test (s 6(1)(b)) on `test_total`, total income
/// before business deductions (s 8(3)); none when total income is nil, as
/// there is then no share to measure.
fn ten_percent(eligible: &Decimal, test_total: &Decimal) -> Result<Option<bool>> {
    if eligible.gt(test_total)? {
        return refuse(
            "government_super_amounts.fact_conflict",
            "eligible_income exceeds total income before business deductions, of which it is a part (s 8(3)).",
        );
    }
    if test_total.is_zero() {
        return Ok(None);
    }
    Ok(Some(!eligible.mul(&int(10))?.lt(test_total)?))
}

/// The person's facts for `year`, from the income measures in `values`; the
/// 10% eligible income test is worked here, on total income before business
/// deductions (s 8(3)), which is the step returned beside them.
fn facts(inputs: &[(&str, &Arg)], values: &Values, year: &str) -> Result<(Facts, Step)> {
    let get = |key: &str| named(inputs, key);
    let (_, lower, higher) = THRESHOLDS
        .iter()
        .find(|(y, ..)| *y == year)
        .expect("a supported year");
    let [eligible, personal, concessional] = [
        "eligible_income",
        "personal_contributions",
        "concessional_contributions",
    ]
    .map(|key| money(get(key)));
    let (eligible, personal, concessional) = (eligible?, personal?, concessional?);
    let (lodged, under_71) = (
        flag(get("return_lodged"), "return_lodged")?,
        flag(get("under_71"), "under_71")?,
    );
    let non_concessional = stated(inputs, "non_concessional_within_cap")?;
    let balance = stated(inputs, "total_super_balance_below_cap")?;
    let (total, business) = (
        value(values, "co_contribution_total_income"),
        value(values, "business_deductions"),
    );
    let test_total = total.add(business)?;
    let ten_percent = ten_percent(&eligible, &test_total)?;
    let step = step(
        "eligible_income_test_total",
        TEST_TOTAL,
        "co_contribution_total_income + business_deductions",
        &[total, business],
        test_total,
    );
    let (lower, higher) = (d(lower), d(higher));
    let facts = Facts {
        lower,
        higher,
        personal,
        concessional,
        ten_percent,
        return_lodged: lodged,
        under_71,
        non_concessional,
        balance,
    };
    Ok((facts, step))
}

/// The super co-contribution, LISTO and, where a spouse contribution is given,
/// the spouse contribution tax offset for one full-year resident and income
/// year, from `inputs` by name: `income-tests`' components and `residency`,
/// the person's contributions and facts, `year`, `scope_confirmed` and the
/// [`SPOUSE_INPUTS`] if any.
pub fn government_super_amounts(inputs: &[(&str, &Arg)]) -> Result<Value> {
    let year = scope(
        named(inputs, "scope_confirmed"),
        named(inputs, "year"),
        KIND,
    )?;
    let with_spouse = spouse_given(inputs)?;
    if flag(named(inputs, "temporary_visa_held"), "temporary_visa_held")? {
        return refuse(
            "government_super_amounts.temporary_visa",
            format!(
                "{ACT} s 6(1)(f) and s 12C(1)(d): the exceptions for New Zealand citizens and visas prescribed under s 20AA(2) of the Superannuation (Unclaimed Money and Lost Members) Act 1999 are not worked out here."
            ),
        );
    }
    let (values, income_steps) = measures(inputs)?;
    precise(|| {
        let (facts, test_step) = facts(inputs, &values, &year)?;
        let mut steps: Vec<Step> = income_steps
            .into_iter()
            .filter(|s| INCOME_STEPS.contains(&s.static_id()))
            .collect();
        steps.push(test_step);
        let mut conditions = Vec::new();
        steps.extend(co_contribution(&values, &facts, &mut conditions)?);
        steps.extend(listo(&values, &facts, &mut conditions)?);
        if with_spouse {
            steps.extend(spouse(inputs, &mut conditions)?);
        }
        let (_, lower, higher) = THRESHOLDS
            .iter()
            .find(|(y, ..)| *y == year)
            .expect("a supported year");
        let mut rates = vec![
            rate("lower_income_threshold", lower),
            rate("higher_income_threshold", higher),
        ];
        rates.extend(RATES.iter().map(|(k, v)| rate(k, v)));
        let mut out = derived_result(KIND, &year, steps, rates)?;
        let shown = conditions.iter().map(|c| json!({"amount": c.amount, "condition": c.id, "authority": c.authority, "met": c.met}));
        out["conditions"] = json!(shown.collect::<Vec<_>>());
        Ok(out)
    })
}
