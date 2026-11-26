// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `payroll_tax_monthly`: the payroll tax, and any surcharge or levy, one
//! employer outside a group owes on one monthly return in 2026-27, as each
//! Act's monthly provisions read: New South Wales, Victoria, Queensland,
//! Western Australia, South Australia, Tasmania, the ACT and the Northern
//! Territory. Each Act other than New South Wales's, and the ACT's
//! DI2026-151, were read at the official legislation site on 26 November
//! 2026; the Payroll Tax Act 2007 (NSW) was read at AustLII (consolidation as
//! at 22 October 2026) on 27 November 2026, as legislation.nsw.gov.au served a
//! Cloudflare challenge. docs/calculation-evidence.md#payroll-tax-monthly
//! lists the versions.

use serde_json::Value;

use super::{TAS_HIGHER, TAS_LOWER, Worked, totalled};
use crate::decimal::Decimal;
use crate::engine::trade_in::when;
use crate::engine::{
    Arg, Result, Step, d, derived_result, flag, int, money, precise, py_max, py_min, rate, refuse,
    scope, step,
};

mod qld;
mod states;
use qld::qld;
use states::{act, nt, sa, vic, wa};

const KIND: &str = "payroll_tax_monthly";

/// The facts as the command passes them; see the README for each.
pub struct PayrollTaxMonthlyArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub month: &'a Arg,
    pub taxable_wages: &'a Arg,
    pub interstate_wages: &'a Arg,
    pub estimated_taxable_wages: &'a Arg,
    pub estimated_interstate_wages: &'a Arg,
    pub regional_employer: Option<&'a Arg>,
    pub previous_interstate_wage_payer: Option<&'a Arg>,
    pub stated_deduction: Option<&'a Arg>,
    pub stated_primary_threshold: Option<&'a Arg>,
    pub stated_additional_threshold: Option<&'a Arg>,
    pub group_member: &'a Arg,
    pub employed_all_year: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One month's facts: the month's taxable (`tw`) and interstate (`iw`)
/// wages, the year's estimated taxable (`etw`) and interstate (`eiw`) wages,
/// the days in the month, any stated deduction and, in Queensland, any
/// determined primary and additional periodic thresholds.
struct Month {
    tw: Decimal,
    iw: Decimal,
    etw: Decimal,
    eiw: Decimal,
    days: Decimal,
    stated: Option<Decimal>,
    thresholds: [Option<Decimal>; 2],
    regional: bool,
    previous: bool,
}

impl Month {
    /// The year's estimated taxable and interstate wages together.
    fn estimated(&self) -> Result<Decimal> {
        Ok(self.etw.add(&self.eiw)?)
    }

    /// `amount` x etw / (etw + eiw): an amount shared by the estimated
    /// taxable wages over the estimated Australian wages.
    fn shared(&self, amount: &Decimal) -> Result<Decimal> {
        Ok(amount.mul(&self.etw)?.div(&self.estimated()?)?)
    }
}

const TAXABLE: &str = "The month's taxable wages less the deduction, never below nil: the amount the rate applies to (each Act: if the deduction is equal to or more than the taxable wages, no payroll tax is payable for the month)";
const NO_LEVY: &str = "0";

/// `deduction`, then `taxable_amount` and `payroll_tax` at `applied`.
fn taxed(
    m: &Month,
    deduction: Step,
    (authority, applied): (&'static str, &Decimal),
) -> Result<Vec<Step>> {
    let rest = py_max(int(0), m.tw.sub(&deduction.value)?)?;
    let operands = [&m.tw, &deduction.value];
    let taxable = step(
        "taxable_amount",
        TAXABLE,
        "max(0, taxable_wages - deduction)",
        &operands,
        rest,
    );
    let tax = taxable.value.mul(applied)?;
    let operands = [&taxable.value, applied];
    let payroll = step(
        "payroll_tax",
        authority,
        "taxable_amount x rate",
        &operands,
        tax,
    );
    Ok(vec![deduction, taxable, payroll])
}

/// A nil step, for a surcharge or levy that does not apply.
fn nil(id: &'static str, authority: &'static str) -> Step {
    step(id, authority, NO_LEVY, &[], int(0))
}

/// `annualised_wages`: the month's taxable wages over the 365 days of 2026-27.
fn annualise(m: &Month, authority: &'static str) -> Result<Step> {
    let year = int(365);
    let annual = m.tw.mul(&year)?.div(&m.days)?;
    let operands = [&m.tw, &year, &m.days];
    let formula = "taxable_wages x days_in_year / days_in_month";
    Ok(step(
        "annualised_wages",
        authority,
        formula,
        &operands,
        annual,
    ))
}

/// `payroll_tax`: the month's share of the year's tax.
fn month_share(m: &Month, year_tax: &Step, authority: &'static str) -> Result<Step> {
    let year = int(365);
    let tax = year_tax.value.mul(&m.days)?.div(&year)?;
    let operands = [&year_tax.value, &m.days, &year];
    let formula = "annual_payroll_tax x days_in_month / days_in_year";
    Ok(step("payroll_tax", authority, formula, &operands, tax))
}

/// Days in each month of 2026-27, by its number.
fn month_days(month: Option<&str>) -> Result<(u32, i64)> {
    let parsed = month.and_then(|m| {
        let (year, number) = m.split_once('-')?;
        let number: u32 = number.parse().ok().filter(|_| number.len() == 2)?;
        match (year, number) {
            ("2026", 7..=12) | ("2027", 1..=6) => Some(number),
            _ => None,
        }
    });
    let Some(number) = parsed else {
        return refuse(
            "payroll_tax_monthly.month_invalid",
            "month must be a month of 2026-27, from 2026-07 to 2027-06.",
        );
    };
    let days = match number {
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    Ok((number, days))
}

const JURISDICTIONS: &str = "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.";

/// The jurisdiction, or the refusal for one not worked.
fn place(given: Option<&str>) -> Result<&str> {
    match given {
        Some(place @ ("nsw" | "vic" | "qld" | "wa" | "sa" | "tas" | "act" | "nt")) => Ok(place),
        _ => refuse("payroll_tax_monthly.jurisdiction_invalid", JURISDICTIONS),
    }
}

/// Refuses an employer or month this worksheet does not work.
fn refuse_unsupported(args: &PayrollTaxMonthlyArgs, place: &str, number: u32) -> Result<()> {
    if flag(args.group_member, "group_member")? {
        return refuse(
            "payroll_tax_monthly.grouped",
            "A member of a group is not worked: each Act sets a group's monthly deduction on the group's wages, and only the designated group employer claims it.",
        );
    }
    if !flag(args.employed_all_year, "employed_all_year")? {
        return refuse(
            "payroll_tax_monthly.part_year",
            "An employer that paid wages for only part of the year is not worked: Queensland and Western Australia scale the month's deduction by the days wages were paid, and South Australia's annualised wages count the days in the year wages were paid.",
        );
    }
    if place == "qld" && number == 6 {
        return refuse(
            "payroll_tax_monthly.last_period",
            "June is the last periodic return period of the year in Queensland, for which no periodic liability or periodic levy liability arises (Payroll Tax Act 1971 (Qld) ss 20(2A) and 43F(4)): the annual return works out June's payroll tax and levy. Use payroll-tax with periodic_tax and periodic_levy.",
        );
    }
    if place == "nsw" && number == 6 {
        return refuse(
            "payroll_tax_monthly.last_period",
            "June is not worked in New South Wales: Revenue NSW's 'Calculate and pay your monthly payroll tax return' (last updated 26 October 2026, read on 27 November 2026) says the month of June is incorporated into the annual return, which settles the year under Payroll Tax Act 2007 (NSW) s 83. Use payroll-tax with periodic_tax and other_refunds.",
        );
    }
    Ok(())
}

const STATED: &str = "for the Northern Territory and an interstate non-group employer in Western Australia (Payroll Tax Act 2009 (NT) Sch 2 cll 6 and 7; Pay-roll Tax Assessment Act 2002 (WA) s 15), and otherwise only as a notified or determined deductible amount: in Queensland (s 21), South Australia (Sch 2 cl 6), and with interstate wages in Victoria (Sch 2 cl 6) and the ACT (Sch 2 s 2.4)";

/// The stated deduction: required when `needed`, taken when `allowed`.
fn stated(args: &PayrollTaxMonthlyArgs, needed: bool, allowed: bool) -> Result<Option<Decimal>> {
    let stated = if allowed && !needed {
        args.stated_deduction
    } else {
        when(needed, args.stated_deduction, "stated_deduction", STATED)?
    };
    stated.map(money).transpose()
}

/// Queensland's determined periodic thresholds, each optional; refused
/// elsewhere.
fn thresholds(args: &PayrollTaxMonthlyArgs, place: &str) -> Result<[Option<Decimal>; 2]> {
    let mut out = [None, None];
    let given = [
        ("stated_primary_threshold", args.stated_primary_threshold),
        (
            "stated_additional_threshold",
            args.stated_additional_threshold,
        ),
    ];
    for (slot, (key, value)) in out.iter_mut().zip(given) {
        let why = "for Queensland only (Payroll Tax Act 1971 (Qld) s 43E(1))";
        let value = if place == "qld" {
            value
        } else {
            when(false, value, key, why)?
        };
        *slot = value.map(money).transpose()?;
    }
    Ok(out)
}

/// Whether the employer is a regional employer (Vic and Qld) and, in a
/// Queensland month without interstate wages (`local`), a previous interstate
/// wage payer.
fn flags(args: &PayrollTaxMonthlyArgs, place: &str, local: bool) -> Result<(bool, bool)> {
    let regional = when(
        matches!(place, "vic" | "qld"),
        args.regional_employer,
        "regional_employer",
        "for Victoria and Queensland (Payroll Tax Act 2007 (Vic) s 3A; Payroll Tax Act 1971 (Qld) s 10A)",
    )?;
    let regional = regional.map(|r| flag(r, "regional_employer")).transpose()?;
    let previous = when(
        place == "qld" && local,
        args.previous_interstate_wage_payer,
        "previous_interstate_wage_payer",
        "for Queensland when the month has no interstate wages (Payroll Tax Act 1971 (Qld) s 20(2)(b) and (3))",
    )?;
    let previous = previous
        .map(|p| flag(p, "previous_interstate_wage_payer"))
        .transpose()?;
    Ok((regional == Some(true), previous == Some(true)))
}

/// The month's facts, with each jurisdiction's own inputs.
fn facts(args: &PayrollTaxMonthlyArgs, place: &str, days: i64) -> Result<Month> {
    let [tw, iw, etw, eiw] = [
        args.taxable_wages,
        args.interstate_wages,
        args.estimated_taxable_wages,
        args.estimated_interstate_wages,
    ]
    .map(money);
    let [tw, iw, etw, eiw] = [tw?, iw?, etw?, eiw?];
    if etw.is_zero() {
        return refuse(
            "payroll_tax_monthly.estimate_invalid",
            "estimated_taxable_wages must be more than nil: each Act shares the deduction, or sets the rate, on the year's estimated wages.",
        );
    }
    let (regional, previous) = flags(args, place, iw.is_zero())?;
    let wa_interstate = place == "wa" && !(iw.is_zero() && eiw.is_zero());
    let notified = match place {
        "qld" | "sa" => true,
        "vic" | "act" => !iw.is_zero(),
        _ => false,
    };
    let stated = stated(args, place == "nt" || wa_interstate, notified)?;
    Ok(Month {
        tw,
        iw,
        etw,
        eiw,
        days: int(days),
        stated,
        thresholds: thresholds(args, place)?,
        regional,
        previous,
    })
}

const TAS_YEAR: &str = "Payroll Tax Act 2008 (Tas) Sch 2 cl 3(b): the year's taxable wages are taken to be the month's multiplied by the days in the financial year (365 in 2026-27) over the days the return relates to, and cl 3(a): wages are taken to be paid all year";
const TAS_ANNUAL: &str = "Payroll Tax Act 2008 (Tas) Sch 2 cl 3 and Sch 1 cl 4(5) (the 2-rate formula, read as published on 26 November 2026): the payroll tax payable for the whole year on the annualised wages";
const TAS_MONTH: &str = "Payroll Tax Act 2008 (Tas) Sch 2 cl 2: the month's payroll tax is the year's in the ratio of the days the return relates to over the days in the financial year";

/// Tasmania's 2 rate steps and the rates they use.
type TasRates = ([Step; 2], Vec<(String, String)>);

/// Sch 1's tax at each rate on `annual` wages, all in Tasmania, and its
/// rates.
fn tas_rates(annual: &Decimal) -> Result<TasRates> {
    let [threshold, top, lower, higher] = ["1250000", "2000000", "0.04", "0.061"].map(d);
    let capped = py_min(annual.clone(), top.clone())?;
    let band = py_max(int(0), capped.sub(&threshold)?)?;
    let first = step(
        "tax_at_lower_rate",
        TAS_LOWER,
        "max(0, min(annualised_wages, top_bracket) - threshold) x lower_rate",
        &[annual, &top, &threshold, &lower],
        band.mul(&lower)?,
    );
    let above = py_max(int(0), annual.sub(&top)?)?;
    let second = step(
        "tax_at_higher_rate",
        TAS_HIGHER,
        "max(0, annualised_wages - top_bracket) x higher_rate",
        &[annual, &top, &higher],
        above.mul(&higher)?,
    );
    let rates = vec![
        rate("threshold", &threshold),
        rate("top_bracket", &top),
        rate("lower_rate", &lower),
        rate("higher_rate", &higher),
    ];
    Ok(([first, second], rates))
}

/// Tasmania: the year's tax on the annualised wages, in the month's share.
fn tas(m: &Month) -> Result<Worked> {
    if !(m.iw.is_zero() && m.eiw.is_zero()) {
        return refuse(
            "payroll_tax_monthly.interstate_unworked",
            "A Tasmanian employer that pays interstate wages in the year is not worked: Payroll Tax Act 2008 (Tas) Sch 2 cl 3 annualises only the taxable wages, and says nothing of the interstate wages Sch 1 cl 4 shares the threshold by.",
        );
    }
    let year = int(365);
    let annualised = annualise(m, TAS_YEAR)?;
    let ([lower, higher], mut rates) = tas_rates(&annualised.value)?;
    rates.extend([rate("days_in_month", &m.days), rate("days_in_year", &year)]);
    let operands = [&lower.value, &higher.value];
    let sum = lower.value.add(&higher.value)?;
    let year_tax = step(
        "annual_payroll_tax",
        TAS_ANNUAL,
        "tax_at_lower_rate + tax_at_higher_rate",
        &operands,
        sum,
    );
    let payroll = month_share(m, &year_tax, TAS_MONTH)?;
    Ok(Worked {
        steps: vec![annualised, lower, higher, year_tax, payroll],
        charges: 1,
        rates,
    })
}

const NSW_YEAR: &str = "Payroll Tax Act 2007 (NSW) Sch 2 cl 3(b) (AustLII consolidation as at 22 October 2026, read on 27 November 2026): the year's taxable wages are taken to be the month's multiplied by the days in the financial year (365 in 2026-27) over the days the return relates to, and cl 3(a): wages are taken to be paid all year";
const NSW_ABOVE: &str = "Payroll Tax Act 2007 (NSW) Sch 1 cll 4 and 5 with cl 1 (TA: $1,200,000 for the financial year commencing on 1 July 2020 and subsequent financial years), as at 22 October 2026, and Revenue NSW's 'Payroll tax thresholds and rates' (last updated 1 July 2026: $1,200,000 for 1 July 2026 to 30 June 2027; 'The payroll tax rate is applied to the wages amount above the threshold'), both read on 27 November 2026: the annualised wages above the threshold, nil at or below it. The cl 5 formula is published as an image, which could not be read; with no interstate wages Revenue NSW's page gives the amount taxed";
const NSW_ANNUAL: &str = "Payroll Tax Act 2007 (NSW) Sch 1 cl 1 (R: 5.45% on and from 1 July 2022) and Revenue NSW's 'Payroll tax thresholds and rates' (5.45% for 1 July 2026 to 30 June 2027), both read on 27 November 2026: the payroll tax for the whole year on the annualised wages";
const NSW_MONTH: &str = "Payroll Tax Act 2007 (NSW) Sch 2 cl 2: the month's payroll tax is the year's in the ratio of the days the return relates to over the days in the financial year. This is the tax on the month's actual wages: an eligible taxpayer that opts under Sch 2 cl 3B to pay estimated monthly payroll tax pays the cl 3D amount instead, which is not worked";

/// New South Wales: the year's tax on the annualised wages, in the month's
/// share (Sch 2 cll 2 and 3).
fn nsw(m: &Month) -> Result<Worked> {
    if !(m.iw.is_zero() && m.eiw.is_zero()) {
        return refuse(
            "payroll_tax_monthly.interstate_unworked",
            "A New South Wales employer that pays interstate wages in the year is not worked: Payroll Tax Act 2007 (NSW) Sch 2 cl 3 annualises only the taxable wages, and says nothing of the interstate wages Sch 1 cll 4 and 5 share the threshold by.",
        );
    }
    let (year, threshold, applied) = (int(365), d("1200000"), d("0.0545"));
    let annualised = annualise(m, NSW_YEAR)?;
    let annual = &annualised.value;
    let above = step(
        "taxable_amount",
        NSW_ABOVE,
        "max(0, annualised_wages - threshold)",
        &[annual, &threshold],
        py_max(int(0), annual.sub(&threshold)?)?,
    );
    let year_tax = step(
        "annual_payroll_tax",
        NSW_ANNUAL,
        "taxable_amount x rate",
        &[&above.value, &applied],
        above.value.mul(&applied)?,
    );
    let payroll = month_share(m, &year_tax, NSW_MONTH)?;
    let rates = vec![
        rate("threshold", &threshold),
        rate("rate", &applied),
        rate("days_in_month", &m.days),
        rate("days_in_year", &year),
    ];
    Ok(Worked {
        steps: vec![annualised, above, year_tax, payroll],
        charges: 1,
        rates,
    })
}

const MONTH_TOTAL: &str =
    "The payroll tax and any surcharge or levy, each to the cent, payable for the month";

pub fn payroll_tax_monthly(args: &PayrollTaxMonthlyArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = place(args.jurisdiction.text())?;
    let (number, days) = month_days(args.month.text())?;
    refuse_unsupported(args, place, number)?;
    let m = facts(args, place, days)?;
    let worked = precise(|| {
        let worked = match place {
            "nsw" => nsw(&m),
            "vic" => vic(&m),
            "qld" => qld(&m),
            "wa" => wa(&m),
            "sa" => sa(&m),
            "tas" => tas(&m),
            "act" => act(&m),
            _ => nt(&m),
        }?;
        totalled(worked, MONTH_TOTAL)
    })?;
    derived_result(KIND, &year, worked.steps, worked.rates)
}
