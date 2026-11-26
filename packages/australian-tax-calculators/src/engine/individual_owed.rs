// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! One single individual's income tax, Medicare levy, Medicare levy surcharge
//! and compulsory study loan repayment for a full income year, composed from
//! `resident-tax` (or `foreign-resident-tax`), `low-income-tax-offset`,
//! `medicare-levy`, `medicare-levy-surcharge` and `study-loan-repayment` as
//! `personal-tax-estimate` composes them: each part rendered to cents, the
//! low income tax offset applied against basic income tax only (ITAA 1997
//! s 63-10(1) item 20). Shared by the site mirrors `salary_dividend`,
//! `sole_trader_set_aside`, `salary_packaging` and `novated_lease`.

use super::{
    Arg, Result, foreign_resident_tax, income_year_days, int, low_income_tax_offset, medicare_levy,
    medicare_levy_surcharge, py_min, refuse, resident_tax, study_loan_repayment,
};
use super::{MedicareExemption, MedicareFamily, SurchargeFamily};
use crate::decimal::Decimal;
use serde_json::Value;

/// The facts: whole-dollar taxable income, reportable fringe benefits and
/// reportable super contributions; the person is single with no dependants.
pub(super) struct Facts<'a> {
    pub year: &'a str,
    pub foreign: bool,
    pub taxable: Decimal,
    pub fringe: Decimal,
    pub reported_super: Decimal,
    pub study_loan: bool,
    pub hospital_cover: bool,
}

/// The parts, each in cents as its worksheet renders it.
#[derive(Clone)]
pub(super) struct Owed {
    pub basic: Decimal,
    pub lito: Decimal,
    pub levy: Decimal,
    pub surcharge: Decimal,
    pub loan: Decimal,
}

impl Owed {
    /// Basic income tax less the low income tax offset.
    pub fn net_tax(&self) -> Result<Decimal> {
        Ok(self.basic.sub(&self.lito)?)
    }

    /// The year's total: net tax, levy, surcharge and repayment.
    pub fn total(&self) -> Result<Decimal> {
        let sum = self.net_tax()?.add(&self.levy)?.add(&self.surcharge)?;
        Ok(sum.add(&self.loan)?)
    }
}

pub(super) const FOREIGN_STUDY_LOAN: &str = "A foreign resident's study loan repayment includes the overseas levy on worldwide income, which study-loan-repayment does not work out.";

fn amount(out: &Value, key: &str) -> Result<Decimal> {
    let text = out["amounts"][key].as_str().expect("a rendered amount");
    Ok(Decimal::parse(text)?)
}

fn dec(value: &Decimal) -> Arg {
    Arg::Decimal(value.clone())
}

/// The year's parts for `facts`, each from its own worksheet.
pub(super) fn owed(facts: &Facts) -> Result<Owed> {
    let (year, yes) = (Arg::Text(facts.year.to_string()), Arg::Bool(true));
    let income = dec(&facts.taxable);
    if facts.foreign {
        if facts.study_loan {
            return refuse("estimate.residency_unsupported", FOREIGN_STUDY_LOAN);
        }
        let basic = amount(
            &foreign_resident_tax(&income, &year, &yes)?,
            "basic_income_tax",
        )?;
        let nil = int(0);
        return Ok(Owed {
            basic,
            lito: nil.clone(),
            levy: nil.clone(),
            surcharge: nil.clone(),
            loan: nil,
        });
    }
    let basic = amount(&resident_tax(&income, &year, &yes)?, "basic_income_tax")?;
    let offset = low_income_tax_offset(&income, &year, &yes)?;
    let lito = py_min(amount(&offset, "low_income_tax_offset")?, basic.clone())?;
    let levy = amount(&levy(&income, &year)?, "medicare_levy")?;
    let surcharge = amount(&surcharge(facts, &year)?, "medicare_levy_surcharge")?;
    let loan = if facts.study_loan {
        let parts = facts
            .taxable
            .add(&facts.fringe)?
            .add(&facts.reported_super)?;
        let out = study_loan_repayment(&dec(&parts), &year, &yes)?;
        amount(&out, "compulsory_repayment")?
    } else {
        int(0)
    };
    Ok(Owed {
        basic,
        lito,
        levy,
        surcharge,
        loan,
    })
}

fn levy(income: &Arg, year: &Arg) -> Result<Value> {
    let family = MedicareFamily {
        sapto_entitled: None,
        spouse_taxable_income: None,
        spouse_sapto_entitled: None,
        dependant_offset_entitled: None,
        dependent_children: None,
    };
    let exemption = MedicareExemption {
        full_days: None,
        half_days: None,
        non_resident_days: None,
    };
    let no = Arg::Bool(false);
    medicare_levy(income, &no, year, &Arg::Bool(true), family, exemption)
}

/// The surcharge for a whole year without, or with, private patient hospital
/// cover; income for surcharge purposes adds the reportable amounts.
fn surcharge(facts: &Facts, year: &Arg) -> Result<Value> {
    let days = if facts.hospital_cover {
        0
    } else {
        income_year_days(facts.year)
    };
    let own = facts
        .taxable
        .add(&facts.fringe)?
        .add(&facts.reported_super)?;
    let family = SurchargeFamily {
        spouse_income: None,
        dependent_children: None,
    };
    medicare_levy_surcharge(
        &dec(&facts.taxable),
        &dec(&facts.fringe),
        &dec(&own),
        &Arg::Int(days),
        year,
        &Arg::Bool(true),
        family,
    )
}
