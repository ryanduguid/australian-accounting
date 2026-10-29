//! The car fringe benefit worksheets against the ATO's and the Library's worked
//! examples (docs/calculation-evidence.md#fbt-car-benefits), and their refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn statutory(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "cost_price": "32000.00", "later_accessories": "0.00", "one_third_reduction": false,
        "days_available": 274, "employee_contribution": "0.00", "electric_car_exempt": false,
        "year_ended": 2026, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("fbt-car-statutory", &input)
}

fn operating(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "running_costs": "5200.00", "lease_charges": "0.00", "depreciated_value": "50000.00",
        "days_held": 365, "business_use": "0.3636", "employee_contribution": "1500.00",
        "statutory_formula_taxable_value": "8500.00", "electric_car_exempt": false, "year_ended": 2026, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("fbt-car-operating-cost", &input)
}

fn amounts(result: Result<Value, String>) -> Value {
    result.unwrap()["amounts"].clone()
}

#[test]
fn statutory_formula_matches_the_fbt_return_2026_example() {
    // Item 23A example 6: $32,000 base value, 274 of 365 days, no contribution;
    // the ATO writes the result in whole dollars as $4,804.
    let result = statutory(json!({})).unwrap();
    assert_eq!(result["period"], "year ended 31 March 2026");
    assert_eq!(
        result["amounts"],
        json!({"base_value": "32000.00", "gross_taxable_value": "4804.38",
               "employee_contribution_applied": "0.00", "excess_employee_contribution": "0.00",
               "taxable_value_before_exemption": "4804.38", "taxable_value": "4804.38"})
    );
    assert_eq!(
        result["rates"],
        json!({"statutory_percentage": "0.20", "days_available": "274", "days_in_year": "365"})
    );
}

#[test]
fn statutory_formula_matches_the_library_car_example() {
    // ¶3-020: $50,000 for the full year less Tom's $1,500 fuel contribution.
    let a = amounts(statutory(json!({
        "cost_price": "50000.00", "days_available": 365, "employee_contribution": "1500.00",
    })));
    assert_eq!(a["gross_taxable_value"], "10000.00");
    assert_eq!(a["taxable_value"], "8500.00");
}

#[test]
fn one_third_reduction_leaves_later_accessories_whole() {
    // The employer's guide, example 26: $30,000 reduced by $10,000, plus $1,000.
    let a = amounts(statutory(json!({
        "cost_price": "30000.00", "later_accessories": "1000.00", "one_third_reduction": true,
        "days_available": 365,
    })));
    assert_eq!(a["base_value"], "21000.00");
    assert_eq!(a["gross_taxable_value"], "4200.00");
    // Example 23: $39,000 reduced by $13,000, plus $4,000 of later accessories.
    let a = amounts(statutory(json!({
        "cost_price": "39000.00", "later_accessories": "4000.00", "one_third_reduction": true,
    })));
    assert_eq!(a["base_value"], "30000.00");
    // A cost price that does not divide by 3 is reduced, then rounded to cents.
    let a = amounts(statutory(
        json!({"cost_price": "100.00", "one_third_reduction": true}),
    ));
    assert_eq!(a["base_value"], "66.67");
}

#[test]
fn the_reduced_base_value_stays_unrounded() {
    // 50000.02 x 2/3 = 33333.3466...; x 0.20 x 164 / 365 = 2995.4349..., where a
    // base rounded first to 33333.35 would give 2995.44.
    let a = amounts(statutory(json!({
        "cost_price": "50000.02", "one_third_reduction": true, "days_available": 164,
    })));
    assert_eq!(a["base_value"], "33333.35");
    assert_eq!(a["gross_taxable_value"], "2995.43");
}

#[test]
fn contributions_stop_at_nil_and_the_excess_is_shown() {
    let a = amounts(statutory(json!({"employee_contribution": "5000.00"})));
    assert_eq!(a["employee_contribution_applied"], "4804.38");
    assert_eq!(a["excess_employee_contribution"], "195.62");
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn an_exempt_electric_car_keeps_its_notional_value() {
    let a = amounts(statutory(json!({"electric_car_exempt": true})));
    assert_eq!(a["taxable_value_before_exemption"], "4804.38");
    assert_eq!(a["taxable_value"], "0.00");
    let a = amounts(operating(json!({"electric_car_exempt": true})));
    assert_eq!(a["taxable_value_before_exemption"], "8500.00");
    assert_eq!(a["taxable_value"], "0.00");
}

#[test]
fn operating_cost_matches_the_library_car_example() {
    // ¶3-020: $5,200 running costs, $12,500 deemed depreciation, $4,310 deemed
    // interest at 8.62%, 63.64% private use, less the $1,500 contribution. The
    // statutory formula's $8,500 is lower, so s 10(5) makes it the taxable value.
    let result = operating(json!({})).unwrap();
    assert_eq!(
        result["amounts"],
        json!({"deemed_depreciation": "12500.00", "deemed_interest": "4310.00",
               "operating_costs": "22010.00", "gross_taxable_value": "14007.16",
               "employee_contribution_applied": "1500.00", "excess_employee_contribution": "0.00",
               "operating_cost_taxable_value": "12507.16",
               "statutory_formula_taxable_value": "8500.00",
               "taxable_value_before_exemption": "8500.00", "taxable_value": "8500.00"})
    );
    assert_eq!(result["rates"]["benchmark_interest_rate"], "0.0862");
    assert_eq!(result["rates"]["private_use"], "0.6364");
}

#[test]
fn the_lower_of_the_two_methods_is_the_taxable_value() {
    for (statutory, taxable) in [
        ("20000.00", "12507.16"),
        ("12507.16", "12507.16"),
        ("12507.15", "12507.15"),
        ("0.00", "0.00"),
    ] {
        let a = amounts(operating(
            json!({"statutory_formula_taxable_value": statutory}),
        ));
        assert_eq!(a["operating_cost_taxable_value"], "12507.16");
        assert_eq!(a["taxable_value"], taxable, "{statutory}");
    }
}

#[test]
fn operating_cost_matches_the_fbt_return_2026_example() {
    // Item 23B example 7: $10,000 total operating costs at 30% private use.
    let a = amounts(operating(json!({
        "running_costs": "10000.00", "depreciated_value": "0.00", "business_use": "0.7",
        "employee_contribution": "0.00",
    })));
    assert_eq!(a["operating_costs"], "10000.00");
    assert_eq!(a["taxable_value"], "3000.00");
}

#[test]
fn operating_cost_follows_the_guide_example_at_each_years_rate() {
    // The employer's guide, example 31, used 5.20% for 2020-21; the same car at
    // 8.62% for 2026 and 8.27% for 2027.
    let facts = json!({
        "running_costs": "7500.00", "depreciated_value": "40000.00", "business_use": "0.75",
        "employee_contribution": "2000.00",
    });
    let a = amounts(operating(facts.clone()));
    assert_eq!(a["deemed_interest"], "3448.00");
    assert_eq!(a["operating_costs"], "20948.00");
    assert_eq!(a["taxable_value"], "3237.00");
    let mut later = facts;
    later["year_ended"] = json!(2027);
    let result = operating(later).unwrap();
    assert_eq!(result["period"], "year ended 31 March 2027");
    assert_eq!(result["amounts"]["deemed_interest"], "3308.00");
    assert_eq!(result["amounts"]["taxable_value"], "3202.00");
}

#[test]
fn deemed_amounts_are_apportioned_to_the_days_held() {
    // 40000 x 0.25 x 182 / 365 = 4986.301...; 40000 x 0.0827 x 182 / 365 = 1649.468...
    let a = amounts(operating(json!({
        "running_costs": "0.00", "depreciated_value": "40000.00", "days_held": 182,
        "business_use": "0", "employee_contribution": "0.00", "year_ended": 2027,
    })));
    assert_eq!(a["deemed_depreciation"], "4986.30");
    assert_eq!(a["deemed_interest"], "1649.47");
    assert_eq!(a["operating_costs"], "6635.77");
    assert_eq!(a["taxable_value"], "6635.77");
}

#[test]
fn deemed_amounts_stay_unrounded_until_the_gross_value() {
    // 500 + 2739.726... + 944.657... = 4184.383...; x 0.5 = 2092.19, where
    // deemed amounts rounded first would give 4184.39 and 2092.20.
    let a = amounts(operating(json!({
        "running_costs": "500.00", "depreciated_value": "40000.00", "days_held": 100,
        "business_use": "0.5", "employee_contribution": "0.00",
    })));
    assert_eq!(a["deemed_depreciation"], "2739.73");
    assert_eq!(a["deemed_interest"], "944.66");
    assert_eq!(a["operating_costs"], "4184.38");
    assert_eq!(a["gross_taxable_value"], "2092.19");
}

#[test]
fn the_gross_value_is_rounded_to_cents_before_the_contribution() {
    // 10.01 x 0.5 = 5.005, rounded half up to 5.01; the $6 contribution then
    // splits into 5.01 applied and 0.99 excess, which add back to $6.
    let a = amounts(operating(json!({
        "running_costs": "10.01", "depreciated_value": "0.00", "business_use": "0.5",
        "employee_contribution": "6.00",
    })));
    assert_eq!(a["gross_taxable_value"], "5.01");
    assert_eq!(a["employee_contribution_applied"], "5.01");
    assert_eq!(a["excess_employee_contribution"], "0.99");
}

#[test]
fn a_leased_car_uses_its_lease_charges() {
    let a = amounts(operating(json!({
        "running_costs": "3000.00", "lease_charges": "9000.00", "depreciated_value": "0.00",
        "business_use": "0.5", "employee_contribution": "0.00",
    })));
    assert_eq!(a["deemed_depreciation"], "0.00");
    assert_eq!(a["deemed_interest"], "0.00");
    assert_eq!(a["taxable_value"], "6000.00");
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let cases = [
        (
            statutory(json!({"scope_confirmed": false})),
            "Establish all stated scope conditions before calculating.",
        ),
        (
            statutory(json!({"year_ended": 2028})),
            "Unsupported period 2028; supported: 2026, 2027.",
        ),
        (
            statutory(json!({"year_ended": "2026"})),
            "Unsupported period '2026'; supported: 2026, 2027.",
        ),
        (
            statutory(json!({"days_available": 366})),
            "days_available must be an integer from 0 to 365.",
        ),
        (
            statutory(json!({"days_available": true})),
            "days_available must be an integer from 0 to 365.",
        ),
        (
            statutory(json!({"one_third_reduction": 1})),
            "one_third_reduction must be a boolean.",
        ),
        (
            statutory(json!({"cost_price": "-1.00"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            operating(json!({"lease_charges": "1.00"})),
            "A car is either owned, with a depreciated_value, or leased, with lease_charges; set the other to 0.",
        ),
        (
            operating(json!({"business_use": "1.01"})),
            "business_use must be a finite Decimal fraction from 0 to 1.",
        ),
        (
            operating(json!({"days_held": -1})),
            "days_held must be an integer from 0 to 365.",
        ),
        (
            operating(json!({"statutory_formula_taxable_value": "1.001"})),
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            operating(json!({"electric_car_exempt": "no"})),
            "electric_car_exempt must be a boolean.",
        ),
    ];
    for (result, message) in cases {
        assert_eq!(result, Err(message.to_string()));
    }
}

#[test]
fn the_catalogue_and_periods_describe_both_worksheets() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    for (kind, command) in [
        ("fbt_car_statutory", "fbt-car-statutory"),
        ("fbt_car_operating_cost", "fbt-car-operating-cost"),
    ] {
        let entry = &catalogue[kind];
        assert_eq!(entry["required_inputs"]["year_ended"], "integer");
        assert_eq!(entry["supported_periods"][1]["end"], "2027-03-31");
        assert_eq!(entry["source_checked"], "2026-10-29");
        // The catalogue's own example runs as given.
        let mut facts = entry["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        let result = api::evaluate(command, &facts).unwrap();
        assert_eq!(result["calculation"], kind);
        assert_eq!(
            api::evaluate("periods", &json!({"kind": kind})).unwrap(),
            json!([
                {"id": "2026", "basis": "FBT year", "start": "2025-04-01", "end": "2026-03-31",
                 "arguments": {"year_ended": 2026}},
                {"id": "2027", "basis": "FBT year", "start": "2026-04-01", "end": "2027-03-31",
                 "arguments": {"year_ended": 2027}},
            ])
        );
    }
    assert_eq!(
        catalogue["fbt_car_statutory"]["example"]["facts"]["days_available"],
        274
    );
}
