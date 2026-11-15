// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `derive` explains each amount of the worksheets it covers (FBT, Division 293
//! and the pension minimum) in a step a reader can check alone: every
//! name in a step's formula is an input, a rate or an earlier step, its operands
//! are those values, and they recompute the amount. These tests recompute every
//! amount that way, independently of the engine's arithmetic.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::{Decimal, Rounding, with_precision};
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

/// The commands `derive` explains.
const DERIVED: [&str; 11] = [
    "fbt",
    "fbt-car-statutory",
    "fbt-car-operating-cost",
    "fbt-loan",
    "fbt-meal-entertainment",
    "fbt-car-parking",
    "fbt-debt-waiver",
    "fbt-expense-payment",
    "fbt-property",
    "division-293",
    "pension-minimum",
];

/// (command, input) for each derived kind: the ATO's published examples and
/// the branches each worksheet takes.
fn cases() -> Vec<(&'static str, Value)> {
    let division_293 = |income: &str, contributions: &str| {
        (
            "division-293",
            json!({"division_293_income": income, "low_tax_contributions": contributions, "year": "2025-26", "scope_confirmed": true}),
        )
    };
    let pension = |balance: &str, age: i64, days: i64| {
        (
            "pension-minimum",
            json!({"account_balance": balance, "age": age, "days": days, "year": "2024-25", "scope_confirmed": true}),
        )
    };
    let fbt = |one: &str, two: &str, year: i64| json!({"type_one_value": one, "type_two_value": two, "year_ended": year, "scope_confirmed": true});
    let statutory = |overrides: Value| {
        let mut input = json!({
            "cost_price": "32000.00", "later_accessories": "0.00", "one_third_reduction": false,
            "days_available": 274, "employee_contribution": "0.00", "electric_car_exempt": false,
            "year_ended": 2026, "scope_confirmed": true,
        });
        merge(&mut input, overrides);
        ("fbt-car-statutory", input)
    };
    let operating = |overrides: Value| {
        let mut input = json!({
            "running_costs": "5200.00", "lease_charges": "0.00", "depreciated_value": "50000.00",
            "effective_life_years": "8", "days_held": 365, "business_use": "0.3636",
            "employee_contribution": "1500.00", "statutory_formula_taxable_value": "8500.00",
            "electric_car_exempt": false, "year_ended": 2026, "scope_confirmed": true,
        });
        merge(&mut input, overrides);
        ("fbt-car-operating-cost", input)
    };
    vec![
        // Calculating your FBT, MantCo, in both years.
        ("fbt", fbt("16500.00", "6000.00", 2026)),
        ("fbt", fbt("16500.00", "6000.00", 2027)),
        ("fbt", fbt("1000.00", "1000.00", 2026)),
        ("fbt", fbt("0.00", "0.00", 2027)),
        ("fbt", fbt("0.01", "999999999999.99", 2026)),
        // FBT return 2026 instructions, item 23A example 6.
        statutory(json!({})),
        // The employer's guide, example 26: a one-third reduction and an accessory.
        statutory(
            json!({"cost_price": "30000.00", "later_accessories": "1000.00",
                         "one_third_reduction": true, "days_available": 365}),
        ),
        statutory(
            json!({"cost_price": "50000.02", "one_third_reduction": true, "days_available": 164}),
        ),
        statutory(json!({"employee_contribution": "5000.00", "year_ended": 2027})),
        statutory(json!({"electric_car_exempt": true, "days_available": 0})),
        // The Library's car example, ¶3-020, where s 10(5) takes the statutory value.
        operating(json!({})),
        // FBT return 2026 instructions, item 23B example 7.
        operating(
            json!({"running_costs": "10000.00", "depreciated_value": "0.00",
                         "business_use": "0.7", "employee_contribution": "0.00"}),
        ),
        operating(
            json!({"depreciated_value": "40000.00", "days_held": 100, "running_costs": "500.00",
                         "business_use": "0.5", "employee_contribution": "0.00",
                         "statutory_formula_taxable_value": "99999.00", "year_ended": 2027}),
        ),
        operating(
            json!({"lease_charges": "9000.00", "depreciated_value": "0.00",
                         "effective_life_years": "0", "electric_car_exempt": true}),
        ),
        operating(json!({"effective_life_years": "3", "employee_contribution": "90000.00"})),
        // The ATO's Division 293 example (Jan): taxable contributions of $5,000.
        division_293("240000.00", "15000.00"),
        division_293("200000.00", "20000.00"),
        division_293("300000.00", "25000.00"),
        division_293("400000.00", "0.00"),
        // A full year, then the evidence's adaptation of the ATO's Thavi example:
        // from 1 January, 181 of 365 days.
        pension("600000.00", 67, 365),
        pension("250000.00", 66, 181),
        // Started on 1 June: cl 4 requires nothing.
        pension("250000.00", 66, 30),
        // $4,005.00 ends in an exact $5, so cl 5 rounds it up.
        pension("100125.00", 60, 365),
        // FBT return 2026 instructions, item 23C example 8, and the Library's ¶3-100.
        benefit(
            "fbt-loan",
            json!({"daily_balance_total": "7300000.00", "interest_accrued": "0.00",
                   "income_use": "0", "accrued_interest_deduction": "0.00"}),
        ),
        benefit(
            "fbt-loan",
            json!({"daily_balance_total": "36500000.00", "interest_accrued": "3450.00",
                   "income_use": "0.4", "accrued_interest_deduction": "1380.00"}),
        ),
        benefit(
            "fbt-loan",
            json!({"daily_balance_total": "18250000.00", "interest_accrued": "2000.00",
                   "income_use": "1", "accrued_interest_deduction": "0.00", "year_ended": 2027}),
        ),
        // Item 23P example 18, and a register percentage that does not terminate.
        benefit(
            "fbt-meal-entertainment",
            json!({"method": "50_50_split", "meal_entertainment_expenditure": "4000.00"}),
        ),
        benefit(
            "fbt-meal-entertainment",
            json!({"method": "12_week_register", "meal_entertainment_expenditure": "900.00",
                   "register_fringe_benefits_value": "1000.00",
                   "register_meal_entertainment_value": "3000.00", "register_first_year": 2026}),
        ),
        // The employer's guide, examples 15 and 17, and a fee at the threshold.
        benefit(
            "fbt-car-parking",
            json!({"method": "average_cost", "first_business_day_fee": "17.00", "benefits": 3,
                   "first_day_fee": "17.25", "last_day_fee": "18.50",
                   "employee_contribution": "20.00"}),
        ),
        benefit(
            "fbt-car-parking",
            json!({"method": "12_week_register", "first_business_day_fee": "20.00",
                   "register_value": "2500.00", "availability_days": 183,
                   "register_first_year": 2026}),
        ),
        benefit(
            "fbt-car-parking",
            json!({"method": "commercial_parking_station", "first_business_day_fee": "11.48",
                   "benefits": 230, "lowest_fee": "18.00", "employee_contribution": "1000.00",
                   "year_ended": 2027}),
        ),
        // FBT return 2026 instructions, item 23D example 9.
        benefit("fbt-debt-waiver", json!({"amount_waived": "500.00"})),
        // The employer's guide, 9.4's first example and 9.3's in-house petrol,
        // with the employee's $1,000 in-house reduction unused.
        benefit(
            "fbt-expense-payment",
            json!({"method": "external", "recipients_expenditure": "500.00",
                   "amount_paid": "250.00", "employee_contribution": "0.00", "income_use": "0.8",
                   "recipients_portion_deduction": "200.00"}),
        ),
        benefit(
            "fbt-expense-payment",
            json!({"method": "in_house_acquired", "recipients_expenditure": "1000.00",
                   "amount_paid": "250.00", "employee_contribution": "0.00",
                   "arms_length_price": "800.00", "notional_value": "1000.00",
                   "in_house_reduction_available": "1000.00"}),
        ),
        // Item 23K example 14, the guide's 17.5 first example, and the
        // otherwise deductible rule before the in-house reduction.
        benefit(
            "fbt-property",
            json!({"method": "in_house_public_price", "lowest_price": "3600.00",
                   "employee_contribution": "300.00", "income_use": "0",
                   "contribution_deduction": "0.00", "in_house_reduction_available": "1000.00"}),
        ),
        benefit(
            "fbt-property",
            json!({"method": "external_notional_value", "notional_value": "500.00",
                   "employee_contribution": "250.00", "income_use": "0.8",
                   "contribution_deduction": "200.00"}),
        ),
        benefit(
            "fbt-property",
            json!({"method": "in_house_acquired", "arms_length_price": "500.00",
                   "notional_value": "800.00", "employee_contribution": "0.00",
                   "income_use": "0.5", "contribution_deduction": "0.00",
                   "in_house_reduction_available": "100.00", "year_ended": 2027}),
        ),
    ]
}

/// An FBT benefit worksheet's input: `facts` for the year ended 31 March 2026
/// unless they say otherwise.
fn benefit(command: &'static str, facts: Value) -> (&'static str, Value) {
    let mut input = json!({"year_ended": 2026, "scope_confirmed": true});
    merge(&mut input, facts);
    (command, input)
}

fn merge(input: &mut Value, overrides: Value) {
    for (key, value) in overrides.as_object().unwrap() {
        input[key] = value.clone();
    }
}

fn derive(command: &str, input: &Value) -> Value {
    api::evaluate("derive", &json!({"command": command, "input": input})).unwrap()
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(String),
    Name(String),
    Function(String),
    Symbol(char),
}

/// A formula or its operands as tokens: numbers, names, functions (a name
/// before `(`), the operators `+`, `-`, `x` and `/`, brackets and commas. A
/// minus written against digits is a number's sign, as in the -0.00 that an
/// input of "-0.00" keeps; subtraction is written with spaces.
fn tokens(text: &str) -> Vec<Token> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let start = i;
        if c == ' ' {
            i += 1;
        } else if c.is_ascii_digit()
            || (c == '-' && chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || chars[i] == '.'
                    || chars[i] == 'E'
                    || (matches!(chars[i], '+' | '-') && chars[i - 1] == 'E'))
            {
                i += 1;
            }
            out.push(Token::Number(chars[start..i].iter().collect()));
        } else if c.is_ascii_lowercase() {
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            out.push(match word.as_str() {
                "x" => Token::Symbol('x'),
                _ if chars.get(i) == Some(&'(') => Token::Function(word),
                _ => Token::Name(word),
            });
        } else {
            assert!("+-/(),".contains(c), "unexpected {c:?} in {text:?}");
            out.push(Token::Symbol(c));
            i += 1;
        }
    }
    out
}

fn dec(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

/// Evaluates operand tokens in the worksheets' 40-digit context: `x` and `/`
/// before `+` and `-`, each left to right.
struct Evaluator<'a> {
    tokens: &'a [Token],
    at: usize,
}

impl Evaluator<'_> {
    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.at);
        self.at += 1;
        token
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn sum(&mut self) -> Decimal {
        let mut value = self.product();
        while let Some(Token::Symbol(op @ ('+' | '-'))) = self.peek().cloned() {
            self.at += 1;
            let right = self.product();
            value = if op == '+' {
                value.add(&right)
            } else {
                value.sub(&right)
            }
            .unwrap();
        }
        value
    }

    fn product(&mut self) -> Decimal {
        let mut value = self.factor();
        while let Some(Token::Symbol(op @ ('x' | '/'))) = self.peek().cloned() {
            self.at += 1;
            let right = self.factor();
            value = if op == 'x' {
                value.mul(&right)
            } else {
                value.div(&right)
            }
            .unwrap();
        }
        value
    }

    fn factor(&mut self) -> Decimal {
        match self.next().cloned() {
            Some(Token::Number(text)) => dec(&text),
            Some(Token::Symbol('(')) => {
                let value = self.sum();
                assert_eq!(self.next(), Some(&Token::Symbol(')')));
                value
            }
            Some(Token::Function(name)) => {
                assert_eq!(self.next(), Some(&Token::Symbol('(')));
                let mut args = vec![self.sum()];
                while self.peek() == Some(&Token::Symbol(',')) {
                    self.at += 1;
                    args.push(self.sum());
                }
                assert_eq!(self.next(), Some(&Token::Symbol(')')));
                match (name.as_str(), &args[..]) {
                    ("min", [a, b]) => if b.lt(a).unwrap() { b } else { a }.clone(),
                    ("max", [a, b]) => if b.gt(a).unwrap() { b } else { a }.clone(),
                    ("cents", [a]) => a.quantize(-2, Rounding::HalfUp).unwrap(),
                    ("dollars", [a]) => a.quantize(0, Rounding::HalfUp).unwrap(),
                    other => panic!("unknown function {other:?}"),
                }
            }
            other => panic!("expected a value, found {other:?}"),
        }
    }
}

fn evaluate(operands: &[Token]) -> Decimal {
    with_precision(40, || {
        let mut evaluator = Evaluator {
            tokens: operands,
            at: 0,
        };
        let value = evaluator.sum();
        assert_eq!(evaluator.at, operands.len(), "trailing tokens");
        value
    })
}

/// The value a formula's name stands for: an earlier step's exact value, a
/// rate, or an input as given.
fn named(name: &str, earlier: &[(String, String)], doc: &Value) -> Option<String> {
    if let Some((_, exact)) = earlier.iter().rev().find(|(id, _)| id == name) {
        return Some(exact.clone());
    }
    if let Some(rate) = doc["result"]["rates"][name].as_str() {
        return Some(rate.to_string());
    }
    match &doc["input"][name] {
        Value::String(text) => Some(text.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// What a reader checking `doc` from its steps alone would find wrong; empty
/// when every amount follows from its step.
fn problems(doc: &Value) -> Vec<String> {
    let mut found = Vec::new();
    let amounts = doc["result"]["amounts"].as_object().unwrap();
    let steps = doc["steps"].as_array().unwrap();
    let ids: Vec<&str> = steps.iter().map(|s| s["id"].as_str().unwrap()).collect();
    let keys: Vec<&str> = amounts.keys().map(String::as_str).collect();
    if ids != keys {
        found.push(format!("steps {ids:?} do not match amounts {keys:?}"));
    }
    let mut earlier: Vec<(String, String)> = Vec::new();
    for step in steps {
        let id = step["id"].as_str().unwrap();
        if step["authority"].as_str().is_none_or(str::is_empty) {
            found.push(format!("{id}: no authority"));
        }
        let working = step["working"].as_str().unwrap();
        let parts: Vec<&str> = working.split(" = ").collect();
        let [formula, operands, exact] = parts[..] else {
            found.push(format!(
                "{id}: working {working:?} is not formula = operands = value"
            ));
            continue;
        };
        let (formula, operands) = (tokens(formula), tokens(operands));
        if formula.len() != operands.len() {
            found.push(format!(
                "{id}: operands {operands:?} do not follow {formula:?}"
            ));
            continue;
        }
        for (name, operand) in formula.iter().zip(&operands) {
            match (name, operand) {
                (Token::Name(name), Token::Number(number)) => {
                    let value = named(name, &earlier, doc);
                    // The same number, though an input may be written as 1e2
                    // where the working shows 1E+2.
                    let same = value
                        .as_deref()
                        .and_then(|value| Decimal::parse(value).ok())
                        .is_some_and(|value| value.equals(&dec(number)).unwrap());
                    if !same {
                        found.push(format!("{id}: {name} is {value:?}, not {number}"));
                    }
                }
                (same, other) if same == other => {}
                (same, other) => {
                    found.push(format!("{id}: {other:?} where the formula has {same:?}"))
                }
            }
        }
        let recomputed = evaluate(&operands).to_string();
        if recomputed != exact {
            found.push(format!("{id}: the operands give {recomputed}, not {exact}"));
        }
        let cents = dec(exact)
            .quantize(-2, Rounding::HalfUp)
            .unwrap()
            .to_string();
        if step["value"] != json!(cents) {
            found.push(format!(
                "{id}: value {} is not {exact} in cents, {cents}",
                step["value"]
            ));
        }
        if step["value"] != amounts[id] {
            found.push(format!(
                "{id}: value {} differs from the amount {}",
                step["value"], amounts[id]
            ));
        }
        earlier.push((id.to_string(), exact.to_string()));
    }
    found
}

#[test]
fn every_derived_amount_is_recomputed_from_its_working() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut cases = cases();
    for command in DERIVED {
        let mut facts = catalogue[command.replace('-', "_")]["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        cases.push((command, facts));
    }
    for (command, input) in cases {
        let doc = derive(command, &input);
        assert_eq!(doc["format"], "australian-tax-calculators/derivation/1");
        assert_eq!(doc["input"], input);
        assert_eq!(doc["calculation"], command.replace('-', "_"));
        let found = problems(&doc);
        assert!(found.is_empty(), "{command} {input}: {found:#?}");
    }
}

#[test]
fn derive_and_the_plain_command_agree_on_every_amount() {
    for (command, input) in cases() {
        let plain = api::evaluate(command, &input).unwrap();
        let doc = derive(command, &input);
        // Byte for byte: the result inside the derivation is the plain output.
        assert_eq!(
            doc["result"].to_string(),
            plain.to_string(),
            "{command} {input}"
        );
        for step in doc["steps"].as_array().unwrap() {
            assert_eq!(
                step["value"],
                plain["amounts"][step["id"].as_str().unwrap()]
            );
        }
    }
}

/// Every case the Python corpus holds for a derived command: derive returns the
/// plain command's result with a working for each amount, or its refusal.
#[test]
fn every_corpus_case_of_a_derived_command_is_explained_or_refused_alike() {
    let corpus: Vec<Value> =
        serde_json::from_str(include_str!("data/python-api.json")).expect("corpus parses");
    let mut explained = 0;
    for case in &corpus {
        let command = case["command"].as_str().unwrap();
        if !DERIVED.contains(&command) {
            continue;
        }
        let (input, label) = (&case["input"], &case["test"]);
        let derived = api::evaluate_coded("derive", &json!({"command": command, "input": input}));
        match api::evaluate_coded(command, input) {
            Ok(plain) => {
                let doc = derived.unwrap();
                assert_eq!(doc["result"].to_string(), plain.to_string(), "{label}");
                let found = problems(&doc);
                assert!(found.is_empty(), "{label} {input}: {found:#?}");
                explained += 1;
            }
            Err(plain) => assert_eq!(derived.unwrap_err(), plain, "{label}"),
        }
    }
    assert!(explained > 500, "only {explained} corpus cases explained");
}

#[test]
fn a_step_value_changed_without_its_amount_is_caught() {
    let (command, input) = &cases()[0];
    let doc = derive(command, input);
    assert!(problems(&doc).is_empty());
    let mut edited = doc.clone();
    edited["steps"][2]["value"] = json!("21452.74");
    assert_eq!(
        problems(&edited),
        [
            "fbt_estimate: value \"21452.74\" is not 21452.72700000 in cents, 21452.73",
            "fbt_estimate: value \"21452.74\" differs from the amount \"21452.73\"",
        ]
    );
    // An operand that is not the value its name stands for, and an amount
    // changed without its step, are caught the same way.
    let mut edited = doc.clone();
    let working = "return_item_15 x fbt_rate = 45644 x 0.48 = 21452.68";
    edited["steps"][6]["working"] = json!(working);
    let found = problems(&edited);
    assert_eq!(
        found[0],
        "return_item_16: fbt_rate is Some(\"0.47\"), not 0.48"
    );
    assert_eq!(
        found[1],
        "return_item_16: the operands give 21909.12, not 21452.68"
    );
    let mut edited = doc;
    edited["result"]["amounts"]["return_item_15"] = json!("45645.00");
    assert_eq!(
        problems(&edited),
        ["return_item_15: value \"45644.00\" differs from the amount \"45645.00\""]
    );
}

#[test]
fn the_ato_examples_show_their_operands() {
    // Calculating your FBT, MantCo: each gross-up rounded to the dollar for items 14A and 14B.
    let doc = derive("fbt", &cases()[1].1);
    let working = |doc: &Value, i: usize| doc["steps"][i]["working"].as_str().unwrap().to_string();
    assert_eq!(
        working(&doc, 0),
        "type_one_value x type_one = 16500.00 x 2.0802 = 34323.300000"
    );
    assert_eq!(
        doc["steps"][0]["authority"],
        "FBTAA s 5B(1B); Calculating your FBT, step 3"
    );
    assert_eq!(
        working(&doc, 3),
        "dollars(type_one_grossed_up) = dollars(34323.300000) = 34323"
    );
    assert_eq!(
        working(&doc, 6),
        "return_item_15 x fbt_rate = 45644 x 0.47 = 21452.68"
    );
    assert_eq!(doc["steps"][6]["value"], "21452.68");
    // Item 23A example 6: $32,000 for 274 of 365 days.
    let doc = derive("fbt-car-statutory", &cases()[5].1);
    assert_eq!(
        working(&doc, 1),
        "cents(base_value x statutory_percentage x days_available / days_in_year) \
         = cents(32000.00 x 0.20 x 274 / 365) = 4804.38"
    );
    assert_eq!(doc["steps"][1]["authority"], "FBTAA s 9(1)");
    // ¶3-020: s 10(5) takes the lower statutory formula value.
    let doc = derive("fbt-car-operating-cost", &cases()[10].1);
    assert_eq!(
        working(&doc, 2),
        "running_costs + lease_charges + deemed_depreciation + deemed_interest \
         = 5200.00 + 0.00 + 12500.0000 + 4310.000000 = 22010.000000"
    );
    assert_eq!(
        working(&doc, 8),
        "min(operating_cost_taxable_value, statutory_formula_taxable_value) \
         = min(12507.16, 8500.00) = 8500.00"
    );
    assert_eq!(doc["steps"][8]["authority"], "FBTAA s 10(5)");
    // Division 293, Jan: $240,000 and $15,000 give $5,000 taxed at 15%.
    let doc = derive("division-293", &cases()[15].1);
    assert_eq!(
        working(&doc, 1),
        "max(0, combined_income_and_contributions - threshold) \
         = max(0, 255000.00 - 250000) = 5000.00"
    );
    assert_eq!(
        working(&doc, 3),
        "taxable_contributions x rate = 5000.00 x 0.15 = 750.0000"
    );
    // Thavi a year on: $250,000 at 5% for 181 of 365 days, rounded to $6,200.
    let doc = derive("pension-minimum", &cases()[20].1);
    assert!(
        working(&doc, 1).starts_with(
            "account_balance_used x percentage_factor x days / days_in_year \
             = 250000.00 x 0.05 x 181 / 365 = 6198.630136"
        ),
        "{}",
        working(&doc, 1)
    );
    assert_eq!(
        [&doc["steps"][1]["value"], &doc["steps"][2]["value"]],
        ["6198.63", "6200.00"]
    );
    // A full year is cl 1(1)'s formula alone: account balance x percentage factor.
    let doc = derive("pension-minimum", &cases()[19].1);
    assert_eq!(
        working(&doc, 1),
        "account_balance_used x percentage_factor = 600000.00 x 0.05 = 30000.0000"
    );
}

/// Each branch's final amount, as its provision sets it.
#[test]
fn each_branch_gives_the_amount_its_source_sets() {
    let amount = |case: usize, id: &str| {
        let (command, input) = &cases()[case];
        derive(command, input)["result"]["amounts"][id].clone()
    };
    // s 293-20(1): $220,000 does not exceed $250,000; $325,000 exceeds it by
    // $75,000, so the lesser is the $25,000 of contributions, taxed at 15%.
    assert_eq!(amount(16, "division_293_tax"), "0.00");
    assert_eq!(amount(17, "division_293_tax"), "3750.00");
    // s 293-20(2): nil low tax contributions, so no taxable contributions.
    assert_eq!(amount(18, "taxable_contributions"), "0.00");
    // cl 4: started on 1 June, so nothing is required.
    assert_eq!(amount(21, "minimum_payment"), "0.00");
    // cl 5: $4,005 ends in an exact $5, so it rounds up to $4,010.
    assert_eq!(amount(22, "minimum_payment"), "4010.00");
}

#[test]
fn each_step_cites_the_provision_its_branch_applies() {
    let authority = |case: usize, id: &str| {
        let (command, input) = &cases()[case];
        let doc = derive(command, input);
        let steps = doc["steps"].as_array().unwrap();
        let step = steps.iter().find(|s| s["id"] == id).unwrap();
        step["authority"].as_str().unwrap().to_string()
    };
    // s 8A exempts an electric car; a car it does not exempt keeps its s 9 or
    // s 10 value.
    assert_eq!(authority(5, "taxable_value"), "FBTAA s 9(1)");
    assert_eq!(authority(9, "taxable_value"), "FBTAA s 8A");
    assert_eq!(authority(10, "taxable_value"), "FBTAA s 10(2) and (5)");
    assert_eq!(authority(13, "taxable_value"), "FBTAA s 8A");
    // s 9(1) gives the statutory formula value that s 10(5) compares.
    assert_eq!(
        authority(10, "statutory_formula_taxable_value"),
        "FBTAA s 9(1) and s 10(5)"
    );
    // s 10(3)(a)(iii) deems depreciation and interest for an owned car only.
    let owned = "FBTAA s 11(1), (1AA) and (1A)";
    let leased = "FBTAA s 10(3)(a)(iii): owned cars only, so nil for a leased car";
    assert_eq!(authority(10, "deemed_depreciation"), owned);
    // An owned car with a nil depreciated value is still an owned car.
    assert_eq!(authority(11, "deemed_depreciation"), owned);
    assert_eq!(authority(13, "deemed_depreciation"), leased);
    assert_eq!(authority(13, "deemed_interest"), leased);
    // SISR Schedule 7 cl 3 pro-rates only a part year, and cl 4 removes the
    // minimum for a pension started in the year's last 30 days.
    let before_rounding = "minimum_before_rounding";
    assert_eq!(authority(19, before_rounding), "SISR Schedule 7 cl 1(1)");
    assert_eq!(
        authority(20, before_rounding),
        "SISR Schedule 7 cl 1(1) and cl 3"
    );
    assert_eq!(authority(21, before_rounding), "SISR Schedule 7 cl 4");
    assert_eq!(
        authority(15, "division_293_tax"),
        "Superannuation (Sustaining the Superannuation Contribution Concession) Imposition Act 2013 s 5"
    );
}

#[test]
fn the_fbt_benefit_steps_cite_their_provisions() {
    let all = cases();
    let steps = |i: usize| derive(all[i].0, &all[i].1)["steps"].clone();
    // Item 23C example 8: $20,000 owed on each of 365 days at 8.62%.
    let loan = steps(23);
    assert_eq!(
        loan[0]["working"],
        "cents(daily_balance_total x statutory_interest_rate / days_in_year) \
         = cents(7300000.00 x 0.0862 / 365) = 1724.00"
    );
    let authority = loan[0]["authority"].as_str().unwrap();
    assert!(authority.starts_with("FBTAA s 18(1) and s 136(1)"));
    assert_eq!(
        loan[7]["authority"],
        "FBTAA s 19(1), TV - ND; FBT return 2026 instructions, item 23C"
    );
    assert_eq!(steps(26)[0]["authority"], "FBTAA s 37BA");
    assert_eq!(steps(27)[0]["authority"], "FBTAA s 37CB(2)");
    assert_eq!(steps(28)[0]["authority"], "FBTAA s 39DA(3)");
    assert_eq!(steps(29)[0]["authority"], "FBTAA s 39GB");
    // A first business day fee at the threshold gives no car parking benefit.
    let parking = steps(30);
    let taxable = parking.as_array().unwrap().last().unwrap();
    assert_eq!(taxable["working"], "0 = 0 = 0");
    let authority = taxable["authority"].as_str().unwrap();
    assert!(authority.starts_with("FBTAA s 39A(1)(a)(iii)"));
}

#[test]
fn the_payment_and_property_steps_cite_their_provisions() {
    let all = cases();
    let steps = |i: usize| derive(all[i].0, &all[i].1)["steps"].clone();
    assert_eq!(
        steps(31)[0]["authority"],
        "FBTAA s 15; FBT return 2026 instructions, item 23D"
    );
    // Guide 9.4: GD $400 on the $500 spent, less RD $200, takes $250 to $50.
    let external = steps(32);
    assert_eq!(external[0]["authority"], "FBTAA s 23");
    assert_eq!(
        external[4]["working"],
        "cents(recipients_expenditure x income_use) = cents(500.00 x 0.8) = 400.00"
    );
    assert_eq!(
        external[8]["working"],
        "value_after_contribution - reduction_applied = 250.00 - 200.00 = 50.00"
    );
    // Guide 9.3: s 42(1)(b)'s lesser value, less the recipient's portion, then s 62.
    let in_house = steps(33);
    assert_eq!(in_house[0]["authority"], "FBTAA s 42(1)(b) and (2)");
    assert_eq!(
        in_house[1]["working"],
        "recipients_expenditure - amount_paid + employee_contribution \
         = 1000.00 - 250.00 + 0.00 = 750.00"
    );
    assert_eq!(in_house[5]["authority"], "FBTAA s 62(1)");
    assert_eq!(in_house[6]["authority"], "FBTAA s 22A(1) and s 62(1)");
    // Item 23K example 14: 75% of $3,600, rounded to cents.
    let property = steps(34);
    assert_eq!(
        property[0]["working"],
        "cents(lowest_price x 0.75) = cents(3600.00 x 0.75) = 2700.00"
    );
    assert_eq!(property[0]["authority"], "FBTAA s 42(1)(a)(ii)");
    let last = steps(36);
    let last = last.as_array().unwrap();
    assert_eq!(
        last[last.len() - 2]["working"],
        "min(in_house_reduction_available, value_after_contribution - reduction_applied) \
         = min(100.00, 500.00 - 250.00) = 100.00"
    );
    assert_eq!(steps(35)[0]["authority"], "FBTAA s 43(c)");
}

#[test]
fn a_refused_derive_leaves_no_steps_for_the_next() {
    let refused = json!({"command": "fbt-car-statutory", "input": {"days_available": 366}});
    for (command, input) in cases() {
        assert!(api::evaluate("derive", &refused).is_err());
        assert!(
            problems(&derive(command, &input)).is_empty(),
            "{command} {input}"
        );
    }
}

#[test]
fn a_refused_input_gets_the_commands_own_refusal() {
    let refused = |command: &str, input: Value| {
        let plain = api::evaluate_coded(command, &input).unwrap_err();
        let derived = api::evaluate_coded("derive", &json!({"command": command, "input": input}));
        assert_eq!(derived.unwrap_err(), plain);
        plain.code
    };
    let mut input = cases()[0].1.clone();
    input["scope_confirmed"] = json!(false);
    assert_eq!(refused("fbt", input), "scope.not_confirmed");
    let mut input = cases()[5].1.clone();
    input["days_available"] = json!(366);
    assert_eq!(
        refused("fbt-car-statutory", input),
        "input.day_count_invalid"
    );
    let mut input = cases()[10].1.clone();
    input["lease_charges"] = json!("1.00");
    assert_eq!(
        refused("fbt-car-operating-cost", input),
        "fbt_car.cost_basis_conflict"
    );
    assert_eq!(refused("fbt", json!([])), "input.not_object");
    let mut input = cases()[15].1.clone();
    input["low_tax_contributions"] = json!("0.001");
    assert_eq!(refused("division-293", input), "input.amount_invalid");
    let mut input = cases()[19].1.clone();
    input["days"] = json!(0);
    assert_eq!(refused("pension-minimum", input), "input.day_count_invalid");
}

#[test]
fn derive_refuses_what_it_does_not_explain() {
    let code = |envelope: Value| api::evaluate_coded("derive", &envelope).unwrap_err().code;
    let gst = json!({"amount": "1100", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"});
    assert_eq!(
        code(json!({"command": "gst", "input": gst})),
        "derivation.unsupported"
    );
    assert_eq!(
        code(json!({"command": "derive", "input": {}})),
        "derivation.unsupported"
    );
    assert_eq!(
        code(json!({"command": 5, "input": {}})),
        "derivation.unsupported"
    );
    assert_eq!(code(json!({"input": {}})), "input.missing_field");
    for command in DERIVED {
        assert_eq!(
            code(json!({"command": command})),
            "input.missing_field",
            "{command}"
        );
    }
    // A key derive does not take is refused, not ignored.
    let (command, input) = &cases()[0];
    let extra = json!({"command": command, "input": input, "steps": false});
    assert_eq!(code(extra), "input.field_unknown");
    let message = api::evaluate("derive", &json!({"command": "gst", "input": {}})).unwrap_err();
    assert_eq!(
        message,
        "derive explains fbt, fbt-car-statutory, fbt-car-operating-cost, fbt-loan, \
         fbt-meal-entertainment, fbt-car-parking, fbt-debt-waiver, fbt-expense-payment, \
         fbt-property, division-293, pension-minimum, not \"gst\""
    );
}

#[test]
fn the_binary_derives_and_refuses_with_a_code() {
    let run = |args: &[&str], stdin: &str| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_australian-tax-calculators"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        (
            out.status.code().unwrap(),
            String::from_utf8(out.stdout).unwrap(),
        )
    };
    let (command, input) = &cases()[5];
    let envelope = json!({"command": command, "input": input}).to_string();
    let (code, out) = run(&["derive"], &envelope);
    assert_eq!(code, 0);
    let doc: Value = serde_json::from_str(&out).unwrap();
    assert!(problems(&doc).is_empty());
    let (code, out) = run(
        &["derive", "--refusal-json"],
        r#"{"command": "gst", "input": {}}"#,
    );
    assert_eq!(code, 2);
    let refusal: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(refusal["code"], "derivation.unsupported");
    let (_, help) = run(&["--api-help"], "");
    assert!(
        help.contains("worksheet-catalogue, periods, derive"),
        "{help}"
    );
}
