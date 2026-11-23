// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Workers compensation premiums against the site's workers compensation
//! calculator (ryanduguid.github.io at dcc7ecbe: its 25 page cases, its unit
//! tests' edge inputs and a sweep of each scheme's edges, run through the
//! page's own `workersCompensation` into
//! `data/workers_compensation_site_cases.json`), and against each scheme's
//! published examples. Every source was read on 24 November 2026
//! (docs/calculation-evidence.md#workers-compensation-premium). Every
//! employer here is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn input(place: &str, wages: &str, rate: &str) -> Value {
    let mut v = json!({
        "jurisdiction": place, "wages": wages, "rate": rate, "group_member": false,
        "year": "2026-27", "scope_confirmed": true,
    });
    let facts = match place {
        "nsw" => json!({"dust_diseases_rate": "0", "safe_employer_reward": "0"}),
        "vic" => json!({"employer": "new", "days_registered": 365, "excess_buy_out": false}),
        "qld" => json!({"policy_start": "2026-07-01"}),
        "sa" => json!({"first_full_year": false, "claims_costs": "0"}),
        _ => json!({}),
    };
    for (key, value) in facts.as_object().unwrap() {
        v[key] = value.clone();
    }
    v
}

fn with(mut v: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value.is_null() {
            v.as_object_mut().unwrap().remove(key);
        } else {
            v[key] = value.clone();
        }
    }
    v
}

fn run(input: &Value) -> Value {
    api::evaluate("workers-compensation-premium", input).unwrap()
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("workers-compensation-premium", input)
        .unwrap_err()
        .code
}

/// The command's input for one of the site's rows.
fn site_input(row: &Value) -> Value {
    let x = &row["input"];
    let text = |key: &str| x[key].as_str().unwrap();
    let place = text("place").trim_start_matches("AU-").to_lowercase();
    let mut v = input(&place, text("wages"), text("rate"));
    v["group_member"] = json!(text("group") == "yes");
    let apprentice = text("apprentice");
    if matches!(place.as_str(), "nsw" | "qld" | "sa") || !matches!(apprentice, "" | "0") {
        v["apprentice_wages"] = json!(apprentice);
    }
    if !text("actual").is_empty() {
        v["actual_wages"] = json!(text("actual"));
    }
    match place.as_str() {
        "nsw" => {
            v["dust_diseases_rate"] = json!(text("dust"));
            v["safe_employer_reward"] = json!(text("reward"));
        }
        "vic" => {
            v["employer"] = json!(text("vicEmployer"));
            v["days_registered"] = json!(text("days").parse::<i64>().unwrap());
            v["excess_buy_out"] = json!(text("buyout") == "yes");
        }
        "qld" => {
            let start = match text("qldStart") {
                "q1" => "2026-07-01",
                "q2" => "2026-10-01",
                "q3" => "2027-01-01",
                "q4" => "2027-04-01",
                _ => "2026-06-30",
            };
            v["policy_start"] = json!(start);
        }
        "sa" => {
            let first = text("firstYear") == "yes";
            v["first_full_year"] = json!(first);
            if first {
                v.as_object_mut().unwrap().remove("claims_costs");
            } else {
                v["claims_costs"] = json!(text("claims"));
            }
        }
        _ => {}
    }
    v
}

/// The code the worksheet gives for the page's refusal.
fn site_code(message: &str) -> &'static str {
    [
        ("neither new nor small", "experience_rated"),
        ("more than $30,000", "experience_rated"),
        ("Apprentice wages are part", "apprentice_above_wages"),
        ("commenced before 1 July 2026", "policy_before_year"),
        ("below icare", "below_minimum"),
        ("member of a group", "group_rated"),
        ("no apprentice deduction", "apprentice_unavailable"),
    ]
    .iter()
    .find(|(said, _)| message.contains(said))
    .map(|(_, code)| *code)
    .unwrap_or_else(|| panic!("an unmapped refusal: {message}"))
}

const FIGURES: [&str; 6] = [
    "premium",
    "gst",
    "duty",
    "total",
    "actual_total",
    "adjustment",
];

/// How one row of the site's fixture came out: the same figures, the same
/// refusal, or a refusal where a source is unsettled.
fn checked(row: &Value) -> &'static str {
    let v = site_input(row);
    let site = &row["site"];
    if let Some(message) = site["refused"].as_str() {
        let err = api::evaluate_coded("workers-compensation-premium", &v).unwrap_err();
        let want = format!("workers_compensation_premium.{}", site_code(message));
        assert_eq!(err.code, want, "{row}");
        if message.starts_with("On the actual wages") {
            assert!(err.message.starts_with("On the actual wages"), "{row}");
        }
        return "refused alike";
    }
    match api::evaluate_coded("workers-compensation-premium", &v) {
        Ok(result) => {
            for key in FIGURES {
                let got = result["amounts"][key].as_str();
                assert_eq!(got, site[key].as_str(), "{key}: {row}");
            }
            "identical"
        }
        Err(err) => match err.code {
            "workers_compensation_premium.band_edge_unsettled" => "SA band edge refused",
            "workers_compensation_premium.rounding_unsettled" => "Vic cl 6(1) refused",
            other => panic!("{other}: {row}"),
        },
    }
}

/// Every row of the site's fixture gives the page's figures or refusal,
/// except where a source is unsettled; the page's own 25 cases all agree.
#[test]
fn the_site_cases_agree_except_where_a_source_is_unsettled() {
    let data: Value =
        serde_json::from_str(include_str!("data/workers_compensation_site_cases.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    let mut tally = BTreeMap::<&str, usize>::new();
    let mut page = BTreeMap::<&str, usize>::new();
    for row in rows {
        let outcome = checked(row);
        *tally.entry(outcome).or_default() += 1;
        if row["source"].as_str().unwrap().starts_with("page: ") {
            *page.entry(outcome).or_default() += 1;
        }
    }
    eprintln!(
        "workers-compensation-premium site parity: {} rows {tally:?}; page cases {page:?}",
        rows.len()
    );
    assert_eq!(rows.len(), 1326);
    assert_eq!(
        page,
        BTreeMap::from([("identical", 18), ("refused alike", 7)])
    );
    assert_eq!(
        tally,
        BTreeMap::from([
            ("identical", IDENTICAL),
            ("refused alike", 308),
            ("SA band edge refused", SA_EDGE),
        ])
    );
}

const IDENTICAL: usize = 994;
const SA_EDGE: usize = 24;

#[test]
fn each_scheme_matches_its_published_example() {
    // NT WorkSafe's builder: $400,000 x 3.50% = $14,000 + $1,400; $500,000
    // actual gives $19,250, $3,850 more.
    let nt = run(&with(
        input("nt", "400000", "3.5"),
        json!({"actual_wages": "500000"}),
    ));
    let figures = ["premium", "gst", "total", "actual_total", "adjustment"].map(|k| amount(&nt, k));
    assert_eq!(
        figures,
        ["14000.00", "1400.00", "15400.00", "19250.00", "3850.00"]
    );
    // ReturnToWorkSA's examples 1 to 3.
    for (wages, rate, claims, premium) in [
        ("450000", "2", "0", "8550.00"),
        ("450000", "2", "2500", "9900.00"),
        ("1000000", "6", "5000", "56000.00"),
    ] {
        let sa = run(&with(
            input("sa", wages, rate),
            json!({"claims_costs": claims}),
        ));
        assert_eq!(amount(&sa, "premium"), premium, "{wages} {claims}");
    }
    // icare's Rob's Restaurant: $1,200,000 at 2.250%, APP $27,000 less the
    // $2,025 Safe Employer Reward.
    let rob = run(&with(
        input("nsw", "1200000", "2.25"),
        json!({"safe_employer_reward": "7.5", "dust_diseases_rate": "0.022"}),
    ));
    let figures = [
        "average_performance_premium",
        "safe_employer_reward",
        "premium",
    ]
    .map(|k| amount(&rob, k));
    assert_eq!(figures, ["27000.00", "2025.00", "24975.00"]);
    // Premiums Order (No. 34) with Gazette S 284: 2% of $150,000 less 2% of
    // $15,500.
    let vic = run(&input("vic", "150000", "2"));
    let figures = [
        "preliminary_premium",
        "deductible_benefit",
        "workcover_premium",
        "total",
    ]
    .map(|k| amount(&vic, k));
    assert_eq!(figures, ["3000.00", "310.00", "2690.00", "2959.00"]);
}

#[test]
fn queensland_minimums_follow_the_quarter_the_policy_commences_in() {
    for (start, total) in [
        ("2026-07-01", "200.00"),
        ("2026-09-30", "200.00"),
        ("2026-10-01", "150.00"),
        ("2026-12-31", "150.00"),
        ("2027-01-01", "100.00"),
        ("2027-04-01", "50.00"),
        ("2027-06-30", "50.00"),
    ] {
        let qld = run(&with(
            input("qld", "0", "1.343"),
            json!({"policy_start": start}),
        ));
        assert_eq!(amount(&qld, "total"), total, "{start}");
    }
    let late = with(
        input("qld", "0", "1.343"),
        json!({"policy_start": "2027-07-01"}),
    );
    assert_eq!(
        code(&late),
        "workers_compensation_premium.policy_after_year"
    );
    let early = with(
        input("qld", "0", "1.343"),
        json!({"policy_start": "2026-06-30"}),
    );
    assert_eq!(
        code(&early),
        "workers_compensation_premium.policy_before_year"
    );
    // $500,000 at 1.343%: duty is 5% of the premium and its GST.
    let qld = run(&input("qld", "500000", "1.343"));
    let figures = ["premium", "gst", "duty", "total"].map(|k| amount(&qld, k));
    assert_eq!(figures, ["6715.00", "671.50", "369.33", "7755.83"]);
}

#[test]
fn a_south_australian_base_premium_just_under_a_band_bound_is_refused() {
    // $9,999.995 of base premium: the table's 'under $10,000' gives 5%, the
    // calculator's lookup 10%.
    let edge = input("sa", "499999.75", "2");
    assert_eq!(
        code(&edge),
        "workers_compensation_premium.band_edge_unsettled"
    );
    for (wages, discount) in [("499999.50", "500.00"), ("500000", "1000.00")] {
        let sa = run(&input("sa", wages, "2"));
        assert_eq!(amount(&sa, "discount"), discount, "{wages}");
    }
    // The first full year takes no discount, so no edge applies.
    let first = with(edge, json!({"first_full_year": true, "claims_costs": null}));
    assert_eq!(amount(&run(&first), "premium"), "9999.00");
}

#[test]
fn a_victorian_deductible_that_six_place_rounding_moves_is_refused() {
    // 1% x 15,500 x 1 / 365 is $0.424657..., $0.42; with 1 / 365 to 6 places
    // (0.002740) it is $0.4247, the same cent. Find a case that moves.
    let mut moved = None;
    'search: for days in 1..=365 {
        for rate in ["0.83", "1.807", "2", "3.5", "7.123"] {
            let v = with(
                input("vic", "150000", rate),
                json!({"days_registered": days}),
            );
            if api::evaluate_coded("workers-compensation-premium", &v).is_err() {
                moved = Some(v);
                break 'search;
            }
        }
    }
    let moved = moved.expect("a case where the 2 readings differ");
    assert_eq!(
        code(&moved),
        "workers_compensation_premium.rounding_unsettled"
    );
}

#[test]
fn each_scheme_asks_only_for_its_own_facts() {
    let nt = input("nt", "400000", "3.5");
    for (key, value) in [
        ("dust_diseases_rate", json!("0")),
        ("employer", json!("new")),
        ("policy_start", json!("2026-07-01")),
        ("first_full_year", json!(true)),
    ] {
        assert_eq!(
            code(&with(nt.clone(), json!({key: value}))),
            "input.field_unknown",
            "{key}"
        );
    }
    let nsw = with(
        input("nsw", "400000", "3.5"),
        json!({"dust_diseases_rate": null}),
    );
    assert_eq!(code(&nsw), "input.missing_field");
    let first = with(input("sa", "450000", "2"), json!({"first_full_year": true}));
    assert_eq!(code(&first), "input.field_unknown");
    let wa = with(
        input("wa", "400000", "3.5"),
        json!({"apprentice_wages": "0"}),
    );
    assert_eq!(amount(&run(&wa), "total"), "15400.00");
}

#[test]
fn invalid_inputs_are_refused() {
    let nt = input("nt", "400000", "3.5");
    for (change, want) in [
        (
            json!({"jurisdiction": "au"}),
            "workers_compensation_premium.jurisdiction_invalid",
        ),
        (
            json!({"rate": "0"}),
            "workers_compensation_premium.rate_invalid",
        ),
        (
            json!({"rate": "100.0001"}),
            "workers_compensation_premium.rate_invalid",
        ),
        (
            json!({"rate": "1.23456"}),
            "workers_compensation_premium.rate_invalid",
        ),
        (
            json!({"rate": 3.5}),
            "workers_compensation_premium.rate_invalid",
        ),
        (json!({"wages": "1.234"}), "input.amount_invalid"),
        (json!({"year": "2025-26"}), "period.unsupported"),
        (json!({"group_member": "no"}), "input.flag_invalid"),
    ] {
        assert_eq!(code(&with(nt.clone(), change.clone())), want, "{change}");
    }
}

#[test]
fn invalid_scheme_facts_are_refused() {
    let nsw = input("nsw", "400000", "3.5");
    for change in [
        json!({"dust_diseases_rate": "10.0001"}),
        json!({"safe_employer_reward": "-1"}),
    ] {
        assert_eq!(
            code(&with(nsw.clone(), change.clone())),
            "workers_compensation_premium.rate_invalid",
            "{change}"
        );
    }
    let vic = input("vic", "400000", "3.5");
    for (change, want) in [
        (
            json!({"employer": "large"}),
            "workers_compensation_premium.employer_invalid",
        ),
        (
            json!({"days_registered": 0}),
            "workers_compensation_premium.days_invalid",
        ),
        (
            json!({"days_registered": 366}),
            "workers_compensation_premium.days_invalid",
        ),
        (
            json!({"days_registered": "365"}),
            "workers_compensation_premium.days_invalid",
        ),
        (json!({"excess_buy_out": 1}), "input.flag_invalid"),
    ] {
        assert_eq!(code(&with(vic.clone(), change.clone())), want, "{change}");
    }
    let qld = with(
        input("qld", "400000", "3.5"),
        json!({"policy_start": "1 July 2026"}),
    );
    assert_eq!(code(&qld), "input.date_invalid");
}
