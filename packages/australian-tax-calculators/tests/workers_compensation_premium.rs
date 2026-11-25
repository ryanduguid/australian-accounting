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
        "sa" => json!({"new_employer": false, "claims_costs": "0"}),
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
            v["new_employer"] = json!(first);
            if first {
                v.as_object_mut().unwrap().remove("claims_costs");
            } else {
                v["claims_costs"] = json!(text("claims"));
            }
            if !text("actual").is_empty() {
                v["actual_apprentice_wages"] = json!(apprentice);
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
        (
            "after premium payable is calculated",
            "apprentice_unsettled",
        ),
        ("no year-end adjustment", "adjustment_unsettled"),
        ("rounds to $0", "remuneration_nil"),
        ("lower discount band", "band_edge_unsettled"),
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

/// The refusals where a source is unsettled or departs from the page, by the
/// tally's name for them.
const DEPARTURES: [(&str, &str); 6] = [
    ("band_edge_unsettled", "SA band edge refused"),
    ("rounding_unsettled", "Qld rounding order refused"),
    ("adjustment_unsettled", "Qld actual wages refused"),
    ("apprentice_unsettled", "NSW apprentice incentive refused"),
    ("remuneration_nil", "Vic nil remuneration refused"),
    ("experience_rated", "NSW exact APP over $30,000 refused"),
];

fn departure(code: &str) -> Option<&'static str> {
    let code = code.strip_prefix("workers_compensation_premium.")?;
    DEPARTURES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, name)| *name)
}

/// How one row of the site's fixture came out: the same figures, the same
/// refusal, or a counted departure where a source is unsettled or decides
/// otherwise.
fn checked(row: &Value) -> &'static str {
    let v = site_input(row);
    let site = &row["site"];
    let got = api::evaluate_coded("workers-compensation-premium", &v);
    if let Some(message) = site["refused"].as_str() {
        let err = got.unwrap_err();
        let want = format!("workers_compensation_premium.{}", site_code(message));
        if err.code != want {
            assert!(departure(err.code).is_some(), "{}: {row}", err.code);
        } else if message.starts_with("On the actual wages") {
            assert!(err.message.starts_with("On the actual wages"), "{row}");
        }
        return "refused alike";
    }
    match got {
        Ok(result) => {
            let same = FIGURES
                .iter()
                .all(|key| result["amounts"][key].as_str() == site[key].as_str());
            if same {
                return "identical";
            }
            assert_eq!(v["jurisdiction"], "vic", "{row}");
            "Vic deductible with N / 365 to 6 places"
        }
        Err(err) => departure(err.code).unwrap_or_else(|| panic!("{}: {row}", err.code)),
    }
}

/// Every row of the site's fixture gives the page's figures or refusal;
/// the page's own 25 cases all agree.
#[test]
fn the_site_cases_agree() {
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
    assert_eq!(tally, BTreeMap::from(TALLY));
}

const TALLY: [(&str, usize); 2] = [("identical", 849), ("refused alike", 477)];

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
}

#[test]
fn icare_and_worksafe_victoria_examples_agree() {
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
        "dust_diseases_contribution",
        "gst",
        "total",
    ]
    .map(|k| amount(&rob, k));
    // The gst amount excludes the GST within the dust diseases contribution.
    assert_eq!(
        figures,
        [
            "27000.00", "2025.00", "24975.00", "264.00", "2497.50", "27736.50"
        ]
    );
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
        ("2027-03-31", "100.00"),
        ("2027-04-01", "50.00"),
        ("2027-06-30", "50.00"),
    ] {
        let qld = run(&with(
            input("qld", "0", "1.343"),
            json!({"policy_start": start}),
        ));
        assert_eq!(amount(&qld, "total"), total, "{start}");
    }
}

#[test]
fn queensland_minimums_split_into_premium_gst_and_duty() {
    // Each minimum's premium, GST and duty: the premium is the minimum / 1.155.
    for (start, parts) in [
        ("2026-07-01", ["173.16", "17.32", "9.52"]),
        ("2026-10-01", ["129.87", "12.99", "7.14"]),
        ("2027-01-01", ["86.58", "8.66", "4.76"]),
        ("2027-04-01", ["43.29", "4.33", "2.38"]),
    ] {
        let qld = run(&with(
            input("qld", "0", "1.343"),
            json!({"policy_start": start}),
        ));
        let got = ["premium", "gst", "duty"].map(|k| amount(&qld, k));
        assert_eq!(got, parts, "{start}");
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
    let first = with(edge, json!({"new_employer": true, "claims_costs": null}));
    assert_eq!(amount(&run(&first), "premium"), "9999.00");
}

#[test]
fn victoria_rounds_n_over_365_to_6_places_and_truncates_the_premiums() {
    // 7.123% x 15,500 x 0.002740 (1 / 365 to 6 places, cl 6(1)) is
    // $3.0251381, $3.03 (cl 6(4)); exactly, 1 / 365 gives $3.02.
    let vic = run(&with(
        input("vic", "150000", "7.123"),
        json!({"days_registered": 1}),
    ));
    let figures = [
        "preliminary_premium",
        "deductible_benefit",
        "workcover_premium",
        "gst",
        "total",
    ]
    .map(|k| amount(&vic, k));
    assert_eq!(
        figures,
        ["10684.50", "3.03", "10681.47", "1068.15", "11749.62"]
    );
    // $2,710.51807 less $280.09 is $2,430.42807, down to $2,430.42 (cl 6(2));
    // the buy-out is 0.1 of it down to the cent, $243.04.
    let vic = run(&with(
        input("vic", "150001", "1.807"),
        json!({"excess_buy_out": true}),
    ));
    let figures =
        ["deductible_benefit", "workcover_premium", "buy_out_premium"].map(|k| amount(&vic, k));
    assert_eq!(figures, ["280.09", "2430.42", "243.04"]);
    // Remuneration is to the dollar (cl 6(6)): $0.49 is nil, which cl 11
    // divides by, and $0.50 is $1, on the $400 minimum.
    for wages in ["0", "0.49"] {
        assert_eq!(
            code(&input("vic", wages, "2")),
            "workers_compensation_premium.remuneration_nil",
            "{wages}"
        );
    }
    let one = run(&input("vic", "0.50", "2"));
    assert_eq!(amount(&one, "workcover_premium"), "400.00");
}

#[test]
fn unsettled_new_south_wales_and_queensland_cases_are_refused() {
    // $1,500,000.01 at 2% is $30,000.0002 of APP, over the limit; $1,500,000
    // is exactly $30,000, which is not.
    assert_eq!(
        code(&input("nsw", "1500000.01", "2")),
        "workers_compensation_premium.experience_rated"
    );
    let at_limit = run(&input("nsw", "1500000", "2"));
    assert_eq!(amount(&at_limit, "premium"), "30000.00");
    let apprentice = with(
        input("nsw", "400000", "2"),
        json!({"apprentice_wages": "1"}),
    );
    assert_eq!(
        code(&apprentice),
        "workers_compensation_premium.apprentice_unsettled"
    );
    // $1,000.005 less $200.0002 is $800.01 rounded step by step and $800.00
    // rounded once.
    let qld = with(
        input("qld", "50000.25", "2"),
        json!({"apprentice_wages": "10000.01"}),
    );
    assert_eq!(
        code(&qld),
        "workers_compensation_premium.rounding_unsettled"
    );
    let actual = with(
        input("qld", "500000", "1.343"),
        json!({"actual_wages": "1"}),
    );
    assert_eq!(
        code(&actual),
        "workers_compensation_premium.adjustment_unsettled"
    );
}

#[test]
fn south_australia_caps_claims_on_gross_base_premium_and_takes_actual_apprentices() {
    // BP $9,000, discount $450, claims $2,500 capped at $1,350, apprentices
    // $1,000: $8,900.
    let sa = run(&with(
        input("sa", "450000", "2"),
        json!({"claims_costs": "2500", "apprentice_wages": "50000"}),
    ));
    assert_eq!(amount(&sa, "premium"), "8900.00");
    // Estimate $100,000 with $10,000 of apprentices; actual $120,000 with
    // $20,000: $1,870 then $2,068, $198 more.
    let sa = run(&with(
        input("sa", "100000", "2"),
        json!({
            "apprentice_wages": "10000",
            "actual_wages": "120000",
            "actual_apprentice_wages": "20000",
        }),
    ));
    let figures = ["total", "actual_total", "adjustment"].map(|k| amount(&sa, k));
    assert_eq!(figures, ["1870.00", "2068.00", "198.00"]);
    let missing = with(
        input("sa", "100000", "2"),
        json!({"actual_wages": "120000"}),
    );
    assert_eq!(code(&missing), "input.missing_field");
}

#[test]
fn each_scheme_asks_only_for_its_own_facts() {
    let nt = input("nt", "400000", "3.5");
    for (key, value) in [
        ("dust_diseases_rate", json!("0")),
        ("employer", json!("new")),
        ("policy_start", json!("2026-07-01")),
        ("new_employer", json!(true)),
        ("actual_apprentice_wages", json!("0")),
    ] {
        assert_eq!(
            code(&with(nt.clone(), json!({key: value}))),
            "input.field_unknown",
            "{key}"
        );
    }
    for place in ["nsw", "vic", "qld", "wa", "tas", "act", "nt"] {
        let claims = with(input(place, "400000", "3.5"), json!({"claims_costs": "0"}));
        assert_eq!(code(&claims), "input.field_unknown", "{place}");
    }
    let nsw = with(
        input("nsw", "400000", "3.5"),
        json!({"dust_diseases_rate": null}),
    );
    assert_eq!(code(&nsw), "input.missing_field");
    let first = with(input("sa", "450000", "2"), json!({"new_employer": true}));
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
