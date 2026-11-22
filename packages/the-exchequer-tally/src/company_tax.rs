// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `company-tax-statement`: a company's taxable income after prior-year losses,
//! the tax on it at the `corporate-tax` rate, its tax offsets in the order of the
//! company tax return's calculation statement, and the amount due or refundable,
//! with the Division 160 loss carry back tax offset. Added after the Python port;
//! it reuses `Bre::corporate_tax` for every rate.
use crate::api::{array, bre, decimal, number, text};
use crate::carry_back;
use crate::decimal::{Decimal as D, Rounding};
use crate::engine::{Result, cents, d, q};
use serde_json::{Value, json};

/// 2020-21 to 2026-27, as income years ending 30 June.
pub const YEARS: std::ops::RangeInclusive<i64> = 2021..=2027;

pub fn refuse(reason: impl std::fmt::Display, provision: &str) -> String {
    format!("refused: {reason} ({provision})")
}
/// A non-negative amount in dollars and cents; omitted or null is nil.
pub fn money(v: &Value, key: &str) -> Result<D> {
    if v.get(key).is_none_or(Value::is_null) {
        return Ok(d("0"));
    }
    let x = decimal(v, key)?;
    if !x.is_finite() || x < d("0") || q(&x, Rounding::Down)? != x {
        return Err(format!(
            "{key} must be a non-negative amount with at most 2 decimal places"
        ));
    }
    Ok(x)
}
pub fn flag(v: &Value, key: &str) -> Result<bool> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        _ => Err(format!("{key} must be a boolean")),
    }
}
pub fn floor(x: D) -> D {
    if x < d("0") { d("0") } else { x }
}
pub fn least(a: D, b: D) -> D {
    if b < a { b } else { a }
}
pub fn amount(x: &D) -> Value {
    json!(x.fixed(2, false))
}

const OUT_OF_SCOPE: [(&str, &str, &str); 5] = [
    (
        "consolidated_group",
        "a head company of a consolidated or MEC group is not modelled",
        "Part 3-90 ITAA 1997",
    ),
    (
        "foreign_hybrid_or_cfc_income",
        "attributable income from a controlled foreign company or a foreign hybrid is not modelled",
        "outside this command's scope",
    ),
    (
        "foreign_resident",
        "a foreign resident company's taxable income and franking cap differ",
        "CTR 2026 label A; s 160-10(3) and (5) ITAA 1997",
    ),
    (
        "special_rate_company",
        "an RSA provider, PDF, life insurance company, non-profit company, recognised medium credit union or trustee has other rates",
        "Income Tax Rates Act 1986 ss 23(3) to (7), 23A",
    ),
    (
        "subdivision_165_b_applies",
        "a year of change of ownership or control without the business continuity test is worked out by periods",
        "Subdiv 165-B ITAA 1997",
    ),
];
fn scope(v: &Value, fy: i64) -> Result<()> {
    if !YEARS.contains(&fy) {
        return Err(refuse(
            format!("financial_year {fy} is outside 2021 (2020-21) to 2027 (2026-27)"),
            "years this command covers",
        ));
    }
    for (key, what, law) in OUT_OF_SCOPE {
        if flag(v, key)? {
            return Err(refuse(what, law));
        }
    }
    Ok(())
}

struct Loss {
    year: i64,
    amount: D,
    test: String,
    basis: &'static str,
    deductible: bool,
}
fn loss(e: &Value, fy: i64) -> Result<Loss> {
    let year = number(e, "loss_year")?;
    if year >= fy {
        return Err(refuse(
            format!("loss year {year} is not an earlier income year"),
            "s 36-17(1) ITAA 1997",
        ));
    }
    let test = text(e, "test")?;
    let (deductible, basis) = match test {
        "continuity_of_ownership" => (true, "s 165-12 ITAA 1997"),
        "same_business" => (true, "ss 165-13, 165-210 ITAA 1997"),
        "similar_business" if year >= 2016 => (true, "ss 165-13, 165-211 ITAA 1997"),
        "similar_business" => {
            return Err(refuse(
                format!(
                    "the similar business test does not apply to the loss year ending in {year}, which started before 1 July 2015"
                ),
                "s 165-211(1)(a) ITAA 1997",
            ));
        }
        "failed" => (false, "s 165-10 ITAA 1997: not deductible"),
        _ => {
            return Err(
                "test must be continuity_of_ownership, same_business, similar_business or failed"
                    .into(),
            );
        }
    };
    Ok(Loss {
        year,
        amount: money(e, "amount")?,
        test: test.into(),
        basis,
        deductible,
    })
}

/// Taxable income or tax loss after prior-year losses (ss 36-10, 36-17).
pub struct Income {
    pub taxable: D,
    pub tax_loss: D,
    pub nei: D,
    pub used: D,
    json: Value,
}
fn chosen(v: &Value, cap: &D, excess: &D) -> Result<D> {
    if v.get("loss_deduction_chosen").is_none_or(Value::is_null) {
        return Ok(cap.clone());
    }
    let c = money(v, "loss_deduction_chosen")?;
    if *excess <= d("0") && c > d("0") {
        return Err(refuse(
            "no choice of loss deduction exists when deductions are not less than assessable income",
            "s 36-17(4) ITAA 1997",
        ));
    }
    if c > *cap {
        return Err(format!(
            "loss_deduction_chosen {c} exceeds the {} the deductible losses allow",
            cap.fixed(2, false)
        ));
    }
    Ok(c)
}
fn income(v: &Value, fy: i64) -> Result<Income> {
    let mut losses = array(v, "prior_year_losses")?
        .iter()
        .map(|e| loss(e, fy))
        .collect::<Result<Vec<_>>>()?;
    losses.sort_by_key(|l| l.year);
    let assessable = money(v, "assessable_income")?;
    let deductions = money(v, "deductions")?;
    let nei = money(v, "net_exempt_income")?;
    let excess = assessable.sub(&deductions);
    let available = losses
        .iter()
        .filter(|l| l.deductible)
        .fold(d("0"), |t, l| t.add(&l.amount));
    // s 36-17(3)(a) and (4): net exempt income, less any shortfall, absorbs losses first.
    let nei_left = floor(nei.sub(&floor(d("0").sub(&excess))));
    let from_nei = least(nei_left, available.clone());
    let cap = least(floor(available.sub(&from_nei)), floor(excess.clone()));
    let choice = chosen(v, &cap, &excess)?;
    let shortfall = floor(d("0").sub(&excess));
    let used = from_nei.add(&choice);
    let json = income_json(
        &losses,
        &used,
        [&assessable, &deductions, &nei, &from_nei, &choice],
    );
    Ok(Income {
        taxable: floor(excess.sub(&choice)),
        tax_loss: floor(shortfall.sub(&nei)),
        nei,
        used,
        json,
    })
}
fn income_json(losses: &[Loss], used: &D, figures: [&D; 5]) -> Value {
    let mut left = used.clone();
    let rows: Vec<Value> = losses
        .iter()
        .map(|l| {
            let deducted = if l.deductible { least(l.amount.clone(), left.clone()) } else { d("0") };
            left = left.sub(&deducted);
            json!({"loss_year": l.year, "amount": amount(&l.amount), "test": l.test, "basis": l.basis,
                "deducted": amount(&deducted), "remaining": amount(&l.amount.sub(&deducted))})
        })
        .collect();
    let [assessable, deductions, nei, from_nei, choice] = figures;
    json!({"assessable_income": amount(assessable), "deductions": amount(deductions),
        "net_exempt_income": amount(nei), "losses_deducted_from_net_exempt_income": amount(from_nei),
        "losses_deducted": amount(choice), "prior_year_losses": rows})
}

/// Offsets that would need rules this command does not model.
fn offset_limits(v: &Value, inc: &Income, gross: &D) -> Result<()> {
    let c = money(v, "non_refundable_non_carry_forward_offsets")?;
    let franking = money(v, "franking_tax_offsets")?;
    if franking > c {
        return Err("franking_tax_offsets is part of non_refundable_non_carry_forward_offsets and cannot exceed it".into());
    }
    if franking > d("0") && inc.used > d("0") {
        return Err(refuse(
            "franking tax offsets limit the choice of prior-year loss deduction",
            "s 36-17(5) ITAA 1997",
        ));
    }
    if franking > floor(gross.sub(&c.sub(&franking))) {
        return Err(refuse(
            "excess franking offsets are converted into a tax loss",
            "s 36-55 ITAA 1997",
        ));
    }
    if money(v, "non_refundable_carry_forward_offsets")? > d("0") && inc.nei > d("0") {
        return Err(refuse(
            "carry forward tax offsets first reduce net exempt income",
            "Div 65 ITAA 1997; CTR 2026 label D",
        ));
    }
    Ok(())
}

/// The calculation statement, labels A to S (CTR 2026 steps 1 to 9).
fn statement(v: &Value, taxable: &D, gross: &D, carry_back: &D) -> Result<(Value, Value)> {
    let get = |k| money(v, k);
    let (c, dd, f) = (
        get("non_refundable_non_carry_forward_offsets")?,
        get("non_refundable_carry_forward_offsets")?,
        get("franking_deficit_tax_offset")?,
    );
    let e = get("refundable_tax_offsets")?.add(carry_back);
    let (g, h, k) = (
        get("section_102aam_interest")?,
        get("eligible_credits")?,
        get("payg_instalments")?,
    );
    let t2 = floor(gross.sub(&c));
    let t3 = floor(t2.sub(&dd));
    let t4 = floor(t3.sub(&e));
    let i = floor(e.sub(&t3));
    let t5 = floor(t4.sub(&f));
    let s = t5.add(&g).sub(&h).sub(&i).sub(&k);
    let labels = json!({"A": amount(taxable), "T1": amount(gross), "B": amount(gross), "C": amount(&c),
        "T2": amount(&t2), "D": amount(&dd), "T3": amount(&t3), "E": amount(&e), "T4": amount(&t4),
        "F": amount(&f), "T5": amount(&t5), "G": amount(&g), "H": amount(&h), "I": amount(&i),
        "K": amount(&k), "S": amount(&s)});
    let rest = json!({"non_refundable_non_carry_forward_offsets_lost": amount(&floor(c.sub(gross))),
        "non_refundable_carry_forward_offsets_carried_forward": amount(&floor(dd.sub(&t2))),
        "franking_deficit_tax_offset_carried_forward": amount(&floor(f.sub(&t4)))});
    Ok((labels, rest))
}

const STEPS: [&str; 7] = [
    "Rate: base rate entity rate for the year if aggregated turnover is under the threshold and base rate entity passive income is no more than 80% of assessable income, otherwise 30% (Income Tax Rates Act 1986 ss 23(2), 23AA, 23AB).",
    "Prior-year losses are deducted only where the stated continuity of ownership (s 165-12) or business continuity (ss 165-13, 165-210, 165-211) outcome allows (s 165-10), oldest first (s 36-17(7)); the command does not judge either test.",
    "Net exempt income absorbs losses first; the company then deducts the amount it chooses from the excess of assessable income over deductions, or the most allowed when no choice is stated (s 36-17(2) to (4)).",
    "Tax loss: deductions less assessable income, less net exempt income (s 36-10).",
    "Gross tax: taxable income times the rate, rounded half up to the cent (CTR 2026 labels T1 and B).",
    "Offsets apply in order C, D, E (including the loss carry back tax offset), F; excess C is lost, excess D and F carry forward, excess E is refunded at I (CTR 2026 calculation statement steps 4 to 7).",
    "Amount due or refundable: S = T5 + G - H - I - K; negative is refundable (CTR 2026 step 9).",
];

pub fn evaluate(v: &Value) -> Result<Value> {
    let fy = number(v, "financial_year")?;
    scope(v, fy)?;
    let rate = bre(v)?.corporate_tax()?;
    let inc = income(v, fy)?;
    let gross = cents(&inc.taxable.mul(&rate.rate))?;
    offset_limits(v, &inc, &gross)?;
    let lcb = carry_back::evaluate(v, fy, &rate.rate, &inc.tax_loss)?;
    let offset = lcb.as_ref().map_or(d("0"), |c| c.offset.clone());
    let back = lcb.as_ref().map_or(d("0"), |c| c.from_current_year.clone());
    let (labels, rest) = statement(v, &inc.taxable, &gross, &offset)?;
    let mut taxable = inc.json;
    taxable["taxable_income"] = amount(&inc.taxable);
    taxable["tax_loss"] = amount(&inc.tax_loss);
    taxable["tax_loss_carried_back"] = amount(&back);
    taxable["tax_loss_carried_forward"] = amount(&inc.tax_loss.sub(&back));
    Ok(
        json!({"financial_year": fy, "corporate_tax": rate.json(), "taxable_income": taxable,
        "loss_carry_back": lcb.map(|c| c.json), "calculation_statement": labels,
        "offsets_not_used": rest, "steps": STEPS}),
    )
}
