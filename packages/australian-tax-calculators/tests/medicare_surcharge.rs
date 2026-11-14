// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The Medicare levy surcharge (Medicare Levy Act 1986 ss 8B and 8D and the
//! Medicare Levy Surcharge - Fringe Benefits Act 1999 ss 12, 15 and 16): the
//! ATO's Tom, Josh, Jacinta and Jill and Kevin examples, every tier boundary,
//! the day apportionment and the refusals
//! (docs/calculation-evidence.md#medicare-levy-surcharge).
use australian_tax_calculators::api;
use australian_tax_calculators::engine::worksheet_catalogue;
use serde_json::{Value, json};

const COMMAND: &str = "medicare-levy-surcharge";

/// Taxable income, reportable fringe benefits and income for surcharge
/// purposes, for the whole year without cover unless `extra` says otherwise.
fn input(year: &str, taxable: &str, fringe: &str, income: &str, extra: Value) -> Value {
    let mut facts = json!({"taxable_income": taxable, "reportable_fringe_benefits": fringe,
                           "income_for_surcharge_purposes": income,
                           "days_without_cover": 365, "year": year, "scope_confirmed": true});
    for (k, v) in extra.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn evaluate(year: &str, taxable: &str, fringe: &str, income: &str, extra: Value) -> Value {
    api::evaluate(COMMAND, &input(year, taxable, fringe, income, extra)).unwrap()
}

/// The surcharge, the rate and the tier for income that is all taxable income.
fn on_income(year: &str, income: &str, extra: Value) -> (String, String, String) {
    let result = evaluate(year, income, "0", income, extra);
    let text = |v: &Value| v.as_str().unwrap().to_string();
    (
        text(&result["amounts"]["medicare_levy_surcharge"]),
        text(&result["rates"]["surcharge_rate"]),
        text(&result["rates"]["tier"]),
    )
}

/// ATO, Medicare levy surcharge income, thresholds and rates (last updated 22
/// June 2026), read on 14 November 2026: in 2026-27 Tom, single without
/// dependants or cover, has taxable income of $90,000 and reportable fringe
/// benefits of $27,000, so income for surcharge purposes of $117,000, a tier 1
/// earner; his surcharge is $1,170 ($117,000 x 1%).
#[test]
fn ato_tom_example_2026_27() {
    let result = evaluate("2026-27", "90000", "27000", "117000", json!({}));
    assert_eq!(
        result["amounts"],
        json!({"income_for_surcharge_purposes": "117000.00", "surcharge_base": "117000.00",
               "medicare_levy_surcharge": "1170.00"})
    );
    assert_eq!(
        result["rates"],
        json!({"tier": "1", "surcharge_rate": "0.01", "tier_1_threshold": "105000",
               "tier_2_threshold": "123000", "tier_3_threshold": "164000",
               "days_without_cover": "365", "days_in_year": "365"})
    );
}

/// ATO, M2 Medicare levy surcharge 2026 (last updated 30 May 2026): Josh's
/// $90,000 taxable income, $20,000 reportable fringe benefits and $17,000 net
/// investment losses make $127,000, a tier 2 earner in 2025-26, but 1.25%
/// applies only to taxable income and reportable fringe benefits: $1,375.
#[test]
fn ato_josh_example_the_tier_comes_from_all_income_the_base_does_not() {
    let result = evaluate("2025-26", "90000", "20000", "127000", json!({}));
    assert_eq!(result["rates"]["tier"], "2");
    assert_eq!(result["rates"]["surcharge_rate"], "0.0125");
    assert_eq!(result["amounts"]["surcharge_base"], "110000.00");
    assert_eq!(result["amounts"]["medicare_levy_surcharge"], "1375.00");
}

/// Tom's facts with $10,000 of reportable super contributions: income for
/// surcharge purposes of $127,000 makes him a tier 2 earner, but the
/// contributions are not in the base, so $117,000 x 1.25% = $1,462.50, not
/// $127,000 x 1.25% = $1,587.50.
#[test]
fn reportable_super_contributions_change_the_tier_not_the_base() {
    let result = evaluate("2026-27", "90000", "27000", "127000", json!({}));
    assert_eq!(result["rates"]["tier"], "2");
    assert_eq!(result["amounts"]["surcharge_base"], "117000.00");
    assert_eq!(result["amounts"]["medicare_levy_surcharge"], "1462.50");
}

/// M2 2026: Jacinta, single with income for surcharge purposes of $109,000 in
/// 2025-26, took out cover on 16 January 2026, so 166 days are covered and
/// 199 are not. The ATO gives the days, not her taxable income: taking it as
/// the whole $109,000, the surcharge is 109,000 x 1% x 199 / 365 = 594.27.
#[test]
fn ato_jacinta_example_apportions_by_the_days_without_cover() {
    let result = evaluate(
        "2025-26",
        "109000",
        "0",
        "109000",
        json!({"days_without_cover": 199}),
    );
    assert_eq!(result["amounts"]["medicare_levy_surcharge"], "594.27");
    assert_eq!(result["rates"]["days_without_cover"], "199");
    let none = evaluate(
        "2025-26",
        "109000",
        "0",
        "109000",
        json!({"days_without_cover": 0}),
    );
    assert_eq!(none["amounts"]["medicare_levy_surcharge"], "0.00");
}

/// M2 2026: Jill and Kevin, married with 3 dependent children, have combined
/// income for surcharge purposes of $220,000 in 2025-26, above their family
/// threshold of $205,000 ($202,000 + 2 x $1,500). Kevin had no cover until 10
/// January 2026, so each is liable for 193 days. Their split is not given:
/// with Kevin on $120,000 and Jill on $100,000, all taxable income, Kevin's is
/// 120,000 x 1% x 193 / 365 = 634.52 and Jill's 100,000 x 1% x 193 / 365 = 528.77.
#[test]
fn ato_jill_and_kevin_example_family_threshold_and_days() {
    let family = |spouse: &str| {
        json!({"spouse_income_for_surcharge_purposes": spouse, "dependent_children": 3,
               "days_without_cover": 193})
    };
    let kevin = evaluate("2025-26", "120000", "0", "120000", family("100000"));
    assert_eq!(
        kevin["amounts"]["family_income_for_surcharge_purposes"],
        "220000.00"
    );
    assert_eq!(kevin["rates"]["tier_1_threshold"], "205000");
    assert_eq!(kevin["rates"]["own_income_threshold"], "28011");
    assert_eq!(kevin["amounts"]["medicare_levy_surcharge"], "634.52");
    let jill = evaluate("2025-26", "100000", "0", "100000", family("120000"));
    assert_eq!(jill["amounts"]["medicare_levy_surcharge"], "528.77");
}

/// Each year's singles and family thresholds, as the ATO's tables give them,
/// at the dollar and the cent either side: income must exceed a threshold to
/// pass it. A family with 1 child has the plain family thresholds; each child
/// after the first adds $1,500.
#[test]
fn every_tier_boundary_in_each_year() {
    let years: [(&str, [i64; 3], [i64; 3]); 3] = [
        ("2024-25", [97000, 113000, 151000], [194000, 226000, 302000]),
        (
            "2025-26",
            [101000, 118000, 158000],
            [202000, 236000, 316000],
        ),
        (
            "2026-27",
            [105000, 123000, 164000],
            [210000, 246000, 328000],
        ),
    ];
    for (year, singles, family) in years {
        let couples: [(i64, i64); 4] = [(0, 0), (1, 0), (2, 1500), (4, 4500)];
        let cases = std::iter::once((json!({}), singles)).chain(couples.iter().map(
            |&(children, raise)| {
                // The spouse's income is nil, so the person's own is the family's.
                let extra = json!({"spouse_income_for_surcharge_purposes": "0",
                                   "dependent_children": children});
                (extra, family.map(|t| t + raise))
            },
        ));
        for (extra, thresholds) in cases {
            for (tier, threshold) in thresholds.iter().enumerate() {
                let case = format!("{year} {extra} {threshold}");
                let below = if tier == 0 {
                    "base".to_string()
                } else {
                    tier.to_string()
                };
                let at = on_income(year, &threshold.to_string(), extra.clone());
                assert_eq!(at.2, below, "{case}");
                let cent = format!("{threshold}.01");
                let above = evaluate(year, &threshold.to_string(), "0", &cent, extra.clone());
                assert_eq!(above["rates"]["tier"], (tier + 1).to_string(), "{case}");
                let rate = ["0.01", "0.0125", "0.015"][tier];
                let over = on_income(year, &(threshold + 1).to_string(), extra.clone());
                assert_eq!(
                    (over.1.as_str(), over.2),
                    (rate, (tier + 1).to_string()),
                    "{case}"
                );
            }
            let base = on_income(year, &thresholds[0].to_string(), extra);
            assert_eq!((base.0.as_str(), base.1.as_str()), ("0.00", "0"), "{year}");
        }
    }
}

/// Tier 3 at the top: 1.5% of $200,000 in 2025-26 is $3,000.
#[test]
fn tier_3_charges_one_and_a_half_percent() {
    assert_eq!(
        on_income("2025-26", "200000", json!({})),
        ("3000.00".to_string(), "0.015".to_string(), "3".to_string())
    );
}

/// s 8D(3)(c): a married person is liable only if their own income for
/// surcharge purposes also exceeds $28,011 ($27,222 for 2024-25), whatever the
/// family's income.
#[test]
fn a_married_person_on_a_low_income_of_their_own_pays_nothing() {
    // With a spouse on $240,000 the family is in tier 2 in each year.
    for (year, floor, above) in [
        ("2024-25", 27222, "340.29"), // 1.25% x 27,223 = 340.2875
        ("2025-26", 28011, "350.15"), // 1.25% x 28,012
        ("2026-27", 28011, "350.15"),
    ] {
        let spouse = json!({"spouse_income_for_surcharge_purposes": "240000"});
        let at = on_income(year, &floor.to_string(), spouse.clone());
        assert_eq!(
            (at.0.as_str(), at.1.as_str(), at.2.as_str()),
            ("0.00", "0", "2"),
            "{year}"
        );
        let over = on_income(year, &(floor + 1).to_string(), spouse);
        assert_eq!(
            (over.0.as_str(), over.1.as_str()),
            (above, "0.0125"),
            "{year}"
        );
    }
}

#[test]
fn inputs_the_worksheet_cannot_settle_are_refused() {
    let refused = |extra: Value| {
        let facts = input("2025-26", "120000", "0", "120000", extra);
        let error = api::evaluate_coded(COMMAND, &facts).unwrap_err();
        (error.code, error.message)
    };
    let field = "input.field_unknown";
    for (extra, code, message) in [
        (
            json!({"dependent_children": 2}),
            field,
            "dependent_children is not an input of medicare-levy-surcharge without spouse_income_for_surcharge_purposes",
        ),
        (
            json!({"private_hospital_cover": false}),
            field,
            "private_hospital_cover is not an input of medicare-levy-surcharge",
        ),
        (
            json!({"spouse_income_for_surcharge_purposes": "0", "dependent_children": -1}),
            "input.dependants_invalid",
            "dependent_children must be a non-negative integer.",
        ),
        (
            json!({"spouse_income_for_surcharge_purposes": "-1"}),
            "input.amount_invalid",
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            json!({"taxable_income": "120000.50"}),
            "input.amount_not_whole_dollars",
            "Supply established whole-dollar taxable income.",
        ),
        (
            json!({"days_without_cover": 366}),
            "input.day_count_invalid",
            "days_without_cover must be an integer from 0 to 365.",
        ),
        (
            json!({"days_without_cover": "365"}),
            "input.day_count_invalid",
            "days_without_cover must be an integer from 0 to 365.",
        ),
        (
            json!({"year": "2023-24"}),
            "period.unsupported",
            "Unsupported period '2023-24'; supported: 2024-25, 2025-26, 2026-27.",
        ),
        (
            json!({"year": "2027-28"}),
            "period.unsupported",
            "Unsupported period '2027-28'; supported: 2024-25, 2025-26, 2026-27.",
        ),
    ] {
        assert_eq!(
            refused(extra.clone()),
            (code, message.to_string()),
            "{extra}"
        );
    }
    // Omitting the optional inputs, or no children, is a single person.
    let single = evaluate("2025-26", "120000", "0", "120000", json!({}));
    assert_eq!(
        single,
        evaluate(
            "2025-26",
            "120000",
            "0",
            "120000",
            json!({"dependent_children": 0})
        )
    );
    assert!(single["rates"].get("own_income_threshold").is_none());
}

/// The catalogue lists the worksheet with its optional inputs, and its example
/// is the ATO's Tom example.
#[test]
fn the_catalogue_example_is_the_tom_example() {
    let catalogue = worksheet_catalogue();
    let entry = &catalogue["medicare_levy_surcharge"];
    let optional = entry["optional_inputs"].as_object().unwrap();
    assert_eq!(optional.len(), 2);
    for key in optional.keys() {
        let mut facts = input("2025-26", "120000", "0", "120000", json!({}));
        facts[key] = json!([]);
        assert!(api::evaluate(COMMAND, &facts).is_err(), "{key}");
    }
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().shift_remove("kind");
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(result["amounts"]["medicare_levy_surcharge"], "1170.00");
    let periods = api::evaluate("periods", &json!({"kind": "medicare_levy_surcharge"})).unwrap();
    assert_eq!(periods.as_array().unwrap().len(), 3);
}
