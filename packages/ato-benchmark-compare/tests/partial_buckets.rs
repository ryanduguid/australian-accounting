// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use ato_benchmark_compare::cli;
use serde_json::Value;

#[test]
fn an_unreadable_wages_row_withholds_a_partially_populated_bucket() {
    let dir = std::env::temp_dir().join(format!("ato-partial-bucket-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pnl = dir.join("invented.csv");
    let mapping = dir.join("reviewed.csv");
    let source = include_str!("../examples/bakery-pnl.csv").replace(
        "Shop wages,175000.00,166000.00",
        "Shop wages,n/a,166000.00\nWolff wages,25000.00,24000.00",
    );
    let reviewed = format!(
        "{}\nWolff wages,salary_wages,reviewed,25000.00,invented regression account\n",
        include_str!("../examples/bakery-mapping.csv").trim_end()
    );
    std::fs::write(&pnl, source).unwrap();
    std::fs::write(&mapping, reviewed).unwrap();
    let args: Vec<String> = [
        "compare",
        "--profit-and-loss",
        pnl.to_str().unwrap(),
        "--mapping",
        mapping.to_str().unwrap(),
        "--industry",
        "Bakeries and hot bread shops",
        "--amount-column",
        "1",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let mut json_args = args.clone();
    json_args.extend(["--json".into(), "-".into()]);
    let (code, output, diagnostics) = cli::run(&json_args);
    assert_eq!(code, 0, "{diagnostics}");
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["bucket_totals"]["salary_wages"], Value::Null);
    assert_eq!(value["figures"]["labour"], Value::Null);
    assert_eq!(value["figures"]["total_expenses"], Value::Null);
    for name in ["labour_to_turnover", "total_expenses_to_turnover"] {
        let ratio = value["ratios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|ratio| ratio["ratio"] == name)
            .unwrap();
        assert_eq!(ratio["value"], Value::Null);
        assert_eq!(ratio["status"], "not_supplied");
    }
    assert_eq!(value["figures"]["sales_of_goods_and_services"], "850000.00");
    let (code, text, diagnostics) = cli::run(&args);
    assert_eq!(code, 0, "{diagnostics}");
    for name in ["Labour to turnover", "Total expenses to turnover"] {
        let line = text.lines().find(|line| line.starts_with(name)).unwrap();
        assert!(line.contains("not_supplied"), "{line}");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn confirming_no_additional_income_cannot_complete_unreadable_mapped_income() {
    let dir = std::env::temp_dir().join(format!("ato-partial-income-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pnl = dir.join("invented.csv");
    let mapping = dir.join("reviewed.csv");
    let source = include_str!("../examples/bakery-pnl.csv").replace(
        "Interest received,1200.00,900.00",
        "Interest received,n/a,900.00\nSecond interest,250.00,240.00",
    );
    let reviewed = format!(
        "{}\nSecond interest,other_income,reviewed,250.00,invented regression account\n",
        include_str!("../examples/bakery-mapping.csv").trim_end()
    );
    std::fs::write(&pnl, source).unwrap();
    std::fs::write(&mapping, reviewed).unwrap();
    let args: Vec<String> = [
        "compare",
        "--profit-and-loss",
        pnl.to_str().unwrap(),
        "--mapping",
        mapping.to_str().unwrap(),
        "--industry",
        "Bakeries and hot bread shops",
        "--amount-column",
        "1",
        "--confirm-other-income-nil",
        "--json",
        "-",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let (code, output, diagnostics) = cli::run(&args);
    assert_eq!(code, 0, "{diagnostics}");
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["bucket_totals"]["other_income"], Value::Null);
    let ratios = value["ratios"].as_array().unwrap();
    assert!(!ratios.is_empty());
    for ratio in ratios {
        assert_eq!(ratio["value"], Value::Null);
        assert_eq!(ratio["status"], "not_supplied");
    }
    std::fs::remove_dir_all(dir).unwrap();
}
