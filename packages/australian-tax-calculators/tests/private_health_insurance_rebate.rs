// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The private health insurance rebate (Private Health Insurance Act 2007
//! ss 22-15 to 22-40 and 282-18; ITAA 1997 ss 61-205 and 61-210) on the ATO's
//! worked examples, every published percentage and threshold from 2020-21 to
//! 2026-27, each tier boundary and every refusal, all read on 22 November 2026
//! (docs/calculation-evidence.md#private-health-insurance-rebate), and its
//! place in `personal-tax-estimate`.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "private-health-insurance-rebate";

fn line(premiums: &str, received: &str, code: i64) -> Value {
    json!({"premiums_eligible": premiums, "rebate_received": received, "benefit_code": code})
}

fn input(year: &str, income: &str, lines: Value) -> Value {
    json!({"year": year, "scope_confirmed": true,
        "income_for_surcharge_purposes": income, "lines": lines})
}

fn run(facts: &Value) -> Value {
    api::evaluate(COMMAND, facts).unwrap()
}

fn code(facts: &Value) -> &'static str {
    api::evaluate_coded(COMMAND, facts).unwrap_err().code
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

fn field<'a>(result: &'a Value, n: usize, key: &str) -> &'a str {
    result["lines"][n][key].as_str().unwrap()
}

/// The ATO's 2025-26 statements for one person: (premiums eligible, rebate
/// received) before 1 April with `first`, and from 1 April with `first + 1`.
fn year_lines(first: i64, before: (&str, &str), after: (&str, &str)) -> Value {
    json!([
        line(before.0, before.1, first),
        line(after.0, after.1, first + 1)
    ])
}

/// 'Income thresholds and rates for the private health insurance rebate'
/// (22 June 2026), Narelle: single, 45, $900 of premiums before 1 April with a
/// $146 reduction and $300 after with $48, income $83,000: 24.288% and
/// 24.118% give $218.59 and $72.35, so a refundable offset of $96.94.
#[test]
fn narelle_gets_the_rebate_she_did_not_receive_as_an_offset() {
    let lines = year_lines(30, ("900", "146"), ("300", "48"));
    let r = run(&input("2025-26", "83000", lines));
    assert_eq!(r["tier"], "base");
    assert_eq!(r["thresholds"], "singles");
    assert_eq!(field(&r, 0, "rebate_percentage"), "24.288");
    assert_eq!(field(&r, 1, "rebate_percentage"), "24.118");
    assert_eq!(field(&r, 0, "share_of_phii_benefit"), "218.59");
    assert_eq!(field(&r, 1, "share_of_phii_benefit"), "72.35");
    assert_eq!(amount(&r, "private_health_insurance_offset"), "96.94");
    assert_eq!(amount(&r, "excess_private_health_reduction"), "0.00");
    assert_eq!(amount(&r, "rebate_received"), "194.00");
}

/// Peta: $1,000 of premiums without her $200 loading, paid $900 and $300 in
/// the 2 periods, so $750 and $250 eligible; income $109,000 is tier 1, so
/// 16.192% and 16.079% against reductions of $182 and $60: $80.36 to repay.
#[test]
fn peta_repays_the_excess_of_a_base_tier_reduction() {
    let lines = year_lines(30, ("750", "182"), ("250", "60"));
    let r = run(&input("2025-26", "109000", lines));
    assert_eq!(r["tier"], "tier_1");
    assert_eq!(field(&r, 1, "share_of_phii_benefit"), "40.20");
    assert_eq!(amount(&r, "excess_private_health_reduction"), "80.36");
    assert_eq!(amount(&r, "private_health_insurance_offset"), "0.00");
}

/// Zoe and Charlie, divorced in January 2026, each an adult on the policy, so
/// each has half: $750 with $182 received and $250 with $60. Zoe, $119,000,
/// tier 2: 8.095% and 8.038% leave $121.29 and $39.90, $161.19 in all ($20.095
/// rounds to $20.10 before the subtraction). Charlie, $164,200, tier 3: $242.
#[test]
fn zoe_and_charlie_repay_on_their_own_shares() {
    let lines = year_lines(30, ("750", "182"), ("250", "60"));
    let zoe = run(&input("2025-26", "119000", lines.clone()));
    assert_eq!(zoe["tier"], "tier_2");
    assert_eq!(field(&zoe, 0, "excess_private_health_reduction"), "121.29");
    assert_eq!(field(&zoe, 1, "excess_private_health_reduction"), "39.90");
    assert_eq!(amount(&zoe, "excess_private_health_reduction"), "161.19");
    let charlie = run(&input("2025-26", "164200", lines));
    assert_eq!(charlie["tier"], "tier_3");
    assert_eq!(field(&charlie, 0, "rebate_percentage"), "0");
    assert_eq!(
        amount(&charlie, "excess_private_health_reduction"),
        "242.00"
    );
}

/// Mike ($160,000) and Elle ($55,000), single, sharing a $1,000 policy with no
/// reduction: Mike gets nothing; Elle 24.288% and 24.118%. The example does
/// not split the premiums by period, so only the tiers and rates are checked.
#[test]
fn mike_and_elle_are_tested_on_their_own_incomes() {
    let lines = year_lines(30, ("250", "0"), ("250", "0"));
    let mike = run(&input("2025-26", "160000", lines.clone()));
    assert_eq!(mike["tier"], "tier_3");
    assert_eq!(amount(&mike, "private_health_insurance_offset"), "0.00");
    let elle = run(&input("2025-26", "55000", lines));
    assert_eq!(elle["tier"], "base");
    assert_eq!(field(&elle, 0, "rebate_percentage"), "24.288");
    assert_eq!(field(&elle, 1, "rebate_percentage"), "24.118");
}

/// 'Claiming the private health insurance rebate' (QC72945): Toby, 67, $1,800
/// and $600 of premiums with $510 and $169 received, income $102,000 (tier 1):
/// 20.240% and 20.098% give $364.32 and $120.59, so $194.09 to repay, which
/// the ATO states to the dollar as $194.
#[test]
fn toby_aged_67_repays_at_the_65_to_69_tier_1_rates() {
    let lines = year_lines(35, ("1800", "510"), ("600", "169"));
    let r = run(&input("2025-26", "102000", lines));
    assert_eq!(field(&r, 0, "age_band"), "65_to_69");
    assert_eq!(field(&r, 0, "share_of_phii_benefit"), "364.32");
    assert_eq!(field(&r, 1, "share_of_phii_benefit"), "120.59");
    assert_eq!(amount(&r, "excess_private_health_reduction"), "194.09");
}

/// Donna paid the year's $1,500 on 1 July 2025 with a tier 2 reduction of
/// $121, but her income was $85,000: 24.288% is $364.32, so $243.32 as an
/// offset. The ATO's page states $243.12, which $364.32 less $121 is not.
#[test]
fn donna_who_prepaid_gets_the_difference_as_an_offset() {
    let r = run(&input("2025-26", "85000", json!([line("1500", "121", 30)])));
    assert_eq!(field(&r, 0, "share_of_phii_benefit"), "364.32");
    assert_eq!(amount(&r, "private_health_insurance_offset"), "243.32");
}

/// An offset on one line and an excess on another are both shown: ITAA 1997
/// s 61-210 and PHI Act s 282-18 each work on a premium.
#[test]
fn an_offset_and_an_excess_on_different_lines_are_not_netted() {
    let lines = year_lines(30, ("900", "300"), ("300", "0"));
    let r = run(&input("2025-26", "83000", lines));
    assert_eq!(amount(&r, "excess_private_health_reduction"), "81.41");
    assert_eq!(amount(&r, "private_health_insurance_offset"), "72.35");
}

/// The percentages for each adjustment year from 1 April 2020 (the ATO's
/// tables for 2020-21 to 2026-27): under 65, 65 to 69, 70 or over; base,
/// tier 1, tier 2.
const PERCENTAGES: [(i64, [[&str; 3]; 3]); 7] = [
    (
        2020,
        [
            ["25.059", "16.706", "8.352"],
            ["29.236", "20.883", "12.529"],
            ["33.413", "25.059", "16.706"],
        ],
    ),
    (
        2021,
        [
            ["24.608", "16.405", "8.202"],
            ["28.710", "20.507", "12.303"],
            ["32.812", "24.608", "16.405"],
        ],
    ),
    (
        2022,
        [
            ["24.608", "16.405", "8.202"],
            ["28.710", "20.507", "12.303"],
            ["32.812", "24.608", "16.405"],
        ],
    ),
    (
        2023,
        [
            ["24.608", "16.405", "8.202"],
            ["28.710", "20.507", "12.303"],
            ["32.812", "24.608", "16.405"],
        ],
    ),
    (
        2024,
        [
            ["24.608", "16.405", "8.202"],
            ["28.710", "20.507", "12.303"],
            ["32.812", "24.608", "16.405"],
        ],
    ),
    (
        2025,
        [
            ["24.288", "16.192", "8.095"],
            ["28.337", "20.240", "12.143"],
            ["32.385", "24.288", "16.192"],
        ],
    ),
    (
        2026,
        [
            ["24.118", "16.079", "8.038"],
            ["28.139", "20.098", "12.058"],
            ["32.158", "24.118", "16.079"],
        ],
    ),
];

/// The singles tier 1 to 3 thresholds the ATO publishes for each year.
const THRESHOLDS: [(&str, [i64; 3]); 7] = [
    ("2020-21", [90_000, 105_000, 140_000]),
    ("2021-22", [90_000, 105_000, 140_000]),
    ("2022-23", [90_000, 105_000, 140_000]),
    ("2023-24", [93_000, 108_000, 144_000]),
    ("2024-25", [97_000, 113_000, 151_000]),
    ("2025-26", [101_000, 118_000, 158_000]),
    ("2026-27", [105_000, 123_000, 164_000]),
];

/// Every published percentage: each year, period, age band and tier, on
/// $1,000 of premiums, so the share is 10 times the percentage. Premiums paid
/// before 1 April take the percentages from 1 April of the year's first
/// calendar year, and later ones those of its second (s 22-15(5C) and (5D)).
#[test]
fn every_published_percentage_applies_to_its_period_band_and_tier() {
    let mut checked = 0;
    for (year, t) in THRESHOLDS {
        let first: i64 = year[..4].parse().unwrap();
        // An income in each tier: the tier 1 threshold itself is still base.
        let incomes = [t[0], t[1], t[2], t[2] + 1];
        for (tier, income) in incomes.iter().enumerate() {
            for (benefit, band, later) in [
                (30, 0, 0),
                (31, 0, 1),
                (35, 1, 0),
                (36, 1, 1),
                (40, 2, 0),
                (41, 2, 1),
            ] {
                let facts = input(
                    year,
                    &income.to_string(),
                    json!([line("1000", "0", benefit)]),
                );
                if year == "2026-27" && later == 1 {
                    assert_eq!(
                        code(&facts),
                        "private_health_insurance_rebate.rate_unpublished"
                    );
                    continue;
                }
                let r = run(&facts);
                let table = PERCENTAGES
                    .iter()
                    .find(|(s, _)| *s == first + later)
                    .unwrap()
                    .1;
                let want = if tier == 3 { "0" } else { table[band][tier] };
                assert_eq!(
                    field(&r, 0, "rebate_percentage"),
                    want,
                    "{year} {benefit} {income}"
                );
                let share = format!("{:.2}", want.parse::<f64>().unwrap() * 10.0);
                assert_eq!(
                    amount(&r, "private_health_insurance_offset"),
                    share,
                    "{year} {benefit} {income}"
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 7 * 4 * 6 - 4 * 3);
}

fn tier(year: &str, facts: Value) -> String {
    let mut all = input(year, "0", json!([line("100", "0", 30)]));
    for (k, v) in facts.as_object().unwrap() {
        all[k] = v.clone();
    }
    run(&all)["tier"].as_str().unwrap().to_string()
}

/// Each singles threshold, each year: income equal to it stays in the lower
/// tier, and $1 more is in the next (s 22-30(4): 'exceeds').
#[test]
fn each_singles_threshold_is_exceeded_only_above_it() {
    for (year, t) in THRESHOLDS {
        for (n, threshold) in t.iter().enumerate() {
            let at = json!({"income_for_surcharge_purposes": threshold.to_string()});
            let above = json!({"income_for_surcharge_purposes": (threshold + 1).to_string()});
            assert_eq!(tier(year, at), ["base", "tier_1", "tier_2"][n], "{year}");
            assert_eq!(
                tier(year, above),
                ["tier_1", "tier_2", "tier_3"][n],
                "{year}"
            );
        }
    }
}

/// A person married on 30 June is tested on the combined income against
/// double the thresholds (s 22-30(1) and (2), s 22-40(1) to (3)).
#[test]
fn a_couple_is_tested_on_combined_income_against_double_thresholds() {
    for (year, t) in THRESHOLDS {
        let own = 2 * t[0] - 60_000;
        let at = json!({"income_for_surcharge_purposes": own.to_string(),
            "spouse_income_for_surcharge_purposes": "60000"});
        let above = json!({"income_for_surcharge_purposes": (own + 1).to_string(),
            "spouse_income_for_surcharge_purposes": "60000"});
        assert_eq!(tier(year, at), "base", "{year}");
        assert_eq!(tier(year, above), "tier_1", "{year}");
    }
    let r = run(&json!({"year": "2025-26", "scope_confirmed": true,
        "income_for_surcharge_purposes": "150000", "spouse_income_for_surcharge_purposes": "60000",
        "lines": [line("100", "0", 30)]}));
    assert_eq!(r["thresholds"], "families");
    assert_eq!(amount(&r, "income_tested"), "210000.00");
    assert_eq!(r["rates"]["tier_1_threshold"], "202000");
}

/// A person who maintained a dependent person who is their child or dependent
/// sibling is on the family thresholds on their own income (s 22-30(1)(b)),
/// and each dependant who is a child after the first raises every family
/// threshold by $1,500 (s 22-40(4)): with 3, by $3,000.
#[test]
fn a_dependent_person_brings_family_thresholds_and_children_raise_them() {
    let family = |income: i64, children: i64| {
        json!({"income_for_surcharge_purposes": income.to_string(),
            "maintained_dependent_person": true, "dependent_children": children})
    };
    for (year, t) in THRESHOLDS {
        let top = 2 * t[2] + 3000;
        assert_eq!(tier(year, family(top, 3)), "tier_2", "{year}");
        assert_eq!(tier(year, family(top + 1, 3)), "tier_3", "{year}");
        assert_eq!(tier(year, family(2 * t[0], 1)), "base", "{year}");
        assert_eq!(tier(year, family(2 * t[0], 0)), "base", "{year}");
        assert_eq!(tier(year, family(2 * t[0] + 1, 0)), "tier_1", "{year}");
    }
}

/// A single parent maintaining a 26-year-old dependent student (a dependent
/// person under the PHI Act, though not a child dependant for s 22-40(4)), on
/// $150,000 in 2025-26: family thresholds, so base tier, and $1,000 of
/// premiums gives $242.88, not the singles tier 2 $80.95.
#[test]
fn a_dependent_person_who_is_no_child_dependant_still_brings_family_thresholds() {
    let mut facts = input("2025-26", "150000", json!([line("1000", "0", 30)]));
    facts["maintained_dependent_person"] = json!(true);
    let r = run(&facts);
    assert_eq!(r["thresholds"], "families");
    assert_eq!(r["tier"], "base");
    assert_eq!(amount(&r, "private_health_insurance_offset"), "242.88");
    facts["maintained_dependent_person"] = json!(false);
    assert_eq!(run(&facts)["tier"], "tier_2");
}

/// The child count raises only family thresholds, so without a spouse or a
/// dependent person it is refused rather than ignored.
#[test]
fn dependent_children_alone_are_refused() {
    let mut facts = input("2025-26", "150000", json!([line("1000", "0", 30)]));
    facts["dependent_children"] = json!(2);
    assert_eq!(code(&facts), "input.field_unknown");
    facts["maintained_dependent_person"] = json!("yes");
    assert_eq!(code(&facts), "input.flag_invalid");
}

/// Two $1,000 premiums before 1 April, tier 1 (16.192%), one reduced at the
/// base rate ($242.88) and one at tier 2 ($80.95): as separate lines, an
/// excess of $80.96 and an offset of $80.97; summed into one statement line,
/// only their net, an offset of $0.01.
#[test]
fn premiums_with_different_reductions_show_both_amounts_as_separate_lines() {
    let apart = json!([line("1000", "242.88", 30), line("1000", "80.95", 30)]);
    let r = run(&input("2025-26", "110000", apart));
    assert_eq!(amount(&r, "excess_private_health_reduction"), "80.96");
    assert_eq!(amount(&r, "private_health_insurance_offset"), "80.97");
    let summed = run(&input(
        "2025-26",
        "110000",
        json!([line("2000", "323.83", 30)]),
    ));
    assert_eq!(amount(&summed, "private_health_insurance_offset"), "0.01");
    assert_eq!(amount(&summed, "excess_private_health_reduction"), "0.00");
}

#[test]
fn premiums_paid_from_1_april_2027_are_refused() {
    for c in [31, 36, 41] {
        let facts = input("2026-27", "50000", json!([line("100", "0", c)]));
        assert_eq!(
            code(&facts),
            "private_health_insurance_rebate.rate_unpublished"
        );
    }
}

#[test]
fn years_outside_2020_21_to_2026_27_are_refused() {
    for year in ["2019-20", "2027-28"] {
        let facts = input(year, "50000", json!([line("100", "0", 30)]));
        assert_eq!(code(&facts), "period.unsupported");
    }
}

#[test]
fn a_lifetime_health_cover_loading_is_refused() {
    let loaded = json!([{"premiums_eligible": "100", "rebate_received": "0", "benefit_code": 30,
        "lifetime_health_cover_loading": "20"}]);
    assert_eq!(
        code(&input("2025-26", "50000", loaded)),
        "private_health_insurance_rebate.lhc_loading"
    );
}

#[test]
fn malformed_lines_are_refused() {
    let invalid = "private_health_insurance_rebate.lines_invalid";
    for lines in [
        json!([]),
        json!("lines"),
        json!([1]),
        json!([line("100", "0", 32)]),
        json!([{"premiums_eligible": "100", "rebate_received": "0", "benefit_code": "30"}]),
        json!([{"premiums_eligible": "100", "rebate_received": "0"}]),
        json!([{"premiums_eligible": "100", "rebate_received": "0", "benefit_code": 30, "adults": 2}]),
        json!([line("-1", "0", 30)]),
        json!([line("100.001", "0", 30)]),
        json!([{"premiums_eligible": 100, "rebate_received": "0", "benefit_code": 30}]),
    ] {
        assert_eq!(
            code(&input("2025-26", "50000", lines.clone())),
            invalid,
            "{lines}"
        );
    }
    let too_many = Value::Array(vec![line("1", "0", 30); 101]);
    assert_eq!(code(&input("2025-26", "50000", too_many)), invalid);
}

#[test]
fn other_inputs_are_refused_as_every_worksheet_refuses_them() {
    let lines = json!([line("100", "0", 30)]);
    let mut facts = input("2025-26", "50000", lines.clone());
    facts["dependent_children"] = json!(-1);
    assert_eq!(code(&facts), "input.dependants_invalid");
    let mut facts = input("2025-26", "50000", lines.clone());
    facts["scope_confirmed"] = json!(false);
    assert_eq!(code(&facts), "scope.not_confirmed");
    let mut facts = input("2025-26", "50000", lines.clone());
    facts["adults"] = json!(2);
    assert_eq!(code(&facts), "input.field_unknown");
    assert_eq!(code(&input("2025-26", "-5", lines)), "input.amount_invalid");
}

const ESTIMATE: &str = "personal-tax-estimate";

fn estimate(extra: Value) -> Result<Value, api::Refusal> {
    let mut facts = json!({
        "year": "2025-26", "scope_confirmed": true, "residency": "full_year_resident",
        "taxable_income": "83000",
        "medicare-levy": {"section_9a_adjustment_applies": false},
        "medicare-levy-surcharge": {"reportable_fringe_benefits": "0",
            "income_for_surcharge_purposes": "83000", "days_without_cover": 0},
        "tax_withheld": "15000",
    });
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    api::evaluate_coded(ESTIMATE, &facts)
}

fn money(out: &Value, key: &str) -> f64 {
    out["amounts"][key].as_str().unwrap().parse().unwrap()
}

/// In the estimate, Narelle's $96.94 offset comes off the total and so off
/// the amount payable, with the income taken from the surcharge part.
#[test]
fn the_estimate_takes_the_offset_off_the_total() {
    let without = estimate(json!({})).unwrap();
    let phi = json!({"lines": year_lines(30, ("900", "146"), ("300", "48"))});
    let with = estimate(json!({"private-health-insurance-rebate": phi})).unwrap();
    assert_eq!(with["amounts"]["private_health_insurance_offset"], "96.94");
    assert_eq!(with["amounts"]["excess_private_health_reduction"], "0.00");
    let moved = money(&without, "total_before_credits") - money(&with, "total_before_credits");
    assert!((moved - 96.94).abs() < 1e-9, "{moved}");
    assert_eq!(with["not_composed"], without["not_composed"]);
    assert_eq!(
        with["components"]["private-health-insurance-rebate"]["amounts"]["income_tested"],
        "83000.00"
    );
    let keys: Vec<&String> = with["amounts"].as_object().unwrap().keys().collect();
    let at = keys
        .iter()
        .position(|k| *k == "total_before_credits")
        .unwrap();
    assert_eq!(keys[at - 2], "private_health_insurance_offset");
    assert!(
        without["amounts"]
            .get("private_health_insurance_offset")
            .is_none()
    );
}

/// An excess premium reduction adds to the total: Zoe's $161.19 on an income
/// of $119,000.
#[test]
fn the_estimate_adds_an_excess_to_the_total() {
    let mut zoe = json!({"taxable_income": "119000",
        "medicare-levy-surcharge": {"reportable_fringe_benefits": "0",
            "income_for_surcharge_purposes": "119000", "days_without_cover": 0}});
    let without = estimate(zoe.clone()).unwrap();
    zoe["private-health-insurance-rebate"] =
        json!({"lines": year_lines(30, ("750", "182"), ("250", "60"))});
    let with = estimate(zoe).unwrap();
    let moved = money(&with, "total_before_credits") - money(&without, "total_before_credits");
    assert!((moved - 161.19).abs() < 1e-9, "{moved}");
}

#[test]
fn the_estimate_refuses_the_part_for_a_foreign_resident_and_income_given_twice() {
    let phi = json!({"lines": [line("100", "0", 30)]});
    let foreign = json!({"residency": "foreign_resident", "medicare-levy": null,
        "private-health-insurance-rebate": phi});
    let mut facts = json!({"year": "2025-26", "scope_confirmed": true, "taxable_income": "83000"});
    for (k, v) in foreign.as_object().unwrap() {
        if !v.is_null() {
            facts[k] = v.clone();
        }
    }
    let r = api::evaluate_coded(ESTIMATE, &facts).unwrap_err();
    assert_eq!(r.code, "input.field_unknown");
    assert!(r.message.contains("61-205(1)(c)"));
    for key in [
        "income_for_surcharge_purposes",
        "spouse_income_for_surcharge_purposes",
        "dependent_children",
    ] {
        let mut part = json!({"lines": [line("100", "0", 30)]});
        part[key] = json!("1");
        let r = estimate(json!({"private-health-insurance-rebate": part})).unwrap_err();
        assert_eq!(r.code, "input.field_unknown", "{key}");
        assert!(r.message.contains("medicare-levy-surcharge"), "{key}");
    }
}

/// The offset is refundable, so with no tax it takes the total below nil and
/// is refunded in full: nil taxable income and withholding, $1,000 of base
/// tier premiums with no reduction gives -$242.88 and a $242.88 refund.
#[test]
fn the_estimate_refunds_an_offset_beyond_the_tax() {
    let mut nil = json!({"taxable_income": "0", "tax_withheld": "0",
        "medicare-levy-surcharge": {"reportable_fringe_benefits": "0",
            "income_for_surcharge_purposes": "0", "days_without_cover": 0}});
    nil["private-health-insurance-rebate"] = json!({"lines": [line("1000", "0", 30)]});
    let out = estimate(nil).unwrap();
    assert_eq!(out["amounts"]["total_before_credits"], "-242.88");
    assert_eq!(out["amounts"]["refund"], "242.88");
    assert_eq!(out["amounts"]["amount_payable"], "0.00");
}

/// A single person maintaining a 26-year-old dependent student: no surcharge
/// dependant, but the rebate's family thresholds (s 22-30(1)(b)), so $150,000
/// is base tier and $1,000 of premiums gives a $242.88 offset.
#[test]
fn the_estimate_gives_a_dependent_person_without_a_spouse_family_thresholds() {
    let mut single = json!({"taxable_income": "150000",
        "medicare-levy-surcharge": {"reportable_fringe_benefits": "0",
            "income_for_surcharge_purposes": "150000", "days_without_cover": 0}});
    single["private-health-insurance-rebate"] =
        json!({"lines": [line("1000", "0", 30)], "maintained_dependent_person": true});
    let out = estimate(single).unwrap();
    let part = &out["components"]["private-health-insurance-rebate"];
    assert_eq!(part["thresholds"], "families");
    assert_eq!(part["tier"], "base");
    assert_eq!(out["amounts"]["private_health_insurance_offset"], "242.88");
}
