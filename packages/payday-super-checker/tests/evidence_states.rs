// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Every evidence state reaches exactly one answer: one verdict or a refusal.
//!
//! The rules are restated here from the README's row-verdict and return-event
//! tables, apart from `src/assess.rs`, and every combination of the listed facts
//! is walked, so a changed branch in the engine disagrees with a table for some
//! state. The walk counts its states and needs every answer to occur, so it
//! cannot pass by reaching none. All facts are fabricated.
//!
//! The walk stops at one line in a run, each with the transition allocation
//! confirmed, and it asserts the verdict alone. Item 4 alignment across lines,
//! the transition path and `horizon_verdicts` stay with `tests/parity.rs`.

use std::collections::BTreeSet;

use payday_super_checker::api;
use payday_super_checker::assess::{AT_RISK, LATE, ON_TIME, SKIPPED, UNKNOWN, UNPAID, VERDICTS};
use payday_super_checker::py::Date;
use serde_json::{Value, json};

const SG: &str = "120.00";
const PART: &str = "40.00";
const REST: &str = "80.00";
const REFUSED: &str = "REFUSED";

fn date(text: &str) -> Date {
    Date::fromisoformat(text).unwrap()
}

fn after(day: Date, days: i64) -> Date {
    day.add_days(days).unwrap()
}

/// A payday and its deadlines, counted by hand in business days.
#[derive(Clone, Copy, Debug)]
struct Payday {
    qe: &'static str,
    usual: &'static str,
    first: &'static str,
    past_calendar: bool,
}

const PAYDAYS: [Payday; 3] = [
    // No holiday in the way of 7 or 20 business days.
    Payday {
        qe: "2026-08-06",
        usual: "2026-08-17",
        first: "2026-09-03",
        past_calendar: false,
    },
    // 25 and 28 December stop the count, then 1 January.
    Payday {
        qe: "2026-12-17",
        usual: "2026-12-30",
        first: "2027-01-19",
        past_calendar: false,
    },
    // Due after 31 August 2027, the last day the calendar is complete.
    Payday {
        qe: "2027-08-26",
        usual: "2027-09-06",
        first: "2027-09-23",
        past_calendar: true,
    },
];

impl Payday {
    fn qe_day(&self) -> Date {
        date(self.qe)
    }

    fn due(&self, first: bool) -> Date {
        date(if first { self.first } else { self.usual })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Class {
    Tested,
    ReturnedSomehow,
    DefinedBenefit,
}

/// One row's evidence, as the columns hold it.
#[derive(Clone, Copy, Debug)]
struct Row {
    payday: Payday,
    first: bool,
    class: Class,
    received: Option<Date>,
    remitted: Option<Date>,
    matched: Option<&'static str>,
    sent: Option<&'static str>,
    as_at: Date,
}

impl Row {
    fn due(&self) -> Date {
        self.payday.due(self.first)
    }

    fn json(&self) -> Value {
        json!({
            "employee_id": "E1",
            "qe_day": self.payday.qe,
            "sg_amount": SG,
            "first_to_fund": self.first,
            "db_interest": self.class == Class::DefinedBenefit,
            "returned_by_fund": self.class == Class::ReturnedSomehow,
            "received": self.received.map(|d| d.isoformat()),
            "remitted": self.remitted.map(|d| d.isoformat()),
            "matched_amount": self.matched,
            "remitted_amount": self.sent,
        })
    }

    fn refused(&self) -> bool {
        self.sent.is_some() && self.remitted.is_none()
            || self.matched == Some(PART) && self.remitted.is_some() && self.sent.is_none()
            || self.matched == Some(PART) && self.sent == Some(SG)
            || matches!((self.received, self.remitted), (Some(r), Some(s)) if r < s)
    }

    /// The README's table of row verdicts.
    fn expected(&self) -> &'static str {
        if self.refused() {
            return REFUSED;
        }
        match self.class {
            Class::DefinedBenefit => SKIPPED,
            Class::ReturnedSomehow => UNKNOWN,
            Class::Tested => match self.received.filter(|r| *r <= self.as_at) {
                Some(received) => self.with_receipt(received),
                None => self.without_receipt(),
            },
        }
    }

    fn with_receipt(&self, received: Date) -> &'static str {
        let (due, past) = (self.due(), self.payday.past_calendar);
        // A day before the 12 months s 18C(1)(c)(ii) lets a pre-payment reach back.
        if received < after(self.payday.qe_day(), -365) {
            return if past || due >= self.as_at {
                UNKNOWN
            } else {
                LATE
            };
        }
        if received > due {
            return if past { UNKNOWN } else { LATE };
        }
        match self.matched.or(self.sent) {
            None => UNKNOWN,
            Some(SG) => ON_TIME,
            Some(_) if !past && due < self.as_at => UNPAID,
            Some(_) => UNKNOWN,
        }
    }

    fn without_receipt(&self) -> &'static str {
        let (due, past) = (self.due(), self.payday.past_calendar);
        match self.remitted.filter(|s| *s <= self.as_at) {
            Some(sent) if self.sent != Some(PART) => {
                if sent <= due {
                    AT_RISK
                } else if past {
                    UNKNOWN
                } else {
                    LATE
                }
            }
            _ if !past && due < self.as_at => UNPAID,
            _ => UNKNOWN,
        }
    }
}

/// The engine's answer for one line: its single verdict, or a refusal.
fn answer(line: Value, as_at: Date) -> String {
    let input = json!({
        "lines": [line],
        "as_at": as_at.isoformat(),
        "allow_stale_gic": true,
        "transition_allocation_confirmed": true,
    });
    match api::evaluate("assess", &input) {
        Err(_) => REFUSED.to_string(),
        Ok(results) => {
            let results = results.as_array().unwrap();
            assert_eq!(results.len(), 1, "{input}");
            let verdict = results[0]["verdict"].as_str().unwrap();
            assert!(VERDICTS.contains(&verdict), "{input}");
            verdict.to_string()
        }
    }
}

fn rows() -> Vec<Row> {
    let mut rows = Vec::new();
    for payday in PAYDAYS {
        for first in [false, true] {
            let (qe, due) = (payday.qe_day(), payday.due(first));
            for class in [Class::Tested, Class::ReturnedSomehow, Class::DefinedBenefit] {
                for received in [
                    None,
                    Some(after(qe, -366)),
                    Some(after(qe, -365)),
                    Some(qe),
                    Some(due),
                    Some(after(due, 1)),
                ] {
                    for remitted in [None, Some(qe), Some(due), Some(after(due, 1))] {
                        for matched in [None, Some(PART), Some(SG)] {
                            for sent in [None, Some(PART), Some(SG)] {
                                for as_at in [after(due, -1), due, after(due, 10)] {
                                    rows.push(Row {
                                        payday,
                                        first,
                                        class,
                                        received,
                                        remitted,
                                        matched,
                                        sent,
                                        as_at,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    rows
}

#[test]
fn every_row_of_evidence_reaches_the_one_verdict_the_table_gives() {
    let rows = rows();
    assert_eq!(rows.len(), 3 * 2 * 3 * 6 * 4 * 3 * 3 * 3);
    let mut reached = BTreeSet::new();
    for row in &rows {
        let got = answer(row.json(), row.as_at);
        assert_eq!(got, row.expected(), "{row:?}");
        reached.insert(got);
    }
    let every: BTreeSet<String> = VERDICTS
        .iter()
        .chain(&[REFUSED])
        .map(|v| v.to_string())
        .collect();
    assert_eq!(reached, every);
}

#[test]
fn a_row_with_no_sg_amount_is_never_decided() {
    for payday in PAYDAYS {
        let due = payday.due(false);
        for as_at in [after(due, -1), after(due, 10)] {
            let line = json!({"employee_id": "E1", "qe_day": payday.qe, "sg_amount": "0.00"});
            assert_eq!(answer(line, as_at), UNKNOWN, "{payday:?}");
        }
    }
}

/// A 7 August 2026 receipt of the whole amount for the 6 August payday, a
/// return out of it, and what followed. The usual deadline is 17 August and the
/// first-contribution deadline 3 September.
#[derive(Clone, Copy, Debug)]
struct Trail {
    first: bool,
    /// What came back of the 120.00.
    back: &'static str,
    returned_on: Date,
    /// When the amount came back to the fund, if it did.
    resent: Option<Date>,
    /// When the fund allocated what it kept of the 7 August receipt.
    kept_allocated: Option<Date>,
    /// When the fund allocated the resubmission.
    resent_allocated: Option<Date>,
    as_at: Date,
}

const RECEIPT: &str = "2026-08-07";

impl Trail {
    fn json(&self) -> Value {
        let mut events = vec![json!({
            "event": "returned", "event_date": self.returned_on.isoformat(),
            "amount": self.back, "receipt_date": RECEIPT,
        })];
        if let Some(day) = self.resent {
            events.push(json!({
                "event": "received", "event_date": day.isoformat(), "amount": self.back,
            }));
            if let Some(allocated) = self.resent_allocated {
                events.push(json!({
                    "event": "allocated", "event_date": allocated.isoformat(),
                    "amount": self.back, "receipt_date": day.isoformat(),
                }));
            }
        }
        if let Some(allocated) = self.kept_allocated {
            events.push(json!({
                "event": "allocated", "event_date": allocated.isoformat(),
                "amount": REST, "receipt_date": RECEIPT,
            }));
        }
        json!({
            "employee_id": "E1",
            "qe_day": "2026-08-06",
            "sg_amount": SG,
            "first_to_fund": self.first,
            "remitted": RECEIPT,
            "remitted_amount": SG,
            "received": RECEIPT,
            "matched_amount": SG,
            "events": events,
        })
    }

    /// The table the README gives for a line with a return.
    fn expected(&self) -> &'static str {
        // 3 business days after 7 August, the fund's limit, is 12 August.
        if self.returned_on > date("2026-08-12") {
            return UNKNOWN;
        }
        let Some(resent) = self.resent.filter(|day| *day <= self.as_at) else {
            return UNKNOWN;
        };
        let seen = |day: Option<Date>| day.filter(|d| *d <= self.as_at);
        let usual = date(PAYDAYS[0].usual);
        let due = PAYDAYS[0].due(self.first);
        let kept_part = self.back == PART;
        // Only the first kept receipt has the 20 business days.
        let window = if kept_part { usual } else { due };
        if kept_part && seen(self.kept_allocated).is_none() {
            return UNKNOWN;
        }
        let Some(allocated) = seen(self.resent_allocated) else {
            return UNKNOWN;
        };
        if kept_part && self.kept_allocated.unwrap() > due.min(self.as_at) {
            return UNKNOWN;
        }
        if resent <= window {
            return if allocated <= window {
                ON_TIME
            } else {
                UNKNOWN
            };
        }
        if resent <= due { UNKNOWN } else { LATE }
    }
}

fn trails() -> Vec<Trail> {
    let mut trails = Vec::new();
    for first in [false, true] {
        for back in [PART, SG] {
            for returned_on in ["2026-08-10", "2026-08-12", "2026-08-13"].map(date) {
                for resent in [
                    None,
                    Some(date("2026-08-14")),
                    Some(date("2026-08-18")),
                    Some(date("2026-09-04")),
                ] {
                    let kept_days = [None, Some("2026-08-10"), Some("2026-09-07")];
                    let kept_days = if back == PART {
                        &kept_days[..]
                    } else {
                        &kept_days[..1]
                    };
                    for kept_allocated in kept_days.iter().map(|d| d.map(date)) {
                        let later = |days| resent.map(|d| after(d, days));
                        for resent_allocated in [None, resent, later(4)] {
                            for as_at in ["2026-08-15", "2026-09-08"].map(date) {
                                trails.push(Trail {
                                    first,
                                    back,
                                    returned_on,
                                    resent,
                                    kept_allocated,
                                    resent_allocated,
                                    as_at,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    trails
}

#[test]
fn every_trail_of_returned_money_reaches_the_one_verdict_the_table_gives() {
    let trails = trails();
    let mut reached = BTreeSet::new();
    for trail in &trails {
        let got = answer(trail.json(), trail.as_at);
        assert_eq!(got, trail.expected(), "{trail:?}");
        reached.insert(got);
    }
    let every: BTreeSet<String> = [ON_TIME, LATE, UNKNOWN].map(String::from).into();
    assert_eq!(reached, every);
}
