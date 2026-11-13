// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The termination payment worksheet against the ATO's Schedule 11 examples 1
//! to 4 for 2026-27 (last updated 17 June 2026), the Library's genuine
//! redundancy example and ITAA 1997 ss 82-10 and 83-170, read on 14 November
//! 2026 (docs/calculation-evidence.md#termination-payment); then each cap,
//! limit and preservation age boundary, the dollar rounding and the refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// No redundancy, no other payment, no other income; born 1 January 1990, so
/// under preservation age at 30 June 2027.
fn input(overrides: Value) -> Value {
    let mut facts = json!({
        "redundancy_payment": "0.00", "completed_years_of_service": 0,
        "excluded_payment": "0.00", "non_excluded_payment": "0.00",
        "other_taxable_payments": "0.00", "date_of_birth": "1990-01-01",
        "year": "2026-27", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn run(overrides: Value) -> Value {
    api::evaluate("termination-payment", &input(overrides)).unwrap()
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded("termination-payment", &input(overrides))
        .unwrap_err()
        .code
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// The tax-free amount, then the excluded part (code R): its taxable component,
/// the part within the ETP cap, the part above it and its withholding.
fn excluded(result: &Value) -> [&str; 5] {
    [
        "tax_free_redundancy_amount",
        "excluded_taxable_component",
        "excluded_within_cap",
        "excluded_above_cap",
        "excluded_withholding",
    ]
    .map(|key| amount(result, key))
}

/// The non-excluded part (code O): the 2 caps compared, the smaller, the part
/// within and above it, its withholding and the total withheld.
fn non_excluded(result: &Value) -> [&str; 7] {
    [
        "remaining_etp_cap",
        "whole_of_income_cap",
        "non_excluded_cap",
        "non_excluded_within_cap",
        "non_excluded_above_cap",
        "non_excluded_withholding",
        "total_withholding",
    ]
    .map(|key| amount(result, key))
}

/// Schedule 11 example 1: Lloyd, 41, receives a $45,000 ETP, the taxable part
/// left after a genuine redundancy payment's tax-free amount; only the ETP cap
/// applies, and 32% is withheld: $14,400. The example gives only the ETP, so it
/// is held here as a $58,598 genuine redundancy payment with no completed year
/// of service, whose tax-free limit is the $13,598 base amount alone.
#[test]
fn lloyd_has_14400_withheld_as_schedule_11_example_1_says() {
    let lloyd = run(json!({"redundancy_payment": "58598.00", "date_of_birth": "1985-09-01"}));
    assert_eq!(
        excluded(&lloyd),
        ["13598.00", "45000.00", "45000.00", "0.00", "14400.00"]
    );
    assert_eq!(amount(&lloyd, "total_withholding"), "14400.00");
    assert_eq!(lloyd["rates"]["withholding_rate_within_cap"], "0.32");
    assert_eq!(lloyd["rates"]["preservation_age"], "60");
    assert_eq!(lloyd["rates"]["preservation_age_reached"], "false");
    assert_eq!(lloyd["calculation"], "termination_payment");
    assert_eq!(lloyd["period"], "2026-27");
}

/// Example 2: Jane, 50, a $10,000 golden handshake after $84,000 of salary and
/// wages. The whole-of-income cap is $180,000 - $84,000 = $96,000, below the
/// $270,000 ETP cap; the ETP is within it, so 32%: $3,200.
#[test]
fn jane_has_3200_withheld_as_schedule_11_example_2_says() {
    let jane = run(json!({
        "non_excluded_payment": "10000.00", "other_taxable_payments": "84000.00",
        "date_of_birth": "1976-08-01",
    }));
    assert_eq!(
        non_excluded(&jane),
        [
            "270000.00",
            "96000.00",
            "96000.00",
            "10000.00",
            "0.00",
            "3200.00",
            "3200.00"
        ]
    );
}

/// Example 3: Chris, under preservation age, a $120,000 non-genuine redundancy
/// treated as a golden handshake after $100,000 of leave and wages. The cap is
/// $80,000: $80,000 x 32% = $25,600 plus $40,000 x 47% = $18,800, $44,400.
#[test]
fn chris_has_44400_withheld_as_schedule_11_example_3_says() {
    let chris = run(json!({
        "non_excluded_payment": "120000.00", "other_taxable_payments": "100000.00",
    }));
    assert_eq!(
        non_excluded(&chris),
        [
            "270000.00",
            "80000.00",
            "80000.00",
            "80000.00",
            "40000.00",
            "44400.00",
            "44400.00"
        ]
    );
}

/// Example 4: Alec, 30, 5 years of service, a $48,000 genuine redundancy part
/// and a $17,678 gratuity after $140,000 of salary and wages. The tax-free limit
/// is $13,598 + 5 x $6,801 = $47,603, leaving a $397 excluded ETP: 32% is
/// $127.04, so $127. The gratuity's caps are $270,000 - $397 = $269,603 and
/// $180,000 - $140,000 = $40,000; it is within the smaller, so 32% of $17,678 is
/// $5,656.96, $5,657.
#[test]
fn alec_has_127_and_5657_withheld_as_schedule_11_example_4_says() {
    let alec = run(json!({
        "redundancy_payment": "48000.00", "completed_years_of_service": 5,
        "non_excluded_payment": "17678.00", "other_taxable_payments": "140000.00",
        "date_of_birth": "1996-09-01",
    }));
    assert_eq!(amount(&alec, "tax_free_limit"), "47603.00");
    assert_eq!(
        excluded(&alec),
        ["47603.00", "397.00", "397.00", "0.00", "127.00"]
    );
    assert_eq!(
        non_excluded(&alec),
        [
            "269603.00",
            "40000.00",
            "40000.00",
            "17678.00",
            "0.00",
            "5657.00",
            "5784.00"
        ]
    );
}

/// The worksheet's catalogue example is example 4.
#[test]
fn the_catalogue_example_is_schedule_11_example_4() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["termination_payment"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("termination-payment", &facts).unwrap();
    assert_eq!(amount(&result, "total_withholding"), "5784.00");
    assert_eq!(
        catalogue["termination_payment"]["example_evidence"]["note"],
        "docs/calculation-evidence.md#termination-payment"
    );
}

/// The Library's ¶1-420 example, a 2025-26 one, held here as 2026-27: Barbra,
/// 58, a $70,000 genuine redundancy payment after 15 years. The limit is
/// $13,598 + 15 x $6,801 = $115,613, so all of it is tax-free and there is no ETP.
#[test]
fn barbras_redundancy_within_the_limit_is_all_tax_free() {
    let barbra = run(json!({
        "redundancy_payment": "70000.00", "completed_years_of_service": 15,
        "date_of_birth": "1968-01-15",
    }));
    assert_eq!(amount(&barbra, "tax_free_limit"), "115613.00");
    assert_eq!(
        excluded(&barbra),
        ["70000.00", "0.00", "0.00", "0.00", "0.00"]
    );
    assert_eq!(amount(&barbra, "total_withholding"), "0.00");
}

/// A redundancy payment equal to the limit is all tax-free; a cent more is a
/// cent of ETP.
#[test]
fn the_tax_free_limit_is_inclusive() {
    let at = run(json!({"redundancy_payment": "47603.00", "completed_years_of_service": 5}));
    assert_eq!(excluded(&at), ["47603.00", "0.00", "0.00", "0.00", "0.00"]);
    let over = run(json!({"redundancy_payment": "47603.01", "completed_years_of_service": 5}));
    assert_eq!(
        excluded(&over),
        ["47603.00", "0.01", "0.01", "0.00", "0.00"]
    );
}

/// The ETP cap holds exactly $270,000 of the excluded part; a cent more is
/// withheld at 47%. The non-excluded part then has no ETP cap left.
#[test]
fn the_etp_cap_is_inclusive_and_used_by_the_excluded_part_first() {
    let at = run(json!({"excluded_payment": "270000.00", "non_excluded_payment": "1000.00"}));
    assert_eq!(
        excluded(&at),
        ["0.00", "270000.00", "270000.00", "0.00", "86400.00"]
    );
    assert_eq!(
        non_excluded(&at),
        [
            "0.00",
            "180000.00",
            "0.00",
            "0.00",
            "1000.00",
            "470.00",
            "86870.00"
        ]
    );
    let over = run(json!({"excluded_payment": "270000.01"}));
    assert_eq!(
        excluded(&over),
        ["0.00", "270000.01", "270000.00", "0.01", "86400.00"]
    );
    // A remaining ETP cap below the whole-of-income cap is the one applied.
    let left = run(json!({"excluded_payment": "200000.00", "non_excluded_payment": "100000.00"}));
    assert_eq!(
        non_excluded(&left),
        [
            "70000.00",
            "180000.00",
            "70000.00",
            "70000.00",
            "30000.00",
            "36500.00",
            "100500.00"
        ]
    );
}

/// The whole-of-income cap is $180,000 less the other taxable payments, never
/// below nil, and ignores them for an excluded part.
#[test]
fn the_whole_of_income_cap_falls_to_nil_and_spares_excluded_payments() {
    for (other, cap) in [
        ("179999.99", "0.01"),
        ("180000.00", "0.00"),
        ("250000.00", "0.00"),
    ] {
        let result = run(json!({
            "excluded_payment": "1000.00", "non_excluded_payment": "1000.00",
            "other_taxable_payments": other,
        }));
        assert_eq!(amount(&result, "whole_of_income_cap"), cap, "{other}");
        assert_eq!(amount(&result, "non_excluded_cap"), cap, "{other}");
        assert_eq!(amount(&result, "excluded_within_cap"), "1000.00", "{other}");
    }
}

/// Preservation age at 30 June 2027 (SISR reg 6.01(2)): 60 for anyone born
/// after 30 June 1964, so reached when born on or before 30 June 1967; 59 for
/// a birthday from 1 July 1963 to 30 June 1964 and 55 before 1 July 1960, both
/// long reached. $1,000 within the cap is withheld at 17% or 32%.
#[test]
fn preservation_age_on_the_last_day_of_the_year_sets_the_rate() {
    for (born, age, reached, withheld) in [
        ("1967-06-30", "60", "true", "170.00"),
        ("1967-07-01", "60", "false", "320.00"),
        ("1964-07-01", "60", "true", "170.00"),
        ("1964-06-30", "59", "true", "170.00"),
        ("1960-07-01", "56", "true", "170.00"),
        ("1960-06-30", "55", "true", "170.00"),
    ] {
        let result = run(json!({"non_excluded_payment": "1000.00", "date_of_birth": born}));
        let rates = &result["rates"];
        assert_eq!(rates["preservation_age"], age, "{born}");
        assert_eq!(rates["preservation_age_reached"], reached, "{born}");
        assert_eq!(amount(&result, "total_withholding"), withheld, "{born}");
    }
}

/// Withholding rounds to the nearest dollar, 50 cents up (Schedule 11):
/// $50.00 at 17% is $8.50, so $9; $49.99 is $8.4983, so $8; above a nil cap,
/// $50.00 at 47% is $23.50, so $24. Each part rounds on its own, as example 4
/// does: $50.00 of each at 17% is $9 and $9, so $18, not the $17 that
/// rounding their $17.00 total once would give.
#[test]
fn withholding_rounds_to_the_dollar_with_50_cents_up() {
    let older = |payment: &str| {
        run(json!({"non_excluded_payment": payment, "date_of_birth": "1950-01-01"}))
    };
    assert_eq!(amount(&older("50.00"), "non_excluded_withholding"), "9.00");
    assert_eq!(amount(&older("49.99"), "non_excluded_withholding"), "8.00");
    let above = run(json!({
        "non_excluded_payment": "50.00", "other_taxable_payments": "180000.00",
    }));
    assert_eq!(amount(&above, "non_excluded_withholding"), "24.00");
    let both = run(json!({
        "excluded_payment": "50.00", "non_excluded_payment": "50.00",
        "date_of_birth": "1967-06-30",
    }));
    assert_eq!(amount(&both, "excluded_withholding"), "9.00");
    assert_eq!(amount(&both, "non_excluded_withholding"), "9.00");
    assert_eq!(amount(&both, "total_withholding"), "18.00");
}

#[test]
fn each_refusal_has_its_code() {
    for (overrides, expected) in [
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"year": "2025-26"}), "period.unsupported"),
        (
            json!({"redundancy_payment": "-1.00"}),
            "input.amount_invalid",
        ),
        (json!({"excluded_payment": "1.001"}), "input.amount_invalid"),
        (
            json!({"non_excluded_payment": 1000}),
            "input.amount_invalid",
        ),
        (
            json!({"other_taxable_payments": "1e13"}),
            "input.amount_invalid",
        ),
        (
            json!({"completed_years_of_service": -1}),
            "input.years_of_service_invalid",
        ),
        (
            json!({"completed_years_of_service": 151}),
            "input.years_of_service_invalid",
        ),
        (
            json!({"completed_years_of_service": "5"}),
            "input.years_of_service_invalid",
        ),
        (
            json!({"completed_years_of_service": true}),
            "input.years_of_service_invalid",
        ),
        (json!({"date_of_birth": "1990-02-30"}), "input.date_invalid"),
        (json!({"date_of_birth": null}), "input.date_invalid"),
        (json!({"death_benefit": true}), "input.field_unknown"),
    ] {
        assert_eq!(code(overrides.clone()), expected, "{overrides}");
    }
    let mut missing = input(json!({}));
    missing.as_object_mut().unwrap().remove("date_of_birth");
    let refusal = api::evaluate_coded("termination-payment", &missing).unwrap_err();
    assert_eq!(refusal.code, "input.missing_field");
}

#[test]
fn every_result_says_it_is_withholding_not_the_tax_payable() {
    let result = run(json!({"non_excluded_payment": "1000.00"}));
    let warnings = result["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w
            .as_str()
            .unwrap()
            .starts_with("Withholding, not the tax payable")),
        "{warnings:?}"
    );
    assert!(
        result["scope"]
            .as_str()
            .unwrap()
            .contains("death benefit ETPs")
    );
}
