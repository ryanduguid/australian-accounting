// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The Division 296 worksheet (ITAA 1997 Subdivision 296-B and the Imposition
//! Act 2026 s 5) against the Explanatory Memorandum's examples 1.1 to 1.3
//! (docs/calculation-evidence.md#division-296), the thresholds and the
//! s 296-40(3) rounding boundaries, the exceptions and its refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(balance: &str, earnings: &str) -> Value {
    json!({
        "total_super_balance_end_of_year": balance, "total_super_earnings": earnings,
        "child_recipient": false, "structured_settlement_contribution": false,
        "died_in_year": false, "year": "2026-27", "scope_confirmed": true,
    })
}

fn run(balance: &str, earnings: &str) -> Value {
    api::evaluate("division-296", &input(balance, earnings)).unwrap()
}

fn with(overrides: Value) -> Value {
    let mut facts = input("4000000.00", "100000.00");
    for (k, v) in overrides.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn refusal_code(facts: &Value) -> &'static str {
    api::evaluate_coded("division-296", facts).unwrap_err().code
}

/// Taxable superannuation earnings, the very large component, the 2 taxes and
/// their total.
fn amounts(result: &Value) -> [&str; 5] {
    [
        "taxable_superannuation_earnings",
        "very_large_balance_earnings_component",
        "tax_on_taxable_superannuation_earnings",
        "tax_on_very_large_balance_earnings_component",
        "division_296_tax",
    ]
    .map(|key| result["amounts"][key].as_str().unwrap())
}

fn percentages(result: &Value) -> (&str, &str) {
    let rates = &result["rates"];
    (
        rates["taxable_percentage"].as_str().unwrap(),
        rates["very_large_percentage"].as_str().unwrap(),
    )
}

const LIABLE: &str = "liable (s 296-15)";
const NOT_ABOVE: &str = "not liable: the total superannuation balance at the end of the year is not greater than the large superannuation balance threshold (s 296-40(1)(a); Income Tax (Transitional Provisions) Act 1997 s 296-1(2))";
const NO_EARNINGS: &str = "not liable: total superannuation earnings for the year are not greater than nil (s 296-40(1)(b))";
const ROUNDS_TO_NIL: &str = "liable (s 296-15), but the s 296-40(3) percentage rounds to 0.00, so taxable superannuation earnings are nil";
const NIL: [&str; 5] = ["0.00"; 5];

/// EM example 1.1: Jordan, $4 million at the end of 2026-27 and $100,000 of
/// earnings: 25% above $3 million, taxable earnings $25,000, tax $3,750.
#[test]
fn jordan_pays_3750_as_the_explanatory_memorandum_says() {
    let jordan = run("4000000.00", "100000.00");
    assert_eq!(percentages(&jordan), ("25.00", "0.00"));
    assert_eq!(
        amounts(&jordan),
        ["25000.00", "0.00", "3750.00", "0.00", "3750.00"]
    );
    assert_eq!(jordan["liability"], LIABLE);
    assert_eq!(jordan["calculation"], "division_296");
    assert_eq!(jordan["period"], "2026-27");
}

/// EM example 1.2: Kelly, $12 million and $500,000: 75% gives $375,000 taxed
/// at 15% ($56,250); 16.67% gives $83,350 taxed at 10% ($8,335); $64,585.
#[test]
fn kelly_pays_64585_as_the_explanatory_memorandum_says() {
    let kelly = run("12000000.00", "500000.00");
    assert_eq!(percentages(&kelly), ("75.00", "16.67"));
    assert_eq!(
        amounts(&kelly),
        ["375000.00", "83350.00", "56250.00", "8335.00", "64585.00"]
    );
    assert_eq!(kelly["liability"], LIABLE);
}

/// EM example 1.3, a 2027-28 example, held here as 2026-27: Kenny's $150,000 at
/// 30 June 2027 is under the threshold, so no taxable superannuation earnings.
/// The example gives no earnings; $10,000 here.
#[test]
fn kenny_under_the_threshold_has_none() {
    let kenny = run("150000.00", "10000.00");
    assert_eq!(amounts(&kenny), NIL);
    assert_eq!(percentages(&kenny), ("0.00", "0.00"));
    assert_eq!(kenny["liability"], NOT_ABOVE);
}

/// 'Greater than' each threshold (ss 296-40(1)(a) and 296-45(1)), with $100,000
/// of earnings: a balance at or a cent either side of $3 million gives no tax,
/// and at or a cent either side of $10 million gives 70.00% and $10,500.
#[test]
fn each_threshold_must_be_exceeded() {
    for balance in ["0.00", "2999999.99", "3000000.00"] {
        let result = run(balance, "100000.00");
        assert_eq!(amounts(&result), NIL, "{balance}");
        assert_eq!(result["liability"], NOT_ABOVE, "{balance}");
    }
    let cent_over = run("3000000.01", "100000.00");
    assert_eq!(percentages(&cent_over), ("0.00", "0.00"));
    assert_eq!(amounts(&cent_over), NIL);
    assert_eq!(cent_over["liability"], ROUNDS_TO_NIL);
    for balance in ["9999999.99", "10000000.00", "10000000.01"] {
        let result = run(balance, "100000.00");
        assert_eq!(percentages(&result), ("70.00", "0.00"), "{balance}");
        assert_eq!(
            amounts(&result),
            ["70000.00", "0.00", "10500.00", "0.00", "10500.00"],
            "{balance}"
        );
        assert_eq!(result["liability"], LIABLE, "{balance}");
    }
}

/// s 296-40(3) and s 296-45(3) round up when the third decimal place is 5 or
/// more: $150 over $3 million is 0.004999...% and $150.01 is 0.005000...%;
/// $500.02 over $10 million is 0.004999...% and $500.03 is 0.005000...%.
#[test]
fn percentages_round_half_up_at_the_third_decimal_place() {
    assert_eq!(percentages(&run("3000150.00", "100000.00")).0, "0.00");
    let over = run("3000150.01", "100000.00");
    assert_eq!(percentages(&over).0, "0.01");
    assert_eq!(amounts(&over), ["10.00", "0.00", "1.50", "0.00", "1.50"]);
    assert_eq!(percentages(&run("10000500.02", "100000.00")).1, "0.00");
    let very = run("10000500.03", "100000.00");
    assert_eq!(percentages(&very), ("70.00", "0.01"));
    assert_eq!(
        amounts(&very),
        ["70000.00", "10.00", "10500.00", "1.00", "10501.00"]
    );
}

/// Exact ties at the third decimal place round up: $32 million gives 90.625%
/// and 68.75%; $64 million gives 95.3125% and 84.375%.
#[test]
fn percentage_ties_round_up() {
    let thirty_two = run("32000000.00", "100000.00");
    assert_eq!(percentages(&thirty_two), ("90.63", "68.75"));
    assert_eq!(
        amounts(&thirty_two),
        ["90630.00", "68750.00", "13594.50", "6875.00", "20469.50"]
    );
    let sixty_four = run("64000000.00", "100000.00");
    assert_eq!(percentages(&sixty_four), ("95.31", "84.38"));
    assert_eq!(
        amounts(&sixty_four),
        ["95310.00", "84380.00", "14296.50", "8438.00", "22734.50"]
    );
}

/// At the largest accepted inputs both shares (99.9997% and 99.999%) round up
/// to 100.00%, so the tax is 25% of the earnings.
#[test]
fn the_largest_inputs_round_both_shares_to_100() {
    let largest = run("1000000000000.00", "1000000000000.00");
    assert_eq!(percentages(&largest), ("100.00", "100.00"));
    assert_eq!(
        amounts(&largest),
        [
            "1000000000000.00",
            "1000000000000.00",
            "150000000000.00",
            "100000000000.00",
            "250000000000.00",
        ]
    );
}

/// Amounts are exact until displayed: 25% of $0.38 is $0.095, shown as $0.10,
/// and 15% of it $0.01425, shown as $0.01 (not 15% of $0.10); $12 million and
/// $0.04 give $0.0045 + $0.000667, so $0.01; 50% of $0.20 taxed is $0.015, so $0.02.
#[test]
fn amounts_round_to_cents_only_for_display() {
    assert_eq!(
        amounts(&run("4000000.00", "0.38")),
        ["0.10", "0.00", "0.01", "0.00", "0.01"]
    );
    // The displayed taxes add to $0.00 but the total shows $0.01, as the warning says.
    assert_eq!(
        amounts(&run("12000000.00", "0.04")),
        ["0.03", "0.01", "0.00", "0.00", "0.01"]
    );
    assert_eq!(amounts(&run("6000000.00", "0.20"))[4], "0.02");
}

/// Earnings of nil or less give no tax and never a refund, even above
/// $10 million, where s 296-45 alone has no 'greater than nil' condition. The
/// very large component it then defines is negative, and as results carry no
/// negative amounts it is reported as null.
#[test]
fn nil_or_negative_earnings_give_no_tax() {
    for (earnings, component) in [
        ("0.00", json!("0.00")),
        ("-50000.00", Value::Null),
        ("-1000000000000.00", Value::Null),
    ] {
        let result = run("12000000.00", earnings);
        let shown = &result["amounts"];
        assert_eq!(
            shown["very_large_balance_earnings_component"], component,
            "{earnings}"
        );
        for key in [
            "taxable_superannuation_earnings",
            "tax_on_taxable_superannuation_earnings",
            "tax_on_very_large_balance_earnings_component",
            "division_296_tax",
        ] {
            assert_eq!(shown[key], "0.00", "{earnings} {key}");
        }
        assert_eq!(percentages(&result), ("75.00", "16.67"), "{earnings}");
        assert_eq!(result["liability"], NO_EARNINGS, "{earnings}");
    }
    // Just above $10 million the share rounds to 0.00, so the component is nil, not -0.00.
    let barely = run("10000000.01", "-50000.00");
    assert_eq!(amounts(&barely), NIL);
}

/// Each exception means no tax, in the order the Act and the transitional
/// rule give them, once every input has been read. ss 296-20 and 296-25 remove
/// the liability, not the earnings s 296-40 defines; after death the balance is
/// nil (s 296-50), so the percentages and earnings are nil too.
#[test]
fn each_exception_means_no_tax() {
    let jordan_untaxed = ["25000.00", "0.00", "0.00", "0.00", "0.00"];
    for (flag, reason, expected, shares) in [
        (
            "child_recipient",
            "not liable: a child recipient of a superannuation income stream in the year (s 296-20)",
            jordan_untaxed,
            ("25.00", "0.00"),
        ),
        (
            "structured_settlement_contribution",
            "not liable: a structured settlement contribution was made in respect of the individual in or before the year (s 296-25)",
            jordan_untaxed,
            ("25.00", "0.00"),
        ),
        (
            "died_in_year",
            "not liable: died on or before the last day of the 2026-27 income year (Income Tax (Transitional Provisions) Act 1997 s 296-1(3))",
            NIL,
            ("0.00", "0.00"),
        ),
    ] {
        let mut facts = with(json!({}));
        facts[flag] = json!(true);
        let result = api::evaluate("division-296", &facts).unwrap();
        assert_eq!(amounts(&result), expected, "{flag}");
        assert_eq!(percentages(&result), shares, "{flag}");
        assert_eq!(result["liability"], reason, "{flag}");
    }
    let kelly_died = with(json!({
        "total_super_balance_end_of_year": "12000000.00",
        "total_super_earnings": "500000.00", "died_in_year": true,
    }));
    let result = api::evaluate("division-296", &kelly_died).unwrap();
    assert_eq!(amounts(&result), NIL);
    assert_eq!(percentages(&result), ("0.00", "0.00"));
    let both = with(json!({"child_recipient": true, "died_in_year": true}));
    assert_eq!(
        api::evaluate("division-296", &both).unwrap()["liability"],
        "not liable: a child recipient of a superannuation income stream in the year (s 296-20)"
    );
    // An exception does not excuse a malformed input.
    let malformed = with(json!({"child_recipient": true, "died_in_year": "no"}));
    assert_eq!(refusal_code(&malformed), "input.flag_invalid");
    let negative =
        with(json!({"child_recipient": true, "total_super_balance_end_of_year": "-1.00"}));
    assert_eq!(refusal_code(&negative), "input.amount_invalid");
}

#[test]
fn out_of_scope_and_malformed_input_is_refused() {
    for (overrides, code) in [
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"year": "2025-26"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (
            json!({"total_super_balance_end_of_year": "-0.01"}),
            "input.amount_invalid",
        ),
        (
            json!({"total_super_balance_end_of_year": "4000000.001"}),
            "input.amount_invalid",
        ),
        (
            json!({"total_super_balance_end_of_year": 4000000}),
            "input.amount_invalid",
        ),
        (
            json!({"total_super_earnings": "100.005"}),
            "input.signed_amount_invalid",
        ),
        (
            json!({"total_super_earnings": "-1000000000000.01"}),
            "input.signed_amount_invalid",
        ),
        (
            json!({"total_super_earnings": "1000000000000.01"}),
            "input.signed_amount_invalid",
        ),
        (
            json!({"total_super_earnings": "NaN"}),
            "input.signed_amount_invalid",
        ),
        (
            json!({"total_super_earnings": 100000}),
            "input.signed_amount_invalid",
        ),
        (
            json!({"structured_settlement_contribution": null}),
            "input.flag_invalid",
        ),
        (json!({"died_in_year": 0}), "input.flag_invalid"),
        // A field the worksheet does not take is refused, never ignored.
        (
            json!({"total_super_balance_start_of_year": "9000000.00"}),
            "input.field_unknown",
        ),
        (
            json!({"defined_benefit_interest": true}),
            "input.field_unknown",
        ),
    ] {
        assert_eq!(refusal_code(&with(overrides.clone())), code, "{overrides}");
    }
    for field in [
        "total_super_balance_end_of_year",
        "total_super_earnings",
        "child_recipient",
        "structured_settlement_contribution",
        "died_in_year",
        "year",
        "scope_confirmed",
    ] {
        let mut facts = with(json!({}));
        facts.as_object_mut().unwrap().remove(field);
        assert_eq!(refusal_code(&facts), "input.missing_field", "{field}");
    }
}

/// Every result says it is an estimate, with the provisions it applies and the
/// worksheet warnings every result carries.
#[test]
fn every_result_is_labelled_an_estimate_with_its_sources() {
    let result = run("4000000.00", "100000.00");
    let warnings: Vec<&str> = result["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap())
        .collect();
    assert_eq!(warnings.len(), 4);
    assert!(warnings[1].starts_with("Worksheet only, not advice, an assessment or a lodgment."));
    assert!(warnings[2].starts_with("An estimate, not the tax payable:"));
    let scope = result["scope"].as_str().unwrap();
    for provision in [
        "s 296-40(1)",
        "s 296-260",
        "s 296-1(2)",
        "Imposition Act 2026 s 5",
    ] {
        assert!(scope.contains(provision), "{provision}");
    }
    let sources = result["sources"].as_array().unwrap();
    assert!(sources.contains(&json!("https://www.legislation.gov.au/C2026A00009/asmade")));
    assert_eq!(result["rates"]["rate"], "0.15");
    assert_eq!(result["rates"]["very_large_rate"], "0.10");
    assert_eq!(result["source_checked"], "2026-11-13");
}
