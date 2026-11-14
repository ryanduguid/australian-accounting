// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use australian_tax_calculators::api;
use australian_tax_calculators::decimal::{Decimal, Rounding};
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

/// Worksheets added after Python 0.2.0, so absent from the frozen corpus;
/// `tests/fbt_car.rs`, `tests/main_residence.rs`,
/// `tests/borrowing_expenses.rs`, `tests/division_296.rs`,
/// `tests/termination_payment.rs`, `tests/schedule_5.rs`,
/// `tests/working_holiday_maker.rs` and `tests/foreign_resident_tax.rs` cover them.
const NEW_KINDS: [(&str, &str); 11] = [
    ("fbt-car-statutory", "fbt_car_statutory"),
    ("fbt-car-operating-cost", "fbt_car_operating_cost"),
    ("main-residence", "main_residence"),
    ("borrowing-expenses", "borrowing_expenses"),
    ("division-296", "division_296"),
    ("termination-payment", "termination_payment"),
    ("schedule-5-method-a", "schedule_5_method_a"),
    ("schedule-5-method-b-i", "schedule_5_method_b_i"),
    ("schedule-5-method-b-ii", "schedule_5_method_b_ii"),
    (
        "working-holiday-maker-withholding",
        "working_holiday_maker_withholding",
    ),
    ("foreign-resident-tax", "foreign_resident_tax"),
];

/// The JSON boundary's answer in the corpus encoding: `{"ok": value}` or `{"error": message}`.
fn outcome(command: &str, input: &Value) -> Value {
    match api::evaluate(command, input) {
        Ok(v) => json!({ "ok": v }),
        Err(m) => json!({ "error": m }),
    }
}

/// The frozen catalogue predates the new worksheets: compare it without them.
fn without_new_kinds(command: &str, mut actual: Value) -> Value {
    if command == "worksheet-catalogue"
        && let Some(entries) = actual.get_mut("ok").and_then(Value::as_object_mut)
    {
        for (_, kind) in NEW_KINDS {
            assert!(entries.shift_remove(kind).is_some(), "{kind} missing");
        }
    }
    actual
}

/// Applies the source-record divergences CHANGELOG.md documents to a frozen
/// Python result; the frozen corpus itself is never edited.
fn adapt(command: &str, input: &Value, mut expected: Value) -> Value {
    // The ATO moved its lump sum in arrears page; the redirect was seen on 26 October 2026.
    let lump_sum_old = "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/lump-sum-payment-in-arrears-tax-offsets";
    let lump_sum_new = "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-you-must-declare/lump-sum-payment-in-arrears";
    let lump_sum = match command {
        "medicare-levy" => expected.pointer_mut("/ok/sources/3"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/medicare_levy/sources/3"),
        _ => None,
    };
    if let Some(source) = lump_sum {
        assert_eq!(*source, json!(lump_sum_old), "frozen lump sum source moved");
        *source = json!(lump_sum_new);
    }
    if let Some(Value::String(error)) = expected.get_mut("error") {
        *error = error.replace(lump_sum_old, lump_sum_new);
    }
    // s 291-20(4): only the unused cap that covers the excess is applied. The
    // frozen value is all the eligible unused cap, so it is the upper bound.
    if command == "contribution-caps" && expected.pointer("/ok/amounts").is_some() {
        let dec = |v: &Value| Decimal::parse(v.as_str().unwrap()).unwrap();
        let zero = Decimal::from_int(0);
        let carried = dec(&expected["ok"]["amounts"]["carry_forward_applied"]);
        let excess = dec(&input["concessional_contributions"])
            .sub(&dec(&expected["ok"]["rates"]["concessional_cap"]))
            .unwrap();
        let needed = if excess.gt(&zero).unwrap() {
            excess
        } else {
            zero
        };
        let applied = if needed.lt(&carried).unwrap() {
            needed
        } else {
            carried
        };
        let cents = applied.quantize(-2, Rounding::HalfUp).unwrap();
        expected["ok"]["amounts"]["carry_forward_applied"] = json!(cents.to_string());
    }
    // gst, capital-gains and depreciation also accept 2026-27 (CHANGELOG.md).
    let opened = ["gst", "capital_gains", "depreciation"];
    let period = json!({"id": "2026-27", "basis": "income year", "start": "2026-07-01",
        "end": "2027-06-30", "arguments": {"year": "2026-27"}});
    match command {
        "periods" if opened.iter().any(|kind| input["kind"] == *kind) => {
            expected["ok"].as_array_mut().unwrap().push(period);
        }
        "worksheet-catalogue" => {
            for kind in opened {
                let periods = expected["ok"][kind]["supported_periods"].as_array_mut();
                periods.unwrap().push(period.clone());
            }
        }
        "gst" | "capital-gains" | "depreciation" => {
            if let Some(Value::String(error)) = expected.get_mut("error") {
                *error = error.replace("; supported: 2025-26.", "; supported: 2025-26, 2026-27.");
            }
        }
        _ => {}
    }
    adapt_payg(command, input, &mut expected);
    adapt_study_loan_withholding(command, input, &mut expected);
    adapt_fbt(command, input, &mut expected);
    adapt_medicare(command, input, &mut expected);
    adapt_depreciation(command, &mut expected);
    let study = match command {
        "study-loan-repayment" => expected.pointer_mut("/ok"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/study_loan_repayment"),
        _ => None,
    };
    let Some(study) = study else {
        return expected;
    };
    // The ATO moved and retitled its study loan page; read again on 20 October 2026.
    let url = "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates";
    study["sources"][0] = json!(url);
    study["source_checked"] = json!("2026-10-20");
    if command == "worksheet-catalogue" {
        study["source"] = json!(url);
        let passage = study["source_passage"].as_str().unwrap().replace(
            "repayment thresholds and rates",
            "minimum repayment income and rates",
        );
        study["source_passage"] = json!(passage);
        let evidence = &mut study["example_evidence"];
        evidence["document"] =
            json!("ATO study and training loan minimum repayment income and rates");
        evidence["reviewed"] = json!("2026-10-13");
        evidence["checked"] = json!("2026-10-20");
    }
    expected
}

/// `payg-withholding` also accepts 2024-25 and 2025-26 with the Schedule 1
/// coefficients for payments from 1 July 2024 to 30 June 2026, and quarterly
/// pays, scale 4, tax offsets, the Medicare levy adjustment and the additional
/// amounts for 53 or 27 pays (CHANGELOG.md).
fn adapt_payg(command: &str, input: &Value, expected: &mut Value) {
    let old_scope = "One regular weekly, fortnightly or monthly payment of salary or wages made from 1 July 2026 to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. Earnings include allowances subject to withholding. Excludes scale 4, tax offsets, Medicare levy adjustments, study and training support loans, extra amounts for 53 or 27 pays, quarterly and bi-monthly pays, back payments, bonuses, termination and other schedules.";
    let new_scope = "One regular weekly, fortnightly, monthly or quarterly payment of salary or wages made from 1 July 2024, in the income year given, on the scale the operator established from the payee's declarations: 1, 2, 3, 5 or 6 for a payee who gave a TFN, or 4 for one who did not, as a resident or foreign resident. The year selects the Schedule 1 coefficients and rules for payments made in it. Earnings include allowances subject to withholding. When given, the yearly tax offsets claimed on a Withholding declaration reduce scales 2, 5 and 6, the Medicare levy adjustment claimed on a Medicare levy variation declaration for a spouse or dependent children reduces scales 2 and 6, and the additional amount a payee asks for in a year of 53 weekly or 27 fortnightly pays is added. Reductions that exceed the amount withheld before them are refused. Excludes a Medicare levy variation that increases withholding, study and training support loans, bi-monthly, daily and casual pays, back payments, bonuses, termination and other schedules.";
    let page = "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026";
    let passage = "Schedule 1 (NAT 1004) for payments from 1 July 2026, and for payments from 1 July 2024 to 30 June 2026: weekly coefficients and scale 4, using a formula (rounding and the 53 and 27 pay tables), working out the weekly earnings (quarterly pays and tax offsets), Medicare levy adjustment, examples of calculating the withholding amounts, sample data";
    let years = ["2024-25", "2025-26"].map(|year| {
        let start = format!("{}-07-01", &year[..4]);
        let end = format!("20{}-06-30", &year[5..]);
        json!({"id": year, "basis": "income year", "start": start, "end": end,
               "arguments": {"year": year}})
    });
    if command == "payg-withholding"
        && let Some(Value::String(error)) = expected.get_mut("error")
    {
        let scale_4 = input["scale"] == 4;
        *error = match error.as_str() {
            "pay_period must be weekly, fortnightly or monthly." => {
                "pay_period must be weekly, fortnightly, monthly or quarterly.".into()
            }
            "scale must be 1, 2, 3, 5 or 6; scale 4 is not supported." if scale_4 => {
                "foreign_resident is required for scale 4.".into()
            }
            "scale must be 1, 2, 3, 5 or 6; scale 4 is not supported." => {
                "scale must be 1, 2, 3, 4, 5 or 6.".into()
            }
            other => other.replace(
                "; supported: 2026-27.",
                "; supported: 2024-25, 2025-26, 2026-27.",
            ),
        };
    }
    let record = match command {
        "payg-withholding" => expected.pointer_mut("/ok"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/payg_withholding"),
        "periods" if input["kind"] == "payg_withholding" => {
            let periods = expected["ok"].as_array_mut().unwrap();
            periods.splice(0..0, years);
            return;
        }
        _ => None,
    };
    let Some(record) = record else {
        return;
    };
    assert_eq!(record["scope"], old_scope, "frozen PAYG scope changed");
    record["scope"] = json!(new_scope);
    record["sources"].as_array_mut().unwrap().push(json!(page));
    record["source_checked"] = json!("2026-11-14");
    if command == "worksheet-catalogue" {
        record["source_passage"] = json!(passage);
        let periods = record["supported_periods"].as_array_mut().unwrap();
        periods.splice(0..0, years);
        let evidence = &mut record["example_evidence"];
        evidence["document"] = json!("ATO Schedule 1 sample data");
        evidence["paragraphs"] = json!(
            "withholding amounts, Medicare levy adjustment scale 2 and Medicare half-levy adjustment scale 6: weekly, fortnightly and monthly tables"
        );
        evidence["checked"] = json!("2026-11-14");
        let inputs = &mut record["required_inputs"];
        inputs["pay_period"] = json!("weekly, fortnightly, monthly or quarterly");
        inputs["scale"] = json!("integer 1 to 6; scale 4 also needs foreign_resident");
        record["optional_inputs"] = json!({
            "foreign_resident": "boolean, scale 4 only and required there: true for a payee who is a foreign resident, false for a resident",
            "tax_offsets": "AUD, scales 2, 5 and 6 only: the yearly total the payee claimed at the tax offsets questions on their Withholding declaration",
            "medicare_adjustment_children": "integer from 0, scales 2 and 6 only, when the payee's Medicare levy variation declaration claims the adjustment (yes at question 10, and yes at question 9 or 12): the dependent children shown at question 12, or 0 for a spouse and no children",
            "extra_pay_withholding": "boolean, weekly and fortnightly pays only: true when the financial year has 53 weekly or 27 fortnightly pays and the payee asked for the additional amount"
        });
    }
}

/// `study-loan-withholding` also accepts 2025-26 pays from 24 September 2025,
/// with the Schedule 8 coefficients of F2025L01016 (CHANGELOG.md).
fn adapt_study_loan_withholding(command: &str, input: &Value, expected: &mut Value) {
    let old_scope = "One regular weekly, fortnightly or monthly payment of salary or wages made from 1 July 2026 to a payee who gave a TFN declaration or withholding declaration saying they have a study and training support loan debt, on scale 1, 2, 3, 5 or 6 as the operator established. Scale 1 uses the Schedule 8 component for a payee who did not claim the tax-free threshold; scales 2, 3, 5 and 6 use the one for a payee who claimed it or is a foreign resident. Earnings include allowances subject to withholding. The result is the Schedule 1 amount plus the Schedule 8 component, each rounded to the dollar; the ATO's combined coefficients can differ by a dollar and it accepts either method. Excludes scale 4, a Medicare levy variation for a spouse or dependants, tax offsets, extra amounts for 53 or 27 pays, quarterly pays, lump sum termination payments, back payments, bonuses and other schedules.";
    let new_scope = "One regular weekly, fortnightly or monthly payment of salary or wages made from 24 September 2025, in the income year given, to a payee who gave a TFN declaration or withholding declaration saying they have a study and training support loan debt, on scale 1, 2, 3, 5 or 6 as the operator established. The year selects the Schedule 1 coefficients and the Schedule 8 component coefficients for payments made in it; a 2025-26 payment needs its pay date, because the Schedule 8 coefficients of F2025L01016 apply from 24 September 2025. Scale 1 uses the Schedule 8 component for a payee who did not claim the tax-free threshold; scales 2, 3, 5 and 6 use the one for a payee who claimed it or is a foreign resident. Earnings include allowances subject to withholding. The result is the Schedule 1 amount plus the Schedule 8 component, each rounded to the dollar; the ATO's combined coefficients can differ by a dollar and it accepts either method. Excludes 2025-26 payments made before 24 September 2025, scale 4, a Medicare levy variation for a spouse or dependants, tax offsets, extra amounts for 53 or 27 pays, quarterly pays, lump sum termination payments, back payments, bonuses and other schedules.";
    let added_sources = [
        "https://www.legislation.gov.au/F2025L01016/asmade/text",
        "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026",
    ];
    let passage = "Schedule 8 (NAT 3539) for payments from 1 July 2026, and Schedule 8 of the Taxation Administration (Withholding Schedules) Instrument (No. 2) 2025 (F2025L01016) for payments from 24 September 2025: study and training support loans component rates, calculating fortnightly and monthly amounts, examples 1 to 3, rounding";
    let year = json!({"id": "2025-26", "basis": "income year", "start": "2025-09-24",
        "end": "2026-06-30", "arguments": {"year": "2025-26"}});
    if command == "study-loan-withholding"
        && let Some(Value::String(error)) = expected.get_mut("error")
    {
        *error = error.replace("; supported: 2026-27.", "; supported: 2025-26, 2026-27.");
    }
    let record = match command {
        "study-loan-withholding" => expected.pointer_mut("/ok"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/study_loan_withholding"),
        "periods" if input["kind"] == "study_loan_withholding" => {
            expected["ok"].as_array_mut().unwrap().insert(0, year);
            return;
        }
        _ => None,
    };
    let Some(record) = record else {
        return;
    };
    assert_eq!(
        record["scope"], old_scope,
        "frozen study loan scope changed"
    );
    record["scope"] = json!(new_scope);
    let sources = record["sources"].as_array_mut().unwrap();
    sources.extend(added_sources.map(|source| json!(source)));
    record["source_checked"] = json!("2026-11-14");
    if command == "worksheet-catalogue" {
        record["source_passage"] = json!(passage);
        let periods = record["supported_periods"].as_array_mut().unwrap();
        periods.insert(0, year);
        let evidence = &mut record["example_evidence"];
        evidence["document"] = json!(
            "ATO Schedule 8 statement of formulas for STSL components, and F2025L01016 Schedule 8 for 2025-26"
        );
        evidence["paragraphs"] = json!("component rates, examples 1 to 3 in each");
        evidence["checked"] = json!("2026-11-14");
        record["optional_inputs"] = json!({
            "pay_date": "YYYY-MM-DD date the payment is made, within the income year; required for 2025-26, whose Schedule 8 coefficients apply from 24 September 2025"
        });
    }
}

/// `fbt` also accepts the FBT year ending 31 March 2027, at the same rates
/// (CHANGELOG.md).
fn adapt_fbt(command: &str, input: &Value, expected: &mut Value) {
    let old_scope = "for the year ended 31 March 2026. Excludes";
    let new_scope = "for the year ended 31 March 2026 or 2027, both at the 47% FBT rate and the 2.0802 and 1.8868 gross-up rates. Excludes";
    let old_end = "before any optional rounding down to 5 cents.";
    let new_end = "before any optional rounding down to 5 cents. For 2027 the items follow the FBT return 2026 instructions until the ATO publishes the 2027 return.";
    let rates =
        "https://www.ato.gov.au/tax-rates-and-codes/fringe-benefits-tax-rates-and-thresholds";
    let year = json!({"id": "2027", "basis": "FBT year", "start": "2026-04-01",
        "end": "2027-03-31", "arguments": {"year_ended": 2027}});
    if command == "fbt"
        && let Some(Value::String(error)) = expected.get_mut("error")
    {
        *error = error.replace("; supported: 2026.", "; supported: 2026, 2027.");
    }
    let record = match command {
        "fbt" => expected.pointer_mut("/ok"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/fbt"),
        "periods" if input["kind"] == "fbt" => {
            expected["ok"].as_array_mut().unwrap().push(year);
            return;
        }
        _ => None,
    };
    let Some(record) = record else {
        return;
    };
    let scope = record["scope"].as_str().unwrap();
    assert!(
        scope.contains(old_scope) && scope.ends_with(old_end),
        "frozen FBT scope changed"
    );
    record["scope"] = json!(
        scope
            .replace(old_scope, new_scope)
            .replace(old_end, new_end)
    );
    record["sources"].as_array_mut().unwrap().push(json!(rates));
    record["source_checked"] = json!("2026-11-04");
    if command == "worksheet-catalogue" {
        let passage = record["source_passage"].as_str().unwrap();
        record["source_passage"] = json!(format!(
            "{passage}; FBT rates and thresholds, the FBT rate and gross-up rates for years ending 31 March 2023 to 2027"
        ));
        record["supported_periods"]
            .as_array_mut()
            .unwrap()
            .push(year);
    }
}

/// `medicare-levy` also accepts 2026-27, at the thresholds Act No. 58, 2026
/// applies to 2025-26 and later years, and cites that Act (CHANGELOG.md).
fn adapt_medicare(command: &str, input: &Value, expected: &mut Value) {
    let act = "https://www.legislation.gov.au/C2026A00058/asmade/text";
    let passage = "Medicare Levy Act 1986 ss 3(1), 6(1) and 7(1) and (2), compilation 53; Treasury Laws Amendment (Delivering an Efficient and Trusted Tax System) Act 2026 Sch 5 item 14 (2025-26 and later years); ATO Medicare levy reduction for low-income earners (2025-26) and myTax 2025 table 2 (2024-25)";
    let year = json!({"id": "2026-27", "basis": "income year", "start": "2026-07-01",
        "end": "2027-06-30", "arguments": {"year": "2026-27"}});
    if command == "medicare-levy"
        && let Some(Value::String(error)) = expected.get_mut("error")
    {
        *error = error.replace(
            "; supported: 2024-25, 2025-26.",
            "; supported: 2024-25, 2025-26, 2026-27.",
        );
    }
    let record = match command {
        "medicare-levy" => expected.pointer_mut("/ok"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/medicare_levy"),
        "periods" if input["kind"] == "medicare_levy" => {
            expected["ok"].as_array_mut().unwrap().push(year);
            return;
        }
        _ => None,
    };
    let Some(record) = record else {
        return;
    };
    record["sources"].as_array_mut().unwrap().push(json!(act));
    record["source_checked"] = json!("2026-11-14");
    if command == "worksheet-catalogue" {
        record["source_passage"] = json!(passage);
        record["supported_periods"]
            .as_array_mut()
            .unwrap()
            .push(year);
    }
}

/// `depreciation` also takes an optional opening adjustable value for a later
/// year (CHANGELOG.md); without it every result is the first-year one.
fn adapt_depreciation(command: &str, expected: &mut Value) {
    let old_scope = "First year only, ordinary tangible Division 40 asset first held on or after 10 May 2006. Established cost, effective life and taxable-use proportion. Days run from first use or installation ready for use. No second-element costs, disposals, changed life, pooling,";
    let new_scope = "Ordinary tangible Division 40 asset first held on or after 10 May 2006, in the income year its start time occurs or, given its established opening adjustable value for the year, a later year. Established cost, effective life and taxable-use proportion. Days run from first use or installation ready for use. No second-element costs after the start year, disposals, changed life or other s 40-75(2) adjustment, pooling,";
    let record = match command {
        "depreciation" => expected.pointer_mut("/ok"),
        "worksheet-catalogue" => expected.pointer_mut("/ok/depreciation"),
        _ => None,
    };
    let Some(record) = record else {
        return;
    };
    let scope = record["scope"].as_str().unwrap();
    assert!(
        scope.starts_with(old_scope),
        "frozen depreciation scope changed"
    );
    record["scope"] = json!(scope.replace(old_scope, new_scope));
    let sections = ["40-70", "40-72", "40-75"].map(|s| {
        json!(format!(
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/{s}"
        ))
    });
    record["sources"].as_array_mut().unwrap().extend(sections);
    record["source_checked"] = json!("2026-11-13");
    if command == "worksheet-catalogue" {
        record["source_passage"] = json!(
            "Decline in value: diminishing value and prime cost methods, with the later-year diminishing value example; taxable use; ITAA 1997 ss 40-70(1), 40-72 and 40-75(1) and (7), base value and the limit on decline in a later year"
        );
        record["optional_inputs"] = json!({"opening_adjustable_value":
            "AUD, for a year after the start time only: cost less the decline in value of earlier years, not the deductions; omit in the start year"});
    }
}

/// Replays every captured call from the Python 0.2.0 tests and differential
/// generator, comparing serialised JSON so key order must match too.
#[test]
fn python_api_corpus_matches() {
    let corpus: Vec<Value> =
        serde_json::from_str(include_str!("data/python-api.json")).expect("corpus parses");
    assert!(corpus.len() > 3000, "corpus too small: {}", corpus.len());
    for command in api::commands() {
        if NEW_KINDS.iter().any(|(c, _)| *c == command) {
            continue;
        }
        assert!(
            corpus.iter().any(|row| row["command"] == command),
            "{command} has no case"
        );
    }
    let mut failures = Vec::new();
    let (mut reopened, mut reopened_fbt, mut reopened_quarterly, mut reopened_medicare) =
        (0, 0, 0, 0);
    for row in &corpus {
        let command = row["command"].as_str().unwrap();
        // Python 0.2.0 refused 2025-26 PAYG withholding, which now has its own
        // coefficients; tests/tables.rs checks that year against the ATO sample.
        if command == "payg-withholding" && row["input"]["year"] == "2025-26" {
            reopened += 1;
            continue;
        }
        // It refused quarterly pays too; tests/payg_withholding.rs checks them
        // against 13 times the ATO's weekly sample.
        if command == "payg-withholding" && row["input"]["pay_period"] == "quarterly" {
            reopened_quarterly += 1;
            continue;
        }
        // It refused the FBT year ending 31 March 2027 too; tests/tables.rs
        // checks it against the ATO's MantCo example.
        if command == "fbt" && row["input"]["year_ended"] == 2027 {
            reopened_fbt += 1;
            continue;
        }
        // And the 2026-27 Medicare levy, for want of published thresholds; Act
        // No. 58, 2026 applies the 2025-26 ones to later years, and
        // tests/tables.rs checks that year.
        if command == "medicare-levy" && row["input"]["year"] == "2026-27" {
            reopened_medicare += 1;
            continue;
        }
        let actual = without_new_kinds(command, outcome(command, &row["input"])).to_string();
        let expected = adapt(command, &row["input"], row["expected"].clone()).to_string();
        if actual != expected {
            failures.push(format!(
                "{} {command} {}\n  expected {expected}\n  actual   {actual}",
                row["test"], row["input"]
            ));
        }
    }
    assert_eq!(reopened, 1, "frozen 2025-26 PAYG refusals");
    assert_eq!(reopened_fbt, 1, "frozen 2027 FBT refusals");
    assert_eq!(reopened_quarterly, 1, "frozen quarterly PAYG refusals");
    assert_eq!(
        reopened_medicare, 1,
        "frozen 2026-27 Medicare levy refusals"
    );
    assert!(
        failures.is_empty(),
        "{} of {} differ:\n{}",
        failures.len(),
        corpus.len(),
        failures[..failures.len().min(5)].join("\n")
    );
}

#[test]
fn unparseable_decimal_strings_raise_invalid_operation() {
    let input = json!({"amount": "12,50", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"});
    assert_eq!(
        outcome("gst", &input),
        json!({"error": "decimal.InvalidOperation"})
    );
    let long = json!({"amount": "1".repeat(8193), "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"});
    assert_eq!(
        outcome("gst", &long),
        json!({"error": "amount: decimal input exceeds 8192 bytes"})
    );
}

#[test]
fn missing_arguments_and_non_objects_are_refused() {
    assert_eq!(
        outcome("resident-tax", &json!({"taxable_income": "1"})),
        json!({"error": "year is required"})
    );
    assert_eq!(
        outcome("gst", &json!([])),
        json!({"error": "input must be a JSON object"})
    );
    assert_eq!(
        outcome("periods", &json!({"kind": "nope"})),
        json!({"error": "unknown worksheet 'nope'"})
    );
}

fn run(args: &[&str], stdin: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_australian-tax-calculators"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap().replace("\r\n", "\n");
    (
        out.status.code().unwrap(),
        text(out.stdout),
        text(out.stderr),
    )
}

#[test]
fn binary_runs_commands_and_refuses_bad_input() {
    let (code, out, err) = run(
        &["gst"],
        r#"{"amount": "1100", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"}"#,
    );
    assert_eq!((code, err.as_str()), (0, ""));
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["amounts"]["gst"], "100.00");

    let (code, out, err) = run(&["gst"], "{");
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(err.starts_with("error: invalid JSON:"), "{err}");

    let (code, out, err) = run(&[], "");
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(
        err.starts_with("usage: australian-tax-calculators COMMAND"),
        "{err}"
    );

    let (code, out, _) = run(&["--api-help"], "");
    assert_eq!(code, 0);
    assert!(out.contains("worksheet-catalogue, periods"), "{out}");
}
