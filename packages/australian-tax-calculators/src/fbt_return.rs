// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `fbt-return`: one employer's fringe benefits for one FBT year, composed into
//! the FBT return. Each benefit's taxable value comes from its own worksheet
//! through [`evaluate_coded`], unchanged, and `fbt` grosses up the type 1 and
//! type 2 totals; this module only sorts and adds the benefits, works out each
//! employee's reportable fringe benefits amount and lays the amounts out by
//! the return's labels. It cites the FBTAA as compiled on 1 July 2026
//! (compilation 97, C2026C00332) and the ATO pages in [`SOURCES`], all read
//! on 16 November 2026.
use crate::api::{Refusal, evaluate_coded};
use crate::decimal::{Decimal, Rounding, Signal};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

const COMMAND: &str = "fbt-return";

/// What an item 23 label's Number column counts (FBT return 2026
/// instructions, item 23): cars, loans and units of accommodation, one to a
/// worksheet run, or the employees who received an allowance.
#[derive(Clone, Copy)]
enum Count {
    Blank,
    Benefits,
    Employees,
}

/// The benefit worksheets the return composes, each with its item 23 label.
const ROWS: [(&str, &str, Count); 13] = [
    ("fbt-car-statutory", "23A", Count::Benefits),
    ("fbt-car-operating-cost", "23B", Count::Benefits),
    ("fbt-loan", "23C", Count::Benefits),
    ("fbt-debt-waiver", "23D", Count::Blank),
    ("fbt-expense-payment", "23E", Count::Blank),
    ("fbt-housing", "23F", Count::Benefits),
    ("fbt-lafha", "23G", Count::Employees),
    ("fbt-board", "23J", Count::Blank),
    ("fbt-property", "23K", Count::Blank),
    ("fbt-tax-exempt-body-entertainment", "23L", Count::Blank),
    ("fbt-residual", "23M", Count::Blank),
    ("fbt-car-parking", "23N", Count::Blank),
    ("fbt-meal-entertainment", "23P", Count::Blank),
];

/// Benefits FBTAA s 5E(3) always excludes from reporting, as their worksheets
/// value them.
const ALWAYS_EXCLUDED: [(&str, &str); 2] = [
    (
        "fbt-car-parking",
        "a car parking fringe benefit is an excluded fringe benefit (FBTAA s 5E(3)(b))",
    ),
    (
        "fbt-meal-entertainment",
        "meal entertainment not provided under a salary packaging arrangement, the only kind fbt-meal-entertainment values, is an excluded fringe benefit (FBTAA s 5E(3)(a))",
    ),
];

/// The inputs the return takes, and those each benefit takes.
const INPUTS: [&str; 4] = ["employer", "year_ended", "scope_confirmed", "benefits"];
const BENEFIT_INPUTS: [&str; 6] = [
    "employee", "command", "input", "gst_type", "excluded", "employer",
];

/// FBTAA s 135P(1): an employee has a reportable fringe benefits amount when
/// the individual fringe benefits amount is more than $2,000.
const REPORTING_THRESHOLD: i64 = 2000;

const SCOPE: &str = "One ordinary taxable employer's fringe benefits for one FBT year, each valued by its own benefit worksheet (fbt-car-statutory, fbt-car-operating-cost, fbt-loan, fbt-debt-waiver, fbt-expense-payment, fbt-housing, fbt-lafha, fbt-board, fbt-property, fbt-tax-exempt-body-entertainment, fbt-residual, fbt-car-parking or fbt-meal-entertainment) on the inputs given, with the employee it relates to, whether it is a GST-creditable benefit (type 1) or not (type 2), and whether it is an excluded fringe benefit under FBTAA s 5E(3), all established by the operator. Confirming scope confirms the scope each worksheet states under components. Adds each benefit's taxable value, excluded benefits included, into the type 1 and type 2 aggregate fringe benefits amounts (s 5C(3) and (4)) and gives them to fbt, which grosses them up and works out the fringe benefits taxable amount and the tax (s 5B(1A) to (1C)). Each employee's individual fringe benefits amount is the sum of the taxable values of their benefits that are not excluded (s 5E(2)), an exempt electric car's notional value included (s 135P(3)); where it is more than $2,000, the reportable fringe benefits amount is that sum times 1 / (1 - the FBT rate) (s 135P(1) and (2)), taken as fbt's type 2 gross-up rate, the 1.8868 the ATO publishes for a 47% rate, and given in whole dollars with cents dropped, as the ATO's examples give it. Each benefit belongs wholly to one employee label: a benefit shared by employees or valued with others in a class (s 5F(3) and (4)), and a benefit partly GST-creditable, are given as separate benefits by the operator. Excess employee contributions are not applied to other benefits. Not for a rebatable employer, a public or not-for-profit hospital, a public ambulance service or an employer exempt under s 57A or s 58 (the return's items 14C, 17 and 18, and s 135Q, are not worked out); not for amortised or reducible fringe benefits, or an employer using s 135G. Does not work out a payroll tax gross-up: payroll tax is imposed by each state and territory under its own Act, which this command does not read. Choosing between providing a benefit and an employee contribution is planning, outside a worksheet.";

const SOURCES: [&str; 7] = [
    "https://www.legislation.gov.au/C2004A03280/2026-07-01/2026-07-01/text",
    "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/fbt-return-2026-calculation-details-for-taxable-employers",
    "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/item-23-fringe-benefit-categories-for-fbt-return-2026",
    "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/completing-your-fbt-return-2026-all-employers",
    "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/fbt-registration-lodgment-payment-and-reporting/reportable-fringe-benefits",
    "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00006",
    "https://www.ato.gov.au/tax-rates-and-codes/fringe-benefits-tax-rates-and-thresholds",
];

/// Where each amount comes from, in the amounts' order.
const WORKING: [(&str, &str); 7] = [
    (
        "type_one_value",
        "sum of the taxable_value of each type 1 benefit, excluded benefits included (FBTAA s 5C(3) steps 1 to 4)",
    ),
    (
        "type_two_value",
        "sum of the taxable_value of each type 2 benefit, excluded benefits included (FBTAA s 5C(4) steps 1 to 4)",
    ),
    (
        "type_one_grossed_up",
        "fbt: type_one_grossed_up (FBTAA s 5B(1B))",
    ),
    (
        "type_two_grossed_up",
        "fbt: type_two_grossed_up (FBTAA s 5B(1C))",
    ),
    (
        "fringe_benefits_taxable_amount",
        "fbt: return_item_15, items 14A and 14B added (FBTAA s 5B(1A))",
    ),
    ("fbt_payable", "fbt: return_item_16 (FBTAA s 66)"),
    (
        "employee_contributions",
        "sum of each benefit's employee_contribution_applied",
    ),
];

/// Where each employee's amounts come from.
const EMPLOYEE_WORKING: [(&str, &str); 3] = [
    (
        "individual_fringe_benefits_amount",
        "sum of the taxable_value of each of the employee's benefits not excluded, an exempt electric car's reportable_taxable_value in place of its nil (FBTAA s 5E(2) and s 135P(3))",
    ),
    (
        "reportable_fringe_benefits_amount",
        "individual_fringe_benefits_amount x fbt's type_two rate, in whole dollars with cents dropped, where individual_fringe_benefits_amount is more than 2000; otherwise nil (FBTAA s 135P(1) and (2))",
    ),
    (
        "employee_contributions",
        "sum of the employee_contribution_applied of each of the employee's benefits",
    ),
];

/// The return's labels this command leaves to the preparer, and why.
const LEFT_TO_PREPARER: [(&str, &str); 5] = [
    (
        "1 to 9, 11 to 13",
        "the employer's identity, contact, hours, future lodgment and EFT details, which this command does not take",
    ),
    (
        "10",
        "the number of employees and associates, current, former and future, who received fringe benefits: a benefit's employee label may stand for several people, and associates are not counted apart",
    ),
    (
        "14C, 17 and 18",
        "left blank by a taxable employer; they apply to rebatable employers, public and not-for-profit hospitals, public ambulance services and eligible public benevolent institutions and health promotion charities, which this command does not compose",
    ),
    (
        "20 to 22",
        "the FBT instalments reported on activity statements, and the payment or credit that follows from them, which this command does not take",
    ),
    ("24 and 25", "the tax agent's or employer's declaration"),
];

/// The FBT return 2026 instructions are the latest the ATO had published when
/// read; a later year's labels follow them until its own are read.
const LABELS_YEAR: i64 = 2026;

fn signal(signal: Signal) -> Refusal {
    Refusal::new(signal.code(), format!("decimal.{}", signal.name()))
}

fn calc(result: Result<Decimal, Signal>) -> Result<Decimal, Refusal> {
    result.map_err(signal)
}

fn missing(message: String) -> Refusal {
    Refusal::new("input.missing_field", message)
}

fn cents(value: &Decimal) -> Result<Value, Refusal> {
    Ok(json!(
        calc(value.quantize(-2, Rounding::HalfUp))?.to_string()
    ))
}

/// An amount a composed result gives, or `None` when it gives none.
fn amount(result: &Value, key: &str) -> Result<Option<Decimal>, Refusal> {
    result["amounts"][key]
        .as_str()
        .map(|text| calc(Decimal::parse(text)))
        .transpose()
}

/// Refuse a field the return does not take, naming the payroll tax gross-up
/// apart.
fn check_fields(v: &Value) -> Result<(), Refusal> {
    for key in v.as_object().into_iter().flat_map(|o| o.keys()) {
        if key.starts_with("payroll_tax") {
            return Err(Refusal::new(
                "fbt_return.payroll_tax_unsupported",
                format!(
                    "{COMMAND} does not work out a payroll tax gross-up: payroll tax is imposed by each state and territory under its own Act, which this command does not read"
                ),
            ));
        }
        if !INPUTS.contains(&key.as_str()) {
            return Err(Refusal::new(
                "input.field_unknown",
                format!("{key} is not an input of {COMMAND}"),
            ));
        }
    }
    Ok(())
}

/// The one employer's label.
fn check_employer(v: &Value) -> Result<&str, Refusal> {
    match v.get("employer") {
        Some(Value::String(label)) if !label.is_empty() => Ok(label),
        Some(Value::Array(labels)) if labels.len() > 1 => Err(Refusal::new(
            "fbt_return.multiple_employers",
            format!(
                "{COMMAND} composes one employer's return: employer gives {} employers",
                labels.len()
            ),
        )),
        _ => Err(missing(
            "employer is required: a label for the one employer whose return this is".into(),
        )),
    }
}

/// The FBT year, one the repository has loaded `fbt`'s rates for.
fn check_year(v: &Value) -> Result<i64, Refusal> {
    let Some(given) = v.get("year_ended") else {
        return Err(missing(
            "year_ended is required: the year the FBT year ends on 31 March".into(),
        ));
    };
    let supported = crate::metadata::supported_periods("fbt");
    match given.as_i64() {
        Some(year) if supported.contains(&year.to_string().as_str()) => Ok(year),
        _ => Err(Refusal::new(
            "period.unsupported",
            format!(
                "{COMMAND} composes the FBT years ended 31 March {}, whose rates are loaded; year_ended {given} is not one of them",
                supported.join(" and ")
            ),
        )),
    }
}

/// One benefit as given: its worksheet, how it is classified and the
/// worksheet's input with the shared year and scope added.
struct Given<'a> {
    employee: &'a str,
    command: &'static str,
    label: &'static str,
    count: Count,
    type_one: bool,
    excluded: bool,
    input: Value,
}

/// The benefit's worksheet and its item 23 label.
fn row_for(at: &str, b: &Value) -> Result<(&'static str, &'static str, Count), Refusal> {
    let named = b.get("command").and_then(Value::as_str);
    match ROWS.iter().find(|(c, ..)| Some(*c) == named) {
        Some(row) => Ok(*row),
        None => Err(Refusal::new(
            "fbt_return.benefit_command_unsupported",
            format!(
                "{at}.command must name the benefit worksheet that values it, one of {}",
                ROWS.map(|(c, ..)| c).join(", ")
            ),
        )),
    }
}

/// Whether the benefit is type 1, as stated.
fn gst_type(at: &str, b: &Value) -> Result<bool, Refusal> {
    match b.get("gst_type").and_then(Value::as_str) {
        Some("type_1") => Ok(true),
        Some("type_2") => Ok(false),
        _ => Err(Refusal::new(
            "fbt_return.gst_type_missing",
            format!(
                "{at}.gst_type is required: type_1 for a GST-creditable benefit (FBTAA s 5C(3) step 1, s 149A), whose provider was entitled to an input tax credit for providing it, or type_2 for any other (s 5C(4))"
            ),
        )),
    }
}

/// Whether the benefit is excluded from reporting, as stated and as the law
/// fixes it for car parking and meal entertainment.
fn excluded(at: &str, command: &str, b: &Value) -> Result<bool, Refusal> {
    let excluded = match b.get("excluded") {
        None => {
            return Err(missing(format!(
                "{at}.excluded is required: whether the benefit is an excluded fringe benefit (FBTAA s 5E(3)), left out of reporting"
            )));
        }
        Some(Value::Bool(flag)) => *flag,
        Some(_) => {
            return Err(Refusal::new(
                "input.flag_invalid",
                format!("{at}.excluded must be a boolean"),
            ));
        }
    };
    match ALWAYS_EXCLUDED.iter().find(|(c, _)| *c == command) {
        Some((_, why)) if !excluded => Err(Refusal::new(
            "fbt_return.exclusion_conflict",
            format!("{at}.excluded must be true: {why}"),
        )),
        _ => Ok(excluded),
    }
}

/// The worksheet's own input, with the return's year and scope added. A year
/// given inside it must be the return's.
fn worksheet_input(at: &str, b: &Value, top: &Value, year: i64) -> Result<Value, Refusal> {
    let Some(mut input) = b.get("input").filter(|i| i.is_object()).cloned() else {
        return Err(missing(format!(
            "{at}.input is required: an object of the worksheet's own inputs"
        )));
    };
    if input.get("scope_confirmed").is_some() {
        return Err(Refusal::new(
            "input.field_unknown",
            format!(
                "scope_confirmed is not an input of {at} within {COMMAND}: it is given once, for every benefit"
            ),
        ));
    }
    if let Some(other) = input.get("year_ended").filter(|y| y.as_i64() != Some(year)) {
        return Err(Refusal::new(
            "fbt_return.mixed_years",
            format!(
                "{at}.input.year_ended is {other}, but the return is for the FBT year ended 31 March {year}: one return covers one FBT year"
            ),
        ));
    }
    input["year_ended"] = json!(year);
    input["scope_confirmed"] = top["scope_confirmed"].clone();
    Ok(input)
}

/// One benefit, checked before any worksheet runs.
fn check_benefit<'a>(
    at: &str,
    b: &'a Value,
    top: &Value,
    employer: &str,
    year: i64,
) -> Result<Given<'a>, Refusal> {
    let Some(fields) = b.as_object() else {
        return Err(Refusal::new(
            "input.not_object",
            format!("{at} must be a JSON object"),
        ));
    };
    if let Some(key) = fields
        .keys()
        .find(|k| !BENEFIT_INPUTS.contains(&k.as_str()))
    {
        return Err(Refusal::new(
            "input.field_unknown",
            format!("{key} is not an input of a benefit within {COMMAND}"),
        ));
    }
    if let Some(other) = b.get("employer").filter(|e| e.as_str() != Some(employer)) {
        return Err(Refusal::new(
            "fbt_return.multiple_employers",
            format!(
                "{at}.employer is {other}, not {employer}: {COMMAND} composes one employer's return"
            ),
        ));
    }
    let (command, label, count) = row_for(at, b)?;
    let Some(employee) = b["employee"].as_str().filter(|e| !e.is_empty()) else {
        return Err(missing(format!(
            "{at}.employee is required: a label for the employee the benefit is provided in respect of"
        )));
    };
    Ok(Given {
        employee,
        command,
        label,
        count,
        type_one: gst_type(at, b)?,
        excluded: excluded(at, command, b)?,
        input: worksheet_input(at, b, top, year)?,
    })
}

/// A benefit's amounts once its worksheet has run.
struct Valued<'a> {
    given: Given<'a>,
    result: Value,
    taxable: Decimal,
    reportable: Decimal,
    contribution: Decimal,
    gross: Decimal,
    exempt: bool,
}

/// FBTAA s 10(5): where the operating cost value exceeds the statutory
/// formula value, the car takes the s 9 value, whose gross value and
/// contribution only fbt-car-statutory gives.
fn check_operating_cost(at: &str, valued: &Valued) -> Result<(), Refusal> {
    if valued.given.command != "fbt-car-operating-cost" || valued.exempt {
        return Ok(());
    }
    let (Some(operating), Some(statutory)) = (
        amount(&valued.result, "operating_cost_taxable_value")?,
        amount(&valued.result, "statutory_formula_taxable_value")?,
    ) else {
        return Ok(());
    };
    if !operating.gt(&statutory).map_err(signal)? {
        return Ok(());
    }
    Err(Refusal::new(
        "fbt_return.car_statutory_lower",
        format!(
            "{at}: the operating cost value exceeds the statutory formula value, so FBTAA s 10(5) gives the car its s 9 value; give the car to fbt-car-statutory, whose gross value and employee contribution item 23 shows"
        ),
    ))
}

/// Run the benefit's worksheet and read the amounts the return uses.
fn value_benefit<'a>(at: &str, given: Given<'a>) -> Result<Valued<'a>, Refusal> {
    let result = evaluate_coded(given.command, &given.input)
        .map_err(|r| Refusal::new(r.code, format!("{at} {}: {}", given.command, r.message)))?;
    let nil = Decimal::from_int(0);
    let taxable = amount(&result, "taxable_value")?.expect("a benefit worksheet's taxable value");
    let reportable = amount(&result, "reportable_taxable_value")?.unwrap_or(taxable.clone());
    let contribution = amount(&result, "employee_contribution_applied")?.unwrap_or(nil.clone());
    // A living-away-from-home allowance shows its exempt components among
    // item 23G's reductions; the allowance is its gross value.
    let gross = match amount(&result, "gross_taxable_value")? {
        Some(gross) => gross,
        None => {
            let accommodation = amount(&result, "exempt_accommodation_component")?;
            let food = amount(&result, "exempt_food_component")?;
            let with = calc(taxable.add(&accommodation.unwrap_or(nil.clone())))?;
            calc(with.add(&food.unwrap_or(nil)))?
        }
    };
    let exempt = given.input["electric_car_exempt"] == json!(true);
    let valued = Valued {
        given,
        result,
        taxable,
        reportable,
        contribution,
        gross,
        exempt,
    };
    check_operating_cost(at, &valued)?;
    Ok(valued)
}

/// Check every benefit, then value each one.
fn run_benefits<'a>(v: &'a Value, employer: &str, year: i64) -> Result<Vec<Valued<'a>>, Refusal> {
    let benefits = match v.get("benefits").and_then(Value::as_array) {
        Some(list) if !list.is_empty() => list,
        _ => {
            return Err(missing(
                "benefits is required: a list of at least one benefit".into(),
            ));
        }
    };
    let mut given = Vec::new();
    for (i, b) in benefits.iter().enumerate() {
        given.push(check_benefit(
            &format!("benefits[{i}]"),
            b,
            v,
            employer,
            year,
        )?);
    }
    given
        .into_iter()
        .enumerate()
        .map(|(i, g)| value_benefit(&format!("benefits[{i}]"), g))
        .collect()
}

/// One item 23 label's columns.
#[derive(Default)]
struct Row {
    benefits: usize,
    employees: BTreeSet<String>,
    gross: Vec<Decimal>,
    contribution: Vec<Decimal>,
    taxable: Vec<Decimal>,
}

fn sum(values: &[Decimal]) -> Result<Decimal, Refusal> {
    values
        .iter()
        .try_fold(Decimal::from_int(0), |total, v| calc(total.add(v)))
}

/// Item 23, by label: an exempt electric car is not a fringe benefit and has
/// no place there.
fn item_23(valued: &[Valued]) -> Result<Map<String, Value>, Refusal> {
    let mut rows: BTreeMap<&str, (Count, Row)> = BTreeMap::new();
    for b in valued.iter().filter(|b| !b.exempt) {
        let (_, row) = rows
            .entry(b.given.label)
            .or_insert((b.given.count, Row::default()));
        row.benefits += 1;
        row.employees.insert(b.given.employee.to_string());
        row.gross.push(b.gross.clone());
        row.contribution.push(b.contribution.clone());
        row.taxable.push(b.taxable.clone());
    }
    let mut out = Map::new();
    for (label, (count, row)) in rows {
        let (gross, contribution, taxable) = (
            sum(&row.gross)?,
            sum(&row.contribution)?,
            sum(&row.taxable)?,
        );
        let reductions = calc(calc(gross.sub(&contribution))?.sub(&taxable))?;
        let number = match count {
            Count::Blank => Value::Null,
            Count::Benefits => json!(row.benefits),
            Count::Employees => json!(row.employees.len()),
        };
        out.insert(
            label.into(),
            json!({
                "number": number,
                "gross_taxable_value": cents(&gross)?,
                "employee_contribution": cents(&contribution)?,
                "value_of_reductions": cents(&reductions)?,
                "taxable_value": cents(&taxable)?,
            }),
        );
    }
    Ok(out)
}

/// Each employee's individual fringe benefits amount (FBTAA s 5E(2), with
/// s 135P(3)) and reportable fringe benefits amount (s 135P(1) and (2)), and
/// the contributions applied to their benefits.
fn employees(valued: &[Valued], type_two_rate: &Decimal) -> Result<Map<String, Value>, Refusal> {
    let mut by_employee: BTreeMap<&str, (Vec<Decimal>, Vec<Decimal>)> = BTreeMap::new();
    for b in valued {
        let (individual, contributions) = by_employee.entry(b.given.employee).or_default();
        if !b.given.excluded {
            individual.push(b.reportable.clone());
        }
        contributions.push(b.contribution.clone());
    }
    let threshold = Decimal::from_int(REPORTING_THRESHOLD);
    let mut out = Map::new();
    for (employee, (individual, contributions)) in by_employee {
        let individual = sum(&individual)?;
        let reportable = if individual.gt(&threshold).map_err(signal)? {
            let grossed = calc(individual.mul(type_two_rate))?;
            calc(grossed.quantize(0, Rounding::Down))?
        } else {
            Decimal::from_int(0)
        };
        out.insert(
            employee.into(),
            json!({
                "individual_fringe_benefits_amount": cents(&individual)?,
                "reportable_fringe_benefits_amount": cents(&reportable)?,
                "employee_contributions": cents(&sum(&contributions)?)?,
            }),
        );
    }
    Ok(out)
}

/// The type 1 and type 2 aggregate amounts and every benefit's taxable value.
fn aggregates(valued: &[Valued]) -> Result<[Decimal; 3], Refusal> {
    let of = |type_one: bool| -> Vec<Decimal> {
        valued
            .iter()
            .filter(|b| b.given.type_one == type_one)
            .map(|b| b.taxable.clone())
            .collect()
    };
    let all: Vec<Decimal> = valued.iter().map(|b| b.taxable.clone()).collect();
    Ok([sum(&of(true))?, sum(&of(false))?, sum(&all)?])
}

/// `fbt` on the aggregate amounts.
fn gross_up(v: &Value, year: i64, one: &Decimal, two: &Decimal) -> Result<Value, Refusal> {
    let input = json!({
        "type_one_value": calc(one.quantize(-2, Rounding::HalfUp))?.to_string(),
        "type_two_value": calc(two.quantize(-2, Rounding::HalfUp))?.to_string(),
        "year_ended": year,
        "scope_confirmed": v["scope_confirmed"],
    });
    evaluate_coded("fbt", &input).map_err(|r| Refusal::new(r.code, format!("fbt: {}", r.message)))
}

/// The amounts in [`WORKING`]'s order, with the reconciliation of the
/// composed totals to the benefits.
fn amounts(
    valued: &[Valued],
    fbt: &Value,
    totals: &[Decimal; 3],
) -> Result<(Map<String, Value>, Value), Refusal> {
    let fbt_amount = |key: &str| amount(fbt, key).map(|a| a.expect("fbt's amount"));
    let contributions: Vec<Decimal> = valued.iter().map(|b| b.contribution.clone()).collect();
    let values = [
        totals[0].clone(),
        totals[1].clone(),
        fbt_amount("type_one_grossed_up")?,
        fbt_amount("type_two_grossed_up")?,
        fbt_amount("return_item_15")?,
        fbt_amount("return_item_16")?,
        sum(&contributions)?,
    ];
    let mut out = Map::new();
    for ((key, _), value) in WORKING.iter().zip(&values) {
        out.insert((*key).into(), cents(value)?);
    }
    let composed = calc(totals[0].add(&totals[1]))?;
    let reconciliation = json!({
        "benefits_taxable_value": cents(&totals[2])?,
        "type_one_value_plus_type_two_value": cents(&composed)?,
        "difference": cents(&calc(totals[2].sub(&composed))?)?,
    });
    Ok((out, reconciliation))
}

/// The amounts by the return's labels.
fn return_labels(fbt: &Value, valued: &[Valued]) -> Result<Map<String, Value>, Refusal> {
    let mut labels = Map::new();
    for (label, key) in [
        ("14A", "return_item_14a"),
        ("14B", "return_item_14b"),
        ("15", "return_item_15"),
        ("16", "return_item_16"),
        ("19", "return_item_16"),
    ] {
        labels.insert(label.into(), fbt["amounts"][key].clone());
    }
    labels.extend(item_23(valued)?);
    Ok(labels)
}

/// A table of (key, text) lines as a JSON object.
fn lines(table: &[(&str, &str)]) -> Map<String, Value> {
    table
        .iter()
        .map(|(key, text)| ((*key).to_string(), json!(text)))
        .collect()
}

fn warnings(year: i64) -> Vec<Value> {
    let mut warnings = vec![
        json!("Operator-confirmed facts and scope; eligibility is not independently verified."),
        json!("Worksheet only, not advice, an assessment or a lodgment. Obtain human review."),
        json!("Each worksheet's own warnings are in its result under components."),
    ];
    if year > LABELS_YEAR {
        warnings.push(json!(format!(
            "The ATO had not published the FBT return {year} instructions when read on 16 November 2026; the labels follow the FBT return {LABELS_YEAR} instructions."
        )));
    }
    warnings
}

/// `fbt-return`: the composed amounts, each employee's reportable amount, the
/// return's labels, and every worksheet's own result.
pub fn compose(v: &Value) -> Result<Value, Refusal> {
    check_fields(v)?;
    let employer = check_employer(v)?;
    let year = check_year(v)?;
    let valued = run_benefits(v, employer, year)?;
    let totals = aggregates(&valued)?;
    let fbt = gross_up(v, year, &totals[0], &totals[1])?;
    let type_two_rate = calc(Decimal::parse(
        fbt["rates"]["type_two"]
            .as_str()
            .expect("fbt's type 2 rate"),
    ))?;
    let (amounts, reconciliation) = amounts(&valued, &fbt, &totals)?;
    let employees = employees(&valued, &type_two_rate)?;
    let labels = return_labels(&fbt, &valued)?;
    let results: Vec<Value> = valued.into_iter().map(|b| b.result).collect();
    Ok(json!({
        "ok": true,
        "engine": fbt["engine"],
        "engine_version": fbt["engine_version"],
        "calculation": "fbt_return",
        "period": fbt["period"],
        "employer": employer,
        "amounts": amounts,
        "working": lines(&WORKING),
        "reconciliation": reconciliation,
        "employees": employees,
        "employee_working": lines(&EMPLOYEE_WORKING),
        "return_labels": labels,
        "left_to_preparer": lines(&LEFT_TO_PREPARER),
        "components": {"fbt": fbt, "benefits": results},
        "scope": SCOPE,
        "sources": SOURCES,
        "source_checked": "2026-11-16",
        "warnings": warnings(year),
    }))
}
