// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `ftb_part_b`: a family's annual rate of family tax benefit Part B for
//! 2025-26 or 2026-27 under A New Tax System (Family Assistance) Act 1999
//! Schedule 1 Part 4 (compilation 124, 1 July 2026, read on 26 November 2026):
//! nil above the income limit on the higher income (clause 28B and Schedule 3
//! clause 3(2)(a)) or for a couple whose youngest FTB child has turned 13
//! (clause 28D); otherwise the standard rate by the youngest child's age
//! (clause 30), the FTB Part B supplement (clause 31A) and, where the family
//! has kept it, the energy supplement (clauses 29AA and 31B), which a couple
//! reduces by 20% of the lower income over the income free area (clauses 29(2),
//! 32 and 33). The indexed amounts are the Department of Social Services'
//! rates list for 1 July 2026 (its previous amounts for 2025-26), read on 26
//! November 2026.

use serde_json::{Value, json};

use super::stated;
use super::{
    Arg, Result, Step, derived_result, flag, int, money, precise, rate, refuse, scope, step,
};
use crate::decimal::Decimal;

const KIND: &str = "ftb_part_b";

/// The facts every case needs, then the one a youngest child of 16 or over
/// needs.
const FACTS: [(&str, &str); 3] = [
    (
        "whole_year",
        "each FTB child was the family's FTB child for every day of the year and in its care all the time (no shared care percentage, clauses 31, 31A and 31C)",
    ),
    (
        "no_parental_leave_pay",
        "no parental leave pay was payable to the individual or partner for any day of the year (clause 28C)",
    ),
    (
        "general_method",
        "the Part B rate is worked out under clause 29, not clause 29A (a secondary earner's return to paid work)",
    ),
];
const SENIOR_FACT: [(&str, &str); 1] = [(
    "senior_secondary",
    "the youngest FTB child, aged 16 or over, is a senior secondary school child (clause 29(3))",
)];

/// One year's indexed annual amounts: the standard rates (youngest under 5,
/// 5 or over), the FTB Part B supplement, the income limit (clause 28B) and
/// the income free area (clause 33).
struct Year {
    year: &'static str,
    standard: [&'static str; 2],
    supplement: &'static str,
    limit: &'static str,
    free_area: &'static str,
}

/// The energy supplement (Part B), clause 31B, not indexed.
const ENERGY: [&str; 2] = ["73.00", "51.10"];

const YEARS: [Year; 2] = [
    Year {
        year: "2025-26",
        standard: ["5040.65", "3518.60"],
        supplement: "459.90",
        limit: "120007",
        free_area: "6935",
    },
    Year {
        year: "2026-27",
        standard: ["5223.15", "3646.35"],
        supplement: "478.15",
        limit: "124327",
        free_area: "7154",
    },
];

/// The facts as the command passes them; see the README for each.
pub struct FtbPartBArgs<'a> {
    pub relationship: &'a Arg,
    pub youngest_child_age: &'a Arg,
    pub adjusted_taxable_income: &'a Arg,
    pub partner_adjusted_taxable_income: &'a Arg,
    pub income_support: &'a Arg,
    pub grandparent: &'a Arg,
    pub energy_supplement: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn couple(given: &Arg) -> Result<bool> {
    match given.text() {
        Some("single") => Ok(false),
        Some("couple") => Ok(true),
        _ => refuse(
            "ftb_part_b.relationship_invalid",
            "relationship must be single or couple.",
        ),
    }
}

fn age(given: &Arg) -> Result<i64> {
    match given.int() {
        Some(Some(n)) if (0..=18).contains(&n) => Ok(n),
        _ => refuse(
            "ftb_part_b.age_invalid",
            "youngest_child_age must be a whole number from 0 to 18: an FTB child for Part B is under 16, or 16 to 18 in senior secondary school (clause 29(3)).",
        ),
    }
}

fn facts(given: &Arg, senior: bool) -> Result<()> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "ftb_part_b.facts_invalid",
            "facts must be an object of booleans.",
        );
    };
    let all: Vec<_> = FACTS.iter().chain(&SENIOR_FACT).copied().collect();
    match stated::unstated(fields, &all, FACTS.len() + usize::from(senior))? {
        None => Ok(()),
        Some(missing) => refuse(
            "ftb_part_b.facts_not_stated",
            format!(
                "Not computed: the worksheet covers only the stated case; not stated: {missing}."
            ),
        ),
    }
}

struct Rates {
    standard: Decimal,
    supplement: Decimal,
    energy: Decimal,
    limit: Decimal,
    free_area: Decimal,
}

impl Rates {
    /// The year's amounts for the youngest FTB child's age.
    fn of(year: &str, youngest: i64, energy: bool) -> Self {
        let y = YEARS
            .iter()
            .find(|y| y.year == year)
            .expect("a supported year");
        let band = usize::from(youngest >= 5);
        let d = super::d;
        Rates {
            standard: d(y.standard[band]),
            supplement: d(y.supplement),
            energy: if energy { d(ENERGY[band]) } else { int(0) },
            limit: d(y.limit),
            free_area: d(y.free_area),
        }
    }

    fn shown(&self) -> Vec<(String, String)> {
        vec![
            rate("standard_rate", &self.standard),
            rate("supplement", &self.supplement),
            rate("energy_supplement", &self.energy),
            rate("income_limit", &self.limit),
            rate("income_free_area", &self.free_area),
        ]
    }
}

/// Why the rate is nil before any taper, if it is.
#[derive(Clone, Copy, PartialEq)]
enum Nil {
    No,
    Income,
    Teenager,
}

pub fn ftb_part_b(args: &FtbPartBArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let couple = couple(args.relationship)?;
    let youngest = age(args.youngest_child_age)?;
    let own = money(args.adjusted_taxable_income)?;
    let partner = money(args.partner_adjusted_taxable_income)?;
    if !couple && !partner.is_zero() {
        return refuse(
            "ftb_part_b.partner_income_single",
            "partner_adjusted_taxable_income must be 0.00 for a single individual.",
        );
    }
    let support = flag(args.income_support, "income_support")?;
    let grandparent = flag(args.grandparent, "grandparent")?;
    let energy = flag(args.energy_supplement, "energy_supplement")?;
    facts(args.facts, youngest >= 16)?;
    let r = Rates::of(&year, youngest, energy);
    let higher = super::py_max(own.clone(), partner.clone())?;
    let nil = if couple && youngest >= 13 && !grandparent {
        Nil::Teenager
    } else if !support && higher.gt(&r.limit)? {
        Nil::Income
    } else {
        Nil::No
    };
    let steps = precise(|| working(couple, nil, &r, [&own, &partner]))?;
    let mut out = derived_result(KIND, &year, steps, r.shown())?;
    out["relationship"] = json!(if couple { "couple" } else { "single" });
    out["nil_because"] = match nil {
        Nil::No => Value::Null,
        Nil::Income => json!(
            "the higher adjusted taxable income is more than the income limit and neither member receives an income support payment (clause 28B; Schedule 3 clause 3(2)(a))"
        ),
        Nil::Teenager => json!(
            "a member of a couple whose youngest FTB child has turned 13, not that child's grandparent or great-grandparent (clause 28D)"
        ),
    };
    Ok(out)
}

const AUTH: [&str; 5] = [
    "FA Act Sch 1 cl 29(1) and (2) step 1, cl 30, cl 31A and cl 31B: the standard rate for the youngest FTB child's age, the FTB Part B supplement and any energy supplement (Part B)",
    "FA Act Sch 1 cl 29(1): a single individual's Part B rate is the maximum rate, with no income reduction",
    "FA Act Sch 1 cl 29(2) steps 2 and 3, cl 32 and cl 33; Sch 3 cl 3(2)(b): the maximum rate less 20% of the lower adjusted taxable income over the income free area, not below nil",
    "FA Act Sch 1 cl 28B and cl 28D: the Part B rate is nil",
    "FA Act Sch 1 cl 29(2) step 2: the lower of the couple's adjusted taxable incomes",
];

fn working(couple: bool, nil: Nil, r: &Rates, [own, partner]: [&Decimal; 2]) -> Result<Vec<Step>> {
    let maximum = r.standard.add(&r.supplement)?.add(&r.energy)?;
    let mut steps = vec![step(
        "maximum_rate",
        AUTH[0],
        "standard_rate + supplement + energy_supplement",
        &[&r.standard, &r.supplement, &r.energy],
        maximum.clone(),
    )];
    steps.push(if nil != Nil::No {
        step("part_b_rate", AUTH[3], "0", &[], int(0))
    } else if !couple {
        step(
            "part_b_rate",
            AUTH[1],
            "maximum_rate",
            &[&maximum],
            maximum.clone(),
        )
    } else {
        let lower = super::py_min(own.clone(), partner.clone())?;
        steps.push(step(
            "lower_income",
            AUTH[4],
            "min(adjusted_taxable_income, partner_adjusted_taxable_income)",
            &[own, partner],
            lower.clone(),
        ));
        let excess = super::py_max(lower.sub(&r.free_area)?, int(0))?;
        let tapered = super::py_max(maximum.sub(&excess.mul(&super::d("0.2"))?)?, int(0))?;
        step(
            "part_b_rate",
            AUTH[2],
            "max(maximum_rate - max(lower_income - income_free_area, 0) x 0.2, 0)",
            &[&maximum, &lower, &r.free_area],
            tapered,
        )
    });
    Ok(steps)
}
