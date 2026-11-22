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

const KIND: &str = "lodgment_required";

/// One year's instrument and its amounts, in whole dollars.
struct Instrument {
    year: &'static str,
    title: &'static str,
    id: &'static str,
    /// A Notice with Tables A to P (to 2022-23), not sections 6 to 8.
    notice: bool,
    /// s 6(10)(b) [Table J(2)]: income more than this.
    study_loan_income: i64,
    /// s 7(2)(b)(ii) [Table K(1)]: taxable income under this, or at most
    /// it when `payment_inclusive`.
    payment_limit: i64,
    payment_inclusive: bool,
    /// s 7(3)(b) [Table K(2)]: rebate income under these, single, apart
    /// and together.
    sapto: [i64; 3],
    due: &'static str,
}

const NOTICE: &str = "Notice of Requirement to Lodge a Return for the Income Year Ended 30 June";
const TAXATION_LAWS: &str = "Taxation Laws (Requirement to Lodge a Return for the";
const OLD_SAPTO: [i64; 3] = [32_280, 31_280, 28_975];
const NEW_SAPTO: [i64; 3] = [34_919, 33_732, 30_994];

const INSTRUMENTS: [Instrument; 6] = [
    Instrument {
        year: "2020-21",
        title: NOTICE,
        id: "F2021L00577",
        notice: true,
        study_loan_income: 11_655,
        payment_limit: 21_885,
        payment_inclusive: false,
        sapto: OLD_SAPTO,
        due: "2021-10-31",
    },
    Instrument {
        year: "2021-22",
        title: NOTICE,
        id: "F2022L00508",
        notice: true,
        study_loan_income: 11_753,
        payment_limit: 21_885,
        payment_inclusive: false,
        sapto: OLD_SAPTO,
        due: "2022-10-31",
    },
    Instrument {
        year: "2022-23",
        title: NOTICE,
        id: "F2023L00533",
        notice: true,
        study_loan_income: 12_090,
        payment_limit: 21_885,
        payment_inclusive: false,
        sapto: OLD_SAPTO,
        due: "2023-10-31",
    },
    Instrument {
        year: "2023-24",
        title: TAXATION_LAWS,
        id: "F2024L00550",
        notice: false,
        study_loan_income: 12_887,
        payment_limit: 21_885,
        payment_inclusive: false,
        sapto: OLD_SAPTO,
        due: "2024-10-31",
    },
    Instrument {
        year: "2024-25",
        title: TAXATION_LAWS,
        id: "F2025L00570",
        notice: false,
        study_loan_income: 13_609,
        payment_limit: 22_575,
        payment_inclusive: false,
        sapto: NEW_SAPTO,
        due: "2025-10-31",
    },
    Instrument {
        year: "2025-26",
        title: TAXATION_LAWS,
        id: "F2026L00618",
        notice: false,
        study_loan_income: 16_750,
        payment_limit: 22_575,
        payment_inclusive: true,
        sapto: NEW_SAPTO,
        due: "2026-10-31",
    },
];

/// Each condition: its key, its provision in the instruments from 2023-24
/// and in the Notices before, and what it says.
const PROVISIONS: [(&str, &str, &str, &str); 23] = [
    (
        "payg_withheld",
        "paragraph 6(2)(a)",
        "Table A item (1)",
        "an amount withheld under PAYG withholding, other than from the payments the paragraph excludes",
    ),
    (
        "tax_loss",
        "paragraph 6(2)(b)",
        "Table A item (2)",
        "a tax loss incurred or deducted",
    ),
    (
        "net_capital_loss",
        "paragraph 6(2)(c)",
        "Table A item (3)",
        "a net capital loss made or applied",
    ),
    (
        "business",
        "paragraph 6(2)(f)",
        "Table A item (6)",
        "a business carried on",
    ),
    (
        "primary_production_trust",
        "paragraph 6(2)(g)",
        "Table A item (7)",
        "income as a beneficiary of a trust that operated a primary production business in Australia",
    ),
    (
        "primary_production_partnership",
        "paragraph 6(2)(h)",
        "Table A item (8)",
        "an interest in a partnership that operated a primary production business in Australia",
    ),
    (
        "under_18",
        "paragraph 6(2)(i)",
        "Table A item (9)",
        "under 18 at all times, with income other than from work, or from dividends and franking credits, over $416",
    ),
    (
        "exempt_overseas_income",
        "paragraph 6(2)(j)",
        "Table A item (10)",
        "income under ITAA 1936 s 23AF or 23AG and $1 or more of other income",
    ),
    (
        "payg_instalment",
        "paragraph 6(2)(k)",
        "Table A item (11)",
        "a PAYG instalment paid",
    ),
    (
        "special_professional",
        "paragraph 6(2)(l)",
        "Table A item (12)",
        "a special professional under ITAA 1997 Division 405",
    ),
    (
        "private_health_offset",
        "paragraph 6(2)(m)",
        "Table A item (13)",
        "entitled to the private health insurance offset and not claiming the correct amount as a premium reduction",
    ),
    (
        "reportable_amounts",
        "paragraph 6(2)(n)",
        "Table A item (14)",
        "reportable fringe benefits or reportable employer superannuation contributions on a payment summary or income statement",
    ),
    (
        "dividends",
        "paragraph 6(2)(o)",
        "Table A item (15)",
        "assessable income from dividends or distributions and franking credits over $18,200",
    ),
    (
        "co_contribution",
        "paragraph 6(2)(p)",
        "Table A item (16)",
        "personal contributions that will receive a super co-contribution",
    ),
    (
        "contributions_cap",
        "paragraph 6(2)(q)",
        "Table A item (17)",
        "a concessional or non-concessional contributions cap exceeded",
    ),
    (
        "super_lump_sum",
        "paragraph 6(2)(r)",
        "Table A item (18)",
        "an Australian superannuation lump sum with an untaxed element at 60 or over, or a taxed or untaxed element under 60",
    ),
    (
        "resident_whole_year",
        "subparagraph 6(3)(a)(i)",
        "Table B item (1)(a)",
        "a resident for the whole year with taxable income over $18,200",
    ),
    (
        "resident_part_year",
        "subparagraph 6(3)(a)(ii)",
        "Table B item (1)(b)",
        "a resident for part of the year with taxable income over the lesser of $18,200 and $13,464 plus $395 a month",
    ),
    (
        "not_resident",
        "paragraph 6(3)(b)",
        "Table B item (2)",
        "not a resident at any time, with income taxable in Australia other than the excluded withholding payments",
    ),
    (
        "study_loan",
        "subsection 6(10)",
        "Table J",
        "a foreign resident with a study or training loan debt on 1 June before the year and income over the year's amount",
    ),
    (
        "working_holiday_maker",
        "paragraph 7(1)(f)",
        "Table P",
        "a working holiday maker with no other assessable income and taxable income, without super or ETP remainders, under $45,001",
    ),
    (
        "listed_payment",
        "subsection 7(2)",
        "Table K item (1)",
        "a payment the instrument lists, and no other assessable income or taxable income under the year's amount",
    ),
    (
        "sapto",
        "subsection 7(3)",
        "Table K item (2)",
        "eligible for the seniors and pensioners tax offset, with rebate income under the year's amount",
    ),
];

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

fn cite(notice: bool, key: &str) -> &'static str {
    let (_, new, old, _) = PROVISIONS
        .iter()
        .find(|p| p.0 == key)
        .expect("known provision");
    if notice { old } else { new }
}

fn says(key: &str) -> &'static str {
    PROVISIONS
        .iter()
        .find(|p| p.0 == key)
        .expect("known provision")
        .3
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
fn section_6_10(w: &mut Work, f: &Facts, i: &Instrument) -> Result<()> {
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
fn listed_payment(given: &Arg, w: &mut Work, f: &Facts, i: &Instrument) -> Result<Option<Relief>> {
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
fn seniors(given: &Arg, w: &mut Work, i: &Instrument) -> Result<Option<Relief>> {
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

/// The exceptions that hold, each with the conditions it relieves.
fn exceptions(
    args: &LodgmentRequiredArgs,
    w: &mut Work,
    f: &Facts,
    i: &Instrument,
) -> Result<Vec<Relief>> {
    let held = [
        holiday_maker(args.working_holiday_maker, w)?,
        listed_payment(args.listed_payment, w, f, i)?,
        seniors(args.sapto, w, i)?,
    ];
    Ok(held.into_iter().flatten().collect())
}

/// s 7(2) names s 6(2)(i) and 6(3); Table K is read with Table B only.
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

/// The answer: the conditions still requiring a return, refusing a case
/// that an exception reaches only in part.
fn decide(w: &Work, held: &[Relief]) -> Result<Vec<&'static str>> {
    let relieved: BTreeSet<&str> = held.iter().flat_map(|(_, r)| r.iter().copied()).collect();
    let left: Vec<&'static str> = w
        .required
        .iter()
        .copied()
        .filter(|k| !relieved.contains(k))
        .collect();
    if !left.is_empty() && left.len() < w.required.len() {
        let names: Vec<&str> = left.iter().map(|k| cite(w.notice, k)).collect();
        return refuse(
            "lodgment_required.exception_overlap",
            format!(
                "An exception relieves a person covered by the provisions it names, but you are also covered by {}; the instrument can be read either way, so the worksheet does not decide.",
                names.join(", ")
            ),
        );
    }
    Ok(left)
}

pub fn lodgment_required(args: &LodgmentRequiredArgs) -> Result<Value> {
    if matches!(args.scope_confirmed, Arg::Bool(true)) && args.year.text() == Some("2026-27") {
        return refuse(
            "lodgment_required.instrument_not_made",
            "No instrument requiring returns for the 2027 year (2026-27) was on the Federal Register of Legislation on 22 November 2026.",
        );
    }
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let i = INSTRUMENTS
        .iter()
        .find(|i| i.year == year)
        .expect("supported year");
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
    let left = decide(&w, &held)?;
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
        held.iter()
            .map(|(k, _)| cite(i.notice, k))
            .collect::<Vec<_>>()
    );
    out["due_date"] = if left.is_empty() {
        Value::Null
    } else {
        json!(i.due)
    };
    out["instrument"] = json!({"id": i.id, "title": title(i)});
    out["tests"] = Value::Object(w.figures);
    out["steps"] = json!(w.steps);
    Ok(out)
}

fn title(i: &Instrument) -> String {
    let end = &i.due[..4];
    if i.notice {
        format!("{} {end}", i.title)
    } else {
        format!("{} {end} Year) Instrument {end}", i.title)
    }
}
