// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Queensland's periodic deduction, rate and mental health levy, as the
//! Payroll Tax Act 1971 (Qld) read on 26 November 2026.

use super::states::{SHARED, stated_deduction};
use super::{Month, taxed};
use crate::decimal::Decimal;
use crate::engine::payroll_tax::Worked;
use crate::engine::{Result, Step, d, int, py_max, rate, refuse, step};

const QLD_ACTUAL: &str = "Payroll Tax Act 1971 (Qld) ss 17 and 20(2)(c), current as at 22 October 2026, read on 26 November 2026: the actual periodic deduction, the greater of nil and FME / G - (TW - FME / G) / 7, with E $108,333 and a whole month (F = G, M = 1), for an employer that pays no interstate wages in the month and is not a previous interstate wage payer";
const QLD_FIXED: &str = "Payroll Tax Act 1971 (Qld) ss 17 and 20(2)(a) and (b): the fixed periodic deduction, the greater of nil and QW / AW x (E - (AW / 12 - E) / 7) x M, with E $108,333, M = 1 and QW and AW the employer's estimated taxable wages and estimated taxable and interstate wages for the year, worked on the most recent calculation day (s 18)";
const QLD_STATED: &str = "Payroll Tax Act 1971 (Qld) s 20(2)(a): the employer's deduction for the period determined by the commissioner under s 21, as stated";
const QLD_RATE: &str = "Payroll Tax Act 1971 (Qld) ss 10(1), (2) and (4) and 20(1)(b): 4.75%, or 4.95% when the month's taxable and interstate wages together are more than $541,666";
const QLD_PRIMARY_T: &str = "Payroll Tax Act 1971 (Qld) ss 43C(1) and 43E(3): the primary periodic threshold, $10 million x C / D x M / 12, C and D the estimated taxable wages and estimated taxable and interstate wages for the year";
const QLD_ADDITIONAL_T: &str = "Payroll Tax Act 1971 (Qld) ss 43D(1) and 43E(4): the additional periodic threshold, $100 million x C / D x M / 12";
const QLD_PRIMARY_D: &str = "Payroll Tax Act 1971 (Qld) s 43E(1)(a) and (3)(a): the primary periodic threshold the commissioner determined for the period, as stated";
const QLD_ADDITIONAL_D: &str = "Payroll Tax Act 1971 (Qld) s 43E(1)(b) and (4)(a): the additional periodic threshold the commissioner determined for the period, as stated";
const QLD_PRIMARY: &str = "Payroll Tax Act 1971 (Qld) s 43F(2): the primary periodic liability, the greater of nil and (TW - PPT x A / B) x 0.25%, wages paid on every day of the month (A = B)";
const QLD_ADDITIONAL: &str = "Payroll Tax Act 1971 (Qld) s 43F(3): the additional periodic liability, the greater of nil and (TW - APT x A / B) x 0.5%";

/// Queensland's actual or fixed periodic deduction.
fn qld_deduction(m: &Month, e: &Decimal) -> Result<Step> {
    if m.iw.is_zero() && !m.previous {
        let fall = m.tw.sub(e)?.div(&int(7))?;
        let value = py_max(int(0), e.sub(&fall)?)?;
        return Ok(step(
            "deduction",
            QLD_ACTUAL,
            "max(0, maximum_monthly_deduction - (taxable_wages - maximum_monthly_deduction) / 7)",
            &[e, &m.tw, e],
            value,
        ));
    }
    let aw = m.estimated()?;
    let fall = aw.div(&int(12))?.sub(e)?.div(&int(7))?;
    let value = py_max(int(0), m.shared(&e.sub(&fall)?)?)?;
    let formula = format!(
        "max(0, estimated_taxable_wages / {SHARED} x (maximum_monthly_deduction - ({SHARED} / 12 - maximum_monthly_deduction) / 7))"
    );
    let operands = [&m.etw, &m.etw, &m.eiw, e, &m.etw, &m.eiw, e];
    Ok(step("deduction", QLD_FIXED, &formula, &operands, value))
}

/// One of Queensland's 2 levies: its periodic threshold (`threshold`, the
/// id, authority and the stated determination's input name, with the
/// determined authority), `whole` x C / D / 12 unless determined, and the
/// levy (`levy`, id and authority) at `charge`, the rate named `rate_name`.
struct Levy<'a> {
    threshold: [&'static str; 4],
    levy: (&'static str, &'static str),
    whole: &'a str,
    charge: &'a Decimal,
    rate_name: &'static str,
    determined: Option<&'a Decimal>,
}

fn qld_levy(m: &Month, l: &Levy) -> Result<[Step; 2]> {
    let [threshold_id, worked, input, determined] = l.threshold;
    let threshold = if let Some(stated) = l.determined {
        step(threshold_id, determined, input, &[stated], stated.clone())
    } else {
        let value = m.shared(&d(l.whole))?.div(&int(12))?;
        let formula = format!("{} x estimated_taxable_wages / {SHARED} / 12", l.whole);
        let operands = [&m.etw, &m.etw, &m.eiw];
        step(threshold_id, worked, &formula, &operands, value)
    };
    let levy = py_max(int(0), m.tw.sub(&threshold.value)?.mul(l.charge)?)?;
    let formula = format!("max(0, (taxable_wages - {threshold_id}) x {})", l.rate_name);
    let operands = [&m.tw, &threshold.value, l.charge];
    let (id, authority) = l.levy;
    let levy = step(id, authority, &formula, &operands, levy);
    Ok([threshold, levy])
}

/// Queensland's 2 periodic thresholds and their levies, at `low` and `high`.
fn qld_levies(m: &Month, low: &Decimal, high: &Decimal) -> Result<[Step; 4]> {
    let [primary, additional] = &m.thresholds;
    let [pt, pl] = qld_levy(
        m,
        &Levy {
            threshold: [
                "primary_periodic_threshold",
                QLD_PRIMARY_T,
                "stated_primary_threshold",
                QLD_PRIMARY_D,
            ],
            levy: ("mental_health_levy_primary", QLD_PRIMARY),
            whole: "10000000",
            charge: low,
            rate_name: "levy_rate",
            determined: primary.as_ref(),
        },
    )?;
    let [at, al] = qld_levy(
        m,
        &Levy {
            threshold: [
                "additional_periodic_threshold",
                QLD_ADDITIONAL_T,
                "stated_additional_threshold",
                QLD_ADDITIONAL_D,
            ],
            levy: ("mental_health_levy_additional", QLD_ADDITIONAL),
            whole: "100000000",
            charge: high,
            rate_name: "additional_levy_rate",
            determined: additional.as_ref(),
        },
    )?;
    Ok([pt, at, pl, al])
}

pub(in crate::engine::payroll_tax) fn qld(m: &Month) -> Result<Worked> {
    if m.regional {
        return refuse(
            "payroll_tax_monthly.regional_discount",
            "A Queensland regional employer is not worked: the 1% discount on the rate (Payroll Tax Act 1971 (Qld) s 10A) turns on where its principal place of employment and employees are, which this worksheet does not take.",
        );
    }
    let [e, upper, low, high] = ["108333", "541666", "0.0025", "0.005"].map(d);
    let applied = d(if m.tw.add(&m.iw)?.gt(&upper)? {
        "0.0495"
    } else {
        "0.0475"
    });
    let deduction = match stated_deduction(m, QLD_STATED, None)? {
        Some(stated) => stated,
        None => qld_deduction(m, &e)?,
    };
    let charged = taxed(m, deduction, (QLD_RATE, &applied))?;
    let [pt, at, pl, al] = qld_levies(m, &low, &high)?;
    // The thresholds come first, so the 3 charges end the steps.
    let mut steps = vec![pt, at];
    steps.extend(charged);
    steps.extend([pl, al]);
    let rates = vec![
        rate("maximum_monthly_deduction", &e),
        rate("rate", &applied),
        rate("levy_rate", &low),
        rate("additional_levy_rate", &high),
    ];
    Ok(Worked {
        steps,
        charges: 3,
        rates,
    })
}
