// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `fhss` from contributions: the limit, the counted contributions and their
//! associated earnings at the shortfall interest charge rate (TAA 1953 Sch 1
//! ss 138-40 and 280-105(2)), with a row for each year and contribution.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::count::{count, order, split_personal};
use super::input::{Piece, day, fy_label, read_contributions, read_deductions};
use super::{Components, FhssArgs};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{Result, d, refuse};

/// The SIC annual rate, in per cent, for each quarter from 1 July 2017:
/// (calendar year, first month of the quarter, rate). The ATO's 'Shortfall
/// interest charge (SIC) rates' page, last updated 4 September 2026, read on
/// 20 November 2026. Each is the base interest rate plus 3 percentage points
/// (s 280-105(2)).
const SIC: [(i64, i64, &str); 38] = [
    (2017, 7, "4.73"),
    (2017, 10, "4.70"),
    (2018, 1, "4.72"),
    (2018, 4, "4.77"),
    (2018, 7, "4.96"),
    (2018, 10, "4.96"),
    (2019, 1, "4.94"),
    (2019, 4, "4.96"),
    (2019, 7, "4.54"),
    (2019, 10, "3.98"),
    (2020, 1, "3.91"),
    (2020, 4, "3.89"),
    (2020, 7, "3.10"),
    (2020, 10, "3.10"),
    (2021, 1, "3.02"),
    (2021, 4, "3.01"),
    (2021, 7, "3.04"),
    (2021, 10, "3.01"),
    (2022, 1, "3.04"),
    (2022, 4, "3.07"),
    (2022, 7, "4.00"),
    (2022, 10, "5.31"),
    (2023, 1, "6.06"),
    (2023, 4, "6.46"),
    (2023, 7, "6.90"),
    (2023, 10, "7.15"),
    (2024, 1, "7.38"),
    (2024, 4, "7.34"),
    (2024, 7, "7.36"),
    (2024, 10, "7.38"),
    (2025, 1, "7.42"),
    (2025, 4, "7.17"),
    (2025, 7, "6.78"),
    (2025, 10, "6.61"),
    (2026, 1, "6.65"),
    (2026, 4, "6.96"),
    (2026, 7, "7.43"),
    (2026, 10, "7.51"),
];

/// The SIC rate for `day`: the quarter's annual rate over the days in the
/// calendar year (s 280-105(2)).
fn sic_rate(day: i64) -> Decimal {
    let (y, m, _) = dates::to_civil(day);
    let quarter = (m - 1) / 3 * 3 + 1;
    let (_, _, annual) = SIC
        .iter()
        .find(|(yy, mm, _)| *yy == y && *mm == quarter)
        .expect("a day inside the table");
    let days = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
        366
    } else {
        365
    };
    d(annual)
        .div(&Decimal::from_int(100 * days))
        .expect("finite rate")
}

/// s 138-40(2) and (3): the sum over each day of the period of the SIC rate
/// times the amount plus the earlier daily amounts, unrounded. Each daily
/// amount is proportional to the amount, so the sum for $1 from a start day,
/// worked out once, serves every contribution whose period starts then.
fn earnings(base: &Decimal, start: i64, end: i64) -> Result<Decimal> {
    let mut total = Decimal::from_int(0);
    for day in start..=end {
        total = total.add(&sic_rate(day).mul(&base.add(&total)?)?)?;
    }
    Ok(total)
}

/// The limit in all: $30,000 for a determination requested before 1 July
/// 2022, otherwise $50,000 (s 138-35(1)(a); GN 2024/1 section 3).
fn total_limit(args: &FhssArgs, determined: i64) -> Result<Decimal> {
    let Some(requested) = args.determination_requested else {
        return refuse(
            "input.missing_field",
            "determination_requested is required with contributions: it sets the $50,000 or $30,000 limit.",
        );
    };
    let requested = day(requested, "determination_requested")?;
    if requested > determined || requested < dates::parse("2018-07-01").expect("date") {
        return refuse(
            "fhss.determination_requested_invalid",
            "determination_requested must be from 1 July 2018, when requests opened, to the determination day.",
        );
    }
    Ok(if requested < dates::parse("2022-07-01").expect("date") {
        d("30000")
    } else {
        d("50000")
    })
}

/// Each contribution's earnings period start (s 138-40(3)) and the amount
/// counted in the releasable contributions amount: 85% of a concessional
/// part (s 138-30(1)(b)).
fn bases(pieces: &[Piece]) -> Result<BTreeMap<usize, (i64, Decimal)>> {
    let mut bases: BTreeMap<usize, (i64, Decimal)> = BTreeMap::new();
    let start_2017 = dates::parse("2017-07-01").expect("date");
    for p in pieces {
        let base = if p.concessional {
            d("0.85").mul(&p.counted)?
        } else {
            p.counted.clone()
        };
        let (y, m, _) = dates::to_civil(p.day);
        let start = if p.fy == 2017 {
            start_2017
        } else {
            dates::parse(&format!("{y:04}-{m:02}-01")).expect("date")
        };
        let entry = bases.entry(p.n).or_insert((start, Decimal::from_int(0)));
        entry.1 = entry.1.add(&base)?;
    }
    Ok(bases)
}

/// Each contribution's associated earnings to the determination day.
fn earned(
    bases: &BTreeMap<usize, (i64, Decimal)>,
    determined: i64,
) -> Result<BTreeMap<usize, (i64, Decimal)>> {
    let mut earned = BTreeMap::new();
    let mut per_dollar: BTreeMap<i64, Decimal> = BTreeMap::new();
    for (n, (start, base)) in bases {
        let e = if base.is_zero() {
            Decimal::from_int(0)
        } else {
            if !per_dollar.contains_key(start) {
                let unit = earnings(&Decimal::from_int(1), *start, determined)?;
                per_dollar.insert(*start, unit);
            }
            // Rounded down to the dollar for each contribution (s 138-40(2)).
            base.mul(&per_dollar[start])?.quantize(0, Rounding::Down)?
        };
        earned.insert(*n, (*start, e));
    }
    Ok(earned)
}

/// What each year counts, concessional and non-concessional, in full.
fn years(pieces: &[Piece]) -> Result<BTreeMap<i64, (Decimal, Decimal)>> {
    let zero = Decimal::from_int(0);
    let mut years: BTreeMap<i64, (Decimal, Decimal)> = BTreeMap::new();
    for p in pieces {
        let entry = years.entry(p.fy).or_insert((zero.clone(), zero.clone()));
        if p.concessional {
            entry.0 = entry.0.add(&p.counted)?;
        } else {
            entry.1 = entry.1.add(&p.counted)?;
        }
    }
    Ok(years)
}

/// The rows the output shows for each year and each contribution.
fn detail(
    pieces: &[Piece],
    earned: &BTreeMap<usize, (i64, Decimal)>,
    years: &BTreeMap<i64, (Decimal, Decimal)>,
) -> (Vec<Value>, Vec<Value>) {
    let rows = pieces
        .iter()
        .map(|p| {
            let (start, e) = &earned[&p.n];
            json!({
                "contribution": p.n,
                "date": dates::format(p.day),
                "financial_year": fy_label(p.fy),
                "kind": p.kind,
                "treated_as": if p.concessional { "concessional" } else { "non_concessional" },
                "amount": p.amount.to_string(),
                "counted": p.counted.to_string(),
                "earnings_from": dates::format(*start),
                "contribution_earnings": e.to_string(),
            })
        })
        .collect();
    let years = years
        .iter()
        .map(|(fy, (c, n))| {
            json!({"financial_year": fy_label(*fy), "concessional_counted": c.to_string(), "non_concessional_counted": n.to_string()})
        })
        .collect();
    (years, rows)
}

pub(super) fn from_contributions(args: &FhssArgs, determined: i64) -> Result<(Components, Value)> {
    let total_limit = total_limit(args, determined)?;
    let deductions = read_deductions(args.personal_deductions)?;
    let mut pieces = read_contributions(args.contributions.expect("checked"), determined)?;
    // In date order, so a year's deduction reaches its latest contributions.
    pieces.sort_by_key(|p| (p.day, p.n));
    let mut pieces = split_personal(pieces, &deductions)?;
    order(&mut pieces, &deductions);
    count(&mut pieces, &total_limit)?;
    let earned = earned(&bases(&pieces)?, determined)?;
    let by_year = years(&pieces)?;
    let zero = Decimal::from_int(0);
    let (mut cc, mut ncc, mut total_earnings) = (zero.clone(), zero.clone(), zero);
    for (c, n) in by_year.values() {
        cc = cc.add(c)?;
        ncc = ncc.add(n)?;
    }
    for (_, e) in earned.values() {
        total_earnings = total_earnings.add(e)?;
    }
    let (years, rows) = detail(&pieces, &earned, &by_year);
    let detail = json!({
        "total_limit": total_limit.to_string(),
        "counted_concessional_in_full": cc.to_string(),
        "years": years,
        "contributions": rows,
    });
    let components = Components {
        concessional: d("0.85").mul(&cc)?,
        non_concessional: ncc,
        earnings: total_earnings,
    };
    Ok((components, detail))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::precise;

    /// One quarter's compounding has a closed form: base x ((1 + r)^n - 1).
    #[test]
    fn earnings_compound_daily() {
        let start = dates::parse("2025-07-01").expect("date");
        let end = dates::parse("2025-09-30").expect("date");
        let e = precise(|| earnings(&d("10000"), start, end)).expect("earnings");
        // 10000 x ((1 + 0.0678 / 365)^92 - 1) = 172.3... by hand.
        assert_eq!(e.quantize(0, Rounding::Down).unwrap().to_string(), "172");
    }
}
