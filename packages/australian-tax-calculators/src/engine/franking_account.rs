// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `franking_account`: one corporate tax entity's franking account for an
//! income year from 1 July to 30 June under ITAA 1997 Division 205, with the
//! benchmark rule (Division 203), the franking deficit tax offset (s 205-70)
//! and the franking return's amounts. Read on the ATO's legal database on
//! 19 November 2026 from 9.45 pm to 10.10 pm AEDT: ss 202-60, 202-65,
//! 202-75, 203-20 to 203-55, 204-70, 204-75, 205-15, 205-25, 205-30, 205-40,
//! 205-45, 205-50, 205-70, 214-15 to 214-25, 214-150 and 214-155; the New
//! Business Tax System (Franking Deficit Tax) Act 2002 s 5 and (Over-franking
//! Tax) Act 2002 s 5 on the Federal Register of Legislation the same evening;
//! and the ATO's Franking account tax return 2026 instructions (published
//! 30 May 2026): 'How to get', 'About', 'How to lodge and pay' and sections A
//! to C.

use serde_json::{Map, Value, json};

use super::{Arg, Result, d, flag, int, money, precise, rate, refuse, result, scope, signed_money};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "franking_account";
const COMMAND: &str = "franking-account";
const EVENT_FIELDS: [&str; 4] = ["date", "type", "amount", "rd_offset_part"];
const DISTRIBUTION_FIELDS: [&str; 5] = [
    "date",
    "amount",
    "franking_credit",
    "permitted_percentage",
    "statement_after_distribution",
];
const INCOME_TAX_FIELDS: [&str; 3] = [
    "before_fdt_offset",
    "offset_brought_forward",
    "earlier_year_liability",
];

/// An event the operator states: its type, whether it is a credit, its item
/// in the s 205-15(1) or s 205-30(1) table and what the amount is.
struct EventType {
    name: &'static str,
    credit: bool,
    item: &'static str,
}

const EVENT_TYPES: [EventType; 17] = [
    credit("payg_instalment", "ITAA 1997 s 205-15(1) item 1"),
    credit("income_tax_paid", "ITAA 1997 s 205-15(1) item 2"),
    credit(
        "franked_distribution_received",
        "ITAA 1997 s 205-15(1) item 3",
    ),
    credit(
        "franked_distribution_flowed_indirectly",
        "ITAA 1997 s 205-15(1) item 4",
    ),
    credit(
        "tax_offset_refund_excess_paid",
        "ITAA 1997 s 205-15(1) item 4A",
    ),
    credit("exploration_credit", "ITAA 1997 s 205-15(1) item 7"),
    credit("dmt_paid", "ITAA 1997 s 205-15(1) item 9"),
    debit("income_tax_refund", "ITAA 1997 s 205-30(1) item 2"),
    debit("linked_distribution", "ITAA 1997 s 205-30(1) item 5"),
    debit("tax_exempt_bonus_shares", "ITAA 1997 s 205-30(1) item 6"),
    debit("streaming_determination", "ITAA 1997 s 205-30(1) item 7"),
    debit("share_capital_tainting", "ITAA 1997 s 205-30(1) item 7A"),
    debit("untainting_choice", "ITAA 1997 s 205-30(1) item 7B"),
    debit("on_market_buy_back", "ITAA 1997 s 205-30(1) item 9"),
    debit("off_market_buy_back", "ITAA 1997 s 205-30(1) item 9A"),
    debit(
        "selective_capital_reduction",
        "ITAA 1997 s 205-30(1) item 9B",
    ),
    debit("dmt_refund", "ITAA 1997 s 205-30(1) item 14"),
];

const fn credit(name: &'static str, item: &'static str) -> EventType {
    EventType {
        name,
        credit: true,
        item,
    }
}

const fn debit(name: &'static str, item: &'static str) -> EventType {
    EventType {
        name,
        credit: false,
        item,
    }
}

const PAYMENT_ITEMS: [&str; 2] = ["payg_instalment", "income_tax_paid"];
const DIST_DEBIT: &str = "ITAA 1997 s 205-30(1) item 1";
const UNDER_FRANKING: &str = "ITAA 1997 s 205-30(1) item 3 and s 203-50(1)(b), (2) and (4)";
const FDT_CREDIT: &str = "ITAA 1997 s 205-15(1) item 5: the previous year's franking deficit tax";

/// One stated event, checked.
struct Event {
    date: i64,
    kind: &'static EventType,
    amount: Decimal,
    rd_part: Decimal,
}

/// One frankable distribution, checked, and what the benchmark rule makes of it.
struct Distribution {
    date: i64,
    amount: Decimal,
    allocated: Decimal,
    permitted: Option<Decimal>,
    late_statement: bool,
}

/// One line of the account: its date, authority, credit or debit and the
/// balance after it.
struct Line {
    date: i64,
    item: String,
    class: Class,
    amount: Decimal,
}

/// The facts that do not vary by event.
struct Facts {
    year_start: i64,
    year_end: i64,
    private: bool,
    exempt: bool,
    rate: Decimal,
    gross_up: Decimal,
}

fn bad<T>(code: &'static str, what: &str, n: usize, why: impl std::fmt::Display) -> Result<T> {
    refuse(code, format!("{what} {n}: {why}"))
}

/// A list input of at most 10,000 objects whose keys are all in `fields`.
fn objects<'a>(
    value: &'a Arg,
    name: &str,
    fields: &[&str],
    code: &'static str,
) -> Result<Vec<&'a Map<String, Value>>> {
    let Arg::Json(Value::Array(items)) = value else {
        return refuse(
            code,
            format!("{name} must be a list of at most 10000 objects."),
        );
    };
    if items.len() > 10_000 {
        return refuse(
            code,
            format!("{name} must be a list of at most 10000 objects."),
        );
    }
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let Some(o) = item.as_object() else {
                return bad(code, name, i + 1, "each must be an object.");
            };
            if let Some(key) = o.keys().find(|k| !fields.contains(&k.as_str())) {
                return refuse(
                    "input.field_unknown",
                    format!("{name} {}: {key} is not an input of {COMMAND}", i + 1),
                );
            }
            Ok(o)
        })
        .collect()
}

/// `field` of `o` as an amount of AUD, or `None` when it is not one.
fn aud(o: &Map<String, Value>, field: &str) -> Option<Decimal> {
    let parsed = o
        .get(field)?
        .as_str()
        .and_then(|s| Decimal::parse(s).ok())?;
    money(&Arg::Decimal(parsed)).ok()
}

/// `field` of `o` as a day number, or `None` when it is not a date.
fn day(o: &Map<String, Value>, field: &str) -> Option<i64> {
    o.get(field)?.as_str().and_then(dates::parse)
}

/// `value` as a percentage from 0 to 100, or `None`.
fn percentage(value: &Value) -> Option<Decimal> {
    let parsed = value.as_str().and_then(|s| Decimal::parse(s).ok())?;
    let ok = parsed.is_finite()
        && !parsed.lt(&int(0)).ok()?
        && parsed.le(&int(100)).ok()?
        && parsed.exponent() >= -10;
    ok.then_some(parsed)
}

fn event(n: usize, o: &Map<String, Value>, facts: &Facts) -> Result<Event> {
    let name = o.get("type").and_then(Value::as_str).unwrap_or_default();
    let Some(kind) = EVENT_TYPES.iter().find(|t| t.name == name) else {
        let names: Vec<&str> = EVENT_TYPES.iter().map(|t| t.name).collect();
        return bad(
            "franking_account.event_invalid",
            "event",
            n,
            format!("type must be one of {}.", names.join(", ")),
        );
    };
    let refund = kind.name == "income_tax_refund";
    // s 205-50(2): a refund paid within 3 months after the year's end can be
    // taken to have been received just before it.
    let last = if refund {
        dates::add_months(facts.year_end, 3)
    } else {
        facts.year_end
    };
    let date = match day(o, "date") {
        Some(date) if (facts.year_start..=last).contains(&date) => date,
        _ => {
            return bad(
                "franking_account.event_invalid",
                "event",
                n,
                format!(
                    "date must be a YYYY-MM-DD date in the income year{}.",
                    if refund {
                        " or the 3 months after it"
                    } else {
                        ""
                    }
                ),
            );
        }
    };
    let Some(amount) = aud(o, "amount").filter(|a| !a.is_zero()) else {
        return bad(
            "franking_account.event_invalid",
            "event",
            n,
            "amount must be a decimal string of AUD above nil, at most 2 decimal places and 1000000000000.00.",
        );
    };
    let rd_part = match (refund, o.get("rd_offset_part")) {
        (false, None) => int(0),
        (false, Some(_)) => {
            return bad(
                "franking_account.event_invalid",
                "event",
                n,
                "rd_offset_part is for an income_tax_refund only.",
            );
        }
        (true, _) => match aud(o, "rd_offset_part") {
            Some(part) if part.le(&amount)? => part,
            _ => {
                return bad(
                    "franking_account.event_invalid",
                    "event",
                    n,
                    "an income_tax_refund needs rd_offset_part, the part of the refund attributable to a refundable R&D tax offset (s 205-30(2)(b)), from 0.00 to the amount.",
                );
            }
        },
    };
    Ok(Event {
        date,
        kind,
        amount,
        rd_part,
    })
}

fn distribution(n: usize, o: &Map<String, Value>, facts: &Facts) -> Result<Distribution> {
    let Some(date) = day(o, "date").filter(|d| (facts.year_start..=facts.year_end).contains(d))
    else {
        return bad(
            "franking_account.distribution_invalid",
            "distribution",
            n,
            "date must be a YYYY-MM-DD date in the income year.",
        );
    };
    let Some(amount) = aud(o, "amount").filter(|a| !a.is_zero()) else {
        return bad(
            "franking_account.distribution_invalid",
            "distribution",
            n,
            "amount must be the frankable distribution in AUD, above nil, at most 2 decimal places.",
        );
    };
    let Some(allocated) = aud(o, "franking_credit") else {
        return bad(
            "franking_account.distribution_invalid",
            "distribution",
            n,
            "franking_credit must be the franking credit allocated to it in AUD, at most 2 decimal places.",
        );
    };
    let permitted = match o.get("permitted_percentage") {
        None => None,
        Some(value) => match percentage(value) {
            Some(p) => Some(p),
            None => {
                return bad(
                    "franking_account.distribution_invalid",
                    "distribution",
                    n,
                    "permitted_percentage must be a decimal string from 0 to 100.",
                );
            }
        },
    };
    let late_statement = match o.get("statement_after_distribution") {
        None => false,
        Some(Value::Bool(late)) if !*late || facts.private => *late,
        Some(Value::Bool(_)) => {
            return bad(
                "franking_account.distribution_invalid",
                "distribution",
                n,
                "only a private company may give a distribution statement after the distribution (s 202-75(2) and (3)).",
            );
        }
        Some(_) => {
            return bad(
                "franking_account.distribution_invalid",
                "distribution",
                n,
                "statement_after_distribution must be a boolean.",
            );
        }
    };
    Ok(Distribution {
        date,
        amount,
        allocated,
        permitted,
        late_statement,
    })
}

/// The corporate tax rate for imputation purposes and its gross-up rate,
/// (100% less the rate) over the rate (s 995-1(1), 'corporate tax gross-up
/// rate'): 25% or 30% for 2025-26 (the ATO's section B, 'Over-franking tax').
fn rates(value: &Arg) -> Result<(Decimal, Decimal)> {
    let rate = match value {
        Arg::Decimal(r) if r.equals(&d("0.25"))? || r.equals(&d("0.30"))? => r.clone(),
        _ => {
            return refuse(
                "franking_account.rate_invalid",
                "corporate_tax_rate must be \"0.25\" or \"0.30\": the entity's corporate tax rate for imputation purposes for the income year.",
            );
        }
    };
    let gross_up = precise(|| Ok(int(1).sub(&rate)?.div(&rate)?))?;
    Ok((rate, gross_up))
}

/// The franking periods of the year: the income year for a private company
/// (s 203-45), otherwise its first 6 months and the rest (s 203-40(2)).
fn periods(facts: &Facts) -> Vec<(i64, i64)> {
    if facts.private {
        return vec![(facts.year_start, facts.year_end)];
    }
    let second = dates::add_months(facts.year_start, 6);
    vec![(facts.year_start, second - 1), (second, facts.year_end)]
}

/// A franking period's benchmark (s 203-30) and the result for each
/// distribution in it.
struct Period {
    start: i64,
    end: i64,
    benchmark: Option<Decimal>,
}

/// What the benchmark rule makes of one distribution.
struct Worked {
    maximum: Decimal,
    credit: Decimal,
    percentage: Decimal,
    period: usize,
    reference: Option<Decimal>,
    over_franking_tax: Decimal,
    under_franking_debit: Decimal,
}

/// ss 202-60, 202-65 and 203-35: a distribution's maximum franking credit,
/// the franking credit on it and its franking percentage.
fn franking(dist: &Distribution, gross_up: &Decimal) -> Result<(Decimal, Decimal, Decimal)> {
    precise(|| {
        let maximum = dist.amount.div(gross_up)?;
        let credit = if dist.allocated.gt(&maximum)? {
            maximum.clone()
        } else {
            dist.allocated.clone()
        };
        let share = dist.allocated.div(&maximum)?.mul(&int(100))?;
        let percentage = if share.gt(&int(100))? {
            int(100)
        } else {
            share
        };
        Ok((maximum, credit, percentage))
    })
}

/// The benchmark rule over the year's distributions, in date order: the first
/// in each franking period sets its benchmark (s 203-30); a later one franked
/// above the benchmark, or the Commissioner's permitted percentage, gives
/// over-franking tax and one below it a franking debit, each the distribution
/// times the differential over the gross-up rate (s 203-50(2); Over-franking
/// Tax Act 2002 s 5). The rule does not apply where s 203-20 exempts the
/// entity.
fn benchmark(dists: &[Distribution], facts: &Facts, periods: &mut [Period]) -> Result<Vec<Worked>> {
    let mut out = Vec::new();
    for (n, dist) in dists.iter().enumerate() {
        let (maximum, credit, pct) = franking(dist, &facts.gross_up)?;
        let period = periods
            .iter()
            .position(|p| (p.start..=p.end).contains(&dist.date))
            .expect("a date in the year");
        let (reference, differential) = match &periods[period].benchmark {
            None => {
                if dist.permitted.is_some() {
                    return bad(
                        "franking_account.distribution_invalid",
                        "distribution",
                        n + 1,
                        "permitted_percentage applies to a distribution after the first in its franking period, whose franking percentage sets the benchmark (s 203-30).",
                    );
                }
                periods[period].benchmark = Some(pct.clone());
                (None, int(0))
            }
            Some(_) if facts.exempt => (None, int(0)),
            Some(set) => {
                let reference = dist.permitted.clone().unwrap_or_else(|| set.clone());
                let differential = precise(|| Ok(pct.sub(&reference)?))?;
                (Some(reference), differential)
            }
        };
        let over = differential.gt(&int(0))?;
        let under = differential.lt(&int(0))?;
        let size = if under {
            differential.neg()?
        } else {
            differential
        };
        let consequence = precise(|| {
            Ok(dist
                .amount
                .mul(&size)?
                .div(&int(100))?
                .div(&facts.gross_up)?)
        })?;
        out.push(Worked {
            maximum,
            credit,
            percentage: pct,
            period,
            reference,
            over_franking_tax: if over { consequence.clone() } else { int(0) },
            under_franking_debit: if under { consequence } else { int(0) },
        });
    }
    Ok(out)
}

/// How a line counts for the offset reduction (s 205-70(8)) and label K.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Credit,
    /// A debit under s 205-30(1) item 1, 3, 5 or 6.
    Trigger,
    /// A debit under item 2, which counts beside a trigger debit only.
    Refund,
    /// Any other debit: items 7, 7A, 7B, 9, 9A, 9B and 14 (label K).
    Other,
}

impl Line {
    fn new(date: i64, item: impl Into<String>, class: Class, amount: Decimal) -> Line {
        Line {
            date,
            item: item.into(),
            class,
            amount,
        }
    }
}

fn class_of(kind: &EventType) -> Class {
    match kind.name {
        _ if kind.credit => Class::Credit,
        "income_tax_refund" => Class::Refund,
        "linked_distribution" | "tax_exempt_bonus_shares" => Class::Trigger,
        _ => Class::Other,
    }
}

/// The account's lines, the R&D deferral left and the refunds after the
/// year's end: those taken to be received before it, with the franking
/// deficit tax each adds, and those left for the next year.
struct Account {
    lines: Vec<Line>,
    rd_pool: Decimal,
    deferred: Vec<(i64, Decimal)>,
    not_posted: Vec<Value>,
}

/// The sum of the credits less the debits, from `opening`.
fn balance(opening: &Decimal, lines: &[Line]) -> Result<Decimal> {
    precise(|| {
        lines.iter().try_fold(opening.clone(), |sum, line| {
            Ok(if line.class == Class::Credit {
                sum.add(&line.amount)?
            } else {
                sum.sub(&line.amount)?
            })
        })
    })
}

fn deficit(balance: &Decimal) -> Result<Decimal> {
    Ok(if balance.lt(&int(0))? {
        balance.neg()?
    } else {
        int(0)
    })
}

/// One stated event on the account. A PAYG instalment or income tax paid is
/// reduced by the R&D deferral left (s 205-15(4)); a refund's debit leaves
/// out its R&D part (s 205-30(2)(b)), which joins the deferral.
fn post_event(event: &Event, rd_pool: &mut Decimal, lines: &mut Vec<Line>) -> Result<()> {
    let mut item = event.kind.item.to_string();
    let mut amount = event.amount.clone();
    if PAYMENT_ITEMS.contains(&event.kind.name) && !rd_pool.is_zero() {
        let reduction = if rd_pool.lt(&amount)? {
            rd_pool.clone()
        } else {
            amount.clone()
        };
        amount = amount.sub(&reduction)?;
        *rd_pool = rd_pool.sub(&reduction)?;
        item = format!("{item}, less {reduction} under s 205-15(4)");
    }
    if event.kind.name == "income_tax_refund" {
        amount = amount.sub(&event.rd_part)?;
        *rd_pool = rd_pool.add(&event.rd_part)?;
        if !event.rd_part.is_zero() {
            item = format!(
                "{item}, less its R&D part {} under s 205-30(2)(b)",
                event.rd_part
            );
        }
    }
    lines.push(Line::new(event.date, item, class_of(event.kind), amount));
    Ok(())
}

/// The year's lines in date order (on one day, events as given, then
/// distributions), opening with the credit for the previous year's franking
/// deficit tax when the year opens in deficit.
fn year_lines(
    opening: &Decimal,
    events: &[Event],
    dists: &[Distribution],
    worked: &[Worked],
    facts: &Facts,
    rd_pool: &mut Decimal,
) -> Result<Vec<Line>> {
    let mut lines = Vec::new();
    if opening.lt(&int(0))? {
        lines.push(Line::new(
            facts.year_start,
            FDT_CREDIT,
            Class::Credit,
            opening.neg()?,
        ));
    }
    let mut order: Vec<(i64, usize, usize)> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.date <= facts.year_end)
        .map(|(i, e)| (e.date, 0, i))
        .chain(dists.iter().enumerate().map(|(i, dist)| (dist.date, 1, i)))
        .collect();
    order.sort();
    for (date, group, i) in order {
        if group == 0 {
            post_event(&events[i], rd_pool, &mut lines)?;
            continue;
        }
        let w = &worked[i];
        if !w.credit.is_zero() {
            lines.push(Line::new(
                date,
                DIST_DEBIT,
                Class::Trigger,
                w.credit.clone(),
            ));
        }
        if !w.under_franking_debit.is_zero() {
            lines.push(Line::new(
                date,
                UNDER_FRANKING,
                Class::Trigger,
                w.under_franking_debit.clone(),
            ));
        }
    }
    Ok(lines)
}

/// s 205-50(2): each refund paid in the 3 months after the year's end, in
/// date order, is taken to be received just before the end when it would put
/// the account into deficit or further into it; the others belong to the
/// next year.
fn after_year_end(
    opening: &Decimal,
    events: &[Event],
    facts: &Facts,
    account: &mut Account,
) -> Result<()> {
    let mut later: Vec<&Event> = events.iter().filter(|e| e.date > facts.year_end).collect();
    later.sort_by_key(|e| e.date);
    for refund in later {
        let before = balance(opening, &account.lines)?;
        let debit = precise(|| Ok(refund.amount.sub(&refund.rd_part)?))?;
        let after = precise(|| Ok(before.sub(&debit)?))?;
        if debit.is_zero() || !after.lt(&int(0))? {
            account.not_posted.push(json!({
                "date": dates::format(refund.date),
                "taken_into_year": false,
                "amount": shown(&refund.amount)?,
                "rd_offset_part": shown(&refund.rd_part)?,
                "reason": "ITAA 1997 s 205-50(2): received after the year's end without putting the account into deficit, or further into it, so it is a debit of the next year",
            }));
            continue;
        }
        let added = precise(|| Ok(deficit(&after)?.sub(&deficit(&before)?)?))?;
        account.rd_pool = precise(|| Ok(account.rd_pool.add(&refund.rd_part)?))?;
        let item = format!(
            "ITAA 1997 s 205-30(1) item 2, received {} and taken to be received just before the year's end under s 205-50(2)",
            dates::format(refund.date)
        );
        account
            .lines
            .push(Line::new(facts.year_end, item, Class::Refund, debit));
        account.deferred.push((refund.date, added));
    }
    Ok(())
}

/// An amount to the cent, half up, as text.
fn shown(value: &Decimal) -> Result<String> {
    Ok(precise(|| Ok(value.quantize(-2, Rounding::HalfUp)?))?.to_string())
}

/// The facts s 205-70 needs beyond the account: the income tax the offset
/// would reduce and the offset left from the last year.
struct IncomeTax {
    before_offset: Decimal,
    brought_forward: Decimal,
    earlier_liability: Option<bool>,
}

fn income_tax(value: Option<&Arg>, private: bool) -> Result<Option<IncomeTax>> {
    let Some(value) = value else { return Ok(None) };
    let Arg::Json(Value::Object(o)) = value else {
        return refuse(
            "franking_account.income_tax_invalid",
            "income_tax must be an object.",
        );
    };
    if let Some(key) = o.keys().find(|k| !INCOME_TAX_FIELDS.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("income_tax: {key} is not an input of {COMMAND}"),
        );
    }
    let (Some(before_offset), Some(brought_forward)) = (
        aud(o, "before_fdt_offset"),
        aud(o, "offset_brought_forward"),
    ) else {
        return refuse(
            "franking_account.income_tax_invalid",
            "income_tax needs before_fdt_offset, the income tax for the year after every tax offset s 63-10 applies before this one, and offset_brought_forward, the offset left from the last year (s 205-70(1)(c)), each in AUD.",
        );
    };
    let earlier_liability = match (o.get("earlier_year_liability"), private) {
        (Some(Value::Bool(b)), true) => Some(*b),
        (None, false) => None,
        (_, true) => {
            return refuse(
                "franking_account.income_tax_invalid",
                "income_tax needs earlier_year_liability for a private company: true when it had an income tax liability for any earlier income year (s 205-70(5)(c)).",
            );
        }
        (Some(_), false) => {
            return refuse(
                "franking_account.income_tax_invalid",
                "earlier_year_liability is for a private company only (s 205-70(5)(a)).",
            );
        }
    };
    Ok(Some(IncomeTax {
        before_offset,
        brought_forward,
        earlier_liability,
    }))
}

/// The year's totals the franking return and s 205-70 use.
struct Totals {
    credits: Decimal,
    debits: Decimal,
    closing: Decimal,
    fdt: Decimal,
    trigger: bool,
    attributable_debits: Decimal,
    label_k: Decimal,
}

fn sum_of(lines: &[Line], keep: impl Fn(&Line) -> bool) -> Result<Decimal> {
    precise(|| {
        lines
            .iter()
            .filter(|l| keep(l))
            .try_fold(int(0), |t, l| Ok(t.add(&l.amount)?))
    })
}

fn totals(opening: &Decimal, lines: &[Line]) -> Result<Totals> {
    let trigger = lines
        .iter()
        .any(|l| l.class == Class::Trigger && !l.amount.is_zero());
    let attributable_debits = sum_of(lines, |l| {
        l.class == Class::Trigger || (trigger && l.class == Class::Refund)
    })?;
    let label_k = sum_of(lines, |l| {
        l.class == Class::Other || (!trigger && l.class == Class::Refund)
    })?;
    let closing = balance(opening, lines)?;
    Ok(Totals {
        credits: sum_of(lines, |l| l.class == Class::Credit)?,
        debits: sum_of(lines, |l| l.class != Class::Credit)?,
        fdt: deficit(&closing)?,
        closing,
        trigger,
        attributable_debits,
        label_k,
    })
}

/// The current year's offset: the amount, the reduction, the franking
/// return's code and the authority.
struct Offset {
    amount: Decimal,
    reduction: Decimal,
    code: Option<&'static str>,
    authority: &'static str,
}

fn whole(t: &Totals, code: Option<&'static str>, authority: &'static str) -> Offset {
    Offset {
        amount: t.fdt.clone(),
        reduction: int(0),
        code,
        authority,
    }
}

/// s 205-70(2) step 1 by the ATO's method (section B, 'How to calculate the
/// reduced offset amount'): the franking deficit tax, less 30% of the part of
/// it attributable to item 1, 2, 3, 5 and 6 debits where that part exceeds
/// 10% of the year's credits; that part is the opening surplus and the
/// year's credits less those debits, when negative. Not reduced without an
/// item 1, 3, 5 or 6 debit (code F), for a private company meeting
/// s 205-70(5) (code P) or under a Commissioner's determination (s 205-70(6)).
fn current_offset(
    opening: &Decimal,
    t: &Totals,
    facts: &Facts,
    tax: Option<&IncomeTax>,
    determination: bool,
) -> Result<Offset> {
    if t.fdt.is_zero() {
        return Ok(whole(
            t,
            None,
            "ITAA 1997 s 205-70(1)(a): no franking deficit tax, so no offset for the year",
        ));
    }
    if !t.trigger {
        return Ok(whole(
            t,
            Some("F"),
            "ITAA 1997 s 205-70(2) step 1 and (8): no item 1, 3, 5 or 6 debit, so the offset is the whole franking deficit tax",
        ));
    }
    let surplus = if opening.gt(&int(0))? {
        opening.clone()
    } else {
        int(0)
    };
    let step3 = precise(|| Ok(surplus.add(&t.credits)?.sub(&t.attributable_debits)?))?;
    let part = deficit(&step3)?;
    let tenth = precise(|| Ok(t.credits.mul(&d("0.10"))?))?;
    if !part.gt(&tenth)? {
        return Ok(whole(
            t,
            None,
            "ITAA 1997 s 205-70(2) step 1: the part of the deficit attributable to item 1, 2, 3, 5 and 6 debits is not more than 10% of the year's credits, so the offset is the whole franking deficit tax",
        ));
    }
    if determination {
        return Ok(whole(
            t,
            None,
            "ITAA 1997 s 205-70(6): the Commissioner determined that the excess was due to events outside the entity's control, so the offset is not reduced",
        ));
    }
    if facts.private && private_exception(t, tax)? {
        return Ok(whole(
            t,
            Some("P"),
            "ITAA 1997 s 205-70(5): a private company's first year of income tax liability, at least 90% of the deficit, so the offset is not reduced",
        ));
    }
    let reduction = precise(|| Ok(part.mul(&d("0.30"))?))?;
    Ok(Offset {
        amount: precise(|| Ok(t.fdt.sub(&reduction)?))?,
        reduction,
        code: None,
        authority: "ITAA 1997 s 205-70(2) step 1: the part of the deficit attributable to item 1, 2, 3, 5 and 6 debits exceeds 10% of the year's credits, so it is reduced by 30%",
    })
}

/// s 205-70(5)(b) to (d), for a private company: it would have had an income
/// tax liability without this offset, had none in an earlier year, and that
/// liability is at least 90% of the deficit.
fn private_exception(t: &Totals, tax: Option<&IncomeTax>) -> Result<bool> {
    let Some(tax) = tax else {
        return refuse(
            "franking_account.income_tax_needed",
            "The offset reduction applies unless s 205-70(5) does, which turns on the private company's income tax: give income_tax with before_fdt_offset, offset_brought_forward and earlier_year_liability.",
        );
    };
    let floor = precise(|| Ok(t.fdt.mul(&d("0.90"))?))?;
    Ok(tax.earlier_liability == Some(false)
        && tax.before_offset.gt(&int(0))?
        && !tax.before_offset.lt(&floor)?)
}

/// The benchmark of the last franking period before the year with a
/// frankable distribution, and the franking periods from the one after it to
/// the year's first, counted (s 204-70(1)).
fn last_benchmark(value: &Arg) -> Result<Option<(Decimal, i64)>> {
    let bad = || {
        refuse(
            "franking_account.last_benchmark_invalid",
            "last_benchmark must be null, when no frankable distribution was made in an earlier franking period, or an object with percentage (a decimal string from 0 to 100, the benchmark franking percentage of the last earlier franking period in which one was made) and periods_since (a whole number from 1: the franking periods from the one after it to the income year's first, counted).",
        )
    };
    match value {
        Arg::Json(Value::Null) => Ok(None),
        Arg::Json(Value::Object(o)) if o.len() == 2 => {
            let pct = o.get("percentage").and_then(percentage);
            let since = o.get("periods_since").and_then(Value::as_i64);
            match (pct, since) {
                (Some(p), Some(n)) if (1..=1000).contains(&n) => Ok(Some((p, n))),
                _ => bad(),
            }
        }
        _ => bad(),
    }
}

fn two_places(value: &Decimal) -> Result<String> {
    Ok(precise(|| Ok(value.quantize(-2, Rounding::HalfUp)?))?.to_string())
}

/// Each franking period with its benchmark, shown to 2 places, 5 up (the
/// ATO's section C), and whether it differs from the last benchmark by more
/// than 20 percentage points for each franking period after that one, up to
/// and including it (s 204-70(1)). The disclosure does not apply where the
/// benchmark rule does not (s 204-70(2)).
fn variations(
    periods: &[Period],
    last: Option<(Decimal, i64)>,
    exempt: bool,
) -> Result<(Vec<Value>, bool)> {
    let mut out = Vec::new();
    let mut any = false;
    // The last benchmark and its period's index, counting this year's first
    // franking period as 0.
    let mut previous: Option<(Decimal, i64)> = last.map(|(p, since)| (p, -since));
    for (k, period) in (0_i64..).zip(periods) {
        let mut row = json!({
            "start": dates::format(period.start),
            "end": dates::format(period.end),
            "benchmark_percentage": Value::Null,
            "significant_variation": false,
        });
        if let Some(b) = &period.benchmark {
            row["benchmark_percentage"] = json!(two_places(b)?);
            if let Some((reference, at)) = &previous {
                let count = k - at;
                let gap = precise(|| Ok(b.sub(reference)?))?;
                let gap = if gap.lt(&int(0))? { gap.neg()? } else { gap };
                let varies = !exempt && gap.gt(&int(20 * count))?;
                row["compared_with"] = json!(two_places(reference)?);
                row["threshold_points"] = json!(20 * count);
                row["significant_variation"] = json!(varies);
                any |= varies;
            }
            previous = Some((b.clone(), k));
        }
        out.push(row);
    }
    Ok((out, any))
}

/// `franking-account`'s inputs, by name.
pub struct FrankingArgs<'a> {
    pub private_company: &'a Arg,
    pub benchmark_rule_exempt: &'a Arg,
    pub corporate_tax_rate: &'a Arg,
    pub opening_balance: &'a Arg,
    pub rd_deferred_debit_brought_forward: &'a Arg,
    pub last_benchmark: &'a Arg,
    pub events: &'a Arg,
    pub distributions: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub income_tax: Option<&'a Arg>,
    pub commissioner_determination: Option<&'a Arg>,
}

fn read_facts(a: &FrankingArgs, year: &str) -> Result<Facts> {
    let private = flag(a.private_company, "private_company")?;
    let exempt = flag(a.benchmark_rule_exempt, "benchmark_rule_exempt")?;
    if private && exempt {
        return refuse(
            "franking_account.facts_conflict",
            "A private company is not a listed public company or its 100% subsidiary, so s 203-20 cannot exempt it from the benchmark rule.",
        );
    }
    let (rate, gross_up) = rates(a.corporate_tax_rate)?;
    let end_year: i64 = year[..4].parse::<i64>().expect("income year") + 1;
    Ok(Facts {
        year_start: dates::parse(&format!("{}-07-01", end_year - 1)).expect("1 July"),
        year_end: dates::parse(&format!("{end_year}-06-30")).expect("30 June"),
        private,
        exempt,
        rate,
        gross_up,
    })
}

/// ITAA 1997 Division 205 for one income year: the account's lines and
/// balance, franking deficit tax and its offset, the benchmark rule's
/// over-franking tax and debits, the significant variation test and the
/// franking return's amounts and due date.
pub fn franking_account(a: &FrankingArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let facts = read_facts(a, &year)?;
    let opening = signed_money(a.opening_balance)?;
    let mut rd_pool = money(a.rd_deferred_debit_brought_forward)?;
    let last = last_benchmark(a.last_benchmark)?;
    let events = objects(
        a.events,
        "event",
        &EVENT_FIELDS,
        "franking_account.event_invalid",
    )?
    .into_iter()
    .enumerate()
    .map(|(i, o)| event(i + 1, o, &facts))
    .collect::<Result<Vec<_>>>()?;
    let dists = objects(
        a.distributions,
        "distribution",
        &DISTRIBUTION_FIELDS,
        "franking_account.distribution_invalid",
    )?
    .into_iter()
    .enumerate()
    .map(|(i, o)| distribution(i + 1, o, &facts))
    .collect::<Result<Vec<_>>>()?;
    let tax = income_tax(a.income_tax, facts.private)?;
    let determination = match a.commissioner_determination {
        None => false,
        Some(given) => flag(given, "commissioner_determination")?,
    };
    if let Some(n) = (1..dists.len()).find(|&i| dists[i].date < dists[i - 1].date) {
        return bad(
            "franking_account.distribution_invalid",
            "distribution",
            n + 1,
            "distributions must be in date order: the first in a franking period sets its benchmark (s 203-30).",
        );
    }
    let mut periods: Vec<Period> = periods(&facts)
        .into_iter()
        .map(|(start, end)| Period {
            start,
            end,
            benchmark: None,
        })
        .collect();
    let worked = benchmark(&dists, &facts, &mut periods)?;
    let lines = year_lines(&opening, &events, &dists, &worked, &facts, &mut rd_pool)?;
    let before_refunds = deficit(&balance(&opening, &lines)?)?;
    let mut account = Account {
        lines,
        rd_pool,
        deferred: Vec::new(),
        not_posted: Vec::new(),
    };
    after_year_end(&opening, &events, &facts, &mut account)?;
    let t = totals(&opening, &account.lines)?;
    let offset = current_offset(&opening, &t, &facts, tax.as_ref(), determination)?;
    let (rows, varies) = variations(&periods, last, facts.exempt)?;
    let oft = sum_of_worked(&worked)?;
    let parts = Parts {
        opening: &opening,
        facts: &facts,
        t: &t,
        offset: &offset,
        oft: &oft,
        varies,
        before_refunds: &before_refunds,
        tax: tax.as_ref(),
    };
    let mut out = summary(&year, &parts, &account)?;
    out["ledger"] = ledger(&opening, &account.lines)?;
    out["distributions"] = distributions_out(&dists, &worked, &periods)?;
    out["franking_periods"] = Value::Array(rows);
    out["refunds_after_year_end"] = deferred_out(&account, &parts)?;
    if facts.private && t.fdt.gt(&int(0))? && dists.iter().any(|x| x.late_statement) {
        out["warnings"]
            .as_array_mut()
            .expect("warnings")
            .push(json!("ITAA 1997 s 202-75(4): a private company may not give a distribution statement after the distribution if the franking credit it allocates, alone or with the year's others, creates or increases franking deficit tax. A distribution here has a statement given late and the year ends in deficit: check that its allocation was allowed; this command does not decide it."));
    }
    Ok(out)
}

fn sum_of_worked(worked: &[Worked]) -> Result<Decimal> {
    precise(|| {
        worked
            .iter()
            .try_fold(int(0), |t, w| Ok(t.add(&w.over_franking_tax)?))
    })
}

/// What the summary and the refund rows are worked from.
struct Parts<'a> {
    opening: &'a Decimal,
    facts: &'a Facts,
    t: &'a Totals,
    offset: &'a Offset,
    oft: &'a Decimal,
    varies: bool,
    before_refunds: &'a Decimal,
    tax: Option<&'a IncomeTax>,
}

/// The last day of the month after the year's end: the franking return's
/// and franking tax's due date (s 214-150(1); the ATO's 'How to lodge and
/// pay').
fn due_date(facts: &Facts) -> i64 {
    dates::add_months(facts.year_end + 1, 1) - 1
}

fn summary(year: &str, p: &Parts, account: &Account) -> Result<Value> {
    let t = p.t;
    let from_refunds = precise(|| Ok(t.fdt.sub(p.before_refunds)?))?;
    let payable = precise(|| Ok(t.fdt.add(p.oft)?))?;
    let mut amounts = vec![
        ("opening_balance", p.opening.clone()),
        ("credits", t.credits.clone()),
        ("debits", t.debits.clone()),
        ("closing_balance", t.closing.clone()),
        ("franking_deficit_tax", t.fdt.clone()),
        ("fdt_from_refunds_after_year_end", from_refunds),
        ("fdt_offset_reduction", p.offset.reduction.clone()),
        ("offsetable_fdt", p.offset.amount.clone()),
        ("debits_not_subject_to_reduction", t.label_k.clone()),
        ("over_franking_tax", p.oft.clone()),
        ("franking_tax_payable", payable),
        ("rd_deferred_debit_carried_forward", account.rd_pool.clone()),
    ];
    if let Some(tax) = p.tax {
        let total = precise(|| Ok(p.offset.amount.add(&tax.brought_forward)?))?;
        let used = if total.lt(&tax.before_offset)? {
            total.clone()
        } else {
            tax.before_offset.clone()
        };
        let carried = precise(|| Ok(total.sub(&used)?))?;
        amounts.extend([
            ("fdt_offset_total", total),
            ("fdt_offset_used", used),
            ("fdt_offset_carried_forward", carried),
        ]);
    }
    let required = !t.fdt.is_zero() || !p.oft.is_zero() || p.varies;
    let rates = vec![
        rate("corporate_tax_rate", &p.facts.rate),
        rate("gross_up_rate", two_places_more(&p.facts.gross_up)?),
        rate(
            "balance_code",
            if t.closing.lt(&int(0))? { "D" } else { "S" },
        ),
        rate("offset_code", p.offset.code.unwrap_or("none")),
        rate("offset_authority", p.offset.authority),
        rate("significant_variation", p.varies),
        rate("return_required", required),
        rate(
            "return_due",
            if required {
                dates::format(due_date(p.facts))
            } else {
                "none".into()
            },
        ),
    ];
    let mut out = result(KIND, year, amounts, rates)?;
    out["return_labels"] = json!({
        "balance": out["amounts"]["closing_balance"],
        "A": out["amounts"]["credits"],
        "B": out["amounts"]["franking_deficit_tax"],
        "C": out["amounts"]["offsetable_fdt"],
        "D": out["amounts"]["over_franking_tax"],
        "E": out["amounts"]["franking_tax_payable"],
        "F": out["amounts"]["fdt_from_refunds_after_year_end"],
        "K": out["amounts"]["debits_not_subject_to_reduction"],
    });
    Ok(out)
}

/// The gross-up rate to 4 places, as the ATO's example 5 shows it; the
/// arithmetic keeps it exact.
fn two_places_more(value: &Decimal) -> Result<String> {
    Ok(precise(|| Ok(value.quantize(-4, Rounding::HalfUp)?))?.to_string())
}

fn ledger(opening: &Decimal, lines: &[Line]) -> Result<Value> {
    let mut running = opening.clone();
    let mut rows = Vec::new();
    for line in lines {
        let credit = line.class == Class::Credit;
        running = precise(|| {
            Ok(if credit {
                running.add(&line.amount)?
            } else {
                running.sub(&line.amount)?
            })
        })?;
        rows.push(json!({
            "date": dates::format(line.date),
            "item": line.item,
            "credit": if credit { json!(shown(&line.amount)?) } else { Value::Null },
            "debit": if credit { Value::Null } else { json!(shown(&line.amount)?) },
            "balance": shown(&running)?,
        }));
    }
    Ok(Value::Array(rows))
}

fn distributions_out(
    dists: &[Distribution],
    worked: &[Worked],
    periods: &[Period],
) -> Result<Value> {
    let mut rows = Vec::new();
    for (dist, w) in dists.iter().zip(worked) {
        rows.push(json!({
            "date": dates::format(dist.date),
            "amount": shown(&dist.amount)?,
            "franking_credit_allocated": shown(&dist.allocated)?,
            "maximum_franking_credit": shown(&w.maximum)?,
            "franking_credit": shown(&w.credit)?,
            "franking_percentage": two_places(&w.percentage)?,
            "franking_period": dates::format(periods[w.period].start),
            "compared_with": match &w.reference { Some(r) => json!(two_places(r)?), None => Value::Null },
            "over_franking_tax": shown(&w.over_franking_tax)?,
            "under_franking_debit": shown(&w.under_franking_debit)?,
        }));
    }
    Ok(Value::Array(rows))
}

/// Each refund paid after the year's end: those s 205-50(2) takes into the
/// year, with the franking deficit tax each adds and when it is due
/// (s 214-150(4)): with a franking return still outstanding for the year,
/// that return's due date if the refund is accounted for in it, otherwise
/// 14 days after the refund was received; and those left for the next year.
fn deferred_out(account: &Account, p: &Parts) -> Result<Value> {
    let outstanding = !p.before_refunds.is_zero() || !p.oft.is_zero() || p.varies;
    let due = due_date(p.facts);
    let mut rows = Vec::new();
    for (date, added) in &account.deferred {
        let with_return = (outstanding && *date <= due).then(|| dates::format(due));
        rows.push(json!({
            "date": dates::format(*date),
            "taken_into_year": true,
            "fdt_added": shown(added)?,
            "due_with_outstanding_return": with_return,
            "due_otherwise": dates::format(date + 14),
        }));
    }
    rows.extend(account.not_posted.iter().cloned());
    Ok(Value::Array(rows))
}
