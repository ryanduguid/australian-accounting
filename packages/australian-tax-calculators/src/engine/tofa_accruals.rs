// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `tofa-accruals`: whether ITAA 1997 Division 230 reaches one financial
//! arrangement under s 230-455 on the operator's stated facts, and, where it
//! does and the accruals method applies to a sufficiently certain overall gain
//! or loss (s 230-100(2)), that gain or loss spread over the arrangement's
//! life (s 230-130(1)) by compounding accruals (s 230-135(2)(a)) or, where the
//! operator states its results approximate them, by straight line over days
//! (s 230-135(2)(b)), then allocated to income years (s 230-170). Sections as
//! compiled on 1 October 2026 (compilation no. 268, volume 5) and on the ATO
//! legal database, read on 26 November 2026; the ATO's guide to the taxation
//! of financial arrangements gives the worked examples the tests use.
//! Intervals are equal and of the stated whole months (s 230-135(4)); every
//! financial benefit falls on the start or on an interval's last day, so the
//! amount the rate applies to in an interval is the opening balance
//! (s 230-135(5) and (6)). The elective methods (Subdivisions 230-C to 230-F),
//! the effective interest method (s 230-140), particular gains (s 230-100(3)
//! and (3A)), the realisation method (s 230-180), re-estimation and
//! reassessment (ss 230-185 to 230-200) and balancing adjustments
//! (Subdivision 230-G) are refused or left out.

use serde_json::{Value, json};

use super::{Arg, Result, d, date_arg, flag, money, precise, refuse, result, scope, signed_money};
use crate::dates;
use crate::decimal::{Calc, Decimal, Rounding};

mod spread;
use spread::*;

const KIND: &str = "tofa_accruals";
const MOST_INTERVALS: i64 = 360;
const MOST_BENEFITS: usize = 361;
const BISECTIONS: usize = 200;

/// The worksheet's inputs, as the caller gave them.
pub struct TofaAccrualsArgs<'a> {
    pub start: &'a Arg,
    pub end: &'a Arg,
    pub interval_months: &'a Arg,
    pub financial_benefits: &'a Arg,
    pub method: &'a Arg,
    pub approximation_confirmed: &'a Arg,
    pub accruals_chosen: &'a Arg,
    pub entity_type: &'a Arg,
    pub aggregated_turnover: Option<&'a Arg>,
    pub assets: Option<&'a Arg>,
    pub financial_assets: Option<&'a Arg>,
    pub qualifying_security: &'a Arg,
    pub threshold_failed_earlier: &'a Arg,
    pub division_230_election: &'a Arg,
    pub elections: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The stated entity kind of s 230-455(1)(a).
#[derive(Clone, Copy, PartialEq)]
enum Entity {
    Individual,
    SuperOrScheme,
    AdiOrSecuritisation,
    Other,
}

/// The s 230-455 facts: which of subsections (2), (3) and (4) the entity
/// satisfies for the year it starts to have the arrangement.
struct Thresholds {
    entity: Entity,
    s2: bool,
    s3: bool,
    s4: bool,
}

/// The arrangement's dates: its start, its interval length and how many.
struct Term {
    start: i64,
    months: i64,
    count: i64,
}

impl Term {
    fn last_day(&self, k: i64) -> i64 {
        dates::months_end(self.start, k * self.months)
    }

    fn first_day(&self, k: i64) -> i64 {
        if k == 1 {
            self.start
        } else {
            self.last_day(k - 1) + 1
        }
    }
}

fn entity(given: &Arg) -> Result<Entity> {
    match given.text() {
        Some("individual") => Ok(Entity::Individual),
        Some("super_or_scheme") => Ok(Entity::SuperOrScheme),
        Some("adi_or_securitisation") => Ok(Entity::AdiOrSecuritisation),
        Some("other") => Ok(Entity::Other),
        _ => refuse(
            "tofa_accruals.entity_type_invalid",
            "entity_type must be individual, super_or_scheme (s 230-455(1)(a)(ii)), adi_or_securitisation ((a)(iii)) or other ((a)(iv)).",
        ),
    }
}

fn figure(given: Option<&Arg>, name: &str) -> Result<Decimal> {
    let Some(value) = given else {
        return refuse(
            "tofa_accruals.threshold_input",
            format!(
                "{name} is required for an entity other than an individual (s 230-455(2) to (4))."
            ),
        );
    };
    money(value)
}

/// Section 230-455(2) to (4), on the values the operator states for the year
/// each subsection tests.
fn thresholds(a: &TofaAccrualsArgs) -> Result<Thresholds> {
    let entity = entity(a.entity_type)?;
    let given = [a.aggregated_turnover, a.assets, a.financial_assets];
    if entity == Entity::Individual {
        if given.iter().any(Option::is_some) {
            return refuse(
                "tofa_accruals.threshold_input",
                "An individual's turnover and assets are not tested (s 230-455(1)(a)(i)): omit aggregated_turnover, assets and financial_assets.",
            );
        }
        return Ok(Thresholds {
            entity,
            s2: false,
            s3: false,
            s4: false,
        });
    }
    let turnover = figure(a.aggregated_turnover, "aggregated_turnover")?;
    let assets = figure(a.assets, "assets")?;
    let financial = figure(a.financial_assets, "financial_assets")?;
    let under = |v: &Decimal, limit: &str| v.lt(&d(limit));
    Ok(Thresholds {
        entity,
        s2: under(&assets, "100000000")?,
        s3: under(&turnover, "20000000")?,
        s4: under(&turnover, "100000000")?
            && under(&financial, "100000000")?
            && under(&assets, "300000000")?,
    })
}

/// Whether s 230-455(1) keeps Division 230 off the arrangement, with the reason.
fn excluded(t: &Thresholds, a: &TofaAccrualsArgs, short: bool) -> Result<Option<&'static str>> {
    if flag(a.division_230_election, "division_230_election")? {
        return Ok(None);
    }
    if t.entity != Entity::Individual
        && flag(a.threshold_failed_earlier, "threshold_failed_earlier")?
    {
        return Ok(None);
    }
    let size = match t.entity {
        Entity::Individual => true,
        Entity::SuperOrScheme => t.s2,
        Entity::AdiOrSecuritisation => t.s3,
        Entity::Other => t.s4,
    };
    let qualifying = flag(a.qualifying_security, "qualifying_security")?;
    Ok((size && (short || !qualifying)).then_some(
        "Division 230 does not apply: s 230-455(1) excludes the arrangement on the stated facts (the entity's kind and size, and an arrangement ending within 12 months or not a qualifying security), with no s 230-455(7) election.",
    ))
}

/// Section 230-100(4): an individual, or an entity satisfying s 230-455(2),
/// (3) or (4), with a qualifying security and no s 230-455(7) election.
fn subsection_4(t: &Thresholds, a: &TofaAccrualsArgs) -> Result<bool> {
    let small = t.entity == Entity::Individual || t.s2 || t.s3 || t.s4;
    Ok(small
        && flag(a.qualifying_security, "qualifying_security")?
        && !flag(a.division_230_election, "division_230_election")?)
}

fn elections(given: &Arg) -> Result<()> {
    const KNOWN: [&str; 5] = [
        "fair_value",
        "foreign_exchange_retranslation",
        "hedging",
        "financial_reports",
        "portfolio_fees",
    ];
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            "tofa_accruals.elections_invalid",
            "elections must be a list.",
        );
    };
    let named: Option<Vec<&str>> = items.iter().map(Value::as_str).collect();
    let Some(named) = named.filter(|n| n.iter().all(|e| KNOWN.contains(e))) else {
        return refuse(
            "tofa_accruals.elections_invalid",
            format!("elections may hold only {}.", KNOWN.join(", ")),
        );
    };
    if let Some(e) = named.first() {
        return refuse(
            "tofa_accruals.election_not_built",
            format!(
                "A {e} election displaces or changes the accruals method (s 230-40(4) and (5); Subdivisions 230-C to 230-F; s 230-150), which this worksheet does not apply."
            ),
        );
    }
    Ok(())
}

fn term(a: &TofaAccrualsArgs, year: &str) -> Result<Term> {
    let start = date_arg(a.start, "start")?;
    let end = date_arg(a.end, "end")?;
    let months = match a.interval_months {
        Arg::Int(n) if (1..=12).contains(n) => *n,
        _ => {
            return refuse(
                "tofa_accruals.interval_invalid",
                "interval_months must be a whole number from 1 to 12: intervals may not exceed 12 months (s 230-135(4)(a)).",
            );
        }
    };
    let first: i64 = year[..4].parse().expect("a supported year");
    let year_first = dates::parse(&format!("{first}-07-01")).expect("a date");
    let year_last = dates::parse(&format!("{}-06-30", first + 1)).expect("a date");
    let count = (1..=MOST_INTERVALS).find(|k| dates::months_end(start, k * months) >= end);
    match count {
        Some(count)
            if start >= year_first
                && start <= year_last
                && dates::months_end(start, count * months) == end =>
        {
            Ok(Term {
                start,
                months,
                count,
            })
        }
        _ => refuse(
            "tofa_accruals.term_invalid",
            format!(
                "start must fall in {year} and end must be the last day of a whole number of interval_months intervals from start, at most {MOST_INTERVALS} intervals; shorter first or last intervals (s 230-135(4)) are not built."
            ),
        ),
    }
}

/// The stated financial benefits, netted onto the start (index 0) and each
/// interval's last day (index k).
fn benefits(given: &Arg, t: &Term) -> Result<Vec<Decimal>> {
    const CODE: &str = "tofa_accruals.benefits_invalid";
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            CODE,
            format!("financial_benefits must be a list of 2 to {MOST_BENEFITS} objects."),
        );
    };
    if !(2..=MOST_BENEFITS).contains(&items.len()) {
        return refuse(
            CODE,
            format!("financial_benefits must be a list of 2 to {MOST_BENEFITS} objects."),
        );
    }
    let mut net = vec![d("0"); usize::try_from(t.count).expect("a count") + 1];
    for (n, v) in items.iter().enumerate() {
        let (day, amount) = benefit(v, n + 1)?;
        let index = if day == t.start {
            Some(0)
        } else {
            (1..=t.count).find(|k| t.last_day(*k) == day)
        };
        let Some(index) = index else {
            return refuse(
                "tofa_accruals.benefit_within_interval",
                format!(
                    "financial benefit {}: falls inside an interval; the amount the rate applies to must then have regard to its timing (s 230-135(6)), which this worksheet does not apply. Date each benefit on start or an interval's last day.",
                    n + 1
                ),
            );
        };
        let slot = &mut net[usize::try_from(index).expect("an index")];
        *slot = slot.add(&amount)?;
    }
    Ok(net)
}

fn benefit(v: &Value, n: usize) -> Result<(i64, Decimal)> {
    let o = v.as_object();
    let day = o
        .and_then(|o| o.get("date"))
        .and_then(Value::as_str)
        .and_then(dates::parse);
    let amount = o
        .and_then(|o| o.get("amount"))
        .and_then(Value::as_str)
        .and_then(|t| Decimal::parse_input(t).ok())
        .map(|v| signed_money(&Arg::Decimal(v)));
    let known = o.is_some_and(|o| o.keys().all(|k| k == "date" || k == "amount"));
    match (day, amount) {
        (Some(day), Some(Ok(amount))) if known && !amount.is_zero() => Ok((day, amount)),
        _ => refuse(
            "tofa_accruals.benefits_invalid",
            format!(
                "financial benefit {n}: needs a YYYY-MM-DD date and a non-zero amount (received positive, provided negative), at most 2 dp and 1e12 either side of nil, and nothing else."
            ),
        ),
    }
}

/// ITAA 1997 Division 230's reach (s 230-455) and its accruals method for
/// one arrangement's sufficiently certain overall gain or loss.
pub fn tofa_accruals(a: &TofaAccrualsArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    elections(a.elections)?;
    let t = term(a, &year)?;
    let net = benefits(a.financial_benefits, &t)?;
    let thresholds = thresholds(a)?;
    let short = t.last_day(t.count) <= dates::months_end(t.start, 12);
    if let Some(why) = excluded(&thresholds, a, short)? {
        let mut out = result(KIND, &year, vec![], vec![])?;
        out["division_230_applies"] = json!(false);
        out["reason"] = json!(why);
        return Ok(out);
    }
    if !flag(a.accruals_chosen, "accruals_chosen")? && !subsection_4(&thresholds, a)? {
        return refuse(
            "tofa_accruals.accruals_not_chosen",
            "The accruals method applies to an overall gain or loss only if chosen or if s 230-100(4) applies (s 230-100(2)(c)); particular gains (s 230-100(3) and (3A)) and the realisation method (s 230-180) are not built.",
        );
    }
    let straight = method(a)?;
    let s = precise(|| spread(&t, &net, straight))?;
    let out = result(
        KIND,
        &year,
        vec![
            ("overall_gain_or_loss", s.gain),
            ("first_income_year_amount", s.first_year),
        ],
        vec![(
            "rate_of_return_per_interval".to_string(),
            s.rate.quantize(-12, Rounding::HalfUp)?.to_string(),
        )],
    );
    let mut out = out?;
    out["division_230_applies"] = json!(true);
    out["method"] = json!(if straight {
        "straight_line"
    } else {
        "compounding"
    });
    out["intervals"] = Value::Array(s.intervals);
    out["income_years"] = Value::Array(s.years);
    Ok(out)
}
