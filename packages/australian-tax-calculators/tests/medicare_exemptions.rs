// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Full and half Medicare levy exemptions (Medicare Levy Act 1986 s 9(1), ITAA
//! 1936 ss 251T and 251U): the ATO's Priya example, the day apportionment after
//! the low-income and family reductions, and the refusals
//! (docs/calculation-evidence.md#exemptions). Every figure is worked by hand in
//! the comments.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "medicare-levy";
const FULL: &str = "medicare_levy_exemption_full_days";
const HALF: &str = "medicare_levy_exemption_half_days";

fn input(income: &str, year: &str, extra: Value) -> Value {
    let mut facts = json!({"taxable_income": income, "section_9a_adjustment_applies": false,
                           "year": year, "scope_confirmed": true});
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn evaluate(income: &str, year: &str, extra: Value) -> Value {
    let input = input(income, year, extra);
    let result = api::evaluate(COMMAND, &input).unwrap();
    // derive explains this command: its result is the plain one, byte for byte.
    let doc = api::evaluate("derive", &json!({"command": COMMAND, "input": input})).unwrap();
    assert_eq!(
        doc["result"].to_string(),
        result.to_string(),
        "derive {input}"
    );
    result
}

fn levy(income: &str, extra: Value) -> String {
    let result = evaluate(income, "2025-26", extra);
    result["amounts"]["medicare_levy"]
        .as_str()
        .unwrap()
        .to_string()
}

fn days(full: i64, half: i64) -> Value {
    json!({FULL: full, HALF: half})
}

/// ATO, Not entitled to Medicare benefits (last updated 22 June 2026), read on
/// 14 November 2026: Priya, on a subclass 482 visa with no dependants, was not
/// entitled to Medicare benefits from 1 July 2025 to 30 June 2026 and enters
/// 365 full exemption days, so she pays no Medicare levy (s 251T). Her income
/// is not given: at $90,000 the levy before s 9 is 2% = $1,800.00.
#[test]
fn ato_priya_example_a_full_year_exemption_leaves_no_levy() {
    let result = evaluate("90000", "2025-26", json!({FULL: 365}));
    assert_eq!(
        result["amounts"],
        json!({"taxable_income_used": "90000.00", "levy_at_full_rate": "1800.00",
               "low_income_reduction": "0.00", "exemption_reduction": "1800.00",
               "medicare_levy": "0.00"})
    );
    assert_eq!(result["rates"]["exemption_full_days"], "365");
    assert_eq!(result["rates"]["exemption_half_days"], "0");
    assert_eq!(result["rates"]["days_in_year"], "365");
    for year in ["2024-25", "2026-27"] {
        let result = evaluate("90000", year, json!({FULL: 365}));
        assert_eq!(result["amounts"]["medicare_levy"], "0.00", "{year}");
    }
}

/// s 251U(3): a half exemption makes the person a prescribed person for half
/// of the period, so a whole year of it halves the levy: 2% x 90,000 / 2.
#[test]
fn a_half_exemption_for_the_whole_year_halves_the_levy() {
    assert_eq!(levy("90000", json!({HALF: 365})), "900.00");
}

/// The ATO's Paul example (Medical exemption from Medicare levy, last updated
/// 22 June 2026) gives no figures: a Gold Card holder whose spouse is below the
/// low-income threshold claims a half exemption. With $90,000 and a spouse on
/// $20,000 in 2025-26, family income 110,000 is past the end of the s 8
/// reduction (2% x 47,238 - 0.08 x 62,762 < 0) and the spouse is not liable,
/// so the levy after s 8 is $1,800.00 and the half exemption leaves $900.00.
#[test]
fn paul_facts_with_a_spouse_below_the_threshold() {
    let family = json!({"spouse_taxable_income": "20000", "spouse_sapto_entitled": false,
                        HALF: 365});
    let result = evaluate("90000", "2025-26", family);
    assert_eq!(result["amounts"]["family_reduction"], "0.00");
    assert_eq!(result["amounts"]["exemption_reduction"], "900.00");
    assert_eq!(result["amounts"]["medicare_levy"], "900.00");
}

/// s 9(1) for part of the year: the levy less the share of the year's days for
/// which the person was a prescribed person, a half day counting as half.
#[test]
fn part_year_days_apportion_the_levy() {
    // 1,000.00 x (365 - 73) / 365 = 800.00.
    let result = evaluate("50000", "2025-26", json!({FULL: 73}));
    assert_eq!(result["amounts"]["exemption_reduction"], "200.00");
    assert_eq!(result["amounts"]["medicare_levy"], "800.00");
    // 1,000.00 x (730 - 200 - 100) / 730 = 589.0410..., and 410.9589... off.
    let result = evaluate("50000", "2025-26", days(100, 100));
    assert_eq!(result["amounts"]["exemption_reduction"], "410.96");
    assert_eq!(result["amounts"]["medicare_levy"], "589.04");
    // 146 half days are 73 prescribed days: the same as 73 full days.
    assert_eq!(levy("50000", json!({HALF: 146})), "800.00");
}

/// s 7(2) and s 8 each work out the levy 'but for' s 9, so the exemption
/// applies to the levy after the low-income and family reductions.
#[test]
fn the_exemption_follows_the_low_income_and_family_reductions() {
    // s 7(2): 10% x (30,000 - 28,011) = 198.90; 198.90 x 219 / 365 = 119.34.
    let result = evaluate("30000", "2025-26", json!({FULL: 146}));
    assert_eq!(result["amounts"]["low_income_reduction"], "401.10");
    assert_eq!(result["amounts"]["exemption_reduction"], "79.56");
    assert_eq!(result["amounts"]["medicare_levy"], "119.34");
    // The ATO's Ashton facts (levy $92.90 after s 8) with a half exemption for
    // the whole year: 92.90 / 2 = 46.45.
    let ashton = json!({"sapto_entitled": true, "spouse_taxable_income": "21700",
                        "spouse_sapto_entitled": true, HALF: 365});
    assert_eq!(levy("49700", ashton), "46.45");
    // s 8(3) sharing (tests/medicare_family.rs): 800.00 - 425.20 x 40 / 70 =
    // 557.0285...; with 73 full days, x 292 / 365 = 445.6228... .
    let shared = json!({"spouse_taxable_income": "30000", "spouse_sapto_entitled": false,
                        "dependent_children": 3, FULL: 73});
    let result = evaluate("40000", "2025-26", shared);
    assert_eq!(result["amounts"]["family_reduction"], "242.97");
    assert_eq!(result["amounts"]["exemption_reduction"], "111.41");
    assert_eq!(result["amounts"]["medicare_levy"], "445.62");
    // s 8(4) (tests/medicare_family.rs): the spouse's excess leaves 800.00 -
    // 505.20 + 98.90 = 393.70 after s 8; a whole year of half days halves it.
    let excess = json!({"spouse_taxable_income": "29000", "spouse_sapto_entitled": false,
                        "dependent_children": 3, HALF: 365});
    let result = evaluate("40000", "2025-26", excess);
    assert_eq!(result["amounts"]["spouse_excess_reduction"], "113.43");
    assert_eq!(result["amounts"]["exemption_reduction"], "196.85");
    assert_eq!(result["amounts"]["medicare_levy"], "196.85");
    // Nil after s 7 stays nil.
    assert_eq!(levy("28011", json!({HALF: 100})), "0.00");
}

/// No days and omitted days give the same result, without exemption amounts.
#[test]
fn no_exemption_days_equal_omitted_days() {
    let omitted = evaluate("50000", "2025-26", json!({}));
    assert_eq!(omitted, evaluate("50000", "2025-26", days(0, 0)));
    assert!(omitted["amounts"].get("exemption_reduction").is_none());
    assert!(omitted["rates"].get("days_in_year").is_none());
    assert_eq!(omitted["amounts"]["medicare_levy"], "1000.00");
}

#[test]
fn exemption_days_the_worksheet_cannot_settle_are_refused() {
    let invalid = "input.day_count_invalid";
    for value in [
        json!(366),
        json!(-1),
        json!("10"),
        json!(1.5),
        json!(true),
        json!(null),
    ] {
        for name in [FULL, HALF] {
            let refused =
                api::evaluate_coded(COMMAND, &input("50000", "2025-26", json!({name: value})))
                    .unwrap_err();
            assert_eq!(refused.code, invalid, "{name} {value}");
            assert_eq!(
                refused.message,
                format!("{name} must be an integer from 0 to 365.")
            );
        }
    }
    // A day counts once (ATO M1 2026, step 1).
    let refused =
        api::evaluate_coded(COMMAND, &input("50000", "2025-26", days(200, 166))).unwrap_err();
    assert_eq!(refused.code, invalid);
    assert_eq!(
        refused.message,
        "medicare_levy_exemption_full_days and medicare_levy_exemption_half_days together must \
         be no more than 365; count a day in both as a full exemption day."
    );
    // A year exactly covered: 1,000.00 x (730 - 400 - 165) / 730 = 226.027... .
    assert_eq!(levy("50000", days(200, 165)), "226.03");
}

const NON_RESIDENT: &str = "non_resident_days";

/// ITAA 1936 s 251U(1)(d): each day of a non-resident period on which every
/// dependant was also a prescribed person (s 251U(2)) is a whole prescribed
/// day, never a half one (s 251U(3) leaves out para (d)); s 9(1) takes those
/// days' share off the levy. ATO M1 2026, category 2, counts them at label V.
#[test]
fn non_resident_days_take_their_share_off_the_levy() {
    // $80,000: 2% = 1,600.00 before s 9; 1,600 x (730 - 184) / 730 = 1,196.712... .
    let result = evaluate("80000", "2025-26", json!({NON_RESIDENT: 92}));
    assert_eq!(result["amounts"]["medicare_levy"], "1196.71");
    let input = input("80000", "2025-26", json!({NON_RESIDENT: 92}));
    let doc = api::evaluate("derive", &json!({"command": COMMAND, "input": input})).unwrap();
    let step = doc["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "exemption_reduction")
        .unwrap();
    assert!(
        step["working"]
            .as_str()
            .unwrap()
            .contains("non_resident_days")
    );
    assert!(step["authority"].as_str().unwrap().contains("s 251U(1)(d)"));
    // With 30 full exemption days as well: 1,600 x (730 - 244) / 730 = 1,065.205... .
    assert_eq!(
        levy("80000", json!({FULL: 30, NON_RESIDENT: 92})),
        "1065.21"
    );
    // With 20 half exemption days: 1,600 x (730 - 184 - 20) / 730 = 1,152.876... .
    assert_eq!(
        levy("80000", json!({HALF: 20, NON_RESIDENT: 92})),
        "1152.88"
    );
    // 2027-28 holds 29 February 2028: 1,600 x (732 - 184) / 732 = 1,197.814... .
    let leap = evaluate("80000", "2027-28", json!({NON_RESIDENT: 92}));
    assert_eq!(leap["amounts"]["medicare_levy"], "1197.81");
}

/// After the low-income reduction (s 7): $30,000 in 2025-26 is in the phase-in
/// band, 10% x (30,000 - 28,011) = 198.90; 198.90 x (730 - 200) / 730 = 144.406... .
#[test]
fn non_resident_days_follow_the_low_income_reduction() {
    assert_eq!(levy("30000", json!({NON_RESIDENT: 100})), "144.41");
}

/// 0 or absent changes nothing, so a full-year resident's result is unchanged.
#[test]
fn no_non_resident_days_equal_omitted_days() {
    for income in ["30000", "80000", "200000"] {
        assert_eq!(
            evaluate(income, "2025-26", json!({NON_RESIDENT: 0})),
            evaluate(income, "2025-26", json!({})),
            "{income}"
        );
    }
}

#[test]
fn non_resident_days_the_worksheet_cannot_settle_are_refused() {
    let invalid = "input.day_count_invalid";
    for value in [json!(-1), json!("10"), json!(1.5), json!(true), json!(366)] {
        let refused = api::evaluate_coded(
            COMMAND,
            &input("50000", "2025-26", json!({NON_RESIDENT: value})),
        )
        .unwrap_err();
        assert_eq!(refused.code, invalid, "{value}");
        assert_eq!(
            refused.message,
            "non_resident_days must be an integer from 0 to 364."
        );
    }
    // Not resident at any time: s 251S(1)(a) levies no Medicare levy.
    let refused = api::evaluate_coded(
        COMMAND,
        &input("50000", "2025-26", json!({NON_RESIDENT: 365})),
    )
    .unwrap_err();
    assert_eq!(refused.code, invalid);
    assert!(
        refused.message.contains("s 251S(1)(a)"),
        "{}",
        refused.message
    );
    // A day counts once (ATO M1 2026, step 1).
    let refused = api::evaluate_coded(
        COMMAND,
        &input(
            "50000",
            "2025-26",
            json!({FULL: 300, HALF: 1, NON_RESIDENT: 65}),
        ),
    )
    .unwrap_err();
    assert_eq!(refused.code, invalid);
    assert_eq!(
        refused.message,
        "medicare_levy_exemption_full_days, medicare_levy_exemption_half_days and \
         non_resident_days together must be no more than 365; count each day once, a \
         non-resident day as non-resident only."
    );
    // One half day left: 1,000 x (730 - 2 x (300 + 64) - 1) / 730 = 1.369... .
    assert_eq!(
        levy("50000", json!({FULL: 300, NON_RESIDENT: 64, HALF: 1})),
        "1.37"
    );
}

/// s 8 before s 9 with non-resident days, on the family cases above: a
/// non-resident day is a whole prescribed day, so 73 of them give what 73 full
/// exemption days give. The s 8(3) case: 557.0285... x 292 / 365 = 445.6228...;
/// the s 8(4) case: 393.70 x 292 / 365 = 314.96.
#[test]
fn non_resident_days_follow_the_family_reduction() {
    let shared = json!({"spouse_taxable_income": "30000", "spouse_sapto_entitled": false,
                        "dependent_children": 3, NON_RESIDENT: 73});
    let result = evaluate("40000", "2025-26", shared);
    assert_eq!(result["amounts"]["family_reduction"], "242.97");
    assert_eq!(result["amounts"]["exemption_reduction"], "111.41");
    assert_eq!(result["amounts"]["medicare_levy"], "445.62");
    let excess = json!({"spouse_taxable_income": "29000", "spouse_sapto_entitled": false,
                        "dependent_children": 3, NON_RESIDENT: 73});
    let result = evaluate("40000", "2025-26", excess);
    assert_eq!(result["amounts"]["exemption_reduction"], "78.74");
    assert_eq!(result["amounts"]["medicare_levy"], "314.96");
}

/// Every amount a non-resident day gives equals a full exemption day's; only
/// the working names it.
#[test]
fn a_non_resident_day_counts_as_a_full_day() {
    for income in ["30000", "50000", "123457"] {
        for days in [1, 73, 364] {
            let non_resident = evaluate(income, "2025-26", json!({NON_RESIDENT: days}));
            let full = evaluate(income, "2025-26", json!({FULL: days}));
            assert_eq!(non_resident["amounts"], full["amounts"], "{income} {days}");
        }
    }
}

/// The working without non-resident days is the one the worksheet has always
/// given: the same formula, authority and rates, in the same order.
#[test]
fn exemption_working_without_non_resident_days_is_unchanged() {
    let input = input("50000", "2025-26", json!({FULL: 73}));
    let doc = api::evaluate("derive", &json!({"command": COMMAND, "input": input})).unwrap();
    let step = doc["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "exemption_reduction")
        .unwrap();
    assert_eq!(
        step["authority"],
        "Medicare Levy Act 1986 s 9(1) and ITAA 1936 s 251U(3): the share of the levy for each full exemption day and half of each half exemption day"
    );
    assert!(
        step["working"].as_str().unwrap().starts_with(
            "levy_at_full_rate - low_income_reduction - (levy_at_full_rate - low_income_reduction) x (2 x days_in_year - (2 x exemption_full_days + exemption_half_days)) / (2 x days_in_year) = "
        ),
        "{}",
        step["working"]
    );
    let days: Vec<&str> = doc["result"]["rates"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .filter(|k| k.contains("days"))
        .collect();
    assert_eq!(
        days,
        ["exemption_full_days", "exemption_half_days", "days_in_year"]
    );
}

/// 2027-28 holds 29 February 2028: 365 non-resident days are the most, and 366
/// is the whole year. 1,600 x (732 - 730) / 732 = 4.371... .
#[test]
fn the_leap_year_bound_on_non_resident_days() {
    let result = evaluate("80000", "2027-28", json!({NON_RESIDENT: 365}));
    assert_eq!(result["amounts"]["medicare_levy"], "4.37");
    let refused = api::evaluate_coded(
        COMMAND,
        &input("80000", "2027-28", json!({NON_RESIDENT: 366})),
    )
    .unwrap_err();
    assert_eq!(refused.code, "input.day_count_invalid");
    assert!(
        refused.message.contains("s 251S(1)(a)"),
        "{}",
        refused.message
    );
}
