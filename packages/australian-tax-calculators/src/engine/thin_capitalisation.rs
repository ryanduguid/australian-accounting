// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `thin-capitalisation`: the debt deductions a company that is a general
//! class investor has disallowed under ITAA 1997 Subdivision 820-AA, by the
//! fixed ratio test (ss 820-50(1)(a), 820-51(1) and 820-52), the group ratio
//! test (ss 820-50(1)(b), 820-51(2) and 820-53 to 820-55) or the third party
//! debt test (ss 820-50(1)(c) and 820-427A), each disallowed deduction's share
//! (s 820-50(2)) and the special deduction for earlier fixed ratio test
//! disallowed amounts (ss 820-56 to 820-59). Exemptions: ss 820-35, 820-37 and
//! 820-39. Read on 26 November 2026 in ITAA 1997 compilation 268
//! (C2026C00471) with the application provisions, Treasury Laws Amendment
//! (Making Multinationals Pay Their Fair Share—Integrity and Transparency) Act
//! 2024 Schedule 2 items 144 and 146: income years starting on or after
//! 1 July 2023, and the older law for Australian plantation forestry entities.
//! Financial entities (non-ADI) and ADIs take Subdivisions 820-B to 820-E
//! (`older`).

mod carry;
mod inputs;
mod older;

use serde_json::{Map, Value, json};

use inputs::*;

use super::{Arg, Result, Step, d, derived_result, flag, precise, rate, refuse, scope, step};
use crate::decimal::{Calc, Decimal, Rounding};

const KIND: &str = "thin_capitalisation";
const MOST_DEDUCTIONS: usize = 200;

/// The worksheet's inputs, as the caller gave them; `given` holds the
/// optional inputs present, by name.
pub struct ThinCapitalisationArgs<'a> {
    pub test: &'a Arg,
    pub entity_class: &'a Arg,
    pub plantation_forestry_entity: &'a Arg,
    pub special_purpose_entity: &'a Arg,
    pub outward_investor_only: &'a Arg,
    pub associate_inclusive_debt_deductions: &'a Arg,
    pub debt_deductions: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub given: &'a [(&'static str, Arg)],
}

impl ThinCapitalisationArgs<'_> {
    fn get(&self, key: &str) -> Option<&Arg> {
        self.given.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }
}

fn total<'a>(values: impl Iterator<Item = &'a Decimal>) -> Calc<Decimal> {
    let mut sum = d("0");
    for v in values {
        sum = sum.add(v)?;
    }
    Ok(sum)
}

/// A step adding `terms`, written as numbers; `0` when there are none.
fn sum_step(id: &'static str, authority: &'static str, terms: &[&Decimal]) -> Calc<Step> {
    let value = total(terms.iter().copied())?;
    let formula = if terms.is_empty() {
        "0".to_string()
    } else {
        terms
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" + ")
    };
    Ok(step(id, authority, &formula, &[], value))
}

fn max0(v: Decimal) -> Calc<Decimal> {
    Ok(if v.gt(&d("0"))? { v } else { d("0") })
}

const DD: &str = "s 820-40 and s 820-50(3)(a): the year's debt deductions, after any disallowed under Subdivision 820-EAA (s 820-31)";
const NDD: &str = "s 820-50(3): debt deductions less the assessable amounts in s 820-50(3)(b) (interest, amounts in the nature of or economically equivalent to interest and the amounts in s 820-50(3)(b)(ii) and (iii)); may be negative";
const EBITDA: &str = "s 820-52(1), (1A), (2), (3) and (10): taxable income (losses of earlier years deducted, a tax loss negative) less franking credits, associate dividends and R&D notional deductions, plus net debt deductions, Division 40 and 43, forestry and s 70-120 deductions and any excess tax EBITDA amount (s 820-60), nil if negative";
const DISALLOWED: &str = "s 820-50(1): the amount by which net debt deductions (fixed ratio and group ratio tests) or debt deductions (third party debt test) exceed the earnings limit";
const ALLOWED: &str = "s 820-50: debt deductions less the total disallowed amount";

/// The year's debt deductions and, given the s 820-50(3)(b) amounts, net debt
/// deductions.
fn opening(items: &[Deduction], interest: Option<&Decimal>) -> Calc<Vec<Step>> {
    let amounts: Vec<&Decimal> = items.iter().map(|x| &x.amount).collect();
    let dd = sum_step("debt_deductions", DD, &amounts)?;
    let Some(i) = interest else {
        return Ok(vec![dd]);
    };
    let ndd = dd.value.sub(i)?;
    let net = step(
        "net_debt_deductions",
        NDD,
        "debt_deductions - interest_and_equivalent_income",
        &[&dd.value, i],
        ndd,
    );
    Ok(vec![dd, net])
}

/// Tax EBITDA (s 820-52(1)) from the stated lines present: taxable income,
/// the subtracted lines, net debt deductions, then the added lines.
fn tax_ebitda(taxable: &Decimal, lines: &[(&'static str, Decimal)], ndd: &Decimal) -> Calc<Step> {
    let mut formula = String::from("max(taxable_income");
    let mut operands = vec![taxable.clone()];
    let mut value = taxable.clone();
    for (name, v) in lines.iter().filter(|(n, _)| SUBTRACTED.contains(n)) {
        formula.push_str(&format!(" - {name}"));
        operands.push(v.clone());
        value = value.sub(v)?;
    }
    formula.push_str(" + net_debt_deductions");
    operands.push(ndd.clone());
    value = value.add(ndd)?;
    for (name, v) in lines.iter().filter(|(n, _)| ADDED.contains(n)) {
        formula.push_str(&format!(" + {name}"));
        operands.push(v.clone());
        value = value.add(v)?;
    }
    formula.push_str(", 0)");
    let refs: Vec<&Decimal> = operands.iter().collect();
    Ok(step("tax_ebitda", EBITDA, &formula, &refs, max0(value)?))
}

/// Taxable income and the stated tax EBITDA lines present.
fn ebitda_lines(a: &ThinCapitalisationArgs) -> Result<(Decimal, Vec<(&'static str, Decimal)>)> {
    let taxable = required(
        a,
        "taxable_income",
        "tax EBITDA starts from it (s 820-52(1)(a))",
    )?;
    let mut lines = Vec::new();
    for name in SUBTRACTED.into_iter().chain(ADDED) {
        let v = if name == "division_40_43_deductions" {
            Some(required(
                a,
                name,
                "tax EBITDA adds it (s 820-52(1)(c)(ii)); state 0.00 if none",
            )?)
        } else {
            optional_amount(a, name)?
        };
        if let Some(v) = v {
            lines.push((name, v));
        }
    }
    Ok((taxable, lines))
}

fn interest(a: &ThinCapitalisationArgs) -> Result<Decimal> {
    required(
        a,
        "interest_and_equivalent_income",
        "net debt deductions subtract it (s 820-50(3)(b)); state 0.00 if none",
    )
}

/// The disallowed amount's share of each debt deduction (s 820-50(2)), in
/// cents rounded cumulatively so the shares add to the total.
fn shares(items: &[Deduction], disallowed: &Decimal, dd: &Decimal) -> Calc<Value> {
    let whole = disallowed.quantize(-2, Rounding::HalfUp)?;
    let (mut running, mut shown) = (d("0"), d("0.00"));
    let mut rows = Vec::with_capacity(items.len());
    for x in items {
        let share = if dd.is_zero() {
            d("0.00")
        } else {
            running = running.add(&whole.mul(&x.amount)?.div(dd)?)?;
            let now = running.quantize(-2, Rounding::HalfUp)?;
            let share = now.sub(&shown)?;
            shown = now;
            share
        };
        let amount = x.amount.quantize(-2, Rounding::HalfUp)?;
        let mut row = Map::new();
        row.insert("label".into(), json!(x.label));
        row.insert("amount".into(), json!(amount.to_string()));
        if let Some(t) = x.third_party {
            row.insert("third_party".into(), json!(t));
        }
        row.insert("disallowed".into(), json!(share.to_string()));
        row.insert("allowed".into(), json!(amount.sub(&share)?.to_string()));
        rows.push(Value::Object(row));
    }
    Ok(Value::Array(rows))
}

fn value_of<'a>(steps: &'a [Step], id: &str) -> &'a Decimal {
    &steps
        .iter()
        .find(|s| s.id == id)
        .expect("an earlier step")
        .value
}

/// The disallowed and allowed steps after an earnings limit.
fn closing(steps: &mut Vec<Step>, limit_id: &'static str, against: &'static str) -> Calc<()> {
    let base = value_of(steps, against).clone();
    let limit = value_of(steps, limit_id).clone();
    let dd = value_of(steps, "debt_deductions").clone();
    let disallowed = max0(base.sub(&limit)?)?;
    let formula = format!("max({against} - {limit_id}, 0)");
    steps.push(step(
        "total_disallowed_amount",
        DISALLOWED,
        &formula,
        &[&base, &limit],
        disallowed.clone(),
    ));
    steps.push(step(
        "allowed_debt_deductions",
        ALLOWED,
        "debt_deductions - total_disallowed_amount",
        &[&dd, &disallowed],
        dd.sub(&disallowed)?,
    ));
    Ok(())
}

struct Worked {
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
    carried: Option<Value>,
}

fn fixed_ratio(a: &ThinCapitalisationArgs, items: &[Deduction], year: &str) -> Result<Worked> {
    let interest = interest(a)?;
    let (taxable, lines) = ebitda_lines(a)?;
    let carried = carry::read(a.get("frt_disallowed_amounts"), year)?;
    precise(|| {
        let mut steps = opening(items, Some(&interest))?;
        let ebitda = tax_ebitda(&taxable, &lines, &steps[1].value)?;
        let limit = d("0.3").mul(&ebitda.value)?;
        let limit = step(
            "fixed_ratio_earnings_limit",
            "s 820-51(1): 30% of tax EBITDA",
            "fixed_ratio x tax_ebitda",
            &[&d("0.3"), &ebitda.value],
            limit,
        );
        steps.extend([ebitda, limit]);
        closing(
            &mut steps,
            "fixed_ratio_earnings_limit",
            "net_debt_deductions",
        )?;
        let (special, rows) = carry::apply(&steps, &carried)?;
        steps.push(special);
        Ok(Worked {
            steps,
            rates: vec![rate("fixed_ratio", "0.3")],
            carried: Some(rows),
        })
    })
}

/// The group ratio test (ss 820-50(1)(b) and 820-51(2)).
fn group_ratio(a: &ThinCapitalisationArgs, items: &[Deduction]) -> Result<Worked> {
    let interest = interest(a)?;
    let (taxable, lines) = ebitda_lines(a)?;
    let g = gr_group(a)?;
    precise(|| {
        let mut steps = opening(items, Some(&interest))?;
        let ebitda = tax_ebitda(&taxable, &lines, &steps[1].value)?;
        let gr = step(
            "gr_group_ebitda",
            "s 820-55(2) and (3): the GR group's net profit before tax, adjusted net third party interest expense and depreciation and amortisation, each member's negative entity EBITDA disregarded",
            &g.formula,
            &[&g.profit, &g.adjusted, &g.da],
            g.ebitda.clone(),
        );
        let ratio = g.expense.div(&g.ebitda)?;
        let limit = step(
            "group_ratio_earnings_limit",
            "s 820-51(2): the group ratio (s 820-53(1): GR group net third party interest expense over GR group EBITDA) times tax EBITDA",
            "group_ratio x tax_ebitda",
            &[&ratio, &ebitda.value],
            ratio.mul(&ebitda.value)?,
        );
        steps.extend([ebitda, gr, limit]);
        closing(
            &mut steps,
            "group_ratio_earnings_limit",
            "net_debt_deductions",
        )?;
        Ok(Worked {
            steps,
            rates: vec![rate("group_ratio", ratio)],
            carried: None,
        })
    })
}

fn third_party_debt(items: &[Deduction]) -> Result<Worked> {
    precise(|| {
        let mut steps = opening(items, None)?;
        let qualifying: Vec<&Decimal> = items
            .iter()
            .filter(|x| x.third_party == Some(true))
            .map(|x| &x.amount)
            .collect();
        let pushed = sum_step(
            "third_party_earnings_limit",
            "s 820-427A(1) and (2): the debt deductions attributable to debt interests satisfying the third party debt conditions",
            &qualifying,
        );
        steps.push(pushed?);
        closing(&mut steps, "third_party_earnings_limit", "debt_deductions")?;
        Ok(Worked {
            steps,
            rates: vec![],
            carried: None,
        })
    })
}

fn exempt(items: &[Deduction], why: &'static str) -> Result<Worked> {
    precise(|| {
        let mut steps = opening(items, None)?;
        let dd = steps[0].value.clone();
        steps.push(step("total_disallowed_amount", why, "0", &[], d("0")));
        steps.push(step(
            "allowed_debt_deductions",
            ALLOWED,
            "debt_deductions - total_disallowed_amount",
            &[&dd, &d("0")],
            dd.clone(),
        ));
        Ok(Worked {
            steps,
            rates: vec![
                rate("de_minimis_threshold", "2000000"),
                rate("assets_threshold", "0.9"),
            ],
            carried: None,
        })
    })
}

/// Subdivision 820-AA for one company and one income year at stated figures.
pub fn thin_capitalisation(a: &ThinCapitalisationArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let t = test(a.test)?;
    let c = class(a.entity_class)?;
    check_test(t, c)?;
    if flag(a.plantation_forestry_entity, "plantation_forestry_entity")? {
        return refuse(
            "thin_capitalisation.plantation_forestry",
            "An Australian plantation forestry entity stays under Division 820 as it stood before the 2024 amendments (Treasury Laws Amendment (Making Multinationals Pay Their Fair Share—Integrity and Transparency) Act 2024 Schedule 2 item 146), which this worksheet does not apply.",
        );
    }
    let items = deductions(a.debt_deductions, t)?;
    check_unused(a, t, c)?;
    let own = precise(|| Ok(total(items.iter().map(|x| &x.amount))?))?;
    if matches!(c, Class::InwardFinancial | Class::InwardAdi)
        && flag(a.outward_investor_only, "outward_investor_only")?
    {
        return fact(
            "outward_investor_only cannot be true for an inward investing entity (s 820-37(1)(b)).",
        );
    }
    let (reason, ratio) = exemption(a, c != Class::NotCovered, &own)?;
    let worked = match (reason, t) {
        (None, _) if c.financial() || c.adi() => older::work(a, c, t, &items)?,
        (Some(_), _) if a.get("frt_disallowed_amounts").is_some() => {
            return refuse(
                "thin_capitalisation.carry_forward_unsettled",
                "Whether s 820-56 allows a deduction for earlier fixed ratio test disallowed amounts in a year Subdivision 820-AA disallows nothing is not settled by the sources read, so the worksheet does not decide it.",
            );
        }
        (Some(why), _) => exempt(&items, why)?,
        (None, Test::Fixed) => fixed_ratio(a, &items, &year)?,
        (None, Test::Group) => group_ratio(a, &items)?,
        (None, Test::ThirdParty) => third_party_debt(&items)?,
        (None, Test::MaxDebt | Test::MinCapital) => {
            unreachable!("check_test keeps these to financial entities and ADIs")
        }
    };
    let disallowed = value_of(&worked.steps, "total_disallowed_amount").clone();
    let mut out = derived_result(KIND, &year, worked.steps, worked.rates)?;
    out["test"] = json!(a.test.text());
    out["exemption"] = json!(reason);
    if let Some(r) = ratio {
        out["australian_assets_ratio"] = json!(r.to_string());
    }
    out["debt_deductions"] = precise(|| Ok(shares(&items, &disallowed, &own)?))?;
    if let Some(rows) = worked.carried {
        out["frt_disallowed_amounts"] = rows;
    }
    Ok(out)
}
