// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Working from home: the fixed rate (52c with a dedicated home office to
//! 2021-22, then PCG 2023/1 Table 1) and, for 2020-21 and 2021-22, the 80c
//! shortcut (PCG 2020/3), each beside the actual cost.

use serde_json::{Value, json};

use super::{
    Arg, Decimal, KIND, Result, Step, conflict, d, derived_result, flag, int, money, named, rate,
    step, sum, with_return,
};

/// The inputs, in [`Inputs`]' order.
const NAMES: [&str; 9] = [
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

const FIXED_RATE_PCG: &str = "PCG 2023/1 paras 23 to 26 and Table 1 (para 26A): hours worked from home in the income year x the fixed rate for the year, which covers energy, internet, mobile and home phone usage, and stationery and computer consumables (para 14)";
const FIXED_RATE_52: &str = "ATO 'Fixed rate method - 52 cents' (1 July 2018 to 30 June 2022, PS LA 2001/6 section 5, retired by PCG 2023/1 para 17): 52c for each hour worked from home with a dedicated home office, covering energy for heating, cooling and lighting, cleaning the home office and the decline in value of home office furniture and furnishings";
const SEPARATE_PCG: &str = "PCG 2023/1 paras 15 and 26(c): the work-related decline in value of depreciating assets, their repairs and running expenses the rate does not cover (cleaning) are claimed beside it";
const SEPARATE_52: &str = "ATO 'Fixed rate method - 52 cents': phone, internet, stationery and computer consumables, the decline in value of depreciating assets other than home office furniture and furnishings, and repairs are claimed beside the 52c rate at their work-related amounts";
const FIXED_TOTAL: &str = "PCG 2023/1 para 26(d), and the ATO's 52 cent method before 2022-23: the hourly amount plus the expenses claimed beside it";
const SHORTCUT: &str = "PCG 2020/3 and ATO 'Shortcut method' (2020-21 and 2021-22): 80c for each hour worked from home, covering every working from home expense, with nothing claimed beside it";
const NOT_INCURRED: &str = "No additional running expenses were incurred as a result of working from home, so no hourly rate applies (PCG 2023/1 paras 18(b) and 49 to 55; PCG 2020/3); the actual cost remains.";
const NOT_DEDICATED: &str = "Before 2022-23 the 52c fixed rate needs a dedicated home office; the shortcut and actual cost methods remain.";
const ACTUAL: &str = "ATO 'Actual cost method' and ITAA 1997 s 8-1: the work-related portion of each additional running expense and of each depreciating asset's decline in value, as apportioned";

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

/// The hours and each expense's work-related amount, in [`NAMES`]' order.
struct Inputs {
    hours: Decimal,
    energy: Decimal,
    phone: Decimal,
    internet: Decimal,
    stationery: Decimal,
    cleaning: Decimal,
    furniture: Decimal,
    assets: Decimal,
    repairs: Decimal,
}

/// With no additional running expense incurred, refuses an amount for one:
/// from 2022-23 the kinds the fixed rate covers (PCG 2023/1 para 14), and
/// before then any running expense, cleaning included.
fn no_running_expense(x: &Inputs, early: bool) -> Result<()> {
    let mut running = vec![&x.energy, &x.phone, &x.internet, &x.stationery];
    if early {
        running.push(&x.cleaning);
    }
    if running.iter().all(|v| v.is_zero()) {
        return Ok(());
    }
    let kinds = if early {
        "energy, phone, internet, stationery or cleaning"
    } else {
        "energy, phone, internet or stationery"
    };
    conflict(format!(
        "running_expenses_incurred is false, yet {kinds} has an amount, which is a running expense incurred."
    ))
}

/// Refuses more hours than the income year holds: 366 days when its
/// February has 29.
fn within_year(hours: &Decimal, year: &str, first: i64) -> Result<()> {
    let days = if (first + 1) % 4 == 0 { 366 } else { 365 };
    if hours.gt(&int(24 * days))? {
        return conflict(format!(
            "hours cannot exceed the {} hours in {year}.",
            24 * days
        ));
    }
    Ok(())
}

/// The inputs, whether additional running expenses were incurred, which
/// every hourly rate needs, and whether the 52c rate's dedicated home office
/// was there (always true from 2022-23, which does not ask).
fn inputs(
    given: &[(String, Arg)],
    when: &str,
    year: &str,
    first: i64,
) -> Result<(Inputs, bool, bool)> {
    let early = first < 2022;
    let mut names = NAMES.to_vec();
    names.push("running_expenses_incurred");
    if early {
        names.push("dedicated_home_office");
    }
    let get = named(given, super::COMMAND, when, &names)?;
    let mut values = Vec::with_capacity(NAMES.len());
    for name in NAMES {
        values.push(money(get(name))?);
    }
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
    ] = <[Decimal; 9]>::try_from(values).expect("9 inputs");
    within_year(&hours, year, first)?;
    let incurred = flag(
        get("running_expenses_incurred"),
        "running_expenses_incurred",
    )?;
    let dedicated = !early || flag(get("dedicated_home_office"), "dedicated_home_office")?;
    let x = Inputs {
        hours,
        energy,
        phone,
        internet,
        stationery,
        cleaning,
        furniture,
        assets,
        repairs,
    };
    if !incurred {
        no_running_expense(&x, early)?;
    }
    Ok((x, incurred, dedicated))
}

/// The fixed rate method's steps (the hourly amount, the expenses claimed
/// beside it, their total and its return amount) and its total.
fn fixed(x: &Inputs, early: bool, hourly: &Decimal) -> Result<(Vec<Step>, Decimal)> {
    let amount = x.hours.mul(hourly)?;
    let hours_step = step(
        "fixed_rate_amount",
        if early { FIXED_RATE_52 } else { FIXED_RATE_PCG },
        "hours x fixed_rate",
        &[&x.hours, hourly],
        amount.clone(),
    );
    let (authority, formula, beside) = if early {
        let values = vec![&x.phone, &x.internet, &x.stationery, &x.assets, &x.repairs];
        let names = "phone + internet + stationery + asset_decline + repairs";
        (SEPARATE_52, names, values)
    } else {
        let values = vec![&x.cleaning, &x.furniture, &x.assets, &x.repairs];
        let names = "cleaning + furniture_decline + asset_decline + repairs";
        (SEPARATE_PCG, names, values)
    };
    let separate = sum(&beside)?;
    let separate_step = step(
        "fixed_rate_separate_expenses",
        authority,
        formula,
        &beside,
        separate.clone(),
    );
    let value = amount.add(&separate)?;
    let total = step(
        "fixed_rate_method",
        FIXED_TOTAL,
        "fixed_rate_amount + fixed_rate_separate_expenses",
        &[&amount, &separate],
        value.clone(),
    );
    let mut steps = vec![hours_step, separate_step];
    steps.extend(with_return(total, "fixed_rate_return_amount")?);
    Ok((steps, value))
}

/// The 80c shortcut's steps for 2020-21 and 2021-22 and its total.
fn shortcut(x: &Inputs, rate: &Decimal) -> Result<([Step; 2], Decimal)> {
    let value = x.hours.mul(rate)?;
    let total = step(
        "shortcut_method",
        SHORTCUT,
        "hours x shortcut_rate",
        &[&x.hours, rate],
        value.clone(),
    );
    Ok((with_return(total, "shortcut_return_amount")?, value))
}

/// The actual cost method's steps: every expense's work-related amount.
fn actual(x: &Inputs) -> Result<[Step; 2]> {
    let all = [
        &x.energy,
        &x.phone,
        &x.internet,
        &x.stationery,
        &x.cleaning,
        &x.furniture,
        &x.assets,
        &x.repairs,
    ];
    let total = step(
        "actual_cost_method",
        ACTUAL,
        "energy + phone + internet + stationery + cleaning + furniture_decline + asset_decline + repairs",
        &all,
        sum(&all)?,
    );
    with_return(total, "actual_cost_return_amount")
}

/// The method with the higher total; the first listed wins a tie, so a fixed
/// rate equal to the actual cost is named, as it needs fewer records.
fn higher(methods: &[(&'static str, Decimal)]) -> Result<&'static str> {
    let mut best = &methods[0];
    for m in &methods[1..] {
        if m.1.gt(&best.1)? {
            best = m;
        }
    }
    Ok(best.0)
}

pub(super) fn home_office(
    given: &[(String, Arg)],
    when: &str,
    year: &str,
    first: i64,
) -> Result<Value> {
    let early = first < 2022;
    let (x, incurred, dedicated) = inputs(given, when, year, first)?;
    let hourly = d(fixed_rate(first));
    let mut steps = Vec::new();
    let mut methods = Vec::new();
    let mut rates = vec![rate("fixed_rate", &hourly)];
    if incurred && dedicated {
        let (fixed_steps, total) = fixed(&x, early, &hourly)?;
        steps.extend(fixed_steps);
        methods.push(("fixed_rate", total));
    }
    if incurred && early {
        let shortcut_rate = d("0.80");
        let (shortcut_steps, total) = shortcut(&x, &shortcut_rate)?;
        steps.extend(shortcut_steps);
        methods.push(("shortcut", total));
        rates.push(rate("shortcut_rate", &shortcut_rate));
    }
    let [total, back] = actual(&x)?;
    methods.push(("actual_cost", total.value.clone()));
    steps.extend([total, back]);
    let higher = higher(&methods)?;
    let mut out = derived_result(KIND, year, steps, rates)?;
    out["method"] = json!("home_office");
    out["methods_available"] = json!(methods.iter().map(|m| m.0).collect::<Vec<_>>());
    out["fixed_rate_unavailable"] = match (incurred, dedicated) {
        (false, _) => json!(NOT_INCURRED),
        (true, false) => json!(NOT_DEDICATED),
        (true, true) => Value::Null,
    };
    out["higher_method"] = json!(higher);
    Ok(out)
}
