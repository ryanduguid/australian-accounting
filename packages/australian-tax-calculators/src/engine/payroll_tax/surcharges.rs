// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Victoria and Queensland, whose phased-out deductions come with 2
//! surcharges or levies on the jurisdiction's share of the Australian wages
//! above $10 million and $100 million.

use super::{Wages, Worked, one_rate};
use crate::decimal::Decimal;
use crate::engine::{Result, Step, d, int, py_max, py_min, rate, step};

/// A surcharge or levy at `charge` on the jurisdiction's share of the
/// Australian wages above each of `thresholds` (named), as each Act
/// apportions its thresholds.
fn levy(
    w: &Wages,
    (id, authority): (&'static str, &'static str),
    thresholds: &[(&str, &Decimal)],
    (rate_name, charge): (&str, &Decimal),
) -> Result<Step> {
    let mut size: Option<Decimal> = None;
    let mut terms = Vec::new();
    let mut operands = vec![&w.taxable];
    for (name, threshold) in thresholds {
        let part = w.above(threshold)?;
        size = Some(match size {
            Some(sum) => sum.add(&part)?,
            None => part,
        });
        terms.push(format!("max(0, australian_wages - {name})"));
        operands.extend([&w.australian, *threshold]);
    }
    operands.extend([charge, &w.australian]);
    let size = size.expect("a threshold");
    let above = match terms.as_slice() {
        [one] => one.clone(),
        _ => format!("({})", terms.join(" + ")),
    };
    let formula = format!("taxable_wages x {above} x {rate_name} / australian_wages");
    let value = w.taxable.mul(&size)?.mul(charge)?.div(&w.australian)?;
    Ok(step(id, authority, &formula, &operands, value))
}

const VIC_BASE: &str = "Payroll Tax Act 2007 (Vic) Sch 1 cl 1 (TA $1,000,000; APT, the phase-out start, $3,000,000; PR, the phase-out rate, 50% from 1 July 2025) and cl 5, authorised version 041 (in force from 1 May 2026), with the State Revenue Office's 'Payroll tax (current rates)' (updated 29 September 2026: no deduction above $5,000,000), both read on 19 November 2026";
const VIC_SHARED: &str = "Payroll Tax Act 2007 (Vic) Sch 1 cl 5: the deduction shared by the Victorian taxable wages over the Australian wages";
const VIC_RATE: &str = "Payroll Tax Act 2007 (Vic) Sch 1 cl 1 (R: 4.85%) and the State Revenue Office's 'Payroll tax (current rates)'";
const VIC_REGIONAL_RATE: &str = "Payroll Tax Act 2007 (Vic) s 3A and Sch 1 cl 1 (R: 1.2125% for a regional employer, one that pays at least 85% of its taxable wages to regional employees) and the State Revenue Office's 'Payroll tax (current rates)'";
const VIC_MENTAL_HEALTH: &str = "Payroll Tax Act 2007 (Vic) Sch 1 cl 1A (STA1 $10,000,000 and STA2 $100,000,000) and cl 5 (S at 0.5%), with the State Revenue Office's 'Payroll tax surcharges' (read 19 November 2026: 0.5% above $10 million and an additional 0.5% above $100 million, on the Victorian share): the mental health and wellbeing surcharge";
const VIC_COVID: &str = "Payroll Tax Act 2007 (Vic) Sch 1 cl 1B (T, equal to S, from 1 July 2023 to 30 June 2033), with the State Revenue Office's 'Payroll tax surcharges': the COVID-19 debt temporary payroll tax surcharge";

pub(super) fn victoria(w: &Wages, regional: bool) -> Result<Worked> {
    let [threshold, from, phase, first, second, charge] = [
        "1000000",
        "3000000",
        "0.5",
        "10000000",
        "100000000",
        "0.005",
    ]
    .map(d);
    let (authority, applied) = if regional {
        (VIC_REGIONAL_RATE, d("0.012125"))
    } else {
        (VIC_RATE, d("0.0485"))
    };
    let tapered = threshold.sub(&w.above(&from)?.mul(&phase)?)?;
    let value = py_min(threshold.clone(), py_max(int(0), tapered)?)?;
    let base = step(
        "australian_deduction",
        VIC_BASE,
        "min(threshold, max(0, threshold - max(0, australian_wages - phase_out_from) x phase_out_rate))",
        &[&threshold, &threshold, &w.australian, &from, &phase],
        value,
    );
    let mut steps = one_rate(w, base, VIC_SHARED, (authority, &applied))?;
    let tiers = [
        ("surcharge_threshold", &first),
        ("second_surcharge_threshold", &second),
    ];
    for surcharge in [
        ("mental_health_and_wellbeing_surcharge", VIC_MENTAL_HEALTH),
        ("covid_19_debt_temporary_surcharge", VIC_COVID),
    ] {
        steps.push(levy(w, surcharge, &tiers, ("surcharge_rate", &charge))?);
    }
    let rates = vec![
        rate("threshold", &threshold),
        rate("phase_out_from", &from),
        rate("phase_out_rate", &phase),
        rate("rate", &applied),
        rate("surcharge_threshold", &first),
        rate("second_surcharge_threshold", &second),
        rate("surcharge_rate", &charge),
    ];
    Ok(Worked {
        steps,
        charges: 3,
        rates,
    })
}

const QLD_BASE: &str = "Payroll Tax Act 1971 (Qld) s 29(1), annual deduction (K $1,300,000, falling by $1 for every $7 of Australian wages above it, never below zero), reprint current as at 22 October 2026, with the Queensland Revenue Office's 'Payroll tax deductions' (nil at $10.4 million), both read on 19 November 2026";
const QLD_SHARED: &str = "Payroll Tax Act 1971 (Qld) s 29(1): AW / (AW + IW), the Queensland taxable wages over the Australian wages, x the deduction";
const QLD_RATE: &str = "Payroll Tax Act 1971 (Qld) s 10(1), (2) and (4) and the Queensland Revenue Office's 'Payroll tax rates and thresholds' (last updated 17 July 2024): 4.75%, or 4.95% when the Australian wages are more than $6,500,000";
const QLD_PRIMARY: &str = "Payroll Tax Act 1971 (Qld) s 43I, primary annual levy amount: (B - C x B / D) x R with C $10m and R 0.25%, with the Queensland Revenue Office's 'Calculating the mental health levy' (last updated 11 August 2026)";
const QLD_ADDITIONAL: &str = "Payroll Tax Act 1971 (Qld) s 43I, additional annual levy amount: (B - C x B / D) x R with C $100m and R 0.5%, with the Queensland Revenue Office's 'Calculating the mental health levy'";

pub(super) fn queensland(w: &Wages) -> Result<Worked> {
    let [k, upper, low, high, first, second, primary, additional] = [
        "1300000",
        "6500000",
        "0.0475",
        "0.0495",
        "10000000",
        "100000000",
        "0.0025",
        "0.005",
    ]
    .map(d);
    let value = py_max(int(0), k.sub(&w.above(&k)?.div(&int(7))?)?)?;
    let base = step(
        "australian_deduction",
        QLD_BASE,
        "max(0, threshold - max(0, australian_wages - threshold) / 7)",
        &[&k, &w.australian, &k],
        value,
    );
    let applied = if w.australian.gt(&upper)? { high } else { low };
    let mut steps = one_rate(w, base, QLD_SHARED, (QLD_RATE, &applied))?;
    steps.push(levy(
        w,
        ("mental_health_levy_primary", QLD_PRIMARY),
        &[("levy_threshold", &first)],
        ("levy_rate", &primary),
    )?);
    steps.push(levy(
        w,
        ("mental_health_levy_additional", QLD_ADDITIONAL),
        &[("additional_levy_threshold", &second)],
        ("additional_levy_rate", &additional),
    )?);
    let rates = vec![
        rate("threshold", &k),
        rate("rate", &applied),
        rate("levy_threshold", &first),
        rate("levy_rate", &primary),
        rate("additional_levy_threshold", &second),
        rate("additional_levy_rate", &additional),
    ];
    Ok(Worked {
        steps,
        charges: 3,
        rates,
    })
}
