// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The served worksheet scope differs from the linked engine's only for a worksheet whose
//! engine periods the input schema refuses, and each of those periods is refused. A schema
//! change that opens the periods, or an engine change that drops a worksheet's wide words,
//! fails here instead of leaving a stale narrowing in `NARROWED_SCOPE`.
use aus_accounting_mcp::handler::AccountingServer;
use serde_json::{Value, json};

#[test]
fn a_narrowed_scope_stands_only_for_periods_the_schema_refuses() {
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
        assert!(
            !refused.is_empty(),
            "{kind}: the served scope is narrowed, but every engine period is advertised"
        );
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
    assert_eq!(narrowed, ["fbt", "payg_withholding"]);
}
