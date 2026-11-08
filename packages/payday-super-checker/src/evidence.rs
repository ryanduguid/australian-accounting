// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! What a line says stands behind its receipt: where the receipt date came
//! from, the fund's member verification answer and any fund error. These facts
//! are written beside the verdict and never weighed in it, so no verdict,
//! figure or exit code depends on them.
//!
//! Sources, read 8 November 2026: ATO, 'Payday Super and SuperStream for super
//! funds' (updated 10 August 2026), sections 'Member verification requests' and
//! 'Rejecting contributions'; ATO, 'Getting it right: compliance in the first
//! year of Payday Super' (23 June 2026); PCG 2026/1 paragraphs 11 and 13.

use crate::assess::{AT_RISK, Assessed, FIRST_YEAR_END};
use crate::deadlines::{ContribLine, EventKind};
use crate::returned::carries_return_facts;

/// Where the row's fund receipt is known from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReceiptEvidence {
    /// The fund's own record dates the receipt.
    Fund,
    /// A clearing house's record of the fund's receipt dates it.
    ClearingHouse,
    /// A record shows the contribution was sent to the fund; nothing confirms
    /// the fund received it.
    Sent,
}

/// The fund's answer to a member verification request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MvrStatus {
    /// It can match the member to an active account and accept a contribution.
    Matched,
    /// It cannot find the member or cannot accept the contribution.
    NotMatched,
    /// No request was sent.
    NotSent,
}

/// A fund error (CTER) for this contribution, or a check that found none.
#[derive(Clone, Debug, PartialEq)]
pub enum FundError {
    NoneReceived,
    Reference(String),
}

/// The optional evidence record on a line. Each field left out is not stated.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Evidence {
    pub receipt: Option<ReceiptEvidence>,
    pub mvr: Option<MvrStatus>,
    pub fund_error: Option<FundError>,
}

pub const RECEIPT_EVIDENCE: [(&str, ReceiptEvidence); 3] = [
    ("fund", ReceiptEvidence::Fund),
    ("clearing_house", ReceiptEvidence::ClearingHouse),
    ("sent", ReceiptEvidence::Sent),
];

pub const MVR_STATUS: [(&str, MvrStatus); 3] = [
    ("matched", MvrStatus::Matched),
    ("not_matched", MvrStatus::NotMatched),
    ("not_sent", MvrStatus::NotSent),
];

fn word<T: Copy + PartialEq>(
    raw: &str,
    field: &str,
    words: &[(&str, T)],
) -> Result<Option<T>, String> {
    let text = raw.trim().to_ascii_lowercase();
    if text.is_empty() {
        return Ok(None);
    }
    words
        .iter()
        .find(|(w, _)| *w == text)
        .map(|(_, v)| Some(*v))
        .ok_or_else(|| {
            let listed: Vec<&str> = words.iter().map(|(w, _)| *w).collect();
            format!(
                "{field} {} is not {} or blank",
                crate::pytext::repr(raw),
                listed.join(", ")
            )
        })
}

fn name<T: Copy + PartialEq>(value: T, words: &[(&'static str, T)]) -> &'static str {
    words
        .iter()
        .find(|(_, v)| *v == value)
        .map(|(w, _)| *w)
        .expect("every value has a word")
}

impl Evidence {
    /// The record from the 3 fields as a file or the API gives them. Blank is
    /// not stated; `none` in `fund_error_ref` says no fund error was received.
    pub fn parse(receipt: &str, mvr: &str, fund_error: &str) -> Result<Evidence, String> {
        let reference = fund_error.trim();
        Ok(Evidence {
            receipt: word(receipt, "receipt_evidence", &RECEIPT_EVIDENCE)?,
            mvr: word(mvr, "mvr_status", &MVR_STATUS)?,
            fund_error: match reference {
                "" => None,
                r if r.eq_ignore_ascii_case("none") => Some(FundError::NoneReceived),
                r => Some(FundError::Reference(r.to_string())),
            },
        })
    }

    /// The 3 fields as they were read, for echoing a line back.
    pub fn fields(&self) -> [(&'static str, Option<String>); 3] {
        [
            (
                "receipt_evidence",
                self.receipt.map(|r| name(r, &RECEIPT_EVIDENCE).to_string()),
            ),
            (
                "mvr_status",
                self.mvr.map(|m| name(m, &MVR_STATUS).to_string()),
            ),
            (
                "fund_error_ref",
                self.fund_error.as_ref().map(|e| match e {
                    FundError::NoneReceived => "none".to_string(),
                    FundError::Reference(r) => r.clone(),
                }),
            ),
        ]
    }
}

/// Whether the row records a fund receipt anywhere: its own receipt date, a
/// received event, or an allocated or returned event naming a receipt date.
fn records_receipt(line: &ContribLine) -> bool {
    line.received.is_some()
        || line.events.iter().any(|e| match e.kind {
            EventKind::Received | EventKind::Allocated { .. } => true,
            EventKind::Returned { receipt, .. } => receipt.is_some(),
        })
}

/// Evidence the line's own facts contradict, each naming the row.
pub fn problems(line: &ContribLine) -> Vec<String> {
    let row = line.row;
    match line.evidence.receipt {
        Some(source @ (ReceiptEvidence::Fund | ReceiptEvidence::ClearingHouse))
            if !records_receipt(line) =>
        {
            vec![format!(
                "row {row}: receipt_evidence={} names where a fund receipt is recorded, \
                 but this row records none. Fill fund_received_date or record the \
                 receipt with --events, or leave receipt_evidence blank",
                name(source, &RECEIPT_EVIDENCE)
            )]
        }
        Some(ReceiptEvidence::Sent) if records_receipt(line) => vec![format!(
            "row {row}: receipt_evidence=sent says the fund's receipt is not confirmed, \
             but this row records one. Name where the receipt is recorded (fund or \
             clearing_house), or remove the receipt"
        )],
        Some(ReceiptEvidence::Sent) if line.remitted.is_none() => vec![format!(
            "row {row}: receipt_evidence=sent needs remitted_date, the day the \
             contribution was paid"
        )],
        _ => Vec::new(),
    }
}

/// Write the evidence beside the verdict: a note listing it, a caveat for each
/// known error, and the ATO's first-year assumption where every condition for
/// it is on the record. None of these touches the verdict.
pub fn annotate(result: &mut Assessed) {
    let evidence = &result.line.evidence;
    let mut listed = Vec::new();
    match evidence.receipt {
        Some(ReceiptEvidence::Fund) => {
            listed.push("the fund receipt date is from the fund's own record".to_string())
        }
        Some(ReceiptEvidence::ClearingHouse) => {
            listed.push("the fund receipt date is from a clearing house's record of it".to_string())
        }
        Some(ReceiptEvidence::Sent) => listed.push(
            "a record shows the contribution sent to the fund; its receipt is not confirmed"
                .to_string(),
        ),
        None => {}
    }
    match evidence.mvr {
        Some(MvrStatus::Matched) => listed.push(
            "member verification: the fund matched the member and can accept a contribution"
                .to_string(),
        ),
        Some(MvrStatus::NotSent) => {
            listed.push("member verification: no request was sent".to_string())
        }
        // A known error is a caveat, below.
        Some(MvrStatus::NotMatched) | None => {}
    }
    if evidence.fund_error == Some(FundError::NoneReceived) {
        listed.push("no fund error was received for this contribution".to_string());
    }
    let mut caveats = Vec::new();
    if let Some(FundError::Reference(r)) = &evidence.fund_error {
        caveats.push(format!(
            "a fund error ({r}) is recorded for this contribution. A fund that rejects a \
             contribution must send an error message saying why, and a rejected amount is \
             not a contribution (LCR 2026/2 paragraphs 24 and 25). This verdict does not \
             reflect the error, and the reference alone does not say an amount came back: \
             if one did, record it with returned_by_fund or --events"
        ));
    }
    if evidence.mvr == Some(MvrStatus::NotMatched) {
        caveats.push(
            "member verification says the fund could not match the member or accept a \
             contribution, so a contribution sent with those details may be rejected. That \
             answer is not a fund error and does not say an amount came back, and this \
             verdict does not reflect it: correct the member details and, if an amount came \
             back, record it with returned_by_fund or --events"
                .to_string(),
        );
    }
    let first_year = first_year_assumption(result);
    if !listed.is_empty() {
        result.notes.push(format!(
            "evidence recorded with this row, which does not change the verdict: {}",
            listed.join("; ")
        ));
    }
    result.caveats.extend(caveats);
    if first_year {
        result.notes.push(
            "first-year approach: the ATO says an employer who pays on payday and has \
             received no error for the contribution can assume the fund received it on \
             time (ATO, Getting it right: compliance in the first year of Payday Super, 23 \
             June 2026). That is the ATO's compliance approach for QE days to 30 June 2027 \
             (PCG 2026/1 paragraph 13), not evidence of receipt, so this row stays AT_RISK. \
             The ATO still applies the law where it has definitive information of a \
             shortfall (PCG 2026/1 paragraph 11)"
                .to_string(),
        );
    }
}

/// Whether the record holds every fact the ATO's first-year scenario needs. The
/// scenario is 'I pay super guarantee on payday but I'm not sure it is received
/// on time', answered 'Check that no errors have been received for this
/// contribution': an `AT_RISK` line for a first-year QE day, `remitted_date` (the
/// day the employer sent the money) on the QE day itself, and
/// `fund_error_ref=none`. A `remitted_date` that is a clearing house's later
/// date only withholds the note. Leaving out a line with a failed member
/// verification or a return is this checker's conservative addition, not an
/// ATO condition.
fn first_year_assumption(result: &Assessed) -> bool {
    let line = &result.line;
    let evidence = &line.evidence;
    result.verdict == AT_RISK
        && line.qe_day <= FIRST_YEAR_END
        && line.remitted == Some(line.qe_day)
        && evidence.fund_error == Some(FundError::NoneReceived)
        && evidence.mvr != Some(MvrStatus::NotMatched)
        && !carries_return_facts(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assess::tests::{day, dec, line};
    use crate::deadlines::ContribEvent;

    fn event(kind: EventKind) -> ContribEvent {
        ContribEvent {
            kind,
            date: day(2027, 7, 14),
            amount: dec("300"),
            row: 2,
        }
    }

    fn with(receipt: ReceiptEvidence, events: Vec<ContribEvent>) -> ContribLine {
        let mut out = line();
        out.remitted = Some(day(2027, 7, 9));
        out.events = events;
        out.evidence.receipt = Some(receipt);
        out
    }

    /// Every event that names a receipt records one; only a returned event with
    /// no receipt date does not.
    #[test]
    fn every_receipt_bearing_event_records_a_receipt() {
        let receipt = day(2027, 7, 12);
        let bearing = [
            EventKind::Received,
            EventKind::Allocated { receipt },
            EventKind::Returned {
                receipt: Some(receipt),
                notified: None,
            },
        ];
        for kind in bearing {
            for source in [ReceiptEvidence::Fund, ReceiptEvidence::ClearingHouse] {
                assert!(
                    problems(&with(source, vec![event(kind)])).is_empty(),
                    "{kind:?}"
                );
            }
            let refused = problems(&with(ReceiptEvidence::Sent, vec![event(kind)]));
            assert!(
                refused[0]
                    .contains("receipt_evidence=sent says the fund's receipt is not confirmed"),
                "{kind:?}: {refused:?}"
            );
        }
        let unnamed = event(EventKind::Returned {
            receipt: None,
            notified: None,
        });
        assert!(problems(&with(ReceiptEvidence::Sent, vec![unnamed.clone()])).is_empty());
        let refused = problems(&with(ReceiptEvidence::Fund, vec![unnamed]));
        assert!(
            refused[0].contains("receipt_evidence=fund names where"),
            "{refused:?}"
        );
    }
}
