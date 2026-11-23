// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Transfer duty in the 8 jurisdictions against the site's transfer duty
//! calculator (ryanduguid.github.io at dcc7ecbe: its 21 cases, its Node tests'
//! figures, the oracle's worked figures, every band and first home threshold
//! edge and seeded values, run through the page's own `transferDuty` and
//! `firstHomeDuty` into `data/transfer_duty_site_cases.json`), and against the
//! Acts and revenue offices' published figures. Where the page differs from an
//! Act, the Act wins: Victoria's duty is chargeable to the nearest whole dollar
//! (Duties Act 2000 (Vic) s 28(1)). Every source was read on 24 November 2026
//! (docs/calculation-evidence.md#transfer-duty). Every buyer here is
//! fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(place: &str, value: &str, buyer: &str) -> Value {
    json!({
        "jurisdiction": place, "dutiable_value": value, "buyer": buyer,
        "residential": true, "year": "2026-27", "scope_confirmed": true,
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
    let result = api::evaluate("transfer-duty", input).unwrap();
    let envelope = json!({"command": "transfer-duty", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// The duty and the general duty for `buyer`.
fn duty(place: &str, value: &str, buyer: &str) -> [String; 2] {
    let result = run(&input(place, value, buyer));
    ["duty", "general_duty"].map(|key| amount(&result, key).to_string())
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("transfer-duty", input)
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

/// Victoria's s 28(1): cents to the nearest whole dollar, 50 cents down.
fn section_28(cents: i64) -> i64 {
    let rest = cents % 100;
    cents - rest + if rest > 50 { 100 } else { 0 }
}

/// The site's duty and general duty moved by Victoria's s 28(1), in cents,
/// worked from the fixture alone: s 57JA(3)'s share is taken of the rounded
/// general duty (B), to the cent half up.
fn expected(row: &Value) -> [i64; 2] {
    let site = ["duty", "general"].map(|key| cents(row[key].as_str().unwrap()));
    if row["jurisdiction"] != "vic" {
        return site;
    }
    let value: i64 = row["dutiable_value"].as_str().unwrap().parse().unwrap();
    let general = section_28(site[1]);
    let duty = match row["buyer"].as_str().unwrap() {
        "general" => general,
        _ if value <= 600_000 => 0,
        _ if value <= 750_000 => (2 * general * (value - 600_000) + 150_000) / 300_000,
        _ => general,
    };
    [duty, general]
}

/// Whether the row matches the site, after checking the worksheet against the
/// expected figures.
fn checked(row: &Value) -> bool {
    let [place, value, buyer] =
        ["jurisdiction", "dutiable_value", "buyer"].map(|key| row[key].as_str().unwrap());
    if let Some(message) = row["error"].as_str() {
        assert_eq!(message, "Enter a whole number between 1 and 1000000000.");
        assert_eq!(
            code(&input(place, value, buyer)),
            "input.amount_not_whole_dollars"
        );
        return true;
    }
    let result = run(&input(place, value, buyer));
    let want = expected(row);
    assert_eq!(
        [amount(&result, "duty"), amount(&result, "general_duty")],
        want.map(shown).each_ref().map(String::as_str),
        "{row}"
    );
    if buyer != "general" {
        assert_eq!(
            amount(&result, "concession"),
            shown(want[1] - want[0]),
            "{row}"
        );
    }
    want == ["duty", "general"].map(|key| cents(row[key].as_str().unwrap()))
}

/// Every row of the site's fixture agrees, or differs only by Victoria's
/// rounding to the dollar.
#[test]
fn the_site_cases_agree_except_where_victoria_rounds() {
    let data: Value =
        serde_json::from_str(include_str!("data/transfer_duty_site_cases.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    let moved: Vec<&Value> = rows.iter().filter(|row| !checked(row)).collect();
    eprintln!(
        "transfer-duty site parity: {} rows, {} identical, {} moved by Duties Act 2000 (Vic) s 28(1)",
        rows.len(),
        rows.len() - moved.len(),
        moved.len()
    );
    assert_eq!(rows.len(), 925);
    assert_eq!(moved.len(), 74);
    assert!(moved.iter().all(|row| row["jurisdiction"] == "vic"));
    // The page's own figures that move: its Node tests' $25,001 and the
    // general duty beside its first home figures on $600,001 and $750,001.
    let page: Vec<String> = moved
        .iter()
        .filter(|row| row["case"].as_str().unwrap().starts_with("node test"))
        .map(|row| {
            format!(
                "{} {}",
                row["dutiable_value"].as_str().unwrap(),
                row["buyer"].as_str().unwrap()
            )
        })
        .collect();
    assert_eq!(
        page,
        [
            "25001 general",
            "600001 first_home_established",
            "750001 first_home_established"
        ]
    );
    // The page's 21 cases and the oracle's 44 worked figures all agree.
    let named = |rows: &[&Value], prefix: &str| {
        rows.iter()
            .filter(|row| row["case"].as_str().unwrap().contains(prefix))
            .count()
    };
    let all: Vec<&Value> = rows.iter().collect();
    assert_eq!(named(&all, "cases: "), 21);
    assert_eq!(named(&all, "oracle: "), 44);
    assert_eq!(named(&moved, "cases: ") + named(&moved, "oracle: "), 0);
}

/// Revenue NSW's 2026-27 examples, s 273's minimum, s 32A's premium rate on
/// residential land only, and the NSW Government's first home figures.
#[test]
fn new_south_wales_agrees_with_revenue_nsw() {
    for (value, want) in [
        ("1350000", "55537.00"),
        ("450000", "14437.00"),
        ("4000000", "203237.00"),
        ("1600", "20.00"),
        ("1601", "21.25"),
        ("18000", "225.00"),
        ("3870000", "194137.00"),
    ] {
        assert_eq!(duty("nsw", value, "general")[0], want, "{value}");
    }
    // Land that is not residential stays at $5.50 above $3,870,000:
    // $52,237 + 27,100 x $5.50.
    let other = with(
        input("nsw", "4000000", "general"),
        json!({"residential": false}),
    );
    assert_eq!(amount(&run(&other), "duty"), "201287.00");
    for (value, want) in [
        ("800000", "0.00"),
        ("800001", "20.00"),
        ("850000", "9796.75"),
        ("900000", "19593.50"),
        ("950000", "29390.25"),
        ("990000", "37227.65"),
        ("999000", "38991.06"),
        ("999999", "39186.85"),
        ("1000000", "39187.00"),
    ] {
        for buyer in ["first_home_established", "first_home_new"] {
            assert_eq!(duty("nsw", value, buyer)[0], want, "{value} {buyer}");
        }
    }
    let result = run(&input("nsw", "900000", "first_home_new"));
    assert_eq!(amount(&result, "duty_on_800000"), "30187.00");
    assert_eq!(amount(&result, "concession"), "15093.50");
}

/// The State Revenue Office's examples, s 28(1)'s rounding and s 57JA.
#[test]
fn victoria_rounds_to_the_dollar() {
    for (value, want) in [
        ("550000", "28070.00"),
        ("400000", "19070.00"),
        ("25001", "350.00"),
        // $350.48 and $350.504; $3.50, $10.50 and $52,805.50 go down.
        ("25020", "350.00"),
        ("25021", "351.00"),
        ("250", "3.00"),
        ("750", "10.00"),
        ("960100", "52805.00"),
        ("960101", "52806.00"),
        ("1", "0.00"),
        ("1000000", "55000.00"),
        ("2000001", "110000.00"),
        ("2000008", "110001.00"),
        ("2000007", "110000.00"),
    ] {
        assert_eq!(duty("vic", value, "general")[0], want, "{value}");
    }
    for (value, want) in [
        ("600000", "0.00"),
        ("600001", "0.21"),
        ("700000", "24713.33"),
        ("750000", "40070.00"),
        ("750001", "40070.00"),
    ] {
        assert_eq!(duty("vic", value, "first_home_new")[0], want, "{value}");
    }
}

/// The Queensland Revenue Office's examples; s 91 matches its home concession
/// table at each band's top; s 92A has no cap.
#[test]
fn queensland_first_homes() {
    assert_eq!(duty("qld", "850000", "general")[0], "31275.00");
    for (value, want) in [
        ("350000", "0.00"),
        ("700000", "0.00"),
        ("709999", "450.00"),
        ("710000", "2185.00"),
        ("730000", "6555.00"),
        ("799999", "20115.00"),
        ("800000", "21850.00"),
        ("850000", "24100.00"),
    ] {
        assert_eq!(
            duty("qld", value, "first_home_established")[0],
            want,
            "{value}"
        );
    }
    // The QRO's home concession table: $3,500 at $350,000, $10,150 at
    // $540,000 and $30,850 at $1,000,000, reached here through s 91(5).
    for (value, want) in [("540000", "10150.00"), ("1000000", "30850.00")] {
        let result = run(&input("qld", value, "first_home_established"));
        assert_eq!(amount(&result, "home_concession_duty"), want);
        assert_eq!(amount(&result, "duty_on_350000"), "10675.00");
    }
    let small = run(&input("qld", "350000", "first_home_established"));
    assert_eq!(amount(&small, "home_concession_duty"), "3500.00");
    assert_eq!(
        duty("qld", "3000000", "first_home_new"),
        ["0.00", "153025.00"].map(String::from)
    );
}

/// Western Australia, South Australia, Tasmania, the ACT and the Northern
/// Territory, by hand from each scale and concession.
#[test]
fn the_other_jurisdictions() {
    let general = |place, value| duty(place, value, "general")[0].clone();
    assert_eq!(general("wa", "800000"), "32315.50");
    assert_eq!(general("wa", "725001"), "28458.15");
    assert_eq!(duty("wa", "700000", "first_home_new")[0], "16150.00");
    assert_eq!(duty("wa", "800001", "first_home_new")[0], "32320.65");
    assert_eq!(general("sa", "600000"), "26830.00");
    assert_eq!(general("sa", "300000"), "11330.00");
    assert_eq!(
        duty("sa", "600000", "first_home_established")[0],
        "26830.00"
    );
    assert_eq!(duty("sa", "5000000", "first_home_new")[0], "0.00");
    assert_eq!(general("tas", "500000"), "18247.50");
    assert_eq!(general("tas", "3000"), "50.00");
    assert_eq!(
        duty("tas", "500000", "first_home_established")[0],
        "18247.50"
    );
    assert_eq!(general("act", "650000"), "17880.00");
    assert_eq!(general("act", "1500000"), "68100.00");
    assert_eq!(duty("act", "3000000", "first_home_established")[0], "0.00");
    // 0.06571441 x 500^2 + 15 x 500 = 23,928.6025; 525^2 gives 25,987.53425625.
    assert_eq!(general("nt", "500000"), "23928.60");
    assert_eq!(general("nt", "525000"), "25987.53");
    assert_eq!(general("nt", "525001"), "25987.55");
    assert_eq!(general("nt", "2999999"), "148499.95");
    assert_eq!(general("nt", "3000000"), "172500.00");
    assert_eq!(general("nt", "5000000"), "297500.00");
    assert_eq!(duty("nt", "500000", "first_home_new")[0], "23928.60");
}

/// Each case changes a valid input and names the refusal.
const REFUSED: [(&str, &str, &str); 13] = [
    (
        "place",
        r#"{"jurisdiction": "jervis-bay"}"#,
        "transfer_duty.jurisdiction_invalid",
    ),
    (
        "buyer",
        r#"{"buyer": "first_home"}"#,
        "transfer_duty.buyer_invalid",
    ),
    ("year", r#"{"year": "2025-26"}"#, "period.unsupported"),
    (
        "cents",
        r#"{"dutiable_value": "500000.50"}"#,
        "input.amount_not_whole_dollars",
    ),
    (
        "nil",
        r#"{"dutiable_value": "0"}"#,
        "transfer_duty.dutiable_value_nil",
    ),
    (
        "negative",
        r#"{"dutiable_value": "-1"}"#,
        "input.amount_invalid",
    ),
    ("flag", r#"{"residential": "yes"}"#, "input.flag_invalid"),
    (
        "scope",
        r#"{"scope_confirmed": false}"#,
        "scope.not_confirmed",
    ),
    ("missing", r#"{"residential": null}"#, "input.missing_field"),
    ("unknown", r#"{"surcharge": true}"#, "input.field_unknown"),
    (
        "first home on other land",
        r#"{"jurisdiction": "vic", "buyer": "first_home_new", "residential": false}"#,
        "transfer_duty.first_home_not_residential",
    ),
    (
        "sa qualifying land",
        r#"{"jurisdiction": "sa", "residential": false}"#,
        "transfer_duty.non_residential_unsupported",
    ),
    (
        "act commercial rates",
        r#"{"jurisdiction": "act", "residential": false}"#,
        "transfer_duty.non_residential_unsupported",
    ),
];

#[test]
fn refuses_what_it_does_not_hold() {
    for (why, changes, want) in REFUSED {
        let changed = with(
            input("nsw", "500000", "general"),
            serde_json::from_str(changes).unwrap(),
        );
        assert_eq!(code(&changed), want, "{why}");
    }
    // Land that is not residential is worked at the general rate elsewhere.
    for place in ["vic", "qld", "wa", "tas", "nt"] {
        let other = with(
            input(place, "500000", "general"),
            json!({"residential": false}),
        );
        let residential = run(&input(place, "500000", "general"));
        assert_eq!(run(&other)["amounts"], residential["amounts"]);
    }
}

/// Each step's working shows its formula, the operands and the exact value.
#[test]
fn the_steps_show_their_working() {
    let envelope =
        json!({"command": "transfer-duty", "input": input("nsw", "900000", "first_home_new")});
    let doc = api::evaluate("derive", &envelope).unwrap();
    let workings: Vec<&str> = doc["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["working"].as_str().unwrap())
        .collect();
    assert_eq!(
        workings,
        [
            "max(base + dollars_up((dutiable_value - threshold) / 100) x rate_per_100, minimum_duty) = max(11602 + dollars_up((900000 - 387000) / 100) x 4.50, 20) = 34687.00",
            "max(exempt_base + dollars_up((first_home_exempt_to - exempt_threshold) / 100) x exempt_rate_per_100, minimum_duty) = max(11602 + dollars_up((800000 - 387000) / 100) x 4.50, 20) = 30187.00",
            "max(general_duty - cents((first_home_limit - dutiable_value) / (first_home_limit - first_home_exempt_to) x duty_on_800000), minimum_duty) = max(34687.00 - cents((1000000 - 900000) / (1000000 - 800000) x 30187.00), 20) = 19593.50",
            "general_duty - duty = 34687.00 - 19593.50 = 15093.50",
        ]
    );
    let vic = json!({"command": "transfer-duty", "input": input("vic", "25001", "general")});
    let doc = api::evaluate("derive", &vic).unwrap();
    assert_eq!(
        doc["steps"][0]["working"],
        "dollars_up(base + (dutiable_value - threshold) x rate - 0.5) = dollars_up(350 + (25001 - 25000) x 0.024 - 0.5) = 350"
    );
}
