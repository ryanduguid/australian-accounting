// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `fhss`: the First Home Super Saver scheme's maximum release amount and the
//! tax on its release. From a list of contributions it counts the eligible
//! ones against the limits (TAA 1953 Sch 1 ss 138-30 and 138-35), works out the
//! releasable contributions amount and the associated earnings (s 138-40) at
//! the shortfall interest charge rate (s 280-105(2)); from a determination's
//! stated components it starts there. Either way it gives the assessable FHSS
//! released amount (ITAA 1997 s 313-20), the withholding (TAA Regulations 2017
//! reg 53A), the offset (s 313-25) and the FHSS tax if the money is neither
//! used nor recontributed (s 313-60). Provisions, the Commissioner's guidance
//! note GN 2024/1 and the SIC rate table were read on 20 November 2026.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::{Arg, Result, d, finite_between, money, precise, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "fhss";

/// The SIC annual rate, in per cent, for each quarter from 1 July 2017:
/// (calendar year, first month of the quarter, rate). The ATO's 'Shortfall
/// interest charge (SIC) rates' page, last updated 4 September 2026, read on
/// 20 November 2026. Each is the base interest rate plus 3 percentage points
/// (s 280-105(2)).
const SIC: [(i64, i64, &str); 38] = [
    (2017, 7, "4.73"),
    (2017, 10, "4.70"),
    (2018, 1, "4.72"),
    (2018, 4, "4.77"),
    (2018, 7, "4.96"),
    (2018, 10, "4.96"),
    (2019, 1, "4.94"),
    (2019, 4, "4.96"),
    (2019, 7, "4.54"),
    (2019, 10, "3.98"),
    (2020, 1, "3.91"),
    (2020, 4, "3.89"),
    (2020, 7, "3.10"),
    (2020, 10, "3.10"),
    (2021, 1, "3.02"),
    (2021, 4, "3.01"),
    (2021, 7, "3.04"),
    (2021, 10, "3.01"),
    (2022, 1, "3.04"),
    (2022, 4, "3.07"),
    (2022, 7, "4.00"),
    (2022, 10, "5.31"),
    (2023, 1, "6.06"),
    (2023, 4, "6.46"),
    (2023, 7, "6.90"),
    (2023, 10, "7.15"),
    (2024, 1, "7.38"),
    (2024, 4, "7.34"),
    (2024, 7, "7.36"),
    (2024, 10, "7.38"),
    (2025, 1, "7.42"),
    (2025, 4, "7.17"),
    (2025, 7, "6.78"),
    (2025, 10, "6.61"),
    (2026, 1, "6.65"),
    (2026, 4, "6.96"),
    (2026, 7, "7.43"),
    (2026, 10, "7.51"),
];

/// The arguments as the command passes them; see the README for each.
pub struct FhssArgs<'a> {
    pub determination_date: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub contributions: Option<&'a Arg>,
    pub determination_requested: Option<&'a Arg>,
    pub personal_deductions: Option<&'a Arg>,
    pub determination: Option<&'a Arg>,
    pub amount_requested: Option<&'a Arg>,
    pub estimated_marginal_rate: Option<&'a Arg>,
}

/// A contribution as given, or the part of one treated as concessional or
/// non-concessional.
#[derive(Clone)]
struct Piece {
    /// The contribution's number in the list, from 1.
    n: usize,
    day: i64,
    fy: i64,
    kind: &'static str,
    concessional: bool,
    amount: Decimal,
    counted: Decimal,
}

fn day(value: &Arg, name: &str) -> Result<i64> {
    match value.text().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => refuse(
            "input.date_invalid",
            format!("{name} must be a YYYY-MM-DD date from 1900 to 2100."),
        ),
    }
}

/// The year a financial year starts in: 2024 for 1 July 2024 to 30 June 2025.
fn fy_of(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    if m >= 7 { y } else { y - 1 }
}

fn fy_label(start: i64) -> String {
    format!("{start}-{:02}", (start + 1) % 100)
}

/// A decimal string field of a list item, as `money` reads one.
fn item_money(fields: &Map<String, Value>, key: &str, what: &str) -> Result<Decimal> {
    let parsed = match fields.get(key) {
        Some(Value::String(text)) if text.len() <= 64 => Decimal::parse(text).ok(),
        _ => None,
    };
    match parsed.map(|v| money(&Arg::Decimal(v))) {
        Some(Ok(v)) => Ok(v),
        _ => refuse(
            "input.amount_invalid",
            format!("{what}: {key} must be a non-negative decimal string, at most 2dp and 1e12."),
        ),
    }
}

fn only_fields(fields: &Map<String, Value>, allowed: &[&str], what: &str) -> Result<()> {
    match fields.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(key) => refuse(
            "input.field_unknown",
            format!("{what}: {key} is not an input of fhss."),
        ),
        None => Ok(()),
    }
}

const NOT_ELIGIBLE: &str = "Only voluntary contributions are eligible (TAA 1953 Sch 1 s 138-35(2)): employer_voluntary (salary sacrifice or another employer contribution that is not mandated) or personal (made by you). Leave out super guarantee and other mandated or required contributions, spouse and family contributions, government co-contributions, contribution splits, downsizer, small business CGT, structured settlement and COVID-19 re-contributions (GN 2024/1 section 1); KiwiSaver and other foreign transfers are outside this command.";

/// The contributions list: each dated from 1 July 2017 to the determination
/// day, with an amount above nil and an eligible kind.
fn read_contributions(list: &Arg, determination: i64) -> Result<Vec<Piece>> {
    let Arg::Json(Value::Array(items)) = list else {
        return refuse(
            "fhss.contributions_invalid",
            "contributions must be a list of at most 10000 objects with date, amount and kind.",
        );
    };
    if items.is_empty() || items.len() > 10_000 {
        return refuse(
            "fhss.contributions_invalid",
            "contributions must be a list of 1 to 10000 objects with date, amount and kind.",
        );
    }
    let first = dates::parse("2017-07-01").expect("date");
    let mut out = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let n = i + 1;
        let what = format!("contribution {n}");
        let Some(fields) = item.as_object() else {
            return refuse(
                "fhss.contributions_invalid",
                format!("{what} must be an object."),
            );
        };
        only_fields(fields, &["date", "amount", "kind"], &what)?;
        let Some(day) = fields
            .get("date")
            .and_then(Value::as_str)
            .and_then(dates::parse)
        else {
            return refuse(
                "input.date_invalid",
                format!("{what}: date must be a YYYY-MM-DD date."),
            );
        };
        if day < first || day > determination {
            return refuse(
                "fhss.contribution_out_of_period",
                format!(
                    "{what}: only contributions made from 1 July 2017 (s 138-30(1)) to the determination day can count in it."
                ),
            );
        }
        let amount = item_money(fields, "amount", &what)?;
        if amount.is_zero() {
            return refuse(
                "input.amount_invalid",
                format!("{what}: amount must be above nil."),
            );
        }
        let kind = match fields.get("kind").and_then(Value::as_str) {
            Some("employer_voluntary") => "employer_voluntary",
            Some("personal") => "personal",
            _ => {
                return refuse(
                    "fhss.contribution_kind_invalid",
                    format!("{what}: kind must be employer_voluntary or personal. {NOT_ELIGIBLE}"),
                );
            }
        };
        out.push(Piece {
            n,
            day,
            fy: fy_of(day),
            kind,
            concessional: kind == "employer_voluntary",
            amount,
            counted: Decimal::from_int(0),
        });
    }
    Ok(out)
}

/// The personal contributions deducted for each financial year (ITAA 1997
/// s 290-150), which makes them concessional.
fn read_deductions(list: Option<&Arg>) -> Result<BTreeMap<i64, Decimal>> {
    let mut out = BTreeMap::new();
    let Some(list) = list else {
        return Ok(out);
    };
    let Arg::Json(Value::Array(items)) = list else {
        return refuse(
            "fhss.deductions_invalid",
            "personal_deductions must be a list of objects with financial_year and amount.",
        );
    };
    for (i, item) in items.iter().enumerate() {
        let what = format!("personal deduction {}", i + 1);
        let Some(fields) = item.as_object() else {
            return refuse(
                "fhss.deductions_invalid",
                format!("{what} must be an object."),
            );
        };
        only_fields(fields, &["financial_year", "amount"], &what)?;
        let fy = fields
            .get("financial_year")
            .and_then(Value::as_str)
            .and_then(|label| {
                let start: i64 = label.get(..4)?.parse().ok()?;
                (fy_label(start) == label).then_some(start)
            });
        let Some(fy) = fy else {
            return refuse(
                "fhss.deductions_invalid",
                format!("{what}: financial_year must be a year such as \"2024-25\"."),
            );
        };
        if out
            .insert(fy, item_money(fields, "amount", &what)?)
            .is_some()
        {
            return refuse(
                "fhss.deductions_invalid",
                format!("{what}: {} is given twice.", fy_label(fy)),
            );
        }
    }
    Ok(out)
}

/// Each year's deducted personal contributions are the latest of its
/// personal contributions, so the non-concessional are made first
/// (s 138-30(3)(b) and its example); a contribution the deduction reaches
/// part of is split, the non-concessional part first.
fn split_personal(pieces: Vec<Piece>, deductions: &BTreeMap<i64, Decimal>) -> Result<Vec<Piece>> {
    let mut by_year: BTreeMap<i64, Decimal> = BTreeMap::new();
    for p in pieces.iter().filter(|p| p.kind == "personal") {
        let total = by_year.entry(p.fy).or_insert_with(|| Decimal::from_int(0));
        *total = total.add(&p.amount)?;
    }
    let mut left = BTreeMap::new();
    for (fy, deducted) in deductions {
        let made = by_year
            .get(fy)
            .cloned()
            .unwrap_or_else(|| Decimal::from_int(0));
        if deducted.gt(&made)? {
            return refuse(
                "fhss.deductions_invalid",
                format!(
                    "The deduction for {} is more than the personal contributions given for that year.",
                    fy_label(*fy)
                ),
            );
        }
        left.insert(*fy, made.sub(deducted)?);
    }
    let mut out = Vec::with_capacity(pieces.len());
    for mut p in pieces {
        if p.kind != "personal" {
            out.push(p);
            continue;
        }
        let Some(room) = left.get_mut(&p.fy) else {
            out.push(p);
            continue;
        };
        if p.amount.le(room)? {
            *room = room.sub(&p.amount)?;
            out.push(p);
        } else if room.is_zero() {
            p.concessional = true;
            out.push(p);
        } else {
            let deducted = p.amount.sub(room)?;
            let free = std::mem::replace(room, Decimal::from_int(0));
            out.push(Piece {
                amount: free,
                ..p.clone()
            });
            p.amount = deducted;
            p.concessional = true;
            out.push(p);
        }
    }
    Ok(out)
}

/// s 138-30(2) and (3): earliest first; a non-concessional contribution made
/// at the same time as a concessional one first; and in a year where you
/// personally made both kinds, the year's non-concessional contributions
/// first.
fn order(pieces: &mut [Piece], deductions: &BTreeMap<i64, Decimal>) {
    let both: Vec<i64> = deductions
        .keys()
        .copied()
        .filter(|fy| {
            pieces
                .iter()
                .any(|p| p.fy == *fy && p.kind == "personal" && !p.concessional)
                && pieces
                    .iter()
                    .any(|p| p.fy == *fy && p.kind == "personal" && p.concessional)
        })
        .collect();
    pieces.sort_by_key(|p| {
        let year_first = both.contains(&p.fy) && !p.concessional;
        (p.fy, !year_first, p.day, p.concessional, p.n)
    });
}

/// s 138-35(1): at most $15,000 of a year's contributions and $50,000 in all
/// ($30,000 for a determination requested before 1 July 2022: GN 2024/1
/// section 3), tested on the contributions in full.
fn count(pieces: &mut [Piece], total_limit: &Decimal) -> Result<()> {
    let year_limit = d("15000");
    let mut total = Decimal::from_int(0);
    let mut years: BTreeMap<i64, Decimal> = BTreeMap::new();
    for p in pieces.iter_mut() {
        let used = years.entry(p.fy).or_insert_with(|| Decimal::from_int(0));
        let year_room = year_limit.sub(used)?;
        let total_room = total_limit.sub(&total)?;
        let room = if year_room.lt(&total_room)? {
            year_room
        } else {
            total_room
        };
        p.counted = if p.amount.lt(&room)? {
            p.amount.clone()
        } else {
            room
        };
        *used = used.add(&p.counted)?;
        total = total.add(&p.counted)?;
    }
    Ok(())
}

/// The SIC rate for `day`: the quarter's annual rate over the days in the
/// calendar year (s 280-105(2)).
fn sic_rate(day: i64) -> Decimal {
    let (y, m, _) = dates::to_civil(day);
    let quarter = (m - 1) / 3 * 3 + 1;
    let (_, _, annual) = SIC
        .iter()
        .find(|(yy, mm, _)| *yy == y && *mm == quarter)
        .expect("a day inside the table");
    let days = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
        366
    } else {
        365
    };
    d(annual)
        .div(&Decimal::from_int(100 * days))
        .expect("finite rate")
}

/// s 138-40(2) and (3): the sum over each day of the period of the SIC rate
/// times the amount plus the earlier daily amounts, unrounded. Each daily
/// amount is proportional to the amount, so the sum for $1 from a start day,
/// worked out once, serves every contribution whose period starts then.
fn earnings(base: &Decimal, start: i64, end: i64) -> Result<Decimal> {
    let mut total = Decimal::from_int(0);
    for day in start..=end {
        total = total.add(&sic_rate(day).mul(&base.add(&total)?)?)?;
    }
    Ok(total)
}

struct Components {
    concessional: Decimal,
    non_concessional: Decimal,
    earnings: Decimal,
}

fn from_contributions(args: &FhssArgs, determined: i64) -> Result<(Components, Value)> {
    let Some(requested) = args.determination_requested else {
        return refuse(
            "input.missing_field",
            "determination_requested is required with contributions: it sets the $50,000 or $30,000 limit.",
        );
    };
    let requested = day(requested, "determination_requested")?;
    if requested > determined || requested < dates::parse("2018-07-01").expect("date") {
        return refuse(
            "fhss.determination_requested_invalid",
            "determination_requested must be from 1 July 2018, when requests opened, to the determination day.",
        );
    }
    let total_limit = if requested < dates::parse("2022-07-01").expect("date") {
        d("30000")
    } else {
        d("50000")
    };
    let deductions = read_deductions(args.personal_deductions)?;
    let mut pieces = read_contributions(args.contributions.expect("checked"), determined)?;
    // In date order, so a year's deduction reaches its latest contributions.
    pieces.sort_by_key(|p| (p.day, p.n));
    let mut pieces = split_personal(pieces, &deductions)?;
    order(&mut pieces, &deductions);
    count(&mut pieces, &total_limit)?;
    let zero = Decimal::from_int(0);
    let (mut cc, mut ncc) = (zero.clone(), zero.clone());
    let mut bases: BTreeMap<usize, (i64, Decimal)> = BTreeMap::new();
    let start_2017 = dates::parse("2017-07-01").expect("date");
    for p in &pieces {
        let base = if p.concessional {
            cc = cc.add(&p.counted)?;
            d("0.85").mul(&p.counted)?
        } else {
            ncc = ncc.add(&p.counted)?;
            p.counted.clone()
        };
        let (y, m, _) = dates::to_civil(p.day);
        let start = if p.fy == 2017 {
            start_2017
        } else {
            dates::parse(&format!("{y:04}-{m:02}-01")).expect("date")
        };
        let entry = bases.entry(p.n).or_insert((start, zero.clone()));
        entry.1 = entry.1.add(&base)?;
    }
    let mut earned = BTreeMap::new();
    let mut total_earnings = zero.clone();
    let mut per_dollar: BTreeMap<i64, Decimal> = BTreeMap::new();
    for (n, (start, base)) in &bases {
        let e = if base.is_zero() {
            zero.clone()
        } else {
            if !per_dollar.contains_key(start) {
                let unit = earnings(&Decimal::from_int(1), *start, determined)?;
                per_dollar.insert(*start, unit);
            }
            // Rounded down to the dollar for each contribution (s 138-40(2)).
            base.mul(&per_dollar[start])?.quantize(0, Rounding::Down)?
        };
        total_earnings = total_earnings.add(&e)?;
        earned.insert(*n, (*start, e));
    }
    let rows: Vec<Value> = pieces
        .iter()
        .map(|p| {
            let (start, e) = &earned[&p.n];
            json!({
                "contribution": p.n,
                "date": dates::format(p.day),
                "financial_year": fy_label(p.fy),
                "kind": p.kind,
                "treated_as": if p.concessional { "concessional" } else { "non_concessional" },
                "amount": p.amount.to_string(),
                "counted": p.counted.to_string(),
                "earnings_from": dates::format(*start),
                "contribution_earnings": e.to_string(),
            })
        })
        .collect();
    let mut years: BTreeMap<i64, (Decimal, Decimal)> = BTreeMap::new();
    for p in &pieces {
        let entry = years.entry(p.fy).or_insert((zero.clone(), zero.clone()));
        if p.concessional {
            entry.0 = entry.0.add(&p.counted)?;
        } else {
            entry.1 = entry.1.add(&p.counted)?;
        }
    }
    let years: Vec<Value> = years
        .iter()
        .map(|(fy, (c, n))| {
            json!({"financial_year": fy_label(*fy), "concessional_counted": c.to_string(), "non_concessional_counted": n.to_string()})
        })
        .collect();
    let detail = json!({
        "total_limit": total_limit.to_string(),
        "counted_concessional_in_full": cc.to_string(),
        "years": years,
        "contributions": rows,
    });
    let components = Components {
        concessional: d("0.85").mul(&cc)?,
        non_concessional: ncc,
        earnings: total_earnings,
    };
    Ok((components, detail))
}

fn from_determination(given: &Arg) -> Result<Components> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "fhss.determination_invalid",
            "determination must be an object with concessional, non_concessional and associated_earnings.",
        );
    };
    let keys = ["concessional", "non_concessional", "associated_earnings"];
    only_fields(fields, &keys, "determination")?;
    let [concessional, non_concessional, earnings] =
        keys.map(|key| item_money(fields, key, "determination"));
    Ok(Components {
        concessional: concessional?,
        non_concessional: non_concessional?,
        earnings: earnings?,
    })
}

const ESTIMATE: &str = "estimate: associated_earnings follows s 138-40 on the counted amount (85% of a concessional contribution, as s 138-30(1)(b) counts it) at the SIC rates the ATO publishes, from the start of the contribution's month to the determination day, rounded down for each contribution. The Commissioner's determination is the figure that counts: GN 2024/1 example 6 (Jill) states $5,190, which neither this reading nor the others tried reproduces, so this is an estimate.";
const STATED: &str = "stated: the components are the determination's own, as given.";

pub fn fhss(args: &FhssArgs) -> Result<Value> {
    let period = scope(args.scope_confirmed, args.year, KIND)?;
    let determined = day(args.determination_date, "determination_date")?;
    if fy_label(fy_of(determined)) != period {
        return refuse(
            "fhss.year_mismatch",
            format!("determination_date must fall in year, the financial year {period}."),
        );
    }
    // The supported years start with 2018-19, when determinations began.
    if determined > dates::parse("2026-12-31").expect("date") {
        return refuse(
            "fhss.rate_unpublished",
            "determination_date must be no later than 31 December 2026, the last quarter whose SIC rate the ATO had published when this was built.",
        );
    }
    let (components, detail, basis) = match (args.contributions, args.determination) {
        (Some(_), None) => {
            let (c, detail) = precise(|| from_contributions(args, determined))?;
            (c, Some(detail), ESTIMATE)
        }
        (None, Some(given)) => {
            for (name, extra) in [
                ("determination_requested", args.determination_requested),
                ("personal_deductions", args.personal_deductions),
            ] {
                if extra.is_some() {
                    return refuse(
                        "input.field_unknown",
                        format!("{name} is an input only with contributions."),
                    );
                }
            }
            (from_determination(given)?, None, STATED)
        }
        _ => {
            return refuse(
                "fhss.basis_invalid",
                "Give either contributions (to estimate the determination) or determination (its stated components), not both or neither.",
            );
        }
    };
    precise(|| release(args, &period, components, detail, basis))
}

fn release(
    args: &FhssArgs,
    period: &str,
    c: Components,
    detail: Option<Value>,
    basis: &str,
) -> Result<Value> {
    let zero = Decimal::from_int(0);
    let maximum = c.concessional.add(&c.non_concessional)?.add(&c.earnings)?;
    let requested = match args.amount_requested {
        Some(arg) => money(arg)?,
        None => maximum.clone(),
    };
    if requested.gt(&maximum)? {
        return refuse(
            "fhss.request_exceeds_maximum",
            "amount_requested is more than the FHSS maximum release amount (TAA 1953 Sch 1 s 131-10(1), table item 4).",
        );
    }
    // s 313-20: the concessional and earnings components, less any shortfall
    // of the amount released below the maximum, not below nil.
    let taxed = c.concessional.add(&c.earnings)?;
    let shortfall = maximum.sub(&requested)?;
    let assessable = taxed.sub(&shortfall)?;
    let assessable = if assessable.lt(&zero)? {
        zero.clone()
    } else {
        assessable
    };
    let withholding_rate = match args.estimated_marginal_rate {
        None => d("0.17"),
        Some(arg) => {
            let Some(marginal) = finite_between(arg, "0", "0.45")? else {
                return refuse(
                    "input.fraction_invalid",
                    "estimated_marginal_rate must be a finite Decimal fraction from 0 to 0.45.",
                );
            };
            let r = marginal.add(&d("0.02"))?.sub(&d("0.30"))?;
            if r.lt(&zero)? { zero.clone() } else { r }
        }
    };
    let withheld = withholding_rate
        .mul(&assessable)?
        .quantize(0, Rounding::Down)?;
    let paid = requested.sub(&withheld)?;
    let offset = d("0.30").mul(&assessable)?;
    let fhss_tax = d("0.20").mul(&assessable)?;
    let recontribute = assessable.sub(&withheld)?;
    let rates = vec![
        rate("annual_limit", "15000"),
        rate("concessional_share", "0.85"),
        rate("withholding_rate", withholding_rate.to_string()),
        rate("fhss_offset_rate", "0.30"),
        rate("fhss_tax_rate", "0.20"),
    ];
    let mut out = result(
        KIND,
        period,
        vec![
            ("concessional_component", c.concessional),
            ("non_concessional_component", c.non_concessional),
            ("associated_earnings", c.earnings),
            ("maximum_release_amount", maximum),
            ("amount_requested", requested),
            ("assessable_fhss_released_amount", assessable),
            ("tax_withheld", withheld),
            ("paid_to_you", paid),
            ("fhss_tax_offset", offset),
            ("fhss_tax_if_not_used", fhss_tax),
            ("recontribution_required", recontribute),
        ],
        rates,
    )?;
    out["associated_earnings_basis"] = json!(basis.split(':').next().expect("a basis"));
    out["basis"] = json!(basis);
    if let Some(Value::Object(fields)) = detail {
        for (key, value) in fields {
            out[key] = value;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One quarter's compounding has a closed form: base x ((1 + r)^n - 1).
    #[test]
    fn earnings_compound_daily() {
        let start = dates::parse("2025-07-01").expect("date");
        let end = dates::parse("2025-09-30").expect("date");
        let e = precise(|| earnings(&d("10000"), start, end)).expect("earnings");
        // 10000 x ((1 + 0.0678 / 365)^92 - 1) = 172.3... by hand.
        assert_eq!(e.quantize(0, Rounding::Down).unwrap().to_string(), "172");
    }
}
