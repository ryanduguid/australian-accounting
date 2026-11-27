// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `age-pension` under Social Security Act 1991 s 1064(4) to (7), Pension
//! Rate Calculator A step 12, Modules BA4, C2 and H (compilation 234). Each
//! expected amount is worked by hand from the Act and the Social Security
//! Guide's amounts from 20 September 2026: MBR $29,520.40 single and
//! $22,253.40 partnered, pension supplement $2,293.20 and $1,729.00, energy
//! supplement $366.60 and $275.60, minimum pension supplement amount
//! $1,235.00 and $930.80 (5.1.9.10), remote area allowance $473.20 and $405.60
//! (Table H) and the $3,247.40 limit, each a year and divided by 26.

use australian_tax_calculators::api;
use serde_json::{Value, json};

const CIRCUMSTANCES: [&str; 6] = [
    "underage_de_facto",
    "advance_deduction",
    "armed_services_payment",
    "remote_area",
    "pension_supplement_election",
    "quarterly_energy_supplement",
];

/// The input before the circumstances: what `age-pension` took until they
/// were added.
fn before(date: &str, year: &str, relationship: &str, homeowner: bool, means: [&str; 3]) -> Value {
    let mut facts = json!({"in_australia": true, "not_blind": true, "not_transitional": true,
                           "no_rent_assistance": true});
    if relationship == "couple" {
        facts["partner_pensioner"] = json!(true);
    }
    json!({"date": date, "relationship": relationship, "homeowner": homeowner,
           "ordinary_income": means[0], "financial_assets": means[1], "assets": means[2],
           "facts": facts, "year": year, "scope_confirmed": true})
}

/// `base` with every circumstance stated false, then `stated` over them.
fn stating(mut base: Value, stated: Value) -> Value {
    for key in CIRCUMSTANCES {
        base["facts"][key] = json!(false);
    }
    for (key, value) in stated.as_object().unwrap() {
        base["facts"][key] = value.clone();
    }
    base
}

/// A single homeowner or a couple on 1 October 2026 with `income` a
/// fortnight and no assets, the circumstances `stated`.
fn case(relationship: &str, income: &str, stated: Value) -> Value {
    let base = before(
        "2026-10-01",
        "2026-27",
        relationship,
        true,
        [income, "0", "0"],
    );
    stating(base, stated)
}

fn amounts(input: &Value) -> Value {
    api::evaluate("age-pension", input).unwrap_or_else(|e| panic!("{e:?}"))["amounts"].clone()
}

fn rate(input: &Value) -> String {
    amounts(input)["rate"].as_str().unwrap().to_owned()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("age-pension", input).unwrap_err().code
}

#[test]
fn the_armed_services_limit_caps_the_rate() {
    let limited = json!({"armed_services_payment": true, "rate_before_november_1986": false});
    // Full rate (29,520.40 + 2,293.20 + 366.60) / 26 = 1,237.70 is above the
    // limit of 3,247.40 / 26 = 124.90 (s 1064(5)(d), (6) and (7)).
    let out = amounts(&case("single", "0", limited.clone()));
    assert_eq!(out["provisional_rate"], "1237.70");
    assert_eq!(out["rate"], "124.90");
    assert_eq!(rate(&case("couple", "0", limited.clone())), "124.90");
    // Income of 2,501.40: (2,501.40 - 5,876 / 26) x 0.5 = 1,137.70 off
    // 1,237.70 leaves 100.00, under the limit.
    assert_eq!(rate(&case("single", "2501.40", limited.clone())), "100.00");
    // At the limit and a cent over it: 1,112.80 off leaves 124.90 exactly;
    // 1,112.75 off leaves 124.95, held to 124.90.
    assert_eq!(rate(&case("single", "2451.60", limited.clone())), "124.90");
    assert_eq!(rate(&case("single", "2451.50", limited)), "124.90");
}

#[test]
fn the_remote_area_allowance_is_added_after_the_provisional_rate() {
    let remote = json!({"remote_area": true, "remote_area_child": false});
    // 473.20 / 26 = 18.20 single; 405.60 / 26 = 15.60 each of a couple.
    let out = amounts(&case("single", "0", remote.clone()));
    assert_eq!(out["remote_area_allowance"], "18.20");
    assert_eq!(out["rate"], "1255.90");
    assert_eq!(rate(&case("couple", "0", remote.clone())), "948.60");
    // A part rate of 37.70 (income 2,626: 1,200 off 1,237.70) takes it too.
    assert_eq!(rate(&case("single", "2626.00", remote.clone())), "55.90");
    // Income of 2,701.40 takes the rate to nil, and H1(aa) then adds none.
    let nil = amounts(&case("single", "2701.40", remote.clone()));
    assert_eq!(nil["remote_area_allowance"], "0.00");
    assert_eq!(nil["rate"], "0.00");
    // With the limit: 110.00 + 18.20 = 128.20 is held to 124.90.
    let both = json!({"remote_area": true, "remote_area_child": false,
                      "armed_services_payment": true, "rate_before_november_1986": false});
    assert_eq!(rate(&case("single", "2481.40", both)), "124.90");
}

#[test]
fn an_election_takes_the_minimum_pension_supplement_amount_out() {
    let elected = json!({"pension_supplement_election": true});
    // (29,520.40 + 2,293.20 - 1,235.00 + 366.60) / 26 = 1,190.20.
    let out = amounts(&case("single", "0", elected.clone()));
    assert_eq!(out["max_rate"], "1190.20");
    assert_eq!(out["rate"], "1190.20");
    // (22,253.40 + 1,729.00 - 930.80 + 275.60) / 26 = 897.20.
    assert_eq!(rate(&case("couple", "0", elected)), "897.20");
    // H1(aa)(ii): income of 2,626 takes 1,200, leaving nil only because of
    // the election (1,237.70 without it), so the allowance is still added.
    let remote = json!({"pension_supplement_election": true, "remote_area": true,
                        "remote_area_child": false});
    let out = amounts(&case("single", "2626.00", remote));
    assert_eq!(out["provisional_rate"], "0.00");
    assert_eq!(out["rate"], "18.20");
    // Without the remote area the election's rate is nil from 1,190.20 off
    // (income 2,606.40), and 10 cents less income leaves 0.05.
    let elected = json!({"pension_supplement_election": true});
    assert_eq!(rate(&case("single", "2606.40", elected.clone())), "0.00");
    assert_eq!(rate(&case("single", "2606.30", elected)), "0.05");
}

#[test]
fn quarterly_energy_supplement_leaves_module_c_out() {
    let quarterly = json!({"quarterly_energy_supplement": true});
    // (29,520.40 + 2,293.20) / 26 = 1,223.60; (22,253.40 + 1,729.00) / 26 = 922.40.
    assert_eq!(rate(&case("single", "0", quarterly.clone())), "1223.60");
    assert_eq!(rate(&case("couple", "0", quarterly)), "922.40");
    let both = json!({"quarterly_energy_supplement": true, "pension_supplement_election": true});
    // (29,520.40 + 2,293.20 - 1,235.00) / 26 = 1,176.10.
    assert_eq!(rate(&case("single", "0", both)), "1176.10");
}

#[test]
fn what_it_does_not_work_is_refused() {
    let refused = |stated: Value| code(&case("single", "0", stated));
    for stated in [
        json!({"underage_de_facto": true}),
        json!({"advance_deduction": true}),
        json!({"armed_services_payment": true, "rate_before_november_1986": true}),
        json!({"remote_area": true, "remote_area_child": true}),
        json!({"pension_supplement_election": true, "armed_services_payment": true,
               "rate_before_november_1986": false}),
    ] {
        assert_eq!(
            refused(stated.clone()),
            "age_pension.case_not_built",
            "{stated}"
        );
    }
    // The minimum pension supplement amount was read only from 20 September 2026.
    let july = stating(
        before("2026-07-01", "2026-27", "single", true, ["0", "0", "0"]),
        json!({"pension_supplement_election": true}),
    );
    assert_eq!(code(&july), "age_pension.case_not_built");
    // A call that does not state the circumstances, or a needed one.
    let old = before("2026-10-01", "2026-27", "single", true, ["0", "0", "0"]);
    assert_eq!(code(&old), "age_pension.facts_not_stated");
    let mut unstated = case("single", "0", json!({"armed_services_payment": true}));
    assert_eq!(code(&unstated), "age_pension.facts_not_stated");
    unstated["facts"]["armed_services_payment"] = json!(false);
    unstated["facts"]
        .as_object_mut()
        .unwrap()
        .remove("remote_area");
    assert_eq!(code(&unstated), "age_pension.facts_not_stated");
    assert_eq!(refused(json!({"remote_area": "yes"})), "input.flag_invalid");
}

/// A child circumstance is read only under its parent; stated without it, it
/// is refused, true or false.
#[test]
fn a_child_circumstance_without_its_parent_is_refused() {
    for (stated, key, parent) in [
        (
            json!({"rate_before_november_1986": true}),
            "rate_before_november_1986",
            "armed_services_payment",
        ),
        (
            json!({"remote_area_child": false}),
            "remote_area_child",
            "remote_area",
        ),
    ] {
        let err =
            api::evaluate_coded("age-pension", &case("single", "0", stated.clone())).unwrap_err();
        assert_eq!(err.code, "input.field_unknown", "{stated}");
        assert_eq!(
            err.message,
            format!("facts: {key} is an input only where {parent} is true.")
        );
    }
}

/// The inputs whose results before the circumstances are kept in
/// `tests/data/age_pension_golden.jsonl`.
fn golden_inputs() -> Vec<Value> {
    let means = [
        ["0", "0", "0"],
        ["150.00", "120000.00", "400000.00"],
        ["900.00", "300000.00", "900000.00"],
        ["2626.00", "0", "0"],
        ["40.00", "80000.00", "520000.00"],
        ["0", "600000.00", "1200000.00"],
    ];
    let mut inputs = Vec::new();
    for (date, year) in [
        ("2025-09-20", "2025-26"),
        ("2026-03-20", "2025-26"),
        ("2026-07-01", "2026-27"),
        ("2026-10-01", "2026-27"),
    ] {
        for relationship in ["single", "couple"] {
            for homeowner in [true, false] {
                for m in means {
                    inputs.push(before(date, year, relationship, homeowner, m));
                }
            }
        }
    }
    inputs
}

/// The fields that describe the command rather than the result.
const DESCRIBED: [&str; 4] = ["engine_version", "scope", "sources", "source_checked"];

/// A result without the fields of [`DESCRIBED`].
fn result(input: &Value) -> Value {
    let mut out = api::evaluate("age-pension", input).unwrap();
    for described in DESCRIBED {
        assert!(out.as_object_mut().unwrap().remove(described).is_some());
    }
    out
}

/// Every result for an input outside the new rules is what it was before
/// them. `AA_GOLDEN_WRITE=<path>` writes the file from the old input shape.
#[test]
fn results_outside_the_new_rules_are_unchanged() {
    let inputs = golden_inputs();
    if let Ok(path) = std::env::var("AA_GOLDEN_WRITE") {
        let lines: Vec<String> = inputs.iter().map(|i| result(i).to_string()).collect();
        std::fs::write(path, lines.join("\n") + "\n").unwrap();
        return;
    }
    let golden = include_str!("data/age_pension_golden.jsonl");
    let expected: Vec<Value> = golden
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(expected.len(), 96);
    for (input, expected) in inputs.into_iter().zip(&expected) {
        let now = result(&stating(input.clone(), json!({})));
        assert_eq!(&now, expected, "{input}");
    }
}
