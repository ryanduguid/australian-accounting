// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! An SMSF's income tax calculation statement (ITAA 1997 Division 295, the
//! Income Tax Rates Act 1986 ss 26 and 29 and the Supervisory Levy Imposition
//! Regulations 2018 s 6, read on 22 November 2026) against the ATO's
//! Instructions to complete your SMSF annual return 2026 (last updated
//! 30 May 2026): Appendix 1's 2 ECPI examples, the Section D examples and
//! table 9 (docs/calculation-evidence.md#smsf-tax).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("smsf-tax", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("smsf-tax", input).unwrap_err().code
}

/// An existing complying fund with nothing but `changes`.
fn fund(changes: Value) -> Value {
    let mut base = json!({
        "year": "2025-26", "scope_confirmed": true, "complying_fund": true,
        "australian_superannuation_fund": true, "assessable_contributions": "0",
        "no_tfn_contributions": "0", "net_non_arms_length_income": "0",
        "general_expense_non_arms_length_income": "0.00", "segregated_pension_income": "0",
        "unsegregated_income": "0", "segregated_non_current_income": "0",
        "exempt_proportion": "0", "disregarded_small_fund_assets": false, "deductions": "0",
        "contribution_deductions": "0", "non_refundable_offsets": "0.00",
        "carry_forward_offsets": "0.00", "refundable_offsets": "0.00",
        "eligible_credits": "0.00", "payg_instalments": "0.00", "interest_charge": "0.00",
        "newly_registered": false, "wound_up": false,
    });
    for (key, value) in changes.as_object().unwrap() {
        base[key] = value.clone();
    }
    base
}

fn amount<'a>(result: &'a Value, id: &str) -> &'a str {
    result["amounts"][id].as_str().unwrap()
}

fn assert_amounts(result: &Value, expected: &[(&str, &str)]) {
    for (id, want) in expected {
        assert_eq!(amount(result, id), *want, "{id}");
    }
}

/// Appendix 1 example 1: the assets are not segregated and the actuary
/// certifies 50%, so half the $234,200 is exempt; $62,000 of franking credits
/// refund all the tax and $44,435 more, less the $259 levy.
#[test]
fn the_ato_proportionate_example_agrees() {
    let out = run(&fund(json!({
        "unsegregated_income": "234200", "exempt_proportion": "0.5",
        "refundable_offsets": "62000.00",
    })));
    assert_amounts(
        &out,
        &[
            ("gross_income", "234200.00"),
            ("exempt_current_pension_income", "117100.00"),
            ("total_assessable_income", "117100.00"),
            ("taxable_income", "117100.00"),
            ("tax_on_taxable_income", "17565.00"),
            ("gross_tax", "17565.00"),
            ("subtotal_1", "17565.00"),
            ("subtotal_2", "17565.00"),
            ("tax_payable", "0.00"),
            ("tax_offset_refunds", "44435.00"),
            ("supervisory_levy", "259.00"),
            ("amount_due_or_refundable", "-44176.00"),
        ],
    );
    assert_eq!(out["return_labels"]["amount_due_or_refundable"], "S");
}

/// Appendix 1 example 2: the shares are segregated to support the pension, so
/// their $200,000 is exempt and the other $34,200 is taxed.
#[test]
fn the_ato_segregated_example_agrees() {
    let out = run(&fund(json!({
        "segregated_pension_income": "200000", "unsegregated_income": "34200",
        "refundable_offsets": "62000.00",
    })));
    assert_amounts(
        &out,
        &[
            ("exempt_current_pension_income", "200000.00"),
            ("taxable_income", "34200.00"),
            ("tax_on_taxable_income", "5130.00"),
            ("tax_offset_refunds", "56870.00"),
            ("amount_due_or_refundable", "-56611.00"),
        ],
    );
}

/// Section D label B example 1 (SMSF B): $14,500 at 15%.
#[test]
fn the_ato_gross_tax_example_agrees() {
    let out = run(&fund(json!({"unsegregated_income": "14500"})));
    assert_amounts(
        &out,
        &[
            ("taxable_income", "14500.00"),
            ("non_arms_length_component", "0.00"),
            ("tax_on_taxable_income", "2175.00"),
            ("no_tfn_contributions_tax", "0.00"),
            ("gross_tax", "2175.00"),
            ("amount_due_or_refundable", "2434.00"),
        ],
    );
}

/// Section D label B example 2 (SMSF BB): the lesser of $21,000 ($15,000 net
/// rent plus twice the $3,000 accounting shortfall) and $11,000 ($20,000 less
/// $10,000 of contributions plus their $1,000 deduction) is taxed at 45%.
#[test]
fn the_ato_non_arms_length_example_agrees() {
    let out = run(&fund(json!({
        "assessable_contributions": "10000", "net_non_arms_length_income": "15000",
        "general_expense_non_arms_length_income": "6000.00", "deductions": "5000",
        "contribution_deductions": "1000",
    })));
    assert_amounts(
        &out,
        &[
            ("gross_income", "25000.00"),
            ("taxable_income", "20000.00"),
            ("non_arms_length_component", "11000.00"),
            ("low_tax_component", "9000.00"),
            ("tax_on_taxable_income", "6300.00"),
        ],
    );
    // The other side of the lesser: without the contributions, $21,000.
    let out = run(&fund(json!({
        "net_non_arms_length_income": "15000", "unsegregated_income": "10000",
        "general_expense_non_arms_length_income": "6000.00", "deductions": "4000",
    })));
    assert_amounts(
        &out,
        &[
            ("non_arms_length_component", "21000.00"),
            ("low_tax_component", "0.00"),
            ("tax_on_taxable_income", "9450.00"),
        ],
    );
}

/// The T2, T3 and T5 examples, each on SMSF B's $2,175 of gross tax or the
/// $1,500 subtotal they reach.
#[test]
fn the_ato_offset_examples_agree() {
    let b = |offsets: Value| {
        let mut changes = json!({"unsegregated_income": "14500"});
        for (k, v) in offsets.as_object().unwrap() {
            changes[k] = v.clone();
        }
        run(&fund(changes))
    };
    let t2 = b(json!({"non_refundable_offsets": "675.00"}));
    assert_amounts(&t2, &[("subtotal_1", "1500.00"), ("subtotal_2", "1500.00")]);
    let over = b(json!({"non_refundable_offsets": "2500.00"}));
    assert_amounts(
        &over,
        &[
            ("subtotal_1", "0.00"),
            ("non_refundable_offsets_lost", "325.00"),
        ],
    );
    let t3 = b(json!({"carry_forward_offsets": "675.00"}));
    assert_amounts(
        &t3,
        &[
            ("subtotal_2", "1500.00"),
            ("carry_forward_offsets_unused", "0.00"),
        ],
    );
    let t3_over =
        b(json!({"non_refundable_offsets": "175.00", "carry_forward_offsets": "2500.00"}));
    assert_amounts(
        &t3_over,
        &[
            ("subtotal_2", "0.00"),
            ("carry_forward_offsets_unused", "500.00"),
        ],
    );
    let t5 = b(json!({"non_refundable_offsets": "675.00", "refundable_offsets": "500.00"}));
    assert_amounts(
        &t5,
        &[("tax_payable", "1000.00"), ("tax_offset_refunds", "0.00")],
    );
    let t5_over = b(json!({"non_refundable_offsets": "675.00", "refundable_offsets": "2000.00"}));
    assert_amounts(
        &t5_over,
        &[("tax_payable", "0.00"), ("tax_offset_refunds", "500.00")],
    );
}

/// The E2 example: Julie's $10,000 of no-TFN contributions for 2024-25 bore
/// $3,200 of extra tax, on top of the 15%.
#[test]
fn no_tfn_contributions_bear_32_percent_more() {
    let out = run(&fund(json!({
        "year": "2024-25", "assessable_contributions": "10000", "no_tfn_contributions": "10000",
    })));
    assert_amounts(
        &out,
        &[
            ("tax_on_taxable_income", "1500.00"),
            ("no_tfn_contributions_tax", "3200.00"),
            ("gross_tax", "4700.00"),
        ],
    );
}

/// Table 9: L less M plus N for each kind of fund, the same in each year.
#[test]
fn the_levy_follows_table_9() {
    let cases = [
        (false, false, ["259.00", "0.00", "0.00"], "259.00"),
        (true, false, ["259.00", "0.00", "259.00"], "518.00"),
        (false, true, ["259.00", "259.00", "0.00"], "0.00"),
        (true, true, ["259.00", "0.00", "0.00"], "259.00"),
    ];
    let years = [
        "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
    ];
    for year in years {
        for (new, wound_up, [l, m, n], due) in cases {
            let out = run(&fund(json!({
                "year": year, "newly_registered": new, "wound_up": wound_up,
            })));
            assert_amounts(
                &out,
                &[
                    ("supervisory_levy", l),
                    ("levy_adjustment_wound_up", m),
                    ("levy_adjustment_new_fund", n),
                    ("amount_due_or_refundable", due),
                ],
            );
        }
    }
}

/// The rates are the same in every year read, at each end of the range, and
/// the years either side are refused.
#[test]
fn the_rates_hold_from_2020_21_to_2026_27() {
    for year in ["2020-21", "2026-27"] {
        let out = run(&fund(json!({"year": year, "unsegregated_income": "14500"})));
        assert_eq!(amount(&out, "tax_on_taxable_income"), "2175.00");
        assert_eq!(out["rates"]["low_tax_rate"], "0.15");
        assert_eq!(out["rates"]["non_arms_length_rate"], "0.45");
        assert_eq!(out["rates"]["no_tfn_rate"], "0.32");
        assert_eq!(out["rates"]["supervisory_levy"], "259");
    }
    for year in ["2019-20", "2027-28"] {
        assert_eq!(code(&fund(json!({"year": year}))), "period.unsupported");
    }
}

/// A loss leaves nil taxable income and no tax, but the levy is still due;
/// the non-arm's length component cannot go below nil.
#[test]
fn a_loss_year_has_nil_taxable_income() {
    let out = run(&fund(json!({
        "unsegregated_income": "1000", "deductions": "3000",
        "net_non_arms_length_income": "500",
    })));
    assert_amounts(
        &out,
        &[
            ("taxable_income", "0.00"),
            ("loss", "1500.00"),
            ("non_arms_length_component", "0.00"),
            ("tax_on_taxable_income", "0.00"),
            ("amount_due_or_refundable", "259.00"),
        ],
    );
}

/// The proportion's product is rounded down to the whole dollar, and income
/// from segregated non-current assets is outside the proportion.
#[test]
fn exempt_income_is_in_whole_dollars() {
    let out = run(&fund(json!({
        "unsegregated_income": "1001", "exempt_proportion": "0.3333",
        "segregated_non_current_income": "500",
    })));
    assert_amounts(
        &out,
        &[
            ("gross_income", "1501.00"),
            ("exempt_current_pension_income", "333.00"),
            ("taxable_income", "1168.00"),
        ],
    );
    let all = run(&fund(
        json!({"unsegregated_income": "1001", "exempt_proportion": "1"}),
    ));
    assert_eq!(amount(&all, "taxable_income"), "0.00");
}

/// Credits, PAYG instalments and the interest charge go to label S.
#[test]
fn credits_and_instalments_reach_the_amount_due() {
    let out = run(&fund(json!({
        "unsegregated_income": "14500", "eligible_credits": "100.00",
        "payg_instalments": "2000.00", "interest_charge": "10.50",
    })));
    // 2,175 + 10.50 + 259 - 100 - 2,000
    assert_eq!(amount(&out, "amount_due_or_refundable"), "344.50");
}

#[test]
fn each_refusal_has_its_code() {
    let cases = [
        (json!({"complying_fund": false}), "smsf_tax.non_complying"),
        (
            json!({"australian_superannuation_fund": false}),
            "smsf_tax.foreign_fund",
        ),
        (
            json!({"segregated_pension_income": "1", "disregarded_small_fund_assets": true}),
            "smsf_tax.disregarded_small_fund_assets",
        ),
        (
            json!({"assessable_contributions": "100", "no_tfn_contributions": "101"}),
            "smsf_tax.part_exceeds_total",
        ),
        (
            json!({"assessable_contributions": "100", "deductions": "50", "contribution_deductions": "51"}),
            "smsf_tax.part_exceeds_total",
        ),
        (
            json!({"assessable_contributions": "10", "deductions": "50", "contribution_deductions": "11"}),
            "smsf_tax.part_exceeds_total",
        ),
        (
            json!({"exempt_proportion": "1.0001"}),
            "smsf_tax.proportion_invalid",
        ),
        (
            json!({"exempt_proportion": "-0.1"}),
            "smsf_tax.proportion_invalid",
        ),
        (
            json!({"unsegregated_income": "100.50"}),
            "input.amount_not_whole_dollars",
        ),
        (
            json!({"net_non_arms_length_income": "-1"}),
            "input.amount_invalid",
        ),
        (json!({"wound_up": "yes"}), "input.flag_invalid"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"member_balance": "1"}), "input.field_unknown"),
    ];
    for (changes, want) in cases {
        assert_eq!(code(&fund(changes.clone())), want, "{changes}");
    }
    // Disregarded small fund assets with all income unsegregated are fine.
    let ok = fund(json!({"disregarded_small_fund_assets": true, "unsegregated_income": "100"}));
    assert!(api::evaluate("smsf-tax", &ok).is_ok());
}
