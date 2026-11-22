// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Self-education: the connection to current employment (ITAA 1997 s 8-1,
//! TR 2024/3) and, before 2022-23, the ITAA 1936 s 82A $250 reduction.

use serde_json::{Value, json};

use super::{
    Arg, COMMAND, Decimal, KIND, Result, Step, conflict, d, derived_result, flag, int, invalid,
    money, named, py_max, rate, refuse, step, sum, with_return,
};

const SELF_EDUCATION_REDUCTION: &str = "250";
const LOG_BOOK_CAR: &str = "log_book_car_in_category_a";

const CONNECTIONS: [&str; 4] = [
    "maintains_or_improves_skills",
    "increases_income",
    "general_only",
    "new_employment",
];

const OFFSET: &str = "ITAA 1936 s 82A as the ATO's 'Self-education reduction in expenses' and TR 98/9 paras 119 to 155 apply it: category C, D and E expenses (repairs, cents per kilometre car expenses and expenses incurred but not deductible) offset the $250";
const REDUCTION: &str = "ITAA 1936 s 82A: the $250 reduction less the offset, never below nil, for a prescribed course of education before 2022-23";
const REDUCED_A: &str = "ITAA 1936 s 82A: category A expenses less the reduction, never below nil";
const SELF_EDUCATION: &str = "ITAA 1997 s 8-1, TR 2024/3 and the ATO's 'Self-education expenses': categories A to D, connected to current employment when incurred; ITAA 1936 s 82A reduces category A before 2022-23 only (Act No. 84, 2022, Schedule 3 items 26 and 33)";
const NOT_CONNECTED: &str = "ITAA 1997 s 8-1, TR 2024/3 and the ATO's 'Self-education expenses': nothing is deductible when, at the time the expense was incurred, the taxpayer was not employed, or the self-education only related in a general way to the employment or enabled new employment";

/// The inputs the method and year take. Category E counts only against
/// s 82A's reduction, so it is asked for only where the reduction can apply.
fn names(given: &[(String, Arg)], early: bool) -> Vec<&'static str> {
    let mut names = vec![
        "connection",
        "employed_when_incurred",
        "category_a",
        "category_b",
        "category_c",
        "category_d",
    ];
    if given.iter().any(|(n, _)| n == LOG_BOOK_CAR) {
        names.push(LOG_BOOK_CAR);
    }
    if early {
        names.push("prescribed_course");
        if given
            .iter()
            .any(|(n, v)| n == "prescribed_course" && matches!(v, Arg::Bool(true)))
        {
            names.push("category_e");
        }
    }
    names
}

/// Whether the expenses were connected to current employment when incurred.
fn connected(connection: &Arg, employed: &Arg) -> Result<bool> {
    let connection = match connection.text() {
        Some(t) if CONNECTIONS.contains(&t) => t,
        _ => {
            let allowed = CONNECTIONS.join(" or ");
            return invalid(format!("connection must be {allowed}."));
        }
    };
    let employed = flag(employed, "employed_when_incurred")?;
    Ok(employed
        && matches!(
            connection,
            "maintains_or_improves_skills" | "increases_income"
        ))
}

/// Category D's cents per kilometre car expenses cannot sit beside log book
/// car expenses in category A (the ATO's 'Self-education reduction in
/// expenses'), so a category D above 0 needs to know which is claimed.
fn one_car_method(given: &[(String, Arg)], category_d: &Decimal, when: &str) -> Result<()> {
    let answer = match given.iter().find(|(n, _)| n == LOG_BOOK_CAR) {
        Some((_, value)) => Some(flag(value, LOG_BOOK_CAR)?),
        None => None,
    };
    match answer {
        _ if category_d.is_zero() => Ok(()),
        None => refuse(
            "input.missing_field",
            format!("{LOG_BOOK_CAR} is required when category_d is above 0{when}"),
        ),
        Some(true) => conflict(
            "category_d, cents per kilometre car expenses, cannot be claimed beside log book car expenses in category_a: a car's expenses take one method (ATO 'Self-education reduction in expenses').",
        ),
        Some(false) => Ok(()),
    }
}

/// The s 82A steps: the offset, the reduction, category A after it and the
/// deduction.
fn reduced(category: [&Decimal; 5], threshold: &Decimal) -> Result<[Step; 4]> {
    let [a, b, c, dd, e] = category;
    let offset = sum(&[c, dd, e])?;
    let reduction = py_max(int(0), threshold.sub(&offset)?)?;
    let reduced = py_max(int(0), a.sub(&reduction)?)?;
    let deduction = sum(&[&reduced, b, c, dd])?;
    Ok([
        step(
            "reduction_offset",
            OFFSET,
            "category_c + category_d + category_e",
            &[c, dd, e],
            offset.clone(),
        ),
        step(
            "reduction",
            REDUCTION,
            "max(0, self_education_reduction - reduction_offset)",
            &[threshold, &offset],
            reduction.clone(),
        ),
        step(
            "category_a_after_reduction",
            REDUCED_A,
            "max(0, category_a - reduction)",
            &[a, &reduction],
            reduced.clone(),
        ),
        step(
            "deduction",
            SELF_EDUCATION,
            "category_a_after_reduction + category_b + category_c + category_d",
            &[&reduced, b, c, dd],
            deduction,
        ),
    ])
}

pub(super) fn self_education(
    given: &[(String, Arg)],
    when: &str,
    year: &str,
    first: i64,
) -> Result<Value> {
    let early = first < 2022;
    let names = names(given, early);
    let get = named(given, COMMAND, when, &names)?;
    let connected = connected(get("connection"), get("employed_when_incurred"))?;
    let mut amounts = Vec::with_capacity(4);
    for name in ["category_a", "category_b", "category_c", "category_d"] {
        amounts.push(money(get(name))?);
    }
    let [a, b, c, dd] = <[Decimal; 4]>::try_from(amounts).expect("4 categories");
    one_car_method(given, &dd, when)?;
    let prescribed = early && flag(get("prescribed_course"), "prescribed_course")?;
    let e = if prescribed {
        Some(money(get("category_e"))?)
    } else {
        None
    };
    let mut rates = Vec::new();
    let mut steps = Vec::new();
    let total = if !connected {
        step("deduction", NOT_CONNECTED, "0", &[], int(0))
    } else if let Some(e) = e {
        let threshold = d(SELF_EDUCATION_REDUCTION);
        let [offset, reduction, after, total] = reduced([&a, &b, &c, &dd, &e], &threshold)?;
        steps.extend([offset, reduction, after]);
        rates.push(rate("self_education_reduction", &threshold));
        total
    } else {
        step(
            "deduction",
            SELF_EDUCATION,
            "category_a + category_b + category_c + category_d",
            &[&a, &b, &c, &dd],
            sum(&[&a, &b, &c, &dd])?,
        )
    };
    steps.extend(with_return(total, "return_amount")?);
    let mut out = derived_result(KIND, year, steps, rates)?;
    out["method"] = json!("self_education");
    out["deductible"] = json!(connected);
    out["reduction_applies"] = json!(connected && prescribed);
    Ok(out)
}
