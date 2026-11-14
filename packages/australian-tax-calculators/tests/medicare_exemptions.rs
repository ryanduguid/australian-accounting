// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

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
    api::evaluate(COMMAND, &input(income, year, extra)).unwrap()
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
