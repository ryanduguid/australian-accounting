// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! JSON boundary for the removed Python import API. Each command takes the
//! original function's keyword arguments as one JSON object: amounts and
//! ratios as decimal strings, the rest as JSON booleans, integers or strings.
//! Values of another type reach the engine, which refuses them as the
//! original's own type checks did.
use crate::decimal::Decimal;
use crate::engine::{self, Arg};
use serde_json::Value;
use std::collections::HashSet;

/// `contract-register`, a command over a contracts CSV rather than one JSON
/// object; see [`crate::contract_register`].
pub use crate::contract_register::command as contract_register;

/// A refusal at the JSON boundary: its stable code and the message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub code: &'static str,
    pub message: String,
}

impl Refusal {
    pub fn new(code: &'static str, message: impl Into<String>) -> Refusal {
        Refusal {
            code,
            message: message.into(),
        }
    }
}

/// The first key an object in `bytes` repeats, which decoding into `Value`
/// resolves silently by keeping the last value. `bytes` must already have
/// decoded as JSON; each key is compared decoded, so escapes cannot hide a repeat.
/// This command and `au-accounting tax` both refuse on it, before the worksheet
/// sees either value, so a repeat is refused whatever the values are.
pub fn repeated_key(bytes: &[u8]) -> Option<String> {
    // One entry per open container: an object's keys so far, or None for an array.
    let mut open: Vec<Option<HashSet<String>>> = Vec::new();
    let mut key_next = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                let start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                if key_next && let Some(Some(keys)) = open.last_mut() {
                    let literal = bytes.get(start..=i).unwrap_or_default();
                    if let Ok(key) = serde_json::from_slice::<String>(literal)
                        && !keys.insert(key.clone())
                    {
                        return Some(key);
                    }
                }
                key_next = false;
            }
            b'{' => {
                open.push(Some(HashSet::new()));
                key_next = true;
            }
            b'[' => {
                open.push(None);
                key_next = false;
            }
            b'}' | b']' => {
                open.pop();
                key_next = false;
            }
            b',' => key_next = matches!(open.last(), Some(Some(_))),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Every refusal code and the failure it names.
pub fn refusal_codes() -> &'static [(&'static str, &'static str)] {
    crate::texts::REFUSAL_CODES
}

fn refusal<T>(code: &'static str, message: impl Into<String>) -> Result<T> {
    Err(Refusal::new(code, message))
}

type Result<T> = std::result::Result<T, Refusal>;

/// Argument kinds: a decimal string, a JSON value the engine reads as JSON
/// (lists of periods), or any other JSON value.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dec,
    Json,
    Raw,
}
use Kind::{Dec, Json, Raw};

/// The FBT benefit worksheets' positional parameters. Their other inputs depend
/// on the method, so [`benefit_inputs`] passes them by name.
const FBT_YEAR_ARGS: &[(&str, Kind)] = &[("year_ended", Raw), ("scope_confirmed", Raw)];
const FBT_METHOD_ARGS: &[(&str, Kind)] = &[
    ("method", Raw),
    ("year_ended", Raw),
    ("scope_confirmed", Raw),
];

/// The parameters of the worksheets that take one pay's earnings on a scale.
const PAY_ARGS: &[(&str, Kind)] = &[
    ("earnings", Dec),
    ("pay_period", Raw),
    ("scale", Raw),
    ("year", Raw),
    ("scope_confirmed", Raw),
];

/// Schedule 3's parameters and Schedule 2's after the first 2. The daily and
/// casual table takes those the 2 schedules share.
const PAYEE_ARGS: &[(&str, Kind)] = &[
    ("performances_per_week", Raw),
    ("performances_per_day", Raw),
    ("earnings", Dec),
    ("tfn_quoted", Raw),
    ("foreign_resident", Raw),
    ("tax_free_threshold_claimed", Raw),
    ("working_holiday_maker", Raw),
    ("year", Raw),
    ("scope_confirmed", Raw),
    ("labour_mobility_scheme", Raw),
];

/// The parameters of the worksheets that take taxable income alone.
const TAXABLE_INCOME_ARGS: &[(&str, Kind)] = &[
    ("taxable_income", Dec),
    ("year", Raw),
    ("scope_confirmed", Raw),
];

/// Command, the worksheet's parameters in the original order.
const COMMAND_ARGS: &[(&str, &[(&str, Kind)])] = &[
    (
        "gst",
        &[
            ("amount", Dec),
            ("gst_inclusive", Raw),
            ("scope_confirmed", Raw),
            ("year", Raw),
        ],
    ),
    (
        "fuel-tax-credits",
        &[("lines", Json), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "alcohol-excise",
        &[("lines", Json), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "gst-margin-scheme",
        &[
            ("interest", Raw),
            ("agreement", Raw),
            ("acquisition", Raw),
            ("consideration", Dec),
            ("method", Raw),
            ("recipient_associate", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    ("resident-tax", TAXABLE_INCOME_ARGS),
    ("foreign-resident-tax", TAXABLE_INCOME_ARGS),
    (
        "capital-gains",
        &[
            ("other_gains", Dec),
            ("discount_gains", Dec),
            ("current_losses", Dec),
            ("prior_losses", Dec),
            ("scope_confirmed", Raw),
            ("year", Raw),
        ],
    ),
    (
        "main-residence",
        &[
            ("capital_gain", Dec),
            ("capital_loss", Dec),
            ("market_value_basis", Raw),
            ("ownership_start", Raw),
            ("ownership_end", Raw),
            ("lived_periods", Json),
            ("income_use_periods", Json),
            ("absence_choice_periods", Json),
            ("new_home_acquired", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "borrowing-expenses",
        &[
            ("borrowing_expenses", Dec),
            ("other_borrowing_expenses_in_year", Dec),
            ("income_use", Dec),
            ("loan_start", Raw),
            ("loan_term_end", Json),
            ("repaid_on", Json),
            ("section_26_50_applies", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "borrowing-expenses-schedule",
        &[("loans", Json), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "hire-purchase-schedule",
        &[
            ("amount_financed", Dec),
            ("balloon", Dec),
            ("timing", Raw),
            ("start_date", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "chattel-mortgage-schedule",
        &[
            ("amount_financed", Dec),
            ("instalments", Raw),
            ("balloon", Dec),
            ("timing", Raw),
            ("start_date", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "loan-transactions",
        &[
            ("opening_balance", Dec),
            ("annual_rate", Dec),
            ("start_date", Raw),
            ("end_date", Raw),
            ("transactions", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "business-capital-expenditure",
        &[("items", Json), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "cgt-parcels",
        &[
            ("entity", Raw),
            ("identification", Raw),
            ("parcels", Json),
            ("disposals", Json),
            ("other_discount_capital_gains", Dec),
            ("other_capital_gains", Dec),
            ("other_capital_losses", Dec),
            ("prior_net_capital_losses", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "cgt-cost-base-adjustments",
        &[
            ("entity", Raw),
            ("parcels", Json),
            ("events", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "balancing-adjustment",
        &[
            ("cost", Dec),
            ("termination_value", Dec),
            ("asset", Raw),
            ("years", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "franking-account",
        &[
            ("private_company", Raw),
            ("benchmark_rule_exempt", Raw),
            ("corporate_tax_rate", Dec),
            ("opening_balance", Dec),
            ("rd_deferred_debit_brought_forward", Dec),
            ("last_benchmark", Json),
            ("events", Json),
            ("distributions", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "dasp-tax",
        &[
            ("tax_free_component", Dec),
            ("taxed_element", Dec),
            ("untaxed_element", Dec),
            ("working_holiday_maker_contributions", Raw),
            ("unclaimed_money_rollover", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "super-withholding",
        &[
            ("benefit", Dec),
            ("interest_value", Dec),
            ("interest_tax_free_component", Dec),
            ("untaxed_element", Dec),
            ("recipient", Raw),
            ("payment_date", Raw),
            ("tfn_provided", Raw),
            ("foreign_resident", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "super-benefit-tax",
        &[
            ("benefit", Dec),
            ("interest_value", Dec),
            ("interest_tax_free_component", Dec),
            ("untaxed_element", Dec),
            ("recipient", Raw),
            ("payment_date", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "trade-in",
        &[
            ("cost", Dec),
            ("asset", Raw),
            ("years", Json),
            ("trade_in_allowance", Dec),
            ("trade_in_taxable", Raw),
            ("replacement_price", Dec),
            ("replacement_asset", Raw),
            ("replacement_creditable_acquisition", Raw),
            ("arms_length", Raw),
            ("involuntary_disposal", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "payroll-tax",
        &[
            ("jurisdiction", Raw),
            ("taxable_wages", Dec),
            ("australian_wages", Dec),
            ("group_member", Raw),
            ("employed_all_year", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "wine-equalisation-tax",
        &[("dealings", Json), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "fbt",
        &[
            ("type_one_value", Dec),
            ("type_two_value", Dec),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "fbt-car-statutory",
        &[
            ("cost_price", Dec),
            ("later_accessories", Dec),
            ("one_third_reduction", Raw),
            ("days_available", Raw),
            ("employee_contribution", Dec),
            ("electric_car_exempt", Raw),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "fbt-car-operating-cost",
        &[
            ("running_costs", Dec),
            ("lease_charges", Dec),
            ("depreciated_value", Dec),
            ("effective_life_years", Dec),
            ("days_held", Raw),
            ("business_use", Dec),
            ("employee_contribution", Dec),
            ("statutory_formula_taxable_value", Dec),
            ("electric_car_exempt", Raw),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "fbt-ev-exemption-check",
        &[
            ("vehicle", Raw),
            ("car", Raw),
            ("current_employee", Raw),
            ("first_held_and_used", Raw),
            ("first_retail_sale", Raw),
            ("luxury_car_tax_value", Dec),
            ("luxury_car_tax_on_other_supply", Raw),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    ("fbt-loan", FBT_YEAR_ARGS),
    ("fbt-meal-entertainment", FBT_METHOD_ARGS),
    ("fbt-car-parking", FBT_METHOD_ARGS),
    ("fbt-debt-waiver", FBT_YEAR_ARGS),
    ("fbt-expense-payment", FBT_METHOD_ARGS),
    ("fbt-property", FBT_METHOD_ARGS),
    (
        "depreciation",
        &[
            ("cost", Dec),
            ("effective_life", Dec),
            ("days", Raw),
            ("taxable_use", Dec),
            ("method", Raw),
            ("scope_confirmed", Raw),
            ("year", Raw),
        ],
    ),
    (
        "depreciation-low-value-pool",
        &[
            ("previous_closing_pool_balance", Dec),
            ("low_cost_assets", Dec),
            ("low_value_assets", Dec),
            ("second_element_costs", Dec),
            ("termination_values", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "depreciation-small-business-pool",
        &[
            ("opening_pool_balance", Dec),
            ("additions", Dec),
            ("cost_additions", Dec),
            ("termination_values", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "capital-works",
        &[
            ("construction_expenditure", Dec),
            ("construction_start", Raw),
            ("four_percent_manner", Raw),
            ("days_used", Raw),
            ("taxable_use", Dec),
            ("undeducted_construction_expenditure", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "horticulture-shearing-withholding",
        PAYEE_ARGS.split_at(2).1,
    ),
    (
        "upward-variation-withholding",
        &[
            ("earnings", Dec),
            ("pay_period", Raw),
            ("scale", Raw),
            ("additional_amount", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "daily-casual-withholding",
        PAYEE_ARGS.split_at(9).0.split_at(2).1,
    ),
    (
        "no-abn-withholding",
        &[
            ("payment", Dec),
            ("gst_included", Dec),
            ("supply_total_excluding_gst", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    ("entertainer-withholding", PAYEE_ARGS.split_at(9).0),
    (
        "return-to-work-withholding",
        &[
            ("payment", Dec),
            ("tfn_quoted", Raw),
            ("foreign_resident", Raw),
            ("working_holiday_maker", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "annuity-withholding",
        &[
            ("payment", Dec),
            ("deductible_amount", Dec),
            ("instalments", Raw),
            ("pay_period", Raw),
            ("scale", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "voluntary-agreement-withholding",
        &[
            ("payment", Dec),
            ("gst_included", Dec),
            ("agreement_in_force", Raw),
            ("abn_quoted", Raw),
            ("agreement_applies_instalment_rate", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "rd-tax-offset",
        &[
            ("notional_deductions", Dec),
            ("rsp_crc_expenditure", Dec),
            ("aggregated_turnover", Dec),
            ("exempt_entity_controlled", Raw),
            ("base_rate_entity", Raw),
            ("total_expenses", Dec),
            ("recoupment_clawback", Dec),
            ("feedstock_revenue", Dec),
            ("feedstock_expenditure", Dec),
            ("present_year_base_rate_entity", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "quarterly-sg",
        &[
            ("ordinary_time_earnings", Dec),
            ("qualifying_contributions", Dec),
            ("year", Raw),
            ("quarter", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    ("payg-withholding", PAY_ARGS),
    (
        "payg-gross-up",
        &[
            ("net_pay", Dec),
            ("pay_period", Raw),
            ("scale", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    ("study-loan-withholding", PAY_ARGS),
    (
        "medicare-levy",
        &[
            ("taxable_income", Dec),
            ("section_9a_adjustment_applies", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "medicare-levy-surcharge",
        &[
            ("taxable_income", Dec),
            ("reportable_fringe_benefits", Dec),
            ("income_for_surcharge_purposes", Dec),
            ("days_without_cover", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    ("low-income-tax-offset", TAXABLE_INCOME_ARGS),
    (
        "seniors-pensioners-tax-offset",
        &[
            ("rebate_income", Dec),
            ("status", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "beneficiary-tax-offset",
        &[
            ("rebatable_benefit", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "invalid-carer-tax-offset",
        &[
            ("dependant", Raw),
            ("adjusted_taxable_income", Dec),
            ("dependant_adjusted_taxable_income", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "zone-tax-offset",
        &[
            ("area", Raw),
            ("days", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "net-medical-expenses-tax-offset",
        &[
            ("net_medical_expenses", Dec),
            ("adjusted_taxable_income", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "lump-sum-arrears-tax-offset",
        &[
            ("taxable_income", Dec),
            ("net_capital_gain", Dec),
            ("accrual_years", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "division-293",
        &[
            ("division_293_income", Dec),
            ("low_tax_contributions", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "division-296",
        &[
            ("total_super_balance_end_of_year", Dec),
            ("total_super_earnings", Dec),
            ("child_recipient", Raw),
            ("structured_settlement_contribution", Raw),
            ("died_in_year", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "termination-payment",
        &[
            ("redundancy_payment", Dec),
            ("completed_years_of_service", Raw),
            ("excluded_payment", Dec),
            ("non_excluded_payment", Dec),
            ("other_taxable_payments", Dec),
            ("date_of_birth", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "etp-tax-free-component",
        &[
            ("payment", Dec),
            ("employment_start", Raw),
            ("termination_date", Raw),
            ("last_retirement_day", Raw),
            ("ill_health", Raw),
            ("medical_certification", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "unused-leave-payment",
        &[
            ("annual_leave", Dec),
            ("leave_loading", Dec),
            ("long_service_leave", Dec),
            ("leave_accrual_start", Raw),
            ("termination_reason", Raw),
            ("working_holiday_maker", Raw),
            ("earnings", Dec),
            ("pay_period", Raw),
            ("scale", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "schedule-5-method-a",
        &[
            ("earnings", Dec),
            ("additional_payment", Dec),
            ("pay_periods", Raw),
            ("pay_period", Raw),
            ("scale", Raw),
            ("study_loan", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "schedule-5-method-b-i",
        &[
            ("earnings", Dec),
            ("back_payment_periods", Json),
            ("pay_period", Raw),
            ("scale", Raw),
            ("study_loan", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "schedule-5-method-b-ii",
        &[
            ("earnings", Dec),
            ("additional_payment", Dec),
            ("year_to_date_earnings", Dec),
            ("pay_periods_to_date", Raw),
            ("earlier_additional_payments", Dec),
            ("earlier_additional_withholding", Dec),
            ("pay_period", Raw),
            ("scale", Raw),
            ("study_loan", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "working-holiday-maker-withholding",
        &[
            ("payment", Dec),
            ("payments_before", Dec),
            ("employer_registered", Raw),
            ("tfn_quoted", Raw),
            ("pay_period", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "seniors-pensioners-withholding",
        &[
            ("earnings", Dec),
            ("pay_period", Raw),
            ("status", Raw),
            ("medicare_levy_exemption", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "study-loan-repayment",
        &[
            ("repayment_income", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "contribution-caps",
        &[
            ("total_super_balance", Dec),
            ("concessional_contributions", Dec),
            ("unused_concessional_cap", Dec),
            ("non_concessional_contributions", Dec),
            ("under_75_in_year", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "pension-minimum",
        &[
            ("account_balance", Dec),
            ("age", Raw),
            ("days", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    ("fbt-residual", FBT_METHOD_ARGS),
    ("fbt-housing", FBT_METHOD_ARGS),
    ("fbt-lafha", FBT_METHOD_ARGS),
    ("fbt-board", FBT_YEAR_ARGS),
    ("fbt-tax-exempt-body-entertainment", FBT_METHOD_ARGS),
    (
        "fbt-meal-entertainment-register",
        &[
            ("register_start", Raw),
            ("register_end", Raw),
            ("entries", Json),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "ppsr-timing",
        &[("question", Raw), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "fbt-minor-benefit-check",
        &[
            ("category", Raw),
            ("notional_taxable_value", Dec),
            ("salary_sacrifice", Raw),
            ("frequency", Raw),
            ("similar_benefits_substantial", Raw),
            ("other_associated_benefits_substantial", Raw),
            ("valuation_difficult", Raw),
            ("unexpected_event", Raw),
            ("reward_for_services", Raw),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "fhss",
        &[
            ("determination_date", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "fbt-entertainment-classifier",
        &[
            ("lines", Json),
            ("income_tax_exempt", Raw),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "luxury-car-tax",
        &[("lines", Json), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "ato-interest-penalties",
        &[("question", Raw), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "transfer-balance",
        &[
            ("events", Json),
            ("to", Raw),
            ("previously_liable", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "excess-contributions",
        &[("question", Raw), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "rental-property",
        &[
            ("ownership_share", Dec),
            ("acquired", Raw),
            ("disposed", Json),
            ("days_available", Raw),
            ("section_26_50_applies", Raw),
            ("income", Json),
            ("expenses", Json),
            ("rental_only_expenses", Json),
            ("loans", Json),
            ("borrowing_loans", Json),
            ("capital_works", Json),
            ("depreciating_assets", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "sb-cgt-concessions",
        &[
            ("entity", Raw),
            ("share_or_trust_interest", Raw),
            ("acquired", Raw),
            ("event_date", Raw),
            ("partnership_asset", Raw),
            ("passively_held", Raw),
            ("business", Raw),
            ("previous_year_turnover", Dec),
            ("year_before_turnover", Dec),
            ("likely_turnover", Dec),
            ("year_turnover", Dec),
            ("net_asset_value", Dec),
            ("active_days", Raw),
            ("business_ceased", Raw),
            ("date_of_birth", Raw),
            ("retirement", Raw),
            ("permanently_incapacitated", Raw),
            ("last_rollover_event", Raw),
            ("earnout_latest_due", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "small-business-income-offset",
        &[
            ("taxable_income", Dec),
            ("basic_income_tax_liability", Dec),
            ("offsets_applied_first", Dec),
            ("net_small_business_income", Dec),
            ("shares", Json),
            ("prescribed_person", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "trading-stock",
        &[
            ("entity", Raw),
            ("opening_value", Dec),
            ("items", Json),
            ("estimated_closing_value", Dec),
            ("simplified_choice", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "prepayments",
        &[
            ("taxpayer", Raw),
            ("individual", Raw),
            ("business_expenditure", Raw),
            ("chose_82kzmd", Raw),
            ("expenditure", Dec),
            ("incurred", Raw),
            ("service_start", Raw),
            ("service_end", Raw),
            ("agreement_date", Raw),
            ("exclusion", Raw),
            ("tax_shelter", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "non-commercial-losses",
        &[
            ("activity", Raw),
            ("assessable_income", Dec),
            ("deductions", Dec),
            ("brought_forward", Dec),
            ("part_year", Raw),
            ("earlier_years", Json),
            ("real_property_value", Dec),
            ("other_assets_value", Dec),
            ("taxable_income", Dec),
            ("business_losses", Dec),
            ("assessable_fhss_released_amount", Dec),
            ("reportable_fringe_benefits", Dec),
            ("reportable_super_contributions", Dec),
            ("total_net_investment_loss", Dec),
            ("net_exempt_income", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "first-home-owner-grant",
        &[
            ("jurisdiction", Raw),
            ("contract_date", Raw),
            ("home", Raw),
            ("total_value", Dec),
            ("facts", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "psi-test",
        &[
            ("earner", Raw),
            ("mainly_personal_efforts", Raw),
            ("determination_in_force", Raw),
            ("payers", Json),
            ("agent", Raw),
            ("principal_work_by_others", Dec),
            ("apprentice", Raw),
            ("premises", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "smsf-tax",
        &[
            ("complying_fund", Raw),
            ("australian_superannuation_fund", Raw),
            ("assessable_contributions", Dec),
            ("no_tfn_contributions", Dec),
            ("net_non_arms_length_income", Dec),
            ("general_expense_non_arms_length_income", Dec),
            ("segregated_pension_income", Dec),
            ("unsegregated_income", Dec),
            ("segregated_non_current_income", Dec),
            ("exempt_proportion", Dec),
            ("disregarded_small_fund_assets", Raw),
            ("deductions", Dec),
            ("contribution_deductions", Dec),
            ("non_refundable_offsets", Dec),
            ("carry_forward_offsets", Dec),
            ("refundable_offsets", Dec),
            ("eligible_credits", Dec),
            ("payg_instalments", Dec),
            ("interest_charge", Dec),
            ("newly_registered", Raw),
            ("wound_up", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "work-expenses",
        &[
            ("method", Raw),
            ("reimbursed", Raw),
            ("allowance_received", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "private-health-insurance-rebate",
        &[
            ("income_for_surcharge_purposes", Dec),
            ("lines", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "residency-tests",
        &[
            ("resides", Raw),
            ("domicile_in_australia", Raw),
            ("permanent_place_of_abode_outside", Raw),
            ("presence", Json),
            ("usual_place_of_abode_outside", Raw),
            ("intends_to_take_up_residence", Raw),
            ("commonwealth_super", Raw),
            ("resident_elsewhere", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "lodgment-required",
        &[
            ("residency", Json),
            ("taxable_income", Dec),
            ("dividends_and_franking_credits", Dec),
            ("under_18", Json),
            ("triggers", Json),
            ("study_loan", Json),
            ("working_holiday_maker", Json),
            ("listed_payment", Json),
            ("sapto", Json),
            ("child_support_parent", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "gst-creditable-purpose-change",
        &[
            ("gst_exclusive_value", Dec),
            ("relates_to_business_finance", Raw),
            ("full_input_tax_credit", Dec),
            ("intended_or_former_application", Dec),
            ("actual_application", Dec),
            ("application_basis", Raw),
            ("attributed_period_end", Raw),
            ("tax_period_months", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "gst-annual-apportionment",
        &[
            ("small_business_entity", Raw),
            ("carries_on_business", Raw),
            ("instalment_election", Raw),
            ("annual_tax_period_election", Raw),
            ("election_made", Raw),
            ("gst_on_acquisition", Dec),
            ("creditable_purpose", Dec),
            ("car", Raw),
            ("reduced_credit_acquisition", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "gst-financial-acquisitions-threshold",
        &[
            ("current_financial_credits", Dec),
            ("current_total_credits", Dec),
            ("future_financial_credits", Dec),
            ("future_total_credits", Dec),
            ("gst_group", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "gst-going-concern",
        &[
            ("for_consideration", Raw),
            ("recipient_registration", Raw),
            ("written_agreement", Raw),
            ("all_things_necessary", Raw),
            ("carried_on_until_supply", Raw),
            ("farm_land", Raw),
            ("part_outside_arrangement", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "gst-second-hand-goods",
        &[
            ("consideration", Dec),
            ("acquired_for_sale_or_exchange", Raw),
            ("animals_or_plants", Raw),
            ("supply_to_you", Raw),
            ("imported", Raw),
            ("by_way_of_hire", Raw),
            ("resupply_taxable", Raw),
            ("divided_for_resupply", Raw),
            ("valuable_metal", Raw),
            ("leased_before_sale", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "gst-bad-debt",
        &[
            ("party", Raw),
            ("event", Raw),
            ("amount", Dec),
            ("taxable_or_creditable", Raw),
            ("cash_basis", Raw),
            ("special_rule", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "deferred-tax",
        &[
            ("items", Json),
            ("unused_tax_losses", Dec),
            ("opening_unused_tax_losses", Dec),
            ("unrecognised_deductible", Dec),
            ("opening_unrecognised_deductible", Dec),
            ("unrecognised_used", Dec),
            ("accounting_profit", Dec),
            ("non_deductible_expenses", Dec),
            ("non_assessable_income", Dec),
            ("taxable_income", Dec),
            ("tax_offsets", Dec),
            ("base_rate_entity", Raw),
            ("expected_rate", Raw),
            ("opening_expected_rate", Raw),
            ("business_combination", Raw),
            ("uncertain_tax_treatment", Raw),
            ("tax_consolidated_group", Raw),
            ("outside_profit_or_loss", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "foreign-income-conversion",
        &[("items", Json), ("year", Raw), ("scope_confirmed", Raw)],
    ),
    (
        "foreign-income-tax-offset",
        &[
            ("foreign_tax_paid", Dec),
            ("nane_foreign_tax", Dec),
            ("limit_method", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "income-tests",
        &[
            ("taxable_income", Dec),
            ("assessable_fhss_released_amount", Dec),
            ("family_trust_distribution_amounts", Dec),
            ("reportable_fringe_benefits_section_57a", Dec),
            ("reportable_fringe_benefits_other", Dec),
            ("reportable_employer_super_contributions", Dec),
            ("deductible_personal_super_contributions", Dec),
            ("excess_concessional_contributions", Dec),
            ("net_financial_investment_loss", Dec),
            ("net_rental_property_loss", Dec),
            ("tax_free_government_pensions", Dec),
            ("target_foreign_income", Dec),
            ("exempt_foreign_employment_income", Dec),
            ("child_support_paid", Dec),
            ("assessable_income", Dec),
            ("business_deductions", Dec),
            ("residency", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "government-super-amounts",
        &[
            ("taxable_income", Dec),
            ("assessable_fhss_released_amount", Dec),
            ("family_trust_distribution_amounts", Dec),
            ("reportable_fringe_benefits_section_57a", Dec),
            ("reportable_fringe_benefits_other", Dec),
            ("reportable_employer_super_contributions", Dec),
            ("deductible_personal_super_contributions", Dec),
            ("excess_concessional_contributions", Dec),
            ("net_financial_investment_loss", Dec),
            ("net_rental_property_loss", Dec),
            ("tax_free_government_pensions", Dec),
            ("target_foreign_income", Dec),
            ("exempt_foreign_employment_income", Dec),
            ("child_support_paid", Dec),
            ("assessable_income", Dec),
            ("business_deductions", Dec),
            ("residency", Raw),
            ("eligible_income", Dec),
            ("personal_contributions", Dec),
            ("concessional_contributions", Dec),
            ("return_lodged", Raw),
            ("under_71", Raw),
            ("temporary_visa_held", Raw),
            ("non_concessional_within_cap", Raw),
            ("total_super_balance_below_cap", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "minors-income",
        &[
            ("taxable_income", Dec),
            ("residency", Raw),
            ("under_18_on_30_june", Raw),
            ("excepted_person", Raw),
            ("excepted_income", Json),
            ("trust_shares", Json),
            ("other_assessable_income", Dec),
            ("deductions_exclusive", Dec),
            ("deductions_related", Dec),
            ("apportionable_deductions", Dec),
            ("special_income_component", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
];

const LICENCE_NOTICE: &str = "\n\
    Copyright 2026 Ryan Duguid. No warranty.\n\
    AGPL-3.0-or-later with additional terms (see NOTICE).\n\
    Licence: https://github.com/ryanduguid/australian-accounting/blob/main/packages/australian-tax-calculators/LICENSE\n";

/// Describe the JSON commands and the licence for this executable.
#[must_use]
pub fn cli_help() -> String {
    format!(
        "JSON stdin commands: {}\nAmounts are decimal strings. See README.md for schemas.\n{LICENCE_NOTICE}",
        commands().join(", ")
    )
}

pub fn commands() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = COMMAND_ARGS.iter().map(|(c, _)| *c).collect();
    out.push("worksheet-catalogue");
    out.push("periods");
    out.push("derive");
    out.push("personal-tax-estimate");
    out.push("dgr-check");
    out.push("fbt-return");
    out
}

/// The commands `derive` explains: each of their amounts is worked out in a step.
/// Every worksheet command is here except `payg-gross-up` and
/// `fbt-entertainment-classifier`. The gross pay of `payg-gross-up` is the
/// one whose own withholding leaves the net, found by searching
/// `payg-withholding`'s results, or `study-loan-withholding`'s for a payee with
/// a study loan: the withholding is worked on the gross, so no step can work
/// the gross out from the inputs and earlier steps alone. `derive` of that
/// command on its `gross_pay` shows that withholding. The classifier's amounts
/// are totals over lines whose shares a decision table sets, not a formula;
/// each line and class carries the authority for its row instead.
const DERIVED: [&str; 80] = [
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
    "depreciation-low-value-pool",
    "depreciation-small-business-pool",
    "capital-works",
    "deferred-tax",
    "horticulture-shearing-withholding",
    "upward-variation-withholding",
    "daily-casual-withholding",
    "no-abn-withholding",
    "entertainer-withholding",
    "return-to-work-withholding",
    "annuity-withholding",
    "voluntary-agreement-withholding",
    "rd-tax-offset",
    "hire-purchase-schedule",
    "chattel-mortgage-schedule",
    "loan-transactions",
    "lump-sum-arrears-tax-offset",
    "income-tests",
    "government-super-amounts",
    "smsf-tax",
    "business-capital-expenditure",
    "balancing-adjustment",
    "trade-in",
    "dasp-tax",
    "super-benefit-tax",
    "super-withholding",
    "gst-margin-scheme",
    "etp-tax-free-component",
    "payroll-tax",
    "cgt-parcels",
    "work-expenses",
    "rental-property",
    "gst-creditable-purpose-change",
    "gst-annual-apportionment",
    "gst-financial-acquisitions-threshold",
    "gst-second-hand-goods",
    "gst-bad-debt",
    "cgt-cost-base-adjustments",
    "foreign-income-tax-offset",
];

/// `derive`: `{"command", "input"}` gives the command's result with the step
/// behind each amount. A refused input gives the command's own refusal.
fn derive(v: &Value) -> Result<Value> {
    let mut keys = v.as_object().into_iter().flat_map(|o| o.keys());
    if let Some(key) = keys.find(|k| !matches!(k.as_str(), "command" | "input")) {
        return refusal(
            "input.field_unknown",
            format!("{key} is not an input of derive"),
        );
    }
    let Some(command) = v.get("command") else {
        return refusal("input.missing_field", "command is required");
    };
    let Some(command) = command.as_str().filter(|c| DERIVED.contains(c)) else {
        return refusal(
            "derivation.unsupported",
            format!("derive explains {}, not {command}", DERIVED.join(", ")),
        );
    };
    let Some(input) = v.get("input") else {
        return refusal("input.missing_field", "input is required");
    };
    let (result, steps) = engine::with_steps(|| evaluate_coded(command, input));
    let result = result?;
    Ok(serde_json::json!({
        "format": "australian-tax-calculators/derivation/1",
        "engine": result["engine"],
        "engine_version": result["engine_version"],
        "calculation": result["calculation"],
        "input": input,
        "steps": steps,
        "result": result,
    }))
}

/// A JSON value as the Python object the original received.
pub fn arg(value: &Value) -> Arg {
    match value {
        Value::Bool(b) => Arg::Bool(*b),
        Value::Number(n) => match n.as_i64() {
            Some(i) => Arg::Int(i),
            None if is_integer(&n.to_string()) => Arg::BigInt(n.to_string()),
            None => Arg::Other(py_repr(value)),
        },
        Value::String(s) => Arg::Text(s.clone()),
        other => Arg::Other(py_repr(other)),
    }
}

/// An integer token of any size, which `json.loads` makes an `int`.
fn is_integer(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// `repr()` of the Python object `json.loads` makes from `value`.
fn py_repr(value: &Value) -> String {
    match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(n) if is_integer(&n.to_string()) => {
            let text = n.to_string();
            // `int("-0")` is 0.
            if text.trim_start_matches('-').bytes().all(|b| b == b'0') {
                "0".into()
            } else {
                text
            }
        }
        // float(text): an overflowing literal such as 1e400 is infinity.
        Value::Number(n) => float_repr(n.to_string().parse().unwrap_or(f64::NAN)),
        Value::String(s) => crate::pytext::repr(s),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(py_repr).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(items) => format!(
            "{{{}}}",
            items
                .iter()
                .map(|(k, v)| format!("{}: {}", crate::pytext::repr(k), py_repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// `format!("{value:e}")`, but where two spellings are equally short, the one
/// CPython's `repr` prints: the nearer to `value`, an exact tie going to the even
/// digit. Rust's shortest formatting can break that tie upwards: `1 + 2**-17` is
/// `1.0000076293945312` in Python, not `…313`. `{:.Ne}` rounds exactly with ties
/// to even, so its spelling is used whenever it still reads back as `value`.
fn shortest_exp(value: f64) -> String {
    let shortest = format!("{value:e}");
    let (mantissa, _) = shortest.split_once('e').expect("exponent form");
    let precision = mantissa.bytes().filter(u8::is_ascii_digit).count() - 1;
    let nearest = format!("{value:.precision$e}");
    if nearest.parse::<f64>() == Ok(value) {
        nearest
    } else {
        shortest
    }
}

/// `repr(float)`: the shortest text that reads back as the same double.
fn float_repr(value: f64) -> String {
    if value.is_nan() {
        return "nan".into();
    }
    if value.is_infinite() {
        return if value > 0.0 { "inf" } else { "-inf" }.into();
    }
    let sci = shortest_exp(value);
    let (mantissa, exponent) = sci.split_once('e').expect("exponent form");
    let exponent: i32 = exponent.parse().expect("exponent digits");
    let (sign, mantissa) = match mantissa.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", mantissa),
    };
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let decpt = exponent + 1;
    if !(-4 < decpt && decpt <= 16) {
        let rest = &digits[1..];
        let dot = if rest.is_empty() {
            String::new()
        } else {
            format!(".{rest}")
        };
        let esign = if exponent < 0 { '-' } else { '+' };
        return format!(
            "{sign}{}{dot}e{esign}{:02}",
            &digits[..1],
            exponent.unsigned_abs()
        );
    }
    let body = if decpt <= 0 {
        format!("0.{}{digits}", "0".repeat((-decpt) as usize))
    } else if decpt as usize >= digits.len() {
        format!("{digits}{}.0", "0".repeat(decpt as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..decpt as usize],
            &digits[decpt as usize..]
        )
    };
    format!("{sign}{body}")
}

/// `payg-withholding`'s optional inputs, which `upward-variation-withholding`
/// takes too: each is absent unless the payee's declarations call for it.
fn payg_option_args(v: &Value) -> Result<[Option<Arg>; 4]> {
    let offsets = match v.get("tax_offsets") {
        None => None,
        Some(_) => Some(argument(v, "tax_offsets", Dec)?),
    };
    let [foreign, children, extra] = [
        "foreign_resident",
        "medicare_adjustment_children",
        "extra_pay_withholding",
    ]
    .map(|key| v.get(key).map(arg));
    Ok([foreign, offsets, children, extra])
}

fn payg_options(o: &[Option<Arg>; 4]) -> engine::PaygOptions<'_> {
    engine::PaygOptions {
        foreign_resident: o[0].as_ref(),
        tax_offsets: o[1].as_ref(),
        medicare_adjustment_children: o[2].as_ref(),
        extra_pay_withholding: o[3].as_ref(),
    }
}

fn argument(v: &Value, key: &str, kind: Kind) -> Result<Arg> {
    let Some(value) = v.get(key) else {
        return refusal("input.missing_field", format!("{key} is required"));
    };
    match (kind, value) {
        (Dec, Value::String(text)) => {
            if text.len() > 8192 {
                return refusal(
                    "input.decimal_too_long",
                    format!("{key}: decimal input exceeds 8192 bytes"),
                );
            }
            Decimal::parse_input(text)
                .map(Arg::Decimal)
                .or_else(|s| refusal("input.decimal_unreadable", format!("decimal.{}", s.name())))
        }
        (Json, _) => Ok(Arg::Json(value.clone())),
        _ => Ok(arg(value)),
    }
}

/// The inputs `v` gives besides `params`, by name: a string as a decimal,
/// except `fbt-housing`'s `state`, and lists and a parking contribution
/// object as JSON for the worksheet to read.
fn benefit_inputs(v: &Value, params: &[(&str, Kind)]) -> Result<Vec<(String, Arg)>> {
    let named = v.as_object().into_iter().flatten();
    named
        .filter(|(key, _)| !params.iter().any(|(p, _)| p == key))
        .map(|(key, value)| {
            let given = match value {
                Value::String(_) if key != "state" => argument(v, key, Dec)?,
                Value::Object(_) if key == "employee_contribution" => Arg::Json(value.clone()),
                Value::Array(_) => Arg::Json(value.clone()),
                other => arg(other),
            };
            Ok((key.clone(), given))
        })
        .collect()
}

/// An optional argument: `None` when `key` is absent.
fn optional(v: &Value, key: &str, kind: Kind) -> Result<Option<Arg>> {
    v.get(key).map(|_| argument(v, key, kind)).transpose()
}

/// An optional decimal input, read as every command reads one.
pub(crate) fn optional_decimal(v: &Value, key: &str) -> Result<Option<Arg>> {
    optional(v, key, Dec)
}

/// `json.loads` refuses an integer token of more than 4,300 digits, as
/// `int(text)` does under Python's default limit.
fn int_limit(value: &Value) -> Result<()> {
    match value {
        Value::Number(n) => {
            let text = n.to_string();
            let digits = text.strip_prefix('-').unwrap_or(&text);
            if digits.len() > 4300 && digits.bytes().all(|b| b.is_ascii_digit()) {
                return refusal(
                    "input.integer_too_long",
                    format!(
                        "ValueError: Exceeds the limit (4300 digits) for integer string conversion: \
                         value has {} digits; use sys.set_int_max_str_digits() to increase the limit",
                        digits.len()
                    ),
                );
            }
            Ok(())
        }
        Value::Array(items) => items.iter().try_for_each(int_limit),
        Value::Object(items) => items.values().try_for_each(int_limit),
        _ => Ok(()),
    }
}

/// Whether `command` takes the input `key`, required or optional.
pub(crate) fn takes(command: &str, key: &str) -> bool {
    let kind = command.replace('-', "_");
    // The catalogue's required and optional inputs, which for the FBT benefit
    // worksheets are more than their positional parameters.
    let named = [
        crate::metadata::input_units(&kind),
        crate::metadata::optional_inputs(&kind),
    ];
    COMMAND_ARGS
        .iter()
        .any(|(c, params)| *c == command && params.iter().any(|(p, _)| *p == key))
        || named.iter().flatten().any(|(n, _)| *n == key)
}

/// The `contribution-caps` inputs that work out the unused concessional cap
/// from each earlier year's contributions instead of taking it as a total.
const CARRY_FORWARD_INPUTS: [&str; 3] = [
    "earlier_concessional_contributions",
    "proposed_concessional_contribution",
    "defined_benefit_interest",
];

/// `contribution-caps` given a carry-forward input. It has no Python original
/// and a misspelt proposed contribution would be silently left out, so a
/// field it does not take is refused, as is the unused total beside the years.
fn contribution_caps_by_year(v: &Value) -> Result<Value> {
    let command = "contribution-caps";
    let mut keys = v.as_object().into_iter().flat_map(|o| o.keys());
    if let Some(key) = keys.find(|k| *k == "unused_concessional_cap" || !takes(command, k)) {
        return refusal(
            "input.field_unknown",
            format!("{key} is not an input of {command} when it works out the unused cap by year"),
        );
    }
    let (_, params) = COMMAND_ARGS
        .iter()
        .find(|(c, _)| *c == command)
        .expect("listed command");
    let a: Vec<Arg> = params
        .iter()
        .filter(|(key, _)| *key != "unused_concessional_cap")
        .map(|(key, kind)| argument(v, key, *kind))
        .collect::<Result<_>>()?;
    let earlier = argument(v, "earlier_concessional_contributions", Json)?;
    let flag = argument(v, "defined_benefit_interest", Raw)?;
    let proposed = optional(v, "proposed_concessional_contribution", Dec)?;
    let date = optional(v, "determination_date", Raw)?;
    let carry = engine::CarryForward {
        earlier_contributions: &earlier,
        proposed_contribution: proposed.as_ref(),
        defined_benefit_interest: &flag,
        determination_date: date.as_ref(),
    };
    engine::contribution_caps_by_year(&a[0], &a[1], &a[2], &a[3], carry, &a[4], &a[5])
        .or_else(|e| refusal(e.code(), e.to_string()))
}

/// Run one JSON stdin command: the result value, or the refusal message.
pub fn evaluate(command: &str, v: &Value) -> std::result::Result<Value, String> {
    evaluate_coded(command, v).map_err(|refusal| refusal.message)
}

/// [`evaluate`], keeping each refusal's stable code beside its message.
pub fn evaluate_coded(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return refusal("input.not_object", "input must be a JSON object");
    }
    int_limit(v)?;
    match command {
        "worksheet-catalogue" => return Ok(engine::worksheet_catalogue()),
        "derive" => return derive(v),
        "personal-tax-estimate" => return crate::estimate::compose(v),
        "contribution-caps" if CARRY_FORWARD_INPUTS.iter().any(|k| v.get(k).is_some()) => {
            return contribution_caps_by_year(v);
        }
        "dgr-check" => return crate::dgr::check(v),
        "fbt-return" => return crate::fbt_return::compose(v),
        "periods" => {
            let kind = match v.get("kind") {
                None => return refusal("input.missing_field", "kind is required"),
                Some(Value::String(kind)) => kind.as_str(),
                Some(_) => return refusal("input.kind_invalid", "kind must be a string"),
            };
            // Python raised KeyError for a worksheet it does not have.
            if crate::metadata::supported_periods(kind).is_empty() {
                return refusal(
                    "worksheet.unknown",
                    format!("unknown worksheet {}", crate::pytext::repr(kind)),
                );
            }
            return Ok(engine::periods(kind));
        }
        _ => {}
    }
    let Some((_, params)) = COMMAND_ARGS.iter().find(|(c, _)| *c == command) else {
        return refusal("command.unknown", format!("unknown command {command}"));
    };
    // Division 296, the termination payment, unused leave, Medicare levy surcharge,
    // Schedule 5, 9 and 15, the seniors and pensioners and lump sum in arrears
    // tax offset worksheets have no
    // Python original whose lenience to keep, and a misspelt optional
    // depreciation, Medicare levy or PAYG withholding input would silently give
    // the start year's, a single person's or the result without it, so a field
    // one does not take, such as an opening balance, another method's input or
    // a TFN exemption, is refused rather than ignored.
    // The FBT benefit worksheets have no Python original either.
    if matches!(
        command,
        "division-296"
            | "depreciation"
            | "fuel-tax-credits"
            | "alcohol-excise"
            | "medicare-levy"
            | "medicare-levy-surcharge"
            | "payg-withholding"
            | "payg-gross-up"
            | "termination-payment"
            | "etp-tax-free-component"
            | "unused-leave-payment"
            | "schedule-5-method-a"
            | "schedule-5-method-b-i"
            | "schedule-5-method-b-ii"
            | "working-holiday-maker-withholding"
            | "study-loan-withholding"
            | "foreign-resident-tax"
            | "seniors-pensioners-withholding"
            | "seniors-pensioners-tax-offset"
            | "lump-sum-arrears-tax-offset"
            | "beneficiary-tax-offset"
            | "invalid-carer-tax-offset"
            | "zone-tax-offset"
            | "net-medical-expenses-tax-offset"
            | "fbt-loan"
            | "fbt-meal-entertainment"
            | "fbt-car-parking"
            | "wine-equalisation-tax"
            | "fbt-debt-waiver"
            | "fbt-expense-payment"
            | "fbt-property"
            | "fbt-residual"
            | "fbt-housing"
            | "fbt-lafha"
            | "fbt-board"
            | "fbt-tax-exempt-body-entertainment"
            | "fbt-entertainment-classifier"
            | "fbt-meal-entertainment-register"
            | "fbt-minor-benefit-check"
            | "fhss"
            | "ppsr-timing"
            | "hire-purchase-schedule"
            | "chattel-mortgage-schedule"
            | "loan-transactions"
            | "borrowing-expenses-schedule"
            | "income-tests"
            | "luxury-car-tax"
            | "sb-cgt-concessions"
            | "work-expenses"
            | "ato-interest-penalties"
            | "transfer-balance"
            | "excess-contributions"
            | "private-health-insurance-rebate"
            | "depreciation-low-value-pool"
            | "depreciation-small-business-pool"
            | "capital-works"
            | "small-business-income-offset"
            | "trading-stock"
            | "prepayments"
            | "non-commercial-losses"
            | "first-home-owner-grant"
            | "psi-test"
            | "smsf-tax"
            | "government-super-amounts"
            | "rental-property"
            | "horticulture-shearing-withholding"
            | "upward-variation-withholding"
            | "daily-casual-withholding"
            | "no-abn-withholding"
            | "entertainer-withholding"
            | "return-to-work-withholding"
            | "annuity-withholding"
            | "voluntary-agreement-withholding"
            | "residency-tests"
            | "lodgment-required"
            | "rd-tax-offset"
            | "gst-creditable-purpose-change"
            | "gst-annual-apportionment"
            | "gst-financial-acquisitions-threshold"
            | "gst-going-concern"
            | "gst-second-hand-goods"
            | "gst-bad-debt"
            | "deferred-tax"
            | "foreign-income-conversion"
            | "foreign-income-tax-offset"
            | "minors-income"
            | "business-capital-expenditure"
            | "balancing-adjustment"
            | "trade-in"
            | "fbt-ev-exemption-check"
            | "dasp-tax"
            | "super-benefit-tax"
            | "super-withholding"
            | "payroll-tax"
            | "gst-margin-scheme"
            | "franking-account"
            | "cgt-parcels"
            | "cgt-cost-base-adjustments"
    ) && let Some(key) = v
        .as_object()
        .into_iter()
        .flat_map(|o| o.keys())
        .find(|k| !takes(command, k))
    {
        return refusal(
            "input.field_unknown",
            format!("{key} is not an input of {command}"),
        );
    }
    let a: Vec<Arg> = params
        .iter()
        .map(|(key, kind)| argument(v, key, *kind))
        .collect::<Result<_>>()?;
    // Optional: the study loan component's pay date, which selects the Schedule 8 coefficients.
    let pay_date = v.get("pay_date").map(arg);
    let result = match command {
        "gst" => engine::gst(&a[0], &a[1], &a[2], &a[3]),
        "fuel-tax-credits" => {
            // Optional: the simplified methods' small claimant statement and
            // tax period, and the return's day for the s 47-5(2) check.
            let small = optional(v, "small_claimant", Raw)?;
            let period = optional(v, "rate_at_period_end", Json)?;
            let returned = optional(v, "return_date", Raw)?;
            engine::fuel_tax_credits(&engine::FuelArgs {
                lines: &a[0],
                small_claimant: small.as_ref(),
                rate_at_period_end: period.as_ref(),
                return_date: returned.as_ref(),
                year: &a[1],
                scope_confirmed: &a[2],
            })
        }
        "alcohol-excise" => {
            // Optional: the remission facts, required or taken only with a
            // line marked remission.
            let used = optional(v, "remission_used_this_year", Dec)?;
            let from = optional(v, "manufacturer_from", Raw)?;
            let paid = optional(v, "duty_paid", Raw)?;
            engine::alcohol_excise(&engine::AlcoholExciseArgs {
                lines: &a[0],
                remission_used_this_year: used.as_ref(),
                manufacturer_from: from.as_ref(),
                duty_paid: paid.as_ref(),
                year: &a[1],
                scope_confirmed: &a[2],
            })
        }
        "resident-tax" => engine::resident_tax(&a[0], &a[1], &a[2]),
        "foreign-resident-tax" => engine::foreign_resident_tax(&a[0], &a[1], &a[2]),
        "capital-gains" => engine::capital_gains(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5]),
        "main-residence" => engine::main_residence(
            &a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7], &a[8], &a[9], &a[10],
        ),
        "borrowing-expenses" => {
            let loan = engine::LoanDates {
                start: &a[3],
                term_end: &a[4],
                repaid_on: &a[5],
            };
            engine::borrowing_expenses(&a[0], &a[1], &a[2], loan, &a[6], &a[7], &a[8])
        }
        "borrowing-expenses-schedule" => {
            // Optional: the income-producing share of borrowing expenses on
            // loans not listed, by income year.
            let other = optional(v, "other_borrowing_expenses", Json)?;
            engine::borrowing_expenses_schedule(&a[0], other.as_ref(), &a[1], &a[2])
        }
        "hire-purchase-schedule" => {
            // Optional: the level regime's instalment, their number and
            // frequency, or segments in their place; a deposit, trade-in,
            // payout and GST facts.
            let [instalment, deposit, trade_in] =
                ["instalment", "deposit", "trade_in"].map(|key| optional(v, key, Dec));
            let [instalments, frequency] =
                ["instalments", "frequency"].map(|key| optional(v, key, Raw));
            let [segments, payout, gst] =
                ["segments", "payout", "gst"].map(|key| optional(v, key, Json));
            let (instalment, deposit, trade_in) = (instalment?, deposit?, trade_in?);
            let (instalments, frequency) = (instalments?, frequency?);
            let (segments, payout, gst) = (segments?, payout?, gst?);
            engine::hire_purchase_schedule(&engine::HirePurchaseArgs {
                amount_financed: &a[0],
                instalment: instalment.as_ref(),
                instalments: instalments.as_ref(),
                frequency: frequency.as_ref(),
                segments: segments.as_ref(),
                balloon: &a[1],
                timing: &a[2],
                start_date: &a[3],
                deposit: deposit.as_ref(),
                trade_in: trade_in.as_ref(),
                payout: payout.as_ref(),
                gst: gst.as_ref(),
                year: &a[4],
                scope_confirmed: &a[5],
            })
        }
        "chattel-mortgage-schedule" => {
            // Optional: the instalment or the annual rate (one of them), the
            // frequency, a deposit, trade-in and GST facts.
            let [instalment, annual_rate, deposit, trade_in] =
                ["instalment", "annual_rate", "deposit", "trade_in"]
                    .map(|key| optional(v, key, Dec));
            let frequency = optional(v, "frequency", Raw)?;
            let gst = optional(v, "gst", Json)?;
            let (instalment, annual_rate) = (instalment?, annual_rate?);
            let (deposit, trade_in) = (deposit?, trade_in?);
            engine::chattel_mortgage_schedule(&engine::ChattelMortgageArgs {
                amount_financed: &a[0],
                instalment: instalment.as_ref(),
                annual_rate: annual_rate.as_ref(),
                instalments: &a[1],
                frequency: frequency.as_ref(),
                balloon: &a[2],
                timing: &a[3],
                start_date: &a[4],
                deposit: deposit.as_ref(),
                trade_in: trade_in.as_ref(),
                gst: gst.as_ref(),
                year: &a[5],
                scope_confirmed: &a[6],
            })
        }
        "loan-transactions" => {
            // Optional: the dated rate changes.
            let rate_changes = optional(v, "rate_changes", Json)?;
            engine::loan_transactions(&engine::LoanTransactionsArgs {
                opening_balance: &a[0],
                annual_rate: &a[1],
                start_date: &a[2],
                end_date: &a[3],
                transactions: &a[4],
                rate_changes: rate_changes.as_ref(),
                year: &a[5],
                scope_confirmed: &a[6],
            })
        }
        "business-capital-expenditure" => engine::business_capital_expenditure(&a[0], &a[1], &a[2]),
        "cgt-parcels" => engine::cgt_parcels(&engine::CgtParcelsArgs {
            entity: &a[0],
            identification: &a[1],
            parcels: &a[2],
            disposals: &a[3],
            other_discount_capital_gains: &a[4],
            other_capital_gains: &a[5],
            other_capital_losses: &a[6],
            prior_net_capital_losses: &a[7],
            year: &a[8],
            scope_confirmed: &a[9],
        }),
        "cgt-cost-base-adjustments" => {
            engine::cgt_cost_base_adjustments(&engine::CgtCostBaseArgs {
                entity: &a[0],
                parcels: &a[1],
                events: &a[2],
                year: &a[3],
                scope_confirmed: &a[4],
            })
        }
        "balancing-adjustment" => {
            // Optional: a car's s 40-325 facts, or null; refused for any other asset.
            let car_limit = optional(v, "car_limit", Json)?;
            engine::balancing_adjustment(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                car_limit.as_ref(),
                &a[4],
                &a[5],
            )
        }
        "franking-account" => {
            // Optional: the income tax facts s 205-70 applies the offset to,
            // and a Commissioner's determination under s 205-70(6).
            let income_tax = optional(v, "income_tax", Json)?;
            let determination = optional(v, "commissioner_determination", Raw)?;
            engine::franking_year(&engine::FrankingArgs {
                private_company: &a[0],
                benchmark_rule_exempt: &a[1],
                corporate_tax_rate: &a[2],
                opening_balance: &a[3],
                rd_deferred_debit_brought_forward: &a[4],
                last_benchmark: &a[5],
                events: &a[6],
                distributions: &a[7],
                year: &a[8],
                scope_confirmed: &a[9],
                income_tax: income_tax.as_ref(),
                commissioner_determination: determination.as_ref(),
            })
        }
        "super-benefit-tax" => {
            // Optional: the facts only a member benefit, or only some, call for.
            let given = engine::SUPER_BENEFIT_OPTIONAL.map(|key| {
                let kind = if key.ends_with("_used") { Dec } else { Raw };
                optional(v, key, kind)
            });
            let [a0, a1, a2, a3, a4, a5, a6] = given;
            let given = [a0?, a1?, a2?, a3?, a4?, a5?, a6?];
            engine::super_benefit_tax(&engine::SuperBenefitTaxArgs {
                benefit: &a[0],
                interest_value: &a[1],
                interest_tax_free_component: &a[2],
                untaxed_element: &a[3],
                recipient: &a[4],
                payment_date: &a[5],
                year: &a[6],
                scope_confirmed: &a[7],
                optional: given.each_ref().map(Option::as_ref),
            })
        }
        "super-withholding" => {
            // Optional: the same facts as super-benefit-tax.
            let given = engine::SUPER_BENEFIT_OPTIONAL.map(|key| {
                let kind = if key.ends_with("_used") { Dec } else { Raw };
                optional(v, key, kind)
            });
            let [a0, a1, a2, a3, a4, a5, a6] = given;
            let given = [a0?, a1?, a2?, a3?, a4?, a5?, a6?];
            engine::super_withholding(&engine::SuperWithholdingArgs {
                base: engine::SuperBenefitTaxArgs {
                    benefit: &a[0],
                    interest_value: &a[1],
                    interest_tax_free_component: &a[2],
                    untaxed_element: &a[3],
                    recipient: &a[4],
                    payment_date: &a[5],
                    year: &a[8],
                    scope_confirmed: &a[9],
                    optional: given.each_ref().map(Option::as_ref),
                },
                tfn_provided: &a[6],
                foreign_resident: &a[7],
            })
        }
        "dasp-tax" => engine::dasp_tax(&engine::DaspTaxArgs {
            tax_free_component: &a[0],
            taxed_element: &a[1],
            untaxed_element: &a[2],
            working_holiday_maker_contributions: &a[3],
            unclaimed_money_rollover: &a[4],
            year: &a[5],
            scope_confirmed: &a[6],
        }),
        "wine-equalisation-tax" => {
            // Optional: the rebate facts, required with a dealing's
            // rebate_conditions.
            let earlier = optional(v, "rebate_claimed_earlier_in_year", Dec)?;
            let group = optional(v, "associated_producer", Raw)?;
            engine::wine_equalisation_tax(&engine::WetArgs {
                dealings: &a[0],
                rebate_claimed_earlier_in_year: earlier.as_ref(),
                associated_producer: group.as_ref(),
                year: &a[1],
                scope_confirmed: &a[2],
            })
        }
        "trade-in" => {
            // Optional: a car's s 40-325 facts, the traded-in asset's input
            // tax credits for a taxable trade-in, a passenger car's luxury car
            // tax and a creditable acquisition's creditable purpose.
            let given = |key| optional(v, key, if key == "car_limit" { Json } else { Dec });
            let [car_limit, adjusted, full, luxury, purpose] = [
                "car_limit",
                "adjusted_input_tax_credit",
                "full_input_tax_credit",
                "replacement_luxury_car_tax",
                "creditable_purpose",
            ]
            .map(given);
            let (car_limit, adjusted, full) = (car_limit?, adjusted?, full?);
            let (luxury, purpose) = (luxury?, purpose?);
            engine::trade_in(&engine::TradeInArgs {
                cost: &a[0],
                asset: &a[1],
                years: &a[2],
                car_limit: car_limit.as_ref(),
                trade_in_allowance: &a[3],
                trade_in_taxable: &a[4],
                adjusted_input_tax_credit: adjusted.as_ref(),
                full_input_tax_credit: full.as_ref(),
                replacement_price: &a[5],
                replacement_asset: &a[6],
                replacement_luxury_car_tax: luxury.as_ref(),
                replacement_creditable_acquisition: &a[7],
                creditable_purpose: purpose.as_ref(),
                arms_length: &a[8],
                involuntary_disposal: &a[9],
                year: &a[10],
                scope_confirmed: &a[11],
            })
        }
        "payroll-tax" => {
            // Optional: whether a Victorian employer is a regional employer.
            let regional = optional(v, "regional_employer", Raw)?;
            engine::payroll_tax(&engine::PayrollTaxArgs {
                jurisdiction: &a[0],
                taxable_wages: &a[1],
                australian_wages: &a[2],
                regional_employer: regional.as_ref(),
                group_member: &a[3],
                employed_all_year: &a[4],
                year: &a[5],
                scope_confirmed: &a[6],
            })
        }
        "gst-margin-scheme" => {
            // Optional: each method's base, its valuation facts and a
            // subdivided lot's share.
            let [paid, unpaid, valuation, share, whole] = [
                "acquisition_consideration",
                "unpaid_acquisition_consideration",
                "approved_valuation",
                "subdivision_share",
                "subdivision_whole",
            ]
            .map(|key| optional(v, key, Dec));
            let [day, registered, government] = [
                "valuation_day",
                "registered_on_1_july_2000",
                "government_unimproved_on_1_july_2000",
            ]
            .map(|key| optional(v, key, Raw));
            let [improved, effective, applied, application, supply] = [
                "improved_on_1_july_2000",
                "registration_effective_day",
                "applied_for_registration",
                "registration_application_day",
                "supply_day",
            ]
            .map(|key| optional(v, key, Raw));
            let [paid, unpaid, valuation, share, whole] =
                [paid?, unpaid?, valuation?, share?, whole?];
            let [day, registered, government] = [day?, registered?, government?];
            engine::gst_margin_scheme(&engine::MarginSchemeArgs {
                interest: &a[0],
                agreement: &a[1],
                acquisition: &a[2],
                consideration: &a[3],
                method: &a[4],
                recipient_associate: &a[5],
                acquisition_consideration: paid.as_ref(),
                unpaid_acquisition_consideration: unpaid.as_ref(),
                approved_valuation: valuation.as_ref(),
                valuation_day: day.as_ref(),
                registered_on_1_july_2000: registered.as_ref(),
                government_unimproved_on_1_july_2000: government.as_ref(),
                improved_on_1_july_2000: improved?.as_ref(),
                registration_effective_day: effective?.as_ref(),
                applied_for_registration: applied?.as_ref(),
                registration_application_day: application?.as_ref(),
                supply_day: supply?.as_ref(),
                subdivision_share: share.as_ref(),
                subdivision_whole: whole.as_ref(),
                year: &a[6],
                scope_confirmed: &a[7],
            })
        }
        "fbt" => engine::fbt(&a[0], &a[1], &a[2], &a[3]),
        "fbt-car-statutory" => {
            engine::fbt_car_statutory(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7])
        }
        "fbt-car-operating-cost" => engine::fbt_car_operating_cost(
            &a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7], &a[8], &a[9], &a[10],
        ),
        "fbt-ev-exemption-check" => {
            // Optional: a plug-in hybrid's transitional facts.
            let [before, new, exempt] = [
                "commitment_before_1_april_2025",
                "new_commitment_from_1_april_2025",
                "exempt_benefit_before_1_april_2025",
            ]
            .map(|key| optional(v, key, Raw));
            let hybrid = [before?, new?, exempt?];
            engine::fbt_ev_exemption_check(&engine::EvExemptionArgs {
                vehicle: &a[0],
                car: &a[1],
                current_employee: &a[2],
                first_held_and_used: &a[3],
                first_retail_sale: &a[4],
                luxury_car_tax_value: &a[5],
                luxury_car_tax_on_other_supply: &a[6],
                hybrid: hybrid.each_ref().map(Option::as_ref),
                year_ended: &a[7],
                scope_confirmed: &a[8],
            })
        }
        "fbt-loan" => engine::fbt_loan(&benefit_inputs(v, params)?, &a[0], &a[1]),
        "fbt-meal-entertainment" => {
            engine::fbt_meal_entertainment(&a[0], &benefit_inputs(v, params)?, &a[1], &a[2])
        }
        "fbt-meal-entertainment-register" => {
            // Optional: each earlier year's expenses, for the 20% rule in a
            // later year of the register's validity.
            let earlier = optional(v, "register_years_expenditure", Json)?;
            engine::fbt_meal_entertainment_register(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                earlier.as_ref(),
            )
        }
        "fbt-car-parking" => {
            engine::fbt_car_parking(&a[0], &benefit_inputs(v, params)?, &a[1], &a[2])
        }
        "fbt-debt-waiver" => engine::fbt_debt_waiver(&benefit_inputs(v, params)?, &a[0], &a[1]),
        "fbt-expense-payment" => {
            engine::fbt_expense_payment(&a[0], &benefit_inputs(v, params)?, &a[1], &a[2])
        }
        "fbt-property" => engine::fbt_property(&a[0], &benefit_inputs(v, params)?, &a[1], &a[2]),
        "fbt-residual" => engine::fbt_residual(&a[0], &benefit_inputs(v, params)?, &a[1], &a[2]),
        "fbt-housing" => engine::fbt_housing(&a[0], &benefit_inputs(v, params)?, &a[1], &a[2]),
        "fbt-lafha" => engine::fbt_lafha(&a[0], &benefit_inputs(v, params)?, &a[1], &a[2]),
        "fbt-board" => engine::fbt_board(&benefit_inputs(v, params)?, &a[0], &a[1]),
        "fbt-tax-exempt-body-entertainment" => {
            let given = benefit_inputs(v, params)?;
            engine::fbt_tax_exempt_body_entertainment(&a[0], &given, &a[1], &a[2])
        }
        "ppsr-timing" => {
            // The question's own inputs, by name: times stay text, day lists JSON.
            let named: Vec<(String, Arg)> = v
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(key, _)| !params.iter().any(|(p, _)| p == key))
                .map(|(key, value)| {
                    let given = match value {
                        Value::Array(_) => Arg::Json(value.clone()),
                        other => arg(other),
                    };
                    (key.clone(), given)
                })
                .collect();
            engine::ppsr_timing(&a[0], &a[1], &a[2], &named)
        }
        "fbt-minor-benefit-check" => {
            // Optional: the facts each category's exclusion needs.
            let [in_house, exception, fifty] =
                ["in_house", "tax_exempt_body_exception", "fifty_fifty_split"]
                    .map(|key| optional(v, key, Raw));
            let [in_house, exception, fifty] = [in_house?, exception?, fifty?];
            engine::fbt_minor_benefit_check(&engine::MinorBenefitArgs {
                category: &a[0],
                notional_taxable_value: &a[1],
                salary_sacrifice: &a[2],
                frequency: &a[3],
                similar_benefits_substantial: &a[4],
                other_associated_benefits_substantial: &a[5],
                valuation_difficult: &a[6],
                unexpected_event: &a[7],
                reward_for_services: &a[8],
                in_house: in_house.as_ref(),
                tax_exempt_body_exception: exception.as_ref(),
                fifty_fifty_split: fifty.as_ref(),
                year_ended: &a[9],
                scope_confirmed: &a[10],
            })
        }
        "fhss" => {
            let [contributions, requested, deductions, determination] = [
                "contributions",
                "determination_requested",
                "personal_deductions",
                "determination",
            ]
            .map(|key| {
                let kind = if key == "determination_requested" {
                    Raw
                } else {
                    Json
                };
                optional(v, key, kind)
            });
            let [contributions, requested, deductions, determination] =
                [contributions?, requested?, deductions?, determination?];
            let amount = optional(v, "amount_requested", Dec)?;
            let marginal = optional(v, "estimated_marginal_rate", Dec)?;
            let cannot = optional(v, "commissioner_cannot_estimate", Raw)?;
            engine::fhss(&engine::FhssArgs {
                determination_date: &a[0],
                year: &a[1],
                scope_confirmed: &a[2],
                contributions: contributions.as_ref(),
                determination_requested: requested.as_ref(),
                personal_deductions: deductions.as_ref(),
                determination: determination.as_ref(),
                amount_requested: amount.as_ref(),
                estimated_marginal_rate: marginal.as_ref(),
                commissioner_cannot_estimate: cannot.as_ref(),
            })
        }
        "fbt-entertainment-classifier" => {
            // Optional: the meal entertainment method, required only when a
            // line is meal entertainment, and the 12 week register's income
            // tax register percentage.
            let method = optional(v, "method", Raw)?;
            let register = optional(v, "income_tax_register_percentage", Dec)?;
            engine::fbt_entertainment_classifier(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                method.as_ref(),
                register.as_ref(),
            )
        }
        "depreciation" => {
            // Optional: present only for an income year after the asset's start time.
            let key = "opening_adjustable_value";
            let opening = match v.get(key) {
                None => None,
                Some(_) => Some(argument(v, key, Dec)?),
            };
            engine::depreciation(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                &a[5],
                &a[6],
                opening.as_ref(),
            )
        }
        "depreciation-low-value-pool" => {
            engine::depreciation_low_value_pool(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6])
        }
        "depreciation-small-business-pool" => {
            engine::depreciation_small_business_pool(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5])
        }
        "capital-works" => {
            engine::capital_works(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7])
        }
        "horticulture-shearing-withholding" => engine::horticulture_shearing_withholding(
            &a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7],
        ),
        "upward-variation-withholding" => {
            let o = payg_option_args(v)?;
            engine::upward_variation_withholding(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                &a[5],
                &payg_options(&o),
            )
        }
        "daily-casual-withholding" => {
            // Optional: absent unless the payee claimed tax offsets.
            let offsets = optional(v, "tax_offsets", Dec)?;
            engine::daily_casual_withholding(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                &a[5],
                &a[6],
                offsets.as_ref(),
            )
        }
        "no-abn-withholding" => engine::no_abn_withholding(&a[0], &a[1], &a[2], &a[3], &a[4]),
        "entertainer-withholding" => {
            // Optional: absent unless the performer claimed tax offsets.
            let offsets = optional(v, "tax_offsets", Dec)?;
            engine::entertainer_withholding(
                [&a[0], &a[1], &a[2]],
                &a[3],
                &a[4],
                &a[5],
                &a[6],
                &a[7],
                &a[8],
                offsets.as_ref(),
            )
        }
        "return-to-work-withholding" => {
            engine::return_to_work_withholding(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5])
        }
        "annuity-withholding" => {
            let o = payg_option_args(v)?;
            engine::annuity_withholding(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                &a[5],
                &a[6],
                &payg_options(&o),
            )
        }
        "voluntary-agreement-withholding" => {
            // Optional: absent when the payee has no Commissioner's instalment rate.
            let cir = optional(v, "instalment_rate", Dec)?;
            engine::voluntary_agreement_withholding(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                &a[5],
                &a[6],
                cir.as_ref(),
            )
        }
        "rd-tax-offset" => engine::rd_tax_offset(&engine::RdTaxOffsetArgs {
            notional_deductions: &a[0],
            rsp_crc_expenditure: &a[1],
            aggregated_turnover: &a[2],
            exempt_entity_controlled: &a[3],
            base_rate_entity: &a[4],
            total_expenses: &a[5],
            recoupment_clawback: &a[6],
            feedstock_revenue: &a[7],
            feedstock_expenditure: &a[8],
            present_year_base_rate_entity: &a[9],
            year: &a[10],
            scope_confirmed: &a[11],
        }),
        "quarterly-sg" => engine::quarterly_sg(&a[0], &a[1], &a[2], &a[3], &a[4]),
        "payg-withholding" => {
            let o = payg_option_args(v)?;
            engine::payg_withholding(&a[0], &a[1], &a[2], &a[3], &a[4], &payg_options(&o))
        }
        "payg-gross-up" => {
            // Optional: scale 4's residency, as payg-withholding takes it, and
            // a study loan with its pay date, as study-loan-withholding takes it.
            let foreign = v.get("foreign_resident").map(arg);
            let study_loan = v.get("study_loan").map_or(Arg::Bool(false), arg);
            engine::payg_gross_up(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                foreign.as_ref(),
                (&study_loan, pay_date.as_ref()),
            )
        }
        "study-loan-withholding" => {
            engine::study_loan_withholding(&a[0], &a[1], &a[2], &a[3], &a[4], pay_date.as_ref())
        }
        "medicare-levy" => {
            // Optional: each absent for a person without SAPTO, spouse,
            // dependants or exemption.
            let o = [
                optional(v, "sapto_entitled", Raw)?,
                optional(v, "spouse_taxable_income", Dec)?,
                optional(v, "spouse_sapto_entitled", Raw)?,
                optional(v, "dependant_offset_entitled", Raw)?,
                optional(v, "dependent_children", Raw)?,
                optional(v, "medicare_levy_exemption_full_days", Raw)?,
                optional(v, "medicare_levy_exemption_half_days", Raw)?,
            ];
            let family = engine::MedicareFamily {
                sapto_entitled: o[0].as_ref(),
                spouse_taxable_income: o[1].as_ref(),
                spouse_sapto_entitled: o[2].as_ref(),
                dependant_offset_entitled: o[3].as_ref(),
                dependent_children: o[4].as_ref(),
            };
            let exemption = engine::MedicareExemption {
                full_days: o[5].as_ref(),
                half_days: o[6].as_ref(),
            };
            engine::medicare_levy(&a[0], &a[1], &a[2], &a[3], family, exemption)
        }
        "medicare-levy-surcharge" => {
            // Optional: absent for a person without a spouse or dependants all year.
            let o = [
                optional(v, "spouse_income_for_surcharge_purposes", Dec)?,
                optional(v, "dependent_children", Raw)?,
            ];
            let family = engine::SurchargeFamily {
                spouse_income: o[0].as_ref(),
                dependent_children: o[1].as_ref(),
            };
            engine::medicare_levy_surcharge(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], family)
        }
        "low-income-tax-offset" => engine::low_income_tax_offset(&a[0], &a[1], &a[2]),
        "seniors-pensioners-tax-offset" => {
            // Optional: absent for a single person, and the transfer without a spouse's.
            let o = [
                optional(v, "spouse_rebate_income", Dec)?,
                optional(v, "spouse_taxable_income", Dec)?,
            ];
            let spouse = engine::SaptoSpouse {
                rebate_income: o[0].as_ref(),
                taxable_income: o[1].as_ref(),
            };
            engine::seniors_pensioners_tax_offset(&a[0], &a[1], &a[2], &a[3], spouse)
        }
        "beneficiary-tax-offset" => engine::rebatable_benefit_offset(&a[0], &a[1], &a[2]),
        "invalid-carer-tax-offset" => {
            // Optional: the part-year days, a spouse who is not the dependant
            // and the refused shared cases.
            let o = [
                optional(v, "days", Raw)?,
                optional(v, "spouse_adjusted_taxable_income", Dec)?,
                optional(v, "spouse_days", Raw)?,
                optional(v, "maintained_by_others", Raw)?,
                optional(v, "ftb_b_shared_care", Raw)?,
                optional(v, "different_spouses", Raw)?,
            ];
            engine::invalid_carer_tax_offset(&engine::InvalidCarerArgs {
                dependant: &a[0],
                adjusted_taxable_income: &a[1],
                dependant_adjusted_taxable_income: &a[2],
                year: &a[3],
                scope_confirmed: &a[4],
                days: o[0].as_ref(),
                spouse_adjusted_taxable_income: o[1].as_ref(),
                spouse_days: o[2].as_ref(),
                maintained_by_others: o[3].as_ref(),
                ftb_b_shared_care: o[4].as_ref(),
                different_spouses: o[5].as_ref(),
            })
        }
        "zone-tax-offset" => {
            // Optional: the days carried from an earlier year, the base
            // amount's parts and the remote area allowance.
            let o = [
                optional(v, "prior_year", Raw)?,
                optional(v, "prior_days", Raw)?,
                optional(v, "continuous_residence", Raw)?,
                optional(v, "includes_first_day", Raw)?,
                optional(v, "dependants", Json)?,
                optional(v, "sole_parent_days", Raw)?,
                optional(v, "invalid_carer_tax_offset", Dec)?,
                optional(v, "remote_area_allowance", Dec)?,
            ];
            engine::zone_tax_offset(&engine::ZoneArgs {
                area: &a[0],
                days: &a[1],
                year: &a[2],
                scope_confirmed: &a[3],
                prior_year: o[0].as_ref(),
                prior_days: o[1].as_ref(),
                continuous_residence: o[2].as_ref(),
                includes_first_day: o[3].as_ref(),
                dependants: o[4].as_ref(),
                sole_parent_days: o[5].as_ref(),
                invalid_carer_tax_offset: o[6].as_ref(),
                remote_area_allowance: o[7].as_ref(),
            })
        }
        "net-medical-expenses-tax-offset" => {
            // Optional: a spouse on 30 June and dependent children.
            let o = [
                optional(v, "spouse_adjusted_taxable_income", Dec)?,
                optional(v, "dependent_children", Raw)?,
            ];
            engine::disability_expenses_offset(&engine::MedicalArgs {
                net_medical_expenses: &a[0],
                adjusted_taxable_income: &a[1],
                year: &a[2],
                scope_confirmed: &a[3],
                spouse_adjusted_taxable_income: o[0].as_ref(),
                dependent_children: o[1].as_ref(),
            })
        }
        "lump-sum-arrears-tax-offset" => {
            engine::lump_sum_arrears_tax_offset(&a[0], &a[1], &a[2], &a[3], &a[4])
        }
        "division-293" => engine::division_293(&a[0], &a[1], &a[2], &a[3]),
        "division-296" => engine::division_296(&a[0], &a[1], [&a[2], &a[3], &a[4]], &a[5], &a[6]),
        "termination-payment" => {
            engine::termination_payment(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7])
        }
        "etp-tax-free-component" => {
            engine::etp_tax_free_component(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7])
        }
        "unused-leave-payment" => {
            // Optional: the parts accrued before the dates the Act splits leave
            // at, required where leave_accrual_start and the reason make them matter.
            let o = [
                optional(v, "annual_leave_before_18_august_1993", Dec)?,
                optional(v, "leave_loading_before_18_august_1993", Dec)?,
                optional(v, "long_service_leave_before_16_august_1978", Dec)?,
                optional(
                    v,
                    "long_service_leave_16_august_1978_to_17_august_1993",
                    Dec,
                )?,
            ];
            let splits = engine::LeaveSplits {
                annual_leave_before_18_august_1993: o[0].as_ref(),
                leave_loading_before_18_august_1993: o[1].as_ref(),
                long_service_leave_before_16_august_1978: o[2].as_ref(),
                long_service_leave_16_august_1978_to_17_august_1993: o[3].as_ref(),
            };
            // Optional: scale 4's residency, as payg-withholding takes it.
            let foreign = v.get("foreign_resident").map(arg);
            let employee = engine::LeavePayee {
                foreign_resident: foreign.as_ref(),
                working_holiday_maker: &a[5],
                earnings: &a[6],
                pay_period: &a[7],
                scale: &a[8],
            };
            engine::unused_leave_payment(
                [&a[0], &a[1], &a[2]],
                &splits,
                (&a[3], &a[4]),
                &employee,
                &a[9],
                &a[10],
            )
        }
        "schedule-5-method-a" => engine::schedule_5_method_a(
            &a[0],
            &a[1],
            &a[2],
            &a[3],
            &a[4],
            (&a[5], pay_date.as_ref()),
            &a[6],
            &a[7],
        ),
        "schedule-5-method-b-i" => engine::schedule_5_method_b_i(
            &a[0],
            &a[1],
            &a[2],
            &a[3],
            (&a[4], pay_date.as_ref()),
            &a[5],
            &a[6],
        ),
        "schedule-5-method-b-ii" => engine::schedule_5_method_b_ii(
            &a[0],
            &a[1],
            [&a[2], &a[3], &a[4], &a[5]],
            &a[6],
            &a[7],
            (&a[8], pay_date.as_ref()),
            &a[9],
            &a[10],
        ),
        "working-holiday-maker-withholding" => engine::working_holiday_maker_withholding(
            &a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6],
        ),
        "seniors-pensioners-withholding" => {
            // Optional: absent for a payee who declared no study loan.
            let study_loan = v.get("study_loan").map_or(Arg::Bool(false), arg);
            engine::seniors_pensioners_withholding(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                &a[5],
                (&study_loan, pay_date.as_ref()),
            )
        }
        "study-loan-repayment" => engine::study_loan_repayment(&a[0], &a[1], &a[2]),
        "contribution-caps" => {
            let date = optional(v, "determination_date", Raw)?;
            engine::contribution_caps(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                &a[5],
                &a[6],
                date.as_ref(),
            )
        }
        "pension-minimum" => engine::pension_minimum(&a[0], &a[1], &a[2], &a[3], &a[4]),
        "ato-interest-penalties" => {
            // The question's own facts, by name, as JSON.
            let facts: serde_json::Map<String, Value> = v
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(key, _)| !params.iter().any(|(p, _)| p == key))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            engine::ato_interest_penalties(&a[0], &a[1], &a[2], &facts)
        }
        "transfer-balance" => engine::transfer_balance(&a[0], &a[1], &a[2], &a[3], &a[4]),
        "excess-contributions" => {
            let facts: serde_json::Map<String, Value> = v
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(key, _)| !params.iter().any(|(p, _)| p == key))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            engine::excess_contributions(&a[0], &a[1], &a[2], &facts)
        }
        "rental-property" => {
            // Optional: other taxable income, for the effect on tax.
            let other = optional(v, "other_taxable_income", Dec)?;
            engine::rental_property(&engine::RentalPropertyArgs {
                ownership_share: &a[0],
                acquired: &a[1],
                disposed: &a[2],
                days_available: &a[3],
                section_26_50_applies: &a[4],
                income: &a[5],
                expenses: &a[6],
                rental_only_expenses: &a[7],
                loans: &a[8],
                borrowing_loans: &a[9],
                capital_works: &a[10],
                depreciating_assets: &a[11],
                other_taxable_income: other.as_ref(),
                year: &a[12],
                scope_confirmed: &a[13],
            })
        }
        "sb-cgt-concessions" => {
            // Optional: the facts only some answers call for.
            let given = engine::SB_CGT_OPTIONAL.map(|key| {
                let kind = if key == "retirement_exemption_used" {
                    Dec
                } else {
                    Raw
                };
                optional(v, key, kind)
            });
            let [a0, a1, a2, a3, a4, a5, a6] = given;
            let given = [a0?, a1?, a2?, a3?, a4?, a5?, a6?];
            engine::sb_cgt_concessions(&engine::SbCgtConcessionsArgs {
                entity: &a[0],
                share_or_trust_interest: &a[1],
                acquired: &a[2],
                event_date: &a[3],
                partnership_asset: &a[4],
                passively_held: &a[5],
                business: &a[6],
                previous_year_turnover: &a[7],
                year_before_turnover: &a[8],
                likely_turnover: &a[9],
                year_turnover: &a[10],
                net_asset_value: &a[11],
                active_days: &a[12],
                business_ceased: &a[13],
                date_of_birth: &a[14],
                retirement: &a[15],
                permanently_incapacitated: &a[16],
                last_rollover_event: &a[17],
                earnout_latest_due: &a[18],
                year: &a[19],
                scope_confirmed: &a[20],
                optional: given.each_ref().map(Option::as_ref),
            })
        }
        "work-expenses" => {
            // The method's and the year's own inputs, by name.
            let mut given = Vec::new();
            for (key, decimal) in engine::WORK_EXPENSES_OPTIONAL {
                if let Some(arg) = optional(v, key, if decimal { Dec } else { Raw })? {
                    given.push((key.to_string(), arg));
                }
            }
            engine::work_expenses(&a[0], &a[1], &a[2], &given, &a[3], &a[4])
        }
        "private-health-insurance-rebate" => {
            // Optional: absent for a person without a spouse on 30 June or a
            // dependant.
            let o = [
                optional(v, "spouse_income_for_surcharge_purposes", Dec)?,
                optional(v, "dependent_children", Raw)?,
                optional(v, "maintained_dependent_person", Raw)?,
            ];
            engine::private_health_insurance_rebate(&engine::PhiRebateArgs {
                income_for_surcharge_purposes: &a[0],
                lines: &a[1],
                year: &a[2],
                scope_confirmed: &a[3],
                family: engine::SurchargeFamily {
                    spouse_income: o[0].as_ref(),
                    dependent_children: o[1].as_ref(),
                },
                maintained_dependent_person: o[2].as_ref(),
            })
        }
        "income-tests" => {
            let named: Vec<(&str, &Arg)> = params.iter().map(|(key, _)| *key).zip(&a).collect();
            engine::income_tests(&named)
        }
        "smsf-tax" => {
            let named: Vec<(&str, &Arg)> = params.iter().map(|(key, _)| *key).zip(&a).collect();
            engine::smsf_tax(&named)
        }
        "small-business-income-offset" => {
            // Optional: the facts only net small business income of your own calls for.
            let given = engine::SBITO_OPTIONAL.map(|key| {
                let kind = if key.ends_with("_turnover") { Dec } else { Raw };
                optional(v, key, kind)
            });
            let [a0, a1, a2, a3, a4, a5] = given;
            let given = [a0?, a1?, a2?, a3?, a4?, a5?];
            engine::small_business_income_offset(&engine::SmallBusinessIncomeOffsetArgs {
                taxable_income: &a[0],
                basic_income_tax_liability: &a[1],
                offsets_applied_first: &a[2],
                net_small_business_income: &a[3],
                shares: &a[4],
                prescribed_person: &a[5],
                year: &a[6],
                scope_confirmed: &a[7],
                optional: given.each_ref().map(Option::as_ref),
            })
        }
        "trading-stock" => engine::trading_stock(&engine::TradingStockArgs {
            entity: &a[0],
            opening_value: &a[1],
            items: &a[2],
            estimated_closing_value: &a[3],
            simplified_choice: &a[4],
            year: &a[5],
            scope_confirmed: &a[6],
        }),
        "prepayments" => engine::prepayments(&engine::PrepaymentsArgs {
            taxpayer: &a[0],
            individual: &a[1],
            business_expenditure: &a[2],
            chose_82kzmd: &a[3],
            expenditure: &a[4],
            incurred: &a[5],
            service_start: &a[6],
            service_end: &a[7],
            agreement_date: &a[8],
            exclusion: &a[9],
            tax_shelter: &a[10],
            year: &a[11],
            scope_confirmed: &a[12],
        }),
        "non-commercial-losses" => {
            // Optional: the estimate for a part year and the other income of
            // an excepted activity.
            let [estimate, other] = engine::NCL_OPTIONAL.map(|key| optional(v, key, Dec));
            let given = [estimate?, other?];
            engine::non_commercial_losses(&engine::NonCommercialLossesArgs {
                activity: &a[0],
                assessable_income: &a[1],
                deductions: &a[2],
                brought_forward: &a[3],
                part_year: &a[4],
                earlier_years: &a[5],
                real_property_value: &a[6],
                other_assets_value: &a[7],
                taxable_income: &a[8],
                business_losses: &a[9],
                assessable_fhss_released_amount: &a[10],
                reportable_fringe_benefits: &a[11],
                reportable_super_contributions: &a[12],
                total_net_investment_loss: &a[13],
                net_exempt_income: &a[14],
                year: &a[15],
                scope_confirmed: &a[16],
                optional: given.each_ref().map(Option::as_ref),
            })
        }
        "first-home-owner-grant" => {
            // Optional: the Western Australian home's location.
            let [north] = engine::FHOG_OPTIONAL.map(|key| optional(v, key, Raw));
            let given = [north?];
            engine::first_home_owner_grant(&engine::FirstHomeOwnerGrantArgs {
                jurisdiction: &a[0],
                contract_date: &a[1],
                home: &a[2],
                total_value: &a[3],
                facts: &a[4],
                year: &a[5],
                scope_confirmed: &a[6],
                optional: given.each_ref().map(Option::as_ref),
            })
        }
        "psi-test" => {
            // Optional: an agent's commission and the principal's payments.
            let [commission, principal] = engine::PSI_OPTIONAL.map(|key| optional(v, key, Dec));
            let given = [commission?, principal?];
            engine::psi_test(&engine::PsiTestArgs {
                earner: &a[0],
                mainly_personal_efforts: &a[1],
                determination_in_force: &a[2],
                payers: &a[3],
                agent: &a[4],
                principal_work_by_others: &a[5],
                apprentice: &a[6],
                premises: &a[7],
                year: &a[8],
                scope_confirmed: &a[9],
                optional: given.each_ref().map(Option::as_ref),
            })
        }
        "minors-income" => {
            // Optional: the facts some excepted person grounds call for.
            let [full_time, dependent] = engine::MINORS_OPTIONAL.map(|key| {
                optional(
                    v,
                    key,
                    if key == "full_time_occupation" {
                        Json
                    } else {
                        Raw
                    },
                )
            });
            let given = [full_time?, dependent?];
            engine::minors_income(&engine::MinorsIncomeArgs {
                taxable_income: &a[0],
                residency: &a[1],
                under_18_on_30_june: &a[2],
                excepted_person: &a[3],
                excepted_income: &a[4],
                trust_shares: &a[5],
                other_assessable_income: &a[6],
                deductions_exclusive: &a[7],
                deductions_related: &a[8],
                apportionable_deductions: &a[9],
                special_income_component: &a[10],
                year: &a[11],
                scope_confirmed: &a[12],
                optional: given.each_ref().map(Option::as_ref),
            })
        }
        "government-super-amounts" => {
            // Optional: the spouse inputs, all or none.
            let spouse = engine::SPOUSE_INPUTS
                .iter()
                .map(|key| {
                    let kind = if key.ends_with("_cap") { Raw } else { Dec };
                    Ok(optional(v, key, kind)?.map(|arg| (*key, arg)))
                })
                .collect::<Result<Vec<_>>>()?;
            let mut named: Vec<(&str, &Arg)> = params.iter().map(|(key, _)| *key).zip(&a).collect();
            named.extend(spouse.iter().flatten().map(|(key, arg)| (*key, arg)));
            engine::government_super_amounts(&named)
        }
        "residency-tests" => engine::residency_tests(&engine::ResidencyTestsArgs {
            resides: &a[0],
            domicile_in_australia: &a[1],
            permanent_place_of_abode_outside: &a[2],
            presence: &a[3],
            usual_place_of_abode_outside: &a[4],
            intends_to_take_up_residence: &a[5],
            commonwealth_super: &a[6],
            resident_elsewhere: &a[7],
            year: &a[8],
            scope_confirmed: &a[9],
        }),
        "lodgment-required" => engine::lodgment_required(&engine::LodgmentRequiredArgs {
            residency: &a[0],
            taxable_income: &a[1],
            dividends_and_franking_credits: &a[2],
            under_18: &a[3],
            triggers: &a[4],
            study_loan: &a[5],
            working_holiday_maker: &a[6],
            listed_payment: &a[7],
            sapto: &a[8],
            child_support_parent: &a[9],
            year: &a[10],
            scope_confirmed: &a[11],
        }),
        "gst-creditable-purpose-change" => {
            // Optional: when the thing was disposed of or its period ended.
            let ended = optional(v, "ended_on", Raw)?;
            let by_132 = optional(v, "division_132_disposal", Raw)?;
            engine::gst_creditable_purpose_change(&engine::CreditablePurposeArgs {
                gst_exclusive_value: &a[0],
                relates_to_business_finance: &a[1],
                full_input_tax_credit: &a[2],
                intended_or_former_application: &a[3],
                actual_application: &a[4],
                application_basis: &a[5],
                attributed_period_end: &a[6],
                tax_period_months: &a[7],
                ended_on: ended.as_ref(),
                division_132_disposal: by_132.as_ref(),
                year: &a[8],
                scope_confirmed: &a[9],
            })
        }
        "gst-annual-apportionment" => {
            // Optional: a non-business entity's turnover and the extents
            // other than creditable purpose.
            let turnover = optional(v, "gst_turnover", Dec)?;
            let input_taxed = optional(v, "non_input_taxed_extent", Dec)?;
            let consideration = optional(v, "consideration_extent", Dec)?;
            engine::gst_annual_apportionment(&engine::AnnualApportionmentArgs {
                small_business_entity: &a[0],
                carries_on_business: &a[1],
                gst_turnover: turnover.as_ref(),
                instalment_election: &a[2],
                annual_tax_period_election: &a[3],
                election_made: &a[4],
                gst_on_acquisition: &a[5],
                creditable_purpose: &a[6],
                non_input_taxed_extent: input_taxed.as_ref(),
                consideration_extent: consideration.as_ref(),
                car: &a[7],
                reduced_credit_acquisition: &a[8],
                year: &a[9],
                scope_confirmed: &a[10],
            })
        }
        "gst-financial-acquisitions-threshold" => {
            engine::gst_financial_acquisitions_threshold(&engine::FinancialAcquisitionsArgs {
                current_financial_credits: &a[0],
                current_total_credits: &a[1],
                future_financial_credits: &a[2],
                future_total_credits: &a[3],
                gst_group: &a[4],
                year: &a[5],
                scope_confirmed: &a[6],
            })
        }
        "gst-going-concern" => engine::gst_going_concern(&engine::GoingConcernArgs {
            for_consideration: &a[0],
            recipient_registration: &a[1],
            written_agreement: &a[2],
            all_things_necessary: &a[3],
            carried_on_until_supply: &a[4],
            farm_land: &a[5],
            part_outside_arrangement: &a[6],
            year: &a[7],
            scope_confirmed: &a[8],
        }),
        "gst-second-hand-goods" => {
            // Optional: the GST on your taxable supply of the goods.
            let resupply = optional(v, "resupply_gst", Dec)?;
            engine::gst_second_hand_goods(&engine::SecondHandGoodsArgs {
                consideration: &a[0],
                acquired_for_sale_or_exchange: &a[1],
                animals_or_plants: &a[2],
                supply_to_you: &a[3],
                imported: &a[4],
                by_way_of_hire: &a[5],
                resupply_taxable: &a[6],
                resupply_gst: resupply.as_ref(),
                divided_for_resupply: &a[7],
                valuable_metal: &a[8],
                leased_before_sale: &a[9],
                year: &a[10],
                scope_confirmed: &a[11],
            })
        }
        "gst-bad-debt" => {
            // Optional: inputs used only for some parties and events.
            let raw = |key| optional(v, key, Raw);
            let dec = |key| optional(v, key, Dec);
            let attributed = raw("attributed")?;
            let (overdue_from, as_at, prior) = (
                raw("overdue_from")?,
                raw("as_at")?,
                raw("prior_adjustment")?,
            );
            let (proportion, purpose, extent) = (
                dec("taxable_proportion")?,
                dec("creditable_purpose")?,
                dec("consideration_extent")?,
            );
            let (reduced, annual) = (
                raw("reduced_credit_acquisition")?,
                raw("annual_apportionment")?,
            );
            engine::gst_bad_debt(&engine::BadDebtArgs {
                party: &a[0],
                event: &a[1],
                amount: &a[2],
                taxable_or_creditable: &a[3],
                cash_basis: &a[4],
                special_rule: &a[5],
                attributed: attributed.as_ref(),
                overdue_from: overdue_from.as_ref(),
                as_at: as_at.as_ref(),
                prior_adjustment: prior.as_ref(),
                taxable_proportion: proportion.as_ref(),
                creditable_purpose: purpose.as_ref(),
                consideration_extent: extent.as_ref(),
                reduced_credit_acquisition: reduced.as_ref(),
                annual_apportionment: annual.as_ref(),
                year: &a[6],
                scope_confirmed: &a[7],
            })
        }
        "deferred-tax" => engine::deferred_tax(&engine::DeferredTaxArgs {
            items: &a[0],
            unused_tax_losses: &a[1],
            opening_unused_tax_losses: &a[2],
            unrecognised_deductible: &a[3],
            opening_unrecognised_deductible: &a[4],
            unrecognised_used: &a[5],
            accounting_profit: &a[6],
            non_deductible_expenses: &a[7],
            non_assessable_income: &a[8],
            taxable_income: &a[9],
            tax_offsets: &a[10],
            base_rate_entity: &a[11],
            expected_rate: &a[12],
            opening_expected_rate: &a[13],
            business_combination: &a[14],
            uncertain_tax_treatment: &a[15],
            tax_consolidated_group: &a[16],
            outside_profit_or_loss: &a[17],
            year: &a[18],
            scope_confirmed: &a[19],
        }),
        "foreign-income-conversion" => engine::foreign_income_conversion(&a[0], &a[1], &a[2]),
        "foreign-income-tax-offset" => {
            // Optional: the facts only the stated or worked limit takes.
            let given = engine::FITO_OPTIONAL.map(|key| {
                let kind = match key {
                    "residency_periods" => Json,
                    "eligible_pensioner" => Raw,
                    _ => Dec,
                };
                optional(v, key, kind)
            });
            let [a0, a1, a2, a3, a4, a5, a6, a7, a8] = given;
            let given = [a0?, a1?, a2?, a3?, a4?, a5?, a6?, a7?, a8?];
            engine::foreign_income_tax_offset(&engine::ForeignIncomeTaxOffsetArgs {
                foreign_tax_paid: &a[0],
                nane_foreign_tax: &a[1],
                limit_method: &a[2],
                year: &a[3],
                scope_confirmed: &a[4],
                optional: given.each_ref().map(Option::as_ref),
            })
        }
        "luxury-car-tax" => engine::luxury_car_tax(&engine::LctArgs {
            lines: &a[0],
            year: &a[1],
            scope_confirmed: &a[2],
        }),
        _ => unreachable!("listed command"),
    };
    result.or_else(|e| refusal(e.code(), e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_repr_matches_cpython_corpus() {
        // CPython 3.15's repr for doubles chosen to exercise ties between two
        // equally short spellings: bits in hex, then the repr.
        assert_eq!(float_repr(1.0 + 2f64.powi(-17)), "1.0000076293945312");
        let wrong: Vec<String> = include_str!("../tests/data/float_repr.txt")
            .lines()
            .filter_map(|line| {
                let (bits, expected) = line.split_once(' ').expect("bits and repr");
                let actual = float_repr(f64::from_bits(u64::from_str_radix(bits, 16).unwrap()));
                (actual != expected).then(|| format!("{bits}: python {expected}, rust {actual}"))
            })
            .collect();
        assert!(
            wrong.is_empty(),
            "{} wrong: {:?}",
            wrong.len(),
            &wrong[..wrong.len().min(10)]
        );
    }

    /// `takes` reads the catalogue's required inputs so that the FBT benefit
    /// worksheets' named inputs are taken; every other command's required
    /// inputs are its parameters, so no other command takes more than before.
    #[test]
    fn only_the_fbt_benefit_worksheets_take_inputs_beyond_their_parameters() {
        let named_only = [
            "fbt-loan",
            "fbt-meal-entertainment",
            "fbt-car-parking",
            "fbt-debt-waiver",
            "fbt-expense-payment",
            "fbt-property",
            "fbt-residual",
            "fbt-housing",
            "fbt-lafha",
            "fbt-board",
            "fbt-tax-exempt-body-entertainment",
        ];
        for (command, params) in COMMAND_ARGS {
            let extra: Vec<_> = crate::metadata::input_units(&command.replace('-', "_"))
                .into_iter()
                .filter(|(n, _)| !params.iter().any(|(p, _)| p == n))
                .collect();
            assert_eq!(
                extra.is_empty(),
                !named_only.contains(command),
                "{command} {extra:?}"
            );
        }
    }
}
