// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The business capital expenditure worksheet's amounts: each item's terms in
//! them under ITAA 1997 s 40-880(2) to (4), and its row.

use serde_json::{Value, json};

use super::super::{Result, Step, int, step};
use super::{FIVE_YEARS, Item, Method, year_text};
use crate::decimal::Decimal;

/// An exact amount with at least 2 decimal places and no trailing zeros
/// beyond them.
fn exact(value: &Decimal) -> String {
    let text = value.fixed_format();
    let Some((whole, fraction)) = text.split_once('.') else {
        return format!("{text}.00");
    };
    let fraction = fraction.trim_end_matches('0');
    format!("{whole}.{fraction:0<2}")
}

/// The amounts: (id, authority). Each one's formula writes in a term for
/// every item, as the items are a list.
const AMOUNTS: [(&str, &str); 4] = [
    (
        "expenditure",
        "ITAA 1997 s 40-880(2): the capital expenditure incurred, each item's amount",
    ),
    (
        "deductible_expenditure",
        "ITAA 1997 s 40-880(3) and (4): each item to the extent of its taxable purpose; nil where s 40-880(4)(b), (5), (7), (8) or (9) excludes it",
    ),
    (
        "deduction",
        "ITAA 1997 s 40-880(2): one fifth of each item's deductible expenditure in each of the 5 income years starting in the year incurred, or under s 40-880(2A) all of it in that year",
    ),
    (
        "remaining_expenditure_after_year",
        "ITAA 1997 s 40-880(2): the fifths of each item's deductible expenditure left for later income years",
    ),
];

/// A term of an amount's formula and its value.
type Term = (String, Decimal);

/// A nil term.
fn nil() -> Term {
    ("0".to_string(), int(0))
}

impl Item {
    /// This item's term in each amount, in `AMOUNTS`' order, and its row.
    fn worked(&self, n: usize, year: i64) -> Result<([Term; 4], Value)> {
        let amount = &self.amount;
        let whole = format!("{amount} x {}", self.taxable_purpose);
        let excluded = !self.excluded_by.is_empty();
        let deductible = if excluded {
            nil()
        } else {
            (whole.clone(), amount.mul(&self.taxable_purpose)?)
        };
        let (deduction, remaining, schedule) = if excluded {
            (nil(), nil(), Vec::new())
        } else {
            self.spread(&whole, &deductible, year)?
        };
        let row = self.row(n, &deductible.1, &deduction.1, &schedule);
        let expenditure = (amount.to_string(), amount.clone());
        Ok(([expenditure, deductible, deduction, remaining], row))
    }

    /// A deductible item's deduction for `year`, what is left after it and
    /// the deduction in each income year.
    fn spread(
        &self,
        whole: &str,
        deductible: &Term,
        year: i64,
    ) -> Result<(Term, Term, Vec<(i64, Decimal)>)> {
        Ok(match &self.method {
            Method::Immediate(_) => {
                let deduction = if self.incurred == year {
                    deductible.clone()
                } else {
                    nil()
                };
                (
                    deduction,
                    nil(),
                    vec![(self.incurred, deductible.1.clone())],
                )
            }
            Method::FiveYears => {
                let fifth = deductible.1.div(&int(5))?;
                let later = self.incurred + 4 - year;
                (
                    // Incurred at most 4 years before the year asked about.
                    if later >= 0 {
                        (format!("{whole} / 5"), fifth.clone())
                    } else {
                        nil()
                    },
                    if later > 0 {
                        let left = deductible.1.mul(&int(later))?.div(&int(5))?;
                        (format!("{whole} x {later} / 5"), left)
                    } else {
                        nil()
                    },
                    (0..5).map(|k| (self.incurred + k, fifth.clone())).collect(),
                )
            }
        })
    }

    /// This item's row.
    fn row(
        &self,
        n: usize,
        deductible: &Decimal,
        deduction: &Decimal,
        schedule: &[(i64, Decimal)],
    ) -> Value {
        let method = match &self.method {
            _ if !self.excluded_by.is_empty() => Value::Null,
            Method::Immediate(reason) => json!(reason),
            Method::FiveYears => json!(FIVE_YEARS),
        };
        let schedule: Vec<Value> = schedule
            .iter()
            .map(|(first, value)| json!({"income_year": year_text(*first), "deduction": exact(value)}))
            .collect();
        json!({
            "item": n,
            "incurred_in": year_text(self.incurred),
            "amount": exact(&self.amount),
            "business": self.business,
            "limit": self.limit,
            "taxable_purpose": self.taxable_purpose.fixed_format(),
            "excluded_by": self.excluded_by,
            "deductible_expenditure": exact(deductible),
            "method": method,
            "deduction": exact(deduction),
            "schedule": schedule,
            "notes": self.notes,
        })
    }
}

/// The amounts' steps, summing each item's term, and the items' rows.
pub(super) fn worked(items: &[Item], year: i64) -> Result<(Vec<Step>, Vec<Value>)> {
    let mut terms: [Vec<Term>; 4] = Default::default();
    let mut rows = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let (item_terms, row) = item.worked(i + 1, year)?;
        for (all, term) in terms.iter_mut().zip(item_terms) {
            all.push(term);
        }
        rows.push(row);
    }
    let mut steps = Vec::new();
    for (&(id, authority), terms) in AMOUNTS.iter().zip(terms) {
        let formula: Vec<&str> = terms.iter().map(|(text, _)| text.as_str()).collect();
        let mut values = terms.iter().map(|(_, value)| value);
        let first = values.next().expect("at least one item").clone();
        let total = values.try_fold(first, |sum, value| sum.add(value))?;
        steps.push(step(id, authority, &formula.join(" + "), &[], total));
    }
    Ok((steps, rows))
}
