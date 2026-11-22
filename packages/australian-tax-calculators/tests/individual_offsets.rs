// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The beneficiary tax offset (ITAA 1936 s 160AAA), the invalid and invalid
//! carer tax offset (ITAA 1997 Subdivision 61-A), the zone and overseas forces
//! tax offsets (ITAA 1936 ss 79A and 79B) and the net medical expenses tax
//! offset (ss 159P and 159Q) on the ATO's examples, every published amount
//! and limit by year, each boundary and every refusal, all read on
//! 22 November 2026 (docs/calculation-evidence.md#zone-tax-offset and the
//! headings beside it).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(command: &str, facts: &Value) -> Value {
    api::evaluate(command, facts).unwrap()
}

fn code(command: &str, facts: &Value) -> &'static str {
    api::evaluate_coded(command, facts).unwrap_err().code
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

fn with(mut base: Value, extra: Value) -> Value {
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

const YEARS: [&str; 7] = [
    "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
];

// Beneficiary tax offset.

const BENEFICIARY: &str = "beneficiary-tax-offset";

fn beneficiary(year: &str, benefit: &str) -> String {
    let facts = json!({"year": year, "scope_confirmed": true, "rebatable_benefit": benefit});
    amount(&run(BENEFICIARY, &facts), "beneficiary_tax_offset").to_string()
}

/// Regulations s 13: 15% above $6,000, plus 15% above $45,000 (the top of the
/// lowest marginal rate in every year), on the benefit rounded down to the
/// dollar, the offset rounded up.
#[test]
fn beneficiary_offset_follows_the_regulation_formula_every_year() {
    for year in YEARS {
        for (benefit, offset) in [
            ("0", "0.00"),
            ("6000", "0.00"),
            ("6000.99", "0.00"),
            ("6001", "1.00"),
            ("20000", "2100.00"),
            ("30123", "3619.00"),
            ("45000", "5850.00"),
            ("45001", "5851.00"),
            ("50000.75", "7350.00"),
        ] {
            assert_eq!(beneficiary(year, benefit), offset, "{year} {benefit}");
        }
    }
}

#[test]
fn beneficiary_offset_shows_the_amount_it_works_on() {
    let facts =
        json!({"year": "2025-26", "scope_confirmed": true, "rebatable_benefit": "30123.60"});
    let r = run(BENEFICIARY, &facts);
    assert_eq!(amount(&r, "rebatable_benefit_amount"), "30123.00");
    assert_eq!(r["rates"]["upper_threshold"], "45000");
}

#[test]
fn beneficiary_offset_refuses_other_years_and_unknown_fields() {
    for year in ["2019-20", "2027-28"] {
        let facts = json!({"year": year, "scope_confirmed": true, "rebatable_benefit": "20000"});
        assert_eq!(code(BENEFICIARY, &facts), "period.unsupported");
    }
    let facts = json!({"year": "2025-26", "scope_confirmed": true, "rebatable_benefit": "20000", "seniors_pensioners_tax_offset": "100"});
    assert_eq!(code(BENEFICIARY, &facts), "input.field_unknown");
}

// Invalid and invalid carer tax offset.

const CARER: &str = "invalid-carer-tax-offset";

fn carer(year: &str, extra: Value) -> Value {
    let base = json!({"year": year, "scope_confirmed": true, "dependant": "other",
        "adjusted_taxable_income": "80000", "dependant_adjusted_taxable_income": "0"});
    with(base, extra)
}

fn carer_offset(facts: &Value) -> String {
    amount(&run(CARER, facts), "invalid_carer_tax_offset").to_string()
}

/// The ATO's T5 instructions for 2021 to 2026: each year's amount at nil
/// income, $1 left one dollar below the cut-out and nil at it
/// ($282 + 4 x the amount), and the daily rate for part of the year.
#[test]
fn invalid_carer_offset_uses_each_years_published_amounts() {
    for (year, maximum, cut_out, daily_100) in [
        ("2020-21", "2816.00", 11546, "772.00"),
        ("2021-22", "2833.00", 11614, "776.00"),
        ("2022-23", "2943.00", 12054, "806.00"),
        ("2023-24", "3152.00", 12890, "861.00"),
        ("2024-25", "3300.00", 13482, "904.00"),
        ("2025-26", "3396.00", 13866, "930.00"),
    ] {
        assert_eq!(carer_offset(&carer(year, json!({}))), maximum, "{year}");
        let below = (cut_out - 1).to_string();
        let at = cut_out.to_string();
        assert_eq!(
            carer_offset(&carer(
                year,
                json!({"dependant_adjusted_taxable_income": below})
            )),
            "1.00",
            "{year}"
        );
        assert_eq!(
            carer_offset(&carer(
                year,
                json!({"dependant_adjusted_taxable_income": at})
            )),
            "0.00",
            "{year}"
        );
        assert_eq!(
            carer_offset(&carer(year, json!({"days": 100}))),
            daily_100,
            "{year}"
        );
    }
}

#[test]
fn invalid_carer_offset_reduces_by_a_dollar_for_every_four_over_282() {
    for (income, offset) in [("285", "3396.00"), ("286", "3395.00"), ("4282", "2396.00")] {
        let facts = carer(
            "2025-26",
            json!({"dependant_adjusted_taxable_income": income}),
        );
        assert_eq!(carer_offset(&facts), offset, "{income}");
    }
    // Part of the year: days x $9.30, less the reduction on the period's income.
    let facts = carer(
        "2025-26",
        json!({"days": 100, "dependant_adjusted_taxable_income": "1000"}),
    );
    let r = run(CARER, &facts);
    assert_eq!(amount(&r, "amount_before_reduction"), "930.00");
    assert_eq!(amount(&r, "income_reduction"), "179.00");
    assert_eq!(amount(&r, "invalid_carer_tax_offset"), "751.00");
    // A spouse for 200 days that count.
    let facts = carer("2025-26", json!({"dependant": "spouse", "days": 200}));
    assert_eq!(carer_offset(&facts), "1860.00");
    // No days that count: nil.
    let facts = carer("2025-26", json!({"dependant": "spouse", "days": 0}));
    assert_eq!(carer_offset(&facts), "0.00");
}

/// s 61-20: more than the FTB Part B limit, with a spouse who is not the
/// dependant added (scaled by the days with that spouse), leaves no offset.
#[test]
fn invalid_carer_offset_applies_the_income_limit() {
    let at = carer("2025-26", json!({"adjusted_taxable_income": "120007"}));
    assert_eq!(carer_offset(&at), "3396.00");
    assert_eq!(run(CARER, &at)["eligible"], true);
    let over = carer("2025-26", json!({"adjusted_taxable_income": "120008"}));
    assert_eq!(carer_offset(&over), "0.00");
    assert_eq!(run(CARER, &over)["eligible"], false);
    let spouse = |income: &str| {
        carer(
            "2025-26",
            json!({"adjusted_taxable_income": "100000",
            "spouse_adjusted_taxable_income": income}),
        )
    };
    assert_eq!(carer_offset(&spouse("20007")), "3396.00");
    assert_eq!(carer_offset(&spouse("20008")), "0.00");
    let part = carer(
        "2025-26",
        json!({"adjusted_taxable_income": "112707",
        "spouse_adjusted_taxable_income": "36500", "spouse_days": 73}),
    );
    let r = run(CARER, &part);
    assert_eq!(amount(&r, "income_tested"), "120007.00");
    assert_eq!(amount(&r, "invalid_carer_tax_offset"), "3396.00");
    for year in ["2020-21", "2023-24"] {
        let limit = if year == "2020-21" { 100000 } else { 112578 };
        let at = carer(year, json!({"adjusted_taxable_income": limit.to_string()}));
        assert_ne!(carer_offset(&at), "0.00", "{year}");
        let over = carer(
            year,
            json!({"adjusted_taxable_income": (limit + 1).to_string()}),
        );
        assert_eq!(carer_offset(&over), "0.00", "{year}");
    }
}

#[test]
fn invalid_carer_offset_refusals() {
    assert_eq!(
        code(CARER, &carer("2026-27", json!({}))),
        "invalid_carer_tax_offset.rate_unpublished"
    );
    assert_eq!(
        code(CARER, &carer("2019-20", json!({}))),
        "period.unsupported"
    );
    assert_eq!(
        code(
            CARER,
            &carer("2025-26", json!({"maintained_by_others": true}))
        ),
        "invalid_carer_tax_offset.shared_maintenance"
    );
    assert_eq!(
        code(CARER, &carer("2025-26", json!({"ftb_b_shared_care": true}))),
        "invalid_carer_tax_offset.ftb_shared_care"
    );
    // s 61-20(3): a former invalid spouse, then a new spouse with income.
    assert_eq!(
        code(
            CARER,
            &carer(
                "2025-26",
                json!({"dependant": "spouse", "days": 100, "different_spouses": true})
            )
        ),
        "invalid_carer_tax_offset.successive_spouses"
    );
    // With 2 faults, shared maintenance is refused first.
    let both = json!({"maintained_by_others": true, "different_spouses": true});
    assert_eq!(
        code(CARER, &carer("2025-26", both)),
        "invalid_carer_tax_offset.shared_maintenance"
    );
    // The limit is strict: a spouse's $3.65 for 1 day of 365 is $0.01,
    // reaching $120,007 exactly; $3.66 goes over it.
    for (spouse, eligible) in [("3.65", true), ("3.66", false)] {
        let facts = carer(
            "2025-26",
            json!({"adjusted_taxable_income": "120006.99",
                "spouse_adjusted_taxable_income": spouse, "spouse_days": 1}),
        );
        assert_eq!(run(CARER, &facts)["eligible"], json!(eligible), "{spouse}");
    }
    let shared_false = carer(
        "2025-26",
        json!({"maintained_by_others": false, "ftb_b_shared_care": false, "different_spouses": false}),
    );
    assert_eq!(carer_offset(&shared_false), "3396.00");
    assert_eq!(
        code(CARER, &carer("2025-26", json!({"dependant": "child"}))),
        "invalid_carer_tax_offset.dependant_invalid"
    );
    let spouse_twice = carer(
        "2025-26",
        json!({"dependant": "spouse", "spouse_adjusted_taxable_income": "1"}),
    );
    assert_eq!(code(CARER, &spouse_twice), "input.field_unknown");
    assert_eq!(
        code(CARER, &carer("2025-26", json!({"spouse_days": 10}))),
        "input.field_unknown"
    );
    assert_eq!(
        code(CARER, &carer("2025-26", json!({"days": 366}))),
        "input.day_count_invalid"
    );
    assert_eq!(
        code(CARER, &carer("2023-24", json!({"days": 367}))),
        "input.day_count_invalid"
    );
}

// Zone and overseas forces tax offsets.

const ZONE: &str = "zone-tax-offset";

fn zone(year: &str, area: &str, days: i64, extra: Value) -> Value {
    let base = json!({"year": year, "scope_confirmed": true, "area": area, "days": days});
    with(base, extra)
}

fn offset(facts: &Value) -> String {
    amount(&run(ZONE, facts), "tax_offset").to_string()
}

/// T4 table 1 in each year's instructions, 2021 to 2026, and the law's
/// amounts for 2026-27.
#[test]
fn zone_fixed_amounts_every_year() {
    for year in YEARS {
        for (area, fixed) in [
            ("special_area", "1173.00"),
            ("zone_a", "338.00"),
            ("zone_b", "57.00"),
            ("overseas_forces", "338.00"),
        ] {
            assert_eq!(
                offset(&zone(year, area, 250, json!({}))),
                fixed,
                "{year} {area}"
            );
        }
    }
}

/// s 79A(3B)(a): more than one-half of the year, 183 days of 365 and 184 of
/// 2023-24's 366; 183 days of 2023-24 is refused, as the ATO says 183 or more.
#[test]
fn zone_residence_needs_more_than_half_the_year() {
    let r = run(ZONE, &zone("2025-26", "zone_a", 183, json!({})));
    assert_eq!(r["resident"], true);
    assert_eq!(amount(&r, "tax_offset"), "338.00");
    let r = run(ZONE, &zone("2025-26", "zone_a", 182, json!({})));
    assert_eq!(r["resident"], false);
    assert_eq!(amount(&r, "tax_offset"), "0.00");
    assert_eq!(offset(&zone("2023-24", "zone_a", 184, json!({}))), "338.00");
    assert_eq!(offset(&zone("2023-24", "zone_a", 182, json!({}))), "0.00");
    assert_eq!(
        code(ZONE, &zone("2023-24", "zone_a", 183, json!({}))),
        "zone_tax_offset.half_year_unclear"
    );
    // Days carried from 2022-23 settle it either way.
    let carried = zone(
        "2023-24",
        "zone_a",
        183,
        json!({"prior_year": "2022-23", "prior_days": 1}),
    );
    assert_eq!(offset(&carried), "338.00");
}

/// The ATO's Narendra example (Zone tax offset, QC105018): Lord Howe Island
/// (a Zone A special area) from 23 March 2025 to 31 October 2025, 100 days in
/// 2024-25 and 124 in 2025-26: 224 days, more than 182, under s 79A(3B)(d).
#[test]
fn narendra_carries_his_2024_25_days() {
    let facts = zone(
        "2025-26",
        "special_area",
        124,
        json!({"prior_year": "2024-25", "prior_days": 100}),
    );
    let r = run(ZONE, &facts);
    assert_eq!(r["resident"], true);
    assert!(r["residence"].as_str().unwrap().starts_with("s 79A(3B)(d)"));
    assert_eq!(amount(&r, "tax_offset"), "1173.00");
    let short = zone(
        "2025-26",
        "special_area",
        82,
        json!({"prior_year": "2024-25", "prior_days": 100}),
    );
    assert_eq!(offset(&short), "0.00");
    let just = zone(
        "2025-26",
        "special_area",
        83,
        json!({"prior_year": "2024-25", "prior_days": 100}),
    );
    assert_eq!(offset(&just), "1173.00");
}

/// The ATO's Garry example (T4 2026): 122 days in 2020-21 carried into 92 in
/// 2025-26. 2020-21 is the fifth preceding year, which s 79A(4)'s words leave
/// out and the ATO counts, so it is refused; from 2022-23 the same days pass
/// under s 79A(3B)(e) only with continuous residence and a period from 1 July.
#[test]
fn garry_and_the_relevant_preceding_year() {
    let garry = json!({"prior_year": "2020-21", "prior_days": 122,
        "continuous_residence": true, "includes_first_day": true});
    assert_eq!(
        code(ZONE, &zone("2025-26", "zone_a", 92, garry)),
        "zone_tax_offset.prior_year_unclear"
    );
    let earlier = json!({"prior_year": "2022-23", "prior_days": 122,
        "continuous_residence": true, "includes_first_day": true});
    let r = run(ZONE, &zone("2025-26", "zone_a", 92, earlier));
    assert!(r["residence"].as_str().unwrap().starts_with("s 79A(3B)(e)"));
    assert_eq!(amount(&r, "tax_offset"), "338.00");
    for (continuous, first) in [(false, true), (true, false)] {
        let facts = zone(
            "2025-26",
            "zone_a",
            92,
            json!({"prior_year": "2022-23",
            "prior_days": 122, "continuous_residence": continuous, "includes_first_day": first}),
        );
        assert_eq!(offset(&facts), "0.00");
    }
    let missing = zone(
        "2025-26",
        "zone_a",
        92,
        json!({"prior_year": "2022-23", "prior_days": 122}),
    );
    assert_eq!(offset(&missing), "0.00");
    // 2021-22 is the fourth year before 2025-26, the last s 79A(4) allows.
    let fourth = json!({"prior_year": "2021-22", "prior_days": 122,
        "continuous_residence": true, "includes_first_day": true});
    assert_eq!(offset(&zone("2025-26", "zone_a", 92, fourth)), "338.00");
}

#[test]
fn zone_prior_year_refusals() {
    let refused = |extra: Value| code(ZONE, &zone("2025-26", "zone_a", 92, extra));
    assert_eq!(
        refused(json!({"prior_year": "2019-20", "prior_days": 100})),
        "zone_tax_offset.prior_year_invalid"
    );
    assert_eq!(
        refused(json!({"prior_year": "2025-26", "prior_days": 100})),
        "zone_tax_offset.prior_year_invalid"
    );
    assert_eq!(
        refused(json!({"prior_year": "2024-26", "prior_days": 100})),
        "zone_tax_offset.prior_year_invalid"
    );
    assert_eq!(
        refused(json!({"prior_year": "2024-25", "prior_days": 183})),
        "zone_tax_offset.prior_year_invalid"
    );
    assert_eq!(
        refused(json!({"prior_year": "2024-25"})),
        "input.field_unknown"
    );
    assert_eq!(refused(json!({"prior_days": 100})), "input.field_unknown");
    assert_eq!(
        refused(json!({"continuous_residence": true})),
        "input.field_unknown"
    );
    assert_eq!(
        refused(json!({"prior_year": "2024-25", "prior_days": 100, "includes_first_day": true})),
        "input.field_unknown"
    );
    // 2023-24 had 366 days: s 79A(3B)(d)(ii) carries 183 of them, which the
    // ATO counts as residence for 2023-24 itself.
    let leap = zone(
        "2024-25",
        "zone_a",
        1,
        json!({"prior_year": "2023-24", "prior_days": 183}),
    );
    assert_eq!(code(ZONE, &leap), "zone_tax_offset.half_year_unclear");
    // With invalid dependants too, the residence refusal comes first.
    let leap_and_dependants = zone(
        "2024-25",
        "zone_a",
        1,
        json!({"prior_year": "2023-24", "prior_days": 183, "dependants": "none"}),
    );
    assert_eq!(
        code(ZONE, &leap_and_dependants),
        "zone_tax_offset.half_year_unclear"
    );
    let short = zone(
        "2024-25",
        "zone_a",
        1,
        json!({"prior_year": "2023-24", "prior_days": 182}),
    );
    assert_eq!(offset(&short), "338.00");
    // 7 bytes with a character across byte 4 is refused, not a panic.
    assert_eq!(
        refused(json!({"prior_year": "aééaa", "prior_days": 100})),
        "zone_tax_offset.prior_year_invalid"
    );
}

/// T4 table 2 and worksheets 1 to 4: $376 for a student and the oldest
/// non-student child, $282 for each other child; apportioned by days; less
/// $1 for every $4 over $282 without cents; then the zone's percentage.
#[test]
fn zone_base_amounts_from_dependants() {
    let dependants = |list: Value| zone("2025-26", "zone_a", 365, json!({"dependants": list}));
    let student = json!({"kind": "student", "adjusted_taxable_income": "0"});
    let child = json!({"kind": "child", "adjusted_taxable_income": "0"});
    assert_eq!(offset(&dependants(json!([student]))), "526.00");
    let r = run(ZONE, &dependants(json!([child, child])));
    assert_eq!(amount(&r, "relevant_rebate_amount"), "658.00");
    assert_eq!(amount(&r, "tax_offset"), "667.00");
    assert_eq!(r["dependants"][1]["full_amount"], "282");
    // Students are $376 each, and a single non-student child is the oldest.
    let r = run(ZONE, &dependants(json!([student, child, student])));
    assert_eq!(amount(&r, "relevant_rebate_amount"), "1128.00");
    let zone_b = zone("2025-26", "zone_b", 365, json!({"dependants": [child]}));
    assert_eq!(amount(&run(ZONE, &zone_b), "full_amount"), "132.20");
    assert_eq!(offset(&zone_b), "132.00");
    let earning = json!({"kind": "child", "adjusted_taxable_income": "1000"});
    let r = run(ZONE, &dependants(json!([earning])));
    assert_eq!(amount(&r, "relevant_rebate_amount"), "197.00");
    assert_eq!(amount(&r, "tax_offset"), "436.00");
    let part = json!({"kind": "child", "adjusted_taxable_income": "0", "days": 100});
    let r = run(ZONE, &dependants(json!([part])));
    assert_eq!(amount(&r, "relevant_rebate_amount"), "103.01");
    assert_eq!(amount(&r, "tax_offset"), "389.00");
    let high = json!({"kind": "student", "adjusted_taxable_income": "2286"});
    assert_eq!(
        amount(
            &run(ZONE, &dependants(json!([high]))),
            "relevant_rebate_amount"
        ),
        "0.00"
    );
}

#[test]
fn zone_sole_parent_invalid_carer_and_remote_area_allowance() {
    let child = json!({"kind": "child", "adjusted_taxable_income": "0"});
    let full = zone(
        "2025-26",
        "zone_a",
        365,
        json!({"dependants": [child], "sole_parent_days": 365}),
    );
    assert_eq!(offset(&full), "1329.00");
    let part = zone(
        "2025-26",
        "zone_a",
        365,
        json!({"dependants": [child], "sole_parent_days": 100}),
    );
    assert_eq!(
        amount(&run(ZONE, &part), "sole_parent_notional_tax_offset"),
        "440.00"
    );
    let leap = zone(
        "2023-24",
        "zone_a",
        366,
        json!({"dependants": [child], "sole_parent_days": 100}),
    );
    assert_eq!(
        amount(&run(ZONE, &leap), "sole_parent_notional_tax_offset"),
        "439.00"
    );
    let none = zone("2025-26", "zone_a", 365, json!({"sole_parent_days": 365}));
    assert_eq!(
        code(ZONE, &none),
        "zone_tax_offset.sole_parent_without_dependant"
    );
    let carer = zone(
        "2025-26",
        "zone_a",
        365,
        json!({"invalid_carer_tax_offset": "3396"}),
    );
    assert_eq!(offset(&carer), "2036.00");
    let allowance = |a: &str| {
        offset(&zone(
            "2025-26",
            "zone_a",
            365,
            json!({"remote_area_allowance": a}),
        ))
    };
    assert_eq!(allowance("100"), "238.00");
    assert_eq!(allowance("400"), "0.00");
}

/// T4 worksheet 6 and the Sharon example: overseas service of not more than
/// one-half of the year gives the amount x days / 183, without cents.
#[test]
fn overseas_forces_part_year() {
    let r = run(ZONE, &zone("2025-26", "overseas_forces", 100, json!({})));
    assert_eq!(r["fraction"], "100/183");
    assert_eq!(amount(&r, "tax_offset"), "184.00");
    assert_eq!(
        offset(&zone("2025-26", "overseas_forces", 183, json!({}))),
        "338.00"
    );
    assert_eq!(
        offset(&zone("2023-24", "overseas_forces", 183, json!({}))),
        "338.00"
    );
    assert_eq!(
        offset(&zone("2025-26", "overseas_forces", 0, json!({}))),
        "0.00"
    );
    let child = json!({"kind": "student", "adjusted_taxable_income": "0"});
    let with_base = zone(
        "2025-26",
        "overseas_forces",
        100,
        json!({"dependants": [child]}),
    );
    assert_eq!(offset(&with_base), "287.00");
    for extra in [
        json!({"remote_area_allowance": "1"}),
        json!({"prior_year": "2024-25", "prior_days": 100}),
    ] {
        let facts = zone("2025-26", "overseas_forces", 100, extra);
        assert_eq!(code(ZONE, &facts), "input.field_unknown");
    }
}

#[test]
fn zone_input_refusals() {
    assert_eq!(
        code(ZONE, &zone("2025-26", "zone_c", 365, json!({}))),
        "zone_tax_offset.area_invalid"
    );
    assert_eq!(
        code(ZONE, &zone("2025-26", "zone_a", 366, json!({}))),
        "input.day_count_invalid"
    );
    assert_eq!(
        code(ZONE, &zone("2019-20", "zone_a", 365, json!({}))),
        "period.unsupported"
    );
    for bad in [
        json!("x"),
        json!([1]),
        json!([{"kind": "baby", "adjusted_taxable_income": "0"}]),
        json!([{"kind": "child", "adjusted_taxable_income": 0}]),
        json!([{"kind": "child", "adjusted_taxable_income": "0", "days": 400}]),
        json!([{"kind": "child", "adjusted_taxable_income": "0", "age": 3}]),
        json!([{"kind": "child", "adjusted_taxable_income": "0", "shared": "no"}]),
    ] {
        let facts = zone("2025-26", "zone_a", 365, json!({"dependants": bad}));
        assert_eq!(
            code(ZONE, &facts),
            "zone_tax_offset.dependants_invalid",
            "{facts}"
        );
    }
    let many: Vec<Value> = (0..51)
        .map(|_| json!({"kind": "student", "adjusted_taxable_income": "0"}))
        .collect();
    let facts = zone("2025-26", "zone_a", 365, json!({"dependants": many}));
    assert_eq!(code(ZONE, &facts), "zone_tax_offset.dependants_invalid");
    let shared = json!([{"kind": "child", "adjusted_taxable_income": "0", "shared": true}]);
    let facts = zone("2025-26", "zone_a", 365, json!({"dependants": shared}));
    assert_eq!(code(ZONE, &facts), "zone_tax_offset.shared_maintenance");
}

// Net medical expenses tax offset.

const MEDICAL: &str = "net-medical-expenses-tax-offset";

fn medical(year: &str, extra: Value) -> Value {
    let base = json!({"year": year, "scope_confirmed": true,
        "net_medical_expenses": "10000", "adjusted_taxable_income": "90000"});
    with(base, extra)
}

fn medical_offset(facts: &Value) -> String {
    amount(&run(MEDICAL, facts), "net_medical_expenses_tax_offset").to_string()
}

/// The ATO's T5 tables 1 and 2 for 2017, 2018 and 2019: 20% over the lower
/// limit, or 10% over the higher limit above $90,000 single or $180,000 plus
/// $1,500 for each child after the first.
#[test]
fn net_medical_expenses_limits_by_year() {
    for (year, lower, higher) in [
        ("2016-17", "1540.20", "457.70"),
        ("2017-18", "1533.40", "449.60"),
        ("2018-19", "1524.60", "439.10"),
    ] {
        assert_eq!(medical_offset(&medical(year, json!({}))), lower, "{year}");
        let over = medical(year, json!({"adjusted_taxable_income": "90001"}));
        assert_eq!(medical_offset(&over), higher, "{year}");
    }
}

#[test]
fn net_medical_expenses_family_thresholds() {
    let married = |spouse: &str| {
        medical(
            "2018-19",
            json!({"adjusted_taxable_income": "100000",
            "spouse_adjusted_taxable_income": spouse}),
        )
    };
    assert_eq!(medical_offset(&married("80000")), "1524.60");
    assert_eq!(medical_offset(&married("80001")), "439.10");
    let children = |income: &str, n: i64| {
        medical(
            "2018-19",
            json!({"adjusted_taxable_income": income, "dependent_children": n}),
        )
    };
    assert_eq!(medical_offset(&children("183000", 3)), "1524.60");
    assert_eq!(medical_offset(&children("183001", 3)), "439.10");
    assert_eq!(medical_offset(&children("180000", 1)), "1524.60");
    assert_eq!(medical_offset(&children("180001", 1)), "439.10");
    let r = run(MEDICAL, &children("100000", 0));
    assert_eq!(r["thresholds"], "singles");
    assert_eq!(r["higher_limit_applies"], true);
    let near =
        |net: &str| medical_offset(&medical("2018-19", json!({"net_medical_expenses": net})));
    assert_eq!(near("2377"), "0.00");
    assert_eq!(near("2378"), "0.20");
    assert_eq!(near("0"), "0.00");
}

#[test]
fn net_medical_expenses_refusals() {
    for year in ["2019-20", "2020-21", "2025-26"] {
        let facts = medical(year, json!({}));
        let refused = api::evaluate_coded(MEDICAL, &facts).unwrap_err();
        assert_eq!(refused.code, "period.unsupported");
        assert!(refused.message.contains("159P(1A)"), "{year}");
    }
    assert_eq!(
        code(MEDICAL, &medical("2015-16", json!({}))),
        "period.unsupported"
    );
    assert_eq!(
        code(MEDICAL, &medical("aééaa", json!({}))),
        "period.unsupported"
    );
    assert_eq!(
        code(
            MEDICAL,
            &medical("2018-19", json!({"dependent_children": -1}))
        ),
        "input.dependants_invalid"
    );
    assert_eq!(
        code(
            MEDICAL,
            &medical("2018-19", json!({"gross_medical_expenses": "1"}))
        ),
        "input.field_unknown"
    );
}

/// Each command needs its scope confirmed.
#[test]
fn every_command_needs_scope_confirmed() {
    for (command, facts) in [
        (
            BENEFICIARY,
            json!({"year": "2025-26", "rebatable_benefit": "1"}),
        ),
        (CARER, carer("2025-26", json!({}))),
        (ZONE, zone("2025-26", "zone_a", 365, json!({}))),
        (MEDICAL, medical("2018-19", json!({}))),
    ] {
        let facts = with(facts, json!({"scope_confirmed": false}));
        assert_eq!(code(command, &facts), "scope.not_confirmed", "{command}");
    }
}
