// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Trading in a depreciating asset for a replacement (ITAA 1997 Subdivisions
//! 27-B and 40-D, ss 40-180 and 40-185, compilation 268; GST Act ss 9-75,
//! 11-30, 69-10 and 132-5, compilation 96) against the ATO's GST examples for
//! Darren's car and Ian's vehicle, read on 18 November 2026, and hand-worked
//! cases (docs/calculation-evidence.md#trade-in). The examples are each one
//! side of a trade-in; the other side and the years held are fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// Ian's vehicle (GST and the disposal of capital assets, 'Capital assets
/// used for private purposes'): $44,000 including $4,000 GST, used 80% in his
/// business, so $3,200 of input tax credit and a cost of $40,800; one year's
/// decline of $10,200 is fabricated. It is traded in for $22,000, a taxable
/// sale, towards Darren's car (Purchasing a motor vehicle, 'Example:
/// purchasing a new car for the business'): $91,242.80 including $8,075.91
/// GST and $2,407.80 luxury car tax, 2026-27.
fn ian_for_darren(purpose: &str) -> Value {
    json!({
        "cost": "40800.00", "asset": "car", "car_limit": null,
        "years": [{"income_year": "2025-26", "decline_in_value": "10200.00",
                   "method": "log_book", "deduction": "8160.00", "days_held": 365}],
        "trade_in_allowance": "22000.00", "trade_in_taxable": true,
        "adjusted_input_tax_credit": "3200.00", "full_input_tax_credit": "4000.00",
        "replacement_price": "91242.80", "replacement_asset": "passenger_car",
        "replacement_luxury_car_tax": "2407.80", "replacement_creditable_acquisition": true,
        "creditable_purpose": purpose, "arms_length": true, "involuntary_disposal": false,
        "year": "2026-27", "scope_confirmed": true,
    })
}

fn run(input: &Value) -> Value {
    let result = api::evaluate("trade-in", input).unwrap();
    // derive gives the same result.
    let envelope = json!({"command": "trade-in", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("trade-in", input).unwrap_err().code
}

fn amounts<'a, const N: usize>(result: &'a Value, keys: [&str; N]) -> [&'a str; N] {
    keys.map(|key| result["amounts"][key].as_str().unwrap())
}

/// Ian's decreasing adjustment is $400 (1/11 x $22,000 x (1 - $3,200 /
/// $4,000)), so the termination value is $22,000 - $2,000 + $400 = $20,400
/// (ITAA 1997 s 27-95(1) and (3)): against $30,600 adjustable value, $10,200
/// less its 20% private share ($2,040) is deductible.
#[test]
fn ian_trade_in_gives_the_termination_value_after_gst() {
    let r = run(&ian_for_darren("0.8"));
    let old = [
        "trade_in_gst",
        "decreasing_adjustment",
        "termination_value",
        "adjustable_value",
        "reduction_for_non_taxable_use",
        "assessable_amount",
        "deductible_amount",
        "capital_loss",
    ];
    assert_eq!(
        amounts(&r, old),
        [
            "2000.00", "400.00", "20400.00", "30600.00", "2040.00", "0.00", "8160.00", "0.00"
        ]
    );
}

/// Darren's credit is capped at 1/11 of the 2026-27 car limit, $6,353, and at
/// 50% business use is $3,176.50; the car's cost is then reduced to the
/// $69,883 car limit (ITAA 1997 s 40-230).
#[test]
fn darren_car_credit_is_capped_at_the_car_limit() {
    let keys = [
        "replacement_full_input_tax_credit",
        "replacement_input_tax_credit",
        "replacement_cost_ignoring_car_limit",
        "replacement_cost",
    ];
    let r = run(&ian_for_darren("1"));
    assert_eq!(
        amounts(&r, keys),
        ["6353.00", "6353.00", "84889.80", "69883.00"]
    );
    assert_eq!(r["rates"]["car_limit"], "69883");
    let r = run(&ian_for_darren("0.5"));
    assert_eq!(
        amounts(&r, keys),
        ["6353.00", "3176.50", "88066.30", "69883.00"]
    );
    // The catalogue's example, at 80%.
    let r = run(&ian_for_darren("0.8"));
    assert_eq!(
        amounts(&r, keys),
        ["6353.00", "5082.40", "86160.40", "69883.00"]
    );
}

/// Worked by hand: an old car whose cost s 40-230 reduced from $80,000 to the
/// 2025-26 car limit of $69,674 (its input tax credit $6,334, a full one,
/// 1/11 of that limit) is traded in for $55,000 including $5,000 GST. The
/// $50,000 after GST is scaled by 69,674 / 80,000 (s 40-325) to $43,546.25,
/// and against its $59,674 adjustable value $16,127.75 is deductible.
#[test]
fn old_car_under_the_car_limit_has_its_termination_value_scaled() {
    let mut input = ian_for_darren("1");
    let facts = input.as_object_mut().unwrap();
    facts.insert("cost".into(), json!("69674.00"));
    facts.insert(
        "car_limit".into(),
        json!({"limit": "69674.00", "second_element_cost": "0.00", "cost_ignoring_car_limit": "80000.00"}),
    );
    facts.insert(
        "years".into(),
        json!([{"income_year": "2025-26", "decline_in_value": "10000.00",
                "method": "log_book", "deduction": "10000.00", "days_held": 365}]),
    );
    facts.insert("trade_in_allowance".into(), json!("55000.00"));
    facts.insert("adjusted_input_tax_credit".into(), json!("6334.00"));
    facts.insert("full_input_tax_credit".into(), json!("6334.00"));
    let r = run(&input);
    let keys = [
        "trade_in_gst",
        "decreasing_adjustment",
        "termination_value",
        "deductible_amount",
    ];
    assert_eq!(
        amounts(&r, keys),
        ["5000.00", "0.00", "43546.25", "16127.75"]
    );
}

/// Worked by hand: GST Act s 132-5(3) caps the decreasing adjustment at the
/// full input tax credit less the adjusted one. A $11,000 asset (full credit
/// $1,000, $800 claimed at 80%) traded in for $22,000: 1/11 x $22,000 x 0.2 =
/// $400 is capped at $200.
#[test]
fn decreasing_adjustment_is_capped() {
    let input = json!({
        "cost": "10200.00", "asset": "other",
        "years": [{"income_year": "2025-26", "decline_in_value": "1000.00", "deduction": "800.00"}],
        "trade_in_allowance": "22000.00", "trade_in_taxable": true,
        "adjusted_input_tax_credit": "800.00", "full_input_tax_credit": "1000.00",
        "replacement_price": "11000.00", "replacement_asset": "other",
        "replacement_creditable_acquisition": false,
        "arms_length": true, "involuntary_disposal": false,
        "year": "2025-26", "scope_confirmed": true,
    });
    let r = run(&input);
    let keys = [
        "trade_in_gst",
        "decreasing_adjustment",
        "termination_value",
        "capital_gain",
    ];
    // A gain of ($20,200 - $10,200) x 200 / 1,000 = $2,000 under s 104-240.
    assert_eq!(
        amounts(&r, keys),
        ["2000.00", "200.00", "20200.00", "2000.00"]
    );
    // Not a creditable acquisition: no credit, and the cost is the price.
    let keys = ["replacement_input_tax_credit", "replacement_cost"];
    assert_eq!(amounts(&r, keys), ["0.00", "11000.00"]);
    // A creditable one wholly for a creditable purpose: $1,000 off.
    let mut input = input;
    input["replacement_creditable_acquisition"] = json!(true);
    input["creditable_purpose"] = json!("1");
    assert_eq!(amounts(&run(&input), keys), ["1000.00", "10000.00"]);
}

/// A trade-in that is not a taxable supply has no GST to take off.
#[test]
fn a_trade_in_that_is_not_taxable_keeps_the_whole_allowance() {
    let mut input = ian_for_darren("1");
    let facts = input.as_object_mut().unwrap();
    facts.insert("trade_in_taxable".into(), json!(false));
    facts.remove("adjusted_input_tax_credit");
    facts.remove("full_input_tax_credit");
    let r = run(&input);
    let keys = ["trade_in_gst", "decreasing_adjustment", "termination_value"];
    assert_eq!(amounts(&r, keys), ["0.00", "0.00", "22000.00"]);
}

#[test]
fn refusals_name_what_is_out_of_scope_or_wrong() {
    let with = |key: &str, value: Value| {
        let mut input = ian_for_darren("1");
        input[key] = value;
        input
    };
    let without = |key: &str| {
        let mut input = ian_for_darren("1");
        input.as_object_mut().unwrap().remove(key);
        input
    };
    let cases = [
        (
            with("arms_length", json!(false)),
            "trade_in.not_arms_length",
        ),
        (
            with("involuntary_disposal", json!(true)),
            "trade_in.involuntary_disposal",
        ),
        (
            with("adjusted_input_tax_credit", json!("4000.01")),
            "trade_in.fact_conflict",
        ),
        (
            with("full_input_tax_credit", json!("0.00")),
            "trade_in.fact_conflict",
        ),
        (
            with("creditable_purpose", json!("0")),
            "trade_in.fact_conflict",
        ),
        (
            with("creditable_purpose", json!("1.01")),
            "input.fraction_invalid",
        ),
        (
            with("replacement_luxury_car_tax", json!("91242.81")),
            "trade_in.fact_conflict",
        ),
        (
            with("replacement_asset", json!("boat")),
            "trade_in.asset_invalid",
        ),
        (with("trade_in_taxable", json!("yes")), "input.flag_invalid"),
        (with("year", json!("2027-28")), "period.unsupported"),
        (with("year", json!("2024-25")), "period.unsupported"),
        (
            with("trade_in_gst", json!("2000.00")),
            "input.field_unknown",
        ),
        (
            with("replacement_creditable_acquisition", json!(false)),
            "input.field_unknown",
        ),
        (
            with("replacement_asset", json!("other")),
            "input.field_unknown",
        ),
        (
            with("trade_in_taxable", json!(false)),
            "input.field_unknown",
        ),
        (without("full_input_tax_credit"), "input.missing_field"),
        (without("replacement_luxury_car_tax"), "input.missing_field"),
        (without("creditable_purpose"), "input.missing_field"),
        (without("car_limit"), "input.missing_field"),
    ];
    for (input, expected) in cases {
        assert_eq!(code(&input), expected, "{input}");
    }
    // The refusal says what s 40-365 would change.
    let refused =
        api::evaluate("trade-in", &with("involuntary_disposal", json!(true))).unwrap_err();
    assert!(refused.contains("s 40-365"), "{refused}");
    assert!(
        refused.contains("reduce the replacement's cost"),
        "{refused}"
    );
}
