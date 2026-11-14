// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `personal-tax-estimate`: one person's income tax and levies for a year,
//! composed from the worksheets that work out each part. Each part comes from
//! its own command through [`evaluate_coded`], unchanged, with its own scope,
//! sources and warnings; this module only applies the offsets against basic
//! income tax and adds the parts up.
use crate::api::{Refusal, evaluate_coded};
use crate::decimal::{Decimal, Rounding, Signal};
use serde_json::{Map, Value, json};

const COMMAND: &str = "personal-tax-estimate";

/// The inputs given once, for every part that takes them.
const SHARED: [&str; 3] = ["year", "scope_confirmed", "taxable_income"];

/// The parts in the order they are worked out: command, whether its input is
/// required, whether it takes `taxable_income`, and the amount it contributes.
/// `resident-tax` and `low-income-tax-offset` take only the shared inputs.
const PARTS: [(&str, bool, bool, &str); 7] = [
    ("resident-tax", true, true, "basic_income_tax"),
    (SAPTO, false, false, "seniors_pensioners_tax_offset"),
    ("low-income-tax-offset", true, true, "low_income_tax_offset"),
    ("medicare-levy", true, true, "medicare_levy"),
    (
        "medicare-levy-surcharge",
        true,
        true,
        "medicare_levy_surcharge",
    ),
    ("study-loan-repayment", false, false, "compulsory_repayment"),
    ("division-293", false, false, "division_293_tax"),
];

const SAPTO: &str = "seniors-pensioners-tax-offset";

/// The parts a caller gives an object of inputs for.
fn takes_object(command: &str) -> bool {
    !matches!(command, "resident-tax" | "low-income-tax-offset")
}

const SCOPE: &str = "One individual who was an Australian resident for the full income year and not a working holiday maker, with established whole-dollar taxable income. Composes resident-tax, low-income-tax-offset, medicare-levy and medicare-levy-surcharge, and seniors-pensioners-tax-offset, study-loan-repayment and division-293 when their inputs are given; confirming scope confirms the scope each of them states under components, where each keeps its own sources and warnings. Applies the seniors and pensioners tax offset, then the low income tax offset, against basic income tax only, never below nil (ITAA 1997 s 63-10(1) items 5 and 20); neither reduces the Medicare levy or the surcharge. The total is before PAYG withholding and other credits, refundable offsets, every other tax offset (such as the private health insurance, franking and foreign income tax offsets), the transfer of unused SAPTO to a spouse and any refund or amount payable. Division 293 tax is shown but left out of the total, because it is assessed on its own notice. Takes taxable income as established; does not work it out from income and deductions. Refuses part-year residents, working holiday makers and foreign residents.";

const SOURCES: [&str; 3] = [
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/63-10",
    "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/low-income-tax-offset",
    "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/caps-limits-and-tax-on-super-contributions/division-293-tax-on-concessional-contributions-by-high-income-earners",
];

/// Where each amount comes from, in the amounts' order.
const WORKING: [(&str, &str); 9] = [
    ("basic_income_tax", "resident-tax: basic_income_tax"),
    (
        "seniors_pensioners_tax_offset_applied",
        "seniors-pensioners-tax-offset: seniors_pensioners_tax_offset, up to basic_income_tax (ITAA 1997 s 63-10(1) item 5); nil when not composed",
    ),
    (
        "low_income_tax_offset_applied",
        "low-income-tax-offset: low_income_tax_offset, up to basic_income_tax - seniors_pensioners_tax_offset_applied (ITAA 1997 s 63-10(1) item 20)",
    ),
    (
        "income_tax_after_offsets",
        "basic_income_tax - seniors_pensioners_tax_offset_applied - low_income_tax_offset_applied",
    ),
    ("medicare_levy", "medicare-levy: medicare_levy"),
    (
        "medicare_levy_surcharge",
        "medicare-levy-surcharge: medicare_levy_surcharge",
    ),
    (
        "study_loan_repayment",
        "study-loan-repayment: compulsory_repayment; nil when not composed",
    ),
    (
        "total_before_credits",
        "income_tax_after_offsets + medicare_levy + medicare_levy_surcharge + study_loan_repayment",
    ),
    (
        "division_293_tax",
        "division-293: division_293_tax; nil when not composed; assessed on its own notice, so not in total_before_credits",
    ),
];

fn signal(signal: Signal) -> Refusal {
    Refusal::new(signal.code(), format!("decimal.{}", signal.name()))
}

/// `residency`: only a full-year resident is composed.
fn check_residency(v: &Value) -> Result<(), Refusal> {
    let reason = match v.get("residency").map(|r| r.as_str()) {
        None => return Err(Refusal::new("input.missing_field", "residency is required")),
        Some(Some("full_year_resident")) => return Ok(()),
        Some(Some("part_year_resident")) => {
            "a part-year resident's tax-free threshold and offsets are apportioned, which no worksheet here does"
        }
        Some(Some("working_holiday_maker")) => {
            "a working holiday maker's income is taxed at the working holiday maker rates, which no worksheet here works out"
        }
        Some(Some("foreign_resident")) => "no worksheet here works out a foreign resident's tax",
        Some(_) => {
            return Err(Refusal::new(
                "input.residency_invalid",
                "residency must be full_year_resident, part_year_resident, working_holiday_maker or foreign_resident",
            ));
        }
    };
    Err(Refusal::new(
        "estimate.residency_unsupported",
        format!("{COMMAND} composes a full-year resident only: {reason}"),
    ))
}

/// Refuse a field the composition does not take, and a shared input given
/// again inside a part, before any part runs.
fn check_fields(v: &Value) -> Result<(), Refusal> {
    let taken = |key: &str| {
        key == "residency"
            || SHARED.contains(&key)
            || PARTS.iter().any(|(c, ..)| *c == key && takes_object(c))
    };
    let object = v.as_object().into_iter().flatten();
    if let Some((key, _)) = object.clone().find(|(k, _)| !taken(k)) {
        return Err(Refusal::new(
            "input.field_unknown",
            format!("{key} is not an input of {COMMAND}"),
        ));
    }
    for (command, input) in object {
        let given_once = |k: &String| {
            SHARED.contains(&k.as_str()) || (command == "medicare-levy" && k == "sapto_entitled")
        };
        if let Some(key) = input
            .as_object()
            .into_iter()
            .flat_map(|o| o.keys())
            .find(|k| given_once(k))
        {
            return Err(Refusal::new(
                "input.field_unknown",
                format!(
                    "{key} is not an input of {command} within {COMMAND}: it is given once, for every part"
                ),
            ));
        }
    }
    Ok(())
}

/// The surcharge worksheet's scope excludes anyone in a Medicare levy
/// exemption category at any time in the year.
fn check_exemption(v: &Value) -> Result<(), Refusal> {
    let exempt = [
        "medicare_levy_exemption_full_days",
        "medicare_levy_exemption_half_days",
    ]
    .iter()
    .any(|key| {
        v["medicare-levy"][key]
            .as_i64()
            .is_some_and(|days| days > 0)
    });
    if exempt && v.get("medicare-levy-surcharge").is_some() {
        return Err(Refusal::new(
            "estimate.surcharge_with_exemption",
            "medicare-levy-surcharge excludes a person in a Medicare levy exemption category at any time in the year, and medicare-levy gives exemption days",
        ));
    }
    Ok(())
}

/// One part's input: its own inputs, if any, with the shared ones added.
fn part_input(v: &Value, command: &str, with_income: bool) -> Value {
    let mut input = if takes_object(command) {
        v[command].clone()
    } else {
        json!({})
    };
    if let Some(object) = input.as_object_mut() {
        for key in SHARED {
            if key != "taxable_income" || with_income {
                object.insert(key.into(), v[key].clone());
            }
        }
    }
    input
}

/// Run each part in turn: its result, or the part's own refusal under its name.
fn run_parts(v: &Value) -> Result<Map<String, Value>, Refusal> {
    let mut results = Map::new();
    for (command, required, with_income, _) in PARTS {
        if !required && v.get(command).is_none() {
            continue;
        }
        if required && takes_object(command) && v.get(command).is_none() {
            return Err(Refusal::new(
                "input.missing_field",
                format!("{command} is required"),
            ));
        }
        let mut input = part_input(v, command, with_income);
        // The Medicare levy's SAPTO thresholds follow the offset worked out
        // above: entitlement at more than nil, as its scope requires.
        if command == "medicare-levy"
            && let (Some(object), Some(sapto)) = (input.as_object_mut(), results.get(SAPTO))
        {
            let amount = sapto["amounts"]["seniors_pensioners_tax_offset"].as_str();
            let entitled = amount
                .and_then(|a| Decimal::parse(a).ok())
                .is_some_and(|a| !a.is_zero());
            object.insert("sapto_entitled".into(), json!(entitled));
        }
        let result = evaluate_coded(command, &input)
            .map_err(|r| Refusal::new(r.code, format!("{command}: {}", r.message)))?;
        results.insert(command.into(), result);
    }
    Ok(results)
}

/// A part's amount, or nil when the part was not composed.
fn amount(results: &Map<String, Value>, command: &str, key: &str) -> Result<Decimal, Refusal> {
    let text = results
        .get(command)
        .and_then(|r| r["amounts"][key].as_str())
        .unwrap_or("0.00");
    Decimal::parse(text).map_err(signal)
}

fn lesser(a: Decimal, b: Decimal) -> Result<Decimal, Refusal> {
    Ok(if a.lt(&b).map_err(signal)? { a } else { b })
}

/// The composed amounts, in [`WORKING`]'s order.
fn compose_amounts(results: &Map<String, Value>) -> Result<[Decimal; 9], Refusal> {
    let part = |i: usize| amount(results, PARTS[i].0, PARTS[i].3);
    let calc = |r: Result<Decimal, Signal>| r.map_err(signal);
    let basic = part(0)?;
    let sapto = lesser(part(1)?, basic.clone())?;
    let lito = lesser(part(2)?, calc(basic.sub(&sapto))?)?;
    let after = calc(calc(basic.sub(&sapto))?.sub(&lito))?;
    let (levy, surcharge, loan) = (part(3)?, part(4)?, part(5)?);
    let total = calc(calc(calc(after.add(&levy))?.add(&surcharge))?.add(&loan))?;
    Ok([
        basic,
        sapto,
        lito,
        after,
        levy,
        surcharge,
        loan,
        total,
        part(6)?,
    ])
}

/// `personal-tax-estimate`: the composed amounts, where each came from, and
/// every part's own result.
pub fn compose(v: &Value) -> Result<Value, Refusal> {
    check_fields(v)?;
    check_exemption(v)?;
    check_residency(v)?;
    let results = run_parts(v)?;
    let mut amounts = Map::new();
    for ((key, _), value) in WORKING.iter().zip(compose_amounts(&results)?) {
        let cents = value.quantize(-2, Rounding::HalfUp).map_err(signal)?;
        amounts.insert((*key).into(), json!(cents.to_string()));
    }
    let not_composed: Vec<&str> = PARTS
        .iter()
        .map(|(c, ..)| *c)
        .filter(|c| !results.contains_key(*c))
        .collect();
    let working: Map<String, Value> = WORKING
        .iter()
        .map(|(k, w)| ((*k).into(), json!(w)))
        .collect();
    let resident_tax = &results["resident-tax"];
    Ok(json!({
        "ok": true,
        "engine": resident_tax["engine"],
        "engine_version": resident_tax["engine_version"],
        "calculation": "personal_tax_estimate",
        "period": resident_tax["period"],
        "residency": "full_year_resident",
        "amounts": amounts,
        "working": working,
        "not_composed": not_composed,
        "components": results,
        "scope": SCOPE,
        "sources": SOURCES,
        "source_checked": "2026-11-15",
        "warnings": [
            "Operator-confirmed facts and scope; eligibility is not independently verified.",
            "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
            "Each part's own warnings are in its result under components.",
        ],
    }))
}
