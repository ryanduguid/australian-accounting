// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `fbt_minor_benefit_check`: a screen of one benefit against the minor
//! benefits exemption, FBTAA s 58P. The exclusions of s 58P(1)(c) and (d) and
//! the less than $300 test of s 58P(1)(e) are decided; the criteria of
//! s 58P(1)(f) are stated facts, weighed as the Commissioner's examples in
//! TR 2007/12 weigh them. The citations are to s 58P as the ATO's legal
//! database held it (s 58P(1)(b), the airline transport exclusion, repealed by
//! Act No. 88 of 2013) and TR 2007/12, both read on 19 November 2026.

use serde_json::{Value, json};

use super::{Arg, Result, d, fbt_year, flag, money, rate, refuse, result};
use crate::decimal::Decimal;

const KIND: &str = "fbt_minor_benefit_check";

/// The facts as the command passes them; see the README for each.
pub struct MinorBenefitArgs<'a> {
    pub category: &'a Arg,
    pub notional_taxable_value: &'a Arg,
    pub salary_sacrifice: &'a Arg,
    pub frequency: &'a Arg,
    pub similar_benefits_substantial: &'a Arg,
    pub other_associated_benefits_substantial: &'a Arg,
    pub valuation_difficult: &'a Arg,
    pub unexpected_event: &'a Arg,
    pub reward_for_services: &'a Arg,
    pub in_house: Option<&'a Arg>,
    pub tax_exempt_body_exception: Option<&'a Arg>,
    pub fifty_fifty_split: Option<&'a Arg>,
    pub year_ended: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const CATEGORIES: [&str; 7] = [
    "expense_payment",
    "property",
    "residual",
    "tax_exempt_body_entertainment",
    "meal_entertainment",
    "entertainment_facility_leasing",
    "other",
];

/// One of `allowed`, or a refusal naming them.
fn choice<'a>(arg: &'a Arg, key: &str, allowed: &[&str]) -> Result<&'a str> {
    match arg.text() {
        Some(value) if allowed.contains(&value) => Ok(value),
        _ => refuse(
            "fbt_minor_benefit.fact_invalid",
            format!("{key} must be one of {}.", allowed.join(", ")),
        ),
    }
}

/// An input that `needed` decides: required when needed, refused when not.
fn when<'a>(needed: bool, given: Option<&'a Arg>, key: &str, why: &str) -> Result<Option<&'a Arg>> {
    match (needed, given) {
        (true, None) => refuse("input.missing_field", format!("{key} is required {why}.")),
        (false, Some(_)) => refuse(
            "input.field_unknown",
            format!("{key} is an input only {why}"),
        ),
        (_, given) => Ok(given),
    }
}

const IN_HOUSE: &str = "FBTAA s 58P(1)(c): an expense payment, property or residual benefit qualifies only if, were it a fringe benefit of that kind, it would not be an in-house fringe benefit";
const TAX_EXEMPT_BODY: &str = "FBTAA s 58P(1)(d): where the provider incurs non-deductible exempt entertainment expenditure for the entertainment, it qualifies only if the entertainment is incidental to entertaining outsiders and is not, and is not provided with, a meal other than light refreshments ((d)(i)), or is provided on the employer's premises solely to recognise the employee's special achievements in the employment ((d)(ii))";
const FIFTY_FIFTY: &str = "TR 2007/12 para 14, the Commissioner's view: where the employer elects the 50/50 split method of Division 9A for meal entertainment or entertainment facility leasing expenses, the minor benefits exemption cannot reduce the taxable value";
const SALARY_SACRIFICE: &str = "TR 2007/12 para 16 and example 13, the Commissioner's view: the exemption does not apply to a benefit provided under a salary sacrifice arrangement";
const LESS_THAN_300: &str = "FBTAA s 58P(1)(e): the notional taxable value of the benefit in the current year of tax is less than $300, tested for each benefit to each employee or associate (TR 2007/12 paras 17 and 18)";

const FREQUENCY: &str = "FBTAA s 58P(1)(f)(i): the infrequency and irregularity with which identical or similar benefits, or benefits provided with it, have been or can reasonably be expected to be provided";
const SIMILAR_TOTAL: &str = "FBTAA s 58P(1)(f)(ii): the sum of the notional taxable values of the benefit and identical or similar associated benefits, in this and any other year of tax";
const OTHER_TOTAL: &str = "FBTAA s 58P(1)(f)(iii): the sum of the notional taxable values of any other associated benefits, in this and any other year of tax";
const DIFFICULTY: &str = "FBTAA s 58P(1)(f)(iv): the practical difficulty for the employer in working out the notional taxable values of the benefit and any associated benefits that are not car benefits";
const UNEXPECTED: &str = "FBTAA s 58P(1)(f)(v)(A): whether the benefit was provided to help the employee deal with an unexpected event";
const REWARD: &str = "FBTAA s 58P(1)(f)(v)(B): whether the benefit was provided otherwise than wholly or principally as a reward for services rendered or to be rendered";

/// One condition row.
fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

/// One s 58P(1)(f) criterion: the fact stated and which way it weighs.
fn criterion(id: &str, authority: &str, stated: Value, weighs: &str) -> Value {
    json!({"criterion": id, "authority": authority, "stated": stated, "weighs": weighs})
}

/// The condition `category`'s own exclusion sets, if any: s 58P(1)(c) or
/// (d), or the ruling's 50/50 split.
fn category_rows(args: &MinorBenefitArgs, category: &str) -> Result<Vec<Value>> {
    let sold = matches!(category, "expense_payment" | "property" | "residual");
    let why = "for an expense_payment, property or residual benefit (s 58P(1)(c))";
    let in_house = when(sold, args.in_house, "in_house", why)?;
    let why = "for tax_exempt_body_entertainment (s 58P(1)(d))";
    let tebe = category == "tax_exempt_body_entertainment";
    let exception = when(
        tebe,
        args.tax_exempt_body_exception,
        "tax_exempt_body_exception",
        why,
    )?;
    let split = matches!(
        category,
        "meal_entertainment" | "entertainment_facility_leasing"
    );
    let why = "for meal_entertainment or entertainment_facility_leasing (TR 2007/12 para 14)";
    let fifty = when(split, args.fifty_fifty_split, "fifty_fifty_split", why)?;
    let mut rows = Vec::new();
    if let Some(in_house) = in_house {
        let met = !flag(in_house, "in_house")?;
        rows.push(condition("not_in_house", IN_HOUSE, met));
    }
    if let Some(exception) = exception {
        let allowed = [
            "incidental_to_outsiders_no_meal",
            "special_achievement_on_premises",
            "none",
        ];
        let met = choice(exception, "tax_exempt_body_exception", &allowed)? != "none";
        rows.push(condition("tax_exempt_body_exception", TAX_EXEMPT_BODY, met));
    }
    if let Some(fifty) = fifty {
        let met = !flag(fifty, "fifty_fifty_split")?;
        rows.push(condition("not_fifty_fifty_split", FIFTY_FIFTY, met));
    }
    Ok(rows)
}

/// The conditions s 58P(1)(c) to (e) and the ruling set, for `category`.
fn conditions(args: &MinorBenefitArgs, category: &str, value: &Decimal) -> Result<Vec<Value>> {
    let mut rows = category_rows(args, category)?;
    let sacrificed = flag(args.salary_sacrifice, "salary_sacrifice")?;
    rows.push(condition(
        "not_salary_sacrifice",
        SALARY_SACRIFICE,
        !sacrificed,
    ));
    rows.push(condition(
        "less_than_300",
        LESS_THAN_300,
        value.lt(&d("300"))?,
    ));
    Ok(rows)
}

/// The s 58P(1)(f) facts the screen weighs, and every criterion's row.
struct Criteria {
    frequent: bool,
    /// Infrequent but regular, and a reward for services: no example has
    /// both.
    regular_reward: bool,
    substantial: [bool; 2],
    rows: Vec<Value>,
}

/// Each yes-or-no criterion: its input, its authority and which way a true
/// and a false statement weigh.
const FLAGS: [(&str, &str, &str, &str); 5] = [
    (
        "similar_benefits_substantial",
        SIMILAR_TOTAL,
        "against",
        "for",
    ),
    (
        "other_associated_benefits_substantial",
        OTHER_TOTAL,
        "against",
        "for",
    ),
    ("valuation_difficult", DIFFICULTY, "for", "neutral"),
    ("unexpected_event", UNEXPECTED, "for", "neutral"),
    ("reward_for_services", REWARD, "against", "for"),
];

fn criteria(args: &MinorBenefitArgs) -> Result<Criteria> {
    let allowed = [
        "infrequent_and_irregular",
        "infrequent_and_regular",
        "frequent",
    ];
    let frequency = choice(args.frequency, "frequency", &allowed)?;
    let weighs = match frequency {
        "infrequent_and_irregular" => "for",
        // TR 2007/12 examples 1 to 3: infrequent but regular, still exempt.
        "infrequent_and_regular" => "neutral",
        _ => "against",
    };
    let mut rows = vec![criterion("frequency", FREQUENCY, json!(frequency), weighs)];
    let given = [
        args.similar_benefits_substantial,
        args.other_associated_benefits_substantial,
        args.valuation_difficult,
        args.unexpected_event,
        args.reward_for_services,
    ];
    let mut stated = [false; 5];
    for (i, ((key, authority, yes, no), arg)) in FLAGS.iter().zip(given).enumerate() {
        stated[i] = flag(arg, key)?;
        let weighs = if stated[i] { yes } else { no };
        rows.push(criterion(key, authority, json!(stated[i]), weighs));
    }
    Ok(Criteria {
        frequent: frequency == "frequent",
        regular_reward: frequency == "infrequent_and_regular" && stated[4],
        substantial: [stated[0], stated[1]],
        rows,
    })
}

const EXEMPT: &str = "exempt: the stated criteria match the TR 2007/12 examples that conclude it would be unreasonable to treat the benefit as a fringe benefit (examples 1 to 3, 5 to 7 and 9 to 11: identical or similar benefits not provided frequently, neither total substantial, and not both regular and a reward for services), so it is an exempt benefit for the current year of tax (s 58P(1)), on the Commissioner's view";
const NOT_EXEMPT_FAILED: &str = "not_exempt: a condition of s 58P(1)(c) to (e), or one the Commissioner's ruling sets, is not met";
const NOT_EXEMPT_FREQUENT: &str = "not_exempt: the stated criteria match TR 2007/12 example 8 (vouchers provided frequently, whose total is substantial), which concludes it would not be unreasonable to treat the benefit as a fringe benefit, on the Commissioner's view";
const JUDGEMENT: &str = "judgement_required: the stated criteria match neither the TR 2007/12 examples that reach the exemption nor example 8, which does not; whether it would be unreasonable to treat the benefit as a fringe benefit, weighing all of s 58P(1)(f), is the employer's to decide";

/// The screen's verdict and its reason.
fn screen(failing: bool, c: &Criteria) -> (Value, &'static str) {
    if failing {
        return (json!(false), NOT_EXEMPT_FAILED);
    }
    if !c.frequent && c.substantial == [false, false] && !c.regular_reward {
        return (json!(true), EXEMPT);
    }
    if c.frequent && c.substantial[0] {
        return (json!(false), NOT_EXEMPT_FREQUENT);
    }
    (Value::Null, JUDGEMENT)
}

const IF_EXEMPT: &str = "An exempt benefit is not a fringe benefit (FBTAA s 136(1), 'fringe benefit' para (g)): it adds nothing to the taxable value in fbt or fbt-return, and nothing to the employee's reportable fringe benefits amount.";
const IF_NOT: &str = "A benefit that is not exempt keeps its taxable value: give it to its benefit worksheet (fbt-expense-payment, fbt-property, fbt-residual, fbt-meal-entertainment, fbt-tax-exempt-body-entertainment or the others) as usual.";

pub fn fbt_minor_benefit_check(args: &MinorBenefitArgs) -> Result<Value> {
    let (_, period, _) = fbt_year(args.scope_confirmed, args.year_ended, KIND)?;
    let category = choice(args.category, "category", &CATEGORIES)?;
    let value = money(args.notional_taxable_value)?;
    let rows = conditions(args, category, &value)?;
    let criteria = criteria(args)?;
    let failing: Vec<Value> = rows
        .iter()
        .filter(|row| row["met"] == json!(false))
        .map(|row| row["condition"].clone())
        .collect();
    let (exempt, reason) = screen(!failing.is_empty(), &criteria);
    let rates = vec![rate("minor_benefit_threshold", "300")];
    let mut out = result(
        KIND,
        &period,
        vec![("notional_taxable_value", value)],
        rates,
    )?;
    out["exempt"] = exempt.clone();
    out["screen"] = json!(reason.split(':').next().expect("a verdict"));
    out["reason"] = json!(reason);
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["criteria"] = Value::Array(criteria.rows);
    out["consequences"] = match exempt {
        Value::Bool(true) => json!([IF_EXEMPT]),
        Value::Bool(false) => json!([IF_NOT]),
        _ => json!([IF_EXEMPT, IF_NOT]),
    };
    Ok(out)
}
