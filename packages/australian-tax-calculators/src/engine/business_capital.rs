// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `business_capital_expenditure`: business capital expenditure (black hole
//! and start-up costs) under ITAA 1997 s 40-880, item by item. Each item is
//! deducted in equal proportions over 5 income years from the year incurred
//! (s 40-880(2)), or in full that year as a start-up cost (s 40-880(2A)),
//! to the extent s 40-880(3) or (4) allows and unless s 40-880(4)(b), (5),
//! (7), (8) or (9) excludes it. The citations are to ITAA 1997 compilation 268
//! (1 October 2026), in which s 40-880 was last amended by Act No 92 of 2020,
//! read on 16 November 2026.

use serde_json::{Map, Value};

use super::{
    Arg, Result, derived_result, finite_between, int, money, precise, rate, refuse, scope,
};
use crate::decimal::Decimal;

mod worked;

const KIND: &str = "business_capital_expenditure";

/// Act No 92 of 2020, Schedule 3 item 40(7): the current s 40-880, with
/// s 40-880(2B), applies to capital expenditure incurred from 1 July 2020.
const FIRST_INCURRED: i64 = 2020;

/// Each paragraph an item may state excludes it: (label, citation). Those of
/// s 40-880(5)(a) and (f) refuse instead, as the expenditure belongs to
/// Division 40 or the CGT provisions.
const EXCLUSIONS: [(&str, &str); 13] = [
    (
        "5(a)",
        "ITAA 1997 s 40-880(5)(a): it forms part of the cost of a depreciating asset you hold, used to hold or will hold",
    ),
    (
        "5(b)",
        "ITAA 1997 s 40-880(5)(b): you can deduct an amount for it under another provision",
    ),
    (
        "5(c)",
        "ITAA 1997 s 40-880(5)(c): it forms part of the cost of land",
    ),
    (
        "5(d)",
        "ITAA 1997 s 40-880(5)(d): it is in relation to a lease or other legal or equitable right",
    ),
    (
        "5(e)",
        "ITAA 1997 s 40-880(5)(e): it would otherwise be taken into account in working out a profit included in assessable income or a loss you can deduct",
    ),
    (
        "5(f)",
        "ITAA 1997 s 40-880(5)(f): it could otherwise be taken into account in working out a capital gain or capital loss from a CGT event",
    ),
    (
        "5(g)",
        "ITAA 1997 s 40-880(5)(g): another provision would expressly make it non-deductible if it were not capital",
    ),
    (
        "5(h)",
        "ITAA 1997 s 40-880(5)(h): another provision expressly prevents it being taken into account as s 40-880(5)(a) to (f) describe, for a reason other than its being capital",
    ),
    (
        "5(i)",
        "ITAA 1997 s 40-880(5)(i): it is of a private or domestic nature",
    ),
    (
        "5(j)",
        "ITAA 1997 s 40-880(5)(j): it is incurred in relation to gaining or producing exempt income or non-assessable non-exempt income",
    ),
    (
        "8",
        "ITAA 1997 s 40-880(8): a market value substitution rule excluded it from the cost of a depreciating asset or the cost base or reduced cost base of a CGT asset",
    ),
    (
        "9(a)",
        "ITAA 1997 s 40-880(9)(a): it is incurred by way of returning an amount you received that was not included in your assessable income",
    ),
    (
        "9(b)",
        "ITAA 1997 s 40-880(9)(b): for another entity it is a return on or of an equity interest, or of a debt interest that is your obligation",
    ),
];

/// The paragraph of s 40-880(2) each kind of business falls in.
const BUSINESSES: [(&str, &str); 4] = [
    (
        "current",
        "ITAA 1997 s 40-880(2)(a): capital expenditure in relation to your business",
    ),
    (
        "former",
        "ITAA 1997 s 40-880(2)(b): capital expenditure in relation to a business that used to be carried on",
    ),
    (
        "proposed",
        "ITAA 1997 s 40-880(2)(c): capital expenditure in relation to a business proposed to be carried on",
    ),
    (
        "wind_up",
        "ITAA 1997 s 40-880(2)(d): capital expenditure to liquidate or deregister a company of which you were a member, or to wind up a partnership or trust in which you were a partner or beneficiary, that carried on a business",
    ),
];

const OWN_LIMIT: &str = "ITAA 1997 s 40-880(3): to the extent the business you carry on, used to carry on or propose to carry on is, was or is proposed to be carried on for a taxable purpose";
const OTHER_LIMIT: &str = "ITAA 1997 s 40-880(4)(a): to the extent the business another entity used to carry on or proposes to carry on was or is proposed to be carried on for a taxable purpose";
const NOT_CONNECTED: &str = "ITAA 1997 s 40-880(4)(b): it is not in connection with your deriving assessable income from the business and with the business itself";
const NOT_REASONABLE_TIME: &str = "ITAA 1997 s 40-880(7): it is not reasonable to conclude that the business is proposed to be carried on within a reasonable time";
const GOODWILL: &str = "ITAA 1997 s 40-880(6): s 40-880(5)(d) and (f) do not apply, as the expenditure preserves (but does not enhance) the value of goodwill and is in relation to a right whose value is solely attributable to its effect on goodwill";
const FIVE_YEARS: &str = "ITAA 1997 s 40-880(2): in equal proportions over 5 income years starting in the year incurred, one fifth a year, not apportioned for part of a year (TR 2011/6 para 10, the Commissioner's view)";

/// s 40-880(2A)(b): the start-up expenditure that can be deducted at once.
const START_UP: [(&str, &str); 2] = [
    (
        "advice_or_services",
        "advice or services relating to the proposed structure or operation of the business (s 40-880(2A)(b)(i))",
    ),
    (
        "government_fees",
        "fees, taxes or charges paid to an Australian government agency relating to establishing the business or its operating structure (s 40-880(2A)(b)(ii))",
    ),
];

/// s 40-880(2A)(c): who can deduct it at once, for the income year incurred.
const ENTITY_TESTS: [(&str, &str); 3] = [
    (
        "small_business_entity",
        "a small business entity (s 40-880(2A)(c))",
    ),
    (
        "covered_by_2b",
        "an entity covered by s 40-880(2B), which would be a small business entity at $50 million (s 40-880(2A)(c))",
    ),
    (
        "not_in_business_and_unconnected",
        "an entity carrying on no business, not connected with or an affiliate of one that carries on a business and is neither a small business entity nor covered by s 40-880(2B) (s 40-880(2A)(c)(i) and (ii))",
    ),
];

/// How an item is deducted once it is deductible.
enum Method {
    /// s 40-880(2A): in full in the income year incurred, with the reason.
    Immediate(String),
    /// s 40-880(2): a fifth a year over 5 income years.
    FiveYears,
}

/// One item of capital expenditure as the worksheet reads it.
struct Item {
    amount: Decimal,
    /// The calendar year the income year incurred starts in.
    incurred: i64,
    business: &'static str,
    limit: &'static str,
    taxable_purpose: Decimal,
    /// The citations that leave nothing to deduct; empty when none applies.
    excluded_by: Vec<&'static str>,
    method: Method,
    notes: Vec<String>,
}

/// `2026-27` for 2026.
pub(super) fn year_text(first: i64) -> String {
    format!("{first}-{:02}", (first + 1) % 100)
}

/// 2026 for `2026-27`; `None` for text that is not an income year.
pub(super) fn year_first(text: &str) -> Option<i64> {
    let first: i64 = text.get(..4)?.parse().ok()?;
    (year_text(first) == text).then_some(first)
}

fn missing<T>(n: usize, key: &str, why: &str) -> Result<T> {
    refuse(
        "business_capital.fact_missing",
        format!("item {n}: {key} is required: {why}."),
    )
}

fn invalid<T>(n: usize, key: &str, expected: &str) -> Result<T> {
    refuse(
        "business_capital.fact_invalid",
        format!("item {n}: {key} must be {expected}."),
    )
}

/// The stated facts of one item, each read once and named in a refusal.
struct Facts<'a> {
    n: usize,
    fields: &'a Map<String, Value>,
    taken: Vec<&'static str>,
}

impl Facts<'_> {
    fn get(&mut self, key: &'static str, why: &str) -> Result<&Value> {
        self.taken.push(key);
        match self.fields.get(key) {
            Some(value) => Ok(value),
            None => missing(self.n, key, why),
        }
    }

    fn flag(&mut self, key: &'static str, why: &str) -> Result<bool> {
        let n = self.n;
        match self.get(key, why)? {
            Value::Bool(b) => Ok(*b),
            _ => invalid(n, key, "a JSON boolean"),
        }
    }

    /// One of `options`' names, returned with its text.
    fn choice(
        &mut self,
        key: &'static str,
        options: &[(&'static str, &'static str)],
        why: &str,
    ) -> Result<(&'static str, &'static str)> {
        let n = self.n;
        let given = self.get(key, why)?.as_str();
        match options.iter().find(|(name, _)| Some(*name) == given) {
            Some(found) => Ok(*found),
            None => {
                let names: Vec<&str> = options.iter().map(|(name, _)| *name).collect();
                invalid(n, key, &format!("one of {}", names.join(", ")))
            }
        }
    }

    fn decimal(&mut self, key: &'static str) -> Result<Option<Arg>> {
        let value = self.get(key, "every item states it")?;
        Ok(value
            .as_str()
            .and_then(|text| Decimal::parse(text).ok())
            .map(Arg::Decimal))
    }

    /// A refusal for any fact given that this item does not take.
    fn none_left(&self) -> Result<()> {
        match self
            .fields
            .keys()
            .find(|key| !self.taken.contains(&key.as_str()))
        {
            Some(key) => refuse(
                "input.field_unknown",
                format!(
                    "item {}: {key} is not a fact this item takes beside its other facts",
                    self.n
                ),
            ),
            None => Ok(()),
        }
    }
}

/// `amount`, `incurred_in` and `capital`: what every item states first.
fn expenditure(facts: &mut Facts, year: i64) -> Result<(Decimal, i64)> {
    let n = facts.n;
    let amount = match facts.decimal("amount")? {
        Some(arg) => money(&arg).ok(),
        None => None,
    };
    let Some(amount) = amount else {
        return refuse(
            "input.amount_invalid",
            format!(
                "item {n}: amount must be a finite non-negative decimal string, at most 2dp and 1e12."
            ),
        );
    };
    let incurred = facts.get("incurred_in", "every item states it")?;
    let Some(incurred) = incurred.as_str().and_then(year_first) else {
        return invalid(n, "incurred_in", "an income year such as \"2025-26\"");
    };
    if incurred < FIRST_INCURRED {
        return refuse(
            "business_capital.incurred_before_july_2020",
            format!(
                "item {n}: expenditure incurred before 1 July 2020 is out of scope: s 40-880 as amended by Act No 92 of 2020 applies from then, and this worksheet does not hold the earlier form."
            ),
        );
    }
    if incurred > year {
        return refuse(
            "business_capital.incurred_after_year",
            format!(
                "item {n}: it was incurred in {}, after the {} income year asked about.",
                year_text(incurred),
                year_text(year)
            ),
        );
    }
    if !facts.flag(
        "capital",
        "the operator establishes that the outlay is capital expenditure",
    )? {
        return refuse(
            "business_capital.not_capital",
            format!(
                "item {n}: s 40-880 deducts capital expenditure only; a revenue outlay is a question for s 8-1, which this worksheet does not decide."
            ),
        );
    }
    Ok((amount, incurred))
}

/// `exclusions`, refusing s 40-880(5)(a) and (f), with the citations of the
/// rest and whether s 40-880(6) was applied.
fn exclusions(facts: &mut Facts) -> Result<(Vec<&'static str>, bool)> {
    let n = facts.n;
    let expected = "a list of the paragraphs of s 40-880 that exclude the expenditure, each once, from \"5(a)\" to \"5(j)\", \"8\", \"9(a)\" and \"9(b)\"; [] when none does";
    let Value::Array(given) = facts.get(
        "exclusions",
        "every item states the exclusions in s 40-880(5), (8) and (9) that apply, [] for none",
    )?
    else {
        return invalid(n, "exclusions", expected);
    };
    let mut labels: Vec<&'static str> = Vec::new();
    for value in given {
        let found = EXCLUSIONS
            .iter()
            .find(|(label, _)| Some(*label) == value.as_str());
        match found {
            Some(&(label, _)) if !labels.contains(&label) => labels.push(label),
            _ => return invalid(n, "exclusions", expected),
        }
    }
    if labels.contains(&"5(a)") {
        return refuse(
            "business_capital.depreciating_asset_cost",
            format!(
                "item {n}: s 40-880(5)(a) excludes expenditure that forms part of the cost of a depreciating asset; it is deducted as the asset's decline in value under Division 40 (s 40-25, with its cost under Subdivision 40-C), which this worksheet does not work out."
            ),
        );
    }
    let goodwill = labels.iter().any(|label| matches!(*label, "5(d)" | "5(f)"))
        && facts.flag(
            "goodwill_right",
            "s 40-880(6) decides whether s 40-880(5)(d) and (f) apply to expenditure preserving goodwill",
        )?;
    if labels.contains(&"5(f)") && !goodwill {
        return refuse(
            "business_capital.cgt_amount",
            format!(
                "item {n}: s 40-880(5)(f) excludes expenditure that could be taken into account in working out a capital gain or loss; the CGT provisions take it, in the cost base (s 110-25), reduced cost base (s 110-55) or capital proceeds (s 116-20), which this worksheet does not work out."
            ),
        );
    }
    let cited = EXCLUSIONS
        .iter()
        .filter(|(label, _)| {
            labels.contains(label) && !(goodwill && matches!(*label, "5(d)" | "5(f)"))
        })
        .map(|(_, citation)| *citation)
        .collect();
    Ok((cited, goodwill))
}

/// s 40-880(2A): whether a proposed business's expenditure is deducted at
/// once, with the reason either way.
fn start_up(facts: &mut Facts) -> Result<(Method, Option<String>)> {
    let neither = [("neither", "")];
    let kinds = [START_UP[0], START_UP[1], neither[0]];
    let (kind, kind_text) = facts.choice(
        "start_up_expense",
        &kinds,
        "s 40-880(2A)(b) decides whether a proposed business's expenditure is deducted at once",
    )?;
    if kind == "neither" {
        let note = "s 40-880(2A)(b) is not met: the expenditure is neither advice or services nor a government agency's fee, tax or charge, so s 40-880(2) spreads it";
        return Ok((Method::FiveYears, Some(note.into())));
    }
    let none = [("none", "")];
    let tests = [ENTITY_TESTS[0], ENTITY_TESTS[1], ENTITY_TESTS[2], none[0]];
    let (test, test_text) = facts.choice(
        "entity_test",
        &tests,
        "s 40-880(2A)(c) decides who deducts start-up expenditure at once",
    )?;
    if test == "none" {
        let note =
            "s 40-880(2A)(c) is not met for the income year incurred, so s 40-880(2) spreads it";
        return Ok((Method::FiveYears, Some(note.into())));
    }
    let reason = format!(
        "ITAA 1997 s 40-880(2A): all of it in the income year incurred, as {kind_text} for a business proposed to be carried on, incurred by {test_text} for that year"
    );
    Ok((Method::Immediate(reason), None))
}

/// The business, its limit in s 40-880(3) or (4) and the facts only some
/// businesses state: who carries it on, s 40-880(4)(b), s 40-880(7) and
/// s 40-880(2A).
fn business(facts: &mut Facts, item: &mut Item) -> Result<()> {
    let (kind, paragraph) = facts.choice("business", &BUSINESSES, "every item states it")?;
    item.business = paragraph;
    let another = match kind {
        "former" | "proposed" => {
            let who = [("you", ""), ("another_entity", "")];
            let why = "s 40-880(3) limits your own former or proposed business and s 40-880(4) another entity's";
            facts.choice("carried_on_by", &who, why)?.0 == "another_entity"
        }
        // The company, partnership or trust wound up carried on the business.
        "wind_up" => true,
        _ => false,
    };
    item.limit = if another { OTHER_LIMIT } else { OWN_LIMIT };
    let why = "s 40-880(4)(b) limits expenditure on another entity's business";
    if another && !facts.flag("assessable_income_connection", why)? {
        item.excluded_by.push(NOT_CONNECTED);
    }
    if kind == "proposed" {
        let why = "s 40-880(7) limits expenditure on a proposed business";
        if !facts.flag("within_reasonable_time", why)? {
            item.excluded_by.push(NOT_REASONABLE_TIME);
        }
        let (method, note) = start_up(facts)?;
        item.method = method;
        item.notes.extend(note);
    }
    Ok(())
}

/// One item from its stated facts, refusing a fact missing, invalid or not
/// taken, and the cases out of scope.
fn read_item(n: usize, value: &Value, year: i64) -> Result<Item> {
    let Value::Object(fields) = value else {
        return items_invalid();
    };
    let mut facts = Facts {
        n,
        fields,
        taken: Vec::new(),
    };
    let (amount, incurred) = expenditure(&mut facts, year)?;
    let mut item = Item {
        amount,
        incurred,
        business: "",
        limit: "",
        taxable_purpose: int(1),
        excluded_by: Vec::new(),
        method: Method::FiveYears,
        notes: Vec::new(),
    };
    business(&mut facts, &mut item)?;
    let share = facts.decimal("taxable_purpose")?;
    let share = match share {
        Some(arg) => finite_between(&arg, "0", "1")?,
        None => None,
    };
    let Some(share) = share else {
        return refuse(
            "input.fraction_invalid",
            format!("item {n}: taxable_purpose must be a finite decimal fraction from 0 to 1."),
        );
    };
    item.taxable_purpose = share;
    let (cited, goodwill) = exclusions(&mut facts)?;
    item.excluded_by.extend(cited);
    if goodwill {
        item.notes.push(GOODWILL.into());
    }
    facts.none_left()?;
    Ok(item)
}

fn items_invalid<T>() -> Result<T> {
    refuse(
        "business_capital.items_invalid",
        "items must be a non-empty list of objects, one for each item of capital expenditure.",
    )
}

pub fn business_capital_expenditure(
    items: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let first = year_first(&year).expect("a supported income year");
    let Arg::Json(Value::Array(list)) = items else {
        return items_invalid();
    };
    if list.is_empty() {
        return items_invalid();
    }
    let items = list
        .iter()
        .enumerate()
        .map(|(i, value)| read_item(i + 1, value, first))
        .collect::<Result<Vec<_>>>()?;
    let (steps, rows) = precise(|| worked::worked(&items, first))?;
    let mut out = derived_result(KIND, &year, steps, vec![rate("items", items.len())])?;
    out["items"] = Value::Array(rows);
    Ok(out)
}
