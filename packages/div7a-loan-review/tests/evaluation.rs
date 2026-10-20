//! The evaluation pack against its pinned expectations, re-performed through
//! the command line, and the workbook's frozen copy of the engine's review.

use div7a_loan_review::{api, cli};
use serde_json::{Value, json};
use std::path::Path;

const PACK: &str = "evaluation/div7a_myr";

fn expected() -> Value {
    serde_json::from_str(include_str!(
        "../evaluation/div7a_myr/expected_results.json"
    ))
    .unwrap()
}

fn scenarios() -> Vec<Value> {
    expected()["scenarios"].as_array().unwrap().clone()
}

fn run(args: &[&str]) -> (i32, String) {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    let (code, out, err) = cli::run(&args);
    assert_eq!(err, "", "{args:?}");
    (code, out)
}

fn review(scenario: &Value) -> (i32, Value) {
    let fixture = format!("{PACK}/fixtures/{}", scenario["fixture"].as_str().unwrap());
    let year = scenario["year_of_income"].as_str().unwrap();
    let (code, out) = run(&[
        "review", "--input", &fixture, "--year", year, "--format", "json",
    ]);
    (code, serde_json::from_str(&out).unwrap())
}

#[test]
fn every_scenario_reproduces_its_pinned_result() {
    for scenario in scenarios() {
        let id = &scenario["id"];
        let fixture = format!("{PACK}/fixtures/{}", scenario["fixture"].as_str().unwrap());
        assert!(Path::new(&fixture).is_file(), "{id}");
        let (code, document) = review(&scenario);
        let line = &document["lines"][0];
        assert_eq!(
            line["gate"]["verdict"], scenario["expected_gate_verdict"],
            "{id}"
        );
        let myr = &line["myr"];
        assert_eq!(myr["verdict"], scenario["expected_myr_verdict"], "{id}");
        assert_eq!(
            myr["myr_required"], scenario["expected_myr_required"],
            "{id}"
        );
        assert_eq!(
            myr["payments_applied"], scenario["expected_payments_applied"],
            "{id}"
        );
        assert_eq!(myr["shortfall"], scenario["expected_shortfall"], "{id}");
        assert_eq!(
            myr["experimental_deemed_dividend_exposure"], scenario["expected_exposure"],
            "{id}"
        );
        assert_eq!(json!(code), scenario["expected_exit"], "{id}");
        if scenario["hand_worked"] == true {
            // The README's paper working uses the numbers the engine plugged in.
            let inputs = &scenario["inputs"];
            assert_eq!(
                myr["amalgamated_loan_unpaid_at_end_of_previous_year"],
                inputs["amalgamated_loan_unpaid_at_end_of_previous_year"],
                "{id}"
            );
            assert_eq!(myr["benchmark_rate"], inputs["benchmark_rate"], "{id}");
            assert_eq!(
                myr["remaining_term_years_used"], inputs["remaining_term_years"],
                "{id}"
            );
            let recomputed = api::evaluate(
                "minimum-yearly-repayment-amount",
                &json!({
                    "principal": inputs["amalgamated_loan_unpaid_at_end_of_previous_year"],
                    "rate": inputs["benchmark_rate"],
                    "remaining_term": inputs["remaining_term_years"],
                }),
            )
            .unwrap();
            assert_eq!(recomputed, scenario["expected_myr_required"], "{id}");
        }
    }
}

/// `f"{Decimal(text):,}"` for a plain decimal string.
fn grouped(text: &str) -> String {
    let (whole, frac) = text.split_once('.').unwrap_or((text, ""));
    let mut out = String::new();
    for (i, ch) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if frac.is_empty() {
        out
    } else {
        format!("{out}.{frac}")
    }
}

#[test]
fn the_readme_prints_every_hand_worked_figure() {
    let readme = include_str!("../evaluation/div7a_myr/README.md");
    let mut hand_worked = 0;
    for scenario in scenarios().iter().filter(|s| s["hand_worked"] == true) {
        hand_worked += 1;
        let id = &scenario["id"];
        assert!(
            readme.contains(scenario["expected_myr_required"].as_str().unwrap()),
            "{id}"
        );
        assert!(
            readme.contains(scenario["inputs"]["benchmark_rate"].as_str().unwrap()),
            "{id}"
        );
        let shortfall = scenario["expected_shortfall"].as_str().unwrap();
        if shortfall != "0.00" {
            assert!(
                readme.contains(&grouped(shortfall)) || readme.contains(shortfall),
                "{id}"
            );
        }
    }
    assert!(hand_worked >= 3);
}

#[test]
fn the_pack_names_its_sources_and_covers_every_outcome() {
    let pack = expected();
    assert!(
        pack["law_compilation"]
            .as_str()
            .unwrap()
            .starts_with("C1936A00027")
    );
    let urls: Vec<&str> = pack["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["url"].as_str().unwrap())
        .collect();
    for host in ["legislation.gov.au", "rba.gov.au", "duguid.com.au"] {
        assert!(urls.iter().any(|u| u.contains(host)), "{host}");
    }
    for verdict in ["MYR_MET", "MYR_SHORT", "REFUSED", "UNKNOWN"] {
        assert!(
            scenarios()
                .iter()
                .any(|s| s["expected_myr_verdict"] == verdict),
            "{verdict}"
        );
    }
}

/// Every identifier in the pack is synthetic: `F1-...` loan ids, `SYN-001`
/// references, no exclusion text and only flags and numbers elsewhere.
#[test]
fn no_fixture_carries_anything_that_looks_like_a_client_record() {
    for entry in std::fs::read_dir(format!("{PACK}/fixtures")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "csv") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let mut lines = text.lines();
        let header: Vec<&str> = lines.next().unwrap().split(',').collect();
        for line in lines.filter(|l| !l.is_empty()) {
            for (key, value) in header.iter().zip(line.split(',')) {
                let ok = match *key {
                    "loan_id" => {
                        let rest = value.strip_prefix('F').unwrap_or("");
                        rest.len() > 2
                            && ('1'..='6').contains(&rest.chars().next().unwrap())
                            && rest[1..].starts_with('-')
                            && rest[2..]
                                .chars()
                                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
                    }
                    "borrower_reference" => {
                        value.len() == 7
                            && value.starts_with("SYN-00")
                            && ('1'..='6').contains(&value.chars().last().unwrap())
                    }
                    "out_of_scope_reason" => value.is_empty(),
                    _ => {
                        ["true", "false", "unknown"].contains(&value)
                            || (!value.is_empty()
                                && value
                                    .chars()
                                    .all(|c| c.is_ascii_digit() || c == '.' || c == '-'))
                    }
                };
                assert!(ok, "{} {key}={value:?}", path.display());
            }
        }
    }
}

/// The workbook test at the repository root reads this frozen review; it must
/// stay the engine's own answer.
#[test]
fn the_workbook_sample_review_is_the_engines() {
    let (code, out) = run(&[
        "review",
        "--input",
        "examples/sample_loans_mixed.csv",
        "--year",
        "2026-27",
        "--format",
        "json",
    ]);
    assert_eq!(code, 2);
    let frozen = include_str!("data/workbook-sample.json").replace("\r\n", "\n");
    assert_eq!(
        out, frozen,
        "regenerate tests/data/workbook-sample.json with `review --format json` after a deliberate change"
    );
}
