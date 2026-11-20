// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The meal entertainment register check against FBTAA ss 37CA to 37CF and
//! the employer's guide, 14.8, read on 19 November 2026, and the Library's
//! ¶3-220 (docs/calculation-evidence.md#fbt-meal-entertainment-register):
//! the 12 week period, each entry's details, the years a register is valid
//! for, the 20% rule and the refusals, then the register's values in
//! fbt-meal-entertainment.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "fbt-meal-entertainment-register";

/// A client dinner off the premises and a staff party on them.
fn entries() -> Value {
    json!([
        {"date": "2025-05-14", "recipients": {"employees": 3, "associates": 0, "others": 2},
         "cost": "500.00", "kind": "client dinner", "location": "restaurant",
         "on_premises": false, "fringe_benefits_value": "300.00"},
        {"date": "2025-07-02", "recipients": {"employees": 10, "associates": 2, "others": 0},
         "cost": "1200.00", "kind": "staff party", "location": "the employer's office",
         "on_premises": true, "in_house_dining_facility": false,
         "fringe_benefits_value": "1200.00"},
    ])
}

/// A register kept from 1 May to 23 July 2025, 84 days in the FBT year ended
/// 31 March 2026, checked for that year.
fn input(overrides: Value) -> Value {
    let mut facts = json!({
        "register_start": "2025-05-01", "register_end": "2025-07-23",
        "entries": entries(), "year_ended": 2026, "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn run(overrides: Value) -> Value {
    api::evaluate(COMMAND, &input(overrides)).unwrap()
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded(COMMAND, &input(overrides))
        .unwrap_err()
        .code
}

/// The check named `id`: whether it passed, and its detail.
fn check(result: &Value, id: &str) -> (bool, String) {
    let found = result["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["check"] == id)
        .unwrap_or_else(|| panic!("no check {id}"));
    (
        found["passed"].as_bool().unwrap(),
        found["detail"].as_str().unwrap().to_string(),
    )
}

#[test]
fn a_complete_register_is_valid_and_gives_its_percentage() {
    let result = run(json!({}));
    assert_eq!(result["valid"], true);
    for id in [
        "period_length",
        "meal_entertainment_provided",
        "entry_details",
        "valid_years",
        "expenditure_increase",
    ] {
        assert!(check(&result, id).0, "{id}");
    }
    let amounts = &result["amounts"];
    assert_eq!(amounts["register_meal_entertainment_value"], "1700.00");
    assert_eq!(amounts["register_fringe_benefits_value"], "1500.00");
    // 1500 / 1700 x 100 = 88.2352...
    assert_eq!(amounts["register_percentage"], "88.24");
    assert_eq!(result["rates"]["register_days"], "84");
    assert_eq!(
        result["fbt_meal_entertainment_input"],
        json!({"method": "12_week_register", "register_fringe_benefits_value": "1500.00",
               "register_meal_entertainment_value": "1700.00", "register_first_year": 2026,
               "register_years_expenditure": []})
    );
}

/// The Library's ¶3-220: ABC's register put 40% of its meal entertainment
/// as fringe benefits, kept in the 2025-26 FBT year, so usable for it and
/// the following 4 FBT years (s 37CD), and $177,500 x 40% = $71,000. The
/// register's values here give 40%; fbt-meal-entertainment then gives
/// $71,000.00 for the year ended 31 March 2027 from them.
#[test]
fn the_librarys_register_feeds_fbt_meal_entertainment() {
    let register = json!([
        {"date": "2025-06-10", "recipients": {"employees": 2, "associates": 0, "others": 3},
         "cost": "1000.00", "kind": "corporate box", "location": "Brisbane Cricket Ground",
         "on_premises": false, "fringe_benefits_value": "400.00"},
    ]);
    let result = run(json!({"entries": register, "year_ended": 2027,
                            "register_years_expenditure": ["150000.00"]}));
    assert_eq!(result["valid"], true);
    assert_eq!(result["amounts"]["register_percentage"], "40.00");
    assert_eq!(result["rates"]["first_valid_year"], "2026");
    assert_eq!(result["rates"]["last_valid_year"], "2030");
    let mut facts = result["fbt_meal_entertainment_input"].clone();
    assert_eq!(facts["register_years_expenditure"], json!(["150000.00"]));
    facts["meal_entertainment_expenditure"] = json!("177500.00");
    facts["year_ended"] = json!(2027);
    facts["scope_confirmed"] = json!(true);
    let valued = api::evaluate("fbt-meal-entertainment", &facts).unwrap();
    assert_eq!(valued["amounts"]["taxable_value"], "71000.00");
}

/// s 37CC(1) and (3): 83 days is short of 12 weeks; both ends count.
#[test]
fn a_period_short_of_12_weeks_is_not_valid() {
    let result = run(json!({"register_end": "2025-07-22"}));
    assert_eq!(result["valid"], false);
    let (passed, detail) = check(&result, "period_length");
    assert!(!passed);
    assert!(detail.contains("83 days, 11 weeks and 6 days"), "{detail}");
    assert!(result.get("fbt_meal_entertainment_input").is_none());
}

/// s 37CD(1) and (2): a period within one FBT year makes the register valid
/// for it and the 4 after; a period spanning 2 only for the second and the 4
/// after it.
#[test]
fn the_years_a_register_is_valid_for() {
    let spanning = json!({"register_start": "2026-02-01", "register_end": "2026-04-30",
                          "entries": []});
    let early = run(spanning.clone());
    let (passed, detail) = check(&early, "valid_years");
    assert!(!passed);
    assert!(
        detail.contains("Not yet valid for the year ended 31 March 2026"),
        "{detail}"
    );
    assert_eq!(early["rates"]["first_valid_year"], "2027");
    let mut next = spanning;
    next["year_ended"] = json!(2027);
    assert!(check(&run(next), "valid_years").0);
    // Kept in the year ended 31 March 2022: valid to the year ended 2026.
    let old = json!([
        {"date": "2021-06-01", "recipients": {"employees": 1, "associates": 0, "others": 0},
         "cost": "100.00", "kind": "lunch", "location": "cafe", "on_premises": false,
         "fringe_benefits_value": "100.00"},
    ]);
    let base = json!({"register_start": "2021-05-01", "register_end": "2021-07-31",
                      "entries": old});
    let mut last = base.clone();
    last["register_years_expenditure"] = json!(["1.00", "1.00", "1.00", "1.00"]);
    assert_eq!(run(last)["valid"], true);
    let mut expired = base;
    expired["year_ended"] = json!(2027);
    let result = run(expired);
    let (passed, detail) = check(&result, "valid_years");
    assert!(!passed);
    assert!(detail.contains("No longer valid"), "{detail}");
    assert_eq!(result["rates"]["last_valid_year"], "2026");
}

/// s 37CD(3): a year's expenses more than 20% above the first valid year's
/// end the register's validity at the end of that year; exactly 20% does not.
#[test]
fn the_20_percent_rule() {
    let register = json!({"register_start": "2024-05-01", "register_end": "2024-07-31",
                          "entries": [{"date": "2024-06-03",
                                       "recipients": {"employees": 1, "associates": 1, "others": 0},
                                       "cost": "250.00", "kind": "dinner", "location": "restaurant",
                                       "on_premises": false, "fringe_benefits_value": "250.00"}],
                          "year_ended": 2027});
    let mut at = register.clone();
    at["register_years_expenditure"] = json!(["10000.00", "12000.00"]);
    assert_eq!(run(at)["valid"], true);
    let mut above = register;
    above["register_years_expenditure"] = json!(["10000.00", "12000.01"]);
    let result = run(above);
    assert_eq!(result["valid"], false);
    let (passed, detail) = check(&result, "expenditure_increase");
    assert!(!passed);
    assert!(detail.contains("year ended 31 March 2026"), "{detail}");
}

/// s 37CE(1): a detail left out or null fails the entry, and the check names
/// it; an entry with no cost is left out of the totals and counted.
#[test]
fn an_entry_missing_a_detail_fails_the_register() {
    let gaps = json!([
        {"date": "2025-05-14", "recipients": {"employees": 3, "associates": 0, "others": 2},
         "cost": "500.00", "kind": " ", "location": "restaurant",
         "on_premises": false, "fringe_benefits_value": "300.00"},
        {"date": "2025-07-02", "recipients": {"employees": 10, "associates": 2, "others": 0},
         "cost": null, "kind": "staff party", "location": "the employer's office",
         "on_premises": true},
        {"recipients": null, "cost": "80.00", "kind": "lunch", "location": "cafe",
         "on_premises": false, "fringe_benefits_value": "0.00"},
        {"date": "2025-07-03", "recipients": {"employees": 0, "associates": 0, "others": 0},
         "cost": "10.00", "kind": "coffee", "location": "cafe",
         "on_premises": false, "fringe_benefits_value": "0.00"},
    ]);
    let result = run(json!({"entries": gaps}));
    assert_eq!(result["valid"], false);
    let (passed, detail) = check(&result, "entry_details");
    assert!(!passed);
    for expected in [
        "entry 1: no record of the kind of meal entertainment (s 37CE(1)(d))",
        "entry 2: no record of the cost (s 37CE(1)(c))",
        "in-house dining facility",
        "entry 3: no record of the date (s 37CE(1)(a)), whether each recipient",
        "entry 4: no record of whether each recipient",
    ] {
        assert!(detail.contains(expected), "{expected}: {detail}");
    }
    assert_eq!(result["rates"]["entries_without_cost"], "1");
    assert_eq!(
        result["amounts"]["register_meal_entertainment_value"],
        "590.00"
    );
}

#[test]
fn a_register_with_no_entries_is_not_valid() {
    let result = run(json!({"entries": []}));
    assert!(!check(&result, "meal_entertainment_provided").0);
    assert!(result["amounts"].get("register_percentage").is_none());
    assert_eq!(result["valid"], false);
}

#[test]
fn a_register_of_nil_costs_cannot_supply_a_register_percentage() {
    let mut entry = entries()[0].clone();
    entry["cost"] = json!("0.00");
    entry["fringe_benefits_value"] = json!("0.00");
    let result = run(json!({"entries": [entry]}));
    assert_eq!(result["valid"], false);
    assert!(!check(&result, "positive_total_cost").0);
    assert!(result["amounts"].get("register_percentage").is_none());
    assert!(result.get("fbt_meal_entertainment_input").is_none());
}

#[test]
fn it_refuses_what_it_cannot_read() {
    let entry = |overrides: Value| {
        let mut one = entries()[0].clone();
        for (k, v) in overrides.as_object().unwrap() {
            one[k] = v.clone();
        }
        json!({"entries": [one]})
    };
    let invalid = "fbt_meal_entertainment_register.entry_invalid";
    for (overrides, expected) in [
        (entry(json!({"date": "2025-07-24"})), invalid),
        (entry(json!({"date": "14/05/2025"})), invalid),
        (entry(json!({"fringe_benefits_value": "500.01"})), invalid),
        (
            entry(json!({"recipients": {"employees": 0, "associates": 0, "others": 5}})),
            invalid,
        ),
        (entry(json!({"recipients": {"employees": 1}})), invalid),
        (entry(json!({"in_house_dining_facility": true})), invalid),
        (entry(json!({"on_premises": "no"})), invalid),
        (entry(json!({"cost": "1.001"})), invalid),
        (entry(json!({"cost": null})), invalid),
        (entry(json!({"fringe_benefits_value": null})), invalid),
        (entry(json!({"attendees": 5})), "input.field_unknown"),
        (
            json!({"entries": {"date": "2025-05-14"}}),
            "fbt_meal_entertainment_register.entries_invalid",
        ),
        (json!({"entries": [1]}), invalid),
        (
            json!({"register_end": "2025-04-30"}),
            "fbt_meal_entertainment_register.period_invalid",
        ),
        (
            json!({"register_start": "2024-03-01", "register_end": "2025-07-23"}),
            "fbt_meal_entertainment_register.period_invalid",
        ),
        (
            json!({"register_start": "1 May 2025"}),
            "input.date_invalid",
        ),
        (
            json!({"register_years_expenditure": []}),
            "input.field_unknown",
        ),
        (json!({"year_ended": 2027}), "input.missing_field"),
        (
            json!({"year_ended": 2027, "register_years_expenditure": ["1.00", "2.00"]}),
            "fbt_meal_entertainment.register_years_invalid",
        ),
        (json!({"year_ended": 2025}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"register_kept": true}), "input.field_unknown"),
    ] {
        assert_eq!(code(overrides.clone()), expected, "{overrides}");
    }
}
