// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The franking account against ITAA 1997 Division 205 and the ATO's
//! Franking account tax return 2026 instructions, read on 19 November 2026
//! (docs/calculation-evidence.md#franking-account): examples 2, 4, 5 and 7 to
//! 9, the s 205-70(2) note's example, hand-worked cases of the offset
//! exceptions, the R&D deferral, refunds after the year's end and the
//! benchmark rule, and the refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "franking-account";

/// A public company with no balance, events or distributions in 2025-26 at
/// 30%, which `overrides` fills in.
fn input(overrides: Value) -> Value {
    let mut facts = json!({
        "private_company": false, "benchmark_rule_exempt": false,
        "corporate_tax_rate": "0.30", "opening_balance": "0.00",
        "rd_deferred_debit_brought_forward": "0.00", "last_benchmark": null,
        "events": [], "distributions": [], "year": "2025-26", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn run(overrides: Value) -> Value {
    api::evaluate(COMMAND, &input(overrides)).unwrap()
}

fn code(overrides: Value) -> &'static str {
    api::evaluate_coded(COMMAND, &input(overrides))
        .unwrap_err()
        .code
}

fn ev(date: &str, kind: &str, amount: &str) -> Value {
    json!({"date": date, "type": kind, "amount": amount})
}

fn refund(date: &str, amount: &str, rd: &str) -> Value {
    json!({"date": date, "type": "income_tax_refund", "amount": amount, "rd_offset_part": rd})
}

fn dist(date: &str, amount: &str, credit: &str) -> Value {
    json!({"date": date, "amount": amount, "franking_credit": credit})
}

fn amount(r: &Value, key: &str) -> String {
    r["amounts"][key]
        .as_str()
        .unwrap_or_else(|| panic!("no {key}"))
        .to_string()
}

/// Section B, example 4: EKW Ltd opens 2025-26 at $10,000, pays $500,000 of
/// PAYG instalments, franks $510,000 of distributions, gets a $60,000 refund
/// and a $20,000 item 7 streaming debit. FDT $80,000; the item 1 and 2 part
/// of the deficit is $60,000, more than 10% of the credits, so the offset is
/// $80,000 less 30% of $60,000, $62,000. A reduction of the whole FDT by 30%
/// would give $56,000; the item 7 debit is left out of the part reduced.
fn ekw() -> Value {
    input(json!({
        "opening_balance": "10000.00",
        "events": [
            ev("2025-10-28", "payg_instalment", "125000.00"),
            refund("2025-11-15", "60000.00", "0.00"),
            ev("2026-02-28", "payg_instalment", "125000.00"),
            ev("2026-03-31", "streaming_determination", "20000.00"),
            ev("2026-04-28", "payg_instalment", "250000.00"),
        ],
        // $1,190,000 franked at 30% carries $510,000, its maximum.
        "distributions": [dist("2025-09-30", "1190000.00", "510000.00")],
    }))
}

#[test]
fn ekw_example_4_reduces_only_the_attributable_part() {
    let r = api::evaluate(COMMAND, &ekw()).unwrap();
    assert_eq!(r["calculation"], "franking_account");
    assert_eq!(amount(&r, "credits"), "500000.00");
    assert_eq!(amount(&r, "closing_balance"), "-80000.00");
    assert_eq!(amount(&r, "franking_deficit_tax"), "80000.00");
    assert_eq!(amount(&r, "fdt_offset_reduction"), "18000.00");
    assert_eq!(amount(&r, "offsetable_fdt"), "62000.00");
    assert_eq!(amount(&r, "debits_not_subject_to_reduction"), "20000.00");
    assert_eq!(amount(&r, "franking_tax_payable"), "80000.00");
    assert_eq!(r["rates"]["balance_code"], "D");
    assert_eq!(r["rates"]["offset_code"], "none");
    assert_eq!(r["rates"]["return_required"], "true");
    assert_eq!(r["rates"]["return_due"], "2026-07-31");
    assert_eq!(r["return_labels"]["C"], "62000.00");
    let ledger = r["ledger"].as_array().unwrap();
    assert_eq!(ledger.len(), 6);
    assert_eq!(ledger[0]["item"], "ITAA 1997 s 205-30(1) item 1");
    assert_eq!(ledger[0]["debit"], "510000.00");
    assert_eq!(ledger[0]["balance"], "-500000.00");
    assert_eq!(ledger[5]["balance"], "-80000.00");
}

#[test]
fn catalogue_example_is_ekw() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["franking_account"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(amount(&r, "offsetable_fdt"), "62000.00");
}

/// Without an item 1, 3, 5 or 6 debit the offset is the whole FDT (code F)
/// and the item 2 debits go to label K beside the item 7 one.
#[test]
fn no_trigger_debit_gives_the_whole_offset() {
    let r = run(json!({
        "opening_balance": "10000.00",
        "events": [
            ev("2025-10-28", "payg_instalment", "50000.00"),
            refund("2025-11-15", "60000.00", "0.00"),
            ev("2026-03-31", "streaming_determination", "20000.00"),
        ],
    }));
    assert_eq!(amount(&r, "franking_deficit_tax"), "20000.00");
    assert_eq!(amount(&r, "offsetable_fdt"), "20000.00");
    assert_eq!(amount(&r, "debits_not_subject_to_reduction"), "80000.00");
    assert_eq!(r["rates"]["offset_code"], "F");
}

/// A deficit whose attributable part is at most 10% of the credits keeps the
/// whole offset: $500,000 of credits and a $50,000 part, exactly 10%.
#[test]
fn ten_percent_or_less_is_not_reduced() {
    let r = run(json!({
        "events": [ev("2025-10-28", "payg_instalment", "500000.00")],
        "distributions": [dist("2025-09-30", "1283333.34", "550000.00")],
    }));
    assert_eq!(amount(&r, "franking_deficit_tax"), "50000.00");
    assert_eq!(amount(&r, "offsetable_fdt"), "50000.00");
    assert_eq!(amount(&r, "fdt_offset_reduction"), "0.00");
}

/// Section B, example 5: Salomon Pty Ltd's benchmark is 40% from an earlier
/// distribution; a $500 distribution franked to 100% at 30% gives
/// over-franking tax of $500 x (60% / 2.3333), $128.57 ($128 on the return).
/// The example's $214 is 99.87% of the $214.2857 maximum under s 203-35, so
/// the test allocates $214.29, which is capped at the maximum (s 202-65).
#[test]
fn salomon_example_5_over_franking_tax() {
    let r = run(json!({
        "private_company": true,
        "opening_balance": "1000.00",
        "distributions": [
            dist("2025-08-01", "700.00", "120.00"),
            dist("2026-03-01", "500.00", "214.29"),
        ],
    }));
    assert_eq!(amount(&r, "over_franking_tax"), "128.57");
    assert_eq!(r["rates"]["gross_up_rate"], "2.3333");
    let second = &r["distributions"][1];
    assert_eq!(second["franking_percentage"], "100.00");
    assert_eq!(second["franking_credit"], "214.29");
    assert_eq!(second["maximum_franking_credit"], "214.29");
    assert_eq!(second["compared_with"], "40.00");
    assert_eq!(r["franking_periods"].as_array().unwrap().len(), 1);
    assert_eq!(r["franking_periods"][0]["benchmark_percentage"], "40.00");
    // The debit is the capped credit, so the balance is 1000 - 120 - 214.2857.
    assert_eq!(amount(&r, "closing_balance"), "665.71");
    assert_eq!(r["rates"]["return_required"], "true");
    assert_eq!(amount(&r, "franking_tax_payable"), "128.57");
}

/// s 203-50(1)(b): a distribution franked below the period's benchmark gives
/// an item 3 debit of the distribution times the differential over the
/// gross-up rate, here $700 x 50% / (70/30) = $150, beside its item 1 debit.
#[test]
fn under_franking_debits_the_account() {
    let r = run(json!({
        "opening_balance": "1000.00",
        "distributions": [
            dist("2025-08-01", "700.00", "300.00"),
            dist("2025-11-01", "700.00", "150.00"),
            dist("2026-02-01", "700.00", "0.00"),
        ],
    }));
    assert_eq!(r["distributions"][1]["under_franking_debit"], "150.00");
    // The third is the first of the second franking period: its benchmark.
    assert_eq!(r["distributions"][2]["under_franking_debit"], "0.00");
    assert_eq!(r["franking_periods"][1]["benchmark_percentage"], "0.00");
    let items: Vec<&str> = r["ledger"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["item"].as_str().unwrap())
        .collect();
    assert!(items[2].starts_with("ITAA 1997 s 205-30(1) item 3"));
    assert_eq!(amount(&r, "closing_balance"), "400.00");
    // 100% then 0% across consecutive periods is a significant variation.
    assert_eq!(r["rates"]["significant_variation"], "true");
}

/// s 203-20: an exempt listed company has no over-franking tax, debit or
/// significant variation, though its benchmarks are still shown.
#[test]
fn exempt_entity_has_no_benchmark_consequences() {
    let r = run(json!({
        "benchmark_rule_exempt": true,
        "opening_balance": "1000.00",
        "distributions": [
            dist("2025-08-01", "700.00", "300.00"),
            dist("2025-11-01", "700.00", "150.00"),
            dist("2026-02-01", "700.00", "0.00"),
        ],
    }));
    assert_eq!(amount(&r, "over_franking_tax"), "0.00");
    assert_eq!(r["distributions"][1]["under_franking_debit"], "0.00");
    assert_eq!(r["rates"]["significant_variation"], "false");
    assert_eq!(r["rates"]["return_required"], "false");
    assert_eq!(r["rates"]["return_due"], "none");
}

/// The Commissioner's permitted percentage replaces the benchmark (s 203-50(2)
/// 'franking % differential' (b)(ii); s 203-55(5)).
#[test]
fn permitted_percentage_replaces_the_benchmark() {
    let mut second = dist("2025-11-01", "700.00", "150.00");
    second["permitted_percentage"] = json!("50");
    let r = run(json!({
        "opening_balance": "1000.00",
        "distributions": [dist("2025-08-01", "700.00", "300.00"), second],
    }));
    assert_eq!(r["distributions"][1]["compared_with"], "50.00");
    assert_eq!(r["distributions"][1]["under_franking_debit"], "0.00");
}

/// 'About', example 2: RI Pty Ltd's $45,000 refund is all refundable R&D
/// offset, so it debits nothing; the $30,000 of tax paid the next year
/// credits nothing and the $36,000 the year after credits $21,000.
#[test]
fn ri_example_2_defers_the_rd_debit() {
    let year1 = run(json!({
        "opening_balance": "50000.00",
        "events": [refund("2025-10-01", "45000.00", "45000.00")],
    }));
    assert_eq!(amount(&year1, "debits"), "0.00");
    assert_eq!(
        amount(&year1, "rd_deferred_debit_carried_forward"),
        "45000.00"
    );
    let year2 = run(json!({
        "opening_balance": "50000.00",
        "rd_deferred_debit_brought_forward": "45000.00",
        "events": [ev("2025-12-01", "income_tax_paid", "30000.00")],
    }));
    assert_eq!(amount(&year2, "credits"), "0.00");
    assert_eq!(
        amount(&year2, "rd_deferred_debit_carried_forward"),
        "15000.00"
    );
    let year3 = run(json!({
        "opening_balance": "50000.00",
        "rd_deferred_debit_brought_forward": "15000.00",
        "events": [ev("2025-12-01", "income_tax_paid", "36000.00")],
    }));
    assert_eq!(amount(&year3, "credits"), "21000.00");
    assert_eq!(amount(&year3, "rd_deferred_debit_carried_forward"), "0.00");
}

/// The s 205-70(2) note's example: $100,000 of tax before offsets, an $80,000
/// foreign income tax offset applied first and a $60,000 FDT offset: $20,000
/// of it is used and $40,000 carried forward.
#[test]
fn offset_note_example_is_used_and_carried_forward() {
    let r = run(json!({
        "events": [refund("2025-11-15", "60000.00", "0.00")],
        "income_tax": {"before_fdt_offset": "20000.00", "offset_brought_forward": "0.00"},
    }));
    assert_eq!(amount(&r, "offsetable_fdt"), "60000.00");
    assert_eq!(amount(&r, "fdt_offset_total"), "60000.00");
    assert_eq!(amount(&r, "fdt_offset_used"), "20000.00");
    assert_eq!(amount(&r, "fdt_offset_carried_forward"), "40000.00");
}

/// A private company opening at $55,000 with 4 PAYG instalments of $42,000, a
/// $13,714 franked dividend credit, a $30,000 item 1 debit and a $330,000
/// refund: deficit and FDT $123,286 on $181,714 of credits, offset
/// $123,286 less 30% of $123,286, $86,300.20 (worked by hand).
fn otage(income_tax: Value) -> Value {
    input(json!({
        "private_company": true,
        "opening_balance": "55000.00",
        "events": [
            ev("2025-10-28", "payg_instalment", "42000.00"),
            ev("2026-02-28", "payg_instalment", "42000.00"),
            ev("2026-04-28", "payg_instalment", "42000.00"),
            ev("2026-06-28", "payg_instalment", "42000.00"),
            ev("2025-12-01", "franked_distribution_received", "13714.00"),
            refund("2026-03-15", "330000.00", "0.00"),
        ],
        "distributions": [dist("2025-09-30", "70000.00", "30000.00")],
        "income_tax": income_tax,
    }))
}

#[test]
fn otage_private_company_reduction_needs_and_uses_income_tax() {
    let mut facts = otage(json!(null));
    facts.as_object_mut().unwrap().remove("income_tax");
    let refused = api::evaluate_coded(COMMAND, &facts).unwrap_err();
    assert_eq!(refused.code, "franking_account.income_tax_needed");
    let reduced = api::evaluate(
        COMMAND,
        &otage(json!({"before_fdt_offset": "0.00", "offset_brought_forward": "0.00", "earlier_year_liability": true})),
    )
    .unwrap();
    assert_eq!(amount(&reduced, "credits"), "181714.00");
    assert_eq!(amount(&reduced, "franking_deficit_tax"), "123286.00");
    assert_eq!(amount(&reduced, "offsetable_fdt"), "86300.20");
    assert_eq!(amount(&reduced, "fdt_offset_carried_forward"), "86300.20");
}

/// s 205-70(5): a private company with no earlier income tax liability whose
/// liability without the offset is at least 90% of the deficit keeps the
/// whole offset (code P); at 89.99% it does not.
#[test]
fn private_company_first_liability_keeps_the_whole_offset() {
    let full = api::evaluate(
        COMMAND,
        &otage(json!({"before_fdt_offset": "110957.40", "offset_brought_forward": "0.00", "earlier_year_liability": false})),
    )
    .unwrap();
    assert_eq!(amount(&full, "offsetable_fdt"), "123286.00");
    assert_eq!(full["rates"]["offset_code"], "P");
    assert_eq!(amount(&full, "fdt_offset_used"), "110957.40");
    assert_eq!(amount(&full, "fdt_offset_carried_forward"), "12328.60");
    let short = api::evaluate(
        COMMAND,
        &otage(json!({"before_fdt_offset": "110957.39", "offset_brought_forward": "0.00", "earlier_year_liability": false})),
    )
    .unwrap();
    assert_eq!(amount(&short, "offsetable_fdt"), "86300.20");
}

/// s 205-70(6): the Commissioner's determination keeps the whole offset.
#[test]
fn commissioner_determination_keeps_the_whole_offset() {
    let mut facts = ekw();
    facts["commissioner_determination"] = json!(true);
    let r = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(amount(&r, "offsetable_fdt"), "80000.00");
}

/// s 205-50(2) and s 214-150(4): a refund paid within 3 months after the
/// year's end that puts the account into deficit is taken into the year and
/// its FDT is due 14 days after it when no return was outstanding; one that
/// leaves the account in surplus belongs to the next year.
#[test]
fn refund_after_year_end_is_taken_back_when_it_creates_a_deficit() {
    let r = run(json!({
        "events": [
            ev("2025-10-28", "payg_instalment", "5000.00"),
            refund("2026-08-15", "20000.00", "0.00"),
            refund("2026-09-30", "1000.00", "1000.00"),
        ],
    }));
    assert_eq!(amount(&r, "franking_deficit_tax"), "15000.00");
    assert_eq!(amount(&r, "fdt_from_refunds_after_year_end"), "15000.00");
    let rows = r["refunds_after_year_end"].as_array().unwrap();
    assert_eq!(rows[0]["taken_into_year"], true);
    assert_eq!(rows[0]["due_otherwise"], "2026-08-29");
    assert_eq!(rows[0]["due_with_outstanding_return"], Value::Null);
    assert_eq!(rows[1]["taken_into_year"], false);
}

#[test]
fn refund_more_than_3_months_after_is_refused() {
    assert_eq!(
        code(json!({"events": [refund("2026-10-01", "1000.00", "0.00")]})),
        "franking_account.event_invalid"
    );
}

/// The year opens in deficit: the previous year's FDT credits it on 1 July
/// (s 205-15(1) item 5) and the credit counts in label A.
#[test]
fn opening_deficit_is_credited_by_last_years_fdt() {
    let r = run(json!({"opening_balance": "-5000.00"}));
    assert_eq!(r["ledger"][0]["date"], "2025-07-01");
    assert_eq!(r["ledger"][0]["credit"], "5000.00");
    assert_eq!(amount(&r, "credits"), "5000.00");
    assert_eq!(amount(&r, "closing_balance"), "0.00");
    assert_eq!(r["rates"]["balance_code"], "S");
}

/// Section C, examples 7 to 9: each must disclose a significant variation;
/// a 15 point change after one period need not.
#[test]
fn section_c_examples_7_to_9_disclose() {
    // XYZ Ltd: 50.46% then 100% in its 2 franking periods.
    let xyz = run(json!({
        "opening_balance": "1000.00",
        "distributions": [dist("2025-08-01", "700.00", "151.38"), dist("2026-02-01", "700.00", "300.00")],
    }));
    assert_eq!(xyz["franking_periods"][0]["benchmark_percentage"], "50.46");
    assert_eq!(xyz["franking_periods"][1]["significant_variation"], true);
    // Isanini Ltd: 80% then 50%.
    let isanini = run(json!({
        "opening_balance": "1000.00",
        "distributions": [dist("2025-08-01", "700.00", "240.00"), dist("2026-02-01", "700.00", "150.00")],
    }));
    assert_eq!(isanini["franking_periods"][1]["threshold_points"], 20);
    assert_eq!(isanini["rates"]["significant_variation"], "true");
    // Dombey Pty Ltd: 60% in 2024-25, 30% in 2025-26.
    let dombey = run(json!({
        "private_company": true, "opening_balance": "1000.00",
        "last_benchmark": {"percentage": "60", "periods_since": 1},
        "distributions": [dist("2025-08-01", "700.00", "90.00")],
    }));
    assert_eq!(dombey["rates"]["significant_variation"], "true");
    // Acolmal Pty Ltd: 50% in 2023-24, none in 2024-25, 100% in 2025-26.
    let acolmal = run(json!({
        "private_company": true, "opening_balance": "1000.00",
        "last_benchmark": {"percentage": "50", "periods_since": 2},
        "distributions": [dist("2025-08-01", "700.00", "300.00")],
    }));
    assert_eq!(acolmal["franking_periods"][0]["threshold_points"], 40);
    assert_eq!(acolmal["rates"]["significant_variation"], "true");
    assert_eq!(acolmal["rates"]["return_due"], "2026-07-31");
    let small = run(json!({
        "private_company": true, "opening_balance": "1000.00",
        "last_benchmark": {"percentage": "60", "periods_since": 1},
        "distributions": [dist("2025-08-01", "700.00", "135.00")],
    }));
    assert_eq!(small["rates"]["significant_variation"], "false");
    assert_eq!(small["rates"]["return_required"], "false");
}

/// s 202-75(4): a private company's late statement in a year ending in
/// deficit is flagged, not decided.
#[test]
fn late_statement_in_a_deficit_year_is_flagged() {
    let mut late = dist("2025-09-30", "7000.00", "3000.00");
    late["statement_after_distribution"] = json!(true);
    let r = run(json!({
        "private_company": true,
        "distributions": [late],
        "income_tax": {"before_fdt_offset": "0.00", "offset_brought_forward": "0.00", "earlier_year_liability": true},
    }));
    let warnings = r["warnings"].to_string();
    assert!(warnings.contains("s 202-75(4)"), "{warnings}");
}

#[test]
fn refusals() {
    let cases = [
        (
            json!({"corporate_tax_rate": "0.275"}),
            "franking_account.rate_invalid",
        ),
        (
            json!({"private_company": true, "benchmark_rule_exempt": true}),
            "franking_account.facts_conflict",
        ),
        (
            json!({"events": [ev("2026-07-01", "payg_instalment", "1.00")]}),
            "franking_account.event_invalid",
        ),
        (
            json!({"events": [ev("2025-07-01", "dpt_paid", "1.00")]}),
            "franking_account.event_invalid",
        ),
        (
            json!({"events": [ev("2025-07-01", "income_tax_refund", "1.00")]}),
            "franking_account.event_invalid",
        ),
        (
            json!({"events": [ev("2025-07-01", "payg_instalment", "0.00")]}),
            "franking_account.event_invalid",
        ),
        (
            json!({"events": [{"date": "2025-07-01", "type": "payg_instalment", "amount": "1.00", "note": "x"}]}),
            "input.field_unknown",
        ),
        (
            json!({"distributions": [dist("2025-09-01", "1.00", "0.00"), dist("2025-08-01", "1.00", "0.00")]}),
            "franking_account.distribution_invalid",
        ),
        (
            json!({"distributions": [{"date": "2025-09-01", "amount": "1.00", "franking_credit": "0.00", "permitted_percentage": "50"}]}),
            "franking_account.distribution_invalid",
        ),
        (
            json!({"distributions": [{"date": "2025-09-01", "amount": "1.00", "franking_credit": "0.00", "statement_after_distribution": true}]}),
            "franking_account.distribution_invalid",
        ),
        (
            json!({"last_benchmark": {"percentage": "60"}}),
            "franking_account.last_benchmark_invalid",
        ),
        (
            json!({"last_benchmark": {"percentage": "160", "periods_since": 1}}),
            "franking_account.last_benchmark_invalid",
        ),
        (
            json!({"income_tax": {"before_fdt_offset": "1.00"}}),
            "franking_account.income_tax_invalid",
        ),
        (
            json!({"income_tax": {"before_fdt_offset": "1.00", "offset_brought_forward": "0.00", "earlier_year_liability": false}}),
            "franking_account.income_tax_invalid",
        ),
        (
            json!({"opening_balance": "1.001"}),
            "input.signed_amount_invalid",
        ),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"surplus": "1.00"}), "input.field_unknown"),
    ];
    for (overrides, expected) in cases {
        assert_eq!(code(overrides.clone()), expected, "{overrides}");
    }
}

/// The EKW case without its item 7 debit, which the-exchequer-tally's
/// `franking-account` also takes (its 7 entry types, opening at nil or
/// more): both give a $60,000 deficit and a $42,000 offset, since every debit
/// is then an item 1 or 2 debit and the attributable part is the whole tax.
#[test]
fn agrees_with_the_exchequer_tally_on_its_entry_types() {
    let r = run(json!({
        "opening_balance": "10000.00",
        "events": [
            ev("2025-10-28", "payg_instalment", "500000.00"),
            refund("2025-11-15", "60000.00", "0.00"),
        ],
        "distributions": [dist("2025-09-30", "1190000.00", "510000.00")],
    }));
    assert_eq!(amount(&r, "closing_balance"), "-60000.00");
    assert_eq!(amount(&r, "franking_deficit_tax"), "60000.00");
    assert_eq!(amount(&r, "offsetable_fdt"), "42000.00");
}
