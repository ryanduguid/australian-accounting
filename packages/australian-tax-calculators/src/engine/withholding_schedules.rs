// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! PAYG withholding beside Schedule 1: Schedule 2 (horticulture and shearing),
//! Schedule 14 (an agreed additional amount), the daily and casual table, and
//! a payment for a supply where no ABN is quoted (TAA 1953 Sch 1 s 12-190).
//! The schedules are those of the Taxation Administration (Withholding
//! Schedules) Instruments 2024 (F2024L00664), 2025 (F2025L00647), (No. 2) 2025
//! (F2025L01016) and 2026 (F2026L00716), whose Schedules 2 and 14 and daily and
//! casual steps are the same in each year; the ATO's tax tables for daily and
//! casual workers (from 1 July 2024 and from 1 July 2026); s 12-190 from the
//! ATO's legal database; the Taxation Administration Regulations 2017 ss 4, 26
//! and 38; and the A New Tax System (Goods and Services Tax) Regulations 2019
//! s 29-80.01. All read on 22 November 2026.

use serde_json::Value;

use super::{
    Arg, PaygOptions, Result, SchedulePay, Step, d, derived_result, flag, lookup, money,
    payg_checks, payg_options, payg_rows, precise, rate, refuse, scope, step, weekly_formula,
};
use crate::decimal::{Decimal, Rounding};
use crate::metadata as meta;

/// Each schedule's rates when no TFN is quoted: resident, foreign resident.
const NO_TFN_RATES: (&str, &str) = ("0.47", "0.45");

/// A refusal for a working holiday maker, whom Schedule 15 governs.
fn not_working_holiday_maker(value: &Arg, schedule: &str) -> Result<()> {
    if flag(value, "working_holiday_maker")? {
        return refuse(
            "withholding.working_holiday_maker",
            format!(
                "{schedule} does not apply to a working holiday maker: Schedule 15 governs every payment to them (working-holiday-maker-withholding)."
            ),
        );
    }
    Ok(())
}

/// The input a flat rate applies to: its name and value.
type Base<'a> = (&'static str, &'a Decimal);

/// `floor(input)` and then `rate x` it, cents in the result ignored or
/// rounded to the dollar: the 2 steps every flat rate here takes.
fn flat_rate_steps(
    (input, amount): Base,
    a: &str,
    rounding: Rounding,
    authorities: (&'static str, &'static str),
) -> Result<Vec<Step>> {
    let a = d(a);
    let used_id = format!("{input}_used");
    let round = if matches!(rounding, Rounding::Down) {
        "floor"
    } else {
        "dollars"
    };
    precise(|| {
        let used = amount.quantize(0, Rounding::Down)?;
        let withheld = a.mul(&used)?.quantize(0, rounding)?;
        Ok(vec![
            step(
                used_id.clone(),
                authorities.0,
                &format!("floor({input})"),
                &[amount],
                used.clone(),
            ),
            step(
                "withholding",
                authorities.1,
                &format!("{round}(rate x {used_id})"),
                &[&a, &used],
                withheld,
            ),
        ])
    })
}

/// 47%, or 45% for a foreign resident, of the amount with cents ignored, cents
/// in the result ignored: the no TFN rule of each schedule here.
fn no_tfn_steps(base: Base, foreign: bool, schedule: &'static str) -> Result<Vec<Step>> {
    let a = if foreign {
        NO_TFN_RATES.1
    } else {
        NO_TFN_RATES.0
    };
    flat_rate_steps(
        base,
        a,
        Rounding::Down,
        (
            schedule,
            "The schedule's 'When a TFN has not been provided': 47% for a resident, 45% for a foreign resident, ignoring any cents",
        ),
    )
}

const HORTICULTURE: &str = "horticulture_shearing_withholding";

/// Schedule 2's eligibility: neither a working holiday maker nor a labour
/// mobility worker, and a payee who quotes a TFN has claimed the tax-free
/// threshold.
fn horticulture_eligible(
    tfn: bool,
    threshold_claimed: &Arg,
    working_holiday_maker: &Arg,
    labour_mobility_scheme: &Arg,
) -> Result<()> {
    let claimed = flag(threshold_claimed, "tax_free_threshold_claimed")?;
    not_working_holiday_maker(working_holiday_maker, "Schedule 2")?;
    if flag(labour_mobility_scheme, "labour_mobility_scheme")? {
        return refuse(
            "horticulture_shearing_withholding.labour_mobility_scheme",
            "Schedule 2 does not apply to a worker under the Seasonal Worker Programme, Pacific Labour Scheme or Pacific Australia Labour Mobility scheme; see the ATO's withholding rates for those schemes.",
        );
    }
    if tfn && !claimed {
        return refuse(
            "horticulture_shearing_withholding.threshold_not_claimed",
            "Schedule 2 is for a worker who has given a TFN declaration claiming the tax-free threshold; use payg-withholding (Schedule 1) otherwise.",
        );
    }
    Ok(())
}

/// Schedule 2: 13% (30% for a foreign resident) of the earnings for the pay
/// period with cents ignored, rounded to the dollar, whatever the pay period;
/// 47% or 45% with cents ignored when no TFN is quoted.
#[allow(clippy::too_many_arguments)]
pub fn horticulture_shearing_withholding(
    earnings: &Arg,
    tfn_quoted: &Arg,
    foreign_resident: &Arg,
    tax_free_threshold_claimed: &Arg,
    working_holiday_maker: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    labour_mobility_scheme: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, HORTICULTURE)?;
    let earnings = money(earnings)?;
    let tfn = flag(tfn_quoted, "tfn_quoted")?;
    let foreign = flag(foreign_resident, "foreign_resident")?;
    horticulture_eligible(
        tfn,
        tax_free_threshold_claimed,
        working_holiday_maker,
        labour_mobility_scheme,
    )?;
    let used =
        "Schedule 2, 'Using a formula': x is the earnings for the pay period, ignoring any cents";
    let a = match (tfn, foreign) {
        (true, false) => "0.13",
        (true, true) => "0.30",
        (false, false) => NO_TFN_RATES.0,
        (false, true) => NO_TFN_RATES.1,
    };
    let steps = if tfn {
        flat_rate_steps(
            ("earnings", &earnings),
            a,
            Rounding::HalfUp,
            (
                used,
                "Schedule 2, 'Using a formula', Table A (TFN provided) and 'Rounding of withholding amounts': to the nearest dollar, 50 cents up",
            ),
        )?
    } else {
        no_tfn_steps(("earnings", &earnings), foreign, used)?
    };
    derived_result(HORTICULTURE, &year, steps, vec![rate("rate", a)])
}

const UPWARD: &str = "upward_variation_withholding";

/// Schedule 14: the Schedule 1 amount for the pay period, as payg-withholding
/// works it out, plus the additional amount the payee asked for in writing.
pub fn upward_variation_withholding(
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    additional_amount: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    options: &PaygOptions,
) -> Result<Value> {
    let (year, earnings, period, scale) =
        payg_checks(UPWARD, earnings, pay_period, scale, year, scope_confirmed)?;
    let additional = money(additional_amount)?;
    let rules = *lookup(meta::PAYG_WITHHOLDING_RULES, &year);
    let pay = SchedulePay {
        choices: payg_options(options, scale, &period)?,
        rules,
        year,
        earnings,
        period,
        scale,
    };
    let mut w = precise(|| pay.workings())?;
    let mut normal = w.steps.pop().expect("a withholding step");
    normal.id = "normal_withholding".into();
    let total = precise(|| Ok(normal.value.add(&additional)?))?;
    let added = step(
        "withholding",
        "Schedule 14, 'How to work out the withholding amount', step 3: the normal amount plus the additional amount",
        "normal_withholding + additional_amount",
        &[&normal.value, &additional],
        total,
    );
    w.steps.push(normal);
    w.steps.push(step(
        "additional_amount",
        "Schedule 14, step 2: the additional amount the payee asked for in writing",
        "additional_amount",
        &[&additional],
        additional.clone(),
    ));
    w.steps.push(added);
    let label = format!("{} {}", pay.year, pay.period);
    derived_result(UPWARD, &label, w.steps, w.rates)
}

const DAILY: &str = "daily_casual_withholding";

/// The tax table for daily and casual workers: a payee who quotes a TFN must
/// have claimed the tax-free threshold and be a resident.
fn daily_eligible(tfn: bool, foreign: bool, threshold_claimed: &Arg) -> Result<()> {
    let claimed = flag(threshold_claimed, "tax_free_threshold_claimed")?;
    if tfn && !claimed {
        return refuse(
            "daily_casual_withholding.threshold_not_claimed",
            "The tax table for daily and casual workers is for a payee who has claimed the tax-free threshold; use payg-withholding (Schedule 1) otherwise.",
        );
    }
    if tfn && foreign {
        return refuse(
            "daily_casual_withholding.foreign_resident_tfn",
            "The tax table for daily and casual workers gives weekly foreign resident rates but no step from daily earnings to them, so a foreign resident who quoted a TFN is not worked out here.",
        );
    }
    Ok(())
}

/// Schedule 1, 'Calculating withholding amounts for payments made on a daily
/// or casual basis': scale 2 on 5 times the daily earnings plus 99 cents,
/// rounded to the dollar, then a fifth of it rounded to the dollar.
fn daily_steps(year: &str, earnings: &Decimal) -> Result<(Vec<Step>, Decimal, Decimal)> {
    let rows = payg_rows(year, 2).expect("scale 2 in every year");
    precise(|| {
        let x = earnings
            .quantize(0, Rounding::Down)?
            .mul(&d("5"))?
            .add(&d("0.99"))?;
        let (a, b, weekly) = weekly_formula(&x, rows)?;
        let daily = weekly.div(&d("5"))?.quantize(0, Rounding::HalfUp)?;
        let steps = vec![
            step(
                "weekly_earnings_used",
                "Schedule 1, daily or casual basis, step 1: earnings ignoring any cents, times 5, plus 99 cents",
                "floor(earnings) x 5 + 0.99",
                &[earnings],
                x.clone(),
            ),
            step(
                "weekly_withholding",
                "Schedule 1, daily or casual basis, steps 2 and 3: the scale 2 coefficients, rounded to the nearest dollar",
                "max(0, dollars(a x weekly_earnings_used - b))",
                &[&a, &x, &b],
                weekly.clone(),
            ),
            step(
                "daily_withholding",
                "Schedule 1, daily or casual basis, steps 4 and 5: divided by 5 and rounded to the nearest dollar",
                "dollars(weekly_withholding / 5)",
                &[&weekly],
                daily,
            ),
        ];
        Ok((steps, a, b))
    })
}

/// The tax offsets' daily value, 0.38% of the yearly amount claimed rounded to
/// the dollar, taken off the daily amount; more than the amount is refused.
/// Without offsets the daily amount is withheld as it is.
fn daily_offsets(mut steps: Vec<Step>, offsets: Option<Decimal>) -> Result<Vec<Step>> {
    let before = steps.last().expect("a daily step").value.clone();
    let Some(offsets) = offsets else {
        steps.push(step(
            "withholding",
            "Tax table for daily and casual workers, 'Working out the withholding amount': no tax offsets claimed",
            "daily_withholding",
            &[&before],
            before.clone(),
        ));
        return Ok(steps);
    };
    let reduction = precise(|| Ok(offsets.mul(&d("0.0038"))?.quantize(0, Rounding::HalfUp)?))?;
    if before.lt(&reduction)? {
        return refuse(
            "daily_casual_withholding.offsets_exceed_withholding",
            "The daily value of the tax offsets exceeds the daily amount withheld before them; the tax table does not say how to treat the excess.",
        );
    }
    let net = precise(|| Ok(before.sub(&reduction)?))?;
    steps.push(step(
        "tax_offset_reduction",
        "Tax table for daily and casual workers, 'Claiming tax offsets': the daily value is 0.38% of the amount claimed, rounded to the nearest dollar",
        "dollars(tax_offsets x 0.0038)",
        &[&offsets],
        reduction.clone(),
    ));
    steps.push(step(
        "withholding",
        "Tax table for daily and casual workers, 'Working out the withholding amount', step 2",
        "daily_withholding - tax_offset_reduction",
        &[&before, &reduction],
        net,
    ));
    Ok(steps)
}

/// The tax table for daily and casual workers, with the Schedule 1 formula.
#[allow(clippy::too_many_arguments)]
pub fn daily_casual_withholding(
    earnings: &Arg,
    tfn_quoted: &Arg,
    foreign_resident: &Arg,
    tax_free_threshold_claimed: &Arg,
    working_holiday_maker: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    tax_offsets: Option<&Arg>,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, DAILY)?;
    let earnings = money(earnings)?;
    let tfn = flag(tfn_quoted, "tfn_quoted")?;
    let foreign = flag(foreign_resident, "foreign_resident")?;
    daily_eligible(tfn, foreign, tax_free_threshold_claimed)?;
    not_working_holiday_maker(
        working_holiday_maker,
        "The tax table for daily and casual workers",
    )?;
    let offsets = tax_offsets.map(money).transpose()?;
    if !tfn {
        if offsets.as_ref().is_some_and(|o| !o.is_zero()) {
            return refuse(
                "daily_casual_withholding.tax_offsets_not_allowed",
                "Tax offsets do not reduce withholding when no TFN has been provided.",
            );
        }
        let steps = no_tfn_steps(
            ("earnings", &earnings),
            foreign,
            "Tax table for daily and casual workers, 'When a TFN has not been provided': the payment, ignoring any cents",
        )?;
        let a = if foreign {
            NO_TFN_RATES.1
        } else {
            NO_TFN_RATES.0
        };
        return derived_result(DAILY, &year, steps, vec![rate("rate", a)]);
    }
    let (steps, a, b) = daily_steps(&year, &earnings)?;
    let steps = daily_offsets(steps, offsets)?;
    let rates = vec![rate("scale", 2), rate("a", &a), rate("b", &b)];
    derived_result(DAILY, &year, steps, rates)
}

const NO_ABN: &str = "no_abn_withholding";

/// The s 12-190(4)(b) threshold: $50 in the Act, raised to $75 by GST
/// Regulations s 29-80.01 for GST Act s 29-80(1).
const NO_ABN_THRESHOLD: &str = "75";
/// Taxation Administration Regulations 2017 s 4 'top rate': the highest Income
/// Tax Rates Act 1986 Sch 7 Part I rate (45%) plus the Medicare levy (2%).
const TOP_RATE: &str = "0.47";

/// s 12-190: the top rate of the payment, cents in the payment and in the
/// result disregarded (Regulations s 26); nil when the supply's payments,
/// disregarding GST, are $75 or less (s 12-190(4)(b)).
pub fn no_abn_withholding(
    payment: &Arg,
    gst_included: &Arg,
    supply_total_excluding_gst: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, NO_ABN)?;
    let payment = money(payment)?;
    let gst = money(gst_included)?;
    let total = money(supply_total_excluding_gst)?;
    let excluding = precise(|| Ok(payment.sub(&gst)?))?;
    if excluding.lt(&d("0"))? {
        return refuse(
            "no_abn_withholding.gst_above_payment",
            "gst_included is more than the payment.",
        );
    }
    if total.lt(&excluding)? {
        return refuse(
            "no_abn_withholding.total_below_payment",
            "supply_total_excluding_gst is less than this payment without its GST; the total includes this payment.",
        );
    }
    let threshold = d(NO_ABN_THRESHOLD);
    let steps = if total.gt(&threshold)? {
        flat_rate_steps(
            ("payment", &payment),
            TOP_RATE,
            Rounding::Down,
            (
                "Taxation Administration Regulations 2017 s 26(1)(a), (2) and (3): cents in the payment disregarded",
                "Regulations s 38(2): the top rate (s 4: 45% plus the 2% Medicare levy) times the payment; s 26(1)(b), (2) and (3): cents in the result disregarded",
            ),
        )?
    } else {
        vec![step(
            "withholding",
            "TAA 1953 Sch 1 s 12-190(4)(b) and GST Regulations 2019 s 29-80.01: no withholding when the payments for the supply, disregarding GST, do not exceed $75",
            "0",
            &[],
            d("0"),
        )]
    };
    let rates = vec![rate("rate", TOP_RATE), rate("threshold", NO_ABN_THRESHOLD)];
    derived_result(NO_ABN, &year, steps, rates)
}
