// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Each contract's schedule and the figures the reports add from it.
use super::input::{Row, column};
use super::{
    COMMON, FIGURES, INTEGERS, Options, calc, cents, day, income_year, money, month, signal, sum,
    zero,
};
use crate::api::{Refusal, evaluate_coded};
use crate::dates;
use crate::decimal::Decimal;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

pub(super) struct Period {
    from: i64,
    to: i64,
    interest: Decimal,
}

/// A contract as its schedule sets it out: each payment as (day, amount),
/// and each period with its interest.
pub(super) struct Contract<'a> {
    row: &'a Row<'a>,
    start: i64,
    financed: Decimal,
    payments: Vec<(i64, Decimal)>,
    periods: Vec<Period>,
}

impl Contract<'_> {
    /// The amount outstanding at the end of `day`, on or after the start.
    fn outstanding(&self, day: i64) -> Result<Decimal, Refusal> {
        let ended = self.periods.iter().filter(|p| p.to - 1 <= day);
        let interest = sum(ended.map(|p| &p.interest))?;
        let paid = sum(self.payments.iter().filter(|p| p.0 <= day).map(|p| &p.1))?;
        calc(self.financed.add(&interest).and_then(|v| v.sub(&paid)))
    }

    fn final_day(&self) -> i64 {
        self.payments.iter().map(|p| p.0).max().expect("a payment")
    }
}

/// The facts the row's kind's command takes, from its cells.
fn facts(row: &Row, start: i64) -> Result<Value, Refusal> {
    let mut input = Map::new();
    for &name in row.kind.required.iter().chain(row.kind.optional) {
        let cell = row.cell(name);
        let value = match cell {
            "" => continue,
            _ if INTEGERS.contains(&name) => json!(
                cell.parse::<u32>()
                    .map_err(|_| column(row.line, format!("{name} must be a whole number")))?
            ),
            _ => json!(cell),
        };
        input.insert(name.into(), value);
    }
    input.insert("start_date".into(), json!(row.cell("start_date")));
    input.insert("year".into(), json!(income_year(start).0));
    input.insert("scope_confirmed".into(), json!(true));
    Ok(Value::Object(input))
}

/// The row's contract through its kind's command, and that command's result.
pub(super) fn schedule<'a>(row: &'a Row<'a>) -> Result<(Contract<'a>, Value), Refusal> {
    let start = dates::parse(row.cell("start_date")).ok_or_else(|| {
        column(
            row.line,
            "start_date must be a YYYY-MM-DD date from 1900 to 2100".into(),
        )
    })?;
    let result = evaluate_coded(row.kind.command, &facts(row, start)?).map_err(|r| {
        let at = format!("line {} (contract {})", row.line, row.cell("contract"));
        Refusal::new(r.code, format!("{at}: {}", r.message))
    })?;
    let mut contract = Contract {
        row,
        start,
        financed: money(&result["amounts"]["amount_financed"])?,
        payments: Vec::new(),
        periods: Vec::new(),
    };
    for period in result["schedule"].as_array().expect("a schedule") {
        let (instalment, balloon) = (money(&period["payment"])?, money(&period["balloon"])?);
        if !instalment.is_zero() {
            contract
                .payments
                .push((day(&period["payment_date"]), instalment));
        }
        let to = day(&period["period_end"]);
        if !balloon.is_zero() {
            contract.payments.push((to, balloon));
        }
        let (from, interest) = (day(&period["period_start"]), money(&period["interest"])?);
        contract.periods.push(Period { from, to, interest });
    }
    Ok((contract, result))
}

/// The contract's figures at the balance date, each null before its start.
pub(super) fn at_balance(
    contract: &Contract,
    options: &Options,
) -> Result<Map<String, Value>, Refusal> {
    let mut out: Map<String, Value> = FIGURES[1..]
        .iter()
        .chain(&[
            "accruing_interest",
            "accruing_days_elapsed",
            "accruing_days",
        ])
        .map(|key| (key.to_string(), Value::Null))
        .collect();
    let balance = options.balance;
    if contract.start > balance {
        return Ok(out);
    }
    let outstanding = contract.outstanding(balance)?;
    let fall = calc(outstanding.sub(&contract.outstanding(options.current_to)?))?;
    let current = if fall.lt(&zero()).map_err(signal)? {
        zero()
    } else {
        fall
    };
    let non_current = calc(outstanding.sub(&current))?;
    let remaining = sum(contract
        .payments
        .iter()
        .filter(|p| p.0 > balance)
        .map(|p| &p.1))?;
    let unexpired = sum(contract
        .periods
        .iter()
        .filter(|p| p.to - 1 > balance)
        .map(|p| &p.interest))?;
    let figures = [remaining, unexpired, outstanding, current, non_current];
    for (key, value) in FIGURES[1..].iter().zip(&figures) {
        out.insert(key.to_string(), cents(value));
    }
    accruing(contract, balance, &mut out);
    Ok(out)
}

/// The interest of the period running at the end of `balance`, its days and
/// how many of them have passed.
fn accruing(contract: &Contract, balance: i64, out: &mut Map<String, Value>) {
    let running = contract
        .periods
        .iter()
        .find(|p| p.from <= balance && balance < p.to - 1);
    if let Some(period) = running {
        out.insert("accruing_interest".into(), cents(&period.interest));
        out.insert(
            "accruing_days_elapsed".into(),
            json!(balance - period.from + 1),
        );
        out.insert("accruing_days".into(), json!(period.to - period.from));
    }
}

/// One income year's payments, interest and the interest of the period
/// spanning its 1 July, with that period's days before and from it.
pub(super) struct Year {
    payments: Decimal,
    interest: Decimal,
    across: Decimal,
    days: Option<(i64, i64)>,
}

pub(super) fn year_of(years: &mut BTreeMap<String, Year>, key: String) -> &mut Year {
    years.entry(key).or_insert_with(|| Year {
        payments: zero(),
        interest: zero(),
        across: zero(),
        days: None,
    })
}

pub(super) fn add_to(total: &mut Decimal, amount: &Decimal) -> Result<(), Refusal> {
    *total = calc(total.add(amount))?;
    Ok(())
}

pub(super) fn by_month(contract: &Contract) -> Result<BTreeMap<String, Decimal>, Refusal> {
    let mut months = BTreeMap::new();
    for (day, amount) in &contract.payments {
        add_to(months.entry(month(*day)).or_insert_with(zero), amount)?;
    }
    Ok(months)
}

pub(super) fn by_income_year(contract: &Contract) -> Result<BTreeMap<String, Year>, Refusal> {
    let mut years = BTreeMap::new();
    for (day, amount) in &contract.payments {
        add_to(
            &mut year_of(&mut years, income_year(*day).0).payments,
            amount,
        )?;
    }
    for period in &contract.periods {
        let (key, opens) = income_year(period.to - 1);
        let year = year_of(&mut years, key);
        add_to(&mut year.interest, &period.interest)?;
        // Periods run on from each other, so one at most spans a 1 July.
        if period.from < opens {
            year.across = period.interest.clone();
            year.days = Some((opens - period.from, period.to - opens));
        }
    }
    Ok(years)
}

pub(super) fn merge_years(
    into: &mut BTreeMap<String, Year>,
    from: &BTreeMap<String, Year>,
) -> Result<(), Refusal> {
    for (key, year) in from {
        let total = year_of(into, key.clone());
        add_to(&mut total.payments, &year.payments)?;
        add_to(&mut total.interest, &year.interest)?;
        add_to(&mut total.across, &year.across)?;
    }
    Ok(())
}

pub(super) fn month_rows(months: &BTreeMap<String, Decimal>) -> Vec<Value> {
    months
        .iter()
        .map(|(month, payments)| json!({"month": month, "payments": cents(payments)}))
        .collect()
}

pub(super) fn year_rows(years: &BTreeMap<String, Year>) -> Result<Vec<Value>, Refusal> {
    years
        .iter()
        .map(|(key, year)| {
            let principal = calc(year.payments.sub(&year.interest))?;
            Ok(json!({
                "income_year": key,
                "payments": cents(&year.payments),
                "interest": cents(&year.interest),
                "interest_across_30_june": cents(&year.across),
                "days_before_1_july": year.days.map(|d| d.0),
                "days_from_1_july": year.days.map(|d| d.1),
                "principal": cents(&principal),
            }))
        })
        .collect()
}

pub(super) fn contract_json(
    contract: &Contract,
    result: &Value,
    options: &Options,
    months: &BTreeMap<String, Decimal>,
    years: &BTreeMap<String, Year>,
) -> Result<Value, Refusal> {
    let row = contract.row;
    let mut out = Map::new();
    out.insert("line".into(), json!(row.line));
    for name in COMMON {
        out.insert(name.into(), json!(row.cell(name)));
    }
    out.insert(
        "final_payment_date".into(),
        json!(dates::format(contract.final_day())),
    );
    for key in ["amount_financed", "total_payments", "total_interest"] {
        out.insert(key.into(), result["amounts"][key].clone());
    }
    out.insert(
        "nominal_annual_rate".into(),
        result["rates"]["nominal_annual_rate"].clone(),
    );
    out.extend(at_balance(contract, options)?);
    out.insert("by_month".into(), json!(month_rows(months)));
    out.insert("by_income_year".into(), json!(year_rows(years)?));
    Ok(Value::Object(out))
}

/// The number of `contracts` and the sum of each of their figures, a null
/// figure adding nil.
pub(super) fn totals<'a>(
    contracts: impl Iterator<Item = &'a Value>,
) -> Result<Map<String, Value>, Refusal> {
    let (mut count, mut sums) = (0, FIGURES.map(|_| zero()));
    for contract in contracts {
        count += 1;
        for (key, total) in FIGURES.iter().zip(&mut sums) {
            if !contract[key].is_null() {
                add_to(total, &money(&contract[key])?)?;
            }
        }
    }
    let mut out = Map::new();
    out.insert("contracts".into(), json!(count));
    for (key, total) in FIGURES.iter().zip(&sums) {
        out.insert(key.to_string(), cents(total));
    }
    Ok(out)
}

/// The totals for each value of `key`, in its order.
pub(super) fn groups(contracts: &[Value], key: &str) -> Result<Vec<Value>, Refusal> {
    let mut names: Vec<&str> = contracts.iter().filter_map(|c| c[key].as_str()).collect();
    names.sort_unstable();
    names.dedup();
    names
        .into_iter()
        .map(|name| {
            let mut out = Map::new();
            out.insert(key.into(), json!(name));
            out.extend(totals(contracts.iter().filter(|c| c[key] == name))?);
            Ok(Value::Object(out))
        })
        .collect()
}

/// The contracts whose final payment falls after the balance date and on or
/// before `expiring_by`, by that date and then contract.
pub(super) fn expiring(contracts: &[Value], options: &Options) -> Vec<Value> {
    let (after, by) = (
        dates::format(options.balance),
        dates::format(options.expiring_by),
    );
    let mut out: Vec<Value> = contracts
        .iter()
        .filter(|c| {
            let last = c["final_payment_date"]
                .as_str()
                .expect("a final payment date");
            last > after.as_str() && last <= by.as_str()
        })
        .map(|c| {
            let keys = [
                "contract",
                "client",
                "financier",
                "kind",
                "final_payment_date",
            ];
            let mut out: Map<String, Value> =
                keys.iter().map(|k| (k.to_string(), c[k].clone())).collect();
            out.insert("remaining_payments".into(), c["remaining_payments"].clone());
            out.insert("outstanding".into(), c["outstanding"].clone());
            Value::Object(out)
        })
        .collect();
    fn order(c: &Value) -> [Option<&str>; 2] {
        [c["final_payment_date"].as_str(), c["contract"].as_str()]
    }
    out.sort_by(|a, b| order(a).cmp(&order(b)));
    out
}

/// Every contract set out: each one's JSON, each kind's command with its
/// scope and sources, and the register's payments by month and income year.
pub(super) struct SetOut {
    pub(super) contracts: Vec<Value>,
    pub(super) kinds: Map<String, Value>,
    pub(super) months: BTreeMap<String, Decimal>,
    pub(super) years: BTreeMap<String, Year>,
}

pub(super) fn set_out(rows: &[Row], options: &Options) -> Result<SetOut, Refusal> {
    let (mut contracts, mut kinds) = (Vec::new(), Map::new());
    let (mut months, mut years) = (BTreeMap::new(), BTreeMap::new());
    for row in rows {
        let (contract, result) = schedule(row)?;
        kinds.entry(row.kind.name).or_insert_with(|| {
            json!({
                "command": row.kind.command,
                "scope": result["scope"],
                "sources": result["sources"],
                "source_checked": result["source_checked"],
            })
        });
        let own_months = by_month(&contract)?;
        for (month, amount) in &own_months {
            add_to(months.entry(month.clone()).or_insert_with(zero), amount)?;
        }
        let own_years = by_income_year(&contract)?;
        merge_years(&mut years, &own_years)?;
        contracts.push(contract_json(
            &contract,
            &result,
            options,
            &own_months,
            &own_years,
        )?);
    }
    Ok(SetOut {
        contracts,
        kinds,
        months,
        years,
    })
}
