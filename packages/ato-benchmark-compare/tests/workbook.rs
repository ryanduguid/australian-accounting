//! The workbook test at the repository root reads tests/data/workbook-sample.json,
//! the engine's comparison of the bakery example. This holds that file to the
//! engine, so a change to either the engine or the frozen answer fails here.

use ato_benchmark_compare::api::evaluate;
use serde_json::{Value, json};

fn example(name: &str) -> String {
    format!("{}/examples/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn the_workbook_sample_comparison_is_the_engines() {
    let source = evaluate("read-pnl", &json!({"path": example("bakery-pnl.csv")})).unwrap();
    let reviewed = evaluate(
        "read-mapping",
        &json!({"path": example("bakery-mapping.csv")}),
    )
    .unwrap();
    let routed = evaluate(
        "route",
        &json!({"rows": source["rows"], "mapping": reviewed, "flip": false}),
    )
    .unwrap();
    let totals: serde_json::Map<String, Value> = routed["totals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pair| (pair[0].as_str().unwrap().to_string(), pair[1].clone()))
        .collect();
    let comparison = evaluate(
        "compare-figures",
        &json!({
            "dataset": {"year": "2023-24", "data_dir": null},
            "industry": "Bakeries and hot bread shops",
            "totals": totals,
            "totals_supplied": routed["supplied_buckets"],
            "output": "to_dict",
        }),
    )
    .unwrap();
    let frozen: Value = serde_json::from_str(include_str!("data/workbook-sample.json")).unwrap();
    assert_eq!(
        comparison.to_string(),
        frozen.to_string(),
        "regenerate tests/data/workbook-sample.json from the compare to_dict output after a deliberate change"
    );
}
