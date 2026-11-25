// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `book-to-tax`: a company's accounting profit reconciled to taxable income
//! through the permanent and temporary differences the preparer states, the
//! company's tax losses deducted (ITAA 1997 ss 4-15, 36-10, 36-17 and 36-20)
//! and the inputs the `deferred_tax` worksheet takes from that reconciliation.
//! The sections are ITAA 1997 as the ATO's legal database shows it, read on
//! 26 November 2026: since 2020-21 only s 36-17(1)'s note 2 (Act No. 92 of
//! 2020, No. 8 of 2022 and No. 71 of 2026) and s 4-15(2) item 1's wording
//! (No. 7 of 2019) changed. AASB 112 paras 5 and 17 (September 2023
//! compilation) and the ATO's Company tax return 2025 instructions,
//! worksheet 2 (item 6 label T to item 7 label T, last updated 29 May 2025),
//! were read the same day.

use serde_json::{Map, Value, json};

use super::{
    Arg, Result, Step, derived_result, flag, money, precise, refuse, scope, signed_money, step,
    steps_from,
};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "book_to_tax";
const MOST: usize = 500;

/// The worksheet's inputs, as the caller gave them.
pub struct BookToTaxArgs<'a> {
    pub accounting_profit: &'a Arg,
    pub adjustments: &'a Arg,
    pub net_exempt_income: &'a Arg,
    pub opening_tax_losses: &'a Arg,
    pub tax_losses_deducted: &'a Arg,
    pub loss_carry_back: &'a Arg,
    pub excess_franking_offsets: &'a Arg,
    pub subdivision_165_b: &'a Arg,
    pub tax_consolidated_group: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One reconciling line: a permanent or temporary difference that adds to or
/// subtracts from accounting profit.
struct Line {
    label: String,
    temporary: bool,
    add: bool,
    amount: Decimal,
}

fn invalid<T>(why: String) -> Result<T> {
    refuse("book_to_tax.adjustment_invalid", why)
}

fn line(n: usize, value: &Value) -> Result<Line> {
    let Some(o) = value.as_object() else {
        return invalid(format!("adjustment {n} must be an object."));
    };
    if let Some(k) = o
        .keys()
        .find(|k| !matches!(k.as_str(), "label" | "difference" | "effect" | "amount"))
    {
        return invalid(format!(
            "adjustment {n}: {k} is not taken; give label, difference, effect and amount."
        ));
    }
    let label = o
        .get("label")
        .and_then(Value::as_str)
        .filter(|l| !l.trim().is_empty() && l.chars().count() <= 200);
    let temporary = match o.get("difference").and_then(Value::as_str) {
        Some("permanent") => Some(false),
        Some("temporary") => Some(true),
        _ => None,
    };
    let add = match o.get("effect").and_then(Value::as_str) {
        Some("add") => Some(true),
        Some("subtract") => Some(false),
        _ => None,
    };
    let amount = o
        .get("amount")
        .and_then(Value::as_str)
        .and_then(|t| Decimal::parse_input(t).ok())
        .and_then(|v| money(&Arg::Decimal(v)).ok());
    match (label, temporary, add, amount) {
        (Some(l), Some(t), Some(a), Some(v)) => Ok(Line {
            label: l.to_string(),
            temporary: t,
            add: a,
            amount: v,
        }),
        _ => invalid(format!(
            "adjustment {n} needs a label of 1 to 200 characters, difference permanent or temporary, effect add or subtract and an amount from 0, at most 2 dp and 1e12."
        )),
    }
}

fn lines(given: &Arg) -> Result<Vec<Line>> {
    match given {
        Arg::Json(Value::Array(list)) if list.len() <= MOST => list
            .iter()
            .enumerate()
            .map(|(i, v)| line(i + 1, v))
            .collect(),
        _ => invalid(format!(
            "adjustments must be a list of 0 to {MOST} objects."
        )),
    }
}

const CARRY_BACK: &str = "A tax loss carried back under Division 160 (ITAA 1997 s 36-17(1) note 2) is used in the earlier year and leaves the losses carried forward; this worksheet does not apply the loss carry back choice.";
const FRANKING: &str = "With excess franking offsets the company must choose a nil or limited amount of tax losses (ITAA 1997 s 36-17(5)) and its tax loss is worked out under s 36-55; this worksheet does not apply either.";
const CONTINUITY: &str = "A company that fails to maintain continuity of ownership and control in the year and does not satisfy the business continuity test works out its taxable income under Subdivision 165-B (ITAA 1997 s 4-15(2) item 1), which this worksheet does not apply.";
const CONSOLIDATED: &str = "A member of a consolidated group has its taxable income worked out under Part 3-90 (ITAA 1997 s 4-15(2) item 1B), which this worksheet does not apply.";

/// The cases the worksheet refuses, each stated as a flag.
fn excluded(a: &BookToTaxArgs) -> Result<()> {
    if flag(a.loss_carry_back, "loss_carry_back")? {
        return refuse("book_to_tax.loss_carry_back", CARRY_BACK);
    }
    if flag(a.excess_franking_offsets, "excess_franking_offsets")? {
        return refuse("book_to_tax.excess_franking_offsets", FRANKING);
    }
    if flag(a.subdivision_165_b, "subdivision_165_b")? {
        return refuse("book_to_tax.subdivision_165_b", CONTINUITY);
    }
    if flag(a.tax_consolidated_group, "tax_consolidated_group")? {
        return refuse("book_to_tax.tax_consolidated_group", CONSOLIDATED);
    }
    Ok(())
}

/// The 4 line totals: (id, temporary, adds, authority).
const GROUPS: [(&str, bool, bool, &str); 4] = [
    (
        "permanent_additions",
        false,
        true,
        "AASB 112 para 5; ATO Company tax return 2025 instructions, worksheet 2: amounts added to accounting profit that never reverse, such as expenses that are never deductible and assessable income outside profit",
    ),
    (
        "permanent_subtractions",
        false,
        false,
        "AASB 112 para 5; ATO Company tax return 2025 instructions, worksheet 2: amounts taken from accounting profit that never reverse, such as exempt and non-assessable non-exempt income and deductions with no expense in profit",
    ),
    (
        "temporary_additions",
        true,
        true,
        "AASB 112 paras 5 and 17: income or expense in accounting profit in another period than in taxable income, added this year (an expense not yet deductible, income assessed before it is recognised)",
    ),
    (
        "temporary_subtractions",
        true,
        false,
        "AASB 112 paras 5 and 17: income or expense in accounting profit in another period than in taxable income, subtracted this year (a deduction ahead of the expense, income recognised before it is assessed)",
    ),
];

/// The 4 line totals, each a sum written out term by term.
fn totals(all: &[Line]) -> Result<Vec<Step>> {
    let mut out = Vec::with_capacity(4);
    for (id, temporary, add, authority) in GROUPS {
        let chosen: Vec<&Decimal> = all
            .iter()
            .filter(|l| l.temporary == temporary && l.add == add)
            .map(|l| &l.amount)
            .collect();
        let mut sum = Decimal::from_int(0);
        for v in &chosen {
            sum = sum.add(v)?;
        }
        let terms: Vec<String> = chosen.iter().map(|v| v.to_string()).collect();
        let formula = if terms.is_empty() {
            "0".to_string()
        } else {
            terms.join(" + ")
        };
        out.push(step(id, authority, &formula, &[], sum));
    }
    Ok(out)
}

/// The facts read once the scope and its refusals pass.
struct Facts {
    year: String,
    lines: Vec<Line>,
    profit: Decimal,
    exempt: Decimal,
    opening: Decimal,
    chosen: Decimal,
}

fn read(a: &BookToTaxArgs) -> Result<Facts> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    excluded(a)?;
    Ok(Facts {
        year,
        lines: lines(a.adjustments)?,
        profit: signed_money(a.accounting_profit)?,
        exempt: money(a.net_exempt_income)?,
        opening: money(a.opening_tax_losses)?,
        chosen: money(a.tax_losses_deducted)?,
    })
}

fn max0(v: Decimal) -> Result<Decimal> {
    Ok(if v.lt(&Decimal::from_int(0))? {
        Decimal::from_int(0)
    } else {
        v
    })
}

fn lesser(a: Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.lt(&a)? { b.clone() } else { a })
}

/// The reconciliation's later steps: (id, authority, formula).
const RULES: [(&str, &str, &str); 8] = [
    (
        "net_income_before_tax_losses",
        "ITAA 1997 s 4-15(1) steps 1 to 3 and s 36-10(1) and (2): assessable income less deductions other than tax losses, reached from accounting profit through the reconciling lines (item 6 label T to item 7)",
        "accounting_profit + permanent_additions - permanent_subtractions + temporary_additions - temporary_subtractions",
    ),
    (
        "excess_deductions",
        "ITAA 1997 ss 36-10(1) and (2) and 36-17(4): the deductions other than tax losses above assessable income",
        "max(0 - net_income_before_tax_losses, 0)",
    ),
    (
        "net_exempt_income_left",
        "ITAA 1997 s 36-17(4)(a): net exempt income less any excess of deductions over assessable income",
        "max(net_exempt_income - excess_deductions, 0)",
    ),
    (
        "tax_losses_against_exempt_income",
        "ITAA 1997 s 36-17(3)(a) and (4)(b): earlier tax losses are first deducted from net exempt income, whatever the company chooses",
        "min(net_exempt_income_left, opening_tax_losses)",
    ),
    (
        "taxable_income",
        "ITAA 1997 ss 4-15(1) and 36-17(2) and (3)(b): the excess of assessable income over deductions, less the tax losses the company chooses to deduct from it (item 7 label R); nil where deductions are the greater",
        "max(net_income_before_tax_losses, 0) - tax_losses_deducted",
    ),
    (
        "tax_loss_for_year",
        "ITAA 1997 s 36-10(3) and (4): the excess of deductions over assessable income, less net exempt income, is the year's tax loss",
        "max(excess_deductions - net_exempt_income, 0)",
    ),
    (
        "closing_tax_losses",
        "ITAA 1997 ss 36-17(7) and 960-20: earlier losses not yet utilised, with the year's tax loss, carried to later years (deferred-tax's unused_tax_losses)",
        "opening_tax_losses - tax_losses_against_exempt_income - tax_losses_deducted + tax_loss_for_year",
    ),
    (
        "net_taxable_temporary_difference_change",
        "AASB 112 paras 5 and 17: a temporary addition raises deductible or lowers taxable temporary differences and a subtraction does the reverse, so deferred-tax's items should show this change in net taxable temporary differences",
        "temporary_subtractions - temporary_additions",
    ),
];

/// The most tax losses the company can choose to deduct (s 36-17(2) and
/// (3)(b)), refusing a stated choice above it.
fn check_choice(f: &Facts, net: &Decimal, absorbed: &Decimal) -> Result<()> {
    let room = lesser(max0(net.clone())?, &f.opening.sub(absorbed)?)?;
    if f.chosen.gt(&room)? {
        return refuse(
            "book_to_tax.losses_deducted_invalid",
            format!(
                "tax_losses_deducted, {}, is more than the company can choose: {room}, the lesser of the excess of assessable income over deductions and the earlier losses left after net exempt income absorbs them (ITAA 1997 s 36-17(2) and (3)).",
                f.chosen
            ),
        );
    }
    Ok(())
}

/// Taxable income, the losses used and carried, and the temporary change.
fn losses(f: &Facts, t: &[Step]) -> Result<Vec<Step>> {
    let [pa, ps, ta, ts] = [&t[0].value, &t[1].value, &t[2].value, &t[3].value];
    let net = f.profit.add(pa)?.sub(ps)?.add(ta)?.sub(ts)?;
    let excess = max0(Decimal::from_int(0).sub(&net)?)?;
    let left = max0(f.exempt.sub(&excess)?)?;
    let absorbed = lesser(left.clone(), &f.opening)?;
    check_choice(f, &net, &absorbed)?;
    let taxable = max0(net.clone())?.sub(&f.chosen)?;
    let loss = max0(excess.sub(&f.exempt)?)?;
    let closing = f.opening.sub(&absorbed)?.sub(&f.chosen)?.add(&loss)?;
    let change = ts.sub(ta)?;
    let rows: [(&[&Decimal], Decimal); 8] = [
        (&[&f.profit, pa, ps, ta, ts], net.clone()),
        (&[&net], excess.clone()),
        (&[&f.exempt, &excess], left.clone()),
        (&[&left, &f.opening], absorbed.clone()),
        (&[&net, &f.chosen], taxable),
        (&[&excess, &f.exempt], loss.clone()),
        (&[&f.opening, &absorbed, &f.chosen, &loss], closing),
        (&[ts, ta], change),
    ];
    Ok(steps_from(&RULES, rows))
}

fn shown_lines(all: &[Line]) -> Value {
    Value::Array(
        all.iter()
            .map(|l| {
                let mut row = Map::new();
                row.insert("label".into(), json!(l.label));
                let kind = if l.temporary {
                    "temporary"
                } else {
                    "permanent"
                };
                row.insert("difference".into(), json!(kind));
                row.insert(
                    "effect".into(),
                    json!(if l.add { "add" } else { "subtract" }),
                );
                row.insert("amount".into(), json!(l.amount.to_string()));
                Value::Object(row)
            })
            .collect(),
    )
}

const LINK: &str = "deferred_tax takes these inputs: every permanent addition as non_deductible_expenses and every permanent subtraction as non_assessable_income, so its reconciliation explains taxable income when its items' net taxable temporary differences change by net_taxable_temporary_difference_change. Losses that net exempt income absorbs reduce unused tax losses without reducing taxable income, and its taxable_income_difference then shows them.";

fn cents(v: &Decimal) -> Result<String> {
    Ok(v.quantize(-2, Rounding::HalfUp)?.to_string())
}

/// The `deferred_tax` arguments this reconciliation settles.
fn link(out: &Value, f: &Facts) -> Result<Value> {
    let a = &out["amounts"];
    Ok(json!({
        "accounting_profit": precise(|| cents(&f.profit))?,
        "non_deductible_expenses": a["permanent_additions"],
        "non_assessable_income": a["permanent_subtractions"],
        "taxable_income": a["taxable_income"],
        "opening_unused_tax_losses": precise(|| cents(&f.opening))?,
        "unused_tax_losses": a["closing_tax_losses"],
    }))
}

/// A company's accounting profit reconciled to taxable income.
pub fn book_to_tax(a: &BookToTaxArgs) -> Result<Value> {
    let f = read(a)?;
    let steps = precise(|| {
        let mut steps = totals(&f.lines)?;
        let more = losses(&f, &steps)?;
        steps.extend(more);
        Ok(steps)
    })?;
    let mut out = derived_result(KIND, &f.year, steps, Vec::new())?;
    out["deferred_tax"] = link(&out, &f)?;
    out["adjustments"] = shown_lines(&f.lines);
    if let Some(w) = out["warnings"].as_array_mut() {
        w.push(json!(LINK));
    }
    Ok(out)
}
