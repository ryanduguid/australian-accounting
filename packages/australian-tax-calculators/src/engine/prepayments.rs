// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `prepayments`: when one prepaid amount is deductible under ITAA 1936
//! Subdivision H (ss 82KZL to 82KZMF), for expenditure incurred in 2020-21
//! to 2026-27. The citations are to ITAA 1936 compilation 193
//! (1 October 2026), read on 22 November 2026; compilation 175 (4 April 2021)
//! reads the same, and the $50 million entity of ss 82KZM(1A) and 82KZMA(2A)
//! applies to expenditure incurred on or after 1 July 2020 (Act No. 92 of
//! 2020 Schedule 3 item 40(5)). Months are read under Acts Interpretation Act
//! 1901 s 2G(2).

use serde_json::{Value, json};

use super::{
    Arg, Result, date_arg, flag, income_year, income_year_start, int, money, refuse, result, scope,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "prepayments";

/// Section 82KZL(1) 'excluded expenditure' (a).
const EXCLUDED_BELOW: &str = "1000";
/// Section 82KZL(1) 'eligible service period' (d): 10 years as 120 months.
const MAX_MONTHS: i64 = 120;
/// Section 82KZM(1)(ba)(i).
const TWELVE_MONTHS: i64 = 12;
/// The last agreement day the command refuses: 11 November 1999, the day of
/// s 82KZME(2)(a) and (8), after the pre-RBT day of s 82KZL(1) (21 September
/// 1999) and the s 82KZM(1)(a) day (25 May 1988).
const CUTOFF: &str = "1999-11-11";

/// The facts as the command passes them; see the README for each.
pub struct PrepaymentsArgs<'a> {
    pub taxpayer: &'a Arg,
    pub individual: &'a Arg,
    pub business_expenditure: &'a Arg,
    pub chose_82kzmd: &'a Arg,
    pub expenditure: &'a Arg,
    pub incurred: &'a Arg,
    pub service_start: &'a Arg,
    pub service_end: &'a Arg,
    pub agreement_date: &'a Arg,
    pub exclusion: &'a Arg,
    pub tax_shelter: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The facts read and checked.
struct Facts {
    /// A small business entity, or one covered by s 82KZM(1A).
    small_or_medium: bool,
    individual: bool,
    business: bool,
    chose: bool,
    expenditure: Decimal,
    incurred: i64,
    start: i64,
    end: i64,
    excluded: bool,
    shelter: bool,
}

/// How the deduction falls: all in the expenditure year, or apportioned over
/// the eligible service period by days.
#[derive(Clone, Copy, PartialEq)]
enum Treatment {
    Immediate(&'static str),
    Apportioned(&'static str),
}

const AUTHORITIES: [(&str, &str); 7] = [
    (
        "excluded_expenditure",
        "ITAA 1936 s 82KZL(1) 'excluded expenditure': less than $1,000, required by a law or court order, or under a contract of service; Subdivision H does not apply and ITAA 1997 s 8-1 allows the deduction when incurred",
    ),
    (
        "not_prepaid",
        "ITAA 1936 s 82KZL(1) 'eligible service period': the thing was to be done no later than the day the expenditure was incurred, so no period remains; s 8-1 allows the deduction when incurred",
    ),
    (
        "done_in_year",
        "ITAA 1936 ss 82KZMA(3)(c) and 82KZME(2)(b): the thing is wholly done within the expenditure year, and s 82KZM(1)(ba) is not met; s 8-1 allows the deduction when incurred",
    ),
    (
        "twelve_month_rule",
        "ITAA 1936 s 82KZM(1)(aa) and (ba): a small business entity or an entity covered by s 82KZM(1A) that has not chosen s 82KZMD, or an individual's non-business expenditure, with an eligible service period of no more than 12 months ending by the last day of the next income year; s 8-1 allows the deduction when incurred",
    ),
    (
        "apportioned_82kzm",
        "ITAA 1936 s 82KZM(1): the deduction x days of the eligible service period in the year / days of the eligible service period, for each year the period touches",
    ),
    (
        "apportioned_82kzmd",
        "ITAA 1936 ss 82KZMA and 82KZMD(2): expenditure x days of the eligible service period in the year / total days of the eligible service period, for business expenditure, a non-individual's non-business expenditure, or a small or medium business's choice under s 82KZMA(2)(b)",
    ),
    (
        "apportioned_82kzmf",
        "ITAA 1936 ss 82KZME and 82KZMF(1): an agreement the operator states s 82KZME applies to; expenditure x days of the eligible service period in the year / total days; s 8-1 no longer allows the whole deduction when incurred, so s 82KZM(1)(c) is not met",
    ),
];

fn taxpayer(given: &Arg) -> Result<bool> {
    match given.text() {
        Some("small_business_entity" | "medium_business_entity") => Ok(true),
        Some("other") => Ok(false),
        _ => refuse(
            "prepayments.taxpayer_invalid",
            "taxpayer must be small_business_entity, medium_business_entity or other.",
        ),
    }
}

/// Whether the expenditure is excluded expenditure (s 82KZL(1)), by amount or
/// by the stated limb.
fn excluded(given: &Arg, expenditure: &Decimal) -> Result<bool> {
    let stated = match given.text() {
        Some("none") => false,
        Some("law_or_court_order" | "contract_of_service") => true,
        _ => {
            return refuse(
                "prepayments.exclusion_invalid",
                "exclusion must be none, law_or_court_order or contract_of_service.",
            );
        }
    };
    Ok(stated || expenditure.lt(&Decimal::parse(EXCLUDED_BELOW)?)?)
}

fn dates_invalid<T>(why: &str) -> Result<T> {
    refuse("prepayments.dates_invalid", why.to_owned())
}

/// The dates, checked against each other and the expenditure year.
fn read_dates(args: &PrepaymentsArgs, first: i64) -> Result<[i64; 4]> {
    let incurred = date_arg(args.incurred, "incurred")?;
    let start = date_arg(args.service_start, "service_start")?;
    let end = date_arg(args.service_end, "service_end")?;
    let agreement = date_arg(args.agreement_date, "agreement_date")?;
    if !(income_year_start(first)..income_year_start(first + 1)).contains(&incurred) {
        return dates_invalid("incurred must fall in the income year.");
    }
    if end < start {
        return dates_invalid("service_end must not be before service_start.");
    }
    if agreement > incurred {
        return dates_invalid("agreement_date must not be after incurred.");
    }
    if agreement <= dates::parse(CUTOFF).expect("cutoff") {
        return refuse(
            "prepayments.agreement_before_cutoff",
            "An agreement entered into on or before 11 November 1999 may carry a pre-RBT obligation (ITAA 1936 s 82KZL(1)) or a pre-existing obligation (s 82KZME(8)), whose rules this worksheet does not apply.",
        );
    }
    Ok([incurred, start, end, agreement])
}

fn read_facts(args: &PrepaymentsArgs, first: i64) -> Result<Facts> {
    let small_or_medium = taxpayer(args.taxpayer)?;
    let chose = flag(args.chose_82kzmd, "chose_82kzmd")?;
    if chose && !small_or_medium {
        return refuse(
            "prepayments.choice_unavailable",
            "Only a small business entity or an entity covered by ITAA 1936 s 82KZMA(2A) chooses to apply s 82KZMD (s 82KZMA(2)(b)).",
        );
    }
    let expenditure = money(args.expenditure)?;
    let [incurred, start, end, _] = read_dates(args, first)?;
    Ok(Facts {
        small_or_medium,
        individual: flag(args.individual, "individual")?,
        business: flag(args.business_expenditure, "business_expenditure")?,
        chose,
        excluded: excluded(args.exclusion, &expenditure)?,
        shelter: flag(args.tax_shelter, "tax_shelter")?,
        expenditure,
        incurred,
        start,
        end,
    })
}

/// The eligible service period (s 82KZL(1)): from the later of the first
/// service day and the day incurred, to the last service day, at most 10
/// years; `None` when the service ended before the expenditure was incurred.
fn service_period(facts: &Facts) -> Option<(i64, i64)> {
    let start = facts.start.max(facts.incurred);
    (facts.end >= start).then(|| (start, facts.end.min(dates::months_end(start, MAX_MONTHS))))
}

fn treatment(facts: &Facts, first: i64, period: Option<(i64, i64)>) -> Treatment {
    use Treatment::{Apportioned, Immediate};
    let year_end = income_year_start(first + 1) - 1;
    if facts.excluded {
        return Immediate("excluded_expenditure");
    }
    let Some((start, end)) = period else {
        return Immediate("not_prepaid");
    };
    if end <= year_end {
        Immediate("done_in_year")
    } else if facts.shelter {
        Apportioned("apportioned_82kzmf")
    } else if (facts.small_or_medium && !facts.chose) || (facts.individual && !facts.business) {
        let next_year_end = income_year_start(first + 2) - 1;
        if end <= dates::months_end(start, TWELVE_MONTHS) && end <= next_year_end {
            Immediate("twelve_month_rule")
        } else {
            Apportioned("apportioned_82kzm")
        }
    } else {
        Apportioned("apportioned_82kzmd")
    }
}

/// Each income year's deduction from the expenditure year on. Apportioned
/// amounts are worked on the running total of days and rounded to the cent
/// half up there, so the years add up to the expenditure.
fn schedule(
    expenditure: &Decimal,
    first: i64,
    treatment: Treatment,
    period: Option<(i64, i64)>,
) -> Result<Vec<(i64, Option<i64>, Decimal)>> {
    let (Treatment::Apportioned(_), Some((start, end))) = (treatment, period) else {
        return Ok(vec![(first, None, expenditure.clone())]);
    };
    let total = int(end - start + 1);
    let (mut rows, mut counted, mut allowed) = (Vec::new(), 0, int(0));
    let mut year = first;
    while income_year_start(year) <= end {
        let from = start.max(income_year_start(year));
        let to = end.min(income_year_start(year + 1) - 1);
        let days = (to - from + 1).max(0);
        counted += days;
        let running = super::precise(|| {
            Ok(expenditure
                .mul(&int(counted))?
                .div(&total)?
                .quantize(-2, Rounding::HalfUp)?)
        })?;
        rows.push((year, Some(days), running.sub(&allowed)?));
        allowed = running;
        year += 1;
    }
    Ok(rows)
}

pub fn prepayments(args: &PrepaymentsArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let first: i64 = year[..4].parse().expect("income year");
    let facts = read_facts(args, first)?;
    let period = service_period(&facts);
    let treatment = treatment(&facts, first, period);
    let rows = schedule(&facts.expenditure, first, treatment, period)?;
    let now = rows[0].2.clone();
    let mut out = result(
        KIND,
        &year,
        vec![
            ("expenditure", facts.expenditure.clone()),
            ("deduction_expenditure_year", now.clone()),
            ("deduction_later_years", facts.expenditure.sub(&now)?),
        ],
        vec![
            super::rate("excluded_below", EXCLUDED_BELOW),
            super::rate("longest_period_months", MAX_MONTHS),
        ],
    )?;
    let shown = if facts.excluded { None } else { period };
    explain(&mut out, treatment, shown, &rows)?;
    Ok(out)
}

fn explain(
    out: &mut Value,
    treatment: Treatment,
    period: Option<(i64, i64)>,
    rows: &[(i64, Option<i64>, Decimal)],
) -> Result<()> {
    let (Treatment::Immediate(id) | Treatment::Apportioned(id)) = treatment;
    let authority = AUTHORITIES
        .iter()
        .find(|(k, _)| *k == id)
        .expect("authority")
        .1;
    out["treatment"] = json!(id);
    out["authority"] = json!(authority);
    out["eligible_service_period"] = match period {
        Some((start, end)) => json!({
            "start": dates::format(start),
            "end": dates::format(end),
            "days": end - start + 1,
        }),
        None => Value::Null,
    };
    let schedule = rows
        .iter()
        .map(|(year, days, amount)| {
            let amount = super::cents(amount.clone())?.to_string();
            Ok(json!({"year": income_year(*year), "days": days, "deduction": amount}))
        })
        .collect::<Result<Vec<Value>>>()?;
    out["schedule"] = json!(schedule);
    Ok(())
}
