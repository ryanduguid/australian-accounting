// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Work expenses from the records themselves: an hours diary for the
//! working from home fixed rate (PCG 2023/1 paras 26(a), 56 to 59) and a car
//! log book's period and journeys (ITAA 1997 ss 28-90(5), 28-115, 28-120 and
//! 28-125), against PCG 2023/1's Gerry, the ATO's Keisha and Lana, and each
//! boundary and refusal, all read on 24 November 2026.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("work-expenses", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("work-expenses", input)
        .unwrap_err()
        .code
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

fn amount<'a>(result: &'a Value, id: &str) -> &'a str {
    result["amounts"][id]
        .as_str()
        .unwrap_or_else(|| panic!("no {id} in {}", result["amounts"]))
}

/// A home office claim with stated `hours`, nothing else entered.
fn home(year: &str, hours: &str) -> Value {
    json!({
        "method": "home_office", "reimbursed": false, "allowance_received": false,
        "hours": hours, "energy": "0.00", "phone": "0.00", "internet": "0.00",
        "stationery": "0.00", "cleaning": "0.00", "furniture_decline": "0.00",
        "asset_decline": "0.00", "repairs": "0.00", "running_expenses_incurred": true,
        "year": year, "scope_confirmed": true,
    })
}

/// `days` consecutive dates from `first`, each with `hours`.
fn rows(first: &str, days: usize, hours: &str) -> Vec<Value> {
    let start = chrono_day(first);
    (0..days)
        .map(|i| json!({"date": day_text(start + i as i64), "hours": hours}))
        .collect()
}

/// The same claim from a diary instead of `hours`.
fn diary(year: &str, entries: Vec<Value>) -> Value {
    with(
        home(year, "0"),
        json!({"hours": "remove", "hours_diary": entries}),
    )
}

// A small civil calendar for building fixtures (days since 1970-01-01).
fn chrono_day(text: &str) -> i64 {
    let y: i64 = text[..4].parse().unwrap();
    let m: i64 = text[5..7].parse().unwrap();
    let d: i64 = text[8..10].parse().unwrap();
    let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn day_text(day: i64) -> String {
    let z = day + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

#[test]
fn a_diary_gives_the_same_claim_as_its_total_hours() {
    // PCG 2023/1 example 1, Gerry: 768 hours x 70c in 2024-25, $537.
    let gerry = run(&diary("2024-25", rows("2024-07-01", 96, "8")));
    let stated = run(&home("2024-25", "768"));
    assert_eq!(amount(&gerry, "diary_hours"), "768.00");
    for id in [
        "fixed_rate_amount",
        "fixed_rate_return_amount",
        "actual_cost_method",
    ] {
        assert_eq!(amount(&gerry, id), amount(&stated, id), "{id}");
    }
    assert_eq!(amount(&gerry, "fixed_rate_return_amount"), "537.00");
    assert_eq!(gerry["diary_days"], 96);
    assert_eq!(gerry["diary_first_date"], "2024-07-01");
    assert_eq!(gerry["diary_last_date"], "2024-10-04");
    // The ATO's Keisha: 112 days at 7.5 hours and one at 3, 843 hours, $590.
    let mut entries = rows("2025-07-01", 112, "7.5");
    entries.push(json!({"date": "2026-06-30", "hours": "3"}));
    let keisha = run(&diary("2025-26", entries));
    assert_eq!(amount(&keisha, "fixed_rate_amount"), "590.10");
    assert_eq!(amount(&keisha, "fixed_rate_return_amount"), "590.00");
    assert_eq!(keisha["diary_last_date"], "2026-06-30");
}

#[test]
fn several_entries_on_one_date_are_added_up_to_24_hours() {
    let sessions = vec![
        json!({"date": "2025-08-04", "hours": "2.5"}),
        json!({"date": "2025-08-04", "hours": "1.25"}),
        json!({"date": "2025-08-05", "hours": "4"}),
    ];
    let result = run(&diary("2025-26", sessions));
    assert_eq!(amount(&result, "diary_hours"), "7.75");
    assert_eq!(result["diary_days"], 2);
    let full = vec![json!({"date": "2025-08-04", "hours": "24"})];
    assert_eq!(
        amount(&run(&diary("2025-26", full)), "diary_hours"),
        "24.00"
    );
    let over = vec![
        json!({"date": "2025-08-04", "hours": "20"}),
        json!({"date": "2025-08-04", "hours": "4.01"}),
    ];
    assert_eq!(code(&diary("2025-26", over)), "work_expenses.fact_conflict");
}

#[test]
fn a_diary_row_must_fall_in_the_year_and_be_well_formed() {
    let cases = [
        (
            json!([{"date": "2026-07-01", "hours": "1"}]),
            "work_expenses.fact_conflict",
        ),
        (
            json!([{"date": "2025-06-30", "hours": "1"}]),
            "work_expenses.fact_conflict",
        ),
        (
            json!([{"date": "2025-02-30", "hours": "1"}]),
            "work_expenses.fact_invalid",
        ),
        (
            json!([{"date": "2025-08-04", "hours": "0"}]),
            "work_expenses.fact_invalid",
        ),
        (
            json!([{"date": "2025-08-04", "hours": "-1"}]),
            "input.amount_invalid",
        ),
        (
            json!([{"date": "2025-08-04", "hours": "1.001"}]),
            "input.amount_invalid",
        ),
        (
            json!([{"date": "2025-08-04", "hours": 1}]),
            "input.amount_invalid",
        ),
        (json!([{"date": "2025-08-04"}]), "input.missing_field"),
        (
            json!([{"date": "2025-08-04", "hours": "1", "note": "x"}]),
            "input.field_unknown",
        ),
        (json!([]), "work_expenses.fact_invalid"),
        (json!({"date": "2025-08-04"}), "work_expenses.fact_invalid"),
        (json!(["2025-08-04"]), "work_expenses.fact_invalid"),
    ];
    for (entries, expected) in cases {
        let input = with(
            home("2025-26", "0"),
            json!({"hours": "remove", "hours_diary": entries}),
        );
        assert_eq!(code(&input), expected, "{entries}");
    }
    let edge = vec![json!({"date": "2025-07-01", "hours": "1"})];
    assert_eq!(amount(&run(&diary("2025-26", edge)), "diary_hours"), "1.00");
}

#[test]
fn hours_and_a_diary_are_not_both_given_and_the_diary_needs_pcg_2023_1() {
    let both = with(
        home("2025-26", "10"),
        json!({"hours_diary": [{"date": "2025-08-04", "hours": "1"}]}),
    );
    assert_eq!(code(&both), "work_expenses.fact_conflict");
    for year in ["2020-21", "2021-22"] {
        let mut early = diary(year, rows(&format!("{}-08-02", &year[..4]), 2, "1"));
        early["dedicated_home_office"] = json!(true);
        assert_eq!(
            code(&early),
            "work_expenses.diary_year_unsupported",
            "{year}"
        );
    }
    let first = diary("2022-23", rows("2022-07-01", 1, "1"));
    assert_eq!(run(&first)["rates"]["fixed_rate"], "0.67");
}

#[test]
fn the_fixed_rate_records_are_reported_as_facts_to_confirm() {
    let stated = run(&home("2025-26", "100"));
    let records = stated["records_to_confirm"].as_array().unwrap();
    let text = records
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("PCG 2023/1 para 58"));
    assert!(text.contains("para 28"));
    assert!(text.contains("paras 56 and 60 to 61"));
    assert!(!text.contains("para 57"));
    let beside = run(&with(
        home("2025-26", "100"),
        json!({"asset_decline": "10.00"}),
    ));
    assert!(beside["records_to_confirm"].to_string().contains("para 57"));
    let first = run(&home("2022-23", "100"));
    let first = first["records_to_confirm"].to_string();
    assert!(first.contains("para 59") && !first.contains("para 58"));
    // No hourly rate, no hourly records; before 2022-23, none reported.
    let none = with(
        home("2025-26", "100"),
        json!({"running_expenses_incurred": false}),
    );
    assert!(run(&none)["records_to_confirm"].is_null());
    let mut early = home("2021-22", "100");
    early["dedicated_home_office"] = json!(true);
    assert!(run(&early)["records_to_confirm"].is_null());
}

/// Lana's 2025-26 claim, with `changes`, from a stated log book.
fn stated_log(changes: Value) -> Value {
    with(
        json!({
            "method": "car_log_book", "reimbursed": false, "allowance_received": false,
            "owned_or_leased": true, "business_kilometres": "69", "total_kilometres": "100",
            "car_expenses": "17275.33", "log_book_year": "2025-26", "log_book_weeks": 12,
            "log_book_representative": true,
            "year": "2025-26", "scope_confirmed": true,
        }),
        changes,
    )
}

/// A log book from 1 August 2025 for `days` days, odometer 10,000 to 13,000,
/// with 3 journeys of 1,000, 570 and 500 km (2,070 km, 69%).
fn log(days: i64) -> Value {
    let end = day_text(chrono_day("2025-08-01") + days - 1);
    json!({
        "period_start": "2025-08-01", "period_end": end,
        "odometer_start": "10000", "odometer_end": "13000",
        "journeys": [
            {"start": "2025-08-04", "end": "2025-08-05", "odometer_start": "10100",
             "odometer_end": "11100", "purpose": "Site inspections, Bendigo"},
            {"start": "2025-08-20", "end": "2025-08-20", "odometer_start": "11500",
             "odometer_end": "12070", "purpose": "Client meetings, Geelong and Ballarat"},
            {"start": "2025-09-15", "end": "2025-09-15", "odometer_start": "12200",
             "odometer_end": "12700", "purpose": "Training at head office"},
        ],
    })
}

/// Lana's claim from the log book itself, 20,000 km in the year.
fn from_log(log: Value) -> Value {
    stated_log(json!({
        "business_kilometres": "remove", "log_book_weeks": "remove",
        "total_kilometres": "20000", "log_book": log,
    }))
}

#[test]
fn a_log_book_gives_lanas_69_per_cent() {
    // ATO 'Logbook method', Lana: $17,275.33 x 69% = $11,919.98.
    let lana = run(&from_log(log(84)));
    assert_eq!(amount(&lana, "log_book_total_kilometres"), "3000.00");
    assert_eq!(amount(&lana, "log_book_business_kilometres"), "2070.00");
    assert_eq!(amount(&lana, "log_book_percentage"), "69.00");
    assert_eq!(amount(&lana, "business_kilometres"), "13800.00");
    assert_eq!(amount(&lana, "business_use_percentage"), "69.00");
    assert_eq!(amount(&lana, "deduction"), "11919.98");
    assert_eq!(amount(&lana, "return_amount"), "11919.00");
    assert_eq!(lana["log_book_weeks"], 12);
    assert_eq!(lana["log_book_days"], 84);
    assert_eq!(lana["log_book_journeys"], 3);
    assert_eq!(lana["log_book_valid_to"], "2029-30");
    let stated = run(&stated_log(json!({})));
    for id in ["deduction", "return_amount"] {
        assert_eq!(amount(&lana, id), amount(&stated, id), "{id}");
    }
    let records = lana["records_to_confirm"].to_string();
    for cited in [
        "s 28-100(1)",
        "s 28-100(3)",
        "s 28-100(4)",
        "s 28-115(3)",
        "s 28-120(3)",
        "s 28-125(2)",
        "s 28-90(4)",
        "s 28-150",
    ] {
        assert!(records.contains(cited), "{cited}");
        assert!(stated["records_to_confirm"].to_string().contains(cited));
    }
}

#[test]
fn a_log_book_period_is_12_weeks_and_includes_part_of_its_year() {
    assert_eq!(code(&from_log(log(83))), "work_expenses.log_book_too_short");
    assert_eq!(run(&from_log(log(90)))["log_book_weeks"], 12);
    assert_eq!(run(&from_log(log(91)))["log_book_weeks"], 13);
    // Kept for 2025-26, a period ending 30 June 2025 has no part of it.
    let mut before = log(84);
    before["period_start"] = json!("2025-04-01");
    before["period_end"] = json!("2025-06-30");
    for journey in before["journeys"].as_array_mut().unwrap() {
        journey["start"] = json!("2025-05-01");
        journey["end"] = json!("2025-05-01");
    }
    let mut overlap = before.clone();
    overlap["period_end"] = json!("2025-07-01");
    assert_eq!(
        code(&from_log(before)),
        "work_expenses.log_book_outside_year"
    );
    assert_eq!(
        amount(&run(&from_log(overlap)), "log_book_percentage"),
        "69.00"
    );
    // The 5 year rule and representativeness still apply.
    let old = with(from_log(log(84)), json!({"log_book_year": "2020-21"}));
    assert_eq!(code(&old), "work_expenses.log_book_expired");
    let changed = with(from_log(log(84)), json!({"log_book_representative": false}));
    assert_eq!(code(&changed), "work_expenses.log_book_unrepresentative");
}

#[test]
fn each_journey_is_recorded_inside_the_period_without_overlap() {
    let edit = |journey: usize, key: &str, value: Value| {
        let mut changed = log(84);
        changed["journeys"][journey][key] = value;
        code(&from_log(changed))
    };
    assert_eq!(
        edit(0, "purpose", json!(" ")),
        "work_expenses.log_book_entry_incomplete"
    );
    assert_eq!(
        edit(0, "purpose", json!(null)),
        "work_expenses.log_book_entry_incomplete"
    );
    assert_eq!(
        edit(0, "start", json!("2025-07-31")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit(2, "end", json!("2025-10-24")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit(0, "end", json!("2025-08-03")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit(0, "odometer_start", json!("9999")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit(2, "odometer_end", json!("13001")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit(0, "odometer_end", json!("10100")),
        "work_expenses.fact_conflict"
    );
    // Overlapping odometer readings, and a later reading on an earlier day.
    assert_eq!(
        edit(1, "odometer_start", json!("11000")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit(2, "start", json!("2025-08-10")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit(0, "start", json!("2025-13-01")),
        "work_expenses.fact_invalid"
    );
    assert_eq!(
        edit(0, "odometer_start", json!(10100)),
        "input.amount_invalid"
    );
    assert_eq!(edit(0, "colour", json!("red")), "input.field_unknown");
    // Journeys may be given in any order; odometer order is used.
    let mut shuffled = log(84);
    shuffled["journeys"].as_array_mut().unwrap().reverse();
    assert_eq!(
        amount(&run(&from_log(shuffled)), "log_book_percentage"),
        "69.00"
    );
}

#[test]
fn the_period_odometer_and_shape_are_checked() {
    let edit = |key: &str, value: Value| {
        let mut changed = log(84);
        if value == "remove" {
            changed.as_object_mut().unwrap().remove(key);
        } else {
            changed[key] = value;
        }
        code(&from_log(changed))
    };
    assert_eq!(
        edit("odometer_end", json!("10000")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit("period_end", json!("2025-07-31")),
        "work_expenses.fact_conflict"
    );
    assert_eq!(
        edit("period_start", json!("1 Aug 2025")),
        "work_expenses.fact_invalid"
    );
    assert_eq!(edit("journeys", json!([])), "work_expenses.fact_invalid");
    assert_eq!(edit("journeys", json!({})), "work_expenses.fact_invalid");
    assert_eq!(
        edit("odometer_start", json!("remove")),
        "input.missing_field"
    );
    assert_eq!(edit("make", json!("x")), "input.field_unknown");
    assert_eq!(code(&from_log(json!([1]))), "work_expenses.fact_invalid");
    // The log book replaces log_book_weeks and business_kilometres.
    let weeks = with(from_log(log(84)), json!({"log_book_weeks": 12}));
    assert_eq!(code(&weeks), "work_expenses.fact_conflict");
    let business = with(from_log(log(84)), json!({"business_kilometres": "100"}));
    assert_eq!(code(&business), "work_expenses.fact_conflict");
    let no_total = with(from_log(log(84)), json!({"total_kilometres": "remove"}));
    assert_eq!(code(&no_total), "input.missing_field");
    let other_method = json!({
        "method": "car_cents_per_kilometre", "reimbursed": false, "allowance_received": false,
        "owned_or_leased": true, "business_kilometres": "10", "log_book": log(84),
        "year": "2025-26", "scope_confirmed": true,
    });
    assert_eq!(code(&other_method), "input.field_unknown");
}

#[test]
fn derive_explains_the_diary_and_log_book_steps() {
    for input in [
        diary("2025-26", rows("2025-07-01", 3, "2")),
        from_log(log(84)),
    ] {
        let doc = api::evaluate(
            "derive",
            &json!({"command": "work-expenses", "input": input}),
        )
        .unwrap();
        let steps = doc["steps"].as_array().unwrap();
        assert!(
            steps
                .iter()
                .all(|s| s["authority"].as_str().is_some_and(|a| !a.is_empty()))
        );
    }
}
