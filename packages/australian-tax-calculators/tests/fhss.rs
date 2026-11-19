// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The First Home Super Saver scheme (TAA 1953 Sch 1 Division 138, ITAA 1997
//! Division 313 and TAA Regulations 2017 reg 53A, read on 20 November 2026)
//! against the Commissioner's GN 2024/1 examples, the ATO's Mary and George
//! examples, the s 138-30 example and each refusal
//! (docs/calculation-evidence.md#fhss).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("fhss", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("fhss", input).unwrap_err().code
}

/// `amount` on the 15th of each month from `from` (year, month) for `months`.
fn monthly(from: (i64, i64), months: i64, amount: &str, kind: &str) -> Vec<Value> {
    (0..months)
        .map(|i| {
            let index = from.0 * 12 + from.1 - 1 + i;
            let (y, m) = (index / 12, index % 12 + 1);
            json!({"date": format!("{y:04}-{m:02}-15"), "amount": amount, "kind": kind})
        })
        .collect()
}

fn estimate(contributions: Vec<Value>, requested: &str, determined: &str) -> Value {
    json!({
        "contributions": contributions, "determination_requested": requested,
        "determination_date": determined, "year": fy(determined), "scope_confirmed": true,
    })
}

/// The financial year of a `YYYY-MM-DD` date.
fn fy(date: &str) -> String {
    let (y, m): (i64, i64) = (date[..4].parse().unwrap(), date[5..7].parse().unwrap());
    let start = if m >= 7 { y } else { y - 1 };
    format!("{start}-{:02}", (start + 1) % 100)
}

fn stated(concessional: &str, non_concessional: &str, earnings: &str) -> Value {
    json!({
        "determination": {
            "concessional": concessional, "non_concessional": non_concessional,
            "associated_earnings": earnings,
        },
        "determination_date": "2025-06-20", "year": "2024-25", "scope_confirmed": true,
    })
}

fn years(r: &Value) -> Vec<(String, String, String)> {
    r["years"]
        .as_array()
        .unwrap()
        .iter()
        .map(|y| {
            (
                y["financial_year"].as_str().unwrap().into(),
                y["concessional_counted"].as_str().unwrap().into(),
                y["non_concessional_counted"].as_str().unwrap().into(),
            )
        })
        .collect()
}

fn year(fy: &str, cc: &str, ncc: &str) -> (String, String, String) {
    (fy.into(), cc.into(), ncc.into())
}

/// GN 2024/1 example 6 (Jill): $1,500 a month salary sacrifice from July 2020
/// to October 2024, determination 15 December 2024. The limits reach the
/// published $50,000 and $42,500. s 138-30(2) counts earliest first, so the
/// last $5,000 is 2023-24's, where the example puts it in 2024-25; and the
/// associated earnings, $7,859 on s 138-40 at the published SIC rates (an
/// independent day-by-day script gives the same), are not the example's
/// $5,190, so they are labelled an estimate.
#[test]
fn jill_reaches_the_limits_and_labels_earnings_an_estimate() {
    let input = estimate(
        monthly((2020, 7), 52, "1500", "employer_voluntary"),
        "2024-12-01",
        "2024-12-15",
    );
    let r = run(&input);
    assert_eq!(
        years(&r),
        vec![
            year("2020-21", "15000", "0"),
            year("2021-22", "15000", "0"),
            year("2022-23", "15000", "0"),
            year("2023-24", "5000", "0"),
            year("2024-25", "0", "0"),
        ]
    );
    assert_eq!(r["counted_concessional_in_full"], "50000");
    assert_eq!(r["amounts"]["concessional_component"], "42500.00");
    assert_eq!(r["amounts"]["non_concessional_component"], "0.00");
    assert_eq!(r["amounts"]["associated_earnings"], "7859.00");
    assert_eq!(r["amounts"]["maximum_release_amount"], "50359.00");
    assert_eq!(r["associated_earnings_basis"], "estimate");
    assert_eq!(r["period"], "2024-25");
    let warnings = r["warnings"].to_string();
    assert!(warnings.contains("$5,190"));
    // October 2023's $1,500 is counted only to the $50,000: $500.
    let rows = r["contributions"].as_array().unwrap();
    let october = rows.iter().find(|c| c["date"] == "2023-10-15").unwrap();
    assert_eq!(october["counted"], "500");
    assert_eq!(october["earnings_from"], "2023-10-01");
}

/// The ATO's 'About FHSS release amounts' (Mary): $25,000 of salary sacrifice
/// in 2023-24 counts $15,000, and $12,750 towards the maximum.
#[test]
fn mary_counts_15000_of_25000() {
    let one = vec![json!({"date": "2024-06-01", "amount": "25000", "kind": "employer_voluntary"})];
    let r = run(&estimate(one, "2024-08-01", "2024-08-01"));
    assert_eq!(years(&r), vec![year("2023-24", "15000", "0")]);
    assert_eq!(r["amounts"]["concessional_component"], "12750.00");
}

/// GN 2024/1 example 7 (Hasan): $1,000 salary sacrifice on the 15th of each
/// month from January 2023 and $5,000 personal on 30 June 2023 and 2024. In
/// 2023-24 the concessional contributions come first, so only $3,000 of the
/// $5,000 counts (Table 2).
#[test]
fn hasan_counts_the_earlier_contributions_first() {
    let mut list = monthly((2023, 1), 30, "1000", "employer_voluntary");
    for date in ["2023-06-30", "2024-06-30"] {
        list.push(json!({"date": date, "amount": "5000", "kind": "personal"}));
    }
    let r = run(&estimate(list, "2025-07-15", "2025-07-15"));
    assert_eq!(
        years(&r),
        vec![
            year("2022-23", "6000", "5000"),
            year("2023-24", "12000", "3000"),
            year("2024-25", "12000", "0"),
        ]
    );
    assert_eq!(r["amounts"]["non_concessional_component"], "8000.00");
    assert_eq!(r["amounts"]["concessional_component"], "25500.00");
}

/// A non-concessional contribution made at the same time as a concessional
/// one is counted first (s 138-30(3)(a)).
#[test]
fn a_simultaneous_non_concessional_contribution_counts_first() {
    let list = vec![
        json!({"date": "2024-03-01", "amount": "10000", "kind": "employer_voluntary"}),
        json!({"date": "2024-03-01", "amount": "10000", "kind": "personal"}),
    ];
    let r = run(&estimate(list, "2024-08-01", "2024-08-01"));
    assert_eq!(years(&r), vec![year("2023-24", "5000", "10000")]);
}

/// The s 138-30 example: $1,000 a fortnight to $25,000 in 2018-19, $15,000
/// deducted. The first 10 contributions are the non-concessional ones; with
/// the $15,000 limit the next 5 count as concessional.
#[test]
fn the_section_example_takes_the_first_10_as_non_concessional() {
    let list: Vec<Value> = (0..25)
        .map(|i| {
            let day = 186 + 14 * i; // 5 July 2018 onwards, as days from 1 January 2018
            let date = ymd(2018, day);
            json!({"date": date, "amount": "1000", "kind": "personal"})
        })
        .collect();
    let mut input = estimate(list, "2019-07-15", "2019-07-15");
    input["personal_deductions"] = json!([{"financial_year": "2018-19", "amount": "15000"}]);
    let r = run(&input);
    assert_eq!(years(&r), vec![year("2018-19", "5000", "10000")]);
    let rows = r["contributions"].as_array().unwrap();
    let treated: Vec<&str> = rows
        .iter()
        .map(|c| c["treated_as"].as_str().unwrap())
        .collect();
    assert!(treated[..10].iter().all(|t| *t == "non_concessional"));
    assert!(treated[10..].iter().all(|t| *t == "concessional"));
    assert_eq!(r["amounts"]["concessional_component"], "4250.00");
    // A determination requested before 1 July 2022 has the $30,000 limit.
    assert_eq!(r["total_limit"], "30000");
}

/// The date `day` days after 1 January of `y` (counting it as day 1).
fn ymd(y: i64, day: i64) -> String {
    let lengths = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let (mut m, mut d) = (0, day);
    while d > lengths[m % 12] {
        d -= lengths[m % 12];
        m += 1;
    }
    format!("{:04}-{:02}-{d:02}", y + (m / 12) as i64, m % 12 + 1)
}

/// Before 1 July 2022 the limit in all was $30,000 (GN 2024/1 section 3).
#[test]
fn a_request_before_july_2022_has_the_30000_limit() {
    let list = monthly((2018, 7), 36, "1250", "employer_voluntary");
    let r = run(&estimate(list.clone(), "2022-06-30", "2022-07-04"));
    assert_eq!(r["counted_concessional_in_full"], "30000");
    let r = run(&estimate(list, "2022-07-01", "2022-07-04"));
    assert_eq!(r["counted_concessional_in_full"], "45000");
}

/// GN 2024/1 example 11 (Sue): $14,350 released in full with no marginal rate
/// estimate: 17% of $9,350 is $1,589.50, withheld as $1,589; the offset is
/// $2,805. It is the catalogue's example.
#[test]
fn sue_has_17_percent_withheld() {
    let r = run(&stated("8500", "5000", "850"));
    assert_eq!(r["amounts"]["maximum_release_amount"], "14350.00");
    assert_eq!(r["amounts"]["assessable_fhss_released_amount"], "9350.00");
    assert_eq!(r["amounts"]["tax_withheld"], "1589.00");
    assert_eq!(r["amounts"]["paid_to_you"], "12761.00");
    assert_eq!(r["amounts"]["fhss_tax_offset"], "2805.00");
    assert_eq!(r["rates"]["withholding_rate"], "0.17");
    assert_eq!(r["associated_earnings_basis"], "stated");
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["fhss"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_eq!(run(&facts)["amounts"]["tax_withheld"], "1589.00");
}

/// GN 2024/1 example 12 (Alex): a 37% marginal rate estimate gives 9%
/// (37% plus 2% less 30%) of $8,500, $765, and pays $27,735.
#[test]
fn alex_has_9_percent_withheld() {
    let mut input = stated("6800", "20000", "1700");
    input["estimated_marginal_rate"] = json!("0.37");
    let r = run(&input);
    assert_eq!(r["amounts"]["maximum_release_amount"], "28500.00");
    assert_eq!(r["amounts"]["assessable_fhss_released_amount"], "8500.00");
    assert_eq!(r["amounts"]["tax_withheld"], "765.00");
    assert_eq!(r["amounts"]["paid_to_you"], "27735.00");
    assert_eq!(r["amounts"]["fhss_tax_offset"], "2550.00");
}

/// GN 2024/1 example 13 and the ATO's 'FHSS tax assessment' (George):
/// $26,100 released, $20,100 assessable, $1,809 withheld at 9%, $24,291 paid,
/// and $4,020 of FHSS tax if he keeps the money. The split of the $20,100
/// between concessional contributions and earnings is not published; any
/// split gives the same figures.
#[test]
fn george_pays_20_percent_if_he_keeps_the_money() {
    let mut input = stated("18000", "6000", "2100");
    input["estimated_marginal_rate"] = json!("0.37");
    let r = run(&input);
    assert_eq!(r["amounts"]["assessable_fhss_released_amount"], "20100.00");
    assert_eq!(r["amounts"]["tax_withheld"], "1809.00");
    assert_eq!(r["amounts"]["paid_to_you"], "24291.00");
    assert_eq!(r["amounts"]["fhss_tax_if_not_used"], "4020.00");
    assert_eq!(r["amounts"]["recontribution_required"], "18291.00");
}

/// s 313-20(2) and (3): releasing less than the maximum takes the shortfall
/// off the assessable amount, not below nil; a low marginal rate withholds
/// nothing.
#[test]
fn a_partial_release_reduces_the_assessable_amount() {
    let mut input = stated("8500", "5000", "850");
    input["amount_requested"] = json!("10000");
    let r = run(&input);
    assert_eq!(r["amounts"]["assessable_fhss_released_amount"], "5000.00");
    input["amount_requested"] = json!("4000");
    input["estimated_marginal_rate"] = json!("0.16");
    let r = run(&input);
    assert_eq!(r["amounts"]["assessable_fhss_released_amount"], "0.00");
    assert_eq!(r["rates"]["withholding_rate"], "0");
    assert_eq!(r["amounts"]["tax_withheld"], "0.00");
}

#[test]
fn refusals() {
    let ok = || {
        estimate(
            monthly((2023, 7), 3, "1000", "employer_voluntary"),
            "2024-08-01",
            "2024-08-01",
        )
    };
    let with = |key: &str, value: Value| {
        let mut input = ok();
        input[key] = value;
        input
    };
    let one = |c: Value| with("contributions", json!([c]));
    assert_eq!(
        code(&with("determination", json!({}))),
        "fhss.basis_invalid"
    );
    let mut neither = ok();
    neither.as_object_mut().unwrap().remove("contributions");
    assert_eq!(code(&neither), "fhss.basis_invalid");
    let sg = json!({"date": "2024-01-15", "amount": "1000", "kind": "super_guarantee"});
    assert_eq!(code(&one(sg)), "fhss.contribution_kind_invalid");
    let early = json!({"date": "2017-06-30", "amount": "1000", "kind": "personal"});
    assert_eq!(code(&one(early)), "fhss.contribution_out_of_period");
    let late = json!({"date": "2024-08-02", "amount": "1000", "kind": "personal"});
    assert_eq!(code(&one(late)), "fhss.contribution_out_of_period");
    let nil = json!({"date": "2024-01-15", "amount": "0", "kind": "personal"});
    assert_eq!(code(&one(nil)), "input.amount_invalid");
    let extra = json!({"date": "2024-01-15", "amount": "1", "kind": "personal", "fund": "x"});
    assert_eq!(code(&one(extra)), "input.field_unknown");
    assert_eq!(
        code(&with("contributions", json!([]))),
        "fhss.contributions_invalid"
    );
    let deduction = json!([{"financial_year": "2023-24", "amount": "1"}]);
    assert_eq!(
        code(&with("personal_deductions", deduction)),
        "fhss.deductions_invalid"
    );
    let bad_year = json!([{"financial_year": "2023-25", "amount": "0"}]);
    assert_eq!(
        code(&with("personal_deductions", bad_year)),
        "fhss.deductions_invalid"
    );
    let mut unrequested = ok();
    unrequested
        .as_object_mut()
        .unwrap()
        .remove("determination_requested");
    assert_eq!(code(&unrequested), "input.missing_field");
    let after = with("determination_requested", json!("2024-08-02"));
    assert_eq!(code(&after), "fhss.determination_requested_invalid");
    let date = |d: &str| {
        let mut input = with("determination_date", json!(d));
        input["year"] = json!(fy(d));
        input
    };
    assert_eq!(code(&date("2027-01-04")), "fhss.rate_unpublished");
    assert_eq!(code(&date("2027-07-01")), "period.unsupported");
    assert_eq!(code(&date("2018-06-30")), "period.unsupported");
    assert_eq!(code(&with("year", json!("2023-24"))), "fhss.year_mismatch");
    assert_eq!(
        code(&with("determination_date", json!("2024-13-01"))),
        "input.date_invalid"
    );
    assert_eq!(
        code(&with("amount_requested", json!("99999"))),
        "fhss.request_exceeds_maximum"
    );
    assert_eq!(
        code(&with("estimated_marginal_rate", json!("0.46"))),
        "input.fraction_invalid"
    );
    assert_eq!(
        code(&with("scope_confirmed", json!(false))),
        "scope.not_confirmed"
    );
    assert_eq!(code(&with("fund", json!("x"))), "input.field_unknown");
    let mut stated_extra = stated("1", "1", "1");
    stated_extra["determination_requested"] = json!("2025-06-01");
    assert_eq!(code(&stated_extra), "input.field_unknown");
    let mut short = stated("1", "1", "1");
    short["determination"] = json!({"concessional": "1"});
    assert_eq!(code(&short), "input.amount_invalid");
}
