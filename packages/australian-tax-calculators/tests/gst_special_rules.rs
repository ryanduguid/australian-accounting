// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! GST special rules: the supply of a going concern (GST Act s 38-325),
//! second-hand goods (Subdivision 66-A) and bad debts (Division 21 with
//! Subdivision 136-A), read on 22 November 2026, against GSTR 2002/5,
//! GSTD 2012/6 and GSTR 2000/2 and each boundary
//! (docs/calculation-evidence.md#gst-going-concern).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const YEARS: [&str; 7] = [
    "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
];

fn with(base: Value, overrides: Value) -> Value {
    let mut input = base;
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
fn run(command: &str, input: &Value) -> Value {
    let result = api::evaluate(command, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": command, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(command: &str, input: &Value) -> &'static str {
    api::evaluate_coded(command, input).unwrap_err().code
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

const GC: &str = "gst-going-concern";

fn going_concern(overrides: Value) -> Value {
    with(
        json!({
            "for_consideration": true, "recipient_registration": "registered",
            "written_agreement": "on_or_before_supply_day", "all_things_necessary": true,
            "carried_on_until_supply": true, "farm_land": false,
            "part_outside_arrangement": false, "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// s 38-325: all 5 conditions met, so the supply is GST-free.
#[test]
fn a_going_concern_meeting_every_condition_is_gst_free() {
    let r = api::evaluate(GC, &going_concern(json!({}))).unwrap();
    assert_eq!(r["gst_free"], true);
    assert_eq!(r["failing_conditions"], json!([]));
    assert_eq!(r["conditions"].as_array().unwrap().len(), 5);
    assert!(r["consequences"][1].as_str().unwrap().contains("s 135-5"));
    let required = going_concern(json!({"recipient_registration": "required_to_be_registered"}));
    assert_eq!(api::evaluate(GC, &required).unwrap()["gst_free"], true);
}

/// Each condition failing in turn; an agreement after the day of the supply
/// does not count (GSTR 2002/5 para 182).
#[test]
fn each_going_concern_condition_can_fail() {
    for (overrides, failing) in [
        (json!({"for_consideration": false}), "for_consideration"),
        (
            json!({"recipient_registration": "neither"}),
            "recipient_registered",
        ),
        (
            json!({"written_agreement": "after_supply_day"}),
            "agreed_in_writing",
        ),
        (json!({"written_agreement": "none"}), "agreed_in_writing"),
        (
            json!({"all_things_necessary": false}),
            "all_things_necessary",
        ),
        (
            json!({"carried_on_until_supply": false}),
            "carried_on_until_supply",
        ),
    ] {
        let r = api::evaluate(GC, &going_concern(overrides.clone())).unwrap();
        assert_eq!(r["gst_free"], false, "{overrides}");
        assert_eq!(r["failing_conditions"], json!([failing]), "{overrides}");
    }
}

/// No fact is inferred: each is required, and the cases outside s 38-325
/// are refused.
#[test]
fn going_concern_refusals() {
    for (overrides, expected) in [
        (json!({"farm_land": true}), "gst_going_concern.farm_land"),
        (
            json!({"part_outside_arrangement": true}),
            "gst_going_concern.part_outside_arrangement",
        ),
        (
            json!({"recipient_registration": "maybe"}),
            "gst_going_concern.fact_invalid",
        ),
        (
            json!({"written_agreement": 1}),
            "gst_going_concern.fact_invalid",
        ),
        (json!({"all_things_necessary": null}), "input.missing_field"),
        (json!({"for_consideration": "yes"}), "input.flag_invalid"),
        (json!({"price": "100.00"}), "input.field_unknown"),
        (json!({"year": "2019-20"}), "period.unsupported"),
    ] {
        assert_eq!(
            code(GC, &going_concern(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}

const SHG: &str = "gst-second-hand-goods";

/// GSTD 2012/6's Finance Co: a $5,500 vehicle from an unregistered vendor.
fn vehicle(overrides: Value) -> Value {
    with(
        json!({
            "consideration": "5500.00", "acquired_for_sale_or_exchange": true,
            "animals_or_plants": false, "supply_to_you": "neither", "imported": false,
            "by_way_of_hire": false, "resupply_taxable": true, "resupply_gst": "800.00",
            "divided_for_resupply": false, "valuable_metal": false,
            "leased_before_sale": false, "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Para 7: 1/11 of $5,500 is $500, under the $800 GST on an $8,800 resale.
#[test]
fn the_vehicles_credit_is_one_eleventh() {
    let r = run(SHG, &vehicle(json!({})));
    assert_eq!(r["applies"], true);
    assert_eq!(amount(&r, "one_eleventh_of_consideration"), "500.00");
    assert_eq!(amount(&r, "input_tax_credit"), "500.00");
}

/// s 66-10(1)(b): resold for $3,300, so the credit is the $300 GST on it.
#[test]
fn the_credit_is_capped_at_the_resale_gst() {
    let r = run(SHG, &vehicle(json!({"resupply_gst": "300.00"})));
    assert_eq!(amount(&r, "input_tax_credit"), "300.00");
}

/// $300 or less takes s 66-10(1A) without a cap; more than $300 needs the
/// resale GST.
#[test]
fn the_three_hundred_dollar_boundary() {
    let at = vehicle(json!({"consideration": "300.00", "resupply_gst": null}));
    let r = run(SHG, &at);
    assert_eq!(amount(&r, "input_tax_credit"), "27.27");
    assert!(r["amounts"].get("one_eleventh_of_consideration").is_none());
    let over = vehicle(json!({"consideration": "300.01", "resupply_gst": "10.00"}));
    assert_eq!(amount(&run(SHG, &over), "input_tax_credit"), "10.00");
    let missing = vehicle(json!({"consideration": "300.01", "resupply_gst": null}));
    assert_eq!(code(SHG, &missing), "input.missing_field");
    let unused = vehicle(json!({"consideration": "300.00", "resupply_gst": "10.00"}));
    assert_eq!(code(SHG, &unused), "input.field_unknown");
}

/// Each s 66-5 condition, and the s 195-1 definition, failing in turn.
#[test]
fn each_second_hand_goods_condition_can_fail() {
    for (overrides, failing) in [
        (
            json!({"acquired_for_sale_or_exchange": false}),
            "acquired_for_sale_or_exchange",
        ),
        (json!({"animals_or_plants": true}), "second_hand_goods"),
        (
            json!({"supply_to_you": "taxable"}),
            "supply_to_you_not_taxable",
        ),
        (
            json!({"supply_to_you": "gst_free"}),
            "supply_to_you_not_taxable",
        ),
        (json!({"imported": true}), "not_imported"),
        (json!({"by_way_of_hire": true}), "not_by_way_of_hire"),
        (json!({"resupply_taxable": false}), "resupply_taxable"),
    ] {
        let r = run(SHG, &vehicle(overrides.clone()));
        assert_eq!(r["applies"], false, "{overrides}");
        assert_eq!(r["failing_conditions"], json!([failing]), "{overrides}");
        assert_eq!(r["amounts"], json!({}), "{overrides}");
    }
}

#[test]
fn second_hand_goods_refusals() {
    for (overrides, expected) in [
        (
            json!({"divided_for_resupply": true}),
            "gst_second_hand_goods.divided_for_resupply",
        ),
        (
            json!({"valuable_metal": true}),
            "gst_second_hand_goods.valuable_metal",
        ),
        (
            json!({"leased_before_sale": true}),
            "gst_second_hand_goods.leased_before_sale",
        ),
        (
            json!({"supply_to_you": "input_taxed"}),
            "gst_second_hand_goods.fact_invalid",
        ),
        (json!({"consideration": "-1.00"}), "input.amount_invalid"),
        (json!({"year": "2027-28"}), "period.unsupported"),
    ] {
        assert_eq!(
            code(SHG, &vehicle(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}

const BD: &str = "gst-bad-debt";

/// GSTR 2000/2's Marcus, 20 years on: $550 overdue from 16 April 2021.
fn marcus(overrides: Value) -> Value {
    with(
        json!({
            "party": "supplier", "event": "overdue_12_months", "amount": "550.00",
            "taxable_or_creditable": true, "cash_basis": false,
            "special_rule": "none", "attributed": true, "overdue_from": "2021-04-16",
            "as_at": "2022-04-16", "year": "2021-22", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// A recipient writing off part of a creditable acquisition's debt.
fn recipient(overrides: Value) -> Value {
    with(
        json!({
            "party": "recipient", "event": "written_off", "amount": "1100.00",
            "taxable_or_creditable": true, "cash_basis": false,
            "special_rule": "none", "attributed": true, "reduced_credit_acquisition": false,
            "annual_apportionment": false, "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Paras 49AC to 53: overdue 12 months on 16 April 2022, a $50 decrease; a
/// day earlier, none.
#[test]
fn marcus_has_a_fifty_dollar_decrease() {
    let r = run(BD, &marcus(json!({})));
    assert_eq!(r["direction"], "decreasing");
    assert_eq!(amount(&r, "decreasing_adjustment"), "50.00");
    let early = run(BD, &marcus(json!({"as_at": "2022-04-15"})));
    assert_eq!(early["applies"], false);
    assert_eq!(
        early["failing_conditions"],
        json!(["written_off_or_overdue_12_months"])
    );
}

/// Para 55: ABC Pty Ltd's $660 at 40% taxable, $60 x 40% = $24 (s 136-5).
#[test]
fn abcs_partly_taxable_supply() {
    let input = marcus(json!({"event": "written_off", "amount": "660.00",
                              "overdue_from": null, "as_at": null, "taxable_proportion": "0.4"}));
    let r = run(BD, &input);
    assert_eq!(amount(&r, "full_adjustment"), "60.00");
    assert_eq!(amount(&r, "decreasing_adjustment"), "24.00");
}

/// Paras 69 and 70: Isabella's $1,100 at 60%, $60; para 72: Y's $4,400 at a
/// 25% extent of consideration, $100 (s 136-10(1)).
#[test]
fn isabella_and_y_have_increases() {
    let isabella = recipient(json!({"creditable_purpose": "0.6", "consideration_extent": "1"}));
    let r = run(BD, &isabella);
    assert_eq!(r["direction"], "increasing");
    assert_eq!(amount(&r, "increasing_adjustment"), "60.00");
    let y = recipient(json!({"amount": "4400.00", "creditable_purpose": "1",
                             "consideration_extent": "0.25"}));
    assert_eq!(amount(&run(BD, &y), "increasing_adjustment"), "100.00");
    let whole = run(BD, &recipient(json!({})));
    assert_eq!(amount(&whole, "increasing_adjustment"), "100.00");
}

/// ss 21-10 and 21-20: a recovery reverses the direction, and needs the
/// earlier adjustment.
#[test]
fn recoveries_reverse_the_adjustment() {
    let supplier = marcus(
        json!({"event": "recovered", "amount": "110.00", "overdue_from": null,
                                 "as_at": null, "attributed": null, "prior_adjustment": true}),
    );
    assert_eq!(
        amount(&run(BD, &supplier), "increasing_adjustment"),
        "10.00"
    );
    let paid = recipient(
        json!({"event": "recovered", "amount": "220.00", "attributed": null,
                         "prior_adjustment": true}),
    );
    assert_eq!(amount(&run(BD, &paid), "decreasing_adjustment"), "20.00");
    let none =
        recipient(json!({"event": "recovered", "attributed": null, "prior_adjustment": false}));
    assert_eq!(
        run(BD, &none)["failing_conditions"],
        json!(["prior_adjustment"])
    );
    // ss 136-5 and 136-10(1) reduce recoveries too: $10 x 40% and
    // $20 x 60% x 50%.
    let partly = marcus(
        json!({"event": "recovered", "amount": "110.00", "overdue_from": null,
                               "as_at": null, "attributed": null, "prior_adjustment": true,
                               "taxable_proportion": "0.4"}),
    );
    assert_eq!(amount(&run(BD, &partly), "increasing_adjustment"), "4.00");
    let repaid = recipient(
        json!({"event": "recovered", "amount": "220.00", "attributed": null,
                                  "prior_adjustment": true, "creditable_purpose": "0.6",
                                  "consideration_extent": "0.5"}),
    );
    assert_eq!(amount(&run(BD, &repaid), "decreasing_adjustment"), "6.00");
}

/// GSTR 2000/2 paras 15 and 21: no adjustment before the GST or the credit is
/// attributed.
#[test]
fn no_adjustment_before_attribution() {
    for input in [
        marcus(json!({"attributed": false})),
        recipient(json!({"attributed": false})),
    ] {
        let r = run(BD, &input);
        assert_eq!(r["applies"], false);
        assert_eq!(r["failing_conditions"], json!(["attributed"]));
    }
}

/// ss 21-5(2) and 21-15(2): no adjustment on a cash basis.
#[test]
fn no_adjustment_on_a_cash_basis() {
    let r = run(BD, &marcus(json!({"cash_basis": true})));
    assert_eq!(r["applies"], false);
    assert_eq!(r["failing_conditions"], json!(["not_cash_basis"]));
    let r = run(BD, &recipient(json!({"taxable_or_creditable": false})));
    assert_eq!(r["failing_conditions"], json!(["taxable_or_creditable"]));
}

#[test]
fn bad_debt_refusals() {
    let supplier = |o: Value| code(BD, &marcus(o));
    let buyer = |o: Value| code(BD, &recipient(o));
    for rule in [
        "taxable_at_less_than_one_eleventh",
        "gambling_supply",
        "debt_assigned",
        "transition_act_s15g",
        "incapacitated_entity_representative",
    ] {
        assert_eq!(
            supplier(json!({"special_rule": rule})),
            "gst_bad_debt.special_rule",
            "{rule}"
        );
    }
    assert_eq!(
        supplier(json!({"special_rule": "other"})),
        "gst_bad_debt.fact_invalid"
    );
    let recovery = json!({"event": "recovered", "attributed": null, "prior_adjustment": true,
                          "cash_basis": true});
    assert_eq!(
        buyer(recovery.clone()),
        "gst_bad_debt.accounting_basis_changed"
    );
    assert_eq!(
        buyer(with(recovery.clone(), json!({"cash_basis": "no"}))),
        "input.flag_invalid"
    );
    assert_eq!(
        buyer(with(
            recovery,
            json!({"cash_basis": false, "attributed": true})
        )),
        "input.field_unknown"
    );
    assert_eq!(supplier(json!({"attributed": null})), "input.missing_field");
    for zero in [
        json!({"taxable_proportion": "0"}),
        json!({"creditable_purpose": "0"}),
        json!({"consideration_extent": "0"}),
    ] {
        let party = if zero.get("taxable_proportion").is_some() {
            supplier(zero.clone())
        } else {
            buyer(zero.clone())
        };
        assert_eq!(party, "gst_bad_debt.fact_conflict", "{zero}");
    }
    assert_eq!(
        buyer(json!({"reduced_credit_acquisition": true})),
        "gst_bad_debt.reduced_credit_acquisition"
    );
    assert_eq!(
        buyer(json!({"annual_apportionment": true})),
        "gst_bad_debt.annual_apportionment"
    );
    assert_eq!(
        buyer(json!({"annual_apportionment": null})),
        "input.missing_field"
    );
    assert_eq!(
        supplier(json!({"reduced_credit_acquisition": false})),
        "input.field_unknown"
    );
    assert_eq!(
        supplier(json!({"overdue_from": null})),
        "input.missing_field"
    );
    assert_eq!(buyer(json!({"as_at": "2025-07-01"})), "input.field_unknown");
    assert_eq!(buyer(json!({"event": "recovered"})), "input.missing_field");
    assert_eq!(
        supplier(json!({"as_at": "2021-04-15"})),
        "gst_bad_debt.dates_invalid"
    );
    assert_eq!(
        supplier(json!({"party": "agent"})),
        "gst_bad_debt.fact_invalid"
    );
    assert_eq!(
        supplier(json!({"event": "disputed"})),
        "gst_bad_debt.fact_invalid"
    );
    assert_eq!(
        buyer(json!({"taxable_proportion": "0.5"})),
        "input.field_unknown"
    );
    assert_eq!(
        supplier(json!({"creditable_purpose": "0.5"})),
        "input.field_unknown"
    );
    assert_eq!(
        supplier(json!({"taxable_proportion": "1.2"})),
        "input.fraction_invalid"
    );
    assert_eq!(
        supplier(json!({"as_at": "16/04/2022"})),
        "input.date_invalid"
    );
}

/// Each command takes every year from 2020-21 to 2026-27 alike.
#[test]
fn the_supported_years() {
    for year in YEARS {
        let gc = api::evaluate(GC, &going_concern(json!({"year": year}))).unwrap();
        assert_eq!(gc["gst_free"], true, "{year}");
        assert_eq!(
            amount(
                &run(SHG, &vehicle(json!({"year": year}))),
                "input_tax_credit"
            ),
            "500.00"
        );
        assert_eq!(
            amount(
                &run(BD, &marcus(json!({"year": year}))),
                "decreasing_adjustment"
            ),
            "50.00"
        );
    }
}

/// The catalogue's example of each command runs.
#[test]
fn the_catalogue_examples_run() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    for (command, kind) in [
        (GC, "gst_going_concern"),
        (SHG, "gst_second_hand_goods"),
        (BD, "gst_bad_debt"),
    ] {
        let mut facts = catalogue[kind]["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        facts["scope_confirmed"] = json!(true);
        assert!(api::evaluate(command, &facts).is_ok(), "{command}");
    }
}
