// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `portable-lsl-levy`: Coal LSL parity against `ryanduguid.github.io`'s
//! `tools/coal-lsl-levy` vectors plus all 19 verified state and territory
//! portable long service leave schemes, statutory rounding boundaries and
//! structured refusals.

use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    let result = api::evaluate("portable-lsl-levy", input).unwrap();
    let doc = api::evaluate(
        "derive",
        &json!({"command": "portable-lsl-levy", "input": input}),
    )
    .unwrap();
    assert_eq!(doc["result"].to_string(), result.to_string());
    result
}

fn refused(input: &Value) -> String {
    let err = api::evaluate_coded("portable-lsl-levy", input).unwrap_err();
    let derived = api::evaluate_coded(
        "derive",
        &json!({"command": "portable-lsl-levy", "input": input}),
    )
    .unwrap_err();
    assert_eq!(derived, err);
    err.code.to_string()
}

#[test]
fn coal_lsl_matches_site_parity_vectors() {
    let fixture: Value =
        serde_json::from_str(include_str!("data/coal_lsl_levy_site_cases.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert!(cases.len() >= 15);
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let out = run(&case["input"]);
        let exp = &case["expected"];
        assert_eq!(out["jurisdiction"], "cth", "{id}");
        assert_eq!(out["scheme"], "coal", "{id}");
        assert_eq!(out["coal_branch"], exp["coal_branch"], "{id}");
        assert_eq!(
            out["eligible_wages_exact"], exp["eligible_wages_exact"],
            "{id}"
        );
        assert_eq!(out["amounts"]["levy"], exp["levy"], "{id}");
        assert_eq!(
            out["levy_before_rounding"], exp["levy_before_rounding"],
            "{id}"
        );
        if let Some(winner) = exp.get("winner") {
            assert_eq!(out["winner"], *winner, "{id}");
        }
        if let Some(fa) = exp.get("formula_a") {
            assert_eq!(out["amounts"]["formula_a"], *fa, "{id}");
        }
        if let Some(fb) = exp.get("formula_b") {
            assert_eq!(out["amounts"]["formula_b"], *fb, "{id}");
        }
        if let Some(n) = exp.get("excluded_count").and_then(Value::as_u64) {
            assert_eq!(
                out["excluded_bonuses"].as_array().unwrap().len() as u64,
                n,
                "{id}"
            );
        }
    }
}

#[test]
fn coal_lsl_pre_2024_casual_supports_base_rate_and_sacrifice() {
    let base = run(&json!({
        "jurisdiction": "cth",
        "scheme": "coal",
        "year": "2023-24",
        "scope_confirmed": true,
        "reporting_month": "2023-12",
        "branch": "casual",
        "eligible_employee": true,
        "base_rate_of_pay": "1800.00"
    }));
    assert_eq!(base["coal_branch"], "pre-2024");
    assert_eq!(base["eligible_wages_exact"], "1800.0000");
    assert_eq!(base["amounts"]["levy"], "48.60");

    let ord = run(&json!({
        "jurisdiction": "cth",
        "scheme": "coal",
        "year": "2023-24",
        "scope_confirmed": true,
        "reporting_month": "2023-12",
        "branch": "casual",
        "eligible_employee": true,
        "ordinary_rate_of_pay": "1000.00",
        "salary_sacrificed": "100.00"
    }));
    assert_eq!(ord["coal_branch"], "pre-2024");
    assert_eq!(ord["eligible_wages_exact"], "1100.0000");
    assert_eq!(ord["amounts"]["levy"], "29.70");

    // Small non-zero levy_before_rounding formats as plain fixed decimal, never scientific notation.
    let tiny = run(&json!({
        "jurisdiction": "cth",
        "scheme": "coal",
        "year": "2026-27",
        "scope_confirmed": true,
        "reporting_month": "2026-10",
        "branch": "base_rate",
        "eligible_employee": true,
        "base_rate_of_pay": "0.01"
    }));
    assert_eq!(tiny["eligible_wages_exact"], "0.0100");
    assert_eq!(tiny["levy_before_rounding"], "0.00027000");
    assert_eq!(tiny["amounts"]["levy"], "0.00");

    // Zero allowances with explicit flag, and s 3B(3)(a) casual without base pay or loading.
    let zero_allow = run(&json!({
        "jurisdiction": "cth",
        "scheme": "coal",
        "year": "2026-27",
        "scope_confirmed": true,
        "reporting_month": "2026-10",
        "branch": "base_rate",
        "eligible_employee": true,
        "base_rate_of_pay": "1000.00",
        "allowances": "0.00",
        "allowances_exclude_expense_reimbursements": false
    }));
    assert_eq!(zero_allow["amounts"]["levy"], "27.00");

    let casual_sac_only = run(&json!({
        "jurisdiction": "cth",
        "scheme": "coal",
        "year": "2026-27",
        "scope_confirmed": true,
        "reporting_month": "2026-10",
        "branch": "casual",
        "eligible_employee": true,
        "instrument_specifies_loading": true,
        "loading_quantifiable": true,
        "salary_sacrificed": "200.00"
    }));
    assert_eq!(casual_sac_only["eligible_wages_exact"], "200.0000");
    assert_eq!(casual_sac_only["amounts"]["levy"], "5.40");

    let casual_empty = run(&json!({
        "jurisdiction": "cth",
        "scheme": "coal",
        "year": "2026-27",
        "scope_confirmed": true,
        "reporting_month": "2026-10",
        "branch": "casual",
        "eligible_employee": true,
        "instrument_specifies_loading": true,
        "loading_quantifiable": true
    }));
    assert_eq!(casual_empty["eligible_wages_exact"], "0.0000");
    assert_eq!(casual_empty["amounts"]["levy"], "0.00");
}

#[test]
fn coal_lsl_refusals_match_service_contract() {
    // R2: employee not eligible
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2025-26",
            "scope_confirmed": true,
            "reporting_month": "2026-06",
            "branch": "annual_salary",
            "eligible_employee": false,
            "annual_salary_paid": "10000.00"
        })),
        "portable_lsl_levy.employee_ineligible"
    );
    // R4: unread casual pay components on s 3B(3)(b)
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2025-26",
            "scope_confirmed": true,
            "reporting_month": "2026-06",
            "branch": "casual",
            "eligible_employee": true,
            "instrument_specifies_loading": false,
            "loading_quantifiable": false,
            "base_rate_of_pay": "1800.00",
            "casual_loading": "450.00",
            "ordinary_rate_of_pay": "0.00"
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    // Unread ordinary_rate_of_pay on s 3B(3)(a)
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2025-26",
            "scope_confirmed": true,
            "reporting_month": "2026-06",
            "branch": "casual",
            "eligible_employee": true,
            "instrument_specifies_loading": true,
            "loading_quantifiable": true,
            "ordinary_rate_of_pay": "2250.00"
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    // Pre-2024 casual with non-zero casual_loading, both pay fields, or post-2024 loading flags
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2023-24",
            "scope_confirmed": true,
            "reporting_month": "2023-12",
            "branch": "casual",
            "eligible_employee": true,
            "base_rate_of_pay": "1800.00",
            "instrument_specifies_loading": true
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2023-24",
            "scope_confirmed": true,
            "reporting_month": "2023-12",
            "branch": "casual",
            "eligible_employee": true,
            "base_rate_of_pay": "1800.00",
            "loading_quantifiable": false
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2023-24",
            "scope_confirmed": true,
            "reporting_month": "2023-12",
            "branch": "casual",
            "eligible_employee": true,
            "base_rate_of_pay": "1800.00",
            "casual_loading": "450.00"
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2023-24",
            "scope_confirmed": true,
            "reporting_month": "2023-12",
            "branch": "casual",
            "eligible_employee": true,
            "base_rate_of_pay": "1800.00",
            "ordinary_rate_of_pay": "2250.00"
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    // R5: positive allowances without reimbursement confirmation
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2025-26",
            "scope_confirmed": true,
            "reporting_month": "2026-06",
            "branch": "base_rate",
            "eligible_employee": true,
            "base_rate_of_pay": "6000.00",
            "allowances": "500.00",
            "allowances_exclude_expense_reimbursements": false
        })),
        "portable_lsl_levy.reimbursements_unconfirmed"
    );
    // N140 / V7: reporting_month before 2023-07, malformed, or outside year
    for (year, month) in [
        ("2023-24", "2023-06"),
        ("2025-26", "2026-13"),
        ("2025-26", "2026-00"),
        ("2025-26", "2026-06-01"),
        ("2025-26", "2026-07"),
    ] {
        assert_eq!(
            refused(&json!({
                "jurisdiction": "cth",
                "scheme": "coal",
                "year": year,
                "scope_confirmed": true,
                "reporting_month": month,
                "branch": "annual_salary",
                "eligible_employee": true,
                "annual_salary_paid": "10000.00"
            })),
            "portable_lsl_levy.reporting_month_invalid",
            "{year} {month}"
        );
    }
    // Invalid branch & invalid bonus frequency / precision
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "reporting_month": "2026-10",
            "branch": "contractor",
            "eligible_employee": true
        })),
        "portable_lsl_levy.branch_invalid"
    );
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "reporting_month": "2026-10",
            "branch": "annual_salary",
            "eligible_employee": true,
            "annual_salary_paid": "10000.00",
            "bonuses": [{"amount": "1.00", "frequency": "daily"}]
        })),
        "portable_lsl_levy.bonus_invalid"
    );
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "reporting_month": "2026-10",
            "branch": "annual_salary",
            "eligible_employee": true,
            "annual_salary_paid": "10000.00",
            "bonuses": [{"amount": "1.005", "frequency": "monthly"}]
        })),
        "portable_lsl_levy.bonus_invalid"
    );
    // Missing required Coal LSL fields and non-string reporting_month
    for missing in [
        json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "branch": "annual_salary",
            "eligible_employee": true,
            "annual_salary_paid": "10000.00"
        }),
        json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "reporting_month": "2026-10",
            "branch": "annual_salary",
            "annual_salary_paid": "10000.00"
        }),
        json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "reporting_month": "2026-10",
            "branch": "casual",
            "eligible_employee": true
        }),
        json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "reporting_month": "2026-10",
            "branch": "casual",
            "eligible_employee": true,
            "instrument_specifies_loading": true
        }),
        json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "reporting_month": "2026-10",
            "branch": "base_rate",
            "eligible_employee": true
        }),
    ] {
        assert_eq!(refused(&missing), "input.missing_field");
    }
    assert_eq!(
        refused(&json!({
            "jurisdiction": "cth",
            "scheme": "coal",
            "year": "2026-27",
            "scope_confirmed": true,
            "reporting_month": 202610,
            "branch": "annual_salary",
            "eligible_employee": true,
            "annual_salary_paid": "10000.00"
        })),
        "portable_lsl_levy.reporting_month_invalid"
    );
    // Malformed bonuses shapes and 501-item limit
    let over_limit: Vec<Value> = (0..501)
        .map(|_| json!({"amount": "1.00", "frequency": "monthly"}))
        .collect();
    for bad_bonuses in [
        json!("not_a_list"),
        json!(["not_an_object"]),
        json!([{"amount": "10.00", "frequency": "monthly", "extra": true}]),
        Value::Array(over_limit),
    ] {
        assert_eq!(
            refused(&json!({
                "jurisdiction": "cth",
                "scheme": "coal",
                "year": "2026-27",
                "scope_confirmed": true,
                "reporting_month": "2026-10",
                "branch": "annual_salary",
                "eligible_employee": true,
                "annual_salary_paid": "10000.00",
                "bonuses": bad_bonuses
            })),
            "portable_lsl_levy.bonus_invalid"
        );
    }
}

#[test]
fn state_payroll_schemes_match_verified_2026_27_rates() {
    let cases = [
        (
            "nsw",
            "contract_cleaning",
            "2026-27",
            "100000.00",
            "0.01",
            "1000.00",
        ),
        (
            "nsw",
            "community_services",
            "2025-26",
            "100000.00",
            "0.013",
            "1300.00",
        ),
        (
            "nsw",
            "community_services",
            "2026-27",
            "100000.00",
            "0.017",
            "1700.00",
        ),
        (
            "vic",
            "construction",
            "2026-27",
            "100000.00",
            "0.027",
            "2700.00",
        ),
        (
            "vic",
            "community_services",
            "2026-27",
            "100000.00",
            "0.0165",
            "1650.00",
        ),
        (
            "vic",
            "contract_cleaning",
            "2026-27",
            "100000.00",
            "0.018",
            "1800.00",
        ),
        (
            "vic",
            "security",
            "2026-27",
            "100000.00",
            "0.018",
            "1800.00",
        ),
        (
            "qld",
            "contract_cleaning",
            "2026-27",
            "100000.00",
            "0.005",
            "500.00",
        ),
        (
            "qld",
            "community_services",
            "2026-27",
            "100000.00",
            "0.0135",
            "1350.00",
        ),
        (
            "sa",
            "construction",
            "2026-27",
            "100000.00",
            "0.02",
            "2000.00",
        ),
        (
            "sa",
            "community_services",
            "2026-27",
            "100000.00",
            "0.022",
            "2200.00",
        ),
        (
            "wa",
            "construction",
            "2026-27",
            "100000.00",
            "0.0075",
            "750.00",
        ),
        (
            "act",
            "construction",
            "2026-27",
            "100000.00",
            "0.0275",
            "2750.00",
        ),
        (
            "act",
            "community_services",
            "2026-27",
            "100000.00",
            "0.0185",
            "1850.00",
        ),
        (
            "act",
            "security",
            "2026-27",
            "100000.00",
            "0.0147",
            "1470.00",
        ),
        (
            "act",
            "contract_cleaning",
            "2026-27",
            "100000.00",
            "0.0107",
            "1070.00",
        ),
    ];
    for (jurisdiction, scheme, year, wages, expected_rate, expected_levy) in cases {
        let out = run(&json!({
            "jurisdiction": jurisdiction,
            "scheme": scheme,
            "year": year,
            "scope_confirmed": true,
            "ordinary_wages": wages
        }));
        assert_eq!(out["basis"], "ordinary_wages", "{jurisdiction}/{scheme}");
        assert_eq!(
            out["rates"]["rate"], expected_rate,
            "{jurisdiction}/{scheme}"
        );
        assert_eq!(
            out["amounts"]["levy"], expected_levy,
            "{jurisdiction}/{scheme}"
        );
    }
}

#[test]
fn tasbuild_requires_paid_on_time_and_applies_discount_or_standard_rate() {
    let on_time = run(&json!({
        "jurisdiction": "tas",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "ordinary_wages": "50000.00",
        "paid_on_time": true
    }));
    assert_eq!(on_time["rates"]["rate"], "0.018");
    assert_eq!(on_time["amounts"]["levy"], "900.00");

    let standard = run(&json!({
        "jurisdiction": "tas",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "ordinary_wages": "50000.00",
        "paid_on_time": false
    }));
    assert_eq!(standard["rates"]["rate"], "0.025");
    assert_eq!(standard["amounts"]["levy"], "1250.00");

    assert_eq!(
        refused(&json!({
            "jurisdiction": "tas",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true,
            "ordinary_wages": "50000.00"
        })),
        "input.missing_field"
    );
}

#[test]
fn project_cost_schemes_enforce_thresholds_and_statutory_rounding() {
    // NSW: threshold $250,000 incl. GST, 0.25%
    let nsw_below = run(&json!({
        "jurisdiction": "nsw",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "249999.99"
    }));
    assert_eq!(nsw_below["amounts"]["levy"], "0.00");

    let nsw_at = run(&json!({
        "jurisdiction": "nsw",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "250000.00",
        "partial_exemption": false
    }));
    assert_eq!(nsw_at["amounts"]["levy"], "625.00");

    // NSW LSC Building Levy Calculator rounds down to the whole dollar:
    // $250,399 x 0.25% = $625.9975 -> $625.00; $250,400 -> $626.00; $300,200 -> $750.00
    let nsw_floor = run(&json!({
        "jurisdiction": "nsw",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "250399.00"
    }));
    assert_eq!(nsw_floor["amounts"]["levy"], "625.00");

    let nsw_next_dollar = run(&json!({
        "jurisdiction": "nsw",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "250400.00"
    }));
    assert_eq!(nsw_next_dollar["amounts"]["levy"], "626.00");

    let nsw_half = run(&json!({
        "jurisdiction": "nsw",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "300200.00"
    }));
    assert_eq!(nsw_half["amounts"]["levy"], "750.00");

    // Qld: threshold $150,000 excl. GST; s 72(2) nearest dollar, 50c rounded downwards
    let qld_below = run(&json!({
        "jurisdiction": "qld",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "149999.99"
    }));
    assert_eq!(qld_below["amounts"]["lsl_levy"], "0.00");
    assert_eq!(qld_below["amounts"]["training_levy"], "0.00");
    assert_eq!(qld_below["amounts"]["whs_levy"], "0.00");
    assert_eq!(qld_below["amounts"]["levy"], "0.00");

    // At $150,000: LSL = 525.00, training = 150.00, WHS = 187.50 -> 50c rounded DOWN = 187.00 -> total 862.00
    let qld_at = run(&json!({
        "jurisdiction": "qld",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "150000.00"
    }));
    assert_eq!(qld_at["amounts"]["lsl_levy"], "525.00");
    assert_eq!(qld_at["amounts"]["training_levy"], "150.00");
    assert_eq!(qld_at["amounts"]["whs_levy"], "187.00");
    assert_eq!(qld_at["amounts"]["levy"], "862.00");

    // At $150,008: LSL = 525.028 -> 525, training = 150.008 -> 150, WHS = 187.51 -> 188 (>50c rounds UP!) -> total 863
    let qld_up = run(&json!({
        "jurisdiction": "qld",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "150008.00"
    }));
    assert_eq!(qld_up["amounts"]["whs_levy"], "188.00");
    assert_eq!(qld_up["amounts"]["levy"], "863.00");

    // NT: $1,000,000 to $5,000,000,000 incl. GST, 0.1%, s 33(2) nearest dollar, 50c rounded downwards
    let nt_below = run(&json!({
        "jurisdiction": "nt",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "999999.99"
    }));
    assert_eq!(nt_below["amounts"]["levy"], "0.00");

    // $1,000,500 x 0.1% = 1000.50 -> 50c rounded DOWN = $1,000.00
    let nt_half_down = run(&json!({
        "jurisdiction": "nt",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "1000500.00"
    }));
    assert_eq!(nt_half_down["amounts"]["levy"], "1000.00");

    // $1,000,501 x 0.1% = 1000.501 -> >50c rounded UP = $1,001.00
    let nt_over_half = run(&json!({
        "jurisdiction": "nt",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "1000501.00"
    }));
    assert_eq!(nt_over_half["amounts"]["levy"], "1001.00");

    // $5,000,000,000 (inclusive upper bound) -> $5,000,000.00
    let nt_cap = run(&json!({
        "jurisdiction": "nt",
        "scheme": "construction",
        "year": "2026-27",
        "scope_confirmed": true,
        "project_cost": "5000000000.00"
    }));
    assert_eq!(nt_cap["amounts"]["levy"], "5000000.00");

    // Over $5 billion refused
    assert_eq!(
        refused(&json!({
            "jurisdiction": "nt",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true,
            "project_cost": "5000000000.01"
        })),
        "portable_lsl_levy.project_cost_above_tier"
    );
}

#[test]
fn refusals_cover_invalid_or_unverified_jurisdictions_schemes_years_and_exemptions() {
    assert_eq!(
        refused(&json!({
            "jurisdiction": "nz",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true
        })),
        "portable_lsl_levy.jurisdiction_invalid"
    );
    assert_eq!(
        refused(&json!({
            "jurisdiction": "nsw",
            "scheme": "hospitality",
            "year": "2026-27",
            "scope_confirmed": true
        })),
        "portable_lsl_levy.scheme_invalid"
    );
    for (j, s) in [
        ("cth", "construction"),
        ("nsw", "coal"),
        ("nsw", "security"),
        ("qld", "security"),
        ("sa", "contract_cleaning"),
        ("wa", "community_services"),
        ("tas", "contract_cleaning"),
        ("nt", "security"),
    ] {
        assert_eq!(
            refused(&json!({
                "jurisdiction": j,
                "scheme": s,
                "year": "2026-27",
                "scope_confirmed": true
            })),
            "portable_lsl_levy.scheme_unavailable",
            "{j}/{s}"
        );
    }
    // Unverified historical years refused cleanly
    for (j, s, y) in [
        ("nsw", "community_services", "2024-25"),
        ("vic", "construction", "2025-26"),
        ("qld", "contract_cleaning", "2025-26"),
        ("qld", "construction", "2024-25"),
        ("sa", "community_services", "2025-26"),
        ("act", "construction", "2024-25"),
        ("tas", "construction", "2025-26"),
        ("nt", "construction", "2025-26"),
    ] {
        assert_eq!(
            refused(&json!({
                "jurisdiction": j,
                "scheme": s,
                "year": y,
                "scope_confirmed": true,
                "ordinary_wages": "10000.00"
            })),
            "portable_lsl_levy.scheme_year_unsupported",
            "{j}/{s}/{y}"
        );
    }
    // Partial exemption refused on project schemes
    assert_eq!(
        refused(&json!({
            "jurisdiction": "nsw",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true,
            "project_cost": "300000.00",
            "partial_exemption": true
        })),
        "portable_lsl_levy.exemption_unsupported"
    );
    // Mismatched fields refused
    assert_eq!(
        refused(&json!({
            "jurisdiction": "vic",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true,
            "ordinary_wages": "50000.00",
            "paid_on_time": true
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    assert_eq!(
        refused(&json!({
            "jurisdiction": "nsw",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true,
            "ordinary_wages": "50000.00"
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    // Missing ordinary_wages on payroll scheme and coal field passed to state scheme
    assert_eq!(
        refused(&json!({
            "jurisdiction": "vic",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true
        })),
        "input.missing_field"
    );
    assert_eq!(
        refused(&json!({
            "jurisdiction": "vic",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true,
            "ordinary_wages": "50000.00",
            "reporting_month": "2026-10"
        })),
        "portable_lsl_levy.branch_field_mismatch"
    );
    // Unknown field refused
    assert_eq!(
        refused(&json!({
            "jurisdiction": "vic",
            "scheme": "construction",
            "year": "2026-27",
            "scope_confirmed": true,
            "ordinary_wages": "50000.00",
            "not_an_input": true
        })),
        "input.field_unknown"
    );
}
