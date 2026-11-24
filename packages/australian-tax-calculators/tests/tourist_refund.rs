// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The tourist refund scheme: GST Act s 168-5(1), GST Regulations 2019
//! Division 168 (compilation 5), WET Act s 25-5(1) and WET Regulations 2019
//! reg 25-5.03, read on 24 November 2026, with each boundary
//! (docs/calculation-evidence.md#tourist-refund).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const TR: &str = "tourist-refund";

fn item(date: &str, price: &str, gst: &str, goods: &str) -> Value {
    let mut out = json!({"date": date, "price": price, "gst": gst, "goods": goods,
                         "taxable_supply": true, "tax_invoice": true, "partly_consumed": false});
    if goods == "wine" {
        out["wine_tax_borne"] = json!(true);
        out["wine_tax_refunded_or_credited"] = json!(false);
    }
    out
}

fn claim(items: Value, overrides: Value) -> Value {
    let mut input = json!({
        "departure_date": "2026-03-20", "departing_by": "aircraft",
        "crew_or_in_charge_in_employment": false, "trs_port": true,
        "accompanied_baggage": true, "minutes_before_departure": 45,
        "verification_complied": true, "acquired_by_traveller": true,
        "supplier_registered": true, "items": items, "year": "2025-26",
        "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            input.as_object_mut().unwrap().remove(k);
        } else {
            input[k] = v.clone();
        }
    }
    input
}

/// The plain result, checked against `derive` byte for byte.
fn run(input: &Value) -> Value {
    let result = api::evaluate(TR, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": TR, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(TR, input).unwrap_err().code
}

fn amounts(r: &Value) -> [String; 4] {
    [
        "qualifying_price",
        "gst_refund",
        "wine_tax_refund",
        "total_refund",
    ]
    .map(|k| r["amounts"][k].as_str().unwrap().to_string())
}

/// Case A: one item of $330 bearing $30 of GST, 10 days before departure.
#[test]
fn one_item_over_the_threshold_refunds_its_gst() {
    let r = run(&claim(
        json!([item("2026-03-10", "330.00", "30.00", "general")]),
        json!({}),
    ));
    assert_eq!(r["eligible"], true);
    assert_eq!(r["failing_conditions"], json!([]));
    assert_eq!(r["conditions"].as_array().unwrap().len(), 8);
    assert_eq!(amounts(&r), ["330.00", "30.00", "0.00", "30.00"]);
}

/// Case B: wine bears 29% of half its GST-inclusive price (reg 25-5.03).
#[test]
fn wine_refunds_its_gst_and_the_wine_tax_borne() {
    let items = json!([
        item("2026-03-10", "330.00", "30.00", "general"),
        item("2026-03-10", "100.00", "9.09", "wine")
    ]);
    let r = run(&claim(items, json!({})));
    assert_eq!(amounts(&r), ["430.00", "39.09", "14.50", "53.59"]);
    assert_eq!(r["items"][1]["wine_tax_borne"], "14.5000");
}

/// Case H: the wine tax is not rounded to 5 cents, only rendered to the cent.
#[test]
fn wine_tax_is_rendered_to_the_cent_not_rounded_for_cash() {
    let items = json!([
        item("2026-03-10", "300.00", "27.27", "general"),
        item("2026-03-10", "49.99", "4.54", "wine")
    ]);
    let r = run(&claim(items, json!({})));
    assert_eq!(amounts(&r), ["349.99", "31.81", "7.25", "39.06"]);
    assert!(r["notes"][0].as_str().unwrap().contains("168-5.14"));
}

/// Case C and the boundary: $299.99 from 2 items fails; $300.00 qualifies.
#[test]
fn the_threshold_is_300_dollars_from_one_supplier() {
    let short = json!([
        item("2026-03-10", "150.00", "13.64", "general"),
        item("2026-03-11", "149.99", "13.64", "general")
    ]);
    let r = run(&claim(short, json!({})));
    assert_eq!(r["eligible"], false);
    assert_eq!(r["failing_conditions"], json!(["purchase_threshold"]));
    assert_eq!(amounts(&r), ["299.99", "0.00", "0.00", "0.00"]);
    let exact = json!([
        item("2026-03-10", "150.00", "13.64", "general"),
        item("2026-03-11", "150.00", "13.64", "general")
    ]);
    assert_eq!(amounts(&run(&claim(exact, json!({}))))[1], "27.28");
}

/// Case D: 60 days after acquisition qualifies, 61 does not; where the late
/// item alone would bring the total to $300, the command refuses.
#[test]
fn goods_leave_within_60_days_after_acquisition() {
    let items = json!([
        item("2026-01-19", "200.00", "18.18", "general"),
        item("2026-01-18", "200.00", "18.18", "general")
    ]);
    assert_eq!(
        code(&claim(items, json!({}))),
        "tourist_refund.threshold_unsettled"
    );
    let enough = json!([
        item("2026-01-19", "300.00", "27.27", "general"),
        item("2026-01-18", "200.00", "18.18", "general")
    ]);
    let r = run(&claim(enough, json!({})));
    assert_eq!(r["items"][0]["qualifies"], true);
    assert_eq!(
        r["items"][1]["failing_conditions"],
        json!(["within_60_days"])
    );
    assert_eq!(amounts(&r), ["300.00", "27.27", "0.00", "27.27"]);
    let short = json!([
        item("2026-01-19", "100.00", "9.09", "general"),
        item("2026-01-18", "100.00", "9.09", "general")
    ]);
    let r = run(&claim(short, json!({})));
    assert_eq!(r["failing_conditions"], json!(["purchase_threshold"]));
    let both = json!([
        item("2026-01-19", "200.00", "18.18", "general"),
        item("2026-03-20", "200.00", "18.18", "general")
    ]);
    assert_eq!(amounts(&run(&claim(both, json!({}))))[1], "36.36");
}

/// Case E: 30 minutes by air, 60 by sea (reg 168-5.10(3)).
#[test]
fn the_claim_is_made_30_or_60_minutes_before_departure() {
    let items = json!([item("2026-03-10", "330.00", "30.00", "general")]);
    for (by, minutes, met) in [
        ("aircraft", 30, true),
        ("aircraft", 29, false),
        ("ship", 60, true),
        ("ship", 59, false),
    ] {
        let r = run(&claim(
            items.clone(),
            json!({"departing_by": by, "minutes_before_departure": minutes}),
        ));
        assert_eq!(r["eligible"], met, "{by} {minutes}");
        if !met {
            assert_eq!(r["failing_conditions"], json!(["presented_in_time"]));
        }
    }
}

/// Case F and each other claim condition failing in turn.
#[test]
fn each_claim_condition_can_fail() {
    let items = json!([item("2026-03-10", "330.00", "30.00", "general")]);
    for (overrides, failing) in [
        (
            json!({"crew_or_in_charge_in_employment": true}),
            "not_crew_or_in_charge",
        ),
        (
            json!({"verification_complied": false}),
            "verification_complied",
        ),
        (
            json!({"acquired_by_traveller": false}),
            "acquired_by_traveller",
        ),
        (json!({"supplier_registered": false}), "supplier_registered"),
        (json!({"trs_port": false}), "trs_port"),
        (json!({"accompanied_baggage": false}), "accompanied_baggage"),
    ] {
        let r = run(&claim(items.clone(), overrides.clone()));
        assert_eq!(r["failing_conditions"], json!([failing]), "{overrides}");
        assert_eq!(amounts(&r)[3], "0.00", "{overrides}");
    }
}

/// Case G and each item condition: tobacco, other alcohol, wine without wine
/// tax borne, GST-free goods, no tax invoice and partly consumed goods.
#[test]
fn each_item_condition_can_fail() {
    let mut no_wet = item("2026-03-10", "100.00", "9.09", "wine");
    no_wet["wine_tax_borne"] = json!(false);
    let mut no_invoice = item("2026-03-10", "100.00", "9.09", "general");
    no_invoice["tax_invoice"] = json!(false);
    let mut gst_free = item("2026-03-10", "400.00", "0.00", "general");
    gst_free["taxable_supply"] = json!(false);
    let mut consumed = item("2026-03-10", "100.00", "9.09", "general");
    consumed["partly_consumed"] = json!(true);
    for (bad, failing) in [
        (
            item("2026-03-10", "400.00", "36.36", "tobacco"),
            "goods_kind",
        ),
        (
            item("2026-03-10", "400.00", "36.36", "other_alcohol"),
            "goods_kind",
        ),
        (no_wet, "goods_kind"),
        (gst_free, "taxable_supply"),
        (no_invoice, "tax_invoice"),
        (consumed, "not_partly_consumed"),
    ] {
        let items = json!([item("2026-03-10", "100.00", "9.09", "general"), bad]);
        let r = run(&claim(items, json!({})));
        assert_eq!(
            r["items"][1]["failing_conditions"],
            json!([failing]),
            "{failing}"
        );
        assert_eq!(r["failing_conditions"], json!(["purchase_threshold"]));
        assert_eq!(amounts(&r)[0], "100.00");
    }
}

/// The refusals: dates, years, values outside those taken and GST above one
/// eleventh of the price.
#[test]
fn the_refusals() {
    let ok = json!([item("2026-03-10", "330.00", "30.00", "general")]);
    let mut wine_flag = item("2026-03-10", "330.00", "30.00", "general");
    wine_flag["wine_tax_borne"] = json!(true);
    let mut not_taxable = item("2026-03-10", "330.00", "30.00", "general");
    not_taxable["taxable_supply"] = json!(false);
    let mut reduced = item("2026-03-10", "330.00", "30.00", "wine");
    reduced["wine_tax_refunded_or_credited"] = json!(true);
    let mut refunded_on_general = item("2026-03-10", "330.00", "30.00", "general");
    refunded_on_general["wine_tax_refunded_or_credited"] = json!(false);
    let mut extra = item("2026-03-10", "330.00", "30.00", "general");
    extra["abn"] = json!("1");
    for (input, expected) in [
        (
            claim(
                json!([item("2026-03-21", "330.00", "30.00", "general")]),
                json!({}),
            ),
            "tourist_refund.date_invalid",
        ),
        (
            claim(
                json!([item("2025-10-01", "330.00", "30.00", "general")]),
                json!({"departure_date": "2025-10-31"}),
            ),
            "tourist_refund.departure_not_covered",
        ),
        (
            claim(ok.clone(), json!({"departure_date": "2026-07-01"})),
            "tourist_refund.departure_outside_year",
        ),
        (
            claim(ok.clone(), json!({"departing_by": "train"})),
            "tourist_refund.fact_invalid",
        ),
        (
            claim(ok.clone(), json!({"crew_or_in_charge_in_employment": "no"})),
            "input.flag_invalid",
        ),
        (
            claim(ok.clone(), json!({"minutes_before_departure": -1})),
            "tourist_refund.fact_invalid",
        ),
        (
            claim(ok.clone(), json!({"minutes_before_departure": 10_081})),
            "tourist_refund.fact_invalid",
        ),
        (
            claim(ok.clone(), json!({"minutes_before_departure": "45"})),
            "tourist_refund.fact_invalid",
        ),
        (
            claim(
                json!([item("2026-03-10", "330.00", "30.01", "general")]),
                json!({}),
            ),
            "tourist_refund.gst_exceeds_price_share",
        ),
        (
            claim(
                json!([item("2026-03-10", "330.00", "30.00", "food")]),
                json!({}),
            ),
            "tourist_refund.fact_invalid",
        ),
        (claim(json!([]), json!({})), "tourist_refund.item_invalid"),
        (
            claim(json!(["item"]), json!({})),
            "tourist_refund.item_invalid",
        ),
        (
            claim(json!([wine_flag]), json!({})),
            "tourist_refund.item_invalid",
        ),
        (
            claim(json!([not_taxable]), json!({})),
            "tourist_refund.item_invalid",
        ),
        (
            claim(json!([refunded_on_general]), json!({})),
            "tourist_refund.item_invalid",
        ),
        (
            claim(json!([reduced]), json!({})),
            "tourist_refund.wine_tax_reduced",
        ),
        (claim(json!([extra]), json!({})), "input.field_unknown"),
        (
            claim(ok.clone(), json!({"trs_port": "yes"})),
            "input.flag_invalid",
        ),
        (
            claim(json!([{"date": "2026-03-10"}]), json!({})),
            "input.missing_field",
        ),
        (
            claim(
                json!([item("2026-3-10", "330.00", "30.00", "general")]),
                json!({}),
            ),
            "input.date_invalid",
        ),
        (
            claim(
                json!([item("2026-03-10", "330.001", "30.00", "general")]),
                json!({}),
            ),
            "input.amount_invalid",
        ),
        (
            claim(ok.clone(), json!({"year": "2024-25"})),
            "period.unsupported",
        ),
        (
            claim(ok.clone(), json!({"scope_confirmed": false})),
            "scope.not_confirmed",
        ),
        (
            claim(ok.clone(), json!({"not_an_input": true})),
            "input.field_unknown",
        ),
    ] {
        assert_eq!(code(&input), expected, "{input}");
    }
    let mut no_wine_flag = item("2026-03-10", "330.00", "30.00", "wine");
    no_wine_flag
        .as_object_mut()
        .unwrap()
        .remove("wine_tax_borne");
    assert_eq!(
        code(&claim(json!([no_wine_flag]), json!({}))),
        "input.missing_field"
    );
    let many: Vec<Value> = (0..1001)
        .map(|_| item("2026-03-10", "1.00", "0.09", "general"))
        .collect();
    assert_eq!(
        code(&claim(json!(many), json!({}))),
        "tourist_refund.item_invalid"
    );
}

/// The first and last departures covered: 1 November 2025 and 30 June 2027.
#[test]
fn the_years_covered() {
    let first = claim(
        json!([item("2025-11-01", "330.00", "30.00", "general")]),
        json!({"departure_date": "2025-11-01"}),
    );
    assert_eq!(run(&first)["eligible"], true);
    let last = claim(
        json!([item("2027-06-30", "330.00", "30.00", "general")]),
        json!({"departure_date": "2027-06-30", "year": "2026-27"}),
    );
    assert_eq!(run(&last)["eligible"], true);
}

/// The catalogue's example runs and gives its stated refund.
#[test]
fn the_catalogue_example_runs() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["tourist_refund"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    facts["scope_confirmed"] = json!(true);
    assert_eq!(amounts(&run(&facts))[3], "53.59");
}

/// GST Act s 9-90: a taxable 5-cent item whose GST rounds to nil still counts
/// towards $300; a GST-free one does not.
#[test]
fn a_taxable_item_with_gst_rounded_to_nil_counts() {
    let items = json!([
        item("2026-03-10", "299.95", "27.27", "general"),
        item("2026-03-10", "0.05", "0.00", "general")
    ]);
    let r = run(&claim(items.clone(), json!({})));
    assert_eq!(amounts(&r), ["300.00", "27.27", "0.00", "27.27"]);
    let mut free = items;
    free[1]["taxable_supply"] = json!(false);
    let r = run(&claim(free, json!({})));
    assert_eq!(r["failing_conditions"], json!(["purchase_threshold"]));
}

/// Wine tax is summed before display: 2 x $0.145 is $0.29, not $0.30; a
/// refund above $200 is not capped.
#[test]
fn wine_tax_is_summed_before_display_and_not_capped() {
    let items = json!([
        item("2026-03-10", "300.00", "27.27", "general"),
        item("2026-03-10", "1.00", "0.09", "wine"),
        item("2026-03-10", "1.00", "0.09", "wine")
    ]);
    assert_eq!(amounts(&run(&claim(items, json!({}))))[2], "0.29");
    let big = json!([item("2026-03-10", "2500.00", "227.27", "general")]);
    assert_eq!(amounts(&run(&claim(big, json!({}))))[3], "227.27");
}

/// Late wine still bears wine tax, though none is refunded; a failing claim
/// condition with wine gives nil of both.
#[test]
fn late_wine_bears_wine_tax_without_a_refund() {
    let items = json!([
        item("2026-03-10", "330.00", "30.00", "general"),
        item("2026-01-01", "100.00", "9.09", "wine")
    ]);
    let r = run(&claim(items.clone(), json!({})));
    assert_eq!(r["items"][1]["qualifies"], false);
    assert_eq!(r["items"][1]["wine_tax_borne"], "14.5000");
    assert_eq!(amounts(&r)[2], "0.00");
    let mut fresh = items;
    fresh[1]["date"] = json!("2026-03-10");
    let r = run(&claim(fresh, json!({"trs_port": false})));
    assert_eq!(amounts(&r)[1..], ["0.00", "0.00", "0.00"]);
}

/// A covered departure on 1 November 2025 takes goods bought in October.
#[test]
fn acquisitions_before_the_first_covered_departure_count() {
    let r = run(&claim(
        json!([item("2025-10-01", "330.00", "30.00", "general")]),
        json!({"departure_date": "2025-11-01"}),
    ));
    assert_eq!(r["eligible"], true);
}
