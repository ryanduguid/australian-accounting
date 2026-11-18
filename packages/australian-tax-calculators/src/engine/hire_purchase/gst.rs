// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The hirer's input tax credit for an agreement entered into from 1 July
//! 2012, when every fact it turns on is given. The citations are to the GST
//! Act compilation 96 (1 January 2026) and the ATO's 'GST – Hire purchase and
//! leasing' (last updated 6 April 2017), 'Purchasing a motor vehicle' (last
//! updated 3 June 2026) and the GST issues register 'Financial services -
//! questions and answers', issues 1.1, 1.3 and 1.17 (updated 7 September
//! 2012), all read on 18 November 2026.

use serde_json::{Map, Value, json};

use crate::decimal::{Decimal, Rounding};
use crate::engine::trade_in::CAR_LIMITS;
use crate::engine::{Arg, Result, Step, d, finite_between, int, money, refuse, step};

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
    /// No figure: s 69-10(1) caps a car's credit, and the sources read do not
    /// settle what the cap takes in.
    Review,
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
/// The first use year is the hirer's, which may come before the agreement,
/// under an earlier hire of the same car (s 69-10(1)(c)).
fn car(fields: &Map<String, Value>) -> Result<Option<Car>> {
    let first_used_year = match fields.get("first_used_year") {
        None => None,
        Some(Value::String(y)) => match CAR_LIMITS.iter().find(|(known, _)| known == y) {
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
pub(super) fn read(gst: Option<&Arg>) -> Result<Facts> {
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
    match registered {
        None => Ok(Facts::Missing(vec!["registered"])),
        Some(false) => Ok(Facts::NotRegistered),
        Some(true) => read_registered(fields),
    }
}

/// `key`'s value: true for `yes`, false for `no`, `None` when not given.
fn either(fields: &Map<String, Value>, key: &str, yes: &str, no: &str) -> Result<Option<bool>> {
    match fields.get(key).map(Value::as_str) {
        None => Ok(None),
        Some(Some(v)) if v == yes => Ok(Some(true)),
        Some(Some(v)) if v == no => Ok(Some(false)),
        Some(_) => invalid(&format!("{key} must be {yes} or {no}")),
    }
}

/// The extent of creditable purpose, `None` when not given.
fn creditable_purpose(fields: &Map<String, Value>) -> Result<Option<Decimal>> {
    let Some(arg) = decimal(fields, "creditable_purpose") else {
        return Ok(None);
    };
    match finite_between(&arg, "0", "1")? {
        Some(p) => Ok(Some(p)),
        None => refuse(
            "input.fraction_invalid",
            "gst: creditable_purpose must be a finite decimal fraction from 0 to 1.",
        ),
    }
}

/// A registered hirer's facts, as [`read`] gives them.
fn read_registered(fields: &Map<String, Value>) -> Result<Facts> {
    let mut missing: Vec<&'static str> = REGISTERED
        .iter()
        .copied()
        .filter(|k| !fields.contains_key(*k))
        .collect();
    let cash_basis = either(fields, "accounting_basis", "cash", "accruals")?;
    let purpose = creditable_purpose(fields)?;
    let passenger_car = either(fields, "asset", "passenger_car", "other")?;
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
        _ => car(fields)?,
    };
    if passenger_car == Some(true) {
        missing.extend(CAR.iter().filter(|k| !fields.contains_key(**k)));
    }
    let (Some(cash_basis), Some(purpose)) = (cash_basis, purpose) else {
        return Ok(Facts::Missing(missing));
    };
    if !missing.is_empty() {
        return Ok(Facts::Missing(missing));
    }
    if let Some(car) = &car
        && !car.quotes_abn
        && car.market_value.gt(&car.limit)?
    {
        return Ok(Facts::Review);
    }
    Ok(Facts::Registered {
        cash_basis,
        purpose,
        car: car.map(Box::new),
    })
}

/// Why a car the s 69-10(1) cap reaches gets no figure.
const CAR_REVIEW: &str = "No GST figure, for review: the car's GST inclusive market value exceeds the car limit for the financial year it is first used and the hirer may not quote an ABN for luxury car tax, so GST Act s 69-10(1) caps the input tax credit on the car at 1/11 of that limit, which the ATO's GST issues register 'Financial services - questions and answers' (issue 1.17) applies under a hire purchase agreement. The sources read do not settle whether the cap also takes in the GST on the credit component, which the same register (issue 1.1) treats as a separate supply of finance, taxable for an agreement entered into from 1 July 2012.";

const TOTAL_CONSIDERATION: &str = "GST Act s 9-75(1), with the ATO's 'GST – Hire purchase and leasing': for an agreement entered into on or after 1 July 2012 every component of the supply is taxable, the credit component included, so the consideration is the price (the amount financed, deposit and trade-in) and the notional interest, the arrangement payments' excess over the amount financed";
const GST_PAYABLE: &str = "GST Act ss 9-70 and 9-75(1): 10% of the value of the taxable supply, which is the price x 10 / 11, so 1/11 of the price, rounded to the nearest cent, 0.5 up (s 9-90(1), or (2)(a) where the credit component is a separate supply on the same invoice)";
const GST_PAYABLE_CAR: &str = "GST Act ss 9-70 and 9-75(1) and (2): 10% of the value, the price less luxury car tax x 10 / 11, so 1/11 of the price less luxury car tax, rounded to the nearest cent, 0.5 up (s 9-90(1), or (2)(a) where the credit component is a separate supply on the same invoice)";
const NOT_REGISTERED: &str = "GST Act s 11-5(d): an entity not registered or required to be registered makes no creditable acquisition, so there is no input tax credit";
const NO_PURPOSE: &str = "GST Act s 11-5(a): nothing is acquired for a creditable purpose, so the acquisition is not creditable and there is no input tax credit";
const FULL: &str =
    "GST Act s 11-25 and s 11-30(3) 'full input tax credit': the GST payable on the supply";
const CAR_NOT_CAPPED: &str = "GST Act s 11-25: the GST payable on the supply; s 69-10(1) does not cap it, as the car's GST inclusive market value does not exceed the car limit for the financial year it is first used, or the hirer may quote an ABN for luxury car tax (s 69-10(1)(b) and (c))";
const PART: &str =
    "GST Act s 11-30(3): the full input tax credit x the extent of creditable purpose";
const ON_INTEREST: &str = "The ATO's 'GST – Hire purchase and leasing' (Albert's second freezer): the credit covers the GST in both the price and the interest charged. This is the journals' allocation of the credit to the interest, 1/11 of the notional interest x the extent of creditable purpose, not a credit of its own: the credit is input_tax_credit";

/// The credit's amounts and what the journals need of them.
pub(super) struct Worked {
    pub steps: Vec<Step>,
    /// The extent of creditable purpose, where a credit's share of the
    /// interest is worked out.
    pub purpose_on_interest: Option<Decimal>,
    pub car_limit: Option<Decimal>,
}

/// The input tax credit's steps for a registered hirer. `consideration` is
/// the price, `interest` the total notional interest.
pub(super) fn worked(
    facts: &Facts,
    consideration: &Decimal,
    interest: &Decimal,
) -> Result<Option<Worked>> {
    let (purpose, car) = match facts {
        Facts::Missing(_) | Facts::Review => return Ok(None),
        Facts::NotRegistered => return Ok(Some(no_credit(Vec::new(), NOT_REGISTERED, None))),
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
    let [payable, full] = full_credit(&total, car.as_deref(), consideration)?;
    let full_value = full.value.clone();
    steps.extend([payable, full]);
    let car_limit = car.as_ref().map(|c| c.limit.clone());
    if purpose.is_zero() {
        return Ok(Some(no_credit(steps, NO_PURPOSE, car_limit)));
    }
    steps.push(step(
        "input_tax_credit",
        PART,
        "full_input_tax_credit x creditable_purpose",
        &[&full_value, purpose],
        full_value.mul(purpose)?,
    ));
    steps.push(step(
        "credit_on_interest",
        ON_INTEREST,
        "total_interest / 11 x creditable_purpose",
        &[interest, purpose],
        interest.div(&int(11))?.mul(purpose)?,
    ));
    Ok(Some(Worked {
        steps,
        purpose_on_interest: Some(purpose.clone()),
        car_limit,
    }))
}

/// `steps` closed by a nil input tax credit, for the reason `authority` gives.
fn no_credit(mut steps: Vec<Step>, authority: &'static str, car_limit: Option<Decimal>) -> Worked {
    steps.push(step("input_tax_credit", authority, "0", &[], int(0)));
    Worked {
        steps,
        purpose_on_interest: None,
        car_limit,
    }
}

/// The GST payable on the supply, rounded to the cent (GST Act s 9-90), and
/// the full input tax credit, which is that GST (s 11-25): for a car, less
/// luxury car tax and not capped, as [`read`] gives no figure where s 69-10(1)
/// would cap it.
fn full_credit(total: &Decimal, car: Option<&Car>, consideration: &Decimal) -> Result<[Step; 2]> {
    let Some(car) = car else {
        let payable = total.div(&int(11))?.quantize(-2, Rounding::HalfUp)?;
        return Ok([
            step(
                "gst_payable",
                GST_PAYABLE,
                "cents(total_consideration / 11)",
                &[total],
                payable.clone(),
            ),
            step(
                "full_input_tax_credit",
                FULL,
                "gst_payable",
                &[&payable],
                payable.clone(),
            ),
        ]);
    };
    if car.luxury_car_tax.gt(consideration)? {
        return refuse(
            "hire_purchase.gst_fact_conflict",
            "gst: luxury_car_tax cannot exceed the price (amount_financed, deposit and trade_in), which includes it.",
        );
    }
    let payable = total
        .sub(&car.luxury_car_tax)?
        .div(&int(11))?
        .quantize(-2, Rounding::HalfUp)?;
    Ok([
        step(
            "gst_payable",
            GST_PAYABLE_CAR,
            "cents((total_consideration - luxury_car_tax) / 11)",
            &[total, &car.luxury_car_tax],
            payable.clone(),
        ),
        step(
            "full_input_tax_credit",
            CAR_NOT_CAPPED,
            "gst_payable",
            &[&payable],
            payable.clone(),
        ),
    ])
}

/// The GST facts and attribution beside the amounts.
pub(super) fn summary(facts: &Facts, first_consideration: &str) -> Value {
    match facts {
        Facts::Missing(missing) => json!({
            "computed": false,
            "missing": missing,
        }),
        Facts::NotRegistered => json!({
            "computed": true,
            "registered": false,
        }),
        Facts::Review => json!({
            "computed": false,
            "review": CAR_REVIEW,
        }),
        Facts::Registered {
            cash_basis, car, ..
        } => {
            let notes = [
                "Claimed only with a tax invoice held when the GST return is given (GST Act s 29-10(3)).",
                "Fees, charges and stamp duty under the agreement are not included.",
            ];
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
