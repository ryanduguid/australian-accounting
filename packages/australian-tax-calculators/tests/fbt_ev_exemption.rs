// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The electric cars exemption (FBTAA s 8A, compilation 97; Treasury Laws
//! Amendment (Electric Car Discount) Act 2022 Sch 1 item 7 and Sch 2 item 5;
//! LCT Act s 25-1) against the ATO's examples in 'Electric cars exemption' and
//! 'FBT on plug-in hybrid electric vehicles', the Library's Cleancut example
//! (Tax Examples, FBT, 3-045) and each failing condition, read on 19 November
//! 2026 (docs/calculation-evidence.md#fbt-ev-exemption-check). The ATO's
//! examples are set in their own years; each is run here for a benefit in the
//! FBT year ended 31 March 2026 or 2027, and a price an example does not give
//! is fabricated below the threshold.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "fbt-ev-exemption-check";

/// A battery electric car first held and used on `first`, first sold in
/// `sale_year` for `value`, for a current employee in the year ended 31
/// March 2026.
fn electric(first: &str, sale_year: &str, value: &str) -> Value {
    json!({
        "vehicle": "battery_electric", "car": true, "current_employee": true,
        "first_held_and_used": first, "first_retail_sale_year": sale_year,
        "luxury_car_tax_value": value, "luxury_car_tax_on_other_supply": false,
        "year_ended": 2026, "scope_confirmed": true,
    })
}

/// A plug-in hybrid from `electric`, with Schedule 2 item 5(2)'s facts.
fn hybrid(first: &str, before: bool, new: bool, exempt: bool, year_ended: i64) -> Value {
    let mut input = electric(first, "2023-24", "60000.00");
    let extra = json!({
        "vehicle": "plug_in_hybrid_electric", "year_ended": year_ended,
        "commitment_before_1_april_2025": before,
        "new_commitment_from_1_april_2025": new,
        "exempt_benefit_before_1_april_2025": exempt,
    });
    for (key, value) in extra.as_object().unwrap() {
        input[key] = value.clone();
    }
    input
}

fn with(mut input: Value, key: &str, value: Value) -> Value {
    input[key] = value;
    input
}

fn run(input: &Value) -> Value {
    api::evaluate(COMMAND, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(COMMAND, input).unwrap_err().code
}

fn failing(result: &Value) -> Vec<&str> {
    result["failing_conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect()
}

fn assert_exempt(result: &Value) {
    assert_eq!(result["exempt"], true, "{result}");
    assert!(failing(result).is_empty());
    assert_eq!(
        result["car_worksheet_input"],
        json!({"electric_car_exempt": true})
    );
}

/// The Library's Cleancut example: a Hyundai Ioniq 5 bought on 1 July 2025
/// for $72,000 excluding GST, $79,200 with it, below the 2025-26 threshold
/// of $91,387, so FBT-free and reportable.
#[test]
fn cleancut_electric_car_is_exempt_and_reportable() {
    let r = run(&electric("2025-07-01", "2025-26", "79200.00"));
    assert_exempt(&r);
    assert_eq!(r["calculation"], "fbt_ev_exemption_check");
    assert_eq!(r["period"], "year ended 31 March 2026");
    assert_eq!(r["amounts"]["luxury_car_tax_value"], "79200.00");
    assert_eq!(
        r["rates"]["fuel_efficient_luxury_car_tax_threshold"],
        "91387"
    );
    assert_eq!(r["rates"]["first_retail_sale_year"], "2025-26");
    let conditions: Vec<&str> = r["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["condition"].as_str().unwrap())
        .collect();
    assert_eq!(
        conditions,
        [
            "car",
            "current_employee",
            "zero_or_low_emissions_vehicle",
            "first_held_and_used_from_1_july_2022",
            "no_luxury_car_tax_at_first_retail_sale",
            "no_luxury_car_tax_on_other_supply",
        ]
    );
    let consequences = r["consequences"].as_array().unwrap();
    assert_eq!(consequences.len(), 3);
    assert!(consequences[1].as_str().unwrap().contains("s 135P(3)"));
    assert!(consequences[2].as_str().unwrap().contains("s 53(1)"));
    assert!(r["warnings"][2].as_str().unwrap().contains("not yet law"));
}

/// Shelly's car (exemption does not apply): bought and first used on 1
/// April 2022, so ABC Co's later benefits are not exempt.
#[test]
fn shelly_car_first_used_before_1_july_2022_is_not_exempt() {
    let r = run(&electric("2022-04-01", "2021-22", "60000.00"));
    assert_eq!(r["exempt"], false);
    assert_eq!(failing(&r), ["first_held_and_used_from_1_july_2022"]);
    assert_eq!(
        r["car_worksheet_input"],
        json!({"electric_car_exempt": false})
    );
    assert_eq!(r["consequences"].as_array().unwrap().len(), 1);
}

/// John's car (exemption applies): held from 15 June 2022 and first used on
/// 5 July 2022, not subject to luxury car tax, so XYZ Co's benefits are
/// exempt. The day before 1 July 2022 is not.
#[test]
fn john_car_first_used_from_1_july_2022_is_exempt() {
    assert_exempt(&run(&electric("2022-07-05", "2021-22", "60000.00")));
    assert_exempt(&run(&electric("2022-07-01", "2021-22", "60000.00")));
    let r = run(&electric("2022-06-30", "2021-22", "60000.00"));
    assert_eq!(failing(&r), ["first_held_and_used_from_1_july_2022"]);
}

/// GHI Co's car: $85,000 with GST in 2022-23, above that year's $84,916, so
/// luxury car tax was payable and Kerry's benefits are not exempt.
#[test]
fn ghi_car_above_the_threshold_at_first_sale_is_not_exempt() {
    let r = run(&electric("2022-07-01", "2022-23", "85000.00"));
    assert_eq!(failing(&r), ["no_luxury_car_tax_at_first_retail_sale"]);
    assert_eq!(
        r["rates"]["fuel_efficient_luxury_car_tax_threshold"],
        "84916"
    );
}

/// A luxury car is one whose value exceeds the threshold (LCT Act
/// s 25-1(1)): at the threshold no luxury car tax is payable; a cent more
/// is.
#[test]
fn the_threshold_itself_is_not_exceeded() {
    assert_exempt(&run(&electric("2025-07-01", "2025-26", "91387.00")));
    let r = run(&electric("2025-07-01", "2025-26", "91387.01"));
    assert_eq!(failing(&r), ["no_luxury_car_tax_at_first_retail_sale"]);
    let later = with(
        electric("2026-07-01", "2026-27", "91661.00"),
        "year_ended",
        json!(2027),
    );
    assert_exempt(&run(&later));
}

/// Every threshold the ATO's table lists for fuel-efficient vehicles.
#[test]
fn each_listed_year_has_its_threshold() {
    let table = [
        ("2012-13", "75375"),
        ("2015-16", "75375"),
        ("2016-17", "75526"),
        ("2019-20", "75526"),
        ("2020-21", "77565"),
        ("2021-22", "79659"),
        ("2022-23", "84916"),
        ("2023-24", "89332"),
        ("2024-25", "91387"),
        ("2025-26", "91387"),
    ];
    for (year, limit) in table {
        let r = run(&electric("2025-07-01", year, "1.00"));
        assert_eq!(r["rates"]["fuel_efficient_luxury_car_tax_threshold"], limit);
    }
}

/// The other failing conditions, one at a time.
#[test]
fn each_other_condition_fails_alone() {
    let base = electric("2025-07-01", "2025-26", "79200.00");
    let cases = [
        ("current_employee", json!(false), "current_employee"),
        ("vehicle", json!("other"), "zero_or_low_emissions_vehicle"),
        (
            "luxury_car_tax_on_other_supply",
            json!(true),
            "no_luxury_car_tax_on_other_supply",
        ),
    ];
    for (key, value, condition) in cases {
        let r = run(&with(base.clone(), key, value));
        assert_eq!(r["exempt"], false);
        assert_eq!(failing(&r), [condition], "{key}");
    }
    let fuel_cell = with(
        base.clone(),
        "vehicle",
        json!("hydrogen_fuel_cell_electric"),
    );
    assert_exempt(&run(&fuel_cell));
    // A motor cycle is not a car: no car benefit, so no car worksheet input.
    let r = run(&with(base, "car", json!(false)));
    assert_eq!(failing(&r), ["car"]);
    assert!(r.get("car_worksheet_input").is_none());
    assert!(
        r["consequences"][0]
            .as_str()
            .unwrap()
            .contains("fbt-residual")
    );
}

/// Several failing conditions are all named, in order.
#[test]
fn every_failing_condition_is_named() {
    let input = with(
        with(
            electric("2022-04-01", "2021-22", "90000.00"),
            "vehicle",
            json!("other"),
        ),
        "current_employee",
        json!(false),
    );
    assert_eq!(
        failing(&run(&input)),
        [
            "current_employee",
            "zero_or_low_emissions_vehicle",
            "first_held_and_used_from_1_july_2022",
            "no_luxury_car_tax_at_first_retail_sale",
        ]
    );
}

/// Simon's novated lease from 1 April 2024 to 31 March 2027: used before 1
/// April 2025 under a binding commitment, so exempt up to 31 March 2027,
/// the end of the FBT year ended 31 March 2027.
#[test]
fn simon_plug_in_hybrid_under_its_commitment_stays_exempt() {
    let r = run(&hybrid("2024-04-01", true, false, true, 2027));
    assert_exempt(&r);
    let ids: Vec<&str> = r["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["condition"].as_str().unwrap())
        .collect();
    assert_eq!(
        &ids[2..5],
        [
            "plug_in_hybrid_commitment_before_1_april_2025",
            "plug_in_hybrid_no_new_commitment",
            "plug_in_hybrid_exempt_before_1_april_2025",
        ]
    );
}

/// Sonia's plug-in hybrid, delivered on 1 May 2025: not used or available
/// for her private use before 1 April 2025, so not exempt.
#[test]
fn sonia_plug_in_hybrid_delivered_late_is_not_exempt() {
    let r = run(&hybrid("2025-05-01", true, false, false, 2026));
    assert_eq!(failing(&r), ["plug_in_hybrid_exempt_before_1_april_2025"]);
}

/// Tony's unpaid leave from 1 May 2025 ends the novation, a new
/// commitment; Courtney's new employer makes one too and had none before 1
/// April 2025; a pool car has no commitment to a designated employee.
#[test]
fn a_new_or_missing_commitment_ends_a_plug_in_hybrid_exemption() {
    let tony = run(&hybrid("2024-09-01", true, true, true, 2026));
    assert_eq!(failing(&tony), ["plug_in_hybrid_no_new_commitment"]);
    let courtney = run(&hybrid("2024-06-01", false, true, true, 2026));
    assert_eq!(
        failing(&courtney),
        [
            "plug_in_hybrid_commitment_before_1_april_2025",
            "plug_in_hybrid_no_new_commitment",
        ]
    );
    let pool = run(&hybrid("2024-06-01", false, false, true, 2026));
    assert_eq!(
        failing(&pool),
        ["plug_in_hybrid_commitment_before_1_april_2025"]
    );
}

/// The check's input runs the car worksheets: an exempt car's taxable value
/// is nil and its notional value stays reportable.
#[test]
fn the_result_feeds_the_statutory_car_worksheet() {
    let check = run(&electric("2025-07-01", "2025-26", "79200.00"));
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut car = catalogue["fbt_car_statutory"]["example"]["facts"].clone();
    car.as_object_mut().unwrap().remove("kind");
    car["electric_car_exempt"] = check["car_worksheet_input"]["electric_car_exempt"].clone();
    let valued = api::evaluate("fbt-car-statutory", &car).unwrap();
    assert_eq!(valued["amounts"]["taxable_value"], "0.00");
    assert_eq!(
        valued["amounts"]["reportable_taxable_value"],
        valued["amounts"]["taxable_value_before_exemption"]
    );
}

#[test]
fn refuses_what_it_cannot_decide() {
    let base = electric("2025-07-01", "2025-26", "79200.00");
    let unsupported = "fbt_ev_exemption.threshold_year_unsupported";
    for year in [
        json!("2011-12"),
        json!("2027-28"),
        json!("2025-2026"),
        json!(2025),
    ] {
        assert_eq!(
            code(&with(base.clone(), "first_retail_sale_year", year)),
            unsupported
        );
    }
    let conflict = "fbt_ev_exemption.fact_conflict";
    assert_eq!(
        code(&with(
            base.clone(),
            "first_held_and_used",
            json!("2026-04-01")
        )),
        conflict
    );
    assert_eq!(
        code(&with(
            base.clone(),
            "first_retail_sale_year",
            json!("2026-27")
        )),
        conflict
    );
    assert_eq!(
        code(&hybrid("2025-04-01", true, false, true, 2026)),
        conflict
    );
    assert_eq!(
        code(&with(base.clone(), "vehicle", json!("hybrid"))),
        "fbt_ev_exemption.vehicle_invalid"
    );
    assert_eq!(
        code(&with(
            base.clone(),
            "first_held_and_used",
            json!("1 July 2025")
        )),
        "input.date_invalid"
    );
    assert_eq!(
        code(&with(base.clone(), "car", json!("yes"))),
        "input.flag_invalid"
    );
    assert_eq!(
        code(&with(base.clone(), "luxury_car_tax_value", json!("-1.00"))),
        "input.amount_invalid"
    );
    // A plug-in hybrid's facts are for a plug-in hybrid only, and required there.
    let extra = with(base.clone(), "commitment_before_1_april_2025", json!(true));
    assert_eq!(code(&extra), "input.field_unknown");
    let mut short = hybrid("2024-04-01", true, false, true, 2026);
    short
        .as_object_mut()
        .unwrap()
        .remove("new_commitment_from_1_april_2025");
    assert_eq!(code(&short), "input.missing_field");
    assert_eq!(
        code(&with(base.clone(), "not_an_input", json!(1))),
        "input.field_unknown"
    );
    // From 1 April 2027 the announced change is not law.
    let refused = api::evaluate(COMMAND, &with(base.clone(), "year_ended", json!(2028)));
    assert_eq!(
        refused,
        Err("Unsupported period 2028; supported: 2026, 2027.".to_string())
    );
    assert_eq!(
        code(&with(base, "scope_confirmed", json!(false))),
        "scope.not_confirmed"
    );
}

#[test]
fn the_catalogue_describes_the_check() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_ev_exemption_check"];
    assert_eq!(entry["required_inputs"]["year_ended"], "integer");
    assert_eq!(entry["source_checked"], "2026-11-19");
    assert_eq!(
        entry["example_evidence"]["note"],
        "docs/calculation-evidence.md#fbt-ev-exemption-check"
    );
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_exempt(&run(&facts));
    assert_eq!(
        entry["optional_inputs"].as_object().unwrap().len(),
        3,
        "the plug-in hybrid facts"
    );
}
