// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The scrip for scrip event of cgt-cost-base-adjustments (ITAA 1997
//! ss 124-780, 124-781, 124-785 and 124-790), read on 26 November 2026,
//! against the Act's examples (Lyn and Glenn in s 124-785, Ken in s 124-790)
//! and made-up figures worked by hand
//! (docs/calculation-evidence.md#cgt-cost-base-adjustments).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "cgt-cost-base-adjustments";

fn run(input: &Value) -> Value {
    let result = api::evaluate(COMMAND, input).unwrap();
    let envelope = json!({"command": COMMAND, "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(COMMAND, input).unwrap_err().code
}

fn parcel(id: &str, acquired: &str, units: &str, cost: &str) -> Value {
    json!({"parcel": id, "security": "Aim", "acquired": acquired, "units": units,
           "acquisition_cost": cost, "incidental_costs": "0.00"})
}

fn exchange(ratio: &str, value: &str, cash: Option<&str>) -> Value {
    let mut e = json!({"event": "scrip_for_scrip", "security": "Aim", "date": "2025-10-01",
                       "new_security": "LBZ", "new_units_per_unit": ratio,
                       "market_value_per_new_unit": value, "rollover": true});
    if let Some(c) = cash {
        e["cash_per_unit"] = json!(c);
    }
    e
}

fn facts(parcels: Value, events: Value) -> Value {
    json!({"entity": "individual", "parcels": parcels, "events": events,
           "year": "2025-26", "scope_confirmed": true})
}

fn amount<'a>(r: &'a Value, key: &str) -> &'a str {
    r["amounts"][key].as_str().unwrap()
}

/// s 124-790(2) example: Ken's 100 Aim shares, cost base $200, for one LBZ
/// share worth $4 and $1 each: $40 goes to the cash, a $60 gain, and the
/// LBZ shares take $160.
#[test]
fn ken_s_partial_roll_over() {
    let r = run(&facts(
        json!([parcel("A1", "2015-01-01", "100", "200.00")]),
        json!([exchange("1", "4", Some("1"))]),
    ));
    assert_eq!(amount(&r, "discount_capital_gains"), "60.00");
    assert_eq!(amount(&r, "other_capital_gains"), "0.00");
    assert_eq!(amount(&r, "cost_base_held"), "160.00");
    let parcels = r["parcels"].as_array().unwrap();
    assert_eq!(parcels.len(), 1);
    assert_eq!(parcels[0]["parcel"], "LBZ from A1");
    assert_eq!(parcels[0]["security"], "LBZ");
    assert_eq!(parcels[0]["units"], "100");
    assert_eq!(parcels[0]["acquired"], "2015-01-01");
    assert_eq!(parcels[0]["acquisition_cost"], "160.00");
    let rows = r["adjustments"].as_array().unwrap();
    assert_eq!(rows[0]["change"], "scrip_for_scrip");
    assert_eq!(rows[0]["amount"], "100.00");
    assert_eq!(rows[0]["capital_gain"], "60.00");
    assert_eq!(rows[0]["cost_base_after"], "0.00");
    assert_eq!(rows[1]["change"], "scrip_for_scrip_replacement");
    assert_eq!(rows[1]["cost_base_after"], "160.00");
}

/// s 124-785 examples 1 and 2: Lyn's $10 share becomes a $10 share; Glenn's
/// $10 and $11 shares become one share with $21.
#[test]
fn lyn_and_glenn() {
    let r = run(&facts(
        json!([parcel("L", "2020-01-01", "1", "10.00")]),
        json!([exchange("1", "12", None)]),
    ));
    assert_eq!(r["parcels"][0]["acquisition_cost"], "10.00");
    assert_eq!(amount(&r, "total_capital_gains"), "0.00");
    assert_eq!(r["adjustments"][0]["amount"], Value::Null);
    let r = run(&facts(
        json!([
            parcel("G1", "2020-01-01", "1", "10.00"),
            parcel("G2", "2021-01-01", "1", "11.00")
        ]),
        json!([exchange("0.5", "50", None)]),
    ));
    assert_eq!(amount(&r, "cost_base_held"), "21.00");
    let units: Vec<&str> = r["parcels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["units"].as_str().unwrap())
        .collect();
    assert_eq!(units, ["0.5", "0.5"]);
}

/// Costs of owning stay out of the reduced cost base on both sides, and the
/// ineligible part takes its share of each.
#[test]
fn costs_of_owning_split_with_the_cash() {
    let mut p = parcel("A1", "2025-05-01", "100", "300.00");
    p["ownership_costs"] = json!("100.00");
    let r = run(&facts(json!([p]), json!([exchange("2", "4", Some("2"))])));
    // Proceeds 800 + 200 = 1,000: a fifth of 300 and of 100 to the cash.
    assert_eq!(r["parcels"][0]["acquisition_cost"], "240.00");
    assert_eq!(r["parcels"][0]["ownership_costs"], "80.00");
    assert_eq!(amount(&r, "reduced_cost_base_held"), "240.00");
    assert_eq!(amount(&r, "cost_base_held"), "320.00");
    // Held under 12 months: the $120 gain is not a discount gain.
    assert_eq!(amount(&r, "other_capital_gains"), "120.00");
}

/// The replacement shares carry on: a later return of capital reduces them,
/// and a trust's units can be exchanged too.
#[test]
fn later_events_and_trust_units() {
    let roc = json!({"event": "return_of_capital", "security": "LBZ", "date": "2026-02-01",
                     "per_unit": "0.50"});
    let r = run(&facts(
        json!([parcel("A1", "2015-01-01", "100", "200.00")]),
        json!([exchange("1", "4", None), roc]),
    ));
    assert_eq!(amount(&r, "cost_base_held"), "150.00");
    let deferred = json!({"event": "tax_deferred", "security": "Aim", "income_year": "2024-25",
                          "amount": "20.00"});
    let r = run(&facts(
        json!([parcel("A1", "2015-01-01", "100", "200.00")]),
        json!([deferred, exchange("1", "4", None)]),
    ));
    assert_eq!(amount(&r, "cost_base_held"), "180.00");
}

#[test]
fn what_the_roll_over_does_not_cover_is_refused() {
    let one = || json!([parcel("A1", "2015-01-01", "100", "400.00")]);
    let mut e = exchange("1", "4", None);
    e["rollover"] = json!(false);
    assert_eq!(
        code(&facts(one(), json!([e]))),
        "cgt_cost_base.scrip_without_rollover"
    );
    let even = facts(one(), json!([exchange("1", "4", None)]));
    assert_eq!(code(&even), "cgt_cost_base.scrip_no_gain");
    let gain = facts(one(), json!([exchange("1", "4.01", None)]));
    assert!(api::evaluate_coded(COMMAND, &gain).is_ok());
    let old = facts(
        json!([parcel("A1", "1985-09-19", "100", "100.00")]),
        json!([exchange("1", "4", None)]),
    );
    assert_eq!(code(&old), "cgt_cost_base.pre_cgt");
    for (key, value) in [
        ("rollover", json!("yes")),
        ("new_units_per_unit", json!("0")),
        ("market_value_per_new_unit", json!("-1")),
        ("cash_per_unit", json!("0")),
    ] {
        let mut e = exchange("1", "5", None);
        e[key] = value;
        assert_eq!(
            code(&facts(one(), json!([e]))),
            "cgt_parcels.fact_invalid",
            "{key}"
        );
    }
    let mut e = exchange("1", "5", None);
    e.as_object_mut()
        .unwrap()
        .remove("market_value_per_new_unit");
    assert_eq!(code(&facts(one(), json!([e]))), "cgt_parcels.fact_missing");
    let mut e = exchange("1", "5", None);
    e["note"] = json!("x");
    assert_eq!(code(&facts(one(), json!([e]))), "input.field_unknown");
    let twice = json!([exchange("1", "5", None), exchange("1", "5", None)]);
    assert_eq!(code(&facts(one(), twice)), "cgt_parcels.fact_conflict");
}
