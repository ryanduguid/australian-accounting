// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The general interest charge and the shortfall interest charge, each
//! compounding daily and worked exactly, before remission.

use num_bigint::BigUint;
use serde_json::{Value, json};

use super::{
    Facts, KIND, Stretch, add_day, date_order, dated_amounts, dollars, fact_invalid, in_year,
    percent, rates, remission_note, stretches_json, unpublished, warn, year_length,
};
use crate::dates;
use crate::engine::{Result, rate, refuse, result};

const GIC_RATE: &str = "TAA 1953 s 8AAD(1): the general interest charge rate for a day is the base interest rate for the day plus 7 percentage points, divided by the number of days in the calendar year; the annual rate for each quarter as the ATO publishes it ('General interest charge (GIC) rates', last updated 4 September 2026)";
const GIC_DAYS: &str = "ITAA 1997 s 5-15 and TAA 1953 s 8AAC(3): GIC for each day from the start of the due day to the end of the last day at whose end any of the amount or its GIC remains unpaid, on the unpaid amount plus unpaid GIC from previous days, so it compounds daily; a payment on a day is applied before that day's charge; worked exactly and rounded half up to the cent once";
const SIC_RATE: &str = "TAA 1953 Sch 1 s 280-105(2): the base interest rate plus 3 percentage points, divided by the number of days in the calendar year; the annual rate for each quarter as the ATO publishes it ('Shortfall interest charge (SIC) rates', last updated 4 September 2026)";
const SIC_DAYS: &str = "TAA 1953 Sch 1 ss 280-100(1) and (2) and 280-105(1): SIC on the additional income tax for each day from the start of the day the first assessment's tax was due (or the earlier amended assessment's, for a reinstated liability, s 280-100(3)) to the end of the day before notice of the amended assessment, on the additional tax plus SIC from previous days; paying early does not stop it (PS LA 2006/8 paras 75 to 78); worked exactly and rounded half up to the cent once";

/// Half up on a non-negative exact ratio, in cents.
fn half_up(numerator: &BigUint, denominator: &BigUint) -> u128 {
    let two = BigUint::from(2_u8);
    ((&two * numerator + denominator) / (&two * denominator))
        .try_into()
        .expect("an amount within u128 cents")
}

/// A day's annual rate in hundredths of 1% and whether it was assumed.
type RateOf<'a> = &'a dyn Fn(i64) -> Result<(u32, bool)>;

/// The exact balance at the end of `last` of `amount` cents unpaid from the
/// start of `first`, charged daily at `rate_of`, with each payment applied on
/// its day before that day's charge, as a ratio and the stretches of days.
fn compound(
    rate_of: RateOf,
    amount: u64,
    first: i64,
    last: i64,
    payments: &[(i64, u64)],
) -> Result<(BigUint, BigUint, Vec<Stretch>)> {
    let (mut numerator, mut denominator) = (BigUint::from(amount), BigUint::from(1_u8));
    let mut stretches = Vec::new();
    for day in first..=last {
        for (_, paid) in payments.iter().filter(|(on, _)| *on == day) {
            let owed = BigUint::from(*paid) * &denominator;
            if owed > numerator {
                return refuse(
                    "ato_interest_penalties.overpaid",
                    format!(
                        "The payment of {} on {} is more than the amount owing that day.",
                        dollars(u128::from(*paid)),
                        dates::format(day)
                    ),
                );
            }
            numerator -= owed;
        }
        let (rate, assumed) = rate_of(day)?;
        let base = 10_000 * year_length(day);
        numerator *= base + rate;
        denominator *= base;
        add_day(&mut stretches, day, rate, assumed);
    }
    Ok((numerator, denominator, stretches))
}

/// GIC payments: a list of `{date, amount}` from the due day to `to`.
fn payments(f: &Facts, due: i64, to: i64) -> Result<Vec<(i64, u64)>> {
    // No payments may be left out or given as an empty list.
    if !f.given("payments") || f.map["payments"].as_array().is_some_and(Vec::is_empty) {
        return Ok(Vec::new());
    }
    let paid = dated_amounts(f, "date")?;
    if let Some(i) = paid.iter().position(|(day, _)| *day < due || *day > to) {
        return date_order(format!(
            "payment {}: date must be from due_date to to: a payment after to does not change the GIC to then.",
            i + 1
        ));
    }
    Ok(paid)
}

/// An optional assumed annual rate for days after the published quarters: a
/// percentage above 0 and at most 100 with at most 2 decimal places.
fn assumed_rate(f: &Facts) -> Result<Option<u32>> {
    if !f.given("assumed_annual_rate") {
        return Ok(None);
    }
    let cents = f.cents("assumed_annual_rate")?;
    match u32::try_from(cents) {
        Ok(rate) if rate > 0 && rate <= 10_000 => Ok(Some(rate)),
        _ => fact_invalid(format!(
            "{}: assumed_annual_rate must be a percentage above 0 and at most 100.",
            f.item()
        )),
    }
}

/// A day's GIC rate: published, or after the table the assumed rate.
fn gic_rate(day: i64, assumed: Option<u32>) -> Result<(u32, bool)> {
    match (rates::gic(day), assumed) {
        (Some(rate), _) => Ok((rate, false)),
        (None, Some(rate)) if day > rates::table_end() => Ok((rate, true)),
        _ => unpublished(format!(
            "The ATO has published GIC rates to {}; give assumed_annual_rate to project {} and later.",
            rates::TABLE_END,
            dates::format(day)
        )),
    }
}

/// The amount and its first and last days, checked.
fn gic_days(f: &Facts, period: &str) -> Result<(u64, i64, i64)> {
    let amount = f.cents("amount")?;
    let (due, to) = (f.date("due_date")?, f.date("to")?);
    in_year(to, period, "to")?;
    if to < due {
        return date_order(format!(
            "{}: to must be on or after due_date, the first day GIC accrues.",
            f.item()
        ));
    }
    if due < rates::first_day(&rates::GIC) {
        return unpublished("GIC rates are published from 1 July 1999.".into());
    }
    Ok((amount, due, to))
}

/// Counts the days at the assumed rate and warns when there are any.
fn note_assumed(out: &mut Value, stretches: &[Stretch]) {
    let days: i64 = stretches
        .iter()
        .filter(|s| s.assumed)
        .map(|s| s.to - s.from + 1)
        .sum();
    out["assumed_days"] = json!(days);
    if days > 0 {
        warn(
            out,
            format!(
                "{days} days after {} use the assumed rate: the GIC for them is an estimate, not a published rate.",
                rates::TABLE_END
            ),
        );
    }
}

pub(super) fn gic(f: &Facts, period: &str) -> Result<Value> {
    f.only(&[
        "amount",
        "due_date",
        "to",
        "payments",
        "assumed_annual_rate",
    ])?;
    let (amount, due, to) = gic_days(f, period)?;
    let assumed = assumed_rate(f)?;
    let paid = payments(f, due, to)?;
    let rate_of = |day: i64| gic_rate(day, assumed);
    let (numerator, denominator, stretches) = compound(&rate_of, amount, due, to, &paid)?;
    let balance = half_up(&numerator, &denominator);
    let total_paid: u128 = paid.iter().map(|(_, cents)| u128::from(*cents)).sum();
    let charge = balance + total_paid - u128::from(amount);
    let mut out = result(
        KIND,
        period,
        vec![
            ("gic", dollars(charge)),
            ("paid", dollars(total_paid)),
            ("balance", dollars(balance)),
        ],
        assumed
            .map(|r| rate("assumed_annual_rate_percent", percent(r)))
            .into_iter()
            .collect(),
    )?;
    out["question"] = json!("gic");
    out["days"] = json!(to - due + 1);
    out["stretches"] = stretches_json(&stretches);
    out["rate_authority"] = json!(GIC_RATE);
    out["gic_authority"] = json!(GIC_DAYS);
    remission_note(&mut out);
    note_assumed(&mut out, &stretches);
    Ok(out)
}

pub(super) fn sic(f: &Facts, period: &str) -> Result<Value> {
    f.only(&["amount", "due_date", "notice_date"])?;
    let amount = f.cents("amount")?;
    let (due, notice) = (f.date("due_date")?, f.date("notice_date")?);
    in_year(notice, period, "notice_date")?;
    let first = rates::first_day(&rates::SIC);
    if due < first && notice > due {
        return unpublished("SIC rates are published from 1 April 2005.".into());
    }
    let rate_of = |day: i64| match rates::sic(day) {
        Some(rate) => Ok((rate, false)),
        None => unpublished(format!(
            "The ATO has published SIC rates to {}; {} is later.",
            rates::TABLE_END,
            dates::format(day)
        )),
    };
    let (numerator, denominator, stretches) = compound(&rate_of, amount, due, notice - 1, &[])?;
    let charge = half_up(&numerator, &denominator) - u128::from(amount);
    let mut out = result(KIND, period, vec![("sic", dollars(charge))], Vec::new())?;
    out["question"] = json!("sic");
    out["days"] = json!((notice - due).max(0));
    if notice > due {
        out["first_day"] = json!(dates::format(due));
        out["last_day"] = json!(dates::format(notice - 1));
    }
    out["stretches"] = stretches_json(&stretches);
    out["rate_authority"] = json!(SIC_RATE);
    out["sic_authority"] = json!(SIC_DAYS);
    remission_note(&mut out);
    Ok(out)
}
