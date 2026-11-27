// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Behaviour tests covering tool dispatch, contracts, adapters and error boundaries.
use aus_accounting_mcp_core::{adapters, call, contracts};
use serde_json::json;

#[test]
fn call_rejects_unknown_tool_non_object_and_unsupported_arguments() {
    let unknown = call("unknown_tool", &json!({}));
    assert_eq!(unknown["isError"], true);
    assert_eq!(unknown["content"][0]["text"], "Unknown tool: unknown_tool");
    assert!(contracts::tool("unknown_tool").is_none());

    let non_object = call("refuse_div7a", &json!(123));
    assert_eq!(non_object["isError"], true);
    assert_eq!(
        non_object["content"][0]["text"],
        "arguments must be an object"
    );

    let extra = call("refuse_div7a", &json!({"z": 1, "a": 2}));
    assert_eq!(extra["isError"], true);
    assert_eq!(
        extra["content"][0]["text"],
        "refuse_div7a: unsupported arguments: a, z"
    );
}

#[test]
fn validate_output_rejects_unknown_tool_and_invalid_payload() {
    assert_eq!(
        contracts::validate_output("unknown_tool", &json!({})),
        Err("Unknown tool".into())
    );
    let invalid = contracts::validate_output("refuse_div7a", &json!({"ok": "bad"})).unwrap_err();
    assert!(invalid.starts_with("Invalid output for refuse_div7a:"));
}

#[test]
fn refuse_div7a_and_unmigrated_adapter_dispatch_match_contract() {
    let reply = call("refuse_div7a", &json!({}));
    assert_eq!(reply["isError"], false);
    assert_eq!(reply["structuredContent"]["ok"], false);
    assert_eq!(reply["structuredContent"]["available"], false);
    assert_eq!(reply["structuredContent"]["reviewed_engine"], true);
    assert_eq!(
        reply["structuredContent"]["code"],
        "ERR_POLICY_DIV7A_SCOPE_REFUSED"
    );
    assert!(
        reply["structuredContent"]["reason"]
            .as_str()
            .unwrap()
            .contains("s 109N")
    );

    let err = adapters::evaluate("list_ato_benchmark_industries", &json!({})).unwrap_err();
    assert_eq!(
        err,
        "list_ato_benchmark_industries: native migration is incomplete; use the existing Python server"
    );
}

#[test]
fn data_freshness_evaluates_valid_as_at_and_refuses_invalid_calendar_date() {
    let valid = call(
        "get_data_freshness",
        &json!({"as_at": "2026-08-01", "tool": "review_div7a_loan"}),
    );
    assert_eq!(valid["isError"], false);
    assert_eq!(valid["structuredContent"]["as_at"], "2026-08-01");
    let datasets = valid["structuredContent"]["datasets"].as_array().unwrap();
    assert!(!datasets.is_empty());
    for row in datasets {
        assert!(matches!(
            row["as_at_status"].as_str(),
            Some("INSIDE_RECORDED_PERIOD" | "OUTSIDE_RECORDED_PERIOD")
        ));
    }

    let invalid = call("get_data_freshness", &json!({"as_at": "2026-02-29"}));
    assert_eq!(invalid["isError"], true);
    assert_eq!(
        invalid["content"][0]["text"],
        "Error executing tool get_data_freshness: as_at must be a YYYY-MM-DD date; got '2026-02-29'"
    );
}

#[test]
fn freshness_report_on_propagates_dataset_errors_and_filters_tools() {
    let err =
        adapters::freshness::report_on(&json!({}), || Err("loader failed".into())).unwrap_err();
    assert_eq!(err, "loader failed");

    let out = adapters::freshness::report_on(&json!({"as_at": "2026-08-01", "tool": "t1"}), || {
        Ok(vec![
            json!({"tools": ["t1"], "effective_periods": "UNKNOWN"}),
            json!({"tools": ["t2"], "effective_periods": []}),
        ])
    })
    .unwrap();
    let rows = out["datasets"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["as_at_status"], "UNKNOWN");

    let unfiltered = call("get_data_freshness", &json!({}));
    assert_eq!(unfiltered["isError"], false);
    assert!(unfiltered["structuredContent"]["tool"].is_null());
}

#[test]
fn div7a_benchmark_rate_supports_full_detail_and_rejects_bad_detail() {
    let full = call(
        "get_div7a_benchmark_rate",
        &json!({"year_of_income": "2026-27", "response_detail": "full"}),
    );
    assert_eq!(full["isError"], false);
    assert_eq!(full["structuredContent"]["verdict"], "KNOWN");
    assert_eq!(full["structuredContent"]["benchmark_rate"], "0.0877");
    assert!(full["structuredContent"].get("provenance").is_some());
    assert!(full["structuredContent"].get("response_detail").is_none());

    let err = adapters::evaluate(
        "get_div7a_benchmark_rate",
        &json!({"year_of_income": "2026-27", "response_detail": "brief"}),
    )
    .unwrap_err();
    assert_eq!(err, "response_detail must be 'summary' or 'full'");
}

#[test]
fn div7a_loan_defaults_loan_id_and_refuses_invalid_numeric_fields() {
    let loan_args = json!({
        "year_of_income": "2026-27",
        "year_loan_made": "2025-26",
        "written_agreement": true,
        "terms_in_place_before_lodgment_day": true,
        "maximum_term_years": "7",
        "secured_by_registered_mortgage_over_real_property": false,
        "interest_rate_for_years_after_year_loan_made": "0.0877",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
        "remaining_term_years": "7",
        "payments_applied_during_the_year": "19715.97"
    });
    let ok = call("review_div7a_loan", &loan_args);
    assert_eq!(ok["isError"], false);
    assert_eq!(ok["structuredContent"]["gate"]["loan_id"], "mcp-div7a-1");

    let direct = adapters::evaluate("review_div7a_loan", &loan_args).unwrap();
    assert_eq!(direct["gate"]["loan_id"], "mcp-div7a-1");

    for field in [
        "remaining_term_years",
        "amalgamated_loan_unpaid_at_end_of_previous_year",
        "payments_applied_during_the_year",
    ] {
        let mut bad = loan_args.clone();
        bad[field] = json!("bad");
        let reply = call("review_div7a_loan", &bad);
        assert_eq!(reply["isError"], true);
        let text = reply["content"][0]["text"].as_str().unwrap();
        assert!(text.contains(field));
    }
}

#[test]
fn div7a_loan_full_detail_and_missing_gate_provenance_fallback() {
    let base = json!({
        "year_of_income": "2026-27",
        "year_loan_made": null,
        "written_agreement": true,
        "terms_in_place_before_lodgment_day": true,
        "maximum_term_years": "7",
        "secured_by_registered_mortgage_over_real_property": false,
        "interest_rate_for_years_after_year_loan_made": "0.0877",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
        "remaining_term_years": "7",
        "payments_applied_during_the_year": "19715.97"
    });
    let summary = call("review_div7a_loan", &base);
    assert_eq!(summary["isError"], false);
    assert_eq!(summary["structuredContent"]["gate"]["verdict"], "UNKNOWN");
    assert!(summary["structuredContent"]["source"]["verify_at"].is_string());

    let mut full_args = base;
    full_args["response_detail"] = json!("full");
    let full = call("review_div7a_loan", &full_args);
    assert_eq!(full["isError"], false);
    assert!(full["structuredContent"].get("response_detail").is_none());
}

#[test]
fn payday_refuses_ambiguous_unparseable_blank_and_overflow_dates() {
    for (args, expected) in [
        (
            json!({"qe_day": "01/02/2026", "sg_amount": "120.00", "as_at": "2026-08-01"}),
            "is ambiguous",
        ),
        (
            json!({"qe_day": "not-a-date", "sg_amount": "120.00", "as_at": "2026-08-01"}),
            "is not a date this tool reads",
        ),
        (
            json!({"qe_day": "   ", "sg_amount": "120.00", "as_at": "2026-08-01"}),
            "qe_day is required (YYYY-MM-DD)",
        ),
        (
            json!({"qe_day": "2026-08-06", "sg_amount": "120.00", "as_at": ""}),
            "as_at is required (YYYY-MM-DD)",
        ),
        (
            json!({"qe_day": "9999-12-31", "sg_amount": "120.00", "as_at": "9999-12-31"}),
            "too far in the future",
        ),
        (
            json!({"qe_day": "2026-08-06", "sg_amount": "bad", "as_at": "2026-08-20"}),
            "is not a decimal amount",
        ),
        (
            json!({"qe_day": "2026-08-06", "sg_amount": "120.00", "as_at": "2026-08-20", "matched_amount": "bad"}),
            "is not a decimal amount",
        ),
    ] {
        let reply = call("calc_payday_super_deadline", &args);
        assert_eq!(reply["isError"], true);
        let text = reply["content"][0]["text"].as_str().unwrap();
        assert!(text.contains(expected));
    }

    let day_first = call(
        "calc_payday_super_deadline",
        &json!({"qe_day": "20/08/2026", "sg_amount": "120.00", "as_at": "2026-09-01"}),
    );
    assert_eq!(day_first["isError"], false);
    assert_eq!(
        day_first["structuredContent"]["result"]["due"],
        "2026-08-31"
    );
}

#[test]
fn tax_worksheet_refuses_invalid_money_and_depreciation_decimals() {
    let bad_money = call(
        "calculate_tax_worksheet",
        &json!({
            "facts": {
                "kind": "gst",
                "scope_confirmed": true,
                "year": "2025-26",
                "amount": "bad",
                "gst_inclusive": true
            }
        }),
    );
    assert_eq!(bad_money["isError"], true);
    assert!(
        bad_money["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("is not a decimal amount")
    );

    let bad_life = call(
        "calculate_tax_worksheet",
        &json!({
            "facts": {
                "kind": "depreciation",
                "scope_confirmed": true,
                "year": "2025-26",
                "cost": "1000.00",
                "effective_life": "bad",
                "days": 100,
                "taxable_use": "1",
                "method": "prime_cost"
            }
        }),
    );
    assert_eq!(bad_life["isError"], true);
    assert!(
        bad_life["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("ConversionSyntax")
    );
}
