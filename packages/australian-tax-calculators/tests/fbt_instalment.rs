// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `fbt-instalment` against the ATO's 'Fringe benefits tax (FBT) instalment'
//! page (last updated 1 May 2025) and the variation examples in chapter 3 of
//! 'Fringe benefits tax - a guide for employers', the relevant fractions of
//! FBTAA s 109, the s 111(1) formula and the s 112A credit, and its
//! refusals (docs/calculation-evidence.md#fbt-instalment).
use australian_tax_calculators::api::{self, Refusal};
use serde_json::{Value, json};

fn input(fields: Value) -> Value {
    let mut out = json!({"year_ended": 2027, "scope_confirmed": true});
    for (k, v) in fields.as_object().unwrap() {
        out[k] = v.clone();
    }
    out
}

fn labels(fields: Value) -> Value {
    api::evaluate_coded("fbt-instalment", &input(fields)).unwrap()["activity_statement_labels"]
        .clone()
}

fn refused(fields: Value) -> Refusal {
    api::evaluate_coded("fbt-instalment", &input(fields)).unwrap_err()
}

fn varied(quarter: i64, estimate: &str, previous: &str) -> Value {
    labels(json!({"quarter": quarter, "estimated_tax": estimate,
                  "previous_instalments": previous, "reason_code": 31}))
}

#[test]
fn matches_the_ato_varying_example() {
    // December quarter: $28,000 x 75% - ($20,000 - 0) = $1,000, code 31.
    let result = labels(json!({"quarter": 3, "estimated_tax": "28000.00",
                               "previous_instalments": "20000.00",
                               "previous_credits": "0.00", "reason_code": 31}));
    assert_eq!(
        result,
        json!({"F2": "28000.00", "F3": "1000.00", "F4": 31, "6A": "1000.00", "6B": "0.00"})
    );
}

#[test]
fn matches_the_employer_guide_examples() {
    // Notional tax of $20,000; an estimate of $16,000 before the first
    // instalment gives $4,000; after a first instalment of $5,000, half of
    // $16,000 less $5,000 is $3,000; then three-quarters less $8,000, $4,000.
    assert_eq!(varied(1, "16000.00", "0.00")["F3"], "4000.00");
    assert_eq!(varied(2, "16000.00", "5000.00")["F3"], "3000.00");
    assert_eq!(varied(3, "16000.00", "8000.00")["F3"], "4000.00");
    // An estimate of $2,400 under the threshold still gives a quarter, $600;
    // the first quarter needs no previous instalments.
    let first = labels(json!({"quarter": 1, "estimated_tax": "2400.00", "reason_code": 30}));
    assert_eq!(first["F3"], "600.00");
    assert_eq!(first["6A"], "600.00");
}

#[test]
fn each_quarter_takes_its_relevant_fraction() {
    for (quarter, expected) in [
        (1, "2500.00"),
        (2, "5000.00"),
        (3, "7500.00"),
        (4, "10000.00"),
    ] {
        let result = api::evaluate_coded(
            "fbt-instalment",
            &input(json!({"quarter": quarter, "estimated_tax": "10000.00",
                          "previous_instalments": "0.00", "reason_code": 22})),
        )
        .unwrap();
        assert_eq!(result["activity_statement_labels"]["F3"], expected);
        assert_eq!(
            result["rates"]["relevant_fraction"],
            ["0.25", "0.50", "0.75", "1.00"][usize::try_from(quarter - 1).unwrap()]
        );
    }
}

#[test]
fn a_negative_amount_is_a_credit_at_6b() {
    // s 112A: half of $8,000 less $5,000 is -$1,000: nil at F3 and 6A, $1,000 at 6B.
    let result = varied(2, "8000.00", "5000.00");
    assert_eq!(result["F3"], "0.00");
    assert_eq!(result["6A"], "0.00");
    assert_eq!(result["6B"], "1000.00");
    // Exactly paid: nil and no credit.
    let even = varied(2, "10000.00", "5000.00");
    assert_eq!(
        (even["F3"].clone(), even["6B"].clone()),
        (json!("0.00"), json!("0.00"))
    );
    // Credits claimed earlier add back: 75% of $8,000 - ($5,000 - $1,000).
    let after = labels(json!({"quarter": 3, "estimated_tax": "8000.00",
                              "previous_instalments": "5000.00",
                              "previous_credits": "1000.00", "reason_code": 31}));
    assert_eq!(after["F3"], "2000.00");
    // Credits equal to the instalments before them: 75% of $1,000 - nil.
    let all_back = labels(json!({"quarter": 3, "estimated_tax": "1000.00",
                                 "previous_instalments": "500.00",
                                 "previous_credits": "500.00", "reason_code": 30}));
    assert_eq!(all_back["F3"], "750.00");
}

#[test]
fn the_share_is_rounded_to_the_cent_half_up() {
    assert_eq!(varied(1, "1000.01", "0.00")["F3"], "250.00");
    assert_eq!(varied(1, "1000.02", "0.00")["F3"], "250.01");
    assert_eq!(varied(3, "1000.02", "0.00")["F3"], "750.02");
}

#[test]
fn copies_f1_when_not_varying() {
    let result = labels(json!({"quarter": 2, "ato_instalment": "5000.00"}));
    assert_eq!(result, json!({"F1": "5000.00", "6A": "5000.00"}));
}

#[test]
fn refuses_what_it_does_not_take() {
    let estimate = json!({"quarter": 2, "estimated_tax": "1000.00",
                          "previous_instalments": "0.00", "reason_code": 30});
    let with = |key: &str, value: Value| {
        let mut out = estimate.clone();
        if value.is_null() {
            out.as_object_mut().unwrap().remove(key);
        } else {
            out[key] = value;
        }
        out
    };
    let cases = [
        (with("quarter", json!(5)), "fbt_instalment.quarter_invalid"),
        (with("quarter", json!(0)), "fbt_instalment.quarter_invalid"),
        (
            with("quarter", json!("1")),
            "fbt_instalment.quarter_invalid",
        ),
        (
            with("reason_code", json!(33)),
            "fbt_instalment.variation_invalid",
        ),
        (
            with("reason_code", Value::Null),
            "fbt_instalment.variation_invalid",
        ),
        (
            with("previous_instalments", Value::Null),
            "fbt_instalment.variation_invalid",
        ),
        (
            with("estimated_tax", json!("-1.00")),
            "input.amount_invalid",
        ),
        (
            with("previous_credits", json!("1.001")),
            "input.amount_invalid",
        ),
        (with("year_ended", json!(2025)), "period.unsupported"),
        (with("year_ended", json!(2028)), "period.unsupported"),
        (with("scope_confirmed", json!(false)), "scope.not_confirmed"),
        (with("not_an_input", json!(1)), "input.field_unknown"),
        (
            json!({"quarter": 1}),
            "fbt_instalment.ato_instalment_missing",
        ),
        (
            json!({"quarter": 1, "ato_instalment": "100.00", "reason_code": 30}),
            "fbt_instalment.variation_invalid",
        ),
        (
            json!({"quarter": 2, "ato_instalment": "100.00", "previous_instalments": "100.00"}),
            "fbt_instalment.variation_invalid",
        ),
        (
            json!({"quarter": 1, "ato_instalment": "-5.00"}),
            "input.amount_invalid",
        ),
        (
            json!({"quarter": 1, "estimated_tax": "100.00", "previous_instalments": "1.00",
                   "reason_code": 30}),
            "fbt_instalment.variation_invalid",
        ),
        (
            json!({"quarter": 1, "estimated_tax": "100.00", "previous_credits": "1.00",
                   "reason_code": 30}),
            "fbt_instalment.variation_invalid",
        ),
        // s 112A: a credit is at most the instalments paid before it.
        (
            json!({"quarter": 3, "estimated_tax": "1000.00", "previous_instalments": "500.00",
                   "previous_credits": "500.01", "reason_code": 30}),
            "fbt_instalment.variation_invalid",
        ),
    ];
    for (fields, code) in cases {
        assert_eq!(refused(fields.clone()).code, code, "{fields}");
    }
}

#[test]
fn takes_both_fbt_years() {
    for year in [2026, 2027] {
        let mut fields = input(json!({"quarter": 1, "ato_instalment": "100.00"}));
        fields["year_ended"] = json!(year);
        let result = api::evaluate_coded("fbt-instalment", &fields).unwrap();
        assert_eq!(result["period"], format!("year ended 31 March {year}"));
    }
}
