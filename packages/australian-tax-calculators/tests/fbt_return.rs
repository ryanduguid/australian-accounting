// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `fbt-return` against the ATO's worked examples for the gross-up, item 23
//! and reportable fringe benefits amounts, the reportable amount rules of
//! FBTAA s 135P (compilation 97, 1 July 2026), its refusals, and a seeded
//! property holding the composed totals to the benefits
//! (docs/calculation-evidence.md#fbt-return).
use australian_tax_calculators::api::{self, Refusal};
use australian_tax_calculators::decimal::Decimal;
use serde_json::{Value, json};

fn benefit(employee: &str, command: &str, input: Value, gst_type: &str, excluded: bool) -> Value {
    json!({"employee": employee, "command": command, "input": input,
           "gst_type": gst_type, "excluded": excluded})
}

fn compose_year(benefits: Vec<Value>, year: i64) -> Result<Value, Refusal> {
    api::evaluate_coded(
        "fbt-return",
        &json!({"employer": "Employer", "year_ended": year, "scope_confirmed": true,
                "benefits": benefits}),
    )
}

fn compose(benefits: Vec<Value>) -> Result<Value, Refusal> {
    compose_year(benefits, 2026)
}

fn refused(input: Value) -> Refusal {
    api::evaluate_coded("fbt-return", &input).unwrap_err()
}

/// A car under the statutory formula for the whole year: 20% of `cost`.
fn car(cost: &str, exempt: bool) -> Value {
    json!({"cost_price": cost, "later_accessories": "0.00", "one_third_reduction": false,
           "days_available": 365, "employee_contribution": "0.00", "electric_car_exempt": exempt})
}

/// An external expense payment with no deductible share: its taxable value is `paid`.
fn expense(paid: &str) -> Value {
    json!({"method": "external", "recipients_expenditure": paid, "amount_paid": paid,
           "employee_contribution": "0.00", "income_use": "0",
           "recipients_portion_deduction": "0.00"})
}

fn waiver(amount: &str) -> Value {
    json!({"amount_waived": amount})
}

/// FBT return 2026 instructions, examples 2 and 4: a $10,000 statutory formula
/// car and $1,000 of meal entertainment, type 1; $6,000 of school fees and
/// $3,000 of remote area rent, type 2. The meal entertainment and the rent are
/// excluded benefits. The example gives only taxable values; the inputs here
/// are ones that produce them.
fn examples_2_and_4() -> Vec<Value> {
    vec![
        benefit(
            "A",
            "fbt-car-statutory",
            car("50000.00", false),
            "type_1",
            false,
        ),
        benefit(
            "staff",
            "fbt-meal-entertainment",
            json!({"method": "50_50_split", "meal_entertainment_expenditure": "2000.00"}),
            "type_1",
            true,
        ),
        benefit(
            "B",
            "fbt-expense-payment",
            expense("6000.00"),
            "type_2",
            false,
        ),
        benefit(
            "C",
            "fbt-expense-payment",
            expense("3000.00"),
            "type_2",
            true,
        ),
    ]
}

#[test]
fn grosses_up_as_the_fbt_return_2026_examples_2_and_4_do() {
    // $11,000 x 2.0802 = $22,882.20 and $9,000 x 1.8868 = $16,981.20; items 14A
    // and 14B in whole dollars, so item 15 is $39,863 and the subtotal at items
    // 16 and 19 is $18,735.61, the figure the instructions give.
    let result = compose(examples_2_and_4()).unwrap();
    assert_eq!(result["calculation"], "fbt_return");
    assert_eq!(result["period"], "year ended 31 March 2026");
    assert_eq!(
        result["amounts"],
        json!({"type_one_value": "11000.00", "type_two_value": "9000.00",
               "type_one_grossed_up": "22882.20", "type_two_grossed_up": "16981.20",
               "fringe_benefits_taxable_amount": "39863.00", "fbt_payable": "18735.61",
               "employee_contributions": "0.00"})
    );
    let labels = &result["return_labels"];
    for (label, expected) in [
        ("14A", "22882.00"),
        ("14B", "16981.00"),
        ("15", "39863.00"),
        ("16", "18735.61"),
        ("19", "18735.61"),
    ] {
        assert_eq!(labels[label], expected, "{label}");
    }
    assert_eq!(
        result["reconciliation"],
        json!({"benefits_taxable_value": "20000.00",
               "type_one_value_plus_type_two_value": "20000.00", "difference": "0.00"})
    );
    // `fbt` is called once, on the aggregate amounts, and each benefit's
    // worksheet result is kept.
    assert_eq!(result["components"]["fbt"]["calculation"], "fbt");
    assert_eq!(
        result["components"]["benefits"].as_array().unwrap().len(),
        4
    );
    // Excluded benefits are taxed but not reported.
    assert_eq!(
        result["employees"]["C"]["individual_fringe_benefits_amount"],
        "0.00"
    );
    // s 135P(2): $10,000 / (1 - 0.47) = $18,867.92, so $18,867; the ATO's
    // 1.8868 would give $18,868.
    assert_eq!(
        result["employees"]["A"]["reportable_fringe_benefits_amount"],
        "18867.00"
    );
    // Item 23P has no Number, employee contribution or reductions column.
    assert_eq!(
        labels["23P"],
        json!({"gross_taxable_value": "1000.00", "taxable_value": "1000.00"})
    );
}

/// FBT return 2026 instructions, item 23 examples 6 to 12, each through the
/// worksheet whose own tests hold it to that example. Example 12's allowance
/// is the 40 weeks the s 31 reduction covers and the 12 weeks it does not.
fn examples_6_to_12() -> Vec<Value> {
    let lafha_40_weeks = json!({
        "method": "maintains_home", "allowance": "23640.00",
        "accommodation_component": "14000.00", "accommodation_expenses": "14000.00",
        "food_component": "9640.00", "food_expenses": "9640.00", "normal_food_amount": "0.00",
        "adult_days": 280, "child_days": 0});
    let mut statutory = car("32000.00", false);
    statutory["days_available"] = json!(274);
    vec![
        benefit("E6", "fbt-car-statutory", statutory, "type_1", false),
        benefit(
            "E7",
            "fbt-car-operating-cost",
            json!({"running_costs": "10000.00", "lease_charges": "0.00",
                   "depreciated_value": "0.00", "effective_life_years": "8", "days_held": 365,
                   "business_use": "0.7", "employee_contribution": "0.00",
                   "statutory_formula_taxable_value": "8500.00", "electric_car_exempt": false}),
            "type_1",
            false,
        ),
        benefit(
            "E8",
            "fbt-loan",
            json!({"daily_balance_total": "7300000.00", "interest_accrued": "0.00",
                   "income_use": "0", "accrued_interest_deduction": "0.00"}),
            "type_2",
            false,
        ),
        benefit("E9", "fbt-debt-waiver", waiver("500.00"), "type_2", false),
        benefit(
            "E10",
            "fbt-expense-payment",
            json!({"method": "external", "recipients_expenditure": "1200.00",
                   "amount_paid": "1200.00", "employee_contribution": "0.00",
                   "income_use": "0.6", "recipients_portion_deduction": "0.00"}),
            "type_1",
            false,
        ),
        benefit(
            "E11",
            "fbt-housing",
            json!({"method": "base_year", "market_value": "41600.00", "tenancy_days": 365,
                   "employee_contribution": "5200.00"}),
            "type_2",
            false,
        ),
        benefit("E12", "fbt-lafha", lafha_40_weeks, "type_2", false),
        benefit(
            "E12",
            "fbt-lafha",
            json!({"method": "other", "allowance": "7092.00"}),
            "type_2",
            false,
        ),
    ]
}

/// An item 23 row; `None` is a column the instructions leave blank, which the
/// result leaves out.
fn row(
    number: Option<u64>,
    gross: &str,
    contribution: Option<&str>,
    reductions: Option<&str>,
    taxable: &str,
) -> Value {
    let mut row = json!({"gross_taxable_value": gross, "taxable_value": taxable});
    for (key, value) in [
        ("number", number.map(|n| json!(n))),
        ("employee_contribution", contribution.map(|c| json!(c))),
        ("value_of_reductions", reductions.map(|r| json!(r))),
    ] {
        if let Some(value) = value {
            row[key] = value;
        }
    }
    row
}

#[test]
fn item_23_matches_the_fbt_return_2026_examples_6_to_12() {
    // The ATO writes example 6's $4,804.38 in whole dollars as $4,804. Each
    // column an example says to leave blank is absent, and the Number of
    // cars, loans and units is left to the preparer.
    let result = compose(examples_6_to_12()).unwrap();
    let labels = &result["return_labels"];
    let expected = [
        ("23A", row(None, "4804.38", Some("0.00"), None, "4804.38")),
        ("23B", row(None, "3000.00", Some("0.00"), None, "3000.00")),
        ("23C", row(None, "1724.00", None, Some("0.00"), "1724.00")),
        ("23D", row(None, "500.00", None, None, "500.00")),
        (
            "23E",
            row(None, "1200.00", Some("0.00"), Some("720.00"), "480.00"),
        ),
        (
            "23F",
            row(None, "41600.00", Some("5200.00"), None, "36400.00"),
        ),
        (
            "23G",
            row(Some(1), "30732.00", None, Some("21960.00"), "8772.00"),
        ),
    ];
    for (label, row) in &expected {
        assert_eq!(&labels[*label], row, "{label}");
    }
    let rows = labels
        .as_object()
        .unwrap()
        .keys()
        .filter(|k| k.starts_with("23"));
    assert_eq!(rows.count(), expected.len());
    // The contribution summary: by employee, by label and in total.
    assert_eq!(
        result["employees"]["E11"]["employee_contributions"],
        "5200.00"
    );
    assert_eq!(result["amounts"]["employee_contributions"], "5200.00");
    // Item 23's taxable values add up to the benefits'.
    let item_23: Decimal = expected
        .iter()
        .map(|(_, r)| Decimal::parse(r["taxable_value"].as_str().unwrap()).unwrap())
        .fold(Decimal::from_int(0), |a, b| a.add(&b).unwrap());
    let total = Decimal::parse(
        result["reconciliation"]["benefits_taxable_value"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert!(item_23.equals(&total).unwrap(), "{item_23} {total}");
}

#[test]
fn reports_derek_as_the_ato_reportable_fringe_benefits_example_does() {
    // ATO, Reportable fringe benefits, step 3 example (FBT year ending 31 March
    // 2022, at the 47% rate and 1.8868 that also apply to 2026): car parking
    // $450, not reportable; a car $3,000 and home internet $500, so $3,500 x
    // 1.8868 = $6,603.
    let parking = json!({"method": "commercial_parking_station",
                         "first_business_day_fee": "18.00", "benefits": 25,
                         "lowest_fee": "18.00", "employee_contribution": "0.00"});
    let result = compose(vec![
        benefit("Derek", "fbt-car-parking", parking, "type_1", true),
        benefit(
            "Derek",
            "fbt-car-statutory",
            car("15000.00", false),
            "type_1",
            false,
        ),
        benefit(
            "Derek",
            "fbt-expense-payment",
            expense("500.00"),
            "type_1",
            false,
        ),
    ])
    .unwrap();
    assert_eq!(
        result["reconciliation"]["benefits_taxable_value"],
        "3950.00"
    );
    assert_eq!(
        result["employees"]["Derek"],
        json!({"individual_fringe_benefits_amount": "3500.00",
               "reportable_fringe_benefits_amount": "6603.00",
               "employee_contributions": "0.00"})
    );
    // Item 23N has no Number or reductions column.
    assert_eq!(
        result["return_labels"]["23N"],
        json!({"gross_taxable_value": "450.00", "employee_contribution": "0.00",
               "taxable_value": "450.00"})
    );
}

#[test]
fn grosses_up_a_reportable_amount_by_the_s_135p_2_formula() {
    // The employer's guide, 5.8: a house with a market value of $15,000 for
    // the year, $15,000 x 1.8868 = $28,302. s 135P(2)'s formula gives
    // $15,000 / (1 - 0.47) = $28,301.89, so $28,301 with the cents dropped.
    // $10,599.96 gives $19,999.92 by the formula and $20,000.00 by 1.8868.
    let house = json!({"method": "base_year", "market_value": "15000.00",
                       "tenancy_days": 365, "employee_contribution": "0.00"});
    let result = compose(vec![
        benefit("Manager", "fbt-housing", house, "type_2", false),
        benefit(
            "Near",
            "fbt-debt-waiver",
            waiver("10599.96"),
            "type_2",
            false,
        ),
    ])
    .unwrap();
    let reported = |e: &str| result["employees"][e]["reportable_fringe_benefits_amount"].clone();
    assert_eq!(reported("Manager"), "28301.00");
    assert_eq!(reported("Near"), "19999.00");
}

#[test]
fn reports_the_employer_guide_5_4_example() {
    // Fringe benefits tax - a guide for employers, 5.4: a car $1,440, holiday
    // accommodation $600, and HELP debt reimbursements of $300 and, for the
    // spouse, $550, all allocated to the employee: $2,890 x 1.8868 = $5,452.
    let holiday = json!({"method": "external_notional_value", "notional_value": "600.00",
                         "employee_contribution": "0.00", "income_use": "0",
                         "contribution_deduction": "0.00"});
    let result = compose(vec![
        benefit(
            "E",
            "fbt-car-statutory",
            car("7200.00", false),
            "type_1",
            false,
        ),
        benefit("E", "fbt-residual", holiday, "type_1", false),
        benefit(
            "E",
            "fbt-expense-payment",
            expense("300.00"),
            "type_2",
            false,
        ),
        benefit(
            "E",
            "fbt-expense-payment",
            expense("550.00"),
            "type_2",
            false,
        ),
    ])
    .unwrap();
    assert_eq!(
        result["employees"]["E"]["individual_fringe_benefits_amount"],
        "2890.00"
    );
    assert_eq!(
        result["employees"]["E"]["reportable_fringe_benefits_amount"],
        "5452.00"
    );
}

#[test]
fn reports_only_above_2000_as_s_135p_1_examples_1_and_2_do() {
    // s 135P(1) example 1: a $2,545 debt waiver gives a reportable amount
    // ($2,545 x 1.8868 = $4,801.906, so $4,801); example 2: $1,900 does not.
    // Exactly $2,000 is not more than $2,000.
    let result = compose(vec![
        benefit(
            "Angela",
            "fbt-debt-waiver",
            waiver("2545.00"),
            "type_2",
            false,
        ),
        benefit(
            "Neil's employee",
            "fbt-debt-waiver",
            waiver("1900.00"),
            "type_2",
            false,
        ),
        benefit(
            "At 2000",
            "fbt-debt-waiver",
            waiver("2000.00"),
            "type_2",
            false,
        ),
        benefit(
            "Above",
            "fbt-debt-waiver",
            waiver("2000.01"),
            "type_2",
            false,
        ),
    ])
    .unwrap();
    let reported = |e: &str| result["employees"][e]["reportable_fringe_benefits_amount"].clone();
    assert_eq!(reported("Angela"), "4801.00");
    assert_eq!(reported("Neil's employee"), "0.00");
    assert_eq!(reported("At 2000"), "0.00");
    assert_eq!(reported("Above"), "3773.00");
}

#[test]
fn an_exempt_electric_car_is_reported_but_not_taxed() {
    // s 135P(3) disregards the s 8A exemption for reporting: the car's $10,000
    // notional value is reportable, and it is in neither aggregate nor item 23.
    let result = compose(vec![benefit(
        "EV",
        "fbt-car-statutory",
        car("50000.00", true),
        "type_1",
        false,
    )])
    .unwrap();
    assert_eq!(result["amounts"]["type_one_value"], "0.00");
    assert_eq!(result["amounts"]["fbt_payable"], "0.00");
    assert!(result["return_labels"].get("23A").is_none());
    assert_eq!(
        result["employees"]["EV"]["individual_fringe_benefits_amount"],
        "10000.00"
    );
    assert_eq!(
        result["employees"]["EV"]["reportable_fringe_benefits_amount"],
        "18867.00"
    );
}

#[test]
fn a_later_year_says_its_labels_follow_the_2026_instructions() {
    let warned = |year: i64| {
        let result = compose_year(
            vec![benefit(
                "A",
                "fbt-debt-waiver",
                waiver("10.00"),
                "type_2",
                false,
            )],
            year,
        )
        .unwrap();
        // The warning leads the list where it applies.
        result["warnings"][0]
            .as_str()
            .unwrap()
            .contains("FBT return 2026 instructions")
    };
    assert!(!warned(2026));
    assert!(warned(2027));
}

fn top(benefits: Value) -> Value {
    json!({"employer": "Employer", "year_ended": 2026, "scope_confirmed": true,
           "benefits": benefits})
}

fn one(fields: Value) -> Value {
    let mut b = benefit("A", "fbt-debt-waiver", waiver("100.00"), "type_2", false);
    for (k, v) in fields.as_object().unwrap() {
        if v.is_null() {
            b.as_object_mut().unwrap().remove(k);
        } else {
            b[k] = v.clone();
        }
    }
    top(json!([b]))
}

#[test]
fn refuses_a_benefit_whose_gst_type_is_not_stated() {
    for gst in [Value::Null, json!("type_3"), json!(1)] {
        let r = refused(one(json!({"gst_type": gst})));
        assert_eq!(r.code, "fbt_return.gst_type_missing");
        assert!(
            r.message.starts_with("benefits[0].gst_type is required"),
            "{}",
            r.message
        );
    }
}

#[test]
fn refuses_a_year_the_repository_has_not_loaded() {
    for year in [json!(2025), json!(2028), json!("2026"), Value::Null] {
        let mut input = one(json!({}));
        if year.is_null() {
            input.as_object_mut().unwrap().remove("year_ended");
            assert_eq!(refused(input).code, "input.missing_field");
            continue;
        }
        input["year_ended"] = year;
        let r = refused(input);
        assert_eq!(r.code, "period.unsupported");
        assert!(r.message.contains("2026 and 2027"), "{}", r.message);
    }
}

#[test]
fn refuses_mixed_years() {
    let mut input = waiver("100.00");
    input["year_ended"] = json!(2027);
    let r = refused(one(json!({"input": input})));
    assert_eq!(r.code, "fbt_return.mixed_years");
    // The return's own year inside a benefit is accepted.
    let mut same = waiver("100.00");
    same["year_ended"] = json!(2026);
    assert!(api::evaluate_coded("fbt-return", &one(json!({"input": same}))).is_ok());
}

#[test]
fn refuses_the_payroll_tax_gross_up() {
    let mut input = one(json!({}));
    input["payroll_tax_state"] = json!("NSW");
    let r = refused(input);
    assert_eq!(r.code, "fbt_return.payroll_tax_unsupported");
    assert!(
        r.message.contains("each state and territory"),
        "{}",
        r.message
    );
}

#[test]
fn refuses_negative_and_non_finite_amounts_as_the_worksheet_does() {
    for amount in ["-5.00", "NaN", "Infinity"] {
        let alone = api::evaluate_coded(
            "fbt-debt-waiver",
            &json!({"amount_waived": amount, "year_ended": 2026, "scope_confirmed": true}),
        )
        .unwrap_err();
        let r = refused(one(json!({"input": waiver(amount)})));
        assert_eq!(r.code, alone.code, "{amount}");
        assert_eq!(
            r.message,
            format!("benefits[0] fbt-debt-waiver: {}", alone.message)
        );
    }
}

#[test]
fn refuses_more_than_one_employer() {
    let mut input = one(json!({}));
    input["employer"] = json!(["Employer", "Other"]);
    assert_eq!(refused(input).code, "fbt_return.multiple_employers");
    let r = refused(one(json!({"employer": "Other"})));
    assert_eq!(r.code, "fbt_return.multiple_employers");
    assert!(api::evaluate_coded("fbt-return", &one(json!({"employer": "Employer"}))).is_ok());
}

#[test]
fn refuses_reporting_a_benefit_s_5e_3_excludes() {
    let parking = json!({"method": "commercial_parking_station",
                         "first_business_day_fee": "18.00", "benefits": 1,
                         "lowest_fee": "18.00", "employee_contribution": "0.00"});
    let meal = json!({"method": "50_50_split", "meal_entertainment_expenditure": "100.00"});
    for (command, input) in [
        ("fbt-car-parking", parking),
        ("fbt-meal-entertainment", meal),
    ] {
        let r = refused(one(
            json!({"command": command, "input": input, "excluded": false}),
        ));
        assert_eq!(r.code, "fbt_return.exclusion_conflict", "{command}");
    }
}

#[test]
fn refuses_an_operating_cost_car_whose_statutory_value_is_lower() {
    // The Library's car: operating cost $12,507.16 against the statutory $8,500.
    let input = json!({"running_costs": "5200.00", "lease_charges": "0.00",
                       "depreciated_value": "50000.00", "effective_life_years": "8",
                       "days_held": 365, "business_use": "0.3636",
                       "employee_contribution": "1500.00",
                       "statutory_formula_taxable_value": "8500.00",
                       "electric_car_exempt": false});
    let r = refused(one(
        json!({"command": "fbt-car-operating-cost", "input": input}),
    ));
    assert_eq!(r.code, "fbt_return.car_statutory_lower");
}

#[test]
fn refuses_what_it_does_not_take() {
    let cases = [
        (
            one(json!({"command": "fbt"})),
            "fbt_return.benefit_command_unsupported",
        ),
        (
            one(json!({"command": Value::Null})),
            "fbt_return.benefit_command_unsupported",
        ),
        (one(json!({"employee": Value::Null})), "input.missing_field"),
        (one(json!({"employee": ""})), "input.missing_field"),
        (one(json!({"excluded": Value::Null})), "input.missing_field"),
        (one(json!({"excluded": "no"})), "input.flag_invalid"),
        (one(json!({"input": Value::Null})), "input.missing_field"),
        (one(json!({"shares": 2})), "input.field_unknown"),
        (top(json!([])), "input.missing_field"),
        (top(json!(["benefit"])), "input.not_object"),
        (json!({"not_an_input": true}), "input.field_unknown"),
    ];
    for (input, code) in cases {
        assert_eq!(refused(input.clone()).code, code, "{input}");
    }
    let mut scoped = waiver("100.00");
    scoped["scope_confirmed"] = json!(true);
    assert_eq!(
        refused(one(json!({"input": scoped}))).code,
        "input.field_unknown"
    );
    let mut unconfirmed = one(json!({}));
    unconfirmed["scope_confirmed"] = json!(false);
    let r = refused(unconfirmed);
    assert_eq!(r.code, "scope.not_confirmed");
    assert!(
        r.message.starts_with("benefits[0] fbt-debt-waiver: "),
        "{}",
        r.message
    );
}

/// SplitMix64, as tests/properties.rs uses.
struct Rng(u64);

impl Rng {
    fn int(&mut self, low: u64, high: u64) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        low + (z ^ (z >> 31)) % (high - low + 1)
    }
}

fn d(value: &Value) -> Decimal {
    Decimal::parse(value.as_str().unwrap()).unwrap()
}

#[test]
fn the_composed_totals_are_the_sum_of_the_benefits() {
    // 200 seeded returns of up to 8 debt waivers across 3 employees: each
    // aggregate is the sum of its type's waivers, the reconciliation balances,
    // and each employee's reportable amount is the s 135P one: the cents over
    // 53, the cents of a dollar dropped.
    let mut rng = Rng(20261116);
    for _ in 0..200 {
        let count = rng.int(1, 8);
        let mut benefits = Vec::new();
        let mut type_one = 0u64;
        let mut type_two = 0u64;
        let mut individual = [0u64; 3];
        for _ in 0..count {
            let cents = rng.int(0, 400_000);
            let employee = rng.int(0, 2) as usize;
            let one = rng.int(0, 1) == 1;
            let excluded = rng.int(0, 3) == 0;
            *if one { &mut type_one } else { &mut type_two } += cents;
            if !excluded {
                individual[employee] += cents;
            }
            let amount = format!("{}.{:02}", cents / 100, cents % 100);
            benefits.push(benefit(
                &format!("E{employee}"),
                "fbt-debt-waiver",
                waiver(&amount),
                if one { "type_1" } else { "type_2" },
                excluded,
            ));
        }
        let result = compose(benefits).unwrap();
        let money = |c: u64| format!("{}.{:02}", c / 100, c % 100);
        assert_eq!(result["amounts"]["type_one_value"], money(type_one));
        assert_eq!(result["amounts"]["type_two_value"], money(type_two));
        assert_eq!(result["reconciliation"]["difference"], "0.00");
        for (label, amounts) in result["employees"].as_object().unwrap() {
            let at: usize = label[1..].parse().unwrap();
            let expected = if individual[at] > 200_000 {
                Decimal::from_int((individual[at] / 53) as i64)
            } else {
                Decimal::from_int(0)
            };
            assert!(
                d(&amounts["reportable_fringe_benefits_amount"])
                    .equals(&expected)
                    .unwrap(),
                "{label} {amounts}"
            );
        }
    }
}

#[test]
fn leaves_the_number_of_cars_loans_and_units_to_the_preparer() {
    // One car shared by 2 employees is given as 2 benefits: the benefits do
    // not show that they are one car, so 23A's Number is not given.
    let result = compose(vec![
        benefit(
            "A",
            "fbt-car-statutory",
            car("20000.00", false),
            "type_1",
            false,
        ),
        benefit(
            "B",
            "fbt-car-statutory",
            car("20000.00", false),
            "type_1",
            false,
        ),
    ])
    .unwrap();
    assert!(result["return_labels"]["23A"].get("number").is_none());
    assert!(
        result["left_to_preparer"]
            .get("23A, 23B, 23C and 23F Number")
            .is_some()
    );
}
