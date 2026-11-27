// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `myr::minimum_yearly_repayment`, the library call other crates make: it
//! answers as `repayment_within` does with no term limit, and says so.

use div7a_loan_review::decimal::Decimal;
use div7a_loan_review::myr::{GateSummary, MyrFacts, minimum_yearly_repayment, repayment_within};
use div7a_loan_review::raw::Raw;
use div7a_loan_review::years::parse_year;

fn facts() -> MyrFacts {
    let year = |label: &str| Some(parse_year(&Raw::text(label), "year").expect("a year"));
    let amount = |text: &str| Some(Decimal::parse(text).expect("a decimal"));
    MyrFacts {
        year_of_income: year("2026-27"),
        amalgamated_loan_unpaid_at_end_of_previous_year: amount("100000.00"),
        remaining_term_years: amount("5"),
        payments_applied_during_the_year: amount("25556.00"),
        gate_result: Some(GateSummary {
            verdict: "COMPLYING".into(),
            benchmark_year_used: "2023-24".into(),
            reasons: vec![],
            caveats: vec![],
        }),
        year_loan_made: year("2023-24"),
        loan_id: "L1".into(),
    }
}

#[test]
fn the_library_call_checks_no_term_limit_and_says_so() {
    let facts = facts();
    let got = minimum_yearly_repayment(&facts, None, None).expect("answers");
    let within = repayment_within(&facts, None, None, None, None).expect("answers");
    assert_eq!(got.to_json(), within.to_json());
    assert_eq!((got.verdict, got.loan_id.as_str()), ("MYR_MET", "L1"));
    assert!(
        got.caveats
            .iter()
            .any(|c| c.starts_with("maximum_term_years was not supplied")),
        "{:?}",
        got.caveats
    );
}
