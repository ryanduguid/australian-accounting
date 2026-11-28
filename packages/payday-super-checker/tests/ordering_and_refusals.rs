// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Super rows that tie on every other field order by amount, an amount that
//! cannot be ordered refuses the join, and the small refusals of the
//! notional-earnings, derivation and USI readers name their cause.
//!
//! The join, notional-earnings and pathway expectations come from the Python
//! engine (`paydaysuper.join.join`, `sgc.notional_earnings`,
//! `deadlines.PATHWAY_WORDS`) run under CPython 3.14 on the Linux test host.
//! The derivation and USI readers have no Python counterpart: their
//! expectations are this crate's documented refusals.

use std::path::Path;

use payday_super_checker::api::evaluate;
use payday_super_checker::deadlines::{ITEM4_ALIGNED, SKIP_DB, pathway_words};
use payday_super_checker::decimal::Decimal;
use payday_super_checker::py::Date;
use payday_super_checker::pyre::Regex;
use payday_super_checker::{derivation, rates, sgc, usi};
use serde_json::{Value, json};

fn super_row(amount: &str) -> Value {
    json!({"employee_id": "E1", "paid_date": "2026-07-20", "amount": amount, "row": 2})
}

fn join(amounts: &[&str]) -> Result<Value, String> {
    let rows: Vec<Value> = amounts.iter().map(|a| super_row(a)).collect();
    evaluate("join", &json!({"payroll_rows": [], "super_rows": rows}))
}

#[test]
fn super_rows_equal_but_for_amount_order_by_amount() {
    // Python: join([], [row 2 for 20.00, row 2 for 10.00]).orphans -> 10.00, 20.00.
    let joined = join(&["20.00", "10.00"]).unwrap();
    let amounts: Vec<&Value> = joined["orphans"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| &o["amount"])
        .collect();
    assert_eq!(amounts, [&json!("10.00"), &json!("20.00")]);
}

#[test]
fn an_amount_that_cannot_be_ordered_refuses_the_join() {
    // Python raises decimal.InvalidOperation for both a quiet and a signalling NaN.
    for nan in ["NaN", "sNaN"] {
        assert_eq!(
            join(&[nan, "10.00"]).unwrap_err(),
            "decimal.InvalidOperation"
        );
    }
}

#[test]
fn a_negative_shortfall_earns_no_notional_interest() {
    // Python: notional_earnings(Decimal("-0.01"), ...) raises
    // ValueError("shortfall cannot be negative").
    let day = |d| Date::new(2026, 7, d).unwrap();
    let refused = sgc::notional_earnings(
        &Decimal::parse("-0.01").unwrap(),
        day(28),
        day(31),
        &rates::load_gic().unwrap(),
        false,
    )
    .unwrap_err();
    assert_eq!(refused.message(), "shortfall cannot be negative");
}

#[test]
fn a_pathway_without_plain_words_keeps_its_code() {
    // Python: PATHWAY_WORDS.get(pathway, pathway).
    assert_eq!(pathway_words(ITEM4_ALIGNED), "item 4 aligned");
    assert_eq!(pathway_words(SKIP_DB), "SKIP_DB");
}

#[test]
fn a_derivation_of_input_that_is_not_an_object_is_refused() {
    assert_eq!(
        derivation::derive(&json!(["2026-07-09"])).unwrap_err(),
        "the JSON input must be an object"
    );
}

#[test]
fn an_extract_whose_last_line_is_not_its_run_date_is_unreadable() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/usi_extract");
    let data = String::from_utf8(std::fs::read(corpus).unwrap()).unwrap();
    assert!(usi::parse_extract(data.as_bytes()).is_ok());
    let renamed = data.replace("This Report Run at:", "This report ran at:");
    assert_ne!(renamed, data);
    assert_eq!(
        usi::parse_extract(renamed.as_bytes()).err(),
        Some("extract_unreadable")
    );
}

#[test]
fn a_usi_file_that_is_not_utf8_names_the_byte() {
    let refused = usi::read_usis("in.csv", b"usi\n\xff\n").unwrap_err();
    assert_eq!(
        refused.message(),
        "in.csv is not UTF-8 text (byte 0xff at position 4). Excel's plain 'CSV' \
         export uses the Windows code page: re-save it as 'CSV UTF-8 (Comma \
         delimited)' and run again."
    );
}

/// CPython 3.15: `re.match(r"(?:(?=(a))x|a)", "a")` ends at 1 with group 1
/// None, as the lookahead's capture goes when what follows it fails.
#[test]
fn a_lookahead_whose_continuation_fails_keeps_no_capture() {
    let m = Regex::python(r"(?:(?=(a))x|a)").captures("a").unwrap();
    assert_eq!((m.group(1), m.end), (None, 1));
}
