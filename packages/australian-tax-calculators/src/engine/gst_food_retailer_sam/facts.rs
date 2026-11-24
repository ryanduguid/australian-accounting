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

/// The window a sample must fall in: the first 2 months of trading when
/// `started` is given, otherwise 1 June to 31 July (`july_half`) or 1 December
/// to 31 January before the half it applies to.
pub(super) fn window(period: &Period, july_half: bool, started: Option<i64>) -> (i64, i64) {
    if let Some(started) = started {
        return (started, dates::months_end(started, 2));
    }
    let (y, _, _) = dates::to_civil(period.first);
    if july_half {
        (day(y, 6, 1), day(y, 7, 31))
    } else {
        (day(y, 12, 1), day(y + 1, 1, 31))
    }
}

pub(super) fn sample(value: &Arg, name: &str, days: i64, (low, high): (i64, i64)) -> Result<Sample> {
    let fields = object(value, name)?;
    known_keys(fields, name, "a sample", &["start", "end", "gst_free", "total"])?;
    let date = |key: &str| -> Result<i64> {
        let text = required(fields, name, key)?;
        date_arg(&Arg::Text(text.as_str().unwrap_or("").to_string()), &format!("{name}.{key}"))
    };
    let (start, end) = (date("start")?, date("end")?);
    if end - start + 1 != days {
        return sample_invalid(name, format!("a sample is {days} consecutive days counting both ends"));
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
