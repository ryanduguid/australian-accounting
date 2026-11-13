// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Every register row reaches exactly one answer to each question: one gate
//! verdict with one `GATE_*` code per limb that did not pass, and one repayment
//! verdict.
//!
//! The rules are restated here from ITAA 1936 s 109N(1) and (3) and the README's
//! register and reason-code tables, apart from `src/gate.rs` and `src/myr.rs`,
//! and every combination of the listed facts is walked through one register
//! review, so a changed branch in the engine disagrees with a table for some
//! row. The walk needs every summary count to occur and every register column
//! to be one it sets, so it cannot pass by reaching none, and a new column or
//! verdict fails it until the table gives it a row. All facts are fabricated.
//!
//! Every row reviews 2025-26 with a 1-year remaining term, which any loan the
//! gate passes can have. The repayment arithmetic and how it moves stay with
//! `tests/properties.rs`.

use std::collections::BTreeSet;

use div7a_loan_review::api;
use div7a_loan_review::register::{GATE_COLUMNS, MYR_COLUMNS, OPTIONAL_COLUMNS, SUMMARY_KEYS};
use serde_json::{Map, Value, json};

// The README's words, spelled here so the engine's own constants cannot move them.
const COMPLYING: &str = "COMPLYING";
const NOT_COMPLYING: &str = "NOT_COMPLYING";
const UNKNOWN: &str = "UNKNOWN";
const MYR_MET: &str = "MYR_MET";
const MYR_SHORT: &str = "MYR_SHORT";
const REFUSED: &str = "REFUSED";
const SKIPPED: &str = "SKIPPED";
const COUNTS: [&str; 7] = [
    COMPLYING,
    NOT_COMPLYING,
    MYR_MET,
    MYR_SHORT,
    UNKNOWN,
    REFUSED,
    SKIPPED,
];

/// The 2024-25 benchmark the gate reads for a loan made that year, and a rate
/// a ten-thousandth below it.
const RATE_2024_25: &str = "0.0877";
const BELOW: &str = "0.0876";
/// 1,000.00 for one year at the 2025-26 benchmark, 8.37%: s 109E(6) with a
/// remaining term of 1 is the balance plus a year's interest.
const REQUIRED: &str = "1083.70";
const CENT_SHORT: &str = "1083.69";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Limb {
    Pass,
    Fail,
    Unknown,
}

impl Limb {
    fn name(self) -> &'static str {
        match self {
            Limb::Pass => "PASS",
            Limb::Fail => "FAIL",
            Limb::Unknown => "UNKNOWN",
        }
    }
}

/// One register row's facts.
#[derive(Clone, Copy, Debug)]
struct Row {
    written: Option<bool>,
    in_time: Option<bool>,
    /// 2024-25 has a reviewed benchmark; 2018-19 is before the table.
    made: &'static str,
    rate: Option<&'static str>,
    term: Option<u32>,
    secured: Option<bool>,
    coverage: Option<&'static str>,
    paid: Option<&'static str>,
}

fn tristate(value: Option<bool>) -> &'static str {
    match value {
        None => "unknown",
        Some(true) => "true",
        Some(false) => "false",
    }
}

fn limb(value: Option<bool>) -> Limb {
    match value {
        None => Limb::Unknown,
        Some(true) => Limb::Pass,
        Some(false) => Limb::Fail,
    }
}

impl Row {
    fn json(&self, id: &str) -> Value {
        json!({
            "loan_id": id,
            "borrower_reference": "B-1",
            "out_of_scope_reason": "",
            "year_of_income_being_tested": "",
            "written_agreement": tristate(self.written),
            "terms_in_place_before_lodgment_day": tristate(self.in_time),
            "maximum_term_years": self.term.map_or(String::new(), |t| t.to_string()),
            "secured_by_registered_mortgage_over_real_property": tristate(self.secured),
            "security_coverage_at_first_made": self.coverage.unwrap_or(""),
            "interest_rate_for_years_after_year_loan_made": self.rate.unwrap_or(""),
            "year_loan_made": self.made,
            "amalgamated_loan_unpaid_at_end_of_previous_year": "1000.00",
            "remaining_term_years": "1",
            "payments_applied_during_the_year": self.paid.unwrap_or(""),
        })
    }

    /// s 109N(1)(b): the agreed rate against the benchmark for the year made.
    fn interest(&self) -> Limb {
        match (self.rate, self.made) {
            (None, _) | (Some(_), "2018-19") => Limb::Unknown,
            (Some(BELOW), _) => Limb::Fail,
            _ => Limb::Pass,
        }
    }

    /// s 109N(3): 25 years for a registered mortgage covering at least 110% when the
    /// loan is first made, 7 for any other loan, and not known until the
    /// security facts are.
    fn maximum(&self) -> Option<u32> {
        match (self.secured, self.coverage) {
            (Some(false), _) => Some(7),
            (Some(true), Some("1.10" | "1.11")) => Some(25),
            (Some(true), Some(_)) => Some(7),
            _ => None,
        }
    }

    /// s 109N(1)(c). A term within 7 years passes whatever the security.
    fn term(&self) -> Limb {
        match (self.term, self.maximum()) {
            (None, _) => Limb::Unknown,
            (Some(term), Some(maximum)) if term > maximum => Limb::Fail,
            (Some(term), None) if term > 7 => Limb::Unknown,
            _ => Limb::Pass,
        }
    }

    fn limbs(&self) -> [(&'static str, Limb); 4] {
        [
            ("WRITTEN_AGREEMENT", limb(self.written)),
            ("LODGMENT_DAY", limb(self.in_time)),
            ("INTEREST", self.interest()),
            ("TERM", self.term()),
        ]
    }

    /// The gate verdict and its codes: any failed limb is a breach, and an
    /// unestablished limb is never read as one.
    fn gate(&self) -> (&'static str, Vec<String>) {
        let limbs = self.limbs();
        let states: Vec<Limb> = limbs.iter().map(|(_, l)| *l).collect();
        let verdict = if states.contains(&Limb::Fail) {
            NOT_COMPLYING
        } else if states.contains(&Limb::Unknown) {
            UNKNOWN
        } else {
            COMPLYING
        };
        let codes = limbs
            .iter()
            .filter(|(_, l)| *l != Limb::Pass)
            .map(|(name, l)| format!("GATE_{name}_{}", l.name()))
            .collect();
        (verdict, codes)
    }

    /// The repayment verdict and its codes: no figure unless the gate passed
    /// (s 109E(3)(b)), and no shortfall from a payment never established.
    fn myr(&self) -> (&'static str, Vec<&'static str>) {
        match (self.gate().0, self.paid) {
            (NOT_COMPLYING, _) => (REFUSED, vec!["REFUSED_GATE_NOT_COMPLYING"]),
            (UNKNOWN, _) => (REFUSED, vec!["REFUSED_GATE_UNKNOWN"]),
            (_, None) => (UNKNOWN, vec!["PAYMENTS_APPLIED_UNKNOWN"]),
            (_, Some(REQUIRED)) => (MYR_MET, vec![]),
            (_, Some(_)) => (MYR_SHORT, vec![]),
        }
    }
}

fn rows() -> Vec<Row> {
    let known = [None, Some(true), Some(false)];
    let mut rows = Vec::new();
    for written in known {
        for in_time in known {
            for made in ["2024-25", "2018-19"] {
                for rate in [None, Some(BELOW), Some(RATE_2024_25), Some("0.0900")] {
                    for term in [None, Some(7), Some(8), Some(25), Some(26)] {
                        for secured in known {
                            for coverage in [None, Some("1.09"), Some("1.10"), Some("1.11")] {
                                for paid in [None, Some(CENT_SHORT), Some(REQUIRED)] {
                                    rows.push(Row {
                                        written,
                                        in_time,
                                        made,
                                        rate,
                                        term,
                                        secured,
                                        coverage,
                                        paid,
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

fn codes(block: &Value) -> Vec<String> {
    block["reason_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect()
}

fn review(rows: Vec<Value>) -> Value {
    api::evaluate(
        "review-register",
        &json!({"rows": rows, "year_of_income": "2025-26"}),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn every_row_of_evidence_reaches_the_one_answer_the_tables_give() {
    let rows = rows();
    assert_eq!(rows.len(), 3 * 3 * 2 * 4 * 5 * 3 * 4 * 3);
    let mut cells: Vec<Value> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| row.json(&format!("L{i}")))
        .collect();
    // An operator's out-of-scope mark and a loan made before 4 December 1997
    // (s 109D(5)) are each skipped, whatever else the row says.
    let mut marked = rows[0].json("marked");
    marked["out_of_scope_reason"] = json!("public company");
    let mut old = rows[0].json("old");
    old["year_loan_made"] = json!("1997-98");
    cells.extend([marked, old]);

    let columns: BTreeSet<&str> = cells[0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let every_column: BTreeSet<&str> = GATE_COLUMNS
        .iter()
        .chain(&MYR_COLUMNS)
        .chain(&OPTIONAL_COLUMNS)
        .copied()
        .collect();
    assert_eq!(
        columns, every_column,
        "a register column the walk does not set"
    );

    let report = review(cells);
    let mut lines: Map<String, Value> = Map::new();
    for line in report["lines"].as_array().unwrap() {
        lines.insert(line["loan_id"].as_str().unwrap().into(), line.clone());
    }
    assert_eq!(lines.len(), rows.len() + 2);
    for (i, row) in rows.iter().enumerate() {
        let line = &lines[&format!("L{i}")];
        assert_eq!(line["status"], "REVIEWED", "{row:?}");
        let (gate, gate_codes) = row.gate();
        assert_eq!(line["gate"]["verdict"], gate, "{row:?}");
        assert_eq!(codes(&line["gate"]), gate_codes, "{row:?}");
        let (myr, myr_codes) = row.myr();
        assert_eq!(line["myr"]["verdict"], myr, "{row:?}");
        assert_eq!(codes(&line["myr"]), myr_codes, "{row:?}");
    }
    for id in ["marked", "old"] {
        assert_eq!(lines[id]["status"], SKIPPED, "{id}");
        assert!(lines[id]["gate"].is_null() && lines[id]["myr"].is_null());
    }

    // Each count is a question's answer, so the walk's own tally must match it,
    // and every count the summary publishes must be reached.
    let summary = report["summary"].as_object().unwrap();
    let published: Vec<&str> = summary.keys().map(String::as_str).collect();
    assert_eq!(published, COUNTS);
    assert_eq!(&SUMMARY_KEYS[..], &COUNTS[..]);
    for key in COUNTS {
        let want = match key {
            SKIPPED => 2,
            // A row counts once here when either question is undecided.
            UNKNOWN => rows
                .iter()
                .filter(|r| r.gate().0 == UNKNOWN || r.myr().0 == UNKNOWN)
                .count(),
            COMPLYING | NOT_COMPLYING => rows.iter().filter(|r| r.gate().0 == key).count(),
            _ => rows.iter().filter(|r| r.myr().0 == key).count(),
        };
        assert!(want > 0, "{key} is never reached");
        assert_eq!(summary[key], want, "{key}");
    }
}

/// A loan the gate passes on a maximum term its security facts establish
/// cannot have a remaining term longer than that maximum (s 109E(3)(b) and (c),
/// s 109N(1)(c) and (3), s 109E(6)). Where the security facts leave the maximum
/// open, or the gate did not pass, this refusal does not apply; a row whose own
/// term is shorter than the remaining term is still refused (next test).
#[test]
fn a_remaining_term_past_the_maximum_term_cannot_hold() {
    let conflict = "REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM";
    let base = Row {
        written: Some(true),
        in_time: Some(true),
        made: "2024-25",
        rate: Some(RATE_2024_25),
        term: Some(7),
        secured: Some(false),
        coverage: None,
        paid: Some(REQUIRED),
    };
    let secured = |coverage, term| Row {
        secured: Some(true),
        coverage: Some(coverage),
        term: Some(term),
        ..base
    };
    let unsecured_unknown = Row {
        secured: None,
        ..base
    };
    let not_complying = Row {
        written: Some(false),
        ..base
    };
    let undecided = Row {
        written: None,
        ..base
    };
    let cases: [(Row, &str, &[&str]); 10] = [
        (base, "7", &[]),
        (base, "7.5", &[conflict]),
        (base, "8", &[conflict]),
        (secured("1.09", 7), "8", &[conflict]),
        (secured("1.10", 25), "25", &[]),
        (secured("1.10", 25), "26", &[conflict]),
        // The maximum is open, but the row's own 7-year term is not.
        (unsecured_unknown, "8", &["REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM"]),
        (not_complying, "30", &["REFUSED_GATE_NOT_COMPLYING"]),
        (undecided, "8", &["REFUSED_GATE_UNKNOWN"]),
        (base, "0", &["REFUSED_REMAINING_TERM_NOT_POSITIVE"]),
    ];
    let cells: Vec<Value> = cases
        .iter()
        .enumerate()
        .map(|(i, (row, remaining, _))| {
            let mut cell = row.json(&format!("C{i}"));
            cell["remaining_term_years"] = json!(remaining);
            cell
        })
        .collect();
    let report = review(cells);
    for (i, (row, remaining, want)) in cases.iter().enumerate() {
        let id = format!("C{i}");
        let line = report["lines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["loan_id"] == id.as_str())
            .unwrap();
        assert_eq!(codes(&line["myr"]), *want, "{row:?} {remaining}");
        if want.is_empty() {
            assert!(
                line["myr"]["myr_required"].is_string(),
                "{row:?} {remaining}"
            );
        }
    }
}

/// The s 109E(6) remaining term is the longest constituent-loan term less the
/// years since the loan year, so on a loan the gate passes it cannot be longer
/// than the row's own term, both rounded up. A 5-year loan entered with 7 years
/// remaining would otherwise get a lower repayment than the Act requires.
#[test]
fn a_remaining_term_past_the_loan_term_cannot_hold() {
    let conflict = "REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM";
    let base = Row {
        written: Some(true),
        in_time: Some(true),
        made: "2024-25",
        rate: Some(RATE_2024_25),
        term: None,
        secured: Some(false),
        coverage: None,
        paid: Some(REQUIRED),
    };
    let secured = Row {
        secured: Some(true),
        coverage: Some("1.10"),
        ..base
    };
    let unsecured_unknown = Row {
        secured: None,
        ..base
    };
    let not_complying = Row {
        written: Some(false),
        ..base
    };
    let cases: [(Row, &str, &str, &[&str]); 9] = [
        (base, "5", "7", &[conflict]),
        (base, "5", "5", &[]),
        (base, "5", "5.5", &[conflict]),
        (base, "5.5", "6", &[]),
        (
            base,
            "5",
            "8",
            &["REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM"],
        ),
        (secured, "20", "21", &[conflict]),
        (secured, "20", "20", &[]),
        (unsecured_unknown, "5", "7", &[conflict]),
        (not_complying, "5", "7", &["REFUSED_GATE_NOT_COMPLYING"]),
    ];
    let cells: Vec<Value> = cases
        .iter()
        .enumerate()
        .map(|(i, (row, term, remaining, _))| {
            let mut cell = row.json(&format!("T{i}"));
            cell["maximum_term_years"] = json!(term);
            cell["remaining_term_years"] = json!(remaining);
            cell
        })
        .collect();
    let report = review(cells);
    for (i, (row, term, remaining, want)) in cases.iter().enumerate() {
        let id = format!("T{i}");
        let line = report["lines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["loan_id"] == id.as_str())
            .unwrap();
        let myr = &line["myr"];
        assert_eq!(codes(myr), *want, "{row:?} {term} {remaining}");
        assert_eq!(
            myr["myr_required"].is_string(),
            want.is_empty(),
            "{row:?} {term} {remaining}"
        );
    }
}
