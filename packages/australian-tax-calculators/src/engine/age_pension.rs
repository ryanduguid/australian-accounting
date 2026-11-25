// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `age_pension`: one fortnight's age pension rate for a single person or for
//! each member of a couple who both receive it, under Pension Rate Calculator
//! A (Social Security Act 1991 s 1064, compilation 234 of 20 September 2026):
//! the maximum payment rate (Module A, MBR, pension supplement and energy
//! supplement) less the greater of the reduction for ordinary income (Module
//! E, deemed income from financial assets under s 1082) and the reduction for
//! assets (Module G). The indexed amounts are the Department of Social
//! Services' Social Security Guide (version 1.345, 3 November 2026: 4.4.1.10,
//! 4.10.3, 5.1.8.10, 5.1.10.20, 5.2.2.10 and 5.2.8.10), read on 26 November
//! 2026, for each day from 20 September 2025 to 19 March 2027.

use serde_json::{Value, json};

use super::stated;
use super::{
    Arg, Result, Step, derived_result, flag, int, money, precise, rate, refuse, scope, step,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "age_pension";

/// The facts every case needs, then those a couple needs.
const FACTS: [(&str, &str); 4] = [
    (
        "in_australia",
        "the person resides in Australia and is in Australia, or has been temporarily absent for no more than 6 weeks (SS Guide 5.1.8.10: the energy supplement and pension supplement are in the maximum rate)",
    ),
    (
        "not_blind",
        "the person is not permanently blind (Pension Rate Calculator A, s 1064, not B)",
    ),
    (
        "not_transitional",
        "the person is not paid a transitional rate (Schedule 1A clause 146)",
    ),
    (
        "no_rent_assistance",
        "the person receives no rent assistance (Module D)",
    ),
];
const COUPLE_FACTS: [(&str, &str); 1] = [(
    "partner_pensioner",
    "the partner also receives age pension and the couple are not illness separated, in respite care or apart because the partner is imprisoned",
)];

/// One period's annual amounts: `[single, partnered each]`, the income free
/// area `[single, couple combined]` and the assets value limits
/// `[single, couple combined]` for a homeowner then a non-homeowner.
struct Period {
    from: &'static str,
    to: &'static str,
    year: &'static str,
    mbr: [&'static str; 2],
    supplement: [&'static str; 2],
    free_area: [&'static str; 2],
    limits: [[&'static str; 2]; 2],
    deeming: [&'static str; 2],
    thresholds: [&'static str; 2],
}

const ENERGY: [&str; 2] = ["366.60", "275.60"];

const PERIODS: [Period; 4] = [
    Period {
        from: "2025-09-20",
        to: "2026-03-19",
        year: "2025-26",
        mbr: ["28072.20", "21161.40"],
        supplement: ["2207.40", "1664.00"],
        free_area: ["5668", "9880"],
        limits: [["321500", "481500"], ["579500", "739500"]],
        deeming: ["0.0075", "0.0275"],
        thresholds: ["64200", "106200"],
    },
    Period {
        from: "2026-03-20",
        to: "2026-06-30",
        year: "2025-26",
        mbr: ["28607.80", "21564.40"],
        supplement: ["2249.00", "1695.20"],
        free_area: ["5668", "9880"],
        limits: [["321500", "481500"], ["579500", "739500"]],
        deeming: ["0.0125", "0.0325"],
        thresholds: ["64200", "106200"],
    },
    Period {
        from: "2026-07-01",
        to: "2026-09-19",
        year: "2026-27",
        mbr: ["28607.80", "21564.40"],
        supplement: ["2249.00", "1695.20"],
        free_area: ["5876", "10296"],
        limits: [["333000", "499000"], ["600000", "766000"]],
        deeming: ["0.0125", "0.0325"],
        thresholds: ["66800", "110600"],
    },
    Period {
        from: "2026-09-20",
        to: "2027-03-19",
        year: "2026-27",
        mbr: ["29520.40", "22253.40"],
        supplement: ["2293.20", "1729.00"],
        free_area: ["5876", "10296"],
        limits: [["333000", "499000"], ["600000", "766000"]],
        deeming: ["0.0175", "0.0375"],
        thresholds: ["66800", "110600"],
    },
];

/// The facts as the command passes them; see the README for each.
pub struct AgePensionArgs<'a> {
    pub date: &'a Arg,
    pub relationship: &'a Arg,
    pub homeowner: &'a Arg,
    pub ordinary_income: &'a Arg,
    pub financial_assets: &'a Arg,
    pub assets: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn d(text: &str) -> Decimal {
    super::d(text)
}

fn period(given: &Arg, year: &str) -> Result<&'static Period> {
    let found = given
        .text()
        .filter(|t| dates::parse(t).is_some())
        .and_then(|t| PERIODS.iter().find(|p| p.from <= t && t <= p.to));
    match found {
        Some(p) if p.year == year => Ok(p),
        Some(_) => refuse(
            "age_pension.date_invalid",
            format!("date must be a YYYY-MM-DD day in {year}."),
        ),
        None => refuse(
            "age_pension.date_uncovered",
            "Not computed: date must be a YYYY-MM-DD day from 20 September 2025 to 19 March 2027, the days the indexed rates were read for.",
        ),
    }
}

fn couple(given: &Arg) -> Result<bool> {
    match given.text() {
        Some("single") => Ok(false),
        Some("couple") => Ok(true),
        _ => refuse(
            "age_pension.relationship_invalid",
            "relationship must be single or couple.",
        ),
    }
}

fn facts(given: &Arg, couple: bool) -> Result<()> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "age_pension.facts_invalid",
            "facts must be an object of booleans.",
        );
    };
    let all: Vec<(&str, &str)> = FACTS.iter().chain(&COUPLE_FACTS).copied().collect();
    let keys: Vec<&str> = all.iter().map(|(k, _)| *k).collect();
    stated::known_keys(fields, "facts", "facts", &keys)?;
    let needed = FACTS.iter().chain(COUPLE_FACTS.iter().filter(|_| couple));
    let mut missing = Vec::new();
    for (key, text) in needed {
        match fields.get(*key) {
            Some(Value::Bool(true)) => {}
            None | Some(Value::Bool(false)) => missing.push(*text),
            Some(_) => {
                return refuse(
                    "input.flag_invalid",
                    format!("facts: {key} must be a boolean."),
                );
            }
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    refuse(
        "age_pension.facts_not_stated",
        format!(
            "Not computed: the worksheet covers only the stated case; not stated: {}.",
            missing.join("; ")
        ),
    )
}

pub fn age_pension(args: &AgePensionArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let p = period(args.date, &year)?;
    let couple = couple(args.relationship)?;
    let homeowner = flag(args.homeowner, "homeowner")?;
    facts(args.facts, couple)?;
    let income = money(args.ordinary_income)?;
    let financial = money(args.financial_assets)?;
    let assets = money(args.assets)?;
    if financial.gt(&assets)? {
        return refuse(
            "age_pension.assets_inconsistent",
            "financial_assets cannot be more than assets, which include them (s 1118 and Division 1B of Part 3.10).",
        );
    }
    let i = usize::from(couple);
    let rates = Rates {
        mbr: d(p.mbr[i]),
        supplement: d(p.supplement[i]),
        energy: d(ENERGY[i]),
        threshold: d(p.thresholds[i]),
        lower: d(p.deeming[0]),
        upper: d(p.deeming[1]),
        free_area: d(p.free_area[i]).div(&int(26))?,
        limit: d(p.limits[usize::from(!homeowner)][i]),
    };
    let steps = precise(|| working(couple, &rates, [&income, &financial, &assets]))?;
    let mut out = derived_result(KIND, &year, steps, rates.shown(couple))?;
    out["date"] = json!(args.date.text());
    out["relationship"] = json!(if couple { "couple" } else { "single" });
    out["homeowner"] = json!(homeowner);
    out["rate_basis"] = json!(if couple {
        "each member of the couple, per fortnight; the couple's income and assets are combined and halved (s 1064-E2 and G2)"
    } else {
        "per fortnight"
    });
    Ok(out)
}

struct Rates {
    mbr: Decimal,
    supplement: Decimal,
    energy: Decimal,
    threshold: Decimal,
    lower: Decimal,
    upper: Decimal,
    free_area: Decimal,
    limit: Decimal,
}

impl Rates {
    fn shown(&self, couple: bool) -> Vec<(String, String)> {
        let limit = if couple {
            self.limit
                .div(&int(2))
                .unwrap_or_else(|_| self.limit.clone())
        } else {
            self.limit.clone()
        };
        vec![
            rate("mbr", &self.mbr),
            rate("pension_supplement", &self.supplement),
            rate("energy_supplement", &self.energy),
            rate("deeming_threshold", &self.threshold),
            rate("lower_deeming_rate", &self.lower),
            rate("upper_deeming_rate", &self.upper),
            rate("income_free_area", &self.free_area),
            rate("assets_value_limit", limit),
        ]
    }
}

const AUTH: [&str; 5] = [
    "SSA s 1064-A1 steps 1 to 4, Module B (MBR), Module BA (pension supplement) and s 1061VA (energy supplement): the maximum payment rate, per year / 26",
    "SSA s 1076 and s 1082: financial assets deemed to earn the below threshold rate up to the deeming threshold and the above threshold rate on the rest, per year / 26",
    "SSA s 1064-E1, E2, E4, E10 and E11: 50% of ordinary income over the free area; a couple's income is combined and halved, so 25% of the couple's income over the couple's combined free area",
    "SSA s 1064-G1 to G5 and G7: $19.50 a year for each whole $250 of assets over the assets value limit, a couple's assets combined and halved against each member's limit, per year / 26",
    "SSA s 1064-A1 steps 5 to 12: the maximum payment rate less the greater of the income and assets reductions, not below nil",
];

/// The deemed income per fortnight (s 1082).
fn deemed(r: &Rates, financial: &Decimal) -> Result<Decimal> {
    let under = super::py_min(financial.clone(), r.threshold.clone())?;
    let over = super::py_max(financial.sub(&r.threshold)?, int(0))?;
    Ok(under
        .mul(&r.lower)?
        .add(&over.mul(&r.upper)?)?
        .div(&int(26))?)
}

/// The reduction for assets per fortnight and each member's limit
/// (s 1064-G2, G4 and G7).
fn assets_reduction(couple: bool, r: &Rates, assets: &Decimal) -> Result<(Decimal, Decimal)> {
    let (held, limit) = if couple {
        (assets.div(&int(2))?, r.limit.div(&int(2))?)
    } else {
        (assets.clone(), r.limit.clone())
    };
    let blocks = super::py_max(held.sub(&limit)?, int(0))?
        .div(&int(250))?
        .quantize(0, Rounding::Down)?;
    Ok((blocks.mul(&d("19.5"))?.div(&int(26))?, limit))
}

/// (income reduction, assets reduction) formulas, single then couple.
const FORMULAS: [(&str, &str); 2] = [
    (
        "max(ordinary_income + deemed_income - income_free_area, 0) x 0.5",
        "floor(max(assets - assets_value_limit, 0) / 250) x 19.5 / 26",
    ),
    (
        "max(ordinary_income + deemed_income - income_free_area, 0) x 0.25",
        "floor(max(assets / 2 - assets_value_limit, 0) / 250) x 19.5 / 26",
    ),
];

fn deemed_step(r: &Rates, financial: &Decimal, deemed: Decimal) -> Step {
    step(
        "deemed_income",
        AUTH[1],
        "(min(financial_assets, deeming_threshold) x lower_deeming_rate + max(financial_assets - deeming_threshold, 0) x upper_deeming_rate) / 26",
        &[
            financial,
            &r.threshold,
            &r.lower,
            financial,
            &r.threshold,
            &r.upper,
        ],
        deemed,
    )
}

fn working(
    couple: bool,
    r: &Rates,
    [income, financial, assets]: [&Decimal; 3],
) -> Result<Vec<Step>> {
    let max_rate = r.mbr.add(&r.supplement)?.add(&r.energy)?.div(&int(26))?;
    let deemed = deemed(r, financial)?;
    let taper = d(if couple { "0.25" } else { "0.5" });
    let excess = super::py_max(income.add(&deemed)?.sub(&r.free_area)?, int(0))?;
    let income_reduction = excess.mul(&taper)?;
    let (assets_reduction, limit) = assets_reduction(couple, r, assets)?;
    let reduction = super::py_max(income_reduction.clone(), assets_reduction.clone())?;
    let pension = super::py_max(max_rate.sub(&reduction)?, int(0))?;
    let (income_formula, assets_formula) = FORMULAS[usize::from(couple)];
    Ok(vec![
        step(
            "max_rate",
            AUTH[0],
            "(mbr + pension_supplement + energy_supplement) / 26",
            &[&r.mbr, &r.supplement, &r.energy],
            max_rate.clone(),
        ),
        deemed_step(r, financial, deemed.clone()),
        step(
            "income_reduction",
            AUTH[2],
            income_formula,
            &[income, &deemed, &r.free_area],
            income_reduction.clone(),
        ),
        step(
            "assets_reduction",
            AUTH[3],
            assets_formula,
            &[assets, &limit],
            assets_reduction.clone(),
        ),
        step(
            "rate",
            AUTH[4],
            "max(max_rate - max(income_reduction, assets_reduction), 0)",
            &[&max_rate, &income_reduction, &assets_reduction],
            pension,
        ),
    ])
}
