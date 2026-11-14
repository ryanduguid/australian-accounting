// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The seniors and pensioners tax offset (ITAA 1936 s 160AAAA; Income Tax
//! Assessment (1936 Act) Regulations 2025 and Regulation 2015 ss 9 to 12): the
//! ATO's examples and thresholds, the shade-out to nil at each cut-out, the
//! transfer of a spouse's unused amount and the refusals
//! (docs/calculation-evidence.md#seniors-and-pensioners-tax-offset). Every
//! figure is the ATO's or worked from the regulations in the comments.
use australian_tax_calculators::api;
use australian_tax_calculators::engine::worksheet_catalogue;
use serde_json::{Value, json};

const COMMAND: &str = "seniors-pensioners-tax-offset";

fn input(income: &str, status: &str, year: &str, spouse: Value) -> Value {
    let mut facts = json!({"rebate_income": income, "status": status, "year": year,
                           "scope_confirmed": true});
    for (k, v) in spouse.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn evaluate(income: &str, status: &str, year: &str, spouse: Value) -> Value {
    api::evaluate(COMMAND, &input(income, status, year, spouse)).unwrap()
}

fn offset(income: &str, status: &str, year: &str, spouse: Value) -> String {
    evaluate(income, status, year, spouse)["amounts"]["seniors_pensioners_tax_offset"]
        .as_str()
        .unwrap()
        .to_string()
}

/// A member of a couple whose spouse has `rebate_income`.
fn spouse(rebate_income: &str) -> Value {
    json!({"spouse_rebate_income": rebate_income})
}

fn refusal(facts: Value) -> (&'static str, String) {
    let refused = api::evaluate_coded(COMMAND, &facts).unwrap_err();
    (refused.code, refused.message)
}

/// ATO, Seniors and pensioners tax offset (last updated 2 October 2026), read
/// on 14 November 2026: the single examples for 2025-26. Simon ($32,178) is
/// below the $34,919 shading-out threshold and Marko ($85,690) above the
/// $52,759 cut-out. José ($39,000): $2,230 - 0.125 x $4,081 = $1,719.875,
/// which the ATO rounds up to $1,720.
#[test]
fn ato_single_examples_2025_26() {
    let jose = evaluate("39000", "single", "2025-26", json!({}));
    assert_eq!(
        jose["amounts"],
        json!({"rebate_income_used": "39000.00", "rebate_income_tested": "39000.00",
               "base_rebate_amount": "2230.00", "rebate_threshold": "34919.00",
               "cut_out_threshold": "52759.00", "seniors_pensioners_tax_offset": "1720.00"})
    );
    assert_eq!(jose["rates"]["lowest_marginal_tax_rate"], "0.16");
    assert_eq!(jose["rates"]["shade_out_rate"], "0.125");
    assert_eq!(jose["warnings"].as_array().unwrap().len(), 2);
    assert_eq!(offset("32178", "single", "2025-26", json!({})), "2230.00");
    assert_eq!(offset("85690", "single", "2025-26", json!({})), "0.00");
}

/// The same page's couple examples for 2025-26. Each is tested on half the
/// couple's rebate income against the $43,810 cut-out and gets $1,602 less
/// 0.125 of its own rebate income above $30,994.
#[test]
fn ato_couple_examples_2025_26() {
    let couple = |own: &str, other: &str| offset(own, "couple", "2025-26", spouse(other));
    // Clare and Roy: half of $48,697 is $24,348.50; both below $30,994.
    let clare = evaluate("23020", "couple", "2025-26", spouse("25677"));
    assert_eq!(clare["amounts"]["rebate_income_tested"], "24348.50");
    assert_eq!(clare["amounts"]["seniors_pensioners_tax_offset"], "1602.00");
    assert_eq!(couple("25677", "23020"), "1602.00");
    // Deb and Ivan: half of $89,697 is $44,848.50, not below $43,810: neither is entitled.
    assert_eq!(couple("64020", "25677"), "0.00");
    assert_eq!(couple("25677", "64020"), "0.00");
    // Ying and Li Jun: half of $79,697 is $39,848.50; Ying's own $54,020 shades hers to nil.
    assert_eq!(couple("54020", "25677"), "0.00");
    assert_eq!(couple("25677", "54020"), "1602.00");
    // Keith, whose spouse Jean has nil: $1,602 - 0.125 x $2,656 = $1,270.
    let keith = evaluate("33650", "couple", "2025-26", spouse("0"));
    assert_eq!(keith["amounts"]["rebate_income_tested"], "16825.00");
    assert_eq!(keith["amounts"]["seniors_pensioners_tax_offset"], "1270.00");
    // Vanh and Mai: $1,602 - 0.125 x $1,596 = $1,402.50, rounded up to $1,403.
    assert_eq!(couple("32590", "26780"), "1403.00");
    assert_eq!(couple("26780", "32590"), "1602.00");
}

/// The same page's example 9 (2025-26): a member of an illness-separated couple
/// with rebate income of $48,000 takes the unused offset of a spouse with
/// taxable income of $10,000: $2,040 - 0.15 x $4,000 = $1,440. The raised base,
/// $3,480, puts the s 9(5) amount ($42,731.25) above $37,000, so s 9(6) gives
/// $12,572 / 0.315 = $39,911.11, rounded up to $39,912, and the offset is
/// $3,480 - 0.125 x $8,088 = $2,469. The example gives no spouse rebate
/// income; $10,000 assumes only taxable income.
#[test]
fn ato_example_9_transfers_the_spouses_unused_amount() {
    let result = evaluate(
        "48000",
        "illness_separated",
        "2025-26",
        json!({"spouse_rebate_income": "10000", "spouse_taxable_income": "10000"}),
    );
    assert_eq!(
        result["amounts"],
        json!({"rebate_income_used": "48000.00", "rebate_income_tested": "29000.00",
               "base_rebate_amount": "2040.00", "spouse_unused_amount_transferred": "1440.00",
               "rebate_threshold": "39912.00", "cut_out_threshold": "50052.00",
               "seniors_pensioners_tax_offset": "2469.00"})
    );
}

/// Each year's shading-out threshold, cut-out and base amount, and the
/// shade-out to nil: the base amount at the threshold, $1 a dollar below the
/// cut-out (0.125, rounded up) and nil at it. A couple is tested here on its
/// own income, the spouse having the same.
#[test]
fn every_threshold_and_cut_out_for_each_year() {
    // 2025-26: the ATO's table; 2024-25: its shading-out thresholds in "Do you
    // need to lodge a tax return? 2025", reason 2, with the cut-outs of s 10;
    // 2026-27: regulations ss 9(5) and 10 at 15%, as (base + 445) / 0.15 +
    // 18,200 rounded up, plus base / 0.125.
    let at_16 = [(34919, 52759), (33732, 50052), (30994, 43810)];
    let at_15 = [(36034, 53874), (34767, 51087), (31847, 44663)];
    for (year, figures) in [("2024-25", at_16), ("2025-26", at_16), ("2026-27", at_15)] {
        for ((status, base), (threshold, cut_out)) in [
            ("single", "2230.00"),
            ("illness_separated", "2040.00"),
            ("couple", "1602.00"),
        ]
        .into_iter()
        .zip(figures)
        {
            let run = |income: i64| {
                let own = income.to_string();
                let other = if status == "single" {
                    json!({})
                } else {
                    spouse(&own)
                };
                evaluate(&own, status, year, other)
            };
            let result = run(threshold);
            assert_eq!(
                result["amounts"]["rebate_threshold"],
                format!("{threshold}.00")
            );
            assert_eq!(
                result["amounts"]["cut_out_threshold"],
                format!("{cut_out}.00")
            );
            assert_eq!(result["amounts"]["seniors_pensioners_tax_offset"], base);
            let after =
                |income: i64| run(income)["amounts"]["seniors_pensioners_tax_offset"].clone();
            assert_eq!(
                after(threshold + 1),
                base,
                "{year} {status}: 0.125 rounds up"
            );
            assert_eq!(after(cut_out - 1), "1.00", "{year} {status}");
            assert_eq!(after(cut_out), "0.00", "{year} {status}");
        }
    }
}

#[test]
fn only_2026_27_warns_that_its_thresholds_are_worked_from_the_regulations() {
    let later = evaluate("39000", "single", "2026-27", json!({}));
    let warnings = later["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 3);
    assert!(
        warnings[2]
            .as_str()
            .unwrap()
            .contains("15% lowest marginal tax rate")
    );
    // $2,230 - 0.125 x ($39,000 - $36,034) = $1,859.25, rounded up.
    assert_eq!(later["amounts"]["seniors_pensioners_tax_offset"], "1860.00");
    let earlier = evaluate("39000", "single", "2024-25", json!({}));
    assert_eq!(earlier["warnings"].as_array().unwrap().len(), 2);
}

/// Regulations s 12(3)(b): up to $6,000 of the spouse's taxable income the
/// whole base amount passes; at $16,680, $1,602 / 0.15 above $6,000, nothing
/// does. A couple in 2025-26 at $40,000 with all $1,602: s 9(6) gives
/// $12,296 / 0.315 = $39,034.92, rounded up to $39,035, and $3,204 - 0.125 x
/// $965 = $3,083.375, rounded up to $3,084.
#[test]
fn the_transfer_shades_out_with_the_spouses_taxable_income() {
    let transfer = |taxable: &str| {
        let facts = json!({"spouse_rebate_income": taxable, "spouse_taxable_income": taxable});
        evaluate("40000", "couple", "2025-26", facts)["amounts"].clone()
    };
    for taxable in ["0", "6000"] {
        let amounts = transfer(taxable);
        assert_eq!(
            amounts["spouse_unused_amount_transferred"], "1602.00",
            "{taxable}"
        );
        assert_eq!(amounts["rebate_threshold"], "39035.00");
        assert_eq!(amounts["seniors_pensioners_tax_offset"], "3084.00");
    }
    // $1,602 - 0.15 x $1 = $1,601.85.
    assert_eq!(
        transfer("6001")["spouse_unused_amount_transferred"],
        "1601.85"
    );
    let none = transfer("16680");
    assert_eq!(none["spouse_unused_amount_transferred"], "0.00");
    // Without a transfer: $1,602 - 0.125 x $9,006 = $476.25, rounded up.
    assert_eq!(none["rebate_threshold"], "30994.00");
    assert_eq!(none["seniors_pensioners_tax_offset"], "477.00");
}

/// s 12(1)(e): the transfer applies only when this person's own offset is
/// below its base amount, so at the $30,994 threshold nothing passes and a
/// dollar above it the whole $1,602 does. The step is the regulation's: below
/// the threshold the offset already covers the tax.
#[test]
fn the_transfer_starts_when_the_own_offset_falls_below_the_base() {
    let at = |income: &str| {
        let facts = json!({"spouse_rebate_income": "0", "spouse_taxable_income": "0"});
        evaluate(income, "couple", "2025-26", facts)["amounts"].clone()
    };
    let below = at("30994");
    assert_eq!(below["spouse_unused_amount_transferred"], "0.00");
    assert_eq!(below["seniors_pensioners_tax_offset"], "1602.00");
    let above = at("30995");
    assert_eq!(above["spouse_unused_amount_transferred"], "1602.00");
    assert_eq!(above["seniors_pensioners_tax_offset"], "3204.00");
    // Not entitled: half of $87,620 is the $43,810 cut-out, so nothing passes.
    let facts = json!({"spouse_rebate_income": "43810", "spouse_taxable_income": "0"});
    let out = evaluate("43810", "couple", "2025-26", facts);
    assert_eq!(out["amounts"]["spouse_unused_amount_transferred"], "0.00");
    assert_eq!(out["amounts"]["seniors_pensioners_tax_offset"], "0.00");
}

#[test]
fn refusals() {
    let base = input("39000", "single", "2025-26", json!({}));
    let with = |key: &str, value: Value| {
        let mut facts = base.clone();
        facts[key] = value;
        refusal(facts)
    };
    assert_eq!(
        with("scope_confirmed", json!(false)).0,
        "scope.not_confirmed"
    );
    assert_eq!(with("year", json!("2027-28")).0, "period.unsupported");
    assert_eq!(
        with("rebate_income", json!("39000.50")).0,
        "input.amount_not_whole_dollars"
    );
    assert_eq!(with("rebate_income", json!("-1")).0, "input.amount_invalid");
    assert_eq!(with("status", json!("widowed")).0, "input.status_invalid");
    assert_eq!(
        with("spouse_rebate_income", json!("0")),
        (
            "input.field_unknown",
            "spouse_rebate_income is not an input of seniors-pensioners-tax-offset for a single person"
                .to_string()
        )
    );
    assert_eq!(
        with("spouse_taxable_income", json!("0")).0,
        "input.field_unknown"
    );
    assert_eq!(
        with("spouse_sapto_entitled", json!(true)).0,
        "input.field_unknown"
    );
    assert_eq!(
        refusal(input("39000", "couple", "2025-26", json!({}))),
        (
            "input.missing_field",
            "spouse_rebate_income is required unless status is single".to_string()
        )
    );
    let cents = json!({"spouse_rebate_income": "0", "spouse_taxable_income": "1.50"});
    assert_eq!(
        refusal(input("39000", "couple", "2025-26", cents)).0,
        "input.amount_not_whole_dollars"
    );
}

#[test]
fn catalogue_lists_the_worksheet_and_its_example_runs() {
    let catalogue = worksheet_catalogue();
    let entry = &catalogue["seniors_pensioners_tax_offset"];
    let years: Vec<&str> = entry["supported_periods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(years, ["2024-25", "2025-26", "2026-27"]);
    let optional: Vec<&String> = entry["optional_inputs"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(optional, ["spouse_rebate_income", "spouse_taxable_income"]);
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(
        result["amounts"]["seniors_pensioners_tax_offset"],
        "1720.00"
    );
}
