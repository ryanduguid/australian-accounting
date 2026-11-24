// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `deferred-revenue`: the revenue recognised and the contract liability at
//! each stated period end for one performance obligation, from its price, its
//! performance period and the progress pattern the operator states: time
//! elapsed by day or by whole month (AASB 15 paras 39, 40, B14 and B18) or
//! milestones reached (para B15). Consideration received or due against
//! revenue gives the contract liability, or a contract asset or receivable
//! (paras 105 to 108). The paragraphs are those of the AASB 15 compilations of
//! December 2022 (COMPdec22_01-23 and COMPdec22_07-26), read on 24 November
//! 2026; none of them was amended after 2014. Whether the obligation is
//! satisfied over time and which measure of progress is faithful are the
//! operator's judgements. The Standard's wording is not reproduced.

use serde_json::{Map, Value, json};

use super::{Arg, Result, d, date_arg, flag, money, precise, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "deferred_revenue";
const MOST_MONTHS: i64 = 120;
const MOST_PERIOD_ENDS: usize = 120;
const MOST_MILESTONES: usize = 100;
const MOST_CONSIDERATION: usize = 500;

/// The worksheet's inputs, as the caller gave them.
pub struct DeferredRevenueArgs<'a> {
    pub contract_amount: &'a Arg,
    pub start: &'a Arg,
    pub end: &'a Arg,
    pub pattern: &'a Arg,
    pub milestones: Option<&'a Arg>,
    pub period_ends: &'a Arg,
    pub consideration: &'a Arg,
    pub variable_consideration: &'a Arg,
    pub significant_financing_component: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The stated progress pattern.
enum Pattern {
    Daily,
    Monthly,
    Milestones(Vec<(i64, Decimal)>),
}

/// One dated amount: a milestone or consideration.
fn dated(v: &Value, code: &'static str, what: &str, n: usize) -> Result<(i64, Decimal)> {
    let bad = || {
        refuse(
            code,
            format!("{what} {n}: needs a YYYY-MM-DD date and an amount above 0, at most 2 dp."),
        )
    };
    let Some(o) = v.as_object() else { return bad() };
    let date = o.get("date").and_then(Value::as_str).and_then(dates::parse);
    let amount = o
        .get("amount")
        .and_then(Value::as_str)
        .and_then(|t| Decimal::parse_input(t).ok())
        .map(|v| money(&Arg::Decimal(v)));
    let known = |k: &String| k == "date" || k == "amount" || (k == "label" && what == "milestone");
    match (date, amount) {
        (Some(day), Some(Ok(amount))) if !amount.is_zero() && o.keys().all(known) => {
            Ok((day, amount))
        }
        _ => bad(),
    }
}

fn list<'a>(
    given: &'a Arg,
    code: &'static str,
    what: &str,
    most: usize,
    least: usize,
) -> Result<&'a Vec<Value>> {
    match given {
        Arg::Json(Value::Array(items)) if (least..=most).contains(&items.len()) => Ok(items),
        _ => refuse(
            code,
            format!("{what} must be a list of {least} to {most} entries."),
        ),
    }
}

fn total(items: &[(i64, Decimal)], through: i64) -> Result<Decimal> {
    let mut sum = d("0.00");
    for (_, amount) in items.iter().filter(|(day, _)| *day <= through) {
        sum = sum.add(amount)?;
    }
    Ok(sum)
}

/// The financial year's first and last days.
fn year_days(year: &str) -> (i64, i64) {
    let first: i64 = year[..4].parse().expect("a supported year");
    let day = |y: i64, md: &str| dates::parse(&format!("{y}-{md}")).expect("a date");
    (day(first, "07-01"), day(first + 1, "06-30"))
}

fn period(a: &DeferredRevenueArgs, year: &str) -> Result<(i64, i64)> {
    let start = date_arg(a.start, "start")?;
    let end = date_arg(a.end, "end")?;
    let (first, last) = year_days(year);
    if end < start || end >= dates::add_months(start, MOST_MONTHS) || start < first || start > last
    {
        return refuse(
            "deferred_revenue.dates_invalid",
            format!(
                "end must be on or after start and the period shorter than {MOST_MONTHS} months, with start in {year}."
            ),
        );
    }
    Ok((start, end))
}

fn milestones(given: &Arg, amount: &Decimal, start: i64, end: i64) -> Result<Vec<(i64, Decimal)>> {
    const CODE: &str = "deferred_revenue.milestones_invalid";
    let items = list(given, CODE, "milestones", MOST_MILESTONES, 1)?;
    let read = items
        .iter()
        .enumerate()
        .map(|(i, v)| dated(v, CODE, "milestone", i + 1))
        .collect::<Result<Vec<_>>>()?;
    if read.iter().any(|(day, _)| *day < start || *day > end)
        || !total(&read, end)?.equals(amount)?
    {
        return refuse(
            CODE,
            "Each milestone must fall in the performance period, and the milestones must sum to contract_amount.",
        );
    }
    Ok(read)
}

fn pattern(a: &DeferredRevenueArgs, amount: &Decimal, start: i64, end: i64) -> Result<Pattern> {
    let bad = |why: &str| refuse("deferred_revenue.pattern_invalid", why.to_string());
    let given = a.pattern.text();
    match (given, a.milestones) {
        (Some("milestones"), Some(m)) => {
            Ok(Pattern::Milestones(milestones(m, amount, start, end)?))
        }
        (Some("milestones"), None) => bad("pattern milestones needs milestones."),
        (Some("daily" | "monthly"), Some(_)) => {
            bad("milestones is taken only with pattern milestones.")
        }
        (Some("daily"), None) => Ok(Pattern::Daily),
        (Some("monthly"), None) => {
            let (_, _, first) = dates::to_civil(start);
            let (_, _, after) = dates::to_civil(end + 1);
            if first != 1 || after != 1 {
                return refuse(
                    "deferred_revenue.month_boundary",
                    "pattern monthly needs start on a month's first day and end on a month's last day.",
                );
            }
            Ok(Pattern::Monthly)
        }
        _ => bad("pattern must be daily, monthly or milestones."),
    }
}

fn period_ends(given: &Arg) -> Result<Vec<i64>> {
    const CODE: &str = "deferred_revenue.period_ends_invalid";
    let why = format!(
        "period_ends must be 1 to {MOST_PERIOD_ENDS} YYYY-MM-DD dates, strictly ascending."
    );
    let items = list(given, CODE, "period_ends", MOST_PERIOD_ENDS, 1)?;
    let days: Option<Vec<i64>> = items
        .iter()
        .map(|v| v.as_str().and_then(dates::parse))
        .collect();
    match days {
        Some(days) if days.windows(2).all(|w| w[0] < w[1]) => Ok(days),
        _ => refuse(CODE, why),
    }
}

fn consideration(given: &Arg, amount: &Decimal) -> Result<Vec<(i64, Decimal)>> {
    const CODE: &str = "deferred_revenue.consideration_invalid";
    let items = list(given, CODE, "consideration", MOST_CONSIDERATION, 0)?;
    let read = items
        .iter()
        .enumerate()
        .map(|(i, v)| dated(v, CODE, "consideration", i + 1))
        .collect::<Result<Vec<_>>>()?;
    if total(&read, i64::MAX)?.gt(amount)? {
        return refuse(
            CODE,
            "Consideration above contract_amount needs a refund liability (AASB 15 para 55), which this worksheet does not measure.",
        );
    }
    Ok(read)
}

/// Whole calendar months from `start` (a month's first day) ended by `day`.
fn months_ended(start: i64, day: i64) -> i64 {
    let (sy, sm, _) = dates::to_civil(start);
    let (y, m, _) = dates::to_civil(day);
    let (_, _, next) = dates::to_civil(day + 1);
    (y * 12 + m) - (sy * 12 + sm) + i64::from(next == 1)
}

/// Revenue to date at `day` before rounding (paras 39, B14, B15 and B18).
fn to_date(p: &Pattern, amount: &Decimal, start: i64, end: i64, day: i64) -> Result<Decimal> {
    let share = |done: i64, whole: i64| -> Result<Decimal> {
        let done = Decimal::from_int(done.clamp(0, whole));
        Ok(amount.mul(&done)?.div(&Decimal::from_int(whole))?)
    };
    match p {
        Pattern::Daily => share(day - start + 1, end - start + 1),
        Pattern::Monthly => share(months_ended(start, day), months_ended(start, end)),
        Pattern::Milestones(m) => total(m, day),
    }
}

fn row(day: i64, revenue: &Decimal, before: &Decimal, paid: &Decimal) -> Result<Value> {
    let net = paid.sub(revenue)?;
    let zero = d("0.00");
    let (liability, asset) = if net.gt(&zero)? {
        (net, zero)
    } else {
        (zero, net.neg()?)
    };
    let mut out = Map::new();
    out.insert("period_end".into(), json!(dates::format(day)));
    out.insert(
        "revenue_in_period".into(),
        json!(revenue.sub(before)?.to_string()),
    );
    out.insert("revenue_to_date".into(), json!(revenue.to_string()));
    out.insert("consideration_to_date".into(), json!(paid.to_string()));
    out.insert("contract_liability".into(), json!(liability.to_string()));
    out.insert(
        "contract_asset_or_receivable".into(),
        json!(asset.to_string()),
    );
    Ok(Value::Object(out))
}

fn schedule(
    p: &Pattern,
    amount: &Decimal,
    span: (i64, i64),
    ends: &[i64],
    paid: &[(i64, Decimal)],
) -> Result<Vec<Value>> {
    let mut before = d("0.00");
    let mut rows = Vec::with_capacity(ends.len());
    for &day in ends {
        let revenue = to_date(p, amount, span.0, span.1, day)?.quantize(-2, Rounding::HalfUp)?;
        rows.push(row(day, &revenue, &before, &total(paid, day)?)?);
        before = revenue;
    }
    Ok(rows)
}

/// AASB 15 for one performance obligation's stated pattern of progress.
pub fn deferred_revenue(a: &DeferredRevenueArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    if flag(a.variable_consideration, "variable_consideration")? {
        return refuse(
            "deferred_revenue.variable_consideration",
            "Variable consideration is estimated and constrained under AASB 15 paras 50 to 59, which this worksheet does not apply.",
        );
    }
    if flag(
        a.significant_financing_component,
        "significant_financing_component",
    )? {
        return refuse(
            "deferred_revenue.financing_component",
            "A significant financing component adjusts the price under AASB 15 paras 60 to 65, which this worksheet does not apply.",
        );
    }
    let amount = money(a.contract_amount)?;
    if amount.is_zero() {
        return refuse(
            "deferred_revenue.amount_invalid",
            "contract_amount must be above 0.",
        );
    }
    let (start, end) = period(a, &year)?;
    let p = pattern(a, &amount, start, end)?;
    let ends = period_ends(a.period_ends)?;
    let paid = consideration(a.consideration, &amount)?;
    let rows = precise(|| schedule(&p, &amount, (start, end), &ends, &paid))?;
    let last = &rows[rows.len() - 1];
    let get = |k: &str| d(last[k].as_str().expect("an amount"));
    let recognised = get("revenue_to_date");
    let amounts = vec![
        ("contract_amount", amount.clone()),
        ("revenue_recognised", recognised.clone()),
        ("revenue_not_yet_recognised", amount.sub(&recognised)?),
        ("contract_liability", get("contract_liability")),
        (
            "contract_asset_or_receivable",
            get("contract_asset_or_receivable"),
        ),
    ];
    let mut out = result(KIND, &year, amounts, vec![])?;
    out["periods"] = Value::Array(rows);
    Ok(out)
}
