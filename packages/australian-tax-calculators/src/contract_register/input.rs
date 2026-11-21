// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The command's arguments and the contracts file's lines, checked or refused.
use super::{COMMON, Format, KINDS, Kind, MOST_CONTRACTS, NAME_RULE, NAMES, Options};
use crate::api::Refusal;
use crate::dates;
use std::collections::{BTreeMap, HashMap};

pub(super) fn argument(message: String) -> Refusal {
    Refusal::new("contract_register.argument_invalid", message)
}

/// The flags given with their values, refusing an unknown or repeated one
/// and a missing `--scope-confirmed`.
fn flags<'a>(args: &[&'a str]) -> Result<BTreeMap<&'a str, &'a str>, Refusal> {
    let (mut given, mut scope) = (BTreeMap::new(), false);
    let mut args = args.iter();
    while let Some(&flag) = args.next() {
        match flag {
            "--scope-confirmed" if !scope => scope = true,
            "--balance-date" | "--expiring-by" | "--format" if !given.contains_key(flag) => {
                let value = args
                    .next()
                    .ok_or_else(|| argument(format!("{flag} needs a value")))?;
                given.insert(flag, *value);
            }
            _ => {
                return Err(argument(format!(
                    "{flag} is not an argument of contract-register, or is given twice"
                )));
            }
        }
    }
    if !scope {
        return Err(Refusal::new(
            "scope.not_confirmed",
            "Establish all stated scope conditions before calculating.",
        ));
    }
    Ok(given)
}

pub(super) fn options(args: &[&str]) -> Result<Options, Refusal> {
    let given = flags(args)?;
    let date = |flag: &str| {
        given
            .get(flag)
            .map(|text| {
                dates::parse(text).ok_or_else(|| {
                    argument(format!(
                        "{flag} must be a YYYY-MM-DD date from 1900 to 2100"
                    ))
                })
            })
            .transpose()
    };
    let balance =
        date("--balance-date")?.ok_or_else(|| argument("--balance-date is required".into()))?;
    let current_to = dates::add_months(balance, 12);
    let expiring_by = date("--expiring-by")?.unwrap_or(current_to);
    if expiring_by < balance {
        return Err(argument(
            "--expiring-by must not be before --balance-date".into(),
        ));
    }
    let format = match given.get("--format").copied() {
        None | Some("json") => Format::Json,
        Some("text") => Format::Text,
        Some("csv") => Format::Csv,
        Some(other) => {
            return Err(argument(format!(
                "--format must be json, text or csv, not {other:?}"
            )));
        }
    };
    Ok(Options {
        balance,
        current_to,
        expiring_by,
        format,
    })
}

/// One contract's line: its number, its cells by column and its kind.
pub(super) struct Row<'a> {
    pub(super) line: usize,
    pub(super) cells: BTreeMap<&'a str, &'a str>,
    pub(super) kind: &'static Kind,
}

impl<'a> Row<'a> {
    pub(super) fn cell(&self, column: &str) -> &'a str {
        self.cells.get(column).copied().unwrap_or("")
    }
}

pub(super) fn column(line: usize, message: String) -> Refusal {
    Refusal::new(
        "contract_register.column_invalid",
        format!("line {line}: {message}"),
    )
}

pub(super) fn header(line: usize, columns: &[String]) -> Result<(), Refusal> {
    let known = |c: &str| COMMON.contains(&c) || KINDS.iter().any(|k| k.takes(c));
    for (i, name) in columns.iter().enumerate() {
        if !known(name) {
            return Err(column(
                line,
                format!("{name:?} is not a column of the register"),
            ));
        }
        if columns[..i].contains(name) {
            return Err(column(line, format!("{name} is given twice")));
        }
    }
    match COMMON
        .iter()
        .find(|c| !columns.iter().any(|name| name == *c))
    {
        Some(missing) => Err(column(line, format!("the header lacks {missing}"))),
        None => Ok(()),
    }
}

pub(super) fn valid_name(text: &str) -> bool {
    (1..=200).contains(&text.chars().count())
        && !text.chars().any(char::is_control)
        && !text.starts_with(['=', '+', '-', '@'])
}

pub(super) fn row<'a>(
    line: usize,
    columns: &'a [String],
    fields: &'a [String],
) -> Result<Row<'a>, Refusal> {
    if fields.len() != columns.len() {
        return Err(Refusal::new(
            "contract_register.csv_invalid",
            format!(
                "line {line} has {} fields where the header has {}",
                fields.len(),
                columns.len()
            ),
        ));
    }
    let cells: BTreeMap<&str, &str> = columns
        .iter()
        .map(String::as_str)
        .zip(fields.iter().map(String::as_str))
        .collect();
    if let Some(name) = NAMES.iter().find(|name| !valid_name(cells[*name])) {
        return Err(Refusal::new(
            "contract_register.text_invalid",
            format!("line {line}: {name} {NAME_RULE}"),
        ));
    }
    let kind = kind(line, &cells)?;
    Ok(Row { line, cells, kind })
}

/// The line's kind, refusing one the register does not set out, a filled
/// cell the kind does not take and an empty one it needs.
fn kind(line: usize, cells: &BTreeMap<&str, &str>) -> Result<&'static Kind, Refusal> {
    let Some(kind) = KINDS.iter().find(|k| k.name == cells["kind"]) else {
        let taken: Vec<&str> = KINDS.iter().map(|k| k.name).collect();
        return Err(Refusal::new(
            "contract_register.kind_unsupported",
            format!(
                "line {line}: kind {:?} is not one the register sets out; it takes {}",
                cells["kind"],
                taken.join(", ")
            ),
        ));
    };
    let stray = cells
        .iter()
        .find(|&(c, cell)| !cell.is_empty() && !COMMON.contains(c) && !kind.takes(c));
    if let Some((name, _)) = stray {
        let why = format!(
            "a {} contract does not take {name}; leave it empty",
            kind.name
        );
        return Err(column(line, why));
    }
    let mut required = kind.required.iter().chain(&["start_date"]);
    if let Some(missing) = required.find(|c| cells.get(**c).is_none_or(|v| v.is_empty())) {
        let why = format!("a {} contract needs {missing}", kind.name);
        return Err(column(line, why));
    }
    Ok(kind)
}

/// The header's and each contract's line, refusing the first that is wrong.
pub(super) fn rows(records: &[(usize, Vec<String>)]) -> Result<Vec<Row<'_>>, Refusal> {
    let Some(((at, columns), body)) = records.split_first() else {
        return Err(Refusal::new(
            "contract_register.csv_invalid",
            "the contracts file is empty: it needs a header line, then one contract a line",
        ));
    };
    header(*at, columns)?;
    if body.is_empty() || body.len() > MOST_CONTRACTS {
        return Err(Refusal::new(
            "contract_register.csv_invalid",
            format!(
                "the register must hold 1 to {MOST_CONTRACTS} contracts, one a line after the header; it holds {}",
                body.len()
            ),
        ));
    }
    let mut seen: HashMap<&str, usize> = HashMap::new();
    body.iter()
        .map(|(line, fields)| {
            let row = row(*line, columns, fields)?;
            if let Some(first) = seen.insert(row.cell("contract"), *line) {
                return Err(Refusal::new(
                    "contract_register.contract_duplicate",
                    format!(
                        "line {line}: contract {:?} is also on line {first}",
                        row.cell("contract")
                    ),
                ));
            }
            Ok(row)
        })
        .collect()
}
