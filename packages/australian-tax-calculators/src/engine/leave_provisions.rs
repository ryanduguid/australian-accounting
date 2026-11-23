// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Annual and long service leave provisions under AASB 119: a short-term
//! line undiscounted (paras 11, 13 and 16), an other long-term line by the
//! projected unit credit method with salary growth, the probability of
//! payment and discounting (paras 67, 68, 72, 83 to 87 and 155), and the
//! current and non-current split of AASB 101 para 69. The paragraphs are those
//! of the AASB 119 compilations of December 2022 (for periods beginning from
//! 1 January 2023, and from 1 July 2026 with AASB 2022-9) and the AASB 101
//! compilations of March 2020 to December 2022, read on 23 November 2026.
//! Probability is measurement and the right to defer is classification, so a
//! line without a right to defer takes any probability. The standards' wording
//! is not reproduced.

use serde_json::{Map, Value, json};

use super::{Arg, Result, Step, derived_result, flag, int, precise, refuse, scope, step};
use crate::decimal::{Calc, Decimal, Rounding};

mod lines;

const KIND: &str = "leave_provisions";

/// The worksheet's inputs, as the caller gave them.
pub struct LeaveProvisionsArgs<'a> {
    pub lines: &'a Arg,
    pub defined_benefit_plan: &'a Arg,
    pub plan_assets: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const DEFINED_BENEFIT: &str = "A defined benefit plan or other post-employment benefit is measured under AASB 119 paras 26 to 152, which this worksheet does not apply.";
const PLAN_ASSETS: &str = "Plan assets and reimbursement rights for other long-term benefits follow AASB 119 paras 113 to 119 (through para 155), which this worksheet does not apply.";

/// One line's measured amounts, rounded to the cent.
struct Measured {
    value: Decimal,
    projected: Option<Decimal>,
    provision: Decimal,
    current: bool,
}

fn cents(v: Decimal) -> Calc<Decimal> {
    v.quantize(-2, Rounding::HalfUp)
}

/// `(1 + rate)` to the power `years`.
fn compound(rate: &Decimal, years: u64) -> Calc<Decimal> {
    let factor = int(1).add(rate)?;
    let mut out = int(1);
    for _ in 0..years {
        out = out.mul(&factor)?;
    }
    Ok(out)
}

/// AASB 119 paras 11 and 16 for a short-term line; paras 155, 67, 68, 72
/// and 83 to 87 for an other long-term one; AASB 101 para 69 for its class.
fn measure(line: &lines::Line) -> Calc<Measured> {
    let value = line.hours.mul(&line.rate)?;
    let Some(lt) = &line.long_term else {
        return Ok(Measured {
            value: cents(value.clone())?,
            projected: None,
            provision: cents(value.mul(&line.probability)?)?,
            current: true,
        });
    };
    let projected = value.mul(&compound(&lt.growth, lt.years)?)?;
    let expected = projected.mul(&line.probability)?;
    let provision = expected.div(&compound(&lt.discount, lt.years)?)?;
    Ok(Measured {
        value: cents(value)?,
        projected: Some(cents(projected)?),
        provision: cents(provision)?,
        current: !lt.can_defer,
    })
}

/// A step summing `terms`, shown as the numbers added.
fn sum(id: &'static str, authority: &'static str, terms: &[&Decimal]) -> Calc<Step> {
    let mut total = d0();
    for term in terms {
        total = total.add(term)?;
    }
    let shown: Vec<String> = terms.iter().map(ToString::to_string).collect();
    let formula = if shown.is_empty() {
        "0".to_string()
    } else {
        shown.join(" + ")
    };
    Ok(step(id, authority, &formula, &[], total))
}

fn d0() -> Decimal {
    super::d("0.00")
}

const ANNUAL: &str = "AASB 119 paras 11, 13(a) and 16 (short-term, undiscounted) and 155 (other long-term): the annual leave lines' provisions";
const LONG_SERVICE: &str = "AASB 119 paras 11 and 16 (short-term) and 153(a), 155, 67, 68, 72 and 83 to 87 (other long-term): the long service leave lines' provisions";
const TOTAL: &str = "AASB 119 paras 11 and 155: the leave provisions recognised as a liability";
const CURRENT: &str = "AASB 101 para 69: short-term lines (para 69(a) and para 70) and long-term lines the entity has no right to defer settling for 12 months (para 69(d): an unconditional right for periods beginning before 1 January 2024 unless AASB 2020-1 and AASB 2022-6 were applied early, a right at the end of the reporting period from then)";
const NON_CURRENT: &str = "AASB 101 para 69: every other liability is non-current (for periods beginning from 1 January 2024, whatever settlement is expected, para 75A)";

fn totals(all: &[lines::Line], measured: &[Measured]) -> Calc<Vec<Step>> {
    let pick = |f: &dyn Fn(usize) -> bool| -> Vec<&Decimal> {
        (0..all.len())
            .filter(|i| f(*i))
            .map(|i| &measured[i].provision)
            .collect()
    };
    let annual = sum(
        "annual_leave_provision",
        ANNUAL,
        &pick(&|i| !all[i].long_service),
    )?;
    let long = sum(
        "long_service_leave_provision",
        LONG_SERVICE,
        &pick(&|i| all[i].long_service),
    )?;
    let total = annual.value.add(&long.value)?;
    let total = step(
        "total_provision",
        TOTAL,
        "annual_leave_provision + long_service_leave_provision",
        &[&annual.value, &long.value],
        total,
    );
    let current = sum(
        "current_provision",
        CURRENT,
        &pick(&|i| measured[i].current),
    )?;
    let rest = total.value.sub(&current.value)?;
    let non_current = step(
        "non_current_provision",
        NON_CURRENT,
        "total_provision - current_provision",
        &[&total.value, &current.value],
        rest,
    );
    Ok(vec![annual, long, total, current, non_current])
}

/// One line's result.
fn shown(line: &lines::Line, m: &Measured) -> Value {
    let mut out = Map::new();
    out.insert("label".into(), json!(line.label));
    let leave = if line.long_service {
        "long_service"
    } else {
        "annual"
    };
    out.insert("leave".into(), json!(leave));
    let category = if line.long_term.is_some() {
        "other_long_term"
    } else {
        "short_term"
    };
    out.insert("category".into(), json!(category));
    out.insert("value_at_current_rate".into(), json!(m.value.to_string()));
    if let Some(p) = &m.projected {
        out.insert("projected_value".into(), json!(p.to_string()));
    }
    out.insert("provision".into(), json!(m.provision.to_string()));
    let class = if m.current { "current" } else { "non_current" };
    out.insert("classification".into(), json!(class));
    Value::Object(out)
}

/// AASB 119 for one entity's leave at one reporting date.
pub fn leave_provisions(a: &LeaveProvisionsArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    if flag(a.defined_benefit_plan, "defined_benefit_plan")? {
        return refuse("leave_provisions.defined_benefit_plan", DEFINED_BENEFIT);
    }
    if flag(a.plan_assets, "plan_assets")? {
        return refuse("leave_provisions.plan_assets", PLAN_ASSETS);
    }
    let all = lines::read(a.lines)?;
    let (measured, steps) = precise(|| {
        let measured = all.iter().map(measure).collect::<Calc<Vec<_>>>()?;
        let steps = totals(&all, &measured)?;
        Ok((measured, steps))
    })?;
    let mut out = derived_result(KIND, &year, steps, vec![])?;
    let shown: Vec<Value> = all
        .iter()
        .zip(&measured)
        .map(|(l, m)| shown(l, m))
        .collect();
    out["lines"] = Value::Array(shown);
    Ok(out)
}
