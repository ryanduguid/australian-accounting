// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Negative schema and CLI contracts from the original application's tests.
use aus_accounting_mcp::{contracts, handler::AccountingServer};
use serde_json::{Value, json};
use std::process::{Command, Stdio};

fn loan() -> Value {
    json!({"year_of_income":"2026-27","year_loan_made":"2025-26","written_agreement":true,
        "terms_in_place_before_lodgment_day":true,"maximum_term_years":"7",
        "secured_by_registered_mortgage_over_real_property":false,
        "interest_rate_for_years_after_year_loan_made":"0.0837",
        "amalgamated_loan_unpaid_at_end_of_previous_year":"100000.00",
        "remaining_term_years":"1","payments_applied_during_the_year":"108770.00",
        "loan_id":"synthetic-schema-loan"})
}
fn payday() -> Value {
    json!({"qe_day":"2027-07-01","sg_amount":"120.00","as_at":"2027-08-01"})
}
fn payload(name: &str, arguments: &Value) -> Value {
    let response = AccountingServer.call(name, arguments);
    assert_eq!(response["isError"], false, "{name}: {response}");
    response["structuredContent"].clone()
}
fn rejects(name: &str, arguments: &Value, pointer: &str, replacement: Option<Value>) {
    let mut result = payload(name, arguments);
    if let Some(replacement) = replacement {
        *result
            .pointer_mut(pointer)
            .expect("original declared output field") = replacement;
    } else {
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        assert!(
            result
                .pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key)
                .is_some()
        );
    }
    assert!(
        contracts::validate_output(name, &result).is_err(),
        "accepted misleading {name} {pointer}: {result}"
    );
}
#[test]
fn declared_outputs_reject_misleading_money_dates_and_safety_markers() {
    rejects(
        "get_div7a_benchmark_rate",
        &json!({"year_of_income":"2027-28"}),
        "/benchmark_rate",
        Some(json!(0)),
    );
    rejects(
        "review_div7a_loan",
        &loan(),
        "/minimum_yearly_repayment/myr_required",
        Some(json!(108770)),
    );
    rejects(
        "calc_payday_super_deadline",
        &payday(),
        "/result/sg_amount",
        Some(json!(120)),
    );
    rejects(
        "generate_synthetic_sbr_fixture",
        &json!({"form_type":"BAS"}),
        "/synthetic",
        Some(json!(false)),
    );
    rejects("refuse_div7a", &json!({}), "/ok", Some(json!(true)));
    for amount in ["abc", "NaN"] {
        rejects(
            "calc_payday_super_deadline",
            &payday(),
            "/result/sg_amount",
            Some(json!(amount)),
        );
    }
    for date in ["not-a-date", "2027-13-01", "2027-02-30"] {
        rejects(
            "calc_payday_super_deadline",
            &payday(),
            "/as_at",
            Some(json!(date)),
        );
    }
    rejects(
        "calc_payday_super_deadline",
        &payday(),
        "/result/received",
        Some(json!("not-a-date")),
    );
    rejects(
        "get_div7a_benchmark_rate",
        &json!({"year_of_income":"2025-26"}),
        "/year_of_income",
        Some(json!("2027")),
    );
    for detail in ["summary", "full"] {
        rejects(
            "review_div7a_loan",
            &json!({"year_of_income":"2025-26","response_detail":detail}),
            "/gate/benchmark_year_used",
            Some(json!("not-a-year")),
        );
    }
}
#[test]
fn published_schemas_require_the_returned_contribution_and_registration_limits() {
    let base = json!({"employee_id":"SYN001","qe_day":"2026-08-06","sg_amount":"800.00","as_at":"2026-09-30"});
    let with = |facts: Value| {
        let mut arguments = base.clone();
        arguments
            .as_object_mut()
            .unwrap()
            .extend(facts.as_object().unwrap().clone());
        arguments
    };
    let on_time = with(json!({"received":"2026-08-10","matched_amount":"800.00"}));
    let candidate = with(json!({"received":"2026-08-10"}));
    let single = "calc_payday_super_deadline";
    assert_eq!(
        payload(single, &candidate)["result"]["horizon_verdicts"],
        json!(["UNPAID", "ON_TIME"])
    );
    for arguments in [&on_time, &candidate] {
        rejects(single, arguments, "/result/verdict_qualification", None);
    }
    rejects(
        single,
        &on_time,
        "/result/verdict_qualification/returned_contributions_assessed",
        Some(json!(true)),
    );
    let row = |facts: Value| {
        let mut row = with(facts);
        row.as_object_mut().unwrap().remove("as_at");
        row.as_object_mut().unwrap().extend(
            json!({"first_to_fund":false,"out_of_cycle":false,"db_interest":false})
                .as_object()
                .unwrap()
                .clone(),
        );
        row
    };
    let rows = json!({"contributions":[row(json!({"received":"2026-08-10","matched_amount":"800.00"}))],"as_at":"2026-09-30"});
    rejects(
        "review_payday_super_contributions",
        &rows,
        "/results/0/verdict_qualification",
        None,
    );
    let pack = "build_payday_super_evidence_pack";
    assert_eq!(payload(pack, &rows)["review_exit_code"], 0);
    rejects(pack, &rows, "/verdict_qualification", None);
    // An UNKNOWN row with no ON_TIME candidate needs no qualification.
    let unknown = json!({"employee_id":"SYN001","qe_day":"2031-07-01","sg_amount":"800.00","received":"2031-07-02","matched_amount":"400.00","as_at":"2032-01-01"});
    let result = payload(single, &unknown);
    assert_eq!(result["result"]["verdict"], "UNKNOWN");
    assert!(result["result"].get("verdict_qualification").is_none());
    for (name, argument, value) in [
        ("validate_abn", "abn", "51 824 753 556"),
        ("validate_acn", "acn", "004 085 616"),
    ] {
        let arguments = json!({argument: value});
        assert_eq!(payload(name, &arguments)["result"], "CHECK_DIGIT_VALID");
        for marker in [
            "/abr_registration_established",
            "/gst_registration_established",
        ] {
            rejects(name, &arguments, marker, Some(json!(true)));
        }
        rejects(name, &arguments, "/result", Some(json!("REGISTERED")));
    }
}
#[test]
fn variants_require_audit_sections_and_preserve_engine_extensions() {
    for (name, arguments, pointer) in [
        (
            "generate_synthetic_sbr_fixture",
            json!({"form_type":"CTR"}),
            "/income_statement",
        ),
        (
            "generate_synthetic_sbr_fixture",
            json!({"form_type":"BAS"}),
            "/gst_labels",
        ),
        (
            "get_div7a_benchmark_rate",
            json!({"year_of_income":"2025-26"}),
            "/source",
        ),
        (
            "get_div7a_benchmark_rate",
            json!({"year_of_income":"2025-26","response_detail":"full"}),
            "/provenance",
        ),
        (
            "get_div7a_benchmark_rate",
            json!({"year_of_income":"2025-26","response_detail":"full"}),
            "/statutory_trace",
        ),
        ("review_div7a_loan", loan(), "/source"),
        (
            "review_div7a_loan",
            {
                let mut args = loan();
                args["response_detail"] = json!("full");
                args
            },
            "/gate/limbs",
        ),
        (
            "review_div7a_loan",
            {
                let mut args = loan();
                args["response_detail"] = json!("full");
                args
            },
            "/minimum_yearly_repayment/statutory_trace",
        ),
    ] {
        rejects(name, &arguments, pointer, None);
    }
    for (form, wrong) in [("CTR", "gst_labels"), ("BAS", "reconciliation")] {
        let mut result = payload("generate_synthetic_sbr_fixture", &json!({"form_type":form}));
        result[wrong] = json!({});
        assert!(contracts::validate_output("generate_synthetic_sbr_fixture", &result).is_err());
    }
    for (name, arguments) in [
        (
            "get_div7a_benchmark_rate",
            json!({"year_of_income":"2025-26","response_detail":"full"}),
        ),
        ("review_div7a_loan", {
            let mut args = loan();
            args["response_detail"] = json!("full");
            args
        }),
        ("generate_synthetic_sbr_fixture", json!({"form_type":"CTR"})),
        ("generate_synthetic_sbr_fixture", json!({"form_type":"BAS"})),
    ] {
        let mut result = payload(name, &arguments);
        result["future_engine_audit"] = json!({"reference":"synthetic","missing":null});
        contracts::validate_output(name, &result).unwrap();
        let response = contracts::success(result.clone());
        assert_eq!(response["structuredContent"], result);
        assert_eq!(
            serde_json::from_str::<Value>(response["content"][0]["text"].as_str().unwrap())
                .unwrap(),
            result
        );
    }
}
#[test]
fn command_help_version_and_unknown_arguments_do_not_start_stdio() {
    for argument in ["--help", "-h", "--version", "--bogus"] {
        let output = Command::new(env!("CARGO_BIN_EXE_aus-accounting-mcp"))
            .arg(argument)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let text = String::from_utf8(output.stdout).unwrap();
        if argument == "--bogus" {
            assert_eq!(output.status.code(), Some(2));
        } else {
            assert!(output.status.success());
            if argument == "--version" {
                assert_eq!(text.trim(), "aus-accounting-mcp 1.0.0");
            } else {
                assert!(
                    text.contains("AUS_ACCOUNTING_LIBRARY_ROOT")
                        && text.contains("AUS_ACCOUNTING_CORPUS_ROOT")
                );
            }
        }
    }
}
