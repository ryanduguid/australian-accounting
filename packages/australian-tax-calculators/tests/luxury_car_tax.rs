// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Luxury car tax (A New Tax System (Luxury Car Tax) Act 1999 ss 5-15, 7-15,
//! 15-20 to 15-30, 18-5, 18-10 and 25-1) on the ATO's worked examples and
//! published thresholds, read on 19 November 2026
//! (docs/calculation-evidence.md#luxury-car-tax).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(year: &str, lines: Value) -> Value {
    json!({"lines": lines, "year": year, "scope_confirmed": true})
}

fn run(input: &Value) -> Value {
    api::evaluate("luxury-car-tax", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("luxury-car-tax", input)
        .unwrap_err()
        .code
}

fn sale(value: &str, rating: Value) -> Value {
    json!({"kind": "sale", "luxury_car_tax_value": value, "fuel_consumption_litres_per_100km": rating})
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

fn field<'a>(result: &'a Value, n: usize, key: &str) -> &'a str {
    result["lines"][n][key].as_str().unwrap()
}

/// The ATO's 'Working out the LCT on a sale' (QC22107): Matty Bee Motors
/// sells a car, not fuel efficient, for $88,000 including GST in 2026-27:
/// ($88,000 - $80,809) x 10/11 x 33% = $2,157.30, charged at $90,157.30.
#[test]
fn a_sale_takes_33_percent_of_the_gst_exclusive_excess() {
    let r = run(&input("2026-27", json!([sale("88000.00", json!("8.5"))])));
    assert_eq!(field(&r, 0, "threshold"), "80809");
    assert_eq!(field(&r, 0, "threshold_kind"), "general");
    assert_eq!(field(&r, 0, "value_over_threshold"), "7191.00");
    assert_eq!(field(&r, 0, "luxury_car_tax"), "2157.30");
    assert_eq!(field(&r, 0, "price_including_lct"), "90157.30");
    assert_eq!(amount(&r, "lct_on_sales"), "2157.30");
    assert_eq!(amount(&r, "label_1e"), "2157.00");
    assert_eq!(amount(&r, "label_1f"), "0.00");
    assert_eq!(r["rates"]["lct_rate"], "0.33");
    assert_eq!(r["rates"]["luxury_car_tax_threshold"], "80809");
    assert_eq!(r["rates"]["fuel_efficient_car_limit"], "91661");
}

/// QC22108: Jimmy imports a 1977 Mustang in 2026-27 with an LCT value of
/// $101,500: $6,207.30, paid to the Department of Home Affairs, not at 1E.
#[test]
fn an_importation_is_paid_to_home_affairs() {
    let line = json!({"kind": "importation", "luxury_car_tax_value": "101500.00", "fuel_consumption_litres_per_100km": null});
    let r = run(&input("2026-27", json!([line])));
    assert_eq!(field(&r, 0, "luxury_car_tax"), "6207.30");
    assert_eq!(field(&r, 0, "payable_to"), "home_affairs");
    assert_eq!(amount(&r, "lct_on_importations"), "6207.30");
    assert_eq!(amount(&r, "label_1e"), "0.00");
}

/// Each financial year's 2 thresholds as QC38161 publishes them, and the
/// Library's ¶12-340 figures for 2025-26 ($91,387 and $80,567): a car worth
/// $100,000 rated 3 litres, then one with no rating.
#[test]
fn every_published_year_takes_its_own_thresholds() {
    let table = [
        ("2019-20", "75526", "67525"),
        ("2020-21", "77565", "68740"),
        ("2021-22", "79659", "69152"),
        ("2022-23", "84916", "71849"),
        ("2023-24", "89332", "76950"),
        ("2024-25", "91387", "80567"),
        ("2025-26", "91387", "80567"),
        ("2026-27", "91661", "80809"),
    ];
    for (year, efficient, general) in table {
        let r = run(&input(
            year,
            json!([
                sale("100000.00", json!("3")),
                sale("100000.00", Value::Null)
            ]),
        ));
        assert_eq!(field(&r, 0, "threshold"), efficient, "{year}");
        assert_eq!(field(&r, 0, "threshold_kind"), "fuel_efficient", "{year}");
        assert_eq!(field(&r, 1, "threshold"), general, "{year}");
        assert_eq!(field(&r, 1, "threshold_kind"), "general", "{year}");
    }
    // ($86,790 - $80,567) x 0.3, the Library's 2025-26 car if it were taxable.
    let r = run(&input("2025-26", json!([sale("86790.00", Value::Null)])));
    assert_eq!(field(&r, 0, "luxury_car_tax"), "1866.90");
}

/// From 1 July 2025 the fuel-efficient rating is 3.5 litres per 100 km, not 7
/// (Act No. 29 of 2025 Sch 1 items 2 and 4(1)), unless before that day the
/// car was supplied or imported and used other than for a quotable purpose
/// (item 4(2)). A $95,000 car rated 5 litres: $4,257.30 in 2026-27 at the
/// general threshold, $1,001.70 under item 4(2) and at 3.5 litres; an
/// adjustment on a 2024-25 sale keeps the 7 litre rating.
#[test]
fn the_fuel_efficient_rating_fell_to_3_5_litres_from_1_july_2025() {
    let mut used = sale("95000.00", json!("5"));
    used["used_before_1_july_2025"] = json!(true);
    let mut new = sale("95000.00", json!("5"));
    new["used_before_1_july_2025"] = json!(false);
    let r = run(&input(
        "2026-27",
        json!([
            new,
            used,
            sale("95000.00", json!("3.5")),
            sale("95000.00", json!("7.01"))
        ]),
    ));
    assert_eq!(field(&r, 0, "luxury_car_tax"), "4257.30");
    assert_eq!(field(&r, 1, "luxury_car_tax"), "1001.70");
    assert!(field(&r, 1, "threshold_authority").contains("item 4(2)"));
    assert_eq!(field(&r, 2, "luxury_car_tax"), "1001.70");
    assert_eq!(field(&r, 3, "threshold_kind"), "general");
    let old = json!({
        "kind": "price_adjustment", "supplied_in": "2024-25",
        "corrected_luxury_car_tax_value": "95000.00",
        "fuel_consumption_litres_per_100km": "7", "previously_attributed_lct": "0",
    });
    let r = run(&input("2026-27", json!([old])));
    assert_eq!(field(&r, 0, "threshold"), "91387");
    assert_eq!(field(&r, 0, "increasing_adjustment"), "1083.90");
}

/// s 5-15(2): a sale's tax less tax payable on an earlier supply or
/// importation of the car, not below nil; nil at or below the threshold.
#[test]
fn earlier_tax_on_the_car_reduces_a_sale_and_nil_below_the_threshold() {
    let mut second = sale("100000.00", Value::Null);
    second["lct_previously_payable"] = json!("2000.00");
    let mut more = sale("100000.00", Value::Null);
    more["lct_previously_payable"] = json!("6000.00");
    let r = run(&input(
        "2026-27",
        json!([second, more, sale("80809.00", Value::Null)]),
    ));
    assert_eq!(field(&r, 0, "luxury_car_tax"), "5757.30");
    assert_eq!(field(&r, 0, "luxury_car_tax_payable"), "3757.30");
    assert_eq!(field(&r, 1, "luxury_car_tax_payable"), "0.00");
    assert_eq!(field(&r, 2, "luxury_car_tax"), "0.00");
    assert_eq!(amount(&r, "lct_on_sales"), "3757.30");
}

/// ss 15-20 and 15-25: the corrected tax on a 2025-26 sale against the tax
/// attributed; s 15-30: change of use. 1E takes sales and increasing
/// adjustments, 1F decreasing ones, each rounded down.
#[test]
fn adjustments_go_to_1e_and_1f() {
    let cut = json!({
        "kind": "price_adjustment", "supplied_in": "2025-26",
        "corrected_luxury_car_tax_value": "85000.00",
        "fuel_consumption_litres_per_100km": null, "previously_attributed_lct": "2157.30",
    });
    let rise = json!({
        "kind": "price_adjustment", "supplied_in": "2025-26",
        "corrected_luxury_car_tax_value": "90000.00",
        "fuel_consumption_litres_per_100km": null, "previously_attributed_lct": "2000.00",
    });
    let quoted = json!({
        "kind": "change_of_use_increasing", "supplied_in": "2025-26",
        "luxury_car_tax_value": "100000.00", "fuel_consumption_litres_per_100km": null,
    });
    let back = json!({"kind": "change_of_use_decreasing", "lct_payable_on_supply": "5829.90"});
    let r = run(&input(
        "2026-27",
        json!([cut, rise, quoted, back, sale("88000.00", Value::Null)]),
    ));
    assert_eq!(field(&r, 0, "corrected_luxury_car_tax"), "1329.90");
    assert_eq!(field(&r, 0, "decreasing_adjustment"), "827.40");
    assert_eq!(field(&r, 1, "increasing_adjustment"), "829.90");
    assert_eq!(field(&r, 2, "increasing_adjustment"), "5829.90");
    assert_eq!(field(&r, 3, "decreasing_adjustment"), "5829.90");
    assert_eq!(amount(&r, "increasing_adjustments"), "6659.80");
    assert_eq!(amount(&r, "decreasing_adjustments"), "6657.30");
    // $2,157.30 + $6,659.80 = $8,817.10; $6,657.30.
    assert_eq!(amount(&r, "label_1e"), "8817.00");
    assert_eq!(amount(&r, "label_1f"), "6657.00");
}

/// ss 18-5 and 18-10: the lesser of the tax borne and $10,000, not on the
/// BAS (QC39467); one primary producer car a financial year (s 18-5(4)).
#[test]
fn refunds_are_capped_at_10000_and_kept_off_the_bas() {
    let refund = |claimant: &str, eligible: bool, borne: &str| json!({"kind": "refund", "claimant": claimant, "refund_eligible_car": eligible, "lct_borne": borne});
    let r = run(&input(
        "2026-27",
        json!([
            refund("primary_producer", true, "12000.00"),
            refund("tourism_operator", true, "4000.00"),
            refund("tourism_operator", true, "15000.00"),
            refund("primary_producer", false, "3000.00"),
        ]),
    ));
    assert_eq!(field(&r, 0, "refund"), "10000.00");
    assert_eq!(field(&r, 1, "refund"), "4000.00");
    assert_eq!(field(&r, 2, "refund"), "10000.00");
    assert_eq!(field(&r, 3, "refund"), "0.00");
    assert_eq!(amount(&r, "refunds"), "24000.00");
    assert_eq!(amount(&r, "label_1e"), "0.00");
    assert_eq!(amount(&r, "label_1f"), "0.00");
    let two = json!([
        refund("primary_producer", true, "12000.00"),
        refund("primary_producer", true, "1000.00"),
    ]);
    assert_eq!(code(&input("2026-27", two)), "luxury_car_tax.refund_limit");
    let mut early = refund("tourism_operator", true, "4000.00");
    early["supplied_in"] = json!("2018-19");
    assert_eq!(
        code(&input("2026-27", json!([early]))),
        "luxury_car_tax.refund_invalid"
    );
    let stranger = refund("dealer", true, "4000.00");
    assert_eq!(
        code(&input("2026-27", json!([stranger]))),
        "luxury_car_tax.refund_invalid"
    );
}

#[test]
fn refuses_what_it_does_not_work_out() {
    let one = |line: Value| input("2026-27", json!([line]));
    assert_eq!(
        code(&input("2027-28", json!([sale("90000", Value::Null)]))),
        "period.unsupported"
    );
    assert_eq!(
        code(&input("2018-19", json!([sale("90000", Value::Null)]))),
        "period.unsupported"
    );
    assert_eq!(
        code(&input("2026-27", json!([]))),
        "luxury_car_tax.lines_invalid"
    );
    assert_eq!(
        code(&input("2026-27", json!({}))),
        "luxury_car_tax.lines_invalid"
    );
    assert_eq!(code(&one(json!("sale"))), "luxury_car_tax.line_invalid");
    assert_eq!(
        code(&one(json!({"kind": "lease"}))),
        "luxury_car_tax.kind_invalid"
    );
    for rating in [json!("101"), json!("3.555"), json!(5), json!("-1")] {
        assert_eq!(
            code(&one(sale("90000", rating))),
            "luxury_car_tax.consumption_invalid"
        );
    }
    assert_eq!(code(&one(sale("90000", json!("5")))), "input.missing_field");
    let mut needless = sale("90000", json!("8"));
    needless["used_before_1_july_2025"] = json!(true);
    assert_eq!(code(&one(needless)), "input.field_unknown");
    let mut extra = sale("90000", Value::Null);
    extra["supplied_in"] = json!("2025-26");
    assert_eq!(code(&one(extra)), "input.field_unknown");
    assert_eq!(code(&one(sale("-1", Value::Null))), "input.amount_invalid");
    assert_eq!(
        code(&one(
            json!({"kind": "sale", "fuel_consumption_litres_per_100km": null})
        )),
        "input.missing_field"
    );
    for year in ["2027-28", "2011-12", "2026"] {
        let line = json!({
            "kind": "change_of_use_increasing", "supplied_in": year,
            "luxury_car_tax_value": "100000.00", "fuel_consumption_litres_per_100km": null,
        });
        assert_eq!(code(&one(line)), "luxury_car_tax.year_invalid", "{year}");
    }
    let mut unknown = input("2026-27", json!([sale("90000", Value::Null)]));
    unknown["quote"] = json!(true);
    assert_eq!(code(&unknown), "input.field_unknown");
    let mut unconfirmed = input("2026-27", json!([sale("90000", Value::Null)]));
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(code(&unconfirmed), "scope.not_confirmed");
}
