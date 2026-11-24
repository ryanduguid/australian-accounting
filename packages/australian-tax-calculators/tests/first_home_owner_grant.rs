// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The first home owner grant in the 8 states and territories against the
//! 26 cases of the site's grant page (ryanduguid.github.io
//! `_data/calculator_cases.json`, `first-home-owner-grant`, checked 23
//! November 2026, copied to `tests/data/first_home_owner_grant_page_cases.json`)
//! and each First Home Owner Grant Act as read on 23 November 2026
//! (docs/calculation-evidence.md#first-home-owner-grant).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const PAGE: &str = include_str!("data/first_home_owner_grant_page_cases.json");
const ALL: [&str; 4] = ["person", "citizen", "first_home", "residence"];

fn run(input: &Value) -> Value {
    api::evaluate("first-home-owner-grant", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("first-home-owner-grant", input)
        .unwrap_err()
        .code
}

/// The financial year holding an ISO date.
fn year_of(date: &str) -> String {
    let y: i32 = date[..4].parse().unwrap();
    let start = if &date[5..7] >= "07" { y } else { y - 1 };
    format!("{start}-{:02}", (start + 1) % 100)
}

/// A case with the 4 eligibility facts and the `extra` facts stated.
fn case(place: &str, date: &str, home: &str, value: u64, extra: &[&str]) -> Value {
    let facts: serde_json::Map<String, Value> = ALL
        .iter()
        .chain(extra)
        .map(|f| ((*f).to_owned(), json!(true)))
        .collect();
    json!({
        "jurisdiction": place, "contract_date": date, "home": home,
        "total_value": value.to_string(), "facts": facts,
        "year": year_of(date), "scope_confirmed": true,
    })
}

fn with(mut input: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            input.as_object_mut().unwrap().remove(key);
        } else {
            input[key] = value.clone();
        }
    }
    input
}

fn grant(input: &Value) -> (String, String) {
    let out = run(input);
    (
        out["amounts"]["grant"].as_str().unwrap().to_owned(),
        out["reason"].as_str().unwrap().to_owned(),
    )
}

/// The page's call as the command's input: the page's `AU-NSW` is `nsw`,
/// and its fact names are the command's shorter ones.
fn from_page(call: &Value) -> Value {
    let names = [
        ("person", "person"),
        ("citizen", "citizen"),
        ("first", "first_home"),
        ("residence", "residence"),
        ("contract", "building_contract"),
        ("nt", "nt_no_replacement"),
        ("tas", "tas_conditions"),
    ];
    let facts: serde_json::Map<String, Value> = names
        .iter()
        .map(|(page, ours)| ((*ours).to_owned(), call["facts"][page].clone()))
        .collect();
    let date = call["date"].as_str().unwrap();
    let mut input = json!({
        "jurisdiction": call["place"].as_str().unwrap()[3..].to_lowercase(),
        "contract_date": date, "home": call["home"], "total_value": call["value"],
        "facts": facts, "year": year_of(date), "scope_confirmed": true,
    });
    if let Some(north) = call.get("north") {
        input["north_of_26th"] = north.clone();
    }
    input
}

#[test]
fn the_page_cases_agree() {
    let page: Value = serde_json::from_str(PAGE).unwrap();
    let rows = page["rows"].as_array().unwrap();
    let mut agreed = 0;
    for row in rows {
        let input = from_page(&row["call"][0]);
        let name = row["case"].as_str().unwrap();
        if row.get("refused").is_some() {
            assert!(
                api::evaluate("first-home-owner-grant", &input).is_err(),
                "{name}"
            );
        } else {
            let (amount, reason) = grant(&input);
            let want = &row["result"];
            assert_eq!(amount, format!("{}.00", want["grant"]), "{name}");
            assert_eq!(reason, want["reason"].as_str().unwrap_or(""), "{name}");
        }
        agreed += 1;
    }
    assert_eq!(agreed, 26);
}

#[test]
fn the_page_refusals_have_their_reasons() {
    for (input, want) in [
        (
            case("wa", "2026-08-01", "bought", 500_000, &[]),
            "first_home_owner_grant.location_needed",
        ),
        (
            case("qld", "2025-10-01", "built", 120_000, &[]),
            "first_home_owner_grant.fact_needed",
        ),
        (
            case("nt", "2026-09-01", "bought", 600_000, &[]),
            "first_home_owner_grant.fact_needed",
        ),
        (
            case("nsw", "2025-06-30", "bought", 500_000, &[]),
            "period.unsupported",
        ),
        (
            case("nsw", "2026-11-23", "bought", 500_000, &[]),
            "first_home_owner_grant.date_uncovered",
        ),
        (
            case("nsw", "2026-08-01", "bought", 5_000, &[]),
            "first_home_owner_grant.below_grant",
        ),
        (
            with(
                case("nsw", "2026-08-01", "bought", 500_000, &[]),
                json!({"facts": {"person": true, "citizen": true, "first_home": true}}),
            ),
            "first_home_owner_grant.facts_not_stated",
        ),
    ] {
        assert_eq!(code(&input), want, "{input}");
    }
}

#[test]
fn the_messages_follow_the_page() {
    let refused = |input: Value| {
        api::evaluate_coded("first-home-owner-grant", &input)
            .unwrap_err()
            .message
    };
    assert_eq!(
        refused(case("nsw", "2026-11-23", "bought", 500_000, &[])),
        "Not computed: choose a contract date from 1 July 2025 to 22 November 2026, the day the grant rules were read."
    );
    assert_eq!(
        refused(case("nsw", "2026-08-01", "bought", 5_000, &[])),
        "Not computed: a value below the $10,000 grant; the New South Wales rule for that case was not confirmed."
    );
    assert_eq!(
        refused(case("qld", "2025-10-01", "built", 120_000, &[])),
        "Not computed: the Queensland grant for this home needs another fact stated: the building contract price is at least the grant."
    );
    assert_eq!(
        refused(with(
            case("nsw", "2026-08-01", "bought", 500_000, &[]),
            json!({"facts": {"citizen": true, "first_home": true}}),
        )),
        "Not computed: the grant needs every eligibility fact stated; not stated: every applicant is a person aged 18 or over; the residence requirement will be met."
    );
    assert_eq!(
        refused(case(
            "qld",
            "2025-10-01",
            "built",
            20_000,
            &["building_contract"]
        )),
        "Not computed: a total value below the $30,000 grant cannot hold a building contract price of at least the grant."
    );
}

/// Each jurisdiction's amount and cap at its edges, with the cap shown.
#[test]
fn caps_and_amounts() {
    let wa_south = |date: &str, value| {
        with(
            case("wa", date, "bought", value, &[]),
            json!({"north_of_26th": false}),
        )
    };
    for (input, want, cap) in [
        (
            case(
                "nsw",
                "2026-08-01",
                "built",
                750_000,
                &["building_contract"],
            ),
            "10000.00",
            "up to $750,000",
        ),
        (
            case(
                "nsw",
                "2026-08-01",
                "built",
                750_001,
                &["building_contract"],
            ),
            "0.00",
            "up to $750,000",
        ),
        (
            case(
                "vic",
                "2026-08-01",
                "built",
                750_000,
                &["building_contract"],
            ),
            "10000.00",
            "up to $750,000",
        ),
        (
            case(
                "qld",
                "2026-08-01",
                "built",
                749_999,
                &["building_contract"],
            ),
            "30000.00",
            "under $750,000",
        ),
        (
            case(
                "qld",
                "2026-08-01",
                "built",
                750_000,
                &["building_contract"],
            ),
            "0.00",
            "under $750,000",
        ),
        (
            wa_south("2026-05-06", 750_000),
            "10000.00",
            "up to $750,000",
        ),
        (wa_south("2026-05-06", 750_001), "0.00", "up to $750,000"),
        (
            wa_south("2026-05-07", 800_000),
            "10000.00",
            "up to $800,000",
        ),
        (wa_south("2026-05-07", 800_001), "0.00", "up to $800,000"),
        (
            with(
                wa_south("2026-05-06", 1_000_001),
                json!({"north_of_26th": true}),
            ),
            "0.00",
            "up to $1,000,000",
        ),
        (
            case(
                "sa",
                "2026-08-01",
                "built",
                3_000_000,
                &["building_contract"],
            ),
            "15000.00",
            "none",
        ),
        (
            case(
                "nt",
                "2026-08-01",
                "built",
                900_000,
                &["building_contract", "nt_no_replacement"],
            ),
            "50000.00",
            "none",
        ),
        (
            case("act", "2026-08-01", "built", 600_000, &[]),
            "0.00",
            "none",
        ),
    ] {
        let out = run(&input);
        assert_eq!(out["amounts"]["grant"], want, "{input}");
        assert_eq!(out["cap"], cap, "{input}");
    }
}

#[test]
fn the_result_names_its_rule_and_residence() {
    let out = run(&case("nsw", "2026-08-01", "bought", 600_000, &[]));
    assert_eq!(out["calculation"], "first_home_owner_grant");
    assert_eq!(out["period"], "2026-27");
    assert_eq!(out["jurisdiction"], "New South Wales");
    assert_eq!(out["contract_date"], "2026-08-01");
    assert_eq!(
        out["residence_requirement"],
        "each applicant occupies the home as their principal place of residence for at least 12 continuous months, starting within 12 months of the purchase or construction"
    );
    assert!(out["rule"].as_str().unwrap().starts_with("Revenue NSW"));
    let act = run(&case("act", "2025-07-01", "bought", 600_000, &[]));
    assert_eq!(
        act["reason"],
        "the Australian Capital Territory has paid none since 1 July 2019"
    );
    assert_eq!(act["residence_requirement"], "none, as no grant is paid");
    let tas = run(&case(
        "tas",
        "2026-07-01",
        "bought",
        400_000,
        &["tas_conditions"],
    ));
    assert!(tas["rule"].as_str().unwrap().contains("s 18M"));
}

/// Where an Act read on 23 November 2026 gives what the page does not.
#[test]
fn the_acts_where_the_page_refuses_or_pays_nil() {
    let nt = |date, home, value| case("nt", date, home, value, &["nt_no_replacement"]);
    // NT s 13(1)(a)(iii) and s 18(3D)(b): an established home from 1 October
    // 2024 to 30 September 2025; s 13(1)(a)(iv): a new home only after.
    assert_eq!(
        grant(&nt("2025-09-30", "established", 500_000)).0,
        "10000.00"
    );
    assert_eq!(
        grant(&nt("2025-10-01", "established", 500_000)),
        ("0.00".into(), "the grant is for a new home only".into())
    );
    assert_eq!(grant(&nt("2025-07-01", "bought", 500_000)).0, "50000.00");
    // The lesser of the consideration and the amount: Vic s 18(1A), WA
    // s 19(2CA), NT s 18(1).
    let lesser = "the grant cannot be more than the consideration";
    for input in [
        case("vic", "2026-01-01", "bought", 8_000, &[]),
        with(
            case("wa", "2026-01-01", "bought", 8_000, &[]),
            json!({"north_of_26th": true}),
        ),
        nt("2026-01-01", "bought", 8_000),
        nt("2025-08-01", "established", 8_000),
    ] {
        assert_eq!(grant(&input), ("8000.00".into(), lesser.into()), "{input}");
    }
    // Vic s 13(1A) and (7)(b): the cap is on the building work alone.
    assert_eq!(
        code(&case(
            "vic",
            "2026-01-01",
            "built",
            750_001,
            &["building_contract"]
        )),
        "first_home_owner_grant.value_basis"
    );
    assert_eq!(
        grant(&case("vic", "2026-01-01", "bought", 750_001, &[])),
        ("0.00".into(), "the value is over the Victoria cap".into())
    );
}

/// Qld s 5(6) to (8) and WA s 14AC test a value that can differ from the
/// consideration, which still limits the grant (Qld s 20(1), WA s 19(2CA)).
#[test]
fn the_cap_value_is_kept_apart_from_the_consideration() {
    let lesser = "the grant cannot be more than the consideration";
    let qld = |home, value, cap: u64| {
        with(
            case("qld", "2026-08-01", home, value, &["building_contract"]),
            json!({"cap_value": cap.to_string()}),
        )
    };
    let wa = |value, cap: u64| {
        with(
            case("wa", "2026-08-01", "bought", value, &[]),
            json!({"north_of_26th": false, "cap_value": cap.to_string()}),
        )
    };
    // A value above the price raises no grant above the consideration.
    assert_eq!(
        grant(&qld("bought", 20_000, 100_000)),
        ("20000.00".into(), lesser.into())
    );
    assert_eq!(
        grant(&wa(5_000, 100_000)),
        ("5000.00".into(), lesser.into())
    );
    // Qld s 5(7): a price under $750,000 on a home and land valued at
    // $750,000 or more.
    assert_eq!(
        grant(&qld("bought", 700_000, 750_000)),
        ("0.00".into(), "the value is $750,000 or more".into())
    );
    assert_eq!(grant(&qld("bought", 700_000, 749_999)).0, "30000.00");
    // A home bought is tested on the greater of the price and the value
    // (Qld s 5(6) and (7); WA s 14AC(2)).
    assert_eq!(grant(&qld("bought", 750_000, 600_000)).0, "0.00");
    assert_eq!(grant(&wa(800_001, 700_000)).0, "0.00");
    assert_eq!(
        grant(&wa(700_000, 800_001)),
        (
            "0.00".into(),
            "the value is over the Western Australia cap".into()
        )
    );
    assert_eq!(grant(&wa(700_000, 800_000)).0, "10000.00");
    // A home built is tested on the Act's total alone (Qld s 5(8), WA
    // s 14AC(3)): the contract and the land's value, not its price.
    assert_eq!(grant(&qld("built", 760_000, 740_000)).0, "30000.00");
    assert_eq!(grant(&qld("built", 700_000, 750_000)).0, "0.00");
    let wa_built = |value, cap: Option<u64>, extra: &[&str]| {
        let mut input = with(
            case("wa", "2026-08-01", "built", value, extra),
            json!({"north_of_26th": false}),
        );
        if let Some(cap) = cap {
            input["cap_value"] = json!(cap.to_string());
        }
        input
    };
    let contract = &["building_contract"];
    assert_eq!(
        grant(&wa_built(820_000, Some(800_000), contract)).0,
        "10000.00"
    );
    assert_eq!(grant(&wa_built(700_000, Some(800_001), contract)).0, "0.00");
    // A home built is paid only on the stated fact that the building
    // contract's consideration is at least the grant, so the land in
    // total_value never stands in for it (Qld s 20(1), WA s 19(2CA)).
    assert_eq!(
        code(&wa_built(500_000, None, &[])),
        "first_home_owner_grant.fact_needed"
    );
    assert_eq!(grant(&wa_built(500_000, None, contract)).0, "10000.00");
    // Without it, the value tested is total_value, as on the page.
    assert_eq!(
        grant(&case("qld", "2026-08-01", "bought", 20_000, &[])),
        ("20000.00".into(), lesser.into())
    );
}

#[test]
fn established_homes_get_nothing_without_their_facts() {
    for place in ["nsw", "vic", "qld", "sa", "tas"] {
        let out = run(&case(place, "2026-01-01", "established", 500_000, &[]));
        assert_eq!(out["amounts"]["grant"], "0.00", "{place}");
        assert_eq!(out["reason"], "the grant is for a new home only", "{place}");
    }
    assert_eq!(
        code(&case("nt", "2025-08-01", "established", 500_000, &[])),
        "first_home_owner_grant.fact_needed"
    );
}

#[test]
fn tasmania_needs_its_conditions_and_changes_on_1_july_2026() {
    assert_eq!(
        code(&case("tas", "2026-01-01", "bought", 400_000, &[])),
        "first_home_owner_grant.fact_needed"
    );
    let tas = |date| case("tas", date, "bought", 400_000, &["tas_conditions"]);
    assert_eq!(grant(&tas("2025-07-01")).0, "30000.00");
    assert_eq!(grant(&tas("2026-06-30")).0, "30000.00");
    assert_eq!(grant(&tas("2026-07-01")).0, "20000.00");
    assert_eq!(grant(&tas("2026-11-22")).0, "20000.00");
}

#[test]
fn refusals() {
    let base = case("nsw", "2026-08-01", "bought", 600_000, &[]);
    for (changes, want) in [
        (
            json!({"jurisdiction": "AU-NSW"}),
            "first_home_owner_grant.jurisdiction_invalid",
        ),
        (
            json!({"jurisdiction": 1}),
            "first_home_owner_grant.jurisdiction_invalid",
        ),
        (
            json!({"home": "renovated"}),
            "first_home_owner_grant.home_invalid",
        ),
        (
            json!({"contract_date": "2026-8-1"}),
            "first_home_owner_grant.date_invalid",
        ),
        (
            json!({"contract_date": "2026-02-30"}),
            "first_home_owner_grant.date_invalid",
        ),
        (
            json!({"contract_date": "2025-08-01"}),
            "first_home_owner_grant.date_invalid",
        ),
        (
            json!({"contract_date": 20_260_801}),
            "first_home_owner_grant.date_invalid",
        ),
        (json!({"year": "2024-25"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (
            json!({"facts": [true]}),
            "first_home_owner_grant.facts_invalid",
        ),
        (json!({"facts": {"person": "yes"}}), "input.flag_invalid"),
        (json!({"facts": {"spouse": true}}), "input.field_unknown"),
        (
            json!({"total_value": "0"}),
            "first_home_owner_grant.value_invalid",
        ),
        (
            json!({"total_value": "1000000001"}),
            "first_home_owner_grant.value_invalid",
        ),
        (
            json!({"total_value": "600000.50"}),
            "input.amount_not_whole_dollars",
        ),
        (json!({"total_value": "-1"}), "input.amount_invalid"),
        (json!({"north_of_26th": false}), "input.field_unknown"),
        (json!({"cap_value": "700000"}), "input.field_unknown"),
        (json!({"total_value": "remove"}), "input.missing_field"),
        (json!({"suburb": "Parramatta"}), "input.field_unknown"),
    ] {
        assert_eq!(
            code(&with(base.clone(), changes.clone())),
            want,
            "{changes}"
        );
    }
    assert_eq!(
        code(&with(
            case("wa", "2026-08-01", "bought", 600_000, &[]),
            json!({"north_of_26th": "north"}),
        )),
        "input.flag_invalid"
    );
    let qld = case("qld", "2026-08-01", "bought", 600_000, &[]);
    for (cap, want) in [
        ("0", "first_home_owner_grant.value_invalid"),
        ("1000000001", "first_home_owner_grant.value_invalid"),
        ("700000.50", "input.amount_not_whole_dollars"),
        ("-1", "input.amount_invalid"),
    ] {
        assert_eq!(
            code(&with(qld.clone(), json!({"cap_value": cap}))),
            want,
            "{cap}"
        );
    }
    assert_eq!(
        code(&case("nsw", "2026-08-01", "built", 600_000, &[])),
        "first_home_owner_grant.fact_needed"
    );
}

/// The grant by the rules as the spec states them, typed apart from the
/// command's tables, or `None` where the command refuses: every jurisdiction,
/// home and edge date, at each amount and cap and either side of them.
fn oracle(place: &str, date: &str, home: &str, value: u64, north: bool) -> Option<u64> {
    if !("2025-07-01"..="2026-11-22").contains(&date) {
        return None;
    }
    let nt_window = place == "nt" && date <= "2025-09-30";
    if place == "act" || (home == "established" && !nt_window) {
        return Some(0);
    }
    let amount = match place {
        "qld" => 30_000,
        "tas" if date <= "2026-06-30" => 30_000,
        "sa" => 15_000,
        "tas" => 20_000,
        "nt" if home == "established" => 10_000,
        "nt" => 50_000,
        _ => 10_000,
    };
    let cap = match place {
        "nsw" if home == "bought" => Some(600_000),
        "nsw" | "vic" => Some(750_000),
        "qld" => Some(749_999),
        "wa" if north => Some(1_000_000),
        "wa" if date >= "2026-05-07" => Some(800_000),
        "wa" => Some(750_000),
        _ => None,
    };
    if cap.is_some_and(|cap| value > cap) {
        return (place != "vic" || home != "built").then_some(0);
    }
    if value < amount {
        return (home != "built" && place != "nsw").then_some(value);
    }
    Some(amount)
}

#[test]
fn every_edge_agrees_with_the_oracle() {
    let dates = [
        "2025-06-30",
        "2025-07-01",
        "2025-09-30",
        "2025-10-01",
        "2026-05-06",
        "2026-05-07",
        "2026-06-30",
        "2026-07-01",
        "2026-11-22",
        "2026-11-23",
    ];
    let mut values = vec![1, 1_000_000_000];
    for edge in [
        8_000, 10_000, 15_000, 20_000, 30_000, 50_000, 600_000, 750_000, 800_000, 1_000_000,
    ] {
        values.extend([edge - 1, edge, edge + 1]);
    }
    let mut checked = 0;
    for place in ["nsw", "vic", "qld", "wa", "sa", "tas", "act", "nt"] {
        for home in ["bought", "built", "established"] {
            for date in dates {
                for &value in &values {
                    for north in [false, true] {
                        if north && place != "wa" {
                            continue;
                        }
                        let facts = ["building_contract", "nt_no_replacement", "tas_conditions"];
                        let mut input = case(place, date, home, value, &facts);
                        if place == "wa" {
                            input["north_of_26th"] = json!(north);
                        }
                        let got = api::evaluate("first-home-owner-grant", &input)
                            .ok()
                            .map(|out| out["amounts"]["grant"].as_str().unwrap().to_owned());
                        let want = oracle(place, date, home, value, north)
                            .map(|grant| format!("{grant}.00"));
                        assert_eq!(got, want, "{input}");
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, 9 * 3 * 10 * 32);
}

/// Each residence requirement as its source states it: principal place of
/// residence, how long and from when.
#[test]
fn residence_requirements_are_stated_in_full() {
    for (place, period, start) in [
        (
            "nsw",
            "at least 12 continuous months",
            "within 12 months of the purchase or construction",
        ),
        (
            "vic",
            "at least 12 months",
            "within 12 months of settlement or of the build finishing",
        ),
        (
            "qld",
            "a continuous period of at least 6 months",
            "within 1 year after completion",
        ),
        (
            "wa",
            "a continuous period of at least 6 months",
            "within 12 months of completion",
        ),
        (
            "sa",
            "a continuous period of at least 6 months",
            "within 12 months after completion",
        ),
        (
            "tas",
            "a continuous period of at least 6 months",
            "within 12 months of completion",
        ),
        (
            "nt",
            "a continuous period of at least 12 months",
            "within 12 months after completion",
        ),
    ] {
        let mut input = case(
            place,
            "2026-08-01",
            "bought",
            500_000,
            &["nt_no_replacement", "tas_conditions"],
        );
        if place == "wa" {
            input["north_of_26th"] = json!(false);
        }
        let text = run(&input)["residence_requirement"]
            .as_str()
            .unwrap()
            .to_owned();
        for part in ["as their principal place of residence", period, start] {
            assert!(text.contains(part), "{place}: {text}");
        }
    }
}

/// For a home built, every Act read on 24 November 2026 makes the
/// consideration the building work alone (Qld schedule 'consideration' (b),
/// WA s 14AB(b), Vic, SA and Tas s 13(7)(b), ACT s 13(6)(b), NT s 13(8)(b)),
/// so the help must not call the land and contract total the consideration
/// that limits the grant.
#[test]
fn total_value_help_keeps_the_land_out_of_a_built_homes_consideration() {
    let units = australian_tax_calculators::metadata::input_units("first_home_owner_grant");
    let total = units
        .iter()
        .find(|(name, _)| *name == "total_value")
        .unwrap()
        .1;
    assert!(
        !total.contains("the consideration, which limits the grant"),
        "{total}"
    );
    assert!(
        total.contains("the consideration is the building contract alone"),
        "{total}"
    );
}
