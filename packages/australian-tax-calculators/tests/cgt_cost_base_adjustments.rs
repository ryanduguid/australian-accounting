// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Cost base adjustments for parcels of shares or units (ITAA 1997 ss 104-70,
//! 104-107A to 104-107C, 104-135 and Subdivision 125-B) against the ATO's
//! Wesfarmers demerger of Coles examples (Tom, Sarah and Denise), CR 2021/50's
//! Endeavour figures, the Guide to capital gains tax 2025 (Rob, example 45),
//! the Personal investors guide to capital gains tax 2025 (Bob, Ilena and
//! Miriam, examples 26 to 28) and hand-worked cases, all read on 22 November
//! 2026 (docs/calculation-evidence.md#cgt-cost-base-adjustments).
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

fn parcel(id: &str, security: &str, acquired: &str, units: &str, cost: &str) -> Value {
    json!({"parcel": id, "security": security, "acquired": acquired, "units": units,
           "acquisition_cost": cost, "incidental_costs": "0.00"})
}

fn facts(entity: &str, year: &str, parcels: Value, events: Value) -> Value {
    json!({"entity": entity, "parcels": parcels, "events": events,
           "year": year, "scope_confirmed": true})
}

fn demerger(key: &str, security: &str, new_security: &str, rollover: bool) -> Value {
    json!({"event": "demerger", "demerger": key, "security": security,
           "new_security": new_security, "rollover": rollover})
}

/// Each output parcel's name and acquisition cost (its reduced cost base).
fn held(result: &Value) -> Vec<(String, String)> {
    result["parcels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let text = |k: &str| p[k].as_str().unwrap().to_string();
            (text("parcel"), text("acquisition_cost"))
        })
        .collect()
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// Tom's post-CGT parcels (examples 1 and 2): 2,000 at $4.00 and 1,000 at
/// $15.00; the dates are fabricated.
fn tom(rollover: bool) -> Value {
    facts(
        "individual",
        "2018-19",
        json!([
            parcel("T2", "WES", "2005-03-01", "2000", "8000.00"),
            parcel("T3", "WES", "2018-01-10", "1000", "15000.00")
        ]),
        json!([demerger("wesfarmers_coles_2018", "WES", "COL", rollover)]),
    )
}

/// $23,000 x 0.7109 = $16,350.70 to the Wesfarmers shares and x 0.2891 =
/// $6,649.30 to the Coles shares ($5.4502 and $2.2164 a share), spread by
/// units; without roll-over the $5.68 capital reduction makes $1.68 a share,
/// $3,360, on the $4.00 parcel and nothing on the $15.00 one.
#[test]
fn tom_s_demerger_without_roll_over() {
    let result = run(&tom(false));
    assert_eq!(
        held(&result),
        [
            ("T2", "10900.47"),
            ("T3", "5450.23"),
            ("COL from T2", "4432.87"),
            ("COL from T3", "2216.43")
        ]
        .map(|(a, b)| (a.to_string(), b.to_string()))
    );
    assert_eq!(amount(&result, "cost_base_held"), "23000.00");
    assert_eq!(amount(&result, "discount_capital_gains"), "3360.00");
    assert_eq!(amount(&result, "other_capital_gains"), "0.00");
    let rows = result["adjustments"].as_array().unwrap();
    assert_eq!(rows[0]["amount"], "11360.00");
    assert_eq!(rows[0]["capital_gain"], "3360.00");
    assert_eq!(rows[2]["capital_gain"], "0.00");
    // The new shares keep the head shares' dates for the discount.
    assert_eq!(result["parcels"][2]["acquired"], "2005-03-01");
}

#[test]
fn tom_s_roll_over_disregards_the_gain_and_splits_the_same() {
    let result = run(&tom(true));
    assert_eq!(amount(&result, "total_capital_gains"), "0.00");
    assert_eq!(held(&result), held(&run(&tom(false))));
}

/// A G1 gain on a parcel held under 12 months is not a discount capital gain.
#[test]
fn a_young_parcel_s_g1_gain_is_not_discounted() {
    let mut input = tom(false);
    input["parcels"][0]["acquired"] = json!("2018-06-01");
    let result = run(&input);
    assert_eq!(amount(&result, "discount_capital_gains"), "0.00");
    assert_eq!(amount(&result, "other_capital_gains"), "3360.00");
}

/// Sarah (example 3): 150 shares at $43.00. $6,450 x 0.7109 = $4,585.305 and
/// x 0.2891 = $1,864.695, each rounded on its own; her Coles shares sold at
/// $11.9921 ($1,798.82) then lose $65.88, as the ATO says.
#[test]
fn sarah_s_cost_bases_round_each_side_on_its_own() {
    let input = facts(
        "individual",
        "2018-19",
        json!([parcel("S", "WES", "2016-05-02", "150", "6450.00")]),
        json!([demerger("wesfarmers_coles_2018", "WES", "COL", true)]),
    );
    let result = run(&input);
    let expected = [("S", "4585.31"), ("COL from S", "1864.70")];
    assert_eq!(
        held(&result),
        expected.map(|(a, b)| (a.to_string(), b.to_string()))
    );
    assert_eq!(amount(&result, "cost_base_held"), "6450.01");
}

/// Denise's parcels (example 4), as a resident with roll-over: $5,500 gives
/// $3,909.95 and $1,590.05, spread 200 to 150 ($11.1713 and $4.5430 a share).
#[test]
fn denise_s_parcels_share_each_side_by_units() {
    let input = facts(
        "individual",
        "2018-19",
        json!([
            parcel("D1", "WES", "2010-01-04", "200", "1000.00"),
            parcel("D2", "WES", "2011-01-04", "150", "4500.00")
        ]),
        json!([demerger("wesfarmers_coles_2018", "WES", "COL", true)]),
    );
    let expected = [
        ("D1", "2234.26"),
        ("D2", "1675.69"),
        ("COL from D1", "908.60"),
        ("COL from D2", "681.45"),
    ];
    assert_eq!(
        held(&run(&input)),
        expected.map(|(a, b)| (a.to_string(), b.to_string()))
    );
}

/// CR 2021/50: 85.81% and 14.19%, and 71 cents of capital reduction a share.
#[test]
fn endeavour_follows_its_class_ruling() {
    let input = |cost: &str| {
        facts(
            "individual",
            "2021-22",
            json!([parcel("W", "WOW", "2019-02-01", "1000", cost)]),
            json!([demerger("woolworths_endeavour_2021", "WOW", "EDV", false)]),
        )
    };
    let result = run(&input("30000.00"));
    let expected = [("W", "25743.00"), ("EDV from W", "4257.00")];
    assert_eq!(
        held(&result),
        expected.map(|(a, b)| (a.to_string(), b.to_string()))
    );
    assert_eq!(amount(&result, "total_capital_gains"), "0.00");
    let low = run(&input("500.00"));
    assert_eq!(amount(&low, "discount_capital_gains"), "210.00");
}

/// A parcel acquired after the record date took no part in the demerger.
#[test]
fn a_parcel_bought_after_the_record_date_is_left_alone() {
    let mut input = tom(true);
    input["parcels"][1]["acquired"] = json!("2018-11-23");
    let result = run(&input);
    let names: Vec<String> = held(&result).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["T2", "T3", "COL from T2"]);
    assert_eq!(result["parcels"][1]["acquisition_cost"], "15000.00");
}

/// Rob (Guide example 45): 1,500 shares at $5 bought on 1 July 1996 and a
/// 50 cent return of capital: $4.50 a share, worked without indexation, with
/// a warning. The payment's date is moved into the years worked.
#[test]
fn rob_s_return_of_capital_reduces_the_unindexed_cost_base() {
    let input = facts(
        "individual",
        "2025-26",
        json!([parcel("R", "RAP", "1996-07-01", "1500", "7500.00")]),
        json!([{"event": "return_of_capital", "security": "RAP", "date": "2025-11-30",
                "per_unit": "0.50"}]),
    );
    let result = run(&input);
    assert_eq!(result["parcels"][0]["acquisition_cost"], "6750.00");
    let warnings = result["warnings"].to_string();
    assert!(warnings.contains("without indexation"), "{warnings}");
    let mut company = input.clone();
    company["entity"] = json!("company");
    assert_eq!(code(&company), "cgt_cost_base.company_indexation");
}

/// Bob (Personal investors guide example 26): cost base $1,200, reduced cost
/// base $1,050 and a $105 tax-deferred amount: $1,095 and $945.
#[test]
fn bob_s_tax_deferred_amount_cuts_both_bases() {
    let mut units = parcel("B", "OZ", "2015-03-02", "1000", "1050.00");
    units["ownership_costs"] = json!("150.00");
    let input = facts(
        "individual",
        "2024-25",
        json!([units]),
        json!([{"event": "tax_deferred", "security": "OZ", "income_year": "2024-25",
                "amount": "105.00"}]),
    );
    let result = run(&input);
    let row = &result["adjustments"][0];
    assert_eq!(row["cost_base_after"], "1095.00");
    assert_eq!(row["reduced_cost_base_after"], "945.00");
    assert_eq!(row["date"], "2025-06-30");
}

/// Ilena (example 27): $5,000 and $4,700, $30 tax-deferred and $35 tax-free:
/// $4,970 and $4,635.
#[test]
fn ilena_s_tax_free_amount_cuts_only_the_reduced_cost_base() {
    let mut units = parcel("I", "XYZ", "2015-03-02", "1000", "4700.00");
    units["ownership_costs"] = json!("300.00");
    let input = facts(
        "individual",
        "2024-25",
        json!([units]),
        json!([{"event": "tax_deferred", "security": "XYZ", "income_year": "2024-25",
                "amount": "30.00", "tax_free": "35.00"}]),
    );
    let result = run(&input);
    assert_eq!(amount(&result, "cost_base_held"), "4970.00");
    assert_eq!(amount(&result, "reduced_cost_base_held"), "4635.00");
    let p = &result["parcels"][0];
    assert_eq!(
        (&p["acquisition_cost"], &p["ownership_costs"]),
        (&json!("4635.00"), &json!("335.00"))
    );
}

/// Miriam (example 28): units at $55 and a $10 a unit AMIT shortfall, $65;
/// an excess of $10 instead, $45. 100 units are fabricated.
#[test]
fn miriam_s_amit_net_amount_moves_the_cost_base_either_way() {
    let input = |event: &str| {
        facts(
            "individual",
            "2024-25",
            json!([parcel("M", "EGF", "2020-08-03", "100", "5500.00")]),
            json!([{"event": event, "security": "EGF", "income_year": "2024-25",
                    "amount": "1000.00"}]),
        )
    };
    let up = run(&input("amit_shortfall"));
    assert_eq!(up["parcels"][0]["acquisition_cost"], "6500.00");
    let down = run(&input("amit_excess"));
    assert_eq!(down["parcels"][0]["acquisition_cost"], "4500.00");
}

/// Amounts above the cost base: G1 and E4 make a gain of the excess, an AMIT
/// excess makes an E10 gain of the excess, and on a nil cost base the whole
/// excess; both bases go to nil. Worked by hand under ss 104-135(3),
/// 104-70(4) and (5) and 104-107A(3).
#[test]
fn amounts_above_the_cost_base_are_gains_of_the_excess() {
    let input = facts(
        "trust",
        "2025-26",
        json!([
            parcel("C", "CO", "2020-01-02", "100", "50.00"),
            parcel("U", "UT", "2025-08-04", "10", "40.00"),
            parcel("A", "AM", "2019-05-06", "10", "30.00")
        ]),
        json!([{"event": "return_of_capital", "security": "CO", "date": "2025-09-01",
                "per_unit": "0.75"},
               {"event": "tax_deferred", "security": "UT", "income_year": "2025-26",
                "amount": "45.00"},
               {"event": "amit_excess", "security": "AM", "income_year": "2024-25",
                "amount": "36.00"},
               {"event": "amit_excess", "security": "AM", "income_year": "2025-26",
                "amount": "12.50"}]),
    );
    let result = run(&input);
    let gains: Vec<&str> = result["adjustments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["capital_gain"].as_str().unwrap())
        .collect();
    assert_eq!(gains, ["6.00", "25.00", "5.00", "12.50"]);
    // The 2024-25 gain is not counted; the E4 parcel was held under 12 months.
    assert_eq!(amount(&result, "discount_capital_gains"), "37.50");
    assert_eq!(amount(&result, "other_capital_gains"), "5.00");
    assert_eq!(amount(&result, "cost_base_held"), "0.00");
}

/// An amount is spread over the parcels by units, the last taking the rest,
/// and `parcels` limits it to the parcels named.
#[test]
fn a_year_s_amount_is_spread_by_units_over_the_parcels_named() {
    let input = |named: Option<Value>| {
        let mut event = json!({"event": "amit_excess", "security": "F",
                               "income_year": "2025-26", "amount": "100.00"});
        if let Some(names) = named {
            event["parcels"] = names;
        }
        facts(
            "individual",
            "2025-26",
            json!([
                parcel("P1", "F", "2021-01-04", "1", "500.00"),
                parcel("P2", "F", "2021-02-01", "2", "500.00")
            ]),
            json!([event]),
        )
    };
    let all = run(&input(None));
    let expected = [("P1", "466.67"), ("P2", "433.33")];
    assert_eq!(
        held(&all),
        expected.map(|(a, b)| (a.to_string(), b.to_string()))
    );
    let one = run(&input(Some(json!(["P2"]))));
    let expected = [("P1", "500.00"), ("P2", "400.00")];
    assert_eq!(
        held(&one),
        expected.map(|(a, b)| (a.to_string(), b.to_string()))
    );
}

/// The output parcels go straight into cgt-parcels.
#[test]
fn the_parcels_feed_cgt_parcels() {
    let mut input = tom(true);
    input["year"] = json!("2025-26");
    let adjusted = run(&input);
    let sale = json!({"security": "COL", "date": "2026-03-02", "units": "1000",
                      "capital_proceeds": "2216.43", "incidental_costs": "0.00",});
    let sold = api::evaluate(
        "cgt-parcels",
        &json!({"entity": "individual", "identification": "first_in_first_out",
                "parcels": adjusted["parcels"], "disposals": [sale],
                "other_discount_capital_gains": "0.00", "other_capital_gains": "0.00",
                "other_capital_losses": "0.00", "prior_net_capital_losses": "0.00",
                "year": "2025-26", "scope_confirmed": true}),
    )
    .unwrap();
    // FIFO takes the parcel from T2 first: 1,000 of its 2,000 shares.
    assert_eq!(sold["matches"][0]["cost_base"], "2216.44");
}

#[test]
fn years_run_from_2018_19_to_2026_27() {
    let input = |year: &str, date: &str| {
        facts(
            "individual",
            year,
            json!([parcel("P", "S", "2010-01-04", "10", "100.00")]),
            json!([{"event": "return_of_capital", "security": "S", "date": date,
                    "per_unit": "1"}]),
        )
    };
    run(&input("2018-19", "2018-07-01"));
    run(&input("2026-27", "2027-06-30"));
    assert_eq!(
        code(&input("2018-19", "2018-06-30")),
        "cgt_cost_base.event_year_unsupported"
    );
    assert_eq!(code(&input("2017-18", "2018-06-30")), "period.unsupported");
    assert_eq!(code(&input("2027-28", "2027-07-01")), "period.unsupported");
    assert_eq!(
        code(&input("2025-26", "2026-07-01")),
        "cgt_parcels.fact_conflict"
    );
}

#[test]
fn refusals_name_what_is_out_of_scope_or_wrong() {
    let mut pre_cgt = tom(true);
    pre_cgt["parcels"][0]["acquired"] = json!("1985-09-19");
    assert_eq!(code(&pre_cgt), "cgt_cost_base.pre_cgt");
    let mut unknown = tom(true);
    unknown["events"][0]["demerger"] = json!("westfield_2018");
    assert_eq!(code(&unknown), "cgt_cost_base.demerger_unknown");
    // A $1,000 cost base with $100 of it costs of owning, cut by $950.
    let mut owning = parcel("O", "S", "2015-01-05", "1", "900.00");
    owning["ownership_costs"] = json!("100.00");
    let cut = facts(
        "individual",
        "2025-26",
        json!([owning]),
        json!([{"event": "return_of_capital", "security": "S", "date": "2025-08-01",
                "per_unit": "950"}]),
    );
    assert_eq!(code(&cut), "cgt_cost_base.below_reduced_cost_base");
    let mut mixed = tom(true);
    mixed["year"] = json!("2025-26");
    mixed["events"].as_array_mut().unwrap().push(
        json!({"event": "amit_excess", "security": "WES", "income_year": "2024-25",
                     "amount": "1.00"}),
    );
    assert_eq!(code(&mixed), "cgt_parcels.fact_conflict");
    let mut nobody = tom(true);
    nobody["events"][0]["security"] = json!("BHP");
    assert_eq!(code(&nobody), "cgt_parcels.fact_conflict");
    let mut kind = tom(true);
    kind["events"][0]["event"] = json!("bonus_issue");
    assert_eq!(code(&kind), "cgt_parcels.fact_invalid");
    let mut extra = tom(true);
    extra["events"][0]["parcels"] = json!(["T2"]);
    assert_eq!(code(&extra), "input.field_unknown");
    let mut entity = tom(true);
    entity["entity"] = json!("partnership");
    assert_eq!(code(&entity), "cgt_parcels.entity_invalid");
}
