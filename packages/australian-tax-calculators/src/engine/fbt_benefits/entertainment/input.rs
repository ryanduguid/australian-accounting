// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! An entertainment expense line's stated facts, checked.

use serde_json::{Map, Value};

use super::tables::{
    CHARACTER, CLASSES, Character, Class, FOOD_TRAVEL, Premises, TRAVEL_NOT_ENTERTAINMENT,
};
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, money, refuse};

use Character::{Meal, NotEntertainment};

/// A line's stated facts, checked.
pub(super) struct Line {
    pub(super) cost: Decimal,
    pub(super) includes_gst: bool,
    pub(super) provided: &'static str,
    pub(super) character: Option<(Character, &'static str)>,
    pub(super) premises: Premises,
    pub(super) reimbursed: bool,
    /// Each class that attended and its head count, where stated.
    pub(super) attendees: Vec<(Class, Option<i64>)>,
    pub(super) infrequent: Option<bool>,
}

const LINE_FIELDS: [&str; 8] = [
    "cost",
    "gst_status",
    "provided",
    "occasion",
    "venue",
    "paid_by",
    "attendees",
    "infrequent_and_irregular",
];

/// The `provided` values, as [`CHARACTER`] names them.
const PROVIDED: [&str; 4] = [
    "food_or_drink",
    "recreation",
    "accommodation_or_travel_for_food_or_drink",
    "accommodation_or_travel_for_recreation",
];

/// The `occasion` values of a food or drink line, as [`CHARACTER`] names them.
const OCCASIONS: [&str; 5] = [
    "social_function",
    "business_meal",
    "with_other_entertainment",
    "light_refreshments",
    "travel_meal",
];

/// The value of `name`, one of `allowed`.
fn choice(
    fields: &Map<String, Value>,
    n: usize,
    name: &str,
    allowed: &[&'static str],
) -> Result<&'static str> {
    let Some(value) = fields.get(name) else {
        return refuse(
            "input.missing_field",
            format!("line {n}: {name} is required"),
        );
    };
    match allowed.iter().find(|a| value.as_str() == Some(**a)) {
        Some(found) => Ok(found),
        None => {
            let (last, rest) = allowed.split_last().expect("choices");
            refuse(
                "fbt_entertainment_classifier.fact_invalid",
                format!("line {n}: {name} must be {} or {last}.", rest.join(", ")),
            )
        }
    }
}

pub(super) fn parse_line(value: &Value, n: usize) -> Result<Line> {
    let Some(fields) = value.as_object() else {
        return refuse(
            "fbt_entertainment_classifier.lines_invalid",
            format!("line {n} is not a JSON object."),
        );
    };
    if let Some(key) = fields.keys().find(|k| !LINE_FIELDS.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("line {n}: {key} is not an input of a line"),
        );
    }
    let cost = line_cost(fields, n)?;
    let includes_gst = gst_included(fields, n)?;
    let provided = choice(fields, n, "provided", &PROVIDED)?;
    let character = line_character(fields, n, provided)?;
    let venue = [
        "business_premises_working_day",
        "business_premises_other_day",
        "elsewhere",
    ];
    let on_premises = choice(fields, n, "venue", &venue)? == venue[0];
    let reimbursed = choice(fields, n, "paid_by", &["employer", "reimbursed"])? == "reimbursed";
    let premises = match (on_premises && !reimbursed, provided) {
        (false, _) => Premises::No,
        (true, "food_or_drink") => Premises::Property,
        (true, _) => Premises::Unsettled,
    };
    let infrequent = infrequent_stated(fields, n)?;
    Ok(Line {
        cost,
        includes_gst,
        provided,
        character,
        premises,
        reimbursed,
        attendees: attendees(fields.get("attendees"), n)?,
        infrequent,
    })
}

/// The line's cost: the employer's expense, net of unreimbursed contributions.
fn line_cost(fields: &Map<String, Value>, n: usize) -> Result<Decimal> {
    let cost = match fields.get("cost").and_then(Value::as_str) {
        Some(text) if text.len() <= 32 => match Decimal::parse(text) {
            Ok(amount) => money(&Arg::Decimal(amount)).ok(),
            Err(_) => None,
        },
        _ => None,
    };
    let Some(cost) = cost else {
        return refuse(
            "input.amount_invalid",
            format!(
                "line {n}: cost must be a decimal string, finite and non-negative, at most 2dp and 1e12: the employer's expense, less any contribution from an employee or associate it does not reimburse (FBTAA s 37AB)."
            ),
        );
    };
    Ok(cost)
}

/// Whether the line's cost includes GST.
fn gst_included(fields: &Map<String, Value>, n: usize) -> Result<bool> {
    Ok(match fields.get("gst_status").and_then(Value::as_str) {
        Some("includes_gst") => true,
        Some("no_gst") => false,
        _ => {
            return refuse(
                "fbt_entertainment_classifier.gst_status_invalid",
                format!(
                    "line {n}: gst_status must be includes_gst or no_gst: whether the cost includes GST sets its GST credit (GST Act s 69-5)."
                ),
            );
        }
    })
}

/// The line's character and its authority from what was provided and the
/// occasion, or none where food or drink has no occasion stated.
fn line_character(
    fields: &Map<String, Value>,
    n: usize,
    provided: &'static str,
) -> Result<Option<(Character, &'static str)>> {
    let by_provided = |p: &str| {
        CHARACTER
            .iter()
            .find(|row| row.0 == p)
            .map(|row| (row.2, row.3))
    };
    Ok(match (provided, fields.get("occasion")) {
        ("food_or_drink" | FOOD_TRAVEL, None) => None,
        ("food_or_drink" | FOOD_TRAVEL, Some(_)) => {
            let occasion = choice(fields, n, "occasion", &OCCASIONS)?;
            let food = CHARACTER
                .iter()
                .find(|row| row.1 == Some(occasion))
                .map(|row| (row.2, row.3));
            match (provided, food) {
                ("food_or_drink", _) => food,
                (_, Some((Meal, _))) => by_provided(provided),
                _ => Some((NotEntertainment, TRAVEL_NOT_ENTERTAINMENT)),
            }
        }
        (_, Some(_)) => {
            return refuse(
                "input.field_unknown",
                format!(
                    "line {n}: occasion is an input of a food_or_drink or {FOOD_TRAVEL} line only"
                ),
            );
        }
        _ => by_provided(provided),
    })
}

/// The operator's statement of s 58P(1)(f)(i), where given.
fn infrequent_stated(fields: &Map<String, Value>, n: usize) -> Result<Option<bool>> {
    Ok(match fields.get("infrequent_and_irregular") {
        None => None,
        Some(Value::Bool(stated)) => Some(*stated),
        Some(_) => {
            return refuse(
                "input.flag_invalid",
                format!("line {n}: infrequent_and_irregular must be a boolean."),
            );
        }
    })
}

/// The classes that attended, with head counts: 0 is absent, null attended
/// but uncounted. A line of more than one class needs every count.
fn attendees(value: Option<&Value>, n: usize) -> Result<Vec<(Class, Option<i64>)>> {
    let bad = || {
        refuse(
            "fbt_entertainment_classifier.fact_invalid",
            format!(
                "line {n}: attendees must be an object of employees, associates and others, each a JSON integer from 0 to 1000000 or null where the count is unknown, at least one more than 0 or null."
            ),
        )
    };
    let Some(Value::Object(given)) = value else {
        return bad();
    };
    if given
        .keys()
        .any(|k| !CLASSES.iter().any(|(name, _)| name == k))
    {
        return bad();
    }
    let mut present = Vec::new();
    for (name, class) in CLASSES {
        match given.get(name) {
            None => {}
            Some(Value::Null) => present.push((class, None)),
            Some(count) => match count.as_i64() {
                Some(0) => {}
                Some(heads @ 1..=1_000_000) => present.push((class, Some(heads))),
                _ => return bad(),
            },
        }
    }
    if present.is_empty() {
        return bad();
    }
    if present.len() > 1 && present.iter().any(|(_, heads)| heads.is_none()) {
        return refuse(
            "fbt_entertainment_classifier.line_not_split",
            format!(
                "line {n}: attendees span more than one class, so each class needs its head count to split the cost per head (TR 97/17 paras 124 and 135); give the counts, or split the line by class."
            ),
        );
    }
    Ok(present)
}
