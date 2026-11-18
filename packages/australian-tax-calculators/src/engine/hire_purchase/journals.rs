// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Journal lines for the agreement: one entry at the start, one for each
//! period and one for a payout, each balanced in cents, then totalled by
//! income year with the periods spanning 30 June set apart, as the interest
//! by income year is. The hire purchase liability is carried at the notional
//! loan principal outstanding, so each period's interest is added to it and
//! each payment taken off it.

use serde_json::{Value, json};

use super::{Period, income_year};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{Result, int};

const ASSET: &str = "hire_purchase_asset";
const CREDIT: &str = "gst_input_tax_credit";
const LIABILITY: &str = "hire_purchase_liability";
const BANK: &str = "bank";
const TRADE_IN: &str = "trade_in_allowance";
const DEFERRED: &str = "deferred_interest_gst";
const INTEREST: &str = "interest_expense";
const DIFFERENCE: &str = "payout_difference";

/// The accounts in the order their lines are listed.
const ACCOUNTS: [&str; 8] = [
    ASSET, CREDIT, LIABILITY, INTEREST, DEFERRED, DIFFERENCE, TRADE_IN, BANK,
];

/// An entry's lines: (account, debit, credit) in cents.
type Lines = Vec<(&'static str, Decimal, Decimal)>;

/// Adds `amount` to `account`: a debit when it is positive, a credit when
/// negative, nothing when nil.
fn post(lines: &mut Lines, account: &'static str, amount: &Decimal) -> Result<()> {
    if amount.is_zero() {
        return Ok(());
    }
    if int(0).lt(amount)? {
        lines.push((account, amount.clone(), int(0)));
    } else {
        lines.push((account, int(0), amount.neg()?));
    }
    Ok(())
}

fn render(lines: &Lines) -> Value {
    let mut sorted: Vec<&(&str, Decimal, Decimal)> = lines.iter().collect();
    sorted.sort_by_key(|(account, _, _)| ACCOUNTS.iter().position(|a| a == account));
    Value::Array(
        sorted
            .into_iter()
            .map(|(account, debit, credit)| {
                json!({"account": account, "debit": debit.fixed(2, false), "credit": credit.fixed(2, false)})
            })
            .collect(),
    )
}

/// `lines` added into `totals`, one debit and one credit total an account.
fn add_into(totals: &mut Lines, lines: &Lines) -> Result<()> {
    for (account, debit, credit) in lines {
        match totals.iter_mut().find(|(a, _, _)| a == account) {
            Some(total) => {
                total.1 = total.1.add(debit)?;
                total.2 = total.2.add(credit)?;
            }
            None => totals.push((account, debit.clone(), credit.clone())),
        }
    }
    Ok(())
}

/// The input tax credit the journals book, in cents: the whole credit, its
/// share of the interest and the extent of creditable purpose that share is
/// worked at.
pub(super) struct Booked {
    pub credit: Decimal,
    pub on_interest: Decimal,
    pub purpose: Decimal,
}

/// What the journals are drawn from, every amount in cents.
pub(super) struct Facts<'a> {
    pub start: i64,
    pub financed: &'a Decimal,
    pub deposit: &'a Decimal,
    pub trade_in: &'a Decimal,
    pub gst: Option<Booked>,
    pub periods: &'a [Period],
    /// The payout's day, the balance it discharges and its amount.
    pub payout: Option<(i64, Decimal, Decimal)>,
    /// Whether the schedule runs to the end of the term.
    pub whole_term: bool,
}

/// The journals as structured output.
pub(super) fn journals(facts: &Facts) -> Result<Value> {
    let consideration = facts.financed.add(facts.deposit)?.add(facts.trade_in)?;
    let mut start: Lines = Vec::new();
    let (on_price, on_interest) = match &facts.gst {
        Some(g) => (g.credit.sub(&g.on_interest)?, g.on_interest.clone()),
        None => (int(0), int(0)),
    };
    post(&mut start, ASSET, &consideration.sub(&on_price)?)?;
    if let Some(g) = &facts.gst {
        post(&mut start, CREDIT, &g.credit)?;
    }
    post(&mut start, LIABILITY, &facts.financed.neg()?)?;
    post(&mut start, BANK, &facts.deposit.neg()?)?;
    post(&mut start, TRADE_IN, &facts.trade_in.neg()?)?;
    post(&mut start, DEFERRED, &on_interest.neg()?)?;
    let (start_year, _) = income_year(facts.start);
    let mut years: Vec<(String, Lines)> = vec![(start_year, start.clone())];
    let (mut entries, mut across) = (Vec::new(), Vec::new());
    let (mut interest_to_date, mut gst_shown) = (int(0), int(0));
    for (i, period) in facts.periods.iter().enumerate() {
        interest_to_date = interest_to_date.add(&period.interest)?;
        // The credit's share of the interest to date, rounded, less what the
        // periods before show, so the shares add up to the whole.
        let share = match &facts.gst {
            None => int(0),
            Some(g) => {
                let through = if facts.whole_term && i + 1 == facts.periods.len() {
                    g.on_interest.clone()
                } else {
                    interest_to_date
                        .div(&int(11))?
                        .mul(&g.purpose)?
                        .quantize(-2, Rounding::HalfUp)?
                };
                let share = through.sub(&gst_shown)?;
                gst_shown = through;
                share
            }
        };
        let mut lines: Lines = Vec::new();
        post(&mut lines, INTEREST, &period.interest.sub(&share)?)?;
        post(&mut lines, DEFERRED, &share)?;
        post(&mut lines, LIABILITY, &period.paid.sub(&period.interest)?)?;
        post(&mut lines, BANK, &period.paid.neg()?)?;
        entries.push(json!({
            "instalment": period.instalment,
            "date": dates::format(period.to),
            "lines": render(&lines),
        }));
        let (year, ends) = income_year(period.from);
        if period.to > ends {
            across.push(json!({
                "instalment": period.instalment,
                "income_years": [year, income_year(ends).0],
                "lines": render(&lines),
            }));
            continue;
        }
        match years.iter_mut().find(|(y, _)| *y == year) {
            Some((_, totals)) => add_into(totals, &lines)?,
            None => years.push((year, lines)),
        }
    }
    let payout = match &facts.payout {
        None => Value::Null,
        Some((day, balance, amount)) => {
            let mut lines: Lines = Vec::new();
            post(&mut lines, LIABILITY, balance)?;
            post(&mut lines, DIFFERENCE, &amount.sub(balance)?)?;
            post(&mut lines, BANK, &amount.neg()?)?;
            let (year, _) = income_year(*day);
            match years.iter_mut().find(|(y, _)| *y == year) {
                Some((_, totals)) => add_into(totals, &lines)?,
                None => years.push((year, lines.clone())),
            }
            json!({"date": dates::format(*day), "lines": render(&lines)})
        }
    };
    let by_income_year: Vec<Value> = years
        .iter()
        .map(|(year, lines)| json!({"income_year": year, "lines": render(lines)}))
        .collect();
    Ok(json!({
        "gst_booked": facts.gst.is_some(),
        "start": {"date": dates::format(facts.start), "lines": render(&start)},
        "periods": entries,
        "payout": payout,
        "by_income_year": by_income_year,
        "across_30_june": across,
    }))
}
