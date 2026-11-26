// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `ftb_part_a`: a family's annual rate of family tax benefit Part A for
//! 2025-26 or 2026-27 under the income test of A New Tax System (Family
//! Assistance) Act 1999 Schedule 1 (compilation 124, 1 July 2026, read on
//! 26 November 2026): Method 1 (clauses 1(2)(a), 3, 4, 7 and 38L to 38N) at an
//! adjusted taxable income up to the higher income free area, Method 2
//! (clauses 1(2)(b), 25, 25D, 26 and 28) above it, with the FTB Part A
//! supplement (clause 38A) and, where the family has kept it, the energy
//! supplement (clauses 6A, 25C, 38AA and 38AF). The indexed amounts are the
//! Department of Social Services' rates list for 1 July 2026 (its previous
//! amounts for 2025-26), read on 26 November 2026.

use serde_json::{Value, json};

use super::stated;
use super::{
    Arg, Result, Step, derived_result, flag, int, money, precise, rate, refuse, scope, step,
};
use crate::decimal::Decimal;

const KIND: &str = "ftb_part_a";

const FACTS: [(&str, &str); 5] = [
    (
        "whole_year",
        "each child counted was the family's FTB child for every day of the year and in its care all the time (no shared care percentage, clause 27)",
    ),
    (
        "no_maintenance",
        "neither parent received maintenance and each took reasonable maintenance action (clauses 10 and 20 to 24)",
    ),
    (
        "no_rent_assistance",
        "the family receives no rent assistance (clauses 38B to 38H)",
    ),
    (
        "no_newborn_or_multiple_birth",
        "no newborn supplement or multiple birth allowance is payable (clauses 35A to 38)",
    ),
    (
        "requirements_met",
        "every child met the immunisation and health check requirements and no FTB advance is unrepaid (ss 61A and 61B, clauses 5 and 25A)",
    ),
];

/// One year's indexed annual amounts: the FTB child rates (under 13, 13 or
/// over), the base FTB child rate, the supplement, the lower and higher
/// income free areas, and the energy supplements (under 13, 13 or over, base).
struct Year {
    year: &'static str,
    child: [&'static str; 2],
    base: &'static str,
    supplement: &'static str,
    free_areas: [&'static str; 2],
}

const ENERGY: [&str; 3] = ["91.25", "116.80", "36.50"];

const YEARS: [Year; 2] = [
    Year {
        year: "2025-26",
        child: ["5927.60", "7712.45"],
        base: "1901.65",
        supplement: "938.05",
        free_areas: ["66722", "118771"],
    },
    Year {
        year: "2026-27",
        child: ["6139.30", "7989.85"],
        base: "1971.00",
        supplement: "970.90",
        free_areas: ["69131", "123078"],
    },
];

/// The facts as the command passes them; see the README for each.
pub struct FtbPartAArgs<'a> {
    pub children_under_13: &'a Arg,
    pub children_13_plus: &'a Arg,
    pub adjusted_taxable_income: &'a Arg,
    pub income_support: &'a Arg,
    pub energy_supplement: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn count(given: &Arg, name: &str) -> Result<i64> {
    match given.int() {
        Some(Some(n)) if (0..=20).contains(&n) => Ok(n),
        _ => refuse(
            "ftb_part_a.children_invalid",
            format!("{name} must be a whole number from 0 to 20."),
        ),
    }
}

fn facts(given: &Arg) -> Result<()> {
    let missing = match given {
        Arg::Json(Value::Object(fields)) => stated::unstated(fields, &FACTS, FACTS.len())?,
        _ => {
            return refuse(
                "ftb_part_a.facts_invalid",
                "facts must be an object of booleans.",
            );
        }
    };
    if let Some(missing) = missing {
        return refuse(
            "ftb_part_a.facts_not_stated",
            format!(
                "Not computed: the worksheet covers only the stated case; not stated: {missing}."
            ),
        );
    }
    Ok(())
}

struct Rates {
    child: [Decimal; 2],
    energy: [Decimal; 3],
    base: Decimal,
    supplement: Decimal,
    lower: Decimal,
    higher: Decimal,
}

fn rates(year: &str, income: &Decimal, energy: bool) -> Result<Rates> {
    let y = YEARS
        .iter()
        .find(|y| y.year == year)
        .expect("a supported year");
    let d = super::d;
    let supplement = if income.gt(&int(80_000))? {
        int(0)
    } else {
        d(y.supplement)
    };
    Ok(Rates {
        child: y.child.map(d),
        energy: if energy {
            ENERGY.map(d)
        } else {
            [int(0), int(0), int(0)]
        },
        base: d(y.base),
        supplement,
        lower: d(y.free_areas[0]),
        higher: d(y.free_areas[1]),
    })
}

pub fn ftb_part_a(args: &FtbPartAArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let young = count(args.children_under_13, "children_under_13")?;
    let older = count(args.children_13_plus, "children_13_plus")?;
    if young + older == 0 || young + older > 20 {
        return refuse(
            "ftb_part_a.children_invalid",
            "Not computed: a family with no FTB child has a Part A rate only for rent assistance (Method 3, clause 28A).",
        );
    }
    let income = money(args.adjusted_taxable_income)?;
    let support = flag(args.income_support, "income_support")?;
    let energy = flag(args.energy_supplement, "energy_supplement")?;
    facts(args.facts)?;
    let r = rates(&year, &income, energy)?;
    let counts = [int(young), int(older)];
    let method = if support {
        Method::Support
    } else if income.gt(&r.higher)? {
        Method::Two
    } else {
        Method::One
    };
    let steps = precise(|| working(method, &r, &counts, &income))?;
    let mut out = derived_result(KIND, &year, steps, r.shown())?;
    out["method"] = json!(match method {
        Method::Support =>
            "Method 1, income excess nil for a pension or benefit recipient (clauses 1(2)(a)(ii) and 38L)",
        Method::One => "Method 1 (clauses 1(2)(a)(i) and 3)",
        Method::Two => "Method 2 (clauses 1(2)(b) and 25)",
    });
    out["supplement_included"] = json!(!income.gt(&int(80_000))?);
    Ok(out)
}

#[derive(Clone, Copy)]
enum Method {
    Support,
    One,
    Two,
}

impl Rates {
    fn shown(&self) -> Vec<(String, String)> {
        vec![
            rate("child_rate_under_13", &self.child[0]),
            rate("child_rate_13_plus", &self.child[1]),
            rate("energy_under_13", &self.energy[0]),
            rate("energy_13_plus", &self.energy[1]),
            rate("base_child_rate", &self.base),
            rate("base_energy", &self.energy[2]),
            rate("supplement_per_child", &self.supplement),
            rate("lower_income_free_area", &self.lower),
            rate("higher_income_free_area", &self.higher),
        ]
    }
}

const AUTH: [&str; 5] = [
    "FA Act Sch 1 cl 3 step 1, cl 7, cl 38A and cl 38AA: the FTB child rate for each child by age, the FTB Part A supplement for each child (nil above $80,000 adjusted taxable income) and any energy supplement",
    "FA Act Sch 1 cl 4, cl 25 step 1, cl 26, cl 38A and cl 38AF: the Method 2 base rate, the base FTB child rate for each child with the supplement and any energy supplement",
    "FA Act Sch 1 cl 3 steps 2 to 4, cl 38L to 38N: the maximum rate less 20% of adjusted taxable income over the lower income free area, not below the base rate; nil reduction for a pension or benefit recipient",
    "FA Act Sch 1 cl 25 steps 2 and 4, cl 28: the Method 2 base rate less 30% of adjusted taxable income over the higher income free area, not below nil",
    "FA Act Sch 1 cl 25 steps 3 and 4, cl 25D: the maximum rate less 20% of the band between the free areas and 30% of income over the higher one, not below nil; the Part A rate is the greater of the 2",
];

fn maximum_step(r: &Rates, counts: &[Decimal; 2], maximum: Decimal) -> Step {
    step(
        "maximum_rate",
        AUTH[0],
        "children_under_13 x (child_rate_under_13 + supplement_per_child + energy_under_13) + children_13_plus x (child_rate_13_plus + supplement_per_child + energy_13_plus)",
        &[
            &counts[0],
            &r.child[0],
            &r.supplement,
            &r.energy[0],
            &counts[1],
            &r.child[1],
            &r.supplement,
            &r.energy[1],
        ],
        maximum,
    )
}

fn working(
    method: Method,
    r: &Rates,
    counts: &[Decimal; 2],
    income: &Decimal,
) -> Result<Vec<Step>> {
    let each = |i: usize| r.child[i].add(&r.supplement)?.add(&r.energy[i]);
    let maximum = counts[0].mul(&each(0)?)?.add(&counts[1].mul(&each(1)?)?)?;
    let per_child = r.base.add(&r.energy[2])?.add(&r.supplement)?;
    let base = counts[0].add(&counts[1])?.mul(&per_child)?;
    let mut steps = vec![
        maximum_step(r, counts, maximum.clone()),
        step(
            "base_rate",
            AUTH[1],
            "(children_under_13 + children_13_plus) x (base_child_rate + base_energy + supplement_per_child)",
            &[&counts[0], &counts[1], &r.base, &r.energy[2], &r.supplement],
            base.clone(),
        ),
    ];
    match method {
        Method::Support => steps.push(step(
            "part_a_rate",
            AUTH[2],
            "maximum_rate",
            &[&maximum],
            maximum.clone(),
        )),
        Method::One => {
            let excess = super::py_max(income.sub(&r.lower)?, int(0))?;
            let tested = super::py_max(maximum.sub(&excess.mul(&super::d("0.2"))?)?, base.clone())?;
            steps.push(step(
                "part_a_rate",
                AUTH[2],
                "max(maximum_rate - max(adjusted_taxable_income - lower_income_free_area, 0) x 0.2, base_rate)",
                &[&maximum, income, &r.lower, &base],
                tested,
            ));
        }
        Method::Two => method_two(&mut steps, r, [&maximum, &base], income)?,
    }
    Ok(steps)
}

fn method_two(
    steps: &mut Vec<Step>,
    r: &Rates,
    [maximum, base]: [&Decimal; 2],
    income: &Decimal,
) -> Result<()> {
    let over = income.sub(&r.higher)?;
    let thirty = over.mul(&super::d("0.3"))?;
    let provisional = super::py_max(base.sub(&thirty)?, int(0))?;
    let band = r.higher.sub(&r.lower)?.mul(&super::d("0.2"))?;
    let tested = super::py_max(maximum.sub(&band)?.sub(&thirty)?, int(0))?;
    let part_a = super::py_max(provisional.clone(), tested.clone())?;
    let operands = [provisional.clone(), tested.clone()];
    steps.push(step(
        "provisional_rate",
        AUTH[3],
        "max(base_rate - (adjusted_taxable_income - higher_income_free_area) x 0.3, 0)",
        &[base, income, &r.higher],
        provisional,
    ));
    steps.push(step(
        "method_1_rate",
        AUTH[4],
        "max(maximum_rate - (higher_income_free_area - lower_income_free_area) x 0.2 - (adjusted_taxable_income - higher_income_free_area) x 0.3, 0)",
        &[maximum, &r.higher, &r.lower, income, &r.higher],
        tested,
    ));
    steps.push(step(
        "part_a_rate",
        AUTH[4],
        "max(provisional_rate, method_1_rate)",
        &[&operands[0], &operands[1]],
        part_a,
    ));
    Ok(())
}
