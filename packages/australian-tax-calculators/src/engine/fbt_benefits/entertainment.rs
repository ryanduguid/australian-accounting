// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The entertainment classifier: for each expense line, whether it is
//! entertainment, the fringe benefit or exemption for each class of attendee,
//! the income tax deduction and the GST credit that follows it. Two tables
//! decide: [`CHARACTER`] from what was provided and on what occasion, then
//! [`TREATMENT`], whose first matching row gives each attendee class's
//! outcome. Read on 16 November 2026, 5.58 pm to 6.11 pm AEDT: ITAA 1997
//! compilation 268 (1 October 2026), Division 32, s 27-5 and s 995-1; the
//! FBTAA compilation 97 (1 July 2026), ss 20, 37A to 37CF, 40, 41, 45 and
//! 58P and s 136(1); ITAA 1936 compilation 193 (1 October 2026), ss 51AEA
//! and 51AEB; the GST Act compilation 96 (1 January 2026), s 69-5. TR 97/17
//! and TR 2007/12, read on the ATO's legal database that day, are the
//! Commissioner's view, cited as such. Whether a fringe benefit is excluded
//! from reporting: FBTAA ss 5E and 135P(1) (compilation 97) and the Fringe
//! Benefits Tax Assessment Regulations 2018 (F2018L01284, as made, the current
//! version), regulations 6 to 11, read on 16 November 2026, 7.50 pm to 7.55 pm
//! AEDT.

use serde_json::{Map, Value, json};

use super::super::{Arg, Result, cents, d, fbt_year, flag, int, money, precise, rate, refuse};
use super::method;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "fbt_entertainment_classifier";
const COMMAND: &str = "fbt-entertainment-classifier";

/// Whether a line is entertainment, and of which kind.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Character {
    /// Meal entertainment (FBTAA s 37AD), which Division 9A can value.
    Meal,
    /// Entertainment by way of recreation, or travel for it: never meal
    /// entertainment, so Division 9A does not reach it.
    Recreation,
    NotEntertainment,
}
use Character::{Meal, NotEntertainment, Recreation};

/// TR 97/17's tests of why, what, when and where food or drink is provided
/// (para 23), applied to the occasion the operator states: (provided,
/// occasion, character, authority). Food or drink with no occasion stated is
/// left undetermined.
const CHARACTER: [(&str, Option<&str>, Character, &str); 8] = [
    (
        "food_or_drink",
        Some("social_function"),
        Meal,
        "ITAA 1997 s 32-10(1)(a) and FBTAA s 37AD(a): food or drink at a party, reception or other social function is entertainment whatever is served (TR 97/17 paras 20, 50 and 62, the Commissioner's view)",
    ),
    (
        "food_or_drink",
        Some("business_meal"),
        Meal,
        "ITAA 1997 s 32-10(1)(a) and (2) and FBTAA s 37AD(a) and (d): a business lunch, dinner or drinks, or another meal beyond light refreshments for attendees not travelling for work, is entertainment even if business is discussed (TR 97/17 paras 20, 23(b), 58, 93 and 138.1, the Commissioner's view)",
    ),
    (
        "food_or_drink",
        Some("with_other_entertainment"),
        Meal,
        "ITAA 1997 s 32-10(1)(a) and FBTAA s 37AD(a): food or drink with other entertainment, such as a floor show, is entertainment (TR 97/17 paras 71, 77 and 119, the Commissioner's view)",
    ),
    (
        "food_or_drink",
        Some("light_refreshments"),
        NotEntertainment,
        "ITAA 1997 s 32-10(1): morning or afternoon tea, light meals or finger food, not at a social function, confer no entertainment (TR 97/17 paras 18, 19, 23(b), 56, 69 and 114, the Commissioner's view)",
    ),
    (
        "food_or_drink",
        Some("travel_meal"),
        NotEntertainment,
        "ITAA 1997 s 32-10(1) note: a meal of attendees all travelling for work or business, with no other entertainment, is not entertainment (TR 97/17 paras 21, 67, 71, 76, 82 and 86, the Commissioner's view)",
    ),
    (
        "recreation",
        None,
        Recreation,
        "ITAA 1997 s 32-10(1)(a): entertainment by way of recreation, which includes amusement, sport or similar leisure-time pursuits (s 995-1); not meal entertainment, which FBTAA s 37AD limits to food or drink (TR 97/17 para 8(b))",
    ),
    (
        "accommodation_or_travel_for_food_or_drink",
        None,
        Meal,
        "ITAA 1997 s 32-10(1)(b) and FBTAA s 37AD(b): accommodation or travel to do with entertainment by way of food or drink",
    ),
    (
        "accommodation_or_travel_for_recreation",
        None,
        Recreation,
        "ITAA 1997 s 32-10(1)(b): accommodation or travel to do with entertainment by way of recreation; not meal entertainment (FBTAA s 37AD(b))",
    ),
];

/// The fact an undetermined food or drink line lacks.
const OCCASION_MISSING: &str = "occasion: whether food or drink is entertainment turns on why, what, when and where it is provided (TR 97/17 paras 7 and 23); state social_function, business_meal, with_other_entertainment, light_refreshments or travel_meal, or split the line until one fits";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Employees,
    Associates,
    Others,
}
use Class::{Associates, Employees, Others};

const CLASSES: [(&str, Class); 3] = [
    ("employees", Employees),
    ("associates", Associates),
    ("others", Others),
];

/// The employer's method of valuing meal entertainment for the FBT year.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Valuation {
    Actual,
    FiftyFifty,
    Register,
}
use Valuation::{Actual, FiftyFifty, Register};

/// Whether FBTAA s 41(1) can reach an employee's share: the employer provided
/// property (food or drink it paid for), consumed on its business premises
/// on a working day; it provided something else there, which may or may not
/// be property; or neither.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Premises {
    Property,
    Unsettled,
    No,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fbt {
    Arises,
    Exempt,
    NotAFringeBenefit,
    InMethodTotal,
    Undetermined,
}

/// The share of a class's cost that is deductible, and so creditable.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Share {
    Whole,
    Nil,
    Half,
    RegisterPercentage,
    Undetermined,
}

/// One row: the line's character, the employer's valuation, the attendee
/// class and the s 41 position it matches (empty matches any), and the
/// outcome it gives.
struct Treatment {
    characters: &'static [Character],
    valuations: &'static [Valuation],
    classes: &'static [Class],
    premises: &'static [Premises],
    fbt: Fbt,
    share: Share,
    authority: &'static str,
}

const ENTERTAINMENT: &[Character] = &[Meal, Recreation];
const RECIPIENTS: &[Class] = &[Employees, Associates];

/// The first row that matches decides; the order is the order of the
/// questions in TR 97/17 Part B: an election under Division 9A first (para
/// 42), then who received it, then s 41.
const TREATMENT: [Treatment; 9] = [
    Treatment {
        characters: &[Meal],
        valuations: &[FiftyFifty],
        classes: &[],
        premises: &[],
        fbt: Fbt::InMethodTotal,
        share: Share::Half,
        authority: "FBTAA ss 37AF and 37BA: under the 50/50 split method the line's whole cost, whoever attended, counts in the year's meal entertainment expenditure and no other fringe benefit arises, so neither s 41 nor s 58P exempts any of it (TR 97/17 paras 31 and 44; TR 2007/12 para 14, the Commissioner's view). ITAA 1936 s 51AEA(1): a deduction of 50% of the expense and no other; GST Act s 69-5(3A)(a): no GST credit for the other 50%.",
    },
    Treatment {
        characters: &[Meal],
        valuations: &[Register],
        classes: &[],
        premises: &[],
        fbt: Fbt::InMethodTotal,
        share: Share::RegisterPercentage,
        authority: "FBTAA s 37CB: under the 12 week register method the line's whole cost, whoever attended, counts in the year's meal entertainment expenditure, which fbt-meal-entertainment values at the register percentage; s 41 does not apply, and minor benefits count only in the register (TR 97/17 paras 32 and 44; TR 2007/12 para 15, the Commissioner's view). ITAA 1936 s 51AEB(1): a deduction of the expense times income_tax_register_percentage and no other; GST Act s 69-5(3A)(b): no GST credit for the rest.",
    },
    Treatment {
        characters: &[NotEntertainment],
        valuations: &[],
        classes: &[Others],
        premises: &[],
        fbt: Fbt::NotAFringeBenefit,
        share: Share::Whole,
        authority: "No fringe benefit: a fringe benefit is provided to an employee or an associate (FBTAA s 136(1)). Not entertainment, so Division 32 denies no deduction and GST Act s 69-5 no GST credit (ITAA 1997 s 8-1; TR 97/17 paras 41, 48, 84 and 88, the Commissioner's view).",
    },
    Treatment {
        characters: ENTERTAINMENT,
        valuations: &[],
        classes: &[Others],
        premises: &[],
        fbt: Fbt::NotAFringeBenefit,
        share: Share::Nil,
        authority: "No fringe benefit: a fringe benefit is provided to an employee or an associate (FBTAA s 136(1)), so ITAA 1997 s 32-20 does not lift s 32-5 and none of it is deductible (TR 97/17 paras 52, 96 and 127, the Commissioner's view); no GST credit (GST Act s 69-5(1) and (3)(f)).",
    },
    Treatment {
        characters: &[NotEntertainment],
        valuations: &[],
        classes: &[Employees],
        premises: &[Premises::Property],
        fbt: Fbt::Exempt,
        share: Share::Whole,
        authority: "FBTAA s 41(1): property provided to and consumed by a current employee on the employer's business premises on a working day is an exempt benefit. Not entertainment, so deductible with its GST credit (ITAA 1997 s 8-1; TR 97/17 paras 38, 41 and 56, the Commissioner's view).",
    },
    Treatment {
        characters: ENTERTAINMENT,
        valuations: &[],
        classes: &[Employees],
        premises: &[Premises::Property],
        fbt: Fbt::Exempt,
        share: Share::Nil,
        authority: "FBTAA s 41(1): property provided to and consumed by a current employee on the employer's business premises on a working day is an exempt benefit (TR 97/17 paras 27, 44 and 50, the Commissioner's view). An exempt benefit is not a fringe benefit (FBTAA s 136(1)), so ITAA 1997 s 32-20 does not lift s 32-5 and none of it is deductible (TR 97/17 paras 36, 46 and 50); no GST credit (GST Act s 69-5(1) and (3)(f)).",
    },
    Treatment {
        characters: ENTERTAINMENT,
        valuations: &[],
        classes: &[Employees],
        premises: &[Premises::Unsettled],
        fbt: Fbt::Undetermined,
        share: Share::Undetermined,
        authority: "FBTAA s 41(1) exempts only property consumed on the business premises on a working day; if it applies, ITAA 1997 s 32-5 denies the deduction, and if not, s 32-20 allows it.",
    },
    Treatment {
        characters: ENTERTAINMENT,
        valuations: &[],
        classes: RECIPIENTS,
        premises: &[],
        fbt: Fbt::Arises,
        share: Share::Whole,
        authority: "A fringe benefit (FBTAA s 136(1)): s 41 reaches only an employee's property consumed on business premises on a working day, and never an associate's (TR 97/17 paras 45, 51 and 62, the Commissioner's view). Deductible because it is a fringe benefit (ITAA 1997 s 32-20; TR 97/17 para 26), with its GST credit.",
    },
    Treatment {
        characters: &[NotEntertainment],
        valuations: &[],
        classes: RECIPIENTS,
        premises: &[],
        fbt: Fbt::Arises,
        share: Share::Whole,
        authority: "A fringe benefit (FBTAA s 136(1)), whose taxable value the otherwise deductible rule can reduce for an employee (FBTAA ss 24 and 44; TR 97/17 paras 39 and 71, the Commissioner's view). Not entertainment, so deductible with its GST credit (ITAA 1997 s 8-1; TR 97/17 para 41).",
    },
];

/// The fact an undetermined s 41 row lacks.
const PROPERTY_MISSING: &str = "whether what the employer provided is property its employees consumed on the business premises (FBTAA ss 40 and 41(1)): if so it is exempt and not deductible, and if not it is a fringe benefit and deductible";

/// Whether a fringe benefit is left out of reporting, as `fbt-return` takes it.
const MEAL_EXCLUDED: &str = "An excluded fringe benefit, left out of the employee's individual fringe benefits amount and so not reportable: meal entertainment (FBTAA s 37AD) not provided under a salary packaging arrangement, whether or not Division 9A is elected (FBTAA s 5E(2) and (3)(a); TR 97/17 Part A table, column R, the Commissioner's view).";
const NOT_EXCLUDED: &str = "Not an excluded fringe benefit: it is not meal entertainment (FBTAA s 5E(3)(a)), and within this classifier's scope no other paragraph of s 5E(3), nor regulations 6 to 11 of the Fringe Benefits Tax Assessment Regulations 2018, covers it. Its taxable value, after any reduction, counts in the employee's individual fringe benefits amount (s 5E(2)), which is reportable when more than $2,000 (s 135P(1)); TR 97/17 Part A table, column R, the Commissioner's view.";

fn treatment(
    character: Character,
    valuation: Valuation,
    class: Class,
    premises: Premises,
) -> &'static Treatment {
    fn has<T: PartialEq>(set: &[T], value: T) -> bool {
        set.is_empty() || set.contains(&value)
    }
    TREATMENT
        .iter()
        .find(|row| {
            has(row.characters, character)
                && has(row.valuations, valuation)
                && has(row.classes, class)
                && has(row.premises, premises)
        })
        .expect("a row for every case")
}

/// The fringe benefit's type: (type, authority).
fn benefit_type(provided: &str, reimbursed: bool) -> (&'static str, &'static str) {
    match (reimbursed, provided) {
        (true, _) => (
            "expense_payment",
            "FBTAA s 20: the employer reimbursed or paid for the attendee who incurred it; s 63A moves the part for others entertained to them (TR 97/17 paras 73 and 98, the Commissioner's view)",
        ),
        (false, "food_or_drink") => (
            "property",
            "FBTAA s 40: food or drink is tangible property (s 136(1)); paying by corporate card does not change that (TR 97/17 paras 62 and 129, the Commissioner's view)",
        ),
        _ => (
            "property_or_residual",
            "FBTAA s 40 if what was provided is property (s 136(1)), otherwise a residual benefit (s 45)",
        ),
    }
}

/// The minor benefits exemption, as a possibility only: s 58P(1)(e)'s
/// threshold from the Act and (f)(i) as the operator states it; the rest of
/// (f) is the employer's judgement.
fn minor_benefit(
    per_head: Option<&Decimal>,
    infrequent: Option<bool>,
    character: Character,
) -> Result<Value> {
    let if_exempt = if character == NotEntertainment {
        "If the employer concludes it is exempt, no fringe benefits tax arises on it and the deduction and GST credit stand (ITAA 1997 s 8-1)."
    } else {
        "If the employer concludes it is exempt, it is not a fringe benefit, so none of it is deductible and no GST credit arises (ITAA 1997 ss 32-5 and 32-20; FBTAA s 136(1); TR 97/17 para 36, the Commissioner's view)."
    };
    let shown = per_head.map(render).transpose()?;
    let stated = match infrequent {
        None => {
            "Whether identical or similar benefits are provided infrequently and irregularly (s 58P(1)(f)(i)) is not stated."
        }
        Some(true) => {
            "Identical or similar benefits are stated to be provided infrequently and irregularly (s 58P(1)(f)(i))."
        }
        Some(false) => {
            "Identical or similar benefits are stated not to be provided infrequently and irregularly (s 58P(1)(f)(i)); the Commissioner weighs that with the other criteria, and in TR 2007/12 example 2 an annual party still qualifies (paras 37 to 41)."
        }
    };
    let (status, reason) = match per_head {
        None => (
            "not_assessed",
            "The head count is not stated, so the notional taxable value of each person's benefit is unknown (FBTAA s 58P(1)(e); TR 2007/12 para 17).".to_string(),
        ),
        Some(value) if !value.lt(&int(300))? => (
            "unavailable",
            format!(
                "The notional taxable value of each person's benefit, {}, is not less than $300 (FBTAA s 58P(1)(e)).",
                shown.as_deref().unwrap_or_default()
            ),
        ),
        Some(_) => (
            "possible",
            format!(
                "Each person's benefit is less than $300 (FBTAA s 58P(1)(e)). {stated} Whether it would be unreasonable to treat it as a fringe benefit, having regard to all of s 58P(1)(f), is the employer's judgement, not this classifier's (TR 2007/12 paras 8, 9 and 20 to 22, the Commissioner's view). {if_exempt}"
            ),
        ),
    };
    Ok(
        json!({"status": status, "per_head": shown, "infrequent_and_irregular": infrequent, "reason": reason}),
    )
}

/// `value` half up to cents, as text.
fn render(value: &Decimal) -> Result<String> {
    Ok(precise(|| Ok(value.quantize(-2, Rounding::HalfUp)?))?.to_string())
}

/// A line's stated facts, checked.
struct Line {
    cost: Decimal,
    includes_gst: bool,
    provided: &'static str,
    character: Option<(Character, &'static str)>,
    premises: Premises,
    reimbursed: bool,
    /// Each class that attended and its head count, where stated.
    attendees: Vec<(Class, Option<i64>)>,
    infrequent: Option<bool>,
}

const LINE_FIELDS: [&str; 8] = [
    "cost",
    "gst_status",
    "provided",
    "occasion",
    "venue",
    "paid_by",
    "attendees",
    "infrequent_and_irregular",
];

/// The `provided` values, as [`CHARACTER`] names them.
const PROVIDED: [&str; 4] = [
    "food_or_drink",
    "recreation",
    "accommodation_or_travel_for_food_or_drink",
    "accommodation_or_travel_for_recreation",
];

/// The `occasion` values of a food or drink line, as [`CHARACTER`] names them.
const OCCASIONS: [&str; 5] = [
    "social_function",
    "business_meal",
    "with_other_entertainment",
    "light_refreshments",
    "travel_meal",
];

/// The value of `name`, one of `allowed`.
fn choice(
    fields: &Map<String, Value>,
    n: usize,
    name: &str,
    allowed: &[&'static str],
) -> Result<&'static str> {
    let Some(value) = fields.get(name) else {
        return refuse(
            "input.missing_field",
            format!("line {n}: {name} is required"),
        );
    };
    match allowed.iter().find(|a| value.as_str() == Some(**a)) {
        Some(found) => Ok(found),
        None => {
            let (last, rest) = allowed.split_last().expect("choices");
            refuse(
                "fbt_entertainment_classifier.fact_invalid",
                format!("line {n}: {name} must be {} or {last}.", rest.join(", ")),
            )
        }
    }
}

fn parse_line(value: &Value, n: usize) -> Result<Line> {
    let Some(fields) = value.as_object() else {
        return refuse(
            "fbt_entertainment_classifier.lines_invalid",
            format!("line {n} is not a JSON object."),
        );
    };
    if let Some(key) = fields.keys().find(|k| !LINE_FIELDS.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("line {n}: {key} is not an input of a line"),
        );
    }
    let cost = match fields.get("cost").and_then(Value::as_str) {
        Some(text) if text.len() <= 32 => match Decimal::parse(text) {
            Ok(amount) => money(&Arg::Decimal(amount)).ok(),
            Err(_) => None,
        },
        _ => None,
    };
    let Some(cost) = cost else {
        return refuse(
            "input.amount_invalid",
            format!(
                "line {n}: cost must be a decimal string, finite and non-negative, at most 2dp and 1e12: the employer's expense, less any contribution from an employee or associate it does not reimburse (FBTAA s 37AB)."
            ),
        );
    };
    let includes_gst = match fields.get("gst_status").and_then(Value::as_str) {
        Some("includes_gst") => true,
        Some("no_gst") => false,
        _ => {
            return refuse(
                "fbt_entertainment_classifier.gst_status_invalid",
                format!(
                    "line {n}: gst_status must be includes_gst or no_gst: whether the cost includes GST sets its GST credit (GST Act s 69-5)."
                ),
            );
        }
    };
    let provided = choice(fields, n, "provided", &PROVIDED)?;
    let character = match (provided, fields.get("occasion")) {
        ("food_or_drink", None) => None,
        ("food_or_drink", Some(_)) => {
            let occasion = choice(fields, n, "occasion", &OCCASIONS)?;
            CHARACTER
                .iter()
                .find(|row| row.1 == Some(occasion))
                .map(|row| (row.2, row.3))
        }
        (_, Some(_)) => {
            return refuse(
                "input.field_unknown",
                format!("line {n}: occasion is an input of a food_or_drink line only"),
            );
        }
        _ => CHARACTER
            .iter()
            .find(|row| row.0 == provided)
            .map(|row| (row.2, row.3)),
    };
    let venue = [
        "business_premises_working_day",
        "business_premises_other_day",
        "elsewhere",
    ];
    let on_premises = choice(fields, n, "venue", &venue)? == venue[0];
    let reimbursed = choice(fields, n, "paid_by", &["employer", "reimbursed"])? == "reimbursed";
    let premises = match (on_premises && !reimbursed, provided) {
        (false, _) => Premises::No,
        (true, "food_or_drink") => Premises::Property,
        (true, _) => Premises::Unsettled,
    };
    let infrequent = match fields.get("infrequent_and_irregular") {
        None => None,
        Some(Value::Bool(stated)) => Some(*stated),
        Some(_) => {
            return refuse(
                "input.flag_invalid",
                format!("line {n}: infrequent_and_irregular must be a boolean."),
            );
        }
    };
    Ok(Line {
        cost,
        includes_gst,
        provided,
        character,
        premises,
        reimbursed,
        attendees: attendees(fields.get("attendees"), n)?,
        infrequent,
    })
}

/// The classes that attended, with head counts: 0 is absent, null attended
/// but uncounted. A line of more than one class needs every count.
fn attendees(value: Option<&Value>, n: usize) -> Result<Vec<(Class, Option<i64>)>> {
    let bad = || {
        refuse(
            "fbt_entertainment_classifier.fact_invalid",
            format!(
                "line {n}: attendees must be an object of employees, associates and others, each a JSON integer from 0 to 1000000 or null where the count is unknown, at least one more than 0 or null."
            ),
        )
    };
    let Some(Value::Object(given)) = value else {
        return bad();
    };
    if given
        .keys()
        .any(|k| !CLASSES.iter().any(|(name, _)| name == k))
    {
        return bad();
    }
    let mut present = Vec::new();
    for (name, class) in CLASSES {
        match given.get(name) {
            None => {}
            Some(Value::Null) => present.push((class, None)),
            Some(count) => match count.as_i64() {
                Some(0) => {}
                Some(heads @ 1..=1_000_000) => present.push((class, Some(heads))),
                _ => return bad(),
            },
        }
    }
    if present.is_empty() {
        return bad();
    }
    if present.len() > 1 && present.iter().any(|(_, heads)| heads.is_none()) {
        return refuse(
            "fbt_entertainment_classifier.line_not_split",
            format!(
                "line {n}: attendees span more than one class, so each class needs its head count to split the cost per head (TR 97/17 paras 124 and 135); give the counts, or split the line by class."
            ),
        );
    }
    Ok(present)
}

/// `cost` split per head between `attendees`, the last class taking the
/// remainder so the parts add up to the cost to the cent.
fn split(cost: &Decimal, attendees: &[(Class, Option<i64>)]) -> Result<Vec<Decimal>> {
    let total: i64 = attendees.iter().filter_map(|(_, heads)| *heads).sum();
    precise(|| {
        let mut parts = Vec::new();
        let mut left = cost.clone();
        for (i, (_, heads)) in attendees.iter().enumerate() {
            let part = match heads {
                Some(heads) if i + 1 < attendees.len() => {
                    cents(cost.mul(&int(*heads))?.div(&int(total))?)?
                }
                _ => left.clone(),
            };
            left = left.sub(&part)?;
            parts.push(part);
        }
        Ok(parts)
    })
}

/// The running totals.
struct Totals {
    cost: Decimal,
    deductible: Decimal,
    gst_credit: Decimal,
    undetermined: Decimal,
    minor_possible: Decimal,
    meal_entertainment: Decimal,
}

impl Totals {
    fn add(field: &mut Decimal, amount: &Decimal) -> Result<()> {
        *field = precise(|| Ok(field.add(amount)?))?;
        Ok(())
    }
}

/// One attendee class's outcome, adding its amounts to `totals`.
fn classify_class(
    line: &Line,
    (class, heads): (Class, Option<i64>),
    cost: Decimal,
    (character, valuation, register): (Character, Valuation, Option<&Decimal>),
    totals: &mut Totals,
) -> Result<Value> {
    let row = treatment(character, valuation, class, line.premises);
    let name = CLASSES.iter().find(|(_, c)| *c == class).expect("class").0;
    let mut out = json!({
        "attendees": name, "heads": heads, "cost": render(&cost)?,
        "fringe_benefit": match row.fbt {
            Fbt::Arises => "arises",
            Fbt::Exempt => "exempt",
            Fbt::NotAFringeBenefit => "none",
            Fbt::InMethodTotal => "in_method_total",
            Fbt::Undetermined => "undetermined",
        },
    });
    let (kind, kind_authority) = match row.fbt {
        Fbt::InMethodTotal if class != Others => (
            Some("meal_entertainment"),
            Some("FBTAA s 37AC: a meal entertainment benefit, valued for the year as a whole"),
        ),
        Fbt::Arises | Fbt::Exempt => {
            let (kind, authority) = benefit_type(line.provided, line.reimbursed);
            (Some(kind), Some(authority))
        }
        _ => (None, None),
    };
    out["benefit_type"] = json!(kind);
    out["benefit_type_authority"] = json!(kind_authority);
    if row.fbt == Fbt::Exempt {
        out["exemption"] = json!("FBTAA s 41");
    } else if kind.is_some() {
        let (excluded, why) = if character == Meal {
            (true, MEAL_EXCLUDED)
        } else {
            (false, NOT_EXCLUDED)
        };
        out["excluded"] = json!(excluded);
        out["excluded_authority"] = json!(why);
    }
    let share = match row.share {
        Share::Whole => Some(int(1)),
        Share::Nil => Some(int(0)),
        Share::Half => Some(d("0.5")),
        Share::RegisterPercentage => {
            let percentage = register.expect("checked with the method");
            Some(precise(|| Ok(percentage.div(&int(100))?))?)
        }
        Share::Undetermined => None,
    };
    if let Some(share) = share {
        let (credit, deductible) = precise(|| {
            let gst = if line.includes_gst {
                cost.div(&int(11))?
            } else {
                int(0)
            };
            let credit = cents(gst.mul(&share)?)?;
            Ok((credit.clone(), cents(cost.mul(&share)?)?.sub(&credit)?))
        })?;
        out["deductible"] = json!(render(&deductible)?);
        out["gst_credit"] = json!(render(&credit)?);
        Totals::add(&mut totals.cost, &cost)?;
        Totals::add(&mut totals.deductible, &deductible)?;
        Totals::add(&mut totals.gst_credit, &credit)?;
    } else {
        out["deductible"] = Value::Null;
        out["gst_credit"] = Value::Null;
        out["missing"] = json!(PROPERTY_MISSING);
        Totals::add(&mut totals.undetermined, &cost)?;
    }
    out["authority"] = json!(row.authority);
    if row.fbt == Fbt::Arises {
        let per_head = heads
            .map(|heads| precise(|| Ok(cost.div(&int(heads))?)))
            .transpose()?;
        let minor = minor_benefit(per_head.as_ref(), line.infrequent, character)?;
        if minor["status"] == "possible" {
            Totals::add(&mut totals.minor_possible, &cost)?;
        }
        out["minor_benefit"] = minor;
    }
    Ok(out)
}

/// The classifier: each line's character and, for each attendee class, its
/// fringe benefit or exemption, deduction and GST credit, with totals.
pub fn fbt_entertainment_classifier(
    lines: &Arg,
    income_tax_exempt: &Arg,
    year_ended: &Arg,
    scope_confirmed: &Arg,
    method_arg: Option<&Arg>,
    register_arg: Option<&Arg>,
) -> Result<Value> {
    let (_, period, _) = fbt_year(scope_confirmed, year_ended, KIND)?;
    if flag(income_tax_exempt, "income_tax_exempt")? {
        return refuse(
            "fbt_entertainment_classifier.tax_exempt_body",
            "An employer exempt from income tax gives rise to tax-exempt body entertainment fringe benefits instead (FBTAA s 38, TR 97/17 paras 33 and 139 to 143): use fbt-tax-exempt-body-entertainment.",
        );
    }
    let valuation = method_arg
        .map(|arg| method(arg, KIND))
        .transpose()?
        .map(|m| match m {
            "50_50_split" => FiftyFifty,
            "12_week_register" => Register,
            _ => Actual,
        });
    let register = match (valuation, register_arg) {
        (Some(Register), Some(arg)) => match super::super::finite_between(arg, "0", "100")? {
            Some(percentage) => Some(percentage),
            None => return register_missing(),
        },
        (_, Some(_)) => {
            return refuse(
                "input.field_unknown",
                format!(
                    "income_tax_register_percentage is not an input of {COMMAND} without method 12_week_register"
                ),
            );
        }
        _ => None,
    };
    let items = match lines {
        Arg::Json(Value::Array(items)) if (1..=10_000).contains(&items.len()) => items,
        _ => {
            return refuse(
                "fbt_entertainment_classifier.lines_invalid",
                "lines must be a list of 1 to 10000 line objects.",
            );
        }
    };
    let parsed = items
        .iter()
        .enumerate()
        .map(|(i, item)| parse_line(item, i + 1))
        .collect::<Result<Vec<_>>>()?;
    let first_meal = parsed
        .iter()
        .position(|line| matches!(line.character, Some((Meal, _))));
    let valuation = match (valuation, first_meal) {
        (None, Some(i)) => {
            return refuse(
                "fbt_entertainment_classifier.method_required",
                format!(
                    "method is required: line {} is meal entertainment, whose fringe benefits, deduction and GST credit turn on whether the employer values meal entertainment as actual, 50_50_split or 12_week_register (FBTAA Division 9A; ITAA 1936 ss 51AEA and 51AEB).",
                    i + 1
                ),
            );
        }
        (Some(Register), Some(_)) if register.is_none() => return register_missing(),
        (valuation, _) => valuation.unwrap_or(Actual),
    };
    let mut totals = Totals {
        cost: int(0),
        deductible: int(0),
        gst_credit: int(0),
        undetermined: int(0),
        minor_possible: int(0),
        meal_entertainment: int(0),
    };
    let mut rendered = Vec::new();
    for (i, line) in parsed.iter().enumerate() {
        let mut out = json!({"line": i + 1, "cost": render(&line.cost)?});
        let Some((character, authority)) = line.character else {
            out["entertainment"] = json!("undetermined");
            out["missing"] = json!(OCCASION_MISSING);
            Totals::add(&mut totals.undetermined, &line.cost)?;
            rendered.push(out);
            continue;
        };
        out["entertainment"] = json!(match character {
            Meal => "meal_entertainment",
            Recreation => "recreation",
            NotEntertainment => "not_entertainment",
        });
        out["authority"] = json!(authority);
        if character == Meal && valuation != Actual {
            Totals::add(&mut totals.meal_entertainment, &line.cost)?;
        }
        let parts = split(&line.cost, &line.attendees)?;
        let classes = line
            .attendees
            .iter()
            .zip(parts)
            .map(|(attendee, cost)| {
                let facts = (character, valuation, register.as_ref());
                classify_class(line, *attendee, cost, facts, &mut totals)
            })
            .collect::<Result<Vec<_>>>()?;
        out["classes"] = Value::Array(classes);
        rendered.push(out);
    }
    let mut amounts = vec![
        ("cost", totals.cost),
        ("deductible", totals.deductible),
        ("gst_credit", totals.gst_credit),
        ("minor_benefit_possible_cost", totals.minor_possible),
        ("undetermined_cost", totals.undetermined),
    ];
    let elected = valuation != Actual;
    if elected {
        amounts.push(("meal_entertainment_expenditure", totals.meal_entertainment));
    }
    let rates = vec![rate("minor_benefit_threshold", "300")];
    let mut out = super::super::result(KIND, &period, amounts, rates)?;
    if elected {
        let method = if valuation == FiftyFifty {
            "50_50_split"
        } else {
            "12_week_register"
        };
        let expenditure = out["amounts"]["meal_entertainment_expenditure"].clone();
        out["fbt_meal_entertainment_input"] =
            json!({"method": method, "meal_entertainment_expenditure": expenditure});
    }
    out["lines"] = Value::Array(rendered);
    Ok(out)
}

fn register_missing<T>() -> Result<T> {
    refuse(
        "fbt_entertainment_classifier.register_percentage_invalid",
        "income_tax_register_percentage must be a decimal string from 0 to 100 with method 12_week_register: the register percentage of ITAA 1936 s 51AEB(2), the deductions that would be allowable for the register period's meal entertainment over its expenses, sets the deduction and the GST credit.",
    )
}
