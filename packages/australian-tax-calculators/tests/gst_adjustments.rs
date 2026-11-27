// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! GST adjustments: a change in creditable purpose (GST Act Division 129),
//! annual apportionment (Division 131) and the financial acquisitions
//! threshold (Division 189), read on 22 November 2026, against the examples of
//! GSTR 2000/24, the Act and GSTR 2003/9 and each boundary
//! (docs/calculation-evidence.md#gst-creditable-purpose-change).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const YEARS: [&str; 7] = [
    "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
];

fn with(base: Value, overrides: Value) -> Value {
    let mut input = base;
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
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

fn amounts<const N: usize>(result: &Value, keys: [&str; N]) -> [String; N] {
    keys.map(|key| result["amounts"][key].as_str().unwrap().to_string())
}

const CPC: &str = "gst-creditable-purpose-change";
const CPC_KEYS: [&str; 2] = ["increasing_adjustment", "decreasing_adjustment"];

/// GSTR 2000/24's Bill, 20 years on: a computer bought for $3,300 in March
/// 2021, attributable to the quarter to 31 March 2021, 90% intended.
fn bill(overrides: Value) -> Value {
    with(
        json!({
            "gst_exclusive_value": "3000.00", "relates_to_business_finance": false,
            "full_input_tax_credit": "300.00", "intended_or_former_application": "0.9",
            "actual_application": "0.6", "application_basis": "Hours logged",
            "attributed_period_end": "2021-03-31", "tax_period_months": 3,
            "year": "2021-22", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Paras 57 to 58 and 71: 60% actual by 30 June 2022, the first adjustment
/// period, so $300 x (90% - 60%) = $90.
#[test]
fn bills_increasing_adjustment() {
    let r = run(CPC, &bill(json!({})));
    assert_eq!(amounts(&r, CPC_KEYS), ["90.00", "0.00"]);
    assert_eq!(r["adjustment_period"]["number"], 1);
    assert_eq!(r["adjustment_period"]["of"], 2);
    assert_eq!(r["rates"]["value_threshold"], "1000");
    assert_eq!(r["application_basis"], "Hours logged");
    assert_eq!(r["no_adjustment_reason"], Value::Null);
}

/// Para 59: the second period compares 70% with the former 60%: a decrease of
/// $300 x 10% = $30 by the s 129-75 formula. A third period is past the 2 a
/// value of $5,000 or less allows (s 129-20(3)(a)).
#[test]
fn bills_second_and_third_periods() {
    let second = json!({"intended_or_former_application": "0.6", "actual_application": "0.7",
                        "year": "2022-23"});
    let r = run(CPC, &bill(second));
    assert_eq!(amounts(&r, CPC_KEYS), ["0.00", "30.00"]);
    assert_eq!(r["adjustment_period"]["number"], 2);
    let r = run(CPC, &bill(json!({"year": "2023-24"})));
    assert_eq!(amounts(&r, CPC_KEYS), ["0.00", "0.00"]);
    assert_eq!(r["adjustment_period"]["is_adjustment_period"], false);
    assert!(
        r["no_adjustment_reason"]
            .as_str()
            .unwrap()
            .contains("s 129-20(2) and (3)")
    );
}

/// The tax period ending 30 June 2021 starts less than 12 months after
/// 31 March 2021, so it is not an adjustment period (s 129-20(1)(a)).
#[test]
fn a_period_within_12_months_is_not_an_adjustment_period() {
    let r = run(CPC, &bill(json!({"year": "2020-21"})));
    assert_eq!(amounts(&r, CPC_KEYS), ["0.00", "0.00"]);
    assert!(
        r["no_adjustment_reason"]
            .as_str()
            .unwrap()
            .contains("s 129-20(1)(a)")
    );
}

/// Paras 64, 66 and 72: the XYZ Credit Union's reduced credit acquisition,
/// 75% intended and 85% actual: $10,000 x 10% = $1,000 decreasing.
#[test]
fn the_credit_unions_decreasing_adjustment() {
    let r = run(
        CPC,
        &bill(
            json!({"gst_exclusive_value": "100000.00", "relates_to_business_finance": true,
                     "full_input_tax_credit": "10000.00", "intended_or_former_application": "0.75",
                     "actual_application": "0.85"}),
        ),
    );
    assert_eq!(amounts(&r, CPC_KEYS), ["0.00", "1000.00"]);
    assert_eq!(r["adjustment_period"]["of"], 5);
    assert_eq!(r["rates"]["value_threshold"], "10000");
}

/// Paras 75 to 78: Elizabeth's full input tax credit after a $4,400 Division
/// 19 and a $2,200 Division 21 change is $26,400 / 11 = $2,400; 90% to 50%
/// gives $960.
#[test]
fn elizabeths_adjustment_after_earlier_adjustments() {
    let r = run(
        CPC,
        &bill(
            json!({"gst_exclusive_value": "30000.00", "full_input_tax_credit": "2400.00",
                     "actual_application": "0.5"}),
        ),
    );
    assert_eq!(amounts(&r, CPC_KEYS), ["960.00", "0.00"]);
}

/// Paras 30, 31 and 36: Farmco's $100,000 machinery, attributable to the
/// quarter to 31 March, has 5 adjustment periods from the next year's June
/// quarter; the sixth year has none.
#[test]
fn farmcos_five_adjustment_periods() {
    let farm = |year: &str| {
        bill(
            json!({"gst_exclusive_value": "100000.00", "full_input_tax_credit": "10000.00",
                    "actual_application": "0.8", "year": year}),
        )
    };
    for (n, year) in YEARS[1..6].iter().enumerate() {
        let r = run(CPC, &farm(year));
        assert_eq!(r["adjustment_period"]["number"], n as i64 + 1, "{year}");
        assert_eq!(amounts(&r, CPC_KEYS), ["1000.00", "0.00"], "{year}");
    }
    let r = run(CPC, &farm("2026-27"));
    assert_eq!(r["adjustment_period"]["is_adjustment_period"], false);
    assert_eq!(
        r["adjustment_period"]["first"],
        "the tax period ending 30 June 2022"
    );
}

/// Para 42: Isabelle's $10,000 acquisition, attributable to the quarter to
/// 30 September (2020 here), destroyed on 15 March (2022): the June 2022
/// quarter is the last adjustment period.
#[test]
fn isabelles_destroyed_acquisition() {
    let isabelle = |year: &str| {
        bill(
            json!({"gst_exclusive_value": "10000.00", "full_input_tax_credit": "1000.00",
                    "attributed_period_end": "2020-09-30", "ended_on": "2022-03-15",
                    "year": year}),
        )
    };
    let r = run(CPC, &isabelle("2021-22"));
    assert_eq!(r["adjustment_period"]["number"], 1);
    assert_eq!(amounts(&r, CPC_KEYS), ["300.00", "0.00"]);
    let r = run(CPC, &isabelle("2022-23"));
    assert_eq!(r["adjustment_period"]["is_adjustment_period"], false);
    assert!(
        r["no_adjustment_reason"]
            .as_str()
            .unwrap()
            .contains("s 129-25(1)")
    );
    let in_june = isabelle("2021-22");
    let in_june = with(in_june, json!({"ended_on": "2022-04-01"}));
    assert_eq!(
        code(CPC, &in_june),
        "gst_creditable_purpose_change.last_period_unsettled"
    );
    // Destroyed on 1 July 2021, the next 30 June tax period (the June 2022
    // quarter) ends before the first one s 129-20(1) allows (the June 2023
    // quarter): whether s 129-25(1) makes it one is refused; the year before
    // is too early, and the years after are past the last.
    let early = |year: &str| {
        bill(
            json!({"attributed_period_end": "2021-06-30", "ended_on": "2021-07-01",
                    "year": year}),
        )
    };
    assert_eq!(
        code(CPC, &early("2021-22")),
        "gst_creditable_purpose_change.last_period_unsettled"
    );
    let r = run(CPC, &early("2020-21"));
    assert_eq!(r["adjustment_period"]["is_adjustment_period"], false);
    for year in ["2022-23", "2023-24"] {
        let r = run(CPC, &early(year));
        assert!(
            r["no_adjustment_reason"]
                .as_str()
                .unwrap()
                .contains("s 129-25(1)"),
            "{year}"
        );
    }
}

/// s 129-25(2): a Division 132 disposal on 15 March 2023 ends the periods at
/// the one to 30 June 2022; one before the first period leaves none.
#[test]
fn a_division_132_disposal_ends_the_periods_before_it() {
    let sold = |year: &str, on: &str| {
        bill(
            json!({"gst_exclusive_value": "10000.00", "attributed_period_end": "2020-09-30",
                    "ended_on": on, "division_132_disposal": true, "year": year}),
        )
    };
    let r = run(CPC, &sold("2021-22", "2023-03-15"));
    assert_eq!(r["adjustment_period"]["number"], 1);
    let r = run(CPC, &sold("2022-23", "2023-03-15"));
    assert!(
        r["no_adjustment_reason"]
            .as_str()
            .unwrap()
            .contains("s 129-25(2)")
    );
    let r = run(CPC, &sold("2021-22", "2022-06-30"));
    assert!(
        r["no_adjustment_reason"]
            .as_str()
            .unwrap()
            .contains("s 129-25(2)")
    );
    let without = bill(json!({"division_132_disposal": true}));
    assert_eq!(code(CPC, &without), "input.missing_field");
}

/// Para 19 and s 129-10: Sophie's $400 desk has no adjustment; the value
/// thresholds are more than $1,000 and, for business finance, $10,000.
#[test]
fn the_value_thresholds() {
    for (value, finance, arises) in [
        ("400.00", false, false),
        ("1000.00", false, false),
        ("1000.01", false, true),
        ("10000.00", true, false),
        ("10000.01", true, true),
    ] {
        let r = run(
            CPC,
            &bill(json!({"gst_exclusive_value": value, "relates_to_business_finance": finance})),
        );
        let increase = &r["amounts"]["increasing_adjustment"];
        assert_eq!(increase != "0.00", arises, "{value} {finance}");
    }
}

/// s 129-20(2) and (3): the number of adjustment periods at each boundary.
#[test]
fn the_adjustment_period_counts() {
    for (value, finance, count) in [
        ("5000.00", false, 2),
        ("5000.01", false, 5),
        ("499999.99", false, 5),
        ("500000.00", false, 10),
        ("50000.00", true, 1),
        ("50000.01", true, 5),
        ("499999.99", true, 5),
        ("500000.00", true, 10),
    ] {
        let r = run(
            CPC,
            &bill(json!({"gst_exclusive_value": value, "relates_to_business_finance": finance})),
        );
        assert_eq!(r["adjustment_period"]["of"], count, "{value} {finance}");
    }
}

/// Monthly and annual tax periods: the June month starts 1 June, the year
/// 1 July; each must start 12 months after the attributed period ends.
#[test]
fn monthly_and_annual_tax_periods() {
    let monthly = bill(
        json!({"tax_period_months": 1, "attributed_period_end": "2021-05-31",
                              "year": "2021-22"}),
    );
    assert_eq!(run(CPC, &monthly)["adjustment_period"]["number"], 1);
    let late = with(monthly, json!({"attributed_period_end": "2021-06-30"}));
    let r = run(CPC, &late);
    assert_eq!(r["adjustment_period"]["is_adjustment_period"], false);
    assert_eq!(
        r["adjustment_period"]["first"],
        "the tax period ending 30 June 2023"
    );
    let annual = bill(
        json!({"tax_period_months": 12, "attributed_period_end": "2021-06-30",
                             "year": "2022-23"}),
    );
    assert_eq!(run(CPC, &annual)["adjustment_period"]["number"], 1);
    let early = with(annual, json!({"year": "2021-22"}));
    assert_eq!(
        run(CPC, &early)["adjustment_period"]["is_adjustment_period"],
        false
    );
}

/// The refusals: an unstated or overlong basis, a period end that is not a
/// month's last day, another tax period length, and fractions outside 0 to 1.
#[test]
fn creditable_purpose_inputs_are_checked() {
    let long = "x".repeat(201);
    for (overrides, expected) in [
        (
            json!({"application_basis": " "}),
            "gst_creditable_purpose_change.basis_missing",
        ),
        (
            json!({"application_basis": long}),
            "gst_creditable_purpose_change.basis_missing",
        ),
        (
            json!({"attributed_period_end": "2021-03-30"}),
            "gst_creditable_purpose_change.period_end_invalid",
        ),
        (
            json!({"tax_period_months": 2}),
            "gst_creditable_purpose_change.tax_period_unsupported",
        ),
        (
            json!({"tax_period_months": "3"}),
            "gst_creditable_purpose_change.tax_period_unsupported",
        ),
        (
            json!({"actual_application": "1.01"}),
            "input.fraction_invalid",
        ),
        (
            json!({"intended_or_former_application": "-0.1"}),
            "input.fraction_invalid",
        ),
        (json!({"ended_on": "2022-02-30"}), "input.date_invalid"),
        (
            json!({"relates_to_business_finance": "no"}),
            "input.flag_invalid",
        ),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"year": "2019-20"}), "period.unsupported"),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"surplus": 1}), "input.field_unknown"),
    ] {
        assert_eq!(code(CPC, &bill(overrides.clone())), expected, "{overrides}");
    }
}

const AA: &str = "gst-annual-apportionment";
const AA_KEYS: [&str; 3] = [
    "credit_without_annual_apportionment",
    "annual_apportionment_credit",
    "increasing_adjustment",
];

/// The GST Act's s 131-55 example: a $1,100 acquisition at 10% creditable
/// purpose by an eligible small business entity that has elected.
fn apportion(overrides: Value) -> Value {
    with(
        json!({
            "small_business_entity": true, "carries_on_business": true,
            "instalment_election": false, "annual_tax_period_election": false,
            "election_made": true, "gst_on_acquisition": "100.00",
            "creditable_purpose": "0.1", "car": false, "reduced_credit_acquisition": false,
            "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// s 131-40(1): the credit is the $100 GST; s 11-30 gives $10; the
/// adjustment is $90. After the example's $10 Division 19 and $60 Division 21
/// adjustments the GST is $50, (2)(a) $50 and (2)(b) $5: $45.
#[test]
fn the_acts_annual_apportionment_example() {
    let r = run(AA, &apportion(json!({})));
    assert_eq!(amounts(&r, AA_KEYS), ["10.00", "100.00", "90.00"]);
    assert_eq!(r["eligible"], true);
    assert_eq!(r["applies"], true);
    assert_eq!(r["rates"]["turnover_threshold"], "2000000");
    let r = run(AA, &apportion(json!({"gst_on_acquisition": "50.00"})));
    assert_eq!(amounts(&r, AA_KEYS), ["5.00", "50.00", "45.00"]);
}

/// s 131-40(2): input taxed use and part consideration: $1,000 x 80% x 50% =
/// $400 against $1,000 x 40% x 50% = $200.
#[test]
fn input_taxed_purpose_and_part_consideration() {
    let r = run(
        AA,
        &apportion(
            json!({"gst_on_acquisition": "1000.00", "creditable_purpose": "0.4",
                          "non_input_taxed_extent": "0.8", "consideration_extent": "0.5"}),
        ),
    );
    assert_eq!(amounts(&r, AA_KEYS), ["200.00", "400.00", "200.00"]);
    let whole_purpose =
        apportion(json!({"creditable_purpose": "1", "consideration_extent": "0.5"}));
    assert_eq!(
        amounts(&run(AA, &whole_purpose), AA_KEYS),
        ["50.00", "50.00", "0.00"]
    );
}

/// s 131-5(1): each condition failing in turn, without an election, leaves
/// the s 11-30 credit and no adjustment. With an election in effect, a later
/// instalment or annual tax period election does not end it (s 131-20(1)),
/// while the entity test failing may, so that alone is refused. An eligible
/// entity that has not elected also has the s 11-30 credit.
#[test]
fn each_eligibility_condition() {
    for (overrides, failing) in [
        (
            json!({"instalment_election": true}),
            "no_instalment_election",
        ),
        (
            json!({"annual_tax_period_election": true}),
            "no_annual_tax_period_election",
        ),
        (json!({"small_business_entity": false}), "entity_eligible"),
    ] {
        if failing == "entity_eligible" {
            assert_eq!(
                code(AA, &apportion(overrides.clone())),
                "gst_annual_apportionment.election_status_unsettled",
                "{overrides}"
            );
        } else {
            let r = run(AA, &apportion(overrides.clone()));
            assert_eq!(r["eligible"], false, "{overrides}");
            assert_eq!(r["applies"], true, "{overrides}");
            assert_eq!(amounts(&r, AA_KEYS), ["10.00", "100.00", "90.00"]);
        }
        let r = run(
            AA,
            &apportion(with(overrides, json!({"election_made": false}))),
        );
        assert_eq!(r["eligible"], false);
        assert_eq!(r["failing_conditions"], json!([failing]));
        assert_eq!(amounts(&r, AA_KEYS), ["10.00", "10.00", "0.00"]);
    }
    let r = run(AA, &apportion(json!({"election_made": false})));
    assert_eq!(r["eligible"], true);
    assert_eq!(r["applies"], false);
    assert_eq!(amounts(&r, AA_KEYS), ["10.00", "10.00", "0.00"]);
}

/// s 131-5(1)(a)(ii) and (2): an entity carrying on no business with a GST
/// turnover of $2 million is eligible; one cent more is not.
#[test]
fn the_non_business_turnover_threshold() {
    let base = json!({"small_business_entity": false, "carries_on_business": false,
                      "election_made": false});
    for (turnover, eligible) in [("2000000.00", true), ("2000000.01", false)] {
        let input = apportion(with(base.clone(), json!({"gst_turnover": turnover})));
        assert_eq!(run(AA, &input)["eligible"], eligible, "{turnover}");
    }
    assert_eq!(code(AA, &apportion(base)), "input.missing_field");
    let extra = apportion(json!({"gst_turnover": "1.00"}));
    assert_eq!(code(AA, &extra), "input.field_unknown");
}

#[test]
fn annual_apportionment_refusals() {
    for (overrides, expected) in [
        (json!({"car": true}), "gst_annual_apportionment.car"),
        (
            json!({"reduced_credit_acquisition": true}),
            "gst_annual_apportionment.reduced_credit_acquisition",
        ),
        (
            json!({"creditable_purpose": "0"}),
            "gst_annual_apportionment.not_partly_creditable",
        ),
        (
            json!({"creditable_purpose": "1"}),
            "gst_annual_apportionment.not_partly_creditable",
        ),
        (
            json!({"consideration_extent": "0"}),
            "gst_annual_apportionment.not_partly_creditable",
        ),
        (
            json!({"non_input_taxed_extent": "0.05"}),
            "gst_annual_apportionment.fact_conflict",
        ),
        (
            json!({"creditable_purpose": "1", "non_input_taxed_extent": "0.8"}),
            "gst_annual_apportionment.fact_conflict",
        ),
        (
            json!({"carries_on_business": false}),
            "gst_annual_apportionment.fact_conflict",
        ),
        (
            json!({"creditable_purpose": "1.5"}),
            "input.fraction_invalid",
        ),
        (json!({"gst_on_acquisition": "-1"}), "input.amount_invalid"),
        (json!({"year": "2027-28"}), "period.unsupported"),
    ] {
        assert_eq!(
            code(AA, &apportion(overrides.clone())),
            expected,
            "{overrides}"
        );
    }
}

const FAT: &str = "gst-financial-acquisitions-threshold";
const FAT_KEYS: [&str; 2] = ["current_ten_percent", "future_ten_percent"];

/// GSTR 2003/9's Cedar, Examples 1A and 2A.
fn cedar(overrides: Value) -> Value {
    with(
        json!({
            "current_financial_credits": "104550.00", "current_total_credits": "1100000.00",
            "future_financial_credits": "131000.00", "future_total_credits": "1220000.00",
            "gst_group": false, "year": "2025-26", "scope_confirmed": true,
        }),
        overrides,
    )
}

/// Cedar: $104,550 is neither over $150,000 nor over $110,000; $131,000 is
/// over 10% of $1,220,000 ($122,000), so it exceeds the threshold.
#[test]
fn cedars_current_and_future_tests() {
    let r = run(FAT, &cedar(json!({})));
    assert_eq!(amounts(&r, FAT_KEYS), ["110000.00", "122000.00"]);
    assert_eq!(r["exceeds_threshold"], true);
    assert_eq!(
        r["failing_conditions"],
        json!(["future_within_ten_percent"])
    );
    assert_eq!(r["rates"]["amount_limit"], "150000");
}

/// Examples 13 and 14: the ABC group's $3,600 of $69,000, then $10,200 of
/// $99,000, over $9,900.
#[test]
fn the_abc_groups_tests() {
    let r = run(
        FAT,
        &cedar(
            json!({"current_financial_credits": "3600.00", "current_total_credits": "69000.00",
                      "future_financial_credits": "10200.00", "future_total_credits": "99000.00",
                      "gst_group": true}),
        ),
    );
    assert_eq!(r["exceeds_threshold"], true);
    assert_eq!(
        r["failing_conditions"],
        json!(["future_within_ten_percent"])
    );
    assert!(
        r["conditions"][0]["authority"]
            .as_str()
            .unwrap()
            .contains("s 189-5(2)(a)")
    );
}

/// The limbs are 'exceed $150,000' and 'more than 10%': equal is within.
#[test]
fn the_threshold_boundaries() {
    let big = "10000000.00";
    for (financial, total, exceeds) in [
        ("150000.00", big, false),
        ("150000.01", big, true),
        ("100000.00", "1000000.00", false),
        ("100000.01", "1000000.00", true),
    ] {
        let input = cedar(
            json!({"current_financial_credits": financial, "current_total_credits": total,
                                 "future_financial_credits": "0.00", "future_total_credits": big}),
        );
        assert_eq!(
            run(FAT, &input)["exceeds_threshold"],
            exceeds,
            "{financial} {total}"
        );
    }
    let within = cedar(json!({"future_financial_credits": "100.00"}));
    assert_eq!(run(FAT, &within)["exceeds_threshold"], false);
}

#[test]
fn threshold_refusals() {
    let over = cedar(json!({"current_financial_credits": "1100000.01"}));
    assert_eq!(
        code(FAT, &over),
        "gst_financial_acquisitions_threshold.fact_conflict"
    );
    assert_eq!(
        code(FAT, &cedar(json!({"gst_group": 1}))),
        "input.flag_invalid"
    );
    assert_eq!(
        code(FAT, &cedar(json!({"future_total_credits": "1.001"}))),
        "input.amount_invalid"
    );
}

/// Each command takes every year from 2020-21 to 2026-27 alike.
#[test]
fn the_supported_years() {
    for year in YEARS {
        let input = cedar(json!({"year": year}));
        assert_eq!(run(FAT, &input)["exceeds_threshold"], true, "{year}");
        let input = apportion(json!({"year": year}));
        assert_eq!(
            run(AA, &input)["amounts"]["increasing_adjustment"],
            "90.00",
            "{year}"
        );
    }
}

/// The catalogue's example of each command runs.
#[test]
fn the_catalogue_examples_run() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    for (command, kind) in [
        (CPC, "gst_creditable_purpose_change"),
        (AA, "gst_annual_apportionment"),
        (FAT, "gst_financial_acquisitions_threshold"),
    ] {
        let mut facts = catalogue[kind]["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        facts["scope_confirmed"] = json!(true);
        assert!(api::evaluate(command, &facts).is_ok(), "{command}");
    }
}

/// s 129-20(1): with annual tax periods and an acquisition attributable to a
/// period ending 30 September 2021, the year to 30 June 2023 starts within 12
/// months, so the first adjustment period is the year to 30 June 2024.
#[test]
fn an_annual_period_starting_within_12_months_moves_the_first_a_year() {
    let annual = bill(
        json!({"tax_period_months": 12, "attributed_period_end": "2021-09-30",
                             "year": "2022-23"}),
    );
    let r = run(CPC, &annual);
    assert_eq!(r["adjustment_period"]["is_adjustment_period"], false);
    assert_eq!(
        r["adjustment_period"]["first"],
        "the tax period ending 30 June 2024"
    );
    let next = with(annual, json!({"year": "2023-24"}));
    assert_eq!(run(CPC, &next)["adjustment_period"]["number"], 1);
}

/// ss 129-70 and 129-75: an actual application equal to the intended one
/// gives neither an increasing nor a decreasing adjustment.
#[test]
fn an_unchanged_application_gives_no_adjustment() {
    let r = run(CPC, &bill(json!({"actual_application": "0.9"})));
    assert_eq!(amounts(&r, CPC_KEYS), ["0.00", "0.00"]);
    assert_eq!(r["adjustment_period"]["number"], 1);
    assert_eq!(r["no_adjustment_reason"], Value::Null);
}
