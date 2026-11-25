// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Thin capitalisation for general class investors (ITAA 1997 Subdivision
//! 820-AA), read on 26 November 2026. T1 is the ATO's fixed ratio test example
//! (corporate tax entity) scaled by 100,000; the rest are made-up figures
//! worked by hand (docs/calculation-evidence.md#thin-capitalisation).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const CMD: &str = "thin-capitalisation";

fn run(input: &Value) -> Value {
    api::evaluate(CMD, input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded(CMD, input).unwrap_err().code
}

fn dd(label: &str, amount: &str) -> Value {
    json!({"label": label, "amount": amount})
}

fn base(test: &str, deductions: Value) -> Value {
    json!({"test": test, "entity_class": "general_class_investor", "plantation_forestry_entity": false,
           "special_purpose_entity": false, "outward_investor_only": false,
           "associate_inclusive_debt_deductions": "50000000.00", "debt_deductions": deductions,
           "year": "2025-26", "scope_confirmed": true})
}

/// The ATO's example: taxable income nil after $100 of earlier losses, $50
/// interest deductions, $10 interest income, $20 depreciation and a $40 excess
/// tax EBITDA amount, all times 100,000.
fn t1() -> Value {
    let mut i = base("fixed_ratio", json!([dd("Interest", "5000000.00")]));
    i["associate_inclusive_debt_deductions"] = json!("5000000.00");
    i["interest_income"] = json!("1000000.00");
    i["taxable_income"] = json!("0.00");
    i["division_40_43_deductions"] = json!("2000000.00");
    i["excess_tax_ebitda_amount"] = json!("4000000.00");
    i
}

fn amount(r: &Value, key: &str) -> String {
    r["amounts"][key].as_str().unwrap().to_string()
}

fn column(r: &Value, list: &str, key: &str) -> Vec<String> {
    r[list]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x[key].as_str().unwrap().to_string())
        .collect()
}

fn group(deductions: Value) -> Value {
    let mut i = base("group_ratio", deductions);
    i["interest_income"] = json!("500000.00");
    i["taxable_income"] = json!("4000000.00");
    i["division_40_43_deductions"] = json!("1000000.00");
    i["gr_group_net_third_party_interest_expense"] = json!("30000000.00");
    i["gr_group_net_profit"] = json!("50000000.00");
    i["gr_group_adjusted_net_third_party_interest_expense"] = json!("30000000.00");
    i["gr_group_depreciation_and_amortisation"] = json!("20000000.00");
    i["gr_group_negative_member_ebitda"] = json!(["-5000000.00"]);
    i
}

#[test]
fn t1_the_ato_fixed_ratio_example() {
    let r = run(&t1());
    assert_eq!(amount(&r, "debt_deductions"), "5000000.00");
    assert_eq!(amount(&r, "net_debt_deductions"), "4000000.00");
    assert_eq!(amount(&r, "tax_ebitda"), "10000000.00");
    assert_eq!(amount(&r, "fixed_ratio_earnings_limit"), "3000000.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "1000000.00");
    assert_eq!(amount(&r, "allowed_debt_deductions"), "4000000.00");
    assert_eq!(amount(&r, "special_deduction"), "0.00");
    assert_eq!(r["exemption"], Value::Null);
    assert_eq!(r["test"], "fixed_ratio");
    assert_eq!(column(&r, "debt_deductions", "disallowed"), ["1000000.00"]);
    assert_eq!(r["frt_disallowed_amounts"], json!([]));
}

#[test]
fn the_unscaled_example_is_under_the_de_minimis_threshold() {
    let mut i = t1();
    i["debt_deductions"] = json!([dd("Interest", "50.00")]);
    i["associate_inclusive_debt_deductions"] = json!("50.00");
    let r = run(&i);
    assert_eq!(amount(&r, "total_disallowed_amount"), "0.00");
    assert_eq!(amount(&r, "allowed_debt_deductions"), "50.00");
    assert!(r["exemption"].as_str().unwrap().starts_with("s 820-35"));
    assert!(r["amounts"].get("tax_ebitda").is_none());
}

#[test]
fn the_de_minimis_threshold_a_cent_either_side() {
    let mut i = base("fixed_ratio", json!([dd("Interest", "2000000.00")]));
    i["associate_inclusive_debt_deductions"] = json!("2000000.00");
    assert!(
        run(&i)["exemption"]
            .as_str()
            .unwrap()
            .starts_with("s 820-35")
    );
    i["associate_inclusive_debt_deductions"] = json!("2000000.01");
    assert_eq!(code(&i), "thin_capitalisation.input_missing");
    i["interest_income"] = json!("0.00");
    i["taxable_income"] = json!("1000000.00");
    i["division_40_43_deductions"] = json!("0.00");
    let r = run(&i);
    // (1,000,000 + 2,000,000) x 30% = 900,000; 2,000,000 - 900,000.
    assert_eq!(amount(&r, "total_disallowed_amount"), "1100000.00");
}

#[test]
fn t2_earlier_disallowed_amounts_are_deducted_earliest_first() {
    let mut i = base("fixed_ratio", json!([dd("Loan", "3000000.00")]));
    i["year"] = json!("2026-27");
    i["interest_income"] = json!("0.00");
    i["taxable_income"] = json!("10000000.00");
    i["division_40_43_deductions"] = json!("2000000.00");
    i["frt_disallowed_amounts"] = json!([
        {"year": "2025-26", "amount": "300000.00", "available": false},
        {"year": "2024-25", "amount": "800000.00", "available": true},
        {"year": "2023-24", "amount": "1000000.00", "available": true}
    ]);
    let r = run(&i);
    // Tax EBITDA 15,000,000, limit 4,500,000, excess 1,500,000.
    assert_eq!(amount(&r, "fixed_ratio_earnings_limit"), "4500000.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "0.00");
    assert_eq!(amount(&r, "special_deduction"), "1500000.00");
    assert_eq!(
        column(&r, "frt_disallowed_amounts", "year"),
        ["2023-24", "2024-25", "2025-26"]
    );
    assert_eq!(
        column(&r, "frt_disallowed_amounts", "applied"),
        ["1000000.00", "500000.00", "0.00"]
    );
    assert_eq!(
        column(&r, "frt_disallowed_amounts", "remaining"),
        ["0.00", "300000.00", "300000.00"]
    );
}

#[test]
fn negative_net_debt_deductions_widen_the_excess() {
    // Net debt deductions -1,000,000; tax EBITDA 4,000,000, limit 1,200,000;
    // the excess over net debt deductions is 2,200,000 (s 820-56(1)(b)).
    let mut i = base("fixed_ratio", json!([dd("Loan", "1000000.00")]));
    i["year"] = json!("2024-25");
    i["interest_income"] = json!("2000000.00");
    i["taxable_income"] = json!("5000000.00");
    i["division_40_43_deductions"] = json!("0.00");
    i["frt_disallowed_amounts"] =
        json!([{"year": "2023-24", "amount": "2500000.00", "available": true}]);
    let r = run(&i);
    assert_eq!(amount(&r, "net_debt_deductions"), "-1000000.00");
    assert_eq!(amount(&r, "tax_ebitda"), "4000000.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "0.00");
    assert_eq!(amount(&r, "special_deduction"), "2200000.00");
}

#[test]
fn a_negative_tax_ebitda_is_nil_and_every_adjustment_line_counts() {
    let mut i = base("fixed_ratio", json!([dd("Loan", "3000000.00")]));
    i["interest_income"] = json!("0.00");
    i["taxable_income"] = json!("-10000000.00");
    i["division_40_43_deductions"] = json!("1000000.00");
    let r = run(&i);
    assert_eq!(amount(&r, "tax_ebitda"), "0.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "3000000.00");
    assert_eq!(amount(&r, "allowed_debt_deductions"), "0.00");
    // 10,000,000 - 100,000 - 200,000 - 300,000 + 3,000,000 + 400,000
    // + 500,000 + 600,000 + 700,000 = 14,600,000.
    for (key, value) in [
        ("taxable_income", "10000000.00"),
        ("franking_credits", "100000.00"),
        ("associate_dividends", "200000.00"),
        ("rd_notional_deductions", "300000.00"),
        ("division_40_43_deductions", "400000.00"),
        ("forestry_deductions", "500000.00"),
        ("section_70_120_deductions", "600000.00"),
        ("excess_tax_ebitda_amount", "700000.00"),
    ] {
        i[key] = json!(value);
    }
    let r = run(&i);
    assert_eq!(amount(&r, "tax_ebitda"), "14600000.00");
    assert_eq!(amount(&r, "fixed_ratio_earnings_limit"), "4380000.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "0.00");
}

#[test]
fn t3_the_group_ratio_test() {
    let r = run(&group(json!([
        dd("Loan A", "4000000.00"),
        dd("Loan B", "2000000.00")
    ])));
    // Tax EBITDA 4,000,000 + 5,500,000 + 1,000,000; GR group EBITDA
    // 100,000,000 with the 5,000,000 negative member disregarded; ratio 30/105.
    assert_eq!(amount(&r, "tax_ebitda"), "10500000.00");
    assert_eq!(amount(&r, "gr_group_ebitda"), "105000000.00");
    assert_eq!(amount(&r, "group_ratio_earnings_limit"), "3000000.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "2500000.00");
    assert_eq!(amount(&r, "allowed_debt_deductions"), "3500000.00");
    assert!(
        r["rates"]["group_ratio"]
            .as_str()
            .unwrap()
            .starts_with("0.2857142857")
    );
    assert_eq!(
        column(&r, "debt_deductions", "disallowed"),
        ["1666666.67", "833333.33"]
    );
    assert_eq!(
        column(&r, "debt_deductions", "allowed"),
        ["2333333.33", "1166666.67"]
    );
    assert!(r.get("frt_disallowed_amounts").is_none());
}

#[test]
fn the_group_ratio_tests_refusals() {
    let mut i = group(json!([dd("Loan", "6000000.00")]));
    i["gr_group_net_profit"] = json!("-55000000.00");
    i["gr_group_negative_member_ebitda"] = json!([]);
    assert_eq!(code(&i), "thin_capitalisation.gr_group_ebitda_not_positive");
    let mut i = group(json!([dd("Loan", "6000000.00")]));
    i["gr_group_net_third_party_interest_expense"] = json!("-0.01");
    assert_eq!(code(&i), "thin_capitalisation.group_ratio_negative");
    for bad in [json!(["5.00"]), json!("-5.00"), json!([5])] {
        let mut i = group(json!([dd("Loan", "6000000.00")]));
        i["gr_group_negative_member_ebitda"] = bad;
        assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    }
    let mut i = group(json!([dd("Loan", "6000000.00")]));
    i.as_object_mut().unwrap().remove("gr_group_net_profit");
    assert_eq!(code(&i), "thin_capitalisation.input_missing");
    for (key, value) in [
        ("excess_tax_ebitda_amount", json!("1.00")),
        ("frt_disallowed_amounts", json!([])),
    ] {
        let mut i = group(json!([dd("Loan", "6000000.00")]));
        i[key] = value;
        assert_eq!(code(&i), "thin_capitalisation.input_not_used", "{key}");
    }
    // Without the negative member list the GR group EBITDA is 100,000,000.
    let mut i = group(json!([dd("Loan", "6000000.00")]));
    i.as_object_mut()
        .unwrap()
        .remove("gr_group_negative_member_ebitda");
    assert_eq!(amount(&run(&i), "gr_group_ebitda"), "100000000.00");
}

#[test]
fn t4_the_third_party_debt_test() {
    let i = base(
        "third_party_debt",
        json!([
            {"label": "Bank loan", "amount": "3000000.00", "third_party": true},
            {"label": "Parent loan", "amount": "1000000.00", "third_party": false},
            {"label": "Swap on the bank loan", "amount": "500000.00", "third_party": true}
        ]),
    );
    let r = run(&i);
    assert_eq!(amount(&r, "debt_deductions"), "4500000.00");
    assert_eq!(amount(&r, "third_party_earnings_limit"), "3500000.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "1000000.00");
    assert_eq!(
        column(&r, "debt_deductions", "disallowed"),
        ["666666.67", "222222.22", "111111.11"]
    );
    assert!(r["amounts"].get("net_debt_deductions").is_none());
    assert_eq!(r["debt_deductions"][1]["third_party"], false);
}

#[test]
fn the_third_party_debt_tests_inputs() {
    let i = base("third_party_debt", json!([dd("Bank loan", "3000000.00")]));
    assert_eq!(code(&i), "thin_capitalisation.input_missing");
    for key in [
        "taxable_income",
        "interest_income",
        "division_40_43_deductions",
        "gr_group_net_profit",
        "frt_disallowed_amounts",
    ] {
        let mut i = base(
            "third_party_debt",
            json!([{"label": "Bank loan", "amount": "1.00", "third_party": true}]),
        );
        i[key] = json!("1.00");
        assert_eq!(code(&i), "thin_capitalisation.input_not_used", "{key}");
    }
    let i = base(
        "fixed_ratio",
        json!([{"label": "Bank loan", "amount": "1.00", "third_party": true}]),
    );
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    let mut i = t1();
    i["gr_group_net_profit"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
}

#[test]
fn the_other_exemptions() {
    let mut i = t1();
    i["associate_inclusive_debt_deductions"] = json!("50000000.00");
    i["special_purpose_entity"] = json!(true);
    assert!(
        run(&i)["exemption"]
            .as_str()
            .unwrap()
            .starts_with("s 820-39")
    );
    let mut i = t1();
    i["associate_inclusive_debt_deductions"] = json!("50000000.00");
    i["entity_class"] = json!("not_covered");
    let r = run(&i);
    assert!(r["exemption"].as_str().unwrap().starts_with("s 820-46(2)"));
    assert_eq!(amount(&r, "total_disallowed_amount"), "0.00");
    // Section 820-37: exactly 90% is enough; a cent less is not.
    let mut i = t1();
    i["associate_inclusive_debt_deductions"] = json!("50000000.00");
    i["outward_investor_only"] = json!(true);
    i["average_australian_assets"] = json!("900.00");
    i["average_total_assets"] = json!("1000.00");
    let r = run(&i);
    assert!(r["exemption"].as_str().unwrap().starts_with("s 820-37"));
    assert_eq!(r["australian_assets_ratio"], "0.9");
    i["average_australian_assets"] = json!("899.99");
    let r = run(&i);
    assert_eq!(r["exemption"], Value::Null);
    assert_eq!(amount(&r, "total_disallowed_amount"), "1000000.00");
    i["average_total_assets"] = json!("899.98");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    i["average_total_assets"] = json!("0.00");
    i["average_australian_assets"] = json!("0.00");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    i.as_object_mut().unwrap().remove("average_total_assets");
    assert_eq!(code(&i), "thin_capitalisation.input_missing");
    let mut i = t1();
    i["average_total_assets"] = json!("1000.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
}

#[test]
fn carried_amounts_and_their_refusals() {
    let mut i = t1();
    i["associate_inclusive_debt_deductions"] = json!("100.00");
    i["debt_deductions"] = json!([dd("Interest", "100.00")]);
    i["frt_disallowed_amounts"] = json!([]);
    assert_eq!(code(&i), "thin_capitalisation.carry_forward_unsettled");
    for bad in [
        json!([{"year": "2022-23", "amount": "1.00", "available": true}]),
        json!([{"year": "2025-26", "amount": "1.00", "available": true}]),
        json!([{"year": "2024-26", "amount": "1.00", "available": true}]),
        json!([{"year": "24-25", "amount": "1.00", "available": true}]),
        json!([{"year": "2024-25", "amount": "-1.00", "available": true}]),
        json!([{"year": "2024-25", "amount": "1.00"}]),
        json!([{"year": "2024-25", "amount": "1.00", "available": true, "note": "x"}]),
        json!([
            {"year": "2024-25", "amount": "1.00", "available": true},
            {"year": "2024-25", "amount": "2.00", "available": true}
        ]),
        json!(vec![
            json!({"year": "2024-25", "amount": "1.00", "available": true});
            16
        ]),
        json!("2024-25"),
    ] {
        let mut i = t1();
        i["frt_disallowed_amounts"] = bad.clone();
        assert_eq!(
            code(&i),
            "thin_capitalisation.carry_forward_invalid",
            "{bad}"
        );
    }
    // In 2023-24, the first year, there is no earlier year to carry from.
    let mut i = t1();
    i["year"] = json!("2023-24");
    i["frt_disallowed_amounts"] = json!([{"year": "2023-24", "amount": "1.00", "available": true}]);
    assert_eq!(code(&i), "thin_capitalisation.carry_forward_invalid");
}

#[test]
fn entity_classes_and_scope() {
    for class in [
        "outward_investing_financial_entity",
        "inward_investing_financial_entity",
        "outward_investing_adi",
        "inward_investing_adi",
    ] {
        let mut i = t1();
        i["entity_class"] = json!(class);
        assert_eq!(
            code(&i),
            "thin_capitalisation.entity_class_not_built",
            "{class}"
        );
    }
    let mut i = t1();
    i["entity_class"] = json!("bank");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    let mut i = t1();
    i["test"] = json!("safe_harbour");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    let mut i = t1();
    i["plantation_forestry_entity"] = json!(true);
    assert_eq!(code(&i), "thin_capitalisation.plantation_forestry");
    let mut i = t1();
    i["special_purpose_entity"] = json!("no");
    assert_eq!(code(&i), "input.flag_invalid");
    let mut i = t1();
    i["associate_inclusive_debt_deductions"] = json!("4999999.99");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    for year in ["2023-24", "2026-27"] {
        let mut i = t1();
        i["year"] = json!(year);
        assert!(api::evaluate(CMD, &i).is_ok(), "{year}");
    }
    for year in ["2022-23", "2027-28"] {
        let mut i = t1();
        i["year"] = json!(year);
        assert_eq!(code(&i), "period.unsupported", "{year}");
    }
    let mut i = t1();
    i["scope_confirmed"] = json!(false);
    assert_eq!(code(&i), "scope.not_confirmed");
    let mut i = t1();
    i["not_an_input"] = json!(1);
    assert_eq!(code(&i), "input.field_unknown");
}

#[test]
fn missing_and_malformed_inputs() {
    for key in [
        "interest_income",
        "taxable_income",
        "division_40_43_deductions",
    ] {
        let mut i = t1();
        i.as_object_mut().unwrap().remove(key);
        assert_eq!(code(&i), "thin_capitalisation.input_missing", "{key}");
    }
    for bad in [
        json!([]),
        json!([{"label": "", "amount": "1.00"}]),
        json!([{"label": "A", "amount": "-1.00"}]),
        json!([{"label": "A", "amount": "1.00", "third_party": "yes"}]),
        json!([{"label": "A", "amount": "1.00", "lender": "B"}]),
        json!(["1.00"]),
        json!(vec![dd("A", "1.00"); 201]),
    ] {
        let mut i = t1();
        i["debt_deductions"] = bad.clone();
        assert_eq!(
            code(&i),
            "thin_capitalisation.debt_deductions_invalid",
            "{bad}"
        );
    }
    let mut i = t1();
    i["interest_income"] = json!("-1.00");
    assert_eq!(code(&i), "input.amount_invalid");
}
