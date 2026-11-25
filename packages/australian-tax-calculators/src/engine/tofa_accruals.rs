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

fn sign(v: &Decimal) -> Calc<i8> {
    Ok(if v.is_zero() {
        0
    } else if v.gt(&d("0"))? {
        1
    } else {
        -1
    })
}

/// The net benefits times (1 + rate) to the power of the intervals left: zero
/// at the rate of return.
fn horner(net: &[Decimal], x: &Decimal) -> Calc<Decimal> {
    let mut acc = d("0");
    for c in net {
        acc = acc.mul(x)?.add(c)?;
    }
    Ok(acc)
}

/// The rate of return per interval: with one change of sign among the net
/// benefits there is exactly one, found by bisection on 1 + rate.
fn rate_of_return(net: &[Decimal]) -> Result<Decimal> {
    let signs: Vec<i8> = net.iter().map(sign).collect::<Calc<_>>()?;
    let nonzero: Vec<i8> = signs.into_iter().filter(|s| *s != 0).collect();
    let changes = nonzero.windows(2).filter(|w| w[0] != w[1]).count();
    if changes != 1 {
        return refuse(
            "tofa_accruals.rate_not_unique",
            "The financial benefits must change sign exactly once (amounts provided, then received, or the reverse), so one rate of return spreads the gain or loss; other patterns are not built.",
        );
    }
    let (lead, tail) = (nonzero[0], nonzero[nonzero.len() - 1]);
    let (mut lo, mut hi) = (d("0.25"), d("4"));
    if sign(&horner(net, &lo)?)? != tail || sign(&horner(net, &hi)?)? != lead {
        return refuse(
            "tofa_accruals.rate_out_of_range",
            "The rate of return per interval falls outside -75% to 300%, the bounds of this worksheet.",
        );
    }
    for _ in 0..BISECTIONS {
        let mid = lo.add(&hi)?.div(&d("2"))?;
        if sign(&horner(net, &mid)?)? == lead {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(lo.add(&hi)?.div(&d("2"))?.sub(&d("1"))?)
}

/// Each interval's exact accrual: the opening balance times the rate, the last
/// taking what remains of the gain or loss (s 230-135(5) and (7)).
fn accruals(net: &[Decimal], rate: &Decimal, gain: &Decimal) -> Calc<Vec<(Decimal, Decimal)>> {
    let mut balance = net[0].neg()?;
    let mut so_far = d("0");
    let mut out = Vec::with_capacity(net.len() - 1);
    for (k, c) in net.iter().enumerate().skip(1) {
        let accrual = if k == net.len() - 1 {
            gain.sub(&so_far)?
        } else {
            balance.mul(rate)?
        };
        out.push((balance.clone(), accrual.clone()));
        so_far = so_far.add(&accrual)?;
        balance = balance.add(&accrual)?.sub(c)?;
    }
    Ok(out)
}

/// Cents by running total, so the rounded parts sum to the rounded whole.
fn rounded(parts: &[Decimal]) -> Calc<Vec<Decimal>> {
    let (mut sum, mut before) = (d("0"), d("0.00"));
    let mut out = Vec::with_capacity(parts.len());
    for p in parts {
        sum = sum.add(p)?;
        let now = sum.quantize(-2, Rounding::HalfUp)?;
        out.push(now.sub(&before)?);
        before = now;
    }
    Ok(out)
}

/// The income year (1 July to 30 June) holding `day`, as its label and index.
fn income_year(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    if m >= 7 { y } else { y - 1 }
}

fn label(first: i64) -> String {
    format!("{first}-{:02}", (first + 1) % 100)
}

/// Days of `from..=to` in each income year, from the first.
fn days_by_year(from: i64, to: i64) -> Vec<(i64, i64)> {
    let mut out: Vec<(i64, i64)> = Vec::new();
    let mut day = from;
    while day <= to {
        let y = income_year(day);
        let next = dates::parse(&format!("{}-07-01", y + 1)).expect("a date");
        let last = to.min(next - 1);
        out.push((y, last - day + 1));
        day = last + 1;
    }
    out
}

/// Each interval's accrual allocated to income years by days: the operator's
/// stated reasonable basis for an interval straddling 2 (s 230-170(2)).
fn by_year(t: &Term, parts: &[Decimal]) -> Calc<Vec<(i64, Decimal)>> {
    let mut years: Vec<(i64, Decimal)> = Vec::new();
    for (k, part) in (1..=t.count).zip(parts) {
        let (from, to) = (t.first_day(k), t.last_day(k));
        let whole = Decimal::from_int(to - from + 1);
        for (y, days) in days_by_year(from, to) {
            let share = part.mul(&Decimal::from_int(days))?.div(&whole)?;
            add_to(&mut years, y, &share)?;
        }
    }
    Ok(years)
}

fn add_to(years: &mut Vec<(i64, Decimal)>, y: i64, v: &Decimal) -> Calc<()> {
    match years.iter_mut().find(|(k, _)| *k == y) {
        Some((_, sum)) => *sum = sum.add(v)?,
        None => years.push((y, v.clone())),
    }
    Ok(())
}

/// The gain or loss by straight line over the days of the life
/// (s 230-135(2)(b)), allocated to income years.
fn straight_line(t: &Term, gain: &Decimal) -> Calc<Vec<(i64, Decimal)>> {
    let end = t.last_day(t.count);
    let whole = Decimal::from_int(end - t.start + 1);
    days_by_year(t.start, end)
        .into_iter()
        .map(|(y, days)| Ok((y, gain.mul(&Decimal::from_int(days))?.div(&whole)?)))
        .collect()
}

fn method(a: &TofaAccrualsArgs) -> Result<bool> {
    match a.method.text() {
        Some("compounding") => Ok(false),
        Some("straight_line") if flag(a.approximation_confirmed, "approximation_confirmed")? => {
            Ok(true)
        }
        Some("straight_line") => refuse(
            "tofa_accruals.approximation_not_confirmed",
            "A straight line is allowed only where its results approximate compounding accruals, having regard to the length of the period (s 230-135(2)(b)): establish that and set approximation_confirmed.",
        ),
        Some("effective_interest") => refuse(
            "tofa_accruals.method_not_built",
            "The effective interest method qualifies only under s 230-140(3)'s conditions, including an audited financial report, which this worksheet does not apply.",
        ),
        _ => refuse(
            "tofa_accruals.method_invalid",
            "method must be compounding or straight_line.",
        ),
    }
}

fn interval_rows(t: &Term, net: &[Decimal], shown: &[Decimal]) -> Calc<Vec<Value>> {
    let mut rows = Vec::with_capacity(shown.len());
    let mut opening = net[0].neg()?;
    for (k, accrual) in (1..=t.count).zip(shown) {
        let benefit = &net[usize::try_from(k).expect("an index")];
        let closing = opening.add(accrual)?.sub(benefit)?;
        rows.push(json!({
            "interval": k,
            "start": dates::format(t.first_day(k)),
            "end": dates::format(t.last_day(k)),
            "opening_balance": opening.quantize(-2, Rounding::HalfUp)?.to_string(),
            "accrual": accrual.to_string(),
            "financial_benefits": benefit.quantize(-2, Rounding::HalfUp)?.to_string(),
            "closing_balance": closing.quantize(-2, Rounding::HalfUp)?.to_string(),
        }));
        opening = closing;
    }
    Ok(rows)
}

fn year_rows(
    compounding: &[(i64, Decimal)],
    straight: Option<&[(i64, Decimal)]>,
) -> Calc<Vec<Value>> {
    let c = rounded(
        &compounding
            .iter()
            .map(|(_, v)| v.clone())
            .collect::<Vec<_>>(),
    )?;
    let s = match straight {
        Some(s) => Some(rounded(
            &s.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>(),
        )?),
        None => None,
    };
    Ok(compounding
        .iter()
        .enumerate()
        .map(|(i, (y, _))| {
            let mut row = json!({"income_year": label(*y), "compounding": c[i].to_string()});
            if let Some(s) = &s {
                row["straight_line"] = json!(s[i].to_string());
            }
            row
        })
        .collect())
}

struct Spread {
    rate: Decimal,
    gain: Decimal,
    intervals: Vec<Value>,
    years: Vec<Value>,
    first_year: Decimal,
}

fn spread(t: &Term, net: &[Decimal], straight: bool) -> Result<Spread> {
    let mut gain = d("0");
    for c in net {
        gain = gain.add(c)?;
    }
    if gain.is_zero() {
        return refuse(
            "tofa_accruals.no_gain_or_loss",
            "The financial benefits net to nil, so there is no overall gain or loss for the accruals method to spread (s 230-100(2)(a)).",
        );
    }
    let rate = rate_of_return(net)?;
    let exact = accruals(net, &rate, &gain)?;
    let parts: Vec<Decimal> = exact.iter().map(|(_, a)| a.clone()).collect();
    let shown = rounded(&parts)?;
    let compounding = by_year(t, &parts)?;
    let line = if straight {
        Some(straight_line(t, &gain)?)
    } else {
        None
    };
    let years = year_rows(&compounding, line.as_deref())?;
    let key = if straight {
        "straight_line"
    } else {
        "compounding"
    };
    Ok(Spread {
        first_year: d(years[0][key].as_str().expect("an amount")),
        intervals: interval_rows(t, net, &shown)?,
        years,
        rate,
        gain,
    })
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
    let mut out = result(
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
    )?;
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
