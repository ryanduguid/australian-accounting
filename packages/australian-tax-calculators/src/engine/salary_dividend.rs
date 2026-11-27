// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `salary_dividend`: one company owner's salary against fully franked
//! dividends for one income year, as the site's salary and dividends page
//! works it (`assets/salary-dividend.mjs`). The company pays the salary and its
//! super guarantee, company tax on what is left, and the rest as a fully
//! franked dividend. The owner is a full-year resident individual, single,
//! with private patient hospital cover, no study loan and no other
//! concessional contributions. Read on 26 November 2026: ITAA 1997 ss 207-20
//! (gross-up and tax offset), 202-60 (the maximum franking credit), 293-20
//! (Division 293 taxable contributions) and 25-130 (the standard deduction) as
//! the ATO's legal database gives them; SGAA 1992 ss 17A(2) (charge percentage
//! 12) and 10A(5) (the annual maximum contributions base from 1 July 2026);
//! Income Tax Rates Act 1986 ss 23 (company rates) and 26(1)(a) (15% on a
//! complying fund's low tax component); the ATO's 'Super guarantee' page
//! (QC18123, last updated 17 April 2026), Tables 21, 23 and 24.

use super::individual_owed::{Facts, owed};
use super::{
    Arg, Result, d, division_293, flag, int, money, precise, py_max, py_min, rate, refuse, result,
    scope,
};
use crate::decimal::{Decimal, Rounding};
use crate::metadata as meta;
use serde_json::Value;

const KIND: &str = "salary_dividend";

/// The year's super guarantee rate and the base for a full year of earnings
/// paid evenly: 2026-27's annual base under SGAA s 10A(5) ($32,500 x 100 / 12,
/// rounded down to $10), earlier years 4 of the quarterly base (Table 24).
const SUPER_GUARANTEE: [(&str, (&str, &str)); 3] = [
    ("2024-25", ("0.115", "260280")),
    ("2025-26", ("0.12", "250000")),
    ("2026-27", ("0.12", "270830")),
];

/// Income Tax Rates Act 1986 s 23(2) and (3): 25% for a base rate entity, else
/// 30%; the franking credit uses the same rate, the owner's stated fact.
const BASE_RATE: &str = "0.25";
const FULL_RATE: &str = "0.30";
/// Rates Act s 26(1)(a) contributions tax and Division 293 tax, each 15%.
const SUPER_TAX: &str = "0.15";
/// ITAA 1997 s 25-130: the lesser of $1,000 and labour income, from 2026-27.
const STANDARD_DEDUCTION: &str = "1000";
const STANDARD_DEDUCTION_FROM: &str = "2026-27";
/// ITAA 1997 s 293-20(1).
const DIVISION_293_THRESHOLD: &str = "250000";
/// The page's offset slope changes, whole dollars: the low income tax
/// offset's (s 61-115(1)) and the earlier low and middle income tax offset's,
/// kept so the candidates are the page's; a point outside a year's offsets
/// only adds a candidate.
const OFFSET_STEPS: [i64; 7] = [37000, 37500, 45000, 48000, 66667, 90000, 126000];
/// The ranking's tie margin, as the page's.
const TIE: &str = "0.000001";

/// The facts as the command passes them.
pub struct SalaryDividendArgs<'a> {
    pub profit: &'a Arg,
    pub base_rate_entity: &'a Arg,
    pub salary: &'a Arg,
    pub other_taxable_income: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

struct Law {
    year: String,
    sg_rate: Decimal,
    base: Decimal,
    company_rate: Decimal,
    deduction: bool,
    /// The tax on the other income alone.
    other_tax: Decimal,
}

/// One split, in dollars and cents.
struct Split {
    salary: Decimal,
    super_guarantee: Decimal,
    company_tax: Decimal,
    dividend: Decimal,
    credit: Decimal,
    taxable: Decimal,
    owner_tax: Decimal,
    contributions_tax: Decimal,
    division_293: Decimal,
    cash: Decimal,
    super_kept: Decimal,
    kept: Decimal,
}

fn cents(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(-2, Rounding::HalfUp)?)
}

fn down(value: &Decimal, places: i128) -> Result<Decimal> {
    Ok(value.quantize(-places, Rounding::Down)?)
}

/// The year's tax and levy on whole-dollar taxable income, after the low
/// income tax offset: the page assumes hospital cover and no study loan.
fn tax_on(year: &str, taxable: &Decimal) -> Result<Decimal> {
    let facts = Facts {
        year,
        foreign: false,
        taxable: py_max(taxable.clone(), int(0))?,
        fringe: int(0),
        reported_super: int(0),
        study_loan: false,
        hospital_cover: true,
    };
    owed(&facts)?.total()
}

/// The super guarantee on a salary: the rate on the salary up to the base,
/// half up to the cent.
fn super_guarantee(salary: &Decimal, law: &Law) -> Result<Decimal> {
    cents(&py_min(salary.clone(), law.base.clone())?.mul(&law.sg_rate)?)
}

fn deduction(salary: &Decimal, law: &Law) -> Result<Decimal> {
    if law.deduction {
        Ok(py_min(d(STANDARD_DEDUCTION), salary.clone())?)
    } else {
        Ok(int(0))
    }
}

/// Division 293 tax on the super guarantee (s 293-20(1)), from `division-293`.
fn division_293_tax(taxable: &Decimal, contributions: &Decimal, year: &str) -> Result<Decimal> {
    let out = division_293(
        &Arg::Decimal(taxable.clone()),
        &Arg::Decimal(contributions.clone()),
        &Arg::Text(year.to_string()),
        &Arg::Bool(true),
    );
    let out = out?;
    let text = out["amounts"]["division_293_tax"]
        .as_str()
        .expect("an amount");
    Ok(Decimal::parse(text)?)
}

/// One split: the company tax half up to the cent on the profit left, the
/// franking credit at the company's rate rounded down (s 202-60), taxable
/// income in whole dollars (s 207-20(1)), and the owner's extra tax: the tax on
/// it less the credit (s 207-20(2)) and the tax on the other income alone.
fn split(salary: &Decimal, profit: &Decimal, other: &Decimal, law: &Law) -> Result<Split> {
    let super_guarantee = super_guarantee(salary, law)?;
    let left = profit.sub(salary)?.sub(&super_guarantee)?;
    let company_tax = cents(&left.mul(&law.company_rate)?)?;
    let dividend = left.sub(&company_tax)?;
    let kept = int(1).sub(&law.company_rate)?;
    let credit = down(&dividend.mul(&law.company_rate)?.div(&kept)?, 2)?;
    let income = other.add(salary)?.add(&dividend)?.add(&credit)?;
    let taxable = down(&income.sub(&deduction(salary, law)?)?, 0)?;
    let owner_tax = tax_on(&law.year, &taxable)?
        .sub(&credit)?
        .sub(&law.other_tax)?;
    let contributions_tax = cents(&super_guarantee.mul(&d(SUPER_TAX))?)?;
    let division_293 = division_293_tax(&taxable, &super_guarantee, &law.year)?;
    let cash = salary.add(&dividend)?.sub(&owner_tax)?;
    let super_kept = super_guarantee
        .sub(&contributions_tax)?
        .sub(&division_293)?;
    let kept = cash.add(&super_kept)?;
    Ok(Split {
        salary: salary.clone(),
        super_guarantee,
        company_tax,
        dividend,
        credit,
        taxable,
        owner_tax,
        contributions_tax,
        division_293,
        cash,
        super_kept,
        kept,
    })
}

/// The kept total with nothing rounded, to rank salaries: the credit equals
/// the company tax, and the tax is taken in a straight line between the whole
/// dollars either side of taxable income.
fn unrounded(salary: &Decimal, profit: &Decimal, other: &Decimal, law: &Law) -> Result<Decimal> {
    let sg = py_min(salary.clone(), law.base.clone())?.mul(&law.sg_rate)?;
    let taxable = other.add(profit)?.sub(&sg)?.sub(&deduction(salary, law)?)?;
    let whole = taxable.quantize(0, Rounding::Down)?;
    let whole = if whole.gt(&taxable)? {
        whole.sub(&int(1))?
    } else {
        whole
    };
    let low = tax_on(&law.year, &whole)?;
    let high = tax_on(&law.year, &whole.add(&int(1))?)?;
    let tax = low.add(&taxable.sub(&whole)?.mul(&high.sub(&low)?)?)?;
    let over = taxable.add(&sg)?.sub(&d(DIVISION_293_THRESHOLD))?;
    let excess = py_max(int(0), py_min(sg.clone(), over)?)?;
    let kept_super = sg
        .mul(&int(1).sub(&d(SUPER_TAX))?)?
        .sub(&excess.mul(&d(SUPER_TAX))?)?;
    Ok(profit
        .sub(&sg)?
        .sub(&tax)?
        .add(&law.other_tax)?
        .add(&kept_super)?)
}

/// The largest salary in cents whose super guarantee the profit also pays.
fn largest_salary(profit: &Decimal, law: &Law) -> Result<Decimal> {
    let cent = d("0.01");
    let fits = |s: &Decimal| -> Result<bool> { Ok(s.add(&super_guarantee(s, law)?)?.le(profit)?) };
    let mut salary = down(&profit.div(&int(1).add(&law.sg_rate)?)?, 2)?;
    if salary.gt(&law.base)? {
        let higher = py_max(
            law.base.clone(),
            profit.sub(&super_guarantee(&law.base, law)?)?,
        );
        salary = higher?;
    }
    // The guess fits: profit less the guess is whole cents and at least the
    // unrounded super guarantee, which `cents` rounds to whole cents; above the
    // base the super guarantee is the base's.
    while fits(&salary.add(&cent)?)? {
        salary = salary.add(&cent)?;
    }
    Ok(salary)
}

/// The first whole dollar of taxable income with tax left after the offset.
fn first_taxed(year: &str) -> Result<i64> {
    let (mut low, mut high) = (0, 200000);
    while low < high {
        let middle = (low + high) / 2;
        let facts = Facts {
            year,
            foreign: false,
            taxable: int(middle),
            fringe: int(0),
            reported_super: int(0),
            study_loan: false,
            hospital_cover: true,
        };
        if owed(&facts)?.net_tax()?.gt(&int(0))? {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    Ok(low)
}

/// Taxable incomes at which the tax changes slope, whole dollars.
fn slope_points(year: &str) -> Result<Vec<i64>> {
    let scale = meta::RESIDENT_TAX_SCALES.iter().find(|(y, _)| *y == year);
    let mut points: Vec<i64> = scale
        .expect("a covered year")
        .1
        .iter()
        .map(|b| b.0)
        .collect();
    points.extend(OFFSET_STEPS);
    let (_, (threshold, limit)) = meta::MEDICARE_LOW_INCOME_THRESHOLDS
        .iter()
        .find(|(y, _)| *y == year)
        .expect("a covered year");
    points.extend([
        threshold.parse::<i64>().expect("dollars"),
        limit.parse().expect("dollars"),
    ]);
    points.push(first_taxed(year)?);
    Ok(points)
}

/// The salaries at which the kept total can change slope, with the whole
/// dollar either side; between 2 neighbours it moves in a straight line.
fn candidates(
    profit: &Decimal,
    other: &Decimal,
    largest: &Decimal,
    law: &Law,
) -> Result<Vec<Decimal>> {
    let deduction = if law.deduction {
        d(STANDARD_DEDUCTION)
    } else {
        int(0)
    };
    let base = other.add(profit)?.sub(&deduction)?;
    let one_plus = int(1).add(&law.sg_rate)?;
    let mut salaries = vec![int(0), largest.clone(), law.base.clone(), deduction.clone()];
    salaries.push(base.sub(&d(DIVISION_293_THRESHOLD))?.div(&law.sg_rate)?);
    for point in slope_points(&law.year)? {
        salaries.push(base.sub(&int(point))?.div(&law.sg_rate)?);
        salaries.push(other.add(profit)?.sub(&int(point))?.div(&one_plus)?);
    }
    let mut found = vec![int(0), largest.clone()];
    for salary in salaries.iter().filter(|s| !s.lt(&int(0)).unwrap_or(true)) {
        let dollar = salary.quantize(0, Rounding::Down)?;
        for near in [dollar.sub(&int(1))?, dollar.clone(), dollar.add(&int(1))?] {
            let inside = !near.lt(&int(0))? && near.le(largest)?;
            if inside && !found.iter().any(|f| f.equals(&near).unwrap_or(false)) {
                found.push(near);
            }
        }
    }
    found.sort_by(|a, b| a.compare(b).expect("finite"));
    Ok(found)
}

/// The salary that keeps the most, a tie keeping the lower salary.
fn best_salary(profit: &Decimal, other: &Decimal, largest: &Decimal, law: &Law) -> Result<Decimal> {
    let mut top: Option<(Decimal, Decimal)> = None;
    for salary in candidates(profit, other, largest, law)? {
        let kept = unrounded(&salary, profit, other, law)?;
        let better = match &top {
            None => true,
            Some((_, best)) => kept.gt(&best.add(&d(TIE))?)?,
        };
        if better {
            top = Some((salary, kept));
        }
    }
    Ok(top.expect("at least no salary").0)
}

fn law(year: String, base_rate: bool, other: &Decimal) -> Result<Law> {
    let (_, (sg_rate, base)) = SUPER_GUARANTEE
        .iter()
        .find(|(y, _)| *y == year)
        .expect("a covered year");
    let other_tax = tax_on(&year, &down(other, 0)?)?;
    Ok(Law {
        deduction: year.as_str() >= STANDARD_DEDUCTION_FROM,
        year,
        sg_rate: d(sg_rate),
        base: d(base),
        company_rate: d(if base_rate { BASE_RATE } else { FULL_RATE }),
        other_tax,
    })
}

pub fn owner_pay_split(args: &SalaryDividendArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let base_rate = flag(args.base_rate_entity, "base_rate_entity")?;
    let profit = money(args.profit)?;
    let salary = money(args.salary)?;
    let other = money(args.other_taxable_income)?;
    precise(|| {
        let law = law(year.clone(), base_rate, &other)?;
        let largest = largest_salary(&profit, &law)?;
        if salary.gt(&largest)? {
            return refuse(
                "salary_dividend.salary_too_large",
                format!(
                    "salary can be no more than {largest}: the profit also pays its super guarantee."
                ),
            );
        }
        let chosen = split(&salary, &profit, &other, &law)?;
        let best = split(
            &best_salary(&profit, &other, &largest, &law)?,
            &profit,
            &other,
            &law,
        );
        let best = best?;
        let all_salary = split(&largest, &profit, &other, &law)?.kept;
        let all_dividends = split(&int(0), &profit, &other, &law)?.kept;
        let mut amounts = amounts(&chosen);
        amounts.extend([
            ("largest_salary", largest),
            ("all_salary_kept", all_salary),
            ("all_dividends_kept", all_dividends),
            ("best_salary", best.salary.clone()),
            ("best_cash", best.cash.clone()),
            ("best_super", best.super_kept.clone()),
            ("best_kept", best.kept),
        ]);
        let rates = vec![
            rate("super_guarantee_rate", law.sg_rate.to_string()),
            rate("maximum_contribution_base", law.base.to_string()),
            rate("company_tax_rate", law.company_rate.to_string()),
            rate("contributions_tax_rate", SUPER_TAX),
        ];
        result(KIND, &year, amounts, rates)
    })
}

fn amounts(s: &Split) -> Vec<(&'static str, Decimal)> {
    vec![
        ("salary", s.salary.clone()),
        ("super_guarantee", s.super_guarantee.clone()),
        ("company_tax", s.company_tax.clone()),
        ("dividend", s.dividend.clone()),
        ("franking_credit", s.credit.clone()),
        ("taxable_income", s.taxable.clone()),
        ("owner_tax", s.owner_tax.clone()),
        ("contributions_tax", s.contributions_tax.clone()),
        ("division_293_tax", s.division_293.clone()),
        ("cash_kept", s.cash.clone()),
        ("super_kept", s.super_kept.clone()),
        ("kept", s.kept.clone()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The largest salary fits and a cent more does not, for profits around
    /// each year's base and a spread of others.
    #[test]
    fn the_largest_salary_is_the_largest_that_fits() {
        let fits = |s: &Decimal, law: &Law, profit: &Decimal| {
            s.add(&super_guarantee(s, law).unwrap())
                .unwrap()
                .le(profit)
                .unwrap()
        };
        for (year, (rate, base)) in SUPER_GUARANTEE {
            let law = law(year.to_string(), false, &int(0)).unwrap();
            let edge = d(base).mul(&int(1).add(&d(rate)).unwrap()).unwrap();
            let edge = down(&edge, 2).unwrap();
            let mut profits: Vec<Decimal> = (-300..=300)
                .map(|c| {
                    edge.add(&d(&format!("{c}")).mul(&d("0.01")).unwrap())
                        .unwrap()
                })
                .collect();
            let mut seed: u64 = 7;
            for _ in 0..400 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                profits.push(d(&format!(
                    "{}.{:02}",
                    (seed >> 33) % 600_000,
                    (seed >> 20) % 100
                )));
            }
            profits.extend(["0", "0.01", "0.11", "1.12", "1000000"].map(d));
            for profit in profits {
                let salary = largest_salary(&profit, &law).unwrap();
                let more = salary.add(&d("0.01")).unwrap();
                assert!(fits(&salary, &law, &profit), "{year} {profit}");
                assert!(!fits(&more, &law, &profit), "{year} {profit}");
            }
        }
    }
}
