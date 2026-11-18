// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The hirer's input tax credit for an agreement entered into from 1 July
//! 2012, when every fact it turns on is given. The citations are to the GST
//! Act compilation 96 (1 January 2026) and the ATO's 'GST – Hire purchase and
//! leasing' (last updated 6 April 2017), 'Purchasing a motor vehicle' (last
//! updated 3 June 2026) and GST issues register 'Input tax credits and hire
//! purchase - acquisition of a car', all read on 18 November 2026.

use serde_json::{Map, Value, json};

use crate::decimal::Decimal;
use crate::engine::trade_in::CAR_LIMITS;
use crate::engine::{Arg, Result, Step, d, finite_between, int, money, py_min, refuse, step};

/// The facts every registered hirer states, then a passenger car's.
const REGISTERED: [&str; 3] = ["accounting_basis", "creditable_purpose", "asset"];
const CAR: [&str; 4] = [
    "first_used_year",
    "gst_inclusive_market_value",
    "luxury_car_tax",
    "quotes_abn_for_luxury_car_tax",
];

/// A passenger car's facts for GST Act s 69-10.
pub(super) struct Car {
    first_used_year: &'static str,
    limit: Decimal,
    market_value: Decimal,
    luxury_car_tax: Decimal,
    quotes_abn: bool,
}

/// The GST facts as given.
pub(super) enum Facts {
    /// No figure: the facts it needs that were not given.
    Missing(Vec<&'static str>),
    NotRegistered,
    Registered {
        cash_basis: bool,
        purpose: Decimal,
        car: Option<Box<Car>>,
    },
}

fn invalid<T>(why: &str) -> Result<T> {
    refuse("hire_purchase.gst_fact_invalid", format!("gst: {why}."))
}

fn flag(fields: &Map<String, Value>, key: &str) -> Result<Option<bool>> {
    match fields.get(key) {
        None => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => refuse(
            "input.flag_invalid",
            format!("gst: {key} must be a boolean."),
        ),
    }
}

fn decimal(fields: &Map<String, Value>, key: &str) -> Option<Arg> {
    fields.get(key).map(
        |value| match value.as_str().and_then(|t| Decimal::parse(t).ok()) {
            Some(v) => Arg::Decimal(v),
            None => Arg::Other(value.to_string()),
        },
    )
}

/// A passenger car's facts, `None` when one is missing.
fn car(fields: &Map<String, Value>, year: &str) -> Result<Option<Car>> {
    let first_used_year = match fields.get("first_used_year") {
        None => None,
        Some(Value::String(y)) => match CAR_LIMITS.iter().find(|(known, _)| known == y) {
            Some((known, _)) if *known < year => {
                return refuse(
                    "hire_purchase.gst_fact_conflict",
                    "gst: first_used_year cannot be before the income year the agreement starts in, when the notional buyer acquires the car (ITAA 1997 s 240-20(1)).",
                );
            }
            Some(found) => Some(*found),
            None => {
                return refuse(
                    "hire_purchase.car_limit_unknown",
                    format!(
                        "gst: first_used_year must be 2024-25, 2025-26 or 2026-27, the financial years whose car limit the ATO has published; {y} is not one."
                    ),
                );
            }
        },
        Some(_) => return invalid("first_used_year must be an income year such as 2026-27"),
    };
    let market_value = decimal(fields, "gst_inclusive_market_value")
        .map(|a| money(&a))
        .transpose()?;
    let luxury_car_tax = decimal(fields, "luxury_car_tax")
        .map(|a| money(&a))
        .transpose()?;
    let quotes_abn = flag(fields, "quotes_abn_for_luxury_car_tax")?;
    let (Some((used, limit)), Some(market_value), Some(luxury_car_tax), Some(quotes_abn)) =
        (first_used_year, market_value, luxury_car_tax, quotes_abn)
    else {
        return Ok(None);
    };
    Ok(Some(Car {
        first_used_year: used,
        limit: d(limit),
        market_value,
        luxury_car_tax,
        quotes_abn,
    }))
}

/// The GST facts in `gst`: each one a figure turns on, or the list of those
/// missing. A fact given that the others make irrelevant is refused.
pub(super) fn read(gst: Option<&Arg>, year: &str) -> Result<Facts> {
    let Some(gst) = gst else {
        return Ok(Facts::Missing(vec!["gst"]));
    };
    let Arg::Json(Value::Object(fields)) = gst else {
        return invalid("must be an object of GST facts");
    };
    let registered = flag(fields, "registered")?;
    let allowed: Vec<&str> = match registered {
        Some(false) => vec!["registered"],
        _ => ["registered"]
            .iter()
            .chain(&REGISTERED)
            .chain(&CAR)
            .copied()
            .collect(),
    };
    if let Some(key) = fields.keys().find(|k| !allowed.contains(&k.as_str())) {
        let why = if registered == Some(false) {
            "is a GST fact only when registered is true"
        } else {
            "is not a GST fact this worksheet takes"
        };
        return refuse("input.field_unknown", format!("gst: {key} {why}"));
    }
    let Some(registered) = registered else {
        return Ok(Facts::Missing(vec!["registered"]));
    };
    if !registered {
        return Ok(Facts::NotRegistered);
    }
    let mut missing: Vec<&'static str> = REGISTERED
        .iter()
        .copied()
        .filter(|k| !fields.contains_key(*k))
        .collect();
    let cash_basis = match fields.get("accounting_basis").map(|v| v.as_str()) {
        None => None,
        Some(Some("cash")) => Some(true),
        Some(Some("accruals")) => Some(false),
        Some(_) => return invalid("accounting_basis must be cash or accruals"),
    };
    let purpose = match decimal(fields, "creditable_purpose") {
        None => None,
        Some(arg) => match finite_between(&arg, "0", "1")? {
            Some(p) => Some(p),
            None => {
                return refuse(
                    "input.fraction_invalid",
                    "gst: creditable_purpose must be a finite decimal fraction from 0 to 1.",
                );
            }
        },
    };
    let passenger_car = match fields.get("asset").map(|v| v.as_str()) {
        None => None,
        Some(Some("passenger_car")) => Some(true),
        Some(Some("other")) => Some(false),
        Some(_) => return invalid("asset must be passenger_car or other"),
    };
    if passenger_car == Some(false)
        && let Some(key) = CAR.iter().find(|k| fields.contains_key(**k))
    {
        return refuse(
            "input.field_unknown",
            format!("gst: {key} is a GST fact only for a passenger_car"),
        );
    }
    // A car's facts are read (and so checked) unless the asset is not a car,
    // and needed only once it is one.
    let car = match passenger_car {
        Some(false) => None,
        _ => car(fields, year)?,
    };
    if passenger_car == Some(true) {
        missing.extend(CAR.iter().filter(|k| !fields.contains_key(**k)));
    }
    match (cash_basis, purpose) {
        (Some(cash_basis), Some(purpose)) if missing.is_empty() => Ok(Facts::Registered {
            cash_basis,
            purpose,
            car: car.map(Box::new),
        }),
        _ => Ok(Facts::Missing(missing)),
    }
}

const TOTAL_CONSIDERATION: &str = "GST Act s 9-75(1), with the ATO's 'GST – Hire purchase and leasing': for an agreement entered into on or after 1 July 2012 every component of the supply is taxable, the credit component included, so the consideration is the price (the amount financed, deposit and trade-in) and the notional interest, the arrangement payments' excess over the amount financed";
const GST_PAYABLE: &str = "GST Act ss 9-70 and 9-75(1): 10% of the value of the taxable supply, which is the price x 10 / 11, so 1/11 of the price";
const GST_PAYABLE_CAR: &str = "GST Act ss 9-70 and 9-75(1) and (2): 10% of the value, the price less luxury car tax x 10 / 11, so 1/11 of the price less luxury car tax";
const NOT_REGISTERED: &str = "GST Act s 11-5(d): an entity not registered or required to be registered makes no creditable acquisition, so there is no input tax credit";
const NO_PURPOSE: &str = "GST Act s 11-5(a): nothing is acquired for a creditable purpose, so the acquisition is not creditable and there is no input tax credit";
const FULL: &str =
    "GST Act s 11-25 and s 11-30(3) 'full input tax credit': the GST payable on the supply";
const CAR_NOT_CAPPED: &str = "GST Act s 11-25: the GST payable on the supply; s 69-10(1) does not cap it, as the car's GST inclusive market value does not exceed the car limit for the financial year it is first used, or the hirer may quote an ABN for luxury car tax (s 69-10(1)(b) and (c))";
const CAR_CAPPED: &str = "GST Act s 69-10(1), with the ATO's 'Purchasing a motor vehicle' and its GST issues register on hire purchase of a car: the GST payable on the supply up to 1/11 of the car limit for the financial year the car is first used, as its GST inclusive market value exceeds that limit";
const PART: &str =
    "GST Act s 11-30(3): the full input tax credit x the extent of creditable purpose";
const PART_CAR: &str = "GST Act ss 11-30(3) and 69-10(3): the full input tax credit x the extent of creditable purpose";
const ON_INTEREST: &str = "The ATO's 'GST – Hire purchase and leasing' (Albert's second freezer): the credit covers the GST in both the price and the interest charged, and the interest's share is 1/11 of the notional interest x the extent of creditable purpose";

/// The credit's amounts and what the journals need of them.
pub(super) struct Worked {
    pub steps: Vec<Step>,
    /// The extent of creditable purpose, where a credit's share of the
    /// interest is worked out (the credit not capped).
    pub purpose_on_interest: Option<Decimal>,
    pub car_limit: Option<Decimal>,
    /// Whether s 69-10 capped the credit.
    pub capped: bool,
}

/// The input tax credit's steps for a registered hirer. `consideration` is
/// the price, `interest` the total notional interest.
pub(super) fn worked(
    facts: &Facts,
    consideration: &Decimal,
    interest: &Decimal,
) -> Result<Option<Worked>> {
    let (purpose, car) = match facts {
        Facts::Missing(_) => return Ok(None),
        Facts::NotRegistered => {
            let nil = step("input_tax_credit", NOT_REGISTERED, "0", &[], int(0));
            return Ok(Some(Worked {
                steps: vec![nil],
                purpose_on_interest: None,
                car_limit: None,
                capped: false,
            }));
        }
        Facts::Registered { purpose, car, .. } => (purpose, car),
    };
    let total = consideration.add(interest)?;
    let mut steps = vec![step(
        "total_consideration",
        TOTAL_CONSIDERATION,
        "consideration + total_interest",
        &[consideration, interest],
        total.clone(),
    )];
    let (full, capped) = match car {
        None => {
            let payable = total.div(&int(11))?;
            steps.push(step(
                "gst_payable",
                GST_PAYABLE,
                "total_consideration / 11",
                &[&total],
                payable.clone(),
            ));
            let full = step(
                "full_input_tax_credit",
                FULL,
                "gst_payable",
                &[&payable],
                payable.clone(),
            );
            (full, false)
        }
        Some(car) => {
            if car.luxury_car_tax.gt(consideration)? {
                return refuse(
                    "hire_purchase.gst_fact_conflict",
                    "gst: luxury_car_tax cannot exceed the price (amount_financed, deposit and trade_in), which includes it.",
                );
            }
            let payable = total.sub(&car.luxury_car_tax)?.div(&int(11))?;
            steps.push(step(
                "gst_payable",
                GST_PAYABLE_CAR,
                "(total_consideration - luxury_car_tax) / 11",
                &[&total, &car.luxury_car_tax],
                payable.clone(),
            ));
            let cap = car.limit.div(&int(11))?;
            let applies = !car.quotes_abn && car.market_value.gt(&car.limit)?;
            if applies && cap.lt(&payable)? {
                let full = step(
                    "full_input_tax_credit",
                    CAR_CAPPED,
                    "min(gst_payable, car_limit / 11)",
                    &[&payable, &car.limit],
                    py_min(payable.clone(), cap)?,
                );
                (full, true)
            } else {
                let full = step(
                    "full_input_tax_credit",
                    CAR_NOT_CAPPED,
                    "gst_payable",
                    &[&payable],
                    payable.clone(),
                );
                (full, false)
            }
        }
    };
    let full_value = full.value.clone();
    steps.push(full);
    if purpose.is_zero() {
        steps.push(step("input_tax_credit", NO_PURPOSE, "0", &[], int(0)));
        return Ok(Some(Worked {
            steps,
            purpose_on_interest: None,
            car_limit: car.as_ref().map(|c| c.limit.clone()),
            capped,
        }));
    }
    let part = if car.is_some() { PART_CAR } else { PART };
    steps.push(step(
        "input_tax_credit",
        part,
        "full_input_tax_credit x creditable_purpose",
        &[&full_value, purpose],
        full_value.mul(purpose)?,
    ));
    let purpose_on_interest = if capped {
        None
    } else {
        steps.push(step(
            "credit_on_interest",
            ON_INTEREST,
            "total_interest / 11 x creditable_purpose",
            &[interest, purpose],
            interest.div(&int(11))?.mul(purpose)?,
        ));
        Some(purpose.clone())
    };
    Ok(Some(Worked {
        steps,
        purpose_on_interest,
        car_limit: car.as_ref().map(|c| c.limit.clone()),
        capped,
    }))
}

/// The GST facts and attribution beside the amounts.
pub(super) fn summary(facts: &Facts, first_consideration: &str, capped: bool) -> Value {
    match facts {
        Facts::Missing(missing) => json!({
            "computed": false,
            "missing": missing,
        }),
        Facts::NotRegistered => json!({
            "computed": true,
            "registered": false,
        }),
        Facts::Registered {
            cash_basis, car, ..
        } => {
            let mut notes = vec![
                "Claimed only with a tax invoice held when the GST return is given (GST Act s 29-10(3)).",
                "Fees, charges and stamp duty under the agreement are not included.",
            ];
            if capped {
                notes.push("The credit is capped at 1/11 of the car limit; the sources read do not say how a capped credit divides between the car's price and the credit component, so the journals do not book it.");
            }
            json!({
                "computed": true,
                "registered": true,
                "accounting_basis": if *cash_basis { "cash" } else { "accruals" },
                "first_used_year": car.as_ref().map(|c| c.first_used_year),
                "attributable": if *cash_basis {
                    "The whole input tax credit is attributable to the tax period in which any consideration is first provided or, before that, an invoice is issued: an entity accounting on a cash basis is treated as not doing so for an acquisition under a hire purchase agreement (GST Act ss 158-5 and 29-10(1); the ATO's 'GST – Hire purchase and leasing', for agreements entered into on or after 1 July 2012)."
                } else {
                    "The whole input tax credit is attributable to the tax period in which any consideration is first provided or, before that, an invoice is issued (GST Act s 29-10(1); the ATO's 'GST – Hire purchase and leasing')."
                },
                "first_consideration_date": first_consideration,
                "notes": notes,
            })
        }
    }
}
