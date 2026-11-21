// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `loan_transactions`: a loan's statement from dated drawings and repayments,
//! with interest worked out day by day at an annual rate that changes on
//! dated events and debited to the loan at each month's end, and the interest
//! charged in each income year. The citation is ITAA 1997 compilation 268
//! (1 October 2026), s 8-1, read on 19 November 2026.

use serde_json::{Map, Value, json};

use super::{income_year, statement, sum_terms};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{
    Arg, Result, Step, date_arg, derived_result, finite_between, int, money, precise, rate, refuse,
    scope, step,
};

const KIND: &str = "loan_transactions";

/// The most drawings and repayments, and rate changes, a statement takes.
const MOST_TRANSACTIONS: usize = 5000;
const MOST_RATE_CHANGES: usize = 1000;

/// Days in the year the annual rate is divided by, in leap years too.
const DAYS_A_YEAR: i64 = 365;

const OPENING: &str = "As established: the balance owing at the start of start_date";
const DRAWINGS: &str = "The drawings listed, each added to the balance on its date";
const REPAYMENTS: &str = "The repayments listed, each taken off the balance on its date";
const INTEREST: &str = "Each day's interest is the balance owing at the end of that day, when above nil, times the annual rate in force that day / 365; the interest is debited to the loan on the last day of each month and on end_date, rounded half up to the cent. Interest charged on money borrowed is deductible under ITAA 1997 s 8-1(1) to the extent the money is used to gain or produce assessable income, and not to the extent it is capital, private or domestic (s 8-1(2)), which the operator establishes";
const CLOSING: &str = "The balance owing at the end of end_date, after its interest is debited";

/// The facts as the command passes them; see the README for each.
pub struct LoanTransactionsArgs<'a> {
    pub opening_balance: &'a Arg,
    pub annual_rate: &'a Arg,
    pub start_date: &'a Arg,
    pub end_date: &'a Arg,
    pub transactions: &'a Arg,
    pub rate_changes: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// A drawing (adding to the balance) or repayment on a day.
struct Movement {
    day: i64,
    drawing: bool,
    amount: Decimal,
}

fn annual_rate(value: &Arg, what: &str) -> Result<Decimal> {
    match finite_between(value, "0", "1")? {
        Some(rate) => Ok(rate),
        None => refuse(
            "input.fraction_invalid",
            format!("{what} must be a finite decimal fraction from 0 to 1."),
        ),
    }
}

/// The statement's first and last days: `start_date` in the income year,
/// `end_date` on or after it and within 30 years.
fn statement_days(args: &LoanTransactionsArgs, year: &str) -> Result<(i64, i64)> {
    let start = date_arg(args.start_date, "start_date")?;
    let end = date_arg(args.end_date, "end_date")?;
    let first: i64 = year[..4].parse().expect("income year");
    if income_year(start).0 != year {
        return refuse(
            "loan_transactions.start_outside_year",
            format!(
                "start_date must fall in the {year} income year, 1 July {first} to 30 June {}.",
                first + 1
            ),
        );
    }
    if end < start || end >= dates::add_months(start, 360) {
        return refuse(
            "loan_transactions.end_date_invalid",
            "end_date must be on or after start_date and less than 30 years after it.",
        );
    }
    Ok((start, end))
}

fn invalid<T>(what: &str, n: usize, why: &str) -> Result<T> {
    refuse(
        "loan_transactions.entry_invalid",
        format!("{what} {n}: {why}."),
    )
}

/// The objects in the list `value`, at most `most` of them.
fn entries<'a>(value: &'a Arg, what: &str, most: usize) -> Result<&'a [Value]> {
    match value {
        Arg::Json(Value::Array(items)) if items.len() <= most => Ok(items),
        _ => refuse(
            "loan_transactions.entry_invalid",
            format!("{what} must be a list of at most {most} objects."),
        ),
    }
}

/// An entry's fields, refusing one it does not take.
fn fields<'a>(
    item: &'a Value,
    what: &str,
    n: usize,
    keys: &[&str],
) -> Result<&'a Map<String, Value>> {
    let Some(fields) = item.as_object() else {
        return invalid(what, n, "must be an object");
    };
    if let Some(key) = fields.keys().find(|k| !keys.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("{what} {n}: {key} is not a fact it takes"),
        );
    }
    Ok(fields)
}

/// An entry's date, refused before the start or after the end, or before the
/// entry listed before it.
fn entry_day(
    fields: &Map<String, Value>,
    (what, n): (&str, usize),
    (first, end): (i64, i64),
    after: i64,
) -> Result<i64> {
    let Some(day) = fields
        .get("date")
        .and_then(Value::as_str)
        .and_then(dates::parse)
    else {
        return invalid(what, n, "date must be a YYYY-MM-DD date");
    };
    if day < first || day > end {
        return refuse(
            "loan_transactions.date_outside_statement",
            format!(
                "{what} {n}: date must be from {} to end_date.",
                dates::format(first)
            ),
        );
    }
    if day < after {
        return refuse(
            "loan_transactions.not_in_date_order",
            format!("{what} {n}: the list must be in date order."),
        );
    }
    Ok(day)
}

/// The drawings and repayments, in date order, from `start` to `end`.
fn movements(value: &Arg, days: (i64, i64)) -> Result<Vec<Movement>> {
    let what = "transaction";
    let items = entries(value, "transactions", MOST_TRANSACTIONS)?;
    let mut out: Vec<Movement> = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let n = i + 1;
        let fields = fields(item, what, n, &["date", "kind", "amount"])?;
        let after = out.last().map_or(days.0, |m| m.day);
        let day = entry_day(fields, (what, n), days, after)?;
        let drawing = match fields.get("kind").and_then(Value::as_str) {
            Some("drawing") => true,
            Some("repayment") => false,
            _ => return invalid(what, n, "kind must be drawing or repayment"),
        };
        let Some(amount) = fields
            .get("amount")
            .and_then(Value::as_str)
            .and_then(|t| Decimal::parse(t).ok())
        else {
            return invalid(what, n, "amount must be a decimal string");
        };
        let amount = money(&Arg::Decimal(amount))?;
        out.push(Movement {
            day,
            drawing,
            amount,
        });
    }
    Ok(out)
}

/// The rate changes, in date order, each after the start and by the end.
fn rate_changes(value: Option<&Arg>, (start, end): (i64, i64)) -> Result<Vec<(i64, Decimal)>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let what = "rate change";
    let items = entries(value, "rate_changes", MOST_RATE_CHANGES)?;
    let mut out: Vec<(i64, Decimal)> = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let n = i + 1;
        let fields = fields(item, what, n, &["date", "annual_rate"])?;
        // Strictly after the change before it: 2 rates cannot start on one day.
        let after = out.last().map_or(start + 1, |c| c.0 + 1);
        let day = entry_day(fields, (what, n), (start + 1, end), after)?;
        let given = fields
            .get("annual_rate")
            .and_then(Value::as_str)
            .and_then(|t| Decimal::parse(t).ok());
        let Some(given) = given else {
            return invalid(what, n, "annual_rate must be a decimal string");
        };
        out.push((day, annual_rate(&Arg::Decimal(given), "annual_rate")?));
    }
    Ok(out)
}

/// One statement line: (day, details, debit, credit, balance).
type Line = (i64, String, Decimal, Decimal, Decimal);

/// What the days give: the statement lines, each interest debit with its
/// day, and the closing balance.
struct Run {
    lines: Vec<Line>,
    debits: Vec<(i64, Decimal)>,
    closing: Decimal,
}

impl Run {
    /// A line on `day` at the balance after it.
    fn line(&mut self, day: i64, details: String, debit: Decimal, credit: Decimal) {
        let balance = self.closing.clone();
        self.lines.push((day, details, debit, credit, balance));
    }

    /// One drawing or repayment.
    fn movement(&mut self, m: &Movement) -> Result<()> {
        if m.drawing {
            self.closing = self.closing.add(&m.amount)?;
            self.line(m.day, "Drawing".into(), m.amount.clone(), int(0));
        } else {
            self.closing = self.closing.sub(&m.amount)?;
            self.line(m.day, "Repayment".into(), int(0), m.amount.clone());
        }
        Ok(())
    }

    /// The interest accrued to `day`, debited to the cent.
    fn debit(&mut self, day: i64, accrued: &Decimal) -> Result<()> {
        let debit = accrued.quantize(-2, Rounding::HalfUp)?;
        if !debit.is_zero() {
            self.closing = self.closing.add(&debit)?;
            self.line(day, "Interest".into(), debit.clone(), int(0));
            self.debits.push((day, debit));
        }
        Ok(())
    }
}

/// The days from `start` to `end`: each day's rate change and transactions,
/// then its interest, debited at the month's end and on the last day. A day
/// that ends below nil is refused.
fn run(
    opening: &Decimal,
    initial_rate: &Decimal,
    days: (i64, i64),
    movements: &[Movement],
    changes: &[(i64, Decimal)],
) -> Result<Run> {
    let (start, end) = days;
    let nil = int(0);
    let (mut rate, mut accrued) = (initial_rate.clone(), int(0));
    let mut out = Run {
        lines: Vec::new(),
        debits: Vec::new(),
        closing: opening.clone(),
    };
    out.line(start, "Opening balance".into(), nil.clone(), nil.clone());
    let (mut next_move, mut next_change) = (movements.iter().peekable(), changes.iter().peekable());
    for day in start..=end {
        if let Some((_, changed)) = next_change.next_if(|c| c.0 == day) {
            rate = changed.clone();
            let details = format!("Annual rate {changed} from this day");
            out.line(day, details, nil.clone(), nil.clone());
        }
        while let Some(m) = next_move.next_if(|m| m.day == day) {
            out.movement(m)?;
        }
        if nil.lt(&out.closing)? {
            accrued = accrued.add(&out.closing.mul(&rate)?.div(&int(DAYS_A_YEAR))?)?;
        }
        if dates::to_civil(day + 1).2 == 1 || day == end {
            out.debit(day, &accrued)?;
            accrued = int(0);
        }
        if out.closing.lt(&nil)? {
            return refuse(
                "loan_transactions.balance_below_nil",
                format!(
                    "The balance is below nil at the end of {}: a loan in credit is not worked out. End the statement on the day a payout repays the balance and the interest to that day.",
                    dates::format(day)
                ),
            );
        }
    }
    Ok(out)
}

/// The interest debited in each income year.
fn by_income_year(debits: &[(i64, Decimal)]) -> Result<Value> {
    let mut years: Vec<(String, Decimal)> = Vec::new();
    for (day, debit) in debits {
        let year = income_year(*day).0;
        match years.last_mut().filter(|(y, _)| *y == year) {
            Some((_, total)) => *total = total.add(debit)?,
            None => years.push((year, debit.clone())),
        }
    }
    Ok(Value::Array(
        years
            .into_iter()
            .map(|(year, interest)| json!({"income_year": year, "interest": interest.fixed(2, false)}))
            .collect(),
    ))
}

/// The steps behind the amounts.
fn steps(opening: &Decimal, movements: &[Movement], run: &Run) -> Result<Vec<Step>> {
    let pick = |drawing: bool| {
        movements
            .iter()
            .filter(|m| m.drawing == drawing)
            .map(|m| &m.amount)
            .collect::<Vec<_>>()
    };
    let (drawn_terms, drawn) = sum_terms(pick(true))?;
    let (repaid_terms, repaid) = sum_terms(pick(false))?;
    let (interest_terms, interest) = sum_terms(run.debits.iter().map(|(_, d)| d).collect())?;
    Ok(vec![
        step(
            "opening_balance",
            OPENING,
            "opening_balance",
            &[opening],
            opening.clone(),
        ),
        step("total_drawings", DRAWINGS, &drawn_terms, &[], drawn.clone()),
        step(
            "total_repayments",
            REPAYMENTS,
            &repaid_terms,
            &[],
            repaid.clone(),
        ),
        step(
            "total_interest",
            INTEREST,
            &interest_terms,
            &[],
            interest.clone(),
        ),
        step(
            "closing_balance",
            CLOSING,
            "opening_balance + total_drawings + total_interest - total_repayments",
            &[opening, &drawn, &interest, &repaid],
            run.closing.clone(),
        ),
    ])
}

/// The statement lines as JSON rows and as a printable Markdown table.
fn statement_out(lines: &[Line]) -> (Value, String) {
    let amount = |v: &Decimal| {
        if v.is_zero() {
            String::new()
        } else {
            v.fixed(2, false)
        }
    };
    let rows = lines
        .iter()
        .map(|(day, details, debit, credit, balance)| {
            json!({"date": dates::format(*day), "details": details, "debit": debit.fixed(2, false),
                   "credit": credit.fixed(2, false), "balance": balance.fixed(2, false)})
        })
        .collect();
    let cells: Vec<Vec<String>> = lines
        .iter()
        .map(|(day, details, debit, credit, balance)| {
            vec![
                dates::format(*day),
                details.clone(),
                amount(debit),
                amount(credit),
                balance.fixed(2, false),
            ]
        })
        .collect();
    let text = statement::markdown(
        &["Date", "Details", "Debit", "Credit", "Balance"],
        &[false, false, true, true, true],
        &cells,
    );
    (Value::Array(rows), text)
}

pub fn loan_transactions(args: &LoanTransactionsArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let opening = money(args.opening_balance)?;
    let initial = annual_rate(args.annual_rate, "annual_rate")?;
    let days = statement_days(args, &year)?;
    let movements = movements(args.transactions, days)?;
    let changes = rate_changes(args.rate_changes, days)?;
    let (run, steps) = precise(|| {
        let run = run(&opening, &initial, days, &movements, &changes)?;
        let steps = steps(&opening, &movements, &run)?;
        Ok((run, steps))
    })?;
    let rates = vec![rate("annual_rate", initial.to_string())];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    let (rows, text) = statement_out(&run.lines);
    out["interest_by_income_year"] = precise(|| by_income_year(&run.debits))?;
    out["statement_rows"] = rows;
    out["statement"] = json!(text);
    Ok(out)
}
