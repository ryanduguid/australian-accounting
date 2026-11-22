// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `fbt-return` for a rebatable employer (FBTAA s 65J) and for the employers
//! whose benefits s 57A exempts up to a cap (s 5B(1D) to (1M)), against the
//! ATO's 'Worked examples for NFP employers completing FBT return 2026', and
//! items 20 to 22 from the instalments reported (docs/calculation-evidence.md#fbt-return).
//! The amounts use the statutory gross-up formulas at full precision, as
//! `fbt` does, so they differ by cents or a dollar from the examples, which
//! multiply by 2.0802 and 1.8868; each difference is noted.
use australian_tax_calculators::api::{self, Refusal};
use serde_json::{Value, json};

fn benefit(employee: &str, command: &str, input: Value, gst_type: &str, excluded: bool) -> Value {
    json!({"employee": employee, "command": command, "input": input,
           "gst_type": gst_type, "excluded": excluded})
}

fn packaged(mut b: Value, marker: &str) -> Value {
    b["entertainment"] = json!(marker);
    b
}

/// A car under the statutory formula for the whole year: 20% of `cost`.
fn car(cost: &str) -> Value {
    json!({"cost_price": cost, "later_accessories": "0.00", "one_third_reduction": false,
           "days_available": 365, "employee_contribution": "0.00", "electric_car_exempt": false})
}

/// An external expense payment with no deductible share: its taxable value is `paid`.
fn expense(paid: &str) -> Value {
    json!({"method": "external", "recipients_expenditure": paid, "amount_paid": paid,
           "employee_contribution": "0.00", "income_use": "0",
           "recipients_portion_deduction": "0.00"})
}

fn entertainment(spent: &str) -> Value {
    json!({"method": "actual", "entertainment_expenditure": spent})
}

fn top(kind: &str, benefits: Vec<Value>) -> Value {
    json!({"employer": "Employer", "year_ended": 2026, "scope_confirmed": true,
           "employer_kind": kind, "benefits": benefits})
}

fn compose(input: &Value) -> Value {
    api::evaluate_coded("fbt-return", input).unwrap()
}

fn refused(input: &Value) -> Refusal {
    api::evaluate_coded("fbt-return", input).unwrap_err()
}

fn labels(result: &Value, expected: &[(&str, &str)]) {
    for (label, value) in expected {
        assert_eq!(result["return_labels"][label], *value, "{label}");
    }
}

/// Worked example 1: school fees of $6,000 for Mark, Sam and 8 others (type
/// 2); cars of $14,000 and $15,000 for Mark and Sam (type 1); salary packaged
/// restaurant meals of $5,500 and $2,295 (type 2).
fn example_1() -> Vec<Value> {
    let mut out = vec![
        benefit(
            "Mark",
            "fbt-car-statutory",
            car("70000.00"),
            "type_1",
            false,
        ),
        benefit("Sam", "fbt-car-statutory", car("75000.00"), "type_1", false),
    ];
    for (who, spent) in [("Mark", "5500.00"), ("Sam", "2295.00")] {
        let meals = benefit(
            who,
            "fbt-tax-exempt-body-entertainment",
            entertainment(spent),
            "type_2",
            false,
        );
        out.push(packaged(meals, "salary_packaged"));
    }
    let others = (3..=10).map(|n| format!("Employee {n}"));
    for who in ["Mark".to_string(), "Sam".to_string()]
        .into_iter()
        .chain(others)
    {
        out.push(benefit(
            &who,
            "fbt-expense-payment",
            expense("6000.00"),
            "type_2",
            false,
        ));
    }
    out
}

#[test]
fn a_rebatable_employer_matches_worked_example_1() {
    let mut input = top("rebatable", example_1());
    input["instalments_reported"] = json!("16000.00");
    let result = compose(&input);
    assert_eq!(result["employer_kind"], "rebatable");
    // The ATO's 2.0802 and 1.8868 give $60,325, $188,240, $88,472.80,
    // $13,322.05, $35,320.85, $53,151.95 and $37,151.95.
    labels(
        &result,
        &[
            ("14A", "60326.00"),
            ("14B", "127915.00"),
            ("15", "188241.00"),
            ("16", "88473.27"),
            ("17", "13322.21"),
            ("18", "35321.00"),
            ("19", "53152.27"),
            ("20", "16000.00"),
            ("21", "37152.27"),
        ],
    );
    assert!(result["return_labels"].get("14C").is_none());
    assert!(result["return_labels"].get("22").is_none());
    // Mark: $50,821.13 less $30,000 less $5,000 of his $10,377.36 of meals;
    // Sam: $46,854.18 less $30,000 less all $4,330.19 of his (the ATO:
    // $50,821.00, $42,523.80 without the meals, and $4,330.20).
    let capping = &result["capping"]["employees"];
    assert_eq!(
        capping["Mark"],
        json!({"individual_grossed_up_amount": "50821.13",
               "salary_packaged_entertainment_grossed_up": "10377.36",
               "amount_over_caps": "15821.13"})
    );
    assert_eq!(capping["Sam"]["amount_over_caps"], "12523.99");
    assert_eq!(capping["Employee 3"]["amount_over_caps"], "0.00");
    assert_eq!(
        result["amounts"]["aggregate_non_rebatable_amount"],
        "13322.21"
    );
    assert_eq!(result["amounts"]["rebate"], "35321.00");
    assert!(result["left_to_preparer"].get("20 to 22").is_none());
    assert!(result["left_to_preparer"].get("14C, 17 and 18").is_none());
    assert_eq!(result["source_checked"], "2026-11-22");
}

#[test]
fn the_rebate_takes_the_rebatable_days() {
    // s 65J(2A): 0.47 x ($88,473.27 - $13,322.21) x 182 / 365.
    let mut input = top("rebatable", example_1());
    input["rebatable_days"] = json!(182);
    input["employer_days"] = json!(365);
    let result = compose(&input);
    labels(&result, &[("18", "17612.11"), ("19", "70861.16")]);
    let warned = result["warnings"].to_string();
    assert!(!warned.contains("rebatable_days and employer_days were not given"));
    let whole = compose(&top("rebatable", example_1()));
    assert!(
        whole["warnings"][0]
            .as_str()
            .unwrap()
            .starts_with("rebatable_days and employer_days were not given")
    );
}

/// Worked examples 2 and 3: cars of $2,000 and $25,000 for Louise and Wendy
/// (type 1); Louise's salary packaged room hire of $5,000 and Wendy's
/// reimbursed houseboat rent of $15,000, not salary packaged (type 2).
fn examples_2_and_3() -> Vec<Value> {
    let hire = benefit(
        "Louise",
        "fbt-tax-exempt-body-entertainment",
        entertainment("5000.00"),
        "type_2",
        false,
    );
    let houseboat = benefit(
        "Wendy",
        "fbt-expense-payment",
        expense("15000.00"),
        "type_2",
        true,
    );
    vec![
        benefit(
            "Louise",
            "fbt-car-statutory",
            car("10000.00"),
            "type_1",
            false,
        ),
        packaged(hire, "salary_packaged"),
        benefit(
            "Wendy",
            "fbt-car-statutory",
            car("125000.00"),
            "type_1",
            false,
        ),
        packaged(houseboat, "not_salary_packaged"),
    ]
}

#[test]
fn a_public_benevolent_institution_matches_worked_example_2() {
    let mut input = top("public_benevolent_institution", examples_2_and_3());
    input["instalments_reported"] = json!("16000.00");
    let result = compose(&input);
    // The ATO's amounts are the same: Wendy's $52,005.40 less $30,000.
    labels(
        &result,
        &[
            ("14C", "22005.00"),
            ("15", "22005.00"),
            ("16", "10342.35"),
            ("19", "10342.35"),
            ("20", "16000.00"),
            ("22", "5657.65"),
        ],
    );
    for absent in ["14A", "14B", "17", "18", "21", "23E"] {
        assert!(result["return_labels"].get(absent).is_none(), "{absent}");
    }
    let capping = &result["capping"];
    assert_eq!(capping["cap"], "30000.00");
    assert_eq!(
        capping["employees"]["Louise"],
        json!({"individual_grossed_up_amount": "13594.39",
               "salary_packaged_entertainment_grossed_up": "9433.96",
               "amount_over_caps": "0.00"})
    );
    assert_eq!(
        capping["employees"]["Wendy"]["individual_grossed_up_amount"],
        "52005.40"
    );
    assert_eq!(result["amounts"]["aggregate_non_exempt_amount"], "22005.40");
    assert!(result["amounts"].get("type_one_grossed_up").is_none());
    assert!(result["components"].get("fbt").is_none());
    // Item 23 as the example's table 2: the houseboat is left out.
    assert_eq!(
        result["return_labels"]["23A"],
        json!({"gross_taxable_value": "27000.00", "employee_contribution": "0.00",
               "taxable_value": "27000.00"})
    );
    assert_eq!(
        result["return_labels"]["23L"],
        json!({"gross_taxable_value": "5000.00", "taxable_value": "5000.00"})
    );
}

#[test]
fn a_public_hospital_matches_worked_example_3() {
    let mut input = top("public_hospital", examples_2_and_3());
    input["instalments_reported"] = json!("16000.00");
    let result = compose(&input);
    // Wendy's $52,005.40 less $17,000; the ATO gives the same amounts.
    labels(
        &result,
        &[
            ("14C", "35005.00"),
            ("15", "35005.00"),
            ("16", "16452.35"),
            ("19", "16452.35"),
            ("21", "452.35"),
        ],
    );
    assert_eq!(result["capping"]["cap"], "17000.00");
}

#[test]
fn each_s_57a_kind_takes_its_cap() {
    for (kind, cap) in [
        ("public_benevolent_institution", "30000.00"),
        ("health_promotion_charity", "30000.00"),
        ("public_hospital", "17000.00"),
        ("not_for_profit_hospital", "17000.00"),
        ("public_ambulance_service", "17000.00"),
    ] {
        let result = compose(&top(kind, examples_2_and_3()));
        assert_eq!(result["capping"]["cap"], cap, "{kind}");
        assert_eq!(result["employer_kind"], kind);
    }
}

fn one_employee(kind: &str, base: &str, meals: Option<&str>) -> Value {
    let mut benefits = vec![benefit(
        "A",
        "fbt-expense-payment",
        expense(base),
        "type_2",
        false,
    )];
    if let Some(spent) = meals {
        let b = benefit(
            "A",
            "fbt-tax-exempt-body-entertainment",
            entertainment(spent),
            "type_2",
            false,
        );
        benefits.push(packaged(b, "salary_packaged"));
    }
    compose(&top(kind, benefits))["capping"]["employees"]["A"].clone()
}

#[test]
fn the_caps_hold_at_their_boundaries() {
    // $15,900 / 0.53 is $30,000 exactly: nothing over the cap.
    let at = one_employee("public_benevolent_institution", "15900.00", None);
    assert_eq!(at["individual_grossed_up_amount"], "30000.00");
    assert_eq!(at["amount_over_caps"], "0.00");
    let over = one_employee("public_benevolent_institution", "15900.01", None);
    assert_eq!(over["amount_over_caps"], "0.02");
    // $8,427 / 0.53 = $15,900 against the $17,000 hospital cap.
    let under = one_employee("public_hospital", "8427.00", None);
    assert_eq!(under["individual_grossed_up_amount"], "15900.00");
    assert_eq!(under["amount_over_caps"], "0.00");
    let hospital = one_employee("public_hospital", "15900.00", None);
    assert_eq!(hospital["amount_over_caps"], "13000.00");
}

#[test]
fn salary_packaged_entertainment_comes_off_up_to_5000() {
    // $20,000 + $4,000 of meals: $45,283.02 less $30,000 less $5,000 of the
    // meals' $7,547.17.
    let capped = one_employee("public_benevolent_institution", "20000.00", Some("4000.00"));
    assert_eq!(
        capped["salary_packaged_entertainment_grossed_up"],
        "7547.17"
    );
    assert_eq!(capped["amount_over_caps"], "10283.02");
    // Under $5,000 of meals, all of them come off.
    let all = one_employee("public_benevolent_institution", "20000.00", Some("2000.00"));
    assert_eq!(all["amount_over_caps"], "7735.85");
    // Not below nil: $1,000 over the cap, less $5,000.
    let nil = one_employee("public_benevolent_institution", "12430.00", Some("4000.00"));
    assert_eq!(nil["individual_grossed_up_amount"], "31000.00");
    assert_eq!(nil["amount_over_caps"], "0.00");
    // The rebatable employer's cap is $30,000 too, with the same $5,000.
    let rebatable = one_employee("rebatable", "20000.00", Some("4000.00"));
    assert_eq!(rebatable["amount_over_caps"], "10283.02");
}

#[test]
fn the_caps_disregard_car_parking_and_meal_entertainment_only() {
    let benefits = vec![
        benefit("A", "fbt-car-statutory", car("125000.00"), "type_1", false),
        benefit(
            "A",
            "fbt-expense-payment",
            expense("3000.00"),
            "type_2",
            true,
        ),
        benefit(
            "A",
            "fbt-car-parking",
            json!({"method": "commercial_parking_station", "first_business_day_fee": "18.00",
                   "benefits": 230, "lowest_fee": "18.00",
                   "employee_contribution": {"capped_total": "1000.00"}}),
            "type_1",
            true,
        ),
        benefit(
            "A",
            "fbt-meal-entertainment",
            json!({"method": "50_50_split", "meal_entertainment_expenditure": "2000.00"}),
            "type_1",
            true,
        ),
    ];
    let result = compose(&top("public_benevolent_institution", benefits));
    // The excluded expense payment counts (s 5B(1L)): $52,005.40 + $5,660.38.
    assert_eq!(
        result["capping"]["employees"]["A"]["amount_over_caps"],
        "27665.78"
    );
    labels(&result, &[("14C", "27666.00")]);
    for absent in ["23N", "23P"] {
        assert!(result["return_labels"].get(absent).is_none(), "{absent}");
    }
    assert_eq!(result["return_labels"]["23E"]["taxable_value"], "3000.00");
}

#[test]
fn a_taxable_employer_named_gives_the_same_return() {
    let benefits = vec![benefit(
        "A",
        "fbt-expense-payment",
        expense("6000.00"),
        "type_2",
        false,
    )];
    let named = compose(&top("taxable", benefits.clone()));
    let unnamed = api::evaluate_coded(
        "fbt-return",
        &json!({"employer": "Employer", "year_ended": 2026, "scope_confirmed": true,
                "benefits": benefits}),
    )
    .unwrap();
    assert_eq!(named, unnamed);
}

#[test]
fn a_taxable_employer_takes_instalments() {
    let benefits = vec![benefit(
        "A",
        "fbt-expense-payment",
        expense("6000.00"),
        "type_2",
        false,
    )];
    // 6,000 / 0.53 = $11,320.75, so $11,321 and $5,320.87 of tax.
    let mut input = top("taxable", benefits);
    for (reported, label, value) in [
        ("5000.00", "21", "320.87"),
        ("6000.00", "22", "679.13"),
        ("5320.87", "20", "5320.87"),
    ] {
        input["instalments_reported"] = json!(reported);
        let result = compose(&input);
        labels(
            &result,
            &[("19", "5320.87"), ("20", reported), (label, value)],
        );
        if reported == "5320.87" {
            assert!(result["return_labels"].get("21").is_none());
            assert!(result["return_labels"].get("22").is_none());
        }
    }
}

#[test]
fn refuses_what_it_does_not_compose() {
    let base = || top("public_benevolent_institution", examples_2_and_3());
    let with = |key: &str, value: Value| {
        let mut input = base();
        input[key] = value;
        input
    };
    let first = |field: &str, value: Value| {
        let mut input = base();
        input["benefits"][0][field] = value;
        input
    };
    let cases = [
        (
            with("employer_kind", json!("government_body")),
            "fbt_return.employer_kind_unsupported",
        ),
        (
            with("employer_kind", json!(true)),
            "fbt_return.employer_kind_unsupported",
        ),
        (with("rebatable_days", json!(100)), "input.field_unknown"),
        (
            with("instalments_reported", json!("-1.00")),
            "input.amount_invalid",
        ),
        (
            with("instalments_reported", json!(5)),
            "input.amount_invalid",
        ),
        (
            with("instalments_reported", json!("1.001")),
            "input.amount_invalid",
        ),
        (with("year_ended", json!(2025)), "period.unsupported"),
        (
            first("entertainment", json!("maybe")),
            "fbt_return.entertainment_invalid",
        ),
        (
            first("entertainment", json!("not_salary_packaged")),
            "fbt_return.entertainment_invalid",
        ),
    ];
    for (input, code) in cases {
        assert_eq!(refused(&input).code, code, "{input}");
    }
    let mut excluded = base();
    excluded["benefits"][1]["excluded"] = json!(true);
    assert_eq!(refused(&excluded).code, "fbt_return.entertainment_invalid");
    let mut parking = base();
    parking["benefits"][1]["command"] = json!("fbt-meal-entertainment");
    parking["benefits"][1]["excluded"] = json!(true);
    assert_eq!(refused(&parking).code, "fbt_return.entertainment_invalid");
    let mut taxable = base();
    taxable["employer_kind"] = json!("taxable");
    assert_eq!(refused(&taxable).code, "input.field_unknown");
    let mut ev = base();
    ev["benefits"][0]["input"]["electric_car_exempt"] = json!(true);
    assert_eq!(refused(&ev).code, "fbt_return.nfp_exempt_car_unsupported");
}

#[test]
fn refuses_rebatable_days_that_do_not_fit_the_year() {
    for (rebatable, employer) in [
        (json!(366), json!(366)),
        (json!(200), json!(100)),
        (json!(-1), json!(100)),
        (json!(0), json!(0)),
        (json!("100"), json!(365)),
        (json!(100), Value::Null),
    ] {
        let mut input = top("rebatable", example_1());
        input["rebatable_days"] = rebatable;
        if !employer.is_null() {
            input["employer_days"] = employer;
        }
        assert_eq!(
            refused(&input).code,
            "fbt_return.rebatable_days_invalid",
            "{input}"
        );
    }
    let mut zero = top("rebatable", example_1());
    zero["rebatable_days"] = json!(0);
    zero["employer_days"] = json!(365);
    labels(&compose(&zero), &[("18", "0.00"), ("19", "88473.27")]);
}
