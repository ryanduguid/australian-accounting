// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The Medicare levy at the SAPTO thresholds and with the family reduction
//! (Medicare Levy Act 1986 ss 3(1), 7 and 8): the ATO's Ashton example, each
//! threshold and the top of each shade-in, the sharing between spouses
//! (s 8(3) and (4)) and the refusals
//! (docs/calculation-evidence.md#medicare-levy). Every figure is the Act's or
//! the ATO's, worked by hand in the comments.
use australian_tax_calculators::api;
use australian_tax_calculators::engine::worksheet_catalogue;
use serde_json::{Value, json};

const COMMAND: &str = "medicare-levy";

fn input(income: &str, year: &str, family: Value) -> Value {
    let mut facts = json!({"taxable_income": income, "section_9a_adjustment_applies": false,
                           "year": year, "scope_confirmed": true});
    for (k, v) in family.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn evaluate(income: &str, year: &str, family: Value) -> Value {
    api::evaluate(COMMAND, &input(income, year, family)).unwrap()
}

fn levy(income: &str, year: &str, family: Value) -> String {
    let result = evaluate(income, year, family);
    result["amounts"]["medicare_levy"]
        .as_str()
        .unwrap()
        .to_string()
}

fn refusal(income: &str, family: Value) -> (&'static str, String) {
    let refused = api::evaluate_coded(COMMAND, &input(income, "2025-26", family)).unwrap_err();
    (refused.code, refused.message)
}

/// A spouse with nil taxable income: family income is the person's own, and
/// the spouse is not liable to the levy, so s 8(3) does not share the reduction.
fn idle_spouse() -> Value {
    json!({"spouse_taxable_income": "0", "spouse_sapto_entitled": false})
}

/// ATO, Medicare levy reduction - family income (last updated 30 June 2026),
/// read on 14 November 2026: Ashton (68, SAPTO) has taxable income of $49,700
/// and his spouse $21,700 in 2025-26, both entitled to SAPTO, with no
/// dependent children; his levy is reduced to $92.90.
#[test]
fn ato_ashton_example_2025_26() {
    let result = evaluate(
        "49700",
        "2025-26",
        json!({"sapto_entitled": true, "spouse_taxable_income": "21700",
               "spouse_sapto_entitled": true}),
    );
    // s 7(2) at the SAPTO threshold amount: 10% x (49,700 - 44,268) = 543.20 < 994.00.
    // s 8(2): 2% x 61,623 - 0.08 x (71,400 - 61,623) = 1,232.46 - 782.16 = 450.30,
    // not shared: the spouse's $21,700 is below the spouse's threshold amount.
    assert_eq!(
        result["amounts"],
        json!({"taxable_income_used": "49700.00", "levy_at_full_rate": "994.00",
               "low_income_reduction": "450.80", "family_income": "71400.00",
               "family_reduction": "450.30", "spouse_excess_reduction": "0.00",
               "medicare_levy": "92.90"})
    );
    assert_eq!(result["rates"]["threshold_amount"], "44268");
    assert_eq!(result["rates"]["phase_in_limit"], "55335");
    assert_eq!(result["rates"]["family_income_threshold"], "61623");
    assert_eq!(result["rates"]["family_shade_out_rate"], "0.08");
    assert_eq!(result["warnings"].as_array().unwrap().len(), 2);
}

/// The same facts in the other years, worked from the Act: 2026-27 has the
/// 2025-26 figures (Act No. 58, 2026, Sch 5 item 14); 2024-25 has $43,020 and
/// $59,886 (Act No. 28, 2025, Sch 2): 10% x 6,680 = 668.00, less
/// 2% x 59,886 - 0.08 x 11,514 = 1,197.72 - 921.12 = 276.60.
#[test]
fn ashton_facts_in_2024_25_and_2026_27() {
    let facts = || {
        json!({"sapto_entitled": true, "spouse_taxable_income": "21700",
               "spouse_sapto_entitled": true})
    };
    assert_eq!(levy("49700", "2024-25", facts()), "391.40");
    let later = evaluate("49700", "2026-27", facts());
    assert_eq!(later["amounts"]["medicare_levy"], "92.90");
    let warnings = later["warnings"].as_array().unwrap();
    assert!(
        warnings[2]
            .as_str()
            .unwrap()
            .starts_with("2026-27 uses the threshold amounts, phase-in limits and family income")
    );
}

/// s 3(1) SAPTO thresholds for a person without a spouse or dependants: nil
/// at and below the threshold amount, 10% of the excess up to the phase-in
/// limit (which is 1.25 x the threshold, where 10% of the excess meets 2% of
/// the whole), and 2% above it.
#[test]
fn sapto_thresholds_in_each_year() {
    let sapto = || json!({"sapto_entitled": true});
    for (year, at, limit) in [
        ("2024-25", 43020, 53775),
        ("2025-26", 44268, 55335),
        ("2026-27", 44268, 55335),
    ] {
        let run = |income: i64| levy(&income.to_string(), year, sapto());
        assert_eq!(run(at - 1), "0.00", "{year}");
        assert_eq!(run(at), "0.00", "{year}");
        assert_eq!(run(at + 1), "0.10", "{year}");
        let shaded = |income: i64| format!("{}.{}0", (income - at) / 10, (income - at) % 10);
        assert_eq!(run(limit - 1), shaded(limit - 1), "{year}");
        assert_eq!(run(limit), shaded(limit), "{year}");
        let full = |income: i64| format!("{}.{:02}", income * 2 / 100, income * 2 % 100);
        assert_eq!(
            run(limit),
            full(limit),
            "{year}: 10% x excess = 2% x income"
        );
        assert_eq!(run(limit + 1), full(limit + 1), "{year}");
        let result = evaluate(&at.to_string(), year, sapto());
        assert_eq!(result["rates"]["threshold_amount"], at.to_string());
        assert_eq!(result["rates"]["phase_in_limit"], limit.to_string());
        assert!(result["amounts"].get("family_income").is_none());
    }
}

/// s 8(1) and (2) at each family income threshold, with a spouse who has no
/// income or as a sole parent: nil at and below it, and above it 2% of the
/// threshold less 8% of the excess, which is 0.10 short of the full levy one
/// dollar above it and reaches nil where family income passes 1.25 x the
/// threshold. The top of each shade-in, rounded down, is the ATO's upper
/// threshold: $57,383 and $74,857 for 2024-25 (myTax 2025, worksheet 2) and
/// $59,047 and $77,028 for 2025-26.
#[test]
fn family_thresholds_and_the_top_of_each_shade_in() {
    // (year, SAPTO, children, threshold, top of the shade-in, levy at the top)
    for (year, sapto, children, threshold, top, at_top) in [
        ("2024-25", false, 0, 45907, 57383, "1147.60"), // 1,147.66 - (918.14 - 918.08)
        ("2024-25", true, 0, 59886, 74857, "1497.10"),  // 1,497.14 - (1,197.72 - 1,197.68)
        ("2024-25", false, 1, 50123, 62653, "1253.00"), // 1,253.06 - (1,002.46 - 1,002.40)
        ("2025-26", false, 0, 47238, 59047, "1180.90"), // 1,180.94 - (944.76 - 944.72)
        ("2025-26", true, 0, 61623, 77028, "1540.50"),  // 1,540.56 - (1,232.46 - 1,232.40)
        ("2025-26", false, 2, 55914, 69892, "1397.80"), // 1,397.84 - (1,118.28 - 1,118.24)
        ("2026-27", false, 0, 47238, 59047, "1180.90"),
        ("2026-27", true, 3, 74637, 93296, "1865.90"), // 1,865.92 - (1,492.74 - 1,492.72)
    ] {
        for basis in [
            json!({"spouse_taxable_income": "0", "spouse_sapto_entitled": false}),
            json!({"dependant_offset_entitled": true}),
        ] {
            let mut family = basis;
            family["sapto_entitled"] = json!(sapto);
            family["dependent_children"] = json!(children);
            let run = |income: i64| levy(&income.to_string(), year, family.clone());
            let case = format!("{year} {family}");
            assert_eq!(run(threshold - 1), "0.00", "{case}");
            assert_eq!(run(threshold), "0.00", "{case}");
            // 2% x (threshold + 1) - (2% x threshold - 0.08 x 1) = 0.02 + 0.08.
            assert_eq!(run(threshold + 1), "0.10", "{case}");
            assert_eq!(run(top), at_top, "{case}");
            let full = (top + 1) * 2;
            assert_eq!(
                run(top + 1),
                format!("{}.{:02}", full / 100, full % 100),
                "{case}"
            );
            let result = evaluate(&top.to_string(), year, family.clone());
            assert_eq!(
                result["rates"]["family_income_threshold"],
                threshold.to_string()
            );
        }
    }
}

/// The ATO's per-child increase in the upper threshold, $5,423 for 2025-26, is
/// 1.25 x $4,338 = $5,422.50 rounded up, so its upper threshold moves 50 cents
/// a child past the family income where the s 8(2) reduction reaches nil. With
/// 2 children, $59,047 + $10,846 = $69,893: 2% x 55,914 - 0.08 x 13,979 =
/// -0.04. With 3, $59,047 + $16,269 = $75,316: 2% x 60,252 - 0.08 x 15,064 =
/// -0.08.
#[test]
fn the_formula_not_the_rounded_upper_threshold_ends_the_reduction() {
    for (children, ato_upper, full_levy) in [(2, "69893", "1397.86"), (3, "75316", "1506.32")] {
        let family = json!({"dependant_offset_entitled": true, "dependent_children": children});
        let result = evaluate(ato_upper, "2025-26", family);
        assert_eq!(result["amounts"]["family_reduction"], "0.00", "{children}");
        assert_eq!(result["amounts"]["medicare_levy"], full_levy, "{children}");
    }
}

/// s 8(3): when the spouse's taxable income exceeds the spouse's threshold
/// amount, each spouse takes the reduction in proportion to taxable income.
/// 2025-26, 3 children: threshold 47,238 + 3 x 4,338 = 60,252, family income
/// 70,000, reduction 1,205.04 - 0.08 x 9,748 = 425.20.
#[test]
fn a_liable_spouse_shares_the_reduction() {
    let family = |spouse: &str| {
        json!({"spouse_taxable_income": spouse, "spouse_sapto_entitled": false,
               "dependent_children": 3})
    };
    // 425.20 x 40/70 = 242.97; 800.00 - 242.97 = 557.03.
    let higher = evaluate("40000", "2025-26", family("30000"));
    assert_eq!(higher["amounts"]["family_reduction"], "242.97");
    assert_eq!(higher["amounts"]["spouse_excess_reduction"], "0.00");
    assert_eq!(higher["amounts"]["medicare_levy"], "557.03");
    // 425.20 x 30/70 = 182.23 of the spouse's 198.90 (10% x 1,989).
    let lower = evaluate("30000", "2025-26", family("40000"));
    assert_eq!(lower["amounts"]["low_income_reduction"], "401.10");
    assert_eq!(lower["amounts"]["family_reduction"], "182.23");
    assert_eq!(lower["amounts"]["medicare_levy"], "16.67");
    for shared in [&higher, &lower] {
        let warnings = shared["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 3);
        assert!(warnings[2].as_str().unwrap().contains("(s 8(3))"));
    }
    // A spouse entitled to SAPTO is not liable at $30,000 (threshold $44,268),
    // so the whole 425.20 reduces this levy: 800.00 - 425.20.
    let mut sapto = family("30000");
    sapto["spouse_sapto_entitled"] = json!(true);
    let unshared = evaluate("40000", "2025-26", sapto);
    assert_eq!(unshared["amounts"]["medicare_levy"], "374.80");
    assert_eq!(unshared["warnings"].as_array().unwrap().len(), 2);
}

/// s 8(3)(c) turns on the spouse's taxable income exceeding the spouse's
/// threshold amount ($28,011 in 2025-26). With 3 children (threshold 60,252):
/// at $28,011 the spouse is not liable and this levy takes the whole
/// 1,205.04 - 0.08 x 7,759 = 584.32 (800.00 - 584.32); a dollar more and the
/// reduction of 584.24 is shared, but the spouse's share above the spouse's
/// $0.10 levy comes back under s 8(4): 800.00 + 0.10 - 584.24.
#[test]
fn the_spouse_threshold_decides_sharing() {
    let family = |spouse: &str| {
        json!({"spouse_taxable_income": spouse, "spouse_sapto_entitled": false,
               "dependent_children": 3})
    };
    assert_eq!(levy("40000", "2025-26", family("28011")), "215.68");
    let shared = evaluate("40000", "2025-26", family("28012"));
    assert_eq!(shared["amounts"]["medicare_levy"], "215.86");
    assert_eq!(shared["amounts"]["spouse_excess_reduction"], "240.53");
}

/// s 8(4): the part of the spouse's shared reduction above the spouse's own
/// levy reduces this levy. 2025-26, 3 children, $40,000 and $29,000: family
/// income 69,000, reduction 1,205.04 - 0.08 x 8,748 = 505.20; the spouse's
/// share 505.20 x 29/69 = 212.33 exceeds the spouse's 98.90 by 113.43, so this
/// levy is 800.00 - 505.20 x 40/69 - 113.43 = 800.00 - 505.20 + 98.90 = 393.70,
/// and the spouse pays nil.
#[test]
fn the_spouse_excess_reduces_this_levy() {
    let family = |spouse: &str| {
        json!({"spouse_taxable_income": spouse, "spouse_sapto_entitled": false,
               "dependent_children": 3})
    };
    let result = evaluate("40000", "2025-26", family("29000"));
    assert_eq!(result["amounts"]["family_reduction"], "292.87");
    assert_eq!(result["amounts"]["spouse_excess_reduction"], "113.43");
    assert_eq!(result["amounts"]["medicare_levy"], "393.70");
    let spouse = evaluate("29000", "2025-26", family("40000"));
    assert_eq!(spouse["amounts"]["family_reduction"], "98.90");
    assert_eq!(spouse["amounts"]["spouse_excess_reduction"], "0.00");
    assert_eq!(spouse["amounts"]["medicare_levy"], "0.00");
}

/// The spouse's excess uses the spouse's own thresholds. 2025-26, no children:
/// family income 75,000 is above 1.25 x $47,238, so this person (no SAPTO) has
/// no reduction of their own. A spouse entitled to SAPTO on $45,000 has
/// 1,232.46 - 0.08 x 13,377 = 162.30, shared 162.30 x 45/75 = 97.38, against
/// a levy of 10% x 732 = 73.20; the 24.18 above it reduces this person's
/// 10% x 1,989 = 198.90 to 174.72. Without SAPTO the spouse has no reduction.
#[test]
fn the_spouse_excess_follows_the_spouse_sapto_thresholds() {
    let family =
        |sapto: bool| json!({"spouse_taxable_income": "45000", "spouse_sapto_entitled": sapto});
    let result = evaluate("30000", "2025-26", family(true));
    assert_eq!(result["amounts"]["family_reduction"], "0.00");
    assert_eq!(result["amounts"]["spouse_excess_reduction"], "24.18");
    assert_eq!(result["amounts"]["medicare_levy"], "174.72");
    assert_eq!(levy("30000", "2025-26", family(false)), "198.90");
}

/// A person at or below their own threshold amount pays nothing under s 7(1)
/// whatever the family facts, and is not liable, so the spouse's reduction is
/// not shared with them (s 8(3)(c)) and none of it passes to them (s 8(4)).
#[test]
fn a_person_below_their_threshold_pays_nil() {
    let family = json!({"spouse_taxable_income": "40000", "spouse_sapto_entitled": false,
                        "dependent_children": 3});
    let result = evaluate("28011", "2025-26", family);
    assert_eq!(result["amounts"]["family_reduction"], "0.00");
    assert_eq!(result["amounts"]["spouse_excess_reduction"], "0.00");
    assert_eq!(result["amounts"]["medicare_levy"], "0.00");
}

/// Omitting the optional inputs and giving their defaults give the same result.
#[test]
fn defaults_equal_omissions() {
    let omitted = evaluate("30000", "2025-26", json!({}));
    let stated = evaluate(
        "30000",
        "2025-26",
        json!({"sapto_entitled": false, "dependant_offset_entitled": false,
               "dependent_children": 0}),
    );
    assert_eq!(omitted, stated);
    assert_eq!(omitted["amounts"]["medicare_levy"], "198.90");
    // A spouse with nil income still brings in s 8: below the family income
    // threshold the levy is nil.
    assert_eq!(levy("30000", "2025-26", idle_spouse()), "0.00");
}

#[test]
fn inputs_the_worksheet_cannot_settle_are_refused() {
    let field = "input.field_unknown";
    for (family, code, message) in [
        (
            json!({"spouse_sapto_entitled": true}),
            field,
            "spouse_sapto_entitled is not an input of medicare-levy without spouse_taxable_income",
        ),
        (
            json!({"spouse_taxable_income": "1000", "spouse_sapto_entitled": false,
                   "dependant_offset_entitled": true}),
            field,
            "dependant_offset_entitled is not an input of medicare-levy with spouse_taxable_income",
        ),
        (
            json!({"dependent_children": 2}),
            field,
            "dependent_children is not an input of medicare-levy without a spouse unless dependant_offset_entitled is true",
        ),
        (
            json!({"dependent_children": 1, "dependant_offset_entitled": false}),
            field,
            "dependent_children is not an input of medicare-levy without a spouse unless dependant_offset_entitled is true",
        ),
        (
            json!({"spouse_income": "1000"}),
            field,
            "spouse_income is not an input of medicare-levy",
        ),
        (
            json!({"spouse_taxable_income": "1000"}),
            "input.missing_field",
            "spouse_sapto_entitled is required with spouse_taxable_income",
        ),
        (
            json!({"spouse_taxable_income": "1000.50", "spouse_sapto_entitled": false}),
            "input.amount_not_whole_dollars",
            "Supply established whole-dollar spouse taxable income.",
        ),
        (
            json!({"spouse_taxable_income": "-1", "spouse_sapto_entitled": false}),
            "input.amount_invalid",
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        ),
        (
            json!({"sapto_entitled": "yes"}),
            "input.flag_invalid",
            "sapto_entitled must be a boolean.",
        ),
        (
            json!({"spouse_taxable_income": "1000", "spouse_sapto_entitled": 1}),
            "input.flag_invalid",
            "spouse_sapto_entitled must be a boolean.",
        ),
        (
            json!({"dependant_offset_entitled": null}),
            "input.flag_invalid",
            "dependant_offset_entitled must be a boolean.",
        ),
    ] {
        let (got, text) = refusal("40000", family.clone());
        assert_eq!((got, text.as_str()), (code, message), "{family}");
    }
    for children in [json!(-1), json!("2"), json!(1.5), json!(true), json!(null)] {
        let family = json!({"dependant_offset_entitled": true, "dependent_children": children});
        assert_eq!(
            refusal("40000", family),
            (
                "input.dependants_invalid",
                "dependent_children must be a non-negative integer.".to_string()
            )
        );
    }
    // s 9A is still refused first.
    let mut facts = input("40000", "2025-26", idle_spouse());
    facts["section_9a_adjustment_applies"] = json!(true);
    let refused = api::evaluate_coded(COMMAND, &facts).unwrap_err();
    assert_eq!(refused.code, "medicare.s9a_adjustment_out_of_scope");
}

/// Every optional input the catalogue lists is read: a value of the wrong
/// type for it is refused rather than ignored.
#[test]
fn every_catalogued_optional_input_is_read() {
    let catalogue = worksheet_catalogue();
    let optional = catalogue["medicare_levy"]["optional_inputs"]
        .as_object()
        .unwrap();
    assert_eq!(optional.len(), 5);
    for key in optional.keys() {
        let mut family = json!({});
        family[key] = json!([]);
        assert!(
            api::evaluate(COMMAND, &input("40000", "2025-26", family)).is_err(),
            "{key}"
        );
    }
}
