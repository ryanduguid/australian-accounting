// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! One year of an amalgamated loan under ITAA 1936 s 109E(7): the interest at
//! the benchmark rate on the balance from time to time, and the amount of the
//! loan not repaid at the end of the year.
//!
//! s 109E(7) sets the rate but no day count. The day count is the ATO's, from
//! the worked example in 'Loans by private companies' (interest 'calculated
//! annually in arrears by reference to the daily balance', a repayment
//! reducing the balance from the day it is made, 61, 272 and 32 days over
//! 365) and from its Division 7A calculator, which divides by 366 in a year of
//! income holding 29 February and rounds each period's interest to cents.

use serde_json::{Value, json};

use crate::decimal::{Calc, Decimal, with_precision};
use crate::money::{FORMULA_PRECISION, ROUNDING, cents_str, d, parse_money, to_cents};
use crate::myr::GENUINE_REPAYMENT_CAVEAT;
use crate::rates::benchmark_rate;
use crate::raw::Raw;
use crate::years::{YearOfIncome, parse_year};

const KEYS: [&str; 4] = [
    "year_of_income",
    "year_loan_made",
    "balance_at_start_of_year",
    "repayments",
];

/// Days since 1970-01-01 of a proleptic Gregorian date (Hinnant's algorithm).
fn day_number(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil(n: i64) -> String {
    let z = n + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// A calendar date written YYYY-MM-DD, as its day number.
fn parse_date(text: &str, where_: &str) -> Result<i64, String> {
    let shown = Raw::text(text).py_repr();
    let refuse = || format!("{where_} {shown} must be a date written like 2026-10-01");
    let bytes = text.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return Err(refuse());
    }
    let part = |range: std::ops::Range<usize>| text[range].parse::<i64>().expect("digits");
    let (year, month, day) = (part(0..4), part(5..7), part(8..10));
    if !(1..=12).contains(&month) || day < 1 {
        return Err(refuse());
    }
    let n = day_number(year, month, day);
    if civil(n) != text {
        return Err(format!("{where_} {shown} is not a calendar date"));
    }
    Ok(n)
}

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    match v.get(key) {
        None | Some(Value::Null) => Err(format!("{key} is required")),
        Some(Value::String(s)) => Ok(s),
        Some(_) => Err(format!("{key} must be a string")),
    }
}

/// One period of unchanged balance: its first day, its days and the balance.
struct Period {
    from: i64,
    days: i64,
    balance: Decimal,
}

/// Balance x rate x days / days in the year, to cents half up.
fn period_interest(period: &Period, rate: &Decimal, days_in_year: i64) -> Calc<Decimal> {
    let raw = with_precision(FORMULA_PRECISION, || {
        period
            .balance
            .mul(rate)?
            .mul(&Decimal::from_int(period.days))?
            .div(&Decimal::from_int(days_in_year))
    })?;
    to_cents(&raw)
}

/// Work out one year of income for the `rollforward` command. Every refusal is
/// an `Err` with the reason, as for the bare s 109E(6) formula.
pub fn rollforward(v: &Value) -> Result<Value, String> {
    let Some(input) = v.as_object() else {
        return Err("input must be a JSON object".into());
    };
    if let Some(key) = input.keys().find(|k| !KEYS.contains(&k.as_str())) {
        return Err(format!(
            "{} is not a rollforward input; it takes {}",
            Raw::text(key).py_repr(),
            KEYS.join(", ")
        ));
    }
    let year = parse_year(&Raw::text(text(v, "year_of_income")?), "year_of_income")?;
    let made = parse_year(&Raw::text(text(v, "year_loan_made")?), "year_loan_made")?;
    if made >= year {
        return Err(format!(
            "year_loan_made {} is not before year_of_income {}. The ATO's guidance treats no \
             interest as payable for the year the loan is made, and s 109E(1)(a) reaches an \
             amalgamated loan only in a later year of income.",
            made.label(),
            year.label()
        ));
    }
    let rate = benchmark_rate(year, None, None);
    let Some(benchmark) = rate.rate.clone().filter(|_| rate.known) else {
        return Err(format!(
            "No reviewed benchmark rate for {}. rollforward reads only the compiled-in rate \
             table and does not extrapolate from an adjacent year.",
            year.label()
        ));
    };
    let opening = parse_money(
        &Raw::text(text(v, "balance_at_start_of_year")?),
        "balance_at_start_of_year",
    )?;
    let Some(Value::Array(items)) = v.get("repayments") else {
        return Err("repayments must be a list of {\"date\", \"amount\"} objects".into());
    };
    let (start, end) = bounds(year);
    let mut repayments: Vec<(i64, Decimal)> = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let at = format!("repayments[{i}]");
        if !item
            .as_object()
            .is_some_and(|o| o.keys().all(|k| k == "date" || k == "amount"))
        {
            return Err(format!(
                "{at} must be an object with only \"date\" and \"amount\""
            ));
        }
        let date = parse_date(
            text(item, "date").map_err(|e| format!("{at} {e}"))?,
            &format!("{at}.date"),
        )?;
        if !(start..end).contains(&date) {
            return Err(format!(
                "{at}.date {} is outside {} ({} to {})",
                civil(date),
                year.label(),
                civil(start),
                civil(end - 1)
            ));
        }
        let amount = parse_money(
            &Raw::text(text(item, "amount").map_err(|e| format!("{at} {e}"))?),
            &format!("{at}.amount"),
        )?;
        // A nil repayment repays nothing, but it would split a period and so
        // can move the rounded interest by a cent.
        if amount.equals(&d("0")).expect("finite") {
            return Err(format!(
                "{at}.amount is {}; a repayment of nil repays nothing, so leave it out",
                cents_str(&amount).expect("bounded amounts quantise")
            ));
        }
        repayments.push((date, amount));
    }
    repayments.sort_by_key(|(date, _)| *date);
    let mut result = compute(year, made, &benchmark, opening, &repayments)
        .map_err(|s| format!("decimal.{}", s.name()))??;
    result["benchmark_provenance"] = rate.provenance();
    result["manifest"] = rate.manifest();
    Ok(result)
}

/// 1 July of the year of income and 1 July of the next, as day numbers.
fn bounds(year: YearOfIncome) -> (i64, i64) {
    (
        day_number(year.start_year, 7, 1),
        day_number(year.start_year + 1, 7, 1),
    )
}

/// The arithmetic, apart from reading the input, so the ATO's worked example
/// (a year outside the frozen table) can be run with its own rate.
fn compute(
    year: YearOfIncome,
    made: YearOfIncome,
    rate: &Decimal,
    opening: Decimal,
    repayments: &[(i64, Decimal)],
) -> Calc<Result<Value, String>> {
    let zero = d("0");
    let nil = |v: Decimal| -> Calc<Decimal> { Ok(if v.lt(&zero)? { zero.clone() } else { v }) };
    let (start, end) = bounds(year);
    let days_in_year = end - start;
    // A repayment reduces the balance from its own date: the ATO's example
    // charges $75,000 to 30 August and $55,000 from the 31 August repayment.
    // Once repayments pass the opening balance there is nothing left to charge,
    // so the balance is taken as nil, as the ATO calculator floors it.
    let mut periods: Vec<Period> = Vec::new();
    let mut balance = opening.clone();
    let mut from = start;
    let mut applied: Vec<Value> = Vec::new();
    // The first day of the first period charged on a balance floored at nil.
    let mut floored_from: Option<i64> = None;
    for (date, amount) in repayments {
        if *date > from {
            if floored_from.is_none() && balance.lt(&zero)? {
                floored_from = Some(from);
            }
            periods.push(Period {
                from,
                days: date - from,
                balance: nil(balance.clone())?,
            });
            from = *date;
        }
        balance = balance.sub(amount)?;
        applied.push(json!({
            "date": civil(*date),
            "amount": cents_str(amount)?,
            "balance_after": cents_str(&nil(balance.clone())?)?,
        }));
    }
    if floored_from.is_none() && balance.lt(&zero)? {
        floored_from = Some(from);
    }
    periods.push(Period {
        from,
        days: end - from,
        balance: nil(balance)?,
    });
    let mut interest = d("0.00");
    let mut rows = Vec::new();
    for period in &periods {
        let charged = period_interest(period, rate, days_in_year)?;
        interest = interest.add(&charged)?;
        rows.push(json!({
            "from": civil(period.from),
            "to": civil(period.from + period.days - 1),
            "days": period.days,
            "balance": cents_str(&period.balance)?,
            "interest": cents_str(&charged)?,
        }));
    }
    let paid = repayments
        .iter()
        .try_fold(d("0.00"), |sum, (_, amount)| sum.add(amount))?;
    if paid.gt(&opening.add(&interest)?)? {
        return Ok(Err(format!(
            "repayments total {}, more than the balance at the start of the year, {}, and \
             the year's interest, {}. The excess repays no part of this loan; check the \
             repayments and the opening balance.",
            cents_str(&paid)?,
            cents_str(&opening)?,
            cents_str(&interest)?
        )));
    }
    // s 109E(7): payments meet the year's interest first; only the rest repays the loan.
    let to_interest = if paid.lt(&interest)? {
        paid.clone()
    } else {
        interest.clone()
    };
    let to_principal = paid.sub(&to_interest)?;
    let closing = opening.sub(&to_principal)?;
    let rate_text = rate.to_sci_string();
    let mut caveats = vec![
        format!(
            "Interest is worked out as the ATO does in 'Loans by private companies' and its \
             Division 7A calculator: on the daily balance in arrears, a repayment reducing the \
             balance from the day it is made, each period's interest being balance x {rate_text} \
             x days / {days_in_year}, rounded to cents half up. s 109E(7) sets the rate, not the \
             day count or rounding."
        ),
        GENUINE_REPAYMENT_CAVEAT.to_string(),
        "Interest is at the benchmark rate whatever rate the loan agreement sets (s 109E(7)), \
         so where the agreement's rate is higher the closing amount is notional, not the \
         balance the agreement gives."
            .to_string(),
    ];
    if days_in_year == 366 {
        caveats.push(format!(
            "{} holds 29 February, so the divisor is 366, as in the ATO calculator. Neither \
             s 109E(7) nor the ATO's worked example, which covers a 365-day year, states the \
             divisor for such a year.",
            year.label()
        ));
    }
    if let Some(day) = floored_from {
        caveats.push(format!(
            "From {} the repayments exceed the balance, so the balance is taken as nil and \
             charged no interest, as in the ATO calculator. Neither s 109E(7) nor the ATO's \
             worked example says how a balance below nil is treated.",
            civil(day)
        ));
    }
    if year.start_year == made.start_year + 1 {
        caveats.push(
            "This is the first year after the loan year. Enter the balance at 1 July, before \
             any repayment made before the lodgment day, and list those repayments here, as \
             the ATO's worked example in 'Loans by private companies' does. The paragraph \
             before that example says payments made before the lodgment day are taken to have \
             been made in the loan year, so the first year opens at the balance immediately \
             before the lodgment day; the guidance does not reconcile the two. The example's \
             method never gives the lower closing balance, as it charges interest on such a \
             repayment from 1 July to its date. That year's minimum yearly repayment can work \
             on a lower balance (s 109E(3))."
                .to_string(),
        );
    }
    if paid.lt(&interest)? {
        caveats.push(
            "The repayments fall short of the year's interest. The unpaid interest is not \
             added to the amount of the loan: s 109E(7) asks how much of the loan is repaid, \
             and none of it is."
                .to_string(),
        );
    }
    Ok(Ok(json!({
        "year_of_income": year.label(),
        "year_loan_made": made.label(),
        "benchmark_rate": rate_text,
        // Filled in from the rate table by `rollforward`.
        "benchmark_provenance": Value::Null,
        "manifest": Value::Null,
        "balance_at_start_of_year": cents_str(&opening)?,
        "days_in_year": days_in_year,
        "repayments": applied,
        "periods": rows,
        "interest": cents_str(&interest)?,
        "repayments_total": cents_str(&paid)?,
        "applied_to_interest": cents_str(&to_interest)?,
        "applied_to_principal": cents_str(&to_principal)?,
        "amalgamated_loan_unpaid_at_end_of_year": cents_str(&closing)?,
        "rounding": ROUNDING,
        "caveats": caveats,
        "statutory_trace": [
            "ITAA 1936 s 109E(7): the amount of an amalgamated loan repaid by the end of a year \
             of income is worked out on the basis that interest is payable on the balance of \
             the loan from time to time at the benchmark interest rate for the year.",
            format!("Benchmark interest rate for {} (s 109N(2)) = {rate_text}.", year.label()),
            format!(
                "Interest = sum of the periods' interest = {}; repayments = {}; applied to \
                 interest = {}; applied to the loan = {}.",
                cents_str(&interest)?,
                cents_str(&paid)?,
                cents_str(&to_interest)?,
                cents_str(&to_principal)?
            ),
            format!(
                "Amount not repaid at the end of {} = {} - {} = {}, the next year's \
                 amalgamated_loan_unpaid_at_end_of_previous_year.",
                year.label(),
                cents_str(&opening)?,
                cents_str(&to_principal)?,
                cents_str(&closing)?
            ),
        ],
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> i64 {
        parse_date(text, "date").unwrap()
    }

    /// ATO, 'Loans by private companies', example 'amount of amalgamated not
    /// repaid by the end of the income year after the loan is made' (page last
    /// updated 2 July 2026, read 13 November 2026): $75,000 at 1 July 2014,
    /// $20,000 repaid on 31 August 2014 and $8,000 on 30 May 2015, at the
    /// 2014-15 benchmark of 5.95%. The ATO gives $745.79 + $2,438.68 +
    /// $245.17 = $3,430 'rounded to the nearest dollar' and $50,430 not repaid.
    #[test]
    fn the_atos_worked_example() {
        let year = YearOfIncome::new(2014).unwrap();
        let made = YearOfIncome::new(2013).unwrap();
        let repayments = [
            (date("2014-08-31"), d("20000.00")),
            (date("2015-05-30"), d("8000.00")),
        ];
        let result = compute(year, made, &d("0.0595"), d("75000.00"), &repayments)
            .unwrap()
            .unwrap();
        let periods: Vec<(i64, &str, &str)> = result["periods"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                (
                    p["days"].as_i64().unwrap(),
                    p["balance"].as_str().unwrap(),
                    p["interest"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            periods,
            [
                (61, "75000.00", "745.79"),
                (272, "55000.00", "2438.68"),
                (32, "47000.00", "245.17"),
            ]
        );
        assert_eq!(result["periods"][0]["to"], "2014-08-30");
        assert_eq!(result["days_in_year"], 365);
        assert_eq!(result["interest"], "3429.64");
        assert_eq!(result["amalgamated_loan_unpaid_at_end_of_year"], "50429.64");
        // The ATO's whole-dollar figures.
        let dollars = |key: &str| {
            d(result[key].as_str().unwrap())
                .quantize(0, crate::decimal::Rounding::HalfUp)
                .unwrap()
        };
        assert!(dollars("interest").equals(&d("3430")).unwrap());
        assert!(
            dollars("amalgamated_loan_unpaid_at_end_of_year")
                .equals(&d("50430"))
                .unwrap()
        );
        // The README's other reading: the 31 August repayment, made before the
        // 15 May 2015 lodgment day, taken as made in the loan year.
        let other = compute(
            year,
            made,
            &d("0.0595"),
            d("55000.00"),
            &[(date("2015-05-30"), d("8000.00"))],
        )
        .unwrap()
        .unwrap();
        assert_eq!(other["amalgamated_loan_unpaid_at_end_of_year"], "50230.77");
    }

    #[test]
    fn it_refuses_what_it_cannot_work_out() {
        let base = json!({"year_of_income": "2026-27", "year_loan_made": "2025-26",
            "balance_at_start_of_year": "1000.00", "repayments": []});
        let with = |key: &str, value: Value| {
            let mut input = base.clone();
            input[key] = value;
            input
        };
        let cases = [
            (
                with("year_loan_made", json!("2026-27")),
                "year_loan_made 2026-27 is not before",
            ),
            (
                with("year_loan_made", json!("2027-28")),
                "year_loan_made 2027-28 is not before",
            ),
            (
                with("year_of_income", json!("2027-28")),
                "No reviewed benchmark rate for 2027-28",
            ),
            (
                with("override", json!("rates.json")),
                "'override' is not a rollforward input",
            ),
            (
                with("balance_at_start_of_year", json!("-1.00")),
                "balance_at_start_of_year is '-1.00'",
            ),
            (with("repayments", json!({})), "repayments must be a list"),
            (
                with("repayments", json!([{"date": "2026-10-01"}])),
                "repayments[0] amount is required",
            ),
            (
                with(
                    "repayments",
                    json!([{"date": "2027-07-01", "amount": "1.00"}]),
                ),
                "repayments[0].date 2027-07-01 is outside 2026-27 (2026-07-01 to 2027-06-30)",
            ),
            (
                with(
                    "repayments",
                    json!([{"date": "1 Oct 2026", "amount": "1.00"}]),
                ),
                "repayments[0].date '1 Oct 2026' must be a date",
            ),
            (
                with(
                    "repayments",
                    json!([{"date": "2026-10-01", "amount": "1.00", "note": "x"}]),
                ),
                "repayments[0] must be an object",
            ),
            // 1,000.00 plus 92 days' interest of 22.11 (1,000 x 0.0877 x 92 / 365
            // = 22.1052), then nothing left to charge.
            (
                with(
                    "repayments",
                    json!([{"date": "2026-10-01", "amount": "1022.12"}]),
                ),
                "repayments total 1022.12, more than",
            ),
        ];
        for (input, start) in cases {
            let refused = rollforward(&input).unwrap_err();
            assert!(refused.starts_with(start), "{input}: {refused}");
        }
        let paid_off = rollforward(&with(
            "repayments",
            json!([{"date": "2026-10-01", "amount": "1022.11"}]),
        ))
        .unwrap();
        assert_eq!(paid_off["amalgamated_loan_unpaid_at_end_of_year"], "0.00");
        // A cent short: the daily balance is nil from 1 October, but 1 cent of
        // the year's interest is not covered, so the loan keeps 0.01.
        let short = rollforward(&with(
            "repayments",
            json!([{"date": "2026-10-01", "amount": "1022.10"}]),
        ))
        .unwrap();
        assert_eq!(short["amalgamated_loan_unpaid_at_end_of_year"], "0.01");
        assert_eq!(paid_off["periods"][1]["balance"], "0.00");
        assert_eq!(rollforward(&base).unwrap()["interest"], "87.70");
    }

    #[test]
    fn dates_are_strict_calendar_dates() {
        assert_eq!(civil(date("2024-02-29")), "2024-02-29");
        assert_eq!(date("2015-07-01") - date("2014-07-01"), 365);
        assert_eq!(date("2024-07-01") - date("2023-07-01"), 366);
        for bad in [
            "2023-02-29",
            "2026-13-01",
            "2026-04-31",
            "2026-1-01",
            "26-10-01",
            "2026/10/01",
            "2026-10-00",
        ] {
            assert!(parse_date(bad, "date").is_err(), "{bad}");
        }
    }
}
