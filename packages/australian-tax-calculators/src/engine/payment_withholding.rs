// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! PAYG withholding for particular payments: Schedule 3 (actors, variety
//! artists and other entertainers), Schedule 4 (return to work payments),
//! Schedule 6 (annuities) and Schedule 10 (voluntary agreements), from the
//! Taxation Administration (Withholding Schedules) Instruments 2024
//! (F2024L00664), 2025 (F2025L00647), (No. 2) 2025 (F2025L01016) and 2026
//! (F2026L00716), read on 22 November 2026. Schedules 4, 6 and 10 are the same
//! in substance in each; Schedule 3's coefficients changed on 1 July 2026.

use serde_json::Value;

use super::withholding_schedules::{NO_TFN_RATES, no_tfn_steps, not_working_holiday_maker};
use super::{
    Arg, CoefficientRow, PaygOptions, Result, SchedulePay, Step, d, derived_result, finite_between,
    flag, int, lookup, money, payg_checks, payg_options, precise, rate, refuse, scope, step,
    weekly_formula,
};
use crate::decimal::{Decimal, Rounding};
use crate::metadata as meta;

/// A whole number from `low` to `high`.
fn count(value: &Arg, name: &str, low: i64, high: i64) -> Result<i64> {
    match value.int() {
        Some(Some(n)) if (low..=high).contains(&n) => Ok(n),
        _ => refuse(
            "input.count_invalid",
            format!("{name} must be an integer from {low} to {high}."),
        ),
    }
}

/// A result's rates, by key.
type Rates = Vec<(String, String)>;

/// The no TFN rate's key and value: 47%, or 45% for a foreign resident.
fn no_tfn_rate(foreign: bool) -> Rates {
    let a = if foreign {
        NO_TFN_RATES.1
    } else {
        NO_TFN_RATES.0
    };
    vec![rate("rate", a)]
}

const ENTERTAINER: &str = "entertainer_withholding";

/// Schedule 3, Table A (tax-free threshold claimed) and Table B (not
/// claimed), for payments from 1 July 2024 to 30 June 2026 (F2024L00664,
/// F2025L00647 and F2025L01016): weekly earnings less than the limit, a, b.
#[rustfmt::skip]
const SCHEDULE_3_FROM_JULY_2024: [&[CoefficientRow]; 2] = [
    &[(Some(451), "0", "0"), (Some(625), "0.1280", "57.8462"), (Some(781), "0.2080", "107.8462"),
      (Some(901), "0.1440", "57.8462"), (Some(1081), "0.1512", "64.3365"),
      (Some(1602), "0.2582", "180.0385"), (Some(3245), "0.2560", "176.5769"),
      (Some(4567), "0.3120", "358.3077"), (None, "0.3760", "650.6154")],
    &[(Some(187), "0.1280", "0.1280"), (Some(463), "0.1694", "7.7550"), (Some(644), "0.1512", "-0.6702"),
      (Some(1165), "0.2582", "68.2367"), (Some(2807), "0.2560", "65.7202"),
      (Some(4129), "0.3120", "222.9510"), (None, "0.3760", "487.2587")],
];

/// Schedule 3, Tables A and B, for payments from 1 July 2026 (F2026L00716).
#[rustfmt::skip]
const SCHEDULE_3_FROM_JULY_2026: [&[CoefficientRow]; 2] = [
    &[(Some(452), "0", "0"), (Some(673), "0.1200", "54.3462"), (Some(841), "0.2000", "108.2135"),
      (Some(901), "0.1360", "54.3473"), (Some(1081), "0.1432", "60.8377"),
      (Some(1602), "0.2582", "185.1935"), (Some(3245), "0.2560", "181.7319"),
      (Some(4567), "0.3120", "363.4627"), (None, "0.3760", "655.7704")],
    &[(Some(235), "0.1200", "0.1200"), (Some(463), "0.1667", "11.0185"), (Some(644), "0.1432", "0.1066"),
      (Some(1165), "0.2582", "74.1674"), (Some(2807), "0.2560", "71.6508"),
      (Some(4129), "0.3120", "228.8816"), (None, "0.3760", "493.1893")],
];

/// The performances in the week and in the day.
type Performances = (i64, i64);

/// Schedule 3's table for the year: A when the tax-free threshold is claimed,
/// otherwise B.
fn schedule_3_rows(year: &str, threshold: bool) -> &'static [CoefficientRow] {
    let tables = if year == "2026-27" {
        SCHEDULE_3_FROM_JULY_2026
    } else {
        SCHEDULE_3_FROM_JULY_2024
    };
    tables[usize::from(!threshold)]
}

/// Schedule 3's eligibility, and who may claim tax offsets: not a working
/// holiday maker, no foreign resident with a TFN (the schedule gives weekly
/// foreign resident rates with no step from performance earnings), and tax
/// offsets only with a TFN, the tax-free threshold and one performance a day.
fn entertainer_eligible(
    tfn: bool,
    foreign: bool,
    threshold: bool,
    offsets: bool,
    per_day: i64,
) -> Result<()> {
    if tfn && foreign {
        return refuse(
            "entertainer_withholding.foreign_resident_tfn",
            "Schedule 3 gives weekly foreign resident rates but no step from performance earnings to them, so a foreign resident who quoted a TFN is not worked out here.",
        );
    }
    if offsets && !(tfn && threshold) {
        return refuse(
            "entertainer_withholding.tax_offsets_not_allowed",
            "Schedule 3 allows no tax offsets when no TFN has been provided or the tax-free threshold has not been claimed.",
        );
    }
    if offsets && per_day > 1 {
        return refuse(
            "entertainer_withholding.offsets_several_performances",
            "Schedule 3 gives the tax offsets a per performance value but no daily value when there is more than one performance a day.",
        );
    }
    Ok(())
}

/// Schedule 3, 'Using a formula': the earnings for a performance with cents
/// ignored, times the week's performances, plus 99 cents; Table A or B,
/// rounded to the dollar; then per performance and times the day's.
fn entertainer_steps(
    year: &str,
    earnings: &Decimal,
    (week, day): Performances,
    threshold: bool,
) -> Result<(Vec<Step>, Rates)> {
    let rows = schedule_3_rows(year, threshold);
    let (week, day) = (int(week), int(day));
    precise(|| {
        let x = earnings
            .quantize(0, Rounding::Down)?
            .mul(&week)?
            .add(&d("0.99"))?;
        let (a, b, weekly) = weekly_formula(&x, rows)?;
        let daily = weekly
            .div(&week)?
            .mul(&day)?
            .quantize(0, Rounding::HalfUp)?;
        let table = if threshold { "A" } else { "B" };
        let steps = vec![
            step(
                "weekly_earnings_used",
                "Schedule 3, 'Using a formula', step 1: the earnings for a performance ignoring any cents, times the performances for the week, plus 99 cents",
                "floor(earnings) x performances_per_week + 0.99",
                &[earnings, &week],
                x.clone(),
            ),
            step(
                "weekly_withholding",
                "Schedule 3, 'Using a formula', step 2: Table A (tax-free threshold claimed) or B, rounded to the nearest dollar",
                "max(0, dollars(a x weekly_earnings_used - b))",
                &[&a, &x, &b],
                weekly.clone(),
            ),
            step(
                "daily_withholding",
                "Schedule 3, 'Using a formula', step 3: divided by the performances for the week, times the performances per day, rounded to the nearest dollar",
                "dollars(weekly_withholding / performances_per_week x performances_per_day)",
                &[&weekly, &week, &day],
                daily,
            ),
        ];
        let rates = vec![rate("table", table), rate("a", &a), rate("b", &b)];
        Ok((steps, rates))
    })
}

/// Schedule 3, 'Claiming tax offsets': the yearly amount divided by 52 and by
/// the week's performances, rounded to the dollar, taken off the amount.
fn entertainer_offsets(
    mut steps: Vec<Step>,
    offsets: Option<Decimal>,
    week: i64,
) -> Result<Vec<Step>> {
    let before = steps.last().expect("a daily step").value.clone();
    let Some(offsets) = offsets.filter(|o| !o.is_zero()) else {
        steps.push(step(
            "withholding",
            "Schedule 3, 'Using a formula': no tax offsets claimed",
            "daily_withholding",
            &[&before],
            before.clone(),
        ));
        return Ok(steps);
    };
    let week = int(week);
    let reduction = precise(|| {
        Ok(offsets
            .div(&d("52"))?
            .div(&week)?
            .quantize(0, Rounding::HalfUp)?)
    })?;
    if before.lt(&reduction)? {
        return refuse(
            "entertainer_withholding.offsets_exceed_withholding",
            "The per performance value of the tax offsets exceeds the amount withheld before them; Schedule 3 does not say how to treat the excess.",
        );
    }
    let net = precise(|| Ok(before.sub(&reduction)?))?;
    steps.push(step(
        "tax_offset_reduction",
        "Schedule 3, 'Claiming tax offsets': the amount claimed divided by 52 and by the performances per week, rounded to the nearest dollar",
        "dollars(tax_offsets / 52 / performances_per_week)",
        &[&offsets, &week],
        reduction.clone(),
    ));
    steps.push(step(
        "withholding",
        "Schedule 3, 'Claiming tax offsets': the amount less the per performance value of the tax offsets",
        "daily_withholding - tax_offset_reduction",
        &[&before, &reduction],
        net,
    ));
    Ok(steps)
}

/// Schedule 3 without a TFN: 47%, or 45% for a foreign resident, of the day's
/// earnings with cents ignored, cents in the result ignored.
fn entertainer_no_tfn(earnings: &Decimal, day: i64, foreign: bool) -> Result<Vec<Step>> {
    let day = int(day);
    let paid = precise(|| Ok(earnings.mul(&day)?))?;
    let mut steps = vec![step(
        "day_earnings",
        "Schedule 3, 'When a TFN has not been provided': the payment for the day's performances",
        "earnings x performances_per_day",
        &[earnings, &day],
        paid.clone(),
    )];
    steps.extend(no_tfn_steps(
        ("day_earnings", &paid),
        foreign,
        "Schedule 3, 'When a TFN has not been provided': the payment, ignoring any cents",
    )?);
    Ok(steps)
}

/// Schedule 3: the amount to withhold from a payment for a day's
/// performances. `pay` is the performances in the week and in the day, and
/// the earnings for one performance.
#[allow(clippy::too_many_arguments)]
pub fn entertainer_withholding(
    pay: [&Arg; 3],
    tfn_quoted: &Arg,
    foreign_resident: &Arg,
    tax_free_threshold_claimed: &Arg,
    working_holiday_maker: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    tax_offsets: Option<&Arg>,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, ENTERTAINER)?;
    let [per_week, per_day, earnings] = pay;
    let week = count(per_week, "performances_per_week", 1, 1000)?;
    let day = count(per_day, "performances_per_day", 1, week)?;
    let earnings = money(earnings)?;
    let tfn = flag(tfn_quoted, "tfn_quoted")?;
    let foreign = flag(foreign_resident, "foreign_resident")?;
    let threshold = flag(tax_free_threshold_claimed, "tax_free_threshold_claimed")?;
    not_working_holiday_maker(working_holiday_maker, "Schedule 3")?;
    let offsets = tax_offsets.map(money).transpose()?;
    let claimed = offsets.as_ref().is_some_and(|o| !o.is_zero());
    entertainer_eligible(tfn, foreign, threshold, claimed, day)?;
    if !tfn {
        let steps = entertainer_no_tfn(&earnings, day, foreign)?;
        return derived_result(ENTERTAINER, &year, steps, no_tfn_rate(foreign));
    }
    let (steps, rates) = entertainer_steps(&year, &earnings, (week, day), threshold)?;
    let steps = entertainer_offsets(steps, offsets, week)?;
    derived_result(ENTERTAINER, &year, steps, rates)
}

const RETURN_TO_WORK: &str = "return_to_work_withholding";

/// Schedule 4: 32% of the payment, for residents and foreign residents alike,
/// rounded to the dollar; 47% or 45% with cents ignored when no TFN is quoted.
pub fn return_to_work_withholding(
    payment: &Arg,
    tfn_quoted: &Arg,
    foreign_resident: &Arg,
    working_holiday_maker: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, RETURN_TO_WORK)?;
    let payment = money(payment)?;
    let tfn = flag(tfn_quoted, "tfn_quoted")?;
    let foreign = flag(foreign_resident, "foreign_resident")?;
    not_working_holiday_maker(working_holiday_maker, "Schedule 4")?;
    if !tfn {
        let steps = no_tfn_steps(
            ("payment", &payment),
            foreign,
            "Schedule 4, 'When a TFN has not been provided': the payment, ignoring any cents",
        );
        let steps = steps?;
        return derived_result(RETURN_TO_WORK, &year, steps, no_tfn_rate(foreign));
    }
    let a = d("0.32");
    let withheld = precise(|| Ok(payment.mul(&a)?.quantize(0, Rounding::HalfUp)?))?;
    let steps = vec![step(
        "withholding",
        "Schedule 4, 'Working out the withholding amount' and 'Rounding of withholding amounts': 32% of the payment, to the nearest dollar, 50 cents up",
        "dollars(rate x payment)",
        &[&a, &payment],
        withheld,
    )];
    derived_result(RETURN_TO_WORK, &year, steps, vec![rate("rate", "0.32")])
}

const ANNUITY: &str = "annuity_withholding";

/// Schedule 6, step 1: the annuity payment less the deductible amount divided
/// by the instalments in the income year, which must come to whole cents and
/// not below nil.
fn annuity_income(payment: &Decimal, deductible: &Decimal, instalments: i64) -> Result<Step> {
    let instalments = int(instalments);
    let income = precise(|| Ok(payment.sub(&deductible.div(&instalments)?)?))?;
    if income.lt(&d("0"))? {
        return refuse(
            "annuity_withholding.deductible_above_payment",
            "The deductible amount for each instalment exceeds the annuity payment; Schedule 6 does not say how to treat the excess.",
        );
    }
    if precise(|| Ok(income.quantize(-2, Rounding::Down)?.lt(&income)?))? {
        return refuse(
            "annuity_withholding.income_not_whole_cents",
            "The deductible amount divided by the instalments does not come to whole cents, and Schedule 6 gives no rounding for the amount of income.",
        );
    }
    Ok(step(
        "earnings",
        "Schedule 6, 'Working out the withholding amount', step 1: the annuity payment less the deductible amount divided by the instalments in the income year",
        "payment - deductible_amount / instalments",
        &[payment, deductible, &instalments],
        income,
    ))
}

/// Schedule 6: Schedule 1, as payg-withholding works it out, on the amount of
/// income in each annuity payment. Without a TFN (scale 4), 47%, or 45% for a
/// foreign resident, of the annuity payment with cents ignored.
#[allow(clippy::too_many_arguments)]
pub fn annuity_withholding(
    payment: &Arg,
    deductible_amount: &Arg,
    instalments: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    options: &PaygOptions,
) -> Result<Value> {
    let (year, payment, period, scale) =
        payg_checks(ANNUITY, payment, pay_period, scale, year, scope_confirmed)?;
    let deductible = money(deductible_amount)?;
    let instalments = count(instalments, "instalments", 1, 366)?;
    let choices = payg_options(options, scale, &period)?;
    if scale == 4 {
        if choices.extra {
            return refuse(
                "annuity_withholding.extra_pay_without_tfn",
                "Schedule 6's rule for a payee without a TFN takes 47% or 45% of the payment and says nothing of the additional amount for 53 weekly or 27 fortnightly pays.",
            );
        }
        let foreign = choices.foreign == Some(true);
        let steps = no_tfn_steps(
            ("payment", &payment),
            foreign,
            "Schedule 6, 'When a TFN has not been provided': the annuity payment, ignoring any cents",
        );
        let steps = steps?;
        return derived_result(ANNUITY, &year, steps, no_tfn_rate(foreign));
    }
    let income = annuity_income(&payment, &deductible, instalments)?;
    let rules = *lookup(meta::PAYG_WITHHOLDING_RULES, &year);
    let pay = SchedulePay {
        choices,
        rules,
        year,
        earnings: income.value.clone(),
        period,
        scale,
    };
    let mut w = precise(|| pay.workings())?;
    w.steps.insert(0, income);
    let label = format!("{} {}", pay.year, pay.period);
    derived_result(ANNUITY, &label, w.steps, w.rates)
}

const VOLUNTARY: &str = "voluntary_agreement_withholding";

/// TAA 1953 Sch 1 s 12-55(1)(c) and (d): the voluntary agreement is in force
/// and quotes the payee's ABN, which is in force.
fn agreement_applies(in_force: &Arg, abn_quoted: &Arg) -> Result<()> {
    if !flag(in_force, "agreement_in_force")? {
        return refuse(
            "voluntary_agreement_withholding.agreement_not_in_force",
            "Schedule 10 applies only under a voluntary agreement in force (TAA 1953 Sch 1 s 12-55(1)(c)); one terminated under s 12-55(3) or never made in the approved form requires no withholding under s 12-55.",
        );
    }
    if !flag(abn_quoted, "abn_quoted")? {
        return refuse(
            "voluntary_agreement_withholding.abn_not_quoted",
            "Schedule 10 applies only when the payee has an ABN in force and it is quoted in the voluntary agreement (TAA 1953 Sch 1 s 12-55(1)(d)).",
        );
    }
    Ok(())
}

/// Schedule 10's rate: the Commissioner's instalment rate when it is 20% or
/// more, or when the agreement says it applies; otherwise 20%.
fn voluntary_rate(instalment_rate: Option<&Arg>, cir_applies: bool) -> Result<Decimal> {
    let Some(value) = instalment_rate else {
        if cir_applies {
            return refuse(
                "voluntary_agreement_withholding.instalment_rate_missing",
                "The agreement says the payee's Commissioner's instalment rate applies, but no instalment_rate was given.",
            );
        }
        return Ok(d("0.20"));
    };
    let Some(percent) = precise(|| Ok(finite_between(value, "0", "100")?))? else {
        return refuse(
            "input.percentage_invalid",
            "instalment_rate must be a finite Decimal percentage from 0 to 100.",
        );
    };
    let cir = precise(|| Ok(percent.div(&d("100"))?))?;
    if cir_applies || !cir.lt(&d("0.20"))? {
        Ok(cir)
    } else {
        Ok(d("0.20"))
    }
}

/// Schedule 10: the payment without its GST times the rate, rounded to the
/// dollar.
#[allow(clippy::too_many_arguments)]
pub fn voluntary_agreement_withholding(
    payment: &Arg,
    gst_included: &Arg,
    agreement_in_force: &Arg,
    abn_quoted: &Arg,
    agreement_applies_instalment_rate: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    instalment_rate: Option<&Arg>,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, VOLUNTARY)?;
    let payment = money(payment)?;
    let gst = money(gst_included)?;
    agreement_applies(agreement_in_force, abn_quoted)?;
    let cir_applies = flag(
        agreement_applies_instalment_rate,
        "agreement_applies_instalment_rate",
    )?;
    let a = voluntary_rate(instalment_rate, cir_applies)?;
    if payment.lt(&gst)? {
        return refuse(
            "voluntary_agreement_withholding.gst_above_payment",
            "The GST included is more than the payment.",
        );
    }
    let (net, withheld) = precise(|| {
        let net = payment.sub(&gst)?;
        let withheld = net.mul(&a)?.quantize(0, Rounding::HalfUp)?;
        Ok((net, withheld))
    })?;
    let steps = vec![
        step(
            "payment_excluding_gst",
            "Schedule 10, 'If the payment includes an amount for GST': the GST is excluded",
            "payment - gst_included",
            &[&payment, &gst],
            net.clone(),
        ),
        step(
            "withholding",
            "Schedule 10, 'If the payee has informed you of their CIR' and 'Rounding of withholding amounts': the rate times the payment, to the nearest dollar, 50 cents up",
            "dollars(rate x payment_excluding_gst)",
            &[&a, &net],
            withheld,
        ),
    ];
    derived_result(VOLUNTARY, &year, steps, vec![rate("rate", &a)])
}
