// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `personal-tax-estimate`: one person's income tax and levies for a year,
//! composed from the worksheets that work out each part. Each part comes from
//! its own command through [`evaluate_coded`], unchanged, with its own scope,
//! sources and warnings; this module only applies the offsets against basic
//! income tax and adds the parts up. It can work out taxable income from
//! labour income and deductions first, and compare the total with the tax
//! withheld.
use crate::api::{Refusal, evaluate_coded, optional_decimal, takes};
use crate::decimal::{Decimal, Rounding, Signal};
use crate::engine;
use serde_json::{Map, Value, json};

const COMMAND: &str = "personal-tax-estimate";

/// The inputs taxable income is worked out from, in place of
/// `taxable_income`: the first is required for it, the others optional.
const INCOME_INPUTS: [&str; 4] = [
    "assessable_labour_income",
    "other_assessable_income",
    "work_related_deductions",
    "other_deductions",
];

/// The year from which ITAA 1997 s 25-130 gives the standard deduction for
/// work-related expenses: Act No. 49 of 2026 Sch 4 item 17(1) applies it to
/// assessments for the 2026-27 income year and later years.
const STANDARD_DEDUCTION_FROM: &str = "2026-27";

/// The year from which ITAA 1997 Subdivision 61-E gives a resident the working
/// Australians tax offset: Act No. 49 of 2026 Sch 3 item 4 applies it to
/// assessments for the 2027-28 income year and later years.
const WATO_FROM: &str = "2027-28";

/// The offset's limit (ITAA 1997 s 61-160(a)).
const WATO_MAXIMUM: i64 = 250;

/// The working Australians tax offset's lines, after `basic_income_tax`: net
/// labour income as given or worked out, the offset and the part applied.
const WATO_WORKING: [(&str, &str); 3] = [
    (
        "net_labour_income",
        "net_labour_income: labour amounts less labour deductions (ITAA 1997 s 61-155(2)), as established",
    ),
    (
        "working_australians_tax_offset",
        "min(250, resident-tax: basic_income_tax on net_labour_income) (ITAA 1997 s 61-160); nil unless net_labour_income exceeds the tax-free threshold (s 61-155(1)(b)), where that tax is nil",
    ),
    (
        "working_australians_tax_offset_applied",
        "working_australians_tax_offset, up to basic_income_tax (ITAA 1997 s 63-10(1) item 3)",
    ),
];

/// `net_labour_income`'s line when it is worked out from the labour inputs.
const NET_LABOUR_WORKED: &str = "max(0, assessable_labour_income - work_related_deductions - standard_deduction): labour amounts less labour deductions (ITAA 1997 s 61-155(2)), with other_assessable_income and other_deductions taken to hold none";

/// The offset lines that follow, with the working Australians tax offset
/// applied first (ITAA 1997 s 63-10(1) item 3).
const WATO_PART_WORKING: [(&str, &str); 3] = [
    (
        "seniors_pensioners_tax_offset_applied",
        "seniors-pensioners-tax-offset: seniors_pensioners_tax_offset, up to basic_income_tax - working_australians_tax_offset_applied (ITAA 1997 s 63-10(1) item 5); nil when not composed",
    ),
    (
        "low_income_tax_offset_applied",
        "low-income-tax-offset: low_income_tax_offset, up to basic_income_tax - working_australians_tax_offset_applied - seniors_pensioners_tax_offset_applied (ITAA 1997 s 63-10(1) item 20)",
    ),
    (
        "income_tax_after_offsets",
        "basic_income_tax - working_australians_tax_offset_applied - seniors_pensioners_tax_offset_applied - low_income_tax_offset_applied",
    ),
];

/// Where each amount worked out before the parts comes from.
const INCOME_WORKING: [(&str, &str); 4] = [
    (
        "total_assessable_income",
        "assessable_labour_income + other_assessable_income",
    ),
    (
        "standard_deduction",
        "max(0, min(1000, assessable_labour_income) - work_related_deductions) for a resident from 2026-27 (ITAA 1997 s 25-130(1) and (2)); nil for a foreign resident and before 2026-27",
    ),
    (
        "total_deductions",
        "work_related_deductions + standard_deduction + other_deductions",
    ),
    (
        "taxable_income",
        "total_assessable_income - total_deductions, nil where the deductions equal or exceed it (ITAA 1997 s 4-15(1)); any tax loss is not worked out",
    ),
];

/// Where each amount compared with the tax withheld comes from.
const CREDIT_WORKING: [(&str, &str); 3] = [
    (
        "tax_withheld",
        "tax_withheld: the amounts withheld from withholding payments in the year, a credit once assessed (TAA 1953 Sch 1 s 18-15(1))",
    ),
    ("refund", "max(0, tax_withheld - total_before_credits)"),
    (
        "amount_payable",
        "max(0, total_before_credits - tax_withheld)",
    ),
];

/// The inputs given once, for every part that takes them.
const SHARED: [&str; 3] = ["year", "scope_confirmed", "taxable_income"];

/// The parts in the order they are worked out: command, whether its input is
/// required, whether it takes `taxable_income`, and the amount it contributes.
/// `resident-tax` and `low-income-tax-offset` take only the shared inputs;
/// [`part_for`] gives the part that runs for a foreign resident.
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

/// A foreign resident's basic tax, in place of `resident-tax`.
const FOREIGN_TAX: &str = "foreign-resident-tax";

/// The parts not composed for a foreign resident: command, the code a caller
/// who gives its input gets, and why.
const NOT_FOR_FOREIGN: [(&str, &str, &str); 5] = [
    (
        "low-income-tax-offset",
        "input.field_unknown",
        "ITAA 1997 s 61-110(1)(a) gives the offset only to a person resident at some time in the year",
    ),
    (
        "medicare-levy",
        "input.field_unknown",
        "ITAA 1936 s 251S(1)(a) levies the Medicare levy only on a person resident at some time in the year",
    ),
    (
        "medicare-levy-surcharge",
        "input.field_unknown",
        "the surcharge increases the Medicare levy, which ITAA 1936 s 251S(1)(a) levies only on a person resident at some time in the year",
    ),
    (
        SAPTO,
        "estimate.residency_unsupported",
        "a foreign resident with a pension assessable in Australia is not a prescribed non-resident (Income Tax Rates Act 1986 s 3(1)) and pays resident rates, which foreign-resident-tax excludes, so the estimate composes the offset for a full-year resident only",
    ),
    (
        "study-loan-repayment",
        "estimate.residency_unsupported",
        "a foreign resident's repayment includes the overseas levy on worldwide income, which study-loan-repayment excludes",
    ),
];

/// The part that runs for `command` (a [`PARTS`] entry), or `None` when it is
/// not composed for this residency.
fn part_for(command: &'static str, foreign: bool) -> Option<&'static str> {
    match command {
        "resident-tax" if foreign => Some(FOREIGN_TAX),
        _ if foreign && NOT_FOR_FOREIGN.iter().any(|(c, ..)| *c == command) => None,
        _ => Some(command),
    }
}

/// The parts a caller gives an object of inputs for.
fn takes_object(command: &str) -> bool {
    !matches!(
        command,
        "resident-tax" | FOREIGN_TAX | "low-income-tax-offset"
    )
}

const SCOPE: &str = "One individual who was an Australian resident for the full income year and not a working holiday maker, or a foreign resident for the full year, with established whole-dollar taxable income, or with established whole-dollar assessable labour income (ITAA 1997 s 25-130(4)), other assessable income, work-related deductions of the kinds s 25-130(2)(c) to (g) count and other deductions, from which taxable income is worked out (s 4-15(1)), with the standard deduction for work-related expenses (s 25-130) for a resident from 2026-27. For a resident, composes resident-tax, low-income-tax-offset, medicare-levy and medicare-levy-surcharge, and seniors-pensioners-tax-offset, study-loan-repayment and division-293 when their inputs are given; confirming scope confirms the scope each of them states under components, where each keeps its own sources and warnings. For a foreign resident, composes foreign-resident-tax and, when its inputs are given, division-293: a foreign resident has no low income tax offset (ITAA 1997 s 61-110(1)(a)) and pays no Medicare levy or surcharge (ITAA 1936 s 251S(1)(a)); the estimate refuses the seniors and pensioners tax offset for one, and a study loan repayment, which for a foreign resident includes the overseas levy on worldwide income that no worksheet here works out. Applies the seniors and pensioners tax offset, then the low income tax offset, against basic income tax only, never below nil (ITAA 1997 s 63-10(1) items 5 and 20); neither reduces the Medicare levy or the surcharge. For a resident from 2027-28, first applies the working Australians tax offset (s 63-10(1) item 3): the lesser of $250 and the basic income tax at the resident rates on net labour income (ss 61-155 and 61-160), given as net_labour_income or, with assessable_labour_income, worked out as it less work_related_deductions and the standard deduction, taking other_assessable_income and other_deductions to hold no labour amounts or labour deductions under s 61-155(2), such as business or personal services income, or the union fees and income protection premiums s 25-130(3) leaves out of work-related deductions; give net_labour_income where they do. From 2027-28 the scope is a resident with no minimum tax capital gain: the extra income tax on one (s 119-10, for CGT events from 1 July 2027) is not worked out. The total is before PAYG withholding and other credits, refundable offsets, every other tax offset (such as the private health insurance, franking and foreign income tax offsets), the transfer of unused SAPTO to a spouse and any refund or amount payable. Division 293 tax is shown but left out of the total, because it is assessed on its own notice. When tax_withheld is given, the total is compared with it (TAA 1953 Sch 1 s 18-15(1)) to give a refund or an amount payable before every other credit, refundable offset and amount owed. Classifying income and deductions, any tax loss and the limits on deductions that would create one are the operator's. Refuses part-year residents and working holiday makers, and parts that describe different families: a spouse in one part but not another, 2 spouse taxable incomes, or dependants without a spouse, whom the surcharge worksheet excludes.";

const SOURCES: [&str; 11] = [
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/4-15",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/25-130",
    "https://www.legislation.gov.au/C2026A00049/asmade/text",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-18-15",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/63-10",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/61-155",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/61-160",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/61-110",
    "https://www.ato.gov.au/law/view/document?docid=PAC/19360027/251S",
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

/// A foreign resident's lines that differ from [`WORKING`].
const FOREIGN_WORKING: [(&str, &str); 4] = [
    ("basic_income_tax", "foreign-resident-tax: basic_income_tax"),
    (
        "low_income_tax_offset_applied",
        "nil: ITAA 1997 s 61-110(1)(a) gives the offset only to a person resident at some time in the year",
    ),
    (
        "medicare_levy",
        "nil: ITAA 1936 s 251S(1)(a) levies the Medicare levy only on a person resident at some time in the year",
    ),
    (
        "medicare_levy_surcharge",
        "nil: the surcharge increases the Medicare levy, which a foreign resident does not pay",
    ),
];

fn signal(signal: Signal) -> Refusal {
    Refusal::new(signal.code(), format!("decimal.{}", signal.name()))
}

/// `residency`: whether the person is a foreign resident, who gives no input
/// for a part not composed for one; part-year residents and working holiday
/// makers are refused.
fn check_residency(v: &Value) -> Result<bool, Refusal> {
    let reason = match v.get("residency").map(|r| r.as_str()) {
        None => return Err(Refusal::new("input.missing_field", "residency is required")),
        Some(Some("full_year_resident")) => return Ok(false),
        Some(Some("foreign_resident")) => {
            return match NOT_FOR_FOREIGN.iter().find(|(c, ..)| v.get(*c).is_some()) {
                Some((command, code, why)) => Err(Refusal::new(
                    code,
                    format!("{command} is not composed for a foreign resident: {why}"),
                )),
                None => Ok(true),
            };
        }
        Some(Some("part_year_resident")) => {
            "a part-year resident's tax-free threshold and offsets are apportioned, which no worksheet here does"
        }
        Some(Some("working_holiday_maker")) => {
            "a working holiday maker's income is taxed at the working holiday maker rates, which no worksheet here works out"
        }
        Some(_) => {
            return Err(Refusal::new(
                "input.residency_invalid",
                "residency must be full_year_resident, part_year_resident, working_holiday_maker or foreign_resident",
            ));
        }
    };
    Err(Refusal::new(
        "estimate.residency_unsupported",
        format!(
            "{COMMAND} composes a full-year resident or a full-year foreign resident only: {reason}"
        ),
    ))
}

/// Refuse a field the composition does not take, a shared input given again
/// inside a part, and a field a part does not take, before any part runs:
/// `study-loan-repayment` and `division-293` alone ignore an unknown field.
fn check_fields(v: &Value) -> Result<(), Refusal> {
    let taken = |key: &str| {
        matches!(key, "residency" | "tax_withheld" | "net_labour_income")
            || INCOME_INPUTS.contains(&key)
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
        if let Some(key) = input
            .as_object()
            .into_iter()
            .flat_map(|o| o.keys())
            .find(|k| !takes(command, k))
        {
            return Err(Refusal::new(
                "input.field_unknown",
                format!("{key} is not an input of {command}"),
            ));
        }
    }
    Ok(())
}

fn family_mismatch(message: String) -> Result<(), Refusal> {
    Err(Refusal::new("estimate.family_mismatch", message))
}

/// The parts describe one family: a spouse in each part or in none, one spouse
/// taxable income, and no dependants without a spouse, a case the surcharge
/// worksheet excludes (Medicare Levy Act 1986 s 8C). Each part's scope already
/// rules out a change of spouse during the year.
fn check_family(v: &Value) -> Result<(), Refusal> {
    let levy = &v["medicare-levy"];
    let mut spouse = vec![
        ("medicare-levy", levy.get("spouse_taxable_income").is_some()),
        (
            "medicare-levy-surcharge",
            v["medicare-levy-surcharge"]
                .get("spouse_income_for_surcharge_purposes")
                .is_some(),
        ),
    ];
    if let Some(status) = v[SAPTO]["status"].as_str() {
        spouse.push((SAPTO, status != "single"));
    }
    if let Some((other, _)) = spouse.iter().find(|(_, s)| *s != spouse[0].1) {
        let (with, without) = if spouse[0].1 {
            (spouse[0].0, *other)
        } else {
            (*other, spouse[0].0)
        };
        return family_mismatch(format!(
            "{with} states a spouse and {without} does not: the parts must describe the same family"
        ));
    }
    let dependants = levy["dependant_offset_entitled"] == json!(true)
        || levy["dependent_children"].as_i64().is_some_and(|n| n > 0);
    if dependants && !spouse[0].1 {
        return family_mismatch(
            "medicare-levy states dependants without a spouse, which medicare-levy-surcharge excludes (Medicare Levy Act 1986 s 8C)".into(),
        );
    }
    let amount = |x: &Value| {
        let text = x.as_str().map_or_else(|| x.to_string(), str::to_owned);
        Decimal::parse(&text).ok()
    };
    let spouse_income = |part: &Value| amount(&part["spouse_taxable_income"]);
    if let (Some(a), Some(b)) = (spouse_income(levy), spouse_income(&v[SAPTO]))
        && matches!(a.equals(&b), Ok(false))
    {
        return family_mismatch(format!(
            "medicare-levy and {SAPTO} give different spouse_taxable_income: {a} and {b}"
        ));
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
fn run_parts(v: &Value, foreign: bool) -> Result<Map<String, Value>, Refusal> {
    let mut results = Map::new();
    for (command, required, with_income, _) in PARTS {
        let Some(command) = part_for(command, foreign) else {
            continue;
        };
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
    let Some(result) = results.get(command) else {
        return Ok(Decimal::from_int(0));
    };
    let text = result["amounts"][key]
        .as_str()
        .expect("a composed part gives its amount");
    Decimal::parse(text).map_err(signal)
}

fn lesser(a: Decimal, b: Decimal) -> Result<Decimal, Refusal> {
    Ok(if a.lt(&b).map_err(signal)? { a } else { b })
}

/// The composed amounts, in [`WORKING`]'s order, and the working Australians
/// tax offset applied: `wato` up to basic income tax, before every other
/// offset (ITAA 1997 s 63-10(1) item 3).
fn compose_amounts(
    results: &Map<String, Value>,
    foreign: bool,
    wato: &Decimal,
) -> Result<([Decimal; 9], Decimal), Refusal> {
    let part = |i: usize| match part_for(PARTS[i].0, foreign) {
        Some(command) => amount(results, command, PARTS[i].3),
        None => Ok(Decimal::from_int(0)),
    };
    let calc = |r: Result<Decimal, Signal>| r.map_err(signal);
    let basic = part(0)?;
    let wato = lesser(wato.clone(), basic.clone())?;
    let less_wato = calc(basic.sub(&wato))?;
    let sapto = lesser(part(1)?, less_wato.clone())?;
    let less_sapto = calc(less_wato.sub(&sapto))?;
    let lito = lesser(part(2)?, less_sapto.clone())?;
    let after = calc(less_sapto.sub(&lito))?;
    let (levy, surcharge, loan) = (part(3)?, part(4)?, part(5)?);
    let total = calc(calc(calc(after.add(&levy))?.add(&surcharge))?.add(&loan))?;
    let amounts = [
        basic,
        sapto,
        lito,
        after,
        levy,
        surcharge,
        loan,
        total,
        part(6)?,
    ];
    Ok((amounts, wato))
}

/// For a resident from 2027-28, net labour income (given, or worked out from
/// `income`'s labour inputs) and the working Australians tax offset on it,
/// with the line saying where net labour income came from; `None` otherwise,
/// when `net_labour_income` is refused.
fn working_australians_offset(
    v: &Value,
    foreign: bool,
    income: Option<&[Decimal; 4]>,
) -> Result<Option<(Decimal, Decimal, &'static str)>, Refusal> {
    let applies = !foreign && v["year"].as_str().is_some_and(|year| year >= WATO_FROM);
    let given = amount_input(v, "net_labour_income", false)?;
    if !applies {
        return match given {
            Some(_) => Err(Refusal::new(
                "input.field_unknown",
                format!(
                    "net_labour_income is not an input of {COMMAND} here: the working Australians tax offset (ITAA 1997 s 61-155) is for a resident from 2027-28"
                ),
            )),
            None => Ok(None),
        };
    }
    let calc = |r: Result<Decimal, Signal>| r.map_err(signal);
    let (net, line) = match (given, income) {
        (Some(net), _) => (net, WATO_WORKING[0].1),
        (None, Some(lines)) => {
            let labour = amount_input(v, "assessable_labour_income", false)?.expect("given");
            let work = amount_input(v, "work_related_deductions", false)?
                .unwrap_or_else(|| Decimal::from_int(0));
            let net = calc(calc(labour.sub(&work))?.sub(&lines[1]))?;
            (greater(Decimal::from_int(0), net)?, NET_LABOUR_WORKED)
        }
        (None, None) => {
            return Err(Refusal::new(
                "input.missing_field",
                "net_labour_income is required with taxable_income for a resident from 2027-28: the working Australians tax offset (ITAA 1997 s 61-160) is worked out on it",
            ));
        }
    };
    let facts = json!({"taxable_income": net.to_string(), "year": v["year"],
        "scope_confirmed": v["scope_confirmed"]});
    let tax = evaluate_coded("resident-tax", &facts)
        .map_err(|r| Refusal::new(r.code, format!("resident-tax: {}", r.message)))?;
    let tax = Decimal::parse(
        tax["amounts"]["basic_income_tax"]
            .as_str()
            .expect("an amount"),
    )
    .map_err(signal)?;
    let offset = lesser(Decimal::from_int(WATO_MAXIMUM), tax)?;
    Ok(Some((net, offset, line)))
}

/// An optional amount input, refused as every worksheet refuses an amount,
/// and as a whole number of dollars unless `cents`.
fn amount_input(v: &Value, key: &str, cents: bool) -> Result<Option<Decimal>, Refusal> {
    let refused = |e: engine::Error| Refusal::new(e.code(), e.to_string());
    let Some(given) = optional_decimal(v, key)? else {
        return Ok(None);
    };
    let amount = engine::money(&given).map_err(refused)?;
    if !cents {
        engine::whole_dollars(&amount, key).map_err(refused)?;
    }
    Ok(Some(amount))
}

fn greater(a: Decimal, b: Decimal) -> Result<Decimal, Refusal> {
    Ok(if a.lt(&b).map_err(signal)? { b } else { a })
}

/// Taxable income worked out from `assessable_labour_income` and the inputs
/// beside it, as [`INCOME_WORKING`]'s amounts; `None` when `taxable_income`
/// is given instead, which excludes them.
fn income_from_labour(v: &Value, foreign: bool) -> Result<Option<[Decimal; 4]>, Refusal> {
    let given = |key: &str| v.get(key).is_some();
    if given("taxable_income") {
        if let Some(key) = INCOME_INPUTS.iter().find(|k| given(k)) {
            return Err(Refusal::new(
                "input.field_unknown",
                format!(
                    "{key} is not an input of {COMMAND} beside taxable_income: give taxable_income, or assessable_labour_income to work it out"
                ),
            ));
        }
        return Ok(None);
    }
    if !given(INCOME_INPUTS[0]) {
        if let Some(key) = INCOME_INPUTS[1..].iter().find(|k| given(k)) {
            return Err(Refusal::new(
                "input.missing_field",
                format!("assessable_labour_income is required with {key}"),
            ));
        }
        return Err(Refusal::new(
            "input.missing_field",
            "taxable_income or assessable_labour_income is required",
        ));
    }
    let mut inputs = Vec::new();
    for key in INCOME_INPUTS {
        inputs.push(amount_input(v, key, false)?.unwrap_or_else(|| Decimal::from_int(0)));
    }
    let [labour, other_income, work, other] = <[Decimal; 4]>::try_from(inputs).expect("4 inputs");
    let calc = |r: Result<Decimal, Signal>| r.map_err(signal);
    let nil = Decimal::from_int(0);
    let from_2026_27 = v["year"]
        .as_str()
        .is_some_and(|year| year >= STANDARD_DEDUCTION_FROM);
    let standard = if foreign || !from_2026_27 {
        nil.clone()
    } else {
        let limit = lesser(Decimal::from_int(1000), labour.clone())?;
        greater(nil.clone(), calc(limit.sub(&work))?)?
    };
    let assessable = calc(labour.add(&other_income))?;
    let deductions = calc(calc(work.add(&standard))?.add(&other))?;
    let taxable = greater(nil, calc(assessable.sub(&deductions))?)?;
    Ok(Some([assessable, standard, deductions, taxable]))
}

/// The refund and amount payable when the total is compared with the tax
/// withheld, as [`CREDIT_WORKING`]'s amounts.
fn credit_lines(withheld: Decimal, total: &Decimal) -> Result<[Decimal; 3], Refusal> {
    let calc = |r: Result<Decimal, Signal>| r.map_err(signal);
    let nil = Decimal::from_int(0);
    let refund = greater(nil.clone(), calc(withheld.sub(total))?)?;
    let payable = greater(nil, calc(total.sub(&withheld))?)?;
    Ok([withheld, refund, payable])
}

/// `personal-tax-estimate`: the composed amounts, where each came from, and
/// every part's own result.
pub fn compose(v: &Value) -> Result<Value, Refusal> {
    check_fields(v)?;
    let foreign = check_residency(v)?;
    check_family(v)?;
    check_exemption(v)?;
    let income = income_from_labour(v, foreign)?;
    let wato = working_australians_offset(v, foreign, income.as_ref())?;
    let withheld = amount_input(v, "tax_withheld", true)?;
    let mut facts = v.clone();
    if let Some(lines) = &income {
        facts["taxable_income"] = json!(lines[3].to_string());
    }
    let results = run_parts(&facts, foreign)?;
    let offset = wato.as_ref().map_or(Decimal::from_int(0), |w| w.1.clone());
    let (composed, wato_applied) = compose_amounts(&results, foreign, &offset)?;
    let credits = match withheld {
        Some(withheld) => Some(credit_lines(withheld, &composed[7])?),
        None => None,
    };
    let mut part_working: Vec<(&str, &str)> = WORKING.to_vec();
    let overrides = match (foreign, &wato) {
        (true, _) => FOREIGN_WORKING.as_slice(),
        (false, Some(_)) => WATO_PART_WORKING.as_slice(),
        (false, None) => &[],
    };
    for (key, line) in overrides {
        let at = part_working.iter().position(|(k, _)| k == key);
        part_working[at.expect("a WORKING line")].1 = *line;
    }
    let mut parts: Vec<(&str, &str, Decimal)> = part_working
        .into_iter()
        .zip(composed)
        .map(|((key, line), value)| (key, line, value))
        .collect();
    if let Some((net, offset, net_line)) = wato {
        let [net_key, offset_line, applied_line] = WATO_WORKING;
        let wato_lines = [
            (net_key.0, net_line, net),
            (offset_line.0, offset_line.1, offset),
            (applied_line.0, applied_line.1, wato_applied),
        ];
        parts.splice(1..1, wato_lines);
    }
    let lines = income
        .into_iter()
        .flat_map(|lines| INCOME_WORKING.into_iter().zip(lines))
        .map(|((key, line), value)| (key, line, value))
        .chain(parts)
        .chain(credits.into_iter().flat_map(|lines| {
            CREDIT_WORKING
                .into_iter()
                .zip(lines)
                .map(|((key, line), value)| (key, line, value))
        }));
    let (mut amounts, mut working) = (Map::new(), Map::new());
    for (key, line, value) in lines {
        let cents = value.quantize(-2, Rounding::HalfUp).map_err(signal)?;
        amounts.insert(key.into(), json!(cents.to_string()));
        working.insert(key.into(), json!(line));
    }
    let not_composed: Vec<&str> = PARTS
        .iter()
        .map(|&(c, ..)| part_for(c, foreign).unwrap_or(c))
        .filter(|c| !results.contains_key(*c))
        .collect();
    let basic = &results[part_for("resident-tax", foreign).expect("basic tax")];
    Ok(json!({
        "ok": true,
        "engine": basic["engine"],
        "engine_version": basic["engine_version"],
        "calculation": "personal_tax_estimate",
        "period": basic["period"],
        "residency": v["residency"],
        "amounts": amounts,
        "working": working,
        "not_composed": not_composed,
        "components": results,
        "scope": SCOPE,
        "sources": SOURCES,
        "source_checked": "2026-11-16",
        "warnings": [
            "Operator-confirmed facts and scope; eligibility is not independently verified.",
            "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
            "Each part's own warnings are in its result under components.",
        ],
    }))
}
