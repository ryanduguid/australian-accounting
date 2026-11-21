// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `fbt_ev_exemption_check`: whether a car benefit is exempt under FBTAA
//! s 8A, the electric cars exemption, condition by condition. The citations
//! are to the FBTAA as compiled on 1 July 2026 (C2004A03280, compilation 97),
//! the Treasury Laws Amendment (Electric Car Discount) Act 2022 (No. 86 of
//! 2022) as made, Schedule 1 item 7 and Schedule 2 item 5, and the A New Tax
//! System (Luxury Car Tax) Act 1999 as compiled on 1 April 2025, ss 5-20 and
//! 25-1, all read on 19 November 2026.

use serde_json::{Value, json};

use super::luxury_car_tax::thresholds;
use super::{Arg, Result, date_arg, fbt_year, flag, money, rate, refuse, result};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "fbt_ev_exemption_check";

/// The facts as the command passes them; see the README for each.
pub struct EvExemptionArgs<'a> {
    pub vehicle: &'a Arg,
    pub car: &'a Arg,
    pub current_employee: &'a Arg,
    pub first_held_and_used: &'a Arg,
    pub first_retail_sale: &'a Arg,
    pub luxury_car_tax_value: &'a Arg,
    pub luxury_car_tax_on_other_supply: &'a Arg,
    /// The plug-in hybrid facts of Schedule 2 item 5(2), in its order:
    /// (b), (c), and (d) with (e).
    pub hybrid: [Option<&'a Arg>; 3],
    pub year_ended: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const HYBRID_KEYS: [&str; 3] = [
    "commitment_before_1_april_2025",
    "new_commitment_from_1_april_2025",
    "exempt_benefit_before_1_april_2025",
];

const CAR: &str = "FBTAA ss 7(1), 8A(1) and 136(1), ITAA 1997 s 995-1: a car benefit needs a car, a motor vehicle (except a motor cycle or similar vehicle) designed to carry a load of less than 1 tonne and fewer than 9 passengers";
const CURRENT_EMPLOYEE: &str = "FBTAA s 8A(1)(a): the benefit is provided in the year of tax in respect of the employment of a current employee";
const ZERO_OR_LOW: &str = "FBTAA s 8A(1)(b), (2) to (4): the car is a battery electric vehicle (only an electric motor for propulsion, no fuel cell or internal combustion engine) or a hydrogen fuel cell electric vehicle (an electric motor, a fuel cell converting hydrogen, no internal combustion engine) when the benefit is provided";
const HYBRID_COMMITMENT: &str = "Electric Car Discount Act 2022 Sch 2 item 5(2)(b): before 1 April 2025 the employer, the employee or an associate of either committed to the car's application or availability, in respect of the employee's employment by the employer, for a period that began before 1 April 2025 and includes the benefit";
const HYBRID_NO_NEW_COMMITMENT: &str = "Electric Car Discount Act 2022 Sch 2 item 5(2)(c): no new commitment for a period including the benefit was made on or after 1 April 2025 and before or at the benefit (the ATO: an optional extension, a break in a novation, changed lease payments or residual value, or a change of employer is one)";
const HYBRID_EXEMPT_BEFORE: &str = "Electric Car Discount Act 2022 Sch 2 item 5(2)(d) and (e): before 1 April 2025 a car benefit relating to the car was provided, and it was exempt under s 8A, which then included a plug-in hybrid electric vehicle (s 8A(2)(c) and (5) as made)";
const HELD_AND_USED: &str = "Electric Car Discount Act 2022 Sch 1 item 7(2): s 8A applies only if the earliest time a person both held and used the car was at or after the start of 1 July 2022";
const FIRST_SALE: &str = "FBTAA s 8A(1)(c); LCT Act ss 5-20 and 25-1(1) and (4); the ATO's 'Electric cars exemption': no luxury car tax became payable on the car's first retail sale before the benefit, its luxury car tax value not exceeding the fuel-efficient car threshold for the financial year of that sale, or the sale coming after the FBT year and so after every benefit in it";
const OTHER_SUPPLY: &str = "FBTAA s 8A(1)(c): no luxury car tax became payable on any other supply or importation of the car before the benefit";

fn conflict<T>(message: &str) -> Result<T> {
    refuse("fbt_ev_exemption.fact_conflict", message)
}

/// The financial year, 1 July to 30 June, that holds `day`.
fn financial_year(day: i64) -> String {
    let (year, month, _) = dates::to_civil(day);
    let start = if month >= 7 { year } else { year - 1 };
    format!("{start}-{:02}", (start + 1) % 100)
}

/// The fuel-efficient car limit for `year` from the ATO's table that
/// `luxury-car-tax` uses; a year it does not hold is refused for a sale within
/// or before the FBT year.
fn threshold(year: &str) -> Result<&'static str> {
    match thresholds::year(year) {
        Some(year) => Ok(thresholds::of(year).0),
        None => refuse(
            "fbt_ev_exemption.threshold_year_unsupported",
            "first_retail_sale must be on or after 1 July 2012 unless it is after the FBT year: the ATO's 'Luxury car tax rate and thresholds' lists fuel-efficient thresholds from 2012-13 to 2026-27.",
        ),
    }
}

/// The plug-in hybrid facts: required for a plug-in hybrid, refused
/// otherwise.
fn hybrid_facts(args: &EvExemptionArgs, hybrid: bool) -> Result<Option<[bool; 3]>> {
    let mut facts = [false; 3];
    for ((key, given), fact) in HYBRID_KEYS.iter().zip(args.hybrid).zip(&mut facts) {
        match (hybrid, given) {
            (true, Some(value)) => *fact = flag(value, key)?,
            (true, None) => {
                return refuse(
                    "input.missing_field",
                    format!("{key} is required for a plug-in hybrid electric vehicle."),
                );
            }
            (false, Some(_)) => {
                return refuse(
                    "input.field_unknown",
                    format!("{key} is an input only for a plug-in hybrid electric vehicle"),
                );
            }
            (false, None) => {}
        }
    }
    Ok(hybrid.then_some(facts))
}

/// One condition row.
fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

/// The vehicle's row, or a plug-in hybrid's 3 Schedule 2 item 5(2) rows.
fn vehicle_rows(vehicle: &str, hybrid: Option<[bool; 3]>) -> Vec<Value> {
    let Some([before, new, exempt]) = hybrid else {
        let met = matches!(vehicle, "battery_electric" | "hydrogen_fuel_cell_electric");
        return vec![condition("zero_or_low_emissions_vehicle", ZERO_OR_LOW, met)];
    };
    vec![
        condition(
            "plug_in_hybrid_commitment_before_1_april_2025",
            HYBRID_COMMITMENT,
            before,
        ),
        condition(
            "plug_in_hybrid_no_new_commitment",
            HYBRID_NO_NEW_COMMITMENT,
            !new,
        ),
        condition(
            "plug_in_hybrid_exempt_before_1_april_2025",
            HYBRID_EXEMPT_BEFORE,
            exempt,
        ),
    ]
}

const EXEMPT: [&str; 3] = [
    "The car benefit's taxable value is nil (FBTAA s 8A(1)): give electric_car_exempt true to fbt-car-statutory or fbt-car-operating-cost, whose taxable_value_before_exemption is the notional value.",
    "That notional value still counts in the employee's individual fringe benefits amount for the reportable fringe benefits amount (s 135P(3)), which is reported when that amount is more than $2,000 (s 135P(1)); fbt-return adds it.",
    "Car expense payment, car property and car residual benefits attributable to the period, such as registration, insurance, repairs, maintenance and fuel, including electricity to charge the car, are exempt too (s 53(1)); exempt under s 53, not s 8A, they do not count in the reportable amount. A home charging station is not one of them (the ATO's 'Electric cars exemption').",
];
const NOT_EXEMPT: &str = "The car benefit is not exempt under FBTAA s 8A: give electric_car_exempt false to fbt-car-statutory or fbt-car-operating-cost, which value it under s 9 or s 10.";
const NOT_A_CAR: &str = "Without a car there is no car benefit, so neither s 8A nor the car worksheets apply: the private use of a motor cycle or similar vehicle is a residual benefit (FBTAA s 45), for fbt-residual.";

fn day(text: &str) -> i64 {
    dates::parse(text).expect("a date literal")
}

fn vehicle(arg: &Arg) -> Result<&str> {
    match arg.text() {
        Some(
            v @ ("battery_electric"
            | "hydrogen_fuel_cell_electric"
            | "plug_in_hybrid_electric"
            | "other"),
        ) => Ok(v),
        _ => refuse(
            "fbt_ev_exemption.vehicle_invalid",
            "vehicle must be battery_electric, hydrogen_fuel_cell_electric, plug_in_hybrid_electric or other.",
        ),
    }
}

/// Refuses dates that cannot all be true of a benefit in the FBT year
/// ending 31 March `ended`.
fn refuse_conflicts(ended: i64, first_used: i64, hybrid: Option<[bool; 3]>) -> Result<()> {
    if first_used > day(&format!("{ended}-03-31")) {
        return conflict(
            "first_held_and_used is after the FBT year ends: a car first held and used later cannot give a car benefit in the year.",
        );
    }
    if hybrid.is_some_and(|[_, _, exempt]| exempt) && first_used >= day("2025-04-01") {
        return conflict(
            "exempt_benefit_before_1_april_2025 is true, but first_held_and_used is on or after 1 April 2025: no car benefit relating to the car could be provided before then.",
        );
    }
    Ok(())
}

/// Adds the verdict, the conditions and their consequences to `out`.
fn add_outcome(out: &mut Value, rows: Vec<Value>, car: bool) {
    let failing: Vec<Value> = rows
        .iter()
        .filter(|row| row["met"] == json!(false))
        .map(|row| row["condition"].clone())
        .collect();
    let exempt = failing.is_empty();
    out["exempt"] = json!(exempt);
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["consequences"] = match (car, exempt) {
        (false, _) => json!([NOT_A_CAR]),
        (true, true) => json!(EXEMPT),
        (true, false) => json!([NOT_EXEMPT]),
    };
    if car {
        out["car_worksheet_input"] = json!({"electric_car_exempt": exempt});
    }
}

/// Whether luxury car tax became payable on the first retail sale, on `sale`,
/// before the benefits, with the rates that decide it. s 8A(1)(c) counts luxury
/// car tax payable before the benefit: a sale after the FBT year comes after
/// every benefit in it, so no threshold applies to it, and a taxed sale within
/// the year after the car was first held and used comes between the benefits
/// and is refused.
fn first_sale_taxed(
    ended: i64,
    first_used: i64,
    sale: i64,
    value: &Decimal,
) -> Result<(bool, Vec<(String, String)>)> {
    let sale_year = financial_year(sale);
    let mut rates = vec![rate("first_retail_sale_year", &sale_year)];
    if sale > day(&format!("{ended}-03-31")) {
        return Ok((false, rates));
    }
    let limit = threshold(&sale_year)?;
    rates.push(rate("fuel_efficient_luxury_car_tax_threshold", limit));
    let taxed = value.gt(&super::d(limit))?;
    if taxed && sale > first_used && sale >= day(&format!("{}-04-01", ended - 1)) {
        return refuse(
            "fbt_ev_exemption.sale_within_year",
            "first_retail_sale is within the FBT year, after the car was first held and used, and luxury car tax was payable on it: it fails FBTAA s 8A(1)(c) only for the benefits after it, so the year has no single result.",
        );
    }
    Ok((taxed, rates))
}

pub fn fbt_ev_exemption_check(args: &EvExemptionArgs) -> Result<Value> {
    let (year, period, _) = fbt_year(args.scope_confirmed, args.year_ended, KIND)?;
    let ended: i64 = year.parse().expect("FBT year");
    let vehicle = vehicle(args.vehicle)?;
    let hybrid = hybrid_facts(args, vehicle == "plug_in_hybrid_electric")?;
    let car = flag(args.car, "car")?;
    let current = flag(args.current_employee, "current_employee")?;
    let first_used = date_arg(args.first_held_and_used, "first_held_and_used")?;
    let first_sale = date_arg(args.first_retail_sale, "first_retail_sale")?;
    let value = money(args.luxury_car_tax_value)?;
    let other_supply = flag(
        args.luxury_car_tax_on_other_supply,
        "luxury_car_tax_on_other_supply",
    )?;
    refuse_conflicts(ended, first_used, hybrid)?;
    let (taxed, rates) = first_sale_taxed(ended, first_used, first_sale, &value)?;
    let mut rows = vec![
        condition("car", CAR, car),
        condition("current_employee", CURRENT_EMPLOYEE, current),
    ];
    rows.extend(vehicle_rows(vehicle, hybrid));
    rows.push(condition(
        "first_held_and_used_from_1_july_2022",
        HELD_AND_USED,
        first_used >= day("2022-07-01"),
    ));
    rows.push(condition(
        "no_luxury_car_tax_at_first_retail_sale",
        FIRST_SALE,
        !taxed,
    ));
    rows.push(condition(
        "no_luxury_car_tax_on_other_supply",
        OTHER_SUPPLY,
        !other_supply,
    ));
    let mut out = result(KIND, &period, vec![("luxury_car_tax_value", value)], rates)?;
    add_outcome(&mut out, rows, car);
    Ok(out)
}
