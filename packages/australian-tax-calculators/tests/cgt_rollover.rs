// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! CGT roll-overs from stated facts (ITAA 1997 Subdivisions 122-A, 126-A and
//! 152-E), read on 26 November 2026, against the Act's own examples (the land
//! of s 126-5(5), s 152-415's $100,000 gain and Nick's business in s 122-50)
//! and made-up figures worked by hand
//! (docs/calculation-evidence.md#cgt-roll-overs).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "cgt-rollover";

fn input(rollover: &str, facts: Value) -> Value {
    json!({"rollover": rollover, "facts": facts, "year": "2025-26", "scope_confirmed": true})
}

fn run(rollover: &str, facts: Value) -> Value {
    api::evaluate(CMD, &input(rollover, facts)).unwrap()
}

fn code(rollover: &str, facts: Value) -> &'static str {
    api::evaluate_coded(CMD, &input(rollover, facts))
        .unwrap_err()
        .code
}

fn amount<'a>(r: &'a Value, key: &str) -> &'a str {
    r["amounts"][key].as_str().unwrap()
}

fn warned(r: &Value, part: &str) -> bool {
    r["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w.as_str().unwrap().contains(part))
}

fn land() -> Value {
    json!({"transferor": "individual", "basis": "court_order", "event": "A1",
           "event_date": "2025-11-03", "trading_stock": false,
           "transferor_acquired": "2012-05-14", "cost_base": "10000.00",
           "reduced_cost_base": "10000.00", "asset_class": "ordinary"})
}

fn with(mut facts: Value, key: &str, value: Value) -> Value {
    facts[key] = value;
    facts
}

/// s 126-5(5) example: the land's $10,000 cost base becomes the first
/// element of the transferee's.
#[test]
fn m1_the_act_s_land_example() {
    let r = run("marriage_breakdown", land());
    assert_eq!(amount(&r, "cost_base"), "10000.00");
    assert_eq!(amount(&r, "reduced_cost_base"), "10000.00");
    assert_eq!(r["acquired"], "2025-11-03");
    assert_eq!(r["acquired_for_discount"], "2012-05-14");
    assert_eq!(r["asset_class"], "ordinary");
    assert_eq!(r["rollover"], "marriage_breakdown");
    assert!(r["amounts"].get("disregarded_capital_gain").is_none());
    assert!(warned(&r, "118-178"));
    assert!(!warned(&r, "126-15(2)"));
}

#[test]
fn m2_the_disregarded_gain_or_loss_for_stated_proceeds() {
    let r = run(
        "marriage_breakdown",
        with(land(), "capital_proceeds", json!("25000.00")),
    );
    assert_eq!(amount(&r, "disregarded_capital_gain"), "15000.00");
    assert_eq!(amount(&r, "disregarded_capital_loss"), "0.00");
    let facts = with(land(), "reduced_cost_base", json!("9000.00"));
    let r = run(
        "marriage_breakdown",
        with(facts, "capital_proceeds", json!("8000.00")),
    );
    assert_eq!(amount(&r, "disregarded_capital_gain"), "0.00");
    assert_eq!(amount(&r, "disregarded_capital_loss"), "1000.00");
}

#[test]
fn m3_agreement_grounds_need_the_s_126_25_conditions() {
    for basis in [
        "financial_agreement",
        "part_viiiab_financial_agreement",
        "state_law_agreement",
    ] {
        let facts = with(land(), "basis", json!(basis));
        let met = with(
            facts.clone(),
            "separated_and_breakdown_connected",
            json!(true),
        );
        assert_eq!(
            amount(&run("marriage_breakdown", met), "cost_base"),
            "10000.00"
        );
        let unmet = with(
            facts.clone(),
            "separated_and_breakdown_connected",
            json!(false),
        );
        assert_eq!(
            code("marriage_breakdown", unmet),
            "cgt_rollover.not_available"
        );
        assert_eq!(
            code("marriage_breakdown", facts),
            "cgt_parcels.fact_missing"
        );
    }
    for basis in ["maintenance_agreement", "arbitral_award"] {
        let facts = with(land(), "basis", json!(basis));
        assert_eq!(
            amount(&run("marriage_breakdown", facts.clone()), "cost_base"),
            "10000.00"
        );
        let extra = with(facts, "separated_and_breakdown_connected", json!(true));
        assert_eq!(code("marriage_breakdown", extra), "input.field_unknown");
    }
}

#[test]
fn m4_exclusions_and_pre_cgt_assets_are_refused() {
    let stock = with(land(), "trading_stock", json!(true));
    assert_eq!(
        code("marriage_breakdown", stock),
        "cgt_rollover.not_available"
    );
    let b1 = with(land(), "event", json!("B1"));
    assert_eq!(
        code("marriage_breakdown", b1.clone()),
        "cgt_parcels.fact_missing"
    );
    let kept = with(b1.clone(), "title_passes", json!(false));
    assert_eq!(
        code("marriage_breakdown", kept),
        "cgt_rollover.not_available"
    );
    let passes = with(b1, "title_passes", json!(true));
    assert_eq!(
        amount(&run("marriage_breakdown", passes), "cost_base"),
        "10000.00"
    );
    let old = with(land(), "transferor_acquired", json!("1985-09-19"));
    assert_eq!(code("marriage_breakdown", old), "cgt_rollover.pre_cgt");
    let first = with(land(), "transferor_acquired", json!("1985-09-20"));
    let r = run("marriage_breakdown", first);
    assert!(warned(&r, "Division 114"));
    let late = with(land(), "transferor_acquired", json!("2025-11-04"));
    assert_eq!(
        code("marriage_breakdown", late),
        "cgt_parcels.fact_conflict"
    );
}

#[test]
fn m5_creation_cases_company_transferors_and_classes() {
    let facts = json!({"transferor": "company", "basis": "court_order", "event": "D2",
                       "event_date": "2026-06-30", "expenditure": "500.00"});
    let r = run("marriage_breakdown", facts);
    assert_eq!(amount(&r, "cost_base"), "500.00");
    assert_eq!(r["acquired_for_discount"], "2026-06-30");
    assert!(warned(&r, "126-15(2)"));
    let r = run(
        "marriage_breakdown",
        with(land(), "asset_class", json!("collectable")),
    );
    assert_eq!(r["asset_class"], "collectable");
    let trustee = with(land(), "transferor", json!("trustee"));
    assert!(warned(&run("marriage_breakdown", trustee), "126-15(2)"));
    let early = with(land(), "event_date", json!("2025-06-30"));
    assert_eq!(
        code("marriage_breakdown", early),
        "cgt_parcels.fact_conflict"
    );
    let odd = with(land(), "event", json!("C2"));
    assert_eq!(code("marriage_breakdown", odd), "cgt_parcels.fact_invalid");
}

fn rolled() -> Value {
    json!({"basic_conditions_met": true, "capital_gain": "25000.00", "disregarded": "20000.00",
           "last_cgt_event": "2025-09-30"})
}

fn asset(label: &str, date: &str, cost: &str, qualifies: bool) -> Value {
    json!({"label": label, "date": date, "first_element_cost_base": cost,
           "incidental_costs": "1000.00", "qualifies": qualifies})
}

/// s 152-415 example: $100,000 reduced to $25,000 by the other concessions,
/// $20,000 disregarded, $5,000 left.
#[test]
fn s1_the_act_s_example_and_the_period() {
    let r = run("small_business", rolled());
    assert_eq!(amount(&r, "capital_gain"), "25000.00");
    assert_eq!(amount(&r, "disregarded_capital_gain"), "20000.00");
    assert_eq!(amount(&r, "remaining_capital_gain"), "5000.00");
    assert_eq!(
        r["replacement_asset_period"],
        json!({"from": "2024-09-30", "to": "2027-09-30"})
    );
    assert_eq!(r["cgt_event"], Value::Null);
    assert!(warned(&r, "No replacement_assets"));
    assert!(warned(&r, "J2"));
}

#[test]
fn s2_j5_j6_or_neither_at_the_period_s_end() {
    let j6 = with(
        rolled(),
        "replacement_assets",
        json!([
            asset("Shop", "2026-03-01", "15000.00", true),
            asset("Van", "2026-04-01", "9000.00", false)
        ]),
    );
    let r = run("small_business", j6);
    assert_eq!(r["cgt_event"], "J6");
    assert_eq!(r["cgt_event_date"], "2027-09-30");
    assert_eq!(amount(&r, "amount_incurred"), "16000.00");
    assert_eq!(amount(&r, "capital_gain_at_period_end"), "4000.00");
    assert!(warned(&r, "115-25(3)"));
    let j5 = with(
        rolled(),
        "replacement_assets",
        json!([asset("Van", "2026-04-01", "30000.00", false)]),
    );
    let r = run("small_business", j5);
    assert_eq!(r["cgt_event"], "J5");
    assert_eq!(amount(&r, "capital_gain_at_period_end"), "20000.00");
    let covered = with(
        rolled(),
        "replacement_assets",
        json!([asset("Shop", "2024-09-30", "19000.00", true)]),
    );
    let r = run("small_business", covered);
    assert_eq!(r["cgt_event"], Value::Null);
    assert_eq!(amount(&r, "amount_incurred"), "20000.00");
    assert_eq!(amount(&r, "capital_gain_at_period_end"), "0.00");
    assert!(!warned(&r, "115-25(3)"));
}

#[test]
fn s3_period_bounds_earnouts_and_29_february() {
    for date in ["2024-09-29", "2027-10-01"] {
        let outside = with(
            rolled(),
            "replacement_assets",
            json!([asset("Shop", date, "1.00", true)]),
        );
        assert_eq!(
            code("small_business", outside),
            "cgt_parcels.fact_conflict",
            "{date}"
        );
    }
    let earnout = with(rolled(), "earnout_latest_due", json!("2028-06-30"));
    let r = run("small_business", earnout);
    assert_eq!(r["replacement_asset_period"]["to"], "2028-12-30");
    let early = with(rolled(), "earnout_latest_due", json!("2025-09-29"));
    assert_eq!(code("small_business", early), "cgt_parcels.fact_conflict");
    let mut leap = input(
        "small_business",
        with(rolled(), "last_cgt_event", json!("2024-02-29")),
    );
    leap["year"] = json!("2023-24");
    let r = api::evaluate(CMD, &leap).unwrap();
    assert_eq!(
        r["replacement_asset_period"],
        json!({"from": "2023-03-01", "to": "2026-03-01"})
    );
}

#[test]
fn s4_choices_outside_the_roll_over_are_refused() {
    let unmet = with(rolled(), "basic_conditions_met", json!(false));
    assert_eq!(code("small_business", unmet), "cgt_rollover.not_available");
    for disregarded in ["25000.01", "0.00"] {
        let bad = with(rolled(), "disregarded", json!(disregarded));
        assert_eq!(code("small_business", bad), "cgt_parcels.fact_conflict");
    }
    let all = with(rolled(), "disregarded", json!("25000.00"));
    assert_eq!(
        amount(&run("small_business", all), "remaining_capital_gain"),
        "0.00"
    );
    let empty = with(rolled(), "replacement_assets", json!([]));
    assert_eq!(code("small_business", empty), "cgt_parcels.fact_invalid");
    let extra = with(
        rolled(),
        "replacement_assets",
        json!([with(
            asset("Shop", "2026-03-01", "1.00", true),
            "note",
            json!("x")
        )]),
    );
    assert_eq!(code("small_business", extra), "input.field_unknown");
}

fn nick() -> Value {
    let precluded = |label: &str, value: &str| {
        json!({"label": label, "acquired": "2010-07-01", "asset_class": "ordinary",
               "cost_base": value, "reduced_cost_base": value, "market_value": value,
               "precluded": true})
    };
    json!({"transferor": "individual", "case": "business", "conditions_met": true,
           "event_date": "2026-01-15", "shares": 10, "liabilities": "15000.00",
           "assets": [precluded("Trading stock", "20000.00"),
                      precluded("Plant and equipment", "50000.00"),
                      precluded("Office furniture", "10000.00"),
                      {"label": "Land and buildings", "acquired": "2010-07-01",
                       "asset_class": "ordinary", "cost_base": "120000.00",
                       "reduced_cost_base": "118000.00", "market_value": "300000.00",
                       "precluded": false}]})
}

/// s 122-50(1) example: Nick's shares take $185,000, $18,500 each.
#[test]
fn c1_nick_s_business() {
    let r = run("wholly_owned_company", nick());
    assert_eq!(amount(&r, "shares_cost_base"), "185000.00");
    assert_eq!(amount(&r, "cost_base_per_share"), "18500.00");
    assert_eq!(amount(&r, "shares_reduced_cost_base"), "183000.00");
    assert_eq!(amount(&r, "reduced_cost_base_per_share"), "18300.00");
    assert_eq!(r["shares_acquired"], "2026-01-15");
    let land = &r["assets"][3];
    assert_eq!(land["cost_base"], "120000.00");
    assert_eq!(land["reduced_cost_base"], "118000.00");
    assert_eq!(land["acquired_for_discount"], "2010-07-01");
    assert!(r["assets"][0].get("cost_base").is_none());
    assert!(warned(&r, "115-34"));
    assert!(warned(&r, "precluded asset"));
    assert!(r["amounts"].get("disregarded_capital_gain").is_none());
}

fn one(cost: &str, liabilities: &str) -> Value {
    json!({"transferor": "trustee", "case": "asset", "conditions_met": true,
           "event_date": "2026-02-01", "shares": 3, "liabilities": liabilities,
           "assets": [{"label": "Land", "acquired": "2018-03-01", "asset_class": "ordinary",
                       "cost_base": cost, "reduced_cost_base": cost,
                       "market_value": "400000.00", "capital_proceeds": "400000.00",
                       "precluded": false}]})
}

#[test]
fn c2_one_asset_with_liabilities_and_proceeds() {
    let r = run("wholly_owned_company", one("300000.00", "50000.00"));
    assert_eq!(amount(&r, "shares_cost_base"), "250000.00");
    assert_eq!(amount(&r, "cost_base_per_share"), "83333.33");
    assert_eq!(amount(&r, "disregarded_capital_gain"), "100000.00");
    assert_eq!(amount(&r, "disregarded_capital_loss"), "0.00");
    assert_eq!(r["assets"][0]["disregarded_capital_gain"], "100000.00");
    assert!(warned(&r, "capital proceeds"));
    let r = run("wholly_owned_company", one("300000.00", "300000.00"));
    assert_eq!(amount(&r, "shares_cost_base"), "0.00");
    let over = one("300000.00", "300000.01");
    assert_eq!(
        code("wholly_owned_company", over),
        "cgt_rollover.not_available"
    );
    let mut none = one("300000.00", "0.00");
    none.as_object_mut().unwrap().remove("liabilities");
    assert_eq!(
        amount(&run("wholly_owned_company", none), "shares_cost_base"),
        "300000.00"
    );
}

#[test]
fn c3_each_exclusion_is_refused() {
    let mut f = one("1.00", "0.00");
    f["conditions_met"] = json!(false);
    assert_eq!(
        code("wholly_owned_company", f),
        "cgt_rollover.not_available"
    );
    let mut f = one("1.00", "0.00");
    f["assets"][0]["precluded"] = json!(true);
    assert_eq!(
        code("wholly_owned_company", f),
        "cgt_rollover.not_available"
    );
    let mut f = one("1.00", "0.00");
    f["assets"][0]["asset_class"] = json!("collectable");
    assert_eq!(
        code("wholly_owned_company", f),
        "cgt_rollover.not_available"
    );
    let mut f = nick();
    f["assets"][3]["acquired"] = json!("1985-09-19");
    assert_eq!(code("wholly_owned_company", f), "cgt_rollover.pre_cgt");
    let mut f = nick();
    f["case"] = json!("asset");
    assert_eq!(code("wholly_owned_company", f), "cgt_parcels.fact_invalid");
    let mut f = nick();
    f["liabilities"] = json!("200000.01");
    assert_eq!(
        code("wholly_owned_company", f),
        "cgt_rollover.not_available"
    );
    for shares in [json!(0), json!("10"), json!(1_000_000_001)] {
        let mut f = nick();
        f["shares"] = shares;
        assert_eq!(code("wholly_owned_company", f), "cgt_parcels.fact_invalid");
    }
    let mut f = nick();
    f["transferor"] = json!("company");
    assert_eq!(code("wholly_owned_company", f), "cgt_parcels.fact_invalid");
    let mut f = nick();
    f["assets"][0]["acquired"] = json!("2026-01-16");
    assert_eq!(code("wholly_owned_company", f), "cgt_parcels.fact_conflict");
    let mut f = nick();
    f.as_object_mut().unwrap().remove("assets");
    assert_eq!(code("wholly_owned_company", f), "cgt_parcels.fact_invalid");
}

#[test]
fn c4_indexation_era_assets_warn() {
    let mut f = one("300000.00", "0.00");
    f["assets"][0]["acquired"] = json!("1999-09-21");
    assert!(warned(&run("wholly_owned_company", f), "Division 114"));
}

#[test]
fn the_command_s_own_inputs_are_checked() {
    assert_eq!(
        code("scrip_for_scrip", land()),
        "cgt_rollover.rollover_invalid"
    );
    let mut i = input("marriage_breakdown", land());
    i["facts"] = json!([land()]);
    assert_eq!(
        api::evaluate_coded(CMD, &i).unwrap_err().code,
        "cgt_rollover.facts_invalid"
    );
    let unknown = with(land(), "note", json!("x"));
    assert_eq!(code("marriage_breakdown", unknown), "input.field_unknown");
    let mut i = input("marriage_breakdown", land());
    i["year"] = json!("2019-20");
    assert_eq!(
        api::evaluate_coded(CMD, &i).unwrap_err().code,
        "period.unsupported"
    );
    let mut i = input("marriage_breakdown", land());
    i["extra"] = json!(true);
    assert_eq!(
        api::evaluate_coded(CMD, &i).unwrap_err().code,
        "input.field_unknown"
    );
}
