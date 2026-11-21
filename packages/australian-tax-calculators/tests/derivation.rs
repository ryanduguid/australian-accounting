// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `derive` explains each amount of the worksheets it covers ([`DERIVED`]) in a
//! step a reader can check alone: every
//! name in a step's formula is an input, a rate or an earlier step, its operands
//! are those values, and they recompute the amount. These tests recompute every
//! amount that way, independently of the engine's arithmetic.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::{Decimal, Rounding, with_precision};
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

/// The commands `derive` explains.
const DERIVED: [&str; 54] = [
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
    "unused-leave-payment",
    "working-holiday-maker-withholding",
    "study-loan-withholding",
    "seniors-pensioners-withholding",
    "quarterly-sg",
    "borrowing-expenses",
    "borrowing-expenses-schedule",
    "resident-tax",
    "foreign-resident-tax",
    "low-income-tax-offset",
    "seniors-pensioners-tax-offset",
    "medicare-levy",
    "medicare-levy-surcharge",
    "division-296",
    "contribution-caps",
    "study-loan-repayment",
    "capital-gains",
    "main-residence",
    "fbt-residual",
    "fbt-housing",
    "fbt-lafha",
    "fbt-board",
    "fbt-tax-exempt-body-entertainment",
    "gst",
    "depreciation",
    "hire-purchase-schedule",
    "chattel-mortgage-schedule",
    "loan-transactions",
    "lump-sum-arrears-tax-offset",
    "income-tests",
    "business-capital-expenditure",
    "balancing-adjustment",
    "trade-in",
    "dasp-tax",
    "gst-margin-scheme",
    "etp-tax-free-component",
    "payroll-tax",
    "cgt-parcels",
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
                   "register_meal_entertainment_value": "3000.00", "register_first_year": 2026,
                   "register_years_expenditure": []}),
        ),
        // The employer's guide, examples 15 and 17, and a fee at the threshold.
        benefit(
            "fbt-car-parking",
            json!({"method": "average_cost", "first_business_day_fee": "17.00", "benefits": 3,
                   "first_day_fee": "17.25", "last_day_fee": "18.50",
                   "employee_contribution": {"capped_total": "17.875"}}),
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
                   "benefits": 230, "lowest_fee": "18.00", "employee_contribution": {"capped_total": "1000.00"},
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
        // The employer's guide, example 18 (the spaces method, with s 39FB),
        // then 2 groups and a negative zero, without s 39FB.
        benefit(
            "fbt-car-parking",
            json!({"method": "spaces", "space_groups": [{"first_business_day_fee": "12.00",
                   "daily_rate_amount": "10.00", "availability_days": 182, "spaces": 4}],
                   "employees_at_start": 3, "employees_at_end": 3,
                   "eligible_spaces_at_start": 4, "eligible_spaces_at_end": 4,
                   "employee_contribution": "2340.00"}),
        ),
        benefit(
            "fbt-car-parking",
            json!({"method": "spaces", "space_groups": [
                       {"first_business_day_fee": "12.00", "daily_rate_amount": "10",
                        "availability_days": 182, "spaces": 4},
                       {"first_business_day_fee": "12", "daily_rate_amount": "-0.00",
                        "availability_days": 365, "spaces": 1}],
                   "employees_at_start": 3, "employees_at_end": 4,
                   "eligible_spaces_at_start": 3, "eligible_spaces_at_end": 4,
                   "employee_contribution": "0.00", "year_ended": 2027}),
        ),
        // Example 17's register: 250 benefits at a $20 market value, $10 paid
        // for each.
        benefit(
            "fbt-car-parking",
            json!({"method": "market_value", "first_business_day_fee": "12.00", "benefits": 250,
                   "market_value": "20.00", "employee_contribution": {"capped_total": "2500.00"}}),
        ),
        // Guide 9.3 and 18.4: the rental's $100 public price, $50 reimbursed.
        benefit(
            "fbt-expense-payment",
            json!({"method": "in_house_residual_public_price", "recipients_expenditure": "100.00",
                   "amount_paid": "50.00", "employee_contribution": "0.00",
                   "lowest_price": "100.00", "in_house_reduction_available": "10.00"}),
        ),
        // A register first valid 2 years earlier, its next year 20% higher.
        benefit(
            "fbt-meal-entertainment",
            json!({"method": "12_week_register", "meal_entertainment_expenditure": "900.00",
                   "register_fringe_benefits_value": "1000.00",
                   "register_meal_entertainment_value": "3000.00", "register_first_year": 2024,
                   "register_years_expenditure": ["500.00", "600.00"]}),
        ),
        // The balancing adjustment's paths: a K7 gain (s 118-10(1)'s Gayle),
        // a car limit (s 40-325), a car claimed by both car expense methods
        // (s 40-370), by cents per kilometre only, and a motor cycle (s 118-5).
        (
            "balancing-adjustment",
            json!({"cost": "450.00", "termination_value": "700.00", "asset": "other",
                   "years": [{"income_year": "2025-26", "decline_in_value": "31.00", "deduction": "6.00"}],
                   "year": "2025-26", "scope_confirmed": true}),
        ),
        (
            "balancing-adjustment",
            json!({"cost": "69674.00", "termination_value": "50000.00", "asset": "car",
                   "car_limit": {"limit": "69674.00", "second_element_cost": "0.00",
                                 "cost_ignoring_car_limit": "80000.00"},
                   "years": [{"income_year": "2025-26", "decline_in_value": "10000.00",
                              "method": "log_book", "deduction": "10000.00", "days_held": 365}],
                   "year": "2025-26", "scope_confirmed": true}),
        ),
        (
            "balancing-adjustment",
            json!({"cost": "30000.00", "termination_value": "18000.00", "asset": "car",
                   "car_limit": null, "years": [
                       {"income_year": "2024-25", "decline_in_value": "7500.00",
                        "method": "cents_per_kilometre", "days_held": 365},
                       {"income_year": "2025-26", "decline_in_value": "5625.00",
                        "method": "log_book", "deduction": "3375.00", "days_held": 365},
                       {"income_year": "2026-27", "decline_in_value": "2000.00",
                        "method": "log_book", "deduction": "1200.00", "days_held": 200}],
                   "year": "2026-27", "scope_confirmed": true}),
        ),
        (
            "balancing-adjustment",
            json!({"cost": "30000.00", "termination_value": "25000.00", "asset": "car",
                   "car_limit": null, "years": [
                       {"income_year": "2025-26", "decline_in_value": "7500.00",
                        "method": "cents_per_kilometre", "days_held": 365}],
                   "year": "2025-26", "scope_confirmed": true}),
        ),
        (
            "balancing-adjustment",
            json!({"cost": "2000.00", "termination_value": "1800.00",
                   "asset": "motor_cycle_or_similar",
                   "years": [{"income_year": "2025-26", "decline_in_value": "500.00", "deduction": "0.00"}],
                   "year": "2025-26", "scope_confirmed": true}),
        ),
        // A trade-in of a car under the car limit (s 40-325) whose GST
        // adjustment is capped (GST Act s 132-5(3)), for a replacement that is
        // not a creditable acquisition and not a passenger car.
        (
            "trade-in",
            json!({"cost": "69674.00", "asset": "car",
                   "car_limit": {"limit": "69674.00", "second_element_cost": "0.00",
                                 "cost_ignoring_car_limit": "80000.00"},
                   "years": [{"income_year": "2024-25", "decline_in_value": "10000.00",
                              "method": "log_book", "deduction": "9000.00", "days_held": 365},
                             {"income_year": "2025-26", "decline_in_value": "0.00",
                              "method": "log_book", "deduction": "0.00", "days_held": 1}],
                   "trade_in_allowance": "88000.00", "trade_in_taxable": true,
                   "adjusted_input_tax_credit": "5700.60", "full_input_tax_credit": "6334.00",
                   "replacement_price": "11000.00", "replacement_asset": "other",
                   "replacement_creditable_acquisition": false,
                   "arms_length": true, "involuntary_disposal": false,
                   "year": "2025-26", "scope_confirmed": true}),
        ),
        // A former working holiday maker's payment with both elements, each
        // at 65% (DASP Tax Act s 5(3)).
        (
            "dasp-tax",
            json!({"tax_free_component": "1250.00", "taxed_element": "8400.37",
                   "untaxed_element": "300.00",
                   "working_holiday_maker_contributions": true,
                   "unclaimed_money_rollover": false,
                   "year": "2026-27", "scope_confirmed": true}),
        ),
        // Victoria's surcharges on $80 million of $150 million (the State
        // Revenue Office's example 2) and Tasmania's 2 rates on $2,400,000
        // of $3,000,000 (its guideline's example 2).
        (
            "payroll-tax",
            json!({"jurisdiction": "vic", "taxable_wages": "80000000.00",
                   "australian_wages": "150000000.00", "regional_employer": false,
                   "group_member": false, "employed_all_year": true,
                   "year": "2026-27", "scope_confirmed": true}),
        ),
        (
            "payroll-tax",
            json!({"jurisdiction": "tas", "taxable_wages": "2400000.00",
                   "australian_wages": "3000000.00",
                   "group_member": false, "employed_all_year": true,
                   "year": "2026-27", "scope_confirmed": true}),
        ),
        // A strata unit's share of land bought with some consideration unpaid
        // (GST Act ss 75-12 and 75-15), for a supply that cannot use the
        // margin scheme: the margin is shown beside the full GST.
        (
            "gst-margin-scheme",
            json!({"interest": "stratum_unit", "agreement": "none",
                   "acquisition": "other_supply_from_1_july_2000",
                   "consideration": "610000.00", "method": "consideration",
                   "recipient_associate": false,
                   "acquisition_consideration": "400000.00",
                   "unpaid_acquisition_consideration": "40000.00",
                   "subdivision_share": "600000.00", "subdivision_whole": "2400000.00",
                   "year": "2026-27", "scope_confirmed": true}),
        ),
        // Parcels of shares for a complying superannuation entity: a
        // selected parcel split 3 ways, a loss that takes all the other gain
        // and part of a discount gain, earlier losses and the third off.
        (
            "cgt-parcels",
            json!({"entity": "complying_superannuation_entity", "identification": "selected",
                   "parcels": [{"parcel": "A", "security": "S", "acquired": "2020-03-02", "units": "3",
                                "acquisition_cost": "100.00", "incidental_costs": "10.00",
                                "ownership_costs": "5.00"},
                               {"parcel": "B", "security": "S", "acquired": "2026-01-05", "units": "10",
                                "acquisition_cost": "500.00", "incidental_costs": "0.00"}],
                   "disposals": [{"security": "S", "date": "2026-03-02", "units": "4",
                                  "capital_proceeds": "1000.00", "incidental_costs": "9.99",
                                  "parcels": [{"parcel": "A", "units": "1"}, {"parcel": "B", "units": "3"}]},
                                 {"security": "S", "date": "2026-04-01", "units": "7",
                                  "capital_proceeds": "70.00", "incidental_costs": "0.00",
                                  "parcels": [{"parcel": "B", "units": "7"}]}],
                   "other_discount_capital_gains": "100.00", "other_capital_gains": "0.01",
                   "other_capital_losses": "0.00", "prior_net_capital_losses": "50.00",
                   "year": "2025-26", "scope_confirmed": true}),
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
    cases.extend(
        LEAVE
            .iter()
            .map(|facts| ("unused-leave-payment", leave(facts))),
    );
    cases
}

/// Unused leave cases beside the catalogue's Schedule 7 example (Norman): each
/// branch of the reason, the accrual parts, the $300 limit and no TFN.
const LEAVE: [&str; 5] = [
    // Greg, the ATO's invalidity example: all of it at 32%.
    r#"{"annual_leave": "2260.00", "leave_loading": "420.00", "termination_reason": "invalidity"}"#,
    // A normal termination with annual leave from before 18 August 1993.
    r#"{"annual_leave": "85000.00", "leave_loading": "0.00", "annual_leave_before_18_august_1993": "3533.00", "leave_loading_before_18_august_1993": "0.00", "leave_accrual_start": "1992-09-18"}"#,
    // A genuine redundancy with pre-16 August 1978 long service leave.
    r#"{"long_service_leave": "50000.00", "long_service_leave_before_16_august_1978": "2000.00", "termination_reason": "genuine_redundancy", "leave_accrual_start": "1976-03-01"}"#,
    // Under $300: the lesser of the marginal rate amount and 32%.
    r#"{"annual_leave": "250.00", "earnings": "3000.00"}"#,
    // No TFN, a foreign resident: 45% of the whole payment, pre-1978 part included.
    r#"{"long_service_leave": "1002.99", "long_service_leave_before_16_august_1978": "900.00", "long_service_leave_16_august_1978_to_17_august_1993": "0.00", "leave_accrual_start": "1976-03-01", "scale": 4, "foreign_resident": true}"#,
];

/// An unused leave input: `facts` over nil leave accrued from 2000, a normal
/// termination and $1,155 a week on scale 2 in 2026-27.
fn leave(facts: &str) -> Value {
    let mut input = json!({
        "annual_leave": "0.00", "leave_loading": "0.00", "long_service_leave": "0.00",
        "leave_accrual_start": "2000-01-01", "termination_reason": "normal",
        "working_holiday_maker": false,
        "earnings": "1155.00", "pay_period": "weekly", "scale": 2,
        "year": "2026-27", "scope_confirmed": true,
    });
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
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
    // Plain integers and a negative zero in the earlier payments.
    (
        "b_i_whole_dollars",
        B_I,
        r#"{"earnings": "1700", "back_payment_periods": [{"earnings": "1600", "back_payment": "100", "withheld": "331"}, {"earnings": "1600.00", "back_payment": "50", "withheld": "-0.00"}], "pay_period": "weekly"}"#,
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

/// The command and input of the [`SCHEDULE_5`] case `name`.
fn schedule_5_case(name: &str) -> (&'static str, Value) {
    let (_, command, facts) = SCHEDULE_5.iter().find(|(n, ..)| *n == name).unwrap();
    (command, schedule_5(facts))
}

fn schedule_5_doc(name: &str) -> Value {
    let (command, input) = schedule_5_case(name);
    derive(command, &input)
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
                    ("dollars_up", [a]) => {
                        let down = a.quantize(0, Rounding::Down).unwrap();
                        if down.lt(a).unwrap() {
                            down.add(&Decimal::from_int(1)).unwrap()
                        } else {
                            down
                        }
                    }
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
        Value::Object(payment) if name == "employee_contribution" => {
            payment.get("capped_total")?.as_str().map(str::to_owned)
        }
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
        // Division 296's very large component, the one amount a result shows
        // as null when negative, is null in its step too.
        let hidden = id == "very_large_balance_earnings_component"
            && step["value"].is_null()
            && dec(exact).lt(&dec("0")).unwrap();
        if step["value"] != json!(cents) && !hidden {
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
    edited["steps"][2]["value"] = json!("21452.84");
    let found = problems(&edited);
    assert_eq!(found.len(), 2);
    assert!(found[0].starts_with("fbt_estimate: value \"21452.84\" is not "));
    assert!(found[0].ends_with(" in cents, 21452.83"));
    assert_eq!(
        found[1],
        "fbt_estimate: value \"21452.84\" differs from the amount \"21452.83\""
    );
    // An operand that is not the value its name stands for, and an amount
    // changed without its step, are caught the same way.
    let mut edited = doc.clone();
    let working = "return_item_15 x fbt_rate = 45645 x 0.48 = 21453.15";
    edited["steps"][6]["working"] = json!(working);
    let found = problems(&edited);
    assert_eq!(
        found[0],
        "return_item_16: fbt_rate is Some(\"0.47\"), not 0.48"
    );
    assert_eq!(
        found[1],
        "return_item_16: the operands give 21909.60, not 21453.15"
    );
    let mut edited = doc;
    edited["result"]["amounts"]["return_item_15"] = json!("45646.00");
    assert_eq!(
        problems(&edited),
        ["return_item_15: value \"45645.00\" differs from the amount \"45646.00\""]
    );
}

#[test]
fn only_the_very_large_component_may_hide_a_negative_value() {
    let doc = |id: &str| {
        let mut doc = json!({
            "result": {"amounts": {}},
            "steps": [{"authority": "s 1", "working": "0 - 1 = 0 - 1 = -1", "value": null}],
        });
        doc["result"]["amounts"][id] = Value::Null;
        doc["steps"][0]["id"] = json!(id);
        doc
    };
    assert!(problems(&doc("very_large_balance_earnings_component")).is_empty());
    assert_eq!(
        problems(&doc("net_capital_gain")),
        ["net_capital_gain: value null is not -1 in cents, -1.00"]
    );
}

#[test]
fn the_ato_examples_show_their_operands() {
    // The statutory formula for the taxable values in the ATO's MantCo example.
    let doc = derive("fbt", &cases()[1].1);
    let working = |doc: &Value, i: usize| doc["steps"][i]["working"].as_str().unwrap().to_string();
    assert!(working(&doc, 0).starts_with(concat!(
        "type_one_value x (fbt_rate + 0.10) / ((1 - fbt_rate) x (1 + 0.10) x fbt_rate)",
        " = 16500.00 x (0.47 + 0.10) / ((1 - 0.47) x (1 + 0.10) x 0.47) = 34323.56"
    )));
    assert_eq!(
        doc["steps"][0]["authority"],
        "FBTAA s 5B(1B); Calculating your FBT, step 3"
    );
    assert_eq!(doc["steps"][0]["value"], "34323.56");
    assert!(working(&doc, 3).ends_with(" = 34324"));
    assert_eq!(
        working(&doc, 6),
        "return_item_15 x fbt_rate = 45645 x 0.47 = 21453.15"
    );
    assert_eq!(doc["steps"][6]["value"], "21453.15");
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
    // Back payments given as 100 and 50, or -0.00, are written as the
    // total adds them.
    let whole_dollars = schedule_5_doc("b_i_whole_dollars");
    assert_eq!(
        working_in(&whole_dollars, "back_payment"),
        "100 + 50 = 100 + 50 = 150"
    );
    assert!(
        working_in(&whole_dollars, "withholding_on_back_payment")
            .contains("(floor(1600 + 100) + 0.99)")
    );
    // $1,700 gives $363 less $331; $1,650 gives $347 less nothing; $363 on
    // $1,700 now: $742.
    assert_eq!(amount("b_i_whole_dollars", "withholding"), "742.00");
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
    // No input says whether the $250 was paid or reimbursed.
    assert_eq!(
        in_house[1]["authority"],
        "FBTAA s 22A(1)(b)(i) for a payment or (ii) for a reimbursement, and s 136(1), \
         recipients portion"
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
    let base = steps(39);
    assert_eq!(
        base[0]["working"],
        "cents(market_value x days_in_year / tenancy_days) \
         = cents(11820.00 x 365 / 274) = 15745.62"
    );
    assert_eq!(base[0]["authority"], "FBTAA s 26(2)(a)");
    assert_eq!(base[1]["authority"], "FBTAA s 26(1)(c)");
    let indexed = steps(40);
    assert_eq!(
        indexed[0]["working"],
        "cents(prior_statutory_annual_value x indexation_factor) \
         = cents(15745.62 x 1.040) = 16375.44"
    );
    assert_eq!(
        find(&indexed, "taxable_value")["value"],
        "3982.64",
        "$4,082.64 less $100 rent"
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
    let mut input = schedule_5_case("jarrod").1;
    input["pay_periods"] = json!(1);
    assert_eq!(refused(A, input), "input.pay_periods_invalid");
    let mut input = schedule_5_case("robyn").1;
    input["back_payment_periods"] = json!([]);
    assert_eq!(refused(B_I, input), "input.back_payment_periods_invalid");
    let mut input = schedule_5_case("laura").1;
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
    // The gross-up's gross is a fixed point of the withholding worked on it.
    let gross_up = json!({"net_pay": "862.00", "pay_period": "weekly", "scale": 2,
                          "year": "2026-27", "scope_confirmed": true});
    assert!(api::evaluate("payg-gross-up", &gross_up).is_ok());
    assert_eq!(
        code(json!({"command": "payg-gross-up", "input": gross_up})),
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
    let request = json!({"command": "payg-gross-up", "input": {}});
    assert_eq!(code(request), "derivation.unsupported");
    let request = json!({"command": "payg-gross-up", "input": {}});
    let message = api::evaluate("derive", &request).unwrap_err();
    assert_eq!(
        message,
        "derive explains fbt, fbt-car-statutory, fbt-car-operating-cost, fbt-loan, \
         fbt-meal-entertainment, fbt-car-parking, fbt-debt-waiver, fbt-expense-payment, \
         fbt-property, division-293, pension-minimum, payg-withholding, schedule-5-method-a, \
         schedule-5-method-b-i, schedule-5-method-b-ii, termination-payment, \
         unused-leave-payment, working-holiday-maker-withholding, study-loan-withholding, \
         seniors-pensioners-withholding, quarterly-sg, borrowing-expenses, borrowing-expenses-schedule, resident-tax, \
         foreign-resident-tax, low-income-tax-offset, seniors-pensioners-tax-offset, \
         medicare-levy, medicare-levy-surcharge, division-296, contribution-caps, \
         study-loan-repayment, capital-gains, main-residence, fbt-residual, fbt-housing, \
         fbt-lafha, fbt-board, fbt-tax-exempt-body-entertainment, gst, depreciation, \
         hire-purchase-schedule, chattel-mortgage-schedule, loan-transactions, \
         lump-sum-arrears-tax-offset, income-tests, business-capital-expenditure, balancing-adjustment, trade-in, dasp-tax, gst-margin-scheme, etp-tax-free-component, payroll-tax, cgt-parcels, not \"payg-gross-up\""
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
        r#"{"command": "payg-gross-up", "input": {}}"#,
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

// Working holiday maker, study loan and seniors and pensioners withholding,
// quarterly SG and borrowing expenses.

const WHM: &str = "working-holiday-maker-withholding";
const STSL: &str = "study-loan-withholding";
const SENIORS: &str = "seniors-pensioners-withholding";
const SG: &str = "quarterly-sg";
const BORROWING: &str = "borrowing-expenses";

/// Fiona and Max's income-producing share, 170,000 / 209,000, to 40 digits.
const FIONA_SHARE: &str = "0.8133971291866028708133971291866028708134";

/// (name, command, facts) for these 5 commands: the published examples of
/// Schedules 8, 9 and 15 of the Withholding Schedules Instrument 2026
/// (F2026L00716) and of (No. 2) 2025 (F2025L01016), the ATO's maximum quarterly
/// SG payment and 'Borrowing expenses' examples, all read on 15 November 2026,
/// and a case for each branch. `facts` change [`withholding_sg_borrowing`]'s
/// defaults.
const WITHHOLDING_SG_BORROWING: [(&str, &str, &str); 30] = [
    // Schedule 15's lookup tool example and examples 1 and 2: $102, $424, $2,452.
    ("whm_lookup_tool", WHM, "{}"),
    (
        "whm_example_1",
        WHM,
        r#"{"payment": "2825.75", "pay_period": "monthly"}"#,
    ),
    (
        "whm_example_2",
        WHM,
        r#"{"payment": "8173.20", "payments_before": "47000.00", "pay_period": "monthly"}"#,
    ),
    // Column 5, without a TFN, and an unregistered employer with and without one.
    (
        "whm_top_rate",
        WHM,
        r#"{"payment": "1000.99", "payments_before": "190000.01"}"#,
    ),
    (
        "whm_no_tfn",
        WHM,
        r#"{"payment": "1000.99", "tfn_quoted": false}"#,
    ),
    (
        "whm_unregistered_no_tfn",
        WHM,
        r#"{"payment": "1000.99", "tfn_quoted": false, "employer_registered": false}"#,
    ),
    ("whm_unregistered", WHM, r#"{"employer_registered": false}"#),
    (
        "whm_unregistered_monthly",
        WHM,
        r#"{"payment": "2825.75", "pay_period": "monthly", "year": "2025-26", "employer_registered": false}"#,
    ),
    (
        "whm_unregistered_33_cents",
        WHM,
        r#"{"payment": "3066.33", "pay_period": "monthly", "employer_registered": false}"#,
    ),
    // Schedule 8's examples 1 to 3, F2025L01016's example 1, a quarterly pay,
    // the nil row and a monthly amount ending in 33 cents.
    ("stsl_example_1", STSL, "{}"),
    (
        "stsl_example_2",
        STSL,
        r#"{"earnings": "4409.75", "pay_period": "fortnightly"}"#,
    ),
    (
        "stsl_example_3",
        STSL,
        r#"{"earnings": "10627.88", "pay_period": "monthly", "scale": 1}"#,
    ),
    (
        "stsl_2025_26",
        STSL,
        r#"{"year": "2025-26", "pay_date": "2025-09-24"}"#,
    ),
    (
        "stsl_quarterly",
        STSL,
        r#"{"earnings": "33908.68", "pay_period": "quarterly"}"#,
    ),
    (
        "stsl_nil",
        STSL,
        r#"{"earnings": "1287.99", "year": "2025-26", "pay_date": "2025-09-24"}"#,
    ),
    (
        "stsl_33_cents",
        STSL,
        r#"{"earnings": "6000.33", "pay_period": "monthly", "scale": 5}"#,
    ),
    // Schedule 9's example, single: $2.00; the pay periods, the nil row and a
    // half exemption under the 2024 instrument.
    ("seniors_example", SENIORS, "{}"),
    (
        "seniors_fortnightly",
        SENIORS,
        r#"{"earnings": "1325.90", "pay_period": "fortnightly"}"#,
    ),
    (
        "seniors_33_cents",
        SENIORS,
        r#"{"earnings": "2877.33", "pay_period": "monthly"}"#,
    ),
    ("seniors_nil", SENIORS, r#"{"earnings": "647.99"}"#),
    (
        "seniors_half_couple",
        SENIORS,
        r#"{"earnings": "4500.00", "pay_period": "monthly", "status": "couple", "medicare_levy_exemption": "half", "year": "2024-25"}"#,
    ),
    // A study loan on a Schedule 9 pay: Schedule 8's example 1, and its
    // example 3's earnings with the tax-free threshold under F2025L01016.
    (
        "seniors_study_loan",
        SENIORS,
        r#"{"earnings": "2608.36", "study_loan": true}"#,
    ),
    (
        "seniors_study_loan_monthly",
        SENIORS,
        r#"{"earnings": "10627.88", "pay_period": "monthly", "year": "2025-26", "study_loan": true, "pay_date": "2025-09-24"}"#,
    ),
    // Earnings over the $62,500 base, whose 12% is the ATO's $7,500.00
    // maximum; and contributions over the minimum.
    ("sg_capped", SG, "{}"),
    (
        "sg_overpaid",
        SG,
        r#"{"ordinary_time_earnings": "10000.00", "qualifying_contributions": "1500.00"}"#,
    ),
    // Peter's first year ($318.07), Years 5 and 6, Fiona and Max's Year 6, a
    // loan repaid early and $100 or less.
    ("peter_year_1", BORROWING, "{}"),
    (
        "peter_year_5",
        BORROWING,
        r#"{"loan_start": "2020-07-03", "loan_term_end": "2045-07-02"}"#,
    ),
    (
        "fiona_year_6",
        BORROWING,
        r#"{"borrowing_expenses": "1670.00", "income_use": "0.8133971291866028708133971291866028708134", "loan_start": "2020-07-17", "loan_term_end": "2040-07-16", "year": "2025-26"}"#,
    ),
    (
        "repaid_early",
        BORROWING,
        r#"{"repaid_on": "2025-12-31", "year": "2025-26"}"#,
    ),
    (
        "not_spread",
        BORROWING,
        r#"{"borrowing_expenses": "90.00"}"#,
    ),
];

/// The input of a [`WITHHOLDING_SG_BORROWING`] case: its command's defaults
/// changed by `facts`.
fn withholding_sg_borrowing(command: &str, facts: &str) -> Value {
    let mut input = match command {
        WHM => json!({"payment": "680.70", "payments_before": "0.00",
                      "employer_registered": true, "tfn_quoted": true,
                      "pay_period": "weekly", "year": "2026-27"}),
        STSL => json!({"earnings": "2608.36", "pay_period": "weekly", "scale": 2,
                       "year": "2026-27"}),
        SENIORS => json!({"earnings": "662.95", "pay_period": "weekly", "status": "single",
                          "medicare_levy_exemption": "none", "year": "2026-27"}),
        SG => json!({"ordinary_time_earnings": "80000.00",
                     "qualifying_contributions": "2000.00", "year": "2025-26", "quarter": 1}),
        _ => json!({"borrowing_expenses": "1600.00", "other_borrowing_expenses_in_year": "0.00",
                    "income_use": "1", "loan_start": "2024-07-03", "loan_term_end": null,
                    "repaid_on": null, "section_26_50_applies": false, "year": "2024-25"}),
    };
    input["scope_confirmed"] = json!(true);
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

/// The derivation of the [`WITHHOLDING_SG_BORROWING`] case `name`.
fn case_doc(name: &str) -> Value {
    let (_, command, facts) = WITHHOLDING_SG_BORROWING
        .iter()
        .find(|(n, ..)| *n == name)
        .unwrap();
    derive(command, &withholding_sg_borrowing(command, facts))
}

#[test]
fn each_withholding_sg_and_borrowing_case_derives_its_plain_result() {
    for (name, command, facts) in WITHHOLDING_SG_BORROWING {
        let input = withholding_sg_borrowing(command, facts);
        let plain = api::evaluate(command, &input).unwrap();
        let doc = derive(command, &input);
        assert_eq!(doc["result"].to_string(), plain.to_string(), "{name}");
        let found = problems(&doc);
        assert!(found.is_empty(), "{name}: {found:#?}");
    }
}

/// Schedules 15, 9 and 8, worked as each schedule works its examples.
#[test]
fn the_schedule_8_9_and_15_examples_show_their_operands() {
    let working = |name: &str, id: &str| working_in(&case_doc(name), id);
    // Schedule 15: y = a x on the payment with cents ignored.
    assert_eq!(
        working("whm_lookup_tool", "payment_used"),
        "floor(payment) = floor(680.70) = 680"
    );
    assert_eq!(
        working("whm_lookup_tool", "withholding"),
        "dollars(rate x payment_used) = dollars(0.15 x 680) = 102"
    );
    assert_eq!(
        working("whm_example_1", "withholding"),
        "dollars(rate x payment_used) = dollars(0.15 x 2825) = 424"
    );
    assert_eq!(
        working("whm_example_2", "withholding"),
        "dollars(rate x payment_used) = dollars(0.30 x 8173) = 2452"
    );
    assert_eq!(
        working("whm_top_rate", "withholding"),
        "floor(rate x payment_used) = floor(0.45 x 1000) = 450"
    );
    // An unregistered employer: Schedule 1 scale 3, 0.3000 x 680.99 - 0.3000
    // = 203.997, $204; monthly, $2,825.75 x 3 / 13 = 652.10, x = 652.99,
    // $196, and $196 x 13 / 3 = $849.33, $849.
    assert_eq!(
        working("whm_unregistered", "payment_used"),
        "payment = 680.70 = 680.70"
    );
    assert_eq!(
        working("whm_unregistered", "withholding"),
        "max(0, dollars(0.3000 x (floor(payment_used) + 0.99) - 0.3000)) \
         = max(0, dollars(0.3000 x (floor(680.70) + 0.99) - 0.3000)) = 204"
    );
    assert_eq!(
        working("whm_unregistered_monthly", "withholding"),
        "dollars(max(0, dollars(0.3000 x (floor(payment_used x 3 / 13) + 0.99) - 0.3000)) x 13 / 3) \
         = dollars(max(0, dollars(0.3000 x (floor(2825.75 x 3 / 13) + 0.99) - 0.3000)) x 13 / 3) = 849"
    );
    assert!(
        working("whm_unregistered_33_cents", "withholding")
            .contains("(floor((payment_used + 0.01) x 3 / 13) + 0.99)")
    );
    // Schedule 9's example: x = 662.99, 0.1500 x 662.99 - 97.2308 = 2.2177, $2.
    assert_eq!(
        working("seniors_example", "weekly_earnings_used"),
        "floor(earnings) + 0.99 = floor(662.95) + 0.99 = 662.99"
    );
    assert_eq!(
        working("seniors_example", "weekly_withholding"),
        "max(0, dollars(a x weekly_earnings_used - b)) \
         = max(0, dollars(0.1500 x 662.99 - 97.2308)) = 2"
    );
    assert_eq!(
        working("seniors_example", "withholding"),
        "weekly_withholding = 2 = 2"
    );
    assert_eq!(
        working("seniors_fortnightly", "weekly_earnings_used"),
        "floor(earnings / 2) + 0.99 = floor(1325.90 / 2) + 0.99 = 662.99"
    );
    assert_eq!(
        working("seniors_fortnightly", "withholding"),
        "weekly_withholding x 2 = 2 x 2 = 4"
    );
    // $2,877.33 gains a cent: x = 664.99, $3 a week, $13 a month.
    assert_eq!(
        working("seniors_33_cents", "weekly_earnings_used"),
        "floor((earnings + 0.01) x 3 / 13) + 0.99 = floor((2877.33 + 0.01) x 3 / 13) + 0.99 = 664.99"
    );
    assert_eq!(
        working("seniors_33_cents", "withholding"),
        "dollars(weekly_withholding x 13 / 3) = dollars(3 x 13 / 3) = 13"
    );
    assert_eq!(
        working("seniors_nil", "weekly_withholding"),
        "max(0, dollars(a x weekly_earnings_used - b)) = max(0, dollars(0 x 647.99 - 0)) = 0"
    );
    // With a study loan, Schedule 9's amount, Schedule 8's component on the
    // same weekly earnings and their total: x = 2,608.99, 0.3900 x 2,608.99 -
    // 363.4627 = 654.0434, $654, and 0.17 x 2,608.99 - 250.4527 = 193.0756,
    // $193, as Schedule 8's example 1, so $847.
    assert_eq!(
        working("seniors_study_loan", "schedule_9_withholding"),
        "weekly_withholding = 654 = 654"
    );
    assert_eq!(
        working("seniors_study_loan", "study_loan_component"),
        "max(0, dollars(component_a x weekly_earnings_used - component_b)) \
         = max(0, dollars(0.17 x 2608.99 - 250.4527)) = 193"
    );
    assert_eq!(
        working("seniors_study_loan", "withholding"),
        "schedule_9_withholding + study_loan_component = 654 + 193 = 847"
    );
    // $10,627.88 a month in 2025-26: x = 2,452.99; 0.3200 x 2,452.99 -
    // 176.5769 = 608.3799, $608 a week, x 13 / 3 = 2,634.67, $2,635; the
    // component 0.17 x 2,452.99 - 241.3462 = 175.6621, $176 a week, x 13 / 3
    // = 762.67, $763; $3,398 in all.
    assert_eq!(
        working("seniors_study_loan_monthly", "schedule_9_withholding"),
        "dollars(weekly_withholding x 13 / 3) = dollars(608 x 13 / 3) = 2635"
    );
    assert_eq!(
        working("seniors_study_loan_monthly", "study_loan_component"),
        "dollars(max(0, dollars(component_a x weekly_earnings_used - component_b)) x 13 / 3) \
         = dollars(max(0, dollars(0.17 x 2452.99 - 241.3462)) x 13 / 3) = 763"
    );
    assert_eq!(
        working("seniors_study_loan_monthly", "withholding"),
        "schedule_9_withholding + study_loan_component = 2635 + 763 = 3398"
    );
    // Schedule 8's examples 1 to 3: $193, $130 x 2 = $260, $226 x 13 / 3 = $979;
    // example 1's Schedule 1 amount, 0.3900 x 2,608.99 - 363.4627, is $654.
    assert_eq!(
        working("stsl_example_1", "study_loan_component"),
        "max(0, dollars(component_a x weekly_earnings_used - component_b)) \
         = max(0, dollars(0.17 x 2608.99 - 250.4527)) = 193"
    );
    assert_eq!(
        working("stsl_example_1", "schedule_1_withholding"),
        "max(0, dollars(a x weekly_earnings_used - b)) \
         = max(0, dollars(0.3900 x 2608.99 - 363.4627)) = 654"
    );
    assert_eq!(
        working("stsl_example_1", "withholding"),
        "schedule_1_withholding + study_loan_component = 654 + 193 = 847"
    );
    assert_eq!(
        working("stsl_example_2", "study_loan_component"),
        "max(0, dollars(component_a x weekly_earnings_used - component_b)) x 2 \
         = max(0, dollars(0.15 x 2204.99 - 200.5615)) x 2 = 260"
    );
    assert_eq!(
        working("stsl_example_3", "weekly_earnings_used"),
        "floor(earnings x 3 / 13) + 0.99 = floor(10627.88 x 3 / 13) + 0.99 = 2452.99"
    );
    assert_eq!(
        working("stsl_example_3", "study_loan_component"),
        "dollars(max(0, dollars(component_a x weekly_earnings_used - component_b)) x 13 / 3) \
         = dollars(max(0, dollars(0.17 x 2452.99 - 190.9527)) x 13 / 3) = 979"
    );
    // F2025L01016's example 1: 0.17 x 2,608.99 - 241.3462, $202.
    assert_eq!(
        case_doc("stsl_2025_26")["result"]["amounts"]["study_loan_component"],
        "202.00"
    );
    // A quarter of 13 weeks of example 1: $654 x 13 = $8,502, and $193 x 13.
    assert_eq!(
        working("stsl_quarterly", "schedule_1_withholding"),
        "max(0, dollars(a x weekly_earnings_used - b)) x 13 \
         = max(0, dollars(0.3900 x 2608.99 - 363.4627)) x 13 = 8502"
    );
    assert_eq!(
        case_doc("stsl_quarterly")["result"]["amounts"]["withholding"],
        "11011.00"
    );
}

/// Quarterly SG and borrowing expenses, worked as the ATO's pages work them.
#[test]
fn the_sg_and_borrowing_examples_show_their_operands() {
    let working = |name: &str, id: &str| working_in(&case_doc(name), id);
    assert_eq!(
        working("sg_capped", "earnings_used"),
        "min(ordinary_time_earnings, maximum_quarterly_base) = min(80000.00, 62500) = 62500"
    );
    assert_eq!(
        working("sg_capped", "minimum_sg"),
        "earnings_used x sg_rate = 62500 x 0.12 = 7500.00"
    );
    assert_eq!(
        working("sg_capped", "additional_contribution"),
        "max(0, minimum_sg - qualifying_contributions) = max(0, 7500.00 - 2000.00) = 5500.00"
    );
    assert_eq!(
        working("sg_overpaid", "additional_contribution"),
        "max(0, minimum_sg - qualifying_contributions) = max(0, 1200.0000 - 1500.00) = 0"
    );
    // Peter's first year: $1,600 x 363 / 1,826, $318.07, the ATO's figure.
    assert_eq!(
        working("peter_year_1", "remaining_expenditure"),
        "borrowing_expenses = 1600.00 = 1600.00"
    );
    assert_eq!(
        working("peter_year_1", "maximum_deduction"),
        "cents(remaining_expenditure x days_in_year / remaining_loan_period_days) \
         = cents(1600.00 x 363 / 1826) = 318.07"
    );
    // Year 5: each earlier year's maximum, as tests/borrowing_expenses.rs
    // works them, then $321.58 x 365 / 367, $319.83, leaving the ATO's $1.75.
    let earlier = "cents(1600.00 x 363 / 1826) - cents(1281.93 x 365 / 1463) \
                   - cents(962.10 x 365 / 1098) - cents(642.28 x 366 / 733)";
    assert_eq!(
        working("peter_year_5", "remaining_expenditure"),
        format!("borrowing_expenses - {earlier} = 1600.00 - {earlier} = 321.58")
    );
    assert_eq!(
        working("peter_year_5", "maximum_deduction"),
        "cents(remaining_expenditure x days_in_year / remaining_loan_period_days) \
         = cents(321.58 x 365 / 367) = 319.83"
    );
    assert_eq!(
        working("peter_year_5", "remaining_expenditure_after_year"),
        "remaining_expenditure - maximum_deduction = 321.58 - 319.83 = 1.75"
    );
    // Fiona and Max's Year 6: $14.63 x their share, $11.90, the ATO's figure.
    assert_eq!(
        working("fiona_year_6", "deduction"),
        format!("cents(maximum_deduction x income_use) = cents(14.63 x {FIONA_SHARE}) = 11.90")
    );
    assert_eq!(
        working("not_spread", "deduction"),
        "cents(maximum_deduction x income_use) = cents(90.00 x 1) = 90.00"
    );
    assert_eq!(
        working("not_spread", "remaining_expenditure_after_year"),
        "0 = 0 = 0"
    );
}

#[test]
fn each_withholding_sg_and_borrowing_step_cites_its_branch() {
    let authority = |name: &str, id: &str| authority_in(&case_doc(name), id);
    let cites = |name: &str, id: &str, start: &str| {
        let cited = authority(name, id);
        assert!(cited.starts_with(start), "{name} {id}: {cited}");
    };
    cites(
        "whm_lookup_tool",
        "withholding",
        "Schedule 15, 'Registered employer', 'Using a formula' and Table A",
    );
    assert!(authority("whm_lookup_tool", "withholding").ends_with("50 cents up"));
    assert!(authority("whm_top_rate", "withholding").ends_with("cents in the result are ignored"));
    cites(
        "whm_no_tfn",
        "withholding",
        "Schedule 15, 'If a TFN has not been provided'",
    );
    cites(
        "whm_unregistered_no_tfn",
        "withholding",
        "Schedule 15, 'Unregistered employer'; Schedule 1, scale 4",
    );
    cites(
        "whm_unregistered_no_tfn",
        "payment_used",
        "Schedule 1, scale 4",
    );
    cites(
        "whm_lookup_tool",
        "payment_used",
        "Schedule 15, 'Using a formula'",
    );
    assert!(
        authority("whm_unregistered", "withholding").contains("'Working out the weekly earnings'")
    );
    assert!(
        authority("whm_unregistered_monthly", "withholding")
            .contains("'Calculating withholding fortnightly, monthly or quarterly amounts'")
    );
    cites(
        "seniors_example",
        "weekly_earnings_used",
        "Schedule 9, 'Using a formula'",
    );
    cites(
        "seniors_example",
        "withholding",
        "Schedule 9, 'Working out the withholding amount'",
    );
    for id in ["weekly_earnings_used", "withholding"] {
        cites(
            "seniors_fortnightly",
            id,
            "Schedule 9, 'Calculating fortnightly or monthly withholding amounts'",
        );
    }
    cites(
        "seniors_study_loan",
        "schedule_9_withholding",
        "Schedule 9, 'Working out the withholding amount'",
    );
    cites(
        "seniors_study_loan",
        "study_loan_component",
        "Schedule 8, 'Study and training support loans component rates' and 'Rounding",
    );
    cites(
        "seniors_study_loan_monthly",
        "study_loan_component",
        "Schedule 8, 'Study and training support loans component rates' and 'Calculating fortnightly, monthly",
    );
    cites(
        "seniors_study_loan",
        "withholding",
        "Schedule 9, 'When your payee has a study and training support loan debt', and the study and training support loans weekly tax table",
    );
    cites(
        "stsl_example_1",
        "weekly_earnings_used",
        "Schedule 8, 'Using a formula'",
    );
    cites(
        "stsl_example_2",
        "weekly_earnings_used",
        "Schedule 8, 'Calculating fortnightly, monthly or quarterly withholding amounts'",
    );
    cites(
        "stsl_example_1",
        "schedule_1_withholding",
        "Schedule 1, 'Using a formula' and 'Rounding of withholding amounts'",
    );
    cites(
        "stsl_example_1",
        "study_loan_component",
        "Schedule 8, 'Study and training support loans component rates'",
    );
    cites("stsl_example_1", "withholding", "Schedule 8, 'Software'");
    cites(
        "sg_capped",
        "earnings_used",
        "SGAA s 6(1), 'ordinary time earnings', paragraph (b)",
    );
    cites("sg_capped", "minimum_sg", "SGAA s 19(2), item 13");
    cites(
        "sg_capped",
        "additional_contribution",
        "SGAA s 23(2) and (10)",
    );
    cites(
        "peter_year_5",
        "remaining_expenditure",
        "ITAA 1997 s 25-25(4) step 1",
    );
    cites(
        "peter_year_5",
        "maximum_deduction",
        "ITAA 1997 s 25-25(4) steps 2 to 4",
    );
    cites("peter_year_5", "deduction", "ITAA 1997 s 25-25(2) and (3)");
    cites("not_spread", "maximum_deduction", "ITAA 1997 s 25-25(6)");
    cites("not_spread", "deduction", "ITAA 1997 s 25-25(6)(c) and (d)");
    assert_eq!(
        case_doc("repaid_early")["result"]["rates"]["period_of_the_loan"],
        "s 25-25(5)(b): to the day the loan was repaid"
    );
}

#[test]
fn a_refused_withholding_sg_or_borrowing_input_gets_the_commands_own_refusal() {
    let refused = |command: &str, facts: &str| {
        let input = withholding_sg_borrowing(command, facts);
        let plain = api::evaluate_coded(command, &input).unwrap_err();
        let derived = api::evaluate_coded("derive", &json!({"command": command, "input": input}));
        assert_eq!(derived.unwrap_err(), plain);
        plain.code
    };
    assert_eq!(
        refused(WHM, r#"{"payments_before": "135000.50"}"#),
        "working_holiday_maker.band_unsettled"
    );
    assert_eq!(
        refused(STSL, r#"{"year": "2025-26"}"#),
        "input.missing_field"
    );
    assert_eq!(
        refused(SENIORS, r#"{"pay_period": "quarterly"}"#),
        "input.pay_period_invalid"
    );
    assert_eq!(refused(SG, r#"{"quarter": 5}"#), "input.quarter_invalid");
    assert_eq!(
        refused(
            BORROWING,
            r#"{"borrowing_expenses": "90.00", "year": "2025-26"}"#
        ),
        "borrowing.s25_25_6_not_spread"
    );
    assert_eq!(
        refused(
            BORROWING,
            r#"{"loan_start": "2020-07-03", "loan_term_end": "2045-07-02", "year": "2026-27"}"#
        ),
        "borrowing.loan_period_ended"
    );
}

// Basic income tax, the low income and seniors and pensioners tax offsets,
// the Medicare levy and the Medicare levy surcharge.

/// Cases by name for the personal tax worksheets: the ATO's examples the
/// worksheets' own tests hold, and a case for each branch. 2025-26 unless a
/// case names its year.
const PERSONAL: [(&str, &str, &str); 42] = [
    // 26,800 at 16% and 55,000 at 30%.
    (
        "resident",
        "resident-tax",
        r#"{"taxable_income": "100000"}"#,
    ),
    (
        "resident_2026",
        "resident-tax",
        r#"{"taxable_income": "200000", "year": "2026-27"}"#,
    ),
    (
        "resident_nil",
        "resident-tax",
        r#"{"taxable_income": "18200"}"#,
    ),
    // The ATO's foreign resident table: $45,000 at 30%.
    (
        "foreign",
        "foreign-resident-tax",
        r#"{"taxable_income": "45000"}"#,
    ),
    (
        "foreign_2026",
        "foreign-resident-tax",
        r#"{"taxable_income": "200000", "year": "2026-27"}"#,
    ),
    // Each item of the s 61-115(1) table, the cut-out and above it.
    (
        "lito_1",
        "low-income-tax-offset",
        r#"{"taxable_income": "30000"}"#,
    ),
    (
        "lito_2",
        "low-income-tax-offset",
        r#"{"taxable_income": "40000"}"#,
    ),
    (
        "lito_3",
        "low-income-tax-offset",
        r#"{"taxable_income": "50000"}"#,
    ),
    (
        "lito_cut_out",
        "low-income-tax-offset",
        r#"{"taxable_income": "66667"}"#,
    ),
    (
        "lito_none",
        "low-income-tax-offset",
        r#"{"taxable_income": "70000"}"#,
    ),
    // The ATO's SAPTO examples: José, Clare, Keith, Deb and example 9.
    (
        "jose",
        "seniors-pensioners-tax-offset",
        r#"{"rebate_income": "39000", "status": "single"}"#,
    ),
    (
        "clare",
        "seniors-pensioners-tax-offset",
        r#"{"rebate_income": "23020", "status": "couple", "spouse_rebate_income": "25677"}"#,
    ),
    (
        "keith",
        "seniors-pensioners-tax-offset",
        r#"{"rebate_income": "33650", "status": "couple", "spouse_rebate_income": "0"}"#,
    ),
    (
        "deb",
        "seniors-pensioners-tax-offset",
        r#"{"rebate_income": "64020", "status": "couple", "spouse_rebate_income": "25677"}"#,
    ),
    (
        "example_9",
        "seniors-pensioners-tax-offset",
        r#"{"rebate_income": "48000", "status": "illness_separated", "spouse_rebate_income": "10000", "spouse_taxable_income": "10000"}"#,
    ),
    // A spouse at most $6,000 passes the whole base amount; an offset at its
    // base amount takes nothing.
    (
        "whole_transfer",
        "seniors-pensioners-tax-offset",
        r#"{"rebate_income": "35000", "status": "couple", "spouse_rebate_income": "5000", "spouse_taxable_income": "5000"}"#,
    ),
    (
        "no_transfer",
        "seniors-pensioners-tax-offset",
        r#"{"rebate_income": "25000", "status": "couple", "spouse_rebate_income": "25000", "spouse_taxable_income": "5000"}"#,
    ),
    (
        "sapto_2026",
        "seniors-pensioners-tax-offset",
        r#"{"rebate_income": "39000", "status": "single", "year": "2026-27"}"#,
    ),
    // s 6(1) above the phase-in limit, s 7(2) and s 7(1).
    ("levy", "medicare-levy", r#"{"taxable_income": "50000"}"#),
    (
        "levy_shade_in",
        "medicare-levy",
        r#"{"taxable_income": "29000", "year": "2026-27"}"#,
    ),
    (
        "levy_nil",
        "medicare-levy",
        r#"{"taxable_income": "28011"}"#,
    ),
    // The ATO's Ashton example: s 7(2), then s 8(2) unshared.
    (
        "ashton",
        "medicare-levy",
        r#"{"taxable_income": "49700", "sapto_entitled": true, "spouse_taxable_income": "21700", "spouse_sapto_entitled": true}"#,
    ),
    // s 8(3) shares the reduction; s 8(4) passes the spouse's excess.
    (
        "shared",
        "medicare-levy",
        r#"{"taxable_income": "40000", "spouse_taxable_income": "30000", "spouse_sapto_entitled": false, "dependent_children": 3}"#,
    ),
    (
        "spouse_excess",
        "medicare-levy",
        r#"{"taxable_income": "40000", "spouse_taxable_income": "29000", "spouse_sapto_entitled": false, "dependent_children": 3}"#,
    ),
    (
        "half_cent",
        "medicare-levy",
        r#"{"taxable_income": "41496", "spouse_taxable_income": "50568", "spouse_sapto_entitled": true, "dependent_children": 7, "year": "2026-27"}"#,
    ),
    (
        "spouse_sapto_excess",
        "medicare-levy",
        r#"{"taxable_income": "30000", "spouse_taxable_income": "45000", "spouse_sapto_entitled": true}"#,
    ),
    // s 8(1): at or below the family income threshold, alone and with a
    // liable spouse.
    (
        "family_nil",
        "medicare-levy",
        r#"{"taxable_income": "40000", "dependant_offset_entitled": true, "dependent_children": 2}"#,
    ),
    (
        "shared_nil",
        "medicare-levy",
        r#"{"taxable_income": "30000", "spouse_taxable_income": "29000", "spouse_sapto_entitled": false, "dependent_children": 5}"#,
    ),
    // s 9(1): Priya's full year, part years, and after ss 8(3) and 8(4).
    (
        "priya",
        "medicare-levy",
        r#"{"taxable_income": "90000", "medicare_levy_exemption_full_days": 365}"#,
    ),
    (
        "part_year",
        "medicare-levy",
        r#"{"taxable_income": "50000", "medicare_levy_exemption_full_days": 100, "medicare_levy_exemption_half_days": 100}"#,
    ),
    (
        "shared_exempt",
        "medicare-levy",
        r#"{"taxable_income": "40000", "spouse_taxable_income": "30000", "spouse_sapto_entitled": false, "dependent_children": 3, "medicare_levy_exemption_full_days": 73}"#,
    ),
    (
        "excess_exempt",
        "medicare-levy",
        r#"{"taxable_income": "40000", "spouse_taxable_income": "29000", "spouse_sapto_entitled": false, "dependent_children": 3, "medicare_levy_exemption_half_days": 365}"#,
    ),
    // The ATO's surcharge examples: Tom, Josh, Jacinta and Kevin.
    (
        "tom",
        "medicare-levy-surcharge",
        r#"{"taxable_income": "90000", "reportable_fringe_benefits": "27000", "income_for_surcharge_purposes": "117000", "days_without_cover": 365, "year": "2026-27"}"#,
    ),
    (
        "josh",
        "medicare-levy-surcharge",
        r#"{"taxable_income": "90000", "reportable_fringe_benefits": "20000", "income_for_surcharge_purposes": "127000", "days_without_cover": 365}"#,
    ),
    (
        "jacinta",
        "medicare-levy-surcharge",
        r#"{"taxable_income": "109000", "reportable_fringe_benefits": "0", "income_for_surcharge_purposes": "109000", "days_without_cover": 199}"#,
    ),
    (
        "kevin",
        "medicare-levy-surcharge",
        r#"{"taxable_income": "120000", "reportable_fringe_benefits": "0", "income_for_surcharge_purposes": "120000", "days_without_cover": 193, "spouse_income_for_surcharge_purposes": "100000", "dependent_children": 3}"#,
    ),
    // Tier 3; under tier 1; a married person on $28,011 of their own, and a
    // family under the family tier 1 threshold.
    (
        "tier_3",
        "medicare-levy-surcharge",
        r#"{"taxable_income": "200000", "reportable_fringe_benefits": "0", "income_for_surcharge_purposes": "200000", "days_without_cover": 365}"#,
    ),
    (
        "under_tier_1",
        "medicare-levy-surcharge",
        r#"{"taxable_income": "90000", "reportable_fringe_benefits": "0", "income_for_surcharge_purposes": "90000", "days_without_cover": 365}"#,
    ),
    (
        "own_income_low",
        "medicare-levy-surcharge",
        r#"{"taxable_income": "28011", "reportable_fringe_benefits": "0", "income_for_surcharge_purposes": "28011", "days_without_cover": 365, "spouse_income_for_surcharge_purposes": "240000"}"#,
    ),
    (
        "family_under_tier_1",
        "medicare-levy-surcharge",
        r#"{"taxable_income": "50000", "reportable_fringe_benefits": "0", "income_for_surcharge_purposes": "50000", "days_without_cover": 365, "spouse_income_for_surcharge_purposes": "50000"}"#,
    ),
    // Whole-dollar inputs without a fractional part stay exact in the working.
    (
        "resident_whole_dollars",
        "resident-tax",
        r#"{"taxable_income": "100000"}"#,
    ),
    (
        "levy_whole_dollars",
        "medicare-levy",
        r#"{"taxable_income": "40000", "dependant_offset_entitled": true}"#,
    ),
];

/// A personal tax input: `facts` for 2025-26 unless they name the year, with
/// no s 9A adjustment for the Medicare levy.
fn personal(command: &str, facts: &str) -> Value {
    let mut input = json!({"year": "2025-26", "scope_confirmed": true});
    if command == "medicare-levy" {
        input["section_9a_adjustment_applies"] = json!(false);
    }
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

/// The derivation of the `PERSONAL` case `name`.
fn personal_doc(name: &str) -> Value {
    let (_, command, facts) = PERSONAL.iter().find(|(n, ..)| *n == name).unwrap();
    derive(command, &personal(command, facts))
}

#[test]
fn each_personal_tax_case_is_recomputed_from_its_working() {
    let mut wrong = Vec::new();
    for (name, command, facts) in PERSONAL {
        let input = personal(command, facts);
        let doc = derive(command, &input);
        let plain = api::evaluate(command, &input).unwrap();
        let (derived, plain) = (doc["result"].to_string(), plain.to_string());
        if derived != plain {
            wrong.push(format!("{name}: the result is not the plain one"));
        }
        wrong.extend(problems(&doc).into_iter().map(|p| format!("{name}: {p}")));
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// How a step's text is checked: whole, by its start, by its end or by a part.
#[derive(Clone, Copy)]
enum Text {
    Is,
    Starts,
    Ends,
    Has,
}

/// Each check of `field` of a `PERSONAL` case's step, failed or not, so one run
/// shows every one that fails.
fn mismatches(checks: &[(&str, &str, &str, Text, &str)]) -> Vec<String> {
    checks
        .iter()
        .filter_map(|&(name, id, field, how, expected)| {
            let found = step_of(&personal_doc(name), id)[field]
                .as_str()
                .unwrap()
                .to_string();
            let ok = match how {
                Text::Is => found == expected,
                Text::Starts => found.starts_with(expected),
                Text::Ends => found.ends_with(expected),
                Text::Has => found.contains(expected),
            };
            (!ok).then(|| format!("{name} {id} {field}: expected {expected}\n  found {found}"))
        })
        .collect()
}

/// The worked examples, each operand as the worksheet used it.
#[test]
fn the_personal_tax_examples_show_their_operands() {
    use Text::{Ends, Has, Is, Starts};
    let (w, v) = ("working", "value");
    let wrong = mismatches(&[
        // Schedule 7 Part I: 26,800 x 16% + 55,000 x 30% = 20,788.
        (
            "resident",
            "basic_income_tax",
            w,
            Is,
            "max(0, min(taxable_income_used, 45000) - 18200) x over_18200 \
             + max(0, min(taxable_income_used, 135000) - 45000) x over_45000 \
             + max(0, min(taxable_income_used, 190000) - 135000) x over_135000 \
             + max(0, taxable_income_used - 190000) x over_190000 \
             = max(0, min(100000, 45000) - 18200) x 0.16 + max(0, min(100000, 135000) - 45000) x 0.30 \
             + max(0, min(100000, 190000) - 135000) x 0.37 + max(0, 100000 - 190000) x 0.45 \
             = 20788.00",
        ),
        // 2026-27: 26,800 x 15% + 90,000 x 30% + 55,000 x 37% + 10,000 x 45%.
        ("resident_2026", "basic_income_tax", v, Is, "55870.00"),
        ("resident_nil", "basic_income_tax", v, Is, "0.00"),
        // Schedule 7 Part II: no tax-free threshold, $45,000 at 30%.
        (
            "foreign",
            "basic_income_tax",
            w,
            Is,
            "max(0, min(taxable_income_used, 135000) - 0) x over_0 \
             + max(0, min(taxable_income_used, 190000) - 135000) x over_135000 \
             + max(0, taxable_income_used - 190000) x over_190000 \
             = max(0, min(45000, 135000) - 0) x 0.30 + max(0, min(45000, 190000) - 135000) x 0.37 \
             + max(0, 45000 - 190000) x 0.45 = 13500.00",
        ),
        ("foreign_2026", "basic_income_tax", v, Is, "65350.00"),
        // s 61-115(1): $700; $700 less 5% of $2,500; $325 less 1.5% of $5,000;
        // at $66,667 item 3 gives minus half a cent, so nil.
        (
            "lito_1",
            "low_income_tax_offset",
            w,
            Is,
            "maximum = 700 = 700",
        ),
        (
            "lito_2",
            "low_income_tax_offset",
            w,
            Is,
            "maximum - (taxable_income_used - first_step) x first_rate \
             = 700 - (40000 - 37500) x 0.05 = 575.00",
        ),
        (
            "lito_3",
            "low_income_tax_offset",
            w,
            Is,
            "max(0, second_amount - (taxable_income_used - second_step) x second_rate) \
             = max(0, 325 - (50000 - 45000) x 0.015) = 250.000",
        ),
        ("lito_cut_out", "low_income_tax_offset", v, Is, "0.00"),
        ("lito_none", "low_income_tax_offset", w, Is, "0 = 0 = 0"),
        // José: $2,230 less 12.5 cents for each of $4,081 above $34,919,
        // $1,719.875, rounded up; the cut-out is $2,230 / 0.125 + $34,919.
        (
            "jose",
            "rebate_threshold",
            w,
            Is,
            "dollars_up((rebate_maximum_amount + base_rebate_amount) / lowest_marginal_tax_rate \
             + tax_free_threshold) = dollars_up((445 + 2230) / 0.16 + 18200) = 34919",
        ),
        (
            "jose",
            "cut_out_threshold",
            w,
            Is,
            "dollars_up(base_rebate_amount / shade_out_rate + rebate_threshold) \
             = dollars_up(2230 / 0.125 + 34919) = 52759",
        ),
        (
            "jose",
            "seniors_pensioners_tax_offset",
            w,
            Is,
            "dollars_up(max(0, base_rebate_amount - max(0, rebate_income_used - rebate_threshold) \
             x shade_out_rate)) = dollars_up(max(0, 2230 - max(0, 39000 - 34919) x 0.125)) = 1720",
        ),
        // Clare is tested on half the couple's $48,697; Deb on half of
        // $89,697, not below the $43,810 cut-out.
        (
            "clare",
            "rebate_income_tested",
            w,
            Is,
            "(rebate_income_used + spouse_rebate_income) / 2 = (23020 + 25677) / 2 = 24348.5",
        ),
        ("keith", "seniors_pensioners_tax_offset", v, Is, "1270.00"),
        ("deb", "seniors_pensioners_tax_offset", w, Is, "0 = 0 = 0"),
        // Example 9: $2,040 less 0.15 x $4,000 transferred; s 9(6) on the
        // raised base gives $12,572 / 0.315, $39,912; the cut-out keeps the
        // unraised threshold; $3,480 less 0.125 x $8,088 is $2,469.
        (
            "example_9",
            "spouse_unused_amount_transferred",
            w,
            Is,
            "max(0, base_rebate_amount - (spouse_taxable_income - transfer_income_floor) \
             x transfer_rate) = max(0, 2040 - (10000 - 6000) x 0.15) = 1440.00",
        ),
        (
            "example_9",
            "rebate_threshold",
            w,
            Is,
            "dollars_up((lowest_marginal_tax_rate x tax_free_threshold + rebate_maximum_amount \
             + base_rebate_amount + spouse_unused_amount_transferred + rebate_reduction_threshold \
             x (rebate_reduction_rate + second_lowest_marginal_tax_rate - lowest_marginal_tax_rate)) \
             / (rebate_reduction_rate + second_lowest_marginal_tax_rate)) \
             = dollars_up((0.16 x 18200 + 445 + 2040 + 1440.00 + 37000 x (0.015 + 0.30 - 0.16)) \
             / (0.015 + 0.30)) = 39912",
        ),
        (
            "example_9",
            "cut_out_threshold",
            w,
            Is,
            "dollars_up(base_rebate_amount / shade_out_rate + dollars_up((rebate_maximum_amount \
             + base_rebate_amount) / lowest_marginal_tax_rate + tax_free_threshold)) \
             = dollars_up(2040 / 0.125 + dollars_up((445 + 2040) / 0.16 + 18200)) = 50052",
        ),
        (
            "example_9",
            "seniors_pensioners_tax_offset",
            w,
            Is,
            "dollars_up(max(0, base_rebate_amount + spouse_unused_amount_transferred \
             - max(0, rebate_income_used - rebate_threshold) x shade_out_rate)) \
             = dollars_up(max(0, 2040 + 1440.00 - max(0, 48000 - 39912) x 0.125)) = 2469",
        ),
        (
            "whole_transfer",
            "spouse_unused_amount_transferred",
            w,
            Is,
            "base_rebate_amount = 1602 = 1602",
        ),
        (
            "no_transfer",
            "spouse_unused_amount_transferred",
            w,
            Is,
            "0 = 0 = 0",
        ),
        // 2026-27 at 15%: (445 + 2,230) / 0.15 + 18,200, rounded up.
        ("sapto_2026", "rebate_threshold", v, Is, "36034.00"),
        // Ashton: s 7(2) leaves 10% of $5,432, $543.20; s 8(2) reduces it
        // by 2% of $61,623 less 0.08 of $9,777, $450.30; $92.90 is left.
        (
            "ashton",
            "low_income_reduction",
            w,
            Is,
            "levy_at_full_rate - min(levy_at_full_rate, (taxable_income_used - threshold_amount) \
             x shade_in_rate) = 994.00 - min(994.00, (49700 - 44268) x 0.10) = 450.80",
        ),
        (
            "ashton",
            "family_reduction",
            w,
            Is,
            "min(min(levy_at_full_rate, (taxable_income_used - threshold_amount) x shade_in_rate), \
             max(0, levy_rate x family_income_threshold - family_shade_out_rate \
             x (family_income - family_income_threshold))) \
             = min(min(994.00, (49700 - 44268) x 0.10), max(0, 0.02 x 61623 - 0.08 x (71400 - 61623))) \
             = 450.30",
        ),
        (
            "ashton",
            "medicare_levy",
            w,
            Is,
            "levy_at_full_rate - low_income_reduction - family_reduction - spouse_excess_reduction \
             = 994.00 - 450.80 - 450.30 - 0 = 92.90",
        ),
        // s 8(3): this person's share of the family's reduction, worked
        // times family income and divided by it once.
        (
            "shared",
            "family_reduction",
            w,
            Starts,
            "min(levy_at_full_rate x family_income, max(0, levy_rate x family_income_threshold \
             - family_shade_out_rate x (family_income - family_income_threshold)) \
             x taxable_income_used) / family_income",
        ),
        ("shared", "medicare_levy", v, Is, "557.03"),
        // s 8(4): the spouse's share above the spouse's own levy, with the
        // spouse's threshold amount and family income threshold as figures.
        (
            "spouse_excess",
            "spouse_excess_reduction",
            w,
            Has,
            " x spouse_taxable_income - min(spouse_taxable_income x levy_rate, \
             (spouse_taxable_income - ",
        ),
        ("spouse_excess", "spouse_excess_reduction", v, Is, "113.43"),
        ("spouse_excess", "medicare_levy", v, Is, "393.70"),
        // The 2 shares divided once: exactly half a cent, rounded up.
        ("half_cent", "medicare_levy", w, Ends, " = 274.515"),
        ("half_cent", "medicare_levy", v, Is, "274.52"),
        ("spouse_sapto_excess", "medicare_levy", v, Is, "174.72"),
        // s 9(1): $1,000 x (730 - 200 - 100) / 730.
        (
            "part_year",
            "medicare_levy",
            w,
            Starts,
            "(levy_at_full_rate - low_income_reduction) \
             x (2 x days_in_year - (2 x exemption_full_days + exemption_half_days)) / (2 x days_in_year) \
             = (1000.00 - 0.00) x (2 x 365 - (2 x 100 + 100)) / (2 x 365) = 589.04109589",
        ),
        ("priya", "medicare_levy", v, Is, "0.00"),
        ("shared_exempt", "medicare_levy", v, Is, "445.62"),
        ("excess_exempt", "medicare_levy", v, Is, "196.85"),
        // Tom: ($90,000 + $27,000) x 1%; Jacinta: $109,000 x 1% x 199 / 365.
        (
            "tom",
            "medicare_levy_surcharge",
            w,
            Is,
            "surcharge_base x surcharge_rate x days_without_cover / days_in_year \
             = 117000 x 0.01 x 365 / 365 = 1170.00",
        ),
        ("josh", "medicare_levy_surcharge", v, Is, "1375.00"),
        ("jacinta", "medicare_levy_surcharge", v, Is, "594.27"),
        (
            "kevin",
            "family_income_for_surcharge_purposes",
            w,
            Is,
            "income_for_surcharge_purposes + spouse_income_for_surcharge_purposes \
             = 120000 + 100000 = 220000",
        ),
        ("kevin", "medicare_levy_surcharge", v, Is, "634.52"),
        ("tier_3", "medicare_levy_surcharge", v, Is, "3000.00"),
    ]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn each_personal_tax_step_cites_the_provision_its_branch_applies() {
    use Text::{Ends, Has, Is, Starts};
    let a = "authority";
    let (lito, sapto, transfer) = (
        "low_income_tax_offset",
        "seniors_pensioners_tax_offset",
        "spouse_unused_amount_transferred",
    );
    let (reduction, family, excess, surcharge) = (
        "low_income_reduction",
        "family_reduction",
        "spouse_excess_reduction",
        "medicare_levy_surcharge",
    );
    let wrong = mismatches(&[
        (
            "resident",
            "basic_income_tax",
            a,
            Has,
            "Schedule 7 Part I clause 1(b), the table for the 2024-25 or 2025-26",
        ),
        (
            "resident_2026",
            "basic_income_tax",
            a,
            Has,
            "Schedule 7 Part I clause 1(b), the table for the 2026-27",
        ),
        (
            "foreign",
            "basic_income_tax",
            a,
            Has,
            "Schedule 7 Part II clause 1(b)",
        ),
        (
            "lito_1",
            lito,
            a,
            Starts,
            "ITAA 1997 s 61-115(1), table item 1",
        ),
        (
            "lito_2",
            lito,
            a,
            Starts,
            "ITAA 1997 s 61-115(1), table item 2",
        ),
        (
            "lito_3",
            lito,
            a,
            Starts,
            "ITAA 1997 s 61-115(1), table item 3",
        ),
        (
            "lito_cut_out",
            lito,
            a,
            Starts,
            "ITAA 1997 s 61-115(1), table item 3",
        ),
        ("lito_none", lito, a, Starts, "ITAA 1997 s 61-110(1)(b)"),
        // The 2015 Regulation before 2026-27, the 2025 Regulations from it.
        (
            "jose",
            "rebate_threshold",
            a,
            Is,
            "Income Tax Assessment (1936 Act) Regulation 2015 s 9(4)(a), (5) and (8)",
        ),
        (
            "sapto_2026",
            "rebate_threshold",
            a,
            Starts,
            "Income Tax Assessment (1936 Act) Regulations 2025 s 9(4)(a)",
        ),
        ("example_9", "rebate_threshold", a, Has, "s 12(3)"),
        (
            "jose",
            "base_rebate_amount",
            a,
            Ends,
            "item 1: not a spouse",
        ),
        ("clare", "base_rebate_amount", a, Ends, "item 2: a spouse"),
        ("example_9", "base_rebate_amount", a, Has, "item 3"),
        (
            "clare",
            "rebate_income_tested",
            a,
            Starts,
            "ITAA 1936 s 160AAAA(4)",
        ),
        ("deb", sapto, a, Has, "s 10 and ITAA 1936 s 160AAAA(3)"),
        ("example_9", transfer, a, Has, "(3)(b)(ii) and (4)"),
        ("whole_transfer", transfer, a, Has, "(3)(b)(i)"),
        ("no_transfer", transfer, a, Has, "s 12(1)(d) and (e)"),
        (
            "levy",
            reduction,
            a,
            Starts,
            "Medicare Levy Act 1986 ss 6(1) and 7",
        ),
        (
            "levy_shade_in",
            reduction,
            a,
            Starts,
            "Medicare Levy Act 1986 s 7(2)",
        ),
        (
            "levy_nil",
            reduction,
            a,
            Starts,
            "Medicare Levy Act 1986 s 7(1)",
        ),
        (
            "ashton",
            family,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8(2):",
        ),
        (
            "shared",
            family,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8(2) and (3)",
        ),
        (
            "family_nil",
            family,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8(1)",
        ),
        (
            "shared_nil",
            family,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8(1)",
        ),
        (
            "spouse_excess",
            excess,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8(4)",
        ),
        (
            "ashton",
            excess,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8(3) and (4)",
        ),
        (
            "part_year",
            "exemption_reduction",
            a,
            Has,
            "s 9(1) and ITAA 1936 s 251U(3)",
        ),
        (
            "tom",
            surcharge,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8B(2) to (4)",
        ),
        (
            "under_tier_1",
            surcharge,
            a,
            Has,
            "singles tier 1 threshold, so nil",
        ),
        (
            "kevin",
            surcharge,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8D(3), (4)",
        ),
        (
            "own_income_low",
            surcharge,
            a,
            Starts,
            "Medicare Levy Act 1986 s 8D(3)(c)",
        ),
        (
            "family_under_tier_1",
            surcharge,
            a,
            Has,
            "family tier 1 threshold, so nil",
        ),
    ]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_refused_personal_tax_input_gets_the_commands_own_refusal() {
    let refused = |name: &str, change: Value| {
        let (_, command, facts) = PERSONAL.iter().find(|(n, ..)| *n == name).unwrap();
        let mut input = personal(command, facts);
        merge(&mut input, change);
        let plain = api::evaluate_coded(command, &input).unwrap_err();
        let derived = api::evaluate_coded("derive", &json!({"command": command, "input": input}));
        assert_eq!(derived.unwrap_err(), plain, "{name}");
        plain.code
    };
    let cents = json!({"taxable_income": "100000.50"});
    assert_eq!(
        refused("resident", cents.clone()),
        "input.amount_not_whole_dollars"
    );
    assert_eq!(
        refused("foreign", json!({"year": "2023-24"})),
        "period.unsupported"
    );
    assert_eq!(refused("lito_1", cents), "input.amount_not_whole_dollars");
    assert_eq!(
        refused("clare", json!({"status": "single"})),
        "input.field_unknown"
    );
    assert_eq!(
        refused("levy", json!({"section_9a_adjustment_applies": true})),
        "medicare.s9a_adjustment_out_of_scope"
    );
    assert_eq!(
        refused("tom", json!({"dependent_children": 1})),
        "input.field_unknown"
    );
}

/// A small seeded generator, so the inputs below are the same on every run.
struct Seeded(u64);

impl Seeded {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % n
    }

    fn dollars(&mut self, most: u64) -> String {
        self.below(most + 1).to_string()
    }
}

/// Seeded inputs across each personal tax worksheet's branches: each derives
/// to the plain result, or the plain refusal, with every step recomputed.
#[test]
fn seeded_personal_tax_inputs_are_recomputed_from_their_working() {
    let mut rng = Seeded(0x3B15_1126);
    let years = ["2024-25", "2025-26", "2026-27"];
    let mut explained = 0;
    for _ in 0..300 {
        let year = years[rng.below(3) as usize];
        let income = rng.dollars(250_000);
        let mut cases = vec![
            ("resident-tax", json!({"taxable_income": income})),
            ("foreign-resident-tax", json!({"taxable_income": income})),
            (
                "low-income-tax-offset",
                json!({"taxable_income": rng.dollars(80_000)}),
            ),
        ];
        let status = ["single", "illness_separated", "couple"][rng.below(3) as usize];
        let mut sapto = json!({"rebate_income": rng.dollars(70_000), "status": status});
        if status != "single" {
            sapto["spouse_rebate_income"] = json!(rng.dollars(70_000));
            if rng.below(2) == 0 {
                sapto["spouse_taxable_income"] = json!(rng.dollars(30_000));
            }
        }
        cases.push(("seniors-pensioners-tax-offset", sapto));
        let mut levy = json!({"taxable_income": rng.dollars(120_000),
                              "section_9a_adjustment_applies": false,
                              "sapto_entitled": rng.below(2) == 0});
        match rng.below(3) {
            0 => {
                levy["spouse_taxable_income"] = json!(rng.dollars(90_000));
                levy["spouse_sapto_entitled"] = json!(rng.below(2) == 0);
                levy["dependent_children"] = json!(rng.below(6));
            }
            1 => {
                levy["dependant_offset_entitled"] = json!(true);
                levy["dependent_children"] = json!(rng.below(4));
            }
            _ => {}
        }
        if rng.below(3) == 0 {
            let full = rng.below(366);
            levy["medicare_levy_exemption_full_days"] = json!(full);
            levy["medicare_levy_exemption_half_days"] = json!(rng.below(366 - full));
        }
        cases.push(("medicare-levy", levy));
        let own = rng.dollars(300_000);
        let mut surcharge = json!({"taxable_income": rng.dollars(200_000),
                                   "reportable_fringe_benefits": rng.dollars(40_000),
                                   "income_for_surcharge_purposes": own,
                                   "days_without_cover": rng.below(366)});
        if rng.below(2) == 0 {
            surcharge["spouse_income_for_surcharge_purposes"] = json!(rng.dollars(300_000));
            surcharge["dependent_children"] = json!(rng.below(5));
        }
        cases.push(("medicare-levy-surcharge", surcharge));
        for (command, mut input) in cases {
            input["year"] = json!(year);
            input["scope_confirmed"] = json!(true);
            let derived =
                api::evaluate_coded("derive", &json!({"command": command, "input": input}));
            match api::evaluate_coded(command, &input) {
                Ok(plain) => {
                    let doc = derived.unwrap();
                    assert_eq!(
                        doc["result"].to_string(),
                        plain.to_string(),
                        "{command} {input}"
                    );
                    let found = problems(&doc);
                    assert!(found.is_empty(), "{command} {input}: {found:#?}");
                    explained += 1;
                }
                Err(plain) => assert_eq!(derived.unwrap_err(), plain, "{command} {input}"),
            }
        }
    }
    assert!(explained > 1500, "only {explained} explained");
}

// Division 296, the contribution caps, study loan repayment, capital gains and
// the main residence exemption.

/// Division 296 cases by name, 2026-27: the Explanatory Memorandum's examples
/// 1.1 to 1.3 as `tests/division_296.rs` holds them, and each branch.
const DIVISION_296: [(&str, &str); 8] = [
    ("jordan", r#"{}"#),
    (
        "kelly",
        r#"{"total_super_balance_end_of_year": "12000000.00", "total_super_earnings": "500000.00"}"#,
    ),
    (
        "kenny",
        r#"{"total_super_balance_end_of_year": "150000.00", "total_super_earnings": "10000.00"}"#,
    ),
    // A cent over $3 million rounds to 0.00%: liable, but nil.
    (
        "rounds_to_nil",
        r#"{"total_super_balance_end_of_year": "3000000.01"}"#,
    ),
    // Negative earnings above $10 million: a negative component, shown as null.
    (
        "negative",
        r#"{"total_super_balance_end_of_year": "12000000.00", "total_super_earnings": "-50000.00"}"#,
    ),
    (
        "nil_earnings",
        r#"{"total_super_balance_end_of_year": "12000000.00", "total_super_earnings": "-0.00"}"#,
    ),
    // s 296-20 removes the liability, not the earnings.
    ("child", r#"{"child_recipient": true}"#),
    // s 296-50: the balance is nil after death.
    (
        "died",
        r#"{"total_super_balance_end_of_year": "12000000.00", "died_in_year": true}"#,
    ),
];

/// A `division-296` input: `facts` over Jordan's $4 million and $100,000.
fn division_296(facts: &str) -> Value {
    let mut input = json!({
        "total_super_balance_end_of_year": "4000000.00", "total_super_earnings": "100000.00",
        "child_recipient": false, "structured_settlement_contribution": false,
        "died_in_year": false, "year": "2026-27", "scope_confirmed": true,
    });
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

/// Contribution caps cases by name, 2025-26 unless they say otherwise.
const CAPS: [(&str, &str); 7] = [
    // ¶7-278, Jane: $1.5 million, $240,000 in 2024-25, 3 years brought forward.
    (
        "jane",
        r#"{"total_super_balance": "1500000", "non_concessional_contributions": "240000", "year": "2024-25"}"#,
    ),
    // s 291-20(4): $15,000 unused covers $10,000 of excess, then all of it.
    (
        "carry_forward",
        r#"{"concessional_contributions": "40000", "unused_concessional_cap": "15000"}"#,
    ),
    (
        "carry_forward_short",
        r#"{"concessional_contributions": "50000", "unused_concessional_cap": "15000"}"#,
    ),
    // s 291-20(3)(b): not below $500,000.
    (
        "no_carry_forward",
        r#"{"total_super_balance": "500000", "concessional_contributions": "40000", "unused_concessional_cap": "15000"}"#,
    ),
    // s 292-85(2)(b): nil at the $2 million transfer balance cap.
    (
        "at_transfer_cap",
        r#"{"total_super_balance": "2000000", "non_concessional_contributions": "1000"}"#,
    ),
    // s 292-85(3)(c): no bring forward at 75.
    (
        "over_75",
        r#"{"under_75_in_year": false, "non_concessional_contributions": "200000"}"#,
    ),
    // s 292-85(5)(a): $1.8 million leaves cap space of twice the cap or less.
    (
        "two_years",
        r#"{"total_super_balance": "1800000", "non_concessional_contributions": "300000"}"#,
    ),
];

fn caps(facts: &str) -> Value {
    let mut input = json!({
        "total_super_balance": "450000", "concessional_contributions": "20000",
        "unused_concessional_cap": "0", "non_concessional_contributions": "0",
        "under_75_in_year": true, "year": "2025-26", "scope_confirmed": true,
    });
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

/// Contribution caps cases with the unused cap worked out by year: the ATO's
/// Sam example (2024-25), the hand-worked history of `tests/contribution_caps.rs`
/// with an earlier year's increase and a proposal (2025-26), and Sam's years at
/// a $500,000 balance.
const CAPS_BY_YEAR: [(&str, &str); 3] = [
    (
        "by_year_sam",
        r#"{"year": "2024-25", "concessional_contributions": "97500", "earlier_concessional_contributions": [
            {"year": "2019-20", "concessional_contributions": "11000"},
            {"year": "2020-21", "concessional_contributions": "12000"},
            {"year": "2021-22", "concessional_contributions": "13000"},
            {"year": "2022-23", "concessional_contributions": "14000"},
            {"year": "2023-24", "concessional_contributions": "15000"}]}"#,
    ),
    (
        "by_year_drawn",
        r#"{"total_super_balance": "400000", "concessional_contributions": "40000",
            "proposed_concessional_contribution": "20000", "earlier_concessional_contributions": [
            {"year": "2018-19", "concessional_contributions": "5000"},
            {"year": "2019-20", "concessional_contributions": "25000"},
            {"year": "2020-21", "concessional_contributions": "15000"},
            {"year": "2021-22", "concessional_contributions": "27500"},
            {"year": "2022-23", "concessional_contributions": "50000", "total_super_balance": "300000"},
            {"year": "2023-24", "concessional_contributions": "7500"},
            {"year": "2024-25", "concessional_contributions": "30000"}]}"#,
    ),
    (
        "by_year_at_limit",
        r#"{"year": "2024-25", "total_super_balance": "500000", "concessional_contributions": "40000", "earlier_concessional_contributions": [
            {"year": "2019-20", "concessional_contributions": "11000"},
            {"year": "2020-21", "concessional_contributions": "12000"},
            {"year": "2021-22", "concessional_contributions": "13000"},
            {"year": "2022-23", "concessional_contributions": "14000"},
            {"year": "2023-24", "concessional_contributions": "15000"}]}"#,
    ),
];

fn caps_by_year(facts: &str) -> Value {
    let mut input = caps(r#"{"defined_benefit_interest": false}"#);
    input
        .as_object_mut()
        .unwrap()
        .remove("unused_concessional_cap");
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

/// Study loan repayment cases: the ATO page's examples 1 to 3 (read on 15
/// November 2026 at 12:23 AEDT) and the minimum repayment income.
const STUDY_LOAN: [(&str, &str, &str); 5] = [
    ("christina", "86380", "2026-27"),
    ("barry", "137064", "2026-27"),
    ("priya", "254780", "2026-27"),
    ("minimum", "69528", "2026-27"),
    ("upper_2025", "150000", "2025-26"),
];

fn study_loan(income: &str, year: &str) -> Value {
    json!({"repayment_income": income, "year": year, "scope_confirmed": true})
}

/// Capital gains cases: the Library's ¶2-240 summary, the corpus's fabricated
/// case with $500 and $1,500 of losses, and earlier years' losses alone.
const CAPITAL_GAINS: [(&str, [&str; 4]); 5] = [
    ("library", ["8000.00", "21435.00", "8660.00", "0.00"]),
    ("fabricated", ["100.00", "1000.00", "500.00", "0.00"]),
    ("absorbed", ["100.00", "1000.00", "1000.00", "500.00"]),
    ("prior_only", ["0.00", "1000.00", "0.00", "300.00"]),
    (
        "negative_zero",
        ["947075.6", "-0.00", "2385959", "12392.20"],
    ),
];

fn capital_gains(amounts: [&str; 4]) -> Value {
    let [other, discount, current, prior] = amounts;
    json!({"other_gains": other, "discount_gains": discount, "current_losses": current,
           "prior_losses": prior, "scope_confirmed": true, "year": "2025-26"})
}

/// Main residence cases: the ATO's and the Library's examples as
/// `tests/main_residence.rs` holds them, and a loss.
const MAIN_RESIDENCE: [(&str, &str); 6] = [
    (
        "peter",
        r#"{"capital_gain": "230000.00", "ownership_start": "2013-10-01", "ownership_end": "2026-03-30",
            "lived_periods": [{"start": "2016-07-01", "end": "2026-03-30"}],
            "income_use_periods": [{"start": "2013-10-01", "end": "2016-06-30", "floor_area": "1"}]}"#,
    ),
    (
        "thomas",
        r#"{"capital_gain": "400000.00", "ownership_start": "2003-07-01", "ownership_end": "2026-06-30",
            "lived_periods": [{"start": "2003-07-01", "end": "2026-06-30"}],
            "income_use_periods": [{"start": "2003-07-01", "end": "2026-06-30", "floor_area": "0.35"}]}"#,
    ),
    (
        "fatima",
        r#"{"capital_gain": "100000.00", "market_value_basis": true, "ownership_start": "1997-12-01",
            "ownership_end": "2026-05-01", "lived_periods": [{"start": "1997-12-01", "end": "2026-05-01"}],
            "income_use_periods": [{"start": "2018-11-01", "end": "2022-08-01", "floor_area": "0.4"}]}"#,
    ),
    (
        "roya",
        r#"{"capital_gain": "320000.00", "market_value_basis": true, "year": "2024-25",
            "ownership_start": "1995-03-01", "ownership_end": "2024-09-29",
            "lived_periods": [{"start": "1995-03-01", "end": "1999-09-28"}],
            "income_use_periods": [{"start": "1999-09-29", "end": "2024-09-29", "floor_area": "1"}],
            "absence_choice_periods": [{"start": "1999-09-29", "end": "2024-09-29"}]}"#,
    ),
    (
        "medford",
        r#"{"capital_gain": "500000.00", "ownership_start": "2013-05-01", "ownership_end": "2026-05-01",
            "lived_periods": [{"start": "2013-05-01", "end": "2025-09-30"}], "new_home_acquired": "2025-10-01"}"#,
    ),
    (
        "loss",
        r#"{"capital_loss": "10000.00", "ownership_start": "2013-10-01", "ownership_end": "2026-03-30",
            "lived_periods": [{"start": "2016-07-01", "end": "2026-03-30"}],
            "income_use_periods": [{"start": "2013-10-01", "end": "2016-06-30", "floor_area": "1"}]}"#,
    ),
];

fn main_residence(facts: &str) -> Value {
    let mut input = json!({
        "capital_gain": "0.00", "capital_loss": "0.00", "market_value_basis": false,
        "lived_periods": [], "income_use_periods": [], "absence_choice_periods": [],
        "new_home_acquired": null, "year": "2025-26", "scope_confirmed": true,
    });
    merge(&mut input, serde_json::from_str(facts).unwrap());
    input
}

/// Every case above as (name, command, input).
fn super_and_cgt_cases() -> Vec<(&'static str, &'static str, Value)> {
    let mut cases = Vec::new();
    for (name, facts) in DIVISION_296 {
        cases.push((name, "division-296", division_296(facts)));
    }
    for (name, facts) in CAPS {
        cases.push((name, "contribution-caps", caps(facts)));
    }
    for (name, facts) in CAPS_BY_YEAR {
        cases.push((name, "contribution-caps", caps_by_year(facts)));
    }
    for (name, income, year) in STUDY_LOAN {
        cases.push((name, "study-loan-repayment", study_loan(income, year)));
    }
    for (name, amounts) in CAPITAL_GAINS {
        cases.push((name, "capital-gains", capital_gains(amounts)));
    }
    for (name, facts) in MAIN_RESIDENCE {
        cases.push((name, "main-residence", main_residence(facts)));
    }
    cases
}

fn super_and_cgt_doc(name: &str) -> Value {
    let cases = super_and_cgt_cases();
    let (_, command, input) = cases.iter().find(|(n, ..)| *n == name).unwrap();
    derive(command, input)
}

#[test]
fn each_super_and_cgt_case_derives_its_plain_result_from_its_working() {
    for (name, command, input) in super_and_cgt_cases() {
        let doc = derive(command, &input);
        let plain = api::evaluate(command, &input).unwrap();
        assert_eq!(doc["result"].to_string(), plain.to_string(), "{name}");
        let found = problems(&doc);
        assert!(found.is_empty(), "{name}: {found:#?}");
    }
}

/// The Explanatory Memorandum's examples 1.1 and 1.2, and each branch's
/// authority: an exception or a nil balance leaves its own reason.
#[test]
fn the_division_296_steps_show_the_memorandum_examples() {
    let jordan = super_and_cgt_doc("jordan");
    assert_eq!(
        working_in(&jordan, "taxable_superannuation_earnings"),
        "total_super_earnings x taxable_percentage / 100 = 100000.00 x 25.00 / 100 = 25000.0000"
    );
    assert_eq!(
        working_in(&jordan, "division_296_tax"),
        "tax_on_taxable_superannuation_earnings + tax_on_very_large_balance_earnings_component \
         = 3750.000000 + 0.00 = 3750.000000"
    );
    let kelly = super_and_cgt_doc("kelly");
    assert_eq!(
        working_in(&kelly, "very_large_balance_earnings_component"),
        "total_super_earnings x very_large_percentage / 100 = 500000.00 x 16.67 / 100 = 83350.0000"
    );
    assert_eq!(
        working_in(&kelly, "tax_on_very_large_balance_earnings_component"),
        "very_large_balance_earnings_component x very_large_rate = 83350.0000 x 0.10 = 8335.000000"
    );
    assert_eq!(kelly["result"]["amounts"]["division_296_tax"], "64585.00");
    assert!(
        authority_in(&kelly, "tax_on_very_large_balance_earnings_component")
            .starts_with("Superannuation (Building a Stronger and Fairer Super System) Imposition Act 2026 s 5(b)(ii)")
    );
    assert!(
        authority_in(
            &super_and_cgt_doc("kenny"),
            "taxable_superannuation_earnings"
        )
        .starts_with("ITAA 1997 s 296-40(1)(a) and (3)")
    );
    let child = super_and_cgt_doc("child");
    assert_eq!(
        child["result"]["amounts"]["taxable_superannuation_earnings"],
        "25000.00"
    );
    assert_eq!(
        working_in(&child, "tax_on_taxable_superannuation_earnings"),
        "0 = 0 = 0"
    );
    assert!(authority_in(&child, "tax_on_taxable_superannuation_earnings").ends_with("(s 296-20)"));
    assert!(
        authority_in(
            &super_and_cgt_doc("died"),
            "taxable_superannuation_earnings"
        )
        .starts_with("ITAA 1997 s 296-50")
    );
    assert!(
        authority_in(
            &super_and_cgt_doc("nil_earnings"),
            "taxable_superannuation_earnings"
        )
        .starts_with("ITAA 1997 s 296-40(1)(b)")
    );
    // s 296-45 has no 'greater than nil' condition: a negative component, which
    // the result and its step show as null.
    let negative = super_and_cgt_doc("negative");
    let component = step_of(&negative, "very_large_balance_earnings_component");
    assert_eq!(
        component["working"],
        "total_super_earnings x very_large_percentage / 100 = -50000.00 x 16.67 / 100 = -8335.0000"
    );
    assert_eq!(component["value"], Value::Null);
    assert_eq!(
        negative["result"]["amounts"]["very_large_balance_earnings_component"],
        Value::Null
    );
}

#[test]
fn the_contribution_caps_steps_cite_their_subsection() {
    let jane = super_and_cgt_doc("jane");
    assert_eq!(
        working_in(&jane, "non_concessional_available"),
        "non_concessional_cap x non_concessional_cap_multiple = 120000 x 3 = 360000"
    );
    assert!(
        authority_in(&jane, "non_concessional_available")
            .starts_with("ITAA 1997 s 292-85(3) and (5)(b)")
    );
    assert_eq!(
        jane["result"]["amounts"]["non_concessional_remaining"],
        "120000.00"
    );
    assert_eq!(
        working_in(&super_and_cgt_doc("carry_forward"), "carry_forward_applied"),
        "min(unused_concessional_cap, max(0, concessional_contributions - concessional_cap)) \
         = min(15000, max(0, 40000 - 30000)) = 10000"
    );
    let short = super_and_cgt_doc("carry_forward_short");
    assert_eq!(
        working_in(&short, "excess_concessional"),
        "max(0, concessional_contributions - concessional_available) = max(0, 50000 - 45000) = 5000"
    );
    let none = super_and_cgt_doc("no_carry_forward");
    assert_eq!(working_in(&none, "carry_forward_applied"), "0 = 0 = 0");
    assert!(authority_in(&none, "carry_forward_applied").starts_with("ITAA 1997 s 291-20(3)(b)"));
    assert_eq!(none["result"]["amounts"]["excess_concessional"], "10000.00");
    let multiple =
        |name: &str| authority_in(&super_and_cgt_doc(name), "non_concessional_available");
    assert!(multiple("at_transfer_cap").starts_with("ITAA 1997 s 292-85(2)(b)"));
    assert!(multiple("over_75").starts_with("ITAA 1997 s 292-85(2)(a)"));
    assert!(multiple("two_years").starts_with("ITAA 1997 s 292-85(3) and (5)(a)"));
    assert_eq!(
        super_and_cgt_doc("at_transfer_cap")["result"]["amounts"]["excess_non_concessional"],
        "1000.00"
    );
}

/// The ATO's examples 1 to 3: 15% over $69,528; 17% over $129,717 on the
/// exact base of $9,028.35, 35 cents above the page's $10,276.99; 10%.
#[test]
fn the_study_loan_steps_show_the_ato_examples() {
    let christina = super_and_cgt_doc("christina");
    assert_eq!(
        working_in(&christina, "marginal_amount"),
        "max(0, min(repayment_income_used, lower_band_top) - minimum_repayment_income) x lower_rate \
         + max(0, repayment_income_used - lower_band_top) x upper_rate \
         = max(0, min(86380, 129717) - 69528) x 0.15 + max(0, 86380 - 129717) x 0.17 = 2527.80"
    );
    assert_eq!(
        christina["result"]["amounts"]["compulsory_repayment"],
        "2527.80"
    );
    assert_eq!(
        super_and_cgt_doc("barry")["result"]["amounts"]["compulsory_repayment"],
        "10277.34"
    );
    let priya = super_and_cgt_doc("priya");
    assert_eq!(
        working_in(&priya, "compulsory_repayment"),
        "ten_percent_of_income = 25478.00 = 25478.00"
    );
    assert!(
        authority_in(&priya, "compulsory_repayment").starts_with("HESA s 154-20(1)(a) and (b)")
    );
    let minimum = super_and_cgt_doc("minimum");
    assert_eq!(working_in(&minimum, "compulsory_repayment"), "0 = 0 = 0");
    assert!(authority_in(&minimum, "compulsory_repayment").starts_with("HESA s 154-1(1)(a)"));
}

/// ¶2-240: $8,660 of losses absorb the $8,000 of other gains and $660 of the
/// discount gains; half of the $20,775 left is $10,387.50.
#[test]
fn the_capital_gains_steps_apply_losses_then_the_discount() {
    let library = super_and_cgt_doc("library");
    assert_eq!(
        working_in(&library, "losses_used"),
        "min(other_gains, current_losses + prior_losses) \
         + min(discount_gains, current_losses + prior_losses - min(other_gains, current_losses + prior_losses)) \
         = min(8000.00, 8660.00 + 0.00) + min(21435.00, 8660.00 + 0.00 - min(8000.00, 8660.00 + 0.00)) = 8660.00"
    );
    assert_eq!(
        working_in(&library, "discount_applied"),
        "(discount_gains - min(discount_gains, current_losses + prior_losses - min(other_gains, current_losses + prior_losses))) \
         x (1 - discount) \
         = (21435.00 - min(21435.00, 8660.00 + 0.00 - min(8000.00, 8660.00 + 0.00))) x (1 - 0.50) = 10387.5000"
    );
    assert_eq!(library["result"]["amounts"]["net_capital_gain"], "10387.50");
    assert!(authority_in(&library, "discount_applied").contains("s 115-100(a)"));
    let absorbed = super_and_cgt_doc("absorbed");
    assert_eq!(
        working_in(&absorbed, "losses_remaining"),
        "current_losses + prior_losses - min(other_gains, current_losses + prior_losses) \
         - min(discount_gains, current_losses + prior_losses - min(other_gains, current_losses + prior_losses)) \
         = 1000.00 + 500.00 - min(100.00, 1000.00 + 500.00) \
         - min(1000.00, 1000.00 + 500.00 - min(100.00, 1000.00 + 500.00)) = 400.00"
    );
    // A discount gain of -0.00 that the losses absorb stays an unsigned nil,
    // as the Python corpus has it.
    let negative_zero = super_and_cgt_doc("negative_zero");
    assert_eq!(
        negative_zero["result"]["amounts"]["discount_applied"],
        "0.00"
    );
}

/// QC66033's Peter and Fatima, and QC66030's Roya: the gain times the taxable
/// days over the days in the ownership period.
#[test]
fn the_main_residence_steps_apportion_by_days() {
    let peter = super_and_cgt_doc("peter");
    assert_eq!(
        working_in(&peter, "assessable_capital_gain"),
        "cents(capital_gain x non_main_residence_days / ownership_period_days) \
         = cents(230000.00 x 1004 / 4564) = 50595.97"
    );
    assert_eq!(
        working_in(&peter, "exempt_capital_gain"),
        "capital_gain - assessable_capital_gain = 230000.00 - 50595.97 = 179404.03"
    );
    assert_eq!(
        authority_in(&peter, "assessable_capital_gain"),
        "ITAA 1997 s 118-185(2) and s 118-190(2)"
    );
    let fatima = super_and_cgt_doc("fatima");
    assert_eq!(
        working_in(&fatima, "assessable_capital_gain"),
        "cents(capital_gain x non_main_residence_days / ownership_period_days) \
         = cents(100000.00 x 548.0 / 2739) = 20007.30"
    );
    assert!(authority_in(&fatima, "assessable_capital_gain").ends_with("(s 118-192(2))"));
    assert_eq!(
        super_and_cgt_doc("roya")["result"]["amounts"]["assessable_capital_gain"],
        "243162.16"
    );
    let loss = super_and_cgt_doc("loss");
    assert_eq!(
        working_in(&loss, "exempt_capital_loss"),
        "capital_loss - assessable_capital_loss = 10000.00 - 2199.82 = 7800.18"
    );
}

/// Each carry-forward step names the provision and the years it works on:
/// 2022-23's increase applies 2018-19's unused cap first (s 291-20(5)).
#[test]
fn the_carry_forward_steps_work_each_year_earliest_first() {
    let drawn = super_and_cgt_doc("by_year_drawn");
    assert_eq!(
        working_in(&drawn, "unused_cap_2018_19"),
        "max(0, concessional_cap_2018_19 - 5000) = max(0, 25000 - 5000) = 20000"
    );
    assert_eq!(
        working_in(&drawn, "applied_2020_21_in_2022_23"),
        "min(unused_cap_2020_21, 50000 - concessional_cap_2022_23 - applied_2018_19_in_2022_23) = min(10000, 50000 - 27500 - 20000) = 2500"
    );
    assert!(
        authority_in(&drawn, "applied_2020_21_in_2022_23")
            .starts_with("ITAA 1997 s 291-20(3) to (5)")
    );
    assert_eq!(
        working_in(&drawn, "unused_concessional_cap"),
        "carried_forward_2020_21 + carried_forward_2021_22 + carried_forward_2022_23 + carried_forward_2023_24 + carried_forward_2024_25 = 7500 + 0 + 0 + 20000 + 0 = 27500"
    );
    assert_eq!(
        working_in(&drawn, "applied_2023_24_in_2025_26"),
        "min(carried_forward_2023_24, max(0, contributions_with_proposal - concessional_cap) - applied_2020_21_in_2025_26) = min(20000, max(0, 60000 - 30000) - 7500) = 20000"
    );
    assert_eq!(
        working_in(&drawn, "excess_with_proposal"),
        "max(0, contributions_with_proposal - concessional_available) = max(0, 60000 - 57500) = 2500"
    );
    let sam = super_and_cgt_doc("by_year_sam");
    assert_eq!(
        working_in(&sam, "unused_cap_expiring"),
        "remaining_2019_20 = 0 = 0"
    );
}

#[test]
fn a_refused_super_or_cgt_input_gets_the_commands_own_refusal() {
    let refused = |command: &str, input: Value| {
        let plain = api::evaluate_coded(command, &input).unwrap_err();
        let derived = api::evaluate_coded("derive", &json!({"command": command, "input": input}));
        assert_eq!(derived.unwrap_err(), plain);
        plain.code
    };
    let mut input = division_296("{}");
    input["year"] = json!("2027-28");
    assert_eq!(refused("division-296", input), "period.unsupported");
    let mut input = caps("{}");
    input["under_75_in_year"] = json!("yes");
    assert_eq!(refused("contribution-caps", input), "input.flag_invalid");
    assert_eq!(
        refused("study-loan-repayment", study_loan("86380.50", "2026-27")),
        "input.amount_not_whole_dollars"
    );
    let mut input = capital_gains(CAPITAL_GAINS[0].1);
    input["scope_confirmed"] = json!(false);
    assert_eq!(refused("capital-gains", input), "scope.not_confirmed");
    let mut input = main_residence(MAIN_RESIDENCE[2].1);
    input["market_value_basis"] = json!(false);
    assert_eq!(
        refused("main-residence", input),
        "main_residence.first_income_use_rule_applies"
    );
}

/// GST, depreciation and hire purchase cases by name, as (name, command,
/// input): the GST Act's s 9-75(1) example and a price GST does not divide
/// exactly; the ATO's depreciation examples ('Prime cost (straight line) and
/// diminishing value methods', last updated 28 August 2026, read on 16
/// November 2026 at 05:47 AEDT) and each limit on the decline; and the hire
/// purchase agreements `tests/hire_purchase.rs` works by hand.
fn last_kinds_cases() -> Vec<(&'static str, &'static str, Value)> {
    let gst = |amount: &str, inclusive: bool| {
        json!({"amount": amount, "gst_inclusive": inclusive, "scope_confirmed": true,
               "year": "2026-27"})
    };
    let depreciation = |facts: Value| {
        let mut input = json!({
            "cost": "80000.00", "effective_life": "5", "days": 365, "taxable_use": "1",
            "method": "diminishing_value", "scope_confirmed": true, "year": "2026-27",
        });
        merge(&mut input, facts);
        input
    };
    let hire_purchase = |facts: Value| {
        let mut input = json!({
            "amount_financed": "45000.00", "instalment": "850.00", "instalments": 60,
            "balloon": "5000.00", "timing": "in_arrears", "start_date": "2026-07-15",
            "year": "2026-27", "scope_confirmed": true,
        });
        merge(&mut input, facts);
        input
    };
    vec![
        // s 9-75(1): a car sold for $22,000 has a value of $20,000 and $2,000 of GST.
        ("car", "gst", gst("22000.00", true)),
        ("car_value", "gst", gst("20000.00", false)),
        ("eleventh", "gst", gst("100.00", true)),
        // $80,000 over 5 years: diminishing value $32,000, then $19,200 on the
        // $48,000 base; prime cost $16,000; and year 2 at 60% taxable use.
        ("dv_year_1", "depreciation", depreciation(json!({}))),
        (
            "dv_year_2",
            "depreciation",
            depreciation(json!({"opening_adjustable_value": "48000.00"})),
        ),
        (
            "pc_year_1",
            "depreciation",
            depreciation(json!({"method": "prime_cost"})),
        ),
        (
            "dv_private",
            "depreciation",
            depreciation(json!({"opening_adjustable_value": "48000.00", "taxable_use": "0.6"})),
        ),
        // Paul's fridge: $2,000 for 122 days over 10 years, $67 to the dollar.
        (
            "fridge",
            "depreciation",
            depreciation(
                json!({"cost": "2000.00", "effective_life": "10", "days": 122,
                                "method": "prime_cost"}),
            ),
        ),
        // s 40-75(7)(b): year 6 of the part-year asset, $16,000 limited to $9,600.
        (
            "pc_limited",
            "depreciation",
            depreciation(
                json!({"method": "prime_cost", "opening_adjustable_value": "9600.00",
                                "taxable_use": "0.5"}),
            ),
        ),
        // s 40-72(3): $1,000 over a 1-year life gives $2,000, limited to $1,000.
        (
            "dv_limited",
            "depreciation",
            depreciation(json!({"effective_life": "1", "opening_adjustable_value": "1000.00"})),
        ),
        // s 40-72(3) in the start year: a cost written off at once.
        (
            "dv_limited_start",
            "depreciation",
            depreciation(json!({"cost": "500.00", "effective_life": "1.5"})),
        ),
        // Exactly 10% a month, in arrears and in advance, and the default agreement.
        (
            "ten_percent",
            "hire-purchase-schedule",
            hire_purchase(json!({"amount_financed": "1000.00", "instalment": "100.00",
                                 "instalments": 2, "balloon": "1000.00"})),
        ),
        (
            "ten_percent_in_advance",
            "hire-purchase-schedule",
            hire_purchase(json!({"amount_financed": "1000.00", "instalment": "100.00",
                                 "instalments": 2, "balloon": "979.00", "timing": "in_advance"})),
        ),
        (
            "five_years",
            "hire-purchase-schedule",
            hire_purchase(json!({})),
        ),
    ]
}

fn last_kinds_doc(name: &str) -> Value {
    let cases = last_kinds_cases();
    let (_, command, input) = cases.iter().find(|(n, ..)| *n == name).unwrap();
    derive(command, input)
}

#[test]
fn each_gst_depreciation_and_hire_purchase_case_derives_its_plain_result() {
    for (name, command, input) in last_kinds_cases() {
        let doc = derive(command, &input);
        let plain = api::evaluate(command, &input).unwrap();
        assert_eq!(doc["result"].to_string(), plain.to_string(), "{name}");
        let found = problems(&doc);
        assert!(found.is_empty(), "{name}: {found:#?}");
    }
}

/// s 9-75(1)'s car: $22,000 x 10 / 11 is $20,000, and 10% of it $2,000.
#[test]
fn the_gst_steps_show_the_acts_example() {
    let car = last_kinds_doc("car");
    assert_eq!(
        working_in(&car, "gst"),
        "amount / 11 = 22000.00 / 11 = 2000.00"
    );
    assert_eq!(
        working_in(&car, "exclusive"),
        "amount - gst = 22000.00 - 2000.00 = 20000.00"
    );
    assert!(authority_in(&car, "gst").starts_with("GST Act s 9-70 and s 9-75(1)"));
    let value = last_kinds_doc("car_value");
    assert!(working_in(&value, "gst").starts_with("amount / 10 = 20000.00 / 10 = "));
    assert_eq!(value["result"]["amounts"]["gst"], "2000.00");
    assert_eq!(value["result"]["amounts"]["inclusive"], "22000.00");
    assert!(authority_in(&value, "gst").starts_with("GST Act s 9-70:"));
    // The value is the price less the unrounded GST, so the 2 amounts shown
    // still add to the price.
    let eleventh = last_kinds_doc("eleventh");
    let amounts = &eleventh["result"]["amounts"];
    assert_eq!([&amounts["gst"], &amounts["exclusive"]], ["9.09", "90.91"]);
}

#[test]
fn the_depreciation_steps_show_the_atos_examples() {
    let dv = last_kinds_doc("dv_year_1");
    assert!(working_in(&dv, "decline_in_value").starts_with(
        "min(cost, cost x days x 2 / (365 x effective_life)) \
             = min(80000.00, 80000.00 x 365 x 2 / (365 x 5)) = "
    ));
    assert_eq!(dv["result"]["amounts"]["decline_in_value"], "32000.00");
    assert!(
        working_in(&dv, "closing_adjustable_value")
            .starts_with("cost - cents(decline_in_value) = 80000.00 - cents(")
    );
    assert_eq!(
        dv["result"]["amounts"]["closing_adjustable_value"],
        "48000.00"
    );
    let year_2 = last_kinds_doc("dv_year_2");
    assert!(
        working_in(&year_2, "decline_in_value").starts_with(
            "min(opening_adjustable_value, opening_adjustable_value x days x 2 / (365 x effective_life)) \
             = min(48000.00, 48000.00 x 365 x 2 / (365 x 5))"
        )
    );
    assert_eq!(year_2["result"]["amounts"]["decline_in_value"], "19200.00");
    let pc = last_kinds_doc("pc_year_1");
    assert!(
        working_in(&pc, "decline_in_value")
            .starts_with("min(cost, cost x days / (365 x effective_life))")
    );
    assert_eq!(pc["result"]["amounts"]["decline_in_value"], "16000.00");
    // At 40% private use 'all figures would be multiplied by a factor of 0.6':
    // $19,200 x 0.6.
    let private = last_kinds_doc("dv_private");
    assert!(working_in(&private, "deduction").starts_with(
        "opening_adjustable_value x days x 2 x taxable_use / (365 x effective_life) \
             = 48000.00 x 365 x 2 x 0.6 / (365 x 5)"
    ));
    assert_eq!(private["result"]["amounts"]["deduction"], "11520.00");
    assert_eq!(
        private["result"]["amounts"]["closing_adjustable_value"],
        "28800.00"
    );
    // $2,000 x 122 / 365 x 10%: $66.85, which the ATO shows as $67.
    let fridge = last_kinds_doc("fridge");
    assert_eq!(fridge["result"]["amounts"]["deduction"], "66.85");
}

/// Each method and year cites its formula, and a limited decline its limit.
#[test]
fn each_depreciation_step_cites_its_method_and_limit() {
    let decline = |name: &str| authority_in(&last_kinds_doc(name), "decline_in_value");
    assert!(decline("dv_year_1").contains("the cost in the start year (s 40-70(1)(a))"));
    assert!(
        decline("dv_year_2")
            .contains("the opening adjustable value in a later year (s 40-70(1)(b))")
    );
    assert!(decline("pc_year_1").ends_with("no more than cost (s 40-75(7)(a))"));
    assert!(decline("pc_limited").ends_with("(s 40-75(7)(b))"));
    let limited = last_kinds_doc("pc_limited");
    assert_eq!(
        working_in(&limited, "decline_in_value"),
        "min(opening_adjustable_value, cost x days / (365 x effective_life)) \
         = min(9600.00, 80000.00 x 365 / (365 x 5)) = 9600.00"
    );
    assert_eq!(
        working_in(&limited, "deduction"),
        "decline_in_value x taxable_use = 9600.00 x 0.5 = 4800.000"
    );
    let dv_limited = last_kinds_doc("dv_limited");
    assert_eq!(
        dv_limited["result"]["amounts"]["decline_in_value"],
        "1000.00"
    );
    assert!(working_in(&dv_limited, "deduction").starts_with("decline_in_value x taxable_use"));
    let start = last_kinds_doc("dv_limited_start");
    assert_eq!(
        working_in(&start, "decline_in_value"),
        "min(cost, cost x days x 2 / (365 x effective_life)) \
         = min(500.00, 500.00 x 365 x 2 / (365 x 1.5)) = 500.00"
    );
    assert!(authority_in(&start, "deduction").starts_with("ITAA 1997 s 40-25(1) and (2)"));
}

/// At exactly 10% a month the interest is each $100 instalment in arrears,
/// and $90 then $89 in advance (tests/hire_purchase.rs).
#[test]
fn the_hire_purchase_steps_total_the_payments_less_the_principal() {
    let arrears = last_kinds_doc("ten_percent");
    assert_eq!(
        working_in(&arrears, "total_payments"),
        "instalment x instalments + balloon = 100.00 x 2 + 1000.00 = 1200.00"
    );
    assert_eq!(
        working_in(&arrears, "total_interest"),
        "total_payments - amount_financed = 1200.00 - 1000.00 = 200.00"
    );
    assert!(authority_in(&arrears, "amount_financed").starts_with("ITAA 1997 s 240-25(3)"));
    let advance = last_kinds_doc("ten_percent_in_advance");
    assert_eq!(advance["result"]["amounts"]["total_interest"], "179.00");
    // The schedule's monthly interest adds up to the total the step works out.
    let five_years = last_kinds_doc("five_years");
    let monthly = five_years["result"]["schedule"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| dec(row["interest"].as_str().unwrap()))
        .fold(dec("0"), |sum, interest| sum.add(&interest).unwrap());
    assert_eq!(
        monthly.to_string(),
        five_years["result"]["amounts"]["total_interest"]
    );
    assert_eq!(
        working_in(&five_years, "total_interest"),
        "total_payments - amount_financed = 56000.00 - 45000.00 = 11000.00"
    );
}

#[test]
fn a_refused_gst_depreciation_or_hire_purchase_input_gets_the_commands_own_refusal() {
    let refused = |command: &str, input: Value| {
        let plain = api::evaluate_coded(command, &input).unwrap_err();
        let derived = api::evaluate_coded("derive", &json!({"command": command, "input": input}));
        assert_eq!(derived.unwrap_err(), plain);
        plain.code
    };
    let cases = last_kinds_cases();
    let input = |name: &str| cases.iter().find(|(n, ..)| *n == name).unwrap().2.clone();
    let mut gst = input("car");
    gst["gst_inclusive"] = json!("yes");
    assert_eq!(refused("gst", gst), "input.flag_invalid");
    let mut depreciation = input("dv_year_1");
    depreciation["days"] = json!(366);
    assert_eq!(
        refused("depreciation", depreciation),
        "input.day_count_invalid"
    );
    let mut depreciation = input("dv_year_2");
    depreciation["opening_adjustable_value"] = json!("80000.01");
    assert_eq!(
        refused("depreciation", depreciation),
        "input.opening_value_above_cost"
    );
    let mut hire_purchase = input("five_years");
    hire_purchase["instalments"] = json!(0);
    assert_eq!(
        refused("hire-purchase-schedule", hire_purchase),
        "hire_purchase.instalments_unsupported"
    );
    let mut hire_purchase = input("five_years");
    hire_purchase["balloon"] = json!("0.00");
    hire_purchase["instalment"] = json!("100.00");
    assert_eq!(
        refused("hire-purchase-schedule", hire_purchase),
        "hire_purchase.rate_unsupported"
    );
}
