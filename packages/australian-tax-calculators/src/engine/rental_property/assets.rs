// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The decline in value of `rental_property`: each depreciating asset's
//! decline in value as given, at the owner's legal interest and the
//! time-based share, nil where ITAA 1997 s 40-27 reduces it (s 40-27(2) to
//! (6), applying as Act No. 126 of 2017 Schedule 2 item 13 says), all read on
//! 22 November 2026. The decline itself is worked out elsewhere, such as by
//! `depreciation`.

use serde_json::{Map, Value, json};

use super::super::{Arg, Result, cents, int, refuse};
use super::{Held, amount, at, conflict, date, item, item_flag, list};
use crate::dates;
use crate::decimal::Decimal;

const FACTS: [(&str, bool); 8] = [
    ("decline_in_value", true),
    ("acquired", true),
    ("first_used", true),
    ("held_when_first_used", true),
    ("residence_or_private_use", true),
    ("low_value_pool", true),
    ("new_premises_exception", false),
    ("deductible_in_2016_17", false),
];

fn required(fields: &Map<String, Value>, key: &str, when: &str) -> Result<bool> {
    match item_flag(fields, key)? {
        Some(b) => Ok(b),
        None => refuse(
            "input.missing_field",
            format!("{key} is required for an asset {when}"),
        ),
    }
}

/// Whether s 40-27 applies to the asset (Act No. 126 of 2017 Schedule 2 item
/// 13): acquired at or after 7.30 pm on 9 May 2017, or earlier with nothing
/// deductible for 2016-17 and a start time (`first_used`) in 2016-17 or before.
fn applies(fields: &Map<String, Value>, acquired: i64, first_used: i64) -> Result<bool> {
    let cutoff = dates::parse("2017-05-09").expect("a valid date");
    if acquired > cutoff {
        return Ok(true);
    }
    if acquired == cutoff {
        return refuse(
            "rental_property.asset_acquired_9_may_2017",
            "The asset was acquired on 9 May 2017, and s 40-27 applies only from 7.30 pm that day (Act No. 126 of 2017 Schedule 2 item 13(1)).",
        );
    }
    if required(
        fields,
        "deductible_in_2016_17",
        "acquired before 9 May 2017",
    )? {
        return Ok(false);
    }
    Ok(first_used <= dates::parse("2017-06-30").expect("a valid date"))
}

/// Why s 40-27 reduces the asset's decline to nil, or `None`.
fn reduced(
    fields: &Map<String, Value>,
    acquired: i64,
    first_used: i64,
) -> Result<Option<&'static str>> {
    if required(fields, "low_value_pool", "")? || !applies(fields, acquired, first_used)? {
        return Ok(None);
    }
    if required(fields, "residence_or_private_use", "")? {
        return Ok(Some(
            "used in one of your residences or for a non-taxable purpose not occasionally (s 40-27(2)(d))",
        ));
    }
    if required(fields, "held_when_first_used", "")? {
        return Ok(None);
    }
    let when = "you did not hold when it was first used or installed ready for use";
    if required(fields, "new_premises_exception", when)? {
        return Ok(None);
    }
    Ok(Some(
        "second-hand: you did not hold it when it was first used or installed ready for use by any entity, and it was not supplied with new residential premises (s 40-27(2)(c), (4) and (5))",
    ))
}

/// The days available and held the asset's decline is apportioned by: the
/// property's, or all of them for an asset in use for the rental only from
/// after the property was first held, in a year it was available every day
/// held. The start time can come before the asset was acquired (s 40-60(2)),
/// but then it was not held when first used, and its rental use starts when
/// both have happened.
fn days(
    fields: &Map<String, Value>,
    first_used: i64,
    acquired: i64,
    held: &Held,
) -> Result<(i64, i64)> {
    let from = first_used.max(acquired);
    if from > held.to {
        return conflict(
            "first_used and acquired must not be after the property was last held in the year.",
        );
    }
    if first_used < acquired && item_flag(fields, "held_when_first_used")? == Some(true) {
        return conflict(
            "An asset first used before it was acquired was not held when it was first used.",
        );
    }
    if from <= held.from || held.available == held.days {
        return Ok((held.available, held.days));
    }
    refuse(
        "rental_property.days_unclear",
        "The asset was first used or acquired after the property was first held in the year, and it was not available for rent on every day held, so the share of its decline for rental use is not known from days_available.",
    )
}

fn asset(value: &Value, share: &Decimal, held: &Held) -> Result<(String, Decimal, Value)> {
    let fields = item(value, "depreciating_assets", &FACTS)?;
    let decline = amount(fields.get("decline_in_value"))?;
    let acquired = date(fields, "acquired")?;
    let first_used = date(fields, "first_used")?;
    let (available, all) = days(fields, first_used, acquired, held)?;
    let why = reduced(fields, acquired, first_used)?;
    let (formula, deduction) = match why {
        Some(_) => (format!("0 x {decline}"), int(0)),
        None => (
            format!("cents({decline} x {share} x {available} / {all})"),
            cents(decline.mul(share)?.mul(&int(available))?.div(&int(all))?)?,
        ),
    };
    let row = json!({
        "decline_in_value": decline.to_string(),
        "acquired": dates::format(acquired),
        "deduction": deduction.to_string(),
        "section_40_27": why,
    });
    Ok((formula, deduction, row))
}

/// The decline in value deduction across the assets: its formula, its total
/// and a row for each asset.
pub(super) fn deduction(
    items: &Arg,
    share: &Decimal,
    held: &Held,
) -> Result<((String, Decimal), Vec<Value>)> {
    let mut terms = Vec::new();
    let mut rows = Vec::new();
    let mut total = int(0);
    for (i, value) in list(items, "depreciating_assets")?.iter().enumerate() {
        let (formula, amount, row) = at(&format!("asset {}", i + 1), asset(value, share, held))?;
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
