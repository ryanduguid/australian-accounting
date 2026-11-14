// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Schedule 9 withholding for seniors and pensioners against each schedule's
//! worked example and every row of its weekly tax table
//! (docs/calculation-evidence.md#seniors-and-pensioners), the Medicare levy
//! exemption scales, the fortnightly and monthly conversions and the refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const COMMAND: &str = "seniors-pensioners-withholding";
const STATUSES: [&str; 3] = ["single", "illness_separated", "couple"];

fn input(earnings: &str, period: &str, status: &str, exemption: &str, year: &str) -> Value {
    json!({
        "earnings": earnings, "pay_period": period, "status": status,
        "medicare_levy_exemption": exemption, "year": year, "scope_confirmed": true,
    })
}

fn withholding(earnings: &str, period: &str, status: &str, exemption: &str, year: &str) -> String {
    let result = api::evaluate(COMMAND, &input(earnings, period, status, exemption, year));
    let result = result.unwrap_or_else(|e| panic!("{year} {period} {earnings} {status}: {e}"));
    result["amounts"]["withholding"]
        .as_str()
        .unwrap()
        .to_string()
}

fn refusal(facts: &Value) -> (&'static str, String) {
    let refusal = api::evaluate_coded(COMMAND, facts).unwrap_err();
    (refusal.code, refusal.message)
}

/// The years each schedule applies to: 1 July 2024 to 30 June 2026, and from 1 July 2026.
const SCHEDULE_YEARS: [(&str, &[&str]); 2] = [
    ("2024-26", &["2024-25", "2025-26"]),
    ("2026-27", &["2026-27"]),
];

/// 'A payee has weekly earnings of $662.95 and has not claimed a Medicare levy
/// exemption': column 2, 3 or 4 by status.
#[test]
fn the_worked_example() {
    for (schedule, expected) in [
        ("2024-26", ["5.00", "11.00", "26.00"]),
        ("2026-27", ["2.00", "6.00", "21.00"]),
    ] {
        let years = SCHEDULE_YEARS
            .iter()
            .find(|(s, _)| *s == schedule)
            .unwrap()
            .1;
        for year in years {
            for (status, amount) in STATUSES.iter().zip(expected) {
                let result =
                    api::evaluate(COMMAND, &input("662.95", "weekly", status, "none", year));
                let result = result.unwrap();
                assert_eq!(result["amounts"]["withholding"], amount, "{year} {status}");
                assert_eq!(result["amounts"]["weekly_earnings_used"], "662.99");
                assert_eq!(result["period"], format!("{year} weekly"));
            }
        }
    }
}

/// Every row of the weekly tax table in the schedule's PDF (NAT 4466), columns
/// 2 to 4: 2,120 weekly earnings for each status.
fn matches_the_weekly_table(table: &str, year: &str) {
    let mut lines = table.lines();
    assert_eq!(
        lines.next(),
        Some("weekly_earnings,single,illness_separated,couple")
    );
    let rows: Vec<Vec<&str>> = lines.map(|l| l.split(',').collect()).collect();
    assert_eq!(rows.len(), 2120);
    for row in &rows {
        assert_eq!(row.len(), 4, "{row:?}");
        for (status, expected) in STATUSES.iter().zip(&row[1..]) {
            let amount = withholding(row[0], "weekly", status, "none", year);
            assert_eq!(&amount, expected, "{year} {row:?} {status}");
        }
    }
}

#[test]
fn every_row_of_the_weekly_table_from_july_2026() {
    // Published on 17 June 2026: weekly earnings $566 to $2,685.
    let table = include_str!("data/seniors_pensioners_weekly_2026_27.csv");
    matches_the_weekly_table(table, "2026-27");
}

#[test]
fn every_row_of_the_weekly_table_from_july_2024_to_june_2026() {
    // Last updated on 17 June 2024: weekly earnings $553 to $2,672.
    let table = include_str!("data/seniors_pensioners_weekly_2024_26.csv");
    for year in ["2024-25", "2025-26"] {
        matches_the_weekly_table(table, year);
    }
}

/// Scales 4 to 9, which the schedule gives as coefficients only: each status
/// without an exemption, with a full one and with a half one, worked by hand
/// from the published coefficients. $1,300 a week shows all 3 apart.
#[test]
fn medicare_levy_exemptions_select_scales_4_to_9() {
    for (schedule, earnings, by_status) in [
        ("2026-27", "1300.00", [["235.00", "209.00", "214.00"]; 3]),
        ("2024-26", "1300.00", [["240.00", "214.00", "224.00"]; 3]),
        (
            "2026-27",
            "1000.00",
            [
                ["128.00", "113.00", "113.00"],
                ["133.00", "118.00", "118.00"],
                ["133.00", "118.00", "118.00"],
            ],
        ),
        (
            "2024-26",
            "1000.00",
            [
                ["141.00", "121.00", "121.00"],
                ["143.00", "123.00", "123.00"],
                ["143.00", "123.00", "123.00"],
            ],
        ),
    ] {
        let years = SCHEDULE_YEARS
            .iter()
            .find(|(s, _)| *s == schedule)
            .unwrap()
            .1;
        for year in years {
            for (status, amounts) in STATUSES.iter().zip(by_status) {
                for (exemption, amount) in ["none", "full", "half"].iter().zip(amounts) {
                    let got = withholding(earnings, "weekly", status, exemption, year);
                    assert_eq!(got, amount, "{year} {earnings} {status} {exemption}");
                }
            }
        }
    }
}

/// The result names the scale and the coefficients it applied.
#[test]
fn the_result_names_the_scale_and_its_coefficients() {
    // Scale 4 (single, full exemption) for 2026-27 at $1,300.99:
    // 0.3000 x 1,300.99 - 181.7308 = 208.5662, $209.
    let result = api::evaluate(
        COMMAND,
        &input("1300", "weekly", "single", "full", "2026-27"),
    );
    let rates = &result.unwrap()["rates"];
    assert_eq!(rates["scale"], "4");
    assert_eq!(
        (&rates["a"], &rates["b"]),
        (&json!("0.3000"), &json!("181.7308"))
    );
    let result = api::evaluate(
        COMMAND,
        &input("1300", "weekly", "couple", "half", "2026-27"),
    );
    assert_eq!(result.unwrap()["rates"]["scale"], "9");
}

/// 'Calculating fortnightly or monthly withholding amounts', worked by hand:
/// $1,325.90 a fortnight is the worked example's $662.99 a week, twice; a
/// month ending in 33 cents gains a cent before 3/13, so $2,877.33 is $664.99
/// a week and $2,877.32 is $663.99, each weekly amount times 13/3, rounded.
#[test]
fn fortnightly_and_monthly_pays_convert_through_the_weekly_amount() {
    for (earnings, period, single) in [
        ("1325.90", "fortnightly", "4.00"),
        ("2877.33", "monthly", "13.00"),
        ("2877.32", "monthly", "9.00"),
        ("2600.00", "fortnightly", "470.00"),
        ("4500.00", "monthly", "641.00"),
    ] {
        let result = api::evaluate(
            COMMAND,
            &input(earnings, period, "single", "none", "2026-27"),
        );
        let result = result.unwrap();
        assert_eq!(
            result["amounts"]["withholding"], single,
            "{earnings} {period}"
        );
        assert_eq!(result["period"], format!("2026-27 {period}"));
    }
    let used = |earnings: &str| {
        let result = api::evaluate(
            COMMAND,
            &input(earnings, "monthly", "single", "none", "2026-27"),
        );
        result.unwrap()["amounts"]["weekly_earnings_used"].clone()
    };
    assert_eq!(used("2877.33"), "664.99");
    assert_eq!(used("2877.32"), "663.99");
    // The couple column a fortnight and the half exemption a month, 2024-25.
    assert_eq!(
        withholding("1325.90", "fortnightly", "couple", "none", "2024-25"),
        "52.00"
    );
    assert_eq!(
        withholding("4500.00", "monthly", "single", "half", "2025-26"),
        "581.00"
    );
}

#[test]
fn below_the_first_step_nothing_is_withheld() {
    // 2026-27 scale 1 withholds nothing below $648 a week; scale 3 below $567.
    assert_eq!(
        withholding("647.99", "weekly", "single", "none", "2026-27"),
        "0.00"
    );
    assert_eq!(
        withholding("566.99", "weekly", "couple", "full", "2026-27"),
        "0.00"
    );
    assert_eq!(
        withholding("0.00", "monthly", "single", "half", "2026-27"),
        "0.00"
    );
}

/// The worked example's facts with one field replaced.
fn with(key: &str, value: Value) -> Value {
    let mut facts = input("662.95", "weekly", "single", "none", "2026-27");
    facts[key] = value;
    facts
}

#[test]
fn what_the_schedule_does_not_settle_is_refused() {
    // No quarterly rule in Schedule 9.
    assert_eq!(
        refusal(&with("pay_period", json!("quarterly"))),
        (
            "input.pay_period_invalid",
            "pay_period must be weekly, fortnightly or monthly.".into()
        )
    );
    // Tax offsets, the Medicare levy adjustment and a Schedule 1 scale are not inputs.
    for (key, value) in [
        ("tax_offsets", json!("422.00")),
        ("medicare_adjustment_children", json!(1)),
        ("scale", json!(2)),
        ("study_loan", json!(true)),
    ] {
        assert_eq!(
            refusal(&with(key, value)),
            (
                "input.field_unknown",
                format!("{key} is not an input of {COMMAND}")
            )
        );
    }
}

#[test]
fn malformed_input_is_refused() {
    assert_eq!(
        refusal(&with("status", json!("married"))),
        (
            "input.status_invalid",
            "status must be single, illness_separated or couple.".into()
        )
    );
    assert_eq!(refusal(&with("status", json!(1))).0, "input.status_invalid");
    assert_eq!(
        refusal(&with("medicare_levy_exemption", json!(true))).0,
        "input.medicare_levy_exemption_invalid"
    );
    let mut missing = with("year", json!("2026-27"));
    missing
        .as_object_mut()
        .unwrap()
        .remove("medicare_levy_exemption");
    assert_eq!(refusal(&missing).0, "input.missing_field");
    assert_eq!(
        refusal(&with("year", json!("2027-28"))),
        (
            "period.unsupported",
            "Unsupported period '2027-28'; supported: 2024-25, 2025-26, 2026-27.".into()
        )
    );
    assert_eq!(
        refusal(&with("scope_confirmed", json!(false))).0,
        "scope.not_confirmed"
    );
    for earnings in [json!("1.005"), json!(662.95), json!("-1.00")] {
        assert_eq!(
            refusal(&with("earnings", earnings)).0,
            "input.amount_invalid"
        );
    }
}

#[test]
fn the_catalogue_lists_the_worksheet_with_its_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["seniors_pensioners_withholding"];
    let facts = &entry["example"]["facts"];
    let mut facts = facts.clone();
    facts.as_object_mut().unwrap().remove("kind");
    let result = api::evaluate(COMMAND, &facts).unwrap();
    assert_eq!(result["amounts"]["withholding"], "2.00");
    assert_eq!(
        entry["source"],
        "https://www.ato.gov.au/tax-rates-and-codes/schedule-9-tax-table-for-seniors-and-pensioners"
    );
    let periods = api::evaluate(
        "periods",
        &json!({"kind": "seniors_pensioners_withholding"}),
    );
    assert_eq!(periods.unwrap().as_array().unwrap().len(), 3);
}
