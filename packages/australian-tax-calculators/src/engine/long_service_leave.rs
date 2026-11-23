// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `long_service_leave`: one employee's long service leave under the general
//! Act of one state or territory, as weeks to take or pay and their value at
//! the weekly pay, and the hours `leave-provisions` takes. It mirrors the long
//! service leave calculator on ryanduguid.github.io, except that Queensland
//! uses the 8.6667 weeks the Industrial Relations Act 2016 s 95 states and the
//! dates are limited to the text of each Act read on 24 November 2026;
//! docs/calculation-evidence.md#long-service-leave lists the versions read.

use serde_json::Value;

use super::trade_in::when;
use super::{Arg, Result, d, date_arg, money, precise, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::Decimal;

mod acts;
use acts::{Outcome, Place, Weeks};

const KIND: &str = "long_service_leave";

/// The facts as the command passes them; see the README for each.
pub struct LongServiceLeaveArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub start: &'a Arg,
    pub end: &'a Arg,
    pub excluded_days: &'a Arg,
    pub leave_taken: &'a Arg,
    pub status: &'a Arg,
    pub reason: Option<&'a Arg>,
    pub weekly_pay: &'a Arg,
    pub hours_per_week: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const EARLIEST_START: &str = "1990-06-23";
const MOST_PAY_CENTS: i128 = 100_000_000;

/// `value` in units of 10^-`places`, for a decimal from 0 to `most` with at
/// most `places` decimal places.
fn scaled(value: &Arg, places: i64, most: i64) -> Option<i128> {
    let Arg::Decimal(v) = value else {
        return None;
    };
    let fits = v.is_finite() && v.exponent() >= -i128::from(places);
    let within =
        fits && !v.lt(&Decimal::from_int(0)).ok()? && !v.gt(&Decimal::from_int(most)).ok()?;
    let places = u32::try_from(places).ok()?;
    within.then(|| v.fixed(places, false).replace('.', "").parse().ok())?
}

/// The ending, if employment ends: the reason, asked for only then.
fn ending(status: &Arg, reason: Option<&Arg>) -> Result<Option<acts::Reason>> {
    let ends = match status.text() {
        Some("ends") => true,
        Some("employed") => false,
        _ => {
            return refuse(
                "long_service_leave.status_invalid",
                "status must be employed or ends.",
            );
        }
    };
    match when(ends, reason, "reason", "when status is ends")? {
        Some(reason) => Ok(Some(acts::reason(reason)?)),
        None => Ok(None),
    }
}

/// The financial year holding `day`, as `2025-26`.
fn financial_year(day: i64) -> String {
    let (y, m, _) = dates::to_civil(day);
    let first = if m >= 7 { y } else { y - 1 };
    format!("{first}-{:02}", (first + 1) % 100)
}

/// The first and last days, checked against the year and the Act's text.
fn service_dates(args: &LongServiceLeaveArgs, place: Place, year: &str) -> Result<(i64, i64)> {
    let start = date_arg(args.start, "start")?;
    let end = date_arg(args.end, "end")?;
    let on = |text: &str| dates::parse(text).expect("a literal date");
    if start < on(EARLIEST_START) {
        return refuse(
            "long_service_leave.start_too_early",
            "start is before 23 June 1990: Queensland counts earlier service under the repealed 1961 Act, and the other Acts' treatment of service that old was not read.",
        );
    }
    if end < start {
        return refuse(
            "long_service_leave.end_before_start",
            "end is before start.",
        );
    }
    if financial_year(end) != year {
        return refuse(
            "long_service_leave.year_mismatch",
            format!(
                "end falls in {}, not the year given, {year}.",
                financial_year(end)
            ),
        );
    }
    if end < on(place.from()) {
        return refuse(
            "long_service_leave.before_current_text",
            format!(
                "end is before {}, when the Act's text this worksheet read began: the earlier text was not read.",
                place.from()
            ),
        );
    }
    Ok((start, end))
}

/// The day after the last counted day: the end, less the excluded days.
fn until(excluded: &Arg, start: i64, end: i64) -> Result<i64> {
    match excluded.int() {
        Some(Some(n)) if (0..=36_500).contains(&n) && end + 1 - n > start => Ok(end + 1 - n),
        _ => refuse(
            "long_service_leave.excluded_invalid",
            "excluded_days must be a whole number from 0 to 36,500 and fewer than the days from start to end.",
        ),
    }
}

/// The facts in fixed point: leave taken in ten-thousandths of a week, pay in
/// cents and hours a week in hundredths.
fn amounts_given(args: &LongServiceLeaveArgs) -> Result<(i128, i128, Option<i128>)> {
    let Some(taken) = scaled(args.leave_taken, 4, 1000) else {
        return refuse(
            "long_service_leave.taken_invalid",
            "leave_taken must be weeks from 0 to 1,000, to at most 4 decimal places.",
        );
    };
    let pay = money(args.weekly_pay)?;
    let pay: i128 = pay
        .fixed(2, false)
        .replace('.', "")
        .parse()
        .expect("money's digits");
    if pay > MOST_PAY_CENTS {
        return refuse(
            "long_service_leave.pay_invalid",
            "weekly_pay must be no more than $1,000,000.",
        );
    }
    let hours = match args.hours_per_week {
        None => None,
        Some(given) => match scaled(given, 2, 168) {
            Some(h) if h > 0 => Some(h),
            _ => {
                return refuse(
                    "long_service_leave.hours_invalid",
                    "hours_per_week must be above 0 to 168, to at most 2 decimal places.",
                );
            }
        },
    };
    Ok((taken, pay, hours))
}

/// `numerator / denominator` half up, for a fraction that is not negative.
fn half(numerator: i128, denominator: i128) -> i128 {
    (2 * numerator + denominator) / (2 * denominator)
}

/// Ten-thousandths as a 4-place decimal.
fn four(value: i128) -> String {
    format!("{}.{:04}", value / 10_000, value % 10_000)
}

/// Hundredths as a 2-place decimal.
fn two(value: i128) -> String {
    format!("{}.{:02}", value / 100, value % 100)
}

/// The weeks left once leave taken is deducted, refused when it is more
/// than the exact weeks.
fn left((n, by): Weeks, taken: i128) -> Result<i128> {
    let left = n * 10_000 - taken * by;
    if left < 0 {
        return refuse(
            "long_service_leave.taken_above_weeks",
            "leave_taken is more than this entitlement: the Acts treat leave taken in advance differently, and this worksheet does not cover it.",
        );
    }
    Ok(left)
}

pub fn long_service_leave(args: &LongServiceLeaveArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = acts::place(args.jurisdiction)?;
    let reason = ending(args.status, args.reason)?;
    let (start, end) = service_dates(args, place, &year)?;
    let until = until(args.excluded_days, start, end)?;
    let (taken, pay, hours) = amounts_given(args)?;
    acts::transition(place, start, taken)?;
    let service = acts::measure(start, until);
    let outcome = match reason {
        Some(reason) => acts::on_ending(place, &service, reason, start, until)?,
        None => acts::available(place, &service)?,
    };
    let left = left(outcome.weeks, taken)?;
    let rates = rates(&service, &outcome, taken, hours.map(|h| (left, h)));
    let per = outcome.weeks.1 * 10_000;
    let value = d(&two(half(left * pay, per)));
    precise(|| result(KIND, &year, vec![("value", value)], rates))
}

/// The service, the weeks, the balance and the hours for `leave-provisions`.
fn rates(
    s: &acts::Service,
    outcome: &Outcome,
    taken: i128,
    hours: Option<(i128, i128)>,
) -> Vec<(String, String)> {
    let (n, by) = outcome.weeks;
    let weeks = half(n * 10_000, by);
    let mut out = vec![
        rate("years", s.years),
        rate("months", s.months),
        rate("days", s.days),
        rate("days_counted", s.total),
        rate("weeks", four(weeks)),
        rate("leave_taken", four(taken)),
        rate("balance", four(weeks - taken)),
        rate("provision", &outcome.provision),
    ];
    if let Some((left, per_week)) = hours {
        out.push(rate("hours", two(half(left * per_week, by * 10_000))));
    }
    out
}
