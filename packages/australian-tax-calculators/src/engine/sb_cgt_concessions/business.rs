// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The tests of `sb_cgt_concessions` that read turnovers and days: the CGT
//! small business entity test (ITAA 1997 s 152-10(1AA) and s 328-110) and the
//! active asset test's period (s 152-35), compilation 264, read on 19 November
//! 2026.

use super::super::{Arg, Result, date_arg, flag, money, refuse};
use super::{SbCgtConcessionsArgs, TURNOVER, conflict, invalid};
use crate::dates;
use crate::decimal::Decimal;

/// The shortest and longest 7.5 years in days: 7 years with 1 leap day and
/// the shortest 6 months (2,556 + 181), and 7 years with 2 leap days and the
/// longest 6 months (2,557 + 184). A total of active days between them may
/// or may not be 7.5 years, depending on the periods.
const SEVEN_AND_A_HALF: (i64, i64) = (2737, 2741);

pub(super) fn choice<'a>(value: &'a Arg, options: &[&'a str], message: &str) -> Result<&'a str> {
    match value.text() {
        Some(text) if options.contains(&text) => Ok(text),
        _ => invalid(message),
    }
}

/// A turnover, or `null` for a year no business was carried on.
fn turnover(value: &Arg) -> Result<Option<Decimal>> {
    match value {
        Arg::Other(none) if none == "None" => Ok(None),
        other => money(other).map(Some),
    }
}

/// A date, or `null`.
pub(super) fn optional_date(value: &Arg, name: &str) -> Result<Option<i64>> {
    match value {
        Arg::Other(none) if none == "None" => Ok(None),
        other => date_arg(other, name).map(Some),
    }
}

/// The same day `years` later or earlier; 29 February becomes 1 March.
pub(super) fn anniversary(day: i64, years: i64) -> i64 {
    let moved = dates::add_months(day, 12 * years);
    if dates::to_civil(moved).2 == dates::to_civil(day).2 {
        moved
    } else {
        moved + 1
    }
}

/// Whether the business entity is a CGT small business entity, and by
/// which rule of s 328-110.
pub(super) struct SmallBusiness {
    pub(super) met: bool,
    pub(super) rule: Option<&'static str>,
}

pub(super) fn small_business(
    args: &SbCgtConcessionsArgs,
    stopped: Option<&Arg>,
) -> Result<SmallBusiness> {
    let business = choice(
        args.business,
        &["carried_on", "winding_up", "not_carried_on"],
        "business must be carried_on, winding_up or not_carried_on.",
    )?;
    let carried = match (business, stopped) {
        ("carried_on", _) => true,
        // s 328-110(5): a business being wound up counts as carried on if
        // the entity was a small business entity in the year it stopped.
        ("winding_up", Some(stopped)) => flag(stopped, "small_business_when_stopped")?,
        _ => false,
    };
    let previous = turnover(args.previous_year_turnover)?;
    let before = turnover(args.year_before_turnover)?;
    let likely = money(args.likely_turnover)?;
    let at_end = money(args.year_turnover)?;
    let limit = Decimal::parse(TURNOVER)?;
    let under = |amount: &Decimal| amount.lt(&limit);
    let previous_under = previous.as_ref().map(under).transpose()?;
    let barred = match (&previous, &before) {
        (Some(p), Some(b)) => !under(p)? && !under(b)?,
        _ => false,
    };
    let rule = if !carried {
        None
    } else if previous_under == Some(true) {
        Some("s 328-110(1)(b)(i)")
    } else if under(&likely)? && !barred {
        Some("s 328-110(1)(b)(ii)")
    } else if under(&at_end)? {
        Some("s 328-110(4)")
    } else {
        None
    };
    Ok(SmallBusiness {
        met: rule.is_some(),
        rule,
    })
}

/// The test period of s 152-35(2) and whether the asset was active for
/// long enough in it. Both ends are counted.
pub(super) struct Active {
    pub(super) met: bool,
    pub(super) from: i64,
    pub(super) to: i64,
    pub(super) days: i64,
    pub(super) active: i64,
    pub(super) over_15: bool,
}

pub(super) fn active_asset(
    args: &SbCgtConcessionsArgs,
    acquired: i64,
    event: i64,
    ceased: Option<(i64, &Arg)>,
) -> Result<Active> {
    let mut to = event;
    if let Some((ceased, longer)) = ceased {
        if ceased < acquired || ceased > event {
            return conflict("business_ceased must be from acquired to event_date.");
        }
        // s 152-35(2)(b)(ii): a business that ceased in the 12 months before
        // the event, or a longer period the Commissioner allows.
        if anniversary(ceased, 1) >= event || flag(longer, "longer_period_allowed")? {
            to = ceased;
        }
    }
    let days = to - acquired + 1;
    let active = match args.active_days.int() {
        Some(Some(n)) if (0..=days).contains(&n) => n,
        _ => {
            return conflict(format!(
                "active_days must be a whole number of days from 0 to the {days} days of the test period."
            ));
        }
    };
    let over_15 = anniversary(acquired, 15) < event;
    let (shortest, longest) = SEVEN_AND_A_HALF;
    if over_15 && (shortest..longest).contains(&active) {
        return refuse(
            "sb_cgt_concessions.active_period_unclear",
            "A total of 7.5 years is 2,737 to 2,741 days, depending on the leap days and months it spans: count the active periods in years and months to see whether they reach 7.5 years (s 152-35(1)(b)).",
        );
    }
    let met = if over_15 {
        active >= longest
    } else {
        2 * active >= days
    };
    Ok(Active {
        met,
        from: acquired,
        to,
        days,
        active,
        over_15,
    })
}

#[cfg(test)]
mod tests {
    use super::anniversary;
    use crate::dates;

    fn day(text: &str) -> i64 {
        dates::parse(text).expect("a date")
    }

    #[test]
    fn an_anniversary_of_29_february_is_1_march() {
        assert_eq!(anniversary(day("2012-02-29"), 15), day("2027-03-01"));
        assert_eq!(anniversary(day("2024-02-29"), -1), day("2023-03-01"));
        assert_eq!(anniversary(day("2011-03-15"), 15), day("2026-03-15"));
        assert_eq!(anniversary(day("2026-02-28"), 2), day("2028-02-28"));
    }
}
