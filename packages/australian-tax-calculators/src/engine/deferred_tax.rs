// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A company's tax effect worksheet under AASB 112: temporary differences from
//! carrying amounts and tax bases, the deferred tax liability and asset at the
//! rate expected when they reverse, the year's movement split into rate
//! change, recognition change and origination, current tax at the year's
//! Income Tax Rates Act 1986 s 23(2) rate, and the para 81(c)(i)
//! reconciliation. The paragraphs are those of the AASB 112 compilations of
//! March 2020 and September 2023 and AASB 18 Appendix D, the rates those of
//! s 23(2) (compilation 66) and the ATO's company tax rate pages, all read on
//! 22 November 2026. The standard's wording is not reproduced.

use serde_json::{Map, Value, json};

use super::{
    Arg, Result, Step, d, derived_result, flag, int, money, precise, rate, refuse, scope,
    signed_money, step,
};
use crate::decimal::{Calc, Decimal, Rounding};

mod amounts;
mod items;

const KIND: &str = "deferred_tax";

/// The worksheet's inputs, as the caller gave them.
pub struct DeferredTaxArgs<'a> {
    pub items: &'a Arg,
    pub unused_tax_losses: &'a Arg,
    pub opening_unused_tax_losses: &'a Arg,
    pub unrecognised_deductible: &'a Arg,
    pub opening_unrecognised_deductible: &'a Arg,
    pub unrecognised_used: &'a Arg,
    pub accounting_profit: &'a Arg,
    pub non_deductible_expenses: &'a Arg,
    pub non_assessable_income: &'a Arg,
    pub taxable_income: &'a Arg,
    pub tax_offsets: &'a Arg,
    pub base_rate_entity: &'a Arg,
    pub expected_rate: &'a Arg,
    pub opening_expected_rate: &'a Arg,
    pub business_combination: &'a Arg,
    pub uncertain_tax_treatment: &'a Arg,
    pub tax_consolidated_group: &'a Arg,
    pub outside_profit_or_loss: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const COMBINATION: &str = "Deferred tax arising in a business combination follows AASB 112 paras 19 and 66 to 68 and AASB 3, which this worksheet does not apply.";
const UNCERTAIN: &str = "An uncertain tax treatment is measured under AASB Interpretation 23, which this worksheet does not apply.";
const CONSOLIDATED: &str = "A tax-consolidated group member's current and deferred tax are allocated under AASB Interpretation 1052, which this worksheet does not apply.";
const OUTSIDE: &str = "Tax on items recognised in other comprehensive income or equity goes outside profit or loss (AASB 112 paras 61A to 63); this worksheet takes every movement to profit or loss.";

/// The company rate for the year (ITRA s 23(2); 26% for a base rate entity
/// in 2020-21 on the ATO's 2020-21 table).
pub(super) fn current_rate(year: &str, base_rate_entity: bool) -> &'static str {
    match (year, base_rate_entity) {
        ("2020-21", true) => "0.26",
        (_, true) => "0.25",
        _ => "0.30",
    }
}

/// The rate a balance is measured at (AASB 112 paras 47 and 48): every
/// difference open at the end of a supported year reverses from 2021-22.
fn deferred_rate(value: &Arg, name: &str) -> Result<&'static str> {
    match value {
        Arg::Text(t) if t == "base_rate_entity" => Ok("0.25"),
        Arg::Text(t) if t == "standard" => Ok("0.30"),
        _ => refuse(
            "deferred_tax.expected_rate_invalid",
            format!("{name} must be base_rate_entity or standard."),
        ),
    }
}

/// The year's facts, checked, before any arithmetic.
struct Facts {
    year: String,
    items: Vec<items::Item>,
    losses: Decimal,
    opening_losses: Decimal,
    unrecognised: Decimal,
    opening_unrecognised: Decimal,
    used: Decimal,
    profit: Decimal,
    non_deductible: Decimal,
    non_assessable: Decimal,
    taxable_income: Decimal,
    offsets: Decimal,
    applicable: Decimal,
    closing_rate: Decimal,
    opening_rate: Decimal,
}

/// The matters the worksheet refuses, each stated as a flag.
fn excluded(a: &DeferredTaxArgs) -> Result<()> {
    if flag(a.business_combination, "business_combination")? {
        return refuse("deferred_tax.business_combination", COMBINATION);
    }
    if flag(a.uncertain_tax_treatment, "uncertain_tax_treatment")? {
        return refuse("deferred_tax.uncertain_tax_treatment", UNCERTAIN);
    }
    if flag(a.tax_consolidated_group, "tax_consolidated_group")? {
        return refuse("deferred_tax.tax_consolidated_group", CONSOLIDATED);
    }
    if flag(a.outside_profit_or_loss, "outside_profit_or_loss")? {
        return refuse("deferred_tax.outside_profit_or_loss", OUTSIDE);
    }
    Ok(())
}

fn read(a: &DeferredTaxArgs) -> Result<Facts> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    excluded(a)?;
    let items = items::read(a.items)?;
    let applicable = current_rate(&year, flag(a.base_rate_entity, "base_rate_entity")?);
    let closing_rate = deferred_rate(a.expected_rate, "expected_rate")?;
    let opening_rate = deferred_rate(a.opening_expected_rate, "opening_expected_rate")?;
    if year == "2020-21" && opening_rate == "0.25" {
        return refuse(
            "deferred_tax.opening_rate_unscheduled",
            "Balances open at the start of 2020-21 reverse partly in 2020-21, at 26% for a base rate entity, and partly later at 25%; measuring them needs a reversal schedule this worksheet does not take.",
        );
    }
    let f = Facts {
        year,
        items,
        losses: money(a.unused_tax_losses)?,
        opening_losses: money(a.opening_unused_tax_losses)?,
        unrecognised: money(a.unrecognised_deductible)?,
        opening_unrecognised: money(a.opening_unrecognised_deductible)?,
        used: money(a.unrecognised_used)?,
        profit: signed_money(a.accounting_profit)?,
        non_deductible: money(a.non_deductible_expenses)?,
        non_assessable: money(a.non_assessable_income)?,
        taxable_income: money(a.taxable_income)?,
        offsets: money(a.tax_offsets)?,
        applicable: d(applicable),
        closing_rate: d(closing_rate),
        opening_rate: d(opening_rate),
    };
    if f.used.gt(&f.opening_unrecognised)? {
        return refuse(
            "deferred_tax.unrecognised_used_above_opening",
            "unrecognised_used is more than opening_unrecognised_deductible: only amounts with no deferred tax asset at the start of the year can be used as previously unrecognised.",
        );
    }
    Ok(f)
}

/// A step's id, authority and formula.
type Rule = (&'static str, &'static str, &'static str);

const DIFFERENCES: &str = "AASB 112 paras 5, 7 and 8: an asset's carrying amount less its tax base, or a liability's tax base less its carrying amount; positive is a taxable temporary difference, negative a deductible one";

/// The items' taxable and deductible differences at one date, as steps.
fn totals(rows: &[(Decimal, Decimal)], opening: bool) -> Calc<[Step; 2]> {
    let (taxable, deductible) = if opening {
        (
            "opening_taxable_temporary_differences",
            "opening_deductible_temporary_differences",
        )
    } else {
        (
            "taxable_temporary_differences",
            "deductible_temporary_differences",
        )
    };
    let pick = |n: usize| -> Calc<Step> {
        let terms: Vec<&Decimal> = rows
            .iter()
            .map(|r| if n == 0 { &r.0 } else { &r.1 })
            .filter(|v| !v.is_zero())
            .collect();
        let mut total = int(0);
        for term in &terms {
            total = total.add(term)?;
        }
        let shown: Vec<String> = terms.iter().map(ToString::to_string).collect();
        let formula = if shown.is_empty() {
            "0".to_string()
        } else {
            shown.join(" + ")
        };
        let id = if n == 0 { taxable } else { deductible };
        Ok(step(id, DIFFERENCES, &formula, &[], total))
    };
    Ok([pick(0)?, pick(1)?])
}

/// `v` rounded half up to the cent, with a negative zero made plain.
fn cents(v: Decimal) -> Calc<Decimal> {
    plain(v.quantize(-2, Rounding::HalfUp)?)
}

fn plain(v: Decimal) -> Calc<Decimal> {
    Ok(if v.is_zero() { d("0.00") } else { v })
}

const BALANCES: [Rule; 5] = [
    (
        "deferred_tax_liability",
        "AASB 112 paras 15, 47 and 53: a deferred tax liability on every taxable temporary difference, at the rate expected when it reverses (ITRA s 23(2)), undiscounted",
        "cents(taxable_temporary_differences x closing_rate)",
    ),
    (
        "deferred_tax_asset",
        "AASB 112 paras 24, 34 and 47: a deferred tax asset on deductible temporary differences and unused tax losses, less those for which future taxable profit is not probable (the preparer's judgement)",
        "cents((deductible_temporary_differences + unused_tax_losses - unrecognised_deductible) x closing_rate)",
    ),
    (
        "opening_deferred_tax_liability",
        "AASB 112 paras 15 and 47: the opening balance at the rate expected at the previous reporting date",
        "cents(opening_taxable_temporary_differences x opening_rate)",
    ),
    (
        "opening_deferred_tax_asset",
        "AASB 112 paras 24, 34 and 47: the opening balance at the rate expected at the previous reporting date",
        "cents((opening_deductible_temporary_differences + opening_unused_tax_losses - opening_unrecognised_deductible) x opening_rate)",
    ),
    (
        "net_deferred_tax_liability",
        "AASB 112 para 74: shown net only as a measure; offsetting in the statement of financial position needs a legally enforceable right of set-off and the same taxation authority",
        "deferred_tax_liability - deferred_tax_asset",
    ),
];

/// AASB 112 paras 24 and 34: no more can lack a deferred tax asset than the
/// deductible differences and losses there are.
fn unrecognised_within(
    deductible: &Decimal,
    losses: &Decimal,
    unrecognised: &Decimal,
    when: &str,
) -> Result<()> {
    if unrecognised.gt(&deductible.add(losses)?)? {
        return refuse(
            "deferred_tax.unrecognised_above_deductible",
            format!(
                "The unrecognised amount at {when} of the year is more than that date's deductible temporary differences and unused tax losses."
            ),
        );
    }
    Ok(())
}

/// AASB 112 paras 47 and 60(a): when the expected rate changes and the
/// year's own rate is not the closing rate, the opening balances that
/// reversed in the year did so at the year's rate and only the rest is
/// remeasured; telling them apart needs a reversal schedule.
fn remeasurable(f: &Facts, opening_liability: &Decimal, opening_asset: &Decimal) -> Result<()> {
    let changed = !f.opening_rate.equals(&f.closing_rate)?;
    let split = !f.applicable.equals(&f.closing_rate)?;
    if changed && split && !(opening_liability.is_zero() && opening_asset.is_zero()) {
        return refuse(
            "deferred_tax.rate_change_unscheduled",
            "The expected rate changed while the year's rate is not the closing rate: opening balances that reversed in the year did so at the year's rate and only the rest is remeasured, which needs a reversal schedule this worksheet does not take.",
        );
    }
    Ok(())
}

/// The balances at both dates, from the 4 difference totals in turn.
fn balances(f: &Facts, t: [&Decimal; 4]) -> Result<Vec<Step>> {
    let [taxable, deductible, opening_taxable, opening_deductible] = t;
    unrecognised_within(deductible, &f.losses, &f.unrecognised, "the end")?;
    let opening = (&f.opening_losses, &f.opening_unrecognised);
    unrecognised_within(opening_deductible, opening.0, opening.1, "the start")?;
    let liability = cents(taxable.mul(&f.closing_rate)?)?;
    let recognised = deductible.add(&f.losses)?.sub(&f.unrecognised)?;
    let asset = cents(recognised.mul(&f.closing_rate)?)?;
    let opening_liability = cents(opening_taxable.mul(&f.opening_rate)?)?;
    let opening_recognised = opening_deductible
        .add(&f.opening_losses)?
        .sub(&f.opening_unrecognised)?;
    let opening_asset = cents(opening_recognised.mul(&f.opening_rate)?)?;
    remeasurable(f, &opening_liability, &opening_asset)?;
    let net = plain(liability.sub(&asset)?)?;
    let rows = [
        (vec![taxable, &f.closing_rate], liability.clone()),
        (
            vec![deductible, &f.losses, &f.unrecognised, &f.closing_rate],
            asset.clone(),
        ),
        (
            vec![opening_taxable, &f.opening_rate],
            opening_liability.clone(),
        ),
        (
            vec![
                opening_deductible,
                &f.opening_losses,
                &f.opening_unrecognised,
                &f.opening_rate,
            ],
            opening_asset.clone(),
        ),
        (vec![&liability, &asset], net),
    ];
    Ok(rule_steps(&BALANCES, rows))
}

/// The steps of `rules`, each with its operands and value in turn.
fn rule_steps<const N: usize>(rules: &[Rule; N], rows: [(Vec<&Decimal>, Decimal); N]) -> Vec<Step> {
    rules
        .iter()
        .zip(rows)
        .map(|((id, authority, formula), (operands, value))| {
            step(*id, authority, formula, &operands, value)
        })
        .collect()
}

/// AASB 112 for one company and income year: the tax effect worksheet, the
/// income tax expense and its para 81(c)(i) reconciliation.
pub fn deferred_tax(a: &DeferredTaxArgs) -> Result<Value> {
    let f = read(a)?;
    let (rows, lines) = precise(|| items::differences(&f.items))?;
    let mut steps = precise(|| {
        let [taxable, deductible] = totals(&rows.0, false)?;
        let [opening_taxable, opening_deductible] = totals(&rows.1, true)?;
        let t = [
            &taxable.value,
            &deductible.value,
            &opening_taxable.value,
            &opening_deductible.value,
        ];
        let mut built = balances(&f, t)?;
        built.extend(amounts::movement(&f, t, &built)?);
        let mut all = vec![taxable, deductible, opening_taxable, opening_deductible];
        all.append(&mut built);
        Ok(all)
    })?;
    let extra = precise(|| amounts::expense(&f, &steps))?;
    steps.extend(extra);
    let effective = amounts::effective_rate(&f, &steps)?;
    let mut rates = vec![
        rate("applicable_rate", &f.applicable),
        rate("closing_rate", &f.closing_rate),
        rate("opening_rate", &f.opening_rate),
    ];
    rates.extend(effective.map(|e| rate("effective_tax_rate", e)));
    let mut out = derived_result(KIND, &f.year, steps, rates)?;
    finish(&mut out, lines);
    Ok(out)
}

const BASIS: &str = "The applicable rate is the company rate for the income year under Income Tax Rates Act 1986 s 23(2), the domestic rate AASB 112 para 85 points to: 30%, or for a base rate entity (s 23AA) 25% (26% in 2020-21).";
const RECOGNITION: &str = "The deferred tax asset rests on unrecognised_deductible and opening_unrecognised_deductible: whether future taxable profit is probable (AASB 112 paras 24, 34 to 36 and 56) and which rate will apply on reversal (paras 47 and 48) are the preparer's judgements, not tested here.";
const MISMATCH: &str = "taxable_income does not equal accounting profit plus non-deductible expenses, less non-assessable income and the change in net temporary differences and unused tax losses (taxable_income_difference): the inputs do not explain taxable income, and reconciliation_difference carries the gap at the applicable rate.";

fn finish(out: &mut Value, lines: Vec<Map<String, Value>>) {
    out["items"] = Value::Array(lines.into_iter().map(Value::Object).collect());
    out["applicable_rate_basis"] = json!(BASIS);
    let balanced = out["amounts"]["taxable_income_difference"].as_str() == Some("0.00");
    if let Some(warnings) = out["warnings"].as_array_mut() {
        warnings.push(json!(RECOGNITION));
        if !balanced {
            warnings.push(json!(MISMATCH));
        }
    }
}
