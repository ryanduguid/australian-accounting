// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `company-tax-statement`: the published examples in the build spec, each year
//! and rate boundary, and every refusal.
use serde_json::{Value, json};
use the_exchequer_tally::api::evaluate;

fn run(v: &Value) -> Result<Value, String> {
    evaluate("company-tax-statement", v)
}
fn ok(v: &Value) -> Value {
    run(v).unwrap_or_else(|e| panic!("{e}"))
}
fn refused(v: &Value, needle: &str) {
    let e = run(v).expect_err("refused");
    assert!(e.contains(needle), "{e}");
}
fn company(fy: i64, turnover: &str, assessable: &str, deductions: &str) -> Value {
    json!({"financial_year": fy, "aggregated_turnover": turnover, "assessable_income": assessable,
        "passive_income": "0", "deductions": deductions})
}
fn labels(v: &Value) -> Value {
    ok(v)["calculation_statement"].clone()
}
const TEMPORARY: [&str; 6] = [
    "corporate_tax_entity_throughout",
    "returns_lodged",
    "choice_in_approved_form",
    "carried_on_business_each_loss_year",
    "no_transferred_losses",
    "no_integrity_scheme",
];
const PERMANENT: [&str; 6] = [
    "corporate_tax_entity_throughout",
    "not_significant_global_entity",
    "returns_lodged",
    "choice_in_approved_form",
    "no_transferred_losses",
    "no_integrity_scheme",
];
fn conditions(keys: &[&str]) -> Value {
    Value::Object(keys.iter().map(|k| (k.to_string(), json!(true))).collect())
}
/// The Division 160 example: Company A, not a base rate entity, a $900,000 loss.
fn company_a(fy: i64, first: i64, balance: &str) -> Value {
    let keys: &[&str] = if fy == 2027 { &PERMANENT } else { &TEMPORARY };
    let mut v = company(fy, "60000000", "100000", "1000000");
    v["loss_carry_back"] = json!({"franking_account_balance": balance, "conditions": conditions(keys),
        "carry_backs": [{"loss_year": fy, "earlier_year": first, "amount": "405000"},
                        {"loss_year": fy, "earlier_year": first + 1, "amount": "495000"}],
        "earlier_years": [{"year": first, "income_tax_liability": "120000", "net_exempt_income": "5000"},
                          {"year": first + 1, "income_tax_liability": "210000"}]});
    v
}

#[test]
fn div_160_example_temporary() {
    let r = ok(&company_a(2021, 2019, "280000"));
    let lcb = &r["loss_carry_back"];
    assert_eq!(lcb["components"][0]["component"], "120000.00");
    assert_eq!(lcb["components"][1]["component"], "148500.00");
    assert_eq!(lcb["offset"], "268500.00");
    let s = &r["calculation_statement"];
    assert_eq!(
        (&s["A"], &s["E"], &s["I"], &s["S"]),
        (
            &json!("0.00"),
            &json!("268500.00"),
            &json!("268500.00"),
            &json!("-268500.00")
        )
    );
    assert_eq!(r["taxable_income"]["tax_loss"], "900000.00");
    assert_eq!(r["taxable_income"]["tax_loss_carried_forward"], "0.00");
}
#[test]
fn div_160_example_franking_cap() {
    assert_eq!(
        ok(&company_a(2021, 2019, "250000"))["loss_carry_back"]["offset"],
        "250000.00"
    );
}
#[test]
fn div_160_example_permanent() {
    let r = ok(&company_a(2027, 2025, "280000"));
    assert_eq!(r["loss_carry_back"]["offset"], "268500.00");
    assert!(
        r["loss_carry_back"]["law"]
            .as_str()
            .unwrap()
            .contains("Tax Reform No. 2")
    );
}
fn loss_year(year: i64, tax_loss: &str, turnover: &str) -> Value {
    json!({"year": year, "tax_loss": tax_loss, "aggregated_turnover": turnover, "assessable_income": "50000", "passive_income": "0"})
}
/// ATO 'Working out the tax offset', GHI Pty Ltd.
#[test]
fn ato_ghi_example() {
    let mut v = company(2021, "60000000", "200000", "100000");
    v["loss_carry_back"] = json!({"franking_account_balance": "25000", "conditions": conditions(&TEMPORARY),
        "loss_years": [loss_year(2020, "100000", "60000000")],
        "carry_backs": [{"loss_year": 2020, "earlier_year": 2019, "amount": "100000"}],
        "earlier_years": [{"year": 2019, "income_tax_liability": "40000"}]});
    let r = ok(&v);
    assert_eq!(
        r["loss_carry_back"]["components"][0]["losses"][0]["step_3"],
        "30000.00"
    );
    assert_eq!(r["loss_carry_back"]["offset"], "25000.00");
    let s = &r["calculation_statement"];
    assert_eq!(
        (&s["B"], &s["T4"], &s["S"]),
        (&json!("30000.00"), &json!("5000.00"), &json!("5000.00"))
    );
    assert_eq!(r["taxable_income"]["tax_loss_carried_back"], "0.00");
}
/// ATO 'Working out the tax offset', MNO Pty Ltd: exempt income reduces the oldest loss only.
#[test]
fn ato_mno_exempt_income_example() {
    let mut v = company(2021, "60000000", "50000", "100000");
    v["loss_carry_back"] = json!({"franking_account_balance": "1000000", "conditions": conditions(&TEMPORARY),
        "loss_years": [loss_year(2020, "100000", "60000000")],
        "carry_backs": [{"loss_year": 2021, "earlier_year": 2019, "amount": "50000"},
                        {"loss_year": 2020, "earlier_year": 2019, "amount": "100000"}],
        "earlier_years": [{"year": 2019, "income_tax_liability": "1000000", "net_exempt_income": "20000"}]});
    let losses = ok(&v)["loss_carry_back"]["components"][0]["losses"].clone();
    assert_eq!(losses[0]["loss_year"], 2020);
    assert_eq!(losses[0]["after_net_exempt_income"], "80000.00");
    assert_eq!(losses[1]["after_net_exempt_income"], "50000.00");
}
/// CTR 2026 calculation statement examples 16a to 20b (a base rate entity, 25%).
fn ctr(a: &str, offsets: &[(&str, &str)]) -> Value {
    let mut v = company(2026, "1000000", a, "0");
    for (k, x) in offsets {
        v[*k] = json!(x);
    }
    labels(&v)
}
const C: &str = "non_refundable_non_carry_forward_offsets";
const D: &str = "non_refundable_carry_forward_offsets";
const E: &str = "refundable_tax_offsets";
const F: &str = "franking_deficit_tax_offset";
#[test]
fn ctr_examples_16() {
    let s = ctr("10000", &[(C, "2000")]);
    assert_eq!(
        (&s["B"], &s["T2"], &s["T5"], &s["I"], &s["S"]),
        (
            &json!("2500.00"),
            &json!("500.00"),
            &json!("500.00"),
            &json!("0.00"),
            &json!("500.00")
        )
    );
    let r = ok(&{
        let mut v = company(2026, "1000000", "10000", "0");
        v[C] = json!("4000");
        v
    });
    assert_eq!(r["calculation_statement"]["T5"], "0.00");
    assert_eq!(
        r["offsets_not_used"]["non_refundable_non_carry_forward_offsets_lost"],
        "1500.00"
    );
}
#[test]
fn ctr_examples_17() {
    let s = ctr("20000", &[(C, "3000"), (D, "1800")]);
    assert_eq!(
        (&s["T2"], &s["T3"], &s["T5"]),
        (&json!("2000.00"), &json!("200.00"), &json!("200.00"))
    );
    let mut v = company(2026, "1000000", "20000", "0");
    v[C] = json!("2800");
    v[D] = json!("4000");
    let r = ok(&v);
    assert_eq!(
        (
            &r["calculation_statement"]["T2"],
            &r["calculation_statement"]["T3"]
        ),
        (&json!("2200.00"), &json!("0.00"))
    );
    assert_eq!(
        r["offsets_not_used"]["non_refundable_carry_forward_offsets_carried_forward"],
        "1800.00"
    );
}
#[test]
fn ctr_examples_18() {
    let s = ctr("30000", &[(C, "3000"), (D, "3000"), (E, "1000")]);
    assert_eq!(
        (&s["T3"], &s["T4"], &s["T5"], &s["S"]),
        (
            &json!("1500.00"),
            &json!("500.00"),
            &json!("500.00"),
            &json!("500.00")
        )
    );
    let s = ctr("30000", &[(C, "3000"), (D, "3000"), (E, "4000")]);
    assert_eq!(
        (&s["T4"], &s["I"], &s["S"]),
        (&json!("0.00"), &json!("2500.00"), &json!("-2500.00"))
    );
}
#[test]
fn ctr_examples_20() {
    let s = ctr(
        "42000",
        &[(C, "3000"), (D, "3000"), (E, "3000"), (F, "1000")],
    );
    assert_eq!(
        (&s["B"], &s["T4"], &s["T5"]),
        (&json!("10500.00"), &json!("1500.00"), &json!("500.00"))
    );
    let mut v = company(2026, "1000000", "42000", "0");
    for k in [C, D, E] {
        v[k] = json!("3000");
    }
    v[F] = json!("4000");
    let r = ok(&v);
    assert_eq!(r["calculation_statement"]["T5"], "0.00");
    assert_eq!(
        r["offsets_not_used"]["franking_deficit_tax_offset_carried_forward"],
        "2500.00"
    );
}
#[test]
fn credits_and_instalments_reach_s() {
    let mut v = company(2026, "1000000", "100000", "0");
    v["section_102aam_interest"] = json!("10");
    v["eligible_credits"] = json!("1000");
    v["payg_instalments"] = json!("20000.50");
    assert_eq!(labels(&v)["S"], "4009.50");
}

#[test]
fn rate_for_each_year() {
    for (fy, rate) in [
        (2021, "0.260"),
        (2022, "0.250"),
        (2023, "0.250"),
        (2026, "0.250"),
        (2027, "0.250"),
    ] {
        let r = ok(&company(fy, "49999999.99", "1000", "0"));
        assert_eq!(r["corporate_tax"]["applicable_rate"], rate, "{fy}");
        assert_eq!(
            ok(&company(fy, "50000000", "1000", "0"))["corporate_tax"]["applicable_rate"],
            "0.300"
        );
    }
    assert_eq!(
        ok(&company(2021, "1", "1000", "0"))["calculation_statement"]["B"],
        "260.00"
    );
}
#[test]
fn gross_tax_rounds_half_up_to_the_cent() {
    assert_eq!(labels(&company(2021, "1", "0.02", "0"))["B"], "0.01");
}
#[test]
fn loss_year_2019_20_rate_is_27_5_percent() {
    let mut v = company(2021, "1000000", "200000", "0");
    v["loss_carry_back"] = json!({"franking_account_balance": "100000", "conditions": conditions(&TEMPORARY),
        "loss_years": [loss_year(2020, "10000", "1000000")],
        "carry_backs": [{"loss_year": 2020, "earlier_year": 2019, "amount": "10000"}],
        "earlier_years": [{"year": 2019, "income_tax_liability": "100000"}]});
    let step = ok(&v)["loss_carry_back"]["components"][0]["losses"][0].clone();
    assert_eq!(
        (&step["loss_year_rate"], &step["step_3"]),
        (&json!("0.275"), &json!("2750.00"))
    );
}
#[test]
fn years_outside_scope_are_refused() {
    refused(&company(2020, "1", "1", "0"), "outside 2021");
    refused(&company(2028, "1", "1", "0"), "outside 2021");
}
#[test]
fn carry_back_claim_years() {
    for fy in [2024, 2025, 2026] {
        refused(
            &company_a(fy, fy - 2, "280000"),
            "no loss carry back tax offset",
        );
    }
    assert_eq!(
        ok(&company_a(2023, 2021, "280000"))["loss_carry_back"]["offset"],
        "268500.00"
    );
    assert_eq!(
        ok(&company_a(2022, 2020, "280000"))["loss_carry_back"]["offset"],
        "268500.00"
    );
}

fn with_losses(losses: Value) -> Value {
    let mut v = company(2026, "1000000", "100000", "40000");
    v["prior_year_losses"] = losses;
    v
}
#[test]
fn losses_follow_the_stated_tests_oldest_first() {
    let v = with_losses(json!([
        {"loss_year": 2024, "amount": "50000", "test": "same_business"},
        {"loss_year": 2016, "amount": "20000", "test": "similar_business"},
        {"loss_year": 2020, "amount": "30000", "test": "failed"},
        {"loss_year": 2022, "amount": "25000", "test": "continuity_of_ownership"}]));
    let r = ok(&v);
    let rows = &r["taxable_income"]["prior_year_losses"];
    assert_eq!(rows[0]["deducted"], "20000.00");
    assert_eq!(rows[1]["deducted"], "0.00");
    assert_eq!(rows[2]["deducted"], "25000.00");
    assert_eq!(
        (&rows[3]["deducted"], &rows[3]["remaining"]),
        (&json!("15000.00"), &json!("35000.00"))
    );
    assert_eq!(r["calculation_statement"]["A"], "0.00");
}
#[test]
fn a_stated_choice_limits_the_deduction() {
    let mut v =
        with_losses(json!([{"loss_year": 2024, "amount": "50000", "test": "same_business"}]));
    v["loss_deduction_chosen"] = json!("10000");
    assert_eq!(labels(&v)["A"], "50000.00");
    v["loss_deduction_chosen"] = json!("60000.01");
    assert!(run(&v).unwrap_err().contains("exceeds"));
}
#[test]
fn net_exempt_income_absorbs_losses_first() {
    let mut v =
        with_losses(json!([{"loss_year": 2024, "amount": "50000", "test": "same_business"}]));
    v["net_exempt_income"] = json!("15000");
    let r = ok(&v);
    assert_eq!(
        r["taxable_income"]["losses_deducted_from_net_exempt_income"],
        "15000.00"
    );
    assert_eq!(r["calculation_statement"]["A"], "25000.00");
}
#[test]
fn a_loss_year_keeps_net_exempt_income_out_of_the_tax_loss() {
    let mut v = company(2026, "1000000", "100000", "130000");
    v["net_exempt_income"] = json!("10000");
    v["prior_year_losses"] =
        json!([{"loss_year": 2024, "amount": "5000", "test": "same_business"}]);
    let r = ok(&v);
    assert_eq!(r["taxable_income"]["tax_loss"], "20000.00");
    assert_eq!(
        r["taxable_income"]["losses_deducted_from_net_exempt_income"],
        "0.00"
    );
    v["loss_deduction_chosen"] = json!("1");
    refused(&v, "s 36-17(4)");
}
#[test]
fn loss_test_refusals() {
    refused(
        &with_losses(json!([{"loss_year": 2015, "amount": "1", "test": "similar_business"}])),
        "s 165-211(1)(a)",
    );
    refused(
        &with_losses(json!([{"loss_year": 2026, "amount": "1", "test": "same_business"}])),
        "s 36-17(1)",
    );
    assert!(
        run(&with_losses(
            json!([{"loss_year": 2025, "amount": "1", "test": "judge"}])
        ))
        .is_err()
    );
}
#[test]
fn scope_flags_are_refused() {
    for (flag, law) in [
        ("consolidated_group", "Part 3-90"),
        ("foreign_hybrid_or_cfc_income", "scope"),
        ("foreign_resident", "160-10(3)"),
        ("special_rate_company", "23A"),
        ("subdivision_165_b_applies", "Subdiv 165-B"),
    ] {
        let mut v = company(2026, "1", "1", "0");
        v[flag] = json!(true);
        refused(&v, law);
        v[flag] = json!(false);
        ok(&v);
    }
}
#[test]
fn offset_rules_not_modelled_are_refused() {
    let mut v =
        with_losses(json!([{"loss_year": 2024, "amount": "50000", "test": "same_business"}]));
    v[C] = json!("100");
    v["franking_tax_offsets"] = json!("100");
    refused(&v, "s 36-17(5)");
    let mut v = company(2026, "1000000", "1000", "0");
    v[C] = json!("300");
    v["franking_tax_offsets"] = json!("100");
    refused(&v, "s 36-55");
    v[C] = json!("250");
    v["franking_tax_offsets"] = json!("200");
    ok(&v);
    v[C] = json!("250.01");
    refused(&v, "s 36-55");
    v[C] = json!("300");
    v["franking_tax_offsets"] = json!("301");
    assert!(run(&v).unwrap_err().contains("cannot exceed"));
    let mut v = company(2026, "1000000", "1000", "0");
    v[D] = json!("1");
    v["net_exempt_income"] = json!("1");
    refused(&v, "Div 65");
}
#[test]
fn carry_back_refusals() {
    for key in TEMPORARY {
        let mut v = company_a(2021, 2019, "280000");
        v["loss_carry_back"]["conditions"][key] = json!(false);
        refused(&v, key);
    }
    let mut v = company_a(2027, 2025, "280000");
    v["loss_carry_back"]["conditions"]["not_significant_global_entity"] = json!(false);
    refused(&v, "160-5(1)(e)");
    let mut v = company_a(2027, 2025, "280000");
    v["loss_carry_back"]["earlier_years"][0]["income_tax_liability"] = json!("0");
    refused(&v, "160-5(2)(b)");
    refused(&company_a(2027, 2024, "280000"), "outside the claim year");
    refused(&company_a(2021, 2018, "280000"), "outside a loss year");
    let mut v = company_a(2021, 2019, "280000");
    v["loss_carry_back"]["carry_backs"][0]["amount"] = json!("405000.01");
    refused(&v, "exceeds its tax loss");
    let mut v = company_a(2021, 2019, "280000");
    v["aggregated_turnover"] = json!("5000000000");
    refused(&v, "s 160-20");
    let mut v = company_a(2021, 2019, "280000");
    v["loss_carry_back"]["franking_account_balance"] = Value::Null;
    assert!(run(&v).unwrap_err().contains("required"));
    let mut v = company_a(2027, 2025, "280000");
    v["deductions"] = json!("0");
    refused(&v, "160-5(1)(c)");
}
#[test]
fn malformed_amounts_are_refused() {
    let mut v = company(2026, "1", "1", "0");
    v["payg_instalments"] = json!("1.001");
    assert!(run(&v).unwrap_err().contains("at most 2 decimal places"));
    v["payg_instalments"] = json!("-1");
    assert!(run(&v).is_err());
}
