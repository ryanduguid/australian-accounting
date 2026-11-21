// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `unused_leave_payment`: unused annual leave, leave loading and long service
//! leave paid on termination. Each part's assessable share and tax offset come
//! from ITAA 1997 Subdivisions 83-A and 83-B (compilation 268, in force from
//! 1 October 2026), its withholding from Schedule 7, 'Tax table for unused leave
//! payments on termination of employment', for payments from 1 July 2026
//! (published 17 June 2026), and its label from Schedule 7's label column and
//! the ATO's STP Phase 2 employer reporting guidelines; all read on
//! 16 November 2026.
//!
//! Schedule 7 withholds 32% from the parts the offset caps at 30%, and works
//! the rest out at marginal rates: its example adds the 5% of pre-16 August
//! 1978 long service leave to the post-17 August 1993 leave and runs the
//! marginal rate calculation once on the sum.

use serde_json::Value;

use super::{
    AdditionalPay, Arg, Result, additional_pay, d, date_arg, derived_result, flag, int, money,
    rate, refuse,
};
use crate::dates;
use crate::decimal::Decimal;
use crate::metadata as meta;

mod steps;

const KIND: &str = "unused_leave_payment";

/// The optional inputs: the parts of each total that accrued before the dates
/// the Act splits leave at, each `None` when not given.
pub struct LeaveSplits<'a> {
    pub annual_leave_before_18_august_1993: Option<&'a Arg>,
    pub leave_loading_before_18_august_1993: Option<&'a Arg>,
    pub long_service_leave_before_16_august_1978: Option<&'a Arg>,
    pub long_service_leave_16_august_1978_to_17_august_1993: Option<&'a Arg>,
}

/// The employee's pay and withholding facts: residency (scale 4 only), whether
/// a working holiday maker, and normal gross earnings, pay period and scale.
pub struct LeavePayee<'a> {
    pub foreign_resident: Option<&'a Arg>,
    pub working_holiday_maker: &'a Arg,
    pub earnings: &'a Arg,
    pub pay_period: &'a Arg,
    pub scale: &'a Arg,
}

/// One part of a total: the input that gives it, or none when the accrual
/// start rules it out or the termination reason makes it irrelevant.
struct Part {
    name: &'static str,
    value: Option<Decimal>,
}

impl Part {
    /// The part as a formula term and its value: the input's name, or 0.
    fn term(&self) -> (&'static str, Decimal) {
        match &self.value {
            Some(v) => (self.name, v.clone()),
            None => ("0", int(0)),
        }
    }
}

/// The part of `total` named `name`: refused when it exceeds the total or
/// accrued before `accrual_start` allows, and required when `needed`.
fn part(
    name: &'static str,
    given: Option<&Arg>,
    total: &Decimal,
    (possible, needed): (bool, bool),
    why: &str,
) -> Result<Part> {
    let value = given.map(money).transpose()?;
    if let Some(v) = &value {
        if !v.is_zero() && !possible {
            return refuse(
                "leave.accrual_split_invalid",
                format!("{name} is leave accrued {why}, but leave_accrual_start is not before it."),
            );
        }
        if v.gt(total)? {
            return refuse(
                "leave.accrual_split_invalid",
                format!("{name} is more than the total it is part of."),
            );
        }
    } else if needed && !total.is_zero() {
        return refuse(
            "leave.accrual_split_missing",
            format!(
                "{name} is required: the leave accrued from leave_accrual_start, so the part accrued {why} is taxed and withheld differently; give 0 if none of it did."
            ),
        );
    }
    Ok(Part { name, value })
}

/// The termination reason: whether it is a genuine redundancy, an early
/// retirement scheme or invalidity (ss 83-15(a) and 83-85(2)(b)).
fn concessional_reason(reason: &Arg) -> Result<bool> {
    match reason.text() {
        Some("normal") => Ok(false),
        Some("genuine_redundancy" | "early_retirement_scheme" | "invalidity") => Ok(true),
        Some("death") => refuse(
            "leave.death",
            "Unused leave paid after an employee's death is not withheld from or shown on their income statement (ATO, 'Payments made after death'), and this command does not work out its tax.",
        ),
        _ => refuse(
            "input.termination_reason_invalid",
            "termination_reason must be normal, genuine_redundancy, early_retirement_scheme, invalidity or death.",
        ),
    }
}

/// Refuse a working holiday maker, whom Schedule 15 covers; then the rate
/// Schedule 7 withholds from the whole payment when no TFN was provided
/// (scale 4): 47%, or 45% for a foreign resident. `None` for a TFN, when its
/// tables apply on the payee's Schedule 1 scale, scale 3 for a foreign resident.
fn payee(
    pay: &AdditionalPay,
    foreign_resident: Option<&Arg>,
    working_holiday_maker: &Arg,
) -> Result<Option<&'static str>> {
    if flag(working_holiday_maker, "working_holiday_maker")? {
        return refuse(
            "leave.working_holiday_maker",
            "Schedule 7 sends every payment to a working holiday maker, unused leave included, to Schedule 15: use working-holiday-maker-withholding.",
        );
    }
    let foreign = super::foreign_resident(foreign_resident, pay.scale)?;
    let key = match foreign {
        None => return Ok(None),
        Some(true) => "no_tfn_foreign_resident_rate",
        Some(false) => "no_tfn_rate",
    };
    Ok(Some(super::lookup(&meta::UNUSED_LEAVE_PAYMENT, key)))
}

/// The totals and their parts, checked against each other and the accrual start.
struct Leave {
    annual: Decimal,
    loading: Decimal,
    long_service: Decimal,
    /// Annual leave and loading before 18 August 1993, long service leave
    /// before 16 August 1978 and from then to 17 August 1993.
    parts: [Part; 4],
}

/// Each part: its name, the total it is part of (annual leave, loading or long
/// service leave) and whether its date is 16 August 1978 rather than 18 August
/// 1993; then the period in words.
const PARTS: [(&str, usize, bool); 4] = [
    ("annual_leave_before_18_august_1993", 0, false),
    ("leave_loading_before_18_august_1993", 1, false),
    ("long_service_leave_before_16_august_1978", 2, true),
    (
        "long_service_leave_16_august_1978_to_17_august_1993",
        2,
        false,
    ),
];
const PERIODS: [&str; 4] = [
    "before 18 August 1993",
    "before 18 August 1993",
    "before 16 August 1978",
    "from 16 August 1978 to 17 August 1993",
];

/// The parts against the totals: the 2 long service leave parts within their
/// total, and Schedule 7's $300 limit, which applies when a TFN is provided to
/// post-17 August 1993 leave on a normal termination (all of each total but
/// its parts) and which it does not say how to apply beside pre-16 August 1978
/// long service leave.
fn totals_rules(
    totals: &[Decimal; 3],
    parts: &[Part; 4],
    (concessional, tfn): (bool, bool),
) -> Result<()> {
    let given: Vec<Decimal> = parts.iter().map(|p| p.term().1).collect();
    if given[2].add(&given[3])?.gt(&totals[2])? {
        return refuse(
            "leave.accrual_split_invalid",
            "long_service_leave_before_16_august_1978 and long_service_leave_16_august_1978_to_17_august_1993 together are more than long_service_leave.",
        );
    }
    let all = totals.iter().try_fold(int(0), |sum, t| sum.add(t))?;
    let post_1993 = given.iter().try_fold(all, |rest, p| rest.sub(p))?;
    let small = d(super::lookup(&meta::UNUSED_LEAVE_PAYMENT, "small_payment"));
    let under_300 = tfn && !concessional && !post_1993.is_zero() && post_1993.lt(&small)?;
    if under_300 && !given[2].is_zero() {
        return refuse(
            "leave.under_300_with_pre_1978",
            "Schedule 7 limits withholding from a post-17 August 1993 payment under $300 to 32% of it, but does not say how when pre-16 August 1978 long service leave joins it in the marginal rate calculation.",
        );
    }
    Ok(())
}

fn leave(
    totals: [&Arg; 3],
    splits: &LeaveSplits,
    accrual_start: i64,
    (concessional, tfn): (bool, bool),
) -> Result<Leave> {
    let [annual, loading, long_service] = totals.map(money);
    let totals = [annual?, loading?, long_service?];
    let before = |day| accrual_start < dates::parse(day).expect("date");
    let (before_1993, before_1978) = (before("1993-08-18"), before("1978-08-16"));
    let given = [
        splits.annual_leave_before_18_august_1993,
        splits.leave_loading_before_18_august_1993,
        splits.long_service_leave_before_16_august_1978,
        splits.long_service_leave_16_august_1978_to_17_august_1993,
    ];
    // A normal termination taxes leave accrued before 18 August 1993
    // differently (ss 83-15(b) and 83-85(2)); pre-16 August 1978 long service
    // leave is 5% assessable whatever the reason (s 83-80(1)).
    let [a, b, c, e] = std::array::from_fn(|i| {
        let (name, total, from_1978) = PARTS[i];
        let when = if from_1978 {
            (before_1978, before_1978)
        } else {
            (before_1993, before_1993 && !concessional)
        };
        part(name, given[i], &totals[total], when, PERIODS[i])
    });
    let parts = [a?, b?, c?, e?];
    totals_rules(&totals, &parts, (concessional, tfn))?;
    let [annual, loading, long_service] = totals;
    Ok(Leave {
        annual,
        loading,
        long_service,
        parts,
    })
}

/// Schedule 7 withholding, assessable shares and labels for one employee's
/// unused leave paid on termination.
pub fn unused_leave_payment(
    totals: [&Arg; 3],
    splits: &LeaveSplits,
    (leave_accrual_start, termination_reason): (&Arg, &Arg),
    employee: &LeavePayee,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let no_loan = Arg::Bool(false);
    let pay = additional_pay(
        KIND,
        employee.earnings,
        employee.pay_period,
        employee.scale,
        (&no_loan, None),
        year,
        scope_confirmed,
    )?;
    let no_tfn_rate = payee(
        &pay,
        employee.foreign_resident,
        employee.working_holiday_maker,
    )?;
    let concessional = concessional_reason(termination_reason)?;
    let start = date_arg(leave_accrual_start, "leave_accrual_start")?;
    let leave = leave(totals, splits, start, (concessional, no_tfn_rate.is_none()))?;
    let steps = steps::leave_steps(&pay, &leave, concessional, no_tfn_rate)?;
    let figure = |key| super::lookup(&meta::UNUSED_LEAVE_PAYMENT, key);
    let mut rates = vec![rate(
        "pre_1978_assessable_rate",
        figure("pre_1978_assessable_rate"),
    )];
    match no_tfn_rate {
        Some(no_tfn) => rates.push(rate("no_tfn_rate", no_tfn)),
        None => rates.extend([
            rate("flat_rate", figure("flat_rate")),
            rate("small_payment", figure("small_payment")),
            rate("pay_periods_in_year", pay.per_year),
        ]),
    }
    rates.extend([
        rate("scale", pay.scale),
        rate("lump_sum_a_type", if concessional { "R" } else { "T" }),
    ]);
    derived_result(KIND, &pay.year, steps, rates)
}
