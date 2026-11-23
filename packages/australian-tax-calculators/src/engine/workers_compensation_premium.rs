// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `workers_compensation_premium`: one employer's workers compensation
//! premium for a 2026-27 policy in one state or territory, on the wages it
//! declares and the industry or premium rate its insurer gives it, with the
//! adjustment on its actual wages. It mirrors the workers compensation
//! calculator on ryanduguid.github.io: icare's small employer formula, the
//! WorkCover Premiums Order (No. 34) for a new or small Victorian employer,
//! WorkCover Queensland Notice (No. 1) of 2026 for a policy that commences in
//! 2026-27, ReturnToWorkSA's premium formula, and the insurer's rate times
//! wages in the 4 privately underwritten schemes. Every figure was read on
//! 24 November 2026; docs/calculation-evidence.md#workers-compensation-premium
//! lists the versions read.

use serde_json::Value;

use super::trade_in::when;
use super::{Arg, Error, Result, date_arg, flag, int, money, precise, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::Decimal;

mod schemes;
use schemes::{Facts, Premium, Qld, Sa, Vic};

const KIND: &str = "workers_compensation_premium";

/// The facts as the command passes them; see the README for each.
pub struct WorkersCompensationArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub wages: &'a Arg,
    pub rate: &'a Arg,
    pub group_member: &'a Arg,
    pub apprentice_wages: Option<&'a Arg>,
    pub actual_wages: Option<&'a Arg>,
    pub actual_apprentice_wages: Option<&'a Arg>,
    pub dust_diseases_rate: Option<&'a Arg>,
    pub safe_employer_reward: Option<&'a Arg>,
    pub employer: Option<&'a Arg>,
    pub days_registered: Option<&'a Arg>,
    pub excess_buy_out: Option<&'a Arg>,
    pub policy_start: Option<&'a Arg>,
    pub new_employer: Option<&'a Arg>,
    pub claims_costs: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The schemes, by the code the command takes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    Nsw,
    Vic,
    Qld,
    Wa,
    Sa,
    Tas,
    Act,
    Nt,
}

const PLACES: [(&str, Place); 8] = [
    ("nsw", Place::Nsw),
    ("vic", Place::Vic),
    ("qld", Place::Qld),
    ("wa", Place::Wa),
    ("sa", Place::Sa),
    ("tas", Place::Tas),
    ("act", Place::Act),
    ("nt", Place::Nt),
];

fn place(jurisdiction: &Arg) -> Result<Place> {
    match PLACES
        .iter()
        .find(|(code, _)| Some(*code) == jurisdiction.text())
    {
        Some((_, place)) => Ok(*place),
        None => refuse(
            "workers_compensation_premium.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
        ),
    }
}

/// `value` in units of 10^-`places`, for a decimal with at most `places`
/// decimal places.
fn scaled(value: &Decimal, places: u32) -> i128 {
    let text = value.fixed(places, false).replace('.', "");
    text.parse().expect("a bounded decimal's digits")
}

/// An amount in whole cents.
fn amount(value: &Arg) -> Result<i128> {
    Ok(scaled(&money(value)?, 2))
}

/// A percentage from `least` (excluded when `open`) to `most`, to at most 4
/// decimal places, in millionths of 1 (ten-thousandths of 1%).
fn percent(value: &Arg, name: &str, (least, open): (i64, bool), most: i64) -> Result<i128> {
    let parsed = match value {
        Arg::Decimal(v) if v.is_finite() && v.exponent() >= -4 => Some(v),
        _ => None,
    };
    if let Some(v) = parsed {
        let low = if open {
            v.gt(&int(least))?
        } else {
            !v.lt(&int(least))?
        };
        if low && !v.gt(&int(most))? {
            return Ok(scaled(v, 4));
        }
    }
    let above = if open { "above" } else { "from" };
    refuse(
        "workers_compensation_premium.rate_invalid",
        format!(
            "{name} must be a percentage {above} {least} to {most}, to at most 4 decimal places."
        ),
    )
}

/// The scheme's own facts, each asked for in its scheme only.
fn facts(place: Place, args: &WorkersCompensationArgs) -> Result<Facts> {
    let nsw = place == Place::Nsw;
    let why = "in nsw";
    let dust = when(nsw, args.dust_diseases_rate, "dust_diseases_rate", why)?;
    let reward = when(nsw, args.safe_employer_reward, "safe_employer_reward", why)?;
    let vic = vic(place == Place::Vic, args)?;
    let qld = qld(place == Place::Qld, args.policy_start)?;
    let sa = sa(place == Place::Sa, args)?;
    Ok(match (place, dust, reward) {
        (Place::Nsw, Some(dust), Some(reward)) => Facts::Nsw {
            dust: percent(dust, "dust_diseases_rate", (0, false), 10)?,
            reward: percent(reward, "safe_employer_reward", (0, false), 100)?,
        },
        (Place::Vic, ..) => Facts::Vic(vic.expect("asked in vic")),
        (Place::Qld, ..) => Facts::Qld(qld.expect("asked in qld")),
        (Place::Sa, ..) => Facts::Sa(sa.expect("asked in sa")),
        _ => Facts::Private,
    })
}

fn vic(asked: bool, args: &WorkersCompensationArgs) -> Result<Option<Vic>> {
    let why = "in vic";
    let employer = when(asked, args.employer, "employer", why)?;
    let days = when(asked, args.days_registered, "days_registered", why)?;
    let buy_out = when(asked, args.excess_buy_out, "excess_buy_out", why)?;
    let (Some(employer), Some(days), Some(buy_out)) = (employer, days, buy_out) else {
        return Ok(None);
    };
    match employer.text() {
        Some("new" | "small") => {}
        Some("other") => {
            return refuse(
                "workers_compensation_premium.experience_rated",
                "An employer that is neither new nor small (annualised 2025-26 remuneration over $200,000) has a performance-rated premium (Premiums Order (No. 34) clause 12): use WorkSafe's premium notice.",
            );
        }
        _ => {
            return refuse(
                "workers_compensation_premium.employer_invalid",
                "employer must be new, small or other.",
            );
        }
    }
    let days = match days.int() {
        Some(Some(n)) if (1..=365).contains(&n) => i128::from(n),
        _ => {
            return refuse(
                "workers_compensation_premium.days_invalid",
                "days_registered must be an integer from 1 to 365.",
            );
        }
    };
    let buy_out = flag(buy_out, "excess_buy_out")?;
    Ok(Some(Vic { days, buy_out }))
}

/// Section 4.1's minimum for a policy commencing on `start`, in cents.
fn qld(asked: bool, given: Option<&Arg>) -> Result<Option<Qld>> {
    let Some(start) = when(asked, given, "policy_start", "in qld")? else {
        return Ok(None);
    };
    let day = date_arg(start, "policy_start")?;
    let on = |text: &str| dates::parse(text).expect("a literal date");
    if day < on("2026-07-01") {
        return refuse(
            "workers_compensation_premium.policy_before_year",
            "A policy that commenced before 1 July 2026 has a premium rate from WorkCover Queensland's simplified or experience-based model and an adjustment for 2025-26 (Notice (No. 1) of 2026, parts 5 and 6): use the premium notice.",
        );
    }
    let minimum = [
        ("2026-09-30", 20_000),
        ("2026-12-31", 15_000),
        ("2027-03-31", 10_000),
        ("2027-06-30", 5_000),
    ]
    .iter()
    .find(|(last, _)| day <= on(last))
    .map(|(_, minimum)| *minimum);
    match minimum {
        Some(minimum) => Ok(Some(Qld { minimum })),
        None => refuse(
            "workers_compensation_premium.policy_after_year",
            "policy_start is after 30 June 2027: only a policy commencing in 2026-27 is worked.",
        ),
    }
}

fn sa(asked: bool, args: &WorkersCompensationArgs) -> Result<Option<Sa>> {
    let first = when(asked, args.new_employer, "new_employer", "in sa")?;
    let first = first.map_or(Ok(false), |given| flag(given, "new_employer"))?;
    let why = "in sa when new_employer is false";
    let claims = when(asked && !first, args.claims_costs, "claims_costs", why)?;
    let claims = claims.map_or(Ok(0), amount)?;
    Ok(asked.then_some(Sa { first, claims }))
}

/// Apprentice wages, which qld and sa deduct at the rate.
fn apprentice(place: Place, given: Option<&Arg>) -> Result<i128> {
    let wages = given.map_or(Ok(0), amount)?;
    if wages == 0 || matches!(place, Place::Qld | Place::Sa) {
        return Ok(wages);
    }
    if place == Place::Nsw {
        return refuse(
            "workers_compensation_premium.apprentice_unsettled",
            "icare discounts the apprentice incentive 'after premium payable is calculated' and does not say whether before or after GST: use the renewal notice.",
        );
    }
    refuse(
        "workers_compensation_premium.apprentice_unavailable",
        "apprentice_wages is above nil, but only qld and sa deduct apprentice wages at the rate: give nil and the wages as the scheme defines them.",
    )
}

/// The refusal on the actual wages, said of them.
fn on_actual(error: Error) -> Error {
    match error {
        Error::Value { code, message } => Error::Value {
            code,
            message: format!("On the actual wages: {message}"),
        },
        other => other,
    }
}

fn dollars(cents: i128) -> Result<Decimal> {
    let cents = i64::try_from(cents).expect("cents within a bounded amount");
    Ok(int(cents).div(&int(100))?)
}

/// The amounts: the scheme's steps, then the premium, GST, duty and total,
/// and with actual wages their total and the adjustment.
fn amounts(estimate: &Premium, actual: Option<&Premium>) -> Result<Vec<(&'static str, Decimal)>> {
    let mut out = Vec::new();
    for (key, cents) in &estimate.lines {
        out.push((*key, dollars(*cents)?));
    }
    let totals = [
        ("premium", estimate.premium),
        ("gst", estimate.gst),
        ("duty", estimate.duty),
        ("total", estimate.total),
    ];
    for (key, cents) in totals {
        out.push((key, dollars(cents)?));
    }
    if let Some(actual) = actual {
        out.push(("actual_total", dollars(actual.total)?));
        out.push(("adjustment", dollars(actual.total - estimate.total)?));
    }
    Ok(out)
}

/// Group members, whom the 4 public schemes rate together.
fn ungrouped(place: Place, group_member: &Arg) -> Result<()> {
    let grouped = flag(group_member, "group_member")?;
    if grouped && matches!(place, Place::Nsw | Place::Vic | Place::Qld | Place::Sa) {
        return refuse(
            "workers_compensation_premium.group_rated",
            "A member of a group is classed and rated on the group's wages or premiums together: use the insurer's notice.",
        );
    }
    Ok(())
}

/// The premium on the actual wages, when they are given, with the actual
/// apprentice wages in sa.
fn on_actual_wages(
    facts: &Facts,
    args: &WorkersCompensationArgs,
    rate: i128,
) -> Result<Option<Premium>> {
    let sa = matches!(facts, Facts::Sa(_));
    let why = "in sa with actual_wages";
    let given = args.actual_wages;
    let apprentice = when(
        sa && given.is_some(),
        args.actual_apprentice_wages,
        "actual_apprentice_wages",
        why,
    )?;
    let Some(given) = given else {
        return Ok(None);
    };
    if matches!(facts, Facts::Qld(_)) {
        return refuse(
            "workers_compensation_premium.adjustment_unsettled",
            "WorkCover Queensland Notice (No. 1) of 2026 sets no year-end adjustment on actual wages for a policy commencing in 2026-27: give no actual_wages.",
        );
    }
    let wages = amount(given)?;
    let apprentice = apprentice.map_or(Ok(0), amount)?;
    let premium = schemes::premium(facts, wages, rate, apprentice).map_err(on_actual)?;
    Ok(Some(premium))
}

pub fn workers_compensation_premium(args: &WorkersCompensationArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = place(args.jurisdiction)?;
    ungrouped(place, args.group_member)?;
    let wages = amount(args.wages)?;
    let applied = percent(args.rate, "rate", (0, true), 100)?;
    let apprentice = apprentice(place, args.apprentice_wages)?;
    schemes::within(apprentice, wages)?;
    let facts = facts(place, args)?;
    let estimate = schemes::premium(&facts, wages, applied, apprentice)?;
    let actual = on_actual_wages(&facts, args, applied)?;
    let mut rates = vec![
        rate("rate_percent", scaled_text(applied)),
        rate("gst_rate", "0.1"),
    ];
    rates.extend(estimate.rates.iter().map(|(k, v)| rate(k, v)));
    let amounts = precise(|| amounts(&estimate, actual.as_ref()))?;
    result(KIND, &year, amounts, rates)
}

/// A rate in millionths of 1 as the percentage it was given as.
fn scaled_text(millionths: i128) -> String {
    let whole = millionths / 10_000;
    let part = format!("{:04}", millionths % 10_000);
    let part = part.trim_end_matches('0');
    if part.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{part}")
    }
}
