// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `transfer_duty`: general transfer (stamp) duty on one dutiable value in
//! one of the 8 states and territories for 2026-27, and the duty an eligible
//! first home buyer of a home pays under that jurisdiction's concession. It
//! mirrors the transfer duty calculator on ryanduguid.github.io and follows the
//! Act where they differ: Victoria's duty is chargeable to the nearest whole
//! dollar (Duties Act 2000 (Vic) s 28(1)). Every figure was read on
//! 24 November 2026; docs/calculation-evidence.md#transfer-duty lists the
//! versions read.

use serde_json::Value;

use super::{
    Arg, Result, Step, d, derived_result, flag, int, money, precise, rate, refuse, scope, step,
    whole_dollars,
};
use crate::decimal::{Decimal, Rounding as Round};

mod scales;
use scales::{
    Band, Charge, FirstHome, PLACES, Place, QLD_FIRST_HOME, QLD_HOME, QLD_SCHEDULE_4A, Rounding,
};

const KIND: &str = "transfer_duty";

/// The facts as the command passes them; see the README for each.
pub struct TransferDutyArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub dutiable_value: &'a Arg,
    pub buyer: &'a Arg,
    pub residential: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Buyer {
    General,
    Established,
    New,
}

/// Rates as a step names them.
type Rates = Vec<(String, String)>;

/// The band of `bands` that holds `value`: the last whose `from` it reaches.
fn band<'a>(bands: &'a [Band], value: &Decimal) -> Result<&'a Band> {
    let mut found = &bands[0];
    for band in bands {
        if !value.lt(&int(band.from))? {
            found = band;
        }
    }
    Ok(found)
}

/// `amount` rounded up to the next whole number.
fn up(amount: &Decimal) -> Result<Decimal> {
    let down = amount.quantize(0, Round::Down)?;
    Ok(if down.lt(amount)? {
        down.add(&int(1))?
    } else {
        down
    })
}

/// `rate` for every $100 or part of $100 in `excess`.
fn per_hundred(excess: &Decimal, rate: &Decimal) -> Result<Decimal> {
    Ok(up(&excess.div(&int(100))?)?.mul(rate)?)
}

/// The Northern Territory's formula on `value` (Stamp Duty Act 1978 (NT) Sch 1
/// cl 1(2)).
fn territory(name: &str, value: &Decimal) -> Result<(Decimal, String, Vec<Decimal>, Rates)> {
    let v = value.div(&int(1000))?;
    let duty = d("0.06571441").mul(&v)?.mul(&v)?.add(&int(15).mul(&v)?)?;
    let formula = format!("0.06571441 x ({name} / 1000) x ({name} / 1000) + 15 x ({name} / 1000)");
    Ok((duty, formula, vec![value.clone(); 3], Vec::new()))
}

/// One band's duty on `value` (named `name` in the formula), with its formula,
/// operands and rates named with `prefix`.
fn charged(
    band: &Band,
    (name, value): (&str, &Decimal),
    prefix: &str,
) -> Result<(Decimal, String, Vec<Decimal>, Rates)> {
    let [base, applied, over] = [d(band.base), d(band.rate), int(band.over)];
    let named = |key: &str, figure: &Decimal| rate(&format!("{prefix}{key}"), figure);
    let excess = value.sub(&over)?;
    let (amount, formula, key) = match band.charge {
        Charge::PerHundred => (
            per_hundred(&excess, &applied)?,
            format!(
                "{prefix}base + dollars_up(({name} - {prefix}threshold) / 100) x {prefix}rate_per_100"
            ),
            "rate_per_100",
        ),
        Charge::Excess => (
            excess.mul(&applied)?,
            format!("{prefix}base + ({name} - {prefix}threshold) x {prefix}rate"),
            "rate",
        ),
        Charge::WholeValue => {
            let formula = format!("{name} x {prefix}rate");
            let operands = vec![value.clone(), applied.clone()];
            return Ok((
                value.mul(&applied)?,
                formula,
                operands,
                vec![named("rate", &applied)],
            ));
        }
        Charge::Territory => return territory(name, value),
    };
    Ok((
        base.add(&amount)?,
        formula,
        vec![base.clone(), value.clone(), over.clone(), applied.clone()],
        vec![
            named("base", &base),
            named(key, &applied),
            named("threshold", &over),
        ],
    ))
}

/// The duty on `value` at the place's general rate (with New South Wales'
/// premium rate on residential land), as step `id`, with the Act's minimum and
/// rounding.
fn general_step(
    place: &Place,
    (id, prefix): (&'static str, &str),
    (name, value): (&str, &Decimal),
    premium: bool,
) -> Result<(Step, Rates)> {
    let (band, authority) = match place.premium {
        Some((top, authority)) if premium && !value.lt(&int(top.from))? => (top, authority),
        _ => (band(place.general.bands, value)?, place.general.authority),
    };
    let (mut amount, mut formula, mut operands, mut rates) = charged(band, (name, value), prefix)?;
    if let Some(minimum) = place.minimum {
        let minimum = d(minimum);
        if amount.lt(&minimum)? {
            amount = minimum.clone();
        }
        formula = format!("max({formula}, minimum_duty)");
        operands.push(minimum.clone());
        rates.push(rate("minimum_duty", &minimum));
    }
    if place.rounding == Rounding::DollarHalfDown {
        amount = up(&amount.sub(&d("0.5"))?)?;
        if amount.is_zero() {
            // Below 50 cents: nil, not the negative zero the ceiling gives.
            amount = d("0");
        }
        formula = format!("dollars_up({formula} - 0.5)");
    }
    let operands: Vec<&Decimal> = operands.iter().collect();
    Ok((step(id, authority, &formula, &operands, amount), rates))
}

/// A step with a fixed amount, its formula the amount itself.
fn fixed(id: &'static str, authority: &'static str, amount: &str) -> Step {
    step(id, authority, amount, &[], d(amount))
}

/// The duty step when the value is outside a scheme's range: nil to
/// `exempt_to`, the general duty above `limit` (`at_limit`: from `limit`).
fn outside(
    authority: &'static str,
    value: &Decimal,
    (exempt_to, limit, at_limit): (i64, i64, bool),
    general: &Step,
) -> Result<Option<Step>> {
    if !value.gt(&int(exempt_to))? {
        return Ok(Some(fixed("duty", authority, "0")));
    }
    let beyond = if at_limit {
        !value.lt(&int(limit))?
    } else {
        value.gt(&int(limit))?
    };
    Ok(beyond.then(|| {
        step(
            "duty",
            authority,
            "general_duty",
            &[&general.value],
            general.value.clone(),
        )
    }))
}

/// Queensland's s 91 home concession duty, with the duty on $350,000 it
/// takes away above $350,000.
fn queensland_home(
    place: &Place,
    value: &Decimal,
    general: &Step,
    rates: &mut Rates,
) -> Result<Vec<Step>> {
    let threshold = int(350_000);
    if !value.gt(&threshold)? {
        let one = d("1.00");
        rates.push(rate("home_rate_per_100", &one));
        let amount = per_hundred(value, &one)?;
        let formula = "dollars_up(dutiable_value / 100) x home_rate_per_100";
        return Ok(vec![step(
            "home_concession_duty",
            QLD_HOME,
            formula,
            &[value, &one],
            amount,
        )]);
    }
    rates.push(rate("home_threshold", &threshold));
    let named = ("duty_on_350000", "home_threshold_");
    let (on, applied) = general_step(place, named, ("home_threshold", &threshold), false)?;
    rates.extend(applied);
    let home_base = int(3500);
    rates.push(rate("home_base", &home_base));
    let amount = home_base.add(&general.value)?.sub(&on.value)?;
    let operands = [&home_base, &general.value, &on.value];
    let formula = "home_base + general_duty - duty_on_350000";
    let home = step("home_concession_duty", QLD_HOME, formula, &operands, amount);
    Ok(vec![on, home])
}

/// Queensland's s 92: the home concession duty less the Sch 4A amount, or
/// all of it where it is less.
fn queensland(
    place: &Place,
    value: &Decimal,
    general: &Step,
    rates: &mut Rates,
) -> Result<Vec<Step>> {
    let mut steps = queensland_home(place, value, general, rates)?;
    let home = &steps[steps.len() - 1].value;
    let mut amount = int(0);
    for (from, figure) in QLD_SCHEDULE_4A {
        if !value.lt(&int(from))? {
            amount = d(figure);
        }
    }
    rates.push(rate("first_home_amount", &amount));
    let off = if amount.lt(home)? {
        amount.clone()
    } else {
        home.clone()
    };
    let duty = home.sub(&off)?;
    let formula = "home_concession_duty - min(home_concession_duty, first_home_amount)";
    let operands = [home, home, &amount];
    let duty = step("duty", QLD_FIRST_HOME, formula, &operands, duty);
    steps.push(duty);
    Ok(steps)
}

/// New South Wales' s 78A bracket: the general duty less the duty on
/// `exempt_to` scaled down evenly to nil at `limit`, rounded to the cent, with
/// the $20 minimum.
fn new_south_wales(
    place: &Place,
    authority: &'static str,
    (value, limit, exempt_to): (&Decimal, Decimal, Decimal),
    general: &Step,
    rates: &mut Rates,
) -> Result<Vec<Step>> {
    rates.push(rate("first_home_exempt_to", &exempt_to));
    rates.push(rate("first_home_limit", &limit));
    let named = ("duty_on_800000", "exempt_");
    let (on, applied) = general_step(place, named, ("first_home_exempt_to", &exempt_to), false)?;
    rates.extend(applied);
    let span = limit.sub(&exempt_to)?;
    let bracket = limit
        .sub(value)?
        .div(&span)?
        .mul(&on.value)?
        .quantize(-2, Round::HalfUp)?;
    let minimum = d(place.minimum.unwrap_or("0"));
    let mut duty = general.value.sub(&bracket)?;
    if duty.lt(&minimum)? {
        duty = minimum.clone();
    }
    let formula = "max(general_duty - cents((first_home_limit - dutiable_value) / (first_home_limit - first_home_exempt_to) x duty_on_800000), minimum_duty)";
    let operands = [
        &general.value,
        &limit,
        value,
        &limit,
        &exempt_to,
        &on.value,
        &minimum,
    ];
    let last = step("duty", authority, formula, &operands, duty);
    Ok(vec![on, last])
}

/// The steps from the first home concession to `duty`.
fn first_home(
    place: &Place,
    scheme: &FirstHome,
    value: &Decimal,
    general: &Step,
    rates: &mut Rates,
) -> Result<Vec<Step>> {
    match *scheme {
        FirstHome::None(authority) => Ok(vec![step(
            "duty",
            authority,
            "general_duty",
            &[&general.value],
            general.value.clone(),
        )]),
        FirstHome::Exempt(authority) => Ok(vec![fixed("duty", authority, "0")]),
        FirstHome::Queensland => queensland(place, value, general, rates),
        FirstHome::PhaseOut {
            authority,
            exempt_to,
            limit,
        } => bounded(authority, value, (exempt_to, limit, true), general, || {
            let edges = (value, int(limit), int(exempt_to));
            new_south_wales(place, authority, edges, general, rates)
        }),
        FirstHome::Share {
            authority,
            exempt_to,
            limit,
        } => bounded(authority, value, (exempt_to, limit, false), general, || {
            share(
                authority,
                value,
                (int(exempt_to), int(limit)),
                general,
                rates,
            )
        }),
        FirstHome::PerHundred {
            authority,
            exempt_to,
            limit,
            rate: per,
        } => bounded(authority, value, (exempt_to, limit, false), general, || {
            first_home_per_hundred(authority, value, (int(exempt_to), d(per)), rates)
        }),
    }
}

/// Nil or the general duty outside the concession's band; else `within`.
fn bounded(
    authority: &'static str,
    value: &Decimal,
    edges: (i64, i64, bool),
    general: &Step,
    within: impl FnOnce() -> Result<Vec<Step>>,
) -> Result<Vec<Step>> {
    match outside(authority, value, edges, general)? {
        Some(duty) => Ok(vec![duty]),
        None => within(),
    }
}

/// Victoria's s 57JA share of the general duty.
fn share(
    authority: &'static str,
    value: &Decimal,
    (from, to): (Decimal, Decimal),
    general: &Step,
    rates: &mut Rates,
) -> Result<Vec<Step>> {
    rates.extend([
        rate("first_home_exempt_to", &from),
        rate("first_home_limit", &to),
    ]);
    let amount = value
        .sub(&from)?
        .div(&to.sub(&from)?)?
        .mul(&general.value)?;
    let formula = "(dutiable_value - first_home_exempt_to) / (first_home_limit - first_home_exempt_to) x general_duty";
    let operands = [value, &from, &to, &from, &general.value];
    Ok(vec![step("duty", authority, formula, &operands, amount)])
}

/// Western Australia's first home owner rate for every $100 or part over the
/// exempt amount.
fn first_home_per_hundred(
    authority: &'static str,
    value: &Decimal,
    (from, per): (Decimal, Decimal),
    rates: &mut Rates,
) -> Result<Vec<Step>> {
    rates.extend([
        rate("first_home_exempt_to", &from),
        rate("first_home_rate_per_100", &per),
    ]);
    let amount = per_hundred(&value.sub(&from)?, &per)?;
    let formula =
        "dollars_up((dutiable_value - first_home_exempt_to) / 100) x first_home_rate_per_100";
    Ok(vec![step(
        "duty",
        authority,
        formula,
        &[value, &from, &per],
        amount,
    )])
}

const GENERAL_ONLY: &str = "No first home concession was asked for: the duty is the general duty";
const CONCESSION: &str =
    "The first home concession: the general duty less the duty the first home buyer pays";

/// Every step and the rates applied.
fn worked(
    place: &Place,
    value: &Decimal,
    buyer: Buyer,
    residential: bool,
) -> Result<(Vec<Step>, Rates)> {
    let (general, mut rates) = general_step(
        place,
        ("general_duty", ""),
        ("dutiable_value", value),
        residential,
    )?;
    let scheme = match buyer {
        Buyer::General => {
            let duty = step(
                "duty",
                GENERAL_ONLY,
                "general_duty",
                &[&general.value],
                general.value.clone(),
            );
            return Ok((vec![general, duty], rates));
        }
        Buyer::Established => &place.established,
        Buyer::New => &place.new,
    };
    let mut steps = first_home(place, scheme, value, &general, &mut rates)?;
    let duty = steps[steps.len() - 1].value.clone();
    let concession = general.value.sub(&duty)?;
    let operands = [&general.value, &duty];
    steps.push(step(
        "concession",
        CONCESSION,
        "general_duty - duty",
        &operands,
        concession,
    ));
    steps.insert(0, general);
    Ok((steps, rates))
}

/// The jurisdiction for `jurisdiction`.
fn place(jurisdiction: &Arg) -> Result<&'static Place> {
    match PLACES.iter().find(|p| Some(p.code) == jurisdiction.text()) {
        Some(place) => Ok(place),
        None => refuse(
            "transfer_duty.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
        ),
    }
}

fn buyer(given: &Arg) -> Result<Buyer> {
    match given.text() {
        Some("general") => Ok(Buyer::General),
        Some("first_home_established") => Ok(Buyer::Established),
        Some("first_home_new") => Ok(Buyer::New),
        _ => refuse(
            "transfer_duty.buyer_invalid",
            "buyer must be general, first_home_established or first_home_new.",
        ),
    }
}

/// Whether the land is wholly residential, refused where the worksheet holds
/// no rate for it.
fn residential(place: &Place, given: &Arg, buyer: Buyer) -> Result<bool> {
    let residential = flag(given, "residential")?;
    if residential {
        return Ok(true);
    }
    if buyer != Buyer::General {
        return refuse(
            "transfer_duty.first_home_not_residential",
            "A first home buyer concession applies to a home on residential land; residential must be true.",
        );
    }
    if matches!(place.code, "sa" | "act") {
        return refuse(
            "transfer_duty.non_residential_unsupported",
            "Land that is not wholly residential is not worked in sa (the Stamp Duties Act 1923 (SA) treats qualifying land apart) or act (DI2026-155 Part 3 sets commercial rates).",
        );
    }
    Ok(false)
}

pub fn transfer_duty(args: &TransferDutyArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = place(args.jurisdiction)?;
    let buyer = buyer(args.buyer)?;
    let residential = residential(place, args.residential, buyer)?;
    let value = money(args.dutiable_value)?;
    whole_dollars(&value, "dutiable values")?;
    if !value.gt(&int(0))? {
        return refuse(
            "transfer_duty.dutiable_value_nil",
            "dutiable_value must be more than nil.",
        );
    }
    let (steps, rates) = precise(|| worked(place, &value, buyer, residential))?;
    derived_result(KIND, &year, steps, rates)
}
