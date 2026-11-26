// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `novated_lease`: one year of a car under a novated lease against paying the
//! same car costs from after-tax pay, as the site's novated lease page works
//! it (`assets/novated-lease.mjs`). FBTAA s 8A decides the treatment: an
//! exempt car's costs all come from pre-tax salary and its statutory formula
//! value (s 9) is still reportable (s 135P(3)); any other car is taken under
//! the employee contribution method, the employee paying the taxable value
//! from after-tax pay, which reduces it to nil (s 9(1)), and the rest from
//! pre-tax salary. Read on 26 November 2026 on the ATO's legal database:
//! FBTAA ss 8A, 9 and 135P; the ATO's 'Fringe benefits tax - rates and
//! thresholds' (last updated 20 May 2026), 'Luxury car tax rate and
//! thresholds' (QC38161, last updated 1 June 2026) and 'FBT on plug-in hybrid
//! electric vehicles' (the exemption kept for a plug-in hybrid used before
//! 1 April 2025 under a financially binding commitment that continues).

use super::luxury_car_tax::thresholds::TABLE as LCT_THRESHOLDS;
use super::salary_packaging::{Employee, RESIDENCY, foreign, reportable, tax_on_pay};
use super::{
    Arg, Result, d, flag, income_year_days, int, money, precise, rate, refuse, result, scope,
    whole_dollars,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use serde_json::{Map, Value, json};

const KIND: &str = "novated_lease";

/// Section 9(2)(b): the statutory percentage.
const STATUTORY_RATE: &str = "0.20";
/// The day s 8A began to apply (1 July 2022) and the day a plug-in hybrid
/// stopped being a zero or low emissions vehicle (1 April 2025).
const EXEMPTION_FROM: &str = "2022-07-01";
const HYBRID_TO: &str = "2025-04-01";
/// April to June: the part of an FBT year before the income year's 1 July.
const APRIL_TO_JUNE: i64 = 91;

/// The facts as the command passes them.
pub struct NovatedLeaseArgs<'a> {
    pub residency: &'a Arg,
    pub salary: &'a Arg,
    pub study_loan: &'a Arg,
    pub hospital_cover: &'a Arg,
    pub car: &'a Arg,
    pub first_used: &'a Arg,
    pub commitment: &'a Arg,
    pub first_sale_year: &'a Arg,
    pub cost_price: &'a Arg,
    pub other_lct_supply: &'a Arg,
    pub lct_timing: &'a Arg,
    pub one_third_reduction: &'a Arg,
    pub days_available: &'a Arg,
    pub costs_with_gst: &'a Arg,
    pub costs_without_gst: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The car's facts for s 8A.
struct Car {
    kind: &'static str,
    first_used: String,
    commitment: bool,
    sale_year: &'static str,
    threshold: Decimal,
    other_supply: bool,
    timing: &'static str,
    days: i64,
}

fn text_choice(value: &Arg, choices: &[&'static str]) -> Option<&'static str> {
    value
        .text()
        .and_then(|t| choices.iter().find(|c| **c == t).copied())
}

fn start_year(year: &str) -> i64 {
    year[..4].parse().expect("a year")
}

/// Bounds on a taxable first retail sale from the year and the private-use
/// days: before this FBT year, within it, or after all its benefits.
fn first_sale_timing(car: &Car, fbt_start: i64, year_days: i64) -> Result<Option<&'static str>> {
    let sale_start = start_year(car.sale_year);
    if sale_start < fbt_start - 1 {
        if car.timing == "after_all" {
            return refuse(
                "novated_lease.lct_timing_invalid",
                "The taxable first retail sale was before this FBT year, so luxury car tax was already payable.",
            );
        }
        return Ok(Some("before_all"));
    }
    let late =
        sale_start > fbt_start || (sale_start == fbt_start && car.days > year_days - APRIL_TO_JUNE);
    if !car.other_supply && car.timing == "before_all" && late {
        return refuse(
            "novated_lease.lct_timing_invalid",
            "The taxable first retail sale cannot be before all the benefits counted.",
        );
    }
    if car.timing == "after_all" && sale_start == fbt_start - 1 && car.days > APRIL_TO_JUNE {
        return refuse(
            "novated_lease.lct_timing_invalid",
            "The taxable first retail sale cannot be after all the benefits counted.",
        );
    }
    Ok((sale_start > fbt_start).then_some("after_all"))
}

/// Why s 8A does not exempt the car's benefits, or `None` when it does. Luxury
/// car tax first payable before a benefit fails s 8A(1)(c); the cost price over
/// the fuel-efficient threshold of the year of first retail sale indicates tax
/// on that sale, and `other_lct_supply` records another supply or importation.
fn not_exempt(
    car: &Car,
    price: &Decimal,
    year: &str,
    year_days: i64,
) -> Result<Option<&'static str>> {
    let first_sale_taxed = price.gt(&car.threshold)?;
    let timed = matches!(car.timing, "before_all" | "after_all");
    if !first_sale_taxed && !car.other_supply && timed {
        return refuse(
            "novated_lease.lct_timing_invalid",
            "lct_timing needs a taxable first retail sale or luxury car tax on another supply or importation.",
        );
    }
    if let Some(reason) = not_eligible(car, year) {
        return Ok(Some(reason));
    }
    if !first_sale_taxed && !car.other_supply {
        return Ok(None);
    }
    lct_reason(car, first_sale_taxed, year, year_days)
}

/// Why the car is outside s 8A whatever luxury car tax says.
fn not_eligible(car: &Car, year: &str) -> Option<&'static str> {
    if car.kind == "other" {
        return Some("it is not a battery electric or hydrogen fuel cell car (s 8A(2))");
    }
    if car.kind == "phev" && year >= "2025-26" && !car.commitment {
        return Some(
            "a plug-in hybrid is exempt from 1 April 2025 only under a commitment made and used before then",
        );
    }
    (car.first_used.as_str() < EXEMPTION_FROM)
        .then_some("it was first held and used before 1 July 2022")
}

/// Section 8A(1)(c): luxury car tax first payable before the benefits.
fn lct_reason(
    car: &Car,
    first_sale_taxed: bool,
    year: &str,
    year_days: i64,
) -> Result<Option<&'static str>> {
    let first_sale = "luxury car tax was payable on its first retail sale (s 8A(1)(c))";
    let timing = if first_sale_taxed {
        first_sale_timing(car, start_year(year), year_days)?
    } else {
        None
    };
    match (timing, car.timing) {
        (Some("before_all"), _) => Ok(Some(first_sale)),
        (Some("after_all"), _) if !car.other_supply => Ok(None),
        (_, "after_all") => Ok(None),
        (_, "before_all") if first_sale_taxed && car.other_supply => Ok(Some(
            "luxury car tax became payable before the benefits (s 8A(1)(c))",
        )),
        (_, "before_all") if first_sale_taxed => Ok(Some(first_sale)),
        (_, "before_all") => Ok(Some(
            "luxury car tax became payable on another supply or an importation of it (s 8A(1)(c))",
        )),
        _ => refuse(
            "novated_lease.lct_timing_invalid",
            "Give lct_timing before_all or after_all: whether luxury car tax first became payable before or after all the benefits counted. The command needs one treatment for the year.",
        ),
    }
}

fn car(args: &NovatedLeaseArgs, ended: i64) -> Result<Car> {
    let Some(kind) = text_choice(args.car, &["bev", "fcev", "phev", "other"]) else {
        return refuse(
            "novated_lease.car_invalid",
            "car must be bev, fcev, phev or other.",
        );
    };
    let first_used = first_used(args.first_used, ended)?;
    let (sale_year, threshold) = sale(args.first_sale_year)?;
    let timing = timing(args.lct_timing)?;
    Ok(Car {
        kind,
        first_used,
        commitment: flag(args.commitment, "commitment")?,
        sale_year,
        threshold: d(threshold),
        other_supply: flag(args.other_lct_supply, "other_lct_supply")?,
        timing,
        days: 0,
    })
}

/// The date of first use, no later than 31 March of the FBT year.
fn first_used(value: &Arg, ended: i64) -> Result<String> {
    let Some(day) = value.text().and_then(dates::parse) else {
        return refuse(
            "input.date_invalid",
            "first_used must be a date, YYYY-MM-DD.",
        );
    };
    let first_used = dates::format(day);
    if first_used > format!("{ended}-03-31") {
        return refuse(
            "novated_lease.first_used_late",
            format!(
                "The car was first held and used after 31 March {ended}, so it gave no car benefit in the FBT year."
            ),
        );
    }
    Ok(first_used)
}

/// The year of first retail sale and its fuel-efficient threshold.
fn sale(value: &Arg) -> Result<(&'static str, &'static str)> {
    let given = value.text().unwrap_or_default();
    let Some((sale_year, threshold, _)) = LCT_THRESHOLDS
        .iter()
        .find(|(y, _, _)| *y == given && *y >= "2021-22")
    else {
        return refuse(
            "novated_lease.sale_year_invalid",
            "first_sale_year must be an income year from 2021-22 to 2026-27.",
        );
    };
    Ok((sale_year, threshold))
}

fn timing(value: &Arg) -> Result<&'static str> {
    match value {
        Arg::Other(none) if none == "None" => Ok("mixed_or_unknown"),
        given => text_choice(given, &["before_all", "after_all", "mixed_or_unknown"]).map_or_else(
            || {
                refuse(
                    "novated_lease.lct_timing_invalid",
                    "lct_timing must be before_all, after_all, mixed_or_unknown or null.",
                )
            },
            Ok,
        ),
    }
}

/// A plug-in hybrid kept under the earlier exemption was used, and so first
/// sold, before 1 April 2025.
fn hybrid_commitment(car: &Car) -> Result<()> {
    if car.kind != "phev" || !car.commitment {
        return Ok(());
    }
    if car.first_used.as_str() >= HYBRID_TO {
        return refuse(
            "novated_lease.hybrid_commitment",
            "The earlier plug-in hybrid exemption needs use before 1 April 2025.",
        );
    }
    if car.sale_year > "2024-25" {
        return refuse(
            "novated_lease.hybrid_commitment",
            "A plug-in hybrid in use before 1 April 2025 was first sold by 2024-25: give that first_sale_year.",
        );
    }
    Ok(())
}

/// The private-use days, no more than the year's days or the days from first
/// use to 31 March.
fn private_days(value: &Arg, first_used: &str, ended: i64, year_days: i64) -> Result<i64> {
    let from_first_use = 1 + dates::parse(&format!("{ended}-03-31")).expect("a date")
        - dates::parse(first_used).expect("a date");
    let limit = year_days.min(from_first_use);
    match value.int() {
        Some(Some(n)) if (0..=limit).contains(&n) => Ok(n),
        _ => refuse(
            "input.day_count_invalid",
            format!("days_available must be an integer from 0 to {limit}."),
        ),
    }
}

/// One way of paying: the pre-tax and after-tax amounts, the reportable
/// amount, the tax and the saving against paying every cost after tax.
struct Scenario {
    pre_tax: Decimal,
    post_tax: Decimal,
    reportable: Decimal,
    owed: Decimal,
    saving: Decimal,
}

pub fn novated_lease(args: &NovatedLeaseArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let employee = Employee {
        year: year.clone(),
        foreign: foreign(args.residency)
            .map_or_else(|| refuse("novated_lease.residency_invalid", RESIDENCY), Ok)?,
        study_loan: flag(args.study_loan, "study_loan")?,
        hospital_cover: flag(args.hospital_cover, "hospital_cover")?,
    };
    let ended = start_year(&year) + 1;
    let mut car = car(args, ended)?;
    hybrid_commitment(&car)?;
    let reduced = flag(args.one_third_reduction, "one_third_reduction")?;
    let salary = money(args.salary)?;
    whole_dollars(&salary, "salary")?;
    let year_days = income_year_days(&year);
    car.days = private_days(args.days_available, &car.first_used, ended, year_days)?;
    let [price, with_gst, without_gst] =
        [args.cost_price, args.costs_with_gst, args.costs_without_gst].map(money);
    let (price, with_gst, without_gst) = (price?, with_gst?, without_gst?);
    if price.is_zero() {
        return refuse(
            "novated_lease.cost_price_nil",
            "cost_price must be more than nil.",
        );
    }
    precise(|| {
        work(
            &employee,
            &car,
            [&salary, &price, &with_gst, &without_gst],
            reduced,
            year_days,
        )
    })
}

fn cents(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(-2, Rounding::HalfUp)?)
}

fn work(
    e: &Employee,
    car: &Car,
    money: [&Decimal; 4],
    reduced: bool,
    year_days: i64,
) -> Result<Value> {
    let [salary, price, with_gst, without_gst] = money;
    let costs = with_gst.add(without_gst)?;
    if costs.gt(salary)? {
        return refuse(
            "novated_lease.costs_exceed_salary",
            "The car's costs are more than the salary.",
        );
    }
    // The employer claims the GST credit, one eleventh of a cost with GST.
    let net_costs = cents(&with_gst.mul(&int(10))?.div(&int(11))?)?.add(without_gst)?;
    let value = statutory_value(price, reduced, car.days, year_days)?;
    let reason = not_exempt(car, price, &e.year, year_days)?;
    let pay = Pay {
        e,
        salary,
        costs: &costs,
        none: tax_on_pay(e, salary, &int(0))?.1.total()?,
    };
    let exempt = match reason {
        None => Some(pay.scenario(net_costs.clone(), int(0), reportable(&value)?)?),
        Some(_) => None,
    };
    let contribution = contribution(&pay, &value, &net_costs, reason.is_some())?;
    let none = pay.none.clone();
    let amounts = vec![
        ("taxable_value", value),
        ("costs", costs),
        ("net_costs", net_costs),
        ("owed_without", none),
    ];
    let rates = vec![rate("statutory_rate", STATUTORY_RATE)];
    let mut out = result(KIND, &e.year, amounts, rates)?;
    out["exempt"] = json!(reason.is_none());
    out["reason"] = json!(reason);
    out["fuel_efficient_threshold"] = json!(car.threshold.to_string());
    out["exempt_case"] = exempt.map_or(Value::Null, |s| shown(&s));
    out["contribution_case"] = contribution.map_or(Value::Null, |s| shown(&s));
    Ok(out)
}

/// The employee's pay without the car, against which each way is costed.
struct Pay<'a> {
    e: &'a Employee,
    salary: &'a Decimal,
    costs: &'a Decimal,
    none: Decimal,
}

impl Pay<'_> {
    fn scenario(&self, pre: Decimal, post: Decimal, fringe: Decimal) -> Result<Scenario> {
        let owed = tax_on_pay(self.e, &self.salary.sub(&pre)?, &fringe)?
            .1
            .total()?;
        let saving = self
            .none
            .sub(&owed)?
            .sub(&pre)?
            .sub(&post)?
            .add(self.costs)?;
        Ok(Scenario {
            pre_tax: pre,
            post_tax: post,
            reportable: fringe,
            owed,
            saving,
        })
    }
}

/// The employee contribution method: the after-tax contribution is
/// consideration on which the employer pays GST, so ten elevenths of it goes
/// to the costs; `None` when that covers them, refused for a car that has no
/// exemption to fall back on.
fn contribution(
    pay: &Pay,
    value: &Decimal,
    net_costs: &Decimal,
    taxable: bool,
) -> Result<Option<Scenario>> {
    let covered = cents(&value.mul(&int(10))?.div(&int(11))?)?;
    if !covered.gt(net_costs)? {
        return Ok(Some(pay.scenario(
            net_costs.sub(&covered)?,
            value.clone(),
            int(0),
        )?));
    }
    if taxable {
        return refuse(
            "novated_lease.contribution_short",
            "The car's yearly costs are less than its taxable value, so an after-tax contribution cannot cover it; the FBT that would remain is not worked out.",
        );
    }
    Ok(None)
}

/// Section 9(1) and (2): base value (two-thirds from the s 9(2)(c) date) x
/// 20% x days available for private use / days in the year.
fn statutory_value(price: &Decimal, reduced: bool, days: i64, year_days: i64) -> Result<Decimal> {
    let base = if reduced {
        price.mul(&int(2))?.div(&int(3))?
    } else {
        price.clone()
    };
    let value = base.mul(&d(STATUTORY_RATE))?.mul(&int(days))?;
    cents(&value.div(&int(year_days))?)
}

fn shown(s: &Scenario) -> Value {
    let pairs = [
        ("pre_tax", &s.pre_tax),
        ("post_tax", &s.post_tax),
        ("reportable_fringe_benefits", &s.reportable),
        ("owed", &s.owed),
        ("saving", &s.saving),
    ];
    let map: Map<String, Value> = pairs
        .iter()
        .map(|(k, v)| {
            (
                (*k).to_string(),
                json!(
                    v.quantize(-2, Rounding::HalfUp)
                        .expect("finite")
                        .to_string()
                ),
            )
        })
        .collect();
    Value::Object(map)
}
