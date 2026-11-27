// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `salary-dividend`, `sole-trader-set-aside`, `salary-packaging` and
//! `novated-lease` against every case of the site pages they mirror
//! (ryanduguid.github.io `_data/calculator_cases.json` and
//! `_data/individual_tax_cases.json` at d876df72, read on 26 November 2026,
//! copied to `tests/data/owner_pay_page_cases.json`). A case in a year before
//! 2024-25 is refused, as the individual tax worksheets hold no earlier year;
//! the cases whose grossed-up FBT amounts the site works from the ATO's 4-place
//! gross-up rates differ where FBTAA s 5B(1F) and (1G)'s formulas give
//! another dollar, and are checked against the formulas.

use australian_tax_calculators::api;
use serde_json::{Map, Value, json};

const PAGES: &str = include_str!("data/owner_pay_page_cases.json");

fn pages() -> Value {
    serde_json::from_str(PAGES).expect("page cases parse")
}

/// The site's field names for each command's: (site, command).
const SALARY_DIVIDEND_IN: [(&str, &str); 5] = [
    ("year", "year"),
    ("profit", "profit"),
    ("baseRate", "base_rate_entity"),
    ("salary", "salary"),
    ("other", "other_taxable_income"),
];
const SALARY_DIVIDEND_OUT: [(&str, &str); 13] = [
    ("superGuarantee", "super_guarantee"),
    ("companyTax", "company_tax"),
    ("dividend", "dividend"),
    ("credit", "franking_credit"),
    ("ownerTax", "owner_tax"),
    ("superTax", "super_tax"),
    ("cash", "cash_kept"),
    ("superNet", "super_kept"),
    ("kept", "kept"),
    ("allSalary", "all_salary_kept"),
    ("allDividends", "all_dividends_kept"),
    ("bestSalary", "best_salary"),
    ("bestKept", "best_kept"),
];
const SOLE_TRADER_IN: [(&str, &str); 12] = [
    ("year", "year"),
    ("income", "business_income"),
    ("expenses", "business_expenses"),
    ("wages", "wages"),
    ("deductions", "other_deductions"),
    ("super", "personal_super_deducted"),
    ("offset", "small_business_offset"),
    ("studyLoan", "study_loan"),
    ("hospitalCover", "hospital_cover"),
    ("gst", "gst_registered"),
    ("credits", "gst_credits"),
    ("invoice", "invoice"),
];
const SOLE_TRADER_OUT: [(&str, &str); 26] = [
    ("profit", "profit"),
    ("deduction", "standard_deduction"),
    ("taxable", "taxable_income"),
    ("tax", "basic_income_tax"),
    ("offset", "low_income_tax_offset"),
    ("smallBusinessOffset", "small_business_income_tax_offset"),
    ("medicare", "medicare_levy"),
    ("surcharge", "medicare_levy_surcharge"),
    ("studyLoan", "study_loan_repayment"),
    ("total", "total"),
    ("wagesTax", "wages_alone_total"),
    ("businessTax", "business_tax"),
    ("gstOnIncome", "gst_on_income"),
    ("credits", "gst_credits"),
    ("gst", "gst"),
    ("setAside", "set_aside"),
    ("taxShare", "tax_share_percent"),
    ("invoiceShare", "invoice_share_percent"),
    ("quarterly", "quarterly"),
    ("monthly", "monthly"),
    ("weekly", "weekly"),
    ("invoice.amount", "invoice"),
    ("invoice.setAside", "invoice_set_aside"),
    ("invoice.left", "invoice_left"),
    ("instalments", "payg_instalments_likely"),
    ("mustRegister", "gst_registration_required"),
];
const PACKAGING_IN: [(&str, &str); 10] = [
    ("year", "year"),
    ("residency", "residency"),
    ("employer", "employer"),
    ("salary", "salary"),
    ("typeTwo", "type_two_benefits"),
    ("typeOne", "type_one_benefits"),
    ("meals", "meal_entertainment"),
    ("mealsTypeOne", "meal_entertainment_type_one"),
    ("studyLoan", "study_loan"),
    ("hospitalCover", "hospital_cover"),
];
const PACKAGING_OUT: [(&str, &str); 20] = [
    ("cap", "cap"),
    ("packaged", "packaged"),
    ("grossedTypeOne", "grossed_type_one"),
    ("grossedTypeTwo", "grossed_type_two"),
    ("grossedMeals", "grossed_meal_entertainment"),
    ("grossed", "grossed_total"),
    ("mealAllowance", "meal_entertainment_allowance"),
    ("excess", "excess"),
    ("fbt", "fbt"),
    ("reportable", "reportable_fringe_benefits"),
    ("taxableBefore", "taxable_income_before"),
    ("taxableAfter", "taxable_income_after"),
    ("owedBefore", "owed_before"),
    ("owedAfter", "owed_after"),
    ("surchargeAfter", "medicare_levy_surcharge_after"),
    ("studyLoanAfter", "study_loan_repayment_after"),
    ("saved", "tax_saved"),
    ("gain", "gain"),
    ("room", "cap_room"),
    ("roomTypeTwo", "cap_room_type_two"),
];
const LEASE_IN: [(&str, &str); 16] = [
    ("year", "year"),
    ("residency", "residency"),
    ("salary", "salary"),
    ("studyLoan", "study_loan"),
    ("hospitalCover", "hospital_cover"),
    ("kind", "car"),
    ("firstUsed", "first_used"),
    ("commitment", "commitment"),
    ("saleYear", "first_sale_year"),
    ("price", "cost_price"),
    ("otherSupply", "other_lct_supply"),
    ("lctTiming", "lct_timing"),
    ("reduced", "one_third_reduction"),
    ("days", "days_available"),
    ("gstCosts", "costs_with_gst"),
    ("otherCosts", "costs_without_gst"),
];
const LEASE_OUT: [(&str, &str); 17] = [
    ("exempt", "exempt"),
    ("threshold", "fuel_efficient_threshold"),
    ("taxableValue", "amounts.taxable_value"),
    ("costs", "amounts.costs"),
    ("netCosts", "amounts.net_costs"),
    ("owedWithout", "amounts.owed_without"),
    ("exemptCase.preTax", "exempt_case.pre_tax"),
    ("exemptCase.postTax", "exempt_case.post_tax"),
    (
        "exemptCase.reportable",
        "exempt_case.reportable_fringe_benefits",
    ),
    ("exemptCase.owed", "exempt_case.owed"),
    ("exemptCase.saving", "exempt_case.saving"),
    ("contribution.preTax", "contribution_case.pre_tax"),
    ("contribution.postTax", "contribution_case.post_tax"),
    (
        "contribution.reportable",
        "contribution_case.reportable_fringe_benefits",
    ),
    ("contribution.owed", "contribution_case.owed"),
    ("contribution.saving", "contribution_case.saving"),
    ("reason", "reason"),
];

/// The command's input from the page's call.
fn input(call: &Value, names: &[(&str, &str)]) -> Value {
    let mut out = Map::new();
    out.insert("scope_confirmed".into(), json!(true));
    for (site, ours) in names.iter().filter(|(_, ours)| !ours.is_empty()) {
        let value = match (&call[*site], *site) {
            (Value::String(text), "residency") if text == "foreign" => json!("foreign_resident"),
            (Value::String(text), "lctTiming") => json!(text.replace('-', "_")),
            (Value::Null, "lctTiming") => Value::Null,
            (Value::String(text), "days") => json!(text.parse::<i64>().unwrap()),
            (value, _) => value.clone(),
        };
        out.insert((*ours).into(), value);
    }
    Value::Object(out)
}

fn at<'a>(value: &'a Value, path: &str) -> &'a Value {
    path.split('.').fold(value, |v, key| &v[key])
}

/// A page figure and the command's, as the same decimal exactly, or as given.
fn same(page: &Value, ours: &Value) -> bool {
    let decimal = |v: &Value| {
        let text = match v {
            Value::Number(n) => n.to_string(),
            Value::String(s) if s.parse::<f64>().is_ok() => s.clone(),
            _ => return None,
        };
        let text = if text.contains('.') {
            text.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            text
        };
        Some(if text == "-0" { "0".to_string() } else { text })
    };
    match (decimal(page), decimal(ours)) {
        (Some(a), Some(b)) => a == b,
        _ => page == ours,
    }
}

/// The command's figure for a page field: its own amounts, or a sum.
fn ours(result: &Value, key: &str) -> Value {
    if key == "super_tax" {
        let a = &result["amounts"];
        let cents: i64 = ["contributions_tax", "division_293_tax"]
            .iter()
            .map(|k| {
                a[*k]
                    .as_str()
                    .unwrap()
                    .replace('.', "")
                    .parse::<i64>()
                    .unwrap()
            })
            .sum();
        return json!(format!("{}.{:02}", cents / 100, cents % 100));
    }
    if key.contains('.') || !result["amounts"].get(key).is_some() {
        return at(result, key).clone();
    }
    result["amounts"][key].clone()
}

/// Each page case: refused alike, or the same figure for every field, except
/// the fields `differ` names for that case.
fn agree(
    page: &str,
    command: &str,
    names: &[(&str, &str)],
    fields: &[(&str, &str)],
    differ: &[(&str, &[&str])],
) -> Vec<String> {
    let mut wrong = Vec::new();
    let pages = pages();
    for row in pages[page]["rows"].as_array().unwrap() {
        let case = row["case"].as_str().unwrap();
        let call = &row["call"][0];
        let got = api::evaluate_coded(command, &input(call, names));
        let year = call["year"].as_str().unwrap();
        if year < "2024-25" {
            match got {
                Err(r) if r.code == "period.unsupported" => continue,
                other => wrong.push(format!("{case}: {year} not refused: {other:?}")),
            }
            continue;
        }
        let expected = &row["result"];
        match (expected.is_null(), got) {
            (true, Err(_)) => {}
            (true, Ok(_)) => wrong.push(format!("{case}: the page refuses, the command does not")),
            (false, Err(r)) => wrong.push(format!("{case}: refused {} {}", r.code, r.message)),
            (false, Ok(result)) => {
                let skip: &[&str] = differ
                    .iter()
                    .find(|(c, _)| *c == case)
                    .map_or(&[], |(_, f)| f);
                for (site, key) in fields.iter().filter(|(s, _)| !skip.contains(s)) {
                    let (page_value, our_value) = (at(expected, site), ours(&result, key));
                    let ok = match *site {
                        "reason" => page_value.is_null() == our_value.is_null(),
                        _ if page_value.is_null() => our_value.is_null(),
                        _ => same(page_value, &our_value),
                    };
                    if !ok {
                        wrong.push(format!("{case}: {site} page {page_value} ours {our_value}"));
                    }
                }
            }
        }
    }
    wrong
}

#[test]
fn salary_dividend_agrees_with_every_page_case() {
    let wrong = agree(
        "salary-dividend",
        "salary-dividend",
        &SALARY_DIVIDEND_IN,
        &SALARY_DIVIDEND_OUT,
        &[],
    );
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn sole_trader_set_aside_agrees_with_every_page_case() {
    let wrong = agree(
        "sole-trader",
        "sole-trader-set-aside",
        &SOLE_TRADER_IN,
        &SOLE_TRADER_OUT,
        &[],
    );
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The page's figures from the ATO's 4-place gross-up rates where the
/// formulas of FBTAA s 5B(1F) and (1G) differ: checked in
/// `the_formulas_decide_where_the_rates_differ`.
const GROSS_UP_DIFFERS: [(&str, &[&str]); 3] = [
    (
        "A cent more is $1 over the cap and $0.47 of FBT",
        &["grossedTypeTwo", "grossed", "excess", "fbt", "gain"],
    ),
    ("Benefits of $2,000 are not reportable", &["roomTypeTwo"]),
    (
        "A cent more is reportable as $3,773, as the ATO’s threshold table says",
        &["roomTypeTwo"],
    ),
];

#[test]
fn salary_packaging_agrees_with_every_page_case() {
    let wrong = agree(
        "salary-packaging",
        "salary-packaging",
        &PACKAGING_IN,
        &PACKAGING_OUT,
        &GROSS_UP_DIFFERS,
    );
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn novated_lease_agrees_with_every_page_case() {
    let wrong = agree("novated-lease", "novated-lease", &LEASE_IN, &LEASE_OUT, &[]);
    assert!(wrong.is_empty(), "{wrong:#?}");
}

// The site's unit tests at d876df72 (scripts/business-calculators.test.mjs
// and scripts/individual-tax.test.mjs) for the years the commands cover.

fn run(command: &str, base: &Value, changes: Value) -> Result<Value, String> {
    let mut input = base.clone();
    for (k, v) in changes.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate_coded(command, &input).map_err(|r| r.code.to_string())
}

fn amount(out: &Value, key: &str) -> String {
    out["amounts"][key].as_str().unwrap().to_string()
}

fn owner() -> Value {
    json!({"year": "2026-27", "profit": "200000", "base_rate_entity": true, "salary": "100000",
        "other_taxable_income": "0", "scope_confirmed": true})
}

#[test]
fn salary_dividend_unit_cases() {
    let pay = |c: Value| run("salary-dividend", &owner(), c).unwrap();
    let r = pay(json!({"profit": "600000", "salary": "270830"}));
    assert_eq!(amount(&r, "division_293_tax"), "4874.94");
    assert_eq!(
        amount(
            &pay(json!({"profit": "600000", "salary": "300000"})),
            "super_guarantee"
        ),
        "32499.60"
    );
    let year = |y: &str, p: &str, s: &str| {
        amount(
            &pay(json!({"year": y, "profit": p, "salary": s})),
            "super_guarantee",
        )
    };
    assert_eq!(year("2025-26", "400000", "260000"), "30000.00");
    assert_eq!(year("2024-25", "200000", "100000"), "11500.00");
    // $40,000 of other income alone: the owner pays only the extra tax.
    let other = pay(json!({"profit": "50000", "salary": "0", "other_taxable_income": "40000"}));
    let got = [
        amount(&other, "taxable_income"),
        amount(&other, "owner_tax"),
        amount(&other, "cash_kept"),
    ];
    assert_eq!(got, ["90000.00", "3325.00", "34175.00"]);
    // At 30%, a $70,000.01 dividend carries 3/7 of it, $30,000.0043, as $30,000.00.
    let odd = pay(json!({"profit": "100000.01", "base_rate_entity": false, "salary": "0"}));
    let got = [
        amount(&odd, "company_tax"),
        amount(&odd, "dividend"),
        amount(&odd, "franking_credit"),
    ];
    assert_eq!(got, ["30000.00", "70000.01", "30000.00"]);
    let refused = |c: Value| run("salary-dividend", &owner(), c).unwrap_err();
    assert_eq!(
        refused(json!({"salary": "178571.44"})),
        "salary_dividend.salary_too_large"
    );
    assert_eq!(
        refused(json!({"base_rate_entity": "yes"})),
        "input.flag_invalid"
    );
    assert_eq!(refused(json!({"year": "2023-24"})), "period.unsupported");
}

#[test]
fn the_best_salary_keeps_no_less_than_the_others() {
    let facts = [
        ("2026-27", "30000", true, "0"),
        ("2025-26", "24000", true, "0"),
        ("2024-25", "38000", true, "12000"),
        ("2026-27", "900000", false, "50000"),
    ];
    for (year, profit, base, other) in facts {
        for paid in ["0", "5000", "20000"] {
            let facts = json!({"year": year, "profit": profit, "base_rate_entity": base, "salary": paid,
                "other_taxable_income": other});
            let r = run("salary-dividend", &owner(), facts.clone()).unwrap();
            let get = |k: &str| amount(&r, k).parse::<f64>().unwrap();
            for kept in ["all_salary_kept", "all_dividends_kept", "kept"] {
                assert!(
                    get("best_kept") >= get(kept) - 0.005,
                    "{year} {profit} {paid} {kept}"
                );
            }
            let mut best = facts;
            best["salary"] = json!(amount(&r, "best_salary"));
            assert_eq!(
                amount(&run("salary-dividend", &owner(), best).unwrap(), "kept"),
                amount(&r, "best_kept")
            );
        }
    }
}

fn sole() -> Value {
    json!({"year": "2026-27", "business_income": "22000", "business_expenses": "2000", "wages": "60000",
        "other_deductions": "0", "personal_super_deducted": "0", "small_business_offset": true, "study_loan": false,
        "hospital_cover": true, "gst_registered": false, "gst_credits": "0", "invoice": "1000", "scope_confirmed": true})
}

#[test]
fn sole_trader_unit_cases() {
    let set = |c: Value| run("sole-trader-set-aside", &sole(), c).unwrap();
    let r = set(json!({}));
    let got: Vec<String> = [
        "taxable_income",
        "basic_income_tax",
        "small_business_income_tax_offset",
        "medicare_levy",
        "total",
    ]
    .iter()
    .map(|k| amount(&r, k))
    .collect();
    assert_eq!(
        got,
        ["79000.00", "14220.00", "576.00", "1580.00", "15224.00"]
    );
    assert_eq!(amount(&r, "wages_alone_total"), "9285.00");
    let capped = set(
        json!({"year": "2024-25", "business_income": "29000", "business_expenses": "0", "wages": "131584"}),
    );
    assert_eq!(
        [
            amount(&capped, "basic_income_tax"),
            amount(&capped, "small_business_income_tax_offset")
        ],
        ["40754.08", "1000.00"]
    );
    assert_eq!(
        amount(
            &set(json!({"small_business_offset": false})),
            "small_business_income_tax_offset"
        ),
        "0.00"
    );
    let deduction = |c: Value| amount(&set(c), "standard_deduction");
    assert_eq!(deduction(json!({"wages": "0"})), "0.00");
    assert_eq!(deduction(json!({"wages": "600.50"})), "600.50");
    assert_eq!(deduction(json!({"year": "2025-26"})), "0.00");
    assert_eq!(
        amount(
            &set(json!({"other_deductions": "2000"})),
            "wages_alone_total"
        ),
        "8615.00"
    );
    let rival = set(
        json!({"business_income": "20000", "business_expenses": "0", "wages": "70000", "small_business_offset": false}),
    );
    let got = [
        amount(&rival, "business_tax"),
        amount(&rival, "tax_share_percent"),
        amount(&rival, "quarterly"),
        amount(&rival, "weekly"),
    ];
    assert_eq!(got, ["6400.00", "32.00", "1600.00", "123.08"]);
    let super_facts = json!({"year": "2025-26", "business_income": "110000", "business_expenses": "0", "wages": "0",
        "personal_super_deducted": "10000", "study_loan": true, "hospital_cover": false});
    let r = set(super_facts);
    let got: Vec<String> = [
        "taxable_income",
        "basic_income_tax",
        "medicare_levy_surcharge",
        "study_loan_repayment",
        "total",
    ]
    .iter()
    .map(|k| amount(&r, k))
    .collect();
    assert_eq!(
        got,
        ["100000.00", "20788.00", "1000.00", "6450.00", "29238.00"]
    );
    // $80,000 of contributions against $20,000 of wages: the wages alone deduct $20,000 and repay nothing.
    let r = set(
        json!({"year": "2025-26", "business_income": "150000", "business_expenses": "0", "wages": "20000",
        "personal_super_deducted": "80000", "study_loan": true}),
    );
    assert_eq!(
        [
            amount(&r, "taxable_income"),
            amount(&r, "wages_alone_total")
        ],
        ["90000.00", "0.00"]
    );
}

#[test]
fn sole_trader_gst_and_thresholds() {
    let set = |c: Value| run("sole-trader-set-aside", &sole(), c).unwrap();
    let r = set(
        json!({"business_income": "90000", "business_expenses": "15000", "wages": "0", "gst_registered": true,
        "gst_credits": "1200", "invoice": "1100"}),
    );
    let keys = [
        "gst_on_income",
        "gst",
        "business_tax",
        "set_aside",
        "invoice_share_percent",
        "invoice_set_aside",
        "invoice_left",
        "quarterly",
        "monthly",
        "weekly",
    ];
    let got: Vec<String> = keys.iter().map(|k| amount(&r, k)).collect();
    let want = [
        "9000.00", "7800.00", "13520.00", "21320.00", "21.54", "236.89", "863.11", "5330.00",
        "1776.67", "410.00",
    ];
    assert_eq!(got, want);
    let tiny = set(
        json!({"business_income": "0.05", "business_expenses": "0", "wages": "0", "gst_registered": true}),
    );
    assert_eq!(amount(&tiny, "gst_on_income"), "0.01");
    let flag = |c: Value, key: &str| set(c)[key].as_bool().unwrap();
    assert!(flag(
        json!({"business_income": "75000", "business_expenses": "0"}),
        "gst_registration_required"
    ));
    assert!(!flag(
        json!({"business_income": "74999.99", "business_expenses": "0"}),
        "gst_registration_required"
    ));
    assert!(!flag(
        json!({"business_income": "75000", "business_expenses": "0", "gst_registered": true}),
        "gst_registration_required"
    ));
    assert!(flag(
        json!({"business_income": "4000", "business_expenses": "0"}),
        "payg_instalments_likely"
    ));
    assert!(!flag(
        json!({"business_income": "3999.99", "business_expenses": "0"}),
        "payg_instalments_likely"
    ));
}

#[test]
fn sole_trader_refusals() {
    let refused = |c: Value| run("sole-trader-set-aside", &sole(), c).unwrap_err();
    assert_eq!(
        refused(json!({"business_expenses": "22000.01"})),
        "sole_trader_set_aside.business_loss"
    );
    let too_much = "sole_trader_set_aside.deductions_exceed_income";
    assert_eq!(
        refused(json!({"wages": "0", "other_deductions": "20000.01"})),
        too_much
    );
    assert_eq!(
        refused(
            json!({"wages": "0", "other_deductions": "10000", "personal_super_deducted": "10000.01"})
        ),
        too_much
    );
    assert_eq!(
        refused(json!({"gst_credits": "1"})),
        "sole_trader_set_aside.credits_unregistered"
    );
    assert_eq!(
        refused(json!({"gst_registered": true, "gst_credits": "2200.01"})),
        "sole_trader_set_aside.gst_refund"
    );
    assert_eq!(
        refused(json!({"gst_registered": "yes"})),
        "input.flag_invalid"
    );
    assert_eq!(refused(json!({"year": "2027-28"})), "period.unsupported");
    assert_eq!(
        refused(json!({"year": "2024-25", "study_loan": true})),
        "period.unsupported"
    );
    for value in ["", "-1", "1.001", "1e5"] {
        assert!(
            refused(json!({"business_income": value})).starts_with("input."),
            "{value}"
        );
    }
}

fn package() -> Value {
    json!({"year": "2026-27", "residency": "resident", "employer": "charity", "salary": "80000", "type_two_benefits": "0",
        "type_one_benefits": "0", "meal_entertainment": "0", "meal_entertainment_type_one": true, "study_loan": false,
        "hospital_cover": true, "scope_confirmed": true})
}

/// Fringe benefits tax - a guide for employers, 6.10: a health promotion
/// charity employee with a $7,700 Type 1 car, $9,000 of Type 2 benefits and
/// $3,300 of Type 2 meal entertainment.
#[test]
fn the_atos_salary_packaging_example() {
    let facts = json!({"type_one_benefits": "7700", "type_two_benefits": "9000", "meal_entertainment": "3300",
        "meal_entertainment_type_one": false});
    let r = run("salary-packaging", &package(), facts).unwrap();
    let keys = [
        "grossed_type_one",
        "grossed_type_two",
        "grossed_total",
        "meal_entertainment_allowance",
        "excess",
        "fbt",
    ];
    let got: Vec<String> = keys.iter().map(|k| amount(&r, k)).collect();
    assert_eq!(
        got,
        [
            "16018.00", "23208.00", "39226.00", "5000.00", "4226.00", "1986.22"
        ]
    );
}

/// FBTAA s 5B(1G): $15,900.21 / 0.53 is $30,000.40 and $9,010.23 / 0.53 is
/// $17,000.43, each within its cap to the dollar, where the ATO's 1.8868
/// gives $30,000.62 and $17,000.50; a dollar over takes $15,900.27.
#[test]
fn the_formulas_decide_where_the_rates_differ() {
    let pack = |c: Value| run("salary-packaging", &package(), c).unwrap();
    let within = pack(json!({"type_two_benefits": "15900.21"}));
    let got = [
        amount(&within, "grossed_type_two"),
        amount(&within, "excess"),
        amount(&within, "fbt"),
    ];
    assert_eq!(got, ["30000.00", "0.00", "0.00"]);
    assert_eq!(
        amount(&pack(json!({"type_two_benefits": "15900.26"})), "excess"),
        "0.00"
    );
    let over = pack(json!({"type_two_benefits": "15900.27"}));
    assert_eq!(
        [amount(&over, "excess"), amount(&over, "fbt")],
        ["1.00", "0.47"]
    );
    assert_eq!(
        amount(
            &pack(json!({"employer": "hospital", "type_two_benefits": "9010.23"})),
            "excess"
        ),
        "0.00"
    );
    assert_eq!(
        amount(
            &pack(json!({"employer": "hospital", "type_two_benefits": "9010.27"})),
            "excess"
        ),
        "1.00"
    );
    // $28,000 of the cap left takes $28,000 x 0.53 = $14,840.00 of Type 2 benefits.
    assert_eq!(
        amount(
            &pack(json!({"type_two_benefits": "1060"})),
            "cap_room_type_two"
        ),
        "14840.00"
    );
}

#[test]
fn salary_packaging_unit_cases() {
    let pack = |c: Value| run("salary-packaging", &package(), c).unwrap();
    assert_eq!(
        amount(&pack(json!({"type_two_benefits": "15900.20"})), "excess"),
        "0.00"
    );
    assert_eq!(
        amount(
            &pack(json!({"employer": "hospital", "type_two_benefits": "9009.96"})),
            "excess"
        ),
        "0.00"
    );
    assert_eq!(
        amount(
            &pack(json!({"type_one_benefits": "1000"})),
            "grossed_type_one"
        ),
        "2080.00"
    );
    let within = pack(json!({"type_two_benefits": "15899.93", "meal_entertainment": "2403.61"}));
    let got = [
        amount(&within, "grossed_meal_entertainment"),
        amount(&within, "meal_entertainment_allowance"),
        amount(&within, "fbt"),
    ];
    assert_eq!(got, ["5000.00", "5000.00", "0.00"]);
    let over = pack(json!({"type_two_benefits": "20000", "meal_entertainment": "3000"}));
    assert_eq!(
        [
            amount(&over, "meal_entertainment_allowance"),
            amount(&over, "excess")
        ],
        ["5000.00", "8977.00"]
    );
    assert_eq!(
        amount(
            &pack(json!({"meal_entertainment": "3000"})),
            "meal_entertainment_allowance"
        ),
        "0.00"
    );
    let reportable = |c: Value| amount(&pack(c), "reportable_fringe_benefits");
    assert_eq!(reportable(json!({"type_two_benefits": "2000"})), "0.00");
    assert_eq!(
        reportable(json!({"type_two_benefits": "2000.01"})),
        "3773.00"
    );
    assert_eq!(
        reportable(json!({"type_two_benefits": "15899.93"})),
        "29999.00"
    );
    assert_eq!(
        reportable(json!({"type_one_benefits": "3000", "meal_entertainment": "2000"})),
        "9433.00"
    );
    let more = |c: Value| {
        amount(&pack(c), "medicare_levy_surcharge_after")
            .parse::<f64>()
            .unwrap()
    };
    let r = pack(
        json!({"salary": "100000", "type_two_benefits": "15899.93", "study_loan": true, "hospital_cover": false}),
    );
    assert!(
        amount(&r, "study_loan_repayment_after")
            .parse::<f64>()
            .unwrap()
            > 0.0
    );
    assert!(
        more(json!({"salary": "100000", "type_two_benefits": "15899.93", "hospital_cover": false}))
            > 0.0
    );
    assert!(more(json!({"salary": "100000", "type_two_benefits": "15899.93"})) == 0.0);
    assert_eq!(
        amount(&pack(json!({})), "taxable_income_before"),
        "79000.00"
    );
    assert_eq!(
        amount(&pack(json!({"year": "2025-26"})), "taxable_income_before"),
        "80000.00"
    );
    assert_eq!(
        amount(
            &pack(json!({"residency": "foreign_resident"})),
            "taxable_income_before"
        ),
        "80000.00"
    );
}

#[test]
fn salary_packaging_refusals() {
    let refused = |c: Value| run("salary-packaging", &package(), c).unwrap_err();
    assert_eq!(
        refused(json!({"residency": "working_holiday"})),
        "salary_packaging.residency_invalid"
    );
    assert_eq!(
        refused(json!({"employer": "rebatable"})),
        "salary_packaging.employer_invalid"
    );
    assert_eq!(refused(json!({"study_loan": "yes"})), "input.flag_invalid");
    assert_eq!(
        refused(json!({"salary": "1000", "type_two_benefits": "1000.01"})),
        "salary_packaging.packaged_exceeds_salary"
    );
    assert!(refused(json!({"type_two_benefits": "-1"})).starts_with("input."));
    assert!(refused(json!({"salary": "1000.50"})).starts_with("input."));
    assert_eq!(
        refused(json!({"residency": "foreign_resident", "study_loan": true})),
        "estimate.residency_unsupported"
    );
    assert_eq!(refused(json!({"year": "2023-24"})), "period.unsupported");
}

fn car() -> Value {
    json!({"year": "2026-27", "residency": "resident", "salary": "100000", "study_loan": false, "hospital_cover": true,
        "car": "bev", "first_used": "2022-07-01", "commitment": false, "first_sale_year": "2026-27", "cost_price": "60000",
        "other_lct_supply": false, "lct_timing": null, "one_third_reduction": false, "days_available": 365,
        "costs_with_gst": "15000", "costs_without_gst": "1000", "scope_confirmed": true})
}

fn lease(c: Value) -> Result<Value, String> {
    run("novated-lease", &car(), c)
}

fn exempt(c: Value) -> bool {
    lease(c).unwrap()["exempt"].as_bool().unwrap()
}

fn reason(c: Value) -> String {
    lease(c).unwrap()["reason"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn novated_lease_tests_each_condition_of_the_exemption() {
    assert!(exempt(json!({})) && exempt(json!({"car": "fcev"})));
    assert!(reason(json!({"car": "other"})).contains("not a battery electric"));
    assert!(reason(json!({"car": "phev"})).contains("only under a commitment"));
    assert!(exempt(
        json!({"car": "phev", "commitment": true, "first_sale_year": "2024-25"})
    ));
    let hybrid = "novated_lease.hybrid_commitment";
    assert_eq!(
        lease(json!({"car": "phev", "commitment": true, "first_sale_year": "2025-26"}))
            .unwrap_err(),
        hybrid
    );
    let late = json!({"car": "phev", "commitment": true, "first_used": "2025-04-01", "first_sale_year": "2024-25"});
    assert_eq!(lease(late).unwrap_err(), hybrid);
    assert!(exempt(
        json!({"car": "phev", "year": "2024-25", "first_sale_year": "2024-25"})
    ));
    assert!(
        reason(json!({"first_used": "2022-06-30", "first_sale_year": "2021-22"}))
            .contains("before 1 July 2022")
    );
    assert!(exempt(
        json!({"cost_price": "91661", "costs_with_gst": "25000"})
    ));
    let dear = json!({"cost_price": "91661.01", "costs_with_gst": "25000", "days_available": 274, "lct_timing": "before_all"});
    assert!(reason(dear).contains("first retail sale"));
    assert!(!exempt(
        json!({"first_sale_year": "2025-26", "cost_price": "91387.01", "costs_with_gst": "25000",
        "lct_timing": "before_all"})
    ));
    assert!(exempt(
        json!({"first_sale_year": "2025-26", "cost_price": "91387", "costs_with_gst": "25000"})
    ));
    assert!(
        reason(json!({"other_lct_supply": true, "lct_timing": "before_all"}))
            .contains("another supply")
    );
}

/// A taxable first retail sale on 1 April 2026 follows every benefit in the
/// FBT year ending 31 March 2026.
fn sale(extra: Value) -> Value {
    let mut c = json!({"year": "2025-26", "first_sale_year": "2025-26", "first_used": "2025-04-01", "cost_price": "100000",
        "costs_with_gst": "25000"});
    for (k, v) in extra.as_object().unwrap() {
        c[k] = v.clone();
    }
    c
}

#[test]
fn novated_lease_applies_luxury_car_tax_at_the_time_of_the_benefits() {
    let timing = "novated_lease.lct_timing_invalid";
    assert!(exempt(sale(json!({"lct_timing": "after_all"}))));
    assert!(!exempt(sale(
        json!({"days_available": 274, "lct_timing": "before_all"})
    )));
    assert_eq!(
        lease(sale(
            json!({"days_available": 275, "lct_timing": "before_all"})
        ))
        .unwrap_err(),
        timing
    );
    assert_eq!(lease(sale(json!({}))).unwrap_err(), timing);
    assert_eq!(
        lease(sale(json!({"lct_timing": "mixed_or_unknown"}))).unwrap_err(),
        timing
    );
    assert!(exempt(
        json!({"other_lct_supply": true, "lct_timing": "after_all"})
    ));
    assert_eq!(
        lease(json!({"other_lct_supply": true, "lct_timing": "mixed_or_unknown"})).unwrap_err(),
        timing
    );
    assert!(!exempt(sale(json!({"first_sale_year": "2023-24"}))));
    assert_eq!(
        lease(sale(
            json!({"first_sale_year": "2023-24", "lct_timing": "after_all"})
        ))
        .unwrap_err(),
        timing
    );
    let overlapping = |extra: Value| {
        let mut c = sale(json!({"first_sale_year": "2024-25", "lct_timing": "after_all"}));
        for (k, v) in extra.as_object().unwrap() {
            c[k] = v.clone();
        }
        c
    };
    assert!(exempt(overlapping(json!({"days_available": 91}))));
    assert_eq!(
        lease(overlapping(json!({"days_available": 92}))).unwrap_err(),
        timing
    );
    assert!(exempt(overlapping(
        json!({"first_used": "2025-07-01", "days_available": 1})
    )));
    assert!(exempt(sale(json!({"first_sale_year": "2026-27"}))));
    assert!(exempt(sale(
        json!({"first_sale_year": "2026-27", "lct_timing": "mixed_or_unknown"})
    )));
    assert_eq!(
        lease(sale(
            json!({"first_sale_year": "2026-27", "lct_timing": "before_all"})
        ))
        .unwrap_err(),
        timing
    );
    for c in [
        json!({"lct_timing": "before_all"}),
        json!({"lct_timing": "after_all"}),
        json!({"car": "other", "lct_timing": "before_all"}),
    ] {
        assert_eq!(lease(c).unwrap_err(), timing);
    }
    assert!(
        reason(sale(
            json!({"other_lct_supply": true, "lct_timing": "before_all"})
        ))
        .contains("before the benefits")
    );
}

#[test]
fn novated_lease_values_and_splits_the_costs() {
    let value = |c: Value| amount(&lease(c).unwrap(), "taxable_value");
    assert_eq!(value(json!({})), "12000.00");
    assert_eq!(value(json!({"one_third_reduction": true})), "8000.00");
    assert_eq!(value(json!({"days_available": 182})), "5983.56");
    assert_eq!(value(json!({"days_available": 0})), "0.00");
    let boundary = json!({"year": "2025-26", "first_sale_year": "2025-26", "first_used": "2026-03-31", "days_available": 1});
    assert!(exempt(boundary.clone()));
    assert_eq!(value(boundary.clone()), "32.88");
    let with = |k: &str, v: Value| {
        let mut c = boundary.clone();
        c[k] = v;
        lease(c).unwrap_err()
    };
    assert_eq!(
        with("first_used", json!("2026-04-01")),
        "novated_lease.first_used_late"
    );
    assert!(with("days_available", json!(2)).starts_with("input."));
    assert!(with("first_used", json!("2026-02-29")).starts_with("input."));
    let r = lease(json!({})).unwrap();
    assert_eq!(amount(&r, "net_costs"), "14636.36");
    let case = |name: &str, key: &str| r[name][key].as_str().unwrap().to_string();
    let exempt_case = [
        "pre_tax",
        "post_tax",
        "reportable_fringe_benefits",
        "saving",
    ]
    .map(|k| case("exempt_case", k));
    assert_eq!(exempt_case, ["14636.36", "0.00", "22641.00", "6047.48"]);
    let contribution = [
        "pre_tax",
        "post_tax",
        "reportable_fringe_benefits",
        "saving",
    ]
    .map(|k| case("contribution_case", k));
    assert_eq!(contribution, ["3727.27", "12000.00", "0.00", "1465.69"]);
    assert_eq!(
        lease(json!({"car": "other"})).unwrap()["exempt_case"],
        Value::Null
    );
    assert_eq!(
        lease(json!({"days_available": 0})).unwrap()["exempt_case"]["reportable_fringe_benefits"],
        json!("0.00")
    );
}

#[test]
fn novated_lease_refusals() {
    let code = |c: Value| lease(c).unwrap_err();
    assert_eq!(
        code(json!({"year": "2023-24", "first_sale_year": "2023-24"})),
        "period.unsupported"
    );
    assert_eq!(code(json!({"car": "diesel"})), "novated_lease.car_invalid");
    assert!(code(json!({"first_used": "later"})).starts_with("input."));
    assert_eq!(
        code(json!({"first_sale_year": "2027-28"})),
        "novated_lease.sale_year_invalid"
    );
    assert_eq!(
        code(json!({"first_sale_year": "2020-21"})),
        "novated_lease.sale_year_invalid"
    );
    assert_eq!(
        code(json!({"cost_price": "0"})),
        "novated_lease.cost_price_nil"
    );
    assert_eq!(
        code(json!({"salary": "10000", "costs_with_gst": "15000"})),
        "novated_lease.costs_exceed_salary"
    );
    assert_eq!(
        code(json!({"car": "other", "costs_with_gst": "1000"})),
        "novated_lease.contribution_short"
    );
    assert_eq!(
        code(json!({"residency": "working_holiday"})),
        "novated_lease.residency_invalid"
    );
    assert_eq!(
        code(json!({"lct_timing": "sometime"})),
        "novated_lease.lct_timing_invalid"
    );
    assert_eq!(code(json!({"commitment": "no"})), "input.flag_invalid");
    assert!(code(json!({"days_available": 366})).starts_with("input."));
    assert!(code(json!({"salary": "100000.50"})).starts_with("input."));
}

/// With no business income there is no share of the tax or the invoice to
/// set aside: both percentages are nil rather than a division by nil.
#[test]
fn sole_trader_without_business_income_sets_aside_nil_shares() {
    let r = run(
        "sole-trader-set-aside",
        &sole(),
        json!({"business_income": "0", "business_expenses": "0", "invoice": "0"}),
    )
    .unwrap();
    assert_eq!(amount(&r, "tax_share_percent"), "0.00");
    assert_eq!(amount(&r, "invoice_share_percent"), "0.00");
}

/// An exempt car whose costs are below its taxable value has no after-tax
/// contribution case: the contribution would be more than the costs it is
/// paid from.
#[test]
fn novated_lease_has_no_contribution_case_when_costs_are_below_the_value() {
    let r = lease(json!({"costs_with_gst": "1100", "costs_without_gst": "0"})).unwrap();
    assert_eq!(r["exempt"], true);
    assert_eq!(r["contribution_case"], Value::Null);
    assert!(!r["exempt_case"].is_null());
}

/// A one-cent profit in 2026-27 pays at most a one-cent salary: its 0.12 cent
/// of super guarantee rounds to nil. Ranking that salary works a taxable income
/// just below nil (the standard deduction takes the whole cent) without error.
#[test]
fn a_one_cent_profit_pays_at_most_a_one_cent_salary() {
    let input = json!({
        "year": "2026-27", "profit": "0.01", "base_rate_entity": true, "salary": "0.01",
        "other_taxable_income": "0.00", "scope_confirmed": true,
    });
    let r = api::evaluate("salary-dividend", &input).unwrap();
    assert_eq!(r["amounts"]["largest_salary"], "0.01");
}
