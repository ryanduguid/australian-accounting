// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A car the taxpayer owned or leased: the cents per kilometre method
//! (ITAA 1997 s 28-25) and the log book method (ss 28-90, 28-115, 28-120).

use serde_json::{Value, json};

use super::{
    Arg, COMMAND, Result, Standard, conflict, d, finish, flag, int, invalid, money, named, py_min,
    rate, refuse, step, with_return,
};

const KILOMETRE_CAP: &str = "5000";

const CAP: &str = "ITAA 1997 s 28-25(2): only the first 5,000 business kilometres the car travelled in the income year count; s 28-25(3), business kilometres are a reasonable estimate";
const CENTS_RATE: &str = "ITAA 1997 s 28-25(1) and (4): business kilometres x the rate of cents per kilometre the Commissioner determines for the income year";
const BUSINESS_USE: &str = "ITAA 1997 s 28-90(3) and (5): business kilometres in the period the car was held in the income year, a reasonable estimate taking the log book into account, over the total kilometres it travelled in that period, as a percentage";
const LOG_BOOK: &str = "ITAA 1997 s 28-90(1) and (2): each car expense deductible outside Division 28 x the business use percentage";

/// The cents per kilometre rate (s 28-25(4)) in dollars: the instruments of
/// 2020 (F2020L00676, 72c, also for 2021-22), 2022 (F2022L00813, 78c), 2023
/// (F2023L00767, 85c), 2024 (F2024L00697, 88c for 2024-25 and 2025-26) and
/// 2026 (F2026L00785, 91c for 2026-27).
fn rate_per_kilometre(year: i64) -> &'static str {
    match year {
        2020 | 2021 => "0.72",
        2022 => "0.78",
        2023 => "0.85",
        2024 | 2025 => "0.88",
        _ => "0.91",
    }
}

fn car_held(value: &Arg) -> Result<()> {
    if flag(value, "owned_or_leased")? {
        return Ok(());
    }
    refuse(
        "work_expenses.car_not_held",
        "Division 28's methods are for a car the taxpayer owned or leased (ITAA 1997 s 28-12(1)); for any other car the worksheet does not work out a deduction.",
    )
}

pub(super) fn cents_per_kilometre(
    given: &[(String, Arg)],
    when: &str,
    year: &str,
    first: i64,
    standard: Option<&Standard>,
) -> Result<Value> {
    let names = ["owned_or_leased", "business_kilometres"];
    let get = named(given, COMMAND, when, &names)?;
    car_held(get("owned_or_leased"))?;
    let kilometres = money(get("business_kilometres"))?;
    let cap = d(KILOMETRE_CAP);
    let per_kilometre = d(rate_per_kilometre(first));
    let counted = py_min(kilometres.clone(), cap.clone())?;
    let deduction = counted.mul(&per_kilometre)?;
    let mut steps = vec![step(
        "claimed_kilometres",
        CAP,
        "min(business_kilometres, kilometre_cap)",
        &[&kilometres, &cap],
        counted.clone(),
    )];
    let total = step(
        "deduction",
        CENTS_RATE,
        "claimed_kilometres x rate_per_kilometre",
        &[&counted, &per_kilometre],
        deduction,
    );
    steps.extend(with_return(total, "return_amount")?);
    let rates = vec![
        rate("rate_per_kilometre", &per_kilometre),
        rate("kilometre_cap", &cap),
    ];
    let mut out = finish(year, steps, rates, standard, "return_amount")?;
    out["method"] = json!("car_cents_per_kilometre");
    out["kilometres_discarded"] = json!(kilometres.gt(&cap)?);
    Ok(out)
}

/// An income year written as `YYYY-YY`: its first calendar year.
fn income_year(value: &Arg, name: &str) -> Result<i64> {
    let parsed = value.text().and_then(|t| {
        let (a, b) = t.split_once('-')?;
        let first: i64 = a.parse().ok()?;
        let ok = a.len() == 4 && b.len() == 2 && b.parse::<i64>().ok()? == (first + 1) % 100;
        ok.then_some(first)
    });
    parsed.map_or_else(
        || invalid(format!("{name} must be an income year YYYY-YY.")),
        Ok,
    )
}

/// The log book's year, once it is shown to serve `year`, to cover at least
/// 12 weeks and still to represent the car's business use.
fn log_book_kept<'a>(get: impl Fn(&str) -> &'a Arg, year: &str, first: i64) -> Result<i64> {
    let (kept, weeks) = (get("log_book_year"), get("log_book_weeks"));
    let kept = income_year(kept, "log_book_year")?;
    if kept > first {
        return conflict("log_book_year cannot be after the year claimed.");
    }
    if first - kept > 4 {
        return refuse(
            "work_expenses.log_book_expired",
            format!(
                "A log book kept for {kept}-{:02} serves that year and the next 4 only (ITAA 1997 s 28-115(2)); a new one is needed for {year}.",
                (kept + 1) % 100
            ),
        );
    }
    let weeks = match weeks.int() {
        Some(Some(n)) if n >= 0 => n,
        _ => return invalid("log_book_weeks must be a whole number of weeks."),
    };
    if weeks < 12 {
        return refuse(
            "work_expenses.log_book_too_short",
            "A log book must cover a continuous period of at least 12 weeks (ITAA 1997 s 28-120(1)); the shorter period for a car held under 12 weeks is outside the worksheet.",
        );
    }
    if !flag(get("log_book_representative"), "log_book_representative")? {
        return refuse(
            "work_expenses.log_book_unrepresentative",
            "A log book that no longer represents the car's business use, as after a change of job or home, does not serve: a new 12 week log book is needed (ATO 'Logbook method').",
        );
    }
    Ok(kept)
}

/// The log book method's inputs.
const LOG_BOOK_INPUTS: [&str; 7] = [
    "owned_or_leased",
    "business_kilometres",
    "total_kilometres",
    "car_expenses",
    "log_book_year",
    "log_book_weeks",
    "log_book_representative",
];

pub(super) fn log_book(
    given: &[(String, Arg)],
    when: &str,
    year: &str,
    first: i64,
    standard: Option<&Standard>,
) -> Result<Value> {
    let get = named(given, COMMAND, when, &LOG_BOOK_INPUTS)?;
    car_held(get("owned_or_leased"))?;
    let kept = log_book_kept(&get, year, first)?;
    let business = money(get("business_kilometres"))?;
    let total = money(get("total_kilometres"))?;
    let expenses = money(get("car_expenses"))?;
    if total.is_zero() || business.gt(&total)? {
        return conflict(
            "total_kilometres must be above 0 and at least business_kilometres (s 28-90(3)).",
        );
    }
    let hundred = int(100);
    let percentage = business.div(&total)?.mul(&hundred)?;
    let deduction = expenses.mul(&percentage)?.div(&hundred)?;
    let mut steps = vec![step(
        "business_use_percentage",
        BUSINESS_USE,
        "business_kilometres / total_kilometres x 100",
        &[&business, &total],
        percentage.clone(),
    )];
    let total = step(
        "deduction",
        LOG_BOOK,
        "car_expenses x business_use_percentage / 100",
        &[&expenses, &percentage],
        deduction,
    );
    steps.extend(with_return(total, "return_amount")?);
    let mut out = finish(year, steps, Vec::new(), standard, "return_amount")?;
    out["method"] = json!("car_log_book");
    out["log_book_valid_to"] = json!(format!("{}-{:02}", kept + 4, (kept + 5) % 100));
    Ok(out)
}
