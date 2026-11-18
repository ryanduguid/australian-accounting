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
use crate::api::Refusal;
use crate::dates;
use crate::decimal::{Decimal, Signal};
use figures::{SetOut, expiring, groups, month_rows, totals, year_rows};
use serde_json::{Value, json};
use std::io::Read;

mod csv;
mod figures;
mod input;
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
    let options = input::options(args)?;
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

fn register(text: &str, options: &Options) -> Result<Value, Refusal> {
    let records = csv::records(text)?;
    let rows = input::rows(&records)?;
    let SetOut {
        contracts,
        kinds,
        months,
        years,
    } = figures::set_out(&rows, options)?;
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
