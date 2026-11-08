// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Relationships and negative observations from the original Python assertions.
use aus_accounting_mcp::{contracts, handler::AccountingServer};
use regex::Regex;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[test]
fn original_behaviour_record_is_complete_and_bound_to_its_reviewed_sources() {
    use sha2::{Digest, Sha256};
    let record: Value =
        serde_json::from_str(include_str!("fixtures/native/behaviour-coverage.json")).unwrap();
    let sources = record["original_sources"].as_object().unwrap();
    assert_eq!(sources.len(), 43);
    for (relative, expected) in sources {
        let path = std::path::Path::new(relative);
        assert!(
            path.starts_with("tests")
                && path
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_)))
        );
        let bytes =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        assert_eq!(
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            expected.as_str().unwrap(),
            "{relative}: re-review changed original assertions"
        );
    }
    let functions = record["functions"].as_array().unwrap();
    assert_eq!(functions.len(), 590);
    let mut identities = BTreeSet::new();
    let mut counts = std::collections::BTreeMap::new();
    for row in functions {
        assert!(identities.insert(row["original"].as_str().unwrap()));
        let source = row["original"].as_str().unwrap().split_once(':').unwrap().0;
        assert_eq!(row["source_sha256"], sources[source]);
        let status = row["status"].as_str().unwrap();
        assert!(record["status_meanings"].get(status).is_some());
        *counts.entry(status).or_insert(0) += 1;
        let proofs = row["proofs"].as_array().unwrap();
        assert!(!proofs.is_empty());
        for proof in proofs {
            assert!(record["proofs"].get(proof.as_str().unwrap()).is_some());
        }
    }
    assert_eq!(serde_json::to_value(counts).unwrap(), record["counts"]);
}

fn call(name: &str, arguments: Value) -> Value {
    let reply = AccountingServer.call(name, &arguments);
    assert_eq!(reply["isError"], false, "{name}: {reply}");
    reply["structuredContent"].clone()
}

fn money_paths(node: &Value, root: &Value, path: &str, found: &mut BTreeSet<String>) {
    if let Some(nodes) = node.as_array() {
        for node in nodes {
            money_paths(node, root, path, found);
        }
    } else if node.is_object() {
        if let Some(reference) = node["$ref"].as_str() {
            money_paths(
                root.pointer(reference.strip_prefix('#').unwrap()).unwrap(),
                root,
                path,
                found,
            );
        }
        if node["pattern"] == r"^\s*-?[0-9]+(?:\.[0-9]{1,2})?\s*$" {
            found.insert(path.into());
        }
        if let Some(properties) = node["properties"].as_object() {
            for (key, child) in properties {
                money_paths(child, root, &format!("{path}.{key}"), found);
            }
        }
        for key in ["anyOf", "oneOf", "allOf", "items"] {
            money_paths(&node[key], root, path, found);
        }
    }
}

#[test]
fn exactly_the_original_46_money_paths_publish_the_runtime_grammar() {
    let original: Value =
        serde_json::from_str(include_str!("fixtures/native/money-relations.json")).unwrap();
    let expected: BTreeSet<_> = original["money_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(expected.len(), 46);
    let mut actual = BTreeSet::new();
    for tool in contracts::CATALOGUE["tools"].as_array().unwrap() {
        let schema = &tool["inputSchema"];
        money_paths(schema, schema, tool["name"].as_str().unwrap(), &mut actual);
    }
    assert_eq!(actual, expected);
    let pattern = Regex::new(r"^\s*-?[0-9]+(?:\.[0-9]{1,2})?\s*$").unwrap();
    for raw in ["12.34", " -0 ", "0012.5"] {
        assert!(pattern.is_match(raw));
        assert_eq!(
            call(
                "generate_synthetic_sbr_fixture",
                json!({"form_type":"BAS", "entity_name":"Fabricated", "revenue_or_sales":raw})
            )["synthetic"],
            true
        );
    }
    for raw in ["1e3", "+12", "1_000", ".5", "12.", "0.001", "1,000"] {
        assert!(!pattern.is_match(raw), "{raw}");
        assert_eq!(
            AccountingServer.call(
                "generate_synthetic_sbr_fixture",
                &json!({"form_type":"BAS", "entity_name":"Fabricated", "revenue_or_sales":raw})
            )["isError"],
            true
        );
    }
}

fn valid(name: &str, arguments: Value) -> bool {
    jsonschema::draft202012::options()
        .build(&contracts::tool(name).unwrap()["inputSchema"])
        .unwrap()
        .is_valid(&arguments)
}

#[test]
fn draft_2020_12_reads_nullable_nested_money_and_non_money_ratios() {
    for (raw, expected) in [
        (" 12.30 ", true),
        ("12.30", true),
        ("+12", false),
        ("1e3", false),
        ("0.001", false),
        ("", false),
        ("1,000.00", false),
    ] {
        for field in ["turnover", "cost_of_sales"] {
            let mut arguments =
                json!({"industry":"Bakeries and hot bread shops","turnover":"850000.00"});
            arguments[field] = json!(raw);
            assert_eq!(
                valid("get_ato_benchmarks", arguments),
                expected,
                "{field}: {raw:?}"
            );
        }
    }
    assert!(valid(
        "get_ato_benchmarks",
        json!({"industry":"Bakeries and hot bread shops","turnover":"850000.00","cost_of_sales":null})
    ));
    let mut arguments = json!({"contributions":[{"employee_id":"fabricated-1","qe_day":"2026-08-06","sg_amount":"800.00","first_to_fund":false,"out_of_cycle":false,"db_interest":false}],"as_at":"2026-08-21"});
    assert!(valid(
        "review_payday_super_contributions",
        arguments.clone()
    ));
    arguments["contributions"][0]["sg_amount"] = json!("1e3");
    assert!(!valid("review_payday_super_contributions", arguments));
    let mut arguments = json!({"facts":{"kind":"gst","scope_confirmed":true,"year":"2025-26","amount":"1100.00","gst_inclusive":true}});
    assert!(valid("calculate_tax_worksheet", arguments.clone()));
    arguments["facts"]["amount"] = json!("+1100.00");
    assert!(!valid("calculate_tax_worksheet", arguments));
    assert!(valid(
        "calculate_tax_worksheet",
        json!({"facts":{"kind":"depreciation","scope_confirmed":true,"year":"2025-26","cost":"10000.00","effective_life":"5","days":365,"taxable_use":"0.375","method":"prime_cost"}})
    ));
}

#[test]
fn published_bounds_match_the_effective_callable_models() {
    fn bound(node: &Value, key: &str) -> u64 {
        if let Some(value) = node[key].as_u64() {
            return value;
        }
        if let Some(value) = node.get("schema") {
            return bound(value, key);
        }
        if let Some(nodes) = node["anyOf"].as_array() {
            return nodes.iter().find_map(|n| n[key].as_u64()).unwrap();
        }
        panic!("missing {key}: {node}")
    }
    let models: Value =
        serde_json::from_str(include_str!("../data/effective-inputs.v1.json")).unwrap();
    fn find_model<'a>(node: &'a Value, name: &str) -> Option<&'a Value> {
        if node["name"] == name {
            return Some(node);
        }
        match node {
            Value::Object(m) => m.values().find_map(|v| find_model(v, name)),
            Value::Array(a) => a.iter().find_map(|v| find_model(v, name)),
            _ => None,
        }
    }
    let contribution = find_model(&models, "ContributionInput").unwrap();
    let employee = bound(
        &contribution["schema"]["fields"]["employee_id"],
        "max_length",
    );
    let variants =
        &models["calculate_tax_worksheet"]["schema"]["fields"]["facts"]["schema"]["choices"];
    let money = bound(&variants["gst"]["schema"]["fields"]["amount"], "max_length");
    let ratio = bound(
        &variants["depreciation"]["schema"]["fields"]["taxable_use"],
        "max_length",
    );
    let mut fields = vec![
        ("calc_payday_super_deadline", "employee_id", employee),
        ("get_ato_benchmarks", "turnover", money),
    ];
    fields.extend(
        [
            ("calc_payday_super_deadline", "sg_amount"),
            ("calc_payday_super_deadline", "remitted_amount"),
            ("calc_payday_super_deadline", "matched_amount"),
            (
                "review_div7a_loan",
                "amalgamated_loan_unpaid_at_end_of_previous_year",
            ),
            ("review_div7a_loan", "payments_applied_during_the_year"),
            ("generate_synthetic_sbr_fixture", "revenue_or_sales"),
        ]
        .map(|(t, f)| (t, f, money)),
    );
    fields.extend(
        [
            "maximum_term_years",
            "remaining_term_years",
            "security_coverage_at_first_made",
            "interest_rate_for_years_after_year_loan_made",
        ]
        .map(|f| ("review_div7a_loan", f, ratio)),
    );
    assert_eq!(fields.len(), 12);
    for (tool, field, expected) in fields {
        assert_eq!(
            bound(
                &contracts::tool(tool).unwrap()["inputSchema"]["properties"][field],
                "maxLength"
            ),
            expected,
            "{tool}.{field}"
        );
    }
}

#[test]
fn pension_365_day_success_does_not_admit_366() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        let mut arguments = json!({"facts":{"kind":"pension_minimum","scope_confirmed":true,"year":year,"account_balance":"500000.00","age":66,"days":365}});
        call("calculate_tax_worksheet", arguments.clone());
        arguments["facts"]["days"] = json!(366);
        let result = AccountingServer.call("calculate_tax_worksheet", &arguments);
        assert_eq!(result["isError"], true, "{year}: {result}");
        assert!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("365")
        );
    }
}

#[test]
fn review_rounding_matches_the_actual_evidence_csv_to_the_cent() {
    let arguments = json!({"contributions":[{"employee_id":"SYN001","qe_day":"2026-09-10","sg_amount":"120.00","first_to_fund":false,"out_of_cycle":false,"db_interest":false}],"as_at":"2026-11-01"});
    let pack = call("build_payday_super_evidence_pack", arguments.clone());
    let csv = pack["files"]["report.csv"]
        .as_str()
        .unwrap()
        .trim_start_matches('\u{feff}');
    let mut reader = payday_super_checker::csv_io::DictReader::new(csv.as_bytes())
        .ok()
        .unwrap();
    let row = reader.next_row().ok().unwrap().unwrap();
    let field = |name| {
        let (_, cell) = row
            .iter()
            .find(|(key, _)| key.as_deref() == Some(name))
            .unwrap();
        let payday_super_checker::csv_io::Cell::Value(text) = cell else {
            panic!("missing CSV value")
        };
        text.as_str()
    };
    assert_eq!(
        (field("notional_earnings"), field("sgc_estimate_high")),
        ("1.56", "194.50")
    );
    let single = call(
        "calc_payday_super_deadline",
        json!({"qe_day":"2026-09-10","sg_amount":"120.00","as_at":"2026-11-01"}),
    );
    let group = call("review_payday_super_contributions", arguments);
    let cents = Regex::new(r"^[0-9]+\.[0-9]{2}$").unwrap();
    for result in [&single["result"], &group["results"][0]] {
        for (path, column) in [
            ("/notional_earnings", "notional_earnings"),
            ("/experimental_sgc_low", "sgc_estimate_low"),
            ("/experimental_sgc_high", "sgc_estimate_high"),
            ("/uplift/clean_history/vds_within_30d", "uplift_best_case"),
            ("/uplift/prior_history/no_vds", "uplift_worst_case"),
        ] {
            assert_eq!(result.pointer(path).unwrap(), field(column), "{path}");
        }
        for key in [
            "notional_earnings",
            "experimental_sgc_low",
            "experimental_sgc_high",
        ] {
            assert!(cents.is_match(result[key].as_str().unwrap()));
        }
        for scenario in result["uplift"].as_object().unwrap().values() {
            for value in scenario.as_object().unwrap().values() {
                assert!(cents.is_match(value.as_str().unwrap()));
            }
        }
    }
    let gic = payday_super_checker::rates::load_gic().unwrap();
    let queue: Value =
        serde_json::from_str(pack["files"]["exceptions.json"].as_str().unwrap()).unwrap();
    let entries = queue["manifest"]["rate_table_uris"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["sha256"], gic.source.unwrap().sha256);
}

#[test]
fn an_unassessed_gic_period_names_the_current_table_boundary() {
    let result = call(
        "calc_payday_super_deadline",
        json!({"qe_day":"2027-07-01","sg_amount":"120.00","as_at":"2027-08-01","received":null}),
    );
    let caveats = result["result"]["caveats"].as_array().unwrap();
    let caveat = caveats
        .iter()
        .map(|v| v.as_str().unwrap())
        .find(|s| s.contains("not assessed for this row"))
        .unwrap();
    assert!(
        caveat.contains(
            &payday_super_checker::rates::load_gic()
                .unwrap()
                .last_known()
                .isoformat()
        )
    );
}

#[test]
fn current_language_neutral_conformance_cases_execute_through_the_dispatcher() {
    let contract: Value = serde_json::from_str(include_str!("../conformance/cases.json")).unwrap();
    assert_eq!(contract["schema_version"], 2);
    let cases = contract["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    let mut ids = BTreeSet::new();
    for case in cases {
        assert!(ids.insert(case["id"].as_str().unwrap()));
        let name = case["tool"].as_str().unwrap();
        assert!(contracts::tool(name).is_some());
        assert_ne!(
            case.get("expect").is_some(),
            case.get("expect_error_contains").is_some()
        );
        let reply = AccountingServer.call(name, &case["arguments"]);
        if let Some(message) = case["expect_error_contains"].as_str() {
            assert_eq!(reply["isError"], true, "{}: {reply}", case["id"]);
            assert!(
                reply["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .contains(message),
                "{}: {reply}",
                case["id"]
            );
        } else {
            assert_eq!(reply["isError"], false, "{}: {reply}", case["id"]);
            for (pointer, expected) in case["expect"].as_object().unwrap() {
                assert!(pointer.starts_with('/'));
                assert_eq!(
                    reply["structuredContent"].pointer(pointer),
                    Some(expected),
                    "{} {pointer}",
                    case["id"]
                );
            }
        }
    }
    let document = json!({"a":[{"b~/c":1}]});
    assert_eq!(document.pointer("/a/0/b~0~1c"), Some(&json!(1)));
    assert!(document.pointer("/a/1").is_none());
    assert!(document.pointer("/x").is_none());
}

#[test]
fn labour_warning_declares_every_fact_and_the_engine_cancels_associates() {
    use ato_benchmark_compare::{decimal::Decimal, mapping::BUCKETS, ratios::compute};
    let mut totals: Vec<_> = BUCKETS
        .iter()
        .map(|(name, _)| (name.to_string(), Decimal::from_int(0)))
        .collect();
    let supplied: Vec<_> = totals.iter().map(|(name, _)| name.clone()).collect();
    fn set(totals: &mut [(String, Decimal)], name: &str, value: i64) {
        totals.iter_mut().find(|(key, _)| key == name).unwrap().1 = Decimal::from_int(value);
    }
    set(&mut totals, "turnover", 850000);
    set(&mut totals, "salary_wages", 1);
    let figures = compute(&totals, &supplied, Some(&Decimal::from_int(200000))).unwrap();
    let checks: Vec<_> = figures
        .warning_details
        .iter()
        .filter(|check| check.code == "w1_used_for_labour")
        .collect();
    assert_eq!(checks.len(), 1);
    let required: BTreeSet<_> = checks[0]
        .required_fields
        .iter()
        .map(String::as_str)
        .collect();
    assert_eq!(
        required,
        BTreeSet::from([
            "w1",
            "salary_wages",
            "cost_of_sales_labour",
            "associated_persons"
        ])
    );
    set(&mut totals, "salary_wages", 80000);
    set(&mut totals, "cost_of_sales_labour", 50000);
    for associates in [0, 50000] {
        set(&mut totals, "associated_persons", associates);
        assert_eq!(
            compute(&totals, &supplied, Some(&Decimal::from_int(50000)))
                .unwrap()
                .labour
                .to_string(),
            "130000"
        );
    }
    let withheld = call(
        "get_ato_benchmarks",
        json!({"industry":"Bakeries and hot bread shops","turnover":"850000.00","other_income":"0","salary_wages":"80000.00","cost_of_sales_labour":"50000.00","contractor_commission":"0","w1":"50000.00"}),
    );
    assert!(withheld["figures"]["labour"].is_null());
    assert_eq!(
        withheld["ratios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["ratio"] == "labour_to_turnover")
            .unwrap()["status"],
        "not_supplied"
    );
}

#[test]
fn div7a_summary_sizes_and_direct_year_parsing_preserve_original_contracts() {
    let original: Value = serde_json::from_str(
        include_str!("../docs/quick-proof.txt")
            .split_once('\n')
            .unwrap()
            .1,
    )
    .unwrap();
    let mut loan = original["calls"][1]["arguments"].clone();
    for (tool, mut arguments) in [
        ("review_div7a_loan", loan.take()),
        (
            "get_div7a_benchmark_rate",
            json!({"year_of_income":"2025-26"}),
        ),
    ] {
        arguments["response_detail"] = json!("summary");
        let summary = call(tool, arguments.clone());
        arguments["response_detail"] = json!("full");
        let full = call(tool, arguments);
        assert!(
            serde_json::to_vec(&summary).unwrap().len() * 2
                < serde_json::to_vec(&full).unwrap().len()
        );
        for key in ["engine", "engine_version", "law_content_date"] {
            assert_eq!(summary[key], full[key]);
        }
        if tool == "review_div7a_loan" {
            assert_eq!(summary["gate"]["verdict"], "COMPLYING");
            assert_eq!(
                summary["minimum_yearly_repayment"]["myr_required"],
                "108770.00"
            );
            assert!(summary["gate"].get("statutory_trace").is_none());
            assert!(full["gate"]["statutory_trace"].is_array());
        } else {
            assert_eq!(summary["benchmark_rate"], "0.0837");
            assert_eq!(
                summary["source"]["verify_at"],
                full["provenance"]["verify_at"]
            );
            assert!(summary.get("provenance").is_none());
            assert!(summary.get("statutory_trace").is_none());
        }
    }
    let error = aus_accounting_mcp::adapters::evaluate(
        "get_div7a_benchmark_rate",
        &json!({"year_of_income":"2026-2027"}),
    )
    .unwrap_err();
    assert!(error.contains("2026-27"), "{error}");
}

#[test]
fn lodgment_fact_descriptions_and_quoted_versions_match_linked_engines() {
    let data: Value = serde_json::from_str(include_str!(
        "../../../packages/lodgment-calendar/data/program-2026-27.json"
    ))
    .unwrap();
    // As in the Python server, the tool offers no fbt_return until the supported
    // distribution pins a lodgment-calendar release that has one.
    let meanings: serde_json::Map<_, _> = data["fact_meanings"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(name, _)| *name != "on_fbt_client_list_by_21_may")
        .map(|(name, meaning)| (name.clone(), meaning.clone()))
        .collect();
    let properties = contracts::tool("get_lodgment_due_dates").unwrap()["inputSchema"]["$defs"]["LodgmentFactsInput"]["properties"].as_object().unwrap();
    assert_eq!(
        properties.keys().collect::<BTreeSet<_>>(),
        meanings.keys().collect()
    );
    for (name, meaning) in &meanings {
        assert!(
            properties[name]["description"]
                .as_str()
                .unwrap()
                .starts_with(&format!("True if {}", meaning.as_str().unwrap()))
        );
    }
    for (name, arguments, version) in [
        (
            "get_ato_benchmarks",
            json!({"industry":"Bakeries and hot bread shops","turnover":"850000.00","cost_of_sales":"270000.00"}),
            ato_benchmark_compare::VERSION,
        ),
        (
            "get_div7a_benchmark_rate",
            json!({"year_of_income":"2025-26"}),
            div7a_loan_review::VERSION,
        ),
        (
            "calc_payday_super_deadline",
            json!({"qe_day":"2026-08-06","sg_amount":"800.00","as_at":"2026-08-21"}),
            payday_super_checker::VERSION,
        ),
    ] {
        assert_eq!(call(name, arguments)["engine_version"], version);
    }
}

#[test]
fn current_catalogue_pages_reconstruct_results_and_stop_at_the_end() {
    for search in [
        Value::Null,
        json!("shop"),
        json!("no-such-synthetic-industry"),
    ] {
        let complete = call(
            "list_ato_benchmark_industries",
            json!({"search":search,"year":"2023-24"}),
        );
        let mut collected = Vec::new();
        let mut offset = 0;
        loop {
            let page = call(
                "list_ato_benchmark_industries",
                json!({"search":search,"year":"2023-24","limit":7,"offset":offset}),
            );
            let industries = page["industries"].as_array().unwrap();
            assert_eq!(page["count"], industries.len());
            assert!(industries.len() <= 7);
            assert_eq!(page["total_count"], complete["count"]);
            assert_eq!(page["offset"], offset);
            for key in [
                "engine",
                "engine_version",
                "benchmark_year",
                "source",
                "total_business_types",
            ] {
                assert_eq!(page[key], complete[key]);
            }
            collected.extend(industries.iter().cloned());
            if page["has_more"] == false {
                assert!(page["next_offset"].is_null());
                break;
            }
            let next = page["next_offset"].as_u64().unwrap();
            assert_eq!(next, offset + industries.len() as u64);
            assert!(next > offset);
            offset = next;
        }
        assert_eq!(collected, *complete["industries"].as_array().unwrap());
    }
    let complete = call("list_ato_benchmark_industries", json!({"year":"2023-24"}));
    let all = complete["industries"].as_array().unwrap();
    assert!(!all.is_empty());
    for offset in [all.len() - 1, all.len(), all.len() + 1] {
        let page = call(
            "list_ato_benchmark_industries",
            json!({"year":"2023-24","limit":1,"offset":offset}),
        );
        assert_eq!(
            page["industries"],
            json!(if offset < all.len() {
                &all[offset..]
            } else {
                &[]
            })
        );
        assert_eq!(page["has_more"], false);
        assert!(page["next_offset"].is_null());
    }
    let remaining = call(
        "list_ato_benchmark_industries",
        json!({"year":"2023-24","limit":null,"offset":1}),
    );
    assert_eq!(remaining["industries"], json!(&all[1..]));
    assert_eq!(remaining["has_more"], false);
    assert!(remaining["next_offset"].is_null());
    let largest = call(
        "list_ato_benchmark_industries",
        json!({"year":"2023-24","limit":100}),
    );
    assert_eq!(largest["industries"], json!(&all[..all.len().min(100)]));
    assert!(
        contracts::validate_output(
            "generate_synthetic_sbr_fixture",
            &json!(["BAS_AU_ACTIVITY_STATEMENT"])
        )
        .is_err()
    );
}

#[test]
fn related_contributions_change_the_alignment_but_single_scope_stays_visible() {
    let row = json!({"employee_id":"synthetic","qe_day":"2027-07-08","sg_amount":"120.00","received":"2027-07-20","matched_amount":"120.00","first_to_fund":false,"out_of_cycle":false,"db_interest":false});
    let group = call(
        "review_payday_super_contributions",
        json!({"contributions":[{"employee_id":"synthetic","qe_day":"2027-07-01","sg_amount":"120.00","received":"2027-07-05","matched_amount":"120.00","first_to_fund":true,"out_of_cycle":false,"db_interest":false},row],"as_at":"2027-08-01"}),
    );
    let result = &group["results"][1];
    assert_eq!(result["verdict"], "ON_TIME");
    assert_eq!(result["pathway"], "ITEM4_ALIGNED");
    assert_eq!(result["due"], "2027-07-29");
    let mut arguments = row;
    arguments["as_at"] = json!("2027-08-01");
    let single = call("calc_payday_super_deadline", arguments);
    assert_eq!(single["result"]["verdict"], "LATE");
    assert_eq!(single["result"]["due"], "2027-07-19");
    assert_eq!(single["assessment_scope"], "single_contribution");
    let caveats = single["result"]["caveats"].to_string();
    assert!(caveats.contains("s 18C(2) item 4") && caveats.contains("related contributions"));
}
