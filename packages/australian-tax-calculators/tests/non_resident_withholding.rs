// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `non-resident-withholding` against TAA 1953 Sch 1 Subdivs 12-F, 12-H and
//! 12-FB, ITAA 1936 s 128B, IT(DIRWHT)A 1974 s 7, IT(MITWHT)A 2008 s 4,
//! TAR 2017 ss 26, 31 to 34, 39 to 45 and 48 to 51, and ITRA 1986 s 23(2) and
//! Sch 7 Pt II (docs/calculation-evidence.md#non-resident-withholding).

use australian_tax_calculators::{api, metadata};
use serde_json::{Value, json};

fn eval(input: &Value) -> Value {
    api::evaluate("non-resident-withholding", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("non-resident-withholding", input)
        .unwrap_err()
        .code
}

fn dividend(
    payment: &str,
    franked: &str,
    cfi: &str,
    treaty: Option<&str>,
    exempt: bool,
    already: &str,
) -> Value {
    json!({
        "payment_type": "dividend",
        "payment": payment,
        "franked_amount": franked,
        "conduit_foreign_income": cfi,
        "treaty_rate": treaty,
        "exempt": exempt,
        "already_withheld": already,
        "year": "2025-26",
        "scope_confirmed": true,
    })
}

fn interest(payment: &str, treaty: Option<&str>, exempt: bool, already: &str) -> Value {
    json!({
        "payment_type": "interest",
        "payment": payment,
        "treaty_rate": treaty,
        "exempt": exempt,
        "already_withheld": already,
        "year": "2025-26",
        "scope_confirmed": true,
    })
}

fn royalty(payment: &str, treaty: Option<&str>, exempt: bool, already: &str) -> Value {
    json!({
        "payment_type": "royalty",
        "payment": payment,
        "treaty_rate": treaty,
        "exempt": exempt,
        "already_withheld": already,
        "year": "2025-26",
        "scope_confirmed": true,
    })
}

fn mit(payment: &str, eoi: bool, clean: &str, ncmi: &str, exempt: bool) -> Value {
    json!({
        "payment_type": "mit_fund_payment",
        "payment": payment,
        "information_exchange_country": eoi,
        "clean_building_mit_amount": clean,
        "non_concessional_mit_income": ncmi,
        "exempt": exempt,
        "year": "2025-26",
        "scope_confirmed": true,
    })
}

fn fb_simple(
    payment_type: &str,
    payment: &str,
    variation: Option<&str>,
    exempt: bool,
    already: &str,
) -> Value {
    json!({
        "payment_type": payment_type,
        "payment": payment,
        "variation_rate": variation,
        "exempt": exempt,
        "already_withheld": already,
        "year": "2025-26",
        "scope_confirmed": true,
    })
}

fn entertainment(
    payment: &str,
    entity: &str,
    bre: Option<bool>,
    variation: Option<&str>,
    exempt: bool,
    already: &str,
) -> Value {
    json!({
        "payment_type": "entertainment_or_sports",
        "payment": payment,
        "payee_entity": entity,
        "base_rate_entity": bre,
        "variation_rate": variation,
        "exempt": exempt,
        "already_withheld": already,
        "year": "2025-26",
        "scope_confirmed": true,
    })
}

#[test]
fn dividend_domestic_treaty_franked_cfi_and_exempt() {
    // Partially franked + CFI + 15% treaty.
    let out = eval(&dividend(
        "10000.00",
        "4000.00",
        "1000.00",
        Some("0.15"),
        false,
        "0.00",
    ));
    assert_eq!(out["calculation"], "non_resident_withholding");
    assert_eq!(out["payment_type"], "dividend");
    assert_eq!(out["subdivision"], "12-F");
    assert_eq!(out["final_tax"], true);
    assert_eq!(out["amounts"]["withholding_base"], "5000.00");
    assert_eq!(out["amounts"]["payment_used"], "5000.00");
    assert_eq!(out["amounts"]["regulation_withholding"], "750.00");
    assert_eq!(out["amounts"]["withholding"], "750.00");
    assert_eq!(out["amounts"]["net_payment"], "9250.00");
    assert_eq!(out["rates"]["domestic_rate"], "0.30");
    assert_eq!(out["rates"]["applied_rate"], "0.15");
    assert_eq!(out["rates"]["treaty_rate"], "0.15");

    // Domestic 30%, cents in payment and result disregarded under TAR 2017 s 26.
    // Unfranked = 1001.95 -> floor = 1001; 30% * 1001 = 300.30 -> floor = 300.00.
    let dom = eval(&dividend("1001.95", "0.00", "0.00", None, false, "0.00"));
    assert_eq!(dom["amounts"]["payment_used"], "1001.00");
    assert_eq!(dom["amounts"]["regulation_withholding"], "300.00");
    assert_eq!(dom["amounts"]["withholding"], "300.00");
    assert_eq!(dom["amounts"]["net_payment"], "701.95");

    // Fully franked + CFI -> nil withholding.
    let nil = eval(&dividend(
        "5000.00", "3000.00", "2000.00", None, false, "0.00",
    ));
    assert_eq!(nil["amounts"]["withholding_base"], "0.00");
    assert_eq!(nil["amounts"]["withholding"], "0.00");
    assert_eq!(nil["amounts"]["net_payment"], "5000.00");

    // Statutory exemption under s 12-300(a).
    let ex = eval(&dividend("5000.00", "0.00", "0.00", None, true, "0.00"));
    assert_eq!(ex["amounts"]["withholding"], "0.00");
    assert_eq!(ex["rates"]["applied_rate"], "0");
    assert_eq!(ex["amounts"]["net_payment"], "5000.00");

    // Intermediary with prior withholding under s 12-300(b).
    let inter = eval(&dividend(
        "1000.90",
        "0.00",
        "0.00",
        Some("0.15"),
        false,
        "50.00",
    ));
    // s 40 reg = 150.00; s 128B liability = 150.135; cap = floor_cents(100.135) = 100.13.
    assert_eq!(inter["amounts"]["regulation_withholding"], "150.00");
    assert_eq!(inter["amounts"]["withholding"], "100.13");
    assert_eq!(inter["amounts"]["net_payment"], "850.77");

    // Over-withheld earlier clamps to nil.
    let clamped = eval(&dividend(
        "1000.00",
        "0.00",
        "0.00",
        Some("0.15"),
        false,
        "200.00",
    ));
    assert_eq!(clamped["amounts"]["withholding"], "0.00");
    assert_eq!(clamped["amounts"]["net_payment"], "800.00");
}

#[test]
fn interest_domestic_treaty_cap_and_rounding() {
    // Domestic 10%: TAR 2017 s 26 + s 41 -> floor(0.10 * floor(1000.90)) = 100.00.
    let dom = eval(&interest("1000.90", None, false, "0.00"));
    assert_eq!(dom["subdivision"], "12-F");
    assert_eq!(dom["final_tax"], true);
    assert_eq!(dom["amounts"]["payment_used"], "1000.00");
    assert_eq!(dom["amounts"]["regulation_withholding"], "100.00");
    assert_eq!(dom["amounts"]["withholding"], "100.00");
    assert_eq!(dom["amounts"]["net_payment"], "900.90");

    // Treaty rate below 10% (e.g. 5%) engages TAA Sch 1 s 12-300(b) ceiling.
    // s 41 reg = 100.00; liability = 50.045 -> floor_cents = 50.04.
    let tr = eval(&interest("1000.90", Some("0.05"), false, "0.00"));
    assert_eq!(tr["amounts"]["regulation_withholding"], "100.00");
    assert_eq!(tr["amounts"]["withholding"], "50.04");
    assert_eq!(tr["amounts"]["net_payment"], "950.86");

    // Treaty 0% (financial institution / government body).
    let zero = eval(&interest("50000.00", Some("0"), false, "0.00"));
    assert_eq!(zero["amounts"]["withholding"], "0.00");
    assert_eq!(zero["amounts"]["net_payment"], "50000.00");

    // Under $1 disregarded by TAR 2017 s 26(2), even with treaty rate.
    let small = eval(&interest("0.99", Some("0.05"), false, "0.00"));
    assert_eq!(small["amounts"]["payment_used"], "0.00");
    assert_eq!(small["amounts"]["regulation_withholding"], "0.00");
    assert_eq!(small["amounts"]["withholding"], "0.00");
    assert_eq!(small["amounts"]["net_payment"], "0.99");

    // Statutory exemption (e.g. s 128F public offer debenture).
    let ex = eval(&interest("25000.00", None, true, "0.00"));
    assert_eq!(ex["amounts"]["withholding"], "0.00");
    assert_eq!(ex["amounts"]["net_payment"], "25000.00");
}

#[test]
fn royalty_domestic_and_treaty_rates() {
    let dom = eval(&royalty("25000.95", None, false, "0.00"));
    assert_eq!(dom["amounts"]["payment_used"], "25000.00");
    assert_eq!(dom["amounts"]["withholding"], "7500.00");
    assert_eq!(dom["amounts"]["net_payment"], "17500.95");

    let tr = eval(&royalty("25000.95", Some("0.10"), false, "0.00"));
    assert_eq!(tr["amounts"]["withholding"], "2500.00");
    assert_eq!(tr["amounts"]["net_payment"], "22500.95");

    // 5% treaty with fractional result: floor(0.05 * 101) = floor(5.05) = 5.00.
    let frac = eval(&royalty("101.80", Some("0.05"), false, "0.00"));
    assert_eq!(frac["amounts"]["withholding"], "5.00");
    assert_eq!(frac["amounts"]["net_payment"], "96.80");
}

#[test]
fn mit_fund_payments_eoi_clean_building_ncmi_and_non_eoi() {
    // Mixed EOI fund payment: $50k standard (15% = $7,500), $30k clean building (10% = $3,000),
    // $20k NCMI (30% = $6,000) -> $16,500.00.
    let mixed = eval(&mit("100000.00", true, "30000.00", "20000.00", false));
    assert_eq!(mixed["subdivision"], "12-H");
    assert_eq!(mixed["final_tax"], true);
    assert_eq!(mixed["amounts"]["standard_fund_payment"], "50000.00");
    assert_eq!(mixed["amounts"]["clean_building_mit_amount"], "30000.00");
    assert_eq!(mixed["amounts"]["non_concessional_mit_income"], "20000.00");
    assert_eq!(mixed["amounts"]["withholding"], "16500.00");
    assert_eq!(mixed["amounts"]["net_payment"], "83500.00");
    let comps = mixed["components"].as_array().unwrap();
    assert_eq!(comps.len(), 3);
    assert_eq!(comps[0]["withholding"], "7500.00");
    assert_eq!(comps[1]["withholding"], "3000.00");
    assert_eq!(comps[2]["withholding"], "6000.00");

    // Non-EOI: 30% on the whole fund payment regardless of clean building share.
    let non_eoi = eval(&mit("100000.00", false, "30000.00", "20000.00", false));
    assert_eq!(non_eoi["amounts"]["withholding"], "30000.00");
    assert_eq!(non_eoi["amounts"]["net_payment"], "70000.00");
    assert_eq!(non_eoi["components"].as_array().unwrap().len(), 1);
    assert_eq!(
        non_eoi["components"][0]["component"],
        "non_eoi_fund_payment"
    );

    // Cents half-up per component, and components always add up to withholding.
    let cents_case = eval(&mit("300.03", true, "100.01", "100.01", false));
    // 100.01 * 0.15 = 15.0015 -> 15.00; 100.01 * 0.10 = 10.001 -> 10.00; 100.01 * 0.30 = 30.003 -> 30.00.
    assert_eq!(cents_case["amounts"]["withholding"], "55.00");
    assert_eq!(cents_case["amounts"]["net_payment"], "245.03");

    // Half-cent rounds half up: $10.05 * 10% = $1.005 -> $1.01.
    let half_up = eval(&mit("10.05", true, "10.05", "0.00", false));
    assert_eq!(half_up["amounts"]["withholding"], "1.01");

    // Exempt under ss 12-385(5) / 12-390(10)(b).
    let ex = eval(&mit("50000.00", false, "0.00", "0.00", true));
    assert_eq!(ex["amounts"]["withholding"], "0.00");
    assert_eq!(ex["amounts"]["net_payment"], "50000.00");
}

#[test]
fn section_12_315_gaming_junket_and_construction() {
    // Gaming junket 3% with TAR 2017 s 26 boundaries: $33.99 -> $0, $34.00 -> $1, $10,000.90 -> $300.
    let low = eval(&fb_simple("gaming_junket", "33.99", None, false, "0.00"));
    assert_eq!(low["subdivision"], "12-FB");
    assert_eq!(low["final_tax"], false);
    assert_eq!(low["amounts"]["payment_used"], "33.00");
    assert_eq!(low["amounts"]["withholding"], "0.00");
    assert_eq!(low["amounts"]["net_payment"], "33.99");

    let one = eval(&fb_simple("gaming_junket", "34.00", None, false, "0.00"));
    assert_eq!(one["amounts"]["withholding"], "1.00");
    assert_eq!(one["amounts"]["net_payment"], "33.00");

    let ten_k = eval(&fb_simple("gaming_junket", "10000.90", None, false, "0.00"));
    assert_eq!(ten_k["amounts"]["withholding"], "300.00");
    assert_eq!(ten_k["amounts"]["net_payment"], "9700.90");

    // Construction 5% with prior withholding reduction (TAR 2017 s 48(2)).
    let con = eval(&fb_simple(
        "construction",
        "80000.75",
        None,
        false,
        "1000.00",
    ));
    assert_eq!(con["amounts"]["payment_used"], "80000.00");
    assert_eq!(con["amounts"]["withholding_before_prior"], "4000.00");
    assert_eq!(con["amounts"]["withholding"], "3000.00");
    assert_eq!(con["amounts"]["net_payment"], "76000.75");

    // Commissioner s 15-15 variation (e.g. 2%) and s 12-319 exemption.
    let varied = eval(&fb_simple(
        "construction",
        "50000.00",
        Some("0.02"),
        false,
        "0.00",
    ));
    assert_eq!(varied["amounts"]["withholding"], "1000.00");
    assert_eq!(varied["rates"]["variation_rate"], "0.02");

    let exempt = eval(&fb_simple("construction", "50000.00", None, true, "0.00"));
    assert_eq!(exempt["amounts"]["withholding"], "0.00");
    assert_eq!(exempt["amounts"]["net_payment"], "50000.00");
}

#[test]
fn section_12_315_entertainment_or_sports_company_and_individual() {
    // Company: base rate entity 25% vs standard 30% (ITRA 1986 s 23(2)).
    let bre = eval(&entertainment(
        "50000.90",
        "company",
        Some(true),
        None,
        false,
        "0.00",
    ));
    assert_eq!(bre["amounts"]["payment_used"], "50000.00");
    assert_eq!(bre["amounts"]["withholding"], "12500.00");
    assert_eq!(bre["rates"]["rate"], "0.25");

    let std = eval(&entertainment(
        "50000.90",
        "company",
        Some(false),
        None,
        false,
        "0.00",
    ));
    assert_eq!(std["amounts"]["withholding"], "15000.00");
    assert_eq!(std["rates"]["rate"], "0.30");

    // Individual foreign resident marginal rates (ITRA 1986 Sch 7 Pt II):
    // Band 1 ($1,001.99 -> $1,001 @ 30% = $300.30 -> floor = $300.00).
    let ind1 = eval(&entertainment(
        "1001.99",
        "individual",
        None,
        None,
        false,
        "0.00",
    ));
    assert_eq!(ind1["amounts"]["payment_used"], "1001.00");
    assert_eq!(ind1["amounts"]["withholding"], "300.00");
    assert_eq!(ind1["amounts"]["net_payment"], "701.99");

    // Band 2 ($150,000: 135k*30% = 40,500; 15k*37% = 5,550 -> 46,050).
    let ind2 = eval(&entertainment(
        "150000.00",
        "individual",
        None,
        None,
        false,
        "0.00",
    ));
    assert_eq!(ind2["amounts"]["withholding"], "46050.00");

    // Band 3 ($200,000.75 -> 200,000: 40,500 + 20,350 + 4,500 = 65,350).
    let ind3 = eval(&entertainment(
        "200000.75",
        "individual",
        None,
        None,
        false,
        "0.00",
    ));
    assert_eq!(ind3["amounts"]["withholding"], "65350.00");
    assert_eq!(ind3["amounts"]["net_payment"], "134650.75");

    // Variation to nil (e.g. support staff class variation under s 15-15).
    let var0 = eval(&entertainment(
        "20000.00",
        "individual",
        None,
        Some("0"),
        false,
        "0.00",
    ));
    assert_eq!(var0["amounts"]["withholding"], "0.00");

    // All 3 supported years give identical rates.
    for yr in ["2024-25", "2025-26", "2026-27"] {
        let mut inp = entertainment("200000.00", "individual", None, None, false, "0.00");
        inp["year"] = json!(yr);
        assert_eq!(eval(&inp)["amounts"]["withholding"], "65350.00");
    }
}

#[test]
fn refusals_cover_every_code_and_boundary() {
    // payment_type_invalid (including helpful hint for capital_gains).
    let cg = json!({"payment_type": "capital_gains", "payment": "1000.00", "year": "2025-26", "scope_confirmed": true});
    let err = api::evaluate_coded("non-resident-withholding", &cg).unwrap_err();
    assert_eq!(err.code, "non_resident_withholding.payment_type_invalid");
    assert!(err.message.contains("foreign-resident-withholding"));

    assert_eq!(
        code(
            &json!({"payment_type": "other", "payment": "1000.00", "year": "2025-26", "scope_confirmed": true})
        ),
        "non_resident_withholding.payment_type_invalid"
    );

    // 2027-28 refused (TAR 2017 sunsets 1 October 2027).
    let mut y28 = interest("1000.00", None, false, "0.00");
    y28["year"] = json!("2027-28");
    assert_eq!(code(&y28), "period.unsupported");

    // Missing required optional field for type, and extraneous field for type.
    assert_eq!(
        code(
            &json!({"payment_type": "interest", "payment": "1000.00", "treaty_rate": null, "exempt": false, "year": "2025-26", "scope_confirmed": true})
        ),
        "input.missing_field"
    );
    let mut extra = interest("1000.00", None, false, "0.00");
    extra["franked_amount"] = json!("0.00");
    assert_eq!(code(&extra), "input.field_unknown");

    // exempt_exceeds_payment (even when exempt: true).
    assert_eq!(
        code(&dividend("1000.00", "600.00", "500.00", None, true, "0.00")),
        "non_resident_withholding.exempt_exceeds_payment"
    );

    // mit_parts_exceed_payment (even when exempt: true).
    assert_eq!(
        code(&mit("1000.00", true, "600.00", "500.00", true)),
        "non_resident_withholding.mit_parts_exceed_payment"
    );

    // treaty_rate_invalid: above domestic rate, negative, non-string, NaN.
    assert_eq!(
        code(&dividend(
            "1000.00",
            "0.00",
            "0.00",
            Some("0.31"),
            false,
            "0.00"
        )),
        "non_resident_withholding.treaty_rate_invalid"
    );
    assert_eq!(
        code(&interest("1000.00", Some("0.15"), false, "0.00")),
        "non_resident_withholding.treaty_rate_invalid"
    );
    assert_eq!(
        code(&royalty("1000.00", Some("-0.05"), false, "0.00")),
        "non_resident_withholding.treaty_rate_invalid"
    );
    let mut num_rate = interest("1000.00", None, false, "0.00");
    num_rate["treaty_rate"] = json!(0.10);
    assert_eq!(
        code(&num_rate),
        "non_resident_withholding.treaty_rate_invalid"
    );

    // variation_rate_invalid: > 1, negative, non-string.
    assert_eq!(
        code(&fb_simple(
            "construction",
            "1000.00",
            Some("1.01"),
            false,
            "0.00"
        )),
        "non_resident_withholding.variation_rate_invalid"
    );
    assert_eq!(
        code(&fb_simple(
            "gaming_junket",
            "1000.00",
            Some("-0.01"),
            false,
            "0.00"
        )),
        "non_resident_withholding.variation_rate_invalid"
    );

    // already_withheld_exceeds_payment (12-F and 12-FB).
    assert_eq!(
        code(&interest("1000.00", None, false, "1000.01")),
        "non_resident_withholding.already_withheld_exceeds_payment"
    );
    assert_eq!(
        code(&fb_simple(
            "construction",
            "1000.00",
            None,
            false,
            "1000.01"
        )),
        "non_resident_withholding.already_withheld_exceeds_payment"
    );

    // payee_entity_unsupported and base_rate_entity_invalid.
    assert_eq!(
        code(&entertainment(
            "1000.00", "trust", None, None, false, "0.00"
        )),
        "non_resident_withholding.payee_entity_unsupported"
    );
    assert_eq!(
        code(&entertainment(
            "1000.00", "company", None, None, false, "0.00"
        )),
        "non_resident_withholding.base_rate_entity_invalid"
    );
    assert_eq!(
        code(&entertainment(
            "1000.00",
            "individual",
            Some(false),
            None,
            false,
            "0.00"
        )),
        "non_resident_withholding.base_rate_entity_invalid"
    );
}

#[test]
fn catalogue_and_metadata_match() {
    let req = metadata::input_units("non_resident_withholding");
    assert_eq!(req.len(), 2);
    let opt = metadata::optional_inputs("non_resident_withholding");
    assert_eq!(opt.len(), 11);
    assert_eq!(
        metadata::supported_periods("non_resident_withholding"),
        vec!["2024-25", "2025-26", "2026-27"]
    );
}
