// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `work_expenses`: an employee's working from home, car and self-education
//! deductions for an income year from 2020-21 to 2026-27, one method a call.
//! The car rules are ITAA 1997 Division 28 (ss 28-12, 28-25, 28-90, 28-115
//! and 28-120) and the Commissioner's cents per kilometre instruments under
//! s 28-25(4); the working from home rates are PCG 2023/1 Table 1 from
//! 2022-23 and, before it, the ATO's 52 cent fixed rate and 80 cent shortcut
//! (PCG 2020/3); the self-education reduction is ITAA 1936 s 82A, repealed
//! for 2022-23 onwards by Act No. 84, 2022, Schedule 3 items 26 and 33. All
//! were read on 22 November 2026. Hours, kilometres and each expense's
//! work-related amount are established facts: the worksheet applies the
//! rates, cap, coverage and reduction to them.

use serde_json::{Value, json};

use super::fbt_benefits::named;
use super::{
    Arg, Result, Step, d, derived_result, flag, int, money, precise, py_max, py_min, rate, refuse,
    scope, step,
};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "work_expenses";
const COMMAND: &str = "work-expenses";

/// The methods `method` accepts, in the catalogue's order.
pub const METHODS: [&str; 4] = [
    "home_office",
    "car_cents_per_kilometre",
    "car_log_book",
    "self_education",
];

/// Each input only some methods or years take, and whether it is a decimal.
pub const WORK_EXPENSES_OPTIONAL: [(&str, bool); 24] = [
    ("hours", true),
    ("energy", true),
    ("phone", true),
    ("internet", true),
    ("stationery", true),
    ("cleaning", true),
    ("furniture_decline", true),
    ("asset_decline", true),
    ("repairs", true),
    ("dedicated_home_office", false),
    ("owned_or_leased", false),
    ("business_kilometres", true),
    ("total_kilometres", true),
    ("car_expenses", true),
    ("log_book_year", false),
    ("log_book_weeks", false),
    ("connection", false),
    ("employed_when_incurred", false),
    ("prescribed_course", false),
    ("category_a", true),
    ("category_b", true),
    ("category_c", true),
    ("category_d", true),
    ("category_e", true),
];

const HOME_OFFICE: [&str; 9] = [
    "hours",
    "energy",
    "phone",
    "internet",
    "stationery",
    "cleaning",
    "furniture_decline",
    "asset_decline",
    "repairs",
];

/// s 28-25(2): only the first 5,000 business kilometres count.
const KILOMETRE_CAP: &str = "5000";
/// ITAA 1936 s 82A, for expenses before 2022-23.
const SELF_EDUCATION_REDUCTION: &str = "250";

const FIXED_RATE_PCG: &str = "PCG 2023/1 paras 23 to 26 and Table 1 (para 26A): hours worked from home in the income year x the fixed rate for the year, which covers energy, internet, mobile and home phone usage, and stationery and computer consumables (para 14)";
const FIXED_RATE_52: &str = "ATO 'Fixed rate method - 52 cents' (1 July 2018 to 30 June 2022, PS LA 2001/6 section 5, retired by PCG 2023/1 para 17): 52c for each hour worked from home with a dedicated home office, covering energy for heating, cooling and lighting, cleaning the home office and the decline in value of home office furniture and furnishings";
const SEPARATE_PCG: &str = "PCG 2023/1 paras 15 and 26(c): the work-related decline in value of depreciating assets, their repairs and running expenses the rate does not cover (cleaning) are claimed beside it";
const SEPARATE_52: &str = "ATO 'Fixed rate method - 52 cents': phone, internet, stationery and computer consumables, the decline in value of depreciating assets other than home office furniture and furnishings, and repairs are claimed beside the 52c rate at their work-related amounts";
const FIXED_TOTAL: &str = "PCG 2023/1 para 26(d), and the ATO's 52 cent method before 2022-23: the hourly amount plus the expenses claimed beside it";
const SHORTCUT: &str = "PCG 2020/3 and ATO 'Shortcut method' (2020-21 and 2021-22): 80c for each hour worked from home, covering every working from home expense, with nothing claimed beside it";
const ACTUAL: &str = "ATO 'Actual cost method' and ITAA 1997 s 8-1: the work-related portion of each additional running expense and of each depreciating asset's decline in value, as apportioned";
const WHOLE_DOLLARS: &str = "TAA 1953 Sch 1 s 388-85 as PCG 2023/1's footnotes and the ATO's examples apply it: the return amount is the total with cents disregarded, not rounded";
const CAP: &str = "ITAA 1997 s 28-25(2): only the first 5,000 business kilometres the car travelled in the income year count; s 28-25(3), business kilometres are a reasonable estimate";
const CENTS_RATE: &str = "ITAA 1997 s 28-25(1) and (4): business kilometres x the rate of cents per kilometre the Commissioner determines for the income year";
const BUSINESS_USE: &str = "ITAA 1997 s 28-90(3) and (5): business kilometres in the period the car was held in the income year, a reasonable estimate taking the log book into account, over the total kilometres it travelled in that period, as a percentage";
const LOG_BOOK: &str = "ITAA 1997 s 28-90(1) and (2): each car expense deductible outside Division 28 x the business use percentage";
const OFFSET: &str = "ITAA 1936 s 82A as the ATO's 'Self-education reduction in expenses' and TR 98/9 paras 119 to 155 apply it: category C, D and E expenses (repairs, cents per kilometre car expenses and expenses incurred but not deductible) offset the $250";
const REDUCTION: &str = "ITAA 1936 s 82A: the $250 reduction less the offset, never below nil, for a prescribed course of education before 2022-23";
const REDUCED_A: &str = "ITAA 1936 s 82A: category A expenses less the reduction, never below nil";
const SELF_EDUCATION: &str = "ITAA 1997 s 8-1, TR 2024/3 and the ATO's 'Self-education expenses': categories A to D, connected to current employment when incurred; ITAA 1936 s 82A reduces category A before 2022-23 only (Act No. 84, 2022, Schedule 3 items 26 and 33)";
const NOT_CONNECTED: &str = "ITAA 1997 s 8-1, TR 2024/3 and the ATO's 'Self-education expenses': nothing is deductible when, at the time the expense was incurred, the taxpayer was not employed, or the self-education only related in a general way to the employment or enabled new employment";

/// The income year's first calendar year.
fn start(year: &str) -> i64 {
    year[..4].parse().expect("income year")
}

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

/// The working from home fixed rate per hour in dollars: 52c to 2021-22,
/// then PCG 2023/1 Table 1, 67c for 2022-23 and 2023-24 and 70c from 1 July
/// 2024.
fn fixed_rate(year: i64) -> &'static str {
    match year {
        2020 | 2021 => "0.52",
        2022 | 2023 => "0.67",
        _ => "0.70",
    }
}

fn invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("work_expenses.fact_invalid", message.into())
}

fn conflict<T>(message: impl Into<String>) -> Result<T> {
    refuse("work_expenses.fact_conflict", message.into())
}

fn text<'a>(value: &'a Arg, name: &str, allowed: &[&str]) -> Result<&'a str> {
    match value.text() {
        Some(t) if allowed.contains(&t) => Ok(t),
        _ => invalid(format!("{name} must be {}.", allowed.join(" or "))),
    }
}

fn sum(values: &[&Decimal]) -> Result<Decimal> {
    values.iter().try_fold(int(0), |total, v| Ok(total.add(v)?))
}

fn floor(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(0, Rounding::Down)?)
}

/// `work-expenses`.
pub fn work_expenses(
    method: &Arg,
    reimbursed: &Arg,
    allowance_received: &Arg,
    given: &[(String, Arg)],
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let Some(method) = method
        .text()
        .and_then(|m| METHODS.iter().find(|x| **x == m))
    else {
        return refuse(
            "work_expenses.method_invalid",
            format!("method must be {}.", METHODS.join(", ")),
        );
    };
    if flag(reimbursed, "reimbursed")? {
        return refuse(
            "work_expenses.reimbursed",
            "An expense reimbursed or paid for by an employer or anyone else is outside the worksheet: give only expenses the taxpayer incurred and bore (PCG 2023/1 para 50).",
        );
    }
    if flag(allowance_received, "allowance_received")? {
        return refuse(
            "work_expenses.allowance_received",
            "An allowance for these expenses brings in the reasonable allowance substantiation exceptions and occupation-specific guidance, which the worksheet does not apply.",
        );
    }
    let when = format!(" with method {method} in {year}");
    let first = start(&year);
    precise(|| match *method {
        "home_office" => home_office(given, &when, &year, first),
        "car_cents_per_kilometre" => cents_per_kilometre(given, &when, &year, first),
        "car_log_book" => log_book(given, &when, &year, first),
        _ => self_education(given, &when, &year, first),
    })
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

/// The working from home methods open in the year, each with its total, and
/// the actual cost beside them.
fn home_office(given: &[(String, Arg)], when: &str, year: &str, first: i64) -> Result<Value> {
    let early = first < 2022;
    let mut names = HOME_OFFICE.to_vec();
    if early {
        names.push("dedicated_home_office");
    }
    let get = named(given, COMMAND, when, &names)?;
    let [
        hours,
        energy,
        phone,
        internet,
        stationery,
        cleaning,
        furniture,
        assets,
        repairs,
    ] = HOME_OFFICE.map(|n| money(get(n)));
    let (hours, energy, phone, internet, stationery) =
        (hours?, energy?, phone?, internet?, stationery?);
    let (cleaning, furniture, assets, repairs) = (cleaning?, furniture?, assets?, repairs?);
    let days = if (first + 1) % 4 == 0 { 366 } else { 365 };
    if hours.gt(&int(24 * days))? {
        return conflict(format!(
            "hours cannot exceed the {} hours in {year}.",
            24 * days
        ));
    }
    let dedicated = if early {
        flag(get("dedicated_home_office"), "dedicated_home_office")?
    } else {
        true
    };
    let hourly = d(fixed_rate(first));
    let mut steps: Vec<Step> = Vec::new();
    let mut methods = Vec::new();
    if dedicated {
        let amount = hours.mul(&hourly)?;
        steps.push(step(
            "fixed_rate_amount",
            if early { FIXED_RATE_52 } else { FIXED_RATE_PCG },
            "hours x fixed_rate",
            &[&hours, &hourly],
            amount.clone(),
        ));
        let separate = if early {
            let value = sum(&[&phone, &internet, &stationery, &assets, &repairs])?;
            step(
                "fixed_rate_separate_expenses",
                SEPARATE_52,
                "phone + internet + stationery + asset_decline + repairs",
                &[&phone, &internet, &stationery, &assets, &repairs],
                value,
            )
        } else {
            let value = sum(&[&cleaning, &furniture, &assets, &repairs])?;
            step(
                "fixed_rate_separate_expenses",
                SEPARATE_PCG,
                "cleaning + furniture_decline + asset_decline + repairs",
                &[&cleaning, &furniture, &assets, &repairs],
                value,
            )
        };
        let total = amount.add(&separate.value)?;
        let method_step = step(
            "fixed_rate_method",
            FIXED_TOTAL,
            "fixed_rate_amount + fixed_rate_separate_expenses",
            &[&amount, &separate.value],
            total.clone(),
        );
        steps.extend([separate, method_step]);
        steps.push(step(
            "fixed_rate_return_amount",
            WHOLE_DOLLARS,
            "floor(fixed_rate_method)",
            &[&total],
            floor(&total)?,
        ));
        methods.push(("fixed_rate", total));
    }
    let shortcut_rate = d("0.80");
    if early {
        let total = hours.mul(&shortcut_rate)?;
        steps.push(step(
            "shortcut_method",
            SHORTCUT,
            "hours x shortcut_rate",
            &[&hours, &shortcut_rate],
            total.clone(),
        ));
        steps.push(step(
            "shortcut_return_amount",
            WHOLE_DOLLARS,
            "floor(shortcut_method)",
            &[&total],
            floor(&total)?,
        ));
        methods.push(("shortcut", total));
    }
    let all = [
        &energy,
        &phone,
        &internet,
        &stationery,
        &cleaning,
        &furniture,
        &assets,
        &repairs,
    ];
    let actual = sum(&all)?;
    steps.push(step(
        "actual_cost_method",
        ACTUAL,
        "energy + phone + internet + stationery + cleaning + furniture_decline + asset_decline + repairs",
        &all,
        actual.clone(),
    ));
    steps.push(step(
        "actual_cost_return_amount",
        WHOLE_DOLLARS,
        "floor(actual_cost_method)",
        &[&actual],
        floor(&actual)?,
    ));
    methods.push(("actual_cost", actual));
    // The first listed wins a tie, so a fixed rate equal to the actual cost
    // is named: it needs fewer records.
    let mut higher = &methods[0];
    for m in &methods[1..] {
        if m.1.gt(&higher.1)? {
            higher = m;
        }
    }
    let higher = higher.0;
    let mut rates = vec![rate("fixed_rate", &hourly)];
    if early {
        rates.push(rate("shortcut_rate", &shortcut_rate));
    }
    let mut out = derived_result(KIND, year, steps, rates)?;
    out["method"] = json!("home_office");
    out["methods_available"] = json!(methods.iter().map(|m| m.0).collect::<Vec<_>>());
    out["fixed_rate_unavailable"] = if dedicated {
        Value::Null
    } else {
        json!(
            "Before 2022-23 the 52c fixed rate needs a dedicated home office; the shortcut and actual cost methods remain."
        )
    };
    out["higher_method"] = json!(higher);
    Ok(out)
}

fn cents_per_kilometre(
    given: &[(String, Arg)],
    when: &str,
    year: &str,
    first: i64,
) -> Result<Value> {
    let get = named(
        given,
        COMMAND,
        when,
        &["owned_or_leased", "business_kilometres"],
    )?;
    car_held(get("owned_or_leased"))?;
    let kilometres = money(get("business_kilometres"))?;
    let cap = d(KILOMETRE_CAP);
    let per_kilometre = d(rate_per_kilometre(first));
    let counted = py_min(kilometres.clone(), cap.clone())?;
    let deduction = counted.mul(&per_kilometre)?;
    let steps = vec![
        step(
            "claimed_kilometres",
            CAP,
            "min(business_kilometres, kilometre_cap)",
            &[&kilometres, &cap],
            counted.clone(),
        ),
        step(
            "deduction",
            CENTS_RATE,
            "claimed_kilometres x rate_per_kilometre",
            &[&counted, &per_kilometre],
            deduction.clone(),
        ),
        step(
            "return_amount",
            WHOLE_DOLLARS,
            "floor(deduction)",
            &[&deduction],
            floor(&deduction)?,
        ),
    ];
    let rates = vec![
        rate("rate_per_kilometre", &per_kilometre),
        rate("kilometre_cap", &cap),
    ];
    let mut out = derived_result(KIND, year, steps, rates)?;
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

fn log_book(given: &[(String, Arg)], when: &str, year: &str, first: i64) -> Result<Value> {
    let names = [
        "owned_or_leased",
        "business_kilometres",
        "total_kilometres",
        "car_expenses",
        "log_book_year",
        "log_book_weeks",
    ];
    let get = named(given, COMMAND, when, &names)?;
    car_held(get("owned_or_leased"))?;
    let kept = income_year(get("log_book_year"), "log_book_year")?;
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
    let weeks = match get("log_book_weeks").int() {
        Some(Some(n)) if n >= 0 => n,
        _ => return invalid("log_book_weeks must be a whole number of weeks."),
    };
    if weeks < 12 {
        return refuse(
            "work_expenses.log_book_too_short",
            "A log book must cover a continuous period of at least 12 weeks (ITAA 1997 s 28-120(1)); the shorter period for a car held under 12 weeks is outside the worksheet.",
        );
    }
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
    let steps = vec![
        step(
            "business_use_percentage",
            BUSINESS_USE,
            "business_kilometres / total_kilometres x 100",
            &[&business, &total],
            percentage.clone(),
        ),
        step(
            "deduction",
            LOG_BOOK,
            "car_expenses x business_use_percentage / 100",
            &[&expenses, &percentage],
            deduction.clone(),
        ),
        step(
            "return_amount",
            WHOLE_DOLLARS,
            "floor(deduction)",
            &[&deduction],
            floor(&deduction)?,
        ),
    ];
    let mut out = derived_result(KIND, year, steps, Vec::new())?;
    out["method"] = json!("car_log_book");
    out["log_book_valid_to"] = json!(format!("{}-{:02}", kept + 4, (kept + 5) % 100));
    Ok(out)
}

fn self_education(given: &[(String, Arg)], when: &str, year: &str, first: i64) -> Result<Value> {
    let early = first < 2022;
    let mut names = vec![
        "connection",
        "employed_when_incurred",
        "category_a",
        "category_b",
        "category_c",
        "category_d",
    ];
    if early {
        names.push("prescribed_course");
        // Category E counts only against s 82A's reduction, so it is asked
        // for only where the reduction can apply.
        if given
            .iter()
            .any(|(n, v)| n == "prescribed_course" && matches!(v, Arg::Bool(true)))
        {
            names.push("category_e");
        }
    }
    let get = named(given, COMMAND, when, &names)?;
    let connection = text(
        get("connection"),
        "connection",
        &[
            "maintains_or_improves_skills",
            "increases_income",
            "general_only",
            "new_employment",
        ],
    )?;
    let employed = flag(get("employed_when_incurred"), "employed_when_incurred")?;
    let [a, b, c, dd] =
        ["category_a", "category_b", "category_c", "category_d"].map(|n| money(get(n)));
    let (a, b, c, dd) = (a?, b?, c?, dd?);
    let prescribed = early && flag(get("prescribed_course"), "prescribed_course")?;
    let e = if prescribed {
        Some(money(get("category_e"))?)
    } else {
        None
    };
    let connected = employed
        && matches!(
            connection,
            "maintains_or_improves_skills" | "increases_income"
        );
    let mut steps = Vec::new();
    let mut rates = Vec::new();
    if !connected {
        steps.push(step("deduction", NOT_CONNECTED, "0", &[], int(0)));
    } else if let Some(e) = e {
        let threshold = d(SELF_EDUCATION_REDUCTION);
        let offset = sum(&[&c, &dd, &e])?;
        let reduction = py_max(int(0), threshold.sub(&offset)?)?;
        let reduced = py_max(int(0), a.sub(&reduction)?)?;
        let deduction = sum(&[&reduced, &b, &c, &dd])?;
        steps.extend([
            step(
                "reduction_offset",
                OFFSET,
                "category_c + category_d + category_e",
                &[&c, &dd, &e],
                offset.clone(),
            ),
            step(
                "reduction",
                REDUCTION,
                "max(0, self_education_reduction - reduction_offset)",
                &[&threshold, &offset],
                reduction.clone(),
            ),
            step(
                "category_a_after_reduction",
                REDUCED_A,
                "max(0, category_a - reduction)",
                &[&a, &reduction],
                reduced.clone(),
            ),
            step(
                "deduction",
                SELF_EDUCATION,
                "category_a_after_reduction + category_b + category_c + category_d",
                &[&reduced, &b, &c, &dd],
                deduction,
            ),
        ]);
        rates.push(rate("self_education_reduction", &threshold));
    } else {
        let deduction = sum(&[&a, &b, &c, &dd])?;
        steps.push(step(
            "deduction",
            SELF_EDUCATION,
            "category_a + category_b + category_c + category_d",
            &[&a, &b, &c, &dd],
            deduction,
        ));
    }
    let deduction = steps.last().expect("a deduction step").value.clone();
    steps.push(step(
        "return_amount",
        WHOLE_DOLLARS,
        "floor(deduction)",
        &[&deduction],
        floor(&deduction)?,
    ));
    let mut out = derived_result(KIND, year, steps, rates)?;
    out["method"] = json!("self_education");
    out["deductible"] = json!(connected);
    out["reduction_applies"] = json!(connected && prescribed);
    Ok(out)
}
