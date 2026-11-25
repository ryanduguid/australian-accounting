// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Every state duty, foreign purchaser surcharge, land tax, first home owner
//! grant, motor vehicle duty and insurance duty jurisdiction, and each
//! Victorian property tax, carries a passage quoted from its primary source,
//! with the provision, the URL and the day it was read, in the worksheet
//! catalogue and in each result.

use australian_tax_calculators::api;
use serde_json::{Value, json};

const ALL: [&str; 8] = ["nsw", "vic", "qld", "wa", "sa", "tas", "act", "nt"];

/// Each command, its worksheet, the jurisdictions it takes and those whose
/// catalogue example refuses (the ACT has paid no grant since 1 July 2019).
const COMMANDS: [(&str, &str, &[&str], &[&str]); 6] = [
    ("transfer-duty", "transfer_duty", &ALL, &[]),
    (
        "foreign-purchaser-surcharge",
        "foreign_purchaser_surcharge",
        &ALL,
        &[],
    ),
    (
        "land-tax",
        "land_tax",
        &["nsw", "vic", "qld", "wa", "sa", "tas", "act"],
        &[],
    ),
    (
        "first-home-owner-grant",
        "first_home_owner_grant",
        &ALL,
        &["act"],
    ),
    ("motor-vehicle-duty", "motor_vehicle_duty", &ALL, &[]),
    ("insurance-duty", "insurance_duty", &ALL, &[]),
];

/// The Victorian property taxes, by the `tax` input that picks a result's
/// passages.
const TAXES: [&str; 3] = [
    "vacant_residential_land",
    "commercial_industrial",
    "windfall_gains",
];

fn quotes_in(catalogue: &Value, kind: &str) -> Vec<Value> {
    catalogue[kind]["source_quotes"]
        .as_array()
        .unwrap_or_else(|| panic!("{kind} has no source_quotes"))
        .clone()
}

fn catalogue_quotes(kind: &str) -> Vec<Value> {
    quotes_in(
        &api::evaluate("worksheet-catalogue", &json!({})).unwrap(),
        kind,
    )
}

fn of(quotes: &[Value], place: &str) -> Vec<Value> {
    quotes
        .iter()
        .filter(|q| q["jurisdiction"] == place)
        .cloned()
        .collect()
}

fn assert_quote_meta(quote: &Value, kind: &str) {
    let text = |field: &str| quote[field].as_str().unwrap_or_default();
    for field in ["provision", "source", "passage"] {
        assert!(!text(field).trim().is_empty(), "{kind} {field}");
    }
    assert!(text("url").starts_with("https://"), "{}", text("url"));
    let host = text("url")[8..].split('/').next().unwrap();
    assert!(host.ends_with(".gov.au"), "{host}");
    assert_eq!(text("verified"), "2026-11-24", "{kind}");
    assert!(!text("passage").contains('\u{200b}'), "{kind}");
}

#[test]
fn every_jurisdiction_has_a_quoted_passage_and_its_date() {
    for (_, kind, places, _) in COMMANDS {
        let quotes = catalogue_quotes(kind);
        for place in places {
            assert!(!of(&quotes, place).is_empty(), "{kind} {place}");
        }
        for quote in &quotes {
            let text = |field: &str| quote[field].as_str().unwrap_or_default();
            assert!(places.contains(&text("jurisdiction")), "{quote}");
            assert_quote_meta(quote, kind);
        }
    }
}

#[test]
fn other_worksheets_carry_no_quotes() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut quoted: Vec<&str> = COMMANDS.iter().map(|c| c.1).collect();
    quoted.push("vic_property_taxes");
    for (kind, entry) in catalogue.as_object().unwrap() {
        assert_eq!(
            entry.get("source_quotes").is_some(),
            quoted.contains(&kind.as_str()),
            "{kind}"
        );
    }
}

/// A passenger car or a general insurance policy in `place`, with the facts
/// the jurisdiction asks for, as tests/motor_vehicle_duty.rs and
/// tests/insurance_duty.rs give them.
fn duty(kind: &str, place: &str) -> Value {
    let mut v = json!({"jurisdiction": place, "year": "2026-27", "scope_confirmed": true});
    let extra = if kind == "motor_vehicle_duty" {
        v["vehicle_class"] = json!("passenger");
        v["dutiable_value"] = json!("45000");
        v["duty_date"] = json!("2026-09-15");
        match place {
            "qld" => json!({"engine": "up_to_4_cylinders"}),
            "vic" => json!({"green_car": false, "primary_producer": false}),
            "act" => json!({"emissions_category": "c", "new_vehicle": true}),
            "tas" => json!({"fleet_discount": false}),
            "nt" => json!({"electric": false}),
            _ => json!({}),
        }
    } else {
        v["premium"] = json!("1000");
        v["effected_date"] = json!("2026-09-15");
        match place {
            "nsw" => json!({"class": "type_a"}),
            "qld" => json!({"class": "class_1"}),
            "nt" => json!({"class": "general", "term_over_one_year": false}),
            _ => json!({"class": "general"}),
        }
    };
    for (key, value) in extra.as_object().unwrap() {
        v[key] = value.clone();
    }
    v
}

/// The catalogue's example for `kind`, moved to `place`.
fn example(kind: &str, place: &str) -> Value {
    if matches!(kind, "motor_vehicle_duty" | "insurance_duty") {
        return duty(kind, place);
    }
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue[kind]["example"]["facts"].clone();
    let fields = facts.as_object_mut().unwrap();
    fields.remove("kind");
    fields.insert("jurisdiction".into(), json!(place));
    if kind == "land_tax" {
        let year = if matches!(place, "nsw" | "vic") {
            "2026"
        } else {
            "2026-27"
        };
        fields.insert("year".into(), json!(year));
        if place == "wa" {
            fields.insert("metropolitan_region".into(), json!("none"));
        }
    }
    if kind == "first_home_owner_grant" && place == "wa" {
        fields.insert("north_of_26th".into(), json!(false));
    }
    facts
}

/// A grant for which the catalogue example lacks a fact the jurisdiction
/// needs is refused; every other result carries its jurisdiction's passages.
#[test]
fn each_result_carries_its_jurisdictions_quotes() {
    for (command, kind, places, refused) in COMMANDS {
        let quotes = catalogue_quotes(kind);
        let mut carried = Vec::new();
        for place in places {
            match api::evaluate_coded(command, &example(kind, place)) {
                Ok(result) => {
                    assert_eq!(
                        result["source_quotes"],
                        Value::Array(of(&quotes, place)),
                        "{command} {place}"
                    );
                    carried.push(*place);
                }
                Err(refusal) => assert!(
                    refused.contains(place) || refusal.code == "first_home_owner_grant.fact_needed",
                    "{command} {place}: {}",
                    refusal.message
                ),
            }
        }
        let least = if kind == "first_home_owner_grant" {
            4
        } else {
            places.len()
        };
        assert!(
            carried.len() >= least && carried.contains(&"nsw"),
            "{command} {carried:?}"
        );
    }
}

/// The figure each jurisdiction's passages must state, as the command's own
/// rate table, surcharge or grant applies it (the top band for a scale, plus
/// ACT Tables 1 to 3, NT EV exemption and Service NSW's complete rule).
const FIGURES: &[(&str, &str, &str)] = &[
    (
        "transfer_duty",
        "nsw",
        "$52,237 plus $5.50 for every $100 over $1,290,000",
    ),
    ("transfer_duty", "vic", "$110,000 plus 6.5%"),
    ("transfer_duty", "qld", "$38,025 plus $5.75"),
    ("transfer_duty", "wa", "$28,453 + $5.15"),
    ("transfer_duty", "sa", "$21,330 plus $5.50"),
    ("transfer_duty", "tas", "$27 810 plus $4.50"),
    ("transfer_duty", "act", "$4.54 per $100"),
    ("transfer_duty", "nt", "0.06571441"),
    ("foreign_purchaser_surcharge", "nsw", "9%"),
    ("foreign_purchaser_surcharge", "nsw", "5%"),
    ("foreign_purchaser_surcharge", "vic", "8%"),
    ("foreign_purchaser_surcharge", "vic", "4%"),
    ("foreign_purchaser_surcharge", "qld", "8%"),
    ("foreign_purchaser_surcharge", "qld", "3%"),
    ("foreign_purchaser_surcharge", "wa", "7%"),
    ("foreign_purchaser_surcharge", "sa", "7%"),
    ("foreign_purchaser_surcharge", "tas", "8 percent"),
    ("foreign_purchaser_surcharge", "act", "0.75%"),
    ("foreign_purchaser_surcharge", "nt", "no land tax"),
    ("land_tax", "nsw", "$88,036 + 2%"),
    ("land_tax", "vic", "$31,650 plus 2.65%"),
    ("land_tax", "qld", "$150,000 plus 2.25 cents"),
    ("land_tax", "wa", "$186,550 + 0.0267"),
    ("land_tax", "sa", "$36,000 plus $2.40"),
    ("land_tax", "tas", "$1 737.50 plus 1.5%"),
    ("land_tax", "act", "$1,778"),
    ("first_home_owner_grant", "nsw", "$10,000"),
    ("first_home_owner_grant", "vic", "$10,000"),
    ("first_home_owner_grant", "qld", "$30,000"),
    ("first_home_owner_grant", "wa", "$10,000"),
    ("first_home_owner_grant", "sa", "$15,000"),
    ("first_home_owner_grant", "tas", "$20 000"),
    ("first_home_owner_grant", "act", "from 1 July 2019"),
    ("first_home_owner_grant", "nt", "$50 000"),
    (
        "motor_vehicle_duty",
        "nsw",
        "$1,350, plus $5 for every $100",
    ),
    ("motor_vehicle_duty", "nsw", "$3 per $100, or part thereof"),
    (
        "motor_vehicle_duty",
        "nsw",
        "$1,350 plus $5 per $100, or part thereof, of the vehicle's value over $45,000",
    ),
    ("motor_vehicle_duty", "vic", "$18.00 per $200"),
    (
        "motor_vehicle_duty",
        "qld",
        "More than $100,000—$6 for each $100",
    ),
    ("motor_vehicle_duty", "wa", "6,666.66"),
    ("motor_vehicle_duty", "sa", "$60 plus $4 for every $100"),
    ("motor_vehicle_duty", "tas", "$1 050 plus $11"),
    ("motor_vehicle_duty", "act", "$4.53 for every $100"),
    ("motor_vehicle_duty", "act", "$1,426.50 plus $5.62"),
    ("motor_vehicle_duty", "act", "$4,772.00 plus $8.00"),
    ("motor_vehicle_duty", "act", "$4.83 for every $100"),
    ("motor_vehicle_duty", "act", "$1,674.00 plus $6.44"),
    ("motor_vehicle_duty", "act", "$4,973.50 plus $8.00"),
    ("motor_vehicle_duty", "nt", "$3 for every $100"),
    (
        "motor_vehicle_duty",
        "nt",
        "the first $50 000 of the dutiable value of the electric motor vehicle is exempt from duty",
    ),
    ("insurance_duty", "nsw", "9 per cent of the premium"),
    ("insurance_duty", "vic", "10% of the gross premium"),
    (
        "insurance_duty",
        "vic",
        "is 7% from 1 July 2026 to 30 June 2027",
    ),
    ("insurance_duty", "qld", "9% of the premium paid"),
    ("insurance_duty", "qld", "10 cents"),
    ("insurance_duty", "wa", "rate of 10%"),
    ("insurance_duty", "sa", "11% of premium subject to duty"),
    ("insurance_duty", "tas", "10% of the amount of the premium"),
    ("insurance_duty", "act", "Insurance duty was abolished"),
    ("insurance_duty", "nt", "10% of the amount of the premium"),
];

#[test]
fn each_passage_states_the_figure_the_command_applies() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    for (kind, place, figure) in FIGURES {
        let quotes = of(&quotes_in(&catalogue, kind), place);
        assert!(
            quotes
                .iter()
                .any(|q| q["passage"].as_str().unwrap().contains(figure)),
            "{kind} {place}: {figure}"
        );
    }
}

/// Motor vehicle duty passages establish ACT Categories A to D across Tables 1
/// to 3 (both determinations), NT's Sch 1 cl 5(2) pinpoint and s 92A electric
/// exemption, and Service NSW's excess-over-$45,000 and part-$100 rule.
#[test]
fn motor_vehicle_duty_passages_cover_rules_and_exemptions() {
    let act =
        api::evaluate_coded("motor-vehicle-duty", &duty("motor_vehicle_duty", "act")).unwrap();
    assert_eq!(act["amounts"]["duty"], "1426.50");
    let act_quotes = act["source_quotes"].as_array().unwrap();
    assert_eq!(act_quotes.len(), 2);
    for (quote, table3_c, expected_cols) in [
        (
            &act_quotes[0],
            "$3,393.50 plus $8.00",
            &["Column 2", "Column 3", "Used eligible hybrid"][..],
        ),
        (&act_quotes[1], "$3,928.00 plus $8.00", &["Column 2"][..]),
    ] {
        assert!(
            quote["provision"]
                .as_str()
                .unwrap()
                .contains("s 6 Tables 1 to 3")
        );
        let text = quote["passage"].as_str().unwrap();
        for marker in [
            "Table 1",
            "Table 2",
            "Table 3",
            "Category AAA",
            "Category AA",
            "Category A",
            "Category B",
            "Non-rated vehicle",
            "Category C",
            "Category D",
            table3_c,
        ]
        .into_iter()
        .chain(expected_cols.iter().copied())
        {
            assert!(text.contains(marker), "ACT missing {marker}");
        }
    }
    assert!(
        act_quotes[0]["passage"]
            .as_str()
            .unwrap()
            .contains("$1,426.50 plus $5.62")
    );

    let mut nt_ev_50 = duty("motor_vehicle_duty", "nt");
    nt_ev_50["dutiable_value"] = json!("50000");
    nt_ev_50["electric"] = json!(true);
    let r50 = api::evaluate_coded("motor-vehicle-duty", &nt_ev_50).unwrap();
    assert_eq!(r50["amounts"]["duty"], "0.00");
    let mut nt_ev_60 = nt_ev_50.clone();
    nt_ev_60["dutiable_value"] = json!("60000");
    let r60 = api::evaluate_coded("motor-vehicle-duty", &nt_ev_60).unwrap();
    assert_eq!(r60["amounts"]["duty"], "300.00");
    let nt_quotes = r50["source_quotes"].as_array().unwrap();
    assert!(
        nt_quotes
            .iter()
            .any(|q| q["provision"] == "Stamp Duty Act 1978 (NT) Sch 1 cl 5(2)")
    );
    assert!(
        nt_quotes
            .iter()
            .all(|q| !q["provision"].as_str().unwrap().contains("Sch 1 cl 6"))
    );
    let s92a = nt_quotes
        .iter()
        .find(|q| q["provision"] == "Stamp Duty Act 1978 (NT) s 92A")
        .expect("missing NT s 92A quote")["passage"]
        .as_str()
        .unwrap();
    for marker in ["1 July 2022", "30 June 2027", "$50 000", "$1 500"] {
        assert!(s92a.contains(marker), "NT s 92A missing {marker}");
    }

    let mut nsw_45001 = duty("motor_vehicle_duty", "nsw");
    nsw_45001["dutiable_value"] = json!("45001");
    let r45 = api::evaluate_coded("motor-vehicle-duty", &nsw_45001).unwrap();
    assert_eq!(r45["amounts"]["duty"], "1355.00");
    let mut nsw_50000 = nsw_45001.clone();
    nsw_50000["dutiable_value"] = json!("50000");
    let r50_nsw = api::evaluate_coded("motor-vehicle-duty", &nsw_50000).unwrap();
    assert_eq!(r50_nsw["amounts"]["duty"], "1600.00");
    let nsw_text: String = r50_nsw["source_quotes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| q["passage"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(nsw_text.contains("or part thereof"));
    assert!(nsw_text.contains("of the vehicle's value over $45,000"));
}

/// The commands take lower-case codes only, so a result's passages cannot
/// belong to another spelling of its jurisdiction.
#[test]
fn an_upper_case_jurisdiction_is_refused() {
    for (command, kind, _, _) in COMMANDS {
        assert!(
            api::evaluate(command, &example(kind, "NSW")).is_err(),
            "{command}"
        );
    }
}

/// A command outside these, with a jurisdiction input, gains nothing.
#[test]
fn other_commands_gain_no_quotes() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["payroll_tax"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert!(facts.get("jurisdiction").is_some());
    let result = api::evaluate("payroll-tax", &facts).unwrap();
    assert!(result.get("source_quotes").is_none());
}

/// Each Victorian property tax's input, as tests/vic_property_taxes.rs gives
/// it, and the figure its passage must state as the command applies it: the
/// top VRLT rate, CIPT's 2035 first tax year after transition (nil in 2026) and
/// windfall gains tax's 62.5% band.
fn vic_tax(tax: &str) -> (Value, &'static str) {
    match tax {
        "vacant_residential_land" => (
            json!({"tax": tax, "capital_improved_value": "800000",
                   "residential_land": "vacant_home", "new_residential_concession": false,
                   "liable_previous_tax_year": true, "liable_tax_year_before": true,
                   "year": "2026", "scope_confirmed": true}),
            "3% of capital improved value",
        ),
        "commercial_industrial" => (
            json!({"tax": tax, "entry_date": "2024-07-01", "year": "2026", "scope_confirmed": true}),
            "2035 tax year",
        ),
        _ => (
            json!({"tax": tax, "parcels": [{"civ_before": "1000000", "civ_after": "1400000"}],
                   "year": "2026", "scope_confirmed": true}),
            "62.5% applies to uplifts",
        ),
    }
}

/// Each Victorian property tax's result carries its own tax's passages only,
/// and each states the figure the command applies.
#[test]
fn each_vic_property_tax_carries_its_own_quotes() {
    let quotes = catalogue_quotes("vic_property_taxes");
    for tax in TAXES {
        let own: Vec<Value> = quotes.iter().filter(|q| q["tax"] == tax).cloned().collect();
        let (input, figure) = vic_tax(tax);
        assert!(
            own.iter()
                .any(|q| q["passage"].as_str().unwrap().contains(figure)),
            "{tax}: {figure}"
        );
        for quote in &own {
            assert_eq!(quote["jurisdiction"], "vic", "{tax}");
            assert!(
                quote["url"]
                    .as_str()
                    .unwrap()
                    .starts_with("https://www.sro.vic.gov.au/")
            );
            assert_quote_meta(quote, tax);
        }
        let result = api::evaluate_coded("vic-property-taxes", &input).unwrap();
        if tax == "commercial_industrial" {
            assert!(
                own.iter()
                    .any(|q| q["passage"].as_str().unwrap().contains("10-year period")),
                "CIPT passage must explain the 10-year transition"
            );
            assert_eq!(result["amounts"]["cipt"], "0.00");
            assert_eq!(result["amounts"]["total"], "0.00");
        }
        assert_eq!(result["source_quotes"], Value::Array(own), "{tax}");
    }
    assert!(quotes.iter().all(|q| TAXES.iter().any(|t| q["tax"] == *t)));
}
