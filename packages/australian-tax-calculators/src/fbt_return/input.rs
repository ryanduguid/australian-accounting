// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The return's inputs and each benefit's, checked or refused before any
//! worksheet runs.
use super::kind::{Entertainment, Kind, check_entertainment, check_exempt_car};
use super::{COMMAND, Columns, Count, ROWS, missing};
use crate::api::Refusal;
use serde_json::{Value, json};

/// Benefits FBTAA s 5E(3) always excludes from reporting, as their worksheets
/// value them.
const ALWAYS_EXCLUDED: [(&str, &str); 2] = [
    (
        "fbt-car-parking",
        "a car parking fringe benefit is an excluded fringe benefit (FBTAA s 5E(3)(b))",
    ),
    (
        "fbt-meal-entertainment",
        "meal entertainment not provided under a salary packaging arrangement, the only kind fbt-meal-entertainment values, is an excluded fringe benefit (FBTAA s 5E(3)(a))",
    ),
];

/// The inputs the return takes, and those each benefit takes.
const INPUTS: [&str; 8] = [
    "employer",
    "year_ended",
    "scope_confirmed",
    "benefits",
    "employer_kind",
    "rebatable_days",
    "employer_days",
    "instalments_reported",
];
const BENEFIT_INPUTS: [&str; 7] = [
    "employee",
    "command",
    "input",
    "gst_type",
    "excluded",
    "employer",
    "entertainment",
];

/// Refuse a field the return does not take, naming the payroll tax gross-up
/// apart.
pub(super) fn check_fields(v: &Value) -> Result<(), Refusal> {
    for key in v.as_object().into_iter().flat_map(|o| o.keys()) {
        if key.starts_with("payroll_tax") {
            return Err(Refusal::new(
                "fbt_return.payroll_tax_unsupported",
                format!(
                    "{COMMAND} does not work out a payroll tax gross-up: payroll tax is imposed by each state and territory under its own Act, which this command does not read"
                ),
            ));
        }
        if !INPUTS.contains(&key.as_str()) {
            return Err(Refusal::new(
                "input.field_unknown",
                format!("{key} is not an input of {COMMAND}"),
            ));
        }
    }
    Ok(())
}

/// The one employer's label.
pub(super) fn check_employer(v: &Value) -> Result<&str, Refusal> {
    match v.get("employer") {
        Some(Value::String(label)) if !label.is_empty() => Ok(label),
        Some(Value::Array(labels)) if labels.len() > 1 => Err(Refusal::new(
            "fbt_return.multiple_employers",
            format!(
                "{COMMAND} composes one employer's return: employer gives {} employers",
                labels.len()
            ),
        )),
        _ => Err(missing(
            "employer is required: a label for the one employer whose return this is".into(),
        )),
    }
}

/// The FBT year, one the repository has loaded `fbt`'s rates for.
pub(super) fn check_year(v: &Value) -> Result<i64, Refusal> {
    let Some(given) = v.get("year_ended") else {
        return Err(missing(
            "year_ended is required: the year the FBT year ends on 31 March".into(),
        ));
    };
    let supported = crate::metadata::supported_periods("fbt");
    match given.as_i64() {
        Some(year) if supported.contains(&year.to_string().as_str()) => Ok(year),
        _ => Err(Refusal::new(
            "period.unsupported",
            format!(
                "{COMMAND} composes the FBT years ended 31 March {}, whose rates are loaded; year_ended {given} is not one of them",
                supported.join(" and ")
            ),
        )),
    }
}

/// One benefit as given: its worksheet, how it is classified and the
/// worksheet's input with the shared year and scope added.
pub(super) struct Given<'a> {
    pub(super) employee: &'a str,
    pub(super) command: &'static str,
    pub(super) label: &'static str,
    pub(super) count: Count,
    pub(super) columns: Columns,
    pub(super) type_one: bool,
    pub(super) excluded: bool,
    pub(super) entertainment: Entertainment,
    pub(super) input: Value,
}

/// The benefit's worksheet and its item 23 label.
fn row_for(at: &str, b: &Value) -> Result<(&'static str, &'static str, Count, Columns), Refusal> {
    let named = b.get("command").and_then(Value::as_str);
    match ROWS.iter().find(|(c, ..)| Some(*c) == named) {
        Some(row) => Ok(*row),
        None => Err(Refusal::new(
            "fbt_return.benefit_command_unsupported",
            format!(
                "{at}.command must name the benefit worksheet that values it, one of {}",
                ROWS.map(|(c, ..)| c).join(", ")
            ),
        )),
    }
}

/// Whether the benefit is type 1, as stated.
fn gst_type(at: &str, b: &Value) -> Result<bool, Refusal> {
    match b.get("gst_type").and_then(Value::as_str) {
        Some("type_1") => Ok(true),
        Some("type_2") => Ok(false),
        _ => Err(Refusal::new(
            "fbt_return.gst_type_missing",
            format!(
                "{at}.gst_type is required: type_1 for a GST-creditable benefit (FBTAA s 5C(3) step 1, s 149A), whose provider was entitled to an input tax credit for providing it, or type_2 for any other (s 5C(4))"
            ),
        )),
    }
}

/// Whether the benefit is excluded from reporting, as stated and as the law
/// fixes it for car parking and meal entertainment.
fn excluded(at: &str, command: &str, b: &Value) -> Result<bool, Refusal> {
    let excluded = match b.get("excluded") {
        None => {
            return Err(missing(format!(
                "{at}.excluded is required: whether the benefit is an excluded fringe benefit (FBTAA s 5E(3)), left out of reporting"
            )));
        }
        Some(Value::Bool(flag)) => *flag,
        Some(_) => {
            return Err(Refusal::new(
                "input.flag_invalid",
                format!("{at}.excluded must be a boolean"),
            ));
        }
    };
    match ALWAYS_EXCLUDED.iter().find(|(c, _)| *c == command) {
        Some((_, why)) if !excluded => Err(Refusal::new(
            "fbt_return.exclusion_conflict",
            format!("{at}.excluded must be true: {why}"),
        )),
        _ => Ok(excluded),
    }
}

/// The worksheet's own input, with the return's year and scope added. A year
/// given inside it must be the return's.
fn worksheet_input(at: &str, b: &Value, top: &Value, year: i64) -> Result<Value, Refusal> {
    let Some(mut input) = b.get("input").filter(|i| i.is_object()).cloned() else {
        return Err(missing(format!(
            "{at}.input is required: an object of the worksheet's own inputs"
        )));
    };
    if input.get("scope_confirmed").is_some() {
        return Err(Refusal::new(
            "input.field_unknown",
            format!(
                "scope_confirmed is not an input of {at} within {COMMAND}: it is given once, for every benefit"
            ),
        ));
    }
    if let Some(other) = input.get("year_ended").filter(|y| y.as_i64() != Some(year)) {
        return Err(Refusal::new(
            "fbt_return.mixed_years",
            format!(
                "{at}.input.year_ended is {other}, but the return is for the FBT year ended 31 March {year}: one return covers one FBT year"
            ),
        ));
    }
    input["year_ended"] = json!(year);
    input["scope_confirmed"] = top["scope_confirmed"].clone();
    Ok(input)
}

/// A benefit's own employer, where given, must be the return's.
fn same_employer(at: &str, b: &Value, employer: &str) -> Result<(), Refusal> {
    match b.get("employer").filter(|e| e.as_str() != Some(employer)) {
        Some(other) => Err(Refusal::new(
            "fbt_return.multiple_employers",
            format!(
                "{at}.employer is {other}, not {employer}: {COMMAND} composes one employer's return"
            ),
        )),
        None => Ok(()),
    }
}

/// One benefit, checked before any worksheet runs.
pub(super) fn check_benefit<'a>(
    at: &str,
    b: &'a Value,
    top: &Value,
    employer: &str,
    (year, kind): (i64, Kind),
) -> Result<Given<'a>, Refusal> {
    let Some(fields) = b.as_object() else {
        return Err(Refusal::new(
            "input.not_object",
            format!("{at} must be a JSON object"),
        ));
    };
    if let Some(key) = fields
        .keys()
        .find(|k| !BENEFIT_INPUTS.contains(&k.as_str()))
    {
        return Err(Refusal::new(
            "input.field_unknown",
            format!("{key} is not an input of a benefit within {COMMAND}"),
        ));
    }
    same_employer(at, b, employer)?;
    let (command, label, count, columns) = row_for(at, b)?;
    let Some(employee) = b["employee"].as_str().filter(|e| !e.is_empty()) else {
        return Err(missing(format!(
            "{at}.employee is required: a label for the employee the benefit is provided in respect of"
        )));
    };
    let type_one = gst_type(at, b)?;
    let excluded = excluded(at, command, b)?;
    let input = worksheet_input(at, b, top, year)?;
    check_exempt_car(at, &input, kind)?;
    Ok(Given {
        employee,
        command,
        label,
        count,
        columns,
        type_one,
        excluded,
        entertainment: check_entertainment(at, b, command, excluded, kind)?,
        input,
    })
}
