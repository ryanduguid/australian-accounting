// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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

mod input;
mod kind;
mod nfp;
use input::{Given, check_benefit, check_employer, check_fields, check_year};
use kind::Kind;

const COMMAND: &str = "fbt-return";

/// What an item 23 label's Number column holds (FBT return 2026
/// instructions, item 23): at 23G the employees who received an allowance,
/// one to an employee label. 23A, 23B, 23C and 23F count cars, loans and units
/// of accommodation, which the benefits do not identify, so those are left to
/// the preparer; the other labels' Number is left blank.
#[derive(Clone, Copy)]
enum Count {
    Blank,
    Employees,
}

/// Whether an item 23 label has an employee contribution column and a value
/// of reductions column; the instructions say to leave the others blank.
#[derive(Clone, Copy)]
struct Columns {
    contribution: bool,
    reductions: bool,
}

const BOTH: Columns = Columns {
    contribution: true,
    reductions: true,
};
const CONTRIBUTION: Columns = Columns {
    contribution: true,
    reductions: false,
};
const REDUCTIONS: Columns = Columns {
    contribution: false,
    reductions: true,
};
const NEITHER: Columns = Columns {
    contribution: false,
    reductions: false,
};

/// The benefit worksheets the return composes, each with its item 23 label.
const ROWS: [(&str, &str, Count, Columns); 13] = [
    ("fbt-car-statutory", "23A", Count::Blank, CONTRIBUTION),
    ("fbt-car-operating-cost", "23B", Count::Blank, CONTRIBUTION),
    ("fbt-loan", "23C", Count::Blank, REDUCTIONS),
    ("fbt-debt-waiver", "23D", Count::Blank, NEITHER),
    ("fbt-expense-payment", "23E", Count::Blank, BOTH),
    ("fbt-housing", "23F", Count::Blank, CONTRIBUTION),
    ("fbt-lafha", "23G", Count::Employees, REDUCTIONS),
    ("fbt-board", "23J", Count::Blank, BOTH),
    ("fbt-property", "23K", Count::Blank, BOTH),
    (
        "fbt-tax-exempt-body-entertainment",
        "23L",
        Count::Blank,
        NEITHER,
    ),
    ("fbt-residual", "23M", Count::Blank, BOTH),
    ("fbt-car-parking", "23N", Count::Blank, CONTRIBUTION),
    ("fbt-meal-entertainment", "23P", Count::Blank, NEITHER),
];

/// FBTAA s 135P(1): an employee has a reportable fringe benefits amount when
/// the individual fringe benefits amount is more than $2,000.
const REPORTING_THRESHOLD: i64 = 2000;

const SCOPE: &str = "One ordinary taxable employer's fringe benefits for one FBT year, each valued by its own benefit worksheet (fbt-car-statutory, fbt-car-operating-cost, fbt-loan, fbt-debt-waiver, fbt-expense-payment, fbt-housing, fbt-lafha, fbt-board, fbt-property, fbt-tax-exempt-body-entertainment, fbt-residual, fbt-car-parking or fbt-meal-entertainment) on the inputs given, with the employee it relates to, whether it is a GST-creditable benefit (type 1) or not (type 2), and whether it is an excluded fringe benefit under FBTAA s 5E(3), all established by the operator. Confirming scope confirms the scope each worksheet states under components. Adds each benefit's taxable value, excluded benefits included, into the type 1 and type 2 aggregate fringe benefits amounts (s 5C(3) and (4)) and gives them to fbt, which grosses them up and works out the fringe benefits taxable amount and the tax (s 5B(1A) to (1C)). Each employee's individual fringe benefits amount is the sum of the taxable values of their benefits that are not excluded (s 5E(2)), an exempt electric car's notional value included (s 135P(3)); where it is more than $2,000, the reportable fringe benefits amount is that sum times 1 / (1 - the FBT rate) (s 135P(1) and (2)), worked out from fbt's FBT rate as the formula states, not from the 1.8868 the ATO publishes for it, which can give a dollar more; it is given in whole dollars with the cents dropped, which no provision states: the ATO's examples do so (the employer's guide, 5.4, gives $3,500 / (1 - 0.47) as $6,603). Item 23G's Number counts the employee labels, each taken as one employee; the Number at 23A, 23B, 23C and 23F, a count of cars, loans and units of accommodation, is left to the preparer. Each benefit belongs wholly to one employee label: a benefit shared by employees or valued with others in a class (s 5F(3) and (4)), and a benefit partly GST-creditable, are given as separate benefits by the operator. Excess employee contributions are not applied to other benefits. Not for a rebatable employer, a public or not-for-profit hospital, a public ambulance service or an employer exempt under s 57A or s 58 (the return's items 14C, 17 and 18, and s 135Q, are not worked out); not for amortised or reducible fringe benefits, or an employer using s 135G. Does not work out a payroll tax gross-up: payroll tax is imposed by each state and territory under its own Act, which this command does not read. Choosing between providing a benefit and an employee contribution is planning, outside a worksheet.";

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
        "individual_fringe_benefits_amount x 1 / (1 - fbt's fbt_rate), where individual_fringe_benefits_amount is more than 2000, otherwise nil (FBTAA s 135P(1) and (2)); in whole dollars with the cents dropped, as the ATO's examples give it (the employer's guide, 5.4), no provision stating a rounding",
    ),
    (
        "employee_contributions",
        "sum of the employee_contribution_applied of each of the employee's benefits",
    ),
];

/// The return's labels this command leaves to the preparer, and why.
const LEFT_TO_PREPARER: [(&str, &str); 6] = [
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
    (
        "23A, 23B, 23C and 23F Number",
        "the number of cars, loans and units of accommodation: one car, loan or unit can be given as several benefits, such as a car shared by employees or held for 2 periods, and the benefits do not identify it",
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
    // The method takes a required statutory formula value and always works
    // both values (engine.rs, S10_PAYMENT_STEPS and S10_5_STEPS).
    let car_amount = |key: &str| amount(&valued.result, key).map(|a| a.expect("the car's amount"));
    let operating = car_amount("operating_cost_taxable_value")?;
    let statutory = car_amount("statutory_formula_taxable_value")?;
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
fn run_benefits<'a>(
    v: &'a Value,
    employer: &str,
    year: i64,
    kind: Kind,
) -> Result<Vec<Valued<'a>>, Refusal> {
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
            (year, kind),
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
/// no place there, nor, for a s 57A employer, a benefit the caps disregard.
/// A column the instructions leave blank is absent.
fn item_23(valued: &[Valued], kind: Kind) -> Result<Map<String, Value>, Refusal> {
    let capped = matches!(kind, Kind::Exempt(_));
    let mut rows: BTreeMap<&str, (Count, Columns, Row)> = BTreeMap::new();
    for b in valued
        .iter()
        .filter(|b| !b.exempt && !(capped && nfp::disregarded(b)))
    {
        let (.., row) =
            rows.entry(b.given.label)
                .or_insert((b.given.count, b.given.columns, Row::default()));
        row.employees.insert(b.given.employee.to_string());
        row.gross.push(b.gross.clone());
        row.contribution.push(b.contribution.clone());
        row.taxable.push(b.taxable.clone());
    }
    let mut out = Map::new();
    for (label, (count, columns, row)) in rows {
        let (gross, contribution, taxable) = (
            sum(&row.gross)?,
            sum(&row.contribution)?,
            sum(&row.taxable)?,
        );
        let reductions = calc(calc(gross.sub(&contribution))?.sub(&taxable))?;
        let mut cells = Map::new();
        match count {
            Count::Blank => {}
            Count::Employees => {
                cells.insert("number".into(), json!(row.employees.len()));
            }
        }
        cells.insert("gross_taxable_value".into(), cents(&gross)?);
        if columns.contribution {
            cells.insert("employee_contribution".into(), cents(&contribution)?);
        }
        if columns.reductions {
            cells.insert("value_of_reductions".into(), cents(&reductions)?);
        }
        cells.insert("taxable_value".into(), cents(&taxable)?);
        out.insert(label.into(), Value::Object(cells));
    }
    Ok(out)
}

/// Each employee's individual fringe benefits amount (FBTAA s 5E(2), with
/// s 135P(3)) and reportable fringe benefits amount (s 135P(1) and (2)), and
/// the contributions applied to their benefits.
fn employees(valued: &[Valued], fbt_rate: &Decimal) -> Result<Map<String, Value>, Refusal> {
    let mut by_employee: BTreeMap<&str, (Vec<Decimal>, Vec<Decimal>)> = BTreeMap::new();
    for b in valued {
        let (individual, contributions) = by_employee.entry(b.given.employee).or_default();
        if !b.given.excluded {
            individual.push(b.reportable.clone());
        }
        contributions.push(b.contribution.clone());
    }
    let threshold = Decimal::from_int(REPORTING_THRESHOLD);
    let kept = calc(Decimal::from_int(1).sub(fbt_rate))?;
    let mut out = Map::new();
    for (employee, (individual, contributions)) in by_employee {
        let individual = sum(&individual)?;
        let reportable = if individual.gt(&threshold).map_err(signal)? {
            // s 135P(2): individual x 1 / (1 - rate), as one division so the
            // cents are dropped from the formula's own amount.
            let grossed = calc(individual.div(&kept))?;
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
fn return_labels(
    fbt: &Value,
    valued: &[Valued],
    kind: Kind,
) -> Result<Map<String, Value>, Refusal> {
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
    labels.extend(item_23(valued, kind)?);
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
    let mut warnings = Vec::new();
    if year > LABELS_YEAR {
        warnings.push(json!(format!(
            "The ATO had not published the FBT return {year} instructions when read on 16 November 2026; the labels follow the FBT return {LABELS_YEAR} instructions, and the {year} form may differ."
        )));
    }
    warnings.extend([
        json!("Operator-confirmed facts and scope; eligibility is not independently verified."),
        json!("Worksheet only, not advice, an assessment or a lodgment. Obtain human review."),
        json!("Each worksheet's own warnings are in its result under components."),
    ]);
    warnings
}

/// `fbt-return`: the composed amounts, each employee's reportable amount, the
/// return's labels, and every worksheet's own result.
pub fn compose(v: &Value) -> Result<Value, Refusal> {
    check_fields(v)?;
    let employer = check_employer(v)?;
    let year = check_year(v)?;
    let (kind_name, kind) = kind::check_kind(v)?;
    let days = kind::check_days(v, kind, year)?;
    let instalments = kind::check_instalments(v)?;
    let valued = run_benefits(v, employer, year, kind)?;
    let totals = aggregates(&valued)?;
    let fbt = gross_up(v, year, &totals[0], &totals[1])?;
    let fbt_rate = calc(Decimal::parse(
        fbt["rates"]["fbt_rate"].as_str().expect("fbt's FBT rate"),
    ));
    let fbt_rate = fbt_rate?;
    let (amounts, reconciliation) = amounts(&valued, &fbt, &totals)?;
    let employees = employees(&valued, &fbt_rate)?;
    let labels = return_labels(&fbt, &valued, kind)?;
    let composed = nfp::apply(kind, v, year, &valued, &fbt, days)?;
    let results: Vec<Value> = valued.into_iter().map(|b| b.result).collect();
    let mut out = json!({
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
    });
    if let Some(composed) = composed {
        nfp::merge(&mut out, kind_name, composed);
    }
    nfp::instalment_labels(&mut out, instalments)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_decimal_signal_is_refused_with_its_code_and_name() {
        let refusal = signal(Signal::DivisionByZero);
        assert_eq!(refusal.code, "decimal.division_by_zero");
        assert_eq!(refusal.message, "decimal.DivisionByZero");
    }
}
