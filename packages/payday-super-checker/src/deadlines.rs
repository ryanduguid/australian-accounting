//! Deadline engine: SGAA 1992 s 18C pathways.
//!
//! - USUAL_7BD: s 6(1) usual period, the 7th business day after the QE day
//! - EXTENDED_20BD: s 18C(2) item 1, a first eligible contribution to a fund
//! - OUT_OF_CYCLE: s 18C(2) item 2 and F2026L00784, the usual period of the
//!   actual subsequent non-out-of-cycle QE payment
//! - ITEM4_ALIGNED: s 18C(2) item 4, a later QE day inheriting an evidenced
//!   earlier eligible contribution's later end
//! - SKIP_DB: defined-benefit interests, not lateness tested (s 18A(3))
//!
//! Exceptional-circumstances determinations (item 3) are not modelled.

use crate::calendar::BusinessCalendar;
use crate::csv_io::zero;
use crate::decimal::Decimal;
use crate::py::Date;
use crate::{Failure, Result, py};

pub const REGIME_START: Date = Date {
    year: 2026,
    month: 7,
    day: 1,
};

pub const USUAL_7BD: &str = "USUAL_7BD";
pub const EXTENDED_20BD: &str = "EXTENDED_20BD";
pub const OUT_OF_CYCLE: &str = "OUT_OF_CYCLE";
pub const ITEM4_ALIGNED: &str = "ITEM4_ALIGNED";
pub const SKIP_DB: &str = "SKIP_DB";

/// Plain-English name for each pathway.
pub fn pathway_words(pathway: &str) -> &str {
    match pathway {
        USUAL_7BD => "strict 7-business-day",
        EXTENDED_20BD => "20-business-day",
        OUT_OF_CYCLE => "out-of-cycle",
        ITEM4_ALIGNED => "item 4 aligned",
        other => other,
    }
}

#[derive(Clone, Debug)]
pub struct ContribLine {
    pub employee_id: String,
    pub qe_day: Date,
    pub sg_amount: Decimal,
    pub remitted: Option<Date>,
    pub remitted_amount: Option<Decimal>,
    pub received: Option<Date>,
    pub first_to_fund: bool,
    pub out_of_cycle: bool,
    pub next_standard_qe_day: Option<Date>,
    pub db_interest: bool,
    pub row: i64,
    pub duplicate_note: String,
    pub matched_amount: Option<Decimal>,
    pub join_caveats: Vec<String>,
}

/// The most a receipt date on this row can evidence.
pub fn receipt_amount_cap(line: &ContribLine) -> &Decimal {
    line.matched_amount
        .as_ref()
        .or(line.remitted_amount.as_ref())
        .unwrap_or(&line.sg_amount)
}

/// Whether the row states an amount its fund-receipt date can evidence.
pub fn receipt_amount_evidenced(line: &ContribLine) -> bool {
    line.matched_amount.is_some() || line.remitted_amount.is_some()
}

#[derive(Clone, Debug)]
pub struct Deadline {
    pub due: Option<Date>,
    pub pathway: String,
    pub notes: Vec<String>,
    pub caveats: Vec<String>,
    pub own_due: Option<Date>,
    pub item2_due: Option<Date>,
    pub possible_item4_due: Option<Date>,
}

impl Deadline {
    fn new(due: Option<Date>, pathway: &str) -> Deadline {
        Deadline {
            due,
            pathway: pathway.to_string(),
            notes: Vec::new(),
            caveats: Vec::new(),
            own_due: None,
            item2_due: None,
            possible_item4_due: None,
        }
    }
}

pub fn compute_due(line: &ContribLine, cal: &BusinessCalendar) -> Result<Deadline> {
    if line.qe_day < REGIME_START {
        return Err(Failure::Value(format!(
            "row {}: QE day {} is before 1 Jul 2026: \
             the old quarterly SG law applies to it; this tool covers payday super only",
            line.row,
            line.qe_day.isoformat()
        )));
    }
    if line.db_interest {
        let mut dl = Deadline::new(None, SKIP_DB);
        dl.notes.push(
            "defined-benefit interest: notional contribution is treated as received \
             on the QE day (SGAA s 18A(3)); lateness testing skipped"
                .into(),
        );
        return Ok(dl);
    }
    let mut notes = Vec::new();
    let mut caveats = Vec::new();
    let mut candidates: Vec<(Date, &str, String)> = Vec::new();
    let mut item2 = None;

    if line.out_of_cycle {
        let Some(next) = line.next_standard_qe_day else {
            return Err(Failure::Value(format!(
                "row {}: out_of_cycle=yes requires next_standard_qe_day. \
                 F2026L00784 s 5 requires a subsequent non-out-of-cycle QE payment \
                 on the next day consistent with the established schedule; a final \
                 or termination payment does not qualify without that subsequent \
                 payment. Supply the next standard payday or set out_of_cycle=no",
                line.row
            )));
        };
        if next <= line.qe_day {
            return Err(Failure::Value(format!(
                "row {}: next_standard_qe_day must be after qe_day",
                line.row
            )));
        }
        candidates.push((
            cal.add_business_days(next, 7)?,
            OUT_OF_CYCLE,
            format!(
                "out-of-cycle earnings: deadline is the usual period of the next \
                 schedule-consistent day on which the employer actually made a \
                 subsequent non-out-of-cycle QE payment, {} \
                 (s 18C(2) item 2; F2026L00784 s 5)",
                next.isoformat()
            ),
        ));
    } else if let Some(next) = line.next_standard_qe_day {
        if next <= line.qe_day {
            caveats.push(format!(
                "next standard QE day {} is not after the QE day {} and the out-of-cycle flag \
                 is not set, so the column was ignored. Correct it or set out_of_cycle=yes",
                next.isoformat(),
                line.qe_day.isoformat()
            ));
        } else {
            item2 = Some(cal.add_business_days(next, 7)?);
        }
    }

    if line.first_to_fund {
        candidates.push((
            cal.add_business_days(line.qe_day, 20)?,
            EXTENDED_20BD,
            "first eligible contribution to this fund: extended usual period, \
             20 business days (s 18C(2) item 1)"
                .into(),
        ));
    }
    if candidates.is_empty() {
        candidates.push((
            cal.add_business_days(line.qe_day, 7)?,
            USUAL_7BD,
            String::new(),
        ));
    }
    // `max` keeps the first of equal candidates.
    let mut best = 0;
    for (i, c) in candidates.iter().enumerate() {
        if c.0 > candidates[best].0 {
            best = i;
        }
    }
    let (due, pathway, note) = candidates.swap_remove(best);
    if !note.is_empty() {
        notes.push(note);
    }
    if line.out_of_cycle && line.first_to_fund && line.next_standard_qe_day.is_some() {
        notes.push(
            "both the out-of-cycle and first-contribution rules apply; using the \
             later deadline"
                .into(),
        );
    }
    let mut dl = Deadline::new(Some(due), pathway);
    dl.notes = notes;
    dl.caveats = caveats;
    dl.item2_due = item2;
    Ok(dl)
}

/// Caveat rows that supply a next standard QE day without the flag,
/// written against the final deadline.
pub fn annotate_missing_flag(pairs: &mut [(ContribLine, Deadline)]) {
    for (line, dl) in pairs.iter_mut() {
        let (Some(item2), Some(due), Some(next)) =
            (dl.item2_due, dl.due, line.next_standard_qe_day)
        else {
            continue;
        };
        let words = pathway_words(&dl.pathway).to_string();
        if item2 > due {
            dl.caveats.push(format!(
                "a next standard QE day {} is supplied but the out-of-cycle flag is not set, \
                 so the {words} deadline {} was used. If this payday \
                 is out of cycle, set out_of_cycle=yes and the deadline becomes \
                 {} (s 18C(2) item 2; F2026L00784)",
                next.isoformat(),
                due.isoformat(),
                item2.isoformat()
            ));
        } else {
            dl.caveats.push(format!(
                "a next standard QE day {} is supplied but the out-of-cycle flag is not set. \
                 The {words} deadline {} was used, and setting out_of_cycle=yes \
                 would not change it: the item 2 deadline is {}, which is no later \
                 (s 18C(2) items 1, 2 and 4)",
                next.isoformat(),
                due.isoformat(),
                item2.isoformat()
            ));
        }
    }
}

/// The same calendar date a year earlier; 29 February falls back to 28.
fn twelve_months_before(d: Date) -> Result<Date> {
    let year = i64::from(d.year) - 1;
    d.with_year(year)
        .or_else(|| Date::new(year, 2, 28).ok())
        .ok_or_else(|| Failure::Value(format!("year {year} is out of range")))
}

/// First day of the 12-month pre-payment window in s 18C(1)(c)(ii).
pub fn earliest_prepayment_day(qe_day: Date) -> Result<Date> {
    Ok(twelve_months_before(qe_day.add_days(-1)?)?.add_days(1)?)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Evidence {
    Confirmed,
    Possible,
    Impossible,
}

fn item4_evidence(
    line: &ContribLine,
    confirmed_due: Date,
    possible_due: Date,
    as_at: Option<Date>,
) -> Result<Evidence> {
    if line.sg_amount.le(&zero())? || line.db_interest || receipt_amount_cap(line).le(&zero())? {
        return Ok(Evidence::Impossible);
    }
    let earliest = earliest_prepayment_day(line.qe_day)?;
    if let Some(receipt) = line.received {
        if receipt < earliest {
            return Ok(Evidence::Impossible);
        }
        if as_at.is_none_or(|a| receipt <= a) {
            if receipt < line.qe_day || receipt <= confirmed_due {
                return Ok(Evidence::Confirmed);
            }
            if receipt > possible_due {
                return Ok(Evidence::Impossible);
            }
            return Ok(Evidence::Possible);
        }
        return Ok(if receipt <= possible_due {
            Evidence::Possible
        } else {
            Evidence::Impossible
        });
    }
    if line.remitted.is_some_and(|r| r > possible_due) {
        return Ok(Evidence::Impossible);
    }
    Ok(Evidence::Possible)
}

/// Apply SGAA s 18C(2) item 4 without manufacturing its missing facts.
pub fn apply_item4(pairs: &mut [(ContribLine, Deadline)], as_at: Option<Date>) -> Result<()> {
    // Employees in first-appearance order, each with its pairs' indices.
    let mut grouped: crate::Groups<usize> = crate::Groups::default();
    for (i, (line, dl)) in pairs.iter().enumerate() {
        if dl.due.is_some() {
            grouped.push(line.employee_id.clone(), i);
        }
    }
    let by_employee = grouped.entries;
    let mut folded: crate::Groups<usize> = crate::Groups::default();
    for (k, (id, _)) in by_employee.iter().enumerate() {
        folded.push(py::casefold(id), k);
    }
    let by_casefold = folded.entries;
    for (_, variants) in &by_casefold {
        if variants.len() < 2 {
            continue;
        }
        let mut rows: Vec<i64> = variants
            .iter()
            .flat_map(|&k| by_employee[k].1.iter().map(|&i| pairs[i].0.row))
            .collect();
        rows.sort();
        let listed: Vec<String> = rows.iter().map(i64::to_string).collect();
        let caveat = format!(
            "the employee ids on rows {} differ only by \
             capitalisation and are treated as different people, so no \
             deadline is aligned between them (s 18C(2) item 4). If they \
             are one person, make the ids match",
            listed.join(", ")
        );
        for &k in variants {
            for &i in &by_employee[k].1 {
                pairs[i].1.caveats.push(caveat.clone());
            }
        }
    }

    for (_, indices) in &by_employee {
        let mut items = indices.clone();
        items.sort_by_key(|&i| (pairs[i].0.qe_day, pairs[i].0.row));
        let mut confirmed_latest: Option<Date> = None;
        let mut possible_latest: Option<Date> = None;
        let mut index = 0;
        while index < items.len() {
            let mut end = index;
            while end < items.len() && pairs[items[end]].0.qe_day == pairs[items[index]].0.qe_day {
                end += 1;
            }
            let mut group_confirmed = confirmed_latest;
            let mut group_possible = possible_latest;
            for &i in &items[index..end] {
                let (line, dl) = &mut pairs[i];
                if let (Some(confirmed), Some(due)) = (confirmed_latest, dl.due)
                    && due < confirmed
                {
                    dl.notes.push(format!(
                        "deadline aligned to an evidenced earlier eligible \
                         contribution's latest due day {} (s 18C(2) item 4)",
                        confirmed.isoformat()
                    ));
                    dl.own_due = dl.due;
                    dl.due = Some(confirmed);
                    dl.pathway = ITEM4_ALIGNED.into();
                }
                let Some(due) = dl.due else { continue };
                let mut widest = due;
                if let Some(possible) = possible_latest
                    && possible > widest
                {
                    widest = possible;
                    dl.possible_item4_due = Some(possible);
                }
                match item4_evidence(line, due, widest, as_at)? {
                    Evidence::Confirmed => {
                        group_confirmed = Some(group_confirmed.map_or(due, |g| g.max(due)));
                        group_possible = Some(group_possible.map_or(widest, |g| g.max(widest)));
                    }
                    Evidence::Possible => {
                        group_possible = Some(group_possible.map_or(widest, |g| g.max(widest)));
                    }
                    Evidence::Impossible => {}
                }
            }
            confirmed_latest = group_confirmed;
            possible_latest = group_possible;
            index = end;
        }
    }
    Ok(())
}

/// Attach calendar caveats to the final due date.
pub fn annotate_calendar_risk(pairs: &mut [(ContribLine, Deadline)], cal: &BusinessCalendar) {
    for (line, dl) in pairs.iter_mut() {
        let Some(due) = dl.due else { continue };
        if let Some(horizon) = cal.check_horizon(due) {
            dl.caveats.push(horizon);
        }
        let provisional = cal.provisional_hits(line.qe_day, due);
        if !provisional.is_empty() {
            dl.caveats.push(format!(
                "deadline window contains unconfirmed holiday dates that were not \
                 used to extend the deadline: {}. Confirm \
                 the date against an official whole-of-jurisdiction source and add it \
                 with --holidays-override if it applies",
                provisional.join("; ")
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decimal::Decimal;

    fn day(y: i64, m: i64, d: i64) -> Date {
        Date::new(y, m, d).unwrap()
    }

    fn dec(text: &str) -> Decimal {
        Decimal::parse(text).unwrap()
    }

    fn line() -> ContribLine {
        ContribLine {
            employee_id: "E1".into(),
            qe_day: day(2026, 7, 9),
            sg_amount: dec("600.00"),
            remitted: None,
            remitted_amount: None,
            received: Some(day(2026, 7, 15)),
            first_to_fund: false,
            out_of_cycle: false,
            next_standard_qe_day: None,
            db_interest: false,
            row: 2,
            duplicate_note: String::new(),
            matched_amount: None,
            join_caveats: Vec::new(),
        }
    }

    #[test]
    fn the_missing_flag_does_not_invent_an_absent_next_qe_day() {
        let mut deadline = Deadline::new(Some(day(2026, 7, 20)), USUAL_7BD);
        deadline.item2_due = Some(day(2026, 7, 30));
        let mut pairs = [(line(), deadline)];
        annotate_missing_flag(&mut pairs);
        assert!(pairs[0].1.caveats.is_empty());
    }

    #[test]
    fn the_receipt_amount_cap_is_an_upper_bound_not_evidence() {
        let blank = ContribLine {
            sg_amount: dec("540.00"),
            received: Some(day(2026, 8, 20)),
            ..line()
        };
        assert_eq!(receipt_amount_cap(&blank).to_sci_string(), "540.00");
        assert!(!receipt_amount_evidenced(&blank));

        let stated = ContribLine {
            matched_amount: Some(dec("540.00")),
            ..blank.clone()
        };
        assert!(receipt_amount_evidenced(&stated));

        let part = ContribLine {
            remitted: Some(day(2026, 8, 18)),
            remitted_amount: Some(dec("200.00")),
            ..blank
        };
        assert!(receipt_amount_evidenced(&part));
        assert_eq!(receipt_amount_cap(&part).to_sci_string(), "200.00");
    }

    #[test]
    fn the_prepayment_window_before_a_leap_day_falls_back_to_28_february() {
        // The window ends on 29 February 2028, which has no date a year earlier.
        assert_eq!(
            earliest_prepayment_day(day(2028, 3, 1)).unwrap(),
            day(2027, 3, 1)
        );
    }
}
