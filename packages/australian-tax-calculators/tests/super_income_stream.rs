// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `super-withholding` with `income_stream_period`: Schedule 13 Part A
//! withholding from one superannuation income stream payment (Schedule 13 of
//! F2024L00664, F2025L00647, F2025L01016 and F2026L00716 and ITAA 1997
//! Divisions 301 and 302, read on 23 November 2026), against each year's
//! Veronica example, each offset row, the offset adjustment's bands, each
//! period, the no-TFN rule and each refusal
//! (docs/calculation-evidence.md#super-withholding).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A resident member's payment with a TFN and the tax-free threshold
/// claimed, whose interest at commencement is the payment itself.
fn stream(year: &str, paid: &str, born: &str, period: &str, parts: [&str; 3]) -> Value {
    let [free, _, untaxed] = parts;
    let total: f64 = parts.iter().map(|p| p.parse::<f64>().unwrap()).sum();
    let payment = format!("{total:.2}");
    json!({
        "benefit": payment, "interest_value": payment,
        "interest_tax_free_component": free, "untaxed_element": untaxed,
        "recipient": "member", "payment_date": paid, "date_of_birth": born,
        "disability_superannuation_benefit": false,
        "tfn_provided": true, "foreign_resident": false,
        "income_stream_period": period, "tax_free_threshold_claimed": true,
        "capped_defined_benefit": false,
        "year": year, "scope_confirmed": true,
    })
}

/// A member aged 66 in 2026-27 paid only an element untaxed in the fund.
fn aged_66(period: &str, untaxed: &str) -> Value {
    stream(
        "2026-27",
        "2026-09-01",
        "1960-01-01",
        period,
        ["0.00", "0.00", untaxed],
    )
}

/// Veronica's fortnightly $1,400: $300 tax free and $1,100 taxed, aged 58.
fn veronica(year: &str, paid: &str, born: &str) -> Value {
    let mut v = stream(
        year,
        paid,
        born,
        "fortnightly",
        ["300.00", "1100.00", "0.00"],
    );
    v["disability_superannuation_benefit"] = json!(true);
    v
}

fn run(input: &Value) -> Value {
    api::evaluate("super-withholding", input).unwrap()
}

fn amount(input: &Value, id: &str) -> String {
    let r = run(input);
    r["amounts"][id]
        .as_str()
        .unwrap_or_else(|| panic!("{id} in {r}"))
        .to_string()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("super-withholding", input)
        .unwrap_err()
        .code
}

/// Each schedule's Case A: Veronica, 58, on a disability super income
/// stream, has $60 (2026-27) or $70 (2024-25, 2025-26) from the fortnightly
/// table, a $165 offset, and the offset adjustment withheld.
#[test]
fn veronica_in_each_year() {
    for (year, paid, born, table, adjustment, withheld) in [
        (
            "2024-25",
            "2024-09-01",
            "1966-01-15",
            "70.00",
            "10.00",
            "10.00",
        ),
        (
            "2025-26",
            "2025-09-01",
            "1967-01-15",
            "70.00",
            "10.00",
            "10.00",
        ),
        (
            "2026-27",
            "2026-09-01",
            "1968-01-15",
            "60.00",
            "2.40",
            "2.00",
        ),
    ] {
        let v = veronica(year, paid, born);
        let r = run(&v);
        assert_eq!(r["treatment"], "income_stream_disability_under_60");
        assert_eq!(r["amounts"]["tax_free_component"], "300.00");
        assert_eq!(r["amounts"]["withheld_from"], "1100.00");
        assert_eq!(r["amounts"]["withholding_amount"], table, "{year}");
        assert_eq!(r["amounts"]["tax_offset"], "165.00");
        assert_eq!(r["amounts"]["offset_adjustment"], adjustment, "{year}");
        assert_eq!(r["amounts"]["withholding"], withheld, "{year}");
    }
}

/// Scale 1 without the tax-free threshold: $208 less the $165 offset.
#[test]
fn scale_1_without_the_threshold() {
    let mut v = veronica("2026-27", "2026-09-01", "1968-01-15");
    v["tax_free_threshold_claimed"] = json!(false);
    assert_eq!(amount(&v, "withholding_amount"), "208.00");
    assert_eq!(amount(&v, "withholding"), "43.00");
}

/// Each offset row of Part A step 3.
#[test]
fn offset_rows() {
    // 60 or over: 10% of the element untaxed in the fund; weekly $1,000 is
    // $138 on scale 2, less $100, above the 2% adjustment of $20.
    let v = aged_66("weekly", "1000.00");
    let r = run(&v);
    assert_eq!(r["treatment"], "income_stream_member_aged_60_or_over");
    assert_eq!(r["amounts"]["withholding_amount"], "138.00");
    assert_eq!(r["amounts"]["tax_offset"], "100.00");
    assert_eq!(r["amounts"]["offset_adjustment"], "20.00");
    assert_eq!(r["amounts"]["withholding"], "38.00");
    // 60 or over, account-based, taxed element and tax free only: nothing.
    let taxed = stream(
        "2026-27",
        "2026-09-01",
        "1960-01-01",
        "weekly",
        ["200.00", "800.00", "0.00"],
    );
    assert_eq!(amount(&taxed, "withheld_from"), "0.00");
    assert_eq!(amount(&taxed, "withholding"), "0.00");
    // Under 60 and not disabled: no offset, so the table amount.
    let mut under = veronica("2026-27", "2026-09-01", "1968-01-15");
    under["disability_superannuation_benefit"] = json!(false);
    let r = run(&under);
    assert_eq!(r["treatment"], "income_stream_member_under_60");
    assert_eq!(r["amounts"]["tax_offset"], "0.00");
    assert_eq!(r["amounts"]["offset_adjustment"], "0.00");
    assert_eq!(r["amounts"]["withholding"], "60.00");
}

/// A death benefit income stream to a dependant.
#[test]
fn dependants() {
    let mut v = veronica("2026-27", "2026-09-01", "1986-01-15");
    let o = v.as_object_mut().unwrap();
    o.remove("disability_superannuation_benefit");
    v["recipient"] = json!("death_benefits_dependant");
    v["deceased_aged_60_or_over"] = json!(false);
    let r = run(&v);
    assert_eq!(r["treatment"], "income_stream_dependant_under_60");
    assert_eq!(r["amounts"]["tax_offset"], "165.00");
    assert_eq!(r["amounts"]["withholding"], "2.00");
    // A dependant 60 or over: 10% of the untaxed element, whatever the
    // deceased's age.
    let mut old = aged_66("weekly", "1000.00");
    let o = old.as_object_mut().unwrap();
    o.remove("disability_superannuation_benefit");
    old["recipient"] = json!("death_benefits_dependant");
    for deceased in [false, true] {
        old["deceased_aged_60_or_over"] = json!(deceased);
        let r = run(&old);
        assert_eq!(r["treatment"], "income_stream_dependant_aged_60_or_over");
        assert_eq!(r["amounts"]["withholding"], "38.00");
    }
    v["deceased_aged_60_or_over"] = json!(true);
    assert_eq!(code(&v), "super_withholding.deceased_60_recipient_under_60");
    v.as_object_mut()
        .unwrap()
        .remove("deceased_aged_60_or_over");
    assert_eq!(code(&v), "input.missing_field");
}

/// Part A step 4's bands on the weekly equivalent in 2026-27: nil below
/// $538, 10% of the excess below $673, 2% below $1,216, nil from it.
#[test]
fn offset_adjustment_bands() {
    for (untaxed, adjustment) in [
        ("537.00", "0.00"),
        ("538.00", "0.00"),
        ("672.00", "13.40"),
        ("673.00", "13.46"),
        ("1215.00", "24.30"),
        ("1216.00", "0.00"),
    ] {
        let v = aged_66("weekly", untaxed);
        assert_eq!(amount(&v, "offset_adjustment"), adjustment, "{untaxed}");
    }
    // 2024-25 and 2025-26: $500, $625 and $1,181.
    for (untaxed, adjustment) in [("1180.00", "23.60"), ("1181.00", "0.00")] {
        let v = stream(
            "2025-26",
            "2025-09-01",
            "1960-01-01",
            "weekly",
            ["0.00", "0.00", untaxed],
        );
        assert_eq!(amount(&v, "offset_adjustment"), adjustment, "{untaxed}");
    }
    // Weekly $600 untaxed: notional $42 - $60 is below the $6.20 adjustment.
    let v = aged_66("weekly", "600.00");
    assert_eq!(amount(&v, "offset_adjustment"), "6.20");
    assert_eq!(amount(&v, "withholding"), "6.00");
}

/// Monthly and quarterly payments convert to and from the weekly
/// equivalent: $2,600 a month and $7,800 a quarter are $600 a week.
#[test]
fn monthly_and_quarterly() {
    let monthly = aged_66("monthly", "2600.00");
    assert_eq!(amount(&monthly, "weekly_equivalent"), "600.00");
    assert_eq!(amount(&monthly, "withholding_amount"), "182.00");
    assert_eq!(amount(&monthly, "offset_adjustment"), "26.87");
    assert_eq!(amount(&monthly, "withholding"), "27.00");
    let quarterly = aged_66("quarterly", "7800.00");
    assert_eq!(amount(&quarterly, "withholding_amount"), "546.00");
    assert_eq!(amount(&quarterly, "offset_adjustment"), "80.60");
    assert_eq!(amount(&quarterly, "withholding"), "81.00");
}

/// Cents in the stream are ignored first, and the result is rounded once,
/// 50 cents up: $139 less 10% of $1,005 is $38.50, withheld as $39.
#[test]
fn cents_and_rounding() {
    let v = aged_66("weekly", "1005.99");
    assert_eq!(amount(&v, "withheld_from"), "1005.00");
    assert_eq!(amount(&v, "withholding_amount"), "139.00");
    assert_eq!(amount(&v, "tax_offset"), "100.50");
    assert_eq!(amount(&v, "withholding"), "39.00");
}

/// Without a TFN: 47% or 45% of the step 1 amount, cents ignored.
#[test]
fn no_tfn() {
    let mut v = stream(
        "2026-27",
        "2026-09-01",
        "1980-01-01",
        "fortnightly",
        ["0.00", "1100.75", "0.00"],
    );
    v["tfn_provided"] = json!(false);
    v.as_object_mut()
        .unwrap()
        .remove("tax_free_threshold_claimed");
    assert_eq!(amount(&v, "withholding"), "517.00");
    let mut foreign = aged_66("weekly", "1000.40");
    foreign["tfn_provided"] = json!(false);
    foreign["foreign_resident"] = json!(true);
    foreign
        .as_object_mut()
        .unwrap()
        .remove("tax_free_threshold_claimed");
    assert_eq!(amount(&foreign, "withholding"), "450.00");
    // The threshold is a TFN declaration answer.
    v["tax_free_threshold_claimed"] = json!(true);
    assert_eq!(code(&v), "input.field_unknown");
}

#[test]
fn refusals() {
    let base = veronica("2026-27", "2026-09-01", "1968-01-15");
    let with = |key: &str, value: Value| {
        let mut v = base.clone();
        v[key] = value;
        v
    };
    let mut early = veronica("2023-24", "2023-09-01", "1966-01-15");
    assert_eq!(code(&early), "super_withholding.income_stream_year");
    early["year"] = json!("2019-20");
    assert_eq!(code(&early), "period.unsupported");
    for period in ["annually", "daily", "1"] {
        assert_eq!(
            code(&with("income_stream_period", json!(period))),
            "super_withholding.income_stream_period_invalid"
        );
    }
    assert_eq!(
        code(&with("foreign_resident", json!(true))),
        "super_withholding.income_stream_foreign_resident"
    );
    for recipient in ["non_dependant", "rollover", "deceased_estate_trustee"] {
        assert_eq!(
            code(&with("recipient", json!(recipient))),
            "super_withholding.recipient_invalid"
        );
    }
    // A capped defined benefit stream: Part A under 60, refused from 60.
    assert_eq!(
        amount(&with("capped_defined_benefit", json!(true)), "withholding"),
        "2.00"
    );
    let mut capped = aged_66("weekly", "1000.00");
    capped["capped_defined_benefit"] = json!(true);
    assert_eq!(code(&capped), "super_withholding.capped_defined_benefit");
}

/// Lump sum facts are not taken; the stream's are required and checked.
#[test]
fn inputs_taken_and_required() {
    let base = veronica("2026-27", "2026-09-01", "1968-01-15");
    let with = |key: &str, value: Value| {
        let mut v = base.clone();
        v[key] = value;
        v
    };
    for (key, value) in [
        ("terminal_medical_condition", json!(false)),
        ("service_before_july_1983", json!(false)),
        ("low_rate_cap_used", json!("0.00")),
        ("untaxed_plan_cap_used", json!("0.00")),
        ("other_lump_sums_this_year", json!(false)),
        ("deceased_aged_60_or_over", json!(false)),
    ] {
        assert_eq!(code(&with(key, value)), "input.field_unknown", "{key}");
    }
    for key in [
        "date_of_birth",
        "disability_superannuation_benefit",
        "capped_defined_benefit",
        "tax_free_threshold_claimed",
    ] {
        let mut v = base.clone();
        v.as_object_mut().unwrap().remove(key);
        assert_eq!(code(&v), "input.missing_field", "{key}");
    }
    assert_eq!(
        code(&with("date_of_birth", json!("2026-09-01"))),
        "super_benefit_tax.fact_conflict"
    );
    assert_eq!(
        code(&with("untaxed_element", json!("1100.01"))),
        "super_benefit_tax.fact_conflict"
    );
    assert_eq!(
        code(&with("capped_defined_benefit", json!("no"))),
        "input.flag_invalid"
    );
}

/// The payment is not limited by the interest's value when the stream
/// commenced (s 307-125(3)(a)): $1,400 from a stream that began at $1,000
/// with $300 tax free has a $420 tax free component.
#[test]
fn proportion_at_commencement() {
    let mut v = veronica("2026-27", "2026-09-01", "1968-01-15");
    v["interest_value"] = json!("1000.00");
    let r = run(&v);
    assert_eq!(r["amounts"]["tax_free_component"], "420.00");
    assert_eq!(r["amounts"]["taxable_component"], "980.00");
}

/// The stream's inputs are refused for a lump sum.
#[test]
fn stream_inputs_need_a_period() {
    let mut v = veronica("2026-27", "2026-09-01", "1968-01-15");
    v.as_object_mut().unwrap().remove("income_stream_period");
    assert_eq!(code(&v), "input.field_unknown");
}
