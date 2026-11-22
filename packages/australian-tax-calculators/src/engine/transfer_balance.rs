// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `transfer_balance`: one person's transfer balance account (ITAA 1997
//! Subdivs 294-B and 294-C) from its credit and debit events to a day: the
//! personal transfer balance cap with proportional indexation (ss 294-35 and
//! 294-40), each day's excess (s 294-30), excess transfer balance earnings
//! (s 294-235) credited at the start of the next day (s 294-25(1) item 3), and
//! excess transfer balance tax for each excess transfer balance period
//! (s 294-230; Superannuation (Excess Transfer Balance Tax) Imposition Act
//! 2016 s 5), with IT(TP)A 1997 s 294-30. An estimate: the command takes no
//! determination or notice of the Commissioner's. Read on 22 November 2026:
//! the sections at the ATO's legal database, the Imposition Act and IT(TP)A
//! compilation 104 at the Federal Register of Legislation, and the ATO's
//! 'Calculating your personal transfer balance cap' (31 March 2026),
//! 'Transfer balance account' (18 June 2025) and 'Excess transfer balance'
//! (7 February 2025).

use num_bigint::BigInt;
use serde_json::Value;

mod account;
mod events;

use super::{Arg, Result, d, refuse, scope};
use crate::dates;
use crate::metadata as meta;
use account::Account;
use events::Event;

const KIND: &str = "transfer_balance";

fn fact_invalid<T>(message: String) -> Result<T> {
    refuse("transfer_balance.fact_invalid", message)
}

fn date_order<T>(message: String) -> Result<T> {
    refuse("transfer_balance.date_order", message)
}

/// The financial year a day falls in, by its first calendar year.
fn year_of(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    if m >= 7 { y } else { y - 1 }
}

fn label(start: i64) -> String {
    format!("{start}-{:02}", (start + 1) % 100)
}

/// The general transfer balance cap in dollars for the financial year
/// starting in `start` (ITAA 1997 s 294-35(3)).
fn general_cap(start: i64) -> i64 {
    let year = label(start);
    meta::EARLIER_TRANSFER_BALANCE_CAPS
        .iter()
        .find(|(y, _)| *y == year)
        .map(|(_, cap)| *cap)
        .or_else(|| {
            meta::CONTRIBUTION_CAPS
                .iter()
                .find(|(y, _)| *y == year)
                .map(|(_, (_, cap))| *cap)
        })
        .expect("a supported year")
        .parse()
        .expect("whole dollars")
}

/// An exact amount of `num / den` cents as dollars, rounded half up (away
/// from nil) to the cent.
pub(super) fn dollars(num: &BigInt, den: &BigInt) -> crate::decimal::Decimal {
    let two = num_bigint::BigUint::from(2_u8);
    let magnitude = (&two * num.magnitude() + den.magnitude()) / (&two * den.magnitude());
    let cents = BigInt::from(magnitude);
    let sign = if num.sign() == num_bigint::Sign::Minus && cents > BigInt::ZERO {
        "-"
    } else {
        ""
    };
    let text = format!("{:03}", cents);
    let (whole, part) = text.split_at(text.len() - 2);
    d(&format!("{sign}{whole}.{part}"))
}

fn date_text(value: &Arg, key: &str) -> Result<i64> {
    match value.text().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => fact_invalid(format!("{key} must be a date YYYY-MM-DD.")),
    }
}

/// `year` must be the financial year of `to`.
fn to_in_year(to: i64, period: &str) -> Result<()> {
    let found = label(year_of(to));
    if found == period {
        return Ok(());
    }
    refuse(
        "transfer_balance.year_mismatch",
        format!(
            "to ({}) is in {found}; year must be the financial year of to.",
            dates::format(to)
        ),
    )
}

/// The command: the account from its `events` to the end of `to`.
pub fn transfer_balance(
    events: &Arg,
    to: &Arg,
    previously_liable: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let period = scope(scope_confirmed, year, KIND)?;
    let to = date_text(to, "to")?;
    to_in_year(to, &period)?;
    let Arg::Bool(previously_liable) = previously_liable else {
        return refuse(
            "input.flag_invalid",
            "previously_liable must be a boolean.",
        );
    };
    let events = events::parse(events, to)?;
    let transitional = events::transitional(&events, to)?;
    let mut account = Account::new(&events, *previously_liable);
    let first = events[0].day;
    for day in first..=to {
        let today: Vec<&Event> = events.iter().filter(|e| e.day == day).collect();
        account.day(day, &today, transitional && day <= events::TRANSITION_END)?;
    }
    account.finish(&period, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dollars_round_half_up_away_from_nil() {
        let (n, den) = (BigInt::from, BigInt::from);
        assert_eq!(dollars(&n(-3_000_000_i64), &den(1)).to_string(), "-30000.00");
        assert_eq!(dollars(&n(1), &den(2)).to_string(), "0.01");
        assert_eq!(dollars(&n(-1), &den(3)).to_string(), "0.00");
        assert_eq!(dollars(&n(5), &den(1)).to_string(), "0.05");
    }

    #[test]
    fn a_day_is_in_its_financial_year() {
        let day = |s| dates::parse(s).unwrap();
        assert_eq!(label(year_of(day("2027-06-30"))), "2026-27");
        assert_eq!(label(year_of(day("2017-07-01"))), "2017-18");
        assert_eq!(general_cap(2017), 1_600_000);
        assert_eq!(general_cap(2026), 2_100_000);
    }
}
