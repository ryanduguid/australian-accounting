// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `motor_vehicle_duty`: the duty on registering, or transferring the
//! registration of, one passenger vehicle in any state or territory, on a
//! dutiable value the operator states, for a duty date in 2026-27. Each scale
//! is the Act's or determination's (New South Wales: Revenue NSW and Service
//! NSW, the legislation site being unreadable), with the concessional rates
//! each sets for passenger vehicles: Victoria's green and primary producer
//! cars, Queensland's engine rows, Tasmania's fleet discount, the ACT's
//! emissions categories and the Northern Territory's electric vehicle
//! exemption. Every figure was read on 24 November 2026;
//! docs/calculation-evidence.md#motor-vehicle-duty lists the versions read.

use std::borrow::Cow;

use serde_json::Value;

use super::business_capital::year_first;
use super::trade_in::when;
use super::{
    Arg, Result, Step, d, date_arg, derived_result, flag, int, money, precise, rate, refuse, scope,
    step, whole_dollars,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

mod scales;
use scales::{
    ACT_DI2025_152, ACT_DI2026_154, Band, NSW, NT, NT_ELECTRIC, QLD, SA, Scale, TAS, TAS_FLEET,
    VIC, VIC_CONCESSION, WA, band,
};

const KIND: &str = "motor_vehicle_duty";
const PLACES: [&str; 8] = ["nsw", "vic", "qld", "wa", "sa", "tas", "act", "nt"];
/// The ACT categories in the determinations' row order; a non-rated vehicle
/// pays Category C's rates.
const CATEGORIES: [(&str, usize); 7] = [
    ("aaa", 0),
    ("aa", 1),
    ("a", 2),
    ("b", 3),
    ("c", 4),
    ("d", 5),
    ("non_rated", 4),
];

/// The facts as the command passes them; see the README for each.
pub struct MotorVehicleDutyArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub vehicle_class: &'a Arg,
    pub dutiable_value: &'a Arg,
    pub duty_date: &'a Arg,
    pub engine: Option<&'a Arg>,
    pub green_car: Option<&'a Arg>,
    pub primary_producer: Option<&'a Arg>,
    pub emissions_category: Option<&'a Arg>,
    pub new_vehicle: Option<&'a Arg>,
    pub fleet_discount: Option<&'a Arg>,
    pub electric: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// How the duty is worked: on a banded scale, or Western Australia's sliding
/// rate.
enum Rule {
    Banded(Scale),
    WesternAustralia,
}

type Rates = Vec<(String, String)>;

fn place(jurisdiction: &Arg) -> Result<&'static str> {
    match PLACES.iter().find(|p| Some(**p) == jurisdiction.text()) {
        Some(place) => Ok(place),
        None => refuse(
            "motor_vehicle_duty.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
        ),
    }
}

fn passenger(vehicle_class: &Arg) -> Result<()> {
    if vehicle_class.text() == Some("passenger") {
        return Ok(());
    }
    refuse(
        "motor_vehicle_duty.vehicle_class_unsupported",
        "vehicle_class must be passenger: motorcycles, goods and commercial vehicles, buses, heavy vehicles, trailers and special or conditionally registered vehicles are not worked.",
    )
}

/// The duty date, inside the financial year `year`.
fn duty_date(value: &Arg, year: &str) -> Result<i64> {
    let first = year_first(year).expect("a supported period is a financial year");
    let start = dates::parse(&format!("{first}-07-01")).expect("a date");
    let end = dates::parse(&format!("{}-06-30", first + 1)).expect("a date");
    let day = date_arg(value, "duty_date")?;
    if !(start..=end).contains(&day) {
        return refuse(
            "motor_vehicle_duty.date_outside_year",
            format!(
                "duty_date must be in the {year} financial year, from {} to {}.",
                dates::format(start),
                dates::format(end)
            ),
        );
    }
    Ok(day)
}

/// A boolean asked in one jurisdiction only, false elsewhere.
fn only_flag(on: bool, given: Option<&Arg>, key: &str, why: &str) -> Result<bool> {
    when(on, given, key, why)?.map_or(Ok(false), |value| flag(value, key))
}

fn queensland(engine: Option<&Arg>) -> Result<Scale> {
    let why = "in Queensland (Duties Act 2001 (Qld) Sch 4C)";
    let given = when(true, engine, "engine", why)?.and_then(Arg::text);
    match QLD.into_iter().find(|(code, _)| Some(*code) == given) {
        Some((_, scale)) => Ok(scale),
        None => refuse(
            "motor_vehicle_duty.engine_invalid",
            "engine must be hybrid_or_electric, up_to_4_cylinders (or 2 rotors or steam), 5_or_6_cylinders (or 3 rotors) or 7_or_more_cylinders.",
        ),
    }
}

fn victoria(args: &MotorVehicleDutyArgs) -> Result<Scale> {
    let why = "in Victoria (Duties Act 2000 (Vic) s 218(1) items 1 and 2)";
    let green = only_flag(true, args.green_car, "green_car", why)?;
    let primary = only_flag(true, args.primary_producer, "primary_producer", why)?;
    Ok(if green || primary {
        VIC_CONCESSION
    } else {
        VIC
    })
}

/// The ACT scale for the determination in force on `day`, the category and
/// whether the vehicle is new.
fn act(args: &MotorVehicleDutyArgs, day: i64) -> Result<Scale> {
    let why = "in the ACT (the motor vehicle duty determination's categories)";
    let given = when(true, args.emissions_category, "emissions_category", why)?.and_then(Arg::text);
    let new = only_flag(true, args.new_vehicle, "new_vehicle", why)?;
    let Some(&(_, mut index)) = CATEGORIES.iter().find(|(code, _)| Some(*code) == given) else {
        return refuse(
            "motor_vehicle_duty.emissions_category_invalid",
            "emissions_category must be aaa, aa, a, b, c, d or non_rated.",
        );
    };
    if !new && index != 0 {
        index = 4;
    }
    let from_february = dates::parse("2027-02-01").expect("a date");
    let period = if day < from_february {
        &ACT_DI2025_152
    } else {
        &ACT_DI2026_154
    };
    let row = &period.rows[index];
    let bands = vec![
        band(0, "0", row.table_1, 0),
        band(45_000, row.table_2_base, row.table_2_rate, 45_000),
        band(80_000, row.table_3_base, "8", 80_000),
    ];
    Ok(Scale {
        authority: period.authority,
        per: 100,
        minimum: None,
        bands: Cow::Owned(bands),
    })
}

fn tasmania(args: &MotorVehicleDutyArgs) -> Result<Scale> {
    let why = "in Tasmania (Duties Act 2001 (Tas) s 197(3A))";
    let fleet = only_flag(true, args.fleet_discount, "fleet_discount", why)?;
    Ok(if fleet { TAS_FLEET } else { TAS })
}

/// Each conditional input given only where it is asked.
fn only_where_asked(place: &str, args: &MotorVehicleDutyArgs) -> Result<()> {
    let asked = [
        ("engine", args.engine, "qld", "in Queensland"),
        ("green_car", args.green_car, "vic", "in Victoria"),
        (
            "primary_producer",
            args.primary_producer,
            "vic",
            "in Victoria",
        ),
        (
            "emissions_category",
            args.emissions_category,
            "act",
            "in the ACT",
        ),
        ("new_vehicle", args.new_vehicle, "act", "in the ACT"),
        ("fleet_discount", args.fleet_discount, "tas", "in Tasmania"),
        ("electric", args.electric, "nt", "in the Northern Territory"),
    ];
    for (key, given, home, why) in asked {
        if home != place {
            when(false, given, key, why)?;
        }
    }
    Ok(())
}

/// The rule for `place`, once its conditional inputs are where they belong.
fn rule(place: &str, args: &MotorVehicleDutyArgs, day: i64) -> Result<Rule> {
    only_where_asked(place, args)?;
    Ok(match place {
        "nsw" => Rule::Banded(NSW),
        "vic" => Rule::Banded(victoria(args)?),
        "qld" => Rule::Banded(queensland(args.engine)?),
        "wa" => Rule::WesternAustralia,
        "sa" => Rule::Banded(SA),
        "tas" => Rule::Banded(tasmania(args)?),
        "act" => Rule::Banded(act(args, day)?),
        _ => {
            let why = "in the Northern Territory (Stamp Duty Act 1978 (NT) s 92A)";
            let electric = only_flag(true, args.electric, "electric", why)?;
            Rule::Banded(if electric { NT_ELECTRIC } else { NT })
        }
    })
}

/// The band of `scale` that holds `value`: the last whose `from` it reaches.
fn band_for<'a>(scale: &'a Scale, value: &Decimal) -> Result<&'a Band> {
    let mut found = &scale.bands[0];
    for band in scale.bands.iter() {
        if !value.lt(&int(band.from))? {
            found = band;
        }
    }
    Ok(found)
}

/// `amount` rounded up to a whole number.
fn up(amount: &Decimal) -> Result<Decimal> {
    let down = amount.quantize(0, Rounding::Down)?;
    Ok(if down.lt(amount)? {
        down.add(&int(1))?
    } else {
        down
    })
}

/// The duty on `value` from the band of `scale` that holds it, and the band's
/// figures as rates.
fn banded(scale: &Scale, value: &Decimal) -> Result<(Step, Rates)> {
    let band = band_for(scale, value)?;
    let [base, applied, over] = [d(band.base), d(band.rate), int(band.over)];
    let minimum = scale.minimum.map(d);
    let per = scale.per;
    let name = format!("rate_per_{per}");
    let mut rates = vec![
        rate("base", band.base),
        rate(&name, band.rate),
        rate("threshold", band.over),
    ];
    let lots = up(&value.sub(&over)?.div(&int(per))?)?;
    let charged = lots.mul(&applied)?;
    let (mut amount, mut formula, mut operands) = if band.rate == "0" {
        (base.clone(), "base".to_string(), vec![&base])
    } else if band.over == 0 {
        let formula = format!("dollars_up(dutiable_value / {per}) x {name}");
        (charged, formula, vec![value, &applied])
    } else {
        let formula = format!("base + dollars_up((dutiable_value - threshold) / {per}) x {name}");
        (
            base.add(&charged)?,
            formula,
            vec![&base, value, &over, &applied],
        )
    };
    if let Some(least) = &minimum {
        if amount.lt(least)? {
            amount = least.clone();
        }
        formula = format!("max(minimum, {formula})");
        operands.insert(0, least);
        rates.push(rate("minimum", least));
    }
    Ok((
        step("duty", scale.authority, &formula, &operands, amount),
        rates,
    ))
}

/// Western Australia's s 236(2) for a vehicle that is not a heavy vehicle,
/// rounded down to the nearest 5 cents (s 236(1)).
fn western_australia(value: &Decimal) -> Result<(Step, Rates)> {
    let to_five_cents = "floor(dutiable_value x {r} / 100 x 20) / 20";
    let (percent, r, operands_rate, mut rates) = wa_rate(value)?;
    let twenty = int(20);
    let amount = value
        .mul(&percent)?
        .div(&int(100))?
        .mul(&twenty)?
        .quantize(0, Rounding::Down)?
        .div(&twenty)?;
    let formula = to_five_cents.replace("{r}", r);
    let mut operands = vec![value];
    operands.extend(operands_rate.iter());
    rates.push(rate("rate_applied_percent", &percent));
    Ok((step("duty", WA, &formula, &operands, amount), rates))
}

/// Western Australia's rate for `value`: 2.75% to $25,000, R% to $50,000
/// (rounded to 2 decimal places) and 6.5% above; with its formula, operands
/// and rates.
fn wa_rate(value: &Decimal) -> Result<(Decimal, &'static str, Vec<Decimal>, Rates)> {
    let [low, high] = [int(25_000), int(50_000)];
    Ok(if value.gt(&low)? && !value.gt(&high)? {
        let [base, divisor] = [d("2.75"), d("6666.66")];
        let exact = base.add(&value.sub(&low)?.div(&divisor)?)?;
        let rates = vec![
            rate("base_rate", &base),
            rate("threshold", &low),
            rate("divisor", &divisor),
        ];
        let r = "cents(base_rate + (dutiable_value - threshold) / divisor)";
        (
            exact.quantize(-2, Rounding::HalfUp)?,
            r,
            vec![base, value.clone(), low, divisor],
            rates,
        )
    } else {
        let percent = if value.gt(&high)? {
            d("6.5")
        } else {
            d("2.75")
        };
        let rates = vec![rate("rate_percent", &percent)];
        (percent.clone(), "rate_percent", vec![percent], rates)
    })
}

pub fn motor_vehicle_duty(args: &MotorVehicleDutyArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = place(args.jurisdiction)?;
    passenger(args.vehicle_class)?;
    let day = duty_date(args.duty_date, &year)?;
    let value = money(args.dutiable_value)?;
    whole_dollars(&value, "dutiable values")?;
    let (step, rates) = precise(|| match rule(place, args, day)? {
        Rule::Banded(scale) => banded(&scale, &value),
        Rule::WesternAustralia => western_australia(&value),
    })?;
    derived_result(KIND, &year, vec![step], rates)
}
