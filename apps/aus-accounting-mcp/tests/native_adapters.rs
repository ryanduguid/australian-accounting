use aus_accounting_mcp::handler::AccountingServer;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn difference(left: &Value, right: &Value, path: &str) -> Option<String> {
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return Some(format!(
                    "{path}: object keys {:?} != {:?}",
                    a.keys().collect::<Vec<_>>(),
                    b.keys().collect::<Vec<_>>()
                ));
            }
            for (key, value) in a {
                if let Some(diff) = difference(value, &b[key], &format!("{path}/{key}")) {
                    return Some(diff);
                }
            }
            None
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some(format!("{path}: array lengths differ"));
            }
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                if let Some(diff) = difference(a, b, &format!("{path}/{i}")) {
                    return Some(diff);
                }
            }
            None
        }
        _ if left == right => None,
        (Value::String(a), Value::String(b)) => {
            let at = a.chars().zip(b.chars()).take_while(|(a, b)| a == b).count();
            let context = |s: &str| {
                s.chars()
                    .skip(at.saturating_sub(25))
                    .take(160)
                    .collect::<String>()
            };
            Some(format!(
                "{path}: char {at}: {:?} != {:?}",
                context(a),
                context(b)
            ))
        }
        _ => Some(format!("{path}: {left} != {right}")),
    }
}

#[test]
fn frozen_effective_adapter_contracts() {
    let mut cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/adapter-contracts.json")).unwrap();
    assert_eq!(cases.len(), 935);
    let edges: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/validation-edges.json")).unwrap();
    assert_eq!(edges.len(), 39);
    cases.extend(edges);
    let edges: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/coverage-edges.json")).unwrap();
    assert_eq!(edges.len(), 22);
    cases.extend(edges);
    let followups: Value =
        serde_json::from_str(include_str!("fixtures/native/calvino-followups.json")).unwrap();
    let dates = followups["date_cases"].as_array().unwrap();
    assert_eq!(dates.len(), 9);
    cases.extend(dates.iter().cloned());
    let mut failures = vec![];
    for (index, case) in cases.iter().enumerate() {
        let name = case["request"]["name"].as_str().unwrap();
        let mut actual = AccountingServer.call(name, &case["request"]["arguments"]);
        let expected = if let Some(error) = case.get("error") {
            aus_accounting_mcp::contracts::error(error["message"].as_str().unwrap().into())
        } else {
            let mut expected = case["result"].clone();
            if expected["structuredContent"]
                .get("engine_version")
                .is_some()
            {
                expected["structuredContent"]["engine_version"] = json!("1.0.0");
            }
            // Already-reviewed native engine correction, commit 4915b94.
            if name == "calculate_tax_worksheet"
                && case["request"]["arguments"]["facts"]["kind"] == "study_loan_repayment"
            {
                expected["structuredContent"]["sources"][0] = json!(
                    "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates"
                );
                expected["structuredContent"]["source_checked"] = json!("2026-10-20");
            }
            if name == "build_payday_super_evidence_pack" {
                native_pack(&mut expected["structuredContent"]["files"]);
            }
            native_provenance(&mut expected["structuredContent"]);
            expected
        };
        // JSON order and indentation are incidental. Both channels retain the
        // complete payload. The exact identity/provenance allowances above and
        // below are documented in NATIVE.md; no figures are normalised.
        if actual["isError"] == false
            && !(name == "build_payday_super_evidence_pack"
                && case["request"]["arguments"]["response_detail"] == "compact")
        {
            let text: Value =
                serde_json::from_str(actual["content"][0]["text"].as_str().unwrap()).unwrap();
            assert_eq!(text, actual["structuredContent"]);
            actual["content"] = json!([]);
        }
        let mut expected = if name == "calculate_tax_worksheet" {
            moved_lump_sum_page(&expected)
        } else {
            expected
        };
        if expected["isError"] == false
            && !(name == "build_payday_super_evidence_pack"
                && case["request"]["arguments"]["response_detail"] == "compact")
        {
            expected["content"] = json!([]);
        }
        if let Some(diff) = difference(&actual, &expected, "") {
            failures.push(format!("case {index} {name}: {diff}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Reviewed native engine correction, afa19a4 (#474): the ATO moved this page.
/// Python 0.2.0 still cites the old address; no figure depends on it.
fn moved_lump_sum_page(value: &Value) -> Value {
    serde_json::from_str(&value.to_string().replace(
        "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/lump-sum-payment-in-arrears-tax-offsets",
        "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-you-must-declare/lump-sum-payment-in-arrears",
    ))
    .unwrap()
}

fn native_provenance(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if key == "rate_table_uris" {
                    for row in value.as_array_mut().unwrap() {
                        if row["uri"] == "div7aloan/data/benchmark_rates.csv" {
                            row["uri"] = json!("data/benchmark_rates.csv");
                        }
                    }
                } else {
                    native_provenance(value);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                native_provenance(item);
            }
        }
        Value::String(text) => {
            *text = text
                .replace("paydaysuper/data/gic_rates.json", "data/gic_rates.json")
                .replace(
                    "paydaysuper/data/business_days.json",
                    "data/business_days.json",
                )
        }
        _ => {}
    }
}

fn native_pack(files: &mut Value) {
    let Some(files) = files.as_object_mut() else {
        return;
    };
    let old_report = files["report.csv"].as_str().unwrap();
    let old_digest = payday_super_checker::rates::hex(&Sha256::digest(old_report.as_bytes()));
    for text in files.values_mut() {
        let mut new = text.as_str().unwrap().to_string();
        // 0.2.2 before the 2026-10-31 refreeze, 0.2.3 after it.
        for version in ["0.2.2", "0.2.3"] {
            new = new
                .replace(
                    &format!("payday-super-checker {version}"),
                    "payday-super-checker 1.0.0",
                )
                .replace(
                    &format!("\"engine_version\": \"{version}\""),
                    "\"engine_version\": \"1.0.0\"",
                );
        }
        *text = json!(new);
        native_provenance(text);
    }
    let digest = payday_super_checker::rates::hex(&Sha256::digest(
        files["report.csv"].as_str().unwrap().as_bytes(),
    ));
    for text in files.values_mut() {
        *text = json!(text.as_str().unwrap().replace(&old_digest, &digest));
    }
}

#[test]
fn frozen_resource_contracts() {
    let resources: Value =
        serde_json::from_str(include_str!("fixtures/native/resource-contracts.json")).unwrap();
    assert_eq!(resources.as_object().unwrap().len(), 6);
    for (uri, expected) in resources.as_object().unwrap() {
        let mut expected = expected.clone();
        if expected.get("engine_version").is_some() {
            expected["engine_version"] = json!("1.0.0");
        }
        if uri == "aus-accounting://component-versions" {
            expected["server"]["version"] = json!("1.0.0");
            for engine in expected["engines"].as_array_mut().unwrap() {
                engine["version"] = json!("1.0.0");
            }
        }
        if uri == "aus-accounting://scope" {
            let study = &mut expected["calculation_worksheets"]["study_loan_repayment"];
            let url = "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates";
            study["source"] = json!(url);
            study["sources"][0] = json!(url);
            study["source_checked"] = json!("2026-10-20");
            study["source_passage"] = json!(study["source_passage"].as_str().unwrap().replace(
                "repayment thresholds and rates",
                "minimum repayment income and rates"
            ));
            study["example_evidence"]["document"] =
                json!("ATO study and training loan minimum repayment income and rates");
            study["example_evidence"]["reviewed"] = json!("2026-10-13");
            study["example_evidence"]["checked"] = json!("2026-10-20");
        }
        let expected = moved_lump_sum_page(&expected);
        let actual = aus_accounting_mcp::resources::read(uri).unwrap();
        assert_eq!(difference(&actual, &expected, ""), None, "{uri}");
    }
}

/// The Python report at the oracle base, with only the identities the native
/// server replaces: its versions, table paths and the reviewed engine corrections.
fn native_freshness(result: &mut Value) {
    result["server"]["version"] = json!("1.0.0");
    for row in result["datasets"].as_array_mut().unwrap() {
        row["engine_version"] = json!("1.0.0");
        let provenance = &mut row["provenance"];
        if let Some(tables) = provenance.get_mut("tables") {
            for table in tables.as_array_mut().unwrap() {
                table["uri"] = json!("data/benchmark_rates.csv");
            }
        }
        if let Some(table) = provenance.get_mut("table") {
            table["uri"] = json!("data/gic_rates.json");
        }
        if row["dataset"] == "tax_worksheet:study_loan_repayment" {
            row["sources"][0] = json!(
                "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates"
            );
            row["source_checked"] = json!("2026-10-20");
            row["provenance"]["source_passage"] = json!(
                row["provenance"]["source_passage"]
                    .as_str()
                    .unwrap()
                    .replace(
                        "repayment thresholds and rates",
                        "minimum repayment income and rates"
                    )
            );
        }
    }
    *result = moved_lump_sum_page(result);
}

#[test]
fn frozen_freshness_contracts() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/freshness-contracts.json")).unwrap();
    assert_eq!(cases.len(), 31);
    for case in cases {
        let arguments = &case["request"]["arguments"];
        let actual = AccountingServer.call("get_data_freshness", arguments);
        let mut expected = case["result"].clone();
        if expected["isError"] == true {
            assert_eq!(actual["isError"], true, "{arguments}");
            assert_eq!(actual["content"], expected["content"], "{arguments}");
            continue;
        }
        native_freshness(&mut expected["structuredContent"]);
        assert_eq!(
            difference(
                &actual["structuredContent"],
                &expected["structuredContent"],
                ""
            ),
            None,
            "{arguments}"
        );
        let text: Value =
            serde_json::from_str(actual["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(text, actual["structuredContent"]);
    }
}

#[test]
fn frozen_prompt_contracts() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/prompt-contracts.json")).unwrap();
    assert_eq!(cases.len(), 34);
    for case in cases {
        let actual = aus_accounting_mcp::prompts::get(
            case["name"].as_str().unwrap(),
            case["arguments"].as_object(),
        );
        if let Some(error) = case.get("error") {
            assert_eq!(actual.unwrap_err(), error["message"].as_str().unwrap());
        } else {
            assert_eq!(
                difference(&actual.unwrap(), &case["result"], ""),
                None,
                "{}",
                case["name"]
            );
        }
    }
}
