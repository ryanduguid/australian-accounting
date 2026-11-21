// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The meal entertainment register check: whether a 12 week register is
//! valid for an FBT year, and the values `fbt-meal-entertainment` takes from
//! it. FBTAA Division 9A Subdivision C, ss 37CA to 37CF, read on the ATO's
//! legal database on 19 November 2026, 9.25 am AEDT, with the ATO's Fringe
//! benefits tax - a guide for employers, 14.8 ('Valid register' and 'Details
//! to be included in register'), read the same day.

use serde_json::{Map, Value, json};

use super::super::{Arg, Error, Result, fbt_year, int, money, precise, rate, refuse, result};
use super::register_still_valid;
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "fbt_meal_entertainment_register";
const COMMAND: &str = "fbt-meal-entertainment-register";
const ENTRY_FIELDS: [&str; 8] = [
    "date",
    "recipients",
    "cost",
    "kind",
    "location",
    "on_premises",
    "in_house_dining_facility",
    "fringe_benefits_value",
];

/// One entry's amounts, where its cost was recorded, and the details
/// s 37CE(1) requires that it does not record.
struct Entry {
    amounts: Option<(Decimal, Decimal)>,
    missing: Vec<&'static str>,
}

fn bad<T>(n: usize, why: impl std::fmt::Display) -> Result<T> {
    refuse(
        "fbt_meal_entertainment_register.entry_invalid",
        format!("entry {n}: {why}"),
    )
}

/// `field` as recorded: `None` when absent, null or an empty string, which
/// the register then does not record.
fn recorded<'a>(fields: &'a Map<String, Value>, field: &str) -> Option<&'a Value> {
    fields
        .get(field)
        .filter(|v| !v.is_null() && v.as_str().is_none_or(|s| !s.trim().is_empty()))
}

/// The employees and associates among the recipients, or `None` where the
/// register does not record them (s 37CE(1)(b)).
fn recipients(n: usize, value: Option<&Value>) -> Result<Option<i64>> {
    let Some(value) = value else { return Ok(None) };
    let counts = ["employees", "associates", "others"].map(|key| {
        value
            .as_object()
            .and_then(|o| o.get(key))
            .and_then(Value::as_i64)
            .filter(|count| (0..=100_000).contains(count))
    });
    let keys_ok = value.as_object().is_some_and(|o| {
        o.keys()
            .all(|k| ["employees", "associates", "others"].contains(&k.as_str()))
    });
    match counts {
        [Some(e), Some(a), Some(o)] if keys_ok => Ok((e + a + o > 0).then_some(e + a)),
        _ => bad(
            n,
            "recipients must be an object of employees, associates and others, each a whole number from 0 to 100000.",
        ),
    }
}

/// An entry checked: its date within the register period, the details it
/// records and, where it records its cost, the cost and the value of the
/// meal entertainment fringe benefits in it.
fn entry(n: usize, item: &Value, period: (i64, i64)) -> Result<Entry> {
    let Some(fields) = item.as_object() else {
        return bad(n, "each entry must be an object.");
    };
    if let Some(key) = fields.keys().find(|k| !ENTRY_FIELDS.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("entry {n}: {key} is not an input of {COMMAND}"),
        );
    }
    let mut missing = Vec::new();
    if !entry_dated(n, fields, period)? {
        missing.push("the date (s 37CE(1)(a))");
    }
    let staff = recipients(n, recorded(fields, "recipients"))?;
    if staff.is_none() {
        missing.push("whether each recipient is an employee or an associate of one (s 37CE(1)(b))");
    }
    let amounts = entry_cost(n, fields, staff)?;
    if amounts.is_none() {
        missing.push("the cost (s 37CE(1)(c))");
    }
    for (field, detail) in [
        ("kind", "the kind of meal entertainment (s 37CE(1)(d))"),
        ("location", "where it was provided (s 37CE(1)(e))"),
    ] {
        match recorded(fields, field) {
            None => missing.push(detail),
            Some(Value::String(_)) => {}
            Some(_) => return bad(n, format!("{field} must be a string.")),
        }
    }
    if !entry_facility_recorded(n, fields)? {
        missing.push(
            "whether it was provided in an in-house dining facility, as it was on the employer's premises (s 37CE(1)(f))",
        );
    }
    Ok(Entry { amounts, missing })
}

/// Whether the entry records its date (s 37CE(1)(a)), which must fall in the
/// register period.
fn entry_dated(n: usize, fields: &Map<String, Value>, period: (i64, i64)) -> Result<bool> {
    let Some(date) = recorded(fields, "date") else {
        return Ok(false);
    };
    match date.as_str().and_then(dates::parse) {
        Some(day) if (period.0..=period.1).contains(&day) => Ok(true),
        Some(_) => bad(
            n,
            "its date is outside register_start to register_end: give only the entries of the register's 12 week period.",
        ),
        None => bad(n, "date must be a YYYY-MM-DD date from 1900 to 2100."),
    }
}

/// The entry's cost and fringe benefits value, or `None` where it does not
/// record its cost (s 37CE(1)(c)).
fn entry_cost(
    n: usize,
    fields: &Map<String, Value>,
    staff: Option<i64>,
) -> Result<Option<(Decimal, Decimal)>> {
    let benefits = fields.get("fringe_benefits_value");
    match recorded(fields, "cost") {
        Some(cost) => entry_amounts(n, cost, benefits, staff).map(Some),
        None if benefits.is_some_and(|v| !v.is_null()) => {
            bad(n, "fringe_benefits_value needs the cost it is part of.")
        }
        None => Ok(None),
    }
}

/// Whether the entry records what s 37CE(1)(f) requires: for meal
/// entertainment on the employer's premises, whether it was in an in-house
/// dining facility; off them, nothing.
fn entry_facility_recorded(n: usize, fields: &Map<String, Value>) -> Result<bool> {
    let Some(&Value::Bool(on_premises)) = fields.get("on_premises") else {
        return bad(
            n,
            "on_premises must be a boolean: whether it was provided on the employer's premises.",
        );
    };
    match (on_premises, fields.get("in_house_dining_facility")) {
        (true, None | Some(Value::Null)) => Ok(false),
        (true, Some(Value::Bool(_))) | (false, None | Some(Value::Null)) => Ok(true),
        (true, Some(_)) => bad(n, "in_house_dining_facility must be a boolean or null."),
        (false, Some(_)) => bad(
            n,
            "in_house_dining_facility applies only to meal entertainment on the employer's premises (s 37CE(1)(f)).",
        ),
    }
}

/// The entry's cost and the value of its meal entertainment fringe benefits,
/// which is at most the cost and nil when no employee or associate received
/// any of it.
fn entry_amounts(
    n: usize,
    cost: &Value,
    benefits: Option<&Value>,
    staff: Option<i64>,
) -> Result<(Decimal, Decimal)> {
    let amount = |value: Option<&Value>, name: &str| -> Result<Decimal> {
        let parsed = value
            .and_then(Value::as_str)
            .and_then(|s| Decimal::parse_input(s).ok());
        match parsed.map(|d| money(&Arg::Decimal(d))) {
            Some(Ok(d)) => Ok(d),
            _ => bad(
                n,
                format!(
                    "{name} must be a decimal string of AUD from 0 to 1000000000000.00, at most 2 decimal places."
                ),
            ),
        }
    };
    let cost = amount(Some(cost), "cost")?;
    let benefits = amount(benefits, "fringe_benefits_value")?;
    if benefits.gt(&cost)? {
        return bad(
            n,
            "fringe_benefits_value cannot exceed the cost: it is the part of the cost that is meal entertainment fringe benefits (FBTAA s 37CB(2)).",
        );
    }
    if staff == Some(0) && !benefits.is_zero() {
        return bad(
            n,
            "fringe_benefits_value must be 0.00 when no employee or associate of one received the meal entertainment: only they receive fringe benefits (FBTAA s 136(1)).",
        );
    }
    Ok((cost, benefits))
}

/// The FBT year, by the year it ends on 31 March, that `day` falls in.
fn fbt_year_of(day: i64) -> i64 {
    let (year, month, _) = dates::to_civil(day);
    if month >= 4 { year + 1 } else { year }
}

/// One check: what it tests, its authority, whether it passed and why.
fn check(id: &str, authority: &str, passed: bool, detail: String) -> Value {
    json!({"check": id, "authority": authority, "passed": passed, "detail": detail})
}

/// s 37CD(1) and (2): the first FBT year the register is valid for.
fn first_valid_year(start: i64, end: i64) -> Result<i64> {
    let (first, last) = (fbt_year_of(start), fbt_year_of(end));
    if last - first > 1 {
        return refuse(
            "fbt_meal_entertainment_register.period_invalid",
            "The register period spans more than 2 FBT years, which s 37CD(1) and (2) do not provide for: a register is kept for a continuous period of at least 12 weeks.",
        );
    }
    Ok(last)
}

/// s 37CD(1) to (3): whether the register is valid for `year`, given each
/// earlier year's expenses for the 20% rule where `year` is a later year.
fn validity(first: i64, year: i64, earlier: Option<&Arg>) -> Result<[Value; 2]> {
    let window = format!(
        "valid from the FBT year ended 31 March {first} to the year ended 31 March {}",
        first + 4
    );
    let in_window = (first..=first + 4).contains(&year);
    let detail = if year < first {
        format!("Not yet valid for the year ended 31 March {year}: {window}.")
    } else if in_window {
        format!("The register is {window}.")
    } else {
        format!(
            "No longer valid for the year ended 31 March {year}: {window}. Keep a new register."
        )
    };
    let span = check("valid_years", "FBTAA s 37CD(1) and (2)", in_window, detail);
    let increase = expenditure_increase(first, year, in_window && year > first, earlier)?;
    Ok([span, increase])
}

/// s 37CD(3): whether a later year's meal entertainment expenses were more
/// than 20% higher than the first valid year's, `needed` when `year` is a
/// later year of the register's validity.
fn expenditure_increase(
    first: i64,
    year: i64,
    needed: bool,
    earlier: Option<&Arg>,
) -> Result<Value> {
    let rule = "FBTAA s 37CD(3)";
    Ok(match (needed, earlier) {
        (false, None) => check(
            "expenditure_increase",
            rule,
            true,
            "No earlier year of the register's validity to compare with its first.".into(),
        ),
        (false, Some(_)) => {
            return refuse(
                "input.field_unknown",
                format!("register_years_expenditure is not an input of {COMMAND} unless year_ended is a later year of the register's validity"),
            );
        }
        (true, None) => {
            return refuse(
                "input.missing_field",
                "register_years_expenditure is required when year_ended is after the register's first valid year: each year's meal entertainment expenses from that year to the one before year_ended (FBTAA s 37CD(3)).",
            );
        }
        (true, Some(earlier)) => match register_still_valid(earlier, first, year) {
            Ok(()) => check(
                "expenditure_increase",
                rule,
                true,
                "No later year's meal entertainment expenses were more than 20% higher than the first valid year's.".into(),
            ),
            Err(Error::Value {
                code: "fbt_meal_entertainment.register_ceased",
                message,
            }) => check("expenditure_increase", rule, false, message),
            Err(other) => return Err(other),
        },
    })
}

/// FBTAA ss 37CC to 37CF: the register period, each entry's details, the FBT
/// years the register is valid for and its values for s 37CB(2).
pub fn fbt_meal_entertainment_register(
    register_start: &Arg,
    register_end: &Arg,
    entries: &Arg,
    year_ended: &Arg,
    scope_confirmed: &Arg,
    earlier: Option<&Arg>,
) -> Result<Value> {
    let (year, period, _) = fbt_year(scope_confirmed, year_ended, KIND)?;
    let year: i64 = year.parse().expect("FBT year");
    let (start, end) = register_period(register_start, register_end)?;
    let first = first_valid_year(start, end)?;
    let items = match entries {
        Arg::Json(Value::Array(items)) if items.len() <= 10_000 => items,
        _ => {
            return refuse(
                "fbt_meal_entertainment_register.entries_invalid",
                "entries must be a list of at most 10000 entry objects.",
            );
        }
    };
    let parsed = items
        .iter()
        .enumerate()
        .map(|(i, item)| entry(i + 1, item, (start, end)))
        .collect::<Result<Vec<_>>>()?;
    let days = end - start + 1;
    let mut checks = register_checks(days, &parsed);
    checks.extend(validity(first, year, earlier)?);
    summary(&period, (first, days), &parsed, checks, earlier)
}

/// The register period's first and last days, in order.
fn register_period(register_start: &Arg, register_end: &Arg) -> Result<(i64, i64)> {
    let day = |value: &Arg, name: &str| match value.text().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => refuse(
            "input.date_invalid",
            format!("{name} must be a YYYY-MM-DD date from 1900 to 2100."),
        ),
    };
    let (start, end) = (
        day(register_start, "register_start")?,
        day(register_end, "register_end")?,
    );
    if start > end {
        return refuse(
            "fbt_meal_entertainment_register.period_invalid",
            "register_start must be on or before register_end.",
        );
    }
    Ok((start, end))
}

/// s 37CC(1) and (3) and s 37CE(1): the period's length, that the register
/// records meal entertainment and each entry's details.
fn register_checks(days: i64, parsed: &[Entry]) -> Vec<Value> {
    let mut checks = vec![check(
        "period_length",
        "FBTAA s 37CC(1) and (3)",
        days >= 84,
        format!(
            "The register covers {days} days, {} weeks and {} days; it must be kept for a continuous period of at least 12 weeks (84 days).",
            days / 7,
            days % 7
        ),
    )];
    checks.push(check(
        "meal_entertainment_provided",
        "FBTAA s 37CC(1)",
        !parsed.is_empty(),
        format!(
            "{} entries. Whether the register was kept, and meal entertainment provided, throughout the period is the employer's to establish.",
            parsed.len()
        ),
    ));
    let gaps: Vec<String> = parsed
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.missing.is_empty())
        .map(|(i, e)| format!("entry {}: no record of {}", i + 1, e.missing.join(", ")))
        .collect();
    checks.push(check(
        "entry_details",
        "FBTAA s 37CE(1)",
        gaps.is_empty(),
        if gaps.is_empty() {
            "Every entry records the date, whether each recipient is an employee or an associate, the cost, the kind, where it was provided and, on the employer's premises, whether in an in-house dining facility.".into()
        } else {
            format!("{}.", gaps.join("; "))
        },
    ));
    checks
}

/// The result: the register's totals and percentage, the checks and, when
/// every check passes, the inputs `fbt-meal-entertainment` takes from it.
fn summary(
    period: &str,
    (first, days): (i64, i64),
    parsed: &[Entry],
    mut checks: Vec<Value>,
    earlier: Option<&Arg>,
) -> Result<Value> {
    let (mut total, mut benefits) = (int(0), int(0));
    for (cost, value) in parsed.iter().filter_map(|e| e.amounts.as_ref()) {
        (total, benefits) = precise(|| Ok((total.add(cost)?, benefits.add(value)?)))?;
    }
    let uncosted = parsed.iter().filter(|e| e.amounts.is_none()).count();
    let mut amounts = vec![
        ("register_meal_entertainment_value", total.clone()),
        ("register_fringe_benefits_value", benefits.clone()),
    ];
    if !total.is_zero() {
        let percentage = precise(|| Ok(benefits.div(&total)?.mul(&int(100))?))?;
        amounts.push(("register_percentage", percentage));
    }
    checks.push(check(
        "positive_total_cost",
        "FBTAA s 37CB(2)",
        !total.is_zero(),
        "The total value of recorded meal entertainment must be positive to calculate the register percentage.".into(),
    ));
    let valid = checks.iter().all(|c| c["passed"] == true);
    let rates = vec![
        rate("register_days", days),
        rate("first_valid_year", first),
        rate("last_valid_year", first + 4),
        rate("entries_without_cost", uncosted),
    ];
    let mut out = result(KIND, period, amounts, rates)?;
    out["valid"] = json!(valid);
    out["checks"] = Value::Array(checks);
    if valid {
        let mut input = json!({
            "method": "12_week_register",
            "register_fringe_benefits_value": out["amounts"]["register_fringe_benefits_value"],
            "register_meal_entertainment_value": out["amounts"]["register_meal_entertainment_value"],
            "register_first_year": first,
        });
        input["register_years_expenditure"] = match earlier {
            Some(Arg::Json(list)) => list.clone(),
            _ => json!([]),
        };
        out["fbt_meal_entertainment_input"] = input;
    }
    Ok(out)
}
