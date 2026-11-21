// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The register's JSON as text tables, or as one CSV table with a row for
//! each line of each report, named in its `report` column.
use super::csv::field;
use serde_json::Value;

/// The CSV's columns: `report`, then every key a report's rows give.
const COLUMNS: [&str; 29] = [
    "report",
    "line",
    "contract",
    "client",
    "financier",
    "kind",
    "start_date",
    "final_payment_date",
    "month",
    "income_year",
    "contracts",
    "amount_financed",
    "total_payments",
    "total_interest",
    "nominal_annual_rate",
    "payments",
    "interest",
    "interest_across_30_june",
    "days_before_1_july",
    "days_from_1_july",
    "principal",
    "remaining_payments",
    "unexpired_interest",
    "outstanding",
    "current",
    "non_current",
    "accruing_interest",
    "accruing_days_elapsed",
    "accruing_days",
];

fn list<'a>(register: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    register[key].as_array().into_iter().flatten()
}

fn csv_line(report: &str, row: &Value) -> String {
    let cells = COLUMNS[1..].iter().map(|key| match &row[*key] {
        Value::String(text) => field(text),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    });
    let line: Vec<String> = std::iter::once(report.to_string()).chain(cells).collect();
    format!("{}\n", line.join(","))
}

/// Each contract's rows by month and by income year, with its line, contract,
/// client and financier.
fn details<'a>(register: &'a Value, key: &'a str) -> impl Iterator<Item = Value> + 'a {
    list(register, "contracts").flat_map(move |contract| {
        contract[key].as_array().into_iter().flatten().map(|row| {
            let mut row = row.clone();
            for id in ["line", "contract", "client", "financier"] {
                row[id] = contract[id].clone();
            }
            row
        })
    })
}

pub(super) fn csv(register: &Value) -> String {
    let mut out = format!("{}\n", COLUMNS.join(","));
    for contract in list(register, "contracts") {
        out.push_str(&csv_line("contract", contract));
    }
    for (report, key) in [
        ("contract_month", "by_month"),
        ("contract_income_year", "by_income_year"),
    ] {
        for row in details(register, key) {
            out.push_str(&csv_line(report, &row));
        }
    }
    for (report, key) in [
        ("expiring", "expiring"),
        ("financier", "by_financier"),
        ("client", "by_client"),
        ("month", "cashflow_by_month"),
        ("income_year", "by_income_year"),
    ] {
        for row in list(register, key) {
            out.push_str(&csv_line(report, row));
        }
    }
    out.push_str(&csv_line("total", &register["totals"]));
    out
}

fn text_cell(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(n) => n.to_string(),
        _ => "-".into(),
    }
}

/// A titled table of `rows` under `columns` (key, heading): the first `left`
/// columns aligned left, the rest right.
fn table(out: &mut String, title: &str, columns: &[(&str, &str)], left: usize, rows: &[Value]) {
    out.push_str(&format!("\n{title}\n"));
    if rows.is_empty() {
        out.push_str("none\n");
        return;
    }
    let headings = columns.iter().map(|c| c.1.to_string()).collect();
    let body = rows
        .iter()
        .map(|row| columns.iter().map(|c| text_cell(&row[c.0])).collect());
    let cells: Vec<Vec<String>> = std::iter::once(headings).chain(body).collect();
    let width = |i: usize| {
        cells
            .iter()
            .map(|r| r[i].chars().count())
            .max()
            .unwrap_or(0)
    };
    let widths: Vec<usize> = (0..columns.len()).map(width).collect();
    for row in &cells {
        let padded: Vec<String> = row
            .iter()
            .zip(&widths)
            .enumerate()
            .map(|(i, (cell, w))| {
                let pad = " ".repeat(w - cell.chars().count());
                if i < left {
                    format!("{cell}{pad}")
                } else {
                    format!("{pad}{cell}")
                }
            })
            .collect();
        out.push_str(padded.join("  ").trim_end());
        out.push('\n');
    }
}

const AT_BALANCE: [(&str, &str); 5] = [
    ("remaining_payments", "Remaining payments"),
    ("unexpired_interest", "Unexpired interest"),
    ("outstanding", "Outstanding"),
    ("current", "Current"),
    ("non_current", "Non-current"),
];

fn with(first: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    first.iter().chain(&AT_BALANCE).copied().collect()
}

pub(super) fn text(register: &Value) -> String {
    let date = |key: &str| text_cell(&register[key]);
    let mut out = format!(
        "Contract register at {}: current to {}, expiring by {}\n",
        date("balance_date"),
        date("current_to"),
        date("expiring_by")
    );
    out.push_str("Current and non-current amounts follow scheduled payments. The preparer determines the applicable reporting framework and assesses the other classification criteria.\n");
    balance_tables(&mut out, register);
    period_tables(&mut out, register);
    interest_tables(&mut out, register);
    out.push_str("\nBasis\n");
    for line in list(register, "basis") {
        out.push_str(&format!("- {}\n", text_cell(line)));
    }
    out
}

fn rows(register: &Value, key: &str) -> Vec<Value> {
    list(register, key).cloned().collect()
}

/// The contracts at the balance date with their total, those expiring, and
/// the totals by financier and by client.
fn balance_tables(out: &mut String, register: &Value) {
    let mut contracts = rows(register, "contracts");
    let mut total = register["totals"].clone();
    total["contract"] = "Total".into();
    contracts.push(total);
    let names = [
        ("contract", "Contract"),
        ("client", "Client"),
        ("financier", "Financier"),
        ("final_payment_date", "Final payment"),
    ];
    let title = "Contracts at the balance date";
    table(out, title, &with(&names), 4, &contracts);
    let mut columns = names.to_vec();
    columns.extend([
        ("remaining_payments", "Remaining payments"),
        ("outstanding", "Outstanding"),
    ]);
    table(out, "Expiring", &columns, 4, &rows(register, "expiring"));
    for (title, key, heading) in [
        ("By financier", "financier", "Financier"),
        ("By client", "client", "Client"),
    ] {
        let columns = with(&[
            (key, heading),
            ("contracts", "Contracts"),
            ("amount_financed", "Amount financed"),
        ]);
        let groups = rows(register, &format!("by_{key}"));
        table(out, title, &columns, 1, &groups);
    }
}

/// Cashflow by month, and the payments, interest and principal by income year.
fn period_tables(out: &mut String, register: &Value) {
    let month = [("month", "Month"), ("payments", "Payments")];
    let months = rows(register, "cashflow_by_month");
    table(out, "Cashflow by month", &month, 1, &months);
    let year = [
        ("income_year", "Income year"),
        ("payments", "Payments"),
        ("interest", "Interest"),
        ("interest_across_30_june", "Across 30 June"),
        ("principal", "Principal"),
    ];
    let years = rows(register, "by_income_year");
    table(out, "By income year", &year, 1, &years);
}

/// The interest of the period running at the balance date and of each period
/// across 30 June, for the preparer to accrue or apportion.
fn interest_tables(out: &mut String, register: &Value) {
    let accruing: Vec<Value> = list(register, "contracts")
        .filter(|c| !c["accruing_interest"].is_null())
        .cloned()
        .collect();
    let columns = [
        ("contract", "Contract"),
        ("accruing_interest", "Interest"),
        ("accruing_days_elapsed", "Days elapsed"),
        ("accruing_days", "Days"),
    ];
    let title = "Interest accruing at the balance date";
    table(out, title, &columns, 1, &accruing);
    let across: Vec<Value> = details(register, "by_income_year")
        .filter(|row| !row["days_before_1_july"].is_null())
        .collect();
    let columns = [
        ("contract", "Contract"),
        ("income_year", "Income year"),
        ("interest_across_30_june", "Interest"),
        ("days_before_1_july", "Days before 1 July"),
        ("days_from_1_july", "Days from 1 July"),
    ];
    table(out, "Interest across 30 June", &columns, 2, &across);
}
