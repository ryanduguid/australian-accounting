// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `salary_packaging`: one employee's salary packaging with an employer whose
//! benefits FBTAA s 57A exempts, for one income year, as the site's salary
//! packaging page works it (`assets/salary-packaging.mjs`): the grossed-up
//! benefits above the employer's cap are taxed (s 5B(1D) and (1E)), the
//! reportable fringe benefits amount is the benefits' taxable value over
//! 1 - the FBT rate (s 135Q), and the employee's tax is worked with and
//! without the packaging. Read on 26 November 2026 on the ATO's legal
//! database: FBTAA ss 5B(1D) to (1M), 57A, 135P and 135Q; ITAA 1997 s 25-130;
//! the ATO's 'Fringe benefits tax - rates and thresholds' (last updated 20 May
//! 2026: 47% for the FBT years ending 31 March 2023 to 2027) and Fringe
//! benefits tax - a guide for employers, chapter 6.
//!
//! The grossed-up amounts use the formulas of s 5B(1F) and (1G), not the
//! 2.0802 and 1.8868 the ATO publishes for them, as `fbt` does. Both give the
//! guide's example at 6.10 ($16,018, $23,208 and FBT of $1,986.22); the 4-place
//! rate puts $15,900.21 of Type 2 benefits a dollar over the $30,000 cap where
//! the formula ($30,000.40) does not.

use super::individual_owed::{Facts, Owed, owed};
use super::{
    Arg, Result, d, flag, int, money, precise, py_max, py_min, rate, refuse, result, scope,
    whole_dollars,
};
use crate::decimal::{Decimal, Rounding};
use serde_json::Value;

const KIND: &str = "salary_packaging";

const FBT_RATE: &str = "0.47";
const GST_RATE: &str = "0.10";
/// Section 5B(1E) steps 2 and 3, and step 4's meal entertainment limit.
const HOSPITAL_CAP: &str = "17000";
const CHARITY_CAP: &str = "30000";
const MEAL_LIMIT: &str = "5000";
/// Section 135Q(2): more than $2,000.
const REPORTING_THRESHOLD: &str = "2000";
const STANDARD_DEDUCTION: &str = "1000";
const STANDARD_DEDUCTION_FROM: &str = "2026-27";

/// The facts as the command passes them.
pub struct SalaryPackagingArgs<'a> {
    pub residency: &'a Arg,
    pub employer: &'a Arg,
    pub salary: &'a Arg,
    pub type_two_benefits: &'a Arg,
    pub type_one_benefits: &'a Arg,
    pub meal_entertainment: &'a Arg,
    pub meal_entertainment_type_one: &'a Arg,
    pub study_loan: &'a Arg,
    pub hospital_cover: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The employee's circumstances the tax is worked from.
pub(super) struct Employee {
    pub year: String,
    pub foreign: bool,
    pub study_loan: bool,
    pub hospital_cover: bool,
}

pub(super) const RESIDENCY: &str = "residency must be resident or foreign_resident.";

/// `residency`: `resident` or `foreign_resident`; `None` for anything else.
pub(super) fn foreign(value: &Arg) -> Option<bool> {
    match value.text() {
        Some("resident") => Some(false),
        Some("foreign_resident") => Some(true),
        _ => None,
    }
}

/// The year's tax, levies and repayment on a salary of `paid` with a
/// reportable fringe benefits amount of `fringe`, after the standard deduction
/// a resident has from 2026-27 (s 25-130), all of the salary being labour
/// income: (taxable income, the parts).
pub(super) fn tax_on_pay(
    e: &Employee,
    paid: &Decimal,
    fringe: &Decimal,
) -> Result<(Decimal, Owed)> {
    let deduction = if !e.foreign && e.year.as_str() >= STANDARD_DEDUCTION_FROM {
        py_min(d(STANDARD_DEDUCTION), paid.clone())?
    } else {
        int(0)
    };
    let taxable = paid.sub(&deduction)?.quantize(0, Rounding::Down)?;
    let facts = Facts {
        year: &e.year,
        foreign: e.foreign,
        taxable: taxable.clone(),
        fringe: fringe.clone(),
        reported_super: int(0),
        study_loan: e.study_loan,
        hospital_cover: e.hospital_cover,
    };
    Ok((taxable, owed(&facts)?))
}

/// Section 135Q(4): the taxable value over 1 - the FBT rate, cents dropped as
/// the ATO's examples drop them, when it is more than $2,000 (s 135Q(2)).
pub(super) fn reportable(value: &Decimal) -> Result<Decimal> {
    if value.gt(&d(REPORTING_THRESHOLD))? {
        let grossed = value.div(&int(1).sub(&d(FBT_RATE))?)?;
        Ok(grossed.quantize(0, Rounding::Down)?)
    } else {
        Ok(int(0))
    }
}

/// The s 5B(1F) and (1G) gross-ups, half up to the dollar as the ATO's guide
/// rounds each grossed-up amount.
fn grossed(value: &Decimal, type_one: bool) -> Result<Decimal> {
    let (fbt, gst) = (d(FBT_RATE), d(GST_RATE));
    let kept = int(1).sub(&fbt)?;
    let factor = if type_one {
        fbt.add(&gst)?
            .div(&kept.mul(&int(1).add(&gst)?)?.mul(&fbt)?)?
    } else {
        int(1).div(&kept)?
    };
    Ok(value.mul(&factor)?.quantize(0, Rounding::HalfUp)?)
}

fn cap(employer: &Arg) -> Result<Decimal> {
    match employer.text() {
        Some("charity") => Ok(d(CHARITY_CAP)),
        Some("hospital") => Ok(d(HOSPITAL_CAP)),
        _ => refuse(
            "salary_packaging.employer_invalid",
            "employer must be charity (a public benevolent institution or health promotion charity, s 5B(1E) step 3) or hospital (a public or not-for-profit hospital or public ambulance service, step 2).",
        ),
    }
}

/// The facts read and checked.
struct Package {
    employee: Employee,
    cap: Decimal,
    meals_type_one: bool,
    salary: Decimal,
    two: Decimal,
    one: Decimal,
    meals: Decimal,
}

fn package(args: &SalaryPackagingArgs) -> Result<Package> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let employee = Employee {
        year,
        foreign: foreign(args.residency).map_or_else(
            || refuse("salary_packaging.residency_invalid", RESIDENCY),
            Ok,
        )?,
        study_loan: flag(args.study_loan, "study_loan")?,
        hospital_cover: flag(args.hospital_cover, "hospital_cover")?,
    };
    let cap = cap(args.employer)?;
    let meals_type_one = flag(
        args.meal_entertainment_type_one,
        "meal_entertainment_type_one",
    )?;
    let salary = money(args.salary)?;
    whole_dollars(&salary, "salary")?;
    Ok(Package {
        employee,
        cap,
        meals_type_one,
        salary,
        two: money(args.type_two_benefits)?,
        one: money(args.type_one_benefits)?,
        meals: money(args.meal_entertainment)?,
    })
}

/// The grossed-up amounts against the cap and the FBT on the excess.
struct Grossed {
    one: Decimal,
    two: Decimal,
    meals: Decimal,
    total: Decimal,
    allowance: Decimal,
    excess: Decimal,
    fbt: Decimal,
}

fn gross(p: &Package) -> Result<Grossed> {
    let (one_base, two_base) = if p.meals_type_one {
        (p.one.add(&p.meals)?, p.two.clone())
    } else {
        (p.one.clone(), p.two.add(&p.meals)?)
    };
    let one = grossed(&one_base, true)?;
    let two = grossed(&two_base, false)?;
    let meals = grossed(&p.meals, p.meals_type_one)?;
    let total = one.add(&two)?;
    let over_cap = py_max(int(0), total.sub(&p.cap)?)?;
    let allowance = py_min(py_min(d(MEAL_LIMIT), meals.clone())?, over_cap.clone())?;
    let excess = over_cap.sub(&allowance)?;
    let fbt = excess.mul(&d(FBT_RATE))?;
    Ok(Grossed {
        one,
        two,
        meals,
        total,
        allowance,
        excess,
        fbt,
    })
}

fn rates() -> Vec<(String, String)> {
    vec![
        rate("fbt_rate", FBT_RATE),
        rate("gst_rate", GST_RATE),
        rate("meal_entertainment_limit", MEAL_LIMIT),
    ]
}

pub fn packaged_benefits(args: &SalaryPackagingArgs) -> Result<Value> {
    let p = package(args)?;
    precise(|| {
        let packaged = p.two.add(&p.one)?.add(&p.meals)?;
        if packaged.gt(&p.salary)? {
            return refuse(
                "salary_packaging.packaged_exceeds_salary",
                "The benefits packaged are more than the salary.",
            );
        }
        let g = gross(&p)?;
        let reportable = reportable(&packaged)?;
        let e = &p.employee;
        let (taxable_before, before) = tax_on_pay(e, &p.salary, &int(0))?;
        let (taxable_after, parts) = tax_on_pay(e, &p.salary.sub(&packaged)?, &reportable)?;
        let (before, after) = (before.total()?, parts.total()?);
        let saved = before.sub(&after)?;
        let room = py_max(int(0), p.cap.sub(&g.total)?)?;
        // Type 2 benefits whose grossed-up value the room takes: x (1 - rate).
        let room_two = room
            .mul(&int(1).sub(&d(FBT_RATE))?)?
            .quantize(-2, Rounding::Down)?;
        let amounts = vec![
            ("cap", p.cap.clone()),
            ("packaged", packaged),
            ("grossed_type_one", g.one),
            ("grossed_type_two", g.two),
            ("grossed_meal_entertainment", g.meals),
            ("grossed_total", g.total),
            ("meal_entertainment_allowance", g.allowance),
            ("excess", g.excess),
            ("fbt", g.fbt.clone()),
            ("reportable_fringe_benefits", reportable),
            ("taxable_income_before", taxable_before),
            ("taxable_income_after", taxable_after),
            ("owed_before", before),
            ("owed_after", after),
            ("medicare_levy_surcharge_after", parts.surcharge),
            ("study_loan_repayment_after", parts.loan),
            ("tax_saved", saved.clone()),
            ("gain", saved.sub(&g.fbt)?),
            ("cap_room", room),
            ("cap_room_type_two", room_two),
        ];
        result(KIND, &e.year, amounts, rates())
    })
}
