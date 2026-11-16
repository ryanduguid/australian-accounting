// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The car parking worksheet's commercial parking station, market value,
//! average cost, spaces and 12 week register methods against the ATO's worked
//! examples (docs/calculation-evidence.md#fbt-car-parking), the car parking
//! threshold and its refusals.
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
    assert_eq!(a, json!({"taxable_value": "0.00"}));
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
    // $26.23 x 52 x 153 / (12 x 366) is exactly $47.515, which rounds half up
    // to $47.52; dividing by 12 and 366 in turn at working precision gives $47.51.
    let a = amounts(register(
        json!({"register_value": "26.23", "availability_days": 153}),
    ));
    assert_eq!(a["gross_taxable_value"], "47.52");
}

/// The employer's guide, example 18: 4 spaces for 3 employees, each at a $10
/// daily rate amount for 182 days, with $2,340 contributed; the first business
/// day fee is set above 2026's threshold.
fn spaces(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "method": "spaces",
        "space_groups": [{"first_business_day_fee": "12.00", "daily_rate_amount": "10.00",
                          "availability_days": 182, "spaces": 4}],
        "employees_at_start": 3, "employees_at_end": 3,
        "eligible_spaces_at_start": 4, "eligible_spaces_at_end": 4,
        "employee_contribution": "2340.00",
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    parking(input)
}

#[test]
fn the_market_value_method_takes_the_valuers_amount_for_each_benefit() {
    // s 39D(2): the guide's example 17 values each of its register's 250
    // benefits at a $20 arm's length value less a $10 recipient contribution,
    // $10 a benefit and $2,500 in all; the payments come off as under s 39C.
    let input = json!({"method": "market_value", "first_business_day_fee": "12.00",
                       "benefits": 250, "market_value": "20.00",
                       "employee_contribution": "2500.00"});
    let result = parking(input.clone()).unwrap();
    assert_eq!(
        result["amounts"],
        json!({"gross_taxable_value": "5000.00", "employee_contribution_applied": "2500.00",
               "excess_employee_contribution": "0.00", "taxable_value": "2500.00"})
    );
    let mut paid = input.clone();
    paid["employee_contribution"] = json!("5500.00");
    let a = amounts(parking(paid));
    assert_eq!(a["excess_employee_contribution"], "500.00");
    assert_eq!(a["taxable_value"], "0.00");
    // The threshold still decides whether a benefit arises (s 39A(1)(a)(iii)).
    let mut below = input;
    below["first_business_day_fee"] = json!("11.03");
    assert_eq!(amounts(parking(below)), json!({"taxable_value": "0.00"}));
}

#[test]
fn the_spaces_method_matches_the_employers_guide_example() {
    // Example 18: 10 x 182 / 366 x 228 = 1,133.77 a space, 4,535.08 for 4;
    // 3 employees average fewer than 4 spaces, so x 3 / 4 = 3,401.31 (s 39FB),
    // less $30 x 26 weeks x 3 employees = $2,340, $1,061.31.
    let result = spaces(json!({})).unwrap();
    assert_eq!(
        result["amounts"],
        json!({"total_statutory_benefit": "4535.08", "average_employees": "3.00",
               "average_eligible_spaces": "4.00", "gross_taxable_value": "3401.31",
               "employee_contribution_applied": "2340.00",
               "excess_employee_contribution": "0.00", "taxable_value": "1061.31"})
    );
    // Without s 39FB, as when the employees average as many as the spaces.
    let a = amounts(spaces(
        json!({"employees_at_start": 4, "employees_at_end": 4}),
    ));
    assert_eq!(a["gross_taxable_value"], "4535.08");
    assert_eq!(a["taxable_value"], "2195.08");
    // Averages from the start and end of the parking period: 3.5 employees
    // over 4.5 spaces, 4,535.08 x 3.5 / 4.5 = 3,527.2844, $3,527.28.
    let a = amounts(spaces(
        json!({"employees_at_end": 4, "eligible_spaces_at_end": 5}),
    ));
    assert_eq!(a["average_employees"], "3.50");
    assert_eq!(a["average_eligible_spaces"], "4.50");
    assert_eq!(a["gross_taxable_value"], "3527.28");
}

#[test]
fn the_spaces_method_adds_each_group_and_divides_once() {
    // A second group at $12 for the whole 365-day year: (10 x 182 x 228 x 4 +
    // 12 x 365 x 228) / 366 = 2,658,480 / 366 = 7,263.6066, $7,263.61. Each
    // space alone, rounded, would give 1,133.77 x 4 + 2,728.52 = 7,263.60.
    let groups = json!([
        {"first_business_day_fee": "12.00", "daily_rate_amount": "10.00",
         "availability_days": 182, "spaces": 4},
        {"first_business_day_fee": "20.00", "daily_rate_amount": "12.00",
         "availability_days": 365, "spaces": 1},
    ]);
    let input = json!({"space_groups": groups, "employees_at_start": 5, "employees_at_end": 5,
                       "eligible_spaces_at_start": 5, "eligible_spaces_at_end": 5,
                       "employee_contribution": "0.00"});
    let a = amounts(spaces(input));
    assert_eq!(a["total_statutory_benefit"], "7263.61");
    assert_eq!(a["taxable_value"], "7263.61");
    // 2027's threshold is $11.48: a group's fee of $11.49 gives a benefit.
    let year = json!({"year_ended": 2027, "space_groups": [{"first_business_day_fee": "11.49",
        "daily_rate_amount": "10.00", "availability_days": 182, "spaces": 4}]});
    let result = spaces(year).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "1061.31");
    assert_eq!(result["rates"]["car_parking_threshold"], "11.48");
}

#[test]
fn the_spaces_method_refuses_spaces_without_a_benefit_and_malformed_groups() {
    let group = |fee: &str, days: Value, count: Value| {
        json!({"space_groups": [{"first_business_day_fee": fee, "daily_rate_amount": "10.00",
                                 "availability_days": days, "spaces": count}]})
    };
    let shape = "space_groups must be a list of 1 to 1000000 objects, each with exactly first_business_day_fee and daily_rate_amount as decimal strings, availability_days as a JSON integer from 1 to the days in the FBT year and spaces as a JSON integer from 1 to 1000000.";
    let cases = [
        (
            spaces(group("11.03", json!(182), json!(4))),
            "A space group's first_business_day_fee, 11.03, is not more than the car parking threshold, 11.03: its spaces gave rise to no car parking benefit (FBTAA s 39A(1)(a)(iii)), so the spaces method does not count them (s 39FA(4), step 1). Leave the group out.",
        ),
        (spaces(group("12.00", json!(0), json!(4))), shape),
        (spaces(group("12.00", json!(366), json!(4))), shape),
        (spaces(group("12.00", json!(182), json!(0))), shape),
        (spaces(group("12.00", json!(182), json!("4"))), shape),
        (spaces(json!({"space_groups": []})), shape),
        (spaces(json!({"space_groups": "10.00"})), shape),
        (
            spaces(json!({"space_groups": [{"daily_rate_amount": "10.00",
                                            "availability_days": 182, "spaces": 4}]})),
            shape,
        ),
        (
            spaces(json!({"space_groups": [{"first_business_day_fee": "12.00",
                "daily_rate_amount": 10, "availability_days": 182, "spaces": 4}]})),
            shape,
        ),
        (
            spaces(json!({"employees_at_start": -1})),
            "employees_at_start, employees_at_end, eligible_spaces_at_start and eligible_spaces_at_end must each be a JSON integer from 0 to 1000000 (FBTAA s 39FB(3) and (4)).",
        ),
        (
            spaces(json!({"benefits": 4})),
            "benefits is not an input of fbt-car-parking with method spaces",
        ),
        (
            spaces(json!({"first_business_day_fee": "12.00"})),
            "first_business_day_fee is not an input of fbt-car-parking with method spaces",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result.map(|_| ()), Err(message.to_string()));
    }
    let input = json!({"method": "spaces", "year_ended": 2026, "scope_confirmed": true,
        "space_groups": [{"first_business_day_fee": "1.00", "daily_rate_amount": "10.00",
                          "availability_days": 182, "spaces": 4}],
        "employees_at_start": 3, "employees_at_end": 3,
        "eligible_spaces_at_start": 4, "eligible_spaces_at_end": 4,
        "employee_contribution": "0.00"});
    let code = api::evaluate_coded("fbt-car-parking", &input)
        .unwrap_err()
        .code;
    assert_eq!(code, "fbt_car_parking.spaces_without_benefit");
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
    assert_eq!(a, json!({"taxable_value": "0.00"}));
    // With no benefit, the result holds no gross value or payment to copy
    // into a return: only the nil taxable value.
    let a = amounts(station(json!({"first_business_day_fee": "11.03"})));
    assert_eq!(a, json!({"taxable_value": "0.00"}));
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
            station(json!({"method": "statutory_formula"})),
            "method must be commercial_parking_station, market_value, average_cost, spaces or 12_week_register.",
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
        // A list reaches the worksheet as a list, which no amount accepts.
        (
            station(json!({"lowest_fee": ["18.00"]})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            station(json!({"benefits": [230]})),
            "benefits must be a JSON integer from 0.",
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
    input["method"] = json!("actual");
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
            "market_value",
            "average_cost",
            "spaces",
            "12_week_register"
        ])
    );
    assert_eq!(entry["source_checked"], "2026-11-16");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-car-parking", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "3140.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_car_parking"})).unwrap()[1]["end"],
        "2027-03-31"
    );
}
