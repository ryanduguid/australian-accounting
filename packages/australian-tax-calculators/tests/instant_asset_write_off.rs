// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The instant asset write-off: ITAA 1997 ss 328-180 and 40-82, IT(TP)A 1997
//! ss 328-180 and 328-181, Subdivision 40-BB and Act No. 71 of 2026 Sch 2,
//! all read on 23 November 2026, against the ATO's 'Instant asset write-off
//! for eligible businesses' and 'TFE for small business entities using
//! simplified depreciation' examples and each date boundary
//! (docs/calculation-evidence.md#instant-asset-write-off).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const IAWO: &str = "instant-asset-write-off";

fn input(overrides: Value) -> Value {
    let mut input = json!({
        "entity": "small_business_simplified", "acquired": "2025-11-18",
        "first_used": "2025-11-18", "cost": "6800.00", "taxable_use": "0.8",
        "year": "2025-26", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

/// The plain result, checked against `derive` byte for byte.
fn run(overrides: Value) -> Value {
    let input = input(overrides);
    let result = api::evaluate(IAWO, &input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": IAWO, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded(IAWO, &input(overrides))
        .unwrap_err()
        .code
}

/// The deduction, the cost limit and whether the asset was written off.
fn outcome(result: &Value) -> (&str, &str, &str) {
    (
        result["amounts"]["deduction"].as_str().unwrap(),
        result["rates"]["cost_limit"].as_str().unwrap(),
        result["rates"]["written_off"].as_str().unwrap(),
    )
}

/// The ATO's examples for 2025-26: Fiona's $6,800 computer used 80% for
/// business ($5,440) and $700 printer; Jack's $4,500 MIG welder.
#[test]
fn the_atos_2025_26_examples() {
    let computer = run(json!({}));
    assert_eq!(outcome(&computer), ("5440.00", "20000", "true"));
    assert_eq!(computer["amounts"]["taxable_purpose_cost"], "5440.00");
    let printer = run(json!({"cost": "700.00", "taxable_use": "1"}));
    assert_eq!(outcome(&printer).0, "700.00");
    let welder = run(json!({"acquired": "2026-03-12", "first_used": "2026-03-12",
        "cost": "4500.00", "taxable_use": "1"}));
    assert_eq!(outcome(&welder).0, "4500.00");
    assert!(
        computer["rates"]["cost_limit_rule"]
            .as_str()
            .unwrap()
            .contains("328-180(4)(d)")
    );
}

/// Daryl's $40,000 ute, used 40% for business, is not less than $20,000:
/// its $16,000 business portion goes to the small business pool instead.
#[test]
fn the_atos_ute_over_the_limit() {
    let ute = run(json!({"acquired": "2025-07-28", "first_used": "2025-07-28",
        "cost": "40000.00", "taxable_use": "0.4"}));
    assert_eq!(outcome(&ute), ("0.00", "20000", "false"));
    assert_eq!(ute["amounts"]["taxable_purpose_cost"], "16000.00");
    assert!(ute["warnings"].to_string().contains("s 328-185"));
}

/// Temporary full expensing for a small business entity using simplified
/// depreciation (IT(TP)A s 328-181(2)): X Co's $35,000 van held and used on
/// 30 November 2020, and Jack's $220,000 excavator first used on
/// 20 January 2023, are each written off whatever their cost.
#[test]
fn the_atos_temporary_full_expensing_examples() {
    let van = run(json!({"year": "2020-21", "acquired": "2020-11-30",
        "first_used": "2020-11-30", "cost": "35000.00", "taxable_use": "1"}));
    assert_eq!(outcome(&van), ("35000.00", "none", "true"));
    assert!(
        van["rates"]["cost_limit_rule"]
            .as_str()
            .unwrap()
            .contains("328-181(2)")
    );
    assert!(
        van["warnings"]
            .to_string()
            .contains("temporary full expensing year")
    );
    let excavator = run(json!({"year": "2022-23", "acquired": "2023-01-20",
        "first_used": "2023-01-20", "cost": "220000.00", "taxable_use": "1"}));
    assert_eq!(outcome(&excavator), ("220000.00", "none", "true"));
}

fn written_off(overrides: Value) -> bool {
    outcome(&run(overrides)).2 == "true"
}

/// Each limit is a cost that must be less than it.
#[test]
fn each_small_business_limit_by_date_of_first_use() {
    for (year, acquired, used, limit, below, at) in [
        // IT(TP)A s 328-180(4A): first used 12 March 2020 to 30 June 2021.
        (
            "2020-21",
            "2020-09-01",
            "2020-09-15",
            "150000",
            "149999.99",
            "150000.00",
        ),
        // Acquired before the 2020 budget time, used 2021-22: the $1,000 base.
        (
            "2021-22",
            "2020-09-01",
            "2021-08-01",
            "1000",
            "999.99",
            "1000.00",
        ),
        (
            "2022-23",
            "2020-10-05",
            "2023-06-30",
            "1000",
            "999.99",
            "1000.00",
        ),
        // IT(TP)A s 328-180(4)(d): first used 1 July 2023 to 30 June 2026.
        (
            "2023-24",
            "2015-05-13",
            "2023-07-01",
            "20000",
            "19999.99",
            "20000.00",
        ),
        (
            "2024-25",
            "2024-07-01",
            "2024-07-01",
            "20000",
            "19999.99",
            "20000.00",
        ),
        (
            "2025-26",
            "2025-01-01",
            "2026-06-30",
            "20000",
            "19999.99",
            "20000.00",
        ),
        // ITAA 1997 s 328-180(1)(b) from 1 July 2026, whenever acquired.
        (
            "2026-27",
            "2010-01-01",
            "2026-07-01",
            "20000",
            "19999.99",
            "20000.00",
        ),
    ] {
        let facts = json!({"year": year, "acquired": acquired, "first_used": used,
            "taxable_use": "1"});
        let mut facts_below = facts.clone();
        facts_below["cost"] = json!(below);
        let r = run(facts_below);
        assert_eq!(outcome(&r), (below, limit, "true"), "{year}");
        let mut facts_at = facts;
        facts_at["cost"] = json!(at);
        assert!(!written_off(facts_at), "{year}");
    }
}

/// The temporary increases need the asset first acquired at or after
/// 7.30 pm on 12 May 2015; earlier, the $1,000 base applies.
#[test]
fn the_2015_budget_time() {
    for (year, used) in [("2020-21", "2020-07-01"), ("2023-24", "2023-08-01")] {
        let early = run(
            json!({"year": year, "acquired": "2015-05-11", "first_used": used,
            "cost": "5000.00"}),
        );
        assert_eq!(outcome(&early), ("0.00", "1000", "false"), "{year}");
        assert!(written_off(json!({"year": year, "acquired": "2015-05-13",
            "first_used": used, "cost": "5000.00"})));
        let budget_day = json!({"year": year, "acquired": "2015-05-12", "first_used": used});
        assert_eq!(code(budget_day), "instant_asset_write_off.budget_time_day");
    }
    // Where the 2015 budget time cannot change the limit, its day is accepted:
    // $1,000 for first use in 2021-22 and $20,000 from 1 July 2026 either way.
    for (year, used, limit) in [
        ("2021-22", "2021-08-01", "1000"),
        ("2026-27", "2026-08-01", "20000"),
    ] {
        let r = run(
            json!({"year": year, "acquired": "2015-05-12", "first_used": used,
            "cost": "500.00", "taxable_use": "1"}),
        );
        assert_eq!(outcome(&r), ("500.00", limit, "true"), "{year}");
    }
}

/// IT(TP)A s 328-181(2): held from 7.30 pm AEDT on 6 October 2020; a date on
/// that day is refused, and the next day has no limit.
#[test]
fn the_2020_budget_time() {
    let facts = json!({"year": "2020-21", "acquired": "2020-10-06",
        "first_used": "2020-10-07", "cost": "200000.00", "taxable_use": "1"});
    assert_eq!(code(facts), "instant_asset_write_off.budget_time_day");
    let next = run(json!({"year": "2020-21", "acquired": "2020-10-07",
        "first_used": "2020-10-07", "cost": "200000.00", "taxable_use": "1"}));
    assert_eq!(outcome(&next), ("200000.00", "none", "true"));
    // Held from 6 October 2020 but first used from 1 July 2023: $20,000.
    let later = run(json!({"year": "2023-24", "acquired": "2021-01-01",
        "first_used": "2023-07-01", "cost": "25000.00"}));
    assert_eq!(outcome(&later), ("0.00", "20000", "false"));
    // The 2020 budget time does not matter after 30 June 2023.
    let after = run(json!({"year": "2023-24", "acquired": "2020-10-06",
        "first_used": "2023-07-01", "cost": "5000.00", "taxable_use": "1"}));
    assert_eq!(outcome(&after).0, "5000.00");
}

/// ITAA 1997 s 40-82(2A): a medium business's asset first acquired from
/// 7.30 pm on 2 April 2019 to 31 December 2020 and first used from
/// 12 March 2020 to 30 June 2021, costing less than $150,000.
#[test]
fn a_medium_business_in_2020_21() {
    let facts = |acquired: &str, cost: &str| {
        json!({"entity": "medium_business", "year": "2020-21", "acquired": acquired,
            "first_used": "2020-08-01", "cost": cost, "taxable_use": "0.5"})
    };
    let r = run(facts("2020-06-01", "149999.99"));
    // $149,999.99 x 0.5 = $74,999.995, half up to the cent.
    assert_eq!(outcome(&r), ("75000.00", "150000", "true"));
    assert!(
        r["rates"]["cost_limit_rule"]
            .as_str()
            .unwrap()
            .contains("40-82(2A)")
    );
    assert!(!written_off(facts("2020-06-01", "150000.00")));
    assert!(written_off(facts("2019-04-03", "1000.00")));
    let early = run(facts("2019-04-01", "1000.00"));
    assert_eq!(outcome(&early), ("0.00", "not_applicable", "false"));
    assert_eq!(
        code(facts("2019-04-02", "1000.00")),
        "instant_asset_write_off.budget_time_day"
    );
    let full_expensing = json!({"entity": "medium_business", "year": "2020-21",
        "acquired": "2021-01-01", "first_used": "2021-02-01"});
    assert_eq!(
        code(full_expensing),
        "instant_asset_write_off.temporary_full_expensing"
    );
    // Acquired before the 2020 budget time, the asset can still have amounts
    // included in the second element of its cost from 7.30 pm on 6 October
    // 2020 before first use, which IT(TP)A s 40-170 fully expenses: first use
    // from that day is refused, and the day before is written off.
    let used = |first_used: &str| {
        json!({"entity": "medium_business", "year": "2020-21", "acquired": "2020-09-01",
            "first_used": first_used, "cost": "100000.00", "taxable_use": "1"})
    };
    for first_used in ["2020-10-06", "2020-11-01"] {
        assert_eq!(
            code(used(first_used)),
            "instant_asset_write_off.temporary_full_expensing",
            "{first_used}"
        );
    }
    assert_eq!(
        outcome(&run(used("2020-10-05"))),
        ("100000.00", "150000", "true")
    );
}

/// From 1 July 2021 s 40-82(5) leaves a medium business no write-off, and an
/// asset temporary full expensing could take is refused for any entity but a
/// small business using simplified depreciation.
#[test]
fn other_entities_outside_2020_21() {
    for (entity, year, acquired, used) in [
        ("medium_business", "2023-24", "2021-01-01", "2023-07-01"),
        ("other", "2020-21", "2020-01-01", "2020-07-01"),
        ("other", "2026-27", "2026-07-01", "2026-07-01"),
    ] {
        let r = run(json!({"entity": entity, "year": year, "acquired": acquired,
            "first_used": used, "cost": "500.00", "taxable_use": "1"}));
        assert_eq!(
            outcome(&r),
            ("0.00", "not_applicable", "false"),
            "{entity} {year}"
        );
        assert!(
            r["warnings"].to_string().contains("Division 40"),
            "{entity}"
        );
    }
    for (entity, year, acquired, used) in [
        ("other", "2022-23", "2021-01-01", "2022-07-01"),
        ("medium_business", "2021-22", "2021-07-01", "2021-07-01"),
        ("other", "2020-21", "2020-10-07", "2021-06-30"),
        // Acquired before or on the budget day: s 40-170 may still apply.
        ("medium_business", "2021-22", "2020-09-01", "2021-07-01"),
        ("other", "2021-22", "2020-10-06", "2021-07-01"),
    ] {
        let facts = json!({"entity": entity, "year": year, "acquired": acquired,
            "first_used": used});
        assert_eq!(
            code(facts),
            "instant_asset_write_off.temporary_full_expensing",
            "{entity} {year}"
        );
    }
}

/// A small business's asset that is not written off in 2020-21 may take
/// backing business investment's 57.5% in the pool.
#[test]
fn a_2020_21_pool_asset_flags_backing_business_investment() {
    let r = run(json!({"year": "2020-21", "acquired": "2020-09-01",
        "first_used": "2020-09-15", "cost": "200000.00"}));
    assert_eq!(outcome(&r).2, "false");
    assert!(r["warnings"].to_string().contains("57.5%"));
    let later = run(json!({"year": "2021-22", "acquired": "2020-09-01",
        "first_used": "2021-08-01", "cost": "2000.00"}));
    assert!(!later["warnings"].to_string().contains("57.5%"));
}

#[test]
fn the_dates_must_fit_the_year() {
    assert_eq!(
        code(json!({"first_used": "2025-06-30"})),
        "instant_asset_write_off.first_use_outside_year"
    );
    assert_eq!(
        code(json!({"first_used": "2026-07-01"})),
        "instant_asset_write_off.first_use_outside_year"
    );
    assert_eq!(
        code(json!({"acquired": "2025-11-19"})),
        "instant_asset_write_off.acquired_after_first_use"
    );
    assert_eq!(
        code(json!({"acquired": "18/11/2025"})),
        "input.date_invalid"
    );
    assert_eq!(code(json!({"first_used": null})), "input.date_invalid");
}

#[test]
fn the_inputs_are_checked() {
    assert_eq!(
        code(json!({"entity": "large"})),
        "instant_asset_write_off.entity_invalid"
    );
    assert_eq!(
        code(json!({"entity": 1})),
        "instant_asset_write_off.entity_invalid"
    );
    assert_eq!(code(json!({"cost": "-1.00"})), "input.amount_invalid");
    assert_eq!(
        code(json!({"taxable_use": "1.01"})),
        "input.fraction_invalid"
    );
    let digits = format!("0.{}", "3".repeat(27));
    assert_eq!(
        code(json!({ "taxable_use": digits })),
        "instant_asset_write_off.precision_exceeded"
    );
    let most = format!("0.{}", "3".repeat(26));
    let r = run(json!({"cost": "1000000000000.00", "taxable_use": most}));
    assert_eq!(r["amounts"]["taxable_purpose_cost"], "333333333333.33");
    assert_eq!(
        code(json!({"scope_confirmed": false})),
        "scope.not_confirmed"
    );
    assert_eq!(code(json!({"days": 365})), "input.field_unknown");
    for year in ["2019-20", "2027-28"] {
        assert_eq!(code(json!({"year": year})), "period.unsupported", "{year}");
    }
}

#[test]
fn the_catalogue_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["instant_asset_write_off"];
    assert_eq!(entry["supported_periods"].as_array().unwrap().len(), 7);
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = api::evaluate(IAWO, &facts).unwrap();
    assert_eq!(r["amounts"]["deduction"], "5440.00");
}
