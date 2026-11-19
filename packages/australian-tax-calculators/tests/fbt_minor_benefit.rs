// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The minor benefits exemption screen (FBTAA s 58P, read on 19 November
//! 2026) against TR 2007/12's examples, the Library's Tax Examples, FBT,
//! 3-060 and 3-080, and the edge of each condition
//! (docs/calculation-evidence.md#fbt-minor-benefit-check).
use australian_tax_calculators::api;
use serde_json::{Value, json};

/// A benefit worth `value` whose criteria all weigh for the exemption.
fn benefit(category: &str, value: &str) -> Value {
    let mut input = json!({
        "category": category, "notional_taxable_value": value,
        "salary_sacrifice": false, "frequency": "infrequent_and_irregular",
        "similar_benefits_substantial": false,
        "other_associated_benefits_substantial": false,
        "valuation_difficult": false, "unexpected_event": false,
        "reward_for_services": false, "year_ended": 2026, "scope_confirmed": true,
    });
    match category {
        "expense_payment" | "property" | "residual" => input["in_house"] = json!(false),
        "meal_entertainment" | "entertainment_facility_leasing" => {
            input["fifty_fifty_split"] = json!(false);
        }
        "tax_exempt_body_entertainment" => input["tax_exempt_body_exception"] = json!("none"),
        _ => {}
    }
    input
}

fn with(mut input: Value, pairs: &[(&str, Value)]) -> Value {
    for (key, value) in pairs {
        input[*key] = value.clone();
    }
    input
}

fn run(input: &Value) -> Value {
    api::evaluate("fbt-minor-benefit-check", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("fbt-minor-benefit-check", input)
        .unwrap_err()
        .code
}

/// The verdict, the screen and the failing conditions.
fn screen(input: &Value) -> (Value, String, Value) {
    let r = run(input);
    let name = r["screen"].as_str().unwrap().to_string();
    (r["exempt"].clone(), name, r["failing_conditions"].clone())
}

/// Tax Examples, FBT, 3-060 (b): an $80 bouquet on the employee's birthday,
/// a property benefit provided infrequently and irregularly, is an exempt
/// minor benefit. It is the catalogue's example.
#[test]
fn birthday_flowers_are_exempt() {
    let input = benefit("property", "80.00");
    let r = run(&input);
    assert_eq!(r["exempt"], json!(true));
    assert_eq!(r["screen"], "exempt");
    assert_eq!(r["amounts"]["notional_taxable_value"], "80.00");
    assert_eq!(r["rates"]["minor_benefit_threshold"], "300");
    assert_eq!(r["period"], "year ended 31 March 2026");
    assert!(r["consequences"][0].as_str().unwrap().contains("para (g)"));
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut facts = catalogue["fbt_minor_benefit_check"]["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_eq!(run(&facts)["exempt"], json!(true));
}

/// Tax Examples, FBT, 3-060 (a): gym fees of $50 a month, paid every month,
/// $600 a year, and 3-080: monthly golf days costing about $1,000 a year.
/// Each payment is under $300, but provided frequently with a substantial
/// total, as in TR 2007/12 example 8, so not exempt.
#[test]
fn frequent_payments_with_a_substantial_total_are_not_exempt() {
    for value in ["50.00", "83.33"] {
        let input = with(
            benefit("expense_payment", value),
            &[
                ("frequency", json!("frequent")),
                ("similar_benefits_substantial", json!(true)),
            ],
        );
        let (exempt, name, failing) = screen(&input);
        assert_eq!(
            (exempt, name.as_str(), failing),
            (json!(false), "not_exempt", json!([]))
        );
    }
}

/// TR 2007/12 examples 1 to 3, 5 to 7 and 9 to 11 reach the exemption: a
/// Christmas gift or party each year (infrequent but regular), flowers on
/// special occasions, a car lent after a storm (an unexpected event), ad hoc
/// road tolls (hard to value), a recognition voucher (a reward, but ad hoc
/// and not substantial), a one-off gym membership and babysitting during
/// unexpected overtime.
#[test]
fn the_rulings_exempt_examples_are_exempt() {
    let regular = [("frequency", json!("infrequent_and_regular"))];
    for input in [
        with(benefit("property", "150.00"), &regular),
        with(benefit("meal_entertainment", "180.00"), &regular),
        benefit("property", "60.00"),
        with(
            benefit("other", "45.00"),
            &[("unexpected_event", json!(true))],
        ),
        with(
            benefit("expense_payment", "6.50"),
            &[("valuation_difficult", json!(true))],
        ),
        with(
            benefit("property", "200.00"),
            &[("reward_for_services", json!(true))],
        ),
        benefit("expense_payment", "240.00"),
        with(
            benefit("expense_payment", "120.00"),
            &[("unexpected_event", json!(true))],
        ),
    ] {
        assert_eq!(screen(&input).0, json!(true), "{input}");
    }
}

/// TR 2007/12 example 8: monthly sales incentive vouchers under $300,
/// received several times this year and last, a reward for services: not
/// exempt.
#[test]
fn incentive_vouchers_are_not_exempt() {
    let input = with(
        benefit("property", "250.00"),
        &[
            ("frequency", json!("frequent")),
            ("similar_benefits_substantial", json!(true)),
            ("reward_for_services", json!(true)),
        ],
    );
    let r = run(&input);
    assert_eq!(r["exempt"], json!(false));
    assert!(r["reason"].as_str().unwrap().contains("example 8"));
    let weighs: Vec<&str> = r["criteria"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["weighs"].as_str().unwrap())
        .collect();
    assert_eq!(
        weighs,
        ["against", "against", "for", "neutral", "neutral", "against"]
    );
}

/// TR 2007/12 example 12: a tax-exempt body's movie tickets for a work
/// target are entertainment it incurs non-deductible exempt entertainment
/// expenditure on, outside both s 58P(1)(d) exceptions; example 13: a benefit
/// under a salary sacrifice arrangement. Neither is exempt.
#[test]
fn tax_exempt_body_entertainment_and_salary_sacrifice_are_excluded() {
    let (exempt, _, failing) = screen(&benefit("tax_exempt_body_entertainment", "40.00"));
    assert_eq!(
        (exempt, failing),
        (json!(false), json!(["tax_exempt_body_exception"]))
    );
    for exception in [
        "incidental_to_outsiders_no_meal",
        "special_achievement_on_premises",
    ] {
        let input = with(
            benefit("tax_exempt_body_entertainment", "40.00"),
            &[("tax_exempt_body_exception", json!(exception))],
        );
        assert_eq!(screen(&input).0, json!(true), "{exception}");
    }
    let input = with(
        benefit("other", "100.00"),
        &[("salary_sacrifice", json!(true))],
    );
    let (exempt, _, failing) = screen(&input);
    assert_eq!(
        (exempt, failing),
        (json!(false), json!(["not_salary_sacrifice"]))
    );
}

/// $300 itself is not less than $300 (s 58P(1)(e)); $299.99 is.
#[test]
fn the_threshold_is_less_than_300() {
    let (exempt, _, failing) = screen(&benefit("property", "300.00"));
    assert_eq!((exempt, failing), (json!(false), json!(["less_than_300"])));
    assert_eq!(screen(&benefit("property", "299.99")).0, json!(true));
}

/// An in-house benefit (s 58P(1)(c)) and meal entertainment under the 50/50
/// split (TR 2007/12 para 14) fail; several failures are each named.
#[test]
fn each_exclusion_fails_alone_and_together() {
    for category in ["expense_payment", "property", "residual"] {
        let input = with(benefit(category, "100.00"), &[("in_house", json!(true))]);
        assert_eq!(screen(&input).2, json!(["not_in_house"]), "{category}");
    }
    for category in ["meal_entertainment", "entertainment_facility_leasing"] {
        let input = with(
            benefit(category, "100.00"),
            &[("fifty_fifty_split", json!(true))],
        );
        assert_eq!(
            screen(&input).2,
            json!(["not_fifty_fifty_split"]),
            "{category}"
        );
    }
    let input = with(
        benefit("residual", "300.00"),
        &[("in_house", json!(true)), ("salary_sacrifice", json!(true))],
    );
    assert_eq!(
        screen(&input).2,
        json!(["not_in_house", "not_salary_sacrifice", "less_than_300"])
    );
}

/// Criteria that match neither the exempt examples nor example 8 are left to
/// the employer: frequent but not substantial, or infrequent with a
/// substantial total.
#[test]
fn mixed_criteria_need_the_employers_judgement() {
    for pairs in [
        vec![("frequency", json!("frequent"))],
        vec![("similar_benefits_substantial", json!(true))],
        vec![("other_associated_benefits_substantial", json!(true))],
        vec![
            ("frequency", json!("frequent")),
            ("other_associated_benefits_substantial", json!(true)),
        ],
    ] {
        let r = run(&with(benefit("property", "100.00"), &pairs));
        assert_eq!(r["exempt"], Value::Null);
        assert_eq!(r["screen"], "judgement_required");
        assert_eq!(r["consequences"].as_array().unwrap().len(), 2);
    }
}

/// Each fact outside its values, each category's fact missing or given to
/// another category, and each period outside the years read is refused.
#[test]
fn bad_inputs_are_refused() {
    let base = benefit("property", "100.00");
    for (key, value) in [
        ("category", json!("fringe")),
        ("frequency", json!("sometimes")),
    ] {
        assert_eq!(
            code(&with(base.clone(), &[(key, value)])),
            "fbt_minor_benefit.fact_invalid"
        );
    }
    let input = with(
        benefit("tax_exempt_body_entertainment", "40.00"),
        &[("tax_exempt_body_exception", json!("party"))],
    );
    assert_eq!(code(&input), "fbt_minor_benefit.fact_invalid");
    let mut missing = base.clone();
    missing.as_object_mut().unwrap().remove("in_house");
    assert_eq!(code(&missing), "input.missing_field");
    let mut missing = benefit("meal_entertainment", "100.00");
    missing.as_object_mut().unwrap().remove("fifty_fifty_split");
    assert_eq!(code(&missing), "input.missing_field");
    let mut missing = benefit("tax_exempt_body_entertainment", "40.00");
    missing
        .as_object_mut()
        .unwrap()
        .remove("tax_exempt_body_exception");
    assert_eq!(code(&missing), "input.missing_field");
    for (key, value) in [
        ("in_house", json!(false)),
        ("fifty_fifty_split", json!(false)),
        ("tax_exempt_body_exception", json!("none")),
    ] {
        let input = with(benefit("other", "100.00"), &[(key, value)]);
        assert_eq!(code(&input), "input.field_unknown", "{key}");
    }
    for key in [
        "salary_sacrifice",
        "similar_benefits_substantial",
        "other_associated_benefits_substantial",
        "valuation_difficult",
        "unexpected_event",
        "reward_for_services",
        "in_house",
    ] {
        assert_eq!(
            code(&with(base.clone(), &[(key, json!("no"))])),
            "input.flag_invalid",
            "{key}"
        );
    }
    assert_eq!(
        code(&with(
            base.clone(),
            &[("notional_taxable_value", json!("-1"))]
        )),
        "input.amount_invalid"
    );
    assert_eq!(
        code(&with(base.clone(), &[("year_ended", json!(2028))])),
        "period.unsupported"
    );
    assert_eq!(
        code(&with(base.clone(), &[("scope_confirmed", json!(false))])),
        "scope.not_confirmed"
    );
    assert_eq!(
        code(&with(base, &[("not_an_input", json!(1))])),
        "input.field_unknown"
    );
}
