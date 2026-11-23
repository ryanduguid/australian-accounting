// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! PCG 2021/4 against the site's risk scorer and allocation calculator
//! (ryanduguid.github.io at dcc7ecbe: the 2 pages' 16 cases, the 96 vectors
//! of their tests and a seeded sweep of 320, run through the pages' own
//! `pcgRisk` and `pcgAllocation` into `data/pcg_2021_4_site_cases.json`),
//! and against the guideline's case studies and examples. PCG 2021/4 as
//! updated on 19 June 2024 was read on 24 November 2026
//! (docs/calculation-evidence.md#pcg-20214). Every arrangement is fabricated
//! or the guideline's own.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Result<Value, &'static str> {
    api::evaluate_coded("pcg-2021-4", input).map_err(|e| e.code)
}

fn scorer(entitlement: &str, returned: &str, tax: &str) -> Value {
    json!({
        "year": "2026-27", "scope_confirmed": true, "gateway_1": true, "gateway_2": true,
        "factor_3": false, "entitlement": entitlement, "returned": returned, "tax": tax,
    })
}

fn person(kind: &str, firm: &str) -> Value {
    json!({"kind": kind, "firm_income": firm, "firm_deductions": "0", "other_income": "0"})
}

fn allocation(year: &str, recipients: Value) -> Value {
    json!({
        "year": year, "scope_confirmed": true, "gateway_1": true, "gateway_2": true,
        "factor_3": false, "recipients": recipients,
    })
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

/// A percentage as the site gives it (a number) and as the command gives it.
fn same_percent(site: &Value, got: &Value) -> bool {
    match (site.as_f64(), got.as_str()) {
        (None, None) => site.is_null() && got.is_null(),
        (Some(s), Some(g)) => format!("{s:.2}") == g,
        _ => false,
    }
}

fn cents(amount: &Value) -> String {
    format!("{:.2}", amount.as_f64().unwrap())
}

/// The site's figures on one row against the command's.
fn agrees(site: &Value, got: &Value) -> bool {
    let (p, s) = (&got["percentages"], &got["scores"]);
    let mut same = same_percent(&site["returned"], &p["returned"])
        && same_percent(&site["taxRate"], &p["effective_tax_rate"])
        && same_percent(&site["benchmarkShare"], &p["benchmark_share"])
        && site["factor1"] == s["factor_1"]
        && site["factor2"] == s["factor_2"]
        && site["factor3"] == s["factor_3"]
        && site["total"] == s["aggregate"]
        && site["zone"] == got["zone"];
    if let Some(recipients) = site["recipients"].as_array() {
        same &= cents(&site["firmIncome"]) == got["amounts"]["firm_income"]
            && cents(&site["tax"]) == got["amounts"]["tax"];
        for (want, have) in recipients.iter().zip(got["recipients"].as_array().unwrap()) {
            same &= cents(&want["tax"]) == have["tax"] && want["rule"] == have["rule"];
        }
        same &= recipients.len() == got["recipients"].as_array().unwrap().len();
    }
    same
}

fn site_input(row: &Value) -> Value {
    let mut input = row["input"].clone();
    input["scope_confirmed"] = json!(true);
    if input.get("year").is_none() {
        input["year"] = json!("2026-27");
    }
    // The site's form passes a missing factor 3 answer as undefined.
    if input.get("factor_3").is_none() {
        input["factor_3"] = Value::Null;
    }
    input
}

/// Every row of the site's fixture: the same figures, or refused alike.
#[test]
fn the_site_cases_agree() {
    let data: Value =
        serde_json::from_str(include_str!("data/pcg_2021_4_site_cases.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    let (mut figures, mut refusals) = (0, 0);
    for row in rows {
        let got = run(&site_input(row));
        match (&row["site"]["refused"], got) {
            (Value::String(_), Err(_)) => refusals += 1,
            (Value::Null, Ok(out)) => {
                assert!(agrees(&row["site"], &out), "{row}\n{out}");
                figures += 1;
            }
            (_, got) => panic!("{row}\n{got:?}"),
        }
    }
    eprintln!(
        "pcg-2021-4 site parity: {} rows, {figures} with the same figures, {refusals} refused alike",
        rows.len()
    );
    assert_eq!(rows.len(), 432);
    assert_eq!(
        [data["page_rows"].clone(), data["test_rows"].clone()],
        [json!(16), json!(96)]
    );
    assert_eq!(refusals, 25);
}

/// Case study 1 (Table 4): Nicolas returns $330,000 of $600,000 and his
/// company pays $70,200 at 26%: 55%, 31.56%, scores 4 and 3, green at 7.
#[test]
fn case_study_1_by_allocation_and_by_totals() {
    let list = json!([
        person("individual", "330000"),
        person("base_rate_company", "270000")
    ]);
    let out = run(&allocation("2020-21", list)).unwrap();
    assert_eq!(out["recipients"][0]["tax"], "119167.00");
    assert_eq!(out["recipients"][1]["tax"], "70200.00");
    assert_eq!(out["amounts"]["tax"], "189367.00");
    assert_eq!(out["percentages"]["effective_tax_rate"], "31.56");
    assert_eq!(
        out["scores"],
        json!({"factor_1": 4, "factor_2": 3, "factor_3": null, "aggregate": 7})
    );
    assert_eq!([&out["zone"], &out["risk_level"]], ["green", "low"]);
    let stated = run(&scorer("600000", "330000", "189367")).unwrap();
    assert_eq!(stated["scores"], out["scores"]);
    assert!(
        stated["steps"][0]["authority"]
            .as_str()
            .unwrap()
            .contains("paragraphs 76, 82 and 83")
    );
}

/// Example 4 (paragraphs 87 to 93): associate 2's $33,850 under 85(b), the
/// others' $22,967 under 85(a); 33.31% overall.
#[test]
fn example_4_counts_the_larger_limb() {
    let other = |firm: &str, other: &str| json!({"kind": "individual", "firm_income": firm, "firm_deductions": "0", "other_income": other});
    let list = json!([
        person("individual", "600000"),
        person("individual", "100000"),
        other("100000", "50000"),
        other("100000", "-50000"),
        other("100000", "-100000"),
        person("individual", "100000"),
    ]);
    let out = run(&allocation("2020-21", list)).unwrap();
    let taxes: Vec<_> = out["recipients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["tax"].clone(), r["rule"].clone()))
        .collect();
    assert_eq!(taxes[0], (json!("240667.00"), json!("85(a)")));
    assert_eq!(taxes[2], (json!("33850.00"), json!("85(b)")));
    assert_eq!(taxes[3], (json!("22967.00"), json!("85(a)")));
    assert_eq!(out["amounts"]["tax"], "366385.00");
    assert_eq!(out["percentages"]["effective_tax_rate"], "33.31");
}

/// Example 5 (paragraph 94): $600,000 less a $40,000 firm-related deduction,
/// with $50,000 other income: $238,450 under 85(b).
#[test]
fn example_5_firm_related_deductions() {
    let ipp = json!({"kind": "individual", "firm_income": "600000", "firm_deductions": "40000", "other_income": "50000"});
    let out = run(&allocation("2020-21", json!([ipp]))).unwrap();
    assert_eq!(
        out["recipients"][0],
        json!({"tax": "238450.00", "rule": "85(b)", "authority": out["recipients"][0]["authority"]})
    );
    // The IPP returns the whole entitlement: green with no other factor (paragraph 81).
    assert_eq!(out["scores"]["aggregate"], Value::Null);
    assert_eq!(out["zone"], "green");
}

/// Case study 7 (Table 10): $360,000 against a $350,000 benchmark, 102.86%,
/// factor 3 scores 3 and the aggregate of 10 is green on all 3.
#[test]
fn case_study_7_scores_factor_3() {
    let list = json!([
        person("individual", "360000"),
        person("individual", "180000"),
        person("base_rate_company", "60000")
    ]);
    let input = with(
        allocation("2020-21", list),
        json!({"factor_3": true, "benchmark": "350000"}),
    );
    let out = run(&input).unwrap();
    assert_eq!(out["percentages"]["benchmark_share"], "102.86");
    assert_eq!(
        out["scores"],
        json!({"factor_1": 4, "factor_2": 3, "factor_3": 3, "aggregate": 10})
    );
    assert_eq!(out["zone"], "green");
    let stated = with(
        scorer("600000", "360000", "199934"),
        json!({"factor_3": true, "remuneration": "360000", "benchmark": "350000"}),
    );
    assert_eq!(run(&stated).unwrap()["scores"], out["scores"]);
}

/// Table 1's inclusive edges: exactly 50% returned scores 4 and exactly
/// 30% tax scores 3; a cent less drops each a band, to red at 9.
#[test]
fn table_1_edges() {
    let at = run(&scorer("100", "50", "30")).unwrap();
    assert_eq!(at["scores"]["aggregate"], 7);
    let below = run(&scorer("100", "49.99", "29.99")).unwrap();
    assert_eq!(
        below["scores"],
        json!({"factor_1": 5, "factor_2": 4, "factor_3": null, "aggregate": 9})
    );
    assert_eq!([&below["zone"], &below["risk_level"]], ["red", "high"]);
    let amber = run(&scorer("100", "55", "28")).unwrap();
    assert_eq!(
        [&amber["zone"], &amber["risk_level"]],
        ["amber", "moderate"]
    );
}

/// 2026-27's 15% band and a company outside the base rate at 30%.
#[test]
fn the_2026_27_rates() {
    let list = json!([person("individual", "45001"), person("company", "100000")]);
    let out = run(&allocation("2026-27", list)).unwrap();
    assert_eq!(out["recipients"][0]["tax"], "4020.30");
    assert_eq!(out["recipients"][1]["tax"], "30000.00");
}

fn refused(cases: &[(Value, &str)]) {
    for (input, want) in cases {
        assert_eq!(run(input).unwrap_err(), *want, "{input}");
    }
}

#[test]
fn refuses_what_the_guideline_does_not_score() {
    let base = scorer("1000", "500", "300");
    refused(&[
        (
            with(base.clone(), json!({"gateway_1": false})),
            "pcg_2021_4.gateway_failed",
        ),
        (
            with(base.clone(), json!({"gateway_2": false})),
            "pcg_2021_4.gateway_failed",
        ),
        (
            with(base.clone(), json!({"gateway_1": "yes"})),
            "input.flag_invalid",
        ),
        (
            with(base.clone(), json!({"year": "2021-22"})),
            "period.unsupported",
        ),
        (
            with(base.clone(), json!({"year": "2027-28"})),
            "period.unsupported",
        ),
        (
            with(
                base.clone(),
                json!({"entitlement": "0", "returned": "0", "tax": "0"}),
            ),
            "pcg_2021_4.entitlement_nil",
        ),
        (
            with(base.clone(), json!({"returned": "1000.01"})),
            "pcg_2021_4.returned_above_entitlement",
        ),
        (
            with(base.clone(), json!({"tax": "1000.01"})),
            "pcg_2021_4.tax_above_entitlement",
        ),
        (
            with(
                base.clone(),
                json!({"factor_3": true, "remuneration": "10", "benchmark": "0"}),
            ),
            "pcg_2021_4.benchmark_nil",
        ),
    ]);
}

/// Paragraph 81: an IPP that returns all of it scores nothing else, so
/// factor 3 then needs no benchmark or remuneration; short of that it does.
#[test]
fn factor_3_needs_no_benchmark_when_everything_is_returned() {
    let three = json!({"factor_3": true});
    let whole = with(scorer("65000", "65000", "11592"), three.clone());
    let alone = allocation("2020-21", json!([person("individual", "65000")]));
    for input in [whole, with(alone, three.clone())] {
        let out = run(&input).unwrap();
        assert_eq!(
            out["scores"],
            json!({"factor_1": 1, "factor_2": null, "factor_3": null, "aggregate": null})
        );
        assert_eq!(out["zone"], "green");
    }
    let short = scorer("65000", "64999", "11592");
    for given in [json!({"benchmark": "10"}), json!({"remuneration": "10"})] {
        let input = with(with(short.clone(), three.clone()), given);
        let refusal = api::evaluate_coded("pcg-2021-4", &input).unwrap_err();
        assert_eq!(refusal.code, "input.missing_field");
        assert!(refusal.message.contains("paragraph 81"));
    }
    let shared = json!([person("individual", "65000"), person("company", "1")]);
    let shared = with(allocation("2020-21", shared), three);
    assert_eq!(run(&shared), Err("input.missing_field"));
}

#[test]
fn refuses_amounts_and_inputs_out_of_place() {
    let base = scorer("1000", "500", "300");
    refused(&[
        (
            with(base.clone(), json!({"factor_3": true, "benchmark": "10"})),
            "input.missing_field",
        ),
        (
            with(base.clone(), json!({"benchmark": "10"})),
            "input.field_unknown",
        ),
        (
            with(base.clone(), json!({"remuneration": "10"})),
            "input.field_unknown",
        ),
        (
            with(base.clone(), json!({"tax": null})),
            "input.missing_field",
        ),
        (
            with(
                base.clone(),
                json!({"tax": "1000000000.01", "entitlement": "1000000000.01"}),
            ),
            "input.amount_invalid",
        ),
        (
            with(
                base.clone(),
                json!({"recipients": [person("individual", "1")]}),
            ),
            "input.field_unknown",
        ),
        (with(base, json!({"zone": "green"})), "input.field_unknown"),
    ]);
}

#[test]
fn refuses_recipients_it_cannot_tax() {
    let ipp = person("individual", "300000");
    let list = |l: Value| allocation("2020-21", l);
    refused(&[
        (
            list(json!([person("base_rate_company", "1")])),
            "pcg_2021_4.recipients_invalid",
        ),
        (
            list(json!([ipp, person("trust", "1")])),
            "pcg_2021_4.recipient_kind_unsupported",
        ),
        (
            list(
                json!([ipp, {"kind": 1, "firm_income": "1", "firm_deductions": "0", "other_income": "0"}]),
            ),
            "pcg_2021_4.recipient_kind_unsupported",
        ),
        (list(json!([])), "pcg_2021_4.recipients_invalid"),
        (
            list(json!([ipp, ipp, ipp, ipp, ipp, ipp, ipp])),
            "pcg_2021_4.recipients_invalid",
        ),
        (
            list(json!({"kind": "individual"})),
            "pcg_2021_4.recipients_invalid",
        ),
        (
            list(json!([ipp, "company"])),
            "pcg_2021_4.recipients_invalid",
        ),
    ]);
}

#[test]
fn refuses_recipients_amounts_out_of_bounds() {
    let list = |l: Value| allocation("2020-21", l);
    refused(&[
        (
            list(json!([
                person("individual", "600000000"),
                person("individual", "400000000.01")
            ])),
            "input.amount_invalid",
        ),
        (
            list(json!([person("individual", "1000000000.01")])),
            "input.amount_invalid",
        ),
        (
            list(
                json!([{"kind": "individual", "firm_income": "1", "firm_deductions": "1000000000.01", "other_income": "0"}]),
            ),
            "input.amount_invalid",
        ),
        (
            list(
                json!([{"kind": "individual", "firm_income": "1", "firm_deductions": "0", "other_income": "-1000000000.01"}]),
            ),
            "input.signed_amount_invalid",
        ),
        (
            list(
                json!([{"kind": "individual", "firm_income": "1", "firm_deductions": "0", "other_income": "1.001"}]),
            ),
            "input.signed_amount_invalid",
        ),
    ]);
}

#[test]
fn refuses_recipient_fields_out_of_place() {
    let ipp = person("individual", "300000");
    let list = |l: Value| allocation("2020-21", l);
    refused(&[
        (
            list(
                json!([{"kind": "individual", "firm_income": "1", "firm_deductions": "0", "other_income": 5}]),
            ),
            "input.signed_amount_invalid",
        ),
        (
            list(json!([{"kind": "individual", "firm_income": "1", "firm_deductions": "0"}])),
            "input.missing_field",
        ),
        (
            list(
                json!([{"kind": "individual", "firm_income": "1", "firm_deductions": "0", "other_income": "0", "age": "40"}]),
            ),
            "input.field_unknown",
        ),
        (
            list(json!([person("individual", "0")])),
            "pcg_2021_4.entitlement_nil",
        ),
        (
            with(list(json!([ipp])), json!({"entitlement": "1"})),
            "input.field_unknown",
        ),
        (
            with(
                list(json!([ipp])),
                json!({"factor_3": true, "benchmark": "1", "remuneration": "1"}),
            ),
            "input.field_unknown",
        ),
    ]);
}
