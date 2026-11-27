// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `alcohol_excise`: excise duty on beer, spirits and other excisable
//! beverages entered for home consumption from 1 July 2022, line by line at
//! the rate in force on the day, with the remission for alcohol manufacturers
//! under its financial year cap and the refund of duty paid where that
//! remission was available. The rates are `data/alcohol-excise-rates.json`,
//! each period with the Commissioner's notice of substituted rates under
//! Excise Tariff Act 1921 s 6A(9). The citations are to the Excise Tariff
//! Act 1921 compilation 58 (1 July 2026), the Excise Act 1901 compilation 69
//! (27 August 2026), the Excise Regulation 2015 compilations 4, 5 and 6, the
//! ATO's Excise guidelines for the alcohol industry chapter 6 and its pages
//! QC63603, QC63605 and QC65935, all read on 22 November 2026.

use serde_json::{Value, json};

mod input;
mod rates;
use input::{Line, parse_line};
use rates::{Period, SUBITEMS, period_json, period_on, rate_of, rates};

use super::{Arg, Result, date_arg, flag, int, money, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "alcohol_excise";
const MAX_LINES: usize = 1000;

const LAL_BEER: &str = "Excise Tariff Act 1921 Schedule item 1: beer's rate is per litre of alcohol 'calculated on that alcohol content by which the percentage by volume of alcohol of the goods exceeds 1.15', so litres of alcohol = volume x (strength - 1.15) / 100 (the ATO's QC63603; Excise guidelines for the alcohol industry 6.3.5)";
const LAL_OTHER: &str = "Excise Tariff Act 1921 Schedule items 2 and 3: litres of alcohol = volume x strength / 100 (the ATO's QC63603; Excise guidelines for the alcohol industry 6.3.5)";
const LABELLED: &str = "Excise Act 1901 s 77FA(1): where the label states a higher percentage by volume of alcohol than the actual one, duty is charged on the labelled percentage; under s 77FA(2) goods a manufacturer enters unlabelled are treated as labelled with the strength of the same kind of beverage it enters labelled";
const RATE: &str = "Excise Tariff Act 1921 s 6A(1), (8) and (9): the rate is the substituted rate the Commissioner gave notice of, in force on the day the goods are entered for home consumption; under a periodic settlement permission the day of delivery, otherwise the day of payment (Excise guidelines for the alcohol industry 6.3.5, Examples 6I and 6J)";
const FREEZE: &str = "Excise Tariff Act 1921 s 6L: subitems 1.2, 1.6 and 1.11 are not indexed on 1 August 2025, 1 February 2026, 1 August 2026 or 1 February 2027";
const RETURN_LINE: &str = "Excise guidelines for the alcohol industry 6.3.5, Examples 6K to 6O; the ATO's QC63603: goods of one subitem at one rate are one return line; their litres of alcohol may be truncated to one decimal place, and the duty (litres x rate) is truncated to the cent; goods at a different rate are a separate line";
const REMISSION: &str = "Excise Regulation 2015 s 8 and Sch 1 cl 2(1) item 10 (Excise Act 1901 s 78(1)(b)): duty on an alcoholic beverage manufactured and entered for home consumption by an alcohol manufacturer is remitted where it is payable by the manufacturer (a), the manufacturer is legally and economically independent of any other that has received the remission in the year (b), it fermented or distilled at least 70% by volume of the alcohol content (d), and for a distilled beverage after its first 2 financial years it solely owns a still of at least 5 litres installed at the start of the year and used in it (e); a line marked remission states (a), (b), (d) and (e), and the cap (c) is applied here";
const CAP: &str = "Excise Regulation 2015 Sch 1 cl 2(1) item 10(c)(i) and s 64: the duty remitted, rebated or refunded under item 10 (or refunded under cl 2(2)) for goods entered in a financial year is at most $400,000 for goods entered from 1 July 2026, and $350,000 for goods entered from 1 July 2021 to 30 June 2026 (compilations 4 and 5; the ATO's QC65935)";
const PART_YEAR: &str = "Excise Regulation 2015 Sch 1 cl 2(1) item 10(c)(ii) and cl 2(3): for a manufacturer that first becomes an alcohol manufacturer after the start of the financial year, the cap x eligible days / 365, the eligible days running from that day to the end of the financial year; truncated to the cent here (the ATO's QC65935 example: $400,000 x 244 / 365 = $267,397)";
const REFUND: &str = "Excise Regulation 2015 Sch 1 cl 1(1) item 3 and cl 2(2): duty paid because of manifest error of fact or patent misconception of the law on goods for which the remission was available may be refunded on application (refund code ERRO), and the refund counts towards the same cap (the ATO's QC65935); duty_paid states that ground";

/// The facts as the command passes them; see the README for each.
pub struct AlcoholExciseArgs<'a> {
    pub lines: &'a Arg,
    pub remission_used_this_year: Option<&'a Arg>,
    pub manufacturer_from: Option<&'a Arg>,
    pub duty_paid: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One excise return line: the goods of one subitem at one rate, with or
/// without the remission, over every rate period that held that rate.
struct ReturnLine {
    periods: Vec<&'static Period>,
    subitem: &'static str,
    remission: bool,
    lal: Decimal,
    lines: Vec<usize>,
}

/// An exact value without trailing zeros after the point.
fn shown(value: &Decimal) -> String {
    let text = value.fixed_format();
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text
    }
}

/// The financial year's first and last days for `year`, `YYYY-YY`.
fn year_days(year: &str) -> (i64, i64) {
    let first: i64 = year[..4].parse().expect("financial year");
    let day = |y: i64, md: &str| dates::parse(&format!("{y}-{md}")).expect("date");
    (day(first, "07-01"), day(first + 1, "06-30"))
}

/// A line's litres of alcohol and its working.
fn work_line(line: &Line, n: usize) -> Result<(Value, Decimal)> {
    let beer = line.product == "beer";
    let alcohol = if beer {
        line.strength.sub(&super::d("1.15"))?
    } else {
        line.strength.clone()
    };
    let lal = line.volume.mul(&alcohol)?.div(&int(100))?;
    let mut authority = vec![if beer { LAL_BEER } else { LAL_OTHER }];
    if line.labelled {
        authority.push(LABELLED);
    }
    let out = json!({
        "line": n,
        "date": dates::format(line.date),
        "product": line.product,
        "subitem": line.subitem,
        "volume_litres": shown(&line.volume),
        "strength_percent": shown(&line.strength),
        "strength_basis": if line.labelled { "labelled" } else { "stated" },
        "litres_of_alcohol": shown(&lal),
        "remission": line.remission,
        "authority": authority,
    });
    Ok((out, lal))
}

/// Add line `n` to its return line, opening one where none matches. Goods
/// of one subitem stay on one line across a rate period whose notice left
/// their rate unchanged, as the s 6L freeze does (Excise guidelines 6.3.5:
/// separate lines only where the rate changes).
fn file_line(groups: &mut Vec<ReturnLine>, line: &Line, n: usize, lal: Decimal) -> Result<()> {
    let period = period_on(line.date);
    let rate = rate_of(period, line.subitem);
    let found = groups.iter_mut().find(|g| {
        g.subitem == line.subitem
            && g.remission == line.remission
            && rate_of(g.periods[0], g.subitem).to_string() == rate.to_string()
    });
    match found {
        Some(g) => {
            g.lal = g.lal.add(&lal)?;
            g.lines.push(n);
            if !g.periods.iter().any(|p| p.from == period.from) {
                g.periods.push(period);
                g.periods.sort_by_key(|p| p.from);
            }
        }
        None => groups.push(ReturnLine {
            periods: vec![period],
            subitem: line.subitem,
            remission: line.remission,
            lal,
            lines: vec![n],
        }),
    }
    Ok(())
}

/// A return line's working and its duty.
fn work_return_line(g: &ReturnLine) -> Result<(Value, Decimal)> {
    let rate_per_litre = rate_of(g.periods[0], g.subitem);
    let truncated = g.lal.quantize(-1, Rounding::Down)?;
    let duty = truncated
        .mul(rate_per_litre)?
        .quantize(-2, Rounding::Down)?;
    let mut authority = vec![RATE, RETURN_LINE];
    let frozen = dates::parse("2025-08-01").expect("date");
    if matches!(g.subitem, "1.2" | "1.6" | "1.11") && g.periods.iter().any(|p| p.from >= frozen) {
        authority.push(FREEZE);
    }
    let out = json!({
        "subitem": g.subitem,
        "rate_periods": g.periods.iter().map(|p| period_json(p)).collect::<Vec<_>>(),
        "rate_per_litre_of_alcohol": rate_per_litre.to_string(),
        "litres_of_alcohol": shown(&g.lal),
        "litres_of_alcohol_truncated": truncated.fixed(1, false),
        "duty": duty.to_string(),
        "remission": g.remission,
        "lines": g.lines,
        "authority": authority,
    });
    Ok((out, duty))
}

pub fn alcohol_excise(args: &AlcoholExciseArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let bounds = year_days(&year);
    let lines = read_lines(args.lines, bounds, &year)?;
    let (mut worked, mut groups) = (Vec::new(), Vec::new());
    for (i, line) in lines.iter().enumerate() {
        let (out, lal) = work_line(line, i + 1)?;
        file_line(&mut groups, line, i + 1, lal)?;
        worked.push(out);
    }
    groups.sort_by_key(|g| {
        let order = SUBITEMS.iter().position(|s| *s == g.subitem);
        (g.periods[0].from, order, g.remission)
    });
    let (mut returned, mut duty, mut eligible) = (Vec::new(), int(0), int(0));
    for g in &groups {
        let (out, amount) = work_return_line(g)?;
        duty = duty.add(&amount)?;
        if g.remission {
            eligible = eligible.add(&amount)?;
        }
        returned.push(out);
    }
    let remission = remission_facts(args, &lines, bounds, &year)?;
    let mut out = match &remission {
        Some(facts) => with_remission(&year, &duty, &eligible, facts)?,
        None => result(
            KIND,
            &year,
            vec![("excise_duty", duty.clone()), ("net_duty", duty)],
            held_rates(),
        )?,
    };
    out["lines"] = Value::Array(worked);
    out["return_lines"] = Value::Array(returned);
    add_warnings(&mut out, &lines, remission.is_some());
    Ok(out)
}

fn held_rates() -> Vec<(String, String)> {
    let last = rates().periods.last().expect("a period").to;
    vec![
        rate("rates_read", &rates().read),
        rate("rates_from", "2022-07-01"),
        rate("rates_to", dates::format(last)),
    ]
}

/// The checked lines, each inside the financial year and on or before the
/// last day a rate is held for.
fn read_lines(given: &Arg, bounds: (i64, i64), year: &str) -> Result<Vec<Line>> {
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            "alcohol_excise.lines_invalid",
            "lines must be a list of line objects.",
        );
    };
    if items.is_empty() || items.len() > MAX_LINES {
        return refuse(
            "alcohol_excise.lines_invalid",
            format!("lines must hold from 1 to {MAX_LINES} lines."),
        );
    }
    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, v)| parse_line(v, i + 1))
        .collect::<Result<_>>()?;
    let last = rates().periods.last().expect("a period").to;
    for (i, line) in lines.iter().enumerate() {
        if line.date < bounds.0 || line.date > bounds.1 {
            return refuse(
                "alcohol_excise.date_outside_year",
                format!(
                    "line {}: date is outside the financial year {year}; run each financial year apart, as the remission cap is yearly.",
                    i + 1
                ),
            );
        }
        if line.date > last {
            return refuse(
                "alcohol_excise.rate_unpublished",
                format!(
                    "line {}: no rate is held for goods entered after {}; the rates are replaced on the February 2027 indexation day (Excise Tariff Act 1921 s 6A) and had not been published when this was checked on {}.",
                    i + 1,
                    dates::format(last),
                    rates().read
                ),
            );
        }
    }
    Ok(lines)
}

/// The remission inputs once checked: the cap, the cap left and whether the
/// duty was already paid.
struct RemissionFacts {
    cap: Decimal,
    part_year: Option<(i64, i64)>,
    used: Decimal,
    duty_paid: bool,
}

/// Required where a line is marked `remission`, and not inputs otherwise.
fn remission_facts(
    args: &AlcoholExciseArgs,
    lines: &[Line],
    bounds: (i64, i64),
    year: &str,
) -> Result<Option<RemissionFacts>> {
    let first = lines.iter().filter(|l| l.remission).map(|l| l.date).min();
    let Some(first) = first else {
        if args.remission_used_this_year.is_some()
            || args.manufacturer_from.is_some()
            || args.duty_paid.is_some()
        {
            return refuse(
                "input.field_unknown",
                "remission_used_this_year, manufacturer_from and duty_paid are inputs only with a line marked remission",
            );
        }
        return Ok(None);
    };
    let Some(used) = args.remission_used_this_year else {
        return refuse(
            "input.missing_field",
            "remission_used_this_year is required with a line marked remission",
        );
    };
    let full = int(if year >= "2026-27" { 400_000 } else { 350_000 });
    let (cap, part_year) = match args.manufacturer_from {
        Some(given) => part_year_cap(given, &full, bounds, first)?,
        None => (full, None),
    };
    Ok(Some(RemissionFacts {
        cap,
        part_year,
        used: money(used)?,
        duty_paid: match args.duty_paid {
            Some(given) => flag(given, "duty_paid")?,
            None => false,
        },
    }))
}

/// The cap for a manufacturer from `given`: the whole cap if it was one at
/// the start of the financial year, else cap x eligible days / 365.
fn part_year_cap(
    given: &Arg,
    full: &Decimal,
    bounds: (i64, i64),
    first: i64,
) -> Result<(Decimal, Option<(i64, i64)>)> {
    let from = date_arg(given, "manufacturer_from")?;
    if from > first {
        return refuse(
            "alcohol_excise.fact_conflict",
            format!(
                "manufacturer_from ({}) is after a line marked remission ({}): the remission is for goods an alcohol manufacturer entered.",
                dates::format(from),
                dates::format(first)
            ),
        );
    }
    if from <= bounds.0 {
        return Ok((full.clone(), None));
    }
    let days = bounds.1 - from + 1;
    let cap = full
        .mul(&int(days))?
        .div(&int(365))?
        .quantize(-2, Rounding::Down)?;
    Ok((cap, Some((from, days))))
}

/// The amounts and remission working where a line is marked `remission`.
fn with_remission(
    year: &str,
    duty: &Decimal,
    eligible: &Decimal,
    facts: &RemissionFacts,
) -> Result<Value> {
    let left = if facts.used.lt(&facts.cap)? {
        facts.cap.sub(&facts.used)?
    } else {
        int(0)
    };
    let applied = if eligible.lt(&left)? {
        eligible.clone()
    } else {
        left.clone()
    };
    let (remitted, refunded) = if facts.duty_paid {
        (int(0), applied.clone())
    } else {
        (applied.clone(), int(0))
    };
    let mut rates_held = held_rates();
    rates_held.push(rate("remission_cap", facts.cap.to_string()));
    rates_held.push(rate("remission_cap_left", left.to_string()));
    let out = result(
        KIND,
        year,
        vec![
            ("excise_duty", duty.clone()),
            ("remission_eligible_duty", eligible.clone()),
            ("remission", remitted),
            ("refund", refunded),
            ("over_cap", eligible.sub(&applied)?),
            ("net_duty", duty.sub(&applied)?),
        ],
        rates_held,
    );
    let mut out = out?;
    let mut authority = vec![REMISSION, CAP];
    if let Some((from, days)) = facts.part_year {
        authority.push(PART_YEAR);
        out["part_year_cap"] =
            json!({"manufacturer_from": dates::format(from), "eligible_days": days});
    }
    if facts.duty_paid {
        authority.push(REFUND);
    }
    out["remission_authority"] = json!(authority);
    Ok(out)
}

/// The read date and remission statement warnings.
fn add_warnings(out: &mut Value, lines: &[Line], remission: bool) {
    let read = dates::parse(&rates().read).expect("read date");
    let refund = out["amounts"]["refund"]
        .as_str()
        .is_some_and(|r| r != "0.00");
    let warnings = out["warnings"].as_array_mut().expect("warnings");
    if lines.iter().any(|l| l.date > read) {
        warnings.push(json!(format!(
            "Some goods are entered after {}, when the rates were last read: check the ATO's 'Excise duty rates for alcohol' page for a change since.",
            rates().read
        )));
    }
    if remission {
        warnings.push(json!(
            "A line marked remission states that the manufacturer meets Excise Regulation 2015 Sch 1 cl 2(1) item 10 (a), (b), (d) and (e) for those goods: independence, 70% fermented or distilled, and the still ownership test. remission_used_this_year must include refunds under cl 2(2) in the same year. Without manufacturer_from the manufacturer is taken to be one at the start of the year (item 10(c)(i))."
        ));
    }
    if refund {
        warnings.push(json!(
            "duty_paid states that the duty was paid because of manifest error of fact or patent misconception of the law (Excise Regulation 2015 Sch 1 cl 1(1) item 3), as where the remission was available but not applied (the ATO's QC65935); the refund is made only on application, under refund code ERRO."
        ));
    }
}

#[cfg(test)]
mod tests;
