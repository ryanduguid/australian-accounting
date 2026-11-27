// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The worked offset limit for a resident individual: basic income tax at the
//! year's resident rates on taxable income and on the s 770-75(4)
//! assumptions, each with the tax-free threshold for a part-year resident
//! (Income Tax Rates Act 1986 ss 18 and 20), plus the Medicare levy and
//! surcharge the operator states for each income.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use super::super::business_capital::year_first;
use super::super::{
    Arg, Result, Step, d, flag, int, lookup, money, precise, py_max, py_min, rate, refuse, stated,
    step, step_by_name, whole_dollars,
};
use super::ForeignIncomeTaxOffsetArgs;
use crate::dates;
use crate::decimal::Decimal;
use crate::metadata as meta;

/// The most residency periods the command takes.
const MAX_PERIODS: usize = 12;

const FULL_THRESHOLD: &str = "Income Tax Rates Act 1986 s 3(1): the tax-free threshold, $18,200, for a resident for the whole year";
const PENSIONER_THRESHOLD: &str = "Income Tax Rates Act 1986 s 18(2)(a): an eligible pensioner for the year (s 16) has no part-year residency period, so the tax-free threshold stays $18,200";
const PART_YEAR_THRESHOLD: &str = "Income Tax Rates Act 1986 s 20(1): (the tax-free threshold of $18,200 - $4,736) + $4,736 x the months of the year in which there is a part-year residency period (s 18(1): from the start of the month residency began, or of the year, to the end of the month it ended, or of the year) / 12; the other rate thresholds do not change";
/// A step's id, authority and formula.
type Rule = (&'static str, &'static str, &'static str);

const TAX_PAYABLE: Rule = (
    "tax_payable",
    "ITAA 1997 s 770-75(2)(b)(i) and (3): income tax payable disregarding tax offsets, with the Medicare levy and surcharge as stated and without penalties or interest (ATO Guide to foreign income tax offset rules 2026, step 1)",
    "basic_income_tax + medicare_levy",
);
const ASSUMED_INCOME: Rule = (
    "assumed_taxable_income",
    "ITAA 1997 s 770-75(4): taxable income without the amounts on which counted foreign income tax was paid and other foreign-source income, and without the debt deductions of an overseas permanent establishment and other deductions reasonably related to those amounts, and (the note to s 770-75(4)) without the foreign loss component of tax losses deducted, not below nil",
    "max(0, taxable_income - disregarded_income + disregarded_deductions)",
);
const TAX_PAYABLE_ASSUMED: Rule = (
    "tax_payable_assumed",
    "ITAA 1997 s 770-75(2)(b)(ii) and (3): income tax payable on the s 770-75(4) assumptions disregarding tax offsets, with the Medicare levy and surcharge as stated for that income (the guide's step 2)",
    "basic_income_tax_assumed + medicare_levy_assumed",
);
const BASIC_TAX: &str = "ITAA 1997 s 4-10(3) step 2; Income Tax Rates Act 1986 s 12(1) and Schedule 7 Part I: each part of the income above the tax-free threshold at the year's resident rate, as resident-tax applies them";

/// The worked method's figures: taxable income, the income and deductions
/// s 770-75(4) disregards, and the levy and surcharge on each income.
struct Facts {
    taxable: Decimal,
    income: Decimal,
    deductions: Decimal,
    levy: Decimal,
    levy_assumed: Decimal,
}

/// The year's rates, named as the steps use them.
type Rates = Vec<(String, String)>;

pub(super) fn steps(args: &ForeignIncomeTaxOffsetArgs, year: &str) -> Result<(Vec<Step>, Rates)> {
    if !meta::RESIDENT_TAX_SCALES.iter().any(|(y, _)| *y == year) {
        return refuse(
            "foreign_income_tax_offset.worked_year",
            format!(
                "The worked limit uses the resident rates, which resident-tax has for 2024-25 to 2026-27, not {year}; give tax_payable and tax_payable_assumed with the stated method."
            ),
        );
    }
    let get = |key| money(args.get(key).expect("checked"));
    let facts = Facts {
        taxable: get("taxable_income")?,
        income: get("disregarded_income")?,
        deductions: get("disregarded_deductions")?,
        levy: get("medicare_levy")?,
        levy_assumed: get("medicare_levy_assumed")?,
    };
    whole_dollars(&facts.taxable, "taxable income")?;
    let (threshold, months) = threshold(args, year)?;
    let scale = lookup(meta::RESIDENT_TAX_SCALES, year);
    let mut rates: Rates = scale
        .iter()
        .map(|(low, _, r)| rate(&format!("over_{low}"), r))
        .collect();
    rates.extend(months.map(|m| rate("residency_months", m)));
    let steps = precise(|| worked(scale, &facts, threshold))?;
    Ok((steps, rates))
}

/// The threshold, then steps 1 and 2 of the ATO's guide.
fn worked(scale: &Scale, facts: &Facts, threshold: Threshold) -> Result<Vec<Step>> {
    let ((rule, formula, operands), threshold) = threshold;
    let operands: Vec<&Decimal> = operands.iter().collect();
    let Facts {
        taxable,
        income,
        deductions,
        levy,
        levy_assumed,
    } = facts;
    let assumed = py_max(int(0), taxable.sub(income)?.add(deductions)?)?;
    let named = [
        ("taxable_income", taxable.clone()),
        ("disregarded_income", income.clone()),
        ("disregarded_deductions", deductions.clone()),
    ];
    let first = step(
        "tax_free_threshold",
        rule,
        formula,
        &operands,
        threshold.clone(),
    );
    let mut steps = vec![first];
    let ids = ("taxable_income", "basic_income_tax");
    steps.extend(payable(scale, taxable, &threshold, ids, levy, TAX_PAYABLE)?);
    let (id, authority, formula) = ASSUMED_INCOME;
    steps.push(step_by_name(
        id,
        authority,
        formula,
        &named,
        assumed.clone(),
    ));
    let ids = ("assumed_taxable_income", "basic_income_tax_assumed");
    let rule = TAX_PAYABLE_ASSUMED;
    let extended = payable(scale, &assumed, &threshold, ids, levy_assumed, rule);
    steps.extend(extended?);
    Ok(steps)
}

/// Basic income tax on `income`, then that tax plus `levy` under `rule`.
fn payable(
    scale: &Scale,
    income: &Decimal,
    threshold: &Decimal,
    (name, id): (&'static str, &'static str),
    levy: &Decimal,
    rule: Rule,
) -> Result<[Step; 2]> {
    let basic = basic_tax(scale, income, threshold, name, id)?;
    let tax = basic.value.clone();
    let total = step(rule.0, rule.1, rule.2, &[&tax, levy], tax.add(levy)?);
    Ok([basic, total])
}

type Scale = [(i64, Option<i64>, &'static str)];

/// The tax-free threshold with its rule, formula and operands.
type Threshold = ((&'static str, &'static str, Vec<Decimal>), Decimal);

/// The tax-free threshold, and the residency months a part-year threshold
/// is worked out from.
fn threshold(args: &ForeignIncomeTaxOffsetArgs, year: &str) -> Result<(Threshold, Option<i64>)> {
    let full = |rule| (((rule, "18200", Vec::new()), d("18200")), None);
    let Some(periods) = args.get("residency_periods") else {
        return Ok(full(FULL_THRESHOLD));
    };
    let months = months(periods, year)?;
    let pensioner = match args.get("eligible_pensioner") {
        Some(given) => flag(given, "eligible_pensioner")?,
        None => false,
    };
    if pensioner {
        return Ok(full(PENSIONER_THRESHOLD));
    }
    let count = int(months);
    let value = precise(|| Ok(d("13464").add(&d("4736").mul(&count)?.div(&int(12))?)?))?;
    let rule = (
        PART_YEAR_THRESHOLD,
        "13464 + 4736 x residency_months / 12",
        vec![count],
    );
    Ok(((rule, value), Some(months)))
}

/// The months of the income year in which there is a residency period, each
/// period counting from the start of its first month to the end of its last
/// (s 18(1)).
fn months(periods: &Arg, year: &str) -> Result<i64> {
    let invalid = |why: String| {
        refuse(
            "foreign_income_tax_offset.residency_periods_invalid",
            format!("residency_periods: {why}"),
        )
    };
    let Arg::Json(Value::Array(items)) = periods else {
        return invalid("must be a list of objects with from and to.".into());
    };
    if items.is_empty() || items.len() > MAX_PERIODS {
        return invalid(format!("must hold 1 to {MAX_PERIODS} periods."));
    }
    let first = year_first(year).expect("a supported year");
    let (start, end) = (
        day(&format!("{first}-07-01")),
        day(&format!("{}-06-30", first + 1)),
    );
    let index = |day: i64| {
        let (y, m, _) = dates::to_civil(day);
        y * 12 + m
    };
    let mut months = BTreeSet::new();
    for (i, item) in items.iter().enumerate() {
        let name = format!("period {}", i + 1);
        let Value::Object(fields) = item else {
            return invalid(format!("{name} must be an object with from and to."));
        };
        let (from, to) = period(fields, &name)?;
        if from > to || from < start || to > end {
            return invalid(format!(
                "{name} must run forwards inside the income year {year}."
            ));
        }
        if (index(from), index(to)) == (index(start), index(end)) {
            return invalid(format!(
                "{name} runs from the first month of the income year to the last, so under Income Tax Rates Act 1986 s 18(1) it is the whole year, which s 18(2)(b) says is not a part-year residency period; leave residency_periods out."
            ));
        }
        months.extend(index(from)..=index(to));
    }
    Ok(i64::try_from(months.len()).expect("at most 12"))
}

fn day(text: &str) -> i64 {
    dates::parse(text).expect("a valid date")
}

fn period(fields: &Map<String, Value>, name: &str) -> Result<(i64, i64)> {
    stated::known_keys(fields, name, "a residency period", &["from", "to"])?;
    let read = |key: &str| match stated::required(fields, name, key)? {
        Value::String(text) if dates::parse(text).is_some() => Ok(day(text)),
        _ => refuse(
            "input.date_invalid",
            format!("{name}: {key} must be a YYYY-MM-DD date from 1900 to 2100."),
        ),
    };
    Ok((read("from")?, read("to")?))
}

/// Basic income tax at the year's resident rates with `threshold` in place of
/// the $18,200 the first band starts at.
fn basic_tax(
    scale: &[(i64, Option<i64>, &'static str)],
    income: &Decimal,
    threshold: &Decimal,
    name: &'static str,
    id: &'static str,
) -> Result<Step> {
    let mut tax = int(0);
    let mut terms = Vec::new();
    for (i, (low, high, rate)) in scale.iter().enumerate() {
        let floor = if i == 0 { threshold.clone() } else { int(*low) };
        let floor_name = if i == 0 {
            "tax_free_threshold".to_owned()
        } else {
            low.to_string()
        };
        let (ceiling, top) = match high {
            None => (income.clone(), name.to_owned()),
            Some(high) => (
                py_min(income.clone(), int(*high))?,
                format!("min({name}, {high})"),
            ),
        };
        tax = tax.add(&py_max(int(0), ceiling.sub(&floor)?)?.mul(&d(rate))?)?;
        terms.push(format!("max(0, {top} - {floor_name}) x {rate}"));
    }
    let values = [
        (name, income.clone()),
        ("tax_free_threshold", threshold.clone()),
    ];
    Ok(step_by_name(
        id,
        BASIC_TAX,
        &terms.join(" + "),
        &values,
        tax,
    ))
}
