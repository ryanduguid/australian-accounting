// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `insurance_duty`: the duty on one premium for general insurance in any
//! state or territory, for a contract effected or renewed in 2026-27, at the
//! rate each Act sets for the class the operator states (New South Wales:
//! Revenue NSW, the legislation site being unreadable). A class whose duty
//! is gone (the ACT's since 1 July 2017, New South Wales Type C since 1
//! January 2018) is nil with the date it went. Every rate was read on 24
//! November 2026; docs/calculation-evidence.md#insurance-duty lists the
//! versions read.

use serde_json::Value;

use super::business_capital::year_first;
use super::trade_in::when;
use super::{
    Arg, Result, Step, d, date_arg, derived_result, flag, money, precise, rate, refuse, scope, step,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "insurance_duty";
const PLACES: [&str; 8] = ["nsw", "vic", "qld", "wa", "sa", "tas", "act", "nt"];

/// The facts as the command passes them; see the README for each.
pub struct InsuranceDutyArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub class: &'a Arg,
    pub premium: &'a Arg,
    pub effected_date: &'a Arg,
    pub term_over_one_year: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// How a class is charged: a percentage of the premium, a fixed amount on
/// each premium, or nil from the date the duty went.
#[derive(Clone, Copy)]
enum Duty {
    Percent(&'static str),
    PerPremium(&'static str),
    Abolished(&'static str),
}

struct Class {
    place: &'static str,
    code: &'static str,
    duty: Duty,
    authority: &'static str,
}

const fn class(
    place: &'static str,
    code: &'static str,
    duty: Duty,
    authority: &'static str,
) -> Class {
    Class {
        place,
        code,
        duty,
        authority,
    }
}

const CLASSES: [Class; 13] = [
    class(
        "nsw",
        "type_a",
        Duty::Percent("9"),
        "Revenue NSW, 'Types of insurance': Type A, any general insurance other than Type B or Type C, 9 per cent of the premium (Duties Act 1997 (NSW) Ch 8, not readable at the legislation site); read on 24 November 2026",
    ),
    class(
        "nsw",
        "type_b",
        Duty::Percent("5"),
        "Revenue NSW, 'Types of insurance': Type B, including motor vehicle, aviation, disability income, occupational indemnity and hospital and ancillary health benefits insurance, 5 per cent of the premium; read on 24 November 2026",
    ),
    class(
        "nsw",
        "type_c",
        Duty::Abolished("2018-01-01"),
        "Revenue NSW, 'Types of insurance': Type C, including crop and livestock insurance, exempt for all policies taken out on or after 1 January 2018; read on 24 November 2026",
    ),
    class(
        "vic",
        "general",
        Duty::Percent("10"),
        "Duties Act 2000 (Vic) s 179(1) (authorised version 141): 10% of the premium for a contract of insurance other than business insurance; read on 24 November 2026",
    ),
    class(
        "vic",
        "business",
        Duty::Percent("7"),
        "Duties Act 2000 (Vic) s 179(2) and (4) and s 3(1) 'business insurance' (authorised version 141): 7% of the premium for a contract of business insurance effected or renewed from 1 July 2026 to 30 June 2027; read on 24 November 2026",
    ),
    class(
        "qld",
        "class_1",
        Duty::Percent("9"),
        "Duties Act 2001 (Qld) s 362(1)(a) (current as at 22 October 2026): 9% of the premium paid to effect class 1 general insurance; read on 24 November 2026",
    ),
    class(
        "qld",
        "class_2",
        Duty::Percent("9"),
        "Duties Act 2001 (Qld) s 362(1)(b) (current as at 22 October 2026): 9% of the premium paid to effect class 2 general insurance; read on 24 November 2026",
    ),
    class(
        "qld",
        "ctp",
        Duty::PerPremium("0.10"),
        "Duties Act 2001 (Qld) s 362(2) (current as at 22 October 2026): 10c on a premium for CTP insurance; read on 24 November 2026",
    ),
    class(
        "wa",
        "general",
        Duty::Percent("10"),
        "Duties Act 2008 (WA) s 215 (03-z0-00): 10% of the premium, or instalment, attributable to general insurance; read on 24 November 2026",
    ),
    class(
        "sa",
        "general",
        Duty::Percent("11"),
        "Stamp Duties Act 1923 (SA) s 34(1)(b) (version 15.9.2026): 11% of premiums for insurance of any kind other than life insurance; read on 24 November 2026",
    ),
    class(
        "tas",
        "general",
        Duty::Percent("10"),
        "Duties Act 2001 (Tas) s 166 (in force 24 November 2026; 10% from 1 October 2012): 10% of the premium paid in relation to a contract of general insurance; read on 24 November 2026",
    ),
    class(
        "act",
        "general",
        Duty::Abolished("2017-07-01"),
        "Duties Act 1999 (ACT) ch 8 (insurance), omitted by the Revenue Legislation Amendment Act 2017 (No 2) A2017-11 s 4, commenced 1 July 2017 (republication 79 endnotes); read on 24 November 2026",
    ),
    class(
        "nt",
        "general",
        Duty::Percent("10"),
        "Stamp Duty Act 1978 (NT) Sch 1 cl 6(3)(a) and (b)(i) (as in force at 1 July 2025): 10% of the premium for a policy of insurance issued or renewed for one year or less; read on 24 November 2026",
    ),
];

fn place(jurisdiction: &Arg) -> Result<&'static str> {
    match PLACES.iter().find(|p| Some(**p) == jurisdiction.text()) {
        Some(place) => Ok(place),
        None => refuse(
            "insurance_duty.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
        ),
    }
}

/// The class `code` names in `place`.
fn class_in(place: &str, code: &Arg) -> Result<&'static Class> {
    let theirs = || CLASSES.iter().filter(move |c| c.place == place);
    if let Some(found) = theirs().find(|c| Some(c.code) == code.text()) {
        return Ok(found);
    }
    let names: Vec<&str> = theirs().map(|c| c.code).collect();
    refuse(
        "insurance_duty.class_invalid",
        format!(
            "class in {place} must be one of {}; life, accident and workers compensation insurance are not worked.",
            names.join(", ")
        ),
    )
}

/// The day the contract was effected or renewed, inside the financial year.
fn effected(value: &Arg, year: &str) -> Result<()> {
    let first = year_first(year).expect("a supported period is a financial year");
    let start = dates::parse(&format!("{first}-07-01")).expect("a date");
    let end = dates::parse(&format!("{}-06-30", first + 1)).expect("a date");
    let day = date_arg(value, "effected_date")?;
    if (start..=end).contains(&day) {
        return Ok(());
    }
    refuse(
        "insurance_duty.date_outside_year",
        format!(
            "effected_date must be in the {year} financial year, from {} to {}.",
            dates::format(start),
            dates::format(end)
        ),
    )
}

/// The Northern Territory's term: a policy for more than one year is
/// charged '10% of the amount of the premium for each year and any
/// fractional part of a year in the term', which does not settle how a
/// premium for the whole term is charged.
fn term(place: &str, given: Option<&Arg>) -> Result<()> {
    let why = "in the Northern Territory (Stamp Duty Act 1978 (NT) Sch 1 cl 6(3))";
    let given = when(place == "nt", given, "term_over_one_year", why)?;
    if given.map_or(Ok(false), |v| flag(v, "term_over_one_year"))? {
        return refuse(
            "insurance_duty.term_unsettled",
            "A Northern Territory policy issued or renewed for more than one year is charged 10% of the premium 'for each year and any fractional part of a year in the term' (Sch 1 cl 6(3)(b)(ii)); how that applies to a premium for the whole term is unsettled.",
        );
    }
    Ok(())
}

fn worked(class: &Class, premium: &Decimal) -> Result<(Step, Vec<(String, String)>)> {
    let percent = |r: &str| -> Result<Decimal> {
        Ok(premium
            .mul(&d(r))?
            .div(&d("100"))?
            .quantize(-2, Rounding::HalfUp)?)
    };
    let formula = "cents(premium x rate_percent / 100)";
    Ok(match class.duty {
        Duty::Percent(r) => (
            step(
                "duty",
                class.authority,
                formula,
                &[premium, &d(r)],
                percent(r)?,
            ),
            vec![rate("rate_percent", r)],
        ),
        Duty::Abolished(from) => (
            step(
                "duty",
                class.authority,
                formula,
                &[premium, &d("0")],
                percent("0")?,
            ),
            vec![rate("rate_percent", "0"), rate("abolished_from", from)],
        ),
        Duty::PerPremium(amount) => (
            step(
                "duty",
                class.authority,
                "duty_per_premium",
                &[&d(amount)],
                d(amount),
            ),
            vec![rate("duty_per_premium", amount)],
        ),
    })
}

pub fn insurance_duty(args: &InsuranceDutyArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = place(args.jurisdiction)?;
    let class = class_in(place, args.class)?;
    effected(args.effected_date, &year)?;
    term(place, args.term_over_one_year)?;
    let premium = money(args.premium)?;
    let (step, rates) = precise(|| worked(class, &premium))?;
    derived_result(KIND, &year, vec![step], rates)
}
