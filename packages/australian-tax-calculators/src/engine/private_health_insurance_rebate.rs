// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `private_health_insurance_rebate`: one person's private health insurance
//! tier for an income year from 2020-21 to 2026-27, the rebate percentage for
//! each line of the private health insurance statement, the share of the PHII
//! benefit, and at tax time the refundable private health insurance tax offset
//! (ITAA 1997 Subdivision 61-G) and the excess premium reduction to repay
//! (Private Health Insurance Act 2007 s 282-18). The citations are to the PHI
//! Act compilation 42 (19 September 2026) and ITAA 1997 compilation 268
//! (1 October 2026), from the Federal Register of Legislation, and the ATO's
//! rebate rates pages for 2020-21 to 2026-27 and its statement page, all read
//! on 22 November 2026.

use serde_json::{Map, Value, json};

use super::{
    Arg, Result, SurchargeFamily, cents, d, dependent_children, int, lookup, money, not_taken,
    optional_flag, precise, rate, refuse, result, scope,
};
use crate::decimal::Decimal;
use crate::metadata as meta;

const KIND: &str = "private_health_insurance_rebate";
const MAX_LINES: usize = 100;
const LINE_FIELDS: [&str; 3] = ["premiums_eligible", "rebate_received", "benefit_code"];
const TIERS: [&str; 4] = ["base", "tier_1", "tier_2", "tier_3"];

const SINGLES: &str = "PHI Act s 22-30(4) table and s 22-35: a person not married on 30 June who maintained no dependent person who is their child or dependent sibling in the year is tested on their own income for surcharge purposes against the singles thresholds, as the ATO publishes them for the year";
const FAMILIES: &str = "PHI Act s 22-30(1) and (2) and s 22-40: a person married on 30 June (A New Tax System (Medicare Levy Surcharge - Fringe Benefits) Act 1999; the ATO counts a spouse who died in the year), whose spouse's income for surcharge purposes is added, or who on any day in the year contributed in a substantial way to the maintenance of a dependent person (PHI Act Schedule 1) who is their child or dependent sibling, is tested against the family thresholds: double the singles thresholds, each raised by $1,500 for every dependant who is a child (Fringe Benefits Act meaning) after the first (s 22-40(4))";
const TIER: &str = "PHI Act s 22-30 tables: a tier 1, 2 or 3 earner is one whose income for surcharge purposes exceeds the tier 1, 2 or 3 threshold; income that does not exceed the tier 1 threshold is the base tier";
const PERCENTAGE: &str = "PHI Act s 22-15(1) to (5E): 30%, 35% or 40% by the oldest insured person's age band, less 10 or 20 percentage points for a tier 1 or 2 earner and nil for tier 3, each adjusted from 1 April each year and applied to premiums paid in that adjustment year, as the ATO publishes them; the statement's benefit code gives the band and whether the premiums were paid before 1 April";
const SHARE: &str = "PHI Act s 22-15(1) and s 22-20: the percentage of the person's share of the premiums eligible, without lifetime health cover loading (s 22-15(6)), rounded half up to the cent as the ATO's worked examples do";
const OFFSET: &str = "ITAA 1997 s 61-210: the share of the PHII benefit less the person's share of the premium reduction received, nil where that share equals or exceeds it; refundable (s 67-23 item 5)";
const EXCESS: &str = "PHI Act s 282-18(2) and (3): the person's share of the premium reduction received less the share of the PHII benefit, where more; payable with the year's income tax (s 282-18(6)) and shown on the notice of assessment";

/// The statement's benefit codes: (code, the period the premiums were paid
/// in, the oldest insured person's age band, the band's row in the
/// percentage tables). The ATO's 'Your private health insurance statement'.
const CODES: [(i64, &str, &str, usize); 6] = [
    (30, "1_july_to_31_march", "under_65", 0),
    (31, "1_april_to_30_june", "under_65", 0),
    (35, "1_july_to_31_march", "65_to_69", 1),
    (36, "1_april_to_30_june", "65_to_69", 1),
    (40, "1_july_to_31_march", "70_or_over", 2),
    (41, "1_april_to_30_june", "70_or_over", 2),
];

/// The arguments as the command passes them; see the README for each.
pub struct PhiRebateArgs<'a> {
    pub income_for_surcharge_purposes: &'a Arg,
    pub lines: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub family: SurchargeFamily<'a>,
    /// PHI Act s 22-30(1)(b): on any day in the year the person contributed in
    /// a substantial way to the maintenance of a dependent person who is their
    /// child or dependent sibling.
    pub maintained_dependent_person: Option<&'a Arg>,
}

/// One statement line as given.
struct Line {
    premiums: Decimal,
    received: Decimal,
    code: (i64, &'static str, &'static str, usize),
}

fn invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("private_health_insurance_rebate.lines_invalid", message)
}

fn line_money(fields: &Map<String, Value>, key: &str, n: usize) -> Result<Decimal> {
    let parsed = match fields.get(key) {
        Some(Value::String(text)) if text.len() <= 64 => Decimal::parse_input(text).ok(),
        _ => None,
    };
    match parsed.map(|v| money(&Arg::Decimal(v))) {
        Some(Ok(v)) => Ok(v),
        _ => invalid(format!(
            "line {n}: {key} must be a non-negative decimal string, at most 2dp and 1e12."
        )),
    }
}

fn read_line(item: &Value, n: usize) -> Result<Line> {
    let Value::Object(fields) = item else {
        return invalid(format!("line {n} must be an object."));
    };
    if fields.contains_key("lifetime_health_cover_loading") {
        return refuse(
            "private_health_insurance_rebate.lhc_loading",
            format!(
                "line {n}: lifetime health cover loading is outside the rebate (PHI Act s 22-15(6)); give premiums_eligible without it, as the statement's label J does. The loading itself is not worked out here."
            ),
        );
    }
    if let Some(key) = fields.keys().find(|k| !LINE_FIELDS.contains(&k.as_str())) {
        return invalid(format!(
            "line {n}: {key} is not a field of a line; give {}.",
            LINE_FIELDS.join(", ")
        ));
    }
    let code = fields
        .get("benefit_code")
        .and_then(Value::as_i64)
        .and_then(|c| CODES.iter().find(|(k, ..)| *k == c));
    let Some(code) = code else {
        return invalid(format!(
            "line {n}: benefit_code must be 30, 31, 35, 36, 40 or 41, the codes the statement uses."
        ));
    };
    Ok(Line {
        premiums: line_money(fields, "premiums_eligible", n)?,
        received: line_money(fields, "rebate_received", n)?,
        code: *code,
    })
}

fn read_lines(lines: &Arg) -> Result<Vec<Line>> {
    let Arg::Json(Value::Array(items)) = lines else {
        return invalid(format!(
            "lines must be a list of 1 to {MAX_LINES} statement line objects."
        ));
    };
    if items.is_empty() || items.len() > MAX_LINES {
        return invalid(format!(
            "lines must be a list of 1 to {MAX_LINES} statement line objects."
        ));
    }
    items
        .iter()
        .enumerate()
        .map(|(i, item)| read_line(item, i + 1))
        .collect()
}

/// The singles tier 1 to 3 thresholds for `year`.
fn singles_thresholds(year: &str) -> [&'static str; 3] {
    match meta::SURCHARGE_THRESHOLDS.iter().find(|(y, _)| *y == year) {
        Some((_, t)) => *t,
        None => *lookup(meta::PHI_EARLIER_THRESHOLDS, year),
    }
}

/// The percentage for a line: the adjustment year is the one starting on
/// 1 April of the income year's first calendar year for premiums paid before
/// 1 April, and of its second for premiums paid from 1 April (s 22-15(5C) and
/// (5D)).
fn percentage(year: &str, line: &Line, tier: usize) -> Result<Decimal> {
    let first: i64 = year[..4].parse().expect("a supported year");
    let (code, period, _, band) = line.code;
    let start = if period == "1_july_to_31_march" {
        first
    } else {
        first + 1
    };
    let Some((_, table)) = meta::PHI_REBATE_PERCENTAGES
        .iter()
        .find(|(s, _)| *s == start.to_string())
    else {
        return refuse(
            "private_health_insurance_rebate.rate_unpublished",
            format!(
                "benefit code {code}: the rebate percentages from 1 April {start} were not published when this was checked on 22 November 2026 (the ATO: 'will become available in March {start}'); PHI Act s 22-15(5E) leaves the adjustment factor to the Private Health Insurance (Incentives) Rules."
            ),
        );
    };
    Ok(if tier == 3 {
        d("0")
    } else {
        d(table[band][tier])
    })
}

pub fn private_health_insurance_rebate(args: &PhiRebateArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let own = money(args.income_for_surcharge_purposes)?;
    let spouse = args.family.spouse_income.map(money).transpose()?;
    let children = dependent_children(args.family.dependent_children)?;
    let maintained = optional_flag(
        args.maintained_dependent_person,
        "maintained_dependent_person",
    )?;
    let families = spouse.is_some() || maintained;
    // s 22-40(4) raises only the family thresholds.
    if children > 0 && !families {
        return not_taken(
            "private-health-insurance-rebate",
            "dependent_children",
            "without spouse_income_for_surcharge_purposes or maintained_dependent_person true: it raises only the family thresholds (PHI Act s 22-40(4))",
        );
    }
    let lines = read_lines(args.lines)?;
    precise(|| work(&year, own, spouse, families, children, &lines))
}

/// The family or singles thresholds for the year, the income tested against
/// them and the tier it falls in (PHI Act ss 22-30 and 22-40).
struct Tier {
    families: bool,
    thresholds: [Decimal; 3],
    income: Decimal,
    tier: usize,
}

fn tier_of(
    year: &str,
    own: Decimal,
    spouse: Option<Decimal>,
    families: bool,
    children: i64,
) -> Result<Tier> {
    let increase = int((children - 1).max(0)).mul(&d(meta::SURCHARGE_CHILD_INCREASE))?;
    let singles = singles_thresholds(year).map(d);
    let thresholds = if families {
        let [a, b, c] = singles.map(|t| t.mul(&int(2)).and_then(|t| t.add(&increase)));
        [a?, b?, c?]
    } else {
        singles
    };
    let income = match &spouse {
        Some(s) => own.add(s)?,
        None => own,
    };
    let mut tier = 0;
    for t in &thresholds {
        if income.gt(t)? {
            tier += 1;
        }
    }
    Ok(Tier {
        families,
        thresholds,
        income,
        tier,
    })
}

/// `value`, or nil where it is below nil.
fn at_least_nil(value: Decimal) -> Result<Decimal> {
    Ok(if value.lt(&int(0))? { int(0) } else { value })
}

/// One line's working, and its share of the PHII benefit, offset and excess.
fn work_line(year: &str, line: &Line, tier: usize) -> Result<(Value, [Decimal; 3])> {
    let pct = percentage(year, line, tier)?;
    let share = cents(line.premiums.mul(&pct)?.div(&int(100))?)?;
    let offset = at_least_nil(share.sub(&line.received)?)?;
    let excess = at_least_nil(line.received.sub(&share)?)?;
    let (code, period, band, _) = line.code;
    let worked = json!({
        "benefit_code": code,
        "period": period,
        "age_band": band,
        "premiums_eligible": line.premiums.to_string(),
        "rebate_received": line.received.to_string(),
        "rebate_percentage": pct.to_string(),
        "share_of_phii_benefit": share.to_string(),
        "private_health_insurance_offset": cents(offset.clone())?.to_string(),
        "excess_private_health_reduction": cents(excess.clone())?.to_string(),
    });
    Ok((worked, [share, offset, excess]))
}

/// The provision behind each figure.
fn authorities(families: bool) -> Value {
    json!({
        "thresholds": if families { FAMILIES } else { SINGLES },
        "tier": TIER,
        "rebate_percentage": PERCENTAGE,
        "share_of_phii_benefit": SHARE,
        "private_health_insurance_offset": OFFSET,
        "excess_private_health_reduction": EXCESS,
    })
}

fn work(
    year: &str,
    own: Decimal,
    spouse: Option<Decimal>,
    families: bool,
    children: i64,
    lines: &[Line],
) -> Result<Value> {
    let t = tier_of(year, own, spouse, families, children)?;
    let mut totals = [int(0), int(0), int(0), int(0)];
    let mut worked = Vec::new();
    for line in lines {
        let (value, [share, offset, excess]) = work_line(year, line, t.tier)?;
        worked.push(value);
        for (total, amount) in totals
            .iter_mut()
            .zip([share, line.received.clone(), offset, excess])
        {
            *total = total.add(&amount)?;
        }
    }
    let [benefit, received, offset, excess] = totals;
    let [t1, t2, t3] = t.thresholds;
    let mut out = result(
        KIND,
        year,
        vec![
            ("income_tested", t.income),
            ("share_of_phii_benefit", benefit),
            ("rebate_received", received),
            ("private_health_insurance_offset", offset),
            ("excess_private_health_reduction", excess),
        ],
        vec![
            rate("tier_1_threshold", t1),
            rate("tier_2_threshold", t2),
            rate("tier_3_threshold", t3),
        ],
    )?;
    out["tier"] = json!(TIERS[t.tier]);
    out["thresholds"] = json!(if t.families { "families" } else { "singles" });
    out["lines"] = Value::Array(worked);
    out["authorities"] = authorities(t.families);
    Ok(out)
}
