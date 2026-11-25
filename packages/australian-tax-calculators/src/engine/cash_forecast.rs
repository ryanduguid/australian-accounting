// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `cash-forecast`: a 13-week cash forecast, as the site's cash page works it
//! (tools/business-calculators/cash, read on 26 November 2026): each week
//! closes at its opening plus receipts less payments; the funding gap is the
//! buffer less the lowest of the opening and every weekly close, never below
//! nil. One receipt may move later by whole weeks; a receipt moved past week
//! 13 is reported as deferred and does not enter the balance. The layout
//! follows business.gov.au's cash flow page. No statute sets a figure here.

use serde_json::{Map, Value, json};

use super::{Arg, Result, d, money, precise, refuse, result, scope, signed_money};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "cash_forecast";
const WEEKS: usize = 13;

pub struct CashForecastArgs<'a> {
    pub opening: &'a Arg,
    pub buffer: &'a Arg,
    pub weeks: &'a Arg,
    pub delayed_receipt: Option<&'a Arg>,
    pub start_date: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn amount(o: &Map<String, Value>, key: &str) -> Option<Decimal> {
    let text = o.get(key)?.as_str()?;
    let v = Decimal::parse_input(text).ok()?;
    money(&Arg::Decimal(v)).ok()
}

/// The 13 weeks' (receipts, payments).
fn weeks(given: &Arg) -> Result<Vec<(Decimal, Decimal)>> {
    let read = |v: &Value| {
        let o = v.as_object()?;
        let known = o
            .keys()
            .all(|k| matches!(k.as_str(), "receipts" | "payments"));
        known.then(|| Some((amount(o, "receipts")?, amount(o, "payments")?)))?
    };
    let rows: Option<Vec<_>> = match given {
        Arg::Json(Value::Array(items)) if items.len() == WEEKS => items.iter().map(read).collect(),
        _ => None,
    };
    rows.map_or_else(
        || {
            refuse(
                "cash_forecast.weeks_invalid",
                "weeks must be a list of 13 objects, each {receipts, payments} in AUD from 0, at most 2 dp.",
            )
        },
        Ok,
    )
}

/// The delayed receipt: (week index, amount, delay in weeks); none moves nothing.
fn delayed(given: Option<&Arg>) -> Result<(usize, Decimal, usize)> {
    let Some(given) = given else {
        return Ok((0, d("0"), 0));
    };
    let read = || {
        let Arg::Json(Value::Object(o)) = given else {
            return None;
        };
        let known = o
            .keys()
            .all(|k| matches!(k.as_str(), "week" | "amount" | "delay_weeks"));
        let week = o.get("week")?.as_u64().filter(|w| (1..=13).contains(w))?;
        let delay = o.get("delay_weeks")?.as_u64().filter(|w| *w <= 13)?;
        known.then(|| Some((week as usize - 1, amount(o, "amount")?, delay as usize)))?
    };
    read().map_or_else(
        || {
            refuse(
                "cash_forecast.delayed_receipt_invalid",
                "delayed_receipt must be {week, amount, delay_weeks}: week 1 to 13, an amount in AUD from 0 at most 2 dp, and 0 to 13 whole weeks.",
            )
        },
        Ok,
    )
}

/// Each week's first and last day, from the first day of week 1.
fn week_dates(given: Option<&Arg>) -> Result<Option<i64>> {
    match given {
        None => Ok(None),
        Some(g) => match g.text().and_then(dates::parse) {
            Some(day) if day + 13 * 7 - 1 <= dates::parse("2100-12-31").expect("a date") => {
                Ok(Some(day))
            }
            _ => refuse(
                "input.date_invalid",
                "start_date must be YYYY-MM-DD from 1900, with week 13 ending by 2100-12-31.",
            ),
        },
    }
}

/// Moves the delayed receipt; returns the amount deferred past week 13.
fn shift(
    rows: &mut [(Decimal, Decimal)],
    (week, moved, delay): (usize, Decimal, usize),
) -> Result<Decimal> {
    if moved.gt(&rows[week].0)? {
        return refuse(
            "cash_forecast.delay_above_receipts",
            format!(
                "The delayed receipt is more than week {}'s receipts.",
                week + 1
            ),
        );
    }
    rows[week].0 = rows[week].0.sub(&moved)?;
    if week + delay < WEEKS {
        rows[week + delay].0 = rows[week + delay].0.add(&moved)?;
        return Ok(d("0"));
    }
    Ok(moved)
}

/// One week's row of the result.
fn week_row(i: usize, start: Option<i64>, figures: [&Decimal; 4]) -> Value {
    let mut week = json!({"week": i + 1});
    if let Some(day) = start {
        let first = day + 7 * i64::try_from(i).expect("13 weeks");
        week["start_date"] = json!(dates::format(first));
        week["end_date"] = json!(dates::format(first + 6));
    }
    for (key, v) in ["opening", "receipts", "payments", "closing"]
        .into_iter()
        .zip(figures)
    {
        week[key] = json!(v.fixed(2, false));
    }
    week
}

/// The weeks run from `opening`: each week's row, the close and the lowest balance.
fn run_weeks(
    opening: &Decimal,
    rows: &[(Decimal, Decimal)],
    start: Option<i64>,
) -> Result<(Vec<Value>, Decimal, Decimal)> {
    let (mut balance, mut lowest) = (opening.clone(), opening.clone());
    let mut out = Vec::with_capacity(WEEKS);
    for (i, (receipts, payments)) in rows.iter().enumerate() {
        let open = balance.clone();
        balance = precise(|| Ok(balance.add(receipts)?.sub(payments)?))?;
        if balance.lt(&lowest)? {
            lowest = balance.clone();
        }
        out.push(week_row(i, start, [&open, receipts, payments, &balance]));
    }
    Ok((out, balance, lowest))
}

/// Closing cash each week, the lowest balance and the funding gap.
pub fn cash_forecast(a: &CashForecastArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let opening = signed_money(a.opening)?;
    let buffer = money(a.buffer)?;
    let mut rows = weeks(a.weeks)?;
    let move_it = delayed(a.delayed_receipt)?;
    let start = week_dates(a.start_date)?;
    let deferred = precise(|| shift(&mut rows, move_it))?;
    let (out, balance, lowest) = run_weeks(&opening, &rows, start)?;
    let short = precise(|| Ok(buffer.sub(&lowest)?))?;
    let gap = if short.gt(&d("0"))? { short } else { d("0") };
    let amounts = vec![
        ("closing_cash", balance),
        ("lowest_balance", lowest),
        ("funding_gap", gap),
        ("deferred_beyond_week_13", deferred),
    ];
    let mut value = result(KIND, &year, amounts, vec![])?;
    value["weeks"] = Value::Array(out);
    Ok(value)
}
