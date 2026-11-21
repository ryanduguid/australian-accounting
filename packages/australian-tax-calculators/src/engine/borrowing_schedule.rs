// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `borrowing_expenses_schedule`: ITAA 1997 s 25-25 for several loans, every
//! income year of each, with each year's income-producing use, refinancing as
//! the repayment of one loan by another and the s 25-25(6) $100 test across
//! the loans first drawn in a year. The amounts are one income year's; each
//! loan's row and the totals give every year. s 25-25 was read on the ATO's
//! legal database on 19 November 2026: step 1 of s 25-25(4) was last
//! substituted by Act No 15 of 2017 (Sch 4 item 26) from 1 April 2017, and
//! nothing in it has changed since.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::{
    Arg, BORROWING_EXPENSES, Error, Loan, LoanDates, Result, SpreadYear, Step, cents,
    derived_result, finite_between, income_year, int, money, precise, rate, refuse, scope, step,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::texts;

const KIND: &str = "borrowing_expenses_schedule";

/// Loans first drawn from 1 July 2017, the first income year wholly under
/// s 25-25 as read.
const FIRST_YEAR: i64 = 2017;

/// The last income year whose law was read; a row after it applies s 25-25
/// as read and is marked projected.
const LAST_YEAR: i64 = 2027;

/// The facts a loan states, and whether each is required.
const FACTS: [(&str, bool); 7] = [
    ("borrowing_expenses", true),
    ("loan_start", true),
    ("loan_term_end", true),
    ("repaid_on", true),
    ("income_use", true),
    ("section_26_50_applies", true),
    ("refinances", false),
];

/// One loan as the schedule reads it.
struct Facts {
    expenditure: Decimal,
    loan: Loan,
    /// The spread year by year under s 25-25(4).
    years: Vec<SpreadYear>,
    /// The extent the money is used to produce assessable income in each of
    /// `years`, in their order.
    uses: Vec<Decimal>,
    refinances: Option<usize>,
}

/// One income year of a loan's schedule.
struct Row {
    first: i64,
    remaining: Decimal,
    maximum: Decimal,
    use_share: Decimal,
    deduction: Decimal,
    after: Decimal,
    /// The spread year, `None` when s 25-25(6) deducts it all.
    spread: Option<(i64, i64, i64)>,
}

/// `loan n: ` before a refusal's message, its code kept.
fn at<T>(n: usize, outcome: Result<T>) -> Result<T> {
    outcome.map_err(|error| match error {
        Error::Value { code, message } => Error::Value {
            code,
            message: format!("loan {n}: {message}"),
        },
        other => other,
    })
}

fn loans_invalid<T>() -> Result<T> {
    refuse(
        "borrowing.loans_invalid",
        "loans must be a non-empty list of objects, one for each loan.",
    )
}

/// An income year such as `2026-27`, by the calendar year it starts in.
fn year_of(text: &str) -> Option<i64> {
    let first: i64 = text.get(..4)?.parse().ok()?;
    (income_year(first) == text).then_some(first)
}

/// A decimal fraction from 0 to 1 given as a string.
fn fraction(value: &Value) -> Result<Option<Decimal>> {
    match value.as_str().map(Decimal::parse) {
        Some(Ok(share)) => finite_between(&Arg::Decimal(share), "0", "1").map_err(Error::from),
        _ => Ok(None),
    }
}

fn income_use_invalid<T>(years: &[SpreadYear]) -> Result<T> {
    let (first, last) = (years[0].first, years[years.len() - 1].first);
    refuse(
        "borrowing.income_use_invalid",
        format!(
            "income_use must be a decimal fraction from 0 to 1, or an object giving one for each income year from {} to {}.",
            income_year(first),
            income_year(last)
        ),
    )
}

/// `income_use`: one fraction for every year, or an object giving one for
/// each income year of the loan's spread.
fn uses(value: &Value, years: &[SpreadYear]) -> Result<Vec<Decimal>> {
    if let Some(share) = fraction(value)? {
        return Ok(vec![share; years.len()]);
    }
    let Value::Object(by_year) = value else {
        return income_use_invalid(years);
    };
    if by_year.len() != years.len() {
        return income_use_invalid(years);
    }
    years
        .iter()
        .map(
            |row| match by_year.get(&income_year(row.first)).map(fraction) {
                Some(Ok(Some(share))) => Ok(share),
                Some(Err(error)) => Err(error),
                _ => income_use_invalid(years),
            },
        )
        .collect()
}

/// Refuses a fact a loan does not take, then a required fact missing.
fn check_fields(fields: &Map<String, Value>) -> Result<()> {
    if let Some(key) = fields
        .keys()
        .find(|key| !FACTS.iter().any(|(name, _)| name == key))
    {
        return refuse(
            "input.field_unknown",
            format!("{key} is not a fact a loan takes"),
        );
    }
    if let Some((key, _)) = FACTS
        .iter()
        .find(|(key, required)| *required && !fields.contains_key(*key))
    {
        return refuse("borrowing.loan_fact_missing", format!("{key} is required."));
    }
    Ok(())
}

/// Refuses a leisure facility (s 26-50) and a flag that is not a boolean.
fn check_s26_50(flag: &Value) -> Result<()> {
    match flag {
        Value::Bool(false) => Ok(()),
        Value::Bool(_) => refuse("borrowing.s26_50_denied", texts::BORROWING_S26_50),
        _ => refuse(
            "input.flag_invalid",
            "section_26_50_applies must be a boolean.",
        ),
    }
}

/// The loan's dates, parsed as `borrowing_expenses` parses them.
fn loan_dates(fields: &Map<String, Value>) -> Result<Loan> {
    let start = match &fields["loan_start"] {
        Value::String(text) => Arg::Text(text.clone()),
        other => Arg::Json(other.clone()),
    };
    let (term_end, repaid_on) = (
        Arg::Json(fields["loan_term_end"].clone()),
        Arg::Json(fields["repaid_on"].clone()),
    );
    LoanDates {
        start: &start,
        term_end: &term_end,
        repaid_on: &repaid_on,
    }
    .parse()
}

/// Refuses a loan first drawn before 1 July 2017 or after the year asked about.
fn check_first_year(loan: &Loan, claimed: i64, year: &str) -> Result<()> {
    if loan.first_year < FIRST_YEAR {
        return refuse(
            "borrowing.loan_before_2017_18",
            "a loan first drawn before 1 July 2017 is out of scope: s 25-25 was read as it applies from the 2017-18 income year.",
        );
    }
    if loan.first_year > claimed {
        return refuse(
            "borrowing.loan_starts_after_year",
            format!("loan_start must not be after the {year} income year ended."),
        );
    }
    Ok(())
}

/// One loan from its stated facts, refusing a fact missing, invalid or not
/// taken, and a loan out of scope.
fn read_loan(value: &Value, claimed: i64, year: &str) -> Result<Facts> {
    let Value::Object(fields) = value else {
        return loans_invalid();
    };
    check_fields(fields)?;
    // money refuses anything but a decimal, as the one-loan worksheet does.
    let amount = fields["borrowing_expenses"]
        .as_str()
        .and_then(|text| Decimal::parse(text).ok());
    let expenditure = money(&amount.map_or(Arg::Json(Value::Null), Arg::Decimal))?;
    check_s26_50(&fields["section_26_50_applies"])?;
    let loan = loan_dates(fields)?;
    check_first_year(&loan, claimed, year)?;
    let years = loan.years(&expenditure)?;
    let uses = uses(&fields["income_use"], &years)?;
    let refinances = match fields.get("refinances") {
        None => None,
        Some(k) => match k.as_u64().and_then(|k| usize::try_from(k).ok()) {
            Some(k) => Some(k),
            None => return refinance_invalid(),
        },
    };
    Ok(Facts {
        expenditure,
        loan,
        years,
        uses,
        refinances,
    })
}

fn refinance_invalid<T>() -> Result<T> {
    refuse(
        "borrowing.refinance_invalid",
        "refinances must be the number of an earlier loan in loans, repaid on this loan's loan_start and refinanced by no other loan.",
    )
}

/// Each loan's `refinances` names an earlier loan, repaid on the day this
/// one is first drawn and refinanced by no other.
fn check_refinancing(loans: &[Facts]) -> Result<Vec<Option<usize>>> {
    let mut refinanced_by = vec![None; loans.len()];
    for (i, facts) in loans.iter().enumerate() {
        let Some(k) = facts.refinances else {
            continue;
        };
        let valid = (1..=i).contains(&k)
            && refinanced_by[k - 1].is_none()
            && loans[k - 1].loan.repaid == Some(facts.loan.start);
        if !valid {
            return at(i + 1, refinance_invalid());
        }
        refinanced_by[k - 1] = Some(i + 1);
    }
    Ok(refinanced_by)
}

/// `other_borrowing_expenses`: the income-producing share of borrowing
/// expenses not in `loans`, by the income year incurred.
fn other_expenses(value: Option<&Arg>) -> Result<BTreeMap<i64, Decimal>> {
    let invalid = || {
        refuse(
            "borrowing.other_expenses_invalid",
            "other_borrowing_expenses must be an object giving, for an income year such as \"2025-26\", the income-producing share of borrowing expenses incurred that year on loans not in loans.",
        )
    };
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let Arg::Json(Value::Object(by_year)) = value else {
        return invalid();
    };
    let mut out = BTreeMap::new();
    for (key, amount) in by_year {
        let (Some(first), Some(Ok(amount))) = (year_of(key), amount.as_str().map(Decimal::parse))
        else {
            return invalid();
        };
        out.insert(first, money(&Arg::Decimal(amount))?);
    }
    Ok(out)
}

/// s 25-25(6): whether each loan's expenditure is deducted in the year
/// incurred, as the total of the income-producing shares of everything
/// incurred for borrowing that year is $100 or less. Money not used to produce
/// income at all is outside s 25-25(6)(a) and (b), so it is spread.
fn not_spread(loans: &[Facts], other: &BTreeMap<i64, Decimal>) -> Result<Vec<bool>> {
    let mut totals: BTreeMap<i64, Decimal> = other.clone();
    for facts in loans {
        let share = facts.expenditure.mul(&facts.uses[0])?;
        let total = totals
            .entry(facts.loan.first_year)
            .or_insert_with(|| int(0));
        *total = total.add(&share)?;
    }
    loans
        .iter()
        .map(|facts| {
            let total = &totals[&facts.loan.first_year];
            Ok(!facts.uses[0].is_zero() && total.le(&int(100))?)
        })
        .collect()
}

impl Facts {
    /// The schedule's rows: one under s 25-25(6), or one for each year of the
    /// spread.
    fn rows(&self, whole: bool) -> Result<Vec<Row>> {
        if whole {
            let share = self.uses[0].clone();
            return Ok(vec![Row {
                first: self.loan.first_year,
                remaining: self.expenditure.clone(),
                maximum: self.expenditure.clone(),
                deduction: cents(self.expenditure.mul(&share)?)?,
                use_share: share,
                after: int(0),
                spread: None,
            }]);
        }
        self.years
            .iter()
            .zip(&self.uses)
            .map(|(year, share)| {
                Ok(Row {
                    first: year.first,
                    remaining: year.remaining.clone(),
                    maximum: year.maximum.clone(),
                    deduction: cents(year.maximum.mul(share)?)?,
                    use_share: share.clone(),
                    after: year.remaining.sub(&year.maximum)?,
                    spread: Some((year.end, year.left, year.days)),
                })
            })
            .collect()
    }
}

/// An amount to the cent, half up.
fn shown(value: &Decimal) -> Result<String> {
    Ok(value.quantize(-2, Rounding::HalfUp)?.to_string())
}

fn row_json(row: &Row) -> Result<Value> {
    let (to, left, days) = match row.spread {
        Some((end, left, days)) => (json!(dates::format(end)), json!(left), json!(days)),
        None => (Value::Null, Value::Null, Value::Null),
    };
    Ok(json!({
        "income_year": income_year(row.first),
        "period_of_the_loan_to": to,
        "remaining_expenditure": shown(&row.remaining)?,
        "remaining_loan_period_days": left,
        "days_in_year": days,
        "maximum_deduction": shown(&row.maximum)?,
        "income_use": row.use_share.to_string(),
        "deduction": shown(&row.deduction)?,
        "remaining_expenditure_after_year": shown(&row.after)?,
        "section_26_155": row.first >= 2027,
        "projected": row.first > LAST_YEAR,
    }))
}

/// A loan's row: its facts, how it is deducted, its schedule and notes.
fn loan_json(n: usize, facts: &Facts, rows: &[Row], refinanced_by: Option<usize>) -> Result<Value> {
    let last = &facts.years[facts.years.len() - 1];
    let whole = rows[0].spread.is_none();
    let (spread, to, basis) = if whole {
        (
            "not spread: $100 or less (s 25-25(6))",
            Value::Null,
            Value::Null,
        )
    } else {
        (
            "spread over the period of the loan (s 25-25(4))",
            json!(dates::format(last.end)),
            json!(facts.loan.basis(last.end)),
        )
    };
    let mut notes = Vec::new();
    if let Some(m) = refinanced_by {
        notes.push(format!(
            "Refinanced by loan {m}: repaid on {}, so the period of the loan ends that day (s 25-25(5)(b)) and the income year it ends in deducts what remains (s 25-25(4)).",
            dates::format(last.end)
        ));
    }
    if let Some(k) = facts.refinances {
        notes.push(format!(
            "Refinances loan {k}: this loan's own borrowing expenses are spread from its first borrowing (s 25-25(5)); the extent its money is used to produce assessable income is the operator's to establish (s 25-25(1))."
        ));
    }
    let total = rows
        .iter()
        .try_fold(int(0), |sum, row| sum.add(&row.deduction))?;
    let schedule = rows.iter().map(row_json).collect::<Result<Vec<_>>>()?;
    Ok(json!({
        "loan": n,
        "borrowing_expenses": shown(&facts.expenditure)?,
        "period_of_the_loan_from": dates::format(facts.loan.start),
        "period_of_the_loan_to": to,
        "period_of_the_loan": basis,
        "spread": spread,
        "refinances": facts.refinances,
        "refinanced_by": refinanced_by,
        "schedule": schedule,
        "total_deduction": shown(&total)?,
        "notes": notes,
    }))
}

/// The amounts: (id, authority). Each formula writes in a term for every
/// loan, as the loans are a list.
const AMOUNTS: [(&str, &str); 4] = [
    ("borrowing_expenses", BORROWING_EXPENSES),
    (
        "maximum_deduction",
        "ITAA 1997 s 25-25(4) steps 1 to 4 for each loan, each rounded to the cent and carried forward as the ATO's 'Borrowing expenses' examples are, or under s 25-25(6) all of it in the year incurred; nil for a loan whose period ended before the year",
    ),
    (
        "deduction",
        "ITAA 1997 s 25-25(2), (3) and (6)(c) and (d): each loan's maximum times the extent its money is used that year to produce assessable income, to the cent",
    ),
    (
        "remaining_expenditure_after_year",
        "ITAA 1997 s 25-25(4) step 1: what remains of each loan's expenditure for later income years",
    ),
];

/// The year's steps, each summing a term for every loan.
fn steps(loans: &[Facts], rows: &[Vec<Row>], claimed: i64) -> Result<Vec<Step>> {
    let mut terms: [Vec<(String, Decimal)>; 4] = Default::default();
    for (facts, rows) in loans.iter().zip(rows) {
        let e = &facts.expenditure;
        terms[0].push((e.to_string(), e.clone()));
        let nil = || ("0".to_string(), int(0));
        let [maximum, deduction, after] = match rows.iter().find(|row| row.first == claimed) {
            Some(row) => [
                match row.spread {
                    Some((_, left, days)) => (
                        format!("cents({} x {days} / {left})", row.remaining),
                        row.maximum.clone(),
                    ),
                    None => (e.to_string(), e.clone()),
                },
                (
                    format!("cents({} x {})", row.maximum, row.use_share),
                    row.deduction.clone(),
                ),
                match row.spread {
                    Some(_) => (
                        format!("{} - {}", row.remaining, row.maximum),
                        row.after.clone(),
                    ),
                    None => nil(),
                },
            ],
            None => [nil(), nil(), nil()],
        };
        terms[1].push(maximum);
        terms[2].push(deduction);
        terms[3].push(after);
    }
    AMOUNTS
        .iter()
        .zip(terms)
        .map(|(&(id, authority), terms)| {
            let formula: Vec<&str> = terms.iter().map(|(text, _)| text.as_str()).collect();
            let total = terms
                .iter()
                .try_fold(int(0), |sum, (_, value)| sum.add(value))?;
            Ok(step(id, authority, &formula.join(" + "), &[], total))
        })
        .collect()
}

/// Every income year's maximum and deduction across the loans.
fn totals(rows: &[Vec<Row>]) -> Result<Vec<Value>> {
    let mut by_year: BTreeMap<i64, (Decimal, Decimal)> = BTreeMap::new();
    for row in rows.iter().flatten() {
        let (maximum, deduction) = by_year.entry(row.first).or_insert((int(0), int(0)));
        *maximum = maximum.add(&row.maximum)?;
        *deduction = deduction.add(&row.deduction)?;
    }
    by_year
        .iter()
        .map(|(first, (maximum, deduction))| {
            Ok(json!({
                "income_year": income_year(*first),
                "maximum_deduction": shown(maximum)?,
                "deduction": shown(deduction)?,
                "section_26_155": *first >= 2027,
                "projected": *first > LAST_YEAR,
            }))
        })
        .collect()
}

/// Warns when a row falls under s 26-155 or after the law read.
fn add_warnings(out: &mut Value, rows: &[Vec<Row>]) {
    let years: Vec<i64> = rows.iter().flatten().map(|row| row.first).collect();
    let warnings = out["warnings"].as_array_mut().expect("warnings");
    if years.iter().any(|first| *first >= 2027) {
        warnings.push(json!(texts::BORROWING_S26_155));
    }
    if years.iter().any(|first| *first > LAST_YEAR) {
        warnings.push(json!(texts::BORROWING_PROJECTED));
    }
}

/// ITAA 1997 s 25-25 for each of `loans`: the year asked about's amounts,
/// and every year of each loan.
pub fn borrowing_expenses_schedule(
    loans: &Arg,
    other_borrowing_expenses: Option<&Arg>,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let claimed = year_of(&year).expect("a supported income year");
    let Arg::Json(Value::Array(list)) = loans else {
        return loans_invalid();
    };
    if list.is_empty() {
        return loans_invalid();
    }
    let loans = list
        .iter()
        .enumerate()
        .map(|(i, value)| at(i + 1, read_loan(value, claimed, &year)))
        .collect::<Result<Vec<_>>>()?;
    let refinanced_by = check_refinancing(&loans)?;
    let other = other_expenses(other_borrowing_expenses)?;
    let (steps, rows, out_loans, totals) = precise(|| {
        let whole = not_spread(&loans, &other)?;
        let rows = loans
            .iter()
            .zip(&whole)
            .map(|(facts, whole)| facts.rows(*whole))
            .collect::<Result<Vec<_>>>()?;
        let out_loans = loans
            .iter()
            .zip(&rows)
            .zip(&refinanced_by)
            .enumerate()
            .map(|(i, ((facts, rows), by))| loan_json(i + 1, facts, rows, *by))
            .collect::<Result<Vec<_>>>()?;
        let (steps, totals) = (steps(&loans, &rows, claimed)?, totals(&rows)?);
        Ok((steps, rows, out_loans, totals))
    })?;
    let mut out = derived_result(KIND, &year, steps, vec![rate("loans", loans.len())])?;
    add_warnings(&mut out, &rows);
    out["loans"] = Value::Array(out_loans);
    out["schedule"] = Value::Array(totals);
    Ok(out)
}
