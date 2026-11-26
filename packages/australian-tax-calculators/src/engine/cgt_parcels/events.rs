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
    /// Shares or trust units exchanged for another entity's under a scrip for
    /// scrip roll-over (Subdivision 124-M).
    ScripForScrip(Scrip),
}

/// A scrip for scrip exchange's terms, per original share or unit.
pub(super) struct Scrip {
    pub(super) new_security: String,
    /// Replacement shares or units for each original one.
    pub(super) ratio: Decimal,
    /// The market value of one replacement share or unit.
    pub(super) value: Decimal,
    /// Ineligible proceeds (s 124-790), such as cash, for each original one.
    pub(super) cash: Decimal,
}

pub(super) struct Event {
    pub(super) n: usize,
    pub(super) security: String,
    /// The day of the event; for a year's E4 or AMIT amount, just before the
    /// year's end or just before another CGT event to the units in the year
    /// (ss 104-70(3) and 104-107B(4)).
    pub(super) day: i64,
    /// The first day of the income year of an E4 or AMIT amount.
    pub(super) year: Option<i64>,
    /// The parcels named for it; all the security's parcels held by then
    /// when `None`.
    pub(super) parcels: Option<Vec<String>>,
    pub(super) change: Change,
}

impl Change {
    /// Whether it happens to shares in a company (`Some(true)`) or units in
    /// a trust (`Some(false)`); a scrip for scrip exchange can be either.
    pub(super) fn company(&self) -> Option<bool> {
        match self {
            Change::ReturnOfCapital { .. } | Change::Demerger { .. } => Some(true),
            Change::ScripForScrip(_) => None,
            _ => Some(false),
        }
    }
}

fn invalid<T>(what: &str, key: &str, expected: &str) -> Result<T> {
    refuse(
        "cgt_parcels.fact_invalid",
        format!("{what}: {key} must be {expected}."),
    )
}

/// When a year's E4 or AMIT amount happens, and the year's first day: just
/// before the end of the income year given as `2025-26`, or just before
/// `before_cgt_event`, another CGT event to the units in that year
/// (ss 104-70(3)(b) and 104-107B(4)(b)).
fn timing(f: &mut Fields, what: &str) -> Result<(i64, Option<i64>)> {
    let text = f.text("income_year")?;
    let Some(first) = year_first(&text) else {
        return invalid(what, "income_year", "an income year such as 2025-26");
    };
    let start = dates::parse(&format!("{first}-07-01")).expect("a date");
    let end = dates::parse(&format!("{}-06-30", first + 1)).expect("a date");
    if f.get("before_cgt_event").is_none() {
        return Ok((end, Some(start)));
    }
    let day = f.date("before_cgt_event")?;
    if day < start || day > end {
        return invalid(what, "before_cgt_event", "a date in income_year");
    }
    Ok((day, Some(start)))
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

fn demerger(f: &mut Fields, what: &str) -> Result<((i64, Option<i64>), Change)> {
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
    Ok(((day, None), change))
}

/// A scrip for scrip exchange, for which the holder chose the roll-over.
fn scrip(f: &mut Fields, what: &str) -> Result<((i64, Option<i64>), Change)> {
    let day = f.date("date")?;
    let new_security = f.text("new_security")?;
    let ratio = f.units("new_units_per_unit")?;
    let value = f.units("market_value_per_new_unit")?;
    let cash = match f.get("cash_per_unit") {
        None => int(0),
        Some(_) => f.units("cash_per_unit")?,
    };
    let chosen = match f.required("rollover")? {
        Value::Bool(chosen) => *chosen,
        _ => return invalid(what, "rollover", "a JSON boolean"),
    };
    if !chosen {
        return refuse(
            "cgt_cost_base.scrip_without_rollover",
            format!(
                "{what}: without the roll-over the exchange is a disposal of the original shares or units (CGT event A1 or C2) and an acquisition of the new ones at market value: work the disposal in cgt-parcels and add the new parcel."
            ),
        );
    }
    let terms = Scrip {
        new_security,
        ratio,
        value,
        cash,
    };
    Ok(((day, None), Change::ScripForScrip(terms)))
}

pub(super) fn read_event(n: usize, fields: &Map<String, Value>) -> Result<Event> {
    let what = format!("event {n}");
    let mut f = Fields::new(what.clone(), fields);
    let kind = f.text("event")?;
    let security = f.text("security")?;
    let ((day, year), change) = match kind.as_str() {
        "return_of_capital" => {
            let day = f.date("date")?;
            let per_unit = f.units("per_unit")?;
            ((day, None), Change::ReturnOfCapital { per_unit })
        }
        "tax_deferred" => {
            let when = timing(&mut f, &what)?;
            let amount = f.amount("amount", false)?;
            let tax_free = f.amount("tax_free", true)?;
            (when, Change::TaxDeferred { amount, tax_free })
        }
        "amit_excess" | "amit_shortfall" => {
            let when = timing(&mut f, &what)?;
            let amount = f.amount("amount", false)?;
            let excess = kind == "amit_excess";
            (when, Change::Amit { amount, excess })
        }
        "demerger" => demerger(&mut f, &what)?,
        "scrip_for_scrip" => scrip(&mut f, &what)?,
        _ => {
            return invalid(
                &what,
                "event",
                "return_of_capital, tax_deferred, amit_excess, amit_shortfall, demerger or scrip_for_scrip",
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
        year,
        parcels,
        change,
    })
}
