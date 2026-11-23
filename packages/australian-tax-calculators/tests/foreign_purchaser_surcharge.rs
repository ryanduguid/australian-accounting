// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Foreign purchaser duty and land tax surcharges against the site's foreign
//! surcharge calculator (ryanduguid.github.io at dcc7ecbe: its cases, its Node
//! tests' and oracle's figures, a grid of every jurisdiction, use and kind of
//! buyer at each edge and seeded values, run through the page's own
//! `foreignSurcharge` into `data/foreign_surcharge_site_cases.json`, with the
//! page's verdicts as the established `foreign_purchaser` and `foreign_owner`),
//! and against the Acts and revenue offices' published figures. Where the page
//! differs from an Act, the Act wins, as `land-tax` does: Queensland's
//! surcharge counts from $349,999 and Tasmania's is rounded to 10 cents. Every
//! source was read on 24 November 2026
//! (docs/calculation-evidence.md#foreign-purchaser-surcharge). Every buyer here
//! is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(
    place: &str,
    property: &str,
    value: &str,
    purchaser: bool,
    land: &str,
    owner: bool,
) -> Value {
    json!({
        "jurisdiction": place, "property_use": property, "dutiable_value": value,
        "foreign_purchaser": purchaser, "land_value": land, "foreign_owner": owner,
        "owner": "individual", "year": "2026-27", "scope_confirmed": true,
    })
}

fn with(mut v: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value.is_null() {
            v.as_object_mut().unwrap().remove(key);
        } else {
            v[key] = value.clone();
        }
    }
    v
}

fn run(input: &Value) -> Value {
    let result = api::evaluate("foreign-purchaser-surcharge", input).unwrap();
    let envelope = json!({"command": "foreign-purchaser-surcharge", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

/// The duty surcharge and the land tax surcharge.
fn both(input: &Value) -> [String; 2] {
    let result = run(input);
    ["duty_surcharge", "land_tax_surcharge"]
        .map(|key| result["amounts"][key].as_str().unwrap().to_string())
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("foreign-purchaser-surcharge", input)
        .unwrap_err()
        .code
}

fn cents(text: &str) -> i64 {
    let (whole, part) = text.split_once('.').unwrap();
    whole.parse::<i64>().unwrap() * 100 + part.parse::<i64>().unwrap()
}

fn shown(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

/// How a row compares with the site.
#[derive(Debug, PartialEq)]
enum Outcome {
    Same,
    /// Queensland from $349,999 or Tasmania's s 7.
    Moved,
    /// A Victorian trust that is a foreign owner (s 46IA).
    Refused,
    /// A fact the command does not take: an exemption or a visa the page refuses.
    NotTaken,
}

/// The site's land tax surcharge moved by the Act, in cents.
fn land_by_the_act(row: &Value, site: i64) -> i64 {
    let value: i64 = row["land_value"].as_str().unwrap().parse().unwrap();
    match row["jurisdiction"].as_str().unwrap() {
        // From $350,000 the Act counts from $349,999; the page's figure is
        // nil at $350,000 itself.
        "qld" if row["foreign_owner"] == true && value >= 350_000 => site + 3,
        "tas" => {
            let rest = site % 10;
            site - rest + if rest > 5 { 10 } else { 0 }
        }
        _ => site,
    }
}

fn compared(row: &Value) -> Outcome {
    if row.get("error").is_some() {
        return Outcome::NotTaken;
    }
    let mut given = json!({"year": "2026-27", "scope_confirmed": true});
    for key in [
        "jurisdiction",
        "property_use",
        "dutiable_value",
        "foreign_purchaser",
        "land_value",
        "foreign_owner",
        "owner",
    ] {
        given[key] = row[key].clone();
    }
    if row["jurisdiction"] == "vic" && row["owner"] == "trust" && row["foreign_owner"] == true {
        assert_eq!(
            code(&given),
            "foreign_purchaser_surcharge.absentee_trust_unsupported",
            "{row}"
        );
        return Outcome::Refused;
    }
    let got = both(&given);
    let site = cents(row["land"].as_str().unwrap());
    let land = land_by_the_act(row, site);
    assert_eq!(
        got,
        [row["duty"].as_str().unwrap().to_string(), shown(land)],
        "{row}"
    );
    if land == site {
        Outcome::Same
    } else {
        Outcome::Moved
    }
}

/// Every row of the site's fixture agrees, or differs only where an Act
/// differs from the page.
#[test]
fn the_site_cases_agree_except_where_an_act_differs() {
    let data: Value =
        serde_json::from_str(include_str!("data/foreign_surcharge_site_cases.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    let outcomes: Vec<(Outcome, &Value)> = rows.iter().map(|row| (compared(row), row)).collect();
    let count = |want: Outcome| outcomes.iter().filter(|(o, _)| *o == want).count();
    let counts = [
        Outcome::Same,
        Outcome::Moved,
        Outcome::Refused,
        Outcome::NotTaken,
    ]
    .map(count);
    eprintln!(
        "foreign-purchaser-surcharge site parity: {} rows, {} identical, {} moved by the Act, {} refused, {} not taken",
        rows.len(),
        counts[0],
        counts[1],
        counts[2],
        counts[3]
    );
    assert_eq!(rows.len(), 980);
    assert_eq!(counts, [871, 77, 30, 2]);
    // The page's own cases: 19 worked here; its 4 FIRB fee cases are a
    // federal fee the command does not work.
    assert_eq!(data["firb_cases_out_of_scope"].as_array().unwrap().len(), 4);
    let page: Vec<&(Outcome, &Value)> = outcomes
        .iter()
        .filter(|(_, row)| row["case"].as_str().unwrap().starts_with("cases: "))
        .collect();
    assert_eq!(page.len(), 19);
    let on_page = [
        Outcome::Same,
        Outcome::Moved,
        Outcome::Refused,
        Outcome::NotTaken,
    ]
    .map(|want| page.iter().filter(|(o, _)| *o == want).count());
    // Queensland's company moves to $1,500.03; SRO Victoria's Clara, a fixed
    // trust, is refused; the exemption and the partner visa are not taken.
    assert_eq!(on_page, [15, 1, 1, 2]);
}

/// The revenue offices' examples.
#[test]
fn the_published_examples() {
    let residential = |place, value, land| input(place, "residential", value, true, land, true);
    // Revenue NSW: 9% of $1,000,000 and 5% of $1,200,000.
    assert_eq!(
        both(&residential("nsw", "1000000", "1200000")),
        ["90000.00", "60000.00"]
    );
    // SRO Victoria: Emily, $16,000 on $200,000, and 4% of $400,000 of land.
    assert_eq!(
        both(&residential("vic", "200000", "400000")),
        ["16000.00", "16000.00"]
    );
    // Queensland: 3.0c for each $1 more than $349,999; the QRO's $1,500 is 3 cents less.
    assert_eq!(
        both(&residential("qld", "400000", "400000")),
        ["32000.00", "1500.03"]
    );
    // The DTF's Simon: 7% of his $200,000 interest; no land tax surcharge.
    assert_eq!(
        both(&residential("wa", "200000", "200000")),
        ["14000.00", "0.00"]
    );
    // RevenueSA: 7% of a 20% interest valued at $100,000.
    assert_eq!(
        both(&residential("sa", "100000", "500000")),
        ["7000.00", "0.00"]
    );
    // Tasmania: 8%, and 1.5% on primary production property.
    assert_eq!(
        both(&residential("tas", "400000", "400000")),
        ["32000.00", "8000.00"]
    );
    let primary = input("tas", "primary_production", "400000", true, "400000", true);
    assert_eq!(both(&primary), ["6000.00", "0.00"]);
    // The ACT: no duty surcharge; 0.75% of $600,000 of AUV.
    assert_eq!(
        both(&residential("act", "600000", "600000")),
        ["0.00", "4500.00"]
    );
    assert_eq!(
        both(&residential("nt", "600000", "600000")),
        ["0.00", "0.00"]
    );
}

/// Each threshold and rounding edge by hand.
#[test]
fn the_edges() {
    let land =
        |place, value| both(&input(place, "residential", "1", false, value, true))[1].clone();
    // Victoria: the absentee rates less the general rates, 4% of the whole value from $50,000.
    assert_eq!(
        [land("vic", "49999"), land("vic", "50000")],
        ["0.00", "2000.00"]
    );
    // Queensland from $350,000: $0.03 on $350,000, as (350,000 - 349,999) x 3c.
    assert_eq!(
        [
            land("qld", "349999"),
            land("qld", "350000"),
            land("qld", "350001")
        ],
        ["0.00", "0.03", "0.06"]
    );
    // Tasmania's s 7: $8,000.02 down, $8,000.06 up, $8,000.10 kept; 5 cents exactly goes down.
    for (value, want) in [
        ("400001", "8000.00"),
        ("400003", "8000.10"),
        ("400005", "8000.10"),
        ("2", "0.00"),
    ] {
        assert_eq!(land("tas", value), want, "{value}");
    }
    assert_eq!(land("tas", "0"), "0.00");
    // The ACT's 0.75% of $7 is 5.25 cents.
    assert_eq!(land("act", "7"), "0.05");
}

/// Which property and owner each surcharge applies to.
#[test]
fn what_each_surcharge_applies_to() {
    // Not foreign: nil, whatever the values.
    assert_eq!(
        both(&input(
            "nsw",
            "residential",
            "1000000",
            false,
            "1000000",
            false
        )),
        ["0.00", "0.00"]
    );
    // Commercial property in New South Wales bears neither surcharge; Victoria's
    // and Queensland's land tax surcharges apply to all land.
    assert_eq!(
        both(&input("nsw", "other", "1000000", true, "1000000", true)),
        ["0.00", "0.00"]
    );
    assert_eq!(
        both(&input("vic", "other", "1000000", true, "1000000", true)),
        ["0.00", "40000.00"]
    );
    assert_eq!(
        both(&input(
            "qld",
            "primary_production",
            "1000000",
            true,
            "1000000",
            true
        )),
        ["0.00", "19500.03"]
    );
}

/// Each case changes a valid input and names the refusal.
const REFUSED: [(&str, &str, &str); 11] = [
    (
        "place",
        r#"{"jurisdiction": "jervis-bay"}"#,
        "foreign_purchaser_surcharge.jurisdiction_invalid",
    ),
    (
        "use",
        r#"{"property_use": "farm"}"#,
        "foreign_purchaser_surcharge.use_invalid",
    ),
    (
        "owner",
        r#"{"owner": "partnership"}"#,
        "foreign_purchaser_surcharge.owner_invalid",
    ),
    (
        "vic trust",
        r#"{"jurisdiction": "vic", "owner": "trust"}"#,
        "foreign_purchaser_surcharge.absentee_trust_unsupported",
    ),
    ("year", r#"{"year": "2025-26"}"#, "period.unsupported"),
    (
        "cents",
        r#"{"dutiable_value": "500000.50"}"#,
        "input.amount_not_whole_dollars",
    ),
    (
        "land cents",
        r#"{"land_value": "1.50"}"#,
        "input.amount_not_whole_dollars",
    ),
    (
        "nil",
        r#"{"dutiable_value": "0"}"#,
        "foreign_purchaser_surcharge.dutiable_value_nil",
    ),
    ("flag", r#"{"foreign_owner": "yes"}"#, "input.flag_invalid"),
    (
        "scope",
        r#"{"scope_confirmed": false}"#,
        "scope.not_confirmed",
    ),
    ("unknown", r#"{"exemption": true}"#, "input.field_unknown"),
];

#[test]
fn refuses_what_it_does_not_hold() {
    let base = input("nsw", "residential", "500000", true, "500000", true);
    for (why, changes, want) in REFUSED {
        let changed = with(base.clone(), serde_json::from_str(changes).unwrap());
        assert_eq!(code(&changed), want, "{why}");
    }
    let missing = with(base.clone(), json!({"owner": null}));
    assert_eq!(code(&missing), "input.missing_field");
    // A Victorian trust that is not an absentee owner, and a trust elsewhere, are worked.
    let resident = with(
        input("vic", "residential", "500000", true, "500000", false),
        json!({"owner": "trust"}),
    );
    assert_eq!(both(&resident), ["40000.00", "0.00"]);
    let qld = with(
        input("qld", "residential", "500000", true, "500000", true),
        json!({"owner": "trust"}),
    );
    assert_eq!(both(&qld), ["40000.00", "4500.03"]);
}

/// Each step's working shows its formula, the operands and the exact value.
#[test]
fn the_steps_show_their_working() {
    let working = |input: Value| {
        let doc = api::evaluate(
            "derive",
            &json!({"command": "foreign-purchaser-surcharge", "input": input}),
        )
        .unwrap();
        doc["steps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["working"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        working(input("qld", "residential", "400000", true, "400000", true)),
        [
            "dutiable_value x duty_surcharge_rate = 400000 x 0.08 = 32000.00",
            "(land_value - land_tax_surcharge_threshold) x land_tax_surcharge_rate = (400000 - 349999) x 0.03 = 1500.03",
        ]
    );
    assert_eq!(
        working(input("tas", "residential", "400001", true, "400003", true))[1],
        "dollars_up(land_value x land_tax_surcharge_rate x 10 - 0.5) / 10 = dollars_up(400003 x 0.02 x 10 - 0.5) / 10 = 8000.1"
    );
}
