// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `super-projection` worked by hand and at each rule of ASIC Instrument
//! 2022/603 s 4 (compilation no. 1), the 2026-27 maximum contribution base and
//! caps (ATO, read on 26 November 2026), and its refusals. The site's own
//! cases are in `site_mirrors.rs`.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn base() -> Value {
    json!({
        "year": "2026-27", "scope_confirmed": true, "age": 30, "retirement_age": 67,
        "balance": "50000", "salary": "80000", "employer_rate": "0.12",
        "salary_sacrifice": "0", "after_tax_contributions": "0", "return_rate": "0.061",
        "percent_fee": "0.0011", "admin_fee": "59", "insurance": "599",
        "wage_growth": "0.037", "discount_rate": "0.037",
    })
}

fn with(changes: Value) -> Value {
    let mut v = base();
    for (k, x) in changes.as_object().unwrap() {
        if x.is_null() {
            v.as_object_mut().unwrap().remove(k);
        } else {
            v[k] = x.clone();
        }
    }
    v
}

fn run(changes: Value) -> Value {
    api::evaluate("super-projection", &with(changes)).unwrap()
}

fn code(changes: Value) -> &'static str {
    api::evaluate_coded("super-projection", &with(changes))
        .unwrap_err()
        .code
}

fn amount(out: &Value, key: &str) -> String {
    out["amounts"][key].as_str().unwrap().to_owned()
}

/// One year by hand: $9,600 from the employer less $1,440 tax, $59 and $599
/// out, so $7,502 of net flows and an average balance of $53,751; 6.1% and
/// 0.11% of it are $3,278.81 and $59.13, and 15% of the $717.13 of fees is
/// deducted, leaving $609.56. $50,000 + $8,160 + $3,278.81 - $609.56 is
/// $60,829.25, and over 1.037 it is $58,658.87.
#[test]
fn one_year_by_hand() {
    let out = run(json!({"age": 66}));
    assert_eq!(out["years"], 1);
    for (k, want) in [
        ("employer_contributions_2026_27", "9600.00"),
        ("contributions_less_tax", "8160.00"),
        ("earnings", "3278.81"),
        ("fees_and_premiums", "609.56"),
        ("balance_at_retirement", "60829.25"),
        ("present_value", "58658.87"),
    ] {
        assert_eq!(amount(&out, k), want, "{k}");
    }
}

/// The employer pays its rate up to the $270,830 maximum contribution base:
/// 12% of it is $32,499.60.
#[test]
fn employer_contributions_stop_at_the_maximum_contribution_base() {
    for (salary, rate, want) in [
        ("270830", "0.12", "32499.60"),
        ("400000", "0.12", "32499.60"),
        ("300000", "0.10", "27083.00"),
        ("80000", "0.15", "12000.00"),
    ] {
        let out = run(json!({"salary": salary, "employer_rate": rate}));
        assert_eq!(amount(&out, "employer_contributions_2026_27"), want);
    }
}

/// s 4 'assumed retirement age': the greater of the age entered and the
/// preservation age; 67 when none is entered, or the next birthday's age from
/// 67. 'present value': 3.7% when no other rate is entered.
#[test]
fn the_instrument_sets_the_retirement_age_and_present_value_rate() {
    let early = run(json!({"age": 45, "retirement_age": 55}));
    assert_eq!((early["retirement_age"].clone(), early["years"].clone()), (json!(60), json!(15)));
    assert_eq!(early["amounts"], run(json!({"age": 45, "retirement_age": 60}))["amounts"]);
    let default = run(json!({"retirement_age": null, "discount_rate": null}));
    assert_eq!(default["retirement_age"], 67);
    assert_eq!(default["amounts"], run(json!({}))["amounts"]);
    assert_eq!(run(json!({"age": 70, "retirement_age": null}))["retirement_age"], 71);
    let flat = run(json!({"discount_rate": "0"}));
    assert_eq!(amount(&flat, "present_value"), amount(&flat, "balance_at_retirement"));
}

/// Fees and premiums take no more than the account holds.
#[test]
fn fees_never_overdraw_the_account() {
    let out = run(json!({"balance": "1000", "salary": "0", "employer_rate": "0"}));
    assert_eq!(amount(&out, "balance_at_retirement"), "0.00");
    assert_eq!(amount(&out, "fees_and_premiums"), "1049.46");
}

#[test]
fn refuses_what_it_does_not_compute() {
    for (changes, want) in [
        (json!({"age": 60, "retirement_age": 59}), "super_projection.retirement_not_after_age"),
        (json!({"age": 70, "retirement_age": 66}), "super_projection.retirement_not_after_age"),
        (json!({"salary_sacrifice": "22900.01"}), "super_projection.cap_exceeded"),
        (json!({"after_tax_contributions": "130000.01"}), "super_projection.cap_exceeded"),
        (json!({"retirement_age": 76}), "super_projection.input_invalid"),
        (json!({"age": 75}), "super_projection.input_invalid"),
        (json!({"age": "30"}), "super_projection.input_invalid"),
        (json!({"employer_rate": "0.30001"}), "super_projection.input_invalid"),
        (json!({"return_rate": "0.21"}), "super_projection.input_invalid"),
        (json!({"wage_growth": 0.03}), "super_projection.input_invalid"),
        (json!({"discount_rate": "-0.01"}), "super_projection.input_invalid"),
        (json!({"salary": "80000.50"}), "input.amount_not_whole_dollars"),
        (json!({"year": "2025-26"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"spouse": true}), "input.field_unknown"),
    ] {
        assert_eq!(code(changes.clone()), want, "{changes}");
    }
}

/// The caps are tested on 2026-27 alone: $32,500 of concessional and
/// $130,000 of after-tax contributions are within them.
#[test]
fn contributions_at_the_caps_are_projected() {
    assert!(api::evaluate("super-projection", &with(json!({"salary_sacrifice": "22900"}))).is_ok());
    assert!(
        api::evaluate("super-projection", &with(json!({"after_tax_contributions": "130000"}))).is_ok()
    );
}
