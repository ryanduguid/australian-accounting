// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The early stage investor tax offset (ITAA 1997 Subdivision 360-A), read on
//! 24 November 2026, against the ATO's examples (Thomas, Savannah, Georgia Co,
//! Jason and Tim), each threshold, each year boundary of Act No. 8 of 2020 and
//! Act No. 92 of 2020, each points combination that crosses 100 and each
//! refusal (docs/calculation-evidence.md#esic).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "esic";

/// A sophisticated individual's $100,000 in a company incorporated in
/// 2024-25 with 100 points, in 2025-26, with `overrides`.
fn input(overrides: Value) -> Value {
    let mut input = json!({
        "investor": "individual", "affiliate_of_company": false,
        "ess_interests": false, "more_than_30_percent": false,
        "amount_paid": "100000.00", "non_cash_benefit": "0.00",
        "disclosure_exemption": true, "incorporated_year": "2024-25",
        "abr_registered_year": "none", "expenses_three_prior_years": "0.00",
        "total_expenses_previous_year": "300000.00",
        "assessable_income_previous_year": "20000.00",
        "accelerating_commercialisation_grant": "0.00", "rd_clawback_income": "0.00",
        "listed": false, "foreign_company": false, "innovation_items": [1, 8],
        "principles_test": false, "other_offsets": "0.00",
        "carried_forward_offsets": "0.00", "year": "2025-26", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

/// The plain result, checked against `derive` byte for byte.
fn run(input: &Value) -> Value {
    let result = api::evaluate(COMMAND, input).unwrap();
    let doc = api::evaluate("derive", &json!({"command": COMMAND, "input": input})).unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string(), "{input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(COMMAND, input).unwrap_err().code
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

/// gross_offset, cap_room and tax_offset.
fn offset(result: &Value) -> [String; 3] {
    ["gross_offset", "cap_room", "tax_offset"].map(|key| amount(result, key))
}

/// Whether the test named `id` is met (`None` where it is not a test).
fn met(result: &Value, id: &str) -> Option<bool> {
    let row = result["tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["test"] == id)
        .unwrap_or_else(|| panic!("no test {id}"));
    row["met"].as_bool()
}

fn entitled(result: &Value) -> bool {
    assert_eq!(result["entitled"], result["modified_cgt_treatment"]);
    result["entitled"].as_bool().unwrap()
}

/// Qualifying as an ESIC, Thomas: $100,000 on 1 June 2017, $20,000.
#[test]
fn thomas() {
    let result = run(&input(
        json!({"year": "2016-17", "incorporated_year": "2015-16",
        "abr_registered_year": "2015-16"}),
    ));
    assert_eq!(offset(&result), ["20000.00", "200000.00", "20000.00"]);
    assert!(entitled(&result));
    assert_eq!(result["esic"], true);
    assert_eq!(met(&result, "not_foreign_company"), None);
}

/// Calculating the offset, Savannah: $4 million in 2016-17, $800,000 capped
/// at $200,000; Georgia Co, her affiliate, $2 million in the same year.
#[test]
fn savannah_and_georgia_co() {
    let base = json!({"year": "2016-17", "incorporated_year": "2016-17",
        "abr_registered_year": "none"});
    let mut savannah = input(base.clone());
    savannah["amount_paid"] = json!("4000000.00");
    let result = run(&savannah);
    assert_eq!(offset(&result), ["800000.00", "200000.00", "200000.00"]);
    let mut georgia = input(base);
    georgia["investor"] = json!("company");
    georgia["amount_paid"] = json!("2000000.00");
    georgia["other_offsets"] = json!("200000.00");
    let result = run(&georgia);
    assert_eq!(offset(&result), ["400000.00", "0.00", "0.00"]);
    assert!(
        entitled(&result),
        "s 360-50 applies whatever the cap leaves"
    );
}

/// Calculating the offset, Jason: $50,000 carried forward from 2016-17 and
/// $950,000 in 2017-18: $190,000, of which $150,000.
#[test]
fn jason_is_the_catalogue_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut call = catalogue["esic"]["example"]["facts"].clone();
    call.as_object_mut().unwrap().remove("kind");
    call["scope_confirmed"] = json!(true);
    let result = run(&call);
    assert_eq!(offset(&result), ["190000.00", "150000.00", "150000.00"]);
    assert_eq!(result["period"], "2017-18");
}

/// Limits without the sophisticated investor test, Tim: $50,000 is $10,000;
/// a further $10,000 to the same company loses the lot; a further $10,000
/// elsewhere is where s 360-20 and the ATO differ.
#[test]
fn tim() {
    let tim = |paid: &str, elsewhere: &str| {
        input(json!({"year": "2016-17", "incorporated_year": "2016-17",
            "amount_paid": paid, "disclosure_exemption": false,
            "other_companies_paid": elsewhere, "other_companies_exempt": false}))
    };
    let result = run(&tim("50000.00", "0.00"));
    assert_eq!(offset(&result), ["10000.00", "200000.00", "10000.00"]);
    assert_eq!(met(&result, "retail_limit"), Some(true));
    let result = run(&tim("60000.00", "0.00"));
    assert_eq!(offset(&result), ["0.00", "200000.00", "0.00"]);
    assert!(!entitled(&result));
    assert_eq!(result["esic"], true);
    assert_eq!(
        code(&tim("50000.00", "10000.00")),
        "esic.retail_limit_unsettled"
    );
    // A further $10,000 to another company that alone takes the total over:
    // both readings agree.
    assert!(!entitled(&run(&tim("50000.01", "10000.00"))));
}

#[test]
fn the_retail_limit_at_and_over_50000() {
    let call = |paid: &str, non_cash: &str| {
        input(json!({"amount_paid": paid, "non_cash_benefit": non_cash,
            "disclosure_exemption": false, "other_companies_paid": "0.00",
            "other_companies_exempt": false}))
    };
    assert_eq!(
        met(&run(&call("50000.00", "0.00")), "retail_limit"),
        Some(true)
    );
    assert_eq!(
        met(&run(&call("50000.01", "0.00")), "retail_limit"),
        Some(false)
    );
    assert_eq!(
        met(&run(&call("30000.00", "20000.00")), "retail_limit"),
        Some(true)
    );
    assert_eq!(
        code(&call("30000.00", "20000.01")),
        "esic.retail_limit_unsettled"
    );
    // Another test failing leaves the difference undecided rather than refused.
    let mut failing = call("30000.00", "20000.01");
    failing["listed"] = json!(true);
    let result = run(&failing);
    assert_eq!(met(&result, "retail_limit"), None);
    assert!(!entitled(&result));
    // With an exemption the limit is not a test, and other_companies_paid is not taken.
    let result = run(&input(json!({"amount_paid": "900000.00"})));
    assert_eq!(met(&result, "retail_limit"), None);
    let mut extra = input(json!({}));
    extra["other_companies_paid"] = json!("0.00");
    assert_eq!(code(&extra), "input.field_unknown");
    let mut missing = input(json!({"disclosure_exemption": false}));
    missing
        .as_object_mut()
        .unwrap()
        .remove("other_companies_paid");
    assert_eq!(code(&missing), "input.missing_field");
}

#[test]
fn non_cash_benefits_from_2018_19() {
    let result = run(&input(
        json!({"year": "2018-19", "incorporated_year": "2017-18",
        "amount_paid": "10000.00", "non_cash_benefit": "5000.00"}),
    ));
    assert_eq!(amount(&result, "offset_base"), "15000.00");
    assert_eq!(offset(&result)[0], "3000.00");
    let earlier = input(json!({"year": "2017-18", "incorporated_year": "2017-18",
        "non_cash_benefit": "0.01"}));
    assert_eq!(code(&earlier), "esic.non_cash_before_2018_19");
}

/// s 360-25(2): the room left by affiliates and carried forward offsets.
#[test]
fn the_cap_with_affiliates_and_carry_forward() {
    let call = |paid: &str, other: &str, carried: &str| {
        run(&input(json!({"amount_paid": paid, "other_offsets": other,
            "carried_forward_offsets": carried})))
    };
    assert_eq!(
        offset(&call("1000000.00", "0.00", "0.00")),
        ["200000.00", "200000.00", "200000.00"]
    );
    assert_eq!(offset(&call("1000000.05", "0.00", "0.00"))[2], "200000.00");
    assert_eq!(
        offset(&call("500000.00", "60000.00", "40000.00")),
        ["100000.00", "100000.00", "100000.00"]
    );
    assert_eq!(
        offset(&call("500000.00", "60000.00", "40000.01")),
        ["100000.00", "99999.99", "99999.99"]
    );
    assert_eq!(
        offset(&call("500000.00", "150000.00", "100000.00")),
        ["100000.00", "0.00", "0.00"]
    );
    // 20% of one cent is worked exactly and shown half up.
    assert_eq!(offset(&call("0.03", "0.00", "0.00"))[2], "0.01");
}

/// s 360-40(1)(a): the 3-year, 6-year and ABR routes and their boundaries.
#[test]
fn the_incorporation_routes() {
    let call = |inc: &str, abr: &str, spent: &str| {
        input(json!({"incorporated_year": inc, "abr_registered_year": abr,
            "expenses_three_prior_years": spent}))
    };
    assert_eq!(
        met(&run(&call("2023-24", "none", "0.00")), "incorporation"),
        Some(true)
    );
    assert_eq!(
        met(&run(&call("none", "2023-24", "0.00")), "incorporation"),
        Some(true)
    );
    assert_eq!(
        met(
            &run(&call("2022-23", "none", "1000000.00")),
            "incorporation"
        ),
        Some(true)
    );
    assert_eq!(
        met(
            &run(&call("2022-23", "none", "1000000.01")),
            "incorporation"
        ),
        Some(false)
    );
    assert_eq!(
        met(&run(&call("2020-21", "none", "0.00")), "incorporation"),
        Some(true)
    );
    let old = run(&call("2019-20", "2019-20", "0.00"));
    assert_eq!(met(&old, "incorporation"), Some(false));
    assert_eq!(old["esic"], false);
    assert_eq!(offset(&old)[2], "0.00");
    assert_eq!(
        met(
            &run(&call("2010-11", "2023-24", "5000000.00")),
            "incorporation"
        ),
        Some(true)
    );
}

/// s 360-40(1)(a): the years given, and the 6-year route before 2018-19.
#[test]
fn the_incorporation_refusals() {
    let call = |inc: &str, abr: &str, spent: &str| {
        input(json!({"incorporated_year": inc, "abr_registered_year": abr,
            "expenses_three_prior_years": spent}))
    };
    // Neither incorporated in Australia nor registered: no route is met.
    let result = run(&input(
        json!({"year": "2017-18", "incorporated_year": "none",
        "abr_registered_year": "none"}),
    ));
    assert_eq!(met(&result, "incorporation"), Some(false));
    assert_eq!(result["esic"], false);
    assert!(!entitled(&result));
    assert_eq!(offset(&result), ["0.00", "200000.00", "0.00"]);
    assert_eq!(
        code(&call("2026-27", "none", "0.00")),
        "esic.incorporation_after_year"
    );
    assert_eq!(code(&call("2024-26", "none", "0.00")), "esic.year_invalid");
    let mut not_text = call("none", "none", "0.00");
    not_text["incorporated_year"] = json!(2024);
    assert_eq!(code(&not_text), "esic.year_invalid");
    // Before 2018-19 the 6-year route is refused where it decides.
    let earlier = |inc: &str| {
        input(json!({"year": "2017-18", "incorporated_year": inc, "abr_registered_year": "none"}))
    };
    assert_eq!(
        code(&earlier("2014-15")),
        "esic.six_year_route_before_2018_19"
    );
    assert_eq!(met(&run(&earlier("2015-16")), "incorporation"), Some(true));
    assert_eq!(met(&run(&earlier("2011-12")), "incorporation"), Some(false));
}

/// s 360-40(1)(b) to (d) and (f) at their limits.
#[test]
fn the_company_tests() {
    let check = |overrides: Value, id: &str| met(&run(&input(overrides)), id);
    assert_eq!(
        check(
            json!({"total_expenses_previous_year": "1000000.00"}),
            "total_expenses"
        ),
        Some(true)
    );
    assert_eq!(
        check(
            json!({"total_expenses_previous_year": "1000000.01"}),
            "total_expenses"
        ),
        Some(false)
    );
    assert_eq!(
        check(
            json!({"assessable_income_previous_year": "200000.00"}),
            "assessable_income"
        ),
        Some(true)
    );
    assert_eq!(
        check(
            json!({"assessable_income_previous_year": "200000.01"}),
            "assessable_income"
        ),
        Some(false)
    );
    assert_eq!(check(json!({"listed": true}), "not_listed"), Some(false));
    assert_eq!(
        check(json!({"foreign_company": true}), "not_foreign_company"),
        Some(false)
    );
    assert_eq!(
        check(
            json!({"year": "2017-18", "incorporated_year": "2017-18", "foreign_company": true}),
            "not_foreign_company"
        ),
        None
    );
}

/// s 360-40(2): the grant always, the clawback amount from 2022-23.
#[test]
fn the_disregarded_income() {
    let call = |year: &str, inc: &str, clawback: &str| {
        input(json!({"year": year, "incorporated_year": inc,
            "assessable_income_previous_year": "450000.00",
            "accelerating_commercialisation_grant": "200000.00",
            "rd_clawback_income": clawback}))
    };
    let result = run(&call("2022-23", "2021-22", "50000.00"));
    assert_eq!(amount(&result, "counted_assessable_income"), "200000.00");
    assert_eq!(met(&result, "assessable_income"), Some(true));
    let result = run(&call("2020-21", "2019-20", "50000.00"));
    assert_eq!(amount(&result, "counted_assessable_income"), "250000.00");
    assert_eq!(met(&result, "assessable_income"), Some(false));
    let result = run(&call("2021-22", "2020-21", "0.00"));
    assert_eq!(amount(&result, "counted_assessable_income"), "250000.00");
    assert_eq!(
        code(&call("2021-22", "2020-21", "0.01")),
        "esic.clawback_year_unsettled"
    );
    assert_eq!(
        code(&call("2025-26", "2024-25", "250000.01")),
        "esic.amount_above_income"
    );
}

/// s 360-45(1): combinations at and either side of 100 points.
#[test]
fn the_100_point_test() {
    let points = |items: Value, principles: bool| {
        let result = run(&input(
            json!({"innovation_items": items, "principles_test": principles}),
        ));
        (
            result["innovation_points"].as_i64().unwrap(),
            met(&result, "innovation").unwrap(),
        )
    };
    assert_eq!(points(json!([1, 8]), false), (100, true));
    assert_eq!(points(json!([2, 7]), false), (100, true));
    assert_eq!(points(json!([3, 4]), false), (100, true));
    assert_eq!(points(json!([5, 6]), false), (100, true));
    assert_eq!(points(json!([1, 7]), false), (100, true));
    assert_eq!(points(json!([6, 7, 8]), false), (75, false));
    assert_eq!(points(json!([6, 7, 8]), true), (75, true));
    assert_eq!(points(json!([3, 7, 8]), false), (100, true));
    assert_eq!(points(json!([3, 8]), false), (75, false));
    assert_eq!(points(json!([]), true), (0, true));
    assert_eq!(points(json!([1, 2, 4, 5, 6, 7, 8]), false), (325, true));
    let refused = |items: Value| code(&input(json!({"innovation_items": items})));
    assert_eq!(refused(json!([1, 3])), "esic.innovation_items_conflict");
    for bad in [
        json!([0]),
        json!([9]),
        json!([1, 1]),
        json!(["1"]),
        json!(1.5),
        json!("1"),
        json!([2.0]),
    ] {
        assert_eq!(refused(bad), "esic.innovation_items_invalid");
    }
}

/// s 360-15(1)(a), (d) to (f).
#[test]
fn the_investor_tests() {
    let call = |overrides: Value| run(&input(overrides));
    assert!(entitled(&call(json!({"investor": "company"}))));
    let held = call(json!({"investor": "widely_held_company"}));
    assert_eq!(
        (met(&held, "investor"), entitled(&held)),
        (Some(false), false)
    );
    assert_eq!(held["esic"], true);
    let fund = call(json!({"investor": "esvclp"}));
    assert_eq!(
        (met(&fund, "investor"), entitled(&fund)),
        (Some(false), false)
    );
    for (key, id) in [
        ("affiliate_of_company", "not_affiliate"),
        ("ess_interests", "not_ess_interests"),
        ("more_than_30_percent", "not_more_than_30_percent"),
    ] {
        let result = call(json!({key: true}));
        assert_eq!(met(&result, id), Some(false), "{key}");
        assert_eq!(offset(&result)[2], "0.00");
    }
    let refused = |investor: &str, year: &str, inc: &str| {
        code(&input(
            json!({"investor": investor, "year": year, "incorporated_year": inc}),
        ))
    };
    assert_eq!(
        refused("trust", "2025-26", "2024-25"),
        "esic.trust_or_partnership_unsupported"
    );
    assert_eq!(
        refused("partnership", "2025-26", "2024-25"),
        "esic.trust_or_partnership_unsupported"
    );
    assert_eq!(
        refused("esvclp", "2017-18", "2017-18"),
        "esic.trust_or_partnership_unsupported"
    );
    assert_eq!(
        refused("super_fund", "2025-26", "2024-25"),
        "esic.investor_invalid"
    );
}

#[test]
fn each_supported_year_and_its_boundaries() {
    for (year, inc) in [
        ("2016-17", "2016-17"),
        ("2017-18", "2016-17"),
        ("2018-19", "2017-18"),
        ("2019-20", "2018-19"),
        ("2020-21", "2019-20"),
        ("2021-22", "2020-21"),
        ("2022-23", "2021-22"),
        ("2023-24", "2022-23"),
        ("2024-25", "2023-24"),
        ("2025-26", "2024-25"),
        ("2026-27", "2025-26"),
    ] {
        let result = run(&input(json!({"year": year, "incorporated_year": inc})));
        assert_eq!(offset(&result)[2], "20000.00", "{year}");
    }
    for year in ["2015-16", "2027-28", "2025"] {
        assert_eq!(
            code(&input(json!({"year": year}))),
            "period.unsupported",
            "{year}"
        );
    }
    assert_eq!(
        code(&input(json!({"scope_confirmed": false}))),
        "scope.not_confirmed"
    );
}

#[test]
fn each_input_is_checked() {
    assert_eq!(
        code(&input(json!({"amount_paid": "-1.00"}))),
        "input.amount_invalid"
    );
    assert_eq!(
        code(&input(json!({"other_offsets": "0.001"}))),
        "input.amount_invalid"
    );
    assert_eq!(code(&input(json!({"listed": "no"}))), "input.flag_invalid");
    assert_eq!(
        code(&input(json!({"principles_test": 1}))),
        "input.flag_invalid"
    );
    assert_eq!(
        code(&input(json!({"disclosure_exemption": null}))),
        "input.flag_invalid"
    );
    assert_eq!(
        code(&input(json!({"unknown": true}))),
        "input.field_unknown"
    );
    let mut missing = input(json!({}));
    missing.as_object_mut().unwrap().remove("innovation_items");
    assert_eq!(code(&missing), "input.missing_field");
}

/// Money paid and money receivable: before 2018-19 the total amount paid
/// ($40,000 paid of $60,000 gives $8,000); from 2018-19 money received or
/// entitled to be received ($12,000). Where counting the receivable as paid
/// decides the $50,000 limit, the result is refused.
#[test]
fn money_paid_and_receivable() {
    let call = |year: &str, inc: &str| {
        input(json!({"year": year, "incorporated_year": inc,
            "amount_paid": "40000.00", "amount_receivable": "20000.00"}))
    };
    let result = run(&call("2017-18", "2016-17"));
    assert_eq!(amount(&result, "offset_base"), "40000.00");
    assert_eq!(offset(&result)[2], "8000.00");
    let result = run(&call("2018-19", "2017-18"));
    assert_eq!(amount(&result, "offset_base"), "60000.00");
    assert_eq!(offset(&result)[2], "12000.00");
    let mut retail = call("2025-26", "2024-25");
    retail["disclosure_exemption"] = json!(false);
    retail["other_companies_paid"] = json!("0.00");
    retail["other_companies_exempt"] = json!(false);
    assert_eq!(code(&retail), "esic.retail_limit_unsettled");
    retail["amount_receivable"] = json!("10000.00");
    assert_eq!(offset(&run(&retail))[2], "10000.00");
    retail["amount_receivable"] = json!("-1.00");
    assert_eq!(code(&retail), "input.amount_invalid");
}

/// The ATO lifts the $50,000 limit for an exempt offer by another ESIC;
/// s 360-20(1)(a) looks at this company's offers only.
#[test]
fn an_exempt_offer_by_another_company() {
    let call = |paid: &str, exempt: bool| {
        input(json!({"amount_paid": paid, "disclosure_exemption": false,
            "other_companies_paid": "10000.00", "other_companies_exempt": exempt}))
    };
    assert_eq!(code(&call("60000.00", true)), "esic.retail_limit_unsettled");
    assert!(!entitled(&run(&call("60000.00", false))));
    let result = run(&call("40000.00", true));
    assert_eq!(met(&result, "retail_limit"), Some(true));
    assert_eq!(offset(&result)[2], "8000.00");
    let mut partial = call("40000.00", true);
    partial
        .as_object_mut()
        .unwrap()
        .remove("other_companies_exempt");
    assert_eq!(code(&partial), "input.missing_field");
    let mut extra = input(json!({}));
    extra["other_companies_exempt"] = json!(false);
    assert_eq!(code(&extra), "input.field_unknown");
    let mut bad = call("40000.00", true);
    bad["other_companies_exempt"] = json!("yes");
    assert_eq!(code(&bad), "input.flag_invalid");
}

/// The 2018-19 switch for a foreign company and an ESVCLP investor.
#[test]
fn foreign_company_and_esvclp_either_side_of_2018_19() {
    let call = |year: &str, inc: &str, extra: Value| {
        let mut given = input(json!({"year": year, "incorporated_year": inc}));
        for (k, v) in extra.as_object().unwrap() {
            given[k] = v.clone();
        }
        given
    };
    let foreign = json!({"foreign_company": true});
    let before = run(&call("2017-18", "2016-17", foreign.clone()));
    assert_eq!(met(&before, "not_foreign_company"), None);
    assert!(entitled(&before));
    let after = run(&call("2018-19", "2017-18", foreign));
    assert_eq!(met(&after, "not_foreign_company"), Some(false));
    assert!(!entitled(&after));
    let esvclp = json!({"investor": "esvclp"});
    assert_eq!(
        code(&call("2017-18", "2016-17", esvclp.clone())),
        "esic.trust_or_partnership_unsupported"
    );
    let after = run(&call("2018-19", "2017-18", esvclp));
    assert_eq!(met(&after, "investor"), Some(false));
    assert_eq!(after["esic"], true);
    assert!(!entitled(&after));
}
