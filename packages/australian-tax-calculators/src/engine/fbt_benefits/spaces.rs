// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The car parking worksheet's spaces method (FBTAA Division 10A, Subdivision
//! C): the total taxable value of the car parking fringe benefits of the
//! employees an election covers, from the spaces that gave rise to them. It
//! cites the FBTAA as compiled on 1 July 2026 (C2004A03280, compilation 97) and
//! 'Fringe benefits tax - a guide for employers', 16.2.8 and example 18, read
//! on 16 November 2026.

use serde_json::Value;

use super::super::{Arg, Result, Step, cents, int, money, precise, refuse, step};
use super::{PARKING, Parking, take, with_payments};
use crate::decimal::Decimal;

/// Spaces sharing a daily rate amount and an availability period, and the
/// lowest all-day fee nearby on the first business day.
struct Group {
    fee: Decimal,
    rate: Decimal,
    days: i64,
    spaces: i64,
}

/// The fields of each group in `space_groups`.
const GROUP_FIELDS: [&str; 4] = [
    "first_business_day_fee",
    "daily_rate_amount",
    "availability_days",
    "spaces",
];

/// The most spaces, employees or groups the worksheet takes.
const MOST: i64 = 1_000_000;

/// The election's head counts for s 39FB, (input, step, authority): the
/// employees, then the eligible spaces, at the start and end of the parking
/// period, averaged.
const AVERAGES: [(&str, &str, &str); 2] = [
    ("employees", "average_employees", "FBTAA s 39FB(3)"),
    (
        "eligible_spaces",
        "average_eligible_spaces",
        "FBTAA s 39FB(4)",
    ),
];

/// s 39FA(4) with s 39FB: for each space, its daily rate amount (s 39FC) times
/// its availability period's days (s 39FD) over 366, times 228; their total
/// (the total statutory benefit), reduced by s 39FB where the election's
/// employees average fewer than its eligible spaces; less the relevant
/// recipients' contributions (s 39FE), never below nil.
pub(super) fn spaces(
    given: &[(String, Arg)],
    when: &str,
    threshold: &Decimal,
    days_in_year: i64,
) -> Result<Parking> {
    let names = [
        "space_groups",
        "employees_at_start",
        "employees_at_end",
        "eligible_spaces_at_start",
        "eligible_spaces_at_end",
        "employee_contribution",
    ];
    let [groups, e0, e1, s0, s1, contribution] = take(given, PARKING, when, names)?;
    let groups = space_groups(groups, threshold, days_in_year)?;
    let count = |value: &Arg| match value.int() {
        Some(Some(n)) if (0..=MOST).contains(&n) => Ok(int(n)),
        _ => refuse(
            "fbt_car_parking.head_counts_invalid",
            "employees_at_start, employees_at_end, eligible_spaces_at_start and eligible_spaces_at_end must each be a JSON integer from 0 to 1000000 (FBTAA s 39FB(3) and (4)).",
        ),
    };
    let counts = [[count(e0)?, count(e1)?], [count(s0)?, count(s1)?]];
    let contribution = money(contribution)?;
    let mut steps = vec![statutory_benefit(&groups)?];
    steps.extend(averages(&counts)?);
    steps.push(gross_value(&steps)?);
    let authority = "FBTAA s 39FA(4), step 3, and s 39FE";
    let (steps, taxable) = with_payments(steps, &contribution, authority)?;
    // Every group's fee is above the threshold, so a benefit arises.
    let mut fee = int(0);
    for group in &groups {
        if group.fee.gt(&fee)? {
            fee = group.fee.clone();
        }
    }
    Ok((fee, steps, taxable))
}

fn groups_invalid<T>() -> Result<T> {
    refuse(
        "fbt_car_parking.space_groups_invalid",
        "space_groups must be a list of 1 to 1000000 objects, each with exactly first_business_day_fee and daily_rate_amount as decimal strings, availability_days as a JSON integer from 1 to the days in the FBT year and spaces as a JSON integer from 1 to 1000000.",
    )
}

/// `space_groups`: each group's fields, refusing one whose first business day
/// fee gives no car parking benefit.
fn space_groups(value: &Arg, threshold: &Decimal, days_in_year: i64) -> Result<Vec<Group>> {
    let Arg::Json(Value::Array(items)) = value else {
        return groups_invalid();
    };
    if items.is_empty() || items.len() as i64 > MOST {
        return groups_invalid();
    }
    items
        .iter()
        .map(|item| {
            let fields = match item {
                Value::Object(f) if f.len() == 4 && GROUP_FIELDS.iter().all(|k| f.contains_key(*k)) => f,
                _ => return groups_invalid(),
            };
            let amount = |key: &str| match fields[key].as_str().map(Decimal::parse_input) {
                Some(Ok(v)) => money(&Arg::Decimal(v)),
                _ => groups_invalid(),
            };
            let whole = |key: &str, most: i64| match fields[key].as_i64() {
                Some(n) if (1..=most).contains(&n) => Ok(n),
                _ => groups_invalid(),
            };
            let rate = amount("daily_rate_amount")?;
            let group = Group {
                fee: amount("first_business_day_fee")?,
                // Written as a sum adds it, to 0, so that the working reads
                // 50 for 5E+1 and 0.00 for -0.00.
                rate: precise(|| Ok(int(0).add(&rate)?))?,
                days: whole("availability_days", days_in_year)?,
                spaces: whole("spaces", MOST)?,
            };
            if !group.fee.gt(threshold)? {
                return refuse(
                    "fbt_car_parking.spaces_without_benefit",
                    format!(
                        "A space group's first_business_day_fee, {}, is not more than the car parking threshold, {threshold}: its spaces gave rise to no car parking benefit (FBTAA s 39A(1)(a)(iii)), so the spaces method does not count them (s 39FA(4), step 1). Leave the group out.",
                        group.fee
                    ),
                );
            }
            Ok(group)
        })
        .collect()
}

/// s 39FA(4) steps 1 and 2: each group's daily rate amount times its days times
/// 228 times its spaces, added together and divided by 366 once, rounded to
/// cents. The groups are a list, so their figures are written into the working.
fn statutory_benefit(groups: &[Group]) -> Result<Step> {
    let terms: Vec<String> = groups
        .iter()
        .map(|g| format!("{} x {} x 228 x {}", g.rate, g.days, g.spaces))
        .collect();
    let total = precise(|| {
        let mut sum = int(0);
        for g in groups {
            let term = g.rate.mul(&int(g.days))?.mul(&int(228))?;
            sum = sum.add(&term.mul(&int(g.spaces))?)?;
        }
        Ok(cents(sum.div(&int(366))?)?)
    })?;
    Ok(step(
        "total_statutory_benefit",
        "FBTAA s 39FA(4), steps 1 and 2: for each space, daily rate amount (s 39FC) x days in its availability period (s 39FD) / 366 x 228, added together",
        &format!("cents(({}) / 366)", terms.join(" + ")),
        &[],
        total,
    ))
}

/// s 39FB(3) and (4): the average employees and eligible spaces.
fn averages(counts: &[[Decimal; 2]; 2]) -> Result<Vec<Step>> {
    AVERAGES
        .iter()
        .zip(counts)
        .map(|((input, id, authority), [start, end])| {
            let average = precise(|| Ok(start.add(end)?.div(&int(2))?))?;
            let formula = format!("({input}_at_start + {input}_at_end) / 2");
            Ok(step(*id, authority, &formula, &[start, end], average))
        })
        .collect()
}

/// The value before the contributions: the total statutory benefit, times the
/// average employees over the average eligible spaces where the employees are
/// fewer (s 39FB(1) and (2)), rounded to cents.
fn gross_value(steps: &[Step]) -> Result<Step> {
    let [total, employees, spaces] = [0, 1, 2].map(|i| &steps[i].value);
    if !employees.lt(spaces)? {
        return Ok(step(
            "gross_taxable_value",
            "FBTAA s 39FA(4), step 2; s 39FB does not apply, as the average employees are not fewer than the average eligible spaces (s 39FB(1))",
            "total_statutory_benefit",
            &[total],
            total.clone(),
        ));
    }
    let reduced = precise(|| Ok(cents(total.mul(employees)?.div(spaces)?)?))?;
    Ok(step(
        "gross_taxable_value",
        "FBTAA s 39FB(1) and (2)",
        "cents(total_statutory_benefit x average_employees / average_eligible_spaces)",
        &[total, employees, spaces],
        reduced,
    ))
}
