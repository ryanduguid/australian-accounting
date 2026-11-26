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

use serde_json::{Map, Value, json};

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

/// The circumstances `facts` must state true or false: (key, what it is, the
/// key whose truth makes it needed, the provision that refuses it when true).
type Circumstance = (
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
);
const CIRCUMSTANCES: [Circumstance; 8] = [
    (
        "underage_de_facto",
        "underage_de_facto: the person is in a de facto relationship in which either is under the age of consent (s 1064(4))",
        None,
        Some(
            "s 1064(4) limits the rate to the one payable were the other person the partner, which needs that person's income and assets",
        ),
    ),
    (
        "advance_deduction",
        "advance_deduction: a special employment advance deduction (Part 3.16B) or advance payment deduction (Part 3.16A) applies",
        None,
        Some(
            "s 1064-A1 step 12(a) and (b) subtract the advance deductions, whose amounts Services Australia sets",
        ),
    ),
    (
        "armed_services_payment",
        "armed_services_payment: the person is an armed services widow or widower receiving a Veterans' Entitlements Act Part II or IV pension at a rate under s 30(1), or the MRCA s 234(1)(b) weekly amount or s 236(5) lump sum (s 1064(5) to (7))",
        None,
        None,
    ),
    (
        "rate_before_november_1986",
        "rate_before_november_1986: the widow has received that pension continuously since before 1 November 1986 and was then receiving a pension of the same type at a rate above $3,247.40 a year (s 1064(5)(c))",
        Some("armed_services_payment"),
        Some("s 1064(5)(c) limits the rate to the one received immediately before 1 November 1986"),
    ),
    (
        "remote_area",
        "remote_area: the person's usual place of residence is in the remote area and the person is physically present there (s 14, s 1064-H1)",
        None,
        None,
    ),
    (
        "remote_area_child",
        "remote_area_child: the person has an FTB child or regular care child, counting a partner's under s 1064-H5 to H7",
        Some("remote_area"),
        Some(
            "s 1064-H2 adds an additional remote area allowance for each FTB child and regular care child",
        ),
    ),
    (
        "pension_supplement_election",
        "pension_supplement_election: an election under s 1061VA(1) to receive the minimum pension supplement amount quarterly is in force (s 1064-BA2)",
        None,
        None,
    ),
    (
        "quarterly_energy_supplement",
        "quarterly_energy_supplement: quarterly energy supplement is payable to the person (s 1064-C2)",
        None,
        None,
    ),
];

/// What the stated circumstances change.
struct Circumstances {
    limited: bool,
    remote_area: bool,
    election: bool,
    quarterly_energy: bool,
}

/// The minimum pension supplement amount a year, `[single, partnered each]`,
/// from its date (SS Guide 5.1.9.10); the earlier periods' were not read.
const MINIMUM_SUPPLEMENT: (&str, [&str; 2]) = ("2026-09-20", ["1235.00", "930.80"]);
/// The remote area allowance a year, `[single, partnered]` (s 1064-H2 Table H
/// column 3, which s 1191 does not index).
const REMOTE_AREA: [&str; 2] = ["473.20", "405.60"];
/// The yearly limit of s 1064(5)(d), (6) and (7), which s 1191 does not index.
const RATE_LIMIT: &str = "3247.40";

/// One day's annual amounts: MBR and pension supplement `[single, partnered
/// each]`, the deeming rates `[below, above the threshold]`, the income free
/// area and deeming threshold `[single, couple combined]` and the assets value
/// limits `[single, couple combined]` for a homeowner then a non-homeowner.
struct Period {
    mbr: [&'static str; 2],
    supplement: [&'static str; 2],
    deeming: [&'static str; 2],
    free_area: [&'static str; 2],
    limits: [[&'static str; 2]; 2],
    thresholds: [&'static str; 2],
}

const ENERGY: [&str; 2] = ["366.60", "275.60"];

/// The amounts indexed on 20 March and 20 September (SS Guide 5.1.8.10 and
/// 4.4.1.10), each from its date to the next, the last to [`UNTIL`]: (from,
/// MBR, pension supplement, deeming rates).
type Indexed = (
    &'static str,
    [&'static str; 2],
    [&'static str; 2],
    [&'static str; 2],
);
const RATES: [Indexed; 3] = [
    (
        "2025-09-20",
        ["28072.20", "21161.40"],
        ["2207.40", "1664.00"],
        ["0.0075", "0.0275"],
    ),
    (
        "2026-03-20",
        ["28607.80", "21564.40"],
        ["2249.00", "1695.20"],
        ["0.0125", "0.0325"],
    ),
    (
        "2026-09-20",
        ["29520.40", "22253.40"],
        ["2293.20", "1729.00"],
        ["0.0175", "0.0375"],
    ),
];
const UNTIL: &str = "2027-03-19";

/// The amounts indexed on 1 July (SS Guide 4.10.3, 4.4.1.10 and 5.2.2.10):
/// (financial year, income free area, assets value limits, deeming
/// thresholds).
type Yearly = (
    &'static str,
    [&'static str; 2],
    [[&'static str; 2]; 2],
    [&'static str; 2],
);
const YEARS: [Yearly; 2] = [
    (
        "2025-26",
        ["5668", "9880"],
        [["321500", "481500"], ["579500", "739500"]],
        ["64200", "106200"],
    ),
    (
        "2026-27",
        ["5876", "10296"],
        [["333000", "499000"], ["600000", "766000"]],
        ["66800", "110600"],
    ),
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

fn period(given: &Arg, year: &str) -> Result<Period> {
    let date = given.text().filter(|t| dates::parse(t).is_some());
    let Some((t, r)) = date
        .filter(|t| RATES[0].0 <= *t && *t <= UNTIL)
        .and_then(|t| Some((t, RATES.iter().rev().find(|r| r.0 <= t)?)))
    else {
        return refuse(
            "age_pension.date_uncovered",
            "Not computed: date must be a YYYY-MM-DD day from 20 September 2025 to 19 March 2027, the days the indexed rates were read for.",
        );
    };
    let start: i64 = year[..4].parse().unwrap_or_default();
    let (first, last) = (format!("{start}-07-01"), format!("{}-06-30", start + 1));
    let Some(y) = YEARS
        .iter()
        .find(|y| y.0 == year && first.as_str() <= t && t <= last.as_str())
    else {
        return refuse(
            "age_pension.date_invalid",
            format!("date must be a YYYY-MM-DD day in {year}."),
        );
    };
    Ok(Period {
        mbr: r.1,
        supplement: r.2,
        deeming: r.3,
        free_area: y.1,
        limits: y.2,
        thresholds: y.3,
    })
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

fn all_facts() -> Vec<(&'static str, &'static str)> {
    FACTS
        .iter()
        .chain(&COUPLE_FACTS)
        .copied()
        .chain(CIRCUMSTANCES.iter().map(|c| (c.0, c.1)))
        .collect()
}

fn facts(given: &Arg, couple: bool) -> Result<Circumstances> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "age_pension.facts_invalid",
            "facts must be an object of booleans.",
        );
    };
    match stated::unstated(fields, &all_facts(), FACTS.len() + usize::from(couple))? {
        None => circumstances(fields),
        Some(missing) => refuse(
            "age_pension.facts_not_stated",
            format!(
                "Not computed: the worksheet covers only the stated case; not stated: {missing}."
            ),
        ),
    }
}

/// Reads each circumstance needed, refusing one unstated or one this command
/// does not work (`age_pension.case_not_built`, naming the provision).
fn circumstances(fields: &Map<String, Value>) -> Result<Circumstances> {
    let mut stated: Vec<(&str, bool)> = Vec::new();
    let mut missing = Vec::new();
    for (key, text, under, _) in CIRCUMSTANCES {
        if under.is_some_and(|u| !stated.contains(&(u, true))) {
            continue;
        }
        match fields.get(key) {
            Some(Value::Bool(value)) => stated.push((key, *value)),
            None => missing.push(text),
            Some(_) => {
                return refuse(
                    "input.flag_invalid",
                    format!("facts: {key} must be a boolean."),
                );
            }
        }
    }
    if !missing.is_empty() {
        return refuse(
            "age_pension.facts_not_stated",
            format!(
                "Not computed: facts must state each circumstance true or false; not stated: {}.",
                missing.join("; ")
            ),
        );
    }
    let is = |key: &str| stated.contains(&(key, true));
    if let Some((_, _, _, Some(why))) = CIRCUMSTANCES.iter().find(|c| c.3.is_some() && is(c.0)) {
        return not_built(why);
    }
    Ok(Circumstances {
        limited: is("armed_services_payment"),
        remote_area: is("remote_area"),
        election: is("pension_supplement_election"),
        quarterly_energy: is("quarterly_energy_supplement"),
    })
}

fn not_built<T>(why: &str) -> Result<T> {
    refuse(
        "age_pension.case_not_built",
        format!("Not computed: {why}; this command does not work that case."),
    )
}

/// The minimum pension supplement amount a year where an s 1061VA(1)
/// election is in force (s 1064-BA4), refused where not read or where the
/// s 1064(5) to (7) limit would also apply.
fn minimum_supplement(c: &Circumstances, date: &str, i: usize) -> Result<Option<Decimal>> {
    if !c.election {
        return Ok(None);
    }
    if date < MINIMUM_SUPPLEMENT.0 {
        return not_built(
            "s 1064-BA4 subtracts the minimum pension supplement amount, which was read only for days from 20 September 2026 (SS Guide 5.1.9.10)",
        );
    }
    if c.limited {
        return not_built(
            "s 1064(5) to (7) limit the pension rate and s 1061VA pays the minimum pension supplement amount quarterly; how the limit meets the quarterly amount is not settled here",
        );
    }
    Ok(Some(d(MINIMUM_SUPPLEMENT.1[i])))
}

pub fn age_pension(args: &AgePensionArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let p = period(args.date, &year)?;
    let couple = couple(args.relationship)?;
    let homeowner = flag(args.homeowner, "homeowner")?;
    let c = facts(args.facts, couple)?;
    let i = usize::from(couple);
    let minimum = minimum_supplement(&c, args.date.text().unwrap_or_default(), i)?;
    let income = money(args.ordinary_income)?;
    let financial = money(args.financial_assets)?;
    let assets = money(args.assets)?;
    if financial.gt(&assets)? {
        return refuse(
            "age_pension.assets_inconsistent",
            "financial_assets cannot be more than assets, which include them (s 1118 and Division 1B of Part 3.10).",
        );
    }
    let rates = Rates {
        mbr: d(p.mbr[i]),
        supplement: d(p.supplement[i]),
        energy: (!c.quarterly_energy).then(|| d(ENERGY[i])),
        threshold: d(p.thresholds[i]),
        lower: d(p.deeming[0]),
        upper: d(p.deeming[1]),
        free_area: d(p.free_area[i]).div(&int(26))?,
        limit: d(p.limits[usize::from(!homeowner)][i]),
        minimum,
        remote_area: c.remote_area.then(|| d(REMOTE_AREA[i])),
        rate_limit: c.limited.then(|| d(RATE_LIMIT)),
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
    /// The energy supplement, none where quarterly energy supplement is payable.
    energy: Option<Decimal>,
    threshold: Decimal,
    lower: Decimal,
    upper: Decimal,
    free_area: Decimal,
    limit: Decimal,
    /// The minimum pension supplement amount where an election is in force.
    minimum: Option<Decimal>,
    /// The remote area allowance a year where Module H applies.
    remote_area: Option<Decimal>,
    /// The yearly limit where s 1064(5) to (7) apply.
    rate_limit: Option<Decimal>,
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
        let mut shown = vec![
            rate("mbr", &self.mbr),
            rate("pension_supplement", &self.supplement),
        ];
        shown.extend(self.energy.iter().map(|e| rate("energy_supplement", e)));
        shown.extend([
            rate("deeming_threshold", &self.threshold),
            rate("lower_deeming_rate", &self.lower),
            rate("upper_deeming_rate", &self.upper),
            rate("income_free_area", &self.free_area),
            rate("assets_value_limit", limit),
        ]);
        for (name, value) in [
            ("minimum_pension_supplement", &self.minimum),
            ("remote_area_allowance", &self.remote_area),
            ("rate_limit", &self.rate_limit),
        ] {
            shown.extend(value.iter().map(|v| rate(name, v)));
        }
        shown
    }

    /// The maximum payment rate per fortnight, less `minimum` a year if any.
    fn max_rate(&self, minimum: Option<&Decimal>) -> Result<Decimal> {
        let mut yearly = self.mbr.add(&self.supplement)?;
        if let Some(minimum) = minimum {
            yearly = yearly.sub(minimum)?;
        }
        if let Some(energy) = &self.energy {
            yearly = yearly.add(energy)?;
        }
        Ok(yearly.div(&int(26))?)
    }

    /// The maximum payment rate's authority, formula and operands, a year.
    fn maximum(&self) -> (&'static str, String, Vec<&Decimal>) {
        let mut formula = String::from("(mbr + pension_supplement");
        let mut operands = vec![&self.mbr, &self.supplement];
        if let Some(minimum) = &self.minimum {
            formula.push_str(" - minimum_pension_supplement");
            operands.push(minimum);
        }
        if let Some(energy) = &self.energy {
            formula.push_str(" + energy_supplement");
            operands.push(energy);
        }
        formula.push_str(") / 26");
        let changed = self.minimum.is_some() || self.energy.is_none();
        (AUTH[if changed { 5 } else { 0 }], formula, operands)
    }
}

const AUTH: [&str; 8] = [
    "SSA s 1064-A1 steps 1 to 4, Module B (MBR), Module BA (pension supplement) and s 1061VA (energy supplement): the maximum payment rate, per year / 26",
    "SSA s 1076 and s 1082: financial assets deemed to earn the below threshold rate up to the deeming threshold and the above threshold rate on the rest, per year / 26",
    "SSA s 1064-E1, E2, E4, E10 and E11: 50% of ordinary income over the free area; a couple's income is combined and halved, so 25% of the couple's income over the couple's combined free area",
    "SSA s 1064-G1 to G5 and G7: $19.50 a year for each whole $250 of assets over the assets value limit, a couple's assets combined and halved against each member's limit, per year / 26",
    "SSA s 1064-A1 steps 5 to 12: the maximum payment rate less the greater of the income and assets reductions, not below nil",
    "SSA s 1064-A1 steps 1 to 4, Module B (MBR), Module BA (pension supplement, less the minimum pension supplement amount under BA4 where an s 1061VA(1) election is in force) and Module C (energy supplement, not added under C2 where quarterly energy supplement is payable): the maximum payment rate, per year / 26",
    "SSA s 1064-A1 step 12(c) and Module H (H1 and H2 Table H, not indexed under s 1191): the remote area allowance, added where the rate apart from it is above nil or is nil only because an s 1061VA(1) election is in force, per year / 26",
    "SSA s 1064-A1 step 12, with s 1064(5)(d), (6) and (7) where they apply: the provisional rate plus any remote area allowance, not above $3,247.40 a year / 26",
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
    let (max_authority, max_formula, max_operands) = r.maximum();
    let max_rate = r.max_rate(r.minimum.as_ref())?;
    let deemed = deemed(r, financial)?;
    let taper = d(if couple { "0.25" } else { "0.5" });
    let excess = super::py_max(income.add(&deemed)?.sub(&r.free_area)?, int(0))?;
    let income_reduction = excess.mul(&taper)?;
    let (assets_reduction, limit) = assets_reduction(couple, r, assets)?;
    let reduction = super::py_max(income_reduction.clone(), assets_reduction.clone())?;
    let pension = super::py_max(max_rate.sub(&reduction)?, int(0))?;
    let (income_formula, assets_formula) = FORMULAS[usize::from(couple)];
    let mut steps = vec![
        step(
            "max_rate",
            max_authority,
            &max_formula,
            &max_operands,
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
    ];
    steps.extend(rate_steps(
        r,
        [&max_rate, &income_reduction, &assets_reduction],
        pension,
    )?);
    Ok(steps)
}

/// Step 11's rate from the maximum payment rate and the 2 reductions, then
/// step 12 where it changes anything (`provisional_rate` then `rate`).
fn rate_steps(r: &Rates, operands: [&Decimal; 3], pension: Decimal) -> Result<Vec<Step>> {
    let provisional = r.remote_area.is_some() || r.rate_limit.is_some();
    let mut steps = vec![step(
        if provisional {
            "provisional_rate"
        } else {
            "rate"
        },
        AUTH[4],
        "max(max_rate - max(income_reduction, assets_reduction), 0)",
        &operands,
        pension.clone(),
    )];
    if provisional {
        let reduction = super::py_max(operands[1].clone(), operands[2].clone())?;
        let allowance = allowance(r, &reduction, &pension)?;
        steps.extend(step_12(r, pension, allowance)?);
    }
    Ok(steps)
}

/// The remote area allowance per fortnight where Module H applies: nil
/// unless the rate apart from it is above nil, or is nil only because the
/// s 1061VA(1) election took the minimum pension supplement amount out of
/// the maximum payment rate (s 1064-H1(aa)).
fn allowance(r: &Rates, reduction: &Decimal, provisional: &Decimal) -> Result<Option<Decimal>> {
    let Some(yearly) = &r.remote_area else {
        return Ok(None);
    };
    let unelected = r.max_rate(None)?;
    let applies = provisional.gt(&int(0))? || unelected.gt(reduction)?;
    Ok(Some(if applies {
        yearly.div(&int(26))?
    } else {
        int(0)
    }))
}

/// Step 12(c) and the limits of s 1064(5)(d), (6) and (7) after the
/// provisional rate: the allowance step, if any, then `rate`.
fn step_12(r: &Rates, provisional: Decimal, allowance: Option<Decimal>) -> Result<Vec<Step>> {
    let mut rate = provisional.clone();
    let mut steps = Vec::new();
    let mut formula = String::from("provisional_rate");
    let mut operands = vec![&provisional];
    if let (Some(value), Some(yearly)) = (&allowance, &r.remote_area) {
        rate = rate.add(value)?;
        steps.push(step(
            "remote_area_allowance",
            AUTH[6],
            "remote_area_allowance / 26",
            &[yearly],
            value.clone(),
        ));
        formula.push_str(" + remote_area_allowance");
        operands.push(value);
    }
    if let Some(limit) = &r.rate_limit {
        rate = super::py_min(rate, limit.div(&int(26))?)?;
        formula = format!("min({formula}, rate_limit / 26)");
        operands.push(limit);
    }
    steps.push(step("rate", AUTH[7], &formula, &operands, rate));
    Ok(steps)
}
