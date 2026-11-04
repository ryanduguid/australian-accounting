// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: FSL-1.1-ALv2

//! `compare-figures` with `"output": "gaps"`: for each ratio the comparison
//! reports above or below its range, the dollar distance from the figure
//! behind it to the nearest range edge at the same turnover, as a working.
//! Every expected figure is the edge times turnover, worked by hand.
use ato_benchmark_compare::api;
use serde_json::{Value, json};

/// The README's bakery, bucket by bucket from examples/bakery-mapping.csv:
/// other expenses are $4,800 + $3,600 + $1,900 + $21,000 + $18,000 + $7,400
/// + $6,200 + $24,750 = $87,650.
fn bakery() -> Value {
    json!({
        "turnover": "850000.00", "other_income": "1200.00",
        "cost_of_sales": "270000.00", "cost_of_sales_labour": "95000.00",
        "salary_wages": "175000.00", "contractor_commission": "7800.00",
        "associated_persons": "45000.00", "rent": "62000.00",
        "motor_vehicle": "9500.00", "other_expense": "87650.00",
    })
}

fn input(totals: Value, output: &str) -> Value {
    json!({
        "dataset": {"year": "2023-24", "data_dir": null},
        "industry": "Bakeries and hot bread shops",
        "totals": totals,
        "output": output,
    })
}

fn gaps(totals: Value) -> Vec<Value> {
    let r = api::evaluate("compare-figures", &input(totals, "gaps")).unwrap();
    assert!(
        r["note"]
            .as_str()
            .unwrap()
            .starts_with("Each gap is a working")
    );
    r["gaps"].as_array().unwrap().clone()
}

/// (ratio, edge, benchmark edge, figure, figure at the edge, gap) per row.
fn rows(gaps: &[Value]) -> Vec<[&str; 6]> {
    gaps.iter()
        .map(|g| {
            [
                "ratio",
                "edge",
                "benchmark_edge",
                "figure",
                "figure_at_edge",
                "gap",
            ]
            .map(|k| g[k].as_str().unwrap())
        })
        .collect()
}

/// The README's report: motor vehicle expenses, $9,500 of $850,000 (1.12%),
/// are above the ATO's single 1% figure, which is 0.01 × $850,000 = $8,500,
/// so the gap is $1,000. Every other ratio is within its range.
#[test]
fn the_bakery_is_1000_dollars_above_the_motor_vehicle_figure() {
    let g = gaps(bakery());
    assert_eq!(
        rows(&g),
        [[
            "motor_vehicle_to_turnover",
            "maximum",
            "0.01",
            "9500.00",
            "8500.0000",
            "1000.0000"
        ]]
    );
    assert_eq!(g[0]["status"], "above");
    assert_eq!(g[0]["is_key_ratio"], false);
    assert_eq!(g[0]["turnover"], "850000.00");
}

/// Rent cut to $30,000: rent is 3.53% against a 5% floor, 0.05 × $850,000 =
/// $42,500, so $12,500 below; total expenses fall by $32,000 to $674,950,
/// 79.41% against an 82% floor, 0.82 × $850,000 = $697,000, so $22,050
/// below. Rows keep the comparison's ratio order.
#[test]
fn a_ratio_below_its_range_measures_up_to_the_floor() {
    let mut totals = bakery();
    totals["rent"] = json!("30000.00");
    let g = gaps(totals);
    assert_eq!(
        rows(&g),
        [
            [
                "total_expenses_to_turnover",
                "minimum",
                "0.82",
                "674950.00",
                "697000.0000",
                "22050.0000"
            ],
            [
                "rent_to_turnover",
                "minimum",
                "0.05",
                "30000.00",
                "42500.0000",
                "12500.0000"
            ],
            [
                "motor_vehicle_to_turnover",
                "maximum",
                "0.01",
                "9500.00",
                "8500.0000",
                "1000.0000"
            ],
        ]
    );
    assert!(g[..2].iter().all(|row| row["status"] == "below"));
}

/// Every ratio above its range, so each row's figure is the one its ratio
/// divides: cost of sales $400,000 less 0.36 × $850,000 = $94,000; total
/// expenses $800,000 less 0.9 × $850,000 = $35,000; labour $300,000 less
/// 0.34 × $850,000 = $11,000; rent $80,000 less 0.08 × $850,000 = $12,000;
/// motor vehicle $20,000 less 0.01 × $850,000 = $11,500.
#[test]
fn each_row_uses_the_figure_its_ratio_divides() {
    let totals = json!({
        "turnover": "850000", "other_income": "0", "cost_of_sales": "400000",
        "cost_of_sales_labour": "0", "salary_wages": "300000",
        "contractor_commission": "0", "associated_persons": "0", "rent": "80000",
        "motor_vehicle": "20000", "other_expense": "0",
    });
    let g = gaps(totals);
    let gap: Vec<(&str, &str)> = g
        .iter()
        .map(|row| (row["ratio"].as_str().unwrap(), row["gap"].as_str().unwrap()))
        .collect();
    assert_eq!(
        gap,
        [
            ("cost_of_sales_to_turnover", "94000.00"),
            ("total_expenses_to_turnover", "35000.0"),
            ("labour_to_turnover", "11000.00"),
            ("rent_to_turnover", "12000.00"),
            ("motor_vehicle_to_turnover", "11500.00"),
        ]
    );
    assert_eq!(g[0]["is_key_ratio"], true);
}

/// A ratio withheld as not_supplied has no gap, and nothing quotes the
/// turnover it would rest on; a ratio within its range has none either.
#[test]
fn only_reported_outside_ratios_have_gaps() {
    let mut totals = bakery();
    totals.as_object_mut().unwrap().remove("other_income");
    assert!(gaps(totals).is_empty());
    let mut within = bakery();
    within["motor_vehicle"] = json!("8500.00");
    assert!(gaps(within).is_empty());
}

/// Ratios are compared at 4 places, so an edge finer than that could put the
/// exact figure on the other side of the verdict: no gap is worked out.
#[test]
fn an_edge_finer_than_the_ratio_is_refused() {
    let mut data: Value =
        serde_json::from_str(include_str!("../data/benchmarks-2023-24.json")).unwrap();
    let bakery_type = data["business_types"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|b| b["name"] == "Bakeries and hot bread shops")
        .unwrap();
    bakery_type["turnover_bands"][2]["cost_of_sales_to_turnover"]["max"] = json!("0.30005");
    let mut request = input(bakery(), "gaps");
    request["dataset"] = json!({"text": data.to_string(), "source_name": "edited.json"});
    assert_eq!(
        api::evaluate("compare-figures", &request).unwrap_err(),
        "Cost of sales to turnover: the range edge 0.30005 has more than 4 decimal places, \
         the precision ratios are compared at, so no gap is worked out"
    );
}
