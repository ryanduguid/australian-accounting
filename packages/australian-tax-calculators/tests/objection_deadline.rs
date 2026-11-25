// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `objection-deadline` at each period of TAA 1953 s 14ZW (compilation no.
//! 227), the counting rules of the Acts Interpretation Act 1901 ss 2G and 36
//! (compilation no. 39) and its refusals, all read on 26 November 2026. The
//! site's own cases are in `site_mirrors.rs`.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(decision: &str, notice: &str, extra: Value) -> Value {
    let mut v = json!({"year": "2026-27", "scope_confirmed": true, "decision": decision, "notice_date": notice});
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    v
}

fn run(decision: &str, notice: &str, extra: Value) -> Value {
    api::evaluate("objection-deadline", &input(decision, notice, extra)).unwrap()
}

fn code(decision: &str, notice: &str, extra: Value) -> &'static str {
    api::evaluate_coded("objection-deadline", &input(decision, notice, extra))
        .unwrap_err()
        .code
}

/// s 14ZW(1)(aa): 2 years where item 1, 2, 3 or 3A applies, else 4; (aaa): 4
/// years for FBT; (c): 60 days. AIA s 36(1) item 6 leaves out the day notice
/// is given: 60 days after 2 August ends on 1 October.
#[test]
fn each_period_runs_from_the_day_after_notice() {
    for item in ["1", "2", "3", "3A"] {
        let out = run("assessment", "2026-08-02", json!({"s170_item": item}));
        assert_eq!(out["provision"], "subparagraph 14ZW(1)(aa)(i)");
        assert_eq!(out["period_ends"], "2028-08-02");
    }
    let four = run("assessment", "2026-08-02", json!({"s170_item": "none"}));
    assert_eq!((four["provision"].clone(), four["period_ends"].clone()), (json!("subparagraph 14ZW(1)(aa)(ii)"), json!("2030-08-02")));
    let fbt = run("fbt_assessment", "2026-08-02", json!({}));
    assert_eq!((fbt["provision"].clone(), fbt["period_ends"].clone()), (json!("paragraph 14ZW(1)(aaa)"), json!("2030-08-02")));
    let other = run("other", "2026-08-02", json!({}));
    assert_eq!((other["provision"].clone(), other["period_ends"].clone()), (json!("paragraph 14ZW(1)(c)"), json!("2026-10-01")));
}

/// AIA s 36(2), with the example in it: a last day on Saturday 31 March 2029
/// moves to Monday 2 April; a stated holiday on the Monday moves it again (s
/// 36(3)).
#[test]
fn a_weekend_or_stated_holiday_moves_the_last_day() {
    let out = run("other", "2029-01-30", json!({}));
    assert_eq!((out["period_ends"].clone(), out["last_day"].clone()), (json!("2029-03-31"), json!("2029-04-02")));
    let held = run("other", "2029-01-30", json!({"holidays": ["2029-04-02", "2029-04-03"]}));
    assert_eq!(held["last_day"], "2029-04-04");
    let weekday = run("other", "2029-01-30", json!({"holidays": ["2029-03-30"]}));
    assert_eq!(weekday["last_day"], "2029-04-02");
}

/// Subsections (1B) and (1BA): the later of 60 days after the amended notice
/// and the period after the assessment amended.
#[test]
fn an_amended_assessment_takes_the_later_end() {
    let late = run("amended_assessment", "2028-10-02", json!({"s170_item": "1", "original_notice_date": "2026-10-20", "amended_before": false}));
    assert_eq!(late["provision"], "subsection 14ZW(1BA)");
    assert_eq!(late["amended_notice_period_ends"], "2028-12-01");
    assert_eq!(late["original_notice_period_ends"], "2028-10-20");
    assert_eq!(late["period_ends"], "2028-12-01");
    let early = run("amended_assessment", "2027-01-04", json!({"s170_item": "none", "original_notice_date": "2026-10-20", "amended_before": true}));
    assert_eq!(early["provision"], "subsection 14ZW(1B)");
    assert_eq!(early["period_ends"], "2030-10-20");
    let fbt = run("amended_fbt_assessment", "2027-06-15", json!({"original_notice_date": "2027-06-15", "amended_before": false}));
    assert_eq!((fbt["provision"].clone(), fbt["period_ends"].clone()), (json!("subsection 14ZW(1B)"), json!("2031-06-15")));
}

/// AIA s 2G(2) for a period of years: a start on 29 February ends at the end of
/// February; a start on 1 March ends on the day before 1 March.
#[test]
fn leap_days_count_as_months_do() {
    assert_eq!(run("assessment", "2026-02-28", json!({"s170_item": "1"}))["period_ends"], "2028-02-29");
    assert_eq!(run("assessment", "2028-02-28", json!({"s170_item": "1"}))["period_ends"], "2030-02-28");
    assert_eq!(run("assessment", "2028-02-29", json!({"s170_item": "1"}))["period_ends"], "2030-02-28");
}

#[test]
fn refuses_what_it_does_not_settle() {
    let amended = json!({"s170_item": "2", "original_notice_date": "2026-10-20", "amended_before": true});
    for (decision, notice, extra, want) in [
        ("amended_assessment", "2027-01-04", amended, "objection_deadline.unsettled"),
        ("ruling", "2026-11-18", json!({}), "objection_deadline.fact_invalid"),
        ("assessment", "2026-11-18", json!({}), "objection_deadline.fact_invalid"),
        ("assessment", "2026-11-18", json!({"s170_item": "4"}), "objection_deadline.fact_invalid"),
        ("fbt_assessment", "2026-11-18", json!({"s170_item": "1"}), "objection_deadline.fact_invalid"),
        ("other", "2026-11-18", json!({"original_notice_date": "2026-11-01"}), "objection_deadline.fact_invalid"),
        ("amended_fbt_assessment", "2026-07-01", json!({"original_notice_date": "2026-07-02", "amended_before": false}), "objection_deadline.fact_invalid"),
        ("amended_fbt_assessment", "2026-07-01", json!({"original_notice_date": "2026-06-02"}), "objection_deadline.fact_invalid"),
        ("amended_fbt_assessment", "2026-07-01", json!({"original_notice_date": "2026-06-02", "amended_before": "no"}), "input.flag_invalid"),
        ("amended_fbt_assessment", "2026-07-01", json!({"original_notice_date": "1 June 2026", "amended_before": false}), "input.date_invalid"),
        ("other", "2026-11-31", json!({}), "input.date_invalid"),
        ("other", "2026-11-18", json!({"holidays": ["2026-13-01"]}), "objection_deadline.fact_invalid"),
        ("other", "2026-11-18", json!({"holidays": "2026-12-25"}), "objection_deadline.fact_invalid"),
        ("other", "2026-11-18", json!({"year": "2025-26"}), "period.unsupported"),
        ("other", "2026-11-18", json!({"lodged": "2026-11-20"}), "input.field_unknown"),
    ] {
        assert_eq!(code(decision, notice, extra.clone()), want, "{decision} {extra}");
    }
}
