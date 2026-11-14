// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The car parking worksheet's commercial parking station, average cost and
//! 12 week register methods against the ATO's worked examples
//! (docs/calculation-evidence.md#fbt-car-parking), the car parking threshold
//! and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn parking(input: Value) -> Result<Value, String> {
    let mut full = json!({"year_ended": 2026, "scope_confirmed": true});
    for (k, v) in input.as_object().unwrap() {
        full[k] = v.clone();
    }
    api::evaluate("fbt-car-parking", &full)
}

fn station(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "commercial_parking_station", "first_business_day_fee": "18.00",
        "benefits": 230, "lowest_fee": "18.00", "employee_contribution": "1000.00",
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    parking(input)
}

fn average(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "average_cost", "first_business_day_fee": "17.00", "benefits": 1,
        "first_day_fee": "17.00", "last_day_fee": "20.00", "employee_contribution": "0.00",
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    parking(input)
}

fn register(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "12_week_register", "first_business_day_fee": "20.00",
        "register_value": "2500.00", "availability_days": 183, "register_first_year": 2026,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    parking(input)
}

fn amounts(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"].clone()
}

#[test]
fn the_commercial_parking_station_method_values_each_benefit_at_the_lowest_fee() {
    // s 39C: 230 benefits at an $18 lowest fee, less $1,000 the employees paid.
    let result = station(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_car_parking");
    assert_eq!(
        result["amounts"],
        json!({"gross_taxable_value": "4140.00", "employee_contribution_applied": "1000.00",
               "excess_employee_contribution": "0.00", "taxable_value": "3140.00"})
    );
    assert_eq!(result["rates"], json!({"car_parking_threshold": "11.03"}));
    // The guide's example 13: $272 a month over 22 business days is a daily
    // rate equivalent the ATO gives as $12.36 (s 39E).
    let a = amounts(station(json!({"lowest_fee": "12.36", "benefits": 22,
                                   "employee_contribution": "0.00"})));
    assert_eq!(a["taxable_value"], "271.92");
}

#[test]
fn the_average_cost_method_matches_the_employers_guide_example() {
    // Example 15, car park A: (17 + 20) / 2 = $18.50 a benefit.
    let a = amounts(average(json!({})));
    assert_eq!(
        a,
        json!({"average_cost": "18.50", "gross_taxable_value": "18.50",
               "employee_contribution_applied": "0.00", "excess_employee_contribution": "0.00",
               "taxable_value": "18.50"})
    );
    // Car park B: (10 + 15) / 2 = $12.50. Its first business day fee, $10, was
    // above the 2023 year's $9.72 threshold but is not above 2026's $11.03, so
    // the same facts give no car parking benefit in 2026.
    let a = amounts(average(json!({"first_business_day_fee": "15.00",
                                   "first_day_fee": "10.00", "last_day_fee": "15.00"})));
    assert_eq!(a["average_cost"], "12.50");
    assert_eq!(a["taxable_value"], "12.50");
    let a = amounts(average(json!({"first_business_day_fee": "10.00",
                                   "first_day_fee": "10.00", "last_day_fee": "15.00"})));
    assert_eq!(a["gross_taxable_value"], "12.50");
    assert_eq!(a["taxable_value"], "0.00");
    // A half-cent average is kept until the gross value is rounded to cents.
    let a = amounts(average(json!({"benefits": 3, "first_day_fee": "17.25",
                                   "last_day_fee": "18.50"})));
    assert_eq!(a["average_cost"], "17.88");
    assert_eq!(a["gross_taxable_value"], "53.63");
}

#[test]
fn the_12_week_register_matches_the_employers_guide_example() {
    // Example 17: $2,500 in the register, x 52 / 12 = $10,833.33, x 183 / 366
    // = $5,416.67. The 2019-20 facts, run in 2026.
    let result = register(json!({})).unwrap();
    assert_eq!(
        result["amounts"],
        json!({"annual_value": "10833.33", "gross_taxable_value": "5416.67",
               "taxable_value": "5416.67"})
    );
    // 366 is the divisor in every year (s 39GB), so a full 365-day year gives
    // a little less than 52 / 12 of the register's value.
    let a = amounts(register(json!({"availability_days": 365})));
    assert_eq!(a["gross_taxable_value"], "10803.73");
}

#[test]
fn the_car_parking_threshold_decides_whether_a_benefit_arises() {
    // s 39A(1)(a)(iii): the first business day's lowest fee must be more than
    // the threshold, $11.03 for 2026 and $11.48 for 2027.
    for (year, fee, taxable) in [
        (2026, "11.03", "0.00"),
        (2026, "11.04", "3140.00"),
        (2027, "11.48", "0.00"),
        (2027, "11.49", "3140.00"),
    ] {
        let result = station(json!({"year_ended": year, "first_business_day_fee": fee})).unwrap();
        assert_eq!(result["amounts"]["taxable_value"], taxable, "{year} {fee}");
    }
    let result = station(json!({"year_ended": 2027})).unwrap();
    assert_eq!(result["rates"]["car_parking_threshold"], "11.48");
    // The guide's example 12: one station's fee above the threshold makes the
    // benefits arise, and they are valued at another station's lower fee
    // ($10 and $5.50 against 2023's $9.72; here $12 against 2026's $11.03).
    let a = amounts(station(
        json!({"first_business_day_fee": "12.00", "lowest_fee": "5.50",
                                   "benefits": 10, "employee_contribution": "0.00"}),
    ));
    assert_eq!(a["taxable_value"], "55.00");
    let a = amounts(register(json!({"first_business_day_fee": "9.72"})));
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn a_recipients_payment_never_takes_the_value_below_nil() {
    let a = amounts(station(json!({"employee_contribution": "5000.00"})));
    assert_eq!(a["employee_contribution_applied"], "4140.00");
    assert_eq!(a["excess_employee_contribution"], "860.00");
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let cases = [
        (
            station(json!({"year_ended": 2028})),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
        (
            station(json!({"method": "spaces"})),
            "method must be commercial_parking_station, average_cost or 12_week_register.",
        ),
        (
            register(json!({"employee_contribution": "1.00"})),
            "employee_contribution is not an input of fbt-car-parking with method 12_week_register",
        ),
        (
            station(json!({"first_day_fee": "1.00"})),
            "first_day_fee is not an input of fbt-car-parking with method commercial_parking_station",
        ),
        (
            register(json!({"register_first_year": 2021})),
            "register_first_year must be the FBT year, from 2022 to 2026, the register was first valid for: a register is valid for that year and the 4 after it (FBTAA s 39GF).",
        ),
        (
            register(json!({"availability_days": 366})),
            "availability_days must be an integer from 0 to 365.",
        ),
        (
            station(json!({"benefits": -1})),
            "benefits must be a JSON integer from 0.",
        ),
        (
            station(json!({"benefits": "230"})),
            "benefits must be a JSON integer from 0.",
        ),
        (
            station(json!({"lowest_fee": "18.001"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            parking(
                json!({"method": "average_cost", "first_business_day_fee": "17.00",
                           "benefits": 1, "first_day_fee": "17.00", "employee_contribution": "0.00"}),
            ),
            "last_day_fee is required with method average_cost",
        ),
        (
            station(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result.map(|_| ()), Err(message.to_string()));
    }
    let mut input = json!({"method": "commercial_parking_station", "first_business_day_fee": "18.00",
        "benefits": -1, "lowest_fee": "18.00", "employee_contribution": "0.00",
        "year_ended": 2026, "scope_confirmed": true});
    let code = |input: &Value| {
        api::evaluate_coded("fbt-car-parking", input)
            .unwrap_err()
            .code
    };
    assert_eq!(code(&input), "input.benefit_count_invalid");
    input["method"] = json!("market_value");
    assert_eq!(code(&input), "input.fbt_method_invalid");
}

#[test]
fn the_catalogue_names_the_methods_and_its_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_car_parking"];
    assert_eq!(
        entry["methods"],
        json!([
            "commercial_parking_station",
            "average_cost",
            "12_week_register"
        ])
    );
    assert_eq!(entry["source_checked"], "2026-11-15");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-car-parking", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "3140.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_car_parking"})).unwrap()[1]["end"],
        "2027-03-31"
    );
}
