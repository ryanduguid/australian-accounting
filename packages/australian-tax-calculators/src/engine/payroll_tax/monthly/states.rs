// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Each jurisdiction's monthly deduction, rate and any surcharge or levy, as
//! its Act's monthly provisions read on 26 November 2026.

use super::{Month, nil, taxed};
use crate::decimal::Decimal;
use crate::engine::payroll_tax::Worked;
use crate::engine::{Result, Step, d, int, py_max, py_min, rate, refuse, step};

const VIC_LOCAL: &str = "Payroll Tax Act 2007 (Vic) Sch 2 cl 5(1) and (1A), authorised version 041 (in force from 1 May 2026), read on 26 November 2026: for a month without interstate wages, the lesser of BDA - (ETW - APT) / 12 x PR (nil if below nil) and BDA, where BDA is $83,333 (cl 1A, a month from 1 July 2025), APT $3,000,000 and PR 50% (Sch 1 cl 1) and ETW the estimated taxable wages for the year";
const VIC_LIMIT: &str = "Payroll Tax Act 2007 (Vic) Sch 2 cl 5(1) and (1A): BDA - (ETW - APT) / 12 x PR (nil if below nil), and not more than BDA; cl 6(4) caps the deduction claimed at this amount";
const VIC_CLAIM: &str = "Payroll Tax Act 2007 (Vic) Sch 2 cl 6(3), (3A) and (4): for a month with interstate wages, the lesser of [BDA x ETW / (ETW + EIW)] - [ETW - APT x ETW / (ETW + EIW)] x 1/12 x PR (nil if below nil) and BDA x ETW / (ETW + EIW), and not more than the cl 5(1) amount; the deductible amount the employer claims by notice under cl 6(2)";
const VIC_RATE: &str = "Payroll Tax Act 2007 (Vic) Sch 2 cl 4(1) and (2) ((TW - D) x R + S + T) and cl 2(h): 4.85%, or 1.2125% for a regional employer (s 3A)";
const VIC_SURCHARGE: &str = "Payroll Tax Act 2007 (Vic) Sch 2 cl 4(3) (a month without interstate wages: nil to $833,333 of the month's taxable wages, (TW - 833,333) x 0.5% to $8,333,333, then TW x 1% - 45,833) or cl 4(4) (a month with interstate wages: nil while ETW + EIW is not more than $10 million, then [TW - ETW x 833,333 / (ETW + EIW)] x 0.5%, and above $100 million TW x 1% - [ETW x 833,333 + ETW x 8,333,333] / (ETW + EIW) x 0.5%): the mental health and wellbeing surcharge, S";
const VIC_COVID: &str = "Payroll Tax Act 2007 (Vic) Sch 2 cl 1B: the COVID-19 debt temporary payroll tax surcharge, T, equal to S for a financial year from 1 July 2023 to 30 June 2033";

const SHARED: &str = "(estimated_taxable_wages + estimated_interstate_wages)";

/// Victoria's cl 5(1) amount.
fn vic_limit(m: &Month, authority: &'static str, id: &'static str) -> Result<Step> {
    let [bda, apt, pr] = ["83333", "3000000", "0.5"].map(d);
    let fall = m.etw.sub(&apt)?.div(&int(12))?.mul(&pr)?;
    let value = py_min(bda.clone(), py_max(int(0), bda.sub(&fall)?)?)?;
    Ok(step(
        id,
        authority,
        "min(base_deductible_amount, max(0, base_deductible_amount - (estimated_taxable_wages - phase_out_threshold) / 12 x phase_out_rate))",
        &[&bda, &bda, &m.etw, &apt, &pr],
        value,
    ))
}

/// Victoria's cl 6(3) amount, capped by cl 6(4) at `limit`.
fn vic_claim(m: &Month, limit: &Step) -> Result<Step> {
    let [bda, apt, pr] = ["83333", "3000000", "0.5"].map(d);
    let share = m.shared(&bda)?;
    let fall = m.etw.sub(&m.shared(&apt)?)?.div(&int(12))?.mul(&pr)?;
    let first = py_max(int(0), share.sub(&fall)?)?;
    let value = py_min(limit.value.clone(), py_min(first, share)?)?;
    let formula = format!(
        "min(clause_5_amount, min(max(0, base_deductible_amount x estimated_taxable_wages / {SHARED} - (estimated_taxable_wages - phase_out_threshold x estimated_taxable_wages / {SHARED}) / 12 x phase_out_rate), base_deductible_amount x estimated_taxable_wages / {SHARED}))"
    );
    let o = [&m.etw, &m.eiw];
    let operands = [
        &limit.value,
        &bda,
        &m.etw,
        o[0],
        o[1],
        &m.etw,
        &apt,
        &m.etw,
        o[0],
        o[1],
        &pr,
        &bda,
        &m.etw,
        o[0],
        o[1],
    ];
    Ok(step("deduction", VIC_CLAIM, &formula, &operands, value))
}

const VIC_S: &str = "mental_health_and_wellbeing_surcharge";

/// A surcharge worked as `formula` on `operands`.
type Surcharge = Option<(String, Vec<Decimal>, Decimal)>;

/// Victoria's S by cl 4(3), for a month without interstate wages; `None`
/// when nil.
fn vic_local_surcharge(m: &Month) -> Result<Surcharge> {
    let [first, second] = ["833333", "8333333"].map(d);
    if !m.tw.gt(&first)? {
        return Ok(None);
    }
    if !m.tw.gt(&second)? {
        let value = m.tw.sub(&first)?.mul(&d("0.005"))?;
        let formula = "(taxable_wages - 833333) x 0.005".to_string();
        return Ok(Some((formula, vec![m.tw.clone()], value)));
    }
    let value = m.tw.mul(&d("0.01"))?.sub(&d("45833"))?;
    let formula = "taxable_wages x 0.01 - 45833".to_string();
    Ok(Some((formula, vec![m.tw.clone()], value)))
}

/// Victoria's S by cl 4(4), for a month with interstate wages; `None` when
/// nil.
fn vic_interstate_surcharge(m: &Month) -> Result<Surcharge> {
    let [first, second, half] = ["833333", "8333333", "0.005"].map(d);
    let total = m.estimated()?;
    if !total.gt(&d("10000000"))? {
        return Ok(None);
    }
    let [tw, etw, eiw] = [&m.tw, &m.etw, &m.eiw].map(Decimal::clone);
    if !total.gt(&d("100000000"))? {
        let value = m.tw.sub(&m.shared(&first)?)?.mul(&half)?;
        let f = format!("(taxable_wages - estimated_taxable_wages x 833333 / {SHARED}) x 0.005");
        return Ok(Some((f, vec![tw, etw.clone(), etw, eiw], value)));
    }
    let both = m.shared(&first)?.add(&m.shared(&second)?)?;
    let value = m.tw.mul(&d("0.01"))?.sub(&both.mul(&half)?)?;
    let f = format!(
        "taxable_wages x 0.01 - (estimated_taxable_wages x 833333 + estimated_taxable_wages x 8333333) / {SHARED} x 0.005"
    );
    let operands = vec![tw, etw.clone(), etw.clone(), etw, eiw];
    Ok(Some((f, operands, value)))
}

/// Victoria's S, by cl 4(3) or (4).
fn vic_surcharge(m: &Month) -> Result<Step> {
    let worked = if m.iw.is_zero() {
        vic_local_surcharge(m)?
    } else {
        vic_interstate_surcharge(m)?
    };
    let Some((formula, operands, value)) = worked else {
        return Ok(nil(VIC_S, VIC_SURCHARGE));
    };
    if value.lt(&int(0))? {
        return refuse(
            "payroll_tax_monthly.surcharge_negative",
            "The month's taxable wages are below the employer's share of the surcharge threshold, so Payroll Tax Act 2007 (Vic) Sch 2 cl 4(4)'s formula gives less than nil, and the clause does not say how a negative surcharge is treated.",
        );
    }
    let operands: Vec<&Decimal> = operands.iter().collect();
    Ok(step(VIC_S, VIC_SURCHARGE, &formula, &operands, value))
}

pub(super) fn vic(m: &Month) -> Result<Worked> {
    let deduction = if m.iw.is_zero() {
        vic_limit(m, VIC_LOCAL, "deduction")?
    } else {
        let limit = vic_limit(m, VIC_LIMIT, "clause_5_amount")?;
        let claim = vic_claim(m, &limit)?;
        return vic_charges(m, vec![limit], claim);
    };
    vic_charges(m, vec![], deduction)
}

fn vic_charges(m: &Month, mut steps: Vec<Step>, deduction: Step) -> Result<Worked> {
    let applied = d(if m.regional { "0.012125" } else { "0.0485" });
    steps.extend(taxed(m, deduction, (VIC_RATE, &applied))?);
    let surcharge = vic_surcharge(m)?;
    let operands = [&surcharge.value];
    let covid = step(
        "covid_19_debt_temporary_surcharge",
        VIC_COVID,
        "mental_health_and_wellbeing_surcharge",
        &operands,
        surcharge.value.clone(),
    );
    steps.extend([surcharge, covid]);
    let rates = vec![
        rate("base_deductible_amount", d("83333")),
        rate("phase_out_threshold", d("3000000")),
        rate("phase_out_rate", d("0.5")),
        rate("rate", &applied),
        rate("surcharge_rate", d("0.005")),
    ];
    Ok(Worked {
        steps,
        charges: 3,
        rates,
    })
}

const QLD_ACTUAL: &str = "Payroll Tax Act 1971 (Qld) ss 17 and 20(2)(c), current as at 22 October 2026, read on 26 November 2026: the actual periodic deduction, the greater of nil and FME / G - (TW - FME / G) / 7, with E $108,333 and a whole month (F = G, M = 1), for an employer that pays no interstate wages in the month and is not a previous interstate wage payer";
const QLD_FIXED: &str = "Payroll Tax Act 1971 (Qld) ss 17 and 20(2)(a) and (b): the fixed periodic deduction, the greater of nil and QW / AW x (E - (AW / 12 - E) / 7) x M, with E $108,333, M = 1 and QW and AW the employer's estimated taxable wages and estimated taxable and interstate wages for the year, worked on the most recent calculation day (s 18)";
const QLD_RATE: &str = "Payroll Tax Act 1971 (Qld) ss 10(1), (2) and (4) and 20(1)(b): 4.75%, or 4.95% when the month's taxable and interstate wages together are more than $541,666";
const QLD_PRIMARY_T: &str = "Payroll Tax Act 1971 (Qld) ss 43C(1) and 43E(3): the primary periodic threshold, $10 million x C / D x M / 12, C and D the estimated taxable wages and estimated taxable and interstate wages for the year";
const QLD_ADDITIONAL_T: &str = "Payroll Tax Act 1971 (Qld) ss 43D(1) and 43E(4): the additional periodic threshold, $100 million x C / D x M / 12";
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

/// A periodic threshold, `whole` x C / D / 12, and its levy at `charge`,
/// the rate named `rate_name`.
fn qld_levy(
    m: &Month,
    (threshold_id, threshold_authority): (&'static str, &'static str),
    (id, authority): (&'static str, &'static str),
    (whole, charge, rate_name): (&Decimal, &Decimal, &str),
) -> Result<[Step; 2]> {
    let value = m.shared(whole)?.div(&int(12))?;
    let formula = format!("{whole} x estimated_taxable_wages / {SHARED} / 12");
    let threshold = step(
        threshold_id,
        threshold_authority,
        &formula,
        &[&m.etw, &m.etw, &m.eiw],
        value,
    );
    let levy = py_max(int(0), m.tw.sub(&threshold.value)?.mul(charge)?)?;
    let formula = format!("max(0, (taxable_wages - {threshold_id}) x {rate_name})");
    let operands = [&m.tw, &threshold.value, charge];
    let levy = step(id, authority, &formula, &operands, levy);
    Ok([threshold, levy])
}

pub(super) fn qld(m: &Month) -> Result<Worked> {
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
    let charged = taxed(m, qld_deduction(m, &e)?, (QLD_RATE, &applied))?;
    let [pt, pl] = qld_levy(
        m,
        ("primary_periodic_threshold", QLD_PRIMARY_T),
        ("mental_health_levy_primary", QLD_PRIMARY),
        (&d("10000000"), &low, "levy_rate"),
    )?;
    let [at, al] = qld_levy(
        m,
        ("additional_periodic_threshold", QLD_ADDITIONAL_T),
        ("mental_health_levy_additional", QLD_ADDITIONAL),
        (&d("100000000"), &high, "additional_levy_rate"),
    )?;
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

const WA_RATE: &str = "Pay-roll Tax Act 2002 (WA) s 5(4), version 00-d0-02, read on 26 November 2026: 5.5% on wages paid or payable on or after 1 July 2023";
const WA_NIL: &str = "Pay-roll Tax Assessment Act 2002 (WA) s 11(1)(a) and s 12(1), version 04-d0-03, read on 26 November 2026: a local non-group employer whose WA taxable wages for the month are not more than the monthly threshold amount, $83,333 (s 8(2)), pays no pay-roll tax, so the whole of them is deducted";
const WA_FULL: &str = "Pay-roll Tax Assessment Act 2002 (WA) s 11(1)(b) and s 12(2): at or above the apportioned upper threshold, $7,500,000 / 12 = $625,000 (s 8(3)), the rate applies to all the month's WA taxable wages";
const WA_TAPER: &str = "Pay-roll Tax Assessment Act 2002 (WA) s 11(1)(c) and (2): the deductable amount AT - (W - AT) x TV, with AT the monthly threshold amount $83,333 (ss 8(2) and 12(1)) and TV the tapering value AT / (UT - AT) for the year, $1,000,000 / $6,500,000 (s 8(4))";
const WA_NOMINATED: &str = "Pay-roll Tax Assessment Act 2002 (WA) s 15(1), (2) and (7): an interstate non-group employer's deduction is the amount the Commissioner nominated (nil if none), as stated; the rate applies to the month's WA taxable wages less it, never below nil";

pub(super) fn wa(m: &Month) -> Result<Worked> {
    let applied = d("0.055");
    let deduction = if let Some(stated) = &m.stated {
        step(
            "deduction",
            WA_NOMINATED,
            "nominated_deduction",
            &[stated],
            stated.clone(),
        )
    } else {
        let [at, upper] = ["83333", "625000"].map(d);
        let tv = d("1000000").div(&d("6500000"))?;
        if !m.tw.gt(&at)? {
            step("deduction", WA_NIL, "taxable_wages", &[&m.tw], m.tw.clone())
        } else if !m.tw.lt(&upper)? {
            step("deduction", WA_FULL, "0", &[], int(0))
        } else {
            let value = at.sub(&m.tw.sub(&at)?.mul(&tv)?)?;
            step(
                "deduction",
                WA_TAPER,
                "monthly_threshold - (taxable_wages - monthly_threshold) x 1000000 / 6500000",
                &[&at, &m.tw, &at],
                value,
            )
        }
    };
    let steps = taxed(m, deduction, (WA_RATE, &applied))?;
    let rates = vec![
        rate("monthly_threshold", d("83333")),
        rate("upper_threshold", d("7500000")),
        rate("rate", &applied),
    ];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

const SA_DEDUCTION: &str = "Payroll Tax Act 2009 (SA) Sch 2 cl 6(3) and (4), version 31.3.2026, read on 26 November 2026: D = T / (T + I) x the prescribed amount, $50,000 (cl 2), T and I the estimated taxable and interstate wages for the year; the deductible amount the employer claims by notice under cl 6(2)";
const SA_NIL: &str = "Payroll Tax Act 2009 (SA) Sch 2 cl 5(1a)(b) sets a rate only when the total annualised relevant wages (the year's estimated taxable and interstate wages, cl 5(1b)) are more than $1,500,000, and Sch 1 cl 4 imposes no tax for a year at or below the threshold: nil";
const SA_PHASED: &str = "Payroll Tax Act 2009 (SA) Sch 2 cl 5(1) and (1a)(b)(i): R = (TARW - $1,500,000) / $200,000 x 4.95% when the total annualised relevant wages are more than $1,500,000 and not more than $1,700,000";
const SA_FULL: &str = "Payroll Tax Act 2009 (SA) Sch 2 cl 5(1) and (1a)(b)(ii): 4.95% when the total annualised relevant wages are more than $1,700,000";

pub(super) fn sa(m: &Month) -> Result<Worked> {
    let [prescribed, threshold, full_from, full] = ["50000", "1500000", "1700000", "0.0495"].map(d);
    let value = m.shared(&prescribed)?;
    let formula = format!("estimated_taxable_wages / {SHARED} x prescribed_amount");
    let operands = [&m.etw, &m.etw, &m.eiw, &prescribed];
    let deduction = step("deduction", SA_DEDUCTION, &formula, &operands, value);
    let tarw = m.estimated()?;
    let (authority, applied) = if !tarw.gt(&threshold)? {
        (SA_NIL, int(0))
    } else if tarw.gt(&full_from)? {
        (SA_FULL, full.clone())
    } else {
        let phased = tarw.sub(&threshold)?.div(&d("200000"))?.mul(&full)?;
        (SA_PHASED, phased)
    };
    let steps = taxed(m, deduction, (authority, &applied))?;
    let shown = applied.to_string();
    let shown = if shown.contains('.') {
        shown.trim_end_matches('0').to_string()
    } else {
        shown
    };
    let rates = vec![
        rate("prescribed_amount", &prescribed),
        rate("threshold", &threshold),
        rate("full_rate", &full),
        rate("rate", shown),
    ];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

const ACT_LOCAL: &str = "Payroll Tax Act 2011 (ACT) Sch 2 s 2.3(1) (republication R20, effective 26 December 2025) and the Taxation Administration (Amounts and Rates—Payroll Tax) Determination 2026 (DI2026-151) s 5(b), both read on 26 November 2026: for a month without interstate wages, the monthly threshold amount for s 86, $145,833.33";
const ACT_SHARED: &str = "Payroll Tax Act 2011 (ACT) Sch 2 s 2.4(3) to (5): D = A x T / (T + I), A the s 86 amount ($145,833.33, DI2026-151 s 5(b)) and T and I the estimated taxable and interstate wages for the year; the deductible amount the employer claims by notice under s 2.4(2)";
const ACT_RATE: &str = "Payroll Tax Act 2011 (ACT) Sch 2 s 2.2 and DI2026-151 s 5(d)(ii) and Table 2, by the month's total wages (taxable and interstate): 6.75% to $1,666,666.66, 6.85% to $4,166,666.66, 7.35% to $8,333,333.33, 7.85% to $12,500,000 and 8.75% above";
const ACT_NONE: &str = "Payroll Tax Act 2011 (ACT) Sch 2 s 2.2(2): the deduction is equal to or more than the month's taxable wages, so no payroll tax is payable (DI2026-151 Table 2 sets no rate for total wages of $145,833.33 or less)";

/// The ACT's Table 2 rate on the month's total wages, or nil when the
/// deduction covers the taxable wages.
fn act_rate(
    m: &Month,
    threshold: &Decimal,
    deduction: &Decimal,
) -> Result<(&'static str, Decimal)> {
    let total = m.tw.add(&m.iw)?;
    if !total.gt(threshold)? {
        if m.tw.gt(deduction)? {
            return refuse(
                "payroll_tax_monthly.rate_unset",
                "DI2026-151 Table 2 sets no monthly rate for total wages of $145,833.33 or less, yet the month's taxable wages are more than the deductible amount.",
            );
        }
        return Ok((ACT_NONE, int(0)));
    }
    let bands = [
        ("1666666.66", "0.0675"),
        ("4166666.66", "0.0685"),
        ("8333333.33", "0.0735"),
        ("12500000", "0.0785"),
    ];
    for (to, band) in bands {
        if !total.gt(&d(to))? {
            return Ok((ACT_RATE, d(band)));
        }
    }
    Ok((ACT_RATE, d("0.0875")))
}

pub(super) fn act(m: &Month) -> Result<Worked> {
    let threshold = d("145833.33");
    let deduction = if m.iw.is_zero() {
        step(
            "deduction",
            ACT_LOCAL,
            "monthly_threshold",
            &[&threshold],
            threshold.clone(),
        )
    } else {
        let formula = format!("monthly_threshold x estimated_taxable_wages / {SHARED}");
        let operands = [&threshold, &m.etw, &m.etw, &m.eiw];
        step(
            "deduction",
            ACT_SHARED,
            &formula,
            &operands,
            m.shared(&threshold)?,
        )
    };
    let (authority, applied) = act_rate(m, &threshold, &deduction.value)?;
    let steps = taxed(m, deduction, (authority, &applied))?;
    let rates = vec![
        rate("monthly_threshold", &threshold),
        rate("rate", &applied),
    ];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}

const NT_DEDUCTION: &str = "Payroll Tax Act 2009 (NT) Sch 2 cll 6(1) and 7, as in force at 1 July 2026, read on 26 November 2026: the deductible amount determined in a manner approved by the Commissioner, as stated, not more than TA / 12 (TA $2,500,000, Sch 1 cl 1)";
const NT_RATE: &str = "Payroll Tax Act 2009 (NT) Sch 2 cl 5 ((TW - D) x R) and s 7A: 5.5%, or 6.5% when the taxable and interstate wages for the year (here as estimated) are equal to or greater than $100,000,000";

pub(super) fn nt(m: &Month) -> Result<Worked> {
    let cap = d("2500000").div(&int(12))?;
    let stated = m
        .stated
        .clone()
        .expect("required for the Northern Territory");
    if stated.gt(&cap)? {
        return refuse(
            "payroll_tax_monthly.deduction_above_cap",
            "stated_deduction cannot be more than the threshold amount over 12, $208,333.33 (Payroll Tax Act 2009 (NT) Sch 2 cll 6(1) and 7(2)).",
        );
    }
    let deduction = step(
        "deduction",
        NT_DEDUCTION,
        "stated_deduction",
        &[&stated],
        stated.clone(),
    );
    let high = !m.estimated()?.lt(&d("100000000"))?;
    let applied = d(if high { "0.065" } else { "0.055" });
    let steps = taxed(m, deduction, (NT_RATE, &applied))?;
    let rates = vec![rate("threshold", d("2500000")), rate("rate", &applied)];
    Ok(Worked {
        steps,
        charges: 1,
        rates,
    })
}
