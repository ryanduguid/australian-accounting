// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The served worksheet scope differs from the linked engine's where the input schema
//! refuses a period or an input the engine's words describe. Each refused period is
//! refused by the tool. `contribution_caps` is the one narrowing without a refused
//! period: the schema takes one unused-cap total, not the year-by-year carry-forward.
//! A schema change that opens those periods or inputs, or an engine change that drops
//! a worksheet's wide words, fails here instead of leaving a stale narrowing.
use aus_accounting_mcp::handler::AccountingServer;
use serde_json::{Value, json};

#[test]
fn a_narrowed_scope_stands_for_what_the_schema_refuses() {
    let engine =
        australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let scope = aus_accounting_mcp::resources::read("aus-accounting://scope").unwrap();
    let served = scope["calculation_worksheets"].as_object().unwrap();
    let mut narrowed = vec![];
    for (kind, entry) in served {
        if entry["scope"] == engine[kind]["scope"] {
            continue;
        }
        narrowed.push(kind.as_str());
        let advertised = entry["supported_periods"].as_array().unwrap();
        let refused: Vec<&Value> = engine[kind]["supported_periods"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|period| !advertised.contains(period))
            .collect();
        if refused.is_empty() {
            assert_eq!(kind, "contribution_caps");
            continue;
        }
        for period in refused {
            let mut facts = entry["example"]["facts"].clone();
            let arguments = period["arguments"].as_object().unwrap().clone();
            facts.as_object_mut().unwrap().extend(arguments);
            let reply =
                AccountingServer.call("calculate_tax_worksheet", &json!({ "facts": facts }));
            assert_eq!(reply["isError"], true, "{kind}: {period} was not refused");
        }
    }
    narrowed.sort_unstable();
    assert_eq!(
        narrowed,
        [
            "contribution_caps",
            "depreciation",
            "fbt",
            "medicare_levy",
            "payg_withholding",
            "study_loan_withholding",
        ]
    );
}
