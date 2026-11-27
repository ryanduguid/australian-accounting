// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The interest the Commissioner pays on early payments and overpayments:
//! simple interest at the base interest rate.

use serde_json::{Value, json};

use super::{
    Facts, KIND, Stretch, add_day, date_order, dated_amounts, dollars, in_year, rates,
    stretches_json, unpublished, warn, year_length,
};
use crate::dates;
use crate::decimal::Decimal;
use crate::engine::{Result, refuse, result};

const CREDIT_RATE: &str = "Taxation (Interest on Overpayments and Early Payments) Act 1983 ss 8C and 10(1)(b): the base interest rate (TAA 1953 s 8AAD(2)), which is the published GIC rate less 7 points (s 8AAD(1)); the ATO's 'Credit interest rates and calculation' (QC58284, last updated 19 June 2026) publishes the same rates from 1 July 2023 to 30 September 2026";
const CREDIT_SUM: &str = "The ATO's 'Credit interest rates and calculation': (A / B) x C x (D / 100), simple interest (PS LA 2011/23 para 4), applied day by day so each day takes its quarter's rate and its year's length; rounded to the nearest cent, half a cent up (s 12D); under 50 cents is not payable (s 12E)";
const EARLY: &str = "Overpayments Act s 8A(1): interest on a payment of income tax, SIC, a compulsory study loan repayment amount or an FS assessment debt made more than 14 days before the day it is due and payable; s 8B(1): from the start of the later of the payment day and the notice day (the payment day for a full self-assessment taxpayer) to the end of the due day";
const OVERPAID: &str = "Overpayments Act s 9(1): interest on relevant tax paid and overpaid as a result of a decision to which the Act applies, then refunded or applied; instalments are separate amounts and a partial refund is attributed to them in reverse order of payment (s 9(2) and (3)); s 10(1)(a): from the later of the day the notice of the assessment, determination or decision was issued and the day the amount was paid, to the day it was refunded or applied";

/// Every day's denominator: 10,000 (hundredths of 1% as a fraction) x 365 x 366.
const SIMPLE_DENOMINATOR: u128 = 10_000 * 365 * 366;

/// Simple interest on `amount` cents from the start of `first` to the end of
/// `last` at the base interest rate, as a numerator over `SIMPLE_DENOMINATOR`
/// in cents, and the stretches of days.
fn simple(amount: u64, first: i64, last: i64) -> Result<(u128, Vec<Stretch>)> {
    let (mut sum, mut stretches) = (0_u128, Vec::new());
    for day in first..=last {
        let Some(rate) = rates::base(day) else {
            return unpublished(format!(
                "No base interest rate is published for {}: the rates run from 1 July 1999 to {}.",
                dates::format(day),
                rates::TABLE_END
            ));
        };
        // rate / (10,000 x days), over 10,000 x 365 x 366.
        let other = if year_length(day) == 365 { 366 } else { 365 };
        sum += u128::from(rate) * other;
        add_day(&mut stretches, day, rate, false);
    }
    Ok((u128::from(amount) * sum, stretches))
}

fn simple_cents(numerator: u128) -> u128 {
    (2 * numerator + SIMPLE_DENOMINATOR) / (2 * SIMPLE_DENOMINATOR)
}

/// The interest and the amount payable (s 12E: nothing under 50 cents).
fn credit_amounts(interest: u128) -> Vec<(&'static str, Decimal)> {
    let payable = if interest >= 50 { interest } else { 0 };
    vec![
        ("interest", dollars(interest)),
        ("interest_payable", dollars(payable)),
    ]
}

fn credit_authorities(out: &mut Value, entitlement: &str) {
    out["entitlement_authority"] = json!(entitlement);
    out["rate_authority"] = json!(CREDIT_RATE);
    out["interest_authority"] = json!(CREDIT_SUM);
}

/// The liabilities s 8A(1)(a) lists that the command takes.
const LIABILITIES: [&str; 4] = [
    "income_tax",
    "shortfall_interest_charge",
    "compulsory_study_loan_repayment",
    "fs_assessment_debt",
];

fn listed_liability(f: &Facts) -> Result<()> {
    let liability = f.value("liability")?;
    if LIABILITIES.iter().any(|l| liability.as_str() == Some(*l)) {
        return Ok(());
    }
    refuse(
        "ato_interest_penalties.liability_not_listed",
        format!(
            "{}: liability must be one Overpayments Act s 8A(1)(a) lists: {}. Other amounts, such as GST or PAYG instalments, earn no early payment interest under s 8A.",
            f.item(),
            LIABILITIES.join(", ")
        ),
    )
}

/// The payment's checked facts: amount, payment day, due day and the first
/// day interest could run from (s 8B(1)).
fn early_facts(f: &Facts, period: &str) -> Result<(u64, i64, i64, i64)> {
    let fsa = f.flag("full_self_assessment")?;
    let mut takes = vec![
        "amount",
        "liability",
        "paid_on",
        "due_date",
        "full_self_assessment",
    ];
    if !fsa {
        takes.push("notice_issued");
    }
    f.only(&takes)?;
    listed_liability(f)?;
    let amount = f.cents("amount")?;
    let (paid, due) = (f.date("paid_on")?, f.date("due_date")?);
    in_year(due, period, "due_date")?;
    if paid > due {
        return date_order(format!(
            "{}: paid_on is after due_date, so the payment is not early (s 8A(1)(b)).",
            f.item()
        ));
    }
    let start = if fsa {
        paid
    } else {
        paid.max(f.date("notice_issued")?)
    };
    Ok((amount, paid, due, start))
}

pub(super) fn early_payment_interest(f: &Facts, period: &str) -> Result<Value> {
    let (amount, paid, due, start) = early_facts(f, period)?;
    let early = due - paid;
    let earns = early > 14 && start <= due;
    let (numerator, stretches) = if earns {
        simple(amount, start, due)?
    } else {
        (0, Vec::new())
    };
    let out = result(
        KIND,
        period,
        credit_amounts(simple_cents(numerator)),
        Vec::new(),
    );
    let mut out = out?;
    out["question"] = json!("early_payment_interest");
    out["days_early"] = json!(early);
    out["entitled"] = json!(early > 14);
    out["days"] = json!(if earns { due - start + 1 } else { 0 });
    if earns {
        out["first_day"] = json!(dates::format(start));
        out["last_day"] = json!(dates::format(due));
    }
    out["stretches"] = stretches_json(&stretches);
    credit_authorities(&mut out, EARLY);
    if early <= 14 {
        warn(
            &mut out,
            format!(
                "The payment was {early} days before the due day, not more than 14 (s 8A(1)(b)): no interest."
            ),
        );
    }
    Ok(out)
}

fn part_iii_applies(f: &Facts) -> Result<()> {
    if f.flag("relevant_tax")? && f.flag("decision_to_which_act_applies")? {
        return Ok(());
    }
    refuse(
        "ato_interest_penalties.not_entitled",
        format!(
            "{}: Part III interest is payable only on relevant tax (s 3C) overpaid as a result of a decision to which the Act applies (s 3(1)); this command does not work other credit interest.",
            f.item()
        ),
    )
}

/// s 9(3): the refund goes to the payments in reverse order of payment, each
/// part earning interest from the later of the notice and its payment day.
fn attribute(
    f: &Facts,
    mut paid: Vec<(i64, u64)>,
    refunded: u64,
    notice: i64,
    refunded_on: i64,
) -> Result<(u128, Vec<Value>)> {
    paid.sort_by_key(|(day, _)| *day);
    let (mut left, mut total, mut parts) = (refunded, 0_u128, Vec::new());
    for (day, cents) in paid.iter().rev() {
        if left == 0 {
            break;
        }
        let share = left.min(*cents);
        left -= share;
        let from = notice.max(*day);
        if from > refunded_on {
            return date_order(format!(
                "{}: refunded_on is before a payment it refunds or the notice day.",
                f.item()
            ));
        }
        let (numerator, stretches) = simple(share, from, refunded_on)?;
        total += numerator;
        parts.push(json!({
            "paid_on": dates::format(*day),
            "attributed": dollars(u128::from(share)).to_string(),
            "first_day": dates::format(from),
            "last_day": dates::format(refunded_on),
            "days": refunded_on - from + 1,
            "stretches": stretches_json(&stretches),
        }));
    }
    Ok((total, parts))
}

pub(super) fn overpayment_interest(f: &Facts, period: &str) -> Result<Value> {
    f.only(&[
        "payments",
        "refunded",
        "refunded_on",
        "notice_issued",
        "relevant_tax",
        "decision_to_which_act_applies",
    ])?;
    part_iii_applies(f)?;
    let (refunded, refunded_on) = (f.cents("refunded")?, f.date("refunded_on")?);
    let notice = f.date("notice_issued")?;
    in_year(refunded_on, period, "refunded_on")?;
    let paid = dated_amounts(f, "paid_on")?;
    if refunded > paid.iter().map(|(_, c)| *c).sum::<u64>() {
        return refuse(
            "ato_interest_penalties.overpaid",
            format!("{}: refunded is more than the payments.", f.item()),
        );
    }
    let (total, parts) = attribute(f, paid, refunded, notice, refunded_on)?;
    let mut amounts = vec![("refunded", dollars(u128::from(refunded)))];
    amounts.extend(credit_amounts(simple_cents(total)));
    let mut out = result(KIND, period, amounts, Vec::new())?;
    out["question"] = json!("overpayment_interest");
    out["attribution"] = Value::Array(parts);
    credit_authorities(&mut out, OVERPAID);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ATO's Evan example ('Credit interest rates and calculation'):
    /// $1,926 for 19 days at 2% in a 365-day year is $2.01.
    #[test]
    fn the_credit_formula_gives_evans_interest() {
        let numerator = 192_600_u128 * 19 * 200 * 366;
        assert_eq!(simple_cents(numerator), 201);
    }
}
