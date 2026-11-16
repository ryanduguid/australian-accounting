// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Business capital expenditure (ITAA 1997 s 40-880) against the
//! Commissioner's examples in TR 2011/6 and the ATO's 'Deductions for
//! depreciating assets and capital expenses' page, both read on
//! 16 November 2026, with the worksheet's refusals
//! (docs/calculation-evidence.md#business-capital-expenditure). TR 2011/6
//! states the Commissioner's view, not the law; the ruling's examples date
//! from 2007 to 2010, so the ones with figures are run with the year moved
//! into the current form of s 40-880, from 1 July 2020.
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A current business's item, wholly for a taxable purpose, with nothing
/// excluding it, changed by `overrides`.
fn item(overrides: Value) -> Value {
    let mut item = json!({
        "amount": "10000.00", "incurred_in": "2020-21", "capital": true,
        "business": "current", "taxable_purpose": "1", "exclusions": [],
    });
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            item.as_object_mut().unwrap().remove(k);
        } else {
            item[k] = v.clone();
        }
    }
    item
}

/// A proposed business's item: another entity's, connected to the
/// taxpayer's assessable income, to start within a reasonable time.
fn proposed(overrides: Value) -> Value {
    let mut fields = json!({
        "business": "proposed", "carried_on_by": "another_entity",
        "assessable_income_connection": true, "within_reasonable_time": true,
        "start_up_expense": "government_fees", "entity_test": "not_in_business_and_unconnected",
    });
    for (k, v) in overrides.as_object().unwrap() {
        fields[k] = v.clone();
    }
    item(fields)
}

fn run(items: Value, year: &str) -> Result<Value, String> {
    let input = json!({"items": items, "year": year, "scope_confirmed": true});
    let result = api::evaluate("business-capital-expenditure", &input);
    // derive gives the same result, or the same refusal.
    let envelope = json!({"command": "business-capital-expenditure", "input": input});
    let doc = api::evaluate("derive", &envelope);
    assert_eq!(
        doc.as_ref().map(|doc| doc["result"].to_string()),
        result.as_ref().map(Value::to_string),
        "derive {input}"
    );
    result
}

fn code(items: Value, year: &str) -> &'static str {
    let input = json!({"items": items, "year": year, "scope_confirmed": true});
    api::evaluate_coded("business-capital-expenditure", &input)
        .unwrap_err()
        .code
}

/// Expenditure, deductible expenditure, deduction and what remains after the year.
fn amounts(result: &Value) -> [&str; 4] {
    [
        "expenditure",
        "deductible_expenditure",
        "deduction",
        "remaining_expenditure_after_year",
    ]
    .map(|key| result["amounts"][key].as_str().unwrap())
}

fn schedule(row: &Value) -> Vec<(&str, &str)> {
    row["schedule"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["income_year"].as_str().unwrap(),
                r["deduction"].as_str().unwrap(),
            )
        })
        .collect()
}

/// TR 2011/6 example 41 (para 321, with paras 10, 319 and 320): $10,000
/// incurred on 1 September gives $2,000 in each income year, the first not
/// apportioned for the 2 months before it was incurred. The ruling's 2007-08
/// is before the current form, so the item is incurred in 2020-21.
#[test]
fn example_41_a_fifth_a_year_with_no_part_year_apportionment() {
    let r = run(json!([item(json!({}))]), "2022-23").unwrap();
    assert_eq!(amounts(&r), ["10000.00", "10000.00", "2000.00", "4000.00"]);
    let row = &r["items"][0];
    assert_eq!(
        schedule(row),
        [
            ("2020-21", "2000.00"),
            ("2021-22", "2000.00"),
            ("2022-23", "2000.00"),
            ("2023-24", "2000.00"),
            ("2024-25", "2000.00"),
        ]
    );
    assert!(
        row["business"]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(2)(a)")
    );
    assert!(
        row["limit"]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(3)")
    );
    assert!(
        row["method"]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(2): in equal proportions over 5 income years")
    );
    // The fifth year takes the last fifth and the sixth nothing.
    let r = run(json!([item(json!({}))]), "2024-25").unwrap();
    assert_eq!(amounts(&r)[2..], ["2000.00", "0.00"]);
    let r = run(json!([item(json!({}))]), "2025-26").unwrap();
    assert_eq!(amounts(&r)[2..], ["0.00", "0.00"]);
    // The ruling's own year is before s 40-880's current form.
    assert_eq!(
        code(json!([item(json!({"incurred_in": "2007-08"}))]), "2022-23"),
        "business_capital.incurred_before_july_2020"
    );
}

/// The ATO's page, 'Other capital expenses': the fee for creating a company
/// to carry on a proposed business is a government agency's fee relating to
/// establishing its operating structure, deducted at once by an entity not in
/// business (s 40-880(2A)(b)(ii) and (c)(i) and (ii)); advice on a business
/// structure, by a small business entity (s 40-880(2A)(b)(i)).
#[test]
fn the_ato_start_up_examples_are_deducted_in_the_year_incurred() {
    let fee = proposed(json!({"amount": "576.00", "incurred_in": "2025-26"}));
    let advice = proposed(json!({
        "amount": "3300.00", "incurred_in": "2025-26", "carried_on_by": "you",
        "assessable_income_connection": null, "start_up_expense": "advice_or_services",
        "entity_test": "small_business_entity",
    }));
    let r = run(json!([fee, advice]), "2025-26").unwrap();
    assert_eq!(amounts(&r), ["3876.00", "3876.00", "3876.00", "0.00"]);
    assert_eq!(schedule(&r["items"][0]), [("2025-26", "576.00")]);
    let method = r["items"][0]["method"].as_str().unwrap();
    assert!(method.starts_with("ITAA 1997 s 40-880(2A): all of it in the income year incurred"));
    assert!(
        method.contains("(s 40-880(2A)(b)(ii))")
            && method.contains("(s 40-880(2A)(c)(i) and (ii))")
    );
    assert!(
        r["items"][0]["limit"]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(4)(a)")
    );
    assert!(
        r["items"][1]["limit"]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(3)")
    );
    // The next year takes nothing more.
    let r = run(json!([fee]), "2026-27").unwrap();
    assert_eq!(amounts(&r)[2..], ["0.00", "0.00"]);
    // Without an entity test, or for other start-up expenditure, s 40-880(2) spreads it.
    for overrides in [
        json!({"amount": "576.00", "incurred_in": "2025-26", "entity_test": "none"}),
        json!({"amount": "576.00", "incurred_in": "2025-26", "start_up_expense": "neither", "entity_test": null}),
    ] {
        let r = run(json!([proposed(overrides.clone())]), "2025-26").unwrap();
        assert_eq!(
            amounts(&r),
            ["576.00", "576.00", "115.20", "460.80"],
            "{overrides}"
        );
        assert_eq!(r["items"][0]["notes"].as_array().unwrap().len(), 1);
    }
}

/// TR 2011/6 example 14 (paras 128 and 129): a business proposed to start
/// when approvals that take 2 years are obtained is proposed within a
/// reasonable time; were it not, s 40-880(7) would leave nothing to deduct.
#[test]
fn example_14_a_proposed_business_within_a_reasonable_time() {
    let legal = |reasonable: bool| {
        proposed(json!({
            "amount": "25000.00", "incurred_in": "2024-25", "carried_on_by": "you",
            "within_reasonable_time": reasonable, "start_up_expense": "neither",
            "assessable_income_connection": null, "entity_test": null,
        }))
    };
    let r = run(json!([legal(true)]), "2024-25").unwrap();
    assert_eq!(amounts(&r), ["25000.00", "25000.00", "5000.00", "20000.00"]);
    let r = run(json!([legal(false)]), "2024-25").unwrap();
    assert_eq!(amounts(&r), ["25000.00", "0.00", "0.00", "0.00"]);
    let row = &r["items"][0];
    assert_eq!(
        row["excluded_by"],
        json!([
            "ITAA 1997 s 40-880(7): it is not reasonable to conclude that the business is proposed to be carried on within a reasonable time"
        ])
    );
    assert_eq!(row["method"], Value::Null);
    assert_eq!(schedule(row), []);
}

/// TR 2011/6 examples 16 to 18 (paras 163 to 173): a business carried on 80%,
/// 87.5% and 25% for a taxable purpose. s 40-880(3) applies the extent to the
/// expenditure; the $10,000 is fabricated.
#[test]
fn examples_16_to_18_the_taxable_purpose_extent() {
    for (extent, deductible, fifth) in [
        ("0.8", "8000.00", "1600.00"),
        ("0.875", "8750.00", "1750.00"),
        ("0.25", "2500.00", "500.00"),
    ] {
        let r = run(json!([item(json!({"taxable_purpose": extent}))]), "2020-21").unwrap();
        assert_eq!(amounts(&r)[1..3], [deductible, fifth], "{extent}");
    }
}

/// TR 2011/6 example 21 (paras 189 and 190): a company that is not a
/// shareholder of the holding company it establishes cannot derive
/// assessable income from that company's business, so s 40-880(4)(b)(i)
/// is not met.
#[test]
fn example_21_no_connection_to_your_assessable_income() {
    let holding = proposed(
        json!({"assessable_income_connection": false, "start_up_expense": "neither", "entity_test": null}),
    );
    let r = run(json!([holding]), "2020-21").unwrap();
    assert_eq!(amounts(&r)[1..], ["0.00", "0.00", "0.00"]);
    assert!(
        r["items"][0]["excluded_by"][0]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(4)(b)")
    );
    // A former business of another entity needs the same fact.
    let former = item(json!({"business": "former", "carried_on_by": "another_entity"}));
    assert_eq!(
        code(json!([former]), "2020-21"),
        "business_capital.fact_missing"
    );
}

/// TR 2011/6 examples 22 and 23 (paras 201 to 205): inspecting an asset that
/// is then bought forms part of its cost (s 40-880(5)(a)), which Division 40
/// takes; inspecting one that is not bought does not.
#[test]
fn examples_22_and_23_the_cost_of_a_depreciating_asset() {
    let bought = item(json!({"exclusions": ["5(a)"]}));
    assert_eq!(
        code(json!([bought]), "2020-21"),
        "business_capital.depreciating_asset_cost"
    );
    let message = run(json!([bought]), "2020-21").unwrap_err();
    assert!(message.contains("Division 40 (s 40-25"), "{message}");
    let r = run(json!([item(json!({}))]), "2020-21").unwrap();
    assert_eq!(amounts(&r)[2], "2000.00");
}

/// TR 2011/6 examples 24 and 25 (paras 209 to 212): expenditure that a
/// project pool or capital works deduction takes, claimed or not, is
/// expenditure you can deduct under another provision (s 40-880(5)(b)).
#[test]
fn examples_24_and_25_deductible_under_another_provision() {
    let r = run(json!([item(json!({"exclusions": ["5(b)"]}))]), "2020-21").unwrap();
    assert_eq!(amounts(&r), ["10000.00", "0.00", "0.00", "0.00"]);
    assert_eq!(
        r["items"][0]["excluded_by"],
        json!([
            "ITAA 1997 s 40-880(5)(b): you can deduct an amount for it under another provision"
        ])
    );
}

/// TR 2011/6 example 40A (paras 318A to 318D): a gaming machine entitlement's
/// price is the first element of its cost base, so s 40-880(5)(f) excludes it
/// and s 40-880(6) does not apply. Where s 40-880(6) does apply, (5)(d) and
/// (f) do not exclude it.
#[test]
fn example_40a_a_cgt_asset_and_the_goodwill_exception() {
    let entitlement = item(json!({"exclusions": ["5(f)"], "goodwill_right": false}));
    assert_eq!(
        code(json!([entitlement]), "2020-21"),
        "business_capital.cgt_amount"
    );
    let message = run(json!([entitlement]), "2020-21").unwrap_err();
    assert!(message.contains("cost base (s 110-25)"), "{message}");
    let goodwill = item(json!({"exclusions": ["5(d)", "5(f)"], "goodwill_right": true}));
    let r = run(json!([goodwill]), "2020-21").unwrap();
    assert_eq!(amounts(&r)[2], "2000.00");
    assert!(
        r["items"][0]["notes"][0]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(6)")
    );
    // Without s 40-880(6) a lease or right excludes it.
    let lease = item(json!({"exclusions": ["5(d)"], "goodwill_right": false}));
    let r = run(json!([lease]), "2020-21").unwrap();
    assert_eq!(amounts(&r)[1], "0.00");
    // s 40-880(6) is a fact (5)(d) and (f) need, and only they take.
    assert_eq!(
        code(json!([item(json!({"exclusions": ["5(d)"]}))]), "2020-21"),
        "business_capital.fact_missing"
    );
    assert_eq!(
        code(json!([item(json!({"goodwill_right": true}))]), "2020-21"),
        "input.field_unknown"
    );
}

/// Each item is exact; the result's amounts are shown to the cent, half up.
#[test]
fn items_add_up_and_each_item_is_exact() {
    let odd = item(json!({"amount": "1666.67", "incurred_in": "2024-25"}));
    let wind_up = item(json!({
        "amount": "900.00", "incurred_in": "2025-26", "business": "wind_up",
        "assessable_income_connection": true, "taxable_purpose": "0.5",
    }));
    let r = run(json!([odd, wind_up]), "2025-26").unwrap();
    // 1,666.67 / 5 = 333.334; 900 x 0.5 / 5 = 90.
    assert_eq!(r["items"][0]["deduction"], "333.334");
    assert_eq!(r["items"][1]["deduction"], "90.00");
    assert_eq!(amounts(&r), ["2566.67", "2116.67", "423.33", "1360.00"]);
    assert_eq!(r["rates"]["items"], "2");
    assert!(
        r["items"][1]["business"]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(2)(d)")
    );
    assert!(
        r["items"][1]["limit"]
            .as_str()
            .unwrap()
            .starts_with("ITAA 1997 s 40-880(4)(a)")
    );
}

#[test]
fn refusals_name_what_is_missing() {
    let year = "2025-26";
    let refused = |items: Value| run(items, year).unwrap_err();
    assert_eq!(
        refused(json!([item(json!({"capital": false}))])),
        "item 1: s 40-880 deducts capital expenditure only; a revenue outlay is a question for s 8-1, which this worksheet does not decide."
    );
    assert_eq!(
        code(json!([item(json!({"capital": false}))]), year),
        "business_capital.not_capital"
    );
    assert_eq!(
        refused(json!([
            item(json!({})),
            item(json!({"business": "former"}))
        ])),
        "item 2: carried_on_by is required: s 40-880(3) limits your own former or proposed business and s 40-880(4) another entity's."
    );
    let no_start_up = item(json!({
        "business": "proposed", "carried_on_by": "you", "within_reasonable_time": true,
    }));
    assert_eq!(
        refused(json!([no_start_up])),
        "item 1: start_up_expense is required: s 40-880(2A)(b) decides whether a proposed business's expenditure is deducted at once."
    );
    assert_eq!(
        refused(json!([item(
            json!({"business": "proposed", "carried_on_by": "you"})
        )])),
        "item 1: within_reasonable_time is required: s 40-880(7) limits expenditure on a proposed business."
    );
    for (bad, expected) in [
        (json!({"amount": "-1.00"}), "input.amount_invalid"),
        (json!({"amount": "NaN"}), "input.amount_invalid"),
        (json!({"amount": "Infinity"}), "input.amount_invalid"),
        (json!({"amount": 100}), "input.amount_invalid"),
        (json!({"amount": null}), "business_capital.fact_missing"),
        (json!({"capital": null}), "business_capital.fact_missing"),
        (json!({"capital": "yes"}), "business_capital.fact_invalid"),
        (
            json!({"incurred_in": "2026-27"}),
            "business_capital.incurred_after_year",
        ),
        (
            json!({"incurred_in": "2019-20"}),
            "business_capital.incurred_before_july_2020",
        ),
        (
            json!({"incurred_in": "2025-27"}),
            "business_capital.fact_invalid",
        ),
        (
            json!({"business": "hobby"}),
            "business_capital.fact_invalid",
        ),
        (json!({"taxable_purpose": "1.5"}), "input.fraction_invalid"),
        (
            json!({"exclusions": ["5(k)"]}),
            "business_capital.fact_invalid",
        ),
        (
            json!({"exclusions": ["5(b)", "5(b)"]}),
            "business_capital.fact_invalid",
        ),
        (json!({"exclusions": null}), "business_capital.fact_missing"),
        (json!({"carried_on_by": "you"}), "input.field_unknown"),
        (json!({"label": "fees"}), "input.field_unknown"),
    ] {
        assert_eq!(code(json!([item(bad.clone())]), year), expected, "{bad}");
    }
    assert_eq!(
        code(
            json!([proposed(json!({"start_up_expense": "neither"}))]),
            year
        ),
        "input.field_unknown"
    );
    assert_eq!(code(json!([]), year), "business_capital.items_invalid");
    assert_eq!(
        code(json!(["item"]), year),
        "business_capital.items_invalid"
    );
    assert_eq!(code(json!({}), year), "business_capital.items_invalid");
    assert_eq!(
        code(json!([item(json!({}))]), "2027-28"),
        "period.unsupported"
    );
    let unconfirmed = json!({"items": [item(json!({}))], "year": year, "scope_confirmed": false});
    assert_eq!(
        api::evaluate_coded("business-capital-expenditure", &unconfirmed)
            .unwrap_err()
            .code,
        "scope.not_confirmed"
    );
}

#[test]
fn the_catalogue_lists_the_worksheet_with_a_runnable_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["business_capital_expenditure"];
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = api::evaluate("business-capital-expenditure", &facts).unwrap();
    // $1,500 / 5 from 2025-26 and the $576 company fee at once in 2026-27.
    assert_eq!(amounts(&r), ["2076.00", "2076.00", "876.00", "900.00"]);
    assert_eq!(entry["supported_periods"][0]["id"], "2020-21");
    assert_eq!(entry["supported_periods"][6]["id"], "2026-27");
    assert_eq!(
        entry["example_evidence"]["note"],
        "docs/calculation-evidence.md#business-capital-expenditure"
    );
}
