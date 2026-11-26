// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The personal services income rules (ITAA 1997 Part 2-42, compilations 210,
//! 223 and 268, read on 22 November 2026) against the 19 cases of the site's
//! PSI page (ryanduguid.github.io `_data/calculator_cases.json`,
//! `psi-determination`, checked 17 November 2026), which include TR 2022/3
//! examples 20, 29, 30 and 40, and hand-worked cases at each threshold
//! (docs/calculation-evidence.md#psi-test).
use australian_tax_calculators::api;
use serde_json::{Value, json};

const PSB: &str = "a personal services business, so Divisions 85 and 86 do not apply";
const S87_15_3: &str =
    "not a personal services business unless a determination is in force, so Division 85 applies";
const S87_15_1: &str = "not a personal services business, so Division 85 applies";

fn run(input: &Value) -> Value {
    api::evaluate("psi-test", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("psi-test", input).unwrap_err().code
}

/// A payer: `flags` holds a for associate, o for offer, r for result, t for
/// tools and d for defects.
fn payer(amount: &str, flags: &str) -> Value {
    json!({
        "amount": amount,
        "associate": flags.contains('a'),
        "offer": flags.contains('o'),
        "result": flags.contains('r'),
        "tools": flags.contains('t'),
        "defects": flags.contains('d'),
    })
}

/// The business premises answers: each of the 5 named in `ticked` is true.
fn premises(ticked: &[&str]) -> Value {
    let keys = [
        "at_all_times",
        "mainly_for_psi",
        "exclusive_use",
        "separate_from_private",
        "separate_from_clients",
    ];
    Value::Object(
        keys.iter()
            .map(|k| ((*k).to_owned(), json!(ticked.contains(k))))
            .collect(),
    )
}

/// A contractor's PSI from the payers given, with no determination, no
/// others' work, no apprentice and no premises.
fn base(payers: &[(&str, &str)]) -> Value {
    json!({
        "year": "2025-26", "scope_confirmed": true, "earner": "sole_trader",
        "mainly_personal_efforts": true, "determination_in_force": false,
        "payers": payers.iter().map(|(a, f)| payer(a, f)).collect::<Vec<_>>(),
        "agent": false, "principal_work_by_others": "0.00", "apprentice": false,
        "premises": premises(&[]),
    })
}

fn with(mut input: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            input.as_object_mut().unwrap().remove(key);
        } else {
            input[key] = value.clone();
        }
    }
    input
}

fn decided(result: &Value) -> (&str, &str) {
    (
        result["rule"].as_str().unwrap(),
        result["effect"].as_str().unwrap(),
    )
}

/// The site's figure names and the worksheet's.
fn figure<'a>(result: &'a Value, site: &str) -> &'a Value {
    let key = match site {
        "results" => "results_test_percent",
        "source" => "one_source_percent",
        "employment" => "employment_percent",
        "unrelated" => "unrelated_clients",
        other => panic!("no figure {other}"),
    };
    &result["tests"][key]
}

/// A case's name and facts, then its rule, effect and figures.
type Case = (
    &'static str,
    Value,
    &'static str,
    &'static str,
    Vec<(&'static str, f64)>,
);

/// The site's 16 decided cases, and a contractor's determination: the facts,
/// then the rule, the effect and the figures its page shows.
#[test]
fn the_site_cases_decide_alike() {
    let all = premises(&[
        "at_all_times",
        "mainly_for_psi",
        "exclusive_use",
        "separate_from_private",
    ]);
    let cases: Vec<Case> = vec![
        (
            "TR 2022/3 example 20: Joe",
            base(&[("18000.00", "rtd")]),
            "subsection 87-18(1)",
            PSB,
            vec![("results", 100.0)],
        ),
        (
            "TR 2022/3 example 30: Ian",
            with(
                base(&[("100000.00", ""), ("40000.00", "")]),
                json!({"principal_work_by_others": "26000.00"}),
            ),
            "subsection 87-15(1)",
            S87_15_1,
            vec![("source", 71.42), ("employment", 18.57)],
        ),
        (
            "TR 2022/3 example 29: Tom",
            with(
                base(&[("156000.00", "")]),
                json!({"principal_work_by_others": "36000.00"}),
            ),
            "subsection 87-15(3)",
            S87_15_3,
            vec![("source", 100.0), ("employment", 23.07)],
        ),
        (
            "exactly 75% for a result",
            base(&[("75.00", "rtd"), ("25.00", "")]),
            "subsection 87-18(1)",
            PSB,
            vec![("results", 75.0)],
        ),
        (
            "74.99% for a result",
            base(&[("74.99", "rtd"), ("25.01", "")]),
            "subsection 87-15(1)",
            S87_15_1,
            vec![("results", 74.99)],
        ),
        (
            "a result without liability for defects",
            base(&[("100.00", "rt")]),
            "subsection 87-15(3)",
            S87_15_3,
            vec![("results", 0.0)],
        ),
        (
            "exactly 80% from one payer",
            with(
                base(&[("80.00", "o"), ("20.00", "o")]),
                json!({"apprentice": true}),
            ),
            "subsection 87-15(3)",
            S87_15_3,
            vec![("source", 80.0), ("unrelated", 2.0)],
        ),
        (
            "79.99% from one payer and 2 unrelated clients",
            base(&[("79.99", "o"), ("20.01", "o")]),
            "section 87-20",
            PSB,
            vec![("source", 79.99), ("unrelated", 2.0)],
        ),
        (
            "an associate is not an unrelated client",
            base(&[("50.00", "ao"), ("50.00", "o")]),
            "subsection 87-15(1)",
            S87_15_1,
            vec![("unrelated", 1.0)],
        ),
        (
            "exactly 20% of principal work by others",
            with(
                base(&[("60.00", ""), ("40.00", "")]),
                json!({"principal_work_by_others": "20.00"}),
            ),
            "section 87-25",
            PSB,
            vec![("employment", 20.0)],
        ),
        (
            "business premises with 4 of the 5 answers",
            with(
                base(&[("60.00", ""), ("40.00", "")]),
                json!({"premises": all}),
            ),
            "subsection 87-15(1)",
            S87_15_1,
            vec![],
        ),
        (
            "an entity meets the results test",
            with(base(&[("120000.00", "rtd")]), json!({"earner": "entity"})),
            "subsection 87-18(3)",
            PSB,
            vec![("results", 100.0)],
        ),
        (
            "not mainly personal efforts or skills",
            with(
                base(&[("100000.00", "")]),
                json!({"mainly_personal_efforts": false}),
            ),
            "subsection 84-5(1)",
            "not personal services income, so Part 2-42 does not apply",
            vec![],
        ),
        (
            "an employee",
            with(base(&[("100000.00", "")]), json!({"earner": "employee"})),
            "subsection 85-35(1)",
            "income as an employee, so the PSI rules do not apply",
            vec![],
        ),
        (
            "a determination in force for an entity",
            with(
                base(&[("100000.00", "")]),
                json!({"earner": "entity", "determination_in_force": true}),
            ),
            "paragraph 87-15(1)(b)",
            PSB,
            vec![],
        ),
        (
            "TR 2022/3 example 40: Chris on 70% commission",
            with(
                base(&[("100000.00", "")]),
                json!({"agent": true, "commission": "70000.00", "principal_payments": "100000.00"}),
            ),
            "subsection 87-15(3)",
            S87_15_3,
            vec![("source", 100.0)],
        ),
        (
            "a determination in force for a contractor",
            with(
                base(&[("100000.00", "")]),
                json!({"determination_in_force": true}),
            ),
            "paragraph 87-15(1)(a)",
            PSB,
            vec![],
        ),
    ];
    for (name, input, rule, effect, figures) in cases {
        let out = run(&input);
        assert_eq!(decided(&out), (rule, effect), "{name}");
        for (site, want) in figures {
            let got = figure(&out, site);
            let got = got
                .as_str()
                .map_or_else(|| got.as_f64().unwrap(), |s| s.parse().unwrap());
            assert!((got - want).abs() < 1e-9, "{name}: {site} {got} != {want}");
        }
    }
}

/// The site's refused cases: 75% commission, no PSI and others' work over
/// the PSI.
#[test]
fn the_site_refusals_refuse_alike() {
    let agent = with(
        base(&[("100000.00", "")]),
        json!({"agent": true, "commission": "75000.00", "principal_payments": "100000.00"}),
    );
    assert_eq!(code(&agent), "psi_test.agent_rules");
    assert_eq!(code(&base(&[("0.00", "")])), "psi_test.no_psi");
    let over = with(
        base(&[("100.00", "")]),
        json!({"principal_work_by_others": "100.01"}),
    );
    assert_eq!(code(&over), "psi_test.others_exceed_psi");
    let at = with(
        base(&[("100.00", "")]),
        json!({"principal_work_by_others": "100.00"}),
    );
    assert_eq!(decided(&run(&at)).0, "subsection 87-15(3)");
}

/// Each share is rounded down to 2 places, so a share under a threshold
/// never shows as reaching it.
#[test]
fn shares_round_down() {
    let out = run(&base(&[("2.00", ""), ("1.00", "")]));
    assert_eq!(out["tests"]["one_source_percent"], "66.66");
}

/// s 87-25: 19.99% of principal work by others is not enough, but an
/// apprentice for half the year is.
#[test]
fn the_employment_test_has_2_limbs() {
    let two = [("60.00", ""), ("40.00", "")];
    let under = with(base(&two), json!({"principal_work_by_others": "19.99"}));
    let out = run(&under);
    assert_eq!(decided(&out), ("subsection 87-15(1)", S87_15_1));
    assert_eq!(out["tests"]["employment_percent"], "19.99");
    let apprentice = with(base(&two), json!({"apprentice": true}));
    assert_eq!(decided(&run(&apprentice)), ("section 87-25", PSB));
}

/// s 87-30: all 5 answers meet the business premises test, and the
/// outcome names every test met.
#[test]
fn the_outcome_names_each_test_met() {
    let two = [("60.00", "o"), ("40.00", "o")];
    let every = premises(&[
        "at_all_times",
        "mainly_for_psi",
        "exclusive_use",
        "separate_from_private",
        "separate_from_clients",
    ]);
    let alone = with(
        base(&[("60.00", ""), ("40.00", "")]),
        json!({"premises": every}),
    );
    assert_eq!(decided(&run(&alone)), ("section 87-30", PSB));
    let pair = with(base(&two), json!({"apprentice": true}));
    assert_eq!(decided(&run(&pair)), ("sections 87-20 and 87-25", PSB));
    let three = with(pair, json!({"premises": every}));
    assert_eq!(
        decided(&run(&three)),
        ("sections 87-20, 87-25 and 87-30", PSB)
    );
}

/// An entity's results test is s 87-18(3) and a failed test sends it to
/// Division 86.
#[test]
fn an_entity_falls_under_division_86() {
    let out = run(&with(base(&[("100.00", "")]), json!({"earner": "entity"})));
    assert_eq!(out["rule"], "subsection 87-15(3)");
    assert!(
        out["effect"]
            .as_str()
            .unwrap()
            .ends_with("Division 86 applies")
    );
    let step = &out["steps"].as_array().unwrap()[2]["step"];
    assert!(step.as_str().unwrap().starts_with("Subsection 87-18(3)"));
}

/// s 87-40: under 75% commission the principal stays one payer, at 74.99%
/// too; payments that cannot be right are refused.
#[test]
fn an_agent_under_75_percent_keeps_the_principal() {
    let agent = |commission: &str, principal: &str| {
        with(
            base(&[("100000.00", "")]),
            json!({"agent": true, "commission": commission, "principal_payments": principal}),
        )
    };
    let out = run(&agent("74990.00", "100000.00"));
    assert_eq!(out["rule"], "subsection 87-15(3)");
    let step = out["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["step"].as_str().unwrap().starts_with("Paragraph 87-40"))
        .unwrap();
    assert_eq!(
        step["finding"],
        "74.99%, under 75%: the agent rules do not apply"
    );
    assert_eq!(
        code(&agent("0.00", "0.00")),
        "psi_test.agent_payments_invalid"
    );
    assert_eq!(
        code(&agent("100.01", "100.00")),
        "psi_test.agent_payments_invalid"
    );
}

/// The amounts appear once the PSI is measured, and the determination is
/// never decided.
#[test]
fn amounts_and_the_determination() {
    let out = run(&with(
        base(&[("156000.00", "")]),
        json!({"principal_work_by_others": "36000.00"}),
    ));
    assert_eq!(out["amounts"]["personal_services_income"], "156000.00");
    assert_eq!(out["amounts"]["principal_work_by_others"], "36000.00");
    assert_eq!(out["personal_services_business"], false);
    assert_eq!(out["determination"]["decided"], false);
    let early = run(&with(
        base(&[("100.00", "")]),
        json!({"mainly_personal_efforts": false}),
    ));
    assert!(early["amounts"].as_object().unwrap().is_empty());
    assert_eq!(early["personal_services_business"], Value::Null);
    assert_eq!(early["tests"], json!({}));
}

/// Part 2-42 reads the same from 2020-21 to 2026-27; other years are
/// refused.
#[test]
fn years() {
    for year in [
        "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
    ] {
        let out = run(&with(base(&[("18000.00", "rtd")]), json!({"year": year})));
        assert_eq!(out["rule"], "subsection 87-18(1)", "{year}");
    }
    for year in ["2019-20", "2027-28"] {
        let input = with(base(&[("18000.00", "rtd")]), json!({"year": year}));
        assert_eq!(code(&input), "period.unsupported", "{year}");
    }
}

#[test]
fn refusals() {
    let one = [("100.00", "")];
    let cases = [
        (json!({"earner": "partner"}), "psi_test.earner_invalid"),
        (json!({"payers": []}), "psi_test.payers_invalid"),
        (
            json!({"payers": {"amount": "1.00"}}),
            "psi_test.payers_invalid",
        ),
        (json!({"payers": ["1.00"]}), "psi_test.payers_invalid"),
        (
            json!({"payers": vec![payer("1.00", ""); 51]}),
            "psi_test.payers_invalid",
        ),
        (
            json!({"payers": [{"amount": "1.00", "associate": false, "offer": false, "result": false, "tools": false}]}),
            "input.missing_field",
        ),
        (
            json!({"payers": [{"amount": "1.00", "associate": false, "offer": false, "result": false, "tools": false, "defects": false, "payer_name": true}]}),
            "input.field_unknown",
        ),
        (
            json!({"payers": [{"amount": "1.00", "associate": "no", "offer": false, "result": false, "tools": false, "defects": false}]}),
            "input.flag_invalid",
        ),
        (json!({"premises": true}), "psi_test.premises_invalid"),
        (
            json!({"premises": {"at_all_times": true}}),
            "input.missing_field",
        ),
        (json!({"commission": "1.00"}), "input.field_unknown"),
        (
            json!({"agent": true, "principal_payments": "1.00"}),
            "input.missing_field",
        ),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
    ];
    for (change, want) in cases {
        assert_eq!(code(&with(base(&one), change.clone())), want, "{change}");
    }
    assert_eq!(
        api::evaluate_coded(
            "psi-test",
            &with(base(&one), json!({"apprentice": "remove"}))
        )
        .unwrap_err()
        .code,
        "input.missing_field"
    );
}

/// One payer whose contracts differ is entered as one row per contract with
/// one source label: the results test counts each contract and the 80% rule
/// and the unrelated clients test count the payer once.
#[test]
fn a_payers_contracts_share_one_source() {
    let mut rows = vec![payer("80.00", "rtd"), payer("20.00", "")];
    for row in &mut rows {
        row["source"] = json!("Acme and associates");
    }
    let out = run(&with(base(&[]), json!({"payers": rows})));
    assert_eq!(decided(&out), ("subsection 87-18(1)", PSB));
    assert_eq!(out["tests"]["results_test_percent"], "80.00");
    rows[0]["amount"] = json!("74.00");
    rows[1]["amount"] = json!("26.00");
    let out = run(&with(base(&[]), json!({"payers": rows})));
    assert_eq!(decided(&out), ("subsection 87-15(3)", S87_15_3));
    assert_eq!(out["tests"]["one_source_percent"], "100.00");
    let mut offers = vec![
        payer("50.00", "o"),
        payer("30.00", "o"),
        payer("20.00", "o"),
    ];
    offers[0]["source"] = json!("A");
    offers[1]["source"] = json!("A");
    let out = run(&with(base(&[]), json!({"payers": offers})));
    assert_eq!(out["tests"]["unrelated_clients"], 2);
    assert_eq!(out["tests"]["one_source_percent"], "80.00");
    assert_eq!(out["rule"], "subsection 87-15(3)");
    let mut wide = payer("1.00", "");
    wide["source"] = json!("é".repeat(100));
    assert!(api::evaluate("psi-test", &with(base(&[]), json!({"payers": [wide]}))).is_ok());
    for bad in [
        json!(""),
        json!(1),
        json!("x".repeat(101)),
        json!("é".repeat(101)),
    ] {
        let mut row = payer("1.00", "");
        row["source"] = bad;
        let input = with(base(&[]), json!({"payers": [row]}));
        assert_eq!(code(&input), "psi_test.payers_invalid");
    }
}

/// s 87-20(1)(a) counts entities that are not associates of each other. 2
/// beneficiaries of one trust are each the trustee's associate, so s 87-15(3)
/// takes them with the trustee as one source, yet they need not be associates
/// of each other: given their own client labels, they are 2 unrelated clients.
#[test]
fn payers_sharing_a_source_can_be_unrelated_clients() {
    let mut rows = vec![payer("40.00", "o"), payer("39.00", "o"), payer("21.00", "")];
    for row in &mut rows[..2] {
        row["source"] = json!("Trustee of the T trust and its associates");
    }
    let out = run(&with(base(&[]), json!({"payers": rows})));
    assert_eq!(out["tests"]["one_source_percent"], "79.00");
    assert_eq!(out["tests"]["unrelated_clients"], 1);
    assert_eq!(decided(&out), ("subsection 87-15(1)", S87_15_1));
    rows[0]["client"] = json!("Beneficiary B");
    rows[1]["client"] = json!("Beneficiary C");
    let out = run(&with(base(&[]), json!({"payers": rows})));
    assert_eq!(out["tests"]["one_source_percent"], "79.00");
    assert_eq!(out["tests"]["unrelated_clients"], 2);
    assert_eq!(decided(&out), ("section 87-20", PSB));
    // A client label is its own name space: it never merges with a source.
    rows[1]["client"] = json!("Trustee of the T trust and its associates");
    let out = run(&with(base(&[]), json!({"payers": rows})));
    assert_eq!(out["tests"]["unrelated_clients"], 2);
    // Associates of each other share a client and count once.
    rows[1]["client"] = json!("Beneficiary B");
    let out = run(&with(base(&[]), json!({"payers": rows})));
    assert_eq!(out["tests"]["unrelated_clients"], 1);
    for bad in [json!(""), json!(true), json!("x".repeat(101))] {
        let mut row = payer("1.00", "o");
        row["client"] = bad;
        let input = with(base(&[]), json!({"payers": [row]}));
        assert_eq!(code(&input), "psi_test.payers_invalid");
    }
}

/// s 85-35(2): a religious practitioner's payment, apart from an employee's
/// (s 85-35(1)).
#[test]
fn a_religious_practitioner_is_under_subsection_85_35_2() {
    let out = run(&with(
        base(&[("100.00", "")]),
        json!({"earner": "religious_practitioner"}),
    ));
    assert_eq!(out["rule"], "subsection 85-35(2)");
    assert_eq!(out["personal_services_business"], Value::Null);
    let employee = run(&with(
        base(&[("100.00", "")]),
        json!({"earner": "employee"}),
    ));
    assert_eq!(employee["rule"], "subsection 85-35(1)");
}

/// An agent's payments are checked whenever an agent is entered; the 75%
/// refusal waits for the 80% rule, as the results test settles it first.
#[test]
fn an_agents_payments_are_checked_before_any_outcome() {
    let agent = |commission: &str| {
        with(
            base(&[("100.00", "rtd")]),
            json!({"agent": true, "commission": commission, "principal_payments": "100.00"}),
        )
    };
    assert_eq!(code(&agent("100.01")), "psi_test.agent_payments_invalid");
    let determined = with(agent("100.01"), json!({"determination_in_force": true}));
    assert_eq!(code(&determined), "psi_test.agent_payments_invalid");
    assert_eq!(decided(&run(&agent("75.00"))), ("subsection 87-18(1)", PSB));
}

/// The steps appear in the page's order, stopping where the outcome is
/// settled.
#[test]
fn the_steps_follow_the_page() {
    let labels = |input: &Value| -> Vec<String> {
        run(input)["steps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                s["step"]
                    .as_str()
                    .unwrap()
                    .split(':')
                    .next()
                    .unwrap()
                    .to_owned()
            })
            .collect()
    };
    let eighty = with(
        base(&[("80.00", "o"), ("20.00", "o")]),
        json!({"agent": true, "commission": "10.00", "principal_payments": "100.00"}),
    );
    assert_eq!(
        labels(&eighty),
        [
            "Subsection 84-5(1)",
            "Paragraph 87-15(1)(a)",
            "Subsection 87-18(1)",
            "Paragraph 87-40(2)(c)",
            "Subsection 87-15(3)",
            "Section 87-20",
            "Section 87-25",
            "Section 87-30",
        ]
    );
    let early = with(base(&[("100.00", "rtd")]), json!({}));
    assert_eq!(
        labels(&early),
        [
            "Subsection 84-5(1)",
            "Paragraph 87-15(1)(a)",
            "Subsection 87-18(1)"
        ]
    );
}
