// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst-food-retailer-sam` against the ATO's worked examples in 'GST for food
//! retailers - simplified accounting methods' and figures worked by hand from
//! F2017L01274, F2025L01055 and F2026L01275, written from the spec, not the
//! code.

use australian_tax_calculators::api;
use serde_json::{Value, json};

const C: &str = "gst-food-retailer-sam";

fn retail(method: &str) -> Value {
    let mut e = json!({"same_premises_taxable_and_gst_free_food": true,
        "sam_turnover": "1500000.00", "adequate_point_of_sale_equipment": false});
    if method.ends_with("five_percent") {
        e["projected_gst_free_at_most_5_percent"] = json!(true);
    }
    if method == "business_norms" {
        e["retailer_type_definition_met"] = json!(true);
    }
    e
}

fn supermarket() -> Value {
    json!({"registered_throughout": true, "mainly_supermarket_or_convenience_range": true,
        "converted_taxable_sales_under_5_percent": true, "gst_turnover": "1800000.00",
        "adequate_point_of_sale_equipment": true, "mainly_sells_fuel": false,
        "section_7_3_goods_left_out": true})
}

fn restaurant() -> Value {
    json!({"registered_throughout": true, "restaurant_cafe_or_catering": true,
        "gst_turnover": "900000.00"})
}

/// A September 2025 quarter under `method` with `extra` merged in.
fn input(method: &str, sales: &str, purchases: &str, extra: Value) -> Value {
    let mut v = json!({
        "method": method, "tax_period_start": "2025-07-01", "tax_period_end": "2025-09-30",
        "total_sales": sales, "trading_stock_purchases": purchases,
        "eligibility": retail(method),
        "year": "2025-26", "scope_confirmed": true,
    });
    for (key, value) in extra.as_object().unwrap() {
        if value.is_null() {
            v.as_object_mut().unwrap().remove(key);
        } else {
            v[key] = value.clone();
        }
    }
    v
}

fn run(input: &Value) -> Value {
    let result = api::evaluate(C, input).unwrap();
    assert_eq!(result["ok"], json!(true), "{result}");
    let doc = api::evaluate("derive", &json!({"command": C, "input": input})).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn amounts(input: &Value) -> Value {
    run(input)["amounts"].clone()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(C, input).unwrap_err().code
}

fn sample(start: &str, end: &str, gst_free: &str, total: &str) -> Value {
    json!({"start": start, "end": end, "gst_free": gst_free, "total": total})
}

#[test]
fn business_norms_carlo_hot_bread_shop() {
    let a = amounts(&input(
        "business_norms",
        "120000.00",
        "95000.00",
        json!({"retailer_type": "hot_bread_shop"}),
    ));
    assert_eq!(a["gst_free_sales"], "60000.00");
    assert_eq!(a["other_sales"], "60000.00");
    assert_eq!(a["gst_free_purchases"], "71250.00");
    assert_eq!(a["other_purchases"], "23750.00");
}

#[test]
fn every_business_norm_in_the_table() {
    // (type, role, sales %, purchases %) of F2017L01274 s 5(1), on $10,000 each.
    let rows = [
        ("cake_shop", None, "200.00", "9500.00"),
        ("continental_delicatessen", None, "8500.00", "9000.00"),
        ("convenience_store", Some("converter"), "2250.00", "3000.00"),
        ("convenience_store", Some("reseller"), "3000.00", "3000.00"),
        ("fresh_fish_shop", None, "3500.00", "9800.00"),
        ("health_food_shop", None, "3500.00", "3500.00"),
        ("hot_bread_shop", None, "5000.00", "7500.00"),
    ];
    for (kind, role, s, p) in rows {
        let mut extra = json!({"retailer_type": kind});
        if let Some(role) = role {
            extra["stock_role"] = json!(role);
        }
        let a = amounts(&input("business_norms", "10000.00", "10000.00", extra));
        assert_eq!((kind, a["gst_free_sales"].as_str().unwrap()), (kind, s));
        assert_eq!((kind, a["gst_free_purchases"].as_str().unwrap()), (kind, p));
    }
}

#[test]
fn rural_convenience_store_deducts_fuel_and_post_first() {
    let a = amounts(&input(
        "business_norms",
        "50000.00",
        "40000.00",
        json!({"retailer_type": "rural_convenience_store", "stock_role": "converter",
               "fuel_and_post_sales": "20000.00", "fuel_and_post_purchases": "18000.00"}),
    ));
    // (50,000 - 20,000) x 22.5% and (40,000 - 18,000) x 30%.
    assert_eq!(a["gst_free_sales"], "6750.00");
    assert_eq!(a["gst_free_purchases"], "6600.00");
}

#[test]
fn pharmacy_components() {
    let a = amounts(&input(
        "business_norms",
        "100000.00",
        "60000.00",
        json!({"retailer_type": "pharmacy", "pbs_sales": "40000.00", "non_pbs_sales": "10000.00",
               "otc_sales": "50000.00", "non_pbs_purchases": "10000.00", "otc_purchases": "50000.00"}),
    ));
    // 40,000 + 98% of 10,000 + 47.5% of 50,000; 0% of 10,000 + 2% of 50,000.
    assert_eq!(a["gst_free_sales"], "73550.00");
    assert_eq!(a["gst_free_purchases"], "1000.00");
}

#[test]
fn stock_purchases_every_period_connie() {
    let a = amounts(&input(
        "stock_purchases_every_period",
        "219500.00",
        "203000.00",
        json!({"stock_role": "reseller", "gst_free_purchases": "99470.00"}),
    ));
    assert_eq!(a["gst_free_sales"], "107555.00");
    assert_eq!(a["gst_free_purchases"], "99470.00");
}

#[test]
fn stock_purchases_two_samples_andrew() {
    let a = amounts(&input(
        "stock_purchases_two_samples",
        "219000.00",
        "203000.00",
        json!({"stock_role": "reseller",
               "purchases_sample": sample("2025-06-01", "2025-06-28", "29400.00", "60000.00")}),
    ));
    assert_eq!(a["gst_free_sales"], "107310.00");
    assert_eq!(a["gst_free_purchases"], "99470.00");
}

#[test]
fn a_january_to_june_period_takes_the_december_window() {
    let mut v = input(
        "stock_purchases_two_samples",
        "1000.00",
        "1000.00",
        json!({"stock_role": "reseller",
               "purchases_sample": sample("2026-01-04", "2026-01-31", "250.00", "1000.00")}),
    );
    v["tax_period_start"] = json!("2026-01-01");
    v["tax_period_end"] = json!("2026-03-31");
    assert_eq!(amounts(&v)["gst_free_sales"], "250.00");
    v["purchases_sample"] = sample("2026-02-01", "2026-02-28", "250.00", "1000.00");
    assert_eq!(code(&v), "gst_food_retailer_sam.sample_invalid");
}

#[test]
fn five_percent_basis_valda() {
    let a = amounts(&input(
        "stock_purchases_five_percent",
        "40000.00",
        "24000.00",
        json!({"stock_role": "reseller", "gst_free_purchases": "750.00",
               "product_lines": [{"gst_free_purchases": "300.00", "markup_percent": "40"},
                                 {"gst_free_purchases": "450.00", "markup_percent": "30"}]}),
    ));
    assert_eq!(a["gst_free_sales"], "1005.00");
    assert_eq!(a["gst_free_purchases"], "750.00");
}

#[test]
fn snapshot_five_percent_lee() {
    let a = amounts(&input(
        "snapshot_five_percent",
        "30000.00",
        "9000.00",
        json!({"gst_free_purchases": "1500.00",
               "product_lines": [{"gst_free_purchases": "500.00", "markup_percent": "60"},
                                 {"gst_free_purchases": "800.00", "markup_percent": "30"}]}),
    ));
    assert_eq!(a["gst_free_sales"], "1840.00");
}

#[test]
fn snapshot_methods_take_sales_from_a_two_week_sample() {
    let sales = sample("2025-07-01", "2025-07-14", "3000.00", "12000.00");
    let a = amounts(&input(
        "snapshot_two_samples",
        "100000.00",
        "80000.00",
        json!({"purchases_sample": sample("2025-06-01", "2025-06-28", "20000.00", "50000.00"),
               "sales_sample": sales}),
    ));
    assert_eq!(a["gst_free_sales"], "25000.00");
    assert_eq!(a["gst_free_purchases"], "32000.00");
    let a = amounts(&input(
        "snapshot_every_period",
        "100000.00",
        "80000.00",
        json!({"gst_free_purchases": "30000.00", "sales_sample": sales}),
    ));
    assert_eq!(a["gst_free_sales"], "25000.00");
    assert_eq!(a["gst_free_purchases"], "30000.00");
}

#[test]
fn supermarket_sales_percentage_anita() {
    let mut v = input(
        "supermarket_sales_percentage",
        "200000.00",
        "100000.00",
        json!({"gst_free_sales": "60000.00"}),
    );
    v["eligibility"] = supermarket();
    v["tax_period_start"] = json!("2025-10-01");
    v["tax_period_end"] = json!("2025-12-31");
    let a = amounts(&v);
    assert_eq!(a["gst_free_purchases"], "30000.00");
    assert_eq!(a["other_purchases"], "70000.00");
    assert_eq!(a["trading_stock_input_tax_credits"], "6363.64");
    // Before F2025L01055 commenced on 9 September 2025.
    v["tax_period_start"] = json!("2025-07-01");
    v["tax_period_end"] = json!("2025-09-30");
    assert_eq!(code(&v), "gst_food_retailer_sam.method_not_in_force");
}

fn anil() -> Value {
    let mut v = input(
        "restaurant_purchases_snapshot",
        "0",
        "65000.00",
        json!({"total_sales": null,
               "purchases_sample": sample("2026-07-03", "2026-07-30", "10500.00", "15000.00")}),
    );
    v["eligibility"] = restaurant();
    v["year"] = json!("2026-27");
    v["tax_period_start"] = json!("2026-10-01");
    v["tax_period_end"] = json!("2026-12-31");
    v
}

#[test]
fn restaurant_purchases_snapshot_anil() {
    let a = amounts(&anil());
    assert_eq!(a["gst_free_purchases"], "45500.00");
    assert_eq!(a["other_purchases"], "19500.00");
    assert_eq!(a["trading_stock_input_tax_credits"], "1772.73");
    assert!(a.get("gst_free_sales").is_none());
}

#[test]
fn restaurant_method_starts_on_24_september_2026() {
    // An annual tax period in 2026-27 starts on 1 July 2026, before
    // F2026L01275 commenced, so s 8 cannot yet apply.
    let mut v = anil();
    v["tax_period_start"] = json!("2026-07-01");
    v["tax_period_end"] = json!("2027-06-30");
    assert_eq!(code(&v), "gst_food_retailer_sam.method_not_in_force");
    v["tax_period_start"] = json!("2027-01-01");
    v["tax_period_end"] = json!("2027-03-31");
    v["purchases_sample"] = sample("2026-12-05", "2027-01-01", "10500.00", "15000.00");
    assert_eq!(amounts(&v)["gst_free_purchases"], "45500.00");
}

#[test]
fn eligibility_and_inputs_are_refused_by_name() {
    let norms = || {
        input(
            "business_norms",
            "1.00",
            "1.00",
            json!({"retailer_type": "cake_shop"}),
        )
    };
    let mut v = norms();
    v["eligibility"]["sam_turnover"] = json!("2000000.01");
    assert_eq!(code(&v), "gst_food_retailer_sam.not_eligible");
    v["eligibility"]["sam_turnover"] = json!("2000000.00");
    assert_eq!(api::evaluate(C, &v).unwrap()["ok"], json!(true));
    v["eligibility"]["adequate_point_of_sale_equipment"] = json!(true);
    assert_eq!(code(&v), "gst_food_retailer_sam.not_eligible");
    let cases = [
        (
            json!({"retailer_type": "bakery"}),
            "gst_food_retailer_sam.retailer_type_invalid",
        ),
        (json!({"gst_free_sales": "1.00"}), "input.field_unknown"),
        (
            json!({"method": "prisons"}),
            "gst_food_retailer_sam.method_invalid",
        ),
        (
            json!({"tax_period_end": "2026-07-01"}),
            "gst_food_retailer_sam.tax_period_invalid",
        ),
        (
            json!({"retailer_type": "convenience_store"}),
            "input.missing_field",
        ),
    ];
    for (change, want) in cases {
        let mut v = norms();
        for (key, value) in change.as_object().unwrap() {
            v[key] = value.clone();
        }
        assert_eq!(code(&v), want, "{change}");
    }
}

#[test]
fn what_the_determinations_do_not_allow_is_refused() {
    let converter = input(
        "stock_purchases_every_period",
        "10.00",
        "10.00",
        json!({"stock_role": "converter", "gst_free_purchases": "1.00"}),
    );
    assert_eq!(code(&converter), "gst_food_retailer_sam.stock_role_invalid");
    let mut across = input(
        "stock_purchases_two_samples",
        "10.00",
        "10.00",
        json!({"stock_role": "reseller",
               "purchases_sample": sample("2025-06-01", "2025-06-28", "1.00", "2.00")}),
    );
    across["tax_period_end"] = json!("2026-06-30");
    assert_eq!(
        code(&across),
        "gst_food_retailer_sam.tax_period_spans_halves"
    );
    across["tax_period_end"] = json!("2025-09-30");
    across["purchases_sample"] = sample("2025-06-01", "2025-06-27", "1.00", "2.00");
    assert_eq!(code(&across), "gst_food_retailer_sam.sample_invalid");
    let unbalanced = input(
        "business_norms",
        "100.00",
        "100.00",
        json!({"retailer_type": "pharmacy", "pbs_sales": "50.00", "non_pbs_sales": "10.00",
               "otc_sales": "30.00", "non_pbs_purchases": "50.00", "otc_purchases": "50.00"}),
    );
    assert_eq!(
        code(&unbalanced),
        "gst_food_retailer_sam.components_unbalanced"
    );
    let over = input(
        "stock_purchases_every_period",
        "10.00",
        "10.00",
        json!({"stock_role": "reseller", "gst_free_purchases": "10.01"}),
    );
    assert_eq!(code(&over), "gst_food_retailer_sam.amount_exceeds_total");
    let nil = input(
        "stock_purchases_every_period",
        "10.00",
        "0.00",
        json!({"stock_role": "reseller", "gst_free_purchases": "0.00"}),
    );
    assert_eq!(code(&nil), "gst_food_retailer_sam.total_nil");
}

#[test]
fn a_new_business_samples_its_first_two_months() {
    let mut v = input(
        "stock_purchases_two_samples",
        "1000.00",
        "1000.00",
        json!({"stock_role": "reseller", "trading_started": "2025-08-10",
               "purchases_sample": sample("2025-09-01", "2025-09-28", "400.00", "1000.00")}),
    );
    assert_eq!(amounts(&v)["gst_free_purchases"], "400.00");
    v["purchases_sample"] = sample("2025-09-20", "2025-10-17", "400.00", "1000.00");
    assert_eq!(code(&v), "gst_food_retailer_sam.sample_invalid");
}

#[test]
fn a_2017_new_business_replaces_only_the_july_to_december_sample() {
    // Started 10 August 2025: the first 2 months serve July to December, but
    // January to March 2026 takes the ordinary 1 December to 31 January window.
    let mut v = input(
        "stock_purchases_two_samples",
        "1000.00",
        "1000.00",
        json!({"stock_role": "reseller", "trading_started": "2025-08-10",
               "purchases_sample": sample("2025-09-01", "2025-09-28", "400.00", "1000.00")}),
    );
    v["tax_period_start"] = json!("2026-01-01");
    v["tax_period_end"] = json!("2026-03-31");
    assert_eq!(code(&v), "gst_food_retailer_sam.sample_invalid");
    v["purchases_sample"] = sample("2025-12-01", "2025-12-28", "300.00", "1000.00");
    assert_eq!(amounts(&v)["gst_free_purchases"], "300.00");
    // Started 15 December: the December window, from the day trading started.
    v["trading_started"] = json!("2025-12-15");
    assert_eq!(code(&v), "gst_food_retailer_sam.sample_invalid");
    v["purchases_sample"] = sample("2025-12-15", "2026-01-11", "300.00", "1000.00");
    assert_eq!(amounts(&v)["gst_free_purchases"], "300.00");
    // Started in January to June: the determination does not settle it.
    v["trading_started"] = json!("2026-02-10");
    v["purchases_sample"] = sample("2026-02-15", "2026-03-14", "300.00", "1000.00");
    assert_eq!(code(&v), "gst_food_retailer_sam.new_business_unsettled");
}

#[test]
fn a_restaurant_new_business_samples_only_the_half_it_starts_in() {
    let mut v = anil();
    v["trading_started"] = json!("2026-10-05");
    v["purchases_sample"] = sample("2026-10-10", "2026-11-06", "10500.00", "15000.00");
    assert_eq!(amounts(&v)["gst_free_purchases"], "45500.00");
    v["purchases_sample"] = sample("2026-11-10", "2026-12-07", "10500.00", "15000.00");
    assert_eq!(code(&v), "gst_food_retailer_sam.sample_invalid");
    // The next half takes para (b)'s December window, not the first months.
    v["tax_period_start"] = json!("2027-01-01");
    v["tax_period_end"] = json!("2027-03-31");
    v["purchases_sample"] = sample("2026-10-10", "2026-11-06", "10500.00", "15000.00");
    assert_eq!(code(&v), "gst_food_retailer_sam.sample_invalid");
    v["purchases_sample"] = sample("2026-12-05", "2027-01-01", "10500.00", "15000.00");
    assert_eq!(amounts(&v)["gst_free_purchases"], "45500.00");
    // Starting in January to June: para (c) once para (b) cannot be applied.
    v["tax_period_start"] = json!("2027-04-01");
    v["tax_period_end"] = json!("2027-06-30");
    v["trading_started"] = json!("2027-02-10");
    v["purchases_sample"] = sample("2027-02-15", "2027-03-14", "10500.00", "15000.00");
    assert_eq!(amounts(&v)["gst_free_purchases"], "45500.00");
    // Started 2 January: para (b) can still be applied, so (c) is not open.
    v["trading_started"] = json!("2027-01-02");
    assert_eq!(code(&v), "gst_food_retailer_sam.sample_invalid");
    v["purchases_sample"] = sample("2027-01-02", "2027-01-29", "10500.00", "15000.00");
    assert_eq!(amounts(&v)["gst_free_purchases"], "45500.00");
}

#[test]
fn attested_facts_the_determinations_need() {
    let mut norms = input(
        "business_norms",
        "1000.00",
        "1000.00",
        json!({"retailer_type": "cake_shop"}),
    );
    norms["eligibility"]["retailer_type_definition_met"] = json!(false);
    assert_eq!(code(&norms), "gst_food_retailer_sam.not_eligible");
    let mut market = input(
        "supermarket_sales_percentage",
        "200000.00",
        "100000.00",
        json!({"gst_free_sales": "60000.00"}),
    );
    market["eligibility"] = supermarket();
    market["eligibility"]["section_7_3_goods_left_out"] = json!(false);
    market["tax_period_start"] = json!("2025-10-01");
    market["tax_period_end"] = json!("2025-12-31");
    assert_eq!(code(&market), "gst_food_retailer_sam.not_eligible");
    market["eligibility"]
        .as_object_mut()
        .unwrap()
        .remove("section_7_3_goods_left_out");
    assert_eq!(code(&market), "input.missing_field");
}

#[test]
fn rural_reseller_pbs_stock_and_a_sampled_five_percent_snapshot() {
    let a = amounts(&input(
        "business_norms",
        "50000.00",
        "40000.00",
        json!({"retailer_type": "rural_convenience_store", "stock_role": "reseller",
               "fuel_and_post_sales": "20000.00", "fuel_and_post_purchases": "18000.00"}),
    ));
    // (50,000 - 20,000) x 30% and (40,000 - 18,000) x 30%.
    assert_eq!(a["gst_free_sales"], "9000.00");
    assert_eq!(a["gst_free_purchases"], "6600.00");
    // PBS stock among the purchases: the components do not make them up.
    let pbs = input(
        "business_norms",
        "100000.00",
        "100000.00",
        json!({"retailer_type": "pharmacy", "pbs_sales": "40000.00", "non_pbs_sales": "10000.00",
               "otc_sales": "50000.00", "non_pbs_purchases": "10000.00", "otc_purchases": "50000.00"}),
    );
    assert_eq!(code(&pbs), "gst_food_retailer_sam.components_unbalanced");
    let a = amounts(&input(
        "snapshot_five_percent",
        "30000.00",
        "9000.00",
        json!({"purchases_sample": sample("2025-06-01", "2025-06-28", "300.00", "6000.00"),
               "product_lines": [{"gst_free_purchases": "500.00", "markup_percent": "60"}]}),
    ));
    // 9,000 x 300 / 6,000; 500 x 160%.
    assert_eq!(a["gst_free_purchases"], "450.00");
    assert_eq!(a["gst_free_sales"], "800.00");
}

#[test]
fn trading_started_metadata_qualifies_first_two_months_and_points_to_december() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let desc = catalogue["gst_food_retailer_sam"]["optional_inputs"]["trading_started"]
        .as_str()
        .unwrap();
    assert!(
        desc.contains("2017") && desc.contains("July to December"),
        "missing 2017 July to December qualification: {desc}"
    );
    assert!(
        desc.contains("restaurant_purchases_snapshot"),
        "missing restaurant method qualification: {desc}"
    );
    assert!(
        desc.contains("1 December to 31 January"),
        "missing December window pointer for January to June periods: {desc}"
    );
}
