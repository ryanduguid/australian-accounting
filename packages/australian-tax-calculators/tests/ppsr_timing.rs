// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! PPSR timing and fees (PPSA ss 10, 62 and 153; Personal Property Securities
//! (Fees) Determination 2018 s 4; Corporations Act ss 9 and 588FL, read on
//! 20 November 2026). Neither the Registrar nor the Acts publish a worked
//! example with dates, so every case is constructed and checked by hand
//! against the calendar (docs/calculation-evidence.md#ppsr-timing).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(question: &str, year: &str, pairs: &[(&str, Value)]) -> Value {
    let mut input = json!({"question": question, "year": year, "scope_confirmed": true});
    for (key, value) in pairs {
        input[*key] = value.clone();
    }
    input
}

fn run(input: &Value) -> Value {
    api::evaluate("ppsr-timing", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("ppsr-timing", input).unwrap_err().code
}

fn registration(time: &str, collateral: &str, serial: bool, end: Option<&str>) -> Value {
    let mut pairs = vec![
        ("registration_time", json!(time)),
        ("collateral", json!(collateral)),
        ("serial_numbered", json!(serial)),
    ];
    if let Some(end) = end {
        pairs.push(("end_date", json!(end)));
    }
    input("registration", "2026-27", &pairs)
}

/// s 153 item 5 and the Fees Determination s 4 items 1 to 3: each tier.
#[test]
fn end_times_and_fees_follow_each_tier() {
    // Consumer property: 7 years at most, the default when none is asked for.
    let r = run(&registration("2026-11-19T10:00", "consumer", false, None));
    assert_eq!(r["latest_end_time"], "2033-11-19 end of day");
    assert_eq!(r["end_time"], "2033-11-19 end of day");
    assert_eq!(r["end_time_basis"], "default_time");
    assert_eq!(r["amounts"]["registration_fee"], "6.00");
    // Commercial with no end time.
    let r = run(&registration("2026-11-19T10:00", "commercial", false, None));
    assert_eq!(r["end_time"], Value::Null);
    assert_eq!(r["end_time_basis"], "no_end_time");
    assert_eq!(r["latest_end_time"], "2051-11-19 end of day");
    assert_eq!(r["amounts"]["registration_fee"], "115.00");
    // Commercial, 10 years: the middle tier.
    let r = run(&registration(
        "2026-11-19T10:00",
        "commercial",
        false,
        Some("2036-11-19"),
    ));
    assert_eq!(r["end_time"], "2036-11-19 end of day");
    assert_eq!(r["end_time_basis"], "requested");
    assert_eq!(r["amounts"]["registration_fee"], "25.00");
    // Commercial ending on the 7 year day itself is still the $6.00 tier.
    let r = run(&registration(
        "2026-11-19T10:00",
        "commercial",
        false,
        Some("2033-11-19"),
    ));
    assert_eq!(r["amounts"]["registration_fee"], "6.00");
    let r = run(&registration(
        "2026-11-19T10:00",
        "commercial",
        false,
        Some("2033-11-20"),
    ));
    assert_eq!(r["amounts"]["registration_fee"], "25.00");
    // The published schedule rides on every answer.
    assert_eq!(r["rates"]["search_online"], "2.00");
    assert_eq!(r["rates"]["search_contact_centre"], "7.00");
    assert_eq!(r["rates"]["maintenance_of_registration"], "4.00");
    assert_eq!(r["rates"]["registrar_report_per_hour"], "300.00");
}

/// s 153(2) and (3): an end time past the default time, or before the
/// registration time, is taken to be the default time.
#[test]
fn a_non_complying_end_time_becomes_the_default_time() {
    let r = run(&registration(
        "2026-11-19T10:00",
        "commercial",
        true,
        Some("2040-01-01"),
    ));
    assert_eq!(r["end_time"], "2033-11-19 end of day");
    assert_eq!(r["end_time_basis"], "default_time");
    assert_eq!(r["amounts"]["registration_fee"], "6.00");
    let r = run(&registration(
        "2026-11-19T10:00",
        "commercial",
        false,
        Some("2060-01-01"),
    ));
    assert_eq!(r["end_time"], "2051-11-19 end of day");
    assert_eq!(r["amounts"]["registration_fee"], "25.00");
    let r = run(&registration(
        "2026-11-19T10:00",
        "commercial",
        false,
        Some("2026-11-18"),
    ));
    assert_eq!(r["end_time_basis"], "default_time");
    assert_eq!(r["end_time"], "2051-11-19 end of day");
    assert!(
        r["end_time_note"]
            .as_str()
            .unwrap()
            .contains("s 153(2) and (3)")
    );
    // The same day ends after the registration time, so it complies.
    let r = run(&registration(
        "2026-11-19T10:00",
        "commercial",
        false,
        Some("2026-11-19"),
    ));
    assert_eq!(r["end_time_basis"], "requested");
}

/// The catalogue's example runs.
#[test]
fn the_catalogue_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["ppsr_timing"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = run(&facts);
    assert_eq!(r["end_time"], "2033-03-02 end of day");
    assert_eq!(r["amounts"]["registration_fee"], "6.00");
}

fn pmsi(inventory: bool, goods: bool, trigger: &str, more: &[(&str, Value)]) -> Value {
    let mut pairs = vec![
        ("inventory", json!(inventory)),
        ("goods", json!(goods)),
        ("trigger_time", json!(trigger)),
    ];
    pairs.extend(more.iter().cloned());
    input("pmsi", "2026-27", &pairs)
}

#[test]
fn refuses_the_skipped_and_repeated_act_hours() {
    for time in [
        "2026-10-04T02:00",
        "2026-10-04T02:59",
        "2027-04-04T02:00",
        "2027-04-04T02:59",
    ] {
        assert_eq!(
            code(&registration(time, "consumer", false, None)),
            "ppsr_timing.time_unresolved",
            "{time}"
        );
    }
    for time in [
        "2026-10-04T01:59",
        "2026-10-04T03:00",
        "2027-04-04T01:59",
        "2027-04-04T03:00",
        "2027-04-11T02:30",
    ] {
        assert_eq!(
            run(&registration(time, "consumer", false, None))["registration_time"],
            time
        );
    }
}

#[test]
fn tests_only_unique_registration_times_for_inventory_priority() {
    let priority = |at: &str| {
        pmsi(
            true,
            true,
            "2027-04-04T03:00",
            &[
                ("registration_time", json!(at)),
                ("pmsi_stated", json!(true)),
            ],
        )
    };
    assert_eq!(
        code(&priority("2027-04-04T02:25")),
        "ppsr_timing.time_unresolved"
    );
    assert_eq!(
        run(&priority("2027-04-04T01:59"))["registered_in_time"],
        true
    );
    assert_eq!(
        run(&priority("2027-04-04T03:01"))["registered_in_time"],
        false
    );
}

/// s 62(3)(b) with s 10: possession on Monday 21 December 2026. 22, 23 and
/// 24 December count; 25 December and 1 January are listed holidays; 26 to
/// 31 December fall between Christmas Day and New Year's Day; the 15th
/// business day is Tuesday 19 January 2027.
#[test]
fn the_15_business_days_skip_the_christmas_break() {
    let holidays = json!(["2026-12-25", "2027-01-01"]);
    let r = run(&pmsi(
        false,
        true,
        "2026-12-21T09:00",
        &[("excluded_days", holidays.clone())],
    ));
    assert_eq!(r["register_by"], "2027-01-19 end of day");
    let on_time = pmsi(
        false,
        true,
        "2026-12-21T09:00",
        &[
            ("excluded_days", holidays.clone()),
            ("registration_time", json!("2027-01-19T23:59")),
            ("pmsi_stated", json!(true)),
        ],
    );
    let r = run(&on_time);
    assert_eq!(r["registered_in_time"], json!(true));
    assert_eq!(r["super_priority"], json!(true));
    let late = pmsi(
        false,
        true,
        "2026-12-21T09:00",
        &[
            ("excluded_days", holidays.clone()),
            ("registration_time", json!("2027-01-20T00:00")),
            ("pmsi_stated", json!(true)),
        ],
    );
    assert_eq!(run(&late)["registered_in_time"], json!(false));
    // In time but without the s 153 item 7 statement: no super-priority.
    let unstated = pmsi(
        false,
        true,
        "2026-12-21T09:00",
        &[
            ("excluded_days", holidays),
            ("registration_time", json!("2026-12-22T09:00")),
            ("pmsi_stated", json!(false)),
        ],
    );
    let r = run(&unstated);
    assert_eq!(r["registered_in_time"], json!(true));
    assert_eq!(r["super_priority"], json!(false));
}

/// s 62(3)(b): an ordinary fortnight with no holiday, counted from the day
/// after the trigger: possession on Monday 2 November 2026 gives Monday
/// 23 November.
#[test]
fn a_plain_count_runs_15_weekdays() {
    let r = run(&pmsi(
        false,
        true,
        "2026-11-02T15:00",
        &[("excluded_days", json!([]))],
    ));
    assert_eq!(r["register_by"], "2026-11-23 end of day");
    // A whole-state holiday listed moves it a day.
    let r = run(&pmsi(
        false,
        true,
        "2026-11-02T15:00",
        &[("excluded_days", json!(["2026-11-10"]))],
    ));
    assert_eq!(r["register_by"], "2026-11-24 end of day");
}

/// s 62(2)(b): inventory must be registered by the time of possession.
#[test]
fn inventory_must_be_registered_by_possession() {
    let at = |reg: &str| {
        pmsi(
            true,
            true,
            "2026-11-02T15:00",
            &[
                ("registration_time", json!(reg)),
                ("pmsi_stated", json!(true)),
            ],
        )
    };
    let r = run(&at("2026-11-02T15:00"));
    assert_eq!(r["register_by"], "2026-11-02T15:00");
    assert_eq!(r["super_priority"], json!(true));
    assert_eq!(run(&at("2026-11-02T15:01"))["super_priority"], json!(false));
    // Inventory takes no holiday list.
    let listed = pmsi(
        true,
        true,
        "2026-11-02T15:00",
        &[("excluded_days", json!([]))],
    );
    assert_eq!(code(&listed), "input.field_unknown");
}

fn company(agreed: &str, holidays: Value, more: &[(&str, Value)]) -> Value {
    let mut pairs = vec![
        ("agreement_in_force_time", json!(agreed)),
        ("place_holidays", holidays),
        ("foreign_law_agreement", json!(false)),
    ];
    pairs.extend(more.iter().cloned());
    input("company_grantor", "2026-27", &pairs)
}

#[test]
fn refuses_a_derived_six_month_time_in_the_repeated_hour() {
    let input = company(
        "2026-09-01T11:00",
        json!([]),
        &[("critical_time", json!("2026-10-05T02:30"))],
    );
    assert_eq!(code(&input), "ppsr_timing.time_unresolved");
}

/// s 588FL(2)(b)(ii) with Corporations Act s 9: an agreement in force on
/// Monday 12 October 2026. Where 3 November is a holiday in the place
/// concerned the 20th business day is 10 November; elsewhere it is 9 November.
#[test]
fn the_20_business_days_turn_on_the_place() {
    let melbourne = json!(["2026-11-03"]);
    let r = run(&company("2026-10-12T11:00", melbourne.clone(), &[]));
    assert_eq!(r["register_by"], "2026-11-10 end of day");
    let r = run(&company("2026-10-12T11:00", json!([]), &[]));
    assert_eq!(r["register_by"], "2026-11-09 end of day");
    // With a critical time of 20 November 2026 the latest time is the 20 day
    // time, as 6 months before (20 May) is earlier.
    let tested = |holidays: &Value, reg: &str| {
        run(&company(
            "2026-10-12T11:00",
            holidays.clone(),
            &[
                ("critical_time", json!("2026-11-20T10:00")),
                ("registration_time", json!(reg)),
            ],
        ))
    };
    let r = tested(&melbourne, "2026-11-10T16:00");
    assert_eq!(r["six_months_before"], "2026-05-20T10:00");
    assert_eq!(r["latest_time"], "2026-11-10 end of day");
    assert_eq!(r["vests"], json!(false));
    assert_eq!(r["vesting"], "does_not_vest");
    let r = tested(&melbourne, "2026-11-11T09:00");
    assert_eq!(r["vests"], json!(true));
    assert_eq!(r["vesting"], "vests");
    let r = tested(&json!([]), "2026-11-10T16:00");
    assert_eq!(r["vests"], json!(true));
}

/// A critical time after compilation 149 ends (30 November 2026): ss 9 and
/// 588FL are unamended by every later version the Register lists. Agreement
/// in force Monday 2 November 2026, Melbourne Cup Day (3 November) listed:
/// the 20th business day is 1 December; with a critical time of 1 March 2027
/// (6 months before: 1 September 2026) that is the latest time.
#[test]
fn a_critical_time_from_1_december_2026_is_worked() {
    let tested = |reg: &str| {
        run(&company(
            "2026-11-02T09:00",
            json!(["2026-11-03"]),
            &[
                ("critical_time", json!("2027-03-01T10:00")),
                ("registration_time", json!(reg)),
            ],
        ))
    };
    let r = tested("2026-11-25T12:00");
    assert_eq!(r["register_by"], "2026-12-01 end of day");
    assert_eq!(r["six_months_before"], "2026-09-01T10:00");
    assert_eq!(r["latest_time"], "2026-12-01 end of day");
    assert_eq!(r["vests"], json!(false));
    assert_eq!(tested("2026-12-01T23:59")["vests"], json!(false));
    assert_eq!(tested("2026-12-02T09:00")["vests"], json!(true));
}

/// s 588FL(2)(b)(i): a registration more than 6 months before the critical
/// time is safe however late after the agreement it was. Agreement 2 February
/// 2026, critical time 1 November 2026: the latest time is 1 May 2026.
#[test]
fn six_months_before_the_critical_time_saves_an_old_registration() {
    let tested = |reg: &str| {
        run(&input(
            "company_grantor",
            "2025-26",
            &[
                ("agreement_in_force_time", json!("2026-02-02T09:00")),
                ("place_holidays", json!([])),
                ("foreign_law_agreement", json!(false)),
                ("critical_time", json!("2026-11-01T10:00")),
                ("registration_time", json!(reg)),
            ],
        ))
    };
    let r = tested("2026-04-15T12:00");
    assert_eq!(r["latest_time"], "2026-05-01T10:00");
    assert_eq!(r["vests"], json!(false));
    assert_eq!(tested("2026-05-01T10:00")["vests"], json!(false));
    assert_eq!(tested("2026-05-01T10:01")["vests"], json!(true));
    // Registered after the critical time: not perfected then, so not s 588FL.
    let r = tested("2026-11-02T10:00");
    assert_eq!(r["vests"], Value::Null);
    assert_eq!(r["vesting"], "registered_after_critical_time");
}

/// s 588FL(2)(b)(ii): the critical time caps the 20 day time, and a court
/// ordered later time (s 588FM) can be the latest.
#[test]
fn the_critical_time_and_a_court_order_move_the_latest_time() {
    let r = run(&company(
        "2026-11-02T09:00",
        json!([]),
        &[("critical_time", json!("2026-11-20T12:00"))],
    ));
    assert_eq!(r["register_by"], "2026-11-30 end of day");
    assert_eq!(r["latest_time"], "2026-11-20T12:00");
    let r = run(&company(
        "2026-11-02T09:00",
        json!([]),
        &[
            ("critical_time", json!("2026-11-20T12:00")),
            ("court_ordered_time", json!("2026-11-25T17:00")),
        ],
    ));
    assert_eq!(r["latest_time"], "2026-11-25T17:00");
}

/// Each refusal.
#[test]
fn refusals() {
    // A year the command does not cover.
    let mut later = registration("2027-11-19T10:00", "consumer", false, None);
    later["year"] = json!("2027-28");
    assert_eq!(code(&later), "period.unsupported");
    // The registration is outside the year given.
    let outside = registration("2026-06-30T10:00", "consumer", false, None);
    assert_eq!(code(&outside), "ppsr_timing.date_outside_year");
    // Times and lists.
    let bad = registration("2026-11-19 10:00", "consumer", false, None);
    assert_eq!(code(&bad), "ppsr_timing.time_invalid");
    let bad = registration("2026-11-19T24:00", "consumer", false, None);
    assert_eq!(code(&bad), "ppsr_timing.time_invalid");
    let bad = pmsi(
        false,
        true,
        "2026-11-02T15:00",
        &[("excluded_days", json!(["3 Nov"]))],
    );
    assert_eq!(code(&bad), "ppsr_timing.days_invalid");
    // A count across Christmas without 25 December listed.
    let gap = pmsi(
        false,
        true,
        "2026-12-21T09:00",
        &[("excluded_days", json!(["2027-01-01"]))],
    );
    assert_eq!(code(&gap), "ppsr_timing.holidays_incomplete");
    // Values outside the lists.
    assert_eq!(
        code(&input("search", "2026-27", &[])),
        "ppsr_timing.fact_invalid"
    );
    let bad = registration("2026-11-19T10:00", "household", false, None);
    assert_eq!(code(&bad), "ppsr_timing.fact_invalid");
    // A foreign-law agreement, a critical time outside the years read, a
    // 6 month day that does not exist.
    let foreign = input(
        "company_grantor",
        "2026-27",
        &[
            ("agreement_in_force_time", json!("2026-11-02T09:00")),
            ("place_holidays", json!([])),
            ("foreign_law_agreement", json!(true)),
        ],
    );
    assert_eq!(code(&foreign), "ppsr_timing.foreign_law_unsupported");
    for outside in ["2025-06-30T23:59", "2027-07-01T00:00"] {
        let unread = company(
            "2026-11-02T09:00",
            json!([]),
            &[("critical_time", json!(outside))],
        );
        assert_eq!(code(&unread), "ppsr_timing.law_unread");
    }
    let edge = company(
        "2026-11-02T09:00",
        json!([]),
        &[("critical_time", json!("2027-06-30T23:59"))],
    );
    assert!(run(&edge)["latest_time"].is_string());
    let short = company(
        "2026-07-01T09:00",
        json!([]),
        &[("critical_time", json!("2026-08-31T10:00"))],
    );
    assert_eq!(code(&short), "ppsr_timing.date_unsettled");
    // Fields another question takes, and missing ones.
    let mixed = registration("2026-11-19T10:00", "consumer", false, None);
    let mixed = {
        let mut m = mixed;
        m["critical_time"] = json!("2026-11-20T10:00");
        m
    };
    assert_eq!(code(&mixed), "input.field_unknown");
    let court_alone = company(
        "2026-11-02T09:00",
        json!([]),
        &[("court_ordered_time", json!("2026-11-25T17:00"))],
    );
    assert_eq!(code(&court_alone), "input.field_unknown");
    let missing = input("pmsi", "2026-27", &[("inventory", json!(false))]);
    assert_eq!(code(&missing), "input.missing_field");
    let unstated = pmsi(
        true,
        true,
        "2026-11-02T15:00",
        &[("registration_time", json!("2026-11-02T14:00"))],
    );
    assert_eq!(code(&unstated), "input.missing_field");
    let mut unconfirmed = registration("2026-11-19T10:00", "consumer", false, None);
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(code(&unconfirmed), "scope.not_confirmed");
}

#[test]
fn warns_that_omitted_holidays_can_bring_a_deadline_forward() {
    let facts = [
        ("goods", json!(true)),
        ("inventory", json!(false)),
        ("trigger_time", json!("2026-05-28T10:00")),
    ];
    let mut input = input("pmsi", "2025-26", &facts);
    input["excluded_days"] = json!(["2026-06-01", "2026-06-08"]);
    let complete = run(&input);
    input["excluded_days"] = json!(["2026-06-08"]);
    let incomplete = run(&input);

    assert_eq!(complete["register_by"], "2026-06-22 end of day");
    assert_eq!(incomplete["register_by"], "2026-06-19 end of day");
    let warnings = incomplete["warnings"].as_array().unwrap();
    assert!(warnings.iter().filter_map(Value::as_str).any(|warning| {
        warning.contains("Omitting an excluded date can make the calculated deadline too early.")
    }));
}

/// For property other than goods the 15 business days run from when the
/// interest attaches, not from possession.
#[test]
fn intangibles_run_from_attachment() {
    let holidays = json!(["2026-12-25", "2027-01-01"]);
    let r = run(&pmsi(
        false,
        false,
        "2026-12-21T09:00",
        &[("excluded_days", holidays)],
    ));
    assert_eq!(r["register_by"], "2027-01-19 end of day");
    assert!(r.to_string().contains("the interest attaches"));
}

/// 2 am on the first Sunday of a month without a daylight saving change is
/// an ordinary time.
#[test]
fn two_am_on_another_first_sunday_is_ordinary() {
    let time = "2027-03-07T02:30";
    assert_eq!(
        run(&registration(time, "consumer", false, None))["registration_time"],
        time
    );
}

/// Excluded days that are not a list and an end date that is not a date are
/// refused.
#[test]
fn malformed_days_and_dates_are_refused() {
    let not_list = pmsi(
        false,
        true,
        "2026-12-21T09:00",
        &[("excluded_days", json!("2026-12-25"))],
    );
    assert_eq!(code(&not_list), "ppsr_timing.days_invalid");
    let bad_end = registration("2026-11-19T10:00", "commercial", false, Some("2030-02-30"));
    assert_eq!(code(&bad_end), "input.date_invalid");
}
