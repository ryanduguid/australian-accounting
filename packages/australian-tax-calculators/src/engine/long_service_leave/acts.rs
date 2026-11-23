// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Each general Act's long service leave, as weeks in exact fractions.

use super::super::{Arg, Result, refuse};
use crate::dates;

/// Weeks as a numerator over a positive denominator.
pub(super) type Weeks = (i128, i128);

/// The Acts, by the code the command takes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Place {
    Nsw,
    Vic,
    Qld,
    Wa,
    Sa,
    Tas,
    Act,
    Nt,
}

/// Each Act's code, the years for the full entitlement and for a pro rata
/// payment, and the first day of the text read on 24 November 2026 (the
/// command's dates start on 1 July 2020 in any case).
const PLACES: [(&str, Place, i64, i64, &str); 8] = [
    ("nsw", Place::Nsw, 10, 5, "2021-03-25"),
    ("vic", Place::Vic, 7, 7, "2021-07-01"),
    ("qld", Place::Qld, 10, 7, "2020-06-22"),
    ("wa", Place::Wa, 10, 7, "2022-06-20"),
    ("sa", Place::Sa, 10, 7, "1988-01-01"),
    ("tas", Place::Tas, 10, 7, "2013-07-01"),
    ("act", Place::Act, 7, 5, "2011-09-21"),
    ("nt", Place::Nt, 10, 7, "2004-10-27"),
];

impl Place {
    fn row(self) -> &'static (&'static str, Place, i64, i64, &'static str) {
        PLACES
            .iter()
            .find(|row| row.1 == self)
            .expect("every place")
    }

    fn full(self) -> i64 {
        self.row().2
    }

    fn early(self) -> i64 {
        self.row().3
    }

    /// The first day of the Act's text as read.
    pub(super) fn from(self) -> &'static str {
        self.row().4
    }
}

pub(super) fn place(jurisdiction: &Arg) -> Result<Place> {
    match PLACES.iter().find(|row| Some(row.0) == jurisdiction.text()) {
        Some(row) => Ok(row.1),
        None => refuse(
            "long_service_leave.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
        ),
    }
}

/// Why employment ended.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Reason {
    Resign,
    Pressing,
    NoNotice,
    Retire,
    Redundancy,
    IllnessDismissal,
    Performance,
    Misconduct,
    FixedTerm,
    Death,
}

const REASONS: [(&str, Reason); 10] = [
    ("resign", Reason::Resign),
    ("pressing", Reason::Pressing),
    ("no_notice", Reason::NoNotice),
    ("retire", Reason::Retire),
    ("redundancy", Reason::Redundancy),
    ("illness_dismissal", Reason::IllnessDismissal),
    ("performance", Reason::Performance),
    ("misconduct", Reason::Misconduct),
    ("fixed_term", Reason::FixedTerm),
    ("death", Reason::Death),
];

pub(super) fn reason(value: &Arg) -> Result<Reason> {
    match REASONS.iter().find(|(code, _)| Some(*code) == value.text()) {
        Some((_, reason)) => Ok(*reason),
        None => refuse(
            "long_service_leave.reason_invalid",
            "reason must be resign, pressing, no_notice, retire, redundancy, \
             illness_dismissal, performance, misconduct, fixed_term or death.",
        ),
    }
}

/// Service in completed years, then completed months, then days, and the
/// days in all.
pub(super) struct Service {
    pub(super) years: i64,
    pub(super) months: i64,
    pub(super) days: i64,
    pub(super) total: i64,
}

/// The day `months` months after `start`: the same day of the month or,
/// where the month has none, the 1st of the next.
pub(super) fn anniversary(start: i64, months: i64) -> i64 {
    let day = dates::add_months(start, months);
    if dates::to_civil(day).2 == dates::to_civil(start).2 {
        day
    } else {
        day + 1
    }
}

/// Service from `start` to the day before `until`.
pub(super) fn measure(start: i64, until: i64) -> Service {
    let mut years = 0;
    while anniversary(start, 12 * (years + 1)) <= until {
        years += 1;
    }
    let mut months = 0;
    while anniversary(start, 12 * years + months + 1) <= until {
        months += 1;
    }
    let days = until - anniversary(start, 12 * years + months);
    Service {
        years,
        months,
        days,
        total: until - start,
    }
}

/// The weeks and the provision applied.
pub(super) struct Outcome {
    pub(super) weeks: Weeks,
    pub(super) provision: String,
}

fn outcome(weeks: Weeks, provision: impl Into<String>) -> Result<Outcome> {
    Ok(Outcome {
        weeks,
        provision: provision.into(),
    })
}

/// Years with the part year as months over 12 and days over 365.25, or in
/// Queensland days over 364 (Business Queensland's 52-week year).
fn exact_years(s: &Service, place: Place) -> Weeks {
    let (y, m, d) = (
        i128::from(s.years),
        i128::from(s.months),
        i128::from(s.days),
    );
    if place == Place::Qld {
        (y * 1092 + m * 91 + d * 3, 1092)
    } else {
        (y * 438_300 + m * 36_525 + d * 1200, 438_300)
    }
}

fn whole(years: i64) -> Weeks {
    (i128::from(years), 1)
}

/// 8 2/3 weeks for 10 years (2 months of 4 1/3 weeks), or in Queensland the
/// 8.6667 weeks section 95(2)(a) and (7) give.
fn two_months((y, by): Weeks, place: Place) -> Weeks {
    if place == Place::Qld {
        (y * 86_667, by * 100_000)
    } else {
        (y * 13, by * 15)
    }
}

/// 1.3 weeks a year.
fn thirteen((y, by): Weeks) -> Weeks {
    (y * 13, by * 10)
}

/// 1/60 of a period in days, in weeks.
fn sixtieth(days: i64) -> Weeks {
    (i128::from(days), 420)
}

/// The years in the first 10 and each completed 5 after.
fn blocks(s: &Service) -> i64 {
    10 + 5 * ((s.years - 10) / 5)
}

/// The leave an employee still employed can take.
pub(super) fn available(place: Place, s: &Service) -> Result<Outcome> {
    let full = place.full();
    if s.years < full {
        return outcome((0, 1), format!("Under {full} years: no leave to take yet"));
    }
    match place {
        Place::Vic => outcome(sixtieth(s.total), "1/60 of the continuous employment, s 6"),
        Place::Qld if s.years < 15 => outcome((86_667, 10_000), "8.6667 weeks, s 95(2)(a)"),
        Place::Qld => outcome(
            two_months(exact_years(s, place), place),
            "8.6667 weeks for 10 years and the proportion for the further period, s 95(2)",
        ),
        Place::Sa => outcome(
            thirteen(whole(s.years)),
            "13 weeks for 10 years, 1.3 a completed year after, s 5(1)",
        ),
        Place::Act => outcome(
            two_months(whole(s.years), place),
            "1/5 of a month a completed year, ss 3 and 4",
        ),
        Place::Nt => outcome(
            thirteen(whole(blocks(s))),
            "1.3 weeks a year for 10 years and each 5 after, s 8",
        ),
        Place::Nsw | Place::Wa | Place::Tas => outcome(
            two_months(whole(blocks(s)), place),
            "8 2/3 weeks for 10 years and 4 1/3 for each 5 after",
        ),
    }
}

/// What an Act pays when employment ends, before leave taken.
pub(super) fn on_ending(
    place: Place,
    s: &Service,
    reason: Reason,
    start: i64,
    until: i64,
) -> Result<Outcome> {
    let early = place.early();
    if s.years < early {
        return outcome((0, 1), format!("Under {early} years: nothing is payable"));
    }
    if s.years < place.full() {
        return pro_rata(place, s, reason);
    }
    match (place, reason) {
        (Place::Wa, Reason::Misconduct) => outcome(
            two_months(whole(blocks(s)), place),
            "Leave accrued only, after dismissal for serious misconduct, s 8(2)",
        ),
        (Place::Nt, Reason::Misconduct) => outcome(
            thirteen(whole(blocks(s))),
            "The completed 10 and 5-year periods only, after serious misconduct, s 10(1A)",
        ),
        (Place::Tas, Reason::Death) => {
            let since = until - anniversary(start, 12 * blocks(s));
            let (accrued, more) = (two_months(whole(blocks(s)), place), sixtieth(since));
            outcome(
                (accrued.0 * more.1 + more.0 * accrued.1, accrued.1 * more.1),
                "Leave accrued and 1/60 of the employment since, s 9(1) and (2)(a)",
            )
        }
        _ => after_full(place, s),
    }
}

/// What an Act pays for any other ending after its full qualifying period.
fn after_full(place: Place, s: &Service) -> Result<Outcome> {
    let exact = exact_years(s, place);
    match place {
        Place::Nsw if s.years < 15 => outcome(
            two_months(exact, place),
            "Proportion on 3 months for 15 years, any reason, s 4(2)(a)(ii)",
        ),
        Place::Vic => outcome(
            sixtieth(s.total),
            "1/60 of the continuous employment, any reason, ss 6 and 9",
        ),
        Place::Qld => outcome(
            two_months(exact, place),
            "Proportionate payment on 8.6667 weeks for 10 years, s 95(3)",
        ),
        Place::Tas => outcome(
            two_months(exact, place),
            "Leave accrued and a proportion of 8 2/3 weeks for the years since, s 8(2)(a)",
        ),
        Place::Sa | Place::Nt => outcome(
            thirteen(whole(s.years)),
            "1.3 weeks a completed year, any reason",
        ),
        Place::Nsw | Place::Wa | Place::Act => outcome(
            two_months(whole(s.years), place),
            "Each completed year at 8 2/3 weeks for 10 years, any reason",
        ),
    }
}

/// The reasons each Act pays for before its full qualifying period.
fn qualifies(place: Place, reason: Reason) -> bool {
    use Reason::*;
    match place {
        Place::Vic => false,
        Place::Nsw => matches!(
            reason,
            Redundancy | IllnessDismissal | Performance | Pressing | Death
        ),
        Place::Qld => matches!(
            reason,
            Redundancy | IllnessDismissal | Pressing | FixedTerm | Death
        ),
        Place::Wa => !matches!(reason, Misconduct),
        Place::Sa => !matches!(reason, NoNotice | Misconduct),
        Place::Tas | Place::Act => matches!(
            reason,
            Pressing | Retire | Redundancy | IllnessDismissal | Performance | Death
        ),
        Place::Nt => matches!(
            reason,
            Pressing | Retire | Redundancy | IllnessDismissal | Performance
        ),
    }
}

/// What an Act pays between its pro rata and full qualifying periods.
fn pro_rata(place: Place, s: &Service, reason: Reason) -> Result<Outcome> {
    let full = place.full();
    unsettled(place, reason)?;
    if !qualifies(place, reason) {
        return outcome(
            (0, 1),
            format!("Under {full} years, this ending does not qualify for pro rata leave"),
        );
    }
    match (place, reason) {
        (Place::Sa | Place::Nt, _) => {
            outcome(thirteen(whole(s.years)), "1.3 weeks a completed year")
        }
        (Place::Act, _) => outcome(
            two_months((i128::from(s.years * 12 + s.months), 12), place),
            "Completed years and months on 2 months for 10 years, s 11C(2)",
        ),
        (Place::Tas, Reason::Death) => outcome(
            sixtieth(s.total),
            "1/60 of the continuous employment, s 9(2)(b)",
        ),
        _ => outcome(
            two_months(exact_years(s, place), place),
            format!(
                "Pro rata on {} weeks for 10 years",
                if place == Place::Qld {
                    "8.6667"
                } else {
                    "8 2/3"
                }
            ),
        ),
    }
}

/// Endings an Act does not settle before its full qualifying period.
fn unsettled(place: Place, reason: Reason) -> Result<()> {
    let why = match (place, reason) {
        (Place::Nsw | Place::Tas | Place::Act | Place::Nt, Reason::FixedTerm) => {
            "The Act pays pro rata leave before the full period when the employer ends the \
             employment and does not say whether a fixed term ending is that."
        }
        (Place::Nt, Reason::Death) => {
            "Under 10 years the NT Act pays on death what the employee would have been paid for \
             leaving that day for a reason in s 10(2) (s 10(3)): give that reason instead."
        }
        _ => return Ok(()),
    };
    refuse("long_service_leave.unsettled", why)
}
