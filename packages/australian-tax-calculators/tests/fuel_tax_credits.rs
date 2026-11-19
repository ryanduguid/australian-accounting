// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Fuel tax credits (Fuel Tax Act 2006 ss 41-5, 41-20, 43-5, 43-10 and 47-5)
//! at the ATO's published rates for fuel acquired from 1 July 2022, with the
//! basic method for heavy vehicles (PCG 2021/2) and PCG 2016/2 Method 2, read
//! on 19 November 2026 (docs/calculation-evidence.md#fuel-tax-credits). The
//! guidelines' own examples, at December 2020 rates outside the bundled
//! periods, are in the module's unit tests.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn line(fuel: &str, use_: &str, acquired: &str, quantity: &str) -> Value {
    json!({"fuel": fuel, "use": use_, "acquired": acquired, "quantity": quantity})
}

fn input(year: &str, lines: Value) -> Value {
    json!({"lines": lines, "year": year, "scope_confirmed": true})
}

fn run(input: &Value) -> Value {
    api::evaluate("fuel-tax-credits", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("fuel-tax-credits", input)
        .unwrap_err()
        .code
}

fn total(result: &Value) -> [&str; 2] {
    ["fuel_tax_credits", "label_7d"].map(|k| result["amounts"][k].as_str().unwrap())
}

/// The financial year a day falls in.
fn year_of(day: &str) -> String {
    let y: i32 = day[..4].parse().unwrap();
    let start = if &day[5..7] >= "07" { y } else { y - 1 };
    format!("{start}-{:02}", (start + 1) % 100)
}

/// Each period's first day and its liquid fuel rates in cents per litre,
/// heavy vehicle on a public road and all other business uses, typed from the
/// ATO's 'Rates - business' year pages (QC69964, QC72768, QC102637, QC105117
/// and QC107638), read on 19 November 2026.
const LIQUID: [(&str, &str, &str); 16] = [
    ("2022-07-01", "0", "22.1"),
    ("2022-08-01", "0", "23"),
    ("2022-09-29", "18.8", "46"),
    ("2023-02-01", "20.5", "47.7"),
    ("2023-07-01", "18.9", "47.7"),
    ("2023-08-01", "20", "48.8"),
    ("2024-02-05", "20.8", "49.6"),
    ("2024-07-01", "19.1", "49.6"),
    ("2024-08-05", "20.1", "50.6"),
    ("2025-02-03", "20.3", "50.8"),
    ("2025-07-01", "18.4", "50.8"),
    ("2025-08-04", "19.2", "51.6"),
    ("2026-02-02", "20.2", "52.6"),
    ("2026-04-01", "20.6", "20.6"),
    ("2026-07-01", "20.2", "36.6"),
    ("2026-08-03", "21.3", "53.7"),
];

/// 100 litres of diesel on each period's first day and the day before the
/// next, in each use, earns the period's rate in dollars.
#[test]
fn every_period_takes_the_ato_rate_for_the_day_of_acquisition() {
    for (i, (from, heavy, other)) in LIQUID.iter().enumerate() {
        let last = match LIQUID.get(i + 1) {
            Some((next, _, _)) => {
                let r = run(&input(
                    &year_of(next),
                    json!([line("diesel", "other_business", next, "1")]),
                ));
                r["lines"][0]["rate_period"]["from"]
                    .as_str()
                    .unwrap()
                    .to_string()
            }
            None => "2026-08-03".into(),
        };
        assert_eq!(last, LIQUID.get(i + 1).map_or("2026-08-03", |n| n.0));
        let r = run(&input(
            &year_of(from),
            json!([
                line("diesel", "heavy_vehicle_public_road", from, "100"),
                line("petrol", "other_business", from, "100"),
            ]),
        ));
        let lines = r["lines"].as_array().unwrap();
        assert_eq!(lines[0]["rate_cents"], json!(heavy), "{from}");
        assert_eq!(lines[1]["rate_cents"], json!(other), "{from}");
        assert_eq!(lines[0]["rate_period"]["from"], json!(from));
        let source = &lines[0]["rate_period"]["source"];
        assert_eq!(source["read"], json!("2026-11-19"));
        assert!(
            source["url"]
                .as_str()
                .unwrap()
                .starts_with("https://www.ato.gov.au/")
        );
    }
}

/// Gaseous fuels and blends from the same pages: LNG or CNG per kilogram,
/// the 1 April to 30 June 2026 heavy vehicle rates equal to the others (nil
/// road user charge), LNG's 3.2 c/kg from 1 July 2026 and B100's yearly rise.
#[test]
fn other_fuels_take_their_own_rows() {
    let cases = [
        (
            "e85",
            "heavy_vehicle_public_road",
            "2026-05-01",
            "8.87",
            "litres",
        ),
        (
            "lpg",
            "heavy_vehicle_public_road",
            "2026-05-01",
            "6.8",
            "litres",
        ),
        (
            "lng_cng",
            "heavy_vehicle_public_road",
            "2026-07-15",
            "3.2",
            "kilograms",
        ),
        (
            "lng_cng",
            "heavy_vehicle_public_road",
            "2026-08-03",
            "0",
            "kilograms",
        ),
        (
            "lng_cng",
            "other_business",
            "2026-08-03",
            "36.8",
            "kilograms",
        ),
        ("b100", "other_business", "2025-06-30", "15.2", "litres"),
        ("b100", "other_business", "2025-07-01", "16.9", "litres"),
        ("e10", "other_business", "2023-01-31", "46", "litres"),
        (
            "b20",
            "heavy_vehicle_public_road",
            "2022-09-28",
            "0",
            "litres",
        ),
    ];
    for (fuel, use_, day, rate, unit) in cases {
        let r = run(&input(&year_of(day), json!([line(fuel, use_, day, "10")])));
        assert_eq!(r["lines"][0]["rate_cents"], json!(rate), "{fuel} {day}");
        assert_eq!(r["lines"][0]["unit"], json!(unit), "{fuel}");
    }
}

/// 10,000 litres of diesel in a heavy vehicle on public roads and 2,000 litres
/// off road, from 3 August 2026: 10,000 x 21.3 / 100 + 2,000 x 53.7 / 100 =
/// $2,130 + $1,074 = $3,204, the catalogue's example.
#[test]
fn heavy_and_off_road_diesel_add_up_for_label_7d() {
    let r = run(&input(
        "2026-27",
        json!([
            line("diesel", "heavy_vehicle_public_road", "2026-09-15", "10000"),
            line("diesel", "other_business", "2026-09-15", "2000"),
        ]),
    ));
    assert_eq!(total(&r), ["3204.00", "3204.00"]);
    assert_eq!(r["lines"][0]["credit"], json!("2130.00"));
    assert_eq!(r["lines"][1]["credit"], json!("1074.00"));
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["fuel_tax_credits"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_eq!(total(&run(&facts)), ["3204.00", "3204.00"]);
}

/// Each line is rounded half up to the cent and label 7D drops the cents:
/// 300 L of E85 at 23.015 c/L is $69.045, so $69.05 (half even would give
/// $69.04), and 1,001 L of diesel at 53.7 c/L is $537.537, so $537.54; the
/// $606.59 total is claimed as $606.
#[test]
fn lines_round_half_up_and_label_7d_drops_the_cents() {
    let r = run(&input(
        "2026-27",
        json!([
            line("e85", "other_business", "2026-10-01", "300"),
            line("diesel", "other_business", "2026-10-01", "1001"),
        ]),
    ));
    assert_eq!(r["lines"][0]["credit"], json!("69.05"));
    assert_eq!(r["lines"][1]["credit"], json!("537.54"));
    assert_eq!(total(&r), ["606.59", "606.00"]);
}

#[test]
fn a_light_vehicle_on_a_public_road_gets_nil() {
    let r = run(&input(
        "2025-26",
        json!([line(
            "petrol",
            "light_vehicle_public_road",
            "2025-10-01",
            "500"
        )]),
    ));
    assert_eq!(r["lines"][0]["credit"], json!("0.00"));
    assert_eq!(r["lines"][0]["rate_cents"], json!("0"));
    let authority = r["lines"][0]["authority"].to_string();
    assert!(authority.contains("s 41-20"), "{authority}");
}

fn basic(fields: Value) -> Value {
    let mut line = json!({"method": "basic_heavy_vehicle", "fuel": "diesel",
                          "acquired": "2026-09-30", "litres": "11430"});
    for (k, v) in fields.as_object().unwrap() {
        line[k] = v.clone();
    }
    line
}

fn basic_input(line: Value) -> Value {
    json!({"lines": [line], "small_claimant": true, "year": "2026-27",
           "scope_confirmed": true})
}

/// Jim's figures (QC64655: 11,430 L, 32,000 km, a rigid truck carrying
/// 32 tonnes) at the rates from 3 August 2026: 31,488 derived km x 0.35 =
/// 11,020.8 L on public roads at 21.3 c/L = $2,347.43, and 409.2 L at
/// 53.7 c/L = $219.74; $2,567.17, claimed as $2,567.
#[test]
fn the_basic_method_splits_diesel_by_public_road_kilometres() {
    let r = run(&basic_input(basic(
        json!({"total_km": "32000", "rigid_truck_gross_weight_tonnes": "32"}),
    )));
    let l = &r["lines"][0];
    assert_eq!(l["public_road_km"], json!("31488"));
    assert_eq!(l["litres_per_km"], json!("0.35"));
    assert_eq!(l["public_road_litres"], json!("11020.8"));
    assert_eq!(l["other_litres"], json!("409.2"));
    assert_eq!(l["public_road_credit"], json!("2347.43"));
    assert_eq!(l["other_credit"], json!("219.74"));
    assert_eq!(total(&r), ["2567.17", "2567.00"]);
    // Actual kilometres and the engine's diagnostic rate, PCG 2021/2 para 9
    // and footnote 5: 1 / 2.03 km/L, given as 0.493 L/km.
    let r = run(&basic_input(basic(
        json!({"public_road_km": "20000", "litres_per_km": "0.493"}),
    )));
    assert_eq!(r["lines"][0]["public_road_litres"], json!("9860"));
    assert_eq!(r["lines"][0]["other_litres"], json!("1570"));
}

/// The rigid truck classes of QC64655 Table 1: each bound belongs to the
/// lighter class.
#[test]
fn rigid_truck_weights_take_their_class_rate() {
    for (weight, rate) in [
        ("4.501", "0.18"),
        ("12", "0.18"),
        ("12.001", "0.23"),
        ("16.5", "0.23"),
        ("24", "0.28"),
        ("30", "0.32"),
        ("30.001", "0.35"),
    ] {
        let r = run(&basic_input(basic(
            json!({"public_road_km": "100", "rigid_truck_gross_weight_tonnes": weight}),
        )));
        assert_eq!(r["lines"][0]["litres_per_km"], json!(rate), "{weight}");
    }
}

/// PCG 2016/2 Method 2 across the 2 February 2026 indexation: 1,000 L
/// acquired on 10 January 2026 takes 31 March's 52.6 c/L ($526), not its
/// own day's 51.6 c/L ($516).
#[test]
fn method_2_takes_the_rate_on_the_last_day_across_indexation() {
    let lines = json!([line("diesel", "other_business", "2026-01-10", "1000")]);
    let own = run(&input("2025-26", lines.clone()));
    assert_eq!(total(&own), ["516.00", "516.00"]);
    let r = run(&json!({"lines": lines, "small_claimant": true,
        "rate_at_period_end": {"start": "2026-01-01", "end": "2026-03-31"},
        "year": "2025-26", "scope_confirmed": true}));
    assert_eq!(total(&r), ["526.00", "526.00"]);
    assert_eq!(r["lines"][0]["rate_day"], json!("2026-03-31"));
    // A period that starts on the change day spans no change.
    let r = run(
        &json!({"lines": [line("diesel", "other_business", "2026-05-01", "1000")],
        "small_claimant": true,
        "rate_at_period_end": {"start": "2026-04-01", "end": "2026-06-30"},
        "year": "2025-26", "scope_confirmed": true}),
    );
    assert_eq!(total(&r), ["206.00", "206.00"]);
}

/// PCG 2016/2 Method 1: $2,459 of diesel at an average 245.9 c/L is 1,000 L
/// ($537 off road from 3 August 2026); $100 is 40.6669 L shown, and its
/// exact quotient x 53.7 c/L is $21.84.
#[test]
fn method_1_works_out_litres_from_the_cost_and_average_price() {
    let from_cost = |cost: &str| {
        json!({"method": "litres_from_cost", "fuel": "diesel", "use": "other_business",
               "acquired": "2026-09-30", "cost": cost, "average_price_cents_per_litre": "245.9"})
    };
    let r = run(
        &json!({"lines": [from_cost("2459.00"), from_cost("100.00")],
        "small_claimant": true, "year": "2026-27", "scope_confirmed": true}),
    );
    assert_eq!(r["lines"][0]["quantity"], json!("1000.0000"));
    assert_eq!(r["lines"][0]["credit"], json!("537.00"));
    assert_eq!(r["lines"][1]["quantity"], json!("40.6669"));
    assert_eq!(r["lines"][1]["credit"], json!("21.84"));
    assert!(
        r["lines"][1]["litres_from_cost_authority"]
            .as_str()
            .unwrap()
            .contains("aip.com.au")
    );
}

/// s 47-5(2): a return given on the 4th anniversary of the acquisition is in
/// time; one given the day after flags the line but keeps it.
#[test]
fn lines_older_than_4_years_at_the_return_are_flagged() {
    let with_return = |day: &str| {
        let mut i = input(
            "2022-23",
            json!([
                line("diesel", "other_business", "2022-11-19", "100"),
                line("diesel", "other_business", "2023-01-05", "100"),
            ]),
        );
        i["return_date"] = json!(day);
        run(&i)
    };
    assert_eq!(with_return("2026-11-19")["time_limit_lines"], json!([]));
    let r = with_return("2026-11-20");
    assert_eq!(r["time_limit_lines"], json!(["1"]));
    assert_eq!(total(&r), ["92.00", "92.00"]);
    let warnings = r["warnings"].to_string();
    assert!(
        warnings.contains("s 47-5(2)") && warnings.contains("Linfox"),
        "{warnings}"
    );
}

#[test]
fn fuel_acquired_after_the_rates_were_read_is_warned_about() {
    let r = run(&input(
        "2026-27",
        json!([line("diesel", "other_business", "2027-01-31", "1")]),
    ));
    assert!(
        r["warnings"]
            .to_string()
            .contains("when the rates were last read")
    );
    let r = run(&input(
        "2026-27",
        json!([line("diesel", "other_business", "2026-11-19", "1")]),
    ));
    assert!(
        !r["warnings"]
            .to_string()
            .contains("when the rates were last read")
    );
}

#[test]
fn refusals_name_what_is_out_of_scope_or_wrong() {
    let one = |year: &str, l: Value| input(year, json!([l]));
    let diesel = |day: &str| line("diesel", "other_business", day, "100");
    let method_2 = |start: &str, end: &str, l: Value| {
        json!({"lines": [l], "small_claimant": true,
               "rate_at_period_end": {"start": start, "end": end},
               "year": "2026-27", "scope_confirmed": true})
    };
    let mut too_many = input("2026-27", json!([]));
    too_many["lines"] = Value::Array(vec![diesel("2026-09-01"); 1001]);
    let cases = [
        (one("2021-22", diesel("2022-06-30")), "period.unsupported"),
        (one("2027-28", diesel("2027-07-01")), "period.unsupported"),
        (
            one("2025-26", diesel("2026-07-01")),
            "fuel_tax_credits.date_outside_year",
        ),
        (
            one("2026-27", diesel("2027-02-01")),
            "fuel_tax_credits.rate_unpublished",
        ),
        (one("2026-27", diesel("2026-02-30")), "input.date_invalid"),
        (
            one(
                "2026-27",
                line("avgas", "other_business", "2026-09-01", "1"),
            ),
            "fuel_tax_credits.fuel_invalid",
        ),
        (
            one("2026-27", line("diesel", "private", "2026-09-01", "1")),
            "fuel_tax_credits.use_invalid",
        ),
        (
            one(
                "2026-27",
                line("diesel", "other_business", "2026-09-01", "-1"),
            ),
            "fuel_tax_credits.quantity_invalid",
        ),
        (
            one(
                "2026-27",
                line("diesel", "other_business", "2026-09-01", "1.23456"),
            ),
            "fuel_tax_credits.quantity_invalid",
        ),
        (
            one(
                "2026-27",
                json!({"fuel": "diesel", "use": "other_business", "acquired": "2026-09-01", "quantity": 5}),
            ),
            "fuel_tax_credits.quantity_invalid",
        ),
        (
            one(
                "2026-27",
                json!({"fuel": "diesel", "use": "other_business", "acquired": "2026-09-01"}),
            ),
            "input.missing_field",
        ),
        (
            one(
                "2026-27",
                json!({"fuel": "diesel", "use": "other_business", "acquired": "2026-09-01", "quantity": "1", "km": "2"}),
            ),
            "input.field_unknown",
        ),
        (
            one("2026-27", json!("diesel")),
            "fuel_tax_credits.lines_invalid",
        ),
        (
            input("2026-27", json!([])),
            "fuel_tax_credits.lines_invalid",
        ),
        (too_many, "fuel_tax_credits.lines_invalid"),
        (
            input("2026-27", json!({"fuel": "diesel"})),
            "fuel_tax_credits.lines_invalid",
        ),
        // The basic method.
        (
            one(
                "2026-27",
                basic(json!({"public_road_km": "1", "litres_per_km": "0.3"})),
            ),
            "input.missing_field",
        ),
        (
            basic_input(basic(
                json!({"method": "per_hour", "public_road_km": "1", "litres_per_km": "0.3"}),
            )),
            "fuel_tax_credits.method_invalid",
        ),
        (
            basic_input(basic(
                json!({"fuel": "petrol", "public_road_km": "1", "litres_per_km": "0.3"}),
            )),
            "fuel_tax_credits.simplified_method_unavailable",
        ),
        (
            basic_input(basic(
                json!({"public_road_km": "1", "rigid_truck_gross_weight_tonnes": "4.5"}),
            )),
            "fuel_tax_credits.simplified_method_unavailable",
        ),
        (
            basic_input(basic(json!({"public_road_km": "1", "litres_per_km": "0"}))),
            "fuel_tax_credits.quantity_invalid",
        ),
        (
            basic_input(basic(
                json!({"public_road_km": "1", "total_km": "1", "litres_per_km": "0.3"}),
            )),
            "fuel_tax_credits.fact_conflict",
        ),
        (
            basic_input(basic(json!({"litres_per_km": "0.3"}))),
            "fuel_tax_credits.fact_conflict",
        ),
        (
            basic_input(basic(
                json!({"public_road_km": "40000", "litres_per_km": "0.35"}),
            )),
            "fuel_tax_credits.fact_conflict",
        ),
        (
            basic_input(basic(
                json!({"public_road_km": "1", "litres_per_km": "0.3", "use": "other_business"}),
            )),
            "input.field_unknown",
        ),
        // Method 1.
        (
            json!({"lines": [{"method": "litres_from_cost", "fuel": "lpg", "use": "other_business", "acquired": "2026-09-01", "cost": "10.00", "average_price_cents_per_litre": "100"}], "small_claimant": true, "year": "2026-27", "scope_confirmed": true}),
            "fuel_tax_credits.simplified_method_unavailable",
        ),
        (
            json!({"lines": [{"method": "litres_from_cost", "fuel": "diesel", "use": "other_business", "acquired": "2026-09-01", "cost": "10.00", "average_price_cents_per_litre": "0"}], "small_claimant": true, "year": "2026-27", "scope_confirmed": true}),
            "fuel_tax_credits.quantity_invalid",
        ),
        (
            json!({"lines": [{"method": "litres_from_cost", "fuel": "diesel", "use": "other_business", "acquired": "2026-09-01", "cost": "10.001", "average_price_cents_per_litre": "200"}], "small_claimant": true, "year": "2026-27", "scope_confirmed": true}),
            "input.amount_invalid",
        ),
        (
            json!({"lines": [{"method": "litres_from_cost", "fuel": "diesel", "use": "other_business", "acquired": "2026-09-01", "average_price_cents_per_litre": "200"}], "small_claimant": true, "year": "2026-27", "scope_confirmed": true}),
            "input.missing_field",
        ),
        (
            json!({"lines": [{"method": "litres_from_cost", "fuel": "diesel", "use": "other_business", "acquired": "2026-09-01", "cost": "10.00", "average_price_cents_per_litre": "200"}], "year": "2026-27", "scope_confirmed": true}),
            "input.missing_field",
        ),
        (
            json!({"lines": [{"method": "litres_from_cost", "fuel": "diesel", "use": "other_business", "acquired": "2026-09-01", "cost": "10.00", "average_price_cents_per_litre": "200", "quantity": "5"}], "small_claimant": true, "year": "2026-27", "scope_confirmed": true}),
            "input.field_unknown",
        ),
        // The small claimant statement.
        (
            {
                let mut i = basic_input(basic(
                    json!({"public_road_km": "1", "litres_per_km": "0.3"}),
                ));
                i["small_claimant"] = json!(false);
                i
            },
            "fuel_tax_credits.simplified_method_unavailable",
        ),
        (
            {
                let mut i = basic_input(basic(
                    json!({"public_road_km": "1", "litres_per_km": "0.3"}),
                ));
                i["small_claimant"] = json!("yes");
                i
            },
            "input.flag_invalid",
        ),
        (
            {
                let mut i = basic_input(basic(
                    json!({"public_road_km": "1", "litres_per_km": "0.3"}),
                ));
                i.as_object_mut().unwrap().remove("small_claimant");
                i
            },
            "input.missing_field",
        ),
        (
            {
                let mut i = one("2026-27", diesel("2026-09-01"));
                i["small_claimant"] = json!(true);
                i
            },
            "input.field_unknown",
        ),
        // Method 2.
        (
            method_2("2026-07-01", "2026-09-30", diesel("2026-07-10")),
            "fuel_tax_credits.simplified_method_unavailable",
        ),
        (
            method_2(
                "2026-10-01",
                "2026-12-31",
                line("lpg", "other_business", "2026-10-10", "1"),
            ),
            "fuel_tax_credits.simplified_method_unavailable",
        ),
        (
            method_2("2026-10-01", "2026-12-31", diesel("2026-09-30")),
            "fuel_tax_credits.fact_conflict",
        ),
        (
            method_2("2026-12-31", "2026-10-01", diesel("2026-10-10")),
            "fuel_tax_credits.fact_conflict",
        ),
        (
            method_2("2026-10-01", "2027-07-31", diesel("2026-10-10")),
            "fuel_tax_credits.fact_conflict",
        ),
        (
            method_2("2026-10-01", "2026-13-01", diesel("2026-10-10")),
            "input.date_invalid",
        ),
        (
            {
                let mut i = method_2("2026-10-01", "2026-12-31", diesel("2026-10-10"));
                i["rate_at_period_end"] = json!("2026-12-31");
                i
            },
            "fuel_tax_credits.period_invalid",
        ),
        (
            {
                let mut i = method_2("2026-10-01", "2026-12-31", diesel("2026-10-10"));
                i["rate_at_period_end"]["days"] = json!(92);
                i
            },
            "input.field_unknown",
        ),
        // The return's day.
        (
            {
                let mut i = one("2026-27", diesel("2026-09-01"));
                i["return_date"] = json!("2026-08-31");
                i
            },
            "fuel_tax_credits.fact_conflict",
        ),
        (
            {
                let mut i = one("2026-27", diesel("2026-09-01"));
                i["return_date"] = json!(20261028);
                i
            },
            "input.date_invalid",
        ),
        (
            {
                let mut i = one("2026-27", diesel("2026-09-01"));
                i["scope_confirmed"] = json!(false);
                i
            },
            "scope.not_confirmed",
        ),
        (
            {
                let mut i = one("2026-27", diesel("2026-09-01"));
                i["vehicle"] = json!("truck");
                i
            },
            "input.field_unknown",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(code(&input), expected, "{input}");
    }
    // The Method 2 refusal names the change day and why.
    let refused = api::evaluate(
        "fuel-tax-credits",
        &method_2("2026-07-01", "2026-09-30", diesel("2026-07-10")),
    )
    .unwrap_err();
    assert!(
        refused.contains("2026-08-03") && refused.contains("indexation"),
        "{refused}"
    );
}
