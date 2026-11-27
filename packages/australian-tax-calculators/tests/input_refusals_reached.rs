// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Inputs that reach the refusals ordinary facts never trip: a non-boolean
//! flag, an unknown key in an item, a missing or misplaced field in a branch,
//! and a valuation the 2021 regulations' tables cannot give. Each case starts
//! from the catalogue's example facts unless it states its own.
use australian_tax_calculators::{api, engine};
use serde_json::{Map, Value, json};
use std::sync::OnceLock;

fn example(kind: &str) -> Map<String, Value> {
    static CATALOGUE: OnceLock<Value> = OnceLock::new();
    let catalogue =
        CATALOGUE.get_or_init(|| api::evaluate("worksheet-catalogue", &json!({})).unwrap());
    let mut facts = catalogue[kind]["example"]["facts"]
        .as_object()
        .unwrap()
        .clone();
    facts.remove("kind");
    facts
}

fn with(kind: &str, changes: Value) -> Value {
    let mut facts = example(kind);
    for (k, v) in changes.as_object().unwrap() {
        if v.is_null() {
            facts.remove(k);
        } else {
            facts.insert(k.clone(), v.clone());
        }
    }
    Value::Object(facts)
}

/// A right acquired on 1 August 2020 whose restrictions lift on 1 August
/// 2023, valued then by the tables from its share's market value; `last` is
/// its last exercise day.
fn early_right(last: &str) -> Value {
    json!({
        "year": "2023-24", "scope_confirmed": true, "income_test": null, "foreign_employment": false,
        "interests": [{
            "kind": "right", "acquired": "2020-08-01", "consideration": "0.00",
            "exercise_price": "0.00", "share_market_value": "3000.00",
            "last_exercise_day": last, "valuation": "market_value",
            "market_value": "1000.00",
            "conditions": {"employed": true, "ordinary_shares_only": true, "not_share_trading_company": true, "minimum_holding_period": true, "ten_percent_limit": true, "non_discriminatory": true, "real_risk_of_forfeiture": true, "salary_sacrifice": false, "deferral_stated": false},
            "start_up": null,
            "restrictions_lifted": "2023-08-01", "exercised": null,
            "share_restrictions_lifted": null, "employment_ended": null, "disposed": null,
            "cost_base": "0.00",
            "taxing_point_value": {"date": "2023-08-01", "share_market_value": "3000.00"},
        }],
    })
}

/// A command, its input, and the refusal's code and message.
type Case = (&'static str, Value, &'static str, &'static str);

fn cases() -> Vec<Case> {
    let alcohol = |key: &str| {
        let mut lines = example("alcohol_excise")["lines"].clone();
        lines[0].as_object_mut().unwrap().remove(key);
        with("alcohol_excise", json!({"lines": lines}))
    };
    let mut items = example("business_capital_expenditure")["items"].clone();
    items[1]["entity_test"] = json!("nope");
    let capital = with("business_capital_expenditure", json!({"items": items}));
    let mut excepted = example("minors_income")["excepted_income"].clone();
    excepted[0]["extra"] = json!("1");
    let mut years = example("non_commercial_losses")["earlier_years"].clone();
    years[0]["extra"] = json!("1");
    let mut sample = example("gst_food_retailer_sam")["purchases_sample"].clone();
    sample["extra"] = json!("1");
    vec![
        // api_lsl
        (
            "unused-leave-payment",
            with(
                "unused_leave_payment",
                json!({"long_service_leave_16_august_1978_to_17_august_1993": "abc"}),
            ),
            "input.decimal_unreadable",
            r#"decimal.InvalidOperation"#,
        ),
        // alcohol_strength
        (
            "alcohol-excise",
            alcohol("strength_percent"),
            "input.missing_field",
            r#"line 1: strength_percent is required"#,
        ),
        // alcohol_container
        (
            "alcohol-excise",
            alcohol("container_litres"),
            "input.missing_field",
            r#"line 1: container_litres is required"#,
        ),
        // capital_entity_test
        (
            "business-capital-expenditure",
            capital,
            "business_capital.fact_invalid",
            r#"item 2: entity_test must be one of small_business_entity, covered_by_2b, not_in_business_and_unconnected, none."#,
        ),
        // ess_last_before
        (
            "ess-discount",
            early_right("2023-07-31"),
            "ess_discount.valuation_unavailable",
            r#"The regulations value a right whose last exercise day is on or after the valuation day and within 15 years after acquisition (s 83A-315.01(1)): give the right's market value."#,
        ),
        // fbt_ev_flag
        (
            "fbt-ev-exemption-check",
            with(
                "fbt_ev_exemption_check",
                json!({"luxury_car_tax_on_other_supply": "yes"}),
            ),
            "input.flag_invalid",
            r#"luxury_car_tax_on_other_supply must be a boolean."#,
        ),
        // gst_annual_flag
        (
            "gst-annual-apportionment",
            with(
                "gst_annual_apportionment",
                json!({"annual_tax_period_election": "yes"}),
            ),
            "input.flag_invalid",
            r#"annual_tax_period_election must be a boolean."#,
        ),
        // gst_div93_flag
        (
            "gst-error-correction",
            with(
                "gst_error_correction",
                json!({"credit_ceased_under_division_93": "yes"}),
            ),
            "input.flag_invalid",
            r#"credit_ceased_under_division_93 must be a boolean."#,
        ),
        // gst_105_65_flag
        (
            "gst-error-correction",
            with(
                "gst_error_correction",
                json!({"unrefundable_under_section_105_65": "yes"}),
            ),
            "input.flag_invalid",
            r#"unrefundable_under_section_105_65 must be a boolean."#,
        ),
        // gst_earlier_flag
        (
            "gst-credit-time-limit",
            with(
                "gst_credit_time_limit",
                json!({"earlier_attributable_return_given": "yes"}),
            ),
            "input.flag_invalid",
            r#"earlier_attributable_return_given must be a boolean."#,
        ),
        // gst_review_flag
        (
            "gst-credit-time-limit",
            with(
                "gst_credit_time_limit",
                json!({"supplier_review_ended_without_invoice": "yes"}),
            ),
            "input.flag_invalid",
            r#"supplier_review_ended_without_invoice must be a boolean."#,
        ),
        // sam_sample_key
        (
            "gst-food-retailer-sam",
            with("gst_food_retailer_sam", json!({"purchases_sample": sample})),
            "input.field_unknown",
            r#"purchases_sample: extra is not an input of a sample"#,
        ),
        // margin_government_flag
        (
            "gst-margin-scheme",
            with(
                "gst_margin_scheme",
                json!({"government_unimproved_on_1_july_2000": "yes"}),
            ),
            "input.flag_invalid",
            r#"government_unimproved_on_1_july_2000 must be a boolean."#,
        ),
        // margin_later_registration
        (
            "gst-margin-scheme",
            with(
                "gst_margin_scheme",
                json!({"improved_on_1_july_2000": true}),
            ),
            "input.field_unknown",
            r#"improved_on_1_july_2000 is an input only for a later registration under item 2"#,
        ),
        // margin_consideration_item_2
        (
            "gst-margin-scheme",
            json!({"interest": "freehold_interest", "agreement": "written_on_or_before_supply", "acquisition": "before_1_july_2000", "consideration": "144000.00", "method": "consideration", "recipient_associate": false, "acquisition_consideration": "75000.00", "unpaid_acquisition_consideration": "0.00", "year": "2025-26", "scope_confirmed": true, "improved_on_1_july_2000": true}),
            "input.field_unknown",
            r#"improved_on_1_july_2000 is an input only with method approved_valuation under item 2"#,
        ),
        // margin_consideration_item_4
        (
            "gst-margin-scheme",
            json!({"interest": "freehold_interest", "agreement": "written_on_or_before_supply", "acquisition": "before_1_july_2000", "consideration": "144000.00", "method": "consideration", "recipient_associate": false, "acquisition_consideration": "75000.00", "unpaid_acquisition_consideration": "0.00", "year": "2025-26", "scope_confirmed": true, "supply_day": "2025-08-01"}),
            "input.field_unknown",
            r#"supply_day is an input only with method approved_valuation under item 4"#,
        ),
        // minors_excepted_key
        (
            "minors-income",
            with("minors_income", json!({"excepted_income": excepted})),
            "input.field_unknown",
            r#"excepted income item 1: extra is not an input of an excepted income item"#,
        ),
        // minors_trust_key
        (
            "minors-income",
            with(
                "minors_income",
                json!({"trust_shares": [{"amount": "100.00", "extra": "1"}]}),
            ),
            "input.field_unknown",
            r#"trust share 1: extra is not an input of a trust share"#,
        ),
        // ncl_year_key
        (
            "non-commercial-losses",
            with("non_commercial_losses", json!({"earlier_years": years})),
            "input.field_unknown",
            r#"earlier year 1: extra is not an input of an earlier year"#,
        ),
        // voluntary_flag
        (
            "voluntary-agreement-withholding",
            with(
                "voluntary_agreement_withholding",
                json!({"agreement_applies_instalment_rate": "yes"}),
            ),
            "input.flag_invalid",
            r#"agreement_applies_instalment_rate must be a boolean."#,
        ),
        // group_dge_unknown
        (
            "payroll-tax-group",
            with(
                "payroll_tax_group",
                json!({"designated_group_employers": ["Z"]}),
            ),
            "payroll_tax_group.person_unknown",
            r#""Z" is not a person listed in persons."#,
        ),
        // group_control_key
        (
            "payroll-tax-group",
            with(
                "payroll_tax_group",
                json!({"controlling_interests": [{"holders": ["A"], "business": "B", "test": "sole_owner", "extra": "1"}]}),
            ),
            "input.field_unknown",
            r#"controlling_interests does not take extra; it takes holders, business, test, percent."#,
        ),
        // group_voting_key
        (
            "payroll-tax-group",
            with(
                "payroll_tax_group",
                json!({"voting_interests": [{"holders": ["A"], "corporation": "B", "percent": "60", "extra": "1"}]}),
            ),
            "input.field_unknown",
            r#"voting_interests does not take extra; it takes holders, corporation, percent."#,
        ),
        // coal_ordinary_wages
        (
            "portable-lsl-levy",
            with("portable_lsl_levy", json!({"ordinary_wages": "100.00"})),
            "portable_lsl_levy.branch_field_mismatch",
            r#"ordinary_wages does not apply to Coal LSL."#,
        ),
        // coal_salary_missing
        (
            "portable-lsl-levy",
            with(
                "portable_lsl_levy",
                json!({"branch": "annual_salary", "base_rate_of_pay": null, "overtime_and_penalty_rates": null, "allowances": null, "allowances_exclude_expense_reimbursements": null}),
            ),
            "input.missing_field",
            r#"annual_salary_paid is required for Coal LSL annual_salary."#,
        ),
        // coal_casual_salary
        (
            "portable-lsl-levy",
            with(
                "portable_lsl_levy",
                json!({"branch": "casual", "base_rate_of_pay": null, "overtime_and_penalty_rates": null, "allowances": null, "allowances_exclude_expense_reimbursements": null, "annual_salary_paid": "100.00"}),
            ),
            "portable_lsl_levy.branch_field_mismatch",
            r#"annual_salary_paid does not apply to Coal LSL casual branch."#,
        ),
        // tas_project_cost
        (
            "portable-lsl-levy",
            json!({"year": "2026-27", "scope_confirmed": true, "jurisdiction": "tas", "scheme": "construction", "project_cost": "100.00", "paid_on_time": true, "ordinary_wages": "100.00"}),
            "portable_lsl_levy.branch_field_mismatch",
            r#"project_cost does not apply to Tasmania construction portable LSL levy."#,
        ),
        // tas_wages_missing
        (
            "portable-lsl-levy",
            json!({"year": "2026-27", "scope_confirmed": true, "jurisdiction": "tas", "scheme": "construction", "paid_on_time": true}),
            "input.missing_field",
            r#"ordinary_wages is required for Tasmania construction portable LSL levy."#,
        ),
        // vacancy_flag
        (
            "residential-vacancy-fee",
            with(
                "residential_vacancy_fee",
                json!({"vacancy_fee_waiver_or_remission": "yes"}),
            ),
            "input.flag_invalid",
            r#"vacancy_fee_waiver_or_remission must be a boolean."#,
        ),
        // packaging_flag
        (
            "salary-packaging",
            with(
                "salary_packaging",
                json!({"meal_entertainment_type_one": "yes"}),
            ),
            "input.flag_invalid",
            r#"meal_entertainment_type_one must be a boolean."#,
        ),
        // smsf_flag
        (
            "smsf-tax",
            with("smsf_tax", json!({"disregarded_small_fund_assets": "yes"})),
            "input.flag_invalid",
            r#"disregarded_small_fund_assets must be a boolean."#,
        ),
        // stream_disability_flag
        (
            "super-withholding",
            json!({"benefit": "1400.00", "interest_value": "1400.00", "interest_tax_free_component": "300.00", "untaxed_element": "0.00", "recipient": "member", "payment_date": "2026-09-01", "date_of_birth": "1968-01-15", "disability_superannuation_benefit": "yes", "income_stream_period": "fortnightly", "tax_free_threshold_claimed": true, "capped_defined_benefit": false, "tfn_provided": true, "foreign_resident": false, "year": "2026-27", "scope_confirmed": true}),
            "input.flag_invalid",
            r#"disability_superannuation_benefit must be a boolean."#,
        ),
        // trade_in_flag
        (
            "trade-in",
            with(
                "trade_in",
                json!({"replacement_creditable_acquisition": "yes"}),
            ),
            "input.flag_invalid",
            r#"replacement_creditable_acquisition must be a boolean."#,
        ),
        // home_office_flag
        (
            "work-expenses",
            with("work_expenses", json!({"running_expenses_incurred": "yes"})),
            "input.flag_invalid",
            r#"running_expenses_incurred must be a boolean."#,
        ),
        // excess_other_question_key
        (
            "excess-contributions",
            with(
                "excess_contributions",
                json!({"question": "concessional_charge", "determination_date": null, "additional_tax": "100.00", "tax_due_date": "2025-09-15"}),
            ),
            "input.field_unknown",
            r#"question concessional_charge: excess is not an input of concessional_charge"#,
        ),
    ]
}

#[test]
fn each_case_is_refused_with_its_code_and_message() {
    for (command, input, code, message) in cases() {
        let refusal = api::evaluate_coded(command, &input).unwrap_err();
        assert_eq!(
            (refusal.code, refusal.message.as_str()),
            (code, message),
            "{command} {input}"
        );
    }
}

/// An application date for a no-notice order is not checked against the
/// vacancy year start, which binds only a notified acquisition or a developer
/// exemption certificate: $500,000 established dwelling, 2024-25 schedule.
#[test]
fn a_no_notice_order_takes_a_later_application_date() {
    let input = with(
        "residential_vacancy_fee",
        json!({"acquisition_basis": "no_notice_order", "application_fee_payable": null,
               "consideration": "500000.00", "dwelling_kind": "established",
               "application_date": "2024-07-01"}),
    );
    let r = api::evaluate("residential-vacancy-fee", &input).unwrap();
    assert_eq!(r["amounts"]["application_fee_base"], "44100.00");
    assert_eq!(r["amounts"]["vacancy_fee_payable"], "88200.00");
}

/// The engine's own interface takes any argument kind, so a text `exclusions`
/// is refused there; the JSON command always passes a JSON value.
#[test]
fn the_engine_refuses_exclusions_that_are_not_a_list() {
    let persons = engine::Arg::Json(json!([
        {"id": "A", "taxable_wages": "1000000.00"},
        {"id": "B", "taxable_wages": "400000.00"},
    ]));
    let related = engine::Arg::Json(json!([["A", "B"]]));
    let exclusions = engine::Arg::Text("A".into());
    let (place, employer, yes, year) = (
        engine::Arg::Text("qld".into()),
        engine::Arg::Text("A".into()),
        engine::Arg::Bool(true),
        engine::Arg::Text("2026-27".into()),
    );
    let refusal = engine::payroll_tax_group(&engine::PayrollTaxGroupArgs {
        jurisdiction: &place,
        employer: &employer,
        persons: &persons,
        related_bodies_corporate: Some(&related),
        common_employees: None,
        controlling_interests: None,
        voting_interests: None,
        exclusions: Some(&exclusions),
        designated_group_employers: None,
        excess_deduction_order: None,
        annual_calculation_allowed: None,
        employed_all_year: &yes,
        year: &year,
        scope_confirmed: &yes,
    })
    .unwrap_err();
    assert_eq!(refusal.code(), "payroll_tax_group.fact_invalid");
    assert_eq!(
        refusal.to_string(),
        "exclusions must be a list of person ids."
    );
}
