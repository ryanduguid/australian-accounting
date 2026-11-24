// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `contribution_caps` by year: each earlier year's unused concessional
//! contributions cap worked out from its contributions, applied to increase a
//! later year's cap earliest year first, and what is left of each after this
//! year's contributions and any proposed one, with the year it lapses. The
//! steps cite ITAA 1997 s 291-20(1) to (7) in compilation 268 (1 October 2026),
//! read through the Federal Register API on 16 November 2026 at 17:22 AEDT.

use serde_json::{Value, json};

use super::{
    Arg, Result, Step, caps_result, concessional_steps, d, int, lookup, money, precise, py_max,
    py_min, rate, refuse, scope, step,
};
use crate::decimal::{Calc, Decimal};
use crate::metadata as meta;

/// s 291-20(7): no unused cap for a year before 2018-19.
const FIRST_YEAR: i64 = 2018;
/// s 291-20(3)(c): the previous 5 financial years.
const YEARS_CARRIED: i64 = 5;

const UNUSED: &str = "ITAA 1997 s 291-20(6) and (7): the shortfall of the year's concessional contributions below its cap, from 2018-19; nil when they reach the cap, increased or not";
const APPLIED: &str = "ITAA 1997 s 291-20(3) to (5): below a $500,000 total superannuation balance just before the year, the unapplied unused cap of the previous 5 years increases the year's cap, earliest year first, by no more than the excess over it";
const CARRIED: &str = "ITAA 1997 s 291-20(3)(c): the year's unused cap not yet applied to increase a later year's cap";
const TOTAL: &str =
    "ITAA 1997 s 291-20(3)(c) and (4): the previously unapplied unused cap of the previous 5 years";
const WITH_PROPOSAL: &str =
    "ITAA 1997 s 291-20(1): the year's concessional contributions with the proposed one";
const REMAINING: &str = "ITAA 1997 s 291-20(3)(c): left after this year's increase, for a later year of which the year is one of the previous 5";
const EXPIRING: &str = "ITAA 1997 s 291-20(3)(c): left of the unused cap of 5 years before, which is not one of the next year's previous 5 years";
const THIS_YEAR_UNUSED: &str =
    "ITAA 1997 s 291-20(6): this year's shortfall below its cap, with any proposed contribution";
const EXCESS_WITH_PROPOSAL: &str =
    "ITAA 1997 s 291-20(1): the contributions with the proposed one over the cap available";

const SHAPE: &str = "earlier_concessional_contributions must be a list with one object for each earlier income year: year as YYYY-YY, concessional_contributions and, where they exceed that year's cap, total_super_balance, as decimal strings. A total across years cannot be split into each year's unused cap (ITAA 1997 s 291-20(6)).";

/// The carry-forward inputs: each earlier year's contributions, a proposed
/// contribution for the year, and whether the person has a defined benefit
/// interest, which this worksheet refuses.
pub struct CarryForward<'a> {
    pub earlier_contributions: &'a Arg,
    pub proposed_contribution: Option<&'a Arg>,
    pub defined_benefit_interest: &'a Arg,
    /// The first excess non-concessional contributions determination's day,
    /// for the release figures; `None` for none.
    pub determination_date: Option<&'a Arg>,
    /// The downsizer contribution test's inputs; `None` for none.
    pub downsizer: Option<&'a super::DownsizerArgs<'a>>,
}

/// One earlier income year as given: its first calendar year, its
/// concessional contributions and the total superannuation balance at the
/// 30 June before it.
struct Earlier {
    start: i64,
    contributions: Decimal,
    balance: Option<Decimal>,
}

/// A year's unused cap as the later years apply it: the amount and each
/// application by its step id.
struct Unused {
    start: i64,
    amount: Decimal,
    applied: Vec<(String, Decimal)>,
}

impl Unused {
    fn left(&self) -> Calc<Decimal> {
        self.applied
            .iter()
            .try_fold(self.amount.clone(), |left, (_, v)| left.sub(v))
    }

    /// What is left, as the unused cap less each application, by name.
    fn formula(&self) -> (String, Vec<Decimal>) {
        let mut formula = format!("unused_cap_{}", tag(self.start));
        let mut values = vec![self.amount.clone()];
        for (id, value) in &self.applied {
            formula.push_str(&format!(" - {id}"));
            values.push(value.clone());
        }
        (formula, values)
    }
}

/// A year's unapplied unused cap as a formula and the values its names stand for.
type Source = fn(&Unused) -> Calc<(String, Vec<Decimal>)>;

/// `2021-22` for 2021.
fn label(start: i64) -> String {
    format!("{start}-{:02}", (start + 1) % 100)
}

/// `2021_22` for 2021, as a step or rate name ends.
fn tag(start: i64) -> String {
    label(start).replace('-', "_")
}

/// The first calendar year of an income year written as YYYY-YY.
fn income_year(text: &str) -> Option<i64> {
    let (first, second) = text.split_once('-')?;
    let ok = first.len() == 4 && second.len() == 2;
    let digits = (first.bytes().chain(second.bytes())).all(|b| b.is_ascii_digit());
    let start: i64 = first.parse().ok().filter(|_| ok && digits)?;
    (second.parse::<i64>().ok()? == (start + 1) % 100).then_some(start)
}

fn cap_of(start: i64) -> Decimal {
    let year = label(start);
    let earlier = meta::EARLIER_CONCESSIONAL_CAPS
        .iter()
        .find(|(y, _)| *y == year);
    d(earlier.map_or_else(|| lookup(meta::CONTRIBUTION_CAPS, &year).0, |(_, cap)| cap))
}

fn refs(values: &[Decimal]) -> Vec<&Decimal> {
    values.iter().collect()
}

fn shape<T>() -> Result<T> {
    refuse("contribution_caps.earlier_years_invalid", SHAPE)
}

fn entry(item: &Value) -> Result<Earlier> {
    let Value::Object(fields) = item else {
        return shape();
    };
    let known = ["year", "concessional_contributions", "total_super_balance"];
    if fields.keys().any(|k| !known.contains(&k.as_str())) {
        return shape();
    }
    let amount = |key: &str| match fields.get(key) {
        None => Ok(None),
        Some(Value::String(text)) => match Decimal::parse_input(text) {
            Ok(v) => money(&Arg::Decimal(v)).map(Some),
            Err(_) => shape(),
        },
        Some(_) => shape(),
    };
    let (Some(start), Some(contributions)) = (
        fields
            .get("year")
            .and_then(Value::as_str)
            .and_then(income_year),
        amount("concessional_contributions")?,
    ) else {
        return shape();
    };
    if start < FIRST_YEAR {
        return refuse(
            "contribution_caps.year_before_carry_forward",
            format!(
                "{} is before 2018-19: ITAA 1997 s 291-20(7) gives no unused concessional contributions cap for it, so leave it out.",
                label(start)
            ),
        );
    }
    Ok(Earlier {
        start,
        contributions,
        balance: amount("total_super_balance")?,
    })
}

/// The earlier years, each year from 5 before `current` to the one before it,
/// and any years back to 2018-19, without a gap.
fn earlier_years(value: &Arg, current: i64) -> Result<Vec<Earlier>> {
    let Arg::Json(Value::Array(items)) = value else {
        return shape();
    };
    let mut years = items.iter().map(entry).collect::<Result<Vec<_>>>()?;
    years.sort_by_key(|e| e.start);
    let first = years.first().map_or(current, |e| e.start);
    let gapless = years
        .iter()
        .zip(first..)
        .all(|(e, expected)| e.start == expected);
    let last = years.last().map_or(first, |e| e.start);
    if !gapless || first > current - YEARS_CARRIED || last != current - 1 {
        return refuse(
            "contribution_caps.earlier_years_invalid",
            format!(
                "earlier_concessional_contributions must give each year from {} to {} once, and may add earlier years back to 2018-19 without a gap; this year's contributions are concessional_contributions.",
                label(current - YEARS_CARRIED),
                label(current - 1)
            ),
        );
    }
    Ok(years)
}

/// Increase year `to`'s cap from `pool`, earliest year first, by no more than
/// the excess `need` gives as its formula, the values its names stand for and
/// its amount: a step for each year applied (s 291-20(4) and (5)). `source`
/// gives a year's unapplied unused cap as a formula and its values.
fn apply(
    pool: &mut [Unused],
    to: i64,
    need: (&str, Vec<Decimal>, Decimal),
    source: Source,
) -> Calc<Vec<Step>> {
    let (need_formula, need_values, mut wanted) = need;
    let (mut steps, mut drawn) = (Vec::new(), Vec::<(String, Decimal)>::new());
    for unused in pool.iter_mut() {
        let available = unused.left()?;
        if wanted.is_zero() || available.is_zero() {
            continue;
        }
        let take = py_min(available, wanted.clone())?;
        let (formula, mut values) = source(unused)?;
        let mut excess = need_formula.to_string();
        values.extend(need_values.iter().cloned());
        for (id, value) in &drawn {
            excess.push_str(&format!(" - {id}"));
            values.push(value.clone());
        }
        let id = format!("applied_{}_in_{}", tag(unused.start), tag(to));
        let formula = format!("min({formula}, {excess})");
        steps.push(step(
            id.clone(),
            APPLIED,
            &formula,
            &refs(&values),
            take.clone(),
        ));
        wanted = wanted.sub(&take)?;
        unused.applied.push((id.clone(), take.clone()));
        drawn.push((id, take));
    }
    Ok(steps)
}

/// An earlier year's increase from the years before it, when its
/// contributions, `written` as the working shows them, exceed its cap.
fn increase(pool: &mut [Unused], year: &Earlier, written: &str) -> Result<Vec<Step>> {
    let (cap, name) = (cap_of(year.start), label(year.start));
    let Some(balance) = &year.balance else {
        return refuse(
            "contribution_caps.earlier_balance_missing",
            format!(
                "{name}'s concessional contributions exceed its ${cap} cap: give its total_super_balance at the 30 June before it, which decides whether ITAA 1997 s 291-20(3)(b) increased its cap."
            ),
        );
    };
    if !balance.lt(&d(meta::CARRY_FORWARD_BALANCE_LIMIT))? {
        return Ok(Vec::new());
    }
    let from = (year.start - YEARS_CARRIED).max(FIRST_YEAR);
    if pool.first().map_or(year.start, |u| u.start) > from {
        return refuse(
            "contribution_caps.earlier_years_short",
            format!(
                "{name}'s concessional contributions exceed its cap, so ITAA 1997 s 291-20(4) and (5) applied unused cap from {} onwards, earliest first: give each year from {}.",
                label(from),
                label(from)
            ),
        );
    }
    let excess = year.contributions.sub(&cap)?;
    let formula = format!("{written} - concessional_cap_{}", tag(year.start));
    let start = pool
        .iter()
        .position(|u| u.start >= from)
        .unwrap_or(pool.len());
    let source: Source = |u| Ok(u.formula());
    Ok(apply(
        &mut pool[start..],
        year.start,
        (&formula, vec![cap], excess),
        source,
    )?)
}

/// Each earlier year's increase and unused cap, in year order.
fn earlier_steps(years: &[Earlier]) -> Result<(Vec<Step>, Vec<Unused>)> {
    let (mut steps, mut pool) = (Vec::new(), Vec::new());
    for year in years {
        // Written as the sum adds it, to 0, so that -0 reads 0.
        let written = int(0).add(&year.contributions)?.to_string();
        let cap = cap_of(year.start);
        if year.contributions.gt(&cap)? {
            steps.extend(increase(&mut pool, year, &written)?);
        }
        let unused = py_max(int(0), cap.sub(&year.contributions)?)?;
        let t = tag(year.start);
        let formula = format!("max(0, concessional_cap_{t} - {written})");
        steps.push(step(
            format!("unused_cap_{t}"),
            UNUSED,
            &formula,
            &[&cap],
            unused.clone(),
        ));
        pool.push(Unused {
            start: year.start,
            amount: unused,
            applied: Vec::new(),
        });
    }
    Ok((steps, pool))
}

/// The previous 5 years' entries in `pool`, which runs to the year before `current`.
fn previous_five(pool: &mut [Unused], current: i64) -> &mut [Unused] {
    let start = pool.iter().position(|u| u.start >= current - YEARS_CARRIED);
    &mut pool[start.expect("the previous 5 years")..]
}

/// What each of the previous 5 years carries into this one, and their total.
fn carried_steps(window: &[Unused], steps: &mut Vec<Step>) -> Calc<Decimal> {
    let (mut names, mut values, mut total) = (Vec::new(), Vec::new(), int(0));
    for unused in window {
        let (formula, operands) = unused.formula();
        let left = unused.left()?;
        let id = format!("carried_forward_{}", tag(unused.start));
        steps.push(step(
            id.clone(),
            CARRIED,
            &formula,
            &refs(&operands),
            left.clone(),
        ));
        total = total.add(&left)?;
        names.push(id);
        values.push(left);
    }
    let formula = names.join(" + ");
    steps.push(step(
        "unused_concessional_cap",
        TOTAL,
        &formula,
        &refs(&values),
        total.clone(),
    ));
    Ok(total)
}

/// The year's facts: the balance before it, its contributions, a proposed
/// contribution and its cap.
struct ThisYear<'a> {
    balance: &'a Decimal,
    concessional: &'a Decimal,
    proposed: Option<&'a Decimal>,
    cap: &'a Decimal,
}

/// After the contributions and any proposed one: each earlier year's unused
/// cap applied, earliest first, what is left of each, what lapses at the end
/// of the year and the year's own unused cap.
fn this_year_steps(window: &mut [Unused], current: i64, year: &ThisYear) -> Calc<Vec<Step>> {
    let mut steps = Vec::new();
    let (name, total) = match year.proposed {
        Some(proposed) => {
            let total = year.concessional.add(proposed)?;
            let formula = "concessional_contributions + proposed_concessional_contribution";
            let operands = [year.concessional, proposed];
            let id = "contributions_with_proposal";
            steps.push(step(id, WITH_PROPOSAL, formula, &operands, total.clone()));
            (id, total)
        }
        None => ("concessional_contributions", year.concessional.clone()),
    };
    let carried: Vec<Decimal> = window.iter().map(Unused::left).collect::<Calc<_>>()?;
    if year.balance.lt(&d(meta::CARRY_FORWARD_BALANCE_LIMIT))? {
        let excess = py_max(int(0), total.sub(year.cap)?)?;
        let need = format!("max(0, {name} - concessional_cap)");
        let values = vec![total.clone(), year.cap.clone()];
        let source: Source = |u| Ok((format!("carried_forward_{}", tag(u.start)), vec![u.left()?]));
        steps.extend(apply(window, current, (&need, values, excess), source)?);
    }
    steps.extend(left_steps(window, &carried, current)?);
    let unused = py_max(int(0), year.cap.sub(&total)?)?;
    let formula = format!("max(0, concessional_cap - {name})");
    let id = format!("unused_cap_{}", tag(current));
    steps.push(step(
        id,
        THIS_YEAR_UNUSED,
        &formula,
        &[year.cap, &total],
        unused,
    ));
    Ok(steps)
}

/// What is left of each of the previous 5 years' unused cap, `carried` into
/// the year, after it, and what lapses at its end.
fn left_steps(window: &[Unused], carried: &[Decimal], current: i64) -> Calc<Vec<Step>> {
    let mut steps = Vec::new();
    let this_year = format!("_in_{}", tag(current));
    for (unused, carried) in window.iter().zip(carried) {
        let t = tag(unused.start);
        let (mut formula, mut operands) = (format!("carried_forward_{t}"), vec![carried]);
        if let Some((id, value)) = unused
            .applied
            .last()
            .filter(|(id, _)| id.ends_with(&this_year))
        {
            formula.push_str(&format!(" - {id}"));
            operands.push(value);
        }
        steps.push(step(
            format!("remaining_{t}"),
            REMAINING,
            &formula,
            &operands,
            unused.left()?,
        ));
    }
    let lapsing = window[0].left()?;
    let formula = format!("remaining_{}", tag(window[0].start));
    steps.push(step(
        "unused_cap_expiring",
        EXPIRING,
        &formula,
        &[&lapsing],
        lapsing.clone(),
    ));
    Ok(steps)
}

/// Every concessional step: the earlier years', the year's from the unused
/// cap carried into it, and those after its contributions and any proposal.
fn by_year_steps(earlier: &[Earlier], current: i64, year: &ThisYear) -> Result<Vec<Step>> {
    let (mut steps, mut pool) = earlier_steps(earlier)?;
    let window = previous_five(&mut pool, current);
    let unused = carried_steps(window, &mut steps)?;
    steps.extend(concessional_steps(
        year.balance,
        year.concessional,
        &unused,
        year.cap,
    )?);
    steps.extend(this_year_steps(window, current, year)?);
    if year.proposed.is_some() {
        steps.push(excess_with_proposal(&steps)?);
    }
    Ok(steps)
}

/// `contribution_caps` with the unused concessional cap worked out from each
/// earlier year's contributions rather than given as a total.
pub fn contribution_caps_by_year(
    total_super_balance: &Arg,
    concessional_contributions: &Arg,
    non_concessional_contributions: &Arg,
    under_75_in_year: &Arg,
    carry: CarryForward,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "contribution_caps")?;
    no_defined_benefit_interest(carry.defined_benefit_interest)?;
    let balance = money(total_super_balance)?;
    let concessional = money(concessional_contributions)?;
    let non_concessional = money(non_concessional_contributions)?;
    let Arg::Bool(under_75) = under_75_in_year else {
        return refuse("input.flag_invalid", "under_75_in_year must be a boolean.");
    };
    let proposed = carry.proposed_contribution.map(money).transpose()?;
    let current: i64 = year[..4].parse().expect("supported period");
    let earlier = earlier_years(carry.earlier_contributions, current)?;
    let cap = d(lookup(meta::CONTRIBUTION_CAPS, &year).0);
    let facts = ThisYear {
        balance: &balance,
        concessional: &concessional,
        proposed: proposed.as_ref(),
        cap: &cap,
    };
    let steps = precise(|| by_year_steps(&earlier, current, &facts))?;
    let rates = earlier
        .iter()
        .map(|e| {
            rate(
                &format!("concessional_cap_{}", tag(e.start)),
                cap_of(e.start),
            )
        })
        .collect();
    let mut out = caps_result(
        &year,
        &balance,
        steps,
        (&non_concessional, *under_75),
        rates,
        (carry.determination_date, carry.downsizer),
    )?;
    out["unused_cap_expiry"] = expiry(&out, current);
    Ok(out)
}

fn no_defined_benefit_interest(flag: &Arg) -> Result<()> {
    match flag {
        Arg::Bool(false) => Ok(()),
        Arg::Bool(true) => refuse(
            "contribution_caps.defined_benefit_interest",
            "A defined benefit interest changes what counts as concessional contributions (ITAA 1997 Subdivision 291-C, notional taxed contributions under s 291-170, and s 291-370); the carry-forward by year is outside this worksheet for it.",
        ),
        _ => refuse(
            "input.flag_invalid",
            "defined_benefit_interest must be a boolean.",
        ),
    }
}

/// The excess once the proposed contribution is added, from the steps before it.
fn excess_with_proposal(steps: &[Step]) -> Calc<Step> {
    let value = |id: &str| {
        &steps
            .iter()
            .find(|s| s.id == id)
            .expect("an earlier step")
            .value
    };
    let (total, available) = (
        value("contributions_with_proposal"),
        value("concessional_available"),
    );
    let formula = "max(0, contributions_with_proposal - concessional_available)";
    let excess = py_max(int(0), total.sub(available)?)?;
    Ok(step(
        "excess_with_proposal",
        EXCESS_WITH_PROPOSAL,
        formula,
        &[total, available],
        excess,
    ))
}

/// Each of the previous 5 years' and this year's unused cap left after the
/// year, with the year at whose end it lapses.
fn expiry(out: &Value, current: i64) -> Value {
    let row = |start: i64, id: String| {
        json!({"year": label(start), "remaining": out["amounts"][&id],
               "expires_at_end_of": label(start + YEARS_CARRIED)})
    };
    let earlier =
        (current - YEARS_CARRIED..current).map(|s| row(s, format!("remaining_{}", tag(s))));
    let this_year = row(current, format!("unused_cap_{}", tag(current)));
    Value::Array(earlier.chain([this_year]).collect())
}
