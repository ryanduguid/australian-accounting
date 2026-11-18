// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `contract-register`: a practice's finance contracts, one line of a CSV
//! file each, set out together. Each contract's schedule comes from its
//! kind's schedule command through [`evaluate_coded`], unchanged; this module
//! only dates the payments and interest that schedule gives and adds them up:
//! cashflow by month, payments, interest and principal by income year, the
//! amount outstanding at a balance date split into current and non-current,
//! the contracts expiring by a date, and totals by financier and by client.
//! The split cites AASB 101 as compiled on 31 December 2022 (compilation 6,
//! the latest), read on 19 November 2026.
use crate::api::{Refusal, evaluate_coded};
use crate::dates;
use crate::decimal::{Decimal, Signal};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;

mod csv;
mod render;

/// A contract kind the register sets out, the schedule command that does it
/// and the columns that command requires and may take besides `start_date`.
/// A command plugs in here when it takes `start_date`, `year` (the income
/// year the start falls in) and `scope_confirmed`, and its result gives
/// `amounts.amount_financed`, `amounts.total_payments`,
/// `amounts.total_interest`, `rates.nominal_annual_rate` and a `schedule`
/// whose rows give `period_start`, `period_end` (the day after the period's
/// last), `payment_date`, `payment` (paid on `payment_date`), `balloon` (paid
/// on `period_end`) and `interest`, as `hire-purchase-schedule` does.
struct Kind {
    name: &'static str,
    command: &'static str,
    required: &'static [&'static str],
    optional: &'static [&'static str],
}

impl Kind {
    fn takes(&self, column: &str) -> bool {
        self.required.contains(&column) || self.optional.contains(&column)
    }
}

const KINDS: [Kind; 1] = [Kind {
    name: "hire_purchase",
    command: "hire-purchase-schedule",
    required: &[
        "amount_financed",
        "instalment",
        "instalments",
        "balloon",
        "timing",
    ],
    optional: &["frequency"],
}];

/// The columns every contract fills.
const COMMON: [&str; 5] = ["contract", "client", "financier", "kind", "start_date"];
/// The columns that name a contract, its client and its financier.
const NAMES: [&str; 3] = ["contract", "client", "financier"];
const NAME_RULE: &str = "must be 1 to 200 characters with no control character, and must not begin with =, +, - or @, which a spreadsheet reads as a formula";
/// The columns a schedule command takes as a JSON integer.
const INTEGERS: [&str; 1] = ["instalments"];
const MOST_CONTRACTS: usize = 5_000;
const MOST_BYTES: u64 = 8 << 20;

/// The figures each contract gives at the balance date, which the totals add.
const FIGURES: [&str; 6] = [
    "amount_financed",
    "remaining_payments",
    "unexpired_interest",
    "outstanding",
    "current",
    "non_current",
];

const SCOPE: &str = "A practice's finance contracts, one a line of the contracts file, each set out by its kind's schedule command on that line's facts, with year the income year its start_date falls in; kinds names each command with its scope and sources, and confirming scope confirms that scope for every contract. Takes hire_purchase contracts with level instalments, through hire-purchase-schedule, starting in a year that command supports. The register dates and adds the schedules' payments and interest and changes none of them. It takes no deposit, trade-in, payout, GST, fees or irregular payments, and does not assess AASB 101 paragraph 69(a), (b) or (d), such as a breached covenant that makes a liability payable on demand (paragraphs 72B to 75).";

const BASIS: [&str; 6] = [
    "A payment counts on the day it is made: an instalment on its row's payment_date, a balloon on its row's period_end. A period runs from period_start up to period_end, so its interest counts on the day before period_end, when the period ends.",
    "outstanding at a date is the amount financed plus the interest of the periods ended by then, less the payments made by then, which is remaining_payments less unexpired_interest. It leaves out the interest of the period running at the balance date: accruing_interest is that period's interest, over accruing_days, of which accruing_days_elapsed fall on or before the balance date.",
    "AASB 101 paragraph 69(c): a liability is current when it is due to be settled within twelve months after the reporting period. current is the part of outstanding that the payments due by current_to, 12 months after the balance date, settle once they have paid the interest of the periods ending by then: outstanding less the amount outstanding at current_to, or nil where that is below nil. non_current is the rest.",
    "By income year, interest counts in the year its period ends. interest_across_30_june is the part of it from a period spanning 30 June, which runs days_before_1_july before 1 July and days_from_1_july from it: ITAA 1997 s 240-60(2) relates such a period's interest to each income year in accordance with accounting principles, which the caller applies. principal is the payments less the interest, the fall in the amount outstanding over the year, below nil where interest builds up before the payments start.",
    "A contract that starts after the balance date has no balance-date figures and adds nil to them; its payments still count by month and by income year.",
    "expiring lists the contracts whose final payment falls after the balance date and on or before expiring_by, 12 months after the balance date unless given.",
];

const SOURCES: [&str; 1] =
    ["https://www.legislation.gov.au/F2015L01626/2022-12-31/2022-12-31/text"];

enum Format {
    Json,
    Text,
    Csv,
}

struct Options {
    balance: i64,
    current_to: i64,
    expiring_by: i64,
    format: Format,
}

/// The `contract-register` command: the contracts file on `input`, the
/// reports as (exit code, stdout, stderr). A refusal writes one `error:` line
/// and exits 2; with `--refusal-json`, stdout also carries its code.
pub fn command(args: &[String], input: &mut dyn Read) -> (i32, String, String) {
    let refusal_json = args.iter().any(|a| a == "--refusal-json");
    let args: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--refusal-json")
        .collect();
    match run(&args, input) {
        Ok(out) => (0, out, String::new()),
        Err(Refusal { code, message }) => {
            let out = if refusal_json {
                let refusal = json!({"ok": false, "code": code, "message": message});
                format!("{refusal}\n")
            } else {
                String::new()
            };
            (2, out, format!("error: {message}\n"))
        }
    }
}

fn run(args: &[&str], input: &mut dyn Read) -> Result<String, Refusal> {
    let options = options(args)?;
    let mut bytes = Vec::new();
    input
        .take(MOST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| Refusal::new("input.unreadable", e.to_string()))?;
    if bytes.len() as u64 > MOST_BYTES {
        return Err(Refusal::new(
            "input.too_large",
            "the contracts file exceeds 8 MiB",
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        Refusal::new(
            "contract_register.csv_invalid",
            "the contracts file must be UTF-8 text",
        )
    })?;
    let register = register(text, &options)?;
    Ok(match options.format {
        Format::Json => format!("{register}\n"),
        Format::Text => render::text(&register),
        Format::Csv => render::csv(&register),
    })
}

fn argument(message: String) -> Refusal {
    Refusal::new("contract_register.argument_invalid", message)
}

fn options(args: &[&str]) -> Result<Options, Refusal> {
    let (mut given, mut scope) = (BTreeMap::new(), false);
    let mut args = args.iter();
    while let Some(&flag) = args.next() {
        match flag {
            "--scope-confirmed" if !scope => scope = true,
            "--balance-date" | "--expiring-by" | "--format" if !given.contains_key(flag) => {
                let value = args
                    .next()
                    .ok_or_else(|| argument(format!("{flag} needs a value")))?;
                given.insert(flag, *value);
            }
            _ => {
                return Err(argument(format!(
                    "{flag} is not an argument of contract-register, or is given twice"
                )));
            }
        }
    }
    if !scope {
        return Err(Refusal::new(
            "scope.not_confirmed",
            "Establish all stated scope conditions before calculating.",
        ));
    }
    let date = |flag: &str| {
        given
            .get(flag)
            .map(|text| {
                dates::parse(text).ok_or_else(|| {
                    argument(format!(
                        "{flag} must be a YYYY-MM-DD date from 1900 to 2100"
                    ))
                })
            })
            .transpose()
    };
    let balance =
        date("--balance-date")?.ok_or_else(|| argument("--balance-date is required".into()))?;
    let current_to = dates::add_months(balance, 12);
    let expiring_by = date("--expiring-by")?.unwrap_or(current_to);
    if expiring_by < balance {
        return Err(argument(
            "--expiring-by must not be before --balance-date".into(),
        ));
    }
    let format = match given.get("--format").copied() {
        None | Some("json") => Format::Json,
        Some("text") => Format::Text,
        Some("csv") => Format::Csv,
        Some(other) => {
            return Err(argument(format!(
                "--format must be json, text or csv, not {other:?}"
            )));
        }
    };
    Ok(Options {
        balance,
        current_to,
        expiring_by,
        format,
    })
}

/// One contract's line: its number, its cells by column and its kind.
struct Row<'a> {
    line: usize,
    cells: BTreeMap<&'a str, &'a str>,
    kind: &'static Kind,
}

impl<'a> Row<'a> {
    fn cell(&self, column: &str) -> &'a str {
        self.cells.get(column).copied().unwrap_or("")
    }
}

fn column(line: usize, message: String) -> Refusal {
    Refusal::new(
        "contract_register.column_invalid",
        format!("line {line}: {message}"),
    )
}

fn header(line: usize, columns: &[String]) -> Result<(), Refusal> {
    let known = |c: &str| COMMON.contains(&c) || KINDS.iter().any(|k| k.takes(c));
    for (i, name) in columns.iter().enumerate() {
        if !known(name) {
            return Err(column(
                line,
                format!("{name:?} is not a column of the register"),
            ));
        }
        if columns[..i].contains(name) {
            return Err(column(line, format!("{name} is given twice")));
        }
    }
    match COMMON
        .iter()
        .find(|c| !columns.iter().any(|name| name == *c))
    {
        Some(missing) => Err(column(line, format!("the header lacks {missing}"))),
        None => Ok(()),
    }
}

fn valid_name(text: &str) -> bool {
    (1..=200).contains(&text.chars().count())
        && !text.chars().any(char::is_control)
        && !text.starts_with(['=', '+', '-', '@'])
}

fn row<'a>(line: usize, columns: &'a [String], fields: &'a [String]) -> Result<Row<'a>, Refusal> {
    if fields.len() != columns.len() {
        return Err(Refusal::new(
            "contract_register.csv_invalid",
            format!(
                "line {line} has {} fields where the header has {}",
                fields.len(),
                columns.len()
            ),
        ));
    }
    let cells: BTreeMap<&str, &str> = columns
        .iter()
        .map(String::as_str)
        .zip(fields.iter().map(String::as_str))
        .collect();
    if let Some(name) = NAMES.iter().find(|name| !valid_name(cells[*name])) {
        return Err(Refusal::new(
            "contract_register.text_invalid",
            format!("line {line}: {name} {NAME_RULE}"),
        ));
    }
    let Some(kind) = KINDS.iter().find(|k| k.name == cells["kind"]) else {
        let taken: Vec<&str> = KINDS.iter().map(|k| k.name).collect();
        return Err(Refusal::new(
            "contract_register.kind_unsupported",
            format!(
                "line {line}: kind {:?} is not one the register sets out; it takes {}",
                cells["kind"],
                taken.join(", ")
            ),
        ));
    };
    let stray = cells
        .iter()
        .find(|&(c, cell)| !cell.is_empty() && !COMMON.contains(c) && !kind.takes(c));
    if let Some((name, _)) = stray {
        let why = format!(
            "a {} contract does not take {name}; leave it empty",
            kind.name
        );
        return Err(column(line, why));
    }
    let mut required = kind.required.iter().chain(&["start_date"]);
    if let Some(missing) = required.find(|c| cells.get(**c).is_none_or(|v| v.is_empty())) {
        let why = format!("a {} contract needs {missing}", kind.name);
        return Err(column(line, why));
    }
    Ok(Row { line, cells, kind })
}

/// The header's and each contract's line, refusing the first that is wrong.
fn rows(records: &[(usize, Vec<String>)]) -> Result<Vec<Row<'_>>, Refusal> {
    let Some(((at, columns), body)) = records.split_first() else {
        return Err(Refusal::new(
            "contract_register.csv_invalid",
            "the contracts file is empty: it needs a header line, then one contract a line",
        ));
    };
    header(*at, columns)?;
    if body.is_empty() || body.len() > MOST_CONTRACTS {
        return Err(Refusal::new(
            "contract_register.csv_invalid",
            format!(
                "the register must hold 1 to {MOST_CONTRACTS} contracts, one a line after the header; it holds {}",
                body.len()
            ),
        ));
    }
    let mut seen: HashMap<&str, usize> = HashMap::new();
    body.iter()
        .map(|(line, fields)| {
            let row = row(*line, columns, fields)?;
            if let Some(first) = seen.insert(row.cell("contract"), *line) {
                return Err(Refusal::new(
                    "contract_register.contract_duplicate",
                    format!(
                        "line {line}: contract {:?} is also on line {first}",
                        row.cell("contract")
                    ),
                ));
            }
            Ok(row)
        })
        .collect()
}

fn signal(signal: Signal) -> Refusal {
    Refusal::new(signal.code(), format!("decimal.{}", signal.name()))
}

fn calc(result: Result<Decimal, Signal>) -> Result<Decimal, Refusal> {
    result.map_err(signal)
}

fn zero() -> Decimal {
    Decimal::from_int(0)
}

fn sum<'a>(values: impl IntoIterator<Item = &'a Decimal>) -> Result<Decimal, Refusal> {
    values
        .into_iter()
        .try_fold(zero(), |total, v| calc(total.add(v)))
}

fn cents(value: &Decimal) -> Value {
    json!(value.fixed(2, false))
}

/// An amount a schedule renders.
fn money(value: &Value) -> Result<Decimal, Refusal> {
    calc(Decimal::parse(value.as_str().expect("a rendered amount")))
}

/// A date a schedule renders, as its day number.
fn day(value: &Value) -> i64 {
    value
        .as_str()
        .and_then(dates::parse)
        .expect("a rendered date")
}

/// The income year `day` falls in, as `2026-27`, and the 1 July it opens on.
fn income_year(day: i64) -> (String, i64) {
    let (y, m, _) = dates::to_civil(day);
    let first = if m >= 7 { y } else { y - 1 };
    let opens = dates::parse(&format!("{first}-07-01")).expect("1 July");
    (format!("{first}-{:02}", (first + 1) % 100), opens)
}

fn month(day: i64) -> String {
    dates::format(day)[..7].to_string()
}

struct Period {
    from: i64,
    to: i64,
    interest: Decimal,
}

/// A contract as its schedule sets it out: each payment as (day, amount),
/// and each period with its interest.
struct Contract<'a> {
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

/// The row's contract through its kind's command, and that command's result.
fn schedule<'a>(row: &'a Row<'a>) -> Result<(Contract<'a>, Value), Refusal> {
    let start = dates::parse(row.cell("start_date")).ok_or_else(|| {
        column(
            row.line,
            "start_date must be a YYYY-MM-DD date from 1900 to 2100".into(),
        )
    })?;
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
    let result = evaluate_coded(row.kind.command, &Value::Object(input)).map_err(|r| {
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
fn at_balance(contract: &Contract, options: &Options) -> Result<Map<String, Value>, Refusal> {
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
    Ok(out)
}

/// One income year's payments, interest and the interest of the period
/// spanning its 1 July, with that period's days before and from it.
struct Year {
    payments: Decimal,
    interest: Decimal,
    across: Decimal,
    days: Option<(i64, i64)>,
}

fn year_of(years: &mut BTreeMap<String, Year>, key: String) -> &mut Year {
    years.entry(key).or_insert_with(|| Year {
        payments: zero(),
        interest: zero(),
        across: zero(),
        days: None,
    })
}

fn add_to(total: &mut Decimal, amount: &Decimal) -> Result<(), Refusal> {
    *total = calc(total.add(amount))?;
    Ok(())
}

fn by_month(contract: &Contract) -> Result<BTreeMap<String, Decimal>, Refusal> {
    let mut months = BTreeMap::new();
    for (day, amount) in &contract.payments {
        add_to(months.entry(month(*day)).or_insert_with(zero), amount)?;
    }
    Ok(months)
}

fn by_income_year(contract: &Contract) -> Result<BTreeMap<String, Year>, Refusal> {
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

fn merge_years(
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

fn month_rows(months: &BTreeMap<String, Decimal>) -> Vec<Value> {
    months
        .iter()
        .map(|(month, payments)| json!({"month": month, "payments": cents(payments)}))
        .collect()
}

fn year_rows(years: &BTreeMap<String, Year>) -> Result<Vec<Value>, Refusal> {
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

fn contract_json(
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
fn totals<'a>(contracts: impl Iterator<Item = &'a Value>) -> Result<Map<String, Value>, Refusal> {
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
fn groups(contracts: &[Value], key: &str) -> Result<Vec<Value>, Refusal> {
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
fn expiring(contracts: &[Value], options: &Options) -> Vec<Value> {
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

fn register(text: &str, options: &Options) -> Result<Value, Refusal> {
    let records = csv::records(text)?;
    let rows = rows(&records)?;
    let (mut contracts, mut kinds) = (Vec::new(), Map::new());
    let (mut months, mut years) = (BTreeMap::new(), BTreeMap::new());
    for row in &rows {
        let (contract, result) = schedule(row)?;
        kinds.entry(row.kind.name).or_insert_with(|| {
            json!({
                "command": row.kind.command,
                "scope": result["scope"],
                "sources": result["sources"],
                "source_checked": result["source_checked"],
            })
        });
        let contract_months = by_month(&contract)?;
        for (month, amount) in &contract_months {
            add_to(months.entry(month.clone()).or_insert_with(zero), amount)?;
        }
        let contract_years = by_income_year(&contract)?;
        merge_years(&mut years, &contract_years)?;
        let json = contract_json(
            &contract,
            &result,
            options,
            &contract_months,
            &contract_years,
        )?;
        contracts.push(json);
    }
    Ok(json!({
        "ok": true,
        "engine": "australian-tax-calculators",
        "engine_version": crate::engine::VERSION,
        "calculation": "contract_register",
        "balance_date": dates::format(options.balance),
        "current_to": dates::format(options.current_to),
        "expiring_by": dates::format(options.expiring_by),
        "totals": totals(contracts.iter())?,
        "expiring": expiring(&contracts, options),
        "by_financier": groups(&contracts, "financier")?,
        "by_client": groups(&contracts, "client")?,
        "cashflow_by_month": month_rows(&months),
        "by_income_year": year_rows(&years)?,
        "contracts": contracts,
        "basis": BASIS,
        "kinds": kinds,
        "scope": SCOPE,
        "sources": SOURCES,
        "source_checked": "2026-11-19",
        "warnings": [
            "Operator-confirmed facts and scope; eligibility is not independently verified.",
            "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
        ],
    }))
}
