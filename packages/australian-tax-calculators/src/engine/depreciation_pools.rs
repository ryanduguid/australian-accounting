// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The pools and capital works beside the single-asset `depreciation`
//! worksheet: a low-value pool (ITAA 1997 ss 40-425 to 40-445), a small
//! business entity's general small business pool (ss 328-185 to 328-215, with
//! the low pool value thresholds of IT(TP)A 1997 ss 328-180(6) and 328-181(5)
//! and Act No. 71 of 2026 Sch 2) and a capital works deduction (Division 43,
//! ss 43-210 and 43-215). The sections are from the ATO's legal database and
//! the Act from its published text, all read on 22 November 2026.

use serde_json::{Value, json};

use super::{
    Arg, Result, Step, cents, d, date_arg, derived_result, finite_between, flag, int, money,
    precise, py_max, py_min, rate, refuse, scope, step,
};
use crate::dates;
use crate::decimal::Decimal;

/// The years whose pool write-off is temporary full expensing's (IT(TP)A s 328-181(5)).
const FULL_EXPENSING_YEARS: [&str; 3] = ["2020-21", "2021-22", "2022-23"];

/// A step's id, authority and formula.
type Rule = (&'static str, &'static str, &'static str);

/// The steps of `rules`, each with its operands and value in turn.
fn steps(rules: &[Rule], rows: Vec<(Vec<&Decimal>, Decimal)>) -> Vec<Step> {
    assert_eq!(rules.len(), rows.len(), "a row for each rule");
    rules
        .iter()
        .zip(rows)
        .map(|((id, authority, formula), (operands, value))| {
            step(*id, authority, formula, &operands, value)
        })
        .collect()
}

/// `out` with `note` added to its warnings in a temporary full expensing year.
pub(super) fn full_expensing_warning(mut out: Value, year: &str, note: &str) -> Value {
    if FULL_EXPENSING_YEARS.contains(&year)
        && let Some(warnings) = out["warnings"].as_array_mut()
    {
        warnings.push(json!(note));
    }
    out
}

const DEPRECIATION_FULL_EXPENSING: &str = "2020-21 to 2022-23 are temporary full expensing years: IT(TP)A 1997 Subdivision 40-BB may apply to an asset held, and first used or installed ready for use, by 30 June 2023 (s 40-150), subject to the entity tests in ss 40-155 and 40-157, the exclusions in ss 40-165 and 40-167 and any choice under s 40-190, and where it applies it works out the decline in value instead of this worksheet (ss 40-145, 40-160 and 40-170); this worksheet does not decide whether it applied.";

const BACKING_BUSINESS: &str = "Backing business investment (IT(TP)A 1997 Subdivision 40-BA) may apply to an asset held, and first used or installed ready for use, from 12 March 2020 to 30 June 2021, subject to ss 40-120 and 40-125 (including s 40-125(7)'s exclusion of an asset another entity held when it was first used) and any choice under s 40-137, and where it applies it changes the start year decline in value (s 40-130); this worksheet does not model it.";

const ADJUSTED_PRIME_COST: &str = "Where backing business investment or temporary full expensing worked out the asset's decline in value for an earlier year, a later prime cost year needs the adjusted formula of ITAA 1997 s 40-75(3) (IT(TP)A 1997 ss 40-135(3) and 40-180(2)), which this worksheet does not apply.";

/// The single-asset `depreciation` worksheet's flags: backing business
/// investment in a 2020-21 start year, the adjusted prime cost formula a later
/// year may need, and the temporary full expensing years.
pub(super) fn depreciation_flags(
    mut out: Value,
    year: &str,
    prime_cost: bool,
    later_year: bool,
) -> Value {
    let warnings = out["warnings"].as_array_mut().expect("warnings");
    if year == "2020-21" && !later_year {
        warnings.push(json!(BACKING_BUSINESS));
    }
    if prime_cost && later_year {
        warnings.push(json!(ADJUSTED_PRIME_COST));
    }
    full_expensing_warning(out, year, DEPRECIATION_FULL_EXPENSING)
}

const LVP_FULL_EXPENSING: &str = "2020-21 to 2022-23 are temporary full expensing years: this worksheet does not decide whether temporary full expensing applied to an asset, and only amounts allocated to the low-value pool belong in its inputs.";

const LVP_RULES: [Rule; 6] = [
    (
        "new_assets_decline",
        "ITAA 1997 s 40-440(1) steps 1 and 2: 18 3/4% of the taxable use percentage of the cost of low-cost assets allocated for the year and of second element amounts for the year",
        "0.1875 x (low_cost_assets + second_element_costs)",
    ),
    (
        "pool_decline",
        "ITAA 1997 s 40-440(1) step 3: 37 1/2% of the closing pool balance for the previous year and the taxable use percentage of the opening adjustable values of low-value assets allocated for the year",
        "0.375 x (previous_closing_pool_balance + low_value_assets)",
    ),
    (
        "decline_in_value",
        "ITAA 1997 s 40-440(1) step 4: the decline in value of the assets in the pool, the deduction (s 40-25(1)), the taxable use percentage being in each amount already",
        "new_assets_decline + pool_decline",
    ),
    (
        "closing_before_disposals",
        "ITAA 1997 s 40-440(2): the previous closing pool balance plus the year's amounts allocated, less the decline in value",
        "previous_closing_pool_balance + low_cost_assets + low_value_assets + second_element_costs - cents(decline_in_value)",
    ),
    (
        "closing_pool_balance",
        "ITAA 1997 s 40-445(1): reduced, but not below zero, by the taxable use percentage of the termination values of pooled assets with a balancing adjustment event in the year",
        "max(closing_before_disposals - termination_values, 0)",
    ),
    (
        "assessable_excess",
        "ITAA 1997 s 40-445(2): the excess of those termination values over the closing pool balance is assessable income",
        "max(termination_values - closing_before_disposals, 0)",
    ),
];

/// ITAA 1997 s 40-440: the decline in value of a low-value pool and its
/// closing pool balance, then s 40-445 for disposals. Every input is the
/// taxable use percentage of the amount (s 40-435).
pub fn depreciation_low_value_pool(
    previous_closing_pool_balance: &Arg,
    low_cost_assets: &Arg,
    low_value_assets: &Arg,
    second_element_costs: &Arg,
    termination_values: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    const KIND: &str = "depreciation_low_value_pool";
    let year = scope(scope_confirmed, year, KIND)?;
    let previous = money(previous_closing_pool_balance)?;
    let low_cost = money(low_cost_assets)?;
    let low_value = money(low_value_assets)?;
    let second = money(second_element_costs)?;
    let termination = money(termination_values)?;
    let built = precise(|| {
        let new_assets = low_cost.add(&second)?.mul(&d("0.1875"))?;
        let pool = previous.add(&low_value)?.mul(&d("0.375"))?;
        let decline = new_assets.add(&pool)?;
        let allocated = previous.add(&low_cost)?.add(&low_value)?.add(&second)?;
        let before = allocated.sub(&cents(decline.clone())?)?;
        let closing = py_max(before.sub(&termination)?, int(0))?;
        let excess = py_max(termination.sub(&before)?, int(0))?;
        let rows = vec![
            (vec![&low_cost, &second], new_assets.clone()),
            (vec![&previous, &low_value], pool.clone()),
            (vec![&new_assets, &pool], decline.clone()),
            (
                vec![&previous, &low_cost, &low_value, &second, &decline],
                before.clone(),
            ),
            (vec![&before, &termination], closing),
            (vec![&termination, &before], excess),
        ];
        Ok(steps(&LVP_RULES, rows))
    })?;
    let rates = vec![
        rate("first_year_rate", "0.1875"),
        rate("pool_rate", "0.375"),
    ];
    let out = derived_result(KIND, &year, built, rates)?;
    Ok(full_expensing_warning(out, &year, LVP_FULL_EXPENSING))
}

const SBP_FULL_EXPENSING: &str = "2020-21 to 2022-23 are temporary full expensing years: the pool's whole balance above zero is written off (IT(TP)A 1997 s 328-181(5)); whether an asset was fully expensed instead of pooled is not decided here.";
const NO_LOW_POOL: &str = "ITAA 1997 s 328-210(1): not taken, as the amount is not above zero and below the year's low pool value threshold";
const NOT_RATED: &str = "ITAA 1997 s 328-210(1) and s 328-215(2)(b): not taken, as the low pool value rule or a negative amount applies instead of s 328-190";
const ASSESSABLE: &str = "ITAA 1997 s 328-215(2) and (3): a closing pool balance, or s 328-210(2) amount, below zero is assessable income, and the closing pool balance becomes zero";
const CLOSING: &str = "ITAA 1997 s 328-200: the s 328-210(2) amount less the deductions under s 328-190, not below zero (s 328-215(3)); zero after a low pool value deduction (s 328-210(3))";

/// The s 328-210(2) amount, the first step whatever the outcome.
const SBP_AMOUNT: Rule = (
    "low_pool_value_amount",
    "ITAA 1997 s 328-210(2): the opening pool balance plus the taxable purpose proportions of the adjustable values of assets allocated for the year and of cost addition amounts, less the taxable purpose proportions of termination values",
    "opening_pool_balance + additions + cost_additions - termination_values",
);

/// An amount above zero and below the threshold: written off (s 328-210).
const SBP_WRITTEN_OFF: [Rule; 7] = [
    (
        "low_pool_value_deduction",
        "ITAA 1997 s 328-210(1) and (3): an amount above zero and below the year's low pool value threshold is the deduction, and the closing pool balance becomes zero",
        "low_pool_value_amount",
    ),
    ("pool_deduction", NOT_RATED, "0"),
    ("new_asset_deduction", NOT_RATED, "0"),
    ("cost_addition_deduction", NOT_RATED, "0"),
    (
        "deduction",
        "The deduction for the pool for the year: the low pool value amount",
        "low_pool_value_deduction",
    ),
    ("assessable_income", ASSESSABLE, "0"),
    ("closing_pool_balance", CLOSING, "0"),
];

/// An amount below zero: assessable (s 328-215(2)(b)).
const SBP_NEGATIVE: [Rule; 7] = [
    ("low_pool_value_deduction", NO_LOW_POOL, "0"),
    ("pool_deduction", NOT_RATED, "0"),
    ("new_asset_deduction", NOT_RATED, "0"),
    ("cost_addition_deduction", NOT_RATED, "0"),
    (
        "deduction",
        "No deduction: the s 328-210(2) amount is below zero",
        "0",
    ),
    ("assessable_income", ASSESSABLE, "0 - low_pool_value_amount"),
    ("closing_pool_balance", CLOSING, "0"),
];

/// Otherwise: the s 328-190 rates and the closing pool balance (s 328-200).
const SBP_RATED: [Rule; 7] = [
    ("low_pool_value_deduction", NO_LOW_POOL, "0"),
    (
        "pool_deduction",
        "ITAA 1997 s 328-190(1): 30% of the opening pool balance",
        "0.30 x opening_pool_balance",
    ),
    (
        "new_asset_deduction",
        "ITAA 1997 s 328-190(2): 15% of the taxable purpose proportion of the adjustable value of each asset allocated for the year",
        "0.15 x additions",
    ),
    (
        "cost_addition_deduction",
        "ITAA 1997 s 328-190(3) and (4): 15% of the taxable purpose proportion of cost addition amounts",
        "0.15 x cost_additions",
    ),
    (
        "deduction",
        "The deduction for the pool for the year: the s 328-190 deductions, each in cents",
        "cents(pool_deduction) + cents(new_asset_deduction) + cents(cost_addition_deduction)",
    ),
    (
        "assessable_income",
        ASSESSABLE,
        "max(deduction - low_pool_value_amount, 0)",
    ),
    (
        "closing_pool_balance",
        CLOSING,
        "max(low_pool_value_amount - deduction, 0)",
    ),
];

/// The low pool value threshold the s 328-210(2) amount must be below, with
/// its provision: `None` for no limit.
fn low_pool_threshold(year: &str) -> (Option<Decimal>, &'static str) {
    if FULL_EXPENSING_YEARS.contains(&year) {
        (
            None,
            "IT(TP)A 1997 s 328-181(5): no limit for an income year ending from 7.30 pm AEDT on 6 October 2020 to 30 June 2023",
        )
    } else if year == "2026-27" {
        (
            Some(int(20000)),
            "ITAA 1997 s 328-210(1), $20,000 as amended by Act No. 71 of 2026 Sch 2 item 7 for income years ending on or after 1 July 2026 (item 15(3))",
        )
    } else {
        (
            Some(int(20000)),
            "IT(TP)A 1997 s 328-180(6)(e): $20,000 for an income year ending from 1 July 2023 to 30 June 2026",
        )
    }
}

/// The pool's figures, each the taxable purpose proportion.
struct Pool {
    opening: Decimal,
    added: Decimal,
    improved: Decimal,
    termination: Decimal,
    amount: Decimal,
}

/// The steps after the s 328-210(2) amount when s 328-190 applies.
fn rated_steps(p: &Pool) -> Result<Vec<Step>> {
    precise(|| {
        let pool = p.opening.mul(&d("0.30"))?;
        let new_assets = p.added.mul(&d("0.15"))?;
        let addition = p.improved.mul(&d("0.15"))?;
        let total = cents(pool.clone())?
            .add(&cents(new_assets.clone())?)?
            .add(&cents(addition.clone())?)?;
        let assessable = py_max(total.sub(&p.amount)?, int(0))?;
        let closing = py_max(p.amount.sub(&total)?, int(0))?;
        let rows = vec![
            (vec![], int(0)),
            (vec![&p.opening], pool.clone()),
            (vec![&p.added], new_assets.clone()),
            (vec![&p.improved], addition.clone()),
            (vec![&pool, &new_assets, &addition], total.clone()),
            (vec![&total, &p.amount], assessable),
            (vec![&p.amount, &total], closing),
        ];
        Ok(steps(&SBP_RATED, rows))
    })
}

/// The steps after the s 328-210(2) amount for the year's threshold.
fn pool_steps(p: &Pool, threshold: Option<&Decimal>) -> Result<Vec<Step>> {
    let zero = || (vec![], int(0));
    let amount = &p.amount;
    let below = threshold.map_or(Ok(true), |limit| amount.lt(limit))?;
    if amount.gt(&int(0))? && below {
        let rows = vec![
            (vec![amount], amount.clone()),
            zero(),
            zero(),
            zero(),
            (vec![amount], amount.clone()),
            zero(),
            zero(),
        ];
        return Ok(steps(&SBP_WRITTEN_OFF, rows));
    }
    if amount.lt(&int(0))? {
        let assessable = precise(|| Ok(int(0).sub(amount)?))?;
        let mut rows: Vec<_> = (0..5).map(|_| zero()).collect();
        rows.extend([(vec![amount], assessable), zero()]);
        return Ok(steps(&SBP_NEGATIVE, rows));
    }
    rated_steps(p)
}

/// 2020-21 is the one year a zero s 328-210(2) amount leaves s 328-190(2) to
/// run on additions that backing business investment (IT(TP)A s 40-120) could
/// rate at 57.5%: refused.
fn refuse_backing_business_investment(year: &str, amount: &Decimal, added: &Decimal) -> Result<()> {
    if year == "2020-21" && amount.is_zero() && !added.is_zero() {
        return refuse(
            "depreciation_small_business_pool.backing_business_investment",
            "In 2020-21 a pool whose s 328-210(2) amount is exactly zero deducts 15% of additions, which backing business investment may rate at 57.5% instead; this worksheet does not model it.",
        );
    }
    Ok(())
}

/// ITAA 1997 Subdivision 328-D: a general small business pool's deduction for
/// the year, the low pool value write-off and the closing pool balance. Every
/// input is the taxable purpose proportion of the amount.
pub fn depreciation_small_business_pool(
    opening_pool_balance: &Arg,
    additions: &Arg,
    cost_additions: &Arg,
    termination_values: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    const KIND: &str = "depreciation_small_business_pool";
    let year = scope(scope_confirmed, year, KIND)?;
    let (opening, added) = (money(opening_pool_balance)?, money(additions)?);
    let (improved, termination) = (money(cost_additions)?, money(termination_values)?);
    let amount = precise(|| Ok(opening.add(&added)?.add(&improved)?.sub(&termination)?))?;
    refuse_backing_business_investment(&year, &amount, &added)?;
    let (threshold, threshold_rule) = low_pool_threshold(&year);
    let p = Pool {
        opening,
        added,
        improved,
        termination,
        amount,
    };
    let first = steps(
        &[SBP_AMOUNT],
        vec![(
            vec![&p.opening, &p.added, &p.improved, &p.termination],
            p.amount.clone(),
        )],
    );
    let all = first
        .into_iter()
        .chain(pool_steps(&p, threshold.as_ref())?)
        .collect();
    let rates = vec![
        rate("pool_rate", "0.30"),
        rate("new_asset_rate", "0.15"),
        rate(
            "low_pool_value_threshold",
            threshold.map_or_else(|| "none".to_string(), |t| t.to_string()),
        ),
        rate("low_pool_value_threshold_rule", threshold_rule),
    ];
    let out = derived_result(KIND, &year, all, rates)?;
    Ok(full_expensing_warning(out, &year, SBP_FULL_EXPENSING))
}

/// The rate and the step rules for works used in the 4% manner or not, begun
/// before 27 February 1992 or not.
fn works_rules(four: bool, before_1992: bool) -> (&'static str, [Rule; 3]) {
    let (rate_text, rule) = match (four, before_1992) {
        (true, _) => (
            "0.04",
            "ITAA 1997 s 43-210 step 1: the part used in the 4% manner (s 43-145), at 4% for the days so used over 365",
        ),
        (false, false) => (
            "0.025",
            "ITAA 1997 s 43-210 step 3: the part used as Table 43-140 describes and not in the 4% manner, at 2.5% for the days so used over 365",
        ),
        (false, true) => (
            "0.025",
            "ITAA 1997 s 43-215 step 1(b): works begun after 15 September 1987 and before 27 February 1992, at 2.5% for the days used over 365",
        ),
    };
    let (reduce, limit) = match (four, before_1992) {
        (_, true) => (
            "ITAA 1997 s 43-215 step 3: reduced to the extent the area was used only partly for producing assessable income or R&D activities",
            "ITAA 1997 s 43-215 step 4: the lesser of the reduced amount and the undeducted construction expenditure",
        ),
        (true, false) => (
            "ITAA 1997 s 43-210 step 2: reduced to the extent the part was used only partly for producing assessable income",
            "ITAA 1997 s 43-210 steps 5 and 6: the lesser of the reduced amount and the undeducted construction expenditure",
        ),
        (false, false) => (
            "ITAA 1997 s 43-210 step 4: reduced to the extent the part was used only partly for producing assessable income or R&D activities",
            "ITAA 1997 s 43-210 steps 5 and 6: the lesser of the reduced amount and the undeducted construction expenditure",
        ),
    };
    let rules = [
        (
            "rate_amount",
            rule,
            "construction_expenditure x days_used x rate / 365",
        ),
        (
            "reduced_amount",
            reduce,
            "construction_expenditure x days_used x rate x taxable_use / 365",
        ),
        (
            "deduction",
            limit,
            "min(reduced_amount, undeducted_construction_expenditure)",
        ),
    ];
    (rate_text, rules)
}

/// The start date, refusing works begun before 16 September 1987 and the 4%
/// manner before 27 February 1992; true when begun before that day.
fn works_begun(construction_start: &Arg, four: bool) -> Result<(i64, bool)> {
    let begun = date_arg(construction_start, "construction_start")?;
    if begun < dates::parse("1987-09-16").expect("date") {
        return refuse(
            "capital_works.begun_before_16_september_1987",
            "Capital works begun before 16 September 1987 (the 4% rate from 22 August 1984 and the earlier rules of s 43-215) are outside this worksheet.",
        );
    }
    let before_1992 = begun < dates::parse("1992-02-27").expect("date");
    if four && before_1992 {
        return refuse(
            "capital_works.four_percent_before_27_february_1992",
            "Capital works begun before 27 February 1992 and after 15 September 1987 have only the 2.5% rate (s 43-215 step 1(b)); the 4% manner (s 43-145) is for works begun later.",
        );
    }
    Ok((begun, before_1992))
}

/// The days used, at most the days of the income year from the later of
/// 1 July and the start of construction: 366 in 2023-24, whilst the
/// denominator stays 365, and none for works begun after the year.
fn days_in_use(days_used: &Arg, year: &str, begun: i64) -> Result<i64> {
    let first = dates::parse(&format!("{}-07-01", &year[..4])).expect("year");
    let last = dates::parse(&format!("20{}-06-30", &year[5..])).expect("year");
    let most = (last - begun.max(first) + 1).max(0);
    match days_used.int() {
        Some(Some(n)) if (0..=most).contains(&n) => Ok(n),
        _ => refuse(
            "input.day_count_invalid",
            format!(
                "days_used must be an integer from 0 to {most} for {year}, the days from the later of 1 July and construction_start to 30 June."
            ),
        ),
    }
}

/// The taxable use, exact to the cent: the 40-digit product of at most 14
/// digits of expenditure, 3 of days, 3 of rate and 20 of use is exact, and its
/// one division by 365 then errs by less than its distance from a cent boundary.
fn works_use(taxable_use: &Arg) -> Result<Decimal> {
    let Some(use_) = finite_between(taxable_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "taxable_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    if use_.adjusted() - use_.exponent() + 1 > 20 {
        return refuse(
            "capital_works.precision_exceeded",
            "taxable_use must have 20 or fewer significant digits, as written.",
        );
    }
    Ok(use_)
}

/// ITAA 1997 Division 43: one area's capital works deduction for the year, at
/// 4% for use in the 4% manner (s 43-145) or else 2.5%, for works begun from
/// 16 September 1987 (s 43-210 after 26 February 1992, s 43-215 before).
#[allow(clippy::too_many_arguments)]
pub fn capital_works(
    construction_expenditure: &Arg,
    construction_start: &Arg,
    four_percent_manner: &Arg,
    days_used: &Arg,
    taxable_use: &Arg,
    undeducted_construction_expenditure: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "capital_works")?;
    let expenditure = money(construction_expenditure)?;
    let four = flag(four_percent_manner, "four_percent_manner")?;
    let (begun, before_1992) = works_begun(construction_start, four)?;
    let days = int(days_in_use(days_used, &year, begun)?);
    let use_ = works_use(taxable_use)?;
    let undeducted = money(undeducted_construction_expenditure)?;
    if undeducted.gt(&expenditure)? {
        return refuse(
            "capital_works.undeducted_above_expenditure",
            "undeducted_construction_expenditure cannot exceed construction_expenditure.",
        );
    }
    let (rate_text, rules) = works_rules(four, before_1992);
    let rate_value = d(rate_text);
    let built = precise(|| {
        // Divide once, last, as the single-asset worksheet does.
        let product = expenditure.mul(&days)?.mul(&rate_value)?;
        let reduced = product.mul(&use_)?.div(&int(365))?;
        let deduction = py_min(reduced.clone(), undeducted.clone())?;
        let rows = vec![
            (
                vec![&expenditure, &days, &rate_value],
                product.div(&int(365))?,
            ),
            (
                vec![&expenditure, &days, &rate_value, &use_],
                reduced.clone(),
            ),
            (vec![&reduced, &undeducted], deduction),
        ];
        Ok(steps(&rules, rows))
    })?;
    let rates = vec![rate("rate", rate_text)];
    derived_result("capital_works", &year, built, rates)
}
