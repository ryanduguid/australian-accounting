// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The transfer balance account (ITAA 1997 Div 294) and excess contributions
//! (TAA 1953 Sch 1 former Div 95 and ss 97-25 and 97-30) on the ATO's
//! examples and on values from an independent exact-fraction oracle over the
//! ATO's published rates, read on 22 November 2026
//! (docs/calculation-evidence.md#transfer-balance).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn events(list: &[(&str, &str, &str)]) -> Value {
    Value::Array(
        list.iter()
            .map(|(date, kind, amount)| json!({"date": date, "kind": kind, "amount": amount}))
            .collect(),
    )
}

fn tba(list: &[(&str, &str, &str)], to: &str, year: &str, liable: bool) -> Value {
    json!({
        "events": events(list),
        "to": to,
        "previously_liable": liable,
        "year": year,
        "scope_confirmed": true,
    })
}

fn run(list: &[(&str, &str, &str)], to: &str, year: &str) -> Value {
    api::evaluate("transfer-balance", &tba(list, to, year, false)).unwrap()
}

fn code(command: &str, input: &Value) -> &'static str {
    api::evaluate_coded(command, input).unwrap_err().code
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

/// Each financial year's personal cap, in order.
fn caps(result: &Value) -> Vec<(String, String)> {
    result["caps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["year"].as_str().unwrap().to_string(),
                c["transfer_balance_cap"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn cap_in(result: &Value, year: &str) -> String {
    caps(result).into_iter().find(|(y, _)| y == year).unwrap().1
}

/// 'Calculating your personal transfer balance cap' (31 March 2026), Nina:
/// 25% of each increase until her 2023 credit, then 5%.
#[test]
fn nina_is_indexed_by_her_unused_cap_percentage() {
    let out = run(
        &[
            ("2018-10-01", "new_income_stream", "1200000.00"),
            ("2023-10-01", "new_income_stream", "400000.00"),
        ],
        "2026-07-01",
        "2026-27",
    );
    let expected = [
        ("2018-19", "1600000.00"),
        ("2019-20", "1600000.00"),
        ("2020-21", "1600000.00"),
        ("2021-22", "1625000.00"),
        ("2022-23", "1625000.00"),
        ("2023-24", "1675000.00"),
        ("2024-25", "1675000.00"),
        ("2025-26", "1680000.00"),
        ("2026-27", "1685000.00"),
    ];
    let got = caps(&out);
    assert_eq!(got.len(), expected.len());
    for ((y, c), (ey, ec)) in got.iter().zip(expected) {
        assert_eq!((y.as_str(), c.as_str()), (ey, ec));
    }
    let row = &out["caps"][7];
    assert_eq!(row["unused_cap_percentage"], 5);
    assert_eq!(row["highest_on"], "2023-10-01");
    assert_eq!(row["cap_on_that_day"], "1675000.00");
    assert_eq!(amount(&out, "transfer_balance"), "1600000.00");
    assert_eq!(amount(&out, "cap_space"), "85000.00");
    assert_eq!(amount(&out, "excess_transfer_balance_tax"), "0.00");
}

/// Leanne, Nada, Maryanne and Simon from the same page.
#[test]
fn the_other_indexation_examples() {
    let leanne = run(
        &[
            ("2025-10-01", "new_income_stream", "1000000.00"),
            ("2026-05-13", "commutation", "200000.00"),
        ],
        "2026-07-01",
        "2026-27",
    );
    assert_eq!(cap_in(&leanne, "2026-27"), "2050000.00");
    assert_eq!(amount(&leanne, "transfer_balance"), "800000.00");
    let first = run(&[("2024-05-14", "new_income_stream", "490000.00")], "2025-07-01", "2025-26");
    assert_eq!(cap_in(&first, "2025-26"), "1975000.00");
    let nada = run(
        &[
            ("2024-05-14", "new_income_stream", "490000.00"),
            ("2025-10-10", "new_income_stream", "510000.00"),
        ],
        "2026-07-01",
        "2026-27",
    );
    assert_eq!(cap_in(&nada, "2026-27"), "2025000.00");
    let maryanne = run(
        &[
            ("2025-12-01", "new_income_stream", "2000000.00"),
            ("2026-07-01", "commutation", "400000.00"),
        ],
        "2026-07-01",
        "2026-27",
    );
    assert_eq!(amount(&maryanne, "transfer_balance_cap"), "2000000.00");
    assert_eq!(amount(&maryanne, "cap_space"), "400000.00");
    let simon = run(
        &[
            ("2025-07-01", "new_income_stream", "2000000.00"),
            ("2026-06-30", "commutation", "2000000.00"),
            ("2026-07-01", "new_income_stream", "2100000.00"),
        ],
        "2026-07-31",
        "2026-27",
    );
    assert_eq!(amount(&simon, "transfer_balance_cap"), "2000000.00");
    assert_eq!(simon["excess_periods"][0]["from"], "2026-07-01");
    // Oracle: 1 to 31 July 2026 at 11.43% a year, compounding by the credits.
    assert_eq!(amount(&simon, "notional_earnings"), "975.34");
    assert_eq!(amount(&simon, "transfer_balance"), "2100943.73");
    assert_eq!(simon["in_excess_at_to"], true);
}

/// 'Transfer balance account' (18 June 2025), Elisabeth: a $30,000 negative
/// balance after a rollover, then $10,000 of excess from 12 October 2019.
#[test]
fn elisabeth_goes_negative_then_into_excess() {
    let list = [
        ("2017-08-02", "new_income_stream", "1600000.00"),
        ("2019-09-07", "commutation", "1630000.00"),
        ("2019-10-12", "new_income_stream", "1640000.00"),
    ];
    let out = run(&list, "2019-12-31", "2019-20");
    assert_eq!(out["ledger"][1]["transfer_balance"], "-30000.00");
    assert_eq!(out["ledger"][2]["transfer_balance"], "1610000.00");
    let period = &out["excess_periods"][0];
    assert_eq!(period["from"], "2019-10-12");
    assert_eq!(period["days"], 81);
    assert_eq!(period["notional_earnings"], "178.65");
    assert_eq!(period["tax_rate_percent"], 15);
    assert_eq!(amount(&out, "excess_transfer_balance_tax"), "26.80");
    assert_eq!(amount(&out, "transfer_balance"), "1610176.42");
    assert_eq!(amount(&out, "excess_transfer_balance"), "10176.42");
    let again = api::evaluate("transfer-balance", &tba(&list, "2019-12-31", "2019-20", true)).unwrap();
    assert_eq!(again["excess_periods"][0]["tax_rate_percent"], 30);
    assert_eq!(amount(&again, "excess_transfer_balance_tax"), "53.59");
}

/// Paolo ('Transfer balance account'): $350,000 of excess from 1 July 2017,
/// commuted on 1 August. The page's $2,745.15 of earnings matches no published
/// GIC rate; at the published 8.73% the 31 days earn $2,604.41 (oracle).
#[test]
fn paolo_commutes_his_excess() {
    let out = run(
        &[
            ("2017-07-01", "existing_income_stream", "1950000.00"),
            ("2017-08-01", "commutation", "353000.00"),
        ],
        "2017-08-01",
        "2017-18",
    );
    let period = &out["excess_periods"][0];
    assert_eq!((period["from"].as_str(), period["to"].as_str()), (Some("2017-07-01"), Some("2017-07-31")));
    assert_eq!(amount(&out, "notional_earnings"), "2604.41");
    assert_eq!(amount(&out, "transfer_balance"), "1599604.41");
    assert_eq!(amount(&out, "excess_transfer_balance_tax"), "390.66");
    assert_eq!(out["in_excess_at_to"], false);
    // No indexation once the cap was exceeded (s 294-40(1)(b)).
    let later = run(
        &[
            ("2017-07-01", "existing_income_stream", "1950000.00"),
            ("2017-08-01", "commutation", "353000.00"),
        ],
        "2021-07-01",
        "2021-22",
    );
    assert_eq!(cap_in(&later, "2021-22"), "1600000.00");
}

/// A second excess period from 1 July 2018 after one from then is taxed at
/// 30% (Imposition Act 2016 s 5(2)); the first at 15%.
#[test]
fn a_second_period_is_taxed_at_30_percent() {
    let out = run(
        &[
            ("2018-07-01", "new_income_stream", "1600000.00"),
            ("2019-01-01", "new_income_stream", "10000.00"),
            ("2019-02-01", "commutation", "20000.00"),
            ("2020-01-01", "new_income_stream", "30000.00"),
        ],
        "2020-01-31",
        "2019-20",
    );
    let periods = out["excess_periods"].as_array().unwrap();
    assert_eq!(periods.len(), 2);
    assert_eq!((&periods[0]["notional_earnings"], &periods[0]["tax_rate_percent"]), (&json!("76.21"), &json!(15)));
    assert_eq!(periods[0]["excess_transfer_balance_tax"], "11.43");
    assert_eq!((&periods[1]["notional_earnings"], &periods[1]["tax_rate_percent"]), (&json!("134.94"), &json!(30)));
    assert_eq!(periods[1]["excess_transfer_balance_tax"], "40.48");
}

/// IT(TP)A 1997 s 294-30: item 1 credits of at most $1.7 million, the excess
/// commuted by 31 December 2017, are no excess on any day to then.
#[test]
fn the_2017_transitional_rule() {
    let met = run(
        &[
            ("2017-07-01", "existing_income_stream", "1650000.00"),
            ("2017-12-01", "commutation", "50000.00"),
        ],
        "2018-06-30",
        "2017-18",
    );
    assert_eq!(amount(&met, "notional_earnings"), "0.00");
    assert!(met["excess_periods"].as_array().unwrap().is_empty());
    let short = run(
        &[
            ("2017-07-01", "existing_income_stream", "1650000.00"),
            ("2017-12-01", "commutation", "40000.00"),
        ],
        "2018-06-30",
        "2017-18",
    );
    assert_eq!(short["excess_periods"][0]["from"], "2017-07-01");
    let early = tba(&[("2017-07-01", "existing_income_stream", "1650000.00")], "2017-09-30", "2017-18", false);
    assert_eq!(code("transfer-balance", &early), "transfer_balance.transition_undecided");
}

#[test]
fn transfer_balance_refusals() {
    let refused = [
        ("reversionary_income_stream", "transfer_balance.event_refused"),
        ("child_recipient", "transfer_balance.event_refused"),
        ("capped_defined_benefit", "transfer_balance.event_refused"),
        ("structured_settlement", "transfer_balance.event_refused"),
        ("regulation_credit", "transfer_balance.event_refused"),
        ("regulation_debit", "transfer_balance.event_refused"),
        ("excess_determination", "transfer_balance.event_refused"),
        ("commutation_authority", "transfer_balance.event_refused"),
        ("non_commutable_notice", "transfer_balance.event_refused"),
        ("pension", "transfer_balance.fact_invalid"),
    ];
    for (kind, expected) in refused {
        let input = tba(&[("2020-01-01", kind, "1.00")], "2020-06-30", "2019-20", false);
        assert_eq!(code("transfer-balance", &input), expected, "{kind}");
    }
    let cases = [
        (tba(&[("2017-06-30", "new_income_stream", "1.00")], "2018-06-30", "2017-18", false), "transfer_balance.date_order"),
        (tba(&[("2017-07-02", "existing_income_stream", "1.00")], "2018-06-30", "2017-18", false), "transfer_balance.date_order"),
        (tba(&[("2020-01-01", "commutation", "1.00")], "2020-06-30", "2019-20", false), "transfer_balance.date_order"),
        (tba(&[("2020-07-01", "new_income_stream", "1.00")], "2020-06-30", "2019-20", false), "transfer_balance.date_order"),
        (tba(&[("2020-01-01", "new_income_stream", "0.00")], "2020-06-30", "2019-20", false), "transfer_balance.fact_invalid"),
        (tba(&[("2020-01-01", "new_income_stream", "1.00")], "2020-07-01", "2019-20", false), "transfer_balance.year_mismatch"),
        (tba(&[("2026-07-01", "new_income_stream", "2200000.00")], "2027-01-01", "2026-27", false), "transfer_balance.rate_unpublished"),
        (tba(&[("2020-01-01", "new_income_stream", "1.00")], "2027-07-01", "2027-28", false), "period.unsupported"),
    ];
    for (input, expected) in cases {
        assert_eq!(code("transfer-balance", &input), expected, "{input}");
    }
    let mut flag = tba(&[("2020-01-01", "new_income_stream", "1.00")], "2020-06-30", "2019-20", false);
    flag["previously_liable"] = json!("no");
    assert_eq!(code("transfer-balance", &flag), "input.flag_invalid");
    flag["previously_liable"] = json!(false);
    flag["extra"] = json!(1);
    assert_eq!(code("transfer-balance", &flag), "input.field_unknown");
}

/// The last published GIC day, 31 December 2026, still earns.
#[test]
fn excess_runs_to_the_last_published_rate() {
    let out = run(&[("2026-12-31", "new_income_stream", "2100000.01")], "2026-12-31", "2026-27");
    assert_eq!(out["excess_periods"][0]["days"], 1);
}

fn excess(question: &str, year: &str, facts: Value) -> Value {
    let mut input = facts;
    input["question"] = json!(question);
    input["year"] = json!(year);
    input["scope_confirmed"] = json!(true);
    input
}

/// Oracle over the ATO's Table 3 rates (the SIC rates, base + 3 points).
#[test]
fn the_concessional_charge_compounds_daily_to_the_day_before_tax_is_due() {
    let cases = [
        ("2020-21", "1000.00", "2021-12-01", "44.17", 518),
        ("2013-14", "2500.50", "2014-11-21", "205.03", 508),
    ];
    for (year, tax, due, charge, days) in cases {
        let input = excess("concessional_charge", year, json!({"additional_tax": tax, "tax_due_date": due}));
        let out = api::evaluate("excess-contributions", &input).unwrap();
        assert_eq!(amount(&out, "excess_concessional_contributions_charge"), charge, "{year}");
        assert_eq!(out["days"], days);
    }
    let abolished = excess("concessional_charge", "2021-22", json!({"additional_tax": "1.00", "tax_due_date": "2022-12-01"}));
    assert_eq!(code("excess-contributions", &abolished), "excess_contributions.charge_abolished");
    let early = excess("concessional_charge", "2020-21", json!({"additional_tax": "1.00", "tax_due_date": "2020-07-01"}));
    assert_eq!(code("excess-contributions", &early), "excess_contributions.date_order");
    let late = excess("concessional_charge", "2020-21", json!({"additional_tax": "1.00", "tax_due_date": "2027-01-02"}));
    assert_eq!(code("excess-contributions", &late), "excess_contributions.rate_unpublished");
}

/// Oracle: s 97-30's day-weighted proxy rate, compounding daily, rounded
/// down to the dollar; 85% released beside the excess; 15% offset.
#[test]
fn the_non_concessional_release() {
    let input = excess("non_concessional_release", "2024-25", json!({"excess": "100000.00", "determination_date": "2025-09-15"}));
    let out = api::evaluate("excess-contributions", &input).unwrap();
    assert_eq!(amount(&out, "associated_earnings"), "14663.00");
    assert_eq!(amount(&out, "total_release_amount"), "112463.55");
    assert_eq!(amount(&out, "assessable_associated_earnings"), "14663.00");
    assert_eq!(amount(&out, "associated_earnings_tax_offset"), "2199.45");
    assert_eq!(out["rates"]["proxy_rate_percent"], "11.332466");
    let warned = out["warnings"].as_array().unwrap().iter().any(|w| w.as_str().unwrap().contains("11.33% a year"));
    assert!(warned);
    let earlier = excess("non_concessional_release", "2020-21", json!({"excess": "50000.00", "determination_date": "2021-11-30"}));
    let out = api::evaluate("excess-contributions", &earlier).unwrap();
    assert_eq!(amount(&out, "associated_earnings"), "5251.00");
    let bad = [
        (excess("non_concessional_release", "2024-25", json!({"excess": "0.00", "determination_date": "2025-09-15"})), "excess_contributions.fact_invalid"),
        (excess("non_concessional_release", "2024-25", json!({"excess": "1.00", "determination_date": "2024-06-30"})), "excess_contributions.date_order"),
        (excess("non_concessional_release", "2026-27", json!({"excess": "1.00", "determination_date": "2027-09-15"})), "period.unsupported"),
        (excess("non_concessional_release", "2012-13", json!({"excess": "1.00", "determination_date": "2013-09-15"})), "period.unsupported"),
        (excess("refund", "2024-25", json!({})), "excess_contributions.question_invalid"),
        (excess("non_concessional_release", "2024-25", json!({"excess": "1.00"})), "input.missing_field"),
        (excess("non_concessional_release", "2024-25", json!({"excess": "1.00", "determination_date": "2025-09-15", "tax_due_date": "2025-12-01"})), "input.field_unknown"),
    ];
    for (input, expected) in bad {
        assert_eq!(code("excess-contributions", &input), expected, "{input}");
    }
}

/// Each contributions year's day-weighted proxy rate rounds to the ATO's
/// Table 5 figure except 2022-23, where the ATO's 9.46% is the simple average
/// of the quarters (9.4575%) and the Act's is 9.451452%.
#[test]
fn proxy_rates_against_the_atos_table_5() {
    let table = [
        ("2013-14", "9.66"), ("2014-15", "9.61"), ("2015-16", "9.20"), ("2016-17", "8.83"),
        ("2017-18", "8.73"), ("2018-19", "8.96"), ("2019-20", "8.08"), ("2020-21", "7.06"),
        ("2021-22", "7.04"), ("2022-23", "9.45"), ("2023-24", "11.19"), ("2024-25", "11.33"),
        ("2025-26", "10.75"),
    ];
    for (year, rounded) in table {
        let next: i64 = year[..4].parse::<i64>().unwrap() + 1;
        let input = excess("non_concessional_release", year, json!({"excess": "1000.00", "determination_date": format!("{next}-06-30")}));
        let out = api::evaluate("excess-contributions", &input).unwrap();
        let exact: f64 = out["rates"]["proxy_rate_percent"].as_str().unwrap().parse().unwrap();
        assert_eq!(format!("{:.2}", exact + 1e-9), rounded, "{year}");
    }
}

/// `contribution-caps` with `determination_date` adds the release figures for
/// its own excess; without it, the result is unchanged.
#[test]
fn contribution_caps_adds_the_release() {
    let mut input = json!({
        "total_super_balance": "1500000.00",
        "concessional_contributions": "0.00",
        "unused_concessional_cap": "0.00",
        "non_concessional_contributions": "400000.00",
        "under_75_in_year": true,
        "year": "2024-25",
        "scope_confirmed": true,
    });
    let plain = api::evaluate("contribution-caps", &input).unwrap();
    assert_eq!(amount(&plain, "excess_non_concessional"), "40000.00");
    assert!(plain["amounts"].get("associated_earnings").is_none());
    input["determination_date"] = json!("2025-09-15");
    let out = api::evaluate("contribution-caps", &input).unwrap();
    assert_eq!(amount(&out, "associated_earnings"), "5865.00");
    assert_eq!(amount(&out, "total_release_amount"), "44985.25");
    assert_eq!(amount(&out, "associated_earnings_tax_offset"), "879.75");
    for key in ["concessional_available", "excess_non_concessional", "non_concessional_available"] {
        assert_eq!(out["amounts"][key], plain["amounts"][key]);
    }
    input["year"] = json!("2026-27");
    input["determination_date"] = json!("2027-09-15");
    assert_eq!(code("contribution-caps", &input), "excess_contributions.rate_unpublished");
    input["year"] = json!("2024-25");
    input["determination_date"] = json!("15/09/2025");
    assert_eq!(code("contribution-caps", &input), "excess_contributions.fact_invalid");
}
