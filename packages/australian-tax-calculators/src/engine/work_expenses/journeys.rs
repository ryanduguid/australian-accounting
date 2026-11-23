// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A car log book given as its period and journeys (ITAA 1997 ss 28-120 and
//! 28-125), and the records Division 28 makes a condition of the log book
//! method, for the operator to confirm.

use serde_json::{Map, Value, json};

use super::car::TOO_SHORT;
use super::rows::{amount, date, fields, list, year_days};
use super::{Arg, Decimal, Result, Step, conflict, int, invalid, refuse, step};
use crate::dates;

const PERIOD_KEYS: [&str; 5] = [
    "period_start",
    "period_end",
    "odometer_start",
    "odometer_end",
    "journeys",
];
const JOURNEY_KEYS: [&str; 5] = ["start", "end", "odometer_start", "odometer_end", "purpose"];

const LOG_TOTAL: &str = "ITAA 1997 s 28-125(4)(b) and (c): the total kilometres the car travelled in the log book period, from its odometer readings at the period's start and end";
const LOG_BUSINESS: &str = "ITAA 1997 s 28-125(2) and (4)(d): the kilometres travelled on the journeys recorded in the log book, each from its odometer readings at the journey's start and end";
const LOG_PERCENTAGE: &str = "ITAA 1997 s 28-125(4)(e) and the ATO's 'Logbook method' steps 1 to 3: the recorded journeys' kilometres as a percentage of the period's total";
const ESTIMATE: &str = "ITAA 1997 s 28-90(5)(a): business kilometres for the year, a reasonable estimate taking the log book into account, as the year's total kilometres (s 28-100(3) odometer records) x the log book percentage";

/// Division 28's record conditions for the log book method.
pub(super) const RECORDS: [&str; 7] = [
    "ITAA 1997 s 28-100(1): confirm each car expense is substantiated under Subdivision 900-C.",
    "ITAA 1997 s 28-100(3) and s 28-140: confirm there are odometer records for the period the car was held in the year, with its make, model, registration number and engine capacity.",
    "ITAA 1997 s 28-100(4): confirm the business kilometres estimate and the business use percentage are recorded in writing before the return is lodged.",
    "ITAA 1997 s 28-115(3) and (4): confirm the Commissioner has not directed a log book for this year and no additional car is claimed under the log book method this year; either needs a new log book for this year.",
    "ITAA 1997 s 28-120(1): confirm the car was held throughout the log book period.",
    "ITAA 1997 s 28-120(3): where the log book method is used for 2 or more cars this year, confirm their log book periods are concurrent.",
    "ITAA 1997 s 28-125(2), (4) and (5): confirm each journey was recorded at its end or as soon as possible after, the period's entries at its start and end, and every entry in English.",
];

/// One recorded journey.
struct Journey {
    start: i64,
    end: i64,
    from: Decimal,
    to: Decimal,
}

/// A checked log book: its period's days and its steps.
pub(super) struct LogBook {
    days: i64,
    pub(super) percentage: Decimal,
    journeys: usize,
    steps: Vec<Step>,
}

fn journey(value: &Value, n: usize, period: (i64, i64)) -> Result<Journey> {
    let what = format!("log_book journey {n}");
    let map = fields(value, &what, &JOURNEY_KEYS)?;
    let blank = map["purpose"].as_str().is_none_or(|p| p.trim().is_empty());
    if blank {
        return refuse(
            "work_expenses.log_book_entry_incomplete",
            format!(
                "{what}: a recorded journey states why it was made (ITAA 1997 s 28-125(2)(d))."
            ),
        );
    }
    let (start, end) = (date(map, "start", &what)?, date(map, "end", &what)?);
    let (from, to) = (amount(map, "odometer_start")?, amount(map, "odometer_end")?);
    if start > end || start < period.0 || end > period.1 {
        return conflict(format!(
            "{what}: its days must run forwards inside the log book period."
        ));
    }
    if !to.gt(&from)? {
        return conflict(format!(
            "{what}: odometer_end must be above odometer_start."
        ));
    }
    Ok(Journey {
        start,
        end,
        from,
        to,
    })
}

/// The journeys in odometer order, each inside the period's readings, none
/// overlapping another or recorded on an earlier day than one before it.
fn ordered(
    map: &Map<String, Value>,
    period: (i64, i64),
    reading: (&Decimal, &Decimal),
) -> Result<Vec<Journey>> {
    let mut journeys = list(&map["journeys"], "log_book journeys")?
        .iter()
        .enumerate()
        .map(|(i, v)| journey(v, i + 1, period))
        .collect::<Result<Vec<_>>>()?;
    journeys.sort_by(|a, b| a.from.compare(&b.from).expect("finite amounts"));
    for j in &journeys {
        if j.from.lt(reading.0)? || j.to.gt(reading.1)? {
            return conflict(
                "log_book: each journey's odometer readings must lie inside the period's.",
            );
        }
    }
    for pair in journeys.windows(2) {
        if pair[1].from.lt(&pair[0].to)? || pair[1].start < pair[0].end {
            return conflict(
                "log_book: journeys overlap, or a later odometer reading is dated before an earlier one.",
            );
        }
    }
    Ok(journeys)
}

impl LogBook {
    /// The log book, kept for `kept`, as s 28-120 and s 28-125 require.
    pub(super) fn read(given: &Arg, kept: i64) -> Result<Self> {
        let Arg::Json(value) = given else {
            return invalid("log_book must be an object.");
        };
        let map = fields(value, "log_book", &PERIOD_KEYS)?;
        let period = (
            date(map, "period_start", "log_book")?,
            date(map, "period_end", "log_book")?,
        );
        if period.1 < period.0 {
            return conflict("log_book: period_end cannot be before period_start.");
        }
        let days = period.1 - period.0 + 1;
        if days < 84 {
            return refuse("work_expenses.log_book_too_short", TOO_SHORT);
        }
        let (first, last) = year_days(kept);
        if period.1 < first || period.0 > last {
            return refuse(
                "work_expenses.log_book_outside_year",
                format!(
                    "The log book period {} to {} includes no part of {kept}-{:02}, the year it is kept for (ITAA 1997 s 28-120(2)).",
                    dates::format(period.0),
                    dates::format(period.1),
                    (kept + 1) % 100
                ),
            );
        }
        let (from, to) = (amount(map, "odometer_start")?, amount(map, "odometer_end")?);
        if !to.gt(&from)? {
            return conflict("log_book: odometer_end must be above odometer_start.");
        }
        let journeys = ordered(map, period, (&from, &to))?;
        Self::totals(days, &journeys, &from, &to)
    }

    fn totals(days: i64, journeys: &[Journey], from: &Decimal, to: &Decimal) -> Result<Self> {
        let total = to.sub(from)?;
        let each = journeys
            .iter()
            .map(|j| j.to.sub(&j.from))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let business = each.iter().try_fold(int(0), |sum, k| sum.add(k))?;
        let terms: Vec<String> = each.iter().map(Decimal::to_string).collect();
        let hundred = int(100);
        let percentage = business.div(&total)?.mul(&hundred)?;
        let steps = vec![
            step(
                "log_book_total_kilometres",
                LOG_TOTAL,
                "odometer_end - odometer_start",
                &[to, from],
                total.clone(),
            ),
            step(
                "log_book_business_kilometres",
                LOG_BUSINESS,
                &terms.join(" + "),
                &[],
                business.clone(),
            ),
            step(
                "log_book_percentage",
                LOG_PERCENTAGE,
                "log_book_business_kilometres / log_book_total_kilometres x 100",
                &[&business, &total],
                percentage.clone(),
            ),
        ];
        Ok(Self {
            days,
            percentage,
            journeys: journeys.len(),
            steps,
        })
    }

    /// The log book's steps, then the year's business kilometres estimated
    /// from `total_kilometres`.
    pub(super) fn into_steps(self, year_total: &Decimal) -> Result<Vec<Step>> {
        let hundred = int(100);
        let business = year_total.mul(&self.percentage)?.div(&hundred)?;
        let mut steps = self.steps;
        steps.push(step(
            "business_kilometres",
            ESTIMATE,
            "total_kilometres x log_book_percentage / 100",
            &[year_total, &self.percentage],
            business,
        ));
        Ok(steps)
    }

    /// The period's days, whole weeks and journeys, for the result.
    pub(super) fn facts(&self) -> [(&'static str, Value); 3] {
        [
            ("log_book_days", json!(self.days)),
            ("log_book_weeks", json!(self.days / 7)),
            ("log_book_journeys", json!(self.journeys)),
        ]
    }
}
