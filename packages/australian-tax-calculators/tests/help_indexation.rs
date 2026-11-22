// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! HELP indexation (HESA Division 140, compilation 105, and the 2024 and 2025
//! amending Acts, read on 22 November 2026) against the ATO's published rates
//! ('Study and training loan indexation rates', last updated 17 April 2026),
//! its 20% reduction examples ('View your study loan account online',
//! Examples 1 and 2, last updated 3 June 2026), the Act's s 140-5 example and
//! hand-worked cases at each boundary
//! (docs/calculation-evidence.md#help-indexation).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("help-indexation", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("help-indexation", input)
        .unwrap_err()
        .code
}

fn base(year: &str, debt: &str) -> Value {
    json!({
        "year": year, "scope_confirmed": true, "location_preferred": false,
        "previous_accumulated_debt": debt,
    })
}

fn with(mut input: Value, key: &str, value: Value) -> Value {
    input[key] = value;
    input
}

fn amount(out: &Value, key: &str) -> String {
    out["amounts"][key].as_str().unwrap().to_owned()
}

#[test]
fn every_published_factor_indexes_a_balance() {
    // The ATO's rate for each 1 June plus one; 2024-25 also takes 20% off.
    let rows = [
        ("2019-20", "1.018", "10180.00", "CPI"),
        ("2020-21", "1.006", "10060.00", "CPI"),
        ("2021-22", "1.039", "10390.00", "CPI"),
        ("2022-23", "1.032", "10320.00", "lower of CPI and WPI"),
        ("2023-24", "1.040", "10400.00", "lower of CPI and WPI"),
        ("2024-25", "1.032", "8256.00", "lower of CPI and WPI"),
        ("2025-26", "1.028", "10280.00", "lower of CPI and WPI"),
    ];
    for (year, factor, debt, rule) in rows {
        let out = run(&base(year, "10000.00"));
        assert_eq!(out["rates"]["indexation_factor"], factor, "{year}");
        assert_eq!(out["rates"]["factor_rule"], rule, "{year}");
        assert_eq!(amount(&out, "accumulated_debt"), debt, "{year}");
    }
}

#[test]
fn recalculated_years_carry_the_first_published_factor() {
    let first = |year| run(&base(year, "1.00"))["rates"]["superseded_factor"].clone();
    assert_eq!(first("2022-23"), "1.071");
    assert_eq!(first("2023-24"), "1.047");
    assert_eq!(first("2021-22"), Value::Null);
    assert_eq!(first("2024-25"), Value::Null);
}

#[test]
fn ato_example_1_maylin() {
    // $90,000 from 1 July 2024 and $11,000 added in November 2024: $18,000
    // off before indexation, indexation of $2,880 less the $576 excess, and
    // $2,200 off the new debt. Her $90,304 also holds the May 2025 debt of
    // $9,000 less 20%, which joins the 1 June 2026 calculation.
    let input = with(
        base("2024-25", "90000.00"),
        "current_debts",
        json!([{"incurred": "2024-11-15", "amount": "11000.00"}]),
    );
    let out = run(&input);
    assert_eq!(amount(&out, "reduction"), "18000.00");
    assert_eq!(amount(&out, "indexation"), "2304.00");
    assert_eq!(amount(&out, "former_accumulated_debt"), "74304.00");
    assert_eq!(amount(&out, "current_reduction"), "2200.00");
    assert_eq!(amount(&out, "current_debts"), "8800.00");
    assert_eq!(amount(&out, "accumulated_debt"), "83104.00");
    assert_eq!(out["rates"]["reduction_rate"], "0.20");
    // 83,104 + 7,200 = 90,304, the ATO's total.
    let next = with(
        base("2025-26", "0.00"),
        "earlier_debts",
        json!([{"incurred": "2025-05-20", "amount": "9000.00"}]),
    );
    assert_eq!(amount(&run(&next), "earlier_debts"), "7200.00");
}

#[test]
fn ato_example_2_asha() {
    // $10,000 on 31 May 2025: $2,000 off, indexation $320 less $64.
    let out = run(&base("2024-25", "10000.00"));
    assert_eq!(amount(&out, "reduction"), "2000.00");
    assert_eq!(amount(&out, "indexation"), "256.00");
    assert_eq!(amount(&out, "accumulated_debt"), "8256.00");
}

#[test]
fn act_example_lorraine_moved_to_2020() {
    // HESA s 140-5 example: 15,000 + 1,500 - 525 - 3,000 = 12,975 before
    // indexation, here dated 7 years later and indexed at 1.018.
    let mut input = base("2019-20", "15000.00");
    input["earlier_debts"] = json!([{"incurred": "2019-03-31", "amount": "1500.00"}]);
    input["voluntary_repayments"] = json!([{"date": "2020-05-01", "amount": "525.00"}]);
    input["compulsory_repayments"] = json!("3000.00");
    let out = run(&input);
    assert_eq!(amount(&out, "before_indexation"), "12975.00");
    assert_eq!(amount(&out, "former_accumulated_debt"), "13208.55");
    // s 140-30: cents disregarded.
    assert_eq!(amount(&out, "accumulated_debt"), "13208.00");
}

#[test]
fn amended_assessments_move_the_amount_indexed() {
    let mut input = base("2025-26", "10000.00");
    input["assessment_increases"] = json!("500.00");
    input["assessment_reductions"] = json!("200.00");
    assert_eq!(amount(&run(&input), "before_indexation"), "9700.00");
}

#[test]
fn a_repayment_before_1_june_avoids_indexation() {
    let input = with(
        base("2025-26", "20000.00"),
        "proposed_repayment",
        json!("5000.00"),
    );
    let out = run(&input);
    assert_eq!(amount(&out, "accumulated_debt"), "20560.00");
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_before_1_june"),
        "15420.00"
    );
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_on_or_after_1_june"),
        "15560.00"
    );
    assert_eq!(amount(&out, "saving_if_paid_before_1_june"), "140.00");
}

#[test]
fn before_the_2025_reduction_an_early_repayment_cost_more() {
    // s 140-26(1): a repayment made before 1 June 2025 is inside the 20%
    // reduction, so it removes only 80% of itself before indexation.
    let input = with(
        base("2024-25", "10000.00"),
        "proposed_repayment",
        json!("1000.00"),
    );
    let out = run(&input);
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_before_1_june"),
        "7430.00"
    );
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_on_or_after_1_june"),
        "7256.00"
    );
    assert_eq!(amount(&out, "saving_if_paid_before_1_june"), "-174.00");
}

#[test]
fn a_proposed_repayment_above_the_indexed_debts_goes_on_to_the_years_debts() {
    // HESA s 151-10(2): a payment without directions goes first to the
    // accumulated HELP debt, then to HELP debts in the order incurred. Found
    // by a fresh-context read; the command refused this before.
    let mut input = base("2025-26", "100.00");
    input["current_debts"] = json!([{"incurred": "2025-08-01", "amount": "1000.00"}]);
    input["proposed_repayment"] = json!("500.00");
    let out = run(&input);
    assert_eq!(amount(&out, "accumulated_debt"), "1102.00");
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_before_1_june"),
        "600.00"
    );
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_on_or_after_1_june"),
        "602.00"
    );
    assert_eq!(amount(&out, "saving_if_paid_before_1_june"), "2.00");
    // 2024-25: the part on the year's debts is inside s 140-26(2)'s 80%.
    let mut input = base("2024-25", "100.00");
    input["current_debts"] = json!([{"incurred": "2024-08-01", "amount": "1000.00"}]);
    input["proposed_repayment"] = json!("500.00");
    let out = run(&input);
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_before_1_june"),
        "480.00"
    );
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_on_or_after_1_june"),
        "382.00"
    );
    // Above both, refused.
    let mut input = base("2025-26", "100.00");
    input["current_debts"] = json!([{"incurred": "2025-08-01", "amount": "1000.00"}]);
    input["proposed_repayment"] = json!("1100.01");
    assert_eq!(code(&input), "help_indexation.repayments_exceed_debt");
}

#[test]
fn the_2025_26_balance_is_the_reduced_1_june_2025_debt() {
    // s 140-5(1) step 1 takes the 1 June 2025 accumulated HELP debt, which
    // s 140-26 already reduced: Asha's 8,256 carried on gives 8,487. Found by
    // a fresh-context read; the command refused the reduced balance before.
    let first = run(&base("2024-25", "10000.00"));
    let carried = amount(&first, "accumulated_debt");
    let next = with(
        base("2025-26", &carried),
        "debts_already_reduced",
        json!(true),
    );
    assert_eq!(amount(&run(&next), "accumulated_debt"), "8487.00");
    // A debt of 1 January to 1 June 2025 is still reduced by the command.
    let early = json!([{"incurred": "2025-03-01", "amount": "100.00"}]);
    let reduced = with(next, "earlier_debts", early);
    assert_eq!(code(&reduced), "help_indexation.reduction_already_applied");
    let later = json!([{"incurred": "2025-06-02", "amount": "100.00"}]);
    let input = with(
        base("2025-26", "0.00"),
        "debts_already_reduced",
        json!(true),
    );
    assert_eq!(
        amount(&run(&with(input, "earlier_debts", later)), "earlier_debts"),
        "100.00"
    );
}

#[test]
fn a_repayment_larger_than_the_debt_after_1_june_leaves_nil() {
    let input = with(
        base("2025-26", "100.00"),
        "proposed_repayment",
        json!("50.00"),
    );
    let out = run(&input);
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_on_or_after_1_june"),
        "52.00"
    );
    let input = with(
        base("2025-26", "100.00"),
        "proposed_repayment",
        json!("100.00"),
    );
    let out = run(&input);
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_before_1_june"),
        "0.00"
    );
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_on_or_after_1_june"),
        "2.00"
    );
    let input = with(base("2025-26", "1.00"), "proposed_repayment", json!("1.00"));
    let out = run(&with(input, "earlier_debts", json!([])));
    assert_eq!(amount(&out, "accumulated_debt"), "1.00");
    let input = with(
        base("2025-26", "1.00"),
        "compulsory_repayments",
        json!("0.50"),
    );
    let input = with(input, "proposed_repayment", json!("0.50"));
    let out = run(&input);
    // 0.50 x 1.028 is under $1, so nil; paying 0.50 later still leaves nil.
    assert_eq!(
        amount(&out, "accumulated_debt_if_paid_on_or_after_1_june"),
        "0.00"
    );
}

#[test]
fn section_137_19a_reduces_debts_from_1_january_to_1_june_2025() {
    let debts = json!([
        {"incurred": "2025-01-01", "amount": "1000.00"},
        {"incurred": "2025-06-01", "amount": "1000.00"},
        {"incurred": "2025-06-02", "amount": "1000.00"},
    ]);
    let out = run(&with(base("2025-26", "0.00"), "earlier_debts", debts));
    assert_eq!(amount(&out, "earlier_debts"), "2600.00");
    // Debts in 2024 are not touched by s 137-19A (s 140-26 reduces them).
    let early = json!([{"incurred": "2024-12-31", "amount": "1000.00"}]);
    let out = run(&with(base("2024-25", "0.00"), "current_debts", early));
    assert_eq!(amount(&out, "current_debts"), "800.00");
}

#[test]
fn each_group_is_cut_to_whole_dollars() {
    let debts = json!([
        {"incurred": "2025-07-01", "amount": "100.60", "group": "course a"},
        {"incurred": "2025-08-01", "amount": "100.60", "group": "course a"},
        {"incurred": "2025-09-01", "amount": "0.99"},
        {"incurred": "2025-10-01", "amount": "50.50"},
    ]);
    let out = run(&with(base("2025-26", "0.00"), "current_debts", debts));
    // 201.20 -> 201; 0.99 -> 0; 50.50 -> 50.
    assert_eq!(amount(&out, "current_debts"), "251.00");
}

#[test]
fn an_accumulated_debt_under_one_dollar_is_nil() {
    assert_eq!(
        amount(&run(&base("2025-26", "0.97")), "accumulated_debt"),
        "0.00"
    );
    assert_eq!(
        amount(&run(&base("2025-26", "0.98")), "accumulated_debt"),
        "1.00"
    );
}

#[test]
fn current_repayments_reduce_current_debts_without_indexation() {
    let mut input = base("2025-26", "1000.00");
    input["current_debts"] = json!([{"incurred": "2025-12-31", "amount": "500.00"}]);
    input["current_repayments"] = json!("200.00");
    let out = run(&input);
    assert_eq!(amount(&out, "accumulated_debt"), "1328.00");
}

#[test]
fn period_boundaries() {
    let repay = |date: &str| {
        with(
            base("2019-20", "1000.00"),
            "voluntary_repayments",
            json!([{"date": date, "amount": "100.00"}]),
        )
    };
    assert_eq!(
        amount(&run(&repay("2019-06-01")), "voluntary_repayments"),
        "100.00"
    );
    assert_eq!(
        amount(&run(&repay("2020-05-31")), "voluntary_repayments"),
        "100.00"
    );
    assert_eq!(
        code(&repay("2019-05-31")),
        "help_indexation.date_outside_period"
    );
    assert_eq!(
        code(&repay("2020-06-01")),
        "help_indexation.date_outside_period"
    );
    let debt = |key: &str, date: &str| {
        with(
            base("2019-20", "0.00"),
            key,
            json!([{"incurred": date, "amount": "100.00"}]),
        )
    };
    for date in ["2019-01-01", "2019-06-30"] {
        assert_eq!(
            amount(&run(&debt("earlier_debts", date)), "earlier_debts"),
            "100.00"
        );
    }
    for date in ["2018-12-31", "2019-07-01"] {
        assert_eq!(
            code(&debt("earlier_debts", date)),
            "help_indexation.date_outside_period"
        );
    }
    for date in ["2019-07-01", "2019-12-31"] {
        assert_eq!(
            amount(&run(&debt("current_debts", date)), "current_debts"),
            "100.00"
        );
    }
    for date in ["2019-06-30", "2020-01-01"] {
        assert_eq!(
            code(&debt("current_debts", date)),
            "help_indexation.date_outside_period"
        );
    }
}

#[test]
fn refusals() {
    let mut missing = base("2025-26", "1.00");
    missing
        .as_object_mut()
        .unwrap()
        .remove("previous_accumulated_debt");
    assert_eq!(code(&missing), "help_indexation.debt_missing");
    let located = with(base("2025-26", "1.00"), "location_preferred", json!(true));
    assert_eq!(code(&located), "help_indexation.location_preferred");
    let reduced = with(
        base("2024-25", "1.00"),
        "debts_already_reduced",
        json!(true),
    );
    assert_eq!(code(&reduced), "help_indexation.reduction_already_applied");
    let not_reduced = with(
        base("2024-25", "10000.00"),
        "debts_already_reduced",
        json!(false),
    );
    assert_eq!(amount(&run(&not_reduced), "accumulated_debt"), "8256.00");
    let over = with(
        base("2025-26", "100.00"),
        "compulsory_repayments",
        json!("100.01"),
    );
    assert_eq!(code(&over), "help_indexation.repayments_exceed_debt");
    let mut current = base("2025-26", "0.00");
    current["current_repayments"] = json!("1.00");
    assert_eq!(code(&current), "help_indexation.repayments_exceed_debt");
    let proposed = with(
        base("2025-26", "100.00"),
        "proposed_repayment",
        json!("100.01"),
    );
    assert_eq!(code(&proposed), "help_indexation.repayments_exceed_debt");
    for bad in [
        json!("x"),
        json!([1]),
        json!([{"incurred": "2025-13-01", "amount": "1.00"}]),
    ] {
        let input = with(base("2025-26", "0.00"), "current_debts", bad);
        assert_eq!(code(&input), "help_indexation.entries_invalid");
    }
    let group = json!([{"incurred": "2025-07-01", "amount": "1.00", "group": 3}]);
    let input = with(base("2025-26", "0.00"), "current_debts", group);
    assert_eq!(code(&input), "help_indexation.entries_invalid");
    let many: Vec<Value> = (0..101)
        .map(|_| json!({"date": "2025-07-01", "amount": "1.00"}))
        .collect();
    let input = with(
        base("2025-26", "1000.00"),
        "voluntary_repayments",
        json!(many),
    );
    assert_eq!(code(&input), "help_indexation.entries_invalid");
    let unknown = json!([{"date": "2025-07-01", "amount": "1.00", "note": "x"}]);
    let input = with(base("2025-26", "1000.00"), "voluntary_repayments", unknown);
    assert_eq!(code(&input), "input.field_unknown");
    for year in ["2018-19", "2026-27"] {
        assert_eq!(code(&base(year, "1.00")), "period.unsupported");
    }
    let unconfirmed = with(base("2025-26", "1.00"), "scope_confirmed", json!(false));
    assert_eq!(code(&unconfirmed), "scope.not_confirmed");
    let stray = with(base("2025-26", "1.00"), "accumulated_debt", json!("1.00"));
    assert_eq!(code(&stray), "input.field_unknown");
    let negative = base("2025-26", "-1.00");
    assert_eq!(code(&negative), "input.amount_invalid");
}

#[test]
fn the_result_names_its_date_and_authorities() {
    let out = run(&base("2025-26", "1.00"));
    assert_eq!(out["indexation_date"], "2026-06-01");
    assert_eq!(out["calculation"], "help_indexation");
    assert!(
        out["authorities"]["reduction"]
            .as_str()
            .unwrap()
            .contains("s 140-26(1)")
    );
}
