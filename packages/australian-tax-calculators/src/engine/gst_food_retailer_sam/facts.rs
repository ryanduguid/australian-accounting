// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The stated facts `gst-food-retailer-sam` reads: the inputs each method
//! takes, the eligibility conditions, the tax period and the sample periods.

use std::cell::RefCell;

use serde_json::{Map, Value};

use super::super::business_capital::year_first;
use super::super::stated::{amount, known_keys, required};
use super::super::{Arg, Result, d, date_arg, refuse};
use crate::dates;
use crate::decimal::Decimal;

/// The optional inputs given, read by name; any left unread is refused.
pub(super) struct Given<'a> {
    pub(super) items: &'a [(&'static str, Arg)],
    pub(super) method: &'static str,
    pub(super) read: RefCell<Vec<&'static str>>,
}

impl<'a> Given<'a> {
    pub(super) fn may(&self, key: &'static str) -> Option<&'a Arg> {
        self.read.borrow_mut().push(key);
        self.items.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    pub(super) fn need(&self, key: &'static str) -> Result<&'a Arg> {
        match self.may(key) {
            Some(value) => Ok(value),
            None => refuse(
                "input.missing_field",
                format!("{key} is required for method {}.", self.method),
            ),
        }
    }

    /// Refuses an input the method did not read.
    pub(super) fn done(&self) -> Result<()> {
        let read = self.read.borrow();
        match self.items.iter().find(|(k, _)| !read.contains(k)) {
            Some((key, _)) => refuse(
                "input.field_unknown",
                format!("{key} is not an input of method {} as given.", self.method),
            ),
            None => Ok(()),
        }
    }
}

/// What a condition asks of its stated fact.
#[derive(Clone, Copy)]
pub(super) enum Want {
    True,
    False,
    AtMostTwoMillion,
}

/// One eligibility condition: its key, what it must be and its authority.
pub(super) type Condition = (&'static str, Want, &'static str);

fn object<'v>(value: &'v Arg, name: &str) -> Result<&'v Map<String, Value>> {
    match value {
        Arg::Json(Value::Object(fields)) => Ok(fields),
        _ => refuse(
            "input.not_object",
            format!("{name} must be an object of stated facts."),
        ),
    }
}

fn met(fields: &Map<String, Value>, (key, want, _): &Condition) -> Result<bool> {
    let wanted = match want {
        Want::True => true,
        Want::False => false,
        Want::AtMostTwoMillion => {
            return Ok(amount(fields, "eligibility", key)?.le(&d("2000000"))?);
        }
    };
    match required(fields, "eligibility", key)? {
        Value::Bool(b) => Ok(*b == wanted),
        _ => refuse(
            "input.flag_invalid",
            format!("eligibility: {key} must be a boolean."),
        ),
    }
}

/// Refuses unless every condition is met, naming each that is not.
pub(super) fn eligible(value: &Arg, conditions: &[Condition]) -> Result<()> {
    let fields = object(value, "eligibility")?;
    let keys: Vec<&str> = conditions.iter().map(|c| c.0).collect();
    known_keys(fields, "eligibility", "this method", &keys)?;
    let mut failing = Vec::new();
    for condition in conditions {
        if !met(fields, condition)? {
            failing.push(format!("{} ({})", condition.0, condition.2));
        }
    }
    if failing.is_empty() {
        return Ok(());
    }
    refuse(
        "gst_food_retailer_sam.not_eligible",
        format!(
            "The method is not available: not met: {}.",
            failing.join("; ")
        ),
    )
}

/// A tax period inside the financial year `year`, as day numbers.
pub(super) struct Period {
    pub(super) start: i64,
    pub(super) end: i64,
    /// 1 July of the financial year.
    pub(super) first: i64,
    /// 1 January within the financial year.
    pub(super) january: i64,
}

impl Period {
    /// Wholly in July to December (`Some(true)`), wholly in January to June
    /// (`Some(false)`) or across 31 December (`None`).
    pub(super) fn half(&self) -> Option<bool> {
        if self.end < self.january {
            Some(true)
        } else if self.start >= self.january {
            Some(false)
        } else {
            None
        }
    }
}

fn day(y: i64, m: u32, dd: u32) -> i64 {
    dates::parse(&format!("{y:04}-{m:02}-{dd:02}")).expect("a date")
}

pub(super) fn period(start: &Arg, end: &Arg, year: &str, commences: &str) -> Result<Period> {
    let y = year_first(year).expect("a supported period is a financial year");
    let (first, last) = (day(y, 7, 1), day(y + 1, 6, 30));
    let (start, end) = (
        date_arg(start, "tax_period_start")?,
        date_arg(end, "tax_period_end")?,
    );
    if !(first <= start && start <= end && end <= last) {
        return refuse(
            "gst_food_retailer_sam.tax_period_invalid",
            format!(
                "tax_period_start to tax_period_end must be a period within {year} ({} to {}).",
                dates::format(first),
                dates::format(last)
            ),
        );
    }
    let from = dates::parse(commences).expect("a date");
    if start < from {
        return refuse(
            "gst_food_retailer_sam.method_not_in_force",
            format!(
                "The method's determination commenced on {commences}; a choice takes effect from the start of a tax period (GST Act s 123-10(3)), so a tax period starting before then is not worked."
            ),
        );
    }
    let january = day(y + 1, 1, 1);
    Ok(Period {
        start,
        end,
        first,
        january,
    })
}

/// A sample period's GST-free and total figures, once its dates fall in the
/// window the method allows.
pub(super) struct Sample {
    pub(super) gst_free: Decimal,
    pub(super) total: Decimal,
}

fn sample_invalid<T>(name: &str, why: impl std::fmt::Display) -> Result<T> {
    refuse(
        "gst_food_retailer_sam.sample_invalid",
        format!("{name}: {why}"),
    )
}

/// The 2017 window a sample must fall in: 1 June to 31 July (`july_half`) or
/// 1 December to 31 January before the half it applies to, never before
/// trading started, or the first 2 months of trading in July to December
/// (F2017L01274 ss 6(4) and 7(3)).
pub(super) fn window(period: &Period, july_half: bool, started: Option<i64>) -> Result<(i64, i64)> {
    let (y, _, _) = dates::to_civil(period.first);
    let ordinary = if july_half {
        (day(y, 6, 1), day(y, 7, 31))
    } else {
        (day(y, 12, 1), day(y + 1, 1, 31))
    };
    let Some(started) = started else {
        return Ok(ordinary);
    };
    let half_start = if july_half {
        period.first
    } else {
        period.january
    };
    let first_months = (started, dates::months_end(started, 2));
    if started < half_start {
        return Ok((ordinary.0.max(started), ordinary.1));
    }
    if july_half {
        Ok(first_months)
    } else {
        refuse(
            "gst_food_retailer_sam.new_business_unsettled",
            "F2017L01274 ss 6(4) and 7(3) replace only the first sample period (1 June to 31 July, for July to December) for a business that starts during the year; a business that starts trading in January to June is not worked.",
        )
    }
}

pub(super) fn sample(
    value: &Arg,
    name: &str,
    days: i64,
    (low, high): (i64, i64),
) -> Result<Sample> {
    let fields = object(value, name)?;
    known_keys(
        fields,
        name,
        "a sample",
        &["start", "end", "gst_free", "total"],
    )?;
    let date = |key: &str| -> Result<i64> {
        let text = required(fields, name, key)?;
        date_arg(
            &Arg::Text(text.as_str().unwrap_or("").to_string()),
            &format!("{name}.{key}"),
        )
    };
    let (start, end) = (date("start")?, date("end")?);
    if end - start + 1 != days {
        return sample_invalid(
            name,
            format!("a sample is {days} consecutive days counting both ends"),
        );
    }
    if start < low || end > high {
        let range = format!("{} to {}", dates::format(low), dates::format(high));
        return sample_invalid(name, format!("the sample must fall within {range}"));
    }
    let gst_free = amount(fields, name, "gst_free")?;
    let total = amount(fields, name, "total")?;
    if total.is_zero() || gst_free.gt(&total)? {
        return sample_invalid(name, "total must be more than nil and at least gst_free");
    }
    Ok(Sample { gst_free, total })
}

fn method_started_window(
    period: &Period,
    ordinary: (i64, i64),
    first_eligible: i64,
    is_first: bool,
    started: Option<i64>,
    m: i64,
) -> Result<(i64, i64)> {
    if m + 27 <= ordinary.1 {
        return Ok((m, ordinary.1));
    }
    if m >= period.start {
        if let Some(t) = started
            && t >= period.start
        {
            return Ok((t, dates::months_end(t, 2).max(period.end)));
        }
        let low = started.map_or(period.start, |t| period.start.max(t));
        return Ok((low, period.end));
    }
    if !is_first && m >= first_eligible {
        let low = started.map_or(first_eligible, |t| first_eligible.max(t));
        let high = if let Some(t) = started
            && t >= first_eligible
            && t + 27 > ordinary.1
        {
            dates::months_end(t, 2).max(period.start - 1)
        } else {
            period.start - 1
        };
        return Ok((low, high));
    }
    refuse(
        "gst_food_retailer_sam.new_business_unsettled",
        "F2026L01275 s 7(d) gives a 4-week sample within the tax period only when the retailer starts using the method during an eligible tax period and paragraphs (a) and (b) cannot apply.",
    )
}

fn trading_started_window(
    period: &Period,
    ordinary: (i64, i64),
    first_eligible: i64,
    is_first: bool,
    t: i64,
) -> Result<(i64, i64)> {
    if t + 27 <= ordinary.1 {
        return Ok((t, ordinary.1));
    }
    if t >= period.start || (!is_first && t >= first_eligible) {
        return Ok((t, dates::months_end(t, 2)));
    }
    refuse(
        "gst_food_retailer_sam.new_business_unsettled",
        "F2026L01275 s 7(c) gives the first 2 months of trading only when the business starts trading during an eligible tax period and paragraphs (a) and (b) cannot apply.",
    )
}

fn half_bounds(period: &Period, july_half: bool) -> ((i64, i64), i64, i64) {
    let (y, _, _) = dates::to_civil(period.first);
    let ordinary = if july_half {
        (day(y, 6, 1), day(y, 7, 31))
    } else {
        (day(y, 12, 1), day(y + 1, 1, 31))
    };
    let half_start = if july_half {
        period.first
    } else {
        period.january
    };
    let commences = dates::parse("2026-09-24").expect("a date");
    let first_eligible = if half_start < commences {
        dates::parse("2026-10-01").expect("a date")
    } else {
        half_start
    };
    (ordinary, half_start, first_eligible)
}

/// Whether `period` is the first tax period in its 6-month half where
/// `restaurant_purchases_snapshot` (F2026L01275 s 7) establishes its
/// GST-free trading stock proportion, and the applicable 4-week sample window.
pub(super) fn restaurant_plan(
    period: &Period,
    july_half: bool,
    started: Option<i64>,
    method_started: Option<i64>,
) -> Result<(bool, (i64, i64))> {
    if let (Some(t), Some(m)) = (started, method_started)
        && m < t
    {
        return refuse(
            "gst_food_retailer_sam.sample_invalid",
            "method_started cannot be before trading_started.",
        );
    }
    let (ordinary, half_start, first_eligible) = half_bounds(period, july_half);
    let is_first = period.start <= first_eligible
        || started.is_some_and(|d| d >= period.start)
        || method_started.is_some_and(|d| d >= period.start);
    let win = if let Some(m) = method_started
        && m >= half_start
    {
        method_started_window(period, ordinary, first_eligible, is_first, started, m)?
    } else if let Some(t) = started
        && t >= half_start
    {
        trading_started_window(period, ordinary, first_eligible, is_first, t)?
    } else {
        let lower = ordinary
            .0
            .max(started.unwrap_or(ordinary.0))
            .max(method_started.unwrap_or(ordinary.0));
        (lower, ordinary.1)
    };
    Ok((is_first, win))
}

/// The day trading started, when the business started in the year: it must
/// fall in the financial year and on or before the tax period's end.
pub(super) fn started(given: Option<&Arg>, period: &Period) -> Result<Option<i64>> {
    let Some(value) = given else {
        return Ok(None);
    };
    let day = date_arg(value, "trading_started")?;
    if day < period.first || day > period.end {
        return refuse(
            "gst_food_retailer_sam.sample_invalid",
            "trading_started must fall in the financial year, on or before tax_period_end.",
        );
    }
    Ok(Some(day))
}

/// The day the retailer started using `restaurant_purchases_snapshot` in the
/// year (F2026L01275 s 7(d)): from 24 September 2026 in the year, up to
/// `tax_period_end`.
pub(super) fn method_started(given: Option<&Arg>, period: &Period) -> Result<Option<i64>> {
    let Some(value) = given else {
        return Ok(None);
    };
    let day = date_arg(value, "method_started")?;
    let commences = dates::parse("2026-09-24").expect("a date");
    if day < period.first || day < commences || day > period.end {
        return refuse(
            "gst_food_retailer_sam.sample_invalid",
            "method_started must fall from 2026-09-24 within the financial year, on or before tax_period_end.",
        );
    }
    Ok(Some(day))
}
