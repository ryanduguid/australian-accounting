// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Payroll tax for 2026-27 in each state and territory against each revenue
//! office's published examples, read on 19 November 2026, examples worked by
//! hand from the Acts where an office publishes none, and the thresholds, rate
//! bands and tapers at their edges (docs/calculation-evidence.md#payroll-tax).
//! The duguid.com.au payroll tax calculator (17 November 2026) holds the same
//! cases. Every employer here is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(place: &str, wages: &str, australian: &str) -> Value {
    let mut v = json!({
        "jurisdiction": place, "taxable_wages": wages, "australian_wages": australian,
        "group_member": false, "employed_all_year": true,
        "year": "2026-27", "scope_confirmed": true,
    });
    if place == "vic" {
        v["regional_employer"] = json!(false);
    }
    v
}

fn run(input: &Value) -> Value {
    let result = api::evaluate("payroll-tax", input).unwrap();
    // derive gives the same result, with a step for each amount.
    let envelope = json!({"command": "payroll-tax", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn tax(place: &str, wages: &str, australian: &str) -> Value {
    run(&input(place, wages, australian))
}

fn amounts<const N: usize>(result: &Value, keys: [&str; N]) -> [String; N] {
    keys.map(|key| result["amounts"][key].as_str().unwrap().to_string())
}

const MAIN: [&str; 4] = ["deduction", "taxable_amount", "payroll_tax", "total"];

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("payroll-tax", input).unwrap_err().code
}

/// Revenue NSW, 'Payroll tax thresholds and rates': '$1,500,000 ... Payroll
/// tax calculated on $300,000', and '$1,200,000 x ($900,000 ÷ $3,000,000) =
/// $360,000' (taxed on $540,000), at 5.45%.
#[test]
fn new_south_wales_agrees_with_revenue_nsw() {
    let one = tax("nsw", "1500000.00", "1500000.00");
    assert_eq!(
        amounts(&one, MAIN),
        ["1200000.00", "300000.00", "16350.00", "16350.00"]
    );
    let shared = tax("nsw", "900000.00", "3000000.00");
    assert_eq!(
        amounts(&shared, MAIN),
        ["360000.00", "540000.00", "29430.00", "29430.00"]
    );
    assert_eq!(one["rates"]["rate"], "0.0545");
}

/// SRO Victoria, 'Payroll tax surcharges' example 2: $80 million in Victoria
/// and $70 million in NSW give thresholds of $5,333,333.33 and
/// $53,333,333.33, and each surcharge is 0.5% of the Victorian wages above
/// each: (80m - 5,333,333.33) x 0.5% + (80m - 53,333,333.33) x 0.5% =
/// $506,666.67. No deduction above $5 million; 4.85% on $80 million.
#[test]
fn victoria_agrees_with_the_state_revenue_office_surcharge_example() {
    let r = tax("vic", "80000000.00", "150000000.00");
    let keys = [
        "deduction",
        "payroll_tax",
        "mental_health_and_wellbeing_surcharge",
        "covid_19_debt_temporary_surcharge",
        "total",
    ];
    assert_eq!(
        amounts(&r, keys),
        ["0.00", "3880000.00", "506666.67", "506666.67", "4893333.34"]
    );
}

/// SRO Victoria, 'Payroll tax (current rates)': the deduction phases out at
/// 50% between $3 million and $5 million, so $4,200,000 has $1,000,000 -
/// $1,200,000 x 50% = $400,000; 4.85%, or 1.2125% for a regional employer.
#[test]
fn victoria_phases_out_the_deduction_and_takes_the_regional_rate() {
    let r = tax("vic", "4200000.00", "4200000.00");
    assert_eq!(
        amounts(&r, MAIN),
        ["400000.00", "3800000.00", "184300.00", "184300.00"]
    );
    let mut regional = input("vic", "4200000.00", "4200000.00");
    regional["regional_employer"] = json!(true);
    let r = run(&regional);
    assert_eq!(
        amounts(&r, MAIN),
        ["400000.00", "3800000.00", "46075.00", "46075.00"]
    );
    assert_eq!(r["rates"]["rate"], "0.012125");
    assert_eq!(
        tax("vic", "3000000.00", "3000000.00")["amounts"]["deduction"],
        "1000000.00"
    );
}

/// Queensland Revenue Office, 'Payroll tax deductions' (annual deduction
/// example): Queensland wages of $1,356,789 and Australian wages of
/// $1,789,776 give $932,460 to the dollar; s 29(1) gives $932,459.64.
#[test]
fn queensland_agrees_with_the_revenue_office_deduction_example() {
    let r = tax("qld", "1356789.00", "1789776.00");
    assert_eq!(
        amounts(&r, MAIN),
        ["932459.64", "424329.36", "20155.64", "20155.64"]
    );
    let levies = [
        "mental_health_levy_primary",
        "mental_health_levy_additional",
    ];
    assert_eq!(amounts(&r, levies), ["0.00", "0.00"]);
}

/// RevenueWA, payroll tax employer guide: annual wages of $1,104,000 give
/// $6,600. Its examples for $1,200,000 and for $1,000,000 of $4,000,000
/// round the deduction to the dollar first ($969,231 and $134,616); the Act
/// sets no rounding, so the cents stay.
#[test]
fn western_australia_agrees_with_revenuewa() {
    let r = tax("wa", "1104000.00", "1104000.00");
    assert_eq!(
        amounts(&r, MAIN),
        ["984000.00", "120000.00", "6600.00", "6600.00"]
    );
    let r = tax("wa", "1200000.00", "1200000.00");
    assert_eq!(
        amounts(&r, ["deduction", "payroll_tax"]),
        ["969230.77", "12692.31"]
    );
    let r = tax("wa", "1000000.00", "4000000.00");
    assert_eq!(
        amounts(&r, ["deduction", "payroll_tax"]),
        ["134615.38", "47596.15"]
    );
}

/// RevenueSA, Guide to Legislation 2026-27: $500,000 of $2,000,000 has a
/// deduction of $150,000 and $800,000 of $1,600,000 one of $300,000. Sch 1
/// cl 5(2)(a) gives (1,600,000 - 1,500,000) / 200,000 x 4.95% = 2.475%.
#[test]
fn south_australia_agrees_with_revenuesa_and_phases_in_the_rate() {
    let r = tax("sa", "500000.00", "2000000.00");
    assert_eq!(
        amounts(&r, MAIN),
        ["150000.00", "350000.00", "17325.00", "17325.00"]
    );
    assert_eq!(r["rates"]["rate"], "0.0495");
    let r = tax("sa", "800000.00", "1600000.00");
    assert_eq!(
        amounts(&r, MAIN),
        ["300000.00", "500000.00", "12375.00", "12375.00"]
    );
    assert_eq!(r["rates"]["rate"], "0.02475");
    assert_eq!(
        tax("sa", "1560000.00", "1560000.00")["rates"]["rate"],
        "0.01485"
    );
    let odd = tax("sa", "1512345.67", "1512345.67");
    assert_eq!(odd["rates"]["rate"], "0.003055553325");
}

/// SRO Tasmania, annual adjustment return guideline: example 1, $2,400,000
/// in Tasmania only, $30,000 + $24,400 = $54,400; example 2, $2,400,000 of
/// $3,000,000, a threshold of $1,000,000 and $24,000 + $48,800 = $72,800.
#[test]
fn tasmania_agrees_with_the_state_revenue_office_examples() {
    let lines = [
        "tax_at_lower_rate",
        "tax_at_higher_rate",
        "payroll_tax",
        "total",
    ];
    let only = tax("tas", "2400000.00", "2400000.00");
    assert_eq!(
        amounts(&only, lines),
        ["30000.00", "24400.00", "54400.00", "54400.00"]
    );
    let shared = tax("tas", "2400000.00", "3000000.00");
    assert_eq!(shared["amounts"]["deduction"], "1000000.00");
    assert_eq!(
        amounts(&shared, lines),
        ["24000.00", "48800.00", "72800.00", "72800.00"]
    );
    let edge = tax("tas", "2000000.00", "2000000.00");
    assert_eq!(
        amounts(&edge, lines),
        ["30000.00", "0.00", "30000.00", "30000.00"]
    );
}

#[test]
fn tasmania_attributes_per_rate_rounding_to_the_worksheet() {
    let facts = input("tas", "2400000.21", "3000000.00");
    let envelope = json!({"command": "payroll-tax", "input": facts});
    let doc = api::evaluate("derive", &envelope).unwrap();
    let payroll = doc["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["id"] == "payroll_tax")
        .unwrap();
    assert_eq!(payroll["value"], "72800.00");
    assert_eq!(doc["result"]["amounts"]["payroll_tax"], "72800.00");
    assert!(
        payroll["authority"]
            .as_str()
            .unwrap()
            .contains("This worksheet rounds the tax at each rate to the cent before adding.")
    );
}

/// Neither the ACT Revenue Office nor the Territory Revenue Office publishes
/// a worked figure that could be read, so these are worked from the Acts.
/// ACT: ($3,000,000 - $1,750,000) x 6.75% = $84,375; $25,000,000 of
/// $30,000,000 shares the threshold to $1,458,333.33, and 6.85% of
/// $23,541,666.67 is $1,612,604.17. NT: $3,000,000 has $2,500,000 - $500,000
/// / 2 = $2,250,000, and 5.5% of $750,000 is $41,250.
#[test]
fn the_territories_agree_with_examples_worked_from_their_acts() {
    let act = tax("act", "3000000.00", "3000000.00");
    assert_eq!(
        amounts(&act, MAIN),
        ["1750000.00", "1250000.00", "84375.00", "84375.00"]
    );
    let act = tax("act", "25000000.00", "30000000.00");
    assert_eq!(
        amounts(&act, ["deduction", "payroll_tax"]),
        ["1458333.33", "1612604.17"]
    );
    let nt = tax("nt", "3000000.00", "3000000.00");
    assert_eq!(
        amounts(&nt, MAIN),
        ["2250000.00", "750000.00", "41250.00", "41250.00"]
    );
}

/// At each threshold the deduction covers the wages and nothing is payable;
/// South Australia's threshold ($1,500,000) is not its deduction ($600,000).
/// No wages at all give nil throughout.
#[test]
fn nothing_is_payable_at_each_threshold_or_without_wages() {
    for (place, threshold, deduction) in [
        ("nsw", "1200000.00", "1200000.00"),
        ("vic", "1000000.00", "1000000.00"),
        ("qld", "1300000.00", "1300000.00"),
        ("wa", "1000000.00", "1000000.00"),
        ("sa", "1500000.00", "600000.00"),
        ("tas", "1250000.00", "1250000.00"),
        ("act", "1750000.00", "1750000.00"),
        ("nt", "2500000.00", "2500000.00"),
    ] {
        let r = tax(place, threshold, threshold);
        assert_eq!(
            amounts(&r, ["deduction", "total"]),
            [deduction, "0.00"],
            "{place}"
        );
        let none = tax(place, "0.00", "0.00");
        assert_eq!(none["amounts"]["total"], "0.00", "{place}");
        assert!(
            none["amounts"]
                .as_object()
                .unwrap()
                .values()
                .all(|v| v == "0.00")
        );
    }
    let sa = tax("sa", "1200000.00", "1200000.00");
    assert_eq!(
        amounts(&sa, MAIN),
        ["600000.00", "600000.00", "0.00", "0.00"]
    );
    let sa = tax("sa", "500000.00", "500000.00");
    assert_eq!(amounts(&sa, ["deduction", "total"]), ["500000.00", "0.00"]);
}

/// Each tapered deduction reaches nil where its Act says: Victoria at $5m,
/// Queensland at $10.4m, Western Australia and the Northern Territory at
/// $7.5m; $7 less still leaves some.
#[test]
fn each_tapered_deduction_reaches_nil_at_its_edge() {
    for (place, nil, below) in [
        ("vic", "5000000.00", "4999993.00"),
        ("qld", "10400000.00", "10399993.00"),
        ("wa", "7500000.00", "7499993.00"),
        ("nt", "7500000.00", "7499993.00"),
    ] {
        assert_eq!(
            tax(place, nil, nil)["amounts"]["deduction"],
            "0.00",
            "{place}"
        );
        assert_ne!(
            tax(place, below, below)["amounts"]["deduction"],
            "0.00",
            "{place}"
        );
    }
}

/// The rate bands turn over where each Act says: Queensland above $6.5m, the
/// ACT above $20m and $150m, the Northern Territory from $100m, and South
/// Australia's phased rate reaches 4.95% at $1.7m.
#[test]
fn rate_bands_turn_over_at_their_edges() {
    let applied = |place: &str, wages: &str| tax(place, wages, wages)["rates"]["rate"].clone();
    for (place, wages, rate) in [
        ("qld", "6500000.00", "0.0475"),
        ("qld", "6500000.01", "0.0495"),
        ("act", "20000000.00", "0.0675"),
        ("act", "20000000.01", "0.0685"),
        ("act", "50000000.01", "0.0735"),
        ("act", "100000000.01", "0.0785"),
        ("act", "150000000.00", "0.0785"),
        ("act", "150000000.01", "0.0875"),
        ("nt", "99999999.99", "0.055"),
        ("nt", "100000000.00", "0.065"),
        ("sa", "1700000.00", "0.0495"),
        ("sa", "1700000.01", "0.0495"),
        ("sa", "1500000.00", "0"),
    ] {
        assert_eq!(applied(place, wages), rate, "{place} {wages}");
    }
    assert_eq!(
        tax("sa", "1500000.01", "1500000.01")["amounts"]["total"],
        "0.00"
    );
}

/// Queensland's levy: 0.25% of $400 above $10m is $1, and nothing above
/// $100m until wages pass it. Victoria above $100m: 0.5% of $90,000,100 and
/// of $100 is $450,000.50 + $0.50 for each surcharge.
#[test]
fn surcharges_and_levies_start_above_their_thresholds() {
    let levy =
        |place: &str, wages: &str, key: &str| tax(place, wages, wages)["amounts"][key].clone();
    assert_eq!(
        levy("qld", "10000000.00", "mental_health_levy_primary"),
        "0.00"
    );
    assert_eq!(
        levy("qld", "10000400.00", "mental_health_levy_primary"),
        "1.00"
    );
    assert_eq!(
        levy("qld", "100000000.00", "mental_health_levy_additional"),
        "0.00"
    );
    assert_eq!(
        levy("qld", "100000400.00", "mental_health_levy_additional"),
        "2.00"
    );
    let vic = "mental_health_and_wellbeing_surcharge";
    assert_eq!(levy("vic", "10000000.00", vic), "0.00");
    assert_eq!(levy("vic", "100000100.00", vic), "450001.00");
    assert_eq!(
        levy("vic", "100000100.00", "covid_19_debt_temporary_surcharge"),
        "450001.00"
    );
}

/// The working of each step names the provision and the figures it used.
#[test]
fn derive_cites_each_jurisdiction_and_shows_the_working() {
    let envelope =
        json!({"command": "payroll-tax", "input": input("nsw", "900000.00", "3000000.00")});
    let doc = api::evaluate("derive", &envelope).unwrap();
    let steps = doc["steps"].as_array().unwrap();
    let ids: Vec<&str> = steps.iter().map(|s| s["id"].as_str().unwrap()).collect();
    assert_eq!(
        ids,
        [
            "australian_deduction",
            "deduction",
            "taxable_amount",
            "payroll_tax",
            "total"
        ]
    );
    assert!(
        steps[0]["authority"]
            .as_str()
            .unwrap()
            .contains("Payroll Tax Act 2007 (NSW) Sch 1 cl 1")
    );
    assert_eq!(
        steps[1]["working"],
        "min(australian_deduction, australian_wages) x taxable_wages / australian_wages = min(1200000, 3000000.00) x 900000.00 / 3000000.00 = 360000"
    );
    for place in ["vic", "qld", "wa", "sa", "tas", "act", "nt"] {
        let envelope =
            json!({"command": "payroll-tax", "input": input(place, "3000000.00", "3000000.00")});
        let doc = api::evaluate("derive", &envelope).unwrap();
        for step in doc["steps"].as_array().unwrap() {
            assert!(!step["authority"].as_str().unwrap().is_empty(), "{place}");
        }
    }
}

/// The first NSW example with `key` set to `value`.
fn with(key: &str, value: Value) -> Value {
    let mut v = input("nsw", "1500000.00", "1500000.00");
    v[key] = value;
    v
}

#[test]
fn refusals_name_what_is_out_of_scope() {
    assert_eq!(
        code(&with("group_member", json!(true))),
        "payroll_tax.grouped"
    );
    assert_eq!(
        code(&with("employed_all_year", json!(false))),
        "payroll_tax.part_year"
    );
    assert_eq!(
        code(&with("australian_wages", json!("1499999.99"))),
        "payroll_tax.wages_conflict"
    );
    for place in ["NSW", "vic_regional", "au-nsw", ""] {
        assert_eq!(
            code(&with("jurisdiction", json!(place))),
            "payroll_tax.jurisdiction_invalid"
        );
    }
}

#[test]
fn refusals_name_the_input_that_is_wrong() {
    assert_eq!(
        code(&with("regional_employer", json!(false))),
        "input.field_unknown"
    );
    let mut vic = input("vic", "1500000.00", "1500000.00");
    vic.as_object_mut().unwrap().remove("regional_employer");
    assert_eq!(code(&vic), "input.missing_field");
    vic["regional_employer"] = json!("yes");
    assert_eq!(code(&vic), "input.flag_invalid");
    assert_eq!(code(&with("year", json!("2025-26"))), "period.unsupported");
    assert_eq!(code(&with("year", json!("2027-28"))), "period.unsupported");
    assert_eq!(
        code(&with("scope_confirmed", json!(false))),
        "scope.not_confirmed"
    );
    assert_eq!(
        code(&with("taxable_wages", json!("-1.00"))),
        "input.amount_invalid"
    );
    assert_eq!(
        code(&with("taxable_wages", json!("1500000.001"))),
        "input.amount_invalid"
    );
    assert_eq!(code(&with("group", json!(false))), "input.field_unknown");
    assert_eq!(
        code(&with("group_member", json!("no"))),
        "input.flag_invalid"
    );
}

/// The annual reconciliation, worked by hand from each Act's annual
/// adjustment, read on 26 November 2026: the year's liability less what the
/// periodic returns paid or made payable.
fn reconciled(place: &str, wages: &str, australian: &str, periodic: &[(&str, &str)]) -> Value {
    let mut v = input(place, wages, australian);
    for (key, amount) in periodic {
        v[*key] = json!(amount);
    }
    run(&v)
}

const SETTLED: [&str; 4] = [
    "total",
    "periodic_tax",
    "adjustment_payable",
    "adjustment_refund",
];

/// Payroll Tax Act 2007 (Vic) s 83(1) and (2): $184,300.00 for the year
/// against $180,000 and $190,000 paid on the returns.
#[test]
fn victoria_reconciles_the_year_against_the_returns() {
    let short = reconciled(
        "vic",
        "4200000.00",
        "4200000.00",
        &[("periodic_tax", "180000.00")],
    );
    assert_eq!(
        amounts(&short, SETTLED),
        ["184300.00", "180000.00", "4300.00", "0.00"]
    );
    let over = reconciled(
        "vic",
        "4200000.00",
        "4200000.00",
        &[("periodic_tax", "190000.00")],
    );
    assert_eq!(
        amounts(&over, SETTLED),
        ["184300.00", "190000.00", "0.00", "5700.00"]
    );
}

/// Payroll Tax Act 2011 (ACT) s 83(3): a difference payable of less than
/// $20 is determined to be $0; $20 or more is payable.
#[test]
fn act_reconciliation_drops_a_shortfall_under_20_dollars() {
    let small = reconciled(
        "act",
        "3000000.00",
        "3000000.00",
        &[("periodic_tax", "84355.01")],
    );
    assert_eq!(small["amounts"]["adjustment_payable"], "0.00");
    let due = reconciled(
        "act",
        "3000000.00",
        "3000000.00",
        &[("periodic_tax", "84355.00")],
    );
    assert_eq!(due["amounts"]["adjustment_payable"], "20.00");
}

/// Payroll Tax Act 1971 (Qld) ss 30, 31, 43J and 43K: payroll tax and levy
/// are settled apart; Pay-roll Tax Assessment Act 2002 (WA) s 16 refunds an
/// excess; with no wages, all that was paid is refunded.
#[test]
fn queensland_settles_tax_and_levy_apart_and_others_refund() {
    let periodic = [("periodic_tax", "20000.00"), ("periodic_levy", "10.00")];
    let r = reconciled("qld", "1356789.00", "1789776.00", &periodic);
    let keys = [
        "adjustment_payable",
        "adjustment_refund",
        "annual_levy_amount",
        "levy_adjustment_payable",
        "levy_adjustment_refund",
    ];
    assert_eq!(
        amounts(&r, keys),
        ["155.64", "0.00", "0.00", "0.00", "10.00"]
    );
    let wa = reconciled(
        "wa",
        "1104000.00",
        "1104000.00",
        &[("periodic_tax", "7000.00")],
    );
    assert_eq!(wa["amounts"]["adjustment_refund"], "400.00");
    let none = reconciled("sa", "0.00", "0.00", &[("periodic_tax", "100.00")]);
    assert_eq!(none["amounts"]["adjustment_refund"], "100.00");
}

#[test]
fn reconciliation_refusals() {
    let with = |place: &str, periodic: &[(&str, &str)]| {
        let mut v = input(place, "1500000.00", "1500000.00");
        for (key, amount) in periodic {
            v[*key] = json!(amount);
        }
        code(&v)
    };
    assert_eq!(
        with("nsw", &[("periodic_tax", "1.00")]),
        "payroll_tax.reconciliation_unread"
    );
    assert_eq!(
        with("vic", &[("periodic_levy", "1.00")]),
        "input.field_unknown"
    );
    assert_eq!(
        with(
            "vic",
            &[("periodic_tax", "1.00"), ("periodic_levy", "1.00")]
        ),
        "input.field_unknown"
    );
    assert_eq!(
        with("qld", &[("periodic_tax", "1.00")]),
        "input.missing_field"
    );
    assert_eq!(
        with("wa", &[("periodic_tax", "-1.00")]),
        "input.amount_invalid"
    );
}
