// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Land tax for the 7 levying jurisdictions against the site's land tax
//! calculator (ryanduguid.github.io at 1aa0a6cb: its 23 calculated cases, the
//! oracle's worked figures and every band edge, run through the page's own
//! `landTaxYear` into `data/land_tax_site_cases.json`), and against the Acts
//! where the page differs from them: Queensland's surcharge over $349,999,
//! South Australia's $20 minimum and Tasmania's rounding to 10 cents. Every
//! source was read on 23 November 2026 (docs/calculation-evidence.md#land-tax).
//! Every owner here is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn year(place: &str) -> &'static str {
    if matches!(place, "nsw" | "vic") {
        "2026"
    } else {
        "2026-27"
    }
}

fn input(place: &str, value: &str) -> Value {
    let mut v = json!({
        "jurisdiction": place, "taxable_value": value, "trust_rates": false,
        "foreign_owner": false, "year": year(place), "scope_confirmed": true,
    });
    if place == "wa" {
        v["metropolitan_region"] = json!("none");
    }
    v
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
    let result = api::evaluate("land-tax", input).unwrap();
    let envelope = json!({"command": "land-tax", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

/// land_tax, surcharge, mrit and total, "0.00" for an amount not worked.
fn figures(result: &Value) -> [String; 4] {
    ["land_tax", "surcharge", "mrit", "total"].map(|key| {
        result["amounts"][key]
            .as_str()
            .unwrap_or("0.00")
            .to_string()
    })
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("land-tax", input).unwrap_err().code
}

fn cents(text: &str) -> i64 {
    let (whole, part) = text.split_once('.').unwrap();
    whole.parse::<i64>().unwrap() * 100 + part.parse::<i64>().unwrap()
}

fn shown(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

/// Why the command's figure differs from the site's on a row, if it does.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Deviation {
    /// Land Tax Act 2010 (Qld) Schs 2 Pt 2 and 3 Pt 2: 3.0c for each $1 over
    /// $349,999, 3 cents more than the page's 3% over $350,000.
    QueenslandSurcharge,
    /// Land Tax Act 1936 (SA) s 6: nil below $20.
    SouthAustraliaMinimum,
    /// Land Tax Rating Act 2000 (Tas) s 7: rounded to 10 cents.
    TasmaniaTenCents,
}

/// The site's figures moved by each rule the Act adds, and the rules that
/// moved them. Tasmania's s 7 works on the exact amount, which the page has
/// already rounded to the cent, so its figure is checked to within 6 cents
/// and to a multiple of 10 cents.
fn expected(row: &Value, got: &[String; 4]) -> ([i64; 4], Vec<Deviation>) {
    let site = &row["site"];
    let mut want = ["tax", "surcharge", "mrit", "total"].map(|k| cents(site[k].as_str().unwrap()));
    let mut why = Vec::new();
    let value: i64 = row["value"].as_str().unwrap().parse().unwrap();
    match row["place"].as_str().unwrap() {
        "qld" if row["foreign"] == true && value >= 350_000 => {
            want[1] += 3;
            want[3] += 3;
            why.push(Deviation::QueenslandSurcharge);
        }
        "sa" if want[0] > 0 && want[0] < 2000 => {
            want = [0; 4];
            why.push(Deviation::SouthAustraliaMinimum);
        }
        "tas" => {
            for (i, figure) in [(0, &got[0]), (1, &got[1])] {
                let got = cents(figure);
                assert_eq!(got % 10, 0, "{row}: {figure} is not a multiple of 10 cents");
                assert!((got - want[i]).abs() <= 6, "{row}: {figure}");
                if got != want[i] {
                    want[3] += got - want[i];
                    want[i] = got;
                    why.push(Deviation::TasmaniaTenCents);
                }
            }
        }
        _ => {}
    }
    (want, why)
}

fn site_input(row: &Value) -> Value {
    let place = row["place"].as_str().unwrap();
    let mut v = input(place, row["value"].as_str().unwrap());
    v["trust_rates"] = row["trust"].clone();
    v["foreign_owner"] = row["foreign"].clone();
    if let Some(separate) = row["surcharge_value"].as_str() {
        v["surcharge_value"] = json!(separate);
    }
    if place == "wa" {
        v["metropolitan_region"] = json!(if row["metro"] == true { "all" } else { "none" });
    }
    v
}

/// Every row of the site's fixture agrees, or differs only by a rule above.
#[test]
fn the_site_cases_agree_except_where_an_act_differs() {
    let data: Value = serde_json::from_str(include_str!("data/land_tax_site_cases.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    let mut same = 0;
    let mut moved = std::collections::BTreeMap::<String, usize>::new();
    for row in rows {
        let got = figures(&run(&site_input(row)));
        let (want, why) = expected(row, &got);
        assert_eq!(got, want.map(shown), "{row}");
        if why.is_empty() {
            same += 1;
        }
        for reason in why {
            *moved.entry(format!("{reason:?}")).or_default() += 1;
        }
    }
    eprintln!(
        "land-tax site parity: {} rows, {same} identical, deviations {moved:?}",
        rows.len()
    );
    assert_eq!(rows.len(), 875);
    assert_eq!(data["page_rows"], 23);
    // The page's own 23 cases: South Australia's $936,050 and the 2
    // Queensland surcharge examples move.
    let page_moved = rows[..23]
        .iter()
        .filter(|row| !expected(row, &figures(&run(&site_input(row)))).1.is_empty())
        .map(|row| row["case"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        page_moved,
        [
            "page: Part of $100 over the South Australian threshold counts as a whole $100",
            "page: The Queensland Revenue Office’s absentee example",
            "page: The Queensland Revenue Office’s foreign company example",
        ]
    );
}

/// The page's 2 refused cases refuse alike.
#[test]
fn the_site_refusals_refuse_alike() {
    let trust = with(input("wa", "1500000"), json!({"trust_rates": true}));
    assert_eq!(code(&trust), "land_tax.trust_rates_unavailable");
    assert_eq!(
        code(&input("nsw", "1100000.50")),
        "input.amount_not_whole_dollars"
    );
}

/// Revenue NSW: $500 on $1,100,000, the special trust's $9,600 on $600,000,
/// and surcharge land tax examples 1 to 3 (5% of the residential land value).
#[test]
fn new_south_wales_agrees_with_revenue_nsw() {
    assert_eq!(
        figures(&run(&input("nsw", "1100000"))),
        ["500.00", "0.00", "0.00", "500.00"]
    );
    let trust = with(input("nsw", "600000"), json!({"trust_rates": true}));
    assert_eq!(figures(&run(&trust))[0], "9600.00");
    let foreign = |value: &str, separate: &str| {
        with(
            input("nsw", value),
            json!({"foreign_owner": true, "surcharge_value": separate}),
        )
    };
    let one = run(&foreign("1200000", "1200000"));
    assert_eq!(figures(&one), ["2100.00", "60000.00", "0.00", "62100.00"]);
    assert_eq!(
        figures(&run(&foreign("500000", "500000"))),
        ["0.00", "25000.00", "0.00", "25000.00"]
    );
    assert_eq!(
        figures(&run(&foreign("1200000", "500000"))),
        ["2100.00", "25000.00", "0.00", "27100.00"]
    );
    assert_eq!(one["rates"]["surcharge_rate"], "0.05");
    assert_eq!(one["period"], "2026");
    // The premium rate starts above $6,571,000, where both rows give $88,036.
    assert_eq!(figures(&run(&input("nsw", "6571000")))[0], "88036.00");
    assert_eq!(figures(&run(&input("nsw", "6571001")))[0], "88036.02");
}

/// Queensland's surcharge follows the Act: 3.0c for each $1 over $349,999
/// from $350,000, so $1,500.03 on $400,000 where the QRO's examples show
/// $1,500, and 3 cents at $350,000 itself.
#[test]
fn queensland_surcharge_follows_the_act() {
    let absentee = with(input("qld", "400000"), json!({"foreign_owner": true}));
    let out = run(&absentee);
    assert_eq!(figures(&out), ["2300.00", "1500.03", "0.00", "3800.03"]);
    assert!(out["sources"].to_string().contains("act-2010-015"));
    let at = with(input("qld", "350000"), json!({"foreign_owner": true}));
    assert_eq!(figures(&run(&at))[1], "0.03");
    let below = with(input("qld", "349999"), json!({"foreign_owner": true}));
    assert_eq!(figures(&run(&below)), ["0.00", "0.00", "0.00", "0.00"]);
    let company = with(input("qld", "6400000"), json!({"trust_rates": true}));
    assert_eq!(figures(&run(&company))[0], "106500.00");
}

/// South Australia's s 6: a total under $20 is nil; $20 itself is payable.
#[test]
fn south_australia_charges_nothing_under_twenty_dollars() {
    assert_eq!(figures(&run(&input("sa", "939900")))[0], "0.00");
    assert_eq!(figures(&run(&input("sa", "940000")))[0], "20.00");
    let out = run(&input("sa", "1470000"));
    assert_eq!(figures(&out)[0], "2670.00");
    assert_eq!(out["rates"]["rate_per_100"], "0.50");
    assert_eq!(figures(&run(&input("sa", "1575000")))[0], "3550.00");
}

/// Tasmania's s 7, worked by hand: $125,001 gives $50.0045, 5,000.45 cents,
/// down to $50.00; $125,011 gives 5,004.95 cents, down; $125,012 gives
/// 5,005.40 cents, more than 5 remaining, so up to $50.10, as is $125,013's
/// 5,005.85. FILTS example 1: $8,000 on $400,000.
#[test]
fn tasmania_rounds_to_ten_cents() {
    for (value, want) in [
        ("125001", "50.00"),
        ("125011", "50.00"),
        ("125012", "50.10"),
        ("125013", "50.10"),
    ] {
        assert_eq!(figures(&run(&input("tas", value)))[0], want, "{value}");
    }
    let filts = with(
        input("tas", "400000"),
        json!({"foreign_owner": true, "surcharge_value": "400000"}),
    );
    assert_eq!(
        figures(&run(&filts)),
        ["1287.50", "8000.00", "0.00", "9287.50"]
    );
    // 2% of $1,234,567 is $24,691.34: 4 cents remain, so down.
    let odd = with(filts, json!({"surcharge_value": "1234567"}));
    assert_eq!(figures(&run(&odd))[1], "24691.30");
}

/// The ACT's fixed charge of $1,778 with Table 1, and the 0.75% surcharge.
#[test]
fn the_act_adds_its_fixed_charge() {
    let out = run(&with(
        input("act", "600000"),
        json!({"foreign_owner": true}),
    ));
    assert_eq!(figures(&out), ["7418.00", "4500.00", "0.00", "11918.00"]);
    assert_eq!(out["rates"]["fixed_charge"], "1778");
    assert_eq!(figures(&run(&input("act", "0")))[0], "1778.00");
}

/// Victoria's surcharge is its absentee table less the general or trust
/// table, shown as its own step; Western Australia's MRIT.
#[test]
fn victoria_and_western_australia() {
    let out = run(&with(
        input("vic", "1000000"),
        json!({"foreign_owner": true}),
    ));
    assert_eq!(out["amounts"]["absentee_rate_tax"], "44650.00");
    assert_eq!(figures(&out), ["4650.00", "40000.00", "0.00", "44650.00"]);
    let metro = with(
        input("wa", "1500000"),
        json!({"metropolitan_region": "all"}),
    );
    let out = run(&metro);
    assert_eq!(figures(&out), ["6250.00", "0.00", "1680.00", "7930.00"]);
    assert_eq!(out["rates"]["mrit_rate"], "0.0014");
}

fn refused(cases: &[(&str, Value, &str)]) {
    for (place, changes, want) in cases {
        let input = with(input(place, "1"), changes.clone());
        assert_eq!(code(&input), *want, "{input}");
    }
}

#[test]
fn refuses_other_places_years_and_rates() {
    refused(&[
        ("nt", json!({}), "land_tax.jurisdiction_invalid"),
        (
            "nsw",
            json!({"jurisdiction": 7}),
            "land_tax.jurisdiction_invalid",
        ),
        ("nsw", json!({"year": "2026-27"}), "period.unsupported"),
        ("qld", json!({"year": "2026"}), "period.unsupported"),
        ("qld", json!({"year": "2025-26"}), "period.unsupported"),
        (
            "tas",
            json!({"trust_rates": true}),
            "land_tax.trust_rates_unavailable",
        ),
        (
            "sa",
            json!({"foreign_owner": true}),
            "land_tax.surcharge_unavailable",
        ),
        (
            "wa",
            json!({"foreign_owner": true}),
            "land_tax.surcharge_unavailable",
        ),
        (
            "vic",
            json!({"scope_confirmed": false}),
            "scope.not_confirmed",
        ),
        (
            "vic",
            json!({"taxable_value": "-1"}),
            "input.amount_invalid",
        ),
    ]);
}

#[test]
fn refuses_inputs_out_of_place() {
    let separate = |v: &str| json!({"foreign_owner": true, "surcharge_value": v});
    refused(&[
        ("nsw", json!({"foreign_owner": true}), "input.missing_field"),
        (
            "nsw",
            json!({"surcharge_value": "1"}),
            "input.field_unknown",
        ),
        ("qld", separate("1"), "input.field_unknown"),
        ("nsw", separate("1.50"), "input.amount_not_whole_dollars"),
        (
            "wa",
            json!({"metropolitan_region": null}),
            "input.missing_field",
        ),
        (
            "vic",
            json!({"metropolitan_region": "all"}),
            "input.field_unknown",
        ),
        (
            "wa",
            json!({"metropolitan_region": "part"}),
            "land_tax.metropolitan_region_part",
        ),
        (
            "wa",
            json!({"metropolitan_region": true}),
            "land_tax.metropolitan_region_invalid",
        ),
        ("vic", json!({"trust_rates": "yes"}), "input.flag_invalid"),
        ("vic", json!({"foreign_owner": 1}), "input.flag_invalid"),
        ("vic", json!({"owner": "trust"}), "input.field_unknown"),
    ]);
}

/// Each step names its authority and shows its working.
#[test]
fn the_steps_show_their_working() {
    let envelope = json!({"command": "land-tax", "input": input("sa", "1470000")});
    let doc = api::evaluate("derive", &envelope).unwrap();
    let land = &doc["steps"][0];
    assert_eq!(land["id"], "land_tax");
    assert!(
        land["authority"]
            .as_str()
            .unwrap()
            .contains("Land Tax Act 1936 (SA) s 8A(1)")
    );
    assert_eq!(
        land["working"],
        "nil_below_20(base + dollars_up((taxable_value - threshold) / 100) x rate_per_100) = nil_below_20(0 + dollars_up((1470000 - 936000) / 100) x 0.50) = 2670.00"
    );
    let envelope = json!({"command": "land-tax", "input": input("act", "600000")});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert!(
        doc["steps"][0]["working"]
            .as_str()
            .unwrap()
            .starts_with("fixed_charge + base")
    );
}
