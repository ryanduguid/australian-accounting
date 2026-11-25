// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `lessee-lease` against IFRS 16 Illustrative Example 13 and AASB 16
//! (compilation no. 8) paras 24, 26, 27, 32 and Appendix A, with its
//! refusals, read on 26 November 2026. The site's own cases are in
//! `site_mirrors.rs`.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn base() -> Value {
    json!({
        "year": "2026-27", "scope_confirmed": true, "payment": "1000", "payments": 60,
        "frequency": "monthly", "timing": "arrears", "discount_rate": "0.075",
        "commencement": "2026-07-01", "ownership_transfers": false, "remeasurement": false,
    })
}

fn with(changes: Value) -> Value {
    let mut v = base();
    for (k, x) in changes.as_object().unwrap() {
        v[k] = x.clone();
    }
    v
}

fn run(changes: Value) -> Value {
    api::evaluate("lessee-lease", &with(changes)).unwrap()
}

fn code(changes: Value) -> &'static str {
    api::evaluate_coded("lessee-lease", &with(changes)).unwrap_err().code
}

fn rounded(text: &Value) -> i64 {
    text.as_str().unwrap().parse::<f64>().unwrap().round() as i64
}

/// Example 13: 10 yearly payments of CU50,000 in advance at 5%, initial direct
/// costs of CU20,000 and an incentive of CU5,000. The liability is CU355,391,
/// the asset CU420,391, interest CU17,770 and CU16,158 in years 1 and 2 and
/// depreciation CU42,039 a year, each to the nearest whole unit.
#[test]
fn ifrs_16_example_13() {
    let out = run(json!({"payment": "50000", "payments": 10, "frequency": "annual", "timing": "advance",
        "discount_rate": "0.05", "initial_direct_costs": "20000", "incentives": "5000"}));
    assert_eq!(rounded(&out["amounts"]["lease_liability"]), 355_391);
    assert_eq!(rounded(&out["amounts"]["right_of_use_asset"]), 420_391);
    let years = out["income_years"].as_array().unwrap();
    assert_eq!(rounded(&years[0]["interest"]), 17_770);
    assert_eq!(rounded(&years[1]["interest"]), 16_158);
    for year in &years[..10] {
        assert_eq!(rounded(&year["depreciation"]), 42_039);
    }
    let journal = &out["journal"];
    assert_eq!(journal[0], json!({"entry": "Debit right-of-use asset", "amount": "420391.08"}));
    assert_eq!(journal[2]["entry"], "Credit bank: paid at or before commencement, less incentives");
    assert_eq!(journal[2]["amount"], "65000.00");
}

/// Para 32: over the useful life where ownership transfers, or where the life
/// ends before the lease term; otherwise over the lease term.
#[test]
fn depreciation_follows_paragraph_32() {
    let transfer = run(json!({"ownership_transfers": true, "useful_life_months": 96, "residual": "5000"}));
    assert_eq!(transfer["depreciated_to"], "2034-07-01");
    let shorter = run(json!({"useful_life_months": 36}));
    assert_eq!(shorter["depreciated_to"], "2029-07-01");
    let longer = run(json!({"useful_life_months": 96}));
    assert_eq!(longer["depreciated_to"], "2031-07-01");
    assert_eq!(longer["lease_term_ends"], "2031-07-01");
}

/// Appendix A: a lease term of 12 months or less with no purchase option.
#[test]
fn a_short_term_lease_is_flagged() {
    assert_eq!(run(json!({"payments": 12}))["short_term"], true);
    assert_eq!(run(json!({"payments": 13}))["short_term"], false);
    assert_eq!(run(json!({"payments": 12, "ownership_transfers": true, "useful_life_months": 60}))["short_term"], false);
}

/// Para 24: an incentive above what was paid at commencement debits the bank.
#[test]
fn net_incentives_debit_the_bank() {
    let out = run(json!({"incentives": "3000", "initial_direct_costs": "500", "restoration": "2000"}));
    let journal = out["journal"].as_array().unwrap();
    assert_eq!(journal[2], json!({"entry": "Credit provision for restoration", "amount": "2000.00"}));
    assert_eq!(journal[3], json!({"entry": "Debit bank: incentives received, less what was paid", "amount": "2500.00"}));
    assert_eq!(out["amounts"]["restoration_provision"], "2000.00");
}

/// A residual value guarantee above the liability is still a lease: the
/// liability is the payments' and the guarantee's present value.
#[test]
fn a_large_residual_is_measured() {
    let out = run(json!({"payment": "100", "payments": 12, "residual": "50000", "discount_rate": "0.05"}));
    let rows = out["schedule"].as_array().unwrap();
    assert_eq!(rows.last().unwrap()["closing"], "0.00");
    assert_eq!(rows.last().unwrap()["paid"], "50100.00");
}

#[test]
fn refuses_what_it_does_not_compute() {
    for (changes, want) in [
        (json!({"remeasurement": true}), "lessee_lease.remeasurement"),
        (json!({"incentives": "60000"}), "lessee_lease.cost_nil"),
        (json!({"ownership_transfers": true, "residual": "5000"}), "lessee_lease.useful_life_needed"),
        (json!({"useful_life_months": 601}), "lessee_lease.fact_invalid"),
        (json!({"frequency": "daily"}), "lessee_lease.fact_invalid"),
        (json!({"timing": "middle"}), "lessee_lease.fact_invalid"),
        (json!({"payments": 601}), "lessee_lease.fact_invalid"),
        (json!({"payments": 0}), "lessee_lease.fact_invalid"),
        (json!({"payment": "0"}), "lessee_lease.fact_invalid"),
        (json!({"discount_rate": "1.01"}), "lessee_lease.fact_invalid"),
        (json!({"discount_rate": "0.0750001"}), "lessee_lease.fact_invalid"),
        (json!({"commencement": "2027-07-01"}), "lessee_lease.fact_invalid"),
        (json!({"commencement": "2026-02-30"}), "input.date_invalid"),
        (json!({"remeasurement": "no"}), "input.flag_invalid"),
        (json!({"year": "2023-24", "commencement": "2023-07-01"}), "period.unsupported"),
        (json!({"variable_payments": "100"}), "input.field_unknown"),
    ] {
        assert_eq!(code(changes.clone()), want, "{changes}");
    }
}

/// The last period's interest takes the cents rounding leaves; where rounding
/// up each period leaves it negative, the convention cannot close the
/// schedule and the case is refused, as the site refuses it.
#[test]
fn negative_last_interest_is_refused() {
    let tiny = json!({"payment": "0.56", "payments": 5, "frequency": "annual", "discount_rate": "0.002983"});
    assert_eq!(code(tiny), "lessee_lease.rounding_negative");
}
