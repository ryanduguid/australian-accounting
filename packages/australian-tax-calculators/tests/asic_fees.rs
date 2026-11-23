// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! ASIC's 2026-27 fees against the site's ASIC fees page (ryanduguid.github.io
//! at dcc7ecbe: the page's 29 cases, the 32 vectors of its tests and every due
//! and received date pair from 1 June 2026 to 31 July 2027, run through the
//! page's own `asicFee` into `data/asic_fees_site_cases.json`), and against
//! the regulations' items. Sources were read on 24 November 2026
//! (docs/calculation-evidence.md#asic-fees). Every date is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Result<Value, &'static str> {
    api::evaluate_coded("asic-fees", input).map_err(|e| e.code)
}

fn fee(key: &str) -> Value {
    json!({"year": "2026-27", "scope_confirmed": true, "fee": key})
}

fn late(key: &str, due: &str, received: &str) -> Value {
    let mut input = fee(key);
    input["due_date"] = json!(due);
    input["received_date"] = json!(received);
    input
}

/// The command's input for a site call: its fee key in snake case and its
/// dates under the command's names.
fn site_input(call: &Value) -> Value {
    let mut input = fee(&call["fee"].as_str().unwrap().replace('-', "_"));
    for (site, ours) in [("due", "due_date"), ("received", "received_date")] {
        if let Some(date) = call.get(site) {
            input[ours] = date.clone();
        }
    }
    input
}

/// The site's result as the command writes it: whole dollars to cents, a
/// late fee that is not late with no code.
fn agrees(site: &Value, out: &Value) -> bool {
    let code = match site["code"].as_str() {
        Some("") => Value::Null,
        _ => site["code"].clone(),
    };
    out["amounts"]["fee"] == json!(format!("{}.00", site["amount"]))
        && out["code"] == code
        && out["tier"] == site["tier"]
        && out["first_month_ends"] == site["firstMonthEnds"]
}

#[test]
fn the_site_cases_agree() {
    let data: Value = serde_json::from_str(include_str!("data/asic_fees_site_cases.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    let (mut figures, mut refusals) = (0, 0);
    for row in rows {
        match (&row["refused"], run(&site_input(&row["call"]))) {
            (Value::String(code), Err(got)) => {
                assert_eq!(code, got, "{row}");
                refusals += 1;
            }
            (Value::Null, Ok(out)) => {
                assert!(agrees(&row["result"], &out), "{row}\n{out}");
                figures += 1;
            }
            (_, got) => panic!("{row}\n{got:?}"),
        }
    }
    let sources: Vec<&str> = rows.iter().map(|r| r["source"].as_str().unwrap()).collect();
    let page = sources.iter().filter(|s| **s == "page").count();
    eprintln!(
        "asic-fees site parity: {} rows ({page} page, {} test), {figures} with the same figures, {refusals} refused alike",
        rows.len(),
        rows.len() - page
    );
    assert_eq!([rows.len(), page, refusals], [61, 29, 27]);
}

/// The sweep's outcome letters: n not late, w within one month, b more than
/// one month, y outside 2026-27, d refused as a weekend could move a date.
/// A worked tier must carry its own amount and code.
fn letter(got: &Result<Value, &str>) -> char {
    match got {
        Ok(out) => {
            let (letter, fee, code) = match out["tier"].as_str().unwrap() {
                "not late" => ('n', "0.00", Value::Null),
                "within one month" => ('w', "102.00", json!("LPF1_ETM")),
                "more than one month" => ('b', "428.00", json!("LPF2_ETM")),
                tier => panic!("{tier}"),
            };
            assert_eq!([&out["amounts"]["fee"], &out["code"]], [&json!(fee), &code]);
            letter
        }
        Err("asic_fees.dates_outside_year") => 'y',
        Err("asic_fees.weekend_undecided") => 'd',
        Err(code) => panic!("{code}"),
    }
}

/// Every due and received date pair the site swept, for both late fees.
#[test]
fn every_due_and_received_date_agrees_with_the_site() {
    let data: Value = serde_json::from_str(include_str!("data/asic_fees_site_cases.json")).unwrap();
    let first = date_number(data["sweep_from"].as_str().unwrap());
    let mut pairs = 0;
    for row in data["sweep"].as_array().unwrap() {
        let due = row["due"].as_str().unwrap();
        let mut received = first;
        for run_ in row["runs"].as_str().unwrap().split(' ') {
            let (want, count) = run_.split_at(1);
            for _ in 0..count.parse::<u32>().unwrap() {
                let on = date_text(received);
                for key in ["late_review_payment", "late_lodgement"] {
                    let got = run(&late(key, due, &on));
                    assert_eq!(letter(&got).to_string(), want, "{key} {due} {on}");
                    if let Ok(out) = got {
                        assert_eq!(out["first_month_ends"], row["firstMonthEnds"], "{due}");
                    }
                }
                received += 1;
                pairs += 1;
            }
        }
    }
    assert_eq!(json!(pairs), data["sweep_pairs"]);
    assert_eq!(pairs, 181_476);
}

/// Days from 1 January 2026, enough for the sweep's dates.
fn date_number(text: &str) -> i64 {
    let (y, m, d): (i64, i64, i64) = (
        text[0..4].parse().unwrap(),
        text[5..7].parse().unwrap(),
        text[8..10].parse().unwrap(),
    );
    let mut days = (y - 2026) * 365 + d - 1;
    for month in 1..m {
        days += month_days(y, month);
    }
    days
}

fn month_days(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 4 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn date_text(mut days: i64) -> String {
    let mut year = 2026;
    while days >= 365 {
        days -= 365;
        year += 1;
    }
    let mut month = 1;
    while days >= month_days(year, month) {
        days -= month_days(year, month);
        month += 1;
    }
    format!("{year}-{month:02}-{:02}", days + 1)
}

/// The fixed fees, each with ASIC's code and the item that imposes it:
/// company registration is Schedule 2 item 1(a) of the Corporations (Fees)
/// Regulations, not Schedule 1.
const FIXED: [[&str; 4]; 9] = [
    [
        "company_share_capital",
        "636.00",
        "I02",
        "Schedule 2 item 1(a)(i)",
    ],
    [
        "company_no_share_capital",
        "524.00",
        "I01",
        "Schedule 2 item 1(a)(ii)",
    ],
    ["name_reservation", "65.00", "R01", "Schedule 2 item 8"],
    [
        "review_public",
        "1591.00",
        "A51",
        "Schedule 1 Part 1 item 1",
    ],
    [
        "review_proprietary",
        "342.00",
        "A52",
        "Schedule 1 Part 1 item 2",
    ],
    [
        "review_special_public",
        "65.00",
        "A53",
        "Schedule 1 Part 1 item 4",
    ],
    [
        "review_special_proprietary",
        "70.00",
        "A58",
        "Schedule 1 Part 1 item 5",
    ],
    [
        "business_name_1",
        "47.00",
        "BZA or BZC",
        "section 6 items 1 and 2",
    ],
    [
        "business_name_3",
        "108.00",
        "BZB or BZD",
        "section 6 items 3 and 4",
    ],
];

#[test]
fn the_fixed_fees_state_their_amount_code_and_item() {
    for [key, amount, code, item] in FIXED {
        let out = run(&fee(key)).unwrap();
        assert_eq!(out["amounts"], json!({"fee": amount}), "{key}");
        assert_eq!([&out["fee"], &out["code"]], [key, code]);
        assert!(out["authority"].as_str().unwrap().ends_with(item), "{key}");
        assert_eq!([&out["tier"], &out["first_month_ends"]], [&Value::Null; 2]);
        assert_eq!(out["steps"], json!([]));
    }
}

/// The page's example: due 15 September 2026, paid on 15 October, the last
/// day of the first month (s 2G(1)), so $102; the day after, $428.
#[test]
fn a_payment_on_the_last_day_of_the_month_is_within_it() {
    let out = run(&late("late_review_payment", "2026-09-15", "2026-10-15")).unwrap();
    assert_eq!(out["amounts"]["fee"], "102.00");
    assert_eq!(
        [&out["code"], &out["tier"]],
        ["LPF1_ETM", "within one month"]
    );
    assert_eq!(out["first_month_ends"], "2026-10-15");
    assert!(
        out["steps"][1]["authority"]
            .as_str()
            .unwrap()
            .contains("Part 3 items 1 and 2")
    );
    let after = run(&late("late_lodgement", "2026-09-15", "2026-10-16")).unwrap();
    assert_eq!(after["amounts"]["fee"], "428.00");
    assert_eq!(
        [&after["code"], &after["tier"]],
        ["LPF2_ETM", "more than one month"]
    );
    assert!(
        after["authority"]
            .as_str()
            .unwrap()
            .ends_with("Schedule 2 item 14")
    );
    let on_time = run(&late("late_lodgement", "2026-09-15", "2026-09-15")).unwrap();
    assert_eq!(on_time["amounts"]["fee"], "0.00");
    assert_eq!(
        [&on_time["code"], &on_time["tier"]],
        [&Value::Null, &json!("not late")]
    );
}

#[test]
fn refuses_years_dates_and_fees_it_does_not_hold() {
    let mut next = fee("review_public");
    next["year"] = json!("2027-28");
    let message = api::evaluate_coded("asic-fees", &next).unwrap_err();
    assert_eq!(message.code, "period.unsupported");
    assert!(message.message.contains("March 2027 quarter CPI"));
    next["year"] = json!("2025-26");
    assert_eq!(run(&next), Err("period.unsupported"));
    let mut unconfirmed = fee("review_public");
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(run(&unconfirmed), Err("scope.not_confirmed"));
    assert_eq!(run(&fee("Review_Public")), Err("asic_fees.fee_unknown"));
    let mut missing = late("late_lodgement", "2026-09-15", "2026-10-01");
    missing.as_object_mut().unwrap().remove("received_date");
    assert_eq!(run(&missing), Err("input.missing_field"));
    missing.as_object_mut().unwrap().remove("due_date");
    assert_eq!(run(&missing), Err("input.missing_field"));
    let mut stray = fee("review_public");
    stray["paid_date"] = json!("2026-09-15");
    assert_eq!(run(&stray), Err("input.field_unknown"));
    let mut number = late("late_lodgement", "2026-09-15", "2026-10-01");
    number["due_date"] = json!(20_260_915);
    assert!(run(&number).is_err());
}

/// The page's dates either side of 1 July and on weekends.
#[test]
fn refuses_what_a_weekend_or_a_new_year_could_change() {
    let refused = |due, received| run(&late("late_review_payment", due, received)).unwrap_err();
    assert_eq!(
        refused("2026-06-30", "2026-07-01"),
        "asic_fees.dates_outside_year"
    );
    assert_eq!(
        refused("2027-06-15", "2027-07-01"),
        "asic_fees.dates_outside_year"
    );
    // Sunday 16 August 2026: payments to Wednesday 19 August are refused.
    assert_eq!(
        refused("2026-08-16", "2026-08-19"),
        "asic_fees.weekend_undecided"
    );
    let thursday = run(&late("late_review_payment", "2026-08-16", "2026-08-20")).unwrap();
    assert_eq!(thursday["amounts"]["fee"], "102.00");
    // Saturday 31 October 2026 ends the month after 30 September.
    let message = api::evaluate_coded(
        "asic-fees",
        &late("late_review_payment", "2026-09-30", "2026-11-04"),
    )
    .unwrap_err();
    assert!(
        message
            .message
            .starts_with("The first month after the due date ends")
    );
    // Saturday 27 February 2027 could move to 3 March, ending the month on
    // Saturday 3 April and so as late as Wednesday 7 April.
    let message = api::evaluate_coded(
        "asic-fees",
        &late("late_review_payment", "2027-02-27", "2027-04-07"),
    )
    .unwrap_err();
    assert!(message.message.contains("may end later"));
    let beyond = run(&late("late_review_payment", "2027-02-27", "2027-04-08")).unwrap();
    assert_eq!(beyond["amounts"]["fee"], "428.00");
}
