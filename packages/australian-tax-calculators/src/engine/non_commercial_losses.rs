// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `non_commercial_losses`: whether ITAA 1997 Division 35 defers an
//! individual's loss from a business activity for an income year from
//! 2020-21 to 2026-27: the income requirement, the 4 tests, the primary
//! production and professional arts exception and the amount deferred. The
//! citations are to ITAA 1997 compilation 268 (1 October 2026), volume 1,
//! read on 22 November 2026; Division 35 reads the same in compilations 210
//! (1 July 2020) and 223 (1 July 2021). Every amount is the operator's figure
//! for the activity, or for a group of similar ones (s 35-10(3)); the
//! Commissioner's discretion (s 35-55) is never decided here.

use serde_json::{Value, json};

use super::stated;
use super::{Arg, Result, flag, int, money, rate, refuse, result, scope, signed_money};
use crate::decimal::Decimal;

const KIND: &str = "non_commercial_losses";

/// Section 35-10(2E).
const INCOME_REQUIREMENT: &str = "250000";
/// Section 35-30.
const ASSESSABLE_INCOME: &str = "20000";
/// Section 35-40(1).
const REAL_PROPERTY: &str = "500000";
/// Section 35-45(1).
const OTHER_ASSETS: &str = "100000";
/// Section 35-10(4)(b).
const OTHER_INCOME: &str = "40000";

/// The facts the command takes only for some answers, in
/// [`NonCommercialLossesArgs::optional`]: the full-year estimate for an
/// activity started or stopped in the year, and the income from other
/// sources for a primary production or professional arts business.
pub const NCL_OPTIONAL: [&str; 2] = ["assessable_income_estimate", "other_income"];

/// The facts as the command passes them; see the README for each.
pub struct NonCommercialLossesArgs<'a> {
    pub activity: &'a Arg,
    pub assessable_income: &'a Arg,
    pub deductions: &'a Arg,
    pub brought_forward: &'a Arg,
    pub part_year: &'a Arg,
    pub earlier_years: &'a Arg,
    pub real_property_value: &'a Arg,
    pub other_assets_value: &'a Arg,
    pub taxable_income: &'a Arg,
    pub business_losses: &'a Arg,
    pub assessable_fhss_released_amount: &'a Arg,
    pub reportable_fringe_benefits: &'a Arg,
    pub reportable_super_contributions: &'a Arg,
    pub total_net_investment_loss: &'a Arg,
    pub net_exempt_income: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 2],
}

fn at_least(amount: &Decimal, threshold: &str) -> Result<bool> {
    Ok(!amount.lt(&Decimal::parse(threshold)?)?)
}

fn nil_floor(amount: Decimal) -> Result<Decimal> {
    Ok(if amount.lt(&int(0))? { int(0) } else { amount })
}

fn test(id: &str, authority: &str, met: bool) -> Value {
    json!({"test": id, "authority": authority, "met": met})
}

/// The 4 income years before this one, latest first: each a profit year when
/// its deductions are less than its assessable income (s 35-35); `null` for a
/// year the activity was not carried on.
fn earlier_profit_years(given: &Arg) -> Result<usize> {
    let bad = || {
        refuse(
            "non_commercial_losses.earlier_years_invalid",
            "earlier_years must be a list of the 4 income years before this one, latest first, each null or an object with assessable_income and deductions.",
        )
    };
    let Arg::Json(Value::Array(years)) = given else {
        return bad();
    };
    if years.len() != 4 {
        return bad();
    }
    let mut profits = 0;
    for (i, year) in years.iter().enumerate() {
        let name = format!("earlier year {}", i + 1);
        match year {
            Value::Null => {}
            Value::Object(fields) => {
                stated::known_keys(
                    fields,
                    &name,
                    "an earlier year",
                    &["assessable_income", "deductions"],
                )?;
                let income = stated::amount(fields, &name, "assessable_income")?;
                let deductions = stated::amount(fields, &name, "deductions")?;
                if deductions.lt(&income)? {
                    profits += 1;
                }
            }
            _ => return bad(),
        }
    }
    Ok(profits)
}

/// The optional facts: each required where `wanted` says so and refused
/// otherwise.
fn optional_facts<'a>(
    args: &NonCommercialLossesArgs<'a>,
    wanted: [bool; 2],
) -> Result<[Option<&'a Arg>; 2]> {
    for ((key, given), want) in NCL_OPTIONAL.iter().zip(args.optional).zip(wanted) {
        match (want, given) {
            (true, None) => {
                return refuse(
                    "input.missing_field",
                    format!("{key} is required for these facts."),
                );
            }
            (false, Some(_)) => {
                return refuse(
                    "input.field_unknown",
                    format!("{key} is not an input for these facts"),
                );
            }
            _ => {}
        }
    }
    Ok(args.optional)
}

pub fn non_commercial_losses(args: &NonCommercialLossesArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let excepted = match args.activity.text() {
        Some("other") => false,
        Some("primary_production" | "professional_arts") => true,
        _ => {
            return refuse(
                "non_commercial_losses.activity_invalid",
                "activity must be other, primary_production or professional_arts.",
            );
        }
    };
    let part_year = flag(args.part_year, "part_year")?;
    let [estimate, other_income] = optional_facts(args, [part_year, excepted])?;
    let income = money(args.assessable_income)?;
    let deductions = money(args.deductions)?;
    let brought_forward = money(args.brought_forward)?;
    let earlier_profits = earlier_profit_years(args.earlier_years)?;
    let real_property = money(args.real_property_value)?;
    let other_assets = money(args.other_assets_value)?;
    let taxable = signed_money(args.taxable_income)?;
    let losses = money(args.business_losses)?;
    let fhss = money(args.assessable_fhss_released_amount)?;
    let fringe = money(args.reportable_fringe_benefits)?;
    let super_contributions = money(args.reportable_super_contributions)?;
    let investment = money(args.total_net_investment_loss)?;
    let exempt = money(args.net_exempt_income)?;

    // s 35-10(2E): taxable income without the FHSS amount or the business
    // losses (the ATO's 'use zero' for a loss), then the other 3 amounts.
    let requirement_income = nil_floor(taxable.add(&losses)?.sub(&fhss)?)?;
    let requirement_total = requirement_income
        .add(&fringe)?
        .add(&super_contributions)?
        .add(&investment)?;
    let requirement_met = requirement_total.lt(&Decimal::parse(INCOME_REQUIREMENT)?)?;

    // s 35-30: a reasonable full-year estimate for a part year.
    let measured = match estimate {
        Some(estimate) => money(estimate)?,
        None => income.clone(),
    };
    let profits = earlier_profits + usize::from(deductions.lt(&income)?);
    let tests = [
        test(
            "assessable_income_test",
            if part_year {
                "ITAA 1997 s 35-30(b): a reasonable estimate of the activity's assessable income for a full year is at least $20,000"
            } else {
                "ITAA 1997 s 35-30(a): the activity's assessable income for the year is at least $20,000"
            },
            at_least(&measured, ASSESSABLE_INCOME)?,
        ),
        test(
            "profits_test",
            "ITAA 1997 s 35-35: in at least 3 of the past 5 income years, this one included, the deductions attributable to the activity (apart from s 35-10(2) and (2C)) are less than its assessable income; for a partner, your own share (s 35-35(2))",
            profits >= 3,
        ),
        test(
            "real_property_test",
            "ITAA 1997 s 35-40: real property used on a continuing basis in the activity, at reduced cost base or higher market value, not a private dwelling or a tenant's fixtures, totals at least $500,000",
            at_least(&real_property, REAL_PROPERTY)?,
        ),
        test(
            "other_assets_test",
            "ITAA 1997 s 35-45: the counted assets used on a continuing basis in the activity, not real property or cars, motor cycles and similar vehicles, total at least $100,000",
            at_least(&other_assets, OTHER_ASSETS)?,
        ),
    ];
    let passed = tests.iter().any(|t| t["met"] == true);
    let exception = match other_income {
        Some(other) => !at_least(&money(other)?, OTHER_INCOME)?,
        None => false,
    };
    let rule_applies = !(requirement_met && passed) && !exception;

    // s 35-10(2): the year's deductions, with amounts brought forward, over
    // the activity's assessable income; s 35-15 takes net exempt income off
    // the amount deferred.
    let excess = nil_floor(deductions.add(&brought_forward)?.sub(&income)?)?;
    let (deferred, deductible) = if rule_applies {
        (nil_floor(excess.sub(&exempt)?)?, int(0))
    } else {
        (int(0), excess.clone())
    };
    let outcome = if excess.is_zero() {
        "no excess of deductions over assessable income from the activity, so nothing is deferred"
    } else if !rule_applies {
        "the excess is not deferred: section 35-10(2) does not apply"
    } else {
        "the excess is deferred to the next income year in which the activity is carried on (s 35-10(2)), unless the Commissioner decides under s 35-55 that the rule does not apply"
    };

    let mut amounts = vec![
        ("income_requirement_taxable_income", requirement_income),
        ("income_requirement_total", requirement_total),
        ("excess", excess),
        ("deferred", deferred),
        ("deductible_this_year", deductible),
    ];
    if part_year {
        amounts.push(("assessable_income_measured", measured));
    }
    let mut out = result(
        KIND,
        &year,
        amounts,
        vec![
            rate("income_requirement", INCOME_REQUIREMENT),
            rate("assessable_income_test", ASSESSABLE_INCOME),
            rate("real_property_test", REAL_PROPERTY),
            rate("other_assets_test", OTHER_ASSETS),
            rate("excepted_activity_other_income", OTHER_INCOME),
        ],
    )?;
    out["income_requirement"] = json!({
        "met": requirement_met,
        "authority": "ITAA 1997 s 35-10(2E): taxable income (without the assessable FHSS released amount and any s 35-10(2) excess), reportable fringe benefits, reportable superannuation contributions and total net investment losses sum to less than $250,000",
    });
    out["tests"] = json!(tests);
    out["profit_years"] = json!(profits);
    out["exception"] = json!({
        "applies": exception,
        "authority": "ITAA 1997 s 35-10(4): a primary production or professional arts business, with assessable income from other sources (except net capital gain) under $40,000",
    });
    out["commissioner_discretion"] = json!({
        "decided": false,
        "authority": "ITAA 1997 s 35-55: on application, the Commissioner may decide the rule in s 35-10(2) does not apply for special circumstances or a lead time; this worksheet does not decide it",
    });
    out["rule_applies"] = json!(rule_applies);
    out["outcome"] = json!(outcome);
    Ok(out)
}
