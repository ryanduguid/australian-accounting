// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `notice_period`: the minimum notice of termination an employer must give
//! under the National Employment Standards (Fair Work Act 2009 s 117), with
//! the s 123 exclusions as the operator states them, and the payment in lieu
//! of s 117(2)(b) from a stated full rate of pay (s 18). The Act was read in
//! compilation 73 (7 July 2026) on 24 November 2026; s 117(4), inserted by Act
//! No. 25, 2021 from 27 March 2021, bounds the notices covered.

use serde_json::{Value, json};

use super::trade_in::when;
use super::{Arg, Result, date_arg, flag, int, money, precise, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "notice_period";

/// The facts as the command passes them; see the README for each.
pub struct NoticePeriodArgs<'a> {
    pub notice_date: &'a Arg,
    pub service_start: &'a Arg,
    pub excluded_days: &'a Arg,
    pub date_of_birth: &'a Arg,
    pub exclusion: &'a Arg,
    pub to_avoid_division: Option<&'a Arg>,
    pub full_rate_of_pay: Option<&'a Arg>,
    pub weekly_hours: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The day s 117(4) commenced (Act No. 25, 2021, s 2 table item 2).
const TEXT_FROM: &str = "2021-03-27";

const NOTICE: &str = "Fair Work Act 2009 s 117(2) and (3)";
const AVOIDED: &str = "Fair Work Act 2009 s 117(2) and (3); s 123(2): s 123(1)(a) does not exclude an employee employed for a specified period, task or season where a substantial reason for that was to avoid the Division";

/// Each exclusion the operator may state, with the provision that removes
/// the notice; `none` and `specified_period` are handled apart.
const EXCLUSIONS: [(&str, &str); 8] = [
    (
        "serious_misconduct",
        "Fair Work Act 2009 s 123(1)(b): the Division does not apply to an employee whose employment is terminated because of serious misconduct",
    ),
    (
        "casual",
        "Fair Work Act 2009 s 123(1)(c): the Division does not apply to a casual employee",
    ),
    (
        "training_arrangement",
        "Fair Work Act 2009 s 123(1)(d): the Division does not apply to an employee (other than an apprentice) under a training arrangement whose employment is for a specified period or limited to the arrangement's duration",
    ),
    (
        "prescribed_division",
        "Fair Work Act 2009 s 123(1)(e): the Division does not apply to an employee the regulations prescribe",
    ),
    (
        "daily_hire_building",
        "Fair Work Act 2009 s 123(3)(b): Subdivision A does not apply to a daily hire employee working in the building and construction industry",
    ),
    (
        "daily_hire_meat",
        "Fair Work Act 2009 s 123(3)(c): Subdivision A does not apply to a daily hire employee working in the meat industry in connection with the slaughter of livestock",
    ),
    (
        "weekly_hire_meat_seasonal",
        "Fair Work Act 2009 s 123(3)(d): Subdivision A does not apply to a weekly hire employee working in connection with the meat industry whose termination is determined solely by seasonal factors",
    ),
    (
        "prescribed_notice",
        "Fair Work Act 2009 s 123(3)(e): Subdivision A does not apply to an employee the regulations prescribe",
    ),
];

const SPECIFIED: &str = "Fair Work Act 2009 s 123(1)(a): the Division does not apply to an employee employed for a specified period of time, for a specified task or for the duration of a specified season";

/// The financial year holding `day`, as `2025-26`.
fn financial_year(day: i64) -> String {
    let (y, m, _) = dates::to_civil(day);
    let first = if m >= 7 { y } else { y - 1 };
    format!("{first}-{:02}", (first + 1) % 100)
}

/// The provision excluding the notice, or `None` when s 117 applies; with
/// it, the provision the notice rests on.
fn excluded(args: &NoticePeriodArgs) -> Result<(Option<&'static str>, &'static str)> {
    let name = args.exclusion.text().unwrap_or_default();
    let specified = name == "specified_period";
    let avoid = when(
        specified,
        args.to_avoid_division,
        "to_avoid_division",
        "when exclusion is specified_period",
    )?;
    if let Some(avoid) = avoid {
        return Ok(if flag(avoid, "to_avoid_division")? {
            (None, AVOIDED)
        } else {
            (Some(SPECIFIED), SPECIFIED)
        });
    }
    if name == "none" {
        return Ok((None, NOTICE));
    }
    match EXCLUSIONS.iter().find(|(n, _)| *n == name) {
        Some((_, provision)) => Ok((Some(provision), provision)),
        None => refuse(
            "notice_period.exclusion_invalid",
            "exclusion must be none, specified_period or one of the other s 123 exclusions the worksheet lists.",
        ),
    }
}

/// The notice day, the first day of service and the date of birth, checked
/// against the year, the Act's text and each other.
fn checked_dates(args: &NoticePeriodArgs, year: &str) -> Result<(i64, i64, i64)> {
    let notice = date_arg(args.notice_date, "notice_date")?;
    let start = date_arg(args.service_start, "service_start")?;
    let born = date_arg(args.date_of_birth, "date_of_birth")?;
    if financial_year(notice) != year {
        return refuse(
            "notice_period.year_mismatch",
            format!(
                "notice_date falls in {}, not the year given, {year}.",
                financial_year(notice)
            ),
        );
    }
    if notice < dates::parse(TEXT_FROM).expect("a literal date") {
        return refuse(
            "notice_period.before_current_text",
            "notice_date is before 27 March 2021, when s 117(4) took casual periods out of continuous service (Act No. 25, 2021): the earlier text was not read.",
        );
    }
    if start > notice {
        return refuse(
            "notice_period.start_after_notice",
            "service_start is after notice_date.",
        );
    }
    if born >= start {
        return refuse(
            "notice_period.birth_after_start",
            "date_of_birth must be before service_start.",
        );
    }
    Ok((notice, start, born))
}

/// The last day counted as service: the notice day less the excluded days.
fn last_counted(excluded: &Arg, start: i64, notice: i64) -> Result<i64> {
    match excluded.int() {
        Some(Some(n)) if (0..=36_500).contains(&n) && notice - n >= start => Ok(notice - n),
        _ => refuse(
            "notice_period.excluded_invalid",
            "excluded_days must be a whole number from 0 to 36,500 and fewer than the days from service_start to notice_date.",
        ),
    }
}

/// Whether service from `start` to the end of `last` runs past `years`
/// years: past the last day of that period (Acts Interpretation Act 1901
/// s 2G(2)).
fn more_than(start: i64, last: i64, years: i64) -> bool {
    last > dates::months_end(start, 12 * years)
}

/// s 117(3)(a): the weeks for the continuous service.
fn base_weeks(start: i64, last: i64) -> i64 {
    1 + [1, 3, 5]
        .iter()
        .filter(|&&n| more_than(start, last, n))
        .count() as i64
}

/// Whether the employee is over 45 at the end of `notice`. An employee aged
/// 45 in completed years is refused: 'over 45 years old' reads as older than
/// 45 years or as aged above 45, and the Act does not say which. A 29
/// February birthday is taken at its earliest (28 February) for the 45th and
/// its latest (1 March) for the 46th.
fn over_45(born: i64, notice: i64) -> Result<bool> {
    if notice < dates::add_months(born, 45 * 12) {
        return Ok(false);
    }
    if notice > dates::months_end(born, 46 * 12) {
        return Ok(true);
    }
    refuse(
        "notice_period.age_unsettled",
        "The employee is 45 in completed years at the end of the notice day: s 117(3)(b)'s 'over 45 years old' may mean older than 45 years or aged above 45, and the worksheet does not choose.",
    )
}

/// The payment in lieu facts: the full rate of pay and the hours a week,
/// both or neither.
/// Whether `hours` is above 0 and no more than the 168 hours of a week.
fn within_week(hours: &Decimal) -> Result<bool> {
    Ok(hours.gt(&int(0))? && !hours.gt(&int(168))?)
}

fn pay_given(args: &NoticePeriodArgs) -> Result<Option<(Decimal, Decimal)>> {
    let hours = when(
        args.full_rate_of_pay.is_some(),
        args.weekly_hours,
        "weekly_hours",
        "with full_rate_of_pay",
    )?;
    let (Some(rate_of_pay), Some(hours)) = (args.full_rate_of_pay, hours) else {
        return Ok(None);
    };
    let rate_of_pay = money(rate_of_pay)?;
    match hours {
        Arg::Decimal(h) if h.is_finite() && h.exponent() >= -2 && within_week(h)? => {
            Ok(Some((rate_of_pay, h.clone())))
        }
        _ => refuse(
            "notice_period.hours_invalid",
            "weekly_hours must be above 0 to 168, to at most 2 decimal places.",
        ),
    }
}

pub fn notice_period(args: &NoticePeriodArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let (notice, start, born) = checked_dates(args, &year)?;
    let last = last_counted(args.excluded_days, start, notice)?;
    let (exclusion, provision) = excluded(args)?;
    let pay = pay_given(args)?;
    let over = match exclusion {
        Some(_) => None,
        None => Some(over_45(born, notice)?),
    };
    let base = base_weeks(start, last);
    let extra = i64::from(over == Some(true) && last >= dates::months_end(start, 24));
    let weeks = if exclusion.is_some() { 0 } else { base + extra };
    let mut amounts = Vec::new();
    let mut rates = vec![
        rate("days_counted", last - start + 1),
        rate("base_weeks", base),
        rate("extra_weeks", extra),
        rate("minimum_notice_weeks", weeks),
        rate(
            "earliest_termination_day",
            dates::format(notice + 7 * weeks),
        ),
    ];
    if let Some((rate_of_pay, hours)) = &pay {
        let payment = precise(|| Ok(int(weeks).mul(hours)?.mul(rate_of_pay)?))?;
        amounts.push(("payment_in_lieu", payment));
        rates.push(rate("full_rate_of_pay", rate_of_pay));
        rates.push(rate("weekly_hours", hours));
    }
    let mut out = precise(|| result(KIND, &year, amounts, rates))?;
    out["notice_required"] = json!(exclusion.is_none());
    out["over_45"] = json!(over);
    out["provision"] = json!(provision);
    Ok(out)
}
