// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The account's events: each `{date, kind, amount}` checked against the
//! credits (ITAA 1997 s 294-25(1)) and debits (s 294-80(1)) the command
//! takes, and the ones it refuses with their reason.

use serde_json::Value;

use super::{date_order, fact_invalid};
use crate::dates;
use crate::decimal::Rounding;
use crate::engine::{Arg, Result, refuse, stated};

/// 1 July 2017, when transfer balance accounts start (s 294-15(2)).
pub(super) const START: i64 = 17_348;
/// 31 December 2017, the end of IT(TP)A s 294-30's 6 months.
pub(super) const TRANSITION_END: i64 = 17_531;

/// (kind, credit, authority) for each event the command takes.
const KINDS: [(&str, bool, &str); 7] = [
    (
        "existing_income_stream",
        true,
        "ITAA 1997 s 294-25(1) item 1: the value just before 1 July 2017 of the interest supporting an income stream you were then the retirement phase recipient of, credited on 1 July 2017",
    ),
    (
        "new_income_stream",
        true,
        "ITAA 1997 s 294-25(1) item 2: the value on the starting day of the interest supporting an income stream you start to be the retirement phase recipient of, credited that day",
    ),
    (
        "lrba_repayment",
        true,
        "ITAA 1997 s 294-25(1) item 4 and s 294-55: the increase in value from a payment under a limited recourse borrowing arrangement of a small fund, at the payment; contracts from 1 July 2017 only (IT(TP)A 1997 s 294-55)",
    ),
    (
        "commutation",
        false,
        "ITAA 1997 s 294-80(1) item 1: the superannuation lump sum from commuting a retirement phase income stream in full or in part, when received",
    ),
    (
        "reduced_super",
        false,
        "ITAA 1997 s 294-80(1) item 3 and s 294-85: a loss from fraud or dishonesty with a conviction, or a Bankruptcy Act 1966 s 139ZQ payment, notified to the Commissioner in the approved form",
    ),
    (
        "payment_split",
        false,
        "ITAA 1997 s 294-80(1) item 4 and s 294-90: the split proportion of the interest's value under a family law payment split, notified to the Commissioner in the approved form",
    ),
    (
        "stopped_retirement_phase",
        false,
        "ITAA 1997 s 294-80(1) items 5 and 6: the value of the interest when an income stream stops being in the retirement phase",
    ),
];

/// Events the command refuses, with the reason.
const REFUSED: [(&str, &str); 9] = [
    (
        "reversionary_income_stream",
        "A reversionary income stream is credited at the end of 12 months from the day benefits first become payable to you (ITAA 1997 s 294-25(1) items 1(b) and 2(b)); this command does not take it.",
    ),
    (
        "child_recipient",
        "A child recipient's cap is the cap increments of ITAA 1997 Subdiv 294-E (ss 294-185 to 294-200), not the general cap; this command does not work it.",
    ),
    (
        "capped_defined_benefit",
        "Capped defined benefit income streams have special values and modified credits and excess under ITAA 1997 Subdiv 294-D; this command does not work them.",
    ),
    (
        "structured_settlement",
        "A structured settlement contribution's debit (ITAA 1997 s 294-80(1) item 2, IT(TP)A 1997 s 294-80) is not worked by this command.",
    ),
    (
        "regulation_credit",
        "Credits under regulations (ITAA 1997 s 294-25(1) item 5) are not worked by this command.",
    ),
    (
        "regulation_debit",
        "Debits under regulations (ITAA 1997 s 294-80(1) item 8) are not worked by this command.",
    ),
    (
        "excess_determination",
        "An excess transfer balance determination stops earnings credits (ITAA 1997 s 294-25(2)) and changes the tax (s 294-230(3)(b)); this command gives an estimate without one and does not take the Commissioner's determination.",
    ),
    (
        "commutation_authority",
        "A commutation authority follows the Commissioner's determination (TAA 1953 Sch 1 Subdiv 136-B); this command does not take determinations or authorities.",
    ),
    (
        "non_commutable_notice",
        "A notice of non-commutable excess transfer balance (TAA 1953 Sch 1 s 136-70; ITAA 1997 s 294-80(1) item 7) is the Commissioner's; this command does not take it.",
    ),
];

/// One credit or debit, in cents.
pub(super) struct Event {
    pub(super) day: i64,
    pub(super) kind: &'static str,
    pub(super) credit: bool,
    pub(super) authority: &'static str,
    pub(super) cents: i64,
}

fn kind_of(name: &str, item: &str) -> Result<(&'static str, bool, &'static str)> {
    if let Some((_, why)) = REFUSED.iter().find(|(k, _)| *k == name) {
        return refuse("transfer_balance.event_refused", format!("{item}: {why}"));
    }
    match KINDS.iter().find(|(k, _, _)| *k == name) {
        Some(found) => Ok(*found),
        None => fact_invalid(format!(
            "{item}: kind must be one of {}.",
            KINDS.map(|(k, _, _)| k).join(", ")
        )),
    }
}

fn event(value: &Value, item: &str) -> Result<Event> {
    let Value::Object(fields) = value else {
        return fact_invalid(format!("{item} must be an object."));
    };
    stated::known_keys(fields, item, "an event", &["date", "kind", "amount"])?;
    let Some(day) = stated::required(fields, item, "date")?
        .as_str()
        .and_then(dates::parse)
    else {
        return fact_invalid(format!("{item}: date must be a date YYYY-MM-DD."));
    };
    let name = stated::required(fields, item, "kind")?
        .as_str()
        .unwrap_or("");
    let (kind, credit, authority) = kind_of(name, item)?;
    let amount = stated::amount(fields, item, "amount")?;
    let cents: i64 = amount
        .quantize(-2, Rounding::HalfUp)?
        .to_string()
        .replace('.', "")
        .parse()
        .expect("cents of a checked amount");
    if cents <= 0 {
        return fact_invalid(format!("{item}: amount must be above nil."));
    }
    Ok(Event {
        day,
        kind,
        credit,
        authority,
        cents,
    })
}

/// Checks each event's date against the account's start and `to`.
fn check_dates(events: &[Event], to: i64) -> Result<()> {
    for (i, e) in events.iter().enumerate() {
        let item = format!("event {}", i + 1);
        if e.day < START {
            return date_order(format!(
                "{item}: no event arises before 1 July 2017 (ITAA 1997 s 294-15(2))."
            ));
        }
        if e.kind == "existing_income_stream" && e.day != START {
            return date_order(format!(
                "{item}: an existing_income_stream credit arises on 1 July 2017 (s 294-25(1) item 1)."
            ));
        }
        if e.day > to {
            return date_order(format!(
                "{item}: is after to; give to on or after the last event."
            ));
        }
    }
    let first = events
        .iter()
        .map(|e| e.day)
        .min()
        .expect("at least one event");
    if !events.iter().any(|e| e.day == first && e.credit) {
        return date_order("The first event's day must have a credit: the account starts with a retirement phase income stream (ITAA 1997 s 294-15).".into());
    }
    Ok(())
}

/// The `events` list, in date order (a stable sort keeps the given order
/// within a day, which does not change the end-of-day balance).
pub(super) fn parse(events: &Arg, to: i64) -> Result<Vec<Event>> {
    let items = match events {
        Arg::Json(Value::Array(items)) if !items.is_empty() && items.len() <= 1000 => items,
        _ => return fact_invalid("events must be a list of 1 to 1000 objects.".into()),
    };
    let mut out = items
        .iter()
        .enumerate()
        .map(|(i, v)| event(v, &format!("event {}", i + 1)))
        .collect::<Result<Vec<_>>>()?;
    check_dates(&out, to)?;
    out.sort_by_key(|e| e.day);
    Ok(out)
}

/// Whether IT(TP)A 1997 s 294-30 disregards the excess on each day from
/// 1 July to 31 December 2017: only item 1 credits in that period, their sum
/// above the $1,600,000 cap and at most $1,700,000, and debits by 31 December
/// 2017 of at least that excess.
pub(super) fn transitional(events: &[Event], to: i64) -> Result<bool> {
    let window: Vec<&Event> = events.iter().filter(|e| e.day <= TRANSITION_END).collect();
    let credits = window.iter().filter(|e| e.credit);
    if credits.clone().any(|e| e.kind != "existing_income_stream") {
        return Ok(false);
    }
    let sum: i64 = credits.map(|e| e.cents).sum();
    let (cap, limit) = (160_000_000, 170_000_000);
    if sum <= cap || sum > limit {
        return Ok(false);
    }
    let debits: i64 = window.iter().filter(|e| !e.credit).map(|e| e.cents).sum();
    if debits >= sum - cap {
        return Ok(true);
    }
    if to < TRANSITION_END {
        return refuse(
            "transfer_balance.transition_undecided",
            "IT(TP)A 1997 s 294-30 may disregard this excess if debits by 31 December 2017 reach it; give to on or after 31 December 2017.",
        );
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_constants_are_their_dates() {
        assert_eq!(dates::parse("2017-07-01"), Some(START));
        assert_eq!(dates::parse("2017-12-31"), Some(TRANSITION_END));
    }
}
