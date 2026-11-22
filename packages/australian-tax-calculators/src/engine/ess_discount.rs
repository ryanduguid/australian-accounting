// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `ess_discount`: what ITAA 1997 Division 83A includes in one individual's
//! assessable income for an income year from 2020-21 to 2026-27 for employee
//! share scheme interests acquired from 1 July 2015, and what s 83A-310
//! excludes for a forfeited one. The citations are to ITAA 1997 compilation 268
//! (1 October 2026), read on 22 November 2026. Division 83A reads the same in
//! compilation 234 (1 July 2022); compilation 210 (1 July 2020) adds only the
//! cessation of employment taxing point, which Act No. 8, 2022 Sch 10 removed
//! for taxing points from 1 July 2022. Values, conditions and dates are the
//! operator's established facts; the command never values a share.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::{Arg, Result, flag, int, py_min, rate, refuse, result, scope, stated};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

mod deferral;
mod input;
mod valuation;

use input::{Conditions, Forfeiture, Held, Interest, Kind, State};

const KIND: &str = "ess_discount";
/// s 83A-35(2)(a).
const CAP: &str = "1000";
/// s 83A-35(2)(b).
const INCOME_LIMIT: &str = "180000";
/// s 83A-33(4).
const START_UP_TURNOVER: &str = "50000000";
/// s 83A-33(5)(a).
const START_UP_DISCOUNT: &str = "0.15";
/// s 83A-105(4)(c).
const SALARY_SACRIFICE: &str = "5000";

/// The facts as the command passes them; see the README for each.
pub struct EssDiscountArgs<'a> {
    pub interests: &'a Arg,
    pub income_test: &'a Arg,
    pub foreign_employment: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// An amount to the cent, half up, as the result shows amounts.
fn shown(amount: &Decimal) -> Result<String> {
    Ok(amount.quantize(-2, Rounding::HalfUp)?.to_string())
}

fn dec(text: &str) -> Decimal {
    Decimal::parse(text).expect("decimal literal")
}

/// The income year (1 July to 30 June) a day falls in, as `2025-26`.
fn income_year(day: i64) -> String {
    let (y, m, _) = dates::to_civil(day);
    let start = if m >= 7 { y } else { y - 1 };
    format!("{start}-{:02}", (start + 1) % 100)
}

/// s 83A-45(1), (2), (3) and (6).
fn general(c: &Conditions) -> bool {
    c.employed && c.ordinary_shares_only && c.not_share_trading_company && c.ten_percent_limit
}

/// All of s 83A-45: the general conditions and the minimum holding period.
fn further(c: &Conditions) -> bool {
    general(c) && c.minimum_holding_period
}

/// s 83A-33(1): subsections (2) to (6), s 83A-45 and, for a share,
/// s 83A-105(2).
fn start_up(kind: Kind, held: &Held<'_>, discount: &Decimal) -> Result<bool> {
    let c = &held.conditions;
    let Some(s) = &c.start_up else {
        return Ok(false);
    };
    let market = match &held.right {
        Some(t) => !t.exercise_price.lt(&t.share_market_value)?,
        None => !discount.gt(&dec(START_UP_DISCOUNT).mul(&held.value)?)?,
    };
    let company = s.unlisted && s.under_ten_years && !s.turnover.gt(&dec(START_UP_TURNOVER))?;
    Ok(company
        && market
        && s.employer_resident
        && further(c)
        && (kind == Kind::Right || c.broad_availability))
}

/// s 83A-105(1)(b) to (d): the subsection that defers, if one does.
fn deferral_rule(kind: Kind, c: &Conditions, sacrifice_ok: bool) -> Option<&'static str> {
    if !general(c) {
        return None;
    }
    match kind {
        Kind::Share if !c.broad_availability => None,
        _ if c.real_risk => Some("s 83A-105(3)"),
        Kind::Share if sacrifice_ok => Some("s 83A-105(4)"),
        Kind::Right if c.deferral_stated => Some("s 83A-105(6)"),
        _ => None,
    }
}

/// The value and discount of a held interest and whether s 83A-33 reduces it.
struct Assessed {
    discount: Decimal,
    start_up: bool,
}

fn assess(interest: &Interest<'_>) -> Result<Option<Assessed>> {
    let State::Held(held) = &interest.state else {
        return Ok(None);
    };
    let discount = held.value.sub(&held.consideration)?;
    let start_up = discount.gt(&int(0))? && start_up(interest.kind, held, &discount)?;
    Ok(Some(Assessed { discount, start_up }))
}

/// A share relying on s 83A-105(4): salary sacrificed, no real risk, the
/// discount equal to its market value, and otherwise deferred.
fn sacrificed(interest: &Interest<'_>, assessed: &Assessed) -> Result<bool> {
    let State::Held(held) = &interest.state else {
        return Ok(false);
    };
    let c = &held.conditions;
    Ok(interest.kind == Kind::Share
        && c.salary_sacrifice
        && !c.real_risk
        && !assessed.start_up
        && assessed.discount.gt(&int(0))?
        && assessed.discount.equals(&held.value)?
        && general(c)
        && c.broad_availability)
}

/// A salary sacrificed interest at a real risk of forfeiture that s 83A-105(4)
/// could also apply to: deferred by s 83A-105(3), its discount equal to its
/// market value.
fn may_count(kind: Kind, held: &Held<'_>, assessed: &Assessed) -> Result<bool> {
    let c = &held.conditions;
    Ok(c.salary_sacrifice
        && c.real_risk
        && !assessed.start_up
        && assessed.discount.gt(&int(0))?
        && assessed.discount.equals(&held.value)?
        && deferral_rule(kind, c, false) == Some("s 83A-105(3)"))
}

/// s 83A-105(4)(c) and (5): the total market value at acquisition of the
/// salary sacrificed shares in each income year, and whether it is within
/// $5,000.
fn sacrifice_years(
    interests: &[Interest<'_>],
    assessed: &[Option<Assessed>],
) -> Result<BTreeMap<String, bool>> {
    let mut totals: BTreeMap<String, Decimal> = BTreeMap::new();
    let mut at_risk: Vec<String> = Vec::new();
    for (interest, assessed) in interests.iter().zip(assessed) {
        let (State::Held(held), Some(assessed)) = (&interest.state, assessed) else {
            continue;
        };
        let year = income_year(interest.acquired);
        if sacrificed(interest, assessed)? {
            let total = totals.remove(&year).unwrap_or_else(|| int(0));
            totals.insert(year, total.add(&held.value)?);
        } else if may_count(interest.kind, held, assessed)? {
            at_risk.push(year);
        }
    }
    if let Some(year) = at_risk.iter().find(|y| totals.contains_key(*y)) {
        return refuse(
            "ess_discount.salary_sacrifice_unclear",
            format!(
                "In {year} salary sacrificed interests at a real risk of forfeiture sit beside shares relying on s 83A-105(4); whether the first count toward the $5,000 of s 83A-105(4)(c) is not settled here: work them separately."
            ),
        );
    }
    let limit = dec(SALARY_SACRIFICE);
    totals
        .into_iter()
        .map(|(year, total)| Ok((year, !total.gt(&limit)?)))
        .collect()
}

pub fn ess_discount(args: &EssDiscountArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    if flag(args.foreign_employment, "foreign_employment")? {
        return refuse("ess_discount.foreign_employment", FOREIGN);
    }
    let mut interests = input::interests(args.interests)?;
    let assessed = interests.iter().map(assess).collect::<Result<Vec<_>>>()?;
    let sacrifice = sacrifice_years(&interests, &assessed)?;
    let mut totals = Totals::new();
    let mut rows = Vec::new();
    for (interest, assessed) in interests.iter_mut().zip(&assessed) {
        let row = match (&mut interest.state, assessed) {
            (State::Held(held), Some(a)) => {
                let ok = sacrifice
                    .get(&income_year(interest.acquired))
                    .copied()
                    .unwrap_or(false);
                let ctx = Context {
                    number: interest.number,
                    kind: interest.kind,
                    acquired: interest.acquired,
                    year: &year,
                };
                held_row(&ctx, held, a, ok, &mut totals)?
            }
            (State::Forfeited(f), _) => {
                forfeited_row(interest.number, interest.kind, f, &mut totals)?
            }
            _ => unreachable!("assessed every held interest"),
        };
        rows.push(row);
    }
    let (reduction, adjusted) = reduction(&totals.eligible, args.income_test)?;
    let mut out = totals.result(&year, reduction, adjusted)?;
    out["interests"] = json!(rows);
    explain(&mut out, &totals);
    Ok(out)
}

/// The year's totals: question 12's labels D, E and F, the start-up
/// discounts, and the forfeiture exclusions by the year included.
struct Totals {
    eligible: Decimal,
    other: Decimal,
    deferred: Decimal,
    start_up: Decimal,
    excluded: BTreeMap<String, Decimal>,
}

impl Totals {
    fn new() -> Self {
        Self {
            eligible: int(0),
            other: int(0),
            deferred: int(0),
            start_up: int(0),
            excluded: BTreeMap::new(),
        }
    }

    fn result(&self, year: &str, reduction: Decimal, adjusted: Option<Decimal>) -> Result<Value> {
        let assessable = self
            .eligible
            .add(&self.other)?
            .add(&self.deferred)?
            .sub(&reduction)?;
        let mut amounts = vec![
            ("upfront_eligible_discounts", self.eligible.clone()),
            ("upfront_other_discounts", self.other.clone()),
            ("deferred_amounts", self.deferred.clone()),
            ("reduction", reduction),
            ("assessable_discount", assessable),
            ("start_up_discounts", self.start_up.clone()),
        ];
        if let Some(adjusted) = adjusted {
            amounts.push(("adjusted_taxable_income", adjusted));
        }
        result(
            KIND,
            year,
            amounts,
            vec![
                rate("reduction_cap", CAP),
                rate("income_test_limit", INCOME_LIMIT),
                rate("start_up_turnover_limit", START_UP_TURNOVER),
                rate("start_up_discount_limit", START_UP_DISCOUNT),
                rate("salary_sacrifice_limit", SALARY_SACRIFICE),
            ],
        )
    }
}

struct Context<'y> {
    number: usize,
    kind: Kind,
    acquired: i64,
    year: &'y str,
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Share => "share",
        Kind::Right => "right",
    }
}

/// A treatment's name, its rule and the income year and amount it includes.
type Treated = (&'static str, &'static str, Option<(String, Decimal)>);

/// One held interest's treatment, its amount for the year and its row.
fn held_row(
    ctx: &Context<'_>,
    held: &mut Held<'_>,
    a: &Assessed,
    sacrifice_ok: bool,
    totals: &mut Totals,
) -> Result<Value> {
    let mut row = json!({
        "interest": ctx.number,
        "kind": kind_name(ctx.kind),
        "acquired": dates::format(ctx.acquired),
        "consideration": shown(&held.consideration)?,
        "value_at_acquisition": shown(&held.value)?,
        "valuation": held.valuation,
        "discount": shown(&py_max0(&a.discount)?)?,
    });
    let (treatment, rule, amount) = treat(ctx, held, a, sacrifice_ok, totals, &mut row)?;
    held.reader.finish(&format!("an interest whose treatment is {treatment}: the deferral facts are taken only where Subdivision 83A-C applies"))?;
    row["treatment"] = json!(treatment);
    row["rule"] = json!(rule);
    row["income_year"] = json!(amount.as_ref().map(|(y, _)| y.clone()));
    let this_year = match &amount {
        Some((y, amount)) if y == ctx.year => amount.clone(),
        _ => int(0),
    };
    row["amount_in_year"] = json!(shown(&this_year)?);
    Ok(row)
}

/// Subdivision 83A-B or 83A-C, s 83A-33, and the totals the interest joins.
fn treat(
    ctx: &Context<'_>,
    held: &mut Held<'_>,
    a: &Assessed,
    sacrifice_ok: bool,
    totals: &mut Totals,
    row: &mut Value,
) -> Result<Treated> {
    let acquired_year = income_year(ctx.acquired);
    let in_year = acquired_year == ctx.year;
    if !a.discount.gt(&int(0))? {
        return Ok((
            "no_discount",
            "ITAA 1997 s 83A-20(1): not acquired at a discount",
            None,
        ));
    }
    if a.start_up {
        add_if(&mut totals.start_up, in_year, &a.discount)?;
        return Ok(("start_up", START_UP_RULE, Some((acquired_year, int(0)))));
    }
    if let Some(rule) = deferral_rule(ctx.kind, &held.conditions, sacrifice_ok) {
        return treat_deferred(ctx, held, rule, totals, row);
    }
    let c = &held.conditions;
    let eligible = c.non_discriminatory && !c.real_risk && further(c);
    let bucket = if eligible {
        &mut totals.eligible
    } else {
        &mut totals.other
    };
    add_if(bucket, in_year, &a.discount)?;
    let name = if eligible {
        "upfront_eligible"
    } else {
        "upfront"
    };
    Ok((
        name,
        UPFRONT_RULE,
        Some((acquired_year, a.discount.clone())),
    ))
}

/// Subdivision 83A-C: the amount at the taxing point, in its income year.
fn treat_deferred(
    ctx: &Context<'_>,
    held: &mut Held<'_>,
    rule: &'static str,
    totals: &mut Totals,
    row: &mut Value,
) -> Result<Treated> {
    let (point, amount, detail) = deferral::deferred(held, ctx.kind, ctx.acquired)?;
    merge(row, detail);
    let point_year = income_year(point);
    let amount = match amount {
        Some(amount) => amount,
        None if point_year == ctx.year => return missing_value(&held.reader.name, point),
        None => int(0),
    };
    add_if(&mut totals.deferred, point_year == ctx.year, &amount)?;
    Ok(("deferred", rule, Some((point_year, amount))))
}

fn py_max0(amount: &Decimal) -> Result<Decimal> {
    Ok(if amount.lt(&int(0))? {
        int(0)
    } else {
        amount.clone()
    })
}

fn add_if(total: &mut Decimal, yes: bool, amount: &Decimal) -> Result<()> {
    if yes {
        *total = total.add(amount)?;
    }
    Ok(())
}

fn merge(row: &mut Value, detail: Value) {
    if let (Value::Object(row), Value::Object(detail)) = (row, detail) {
        row.extend(detail);
    }
}

fn missing_value<T>(name: &str, point: i64) -> Result<T> {
    refuse(
        "ess_discount.taxing_point_value",
        format!(
            "{name}: the deferred taxing point is {} in this year; give taxing_point_value with that date and the market value then (s 83A-110(1)).",
            dates::format(point)
        ),
    )
}

/// s 83A-310: whether the amount included is excluded, and the row.
fn forfeited_row(number: usize, kind: Kind, f: &Forfeiture, totals: &mut Totals) -> Result<Value> {
    let not_a_choice = matches!(
        f.cause,
        "scheme_condition" | "ceased_employment" | "lapsed_unexercised" | "cancellation_allowed"
    );
    let excluded = not_a_choice
        && !f.protects_against_fall
        && !f.consideration_received
        && f.amount_included.gt(&int(0))?;
    let amount = if excluded {
        f.amount_included.clone()
    } else {
        int(0)
    };
    if excluded {
        let total = totals
            .excluded
            .remove(&f.year_included)
            .unwrap_or_else(|| int(0));
        totals
            .excluded
            .insert(f.year_included.clone(), total.add(&amount)?);
    }
    Ok(json!({
        "interest": number,
        "kind": kind_name(kind),
        "treatment": "forfeited",
        "rule": FORFEITURE_RULE,
        "cause": f.cause,
        "excluded": excluded,
        "amount_excluded": shown(&amount)?,
        "year_included": f.year_included,
    }))
}

/// s 83A-35: the reduction, and the adjusted taxable income when given.
fn reduction(eligible: &Decimal, test: &Arg) -> Result<(Decimal, Option<Decimal>)> {
    let adjusted = adjusted_taxable_income(test)?;
    if eligible.is_zero() {
        return Ok((int(0), adjusted));
    }
    let Some(income) = adjusted else {
        return refuse(
            "input.missing_field",
            "income_test is required with an upfront discount eligible for the reduction (s 83A-35(2)(b)).",
        );
    };
    let cut = if income.gt(&dec(INCOME_LIMIT))? {
        int(0)
    } else {
        py_min(eligible.clone(), dec(CAP))?
    };
    Ok((cut, Some(income)))
}

const INCOME_KEYS: [&str; 4] = [
    "taxable_income",
    "reportable_fringe_benefits_total",
    "reportable_super_contributions",
    "total_net_investment_loss",
];

/// s 83A-35(2)(b)(i) to (iv), summed.
fn adjusted_taxable_income(test: &Arg) -> Result<Option<Decimal>> {
    let fields = match test {
        Arg::Json(Value::Null) => return Ok(None),
        Arg::Json(Value::Object(fields)) => fields,
        _ => {
            return refuse(
                "ess_discount.income_test_invalid",
                "income_test must be an object with taxable_income, reportable_fringe_benefits_total, reportable_super_contributions and total_net_investment_loss, or null.",
            );
        }
    };
    stated::known_keys(fields, "income_test", "income_test", &INCOME_KEYS)?;
    let mut total = int(0);
    for key in INCOME_KEYS {
        total = total.add(&stated::amount(fields, "income_test", key)?)?;
    }
    Ok(Some(total))
}

const FOREIGN: &str = "Part of a discount relating to employment outside Australia is foreign source income (ss 83A-25(2) and 83A-110(2)), and foreign and temporary residents are taxed on it differently; this worksheet takes only an Australian resident, not a temporary resident, whose interests relate wholly to employment in Australia.";

const START_UP_RULE: &str = "ITAA 1997 s 83A-33: the start-up concession reduces the amount included for the interest to nil";

const UPFRONT_RULE: &str = "ITAA 1997 s 83A-25(1): the discount is included in the income year of acquisition; Subdivision 83A-C does not apply (s 83A-105(1))";

const FORFEITURE_RULE: &str = "ITAA 1997 s 83A-310: Division 83A (but Subdivision 83A-E) is taken never to have applied where an amount was included and the interest is forfeited, or a right lost without disposal or exercise, not by a choice other than to cease employment or (for a right) to let it lapse or be cancelled, nor by a condition protecting against a fall in market value; the earlier assessment may be amended";

const STEPS: [(&str, &str); 6] = [
    (
        "discount",
        "ITAA 1997 s 83A-25(1) with s 83A-315: market value when acquired (for an unlisted right, the regulations' value where chosen) less the consideration",
    ),
    ("start_up_discounts", START_UP_RULE),
    (
        "upfront_eligible_discounts",
        "Question 12 label D: discounts in the year's acquisitions taxed upfront that meet ss 83A-35(6), (7) and 83A-45",
    ),
    (
        "deferred_amounts",
        "ITAA 1997 s 83A-110(1): market value at the deferred taxing point (ss 83A-115, 83A-120) less the cost base, for taxing points in the year; question 12 label F",
    ),
    (
        "reduction",
        "ITAA 1997 s 83A-35(1) and (2): label D, at most $1,000, where taxable income, reportable fringe benefits, reportable super contributions and total net investment loss total $180,000 or less",
    ),
    (
        "assessable_discount",
        "Question 12 label B: labels D, E and F less the reduction",
    ),
];

/// The authorities, and the forfeiture exclusions by the year included.
fn explain(out: &mut Value, totals: &Totals) {
    out["authorities"] = Value::Object(
        STEPS
            .iter()
            .map(|(step, text)| ((*step).to_owned(), json!(text)))
            .collect::<Map<_, _>>(),
    );
    out["forfeiture_exclusions"] = Value::Object(
        totals
            .excluded
            .iter()
            .map(|(year, amount)| (year.clone(), json!(shown(amount).unwrap_or_default())))
            .collect(),
    );
}
