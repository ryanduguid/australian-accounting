// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `lessee-lease`: a lessee's lease under AASB 16 (compiled Standard,
//! compilation no. 8 of 31 December 2023, F2024C00049, read on 26 November
//! 2026), as the site's AASB 16 lease page works it (ryanduguid.github.io
//! `assets/lease.mjs` and `assets/finance-schedule.mjs`, checked 26 November
//! 2026): the lease liability and right-of-use asset at commencement, the
//! liability's schedule, and each income year's interest, payments,
//! depreciation and closing balances.
//!
//! The liability is the present value of the payments not paid at
//! commencement, with any residual value guarantee expected to be payable or
//! purchase option price reasonably certain to be paid, at the stated rate
//! (paras 26 and 27), rounded half up to the cent. The asset's cost adds the
//! payments at or before commencement less incentives, the initial direct
//! costs and the restoration estimate (para 24). Interest is the period rate on
//! the liability (para 37), half up to the cent, the last period taking the
//! cents rounding leaves; the asset is depreciated straight line (para 31 and
//! AASB 116 para 62) over the lease term, or the useful life where ownership
//! transfers or a purchase option is reasonably certain, or where the useful
//! life ends first (para 32). A period across 30 June is shared by days; the
//! rounding and day sharing are the site's conventions, not rules of AASB 16.

use std::collections::BTreeMap;

use num_bigint::BigInt;
use serde_json::{Value, json};

use super::{Arg, Result, d, date_arg, flag, money, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "lessee_lease";
const MOST_LIFE_MONTHS: i64 = 600;
/// Payments a year, the step in days or months, and the most payments (50 years).
const FREQUENCIES: [(&str, i64, i64, i64, i64); 5] = [
    ("weekly", 52, 7, 0, 2600),
    ("fortnightly", 26, 14, 0, 1300),
    ("monthly", 12, 0, 1, 600),
    ("quarterly", 4, 0, 3, 200),
    ("annual", 1, 0, 12, 50),
];
const MONTHLY: (i64, i64, i64) = (12, 0, 1);

/// The worksheet's inputs, as the caller gave them.
pub struct LesseeLeaseArgs<'a> {
    pub payment: &'a Arg,
    pub payments: &'a Arg,
    pub frequency: &'a Arg,
    pub timing: &'a Arg,
    pub discount_rate: &'a Arg,
    pub commencement: &'a Arg,
    pub residual: Option<&'a Arg>,
    pub prepaid: Option<&'a Arg>,
    pub incentives: Option<&'a Arg>,
    pub initial_direct_costs: Option<&'a Arg>,
    pub restoration: Option<&'a Arg>,
    pub ownership_transfers: &'a Arg,
    pub useful_life_months: Option<&'a Arg>,
    pub purchase_option: Option<&'a Arg>,
    pub remeasurement: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The lease in cents, days and millionths of 1% a year.
struct Terms {
    payment: i128,
    count: i64,
    /// Payments a year, and the step in days or months.
    freq: (i64, i64, i64),
    advance: bool,
    rate: i128,
    start: i64,
    residual: i128,
    prepaid: i128,
    incentives: i128,
    direct: i128,
    restoration: i128,
    ownership: bool,
    /// The lease contains a purchase option, reasonably certain or not.
    option: bool,
    life: Option<i64>,
}

struct Row {
    period: i64,
    date: i64,
    opening: i128,
    payment: i128,
    interest: i128,
}

fn fact_invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("lessee_lease.fact_invalid", message)
}

/// `num / den` cents, half up, for a non-negative numerator.
fn ratio(num: i128, den: i128) -> i128 {
    (2 * num + den) / (2 * den)
}

fn text(cents: i128) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let c = cents.abs();
    format!("{sign}{}.{:02}", c / 100, c % 100)
}

fn dollars(cents: i128) -> Decimal {
    d(&text(cents))
}

/// The day `count` periods after `start`: every 7 or 14 days, or the same day
/// of the month on, the month's last day where it has no such day.
fn periods_after(start: i64, count: i64, freq: (i64, i64, i64)) -> i64 {
    let (_, days, months) = freq;
    if days > 0 {
        start + count * days
    } else {
        dates::add_months(start, count * months)
    }
}

/// The first calendar year of the income year holding `day`.
fn year_starting(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    if m < 7 { y - 1 } else { y }
}

fn july(first: i64) -> i64 {
    dates::parse(&format!("{first}-07-01")).expect("1 July")
}

fn label(first: i64) -> String {
    format!("{first}-{:02}", (first + 1) % 100)
}

/// A checked amount in cents; absent is nil.
fn cents(value: Option<&Arg>) -> Result<i128> {
    let Some(value) = value else { return Ok(0) };
    let v = money(value)?.quantize(-2, Rounding::HalfUp)?;
    Ok(v.to_string().replace('.', "").parse().expect("cents"))
}

/// A decimal fraction from 0 to 1, at most 6 places, in millionths.
fn discount(value: &Arg) -> Result<i128> {
    if let Arg::Decimal(v) = value
        && v.is_finite()
        && !v.lt(&d("0"))?
        && !v.gt(&d("1"))?
        && v.exponent() >= -6
    {
        let units = v.quantize(-6, Rounding::HalfUp)?.to_string();
        return Ok(units.replace('.', "").parse().expect("millionths"));
    }
    fact_invalid("discount_rate must be a decimal fraction from 0 to 1, at most 6 places.")
}

fn schedule_shape(a: &LesseeLeaseArgs) -> Result<((i64, i64, i64), i64, bool)> {
    let Some(&(_, per_year, days, months, most)) =
        FREQUENCIES.iter().find(|f| a.frequency.text() == Some(f.0))
    else {
        return fact_invalid(
            "frequency must be one of weekly, fortnightly, monthly, quarterly, annual.",
        );
    };
    let count = match a.payments {
        Arg::Int(n) if (1..=most).contains(n) => *n,
        _ => return fact_invalid(format!("payments must be a whole number from 1 to {most}.")),
    };
    let advance = match a.timing.text() {
        Some("arrears") => false,
        Some("advance") => true,
        _ => return fact_invalid("timing must be arrears or advance."),
    };
    Ok(((per_year, days, months), count, advance))
}

fn life(value: Option<&Arg>, ownership: bool) -> Result<Option<i64>> {
    match value {
        None if ownership => refuse(
            "lessee_lease.useful_life_needed",
            "With ownership transferring or a purchase option reasonably certain, the asset is depreciated over its useful life (para 32): give useful_life_months.",
        ),
        None => Ok(None),
        Some(Arg::Int(n)) if (1..=MOST_LIFE_MONTHS).contains(n) => Ok(Some(*n)),
        Some(_) => fact_invalid(format!(
            "useful_life_months must be a whole number from 1 to {MOST_LIFE_MONTHS}."
        )),
    }
}

/// `commencement`, refused outside the income year.
fn commencement(value: &Arg, year: &str) -> Result<i64> {
    let start = date_arg(value, "commencement")?;
    if year_starting(start) != year[..4].parse::<i64>().expect("a supported year") {
        return fact_invalid(format!("commencement must fall in {year}."));
    }
    Ok(start)
}

fn terms(a: &LesseeLeaseArgs, year: &str) -> Result<Terms> {
    if flag(a.remeasurement, "remeasurement")? {
        return refuse(
            "lessee_lease.remeasurement",
            "A reassessment, lease modification, change in an index or rate or revised in-substance fixed payment remeasures the lease liability (paras 36(c) and 39 to 46), which this worksheet does not compute.",
        );
    }
    let (freq, count, advance) = schedule_shape(a)?;
    let payment = cents(Some(a.payment))?;
    if payment == 0 {
        return fact_invalid("payment must be above 0.");
    }
    let ownership = flag(a.ownership_transfers, "ownership_transfers")?;
    Ok(Terms {
        payment,
        count,
        freq,
        advance,
        rate: discount(a.discount_rate)?,
        start: commencement(a.commencement, year)?,
        residual: cents(a.residual)?,
        prepaid: cents(a.prepaid)?,
        incentives: cents(a.incentives)?,
        direct: cents(a.initial_direct_costs)?,
        restoration: cents(a.restoration)?,
        ownership,
        option: match a.purchase_option {
            Some(v) => flag(v, "purchase_option")?,
            None => false,
        },
        life: life(a.useful_life_months, ownership)?,
    })
}

/// Para 26: the payments after commencement and the residual, discounted one
/// period at a time at the period rate, half up to the cent. A payment in
/// advance falls at commencement and is left out.
fn liability(t: &Terms) -> i128 {
    let count = u32::try_from(t.count).expect("at most 2,600 payments");
    let at_start = if t.advance { t.payment } else { 0 };
    if t.rate == 0 {
        return t.payment * i128::from(t.count) + t.residual - at_start;
    }
    let (den, num) = (BigInt::from(t.freq.0 * 1_000_000), BigInt::from(t.rate));
    let k = if t.advance { &den + &num } else { den.clone() };
    let growth = (&den + &num).pow(count);
    let scale = den.pow(count);
    let worth =
        BigInt::from(t.payment) * k * (&growth - &scale) + BigInt::from(t.residual) * &scale * &num;
    let under = num * growth;
    let two = BigInt::from(2);
    let amount: BigInt = (&two * worth + &under) / (two * under);
    i128::try_from(amount).expect("cents") - at_start
}

/// Each payment after commencement, then the residual: in arrears with the
/// last payment, in advance a period after it. The last row's interest takes
/// the cents rounding leaves, so the liability closes on nil.
fn rows(t: &Terms, liability: i128) -> Result<Vec<Row>> {
    let den = i128::from(t.freq.0) * 1_000_000;
    let last = if t.advance && t.residual > 0 {
        t.count + 1
    } else {
        t.count
    };
    let mut balance = liability;
    let mut out = Vec::new();
    for period in (if t.advance { 2 } else { 1 })..=last {
        let regular = if period <= t.count { t.payment } else { 0 };
        let paid = regular + if period == last { t.residual } else { 0 };
        let interest = if period == last {
            paid - balance
        } else {
            ratio(balance * t.rate, den)
        };
        if interest < 0 {
            return refuse(
                "lessee_lease.rounding_negative",
                "Rounding each period's interest to the cent leaves the last period with negative interest; use a larger payment or fewer payments.",
            );
        }
        let at = if t.advance { period - 1 } else { period };
        out.push(Row {
            period,
            date: periods_after(t.start, at, t.freq),
            opening: balance,
            payment: paid,
            interest,
        });
        balance += interest - paid;
    }
    Ok(out)
}

/// Adds `amount` accrued from `from` to `to` to the income years, the part
/// before the last 1 July in the span by days, half up, the rest after it.
fn spread(years: &mut BTreeMap<i64, i128>, from: i64, to: i64, amount: i128) {
    let cut = july(year_starting(to - 1));
    let before = if cut > from && cut < to {
        ratio(amount * i128::from(cut - from), i128::from(to - from))
    } else {
        amount
    };
    *years.entry(year_starting(from)).or_default() += before;
    if before != amount {
        *years.entry(year_starting(cut)).or_default() += amount - before;
    }
}

/// Straight line over `count` periods from `start`: equal shares, rounding
/// spread so they add up to `cost`.
fn straight_line(cost: i128, start: i64, count: i64, freq: (i64, i64, i64)) -> BTreeMap<i64, i128> {
    let mut years = BTreeMap::new();
    let n = i128::from(count);
    for k in 1..=count {
        let share = ratio(cost * i128::from(k), n) - ratio(cost * i128::from(k - 1), n);
        let (from, to) = (
            periods_after(start, k - 1, freq),
            periods_after(start, k, freq),
        );
        spread(&mut years, from, to, share);
    }
    years
}

/// Each row's interest by income year, over the period that ends on its date.
fn interest_by_year(t: &Terms, rows: &[Row]) -> BTreeMap<i64, i128> {
    let mut years = BTreeMap::new();
    for row in rows.iter().filter(|r| r.interest != 0) {
        let back = if t.advance {
            row.period - 2
        } else {
            row.period - 1
        };
        spread(
            &mut years,
            periods_after(t.start, back, t.freq),
            row.date,
            row.interest,
        );
    }
    years
}

fn income_years(
    t: &Terms,
    rows: &[Row],
    depreciation: &BTreeMap<i64, i128>,
    opening: (i128, i128),
) -> Vec<Value> {
    let interest = interest_by_year(t, rows);
    let mut paid: BTreeMap<i64, i128> = BTreeMap::new();
    for row in rows {
        *paid.entry(year_starting(row.date)).or_default() += row.payment;
    }
    let mut labels: Vec<i64> = depreciation
        .keys()
        .chain(interest.keys())
        .chain(paid.keys())
        .copied()
        .collect();
    labels.sort_unstable();
    labels.dedup();
    let (mut owed, mut carried) = opening;
    let get = |m: &BTreeMap<i64, i128>, y: i64| m.get(&y).copied().unwrap_or(0);
    labels
        .into_iter()
        .map(|y| {
            let (i, p, dep) = (get(&interest, y), get(&paid, y), get(depreciation, y));
            owed += i - p;
            carried -= dep;
            json!({"year": label(y), "interest": text(i), "payments": text(p),
                "lease_liability": text(owed), "depreciation": text(dep),
                "right_of_use_asset": text(carried)})
        })
        .collect()
}

fn schedule(rows: &[Row], count: i64) -> Vec<Value> {
    rows.iter()
        .map(|r| {
            json!({"payment": if r.period > count { "residual".into() } else { r.period.to_string() },
                "date": dates::format(r.date), "opening": text(r.opening),
                "paid": text(r.payment), "interest": text(r.interest),
                "principal": text(r.payment - r.interest),
                "closing": text(r.opening + r.interest - r.payment)})
        })
        .collect()
}

fn journal(cost: i128, liability: i128, t: &Terms, at_start: i128) -> Vec<Value> {
    let mut out = vec![
        json!({"entry": "Debit right-of-use asset", "amount": text(cost)}),
        json!({"entry": "Credit lease liability", "amount": text(liability)}),
    ];
    if t.restoration > 0 {
        out.push(
            json!({"entry": "Credit provision for restoration", "amount": text(t.restoration)}),
        );
    }
    if at_start != 0 {
        let entry = if at_start > 0 {
            "Credit bank: paid at or before commencement, less incentives"
        } else {
            "Debit bank: incentives received, less what was paid"
        };
        out.push(json!({"entry": entry, "amount": text(at_start.abs())}));
    }
    out
}

/// Para 32: over the useful life where ownership transfers or it ends before
/// the lease term, else over the lease term's own periods.
fn depreciation(t: &Terms, cost: i128, end: i64) -> (BTreeMap<i64, i128>, i64) {
    match t.life {
        Some(months) if t.ownership || periods_after(t.start, months, MONTHLY) < end => (
            straight_line(cost, t.start, months, MONTHLY),
            periods_after(t.start, months, MONTHLY),
        ),
        _ => (straight_line(cost, t.start, t.count, t.freq), end),
    }
}

/// The lease at commencement and by income year to its end.
pub fn lessee_lease(a: &LesseeLeaseArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let t = terms(a, &year)?;
    let owed = liability(&t);
    let first = if t.advance { t.payment } else { 0 };
    let at_start = t.prepaid + first + t.direct - t.incentives;
    let cost = owed + at_start + t.restoration;
    if cost <= 0 {
        return refuse(
            "lessee_lease.cost_nil",
            "The incentives received are as much as the rest of the right-of-use asset's cost, which leaves it nothing to depreciate.",
        );
    }
    let rows = rows(&t, owed)?;
    let end = periods_after(t.start, t.count, t.freq);
    let (charges, depreciated_to) = depreciation(&t, cost, end);
    let total_interest: i128 = rows.iter().map(|r| r.interest).sum();
    let amounts = vec![
        ("lease_liability", dollars(owed)),
        ("right_of_use_asset", dollars(cost)),
        ("total_interest", dollars(total_interest)),
        ("restoration_provision", dollars(t.restoration)),
    ];
    let mut out = result(KIND, &year, amounts, vec![])?;
    out["lease_term_ends"] = json!(dates::format(end));
    out["depreciated_to"] = json!(dates::format(depreciated_to));
    // Appendix A: a lease term of 12 months or less, with no purchase option.
    let no_option = !t.ownership && !t.option;
    out["short_term"] = json!(end <= periods_after(t.start, 12, MONTHLY) && no_option);
    out["income_years"] = Value::Array(income_years(&t, &rows, &charges, (owed, cost)));
    out["schedule"] = Value::Array(schedule(&rows, t.count));
    out["journal"] = Value::Array(journal(cost, owed, &t, at_start));
    Ok(out)
}
