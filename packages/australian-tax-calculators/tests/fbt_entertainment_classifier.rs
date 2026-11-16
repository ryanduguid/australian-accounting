// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The entertainment classifier against TR 97/17's examples and TR 2007/12's
//! minor benefits view, cited by paragraph as the Commissioner's view
//! (docs/calculation-evidence.md#fbt-entertainment-classifier), and its
//! refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "fbt-entertainment-classifier";

/// The input for `lines` in the year ended 31 March 2027, with `overrides`;
/// a null override removes the input.
fn input(lines: Value, overrides: Value) -> Value {
    let mut input = json!({
        "lines": lines, "income_tax_exempt": false, "method": "actual",
        "year_ended": 2027, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            input.as_object_mut().unwrap().remove(k);
        } else {
            input[k] = v.clone();
        }
    }
    input
}

fn classify(lines: Value, overrides: Value) -> Result<Value, String> {
    api::evaluate(COMMAND, &input(lines, overrides))
}

/// One line: food or drink at a social function, provided by the employer,
/// with `overrides`.
fn line(overrides: Value) -> Value {
    let mut line = json!({
        "cost": "1100.00", "gst_status": "includes_gst", "provided": "food_or_drink",
        "occasion": "social_function", "venue": "elsewhere", "paid_by": "employer",
        "attendees": {"employees": 10},
    });
    for (k, v) in overrides.as_object().unwrap() {
        line[k] = v.clone();
    }
    line
}

/// TR 97/17 example 1.1 (para 49): an end of financial year function on the
/// employer's premises for employees, their associates and clients.
fn example_1_1() -> Value {
    line(json!({
        "venue": "business_premises_working_day",
        "attendees": {"employees": 6, "associates": 2, "others": 2},
    }))
}

/// Each class's (fringe benefit, deductible, GST credit) on line 1.
fn outcomes(result: &Value) -> Vec<(String, String, String, String)> {
    result["lines"][0]["classes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let text = |k: &str| c[k].as_str().unwrap_or("null").to_string();
            (
                text("attendees"),
                text("fringe_benefit"),
                text("deductible"),
                text("gst_credit"),
            )
        })
        .collect()
}

fn row(class: &str, fbt: &str, deductible: &str, credit: &str) -> (String, String, String, String) {
    (class.into(), fbt.into(), deductible.into(), credit.into())
}

#[test]
fn example_1_1_splits_the_function_by_class() {
    let result = classify(json!([example_1_1()]), json!({})).unwrap();
    assert_eq!(result["calculation"], "fbt_entertainment_classifier");
    assert_eq!(result["lines"][0]["entertainment"], "meal_entertainment");
    // Para 50: the employees' share is exempt under s 41 and not deductible;
    // para 51: the associates' is a property fringe benefit and deductible;
    // para 52: the clients' is neither.
    assert_eq!(
        outcomes(&result),
        [
            row("employees", "exempt", "0.00", "0.00"),
            row("associates", "arises", "200.00", "20.00"),
            row("others", "none", "0.00", "0.00"),
        ]
    );
    let classes = &result["lines"][0]["classes"];
    assert_eq!(classes[0]["exemption"], "FBTAA s 41");
    assert_eq!(classes[0]["cost"], "660.00");
    assert_eq!(classes[1]["benefit_type"], "property");
    assert_eq!(classes[2]["benefit_type"], Value::Null);
    assert_eq!(
        result["amounts"],
        json!({"cost": "1100.00", "deductible": "200.00", "gst_credit": "20.00",
            "minor_benefit_possible_cost": "220.00", "undetermined_cost": "0.00"})
    );
    assert_eq!(result["rates"], json!({"minor_benefit_threshold": "300"}));
    assert!(result.get("fbt_meal_entertainment_input").is_none());
}

#[test]
fn examples_1_3_and_2_1_turn_on_what_and_where() {
    // Example 1.3 (paras 55 and 56): sandwiches and juice for employees on
    // the premises: not entertainment, exempt under s 41, deductible. Not
    // meal entertainment, so no method is needed.
    let lines = json!([line(json!({
        "cost": "55.00", "occasion": "light_refreshments",
        "venue": "business_premises_working_day", "attendees": {"employees": 5},
    }))]);
    let result = classify(lines, json!({"method": null})).unwrap();
    assert_eq!(result["lines"][0]["entertainment"], "not_entertainment");
    assert_eq!(
        outcomes(&result),
        [row("employees", "exempt", "50.00", "5.00")]
    );
    // Example 2.1 (paras 61 and 62): a social function at a restaurant: a
    // property fringe benefit, deductible under s 32-20.
    let result = classify(json!([line(json!({}))]), json!({})).unwrap();
    assert_eq!(
        outcomes(&result),
        [row("employees", "arises", "1000.00", "100.00")]
    );
    assert_eq!(result["lines"][0]["classes"][0]["benefit_type"], "property");
}

#[test]
fn travel_meals_follow_examples_3_1_6_2_and_questions_9_and_10() {
    // Example 3.1 (paras 66 and 67): a travelling employee's reimbursed meal
    // with wine is not meal entertainment: an expense payment benefit,
    // deductible.
    let travel = |cost: &str, paid_by: &str, attendees: Value| {
        line(
            json!({"cost": cost, "gst_status": "no_gst", "occasion": "travel_meal",
            "paid_by": paid_by, "attendees": attendees}),
        )
    };
    let lines = json!([travel("66.00", "reimbursed", json!({"employees": 1}))]);
    let result = classify(lines, json!({})).unwrap();
    assert_eq!(result["lines"][0]["entertainment"], "not_entertainment");
    assert_eq!(
        outcomes(&result),
        [row("employees", "arises", "66.00", "0.00")]
    );
    assert_eq!(
        result["lines"][0]["classes"][0]["benefit_type"],
        "expense_payment"
    );
    // Example 6.2 (para 84): employee and client both travelling, the
    // employer paying both: neither is entertainment, both deductible.
    let lines = json!([travel(
        "120.00",
        "employer",
        json!({"employees": 1, "others": 1})
    )]);
    let result = classify(lines, json!({})).unwrap();
    assert_eq!(
        outcomes(&result),
        [
            row("employees", "arises", "60.00", "0.00"),
            row("others", "none", "60.00", "0.00"),
        ]
    );
    // Questions 9 and 10 (paras 89 to 96): the travelling employee's meal is
    // still not entertainment; the non-travelling employees' is, and is
    // deductible as a fringe benefit (para 93); the non-travelling client's
    // is not deductible (para 96). Each goes on its own line.
    let lines = json!([
        travel("50.00", "employer", json!({"employees": 1})),
        line(
            json!({"cost": "100.00", "gst_status": "no_gst", "occasion": "business_meal",
            "attendees": {"employees": 2}})
        ),
        line(
            json!({"cost": "50.00", "gst_status": "no_gst", "occasion": "business_meal",
            "attendees": {"others": 1}})
        ),
    ]);
    let result = classify(lines, json!({})).unwrap();
    let deductible: Vec<_> = (0..3)
        .map(|i| result["lines"][i]["classes"][0]["deductible"].clone())
        .collect();
    assert_eq!(deductible, [json!("50.00"), json!("100.00"), json!("0.00")]);
    assert_eq!(result["amounts"]["deductible"], "150.00");
}

#[test]
fn reimbursed_and_card_paid_meals_follow_examples_11_1_and_16_1() {
    // Example 11.1 (paras 100 and 101): an employee reimbursed $200 for a
    // business lunch with another employee: deductible $200.
    let reimbursed = line(json!({"cost": "200.00", "gst_status": "no_gst",
        "occasion": "business_meal", "paid_by": "reimbursed", "attendees": {"employees": 2}}));
    let result = classify(json!([reimbursed]), json!({})).unwrap();
    assert_eq!(
        outcomes(&result),
        [row("employees", "arises", "200.00", "0.00")]
    );
    assert_eq!(
        result["lines"][0]["classes"][0]["benefit_type"],
        "expense_payment"
    );
    // Example 16.1 (para 130): employees dine together on the corporate card:
    // property fringe benefits, deductible.
    let card = line(json!({"cost": "440.00", "occasion": "business_meal",
        "attendees": {"employees": 4}}));
    let result = classify(json!([card]), json!({})).unwrap();
    assert_eq!(
        outcomes(&result),
        [row("employees", "arises", "400.00", "40.00")]
    );
    assert_eq!(result["lines"][0]["classes"][0]["benefit_type"], "property");
}

#[test]
fn discounted_bills_split_per_head_as_examples_17_1_and_17_2() {
    // Example 17.1 (para 134): a $50 discounted bill for an employee and a
    // client: $25 is the employee's.
    let meal = |cost: &str, others: i64| {
        line(
            json!({"cost": cost, "gst_status": "no_gst", "occasion": "business_meal",
            "paid_by": "reimbursed", "attendees": {"employees": 1, "others": others}}),
        )
    };
    let result = classify(json!([meal("50.00", 1)]), json!({})).unwrap();
    assert_eq!(
        outcomes(&result),
        [
            row("employees", "arises", "25.00", "0.00"),
            row("others", "none", "0.00", "0.00"),
        ]
    );
    // Example 17.2 (para 135): $120 discounted, 3 clients: $30 per head.
    let result = classify(json!([meal("120.00", 3)]), json!({})).unwrap();
    let classes = &result["lines"][0]["classes"];
    assert_eq!(classes[0]["cost"], "30.00");
    assert_eq!(classes[1]["cost"], "90.00");
    // A cost that does not divide evenly: the last class takes the
    // remainder, so the parts add up to the cost.
    let result = classify(json!([meal("100.00", 2)]), json!({})).unwrap();
    let classes = &result["lines"][0]["classes"];
    assert_eq!(classes[0]["cost"], "33.33");
    assert_eq!(classes[1]["cost"], "66.67");
}

#[test]
fn a_travelling_employees_spouse_follows_paras_136_to_138_1() {
    // The employee's meal on work travel is not entertainment (para 138); the
    // spouse's is meal entertainment and a fringe benefit (para 138.1).
    let lines = json!([
        line(
            json!({"cost": "80.00", "gst_status": "no_gst", "occasion": "travel_meal",
            "paid_by": "reimbursed", "attendees": {"employees": 1}})
        ),
        line(
            json!({"cost": "80.00", "gst_status": "no_gst", "occasion": "business_meal",
            "paid_by": "reimbursed", "attendees": {"associates": 1}})
        ),
    ]);
    let result = classify(lines, json!({})).unwrap();
    assert_eq!(result["lines"][1]["entertainment"], "meal_entertainment");
    let spouse = &result["lines"][1]["classes"][0];
    assert_eq!(spouse["fringe_benefit"], "arises");
    assert_eq!(spouse["benefit_type"], "expense_payment");
    assert_eq!(result["amounts"]["deductible"], "160.00");
}

#[test]
fn a_division_9a_election_displaces_s_41_and_minor_benefits() {
    // TR 97/17 para 44: s 41 does not apply under an election; TR 2007/12 para
    // 14: nor do minor benefits under the 50/50 split method. ITAA 1936
    // s 51AEA and GST Act s 69-5(3A)(a): half deductible and creditable.
    let result = classify(json!([example_1_1()]), json!({"method": "50_50_split"})).unwrap();
    assert_eq!(
        outcomes(&result),
        [
            row("employees", "in_method_total", "300.00", "30.00"),
            row("associates", "in_method_total", "100.00", "10.00"),
            row("others", "in_method_total", "100.00", "10.00"),
        ]
    );
    let classes = &result["lines"][0]["classes"];
    assert_eq!(classes[0]["benefit_type"], "meal_entertainment");
    assert_eq!(classes[2]["benefit_type"], Value::Null);
    assert!(classes[1].get("minor_benefit").is_none());
    assert!(classes[0].get("exemption").is_none());
    assert_eq!(result["amounts"]["deductible"], "500.00");
    assert_eq!(
        result["amounts"]["meal_entertainment_expenditure"],
        "1100.00"
    );
    assert_eq!(
        result["fbt_meal_entertainment_input"],
        json!({"method": "50_50_split", "meal_entertainment_expenditure": "1100.00"})
    );
    // The total feeds fbt-meal-entertainment, which values it.
    let valued = api::evaluate(
        "fbt-meal-entertainment",
        &json!({"method": "50_50_split", "meal_entertainment_expenditure": "1100.00",
            "year_ended": 2027, "scope_confirmed": true}),
    );
    assert!(valued.is_ok(), "{valued:?}");
    // ITAA 1936 s 51AEB and s 69-5(3A)(b): the register percentage.
    let result = classify(
        json!([example_1_1()]),
        json!({"method": "12_week_register", "income_tax_register_percentage": "40"}),
    )
    .unwrap();
    assert_eq!(
        outcomes(&result),
        [
            row("employees", "in_method_total", "240.00", "24.00"),
            row("associates", "in_method_total", "80.00", "8.00"),
            row("others", "in_method_total", "80.00", "8.00"),
        ]
    );
    assert_eq!(
        result["fbt_meal_entertainment_input"]["method"],
        "12_week_register"
    );
}

#[test]
fn recreation_is_never_meal_entertainment() {
    let theatre = |venue: &str, attendees: Value| {
        json!({"cost": "330.00", "gst_status": "includes_gst", "provided": "recreation",
            "venue": venue, "paid_by": "employer", "attendees": attendees})
    };
    // Division 9A reaches only meal entertainment (FBTAA s 37AD), so an
    // election leaves recreation a fringe benefit, and no method is needed.
    let result = classify(
        json!([theatre("elsewhere", json!({"employees": 3}))]),
        json!({"method": "50_50_split"}),
    )
    .unwrap();
    assert_eq!(result["lines"][0]["entertainment"], "recreation");
    assert_eq!(
        outcomes(&result),
        [row("employees", "arises", "300.00", "30.00")]
    );
    assert_eq!(
        result["lines"][0]["classes"][0]["benefit_type"],
        "property_or_residual"
    );
    assert_eq!(result["amounts"]["meal_entertainment_expenditure"], "0.00");
    let result = classify(
        json!([theatre("elsewhere", json!({"others": 3}))]),
        json!({"method": null}),
    )
    .unwrap();
    assert_eq!(outcomes(&result), [row("others", "none", "0.00", "0.00")]);
    // On the premises on a working day, s 41 turns on whether it is property
    // consumed there: undetermined, out of the totals.
    let result = classify(
        json!([theatre(
            "business_premises_working_day",
            json!({"employees": 3})
        )]),
        json!({}),
    )
    .unwrap();
    let class = &result["lines"][0]["classes"][0];
    assert_eq!(class["fringe_benefit"], "undetermined");
    assert_eq!(class["deductible"], Value::Null);
    assert!(class["missing"].as_str().unwrap().contains("41(1)"));
    assert_eq!(result["amounts"]["undetermined_cost"], "330.00");
    assert_eq!(result["amounts"]["cost"], "0.00");
}

#[test]
fn food_or_drink_without_an_occasion_is_undetermined() {
    let mut meal = line(json!({}));
    meal.as_object_mut().unwrap().remove("occasion");
    // Undetermined, so not meal entertainment: no method needed.
    let result = classify(json!([meal]), json!({"method": null})).unwrap();
    let out = &result["lines"][0];
    assert_eq!(out["entertainment"], "undetermined");
    assert!(out["missing"].as_str().unwrap().starts_with("occasion:"));
    assert!(out.get("classes").is_none());
    assert_eq!(result["amounts"]["undetermined_cost"], "1100.00");
}

#[test]
fn minor_benefits_are_only_ever_possible() {
    let meal = |cost: &str, extra: Value| {
        let mut line = line(json!({"cost": cost, "gst_status": "no_gst",
            "occasion": "business_meal", "attendees": {"employees": 1}}));
        for (k, v) in extra.as_object().unwrap() {
            line[k] = v.clone();
        }
        let result = classify(json!([line]), json!({})).unwrap();
        result["lines"][0]["classes"][0]["minor_benefit"].clone()
    };
    // s 58P(1)(e): less than $300.
    assert_eq!(meal("300.00", json!({}))["status"], "unavailable");
    let minor = meal("299.99", json!({"infrequent_and_irregular": true}));
    assert_eq!(minor["status"], "possible");
    assert_eq!(minor["per_head"], "299.99");
    assert!(
        minor["reason"]
            .as_str()
            .unwrap()
            .contains("employer's judgement")
    );
    // TR 2007/12 example 2 (paras 31 to 41): an annual party, regular but
    // infrequent, can still be exempt; the stated fact does not decide it.
    let minor = meal("100.00", json!({"infrequent_and_irregular": false}));
    assert_eq!(minor["status"], "possible");
    assert_eq!(minor["infrequent_and_irregular"], false);
    // A single class of unknown size: each person's value is unknown.
    let minor = meal("100.00", json!({"attendees": {"employees": null}}));
    assert_eq!(minor["status"], "not_assessed");
}

#[test]
fn refusals_name_the_fact_to_establish() {
    let coded = |lines: Value, overrides: Value| {
        api::evaluate_coded(COMMAND, &input(lines, overrides))
            .unwrap_err()
            .code
    };
    let one = |overrides: Value| json!([line(overrides)]);
    let none = json!({});
    let cases = [
        (
            coded(one(none.clone()), json!({"income_tax_exempt": true})),
            "fbt_entertainment_classifier.tax_exempt_body",
        ),
        (
            coded(
                one(json!({"attendees": {"employees": null, "others": 2}})),
                none.clone(),
            ),
            "fbt_entertainment_classifier.line_not_split",
        ),
        (
            coded(one(json!({"gst_status": "unknown"})), none.clone()),
            "fbt_entertainment_classifier.gst_status_invalid",
        ),
        (
            coded(one(json!({"gst_status": null})), none.clone()),
            "fbt_entertainment_classifier.gst_status_invalid",
        ),
        (
            coded(one(none.clone()), json!({"method": null})),
            "fbt_entertainment_classifier.method_required",
        ),
        (
            coded(one(none.clone()), json!({"method": "12_week_register"})),
            "fbt_entertainment_classifier.register_percentage_invalid",
        ),
        (
            coded(
                one(none.clone()),
                json!({"method": "12_week_register", "income_tax_register_percentage": "100.01"}),
            ),
            "fbt_entertainment_classifier.register_percentage_invalid",
        ),
        (
            coded(
                one(none.clone()),
                json!({"income_tax_register_percentage": "40"}),
            ),
            "input.field_unknown",
        ),
        (
            coded(json!([]), none.clone()),
            "fbt_entertainment_classifier.lines_invalid",
        ),
        (
            coded(json!("x"), none.clone()),
            "fbt_entertainment_classifier.lines_invalid",
        ),
        (
            coded(json!([1]), none.clone()),
            "fbt_entertainment_classifier.lines_invalid",
        ),
        (
            coded(one(json!({"provided": "gift"})), none.clone()),
            "fbt_entertainment_classifier.fact_invalid",
        ),
        (
            coded(one(json!({"occasion": "party"})), none.clone()),
            "fbt_entertainment_classifier.fact_invalid",
        ),
        (
            coded(one(json!({"attendees": {}})), none.clone()),
            "fbt_entertainment_classifier.fact_invalid",
        ),
        (
            coded(one(json!({"attendees": {"staff": 1}})), none.clone()),
            "fbt_entertainment_classifier.fact_invalid",
        ),
        (
            coded(one(json!({"attendees": {"employees": -1}})), none.clone()),
            "fbt_entertainment_classifier.fact_invalid",
        ),
        (
            coded(one(json!({"attendees": {"employees": 1.5}})), none.clone()),
            "fbt_entertainment_classifier.fact_invalid",
        ),
        (
            coded(one(json!({"venue": null})), none.clone()),
            "fbt_entertainment_classifier.fact_invalid",
        ),
        (
            coded(one(json!({"note": "x"})), none.clone()),
            "input.field_unknown",
        ),
        (
            coded(one(json!({"provided": "recreation"})), none.clone()),
            "input.field_unknown",
        ),
        (
            coded(one(json!({"cost": "-5.00"})), none.clone()),
            "input.amount_invalid",
        ),
        (
            coded(one(json!({"cost": "NaN"})), none.clone()),
            "input.amount_invalid",
        ),
        (
            coded(one(json!({"cost": "Infinity"})), none.clone()),
            "input.amount_invalid",
        ),
        (
            coded(one(json!({"cost": "1.005"})), none.clone()),
            "input.amount_invalid",
        ),
        (
            coded(one(json!({"cost": 5})), none.clone()),
            "input.amount_invalid",
        ),
        (
            coded(
                one(json!({"infrequent_and_irregular": "yes"})),
                none.clone(),
            ),
            "input.flag_invalid",
        ),
        (
            coded(one(none.clone()), json!({"method": "per_head"})),
            "input.fbt_method_invalid",
        ),
        (
            coded(one(none.clone()), json!({"year_ended": 2028})),
            "period.unsupported",
        ),
    ];
    for (code, expected) in cases {
        assert_eq!(code, expected);
    }
    let mut meal = line(json!({}));
    meal.as_object_mut().unwrap().remove("venue");
    assert_eq!(
        classify(json!([meal]), json!({})).unwrap_err(),
        "line 1: venue is required"
    );
}

#[test]
fn the_catalogue_names_the_methods_and_its_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["fbt_entertainment_classifier"];
    assert_eq!(
        entry["methods"],
        json!(["actual", "50_50_split", "12_week_register"])
    );
    assert_eq!(entry["source_checked"], "2026-11-16");
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(result["amounts"]["deductible"], "200.00");
    assert_eq!(
        api::evaluate("periods", &json!({"kind": "fbt_entertainment_classifier"})).unwrap()[1]["end"],
        "2027-03-31"
    );
}
