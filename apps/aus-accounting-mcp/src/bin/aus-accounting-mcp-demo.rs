// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Fabricated proof through the same validated dispatcher as MCP tools/call.
use aus_accounting_mcp::handler::AccountingServer;
use serde_json::{Value, json};

const DEMO_COMMAND: &str = "cargo run --locked --bin aus-accounting-mcp-demo";

fn structured_result(name: &str, result: Value) -> Result<Value, String> {
    if result.get("isError") != Some(&Value::Bool(false)) {
        return Err(format!("{name} did not return a complete MCP tool result"));
    }
    match result.get("structuredContent") {
        Some(Value::Object(_)) => Ok(result["structuredContent"].clone()),
        _ => Err(format!("{name} did not return structured MCP content")),
    }
}

fn demo_payload() -> Result<Value, String> {
    let calls = [
        (
            "generate_synthetic_sbr_fixture",
            json!({
                "form_type": "BAS",
                "entity_name": "Example Firm Pty Ltd",
                "revenue_or_sales": "110000.00"
            }),
        ),
        (
            "review_div7a_loan",
            json!({
                "year_of_income": "2026-27",
                "year_loan_made": "2025-26",
                "written_agreement": true,
                "terms_in_place_before_lodgment_day": true,
                "maximum_term_years": "7",
                "secured_by_registered_mortgage_over_real_property": false,
                "interest_rate_for_years_after_year_loan_made": "0.0837",
                "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
                "remaining_term_years": "1",
                "payments_applied_during_the_year": "108770.00",
                "loan_id": "synthetic-loan-1"
            }),
        ),
    ];
    let calls = calls
        .into_iter()
        .map(|(name, arguments)| {
            let result = structured_result(name, AccountingServer.call(name, &arguments))?;
            Ok(json!({"tool": name, "arguments": arguments, "result": result}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"calls": calls}))
}

fn render_transcript(mut payload: Value) -> Result<String, serde_json::Error> {
    payload.sort_all_objects();
    Ok(format!(
        "$ {DEMO_COMMAND}\n{}\n",
        serde_json::to_string_pretty(&payload)?
    ))
}

fn main() {
    match demo_payload().and_then(|payload| render_transcript(payload).map_err(|e| e.to_string())) {
        Ok(transcript) => print!("{transcript}"),
        Err(error) => {
            eprintln!("aus-accounting-mcp-demo: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_incomplete_or_unstructured_results() {
        for result in [json!({}), json!({"isError": true, "structuredContent": {}})] {
            assert_eq!(
                structured_result("example", result).unwrap_err(),
                "example did not return a complete MCP tool result"
            );
        }
        for content in [Value::Null, json!([]), json!("text"), json!(42)] {
            assert_eq!(
                structured_result(
                    "example",
                    json!({"isError": false, "structuredContent": content})
                )
                .unwrap_err(),
                "example did not return structured MCP content"
            );
        }
    }

    #[test]
    fn transcript_sorts_nested_keys_and_ends_with_one_newline() {
        assert_eq!(
            render_transcript(json!({"z": 1, "a": [{"z": true, "a": "0.00"}]})).unwrap(),
            format!(
                "$ {DEMO_COMMAND}\n{{\n  \"a\": [\n    {{\n      \"a\": \"0.00\",\n      \"z\": true\n    }}\n  ],\n  \"z\": 1\n}}\n"
            )
        );
    }
}
