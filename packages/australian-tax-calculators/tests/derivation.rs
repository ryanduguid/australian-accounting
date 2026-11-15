// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `derive` explains each amount of the worksheets it covers (FBT, Division 293,
//! the pension minimum, PAYG withholding, Schedule 5 and termination payments) in a step a reader can check alone: every
//! name in a step's formula is an input, a rate or an earlier step, its operands
//! are those values, and they recompute the amount. These tests recompute every
//! amount that way, independently of the engine's arithmetic.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::{Decimal, Rounding, with_precision};
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

/// The commands `derive` explains.
const DERIVED: [&str; 21] = [
    "fbt",
    "fbt-car-statutory",
    "fbt-car-operating-cost",
    "fbt-loan",
    "fbt-meal-entertainment",
    "fbt-car-parking",
    "fbt-debt-waiver",
    "fbt-expense-payment",
    "fbt-property",
    "division-293",
    "pension-minimum",
    "payg-withholding",
    "schedule-5-method-a",
    "schedule-5-method-b-i",
    "schedule-5-method-b-ii",
    "termination-payment",
    "fbt-residual",
    "fbt-housing",
    "fbt-lafha",
    "fbt-board",
    "fbt-tax-exempt-body-entertainment",
];

/// (command, input) for each derived kind: the ATO's published examples and
/// the branches each worksheet takes.
fn cases() -> Vec<(&'static str, Value)> {
    let division_293 = |income: &str, contributions: &str| {
        (
            "division-293",
            json!({"division_293_income": income, "low_tax_contributions": contributions, "year": "2025-26", "scope_confirmed": true}),
        )
    };
    let pension = |balance: &str, age: i64, days: i64| {
        (
            "pension-minimum",
            json!({"account_balance": balance, "age": age, "days": days, "year": "2024-25", "scope_confirmed": true}),
        )
    };
    let fbt = |one: &str, two: &str, year: i64| json!({"type_one_value": one, "type_two_value": two, "year_ended": year, "scope_confirmed": true});
    let statutory = |overrides: Value| {
        let mut input = json!({
            "cost_price": "32000.00", "later_accessories": "0.00", "one_third_reduction": false,
            "days_available": 274, "employee_contribution": "0.00", "electric_car_exempt": false,
            "year_ended": 2026, "scope_confirmed": true,
        });
        merge(&mut input, overrides);
        ("fbt-car-statutory", input)
    };
    let operating = |overrides: Value| {
        let mut input = json!({
            "running_costs": "5200.00", "lease_charges": "0.00", "depreciated_value": "50000.00",
            "effective_life_years": "8", "days_held": 365, "business_use": "0.3636",
            "employee_contribution": "1500.00", "statutory_formula_taxable_value": "8500.00",
            "electric_car_exempt": false, "year_ended": 2026, "scope_confirmed": true,
        });
        merge(&mut input, overrides);
        ("fbt-car-operating-cost", input)
    };
    let mut cases = vec![
        // Calculating your FBT, MantCo, in both years.
        ("fbt", fbt("16500.00", "6000.00", 2026)),
        ("fbt", fbt("16500.00", "6000.00", 2027)),
        ("fbt", fbt("1000.00", "1000.00", 2026)),
        ("fbt", fbt("0.00", "0.00", 2027)),
        ("fbt", fbt("0.01", "999999999999.99", 2026)),
        // FBT return 2026 instructions, item 23A example 6.
        statutory(json!({})),
        // The employer's guide, example 26: a one-third reduction and an accessory.
        statutory(
            json!({"cost_price": "30000.00", "later_accessories": "1000.00",
                         "one_third_reduction": true, "days_available": 365}),
        ),
        statutory(
            json!({"cost_price": "50000.02", "one_third_reduction": true, "days_available": 164}),
        ),
        statutory(json!({"employee_contribution": "5000.00", "year_ended": 2027})),
        statutory(json!({"electric_car_exempt": true, "days_available": 0})),
        // The Library's car example, ¶3-020, where s 10(5) takes the statutory value.
        operating(json!({})),
        // FBT return 2026 instructions, item 23B example 7.
        operating(
            json!({"running_costs": "10000.00", "depreciated_value": "0.00",
                         "business_use": "0.7", "employee_contribution": "0.00"}),
        ),
        operating(
            json!({"depreciated_value": "40000.00", "days_held": 100, "running_costs": "500.00",
                         "business_use": "0.5", "employee_contribution": "0.00",
                         "statutory_formula_taxable_value": "99999.00", "year_ended": 2027}),
        ),
        operating(
            json!({"lease_charges": "9000.00", "depreciated_value": "0.00",
                         "effective_life_years": "0", "electric_car_exempt": true}),
        ),
        operating(json!({"effective_life_years": "3", "employee_contribution": "90000.00"})),
        // The ATO's Division 293 example (Jan): taxable contributions of $5,000.
        division_293("240000.00", "15000.00"),
        division_293("200000.00", "20000.00"),
        division_293("300000.00", "25000.00"),
        division_293("400000.00", "0.00"),
        // A full year, then the evidence's adaptation of the ATO's Thavi example:
        // from 1 January, 181 of 365 days.
        pension("600000.00", 67, 365),
        pension("250000.00", 66, 181),
        // Started on 1 June: cl 4 requires nothing.
        pension("250000.00", 66, 30),
        // $4,005.00 ends in an exact $5, so cl 5 rounds it up.
        pension("100125.00", 60, 365),
        // FBT return 2026 instructions, item 23C example 8, and the Library's ¶3-100.
        benefit(
            "fbt-loan",
            json!({"daily_balance_total": "7300000.00", "interest_accrued": "0.00",
                   "income_use": "0", "accrued_interest_deduction": "0.00"}),
        ),
        benefit(
            "fbt-loan",
            json!({"daily_balance_total": "36500000.00", "interest_accrued": "3450.00",
                   "income_use": "0.4", "accrued_interest_deduction": "1380.00"}),
        ),
        benefit(
            "fbt-loan",
            json!({"daily_balance_total": "18250000.00", "interest_accrued": "2000.00",
                   "income_use": "1", "accrued_interest_deduction": "0.00", "year_ended": 2027}),
        ),
        // Item 23P example 18, and a register percentage that does not terminate.
        benefit(
            "fbt-meal-entertainment",
            json!({"method": "50_50_split", "meal_entertainment_expenditure": "4000.00"}),
        ),
        benefit(
            "fbt-meal-entertainment",
            json!({"method": "12_week_register", "meal_entertainment_expenditure": "900.00",
                   "register_fringe_benefits_value": "1000.00",
                   "register_meal_entertainment_value": "3000.00", "register_first_year": 2026}),
        ),
        // The employer's guide, examples 15 and 17, and a fee at the threshold.
        benefit(
            "fbt-car-parking",
            json!({"method": "average_cost", "first_business_day_fee": "17.00", "benefits": 3,
                   "first_day_fee": "17.25", "last_day_fee": "18.50",
                   "employee_contribution": "20.00"}),
        ),
        benefit(
            "fbt-car-parking",
            json!({"method": "12_week_register", "first_business_day_fee": "20.00",
                   "register_value": "2500.00", "availability_days": 183,
                   "register_first_year": 2026}),
        ),
        benefit(
            "fbt-car-parking",
            json!({"method": "commercial_parking_station", "first_business_day_fee": "11.48",
                   "benefits": 230, "lowest_fee": "18.00", "employee_contribution": "1000.00",
                   "year_ended": 2027}),
        ),
        // FBT return 2026 instructions, item 23D example 9.
        benefit("fbt-debt-waiver", json!({"amount_waived": "500.00"})),
        // The employer's guide, 9.4's first example and 9.3's in-house petrol,
        // with the employee's $1,000 in-house reduction unused.
        benefit(
            "fbt-expense-payment",
            json!({"method": "external", "recipients_expenditure": "500.00",
                   "amount_paid": "250.00", "employee_contribution": "0.00", "income_use": "0.8",
                   "recipients_portion_deduction": "200.00"}),
        ),
        benefit(
            "fbt-expense-payment",
            json!({"method": "in_house_acquired", "recipients_expenditure": "1000.00",
                   "amount_paid": "250.00", "employee_contribution": "0.00",
                   "arms_length_price": "800.00", "notional_value": "1000.00",
                   "in_house_reduction_available": "1000.00"}),
        ),
        // Item 23K example 14, the guide's 17.5 first example, and the
        // otherwise deductible rule before the in-house reduction.
        benefit(
            "fbt-property",
            json!({"method": "in_house_public_price", "lowest_price": "3600.00",
                   "employee_contribution": "300.00", "income_use": "0",
                   "contribution_deduction": "0.00", "in_house_reduction_available": "1000.00"}),
        ),
        benefit(
            "fbt-property",
            json!({"method": "external_notional_value", "notional_value": "500.00",
                   "employee_contribution": "250.00", "income_use": "0.8",
                   "contribution_deduction": "200.00"}),
        ),
        benefit(
            "fbt-property",
            json!({"method": "in_house_acquired", "arms_length_price": "500.00",
                   "notional_value": "800.00", "employee_contribution": "0.00",
                   "income_use": "0.5", "contribution_deduction": "0.00",
                   "in_house_reduction_available": "100.00", "year_ended": 2027}),
        ),
        // The employer's guide, 18.4 and 18.7: an in-house rental with the
        // $1,000 reduction, and hired goods 80% used to produce income.
        benefit(
            "fbt-residual",
            json!({"method": "in_house_public_price", "lowest_price": "100.00",
                   "employee_contribution": "50.00", "income_use": "0",
                   "contribution_deduction": "0.00", "in_house_reduction_available": "10.00"}),
        ),
        benefit(
            "fbt-residual",
            json!({"method": "external_cost_price", "cost_price": "500.00",
                   "employee_contribution": "250.00", "income_use": "0.8",
                   "contribution_deduction": "200.00"}),
        ),
        // Guide 10.5: a base year over 274 days, then indexed for NSW, and a
        // hotel room valued as an in-house benefit.
        benefit(
            "fbt-housing",
            json!({"method": "base_year", "market_value": "11820.00", "tenancy_days": 274,
                   "employee_contribution": "0.00"}),
        ),
        benefit(
            "fbt-housing",
            json!({"method": "indexed", "prior_statutory_annual_value": "15745.62",
                   "state": "NSW", "base_year_ended": 2025, "tenancy_days": 91,
                   "employee_contribution": "100.00", "year_ended": 2027}),
        ),
        benefit(
            "fbt-housing",
            json!({"method": "accommodation_business", "market_value": "10000.01",
                   "in_house": true, "employee_contribution": "0.00"}),
        ),
        // Guide 11.6: total food costs, $260 spent, and a child's days.
        benefit(
            "fbt-lafha",
            json!({"method": "maintains_home", "allowance": "900.00",
                   "accommodation_component": "500.00", "accommodation_expenses": "500.00",
                   "food_component": "320.00", "food_expenses": "260.00",
                   "normal_food_amount": "0.00", "adult_days": 7, "child_days": 3}),
        ),
        // Board with the otherwise deductible rule; a per head share.
        benefit(
            "fbt-board",
            json!({"adult_meals": 10950, "child_meals": 5, "employee_contribution": "20.00",
                   "income_use": "0.5", "contribution_deduction": "10.00"}),
        ),
        benefit(
            "fbt-tax-exempt-body-entertainment",
            json!({"method": "per_head", "entertainment_expenditure": "100.00",
                   "employee_heads": 1, "total_heads": 3}),
        ),
    ];
    cases.extend(
        PAYG.iter()
            .map(|(_, facts)| ("payg-withholding", payg(facts))),
    );
    cases.extend(
        SCHEDULE_5
            .iter()
            .map(|(_, command, facts)| (*command, schedule_5(facts))),
    );
    cases.extend(
        ETP.iter()
            .map(|(_, facts)| ("termination-payment", etp(facts))),
    );
    cases
}

const A: &str = "schedule-5-method-a";
const B_I: &str = "schedule-5-method-b-i";
const B_II: &str = "schedule-5-method-b-ii";

/// Schedule 5 cases by name, 2026-27 and scale 2 unless they say otherwise:
/// the instrument's examples 1 to 4 and a case for each branch.
const SCHEDULE_5: [(&str, &str, &str); 14] = [
    // Example 1, Jarrod: a $900 bonus on $1,400 a week, without and with HELP.
    (
        "jarrod",
        A,
        r#"{"earnings": "1400.00", "additional_payment": "900.00", "pay_periods": 52, "pay_period": "weekly"}"#,
    ),
    (
        "jarrod_help",
        A,
        r#"{"earnings": "1400.00", "additional_payment": "900.00", "pay_periods": 52, "pay_period": "weekly", "study_loan": true}"#,
    ),
    // A commission for 4 weeks, spread over the 4 pay periods it relates to.
    (
        "commission",
        A,
        r#"{"earnings": "1400.00", "additional_payment": "400.00", "pay_periods": 4, "pay_period": "weekly"}"#,
    ),
    // The 47% limit binds.
    (
        "limit",
        A,
        r#"{"earnings": "4000.00", "additional_payment": "5200.99", "pay_periods": 52, "pay_period": "weekly", "study_loan": true}"#,
    ),
    // Example 2, Robyn: $100 back pay for each of 8 weeks, without and with HELP.
    (
        "robyn",
        B_I,
        r#"{"earnings": "1700.00", "back_payment_periods": [{"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00"}], "pay_period": "weekly"}"#,
    ),
    (
        "robyn_help",
        B_I,
        r#"{"earnings": "1700.00", "back_payment_periods": [{"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}], "pay_period": "weekly", "study_loan": true}"#,
    ),
    // More was withheld than the recalculation gives, so nil.
    (
        "b_i_nil",
        B_I,
        r#"{"earnings": "1700.00", "back_payment_periods": [{"earnings": "1600.00", "back_payment": "0.00", "withheld": "400.00"}], "pay_period": "weekly"}"#,
    ),
    // Monthly amounts ending in 33 cents gain a cent in each table, steps 3 and 6.
    (
        "b_i_monthly",
        B_I,
        r#"{"earnings": "4346.33", "back_payment_periods": [{"earnings": "4246.33", "back_payment": "100.00", "withheld": "560.00"}], "pay_period": "monthly"}"#,
    ),
    // Example 3, Laura: $2,480 of arrears averaged over 26 fortnights.
    (
        "laura",
        B_II,
        r#"{"earnings": "1800.00", "additional_payment": "2480.00", "year_to_date_earnings": "18000.00", "pay_periods_to_date": 10, "earlier_additional_payments": "0.00", "earlier_additional_withholding": "0.00", "pay_period": "fortnightly"}"#,
    ),
    // Example 4, Bradley: last year's $8,800 by B(ii), with an FS debt.
    (
        "bradley_fs",
        B_II,
        r#"{"earnings": "7000.00", "additional_payment": "8800.00", "year_to_date_earnings": "21000.00", "pay_periods_to_date": 3, "earlier_additional_payments": "0.00", "earlier_additional_withholding": "0.00", "pay_period": "monthly", "study_loan": true}"#,
    ),
    // Step 8: what earlier B(ii) payments had withheld comes off; past nil, nil.
    (
        "b_ii_earlier",
        B_II,
        r#"{"earnings": "1800.00", "additional_payment": "2480.00", "year_to_date_earnings": "18000.00", "pay_periods_to_date": 10, "earlier_additional_payments": "1000.00", "earlier_additional_withholding": "500.00", "pay_period": "fortnightly"}"#,
    ),
    (
        "b_ii_nil",
        B_II,
        r#"{"earnings": "1800.00", "additional_payment": "2480.00", "year_to_date_earnings": "18000.00", "pay_periods_to_date": 10, "earlier_additional_payments": "1000.00", "earlier_additional_withholding": "9999.00", "pay_period": "fortnightly"}"#,
    ),
    // Figures in exponent form and negative zeros, as a list item may give them.
    (
        "b_i_exponent",
        B_I,
        r#"{"earnings": "1.7E+3", "back_payment_periods": [{"earnings": "1.6E+3", "back_payment": "1E+2", "withheld": "3.31E+2"}, {"earnings": "1600.00", "back_payment": "5E+1", "withheld": "-0.00"}], "pay_period": "weekly"}"#,
    ),
    (
        "b_i_negative_zero",
        B_I,
        r#"{"earnings": "1700.00", "back_payment_periods": [{"earnings": "1600.00", "back_payment": "-0.00", "withheld": "331.00"}], "pay_period": "weekly"}"#,
    ),
];

/// A Schedule 5 input: `facts` for scale 2 in 2026-27 without a study loan
/// unless they say otherwise.
fn schedule_5(facts: &str) -> Value {
    let mut input =
        json!({"scale": 2, "study_loan": false, "year": "2026-27", "scope_confirmed": true});
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

fn schedule_5_doc(name: &str) -> Value {
    let (_, command, facts) = SCHEDULE_5.iter().find(|(n, ..)| *n == name).unwrap();
    derive(command, &schedule_5(facts))
}

/// Termination payment cases by name: Schedule 11's examples 1 to 4 and the
/// Library's ¶1-420 example as the termination payment tests hold them, and
/// each cap and rate branch. Born 1 January 1990 unless they say otherwise.
const ETP: [(&str, &str); 8] = [
    (
        "lloyd",
        r#"{"redundancy_payment": "58598.00", "date_of_birth": "1985-09-01"}"#,
    ),
    (
        "jane",
        r#"{"non_excluded_payment": "10000.00", "other_taxable_payments": "84000.00", "date_of_birth": "1976-08-01"}"#,
    ),
    (
        "chris",
        r#"{"non_excluded_payment": "120000.00", "other_taxable_payments": "100000.00"}"#,
    ),
    (
        "alec",
        r#"{"redundancy_payment": "48000.00", "completed_years_of_service": 5, "non_excluded_payment": "17678.00", "other_taxable_payments": "140000.00", "date_of_birth": "1996-09-01"}"#,
    ),
    (
        "barbra",
        r#"{"redundancy_payment": "70000.00", "completed_years_of_service": 15, "date_of_birth": "1968-01-15"}"#,
    ),
    // Preservation age (60) reached on 30 June 2027, so 17%.
    (
        "reached",
        r#"{"non_excluded_payment": "1000.00", "date_of_birth": "1967-06-30"}"#,
    ),
    // A cent over the ETP cap, and no ETP cap left for the other part.
    (
        "over_etp_cap",
        r#"{"excluded_payment": "270000.01", "non_excluded_payment": "1000.00"}"#,
    ),
    // Other payments past $180,000 leave no whole-of-income cap.
    (
        "no_whole_of_income_cap",
        r#"{"excluded_payment": "1000.00", "non_excluded_payment": "1000.00", "other_taxable_payments": "250000.00"}"#,
    ),
];

fn etp(facts: &str) -> Value {
    let mut input = json!({
        "redundancy_payment": "0.00", "completed_years_of_service": 0,
        "excluded_payment": "0.00", "non_excluded_payment": "0.00",
        "other_taxable_payments": "0.00", "date_of_birth": "1990-01-01",
        "year": "2026-27", "scope_confirmed": true,
    });
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

fn etp_doc(name: &str) -> Value {
    let (_, facts) = ETP.iter().find(|(n, _)| *n == name).unwrap();
    derive("termination-payment", &etp(facts))
}

fn working_in(doc: &Value, id: &str) -> String {
    step_of(doc, id)["working"].as_str().unwrap().to_string()
}

fn authority_in(doc: &Value, id: &str) -> String {
    step_of(doc, id)["authority"].as_str().unwrap().to_string()
}

/// Schedule 1 cases by name: its examples in the Taxation Administration
/// (Withholding Schedules) Instrument 2026 (F2026L00716), read on 15 November
/// 2026, and a case for each branch of its rules. Each is 2026-27 unless it
/// names its year.
const PAYG: [(&str, &str); 21] = [
    // General example 1: $245 less a $26 levy adjustment for 5 children.
    (
        "general_1",
        r#"{"earnings": "1333.45", "pay_period": "weekly", "scale": 2, "medicare_adjustment_children": 5}"#,
    ),
    // Weekly levy adjustment examples 1 and 2.
    (
        "levy_1",
        r#"{"earnings": "633.06", "pay_period": "weekly", "scale": 2, "medicare_adjustment_children": 0}"#,
    ),
    (
        "levy_2",
        r#"{"earnings": "1142.09", "pay_period": "weekly", "scale": 6, "medicare_adjustment_children": 3}"#,
    ),
    // The fortnightly and monthly levy adjustment examples.
    (
        "levy_fortnightly",
        r#"{"earnings": "2026.77", "pay_period": "fortnightly", "scale": 2, "medicare_adjustment_children": 1}"#,
    ),
    (
        "levy_monthly",
        r#"{"earnings": "3066.33", "pay_period": "monthly", "scale": 2, "medicare_adjustment_children": 0}"#,
    ),
    // General examples 2 and 3: tax offsets on scales 5 and 2.
    (
        "general_2",
        r#"{"earnings": "1299.30", "pay_period": "fortnightly", "scale": 5, "tax_offsets": "1645"}"#,
    ),
    (
        "general_3",
        r#"{"earnings": "5400.33", "pay_period": "monthly", "scale": 2, "tax_offsets": "1365"}"#,
    ),
    // Scale 4, resident and foreign resident.
    (
        "no_tfn",
        r#"{"earnings": "1000.99", "pay_period": "weekly", "scale": 4, "foreign_resident": false}"#,
    ),
    (
        "no_tfn_foreign",
        r#"{"earnings": "1000.99", "pay_period": "fortnightly", "scale": 4, "foreign_resident": true}"#,
    ),
    (
        "quarterly",
        r#"{"earnings": "13000.00", "pay_period": "quarterly", "scale": 1}"#,
    ),
    (
        "monthly",
        r#"{"earnings": "5000.00", "pay_period": "monthly", "scale": 3}"#,
    ),
    // Scale 2's first row, whose coefficients the instrument prints as dashes.
    (
        "nil_row",
        r#"{"earnings": "300.00", "pay_period": "weekly", "scale": 2}"#,
    ),
    // No levy adjustment below $538, nor from the shading out point.
    (
        "levy_below",
        r#"{"earnings": "500.00", "pay_period": "weekly", "scale": 2, "medicare_adjustment_children": 0}"#,
    ),
    (
        "levy_above",
        r#"{"earnings": "1200.00", "pay_period": "weekly", "scale": 2, "medicare_adjustment_children": 0}"#,
    ),
    // The 53 and 27 pay tables.
    (
        "pays_53",
        r#"{"earnings": "2600.00", "pay_period": "weekly", "scale": 2, "extra_pay_withholding": true}"#,
    ),
    (
        "pays_27_below",
        r#"{"earnings": "1699.99", "pay_period": "fortnightly", "scale": 2, "extra_pay_withholding": true}"#,
    ),
    (
        "pays_27_top",
        r#"{"earnings": "7250.00", "pay_period": "fortnightly", "scale": 1, "extra_pay_withholding": true}"#,
    ),
    // 2024-25: the coefficients of the 2024 instrument (F2024L00664).
    (
        "year_2024",
        r#"{"earnings": "1000.00", "pay_period": "weekly", "scale": 2, "year": "2024-25"}"#,
    ),
    // Every adjustment at once, under the 2025 instruments' rules.
    (
        "all_adjustments",
        r#"{"earnings": "1100.00", "pay_period": "weekly", "scale": 2, "tax_offsets": "500", "medicare_adjustment_children": 2, "extra_pay_withholding": true, "year": "2025-26"}"#,
    ),
    (
        "levy_quarterly",
        r#"{"earnings": "13000.00", "pay_period": "quarterly", "scale": 2, "medicare_adjustment_children": 0}"#,
    ),
    (
        "levy_scale_6",
        r#"{"earnings": "950.00", "pay_period": "weekly", "scale": 6, "medicare_adjustment_children": 0}"#,
    ),
];

/// A `payg-withholding` input: `facts` for 2026-27 unless they name the year.
fn payg(facts: &str) -> Value {
    let mut input = json!({"year": "2026-27", "scope_confirmed": true});
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

/// The derivation of the `PAYG` case `name`.
fn payg_doc(name: &str) -> Value {
    let (_, facts) = PAYG.iter().find(|(n, _)| *n == name).unwrap();
    derive("payg-withholding", &payg(facts))
}

/// The step `id` of `doc`.
fn step_of<'a>(doc: &'a Value, id: &str) -> &'a Value {
    let steps = doc["steps"].as_array().unwrap();
    steps
        .iter()
        .find(|s| s["id"] == id)
        .unwrap_or_else(|| panic!("no step {id}"))
}

/// An FBT benefit worksheet's input: `facts` for the year ended 31 March 2026
/// unless they say otherwise.
fn benefit(command: &'static str, facts: Value) -> (&'static str, Value) {
    let mut input = json!({"year_ended": 2026, "scope_confirmed": true});
    merge(&mut input, facts);
    (command, input)
}

fn merge(input: &mut Value, overrides: Value) {
    for (key, value) in overrides.as_object().unwrap() {
        input[key] = value.clone();
    }
}

fn derive(command: &str, input: &Value) -> Value {
    api::evaluate("derive", &json!({"command": command, "input": input})).unwrap()
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(String),
    Name(String),
    Function(String),
    Symbol(char),
}

/// A formula or its operands as tokens: numbers, names, functions (a name
/// before `(`), the operators `+`, `-`, `x` and `/`, brackets and commas. A
/// minus written against digits is a number's sign, as in the -0.00 that an
/// input of "-0.00" keeps; subtraction is written with spaces.
fn tokens(text: &str) -> Vec<Token> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let start = i;
        if c == ' ' {
            i += 1;
        } else if c.is_ascii_digit()
            || (c == '-' && chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || chars[i] == '.'
                    || chars[i] == 'E'
                    || (matches!(chars[i], '+' | '-') && chars[i - 1] == 'E'))
            {
                i += 1;
            }
            out.push(Token::Number(chars[start..i].iter().collect()));
        } else if c.is_ascii_lowercase() {
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            out.push(match word.as_str() {
                "x" => Token::Symbol('x'),
                _ if chars.get(i) == Some(&'(') => Token::Function(word),
                _ => Token::Name(word),
            });
        } else {
            assert!("+-/(),".contains(c), "unexpected {c:?} in {text:?}");
            out.push(Token::Symbol(c));
            i += 1;
        }
    }
    out
}

fn dec(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

/// Evaluates operand tokens in the worksheets' 40-digit context: `x` and `/`
/// before `+` and `-`, each left to right.
struct Evaluator<'a> {
    tokens: &'a [Token],
    at: usize,
}

impl Evaluator<'_> {
    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.at);
        self.at += 1;
        token
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn sum(&mut self) -> Decimal {
        let mut value = self.product();
        while let Some(Token::Symbol(op @ ('+' | '-'))) = self.peek().cloned() {
            self.at += 1;
            let right = self.product();
            value = if op == '+' {
                value.add(&right)
            } else {
                value.sub(&right)
            }
            .unwrap();
        }
        value
    }

    fn product(&mut self) -> Decimal {
        let mut value = self.factor();
        while let Some(Token::Symbol(op @ ('x' | '/'))) = self.peek().cloned() {
            self.at += 1;
            let right = self.factor();
            value = if op == 'x' {
                value.mul(&right)
            } else {
                value.div(&right)
            }
            .unwrap();
        }
        value
    }

    fn factor(&mut self) -> Decimal {
        match self.next().cloned() {
            Some(Token::Number(text)) => dec(&text),
            Some(Token::Symbol('(')) => {
                let value = self.sum();
                assert_eq!(self.next(), Some(&Token::Symbol(')')));
                value
            }
            Some(Token::Function(name)) => {
                assert_eq!(self.next(), Some(&Token::Symbol('(')));
                let mut args = vec![self.sum()];
                while self.peek() == Some(&Token::Symbol(',')) {
                    self.at += 1;
                    args.push(self.sum());
                }
                assert_eq!(self.next(), Some(&Token::Symbol(')')));
                match (name.as_str(), &args[..]) {
                    ("min", [a, b]) => if b.lt(a).unwrap() { b } else { a }.clone(),
                    ("max", [a, b]) => if b.gt(a).unwrap() { b } else { a }.clone(),
                    ("cents", [a]) => a.quantize(-2, Rounding::HalfUp).unwrap(),
                    ("dollars", [a]) => a.quantize(0, Rounding::HalfUp).unwrap(),
                    ("floor", [a]) => a.quantize(0, Rounding::Down).unwrap(),
                    other => panic!("unknown function {other:?}"),
                }
            }
            other => panic!("expected a value, found {other:?}"),
        }
    }
}

fn evaluate(operands: &[Token]) -> Decimal {
    with_precision(40, || {
        let mut evaluator = Evaluator {
            tokens: operands,
            at: 0,
        };
        let value = evaluator.sum();
        assert_eq!(evaluator.at, operands.len(), "trailing tokens");
        value
    })
}

/// The value a formula's name stands for: an earlier step's exact value, a
/// rate, or an input as given.
fn named(name: &str, earlier: &[(String, String)], doc: &Value) -> Option<String> {
    if let Some((_, exact)) = earlier.iter().rev().find(|(id, _)| id == name) {
        return Some(exact.clone());
    }
    if let Some(rate) = doc["result"]["rates"][name].as_str() {
        return Some(rate.to_string());
    }
    match &doc["input"][name] {
        Value::String(text) => Some(text.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// What a reader checking `doc` from its steps alone would find wrong; empty
/// when every amount follows from its step.
fn problems(doc: &Value) -> Vec<String> {
    let mut found = Vec::new();
    let amounts = doc["result"]["amounts"].as_object().unwrap();
    let steps = doc["steps"].as_array().unwrap();
    let ids: Vec<&str> = steps.iter().map(|s| s["id"].as_str().unwrap()).collect();
    let keys: Vec<&str> = amounts.keys().map(String::as_str).collect();
    if ids != keys {
        found.push(format!("steps {ids:?} do not match amounts {keys:?}"));
    }
    let mut earlier: Vec<(String, String)> = Vec::new();
    for step in steps {
        let id = step["id"].as_str().unwrap();
        if step["authority"].as_str().is_none_or(str::is_empty) {
            found.push(format!("{id}: no authority"));
        }
        let working = step["working"].as_str().unwrap();
        let parts: Vec<&str> = working.split(" = ").collect();
        let [formula, operands, exact] = parts[..] else {
            found.push(format!(
                "{id}: working {working:?} is not formula = operands = value"
            ));
            continue;
        };
        let (formula, operands) = (tokens(formula), tokens(operands));
        if formula.len() != operands.len() {
            found.push(format!(
                "{id}: operands {operands:?} do not follow {formula:?}"
            ));
            continue;
        }
        for (name, operand) in formula.iter().zip(&operands) {
            match (name, operand) {
                (Token::Name(name), Token::Number(number)) => {
                    let value = named(name, &earlier, doc);
                    // The same number, though an input may be written as 1e2
                    // where the working shows 1E+2.
                    let same = value
                        .as_deref()
                        .and_then(|value| Decimal::parse(value).ok())
                        .is_some_and(|value| value.equals(&dec(number)).unwrap());
                    if !same {
                        found.push(format!("{id}: {name} is {value:?}, not {number}"));
                    }
                }
                (same, other) if same == other => {}
                (same, other) => {
                    found.push(format!("{id}: {other:?} where the formula has {same:?}"))
                }
            }
        }
        let recomputed = evaluate(&operands).to_string();
        if recomputed != exact {
            found.push(format!("{id}: the operands give {recomputed}, not {exact}"));
        }
        let cents = dec(exact)
            .quantize(-2, Rounding::HalfUp)
            .unwrap()
            .to_string();
        if step["value"] != json!(cents) {
            found.push(format!(
                "{id}: value {} is not {exact} in cents, {cents}",
                step["value"]
            ));
        }
        if step["value"] != amounts[id] {
            found.push(format!(
                "{id}: value {} differs from the amount {}",
                step["value"], amounts[id]
            ));
        }
        earlier.push((id.to_string(), exact.to_string()));
    }
    found
}

#[test]
fn every_derived_amount_is_recomputed_from_its_working() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut cases = cases();
    for command in DERIVED {
        let mut facts = catalogue[command.replace('-', "_")]["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        cases.push((command, facts));
    }
    for (command, input) in cases {
        let doc = derive(command, &input);
        assert_eq!(doc["format"], "australian-tax-calculators/derivation/1");
        assert_eq!(doc["input"], input);
        assert_eq!(doc["calculation"], command.replace('-', "_"));
        let found = problems(&doc);
        assert!(found.is_empty(), "{command} {input}: {found:#?}");
    }
}

#[test]
fn derive_and_the_plain_command_agree_on_every_amount() {
    for (command, input) in cases() {
        let plain = api::evaluate(command, &input).unwrap();
        let doc = derive(command, &input);
        // Byte for byte: the result inside the derivation is the plain output.
        assert_eq!(
            doc["result"].to_string(),
            plain.to_string(),
            "{command} {input}"
        );
        for step in doc["steps"].as_array().unwrap() {
            assert_eq!(
                step["value"],
                plain["amounts"][step["id"].as_str().unwrap()]
            );
        }
    }
}

/// Every case the Python corpus holds for a derived command: derive returns the
/// plain command's result with a working for each amount, or its refusal.
#[test]
fn every_corpus_case_of_a_derived_command_is_explained_or_refused_alike() {
    let corpus: Vec<Value> =
        serde_json::from_str(include_str!("data/python-api.json")).expect("corpus parses");
    let mut explained = 0;
    for case in &corpus {
        let command = case["command"].as_str().unwrap();
        if !DERIVED.contains(&command) {
            continue;
        }
        let (input, label) = (&case["input"], &case["test"]);
        let derived = api::evaluate_coded("derive", &json!({"command": command, "input": input}));
        match api::evaluate_coded(command, input) {
            Ok(plain) => {
                let doc = derived.unwrap();
                assert_eq!(doc["result"].to_string(), plain.to_string(), "{label}");
                let found = problems(&doc);
                assert!(found.is_empty(), "{label} {input}: {found:#?}");
                explained += 1;
            }
            Err(plain) => assert_eq!(derived.unwrap_err(), plain, "{label}"),
        }
    }
    assert!(explained > 500, "only {explained} corpus cases explained");
}

#[test]
fn a_step_value_changed_without_its_amount_is_caught() {
    let (command, input) = &cases()[0];
    let doc = derive(command, input);
    assert!(problems(&doc).is_empty());
    let mut edited = doc.clone();
    edited["steps"][2]["value"] = json!("21452.74");
    assert_eq!(
        problems(&edited),
        [
            "fbt_estimate: value \"21452.74\" is not 21452.72700000 in cents, 21452.73",
            "fbt_estimate: value \"21452.74\" differs from the amount \"21452.73\"",
        ]
    );
    // An operand that is not the value its name stands for, and an amount
    // changed without its step, are caught the same way.
    let mut edited = doc.clone();
    let working = "return_item_15 x fbt_rate = 45644 x 0.48 = 21452.68";
    edited["steps"][6]["working"] = json!(working);
    let found = problems(&edited);
    assert_eq!(
        found[0],
        "return_item_16: fbt_rate is Some(\"0.47\"), not 0.48"
    );
    assert_eq!(
        found[1],
        "return_item_16: the operands give 21909.12, not 21452.68"
    );
    let mut edited = doc;
    edited["result"]["amounts"]["return_item_15"] = json!("45645.00");
    assert_eq!(
        problems(&edited),
        ["return_item_15: value \"45644.00\" differs from the amount \"45645.00\""]
    );
}

#[test]
fn the_ato_examples_show_their_operands() {
    // Calculating your FBT, MantCo: each gross-up rounded to the dollar for items 14A and 14B.
    let doc = derive("fbt", &cases()[1].1);
    let working = |doc: &Value, i: usize| doc["steps"][i]["working"].as_str().unwrap().to_string();
    assert_eq!(
        working(&doc, 0),
        "type_one_value x type_one = 16500.00 x 2.0802 = 34323.300000"
    );
    assert_eq!(
        doc["steps"][0]["authority"],
        "FBTAA s 5B(1B); Calculating your FBT, step 3"
    );
    assert_eq!(
        working(&doc, 3),
        "dollars(type_one_grossed_up) = dollars(34323.300000) = 34323"
    );
    assert_eq!(
        working(&doc, 6),
        "return_item_15 x fbt_rate = 45644 x 0.47 = 21452.68"
    );
    assert_eq!(doc["steps"][6]["value"], "21452.68");
    // Item 23A example 6: $32,000 for 274 of 365 days.
    let doc = derive("fbt-car-statutory", &cases()[5].1);
    assert_eq!(
        working(&doc, 1),
        "cents(base_value x statutory_percentage x days_available / days_in_year) \
         = cents(32000.00 x 0.20 x 274 / 365) = 4804.38"
    );
    assert_eq!(doc["steps"][1]["authority"], "FBTAA s 9(1)");
    // ¶3-020: s 10(5) takes the lower statutory formula value.
    let doc = derive("fbt-car-operating-cost", &cases()[10].1);
    assert_eq!(
        working(&doc, 2),
        "running_costs + lease_charges + deemed_depreciation + deemed_interest \
         = 5200.00 + 0.00 + 12500.0000 + 4310.000000 = 22010.000000"
    );
    assert_eq!(
        working(&doc, 8),
        "min(operating_cost_taxable_value, statutory_formula_taxable_value) \
         = min(12507.16, 8500.00) = 8500.00"
    );
    assert_eq!(doc["steps"][8]["authority"], "FBTAA s 10(5)");
    // Division 293, Jan: $240,000 and $15,000 give $5,000 taxed at 15%.
    let doc = derive("division-293", &cases()[15].1);
    assert_eq!(
        working(&doc, 1),
        "max(0, combined_income_and_contributions - threshold) \
         = max(0, 255000.00 - 250000) = 5000.00"
    );
    assert_eq!(
        working(&doc, 3),
        "taxable_contributions x rate = 5000.00 x 0.15 = 750.0000"
    );
    // Thavi a year on: $250,000 at 5% for 181 of 365 days, rounded to $6,200.
    let doc = derive("pension-minimum", &cases()[20].1);
    assert!(
        working(&doc, 1).starts_with(
            "account_balance_used x percentage_factor x days / days_in_year \
             = 250000.00 x 0.05 x 181 / 365 = 6198.630136"
        ),
        "{}",
        working(&doc, 1)
    );
    assert_eq!(
        [&doc["steps"][1]["value"], &doc["steps"][2]["value"]],
        ["6198.63", "6200.00"]
    );
    // A full year is cl 1(1)'s formula alone: account balance x percentage factor.
    let doc = derive("pension-minimum", &cases()[19].1);
    assert_eq!(
        working(&doc, 1),
        "account_balance_used x percentage_factor = 600000.00 x 0.05 = 30000.0000"
    );
}

/// Each branch's final amount, as its provision sets it.
#[test]
fn each_branch_gives_the_amount_its_source_sets() {
    let amount = |case: usize, id: &str| {
        let (command, input) = &cases()[case];
        derive(command, input)["result"]["amounts"][id].clone()
    };
    // s 293-20(1): $220,000 does not exceed $250,000; $325,000 exceeds it by
    // $75,000, so the lesser is the $25,000 of contributions, taxed at 15%.
    assert_eq!(amount(16, "division_293_tax"), "0.00");
    assert_eq!(amount(17, "division_293_tax"), "3750.00");
    // s 293-20(2): nil low tax contributions, so no taxable contributions.
    assert_eq!(amount(18, "taxable_contributions"), "0.00");
    // cl 4: started on 1 June, so nothing is required.
    assert_eq!(amount(21, "minimum_payment"), "0.00");
    // cl 5: $4,005 ends in an exact $5, so it rounds up to $4,010.
    assert_eq!(amount(22, "minimum_payment"), "4010.00");
}

#[test]
fn each_step_cites_the_provision_its_branch_applies() {
    let authority = |case: usize, id: &str| {
        let (command, input) = &cases()[case];
        let doc = derive(command, input);
        let steps = doc["steps"].as_array().unwrap();
        let step = steps.iter().find(|s| s["id"] == id).unwrap();
        step["authority"].as_str().unwrap().to_string()
    };
    // s 8A exempts an electric car; a car it does not exempt keeps its s 9 or
    // s 10 value.
    assert_eq!(authority(5, "taxable_value"), "FBTAA s 9(1)");
    assert_eq!(authority(9, "taxable_value"), "FBTAA s 8A");
    assert_eq!(authority(10, "taxable_value"), "FBTAA s 10(2) and (5)");
    assert_eq!(authority(13, "taxable_value"), "FBTAA s 8A");
    // s 9(1) gives the statutory formula value that s 10(5) compares.
    assert_eq!(
        authority(10, "statutory_formula_taxable_value"),
        "FBTAA s 9(1) and s 10(5)"
    );
    // s 10(3)(a)(iii) deems depreciation and interest for an owned car only.
    let owned = "FBTAA s 11(1), (1AA) and (1A)";
    let leased = "FBTAA s 10(3)(a)(iii): owned cars only, so nil for a leased car";
    assert_eq!(authority(10, "deemed_depreciation"), owned);
    // An owned car with a nil depreciated value is still an owned car.
    assert_eq!(authority(11, "deemed_depreciation"), owned);
    assert_eq!(authority(13, "deemed_depreciation"), leased);
    assert_eq!(authority(13, "deemed_interest"), leased);
    // SISR Schedule 7 cl 3 pro-rates only a part year, and cl 4 removes the
    // minimum for a pension started in the year's last 30 days.
    let before_rounding = "minimum_before_rounding";
    assert_eq!(authority(19, before_rounding), "SISR Schedule 7 cl 1(1)");
    assert_eq!(
        authority(20, before_rounding),
        "SISR Schedule 7 cl 1(1) and cl 3"
    );
    assert_eq!(authority(21, before_rounding), "SISR Schedule 7 cl 4");
    assert_eq!(
        authority(15, "division_293_tax"),
        "Superannuation (Sustaining the Superannuation Contribution Concession) Imposition Act 2013 s 5"
    );
}

/// Schedule 1's published examples, worked as the instrument works them.
#[test]
fn the_schedule_1_examples_show_their_operands() {
    let working = |name: &str, id: &str| {
        step_of(&payg_doc(name), id)["working"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let amount = |name: &str, id: &str| payg_doc(name)["result"]["amounts"][id].clone();
    // General example 1: x = 1,333.99; y = 0.3200 x 1,333.99 - 181.7319, $245;
    // formula (3) on WFT $1,325.54 gives $26; $245 - $26 = $219.
    assert_eq!(
        working("general_1", "weekly_earnings_used"),
        "floor(earnings) + 0.99 = floor(1333.45) + 0.99 = 1333.99"
    );
    assert_eq!(
        working("general_1", "weekly_withholding"),
        "max(0, dollars(a x weekly_earnings_used - b)) \
         = max(0, dollars(0.3200 x 1333.99 - 181.7319)) = 245"
    );
    assert_eq!(
        working("general_1", "weekly_medicare_levy_adjustment"),
        "dollars(weekly_family_threshold x 0.0200 - (weekly_earnings_used - weekly_family_threshold) x 0.0800) \
         = dollars(1325.54 x 0.0200 - (1333.99 - 1325.54) x 0.0800) = 26"
    );
    assert_eq!(
        working("general_1", "withholding"),
        "withholding_before_adjustments - medicare_levy_adjustment = 245 - 26 = 219"
    );
    // Weekly levy adjustment examples 1 and 2: formula (1), $10; formula (2), $11.
    assert_eq!(
        working("levy_1", "weekly_medicare_levy_adjustment"),
        "dollars((weekly_earnings_used - 538.67) x 0.1) = dollars((633.99 - 538.67) x 0.1) = 10"
    );
    assert_eq!(
        working("levy_2", "weekly_medicare_levy_adjustment"),
        "dollars(weekly_earnings_used x 0.0100) = dollars(1142.99 x 0.0100) = 11"
    );
    // The fortnightly example: $1,013.99 against WFT $991.85 gives $18, twice.
    assert_eq!(
        working("levy_fortnightly", "weekly_earnings_used"),
        "floor(earnings / 2) + 0.99 = floor(2026.77 / 2) + 0.99 = 1013.99"
    );
    assert_eq!(
        working("levy_fortnightly", "medicare_levy_adjustment"),
        "weekly_medicare_levy_adjustment x 2 = 18 x 2 = 36"
    );
    // The monthly example: $3,066.33 ends in 33 cents, so a cent is added
    // first; x = 707.99, $14, and $14 x 13 / 3 is $61.
    assert_eq!(
        working("levy_monthly", "weekly_earnings_used"),
        "floor((earnings + 0.01) x 3 / 13) + 0.99 = floor((3066.33 + 0.01) x 3 / 13) + 0.99 = 707.99"
    );
    assert_eq!(
        working("levy_monthly", "medicare_levy_adjustment"),
        "dollars(weekly_medicare_levy_adjustment x 13 / 3) = dollars(14 x 13 / 3) = 61"
    );
    // General example 2: x = 649.99, $43, $86 a fortnight, less 3.8% of
    // $1,645, $63: $23.
    assert_eq!(
        working("general_2", "weekly_withholding"),
        "max(0, dollars(a x weekly_earnings_used - b)) \
         = max(0, dollars(0.1500 x 649.99 - 54.3462)) = 43"
    );
    assert_eq!(
        working("general_2", "tax_offset_reduction"),
        "dollars(tax_offsets x tax_offset_rate) = dollars(1645 x 0.038) = 63"
    );
    assert_eq!(
        working("general_2", "withholding"),
        "withholding_before_adjustments - tax_offset_reduction = 86 - 63 = 23"
    );
    // General example 3: $5,400.33 gains a cent; x = 1,246.99, $217, $940 a
    // month, less 8.3% of $1,365, $113: $827.
    assert_eq!(
        working("general_3", "withholding_before_adjustments"),
        "dollars(weekly_withholding x 13 / 3) = dollars(217 x 13 / 3) = 940"
    );
    assert_eq!(amount("general_3", "tax_offset_reduction"), "113.00");
    assert_eq!(amount("general_3", "withholding"), "827.00");
}

/// Each branch of Schedule 1's rules: the scale 4 rates, the pay periods, the
/// coefficient rows, the levy adjustment's ranges and the 53 and 27 pay tables.
#[test]
fn each_schedule_1_branch_gives_the_amount_its_source_sets() {
    let working = |name: &str, id: &str| {
        step_of(&payg_doc(name), id)["working"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let amount = |name: &str, id: &str| payg_doc(name)["result"]["amounts"][id].clone();
    // Scale 4: 47% for a resident, 45% for a foreign resident, cents ignored
    // in the earnings and the result.
    assert_eq!(
        working("no_tfn", "earnings_used"),
        "floor(earnings) = floor(1000.99) = 1000"
    );
    assert_eq!(
        working("no_tfn", "withholding"),
        "floor(earnings_used x tax_rate) = floor(1000 x 0.47) = 470"
    );
    assert_eq!(amount("no_tfn_foreign", "withholding"), "450.00");
    // Quarterly: a thirteenth of the earnings, and 13 weekly amounts; scale
    // 1's row below $2,246 (0.3200, 71.6508).
    assert_eq!(
        working("quarterly", "weekly_earnings_used"),
        "floor(earnings / 13) + 0.99 = floor(13000.00 / 13) + 0.99 = 1000.99"
    );
    assert_eq!(
        working("quarterly", "withholding"),
        "weekly_withholding x 13 = 249 x 13 = 3237"
    );
    // Monthly without 33 cents: no cent added; scale 3's row below $2,596.
    assert_eq!(
        working("monthly", "weekly_earnings_used"),
        "floor(earnings x 3 / 13) + 0.99 = floor(5000.00 x 3 / 13) + 0.99 = 1153.99"
    );
    assert_eq!(amount("monthly", "withholding"), "1499.00");
    // Scale 2 below $362 withholds nothing.
    assert_eq!(
        working("nil_row", "weekly_withholding"),
        "max(0, dollars(a x weekly_earnings_used - b)) = max(0, dollars(0 x 300.99 - 0)) = 0"
    );
    // The 2024 instrument's row below $1,282 is 0.3227 and 180.0385, the 2026
    // instrument's 0.3227 and 185.1935.
    assert_eq!(amount("year_2024", "withholding"), "143.00");
    // No levy adjustment below the threshold or from the shading out point
    // ($908.42 x 0.1 / 0.08, $1,135).
    assert_eq!(
        working("levy_below", "weekly_medicare_levy_adjustment"),
        "0 = 0 = 0"
    );
    assert_eq!(
        working("levy_above", "weekly_medicare_levy_adjustment"),
        "0 = 0 = 0"
    );
    assert_eq!(
        payg_doc("levy_above")["result"]["rates"]["shading_out_point"],
        "1135"
    );
    // Quarterly levy adjustment: 13 weekly adjustments.
    assert_eq!(
        working("levy_quarterly", "medicare_levy_adjustment"),
        "weekly_medicare_levy_adjustment x 13 = 11 x 13 = 143"
    );
    // Scale 6's formula (1): (x - 908.42) x 0.05.
    assert_eq!(
        working("levy_scale_6", "weekly_medicare_levy_adjustment"),
        "dollars((weekly_earnings_used - 908.42) x 0.05) = dollars((950.99 - 908.42) x 0.05) = 2"
    );
    // 53 weekly pays: $2,575 to $3,649 adds $7; 27 fortnightly pays: nothing
    // below $1,700, and $48 from $7,250.
    assert_eq!(working("pays_53", "additional_withholding"), "7 = 7 = 7");
    assert_eq!(amount("pays_53", "withholding"), "658.00");
    assert_eq!(amount("pays_27_below", "additional_withholding"), "0.00");
    assert_eq!(amount("pays_27_top", "additional_withholding"), "48.00");
    // Every adjustment: $175, less $10 of offsets and a $12 levy adjustment,
    // plus $3 for 53 pays.
    assert_eq!(
        working("all_adjustments", "withholding"),
        "withholding_before_adjustments - tax_offset_reduction - medicare_levy_adjustment \
         + additional_withholding = 175 - 10 - 12 + 3 = 156"
    );
}

#[test]
fn each_schedule_1_step_cites_the_section_its_branch_applies() {
    let authority = |name: &str, id: &str| {
        step_of(&payg_doc(name), id)["authority"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let levy = "weekly_medicare_levy_adjustment";
    assert_eq!(
        authority("general_1", "weekly_earnings_used"),
        "Schedule 1, 'Working out the weekly earnings'"
    );
    let other_periods =
        "Schedule 1, 'Calculating withholding fortnightly, monthly or quarterly amounts'";
    assert_eq!(
        authority("quarterly", "weekly_earnings_used"),
        other_periods
    );
    assert_eq!(authority("quarterly", "withholding"), other_periods);
    assert_eq!(
        authority("general_1", "weekly_withholding"),
        "Schedule 1, 'Using a formula' and 'Rounding of withholding amounts'"
    );
    assert!(authority("no_tfn", "withholding").starts_with("Schedule 1, scale 4 tax rate"));
    assert_eq!(
        authority("levy_1", levy),
        "Schedule 1, 'Weekly levy adjustment (WLA)', formula (1)"
    );
    assert_eq!(
        authority("levy_2", levy),
        "Schedule 1, 'Weekly levy adjustment (WLA)', formula (2)"
    );
    assert_eq!(
        authority("general_1", levy),
        "Schedule 1, 'Weekly levy adjustment (WLA)', formula (3)"
    );
    assert!(authority("levy_below", levy).contains("none below"));
    assert!(authority("levy_above", levy).contains("none from the shading out point"));
    assert_eq!(
        authority("levy_fortnightly", "medicare_levy_adjustment"),
        "Schedule 1, 'Fortnightly levy adjustment'"
    );
    assert_eq!(
        authority("levy_monthly", "medicare_levy_adjustment"),
        "Schedule 1, 'Monthly levy adjustment'"
    );
    assert_eq!(
        authority("levy_quarterly", "medicare_levy_adjustment"),
        "Schedule 1, 'Quarterly levy adjustment'"
    );
    assert_eq!(
        authority("general_2", "tax_offset_reduction"),
        "Schedule 1, 'Tax offsets'"
    );
    assert!(authority("pays_53", "additional_withholding").contains("53 pays"));
    assert!(authority("pays_27_top", "additional_withholding").contains("27 pays"));
}

/// Schedule 5's examples 1 to 4 in the 2026 instrument, step by step: each
/// tax table amount is Schedule 1's formula (and Schedule 8's for a study
/// loan) with the coefficients of its row written in.
#[test]
fn the_schedule_5_examples_show_their_operands() {
    // Example 1: $1,400 gives $267; $900 / 52 is $17; $1,417 gives $272;
    // $5 x 52 = $260, under $423; $527 in all.
    let jarrod = schedule_5_doc("jarrod");
    assert_eq!(
        working_in(&jarrod, "earnings_used"),
        "floor(earnings) = floor(1400.00) = 1400"
    );
    assert_eq!(
        working_in(&jarrod, "withholding_on_earnings"),
        "cents(max(0, dollars(0.3200 x (floor(earnings_used) + 0.99) - 181.7319))) \
         = cents(max(0, dollars(0.3200 x (floor(1400) + 0.99) - 181.7319))) = 267.00"
    );
    assert_eq!(
        working_in(&jarrod, "additional_payment_per_period"),
        "floor(additional_payment / pay_periods) = floor(900.00 / 52) = 17"
    );
    assert_eq!(
        working_in(&jarrod, "withholding_on_earnings_and_share"),
        "cents(max(0, dollars(0.3200 x (floor(earnings_used + additional_payment_per_period) + 0.99) - 181.7319))) \
         = cents(max(0, dollars(0.3200 x (floor(1400 + 17) + 0.99) - 181.7319))) = 272.00"
    );
    assert_eq!(
        working_in(&jarrod, "withholding_before_limit"),
        "(withholding_on_earnings_and_share - withholding_on_earnings) x pay_periods \
         = (272.00 - 267.00) x 52 = 260.00"
    );
    assert_eq!(
        working_in(&jarrod, "withholding_on_additional_payment"),
        "floor(max(0, min(withholding_before_limit, withholding_limit))) \
         = floor(max(0, min(260.00, 423.0000))) = 260"
    );
    assert_eq!(jarrod["result"]["amounts"]["withholding"], "527.00");
    // With HELP, Schedule 8's component is added in each table: $267 + $10 on
    // $1,400, and $260 + $104 on the bonus; $641 in all.
    let help = schedule_5_doc("jarrod_help");
    assert_eq!(
        working_in(&help, "withholding_on_earnings"),
        "cents(max(0, dollars(0.3200 x (floor(earnings_used) + 0.99) - 181.7319)) \
         + max(0, dollars(0.15 x (floor(earnings_used) + 0.99) - 200.5615))) \
         = cents(max(0, dollars(0.3200 x (floor(1400) + 0.99) - 181.7319)) \
         + max(0, dollars(0.15 x (floor(1400) + 0.99) - 200.5615))) = 277.00"
    );
    assert_eq!(
        help["result"]["amounts"]["withholding_before_limit"],
        "364.00"
    );
    assert_eq!(help["result"]["amounts"]["withholding"], "641.00");
    // Example 2: each of 8 weeks recalculated at $1,700, $363 less $331.
    let robyn = schedule_5_doc("robyn");
    let week =
        "cents(max(0, dollars(0.3200 x (floor(1600.00 + 100.00) + 0.99) - 181.7319))) - 331.00";
    assert_eq!(
        working_in(&robyn, "withholding_on_back_payment"),
        format!(
            "max(0, {}) = max(0, {}) = 256.00",
            [week; 8].join(" + "),
            [week; 8].join(" + ")
        )
    );
    assert_eq!(
        working_in(&robyn, "back_payment"),
        "100.00 + 100.00 + 100.00 + 100.00 + 100.00 + 100.00 + 100.00 + 100.00 \
         = 100.00 + 100.00 + 100.00 + 100.00 + 100.00 + 100.00 + 100.00 + 100.00 = 800.00"
    );
    assert_eq!(robyn["result"]["amounts"]["withholding"], "619.00");
    let help = schedule_5_doc("robyn_help");
    assert_eq!(
        [
            &help["result"]["amounts"]["withholding_on_back_payment"],
            &help["result"]["amounts"]["withholding"]
        ],
        ["376.00", "794.00"]
    );
    // Example 3: $18,000 over 10 fortnights, $1,800, gives $212; $2,480 / 26
    // is $95; $1,895 gives $242; $30 x 26 = $780 under $1,165.60; $992.
    let laura = schedule_5_doc("laura");
    assert_eq!(
        working_in(&laura, "average_total_earnings"),
        "floor(year_to_date_earnings / pay_periods_to_date) = floor(18000.00 / 10) = 1800"
    );
    assert_eq!(
        working_in(&laura, "additional_payments_per_period"),
        "floor((earlier_additional_payments + additional_payment) / pay_periods) \
         = floor((0.00 + 2480.00) / 26) = 95"
    );
    assert_eq!(
        working_in(&laura, "withholding_on_average_earnings"),
        "cents(max(0, dollars(0.3227 x (floor(average_total_earnings / 2) + 0.99) - 185.1935)) x 2) \
         = cents(max(0, dollars(0.3227 x (floor(1800 / 2) + 0.99) - 185.1935)) x 2) = 212.00"
    );
    assert_eq!(laura["result"]["amounts"]["withholding"], "992.00");
    // Example 4's B(ii) part with FS: $1,634 on $7,000, $4,104 for the year
    // under the $4,136 limit, $5,738 in all.
    let bradley = schedule_5_doc("bradley_fs");
    assert_eq!(
        working_in(&bradley, "withholding_for_the_year"),
        "(withholding_on_average_and_share - withholding_on_average_earnings) x pay_periods \
         = (1976.00 - 1634.00) x 12 = 4104.00"
    );
    assert_eq!(bradley["result"]["amounts"]["withholding"], "5738.00");
}

/// Each branch of Schedule 5's rules: the 47% limit, the pay periods a
/// commission relates to, a negative result as nil, earlier B(ii) withholding
/// and the 33 cents Schedule 1 adds to a monthly amount.
#[test]
fn each_schedule_5_branch_gives_the_amount_its_source_sets() {
    let amount = |name: &str, id: &str| schedule_5_doc(name)["result"]["amounts"][id].clone();
    // The limit: $5,200.99 x 47% = $2,444.4653 is less than $2,964, and its
    // cents are ignored.
    let limit = schedule_5_doc("limit");
    assert_eq!(
        working_in(&limit, "withholding_on_additional_payment"),
        "floor(max(0, min(withholding_before_limit, withholding_limit))) \
         = floor(max(0, min(2964.00, 2444.4653))) = 2444"
    );
    assert_eq!(
        authority_in(&limit, "withholding_limit"),
        "Schedule 5, Method A step 8 and 'Withholding limit'"
    );
    // A 4-week commission: $100 a week, $1,500 gives $299, $32 x 4 = $128.
    assert_eq!(amount("commission", "withholding_before_limit"), "128.00");
    // B(i) has no limit, and a negative total is nil.
    assert_eq!(
        working_in(&schedule_5_doc("b_i_nil"), "withholding_on_back_payment"),
        "max(0, cents(max(0, dollars(0.3200 x (floor(1600.00 + 0.00) + 0.99) - 181.7319))) - 400.00) \
         = max(0, cents(max(0, dollars(0.3200 x (floor(1600.00 + 0.00) + 0.99) - 181.7319))) - 400.00) = 0"
    );
    // B(i) monthly: $4,346.33 gains a cent before the table, in the earlier
    // period's recalculation and on the current pay, $602 each.
    let monthly = schedule_5_doc("b_i_monthly");
    assert_eq!(
        working_in(&monthly, "withholding_on_earnings"),
        "cents(dollars(max(0, dollars(0.3227 x (floor((earnings + 0.01) x 3 / 13) + 0.99) - 185.1935)) x 13 / 3)) \
         = cents(dollars(max(0, dollars(0.3227 x (floor((4346.33 + 0.01) x 3 / 13) + 0.99) - 185.1935)) x 13 / 3)) = 602.00"
    );
    assert_eq!(
        amount("b_i_monthly", "withholding_on_back_payment"),
        "42.00"
    );
    // Back payments given as 1E+2 and 5E+1, or -0.00, are written as the
    // total adds them.
    let exponent = schedule_5_doc("b_i_exponent");
    assert_eq!(
        working_in(&exponent, "back_payment"),
        "100 + 50 = 100 + 50 = 150"
    );
    assert!(
        working_in(&exponent, "withholding_on_back_payment")
            .contains("(floor(1.6E+3 + 1E+2) + 0.99)")
    );
    // $1,700 gives $363 less $331; $1,650 gives $347 less nothing; $363 on
    // $1,700 now: $742.
    assert_eq!(amount("b_i_exponent", "withholding"), "742.00");
    assert_eq!(
        working_in(&schedule_5_doc("b_i_negative_zero"), "back_payment"),
        "0.00 = 0.00 = 0.00"
    );
    // B(ii) step 8: $500 already withheld comes off before the limit; more
    // than the year's withholding leaves nil.
    let earlier = schedule_5_doc("b_ii_earlier");
    assert!(
        working_in(&earlier, "withholding_on_additional_payment").starts_with(
            "floor(max(0, min(withholding_for_the_year - earlier_additional_withholding, withholding_limit))) \
             = floor(max(0, min("
        )
    );
    assert!(working_in(&earlier, "withholding_on_additional_payment").contains(" - 500.00, "));
    assert_eq!(
        amount("b_ii_nil", "withholding_on_additional_payment"),
        "0.00"
    );
}

#[test]
fn each_schedule_5_step_cites_its_method_step() {
    let jarrod = schedule_5_doc("jarrod");
    let cited: Vec<&str> = jarrod["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["authority"].as_str().unwrap().split(';').next().unwrap())
        .collect();
    assert_eq!(
        cited,
        [
            "Schedule 5, Method A step 1",
            "Schedule 5, Method A step 2",
            "Schedule 5, Method A step 3",
            "Schedule 5, Method A steps 4 and 5",
            "Schedule 5, Method A steps 6 and 7",
            "Schedule 5, Method A step 8 and 'Withholding limit'",
            "Schedule 5, Method A step 9",
            "Schedule 5, Method A step 10",
        ]
    );
    let robyn = schedule_5_doc("robyn");
    assert!(
        authority_in(&robyn, "withholding_on_back_payment")
            .starts_with("Schedule 5, Method B(i) steps 2 to 5")
    );
    assert!(
        authority_in(&robyn, "withholding_on_earnings")
            .starts_with("Schedule 5, Method B(i) step 6")
    );
    let laura = schedule_5_doc("laura");
    assert!(
        authority_in(&laura, "withholding_on_additional_payment")
            .starts_with("Schedule 5, Method B(ii) steps 8 and 10")
    );
    assert!(
        authority_in(&laura, "withholding_on_earnings")
            .starts_with("Schedule 5, Method B(ii) step 11")
    );
    for doc in [&jarrod, &robyn, &laura] {
        let tables = doc["steps"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["working"].as_str().unwrap().starts_with("cents("));
        for table in tables {
            assert!(
                table["authority"]
                    .as_str()
                    .unwrap()
                    .contains("Schedule 1, 'Using a formula'"),
                "{table}"
            );
        }
    }
}

/// Schedule 11's examples 1 to 4 and the tax-free limit of ITAA 1997
/// s 83-170(3), step by step.
#[test]
fn the_schedule_11_examples_show_their_operands() {
    // Example 4, Alec: $13,598 + 5 x $6,801 = $47,603; $397 over it at 32%,
    // $127; $40,000 of whole-of-income cap; $17,678 at 32%, $5,657; $5,784.
    let alec = etp_doc("alec");
    assert_eq!(
        working_in(&alec, "tax_free_limit"),
        "redundancy_base_amount + redundancy_service_amount x completed_years_of_service \
         = 13598 + 6801 x 5 = 47603"
    );
    assert_eq!(
        working_in(&alec, "excluded_taxable_component"),
        "redundancy_payment - tax_free_redundancy_amount + excluded_payment \
         = 48000.00 - 47603 + 0.00 = 397.00"
    );
    assert_eq!(
        working_in(&alec, "excluded_withholding"),
        "dollars(excluded_within_cap x withholding_rate_within_cap + excluded_above_cap x withholding_rate_above_cap) \
         = dollars(397.00 x 0.32 + 0.00 x 0.47) = 127"
    );
    assert_eq!(
        working_in(&alec, "remaining_etp_cap"),
        "etp_cap - excluded_within_cap = 270000 - 397.00 = 269603.00"
    );
    assert_eq!(
        working_in(&alec, "whole_of_income_cap"),
        "max(0, whole_of_income_cap_amount - other_taxable_payments) = max(0, 180000 - 140000.00) = 40000.00"
    );
    assert_eq!(
        working_in(&alec, "non_excluded_withholding"),
        "dollars(non_excluded_within_cap x withholding_rate_within_cap + non_excluded_above_cap x withholding_rate_above_cap) \
         = dollars(17678.00 x 0.32 + 0.00 x 0.47) = 5657"
    );
    assert_eq!(alec["result"]["amounts"]["total_withholding"], "5784.00");
    // Example 3, Chris: $80,000 at 32% and $40,000 over the cap at 47%.
    assert_eq!(
        working_in(&etp_doc("chris"), "non_excluded_withholding"),
        "dollars(non_excluded_within_cap x withholding_rate_within_cap + non_excluded_above_cap x withholding_rate_above_cap) \
         = dollars(80000.00 x 0.32 + 40000.00 x 0.47) = 44400"
    );
    // Example 2, Jane: $180,000 - $84,000 = $96,000, below the ETP cap.
    assert_eq!(
        working_in(&etp_doc("jane"), "non_excluded_cap"),
        "min(remaining_etp_cap, whole_of_income_cap) = min(270000.00, 96000.00) = 96000.00"
    );
    // Example 1, Lloyd: $45,000 within the ETP cap at 32%, $14,400.
    assert_eq!(
        etp_doc("lloyd")["result"]["amounts"]["total_withholding"],
        "14400.00"
    );
}

#[test]
fn each_schedule_11_branch_gives_the_amount_and_cites_the_rate_its_source_sets() {
    // Barbra: the whole redundancy payment is within the s 83-170 limit.
    let barbra = etp_doc("barbra");
    assert_eq!(
        working_in(&barbra, "tax_free_redundancy_amount"),
        "min(redundancy_payment, tax_free_limit) = min(70000.00, 115613) = 70000.00"
    );
    assert_eq!(
        authority_in(&barbra, "tax_free_redundancy_amount"),
        "ITAA 1997 s 83-170(2)"
    );
    // Preservation age reached: Table A's 17%; under it, 32%.
    let reached = etp_doc("reached");
    assert_eq!(
        working_in(&reached, "non_excluded_withholding"),
        "dollars(non_excluded_within_cap x withholding_rate_within_cap + non_excluded_above_cap x withholding_rate_above_cap) \
         = dollars(1000.00 x 0.17 + 0.00 x 0.47) = 170"
    );
    assert!(
        authority_in(&reached, "non_excluded_withholding")
            .starts_with("Schedule 11, Table A, preservation age or over")
    );
    assert!(
        authority_in(&etp_doc("chris"), "non_excluded_withholding")
            .starts_with("Schedule 11, Table A, under preservation age")
    );
    // A cent over the ETP cap is withheld at 47%, and leaves the other part no cap.
    let over = etp_doc("over_etp_cap");
    assert_eq!(
        working_in(&over, "excluded_above_cap"),
        "excluded_taxable_component - excluded_within_cap = 270000.01 - 270000 = 0.01"
    );
    assert_eq!(
        working_in(&over, "non_excluded_cap"),
        "min(remaining_etp_cap, whole_of_income_cap) = min(0, 180000.00) = 0"
    );
    assert_eq!(
        authority_in(&over, "excluded_within_cap"),
        "ITAA 1997 s 82-10(4)(a), (6) and (7)(a)(i); Schedule 11, 'ETP caps', column 1: the ETP cap alone"
    );
    // Other payments over $180,000 leave no whole-of-income cap, which the
    // excluded part does not use.
    let none = etp_doc("no_whole_of_income_cap");
    assert_eq!(
        working_in(&none, "whole_of_income_cap"),
        "max(0, whole_of_income_cap_amount - other_taxable_payments) = max(0, 180000 - 250000.00) = 0"
    );
    assert_eq!(none["result"]["amounts"]["excluded_within_cap"], "1000.00");
    assert_eq!(
        authority_in(&none, "whole_of_income_cap"),
        "ITAA 1997 s 82-10(4)(c) and (5); Schedule 11, 'Steps to work out smallest cap', steps 1 to 3"
    );
}

#[test]
fn the_fbt_benefit_steps_cite_their_provisions() {
    let all = cases();
    let steps = |i: usize| derive(all[i].0, &all[i].1)["steps"].clone();
    // Item 23C example 8: $20,000 owed on each of 365 days at 8.62%.
    let loan = steps(23);
    assert_eq!(
        loan[0]["working"],
        "cents(daily_balance_total x statutory_interest_rate / days_in_year) \
         = cents(7300000.00 x 0.0862 / 365) = 1724.00"
    );
    let authority = loan[0]["authority"].as_str().unwrap();
    assert!(authority.starts_with("FBTAA s 18(1) and s 136(1)"));
    assert_eq!(
        loan[7]["authority"],
        "FBTAA s 19(1), TV - ND; FBT return 2026 instructions, item 23C"
    );
    assert_eq!(steps(26)[0]["authority"], "FBTAA s 37BA");
    assert_eq!(steps(27)[0]["authority"], "FBTAA s 37CB(2)");
    assert_eq!(steps(28)[0]["authority"], "FBTAA s 39DA(3)");
    assert_eq!(steps(29)[0]["authority"], "FBTAA s 39GB");
    // A first business day fee at the threshold gives no car parking benefit.
    let parking = steps(30);
    let taxable = parking.as_array().unwrap().last().unwrap();
    assert_eq!(taxable["working"], "0 = 0 = 0");
    let authority = taxable["authority"].as_str().unwrap();
    assert!(authority.starts_with("FBTAA s 39A(1)(a)(iii)"));
}

#[test]
fn the_payment_and_property_steps_cite_their_provisions() {
    let all = cases();
    let steps = |i: usize| derive(all[i].0, &all[i].1)["steps"].clone();
    assert_eq!(
        steps(31)[0]["authority"],
        "FBTAA s 15; FBT return 2026 instructions, item 23D"
    );
    // Guide 9.4: GD $400 on the $500 spent, less RD $200, takes $250 to $50.
    let external = steps(32);
    assert_eq!(external[0]["authority"], "FBTAA s 23");
    assert_eq!(
        external[4]["working"],
        "cents(recipients_expenditure x income_use) = cents(500.00 x 0.8) = 400.00"
    );
    assert_eq!(
        external[8]["working"],
        "value_after_contribution - reduction_applied = 250.00 - 200.00 = 50.00"
    );
    // Guide 9.3: s 42(1)(b)'s lesser value, less the recipient's portion, then s 62.
    let in_house = steps(33);
    assert_eq!(in_house[0]["authority"], "FBTAA s 42(1)(b) and (2)");
    assert_eq!(
        in_house[1]["working"],
        "recipients_expenditure - amount_paid + employee_contribution \
         = 1000.00 - 250.00 + 0.00 = 750.00"
    );
    assert_eq!(in_house[5]["authority"], "FBTAA s 62(1)");
    assert_eq!(in_house[6]["authority"], "FBTAA s 22A(1) and s 62(1)");
    // Item 23K example 14: 75% of $3,600, rounded to cents.
    let property = steps(34);
    assert_eq!(
        property[0]["working"],
        "cents(lowest_price x 0.75) = cents(3600.00 x 0.75) = 2700.00"
    );
    assert_eq!(property[0]["authority"], "FBTAA s 42(1)(a)(ii)");
    let last = steps(36);
    let last = last.as_array().unwrap();
    assert_eq!(
        last[last.len() - 2]["working"],
        "min(in_house_reduction_available, value_after_contribution - reduction_applied) \
         = min(100.00, 500.00 - 250.00) = 100.00"
    );
    assert_eq!(steps(35)[0]["authority"], "FBTAA s 43(c)");
}

#[test]
fn the_second_fbt_benefit_steps_cite_their_provisions() {
    let all = cases();
    let steps = |i: usize| derive(all[i].0, &all[i].1)["steps"].clone();
    let find = |steps: &Value, id: &str| {
        let found = steps.as_array().unwrap().iter().find(|s| s["id"] == id);
        found.unwrap().clone()
    };
    // Residual: 75% of the public price in-house, s 62's reduction after s 52.
    let rental = steps(37);
    assert_eq!(rental[0]["authority"], "FBTAA s 48(a) and s 49(a)");
    assert_eq!(
        rental[0]["working"],
        "cents(0.75 x lowest_price) = cents(0.75 x 100.00) = 75.00"
    );
    let reduction = find(&rental, "in_house_reduction_applied");
    assert_eq!(reduction["authority"], "FBTAA s 62(1)");
    assert_eq!(reduction["value"], "10.00");
    let taxable = find(&rental, "taxable_value");
    assert_eq!(taxable["authority"], "FBTAA s 52(1) and s 62(1)");
    assert_eq!(taxable["value"], "15.00");
    let hired = steps(38);
    assert_eq!(hired[0]["authority"], "FBTAA s 50(a) and s 51(a)");
    assert_eq!(
        find(&hired, "contribution_deduction_applied")["authority"],
        "FBTAA s 52(1)(ba), RD"
    );
    assert_eq!(find(&hired, "taxable_value")["value"], "50.00");
    // Housing: s 26(2)(a) annualises the base year; s 26(2)(b) indexes.
    // Neither statutory annual value is rounded before it is used (s 26).
    let base = steps(39);
    let working = base[0]["working"].as_str().unwrap();
    assert!(
        working.starts_with(
            "market_value x days_in_year / tenancy_days = 11820.00 x 365 / 274 = 15745.6204"
        ),
        "{working}"
    );
    assert_eq!(base[0]["authority"], "FBTAA s 26(2)(a)");
    assert_eq!(base[1]["authority"], "FBTAA s 26(1)(c)");
    let indexed = steps(40);
    let working = indexed[0]["working"].as_str().unwrap();
    assert!(
        working.starts_with(
            "prior_statutory_annual_value x indexation_factor = 15745.62 x 1.040 = 16375.4448"
        ),
        "{working}"
    );
    assert_eq!(
        find(&indexed, "taxable_value")["value"],
        "3982.65",
        "$4,082.65 less $100 rent"
    );
    assert_eq!(steps(41)[0]["authority"], "FBTAA s 26(1)(b)");
    // LAFHA: s 31H's food component; $42 a week from 12 and $21 under.
    let lafha = steps(42);
    assert_eq!(find(&lafha, "statutory_food_total")["value"], "51.00");
    assert_eq!(
        find(&lafha, "exempt_food_component")["authority"],
        "FBTAA s 31H(1) and (2)"
    );
    assert_eq!(find(&lafha, "taxable_value")["value"], "140.00");
    // Board: s 36's meal values, then s 37.
    let board = steps(43);
    assert_eq!(board[0]["authority"], "FBTAA s 36");
    assert_eq!(board[0]["value"], "21905.00");
    assert_eq!(
        find(&board, "taxable_value")["authority"],
        "FBTAA s 36 and s 37"
    );
    // Tax-exempt body entertainment: s 39, per head.
    let per_head = steps(44);
    let authority = per_head[0]["authority"].as_str().unwrap();
    assert!(authority.starts_with("FBTAA s 39; "));
    assert_eq!(per_head[0]["value"], "33.33");
}

#[test]
fn a_refused_derive_leaves_no_steps_for_the_next() {
    let refused = json!({"command": "fbt-car-statutory", "input": {"days_available": 366}});
    for (command, input) in cases() {
        assert!(api::evaluate("derive", &refused).is_err());
        assert!(
            problems(&derive(command, &input)).is_empty(),
            "{command} {input}"
        );
    }
}

#[test]
fn a_refused_input_gets_the_commands_own_refusal() {
    let refused = |command: &str, input: Value| {
        let plain = api::evaluate_coded(command, &input).unwrap_err();
        let derived = api::evaluate_coded("derive", &json!({"command": command, "input": input}));
        assert_eq!(derived.unwrap_err(), plain);
        plain.code
    };
    let mut input = cases()[0].1.clone();
    input["scope_confirmed"] = json!(false);
    assert_eq!(refused("fbt", input), "scope.not_confirmed");
    let mut input = cases()[5].1.clone();
    input["days_available"] = json!(366);
    assert_eq!(
        refused("fbt-car-statutory", input),
        "input.day_count_invalid"
    );
    let mut input = cases()[10].1.clone();
    input["lease_charges"] = json!("1.00");
    assert_eq!(
        refused("fbt-car-operating-cost", input),
        "fbt_car.cost_basis_conflict"
    );
    assert_eq!(refused("fbt", json!([])), "input.not_object");
    let mut input = cases()[15].1.clone();
    input["low_tax_contributions"] = json!("0.001");
    assert_eq!(refused("division-293", input), "input.amount_invalid");
    let mut input = cases()[19].1.clone();
    input["days"] = json!(0);
    assert_eq!(refused("pension-minimum", input), "input.day_count_invalid");
    let mut input = payg(PAYG[7].1);
    input.as_object_mut().unwrap().remove("foreign_resident");
    assert_eq!(refused("payg-withholding", input), "input.missing_field");
    let mut input = payg(PAYG[5].1);
    input["tax_offsets"] = json!("99999");
    assert_eq!(
        refused("payg-withholding", input),
        "payg.reductions_exceed_withholding"
    );
    let mut input = schedule_5(SCHEDULE_5[0].2);
    input["pay_periods"] = json!(1);
    assert_eq!(refused(A, input), "input.pay_periods_invalid");
    let mut input = schedule_5(SCHEDULE_5[4].2);
    input["back_payment_periods"] = json!([]);
    assert_eq!(refused(B_I, input), "input.back_payment_periods_invalid");
    let mut input = schedule_5(SCHEDULE_5[8].2);
    input["scale"] = json!(4);
    assert_eq!(refused(B_II, input), "input.tax_scale_unsupported");
    let mut input = etp(ETP[0].1);
    input["completed_years_of_service"] = json!(151);
    assert_eq!(
        refused("termination-payment", input),
        "input.years_of_service_invalid"
    );
}

#[test]
fn derive_refuses_what_it_does_not_explain() {
    let code = |envelope: Value| api::evaluate_coded("derive", &envelope).unwrap_err().code;
    let gst = json!({"amount": "1100", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"});
    assert_eq!(
        code(json!({"command": "gst", "input": gst})),
        "derivation.unsupported"
    );
    assert_eq!(
        code(json!({"command": "derive", "input": {}})),
        "derivation.unsupported"
    );
    assert_eq!(
        code(json!({"command": 5, "input": {}})),
        "derivation.unsupported"
    );
    assert_eq!(code(json!({"input": {}})), "input.missing_field");
    for command in DERIVED {
        assert_eq!(
            code(json!({"command": command})),
            "input.missing_field",
            "{command}"
        );
    }
    // A key derive does not take is refused, not ignored.
    let (command, input) = &cases()[0];
    let extra = json!({"command": command, "input": input, "steps": false});
    assert_eq!(code(extra), "input.field_unknown");
    let message = api::evaluate("derive", &json!({"command": "gst", "input": {}})).unwrap_err();
    assert_eq!(
        message,
        "derive explains fbt, fbt-car-statutory, fbt-car-operating-cost, fbt-loan, \
         fbt-meal-entertainment, fbt-car-parking, fbt-debt-waiver, fbt-expense-payment, \
         fbt-property, division-293, pension-minimum, payg-withholding, schedule-5-method-a, \
         schedule-5-method-b-i, schedule-5-method-b-ii, termination-payment, fbt-residual, \
         fbt-housing, fbt-lafha, fbt-board, fbt-tax-exempt-body-entertainment, not \"gst\""
    );
}

#[test]
fn the_binary_derives_and_refuses_with_a_code() {
    let run = |args: &[&str], stdin: &str| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_australian-tax-calculators"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        (
            out.status.code().unwrap(),
            String::from_utf8(out.stdout).unwrap(),
        )
    };
    let (command, input) = &cases()[5];
    let envelope = json!({"command": command, "input": input}).to_string();
    let (code, out) = run(&["derive"], &envelope);
    assert_eq!(code, 0);
    let doc: Value = serde_json::from_str(&out).unwrap();
    assert!(problems(&doc).is_empty());
    let (code, out) = run(
        &["derive", "--refusal-json"],
        r#"{"command": "gst", "input": {}}"#,
    );
    assert_eq!(code, 2);
    let refusal: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(refusal["code"], "derivation.unsupported");
    let (_, help) = run(&["--api-help"], "");
    assert!(
        help.contains("worksheet-catalogue, periods, derive"),
        "{help}"
    );
}
