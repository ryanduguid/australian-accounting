// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `ato_interest_penalties`: the general interest charge (TAA 1953 ss 8AAC
//! and 8AAD; ITAA 1997 s 5-15), the shortfall interest charge (TAA Sch 1
//! ss 280-100 and 280-105), the failure to lodge on time penalty (s 286-80),
//! the false or misleading statement and not reasonably arguable penalties
//! (ss 284-85, 284-90, 284-220, 284-224 and 284-225), and the interest the
//! Commissioner pays on early payments and overpayments (Taxation (Interest on
//! Overpayments and Early Payments) Act 1983 ss 8A to 8C, 9, 10, 12D and 12E),
//! each before any remission. Read on 22 November 2026: the sections at the
//! ATO's legal database; the Overpayments Act, compilation 58 (14 October
//! 2024), at the Federal Register of Legislation; Crimes Act 1914 s 4AA; the
//! ATO's GIC, SIC, credit interest and penalty unit pages; PS LA 2006/8,
//! 2011/19, 2011/23 and 2012/5; MT 2008/2 and MT 2012/3.

use serde_json::{Map, Value, json};

mod charges;
mod credit;
mod penalties;
pub(super) mod rates;

use super::{Arg, Result, d, refuse, scope, stated};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "ato_interest_penalties";
const QUESTIONS: [&str; 6] = [
    "gic",
    "sic",
    "failure_to_lodge",
    "statement_penalty",
    "early_payment_interest",
    "overpayment_interest",
];

const REMISSION: &str = "Before remission: the Commissioner may remit GIC (TAA 1953 s 8AAG), SIC (TAA Sch 1 s 280-160) and administrative penalties (s 298-20), a discretion this command does not estimate";

/// The question's facts: every field beside `question`, `year` and
/// `scope_confirmed`.
pub(super) struct Facts<'a> {
    question: &'static str,
    map: &'a Map<String, Value>,
}

impl Facts<'_> {
    fn item(&self) -> String {
        format!("question {}", self.question)
    }

    fn only(&self, takes: &[&str]) -> Result<()> {
        stated::known_keys(self.map, &self.item(), self.question, takes)
    }

    fn given(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    fn value(&self, key: &str) -> Result<&Value> {
        stated::required(self.map, &self.item(), key)
    }

    fn cents(&self, key: &str) -> Result<u64> {
        to_cents(&stated::amount(self.map, &self.item(), key)?)
    }

    fn date(&self, key: &str) -> Result<i64> {
        date(self.value(key)?, &self.item(), key)
    }

    fn flag(&self, key: &str) -> Result<bool> {
        flag(self.value(key)?, &self.item(), key)
    }

    fn choice(&self, key: &str, choices: &[&'static str]) -> Result<&'static str> {
        choice(self.value(key)?, &self.item(), key, choices)
    }

    fn list(&self, key: &str, most: usize) -> Result<&Vec<Value>> {
        match self.value(key)? {
            Value::Array(items) if !items.is_empty() && items.len() <= most => Ok(items),
            _ => fact_invalid(format!(
                "{}: {key} must be a list of 1 to {most} objects.",
                self.item()
            )),
        }
    }
}

fn fact_invalid<T>(message: String) -> Result<T> {
    refuse("ato_interest_penalties.fact_invalid", message)
}

/// A whole number of cents from a checked amount (at most 2 decimal places).
fn to_cents(value: &Decimal) -> Result<u64> {
    let text = value.quantize(-2, Rounding::HalfUp)?.to_string();
    Ok(text
        .replace('.', "")
        .parse()
        .expect("cents of a checked amount"))
}

fn dollars(cents: u128) -> Decimal {
    d(&format!("{}.{:02}", cents / 100, cents % 100))
}

fn date(value: &Value, item: &str, key: &str) -> Result<i64> {
    match value.as_str().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => fact_invalid(format!("{item}: {key} must be a date YYYY-MM-DD.")),
    }
}

fn flag(value: &Value, item: &str, key: &str) -> Result<bool> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => refuse(
            "input.flag_invalid",
            format!("{item}: {key} must be a boolean."),
        ),
    }
}

fn choice(value: &Value, item: &str, key: &str, choices: &[&'static str]) -> Result<&'static str> {
    match choices.iter().find(|c| value.as_str() == Some(**c)) {
        Some(found) => Ok(found),
        None => fact_invalid(format!(
            "{item}: {key} must be one of {}.",
            choices.join(", ")
        )),
    }
}

/// The financial year a day falls in, as `2026-27`.
fn financial_year(day: i64) -> String {
    let (y, m, _) = dates::to_civil(day);
    let start = if m >= 7 { y } else { y - 1 };
    format!("{start}-{:02}", (start + 1) % 100)
}

/// `year` must be the financial year of the question's end date `key`.
fn in_year(day: i64, period: &str, key: &str) -> Result<()> {
    let found = financial_year(day);
    if found == period {
        return Ok(());
    }
    refuse(
        "ato_interest_penalties.year_mismatch",
        format!(
            "{key} ({}) is in {found}; year must be the financial year of {key}.",
            dates::format(day)
        ),
    )
}

fn date_order<T>(message: String) -> Result<T> {
    refuse("ato_interest_penalties.date_order", message)
}

fn unpublished<T>(message: String) -> Result<T> {
    refuse("ato_interest_penalties.rate_unpublished", message)
}

/// Hundredths of 1% as a percentage string: 1151 is 11.51.
fn percent(hundredths: u32) -> String {
    format!("{}.{:02}", hundredths / 100, hundredths % 100)
}

fn year_length(day: i64) -> u32 {
    let (y, _, _) = dates::to_civil(day);
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    if leap { 366 } else { 365 }
}

/// The first day of `day`'s calendar quarter.
fn quarter_start(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    let first = (m - 1) / 3 * 3 + 1;
    dates::parse(&format!("{y:04}-{first:02}-01")).expect("quarter start")
}

/// Consecutive days at one annual rate within one calendar quarter.
struct Stretch {
    from: i64,
    to: i64,
    rate: u32,
    assumed: bool,
}

fn add_day(stretches: &mut Vec<Stretch>, day: i64, rate: u32, assumed: bool) {
    match stretches.last_mut() {
        Some(last)
            if last.to + 1 == day
                && last.rate == rate
                && last.assumed == assumed
                && quarter_start(last.from) == quarter_start(day) =>
        {
            last.to = day;
        }
        _ => stretches.push(Stretch {
            from: day,
            to: day,
            rate,
            assumed,
        }),
    }
}

fn stretches_json(stretches: &[Stretch]) -> Value {
    Value::Array(
        stretches
            .iter()
            .map(|s| {
                let mut out = json!({
                    "from": dates::format(s.from),
                    "to": dates::format(s.to),
                    "days": s.to - s.from + 1,
                    "annual_rate_percent": percent(s.rate),
                });
                if s.assumed {
                    out["assumed"] = json!(true);
                }
                out
            })
            .collect(),
    )
}

/// The `payments` list of `{<date_key>, amount}` objects, in the order given.
fn dated_amounts(f: &Facts, date_key: &str) -> Result<Vec<(i64, u64)>> {
    let mut out = Vec::new();
    for (i, item) in f.list("payments", 1000)?.iter().enumerate() {
        let name = format!("payment {}", i + 1);
        let Value::Object(fields) = item else {
            return fact_invalid(format!("{name} must be an object."));
        };
        stated::known_keys(fields, &name, "a payment", &[date_key, "amount"])?;
        let day = date(stated::required(fields, &name, date_key)?, &name, date_key)?;
        out.push((day, to_cents(&stated::amount(fields, &name, "amount")?)?));
    }
    Ok(out)
}

fn remission_note(out: &mut Value) {
    out["remission"] = json!(REMISSION);
}

pub(super) fn warn(out: &mut Value, text: String) {
    out["warnings"]
        .as_array_mut()
        .expect("warnings")
        .push(json!(text));
}

/// The command: `question` picks the charge, penalty or interest.
pub fn ato_interest_penalties(
    question: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    facts: &Map<String, Value>,
) -> Result<Value> {
    let period = scope(scope_confirmed, year, KIND)?;
    let question = match question.text() {
        Some("remission") => {
            return refuse(
                "ato_interest_penalties.remission_discretionary",
                "Remission of GIC (TAA 1953 s 8AAG), SIC (TAA Sch 1 s 280-160) and administrative penalties (s 298-20) is the Commissioner's discretion; every amount this command gives is before remission, and it does not estimate one.",
            );
        }
        Some("scheme_penalty") => {
            return refuse(
                "ato_interest_penalties.scheme_penalty",
                "Scheme penalties (TAA Sch 1 Subdiv 284-C) turn on the scheme's dominant purpose and whether the position was reasonably arguable; this command does not work them.",
            );
        }
        given => match QUESTIONS.iter().find(|q| given == Some(**q)) {
            Some(found) => *found,
            None => {
                return refuse(
                    "ato_interest_penalties.question_invalid",
                    format!("question must be one of {}.", QUESTIONS.join(", ")),
                );
            }
        },
    };
    let f = Facts {
        question,
        map: facts,
    };
    match question {
        "gic" => charges::gic(&f, &period),
        "sic" => charges::sic(&f, &period),
        "failure_to_lodge" => penalties::failure_to_lodge(&f, &period),
        "statement_penalty" => penalties::statement_penalty(&f, &period),
        "early_payment_interest" => credit::early_payment_interest(&f, &period),
        _ => credit::overpayment_interest(&f, &period),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_is_in_its_financial_year() {
        let day = |s| dates::parse(s).unwrap();
        assert_eq!(financial_year(day("2026-06-30")), "2025-26");
        assert_eq!(financial_year(day("2026-07-01")), "2026-27");
        assert_eq!(financial_year(day("2099-07-01")), "2099-00");
    }
}
