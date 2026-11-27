// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `dgr-check`'s unit tests, on a fabricated register.
use super::register::{abn_digits, parse_register};
use super::*;

// The fabricated register's ABNs, kept apart from their labels so the
// repository's ABN secret scan does not read them as real ones.
const SCHOOL: &str = "23000000004";
const SCHOOL_SPACED: &str = "23 000 000 004";
const CLOSED: &str = "42000000003";
const RIVER: &str = "61000000002";
const RED: &str = "80000000001";
/// Endorsed for a fund the data does not name, then in its own right.
const HARBOUR: &str = "74000000006";
/// Passes the check digit test but is not in the register.
const ABSENT: &str = "93000000005";
const BAD_CHECK_DIGIT: &str = "23000000005";
const DASHED: &str = "23-000-000-004";
const TWELVE_DIGITS: &str = "023000000004";

/// A fabricated register: ABNs that pass the check digit, with entries
/// chosen to reach each reading.
const REGISTER: &str = r#"{"source":"fabricated","extract_time":"2026-11-11T12:20:05","abns":5}
["23000000004","ACT","2000-02-24","EXAMPLE SCHOOL LTD",[["2000-07-01","EXAMPLE SCHOOL BUILDING FUND"],["2010-07-01",null]],["EXAMPLE COLLEGE"]]
["42000000003","CAN","2020-01-01","CLOSED CHARITY LIMITED",[["2001-01-01",null]],[]]
["61000000002","ACT","2015-01-01","RIVER CARE ASSOCIATION INC",[["2015-03-01","RIVER CARE FUND"],["2016-03-01","RIVER CARE LIBRARY"]],[]]
["74000000006","ACT","2000-01-01","HARBOUR AID LTD",[["2000-07-01",""],["2018-01-01",null]],[]]
["80000000001","ACT","2000-01-01","RED EXAMPLE CROSS SOCIETY",[["2000-07-01",null]],["RED CROSS SHOPS"]]"#;

fn run(donations: Value) -> Result<Value, Refusal> {
    check_with(&parse_register(REGISTER), &json!({"donations": donations}))
}

fn one(donation: Value) -> Value {
    run(json!([donation])).unwrap()["donations"][0].clone()
}

fn refused(donations: Value) -> (&'static str, String) {
    let r = run(donations).unwrap_err();
    (r.code, r.message)
}

fn found<'a>(result: &'a Value, id: &str) -> Option<&'a Value> {
    let conditions = result["conditions"].as_array().unwrap();
    conditions.iter().find(|c| c["condition"] == id)
}

fn status(result: &Value, id: &str) -> Option<String> {
    found(result, id).map(|c| c["status"].as_str().unwrap().to_string())
}

#[test]
fn an_entity_endorsed_in_its_own_right_is_shown_on_a_date_inside_its_entry() {
    let r = one(json!({"abn": SCHOOL_SPACED, "date": "2012-05-01", "amount": "50.00"}));
    assert_eq!(r["register"], "shown_as_dgr");
    assert_eq!(r["on_payment_date"], "shown");
    assert_eq!(r["entity"]["name"], "EXAMPLE SCHOOL LTD");
    assert_eq!(r["other_names"], json!(["EXAMPLE COLLEGE"]));
    assert_eq!(status(&r, "gift_to_the_fund"), None);
    assert_eq!(status(&r, "two_dollar_minimum").as_deref(), Some("met"));
    assert_eq!(status(&r, "a_gift").as_deref(), Some("open"));
    let finding = r["finding"].as_str().unwrap();
    assert!(
        finding.starts_with(&format!("ABN {SCHOOL_SPACED} (EXAMPLE SCHOOL LTD) is shown as a deductible gift recipient in the data of 11 November 2026, with entries from 1 July 2000.")),
        "{finding}"
    );
}

#[test]
fn before_the_entity_entry_only_its_fund_is_shown_and_the_gift_must_go_to_it() {
    let r = one(json!({"abn": SCHOOL, "date": "2005-01-01"}));
    assert_eq!(r["on_payment_date"], "shown_for_a_fund");
    let fund = found(&r, "gift_to_the_fund").unwrap()["text"]
        .as_str()
        .unwrap();
    assert!(fund.contains("EXAMPLE SCHOOL BUILDING FUND"), "{fund}");
}

#[test]
fn a_date_before_every_entry_is_not_shown_and_a_later_one_is_after_the_data() {
    let early = one(json!({"abn": SCHOOL, "date": "2000-06-30"}));
    assert_eq!(early["on_payment_date"], "not_shown");
    assert_eq!(early["register"], "shown_as_dgr");
    let finding = early["finding"].as_str().unwrap();
    assert!(
        finding.contains("only from 1 July 2000, after the payment"),
        "{finding}"
    );
    let late = one(json!({"abn": SCHOOL, "date": "2026-11-12"}));
    assert_eq!(late["on_payment_date"], "after_data_date");
    let on = one(json!({"abn": SCHOOL, "date": "2026-11-11"}));
    assert_eq!(on["on_payment_date"], "shown");
    let first_day = one(json!({"abn": SCHOOL, "date": "2000-07-01"}));
    assert_eq!(first_day["on_payment_date"], "shown_for_a_fund");
}

#[test]
fn a_cancelled_abn_is_not_read_as_a_dgr_from_its_cancellation() {
    let after = one(json!({"abn": CLOSED, "date": "2020-01-01"}));
    assert_eq!(after["on_payment_date"], "abn_cancelled");
    assert_eq!(after["entity"]["abn_status"], "cancelled");
    let before = one(json!({"abn": CLOSED, "date": "2019-12-31"}));
    assert_eq!(before["on_payment_date"], "shown");
}

#[test]
fn a_fund_only_entity_names_the_funds_held_on_the_date() {
    let r = one(json!({"abn": RIVER}));
    assert_eq!(r["on_payment_date"], "no_date");
    let text = found(&r, "gift_to_the_fund").unwrap()["text"]
        .as_str()
        .unwrap();
    assert!(
        text.contains("RIVER CARE FUND; RIVER CARE LIBRARY"),
        "{text}"
    );
    let r = one(json!({"abn": RIVER, "date": "2015-06-01"}));
    let text = found(&r, "gift_to_the_fund").unwrap()["text"]
        .as_str()
        .unwrap();
    assert!(
        text.contains("RIVER CARE FUND") && !text.contains("LIBRARY"),
        "{text}"
    );
    // Before either fund the data shows no entry, and an ended one could
    // have been for the entity itself, so no fund is named.
    let r = one(json!({"abn": RIVER, "date": "2015-01-01"}));
    assert_eq!(r["on_payment_date"], "not_shown");
    assert_eq!(found(&r, "gift_to_the_fund"), None);
    assert_eq!(status(&r, "two_dollar_minimum").as_deref(), Some("open"));
    // After the data date the entries on the payment date are unknown too.
    let r = one(json!({"abn": RIVER, "date": "2026-11-12"}));
    assert_eq!(r["on_payment_date"], "after_data_date");
    assert_eq!(found(&r, "gift_to_the_fund"), None);
}

#[test]
fn a_fund_the_extract_does_not_name_is_still_a_fund() {
    let r = one(json!({"abn": HARBOUR, "date": "2010-01-01"}));
    assert_eq!(r["on_payment_date"], "shown_for_a_fund");
    let unnamed = "a fund, authority or institution the data does not name";
    assert_eq!(r["dgr_entries"][0]["entry_for"], unnamed);
    let text = found(&r, "gift_to_the_fund").unwrap()["text"]
        .as_str()
        .unwrap();
    assert!(
        text.ends_with(&format!(
            "made to it: {unnamed} (s 30-17(2)(b) and (c), s 30-229(2))."
        )),
        "{text}"
    );
    let later = one(json!({"abn": HARBOUR, "date": "2018-01-01"}));
    assert_eq!(later["on_payment_date"], "shown");
    assert_eq!(found(&later, "gift_to_the_fund"), None);
}

#[test]
fn an_abn_not_in_the_data_is_not_shown_never_not_a_dgr() {
    let r = one(json!({"abn": ABSENT, "date": "2025-01-01", "amount": "1.00"}));
    assert_eq!(r["register"], "not_shown_as_dgr");
    let finding = r["finding"].as_str().unwrap();
    assert!(
        finding.starts_with(&format!("ABN {} is not shown as a deductible gift recipient in the data of 11 November 2026. That does not establish that it is not one", spaced_abn(ABSENT))),
        "{finding}"
    );
    assert!(r.get("entity").is_none());
    // A gift from 1 July 2024 has no minimum, so no $2 condition at all.
    assert_eq!(status(&r, "two_dollar_minimum"), None);
}

#[test]
fn the_two_dollar_minimum_applies_to_gifts_before_1_july_2024_only() {
    let minimum = |date: &str, amount: &str| {
        let r = one(json!({"abn": RED, "date": date, "amount": amount}));
        status(&r, "two_dollar_minimum")
    };
    assert_eq!(minimum("2024-06-30", "1.99").as_deref(), Some("fails"));
    assert_eq!(minimum("2024-06-30", "2.00").as_deref(), Some("met"));
    assert_eq!(minimum("2024-06-30", "2").as_deref(), Some("met"));
    assert_eq!(minimum("2024-07-01", "1.99"), None);
    let undated = one(json!({"abn": RED, "amount": "1.00"}));
    assert_eq!(
        status(&undated, "two_dollar_minimum").as_deref(),
        Some("open")
    );
    let no_amount = one(json!({"abn": RED, "date": "2020-01-01"}));
    assert_eq!(
        status(&no_amount, "two_dollar_minimum").as_deref(),
        Some("open")
    );
}

#[test]
fn a_name_gives_unconfirmed_candidates_and_never_a_reading() {
    let r = one(json!({"name": "Example School Ltd", "date": "2025-01-01"}));
    assert_eq!(r["register"], "abn_needed");
    assert_eq!(r["candidates_total"], 1);
    let candidate = &r["candidates"][0];
    assert_eq!(candidate["status"], "unconfirmed");
    assert_eq!(candidate["entity"]["abn"], SCHOOL);
    assert_eq!(candidate["matched_name"], "EXAMPLE SCHOOL LTD");
    assert!(r.get("on_payment_date").is_none());
    // Fund names and other names match too, by whole words.
    assert_eq!(
        one(json!({"name": "river care library"}))["candidates_total"],
        1
    );
    assert_eq!(one(json!({"name": "Red Cross"}))["candidates_total"], 1);
    assert_eq!(one(json!({"name": "Red Cros"}))["candidates_total"], 0);
    assert_eq!(one(json!({"name": "example"}))["candidates_total"], 2);
    let none = one(json!({"name": "Nobody Here"}));
    assert_eq!(none["candidates_total"], 0);
    let finding = none["finding"].as_str().unwrap();
    assert!(
        finding.contains("That does not establish that the payee is not one"),
        "{finding}"
    );
}

#[test]
fn candidates_stop_at_the_cap_but_the_total_counts_every_match() {
    let mut text =
        String::from(r#"{"source":"fabricated","extract_time":"2026-11-11T12:20:05","abns":12}"#);
    let mut abns = Vec::new();
    for body in 0..10_000_000u32 {
        let base = format!("{body:09}");
        if let Some(abn) = (10..100)
            .map(|c| format!("{c}{base}"))
            .find(|a| abn_digits(a).is_some())
        {
            abns.push(abn);
        }
        if abns.len() == 12 {
            break;
        }
    }
    abns.sort();
    for abn in &abns {
        text.push_str(&format!(
            "\n[\"{abn}\",\"ACT\",\"2000-01-01\",\"SAME NAME\",[[\"2000-07-01\",null]],[]]"
        ));
    }
    let r = check_with(
        &parse_register(&text),
        &json!({"donations": [{"name": "same name"}]}),
    )
    .unwrap();
    assert_eq!(r["donations"][0]["candidates_total"], 12);
    assert_eq!(
        r["donations"][0]["candidates"].as_array().unwrap().len(),
        MAX_CANDIDATES
    );
}

#[test]
fn a_name_beside_an_abn_is_compared_with_the_register() {
    let r = one(json!({"abn": RED, "name": "Red Cross Shops"}));
    assert_eq!(r["name_matches_register"], true);
    let r = one(json!({"abn": RED, "name": "Blue Cross"}));
    assert_eq!(r["name_matches_register"], false);
    assert_eq!(r["register"], "shown_as_dgr");
}

#[test]
fn results_keep_the_input_order_and_never_say_deductible() {
    let donations = json!([{"abn": ABSENT}, {"name": "river"}, {"abn": RED}]);
    let r = run(donations).unwrap();
    let donations = r["donations"].as_array().unwrap();
    let order: Vec<&Value> = donations.iter().map(|d| &d["index"]).collect();
    assert_eq!(order, [0, 1, 2]);
    assert_eq!(donations[1]["input"], json!({"name": "river"}));
    assert_eq!(r["data"]["data_date"], "2026-11-11");
    assert_eq!(r["data"]["abns_with_dgr_entries"], 5);
    let text = r.to_string();
    assert!(!text.contains("\"deductible\"") && !text.contains("is deductible"));
}

#[test]
fn malformed_input_is_refused() {
    let cases = [
        (json!({"donations": []}), "input.donations_invalid"),
        (json!({"donations": {}}), "input.donations_invalid"),
        (json!({}), "input.missing_field"),
        (
            json!({"donations": [{}], "year": "2025-26"}),
            "input.field_unknown",
        ),
    ];
    for (input, code) in cases {
        let r = check_with(&parse_register(REGISTER), &input).unwrap_err();
        assert_eq!(r.code, code, "{input}");
    }
    let too_many = vec![json!({"abn": SCHOOL}); MAX_DONATIONS + 1];
    assert_eq!(refused(json!(too_many)).0, "input.donations_invalid");
}

/// Each list of donations is refused with the code and a message that starts
/// by naming the field.
fn refuses_each(cases: &[(Value, &str, &str)]) {
    for (donations, code, message) in cases {
        let (got, text) = refused(donations.clone());
        assert_eq!(got, *code, "{donations}");
        assert!(text.starts_with(message), "{donations}: {text}");
    }
}

#[test]
fn a_malformed_donation_abn_or_name_is_refused_naming_it() {
    let long = "x".repeat(201);
    refuses_each(&[
        (
            json!([{"abn": SCHOOL}, {"abn": BAD_CHECK_DIGIT}]),
            "input.abn_invalid",
            "donations[1].abn \"",
        ),
        (
            json!([{"abn": DASHED}]),
            "input.abn_invalid",
            "donations[0].abn",
        ),
        (
            json!([{"abn": TWELVE_DIGITS}]),
            "input.abn_invalid",
            "donations[0].abn",
        ),
        (
            json!([{"abn": 23000000004_u64}]),
            "input.abn_invalid",
            "donations[0].abn must be a string",
        ),
        (
            json!([{"name": " - "}]),
            "input.name_invalid",
            "donations[0].name",
        ),
        // name_words reads & as AND; the name itself has no letter or digit.
        (
            json!([{"name": "&"}]),
            "input.name_invalid",
            "donations[0].name",
        ),
        (
            json!([{"abn": SCHOOL, "name": "&"}]),
            "input.name_invalid",
            "donations[0].name",
        ),
        (
            json!([{"name": long}]),
            "input.name_invalid",
            "donations[0].name",
        ),
    ]);
}

#[test]
fn a_donation_that_is_not_an_object_lacks_both_or_has_a_bad_date_is_refused() {
    refuses_each(&[
        (
            json!([{"name": 7}]),
            "input.name_invalid",
            "donations[0].name",
        ),
        (
            json!(["23000000004"]),
            "input.donations_invalid",
            "donations[0] must be an object",
        ),
        (
            json!([{}]),
            "input.missing_field",
            "donations[0] needs an abn or a name",
        ),
        (
            json!([{"abn": SCHOOL, "date": "2025-02-30"}]),
            "input.date_invalid",
            "donations[0].date",
        ),
        (
            json!([{"abn": SCHOOL, "date": 20250101}]),
            "input.date_invalid",
            "donations[0].date",
        ),
    ]);
}

#[test]
fn a_malformed_donation_amount_or_an_unknown_field_is_refused_naming_it() {
    refuses_each(&[
        (
            json!([{"abn": SCHOOL, "amount": "-5.00"}]),
            "input.amount_invalid",
            "donations[0].amount: ",
        ),
        (
            json!([{"abn": SCHOOL, "amount": "5.001"}]),
            "input.amount_invalid",
            "donations[0].amount: ",
        ),
        (
            json!([{"abn": SCHOOL, "amount": "five"}]),
            "input.decimal_unreadable",
            "donations[0].amount: ",
        ),
        (
            json!([{"abn": SCHOOL, "amount": 5}]),
            "input.amount_invalid",
            "donations[0].amount: ",
        ),
        (
            json!([{"abn": SCHOOL, "ref": "INV-1"}]),
            "input.field_unknown",
            "donations[0].ref is not an input of dgr-check",
        ),
    ]);
}

#[test]
fn the_abn_check_follows_abn_lookups_worked_example() {
    // https://abr.business.gov.au/Help/AbnFormat (version 9.9.7, read 16
    // November 2026): 51 824 753 556 gives 534, which divides by 89.
    assert_eq!(abn_digits("51 824 753 556").as_deref(), Some("51824753556"));
    assert_eq!(abn_digits("51824753557"), None);
    assert_eq!(abn_digits("5182475355"), None);
    assert_eq!(abn_digits("51\u{a0}824\u{a0}753\u{a0}556"), None);
}

#[test]
fn name_words_read_abbreviations_and_ampersands_as_words() {
    let words = name_words("Smith & Co Pty Ltd");
    assert_eq!(words, ["AND", "CO", "LIMITED", "PROPRIETARY", "SMITH"]);
    assert_eq!(
        name_words("The Example Inc."),
        ["EXAMPLE", "INCORPORATED", "THE"]
    );
    assert_eq!(long_date("2024-07-01"), "1 July 2024");
}

#[test]
fn a_decimal_signal_is_refused_with_its_code_and_name() {
    let refusal = signal(crate::decimal::Signal::Overflow);
    assert_eq!(refusal.code, "decimal.overflow");
    assert_eq!(refusal.message, "decimal.Overflow");
}
