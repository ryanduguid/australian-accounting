// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `contract-register` over the fabricated register in
//! examples/contract-register.csv: every total is the sum of each contract's
//! own `hire-purchase-schedule`, run here on facts written out by hand, the
//! balance-date split is worked by hand for 2 contracts, and a malformed line
//! is refused with its number.
use australian_tax_calculators::api;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const EXAMPLE: &str = include_str!("../examples/contract-register.csv");
const HEADER: &str = "contract,client,financier,kind,start_date,amount_financed,instalment,instalments,frequency,timing,balloon";

/// (contract, start_date, amount_financed, instalment, instalments,
/// frequency, timing, balloon)
type Facts = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    i64,
    &'static str,
    &'static str,
    &'static str,
);

/// The example's contracts as written there.
const FACTS: [Facts; 8] = [
    (
        "HP-1001",
        "2024-09-15",
        "48000.00",
        "850.00",
        60,
        "monthly",
        "in_arrears",
        "8000.00",
    ),
    (
        "HP-1002",
        "2025-07-01",
        "30000.00",
        "160.00",
        208,
        "weekly",
        "in_advance",
        "0.00",
    ),
    (
        "HP-1003",
        "2025-11-30",
        "75000.00",
        "4800.00",
        20,
        "quarterly",
        "in_arrears",
        "0.00",
    ),
    (
        "HP-1004",
        "2026-03-10",
        "22000.00",
        "380.00",
        65,
        "fortnightly",
        "in_arrears",
        "2000.00",
    ),
    (
        "HP-1005",
        "2026-06-01",
        "120000.00",
        "2300.00",
        48,
        "monthly",
        "in_advance",
        "30000.00",
    ),
    (
        "HP-1006",
        "2026-08-20",
        "38000.00",
        "9000.00",
        5,
        "annual",
        "in_arrears",
        "0.00",
    ),
    (
        "HP-1007",
        "2026-03-01",
        "10000.00",
        "100.00",
        12,
        "monthly",
        "in_arrears",
        "10000.00",
    ),
    (
        "HP-1008",
        "2025-08-01",
        "24000.00",
        "1000.00",
        24,
        "monthly",
        "in_arrears",
        "0.00",
    ),
];

const BALANCE: &str = "2026-06-30";

fn run(args: &[&str], csv: &str) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    api::contract_register(&args, &mut csv.as_bytes())
}

fn register(csv: &str) -> Value {
    let (code, out, err) = run(&["--balance-date", BALANCE, "--scope-confirmed"], csv);
    assert_eq!(code, 0, "{err}");
    serde_json::from_str(&out).unwrap()
}

/// Cents, worked apart from the crate's decimals.
fn cents(value: &Value) -> i64 {
    let text = value
        .as_str()
        .unwrap_or_else(|| panic!("{value} is not an amount"));
    let (whole, part) = text.split_once('.').unwrap();
    let sign = if whole.starts_with('-') { -1 } else { 1 };
    sign * (whole.trim_start_matches('-').parse::<i64>().unwrap() * 100
        + part.parse::<i64>().unwrap())
}

fn total<'a>(rows: impl IntoIterator<Item = &'a Value>, key: &str) -> i64 {
    rows.into_iter().map(|row| cents(&row[key])).sum()
}

fn list<'a>(value: &'a Value, key: &str) -> &'a Vec<Value> {
    value[key].as_array().unwrap()
}

/// Each example contract's own schedule from `hire-purchase-schedule`.
fn schedules() -> BTreeMap<&'static str, Value> {
    FACTS
        .iter()
        .map(|&(contract, start, financed, instalment, n, frequency, timing, balloon)| {
            let first: i64 = start[..4].parse().unwrap();
            let first = if &start[5..7] >= "07" { first } else { first - 1 };
            let input = json!({
                "amount_financed": financed, "instalment": instalment, "instalments": n,
                "frequency": frequency, "timing": timing, "balloon": balloon, "start_date": start,
                "year": format!("{first}-{:02}", (first + 1) % 100), "scope_confirmed": true,
            });
            (contract, api::evaluate("hire-purchase-schedule", &input).unwrap())
        })
        .collect()
}

fn contract<'a>(register: &'a Value, id: &str) -> &'a Value {
    list(register, "contracts")
        .iter()
        .find(|c| c["contract"] == id)
        .unwrap()
}

/// Each schedule row's payments as (date, cents): an instalment on its
/// payment_date, a balloon on its period_end.
fn payments(schedule: &Value) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    for row in list(schedule, "schedule") {
        if cents(&row["payment"]) != 0 {
            out.push((
                row["payment_date"].as_str().unwrap().to_string(),
                cents(&row["payment"]),
            ));
        }
        if cents(&row["balloon"]) != 0 {
            out.push((
                row["period_end"].as_str().unwrap().to_string(),
                cents(&row["balloon"]),
            ));
        }
    }
    out
}

/// The day before `date`, which ends a period whose period_end is `date`.
fn day_before(date: &str) -> String {
    let (y, m, d): (i64, i64, i64) = (
        date[..4].parse().unwrap(),
        date[5..7].parse().unwrap(),
        date[8..].parse().unwrap(),
    );
    if d > 1 {
        return format!("{y:04}-{m:02}-{:02}", d - 1);
    }
    let (y, m) = if m == 1 { (y - 1, 12) } else { (y, m - 1) };
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let last = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ][(m - 1) as usize];
    format!("{y:04}-{m:02}-{last:02}")
}

#[test]
fn every_total_is_the_sum_of_each_contracts_own_schedule() {
    let register = register(EXAMPLE);
    let schedules = schedules();
    assert_eq!(list(&register, "contracts").len(), FACTS.len());
    let (mut paid, mut interest, mut financed) = (0, 0, 0);
    for (id, schedule) in &schedules {
        let c = contract(&register, id);
        for key in ["amount_financed", "total_payments", "total_interest"] {
            assert_eq!(c[key], schedule["amounts"][key], "{id} {key}");
        }
        let own = payments(schedule);
        let scheduled: i64 = own.iter().map(|p| p.1).sum();
        assert_eq!(scheduled, cents(&c["total_payments"]), "{id}");
        assert_eq!(total(list(c, "by_month"), "payments"), scheduled, "{id}");
        let years = list(c, "by_income_year");
        assert_eq!(total(years, "payments"), scheduled, "{id}");
        assert_eq!(
            total(years, "interest"),
            total(list(schedule, "schedule"), "interest"),
            "{id}"
        );
        assert_eq!(
            total(years, "principal"),
            cents(&c["amount_financed"]),
            "{id}"
        );
        assert_eq!(
            c["final_payment_date"].as_str().unwrap(),
            own.iter().map(|p| p.0.as_str()).max().unwrap()
        );
        // Each month's payments are that month's payments in the schedule.
        for month in list(c, "by_month") {
            let key = month["month"].as_str().unwrap();
            let own: i64 = own
                .iter()
                .filter(|p| p.0.starts_with(key))
                .map(|p| p.1)
                .sum();
            assert_eq!(cents(&month["payments"]), own, "{id} {key}");
        }
        paid += scheduled;
        interest += cents(&schedule["amounts"]["total_interest"]);
        financed += cents(&schedule["amounts"]["amount_financed"]);
    }
    assert_eq!(
        total(list(&register, "cashflow_by_month"), "payments"),
        paid
    );
    let years = list(&register, "by_income_year");
    assert_eq!(total(years, "payments"), paid);
    assert_eq!(total(years, "interest"), interest);
    assert_eq!(total(years, "principal"), financed);
    for year in years {
        let key = &year["income_year"];
        for field in [
            "payments",
            "interest",
            "interest_across_30_june",
            "principal",
        ] {
            let own: i64 = list(&register, "contracts")
                .iter()
                .flat_map(|c| list(c, "by_income_year"))
                .filter(|row| row["income_year"] == *key)
                .map(|row| cents(&row[field]))
                .sum();
            assert_eq!(cents(&year[field]), own, "{key} {field}");
        }
    }
    assert_eq!(cents(&register["totals"]["amount_financed"]), financed);
    for key in [
        "remaining_payments",
        "unexpired_interest",
        "outstanding",
        "current",
        "non_current",
    ] {
        let started = list(&register, "contracts")
            .iter()
            .filter(|c| !c[key].is_null());
        assert_eq!(
            cents(&register["totals"][key]),
            total(started, key),
            "{key}"
        );
    }
}

/// The amount outstanding at the balance date, from each schedule's own
/// closing balances: the closing balance of the last period ended by then,
/// plus what it lists paid after the balance date, less what later periods
/// list paid by then.
#[test]
fn the_amount_outstanding_follows_each_schedules_closing_balance() {
    let register = register(EXAMPLE);
    for (id, schedule) in schedules() {
        let c = contract(&register, id);
        let rows = list(&schedule, "schedule");
        let ended = rows
            .iter()
            .filter(|row| day_before(row["period_end"].as_str().unwrap()).as_str() <= BALANCE)
            .count();
        if rows[0]["period_start"].as_str().unwrap() > BALANCE {
            assert!(c["outstanding"].is_null(), "{id}");
            continue;
        }
        let mut expected = if ended == 0 {
            cents(&schedule["amounts"]["amount_financed"])
        } else {
            cents(&rows[ended - 1]["closing"])
        };
        for (i, row) in rows.iter().enumerate() {
            for (day, amount) in payments(&json!({"schedule": [row]})) {
                if i < ended && day.as_str() > BALANCE {
                    expected += amount;
                } else if i >= ended && day.as_str() <= BALANCE {
                    expected -= amount;
                }
            }
        }
        assert_eq!(cents(&c["outstanding"]), expected, "{id}");
        let left = cents(&c["remaining_payments"]) - cents(&c["unexpired_interest"]);
        assert_eq!(left, expected, "{id}");
        assert_eq!(
            cents(&c["current"]) + cents(&c["non_current"]),
            expected,
            "{id}"
        );
    }
}

#[test]
fn the_balance_date_split_is_worked_by_hand() {
    let register = register(EXAMPLE);
    // HP-1007: $10,000 at exactly 1% a month, $100 a month in arrears from
    // 1 March 2026 and the $10,000 back on 1 March 2027. By 30 June the
    // periods ending 31 March, 30 April, 31 May and 30 June have charged $400
    // and the instalments of 1 April, 1 May and 1 June have paid $300, so
    // $10,100 is outstanding; the balloon and the last 9 instalments fall due
    // by 30 June 2027 and leave nil, so all of it is current.
    let c = contract(&register, "HP-1007");
    assert_eq!(c["total_interest"], "1200.00");
    assert_eq!(c["outstanding"], "10100.00");
    assert_eq!(c["remaining_payments"], "10900.00");
    assert_eq!(c["unexpired_interest"], "800.00");
    assert_eq!(c["current"], "10100.00");
    assert_eq!(c["non_current"], "0.00");
    assert!(c["accruing_interest"].is_null());
    // HP-1008: $24,000 at nil, $1,000 a month in arrears from 1 September 2025
    // to 1 August 2027. 10 are paid by 30 June 2026 and 12 fall due in the
    // year to 30 June 2027, leaving the 2 of July and August 2027.
    let c = contract(&register, "HP-1008");
    assert_eq!(c["outstanding"], "14000.00");
    assert_eq!(c["current"], "12000.00");
    assert_eq!(c["non_current"], "2000.00");
    assert_eq!(c["unexpired_interest"], "0.00");
    // HP-1001's period from 15 June to 15 July 2026 runs at the balance date:
    // 16 of its 30 days, 15 to 30 June, fall on or before it.
    let c = contract(&register, "HP-1001");
    let schedule = &schedules()["HP-1001"];
    let running = list(schedule, "schedule")
        .iter()
        .find(|row| row["period_start"] == "2026-06-15")
        .unwrap();
    assert_eq!(c["accruing_interest"], running["interest"]);
    assert_eq!(
        (
            c["accruing_days_elapsed"].as_i64(),
            c["accruing_days"].as_i64()
        ),
        (Some(16), Some(30))
    );
    // HP-1006 starts after the balance date: no figures, and nil in the totals.
    let c = contract(&register, "HP-1006");
    for key in [
        "outstanding",
        "current",
        "non_current",
        "remaining_payments",
    ] {
        assert!(c[key].is_null(), "{key}");
    }
    assert_eq!(register["current_to"], "2027-06-30");
}

/// A period across 30 June: its interest counts in the year it ends, set
/// apart with its days either side of 1 July as the schedule gives them.
#[test]
fn interest_across_30_june_is_set_apart_as_the_schedule_sets_it_apart() {
    let register = register(EXAMPLE);
    for (id, schedule) in schedules() {
        let c = contract(&register, id);
        let years = list(c, "by_income_year");
        for across in list(&schedule, "interest_across_30_june") {
            let later = &across["income_years"][1];
            let row = years
                .iter()
                .find(|row| row["income_year"] == *later)
                .unwrap();
            assert_eq!(row["interest_across_30_june"], across["interest"], "{id}");
            assert_eq!(row["days_before_1_july"], across["days"][0], "{id}");
            assert_eq!(row["days_from_1_july"], across["days"][1], "{id}");
        }
        for row in years {
            let year = &row["income_year"];
            let wholly: i64 = list(&schedule, "interest_by_income_year")
                .iter()
                .filter(|y| y["income_year"] == *year)
                .map(|y| cents(&y["interest"]))
                .sum();
            assert_eq!(
                cents(&row["interest"]),
                wholly + cents(&row["interest_across_30_june"]),
                "{id} {year}"
            );
        }
    }
    let c = contract(&register, "HP-1001");
    let row = list(c, "by_income_year")
        .iter()
        .find(|r| r["income_year"] == "2025-26")
        .unwrap();
    assert_eq!(
        (
            row["days_before_1_july"].as_i64(),
            row["days_from_1_july"].as_i64()
        ),
        (Some(16), Some(14))
    );
}

#[test]
fn expiring_financier_and_client_reports_add_the_contracts() {
    let register = register(EXAMPLE);
    // Final payments after 30 June 2026 and by 30 June 2027: HP-1007 on 1 March 2027.
    let ids = |r: &Value| -> Vec<String> {
        list(r, "expiring")
            .iter()
            .map(|e| e["contract"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(ids(&register), ["HP-1007"]);
    let (code, out, err) = run(
        &[
            "--balance-date",
            BALANCE,
            "--expiring-by",
            "2027-08-31",
            "--scope-confirmed",
        ],
        EXAMPLE,
    );
    assert_eq!(code, 0, "{err}");
    let later: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(ids(&later), ["HP-1007", "HP-1008"]);
    for (report, key) in [("by_financier", "financier"), ("by_client", "client")] {
        let groups = list(&register, report);
        let names: Vec<&str> = groups.iter().map(|g| g[key].as_str().unwrap()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        assert_eq!(
            groups
                .iter()
                .map(|g| g["contracts"].as_i64().unwrap())
                .sum::<i64>(),
            8
        );
        for field in ["amount_financed", "outstanding", "current", "non_current"] {
            assert_eq!(
                total(groups, field),
                cents(&register["totals"][field]),
                "{report} {field}"
            );
        }
    }
    let quokka = list(&register, "by_financier")
        .iter()
        .find(|g| g["financier"] == "Quokka Credit")
        .unwrap();
    assert_eq!(quokka["contracts"], 3);
}

/// Payments below the interest build the balance up, so the next 12 months
/// settle none of it: all of it is non-current.
#[test]
fn a_balance_the_year_ahead_builds_up_is_all_non_current() {
    let csv = format!(
        "{HEADER}\nHP-1,Client,Financier,hire_purchase,2026-03-01,10000.00,10.00,24,monthly,in_arrears,13000.00\n"
    );
    let register = register(&csv);
    let c = &register["contracts"][0];
    assert_eq!(c["current"], "0.00");
    assert_eq!(c["non_current"], c["outstanding"]);
    assert!(cents(&c["outstanding"]) > 1_000_000);
    let year = list(c, "by_income_year")
        .iter()
        .find(|y| y["income_year"] == "2026-27")
        .unwrap();
    assert!(cents(&year["principal"]) < 0);
}

#[test]
fn text_csv_and_json_carry_the_same_figures_every_time() {
    let args = |format: &'static str| {
        [
            "--balance-date",
            BALANCE,
            "--format",
            format,
            "--scope-confirmed",
        ]
    };
    let register = register(EXAMPLE);
    let (code, csv, err) = run(&args("csv"), EXAMPLE);
    assert_eq!(code, 0, "{err}");
    assert_eq!(run(&args("csv"), EXAMPLE).1, csv);
    let mut lines = csv.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    assert_eq!(header[0], "report");
    let rows: Vec<&str> = lines.collect();
    // The client holding a comma is quoted, so a naive split finds one field more.
    let quoted = rows
        .iter()
        .find(|l| l.contains("\"Coolabah Plumbing, Gas & Drains\""))
        .unwrap();
    assert_eq!(quoted.split(',').count(), header.len() + 1);
    let total = rows.iter().find(|l| l.starts_with("total,")).unwrap();
    let cell = |line: &str, key: &str| {
        line.split(',')
            .nth(header.iter().position(|h| *h == key).unwrap())
            .unwrap()
            .to_string()
    };
    for key in [
        "contracts",
        "amount_financed",
        "outstanding",
        "current",
        "non_current",
    ] {
        assert_eq!(
            cell(total, key),
            register["totals"][key].to_string().trim_matches('"'),
            "{key}"
        );
    }
    let count = |report: &str| {
        rows.iter()
            .filter(|l| l.starts_with(&format!("{report},")))
            .count()
    };
    assert_eq!(count("contract"), 8);
    assert_eq!(count("month"), list(&register, "cashflow_by_month").len());
    assert_eq!(
        count("income_year"),
        list(&register, "by_income_year").len()
    );
    let (code, text, err) = run(&args("text"), EXAMPLE);
    assert_eq!(code, 0, "{err}");
    assert!(text.starts_with(
        "Contract register at 2026-06-30: current to 2027-06-30, expiring by 2027-06-30\n"
    ));
    for heading in [
        "Contracts at the balance date",
        "Expiring",
        "By financier",
        "By client",
        "Cashflow by month",
        "By income year",
        "Interest accruing at the balance date",
        "Interest across 30 June",
        "Basis",
    ] {
        assert!(text.contains(&format!("\n{heading}\n")), "{heading}");
    }
    let outstanding = register["totals"]["outstanding"].as_str().unwrap();
    assert!(
        text.lines()
            .any(|l| l.starts_with("Total") && l.contains(outstanding))
    );
    assert_eq!(run(&args("json"), EXAMPLE).1, run(&args("json"), EXAMPLE).1);
}

/// (stdin, code, message start) for a register refused at one line.
fn refused(args: &[&str], csv: &str) -> (String, String) {
    let mut args = args.to_vec();
    args.push("--refusal-json");
    let (code, out, err) = run(&args, csv);
    assert_eq!(code, 2, "{out}");
    let refusal: Value = serde_json::from_str(&out).unwrap();
    let message = refusal["message"].as_str().unwrap().to_string();
    assert_eq!(err, format!("error: {message}\n"));
    (refusal["code"].as_str().unwrap().to_string(), message)
}

#[test]
fn a_malformed_line_is_refused_with_its_number() {
    let args = ["--balance-date", BALANCE, "--scope-confirmed"];
    let good = "HP-1,Client,Financier,hire_purchase,2026-03-01,10000.00,100.00,12,monthly,in_arrears,10000.00";
    let with = |line: &str| format!("{HEADER}\n{good}\n{line}\n");
    let cases = [
        (
            with("HP-2,\"Client,Financier,hire_purchase,2026-03-01,1.00,1.00,1,,in_arrears,0.00"),
            "contract_register.csv_invalid",
            "line 3: a quoted field is not closed",
        ),
        (
            with("HP-2,Cli\"ent,Financier,hire_purchase,2026-03-01,1.00,1.00,1,,in_arrears,0.00"),
            "contract_register.csv_invalid",
            "line 3: a quote may appear",
        ),
        (
            with("HP-2,Client,Financier,hire_purchase,2026-03-01"),
            "contract_register.csv_invalid",
            "line 3 has 5 fields where the header has 11",
        ),
        (
            format!("{HEADER}\n{good}\n\n{}\n", good.replace("HP-1,", "HP-2,")),
            "contract_register.csv_invalid",
            "line 3: an empty line may only end the file",
        ),
        (
            with(good),
            "contract_register.contract_duplicate",
            "line 3: contract \"HP-1\" is also on line 2",
        ),
        (
            with(
                &good
                    .replace("HP-1,", "HP-2,")
                    .replace("hire_purchase", "lease"),
            ),
            "contract_register.kind_unsupported",
            "line 3: kind \"lease\" is not one the register sets out; it takes hire_purchase, chattel_mortgage",
        ),
        (
            with(&good.replace("HP-1,Client", "HP-2,=HYPERLINK(1)")),
            "contract_register.text_invalid",
            "line 3: client must be",
        ),
        (
            with(
                good.replace("HP-1,", "HP-2,")
                    .strip_suffix("10000.00")
                    .unwrap(),
            ),
            "contract_register.column_invalid",
            "line 3: a hire_purchase contract needs balloon",
        ),
        (
            with(&good.replace("HP-1,", "HP-2,").replace(",12,", ",twelve,")),
            "contract_register.column_invalid",
            "line 3: instalments must be a whole number",
        ),
        (
            with(
                &good
                    .replace("HP-1,", "HP-2,")
                    .replace("2026-03-01", "2026-02-30"),
            ),
            "contract_register.column_invalid",
            "line 3: start_date must be",
        ),
        (
            with(
                &good
                    .replace("HP-1,", "HP-2,")
                    .replace("2026-03-01", "2023-05-01"),
            ),
            "period.unsupported",
            "line 3 (contract HP-2): Unsupported period '2022-23'",
        ),
        (
            with(
                &good
                    .replace("HP-1,", "HP-2,")
                    .replace("in_arrears", "monthly"),
            ),
            "input.timing_invalid",
            "line 3 (contract HP-2): timing must be",
        ),
        (
            with(&good.replace("HP-1,", "HP-2,").replace(
                "100.00,12,monthly,in_arrears,10000.00",
                "10.00,12,monthly,in_arrears,0.00",
            )),
            "hire_purchase.rate_unsupported",
            "line 3 (contract HP-2): ",
        ),
        (
            format!("{HEADER},rate\n{good},\n"),
            "contract_register.column_invalid",
            "line 1: \"rate\" is not a column",
        ),
        (
            format!("{},contract\n{good},x\n", HEADER),
            "contract_register.column_invalid",
            "line 1: contract is given twice",
        ),
        (
            format!("{}\n", HEADER.replace("financier,", "")),
            "contract_register.column_invalid",
            "line 1: the header lacks financier",
        ),
        (
            format!("{HEADER}\n"),
            "contract_register.csv_invalid",
            "the register must hold 1 to 5000 contracts",
        ),
        (
            String::new(),
            "contract_register.csv_invalid",
            "the contracts file is empty",
        ),
    ];
    for (csv, code, start) in cases {
        let (got, message) = refused(&args, &csv);
        assert_eq!(got, code, "{message}");
        assert!(message.starts_with(start), "{message}");
    }
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    let (code, _, err) = api::contract_register(&args, &mut &[0xff_u8][..]);
    assert_eq!(
        (code, err.as_str()),
        (2, "error: the contracts file must be UTF-8 text\n")
    );
}

#[test]
fn the_arguments_are_refused_unless_complete_and_valid() {
    let csv = EXAMPLE;
    for (args, code, start) in [
        (
            &["--balance-date", BALANCE][..],
            "scope.not_confirmed",
            "Establish",
        ),
        (
            &["--scope-confirmed"][..],
            "contract_register.argument_invalid",
            "--balance-date is required",
        ),
        (
            &["--balance-date", "30/06/2026", "--scope-confirmed"][..],
            "contract_register.argument_invalid",
            "--balance-date must be a YYYY-MM-DD date",
        ),
        (
            &[
                "--balance-date",
                BALANCE,
                "--scope-confirmed",
                "--format",
                "xlsx",
            ][..],
            "contract_register.argument_invalid",
            "--format must be json, text or csv",
        ),
        (
            &[
                "--balance-date",
                BALANCE,
                "--scope-confirmed",
                "--expiring-by",
                "2026-06-29",
            ][..],
            "contract_register.argument_invalid",
            "--expiring-by must not be before",
        ),
        (
            &[
                "--balance-date",
                BALANCE,
                "--balance-date",
                BALANCE,
                "--scope-confirmed",
            ][..],
            "contract_register.argument_invalid",
            "--balance-date is not an argument",
        ),
        (
            &["--balance-date", BALANCE, "--scope-confirmed", "--year"][..],
            "contract_register.argument_invalid",
            "--year is not an argument",
        ),
        (
            &["--scope-confirmed", "--balance-date"][..],
            "contract_register.argument_invalid",
            "--balance-date needs a value",
        ),
    ] {
        let (got, message) = refused(args, csv);
        assert_eq!(got, code, "{message}");
        assert!(message.starts_with(start), "{message}");
    }
    // Without --refusal-json, stdout stays empty.
    let (code, out, err) = run(&["--scope-confirmed"], csv);
    assert_eq!((code, out.as_str()), (2, ""));
    assert_eq!(err, "error: --balance-date is required\n");
}

/// A chattel mortgage at a stated 7.99% from 15 July 2025, 60 monthly
/// instalments in arrears and a $5,000 balloon, is set out by its own
/// `chattel-mortgage-schedule`: at 30 June 2026 the 11th instalment (15 June)
/// is paid and the 12th period (15 June to 15 July, 30 days, 16 of them by
/// the balance date) is running, so the amount outstanding is the 11th row's
/// closing balance and the current part is its fall to the 23rd row's.
#[test]
fn a_chattel_mortgage_is_set_out_by_its_own_schedule() {
    let header = format!("{HEADER},annual_rate");
    let line = "CM-1,Client,Financier,chattel_mortgage,2025-07-15,45000.00,,60,monthly,in_arrears,5000.00,0.0799";
    let register = register(&format!(
        "{header}
{line}
"
    ));
    let input = json!({
        "amount_financed": "45000.00", "annual_rate": "0.0799", "instalments": 60,
        "frequency": "monthly", "timing": "in_arrears", "balloon": "5000.00",
        "start_date": "2025-07-15", "year": "2025-26", "scope_confirmed": true,
    });
    let schedule = api::evaluate("chattel-mortgage-schedule", &input).unwrap();
    let rows = list(&schedule, "schedule");
    let c = contract(&register, "CM-1");
    for key in ["amount_financed", "total_payments", "total_interest"] {
        assert_eq!(c[key], schedule["amounts"][key], "{key}");
    }
    assert_eq!(c["nominal_annual_rate"], "0.0799000000000000");
    assert_eq!(c["final_payment_date"], "2030-07-15");
    assert_eq!(c["outstanding"], rows[10]["closing"]);
    let fall = cents(&rows[10]["closing"]) - cents(&rows[22]["closing"]);
    assert_eq!(cents(&c["current"]), fall);
    assert_eq!(c["accruing_interest"], rows[11]["interest"]);
    assert_eq!(
        (
            c["accruing_days"].clone(),
            c["accruing_days_elapsed"].clone()
        ),
        (json!(30), json!(16))
    );
    let paid: i64 = payments(&schedule).iter().map(|p| p.1).sum();
    assert_eq!(total(list(c, "by_month"), "payments"), paid);

    // The command's own refusal, after the line and contract.
    let args = ["--balance-date", BALANCE, "--scope-confirmed"];
    let both = line.replace(",,60,", ",844.16,60,");
    let (code, message) = refused(
        &args,
        &format!(
            "{header}
{both}
"
        ),
    );
    assert_eq!(code, "chattel_mortgage.terms_conflict");
    assert!(
        message.starts_with("line 2 (contract CM-1): Give instalment"),
        "{message}"
    );
    // A hire purchase line does not take a stated rate.
    let hire = "HP-1,Client,Financier,hire_purchase,2026-03-01,10000.00,100.00,12,monthly,in_arrears,10000.00,0.05";
    let (code, message) = refused(
        &args,
        &format!(
            "{header}
{hire}
"
        ),
    );
    assert_eq!(code, "contract_register.column_invalid");
    assert_eq!(
        message,
        "line 2: a hire_purchase contract does not take annual_rate; leave it empty"
    );
}
