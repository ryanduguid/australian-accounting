// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `foreign_pension_upp`: the deductible amount of the undeducted purchase
//! price of a foreign pension or annuity. `formula` works ITAA 1936 s 27H(2),
//! A x (B - C) / D, which reads the same in the compilation of 11 May 2007
//! (as amended for 1 July 2007) and on the ATO's legal database on
//! 24 November 2026; the life expectation factor is the operator's, read from
//! the Life Tables the Income Tax Assessment (1936 Act) Regulations 2025 s 7
//! prescribes. `uk_state_pension` and `dutch_svb_fallback` apply the ATO's
//! 2026 D11 instructions (8% of a UK category A or B widow's pension, nil for
//! category C or D; 25% of a Dutch SVB pension whose amount cannot be
//! determined), for 2025-26 only. The ATO itself decides a foreign pension's
//! amount by private ruling on NAT 16543.

use serde_json::Value;

use super::stated;
use super::{
    Arg, Result, Step, derived_result, flag, int, money, precise, rate, refuse, scope, step,
};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "foreign_pension_upp";

/// The years the ATO's 2026 instructions cover for the 2 shortcuts.
const SHORTCUT_YEARS: [&str; 1] = ["2025-26"];

pub struct ForeignPensionUppArgs<'a> {
    pub method: &'a Arg,
    pub annuity_derived: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    /// `undeducted_purchase_price`, `residual_capital_value`,
    /// `relevant_number`, `total_annuity_derived`, `commuted`, `whole_year`
    /// and `uk_category`, as given.
    pub optional: [Option<&'a Arg>; 7],
}

pub const FOREIGN_PENSION_UPP_OPTIONAL: [&str; 7] = [
    "undeducted_purchase_price",
    "residual_capital_value",
    "relevant_number",
    "total_annuity_derived",
    "commuted",
    "whole_year",
    "uk_category",
];

/// The inputs `method` needs: each present when needed and absent otherwise.
fn given<'a>(args: &'a ForeignPensionUppArgs, method: &str) -> Result<[Option<&'a Arg>; 7]> {
    let needs: [bool; 7] = match method {
        "formula" => [true, true, true, false, true, true, false],
        "uk_state_pension" => [false, false, false, false, false, false, true],
        _ => [false; 7],
    };
    for ((key, need), arg) in FOREIGN_PENSION_UPP_OPTIONAL
        .iter()
        .zip(needs)
        .zip(args.optional)
    {
        let optional_share = *key == "total_annuity_derived" && method == "formula";
        if need && arg.is_none() {
            return refuse(
                "input.missing_field",
                format!("{key} is required for method {method}"),
            );
        }
        if !need && !optional_share && arg.is_some() {
            return refuse(
                "input.field_unknown",
                format!("{key} is not an input of foreign-pension-upp with method {method}"),
            );
        }
    }
    Ok(args.optional)
}

/// s 27H(4) 'relevant number': a term of years certain or the life
/// expectation factor; the Commissioner's number for any other annuity is
/// refused.
fn relevant_number(value: &Arg) -> Result<(&'static str, Decimal)> {
    let bad = || {
        refuse(
            "foreign_pension_upp.relevant_number_invalid",
            "relevant_number must be {basis: term_years or life_expectation_factor, years: a decimal string above 0 and at most 150}. For an annuity neither for a term of years certain nor for one person's life, s 27H(4) makes the relevant number the one the Commissioner considers appropriate: request a determination (NAT 16543).",
        )
    };
    let Arg::Json(Value::Object(fields)) = value else {
        return bad();
    };
    if stated::known_keys(
        fields,
        "relevant_number",
        "relevant_number",
        &["basis", "years"],
    )
    .is_err()
    {
        return bad();
    }
    let basis = match fields.get("basis").and_then(Value::as_str) {
        Some("term_years") => "term_years",
        Some("life_expectation_factor") => "life_expectation_factor",
        _ => return bad(),
    };
    let years = match fields.get("years") {
        Some(Value::String(text)) if text.len() <= 32 => Decimal::parse_input(text).ok(),
        _ => None,
    };
    match years {
        Some(y) if y.is_finite() && y.gt(&int(0))? && y.le(&int(150))? => Ok((basis, y)),
        _ => bad(),
    }
}

/// s 27H(4) 'relevant share': whole dollars of the taxpayer's annuity over
/// whole dollars of the whole annuity, or 1.
fn relevant_share(own: &Decimal, total: Option<&Arg>) -> Result<Decimal> {
    let Some(total) = total else {
        return Ok(int(1));
    };
    let total = money(total)?;
    let own_dollars = own.quantize(0, Rounding::Down)?;
    let total_dollars = total.quantize(0, Rounding::Down)?;
    if total_dollars.lt(&own_dollars)? || total_dollars.is_zero() {
        return refuse(
            "foreign_pension_upp.share_invalid",
            "total_annuity_derived must be at least annuity_derived and at least $1: it is the whole annuity derived in the year by you and the others it is shared with (s 27H(4) 'relevant share').",
        );
    }
    Ok(own_dollars.div(&total_dollars)?)
}

/// The refused facts of the formula method: a commuted annuity and part of a
/// year.
fn formula_facts(commuted: &Arg, whole_year: &Arg) -> Result<()> {
    if flag(commuted, "commuted")? {
        return refuse(
            "foreign_pension_upp.commuted",
            "Part of the annuity has been commuted: s 27H(3A) reduces the deductible amount by what the Commissioner considers appropriate. Request a determination (NAT 16543).",
        );
    }
    if !flag(whole_year, "whole_year")? {
        return refuse(
            "foreign_pension_upp.part_year",
            "The annuity was not derived for the whole income year: s 27H(2) sets an amount for the year and gives no rule for apportioning it, and the ATO decides such cases on a request for a determination (NAT 16543).",
        );
    }
    Ok(())
}

/// s 27H(2): A x (B - C) / D.
fn formula_step(o: &[Option<&Arg>; 7], annuity: &Decimal) -> Result<(Step, Vec<(String, String)>)> {
    let [price, residual, number, total, commuted, whole_year, _] = o;
    formula_facts(commuted.expect("checked"), whole_year.expect("checked"))?;
    let price = money(price.expect("checked"))?;
    let residual = money(residual.expect("checked"))?;
    if residual.gt(&price)? {
        return refuse(
            "foreign_pension_upp.residual_exceeds_price",
            "residual_capital_value cannot exceed undeducted_purchase_price: B - C in s 27H(2) would be negative.",
        );
    }
    let (basis, years) = relevant_number(number.expect("checked"))?;
    let share = relevant_share(annuity, *total)?;
    let value = precise(|| Ok(share.mul(&price.sub(&residual)?)?.div(&years)?))?;
    let rates = vec![
        rate("relevant_share", &share),
        rate("relevant_number", &years),
        rate("relevant_number_basis", basis),
    ];
    let step = step(
        "deductible_amount",
        "ITAA 1936 s 27H(2): A x (B - C) / D, the relevant share times the undeducted purchase price less the residual capital value, over the relevant number",
        "relevant_share x (undeducted_purchase_price - residual_capital_value) / relevant_number",
        &[&share, &price, &residual, &years],
        value,
    );
    Ok((step, rates))
}

/// The ATO's UK State Pension shortcut: 8% for category A or a category B
/// widow's pension, nil for category C or D.
fn uk_step(category: &Arg, annuity: &Decimal) -> Result<(Step, Vec<(String, String)>)> {
    let rate = match category.text() {
        Some("a" | "b_widow") => "0.08",
        Some("c" | "d") => "0",
        _ => {
            return refuse(
                "foreign_pension_upp.uk_category_invalid",
                "uk_category must be a, b_widow, c or d: the UK State Pension category.",
            );
        }
    };
    let uk_rate = super::d(rate);
    let step = step(
        "deductible_amount",
        "ATO individual supplementary tax return instructions 2026, D11: a UK State Pension category A or category B widow's pension, 8% of the pension in AUD; category C or D, nil",
        "annuity_derived x uk_rate",
        &[annuity, &uk_rate],
        annuity.mul(&uk_rate)?,
    );
    Ok((step, vec![super::rate("uk_rate", &uk_rate)]))
}

/// The excluded amount, at most the annuity, and the assessable rest
/// (s 27H(1)(a)).
fn excluded_steps(deductible: &Decimal, annuity: &Decimal) -> Result<[Step; 2]> {
    let excluded = super::py_min(deductible.clone(), annuity.clone())?;
    let assessable = annuity.sub(&excluded)?;
    Ok([
        step(
            "excluded_amount",
            "ITAA 1936 s 27H(1)(a): the annuity is assessable excluding the deductible amount, which cannot exclude more than the annuity",
            "min(deductible_amount, annuity_derived)",
            &[deductible, annuity],
            excluded.clone(),
        ),
        step(
            "assessable_amount",
            "ITAA 1936 s 27H(1)(a): the annuity less the excluded amount",
            "annuity_derived - excluded_amount",
            &[annuity, &excluded],
            assessable,
        ),
    ])
}

fn method(value: &Arg) -> Result<&'static str> {
    match value.text() {
        Some("formula") => Ok("formula"),
        Some("uk_state_pension") => Ok("uk_state_pension"),
        Some("dutch_svb_fallback") => Ok("dutch_svb_fallback"),
        _ => refuse(
            "foreign_pension_upp.method_invalid",
            "method must be formula, uk_state_pension or dutch_svb_fallback.",
        ),
    }
}

/// The deductible amount's step and rates for `method`.
fn deductible(
    method: &str,
    o: &[Option<&Arg>; 7],
    annuity: &Decimal,
    year: &str,
) -> Result<(Step, Vec<(String, String)>)> {
    if method != "formula" && !SHORTCUT_YEARS.contains(&year) {
        return refuse(
            "foreign_pension_upp.method_year_unsupported",
            format!(
                "{method} follows the ATO's 2026 D11 instructions, read for 2025-26 only; use formula for {year}."
            ),
        );
    }
    match method {
        "formula" => formula_step(o, annuity),
        "uk_state_pension" => uk_step(o[6].expect("checked"), annuity),
        _ => {
            let quarter = super::d("0.25");
            let step = step(
                "deductible_amount",
                "ATO individual supplementary tax return instructions 2026, D11: a Dutch SVB pension whose deductible amount cannot be determined, 25% of the gross pension",
                "annuity_derived x 0.25",
                &[annuity],
                annuity.mul(&quarter)?,
            );
            Ok((step, Vec::new()))
        }
    }
}

pub fn foreign_pension_upp(args: &ForeignPensionUppArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let method = method(args.method)?;
    let annuity = money(args.annuity_derived)?;
    let o = given(args, method)?;
    let (first, mut rates) = precise(|| deductible(method, &o, &annuity, &year))?;
    let rest = precise(|| excluded_steps(&first.value, &annuity))?;
    let mut steps = vec![first];
    steps.extend(rest);
    rates.insert(0, rate("method", method));
    derived_result(KIND, &year, steps, rates)
}
