// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The GST margin scheme (GST Act Division 75, read on 19 November 2026)
//! against the Library's worked examples (GST, Real Estate, 11-100 and 11-110;
//! Tax Examples, GST, 12-200 and 12-220), GSTR 2006/8 example 1 and
//! hand-worked cases (docs/calculation-evidence.md#gst-margin-scheme).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A freehold sale under a written agreement, worked by the consideration
/// method.
fn sale(acquisition: &str, consideration: &str, paid: &str) -> Value {
    json!({
        "interest": "freehold_interest", "agreement": "written_on_or_before_supply",
        "acquisition": acquisition, "consideration": consideration,
        "method": "consideration", "recipient_associate": false,
        "acquisition_consideration": paid,
        "unpaid_acquisition_consideration": "0.00",
        "year": "2025-26", "scope_confirmed": true,
    })
}

/// A sale of land held before 1 July 2000, worked from an approved valuation.
fn valued(consideration: &str, valuation: &str, day: &str) -> Value {
    json!({
        "interest": "freehold_interest", "agreement": "written_on_or_before_supply",
        "acquisition": "before_1_july_2000", "consideration": consideration,
        "method": "approved_valuation", "recipient_associate": false,
        "approved_valuation": valuation,
        "valuation_day": day, "registered_on_1_july_2000": true,
        "government_unimproved_on_1_july_2000": false,
        "year": "2025-26", "scope_confirmed": true,
    })
}

fn with(mut input: Value, pairs: &[(&str, Value)]) -> Value {
    for (key, value) in pairs {
        input[*key] = value.clone();
    }
    input
}

fn without(mut input: Value, key: &str) -> Value {
    input.as_object_mut().unwrap().remove(key);
    input
}

fn run(input: &Value) -> Value {
    let result = api::evaluate("gst-margin-scheme", input).unwrap();
    // derive gives the same result.
    let envelope = json!({"command": "gst-margin-scheme", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("gst-margin-scheme", input)
        .unwrap_err()
        .code
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// Tax Examples, GST, 12-200 (Joanne Black): land bought in January 2000,
/// worth $448,000 on 1 July 2000 when she registered, sold with a house in
/// January 2026 for $640,000. The margin is $192,000 and the GST 1/11 of it,
/// $17,454.55 to the cent (the example shows whole dollars, $17,454).
#[test]
fn joanne_black_pays_gst_on_the_value_added_since_1_july_2000() {
    let r = run(&valued("640000.00", "448000.00", "2000-07-01"));
    assert_eq!(r["eligible"], json!(true));
    assert_eq!(r["failing_conditions"], json!([]));
    assert_eq!(amount(&r, "margin_base"), "448000.00");
    assert_eq!(amount(&r, "margin"), "192000.00");
    assert_eq!(amount(&r, "gst_under_margin_scheme"), "17454.55");
    assert_eq!(amount(&r, "gst_without_margin_scheme"), "58181.82");
    assert_eq!(amount(&r, "gst"), "17454.55");
    assert!(r["base_rule"].as_str().unwrap().contains("items 1 and 3"));
    assert!(r["consequences"][0].as_str().unwrap().contains("s 75-20"));
    // The catalogue's example is the same sale.
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["gst_margin_scheme"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_eq!(run(&facts)["amounts"], r["amounts"]);
}

/// GST, Real Estate, 11-100 (property held at 1 July 2000): bought in 1999
/// for $75,000, valued at $100,000 on 1 July 2000, sold for $144,000: GST
/// $4,000. By the consideration method the base would be the $75,000 paid.
#[test]
fn land_held_at_1_july_2000_by_either_method() {
    let r = run(&valued("144000.00", "100000.00", "2000-07-01"));
    assert_eq!(amount(&r, "margin"), "44000.00");
    assert_eq!(amount(&r, "gst"), "4000.00");
    let r = run(&sale("before_1_july_2000", "144000.00", "75000.00"));
    assert_eq!(amount(&r, "margin"), "69000.00");
    assert_eq!(amount(&r, "gst"), "6272.73");
}

/// GST, Real Estate, 11-110 (cost of improvements): land costing $250,000,
/// improved for $50,000 and sold for $360,000. The improvements are not part
/// of the base (s 75-14): margin $110,000, GST $10,000.
#[test]
fn improvements_stay_out_of_the_margin() {
    let r = run(&sale(
        "other_supply_from_1_july_2000",
        "360000.00",
        "250000.00",
    ));
    assert_eq!(amount(&r, "margin"), "110000.00");
    assert_eq!(amount(&r, "gst_under_margin_scheme"), "10000.00");
    assert_eq!(amount(&r, "gst"), "10000.00");
    assert!(r["amounts"].get("apportioned_base").is_none());
}

/// GST, Real Estate, 11-110 (allocating cost to subdivisions): land bought
/// for $500,000 and cut into 5 equal blocks of 600 square metres, one sold for
/// $155,000 (GST $5,000) and 4 for $166,000 (GST $6,000 each).
#[test]
fn equal_blocks_take_a_fifth_of_the_cost_each() {
    let block = |price: &str| {
        let input = sale("taxable_supply_under_margin_scheme", price, "500000.00");
        run(&with(
            input,
            &[
                ("subdivision_share", json!("600")),
                ("subdivision_whole", json!("3000")),
            ],
        ))
    };
    let r = block("155000.00");
    assert_eq!(amount(&r, "apportioned_base"), "100000.00");
    assert_eq!(amount(&r, "gst"), "5000.00");
    assert_eq!(amount(&block("166000.00"), "gst"), "6000.00");
}

/// The same land with one block of 1,200 and 4 of 600 square metres: the
/// large block's base is 1,200 / 3,600 of $500,000. Sold for $300,000, its
/// margin is $133,333.33 and its GST $12,121.21. The Library shows $12,121.18,
/// 1/11 of the margin rounded to whole dollars first; s 75-15 and s 9-90 round
/// only the GST, to the cent.
#[test]
fn an_unequal_block_takes_its_share_by_area() {
    let input = sale("other_supply_from_1_july_2000", "300000.00", "500000.00");
    let input = with(
        input,
        &[
            ("subdivision_share", json!("1200")),
            ("subdivision_whole", json!("3600")),
        ],
    );
    let r = run(&input);
    assert_eq!(amount(&r, "apportioned_base"), "166666.67");
    assert_eq!(amount(&r, "margin"), "133333.33");
    assert_eq!(amount(&r, "gst"), "12121.21");
}

/// GST, Real Estate, 11-110: land bought for $400,000 for 4 strata units
/// expected to sell for $2,400,000 in all; unit 3, expected at $600,000, takes
/// $100,000 of the cost and sells for $610,000. The GST on its $510,000 margin
/// is $46,363.636..., $46,363.64 rounded half up (s 9-90(1)); the Library
/// shows $46,363.63, cut off rather than rounded.
#[test]
fn a_strata_unit_takes_its_share_by_expected_price() {
    let input = sale("other_supply_from_1_july_2000", "610000.00", "400000.00");
    let input = with(
        input,
        &[
            ("interest", json!("stratum_unit")),
            ("subdivision_share", json!("600000.00")),
            ("subdivision_whole", json!("2400000.00")),
        ],
    );
    let r = run(&input);
    assert_eq!(amount(&r, "apportioned_base"), "100000.00");
    assert_eq!(amount(&r, "margin"), "510000.00");
    assert_eq!(amount(&r, "gst"), "46363.64");
}

/// Tax Examples, GST, 12-220 (Hans and Luke), the margin scheme option: Hans,
/// registered on 1 July 2000 with land then valued at $250,000, sells it to
/// Luke under the margin scheme for $1.5 million (GST $113,636.36; the example
/// shows $113,636). Luke sells 3 townhouses on it for $900,000 each: a third
/// of his $1.5 million each, so a $400,000 margin and $36,363.64 GST each
/// (shown as $36,363), $109,090.91 in all.
#[test]
fn hans_and_luke_under_the_margin_scheme() {
    let hans = run(&valued("1500000.00", "250000.00", "2000-07-01"));
    assert_eq!(amount(&hans, "margin"), "1250000.00");
    assert_eq!(amount(&hans, "gst"), "113636.36");
    let townhouse = with(
        sale(
            "taxable_supply_under_margin_scheme",
            "900000.00",
            "1500000.00",
        ),
        &[
            ("subdivision_share", json!("1")),
            ("subdivision_whole", json!("3")),
        ],
    );
    let r = run(&townhouse);
    assert_eq!(amount(&r, "apportioned_base"), "500000.00");
    assert_eq!(amount(&r, "margin"), "400000.00");
    assert_eq!(amount(&r, "gst"), "36363.64");
}

/// GSTR 2006/8 example 1 (Tureia): land bought for $220,000 through a
/// taxable supply on which GST was worked out without the margin scheme, so
/// her later sale cannot use it (s 75-5(2) and (3)(a)). Her units' price is
/// not given; $1,100,000 is fabricated. The margin is still shown for
/// comparison, but the GST is 1/11 of the price.
#[test]
fn tureia_cannot_use_the_margin_scheme() {
    let input = sale(
        "taxable_supply_without_margin_scheme",
        "1100000.00",
        "220000.00",
    );
    let r = run(&input);
    assert_eq!(r["eligible"], json!(false));
    assert_eq!(r["failing_conditions"], json!(["acquisition_eligible"]));
    assert_eq!(amount(&r, "gst_under_margin_scheme"), "80000.00");
    assert_eq!(amount(&r, "gst_without_margin_scheme"), "100000.00");
    assert_eq!(amount(&r, "gst"), "100000.00");
    assert!(
        r["consequences"][0]
            .as_str()
            .unwrap()
            .contains("does not apply")
    );
}

/// Each condition of s 75-5 fails alone, and together.
#[test]
fn each_condition_fails_alone_and_together() {
    let base = sale("other_supply_from_1_july_2000", "330000.00", "220000.00");
    let failing = |pairs: &[(&str, Value)]| {
        let r = run(&with(base.clone(), pairs));
        assert_eq!(r["eligible"], json!(r["failing_conditions"] == json!([])));
        assert_eq!(
            amount(&r, "gst"),
            if r["eligible"] == json!(true) {
                "10000.00"
            } else {
                "30000.00"
            }
        );
        r["failing_conditions"].clone()
    };
    assert_eq!(failing(&[]), json!([]));
    assert_eq!(
        failing(&[("interest", json!("other"))]),
        json!(["real_property_interest"])
    );
    assert_eq!(
        failing(&[("agreement", json!("none"))]),
        json!(["written_agreement"])
    );
    assert_eq!(
        failing(&[("agreement", json!("written_in_further_period_allowed"))]),
        json!([])
    );
    assert_eq!(
        failing(&[("interest", json!("long_term_lease"))]),
        json!([])
    );
    assert_eq!(
        failing(&[
            ("interest", json!("other")),
            ("agreement", json!("none")),
            ("acquisition", json!("taxable_supply_without_margin_scheme")),
        ]),
        json!([
            "real_property_interest",
            "written_agreement",
            "acquisition_eligible"
        ])
    );
}

/// A sale for no more than the base has a nil margin and no GST under the
/// margin scheme (s 75-10(2): the amount by which it exceeds the base).
#[test]
fn a_sale_at_or_below_the_base_has_a_nil_margin() {
    for price in ["200000.00", "180000.00"] {
        let r = run(&sale("other_supply_from_1_july_2000", price, "200000.00"));
        assert_eq!(amount(&r, "margin"), "0.00");
        assert_eq!(amount(&r, "gst"), "0.00");
    }
}

/// Consideration still unpaid when the land is sold on reduces the base
/// (s 75-12): $300,000 with $50,000 unpaid leaves $250,000.
#[test]
fn unpaid_consideration_reduces_the_base() {
    let input = with(
        sale("other_supply_from_1_july_2000", "360000.00", "300000.00"),
        &[("unpaid_acquisition_consideration", json!("50000.00"))],
    );
    let r = run(&input);
    assert_eq!(amount(&r, "margin_base"), "250000.00");
    assert_eq!(amount(&r, "margin"), "110000.00");
    let input = with(
        input,
        &[("unpaid_acquisition_consideration", json!("300000.01"))],
    );
    assert_eq!(code(&input), "gst_margin_scheme.fact_conflict");
}

/// s 75-10(3) item 2: a supplier registered after 1 July 2000 uses a
/// valuation as at its registration, or the earlier day it applied, which may
/// be on or before 1 July 2000; item 4: government land unimproved on
/// 1 July 2000 uses one as at the day of the supply, in its year.
#[test]
fn the_valuation_day_follows_the_item() {
    let late = with(
        valued("500000.00", "300000.00", "2004-03-01"),
        &[("registered_on_1_july_2000", json!(false))],
    );
    let r = run(&late);
    assert!(r["base_rule"].as_str().unwrap().contains("item 2"));
    assert_eq!(amount(&r, "gst"), "18181.82");
    for day in ["2000-06-30", "2000-07-01"] {
        let applied = with(late.clone(), &[("valuation_day", json!(day))]);
        assert!(
            run(&applied)["base_rule"]
                .as_str()
                .unwrap()
                .contains("item 2")
        );
    }
    let bad = with(late.clone(), &[("valuation_day", json!("2026-07-01"))]);
    assert_eq!(code(&bad), "gst_margin_scheme.valuation_day_invalid");
    let crown = with(
        valued("500000.00", "500000.00", "2026-02-10"),
        &[("government_unimproved_on_1_july_2000", json!(true))],
    );
    let r = run(&crown);
    assert!(r["base_rule"].as_str().unwrap().contains("item 4"));
    assert_eq!(amount(&r, "gst"), "0.00");
    for day in ["2000-07-01", "2025-06-30", "2026-07-01"] {
        let bad = with(crown.clone(), &[("valuation_day", json!(day))]);
        assert_eq!(
            code(&bad),
            "gst_margin_scheme.valuation_day_invalid",
            "{day}"
        );
    }
    let both = with(crown, &[("registered_on_1_july_2000", json!(false))]);
    assert_eq!(code(&both), "gst_margin_scheme.valuation_item_unsettled");
    let bad = valued("640000.00", "448000.00", "2000-06-30");
    assert_eq!(code(&bad), "gst_margin_scheme.valuation_day_invalid");
}

/// An approved valuation is only for an interest acquired before 1 July 2000.
#[test]
fn a_valuation_needs_an_interest_held_before_1_july_2000() {
    for acquisition in [
        "taxable_supply_under_margin_scheme",
        "taxable_supply_without_margin_scheme",
        "other_supply_from_1_july_2000",
    ] {
        let input = with(
            valued("640000.00", "448000.00", "2000-07-01"),
            &[("acquisition", json!(acquisition))],
        );
        assert_eq!(code(&input), "gst_margin_scheme.valuation_unavailable");
    }
}

/// A supply to an associate is refused: s 75-13 works its margin from the
/// GST inclusive market value, which is not an input.
#[test]
fn a_supply_to_an_associate_is_refused() {
    let input = sale("other_supply_from_1_july_2000", "330000.00", "220000.00");
    let input = with(input, &[("recipient_associate", json!(true))]);
    let refused = api::evaluate_coded("gst-margin-scheme", &input).unwrap_err();
    assert_eq!(refused.code, "gst_margin_scheme.supply_to_associate");
    assert!(refused.message.contains("s 75-13"));
    let input = with(input, &[("recipient_associate", json!("no"))]);
    assert_eq!(code(&input), "input.flag_invalid");
    let input = sale("other_supply_from_1_july_2000", "330000.00", "220000.00");
    assert_eq!(
        code(&without(input, "recipient_associate")),
        "input.missing_field"
    );
}

/// A half cent of GST rounds up, from the unrounded apportioned base: 1/2 of
/// $0.09 is $0.045 (shown as $0.05), the margin $0.055 and the GST $0.005.
#[test]
fn half_a_cent_of_gst_rounds_up_from_the_unrounded_base() {
    let input = sale("other_supply_from_1_july_2000", "0.10", "0.09");
    let input = with(
        input,
        &[
            ("subdivision_share", json!("1")),
            ("subdivision_whole", json!("2")),
        ],
    );
    let r = run(&input);
    assert_eq!(amount(&r, "apportioned_base"), "0.05");
    assert_eq!(amount(&r, "margin"), "0.06");
    assert_eq!(amount(&r, "gst"), "0.01");
}

/// Acquisitions that s 75-5(3)(b) to (g), s 75-10(3) item 2A, s 75-11 or
/// s 75-22 govern are refused, naming the provision.
#[test]
fn acquisitions_this_does_not_work_are_refused() {
    for (acquisition, provision) in [
        ("gst_group_member", "s 75-11(1)"),
        ("joint_venture_operator", "s 75-11(2A)"),
        ("inheritance", "s 75-11(3)"),
        ("gst_free_going_concern", "s 75-11(5)"),
        ("gst_free_farm_land", "Subdivision 38-O"),
        ("associate", "s 75-11(6)"),
        ("gst_free_government_land", "item 2A"),
        ("part_through_ineligible_supply", "s 75-22"),
    ] {
        let input = sale(acquisition, "500000.00", "300000.00");
        let refused = api::evaluate_coded("gst-margin-scheme", &input).unwrap_err();
        assert_eq!(refused.code, "gst_margin_scheme.acquisition_unsupported");
        assert!(refused.message.contains(provision), "{acquisition}");
    }
}

/// Each fact outside its values, each input missing or out of place, and
/// each period outside the years read is refused.
#[test]
fn bad_inputs_are_refused() {
    let base = sale("other_supply_from_1_july_2000", "500000.00", "300000.00");
    for (key, value) in [
        ("interest", json!("leasehold")),
        ("agreement", json!("oral")),
        ("acquisition", json!("gift")),
        ("method", json!("market_value")),
    ] {
        let input = with(base.clone(), &[(key, value)]);
        assert_eq!(code(&input), "gst_margin_scheme.fact_invalid", "{key}");
    }
    for key in [
        "acquisition_consideration",
        "unpaid_acquisition_consideration",
    ] {
        assert_eq!(
            code(&without(base.clone(), key)),
            "input.missing_field",
            "{key}"
        );
    }
    for key in [
        "approved_valuation",
        "valuation_day",
        "registered_on_1_july_2000",
        "government_unimproved_on_1_july_2000",
    ] {
        let joanne = valued("640000.00", "448000.00", "2000-07-01");
        assert_eq!(
            code(&without(joanne.clone(), key)),
            "input.missing_field",
            "{key}"
        );
        let input = with(base.clone(), &[(key, joanne[key].clone())]);
        assert_eq!(code(&input), "input.field_unknown", "{key}");
    }
    let input = with(
        valued("640000.00", "448000.00", "2000-07-01"),
        &[("acquisition_consideration", json!("1.00"))],
    );
    assert_eq!(code(&input), "input.field_unknown");
    let input = with(base.clone(), &[("subdivision_share", json!("1"))]);
    assert_eq!(code(&input), "input.missing_field");
    let input = with(base.clone(), &[("subdivision_whole", json!("1"))]);
    assert_eq!(code(&input), "input.missing_field");
    for share in ["0", "3.01"] {
        let input = with(
            base.clone(),
            &[
                ("subdivision_share", json!(share)),
                ("subdivision_whole", json!("3")),
            ],
        );
        assert_eq!(code(&input), "gst_margin_scheme.fact_conflict", "{share}");
    }
    let input = with(
        valued("640000.00", "448000.00", "2000-07-01"),
        &[("registered_on_1_july_2000", json!("yes"))],
    );
    assert_eq!(code(&input), "input.flag_invalid");
    let input = with(valued("640000.00", "448000.00", "1 July 2000"), &[]);
    assert_eq!(code(&input), "input.date_invalid");
    assert_eq!(
        code(&with(base.clone(), &[("consideration", json!("-1"))])),
        "input.amount_invalid"
    );
    assert_eq!(
        code(&with(base.clone(), &[("year", json!("2024-25"))])),
        "period.unsupported"
    );
    assert_eq!(
        code(&with(base.clone(), &[("scope_confirmed", json!(false))])),
        "scope.not_confirmed"
    );
    assert_eq!(
        code(&with(base, &[("not_an_input", json!(true))])),
        "input.field_unknown"
    );
}
