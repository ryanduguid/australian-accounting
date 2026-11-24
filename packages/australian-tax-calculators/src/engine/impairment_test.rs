// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `impairment-test`: an asset's or cash-generating unit's value in use from
//! stated pre-tax cash flows, a terminal value and a pre-tax discount rate
//! (AASB 136 paras 30, 31, 33, 39, 50 to 52 and 55), its recoverable amount
//! (para 18), the impairment loss (paras 59 and 104) and the loss allocated to
//! goodwill and then pro rata to the other assets, none below its floor
//! (paras 104 and 105). The paragraphs are those of the AASB 136 compilations
//! COMPjan26_07-26 (F2026C01108) and COMPdec22_01-23, read on 24 November 2026;
//! none of them was amended after 2015. The rate, the cash flows and the
//! floors are the preparer's. The Standard's wording is not reproduced.

use serde_json::{Map, Value, json};

use super::{Arg, Result, d, flag, money, precise, refuse, result, scope, signed_money};
use crate::decimal::{Calc, Decimal, Rounding};

const KIND: &str = "impairment_test";
const MOST_YEARS: usize = 50;
const MOST_ASSETS: usize = 200;

/// The worksheet's inputs, as the caller gave them.
pub struct ImpairmentTestArgs<'a> {
    pub cash_flows: &'a Arg,
    pub terminal_value: Option<&'a Arg>,
    pub discount_rate: &'a Arg,
    pub fair_value_less_costs_of_disposal: Option<&'a Arg>,
    pub assets: &'a Arg,
    pub revalued: &'a Arg,
    pub non_controlling_interest: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One asset of the unit, or the asset tested.
struct Asset {
    label: String,
    carrying: Decimal,
    goodwill: bool,
    floor: Decimal,
}

fn cents(v: &Decimal) -> Calc<Decimal> {
    v.quantize(-2, Rounding::HalfUp)
}

fn text_amount(v: Option<&Value>, signed: bool) -> Option<Decimal> {
    let parsed = Arg::Decimal(Decimal::parse_input(v?.as_str()?).ok()?);
    if signed {
        signed_money(&parsed)
    } else {
        money(&parsed)
    }
    .ok()
}

fn cash_flows(given: &Arg) -> Result<Vec<Decimal>> {
    let read: Option<Vec<Decimal>> = match given {
        Arg::Json(Value::Array(items)) if (1..=MOST_YEARS).contains(&items.len()) => {
            items.iter().map(|v| text_amount(Some(v), true)).collect()
        }
        _ => None,
    };
    read.map_or_else(
        || {
            refuse(
                "impairment_test.cash_flows_invalid",
                format!("cash_flows must be a list of 1 to {MOST_YEARS} decimal strings, at most 2 dp and 1e12 either side of nil."),
            )
        },
        Ok,
    )
}

fn discount_rate(given: &Arg) -> Result<Decimal> {
    match given {
        Arg::Decimal(r)
            if r.is_finite() && r.exponent() >= -6 && r.gt(&d("0"))? && !r.gt(&d("1"))? =>
        {
            Ok(r.clone())
        }
        _ => refuse(
            "impairment_test.rate_invalid",
            "discount_rate must be a decimal fraction above 0 and at most 1 (100%), at most 6 places: the stated pre-tax rate (AASB 136 para 55). The bounds are this worksheet's, not the Standard's.",
        ),
    }
}

fn asset(v: &Value, n: usize) -> Result<Asset> {
    let wrong = |why: &str| refuse("impairment_test.asset_invalid", format!("asset {n}: {why}"));
    let Some(o) = v.as_object() else {
        return wrong("must be an object.");
    };
    let known = |k: &String| {
        matches!(
            k.as_str(),
            "label" | "carrying_amount" | "goodwill" | "floor"
        )
    };
    let label = o
        .get("label")
        .and_then(Value::as_str)
        .filter(|l| !l.trim().is_empty());
    let carrying = text_amount(o.get("carrying_amount"), false);
    let goodwill = o.get("goodwill").and_then(Value::as_bool);
    let floor = o.get("floor").map(|f| text_amount(Some(f), false));
    match (label, carrying, goodwill, floor) {
        _ if !o.keys().all(known) => {
            wrong("takes label, carrying_amount, goodwill and floor only.")
        }
        (Some(_), Some(_), Some(true), Some(_)) => {
            wrong("goodwill takes no floor; it may be reduced to nil (AASB 136 para 104(a)).")
        }
        (Some(l), Some(c), Some(g), None) => Ok(Asset {
            label: l.to_string(),
            carrying: c,
            goodwill: g,
            floor: d("0.00"),
        }),
        (Some(l), Some(c), Some(g), Some(Some(f))) => Ok(Asset {
            label: l.to_string(),
            carrying: c,
            goodwill: g,
            floor: f,
        }),
        _ => wrong(
            "needs a label, a carrying_amount in AUD from 0, goodwill true or false and, optionally, a floor in AUD from 0.",
        ),
    }
}

fn assets(given: &Arg) -> Result<Vec<Asset>> {
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            "impairment_test.asset_invalid",
            format!("assets must be a list of 1 to {MOST_ASSETS} objects."),
        );
    };
    if items.is_empty() || items.len() > MOST_ASSETS {
        return refuse(
            "impairment_test.asset_invalid",
            format!("assets must be a list of 1 to {MOST_ASSETS} objects."),
        );
    }
    let all = items
        .iter()
        .enumerate()
        .map(|(i, v)| asset(v, i + 1))
        .collect::<Result<Vec<_>>>()?;
    if all.iter().filter(|a| a.goodwill).count() > 1 || all.iter().all(|a| a.carrying.is_zero()) {
        return refuse(
            "impairment_test.asset_invalid",
            "assets may hold one goodwill line and must carry an amount above nil in total.",
        );
    }
    if all.iter().all(|a| a.goodwill) {
        return refuse(
            "impairment_test.asset_invalid",
            "Goodwill does not generate cash flows independently (AASB 136 para 81): list the unit's other assets beside the goodwill line.",
        );
    }
    Ok(all)
}

/// Each year's cash flow's present value, and the terminal value's
/// (AASB 136 paras 31(b), 33 and 39; one rate, para A21).
fn present_values(
    flows: &[Decimal],
    terminal: &Decimal,
    rate: &Decimal,
) -> Calc<(Vec<Decimal>, Decimal)> {
    let one_plus = d("1").add(rate)?;
    let mut factor = d("1");
    let mut out = Vec::with_capacity(flows.len());
    for flow in flows {
        factor = factor.mul(&one_plus)?;
        out.push(flow.div(&factor)?);
    }
    Ok((out, terminal.div(&factor)?))
}

fn total<'a>(values: impl Iterator<Item = &'a Decimal>) -> Calc<Decimal> {
    let mut sum = d("0");
    for v in values {
        sum = sum.add(v)?;
    }
    Ok(sum)
}

/// How far each non-goodwill asset may be reduced: to its floor or nil
/// (para 105); nil for goodwill, which `allocate` reduces first.
fn room(all: &[Asset]) -> Calc<Vec<Decimal>> {
    all.iter()
        .map(|a| {
            let cap = a.carrying.sub(&a.floor)?;
            Ok(if a.goodwill || !cap.gt(&d("0"))? {
                d("0")
            } else {
                cap
            })
        })
        .collect()
}

/// One round of para 105: `left` spread over `active` by carrying amount.
/// The assets it would take to their floor or below are returned and their
/// shares left as they were; with none, the shares are added.
fn spread(
    all: &[Asset],
    caps: &[Decimal],
    active: &[usize],
    share: &mut [Decimal],
    left: &Decimal,
) -> Calc<Vec<usize>> {
    let base = total(active.iter().map(|&i| &all[i].carrying))?;
    let mut next = Vec::with_capacity(active.len());
    let mut over = Vec::new();
    for &i in active {
        let with = share[i].add(&left.mul(&all[i].carrying)?.div(&base)?)?;
        if !with.lt(&caps[i])? {
            over.push(i);
        }
        next.push(with);
    }
    if over.is_empty() {
        for (k, &i) in active.iter().enumerate() {
            share[i] = next[k].clone();
        }
    }
    Ok(over)
}

/// Shares of `rest` for the non-goodwill assets by carrying amount, none
/// below its floor, the excess passed pro rata to the others (para 105); the
/// shares unrounded, which assets reached their floor, and what is left.
fn pro_rata(all: &[Asset], rest: &Decimal) -> Calc<(Vec<Decimal>, Vec<bool>, Decimal)> {
    let caps = room(all)?;
    let mut share = vec![d("0"); all.len()];
    let mut capped = vec![false; all.len()];
    let mut active = Vec::new();
    for (i, cap) in caps.iter().enumerate() {
        if cap.gt(&d("0"))? {
            active.push(i);
        }
    }
    loop {
        let left = rest.sub(&total(share.iter())?)?;
        if !left.gt(&d("0"))? {
            return Ok((share, capped, d("0")));
        }
        if active.is_empty() {
            return Ok((share, capped, left));
        }
        let over = spread(all, &caps, &active, &mut share, &left)?;
        if over.is_empty() {
            return Ok((share, capped, d("0")));
        }
        for i in over {
            share[i] = caps[i].clone();
            capped[i] = true;
            active.retain(|&j| j != i);
        }
    }
}

/// The loss's allocation in cents (para 104), and what is left once every
/// asset is at its floor (para 62).
fn allocate(all: &[Asset], loss: &Decimal) -> Calc<(Vec<Decimal>, Decimal)> {
    let mut out = vec![d("0.00"); all.len()];
    let mut rest = loss.clone();
    if let Some(g) = all.iter().position(|a| a.goodwill) {
        let take = if loss.lt(&all[g].carrying)? {
            loss.clone()
        } else {
            all[g].carrying.clone()
        };
        rest = rest.sub(&take)?;
        out[g] = take;
    }
    let (share, capped, left) = pro_rata(all, &rest)?;
    let (mut running, mut shown) = (d("0"), d("0.00"));
    for i in (0..all.len()).filter(|&i| !all[i].goodwill) {
        if capped[i] {
            out[i] = cents(&share[i])?;
        } else {
            running = running.add(&share[i])?;
            let now = cents(&running)?;
            out[i] = now.sub(&shown)?;
            shown = now;
        }
    }
    Ok((out, cents(&left)?))
}

fn shown_assets(all: &[Asset], losses: &[Decimal]) -> Calc<Value> {
    let mut rows = Vec::with_capacity(all.len());
    for (a, loss) in all.iter().zip(losses) {
        let mut row = Map::new();
        row.insert("label".into(), json!(a.label));
        row.insert("goodwill".into(), json!(a.goodwill));
        row.insert("carrying_amount".into(), json!(a.carrying.to_string()));
        row.insert("impairment_loss".into(), json!(loss.to_string()));
        row.insert(
            "carrying_amount_after".into(),
            json!(a.carrying.sub(loss)?.to_string()),
        );
        rows.push(Value::Object(row));
    }
    Ok(Value::Array(rows))
}

fn shown_years(flows: &[Decimal], pv: &[Decimal]) -> Calc<Value> {
    let mut rows = Vec::with_capacity(flows.len());
    for (t, (flow, p)) in flows.iter().zip(pv).enumerate() {
        rows.push(json!({"year": t + 1, "cash_flow": flow.to_string(), "present_value": cents(p)?.to_string()}));
    }
    Ok(Value::Array(rows))
}

/// The cash flows, terminal value, rate, fair value and assets, as read.
type Inputs = (Vec<Decimal>, Decimal, Decimal, Option<Decimal>, Vec<Asset>);

/// The scope's refusals, then the inputs read.
fn read(a: &ImpairmentTestArgs) -> Result<Inputs> {
    if flag(a.revalued, "revalued")? {
        return refuse(
            "impairment_test.revalued",
            "A revalued asset's impairment loss is a revaluation decrease under its own Standard (AASB 136 para 60), which this worksheet does not apply.",
        );
    }
    let all = assets(a.assets)?;
    if flag(a.non_controlling_interest, "non_controlling_interest")?
        && all.iter().any(|x| x.goodwill)
    {
        return refuse(
            "impairment_test.non_controlling_interest",
            "A unit with goodwill and a non-controlling interest is tested with the goodwill grossed up and allocated under AASB 136 Appendix C, which this worksheet does not apply.",
        );
    }
    let flows = cash_flows(a.cash_flows)?;
    let terminal = a
        .terminal_value
        .map_or_else(|| Ok(d("0.00")), signed_money)?;
    let rate = discount_rate(a.discount_rate)?;
    let fair = a.fair_value_less_costs_of_disposal.map(money).transpose()?;
    Ok((flows, terminal, rate, fair, all))
}

/// The measured figures, rounded to the cent.
struct Measured {
    pv: Vec<Decimal>,
    tv: Decimal,
    viu: Decimal,
    recoverable: Decimal,
    carrying: Decimal,
    loss: Decimal,
    losses: Vec<Decimal>,
    unallocated: Decimal,
}

/// Paras 18, 59, 104 and 105 on the stated figures.
fn measure(
    flows: &[Decimal],
    terminal: &Decimal,
    rate: &Decimal,
    fair: Option<&Decimal>,
    all: &[Asset],
) -> Calc<Measured> {
    let (pv, tv) = present_values(flows, terminal, rate)?;
    let viu = cents(&tv.add(&total(pv.iter())?)?)?;
    let recoverable = match fair {
        Some(f) if f.gt(&viu)? => f.clone(),
        _ => viu.clone(),
    };
    let carrying = total(all.iter().map(|x| &x.carrying))?;
    let gap = carrying.sub(&recoverable)?;
    let loss = if gap.gt(&d("0"))? { gap } else { d("0.00") };
    let (losses, unallocated) = allocate(all, &loss)?;
    Ok(Measured {
        pv,
        tv,
        viu,
        recoverable,
        carrying,
        loss,
        losses,
        unallocated,
    })
}

/// The result's amounts: the goodwill and other assets' shares of the loss.
fn amounts(
    m: &Measured,
    all: &[Asset],
    fair: Option<&Decimal>,
) -> Calc<Vec<(&'static str, Decimal)>> {
    let goodwill = total(
        all.iter()
            .zip(&m.losses)
            .filter(|(x, _)| x.goodwill)
            .map(|(_, l)| l),
    )?;
    let others = m.loss.sub(&goodwill)?.sub(&m.unallocated)?;
    let mut out = vec![("value_in_use", m.viu.clone())];
    if let Some(f) = fair {
        out.push(("fair_value_less_costs_of_disposal", f.clone()));
    }
    out.extend([
        ("recoverable_amount", m.recoverable.clone()),
        ("carrying_amount", m.carrying.clone()),
        ("impairment_loss", m.loss.clone()),
        ("goodwill_impairment", goodwill),
        ("other_assets_impairment", others),
        ("unallocated", m.unallocated.clone()),
    ]);
    Ok(out)
}

/// AASB 136 for one asset or cash-generating unit at stated figures.
pub fn impairment_test(a: &ImpairmentTestArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let (flows, terminal, rate, fair, all) = read(a)?;
    let m = precise(|| Ok(measure(&flows, &terminal, &rate, fair.as_ref(), &all)?))?;
    let amounts = precise(|| Ok(amounts(&m, &all, fair.as_ref())?))?;
    let mut out = result(KIND, &year, amounts, vec![])?;
    out["years"] = precise(|| Ok(shown_years(&flows, &m.pv)?))?;
    out["terminal_present_value"] = json!(precise(|| Ok(cents(&m.tv)?))?.to_string());
    out["assets"] = precise(|| Ok(shown_assets(&all, &m.losses)?))?;
    Ok(out)
}
