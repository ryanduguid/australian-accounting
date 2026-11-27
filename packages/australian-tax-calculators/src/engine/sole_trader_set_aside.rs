// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `sole_trader_set_aside`: what a full-year resident sole trader sets aside
//! from one income year's business income, as the site's sole trader page
//! works it (`assets/sole-trader.mjs`): the income tax, Medicare levy, any
//! surcharge and study loan repayment that the profit adds to any wages, after
//! the small business income tax offset, and the GST on taxable sales less
//! GST credits, as a share of each invoice and an amount each quarter, month
//! and week. Read on 26 November 2026: ITAA 1997 ss 328-360 and 328-365 and
//! 25-130 on the ATO's legal database; the ATO's 'Registering for GST' (last
//! updated 14 September 2026, the $75,000 threshold) and 'Starting PAYG
//! instalments' (last updated 17 December 2024: instalment income of $4,000
//! or more and tax payable of $1,000 or more on the latest assessment).

use super::individual_owed::{Facts, Owed, owed};
use super::{
    Arg, Result, d, flag, int, money, precise, py_max, py_min, rate, refuse, result, scope,
};
use crate::decimal::{Decimal, Rounding};
use serde_json::{Value, json};

const KIND: &str = "sole_trader_set_aside";

/// ITAA 1997 s 328-360(1) and (2): 16% from 2021-22, at most $1,000.
const OFFSET_RATE: &str = "0.16";
const OFFSET_CAP: &str = "1000";
const STANDARD_DEDUCTION: &str = "1000";
const STANDARD_DEDUCTION_FROM: &str = "2026-27";
/// GST Act s 9-70: 10% of the value of a taxable supply.
const GST_RATE: &str = "0.10";
const GST_THRESHOLD: &str = "75000";
const INSTALMENT_INCOME: &str = "4000";
const INSTALMENT_TAX: &str = "1000";

/// The facts as the command passes them.
pub struct SoleTraderSetAsideArgs<'a> {
    pub business_income: &'a Arg,
    pub business_expenses: &'a Arg,
    pub wages: &'a Arg,
    pub other_deductions: &'a Arg,
    pub personal_super_deducted: &'a Arg,
    pub small_business_offset: &'a Arg,
    pub study_loan: &'a Arg,
    pub hospital_cover: &'a Arg,
    pub gst_registered: &'a Arg,
    pub gst_credits: &'a Arg,
    pub invoice: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

struct Flags {
    offset: bool,
    study_loan: bool,
    hospital_cover: bool,
    gst: bool,
}

/// One assessment: the parts and the offset applied.
struct Assessed {
    owed: Owed,
    offset: Decimal,
    total: Decimal,
}

fn cents(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(-2, Rounding::HalfUp)?)
}

fn whole(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(0, Rounding::Down)?)
}

/// The year's parts on whole-dollar taxable income, with `business` dollars
/// of net small business income for the offset: 16% of total net small
/// business income (no more than taxable income) over taxable income times the
/// basic income tax liability (s 328-360(1)), at most $1,000 (s 328-360(2)),
/// applied after the low income tax offset and never below nil (s 63-10(1)
/// item 20). Deducted personal super contributions are reportable super
/// contributions for the surcharge and repayment income.
fn assess(
    year: &str,
    taxable: Decimal,
    business: &Decimal,
    flags: &Flags,
    reported: Decimal,
) -> Result<Assessed> {
    let facts = Facts {
        year,
        foreign: false,
        taxable: taxable.clone(),
        fringe: int(0),
        reported_super: reported,
        study_loan: flags.study_loan,
        hospital_cover: flags.hospital_cover,
    };
    let owed = owed(&facts)?;
    let share = if taxable.is_zero() {
        int(0)
    } else {
        let counted = py_min(business.clone(), taxable.clone())?;
        let offset_share = cents(
            &d(OFFSET_RATE)
                .mul(&counted)?
                .mul(&owed.basic)?
                .div(&taxable)?,
        );
        offset_share?
    };
    let offset = py_min(py_min(share, d(OFFSET_CAP))?, owed.net_tax()?)?;
    let total = owed.total()?.sub(&offset)?;
    Ok(Assessed {
        owed,
        offset,
        total,
    })
}

/// The GST on the income and the credits, refused where the page refuses.
fn gst(income: &Decimal, credits: &Decimal, registered: bool) -> Result<Decimal> {
    let on_income = if registered {
        cents(&income.mul(&d(GST_RATE))?)?
    } else {
        int(0)
    };
    if !registered && !credits.is_zero() {
        return refuse(
            "sole_trader_set_aside.credits_unregistered",
            "gst_credits must be 0 when the business is not registered for GST.",
        );
    }
    if credits.gt(&on_income)? {
        return refuse(
            "sole_trader_set_aside.gst_refund",
            "gst_credits are more than the GST on the business income: a refund is not worked here.",
        );
    }
    Ok(on_income)
}

fn flags(args: &SoleTraderSetAsideArgs) -> Result<Flags> {
    Ok(Flags {
        offset: flag(args.small_business_offset, "small_business_offset")?,
        study_loan: flag(args.study_loan, "study_loan")?,
        hospital_cover: flag(args.hospital_cover, "hospital_cover")?,
        gst: flag(args.gst_registered, "gst_registered")?,
    })
}

/// The amounts as stated.
struct Stated {
    income: Decimal,
    expenses: Decimal,
    wages: Decimal,
    deductions: Decimal,
    contributions: Decimal,
    credits: Decimal,
    invoice: Decimal,
}

fn stated(args: &SoleTraderSetAsideArgs) -> Result<Stated> {
    Ok(Stated {
        income: money(args.business_income)?,
        expenses: money(args.business_expenses)?,
        wages: money(args.wages)?,
        deductions: money(args.other_deductions)?,
        contributions: money(args.personal_super_deducted)?,
        credits: money(args.gst_credits)?,
        invoice: money(args.invoice)?,
    })
}

/// The standard deduction from 2026-27 (s 25-130), from the wages only.
fn standard_deduction(year: &str, wages: &Decimal) -> Result<Decimal> {
    if year >= STANDARD_DEDUCTION_FROM {
        Ok(py_min(d(STANDARD_DEDUCTION), wages.clone())?)
    } else {
        Ok(int(0))
    }
}

/// On the wages alone, only the contributions the wages left after the other
/// deductions can absorb are deducted, and so reported.
fn wages_alone(year: &str, room: &Decimal, m: &Stated, flags: &Flags) -> Result<Assessed> {
    let left = py_max(int(0), room.sub(&m.deductions)?)?;
    let alone_super = py_min(m.contributions.clone(), left.clone())?;
    let alone_taxable = whole(&left.sub(&alone_super)?)?;
    assess(year, alone_taxable, &int(0), flags, whole(&alone_super)?)
}

pub fn sole_trader_set_aside(args: &SoleTraderSetAsideArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let flags = flags(args)?;
    let m = stated(args)?;
    precise(|| {
        if m.expenses.gt(&m.income)? {
            return refuse(
                "sole_trader_set_aside.business_loss",
                "business_expenses are more than business_income: a loss is not worked here.",
            );
        }
        let profit = m.income.sub(&m.expenses)?;
        let deduction = standard_deduction(&year, &m.wages)?;
        let room = m.wages.sub(&deduction)?;
        let claimed = m.deductions.add(&m.contributions)?;
        if claimed.gt(&profit.add(&room)?)? {
            return refuse(
                "sole_trader_set_aside.deductions_exceed_income",
                "other_deductions and personal_super_deducted are more than the profit and wages less any standard deduction.",
            );
        }
        let gst_on_income = gst(&m.income, &m.credits, flags.gst)?;
        let taxable = whole(&profit.add(&room)?.sub(&claimed)?)?;
        let business = if flags.offset {
            whole(&profit)?
        } else {
            int(0)
        };
        let reported = whole(&m.contributions)?;
        let all = assess(&year, taxable.clone(), &business, &flags, reported)?;
        let alone = wages_alone(&year, &room, &m, &flags)?;
        let parts = Parts {
            profit,
            deduction,
            taxable,
            gst_on_income,
            credits: m.credits.clone(),
            invoice: m.invoice.clone(),
            income: m.income.clone(),
        };
        finish(&year, &parts, &all, &alone, flags.gst)
    })
}

struct Parts {
    income: Decimal,
    profit: Decimal,
    deduction: Decimal,
    taxable: Decimal,
    gst_on_income: Decimal,
    credits: Decimal,
    invoice: Decimal,
}

/// A percentage of `part` in `whole`, half up to 2 places; nil for no whole.
fn percent(part: &Decimal, whole: &Decimal) -> Result<Decimal> {
    if whole.is_zero() {
        return Ok(int(0));
    }
    cents(&part.mul(&int(100))?.div(whole)?)
}

fn finish(
    year: &str,
    p: &Parts,
    all: &Assessed,
    alone: &Assessed,
    registered: bool,
) -> Result<Value> {
    let business_tax = all.total.sub(&alone.total)?;
    let gst = p.gst_on_income.sub(&p.credits)?;
    let set_aside = business_tax.add(&gst)?;
    let invoiced = p.income.add(&p.gst_on_income)?;
    let from_invoice = if invoiced.is_zero() {
        int(0)
    } else {
        cents(&p.invoice.mul(&set_aside)?.div(&invoiced)?)?
    };
    let each = |n: i64| -> Result<Decimal> { cents(&set_aside.div(&int(n))?) };
    let amounts = vec![
        ("profit", p.profit.clone()),
        ("standard_deduction", p.deduction.clone()),
        ("taxable_income", p.taxable.clone()),
        ("basic_income_tax", all.owed.basic.clone()),
        ("low_income_tax_offset", all.owed.lito.clone()),
        ("small_business_income_tax_offset", all.offset.clone()),
        ("medicare_levy", all.owed.levy.clone()),
        ("medicare_levy_surcharge", all.owed.surcharge.clone()),
        ("study_loan_repayment", all.owed.loan.clone()),
        ("total", all.total.clone()),
        ("wages_alone_total", alone.total.clone()),
        ("business_tax", business_tax.clone()),
        ("gst_on_income", p.gst_on_income.clone()),
        ("gst_credits", p.credits.clone()),
        ("gst", gst),
        ("set_aside", set_aside.clone()),
        ("tax_share_percent", percent(&business_tax, &p.income)?),
        ("invoice_share_percent", percent(&set_aside, &invoiced)?),
        ("quarterly", each(4)?),
        ("monthly", each(12)?),
        ("weekly", each(52)?),
        ("invoice", p.invoice.clone()),
        ("invoice_set_aside", from_invoice.clone()),
        ("invoice_left", p.invoice.sub(&from_invoice)?),
    ];
    let mut out = result(KIND, year, amounts, rates())?;
    out["payg_instalments_likely"] = json!(instalments(&p.income, all)?);
    out["gst_registration_required"] = json!(!registered && !p.income.lt(&d(GST_THRESHOLD))?);
    Ok(out)
}

/// The ATO's entry figures for PAYG instalments: instalment income of $4,000
/// or more and tax payable of $1,000 or more.
fn instalments(income: &Decimal, all: &Assessed) -> Result<bool> {
    Ok(!income.lt(&d(INSTALMENT_INCOME))?
        && !all.total.sub(&all.owed.loan)?.lt(&d(INSTALMENT_TAX))?)
}

fn rates() -> Vec<(String, String)> {
    vec![
        rate("offset_rate", OFFSET_RATE),
        rate("offset_cap", OFFSET_CAP),
        rate("gst_rate", GST_RATE),
        rate("gst_threshold", GST_THRESHOLD),
    ]
}
