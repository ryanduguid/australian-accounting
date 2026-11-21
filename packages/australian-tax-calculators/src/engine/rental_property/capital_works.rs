// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The capital works deduction of `rental_property`: ITAA 1997 Division 43
//! for a residential rental, each item's rate by when it began (ss 43-20,
//! 43-25 and 43-220, and the ATO's Rental properties guide 2026 Tables 1 and
//! 2), the deduction for the days used (ss 43-210 and 43-215) and its limit,
//! the undeducted construction expenditure (ss 43-15(1), 43-230, 43-235 and
//! 43-240), all read on 22 November 2026. The construction expenditure is
//! given, never derived from a price or a valuation.

use serde_json::{Map, Value, json};

use super::super::{Result, cents, d, int, refuse};
use super::{Held, Year, amount, at, conflict, invalid, item, item_flag, list};
use crate::dates;
use crate::decimal::Decimal;

const FACTS: [(&str, bool); 5] = [
    ("construction_expenditure", true),
    ("kind", true),
    ("began", true),
    ("first_used", true),
    ("contract_before_16_september_1987", false),
];

/// Why an item has no deduction, or its rate.
enum Rate {
    Nil(&'static str),
    Of(&'static str),
}

fn day(text: &str) -> i64 {
    dates::parse(text).expect("a valid date")
}

/// The rate for a residential rental by when the works began.
fn rate(kind: &str, began: i64, fields: &Map<String, Value>) -> Result<Rate> {
    let structural = match kind {
        "building" => false,
        "structural_improvement" => true,
        _ => return invalid("kind must be building or structural_improvement."),
    };
    if structural {
        return Ok(if began < day("1992-02-27") {
            Rate::Nil(
                "a structural improvement begun before 27 February 1992 (ITAA 1997 s 43-20(2); Rental properties guide 2026, Table 1 and Example 29)",
            )
        } else {
            Rate::Of("0.025")
        });
    }
    if began < day("1985-07-18") {
        return Ok(Rate::Nil(
            "a building begun before 18 July 1985, before which only short-term traveller accommodation and non-residential buildings qualify (Rental properties guide 2026, Table 1)",
        ));
    }
    if began < day("1987-09-16") {
        return Ok(Rate::Of("0.04"));
    }
    if began < day("1992-02-27") {
        return match item_flag(fields, "contract_before_16_september_1987")? {
            Some(true) => Ok(Rate::Of("0.04")),
            Some(false) => Ok(Rate::Of("0.025")),
            None => refuse(
                "input.missing_field",
                "contract_before_16_september_1987 is required for a building begun from 16 September 1987 to 26 February 1992 (ITAA 1997 s 43-220(1)(a))",
            ),
        };
    }
    Ok(Rate::Of("0.025"))
}

/// The days the item was used to produce rent in the year.
fn days_used(first_used: i64, held: &Held) -> Result<i64> {
    if first_used > held.to {
        return Ok(0);
    }
    if first_used <= held.from {
        return Ok(held.available);
    }
    if held.available < held.days {
        return refuse(
            "rental_property.works_days_unclear",
            "The works were first used after the property was first held in the year, and it was not available for rent on every day held, so their days used are not known from days_available.",
        );
    }
    Ok(held.to - first_used + 1)
}

/// The days of the use period (s 43-230(2)): from first use to the end of the
/// previous income year, or to the day before an acquisition in the year.
fn use_days(first_used: i64, held: &Held, year: &Year) -> i64 {
    let end = if held.acquired_in_year {
        held.from - 1
    } else {
        year.start - 1
    };
    (end - first_used + 1).max(0)
}

/// The undeducted construction expenditure at the year's start and the
/// deduction for the days used, no more than it (ss 43-15(1), 43-210 and
/// 43-235).
fn limited(portion: &Decimal, rate: &Decimal, used: i64, spent: i64) -> Result<(Decimal, Decimal)> {
    let written = portion.mul(rate)?.mul(&int(spent))?.div(&int(365))?;
    let undeducted = if written.gt(portion)? {
        int(0)
    } else {
        portion.sub(&written)?
    };
    let full = portion.mul(rate)?.mul(&int(used))?.div(&int(365))?;
    let deduction = cents(if full.lt(&undeducted)? {
        full
    } else {
        undeducted.clone()
    })?;
    Ok((undeducted, deduction))
}

/// One item's deduction, its formula and its row.
fn work(
    value: &Value,
    share: &Decimal,
    held: &Held,
    year: &Year,
) -> Result<(String, Decimal, Value)> {
    let fields = item(value, "capital_works", &FACTS)?;
    let expenditure = amount(fields.get("construction_expenditure"))?;
    let date = |key: &str| match fields
        .get(key)
        .and_then(Value::as_str)
        .and_then(dates::parse)
    {
        Some(day) => Ok(day),
        None => invalid(format!("{key} must be a YYYY-MM-DD date.")),
    };
    let (began, first_used) = (date("began")?, date("first_used")?);
    if first_used < began {
        return conflict("first_used must not be before began.");
    }
    let kind = fields["kind"].as_str().unwrap_or_default();
    let rate = match rate(kind, began, fields)? {
        Rate::Nil(why) => {
            let row = json!({"kind": kind, "began": dates::format(began), "rate": null, "deduction": "0.00", "basis": why});
            return Ok(("0".into(), int(0), row));
        }
        Rate::Of(rate) => d(rate),
    };
    let used = days_used(first_used, held)?;
    let spent = use_days(first_used, held, year);
    let portion = expenditure.mul(share)?;
    let (undeducted, deduction) = limited(&portion, &rate, used, spent)?;
    let formula = format!(
        "cents(min({expenditure} x {share} x {rate} x {used} / 365, {} - {expenditure} x {share} x {rate} x {spent} / 365))",
        portion
    );
    let row = json!({
        "kind": kind,
        "began": dates::format(began),
        "rate": rate.to_string(),
        "use_period_days": spent,
        "undeducted_construction_expenditure": cents(undeducted)?.to_string(),
        "days_used": used,
        "deduction": deduction.to_string(),
    });
    Ok((formula, deduction, row))
}

/// The capital works deduction across the items: its formula, its total and
/// a row for each item.
pub(super) fn deduction(
    items: &super::super::Arg,
    share: &Decimal,
    held: &Held,
    year: &Year,
) -> Result<((String, Decimal), Vec<Value>)> {
    let mut terms = Vec::new();
    let mut rows = Vec::new();
    let mut total = int(0);
    for (i, value) in list(items, "capital_works")?.iter().enumerate() {
        let (formula, amount, row) = at(
            &format!("capital works {}", i + 1),
            work(value, share, held, year),
        )?;
        terms.push(formula);
        total = total.add(&amount)?;
        rows.push(row);
    }
    let formula = if terms.is_empty() {
        "0".into()
    } else {
        terms.join(" + ")
    };
    Ok(((formula, total), rows))
}
