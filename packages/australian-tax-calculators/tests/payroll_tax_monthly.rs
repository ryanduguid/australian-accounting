// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! One monthly payroll tax return in 2026-27 against the Queensland Revenue
//! Office's periodic deduction examples and cases worked by hand from each
//! Act's monthly provisions, read on 26 November 2026
//! (docs/calculation-evidence.md#payroll-tax-monthly). Every employer here is
//! fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(place: &str, month: &str, wages: [&str; 4]) -> Value {
    let [tw, iw, etw, eiw] = wages;
    let mut v = json!({
        "jurisdiction": place, "month": month,
        "taxable_wages": tw, "interstate_wages": iw,
        "estimated_taxable_wages": etw, "estimated_interstate_wages": eiw,
        "group_member": false, "employed_all_year": true,
        "year": "2026-27", "scope_confirmed": true,
    });
    if matches!(place, "vic" | "qld") {
        v["regional_employer"] = json!(false);
    }
    if place == "qld" && iw == "0.00" {
        v["previous_interstate_wage_payer"] = json!(false);
    }
    v
}

fn run(input: &Value) -> Value {
    let result = api::evaluate("payroll-tax-monthly", input).unwrap();
    let envelope = json!({"command": "payroll-tax-monthly", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn tax(place: &str, month: &str, wages: [&str; 4]) -> Value {
    run(&input(place, month, wages))
}

fn amounts<const N: usize>(result: &Value, keys: [&str; N]) -> [String; N] {
    keys.map(|key| result["amounts"][key].as_str().unwrap().to_string())
}

const MAIN: [&str; 4] = ["deduction", "taxable_amount", "payroll_tax", "total"];

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("payroll-tax-monthly", input)
        .unwrap_err()
        .code
}

/// QRO 'Payroll tax deductions', 'Example—actual periodic deduction':
/// $158,521 of monthly wages, $108,333 - $50,188 / 7 = $101,163 to the
/// dollar (s 17). The levy's periodic threshold is $10 million / 12.
#[test]
fn queensland_actual_periodic_deduction_agrees_with_the_qro() {
    let r = tax(
        "qld",
        "2026-07",
        ["158521.00", "0.00", "1902252.00", "0.00"],
    );
    assert_eq!(
        amounts(&r, MAIN),
        ["101163.29", "57357.71", "2724.49", "2724.49"]
    );
    assert_eq!(r["amounts"]["primary_periodic_threshold"], "833333.33");
    assert_eq!(r["amounts"]["mental_health_levy_primary"], "0.00");
    assert_eq!(r["rates"]["rate"], "0.0475");
}

/// QRO, 'Example—fixed periodic deduction': $387,524 of $1,500,000
/// estimated, $387,524 / $1,500,000 x ($108,333 - $2,381) = $27,373 to the
/// dollar (s 17); a month with interstate wages takes it (s 20(2)(a)).
#[test]
fn queensland_fixed_periodic_deduction_agrees_with_the_qro() {
    let r = tax(
        "qld",
        "2026-08",
        ["32294.00", "92706.00", "387524.00", "1112476.00"],
    );
    assert_eq!(
        amounts(&r, MAIN),
        ["27372.63", "4921.37", "233.77", "233.77"]
    );
    // A previous interstate wage payer takes it in a month of local wages too.
    let mut v = input(
        "qld",
        "2026-08",
        ["32294.00", "0.00", "387524.00", "1112476.00"],
    );
    v["previous_interstate_wage_payer"] = json!(true);
    assert_eq!(run(&v)["amounts"]["deduction"], "27372.63");
}

/// Worked by hand from ss 10(2), 17 and 43C to 43F: $6 million in a month
/// takes 4.95%, no deduction, and the primary levy on the wages above
/// $10 million / 12; the additional levy's threshold is $100 million / 12.
#[test]
fn queensland_rate_and_levy_in_a_large_month() {
    let r = tax(
        "qld",
        "2027-01",
        ["6000000.00", "0.00", "60000000.00", "0.00"],
    );
    assert_eq!(
        amounts(
            &r,
            [
                "deduction",
                "payroll_tax",
                "mental_health_levy_primary",
                "mental_health_levy_additional",
                "total"
            ]
        ),
        ["0.00", "297000.00", "12916.67", "0.00", "309916.67"]
    );
    assert_eq!(r["amounts"]["additional_periodic_threshold"], "8333333.33");
    // $541,666 of wages in the month keeps 4.75%; a cent more takes 4.95%.
    let at = |w: &str| tax("qld", "2026-09", [w, "0.00", "6500000.00", "0.00"]);
    assert_eq!(at("541666.00")["rates"]["rate"], "0.0475");
    assert_eq!(at("541666.01")["rates"]["rate"], "0.0495");
}

/// Worked by hand from Sch 2 cl 5: $83,333 - ($4,200,000 - $3,000,000) / 12
/// x 50% = $33,333, then 4.85% (or 1.2125% for a regional employer).
#[test]
fn victoria_phases_out_the_monthly_deduction_on_the_estimate() {
    let r = tax(
        "vic",
        "2026-07",
        ["350000.00", "0.00", "4200000.00", "0.00"],
    );
    assert_eq!(
        amounts(&r, MAIN),
        ["33333.00", "316667.00", "15358.35", "15358.35"]
    );
    let mut v = input(
        "vic",
        "2026-07",
        ["350000.00", "0.00", "4200000.00", "0.00"],
    );
    v["regional_employer"] = json!(true);
    assert_eq!(run(&v)["amounts"]["payroll_tax"], "3839.59");
    // At or above $5 million the deduction is nil; at $3 million it is BDA.
    let none = tax("vic", "2026-07", ["90000.00", "0.00", "5000000.00", "0.00"]);
    assert_eq!(none["amounts"]["deduction"], "0.00");
    let all = tax("vic", "2026-07", ["90000.00", "0.00", "3000000.00", "0.00"]);
    assert_eq!(all["amounts"]["deduction"], "83333.00");
}

/// Worked by hand from Sch 2 cl 4(3): (TW - 833,333) x 0.5% to $8,333,333,
/// then TW x 1% - 45,833, each surcharge charged twice (S and T, cl 1B).
#[test]
fn victoria_surcharges_on_a_month_without_interstate_wages() {
    let r = tax(
        "vic",
        "2026-10",
        ["900000.00", "0.00", "10800000.00", "0.00"],
    );
    let keys = [
        "payroll_tax",
        "mental_health_and_wellbeing_surcharge",
        "covid_19_debt_temporary_surcharge",
        "total",
    ];
    assert_eq!(
        amounts(&r, keys),
        ["43650.00", "333.34", "333.34", "44316.68"]
    );
    let big = tax(
        "vic",
        "2026-10",
        ["9000000.00", "0.00", "108000000.00", "0.00"],
    );
    assert_eq!(
        big["amounts"]["mental_health_and_wellbeing_surcharge"],
        "44167.00"
    );
    let none = tax(
        "vic",
        "2026-10",
        ["833333.00", "0.00", "10000000.00", "0.00"],
    );
    assert_eq!(
        none["amounts"]["mental_health_and_wellbeing_surcharge"],
        "0.00"
    );
}

/// Worked by hand from Sch 2 cl 6(3): half the wages estimated in Victoria
/// share BDA and the phase-out, $41,666.50 - $20,833.33 = $20,833.17,
/// below the cl 5(1) cap of $83,333; cl 4(4) shares the surcharge
/// thresholds by the same estimate.
#[test]
fn victoria_shares_the_deduction_and_surcharge_by_the_estimate() {
    let r = tax(
        "vic",
        "2026-11",
        ["170000.00", "160000.00", "2000000.00", "2000000.00"],
    );
    assert_eq!(
        amounts(&r, ["clause_5_amount", "deduction", "payroll_tax"]),
        ["83333.00", "20833.17", "7234.59"]
    );
    let s = tax(
        "vic",
        "2026-11",
        ["1000000.00", "1.00", "30000000.00", "10000000.00"],
    );
    assert_eq!(
        s["amounts"]["mental_health_and_wellbeing_surcharge"],
        "1875.00"
    );
    let big = tax(
        "vic",
        "2026-11",
        ["15000000.00", "1.00", "150000000.00", "50000000.00"],
    );
    assert_eq!(
        amounts(
            &big,
            ["payroll_tax", "mental_health_and_wellbeing_surcharge"]
        ),
        ["727500.00", "115625.00"]
    );
    let small = tax(
        "vic",
        "2026-11",
        ["1.00", "1.00", "5000000.00", "5000000.00"],
    );
    assert_eq!(
        small["amounts"]["mental_health_and_wellbeing_surcharge"],
        "0.00"
    );
    // A month below the employer's share of the threshold gives less than nil.
    let low = input(
        "vic",
        "2026-11",
        ["300000.00", "1.00", "30000000.00", "30000000.00"],
    );
    assert_eq!(code(&low), "payroll_tax_monthly.surcharge_negative");
}

/// Worked by hand from Pay-roll Tax Assessment Act 2002 (WA) ss 8, 11 and
/// 12: nil to $83,333, the taper to $625,000, then all wages at 5.5%; an
/// interstate employer deducts the nominated amount (s 15).
#[test]
fn western_australia_monthly_threshold_taper_and_nominated_deduction() {
    let r = tax("wa", "2026-12", ["300000.00", "0.00", "3600000.00", "0.00"]);
    assert_eq!(
        amounts(&r, MAIN),
        ["49999.62", "250000.38", "13750.02", "13750.02"]
    );
    let nil = tax("wa", "2026-12", ["83333.00", "0.00", "1000000.00", "0.00"]);
    assert_eq!(nil["amounts"]["total"], "0.00");
    let full = tax("wa", "2026-12", ["625000.00", "0.00", "7500000.00", "0.00"]);
    assert_eq!(full["amounts"]["total"], "34375.00");
    let mut v = input(
        "wa",
        "2026-12",
        ["100000.00", "0.00", "1200000.00", "500000.00"],
    );
    v["stated_deduction"] = json!("20000.00");
    assert_eq!(run(&v)["amounts"]["total"], "4400.00");
    v["stated_deduction"] = json!("200000.00");
    assert_eq!(run(&v)["amounts"]["total"], "0.00");
}

/// Worked by hand from Payroll Tax Act 2009 (SA) Sch 2 cll 5 and 6: the
/// deduction is $50,000 shared by the estimate, and the rate is phased in on
/// the year's estimated wages from $1,500,000 to $1,700,000.
#[test]
fn south_australia_phases_in_the_rate_on_the_estimate() {
    let r = tax("sa", "2027-01", ["140000.00", "0.00", "1600000.00", "0.00"]);
    assert_eq!(
        amounts(&r, MAIN),
        ["50000.00", "90000.00", "2227.50", "2227.50"]
    );
    assert_eq!(r["rates"]["rate"], "0.02475");
    let full = tax(
        "sa",
        "2027-01",
        ["125000.00", "1.00", "1500000.00", "1500000.00"],
    );
    assert_eq!(
        amounts(&full, MAIN),
        ["25000.00", "100000.00", "4950.00", "4950.00"]
    );
    let none = tax("sa", "2027-01", ["125000.00", "0.00", "1500000.00", "0.00"]);
    assert_eq!(none["amounts"]["total"], "0.00");
    assert_eq!(none["rates"]["rate"], "0");
}

/// Worked by hand from Payroll Tax Act 2008 (Tas) Sch 2 cll 2 and 3 and
/// Sch 1 cl 4: $200,000 in July is $2,354,838.71 a year, taxed at 4% to
/// $2 million and 6.1% above, then 31 / 365 of it.
#[test]
fn tasmania_annualises_the_month() {
    let r = tax(
        "tas",
        "2026-07",
        ["200000.00", "0.00", "2400000.00", "0.00"],
    );
    let keys = [
        "annualised_wages",
        "tax_at_lower_rate",
        "tax_at_higher_rate",
        "payroll_tax",
        "total",
    ];
    assert_eq!(
        amounts(&r, keys),
        ["2354838.71", "30000.00", "21645.16", "4386.30", "4386.30"]
    );
    let below = tax("tas", "2027-02", ["90000.00", "0.00", "1000000.00", "0.00"]);
    assert_eq!(below["amounts"]["total"], "0.00");
    let none = tax("tas", "2027-02", ["0.00", "0.00", "1000000.00", "0.00"]);
    assert_eq!(none["amounts"]["total"], "0.00");
    let inter = input("tas", "2027-02", ["90000.00", "0.00", "1000000.00", "1.00"]);
    assert_eq!(code(&inter), "payroll_tax_monthly.interstate_unworked");
}

/// Worked by hand from Payroll Tax Act 2011 (ACT) Sch 2 ss 2.2 to 2.4 and
/// DI2026-151 s 5(b) and (d): $145,833.33 a month (shared by the estimate
/// with interstate wages), at the Table 2 rate for the month's total wages.
#[test]
fn act_monthly_threshold_and_table_2_rates() {
    let r = tax(
        "act",
        "2027-03",
        ["300000.00", "0.00", "3600000.00", "0.00"],
    );
    assert_eq!(
        amounts(&r, ["deduction", "payroll_tax"]),
        ["145833.33", "10406.25"]
    );
    let shared = tax(
        "act",
        "2027-03",
        ["250000.00", "80000.00", "3000000.00", "1000000.00"],
    );
    assert_eq!(
        amounts(&shared, ["deduction", "payroll_tax"]),
        ["109375.00", "9492.19"]
    );
    let band = tax(
        "act",
        "2027-03",
        ["2000000.00", "0.00", "24000000.00", "0.00"],
    );
    assert_eq!(band["amounts"]["payroll_tax"], "127010.42");
    assert_eq!(band["rates"]["rate"], "0.0685");
    let top = tax(
        "act",
        "2027-03",
        ["12500000.01", "0.00", "150000000.00", "0.00"],
    );
    assert_eq!(top["rates"]["rate"], "0.0875");
    let small = tax(
        "act",
        "2027-03",
        ["100000.00", "0.00", "1200000.00", "0.00"],
    );
    assert_eq!(small["amounts"]["total"], "0.00");
    let unset = input(
        "act",
        "2027-03",
        ["100000.00", "40000.00", "1000000.00", "3000000.00"],
    );
    assert_eq!(code(&unset), "payroll_tax_monthly.rate_unset");
}

/// Payroll Tax Act 2009 (NT) Sch 2 cll 5 to 7 and s 7A: the stated
/// deduction, at most $2,500,000 / 12, at 5.5% or, with $100 million
/// estimated, 6.5%.
#[test]
fn northern_territory_stated_deduction_and_rate() {
    let mut v = input("nt", "2027-04", ["300000.00", "0.00", "3600000.00", "0.00"]);
    v["stated_deduction"] = json!("208333.33");
    assert_eq!(run(&v)["amounts"]["total"], "5041.67");
    v["estimated_taxable_wages"] = json!("100000000.00");
    assert_eq!(run(&v)["rates"]["rate"], "0.065");
    v["stated_deduction"] = json!("208333.34");
    assert_eq!(code(&v), "payroll_tax_monthly.deduction_above_cap");
}

#[test]
fn refuses_what_it_does_not_work() {
    let base = input("vic", "2026-07", ["1.00", "0.00", "1.00", "0.00"]);
    let with = |key: &str, value: Value| {
        let mut v = base.clone();
        v[key] = value;
        code(&v)
    };
    assert_eq!(
        with("jurisdiction", json!("nsw")),
        "payroll_tax_monthly.act_unread"
    );
    assert_eq!(
        with("jurisdiction", json!("xx")),
        "payroll_tax_monthly.jurisdiction_invalid"
    );
    for month in ["2026-06", "2027-07", "2026-7", "July", "2026-13"] {
        assert_eq!(
            with("month", json!(month)),
            "payroll_tax_monthly.month_invalid"
        );
    }
    assert_eq!(
        with("group_member", json!(true)),
        "payroll_tax_monthly.grouped"
    );
    assert_eq!(
        with("employed_all_year", json!(false)),
        "payroll_tax_monthly.part_year"
    );
    assert_eq!(
        with("estimated_taxable_wages", json!("0.00")),
        "payroll_tax_monthly.estimate_invalid"
    );
    assert_eq!(
        with("stated_deduction", json!("1.00")),
        "input.field_unknown"
    );
    let june = input("qld", "2027-06", ["1.00", "0.00", "1.00", "0.00"]);
    assert_eq!(code(&june), "payroll_tax_monthly.last_period");
    let mut regional = input("qld", "2026-07", ["1.00", "0.00", "1.00", "0.00"]);
    regional["regional_employer"] = json!(true);
    assert_eq!(code(&regional), "payroll_tax_monthly.regional_discount");
    let mut missing = base.clone();
    missing.as_object_mut().unwrap().remove("regional_employer");
    assert_eq!(code(&missing), "input.missing_field");
    let nt = input("nt", "2026-07", ["1.00", "0.00", "1.00", "0.00"]);
    assert_eq!(code(&nt), "input.missing_field");
}
