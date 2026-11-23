// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Minimum notice of termination under the Fair Work Act 2009 s 117, against
//! the Fair Work Ombudsman's Gunter and Joan examples, each band boundary,
//! the over-45 week, each s 123 exclusion and each refusal, all read on 24
//! November 2026 (docs/calculation-evidence.md#notice-period).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// An employee given notice on `notice`, with service from `start` and born
/// on `born`, none excluded.
fn facts(notice: &str, start: &str, born: &str) -> Value {
    json!({
        "notice_date": notice, "service_start": start, "excluded_days": 0,
        "date_of_birth": born, "exclusion": "none",
        "year": year_of(notice), "scope_confirmed": true,
    })
}

fn year_of(day: &str) -> String {
    let (y, m): (i32, i32) = (day[..4].parse().unwrap(), day[5..7].parse().unwrap());
    let first = if m >= 7 { y } else { y - 1 };
    format!("{first}-{:02}", (first + 1) % 100)
}

fn with(mut base: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            base.as_object_mut().unwrap().remove(key);
        } else {
            base[key] = value.clone();
        }
    }
    base
}

fn run(input: &Value) -> Value {
    api::evaluate("notice-period", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("notice-period", input)
        .unwrap_err()
        .code
}

fn weeks(input: &Value) -> String {
    run(input)["rates"]["minimum_notice_weeks"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn the_fair_work_ombudsmans_gunter_gets_a_week() {
    // Gunter: 5 months' service, notice on a Tuesday; the period starts on
    // the Wednesday and a week ends on the next Tuesday.
    let gunter = run(&facts("2026-08-04", "2026-03-04", "1996-01-01"));
    assert_eq!(gunter["calculation"], "notice_period");
    assert_eq!(gunter["notice_required"], true);
    assert_eq!(gunter["over_45"], false);
    assert_eq!(gunter["rates"]["base_weeks"], "1");
    assert_eq!(gunter["rates"]["extra_weeks"], "0");
    assert_eq!(gunter["rates"]["minimum_notice_weeks"], "1");
    assert_eq!(gunter["rates"]["earliest_termination_day"], "2026-08-11");
    assert_eq!(gunter["rates"]["days_counted"], "154");
    assert!(gunter["amounts"].as_object().unwrap().is_empty());
}

#[test]
fn the_fair_work_ombudsmans_joan_counts_unpaid_parental_leave() {
    // Joan, 35: 5 years and 3 months including 12 months' unpaid parental
    // leave, which counts (s 22(4)), so more than 5 years: 4 weeks.
    assert_eq!(weeks(&facts("2026-08-01", "2021-05-01", "1991-01-01")), "4");
}

#[test]
fn an_employee_over_45_with_2_years_gets_another_week() {
    // 50, 3 1/2 years: 3 weeks plus 1 (s 117(3)(b)).
    let result = run(&facts("2026-08-01", "2023-02-01", "1976-01-01"));
    assert_eq!(result["over_45"], true);
    assert_eq!(result["rates"]["base_weeks"], "3");
    assert_eq!(result["rates"]["extra_weeks"], "1");
    assert_eq!(result["rates"]["minimum_notice_weeks"], "4");
    assert_eq!(result["rates"]["earliest_termination_day"], "2026-08-29");
}

#[test]
fn each_band_ends_at_its_anniversary() {
    for (start, last, more) in [
        ("2025-08-01", "1", "2"),
        ("2023-08-01", "2", "3"),
        ("2021-08-01", "3", "4"),
    ] {
        // Service to the end of 31 July is exactly the years; 1 August is more.
        assert_eq!(
            weeks(&facts("2026-07-31", start, "1996-01-01")),
            last,
            "{start}"
        );
        assert_eq!(
            weeks(&facts("2026-08-01", start, "1996-01-01")),
            more,
            "{start}"
        );
    }
}

#[test]
fn the_extra_week_needs_2_completed_years() {
    let exactly = facts("2026-07-31", "2024-08-01", "1976-01-01");
    assert_eq!(weeks(&exactly), "3");
    let short = facts("2026-07-30", "2024-08-01", "1976-01-01");
    assert_eq!(weeks(&short), "2");
}

#[test]
fn the_45th_year_of_age_is_refused() {
    let at = |born: &str| facts("2026-08-01", "2020-01-01", born);
    assert_eq!(run(&at("1981-08-02"))["over_45"], false);
    assert_eq!(weeks(&at("1981-08-02")), "4");
    assert_eq!(code(&at("1981-08-01")), "notice_period.age_unsettled");
    assert_eq!(code(&at("1980-08-02")), "notice_period.age_unsettled");
    assert_eq!(run(&at("1980-08-01"))["over_45"], true);
    assert_eq!(weeks(&at("1980-08-01")), "5");
    // Born on 29 February: the 45th and 46th birthdays in common years are
    // taken at their earliest and latest, 28 February and 1 March.
    let leap = |notice: &str| facts(notice, "2020-01-01", "1980-02-29");
    assert_eq!(code(&leap("2025-02-28")), "notice_period.age_unsettled");
    assert_eq!(run(&leap("2025-02-27"))["over_45"], false);
    assert_eq!(code(&leap("2026-02-28")), "notice_period.age_unsettled");
    assert_eq!(run(&leap("2026-03-01"))["over_45"], true);
}

#[test]
fn excluded_days_come_off_the_service() {
    let base = facts("2026-07-31", "2025-07-22", "1996-01-01");
    assert_eq!(weeks(&base), "2");
    let excluded = with(base, json!({"excluded_days": 10}));
    assert_eq!(weeks(&excluded), "1");
    assert_eq!(run(&excluded)["rates"]["days_counted"], "365");
}

#[test]
fn payment_in_lieu_is_the_full_rate_for_the_weeks() {
    let input = with(
        facts("2026-08-01", "2022-08-01", "1996-01-01"),
        json!({"full_rate_of_pay": "40.00", "weekly_hours": "38"}),
    );
    let result = run(&input);
    assert_eq!(result["rates"]["minimum_notice_weeks"], "3");
    assert_eq!(result["amounts"]["payment_in_lieu"], "4560.00");
    // Half up to the cent: 1 week x 7.25 hours x $33.33 = $241.6425.
    let odd = with(
        facts("2026-08-01", "2026-01-01", "1996-01-01"),
        json!({"full_rate_of_pay": "33.33", "weekly_hours": "7.25"}),
    );
    assert_eq!(run(&odd)["amounts"]["payment_in_lieu"], "241.64");
    for hours in ["0", "168.01", "1.001", "-1"] {
        let bad = with(odd.clone(), json!({"weekly_hours": hours}));
        assert_eq!(code(&bad), "notice_period.hours_invalid", "{hours}");
    }
    let alone = with(odd, json!({"weekly_hours": "remove"}));
    assert_eq!(code(&alone), "input.missing_field");
}

#[test]
fn each_s_123_exclusion_removes_the_notice() {
    let base = with(
        facts("2026-08-01", "2022-08-01", "1976-01-01"),
        json!({"full_rate_of_pay": "40.00", "weekly_hours": "38"}),
    );
    for (exclusion, provision) in [
        ("serious_misconduct", "s 123(1)(b)"),
        ("casual", "s 123(1)(c)"),
        ("training_arrangement", "s 123(1)(d)"),
        ("prescribed_division", "s 123(1)(e)"),
        ("daily_hire_building", "s 123(3)(b)"),
        ("daily_hire_meat", "s 123(3)(c)"),
        ("weekly_hire_meat_seasonal", "s 123(3)(d)"),
        ("prescribed_notice", "s 123(3)(e)"),
    ] {
        let result = run(&with(base.clone(), json!({"exclusion": exclusion})));
        assert_eq!(result["notice_required"], false, "{exclusion}");
        assert_eq!(result["rates"]["minimum_notice_weeks"], "0", "{exclusion}");
        assert_eq!(result["rates"]["earliest_termination_day"], "2026-08-01");
        assert_eq!(result["amounts"]["payment_in_lieu"], "0.00");
        assert!(
            result["provision"].as_str().unwrap().contains(provision),
            "{exclusion}"
        );
    }
    let fixed = with(base.clone(), json!({"exclusion": "specified_period"}));
    assert_eq!(code(&fixed), "input.missing_field");
    let honest = run(&with(fixed.clone(), json!({"to_avoid_division": false})));
    assert_eq!(honest["notice_required"], false);
    assert!(
        honest["provision"]
            .as_str()
            .unwrap()
            .contains("s 123(1)(a)")
    );
    let avoided = run(&with(fixed, json!({"to_avoid_division": true})));
    assert_eq!(avoided["notice_required"], true);
    assert_eq!(avoided["rates"]["minimum_notice_weeks"], "4");
    assert!(avoided["provision"].as_str().unwrap().contains("s 123(2)"));
    let stray = with(base.clone(), json!({"to_avoid_division": true}));
    assert_eq!(code(&stray), "input.field_unknown");
    let unknown = with(base, json!({"exclusion": "contractor"}));
    assert_eq!(code(&unknown), "notice_period.exclusion_invalid");
}

#[test]
fn dates_outside_the_text_or_the_year_are_refused() {
    let early = facts("2021-03-26", "2019-01-01", "1980-01-01");
    assert_eq!(code(&early), "notice_period.before_current_text");
    assert_eq!(weeks(&facts("2021-03-27", "2019-01-01", "1980-01-01")), "2");
    let elsewhere = with(
        facts("2026-08-01", "2020-01-01", "1990-01-01"),
        json!({"year": "2025-26"}),
    );
    assert_eq!(code(&elsewhere), "notice_period.year_mismatch");
    let later = facts("2026-08-01", "2026-08-02", "1990-01-01");
    assert_eq!(code(&later), "notice_period.start_after_notice");
    let unborn = facts("2026-08-01", "2020-01-01", "2020-01-02");
    assert_eq!(code(&unborn), "notice_period.birth_after_start");
    let bad_date = with(
        facts("2026-08-01", "2020-01-01", "1990-01-01"),
        json!({"notice_date": "1/8/2026"}),
    );
    assert_eq!(code(&bad_date), "input.date_invalid");
    for days in [json!(-1), json!(2405), json!("3"), json!(36_501)] {
        let bad = with(
            facts("2026-08-01", "2020-01-01", "1990-01-01"),
            json!({"excluded_days": days}),
        );
        assert_eq!(code(&bad), "notice_period.excluded_invalid", "{days}");
    }
    let flag = with(
        facts("2026-08-01", "2020-01-01", "1990-01-01"),
        json!({"exclusion": "specified_period", "to_avoid_division": "no"}),
    );
    assert_eq!(code(&flag), "input.flag_invalid");
    let unconfirmed = with(
        facts("2026-08-01", "2020-01-01", "1990-01-01"),
        json!({"scope_confirmed": false}),
    );
    assert_eq!(code(&unconfirmed), "scope.not_confirmed");
}
