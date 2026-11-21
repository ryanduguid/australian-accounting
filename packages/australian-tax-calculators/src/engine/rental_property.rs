// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `rental_property`: one residential rental property's income and
//! deductions for one owner and income year, 2020-21 to 2026-27: rent and
//! expenses by the ATO worksheet's categories at the owner's legal interest,
//! the time-based share for private use, interest on a mixed-use loan,
//! borrowing expenses through `borrowing_expenses_schedule`, capital works
//! under Division 43, decline in value after s 40-27, the net rent and, for
//! 2024-25 to 2026-27, its effect on tax at the resident scale. The citations
//! are to ITAA 1997 ss 6-5, 8-1, 26-31, 40-27, 43-15, 43-20, 43-25, 43-210,
//! 43-215, 43-220, 43-230, 43-235 and 43-240 (the ATO's legal database, and
//! the amendment history of compilation 264), Act No. 126 of 2017 Schedule 2
//! item 13, TR 2000/2, TR 93/32 and the ATO's Rental properties guide 2026
//! (NAT 1729-06.2026), all read on 22 November 2026.

use std::borrow::Cow;

use serde_json::{Map, Value, json};

mod assets;
mod capital_works;

use super::{
    Arg, Result, Step, borrowing_expenses_schedule, cents, date_arg, derived_result,
    finite_between, flag, int, money, rate, refuse, resident_tax, scope, step, whole_dollars,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "rental_property";

/// The facts as the command passes them; see the README for each.
pub struct RentalPropertyArgs<'a> {
    pub ownership_share: &'a Arg,
    pub acquired: &'a Arg,
    pub disposed: &'a Arg,
    pub days_available: &'a Arg,
    pub section_26_50_applies: &'a Arg,
    pub income: &'a Arg,
    pub expenses: &'a Arg,
    pub rental_only_expenses: &'a Arg,
    pub loans: &'a Arg,
    pub borrowing_loans: &'a Arg,
    pub capital_works: &'a Arg,
    pub depreciating_assets: &'a Arg,
    pub other_taxable_income: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The income lines of the ATO's rental property worksheet.
const INCOME: [&str; 2] = ["rental_income", "other_rental_income"];

/// The expense lines of the ATO's rental property worksheet (Rental
/// properties guide 2026, Example 33) that are entered as amounts; interest,
/// borrowing expenses, decline in value and capital works have inputs of
/// their own.
pub const CATEGORIES: [&str; 15] = [
    "advertising_for_tenants",
    "body_corporate_fees",
    "cleaning",
    "council_rates",
    "gardening",
    "insurance",
    "land_tax",
    "legal_expenses",
    "pest_control",
    "property_agent_fees",
    "repairs_and_maintenance",
    "stationery_telephone_postage",
    "travel",
    "water_charges",
    "sundry",
];

/// The first income year the resident scale is held for.
const FIRST_TAX_YEAR: i64 = 2024;

const GROSS_RENT: &str = "ITAA 1997 s 6-5: rent and other rental-related income are ordinary income; co-owners not in a business of letting share it by legal interest (TR 93/32 paragraphs 6 and 11 to 13; Rental properties guide 2026, Examples 1 and 2)";
const EXPENSE: &str = "ITAA 1997 s 8-1(1)(a): deductible to the extent incurred in producing assessable income, at the owner's legal interest (TR 93/32); an expense of the property for the whole period held is apportioned by the days let or available for rent on commercial terms over the days held, and one relating solely to renting is not (Rental properties guide 2026, 'Time-based method' and Example 34)";
const TRAVEL: &str = "ITAA 1997 s 26-31(1): travel related to using residential premises as residential accommodation, not in a business, is not deductible, from 1 July 2017 (Act No. 126 of 2017 Schedule 1)";
const TRAVEL_DENIED: &str =
    "ITAA 1997 s 26-31(1): the travel given, at the owner's legal interest, that is not deducted";
const INTEREST: &str = "ITAA 1997 s 8-1; TR 2000/2 paragraphs 15 and 16: each loan's interest times the principal used for the rental over the total principal, repayments reducing both in proportion (Rental properties guide 2026, Examples 12 and 40), then the time-based share; a loan is the owner's own and is not divided by legal interest";
const BORROWING: &str = "ITAA 1997 s 25-25: the year's deduction across the owner's loans, as borrowing-expenses-schedule works it out for the loans given";
const DECLINE: &str = "ITAA 1997 s 40-25 and s 40-27(2) to (6), Act No. 126 of 2017 Schedule 2 item 13: each asset's decline in value given, at the owner's legal interest and the time-based share, nil where s 40-27 reduces it";
const CAPITAL_WORKS: &str = "ITAA 1997 ss 43-10, 43-15(1), 43-25, 43-210 and 43-215: for each item, the owner's share of the construction expenditure times the rate times the days used over 365, no more than the undeducted construction expenditure (ss 43-230, 43-235 and 43-240)";
const TOTAL: &str = "The rental deductions: the expense lines, interest, borrowing expenses, decline in value and capital works (Rental properties guide 2026, Example 33)";
const NET: &str = "Rental properties guide 2026, Example 33: gross rent less the rental deductions; a negative amount is a rental loss";
const TAXABLE_WITHOUT: &str = "The other taxable income given, without the rental result";
const TAXABLE_WITH: &str = "The other taxable income plus the net rent with its cents dropped, as a return shows it, and nil if that is negative (a tax loss under ITAA 1997 Division 36 is not worked out here)";
const TAX_WITHOUT: &str = "Income Tax Rates Act 1986 Schedule 7 Part I clause 1, as resident-tax works it out, on the taxable income without the rental result";
const TAX_WITH: &str = "Income Tax Rates Act 1986 Schedule 7 Part I clause 1, as resident-tax works it out, on the taxable income with the rental result";
const TAX_EFFECT: &str = "The tax at the resident scale without the rental result less the tax with it: positive is less tax. The Medicare levy, the surcharge, offsets and study loan repayments are not included";

/// The days of the income year that start and end it, and the year's first
/// calendar year.
pub(super) struct Year {
    pub first: i64,
    pub start: i64,
    pub end: i64,
}

impl Year {
    fn of(year: &str) -> Year {
        let first: i64 = year[..4].parse().expect("a supported income year");
        let day = |y: i64, md: &str| dates::parse(&format!("{y}-{md}")).expect("a valid date");
        Year {
            first,
            start: day(first, "07-01"),
            end: day(first + 1, "06-30"),
        }
    }
}

/// The days the owner held the property in the year and the days it was let
/// or available for rent on commercial terms.
pub(super) struct Held {
    pub from: i64,
    pub to: i64,
    pub acquired_in_year: bool,
    pub days: i64,
    pub available: i64,
}

pub(super) fn conflict<T>(message: impl Into<String>) -> Result<T> {
    refuse("rental_property.fact_conflict", message)
}

pub(super) fn invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("rental_property.fact_invalid", message)
}

/// A decimal string as an amount, refused as `money` refuses one.
/// An item's date field, refused unless it is a YYYY-MM-DD date.
pub(super) fn date(fields: &Map<String, Value>, key: &str) -> Result<i64> {
    match fields
        .get(key)
        .and_then(Value::as_str)
        .and_then(dates::parse)
    {
        Some(day) => Ok(day),
        None => invalid(format!("{key} must be a YYYY-MM-DD date.")),
    }
}

pub(super) fn amount(value: Option<&Value>) -> Result<Decimal> {
    let parsed = value
        .and_then(Value::as_str)
        .and_then(|text| Decimal::parse_input(text).ok());
    money(&parsed.map_or(Arg::Json(Value::Null), Arg::Decimal))
}

/// A boolean fact of a list item.
pub(super) fn item_flag(fields: &Map<String, Value>, key: &str) -> Result<Option<bool>> {
    match fields.get(key) {
        None => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => refuse("input.flag_invalid", format!("{key} must be a boolean.")),
    }
}

/// A list item's fields, refusing one it does not take or a required one
/// missing.
pub(super) fn item<'a>(
    value: &'a Value,
    list: &str,
    facts: &[(&str, bool)],
) -> Result<&'a Map<String, Value>> {
    let Value::Object(fields) = value else {
        return invalid(format!("each of {list} must be an object."));
    };
    if let Some(key) = fields.keys().find(|k| !facts.iter().any(|(f, _)| f == k)) {
        return refuse(
            "input.field_unknown",
            format!("{key} is not a fact {list} takes"),
        );
    }
    if let Some((key, _)) = facts
        .iter()
        .find(|(k, req)| *req && !fields.contains_key(*k))
    {
        return refuse("input.missing_field", format!("{list}: {key} is required"));
    }
    Ok(fields)
}

/// A list input, or a refusal naming it.
pub(super) fn list<'a>(value: &'a Arg, name: &str) -> Result<&'a [Value]> {
    match value {
        Arg::Json(Value::Array(items)) => Ok(items),
        _ => invalid(format!("{name} must be a list, empty if there are none.")),
    }
}

/// `label: ` before a refusal's message, its code kept.
pub(super) fn at<T>(label: &str, outcome: Result<T>) -> Result<T> {
    outcome.map_err(|error| match error {
        super::Error::Value { code, message } => super::Error::Value {
            code,
            message: format!("{label}: {message}"),
        },
        other => other,
    })
}

fn held(args: &RentalPropertyArgs, year: &Year) -> Result<Held> {
    let acquired = date_arg(args.acquired, "acquired")?;
    let disposed = match args.disposed {
        Arg::Json(Value::Null) => None,
        Arg::Json(Value::String(text)) => Some(date_arg(&Arg::Text(text.clone()), "disposed")?),
        _ => return invalid("disposed must be a YYYY-MM-DD date or null."),
    };
    if acquired > year.end {
        return conflict("acquired is after the income year.");
    }
    let to = disposed.unwrap_or(year.end);
    if disposed.is_some_and(|day| day < year.start || day > year.end || day < acquired) {
        return conflict("disposed must be in the income year and not before acquired.");
    }
    let from = acquired.max(year.start);
    let days = to - from + 1;
    let available = match args.days_available {
        Arg::Int(n) if (0..=days).contains(n) => *n,
        Arg::Int(_) => {
            return conflict(format!(
                "days_available must be from 0 to the {days} days the property was held in the year."
            ));
        }
        _ => return invalid("days_available must be a whole number of days."),
    };
    Ok(Held {
        from,
        to,
        acquired_in_year: acquired > year.start,
        days,
        available,
    })
}

/// The amounts an object gives by line, each refused as `money` refuses one,
/// and a line the worksheet does not have.
fn lines(value: &Arg, name: &str, keys: &[&str]) -> Result<Vec<(String, Decimal)>> {
    let Arg::Json(Value::Object(given)) = value else {
        return invalid(format!("{name} must be an object of amounts by line."));
    };
    let mut out = Vec::new();
    for (key, value) in given {
        if !keys.contains(&key.as_str()) {
            return refuse(
                "input.field_unknown",
                format!("{key} is not a line of {name}"),
            );
        }
        out.push((
            key.clone(),
            at(&format!("{name}.{key}"), amount(Some(value)))?,
        ));
    }
    Ok(out)
}

fn line<'a>(given: &'a [(String, Decimal)], key: &str) -> Option<&'a Decimal> {
    given.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn ownership(value: &Arg) -> Result<Decimal> {
    match finite_between(value, "0", "1")? {
        Some(share) if !share.is_zero() => Ok(share),
        _ => invalid("ownership_share must be a decimal fraction above 0 and at most 1."),
    }
}

/// Each expense line given: the apportioned part times the time-based share
/// plus the part relating solely to renting, at the legal interest; travel is
/// shown and denied.
fn expense_steps(
    apportioned: &[(String, Decimal)],
    rental_only: &[(String, Decimal)],
    share: &Decimal,
    held: &Held,
) -> Result<Vec<Step>> {
    let zero = int(0);
    let (days, available) = (int(held.days), int(held.available));
    let mut steps = Vec::new();
    for key in CATEGORIES {
        let (a, r) = (line(apportioned, key), line(rental_only, key));
        if a.is_none() && r.is_none() {
            continue;
        }
        let (a, r) = (a.unwrap_or(&zero), r.unwrap_or(&zero));
        if key == "travel" {
            let given = a.add(r)?;
            steps.push(step("travel", TRAVEL, "0 x travel", &[&given], int(0)));
            steps.push(step(
                "travel_denied",
                TRAVEL_DENIED,
                "cents(travel x ownership_share)",
                &[&given, share],
                cents(given.mul(share)?)?,
            ));
            continue;
        }
        let value = a
            .mul(share)?
            .mul(&available)?
            .div(&days)?
            .add(&r.mul(share)?)?;
        steps.push(step(
            Cow::Borrowed(key),
            EXPENSE,
            "cents(apportioned x ownership_share x days_available / days_held + rental_only x ownership_share)",
            &[a, share, &available, &days, r, share],
            cents(value)?,
        ));
    }
    Ok(steps)
}

/// A loan's interest facts.
const LOAN_FACTS: [(&str, bool); 4] = [
    ("interest", true),
    ("principal_for_rental", true),
    ("principal_total", true),
    ("redrawn_in_year", true),
];

/// The deductible interest on each loan, the time-based share applied, and
/// each loan's row.
fn interest(loans: &Arg, held: &Held) -> Result<(Step, Vec<Value>)> {
    let (available, days) = (int(held.available), int(held.days));
    let mut terms = Vec::new();
    let mut rows = Vec::new();
    let mut total = int(0);
    for (i, value) in list(loans, "loans")?.iter().enumerate() {
        let (charged, rental, whole) = at(&format!("loan {}", i + 1), loan(value))?;
        let deduction = cents(
            charged
                .mul(&rental)?
                .div(&whole)?
                .mul(&available)?
                .div(&days)?,
        )?;
        terms.push(format!(
            "cents({charged} x {rental} / {whole} x {} / {})",
            held.available, held.days
        ));
        rows.push(json!({
            "loan": i + 1,
            "interest": charged.to_string(),
            "income_producing_share": format!("{rental} / {whole}"),
            "deduction": deduction.to_string(),
        }));
        total = total.add(&deduction)?;
    }
    let formula = if terms.is_empty() {
        "0".to_string()
    } else {
        terms.join(" + ")
    };
    Ok((step("interest", INTEREST, &formula, &[], total), rows))
}

fn loan(value: &Value) -> Result<(Decimal, Decimal, Decimal)> {
    let fields = item(value, "a loan", &LOAN_FACTS)?;
    let charged = amount(fields.get("interest"))?;
    let rental = amount(fields.get("principal_for_rental"))?;
    let whole = amount(fields.get("principal_total"))?;
    if item_flag(fields, "redrawn_in_year")? == Some(true) {
        return refuse(
            "rental_property.interest_redraw",
            "A redraw or further drawing in the year changes the share of the principal used for the rental from that day (TR 2000/2 paragraphs 23 to 25); work the loan's sub-accounts and give each as a loan of its own.",
        );
    }
    if whole.is_zero() || rental.gt(&whole)? {
        return conflict(
            "principal_for_rental must not be more than principal_total, which must be above nil.",
        );
    }
    Ok((charged, rental, whole))
}

/// The year's borrowing expenses deduction from `borrowing_expenses_schedule`
/// on the loans given, with its loan rows; nil and no rows for no loans.
fn borrowing(loans: &Arg, year: &str) -> Result<(Step, Value)> {
    let list = list(loans, "borrowing_loans")?;
    if list.is_empty() {
        let none = step("borrowing_expenses", BORROWING, "0", &[], int(0));
        return Ok((none, json!([])));
    }
    let out =
        borrowing_expenses_schedule(loans, None, &Arg::Text(year.to_string()), &Arg::Bool(true))?;
    let deduction = Decimal::parse(out["amounts"]["deduction"].as_str().expect("an amount"))?;
    let step = step(
        "borrowing_expenses",
        BORROWING,
        "schedule_deduction",
        &[&deduction],
        deduction.clone(),
    );
    Ok((step, out["loans"].clone()))
}

/// The resident scale's tax on a whole-dollar taxable income.
fn scale_tax(income: &Decimal, year: &str) -> Result<Decimal> {
    let out = resident_tax(
        &Arg::Decimal(income.clone()),
        &Arg::Text(year.to_string()),
        &Arg::Bool(true),
    )?;
    let tax = out["amounts"]["basic_income_tax"]
        .as_str()
        .expect("an amount");
    Ok(Decimal::parse(tax)?)
}

/// Other taxable income, the net rent with its cents dropped, and the
/// taxable income with it, nil if negative.
fn taxable(other: &Arg, net: &Decimal, year: &Year) -> Result<[Decimal; 3]> {
    if year.first < FIRST_TAX_YEAR {
        return refuse(
            "rental_property.tax_effect_year",
            "The effect on tax is worked out for 2024-25 to 2026-27, the years resident-tax holds the scale for; leave out other_taxable_income for an earlier year.",
        );
    }
    let without = money(other)?;
    whole_dollars(&without, "other_taxable_income")?;
    let dropped = net.quantize(0, Rounding::Down)?;
    let sum = without.add(&dropped)?;
    let with = if sum.lt(&int(0))? { int(0) } else { sum };
    Ok([without, dropped, with])
}

/// The tax at the resident scale without and with the net rent, and the
/// difference, for the years the scale is held for.
fn tax_effect(other: &Arg, net: &Decimal, year: &Year, label: &str) -> Result<Vec<Step>> {
    let [without, dropped, with] = taxable(other, net, year)?;
    let (tax_without, tax_with) = (scale_tax(&without, label)?, scale_tax(&with, label)?);
    let effect = tax_without.sub(&tax_with)?;
    let rows: [(&'static str, &'static str, &str, Vec<&Decimal>, Decimal); 5] = [
        (
            "taxable_income_without_rental",
            TAXABLE_WITHOUT,
            "other_taxable_income",
            vec![&without],
            without.clone(),
        ),
        (
            "taxable_income_with_rental",
            TAXABLE_WITH,
            "max(0, other_taxable_income + net_rent_whole_dollars)",
            vec![&without, &dropped],
            with.clone(),
        ),
        (
            "tax_without_rental",
            TAX_WITHOUT,
            "resident_tax(taxable_income_without_rental)",
            vec![&without],
            tax_without.clone(),
        ),
        (
            "tax_with_rental",
            TAX_WITH,
            "resident_tax(taxable_income_with_rental)",
            vec![&with],
            tax_with.clone(),
        ),
        (
            "tax_effect",
            TAX_EFFECT,
            "tax_without_rental - tax_with_rental",
            vec![&tax_without, &tax_with],
            effect,
        ),
    ];
    Ok(rows
        .into_iter()
        .map(|(id, authority, formula, operands, value)| {
            step(id, authority, formula, &operands, value)
        })
        .collect())
}

/// A step summing the values of `parts`.
fn sum_step(id: &'static str, authority: &'static str, parts: &[&Step]) -> Result<Step> {
    let names: Vec<&str> = parts.iter().map(|s| s.id.as_ref()).collect();
    let values: Vec<&Decimal> = parts.iter().map(|s| &s.value).collect();
    let total = values.iter().try_fold(int(0), |sum, v| sum.add(v))?;
    let formula = if names.is_empty() {
        "0".to_string()
    } else {
        names.join(" + ")
    };
    Ok(step(id, authority, &formula, &values, total))
}

/// The owner's legal interest, refusing a holiday home s 26-50 denies.
fn checked_share(args: &RentalPropertyArgs) -> Result<Decimal> {
    let share = ownership(args.ownership_share)?;
    if flag(args.section_26_50_applies, "section_26_50_applies")? {
        return refuse(
            "rental_property.s26_50_denied",
            "Section 26-50 denies the costs of owning a holiday home not used or held mainly to produce rent at all times in the year, before any apportionment (TR 2026/1 paragraphs 17 and 31); the worksheet does not decide 'mainly'.",
        );
    }
    Ok(share)
}

/// The gross rent and the expense lines' steps.
fn line_steps(args: &RentalPropertyArgs, share: &Decimal, held: &Held) -> Result<Vec<Step>> {
    let income = lines(args.income, "income", &INCOME)?;
    let apportioned = lines(args.expenses, "expenses", &CATEGORIES)?;
    let rental_only = lines(
        args.rental_only_expenses,
        "rental_only_expenses",
        &CATEGORIES,
    )?;
    let given = |key| line(&income, key).cloned().unwrap_or(int(0));
    let (rent, other) = (given("rental_income"), given("other_rental_income"));
    let gross = step(
        "gross_rent",
        GROSS_RENT,
        "cents((rental_income + other_rental_income) x ownership_share)",
        &[&rent, &other, share],
        cents(rent.add(&other)?.mul(share)?)?,
    );
    let mut steps = vec![gross];
    steps.extend(expense_steps(&apportioned, &rental_only, share, held)?);
    Ok(steps)
}

/// Adds the total deductions and the net rent after the gross rent and the
/// deductions, and returns the net rent.
fn add_net(steps: &mut Vec<Step>) -> Result<Decimal> {
    let parts: Vec<&Step> = steps[1..]
        .iter()
        .filter(|s| s.id != "travel_denied")
        .collect();
    let total = sum_step("total_deductions", TOTAL, &parts)?;
    let gross = &steps[0].value;
    let value = gross.sub(&total.value)?;
    let net = step(
        "net_rent",
        NET,
        "gross_rent - total_deductions",
        &[gross, &total.value],
        value.clone(),
    );
    steps.extend([total, net]);
    Ok(value)
}

pub fn rental_property(args: &RentalPropertyArgs) -> Result<Value> {
    let label = scope(args.scope_confirmed, args.year, KIND)?;
    let year = Year::of(&label);
    let share = checked_share(args)?;
    let held = held(args, &year)?;
    let mut steps = line_steps(args, &share, &held)?;
    let (interest, loan_rows) = interest(args.loans, &held)?;
    let (borrowing, schedule) = borrowing(args.borrowing_loans, &label)?;
    let (works, works_rows) = capital_works::deduction(args.capital_works, &share, &held, &year)?;
    let (decline, asset_rows) = assets::deduction(args.depreciating_assets, &share, &held)?;
    let decline = step("decline_in_value", DECLINE, &decline.0, &[], decline.1);
    let works = step("capital_works", CAPITAL_WORKS, &works.0, &[], works.1);
    steps.extend([interest, borrowing, decline, works]);
    let net = add_net(&mut steps)?;
    if let Some(other) = args.other_taxable_income {
        steps.extend(tax_effect(other, &net, &year, &label)?);
    }
    let rates = vec![
        rate("ownership_share", &share),
        rate("days_held", held.days),
        rate("days_available", held.available),
    ];
    let mut out = derived_result(KIND, &label, steps, rates)?;
    out["loans"] = Value::Array(loan_rows);
    out["borrowing_loans"] = schedule;
    out["capital_works"] = Value::Array(works_rows);
    out["depreciating_assets"] = Value::Array(asset_rows);
    out["held"] = json!({"from": dates::format(held.from), "to": dates::format(held.to)});
    Ok(out)
}
