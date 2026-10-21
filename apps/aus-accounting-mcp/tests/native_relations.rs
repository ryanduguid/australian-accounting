//! Preserve original cross-source assertions that frozen payloads cannot prove.
use ato_benchmark_compare::dataset::{self, DataDir};
use aus_accounting_mcp::{contracts, handler::AccountingServer, prompts, resources};
use chrono::NaiveDate;
use payday_super_checker::{calendar, csv_io, rates};
use rmcp::ServerHandler;
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn normalise(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn length(tool: &str, argument: &str) -> usize {
    fn declared(schema: &Value) -> Option<u64> {
        schema["maxLength"]
            .as_u64()
            .or_else(|| schema["anyOf"].as_array()?.iter().find_map(declared))
    }
    declared(&contracts::tool(tool).unwrap()["inputSchema"]["properties"][argument])
        .expect("published string bound") as usize
}

#[test]
fn public_disclaimer_and_linked_worksheet_scopes_match_served_boundaries() {
    // test_resources.py: public disclaimer paragraphs and every engine boundary.
    let served = resources::read("aus-accounting://disclaimer").unwrap();
    let served = served.as_str().unwrap();
    let boundary = served
        .strip_prefix("# Boundary\n\n")
        .unwrap()
        .split_once("\n\n# ato-benchmark-compare")
        .unwrap()
        .0;
    let public = normalise(include_str!("../DISCLAIMER.md"));
    let paragraphs: Vec<_> = boundary.split("\n\n").collect();
    assert_eq!(paragraphs.len(), 6);
    for paragraph in paragraphs {
        assert!(public.contains(&normalise(paragraph)), "{paragraph}");
    }
    let scope = resources::read("aus-accounting://scope").unwrap();
    for (kind, entry) in scope["calculation_worksheets"].as_object().unwrap() {
        assert!(served.contains(entry["scope"].as_str().unwrap()), "{kind}");
    }
    for engine in resources::read("aus-accounting://component-versions").unwrap()["engines"]
        .as_array()
        .unwrap()
    {
        assert!(served.contains(engine["distribution"].as_str().unwrap()));
    }
}

#[test]
fn supported_tool_registry_and_standing_refusal_agree_with_scope() {
    // test_resources.py: scope distinguishes tools; refusal is the served text.
    let scope = resources::read("aus-accounting://scope").unwrap();
    let registered: BTreeSet<_> = contracts::CATALOGUE["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .filter(|name| *name != "generate_synthetic_sbr_fixture")
        .collect();
    let reviews: BTreeSet<_> = scope["review_tools"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(reviews, registered);
    assert_eq!(
        scope["synthetic_only_tools"],
        json!(["generate_synthetic_sbr_fixture"])
    );
    let refusal = AccountingServer.call("refuse_div7a", &json!({}));
    assert_eq!(refusal["isError"], false);
    assert_eq!(
        refusal["structuredContent"]["reason"],
        resources::read("aus-accounting://div7a-scope").unwrap()
    );
}

#[test]
fn advertised_worksheet_required_inputs_match_each_discovered_variant() {
    // test_worksheet_catalogue.py: required_inputs follows the callable model.
    let scope = resources::read("aus-accounting://scope").unwrap();
    let schema = &contracts::tool("calculate_tax_worksheet").unwrap()["inputSchema"];
    for (kind, entry) in scope["calculation_worksheets"].as_object().unwrap() {
        let variants: Vec<_> = schema["$defs"]
            .as_object()
            .unwrap()
            .values()
            .filter(|variant| variant["properties"]["kind"]["const"] == kind.as_str())
            .collect();
        assert_eq!(variants.len(), 1, "{kind}");
        let advertised: BTreeSet<_> = entry["required_inputs"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        let required: BTreeSet<_> = variants[0]["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| name.as_str().unwrap())
            .collect();
        assert_eq!(advertised, required, "{kind}");
    }
}

#[test]
fn initialization_guidance_and_shared_public_identity_follow_discovery() {
    // Original stdio helpers require live tool/resource names and shared icons/site.
    // The public record's Python package versions remain historical release data.
    let info = serde_json::to_value(AccountingServer.get_info()).unwrap();
    let instructions = info["instructions"].as_str().unwrap();
    for (collection, key) in [("tools", "name"), ("resources", "uri")] {
        for item in contracts::CATALOGUE[collection].as_array().unwrap() {
            let name = item[key].as_str().unwrap();
            assert!(instructions.contains(name), "{name}");
        }
    }
    let record: Value = serde_json::from_str(include_str!("../server.json")).unwrap();
    for key in ["websiteUrl", "icons"] {
        assert_eq!(info["serverInfo"][key], record[key], "{key}");
    }
    assert_eq!(info["serverInfo"]["version"], aus_accounting_mcp::VERSION);
}

#[test]
fn benchmark_prompt_names_current_required_inputs_and_expense_condition() {
    let prompt = prompts::get("compare_ato_benchmarks", None).unwrap();
    let text = prompt["messages"][0]["content"]["text"].as_str().unwrap();
    for required in contracts::tool("get_ato_benchmarks").unwrap()["inputSchema"]["required"]
        .as_array()
        .unwrap()
    {
        assert!(text.contains(required.as_str().unwrap()), "{required}");
    }
    assert!(text.contains("at least one expense bucket"));
    let refused = AccountingServer.call(
        "get_ato_benchmarks",
        &json!({
            "industry":"Bakeries and hot bread shops", "turnover":"850000.00", "other_income":"0"
        }),
    );
    assert_eq!(refused["isError"], true);
    assert!(
        refused["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("no expense figures were supplied")
    );
}

#[test]
fn resource_coverage_and_provenance_come_from_current_linked_tables() {
    // Native adaptation of monkeypatched Python loaders: compare the actual linked
    // tables directly, without adding a runtime injection interface.
    let served = resources::read("aus-accounting://payday-coverage").unwrap();
    let calendar = calendar::load_calendar(None).unwrap();
    let gic = rates::load_gic().unwrap();
    assert_eq!(
        served["calendar"],
        json!({
            "verified_from":calendar.verified_from.isoformat(),
            "verified_until":calendar.verified_until.isoformat(),
            "coverage_until":calendar.coverage_until.isoformat()
        })
    );
    assert_eq!(served["gic"]["known_until"], gic.last_known().isoformat());
    assert_eq!(served["gic"]["provenance"], gic.provenance());
    let served = resources::read("aus-accounting://benchmark-dataset-years").unwrap();
    let years = dataset::available_years(&DataDir::Shipped);
    assert!(!years.is_empty());
    assert_eq!(served["latest_benchmark_year"], json!(years.last()));
    let entries = served["datasets"].as_array().unwrap();
    assert_eq!(entries.len(), years.len());
    for (entry, year) in entries.iter().zip(&years) {
        let data = dataset::load(Some(year), &DataDir::Shipped).unwrap();
        assert_eq!(entry["benchmark_year"], *year);
        assert_eq!(entry["business_types"], data.business_types.len());
        assert_eq!(
            entry["source"],
            ato_benchmark_compare::report::raw_json(&data.source)
        );
    }
}

#[test]
fn discovery_year_and_industry_bounds_admit_every_shipped_dataset() {
    let bound = length("get_div7a_benchmark_rate", "year_of_income");
    for (tool, argument) in [
        ("list_ato_benchmark_industries", "year"),
        ("get_ato_benchmarks", "year"),
        ("review_div7a_loan", "year_of_income"),
        ("review_div7a_loan", "year_loan_made"),
    ] {
        assert_eq!(length(tool, argument), bound);
    }
    let year = div7a_loan_review::years::parse_year(
        &div7a_loan_review::raw::Raw::Str("2026-27".into()),
        "year",
    )
    .unwrap();
    assert_eq!(year.label().chars().count(), bound);
    let industry = length("get_ato_benchmarks", "industry");
    assert_eq!(length("list_ato_benchmark_industries", "search"), industry);
    let years = dataset::available_years(&DataDir::Shipped);
    assert!(!years.is_empty());
    let mut names = 0;
    for year in years {
        assert_eq!(year.chars().count(), bound, "{year}");
        for business in dataset::load(Some(&year), &DataDir::Shipped)
            .unwrap()
            .business_types
        {
            assert!(
                business.name.chars().count() <= industry,
                "{}",
                business.name
            );
            names += 1;
        }
    }
    assert!(names > 0);
}

#[test]
fn discovery_date_bounds_admit_every_linked_engine_format() {
    let bound = length("calc_payday_super_deadline", "qe_day");
    for (tool, argument) in [
        ("calc_payday_super_deadline", "as_at"),
        ("calc_payday_super_deadline", "remitted"),
        ("calc_payday_super_deadline", "received"),
        ("calc_payday_super_deadline", "next_standard_qe_day"),
        ("review_payday_super_contributions", "as_at"),
        ("build_payday_super_evidence_pack", "as_at"),
    ] {
        assert_eq!(length(tool, argument), bound);
    }
    let moment = NaiveDate::from_ymd_opt(2027, 9, 13)
        .unwrap()
        .and_hms_micro_opt(22, 30, 45, 123456)
        .unwrap();
    let mut shapes = vec![moment.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()];
    for date in csv_io::DATE_FORMATS {
        shapes.push(moment.format(date).to_string());
        for time in csv_io::TIME_FORMATS {
            // Python %f is six digits; chrono's bare %f defaults to nine.
            let format = format!("{date} {time}").replace("%f", "%6f");
            shapes.push(moment.format(&format).to_string());
        }
    }
    let expected = payday_super_checker::py::Date::new(2027, 9, 13).unwrap();
    for shape in shapes {
        assert_eq!(
            csv_io::parse_date_text(&shape).unwrap(),
            Some(expected),
            "{shape}"
        );
        assert!(shape.chars().count() <= bound, "{shape}");
    }
}
