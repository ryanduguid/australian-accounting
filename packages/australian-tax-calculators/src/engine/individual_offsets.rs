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

use serde_json::{Map, Value, json};

use super::{
    Arg, Result, cents, d, day_count, dependent_children, flag, int, lookup, money, precise, rate,
    refuse, result, scope,
};
use crate::decimal::{Decimal, Rounding};
use crate::metadata as meta;

const BENEFICIARY: &str = "beneficiary_tax_offset";
const INVALID_CARER: &str = "invalid_carer_tax_offset";
const ZONE: &str = "zone_tax_offset";
const MEDICAL: &str = "net_medical_expenses_tax_offset";

/// The most dependants one zone or overseas forces claim takes.
const MAX_DEPENDANTS: usize = 50;

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
pub fn beneficiary_tax_offset(benefit: &Arg, year: &Arg, scope_confirmed: &Arg) -> Result<Value> {
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
        let mut out = result(
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
        )?;
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
}

const LIMIT: &str = "ITAA 1997 s 61-20: no offset where the person's adjusted taxable income for offsets plus, for a spouse who is not the dependant, the spouse's (for part of the year, x days with a spouse / days in the year, s 61-20(2)) is more than the FTB Part B limit, as the ATO publishes it for the year";
const AMOUNT: &str = "ITAA 1997 s 61-30: $2,423 indexed, as the ATO publishes it for the year; for part of the year (s 61-40), days x the ATO's daily rate (T5 worksheets 1A and 3A), for a spouse counting only days when neither was eligible for FTB Part B or received parental leave pay (s 61-40(1)(f) and (g), (3))";
const REDUCTION: &str = "ITAA 1997 s 61-45: $1 for every $4 by which the dependant's adjusted taxable income for offsets, for the period maintained, exceeds $282";
const OFFSET: &str = "ITAA 1997 ss 61-30 to 61-45: the amount less the reduction, nil where less than $1 (ATO T5 worksheets 1E, 2A and 3B)";

fn refused_flag(value: Option<&Arg>, name: &str) -> Result<bool> {
    value.map_or(Ok(false), |v| flag(v, name))
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
    let own = money(a.adjusted_taxable_income)?;
    let dependant_income = money(a.dependant_adjusted_taxable_income)?;
    let days_in_year = days_in(&year);
    let days = match a.days {
        Some(v) => day_count(v, days_in_year, "days")?,
        None => days_in_year,
    };
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
    let [maximum, daily, limit] = lookup(meta::INVALID_CARER_OFFSET, &year).map(d);
    precise(|| {
        let counted_spouse = match &spouse_income {
            Some(income) => income.mul(&int(spouse_days))?.div(&int(days_in_year))?,
            None => int(0),
        };
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
        let mut out = result(
            INVALID_CARER,
            &year,
            vec![
                ("income_tested", tested),
                ("amount_before_reduction", amount),
                ("income_reduction", reduction),
                ("invalid_carer_tax_offset", offset),
            ],
            vec![
                rate("maximum", &maximum),
                rate("daily_rate", &daily),
                rate("ftb_part_b_limit", &limit),
                rate("income_free_area", 282),
            ],
        )?;
        out["eligible"] = json!(eligible);
        out["days"] = json!(days);
        out["authorities"] = json!({
            "income_tested": LIMIT,
            "amount_before_reduction": AMOUNT,
            "income_reduction": REDUCTION,
            "invalid_carer_tax_offset": OFFSET,
        });
        Ok(out)
    })
}

/// The arguments as the command passes them; see the README for each.
pub struct ZoneArgs<'a> {
    pub area: &'a Arg,
    pub days: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub prior_year: Option<&'a Arg>,
    pub prior_days: Option<&'a Arg>,
    pub continuous_residence: Option<&'a Arg>,
    pub includes_first_day: Option<&'a Arg>,
    pub dependants: Option<&'a Arg>,
    pub sole_parent_days: Option<&'a Arg>,
    pub invalid_carer_tax_offset: Option<&'a Arg>,
    pub remote_area_allowance: Option<&'a Arg>,
}

/// (area, fixed amount, percentage of the relevant rebate amount): ITAA 1936
/// s 79A(2)(a) to (c) and s 79B(2)(a).
const AREAS: [(&str, i64, &str); 4] = [
    ("special_area", 1173, "0.5"),
    ("zone_a", 338, "0.5"),
    ("zone_b", 57, "0.2"),
    ("overseas_forces", 338, "0.5"),
];

const DEPENDANT_FIELDS: [&str; 4] = ["kind", "adjusted_taxable_income", "days", "shared"];

fn dependants_invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("zone_tax_offset.dependants_invalid", message)
}

/// One dependant's notional tax offset (ITAA 1997 ss 961-10 to 961-20): the
/// full amount, or for part of the year the ATO's base x days / days in the
/// year to the cent (T4 worksheet 1); less $1 for every $4 of adjusted
/// taxable income over $282, without cents and not below nil (worksheet 2).
fn notional(full: i64, days: i64, days_in_year: i64, income: &Decimal) -> Result<Decimal> {
    let amount = if days == days_in_year {
        int(full)
    } else {
        cents(int(full).mul(&int(days))?.div(&int(days_in_year))?)?
    };
    let reduction = income_reduction(income)?;
    if reduction.is_zero() {
        return Ok(amount);
    }
    at_least_nil(dollars(&amount.sub(&reduction)?)?)
}

/// The dependants' notional tax offsets (ITAA 1997 Subdivision 961-A) and the
/// worked lines.
fn dependant_offsets(value: Option<&Arg>, days_in_year: i64) -> Result<(Decimal, Vec<Value>)> {
    let Some(value) = value else {
        return Ok((int(0), Vec::new()));
    };
    let Arg::Json(Value::Array(items)) = value else {
        return dependants_invalid("dependants must be a list of objects.");
    };
    if items.len() > MAX_DEPENDANTS {
        return dependants_invalid(format!("dependants takes at most {MAX_DEPENDANTS}."));
    }
    let mut read = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let n = i + 1;
        let Value::Object(fields) = item else {
            return dependants_invalid(format!("dependant {n} must be an object."));
        };
        if let Some(key) = fields
            .keys()
            .find(|k| !DEPENDANT_FIELDS.contains(&k.as_str()))
        {
            return dependants_invalid(format!(
                "dependant {n}: {key} is not a field; give kind, adjusted_taxable_income and optional days."
            ));
        }
        if fields.get("shared") == Some(&Value::Bool(true)) {
            return refuse(
                "zone_tax_offset.shared_maintenance",
                format!(
                    "dependant {n}: another person also maintained them; ITAA 1997 s 961-15(1)(c) leaves the apportionment to the Commissioner and the ATO publishes no method."
                ),
            );
        }
        if fields
            .get("shared")
            .is_some_and(|s| s != &Value::Bool(false))
        {
            return dependants_invalid(format!("dependant {n}: shared must be a boolean."));
        }
        let student = match fields.get("kind").and_then(Value::as_str) {
            Some("student") => true,
            Some("child") => false,
            _ => {
                return dependants_invalid(format!(
                    "dependant {n}: kind must be \"student\" (full-time, under 25) or \"child\" (not a student, under 21)."
                ));
            }
        };
        let income = match fields.get("adjusted_taxable_income") {
            Some(Value::String(text)) if text.len() <= 64 => Decimal::parse_input(text)
                .ok()
                .and_then(|v| money(&Arg::Decimal(v)).ok()),
            _ => None,
        };
        let Some(income) = income else {
            return dependants_invalid(format!(
                "dependant {n}: adjusted_taxable_income must be a non-negative decimal string, at most 2dp and 1e12."
            ));
        };
        let days = match fields.get("days") {
            None => days_in_year,
            Some(v) => match v.as_i64() {
                Some(days) if (0..=days_in_year).contains(&days) => days,
                _ => {
                    return dependants_invalid(format!(
                        "dependant {n}: days must be an integer from 0 to {days_in_year}."
                    ));
                }
            },
        };
        read.push((student, income, days));
    }
    // s 961-10(2): $282 for each non-student child after the oldest, where
    // there are 2 or more; the operator lists them oldest first.
    let children = read.iter().filter(|(student, ..)| !student).count();
    let mut seen_child = false;
    let mut total = int(0);
    let mut lines = Vec::new();
    for (student, income, days) in read {
        let full = if student || !seen_child || children < 2 {
            376
        } else {
            282
        };
        seen_child |= !student;
        let amount = notional(full, days, days_in_year, &income)?;
        total = total.add(&amount)?;
        lines.push(json!({
            "kind": if student { "student" } else { "child" },
            "full_amount": full.to_string(),
            "days": days,
            "notional_tax_offset": cents(amount)?.to_string(),
        }));
    }
    Ok((total, lines))
}

/// Whether the person is a resident of the area in the year (ITAA 1936
/// s 79A(3B)), and the paragraph that makes them one, or why not.
fn zone_residence(a: &ZoneArgs, year: &str, days: i64) -> Result<(bool, &'static str)> {
    let days_in_year = days_in(year);
    let carried = match (a.prior_year, a.prior_days) {
        (None, None) => {
            if a.continuous_residence.is_some() || a.includes_first_day.is_some() {
                return unknown("continuous_residence and includes_first_day are inputs of zone-tax-offset only with prior_year and prior_days".into());
            }
            None
        }
        (Some(prior_year), Some(prior_days)) => {
            let start: i64 = year[..4].parse().expect("income year");
            let prior = prior_year.text().filter(|t| {
                t.len() == 7
                    && t[..4].parse::<i64>().is_ok()
                    && t.as_bytes()[4] == b'-'
                    && t[5..].parse::<i64>().ok()
                        == t[..4].parse::<i64>().ok().map(|y| (y + 1) % 100)
            });
            let Some(prior) = prior else {
                return refuse(
                    "zone_tax_offset.prior_year_invalid",
                    "prior_year must be an income year such as \"2024-25\".",
                );
            };
            let gap = start - prior[..4].parse::<i64>().expect("checked");
            if gap == 5 {
                return refuse(
                    "zone_tax_offset.prior_year_unclear",
                    "ITAA 1936 s 79A(4) makes a relevant preceding year 'any of the next 4 preceding years of income other than the immediately preceding year', read here as the second to fourth; the ATO's Garry example (T4 2026) carries days from the fifth. The 2 readings differ only for the fifth year, which this worksheet does not decide.",
                );
            }
            if !(1..=4).contains(&gap) {
                return refuse(
                    "zone_tax_offset.prior_year_invalid",
                    format!(
                        "prior_year must be one of the 4 income years before {year} (ITAA 1936 s 79A(3B)(d) and (e), (4))."
                    ),
                );
            }
            let prior_length = days_in(prior);
            let prior_days = day_count(prior_days, prior_length, "prior_days")?;
            if prior_days * 2 > prior_length {
                return refuse(
                    "zone_tax_offset.prior_year_invalid",
                    format!(
                        "prior_days must be not more than one-half of {prior} (ITAA 1936 s 79A(3B)(d)(ii) and (e)(ii)): more makes the person a resident of the area that year, whose days are not carried."
                    ),
                );
            }
            let earlier = gap > 1;
            if !earlier && (a.continuous_residence.is_some() || a.includes_first_day.is_some()) {
                return unknown("continuous_residence and includes_first_day are inputs of zone-tax-offset only with a prior_year 2 to 4 years before (s 79A(3B)(e))".into());
            }
            let continuous = refused_flag(a.continuous_residence, "continuous_residence")?;
            let first_day = refused_flag(a.includes_first_day, "includes_first_day")?;
            Some((prior_days, earlier, continuous && first_day))
        }
        _ => {
            return unknown(
                "prior_year and prior_days are inputs of zone-tax-offset only together".into(),
            );
        }
    };
    if days * 2 > days_in_year {
        return Ok((true, "s 79A(3B)(a): more than one-half of the year"));
    }
    let mut why = "s 79A(3B)(a) not met: not more than one-half of the year";
    if let Some((prior_days, earlier, conditions)) = carried {
        if days + prior_days <= 182 {
            why = "s 79A(3B)(a), (d) and (e) not met: not more than one-half of the year, and the days carried add to not more than 182";
        } else if !earlier {
            return Ok((
                true,
                "s 79A(3B)(d): not more than one-half of the year and of the immediately preceding year, adding to more than 182 days",
            ));
        } else if conditions {
            return Ok((
                true,
                "s 79A(3B)(e): not more than one-half of the year, from its first day, and of a relevant preceding year, adding to more than 182 days, with continuous residence between",
            ));
        } else {
            why = "s 79A(3B)(e) not met: the period this year must include its first day and the residence must be continuous from the relevant preceding year";
        }
    }
    if days_in_year == 366 && days == 183 {
        return refuse(
            "zone_tax_offset.half_year_unclear",
            "183 days is exactly one-half of a 366-day year: ITAA 1936 s 79A(3B)(a) needs more than one-half, but the ATO says 183 days or more; this worksheet does not decide.",
        );
    }
    Ok((false, why))
}

const RELEVANT: &str = "ITAA 1936 s 79A(4) (relevant rebate amount) and s 79B(6) (concessional rebate amount): the invalid and invalid carer tax offset (ITAA 1997 Subdivision 61-A) plus the dependant notional tax offsets (Subdivision 961-A) and the sole parent notional tax offset (Subdivision 961-B)";
const DEPENDANTS: &str = "ITAA 1997 ss 961-5 to 961-20: $376 for each full-time student under 25 and for the oldest non-student child under 21, $282 for each other non-student child; for part of the year x days / days in the year (ATO T4 worksheet 1); less $1 for every $4 of adjusted taxable income over $282, without cents (T4 worksheet 2)";
const SOLE_PARENT: &str = "ITAA 1997 ss 961-55 to 961-65: $1,607 for sole care for the whole year of a dependant with a 961-A notional tax offset and no spouse; for part of the year days x $1,607 / days in the year to the cent, the ATO's daily rate (T4 worksheet 3)";
const ZONE_FULL: &str = "ITAA 1936 s 79A(2)(a) to (c): special area $1,173 + 50%, Zone A $338 + 50%, Zone B $57 + 20% of the relevant rebate amount; s 79B(2)(a): overseas forces $338 + 50% of the concessional rebate amount";
const ZONE_OFFSET: &str = "ITAA 1936 s 79A(1) and (2A): for a resident of the area (s 79A(3B)), the amount less any remote area allowance, not below nil; without cents (ATO T4 label R)";
const FORCES_OFFSET: &str = "ITAA 1936 s 79B(2): the amount for service at overseas localities for more than one-half of the year (counting Defence Force service in the prescribed area, s 79B(3)); otherwise the Commissioner's reasonable amount, the amount x days / 183 (ATO T4 worksheet 6); without cents (T4 label R)";

/// The zone or overseas forces tax offset (ITAA 1936 ss 79A and 79B).
pub fn zone_tax_offset(a: &ZoneArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, ZONE)?;
    let Some((area, fixed, percentage)) = a
        .area
        .text()
        .and_then(|t| AREAS.iter().find(|(name, ..)| *name == t))
        .copied()
    else {
        return refuse(
            "zone_tax_offset.area_invalid",
            "area must be \"special_area\", \"zone_a\", \"zone_b\" or \"overseas_forces\": the zone a place is in is the operator's, from ITAA 1936 Schedule 2 and the ATO's Australian zone list.",
        );
    };
    let forces = area == "overseas_forces";
    if forces {
        for (name, given) in [
            ("prior_year", a.prior_year),
            ("prior_days", a.prior_days),
            ("continuous_residence", a.continuous_residence),
            ("includes_first_day", a.includes_first_day),
            ("remote_area_allowance", a.remote_area_allowance),
        ] {
            if given.is_some() {
                return unknown(format!(
                    "{name} is not an input of zone-tax-offset for overseas_forces: s 79B carries no days from an earlier year and s 79A(2A)'s remote area allowance reduces the zone offset only"
                ));
            }
        }
    }
    let days_in_year = days_in(&year);
    let days = day_count(a.days, days_in_year, "days")?;
    let (resident, residence) = if forces {
        (true, "")
    } else {
        zone_residence(a, &year, days)?
    };
    let (dependants, lines) = dependant_offsets(a.dependants, days_in_year)?;
    let sole_days = a
        .sole_parent_days
        .map(|v| day_count(v, days_in_year, "sole_parent_days"))
        .transpose()?;
    let carer = optional_money(a.invalid_carer_tax_offset)?.unwrap_or_else(|| int(0));
    let allowance = optional_money(a.remote_area_allowance)?.unwrap_or_else(|| int(0));
    precise(|| {
        let sole = match sole_days {
            None => int(0),
            Some(_) if dependants.lt(&int(1))? => {
                return refuse(
                    "zone_tax_offset.sole_parent_without_dependant",
                    "The sole parent notional tax offset needs a dependant notional tax offset of at least $1 (ITAA 1997 s 961-55(1)(b); ATO T4).",
                );
            }
            Some(n) if n == days_in_year => int(1607),
            Some(n) => cents(int(1607).div(&int(days_in_year))?)?.mul(&int(n))?,
        };
        let relevant = dependants.add(&sole)?.add(&carer)?;
        let full = int(fixed).add(&d(percentage).mul(&relevant)?)?;
        let (offset, fraction) = if forces {
            if days * 2 > days_in_year {
                (full.clone(), None)
            } else {
                let share = full.mul(&int(days))?.div(&int(183))?;
                (share, Some(format!("{days}/183")))
            }
        } else if resident {
            (at_least_nil(full.sub(&allowance)?)?, None)
        } else {
            (int(0), None)
        };
        let offset = dollars(&offset)?;
        let mut amounts = vec![
            ("dependant_notional_tax_offsets", dependants),
            ("sole_parent_notional_tax_offset", sole),
            ("invalid_carer_tax_offset", carer),
            ("relevant_rebate_amount", relevant),
            ("fixed_amount", int(fixed)),
            ("full_amount", full),
        ];
        if !forces {
            amounts.push(("remote_area_allowance", allowance));
        }
        amounts.push(("tax_offset", offset));
        let mut out = result(
            ZONE,
            &year,
            amounts,
            vec![
                rate("fixed_amount", fixed),
                rate("percentage", percentage),
                rate("days_in_year", days_in_year),
            ],
        )?;
        let mut extra = Map::new();
        extra.insert("area".into(), json!(area));
        extra.insert("days".into(), json!(days));
        if forces {
            extra.insert("full_offset_days".into(), json!(days * 2 > days_in_year));
            if let Some(fraction) = fraction {
                extra.insert("fraction".into(), json!(fraction));
            }
        } else {
            extra.insert("resident".into(), json!(resident));
            extra.insert("residence".into(), json!(residence));
        }
        extra.insert("dependants".into(), Value::Array(lines));
        extra.insert(
            "authorities".into(),
            json!({
                "dependant_notional_tax_offsets": DEPENDANTS,
                "sole_parent_notional_tax_offset": SOLE_PARENT,
                "relevant_rebate_amount": RELEVANT,
                "full_amount": ZONE_FULL,
                "tax_offset": if forces { FORCES_OFFSET } else { ZONE_OFFSET },
            }),
        );
        if let Value::Object(map) = &mut out {
            map.extend(extra);
        }
        Ok(out)
    })
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

const MEDICAL_INCOME: &str = "ITAA 1936 s 159Q(1) (compilation 161): married on the last day of the year, the person's and the spouse's adjusted taxable income for rebates against the family tier 1 threshold; otherwise, with a dependant on any day, the person's own against the family tier 1 threshold; otherwise the person's own against the singles tier 1 threshold";
const MEDICAL_OFFSET: &str = "ITAA 1936 s 159P(3AA) (compilation 161): 10% of the net medical expenses over the higher phase-in limit where the income exceeds the threshold, otherwise 20% over the lower phase-in limit (s 159Q(5) and (6), indexed), as the ATO publishes them for the year; only disability aids, attendant care and aged care count (s 159P(1B))";

/// The net medical expenses tax offset for 2016-17 to 2018-19 (ITAA 1936
/// ss 159P and 159Q).
pub fn net_medical_expenses_tax_offset(a: &MedicalArgs) -> Result<Value> {
    if matches!(a.scope_confirmed, Arg::Bool(true))
        && a.year
            .text()
            .is_some_and(|y| y.len() == 7 && y[..4].parse::<i64>().is_ok_and(|start| start >= 2019))
    {
        return refuse(
            "period.unsupported",
            "ITAA 1936 s 159P(1A): no net medical expenses tax offset for 2019-20 or a later year; supported: 2016-17, 2017-18, 2018-19.",
        );
    }
    let year = scope(a.scope_confirmed, a.year, MEDICAL)?;
    let net = money(a.net_medical_expenses)?;
    let own = money(a.adjusted_taxable_income)?;
    let spouse = optional_money(a.spouse_adjusted_taxable_income)?;
    let children = dependent_children(a.dependent_children)?;
    let [lower, higher] = lookup(meta::NET_MEDICAL_LIMITS, &year).map(d);
    precise(|| {
        let family = spouse.is_some() || children > 0;
        let threshold = if family {
            int(180_000 + 1_500 * (children - 1).max(0))
        } else {
            int(90_000)
        };
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
        let mut out = result(
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
        )?;
        out["thresholds"] = json!(if family { "family" } else { "singles" });
        out["higher_limit_applies"] = json!(higher_applies);
        out["authorities"] = json!({
            "income_tested": MEDICAL_INCOME,
            "net_medical_expenses_tax_offset": MEDICAL_OFFSET,
        });
        Ok(out)
    })
}
