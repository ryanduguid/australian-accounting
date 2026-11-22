// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The zone and overseas forces tax offsets (ITAA 1936 ss 79A and 79B), with
//! the notional tax offsets of ITAA 1997 Division 961 that their relevant
//! rebate amount counts.

use serde_json::{Map, Value, json};

use super::{
    at_least_nil, days_in, dollars, income_reduction, optional_money, refused_flag, unknown,
};
use crate::decimal::Decimal;
use crate::engine::{
    Arg, Result, cents, d, day_count, int, money, precise, rate, refuse, result, scope,
};

const ZONE: &str = "zone_tax_offset";

/// The most dependants one zone or overseas forces claim takes.
const MAX_DEPENDANTS: usize = 50;

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

/// Dependant `n`'s adjusted taxable income and days in the year.
fn dependant_figures(
    n: usize,
    fields: &Map<String, Value>,
    days_in_year: i64,
) -> Result<(Decimal, i64)> {
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
    Ok((income, days))
}

/// Dependant `n` as (student, adjusted taxable income, days).
fn read_dependant(n: usize, item: &Value, days_in_year: i64) -> Result<(bool, Decimal, i64)> {
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
    let (income, days) = dependant_figures(n, fields, days_in_year)?;
    Ok((student, income, days))
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
    let read = items
        .iter()
        .enumerate()
        .map(|(i, item)| read_dependant(i + 1, item, days_in_year))
        .collect::<Result<Vec<_>>>()?;
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

/// `prior_year` checked against `year`: the year and how many years before
/// `year` it is, 1 to 4 (ITAA 1936 s 79A(3B)(d) and (e), (4)).
fn prior_year_gap<'a>(prior_year: &'a Arg, year: &str) -> Result<(&'a str, i64)> {
    let start: i64 = year[..4].parse().expect("income year");
    let prior = prior_year.text().filter(|t| {
        t.len() == 7
            && t.is_ascii()
            && t[..4].parse::<i64>().is_ok()
            && t.as_bytes()[4] == b'-'
            && t[5..].parse::<i64>().ok() == t[..4].parse::<i64>().ok().map(|y| (y + 1) % 100)
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
    Ok((prior, gap))
}

/// The days carried from an earlier year, if given: (days, whether the year
/// is 2 to 4 years back, whether s 79A(3B)(e)'s conditions are met).
fn carried_days(a: &ZoneArgs, year: &str) -> Result<Option<(i64, bool, bool)>> {
    let conditions = a.continuous_residence.is_some() || a.includes_first_day.is_some();
    let (prior_year, prior_days) = match (a.prior_year, a.prior_days) {
        (None, None) if conditions => {
            return unknown("continuous_residence and includes_first_day are inputs of zone-tax-offset only with prior_year and prior_days".into());
        }
        (None, None) => return Ok(None),
        (Some(prior_year), Some(prior_days)) => (prior_year, prior_days),
        _ => {
            return unknown(
                "prior_year and prior_days are inputs of zone-tax-offset only together".into(),
            );
        }
    };
    let (prior, gap) = prior_year_gap(prior_year, year)?;
    let prior_length = days_in(prior);
    let prior_days = day_count(prior_days, prior_length, "prior_days")?;
    if prior_length == 366 && prior_days == 183 {
        return refuse(
            "zone_tax_offset.half_year_unclear",
            format!(
                "183 days is exactly one-half of {prior}, a 366-day year: ITAA 1936 s 79A(3B)(d)(ii) and (e)(ii) carry days of not more than one-half, but the ATO counts 183 days or more as residence for that year, whose days are not carried; this worksheet does not decide."
            ),
        );
    }
    if prior_days * 2 > prior_length {
        return refuse(
            "zone_tax_offset.prior_year_invalid",
            format!(
                "prior_days must be not more than one-half of {prior} (ITAA 1936 s 79A(3B)(d)(ii) and (e)(ii)): more makes the person a resident of the area that year, whose days are not carried."
            ),
        );
    }
    let earlier = gap > 1;
    if !earlier && conditions {
        return unknown("continuous_residence and includes_first_day are inputs of zone-tax-offset only with a prior_year 2 to 4 years before (s 79A(3B)(e))".into());
    }
    let continuous = refused_flag(a.continuous_residence, "continuous_residence")?;
    let first_day = refused_flag(a.includes_first_day, "includes_first_day")?;
    Ok(Some((prior_days, earlier, continuous && first_day)))
}

/// Whether the person is a resident of the area in the year (ITAA 1936
/// s 79A(3B)), and the paragraph that makes them one, or why not.
fn zone_residence(a: &ZoneArgs, year: &str, days: i64) -> Result<(bool, &'static str)> {
    let days_in_year = days_in(year);
    let carried = carried_days(a, year)?;
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

/// The area, its fixed amount and percentage, refusing inputs s 79B does not
/// take for overseas forces.
fn zone_area(a: &ZoneArgs) -> Result<(&'static str, i64, &'static str)> {
    let Some(found) = a
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
    if found.0 != "overseas_forces" {
        return Ok(found);
    }
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
    Ok(found)
}

/// The sole parent notional tax offset (ITAA 1997 ss 961-55 to 961-65).
fn sole_parent(days: Option<i64>, dependants: &Decimal, days_in_year: i64) -> Result<Decimal> {
    match days {
        None => Ok(int(0)),
        Some(_) if dependants.lt(&int(1))? => refuse(
            "zone_tax_offset.sole_parent_without_dependant",
            "The sole parent notional tax offset needs a dependant notional tax offset of at least $1 (ITAA 1997 s 961-55(1)(b); ATO T4).",
        ),
        Some(n) if n == days_in_year => Ok(int(1607)),
        Some(n) => Ok(cents(int(1607).div(&int(days_in_year))?)?.mul(&int(n))?),
    }
}

/// How the offset follows from the full amount: overseas service for a
/// fraction of the year, or zone residence with its paragraph.
enum Basis {
    Forces(Option<String>),
    Zone(bool, &'static str),
}

/// The output fields beside the amounts and rates.
fn zone_extra(
    area: &str,
    days: i64,
    days_in_year: i64,
    basis: Basis,
    lines: Vec<Value>,
) -> Map<String, Value> {
    let mut extra = Map::new();
    extra.insert("area".into(), json!(area));
    extra.insert("days".into(), json!(days));
    let forces = matches!(basis, Basis::Forces(_));
    match basis {
        Basis::Forces(fraction) => {
            extra.insert("full_offset_days".into(), json!(days * 2 > days_in_year));
            if let Some(fraction) = fraction {
                extra.insert("fraction".into(), json!(fraction));
            }
        }
        Basis::Zone(resident, residence) => {
            extra.insert("resident".into(), json!(resident));
            extra.insert("residence".into(), json!(residence));
        }
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
    extra
}

/// The offset from the full amount, before cents are dropped, and its basis.
fn offset_from_full(
    full: &Decimal,
    allowance: &Decimal,
    days: i64,
    days_in_year: i64,
    residence: Option<(bool, &'static str)>,
) -> Result<(Decimal, Basis)> {
    Ok(match residence {
        None if days * 2 > days_in_year => (full.clone(), Basis::Forces(None)),
        None => (
            full.mul(&int(days))?.div(&int(183))?,
            Basis::Forces(Some(format!("{days}/183"))),
        ),
        Some((true, why)) => (at_least_nil(full.sub(allowance)?)?, Basis::Zone(true, why)),
        Some((false, why)) => (int(0), Basis::Zone(false, why)),
    })
}

/// The zone or overseas forces tax offset (ITAA 1936 ss 79A and 79B).
pub fn zone_tax_offset(a: &ZoneArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, ZONE)?;
    let (area, fixed, percentage) = zone_area(a)?;
    let forces = area == "overseas_forces";
    let days_in_year = days_in(&year);
    let days = day_count(a.days, days_in_year, "days")?;
    let residence = if forces {
        None
    } else {
        Some(zone_residence(a, &year, days)?)
    };
    let (dependants, lines) = dependant_offsets(a.dependants, days_in_year)?;
    let sole_days = a
        .sole_parent_days
        .map(|v| day_count(v, days_in_year, "sole_parent_days"))
        .transpose()?;
    let carer = optional_money(a.invalid_carer_tax_offset)?.unwrap_or_else(|| int(0));
    let allowance = optional_money(a.remote_area_allowance)?.unwrap_or_else(|| int(0));
    precise(|| {
        let sole = sole_parent(sole_days, &dependants, days_in_year)?;
        let relevant = dependants.add(&sole)?.add(&carer)?;
        let full = int(fixed).add(&d(percentage).mul(&relevant)?)?;
        let (offset, basis) = offset_from_full(&full, &allowance, days, days_in_year, residence)?;
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
        amounts.push(("tax_offset", dollars(&offset)?));
        let rates = vec![
            rate("fixed_amount", fixed),
            rate("percentage", percentage),
            rate("days_in_year", days_in_year),
        ];
        let mut out = result(ZONE, &year, amounts, rates)?;
        if let Value::Object(map) = &mut out {
            map.extend(zone_extra(area, days, days_in_year, basis, lines));
        }
        Ok(out)
    })
}
