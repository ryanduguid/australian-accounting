// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The downsizer contribution test (ITAA 1997 s 292-102) in
//! `contribution-caps` and the transition to retirement maximum (SISR reg
//! 6.01(2)) in `pension-minimum`, read on 24 November 2026, against the ATO's
//! 'Downsizer super contributions' examples and its TRIS rules, each boundary
//! and each refusal; without the new inputs both commands answer as before
//! (docs/calculation-evidence.md#downsizer-contributions).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A 2025-26 contribution-caps call with nothing over a cap.
fn caps() -> Value {
    json!({
        "total_super_balance": "450000.00", "concessional_contributions": "20000.00",
        "unused_concessional_cap": "0.00", "non_concessional_contributions": "0.00",
        "under_75_in_year": true, "year": "2025-26", "scope_confirmed": true,
    })
}

/// `caps()` with a $300,000 downsizer contribution by a 70-year-old 30 days
/// after settlement, with `overrides`.
fn downsizer(overrides: Value) -> Value {
    let mut input = caps();
    let given = json!({
        "downsizer_contribution": "300000.00", "downsizer_contribution_date": "2025-10-31",
        "downsizer_age": 70, "downsizer_contract_date": "2025-08-01",
        "downsizer_settlement_date": "2025-10-01", "downsizer_longer_period_allowed": false,
        "downsizer_capital_proceeds": "800000.00", "downsizer_earlier_own": "0.00",
        "downsizer_earlier_from_proceeds": "0.00", "downsizer_conditions_met": true,
    });
    for (k, v) in given
        .as_object()
        .unwrap()
        .iter()
        .chain(overrides.as_object().unwrap())
    {
        input[k] = v.clone();
    }
    input
}

/// The plain result, checked against `derive` byte for byte.
fn run(command: &str, input: &Value) -> Value {
    let result = api::evaluate(command, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": command, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(command: &str, input: &Value) -> &'static str {
    api::evaluate_coded(command, input).unwrap_err().code
}

/// downsizer_covered and downsizer_not_covered.
fn covered(input: &Value) -> [String; 2] {
    let result = run("contribution-caps", input);
    ["downsizer_covered", "downsizer_not_covered"]
        .map(|key| result["amounts"][key].as_str().unwrap().to_string())
}

fn met(result: &Value, id: &str) -> bool {
    result["downsizer_tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["test"] == id)
        .unwrap()["met"]
        .as_bool()
        .unwrap()
}

/// Without the downsizer inputs the result is the one before, plus nothing:
/// adding them only adds the downsizer fields, a source and a warning.
#[test]
fn the_downsizer_adds_to_the_result_and_changes_nothing_else() {
    let plain = run("contribution-caps", &caps());
    assert!(plain.get("downsizer_tests").is_none());
    assert!(plain["amounts"].get("downsizer_covered").is_none());
    let with = run("contribution-caps", &downsizer(json!({})));
    for (key, value) in plain["amounts"].as_object().unwrap() {
        assert_eq!(&with["amounts"][key], value, "{key}");
    }
    assert_eq!(with["rates"], plain["rates"]);
    let sources = with["sources"].as_array().unwrap();
    assert_eq!(
        sources[..sources.len() - 1],
        plain["sources"].as_array().unwrap()[..]
    );
    assert_eq!(with["downsizer_eligible"], true);
    let scope = with["scope"].as_str().unwrap();
    assert!(
        plain["scope"]
            .as_str()
            .unwrap()
            .contains("the CGT cap, downsizer and")
    );
    assert!(!scope.contains("the CGT cap, downsizer and"), "{scope}");
    assert!(
        scope.contains("tested under ITAA 1997 s 292-102"),
        "{scope}"
    );
    let warnings = with["warnings"].as_array().unwrap();
    assert_eq!(
        warnings.len(),
        plain["warnings"].as_array().unwrap().len() + 1
    );
}

/// George and Jane sell for $800,000; each may contribute $300,000. Alisha
/// sells for $1.5 million and offers $400,000.
#[test]
fn george_jane_and_alisha() {
    assert_eq!(covered(&downsizer(json!({}))), ["300000.00", "0.00"]);
    let jane = downsizer(json!({"downsizer_earlier_from_proceeds": "300000.00"}));
    assert_eq!(covered(&jane), ["300000.00", "0.00"]);
    let alisha = downsizer(json!({"downsizer_contribution": "400000.00",
        "downsizer_capital_proceeds": "1500000.00"}));
    assert_eq!(covered(&alisha), ["300000.00", "100000.00"]);
}

/// Bruce and Betty receive $400,000: Betty $300,000, then Bruce $100,000;
/// a cent more for Bruce is not covered. Robert and Wendy sell 20% for
/// $100,000 and contribute $60,000 each.
#[test]
fn the_proceeds_limit() {
    let betty = downsizer(json!({"downsizer_capital_proceeds": "400000.00"}));
    assert_eq!(covered(&betty), ["300000.00", "0.00"]);
    let bruce = |amount: &str| {
        downsizer(json!({"downsizer_capital_proceeds": "400000.00",
            "downsizer_earlier_from_proceeds": "300000.00", "downsizer_contribution": amount}))
    };
    assert_eq!(covered(&bruce("100000.00")), ["100000.00", "0.00"]);
    assert_eq!(covered(&bruce("100000.01")), ["100000.00", "0.01"]);
    let wendy = |earlier: &str| {
        downsizer(json!({"downsizer_capital_proceeds": "100000.00",
            "downsizer_earlier_from_proceeds": earlier, "downsizer_contribution": "60000.00"}))
    };
    assert_eq!(covered(&wendy("0.00")), ["60000.00", "0.00"]);
    assert_eq!(covered(&wendy("60000.00")), ["40000.00", "20000.00"]);
    assert_eq!(covered(&wendy("150000.00")), ["0.00", "60000.00"]);
}

/// s 292-102(3)(a): $300,000 less what is already covered for the person.
#[test]
fn the_300000_limit() {
    let call = |earlier: &str, amount: &str| {
        covered(&downsizer(
            json!({"downsizer_earlier_own": earlier, "downsizer_contribution": amount}),
        ))
    };
    assert_eq!(call("100000.00", "200000.00"), ["200000.00", "0.00"]);
    assert_eq!(call("100000.00", "200000.01"), ["200000.00", "0.01"]);
    assert_eq!(call("300000.00", "1.00"), ["0.00", "1.00"]);
    assert_eq!(call("400000.00", "1.00"), ["0.00", "1.00"]);
}

/// Ben settles on 1 August 2024 and is allowed until 1 February 2025; the
/// 90 days run from the day after the change of ownership.
#[test]
fn ben_and_the_90_days() {
    let ben = |made: &str, longer: bool| {
        downsizer(
            json!({"year": "2024-25", "downsizer_contract_date": "2024-06-15",
            "downsizer_settlement_date": "2024-08-01", "downsizer_contribution_date": made,
            "downsizer_longer_period_allowed": longer, "downsizer_age": 77}),
        )
    };
    let result = run("contribution-caps", &ben("2025-02-01", true));
    assert!(met(&result, "within_90_days"));
    assert_eq!(covered(&ben("2025-02-01", true))[0], "300000.00");
    let late = run("contribution-caps", &ben("2025-02-01", false));
    assert!(!met(&late, "within_90_days"));
    assert_eq!(late["downsizer_eligible"], false);
    assert_eq!(covered(&ben("2025-02-01", false)), ["0.00", "300000.00"]);
    // 1 August plus 90 days is 30 October.
    assert_eq!(covered(&ben("2024-10-30", false))[0], "300000.00");
    assert_eq!(covered(&ben("2024-10-31", false))[0], "0.00");
    assert_eq!(covered(&ben("2024-08-01", false))[0], "300000.00");
}

/// Rebecca is 54 when she would contribute; James is 60.
#[test]
fn rebecca_and_james() {
    let call = |age: i64| downsizer(json!({"downsizer_age": age}));
    let rebecca = run("contribution-caps", &call(54));
    assert!(!met(&rebecca, "age"));
    assert_eq!(covered(&call(54)), ["0.00", "300000.00"]);
    assert_eq!(covered(&call(55))[0], "300000.00");
    assert_eq!(covered(&call(60))[0], "300000.00");
}

#[test]
fn the_contract_date_and_the_stated_conditions() {
    let call = |contract: &str| downsizer(json!({"downsizer_contract_date": contract}));
    assert!(!met(
        &run("contribution-caps", &call("2018-06-30")),
        "contract_date"
    ));
    assert!(met(
        &run("contribution-caps", &call("2018-07-01")),
        "contract_date"
    ));
    let unmet = downsizer(json!({"downsizer_conditions_met": false}));
    assert!(!met(&run("contribution-caps", &unmet), "stated_conditions"));
    assert_eq!(covered(&unmet), ["0.00", "300000.00"]);
}

/// The downsizer inputs beside the unused cap worked out by year.
#[test]
fn with_the_carry_forward_inputs() {
    let mut input = downsizer(json!({}));
    input
        .as_object_mut()
        .unwrap()
        .remove("unused_concessional_cap");
    let years = ["2020-21", "2021-22", "2022-23", "2023-24", "2024-25"];
    input["earlier_concessional_contributions"] = years
        .iter()
        .map(|y| json!({"year": y, "concessional_contributions": "0.00"}))
        .collect();
    input["defined_benefit_interest"] = json!(false);
    let result = run("contribution-caps", &input);
    assert_eq!(result["amounts"]["downsizer_covered"], "300000.00");
}

#[test]
fn each_downsizer_refusal() {
    let refused = |overrides: Value| code("contribution-caps", &downsizer(overrides));
    assert_eq!(
        refused(json!({"downsizer_contribution_date": "2025-06-30"})),
        "downsizer.date_outside_year"
    );
    assert_eq!(
        refused(json!({"downsizer_contribution_date": "2026-07-01"})),
        "downsizer.date_outside_year"
    );
    assert_eq!(
        refused(json!({"downsizer_settlement_date": "2025-07-31"})),
        "downsizer.settlement_before_contract"
    );
    assert_eq!(
        refused(json!({"downsizer_contribution_date": "2025-09-30"})),
        "downsizer.contribution_before_settlement"
    );
    assert_eq!(
        refused(json!({"downsizer_contract_date": "2025-02-30"})),
        "input.date_invalid"
    );
    assert_eq!(refused(json!({"downsizer_age": "70"})), "input.age_invalid");
    assert_eq!(refused(json!({"downsizer_age": 151})), "input.age_invalid");
    assert_eq!(
        refused(json!({"downsizer_conditions_met": "yes"})),
        "input.flag_invalid"
    );
    assert_eq!(
        refused(json!({"downsizer_contribution": "-1.00"})),
        "input.amount_invalid"
    );
    let mut partial = caps();
    partial["downsizer_contribution"] = json!("1000.00");
    assert_eq!(code("contribution-caps", &partial), "input.missing_field");
}

/// SISR reg 6.01(2): 10% of the 1 July balance; the minimum is unchanged.
#[test]
fn the_ttr_maximum() {
    let call = |balance: &str, age: i64, days: i64, ttr: Option<bool>| {
        let mut input = json!({"account_balance": balance, "age": age, "days": days,
            "year": "2025-26", "scope_confirmed": true});
        if let Some(ttr) = ttr {
            input["transition_to_retirement"] = json!(ttr);
        }
        run("pension-minimum", &input)
    };
    let plain = call("500000.00", 60, 365, None);
    let ttr = call("500000.00", 60, 365, Some(true));
    assert_eq!(
        ttr["amounts"]["minimum_payment"],
        plain["amounts"]["minimum_payment"]
    );
    assert_eq!(plain["amounts"]["minimum_payment"], "20000.00");
    assert_eq!(ttr["amounts"]["maximum_payment"], "50000.00");
    assert!(plain["amounts"].get("maximum_payment").is_none());
    assert_eq!(call("500000.00", 60, 365, Some(false)), plain);
    let scope = ttr["scope"].as_str().unwrap();
    assert!(
        plain["scope"]
            .as_str()
            .unwrap()
            .contains("the transition to retirement 10% maximum, ")
    );
    assert!(
        !scope.contains("the transition to retirement 10% maximum, "),
        "{scope}"
    );
    assert!(
        scope.contains("Includes the transition to retirement 10% maximum"),
        "{scope}"
    );
    // Commenced on 1 January 2026: the minimum is pro-rated, the maximum is not.
    let part = call("400000.00", 62, 181, Some(true));
    assert_eq!(part["amounts"]["minimum_payment"], "7930.00");
    assert_eq!(part["amounts"]["maximum_payment"], "40000.00");
    // Within the last 30 days nothing is required, and the maximum stays.
    let late = call("123456.78", 58, 30, Some(true));
    assert_eq!(late["amounts"]["minimum_payment"], "0.00");
    assert_eq!(late["amounts"]["maximum_payment"], "12345.68");
    let mut bad = json!({"account_balance": "1.00", "age": 60, "days": 365,
        "year": "2025-26", "scope_confirmed": true, "transition_to_retirement": "yes"});
    assert_eq!(code("pension-minimum", &bad), "input.flag_invalid");
    bad["transition_to_retirement"] = json!(true);
    bad["year"] = json!("2023-24");
    assert_eq!(code("pension-minimum", &bad), "period.unsupported");
}
