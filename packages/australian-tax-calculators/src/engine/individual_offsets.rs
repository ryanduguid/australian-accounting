// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The remaining individual tax offsets: the beneficiary tax offset (ITAA 1936
//! s 160AAA), the invalid and invalid carer tax offset (ITAA 1997
//! Subdivision 61-A), the zone and overseas forces tax offsets (ITAA 1936
//! ss 79A and 79B, with ITAA 1997 Division 961) and the net medical expenses
//! tax offset (ITAA 1936 ss 159P and 159Q, 2016-17 to 2018-19). The citations
//! are to ITAA 1936 compilations 193 (1 October 2026) and 161 (6 April 2019),
//! ITAA 1997 compilation 268 (1 October 2026), the Income Tax Assessment
//! (1936 Act) Regulations 2025 and Regulation 2015 (compilation 8), the Income
//! Tax Rates Act 1986 compilations 61 and 66, and the ATO's T4, T5 and net
//! medical expenses return instructions for each year, all read on
//! 22 November 2026.

use serde_json::{Value, json};

use super::{
    Arg, Result, d, day_count, dependent_children, flag, int, lookup, money, precise, rate, refuse,
    result, scope,
};
use crate::decimal::{Decimal, Rounding};
use crate::metadata as meta;

mod zone;
pub use zone::{ZoneArgs, zone_tax_offset};

const BENEFICIARY: &str = "beneficiary_tax_offset";
const INVALID_CARER: &str = "invalid_carer_tax_offset";
const MEDICAL: &str = "net_medical_expenses_tax_offset";

/// Days in income year `year` ("2023-24"): 366 where it holds 29 February.
fn days_in(year: &str) -> i64 {
    let end: i64 = year[..4].parse::<i64>().expect("income year") + 1;
    if end % 4 == 0 && (end % 100 != 0 || end % 400 == 0) {
        366
    } else {
        365
    }
}

/// `x` rounded up to the whole dollar; `x` is not negative.
fn up(x: &Decimal) -> Result<Decimal> {
    let down = x.quantize(0, Rounding::Down)?;
    Ok(if down.equals(x)? {
        down
    } else {
        down.add(&int(1))?
    })
}

fn dollars(x: &Decimal) -> Result<Decimal> {
    Ok(x.quantize(0, Rounding::Down)?)
}

fn at_least_nil(x: Decimal) -> Result<Decimal> {
    Ok(if x.lt(&int(0))? { int(0) } else { x })
}

/// $1 for every $4 by which `income` exceeds $282, in whole dollars (ITAA 1997
/// ss 61-45 and 961-20).
fn income_reduction(income: &Decimal) -> Result<Decimal> {
    let over = at_least_nil(income.sub(&int(282))?)?;
    dollars(&over.div(&int(4))?)
}

fn optional_money(value: Option<&Arg>) -> Result<Option<Decimal>> {
    value.map(money).transpose()
}

fn unknown<T>(message: String) -> Result<T> {
    refuse("input.field_unknown", message)
}

const BENEFIT_AMOUNT: &str = "Income Tax Assessment (1936 Act) Regulations 2025 s 13(4) (2015 Regulation s 13(4) for years before 2026-27): the rebatable benefit in assessable income, rounded down to the whole dollar";
const BENEFIT_OFFSET: &str = "ITAA 1936 s 160AAA(3) and the regulations s 13(2), (3) and (5): 0.15 x (amount - $6,000), plus 0.15 x (amount - $45,000) where the amount is greater than $45,000, the top of the lowest marginal rate in Income Tax Rates Act 1986 Sch 7 Part I item 1; nil at or below $6,000; rounded up to the whole dollar";

/// The beneficiary tax offset (ITAA 1936 s 160AAA(3)).
pub fn rebatable_benefit_offset(benefit: &Arg, year: &Arg, scope_confirmed: &Arg) -> Result<Value> {
    let year = scope(scope_confirmed, year, BENEFICIARY)?;
    let benefit = money(benefit)?;
    precise(|| {
        let amount = dollars(&benefit)?;
        let rate_15 = d("0.15");
        let threshold = int(meta::BENEFICIARY_UPPER_THRESHOLD);
        let lower = int(meta::BENEFICIARY_LOWER_THRESHOLD);
        let mut offset = at_least_nil(rate_15.mul(&amount.sub(&lower)?)?)?;
        if amount.gt(&threshold)? {
            offset = offset.add(&rate_15.mul(&amount.sub(&threshold)?)?)?;
        }
        let offset = up(&offset)?;
        let out = result(
            BENEFICIARY,
            &year,
            vec![
                ("rebatable_benefit_amount", amount),
                ("beneficiary_tax_offset", offset),
            ],
            vec![
                rate("rate", "0.15"),
                rate("lower_threshold", meta::BENEFICIARY_LOWER_THRESHOLD),
                rate("upper_threshold", meta::BENEFICIARY_UPPER_THRESHOLD),
            ],
        );
        let mut out = out?;
        out["authorities"] = json!({
            "rebatable_benefit_amount": BENEFIT_AMOUNT,
            "beneficiary_tax_offset": BENEFIT_OFFSET,
        });
        Ok(out)
    })
}

/// The arguments as the command passes them; see the README for each.
pub struct InvalidCarerArgs<'a> {
    pub dependant: &'a Arg,
    pub adjusted_taxable_income: &'a Arg,
    pub dependant_adjusted_taxable_income: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub days: Option<&'a Arg>,
    pub spouse_adjusted_taxable_income: Option<&'a Arg>,
    pub spouse_days: Option<&'a Arg>,
    pub maintained_by_others: Option<&'a Arg>,
    pub ftb_b_shared_care: Option<&'a Arg>,
    pub different_spouses: Option<&'a Arg>,
}

const LIMIT: &str = "ITAA 1997 s 61-20: no offset where the person's adjusted taxable income for offsets plus, for a spouse who is not the dependant, the spouse's (for part of the year, x days with a spouse / days in the year, s 61-20(2)) is more than the FTB Part B limit, as the ATO publishes it for the year";
const AMOUNT: &str = "ITAA 1997 s 61-30: $2,423 indexed, as the ATO publishes it for the year; for part of the year (s 61-40), days x the ATO's daily rate (T5 worksheets 1A and 3A), for a spouse counting only days when neither was eligible for FTB Part B or received parental leave pay (s 61-40(1)(f) and (g), (3))";
const REDUCTION: &str = "ITAA 1997 s 61-45: $1 for every $4 by which the dependant's adjusted taxable income for offsets, for the period maintained, exceeds $282";
const OFFSET: &str = "ITAA 1997 ss 61-30 to 61-45: the amount less the reduction, nil where less than $1 (ATO T5 worksheets 1E, 2A and 3B)";

fn refused_flag(value: Option<&Arg>, name: &str) -> Result<bool> {
    value.map_or(Ok(false), |v| flag(v, name))
}

/// Whether the dependant is the spouse, refusing the shared cases this
/// worksheet does not work out.
fn carer_dependant(a: &InvalidCarerArgs) -> Result<bool> {
    let spouse = match a.dependant.text() {
        Some("spouse") => true,
        Some("other") => false,
        _ => {
            return refuse(
                "invalid_carer_tax_offset.dependant_invalid",
                "dependant must be \"spouse\" or \"other\" (a parent, the spouse's parent, or a child, brother or sister of the person or the spouse aged 16 or over).",
            );
        }
    };
    if refused_flag(a.maintained_by_others, "maintained_by_others")? {
        return refuse(
            "invalid_carer_tax_offset.shared_maintenance",
            "Another person also maintained the dependant: ITAA 1997 s 61-40(1)(b) apportions the offset before s 61-45's income reduction, but the ATO's worksheets apply the share after it, and the 2 differ; this worksheet does not choose.",
        );
    }
    if refused_flag(a.ftb_b_shared_care, "ftb_b_shared_care")? {
        return refuse(
            "invalid_carer_tax_offset.ftb_shared_care",
            "ITAA 1997 s 61-35 reduces the offset by the FTB Part B shared-care rate over the non-shared-care rate, which Services Australia works out; this worksheet does not.",
        );
    }
    if refused_flag(a.different_spouses, "different_spouses")? {
        return refuse(
            "invalid_carer_tax_offset.successive_spouses",
            "You had a different spouse during different parts of the year: ITAA 1997 s 61-20(3) adds the adjusted taxable income for offsets of each spouse who is not the dependant, each apportioned under s 61-20(2), which one spouse input does not hold; this worksheet does not.",
        );
    }
    Ok(spouse)
}

/// The income of a spouse who is not the dependant, as counted for the
/// income limit: x days with that spouse / days in the year (s 61-20(2)).
fn counted_spouse_income(a: &InvalidCarerArgs, spouse: bool, days_in_year: i64) -> Result<Decimal> {
    if spouse && a.spouse_adjusted_taxable_income.is_some() {
        return unknown("spouse_adjusted_taxable_income is not an input of invalid-carer-tax-offset when the dependant is the spouse: s 61-20(1)(b) adds a spouse who is not the dependant".into());
    }
    if a.spouse_days.is_some() && a.spouse_adjusted_taxable_income.is_none() {
        return unknown(
            "spouse_days is an input of invalid-carer-tax-offset only with spouse_adjusted_taxable_income".into(),
        );
    }
    let spouse_income = optional_money(a.spouse_adjusted_taxable_income)?;
    let spouse_days = match a.spouse_days {
        Some(v) => day_count(v, days_in_year, "spouse_days")?,
        None => days_in_year,
    };
    precise(|| match &spouse_income {
        Some(income) => Ok(income.mul(&int(spouse_days))?.div(&int(days_in_year))?),
        None => Ok(int(0)),
    })
}

/// The invalid and invalid carer tax offset for one dependant (ITAA 1997
/// Subdivision 61-A).
pub fn invalid_carer_tax_offset(a: &InvalidCarerArgs) -> Result<Value> {
    if matches!(a.scope_confirmed, Arg::Bool(true)) && a.year.text() == Some("2026-27") {
        return refuse(
            "invalid_carer_tax_offset.rate_unpublished",
            "The 2026-27 offset amount and FTB Part B income limit, both indexed (ITAA 1997 s 61-30; s 61-20), had not been published when this was checked on 22 November 2026.",
        );
    }
    let year = scope(a.scope_confirmed, a.year, INVALID_CARER)?;
    let spouse = carer_dependant(a)?;
    let own = money(a.adjusted_taxable_income)?;
    let dependant_income = money(a.dependant_adjusted_taxable_income)?;
    let days_in_year = days_in(&year);
    let days = match a.days {
        Some(v) => day_count(v, days_in_year, "days")?,
        None => days_in_year,
    };
    let counted_spouse = counted_spouse_income(a, spouse, days_in_year)?;
    let [maximum, daily, limit] = lookup(meta::INVALID_CARER_OFFSET, &year).map(d);
    precise(|| {
        let tested = own.add(&counted_spouse)?;
        let eligible = !tested.gt(&limit)?;
        let amount = if days == days_in_year {
            maximum.clone()
        } else {
            daily.mul(&int(days))?
        };
        let reduction = income_reduction(&dependant_income)?;
        let left = amount.sub(&reduction)?;
        let offset = if eligible && !left.lt(&int(1))? {
            left
        } else {
            int(0)
        };
        let lines = [tested, amount, reduction, offset];
        carer_result(&year, lines, [&maximum, &daily, &limit], eligible, days)
    })
}

/// The worked lines, rates and authorities for the invalid carer offset.
fn carer_result(
    year: &str,
    lines: [Decimal; 4],
    rates: [&Decimal; 3],
    eligible: bool,
    days: i64,
) -> Result<Value> {
    let [tested, amount, reduction, offset] = lines;
    let [maximum, daily, limit] = rates;
    let lines = vec![
        ("income_tested", tested),
        ("amount_before_reduction", amount),
        ("income_reduction", reduction),
        ("invalid_carer_tax_offset", offset),
    ];
    let rates = vec![
        rate("maximum", maximum),
        rate("daily_rate", daily),
        rate("ftb_part_b_limit", limit),
        rate("income_free_area", 282),
    ];
    let mut out = result(INVALID_CARER, year, lines, rates)?;
    out["eligible"] = json!(eligible);
    out["days"] = json!(days);
    out["authorities"] = json!({
        "income_tested": LIMIT,
        "amount_before_reduction": AMOUNT,
        "income_reduction": REDUCTION,
        "invalid_carer_tax_offset": OFFSET,
    });
    Ok(out)
}

/// The arguments as the command passes them; see the README for each.
pub struct MedicalArgs<'a> {
    pub net_medical_expenses: &'a Arg,
    pub adjusted_taxable_income: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub spouse_adjusted_taxable_income: Option<&'a Arg>,
    pub dependent_children: Option<&'a Arg>,
}

const MEDICAL_INCOME: &str = "ITAA 1936 s 159Q(1) (compilation 161): married on the last day of the year, or on the day the person died for one who died in the year (s 159Q(2)), the person's and the spouse's adjusted taxable income for rebates against the family tier 1 threshold; otherwise, with a dependant on any day, the person's own against the family tier 1 threshold; otherwise the person's own against the singles tier 1 threshold";
const MEDICAL_OFFSET: &str = "ITAA 1936 s 159P(3AA) (compilation 161): 10% of the net medical expenses over the higher phase-in limit where the income exceeds the threshold, otherwise 20% over the lower phase-in limit (s 159Q(5) and (6), indexed), as the ATO publishes them for the year; only disability aids, attendant care and aged care count (s 159P(1B))";

/// The year, refusing 2019-20 and later with the reason the offset ended.
fn medical_year(a: &MedicalArgs) -> Result<String> {
    if matches!(a.scope_confirmed, Arg::Bool(true))
        && a.year.text().is_some_and(|y| {
            y.len() == 7 && y.is_ascii() && y[..4].parse::<i64>().is_ok_and(|start| start >= 2019)
        })
    {
        return refuse(
            "period.unsupported",
            "ITAA 1936 s 159P(1A): no net medical expenses tax offset for 2019-20 or a later year; supported: 2016-17, 2017-18, 2018-19.",
        );
    }
    scope(a.scope_confirmed, a.year, MEDICAL)
}

/// Whether the family threshold applies, and the threshold (s 159Q(1)): a
/// spouse or a dependent child makes it $180,000 plus $1,500 for each child
/// after the first; otherwise $90,000.
fn medical_threshold(spouse: bool, children: i64) -> (bool, Decimal) {
    if spouse || children > 0 {
        (true, int(180_000 + 1_500 * (children - 1).max(0)))
    } else {
        (false, int(90_000))
    }
}

/// The net medical expenses tax offset for 2016-17 to 2018-19 (ITAA 1936
/// ss 159P and 159Q).
pub fn disability_expenses_offset(a: &MedicalArgs) -> Result<Value> {
    let year = medical_year(a)?;
    let net = money(a.net_medical_expenses)?;
    let own = money(a.adjusted_taxable_income)?;
    let spouse = optional_money(a.spouse_adjusted_taxable_income)?;
    let children = dependent_children(a.dependent_children)?;
    let [lower, higher] = lookup(meta::NET_MEDICAL_LIMITS, &year).map(d);
    precise(|| {
        let (family, threshold) = medical_threshold(spouse.is_some(), children);
        let tested = match &spouse {
            Some(income) => own.add(income)?,
            None => own.clone(),
        };
        let higher_applies = tested.gt(&threshold)?;
        let (limit, pct) = if higher_applies {
            (higher.clone(), d("0.10"))
        } else {
            (lower.clone(), d("0.20"))
        };
        let excess = at_least_nil(net.sub(&limit)?)?;
        let offset = pct.mul(&excess)?;
        let out = result(
            MEDICAL,
            &year,
            vec![
                ("income_tested", tested),
                ("net_medical_expenses", net),
                ("excess", excess),
                ("net_medical_expenses_tax_offset", offset),
            ],
            vec![
                rate("lower_phase_in_limit", &lower),
                rate("higher_phase_in_limit", &higher),
                rate("threshold", &threshold),
                rate("percentage", if higher_applies { "0.10" } else { "0.20" }),
            ],
        );
        let mut out = out?;
        out["thresholds"] = json!(if family { "family" } else { "singles" });
        out["higher_limit_applies"] = json!(higher_applies);
        out["authorities"] = json!({
            "income_tested": MEDICAL_INCOME,
            "net_medical_expenses_tax_offset": MEDICAL_OFFSET,
        });
        Ok(out)
    })
}
