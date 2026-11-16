// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `contribution-caps` with the unused concessional cap worked out from each
//! earlier year's contributions (ITAA 1997 s 291-20(3) to (7)), against the
//! ATO's carry-forward examples (docs/calculation-evidence.md#contribution-caps),
//! hand-worked applications in an earlier year, its refusals, and the supplied
//! total, which still gives exactly what it gave.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// The ATO's 'Concessional contributions cap' page (last updated 2 July 2026),
/// read on 16 November 2026 at 17:24 AEDT: Sam's concessional contributions,
/// 2019-20 to 2023-24, from table 1.
const SAM: [(&str, &str); 5] = [
    ("2019-20", "11000"),
    ("2020-21", "12000"),
    ("2021-22", "13000"),
    ("2022-23", "14000"),
    ("2023-24", "15000"),
];

/// A hand-worked history to 2024-25. 2022-23's $50,000 is $22,500 over its
/// $27,500 cap with a balance below $500,000, so s 291-20(4) and (5) apply
/// 2018-19's $20,000 and then $2,500 of 2020-21's $10,000.
const DRAWN: [(&str, &str); 7] = [
    ("2018-19", "5000"),
    ("2019-20", "25000"),
    ("2020-21", "15000"),
    ("2021-22", "27500"),
    ("2022-23", "50000"),
    ("2023-24", "7500"),
    ("2024-25", "30000"),
];

fn earlier(rows: &[(&str, &str)]) -> Value {
    let rows = rows.iter().map(|(year, contributions)| {
        let mut row = json!({"year": year, "concessional_contributions": contributions});
        // Only a year over its cap needs its balance.
        if *year == "2022-23" && *contributions == "50000" {
            row["total_super_balance"] = json!("300000");
        }
        row
    });
    Value::Array(rows.collect())
}

fn input(year: &str, balance: &str, concessional: &str, rows: Value) -> Value {
    json!({
        "total_super_balance": balance, "concessional_contributions": concessional,
        "non_concessional_contributions": "0", "under_75_in_year": true,
        "earlier_concessional_contributions": rows, "defined_benefit_interest": false,
        "year": year, "scope_confirmed": true,
    })
}

/// The plain result, which derive gives too, byte for byte.
fn run(facts: &Value) -> Value {
    let result = api::evaluate("contribution-caps", facts).unwrap();
    let envelope = json!({"command": "contribution-caps", "input": facts});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{facts}");
    result
}

fn refused(facts: &Value) -> (&'static str, String) {
    let refusal = api::evaluate_coded("contribution-caps", facts).unwrap_err();
    (refusal.code, refusal.message)
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} in {result}"))
}

fn with(mut facts: Value, overrides: Value) -> Value {
    for (key, value) in overrides.as_object().unwrap() {
        facts[key] = value.clone();
    }
    facts
}

#[test]
fn sam_carries_his_5_years_unused_cap_into_2024_25() {
    let result = run(&input("2024-25", "450000", "97500", earlier(&SAM)));
    // Table 1: the general cap, unused cap and cumulative unused caps by year.
    let table = [
        ("2019_20", "25000", "14000.00", 14000),
        ("2020_21", "25000", "13000.00", 27000),
        ("2021_22", "27500", "14500.00", 41500),
        ("2022_23", "27500", "13500.00", 55000),
        ("2023_24", "27500", "12500.00", 67500),
    ];
    let mut cumulative = 0;
    for (year, cap, unused, total) in table {
        assert_eq!(result["rates"][format!("concessional_cap_{year}")], cap);
        assert_eq!(amount(&result, &format!("unused_cap_{year}")), unused);
        cumulative += unused.trim_end_matches(".00").parse::<i64>().unwrap();
        assert_eq!(cumulative, total, "{year}");
        // Applied in 2024-25, earliest year first, and nothing left.
        let applied = amount(&result, &format!("applied_{year}_in_2024_25"));
        assert_eq!(applied, unused);
        assert_eq!(amount(&result, &format!("remaining_{year}")), "0.00");
    }
    // 'an individual concessional cap of $97,500 ... used up all his
    // carried-forward caps and has not made excess concessional contributions'
    for (key, value) in [
        ("unused_concessional_cap", "67500.00"),
        ("concessional_available", "97500.00"),
        ("carry_forward_applied", "67500.00"),
        ("concessional_remaining", "0.00"),
        ("excess_concessional", "0.00"),
        ("unused_cap_expiring", "0.00"),
        ("unused_cap_2024_25", "0.00"),
    ] {
        assert_eq!(amount(&result, key), value, "{key}");
    }
}

#[test]
fn a_proposed_contribution_applies_the_earliest_years_first() {
    // Sam with $30,000 contributed: $67,500 still available, then a $40,000
    // proposal applies 2019-20's and 2020-21's amounts and $13,000 of 2021-22's.
    let result = run(&with(
        input("2024-25", "450000", "30000", earlier(&SAM)),
        json!({"proposed_concessional_contribution": "40000"}),
    ));
    assert_eq!(amount(&result, "concessional_remaining"), "67500.00");
    assert_eq!(amount(&result, "contributions_with_proposal"), "70000.00");
    for (year, applied, left) in [
        ("2019_20", Some("14000.00"), "0.00"),
        ("2020_21", Some("13000.00"), "0.00"),
        ("2021_22", Some("13000.00"), "1500.00"),
        ("2022_23", None, "13500.00"),
        ("2023_24", None, "12500.00"),
    ] {
        let key = format!("applied_{year}_in_2024_25");
        assert_eq!(result["amounts"][&key].as_str(), applied, "{key}");
        assert_eq!(amount(&result, &format!("remaining_{year}")), left);
    }
    assert_eq!(amount(&result, "excess_with_proposal"), "0.00");
    // The 2019-20 amount would lapse at the end of 2024-25 (ATO, 'Contributions
    // caps', the unused concessional cap carry forward example).
    assert_eq!(
        result["unused_cap_expiry"],
        json!([
            {"year": "2019-20", "remaining": "0.00", "expires_at_end_of": "2024-25"},
            {"year": "2020-21", "remaining": "0.00", "expires_at_end_of": "2025-26"},
            {"year": "2021-22", "remaining": "1500.00", "expires_at_end_of": "2026-27"},
            {"year": "2022-23", "remaining": "13500.00", "expires_at_end_of": "2027-28"},
            {"year": "2023-24", "remaining": "12500.00", "expires_at_end_of": "2028-29"},
            {"year": "2024-25", "remaining": "0.00", "expires_at_end_of": "2029-30"},
        ])
    );
}

/// The ATO's 'Contributions caps' page (last updated 11 September 2026), read
/// on 16 November 2026 at 17:23 AEDT: Table 2's contributions for 2018-19 to
/// 2021-22, then nil, to 2024-25. Its 2018-19 amount has lapsed by then, and
/// its 2019-20 amount lapses at the end of 2024-25 if unused.
#[test]
fn table_2s_unused_amounts_accrue_and_lapse_after_5_years() {
    let rows = [
        ("2018-19", "3000"),
        ("2019-20", "3000"),
        ("2020-21", "0"),
        ("2021-22", "0"),
        ("2022-23", "0"),
        ("2023-24", "0"),
    ];
    let result = run(&input("2024-25", "450000", "0", earlier(&rows)));
    for (year, cap, unused) in [
        ("2018_19", "25000", "22000.00"),
        ("2019_20", "25000", "22000.00"),
        ("2020_21", "25000", "25000.00"),
        ("2021_22", "27500", "27500.00"),
    ] {
        assert_eq!(result["rates"][format!("concessional_cap_{year}")], cap);
        assert_eq!(amount(&result, &format!("unused_cap_{year}")), unused);
    }
    assert!(result["amounts"]["carried_forward_2018_19"].is_null());
    assert_eq!(amount(&result, "unused_concessional_cap"), "129500.00");
    assert_eq!(amount(&result, "concessional_available"), "159500.00");
    assert_eq!(amount(&result, "unused_cap_expiring"), "22000.00");
    assert_eq!(amount(&result, "unused_cap_2024_25"), "30000.00");
}

#[test]
fn an_earlier_years_increase_uses_the_earliest_unused_cap_first() {
    let result = run(&input("2025-26", "400000", "40000", earlier(&DRAWN)));
    for (key, value) in [
        ("unused_cap_2018_19", "20000.00"),
        ("applied_2018_19_in_2022_23", "20000.00"),
        ("applied_2020_21_in_2022_23", "2500.00"),
        ("unused_cap_2022_23", "0.00"),
        ("carried_forward_2020_21", "7500.00"),
        ("carried_forward_2023_24", "20000.00"),
        ("unused_concessional_cap", "27500.00"),
        ("carry_forward_applied", "10000.00"),
        ("applied_2020_21_in_2025_26", "7500.00"),
        ("applied_2023_24_in_2025_26", "2500.00"),
        ("remaining_2023_24", "17500.00"),
        ("unused_cap_expiring", "0.00"),
    ] {
        assert_eq!(amount(&result, key), value, "{key}");
    }
    assert!(result["amounts"]["applied_2019_20_in_2022_23"].is_null());
    // A $20,000 proposal takes the rest, and $2,500 more is excess.
    let proposed = run(&with(
        input("2025-26", "400000", "40000", earlier(&DRAWN)),
        json!({"proposed_concessional_contribution": "20000"}),
    ));
    assert_eq!(amount(&proposed, "applied_2023_24_in_2025_26"), "20000.00");
    assert_eq!(amount(&proposed, "remaining_2023_24"), "0.00");
    assert_eq!(amount(&proposed, "excess_with_proposal"), "2500.00");
}

/// At or above $500,000 just before the year there is no increase that year
/// (s 291-20(3)(b)), but the basic cap is reported, the unused amounts still
/// age, and the 5th year's lapses at the year's end.
#[test]
fn no_carry_forward_at_a_500000_balance_but_the_amounts_still_age() {
    let result = run(&input("2024-25", "500000", "40000", earlier(&SAM)));
    assert_eq!(amount(&result, "concessional_available"), "30000.00");
    assert_eq!(amount(&result, "carry_forward_applied"), "0.00");
    assert_eq!(amount(&result, "excess_concessional"), "10000.00");
    assert_eq!(amount(&result, "unused_concessional_cap"), "67500.00");
    assert_eq!(amount(&result, "unused_cap_expiring"), "14000.00");
    assert!(result["amounts"]["applied_2019_20_in_2024_25"].is_null());
    let envelope = json!({"command": "contribution-caps", "input": input("2024-25", "500000", "40000", earlier(&SAM))});
    let doc = api::evaluate("derive", &envelope).unwrap();
    let step = doc["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "carry_forward_applied")
        .unwrap();
    assert!(
        step["authority"]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 291-20(3)(b)")
    );
}

/// An earlier year over its cap at or above $500,000 had no increase, so the
/// years before it are not needed.
#[test]
fn an_earlier_year_over_its_cap_at_500000_applies_nothing() {
    let mut rows = earlier(&SAM);
    rows[2] = json!({"year": "2021-22", "concessional_contributions": "40000", "total_super_balance": "500000"});
    let result = run(&input("2024-25", "450000", "0", rows));
    assert_eq!(amount(&result, "unused_cap_2021_22"), "0.00");
    assert_eq!(amount(&result, "carried_forward_2019_20"), "14000.00");
    assert_eq!(amount(&result, "unused_concessional_cap"), "53000.00");
}

#[test]
fn each_refusal_names_what_is_missing() {
    let sam = || input("2024-25", "450000", "30000", earlier(&SAM));
    let code = |facts: Value| refused(&facts).0;
    assert_eq!(
        code(with(sam(), json!({"defined_benefit_interest": true}))),
        "contribution_caps.defined_benefit_interest"
    );
    assert_eq!(
        code(with(sam(), json!({"defined_benefit_interest": "no"}))),
        "input.flag_invalid"
    );
    let mut rows = earlier(&SAM);
    rows.as_array_mut()
        .unwrap()
        .push(json!({"year": "2017-18", "concessional_contributions": "0"}));
    let (code_, message) = refused(&with(
        sam(),
        json!({"earlier_concessional_contributions": rows}),
    ));
    assert_eq!(code_, "contribution_caps.year_before_carry_forward");
    assert!(message.contains("s 291-20(7)"), "{message}");
    // A total across years, as one entry or one amount.
    for total in [
        json!([{"year": "2019-20 to 2023-24", "concessional_contributions": "65000"}]),
        json!("65000"),
        json!([{"year": "2019-20", "concessional_contributions": 11000}]),
        json!([{"year": "2019-20", "concessional_contributions": "11000", "note": "x"}]),
    ] {
        let facts = with(sam(), json!({"earlier_concessional_contributions": total}));
        assert_eq!(code(facts), "contribution_caps.earlier_years_invalid");
    }
    // A missing, repeated or current year.
    for rows in [&SAM[1..], &[SAM[0], SAM[0], SAM[2], SAM[3], SAM[4]][..]] {
        let facts = with(
            sam(),
            json!({"earlier_concessional_contributions": earlier(rows)}),
        );
        assert_eq!(code(facts), "contribution_caps.earlier_years_invalid");
    }
    let mut rows = earlier(&SAM);
    rows.as_array_mut()
        .unwrap()
        .push(json!({"year": "2024-25", "concessional_contributions": "0"}));
    let facts = with(sam(), json!({"earlier_concessional_contributions": rows}));
    assert_eq!(code(facts), "contribution_caps.earlier_years_invalid");
    // Over its cap without its balance, and over it below $500,000 without
    // the years before the first one given.
    let mut rows = earlier(&SAM);
    rows[2] = json!({"year": "2021-22", "concessional_contributions": "40000"});
    let facts = with(sam(), json!({"earlier_concessional_contributions": rows}));
    assert_eq!(code(facts), "contribution_caps.earlier_balance_missing");
    let facts = input("2025-26", "400000", "40000", earlier(&DRAWN[2..]));
    let (code_, message) = refused(&facts);
    assert_eq!(code_, "contribution_caps.earlier_years_short");
    assert!(message.contains("from 2018-19"), "{message}");
    // The supplied total beside the years, a misspelt input, and a proposal
    // without the years.
    for (overrides, field) in [
        (
            json!({"unused_concessional_cap": "67500"}),
            "unused_concessional_cap",
        ),
        (
            json!({"proposed_contribution": "1000"}),
            "proposed_contribution",
        ),
    ] {
        let (code_, message) = refused(&with(sam(), overrides));
        assert_eq!(code_, "input.field_unknown");
        assert!(message.starts_with(field), "{message}");
    }
    let mut facts = with(sam(), json!({"proposed_concessional_contribution": "1000"}));
    facts
        .as_object_mut()
        .unwrap()
        .remove("earlier_concessional_contributions");
    assert_eq!(
        refused(&facts),
        (
            "input.missing_field",
            "earlier_concessional_contributions is required".into()
        )
    );
    assert_eq!(
        code(with(sam(), json!({"year": "2027-28"}))),
        "period.unsupported"
    );
}

/// The supplied total gives exactly the amounts and rates it gave before:
/// the frozen corpus in `tests/parity.rs` replays every earlier call, and for
/// Sam's year it agrees with the total worked out by year.
#[test]
fn the_supplied_total_is_unchanged_and_agrees_with_the_years() {
    let total = json!({
        "total_super_balance": "450000", "concessional_contributions": "97500",
        "unused_concessional_cap": "67500", "non_concessional_contributions": "0",
        "under_75_in_year": true, "year": "2024-25", "scope_confirmed": true,
    });
    let plain = run(&total);
    let keys = |v: &Value, part: &str| -> Vec<String> {
        v[part].as_object().unwrap().keys().cloned().collect()
    };
    assert_eq!(
        keys(&plain, "amounts"),
        [
            "carry_forward_applied",
            "concessional_available",
            "concessional_remaining",
            "excess_concessional",
            "non_concessional_available",
            "non_concessional_remaining",
            "excess_non_concessional",
        ]
    );
    assert_eq!(
        keys(&plain, "rates"),
        [
            "concessional_cap",
            "non_concessional_cap",
            "general_transfer_balance_cap",
            "carry_forward_balance_limit",
            "non_concessional_cap_multiple",
            "bring_forward_period_years",
        ]
    );
    assert!(plain.get("unused_cap_expiry").is_none());
    let by_year = run(&input("2024-25", "450000", "97500", earlier(&SAM)));
    for key in keys(&plain, "amounts") {
        assert_eq!(plain["amounts"][&key], by_year["amounts"][&key], "{key}");
    }
    for key in keys(&plain, "rates") {
        assert_eq!(plain["rates"][&key], by_year["rates"][&key], "{key}");
    }
}
