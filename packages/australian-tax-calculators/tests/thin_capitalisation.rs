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
    i["interest_and_equivalent_income"] = json!("1000000.00");
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
    i["interest_and_equivalent_income"] = json!("500000.00");
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
    i["interest_and_equivalent_income"] = json!("0.00");
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
    i["interest_and_equivalent_income"] = json!("0.00");
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
    i["interest_and_equivalent_income"] = json!("2000000.00");
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
    i["interest_and_equivalent_income"] = json!("0.00");
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
        "interest_and_equivalent_income",
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
    // Subdivision 820-AA's tests are not those of the other classes.
    for class in [
        "outward_investing_financial_entity",
        "inward_investing_financial_entity",
        "outward_investing_adi",
        "inward_investing_adi",
    ] {
        let mut i = t1();
        i["entity_class"] = json!(class);
        assert_eq!(code(&i), "thin_capitalisation.fact_invalid", "{class}");
    }
    for test in ["maximum_allowable_debt", "minimum_capital_amount"] {
        let mut i = t1();
        i["test"] = json!(test);
        let e = api::evaluate_coded(CMD, &i).unwrap_err();
        assert_eq!(e.code, "thin_capitalisation.fact_invalid", "{test}");
        assert!(
            e.message
                .contains("fixed_ratio, group_ratio or third_party_debt."),
            "{test}"
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
        "interest_and_equivalent_income",
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
    i["interest_and_equivalent_income"] = json!("-1.00");
    assert_eq!(code(&i), "input.amount_invalid");
}

// Subdivisions 820-B to 820-E, read on 27 November 2026. GLM Limited, TRR
// Limited, Southern Cross Bank and Global Bank are the Act's own examples
// (ss 820-100, 820-110, 820-111, 820-210, 820-310, 820-320 and 820-405); the
// adjusted average debt, equity capital and deductions around them are
// made-up figures worked by hand.

fn older(class: &str, test: &str, deductions: Value) -> Value {
    let mut i = base(test, deductions);
    i["entity_class"] = json!(class);
    i["whole_year"] = json!(true);
    i
}

fn drop(i: &mut Value, keys: &[&str]) {
    for k in keys {
        i.as_object_mut().unwrap().remove(*k);
    }
}

fn refusal(i: &Value) -> api::Refusal {
    api::evaluate_coded(CMD, i).unwrap_err()
}

/// GLM Limited (ss 820-100(2), (3) and 820-110), with $110m of average debt
/// capital and $8.8m of debt deductions.
fn glm() -> Value {
    let mut i = older(
        "outward_investing_financial_entity",
        "maximum_allowable_debt",
        json!([dd("Bonds", "8000000.00"), dd("Loan", "800000.00")]),
    );
    for (k, v) in [
        ("average_debt_capital", "110000000.00"),
        ("average_associate_entity_debt", "5000000.00"),
        ("average_controlled_foreign_entity_debt", "9000000.00"),
        ("average_borrowed_securities_amount", "0.00"),
        ("average_cost_free_debt_capital", "0.00"),
        ("average_assets", "160000000.00"),
        ("average_excluded_equity_interests", "5000000.00"),
        ("average_associate_entity_equity", "5000000.00"),
        ("average_controlled_foreign_entity_equity", "6000000.00"),
        ("average_non_debt_liabilities", "5000000.00"),
        ("average_zero_capital_amount", "4000000.00"),
        ("average_associate_entity_excess_amount", "0.00"),
        ("average_on_lent_amount", "35000000.00"),
        ("average_worldwide_debt", "120000000.00"),
        ("average_worldwide_equity", "40000000.00"),
    ] {
        i[k] = json!(v);
    }
    i["inward_investment_vehicle"] = json!(false);
    i
}

#[test]
fn glm_outward_financial_entity() {
    let r = run(&glm());
    assert_eq!(amount(&r, "adjusted_average_debt"), "96000000.00");
    assert_eq!(amount(&r, "total_debt_base"), "121000000.00");
    assert_eq!(amount(&r, "total_debt_amount"), "117437500.00");
    assert_eq!(amount(&r, "on_lent_base"), "95000000.00");
    assert_eq!(amount(&r, "adjusted_on_lent_amount"), "87000000.00");
    assert_eq!(amount(&r, "safe_harbour_debt_amount"), "87000000.00");
    assert_eq!(amount(&r, "worldwide_gearing_debt_amount"), "94750000.00");
    assert_eq!(amount(&r, "maximum_allowable_debt"), "94750000.00");
    assert_eq!(amount(&r, "excess_debt"), "1250000.00");
    // 8,800,000 x 1,250,000 / 110,000,000.
    assert_eq!(amount(&r, "total_disallowed_amount"), "100000.00");
    assert_eq!(
        column(&r, "debt_deductions", "disallowed"),
        ["90909.09", "9090.91"]
    );
    assert_eq!(r["exemption"], Value::Null);
    // Within the maximum allowable debt nothing is disallowed.
    let mut i = glm();
    i["average_debt_capital"] = json!("108750000.00");
    assert_eq!(amount(&run(&i), "total_disallowed_amount"), "0.00");
    // Worldwide equity of nil leaves the safe harbour debt amount alone
    // (s 820-90(1)(c)): excess 96m - 87m = 9m; 8.8m x 9 / 110 = 720,000.
    let mut i = glm();
    i["average_worldwide_equity"] = json!("0.00");
    let r = run(&i);
    assert_eq!(amount(&r, "maximum_allowable_debt"), "87000000.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "720000.00");
    assert!(r["amounts"].get("worldwide_gearing_debt_amount").is_none());
    // Adjusted average debt of nil or less exceeds nothing (s 820-85(4)).
    let mut i = glm();
    i["average_debt_capital"] = json!("1000000.00");
    assert_eq!(amount(&run(&i), "excess_debt"), "0.00");
}

#[test]
fn a_maximum_allowable_debt_not_worked_out_is_refused() {
    // Without the worldwide figures an excess over the safe harbour debt
    // amount is refused, never estimated (ss 820-90 and 820-190).
    let mut i = glm();
    drop(
        &mut i,
        &["average_worldwide_debt", "average_worldwide_equity"],
    );
    let e = refusal(&i);
    assert_eq!(e.code, "thin_capitalisation.amount_not_worked_out");
    assert!(e.message.contains("ss 820-90 and 820-190"));
    // Below the safe harbour debt amount the answer needs nothing more.
    i["average_debt_capital"] = json!("100000000.00");
    assert_eq!(amount(&run(&i), "total_disallowed_amount"), "0.00");
    // Half the worldwide figures are refused.
    let mut i = glm();
    drop(&mut i, &["average_worldwide_debt"]);
    assert_eq!(code(&i), "thin_capitalisation.input_missing");
}

/// TRR Limited (s 820-111): an outward investor that is also an inward
/// investment vehicle, with a total debt base of $100m (140m less 5m, 5m,
/// 5m, 9m, 6m, 5m and a zero-capital amount of 5m).
fn trr() -> Value {
    let mut i = glm();
    i["inward_investment_vehicle"] = json!(true);
    drop(
        &mut i,
        &["average_worldwide_debt", "average_worldwide_equity"],
    );
    i["average_assets"] = json!("140000000.00");
    i["average_zero_capital_amount"] = json!("5000000.00");
    i["statement_worldwide_debt"] = json!("90000000.00");
    i["statement_worldwide_equity"] = json!("30000000.00");
    i["statement_worldwide_assets"] = json!("120000000.00");
    i["statement_average_australian_assets"] = json!("60000000.00");
    i["audited_consolidated_statements"] = json!(true);
    i
}

#[test]
fn trr_outward_investor_also_an_inward_investment_vehicle() {
    let r = run(&trr());
    assert_eq!(amount(&r, "total_debt_base"), "100000000.00");
    assert_eq!(amount(&r, "worldwide_gearing_debt_amount"), "80000000.00");
    assert_eq!(amount(&r, "maximum_allowable_debt"), "80000000.00");
    // Above half Australian assets, without audited statements or with nil
    // statement equity or assets, the worldwide gearing debt amount is not
    // available (s 820-90(3)).
    for (k, v) in [
        ("statement_average_australian_assets", json!("60000000.01")),
        ("audited_consolidated_statements", json!(false)),
        ("statement_worldwide_equity", json!("-1.00")),
        ("statement_worldwide_assets", json!("0.00")),
    ] {
        let mut i = trr();
        i[k] = v;
        let r = run(&i);
        assert!(
            r["amounts"].get("worldwide_gearing_debt_amount").is_none(),
            "{k}"
        );
        assert_eq!(amount(&r, "maximum_allowable_debt"), "75000000.00", "{k}");
    }
    let mut i = trr();
    drop(&mut i, &["audited_consolidated_statements"]);
    assert_eq!(code(&i), "thin_capitalisation.input_missing");
    let mut i = trr();
    drop(
        &mut i,
        &[
            "statement_worldwide_debt",
            "statement_worldwide_equity",
            "statement_worldwide_assets",
            "statement_average_australian_assets",
        ],
    );
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    let mut i = trr();
    i["average_worldwide_debt"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
}

/// The inward investor of s 820-210's examples: Australian investments of
/// $120m; its statements are not audited, so s 820-190(2)(b) leaves it the
/// safe harbour debt amount.
fn investor() -> Value {
    let mut i = older(
        "inward_investing_financial_entity",
        "maximum_allowable_debt",
        json!([dd("Loan", "6000000.00")]),
    );
    for (k, v) in [
        ("average_debt_capital", "90000000.00"),
        ("average_associate_entity_debt", "5000000.00"),
        ("average_pe_associate_entity_debt", "10000000.00"),
        ("average_borrowed_securities_amount", "0.00"),
        ("average_cost_free_debt_capital", "0.00"),
        ("average_assets", "120000000.00"),
        ("average_excluded_equity_interests", "5000000.00"),
        ("average_associate_entity_equity", "2000000.00"),
        ("average_non_debt_liabilities", "3000000.00"),
        ("average_zero_capital_amount", "5000000.00"),
        ("average_associate_entity_excess_amount", "0.00"),
        ("average_on_lent_amount", "35000000.00"),
        ("statement_worldwide_debt", "90000000.00"),
        ("statement_worldwide_equity", "30000000.00"),
        ("statement_worldwide_assets", "200000000.00"),
        ("statement_average_australian_assets", "100000000.00"),
    ] {
        i[k] = json!(v);
    }
    i["audited_consolidated_statements"] = json!(false);
    i["inward_investment_vehicle"] = json!(false);
    i
}

#[test]
fn inward_investor_and_inward_investment_vehicle() {
    let r = run(&investor());
    assert_eq!(amount(&r, "total_debt_amount"), "98750000.00");
    assert_eq!(amount(&r, "adjusted_on_lent_amount"), "75000000.00");
    assert_eq!(amount(&r, "safe_harbour_debt_amount"), "75000000.00");
    // 90m less 10m of associate entity debt attributable to the Australian
    // permanent establishments = 80m; excess 5m; 6m x 5 / 90 = 333,333.33.
    assert_eq!(amount(&r, "adjusted_average_debt"), "80000000.00");
    assert_eq!(amount(&r, "total_disallowed_amount"), "333333.33");
    // Step 7 of s 820-210(3) takes a negative result as nil.
    let mut i = investor();
    i["average_associate_entity_debt"] = json!("90000000.00");
    assert_eq!(amount(&run(&i), "adjusted_on_lent_amount"), "0.00");
    // The associate entity excess amount is added after that floor (step 8).
    i["average_associate_entity_excess_amount"] = json!("1000000.00");
    assert_eq!(amount(&run(&i), "adjusted_on_lent_amount"), "1000000.00");
    // An inward investment vehicle reduces by all associate entity debt and
    // has no step 7 floor (s 820-200(3)); adjusted average debt of nil
    // exceeds nothing (s 820-185(4)).
    let mut i = investor();
    i["inward_investment_vehicle"] = json!(true);
    drop(&mut i, &["average_pe_associate_entity_debt"]);
    i["average_associate_entity_debt"] = json!("90000000.00");
    let r = run(&i);
    assert_eq!(amount(&r, "adjusted_average_debt"), "0.00");
    assert_eq!(amount(&r, "adjusted_on_lent_amount"), "-10000000.00");
    assert_eq!(amount(&r, "excess_debt"), "0.00");
    // Audited statements for s 820-217: 90 / 30 = 3; 3 / 4 x 100m + 5m.
    let mut i = investor();
    i["inward_investment_vehicle"] = json!(true);
    drop(&mut i, &["average_pe_associate_entity_debt"]);
    i["audited_consolidated_statements"] = json!(true);
    let r = run(&i);
    assert_eq!(amount(&r, "worldwide_gearing_debt_amount"), "80000000.00");
    assert_eq!(amount(&r, "maximum_allowable_debt"), "80000000.00");
    // And s 820-219 for the inward investor.
    let mut i = investor();
    i["audited_consolidated_statements"] = json!(true);
    assert_eq!(
        amount(&run(&i), "worldwide_gearing_debt_amount"),
        "80000000.00"
    );
    // Without the statement figures an excess is refused, and below the safe
    // harbour debt amount nothing more is needed.
    let mut i = investor();
    drop(
        &mut i,
        &[
            "statement_worldwide_debt",
            "statement_worldwide_equity",
            "statement_worldwide_assets",
            "statement_average_australian_assets",
            "audited_consolidated_statements",
        ],
    );
    assert_eq!(code(&i), "thin_capitalisation.amount_not_worked_out");
    i["average_debt_capital"] = json!("80000000.00");
    assert_eq!(amount(&run(&i), "total_disallowed_amount"), "0.00");
    // Inward entities have no controlled foreign entity figures or s 820-37.
    let mut i = investor();
    i["average_controlled_foreign_entity_debt"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    let mut i = investor();
    i["outward_investor_only"] = json!(true);
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    let mut i = investor();
    drop(&mut i, &["average_pe_associate_entity_debt"]);
    assert_eq!(code(&i), "thin_capitalisation.input_missing");
}

#[test]
fn financial_entity_edges() {
    // More excess than average debt disallows all of each deduction, no more.
    let mut i = glm();
    i["average_borrowed_securities_amount"] = json!("1000000000.00");
    let r = run(&i);
    assert_eq!(amount(&r, "total_disallowed_amount"), "8800000.00");
    assert_eq!(amount(&r, "allowed_debt_deductions"), "0.00");
    // Excess debt with nil average debt has no proportion to apply.
    let mut i = glm();
    for k in [
        "average_debt_capital",
        "average_associate_entity_debt",
        "average_controlled_foreign_entity_debt",
    ] {
        i[k] = json!("0.00");
    }
    i["average_borrowed_securities_amount"] = json!("200000000.00");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    // The third party debt choice (ss 820-85(2C) and 820-185(2C)).
    let mut i = older(
        "outward_investing_financial_entity",
        "third_party_debt",
        json!([{"label": "A", "amount": "3000000.00", "third_party": true},
               {"label": "B", "amount": "1000000.00", "third_party": false}]),
    );
    let r = run(&i);
    assert_eq!(amount(&r, "total_disallowed_amount"), "1000000.00");
    i["entity_class"] = json!("inward_investing_financial_entity");
    assert_eq!(amount(&run(&i), "total_disallowed_amount"), "1000000.00");
    i["average_assets"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    // Exemptions apply to every class (ss 820-35 and 820-37).
    let mut i = glm();
    i["associate_inclusive_debt_deductions"] = json!("2000000.00");
    i["debt_deductions"] = json!([dd("A", "2000000.00")]);
    assert!(
        run(&i)["exemption"]
            .as_str()
            .unwrap()
            .starts_with("s 820-35")
    );
    let mut i = glm();
    i["outward_investor_only"] = json!(true);
    i["average_australian_assets"] = json!("90.00");
    i["average_total_assets"] = json!("100.00");
    assert!(
        run(&i)["exemption"]
            .as_str()
            .unwrap()
            .starts_with("s 820-37")
    );
    // Part-year periods are refused with their provisions named.
    let mut i = glm();
    i["whole_year"] = json!(false);
    let e = refusal(&i);
    assert_eq!(e.code, "thin_capitalisation.part_year_not_built");
    assert!(
        e.message
            .contains("ss 820-120, 820-225, 820-330 or 820-420")
    );
    for key in [
        "whole_year",
        "inward_investment_vehicle",
        "average_on_lent_amount",
    ] {
        let mut i = glm();
        drop(&mut i, &[key]);
        assert_eq!(code(&i), "thin_capitalisation.input_missing", "{key}");
    }
    // The other Subdivisions' inputs are refused, either way round.
    let mut i = glm();
    i["taxable_income"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    let mut i = glm();
    i["average_risk_weighted_assets"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    let mut i = t1();
    i["average_assets"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    // Under every general class test, whatever the test (review S1).
    for test in ["group_ratio", "third_party_debt"] {
        let marked = json!([{"label": "A", "amount": "1.00", "third_party": true}]);
        let mut i = base(
            test,
            if test == "group_ratio" {
                json!([dd("A", "1.00")])
            } else {
                marked
            },
        );
        i["average_assets"] = json!("1.00");
        assert_eq!(code(&i), "thin_capitalisation.input_not_used", "{test}");
    }
    let mut i = glm();
    i["test"] = json!("minimum_capital_amount");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    let mut i = glm();
    i["debt_deductions"] = json!([{"label": "A", "amount": "1.00", "third_party": true}]);
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    let mut i = glm();
    i["plantation_forestry_entity"] = json!(true);
    assert_eq!(code(&i), "thin_capitalisation.plantation_forestry");
}

/// Southern Cross Bank (ss 820-310 and 820-320), with $10m of ADI equity
/// capital, $200m of average debt and $10m of debt deductions.
fn southern_cross() -> Value {
    let mut i = older(
        "outward_investing_adi",
        "minimum_capital_amount",
        json!([dd("Deposits", "10000000.00")]),
    );
    for (k, v) in [
        ("average_debt_capital", "200000000.00"),
        ("average_adi_equity_capital", "10000000.00"),
        ("average_controlled_foreign_entity_equity", "0.00"),
        ("average_risk_weighted_assets", "150000000.00"),
        ("average_tier_1_prudential_capital_deductions", "2000000.00"),
        ("worldwide_capital_risk_weighted_assets", "150000000.00"),
        (
            "worldwide_capital_tier_1_prudential_capital_deductions",
            "2000000.00",
        ),
        ("group_average_tier_1_capital", "14000000.00"),
        ("group_average_risk_weighted_assets", "160000000.00"),
        ("arm_length_capital_amount", "12000000.00"),
    ] {
        i[k] = json!(v);
    }
    i["foreign_controlled_australian_entity"] = json!(false);
    i
}

#[test]
fn southern_cross_outward_adi() {
    let r = run(&southern_cross());
    assert_eq!(amount(&r, "safe_harbour_capital_amount"), "11000000.00");
    assert_eq!(amount(&r, "worldwide_capital_amount"), "15125000.00");
    assert_eq!(amount(&r, "minimum_capital_amount"), "11000000.00");
    assert_eq!(amount(&r, "capital_shortfall"), "1000000.00");
    // 10m x 1m / 200m.
    assert_eq!(amount(&r, "total_disallowed_amount"), "50000.00");
    // A lower arm's length capital amount, as stated, sets the minimum.
    let mut i = southern_cross();
    i["arm_length_capital_amount"] = json!("9000000.00");
    assert_eq!(amount(&run(&i), "total_disallowed_amount"), "0.00");
    // Without it a shortfall is refused, never estimated (s 820-315).
    let mut i = southern_cross();
    drop(&mut i, &["arm_length_capital_amount"]);
    let e = refusal(&i);
    assert_eq!(e.code, "thin_capitalisation.amount_not_worked_out");
    assert!(e.message.contains("s 820-315"));
    // With enough equity capital nothing more is needed.
    i["average_adi_equity_capital"] = json!("11000000.00");
    assert_eq!(amount(&run(&i), "total_disallowed_amount"), "0.00");
    // Without the worldwide figures, s 820-320 is named too.
    let mut i = southern_cross();
    drop(
        &mut i,
        &[
            "worldwide_capital_risk_weighted_assets",
            "worldwide_capital_tier_1_prudential_capital_deductions",
            "group_average_tier_1_capital",
            "group_average_risk_weighted_assets",
            "arm_length_capital_amount",
        ],
    );
    assert!(refusal(&i).message.contains("s 820-320"));
    // A foreign controlled Australian entity has no worldwide capital amount.
    i["foreign_controlled_australian_entity"] = json!(true);
    i["arm_length_capital_amount"] = json!("12000000.00");
    assert_eq!(amount(&run(&i), "minimum_capital_amount"), "11000000.00");
    let mut i = southern_cross();
    i["foreign_controlled_australian_entity"] = json!(true);
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    let mut i = southern_cross();
    i["group_average_risk_weighted_assets"] = json!("0.00");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
    let mut i = southern_cross();
    i["average_interest_free_pe_loans"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
    let mut i = southern_cross();
    i["test"] = json!("maximum_allowable_debt");
    assert_eq!(code(&i), "thin_capitalisation.fact_invalid");
}

/// Global Bank (s 820-405): a foreign bank's Australian branch, with $8.2m
/// of average equity capital and $5m of debt deductions.
fn global_bank() -> Value {
    let mut i = older(
        "inward_investing_adi",
        "minimum_capital_amount",
        json!([dd("Branch funding", "5000000.00")]),
    );
    for (k, v) in [
        ("average_debt_capital", "100000000.00"),
        ("average_adi_equity_capital", "8000000.00"),
        ("average_interest_free_pe_loans", "200000.00"),
        ("average_risk_weighted_assets", "140000000.00"),
        ("arm_length_capital_amount", "9000000.00"),
    ] {
        i[k] = json!(v);
    }
    i
}

#[test]
fn global_bank_inward_adi() {
    let r = run(&global_bank());
    assert_eq!(amount(&r, "safe_harbour_capital_amount"), "8400000.00");
    assert_eq!(amount(&r, "average_equity_capital"), "8200000.00");
    assert_eq!(amount(&r, "minimum_capital_amount"), "8400000.00");
    // 5m x 200,000 / 100m.
    assert_eq!(amount(&r, "total_disallowed_amount"), "10000.00");
    let mut i = global_bank();
    drop(&mut i, &["arm_length_capital_amount"]);
    let e = refusal(&i);
    assert_eq!(e.code, "thin_capitalisation.amount_not_worked_out");
    assert!(e.message.contains("s 820-410"));
    let mut i = global_bank();
    i["average_tier_1_prudential_capital_deductions"] = json!("1.00");
    assert_eq!(code(&i), "thin_capitalisation.input_not_used");
}

/// What a class's Subdivision does not provide, or this worksheet does not
/// apply, is refused with the provision named, for each of the 4 classes.
#[test]
fn unmodelled_tests_and_periods_are_refused_with_the_provision() {
    let classes = [
        (glm(), "s 820-90 or 820-190"),
        (investor(), "s 820-90 or 820-190"),
        (southern_cross(), "s 820-305 or 820-400"),
        (global_bank(), "s 820-305 or 820-400"),
    ];
    for (base, provision) in classes {
        let class = base["entity_class"].clone();
        for test in ["fixed_ratio", "group_ratio"] {
            let mut i = base.clone();
            i["test"] = json!(test);
            let e = refusal(&i);
            assert_eq!(e.code, "thin_capitalisation.fact_invalid", "{class} {test}");
            assert!(e.message.contains(provision), "{class} {test}");
        }
        let mut i = base;
        i["whole_year"] = json!(false);
        let e = refusal(&i);
        assert_eq!(e.code, "thin_capitalisation.part_year_not_built", "{class}");
        assert!(
            e.message
                .contains("ss 820-120, 820-225, 820-330 or 820-420"),
            "{class}"
        );
    }
}

/// Every general class result recorded on main before Subdivisions 820-B to
/// 820-E were built (4df634bb) is unchanged: amounts, rates, each deduction's
/// share and every refusal. Only the descriptive scope, sources, date read
/// and warnings text may differ.
#[test]
fn general_class_results_unchanged() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("data/thin_capitalisation_general.json")).unwrap();
    assert_eq!(rows.len(), 11);
    let text = ["scope", "sources", "source_checked", "warnings"];
    for row in rows {
        let input = &row["input"];
        match api::evaluate_coded(CMD, input) {
            Ok(mut r) => {
                let mut was = row["result"].clone();
                for k in text {
                    r.as_object_mut().unwrap().remove(k);
                    was.as_object_mut().unwrap().remove(k);
                }
                assert_eq!(r, was, "{input}");
            }
            Err(e) => {
                assert_eq!(e.code, row["refusal"]["code"], "{input}");
                assert_eq!(e.message, row["refusal"]["message"], "{input}");
            }
        }
    }
}
