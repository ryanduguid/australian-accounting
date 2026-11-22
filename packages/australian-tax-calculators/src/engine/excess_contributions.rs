// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `excess_contributions`: the excess concessional contributions charge
//! (TAA 1953 Sch 1 former Div 95; Superannuation (Excess Concessional
//! Contributions Charge) Act 2013 ss 4 and 5), for 2013-14 to 2020-21, and
//! the release of excess non-concessional contributions with their associated
//! earnings (TAA 1953 Sch 1 ss 97-25 and 97-30; ITAA 1997 ss 292-20 to
//! 292-30), for contributions years 2013-14 to 2025-26. Estimates of what the
//! Commissioner's determination states. Read on 22 November 2026: TAA
//! compilation 227 (C2026C00468) and, for Div 95, compilation 180
//! (C2021C00276); the 2013 Charge Act (C2013A00116); Act No. 45 of 2021
//! Sch 2; and the ATO's 'Contributions caps' page (last updated 11 September
//! 2026), Tables 3 and 5.

use num_bigint::BigInt;
use serde_json::{Map, Value, json};

use super::ato_interest_penalties::rates;
use super::transfer_balance::dollars;
use super::{Arg, Result, Step, d, rate, refuse, result, scope, stated, step};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "excess_contributions";
const QUESTIONS: [&str; 2] = ["concessional_charge", "non_concessional_release"];

const CHARGE: &str = "TAA 1953 Sch 1 former ss 95-10 and 95-15 (compilation 180): the charge for each day from the first day of the income year to the day before tax under the first notice of assessment is due, at the day's rate on the tax plus the charge from previous days, so it compounds daily; worked exactly and rounded half up to the cent once";
const CHARGE_RATE: &str = "Superannuation (Excess Concessional Contributions Charge) Act 2013 ss 4 and 5: the base interest rate plus 3 percentage points over the days in the calendar year, the rate of the ATO's 'Contributions caps' Table 3 and of the shortfall interest charge (TAA 1953 Sch 1 s 280-105(2))";
const EARNINGS: &str = "TAA 1953 Sch 1 s 97-30(1): for each day from the first day of the contributions year to the day of the first determination, proxy rate x (excess + sum of earlier daily proxy amounts); the sum rounded down to the dollar";
const PROXY: &str = "TAA 1953 Sch 1 s 97-30(1) 'proxy rate': the s 8AAD(1) rate for the first day of the contributions year with the base interest rate replaced by the average of the base interest rates for each day of the year (the base interest rate plus 7 points over the days in that calendar year); no lower rate has been determined under s 97-30(2)";
const RELEASE: &str = "TAA 1953 Sch 1 s 97-25(1)(c): the excess + 0.85 x the associated earnings";
const ASSESSABLE: &str = "ITAA 1997 ss 292-20 and 292-25(1): the associated earnings, when the total release amount is released in full";
const OFFSET: &str =
    "ITAA 1997 s 292-30: a non-refundable tax offset of 15% of the amount in assessable income";

/// The ATO's published associated earnings rates ('Contributions caps',
/// Table 5): (contributions year, annual rate, daily rate), for the warning.
const ATO_TABLE_5: [(&str, &str, &str); 13] = [
    ("2013-14", "9.66", "0.02646575"),
    ("2014-15", "9.61", "0.02632876"),
    ("2015-16", "9.20", "0.02513661"),
    ("2016-17", "8.83", "0.02419178"),
    ("2017-18", "8.73", "0.02391780"),
    ("2018-19", "8.96", "0.02454795"),
    ("2019-20", "8.08", "0.02207650"),
    ("2020-21", "7.06", "0.01934247"),
    ("2021-22", "7.04", "0.01928767"),
    ("2022-23", "9.46", "0.02591781"),
    ("2023-24", "11.19", "0.03057377"),
    ("2024-25", "11.33", "0.03104110"),
    ("2025-26", "10.75", "0.02945205"),
];

fn fact_invalid<T>(message: String) -> Result<T> {
    refuse("excess_contributions.fact_invalid", message)
}

fn date_order<T>(message: String) -> Result<T> {
    refuse("excess_contributions.date_order", message)
}

fn first_day(start: i64) -> i64 {
    dates::parse(&format!("{start}-07-01")).expect("1 July")
}

fn year_length(day: i64) -> i64 {
    let (y, _, _) = dates::to_civil(day);
    if super::is_leap(y) { 366 } else { 365 }
}

fn cents(amount: &Decimal) -> Result<BigInt> {
    let text = amount.quantize(-2, Rounding::HalfUp)?.to_string();
    Ok(text
        .replace('.', "")
        .parse()
        .expect("cents of a checked amount"))
}

fn fact_date(facts: &Map<String, Value>, item: &str, key: &str) -> Result<i64> {
    match stated::required(facts, item, key)?
        .as_str()
        .and_then(dates::parse)
    {
        Some(day) => Ok(day),
        None => fact_invalid(format!("{item}: {key} must be a date YYYY-MM-DD.")),
    }
}

/// The charge on `tax` cents from 1 July of `start` to the day before `due`.
fn charge(tax: &BigInt, start: i64, due: i64) -> Result<BigInt> {
    let (mut num, mut den) = (tax.clone(), BigInt::from(1));
    for day in first_day(start)..due {
        let Some(rate) = rates::sic(day) else {
            return refuse(
                "excess_contributions.rate_unpublished",
                format!(
                    "The ATO has published the charge's quarterly rate to {}; {} is later.",
                    rates::TABLE_END,
                    dates::format(day)
                ),
            );
        };
        let base = BigInt::from(10_000 * year_length(day));
        num *= &base + rate;
        den *= base;
    }
    let two = BigInt::from(2);
    Ok((&two * num + &den) / (&two * den) - tax)
}

fn concessional_charge(facts: &Map<String, Value>, period: &str) -> Result<Value> {
    let item = "question concessional_charge";
    stated::known_keys(
        facts,
        item,
        "concessional_charge",
        &["additional_tax", "tax_due_date"],
    )?;
    let start: i64 = period[..4].parse().expect("a supported year");
    if start >= 2021 {
        return refuse(
            "excess_contributions.charge_abolished",
            "No excess concessional contributions charge arises for 2021-22 or a later year: Treasury Laws Amendment (More Flexible Superannuation) Act 2021 Sch 2 repealed TAA 1953 Sch 1 Div 95 for excess concessional contributions for a financial year starting on or after 1 July 2021 (item 14).",
        );
    }
    let tax = cents(&stated::amount(facts, item, "additional_tax")?)?;
    let due = fact_date(facts, item, "tax_due_date")?;
    if due <= first_day(start) {
        return date_order(format!(
            "{item}: tax_due_date must be after 1 July of the year, the charge's first day."
        ));
    }
    let owed = charge(&tax, start, due)?;
    let one = BigInt::from(1);
    let mut out = result(
        KIND,
        period,
        vec![(
            "excess_concessional_contributions_charge",
            dollars(&owed, &one),
        )],
        Vec::new(),
    )?;
    out["question"] = json!("concessional_charge");
    out["first_day"] = json!(dates::format(first_day(start)));
    out["last_day"] = json!(dates::format(due - 1));
    out["days"] = json!(due - first_day(start));
    out["charge_authority"] = json!(CHARGE);
    out["rate_authority"] = json!(CHARGE_RATE);
    Ok(out)
}

/// A release's figures: the earnings days, the proxy rate, the associated
/// earnings, the total release amount, the tax offset and the ATO's figures.
pub(super) struct Release {
    days: i64,
    proxy_percent: String,
    earnings: Decimal,
    total: Decimal,
    offset: Decimal,
    warning: String,
}

/// The proxy rate's daily ratio for the contributions year from `start`, as
/// (numerator, denominator), and the annual percentage to 6 places.
fn proxy(start: i64) -> Result<(BigInt, BigInt, String)> {
    let (first, last) = (first_day(start), first_day(start + 1) - 1);
    if rates::base(last).is_none() || start < 2013 {
        return refuse(
            "excess_contributions.rate_unpublished",
            format!(
                "The proxy rate averages the base interest rate for every day of the contributions year (TAA 1953 Sch 1 s 97-30); the ATO's GIC rates run to {}, and s 97-30 applies from 2013-14.",
                rates::TABLE_END
            ),
        );
    }
    let days = last - first + 1;
    let sum: i64 = (first..=last)
        .map(|day| i64::from(rates::base(day).expect("a published day")))
        .sum();
    let n = BigInt::from(sum + 700 * days);
    let den = BigInt::from(days * 10_000 * year_length(first));
    let micro = (BigInt::from(sum + 700 * days) * 10_000 * 2 / days + 1) / 2;
    let million = BigInt::from(1_000_000);
    let percent = format!("{}.{:06}", &micro / &million, &micro % &million);
    Ok((n, den, percent))
}

fn warning(start: i64, label: &str) -> String {
    let (_, annual, daily) = ATO_TABLE_5
        .iter()
        .find(|(y, _, _)| *y == label)
        .expect("a year with a proxy rate");
    let divisor = year_length(first_day(start + 1) - 1);
    format!(
        "The ATO publishes {annual}% a year and {daily}% a day for {label} ('Contributions caps', Table 5): a simple average of the 4 quarterly rates, rounded to 2 places, over the {divisor} days of the financial year. This command follows s 97-30's day-weighted average over the days of calendar year {start}, so an ATO determination using its published rate can differ by some dollars."
    )
}

/// The release figures for an excess (in dollars) of the contributions year
/// from `start`, the first determination made on `determination`.
pub(super) fn release(
    excess: &Decimal,
    start: i64,
    label: &str,
    determination: i64,
) -> Result<Release> {
    let (n, den, proxy_percent) = proxy(start)?;
    if determination < first_day(start) {
        return date_order(
            "determination_date must be on or after 1 July of the contributions year.".into(),
        );
    }
    let days = determination - first_day(start) + 1;
    let k = u32::try_from(days).expect("a period in days");
    let grown = (&den + &n).pow(k);
    let base = den.pow(k);
    let earned: BigInt = cents(excess)? * (grown - &base) / (base * 100);
    let earnings = d(&format!("{earned}"));
    let total = excess.add(&earnings.mul(&d("0.85"))?)?;
    let offset = earnings.mul(&d("0.15"))?;
    Ok(Release {
        days,
        proxy_percent,
        earnings,
        total,
        offset,
        warning: warning(start, label),
    })
}

fn non_concessional_release(facts: &Map<String, Value>, period: &str) -> Result<Value> {
    let item = "question non_concessional_release";
    stated::known_keys(
        facts,
        item,
        "non_concessional_release",
        &["excess", "determination_date"],
    )?;
    let excess = stated::amount(facts, item, "excess")?;
    if !excess.gt(&d("0"))? {
        return fact_invalid(format!("{item}: excess must be above nil."));
    }
    let determination = fact_date(facts, item, "determination_date")?;
    let start: i64 = period[..4].parse().expect("a supported year");
    let r = release(&excess, start, period, determination)?;
    let mut out = result(
        KIND,
        period,
        vec![
            ("associated_earnings", r.earnings.clone()),
            ("total_release_amount", r.total),
            ("assessable_associated_earnings", r.earnings),
            ("associated_earnings_tax_offset", r.offset),
        ],
        vec![rate("proxy_rate_percent", &r.proxy_percent)],
    )?;
    out["question"] = json!("non_concessional_release");
    out["days"] = json!(r.days);
    out["earnings_authority"] = json!(EARNINGS);
    out["proxy_authority"] = json!(PROXY);
    out["release_authority"] = json!(RELEASE);
    out["assessable_authority"] = json!(ASSESSABLE);
    out["offset_authority"] = json!(OFFSET);
    super::ato_interest_penalties::warn(&mut out, r.warning);
    Ok(out)
}

/// The release's steps, its rates and the ATO-figures warning.
pub(super) type ReleaseSteps = (Vec<Step>, Vec<(String, String)>, String);

/// `contribution-caps`' release steps for its excess non-concessional
/// contributions, with `determination_date`.
pub(super) fn release_steps(
    excess: &Decimal,
    year: &str,
    determination_date: &Arg,
) -> Result<ReleaseSteps> {
    let Some(day) = determination_date.text().and_then(dates::parse) else {
        return fact_invalid("determination_date must be a date YYYY-MM-DD.".into());
    };
    let start: i64 = year[..4].parse().expect("a supported year");
    let r = release(excess, start, year, day)?;
    let days = Decimal::from_int(r.days);
    let formula = format!(
        "floor(excess_non_concessional x ((1 + proxy_rate_percent / 100 / {}) ^ days - 1))",
        year_length(first_day(start))
    );
    let steps = vec![
        step(
            "associated_earnings",
            EARNINGS,
            &formula,
            &[excess, &d(&r.proxy_percent), &days],
            r.earnings.clone(),
        ),
        step(
            "total_release_amount",
            RELEASE,
            "excess_non_concessional + 0.85 x associated_earnings",
            &[excess, &r.earnings],
            r.total.clone(),
        ),
        step(
            "associated_earnings_tax_offset",
            OFFSET,
            "0.15 x associated_earnings",
            &[&r.earnings],
            r.offset.clone(),
        ),
    ];
    Ok((
        steps,
        vec![rate("proxy_rate_percent", &r.proxy_percent)],
        r.warning,
    ))
}

/// The command: one `question` with its facts.
pub fn excess_contributions(
    question: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    facts: &Map<String, Value>,
) -> Result<Value> {
    let period = scope(scope_confirmed, year, KIND)?;
    match question.text() {
        Some("concessional_charge") => concessional_charge(facts, &period),
        Some("non_concessional_release") => non_concessional_release(facts, &period),
        _ => refuse(
            "excess_contributions.question_invalid",
            format!("question must be one of {}.", QUESTIONS.join(", ")),
        ),
    }
}
