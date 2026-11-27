// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `payroll_tax`: one employer's annual payroll tax for 2026-27 in one state
//! or territory, for an employer that is not a member of a group and pays or
//! is liable to pay wages on every day of the year. Each jurisdiction allows a
//! deduction (or threshold) worked out on the employer's Australian wages, its
//! taxable and interstate wages together, and shares it by the jurisdiction's
//! taxable wages over the Australian wages; the rest is taxed at the rate the
//! Australian wages set, and Victoria and Queensland add 2 surcharges or levies
//! on the jurisdiction's share of the Australian wages above their thresholds.
//! Every figure was read at the revenue office and in the Act on 19 November
//! 2026; docs/calculation-evidence.md#payroll-tax lists the versions read.

use serde_json::Value;

use super::trade_in::when;
use super::{
    Arg, Result, Step, cents, d, derived_result, flag, int, money, precise, py_max, py_min, rate,
    refuse, scope, step,
};
use crate::decimal::Decimal;

mod group;
mod monthly;
mod reconcile;
mod surcharges;
pub use group::{PayrollTaxGroupArgs, payroll_tax_group};
pub use monthly::{PayrollTaxMonthlyArgs, payroll_tax_monthly};
use surcharges::{queensland, victoria};

const KIND: &str = "payroll_tax";

/// The facts as the command passes them; see the README for each.
pub struct PayrollTaxArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub taxable_wages: &'a Arg,
    pub australian_wages: &'a Arg,
    pub regional_employer: Option<&'a Arg>,
    pub group_member: &'a Arg,
    pub employed_all_year: &'a Arg,
    pub periodic_tax: Option<&'a Arg>,
    pub periodic_levy: Option<&'a Arg>,
    pub other_refunds: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The jurisdiction's taxable wages and the employer's Australian wages.
struct Wages {
    taxable: Decimal,
    australian: Decimal,
}

impl Wages {
    /// max(0, australian_wages - threshold).
    fn above(&self, threshold: &Decimal) -> Result<Decimal> {
        Ok(py_max(int(0), self.australian.sub(threshold)?)?)
    }
}

/// A jurisdiction's steps; the last `charges` of them are the amounts payable
/// that `total` adds.
struct Worked {
    steps: Vec<Step>,
    charges: usize,
    rates: Vec<(String, String)>,
}

const TAXABLE: &str =
    "The jurisdiction's taxable wages less the deduction: the amount the rate applies to";

/// The `australian_deduction` step `base`, then the `deduction` that shares it
/// by the taxable wages over the Australian wages (`shared` cites how) and the
/// `taxable_amount`. Every value here is worked as its formula reads.
fn deduction(w: &Wages, base: Step, shared: &'static str) -> Result<Vec<Step>> {
    let value = py_min(base.value.clone(), w.australian.clone())?
        .mul(&w.taxable)?
        .div(&w.australian)?;
    let deduction = step(
        "deduction",
        shared,
        "min(australian_deduction, australian_wages) x taxable_wages / australian_wages",
        &[&base.value, &w.australian, &w.taxable, &w.australian],
        value,
    );
    let rest = w.taxable.sub(&deduction.value)?;
    let operands = [&w.taxable, &deduction.value];
    let taxable = step(
        "taxable_amount",
        TAXABLE,
        "taxable_wages - deduction",
        &operands,
        rest,
    );
    Ok(vec![base, deduction, taxable])
}

/// The deduction's steps, then `payroll_tax` at one rate on the taxable amount.
fn one_rate(
    w: &Wages,
    base: Step,
    shared: &'static str,
    (authority, applied): (&'static str, &Decimal),
) -> Result<Vec<Step>> {
    let mut steps = deduction(w, base, shared)?;
    let taxable = &steps[2].value;
    let tax = taxable.mul(applied)?;
    let operands = [taxable, applied];
    let payroll = step(
        "payroll_tax",
        authority,
        "taxable_amount x rate",
        &operands,
        tax,
    );
    steps.push(payroll);
    Ok(steps)
}

/// An `australian_deduction` step that is a figure of the Act itself.
fn figure_step(authority: &'static str, name: &str, figure: &Decimal) -> Step {
    step(
        "australian_deduction",
        authority,
        name,
        &[figure],
        figure.clone(),
    )
}

const NSW_THRESHOLD: &str = "Payroll Tax Act 2007 (NSW) Sch 1 cl 1 (TA: $1,200,000 for the financial year commencing on 1 July 2020 and later years; version current from 10 July 2026) and Revenue NSW's 'Payroll tax thresholds and rates' (last updated 1 July 2026: $1,200,000 for 1 July 2026 to 30 June 2027), both read on 19 November 2026";
const NSW_SHARED: &str = "Revenue NSW's 'Payroll tax thresholds and rates': the NSW payroll tax threshold x (total NSW wages / total Australian wages)";
const NSW_RATE: &str = "Payroll Tax Act 2007 (NSW) Sch 1 cl 1 (R: 5.45% on and from 1 July 2022) and Revenue NSW's 'Payroll tax thresholds and rates' (5.45% for 2026-27): the rate on the wages above the threshold";

fn new_south_wales(w: &Wages) -> Result<Worked> {
    let (threshold, applied) = (d("1200000"), d("0.0545"));
    let base = figure_step(NSW_THRESHOLD, "threshold", &threshold);
    let steps = one_rate(w, base, NSW_SHARED, (NSW_RATE, &applied))?;
    let rates = vec![rate("threshold", &threshold), rate("rate", &applied)];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

const WA_BASE: &str = "Pay-roll Tax Assessment Act 2002 (WA) ss 8 and 13 (AT $1,000,000 and UT $7,500,000 for a financial year beginning on or after 1 July 2020; the deduction falls by AT / (UT - AT) for each dollar above AT), version 04-d0-03 current from 1 July 2019, with RevenueWA's 'Calculation: Payroll Tax Employer Guide' (last updated 2 June 2026), both read on 19 November 2026";
const WA_SHARED: &str = "Pay-roll Tax Assessment Act 2002 (WA) s 13: the deduction x the Western Australian taxable wages over the Australian wages";
const WA_RATE: &str = "Pay-roll Tax Act 2002 (WA) s 5(4) (version 00-d0-02 current from 14 December 2017) and RevenueWA's 'Calculation: Payroll Tax Employer Guide': 5.5%";

fn western_australia(w: &Wages) -> Result<Worked> {
    let [threshold, upper, applied] = ["1000000", "7500000", "0.055"].map(d);
    let fall = w
        .above(&threshold)?
        .mul(&threshold)?
        .div(&upper.sub(&threshold)?)?;
    let value = py_max(int(0), threshold.sub(&fall)?)?;
    let base = step(
        "australian_deduction",
        WA_BASE,
        "max(0, threshold - max(0, australian_wages - threshold) x threshold / (upper_threshold - threshold))",
        &[
            &threshold,
            &w.australian,
            &threshold,
            &threshold,
            &upper,
            &threshold,
        ],
        value,
    );
    let steps = one_rate(w, base, WA_SHARED, (WA_RATE, &applied))?;
    let rates = vec![
        rate("threshold", &threshold),
        rate("upper_threshold", &upper),
        rate("rate", &applied),
    ];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

const SA_BASE: &str = "Payroll Tax Act 2009 (SA) Sch 1 cl 1 (EA $600,000) and cl 5(1), version 31.3.2026, with RevenueSA's 'Rates and Thresholds' (maximum deduction $600,000 a year from 1 July 2009), both read on 19 November 2026";
const SA_SHARED: &str = "Payroll Tax Act 2009 (SA) Sch 1 cl 5(1): TW / (TW + IW) x EA, the deduction shared by the South Australian taxable wages over the Australian wages";
const SA_NIL: &str = "Payroll Tax Act 2009 (SA) Sch 1 cl 1 (TA $1,500,000) and cl 4: no payroll tax when the Australian wages are not more than the threshold";
const SA_PHASED: &str = "Payroll Tax Act 2009 (SA) Sch 1 cl 5(2)(a): R = (TARW - $1,500,000) / $200,000 x 4.95% when the Australian wages are more than $1,500,000 and not more than $1,700,000 (RevenueSA's 'Rates and Thresholds': variable from 0% to 4.95%)";
const SA_FULL: &str = "Payroll Tax Act 2009 (SA) Sch 1 cl 5(2)(b) and RevenueSA's 'Rates and Thresholds': 4.95% when the Australian wages are more than $1,700,000";

/// South Australia's rate, shown as `rate`, and its `payroll_tax` step: nil to
/// the threshold, phased in to 4.95% at $1,700,000, then 4.95%.
fn sa_payroll_tax(
    w: &Wages,
    taxable: &Decimal,
    threshold: &Decimal,
    full: &Decimal,
) -> Result<(String, Step)> {
    let [full_from, span] = ["1700000", "200000"].map(d);
    if !w.australian.gt(threshold)? {
        let nil = step("payroll_tax", SA_NIL, "0", &[], int(0));
        return Ok(("0".to_string(), nil));
    }
    if w.australian.gt(&full_from)? {
        let operands = [taxable, full];
        let tax = taxable.mul(full)?;
        let payroll = step(
            "payroll_tax",
            SA_FULL,
            "taxable_amount x full_rate",
            &operands,
            tax,
        );
        return Ok((full.to_string(), payroll));
    }
    let over = w.australian.sub(threshold)?;
    // The phased rate has at least 6 places (2 from the wages' cents and 4
    // from 4.95%) and is shown without its trailing zeros.
    let phased = over.div(&span)?.mul(full)?.to_string();
    let tax = taxable.mul(&over)?.div(&span)?.mul(full)?;
    let payroll = step(
        "payroll_tax",
        SA_PHASED,
        "taxable_amount x (australian_wages - threshold) / 200000 x full_rate",
        &[taxable, &w.australian, threshold, full],
        tax,
    );
    Ok((phased.trim_end_matches('0').to_string(), payroll))
}

fn south_australia(w: &Wages) -> Result<Worked> {
    let [threshold, maximum, full] = ["1500000", "600000", "0.0495"].map(d);
    let base = figure_step(SA_BASE, "maximum_deduction", &maximum);
    let mut steps = deduction(w, base, SA_SHARED)?;
    let taxable = steps[2].value.clone();
    let (applied, payroll) = sa_payroll_tax(w, &taxable, &threshold, &full)?;
    steps.push(payroll);
    let rates = vec![
        rate("threshold", &threshold),
        rate("maximum_deduction", &maximum),
        rate("full_rate", &full),
        rate("rate", &applied),
    ];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

const TAS_BASE: &str = "Payroll Tax Act 2008 (Tas) Sch 1 cl 1 (TA $1,250,000 for a financial year commencing on or after 1 July 2018) and cl 4, with the State Revenue Office's 'Rates and thresholds' for the 2026-27 financial year, both read on 19 November 2026";
const TAS_SHARED: &str = "Payroll Tax Act 2008 (Tas) Sch 1 cl 4(4) and (5): TW / (TW + IW) x TA, the threshold shared by the Tasmanian taxable wages over the Australian wages";
const TAS_LOWER: &str = "Payroll Tax Act 2008 (Tas) Sch 1 cl 1 (R1 4%) and cl 4(4) and (5): R1 on the Tasmanian share of the Australian wages from the threshold to the top tax bracket threshold, TTB $2,000,000 (the State Revenue Office: 4% on $1,250,001 to $2,000,000)";
const TAS_HIGHER: &str = "Payroll Tax Act 2008 (Tas) Sch 1 cl 1 (R2 6.1%) and cl 4(5): R2 on the Tasmanian wages less the Tasmanian share of TTB (the State Revenue Office: 6.1% from $2,000,001)";
const TAS_TOTAL: &str = "Payroll Tax Act 2008 (Tas) Sch 1 cl 4(5) supplies the 2-rate formula. This worksheet rounds the tax at each rate to the cent before adding.";

/// The top tax bracket threshold, TTB (Payroll Tax Act 2008 (Tas) Sch 1 cl 1).
const TAS_TOP: &str = "2000000";

fn tasmania(w: &Wages) -> Result<Worked> {
    let [threshold, top, lower, higher] = ["1250000", TAS_TOP, "0.04", "0.061"].map(d);
    let base = figure_step(TAS_BASE, "threshold", &threshold);
    let mut steps = deduction(w, base, TAS_SHARED)?;
    let t = &w.australian;
    let middle = py_min(t.clone(), top.clone())?.sub(&py_min(t.clone(), threshold.clone())?)?;
    let first = step(
        "tax_at_lower_rate",
        TAS_LOWER,
        "taxable_wages x (min(australian_wages, top_bracket) - min(australian_wages, threshold)) x lower_rate / australian_wages",
        &[&w.taxable, t, &top, t, &threshold, &lower, t],
        w.taxable.mul(&middle)?.mul(&lower)?.div(t)?,
    );
    let second = step(
        "tax_at_higher_rate",
        TAS_HIGHER,
        "taxable_wages x max(0, australian_wages - top_bracket) x higher_rate / australian_wages",
        &[&w.taxable, t, &top, &higher, t],
        w.taxable.mul(&w.above(&top)?)?.mul(&higher)?.div(t)?,
    );
    let tax = cents(first.value.clone())?.add(&cents(second.value.clone())?)?;
    let total = step(
        "payroll_tax",
        TAS_TOTAL,
        "cents(tax_at_lower_rate) + cents(tax_at_higher_rate)",
        &[&first.value, &second.value],
        tax,
    );
    steps.extend([first, second, total]);
    let rates = vec![
        rate("threshold", &threshold),
        rate("top_bracket", &top),
        rate("lower_rate", &lower),
        rate("higher_rate", &higher),
    ];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

const ACT_BASE: &str = "Taxation Administration (Amounts and Rates—Payroll Tax) Determination 2026 (DI2026-151) s 5(a) (annual threshold $1,750,000, from 1 July 2026) for the Payroll Tax Act 2011 (ACT) Sch 1 (republication R20, effective 26 December 2025), with the ACT Revenue Office's 'About payroll tax', both read on 19 November 2026";
const ACT_SHARED: &str = "Payroll Tax Act 2011 (ACT) Sch 1 and the ACT Revenue Office's 'Calculating payroll tax' step 2: the ACT taxable wages over the Australia-wide wages x the threshold";
const ACT_RATE: &str = "DI2026-151 s 5(c)(ii) and Table 1, by total wages for the financial year: 6.75% to $20 million, 6.85% to $50 million, 7.35% to $100 million, 7.85% to $150 million and 8.75% above (the ACT Revenue Office's 'About payroll tax', Table 1)";

fn australian_capital_territory(w: &Wages) -> Result<Worked> {
    let threshold = d("1750000");
    let bands = [
        ("20000000", "0.0675"),
        ("50000000", "0.0685"),
        ("100000000", "0.0735"),
        ("150000000", "0.0785"),
    ];
    let mut applied = d("0.0875");
    for (to, band) in bands {
        if !w.australian.gt(&d(to))? {
            applied = d(band);
            break;
        }
    }
    let base = figure_step(ACT_BASE, "threshold", &threshold);
    let steps = one_rate(w, base, ACT_SHARED, (ACT_RATE, &applied))?;
    let rates = vec![rate("threshold", &threshold), rate("rate", &applied)];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

const NT_BASE: &str = "Payroll Tax Act 2009 (NT) Sch 1 cl 1 (TA $2,500,000) and cl 3 (the annual deduction falls by $1 for every $2 of Australian wages above the threshold, never below zero), as in force at 1 July 2026, with the Territory Revenue Office's 'Payroll tax rates and thresholds' and 'Payroll changes from 1 July 2026' (the deduction settings unchanged), both read on 19 November 2026";
const NT_SHARED: &str = "Payroll Tax Act 2009 (NT) Sch 1 cl 5: TW / (TW + IW) x AD, the deduction shared by the Territory taxable wages over the Australian wages";
const NT_RATE: &str = "Payroll Tax Act 2009 (NT) s 7A and the Territory Revenue Office's 'Payroll tax rates and thresholds': 5.5%, or 6.5% when the Australian wages are equal to or greater than $100,000,000";

fn northern_territory(w: &Wages) -> Result<Worked> {
    let [threshold, upper, low, high] = ["2500000", "100000000", "0.055", "0.065"].map(d);
    let value = py_max(int(0), threshold.sub(&w.above(&threshold)?.div(&int(2))?)?)?;
    let base = step(
        "australian_deduction",
        NT_BASE,
        "max(0, threshold - max(0, australian_wages - threshold) / 2)",
        &[&threshold, &w.australian, &threshold],
        value,
    );
    let applied = if w.australian.lt(&upper)? { low } else { high };
    let steps = one_rate(w, base, NT_SHARED, (NT_RATE, &applied))?;
    let rates = vec![rate("threshold", &threshold), rate("rate", &applied)];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

/// Refuses an employer this worksheet does not work.
fn refuse_unsupported(args: &PayrollTaxArgs) -> Result<()> {
    if flag(args.group_member, "group_member")? {
        return refuse(
            "payroll_tax.grouped",
            "A member of a group is not worked: each jurisdiction tests its threshold, rate and any surcharge or levy on the group's Australian wages, and only one member claims the deduction (Revenue NSW: 'Only one group member can claim the payroll tax threshold in NSW'; the Queensland Revenue Office: 'only the designated group employer (DGE) can claim a deduction').",
        );
    }
    if !flag(args.employed_all_year, "employed_all_year")? {
        return refuse(
            "payroll_tax.part_year",
            "An employer that paid wages for only part of the year is not worked: each jurisdiction reduces its threshold by the days it paid or was liable to pay wages over the days in the year (Revenue NSW: $1,200,000 x (184 / 365) = $604,931.51 for 184 days).",
        );
    }
    Ok(())
}

const NO_WAGES: &str = "No taxable or interstate wages were paid or payable in the year, so there is no deduction and no payroll tax";
const TOTAL: &str =
    "The payroll tax and any surcharge or levy, each to the cent, payable for the financial year";

/// The jurisdiction's working on `w`, with `total` added as the last step.
fn worked(place: &str, w: &Wages, regional: bool) -> Result<Worked> {
    let worked = match place {
        "nsw" => new_south_wales(w),
        "vic" => victoria(w, regional),
        "qld" => queensland(w),
        "wa" => western_australia(w),
        "sa" => south_australia(w),
        "tas" => tasmania(w),
        "act" => australian_capital_territory(w),
        _ => northern_territory(w),
    };
    totalled(worked?, TOTAL)
}

/// `worked` with a `total` step (cited by `authority`) that adds its charges,
/// each to the cent.
fn totalled(mut worked: Worked, authority: &'static str) -> Result<Worked> {
    let charged = &worked.steps[worked.steps.len() - worked.charges..];
    let names: Vec<String> = charged
        .iter()
        .map(|s| format!("cents({})", s.static_id()))
        .collect();
    let operands: Vec<&Decimal> = charged.iter().map(|s| &s.value).collect();
    let mut sum = cents(charged[0].value.clone())?;
    for s in &charged[1..] {
        sum = sum.add(&cents(s.value.clone())?)?;
    }
    let total = step("total", authority, &names.join(" + "), &operands, sum);
    worked.steps.push(total);
    Ok(worked)
}

pub fn payroll_tax(args: &PayrollTaxArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = match args.jurisdiction.text() {
        Some(place @ ("nsw" | "vic" | "qld" | "wa" | "sa" | "tas" | "act" | "nt")) => place,
        _ => {
            return refuse(
                "payroll_tax.jurisdiction_invalid",
                "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
            );
        }
    };
    let why = "for Victoria (Payroll Tax Act 2007 (Vic) s 3A)";
    let regional = when(
        place == "vic",
        args.regional_employer,
        "regional_employer",
        why,
    )?;
    let regional = regional
        .map(|given| flag(given, "regional_employer"))
        .transpose()?;
    refuse_unsupported(args)?;
    let wages = Wages {
        taxable: money(args.taxable_wages)?,
        australian: money(args.australian_wages)?,
    };
    if wages.australian.lt(&wages.taxable)? {
        return refuse(
            "payroll_tax.wages_conflict",
            "australian_wages, the taxable and interstate wages together, cannot be less than taxable_wages.",
        );
    }
    let periodic = reconcile::periodic(args, place)?;
    let Worked {
        mut steps, rates, ..
    } = year_worked(place, &wages, regional == Some(true))?;
    if let Some(periodic) = periodic {
        let settled = precise(|| reconcile::steps(place, &steps, &periodic))?;
        steps.extend(settled);
    }
    derived_result(KIND, &year, steps, rates)
}

/// The year's working; with no wages the steps keep their names, worked on
/// $1 of Australian wages so that nothing divides by nil, and each is nil.
fn year_worked(place: &str, wages: &Wages, regional: bool) -> Result<Worked> {
    if !wages.australian.is_zero() {
        return precise(|| worked(place, wages, regional));
    }
    let one = Wages {
        taxable: int(0),
        australian: int(1),
    };
    let Worked {
        steps,
        rates,
        charges,
    } = precise(|| worked(place, &one, regional))?;
    let steps = steps
        .into_iter()
        .map(|s| step(s.id, NO_WAGES, "0", &[], int(0)))
        .collect();
    Ok(Worked {
        steps,
        charges,
        rates,
    })
}
