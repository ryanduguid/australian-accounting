// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! An agreement's payment regime: one level instalment at a frequency, or
//! dated segments of instalments, as amounts due on the boundaries of its
//! arrangement payment periods (ITAA 1997 s 240-70), counted from the start
//! in months or in weeks.

use serde_json::Value;

use crate::dates;
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, int, money, refuse};

/// What periods are counted in: calendar months from the start date's day of
/// the month (or the month's last day when it is shorter), or weeks of 7 days.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Unit {
    Month,
    Week,
}

impl Unit {
    /// The day `n` units after `start`.
    pub(super) fn date(self, start: i64, n: i64) -> i64 {
        match self {
            Unit::Month => dates::add_months(start, n),
            Unit::Week => start + 7 * n,
        }
    }

    /// The longest term: 30 years, as 360 months or 1,565 weeks.
    fn most(self) -> i64 {
        match self {
            Unit::Month => 360,
            Unit::Week => 1565,
        }
    }
}

/// The frequencies taken: (name, unit, units from one instalment to the next).
const FREQUENCIES: [(&str, Unit, i64); 5] = [
    ("weekly", Unit::Week, 1),
    ("fortnightly", Unit::Week, 2),
    ("monthly", Unit::Month, 1),
    ("quarterly", Unit::Month, 3),
    ("annual", Unit::Month, 12),
];

/// ITAA 1997 s 240-70(2): a period over 6 months is split into parts of 6
/// months, each its own arrangement payment period.
const LONGEST_PERIOD_MONTHS: i64 = 6;

/// A run of `count` level instalments of `amount`, `length` units apart.
struct Segment {
    amount: Decimal,
    count: i64,
    unit: Unit,
    length: i64,
}

/// The payments the agreement states, the balloon apart.
pub(super) struct Regime {
    pub unit: Unit,
    /// Each arrangement payment period, from and to in units from the start.
    pub periods: Vec<(i64, i64)>,
    /// The instalments due on each boundary, 0 to the term's end.
    pub due: Vec<Decimal>,
    /// The instalments' total, written as `total_payments`' formula has it.
    pub total: Decimal,
    /// `amount x count` for each segment, or `None` for a level regime.
    pub terms: Option<String>,
}

impl Regime {
    /// The term in units: the end of the last period.
    pub fn term(&self) -> i64 {
        self.periods.last().map_or(0, |p| p.1)
    }
}

fn frequency(given: Option<&str>, what: &str) -> Result<(Unit, i64)> {
    match FREQUENCIES.iter().find(|(name, _, _)| Some(*name) == given) {
        Some((_, unit, length)) => Ok((*unit, *length)),
        None => refuse(
            "hire_purchase.frequency_invalid",
            format!("{what} must be weekly, fortnightly, monthly, quarterly or annual."),
        ),
    }
}

/// The instalments an agreement states, before their timing is known.
pub(super) struct Plan {
    segments: Vec<Segment>,
    /// `amount x count` for each segment, or `None` for a level regime.
    terms: Option<String>,
}

/// The level regime: `instalments` instalments of `amount` at `frequency`
/// (monthly when not given).
pub(super) fn level(
    amount: Decimal,
    instalments: &Arg,
    frequency_given: Option<&Arg>,
) -> Result<Plan> {
    let (unit, length) = match frequency_given {
        None => (Unit::Month, 1),
        Some(arg) => frequency(arg.text(), "frequency")?,
    };
    let name = frequency_given.and_then(Arg::text).unwrap_or("monthly");
    let most = unit.most() / length;
    let count = match instalments.int() {
        Some(Some(n)) if (1..=most).contains(&n) => n,
        _ => {
            return refuse(
                "hire_purchase.instalments_unsupported",
                format!("instalments must be an integer from 1 to {most}, {name}."),
            );
        }
    };
    let segment = Segment {
        amount,
        count,
        unit,
        length,
    };
    Ok(Plan {
        segments: vec![segment],
        terms: None,
    })
}

fn segment_invalid<T>(n: usize, why: &str) -> Result<T> {
    refuse(
        "hire_purchase.segments_invalid",
        format!("segment {n}: {why}."),
    )
}

/// One segment's stated facts: its first period's start day and the segment.
fn read_segment(n: usize, value: &Value) -> Result<(i64, Segment)> {
    let Some(fields) = value.as_object() else {
        return segment_invalid(n, "must be an object");
    };
    if let Some(key) = fields
        .keys()
        .find(|k| !matches!(k.as_str(), "start" | "count" | "amount" | "frequency"))
    {
        return refuse(
            "input.field_unknown",
            format!("segment {n}: {key} is not a fact a segment takes"),
        );
    }
    let text = |key: &str| fields.get(key).and_then(Value::as_str);
    let Some(start) = text("start").and_then(dates::parse) else {
        return segment_invalid(n, "start must be a YYYY-MM-DD date from 1900 to 2100");
    };
    let Some(count) = fields
        .get("count")
        .and_then(Value::as_i64)
        .filter(|c| *c >= 1)
    else {
        return segment_invalid(n, "count must be a JSON integer of 1 or more");
    };
    let Some(amount) = text("amount").and_then(|t| Decimal::parse(t).ok()) else {
        return segment_invalid(n, "amount must be a decimal string");
    };
    let amount = money(&Arg::Decimal(amount))?;
    let (unit, length) = frequency(text("frequency"), &format!("segment {n}: frequency"))?;
    Ok((
        start,
        Segment {
            amount,
            count,
            unit,
            length,
        },
    ))
}

/// Dated segments, the first starting on `start`, each where the one before
/// it ends, all counted in months or all in weeks.
pub(super) fn segments(list: &Arg, start: i64) -> Result<Plan> {
    let Arg::Json(Value::Array(items)) = list else {
        return refuse(
            "hire_purchase.segments_invalid",
            "segments must be a non-empty list of objects, each with start, count, amount and frequency.",
        );
    };
    if items.is_empty() {
        return refuse(
            "hire_purchase.segments_invalid",
            "segments must be a non-empty list of objects, each with start, count, amount and frequency.",
        );
    }
    let read: Vec<(i64, Segment)> = items
        .iter()
        .enumerate()
        .map(|(i, item)| read_segment(i + 1, item))
        .collect::<Result<_>>()?;
    run_on(&read, start)?;
    if read.iter().all(|(_, s)| s.amount.is_zero()) {
        return refuse(
            "hire_purchase.amount_nil",
            "At least one segment's amount must be more than nil.",
        );
    }
    let segments: Vec<Segment> = read.into_iter().map(|(_, s)| s).collect();
    let terms = segments
        .iter()
        .map(|s| format!("{} x {}", s.amount, s.count))
        .collect::<Vec<_>>()
        .join(" + ");
    Ok(Plan {
        segments,
        terms: Some(terms),
    })
}

/// Refuses segments, each with its first period's start day, that do not
/// all count in one unit, each starting where the one before it ends from
/// `start`, and ending within 30 years.
fn run_on(read: &[(i64, Segment)], start: i64) -> Result<()> {
    let unit = read[0].1.unit;
    let mut at = 0;
    for (i, (day, segment)) in read.iter().enumerate() {
        if segment.unit != unit {
            return refuse(
                "hire_purchase.frequencies_mixed",
                "Segments must all be weekly or fortnightly, or all monthly, quarterly or annual: periods are counted in weeks or in months, not both.",
            );
        }
        let expected = unit.date(start, at);
        if *day != expected {
            return refuse(
                "hire_purchase.segments_not_contiguous",
                format!(
                    "segment {}: start must be {}, {}.",
                    i + 1,
                    dates::format(expected),
                    if i == 0 {
                        "the start_date"
                    } else {
                        "the end of the segment before it"
                    }
                ),
            );
        }
        // Bounded before multiplying, so no count can overflow the term.
        if segment.count > (unit.most() - at) / segment.length {
            return refuse(
                "hire_purchase.instalments_unsupported",
                format!(
                    "segments must end within 30 years of the start_date ({} {}).",
                    unit.most(),
                    if unit == Unit::Month {
                        "months"
                    } else {
                        "weeks"
                    }
                ),
            );
        }
        at += segment.count * segment.length;
    }
    Ok(())
}

/// The periods and amounts due: each instalment at its period's start in
/// advance or its end in arrears, and, where `split` (Division 240 applies),
/// each period over 6 months split.
pub(super) fn build(plan: Plan, in_advance: bool, split: bool) -> Result<Regime> {
    let Plan { segments, terms } = plan;
    let unit = segments[0].unit;
    let term: i64 = segments.iter().map(|s| s.count * s.length).sum();
    let mut due = vec![int(0); term as usize + 1];
    let (mut periods, mut total, mut at) = (Vec::new(), int(0), 0);
    for segment in &segments {
        let part = match unit {
            Unit::Month if split => LONGEST_PERIOD_MONTHS,
            _ => segment.length,
        };
        for _ in 0..segment.count {
            let end = at + segment.length;
            let paid = if in_advance { at } else { end } as usize;
            due[paid] = due[paid].add(&segment.amount)?;
            total = total.add(&segment.amount)?;
            let mut from = at;
            while from < end {
                let to = (from + part).min(end);
                periods.push((from, to));
                from = to;
            }
            at = end;
        }
    }
    Ok(Regime {
        unit,
        periods,
        due,
        total,
        terms,
    })
}
