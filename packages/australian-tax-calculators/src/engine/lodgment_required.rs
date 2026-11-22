// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `lodgment_required`: whether an individual must lodge their own income
//! tax return for an income year from 2020-21 to 2025-26 under that year's
//! lodgment instrument, read on the Federal Register of Legislation on
//! 22 November 2026: the Notices of Requirement to Lodge a Return for the
//! income years ended 30 June 2021, 2022 and 2023 (Tables A, B, J, K and P)
//! and the Taxation Laws (Requirement to Lodge a Return for the 2024, 2025
//! and 2026 Year) Instruments (ss 6(2), 6(3), 6(10) and 7). Each condition
//! that turns on a judgement or a figure the worksheet does not work out is
//! the operator's answer. No instrument for 2026-27 (the 2027 year) was on
//! the register that day, so 2026-27 is refused.

use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

use super::stated;
use super::{Arg, Result, flag, int, money, refuse, result, scope};
use crate::decimal::Decimal;
use crate::texts;

const KIND: &str = "lodgment_required";

/// One year's instrument: the year, its register id, the income over which
/// s 6(10)(b) [Table J(2)] applies, the taxable income limit of s 7(2)(b)(ii)
/// [Table K(1)] and whether it is 'less than or equal to' rather than 'less
/// than', and the rebate income limits of s 7(3)(b) [Table K(2)] for single,
/// apart and together; amounts in whole dollars.
type Instrument = (&'static str, &'static str, i64, i64, bool, [i64; 3]);

const OLD_SAPTO: [i64; 3] = [32_280, 31_280, 28_975];
const NEW_SAPTO: [i64; 3] = [34_919, 33_732, 30_994];

const INSTRUMENTS: [Instrument; 6] = [
    ("2020-21", "F2021L00577", 11_655, 21_885, false, OLD_SAPTO),
    ("2021-22", "F2022L00508", 11_753, 21_885, false, OLD_SAPTO),
    ("2022-23", "F2023L00533", 12_090, 21_885, false, OLD_SAPTO),
    ("2023-24", "F2024L00550", 12_887, 21_885, false, OLD_SAPTO),
    ("2024-25", "F2025L00570", 13_609, 22_575, false, NEW_SAPTO),
    ("2025-26", "F2026L00618", 16_750, 22_575, true, NEW_SAPTO),
];

/// A year's instrument, read from its row.
struct Year {
    year: &'static str,
    id: &'static str,
    /// A Notice with Tables A to P (to 2022-23), not sections 6 to 8.
    notice: bool,
    study_loan_income: i64,
    payment_limit: i64,
    payment_inclusive: bool,
    sapto: [i64; 3],
    /// The calendar year the income year ends in.
    end: i64,
}

fn instrument(year: &str) -> Year {
    let (year, id, study_loan_income, payment_limit, payment_inclusive, sapto) = *INSTRUMENTS
        .iter()
        .find(|row| row.0 == year)
        .expect("supported year");
    let end = year[..4].parse::<i64>().expect("year") + 1;
    Year {
        year,
        id,
        notice: end <= 2023,
        study_loan_income,
        payment_limit,
        payment_inclusive,
        sapto,
        end,
    }
}

/// The conditions the operator states in `triggers`.
const STATED: [&str; 14] = [
    "payg_withheld",
    "tax_loss",
    "net_capital_loss",
    "business",
    "primary_production_trust",
    "primary_production_partnership",
    "exempt_overseas_income",
    "payg_instalment",
    "special_professional",
    "private_health_offset",
    "reportable_amounts",
    "co_contribution",
    "contributions_cap",
    "super_lump_sum",
];

const SUBSECTION_6_3: [&str; 3] = ["resident_whole_year", "resident_part_year", "not_resident"];

/// The facts as the command passes them; see the README for each.
pub struct LodgmentRequiredArgs<'a> {
    pub residency: &'a Arg,
    pub taxable_income: &'a Arg,
    pub dividends_and_franking_credits: &'a Arg,
    pub under_18: &'a Arg,
    pub triggers: &'a Arg,
    pub study_loan: &'a Arg,
    pub working_holiday_maker: &'a Arg,
    pub listed_payment: &'a Arg,
    pub sapto: &'a Arg,
    pub child_support_parent: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// A condition's row in [`texts::LODGMENT_PROVISIONS`].
fn provision(key: &str) -> &'static (&'static str, &'static str, &'static str, &'static str) {
    texts::LODGMENT_PROVISIONS
        .iter()
        .find(|p| p.0 == key)
        .expect("known provision")
}

fn cite(notice: bool, key: &str) -> &'static str {
    let (_, new, old, _) = provision(key);
    if notice { old } else { new }
}

fn says(key: &str) -> &'static str {
    provision(key).3
}

/// An optional fact group: `null`, or an object of the keys given.
fn group<'a>(given: &'a Arg, name: &str, keys: &[&str]) -> Result<Option<&'a Map<String, Value>>> {
    match given {
        Arg::Json(Value::Null) => Ok(None),
        Arg::Json(Value::Object(fields)) => {
            stated::known_keys(fields, name, name, keys)?;
            Ok(Some(fields))
        }
        _ => refuse(
            "lodgment_required.group_invalid",
            format!(
                "{name} must be null or an object with {}.",
                keys.join(" and ")
            ),
        ),
    }
}

fn boolean(fields: &Map<String, Value>, item: &str, key: &str) -> Result<bool> {
    match stated::required(fields, item, key)? {
        Value::Bool(b) => Ok(*b),
        _ => refuse(
            "input.flag_invalid",
            format!("{item}: {key} must be a boolean."),
        ),
    }
}

/// Residency in the year: the whole year, part of it (months counted as
/// s 6(3)(a)(ii) counts them) or none (with whether income is taxable in
/// Australia).
enum Residency {
    Whole,
    Part(i64),
    None(bool),
}

fn bad_residency<T>() -> Result<T> {
    refuse(
        "lodgment_required.residency_invalid",
        "residency must be {status: whole_year}, {status: part_year, months: 1 to 12} or {status: none, taxable_australian_income: boolean}.",
    )
}

fn residency(given: &Arg) -> Result<Residency> {
    let Arg::Json(Value::Object(fields)) = given else {
        return bad_residency();
    };
    let status = fields.get("status").and_then(Value::as_str);
    let other = match status {
        Some("whole_year") => None,
        Some("part_year") => Some("months"),
        Some("none") => Some("taxable_australian_income"),
        _ => return bad_residency(),
    };
    let keys: Vec<&str> = ["status"].into_iter().chain(other).collect();
    stated::known_keys(fields, "residency", "residency", &keys)?;
    match other {
        None => Ok(Residency::Whole),
        Some("months") => match fields.get("months").and_then(Value::as_i64) {
            Some(m) if (1..=12).contains(&m) => Ok(Residency::Part(m)),
            _ => bad_residency(),
        },
        Some(key) => Ok(Residency::None(boolean(fields, "residency", key)?)),
    }
}

fn triggers(given: &Arg) -> Result<BTreeSet<&'static str>> {
    let bad = || {
        refuse(
            "lodgment_required.triggers_invalid",
            format!(
                "triggers must be a list of these conditions: {}.",
                STATED.join(", ")
            ),
        )
    };
    let Arg::Json(Value::Array(items)) = given else {
        return bad();
    };
    let mut out = BTreeSet::new();
    for item in items {
        match STATED.iter().find(|k| Some(**k) == item.as_str()) {
            Some(key) => out.insert(*key),
            None => return bad(),
        };
    }
    Ok(out)
}

/// The steps shown, the conditions met and the figures worked out.
struct Work {
    notice: bool,
    steps: Vec<Value>,
    required: Vec<&'static str>,
    figures: Map<String, Value>,
}

impl Work {
    fn push(&mut self, key: &str, finding: impl Into<String>) {
        let label = format!("{}: {}", upper(cite(self.notice, key)), says(key));
        self.steps
            .push(json!({"step": label, "finding": finding.into()}));
    }

    fn test(&mut self, key: &'static str, met: bool, finding: impl Into<String>) {
        let finding = finding.into();
        self.push(
            key,
            format!("{finding}: {}", if met { "met" } else { "not met" }),
        );
        if met {
            self.required.push(key);
        }
    }
}

fn upper(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn more_than(amount: &Decimal, limit: i64) -> Result<bool> {
    Ok(amount.gt(&int(limit))?)
}

/// s 6(2) [Table A]: the stated conditions, then the 2 the worksheet works out.
fn section_6_2(w: &mut Work, f: &Facts) -> Result<()> {
    for key in STATED {
        if f.triggers.contains(key) {
            w.test(key, true, "Yes, as you answered");
        }
    }
    if let Some(income) = &f.under_18 {
        let met = more_than(income, 416)? || more_than(&f.dividends, 416)?;
        w.test(
            "under_18",
            met,
            format!("under 18 at all times; ${income} other than from work and ${} from dividends and franking credits", f.dividends),
        );
    }
    let met = more_than(&f.dividends, 18_200)?;
    w.test("dividends", met, format!("${}", f.dividends));
    Ok(())
}

/// s 6(3) [Table B]: by residency in the year.
fn section_6_3(w: &mut Work, f: &Facts) -> Result<()> {
    match f.residency {
        Residency::Whole => {
            let met = more_than(&f.taxable, 18_200)?;
            w.test(
                "resident_whole_year",
                met,
                format!("taxable income ${}", f.taxable),
            );
        }
        Residency::Part(months) => {
            let limit = (13_464 + 395 * months).min(18_200);
            let met = more_than(&f.taxable, limit)?;
            w.figures
                .insert("part_year_limit".into(), json!(limit.to_string()));
            w.test(
                "resident_part_year",
                met,
                format!("{months} months resident, so the lesser of $18,200 and ${} is ${limit}; taxable income ${}", 13_464 + 395 * months, f.taxable),
            );
        }
        Residency::None(taxable) => {
            w.test(
                "not_resident",
                taxable,
                "income taxable in Australia, as you answered: ".to_owned()
                    + if taxable { "yes" } else { "no" },
            );
        }
    }
    Ok(())
}

/// s 6(10) [Table J]: a foreign resident at some time in the year.
fn section_6_10(w: &mut Work, f: &Facts, i: &Year) -> Result<()> {
    let (Some(income), false) = (&f.study_loan, matches!(f.residency, Residency::Whole)) else {
        return Ok(());
    };
    let met = more_than(income, i.study_loan_income)?;
    w.test(
        "study_loan",
        met,
        format!(
            "income ${income} against more than ${} for {}",
            i.study_loan_income, i.year
        ),
    );
    Ok(())
}

struct Facts {
    residency: Residency,
    taxable: Decimal,
    dividends: Decimal,
    under_18: Option<Decimal>,
    triggers: BTreeSet<&'static str>,
    study_loan: Option<Decimal>,
}

fn facts(args: &LodgmentRequiredArgs) -> Result<Facts> {
    let under_18 = group(args.under_18, "under_18", &["income_excluding_work"])?
        .map(|g| stated::amount(g, "under_18", "income_excluding_work"))
        .transpose()?;
    let study_loan = group(args.study_loan, "study_loan", &["income"])?
        .map(|g| stated::amount(g, "study_loan", "income"))
        .transpose()?;
    Ok(Facts {
        residency: residency(args.residency)?,
        taxable: money(args.taxable_income)?,
        dividends: money(args.dividends_and_franking_credits)?,
        under_18,
        triggers: triggers(args.triggers)?,
        study_loan,
    })
}

type Relief = (&'static str, Vec<&'static str>);

/// s 7(1)(f) [Table P]: relieves every requirement of s 6(2) to 6(10).
fn holiday_maker(given: &Arg, w: &mut Work) -> Result<Option<Relief>> {
    let keys = [
        "other_assessable_income",
        "taxable_income_excluding_remainders",
    ];
    let Some(g) = group(given, "working_holiday_maker", &keys)? else {
        return Ok(None);
    };
    let other = boolean(g, "working_holiday_maker", keys[0])?;
    let taxable = stated::amount(g, "working_holiday_maker", keys[1])?;
    let holds = !other && taxable.lt(&int(45_001))?;
    w.push(
        "working_holiday_maker",
        format!(
            "other assessable income: {}; taxable income without remainders ${taxable}: {}",
            yes_no(other),
            applies(holds)
        ),
    );
    Ok(holds.then(|| ("working_holiday_maker", w.required.clone())))
}

/// s 7(2) [Table K(1)]: a listed payment.
fn listed_payment(given: &Arg, w: &mut Work, f: &Facts, i: &Year) -> Result<Option<Relief>> {
    let Some(g) = group(given, "listed_payment", &["other_assessable_income"])? else {
        return Ok(None);
    };
    let other = boolean(g, "listed_payment", "other_assessable_income")?;
    let limit = int(i.payment_limit);
    let (under, test) = if i.payment_inclusive {
        (f.taxable.le(&limit)?, "at most")
    } else {
        (f.taxable.lt(&limit)?, "less than")
    };
    let holds = !other || under;
    w.push(
        "listed_payment",
        format!(
            "other assessable income: {}; taxable income ${} against {test} ${}: {}",
            yes_no(other),
            f.taxable,
            i.payment_limit,
            applies(holds)
        ),
    );
    Ok(holds.then(|| ("listed_payment", payment_relief(i.notice))))
}

/// s 7(3) [Table K(2)]: the seniors and pensioners tax offset.
fn seniors(given: &Arg, w: &mut Work, i: &Year) -> Result<Option<Relief>> {
    let Some(g) = group(given, "sapto", &["rebate_income", "category"])? else {
        return Ok(None);
    };
    let rebate = stated::amount(g, "sapto", "rebate_income")?;
    let index = match stated::required(g, "sapto", "category")?.as_str() {
        Some("single") => 0,
        Some("apart") => 1,
        Some("together") => 2,
        _ => {
            return refuse(
                "lodgment_required.group_invalid",
                "sapto: category must be single, apart or together.",
            );
        }
    };
    let holds = rebate.lt(&int(i.sapto[index]))?;
    w.push(
        "sapto",
        format!(
            "rebate income ${rebate} against less than ${}: {}",
            i.sapto[index],
            applies(holds)
        ),
    );
    Ok(holds.then(|| ("sapto", SUBSECTION_6_3.to_vec())))
}

/// The exceptions that hold, each with the provisions it names.
fn exceptions(
    args: &LodgmentRequiredArgs,
    w: &mut Work,
    f: &Facts,
    i: &Year,
) -> Result<Vec<Relief>> {
    let held = [
        holiday_maker(args.working_holiday_maker, w)?,
        listed_payment(args.listed_payment, w, f, i)?,
        seniors(args.sapto, w, i)?,
    ];
    Ok(held.into_iter().flatten().collect())
}

/// s 7(2) names s 6(2)(i) and 6(3); section 4 of the Notices reaches a
/// person described in Table B only.
fn payment_relief(notice: bool) -> Vec<&'static str> {
    let mut relief = SUBSECTION_6_3.to_vec();
    if !notice {
        relief.push("under_18");
    }
    relief
}

fn yes_no(answer: bool) -> &'static str {
    if answer { "yes" } else { "no" }
}

fn applies(holds: bool) -> &'static str {
    if holds { "applies" } else { "does not apply" }
}

/// The answer: the conditions still requiring a return and the exceptions
/// applied. An exception that holds for a person covered by a provision it
/// names leaves no requirement 'under this instrument' (s 7(1) to 7(3); in
/// the Notices, section 4 with Table K for a person described in Table B).
fn decide(w: &mut Work, held: Vec<Relief>) -> (Vec<&'static str>, Vec<&'static str>) {
    let mut applied = Vec::new();
    for (key, names) in held {
        if names.iter().any(|k| w.required.contains(k)) {
            applied.push(key);
        } else {
            w.push(
                key,
                "covered by no provision it names, so it relieves nothing",
            );
        }
    }
    let left = if applied.is_empty() {
        w.required.clone()
    } else {
        Vec::new()
    };
    (left, applied)
}

pub fn lodgment_required(args: &LodgmentRequiredArgs) -> Result<Value> {
    if matches!(args.scope_confirmed, Arg::Bool(true)) && args.year.text() == Some("2026-27") {
        return refuse(
            "lodgment_required.instrument_not_made",
            "No instrument requiring returns for the 2027 year (2026-27) was on the Federal Register of Legislation on 22 November 2026.",
        );
    }
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let i = &instrument(&year);
    let f = facts(args)?;
    let parent = flag(args.child_support_parent, "child_support_parent")?;
    let mut w = Work {
        notice: i.notice,
        steps: Vec::new(),
        required: Vec::new(),
        figures: Map::new(),
    };
    section_6_2(&mut w, &f)?;
    section_6_3(&mut w, &f)?;
    section_6_10(&mut w, &f, i)?;
    let held = exceptions(args, &mut w, &f, i)?;
    let (left, applied) = decide(&mut w, held);
    if left.is_empty() && parent {
        return refuse(
            "lodgment_required.child_support_parent",
            "This instrument does not require a return, but a parent liable for or entitled to child support may be required to lodge under the separate instrument for parents for the year; the worksheet does not apply it.",
        );
    }
    let mut out = result(KIND, &year, Vec::new(), Vec::new())?;
    out["lodgment_required"] = json!(!left.is_empty());
    out["required_by"] = json!(left.iter().map(|k| cite(i.notice, k)).collect::<Vec<_>>());
    out["exceptions_applied"] = json!(
        applied
            .iter()
            .map(|k| cite(i.notice, k))
            .collect::<Vec<_>>()
    );
    out["due_date"] = if left.is_empty() {
        Value::Null
    } else {
        json!(format!("{}-10-31", i.end))
    };
    out["instrument"] = json!({"id": i.id, "title": title(i)});
    out["tests"] = Value::Object(w.figures);
    out["steps"] = json!(w.steps);
    Ok(out)
}

fn title(i: &Year) -> String {
    let end = i.end;
    if i.notice {
        format!("Notice of Requirement to Lodge a Return for the Income Year Ended 30 June {end}")
    } else {
        format!("Taxation Laws (Requirement to Lodge a Return for the {end} Year) Instrument {end}")
    }
}
