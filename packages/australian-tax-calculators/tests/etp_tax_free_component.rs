// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The ETP tax free component against the ATO's Jackson example ('Tax-free
//! component of ETPs', last updated 24 July 2025), the Library's Jim (¶8-850)
//! and Jon (¶8-860) examples and ITAA 1997 ss 82-10(6), 82-140 to 82-155 and
//! 995-1(1), read on 19 November 2026
//! (docs/calculation-evidence.md#etp-tax-free-component);
//! then each condition of s 82-150(1), the day counts at their edges and the
//! refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "etp-tax-free-component";

/// A $100,000 invalidity payment after employment from 1 July 2000, terminated
/// 2 years before the last retirement day.
fn input(overrides: Value) -> Value {
    let mut facts = json!({
        "payment": "100000.00", "employment_start": "2000-07-01",
        "termination_date": "2026-09-30", "last_retirement_day": "2028-09-30",
        "ill_health": true, "medical_certification": true,
        "year": "2026-27", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn run(overrides: Value) -> Value {
    let facts = input(overrides);
    let result = api::evaluate(COMMAND, &facts).unwrap();
    // derive gives the same result, byte for byte.
    let envelope = json!({"command": COMMAND, "input": facts});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(
        doc["result"].to_string(),
        result.to_string(),
        "derive {facts}"
    );
    result
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded(COMMAND, &input(overrides))
        .unwrap_err()
        .code
}

/// The invalidity segment, the pre-July 83 segment, the tax free component
/// and the taxable component.
fn components(result: &Value) -> [&str; 4] {
    [
        "invalidity_segment",
        "pre_july_83_segment",
        "tax_free_component",
        "taxable_component",
    ]
    .map(|key| result["amounts"][key].as_str().unwrap())
}

/// The employment days, the days to retirement and the days before 1 July 1983.
fn days(result: &Value) -> [&str; 3] {
    [
        "employment_days",
        "days_to_retirement",
        "pre_july_1983_days",
    ]
    .map(|key| result["rates"][key].as_str().unwrap())
}

/// Jackson's $102,000 ETP: $102,000 x 370 / (3,305 + 370) = $10,269 tax free
/// and $91,731 taxable, which the ATO shows in whole dollars. The page's own
/// dates (2 September 2013 to 25 September 2022, a 65th birthday on
/// 25 September 2023) count 3,311 and 365 days, not its 3,305 and 370, so
/// these dates are chosen to give its counts: 8 September 2017 to
/// 25 September 2026 is 3,305 days with both counted, and 370 days on is
/// 30 September 2027. The test holds the formula's arithmetic to the page.
#[test]
fn jacksons_example_gives_the_atos_invalidity_segment() {
    let result = run(json!({
        "payment": "102000.00", "employment_start": "2017-09-08",
        "termination_date": "2026-09-25", "last_retirement_day": "2027-09-30",
    }));
    assert_eq!(days(&result), ["3305", "370", "0"]);
    // 102000 x 370 / 3675 = 10269.3877..., to the cent.
    assert_eq!(
        components(&result),
        ["10269.39", "0.00", "10269.39", "91730.61"]
    );
    assert_eq!(result["rates"]["invalidity_excluded_payment"], "true");
}

/// The same payment on the page's own dates, counted as NAT 71949 counts
/// them: 3,311 employment days and 365 days to retirement.
#[test]
fn jacksons_own_dates_are_counted_as_the_worksheet_counts() {
    let result = run(json!({
        "payment": "102000.00", "employment_start": "2013-09-02",
        "termination_date": "2022-09-25", "last_retirement_day": "2023-09-25",
    }));
    assert_eq!(days(&result), ["3311", "365", "0"]);
    // 102000 x 365 / 3676 = 10127.8563...
    assert_eq!(
        components(&result),
        ["10127.86", "0.00", "10127.86", "91872.14"]
    );
}

/// The Library's Jim (¶8-850): a $100,000 ETP on termination for invalidity on
/// 31 July 2026, his 65th birthday on 15 July 2027: 349 days to retirement
/// (1 August 2026 to 15 July 2027). The example's 16,050 employment days give
/// its $2,128 (100000 x 349 / 16399 = 2128.18); 22 August 1982 to 31 July 2026
/// is 16,050 days with both counted. Its own start, 20 August 1981, gives
/// 16,417 days, which the next test works.
#[test]
fn jims_example_gives_the_librarys_invalidity_segment() {
    let result = run(json!({
        "employment_start": "1982-08-22", "termination_date": "2026-07-31",
        "last_retirement_day": "2027-07-15",
    }));
    assert_eq!(days(&result)[..2], ["16050", "349"]);
    assert_eq!(components(&result)[0], "2128.18");
}

/// Jim's own dates: employment from 20 August 1981, so 680 of the 16,417
/// employment days fall before 1 July 1983 (20 August 1981 to 30 June 1983),
/// and the pre-July 83 segment is the rest of the payment times 680 / 16,417
/// (s 82-155(2)), worked by hand:
/// invalidity 100000 x 349 / 16766 = 2081.5937... = 2081.59;
/// pre-July 83 97918.41 x 680 / 16417 = 4055.8262... = 4055.83;
/// tax free 6137.42; taxable 100000.00 - 6137.42 = 93862.58.
#[test]
fn employment_before_july_1983_adds_the_pre_july_83_segment() {
    let result = run(json!({
        "employment_start": "1981-08-20", "termination_date": "2026-07-31",
        "last_retirement_day": "2027-07-15",
    }));
    assert_eq!(days(&result), ["16417", "349", "680"]);
    assert_eq!(
        components(&result),
        ["2081.59", "4055.83", "6137.42", "93862.58"]
    );
    assert_eq!(
        result["amounts"]["payment_less_invalidity_segment"],
        "97918.41"
    );
}

/// The Library's Jon (¶8-860): employment from 1 January 1980, terminated on
/// 31 December 2026, $70,000 at step 1 (his $100,000 ETP less its $30,000
/// invalidity segment), given here as a payment with none. Its 1,277 days
/// before 1 July 1983 are the count made here; its 17,175 total days are not:
/// 1 January 1980 to 31 December 2026 is 17,167 days with both counted, so
/// 70000 x 1277 / 17167 = 5207.0828... = 5207.08, where the example's
/// 70000 x 1277 / 17175 gives its $5,205.
#[test]
fn jons_example_gives_the_librarys_pre_july_83_days() {
    let result = run(json!({
        "payment": "70000.00", "employment_start": "1980-01-01",
        "termination_date": "2026-12-31", "ill_health": false,
    }));
    assert_eq!(days(&result), ["17167", "0", "1277"]);
    assert_eq!(
        components(&result),
        ["0.00", "5207.08", "5207.08", "64792.92"]
    );
}

/// s 82-150(1)(a), (b) and (d): without the ill-health or the 2 certificates
/// the payment has no invalidity segment and is not excluded on that ground;
/// the pre-July 83 segment still applies to the whole payment.
#[test]
fn without_ill_health_or_certificates_there_is_no_invalidity_segment() {
    for (field, section) in [
        ("ill_health", "s 82-150(1)(a) and (b)"),
        ("medical_certification", "s 82-150(1)(d)"),
    ] {
        let mut overrides = json!({"employment_start": "1981-08-20"});
        overrides[field] = json!(false);
        let facts = input(overrides);
        let result = run(facts.clone());
        assert_eq!(components(&result)[0], "0.00", "{field}");
        assert_eq!(days(&result)[1], "0", "{field}");
        assert_eq!(result["rates"]["invalidity_excluded_payment"], "false");
        // 100000 x 680 / 16478 (20 August 1981 to 30 September 2026) =
        // 4126.7144...
        assert_eq!(days(&result)[0], "16478");
        assert_eq!(components(&result)[1], "4126.71", "{field}");
        let envelope = json!({"command": COMMAND, "input": facts});
        let doc = api::evaluate("derive", &envelope).unwrap();
        let authority = doc["steps"][0]["authority"].as_str().unwrap();
        assert!(authority.contains(section), "{authority}");
    }
}

/// s 82-150(1)(c): employment that stops on the last retirement day did not
/// stop before it, so there is no segment; the payment is still an excluded
/// payment, as it would include one but for paragraph (c) (s 82-10(6)(c)).
/// The day before leaves 1 day: 50000 x 1 / 2631 = 19.0041... = 19.00.
#[test]
fn the_last_retirement_day_ends_the_segment() {
    let on = run(json!({"termination_date": "2028-09-30"}));
    assert_eq!(components(&on), ["0.00", "0.00", "0.00", "100000.00"]);
    assert_eq!(on["rates"]["invalidity_excluded_payment"], "true");
    let after = run(json!({"termination_date": "2028-10-01"}));
    assert_eq!(components(&after)[0], "0.00");
    assert_eq!(after["rates"]["invalidity_excluded_payment"], "true");
    let before = run(json!({
        "payment": "50000.00", "employment_start": "2020-01-01",
        "termination_date": "2027-03-14", "last_retirement_day": "2027-03-15",
    }));
    assert_eq!(days(&before), ["2630", "1", "0"]);
    assert_eq!(components(&before), ["19.00", "0.00", "19.00", "49981.00"]);
}

/// Both ends count: one day of employment is 1 employment day, and a start on
/// 30 June 1983 puts 1 day before 1 July 1983.
#[test]
fn employment_days_count_both_ends() {
    let one = run(json!({"employment_start": "2026-09-30"}));
    assert_eq!(days(&one), ["1", "731", "0"]);
    // 100000 x 731 / 732 = 99863.3879...
    assert_eq!(components(&one)[..2], ["99863.39", "0.00"]);
    let edge = run(json!({
        "payment": "1000.00", "employment_start": "1983-06-30",
        "termination_date": "2026-07-01", "last_retirement_day": "2026-07-01",
    }));
    assert_eq!(days(&edge), ["15708", "0", "1"]);
    // 1000 x 1 / 15708 = 0.0636...
    assert_eq!(components(&edge), ["0.00", "0.06", "0.06", "999.94"]);
}

/// The components add up to the payment to the cent, whatever the rounding.
#[test]
fn the_components_add_up_to_the_payment() {
    for payment in ["0.00", "0.01", "33333.33", "999999999999.99"] {
        let result = run(json!({"payment": payment, "employment_start": "1980-02-29"}));
        let [_, _, free, taxable] = components(&result);
        let cents = |s: &str| s.replace('.', "").parse::<i128>().unwrap();
        assert_eq!(cents(free) + cents(taxable), cents(payment), "{payment}");
    }
}

#[test]
fn it_refuses_what_it_cannot_work() {
    for (overrides, expected) in [
        (
            json!({"employment_start": "2026-10-01"}),
            "etp_tax_free_component.dates_invalid",
        ),
        (
            json!({"termination_date": "30/09/2026"}),
            "input.date_invalid",
        ),
        (json!({"last_retirement_day": null}), "input.date_invalid"),
        (json!({"ill_health": "yes"}), "input.flag_invalid"),
        (json!({"medical_certification": 1}), "input.flag_invalid"),
        (json!({"payment": "-1.00"}), "input.amount_invalid"),
        (json!({"payment": "1.001"}), "input.amount_invalid"),
        (json!({"year": "2025-26"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"death_benefit": true}), "input.field_unknown"),
    ] {
        assert_eq!(code(overrides.clone()), expected, "{overrides}");
    }
    let mut missing = input(json!({}));
    missing
        .as_object_mut()
        .unwrap()
        .remove("last_retirement_day");
    let refusal = api::evaluate_coded(COMMAND, &missing).unwrap_err();
    assert_eq!(refusal.code, "input.missing_field");
}

/// 2027-28: the sections set no figure by year.
#[test]
fn it_takes_2027_28() {
    let result = run(json!({"year": "2027-28"}));
    assert_eq!(result["period"], "2027-28");
    // 100000 x 731 / (9588 + 731) = 7084.0198...: 1 July 2000 to
    // 30 September 2026, then to 30 September 2028 across 29 February 2028.
    assert_eq!(days(&result)[..2], ["9588", "731"]);
    assert_eq!(components(&result)[0], "7084.02");
}
