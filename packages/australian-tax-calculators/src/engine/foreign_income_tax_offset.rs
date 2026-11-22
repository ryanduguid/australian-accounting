// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `foreign_income_tax_offset`: the foreign income tax offset under ITAA 1997
//! Division 770 for 2020-21 to 2026-27, with its limit by the $1,000 default,
//! from income tax payable figures the operator states, or worked out for a
//! resident individual at the year's resident rates, with the part-year
//! tax-free threshold of the Income Tax Rates Act 1986 ss 18 and 20. Sections
//! 770-70 to 770-80 read the same in ITAA 1997 compilations 210 (1 July 2020)
//! and 268 (1 October 2026), and ss 18 and 20 in Rates Act compilations 55
//! (1 July 2020) and 66 (1 July 2026), all read on 22 November 2026.

use serde_json::Value;

use super::{
    Arg, Result, Step, d, derived_result, money, precise, py_max, py_min, rate, refuse, scope, step,
};
use crate::decimal::Decimal;

mod worked;

const KIND: &str = "foreign_income_tax_offset";

/// The facts only the stated or worked limit takes, in
/// [`ForeignIncomeTaxOffsetArgs::optional`].
pub const FITO_OPTIONAL: [&str; 9] = [
    "tax_payable",
    "tax_payable_assumed",
    "taxable_income",
    "disregarded_income",
    "disregarded_deductions",
    "medicare_levy",
    "medicare_levy_assumed",
    "residency_periods",
    "eligible_pensioner",
];

/// Which optional facts each limit method requires (`true`), allows (`false`
/// in the list) or refuses (absent).
const STATED: [(&str, bool); 2] = [("tax_payable", true), ("tax_payable_assumed", true)];
const WORKED: [(&str, bool); 7] = [
    ("taxable_income", true),
    ("disregarded_income", true),
    ("disregarded_deductions", true),
    ("medicare_levy", true),
    ("medicare_levy_assumed", true),
    ("residency_periods", false),
    ("eligible_pensioner", false),
];

/// The facts as the command passes them; see the README for each.
pub struct ForeignIncomeTaxOffsetArgs<'a> {
    pub foreign_tax_paid: &'a Arg,
    pub nane_foreign_tax: &'a Arg,
    pub limit_method: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 9],
}

impl<'a> ForeignIncomeTaxOffsetArgs<'a> {
    /// The optional fact `key`, which the method's checks have made present.
    fn get(&self, key: &str) -> Option<&'a Arg> {
        let i = FITO_OPTIONAL
            .iter()
            .position(|k| *k == key)
            .expect("an optional fact");
        self.optional[i]
    }
}

const OFFSET_BEFORE_LIMIT: (&str, &str, &str) = (
    "offset_before_limit",
    "ITAA 1997 s 770-70: the sum of the foreign income tax that counts towards the offset for the year, paid on assessable income (s 770-10(1)) or on s 23AI or 23AK non-assessable non-exempt income (s 770-10(2))",
    "foreign_tax_paid + nane_foreign_tax",
);
const DEFAULT_LIMIT: (&str, &str, &str) = (
    "offset_limit",
    "ITAA 1997 s 770-75(2)(a) and note 1: $1,000, without working out paragraph (b), raised by the s 770-10(2) tax (s 770-80)",
    "1000 + nane_foreign_tax",
);
const COMPUTED_LIMIT: (&str, &str, &str) = (
    "offset_limit",
    "ITAA 1997 s 770-75(2): the greater of $1,000 and the income tax payable less the income tax payable on the s 770-75(4) assumptions, raised by the s 770-10(2) tax (s 770-80)",
    "max(1000, tax_payable - tax_payable_assumed) + nane_foreign_tax",
);
const OFFSET: (&str, &str, &str) = (
    "offset",
    "ITAA 1997 s 770-75(1): the offset reduced by any excess over the offset limit",
    "min(offset_before_limit, offset_limit)",
);
const ABOVE_LIMIT: (&str, &str, &str) = (
    "offset_above_limit",
    "ITAA 1997 s 770-75(1): the foreign income tax above the offset limit, which is lost: no refund and no carry forward (s 63-10(1) item 22)",
    "offset_before_limit - offset",
);
const STATED_STEPS: [(&str, &str, &str); 2] = [
    (
        "tax_payable",
        "ITAA 1997 s 770-75(2)(b)(i) and (3): income tax payable for the year disregarding tax offsets, with the Medicare levy and surcharge and without penalties or interest (ATO Guide to foreign income tax offset rules 2026, step 1), as stated",
        "tax_payable",
    ),
    (
        "tax_payable_assumed",
        "ITAA 1997 s 770-75(2)(b)(ii), (3) and (4): the same on the assumptions that assessable income leaves out amounts on which counted foreign income tax was paid and other foreign-source income, and that the debt deductions of an overseas permanent establishment and other deductions reasonably related to those amounts are not allowed (the guide's step 2), as stated",
        "tax_payable_assumed",
    ),
];

pub fn foreign_income_tax_offset(args: &ForeignIncomeTaxOffsetArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let paid = money(args.foreign_tax_paid)?;
    let nane = money(args.nane_foreign_tax)?;
    let method = match args.limit_method.text() {
        Some(m @ ("default" | "stated" | "worked")) => m,
        _ => {
            return refuse(
                "foreign_income_tax_offset.limit_method_invalid",
                "limit_method must be default, stated or worked.",
            );
        }
    };
    check_facts(args, method)?;
    let (mut steps, rates) = match method {
        "default" => (Vec::new(), Vec::new()),
        "stated" => (stated(args)?, Vec::new()),
        _ => worked::steps(args, &year)?,
    };
    let limit_rule = if method == "default" {
        DEFAULT_LIMIT
    } else {
        COMPUTED_LIMIT
    };
    let payable = |id: &str| steps.iter().find(|s| s.id == id).map(|s| s.value.clone());
    let limit_inputs = (payable("tax_payable"), payable("tax_payable_assumed"));
    steps.extend(offset_steps(&paid, &nane, limit_rule, limit_inputs)?);
    let mut rates = rates;
    rates.push(rate("offset_limit_minimum", "1000"));
    derived_result(KIND, &year, steps, rates)
}

/// Refuses an optional fact the method does not take, and requires the ones
/// it does.
fn check_facts(args: &ForeignIncomeTaxOffsetArgs, method: &str) -> Result<()> {
    let table: &[(&str, bool)] = match method {
        "stated" => &STATED,
        "worked" => &WORKED,
        _ => &[],
    };
    for (key, given) in FITO_OPTIONAL.iter().zip(args.optional) {
        match (table.iter().find(|(k, _)| k == key), given) {
            (None, Some(_)) => {
                return refuse(
                    "input.field_unknown",
                    format!("{key} is not an input of the {method} limit method"),
                );
            }
            (Some((_, true)), None) => {
                return refuse(
                    "input.missing_field",
                    format!("{key} is required for the {method} limit method."),
                );
            }
            _ => {}
        }
    }
    if args.get("eligible_pensioner").is_some() && args.get("residency_periods").is_none() {
        return refuse(
            "input.field_unknown",
            "eligible_pensioner is an input only with residency_periods",
        );
    }
    Ok(())
}

fn stated(args: &ForeignIncomeTaxOffsetArgs) -> Result<Vec<Step>> {
    let actual = money(args.get("tax_payable").expect("checked"))?;
    let assumed = money(args.get("tax_payable_assumed").expect("checked"))?;
    Ok(STATED_STEPS
        .iter()
        .zip([actual, assumed])
        .map(|((id, authority, formula), value)| {
            step(*id, authority, formula, &[&value], value.clone())
        })
        .collect())
}

/// The offset, its limit, the offset after the limit and the part lost.
fn offset_steps(
    paid: &Decimal,
    nane: &Decimal,
    limit_rule: (&'static str, &'static str, &'static str),
    payable: (Option<Decimal>, Option<Decimal>),
) -> Result<Vec<Step>> {
    precise(|| {
        let before = paid.add(nane)?;
        let thousand = d("1000");
        let (limit, limit_operands) = match &payable {
            (Some(actual), Some(assumed)) => (
                py_max(thousand, actual.sub(assumed)?)?.add(nane)?,
                vec![actual, assumed, nane],
            ),
            _ => (thousand.add(nane)?, vec![nane]),
        };
        let offset = py_min(before.clone(), limit.clone())?;
        let above = before.sub(&offset)?;
        let (id, authority, formula) = limit_rule;
        Ok(vec![
            step(
                OFFSET_BEFORE_LIMIT.0,
                OFFSET_BEFORE_LIMIT.1,
                OFFSET_BEFORE_LIMIT.2,
                &[paid, nane],
                before.clone(),
            ),
            step(id, authority, formula, &limit_operands, limit.clone()),
            step(
                OFFSET.0,
                OFFSET.1,
                OFFSET.2,
                &[&before, &limit],
                offset.clone(),
            ),
            step(
                ABOVE_LIMIT.0,
                ABOVE_LIMIT.1,
                ABOVE_LIMIT.2,
                &[&before, &offset],
                above,
            ),
        ])
    })
}
