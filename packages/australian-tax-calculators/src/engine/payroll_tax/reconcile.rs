// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The annual reconciliation `payroll_tax` adds when the year's periodic
//! returns are stated: the year's liability less the payroll tax (and any
//! surcharge, or Queensland's levy) paid or payable on them, as each Act's
//! annual adjustment reads. Each was read on 26 November 2026, and the
//! Payroll Tax Act 2007 (NSW) s 83 at AustLII on 27 November 2026.

use super::PayrollTaxArgs;
use crate::decimal::Decimal;
use crate::engine::trade_in::when;
use crate::engine::{Result, Step, cents, int, money, py_max, step};

/// The payroll tax, and in Queensland the levy, on the year's periodic
/// returns, and where s 83 reduces a refund by them, the other refunds of
/// payroll tax already made for the year.
pub(super) struct Periodic {
    tax: Decimal,
    levy: Option<Decimal>,
    other_refunds: Option<Decimal>,
}

/// The jurisdictions whose s 83 reduces the annual refund by any other
/// refund of payroll tax made for the year: NSW, Vic, SA, Tas and NT
/// s 83(4), ACT s 83(5).
const REDUCED: [&str; 6] = ["nsw", "vic", "sa", "tas", "act", "nt"];

/// The stated periodic amounts.
pub(super) fn periodic(args: &PayrollTaxArgs, place: &str) -> Result<Option<Periodic>> {
    let Some(tax) = args.periodic_tax else {
        when(
            false,
            args.periodic_levy,
            "periodic_levy",
            "beside periodic_tax",
        )?;
        when(
            false,
            args.other_refunds,
            "other_refunds",
            "beside periodic_tax",
        )?;
        return Ok(None);
    };
    let levy = when(
        place == "qld",
        args.periodic_levy,
        "periodic_levy",
        "for Queensland, beside periodic_tax (Payroll Tax Act 1971 (Qld) s 43H)",
    )?;
    let other_refunds = when(
        REDUCED.contains(&place),
        args.other_refunds,
        "other_refunds",
        "beside periodic_tax for nsw, vic, sa, tas, act and nt, whose s 83 reduces the annual refund by any other refund of payroll tax made for the year (nil if none)",
    )?;
    Ok(Some(Periodic {
        tax: money(tax)?,
        levy: levy.map(money).transpose()?,
        other_refunds: other_refunds.map(money).transpose()?,
    }))
}

const PERIODIC: &str = "The payroll tax, with any surcharge, paid or payable on the returns for the year's return periods, as stated";
const PERIODIC_QLD: &str = "Payroll Tax Act 1971 (Qld) ss 29(1) and 30(1)(a): the employer's periodic liability for the periodic return periods in the year, as stated";
const PERIODIC_LEVY: &str = "Payroll Tax Act 1971 (Qld) s 43H(a): the combined periodic liability, the periodic levy liability for each periodic return period in the year, as stated";

/// Each Act's annual adjustment: (shortfall payable, excess refunded).
fn adjustment(place: &str) -> (&'static str, &'static str) {
    match place {
        "nsw" => (
            "Payroll Tax Act 2007 (NSW) s 83(2) and (3), AustLII consolidation as at 22 October 2026, read on 27 November 2026: the correct amount for the year less the amount paid or payable on the year's returns, payable with the June return",
            "Payroll Tax Act 2007 (NSW) s 83(1) and (4): the amount paid or payable on the year's returns less the correct amount, refunded on application and reduced by any other refund of payroll tax made for the year",
        ),
        "vic" => (
            "Payroll Tax Act 2007 (Vic) s 83(2) and (3), authorised version 041: the correct amount for the year (s 82, Sch 1) less the amount paid or payable on the year's returns, payable with the June return",
            "Payroll Tax Act 2007 (Vic) s 83(1) and (4): the amount paid or payable on the year's returns less the correct amount, refunded on application and reduced by any other refund of payroll tax made for the year",
        ),
        "qld" => (
            "Payroll Tax Act 1971 (Qld) s 30(1)(a), current as at 22 October 2026: the annual adjustment amount, the annual payroll tax amount less the periodic liability",
            "Payroll Tax Act 1971 (Qld) s 31(1) and (2): the annual refund amount, the periodic liability less the annual payroll tax amount",
        ),
        "wa" => (
            "Pay-roll Tax Assessment Act 2002 (WA) s 16(2), version 04-d0-03: the year's tax (s 10 or 13) less the tax paid or payable for the progressive return periods, payable",
            "Pay-roll Tax Assessment Act 2002 (WA) s 16(1): the tax paid or payable for the progressive return periods less the year's tax, refunded or rebated",
        ),
        "sa" => (
            "Payroll Tax Act 2009 (SA) s 83(2) and (3), version 31.3.2026: the correct amount for the year less the amount paid or payable on the year's returns, payable with the June return",
            "Payroll Tax Act 2009 (SA) s 83(1) and (4): the amount paid or payable on the year's returns less the correct amount, refunded on application and reduced by any other refund of payroll tax made for the year",
        ),
        "tas" => (
            "Payroll Tax Act 2008 (Tas) s 83(2) and (3): the correct amount for the year less the amount paid or payable on the year's returns, payable with the June return",
            "Payroll Tax Act 2008 (Tas) s 83(1) and (4): the amount paid or payable on the year's returns less the correct amount, refunded on application and reduced by any other refund of payroll tax made for the year",
        ),
        "act" => (
            "Payroll Tax Act 2011 (ACT) s 83(2) to (4), republication R20: the correct amount for the year less the amount paid or payable on the year's returns, payable with the June return; a difference of less than $20 is determined to be $0",
            "Payroll Tax Act 2011 (ACT) s 83(1) and (5): the amount paid or payable on the year's returns less the correct amount, refunded on application and reduced by any other refund of payroll tax made for the year",
        ),
        _ => (
            "Payroll Tax Act 2009 (NT) s 83(2) and (3), as in force at 1 July 2026: the correct amount for the year less the amount paid or payable on the year's returns, payable with the June return",
            "Payroll Tax Act 2009 (NT) s 83(1) and (4): the amount paid or payable on the year's returns less the correct amount, refunded on application and reduced by any other refund of payroll tax made for the year",
        ),
    }
}

const LEVY_AMOUNT: &str = "Payroll Tax Act 1971 (Qld) s 43I: the annual levy amount, the primary and additional annual levy amounts for the year, each to the cent";
const LEVY_DUE: &str = "Payroll Tax Act 1971 (Qld) s 43J(1)(a): the annual levy adjustment amount, the annual levy amount (s 43I) less the combined periodic liability";
const LEVY_BACK: &str = "Payroll Tax Act 1971 (Qld) s 43K(1) and (2): the annual levy refund amount, the combined periodic liability less the annual levy amount";

/// One settlement: the amount paid (`paid`, its id, authority and value),
/// the ids and authorities of what is payable and what is refunded, the
/// year's amount `annual` that `name` works from the `operand` the working
/// shows, the ACT's $20 floor (`small`) and the other refunds that reduce the
/// refund.
struct Settle<'a> {
    paid: (&'static str, &'static str, &'a Decimal),
    ids: (&'static str, &'static str),
    authorities: (&'static str, &'static str),
    name: &'a str,
    annual: &'a Decimal,
    operand: &'a Decimal,
    small: Option<&'a Decimal>,
    other_refunds: Option<&'a Decimal>,
}

/// `paid` as a step, then what is payable and what is refunded.
fn settle(s: &Settle) -> Result<[Step; 3]> {
    let (paid_id, paid_authority, paid) = s.paid;
    let ((due_id, back_id), (due_authority, back_authority)) = (s.ids, s.authorities);
    let (name, operand) = (s.name, s.operand);
    let shown = step(paid_id, paid_authority, paid_id, &[paid], paid.clone());
    let mut owed = py_max(int(0), s.annual.sub(paid)?)?;
    let mut formula = format!("max(0, {name} - {paid_id})");
    if let Some(small) = s.small
        && owed.lt(small)?
    {
        owed = int(0);
        formula = format!("0 (max(0, {name} - {paid_id}) < 20)");
    }
    let due = step(due_id, due_authority, &formula, &[operand, paid], owed);
    let excess = py_max(int(0), paid.sub(s.annual)?)?;
    let back = match s.other_refunds {
        Some(other) => step(
            back_id,
            back_authority,
            &format!("max(0, max(0, {paid_id} - {name}) - other_refunds)"),
            &[paid, operand, other],
            py_max(int(0), excess.sub(other)?)?,
        ),
        None => step(
            back_id,
            back_authority,
            &format!("max(0, {paid_id} - {name})"),
            &[paid, operand],
            excess,
        ),
    };
    Ok([shown, due, back])
}

/// The amount a step named `id` in `steps` holds, exactly and to the cent.
fn held(steps: &[Step], id: &str) -> Result<(Decimal, Decimal)> {
    let s = steps
        .iter()
        .find(|s| s.id == id)
        .expect("a step of the year");
    Ok((s.value.clone(), cents(s.value.clone())?))
}

const IDS: (&str, &str) = ("adjustment_payable", "adjustment_refund");

/// The reconciliation's steps, after the year's `steps`.
pub(super) fn steps(place: &str, year: &[Step], p: &Periodic) -> Result<Vec<Step>> {
    let threshold = int(20);
    if place != "qld" {
        let (exact, annual) = held(year, "total")?;
        let settled = settle(&Settle {
            paid: ("periodic_tax", PERIODIC, &p.tax),
            ids: IDS,
            authorities: adjustment(place),
            name: "total",
            annual: &annual,
            operand: &exact,
            small: (place == "act").then_some(&threshold),
            other_refunds: p.other_refunds.as_ref(),
        })?;
        return Ok(settled.into());
    }
    qld_steps(year, p)
}

/// Queensland's reconciliation: payroll tax (ss 30 and 31), then the levy
/// (ss 43I to 43K).
fn qld_steps(year: &[Step], p: &Periodic) -> Result<Vec<Step>> {
    let (exact, tax) = held(year, "payroll_tax")?;
    let mut out: Vec<Step> = settle(&Settle {
        paid: ("periodic_tax", PERIODIC_QLD, &p.tax),
        ids: IDS,
        authorities: adjustment("qld"),
        name: "cents(payroll_tax)",
        annual: &tax,
        operand: &exact,
        small: None,
        other_refunds: None,
    })?
    .into();
    let [primary, additional] = [
        "mental_health_levy_primary",
        "mental_health_levy_additional",
    ]
    .map(|id| held(year, id));
    let [(primary_exact, primary), (additional_exact, additional)] = [primary?, additional?];
    let levy = step(
        "annual_levy_amount",
        LEVY_AMOUNT,
        "cents(mental_health_levy_primary) + cents(mental_health_levy_additional)",
        &[&primary_exact, &additional_exact],
        primary.add(&additional)?,
    );
    let periodic_levy = p.levy.as_ref().expect("required for Queensland");
    let settled = settle(&Settle {
        paid: ("periodic_levy", PERIODIC_LEVY, periodic_levy),
        ids: ("levy_adjustment_payable", "levy_adjustment_refund"),
        authorities: (LEVY_DUE, LEVY_BACK),
        name: "annual_levy_amount",
        annual: &levy.value,
        operand: &levy.value,
        small: None,
        other_refunds: None,
    })?;
    out.push(levy);
    out.extend(settled);
    Ok(out)
}
