// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Motor vehicle duty on a passenger vehicle in each state and territory for
//! 2026-27: the Transport and Main Roads examples for Queensland, figures
//! worked by hand from each Act or determination at its band edges, and an
//! independent oracle's 936 cases (`data/motor_vehicle_duty_cases.json`:
//! every band edge, minus 1, plus 1, plus 99 and plus 100, and common values,
//! under every choice each jurisdiction asks), written from the spec, not the
//! code. Every source was read on 24 November 2026
//! (docs/calculation-evidence.md#motor-vehicle-duty). Every vehicle here is
//! fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A passenger vehicle in `place`, with the choices each jurisdiction asks
/// set to their plain values.
fn input(place: &str, value: &str) -> Value {
    let mut v = json!({
        "jurisdiction": place, "vehicle_class": "passenger", "dutiable_value": value,
        "duty_date": "2026-09-15", "year": "2026-27", "scope_confirmed": true,
    });
    let extra = match place {
        "qld" => json!({"engine": "up_to_4_cylinders"}),
        "vic" => json!({"green_car": false, "primary_producer": false}),
        "act" => json!({"emissions_category": "c", "new_vehicle": true}),
        "tas" => json!({"fleet_discount": false}),
        "nt" => json!({"electric": false}),
        _ => json!({}),
    };
    with(v.take(), extra)
}

fn with(mut v: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value.is_null() {
            v.as_object_mut().unwrap().remove(key);
        } else {
            v[key] = value.clone();
        }
    }
    v
}

fn run(input: &Value) -> Value {
    let result = api::evaluate("motor-vehicle-duty", input).unwrap();
    let envelope = json!({"command": "motor-vehicle-duty", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn duty(place: &str, value: &str, changes: Value) -> String {
    let result = run(&with(input(place, value), changes));
    result["amounts"]["duty"].as_str().unwrap().to_string()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("motor-vehicle-duty", input)
        .unwrap_err()
        .code
}

/// TMR, 'Vehicle registration duty help' (reviewed 14 August 2026): a light
/// vehicle at $35,600 and $135,600 for each Schedule 4C row.
#[test]
fn queensland_matches_the_published_examples() {
    let cases = [
        ("hybrid_or_electric", "712.00", "5424.00"),
        ("up_to_4_cylinders", "1068.00", "6780.00"),
        ("5_or_6_cylinders", "1246.00", "7458.00"),
        ("7_or_more_cylinders", "1424.00", "8136.00"),
    ];
    for (engine, low, high) in cases {
        let row = json!({"engine": engine});
        assert_eq!(duty("qld", "35600", row.clone()), low, "{engine}");
        assert_eq!(duty("qld", "135600", row), high, "{engine}");
    }
}

/// Figures worked by hand from each scale at its edges.
#[test]
fn each_scale_turns_at_its_edges() {
    let none = json!({});
    let cases = [
        ("nsw", "45000", none.clone(), "1350.00"),
        ("nsw", "45001", none.clone(), "1355.00"),
        ("nsw", "45100", none.clone(), "1355.00"),
        ("nsw", "45101", none.clone(), "1360.00"),
        // 405 lots of $200 either side of the luxury car tax threshold.
        ("vic", "80809", none.clone(), "3402.00"),
        ("vic", "80810", none.clone(), "4212.00"),
        ("vic", "150001", none.clone(), "13518.00"),
        ("vic", "150001", json!({"green_car": true}), "6308.40"),
        (
            "vic",
            "150001",
            json!({"primary_producer": true}),
            "6308.40",
        ),
        ("qld", "100000", none.clone(), "3000.00"),
        ("qld", "100001", none.clone(), "5005.00"),
        // R = 3.50000075 to 3.50; R = 4.625001875 to 4.63.
        ("wa", "25000", none.clone(), "687.50"),
        ("wa", "30000", none.clone(), "1050.00"),
        ("wa", "37500", none.clone(), "1736.25"),
        ("wa", "50000", none.clone(), "3250.00"),
        ("wa", "50001", none.clone(), "3250.05"),
        ("sa", "300", none.clone(), "5.00"),
        ("sa", "1001", none.clone(), "12.00"),
        ("sa", "3001", none.clone(), "64.00"),
        ("tas", "0", none.clone(), "20.00"),
        ("tas", "600", none.clone(), "20.00"),
        ("tas", "601", none.clone(), "21.00"),
        ("tas", "35001", none.clone(), "1061.00"),
        ("tas", "40000", none.clone(), "1600.00"),
        ("tas", "40001", none.clone(), "1604.00"),
        ("tas", "40001", json!({"fleet_discount": true}), "1403.50"),
        ("nt", "80000", none.clone(), "2400.00"),
        ("nt", "50000", json!({"electric": true}), "0.00"),
        ("nt", "50001", json!({"electric": true}), "3.00"),
        ("nt", "80000", json!({"electric": true}), "900.00"),
    ];
    for (place, value, changes, want) in cases {
        assert_eq!(
            duty(place, value, changes.clone()),
            want,
            "{place} {value} {changes}"
        );
    }
}

/// The ACT changes determination on 1 February 2027 (DI2026-154 s 2;
/// Legislation Act 2001 (ACT) s 74(2): from the beginning of the day).
#[test]
fn the_act_changes_rates_on_1_february_2027() {
    let new_b = json!({"emissions_category": "b", "new_vehicle": true});
    let on = |day: &str, value: &str, facts: &Value| {
        let mut changes = facts.clone();
        changes["duty_date"] = json!(day);
        duty("act", value, changes)
    };
    assert_eq!(on("2027-01-31", "80000", &new_b), "3177.00");
    assert_eq!(on("2027-02-01", "80000", &new_b), "3419.00");
    assert_eq!(on("2027-01-31", "44999", &new_b), "1350.00");
    assert_eq!(on("2027-02-01", "44999", &new_b), "1462.50");
    // A used vehicle other than Category AAA pays the non-rated rates.
    let used_a = json!({"emissions_category": "a", "new_vehicle": false});
    assert_eq!(on("2026-07-01", "30000", &used_a), "951.00");
    assert_eq!(on("2027-06-30", "30000", &used_a), "1116.00");
    let used_aaa = json!({"emissions_category": "aaa", "new_vehicle": false});
    assert_eq!(on("2027-06-30", "90000", &used_aaa), "3325.00");
}

/// The oracle's cases, each through `derive` as well.
#[test]
fn every_oracle_case_agrees() {
    let text = include_str!("data/motor_vehicle_duty_cases.json");
    let rows: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(rows.len(), 936);
    for row in &rows {
        let facts = &row["input"];
        let place = facts["jurisdiction"].as_str().unwrap();
        let value = facts["dutiable_value"].as_str().unwrap();
        let got = duty(place, value, facts.clone());
        assert_eq!(got, row["duty"].as_str().unwrap(), "{facts}");
    }
}

/// Each step names its authority and shows its working.
#[test]
fn the_steps_show_their_working() {
    let working = |place: &str, value: &str| {
        let envelope = json!({"command": "motor-vehicle-duty", "input": input(place, value)});
        let doc = api::evaluate("derive", &envelope).unwrap();
        let step = &doc["steps"][0];
        assert_eq!(step["id"], "duty");
        let authority = step["authority"].as_str().unwrap().to_string();
        (authority, step["working"].as_str().unwrap().to_string())
    };
    let (authority, sum) = working("nsw", "52000");
    assert!(authority.contains("Service NSW"));
    assert_eq!(
        sum,
        "base + dollars_up((dutiable_value - threshold) / 100) x rate_per_100 = 1350 + dollars_up((52000 - 45000) / 100) x 5 = 1700"
    );
    let (authority, sum) = working("wa", "37500");
    assert!(authority.contains("Duties Act 2008 (WA) s 236"));
    assert_eq!(
        sum,
        "floor(dutiable_value x cents(base_rate + (dutiable_value - threshold) / divisor) / 100 x 20) / 20 = floor(37500 x cents(2.75 + (37500 - 25000) / 6666.66) / 100 x 20) / 20 = 1736.25"
    );
    let (_, sum) = working("sa", "300");
    assert!(sum.starts_with("max(minimum, dollars_up(dutiable_value / 100) x rate_per_100)"));
    let (_, sum) = working("vic", "90000");
    assert!(sum.starts_with("dollars_up(dutiable_value / 200) x rate_per_200"));
    let (_, sum) = working("tas", "500");
    assert_eq!(sum, "base = 20 = 20");
}

fn refused(cases: &[(&str, Value, &str)]) {
    for (place, changes, want) in cases {
        let input = with(input(place, "30000"), changes.clone());
        assert_eq!(code(&input), *want, "{input}");
    }
}

#[test]
fn what_the_acts_do_not_settle_is_refused() {
    refused(&[
        (
            "nsw",
            json!({"jurisdiction": "jbt"}),
            "motor_vehicle_duty.jurisdiction_invalid",
        ),
        (
            "nsw",
            json!({"vehicle_class": "motorcycle"}),
            "motor_vehicle_duty.vehicle_class_unsupported",
        ),
        (
            "nsw",
            json!({"vehicle_class": "heavy"}),
            "motor_vehicle_duty.vehicle_class_unsupported",
        ),
        (
            "nsw",
            json!({"duty_date": "2026-06-30"}),
            "motor_vehicle_duty.date_outside_year",
        ),
        (
            "nsw",
            json!({"duty_date": "2027-07-01"}),
            "motor_vehicle_duty.date_outside_year",
        ),
        (
            "nsw",
            json!({"duty_date": "15/09/2026"}),
            "input.date_invalid",
        ),
        ("nsw", json!({"year": "2025-26"}), "period.unsupported"),
        (
            "nsw",
            json!({"dutiable_value": "30000.50"}),
            "input.amount_not_whole_dollars",
        ),
        (
            "nsw",
            json!({"scope_confirmed": false}),
            "scope.not_confirmed",
        ),
        (
            "qld",
            json!({"engine": "diesel"}),
            "motor_vehicle_duty.engine_invalid",
        ),
        ("qld", json!({"engine": null}), "input.missing_field"),
        (
            "act",
            json!({"emissions_category": "e"}),
            "motor_vehicle_duty.emissions_category_invalid",
        ),
        ("act", json!({"new_vehicle": null}), "input.missing_field"),
        ("act", json!({"new_vehicle": "yes"}), "input.flag_invalid"),
        ("vic", json!({"green_car": null}), "input.missing_field"),
        (
            "tas",
            json!({"fleet_discount": true, "dutiable_value": "600"}),
            "motor_vehicle_duty.fleet_value_unsettled",
        ),
        ("nt", json!({"electric": null}), "input.missing_field"),
        (
            "nsw",
            json!({"engine": "up_to_4_cylinders"}),
            "input.field_unknown",
        ),
        ("qld", json!({"green_car": true}), "input.field_unknown"),
        ("vic", json!({"electric": true}), "input.field_unknown"),
        (
            "wa",
            json!({"fleet_discount": false}),
            "input.field_unknown",
        ),
        ("sa", json!({"new_vehicle": true}), "input.field_unknown"),
        (
            "nt",
            json!({"emissions_category": "aaa"}),
            "input.field_unknown",
        ),
        (
            "tas",
            json!({"primary_producer": false}),
            "input.field_unknown",
        ),
    ]);
}
