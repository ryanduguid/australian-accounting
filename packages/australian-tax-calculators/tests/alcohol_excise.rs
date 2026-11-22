// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Alcohol excise (Excise Tariff Act 1921 Schedule items 1 to 3, ss 6A and
//! 6L; Excise Regulation 2015 Sch 1 cl 2(1) item 10) on the ATO's and the
//! Excise guidelines' worked examples and the Commissioner's notices of
//! substituted rates, read on 22 November 2026
//! (docs/calculation-evidence.md#alcohol-excise).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(year: &str, lines: Value) -> Value {
    json!({"lines": lines, "year": year, "scope_confirmed": true})
}

fn run(input: &Value) -> Value {
    api::evaluate("alcohol-excise", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("alcohol-excise", input)
        .unwrap_err()
        .code
}

fn line(date: &str, product: &str, volume: &str, strength: &str) -> Value {
    json!({"date": date, "product": product, "volume_litres": volume, "strength_percent": strength})
}

fn beer(date: &str, volume: &str, strength: &str, container: &str) -> Value {
    let mut l = line(date, "beer", volume, strength);
    l["container_litres"] = json!(container);
    l
}

fn keg(date: &str, volume: &str, strength: &str, container: &str, pressurised: bool) -> Value {
    let mut l = beer(date, volume, strength, container);
    l["pressurised_system"] = json!(pressurised);
    l
}

fn year_of(date: &str) -> String {
    let (y, m): (i32, i32) = (date[..4].parse().unwrap(), date[5..7].parse().unwrap());
    let first = if m >= 7 { y } else { y - 1 };
    format!("{first}-{:02}", (first + 1) % 100)
}

fn one(l: Value) -> Value {
    let year = year_of(l["date"].as_str().unwrap());
    run(&input(&year, json!([l])))
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

fn first_return_line(result: &Value) -> &Value {
    &result["return_lines"][0]
}

/// The ATO's 'Calculating alcohol excise duty' (QC63603): Brewery Co's 10
/// cases of 24 x 355 ml at 5% on 8 March 2023, subitem 1.10 at $57.79:
/// 3.28 LAL, truncated to 3.2, $184.92.
#[test]
fn the_ato_beer_example() {
    let r = one(beer("2023-03-08", "85.2", "5", "0.355"));
    let g = first_return_line(&r);
    assert_eq!(g["subitem"], "1.10");
    assert_eq!(g["rate_per_litre_of_alcohol"], "57.79");
    assert_eq!(r["lines"][0]["litres_of_alcohol"], "3.2802");
    assert_eq!(g["litres_of_alcohol_truncated"], "3.2");
    assert_eq!(amount(&r, "excise_duty"), "184.92");
    assert_eq!(amount(&r, "net_duty"), "184.92");
}

/// QC63603: Bottler's Gin, 840 litres at 37.2% on 10 February 2023,
/// subitem 3.2 at $97.90: 312.48 LAL, 312.4, $30,583.96.
#[test]
fn the_ato_spirits_example() {
    let r = one(line(
        "2023-02-10",
        "other_excisable_beverage",
        "840",
        "37.2",
    ));
    let g = first_return_line(&r);
    assert_eq!(g["subitem"], "3.2");
    assert_eq!(g["litres_of_alcohol"], "312.48");
    assert_eq!(g["litres_of_alcohol_truncated"], "312.4");
    assert_eq!(amount(&r, "excise_duty"), "30583.96");
}

/// Excise guidelines for the alcohol industry, Example 6M: the same gin on
/// 10 February 2025 at $104.31: 32,586.444 truncated to $32,586.44.
#[test]
fn the_guidelines_example_6m_truncates_the_duty() {
    let r = one(line(
        "2025-02-10",
        "other_excisable_beverage",
        "840",
        "37.2",
    ));
    assert_eq!(first_return_line(&r)["rate_per_litre_of_alcohol"], "104.31");
    assert_eq!(amount(&r, "excise_duty"), "32586.44");
}

/// Examples 6B and 6D: 840 litres of brandy at 37.2% is 312.48 LAL; 100
/// litres of beer at 5% is 3.85 LAL.
#[test]
fn the_guidelines_litres_of_alcohol() {
    let r = one(line("2025-02-10", "brandy", "840", "37.2"));
    assert_eq!(r["lines"][0]["litres_of_alcohol"], "312.48");
    assert_eq!(first_return_line(&r)["subitem"], "3.1");
    let r = one(beer("2025-02-10", "100", "5", "0.375"));
    assert_eq!(r["lines"][0]["litres_of_alcohol"], "3.85");
}

/// Example 6O: RTDs under item 2, 189.5 LAL on Sunday 2 February 2025 at
/// $103.89 and 1,041.3 LAL after the rate rose to $104.31 on 3 February are
/// separate return lines.
#[test]
fn a_rate_change_inside_a_return_makes_separate_lines() {
    let lines = json!([
        line("2025-02-02", "other_excisable_beverage", "1895", "10"),
        line("2025-02-04", "other_excisable_beverage", "10413", "10"),
    ]);
    let r = run(&input("2024-25", lines));
    let g = r["return_lines"].as_array().unwrap();
    assert_eq!(g.len(), 2);
    assert_eq!(
        (&g[0]["subitem"], &g[0]["rate_per_litre_of_alcohol"]),
        (&json!("2"), &json!("103.89"))
    );
    assert_eq!(g[0]["duty"], "19687.15");
    assert_eq!(g[1]["rate_per_litre_of_alcohol"], "104.31");
    assert_eq!(g[1]["duty"], "108618.00");
    assert_eq!(amount(&r, "excise_duty"), "128305.15");
}

/// Example 6L: lines of one subitem at one rate are added before the
/// truncation (2,356.79 LAL is reported as 2,356.7).
#[test]
fn lines_of_one_subitem_and_rate_are_one_return_line() {
    let lines = json!([
        line("2025-02-10", "brandy", "756.23", "100"),
        line("2025-02-11", "brandy", "497.54", "100"),
        line("2025-02-12", "brandy", "1103.02", "100"),
    ]);
    let r = run(&input("2024-25", lines));
    let g = r["return_lines"].as_array().unwrap();
    assert_eq!(g.len(), 1);
    assert_eq!(g[0]["litres_of_alcohol"], "2356.79");
    assert_eq!(g[0]["litres_of_alcohol_truncated"], "2356.7");
    assert_eq!(g[0]["lines"], json!([1, 2, 3]));
    // 2,356.7 x $97.41 = 229,566.147.
    assert_eq!(amount(&r, "excise_duty"), "229566.14");
}

fn subitem_of(l: Value) -> String {
    first_return_line(&one(l))["subitem"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Schedule item 1: the container tests at 7.99, 8, 48 and 48.01 litres.
#[test]
fn the_container_boundaries() {
    let d = "2026-09-15";
    assert_eq!(subitem_of(beer(d, "10", "5", "7.99")), "1.10");
    assert_eq!(subitem_of(keg(d, "10", "5", "8", false)), "1.10");
    assert_eq!(subitem_of(keg(d, "10", "5", "8", true)), "1.11");
    assert_eq!(subitem_of(keg(d, "10", "5", "48", false)), "1.10");
    assert_eq!(subitem_of(keg(d, "10", "5", "48", true)), "1.11");
    assert_eq!(subitem_of(beer(d, "10", "5", "48.01")), "1.11");
    assert_eq!(
        code(&input("2026-27", json!([beer(d, "10", "5", "8")]))),
        "input.missing_field"
    );
    assert_eq!(
        code(&input("2026-27", json!([keg(d, "10", "5", "50", true)]))),
        "input.field_unknown"
    );
}

/// Schedule item 1's strength bands and the 1.15% threshold; items 2 and
/// 3.2 at 10%.
#[test]
fn the_strength_boundaries() {
    let d = "2026-09-15";
    assert_eq!(subitem_of(beer(d, "10", "3", "0.375")), "1.1");
    assert_eq!(subitem_of(beer(d, "10", "3.001", "0.375")), "1.5");
    assert_eq!(subitem_of(beer(d, "10", "3.5", "0.375")), "1.5");
    assert_eq!(subitem_of(beer(d, "10", "3.501", "0.375")), "1.10");
    assert_eq!(subitem_of(keg(d, "10", "3", "30", true)), "1.2");
    assert_eq!(subitem_of(keg(d, "10", "3.5", "30", true)), "1.6");
    assert_eq!(subitem_of(beer(d, "10", "1.151", "0.375")), "1.1");
    let o = "other_excisable_beverage";
    assert_eq!(subitem_of(line(d, o, "10", "10")), "2");
    assert_eq!(subitem_of(line(d, o, "10", "10.001")), "3.2");
    assert_eq!(subitem_of(line(d, "spirit", "10", "40")), "3.10");
    for low in [beer(d, "10", "1.15", "0.375"), line(d, o, "10", "1.15")] {
        assert_eq!(
            code(&input("2026-27", json!([low]))),
            "alcohol_excise.not_excisable"
        );
    }
}

/// Subitems 1.15 and 1.16: beer brewed on premises, whatever the container.
#[test]
fn brew_on_premises_beer() {
    let d = "2026-09-15";
    let mut l = line(d, "beer", "20", "3");
    l["brew_on_premises"] = json!(true);
    assert_eq!(subitem_of(l.clone()), "1.15");
    l["strength_percent"] = json!("3.001");
    assert_eq!(subitem_of(l.clone()), "1.16");
    l["container_litres"] = json!("20");
    assert_eq!(code(&input("2026-27", json!([l]))), "input.field_unknown");
}

/// Excise Tariff Act 1921 s 6L: 1.11 stays at $43.39 from 3 February 2025
/// through the August 2025, February 2026 and August 2026 indexation days,
/// while 1.10 rises.
#[test]
fn the_draught_beer_freeze() {
    for d in ["2025-02-03", "2025-08-04", "2026-02-02", "2026-08-03"] {
        let r = one(beer(d, "100", "5.15", "50"));
        assert_eq!(
            first_return_line(&r)["rate_per_litre_of_alcohol"],
            "43.39",
            "{d}"
        );
    }
    let r = one(beer("2025-08-04", "100", "5.15", "0.375"));
    assert_eq!(first_return_line(&r)["rate_per_litre_of_alcohol"], "62.56");
}

/// Excise Act 1901 s 77FA: duty on the labelled strength where it is the
/// higher; refused where the label would change the subitem.
#[test]
fn a_higher_labelled_strength_is_charged() {
    let mut l = beer("2026-09-15", "100", "4.8", "0.375");
    l["labelled_strength_percent"] = json!("5.0");
    let r = one(l.clone());
    assert_eq!(r["lines"][0]["strength_basis"], "labelled");
    assert_eq!(r["lines"][0]["litres_of_alcohol"], "3.85");
    l["labelled_strength_percent"] = json!("4.5");
    assert_eq!(one(l.clone())["lines"][0]["litres_of_alcohol"], "3.65");
    let mut crossing = line("2026-09-15", "other_excisable_beverage", "10", "9.9");
    crossing["labelled_strength_percent"] = json!("10.5");
    assert_eq!(
        code(&input("2026-27", json!([crossing]))),
        "alcohol_excise.classification_unsettled"
    );
}

/// The Commissioner's substituted rates (Notices No. 1 and 3 of 2022 to
/// 2026), per litre of alcohol, by subitem in the order 1.1, 1.2, 1.5, 1.6,
/// 1.10, 1.11, 1.15, 1.16, 2, 3.1, 3.2, 3.10.
const NOTICES: [(&str, &str, [&str; 12]); 10] = [
    (
        "2022-07-01",
        "2022-07-31",
        [
            "46.02", "9.20", "53.59", "28.82", "53.59", "37.76", "3.24", "3.73", "90.78", "84.78",
            "90.78", "90.78",
        ],
    ),
    (
        "2022-08-01",
        "2023-01-31",
        [
            "47.86", "9.57", "55.73", "29.97", "55.73", "39.27", "3.37", "3.88", "94.41", "88.17",
            "94.41", "94.41",
        ],
    ),
    (
        "2023-02-01",
        "2023-07-31",
        [
            "49.63", "9.92", "57.79", "31.08", "57.79", "40.72", "3.49", "4.02", "97.90", "91.43",
            "97.90", "97.90",
        ],
    ),
    (
        "2023-08-01",
        "2024-02-04",
        [
            "50.72", "10.14", "59.06", "31.76", "59.06", "41.62", "3.57", "4.11", "100.05",
            "93.44", "100.05", "100.05",
        ],
    ),
    (
        "2024-02-05",
        "2024-08-04",
        [
            "51.63", "10.32", "60.12", "32.33", "60.12", "42.37", "3.63", "4.18", "101.85",
            "95.12", "101.85", "101.85",
        ],
    ),
    (
        "2024-08-05",
        "2025-02-02",
        [
            "52.66", "10.53", "61.32", "32.98", "61.32", "43.22", "3.70", "4.26", "103.89",
            "97.02", "103.89", "103.89",
        ],
    ),
    (
        "2025-02-03",
        "2025-08-03",
        [
            "52.87", "10.57", "61.57", "33.11", "61.57", "43.39", "3.71", "4.28", "104.31",
            "97.41", "104.31", "104.31",
        ],
    ),
    (
        "2025-08-04",
        "2026-02-01",
        [
            "53.72", "10.57", "62.56", "33.11", "62.56", "43.39", "3.77", "4.35", "105.98",
            "98.97", "105.98", "105.98",
        ],
    ),
    (
        "2026-02-02",
        "2026-08-02",
        [
            "54.74", "10.57", "63.75", "33.11", "63.75", "43.39", "3.84", "4.43", "107.99",
            "100.85", "107.99", "107.99",
        ],
    ),
    (
        "2026-08-03",
        "2027-01-31",
        [
            "55.83", "10.57", "65.03", "33.11", "65.03", "43.39", "3.92", "4.52", "110.15",
            "102.87", "110.15", "110.15",
        ],
    ),
];

/// A line of exactly 10 litres of alcohol in each subitem on `date`.
fn ten_litres_each(date: &str) -> Vec<Value> {
    let bop = |strength: &str, volume: &str| {
        let mut l = line(date, "beer", volume, strength);
        l["brew_on_premises"] = json!(true);
        l
    };
    vec![
        beer(date, "1000", "2.15", "0.375"),
        beer(date, "1000", "2.15", "50"),
        beer(date, "500", "3.15", "0.375"),
        beer(date, "500", "3.15", "50"),
        beer(date, "250", "5.15", "0.375"),
        beer(date, "250", "5.15", "50"),
        bop("2.15", "1000"),
        bop("3.15", "500"),
        line(date, "other_excisable_beverage", "100", "10"),
        line(date, "brandy", "25", "40"),
        line(date, "other_excisable_beverage", "25", "40"),
        line(date, "spirit", "25", "40"),
    ]
}

/// Every rate on the first and last day of every period: 10 LAL x the rate.
#[test]
fn every_rate_table_entry_on_each_period_boundary() {
    let subitems = [
        "1.1", "1.2", "1.5", "1.6", "1.10", "1.11", "1.15", "1.16", "2", "3.1", "3.2", "3.10",
    ];
    for (from, to, rates) in NOTICES {
        for date in [from, to] {
            let r = run(&input(&year_of(date), json!(ten_litres_each(date))));
            let g = r["return_lines"].as_array().unwrap();
            assert_eq!(g.len(), 12, "{date}");
            for (i, s) in subitems.iter().enumerate() {
                assert_eq!(g[i]["subitem"], *s, "{date}");
                assert_eq!(g[i]["rate_per_litre_of_alcohol"], rates[i], "{date} {s}");
                assert_eq!(g[i]["litres_of_alcohol_truncated"], "10.0", "{date} {s}");
                let rate: f64 = rates[i].parse().unwrap();
                assert_eq!(g[i]["duty"], format!("{:.2}", rate * 10.0), "{date} {s}");
            }
        }
    }
}

/// The held range: 1 July 2022 to 31 January 2027.
#[test]
fn dates_outside_the_held_rates_are_refused() {
    assert_eq!(
        code(&input(
            "2022-23",
            json!([line("2022-06-30", "spirit", "1", "40")])
        )),
        "alcohol_excise.date_outside_year"
    );
    assert_eq!(
        code(&input(
            "2026-27",
            json!([line("2027-02-01", "spirit", "1", "40")])
        )),
        "alcohol_excise.rate_unpublished"
    );
    assert_eq!(
        code(&input(
            "2021-22",
            json!([line("2022-06-30", "spirit", "1", "40")])
        )),
        "period.unsupported"
    );
    assert_eq!(
        code(&input(
            "2025-26",
            json!([line("2026-07-01", "spirit", "1", "40")])
        )),
        "alcohol_excise.date_outside_year"
    );
}

/// The map row's refusals: wine, tobacco, fuel, the approval-based free
/// spirit, non-commercial liquor and imported goods.
#[test]
fn products_outside_the_worksheet_are_refused() {
    let d = "2026-09-15";
    for (product, expected) in [
        ("wine", "alcohol_excise.not_excisable"),
        ("non_commercial", "alcohol_excise.not_excisable"),
        ("tobacco", "alcohol_excise.product_out_of_scope"),
        ("fuel", "alcohol_excise.product_out_of_scope"),
        ("concessional_spirit", "alcohol_excise.product_out_of_scope"),
        ("cider", "alcohol_excise.product_invalid"),
    ] {
        assert_eq!(
            code(&input("2026-27", json!([line(d, product, "1", "12")]))),
            expected,
            "{product}"
        );
    }
    let mut imported = line(d, "spirit", "1", "40");
    imported["imported"] = json!(true);
    assert_eq!(
        code(&input("2026-27", json!([imported]))),
        "alcohol_excise.product_out_of_scope"
    );
}

#[test]
fn malformed_lines_are_refused() {
    let d = "2026-09-15";
    for (lines, expected) in [
        (json!([]), "alcohol_excise.lines_invalid"),
        (json!("x"), "alcohol_excise.lines_invalid"),
        (json!([1]), "alcohol_excise.line_invalid"),
        (
            json!([line(d, "spirit", "0", "40")]),
            "alcohol_excise.quantity_invalid",
        ),
        (
            json!([line(d, "spirit", "1", "100.1")]),
            "alcohol_excise.quantity_invalid",
        ),
        (
            json!([line(d, "spirit", "1.00001", "40")]),
            "alcohol_excise.quantity_invalid",
        ),
        (
            json!([line("2026-13-01", "spirit", "1", "40")]),
            "input.date_invalid",
        ),
        (
            json!([{"date": d, "product": "spirit", "strength_percent": "40"}]),
            "input.missing_field",
        ),
        (
            json!([{"date": d, "volume_litres": "1", "strength_percent": "40"}]),
            "input.missing_field",
        ),
    ] {
        assert_eq!(code(&input("2026-27", lines.clone())), expected, "{lines}");
    }
    let mut spirit = line(d, "spirit", "1", "40");
    spirit["container_litres"] = json!("0.7");
    assert_eq!(
        code(&input("2026-27", json!([spirit]))),
        "input.field_unknown"
    );
    let mut bad_flag = line(d, "spirit", "1", "40");
    bad_flag["remission"] = json!("yes");
    assert_eq!(
        code(&input("2026-27", json!([bad_flag]))),
        "input.flag_invalid"
    );
    let mut i = input("2026-27", json!([line(d, "spirit", "1", "40")]));
    i["scope_confirmed"] = json!(false);
    assert_eq!(code(&i), "scope.not_confirmed");
}

fn remitted(year: &str, lines: Value, used: &str) -> Value {
    let mut i = input(year, lines);
    i["remission_used_this_year"] = json!(used);
    i
}

fn remission_line(date: &str, volume: &str) -> Value {
    let mut l = line(date, "spirit", volume, "40");
    l["remission"] = json!(true);
    l
}

/// Excise Regulation 2015 item 10(c)(i): $400,000 for goods entered from
/// 1 July 2026, $350,000 before. 10,000 litres of 40% spirit is 4,000 LAL.
#[test]
fn the_remission_cap_by_year() {
    // 4,000 x $110.15 = $440,600; capped at $400,000.
    let r = run(&remitted(
        "2026-27",
        json!([remission_line("2026-09-15", "10000")]),
        "0",
    ));
    assert_eq!(amount(&r, "excise_duty"), "440600.00");
    assert_eq!(amount(&r, "remission"), "400000.00");
    assert_eq!(amount(&r, "over_cap"), "40600.00");
    assert_eq!(amount(&r, "net_duty"), "40600.00");
    assert_eq!(r["rates"]["remission_cap"], "400000");
    // 30 June 2026: 4,000 x $107.99 = $431,960; capped at $350,000.
    let r = run(&remitted(
        "2025-26",
        json!([remission_line("2026-06-30", "10000")]),
        "0",
    ));
    assert_eq!(amount(&r, "remission"), "350000.00");
    assert_eq!(amount(&r, "net_duty"), "81960.00");
}

/// The cap left after earlier remissions, and lines without the remission.
#[test]
fn the_remission_uses_the_cap_left() {
    let lines = json!([
        remission_line("2026-09-15", "100"),
        line("2026-09-15", "spirit", "100", "40")
    ]);
    // Each line is 40 LAL x $110.15 = $4,406.00.
    let r = run(&remitted("2026-27", lines.clone(), "398000.00"));
    assert_eq!(amount(&r, "excise_duty"), "8812.00");
    assert_eq!(amount(&r, "remission_eligible_duty"), "4406.00");
    assert_eq!(amount(&r, "remission"), "2000.00");
    assert_eq!(amount(&r, "net_duty"), "6812.00");
    assert_eq!(r["return_lines"].as_array().unwrap().len(), 2);
    let r = run(&remitted("2026-27", lines, "400000.00"));
    assert_eq!(amount(&r, "remission"), "0.00");
    assert_eq!(r["rates"]["remission_cap_left"], "0");
}

/// QC65935: a licence from 30 October 2026 has $400,000 x 244 / 365 =
/// $267,397.26 (the ATO shows $267,397).
#[test]
fn the_part_year_cap() {
    let mut i = remitted(
        "2026-27",
        json!([remission_line("2026-11-02", "10000")]),
        "0",
    );
    i["manufacturer_from"] = json!("2026-10-30");
    let r = run(&i);
    assert_eq!(r["rates"]["remission_cap"], "267397.26");
    assert_eq!(r["part_year_cap"]["eligible_days"], 244);
    assert_eq!(amount(&r, "remission"), "267397.26");
    // A manufacturer from the year's first day or before has the whole cap.
    i["manufacturer_from"] = json!("2026-07-01");
    assert_eq!(run(&i)["rates"]["remission_cap"], "400000");
    // Not a manufacturer yet when the goods were entered.
    i["manufacturer_from"] = json!("2026-11-03");
    assert_eq!(code(&i), "alcohol_excise.fact_conflict");
}

/// Excise Regulation 2015 cl 2(2): duty paid where the remission was
/// available is refunded, against the same cap.
#[test]
fn duty_paid_is_a_refund() {
    let mut i = remitted("2026-27", json!([remission_line("2026-09-15", "100")]), "0");
    i["duty_paid"] = json!(true);
    let r = run(&i);
    assert_eq!(amount(&r, "refund"), "4406.00");
    assert_eq!(amount(&r, "remission"), "0.00");
    assert_eq!(amount(&r, "net_duty"), "0.00");
}

#[test]
fn remission_inputs_go_with_a_remission_line() {
    let d = "2026-09-15";
    assert_eq!(
        code(&input("2026-27", json!([remission_line(d, "1")]))),
        "input.missing_field"
    );
    let mut i = remitted("2026-27", json!([line(d, "spirit", "1", "40")]), "0");
    assert_eq!(code(&i), "input.field_unknown");
    i = input("2026-27", json!([line(d, "spirit", "1", "40")]));
    i["duty_paid"] = json!(false);
    assert_eq!(code(&i), "input.field_unknown");
    let mut i = remitted("2026-27", json!([remission_line(d, "1")]), "-1");
    assert_eq!(code(&i), "input.amount_invalid");
    i["remission_used_this_year"] = json!("0");
    i["duty_paid"] = json!("yes");
    assert_eq!(code(&i), "input.flag_invalid");
}

#[test]
fn an_unknown_top_level_field_is_refused() {
    let mut i = input("2026-27", json!([line("2026-09-15", "spirit", "1", "40")]));
    i["remision"] = json!(true);
    assert_eq!(code(&i), "input.field_unknown");
}

/// The catalogue example: cans and a keg of 4.8% beer on 15 September 2026.
#[test]
fn the_catalogue_example() {
    let lines = json!([
        beer("2026-09-15", "90", "4.8", "0.375"),
        beer("2026-09-15", "50", "4.8", "50"),
    ]);
    let r = run(&input("2026-27", lines));
    assert_eq!(r["return_lines"][0]["duty"], "208.09");
    assert_eq!(r["return_lines"][1]["duty"], "78.10");
    assert_eq!(amount(&r, "excise_duty"), "286.19");
}
