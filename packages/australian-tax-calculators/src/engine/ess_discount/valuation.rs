// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The value of an unlisted right under the Income Tax Assessment (1997 Act)
//! Regulations 2021 ss 83A-315.02 to 83A-315.09, compilation 16 (27 June 2026),
//! read on 22 November 2026. The tables below are transcribed from it and agree
//! entry for entry with the ATO legal database's copy read the same day.

use serde_json::{Value, json};

use super::shown;
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{Result, int, precise, py_max, refuse};

/// The upper bound, in months, of each exercise period row, shortest first.
const PERIODS: [i64; 19] = [
    3, 6, 9, 12, 18, 24, 36, 48, 60, 72, 84, 96, 108, 120, 132, 144, 156, 168, 180,
];

/// The upper bound of each calculation percentage column of s 83A-315.08.
const COLUMNS: [&str; 14] = [
    "60", "70", "75", "80", "85", "90", "92.5", "95", "97.5", "100", "102.5", "105", "107.5", "110",
];

/// s 83A-315.08(1), items 19 down to 1 (columns 50 to 92.5) joined with items
/// 38 down to 20 (columns 92.5 to 110), each row in the order of [`PERIODS`].
const TABLE: &str = "0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.1 0.5 1.4 3.0 5.1 7.5
0.0 0.0 0.0 0.0 0.0 0.0 0.1 0.3 0.6 1.3 2.4 3.8 5.7 7.8
0.0 0.0 0.0 0.0 0.0 0.1 0.4 0.8 1.4 2.2 3.3 4.7 6.4 8.3
0.0 0.0 0.0 0.0 0.1 0.2 0.8 1.3 2.0 2.9 4.0 5.4 7.0 8.8
0.0 0.0 0.0 0.0 0.1 0.4 1.1 1.7 2.5 3.4 4.6 6.0 7.5 9.2
0.0 0.0 0.0 0.1 0.3 0.9 1.8 2.5 3.4 4.4 5.5 6.8 8.3 9.9
0.0 0.0 0.1 0.3 0.6 1.3 2.4 3.2 4.1 5.1 6.3 7.6 9.0 10.5
0.0 0.0 0.3 0.6 1.2 2.1 3.4 4.2 5.2 6.2 7.4 8.6 10.0 11.4
0.0 0.1 0.5 1.0 1.7 2.8 4.2 5.1 6.0 7.0 8.2 9.4 10.7 12.1
0.0 0.2 0.8 1.4 2.2 3.4 4.9 5.7 6.7 7.7 8.8 9.9 11.2 12.5
0.1 0.3 1.1 1.7 2.7 3.9 5.4 6.2 7.2 8.2 9.2 10.4 11.6 12.8
0.1 0.4 1.3 2.1 3.0 4.3 5.8 6.6 7.6 8.5 9.6 10.7 11.8 13.0
0.1 0.6 1.6 2.4 3.4 4.6 6.1 7.0 7.9 8.8 9.8 10.9 12.0 13.2
0.2 0.7 1.8 2.6 3.7 4.9 6.4 7.2 8.1 9.1 10.0 11.1 12.1 13.3
0.2 0.8 2.0 2.8 3.9 5.2 6.6 7.5 8.3 9.2 10.2 11.2 12.2 13.3
0.3 0.9 2.2 3.0 4.1 5.4 6.8 7.6 8.4 9.3 10.2 11.2 12.2 13.3
0.4 1.0 2.3 3.2 4.3 5.5 7.0 7.7 8.5 9.4 10.3 11.2 12.2 13.3
0.4 1.2 2.5 3.4 4.4 5.7 7.1 7.8 8.6 9.4 10.3 11.2 12.2 13.3
0.5 1.3 2.6 3.5 4.6 5.8 7.1 7.9 8.6 9.4 10.3 11.2 12.2 13.3";

/// s 83A-315.09(2): the base and additional percentages, in the order of
/// [`PERIODS`] (items 19 down to 1).
const BASE: [(&str, &str); 19] = [
    ("7.5", "1.0"),
    ("7.8", "0.9"),
    ("8.3", "0.9"),
    ("8.8", "0.9"),
    ("9.2", "0.9"),
    ("9.9", "0.8"),
    ("10.5", "0.8"),
    ("11.4", "0.8"),
    ("12.1", "0.7"),
    ("12.5", "0.7"),
    ("12.8", "0.7"),
    ("13.0", "0.6"),
    ("13.2", "0.6"),
    ("13.3", "0.6"),
    ("13.3", "0.6"),
    ("13.3", "0.6"),
    ("13.3", "0.5"),
    ("13.3", "0.5"),
    ("13.3", "0.5"),
];

/// The first day the 2021 regulations value a right on: they apply to income
/// years starting at or after 1 April 2021 (s 1000-1.02).
pub(super) const FIRST_DAY: &str = "2021-07-01";

/// The row for the period from `day` to `last`: the first whose bound, as the
/// same calendar day that many months on, is not before `last`. A period on a
/// bound takes the lower row (ss 83A-315.08(2), 83A-315.09(3)).
fn row(day: i64, last: i64) -> Option<usize> {
    PERIODS
        .iter()
        .position(|months| last <= dates::add_months(day, *months))
}

/// The s 83A-315.08 entry in a row of [`TABLE`] and a column of [`COLUMNS`].
fn table_entry(row: usize, column: usize) -> &'static str {
    TABLE
        .lines()
        .nth(row)
        .and_then(|line| line.split_whitespace().nth(column))
        .expect("table entry")
}

fn percent(text: &str) -> Result<Decimal> {
    Ok(Decimal::parse(text)?.div(&int(100))?)
}

/// A right's value on `day` under the regulations, totals for the parcel:
/// `share` is the market value of the shares the rights may acquire and
/// `exercise` the lowest total that must be paid to exercise them. The value
/// and its working, amounts to the cent.
pub(super) fn right_value(
    share: &Decimal,
    exercise: &Decimal,
    day: i64,
    last: i64,
    acquired: i64,
) -> Result<(Decimal, Value)> {
    if day < dates::parse(FIRST_DAY).expect("date") {
        return refuse(
            "ess_discount.valuation_unavailable",
            format!(
                "The 2021 regulations' tables value a right only on a day from {FIRST_DAY} (s 1000-1.02); the Income Tax Assessment Regulations 1997 governed {} and are not applied: give the right's market value.",
                dates::format(day)
            ),
        );
    }
    // s 83A-315.01(1): a right that must be exercised within 15 years after
    // the day it was acquired; then no period from `day` passes 180 months.
    let within = last >= day && last <= dates::add_months(acquired, 180);
    let Some(row) = within.then(|| row(day, last)).flatten() else {
        return refuse(
            "ess_discount.valuation_unavailable",
            "The regulations value a right whose last exercise day is on or after the valuation day and within 15 years after acquisition (s 83A-315.01(1)): give the right's market value.",
        );
    };
    if exercise.is_zero() {
        let detail = json!({"rule": "s 83A-315.03", "value": shown(share)?});
        return Ok((share.clone(), detail));
    }
    precise(|| table_value(share, exercise, row))
}

/// The table's percentage of the exercise price for a calculation percentage
/// and exercise period row, and the section that gives it.
fn table_rate(calculation: &Decimal, row: usize) -> Result<(&'static str, Decimal)> {
    if calculation.lt(&int(50))? {
        return Ok(("s 83A-315.05 step 2", int(0)));
    }
    if calculation.lt(&int(110))? {
        let column = COLUMNS
            .iter()
            .position(|top| {
                calculation
                    .le(&Decimal::parse(top).expect("column"))
                    .unwrap_or(false)
            })
            .expect("below 110");
        return Ok(("s 83A-315.08", percent(table_entry(row, column))?));
    }
    let excess = calculation.sub(&int(110))?.quantize(0, Rounding::Down)?;
    let (base, additional) = BASE[row];
    let rate = percent(base)?.add(&excess.mul(&percent(additional)?)?)?;
    Ok(("s 83A-315.09", rate))
}

/// The greater of the intrinsic value and the table value (s 83A-315.02(1)).
fn table_value(share: &Decimal, exercise: &Decimal, row: usize) -> Result<(Decimal, Value)> {
    let calculation = share.mul(&int(100))?.div(exercise)?;
    let (rule, rate) = table_rate(&calculation, row)?;
    let tabled = exercise.mul(&rate)?;
    let intrinsic = share.sub(exercise)?;
    let value = py_max(intrinsic.clone(), tabled.clone())?;
    let detail = json!({
        "rule": rule,
        "calculation_percentage": calculation.to_string(),
        "exercise_period_months_up_to": PERIODS[row],
        "intrinsic_value": shown(&intrinsic)?,
        "table_value": shown(&tabled)?,
        "value": shown(&value)?,
    });
    Ok((value, detail))
}
