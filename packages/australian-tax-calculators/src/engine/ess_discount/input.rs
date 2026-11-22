// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Reading `ess_discount`'s interests: each fact checked once, and a fact the
//! interest's other facts do not call for refused.

use serde_json::{Map, Value};

use super::valuation;
use crate::dates;
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, refuse, stated};

/// The most interest parcels one call takes.
const MOST: usize = 50;
/// The first acquisition the rules from 1 July 2015 govern.
const FIRST_ACQUISITION: &str = "2015-07-01";

const INVALID: &str = "ess_discount.interests_invalid";

/// One parcel's stated facts, each read once.
pub(super) struct Reader<'a> {
    pub name: String,
    fields: &'a Map<String, Value>,
    taken: Vec<&'static str>,
}

impl<'a> Reader<'a> {
    fn new(name: String, value: &'a Value) -> Result<Self> {
        match value {
            Value::Object(fields) => Ok(Self {
                name,
                fields,
                taken: Vec::new(),
            }),
            _ => refuse(INVALID, format!("{name} must be an object.")),
        }
    }

    pub fn invalid<T>(&self, key: &str, expected: &str) -> Result<T> {
        refuse(
            "ess_discount.interests_invalid",
            format!("{}: {key} must be {expected}.", self.name),
        )
    }

    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    pub fn value(&mut self, key: &'static str) -> Result<&'a Value> {
        self.taken.push(key);
        stated::required(self.fields, &self.name, key)
    }

    pub fn amount(&mut self, key: &'static str) -> Result<Decimal> {
        self.taken.push(key);
        stated::amount(self.fields, &self.name, key)
    }

    pub fn flag(&mut self, key: &'static str) -> Result<bool> {
        match self.value(key)? {
            Value::Bool(b) => Ok(*b),
            _ => self.invalid(key, "a boolean"),
        }
    }

    /// A date, or `None` for `null` where `nullable`.
    pub fn day(&mut self, key: &'static str, nullable: bool) -> Result<Option<i64>> {
        match self.value(key)? {
            Value::Null if nullable => Ok(None),
            Value::String(text) if dates::parse(text).is_some() => Ok(dates::parse(text)),
            _ => self.invalid(
                key,
                if nullable {
                    "a YYYY-MM-DD date or null"
                } else {
                    "a YYYY-MM-DD date"
                },
            ),
        }
    }

    pub fn choice(&mut self, key: &'static str, allowed: &[&'static str]) -> Result<&'static str> {
        let given = self.value(key)?.as_str();
        match allowed.iter().find(|a| Some(**a) == given) {
            Some(found) => Ok(found),
            None => self.invalid(key, &format!("one of {}", allowed.join(", "))),
        }
    }

    pub fn object(&mut self, key: &'static str) -> Result<Option<Reader<'a>>> {
        let name = format!("{} {key}", self.name);
        match self.value(key)? {
            Value::Null => Ok(None),
            value => Reader::new(name, value).map(Some),
        }
    }

    /// Refuses a key not read, `why` saying what calls for the facts read.
    pub fn finish(&self, why: &str) -> Result<()> {
        match self
            .fields
            .keys()
            .find(|k| !self.taken.contains(&k.as_str()))
        {
            Some(key) => refuse(
                "input.field_unknown",
                format!("{}: {key} is not an input of {why}", self.name),
            ),
            None => Ok(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Share,
    Right,
}

/// A right's terms, as totals for the parcel.
pub(super) struct RightTerms {
    pub exercise_price: Decimal,
    pub share_market_value: Decimal,
    pub last_exercise_day: i64,
}

pub(super) struct StartUp {
    pub unlisted: bool,
    pub under_ten_years: bool,
    pub turnover: Decimal,
    pub employer_resident: bool,
}

/// The scheme and personal conditions, each the subsection it states.
pub(super) struct Conditions {
    pub employed: bool,
    pub ordinary_shares_only: bool,
    pub not_share_trading_company: bool,
    pub minimum_holding_period: bool,
    pub ten_percent_limit: bool,
    pub non_discriminatory: bool,
    pub real_risk: bool,
    /// Shares only: s 83A-105(2).
    pub broad_availability: bool,
    /// s 83A-105(4)(a), (b)(ii) and (iii); a right's counts only toward the
    /// $5,000 of s 83A-105(4)(c).
    pub salary_sacrifice: bool,
    /// Rights only: s 83A-105(6)(b).
    pub deferral_stated: bool,
    pub start_up: Option<StartUp>,
}

pub(super) struct Held<'a> {
    pub consideration: Decimal,
    pub value: Decimal,
    pub valuation: Value,
    pub right: Option<RightTerms>,
    pub conditions: Conditions,
    pub reader: Reader<'a>,
}

pub(super) struct Forfeiture {
    pub cause: &'static str,
    pub protects_against_fall: bool,
    /// Valuable consideration for the loss: a disposal, not a forfeiture.
    pub consideration_received: bool,
    pub amount_included: Decimal,
    pub year_included: String,
}

pub(super) enum State<'a> {
    Held(Box<Held<'a>>),
    Forfeited(Forfeiture),
}

pub(super) struct Interest<'a> {
    pub number: usize,
    pub kind: Kind,
    pub acquired: i64,
    pub state: State<'a>,
}

pub(super) fn interests(given: &Arg) -> Result<Vec<Interest<'_>>> {
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            INVALID,
            format!("interests must be a list of 1 to {MOST} objects."),
        );
    };
    if items.is_empty() || items.len() > MOST {
        return refuse(
            INVALID,
            format!("interests must be a list of 1 to {MOST} objects."),
        );
    }
    items
        .iter()
        .enumerate()
        .map(|(i, item)| interest(i + 1, item))
        .collect()
}

fn interest(number: usize, item: &Value) -> Result<Interest<'_>> {
    let mut reader = Reader::new(format!("interest {number}"), item)?;
    let kind = match reader.choice("kind", &["share", "right"])? {
        "share" => Kind::Share,
        _ => Kind::Right,
    };
    let acquired = reader.day("acquired", false)?.expect("not nullable");
    if acquired < dates::parse(FIRST_ACQUISITION).expect("date") {
        return refuse(
            "ess_discount.acquired_before_2015",
            format!(
                "{}: acquired before {FIRST_ACQUISITION}; such interests have a 7-year deferral, no start-up concession and a narrower forfeiture rule, which this worksheet does not apply.",
                reader.name
            ),
        );
    }
    let state = if reader.has("forfeiture") {
        State::Forfeited(forfeiture(&mut reader, kind, acquired)?)
    } else {
        State::Held(Box::new(held(reader, kind, acquired)?))
    };
    Ok(Interest {
        number,
        kind,
        acquired,
        state,
    })
}

fn held(mut reader: Reader<'_>, kind: Kind, acquired: i64) -> Result<Held<'_>> {
    let consideration = reader.amount("consideration")?;
    let right = match kind {
        Kind::Right => Some(right_terms(&mut reader, acquired)?),
        Kind::Share => None,
    };
    let regulations = match &right {
        Some(_) => reader.choice("valuation", &["market_value", "regulations"])? == "regulations",
        None => false,
    };
    let (value, valuation) = match (&right, regulations) {
        (Some(terms), true) => {
            let (value, detail) = valuation::right_value(
                &terms.share_market_value,
                &terms.exercise_price,
                acquired,
                terms.last_exercise_day,
                acquired,
            )?;
            (value, detail)
        }
        _ => (reader.amount("market_value")?, Value::Null),
    };
    let mut conditions = reader.object("conditions")?.map_or_else(
        || reader.invalid("conditions", "an object"),
        |c| conditions(c, kind),
    )?;
    conditions.start_up = start_up(&mut reader)?;
    Ok(Held {
        consideration,
        value,
        valuation,
        right,
        conditions,
        reader,
    })
}

fn right_terms(reader: &mut Reader<'_>, acquired: i64) -> Result<RightTerms> {
    let exercise_price = reader.amount("exercise_price")?;
    let share_market_value = reader.amount("share_market_value")?;
    let last = reader
        .day("last_exercise_day", false)?
        .expect("not nullable");
    if last < acquired {
        return refuse(
            "ess_discount.dates_inconsistent",
            format!("{}: last_exercise_day is before acquired.", reader.name),
        );
    }
    Ok(RightTerms {
        exercise_price,
        share_market_value,
        last_exercise_day: last,
    })
}

fn conditions(mut c: Reader<'_>, kind: Kind) -> Result<Conditions> {
    let share = kind == Kind::Share;
    let shares_only = |c: &mut Reader<'_>, key| if share { c.flag(key) } else { Ok(false) };
    let out = Conditions {
        employed: c.flag("employed")?,
        ordinary_shares_only: c.flag("ordinary_shares_only")?,
        not_share_trading_company: c.flag("not_share_trading_company")?,
        minimum_holding_period: c.flag("minimum_holding_period")?,
        ten_percent_limit: c.flag("ten_percent_limit")?,
        non_discriminatory: c.flag("non_discriminatory")?,
        real_risk: c.flag("real_risk_of_forfeiture")?,
        broad_availability: shares_only(&mut c, "broad_availability")?,
        salary_sacrifice: c.flag("salary_sacrifice")?,
        deferral_stated: if share {
            false
        } else {
            c.flag("deferral_stated")?
        },
        start_up: None,
    };
    c.finish(if share {
        "the conditions of a share"
    } else {
        "the conditions of a right"
    })?;
    Ok(out)
}

fn start_up(reader: &mut Reader<'_>) -> Result<Option<StartUp>> {
    let Some(mut s) = reader.object("start_up")? else {
        return Ok(None);
    };
    let out = StartUp {
        unlisted: s.flag("unlisted")?,
        under_ten_years: s.flag("incorporated_under_10_years")?,
        turnover: s.amount("aggregated_turnover")?,
        employer_resident: s.flag("employer_australian_resident")?,
    };
    s.finish("start_up")?;
    Ok(Some(out))
}

/// The causes of a forfeiture or loss the command takes.
const CAUSES: [&str; 5] = [
    "scheme_condition",
    "ceased_employment",
    "lapsed_unexercised",
    "cancellation_allowed",
    "other_choice",
];

fn forfeiture(reader: &mut Reader<'_>, kind: Kind, acquired: i64) -> Result<Forfeiture> {
    let Some(mut f) = reader.object("forfeiture")? else {
        return reader.invalid("forfeiture", "an object");
    };
    let cause = f.choice("cause", &CAUSES)?;
    if kind == Kind::Share && matches!(cause, "lapsed_unexercised" | "cancellation_allowed") {
        return f.invalid(
            "cause",
            "scheme_condition, ceased_employment or other_choice for a share",
        );
    }
    let out = Forfeiture {
        cause,
        protects_against_fall: f.flag("protects_against_fall")?,
        consideration_received: f.flag("consideration_received")?,
        amount_included: f.amount("amount_included")?,
        year_included: included_year(&mut f, acquired)?,
    };
    f.finish("forfeiture")?;
    reader.finish("a forfeited interest")?;
    Ok(out)
}

/// The income year the amount was included in: from the acquisition's year to 2026-27.
fn included_year(f: &mut Reader<'_>, acquired: i64) -> Result<String> {
    let first: i64 = super::income_year(acquired)[..4].parse().expect("year");
    let given = f
        .value("year_included")?
        .as_str()
        .unwrap_or_default()
        .to_owned();
    if (first..=2026).any(|y| given == format!("{y}-{:02}", (y + 1) % 100)) {
        Ok(given)
    } else {
        f.invalid(
            "year_included",
            &format!(
                "an income year from {} to 2026-27",
                super::income_year(acquired)
            ),
        )
    }
}
