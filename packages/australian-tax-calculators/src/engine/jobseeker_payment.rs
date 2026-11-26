// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `jobseeker_payment`: one fortnight's jobseeker payment under Benefit Rate
//! Calculator B (Social Security Act 1991 s 1068, compilation 234 of 20
//! September 2026, read on 26 November 2026): the maximum payment rate
//! (Module B maximum basic rate, Module C energy supplement and, where stated,
//! Module D pharmaceutical allowance) less the income reduction of Module G,
//! the partner income reduction (points 1068-G9 to G11) plus the ordinary
//! income reduction (points 1068-G12 to G17). Rent assistance, the working
//! credit (Part 3.1A), the assets test, waiting periods, compensation,
//! deductions and remote area allowance are outside the worksheet. The
//! indexed rates are the Department of Social Services' Social Security
//! Guide 5.1.8.20 and 5.1.10.20 and Services Australia's 'A guide to
//! Australian Government payments' for 1 July to 19 September 2026 and 20
//! September to 31 December 2026, read on 26 November 2026, for each day from
//! 1 July 2026 to 19 March 2027.

use serde_json::{Value, json};

use super::stated;
use super::{
    Arg, Result, Step, derived_result, flag, int, money, precise, rate, refuse, scope, step,
};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "jobseeker_payment";

const FACTS: [(&str, &str); 5] = [
    (
        "qualified",
        "the person qualifies for jobseeker payment, is aged 22 or over and is paid under Benefit Rate Calculator B, not at the rate for a single principal carer exempt from mutual obligation requirements (point 1068-B5)",
    ),
    (
        "no_working_credit",
        "no working credit balance reduces the person's employment income (Part 3.1A): this worksheet leaves the working credit out",
    ),
    (
        "no_rent_assistance",
        "no rent assistance is added (step 3 of point 1068-A1; see rent-assistance)",
    ),
    (
        "assets_within_limit",
        "the assets test does not make the payment nil, and no waiting period, preclusion or compensation affects it",
    ),
    (
        "no_adjustments",
        "no pension supplement (Module BA), special employment advance or advance payment deduction, or remote area allowance (step 7 of point 1068-A1) applies",
    ),
];

/// Each family situation: (name, rate column, single principal carer).
/// Columns: 0 single without a dependent child, 1 single at the higher
/// Table B rate, 2 partnered.
const SITUATIONS: [(&str, usize, bool); 6] = [
    ("single", 0, false),
    ("single_with_child", 1, false),
    ("single_principal_carer", 1, true),
    ("single_55_after_9_months", 1, false),
    ("single_partial_capacity", 1, false),
    ("partnered", 2, false),
];

/// The maximum basic rates per fortnight by column, each from its date to the
/// next, the last to [`UNTIL`].
const RATES: [(&str, [&str; 3]); 2] = [
    ("2026-07-01", ["808.70", "866.00", "740.30"]),
    ("2026-09-20", ["824.90", "883.30", "755.10"]),
];
const UNTIL: &str = "2027-03-19";
/// The energy supplement per year by column (SS Guide 5.1.10.20), not indexed.
const ENERGY: [&str; 3] = ["228.80", "247.00", "205.40"];
/// Pharmaceutical allowance per fortnight, single then partnered, indexed on
/// 1 January, so read only for days to [`PHARMACEUTICAL_UNTIL`].
const PHARMACEUTICAL: [&str; 2] = ["7.00", "3.50"];
const PHARMACEUTICAL_UNTIL: &str = "2026-12-31";

/// The facts as the command passes them; see the README for each.
pub struct JobseekerPaymentArgs<'a> {
    pub date: &'a Arg,
    pub situation: &'a Arg,
    pub pharmaceutical_allowance: &'a Arg,
    pub ordinary_income: &'a Arg,
    pub partner: &'a Arg,
    pub partner_income: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn period(given: &Arg) -> Result<&'static [&'static str; 3]> {
    match super::dated(given, &RATES, UNTIL) {
        Some(rates) => Ok(rates),
        None => refuse(
            "jobseeker_payment.date_uncovered",
            "Not computed: date must be a YYYY-MM-DD day from 1 July 2026 to 19 March 2027, the days the indexed rates were read for.",
        ),
    }
}

fn facts(given: &Arg) -> Result<()> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "jobseeker_payment.facts_invalid",
            "facts must be an object of booleans.",
        );
    };
    match stated::unstated(fields, &FACTS, FACTS.len())? {
        None => Ok(()),
        Some(missing) => refuse(
            "jobseeker_payment.facts_not_stated",
            format!(
                "Not computed: the worksheet covers only the stated case; not stated: {missing}."
            ),
        ),
    }
}

/// How the partner's income counts: none (single), over a partner income
/// free area (partner aged 22 or over and receiving no payment), or halved
/// with the person's (partner receiving a pension, point 1068-G2).
#[derive(Clone, Copy, PartialEq)]
enum Partner {
    None,
    NotReceiving,
    Pension,
}

fn partner(given: &Arg, partnered: bool, income: &Decimal) -> Result<Partner> {
    let status = match given.text() {
        Some("none") => Partner::None,
        Some("not_receiving") => Partner::NotReceiving,
        Some("pension") => Partner::Pension,
        _ => {
            return refuse(
                "jobseeker_payment.partner_invalid",
                "partner must be none, not_receiving (aged 22 or over, receiving no social security payment) or pension; a partner receiving a benefit or under 22 is not worked here (point 1068-G9(a) and (c)).",
            );
        }
    };
    if partnered == (status == Partner::None) || (status == Partner::None && !income.is_zero()) {
        return refuse(
            "jobseeker_payment.partner_inconsistent",
            "partner must be none, with partner_income 0.00, for a single situation, and not_receiving or pension for partnered.",
        );
    }
    Ok(status)
}

pub fn jobseeker_payment(args: &JobseekerPaymentArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let rates = period(args.date)?;
    let Some(&(name, column, carer)) = SITUATIONS
        .iter()
        .find(|s| Some(s.0) == args.situation.text())
    else {
        return refuse(
            "jobseeker_payment.situation_invalid",
            "situation must be single, single_with_child, single_principal_carer, single_55_after_9_months, single_partial_capacity or partnered.",
        );
    };
    let pharmaceutical = flag(args.pharmaceutical_allowance, "pharmaceutical_allowance")?;
    if pharmaceutical && args.date.text() > Some(PHARMACEUTICAL_UNTIL) {
        return refuse(
            "jobseeker_payment.pharmaceutical_uncovered",
            "Not computed: pharmaceutical allowance is indexed on 1 January and its rate was read for days to 31 December 2026 only.",
        );
    }
    let own = money(args.ordinary_income)?;
    let partner_income = money(args.partner_income)?;
    let status = partner(args.partner, column == 2, &partner_income)?;
    facts(args.facts)?;
    let d = super::d;
    let r = Rates {
        mbr: d(rates[column]),
        energy: d(ENERGY[column]),
        pharmaceutical: if pharmaceutical {
            d(PHARMACEUTICAL[usize::from(column == 2)])
        } else {
            int(0)
        },
        partner_mbr: d(rates[2]),
        partner_energy: d(ENERGY[2]),
    };
    let steps = precise(|| working(&r, carer, status, [&own, &partner_income]))?;
    let mut out = derived_result(KIND, &year, steps, r.shown())?;
    out["date"] = json!(args.date.text());
    out["situation"] = json!(name);
    out["rate_basis"] = json!(
        "per fortnight; the rate is a daily rate of the fortnightly rate / 14 (point 1068-A1)"
    );
    Ok(out)
}

struct Rates {
    mbr: Decimal,
    energy: Decimal,
    pharmaceutical: Decimal,
    partner_mbr: Decimal,
    partner_energy: Decimal,
}

impl Rates {
    fn shown(&self) -> Vec<(String, String)> {
        vec![
            rate("maximum_basic_rate", &self.mbr),
            rate("energy_supplement_per_year", &self.energy),
            rate("pharmaceutical_allowance", &self.pharmaceutical),
            rate("ordinary_income_free_area", "150"),
        ]
    }
}

const AUTH: [&str; 6] = [
    "SSA s 1068-A1 steps 1 to 4, Module B Table B, Module C (energy supplement, SS Guide 5.1.10.20, per year / 26) and Module D (pharmaceutical allowance): the maximum payment rate",
    "SSA s 1068-G2: the ordinary income of a person whose partner receives a pension is half the couple's combined ordinary income",
    "SSA s 1068-G9(b) and G9B: the partner income free area is the partner's income, rounded up to the nearest dollar, at which a partnered jobseeker payment (maximum basic rate and energy supplement) would reach nil",
    "SSA s 1068-G10 and G11: 60% of the partner's ordinary income over the partner income free area",
    "SSA s 1068-G12 to G16: 50% of ordinary income over $150 up to $106 of it, and 60% of the rest",
    "SSA s 1068-A1 steps 5 and 6, s 1068-G1 step 11: the maximum payment rate less the partner and ordinary income reductions, not below nil",
];
const CARER_AUTH: &str = "SSA s 1068-G12 and G17: a single principal carer's reduction is 40% of ordinary income over $150";

/// The ordinary income reduction outside point 1068-G17.
fn tapered(income: &Decimal) -> Result<Decimal> {
    let lower = super::py_min(super::py_max(income.sub(&int(150))?, int(0))?, int(106))?;
    let upper = super::py_max(income.sub(&int(256))?, int(0))?;
    Ok(lower
        .mul(&super::d("0.5"))?
        .add(&upper.mul(&super::d("0.6"))?)?)
}

/// The partner income free area (point 1068-G9(b)): the income at which a
/// partnered rate, always above the $53 reduction at $256, reaches nil,
/// rounded up to the nearest dollar.
fn free_area(r: &Rates) -> Result<Decimal> {
    let rate = r.partner_mbr.add(&r.partner_energy.div(&int(26))?)?;
    let cut_out = int(256).add(&rate.sub(&int(53))?.div(&super::d("0.6"))?)?;
    let down = cut_out.quantize(0, Rounding::Down)?;
    Ok(if down.lt(&cut_out)? {
        down.add(&int(1))?
    } else {
        down
    })
}

fn working(
    r: &Rates,
    carer: bool,
    status: Partner,
    [own, partner_income]: [&Decimal; 2],
) -> Result<Vec<Step>> {
    let max_rate = r
        .mbr
        .add(&r.energy.div(&int(26))?)?
        .add(&r.pharmaceutical)?;
    let mut steps = vec![step(
        "max_rate",
        AUTH[0],
        "maximum_basic_rate + energy_supplement_per_year / 26 + pharmaceutical_allowance",
        &[&r.mbr, &r.energy, &r.pharmaceutical],
        max_rate.clone(),
    )];
    let mut income = own.clone();
    let mut partner_reduction = int(0);
    match status {
        Partner::None => {}
        Partner::Pension => {
            income = own.add(partner_income)?.div(&int(2))?;
            steps.push(step(
                "ordinary_income",
                AUTH[1],
                "(own_income + partner_income) / 2",
                &[own, partner_income],
                income.clone(),
            ));
        }
        Partner::NotReceiving => {
            partner_reduction = partner_test(&mut steps, r, partner_income)?;
        }
    }
    let reduction = income_reduction(&mut steps, carer, &income)?;
    let total = reduction.add(&partner_reduction)?;
    steps.push(step(
        "rate",
        AUTH[5],
        "max(max_rate - (partner_income_reduction + ordinary_income_reduction), 0)",
        &[&max_rate, &partner_reduction, &reduction],
        super::py_max(max_rate.sub(&total)?, int(0))?,
    ));
    Ok(steps)
}

/// Pushes the ordinary income reduction step (points 1068-G14 to G17);
/// returns the reduction.
fn income_reduction(steps: &mut Vec<Step>, carer: bool, income: &Decimal) -> Result<Decimal> {
    let both = [income, income];
    let (reduction, authority, formula, inputs) = if carer {
        (
            super::py_max(income.sub(&int(150))?, int(0))?.mul(&super::d("0.4"))?,
            CARER_AUTH,
            "max(ordinary_income - 150, 0) x 0.4",
            &both[..1],
        )
    } else {
        (
            tapered(income)?,
            AUTH[4],
            "min(max(ordinary_income - 150, 0), 106) x 0.5 + max(ordinary_income - 256, 0) x 0.6",
            &both[..],
        )
    };
    steps.push(step(
        "ordinary_income_reduction",
        authority,
        formula,
        inputs,
        reduction.clone(),
    ));
    Ok(reduction)
}

/// Pushes the partner income free area and reduction steps; returns the
/// reduction.
fn partner_test(steps: &mut Vec<Step>, r: &Rates, partner_income: &Decimal) -> Result<Decimal> {
    let area = free_area(r)?;
    let reduction = super::py_max(partner_income.sub(&area)?, int(0))?.mul(&super::d("0.6"))?;
    steps.push(step(
        "partner_income_free_area",
        AUTH[2],
        "dollars_up(256 + (partnered_maximum_basic_rate + partnered_energy_per_year / 26 - 53) / 0.6)",
        &[&r.partner_mbr, &r.partner_energy],
        area.clone(),
    ));
    steps.push(step(
        "partner_income_reduction",
        AUTH[3],
        "max(partner_income - partner_income_free_area, 0) x 0.6",
        &[partner_income, &area],
        reduction.clone(),
    ));
    Ok(reduction)
}
