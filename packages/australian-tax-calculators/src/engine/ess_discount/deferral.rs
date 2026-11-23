// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The deferred taxing point of an interest Subdivision 83A-C applies to
//! (ss 83A-115 and 83A-120) and the amount s 83A-110 includes then.

use serde_json::{Value, json};

use super::input::{Held, Kind, Reader};
use super::{shown, valuation};
use crate::dates;
use crate::decimal::Decimal;
use crate::engine::{Result, int, py_max, refuse};

/// Act No. 8, 2022 Sch 10 item 9: cessation of employment is a taxing point
/// only where the taxing point occurs before this day.
pub(super) const CESSATION_ENDS: &str = "2022-07-01";

/// The dated facts behind the taxing point.
struct Facts {
    lifted: Option<i64>,
    exercised: Option<i64>,
    share_lifted: Option<i64>,
    employment_ended: Option<i64>,
    disposed: Option<i64>,
}

fn facts(r: &mut Reader<'_>, kind: Kind, acquired: i64, last: Option<i64>) -> Result<Facts> {
    let lifted = r.day("restrictions_lifted", true)?;
    let (exercised, share_lifted) = match kind {
        Kind::Right => (
            r.day("exercised", true)?,
            r.day("share_restrictions_lifted", true)?,
        ),
        Kind::Share => (None, None),
    };
    let employment_ended = if acquired < day(CESSATION_ENDS) {
        if !r.has("employment_ended") {
            return refuse(
                "ess_discount.employment_end_not_stated",
                format!(
                    "{}: employment_ended (a date, or null while still employed) is required for an interest acquired before {CESSATION_ENDS}: ending employment before then was a taxing point (former ss 83A-115(5) and 83A-120(5)).",
                    r.name
                ),
            );
        }
        r.day("employment_ended", true)?
    } else {
        None
    };
    let disposed = r.day("disposed", true)?;
    let out = Facts {
        lifted,
        exercised,
        share_lifted,
        employment_ended,
        disposed,
    };
    consistent(r, &out, acquired, last)?;
    Ok(out)
}

/// The dates in order: none before acquisition, a share's restrictions
/// lifting only after an exercise, an exercise by the last exercise day and
/// no disposal of a right before it is exercised.
fn consistent(r: &Reader<'_>, f: &Facts, acquired: i64, last: Option<i64>) -> Result<()> {
    let early = [
        f.lifted,
        f.exercised,
        f.share_lifted,
        f.employment_ended,
        f.disposed,
    ]
    .iter()
    .flatten()
    .any(|d| *d < acquired);
    let unexercised = f.share_lifted.is_some() && f.exercised.is_none();
    let before_exercise = matches!((f.exercised, f.share_lifted), (Some(e), Some(s)) if s < e);
    let expired = matches!((f.exercised, last), (Some(e), Some(l)) if e > l);
    let disposed_first = matches!((f.disposed, f.exercised), (Some(d), Some(e)) if d < e);
    if early || unexercised || before_exercise || expired || disposed_first {
        return refuse(
            "ess_discount.dates_inconsistent",
            format!(
                "{}: every date must be on or after acquired, share_restrictions_lifted needs an exercise on or before it, and a right is exercised by its last_exercise_day and before any disposal.",
                r.name
            ),
        );
    }
    Ok(())
}

fn day(text: &str) -> i64 {
    dates::parse(text).expect("date")
}

/// The earliest time s 83A-115(2) or s 83A-120(2) gives, and its rule, with
/// or without the former cessation of employment point.
fn earliest(kind: Kind, acquired: i64, f: &Facts, cessation: bool) -> (i64, &'static str) {
    let share = kind == Kind::Share;
    let fifteen = dates::add_months(acquired, 180) - 1;
    let mut points = vec![(
        fifteen,
        if share {
            "s 83A-115(6)"
        } else {
            "s 83A-120(6)"
        },
    )];
    match (share, f.lifted) {
        (true, Some(l)) => points.push((l, "s 83A-115(4)")),
        (false, Some(l)) if f.exercised.is_none_or(|e| l < e) => points.push((l, "s 83A-120(4)")),
        _ => {}
    }
    if let (Some(e), Some(s)) = (f.exercised, f.share_lifted) {
        points.push((e.max(s), "s 83A-120(7)"));
    }
    if let Some(e) = f
        .employment_ended
        .filter(|e| cessation && *e < day(CESSATION_ENDS))
    {
        points.push((
            e,
            if share {
                "former s 83A-115(5)"
            } else {
                "former s 83A-120(5)"
            },
        ));
    }
    points
        .into_iter()
        .min_by_key(|(d, _)| *d)
        .expect("15 years")
}

/// The taxing point after the 30-day rule (ss 83A-115(3), 83A-120(3)).
/// Act No. 8, 2022 Sch 10 item 9 applies the repeal of the cessation point
/// where the taxing point, after that rule, is on or after 1 July 2022, so
/// such a point is worked out again without it.
fn taxing_point(
    r: &Reader<'_>,
    kind: Kind,
    acquired: i64,
    f: &Facts,
) -> Result<(i64, &'static str)> {
    let point = moved(r, kind, f, earliest(kind, acquired, f, true))?;
    if point.0 < day(CESSATION_ENDS) {
        return Ok(point);
    }
    moved(r, kind, f, earliest(kind, acquired, f, false))
}

/// A point moved to a disposal within 30 days after it.
fn moved(
    r: &Reader<'_>,
    kind: Kind,
    f: &Facts,
    (base, rule): (i64, &'static str),
) -> Result<(i64, &'static str)> {
    match f.disposed {
        Some(d) if d < base => refuse(
            "ess_discount.dates_inconsistent",
            format!(
                "{}: disposed is before the taxing point {} ({rule}); the worksheet does not take a disposal before it.",
                r.name,
                dates::format(base)
            ),
        ),
        Some(d) if d - base <= 30 => Ok((
            d,
            if kind == Kind::Share {
                "s 83A-115(3)"
            } else {
                "s 83A-120(3)"
            },
        )),
        _ => Ok((base, rule)),
    }
}

/// The deferred interest's taxing point, the amount s 83A-110(1) includes
/// then (nil where the cost base is more; `None` without the value) and its
/// detail.
pub(super) fn deferred(
    held: &mut Held<'_>,
    kind: Kind,
    acquired: i64,
) -> Result<(i64, Option<Decimal>, Value)> {
    let last = held.right.as_ref().map(|t| t.last_exercise_day);
    let f = facts(&mut held.reader, kind, acquired, last)?;
    let (point, rule) = taxing_point(&held.reader, kind, acquired, &f)?;
    let cost_base = held.reader.amount("cost_base")?;
    // The regulations value a right not yet exercised, not one disposed of
    // under the 30-day rule (s 83A-315.01(2)).
    let tables =
        kind == Kind::Right && !rule.ends_with("(3)") && f.exercised.is_none_or(|e| e > point);
    let (value, valuation) = value_at(held, acquired, point, tables)?;
    let amount = match &value {
        Some(v) => Some(py_max(v.sub(&cost_base)?, int(0))?),
        None => None,
    };
    let detail = json!({
        "deferred_taxing_point": dates::format(point),
        "taxing_point_rule": rule,
        "cost_base": shown(&cost_base)?,
        "value_at_taxing_point": value.as_ref().map(shown).transpose()?,
        "valuation_at_taxing_point": valuation,
    });
    Ok((point, amount, detail))
}

/// `taxing_point_value`: `null`, or the value on the taxing point as
/// `market_value`, or for a right the regulations allow, `share_market_value`.
fn value_at(
    held: &mut Held<'_>,
    acquired: i64,
    point: i64,
    tables: bool,
) -> Result<(Option<Decimal>, Value)> {
    let Some(mut v) = held.reader.object("taxing_point_value")? else {
        return Ok((None, Value::Null));
    };
    let on = v.day("date", false)?.expect("not nullable");
    if on != point {
        return refuse(
            "ess_discount.taxing_point_value",
            format!(
                "{}: the value is stated on {}, but the deferred taxing point is {}.",
                held.reader.name,
                dates::format(on),
                dates::format(point)
            ),
        );
    }
    let out = match (held.right.as_ref(), v.has("share_market_value")) {
        (Some(terms), true) if tables => {
            let share = v.amount("share_market_value")?;
            let (value, detail) = valuation::right_value(
                &share,
                &terms.exercise_price,
                point,
                terms.last_exercise_day,
                acquired,
            )?;
            (Some(value), detail)
        }
        (_, true) => {
            return refuse(
                "ess_discount.valuation_unavailable",
                format!(
                    "{}: the regulations value only a right not yet exercised, and not on a disposal (s 83A-315.01(2)): give market_value.",
                    held.reader.name
                ),
            );
        }
        _ => (Some(v.amount("market_value")?), Value::Null),
    };
    v.finish("taxing_point_value")?;
    Ok(out)
}
