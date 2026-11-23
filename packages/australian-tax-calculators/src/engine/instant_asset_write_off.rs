// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The instant asset write-off of one asset in the income year it is first
//! used, or installed ready for use, for a taxable purpose: a small business
//! entity's under ITAA 1997 s 328-180, with the limits of IT(TP)A 1997
//! ss 328-180(4), (4A) and 328-181(2) and Act No. 71 of 2026 Sch 2, and a
//! medium sized business's under ITAA 1997 s 40-82(2A). The sections are from
//! the ATO's legal database and the Act from its published text, all read on
//! 23 November 2026.

use serde_json::{Value, json};

use super::depreciation_pools::full_expensing_warning;
use super::{
    Arg, Result, Step, date_arg, derived_result, finite_between, int, money, precise, rate, refuse,
    scope, step,
};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "instant_asset_write_off";

/// The days of the 7.30 pm budget times (legal time in the ACT): IT(TP)A
/// 1997 s 328-180(1) and s 40-140.
const BUDGET_2015: &str = "2015-05-12";
const BUDGET_2019: &str = "2019-04-02";
const BUDGET_2020: &str = "2020-10-06";

const RULE_2026: &str = "ITAA 1997 s 328-180(1)(b): less than $20,000, as amended by Act No. 71 of 2026 Sch 2 item 3 for an asset first used or installed ready for use on or after 1 July 2026 (item 15(1))";
const RULE_FULL_EXPENSING: &str = "IT(TP)A 1997 s 328-181(2): no cost limit for an asset held, and first used or installed ready for use, from 7.30 pm AEDT on 6 October 2020 to 30 June 2023 (temporary full expensing)";
const RULE_4A: &str = "IT(TP)A 1997 s 328-180(4A): less than $150,000 for an asset first acquired from 7.30 pm on 12 May 2015 and first used or installed ready for use from 12 March 2020 to 30 June 2021";
const RULE_4D: &str = "IT(TP)A 1997 s 328-180(4)(d): less than $20,000 for an asset first acquired from 7.30 pm on 12 May 2015 and first used or installed ready for use from 1 July 2023 to 30 June 2026";
const RULE_BASE: &str = "ITAA 1997 s 328-180(1)(b) as in force before the threshold change day: less than $1,000, as no temporary increase in IT(TP)A 1997 ss 328-180 and 328-181 covers the asset's dates";
const RULE_40_82: &str = "ITAA 1997 s 40-82(2A) and (2B): less than $150,000 for a medium sized business's asset first acquired from 7.30 pm on 2 April 2019 to 31 December 2020 and first used or installed ready for use from 12 March 2020 to 30 June 2021";
const NONE_AFTER_2021: &str = "ITAA 1997 s 40-82(5): no write-off for a medium sized business's asset first used or installed ready for use after 30 June 2021";
const NONE_BEFORE_2019: &str = "ITAA 1997 s 40-82(2A)(b): no write-off for an asset first acquired before 7.30 pm on 2 April 2019";
const NONE_OTHER: &str = "ITAA 1997 ss 328-180(1)(a) and (ab) and 40-82(4A): the write-off is for a small business entity using simplified depreciation or, to 30 June 2021, a medium sized business";

/// The cost the asset must be less than, and the rule that sets it.
enum Limit {
    Below(&'static str, &'static str),
    Unlimited(&'static str),
    Inapplicable(&'static str),
}

#[derive(Clone, Copy, PartialEq)]
enum Entity {
    SmallBusiness,
    Medium,
    Other,
}

fn day(text: &str) -> i64 {
    dates::parse(text).expect("date")
}

/// Whether the asset was acquired at or after 7.30 pm on `budget`, refusing
/// that day itself: the command takes dates, not times.
fn held_from(acquired: i64, budget: &str) -> Result<bool> {
    if acquired == day(budget) {
        return refuse(
            "instant_asset_write_off.budget_time_day",
            format!(
                "The result turns on whether the asset was first acquired before or after 7.30 pm on {budget} (legal time in the ACT), and this command takes dates only."
            ),
        );
    }
    Ok(acquired > day(budget))
}

/// A small business entity's limit by the date of first use (s 328-180(1)(b)
/// and IT(TP)A 1997 ss 328-180(4) and (4A) and 328-181(2)).
fn small_business_limit(acquired: i64, used: i64) -> Result<Limit> {
    if used >= day("2026-07-01") {
        return Ok(Limit::Below("20000", RULE_2026));
    }
    if used <= day("2023-06-30") && held_from(acquired, BUDGET_2020)? {
        return Ok(Limit::Unlimited(RULE_FULL_EXPENSING));
    }
    let increased = if used <= day("2021-06-30") {
        Some(("150000", RULE_4A))
    } else if used >= day("2023-07-01") {
        Some(("20000", RULE_4D))
    } else {
        None
    };
    match increased {
        Some((limit, rule)) if held_from(acquired, BUDGET_2015)? => Ok(Limit::Below(limit, rule)),
        _ => Ok(Limit::Below("1000", RULE_BASE)),
    }
}

/// Any other entity: temporary full expensing (IT(TP)A Subdivision 40-BB)
/// displaces every other rule where it may apply (s 40-145), and is refused;
/// otherwise s 40-82(2A) for a medium sized business. An asset first used
/// from the 2020 budget day to 30 June 2023 is refused whenever it was
/// acquired: s 40-170 needs no acquisition after the budget time, only an
/// amount included in the second element of its cost from that time, which
/// can precede first use. Earlier first use rules that out, and an asset
/// acquired after 31 December 2020 is first used after it.
fn other_limit(entity: Entity, acquired: i64, used: i64) -> Result<Limit> {
    if (day(BUDGET_2020)..=day("2023-06-30")).contains(&used) {
        return refuse(
            "instant_asset_write_off.temporary_full_expensing",
            "An asset first used or installed ready for use from 7.30 pm AEDT on 6 October 2020 to 30 June 2023 may be fully expensed under IT(TP)A 1997 Subdivision 40-BB, which displaces s 40-82 (s 40-145): its cost under s 40-160 if held from that time, or under s 40-170 any amount included in the second element of its cost from that time. Whether it was turns on those amounts, the entity's turnover or income, the s 40-165 and 40-167 exclusions and any s 40-190 choice, which this command does not decide.",
        );
    }
    if entity == Entity::Other {
        return Ok(Limit::Inapplicable(NONE_OTHER));
    }
    if used > day("2021-06-30") {
        return Ok(Limit::Inapplicable(NONE_AFTER_2021));
    }
    if held_from(acquired, BUDGET_2019)? {
        return Ok(Limit::Below("150000", RULE_40_82));
    }
    Ok(Limit::Inapplicable(NONE_BEFORE_2019))
}

fn entity_arg(entity: &Arg) -> Result<Entity> {
    match entity.text() {
        Some("small_business_simplified") => Ok(Entity::SmallBusiness),
        Some("medium_business") => Ok(Entity::Medium),
        Some("other") => Ok(Entity::Other),
        _ => refuse(
            "instant_asset_write_off.entity_invalid",
            "entity must be small_business_simplified, medium_business or other.",
        ),
    }
}

/// The acquisition and first use days, first use within `year` and not
/// before acquisition.
fn asset_dates(acquired: &Arg, first_used: &Arg, year: &str) -> Result<(i64, i64)> {
    let acquired = date_arg(acquired, "acquired")?;
    let used = date_arg(first_used, "first_used")?;
    let start = day(&format!("{}-07-01", &year[..4]));
    let end = day(&format!("20{}-06-30", &year[5..]));
    if !(start..=end).contains(&used) {
        return refuse(
            "instant_asset_write_off.first_use_outside_year",
            format!("first_used must fall in {year}, the income year of the write-off."),
        );
    }
    if acquired > used {
        return refuse(
            "instant_asset_write_off.acquired_after_first_use",
            "acquired cannot be after first_used.",
        );
    }
    Ok((acquired, used))
}

/// The taxable purpose proportion: the product of a cost of at most 15
/// digits and a use of at most 26 is exact in 40 digits, rounded once.
fn use_arg(taxable_use: &Arg) -> Result<Decimal> {
    let Some(use_) = finite_between(taxable_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "taxable_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    if use_.adjusted() - use_.exponent() + 1 > 26 {
        return refuse(
            "instant_asset_write_off.precision_exceeded",
            "taxable_use must have 26 or fewer significant digits, as written.",
        );
    }
    Ok(use_)
}

/// The two steps: the taxable purpose proportion of cost, and the deduction.
fn write_off_steps(
    entity: Entity,
    cost: &Decimal,
    use_: &Decimal,
    rule: Option<&'static str>,
) -> Result<Vec<Step>> {
    let proportion = precise(|| Ok(cost.mul(use_)?))?;
    let authority = if entity == Entity::SmallBusiness {
        "ITAA 1997 s 328-180(1): the taxable purpose proportion of the asset's adjustable value, its cost in the year of first use"
    } else {
        "ITAA 1997 s 40-25(2): the decline in value, the asset's cost, less the part attributable to use for other than a taxable purpose"
    };
    let deduction = match rule {
        Some(rule) => step(
            "deduction",
            rule,
            "taxable_purpose_cost",
            &[&proportion],
            proportion.clone(),
        ),
        None => step(
            "deduction",
            "No write-off: the cost is not less than the limit, or no write-off rule applies (cost_limit_rule)",
            "0",
            &[],
            int(0),
        ),
    };
    Ok(vec![
        step(
            "taxable_purpose_cost",
            authority,
            "cost x taxable_use",
            &[cost, use_],
            proportion,
        ),
        deduction,
    ])
}

/// Where an asset not written off goes instead.
fn not_written_off_warning(entity: Entity, year: &str) -> &'static str {
    match (entity, year) {
        (Entity::SmallBusiness, "2020-21") => {
            "Not written off: the taxable purpose proportion goes to the general small business pool (ITAA 1997 s 328-185), at 57.5% in its first year where backing business investment applies (IT(TP)A 1997 s 328-182)."
        }
        (Entity::SmallBusiness, _) => {
            "Not written off: the taxable purpose proportion goes to the general small business pool (ITAA 1997 s 328-185)."
        }
        _ => "Not written off: work out the asset's decline in value under Division 40's methods.",
    }
}

const FULL_EXPENSING_NOTE: &str = "2020-21 to 2022-23 are temporary full expensing years: a small business entity using simplified depreciation cannot opt out of it for an asset it held, and first used or installed ready for use, from 7.30 pm AEDT on 6 October 2020 to 30 June 2023 (IT(TP)A 1997 s 328-181(2)); for other entities this command refuses those assets.";

/// The instant asset write-off of one asset for the income year of its first
/// use, or installation ready for use, for a taxable purpose.
pub fn instant_asset_write_off(
    entity: &Arg,
    acquired: &Arg,
    first_used: &Arg,
    cost: &Arg,
    taxable_use: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let entity = entity_arg(entity)?;
    let (acquired, used) = asset_dates(acquired, first_used, &year)?;
    let cost = money(cost)?;
    let use_ = use_arg(taxable_use)?;
    let limit = match entity {
        Entity::SmallBusiness => small_business_limit(acquired, used)?,
        _ => other_limit(entity, acquired, used)?,
    };
    let (limit_text, rule, written_off) = match limit {
        Limit::Below(amount, rule) => (
            amount,
            rule,
            cost.lt(&Decimal::parse(amount).expect("limit"))?,
        ),
        Limit::Unlimited(rule) => ("none", rule, true),
        Limit::Inapplicable(rule) => ("not_applicable", rule, false),
    };
    let steps = write_off_steps(entity, &cost, &use_, written_off.then_some(rule))?;
    let rates = vec![
        rate("cost_limit", limit_text),
        rate("cost_limit_rule", rule),
        rate("written_off", written_off),
    ];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    if !written_off {
        out["warnings"]
            .as_array_mut()
            .expect("warnings")
            .push(json!(not_written_off_warning(entity, &year)));
    }
    Ok(full_expensing_warning(out, &year, FULL_EXPENSING_NOTE))
}
