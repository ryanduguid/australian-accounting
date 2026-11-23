// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! One foreign currency amount: the time its special translation rule names,
//! the rate the operator states for that time or as an average, and the
//! Australian dollar amount.

use serde_json::{Map, Value, json};

use super::super::{Result, cents, precise, refuse, stated};
use super::{date, decimal, text};
use crate::dates;
use crate::decimal::Decimal;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    OrdinaryIncome,
    StatutoryIncome,
    Deduction,
    DepreciatingAssetCost,
    CgtEvent,
    ForeignTaxPaid,
}

/// A kind's name, the time its item names, the other time that applies when
/// it comes first (or on the same day, unless `strict`), and the item's two
/// results.
struct Rule {
    kind: Kind,
    name: &'static str,
    time: &'static str,
    earlier: Option<&'static str>,
    strict: bool,
    at_time: &'static str,
    at_earlier: &'static str,
}

const RULES: [Rule; 6] = [
    Rule {
        kind: Kind::OrdinaryIncome,
        name: "ordinary_income",
        time: "derived",
        earlier: Some("received"),
        strict: false,
        at_time: "ITAA 1997 s 960-50(6) item 6(b): ordinary income not received at or before it is derived, at the rate when it is derived",
        at_earlier: "ITAA 1997 s 960-50(6) item 6(a): ordinary income received at or before it is derived, at the rate at receipt",
    },
    Rule {
        kind: Kind::StatutoryIncome,
        name: "statutory_income",
        time: "included",
        earlier: Some("received"),
        strict: false,
        at_time: "ITAA 1997 s 960-50(6) item 7(b): statutory income (not under Division 102), at the rate when the requirement to include it in assessable income first arose",
        at_earlier: "ITAA 1997 s 960-50(6) item 7(a): statutory income (not under Division 102) received at or before the requirement to include it first arose, at the rate at receipt",
    },
    Rule {
        kind: Kind::Deduction,
        name: "deduction",
        time: "deductible",
        earlier: Some("paid"),
        strict: false,
        at_time: "ITAA 1997 s 960-50(6) item 8(b), as the regulations substitute it (s 960-50(7)): a deduction (not under s 25-35 or Division 40) not paid at or before it became deductible, at the rate when it became deductible",
        at_earlier: "ITAA 1997 s 960-50(6) item 8(a), as the regulations substitute it (s 960-50(7)): a deduction (not under s 25-35 or Division 40) paid at or before it became deductible, at the rate at payment",
    },
    Rule {
        kind: Kind::DepreciatingAssetCost,
        name: "depreciating_asset_cost",
        time: "held",
        earlier: Some("satisfied"),
        strict: true,
        at_time: "ITAA 1997 s 960-50(6) item 2(a): the obligation for the asset not satisfied before you begin to hold it, at the rate when you begin to hold it",
        at_earlier: "ITAA 1997 s 960-50(6) item 2(b): the obligation for the asset satisfied before you begin to hold it, at the rate when it is satisfied",
    },
    Rule {
        kind: Kind::CgtEvent,
        name: "cgt_event",
        time: "event",
        earlier: None,
        strict: false,
        at_time: "ITAA 1997 s 960-50(6) item 5: an amount or market value relevant to Part 3-1 or 3-3, at the rate at the time of the transaction or event",
        at_earlier: "",
    },
    Rule {
        kind: Kind::ForeignTaxPaid,
        name: "foreign_tax_paid",
        time: "paid",
        earlier: None,
        strict: false,
        at_time: "ITAA 1997 s 960-50(6) item 11: a payment no other item covers, such as foreign income tax paid, at the rate at the time of payment",
        at_earlier: "",
    },
];

const COMMON: [&str; 6] = ["kind", "currency", "amount", "rate", "basis", "rate_source"];

/// The rate the operator states: for the translation day, or an average over
/// a period of up to 12 months that contains it.
enum Basis {
    Specific(i64),
    Average(i64, i64),
}

pub(super) struct Item {
    pub(super) kind: Kind,
    name: &'static str,
    currency: String,
    amount: Decimal,
    rate: Decimal,
    source: String,
    basis: Basis,
    translated_on: i64,
    authority: &'static str,
    pub(super) aud: Decimal,
}

impl Item {
    pub(super) fn read(value: &Value, item: &str) -> Result<Item> {
        let Value::Object(fields) = value else {
            return refuse(
                "foreign_income_conversion.items_invalid",
                format!("{item} must be an object."),
            );
        };
        let rule = rule(fields, item)?;
        let (translated_on, authority) = translation_time(fields, item, rule)?;
        let basis = basis(fields, item, rule, translated_on)?;
        let currency = currency(fields, item)?;
        let amount = decimal(fields, item, "amount", 4, false)?;
        let rate = decimal(fields, item, "rate", 6, true)?;
        let aud = precise(|| Ok(cents(amount.div(&rate)?)?))?;
        Ok(Item {
            kind: rule.kind,
            name: rule.name,
            currency,
            amount,
            rate,
            source: text(fields, item, "rate_source", 200)?.to_owned(),
            basis,
            translated_on,
            authority,
            aud,
        })
    }

    pub(super) fn is_average(&self) -> bool {
        matches!(self.basis, Basis::Average(..))
    }

    pub(super) fn average_capital(&self) -> bool {
        self.is_average() && matches!(self.kind, Kind::CgtEvent | Kind::DepreciatingAssetCost)
    }

    pub(super) fn render(&self) -> Value {
        let (basis, period) = match self.basis {
            Basis::Specific(day) => ("specific", json!({"date": dates::format(day)})),
            Basis::Average(from, to) => (
                "average",
                json!({"from": dates::format(from), "to": dates::format(to)}),
            ),
        };
        json!({
            "kind": self.name,
            "currency": self.currency,
            "amount": self.amount.to_string(),
            "translated_on": dates::format(self.translated_on),
            "translation_rule": self.authority,
            "basis": basis,
            "rate": self.rate.to_string(),
            "rate_period": period,
            "rate_source": self.source,
            "aud": self.aud.to_string(),
        })
    }
}

fn rule(fields: &Map<String, Value>, item: &str) -> Result<&'static Rule> {
    let named = fields.get("kind").and_then(Value::as_str);
    let Some(rule) = RULES.iter().find(|r| Some(r.name) == named) else {
        return refuse(
            "foreign_income_conversion.kind_invalid",
            format!(
                "{item}: kind must be ordinary_income, statutory_income, deduction, depreciating_asset_cost, cgt_event or foreign_tax_paid."
            ),
        );
    };
    let dated: &[&str] = match fields.get("basis").and_then(Value::as_str) {
        Some("average") => &["average_from", "average_to"],
        _ => &["rate_date"],
    };
    let mut allowed: Vec<&str> = COMMON.to_vec();
    allowed.push(rule.time);
    allowed.extend(rule.earlier);
    allowed.extend(dated);
    stated::known_keys(fields, item, "this kind and basis", &allowed)?;
    Ok(rule)
}

/// The day the item's rule names, and the rule.
fn translation_time(
    fields: &Map<String, Value>,
    item: &str,
    rule: &Rule,
) -> Result<(i64, &'static str)> {
    let Some(time) = date(fields, item, rule.time)? else {
        return refuse(
            "input.missing_field",
            format!("{item}: {} is required", rule.time),
        );
    };
    let earlier = match rule.earlier {
        Some(key) => date(fields, item, key)?,
        None => None,
    };
    Ok(match earlier {
        Some(e) if e < time || (e == time && !rule.strict) => (e, rule.at_earlier),
        _ => (time, rule.at_time),
    })
}

fn basis(fields: &Map<String, Value>, item: &str, rule: &Rule, on: i64) -> Result<Basis> {
    let required = |key: &str| -> Result<i64> {
        date(fields, item, key)?.map_or_else(
            || refuse("input.missing_field", format!("{item}: {key} is required")),
            Ok,
        )
    };
    match fields.get("basis").and_then(Value::as_str) {
        Some("specific") => {
            let day = required("rate_date")?;
            if day != on {
                return refuse(
                    "foreign_income_conversion.rate_date",
                    format!(
                        "{item}: the rate is for {}, but the {} rule translates at the rate applicable on {}. State that day's rate, or an average rate.",
                        dates::format(day),
                        rule.name,
                        dates::format(on)
                    ),
                );
            }
            Ok(Basis::Specific(day))
        }
        Some("average") => average(required("average_from")?, required("average_to")?, on, item),
        _ => refuse(
            "foreign_income_conversion.basis_invalid",
            format!("{item}: basis must be specific or average."),
        ),
    }
}

/// An average over a period of at most 12 months that contains the day the
/// amount relates to (Schedule 2 clause 3(1) and (2)(c); 1997 clause 1.3).
fn average(from: i64, to: i64, on: i64, item: &str) -> Result<Basis> {
    let why = if to < from {
        Some("average_to is before average_from")
    } else if to >= dates::add_months(from, 12) {
        Some("the averaging period is longer than 12 months")
    } else if on < from || on > to {
        Some("the averaging period does not contain the day the amount is translated on")
    } else {
        None
    };
    match why {
        Some(why) => refuse(
            "foreign_income_conversion.average_period",
            format!(
                "{item}: {why} (an average is of the rates during a period not exceeding 12 months, used for amounts relating to that period)."
            ),
        ),
        None => Ok(Basis::Average(from, to)),
    }
}

fn currency(fields: &Map<String, Value>, item: &str) -> Result<String> {
    match fields.get("currency") {
        None => refuse(
            "input.missing_field",
            format!("{item}: currency is required"),
        ),
        Some(Value::String(code))
            if code.len() == 3 && code.bytes().all(|b| b.is_ascii_uppercase()) && code != "AUD" =>
        {
            Ok(code.clone())
        }
        Some(_) => refuse(
            "foreign_income_conversion.currency_invalid",
            format!("{item}: currency must be a 3-letter code other than AUD, such as NZD."),
        ),
    }
}
