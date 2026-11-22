// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst_creditable_purpose_change`: GST Act Division 129, whether a tax
//! period ending on 30 June is an adjustment period for an acquisition
//! (ss 129-10, 129-20 and 129-25) and the increasing or decreasing adjustment
//! for it (ss 129-40, 129-70 and 129-75), with GSTR 2000/24's examples.

use serde_json::{Value, json};

use super::super::{
    Arg, Result, Step, d, date_arg, derived_result, flag, int, money, precise, rate, refuse, scope,
    step,
};
use super::fraction;
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "gst_creditable_purpose_change";

/// The facts as the command passes them; see the README for each.
pub struct CreditablePurposeArgs<'a> {
    pub gst_exclusive_value: &'a Arg,
    pub relates_to_business_finance: &'a Arg,
    pub full_input_tax_credit: &'a Arg,
    pub intended_or_former_application: &'a Arg,
    pub actual_application: &'a Arg,
    pub application_basis: &'a Arg,
    pub attributed_period_end: &'a Arg,
    pub tax_period_months: &'a Arg,
    pub ended_on: Option<&'a Arg>,
    pub division_132_disposal: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The longest `application_basis` taken.
const MAX_BASIS: usize = 200;

const INCREASING: &str = "GST Act s 129-40(1) step 3 and s 129-70: the actual application is less than the intended or former application, so an increasing adjustment of the full input tax credit times the difference";
const DECREASING: &str = "GST Act s 129-40(1) step 4 and s 129-75: the actual application is greater than the intended or former application, so a decreasing adjustment of the full input tax credit times the difference";
const NOT_INCREASING: &str = "GST Act s 129-40(1) steps 3 to 5: the actual application is not less than the intended or former application";
const NOT_DECREASING: &str = "GST Act s 129-40(1) steps 3 to 5: the actual application is not greater than the intended or former application";
const BELOW_FINANCE: &str = "GST Act s 129-10(1): no adjustment for an acquisition that relates to business finance unless its GST exclusive value is more than $10,000";
const BELOW_OTHER: &str = "GST Act s 129-10(2): no adjustment for an acquisition that does not relate to business finance unless its GST exclusive value is more than $1,000";
const TOO_EARLY: &str = "GST Act s 129-20(1)(a): the tax period starts less than 12 months after the end of the tax period the acquisition is attributable to, so it is not an adjustment period";
const TOO_LATE: &str = "GST Act s 129-20(2) and (3): the tax period is after the last of the adjustment periods the GST exclusive value allows";
const AFTER_END: &str = "GST Act s 129-25(1): the thing was disposed of, lost, stolen or destroyed, or its period expired, and the next tax period ending on 30 June after that was the last adjustment period";
const AFTER_132: &str = "GST Act s 129-25(2): the thing was disposed of with a Division 132 decreasing adjustment, and the last adjustment period is the last to end before the disposal";

/// Whether the tested tax period is an adjustment period: its number and the
/// count, or why not.
struct Period {
    number: i64,
    count: i64,
    first_year: i64,
    not_one: Option<&'static str>,
}

/// The tax period length in months: 1, 3 or 12.
fn months(value: &Arg) -> Result<i64> {
    match value.int() {
        Some(Some(n)) if [1, 3, 12].contains(&n) => Ok(n),
        _ => refuse(
            "gst_creditable_purpose_change.tax_period_unsupported",
            "tax_period_months must be 1, 3 or 12: calendar months, quarters or a financial year, each with a tax period ending on 30 June.",
        ),
    }
}

/// The first day of the tax period of `months` months that ends on 30 June of
/// `end_year`.
fn period_start(end_year: i64, months: i64) -> i64 {
    let july = dates::parse(&format!("{end_year}-07-01")).expect("year");
    dates::add_months(july, -months)
}

/// The calendar year in which the financial year containing `day` ends.
fn year_ending(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    if m >= 7 { y + 1 } else { y }
}

/// The last day of a month, the end of the tax period the acquisition is
/// attributable to.
fn attributed_end(value: &Arg) -> Result<i64> {
    let day = date_arg(value, "attributed_period_end")?;
    if dates::to_civil(day + 1).2 != 1 {
        return refuse(
            "gst_creditable_purpose_change.period_end_invalid",
            "attributed_period_end must be the last day of a month, the end of the tax period the acquisition is attributable to.",
        );
    }
    Ok(day)
}

/// The number of adjustment periods s 129-20(2) or (3) allows.
fn period_count(value: &Decimal, finance: bool) -> Result<i64> {
    let (low, low_count) = if finance { ("50000", 1) } else { ("5000", 2) };
    Ok(if value.le(&d(low))? {
        low_count
    } else if value.lt(&d("500000"))? {
        5
    } else {
        10
    })
}

/// The last financial year (by its ending year) with an adjustment period
/// under s 129-25, if the thing was disposed of or ended.
fn last_year(args: &CreditablePurposeArgs, months: i64) -> Result<Option<(i64, &'static str)>> {
    let Some(ended) = args.ended_on else {
        if args.division_132_disposal.is_some() {
            return refuse(
                "input.missing_field",
                "ended_on is required with division_132_disposal.",
            );
        }
        return Ok(None);
    };
    let ended = date_arg(ended, "ended_on")?;
    let by_132 = match args.division_132_disposal {
        Some(value) => flag(value, "division_132_disposal")?,
        None => false,
    };
    let own = year_ending(ended);
    if by_132 {
        return Ok(Some((own - 1, AFTER_132)));
    }
    if ended >= period_start(own, months) {
        return refuse(
            "gst_creditable_purpose_change.last_period_unsettled",
            "ended_on falls in a tax period that itself ends on 30 June; whether s 129-25(1)'s 'next tax period' that ends on 30 June is that period or the following one is not settled here.",
        );
    }
    Ok(Some((own, AFTER_END)))
}

/// Whether `year`'s 30 June tax period is an adjustment period (s 129-20 and
/// s 129-25).
fn adjustment_period(
    args: &CreditablePurposeArgs,
    year: &str,
    value: &Decimal,
    finance: bool,
) -> Result<Period> {
    let months = months(args.tax_period_months)?;
    let attributed = attributed_end(args.attributed_period_end)?;
    let earliest = dates::add_months(attributed + 1, 12);
    let mut first_year = year_ending(earliest);
    if period_start(first_year, months) < earliest {
        first_year += 1;
    }
    let tested: i64 = year[..4].parse::<i64>().expect("year") + 1;
    let count = period_count(value, finance)?;
    let number = tested - first_year + 1;
    let last = last_year(args, months)?;
    if let Some((end, AFTER_END)) = last
        && end < first_year
        && end == tested
    {
        return refuse(
            "gst_creditable_purpose_change.last_period_unsettled",
            "The next tax period ending on 30 June after ended_on ends before the first adjustment period s 129-20(1) allows; whether s 129-25(1), which applies despite s 129-20, makes it an adjustment period is not settled here.",
        );
    }
    let not_one = match last {
        _ if number < 1 => Some(TOO_EARLY),
        Some((last, why)) if tested > last => Some(why),
        _ if number > count => Some(TOO_LATE),
        _ => None,
    };
    Ok(Period {
        number,
        count,
        first_year,
        not_one,
    })
}

/// The operator's basis for the 2 applications: echoed, never judged.
fn basis(value: &Arg) -> Result<String> {
    match value.text() {
        Some(text) if !text.trim().is_empty() && text.chars().count() <= MAX_BASIS => {
            Ok(text.to_string())
        }
        _ => refuse(
            "gst_creditable_purpose_change.basis_missing",
            format!(
                "application_basis must state, in 1 to {MAX_BASIS} characters, the method and records the 2 applications rest on; the command does not judge whether the method is fair or reasonable."
            ),
        ),
    }
}

/// Both adjustments nil, for `why`.
fn no_adjustment(why: &'static str) -> Vec<Step> {
    vec![
        step("increasing_adjustment", why, "0", &[], int(0)),
        step("decreasing_adjustment", why, "0", &[], int(0)),
    ]
}

/// The s 129-70 and s 129-75 amounts, one of them nil.
fn adjustments(full: &Decimal, former: &Decimal, actual: &Decimal) -> Result<Vec<Step>> {
    precise(|| {
        let up = "full_input_tax_credit x (intended_or_former_application - actual_application)";
        let down = "full_input_tax_credit x (actual_application - intended_or_former_application)";
        let operands = [full, former, actual];
        let down_operands = [full, actual, former];
        Ok(if actual.lt(former)? {
            let amount = full.mul(&former.sub(actual)?)?;
            vec![
                step("increasing_adjustment", INCREASING, up, &operands, amount),
                step("decreasing_adjustment", NOT_DECREASING, "0", &[], int(0)),
            ]
        } else if actual.gt(former)? {
            let amount = full.mul(&actual.sub(former)?)?;
            vec![
                step("increasing_adjustment", NOT_INCREASING, "0", &[], int(0)),
                step(
                    "decreasing_adjustment",
                    DECREASING,
                    down,
                    &down_operands,
                    amount,
                ),
            ]
        } else {
            vec![
                step("increasing_adjustment", NOT_INCREASING, "0", &[], int(0)),
                step("decreasing_adjustment", NOT_DECREASING, "0", &[], int(0)),
            ]
        })
    })
}

/// Why no adjustment can arise for the value alone (s 129-10), if so.
fn below_threshold(value: &Decimal, finance: bool) -> Result<Option<&'static str>> {
    let (limit, why) = if finance {
        ("10000", BELOW_FINANCE)
    } else {
        ("1000", BELOW_OTHER)
    };
    Ok((!value.gt(&d(limit))?).then_some(why))
}

/// GST Act Division 129: the adjustment, if any, for one acquisition in the
/// tax period ending on 30 June of `year`.
pub fn gst_creditable_purpose_change(args: &CreditablePurposeArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let value = money(args.gst_exclusive_value)?;
    let finance = flag(
        args.relates_to_business_finance,
        "relates_to_business_finance",
    )?;
    let full = money(args.full_input_tax_credit)?;
    let former = fraction(
        args.intended_or_former_application,
        "intended_or_former_application",
    )?;
    let actual = fraction(args.actual_application, "actual_application")?;
    let basis = basis(args.application_basis)?;
    let period = adjustment_period(args, &year, &value, finance)?;
    let why = below_threshold(&value, finance)?.or(period.not_one);
    let steps = match why {
        Some(why) => no_adjustment(why),
        None => adjustments(&full, &former, &actual)?,
    };
    let rates = vec![
        rate("value_threshold", if finance { "10000" } else { "1000" }),
        rate("adjustment_periods", period.count),
    ];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["adjustment_period"] = json!({
        "is_adjustment_period": period.not_one.is_none(),
        "number": (period.not_one.is_none()).then_some(period.number),
        "of": period.count,
        "first": format!("the tax period ending 30 June {}", period.first_year),
    });
    out["no_adjustment_reason"] = json!(why);
    out["application_basis"] = json!(basis);
    Ok(out)
}
