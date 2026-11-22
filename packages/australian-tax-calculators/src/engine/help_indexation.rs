// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `help_indexation`: a person's accumulated HELP debt on 1 June from 2020 to
//! 2026 under HESA Division 140, and what a voluntary repayment's timing does
//! to it. The citations are to HESA compilation 105 (17 October 2026), the
//! Universities Accord (Student Support and Other Measures) Act 2024 (No. 108,
//! 2024; Schedule 1 items 18, 19 and 56) and the Universities Accord (Cutting
//! Student Debt by 20 Per Cent) Act 2025 (No. 30, 2025; Schedule 2 items 33
//! and 37), read on 22 November 2026. Each factor is the ATO's published rate
//! ('Study and training loan indexation rates', last updated 17 April 2026)
//! plus one; s 140-20 has the Commissioner publish it.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::stated;
use super::{Arg, Result, flag, int, money, precise, py_min, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "help_indexation";
const MAX_ENTRIES: usize = 100;
/// HESA ss 137-19A and 140-26: the amount left after the 20% reduction.
const KEPT: &str = "0.8";
/// The year s 140-26 applies to (the accumulated HELP debt of 1 June 2025).
const REDUCTION_YEAR: &str = "2024-25";
/// HESA s 137-19A(1): debts incurred in this period are reduced by 20%.
const REDUCED_FROM: &str = "2025-01-01";
const REDUCED_TO: &str = "2025-06-01";

/// The HELP debt indexation factor for 1 June at the end of each year, the
/// rule that set it, and the factor first published before the 2024 Act
/// recalculated it (Schedule 1 item 56).
const FACTORS: [(&str, &str, &str, Option<&str>); 7] = [
    ("2019-20", "1.018", "CPI", None),
    ("2020-21", "1.006", "CPI", None),
    ("2021-22", "1.039", "CPI", None),
    ("2022-23", "1.032", "lower of CPI and WPI", Some("1.071")),
    ("2023-24", "1.040", "lower of CPI and WPI", Some("1.047")),
    ("2024-25", "1.032", "lower of CPI and WPI", None),
    ("2025-26", "1.028", "lower of CPI and WPI", None),
];

/// The inputs the command takes besides the location-preferred flag, the year
/// and the scope, in [`HelpIndexationArgs::optional`]'s order.
pub const HELP_INDEXATION_OPTIONAL: [&str; 10] = [
    "previous_accumulated_debt",
    "earlier_debts",
    "voluntary_repayments",
    "compulsory_repayments",
    "assessment_increases",
    "assessment_reductions",
    "current_debts",
    "current_repayments",
    "proposed_repayment",
    "debts_already_reduced",
];

/// The facts as the command passes them; see the README for each.
pub struct HelpIndexationArgs<'a> {
    pub location_preferred: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 10],
}

const DEBT_MISSING: &str = "Supply previous_accumulated_debt, the accumulated HELP debt incurred on the preceding 1 June as your ATO account shows it (nil for none): the command never estimates a balance.";
const LOCATION_PREFERRED: &str = "A Secretary's determination for a location-preferred HELP debtor reduces the indexation factor (HESA s 140-10(1)(a) and (1A)) or the debt (ss 142-15 and 144-10), which this command does not take.";
const ALREADY_REDUCED: &str = "For 2024-25, give the balance and debts before the 20% reduction, and give a debt incurred from 1 January to 1 June 2025 as incurred: the command applies HESA ss 140-26 and 137-19A to them itself, and reducing an amount twice would go beyond what the Act states.";

/// The calendar year in which `year` ends: 2020 for 2019-20.
fn end_year(year: &str) -> i64 {
    year[..4].parse::<i64>().expect("a supported year") + 1
}

fn day(text: &str) -> i64 {
    dates::parse(text).expect("a date")
}

fn entries_invalid<T>(message: String) -> Result<T> {
    refuse("help_indexation.entries_invalid", message)
}

/// An optional amount: nil when not given.
fn amount_or_nil(given: Option<&Arg>) -> Result<Decimal> {
    given.map_or(Ok(int(0)), money)
}

/// The objects of a list input, at most [`MAX_ENTRIES`].
fn objects<'v>(given: Option<&'v Arg>, name: &str) -> Result<Vec<&'v Map<String, Value>>> {
    let items = match given {
        None => return Ok(Vec::new()),
        Some(Arg::Json(Value::Array(items))) if items.len() <= MAX_ENTRIES => items,
        _ => {
            return entries_invalid(format!(
                "{name} must be a list of at most {MAX_ENTRIES} objects."
            ));
        }
    };
    items
        .iter()
        .map(|item| match item {
            Value::Object(fields) => Ok(fields),
            _ => entries_invalid(format!("each entry of {name} must be an object.")),
        })
        .collect()
}

/// A dated entry's day, refused outside `from..=to`.
fn dated(
    fields: &Map<String, Value>,
    item: &str,
    key: &str,
    (from, to): (i64, i64),
) -> Result<i64> {
    let Some(on) = stated::required(fields, item, key)?
        .as_str()
        .and_then(dates::parse)
    else {
        return entries_invalid(format!("{item}: {key} must be a YYYY-MM-DD date."));
    };
    if on < from || on > to {
        return refuse(
            "help_indexation.date_outside_period",
            format!(
                "{item}: {key} {} is outside {} to {}, the period this input covers for the year.",
                dates::format(on),
                dates::format(from),
                dates::format(to)
            ),
        );
    }
    Ok(on)
}

/// HESA s 140-5(1) steps 2 and 2A, or s 140-25(1A): each debt (less 20% when
/// s 137-19A reduces it) added to its group, and each group's total cut to
/// whole dollars. A debt without a `group` is a group of its own. The flag
/// says whether s 137-19A reduced any of them.
fn debt_total(given: Option<&Arg>, name: &str, period: (i64, i64)) -> Result<(Decimal, bool)> {
    let reduced = (day(REDUCED_FROM), day(REDUCED_TO));
    let mut groups: BTreeMap<String, Decimal> = BTreeMap::new();
    let mut any_reduced = false;
    for (i, fields) in objects(given, name)?.into_iter().enumerate() {
        let item = format!("{name} {}", i + 1);
        stated::known_keys(fields, &item, "a debt", &["incurred", "amount", "group"])?;
        let on = dated(fields, &item, "incurred", period)?;
        let mut amount = stated::amount(fields, &item, "amount")?;
        if on >= reduced.0 && on <= reduced.1 {
            amount = amount.mul(&Decimal::parse(KEPT)?)?;
            any_reduced = true;
        }
        let key = match fields.get("group") {
            None => format!("\u{0}{i}"),
            Some(Value::String(group)) if !group.is_empty() => group.clone(),
            Some(_) => {
                return entries_invalid(format!("{item}: group must be a non-empty string."));
            }
        };
        let total = groups.entry(key).or_insert_with(|| int(0));
        *total = total.add(&amount)?;
    }
    let total = groups.values().try_fold(int(0), |sum, group| {
        Ok::<_, super::Error>(sum.add(&group.quantize(0, Rounding::Down)?)?)
    })?;
    Ok((total, any_reduced))
}

/// HESA s 140-5(1) step 3: the voluntary repayments made from 1 June in the
/// preceding financial year to immediately before the next 1 June.
fn repayment_total(given: Option<&Arg>, period: (i64, i64)) -> Result<Decimal> {
    let name = "voluntary_repayments";
    objects(given, name)?
        .into_iter()
        .enumerate()
        .try_fold(int(0), |sum, (i, fields)| {
            let item = format!("{name} {}", i + 1);
            stated::known_keys(fields, &item, "a repayment", &["date", "amount"])?;
            dated(fields, &item, "date", period)?;
            Ok(sum.add(&stated::amount(fields, &item, "amount")?)?)
        })
}

/// The facts read from the inputs, each step's amount as HESA names it.
#[derive(Clone)]
struct Facts {
    previous: Decimal,
    earlier: Decimal,
    repaid: Decimal,
    compulsory: Decimal,
    increases: Decimal,
    reductions: Decimal,
    current: Decimal,
    current_repaid: Decimal,
    proposed: Option<Decimal>,
}

fn facts(year: &str, given: [Option<&Arg>; 10]) -> Result<Facts> {
    let [
        previous,
        earlier,
        repaid,
        compulsory,
        increases,
        reductions,
        current,
        current_repaid,
        proposed,
        reduced,
    ] = given;
    let Some(previous) = previous else {
        return refuse("help_indexation.debt_missing", DEBT_MISSING);
    };
    let y = end_year(year);
    let date = |m: &str, y: i64| day(&format!("{y:04}-{m}"));
    let first_half = (date("07-01", y - 1), date("12-31", y - 1));
    let (earlier, earlier_reduced) = debt_total(
        earlier,
        "earlier_debts",
        (date("01-01", y - 1), date("06-30", y - 1)),
    )?;
    let (current, _) = debt_total(current, "current_debts", first_half)?;
    // Only amounts the command itself reduces must be given unreduced: every
    // input for 2024-25 (s 140-26) and debts of 1 January to 1 June 2025
    // (s 137-19A). The 1 June 2025 debt that 2025-26 starts from is the
    // reduced one (s 140-5(1) step 1).
    let reduces = year == REDUCTION_YEAR || earlier_reduced;
    if reduces
        && reduced
            .map(|r| flag(r, "debts_already_reduced"))
            .transpose()?
            == Some(true)
    {
        return refuse("help_indexation.reduction_already_applied", ALREADY_REDUCED);
    }
    Ok(Facts {
        previous: money(previous)?,
        earlier,
        repaid: repayment_total(repaid, (date("06-01", y - 1), date("05-31", y)))?,
        compulsory: amount_or_nil(compulsory)?,
        increases: amount_or_nil(increases)?,
        reductions: amount_or_nil(reductions)?,
        current,
        current_repaid: amount_or_nil(current_repaid)?,
        proposed: proposed.map(money).transpose()?,
    })
}

/// The amounts from HESA s 140-5(1) steps 1 to 7, s 140-25(1) and s 140-30.
struct Worked {
    before_indexation: Decimal,
    reduction: Decimal,
    indexed_base: Decimal,
    former: Decimal,
    current_net: Decimal,
    current_reduction: Decimal,
    accumulated: Decimal,
}

fn exceeds<T>() -> Result<T> {
    refuse(
        "help_indexation.repayments_exceed_debt",
        "The repayments given exceed the debts they reduce (HESA ss 140-5(1) and 140-25(1)); check the amounts and the periods they were made in.",
    )
}

/// The accumulated HELP debt with `repaid` as the step 3 total.
fn work(year: &str, f: &Facts, repaid: &Decimal, factor: &Decimal) -> Result<Worked> {
    let reduce = year == REDUCTION_YEAR;
    let kept = Decimal::parse(KEPT)?;
    precise(|| {
        let before = f
            .previous
            .add(&f.earlier)?
            .sub(repaid)?
            .sub(&f.compulsory)?;
        let before = before.sub(&f.increases)?.add(&f.reductions)?;
        let current_net = f.current.sub(&f.current_repaid)?;
        if before.lt(&int(0))? || current_net.lt(&int(0))? {
            return exceeds();
        }
        let base = if reduce {
            before.mul(&kept)?
        } else {
            before.clone()
        };
        let current = if reduce {
            current_net.mul(&kept)?
        } else {
            current_net.clone()
        };
        let former = base.mul(factor)?;
        let accumulated = former.add(&current)?.quantize(0, Rounding::Down)?;
        Ok(Worked {
            reduction: before.sub(&base)?,
            before_indexation: before,
            indexed_base: base,
            former,
            current_reduction: current_net.sub(&current)?,
            current_net,
            accumulated,
        })
    })
}

const STEPS: [(&str, &str); 8] = [
    (
        "before_indexation",
        "HESA s 140-5(1) steps 1 to 6: the preceding accumulated HELP debt, plus debts incurred from 1 January to 30 June of the preceding year (each group's total in whole dollars, step 2A), less voluntary repayments made from the preceding 1 June to 31 May, less compulsory repayment amounts assessed in that period and increases to them, plus reductions to them",
    ),
    (
        "reduction",
        "HESA s 140-26(1) (2024-25 only): step 7 reduces the step 6 amount by 20% before indexation",
    ),
    (
        "indexation",
        "HESA s 140-5(1) and s 140-10: the amount after step 6 (or 7) times the HELP debt indexation factor for 1 June, less that amount; the factor is the lower of the CPI and WPI factors from 2022-23 (Act No. 108, 2024, Sch 1 item 56) and the CPI factor before",
    ),
    (
        "current_debts",
        "HESA s 140-25(1) and (1A): debts incurred from 1 July to 31 December of the year, each group's total in whole dollars, less voluntary repayments of them made before 1 June; for 2024-25, 80% of that (s 140-26(2))",
    ),
    (
        "earlier_debts",
        "HESA s 137-19A: a HELP debt incurred from 1 January 2025 to 1 June 2025 is 20% less, before it is grouped",
    ),
    (
        "accumulated_debt",
        "HESA s 140-25(1) and s 140-30: the former accumulated HELP debt plus the current debts, cents disregarded and under $1 nil",
    ),
    (
        "accumulated_debt_if_paid_before_1_june",
        "HESA s 140-5(1) step 3: a voluntary repayment made before 1 June reduces the amount indexed (and for 2024-25 is inside the 20% reduction)",
    ),
    (
        "accumulated_debt_if_paid_on_or_after_1_june",
        "HESA s 140-5(1) step 3 for the next year: a voluntary repayment made on or after 1 June reduces the accumulated HELP debt already indexed",
    ),
];

pub fn help_indexation(args: &HelpIndexationArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    if flag(args.location_preferred, "location_preferred")? {
        return refuse("help_indexation.location_preferred", LOCATION_PREFERRED);
    }
    let f = facts(&year, args.optional)?;
    let (_, factor_text, rule, superseded) = FACTORS
        .iter()
        .find(|(y, ..)| *y == year)
        .expect("a factor for each supported year");
    let factor = Decimal::parse(factor_text)?;
    let w = work(&year, &f, &f.repaid, &factor)?;
    let indexation = precise(|| Ok(w.former.sub(&w.indexed_base)?))?;
    let mut amounts = vec![
        ("previous_accumulated_debt", f.previous.clone()),
        ("earlier_debts", f.earlier.clone()),
        ("voluntary_repayments", f.repaid.clone()),
        ("compulsory_repayments", f.compulsory.clone()),
        ("before_indexation", w.before_indexation.clone()),
        ("reduction", w.reduction.clone()),
        ("indexation", indexation),
        ("former_accumulated_debt", w.former.clone()),
        ("current_debts", w.current_net.sub(&w.current_reduction)?),
        ("current_reduction", w.current_reduction.clone()),
        ("accumulated_debt", w.accumulated.clone()),
    ];
    if let Some(proposed) = &f.proposed {
        amounts.extend(timing(&year, &f, &factor, &w, proposed)?);
    }
    let mut rates = vec![
        rate("indexation_factor", factor_text),
        rate("factor_rule", rule),
    ];
    if let Some(first) = superseded {
        rates.push(rate("superseded_factor", first));
    }
    if year == REDUCTION_YEAR {
        rates.push(rate("reduction_rate", "0.20"));
    }
    let mut out = result(KIND, &year, amounts, rates)?;
    out["indexation_date"] = json!(format!("{}-06-01", end_year(&year)));
    out["authorities"] = Value::Object(
        STEPS
            .iter()
            .map(|(step, text)| ((*step).to_owned(), json!(text)))
            .collect::<Map<_, _>>(),
    );
    Ok(out)
}

/// The accumulated HELP debt with `proposed` repaid before 1 June and on or
/// after it, and the indexation the earlier payment avoids. Paid before
/// 1 June, it goes first to the debts indexed (step 3) and the rest to the
/// year's own debts (s 140-25), as HESA s 151-10(2) applies a payment without
/// directions.
fn timing(
    year: &str,
    f: &Facts,
    factor: &Decimal,
    w: &Worked,
    proposed: &Decimal,
) -> Result<Vec<(&'static str, Decimal)>> {
    let to_indexed = py_min(proposed.clone(), w.before_indexation.clone())?;
    let mut paid = f.clone();
    paid.repaid = f.repaid.add(&to_indexed)?;
    paid.current_repaid = f.current_repaid.add(&proposed.sub(&to_indexed)?)?;
    let before = work(year, &paid, &paid.repaid, factor)?.accumulated;
    let after = w.accumulated.sub(proposed)?;
    let after = if after.lt(&int(0))? { int(0) } else { after };
    Ok(vec![
        ("proposed_repayment", proposed.clone()),
        ("accumulated_debt_if_paid_before_1_june", before.clone()),
        ("accumulated_debt_if_paid_on_or_after_1_june", after.clone()),
        ("saving_if_paid_before_1_june", after.sub(&before)?),
    ])
}
