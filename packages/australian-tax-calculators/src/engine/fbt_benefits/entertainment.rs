// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The entertainment classifier: for each expense line, whether it is
//! entertainment, the fringe benefit or exemption for each class of attendee,
//! the income tax deduction and the GST credit that follows it. Two tables
//! decide: [`tables::CHARACTER`] from what was provided and on what occasion,
//! then [`tables::TREATMENT`], whose first matching row gives each attendee
//! class's outcome. Read on 16 November 2026, 5.58 pm to 6.11 pm AEDT: ITAA 1997
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

use serde_json::{Value, json};

use super::super::{Arg, Result, cents, d, fbt_year, flag, int, precise, rate, refuse};
use super::method;
use crate::decimal::{Decimal, Rounding};

mod input;
mod tables;

use Character::{Meal, NotEntertainment, Recreation};
use Valuation::{Actual, FiftyFifty, Register};
use input::{Line, parse_line};
use tables::{
    CLASSES, Character, Class, Fbt, MEAL_EXCLUDED, NOT_EXCLUDED, OCCASION_MISSING,
    PROPERTY_MISSING, Share, Treatment, Valuation, benefit_type, treatment,
};

const KIND: &str = "fbt_entertainment_classifier";
const COMMAND: &str = "fbt-entertainment-classifier";

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
    attendee: (Class, Option<i64>),
    cost: Decimal,
    facts: (Character, Valuation, Option<&Decimal>),
    totals: &mut Totals,
) -> Result<Value> {
    let ((class, heads), (character, valuation, register)) = (attendee, facts);
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
    benefit_fields(&mut out, row, line, attendee, character);
    share_fields(&mut out, row, line, &cost, register, totals)?;
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

/// The class's fringe benefit type with its authority, and then the s 41
/// exemption or whether the benefit is excluded from reporting.
fn benefit_fields(
    out: &mut Value,
    row: &Treatment,
    line: &Line,
    attendee: (Class, Option<i64>),
    character: Character,
) {
    let (kind, kind_authority) = match row.fbt {
        Fbt::InMethodTotal => (
            Some("meal_entertainment"),
            Some("FBTAA s 37AC: a meal entertainment benefit, valued for the year as a whole"),
        ),
        Fbt::Arises | Fbt::Exempt => {
            let (kind, authority) = benefit_type(line.provided, line.reimbursed, attendee);
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
}

/// The class's deductible share of `cost` and its GST credit, added to
/// `totals`; or neither, with the fact s 41 turns on, where the row leaves the
/// share undetermined.
fn share_fields(
    out: &mut Value,
    row: &Treatment,
    line: &Line,
    cost: &Decimal,
    register: Option<&Decimal>,
    totals: &mut Totals,
) -> Result<()> {
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
        Totals::add(&mut totals.cost, cost)?;
        Totals::add(&mut totals.deductible, &deductible)?;
        Totals::add(&mut totals.gst_credit, &credit)?;
    } else {
        out["deductible"] = Value::Null;
        out["gst_credit"] = Value::Null;
        out["missing"] = json!(PROPERTY_MISSING);
        Totals::add(&mut totals.undetermined, cost)?;
    }
    Ok(())
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
    let (valuation, register) = valuation_and_register(method_arg, register_arg)?;
    let parsed = parse_lines(lines)?;
    let valuation = settle_valuation(valuation, register.as_ref(), &parsed)?;
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
        rendered.push(classify_line(
            i + 1,
            line,
            valuation,
            register.as_ref(),
            &mut totals,
        )?);
    }
    summary(&period, totals, valuation, rendered)
}

/// The employer's valuation method, where given, and the register
/// percentage the 12 week register method needs.
fn valuation_and_register(
    method_arg: Option<&Arg>,
    register_arg: Option<&Arg>,
) -> Result<(Option<Valuation>, Option<Decimal>)> {
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
    Ok((valuation, register))
}

/// `lines`, each checked in order.
fn parse_lines(lines: &Arg) -> Result<Vec<Line>> {
    let items = match lines {
        Arg::Json(Value::Array(items)) if (1..=10_000).contains(&items.len()) => items,
        _ => {
            return refuse(
                "fbt_entertainment_classifier.lines_invalid",
                "lines must be a list of 1 to 10000 line objects.",
            );
        }
    };
    items
        .iter()
        .enumerate()
        .map(|(i, item)| parse_line(item, i + 1))
        .collect()
}

/// The method the lines need: one whenever a line is meal entertainment,
/// and with the 12 week register, its percentage.
fn settle_valuation(
    valuation: Option<Valuation>,
    register: Option<&Decimal>,
    parsed: &[Line],
) -> Result<Valuation> {
    let first_meal = parsed
        .iter()
        .position(|line| matches!(line.character, Some((Meal, _))));
    match (valuation, first_meal) {
        (None, Some(i)) => refuse(
            "fbt_entertainment_classifier.method_required",
            format!(
                "method is required: line {} is meal entertainment, whose fringe benefits, deduction and GST credit turn on whether the employer values meal entertainment as actual, 50_50_split or 12_week_register (FBTAA Division 9A; ITAA 1936 ss 51AEA and 51AEB).",
                i + 1
            ),
        ),
        (Some(Register), Some(_)) if register.is_none() => register_missing(),
        (valuation, _) => Ok(valuation.unwrap_or(Actual)),
    }
}

/// One line's character and each attendee class's outcome, adding its
/// amounts to `totals`.
fn classify_line(
    n: usize,
    line: &Line,
    valuation: Valuation,
    register: Option<&Decimal>,
    totals: &mut Totals,
) -> Result<Value> {
    let mut out = json!({"line": n, "cost": render(&line.cost)?});
    let Some((character, authority)) = line.character else {
        out["entertainment"] = json!("undetermined");
        out["missing"] = json!(OCCASION_MISSING);
        Totals::add(&mut totals.undetermined, &line.cost)?;
        return Ok(out);
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
            let facts = (character, valuation, register);
            classify_class(line, *attendee, cost, facts, totals)
        })
        .collect::<Result<Vec<_>>>()?;
    out["classes"] = Value::Array(classes);
    Ok(out)
}

/// The result: the totals, each line, and under an election the input
/// fbt-meal-entertainment takes.
fn summary(
    period: &str,
    totals: Totals,
    valuation: Valuation,
    rendered: Vec<Value>,
) -> Result<Value> {
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
    let mut out = super::super::result(KIND, period, amounts, rates)?;
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
