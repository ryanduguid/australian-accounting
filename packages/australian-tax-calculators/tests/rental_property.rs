// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `rental-property` against the ATO's Rental properties guide 2026 (NAT
//! 1729-06.2026, Examples 12, 20, 21, 23, 29, 33 and 34), the rental skill's
//! mixed-use early repayment card and ITAA 1997 Division 43's rate table and
//! limit, s 40-27 and s 26-31, all read on 22 November 2026
//! (docs/calculation-evidence.md#rental-property).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A property held all of 2025-26 and let all year, at a full interest, with
/// nothing else; each test sets what it needs.
fn base() -> Value {
    json!({
        "ownership_share": "1", "acquired": "2020-07-01", "disposed": null,
        "days_available": 365, "section_26_50_applies": false,
        "income": {}, "expenses": {}, "rental_only_expenses": {},
        "loans": [], "borrowing_loans": [], "capital_works": [],
        "depreciating_assets": [], "year": "2025-26", "scope_confirmed": true,
    })
}

fn with(changes: Value) -> Value {
    let mut input = base();
    for (key, value) in changes.as_object().unwrap() {
        input[key] = value.clone();
    }
    input
}

fn run(input: &Value) -> Value {
    api::evaluate("rental-property", input).unwrap()
}

fn amount(input: &Value, key: &str) -> String {
    run(input)["amounts"][key].as_str().unwrap().to_string()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("rental-property", input)
        .unwrap_err()
        .code
}

/// Example 34: $50,000 rent, $37,000 of costs apportioned by 330 of 365 days
/// and $3,000 of agent commission and advertising that are not. The ATO
/// gives $36,452 of deductions and $13,548 net, $6,774 to each owner, all to
/// the dollar. It is the catalogue's example.
#[test]
fn example_34_apportions_by_days_and_splits_by_interest() {
    let whole = with(json!({
        "days_available": 330,
        "income": {"rental_income": "50000.00"},
        "expenses": {"sundry": "37000.00"},
        "rental_only_expenses": {"property_agent_fees": "3000.00"},
    }));
    let r = run(&whole);
    assert_eq!(r["amounts"]["total_deductions"], "36452.05");
    assert_eq!(r["amounts"]["net_rent"], "13547.95");
    let mut half = whole.clone();
    half["ownership_share"] = json!("0.5");
    let r = run(&half);
    assert_eq!(r["amounts"]["gross_rent"], "25000.00");
    assert_eq!(r["amounts"]["sundry"], "16726.03");
    assert_eq!(r["amounts"]["property_agent_fees"], "1500.00");
    assert_eq!(r["amounts"]["net_rent"], "6773.97");
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["rental_property"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_eq!(run(&facts)["amounts"]["net_rent"], "6773.97");
}

/// Example 12: $14,108 of interest on $209,000, of which $170,000 bought the
/// rental: $11,475 deductible (to the dollar). A co-borrower's half of the
/// interest gives half of it, $5,737.70 unrounded where the ATO halves the
/// rounded figure to $5,737.50.
#[test]
fn example_12_apportions_a_mixed_use_loan() {
    let loan = |interest: &str| {
        with(json!({"loans": [{
            "interest": interest, "principal_for_rental": "170000.00",
            "principal_total": "209000.00", "redrawn_in_year": false,
        }]}))
    };
    assert_eq!(amount(&loan("14108.00"), "interest"), "11475.41");
    assert_eq!(amount(&loan("7054.00"), "interest"), "5737.70");
    // The loan is the owner's own: the legal interest does not divide it.
    let mut shared = loan("14108.00");
    shared["ownership_share"] = json!("0.5");
    assert_eq!(amount(&shared, "interest"), "11475.41");
    // Private use days take their share of interest too.
    let mut part = loan("14108.00");
    part["days_available"] = json!(73);
    assert_eq!(amount(&part, "interest"), "2295.08");
}

fn borrowing(start: &str, end: &str, repaid: Value, amount: &str, using: Value) -> Value {
    json!([{
        "borrowing_expenses": amount, "loan_start": start, "loan_term_end": end,
        "repaid_on": repaid, "income_use": using, "section_26_50_applies": false,
    }])
}

/// Example 20: $1,670 of borrowing expenses on a 20-year loan from 17 July,
/// $170,000 of $209,000 for the rental: $259.62, then $271.53 and $271.53,
/// through borrowing-expenses-schedule.
#[test]
fn example_20_borrowing_expenses_come_from_the_schedule() {
    let share = "0.8133971291866028708133971291866028708134";
    let loans = borrowing(
        "2024-07-17",
        "2044-07-16",
        Value::Null,
        "1670.00",
        json!(share),
    );
    for (year, expected) in [
        ("2024-25", "259.62"),
        ("2025-26", "271.53"),
        ("2026-27", "271.53"),
    ] {
        let input = with(json!({"year": year, "acquired": "2024-07-01", "borrowing_loans": loans}));
        assert_eq!(amount(&input, "borrowing_expenses"), expected, "{year}");
    }
    let r = run(&with(json!({"borrowing_loans": loans})));
    assert_eq!(
        r["borrowing_loans"][0]["schedule"][0]["deduction"],
        "259.62"
    );
}

/// The skill's card (skills PR 464): $1,826 from 1 July 2024 on a 10-year
/// loan repaid on 31 March 2026, used 75% then 50%: $273.75, then $730.50
/// on the $1,461 that remains, not on the original less $273.75.
#[test]
fn a_mixed_use_loan_repaid_early_reduces_by_the_maximum() {
    let loans = borrowing(
        "2024-07-01",
        "2034-06-30",
        json!("2026-03-31"),
        "1826.00",
        json!({"2024-25": "0.75", "2025-26": "0.5"}),
    );
    let input =
        |year| with(json!({"year": year, "acquired": "2024-07-01", "borrowing_loans": loans}));
    assert_eq!(amount(&input("2024-25"), "borrowing_expenses"), "273.75");
    assert_eq!(amount(&input("2025-26"), "borrowing_expenses"), "730.50");
}

fn works(expenditure: &str, kind: &str, began: &str, first_used: &str) -> Value {
    let mut item = json!({
        "construction_expenditure": expenditure, "kind": kind,
        "began": began, "first_used": first_used,
    });
    if kind == "building" {
        item["taken_to_begin_before_16_september_1987"] = json!(false);
    }
    item
}

/// Example 29: $115,800 of construction expenditure on a house built in
/// 1991, acquired on 20 July 2025: 2.5% for 346 days over 365, $2,744.
#[test]
fn example_29_capital_works_for_part_of_the_year() {
    let item = works("115800.00", "building", "1991-08-01", "1992-03-01");
    let input = with(json!({
        "acquired": "2025-07-20", "days_available": 346, "capital_works": [item],
    }));
    let r = run(&input);
    assert_eq!(r["amounts"]["capital_works"], "2744.30");
    assert_eq!(r["capital_works"][0]["rate"], "0.025");
    assert_eq!(r["capital_works"][0]["days_used"], 346);
    // The use period ends the day before the acquisition (s 43-230(2)(b)).
    assert_eq!(r["capital_works"][0]["use_period_days"], 12194);
    assert_eq!(
        r["capital_works"][0]["undeducted_construction_expenditure"],
        "19083.21"
    );
    assert_eq!(r["rates"]["days_held"], "346");
}

fn rate_of(kind: &str, began: &str, contract: Option<bool>) -> Value {
    let mut item = works("100000.00", kind, began, "2015-07-01");
    let key = "taken_to_begin_before_16_september_1987";
    match contract {
        Some(flag) => item[key] = json!(flag),
        None => {
            item.as_object_mut().unwrap().remove(key);
        }
    }
    run(&with(json!({"capital_works": [item]})))["capital_works"][0]["rate"].clone()
}

/// ss 43-20, 43-25 and 43-220 and the guide's Tables 1 and 2, at each edge.
#[test]
fn capital_works_rate_by_when_works_began() {
    assert_eq!(rate_of("building", "1979-08-21", None), Value::Null);
    for began in ["1979-08-22", "1985-07-17"] {
        let item = works("1.00", "building", began, "2015-07-01");
        let input = with(json!({"capital_works": [item]}));
        assert_eq!(
            code(&input),
            "rental_property.works_intended_use",
            "{began}"
        );
    }
    assert_eq!(rate_of("building", "1985-07-18", None), "0.04");
    assert_eq!(rate_of("building", "1987-09-15", None), "0.04");
    assert_eq!(rate_of("building", "1987-09-16", Some(false)), "0.025");
    assert_eq!(rate_of("building", "1987-09-16", Some(true)), "0.04");
    assert_eq!(rate_of("building", "1992-02-26", Some(true)), "0.04");
    // s 43-220 has no end date: the fact is needed for any building begun
    // after 15 September 1987.
    assert_eq!(rate_of("building", "1992-02-27", Some(false)), "0.025");
    assert_eq!(rate_of("building", "1992-02-27", Some(true)), "0.04");
    assert_eq!(
        rate_of("structural_improvement", "1992-02-26", None),
        Value::Null
    );
    assert_eq!(
        rate_of("structural_improvement", "1992-02-27", None),
        "0.025"
    );
    for began in ["1990-01-01", "1992-02-27", "2010-01-01"] {
        let mut item = works("1.00", "building", began, "2015-07-01");
        item.as_object_mut()
            .unwrap()
            .remove("taken_to_begin_before_16_september_1987");
        let missing = with(json!({"capital_works": [item]}));
        assert_eq!(code(&missing), "input.missing_field", "{began}");
    }
    let bad = with(json!({"capital_works": [works("1.00", "shed", "2000-01-01", "2015-07-01")]}));
    assert_eq!(code(&bad), "rental_property.fact_invalid");
}

/// A full year is 365 days over 365, and a leap year's 366 days over 365
/// (s 43-210; Example 29's note).
#[test]
fn capital_works_divides_by_365_in_a_leap_year() {
    let item = works("100000.00", "building", "2010-01-01", "2010-07-01");
    let input = |year: &str, days: i64| {
        with(json!({"year": year, "days_available": days, "capital_works": [item]}))
    };
    assert_eq!(amount(&input("2025-26", 365), "capital_works"), "2500.00");
    assert_eq!(amount(&input("2023-24", 366), "capital_works"), "2506.85");
}

/// The undeducted construction expenditure limits the deduction (s 43-15(1)
/// and s 43-240): 4% works first used on 1 January 1986 are spent after 25
/// years; first used on 1 December 2000, 8,978 days to 30 June 2025 leave
/// $1,610.96, less than a year's $4,000.
#[test]
fn capital_works_stop_at_the_undeducted_expenditure() {
    let spent = works("100000.00", "building", "1985-10-01", "1986-01-01");
    assert_eq!(
        amount(&with(json!({"capital_works": [spent]})), "capital_works"),
        "0.00"
    );
    let item = works("100000.00", "building", "1987-01-01", "2000-12-01");
    let r = run(&with(json!({"capital_works": [item]})));
    assert_eq!(r["capital_works"][0]["use_period_days"], 8978);
    assert_eq!(
        r["capital_works"][0]["undeducted_construction_expenditure"],
        "1610.96"
    );
    assert_eq!(r["amounts"]["capital_works"], "1610.96");
}

/// Works first used during a part-year let all year count from first use;
/// with private use in the year their days are not known and are refused.
#[test]
fn works_first_used_in_the_year() {
    let item = works("36500.00", "building", "2025-09-01", "2026-01-01");
    let all = with(json!({"capital_works": [item]}));
    assert_eq!(amount(&all, "capital_works"), "452.50");
    let mut part = all.clone();
    part["days_available"] = json!(300);
    assert_eq!(code(&part), "rental_property.days_unclear");
    let later =
        with(json!({"capital_works": [works("1.00", "building", "2026-07-01", "2026-07-02")]}));
    assert_eq!(amount(&later, "capital_works"), "0.00");
    let backwards =
        with(json!({"capital_works": [works("1.00", "building", "2020-01-02", "2020-01-01")]}));
    assert_eq!(code(&backwards), "rental_property.fact_conflict");
}

fn asset(acquired: &str, held_first: bool, private: bool) -> Value {
    json!({
        "decline_in_value": "1000.00", "acquired": acquired, "first_used": acquired,
        "held_when_first_used": held_first, "residence_or_private_use": private,
        "low_value_pool": false,
    })
}

fn decline(assets: Value) -> Value {
    run(&with(
        json!({"acquired": "2025-08-20", "days_available": 315, "depreciating_assets": assets}),
    ))
}

/// Example 21: new curtains are deductible; the carpet the previous owner
/// installed, a friend's television and a dryer from her home are not
/// (s 40-27(2)(c) and (d)).
#[test]
fn example_21_second_hand_assets_are_nil() {
    let mut carpet = asset("2025-08-20", false, false);
    carpet["new_premises_exception"] = json!(false);
    let mut television = asset("2025-08-25", false, false);
    television["new_premises_exception"] = json!(false);
    let dryer = asset("2019-01-01", true, true);
    let r = decline(json!([
        asset("2025-08-25", true, false),
        carpet,
        television
    ]));
    assert_eq!(r["amounts"]["decline_in_value"], "1000.00");
    let rows = r["depreciating_assets"].as_array().unwrap();
    assert!(rows[0]["section_40_27"].is_null());
    assert!(
        rows[1]["section_40_27"]
            .as_str()
            .unwrap()
            .contains("40-27(2)(c)")
    );
    let r = decline(json!([dryer]));
    assert_eq!(r["amounts"]["decline_in_value"], "0.00");
    assert!(
        r["depreciating_assets"][0]["section_40_27"]
            .as_str()
            .unwrap()
            .contains("40-27(2)(d)")
    );
}

/// Example 23: assets supplied with new apartments within 6 months, one
/// tenanted for 4 months, are deductible (s 40-27(4) and (5)); a low-value
/// pool is outside s 40-27(2) (s 40-27(6)).
#[test]
fn example_23_new_premises_and_pools_are_deductible() {
    let mut supplied = asset("2025-08-20", false, false);
    supplied["new_premises_exception"] = json!(true);
    let mut pooled = asset("2025-08-20", false, true);
    pooled["low_value_pool"] = json!(true);
    let r = decline(json!([supplied, pooled]));
    assert_eq!(r["amounts"]["decline_in_value"], "2000.00");
    // Apportioned like the rest: 1,000 x 0.5 x 200 / 315.
    let mut half =
        with(json!({"acquired": "2025-08-20", "days_available": 200, "ownership_share": "0.5"}));
    half["depreciating_assets"] = json!([asset("2025-08-20", true, false)]);
    assert_eq!(amount(&half, "decline_in_value"), "317.46");
}

/// Act No. 126 of 2017 Schedule 2 item 13: from 7.30 pm on 9 May 2017.
#[test]
fn section_40_27_applies_from_9_may_2017() {
    let mut after = asset("2017-05-10", false, false);
    after["new_premises_exception"] = json!(false);
    assert_eq!(
        decline(json!([after]))["amounts"]["decline_in_value"],
        "0.00"
    );
    assert_eq!(
        code(&with(
            json!({"depreciating_assets": [asset("2017-05-09", false, false)]})
        )),
        "rental_property.asset_acquired_9_may_2017"
    );
    let mut before = asset("2017-05-08", false, true);
    before["deductible_in_2016_17"] = json!(true);
    assert_eq!(
        decline(json!([before]))["amounts"]["decline_in_value"],
        "1000.00"
    );
    // Acquired earlier with nothing deductible for 2016-17: s 40-27 applies
    // only if the start time was by 30 June 2017 (item 13(2)).
    let mut late_start = asset("2017-05-08", true, true);
    late_start["deductible_in_2016_17"] = json!(false);
    late_start["first_used"] = json!("2017-07-01");
    assert_eq!(
        decline(json!([late_start.clone()]))["amounts"]["decline_in_value"],
        "1000.00"
    );
    late_start["first_used"] = json!("2017-05-10");
    assert_eq!(
        decline(json!([late_start]))["amounts"]["decline_in_value"],
        "0.00"
    );
    let missing = asset("2017-05-08", true, false);
    assert_eq!(
        code(&with(json!({"depreciating_assets": [missing]}))),
        "input.missing_field"
    );
    let not_new = asset("2020-01-01", false, false);
    assert_eq!(
        code(&with(json!({"depreciating_assets": [not_new]}))),
        "input.missing_field"
    );
}

/// Example 33's worksheet for 2025-26 with this file's interest, borrowing,
/// capital works and decline: $20,193.33 of deductions and -$10,893.33. The
/// ATO's $20,629 claims $436 of travel, which s 26-31 denies (its Note 2).
#[test]
fn example_33_worksheet_denies_travel() {
    let cw = works("115800.00", "building", "1991-08-01", "1992-03-01");
    let input = with(json!({
        "acquired": "2025-07-20", "days_available": 346,
        "income": {"rental_income": "8500.00", "other_rental_income": "800.00"},
        "expenses": {
            "body_corporate_fees": "500.00", "cleaning": "100.00", "council_rates": "700.00",
            "gardening": "350.00", "insurance": "495.00", "land_tax": "200.00",
            "legal_expenses": "150.00", "pest_control": "50.00", "repairs_and_maintenance": "1000.00",
            "stationery_telephone_postage": "80.00", "water_charges": "350.00", "sundry": "95.00",
            "travel": "436.00",
        },
        "rental_only_expenses": {"advertising_for_tenants": "48.00", "property_agent_fees": "800.00"},
        "loans": [{"interest": "14108.00", "principal_for_rental": "170000.00",
                   "principal_total": "209000.00", "redrawn_in_year": false}],
        "borrowing_loans": borrowing("2025-07-17", "2045-07-16", Value::Null, "1670.00",
            json!("0.8133971291866028708133971291866028708134")),
        "capital_works": [cw],
        "depreciating_assets": [asset("2025-07-20", true, false)],
    }));
    let mut input = input;
    input["depreciating_assets"][0]["decline_in_value"] = json!("796.00");
    let r = run(&input);
    assert_eq!(r["amounts"]["gross_rent"], "9300.00");
    assert_eq!(r["amounts"]["travel"], "0.00");
    assert_eq!(r["amounts"]["travel_denied"], "436.00");
    assert_eq!(r["amounts"]["borrowing_expenses"], "259.62");
    assert_eq!(r["amounts"]["total_deductions"], "20193.33");
    assert_eq!(r["amounts"]["net_rent"], "-10893.33");
    // With $90,000 of other income: tax on $79,107 against $90,000 at the
    // 2025-26 resident scale (16% to $45,000, 30% to $135,000).
    input["other_taxable_income"] = json!("90000");
    let r = run(&input);
    assert_eq!(r["amounts"]["taxable_income_with_rental"], "79107.00");
    assert_eq!(r["amounts"]["tax_without_rental"], "17788.00");
    assert_eq!(r["amounts"]["tax_with_rental"], "14520.10");
    assert_eq!(r["amounts"]["tax_effect"], "3267.90");
    let derived = api::evaluate(
        "derive",
        &json!({"command": "rental-property", "input": input}),
    )
    .unwrap();
    let step = |id: &str| {
        derived["steps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == id)
            .unwrap()["working"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert!(
        step("tax_with_rental").contains("resident_tax(79107)"),
        "{}",
        step("tax_with_rental")
    );
    assert!(
        step("interest").ends_with("= 11475.41"),
        "{}",
        step("interest")
    );
}

/// A loss above other income leaves nil taxable income; the effect is for
/// 2024-25 to 2026-27 only, on whole dollars.
#[test]
fn tax_effect_years_and_floor() {
    let loss =
        with(json!({"expenses": {"council_rates": "30000.00"}, "other_taxable_income": "20000"}));
    let r = run(&loss);
    assert_eq!(r["amounts"]["taxable_income_with_rental"], "0.00");
    assert_eq!(r["amounts"]["tax_effect"], "288.00");
    let mut early = loss.clone();
    early["year"] = json!("2024-25");
    assert!(run(&early)["amounts"]["tax_effect"].is_string());
    early["year"] = json!("2023-24");
    early["days_available"] = json!(366);
    assert_eq!(code(&early), "rental_property.tax_effect_year");
    early
        .as_object_mut()
        .unwrap()
        .remove("other_taxable_income");
    assert_eq!(amount(&early, "net_rent"), "-30000.00");
    let mut cents = loss.clone();
    cents["other_taxable_income"] = json!("20000.50");
    assert_eq!(code(&cents), "input.amount_not_whole_dollars");
}

/// 2020-21 to 2026-27; 2019-20 and 2027-28 are refused, the latter naming
/// s 26-155.
#[test]
fn income_years_supported() {
    let at =
        |year: &str| with(json!({"year": year, "acquired": "2019-07-01", "days_available": 365}));
    assert_eq!(amount(&at("2020-21"), "net_rent"), "0.00");
    assert_eq!(amount(&at("2026-27"), "net_rent"), "0.00");
    assert_eq!(code(&at("2019-20")), "period.unsupported");
    let late = api::evaluate_coded("rental-property", &at("2027-28")).unwrap_err();
    assert_eq!(late.code, "period.unsupported");
    assert!(late.message.contains("26-155"));
}

#[test]
fn refusals() {
    assert_eq!(
        code(&with(json!({"section_26_50_applies": true}))),
        "rental_property.s26_50_denied"
    );
    let redraw = with(
        json!({"loans": [{"interest": "1.00", "principal_for_rental": "1.00",
        "principal_total": "2.00", "redrawn_in_year": true}]}),
    );
    assert_eq!(code(&redraw), "rental_property.interest_redraw");
    let over = with(
        json!({"loans": [{"interest": "1.00", "principal_for_rental": "3.00",
        "principal_total": "2.00", "redrawn_in_year": false}]}),
    );
    assert_eq!(code(&over), "rental_property.fact_conflict");
    assert_eq!(
        code(&with(json!({"days_available": 366}))),
        "rental_property.fact_conflict"
    );
    assert_eq!(
        code(&with(json!({"acquired": "2026-07-01"}))),
        "rental_property.fact_conflict"
    );
    assert_eq!(
        code(&with(json!({"disposed": "2025-06-30"}))),
        "rental_property.fact_conflict"
    );
    assert_eq!(
        code(&with(json!({"ownership_share": "0"}))),
        "rental_property.fact_invalid"
    );
    assert_eq!(
        code(&with(json!({"ownership_share": "1.01"}))),
        "rental_property.fact_invalid"
    );
    assert_eq!(
        code(&with(json!({"loans": {}}))),
        "rental_property.fact_invalid"
    );
    assert_eq!(
        code(&with(json!({"expenses": {"interest": "1.00"}}))),
        "input.field_unknown"
    );
    assert_eq!(
        code(&with(json!({"expenses": {"sundry": "-1.00"}}))),
        "input.amount_invalid"
    );
    assert_eq!(
        code(&with(json!({"scope_confirmed": false}))),
        "scope.not_confirmed"
    );
    // A borrowing loan the schedule refuses keeps its code.
    let early = borrowing(
        "2016-07-01",
        "2026-06-30",
        Value::Null,
        "500.00",
        json!("1"),
    );
    assert_eq!(
        code(&with(json!({"borrowing_loans": early}))),
        "borrowing.loan_before_2017_18"
    );
}

/// A sale in the year ends the days held: rates for 1 July to 31 December
/// held, let throughout, are deducted in full.
#[test]
fn disposal_in_the_year() {
    let input = with(json!({"disposed": "2025-12-31", "days_available": 184,
        "expenses": {"council_rates": "1000.00"}}));
    let r = run(&input);
    assert_eq!(r["amounts"]["council_rates"], "1000.00");
    assert_eq!(r["held"]["to"], "2025-12-31");
}

/// An asset first used after the property was first held counts all its
/// decline when the property was available every day held, and is refused
/// otherwise; one first used after the year, or before it was acquired yet
/// said to be held when first used, is a conflict.
#[test]
fn asset_first_used_in_the_year() {
    let mut curtains = asset("2026-01-01", true, false);
    curtains["decline_in_value"] = json!("100.00");
    let all = with(json!({"depreciating_assets": [curtains]}));
    assert_eq!(amount(&all, "decline_in_value"), "100.00");
    let mut private_half = all.clone();
    private_half["days_available"] = json!(181);
    assert_eq!(code(&private_half), "rental_property.days_unclear");
    let mut late = asset("2026-07-01", true, false);
    late["first_used"] = json!("2026-07-01");
    assert_eq!(
        code(&with(json!({"depreciating_assets": [late]}))),
        "rental_property.fact_conflict"
    );
    let mut backwards = asset("2024-01-02", true, false);
    backwards["first_used"] = json!("2024-01-01");
    assert_eq!(
        code(&with(json!({"depreciating_assets": [backwards]}))),
        "rental_property.fact_conflict"
    );
    // A contract made on 8 May 2017 for an asset first used on 10 May, which
    // was deductible for 2016-17, is outside s 40-27 (item 13).
    let mut contracted = asset("2017-05-08", false, true);
    contracted["first_used"] = json!("2017-05-10");
    contracted["deductible_in_2016_17"] = json!(true);
    assert_eq!(
        amount(
            &with(json!({"depreciating_assets": [contracted]})),
            "decline_in_value"
        ),
        "1000.00"
    );
}

/// A property acquired on 1 January 2026 and available every day held: an
/// asset first used in March takes all its stated decline, which is for its
/// own days.
#[test]
fn asset_first_used_after_a_part_year_acquisition() {
    let mut oven = asset("2026-03-01", true, false);
    oven["decline_in_value"] = json!("250.00");
    let input = with(json!({
        "acquired": "2026-01-01", "days_available": 181, "depreciating_assets": [oven],
    }));
    assert_eq!(amount(&input, "decline_in_value"), "250.00");
}

/// The start time is the taxpayer's own first use, which can come before
/// they acquired the asset: a borrowed asset used privately from July 2016 and
/// bought on 8 May 2017, with nothing deductible for 2016-17, is under s 40-27
/// (item 13(2)) and is not held when first used.
#[test]
fn asset_used_before_it_was_acquired() {
    let mut borrowed = asset("2017-05-08", false, true);
    borrowed["first_used"] = json!("2016-07-01");
    borrowed["deductible_in_2016_17"] = json!(false);
    let r = decline(json!([borrowed.clone()]));
    assert_eq!(r["amounts"]["decline_in_value"], "0.00");
    assert!(
        r["depreciating_assets"][0]["section_40_27"]
            .as_str()
            .unwrap()
            .contains("40-27(2)(d)")
    );
    borrowed["held_when_first_used"] = json!(true);
    assert_eq!(
        code(&with(json!({"depreciating_assets": [borrowed]}))),
        "rental_property.fact_conflict"
    );
}

/// An asset used before it was acquired counts for the rental from its
/// acquisition: one acquired after the property was disposed of is a
/// conflict, and one acquired part way through a year with private use is
/// refused.
#[test]
fn asset_rental_use_starts_at_the_later_of_first_use_and_acquisition() {
    let mut pooled = asset("2026-03-01", false, false);
    pooled["decline_in_value"] = json!("100.00");
    pooled["first_used"] = json!("2025-01-01");
    pooled["low_value_pool"] = json!(true);
    let disposed = with(json!({
        "disposed": "2025-12-31", "days_available": 184, "depreciating_assets": [pooled.clone()],
    }));
    assert_eq!(code(&disposed), "rental_property.fact_conflict");
    let private_half =
        with(json!({"days_available": 181, "depreciating_assets": [pooled.clone()]}));
    assert_eq!(code(&private_half), "rental_property.days_unclear");
    let every_day = with(json!({"depreciating_assets": [pooled.clone()]}));
    assert_eq!(amount(&every_day, "decline_in_value"), "100.00");
    // Outside the pool and used privately, s 40-27(2)(d) makes it nil
    // whatever its days.
    pooled["low_value_pool"] = json!(false);
    pooled["residence_or_private_use"] = json!(true);
    let nil = with(json!({"days_available": 181, "depreciating_assets": [pooled]}));
    assert_eq!(amount(&nil, "decline_in_value"), "0.00");
}

/// A loan with every fact given, `changes` replacing or adding facts.
fn loan_with(changes: Value) -> Value {
    let mut loan = json!({
        "interest": "1000.00", "principal_for_rental": "20000.00",
        "principal_total": "20000.00", "redrawn_in_year": false,
    });
    for (key, value) in changes.as_object().unwrap() {
        loan[key] = value.clone();
    }
    loan
}

/// A list item that is not an object, a fact the list does not take, a
/// missing required fact and a flag that is not a boolean are each refused,
/// naming the list.
#[test]
fn malformed_list_items_are_refused() {
    let refused =
        |changes: Value| api::evaluate_coded("rental-property", &with(changes)).unwrap_err();
    let not_object = refused(json!({"loans": ["interest"]}));
    assert_eq!(not_object.code, "rental_property.fact_invalid");
    assert_eq!(
        not_object.message,
        "loan 1: each of a loan must be an object."
    );
    let unknown = refused(json!({"loans": [loan_with(json!({"offset": "0"}))]}));
    assert_eq!(unknown.code, "input.field_unknown");
    assert!(
        unknown
            .message
            .ends_with("offset is not a fact a loan takes")
    );
    let missing = refused(json!({"loans": [{"interest": "1000.00"}]}));
    assert_eq!(missing.code, "input.missing_field");
    assert!(
        missing
            .message
            .ends_with("a loan: principal_for_rental is required")
    );
    let flag = refused(json!({"loans": [loan_with(json!({"redrawn_in_year": "no"}))]}));
    assert_eq!(flag.code, "input.flag_invalid");
    assert!(flag.message.ends_with("redrawn_in_year must be a boolean."));
}

/// An asset date that is not a calendar date, a disposal that is neither a
/// date nor null, days that are not a whole number and income that is not an
/// object of lines are refused as invalid facts.
#[test]
fn malformed_dates_days_and_lines_are_refused() {
    let mut bad_date = asset("2020-07-01", true, false);
    bad_date["acquired"] = json!("2020-13-01");
    for (changes, message) in [
        (
            json!({"depreciating_assets": [bad_date]}),
            "acquired must be a YYYY-MM-DD date.",
        ),
        (
            json!({"disposed": 2026}),
            "disposed must be a YYYY-MM-DD date or null.",
        ),
        (
            json!({"days_available": "365"}),
            "days_available must be a whole number of days.",
        ),
        (
            json!({"income": ["rent"]}),
            "income must be an object of amounts by line.",
        ),
    ] {
        let error = api::evaluate_coded("rental-property", &with(changes)).unwrap_err();
        assert_eq!(error.code, "rental_property.fact_invalid");
        assert!(error.message.ends_with(message));
    }
}
