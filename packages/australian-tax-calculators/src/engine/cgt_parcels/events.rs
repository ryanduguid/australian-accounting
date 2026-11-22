// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Reading `cgt_cost_base_adjustments`'s events, each fact checked once.

use super::demergers::{DEMERGERS, Demerger};
use super::input::Fields;
use super::*;

/// What happened to the security's shares or units.
pub(super) enum Change {
    /// A company's non-assessable payment per share (CGT event G1).
    ReturnOfCapital { per_unit: Decimal },
    /// A non-AMIT trust's non-assessable part for the year (CGT event E4),
    /// and the tax-free amounts that reduce only the reduced cost base.
    TaxDeferred { amount: Decimal, tax_free: Decimal },
    /// An AMIT cost base net amount: an excess when `excess`, else a shortfall.
    Amit { amount: Decimal, excess: bool },
    Demerger {
        demerger: &'static Demerger,
        new_security: String,
        rollover: bool,
    },
}

pub(super) struct Event {
    pub(super) n: usize,
    pub(super) security: String,
    /// The day of the event; just before the end of 30 June for a year's
    /// E4 or AMIT amount.
    pub(super) day: i64,
    /// The parcels named for it; all the security's parcels held by then
    /// when `None`.
    pub(super) parcels: Option<Vec<String>>,
    pub(super) change: Change,
}

impl Change {
    /// Whether it happens to shares in a company, not units in a trust.
    pub(super) fn company(&self) -> bool {
        matches!(
            self,
            Change::ReturnOfCapital { .. } | Change::Demerger { .. }
        )
    }
}

fn invalid<T>(what: &str, key: &str, expected: &str) -> Result<T> {
    refuse(
        "cgt_parcels.fact_invalid",
        format!("{what}: {key} must be {expected}."),
    )
}

/// The last day of an income year given as `2025-26`.
fn year_end(f: &mut Fields, what: &str) -> Result<i64> {
    let text = f.text("income_year")?;
    match year_first(&text) {
        Some(first) => Ok(dates::parse(&format!("{}-06-30", first + 1)).expect("a date")),
        None => invalid(what, "income_year", "an income year such as 2025-26"),
    }
}

fn parcel_names(f: &mut Fields, what: &str) -> Result<Option<Vec<String>>> {
    let Some(value) = f.get("parcels") else {
        return Ok(None);
    };
    let names: Option<Vec<String>> = value.as_array().and_then(|list| {
        list.iter()
            .map(|v| {
                v.as_str()
                    .filter(|s| !s.trim().is_empty())
                    .map(String::from)
            })
            .collect()
    });
    match names {
        Some(names) if !names.is_empty() => Ok(Some(names)),
        _ => invalid(what, "parcels", "a non-empty list of parcel names"),
    }
}

fn demerger(f: &mut Fields, what: &str) -> Result<(i64, Change)> {
    let key = f.text("demerger")?;
    let Some(demerger) = DEMERGERS.iter().find(|d| d.key == key) else {
        let known: Vec<&str> = DEMERGERS.iter().map(|d| d.key).collect();
        return refuse(
            "cgt_cost_base.demerger_unknown",
            format!(
                "{what}: {key} is not a demerger this command carries ({}): the reasonable proportion of ITAA 1997 s 125-80(3) is taken only from a class ruling read at the source.",
                known.join(", ")
            ),
        );
    };
    let new_security = f.text("new_security")?;
    let rollover = match f.required("rollover")? {
        Value::Bool(chosen) => *chosen,
        _ => return invalid(what, "rollover", "a JSON boolean"),
    };
    let day = dates::parse(demerger.implemented).expect("a date");
    let change = Change::Demerger {
        demerger,
        new_security,
        rollover,
    };
    Ok((day, change))
}

pub(super) fn read_event(n: usize, fields: &Map<String, Value>) -> Result<Event> {
    let what = format!("event {n}");
    let mut f = Fields::new(what.clone(), fields);
    let kind = f.text("event")?;
    let security = f.text("security")?;
    let (day, change) = match kind.as_str() {
        "return_of_capital" => {
            let day = f.date("date")?;
            let per_unit = f.units("per_unit")?;
            (day, Change::ReturnOfCapital { per_unit })
        }
        "tax_deferred" => {
            let day = year_end(&mut f, &what)?;
            let amount = f.amount("amount", false)?;
            let tax_free = f.amount("tax_free", true)?;
            (day, Change::TaxDeferred { amount, tax_free })
        }
        "amit_excess" | "amit_shortfall" => {
            let day = year_end(&mut f, &what)?;
            let amount = f.amount("amount", false)?;
            let excess = kind == "amit_excess";
            (day, Change::Amit { amount, excess })
        }
        "demerger" => demerger(&mut f, &what)?,
        _ => {
            return invalid(
                &what,
                "event",
                "return_of_capital, tax_deferred, amit_excess, amit_shortfall or demerger",
            );
        }
    };
    let parcels = if matches!(change, Change::Demerger { .. }) {
        None
    } else {
        parcel_names(&mut f, &what)?
    };
    f.none_left()?;
    Ok(Event {
        n,
        security,
        day,
        parcels,
        change,
    })
}
