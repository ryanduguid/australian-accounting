// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Wine equalisation tax (A New Tax System (Wine Equalisation Tax) Act 1999
//! ss 5-5, 9-25 to 9-40 and Division 19) on the ATO's, WETR 2009/1's and the
//! Library's worked examples, read on 19 November 2026
//! (docs/calculation-evidence.md#wine-equalisation-tax).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(year: &str, dealings: Value) -> Value {
    json!({"dealings": dealings, "year": year, "scope_confirmed": true})
}

fn run(input: &Value) -> Value {
    api::evaluate("wine-equalisation-tax", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("wine-equalisation-tax", input)
        .unwrap_err()
        .code
}

fn wholesale(price: &str) -> Value {
    json!({"kind": "wholesale_sale", "price_excluding_wet_and_gst": price})
}

fn retail(price: &str) -> Value {
    json!({"kind": "retail_sale", "retail_price_including_wet_and_gst": price})
}

fn own_use(price: &str) -> Value {
    json!({"kind": "own_use", "retail_price_including_wet_and_gst": price})
}

fn quoted(price: &str, intention: &str) -> Value {
    json!({"kind": "quoted_sale", "price_excluding_wet_and_gst": price, "purchaser_intention": intention})
}

/// `dealing` claiming the rebate with every stated condition met.
fn claiming(mut dealing: Value) -> Value {
    dealing["rebate_conditions"] = json!({
        "producer": true,
        "source_product_owned_85_percent": true,
        "container_and_trade_mark": true,
    });
    dealing
}

fn with_rebate(year: &str, dealings: Value, earlier: &str, group: bool) -> Value {
    let mut i = input(year, dealings);
    i["rebate_claimed_earlier_in_year"] = json!(earlier);
    i["associated_producer"] = json!(group);
    i
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

fn field<'a>(result: &'a Value, n: usize, key: &str) -> &'a str {
    result["dealings"][n][key].as_str().unwrap()
}

/// The ATO's 'Wholesale sales' (QC22750): Alison sells a dozen for $120
/// before tax; WET $34.80, GST $15.48 on $154.80, $170.28 in all. The
/// Library's ¶22-280 (Swillo, 5 cases at $120): $174.00, $77.40, $851.40.
#[test]
fn wholesale_sales_take_29_percent_of_the_price_and_gst_on_top() {
    let r = run(&input(
        "2026-27",
        json!([wholesale("120.00"), wholesale("600.00")]),
    ));
    assert_eq!(field(&r, 0, "taxable_value"), "120.00");
    assert_eq!(field(&r, 0, "wine_tax"), "34.80");
    assert_eq!(field(&r, 0, "gst"), "15.48");
    assert_eq!(field(&r, 0, "price_including_wet_and_gst"), "170.28");
    assert_eq!(field(&r, 1, "wine_tax"), "174.00");
    assert_eq!(field(&r, 1, "gst"), "77.40");
    assert_eq!(field(&r, 1, "price_including_wet_and_gst"), "851.40");
    assert_eq!(amount(&r, "wine_tax"), "208.80");
    assert_eq!(amount(&r, "label_1c"), "208.00");
    assert_eq!(r["rates"]["wine_tax_rate"], "0.29");
}

/// QC22751 (Happy Valley): a dozen at the cellar door for $140 has a
/// taxable value of $70.00 and WET of $20.30; a bottle of it used for
/// tastings, at $11.67 retail, WET of $1.69 (the ATO rounds its value to
/// $5.83 first, this command to $5.84; the tax is the same). WETR 2009/1
/// example 15: $22 gives $3.19 and GST of $2.00; para 159: a staff sale is
/// taxed at the normal retail price, $150 a dozen giving $21.75.
#[test]
fn half_retail_price_is_half_the_price_including_wet_and_gst() {
    let r = run(&input(
        "2025-26",
        json!([
            retail("140.00"),
            own_use("11.67"),
            retail("22.00"),
            retail("150.00")
        ]),
    ));
    assert_eq!(field(&r, 0, "taxable_value"), "70.00");
    assert_eq!(field(&r, 0, "wine_tax"), "20.30");
    assert_eq!(field(&r, 0, "taxable_value_method"), "half_retail_price");
    assert_eq!(field(&r, 1, "taxable_value"), "5.84");
    assert_eq!(field(&r, 1, "wine_tax"), "1.69");
    assert!(r["dealings"][1].get("gst").is_none());
    assert_eq!(field(&r, 2, "wine_tax"), "3.19");
    assert_eq!(field(&r, 2, "gst"), "2.00");
    assert_eq!(field(&r, 3, "wine_tax"), "21.75");
    assert_eq!(amount(&r, "wine_tax"), "46.93");
    assert_eq!(amount(&r, "label_1c"), "46.00");
}

/// WETR 2009/1 example 16 and QC22751 (Good Wines): 70% of the period's
/// wholesale sales at $80 a dozen and 30% at $90 average $83; a dozen sold
/// at the cellar door for $190 then bears WET of $24.07 and GST of $17.27.
#[test]
fn average_wholesale_price_is_the_weighted_average_of_the_periods_wholesale_sales() {
    let average = json!({
        "grape_wine": true,
        "wholesale_share_at_least_10_percent": true,
        "quantity": "1",
        "wholesale_sales": [
            {"quantity": "7", "price_excluding_wet_and_gst": "560.00"},
            {"quantity": "3", "price_excluding_wet_and_gst": "270.00"},
        ],
    });
    let mut sale = retail("190.00");
    sale["average_wholesale"] = average.clone();
    let use_ = json!({"kind": "own_use", "average_wholesale": average});
    let r = run(&input("2024-25", json!([sale, use_])));
    assert_eq!(field(&r, 0, "taxable_value"), "83.00");
    assert_eq!(
        field(&r, 0, "taxable_value_method"),
        "average_wholesale_price"
    );
    assert_eq!(field(&r, 0, "wine_tax"), "24.07");
    assert_eq!(field(&r, 0, "gst"), "17.27");
    assert_eq!(field(&r, 1, "wine_tax"), "24.07");
}

/// QC22758's table of rebates in 2026-27, half retail price method: a
/// $20,000 retail sale ($2,900 and $2,900), a $30,000 wholesale sale
/// ($8,700 and $8,700), a $15,000 sale under quote ($0 and $4,350), a $20,000
/// one whose purchaser will export ($0 and $0) and $2,000 of tastings ($290
/// and $290).
#[test]
fn producer_rebate_follows_the_atos_table() {
    let r = run(&with_rebate(
        "2026-27",
        json!([
            claiming(retail("20000.00")),
            claiming(wholesale("30000.00")),
            claiming(quoted("15000.00", "taxable_dealing")),
            claiming(quoted("20000.00", "gst_free_supply")),
            claiming(own_use("2000.00")),
        ]),
        "0.00",
        false,
    ));
    let taxes: Vec<&str> = (0..5).map(|n| field(&r, n, "wine_tax")).collect();
    assert_eq!(taxes, ["2900.00", "8700.00", "0.00", "0.00", "290.00"]);
    let rebates: Vec<&str> = (0..5)
        .map(|n| {
            r["dealings"][n]["producer_rebate"]["rebate"]
                .as_str()
                .unwrap()
        })
        .collect();
    assert_eq!(rebates, ["2900.00", "8700.00", "4350.00", "0.00", "290.00"]);
    assert_eq!(r["dealings"][3]["producer_rebate"]["all_met"], false);
    let liable = &r["dealings"][3]["producer_rebate"]["conditions"][1];
    assert_eq!(
        liable["condition"],
        "liable_to_wine_tax_or_quoted_for_a_taxable_dealing"
    );
    assert_eq!(liable["met"], false);
    assert_eq!(amount(&r, "wine_tax"), "11890.00");
    assert_eq!(amount(&r, "producer_rebate"), "16240.00");
    assert_eq!(amount(&r, "label_1c"), "11890.00");
    assert_eq!(amount(&r, "label_1d"), "16240.00");
    assert_eq!(r["rates"]["producer_rebate_cap"], "400000");
}

/// The cap is $350,000 to 2025-26 and $400,000 from 2026-27 (s 19-15(2), Act
/// No. 12 of 2026 Sch 6), less the year's earlier claims; a $50,000
/// wholesale sale earns $14,500 of which $10,000 fits.
#[test]
fn producer_rebate_stops_at_the_years_cap() {
    for (year, earlier) in [
        ("2025-26", "340000.00"),
        ("2026-27", "390000.00"),
        ("2027-28", "390000.00"),
    ] {
        let r = run(&with_rebate(
            year,
            json!([claiming(wholesale("50000.00"))]),
            earlier,
            false,
        ));
        assert_eq!(
            amount(&r, "producer_rebate_before_cap"),
            "14500.00",
            "{year}"
        );
        assert_eq!(amount(&r, "producer_rebate"), "10000.00", "{year}");
        assert_eq!(amount(&r, "rebate_over_cap"), "4500.00", "{year}");
        assert_eq!(amount(&r, "label_1d"), "10000.00", "{year}");
        assert_eq!(amount(&r, "wine_tax"), "14500.00", "{year}");
    }
    let r = run(&with_rebate(
        "2023-24",
        json!([claiming(wholesale("50000.00"))]),
        "350000.00",
        false,
    ));
    assert_eq!(r["rates"]["producer_rebate_cap"], "350000");
    assert_eq!(amount(&r, "producer_rebate"), "0.00");
}

/// A group shares the cap; claims already above it carry the s 19-25 warning.
#[test]
fn group_and_excess_claims_are_warned() {
    let text = |r: &Value| r["warnings"].to_string();
    let r = run(&with_rebate(
        "2026-27",
        json!([claiming(wholesale("100.00"))]),
        "0.00",
        true,
    ));
    assert!(text(&r).contains("associated producer"));
    assert!(!text(&r).contains("19-25"));
    let r = run(&with_rebate(
        "2026-27",
        json!([claiming(wholesale("100.00"))]),
        "410000.00",
        false,
    ));
    assert!(text(&r).contains("19-25"));
    assert_eq!(amount(&r, "producer_rebate"), "0.00");
    let r = run(&input("2026-27", json!([wholesale("100.00")])));
    assert!(!text(&r).contains("WETR 2009/2"));
}

/// A stated condition not met earns no rebate and says which.
#[test]
fn an_unmet_condition_earns_no_rebate() {
    let mut sale = claiming(wholesale("1000.00"));
    sale["rebate_conditions"]["container_and_trade_mark"] = json!(false);
    let r = run(&with_rebate("2026-27", json!([sale]), "0.00", false));
    let rebate = &r["dealings"][0]["producer_rebate"];
    assert_eq!(rebate["all_met"], false);
    assert_eq!(rebate["rebate"], "0.00");
    assert_eq!(
        rebate["conditions"][3]["condition"],
        "container_and_trade_mark"
    );
    assert_eq!(rebate["conditions"][3]["met"], false);
    assert_eq!(amount(&r, "wine_tax"), "290.00");
    assert_eq!(amount(&r, "producer_rebate"), "0.00");
}

/// Own use of wine bought under quote is taxed on the purchase price; the
/// Library's hamper (¶22-280) is the wine's $150 share; 12.995 rounds up.
#[test]
fn purchase_price_hamper_share_and_rounding() {
    let r = run(&input(
        "2026-27",
        json!([
            {"kind": "own_use_purchased_under_quote", "purchase_price_excluding_gst": "100.00"},
            wholesale("150.00"),
            retail("25.99"),
        ]),
    ));
    assert_eq!(field(&r, 0, "wine_tax"), "29.00");
    assert_eq!(field(&r, 0, "taxable_value_method"), "purchase_price");
    assert_eq!(field(&r, 1, "wine_tax"), "43.50");
    assert_eq!(field(&r, 2, "taxable_value"), "13.00");
    assert_eq!(field(&r, 2, "wine_tax"), "3.77");
    assert_eq!(amount(&r, "label_1c"), "76.00");
}

#[test]
fn refuses_what_it_does_not_work_out() {
    let ok = json!([wholesale("1.00")]);
    assert_eq!(code(&input("2022-23", ok.clone())), "period.unsupported");
    let mut not_confirmed = input("2026-27", ok.clone());
    not_confirmed["scope_confirmed"] = json!(false);
    assert_eq!(code(&not_confirmed), "scope.not_confirmed");
    let average = |grape: bool, share: bool| {
        let mut sale = retail("10.00");
        sale["average_wholesale"] = json!({
            "grape_wine": grape,
            "wholesale_share_at_least_10_percent": share,
            "quantity": "1",
            "wholesale_sales": [{"quantity": "1", "price_excluding_wet_and_gst": "5.00"}],
        });
        input("2026-27", json!([sale]))
    };
    assert_eq!(
        code(&average(false, true)),
        "wine_equalisation_tax.method_unavailable"
    );
    assert_eq!(
        code(&average(true, false)),
        "wine_equalisation_tax.method_unavailable"
    );
    let mut zero = average(true, true);
    zero["dealings"][0]["average_wholesale"]["quantity"] = json!("0");
    assert_eq!(code(&zero), "wine_equalisation_tax.quantity_invalid");
    let mut empty = average(true, true);
    empty["dealings"][0]["average_wholesale"]["wholesale_sales"] = json!([]);
    assert_eq!(code(&empty), "wine_equalisation_tax.dealing_invalid");
    let mut both = own_use("10.00");
    both["average_wholesale"] = average(true, true)["dealings"][0]["average_wholesale"].clone();
    assert_eq!(
        code(&input("2026-27", json!([both]))),
        "input.field_unknown"
    );
    let bought = claiming(
        json!({"kind": "own_use_purchased_under_quote", "purchase_price_excluding_gst": "1.00"}),
    );
    assert_eq!(
        code(&with_rebate("2026-27", json!([bought]), "0.00", false)),
        "input.field_unknown"
    );
    let cases = [
        (json!([]), "wine_equalisation_tax.dealings_invalid"),
        (json!("x"), "wine_equalisation_tax.dealings_invalid"),
        (json!([1]), "wine_equalisation_tax.dealing_invalid"),
        (
            json!([{"kind": "export"}]),
            "wine_equalisation_tax.kind_invalid",
        ),
        (
            json!([quoted("1.00", "export")]),
            "wine_equalisation_tax.intention_invalid",
        ),
        (json!([wholesale("1.005")]), "input.amount_invalid"),
        (json!([{"kind": "wholesale_sale"}]), "input.missing_field"),
        (json!([{"kind": "retail_sale"}]), "input.missing_field"),
        (
            json!([{"kind": "wholesale_sale", "price_excluding_wet_and_gst": "1.00", "gst": "0.10"}]),
            "input.field_unknown",
        ),
    ];
    for (dealings, expected) in cases {
        assert_eq!(
            code(&input("2026-27", dealings.clone())),
            expected,
            "{dealings}"
        );
    }
    let mut bad_flag = claiming(wholesale("1.00"));
    bad_flag["rebate_conditions"]["producer"] = json!("yes");
    assert_eq!(
        code(&with_rebate("2026-27", json!([bad_flag]), "0.00", false)),
        "input.flag_invalid"
    );
    let mut missing = input("2026-27", json!([claiming(wholesale("1.00"))]));
    assert_eq!(code(&missing), "input.missing_field");
    missing["rebate_claimed_earlier_in_year"] = json!("0.00");
    assert_eq!(code(&missing), "input.missing_field");
    let unclaimed = with_rebate("2026-27", ok.clone(), "0.00", false);
    assert_eq!(code(&unclaimed), "input.field_unknown");
    let mut stray = input("2026-27", ok);
    stray["method"] = json!("half_retail");
    assert_eq!(code(&stray), "input.field_unknown");
}

/// Average wholesale price facts that are not an object, a quantity that is
/// not a decimal string and wholesale sales that are not a list are refused,
/// naming the dealing.
#[test]
fn malformed_average_wholesale_facts_are_refused() {
    let average = |changes: Value| {
        let mut facts = json!({
            "grape_wine": true, "wholesale_share_at_least_10_percent": true,
            "quantity": "1",
            "wholesale_sales": [{"quantity": "7", "price_excluding_wet_and_gst": "560.00"}],
        });
        for (key, value) in changes.as_object().unwrap() {
            facts[key] = value.clone();
        }
        let mut sale = retail("190.00");
        sale["average_wholesale"] = facts;
        input("2024-25", json!([sale]))
    };
    let mut not_object = retail("190.00");
    not_object["average_wholesale"] = json!("560.00");
    for (input, message) in [
        (input("2024-25", json!([not_object])), "dealing 1: average_wholesale must be an object."),
        (average(json!({"wholesale_sales": "7 at $560"})), "dealing 1: wholesale_sales must be a list"),
    ] {
        let error = api::evaluate_coded("wine-equalisation-tax", &input).unwrap_err();
        assert_eq!(error.code, "wine_equalisation_tax.dealing_invalid");
        assert!(error.message.starts_with(message));
    }
    let number = api::evaluate_coded("wine-equalisation-tax", &average(json!({"quantity": 1})));
    assert_eq!(number.unwrap_err().code, "wine_equalisation_tax.quantity_invalid");
}
