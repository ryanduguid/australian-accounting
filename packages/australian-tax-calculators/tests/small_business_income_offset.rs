// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The small business income tax offset (ITAA 1997 Subdivision 328-F,
//! compilations 210, 223 and 268, read on 22 November 2026) against the
//! sole trader vectors of ryanduguid.github.io (`_data/individual_tax_cases.json`,
//! checked 19 November 2026), the ATO's rate table and hand-worked cases at
//! each edge (docs/calculation-evidence.md#small-business-income-offset).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("small-business-income-offset", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("small-business-income-offset", input)
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

/// A sole trader on $22,000 of turnover each year, with `own` net small
/// business income, `taxable` income and the basic income tax liability.
fn sole(year: &str, taxable: &str, own: &str, liability: &str, first: &str) -> Value {
    json!({
        "year": year, "scope_confirmed": true, "taxable_income": taxable,
        "basic_income_tax_liability": liability, "offsets_applied_first": first,
        "net_small_business_income": own, "shares": [], "prescribed_person": false,
        "business": "carried_on", "previous_year_turnover": "21000.00",
        "year_before_turnover": "18000.00", "likely_turnover": "22000.00",
        "year_turnover": "22000.00",
    })
}

/// Only partnership or trust amounts: none of the business facts.
fn partner(shares: Value) -> Value {
    json!({
        "year": "2025-26", "scope_confirmed": true, "taxable_income": "90000",
        "basic_income_tax_liability": "18788.00", "offsets_applied_first": "0.00",
        "net_small_business_income": "0.00", "shares": shares, "prescribed_person": false,
    })
}

fn amounts(result: &Value) -> [&str; 4] {
    let a = &result["amounts"];
    [
        a["total_net_small_business_income"].as_str().unwrap(),
        a["offset"].as_str().unwrap(),
        a["offset_applied"].as_str().unwrap(),
        a["offset_unused"].as_str().unwrap(),
    ]
}

/// The site's 6 sole trader vectors with the offset: each gives the same
/// `smallBusinessOffset`, the basic income tax liability being the site's
/// `tax` and the offsets applied first its low income and low and middle
/// income tax offsets.
#[test]
fn the_site_sole_trader_vectors_agree() {
    let cases = [
        // $20,000 of profit on $60,000 of wages, 2026-27: 20/79 of 16%.
        (
            "2026-27",
            "79000",
            "20000.00",
            "14220.00",
            "0.00",
            ["20000.00", "576.00", "576.00", "0.00"],
        ),
        // No job: 16% of $13,020 is $2,083.20, so the $1,000 cap.
        (
            "2026-27",
            "75000",
            "75000.00",
            "13020.00",
            "0.00",
            ["75000.00", "1000.00", "1000.00", "0.00"],
        ),
        // 2024-25: $1,177.57 before the cap.
        (
            "2024-25",
            "160584",
            "29000.00",
            "40754.08",
            "0.00",
            ["29000.00", "1000.00", "1000.00", "0.00"],
        ),
        (
            "2025-26",
            "120000",
            "120000.00",
            "26788.00",
            "0.00",
            ["120000.00", "1000.00", "1000.00", "0.00"],
        ),
        // 2023-24: the profit is above taxable income, so the share is 1.
        (
            "2023-24",
            "32000",
            "35000.00",
            "2622.00",
            "700.00",
            ["32000.00", "419.52", "419.52", "0.00"],
        ),
        // 2021-22: the low and middle income offsets leave no tax for it.
        (
            "2021-22",
            "25000",
            "25000.00",
            "1292.00",
            "1292.00",
            ["25000.00", "206.72", "0.00", "206.72"],
        ),
    ];
    for (year, taxable, own, liability, first, expected) in cases {
        let result = run(&sole(year, taxable, own, liability, first));
        assert_eq!(amounts(&result), expected, "{year} {taxable}");
        assert_eq!(result["rates"]["offset_rate"], "0.16");
        assert_eq!(result["small_business_entity_rule"], "s 328-110(1)(b)(i)");
    }
    let capped = run(&sole("2024-25", "160584", "29000.00", "40754.08", "0.00"));
    assert_eq!(capped["amounts"]["offset_before_cap"], "1177.57");
}

/// The site's basic income tax liabilities are the resident scale's tax for
/// the years `resident-tax` covers.
#[test]
fn the_vectors_liabilities_are_the_resident_scale() {
    for (year, taxable, tax) in [
        ("2026-27", "79000", "14220.00"),
        ("2026-27", "75000", "13020.00"),
        ("2024-25", "160584", "40754.08"),
        ("2025-26", "120000", "26788.00"),
    ] {
        let result = api::evaluate(
            "resident-tax",
            &json!({"taxable_income": taxable, "year": year, "scope_confirmed": true}),
        )
        .unwrap();
        assert_eq!(
            result["amounts"]["basic_income_tax"], tax,
            "{year} {taxable}"
        );
    }
}

/// The ATO's table: 13% for 2020-21, 16% from 2021-22, each year to 2026-27.
#[test]
fn each_year_takes_its_rate() {
    let result = run(&sole("2020-21", "50000", "50000.00", "6717.00", "0.00"));
    assert_eq!(result["rates"]["offset_rate"], "0.13");
    assert_eq!(result["amounts"]["offset"], "873.21");
    for year in [
        "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
    ] {
        let result = run(&sole(year, "50000", "50000.00", "6717.00", "0.00"));
        assert_eq!(result["rates"]["offset_rate"], "0.16", "{year}");
        assert_eq!(result["amounts"]["offset"], "1000.00", "{year}");
    }
    for year in ["2019-20", "2027-28"] {
        assert_eq!(
            code(&sole(year, "50000", "50000.00", "6717.00", "0.00")),
            "period.unsupported"
        );
    }
}

/// $1,000 exactly is the offset; a cent over is capped to it.
#[test]
fn the_cap_holds_at_1000() {
    // 16% of $6,250 is $1,000.
    let at = run(&sole("2025-26", "50000", "50000.00", "6250.00", "0.00"));
    assert_eq!(at["amounts"]["offset_before_cap"], "1000.00");
    assert_eq!(at["amounts"]["offset"], "1000.00");
    let over = run(&sole("2025-26", "50000", "50000.00", "6250.07", "0.00"));
    assert_eq!(over["amounts"]["offset_before_cap"], "1000.01");
    assert_eq!(over["amounts"]["offset"], "1000.00");
}

/// The turnover test at $5 million (s 328-357) by each rule of s 328-110.
#[test]
fn the_turnover_test_is_at_5_million() {
    let base = sole("2025-26", "50000", "50000.00", "6717.00", "0.00");
    let turnovers = |previous: Value, before: Value, likely: &str, at_end: &str| {
        run(&with(
            base.clone(),
            json!({"previous_year_turnover": previous, "year_before_turnover": before,
                "likely_turnover": likely, "year_turnover": at_end}),
        ))
    };
    let previous = turnovers(
        json!("4999999.99"),
        json!("6000000.00"),
        "6000000.00",
        "6000000.00",
    );
    assert_eq!(previous["small_business_entity_rule"], "s 328-110(1)(b)(i)");
    let likely = turnovers(
        json!("5000000.00"),
        json!("4000000.00"),
        "4999999.99",
        "5000000.00",
    );
    assert_eq!(likely["small_business_entity_rule"], "s 328-110(1)(b)(ii)");
    // Two years at $5 million bar the likely turnover (s 328-110(3)).
    let barred = turnovers(
        json!("5000000.00"),
        json!("5000000.00"),
        "4000000.00",
        "5000000.00",
    );
    assert_eq!(barred["small_business_entity_rule"], Value::Null);
    assert_eq!(barred["amounts"]["offset"], "0.00");
    assert_eq!(barred["conditions"][0]["met"], false);
    let at_end = turnovers(
        json!("5000000.00"),
        json!("5000000.00"),
        "4000000.00",
        "4999999.99",
    );
    assert_eq!(at_end["small_business_entity_rule"], "s 328-110(4)");
    // A new business: no previous year.
    let new = turnovers(Value::Null, Value::Null, "300000.00", "300000.00");
    assert_eq!(new["small_business_entity_rule"], "s 328-110(1)(b)(ii)");
    let over = turnovers(json!("5000000.00"), Value::Null, "5000000.00", "5000000.00");
    assert_eq!(over["amounts"]["net_small_business_income_counted"], "0.00");
}

/// A business being wound up counts only if it was a small business entity
/// in the year it stopped (s 328-110(5)).
#[test]
fn a_business_being_wound_up() {
    let base = with(
        sole("2025-26", "50000", "50000.00", "6717.00", "0.00"),
        json!({"business": "winding_up", "previous_year_turnover": "100000.00"}),
    );
    assert_eq!(code(&base), "input.missing_field");
    let was = run(&with(
        base.clone(),
        json!({"small_business_when_stopped": true}),
    ));
    assert_eq!(was["amounts"]["offset"], "1000.00");
    let was_not = run(&with(base, json!({"small_business_when_stopped": false})));
    assert_eq!(was_not["amounts"]["offset"], "0.00");
    let carried = with(
        sole("2025-26", "50000", "50000.00", "6717.00", "0.00"),
        json!({"small_business_when_stopped": true}),
    );
    assert_eq!(code(&carried), "input.field_unknown");
    let other = with(
        sole("2025-26", "50000", "50000.00", "6717.00", "0.00"),
        json!({"business": "not_carried_on"}),
    );
    assert_eq!(
        code(&other),
        "small_business_income_offset.business_invalid"
    );
}

/// Each partnership or trust amount less its deductions, not below zero:
/// one share's excess deductions do not reduce another's (s 328-360(1)(b);
/// the ATO's 'treat your share from that partnership as zero').
#[test]
fn each_share_is_net_of_its_deductions_and_not_below_zero() {
    let result = run(&partner(json!([
        {"amount": "10000.00", "deductions": "12000.00"},
        {"amount": "5000.00", "deductions": "1000.00"},
    ])));
    assert_eq!(result["amounts"]["shares_net"], "4000.00");
    assert_eq!(
        result["amounts"]["total_net_small_business_income"],
        "4000.00"
    );
    // 16% x 18,788 x 4,000 / 90,000 = 133.60.
    assert_eq!(result["amounts"]["offset"], "133.60");
    assert_eq!(result["small_business_entity_rule"], Value::Null);
    assert_eq!(result["conditions"], json!([]));
    // No business facts are taken without net small business income of your own.
    let extra = with(partner(json!([])), json!({"business": "carried_on"}));
    assert_eq!(code(&extra), "input.field_unknown");
    let none = run(&partner(json!([])));
    assert_eq!(none["amounts"]["offset"], "0.00");
    assert_eq!(none["conditions"], json!([]));
}

/// Own and partnership income together, capped at taxable income.
#[test]
fn own_and_share_income_add_up_to_taxable_income() {
    let result = run(&with(
        sole("2025-26", "30000", "20000.00", "2242.00", "700.00"),
        json!({"shares": [{"amount": "15000.00", "deductions": "0.00"}]}),
    ));
    assert_eq!(
        result["amounts"]["total_net_small_business_income"],
        "30000.00"
    );
    // 16% of 2,242 = 358.72; 1,542 is left after the low income tax offset.
    assert_eq!(result["amounts"]["offset"], "358.72");
    assert_eq!(result["amounts"]["offset_applied"], "358.72");
}

/// Offsets applied first that leave less than this offset: the liability
/// left absorbs part of it and the rest is lost (s 63-10(1) item 20).
#[test]
fn part_of_the_offset_is_absorbed() {
    let result = run(&sole("2021-22", "25000", "25000.00", "1292.00", "1200.00"));
    assert_eq!(amounts(&result), ["25000.00", "206.72", "92.00", "114.72"]);
}

#[test]
fn nil_taxable_income_gives_no_offset() {
    let result = run(&sole("2025-26", "0", "1000.00", "0.00", "0.00"));
    assert_eq!(result["amounts"]["offset"], "0.00");
    assert_eq!(result["amounts"]["total_net_small_business_income"], "0.00");
}

#[test]
fn refusals() {
    let base = sole("2025-26", "50000", "50000.00", "6717.00", "0.00");
    assert_eq!(
        code(&with(base.clone(), json!({"prescribed_person": true}))),
        "small_business_income_offset.prescribed_person"
    );
    assert_eq!(
        code(&with(base.clone(), json!({"taxable_income": "50000.50"}))),
        "input.amount_not_whole_dollars"
    );
    assert_eq!(
        code(&with(base.clone(), json!({"likely_turnover": "remove"}))),
        "input.missing_field"
    );
    assert_eq!(
        code(&with(base.clone(), json!({"shares": {"amount": "1.00"}}))),
        "small_business_income_offset.shares_invalid"
    );
    assert_eq!(
        code(&with(base.clone(), json!({"shares": [{"amount": "1.00"}]}))),
        "input.missing_field"
    );
    assert_eq!(
        code(&with(
            base.clone(),
            json!({"shares": [{"amount": "1.00", "deductions": "0.00", "other": "1"}]})
        )),
        "input.field_unknown"
    );
    let many: Vec<Value> = (0..51)
        .map(|_| json!({"amount": "1.00", "deductions": "0.00"}))
        .collect();
    assert_eq!(
        code(&with(base.clone(), json!({"shares": many}))),
        "small_business_income_offset.shares_invalid"
    );
    assert_eq!(
        code(&with(base.clone(), json!({"scope_confirmed": false}))),
        "scope.not_confirmed"
    );
    assert_eq!(
        code(&with(base, json!({"not_an_input": 1}))),
        "input.field_unknown"
    );
}

/// A share that is not an object is refused, naming it.
#[test]
fn a_share_must_be_an_object() {
    let error = api::evaluate_coded(
        "small-business-income-offset",
        &partner(json!([{"amount": "100.00", "deductions": "0.00"}, "200.00"])),
    )
    .unwrap_err();
    assert_eq!(error.code, "small_business_income_offset.shares_invalid");
    assert_eq!(
        error.message,
        "share 2 must be an object with amount and deductions."
    );
}
