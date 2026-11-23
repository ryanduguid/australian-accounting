// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Schedule 13, Part A: withholding from one superannuation income stream
//! payment, from Schedule 13 of the Taxation Administration (Withholding
//! Schedules) Instruments 2024 (F2024L00664), 2025 (F2025L00647 and
//! F2025L01016) and 2026 (F2026L00716) and ITAA 1997 Divisions 301 and 302,
//! all read on 23 November 2026.

use serde_json::{Value, json};

use super::{Components, SuperBenefitTaxArgs, age_on, components, conflict, when};
use crate::decimal::{Decimal, Rounding};
use crate::engine::{
    Arg, PaygChoices, Result, SchedulePay, Step, d, date_arg, derived_result, flag, int, lookup,
    precise, rate, refuse, step,
};
use crate::metadata as meta;

const KIND: &str = "super_withholding";

/// The inputs only an income stream payment takes, in this order.
pub const STREAM_OPTIONAL: [&str; 4] = [
    "income_stream_period",
    "tax_free_threshold_claimed",
    "capped_defined_benefit",
    "deceased_aged_60_or_over",
];

const PERIODS: [&str; 4] = ["weekly", "fortnightly", "monthly", "quarterly"];
const FIRST_YEAR: &str = "2024-25";
const LUMP_SUM_ONLY: [usize; 5] = [1, 3, 4, 5, 6];
const WITHHELD_FROM: &str = "Schedule 13, 'Rounding of withholding amounts' and Part A step 1: cents in the income stream are ignored; withholding applies to the taxable component under 60 and to the element untaxed in the fund at 60 or over";
const SCHEDULE_1: &str = "Schedule 13, Part A step 2: the PAYG withholding tax table for the payment cycle (Schedule 1, scale 2 with the tax-free threshold claimed, scale 1 without) on the step 1 amount";
const OFFSET: &str = "Schedule 13, Part A step 3: the tax offset per payment for the payee's age, the deceased's age and a disability super income stream (ITAA 1997 ss 301-40(2), 301-100(2), 302-75(2) and 302-85(2))";
const NO_OFFSET: &str = "Schedule 13, Part A step 3: no tax offset for a payee below preservation age whose income stream is not a disability super income stream";
const WEEKLY: &str = "Schedule 13, Part A step 4: the weekly equivalent of the step 1 amount for a fortnightly, monthly or quarterly payment";
const ADJUSTMENT: &str = "Schedule 13, Part A step 4: the offset adjustment on the weekly equivalent, with Schedule 1's Medicare levy threshold and shade-in point for singles, converted to the payment's period";
const NO_ADJUSTMENT: &str =
    "Schedule 13, Part A step 4: no offset adjustment when the tax offset is nil";
const RESULT: &str = "Schedule 13, Part A step 5 and 'Rounding of withholding amounts': the greater of the withholding amount less the tax offset and the offset adjustment, to the nearest dollar, 50 cents up, with no preliminary rounding to the cent";
const NO_TFN: &str = "Schedule 13, 'When a TFN has not been provided': 47% for a resident or 45% for a foreign resident of the step 1 amount, ignoring cents, with no tax offsets or Medicare levy adjustments";

/// A result's steps after the components, and its rates.
type Tail = (Vec<Step>, Vec<(String, String)>);

/// Schedule 13 Part A's offset row for the payee.
#[derive(Clone, Copy)]
enum Row {
    MemberUnder60,
    DisabilityUnder60,
    MemberAged60,
    DependantUnder60,
    DependantAged60,
}

impl Row {
    fn name(self) -> &'static str {
        match self {
            Self::MemberUnder60 => "income_stream_member_under_60",
            Self::DisabilityUnder60 => "income_stream_disability_under_60",
            Self::MemberAged60 => "income_stream_member_aged_60_or_over",
            Self::DependantUnder60 => "income_stream_dependant_under_60",
            Self::DependantAged60 => "income_stream_dependant_aged_60_or_over",
        }
    }

    fn aged_60(self) -> bool {
        matches!(self, Self::MemberAged60 | Self::DependantAged60)
    }

    /// The element the offset is on and its rate, or none.
    fn offset(self, c: &Components) -> Option<(&'static str, Decimal, &'static str)> {
        match self {
            Self::MemberUnder60 => None,
            Self::DisabilityUnder60 | Self::DependantUnder60 => {
                Some(("taxed_element", c.taxed.value.clone(), "0.15"))
            }
            Self::MemberAged60 | Self::DependantAged60 => {
                Some(("untaxed_element", c.untaxed.value.clone(), "0.10"))
            }
        }
    }
}

/// The payment cycle, refused when Schedule 1 has no formula for it.
fn period(value: &Arg) -> Result<String> {
    match value.text() {
        Some(p) if PERIODS.contains(&p) => Ok(p.to_string()),
        _ => refuse(
            "super_withholding.income_stream_period_invalid",
            "income_stream_period must be weekly, fortnightly, monthly or quarterly; Schedule 13 step 2 uses the PAYG withholding tax table for the payment cycle, and Schedule 1 has no formula for an annual payment.",
        ),
    }
}

/// The payee's age on the payment date and the Part A row, refusing the
/// cases Part A does not settle.
fn row(b: &SuperBenefitTaxArgs, stream: &[Option<&Arg>; 4], paid: i64) -> Result<(i64, Row)> {
    let member = match b.recipient.text() {
        Some("member") => true,
        Some("death_benefits_dependant") => false,
        _ => {
            return refuse(
                "super_withholding.recipient_invalid",
                "An income stream payment's recipient must be member or death_benefits_dependant: Schedule 13 Part A's offsets cover those two.",
            );
        }
    };
    let born = when(true, b.optional[0], "date_of_birth", "for an income stream")?;
    let born = date_arg(born.expect("required"), "date_of_birth")?;
    if born >= paid {
        return conflict("date_of_birth must be before payment_date.");
    }
    let age = age_on(born, paid);
    let why = "for an income stream (income_stream_period given)";
    let capped = when(true, stream[2], "capped_defined_benefit", why)?.expect("required");
    if flag(capped, "capped_defined_benefit")? && age >= 60 {
        return refuse(
            "super_withholding.capped_defined_benefit",
            "Schedule 13 Parts B to E, not Part A, apply to a capped defined benefit income stream paid to a payee 60 or over, with the defined benefit income cap; this command does not take them.",
        );
    }
    let row = if member {
        let why = "for a death benefit income stream (recipient death_benefits_dependant)";
        when(false, stream[3], "deceased_aged_60_or_over", why)?;
        member_row(b, age)?
    } else {
        let why = "for a member's income stream (recipient member)";
        when(
            false,
            b.optional[2],
            "disability_superannuation_benefit",
            why,
        )?;
        dependant_row(stream, age)?
    };
    Ok((age, row))
}

fn member_row(b: &SuperBenefitTaxArgs, age: i64) -> Result<Row> {
    let why = "for a member (recipient member)";
    let disability = when(
        true,
        b.optional[2],
        "disability_superannuation_benefit",
        why,
    )?;
    let disability = flag(
        disability.expect("required"),
        "disability_superannuation_benefit",
    )?;
    Ok(if age >= 60 {
        Row::MemberAged60
    } else if disability {
        Row::DisabilityUnder60
    } else {
        Row::MemberUnder60
    })
}

fn dependant_row(stream: &[Option<&Arg>; 4], age: i64) -> Result<Row> {
    let why = "for a death benefit income stream (recipient death_benefits_dependant)";
    let deceased = when(true, stream[3], "deceased_aged_60_or_over", why)?.expect("required");
    let deceased_60 = flag(deceased, "deceased_aged_60_or_over")?;
    if age >= 60 {
        return Ok(Row::DependantAged60);
    }
    if deceased_60 {
        return refuse(
            "super_withholding.deceased_60_recipient_under_60",
            "Schedule 13 Part A withholds from the element taxed in the fund of a payee under 60 with an offset only on the element untaxed in the fund where the deceased died aged 60 or over, whilst ITAA 1997 s 302-65(b) makes that stream, apart from the element untaxed in the fund, not assessable and not exempt; for a capped defined benefit stream Part A sends this case to Part E. The command does not choose between them.",
        );
    }
    Ok(Row::DependantUnder60)
}

/// Step 1: the whole dollars of the amount withholding applies to.
fn withheld_from(r: Row, c: &Components) -> Result<Step> {
    let (name, base) = if r.aged_60() {
        ("untaxed_element", &c.untaxed.value)
    } else {
        ("taxable_component", &c.taxable.value)
    };
    let used = base.quantize(0, Rounding::Down)?;
    let formula = format!("floor({name})");
    Ok(step(
        "withheld_from",
        WITHHELD_FROM,
        &formula,
        &[base],
        used,
    ))
}

/// Without a TFN: 47% or 45% of the step 1 amount, cents ignored.
fn no_tfn(from: Step, foreign: bool) -> Result<Tail> {
    let r = d(if foreign { "0.45" } else { "0.47" });
    let withheld = from.value.mul(&r)?.quantize(0, Rounding::Down)?;
    let formula = "floor(withheld_from x no_tfn_rate)";
    let last = step("withholding", NO_TFN, formula, &[&from.value, &r], withheld);
    Ok((vec![from, last], vec![rate("no_tfn_rate", &r)]))
}

/// Step 2: Schedule 1 on the step 1 amount, its last step renamed.
fn schedule_1(year: &str, amount: &Decimal, period: &str, scale: i64) -> Result<Tail> {
    let rules = *lookup(meta::PAYG_WITHHOLDING_RULES, year);
    let pay = SchedulePay {
        choices: PaygChoices {
            foreign: None,
            offsets: None,
            children: None,
            extra: false,
        },
        rules,
        year: year.to_string(),
        earnings: amount.clone(),
        period: period.to_string(),
        scale,
    };
    let mut w = precise(|| pay.workings())?;
    // Schedule 1 names the pay's earnings; here they are the step 1 amount.
    let first = w.steps.iter_mut().find(|s| s.id == "weekly_earnings_used");
    let first = first.expect("weekly earnings");
    first.formula = first.formula.replace("earnings", "withheld_from");
    let last = w.steps.last_mut().expect("a withholding step");
    last.id = "withholding_amount".into();
    last.authority = SCHEDULE_1;
    Ok((w.steps, w.rates))
}

/// Step 3: the tax offset per payment on the whole dollars of its element.
fn offset(r: Row, c: &Components, rates: &mut Vec<(String, String)>) -> Result<Step> {
    let Some((name, element, share)) = r.offset(c) else {
        return Ok(step("tax_offset", NO_OFFSET, "0", &[], int(0)));
    };
    let used = element.quantize(0, Rounding::Down)?;
    let share = d(share);
    rates.push(rate("tax_offset_rate", &share));
    let formula = format!("floor({name}) x tax_offset_rate");
    let value = used.mul(&share)?;
    Ok(step(
        "tax_offset",
        OFFSET,
        &formula,
        &[&element, &share],
        value,
    ))
}

/// The weekly equivalent of a period's amount, its formula, and the
/// multiplier and divisor that convert a weekly amount back to the period.
fn weekly(amount: &Decimal, period: &str) -> Result<(Decimal, &'static str, i64, i64)> {
    Ok(match period {
        "weekly" => (amount.clone(), "withheld_from", 1, 1),
        "fortnightly" => (amount.div(&int(2))?, "withheld_from / 2", 2, 1),
        "quarterly" => (amount.div(&int(13))?, "withheld_from / 13", 13, 1),
        _ => (
            amount.mul(&int(3))?.div(&int(13))?,
            "withheld_from x 3 / 13",
            13,
            3,
        ),
    })
}

/// The year's singles threshold and shade-in point (Schedule 1, scale 2) and
/// the upper limit of the 2% band.
fn levy_points(year: &str) -> (i64, i64, i64) {
    let rules = *lookup(meta::PAYG_WITHHOLDING_RULES, year);
    let (_, m) = rules
        .medicare_scales
        .iter()
        .find(|(s, _)| *s == 2)
        .expect("scale 2");
    let limit = *lookup(meta::SCHEDULE_13_ADJUSTMENT_LIMIT, year);
    (m.threshold, m.shade_in, limit)
}

/// Step 4: the offset adjustment, nil when the offset is nil.
fn adjustment(
    year: &str,
    period: &str,
    from: &Decimal,
    rates: &mut Vec<(String, String)>,
) -> Result<Vec<Step>> {
    let (w, formula, times, per) = weekly(from, period)?;
    let (threshold, shade_in, limit) = levy_points(year);
    rates.push(rate("medicare_levy_threshold_weekly", threshold));
    rates.push(rate("medicare_levy_shade_in_weekly", shade_in));
    rates.push(rate("adjustment_limit_weekly", limit));
    let t = int(threshold);
    // The weekly band times `times / per`, rearranged so the only division
    // is the last one by `per` (1 or 3): an exact half dollar stays exact.
    let (formula_out, operands, value) = if w.lt(&t)? || !w.lt(&int(limit))? {
        ("0".to_string(), vec![], int(0))
    } else if w.lt(&int(shade_in))? {
        let v = from
            .mul(&int(per))?
            .sub(&t.mul(&int(times))?)?
            .mul(&d("0.10"))?
            .div(&int(per))?;
        let band = format!(
            "(withheld_from x {per} - medicare_levy_threshold_weekly x {times}) x 0.10 / {per}"
        );
        (band, vec![from, &t], v)
    } else {
        let band = "withheld_from x 0.02".to_string();
        (band, vec![from], from.mul(&d("0.02"))?)
    };
    Ok(vec![
        step("weekly_equivalent", WEEKLY, formula, &[from], w.clone()),
        step(
            "offset_adjustment",
            ADJUSTMENT,
            &formula_out,
            &operands,
            value,
        ),
    ])
}

/// Steps 2 to 5 with a TFN.
fn with_tfn(
    year: &str,
    period: &str,
    r: Row,
    c: &Components,
    (from, scale): (Step, i64),
) -> Result<Tail> {
    let (mut steps, mut rates) = schedule_1(year, &from.value, period, scale)?;
    let amount = steps.last().expect("step 2").value.clone();
    let off = offset(r, c, &mut rates)?;
    let adjust = if off.value.is_zero() {
        vec![step("offset_adjustment", NO_ADJUSTMENT, "0", &[], int(0))]
    } else {
        adjustment(year, period, &from.value, &mut rates)?
    };
    let adjusted = adjust.last().expect("step 4").value.clone();
    let notional = amount.sub(&off.value)?;
    let greater = if notional.gt(&adjusted)? {
        notional
    } else {
        adjusted.clone()
    };
    let withheld = greater.quantize(0, Rounding::HalfUp)?;
    let formula = "dollars(max(withholding_amount - tax_offset, offset_adjustment))";
    let last = step(
        "withholding",
        RESULT,
        formula,
        &[&amount, &off.value, &adjusted],
        withheld,
    );
    let mut all = vec![from];
    all.append(&mut steps);
    all.push(off);
    all.extend(adjust);
    all.push(last);
    Ok((all, rates))
}

/// The flags and threshold choice: TFN, foreign resident and the scale.
fn tfn_and_scale(tfn: bool, foreign: bool, threshold: Option<&Arg>) -> Result<Option<i64>> {
    if tfn && foreign {
        return refuse(
            "super_withholding.income_stream_foreign_resident",
            "Schedule 13 says a foreign resident's income stream assessable in Australia is withheld from but names no scale for one who gave a TFN, and its step 4 adjustment is for the Medicare levy, which a foreign resident does not pay.",
        );
    }
    let claimed = when(
        tfn,
        threshold,
        "tax_free_threshold_claimed",
        "with a TFN (tfn_provided true)",
    )?;
    match claimed {
        Some(value) => Ok(Some(if flag(value, "tax_free_threshold_claimed")? {
            2
        } else {
            1
        })),
        None => Ok(None),
    }
}

/// Schedule 13 Part A for one income stream payment.
pub fn income_stream(
    b: &SuperBenefitTaxArgs,
    stream: &[Option<&Arg>; 4],
    (year, paid): (&str, i64),
    (tfn, foreign): (bool, bool),
) -> Result<Value> {
    if year < FIRST_YEAR {
        return refuse(
            "super_withholding.income_stream_year",
            "Schedule 13 before 1 July 2024 was not read for this command; an income stream payment needs year 2024-25 or later.",
        );
    }
    let period = period(stream[0].expect("selected by the period"))?;
    for i in LUMP_SUM_ONLY {
        let key = super::SUPER_BENEFIT_OPTIONAL[i];
        when(false, b.optional[i], key, "for a superannuation lump sum")?;
    }
    let scale = tfn_and_scale(tfn, foreign, stream[1])?;
    let (age, r) = row(b, stream, paid)?;
    let c = components(b, true)?;
    let from = withheld_from(r, &c)?;
    let (tail, mut rates) = match scale {
        None => no_tfn(from, foreign)?,
        Some(scale) => precise(|| with_tfn(year, &period, r, &c, (from, scale)))?,
    };
    rates.push(rate("age", age));
    let mut steps = vec![c.tax_free, c.taxable, c.taxed, c.untaxed];
    steps.extend(tail);
    let mut out = derived_result(KIND, year, steps, rates)?;
    out["treatment"] = json!(r.name());
    out["income_stream_period"] = json!(period);
    Ok(out)
}
