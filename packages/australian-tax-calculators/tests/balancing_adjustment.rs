// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The balancing adjustment for a depreciating asset used partly for a
//! non-taxable purpose (ITAA 1997 Subdivision 40-D, compilation 268) against
//! the Guide to depreciating assets 2026's examples and the example in
//! s 118-10(1), all read on 18 November 2026, with hand-worked cases for the
//! car rules and the worksheet's refusals
//! (docs/calculation-evidence.md#balancing-adjustment). The examples state
//! only some facts (an adjustable value, not each year's decline); the facts
//! they leave out are fabricated so that they give the stated figures.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(cost: &str, tv: &str, asset: &str, years: Value, year: &str) -> Value {
    json!({
        "cost": cost, "termination_value": tv, "asset": asset, "years": years,
        "year": year, "scope_confirmed": true,
    })
}

fn run(input: &Value) -> Result<Value, String> {
    let result = api::evaluate("balancing-adjustment", input);
    // derive gives the same result, or the same refusal.
    let envelope = json!({"command": "balancing-adjustment", "input": input});
    let doc = api::evaluate("derive", &envelope);
    assert_eq!(
        doc.as_ref().map(|doc| doc["result"].to_string()),
        result.as_ref().map(Value::to_string),
        "derive {input}"
    );
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("balancing-adjustment", input)
        .unwrap_err()
        .code
}

/// Assessable, deductible, capital gain and capital loss.
fn outcome(result: &Value) -> [&str; 4] {
    [
        "assessable_amount",
        "deductible_amount",
        "capital_gain",
        "capital_loss",
    ]
    .map(|key| result["amounts"][key].as_str().unwrap())
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// A car's year: log book with its deduction, or cents per kilometre.
fn car_year(year: &str, decline: &str, deduction: Option<&str>, days: i64) -> Value {
    match deduction {
        Some(deduction) => json!({
            "income_year": year, "decline_in_value": decline, "method": "log_book",
            "deduction": deduction, "days_held": days,
        }),
        None => json!({
            "income_year": year, "decline_in_value": decline,
            "method": "cents_per_kilometre", "days_held": days,
        }),
    }
}

fn car(cost: &str, tv: &str, years: Value, year: &str, car_limit: Value) -> Value {
    let mut facts = input(cost, tv, "car", years, year);
    facts["car_limit"] = car_limit;
    facts
}

/// Guide to depreciating assets 2026, 'Example: working out an assessable
/// balancing adjustment amount' and the deductible one after it: Bridget's
/// cabinet, used wholly for a taxable purpose, has an adjustable value of
/// $1,200 when sold for $1,300 ($100 assessable) or $1,000 ($200
/// deductible). The cost of $1,500 and the 2 years' declines are fabricated.
#[test]
fn bridget_wholly_taxable_use_is_not_reduced() {
    let years = json!([
        {"income_year": "2024-25", "decline_in_value": "150.00", "deduction": "150.00"},
        {"income_year": "2025-26", "decline_in_value": "150.00", "deduction": "150.00"},
    ]);
    let r = run(&input(
        "1500.00",
        "1300.00",
        "other",
        years.clone(),
        "2025-26",
    ))
    .unwrap();
    assert_eq!(amount(&r, "adjustable_value"), "1200.00");
    assert_eq!(amount(&r, "reduction_for_non_taxable_use"), "0.00");
    assert_eq!(outcome(&r), ["100.00", "0.00", "0.00", "0.00"]);
    let r = run(&input("1500.00", "1000.00", "other", years, "2025-26")).unwrap();
    assert_eq!(amount(&r, "balancing_adjustment_amount"), "200.00");
    assert_eq!(outcome(&r), ["0.00", "200.00", "0.00", "0.00"]);
    assert_eq!(r["rates"]["provision"], "ITAA 1997 s 40-285 and s 40-290");
}

/// Guide to depreciating assets 2026, 'Example: depreciating asset used
/// partly for a taxable purpose - balancing adjustment and capital loss':
/// Andrew's $1,000 computer, used 60% for a taxable purpose, declined $300
/// and sells for $600. He deducts $60 (60% of the $100 balancing adjustment
/// amount) and makes a capital loss of $160 (40% of the $400 by which cost
/// exceeds termination value). The sale is early in 2026-27, whose row
/// declines nil.
#[test]
fn andrew_computer_deduction_and_k7_capital_loss() {
    let years = json!([
        {"income_year": "2025-26", "decline_in_value": "300.00", "deduction": "180.00"},
        {"income_year": "2026-27", "decline_in_value": "0.00", "deduction": "0.00"},
    ]);
    let r = run(&input("1000.00", "600.00", "other", years, "2026-27")).unwrap();
    assert_eq!(amount(&r, "adjustable_value"), "700.00");
    assert_eq!(amount(&r, "sum_of_reductions"), "120.00");
    assert_eq!(amount(&r, "balancing_adjustment_amount"), "100.00");
    assert_eq!(amount(&r, "reduction_for_non_taxable_use"), "40.00");
    assert_eq!(outcome(&r), ["0.00", "60.00", "0.00", "160.00"]);
}

/// Guide to depreciating assets 2026, 'Example: using the logbook method':
/// Louise's car, bought on 1 July 2024 and sold for $24,500 on 30 June 2026,
/// has an adjustable value of $18,200 and 40% business use in both years, so
/// $2,520 of the $6,300 is assessable. Its cost of $30,000 and each year's
/// decline are fabricated; a car's capital gain or loss is disregarded.
#[test]
fn louise_log_book_car() {
    let years = json!([
        car_year("2024-25", "7500.00", Some("3000.00"), 365),
        car_year("2025-26", "4300.00", Some("1720.00"), 365),
    ]);
    let r = run(&car("30000.00", "24500.00", years, "2025-26", Value::Null)).unwrap();
    assert_eq!(amount(&r, "adjustable_value"), "18200.00");
    assert_eq!(amount(&r, "balancing_adjustment_amount"), "6300.00");
    assert_eq!(outcome(&r), ["2520.00", "0.00", "0.00", "0.00"]);
}

/// ITAA 1997 s 118-10(1), its example: Gayle's $450 print declines $31 and
/// she deducts $6; sold for $700, $54 is assessable under s 40-285 as reduced
/// by s 40-290, and her capital gain of $202 is disregarded as a collectable
/// costing $500 or less. The Act rounds to dollars; worked exactly, the
/// amounts are $54.39 and $201.61. This worksheet does not apply s 118-10, so
/// it reports the gain. The example's 2001-02 is moved to 2025-26.
#[test]
fn gayle_print_under_section_118_10() {
    let years =
        json!([{"income_year": "2025-26", "decline_in_value": "31.00", "deduction": "6.00"}]);
    let r = run(&input("450.00", "700.00", "other", years, "2025-26")).unwrap();
    assert_eq!(amount(&r, "adjustable_value"), "419.00");
    assert_eq!(amount(&r, "balancing_adjustment_amount"), "281.00");
    assert_eq!(amount(&r, "reduction_for_non_taxable_use"), "226.61");
    assert_eq!(outcome(&r), ["54.39", "0.00", "201.61", "0.00"]);
}

/// Guide to depreciating assets 2026: for an asset used wholly for a
/// non-taxable purpose the balancing adjustment amount is reduced to nil and
/// the difference between termination value and cost is the capital gain or
/// loss. Worked by hand: $2,000 cost, $500 declined, sold for $1,800.
#[test]
fn wholly_private_use_leaves_only_the_k7_amount() {
    let years =
        json!([{"income_year": "2025-26", "decline_in_value": "500.00", "deduction": "0.00"}]);
    let r = run(&input("2000.00", "1800.00", "other", years, "2025-26")).unwrap();
    assert_eq!(amount(&r, "balancing_adjustment_amount"), "300.00");
    assert_eq!(amount(&r, "reduction_for_non_taxable_use"), "300.00");
    assert_eq!(outcome(&r), ["0.00", "0.00", "0.00", "200.00"]);
    // A motor cycle's is disregarded (s 118-5).
    let years =
        json!([{"income_year": "2025-26", "decline_in_value": "500.00", "deduction": "0.00"}]);
    let r = run(&input(
        "2000.00",
        "1800.00",
        "motor_cycle_or_similar",
        years,
        "2025-26",
    ))
    .unwrap();
    assert_eq!(outcome(&r), ["0.00", "0.00", "0.00", "0.00"]);
}

/// ITAA 1997 s 40-370, worked by hand: a $30,000 car claimed by the cents per
/// kilometre method in 2024-25 (notional decline $7,500) and the log book
/// method at 60% in 2025-26 ($5,625 decline) and, for 200 days, 2026-27
/// ($2,000), sold for $18,000. Step 1: $18,000 - $14,875 = $3,125. Step 2:
/// the sum of reductions is $7,500 x 80% + $2,250 + $800 = $9,050, so
/// $3,125 x 9,050 / 15,125 = $1,869.83 comes off. Steps 3 and 4: x 565 days
/// deducted / 930 days held = $762.5467, so $762.55 is assessable (step 5).
#[test]
fn switched_car_methods_under_section_40_370() {
    let years = json!([
        car_year("2024-25", "7500.00", None, 365),
        car_year("2025-26", "5625.00", Some("3375.00"), 365),
        car_year("2026-27", "2000.00", Some("1200.00"), 200),
    ]);
    let r = run(&car("30000.00", "18000.00", years, "2026-27", Value::Null)).unwrap();
    assert_eq!(amount(&r, "sum_of_reductions"), "9050.00");
    assert_eq!(amount(&r, "reduction_for_non_taxable_use"), "1869.83");
    assert_eq!(outcome(&r), ["762.55", "0.00", "0.00", "0.00"]);
    assert_eq!(r["rates"]["provision"], "ITAA 1997 s 40-370");
    assert_eq!(r["rates"]["days_deducted"], "565");
    assert_eq!(r["rates"]["days_held"], "930");
    // A log book year with nothing deducted adds no days to step 3: with
    // 2026-27's deduction nil, the sum of reductions is $6,000 + $2,250 +
    // $2,000 = $10,250, so $3,125 x 10,250 / 15,125 = $2,117.77 comes off, and
    // x 365 days deducted / 930 days held = $395.3113, so $395.31 is
    // assessable.
    let years = json!([
        car_year("2024-25", "7500.00", None, 365),
        car_year("2025-26", "5625.00", Some("3375.00"), 365),
        car_year("2026-27", "2000.00", Some("0.00"), 200),
    ]);
    let r = run(&car("30000.00", "18000.00", years, "2026-27", Value::Null)).unwrap();
    assert_eq!(amount(&r, "reduction_for_non_taxable_use"), "2117.77");
    assert_eq!(outcome(&r), ["395.31", "0.00", "0.00", "0.00"]);
    assert_eq!(r["rates"]["days_deducted"], "365");
    assert_eq!(r["rates"]["days_held"], "930");
}

/// ITAA 1997 s 40-370(1) Note 2: a car claimed only by the cents per
/// kilometre method has nothing assessable or deductible.
#[test]
fn cents_per_kilometre_only_has_no_balancing_adjustment() {
    let years = json!([
        car_year("2024-25", "7500.00", None, 365),
        car_year("2025-26", "5625.00", None, 365),
    ]);
    let r = run(&car("30000.00", "25000.00", years, "2025-26", Value::Null)).unwrap();
    assert_eq!(outcome(&r), ["0.00", "0.00", "0.00", "0.00"]);
    assert_eq!(r["rates"]["provision"], "ITAA 1997 s 40-370(1) Note 2");
}

/// ITAA 1997 s 40-325, worked by hand: a car costing $80,000 after
/// Subdivision 27-B, first used in 2025-26 (car limit $69,674, Guide to
/// depreciating assets 2026), with no second element, sold for $50,000:
/// $50,000 x 69,674 / 80,000 = $43,546.25 is its termination value, and
/// against its $59,674 adjustable value (100% business use) $16,127.75 is
/// deductible.
#[test]
fn car_limit_scales_the_termination_value() {
    let years = json!([car_year("2025-26", "10000.00", Some("10000.00"), 365)]);
    let limit = json!({"limit": "69674.00", "second_element_cost": "0.00", "cost_ignoring_car_limit": "80000.00"});
    let r = run(&car(
        "69674.00",
        "50000.00",
        years.clone(),
        "2025-26",
        limit,
    ))
    .unwrap();
    assert_eq!(amount(&r, "termination_value"), "43546.25");
    assert_eq!(outcome(&r), ["0.00", "16127.75", "0.00", "0.00"]);
    // Facts that show s 40-230 did not reduce the cost are refused.
    let limit = json!({"limit": "69674.00", "second_element_cost": "0.00", "cost_ignoring_car_limit": "60000.00"});
    let facts = car("60000.00", "50000.00", years.clone(), "2025-26", limit);
    assert_eq!(code(&facts), "balancing.car_limit_not_reduced");
    // A cost other than the car limit plus the second element contradicts
    // s 40-230.
    let limit = json!({"limit": "69674.00", "second_element_cost": "0.00", "cost_ignoring_car_limit": "80000.00"});
    let facts = car("60000.00", "50000.00", years.clone(), "2025-26", limit);
    assert_eq!(code(&facts), "balancing.car_limit_cost_mismatch");
    // A car states car_limit, even as null; no other asset takes it.
    let mut facts = car(
        "69674.00",
        "50000.00",
        years.clone(),
        "2026-27",
        Value::Null,
    );
    facts.as_object_mut().unwrap().remove("car_limit");
    assert_eq!(code(&facts), "input.missing_field");
    let other =
        json!([{"income_year": "2025-26", "decline_in_value": "10.00", "deduction": "10.00"}]);
    let mut facts = input("100.00", "50.00", "other", other, "2026-27");
    facts["car_limit"] = Value::Null;
    assert_eq!(code(&facts), "input.field_unknown");
    let facts = car("69674.00", "50000.00", years, "2026-27", json!("69674.00"));
    assert_eq!(code(&facts), "balancing.car_limit_invalid");
}

#[test]
fn refusals_name_what_is_out_of_scope_or_wrong() {
    let row = |year: &str, decline: &str, deduction: &str| json!({"income_year": year, "decline_in_value": decline, "deduction": deduction});
    let other = |years: Value| input("1000.00", "500.00", "other", years, "2025-26");
    let cases = [
        (
            other(json!([row("2026-27", "100.00", "50.00")])),
            "balancing.year_after_event",
        ),
        (
            other(json!([row("2000-01", "100.00", "50.00")])),
            "balancing.year_before_scope",
        ),
        (
            other(json!([row("2025-26", "100.00", "150.00")])),
            "balancing.deduction_above_decline",
        ),
        (
            other(json!([row("2025-26", "1000.01", "0.00")])),
            "balancing.decline_above_cost",
        ),
        (
            other(json!([row("2025-26", "0.00", "0.00")])),
            "balancing.no_decline",
        ),
        (
            other(json!([
                row("2024-25", "100.00", "50.00"),
                row("2024-25", "100.00", "50.00")
            ])),
            "balancing.year_repeated",
        ),
        (other(json!([])), "balancing.years_invalid"),
        // Every year from the first held to the event's: none skipped, and
        // the event's year given.
        (
            other(json!([
                row("2023-24", "100.00", "50.00"),
                row("2025-26", "100.00", "50.00")
            ])),
            "balancing.years_incomplete",
        ),
        (
            other(json!([row("2024-25", "100.00", "50.00")])),
            "balancing.years_incomplete",
        ),
        (
            other(json!([{"income_year": "2025-26", "decline_in_value": "100.00"}])),
            "balancing.fact_missing",
        ),
        (
            other(json!([row("2025-26", "1.005", "0.00")])),
            "input.amount_invalid",
        ),
        (
            other(json!([row("2025", "100.00", "50.00")])),
            "balancing.fact_invalid",
        ),
        (
            other(
                json!([{"income_year": "2025-26", "decline_in_value": "100.00", "deduction": "50.00", "days_held": 365}]),
            ),
            "input.field_unknown",
        ),
        (
            input("1000.00", "500.00", "boat", json!([]), "2025-26"),
            "balancing.asset_invalid",
        ),
        (
            input(
                "1000.00",
                "500.00",
                "other",
                json!([row("2025-26", "100.00", "50.00")]),
                "2027-28",
            ),
            "period.unsupported",
        ),
    ];
    for (facts, expected) in cases {
        assert_eq!(code(&facts), expected, "{facts}");
    }
    // A car year before 2015-16, when the one-third and 12% methods existed.
    let years = json!([car_year("2014-15", "100.00", Some("50.00"), 365)]);
    let facts = car("1000.00", "500.00", years, "2025-26", Value::Null);
    assert_eq!(code(&facts), "balancing.year_before_scope");
    // A car row's method is required, and a cents per kilometre row takes no deduction.
    let mut years = json!([car_year("2024-25", "100.00", Some("50.00"), 365)]);
    years[0].as_object_mut().unwrap().remove("method");
    assert_eq!(
        code(&car("1000.00", "500.00", years, "2025-26", Value::Null)),
        "balancing.fact_missing"
    );
    let mut years = json!([car_year("2024-25", "100.00", None, 365)]);
    years[0]["deduction"] = json!("10.00");
    assert_eq!(
        code(&car("1000.00", "500.00", years, "2025-26", Value::Null)),
        "input.field_unknown"
    );
    let years = json!([car_year("2024-25", "100.00", None, 367)]);
    assert_eq!(
        code(&car("1000.00", "500.00", years, "2025-26", Value::Null)),
        "balancing.fact_invalid"
    );
    // A car is held at most the days in each income year: 365 in 2024-25,
    // 366 in 2023-24, which includes 29 February 2024.
    let years = json!([car_year("2024-25", "100.00", None, 366)]);
    assert_eq!(
        code(&car("1000.00", "500.00", years, "2025-26", Value::Null)),
        "balancing.fact_invalid"
    );
    let years = json!([
        car_year("2023-24", "100.00", Some("50.00"), 366),
        car_year("2024-25", "100.00", None, 365),
    ]);
    let r = run(&car("1000.00", "500.00", years, "2024-25", Value::Null)).unwrap();
    assert_eq!(r["rates"]["days_held"], "731");
    // 2027-28 says why: the CGT changes from 1 July 2027.
    let refused = api::evaluate(
        "balancing-adjustment",
        &input(
            "1000.00",
            "500.00",
            "other",
            json!([row("2025-26", "100.00", "50.00")]),
            "2027-28",
        ),
    )
    .unwrap_err();
    assert!(refused.contains("Act No. 49 of 2026"), "{refused}");
}
