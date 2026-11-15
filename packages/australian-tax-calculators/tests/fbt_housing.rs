// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The housing worksheet's market value, base year and indexed statutory
//! annual value methods against the ATO's worked examples
//! (docs/calculation-evidence.md#fbt-housing), the published indexation
//! factors and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// `base` with `overrides`; a null override removes the input.
fn housing(base: Value, overrides: Value) -> Result<Value, String> {
    let mut input = base;
    input["year_ended"] = json!(2026);
    input["scope_confirmed"] = json!(true);
    let fields = input.as_object_mut().unwrap();
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            fields.remove(k);
        } else {
            fields.insert(k.clone(), v.clone());
        }
    }
    api::evaluate("fbt-housing", &input)
}

/// FBT return 2026 instructions, item 23F example 11: a house with a market
/// rent of $41,600 for the year, the employee paying $5,200, in its base year.
fn base_year(overrides: Value) -> Result<Value, String> {
    let base = json!({"method": "base_year", "market_value": "41600.00",
                      "tenancy_days": 365, "employee_contribution": "5200.00"});
    housing(base, overrides)
}

/// The employer's guide, 10.5: the unit's statutory annual value from its base
/// year, $15,745.62, indexed for New South Wales.
fn indexed(overrides: Value) -> Result<Value, String> {
    let base = json!({"method": "indexed", "prior_statutory_annual_value": "15745.62",
                      "state": "NSW", "base_year_ended": 2025, "tenancy_days": 91,
                      "employee_contribution": "0.00"});
    let mut overrides = overrides;
    if overrides.get("year_ended").is_none() {
        overrides["year_ended"] = json!(2027);
    }
    housing(base, overrides)
}

fn amounts(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"].clone()
}

#[test]
fn matches_the_fbt_return_2026_example() {
    // $41,600 - $5,200 = $36,400.
    let result = base_year(json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_housing");
    assert_eq!(
        result["amounts"],
        json!({"statutory_annual_value": "41600.00", "gross_taxable_value": "41600.00",
               "employee_contribution_applied": "5200.00",
               "excess_employee_contribution": "0.00", "taxable_value": "36400.00"})
    );
    assert_eq!(result["rates"], json!({"days_in_year": "365"}));
}

#[test]
fn a_base_year_annualises_the_market_value_for_the_tenancy() {
    // Guide 10.5: $11,820 of market rent over 274 days of the base year is a
    // statutory annual value of $15,745.62, so the year's value is $11,820.
    let a = amounts(base_year(
        json!({"market_value": "11820.00", "tenancy_days": 274,
                                     "employee_contribution": "0.00"}),
    ));
    assert_eq!(a["statutory_annual_value"], "15745.62");
    assert_eq!(a["gross_taxable_value"], "11820.00");
    // Guide 10.5: $5,200 over 121 days. The guide rounds the daily rate first
    // and prints $15,687.70; s 26(2)(a) gives $5,200 x 365 / 121 = $15,685.95.
    let a = amounts(base_year(
        json!({"market_value": "5200.00", "tenancy_days": 121,
                                     "employee_contribution": "0.00"}),
    ));
    assert_eq!(a["statutory_annual_value"], "15685.95");
    assert_eq!(a["taxable_value"], "5200.00");
    // The Library's ¶3-120: $350 a week for 39 weeks, less $50 a week rent,
    // $13,650 - $1,950 = $11,700.
    let a = amounts(base_year(
        json!({"market_value": "13650.00", "tenancy_days": 273,
                                     "employee_contribution": "1950.00"}),
    ));
    assert_eq!(a["statutory_annual_value"], "18250.00");
    assert_eq!(a["taxable_value"], "11700.00");
}

#[test]
fn an_indexed_year_applies_the_published_factor() {
    // Guide 10.5's unit in a later year: $15,745.62 x 1.040 (NSW, 2027) =
    // $16,375.44, for 91 days of 365, $4,082.64.
    let result = indexed(json!({})).unwrap();
    assert_eq!(
        result["amounts"],
        json!({"statutory_annual_value": "16375.44", "gross_taxable_value": "4082.64",
               "employee_contribution_applied": "0.00",
               "excess_employee_contribution": "0.00", "taxable_value": "4082.64"})
    );
    assert_eq!(
        result["rates"],
        json!({"indexation_factor": "1.040", "days_in_year": "365"})
    );
    // 2026's NSW factor, 1.076, then 2027's on that value for 243 days.
    let a = amounts(indexed(
        json!({"year_ended": 2026, "base_year_ended": 2025}),
    ));
    assert_eq!(a["statutory_annual_value"], "16942.29");
    assert_eq!(a["gross_taxable_value"], "4223.97");
    let a = amounts(indexed(json!({"prior_statutory_annual_value": "16942.29",
                                  "tenancy_days": 243})));
    assert_eq!(a["statutory_annual_value"], "17619.98");
    assert_eq!(a["taxable_value"], "11730.56");
}

#[test]
fn each_state_and_territory_has_its_factor_for_each_year() {
    let factors = [
        ("NSW", "1.076", "1.040"),
        ("VIC", "1.066", "1.044"),
        ("QLD", "1.076", "1.051"),
        ("SA", "1.060", "1.051"),
        ("WA", "1.100", "1.073"),
        ("TAS", "0.988", "0.999"),
        ("NT", "1.031", "1.029"),
        ("ACT", "1.010", "1.007"),
    ];
    for (state, f2026, f2027) in factors {
        for (year, factor) in [(2026, f2026), (2027, f2027)] {
            let result = indexed(json!({"state": state, "year_ended": year,
                                        "base_year_ended": year - 1}))
            .unwrap();
            assert_eq!(
                result["rates"]["indexation_factor"], factor,
                "{state} {year}"
            );
        }
    }
}

#[test]
fn market_value_methods_take_the_value_less_rent() {
    // s 26(1)(a): outside Australia, the market value less rent.
    let outside = json!({"method": "outside_australia", "market_value": "30000.00",
                         "employee_contribution": "31000.00"});
    let a = amounts(housing(outside, json!({})));
    assert_eq!(a["gross_taxable_value"], "30000.00");
    assert_eq!(a["employee_contribution_applied"], "30000.00");
    assert_eq!(a["excess_employee_contribution"], "1000.00");
    assert_eq!(a["taxable_value"], "0.00");
    // s 26(1)(b): a hotel employee's room; 75% where it would be an in-house
    // residual benefit, rounded to cents.
    let business = json!({"method": "accommodation_business", "market_value": "10000.01",
                          "in_house": true, "employee_contribution": "0.00"});
    let result = housing(business.clone(), json!({})).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "7500.01");
    assert_eq!(result["rates"], json!({"in_house_factor": "0.75"}));
    let a = amounts(housing(business, json!({"in_house": false})));
    assert_eq!(a["taxable_value"], "10000.01");
}

#[test]
fn tenancy_days_bound_each_method() {
    // A base year needs a tenancy day to annualise over; an indexed year may
    // have none and then has no value.
    let a = amounts(base_year(
        json!({"tenancy_days": 1, "market_value": "100.00"}),
    ));
    assert_eq!(a["statutory_annual_value"], "36500.00");
    assert_eq!(a["gross_taxable_value"], "100.00");
    let a = amounts(indexed(json!({"tenancy_days": 0})));
    assert_eq!(a["taxable_value"], "0.00");
    // The 2028 year would have 366 days; neither supported year does.
    let a = amounts(indexed(json!({"tenancy_days": 365})));
    assert_eq!(a["gross_taxable_value"], "16375.44");
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let cases = [
        (
            base_year(json!({"tenancy_days": 0})),
            "tenancy_days must be an integer from 1 to 365 with method base_year.",
        ),
        (
            base_year(json!({"tenancy_days": 366})),
            "tenancy_days must be an integer from 1 to 365 with method base_year.",
        ),
        (
            indexed(json!({"tenancy_days": 366})),
            "tenancy_days must be an integer from 0 to 365.",
        ),
        (
            indexed(json!({"state": "nsw"})),
            "state must be NSW, VIC, QLD, SA, WA, TAS, NT or ACT, where the unit of accommodation is: the Jervis Bay Territory is part of NSW, and Christmas Island and the Cocos (Keeling) Islands are part of NT (FBTAA s 28(5)).",
        ),
        (
            indexed(json!({"base_year_ended": 2017})),
            "base_year_ended must be the FBT year, from 2018 to 2026, last treated as a base year for this housing right: a year after 9 years without one is a base year (FBTAA s 26(3)(c)), so use method base_year.",
        ),
        (
            indexed(json!({"base_year_ended": 2027})),
            "base_year_ended must be the FBT year, from 2018 to 2026, last treated as a base year for this housing right: a year after 9 years without one is a base year (FBTAA s 26(3)(c)), so use method base_year.",
        ),
        (
            indexed(json!({"state": null})),
            "state is required with method indexed",
        ),
        (
            base_year(json!({"state": "NSW"})),
            "state is not an input of fbt-housing with method base_year",
        ),
        (
            housing(
                json!({"method": "accommodation_business", "market_value": "1.00",
                       "in_house": 1, "employee_contribution": "0.00"}),
                json!({}),
            ),
            "in_house must be a boolean.",
        ),
        (
            base_year(json!({"method": "remote_area"})),
            "method must be outside_australia, accommodation_business, base_year or indexed.",
        ),
        (
            base_year(json!({"market_value": "1.001"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            base_year(json!({"year_ended": 2025})),
            "Unsupported period 2025; supported: 2026, 2027.",
        ),
        (
            base_year(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result.map(|_| ()), Err(message.to_string()));
    }
    let code = |input: Value| {
        let mut full = json!({"method": "indexed", "prior_statutory_annual_value": "1.00",
                              "state": "NSW", "base_year_ended": 2026, "tenancy_days": 1,
                              "employee_contribution": "0.00", "year_ended": 2027,
                              "scope_confirmed": true});
        for (k, v) in input.as_object().unwrap() {
            full[k] = v.clone();
        }
        api::evaluate_coded("fbt-housing", &full).unwrap_err().code
    };
    assert_eq!(code(json!({"state": "JBT"})), "input.state_invalid");
    assert_eq!(code(json!({"state": 1})), "input.state_invalid");
    assert_eq!(
        code(json!({"base_year_ended": 2017})),
        "fbt_housing.base_year_invalid"
    );
    assert_eq!(code(json!({"tenancy_days": -1})), "input.day_count_invalid");
}

#[test]
fn the_catalogue_names_the_methods_and_its_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_housing"];
    assert_eq!(
        entry["methods"],
        json!([
            "outside_australia",
            "accommodation_business",
            "base_year",
            "indexed"
        ])
    );
    assert_eq!(entry["source_checked"], "2026-11-15");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate("fbt-housing", &facts).unwrap();
    assert_eq!(result["amounts"]["taxable_value"], "36400.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_housing"})).unwrap()[1]["end"],
        "2027-03-31"
    );
}
