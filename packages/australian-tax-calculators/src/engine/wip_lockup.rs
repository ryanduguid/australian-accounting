// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `wip-lockup`: a professional firm's unbilled work in progress aged at a
//! date, with WIP days, receivable days and the lock-up days they add to. The
//! day counts mirror Ozzit's `oz.WIPDaysλ` and `oz.ReceivableDaysλ` (function
//! reference v3.7.0, read on 24 November 2026): balance / flow x days, days
//! defaulting to 365, a nil flow refused as Ozzit's #DIV/0! and days below 1
//! as its #NUM!. No Standard or Act defines lock-up or ageing buckets; how the
//! WIP is measured is the firm's.

use serde_json::{Value, json};

use super::{Arg, Result, d, date_arg, money, precise, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "wip_lockup";
const MOST_ENTRIES: usize = 5000;
const MOST_BUCKETS: usize = 6;
const MOST_BUCKET_DAYS: u64 = 3650;
const MOST_DAYS: i64 = 3660;
const DEFAULT_BUCKETS: [u64; 3] = [30, 60, 90];

/// The worksheet's inputs, as the caller gave them.
pub struct WipLockupArgs<'a> {
    pub entries: &'a Arg,
    pub as_at: &'a Arg,
    pub bucket_days: Option<&'a Arg>,
    pub fee_revenue: &'a Arg,
    pub receivables: &'a Arg,
    pub billings: &'a Arg,
    pub days: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One entry's date and amount.
fn entry(v: &Value, as_at: i64, n: usize) -> Result<(i64, Decimal)> {
    let bad = || {
        refuse(
            "wip_lockup.entry_invalid",
            format!(
                "entry {n}: needs a label, a YYYY-MM-DD date on or before as_at and an amount from 0, at most 2 dp."
            ),
        )
    };
    let Some(o) = v.as_object() else { return bad() };
    let label = o.get("label").and_then(Value::as_str);
    let date = o.get("date").and_then(Value::as_str).and_then(dates::parse);
    let amount = o
        .get("amount")
        .and_then(Value::as_str)
        .and_then(|t| Decimal::parse_input(t).ok())
        .map(|v| money(&Arg::Decimal(v)));
    let known = o
        .keys()
        .all(|k| matches!(k.as_str(), "label" | "date" | "amount"));
    match (label, date, amount) {
        (Some(l), Some(day), Some(Ok(a))) if known && !l.trim().is_empty() && day <= as_at => {
            Ok((day, a))
        }
        _ => bad(),
    }
}

fn entries(given: &Arg, as_at: i64) -> Result<Vec<(i64, Decimal)>> {
    match given {
        Arg::Json(Value::Array(items)) if (1..=MOST_ENTRIES).contains(&items.len()) => items
            .iter()
            .enumerate()
            .map(|(i, v)| entry(v, as_at, i + 1))
            .collect(),
        _ => refuse(
            "wip_lockup.entry_invalid",
            format!("entries must be a list of 1 to {MOST_ENTRIES} objects."),
        ),
    }
}

fn buckets(given: Option<&Arg>) -> Result<Vec<u64>> {
    let Some(given) = given else {
        return Ok(DEFAULT_BUCKETS.to_vec());
    };
    let read: Option<Vec<u64>> = match given {
        Arg::Json(Value::Array(items)) if (1..=MOST_BUCKETS).contains(&items.len()) => {
            items.iter().map(Value::as_u64).collect()
        }
        _ => None,
    };
    match read {
        Some(b)
            if b[0] >= 1
                && b.windows(2).all(|w| w[0] < w[1])
                && b[b.len() - 1] <= MOST_BUCKET_DAYS =>
        {
            Ok(b)
        }
        _ => refuse(
            "wip_lockup.buckets_invalid",
            format!(
                "bucket_days must be 1 to {MOST_BUCKETS} strictly ascending whole numbers from 1 to {MOST_BUCKET_DAYS}."
            ),
        ),
    }
}

fn days(given: Option<&Arg>) -> Result<i64> {
    match given {
        None => Ok(365),
        Some(Arg::Int(n)) if (1..=MOST_DAYS).contains(n) => Ok(*n),
        Some(_) => refuse(
            "wip_lockup.days_invalid",
            format!("days must be a whole number from 1 to {MOST_DAYS}, the days the flows cover."),
        ),
    }
}

/// A flow above nil, or Ozzit's #DIV/0!.
fn flow(given: &Arg, name: &str) -> Result<Decimal> {
    let v = money(given)?;
    if v.is_zero() {
        return refuse(
            "wip_lockup.flow_nil",
            format!("{name} is nil, so the day count has no denominator."),
        );
    }
    Ok(v)
}

/// Each bucket's bounds in days (the last open) and the entries' total in it.
fn aged(all: &[(i64, Decimal)], as_at: i64, bounds: &[u64]) -> Result<Vec<Value>> {
    let mut totals = vec![d("0.00"); bounds.len() + 1];
    for (day, amount) in all {
        let age = u64::try_from(as_at - day).expect("on or before as_at");
        let i = bounds
            .iter()
            .position(|b| age <= *b)
            .unwrap_or(bounds.len());
        totals[i] = totals[i].add(amount)?;
    }
    let mut low = 0;
    let mut out = Vec::with_capacity(totals.len());
    for (i, total) in totals.iter().enumerate() {
        let high = bounds.get(i).map_or(Value::Null, |b| json!(b));
        out.push(json!({"from_days": low, "to_days": high, "amount": total.to_string()}));
        low = bounds.get(i).map_or(0, |b| b + 1);
    }
    Ok(out)
}

fn four(v: &Decimal) -> Result<String> {
    Ok(v.quantize(-4, Rounding::HalfUp)?.to_string())
}

/// `as_at`, refused outside the financial year.
fn as_at_in(given: &Arg, year: &str) -> Result<i64> {
    let as_at = date_arg(given, "as_at")?;
    let first: i64 = year[..4].parse().expect("a supported year");
    let in_year = |y: i64, md: &str| dates::parse(&format!("{y}-{md}")).expect("a date");
    if as_at < in_year(first, "07-01") || as_at > in_year(first + 1, "06-30") {
        return refuse(
            "wip_lockup.as_at_outside_year",
            format!("as_at must fall in {year}."),
        );
    }
    Ok(as_at)
}

/// WIP, and the WIP, receivable and lock-up days, unrounded.
fn day_counts(
    all: &[(i64, Decimal)],
    flows: [&Decimal; 3],
    span: &Decimal,
) -> Result<[Decimal; 4]> {
    let [fees, receivables, billings] = flows;
    precise(|| {
        let mut wip = d("0.00");
        for (_, amount) in all {
            wip = wip.add(amount)?;
        }
        let wip_days = wip.div(fees)?.mul(span)?;
        let receivable_days = receivables.div(billings)?.mul(span)?;
        let lock_up = wip_days.add(&receivable_days)?;
        Ok([wip, wip_days, receivable_days, lock_up])
    })
}

/// Ageing at `as_at`, and WIP, receivable and lock-up days.
pub fn wip_lockup(a: &WipLockupArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let as_at = as_at_in(a.as_at, &year)?;
    let all = entries(a.entries, as_at)?;
    let bounds = buckets(a.bucket_days)?;
    let fees = flow(a.fee_revenue, "fee_revenue")?;
    let receivables = money(a.receivables)?;
    let billings = flow(a.billings, "billings")?;
    let span = Decimal::from_int(days(a.days)?);
    let groups = precise(|| aged(&all, as_at, &bounds))?;
    let [wip, wip_days, receivable_days, lock_up] =
        day_counts(&all, [&fees, &receivables, &billings], &span)?;
    let rates = vec![
        rate("wip_days", four(&wip_days)?),
        rate("receivable_days", four(&receivable_days)?),
        rate("lock_up_days", four(&lock_up)?),
    ];
    let amounts = vec![
        ("wip", wip),
        ("receivables", receivables),
        ("wip_days", wip_days),
        ("receivable_days", receivable_days),
        ("lock_up_days", lock_up),
    ];
    let mut out = result(KIND, &year, amounts, rates)?;
    out["ageing"] = Value::Array(groups);
    Ok(out)
}
