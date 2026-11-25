// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Explicitly bounded worksheets using Decimal and published official methods,
//! ported from `austaxcalc.calculations` without changing a rate, rounding
//! step, scope statement or message. Every worksheet computes in a 40-digit
//! context, as the original's `localcontext` did.

use std::borrow::Cow;

use serde_json::{Map, Value, json};

use crate::dates;
use crate::decimal::{Calc, Decimal, Rounding, Signal, with_precision};
use crate::metadata::{self as meta, CoefficientRow, Example};
use crate::pytext;
use crate::texts;

mod fbt_benefits;
pub use fbt_benefits::{
    fbt_board, fbt_housing, fbt_lafha, fbt_residual, fbt_tax_exempt_body_entertainment,
};
pub use fbt_benefits::{
    fbt_car_parking, fbt_loan, fbt_meal_entertainment, fbt_meal_entertainment_register,
};
pub use fbt_benefits::{
    fbt_debt_waiver, fbt_entertainment_classifier, fbt_expense_payment, fbt_property,
};
mod fuel_tax_credits;
pub use fuel_tax_credits::{FuelArgs, fuel_tax_credits};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const WORKING_PRECISION: i128 = 40;

/// A refusal: Python's ValueError with its message, or a trapped decimal signal.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Value {
        /// The refused rule's stable code, from `texts::REFUSAL_CODES`.
        code: &'static str,
        message: String,
    },
    Decimal(Signal),
}

impl From<Signal> for Error {
    fn from(signal: Signal) -> Error {
        Error::Decimal(signal)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Value { message, .. } => f.write_str(message),
            Error::Decimal(s) => write!(f, "decimal.{}", s.name()),
        }
    }
}

impl Error {
    /// The refusal's stable code; the message beside it may be reworded.
    pub fn code(&self) -> &'static str {
        match self {
            Error::Value { code, .. } => code,
            Error::Decimal(signal) => signal.code(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn refuse<T>(code: &'static str, message: impl Into<String>) -> Result<T> {
    Err(Error::Value {
        code,
        message: message.into(),
    })
}

pub fn d(text: &str) -> Decimal {
    Decimal::parse(text).expect("decimal literal")
}

fn int(n: i64) -> Decimal {
    Decimal::from_int(n)
}

/// An argument as a caller passed it: the original checks types itself.
#[derive(Clone, Debug)]
pub enum Arg {
    Decimal(Decimal),
    Bool(bool),
    Int(i64),
    /// An integer outside the 64-bit range: still an `int`, never in range.
    BigInt(String),
    Text(String),
    /// Any other value: a float, None, a list or a mapping.
    Other(String),
    /// A JSON value for a parameter the original never had (main residence periods).
    Json(Value),
}

impl Arg {
    /// `repr(value)`.
    fn repr(&self) -> String {
        match self {
            Arg::Decimal(d) => format!("Decimal('{d}')"),
            Arg::Bool(true) => "True".into(),
            Arg::Bool(false) => "False".into(),
            Arg::Int(n) => n.to_string(),
            Arg::BigInt(s) | Arg::Other(s) => s.clone(),
            Arg::Text(s) => pytext::repr(s),
            Arg::Json(v) => v.to_string(),
        }
    }

    /// `str(value)`.
    fn py_str(&self) -> String {
        match self {
            Arg::Text(s) => s.clone(),
            Arg::Decimal(d) => d.to_string(),
            other => other.repr(),
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            Arg::Text(s) => Some(s),
            _ => None,
        }
    }

    /// `type(value) is int`: a bool is not an int here.
    fn int(&self) -> Option<Option<i64>> {
        match self {
            Arg::Int(n) => Some(Some(*n)),
            Arg::BigInt(_) => Some(None),
            _ => None,
        }
    }
}

fn py_min(a: Decimal, b: Decimal) -> Calc<Decimal> {
    Ok(if b.lt(&a)? { b } else { a })
}

fn py_max(a: Decimal, b: Decimal) -> Calc<Decimal> {
    Ok(if b.gt(&a)? { b } else { a })
}

fn lookup<'a, T>(table: &'a [(&str, T)], key: &str) -> &'a T {
    &table
        .iter()
        .find(|(k, _)| *k == key)
        .expect("validated key")
        .1
}

fn sources(kind: &str) -> Vec<&'static str> {
    let mut out = vec![*lookup(texts::SOURCES, kind)];
    if let Some((_, extra)) = texts::EXTRA_SOURCES.iter().find(|(k, _)| *k == kind) {
        out.extend(extra.iter().copied());
    }
    out
}

/// `_money`: a finite non-negative Decimal at most 2dp and 1e12.
pub(crate) fn money(value: &Arg) -> Result<Decimal> {
    let bad = || {
        refuse(
            "input.amount_invalid",
            "Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
        )
    };
    let Arg::Decimal(v) = value else {
        return bad();
    };
    if !v.is_finite() || v.lt(&int(0))? || v.gt(&d("1e12"))? || v.exponent() < -2 {
        return bad();
    }
    Ok(v.clone())
}

/// `_scope`: confirmation, then a supported period.
fn scope(confirmed: &Arg, year: &Arg, kind: &str) -> Result<String> {
    if !matches!(confirmed, Arg::Bool(true)) {
        return refuse(
            "scope.not_confirmed",
            "Establish all stated scope conditions before calculating.",
        );
    }
    let supported = meta::supported_periods(kind);
    match year.text() {
        Some(y) if supported.contains(&y) => Ok(y.to_string()),
        given => {
            let mut message = format!(
                "Unsupported period {}; supported: {}.",
                year.repr(),
                supported.join(", ")
            );
            if given == Some("2027-28")
                && let Some((_, why)) = texts::UNPUBLISHED_2027_28.iter().find(|(k, _)| *k == kind)
            {
                message = format!("{message} {why}");
            }
            refuse("period.unsupported", message)
        }
    }
}

pub(crate) fn whole_dollars(income: &Decimal, what: &str) -> Result<()> {
    if !income.equals(&income.to_integral_value()?)? {
        return refuse(
            "input.amount_not_whole_dollars",
            format!("Supply established whole-dollar {what}."),
        );
    }
    Ok(())
}

/// `_result`: amounts rendered to cents half up, in a 40-digit context.
fn result(
    kind: &str,
    period: &str,
    amounts: Vec<(&str, Decimal)>,
    rates: Vec<(String, String)>,
) -> Result<Value> {
    let mut rendered = Map::new();
    for (key, value) in amounts {
        let cents = with_precision(WORKING_PRECISION, || value.quantize(-2, Rounding::HalfUp))?;
        rendered.insert(key.to_string(), json!(cents.to_string()));
    }
    let rates: Map<String, Value> = rates.into_iter().map(|(k, v)| (k, json!(v))).collect();
    let mut warnings = vec![
        "Operator-confirmed facts and scope; eligibility is not independently verified.",
        "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
    ];
    if let Some((_, extra)) = texts::EXTRA_WARNINGS.iter().find(|(k, _)| *k == kind) {
        warnings.extend(extra.iter().copied());
    }
    Ok(json!({
        "ok": true,
        "engine": "australian-tax-calculators",
        "engine_version": VERSION,
        "calculation": kind,
        "period": period,
        "amounts": rendered,
        "rates": rates,
        "scope": lookup(texts::SCOPES, kind),
        "sources": sources(kind),
        "source_checked": meta::source_review(kind).0,
        "warnings": warnings,
    }))
}

/// One amount and how it was worked out: the provision or ATO page that sets
/// it, its formula, the formula on the operands the arithmetic used, and the
/// unrounded value the amount is rendered from. The FBT steps cite the FBTAA as
/// compiled on 1 July 2026 (C2004A03280) and ATO pages, read on 14 November 2026.
/// The Division 293 steps cite ITAA 1997 compilation 266 (1 July 2026) and the
/// 2013 Imposition Act as made, and the pension minimum steps SISR Schedule 7 as
/// in force from 1 July 2026, all read on 15 November 2026. The PAYG withholding
/// and Schedule 5 steps cite the headings and method steps of the Taxation
/// Administration (Withholding Schedules) Instruments 2024 (F2024L00664), 2025
/// (F2025L00647), (No. 2) 2025 (F2025L01016) and 2026 (F2026L00716), which
/// share them, and the termination payment steps ITAA 1997 compilation 268
/// (1 October 2026) and Schedule 11 of those instruments, all read on
/// 15 November 2026.
struct Step {
    id: Cow<'static, str>,
    authority: &'static str,
    formula: String,
    operands: String,
    value: Decimal,
}

/// A step whose operands are `formula` with each name in turn replaced by the
/// next of `operands`. Numbers, operators (`x` multiplies) and the functions
/// `min`, `max`, `cents`, `dollars` and `floor` stay as written.
fn step(
    id: impl Into<Cow<'static, str>>,
    authority: &'static str,
    formula: &str,
    operands: &[&Decimal],
    value: Decimal,
) -> Step {
    let mut values = operands.iter();
    let (mut filled, mut rest) = (String::new(), formula);
    while let Some(start) = rest.find(|c: char| c.is_ascii_lowercase()) {
        let end = rest[start..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map_or(rest.len(), |n| start + n);
        let (name, tail) = (&rest[start..end], &rest[end..]);
        filled.push_str(&rest[..start]);
        if name == "x" || tail.starts_with('(') {
            filled.push_str(name);
        } else {
            filled.push_str(&values.next().expect("an operand for each name").to_string());
        }
        rest = tail;
    }
    filled.push_str(rest);
    assert!(values.next().is_none(), "a name for each operand");
    let formula = formula.to_string();
    Step {
        id: id.into(),
        authority,
        formula,
        operands: filled,
        value,
    }
}

impl Step {
    /// The id of a step the code names with a literal, as a later formula's
    /// name for it; only the carry-forward years' steps are named at run time.
    fn static_id(&self) -> &'static str {
        match self.id {
            Cow::Borrowed(id) => id,
            Cow::Owned(_) => unreachable!("a step named with a literal"),
        }
    }
}

/// `table`'s (id, authority, formula) rows as steps, with each row's operands
/// and value.
fn steps_from<const N: usize>(
    table: &[(&'static str, &'static str, &str); N],
    rows: [(&[&Decimal], Decimal); N],
) -> Vec<Step> {
    table
        .iter()
        .zip(rows)
        .map(|((id, authority, formula), (operands, value))| {
            step(*id, authority, formula, operands, value)
        })
        .collect()
}

thread_local! {
    /// The steps `derive` collects from the worksheet it runs; `None` for a
    /// plain command, which keeps nothing.
    static STEPS: std::cell::RefCell<Option<Vec<Value>>> = const { std::cell::RefCell::new(None) };
}

/// Run `f`, one command, and return its outcome with the step behind each of
/// its amounts: none for a worksheet that does not derive them. The steps
/// travel beside the result, as the decimal precision does, so the worksheets'
/// public signatures and plain output stay as they were.
pub(crate) fn with_steps<T>(f: impl FnOnce() -> T) -> (T, Vec<Value>) {
    let previous = STEPS.replace(Some(Vec::new()));
    let outcome = f();
    let steps = STEPS.replace(previous).unwrap_or_default();
    (outcome, steps)
}

/// [`result`] for a worksheet that works out each amount in a step. The
/// amounts are the steps' values, and each step's value is its rendered
/// amount, so a derivation cannot differ from the result it explains.
fn derived_result(
    kind: &str,
    period: &str,
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
) -> Result<Value> {
    let amounts = steps
        .iter()
        .map(|s| (s.id.as_ref(), s.value.clone()))
        .collect();
    let out = result(kind, period, amounts, rates)?;
    STEPS.with_borrow_mut(|recorded| {
        if let Some(recorded) = recorded {
            *recorded = steps
                .iter()
                .map(|s| {
                    json!({
                        "id": s.id,
                        "authority": s.authority,
                        "working": format!("{} = {} = {}", s.formula, s.operands, s.value),
                        "value": out["amounts"][s.id.as_ref()],
                    })
                })
                .collect();
        }
    });
    Ok(out)
}

fn rate(key: &str, value: impl ToString) -> (String, String) {
    (key.to_string(), value.to_string())
}

/// Run `f` in the worksheets' 40-digit context.
fn precise<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    with_precision(WORKING_PRECISION, f)
}

pub fn gst(amount: &Arg, gst_inclusive: &Arg, scope_confirmed: &Arg, year: &Arg) -> Result<Value> {
    let year = scope(scope_confirmed, year, "gst")?;
    let amount = money(amount)?;
    let Arg::Bool(inclusive) = gst_inclusive else {
        return refuse("input.flag_invalid", "gst_inclusive must be a boolean.");
    };
    let steps = precise(|| {
        let (tax, exclusive) = if *inclusive {
            let tax = amount.div(&int(11))?;
            (tax.clone(), amount.sub(&tax)?)
        } else {
            (amount.div(&int(10))?, amount.clone())
        };
        let total = exclusive.add(&tax)?;
        let table = if *inclusive {
            &GST_INCLUSIVE_STEPS
        } else {
            &GST_EXCLUSIVE_STEPS
        };
        let exclusive_of: &[&Decimal] = if *inclusive {
            &[&amount, &tax]
        } else {
            &[&amount]
        };
        Ok(steps_from(
            table,
            [
                (&[&amount], tax.clone()),
                (exclusive_of, exclusive.clone()),
                (&[&exclusive, &tax], total),
            ],
        ))
    })?;
    derived_result("gst", &year, steps, vec![rate("gst_rate", "0.10")])
}

/// `gst`'s amounts for a GST-inclusive amount: (id, authority, formula). The
/// GST steps cite the A New Tax System (Goods and Services Tax) Act 1999
/// compilation 96 (C2026C00081, from 1 January 2026), read on 16 November 2026.
const GST_INCLUSIVE_STEPS: [(&str, &str, &str); 3] = [
    (
        "gst",
        "GST Act s 9-70 and s 9-75(1): 10% of the value, which is the price x 10 / 11, so the price / 11",
        "amount / 11",
    ),
    (
        "exclusive",
        "GST Act s 9-75(1): the value of the supply, the price less its GST",
        "amount - gst",
    ),
    (
        "inclusive",
        "GST Act s 9-75(1): the price, the value with its GST",
        "exclusive + gst",
    ),
];

/// `gst`'s amounts for a GST-exclusive amount, the value of the supply.
const GST_EXCLUSIVE_STEPS: [(&str, &str, &str); 3] = [
    (
        "gst",
        "GST Act s 9-70: 10% of the value of the taxable supply",
        "amount / 10",
    ),
    (
        "exclusive",
        "GST Act s 9-75(1): the value of the supply, the amount given",
        "amount",
    ),
    (
        "inclusive",
        "GST Act s 9-75(1): the price, the value with its GST",
        "exclusive + gst",
    ),
];

pub fn resident_tax(taxable_income: &Arg, year: &Arg, scope_confirmed: &Arg) -> Result<Value> {
    let year = scope(scope_confirmed, year, "resident_tax")?;
    let mut out = basic_tax(
        "resident_tax",
        taxable_income,
        &year,
        meta::RESIDENT_TAX_SCALES,
        None,
    )?;
    year_warning(&mut out, texts::RESIDENT_TAX_YEAR_WARNINGS, &year);
    Ok(out)
}

/// Income Tax Rates Act 1986 s 20(1), with ss 16 and 18, compilation 66
/// (1 July 2026), whose Division 4 reads the same in the compilations of
/// 1 July 2024 and 28 March 2025 apart from s 16's list, all read on
/// 23 November 2026.
const PART_YEAR_THRESHOLD: &str = "Income Tax Rates Act 1986 s 20(1) and s 3(1): with one or more part-year residency periods (s 18(1)), the Schedule 7 Part I table applies as if its tax-free threshold were (the $18,200 tax-free threshold - $4,736) + $4,736 x the number of months in the year of income during which there is a part-year residency period / 12";
const ELIGIBLE_PENSIONER: &str = "Income Tax Rates Act 1986 s 18(2)(a): no period is a part-year residency period for an eligible pensioner (s 16), so the full $18,200 tax-free threshold applies";

/// The pro-rated tax-free threshold (s 20(1)), or $18,200 for an eligible
/// pensioner (s 18(2)(a)).
fn part_year_threshold(months: &Arg, eligible_pensioner: Option<&Arg>) -> Result<Step> {
    let m = match months.int() {
        Some(Some(n)) if (1..=12).contains(&n) => int(n),
        _ => {
            return refuse(
                "input.count_invalid",
                "part_year_residency_months must be an integer from 1 to 12.",
            );
        }
    };
    let Some(pensioner) = eligible_pensioner else {
        return refuse(
            "input.missing_field",
            "eligible_pensioner is required with part_year_residency_months.",
        );
    };
    if flag(pensioner, "eligible_pensioner")? {
        let full = int(18200);
        return Ok(step(
            "tax_free_threshold",
            ELIGIBLE_PENSIONER,
            "18200",
            &[],
            full,
        ));
    }
    let value = precise(|| Ok(int(13464).add(&int(4736).mul(&m)?.div(&int(12))?)?))?;
    let formula = "18200 - 4736 + 4736 x part_year_residency_months / 12";
    Ok(step(
        "tax_free_threshold",
        PART_YEAR_THRESHOLD,
        formula,
        &[&m],
        value,
    ))
}

/// `resident_tax`, or for a person resident for part of the year the same with
/// the tax-free threshold pro-rated by `months` unless the person is an
/// eligible pensioner.
pub fn resident_tax_part_year(
    taxable_income: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    months: Option<&Arg>,
    eligible_pensioner: Option<&Arg>,
) -> Result<Value> {
    let Some(months) = months else {
        if eligible_pensioner.is_some() {
            return refuse(
                "input.field_unknown",
                "eligible_pensioner is an input only with part_year_residency_months.",
            );
        }
        return resident_tax(taxable_income, year, scope_confirmed);
    };
    let year = scope(scope_confirmed, year, "resident_tax")?;
    let threshold = part_year_threshold(months, eligible_pensioner)?;
    let mut out = basic_tax(
        "resident_tax",
        taxable_income,
        &year,
        meta::RESIDENT_TAX_SCALES,
        Some(threshold),
    )?;
    year_warning(&mut out, texts::RESIDENT_TAX_YEAR_WARNINGS, &year);
    Ok(out)
}

/// Add the year's warning from `table`, if it has one.
fn year_warning(out: &mut Value, table: &[(&str, &str)], year: &str) {
    if let Some((_, note)) = table.iter().find(|(y, _)| *y == year) {
        out["warnings"]
            .as_array_mut()
            .expect("warnings")
            .push(json!(note));
    }
}

pub fn foreign_resident_tax(
    taxable_income: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "foreign_resident_tax";
    let year = scope(scope_confirmed, year, kind)?;
    let mut out = basic_tax(
        kind,
        taxable_income,
        &year,
        meta::FOREIGN_RESIDENT_TAX_SCALES,
        None,
    )?;
    year_warning(&mut out, texts::FOREIGN_RESIDENT_YEAR_WARNINGS, &year);
    Ok(out)
}

/// Tax scales by income year: (lower bound, upper bound, rate) for each band.
type TaxScales = [(&'static str, &'static [(i64, Option<i64>, &'static str)])];

/// [`step`] with each name in `formula` given the value `values` holds for it,
/// for a formula that names a value more than once.
fn step_by_name(
    id: &'static str,
    authority: &'static str,
    formula: &str,
    values: &[(&str, Decimal)],
    value: Decimal,
) -> Step {
    let mut operands = Vec::new();
    let mut rest = formula;
    while let Some(start) = rest.find(|c: char| c.is_ascii_lowercase()) {
        let end = rest[start..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map_or(rest.len(), |n| start + n);
        let (name, tail) = (&rest[start..end], &rest[end..]);
        if name != "x" && !tail.starts_with('(') {
            let (_, value) = values
                .iter()
                .find(|(n, _)| *n == name)
                .unwrap_or_else(|| panic!("a value for {name}"));
            operands.push(value);
        }
        rest = tail;
    }
    step(id, authority, formula, &operands, value)
}

/// The provisions `basic_tax` applies, in ITAA 1997 compilation 268 (1 October
/// 2026) and the Income Tax Rates Act 1986 compilation 66 (1 July 2026), read
/// on 15 November 2026.
fn basic_tax_authority(kind: &str, year: &str) -> &'static str {
    match (kind, year) {
        ("foreign_resident_tax", _) => {
            "ITAA 1997 s 4-10(3) step 2; Income Tax Rates Act 1986 s 12(1) and Schedule 7 Part II clause 1(b), the table for the 2024-25 year of income or a later year: to $135,000 at the second and to $190,000 at the third resident personal tax rate (s 3(1)), then 45%"
        }
        (_, "2026-27") => {
            "ITAA 1997 s 4-10(3) step 2; Income Tax Rates Act 1986 s 12(1) and Schedule 7 Part I clause 1(b), the table for the 2026-27 year of income: each part above the tax-free threshold (s 3(1)) at its rate"
        }
        (_, "2027-28") => {
            "ITAA 1997 s 4-10(3) step 2; Income Tax Rates Act 1986 s 12(1) and Schedule 7 Part I clause 1(b), the table for the 2027-28 year of income or a later year of income: each part above the tax-free threshold (s 3(1)) at its rate"
        }
        (_, "2024-25" | "2025-26") => {
            "ITAA 1997 s 4-10(3) step 2; Income Tax Rates Act 1986 s 12(1) and Schedule 7 Part I clause 1(b), the table for the 2024-25 or 2025-26 year of income: each part above the tax-free threshold (s 3(1)) at its rate"
        }
        // scope() admits only the years of RESIDENT_TAX_SCALES, each named above.
        (_, other) => unreachable!("no Schedule 7 Part I table is cited for {other}"),
    }
}

/// Basic income tax on whole-dollar taxable income: each band of the year's
/// scale at its rate. `threshold`, when given, is the step for a part-year
/// resident's tax-free threshold, which replaces the first band's lower limit.
/// The tax, each band's term and the values the terms read.
type Bands<'a> = (Decimal, Vec<String>, Vec<(&'a str, Decimal)>);

/// The tax of each band of `scale` on `income`, with its terms and the values
/// they read; the first band starts at `threshold` when one is given.
fn tax_by_band<'a>(
    scale: &[(i64, Option<i64>, &'static str)],
    rates: &'a [(String, String)],
    income: &Decimal,
    threshold: Option<&Step>,
) -> Result<Bands<'a>> {
    let mut values = vec![("taxable_income_used", income.clone())];
    let mut terms = Vec::new();
    let tax = precise(|| {
        let mut tax = int(0);
        for (i, ((low, high, rate), (name, _))) in scale.iter().zip(rates).enumerate() {
            let (ceiling, top) = match high {
                None => (income.clone(), "taxable_income_used".to_string()),
                Some(high) => (
                    py_min(income.clone(), int(*high))?,
                    format!("min(taxable_income_used, {high})"),
                ),
            };
            let (floor, label) = match threshold {
                Some(t) if i == 0 => {
                    values.push(("tax_free_threshold", t.value.clone()));
                    (t.value.clone(), "tax_free_threshold".to_string())
                }
                _ => (int(*low), low.to_string()),
            };
            let band = py_max(int(0), ceiling.sub(&floor)?)?;
            tax = tax.add(&band.mul(&d(rate))?)?;
            terms.push(format!("max(0, {top} - {label}) x {name}"));
            values.push((name.as_str(), d(rate)));
        }
        Ok(tax)
    })?;
    Ok((tax, terms, values))
}

fn basic_tax(
    kind: &str,
    taxable_income: &Arg,
    year: &str,
    scales: &TaxScales,
    threshold: Option<Step>,
) -> Result<Value> {
    let income = money(taxable_income)?;
    whole_dollars(&income, "taxable income")?;
    let scale = lookup(scales, year);
    let mut rates: Vec<(String, String)> = scale
        .iter()
        .map(|(low, _, r)| rate(&format!("over_{low}"), r))
        .collect();
    if threshold.is_some() {
        rates[0].0 = "over_tax_free_threshold".to_string();
    }
    let (tax, terms, values) = tax_by_band(scale, &rates, &income, threshold.as_ref())?;
    let mut steps = vec![step(
        "taxable_income_used",
        "ITAA 1997 s 4-10(3) step 1: taxable income, as established",
        "taxable_income",
        &[&income],
        income.clone(),
    )];
    steps.extend(threshold);
    steps.push(step_by_name(
        "basic_income_tax",
        basic_tax_authority(kind, year),
        &terms.join(" + "),
        &values,
        tax,
    ));
    derived_result(kind, year, steps, rates)
}

pub fn capital_gains(
    other_gains: &Arg,
    discount_gains: &Arg,
    current_losses: &Arg,
    prior_losses: &Arg,
    scope_confirmed: &Arg,
    year: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "capital_gains")?;
    let other = money(other_gains)?;
    let discount = money(discount_gains)?;
    let current = money(current_losses)?;
    let prior = money(prior_losses)?;
    let rates = vec![rate("discount", "0.50")];
    let discount_rate = d(&rates[0].1);
    let steps = precise(|| {
        let losses = current.add(&prior)?;
        let other_offset = py_min(other.clone(), losses.clone())?;
        let discount_offset = py_min(discount.clone(), losses.sub(&other_offset)?)?;
        let used = other_offset.add(&discount_offset)?;
        let remaining = losses.sub(&other_offset)?.sub(&discount_offset)?;
        let discounted = discount
            .sub(&discount_offset)?
            .mul(&int(1).sub(&discount_rate)?)?;
        let net = other.sub(&other_offset)?.add(&discounted)?;
        // The operands of the 2 offsets, as the formulas write them out.
        let other_offset_of = [&other, &current, &prior];
        let discount_offset_of = [&discount, &current, &prior, &other, &current, &prior];
        let used_of = [&other_offset_of[..], &discount_offset_of].concat();
        let remaining_of = [
            &[&current, &prior][..],
            &other_offset_of,
            &discount_offset_of,
        ]
        .concat();
        let discounted_of = [&[&discount][..], &discount_offset_of, &[&discount_rate]].concat();
        let net_of = [&[&other][..], &other_offset_of, &discounted_of].concat();
        Ok(steps_from(
            &CAPITAL_GAINS_STEPS,
            [
                (&used_of[..], used.clone()),
                (&remaining_of[..], remaining),
                (&net_of[..], net),
                (&discounted_of[..], discounted),
            ],
        ))
    })?;
    derived_result("capital_gains", &year, steps, rates)
}

/// `capital_gains`'s amounts: (id, authority, formula). The losses of the year
/// and earlier years reduce the other gains first, then the discount gains
/// (s 102-5(1) step 1 note 1 and step 2 note 2 let the order be chosen). Each
/// formula writes the 2 offsets out as the arithmetic takes them, so a nil
/// amount keeps the sign it is rendered with. The steps cite ITAA 1997
/// compilation 264 (21 May 2026), the text for 2025-26 and 2026-27, read on
/// 20 November 2026: Act No. 49 of 2026, Schedule 1 item 5, renumbers the
/// method statement (discount step 5, net capital gain step 7), and item 82
/// applies it from the income year that includes 1 July 2027.
const CAPITAL_GAINS_STEPS: [(&str, &str, &str); 4] = [
    (
        "losses_used",
        "ITAA 1997 s 102-5(1) steps 1 and 2: capital losses, then earlier net capital losses, reduce the gains",
        "min(other_gains, current_losses + prior_losses) + min(discount_gains, current_losses + prior_losses - min(other_gains, current_losses + prior_losses))",
    ),
    (
        "losses_remaining",
        "ITAA 1997 s 102-10(1) and s 102-15: losses not applied remain a net capital loss",
        "current_losses + prior_losses - min(other_gains, current_losses + prior_losses) - min(discount_gains, current_losses + prior_losses - min(other_gains, current_losses + prior_losses))",
    ),
    (
        "net_capital_gain",
        "ITAA 1997 s 102-5(1) steps 3 and 5 and s 115-100(a): the gains left, each discount gain reduced by the discount percentage",
        "other_gains - min(other_gains, current_losses + prior_losses) + (discount_gains - min(discount_gains, current_losses + prior_losses - min(other_gains, current_losses + prior_losses))) x (1 - discount)",
    ),
    (
        "discount_applied",
        "ITAA 1997 s 102-5(1) step 3 and s 115-100(a): the discount gains left after losses, reduced by the discount percentage",
        "(discount_gains - min(discount_gains, current_losses + prior_losses - min(other_gains, current_losses + prior_losses))) x (1 - discount)",
    ),
];
/// `fbt`'s amounts: (id, authority, formula). The ATO's MantCo example rounds
/// each gross-up to the nearest dollar for items 14A and 14B.
const FBT_STEPS: [(&str, &str, &str); 7] = [
    (
        "type_one_grossed_up",
        "FBTAA s 5B(1B); Calculating your FBT, step 3",
        "type_one_value x (fbt_rate + 0.10) / ((1 - fbt_rate) x (1 + 0.10) x fbt_rate)",
    ),
    (
        "type_two_grossed_up",
        "FBTAA s 5B(1C); Calculating your FBT, step 5",
        "type_two_value / (1 - fbt_rate)",
    ),
    (
        "fbt_estimate",
        "FBTAA s 5B(1A) and s 66; Calculating your FBT, steps 6 and 7",
        "(type_one_grossed_up + type_two_grossed_up) x fbt_rate",
    ),
    (
        "return_item_14a",
        "FBT return 2026 instructions, item 14A; Calculating your FBT, MantCo example",
        "dollars(type_one_grossed_up)",
    ),
    (
        "return_item_14b",
        "FBT return 2026 instructions, item 14B; Calculating your FBT, MantCo example",
        "dollars(type_two_grossed_up)",
    ),
    (
        "return_item_15",
        "FBT return 2026 instructions, item 15",
        "return_item_14a + return_item_14b",
    ),
    (
        "return_item_16",
        "FBT return 2026 instructions, item 16",
        "return_item_15 x fbt_rate",
    ),
];

pub fn fbt(
    type_one_value: &Arg,
    type_two_value: &Arg,
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    // `_scope` reports the period as the original's `str(year_ended)`.
    let year = scope(scope_confirmed, &Arg::Text(year_ended.py_str()), "fbt")?;
    let fbt_rate = d("0.47");
    let gst_rate = d("0.10");
    let steps = precise(|| {
        let kept = int(1).sub(&fbt_rate)?;
        let denominator = kept.mul(&int(1).add(&gst_rate)?)?.mul(&fbt_rate)?;
        let one = money(type_one_value)?;
        let first = one.mul(&fbt_rate.add(&gst_rate)?)?.div(&denominator)?;
        let two = money(type_two_value)?;
        let second = two.div(&kept)?;
        let item_14a = first.quantize(0, Rounding::HalfUp)?;
        let item_14b = second.quantize(0, Rounding::HalfUp)?;
        let item_15 = item_14a.add(&item_14b)?;
        let estimate = first.add(&second)?.mul(&fbt_rate)?;
        let item_16 = item_15.mul(&fbt_rate)?;
        Ok(steps_from(
            &FBT_STEPS,
            [
                (&[&one, &fbt_rate, &fbt_rate, &fbt_rate], first.clone()),
                (&[&two, &fbt_rate], second.clone()),
                (&[&first, &second, &fbt_rate], estimate),
                (&[&first], item_14a.clone()),
                (&[&second], item_14b.clone()),
                (&[&item_14a, &item_14b], item_15.clone()),
                (&[&item_15, &fbt_rate], item_16),
            ],
        ))
    })?;
    // The ATO rates are display values rounded to 4 places. The amounts use
    // the formulas in s 5B(1B) and (1C) at full worksheet precision.
    let rates = vec![
        rate("type_one", "2.0802"),
        rate("type_two", "1.8868"),
        rate("fbt_rate", fbt_rate),
    ];
    derived_result("fbt", &format!("year ended 31 March {year}"), steps, rates)
}

/// A car worksheet's FBT year: `year_ended` must be an integer. Returns the
/// period id, its label and the days in the year (366 when it holds 29 February).
fn fbt_year(confirmed: &Arg, year_ended: &Arg, kind: &str) -> Result<(String, String, i64)> {
    // Only a supported integer passes; anything else is quoted as given.
    let period = match year_ended.int() {
        Some(Some(n)) if meta::supported_periods(kind).contains(&n.to_string().as_str()) => {
            Arg::Text(n.to_string())
        }
        _ => Arg::Other(year_ended.repr()),
    };
    let year = scope(confirmed, &period, kind)?;
    let ended: i64 = year.parse().expect("FBT year");
    let days = if is_leap(ended) { 366 } else { 365 };
    Ok((year, format!("year ended 31 March {ended}"), days))
}

fn day_count(value: &Arg, days_in_year: i64, name: &str) -> Result<i64> {
    match value.int() {
        Some(Some(n)) if (0..=days_in_year).contains(&n) => Ok(n),
        _ => refuse(
            "input.day_count_invalid",
            format!("{name} must be an integer from 0 to {days_in_year}."),
        ),
    }
}

fn flag(value: &Arg, name: &str) -> Result<bool> {
    match value {
        Arg::Bool(b) => Ok(*b),
        _ => refuse("input.flag_invalid", format!("{name} must be a boolean.")),
    }
}

fn cents(value: Decimal) -> Calc<Decimal> {
    value.quantize(-2, Rounding::HalfUp)
}

/// The steps after a car's gross taxable value. The gross value is rounded to
/// cents first: the recipient's payment is in cents, so this moves no taxable
/// value and keeps the applied and excess parts adding up. The payment cannot
/// take the value below nil. Under the operating cost method, s 10(5) then
/// substitutes the statutory formula value when it is lower.
fn car_taxable_values(
    gross: Step,
    contribution: &Decimal,
    statutory: Option<&Decimal>,
    exempt: bool,
) -> Calc<Vec<Step>> {
    let value = cents(gross.value.clone())?;
    let applied = py_min(contribution.clone(), value.clone())?;
    let net = value.sub(&applied)?;
    let payment: [(&[&Decimal], Decimal); 3] = [
        (&[contribution, &value], applied.clone()),
        (&[contribution, &applied], contribution.sub(&applied)?),
        (&[&value, &applied], net.clone()),
    ];
    let mut out = vec![Step {
        formula: format!("cents({})", gross.formula),
        operands: format!("cents({})", gross.operands),
        value: value.clone(),
        ..gross
    }];
    let (before, valuation) = match statutory {
        None => {
            out.extend(steps_from(&S9_PAYMENT_STEPS, payment));
            (net, "FBTAA s 9(1)")
        }
        Some(statutory) => {
            out.extend(steps_from(&S10_PAYMENT_STEPS, payment));
            let lower = py_min(net.clone(), statutory.clone())?;
            let comparison: [(&[&Decimal], Decimal); 2] = [
                (&[statutory], statutory.clone()),
                (&[&net, statutory], lower.clone()),
            ];
            out.extend(steps_from(&S10_5_STEPS, comparison));
            (lower, "FBTAA s 10(2) and (5)")
        }
    };
    out.extend(exemption_steps(&before, exempt, valuation));
    Ok(out)
}

/// An exempt electric car's taxable value is nil under s 8A; another car keeps
/// its value under `valuation`. The value before the exemption is the notional
/// value its reportable fringe benefits amount is worked out from, so it is
/// also the car's reportable taxable value, before the employee's $2,000
/// threshold and the type 2 gross-up, which apply to all their benefits.
fn exemption_steps(before: &Decimal, exempt: bool, valuation: &'static str) -> Vec<Step> {
    let before_exemption = "taxable_value_before_exemption";
    let (authority, formula, operands, taxable): (_, _, &[&Decimal], _) = if exempt {
        ("FBTAA s 8A", "0", &[], int(0))
    } else {
        (valuation, before_exemption, &[before], before.clone())
    };
    let reportable = "Reportable fringe benefits (ATO)";
    vec![
        step("taxable_value", authority, formula, operands, taxable),
        step(
            "reportable_taxable_value",
            reportable,
            before_exemption,
            &[before],
            before.clone(),
        ),
    ]
}

/// The recipient's payment applied, its excess and the value after it.
const APPLIED: &str = "min(employee_contribution, gross_taxable_value)";
const EXCESS: &str = "employee_contribution - employee_contribution_applied";
const NET: &str = "gross_taxable_value - employee_contribution_applied";
const S9_PAYMENT: &str = "FBTAA s 9(1) and s 9(2)(e)";
const S10_PAYMENT: &str = "FBTAA s 10(2) and s 10(3)(c)";

/// The steps from a car's recipient's payment under the statutory formula and
/// under the operating cost method, which then makes the s 10(5) comparison.
const S9_PAYMENT_STEPS: [(&str, &str, &str); 3] = [
    ("employee_contribution_applied", S9_PAYMENT, APPLIED),
    ("excess_employee_contribution", S9_PAYMENT, EXCESS),
    ("taxable_value_before_exemption", "FBTAA s 9(1)", NET),
];
const S10_PAYMENT_STEPS: [(&str, &str, &str); 3] = [
    ("employee_contribution_applied", S10_PAYMENT, APPLIED),
    ("excess_employee_contribution", S10_PAYMENT, EXCESS),
    ("operating_cost_taxable_value", "FBTAA s 10(2)", NET),
];
const S10_5_STEPS: [(&str, &str, &str); 2] = [
    (
        "statutory_formula_taxable_value",
        "FBTAA s 9(1) and s 10(5)",
        "statutory_formula_taxable_value",
    ),
    (
        "taxable_value_before_exemption",
        "FBTAA s 10(5)",
        "min(operating_cost_taxable_value, statutory_formula_taxable_value)",
    ),
];

/// The operating cost method's steps before its gross taxable value.
const S10_COST_STEPS: [(&str, &str, &str); 3] = [
    (
        "deemed_depreciation",
        "FBTAA s 11(1), (1AA) and (1A)",
        "depreciated_value x depreciation_rate x days_held / days_in_year",
    ),
    (
        "deemed_interest",
        "FBTAA s 11(2) and (1B); FBT rates and thresholds, statutory or benchmark interest rate",
        "depreciated_value x benchmark_interest_rate x days_held / days_in_year",
    ),
    (
        "operating_costs",
        "FBTAA s 10(3)(a)",
        "running_costs + lease_charges + deemed_depreciation + deemed_interest",
    ),
];

/// s 9(2)(a): the one-third reduction never reaches accessories added after
/// acquisition.
fn base_value(cost: &Decimal, accessories: &Decimal, reduce: bool) -> Calc<Step> {
    let (reduced, formula, operands): (_, _, &[&Decimal]) = if reduce {
        let formula = "cost_price - cost_price / 3 + later_accessories";
        (
            cost.sub(&cost.div(&int(3))?)?,
            formula,
            &[cost, cost, accessories],
        )
    } else {
        let formula = "cost_price + later_accessories";
        (cost.clone(), formula, &[cost, accessories])
    };
    let base = reduced.add(accessories)?;
    Ok(step(
        "base_value",
        "FBTAA s 9(2)(a)",
        formula,
        operands,
        base,
    ))
}

/// FBTAA s 9: `(A x B x C) / D - E` on the operator's established base value.
#[allow(clippy::too_many_arguments)]
pub fn fbt_car_statutory(
    cost_price: &Arg,
    later_accessories: &Arg,
    one_third_reduction: &Arg,
    days_available: &Arg,
    employee_contribution: &Arg,
    electric_car_exempt: &Arg,
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_car_statutory";
    let (_, period, days_in_year) = fbt_year(scope_confirmed, year_ended, kind)?;
    let cost = money(cost_price)?;
    let accessories = money(later_accessories)?;
    let reduce = flag(one_third_reduction, "one_third_reduction")?;
    let days = day_count(days_available, days_in_year, "days_available")?;
    let contribution = money(employee_contribution)?;
    let exempt = flag(electric_car_exempt, "electric_car_exempt")?;
    let percentage = d("0.20");
    let steps = precise(|| {
        let base = base_value(&cost, &accessories, reduce)?;
        let (held, year) = (int(days), int(days_in_year));
        let gross = base.value.mul(&percentage)?.mul(&held)?.div(&year)?;
        let formula = "base_value x statutory_percentage x days_available / days_in_year";
        let operands = [&base.value, &percentage, &held, &year];
        let gross = step(
            "gross_taxable_value",
            "FBTAA s 9(1)",
            formula,
            &operands,
            gross,
        );
        let mut out = vec![base];
        out.extend(car_taxable_values(gross, &contribution, None, exempt)?);
        Ok(out)
    })?;
    derived_result(
        kind,
        &period,
        steps,
        vec![
            rate("statutory_percentage", &percentage),
            rate("days_available", days),
            rate("days_in_year", days_in_year),
        ],
    )
}

/// s 10(3)(a): an owned car's deemed depreciation and interest at `rates` on
/// its depreciated value for `days` held of the days in the year, then its
/// operating costs.
fn operating_costs(
    running: &Decimal,
    lease: &Decimal,
    value: &Decimal,
    rates: [&Decimal; 2],
    days: [i64; 2],
) -> Calc<Vec<Step>> {
    let ([depreciation_rate, interest_rate], [held, year]) = (rates, days.map(int));
    let deemed = |rate: &Decimal| value.mul(rate)?.mul(&held)?.div(&year);
    let depreciation = deemed(depreciation_rate)?;
    let interest = deemed(interest_rate)?;
    let costs = running.add(lease)?.add(&depreciation)?.add(&interest)?;
    let rows: [(&[&Decimal], Decimal); 3] = [
        (
            &[value, depreciation_rate, &held, &year],
            depreciation.clone(),
        ),
        (&[value, interest_rate, &held, &year], interest.clone()),
        (&[running, lease, &depreciation, &interest], costs),
    ];
    let mut steps = steps_from(&S10_COST_STEPS, rows);
    // A leased car has no depreciated value, so both deemed amounts are nil.
    if lease.gt(&int(0))? {
        for deemed in &mut steps[..2] {
            deemed.authority = "FBTAA s 10(3)(a)(iii): owned cars only, so nil for a leased car";
        }
    }
    Ok(steps)
}

/// FBTAA s 10: `C x (100% - BP) - R`, with deemed depreciation and interest on
/// an owned car's depreciated value (ss 11 and 12), apportioned to the days in
/// the holding period it was owned, and the s 10(5) statutory comparison. The
/// deemed depreciation rate is 200% over the car's effective life (s 11(1AA)).
#[allow(clippy::too_many_arguments)]
pub fn fbt_car_operating_cost(
    running_costs: &Arg,
    lease_charges: &Arg,
    depreciated_value: &Arg,
    effective_life_years: &Arg,
    days_held: &Arg,
    business_use: &Arg,
    employee_contribution: &Arg,
    statutory_formula_taxable_value: &Arg,
    electric_car_exempt: &Arg,
    year_ended: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "fbt_car_operating_cost";
    let (year, period, days_in_year) = fbt_year(scope_confirmed, year_ended, kind)?;
    let running = money(running_costs)?;
    let lease = money(lease_charges)?;
    let value = money(depreciated_value)?;
    if lease.gt(&int(0))? && value.gt(&int(0))? {
        return refuse(
            "fbt_car.cost_basis_conflict",
            "A car is either owned, with a depreciated_value, or leased, with lease_charges; set the other to 0.",
        );
    }
    // An owned car can reach a nil depreciated value, so a nil value does not
    // mean a leased car; either way it deems no depreciation.
    let depreciable = value.gt(&int(0))?;
    // s 11 has no cost cap, so a life under 2 would deem more than the car's value.
    let usable = finite_between(effective_life_years, "2", "1000")?;
    let nil = finite_between(effective_life_years, "0", "0")?;
    let Some(life) = usable.or(if depreciable { None } else { nil }) else {
        return refuse(
            "input.effective_life_invalid",
            "effective_life_years must be a finite Decimal from 2 to 1000 years, or 0 for a leased car or an owned car with no depreciated value left.",
        );
    };
    let days = day_count(days_held, days_in_year, "days_held")?;
    let Some(business) = finite_between(business_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "business_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    let contribution = money(employee_contribution)?;
    let statutory = money(statutory_formula_taxable_value)?;
    let exempt = flag(electric_car_exempt, "electric_car_exempt")?;
    let interest_rate = d(lookup(meta::FBT_BENCHMARK_INTEREST_RATES, &year));
    let private = precise(|| Ok(int(1).sub(&business)?))?;
    let depreciation_rate = precise(|| {
        Ok(if depreciable {
            int(2).div(&life)?
        } else {
            int(0)
        })
    })?;
    let steps = precise(|| {
        let rates = [&depreciation_rate, &interest_rate];
        let mut out = operating_costs(&running, &lease, &value, rates, [days, days_in_year])?;
        let (costs, formula) = (&out[2].value, "operating_costs x private_use");
        let gross = costs.mul(&private)?;
        let gross = step(
            "gross_taxable_value",
            "FBTAA s 10(2)",
            formula,
            &[costs, &private],
            gross,
        );
        out.extend(car_taxable_values(
            gross,
            &contribution,
            Some(&statutory),
            exempt,
        )?);
        Ok(out)
    })?;
    derived_result(
        kind,
        &period,
        steps,
        vec![
            rate("depreciation_rate", &depreciation_rate),
            rate("benchmark_interest_rate", &interest_rate),
            rate("days_held", days),
            rate("days_in_year", days_in_year),
            rate("private_use", &private),
        ],
    )
}

fn finite_between(value: &Arg, low: &str, high: &str) -> Calc<Option<Decimal>> {
    let Arg::Decimal(v) = value else {
        return Ok(None);
    };
    if !v.is_finite() || !(d(low).le(v)? && v.le(&d(high))?) {
        return Ok(None);
    }
    Ok(Some(v.clone()))
}

#[allow(clippy::too_many_arguments)]
pub fn depreciation(
    cost: &Arg,
    effective_life: &Arg,
    days: &Arg,
    taxable_use: &Arg,
    method: &Arg,
    scope_confirmed: &Arg,
    year: &Arg,
    opening_adjustable_value: Option<&Arg>,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "depreciation")?;
    let cost = money(cost)?;
    let Some(life) = finite_between(effective_life, "0.01", "1000")? else {
        return refuse(
            "input.effective_life_invalid",
            "effective_life must be a finite Decimal between 0.01 and 1000 years.",
        );
    };
    let Some(use_) = finite_between(taxable_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "taxable_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    // Exact to the cent only within this bound. For a life of l significant
    // digits and a use of u decimal places, a deduction that is not exactly a
    // half cent is at least 1 / (730 * l-digit integer * 10^u) cents from one;
    // with l + u <= 22 the 40-digit products are exact and the one division errs
    // by under 1e-25 cents on a cost of at most 1e12, which is closer. (A life
    // written with a positive exponent widens that denominator by its trailing
    // zeros, and shrinks the deduction, and so the error, by the same factor.)
    let life_digits = life.adjusted() - life.exponent() + 1;
    if life_digits + (-use_.exponent()).max(0) > 22 {
        return refuse(
            "depreciation.precision_exceeded",
            "effective_life's significant digits and taxable_use's decimal places, as written, must total 22 or fewer: beyond that the 40-digit working precision can miss the exact cent.",
        );
    }
    // Days held can be 366 in a leap year, over 365 (every Guide to
    // depreciating assets from 2021 to 2026): 2023-24 alone of these years.
    let most = dates::parse(&format!("20{}-06-30", &year[5..])).expect("year")
        - dates::parse(&format!("{}-07-01", &year[..4])).expect("year")
        + 1;
    let Some(Some(days)) = days
        .int()
        .filter(|n| n.is_some_and(|n| (0..=most).contains(&n)))
    else {
        return refuse(
            "input.day_count_invalid",
            format!("days must be an integer from 0 to {most} for {year}."),
        );
    };
    let factor = match method.text() {
        Some("prime_cost") => int(1),
        Some("diminishing_value") => int(2),
        _ => {
            return refuse(
                "input.depreciation_method_invalid",
                "method must be prime_cost or diminishing_value.",
            );
        }
    };
    // A later year starts from its opening adjustable value: the diminishing
    // value base and the limit on either method's decline (ss 40-70(1), 40-72(3)
    // and 40-75(7)); prime cost still applies its rate to cost (s 40-75(1)).
    let opening = match opening_adjustable_value {
        None => cost.clone(),
        Some(value) => {
            let opening = money(value)?;
            if opening.gt(&cost)? {
                return refuse(
                    "input.opening_value_above_cost",
                    "opening_adjustable_value cannot exceed cost.",
                );
            }
            opening
        }
    };
    let prime_cost = method.text() == Some("prime_cost");
    let base = if prime_cost { &cost } else { &opening };
    // The names the steps give the opening and base values, and the formula
    // the method applies to the base (200% for diminishing value).
    let opening_name = match opening_adjustable_value {
        None => "cost",
        Some(_) => "opening_adjustable_value",
    };
    let formula = if prime_cost {
        "cost x days".to_string()
    } else {
        format!("{opening_name} x days x 2")
    };
    let steps = precise(|| {
        // Divide once, last: a quotient taken first is rounded to the working
        // precision, which can move an exact half cent below it.
        let numerator = base.mul(&int(days))?.mul(&factor)?;
        let denominator = int(365).mul(&life)?;
        let raw = numerator.div(&denominator)?;
        let days = int(days);
        // py_min(opening, raw): the opening value unless raw is lower.
        let limited = !raw.lt(&opening)?;
        let (decline, deduction) = if limited {
            (opening.clone(), opening.mul(&use_)?)
        } else {
            (raw, numerator.mul(&use_)?.div(&denominator)?)
        };
        // Less the decline as shown, so a closing value carried forward as the
        // next opening value never writes off more than cost.
        let closing = opening.sub(&cents(decline.clone())?)?;
        let deduction_step = if limited {
            step(
                "deduction",
                "ITAA 1997 s 40-25(1) and (2): the decline in value less the part attributable to use for other than a taxable purpose",
                "decline_in_value x taxable_use",
                &[&decline, &use_],
                deduction,
            )
        } else {
            step(
                "deduction",
                "ITAA 1997 s 40-25(1) and (2): the decline in value less the part attributable to use for other than a taxable purpose, worked with one division, last",
                &format!("{formula} x taxable_use / (365 x effective_life)"),
                &[base, &days, &use_, &life],
                deduction,
            )
        };
        Ok(vec![
            step(
                "decline_in_value",
                depreciation_authority(prime_cost, opening_adjustable_value.is_some()),
                &format!("min({opening_name}, {formula} / (365 x effective_life))"),
                &[&opening, base, &days, &life],
                decline.clone(),
            ),
            deduction_step,
            step(
                "closing_adjustable_value",
                "ITAA 1997 s 40-85(1)(b) and (c): the year's opening value less its decline in value",
                &format!("{opening_name} - cents(decline_in_value)"),
                &[&opening, &decline],
                closing,
            ),
        ])
    })?;
    let out = derived_result("depreciation", &year, steps, Vec::new())?;
    let later_year = opening_adjustable_value.is_some();
    Ok(depreciation_pools::depreciation_flags(
        out, &year, prime_cost, later_year,
    ))
}

/// The provisions `depreciation`'s decline in value applies, in ITAA 1997
/// compilation 268 (1 October 2026), read on 16 November 2026: the method's
/// formula on cost in the start year or, in a later year, on the opening
/// adjustable value for diminishing value, and its limit on the decline.
fn depreciation_authority(prime_cost: bool, later_year: bool) -> &'static str {
    match (prime_cost, later_year) {
        (true, false) => {
            "ITAA 1997 s 40-75(1): cost x days held / 365 x 100% / effective life, no more than cost (s 40-75(7)(a))"
        }
        (true, true) => {
            "ITAA 1997 s 40-75(1): cost x days held / 365 x 100% / effective life, no more than the opening adjustable value (s 40-75(7)(b))"
        }
        (false, false) => {
            "ITAA 1997 s 40-72(1): base value, the cost in the start year (s 40-70(1)(a)), x days held / 365 x 200% / effective life, no more than the base value (s 40-72(3))"
        }
        (false, true) => {
            "ITAA 1997 s 40-72(1): base value, the opening adjustable value in a later year (s 40-70(1)(b)), x days held / 365 x 200% / effective life, no more than the base value (s 40-72(3))"
        }
    }
}

/// `quarterly_sg`'s amounts: (id, authority, formula). They cite the
/// Superannuation Guarantee (Administration) Act 1992 (SGAA) compilation 77
/// (C2026C00146), in force from 27 March to 30 June 2026, and the ATO's 'Super
/// guarantee' page (updated 17 April 2026), both read on 15 November 2026.
const QUARTERLY_SG_STEPS: [(&str, &str, &str); 3] = [
    (
        "earnings_used",
        "SGAA s 6(1), 'ordinary time earnings', paragraph (b), and s 15: no more than the quarter's maximum contribution base, $62,500 in 2025-26 (ATO 'Super guarantee', Table 24)",
        "min(ordinary_time_earnings, maximum_quarterly_base)",
    ),
    (
        "minimum_sg",
        "SGAA s 19(2), item 13: a charge percentage of 12 for a quarter from 1 July 2025, on the ordinary time earnings base of s 23(2)",
        "earnings_used x sg_rate",
    ),
    (
        "additional_contribution",
        "SGAA s 23(2) and (10): contributions in the quarter reduce the charge percentage, never below 0",
        "max(0, minimum_sg - qualifying_contributions)",
    ),
];

pub fn quarterly_sg(
    ordinary_time_earnings: &Arg,
    qualifying_contributions: &Arg,
    year: &Arg,
    quarter: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "quarterly_sg")?;
    let Some(Some(quarter)) = quarter
        .int()
        .filter(|n| n.is_some_and(|n| (1..=4).contains(&n)))
    else {
        return refuse(
            "input.quarter_invalid",
            "quarter must be 1 (Jul-Sep), 2, 3 or 4 (Apr-Jun).",
        );
    };
    let steps = precise(|| {
        let (earnings, base) = (money(ordinary_time_earnings)?, d("62500"));
        let used = py_min(earnings.clone(), base.clone())?;
        let paid = money(qualifying_contributions)?;
        let sg_rate = d("0.12");
        let minimum = used.mul(&sg_rate)?;
        let additional = py_max(int(0), minimum.sub(&paid)?)?;
        Ok(steps_from(
            &QUARTERLY_SG_STEPS,
            [
                (&[&earnings, &base], used.clone()),
                (&[&used, &sg_rate], minimum.clone()),
                (&[&minimum, &paid], additional),
            ],
        ))
    })?;
    derived_result(
        "quarterly_sg",
        &format!("{year} Q{quarter}"),
        steps,
        vec![
            rate("sg_rate", "0.12"),
            rate("maximum_quarterly_base", "62500.00"),
        ],
    )
}

fn payg_rows(year: &str, scale: i64) -> Option<&'static [CoefficientRow]> {
    let scales = lookup(meta::PAYG_WITHHOLDING_COEFFICIENTS, year);
    scales
        .iter()
        .find(|(s, _)| *s == scale)
        .map(|(_, rows)| *rows)
}

/// `_payg_checks`: scope, earnings, pay period and scale, in that order. Only
/// the Schedule 1 kinds (`payg_withholding`, and `upward_variation_withholding`
/// and `annuity_withholding`, which work out its amount) and
/// `unused_leave_payment` take scale 4, which Schedules 1 and 7 cover and
/// Schedule 5 does not, and only the Schedule 1 kinds and
/// `study_loan_withholding` quarterly pays, which Schedules 1 and 8 cover.
fn payg_checks(
    kind: &str,
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<(String, Decimal, String, i64)> {
    let year = scope(scope_confirmed, year, kind)?;
    let earnings = money(earnings)?;
    // Schedules 14 and 6 work out the Schedule 1 amount, so take Schedule 1's
    // pays and scale 4 (Schedule 6 applies its own no TFN rule to the payment).
    let schedule_1 = matches!(
        kind,
        "payg_withholding" | "upward_variation_withholding" | "annuity_withholding"
    );
    let quarterly = schedule_1 || kind == "study_loan_withholding";
    let period = match pay_period.text() {
        Some(p @ ("weekly" | "fortnightly" | "monthly")) => p.to_string(),
        Some("quarterly") if quarterly => "quarterly".to_string(),
        _ => {
            return refuse(
                "input.pay_period_invalid",
                if quarterly {
                    "pay_period must be weekly, fortnightly, monthly or quarterly."
                } else {
                    "pay_period must be weekly, fortnightly or monthly."
                },
            );
        }
    };
    let no_tfn = schedule_1 || kind == "unused_leave_payment";
    let unsupported = || {
        refuse(
            "input.tax_scale_unsupported",
            if no_tfn {
                "scale must be 1, 2, 3, 4, 5 or 6."
            } else {
                "scale must be 1, 2, 3, 5 or 6; scale 4 is not supported."
            },
        )
    };
    let Some(Some(scale)) = scale.int() else {
        return unsupported();
    };
    if payg_rows(&year, scale).is_none() && !(no_tfn && scale == 4) {
        return unsupported();
    }
    Ok((year, earnings, period, scale))
}

/// `formula` in brackets unless it is a single name, number or call.
fn group(formula: &str) -> String {
    let mut depth = 0;
    let single = formula.chars().all(|c| {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        c != ' ' || depth > 0
    });
    if single {
        formula.to_string()
    } else {
        format!("({formula})")
    }
}

/// Schedule 1 and 8: the weekly equivalent in whole dollars plus 99 cents.
fn weekly_earnings(earnings: &Decimal, pay_period: &str) -> Calc<Decimal> {
    Ok(weekly_earnings_working(earnings, pay_period, "earnings")?.0)
}

/// [`weekly_earnings`] with its formula on `e`, the formula for the earnings,
/// and the Schedule 1 section that sets it.
fn weekly_earnings_working(
    earnings: &Decimal,
    pay_period: &str,
    e: &str,
) -> Calc<(Decimal, String, &'static str)> {
    let (weekly, formula) = match pay_period {
        "weekly" => (earnings.clone(), e.to_string()),
        "fortnightly" => (earnings.div(&int(2))?, format!("{} / 2", group(e))),
        "quarterly" => (earnings.div(&int(13))?, format!("{} / 13", group(e))),
        // Schedule 1: a monthly amount ending in 33 cents gains a cent first.
        _ if earnings.rem(&int(1))?.equals(&d("0.33"))? => (
            earnings.add(&d("0.01"))?.mul(&int(3))?.div(&int(13))?,
            format!("({e} + 0.01) x 3 / 13"),
        ),
        _ => (
            earnings.mul(&int(3))?.div(&int(13))?,
            format!("{} x 3 / 13", group(e)),
        ),
    };
    let authority = if pay_period == "weekly" {
        "Schedule 1, 'Working out the weekly earnings'"
    } else {
        "Schedule 1, 'Calculating withholding fortnightly, monthly or quarterly amounts'"
    };
    Ok((
        weekly.quantize(0, Rounding::Down)?.add(&d("0.99"))?,
        format!("floor({formula}) + 0.99"),
        authority,
    ))
}

/// y = a * x - b on the first applicable row, rounded straight to the dollar.
fn weekly_formula(x: &Decimal, rows: &[CoefficientRow]) -> Calc<(Decimal, Decimal, Decimal)> {
    let mut chosen = None;
    for (limit, a, b) in rows {
        let applies = match limit {
            None => true,
            Some(limit) => x.lt(&int(*limit))?,
        };
        if applies {
            chosen = Some((d(a), d(b)));
            break;
        }
    }
    let (a, b) = chosen.expect("the last row has no limit");
    let amount = py_max(int(0), a.mul(x)?.sub(&b)?.quantize(0, Rounding::HalfUp)?)?;
    Ok((a, b, amount))
}

/// A rounded weekly amount for the pay period, with its formula on `w`, the
/// formula for the weekly amount: the withholding amount and the Medicare levy
/// adjustment convert the same way.
fn for_pay_period_working(
    weekly_amount: &Decimal,
    pay_period: &str,
    w: &str,
) -> Calc<(Decimal, String)> {
    let w = group(w);
    Ok(match pay_period {
        "weekly" => (weekly_amount.clone(), w),
        "fortnightly" => (weekly_amount.mul(&int(2))?, format!("{w} x 2")),
        "quarterly" => (weekly_amount.mul(&int(13))?, format!("{w} x 13")),
        _ => (
            weekly_amount
                .mul(&int(13))?
                .div(&int(3))?
                .quantize(0, Rounding::HalfUp)?,
            format!("dollars({w} x 13 / 3)"),
        ),
    })
}

/// `payg_withholding`'s optional inputs, each `None` when not given.
#[derive(Default)]
pub struct PaygOptions<'a> {
    pub foreign_resident: Option<&'a Arg>,
    pub tax_offsets: Option<&'a Arg>,
    pub medicare_adjustment_children: Option<&'a Arg>,
    pub extra_pay_withholding: Option<&'a Arg>,
}

/// The optional inputs once checked against the scale and pay period.
struct PaygChoices {
    foreign: Option<bool>,
    offsets: Option<Decimal>,
    children: Option<i64>,
    extra: bool,
}

/// Residency, which scale 4 requires and no other scale takes.
fn foreign_resident(value: Option<&Arg>, scale: i64) -> Result<Option<bool>> {
    match value {
        None if scale == 4 => refuse(
            "input.missing_field",
            "foreign_resident is required for scale 4.",
        ),
        None => Ok(None),
        Some(_) if scale != 4 => refuse(
            "payg.foreign_resident_not_scale_4",
            "foreign_resident applies to scale 4 only; scale 3 is for a foreign resident who gave a TFN.",
        ),
        Some(value) => Ok(Some(flag(value, "foreign_resident")?)),
    }
}

/// The dependent children for a Medicare levy adjustment, scales 2 and 6 only.
fn adjustment_children(value: Option<&Arg>, scale: i64) -> Result<Option<i64>> {
    match value {
        None => Ok(None),
        Some(_) if ![2, 6].contains(&scale) => refuse(
            "payg.medicare_adjustment_not_allowed",
            "A Medicare levy adjustment applies on scales 2 and 6 only.",
        ),
        Some(value) => match value.int() {
            Some(Some(n)) if n >= 0 => Ok(Some(n)),
            _ => refuse(
                "input.children_invalid",
                "medicare_adjustment_children must be a whole number from 0.",
            ),
        },
    }
}

/// The optional inputs checked against the scale and pay period: residency
/// (scale 4), offsets, Medicare levy adjustment children, the 53 or 27 pay ask.
fn payg_options(options: &PaygOptions, scale: i64, period: &str) -> Result<PaygChoices> {
    let foreign = foreign_resident(options.foreign_resident, scale)?;
    let offsets = match options.tax_offsets {
        None => None,
        Some(_) if ![2, 5, 6].contains(&scale) => {
            return refuse(
                "payg.tax_offsets_not_allowed",
                "Tax offsets reduce withholding on scales 2, 5 and 6 only.",
            );
        }
        Some(value) => Some(money(value)?),
    };
    let children = adjustment_children(options.medicare_adjustment_children, scale)?;
    let extra = match options.extra_pay_withholding {
        None => false,
        Some(value) => flag(value, "extra_pay_withholding")?,
    };
    if extra && !matches!(period, "weekly" | "fortnightly") {
        return refuse(
            "payg.extra_pays_not_weekly_or_fortnightly",
            "Schedule 1 has additional amounts for 53 weekly or 27 fortnightly pays only.",
        );
    }
    Ok(PaygChoices {
        foreign,
        offsets,
        children,
        extra,
    })
}

/// Schedule 1 'Medicare levy adjustment' for scale 2 or 6: the weekly family
/// threshold, the shading out point (the scale's multiplier and divisor in
/// 'Medicare levy parameters') and the weekly levy adjustment, rounded
/// straight to the dollar, in the step for `weekly_medicare_levy_adjustment`.
fn weekly_levy_adjustment(
    x: &Decimal,
    rules: &meta::Schedule1Rules,
    scale: i64,
    children: i64,
) -> Calc<(Decimal, Decimal, Step)> {
    let (_, m) = rules
        .medicare_scales
        .iter()
        .find(|(s, _)| *s == scale)
        .expect("scale 2 or 6");
    let per_child = int(rules.family_threshold_per_child).mul(&int(children))?;
    let wft = int(rules.family_threshold)
        .add(&per_child)?
        .div(&int(52))?
        .quantize(-2, Rounding::HalfUp)?;
    let (multiplier, divisor) = m.shading_out_point;
    let sop = wft
        .mul(&d(multiplier))?
        .div(&d(divisor))?
        .quantize(0, Rounding::Down)?;
    let (authority, formula, operands, wla) = if x.lt(&int(m.threshold))? {
        let below = "Schedule 1, 'Medicare levy adjustment': none below the scale's weekly earnings threshold";
        (below, "0".to_string(), vec![], int(0))
    } else if !x.lt(&sop)? {
        let above = "Schedule 1, 'Weekly levy adjustment (WLA)': none from the shading out point";
        (above, "0".to_string(), vec![], int(0))
    } else if x.lt(&int(m.shade_in))? {
        (
            "Schedule 1, 'Weekly levy adjustment (WLA)', formula (1)",
            format!(
                "dollars((weekly_earnings_used - {}) x {})",
                m.formula_1_less, m.shade_in_rate
            ),
            vec![x],
            x.sub(&d(m.formula_1_less))?.mul(&d(m.shade_in_rate))?,
        )
    } else if x.lt(&wft)? {
        (
            "Schedule 1, 'Weekly levy adjustment (WLA)', formula (2)",
            format!("dollars(weekly_earnings_used x {})", m.levy_rate),
            vec![x],
            x.mul(&d(m.levy_rate))?,
        )
    } else {
        let shading = x.sub(&wft)?.mul(&d(m.shade_out_rate))?;
        (
            "Schedule 1, 'Weekly levy adjustment (WLA)', formula (3)",
            format!(
                "dollars(weekly_family_threshold x {} - (weekly_earnings_used - weekly_family_threshold) x {})",
                m.levy_rate, m.shade_out_rate
            ),
            vec![&wft, x, &wft],
            wft.mul(&d(m.levy_rate))?.sub(&shading)?,
        )
    };
    let rounded = wla.quantize(0, Rounding::HalfUp)?;
    let wla = step(
        "weekly_medicare_levy_adjustment",
        authority,
        &formula,
        &operands,
        rounded,
    );
    Ok((wft, sop, wla))
}

/// The additional amount for a year of 53 weekly or 27 fortnightly pays: the
/// last band whose whole-dollar lower limit the pay's earnings reach.
fn extra_pay_amount(
    earnings: &Decimal,
    period: &str,
    rules: &meta::Schedule1Rules,
) -> Calc<Decimal> {
    let bands = if period == "weekly" {
        &rules.extra_weekly
    } else {
        &rules.extra_fortnightly
    };
    let mut amount = 0;
    for (from, extra) in bands {
        if !earnings.lt(&int(*from))? {
            amount = *extra;
        }
    }
    Ok(int(amount))
}

/// One pay as `payg_withholding` checked it, with the year's Schedule 1 rules.
struct SchedulePay {
    year: String,
    earnings: Decimal,
    period: String,
    scale: i64,
    choices: PaygChoices,
    rules: &'static meta::Schedule1Rules,
}

/// A result's steps and rates, in the order it lists them.
struct Workings {
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
}

/// A step's arithmetic before its id is known: authority, formula, operands
/// and value.
type Working = (&'static str, String, Vec<Decimal>, Decimal);

impl SchedulePay {
    /// The amount before tax offsets and adjustments. Scale 4 (note 3 to the
    /// coefficients) applies its rate to the pay's earnings, ignoring cents in
    /// the earnings and in the result; the other scales use the weekly formula.
    fn before_adjustments(&self, w: &mut Workings) -> Result<Working> {
        if self.scale == 4 {
            let (resident, foreign) = self.rules.no_tfn_rates;
            let tax_rate = if self.choices.foreign == Some(true) {
                foreign
            } else {
                resident
            };
            let used = self.earnings.quantize(0, Rounding::Down)?;
            w.rates.push(rate("tax_rate", tax_rate));
            w.steps.push(step(
                "earnings_used",
                "Schedule 1, scale 4 and 'Rounding of withholding amounts': cents in earnings are ignored",
                "floor(earnings)",
                &[&self.earnings],
                used.clone(),
            ));
            let withheld = used.mul(&d(tax_rate))?.quantize(0, Rounding::Down)?;
            return Ok((
                "Schedule 1, scale 4 tax rate and 'Rounding of withholding amounts': cents in the result are ignored",
                "floor(earnings_used x tax_rate)".to_string(),
                vec![used, d(tax_rate)],
                withheld,
            ));
        }
        let rows = payg_rows(&self.year, self.scale).expect("checked scale");
        let (x, formula, authority) =
            weekly_earnings_working(&self.earnings, &self.period, "earnings")?;
        let (a, b, weekly) = weekly_formula(&x, rows)?;
        w.rates.push(rate("a", &a));
        w.rates.push(rate("b", &b));
        w.steps.push(step(
            "weekly_earnings_used",
            authority,
            &formula,
            &[&self.earnings],
            x.clone(),
        ));
        w.steps.push(step(
            "weekly_withholding",
            "Schedule 1, 'Using a formula' and 'Rounding of withholding amounts'",
            "max(0, dollars(a x weekly_earnings_used - b))",
            &[&a, &x, &b],
            weekly.clone(),
        ));
        let (withheld, formula) =
            for_pay_period_working(&weekly, &self.period, "weekly_withholding")?;
        let authority = if self.period == "weekly" {
            "Schedule 1, 'Using a formula'"
        } else {
            "Schedule 1, 'Calculating withholding fortnightly, monthly or quarterly amounts'"
        };
        Ok((authority, formula, vec![weekly], withheld))
    }

    /// The tax offsets and the Medicare levy adjustment claimed, which reduce
    /// the amount before them.
    fn reductions(&self, w: &mut Workings) -> Result<Decimal> {
        let mut reductions = int(0);
        if let Some(offsets) = &self.choices.offsets {
            let share = *lookup(&self.rules.tax_offset_rates, &self.period);
            let reduction = offsets.mul(&d(share))?.quantize(0, Rounding::HalfUp)?;
            w.rates.push(rate("tax_offset_rate", share));
            reductions = reductions.add(&reduction)?;
            w.steps.push(step(
                "tax_offset_reduction",
                "Schedule 1, 'Tax offsets'",
                "dollars(tax_offsets x tax_offset_rate)",
                &[offsets, &d(share)],
                reduction,
            ));
        }
        if let Some(children) = self.choices.children {
            let x = weekly_earnings(&self.earnings, &self.period)?;
            let (wft, sop, weekly) = weekly_levy_adjustment(&x, self.rules, self.scale, children)?;
            let (adjustment, formula) = for_pay_period_working(
                &weekly.value,
                &self.period,
                "weekly_medicare_levy_adjustment",
            )?;
            w.rates.push(rate("medicare_adjustment_children", children));
            w.rates.push(rate("weekly_family_threshold", wft));
            w.rates.push(rate("shading_out_point", sop));
            reductions = reductions.add(&adjustment)?;
            let authority = match self.period.as_str() {
                "weekly" => "Schedule 1, 'Weekly levy adjustment (WLA)'",
                "fortnightly" => "Schedule 1, 'Fortnightly levy adjustment'",
                "monthly" => "Schedule 1, 'Monthly levy adjustment'",
                _ => "Schedule 1, 'Quarterly levy adjustment'",
            };
            let operands = [&weekly.value];
            let adjustment = step(
                "medicare_levy_adjustment",
                authority,
                &formula,
                &operands,
                adjustment,
            );
            w.steps.push(weekly);
            w.steps.push(adjustment);
        }
        Ok(reductions)
    }

    fn workings(&self) -> Result<Workings> {
        let mut w = Workings {
            steps: Vec::new(),
            rates: vec![rate("scale", self.scale)],
        };
        let (authority, formula, operands, before) = self.before_adjustments(&mut w)?;
        let operands: Vec<&Decimal> = operands.iter().collect();
        let choices = &self.choices;
        let adjusted = choices.offsets.is_some() || choices.children.is_some() || choices.extra;
        if !adjusted {
            w.steps
                .push(step("withholding", authority, &formula, &operands, before));
            return Ok(w);
        }
        w.steps.push(step(
            "withholding_before_adjustments",
            authority,
            &formula,
            &operands,
            before.clone(),
        ));
        let reductions = self.reductions(&mut w)?;
        // Schedule 1 says these reduce the amount withheld, not what an excess
        // means, so an excess is refused rather than taken as nil.
        if before.lt(&reductions)? {
            return refuse(
                "payg.reductions_exceed_withholding",
                format!(
                    "The tax offsets and Medicare levy adjustment (${}) exceed the amount withheld before them (${}); Schedule 1 does not say how to treat the excess. Check the payee's declarations and these inputs; if they are right, ask the ATO how much to withhold before making the payment.",
                    cents(reductions)?,
                    cents(before)?
                ),
            );
        }
        let mut withholding = before.sub(&reductions)?;
        if choices.extra {
            let additional = extra_pay_amount(&self.earnings, &self.period, self.rules)?;
            withholding = withholding.add(&additional)?;
            let authority = if self.period == "weekly" {
                "Schedule 1, 'When there are 53 pays in a financial year': the band the weekly earnings reach"
            } else {
                "Schedule 1, 'When there are 27 pays in a financial year': the band the fortnightly earnings reach"
            };
            let amount = additional.to_string();
            w.steps.push(step(
                "additional_withholding",
                authority,
                &amount,
                &[],
                additional,
            ));
        }
        // The amount before them, less the reductions, plus any additional amount.
        let mut formula = "withholding_before_adjustments".to_string();
        let mut operands = Vec::new();
        for s in &w.steps {
            let sign = match s.id.as_ref() {
                "withholding_before_adjustments" => "",
                "tax_offset_reduction" | "medicare_levy_adjustment" => " - ",
                "additional_withholding" => " + ",
                _ => continue,
            };
            if !sign.is_empty() {
                formula.push_str(sign);
                formula.push_str(&s.id);
            }
            operands.push(s.value.clone());
        }
        let operands: Vec<&Decimal> = operands.iter().collect();
        let total = step(
            "withholding",
            "Schedule 1, 'Tax offsets' and 'Medicare levy adjustment', and the additional amount for 53 or 27 pays",
            &formula,
            &operands,
            withholding,
        );
        w.steps.push(total);
        Ok(w)
    }
}

pub fn payg_withholding(
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    options: &PaygOptions,
) -> Result<Value> {
    let (year, earnings, period, scale) = payg_checks(
        "payg_withholding",
        earnings,
        pay_period,
        scale,
        year,
        scope_confirmed,
    )?;
    let rules = *lookup(meta::PAYG_WITHHOLDING_RULES, &year);
    let pay = SchedulePay {
        choices: payg_options(options, scale, &period)?,
        rules,
        year,
        earnings,
        period,
        scale,
    };
    let w = precise(|| pay.workings())?;
    derived_result(
        "payg_withholding",
        &format!("{} {}", pay.year, pay.period),
        w.steps,
        w.rates,
    )
}

/// The pay date checked against the income year and the first payment the
/// year's Schedule 8 coefficients apply to. It is required when they start
/// after 1 July, as 2025-26's do.
fn schedule_8_pay_date(pay_date: Option<&Arg>, year: &str) -> Result<()> {
    let start: i64 = year[..4].parse().expect("income year");
    let first = dates::parse(&format!("{start}-07-01")).expect("income year");
    let last = dates::parse(&format!("{}-06-30", start + 1)).expect("income year");
    let from = *lookup(meta::STUDY_LOAN_WITHHOLDING_FROM, year);
    let applies_from = dates::parse(from).expect("schedule start");
    let day = match pay_date {
        Some(value) => date_arg(value, "pay_date")?,
        None if applies_from == first => return Ok(()),
        None => {
            return refuse(
                "input.missing_field",
                format!(
                    "pay_date is required for {year}: its Schedule 8 coefficients apply to payments made on or after {from}."
                ),
            );
        }
    };
    if day < first || day > last {
        return refuse(
            "payg.pay_date_outside_year",
            format!(
                "pay_date {} is outside the {year} income year.",
                dates::format(day)
            ),
        );
    }
    if day < applies_from {
        return refuse(
            "payg.pay_date_before_schedule",
            format!(
                "Schedule 8's component coefficients for {year} apply to payments made on or after {from}; earlier payments used a percentage of weekly earnings, which this worksheet does not hold."
            ),
        );
    }
    Ok(())
}

/// Schedule 8's component coefficients for a Schedule 1 scale: scale 1 has no
/// tax-free threshold; the other scales claim it or are foreign residents.
fn study_loan_rows(year: &str, scale: i64) -> &'static [CoefficientRow] {
    let table = if scale == 1 {
        "no_threshold"
    } else {
        "threshold_or_foreign"
    };
    lookup(
        lookup(meta::STUDY_LOAN_WITHHOLDING_COEFFICIENTS, year),
        table,
    )
}

/// [`for_pay_period_working`] on a weekly formula amount: the
/// formula y = a x - b on `weekly_earnings_used`, rounded to the dollar and
/// never below nil, whose coefficients are the rates named `[a, b]`.
fn formula_for_pay_period(
    weekly: &Decimal,
    period: &str,
    [a, b]: [&str; 2],
) -> Calc<(Decimal, String)> {
    let formula = format!("max(0, dollars({a} x weekly_earnings_used - {b}))");
    for_pay_period_working(weekly, period, &formula)
}

/// Schedule 8's component on the weekly earnings `x`, converted to the pay
/// period, as the `study_loan_component` step, with its coefficients as the
/// rates `component_a` and `component_b`.
fn study_loan_component(
    x: &Decimal,
    period: &str,
    rows: &[CoefficientRow],
) -> Calc<(Step, [(String, String); 2])> {
    let (a, b, weekly) = weekly_formula(x, rows)?;
    let (component, formula) =
        formula_for_pay_period(&weekly, period, ["component_a", "component_b"])?;
    let authority = if period == "weekly" {
        "Schedule 8, 'Study and training support loans component rates' and 'Rounding of withholding amounts'"
    } else {
        "Schedule 8, 'Study and training support loans component rates' and 'Calculating fortnightly, monthly or quarterly withholding amounts'"
    };
    Ok((
        step(
            "study_loan_component",
            authority,
            &formula,
            &[&a, x, &b],
            component,
        ),
        [rate("component_a", a), rate("component_b", b)],
    ))
}

/// Schedule 8's component added to the Schedule 1 amount. The steps cite
/// Schedules 1 and 8 of the Withholding Schedules Instruments (No. 2) 2025
/// (F2025L01016) and 2026 (F2026L00716), whose headings they share, read on
/// 15 November 2026.
pub fn study_loan_withholding(
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    pay_date: Option<&Arg>,
) -> Result<Value> {
    let (year, earnings, period, scale) = payg_checks(
        "study_loan_withholding",
        earnings,
        pay_period,
        scale,
        year,
        scope_confirmed,
    )?;
    schedule_8_pay_date(pay_date, &year)?;
    let tax_rows = payg_rows(&year, scale).expect("checked scale");
    let loan_rows = study_loan_rows(&year, scale);
    let weekly = period == "weekly";
    let (steps, rates) = precise(|| {
        let (x, x_formula, _) = weekly_earnings_working(&earnings, &period, "earnings")?;
        let (a, b, weekly_tax) = weekly_formula(&x, tax_rows)?;
        let (tax, tax_formula) = formula_for_pay_period(&weekly_tax, &period, ["a", "b"])?;
        let (component, component_rates) = study_loan_component(&x, &period, loan_rows)?;
        let [x_authority, tax_authority] = if weekly {
            [
                "Schedule 8, 'Using a formula': the weekly earnings in whole dollars plus 99 cents",
                "Schedule 1, 'Using a formula' and 'Rounding of withholding amounts'",
            ]
        } else {
            [
                "Schedule 8, 'Calculating fortnightly, monthly or quarterly withholding amounts'",
                "Schedule 1, 'Using a formula' and 'Calculating withholding fortnightly, monthly or quarterly amounts'",
            ]
        };
        let total = step(
            "withholding",
            "Schedule 8, 'Software': the sum of the Schedule 1 amount and the component, which the schedule accepts in place of its combined coefficients",
            "schedule_1_withholding + study_loan_component",
            &[&tax, &component.value],
            tax.add(&component.value)?,
        );
        let mut rates = vec![rate("scale", scale), rate("a", &a), rate("b", &b)];
        rates.extend(component_rates);
        Ok((
            vec![
                step(
                    "weekly_earnings_used",
                    x_authority,
                    &x_formula,
                    &[&earnings],
                    x.clone(),
                ),
                step(
                    "schedule_1_withholding",
                    tax_authority,
                    &tax_formula,
                    &[&a, &x, &b],
                    tax,
                ),
                component,
                total,
            ],
            rates,
        ))
    })?;
    derived_result(
        "study_loan_withholding",
        &format!("{year} {period}"),
        steps,
        rates,
    )
}

mod payg_gross_up;
pub use payg_gross_up::payg_gross_up;

/// The optional `medicare-levy` inputs, each `None` when not given.
pub struct MedicareFamily<'a> {
    pub sapto_entitled: Option<&'a Arg>,
    pub spouse_taxable_income: Option<&'a Arg>,
    pub spouse_sapto_entitled: Option<&'a Arg>,
    pub dependant_offset_entitled: Option<&'a Arg>,
    pub dependent_children: Option<&'a Arg>,
}

/// A spouse on the last day of the year: taxable income and SAPTO entitlement.
struct Spouse {
    income: Decimal,
    sapto: bool,
}

/// The facts that select the thresholds and whether s 8 applies.
struct Family {
    sapto: bool,
    spouse: Option<Spouse>,
    /// s 8(1)(b) or (c), for a person without a spouse.
    dependant_offset: bool,
    children: i64,
}

impl Family {
    /// s 8(1) and (2) apply to a person with a spouse or one of those offsets.
    fn reduces(&self) -> bool {
        self.spouse.is_some() || self.dependant_offset
    }
}

/// The optional `medicare-levy` exemption days, each `None` when not given.
pub struct MedicareExemption<'a> {
    pub full_days: Option<&'a Arg>,
    pub half_days: Option<&'a Arg>,
    pub non_resident_days: Option<&'a Arg>,
}

/// The days in the year a person was in a Medicare levy exemption category.
struct Exemption {
    full: i64,
    half: i64,
    /// The days a non-resident and so a prescribed person (ITAA 1936
    /// s 251U(1)(d)), each a whole day: s 251U(3) gives no half under para (d).
    non_resident: i64,
    days_in_year: i64,
}

/// The non-resident days, from 0 to the days in the year less 1: a person not
/// resident at any time pays no Medicare levy (ITAA 1936 s 251S(1)(a)).
fn non_resident_days(value: Option<&Arg>, days_in_year: i64) -> Result<i64> {
    let Some(value) = value else {
        return Ok(0);
    };
    if value.int() == Some(Some(days_in_year)) {
        return refuse(
            "input.day_count_invalid",
            format!(
                "non_resident_days of {days_in_year} is the whole year: ITAA 1936 s 251S(1)(a) levies the Medicare levy only on a person resident at some time in the year, so a person not resident at any time pays none (see foreign-resident-tax)."
            ),
        );
    }
    day_count(value, days_in_year - 1, "non_resident_days")
}

/// The days in an income year such as "2025-26": 366 when it holds 29 February.
pub(crate) fn income_year_days(year: &str) -> i64 {
    let start: i64 = year[..4].parse().expect("income year");
    if is_leap(start + 1) { 366 } else { 365 }
}

/// The exemption days, or `None` for none. A day can be counted once: the ATO's
/// M1 instructions count a day in both a full and a half period as a full day.
fn medicare_exemption(exemption: &MedicareExemption, year: &str) -> Result<Option<Exemption>> {
    let days_in_year = income_year_days(year);
    let count =
        |value: Option<&Arg>, name| value.map_or(Ok(0), |v| day_count(v, days_in_year, name));
    let full = count(exemption.full_days, "medicare_levy_exemption_full_days")?;
    let half = count(exemption.half_days, "medicare_levy_exemption_half_days")?;
    let non_resident = non_resident_days(exemption.non_resident_days, days_in_year)?;
    if non_resident > 0 && full + half + non_resident > days_in_year {
        return refuse(
            "input.day_count_invalid",
            format!(
                "medicare_levy_exemption_full_days, medicare_levy_exemption_half_days and non_resident_days together must be no more than {days_in_year}; count each day once, a non-resident day as non-resident only."
            ),
        );
    }
    if full + half > days_in_year {
        return refuse(
            "input.day_count_invalid",
            format!(
                "medicare_levy_exemption_full_days and medicare_levy_exemption_half_days together \
                 must be no more than {days_in_year}; count a day in both as a full exemption day."
            ),
        );
    }
    Ok((full + half + non_resident > 0).then_some(Exemption {
        full,
        half,
        non_resident,
        days_in_year,
    }))
}

/// s 9(1): `levy` (times `scale`, see `family_reductions`) less the share of it
/// for the days the person was a prescribed person: each full exemption day and
/// non-resident day, and half of each half exemption day (ITAA 1936 s 251U(3)).
/// It applies to the levy after ss 7 and 8, which each work out the levy 'but
/// for' s 9.
fn levy_after_exemption(levy: &Decimal, scale: &Decimal, exemption: &Exemption) -> Calc<Decimal> {
    let halves = int(2 * exemption.days_in_year);
    let prescribed_halves = 2 * (exemption.full + exemption.non_resident) + exemption.half;
    let kept = halves.sub(&int(prescribed_halves))?;
    levy.mul(&kept)?.div(&scale.mul(&halves)?)
}

fn optional_flag(value: Option<&Arg>, name: &str) -> Result<bool> {
    value.map_or(Ok(false), |v| flag(v, name))
}

fn not_taken<T>(command: &str, name: &str, when: &str) -> Result<T> {
    refuse(
        "input.field_unknown",
        format!("{name} is not an input of {command} {when}"),
    )
}

fn medicare_spouse(family: &MedicareFamily) -> Result<Option<Spouse>> {
    let Some(income) = family.spouse_taxable_income else {
        if family.spouse_sapto_entitled.is_some() {
            return not_taken(
                "medicare-levy",
                "spouse_sapto_entitled",
                "without spouse_taxable_income",
            );
        }
        return Ok(None);
    };
    let income = money(income)?;
    whole_dollars(&income, "spouse taxable income")?;
    if family.dependant_offset_entitled.is_some() {
        return not_taken(
            "medicare-levy",
            "dependant_offset_entitled",
            "with spouse_taxable_income",
        );
    }
    let Some(sapto) = family.spouse_sapto_entitled else {
        return refuse(
            "input.missing_field",
            "spouse_sapto_entitled is required with spouse_taxable_income",
        );
    };
    let sapto = flag(sapto, "spouse_sapto_entitled")?;
    Ok(Some(Spouse { income, sapto }))
}

/// An optional count of dependent children, 0 when not given.
fn dependent_children(value: Option<&Arg>) -> Result<i64> {
    match value.map(Arg::int) {
        None => Ok(0),
        Some(Some(Some(n))) if n >= 0 => Ok(n),
        Some(_) => refuse(
            "input.dependants_invalid",
            "dependent_children must be a non-negative integer.",
        ),
    }
}

fn medicare_family(family: &MedicareFamily) -> Result<Family> {
    let sapto = optional_flag(family.sapto_entitled, "sapto_entitled")?;
    let spouse = medicare_spouse(family)?;
    let dependant_offset = optional_flag(
        family.dependant_offset_entitled,
        "dependant_offset_entitled",
    )?;
    let children = dependent_children(family.dependent_children)?;
    // Without a spouse the children raise a threshold only s 8(1)(b) or (c) brings in.
    if children > 0 && spouse.is_none() && !dependant_offset {
        return not_taken(
            "medicare-levy",
            "dependent_children",
            "without a spouse unless dependant_offset_entitled is true",
        );
    }
    Ok(Family {
        sapto,
        spouse,
        dependant_offset,
        children,
    })
}

fn medicare_rate(key: &str) -> Decimal {
    d(lookup(&meta::MEDICARE_LEVY_RATES, key))
}

/// s 3(1): the threshold amount and phase-in limit.
fn medicare_thresholds(year: &str, sapto: bool) -> (Decimal, Decimal) {
    let table = if sapto {
        meta::MEDICARE_SAPTO_THRESHOLDS
    } else {
        meta::MEDICARE_LOW_INCOME_THRESHOLDS
    };
    let (threshold, limit) = lookup(table, year);
    (d(threshold), d(limit))
}

/// One person's names in a `medicare-levy` formula: their taxable income, the
/// levy on it at the full rate, and their s 3(1) threshold amount.
type LevyNames<'a> = (&'a str, &'a str, &'a str);

/// ss 6(1) and 7: the levy on `income` but for ss 8 and 9, its formula on
/// `names`, and the provision that sets the low-income reduction. The Medicare
/// Levy Act 1986 steps cite compilation 53 (1 July 2026), read on 15 November
/// 2026, whose s 8(2) formula is an image read from its alternative text.
fn low_income_levy(
    income: &Decimal,
    year: &str,
    sapto: bool,
    (i, full_name, t): LevyNames,
) -> Calc<(Decimal, String, &'static str)> {
    let (threshold, limit) = medicare_thresholds(year, sapto);
    let full = income.mul(&medicare_rate("levy"))?;
    Ok(if income.le(&threshold)? {
        (
            int(0),
            "0".to_string(),
            "Medicare Levy Act 1986 s 7(1): no levy at or below the threshold amount (s 3(1)), so the whole levy",
        )
    } else if income.le(&limit)? {
        (
            py_min(
                full,
                income.sub(&threshold)?.mul(&medicare_rate("shade_in"))?,
            )?,
            format!("min({full_name}, ({i} - {t}) x shade_in_rate)"),
            "Medicare Levy Act 1986 s 7(2): up to the phase-in limit, the levy is no more than 10% of the excess over the threshold amount (s 3(1))",
        )
    } else {
        (
            full,
            full_name.to_string(),
            "Medicare Levy Act 1986 ss 6(1) and 7: above the phase-in limit (s 3(1)), none",
        )
    })
}

/// s 8(5) to (7): the family income threshold.
fn family_threshold(year: &str, sapto: bool, children: i64) -> Calc<Decimal> {
    let (base, sapto_base, per_child) = lookup(meta::MEDICARE_FAMILY_THRESHOLDS, year);
    d(if sapto { sapto_base } else { base }).add(&int(children).mul(&d(per_child))?)
}

/// The s 8(2) formula on the family income threshold `threshold`, written `t`,
/// nil where it is not positive.
fn family_formula(
    threshold: &Decimal,
    family_income: &Decimal,
    t: &str,
) -> Calc<(Decimal, String)> {
    let start = medicare_rate("levy").mul(threshold)?;
    let excess = family_income.sub(threshold)?;
    let reduction = py_max(
        int(0),
        start.sub(&medicare_rate("family_shade_out").mul(&excess)?)?,
    )?;
    let formula =
        format!("max(0, levy_rate x {t} - family_shade_out_rate x (family_income - {t}))");
    Ok((reduction, formula))
}

/// s 8(4): the part of the spouse's shared reduction above the spouse's levy,
/// which reduces this person's levy, times family income (see
/// `family_reductions`), and its formula. Only a spouse liable to the levy has
/// one, and s 8(3) shares it only when this person is liable too; otherwise
/// this person's own reduction was not shared, and nothing passes to them. The
/// spouse's thresholds are written in as figures.
fn spouse_excess_scaled(
    spouse: &Spouse,
    year: &str,
    children: i64,
    family_income: &Decimal,
) -> Calc<(Decimal, String)> {
    let threshold = family_threshold(year, spouse.sapto, children)?;
    if family_income.le(&threshold)? {
        return Ok((int(0), "0".to_string()));
    }
    let (reduction, r) = family_formula(&threshold, family_income, &threshold.to_string())?;
    let share = reduction.mul(&spouse.income)?;
    let spouse_threshold = medicare_thresholds(year, spouse.sapto).0.to_string();
    let names = (
        "spouse_taxable_income",
        "spouse_taxable_income x levy_rate",
        spouse_threshold.as_str(),
    );
    let (levy, l, _) = low_income_levy(&spouse.income, year, spouse.sapto, names)?;
    let excess = py_max(int(0), share.sub(&levy.mul(family_income)?)?)?;
    let formula = format!(
        "max(0, {r} x spouse_taxable_income - {} x family_income)",
        group(&l)
    );
    Ok((excess, formula))
}

const FAMILY_INCOME: &str = "Medicare Levy Act 1986 s 8(5), family income: the person's taxable income and, for a married person, the spouse's";
const NOT_ABOVE: &str = "Medicare Levy Act 1986 s 8(1): no levy where family income does not exceed the family income threshold (s 8(5) to (7)), so the whole levy";
const OWN_REDUCTION: &str = "Medicare Levy Act 1986 s 8(2): the levy is reduced by 2% of the family income threshold less 0.08 of the excess over it, up to the levy";
const SHARED_REDUCTION: &str = "Medicare Levy Act 1986 s 8(2) and (3): the s 8(2) reduction in the person's share of family income, as the spouse is liable to the levy, up to the levy";
const SPOUSE_EXCESS: &str = "Medicare Levy Act 1986 s 8(4): the part of the spouse's s 8(2) and (3) reduction above the spouse's levy reduces this levy";
const NO_SPOUSE_EXCESS: &str = "Medicare Levy Act 1986 s 8(3) and (4): only a spouse liable to the levy, whose reduction s 8(3) shares because this person is liable too, passes on an excess; not here, so nil";

/// The s 8 working for one person.
struct FamilyWorking {
    /// The steps for family income, this person's own reduction and, with a
    /// spouse, the spouse's excess reduction.
    steps: Vec<Step>,
    threshold: Decimal,
    /// When s 8(3) shares the reduction: the levy after s 8 times family
    /// income, and its formula. Otherwise the steps' values give the levy.
    scaled: Option<(Decimal, String)>,
    /// Family income when s 8(3) shares the reduction, otherwise 1.
    scale: Decimal,
    /// Whether s 8(3) shared this person's reduction.
    shared: bool,
}

/// s 8 on `levy`, the levy on `income` but for ss 8 and 9, written `l`.
/// `values` holds the names the formulas use and gains family income.
fn family_reductions(
    income: &Decimal,
    (levy, l): (&Decimal, &str),
    year: &str,
    family: &Family,
    values: &mut Vec<(&'static str, Decimal)>,
) -> Calc<FamilyWorking> {
    let (family_income, formula) = match &family.spouse {
        Some(spouse) => (
            income.add(&spouse.income)?,
            "taxable_income_used + spouse_taxable_income",
        ),
        None => (income.clone(), "taxable_income_used"),
    };
    values.push(("family_income", family_income.clone()));
    let mut steps = vec![step_by_name(
        "family_income",
        FAMILY_INCOME,
        formula,
        values,
        family_income.clone(),
    )];
    let threshold = family_threshold(year, family.sapto, family.children)?;
    values.push(("family_income_threshold", threshold.clone()));
    values.push(("family_shade_out_rate", medicare_rate("family_shade_out")));
    let liable = |amount: &Decimal, sapto: bool| amount.gt(&medicare_thresholds(year, sapto).0);
    let spouse_liable = match &family.spouse {
        Some(spouse) => liable(&spouse.income, spouse.sapto)?,
        None => false,
    };
    // When s 8(3) shares the reduction, every amount is worked times family
    // income, so each share is a product, and divided once at the end: the
    // levy is then exact wherever it terminates, and a half cent rounds up.
    let (scale, lx) = if spouse_liable {
        (
            family_income.clone(),
            format!("{} x family_income", group(l)),
        )
    } else {
        (int(1), l.to_string())
    };
    let levy = levy.mul(&scale)?;
    // s 8(1): no levy at or below the threshold.
    let above = family_income.gt(&threshold)?;
    let (own, own_formula, authority) = if above {
        let (reduction, r) = family_formula(&threshold, &family_income, "family_income_threshold")?;
        if spouse_liable {
            (
                py_min(levy.clone(), reduction.mul(income)?)?,
                format!("min({lx}, {r} x taxable_income_used)"),
                SHARED_REDUCTION,
            )
        } else {
            (
                py_min(levy.clone(), reduction)?,
                format!("min({lx}, {r})"),
                OWN_REDUCTION,
            )
        }
    } else {
        (levy.clone(), lx.clone(), NOT_ABOVE)
    };
    let per = |formula: &str| {
        if spouse_liable {
            format!("{} / family_income", group(formula))
        } else {
            formula.to_string()
        }
    };
    steps.push(step_by_name(
        "family_reduction",
        authority,
        &per(&own_formula),
        values,
        own.div(&scale)?,
    ));
    let mut after = levy.sub(&own)?;
    let mut after_formula = format!("{lx} - {}", group(&own_formula));
    if let Some(spouse) = &family.spouse {
        if spouse_liable && liable(income, family.sapto)? {
            let (excess, e) = spouse_excess_scaled(spouse, year, family.children, &family_income)?;
            let from = py_min(after.clone(), excess)?;
            let from_formula = format!("min({after_formula}, {e})");
            steps.push(step_by_name(
                "spouse_excess_reduction",
                SPOUSE_EXCESS,
                &per(&from_formula),
                values,
                from.div(&scale)?,
            ));
            after = after.sub(&from)?;
            after_formula = format!("{after_formula} - {from_formula}");
        } else {
            steps.push(step(
                "spouse_excess_reduction",
                NO_SPOUSE_EXCESS,
                "0",
                &[],
                int(0),
            ));
        }
    }
    Ok(FamilyWorking {
        steps,
        threshold,
        scaled: spouse_liable.then_some((after, after_formula)),
        scale,
        shared: spouse_liable && above,
    })
}

/// s 9A: the worksheet does not adjust for lump sums in arrears.
fn refuse_section_9a(section_9a_adjustment_applies: &Arg) -> Result<()> {
    let Arg::Bool(adjusted) = section_9a_adjustment_applies else {
        return refuse(
            "input.flag_invalid",
            "section_9a_adjustment_applies must be a boolean.",
        );
    };
    if *adjusted {
        let extra = texts::EXTRA_SOURCES
            .iter()
            .find(|(k, _)| *k == "medicare_levy")
            .expect("medicare sources")
            .1[2];
        return refuse(
            "medicare.s9a_adjustment_out_of_scope",
            format!(
                "Medicare Levy Act 1986 s 9A (lump sum payments in arrears) applies, and this \
             worksheet does not make that adjustment. See {extra}"
            ),
        );
    }
    Ok(())
}

/// The steps, the rates and whether s 8(3) shared the family reduction.
type MedicareWorking = (Vec<Step>, Vec<(String, String)>, bool);

fn medicare_working(
    income: &Decimal,
    year: &str,
    family: &Family,
    exemption: Option<&Exemption>,
) -> Calc<MedicareWorking> {
    let (threshold, limit) = medicare_thresholds(year, family.sapto);
    let mut rates = vec![
        rate("levy_rate", medicare_rate("levy")),
        rate("shade_in_rate", medicare_rate("shade_in")),
        rate("threshold_amount", &threshold),
        rate("phase_in_limit", &limit),
    ];
    let full = income.mul(&medicare_rate("levy"))?;
    let names = (
        "taxable_income_used",
        "levy_at_full_rate",
        "threshold_amount",
    );
    let (levy, l, authority) = low_income_levy(income, year, family.sapto, names)?;
    let reduction = full.sub(&levy)?;
    let mut values = vec![
        ("taxable_income", income.clone()),
        ("taxable_income_used", income.clone()),
        ("levy_rate", medicare_rate("levy")),
        ("shade_in_rate", medicare_rate("shade_in")),
        ("threshold_amount", threshold),
        ("levy_at_full_rate", full.clone()),
        ("low_income_reduction", reduction.clone()),
    ];
    if let Some(spouse) = &family.spouse {
        values.push(("spouse_taxable_income", spouse.income.clone()));
    }
    let mut steps = vec![
        step(
            "taxable_income_used",
            "Medicare Levy Act 1986 s 6(1): the levy is on taxable income, as established",
            "taxable_income",
            &[income],
            income.clone(),
        ),
        step(
            "levy_at_full_rate",
            "Medicare Levy Act 1986 s 6(1): 2%",
            "taxable_income_used x levy_rate",
            &[income, &medicare_rate("levy")],
            full.clone(),
        ),
        step_by_name(
            "low_income_reduction",
            authority,
            &format!("levy_at_full_rate - {}", group(&l)),
            &values,
            reduction.clone(),
        ),
    ];
    // The levy before s 9 and its formula; when s 8(3) shares the family
    // reduction, both are times family income, the `scale`.
    let mut before = full.sub(&reduction)?;
    let mut before_formula = "levy_at_full_rate - low_income_reduction".to_string();
    let (mut shared, mut scale, mut times_family_income) = (false, int(1), false);
    if family.reduces() {
        let working = family_reductions(income, (&levy, &l), year, family, &mut values)?;
        if let Some(scaled) = working.scaled {
            (before, before_formula) = scaled;
            times_family_income = true;
        } else {
            // Unshared, each reduction is exact, so it comes off the levy by name.
            for reduction in &working.steps[1..] {
                before = before.sub(&reduction.value)?;
                before_formula = format!("{before_formula} - {}", reduction.id);
                values.push((reduction.static_id(), reduction.value.clone()));
            }
        }
        rates.push(rate("family_income_threshold", &working.threshold));
        rates.push(rate(
            "family_shade_out_rate",
            medicare_rate("family_shade_out"),
        ));
        (shared, scale) = (working.shared, working.scale);
        steps.extend(working.steps);
    }
    let scaled = |formula: &str| {
        if times_family_income {
            format!("({formula}) / family_income")
        } else {
            formula.to_string()
        }
    };
    let Some(exemption) = exemption else {
        steps.push(step_by_name(
            "medicare_levy",
            if times_family_income {
                "Medicare Levy Act 1986 ss 6 to 8: the levy after the low-income and family reductions, worked out times family income for s 8(3) and divided by it once"
            } else if family.reduces() {
                "Medicare Levy Act 1986 ss 6 to 8: the levy after the low-income and family reductions"
            } else {
                "Medicare Levy Act 1986 ss 6 and 7: the levy after the low-income reduction"
            },
            &scaled(&before_formula),
            &values,
            before.div(&scale)?,
        ));
        return Ok((steps, rates, shared));
    };
    let levy = (
        &before,
        &scale,
        before_formula.as_str(),
        times_family_income,
    );
    steps.extend(exemption_working(levy, exemption, values, &mut rates)?);
    Ok((steps, rates, shared))
}

const EXEMPTION_AUTHORITY: &str = "Medicare Levy Act 1986 s 9(1) and ITAA 1936 s 251U(3): the share of the levy for each full exemption day and half of each half exemption day";
const NON_RESIDENT_AUTHORITY: &str = "Medicare Levy Act 1986 s 9(1) and ITAA 1936 s 251U(1)(d), (2) and (3): the share of the levy for each full exemption day, each non-resident day on which every dependant was also a prescribed person (a whole day: s 251U(3) gives no half under para (d)) and half of each half exemption day";

/// The exemption's day counts as rates and values, in the order the result has
/// always given them, and the full days' term and authority: with non-resident
/// days, both name them.
fn exemption_facts(
    exemption: &Exemption,
    values: &mut Vec<(&str, Decimal)>,
    rates: &mut Vec<(String, String)>,
) -> (&'static str, &'static str) {
    let mut days = vec![
        ("exemption_full_days", exemption.full),
        ("exemption_half_days", exemption.half),
    ];
    let named = if exemption.non_resident > 0 {
        days.push(("non_resident_days", exemption.non_resident));
        (
            "(exemption_full_days + non_resident_days)",
            NON_RESIDENT_AUTHORITY,
        )
    } else {
        ("exemption_full_days", EXEMPTION_AUTHORITY)
    };
    days.push(("days_in_year", exemption.days_in_year));
    for (key, n) in days {
        rates.push(rate(key, n));
        values.push((key, int(n)));
    }
    named
}

/// The s 9(1) steps: the share of the levy before s 9 (`levy`: the levy times
/// `scale` and its formula, times family income where the last is true) for
/// the prescribed days, and the levy left.
fn exemption_working(
    levy: (&Decimal, &Decimal, &str, bool),
    exemption: &Exemption,
    mut values: Vec<(&str, Decimal)>,
    rates: &mut Vec<(String, String)>,
) -> Calc<Vec<Step>> {
    let (before, scale, before_formula, times_family_income) = levy;
    let (full_days, authority) = exemption_facts(exemption, &mut values, rates);
    let halves = if times_family_income {
        "family_income x 2 x days_in_year"
    } else {
        "2 x days_in_year"
    };
    let after = levy_after_exemption(before, scale, exemption)?;
    let scaled = if times_family_income {
        format!("({before_formula}) / family_income")
    } else {
        before_formula.to_string()
    };
    let after_formula = format!(
        "({before_formula}) x (2 x days_in_year - (2 x {full_days} + exemption_half_days)) / ({halves})"
    );
    Ok(vec![
        step_by_name(
            "exemption_reduction",
            authority,
            &format!("{scaled} - {after_formula}"),
            &values,
            before.div(scale)?.sub(&after)?,
        ),
        step_by_name(
            "medicare_levy",
            "Medicare Levy Act 1986 s 9(1): the levy after ss 7 and 8 for the days the person was not a prescribed person",
            &after_formula,
            &values,
            after,
        ),
    ])
}

pub fn medicare_levy(
    taxable_income: &Arg,
    section_9a_adjustment_applies: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    family: MedicareFamily,
    exemption: MedicareExemption,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "medicare_levy")?;
    let income = money(taxable_income)?;
    whole_dollars(&income, "taxable income")?;
    refuse_section_9a(section_9a_adjustment_applies)?;
    let family = medicare_family(&family)?;
    let exemption = medicare_exemption(&exemption, &year)?;
    let (steps, rates, shared) = precise(|| {
        Ok(medicare_working(
            &income,
            &year,
            &family,
            exemption.as_ref(),
        )?)
    })?;
    let mut out = derived_result("medicare_levy", &year, steps, rates)?;
    year_warning(&mut out, texts::MEDICARE_YEAR_WARNINGS, &year);
    if shared {
        let warnings = out["warnings"].as_array_mut().expect("warnings");
        warnings.push(json!(texts::MEDICARE_SHARED_REDUCTION_NOTE));
    }
    Ok(out)
}

const SURCHARGE: &str = "medicare_levy_surcharge";

/// The optional `medicare-levy-surcharge` inputs, each `None` when not given.
pub struct SurchargeFamily<'a> {
    pub spouse_income: Option<&'a Arg>,
    pub dependent_children: Option<&'a Arg>,
}

/// The facts a surcharge is worked from.
struct SurchargeFacts {
    taxable: Decimal,
    fringe: Decimal,
    /// The person's income for surcharge purposes.
    own: Decimal,
    days: i64,
    days_in_year: i64,
    /// For a person married for the whole year: the spouse's income for
    /// surcharge purposes and the MLS dependent children.
    spouse: Option<(Decimal, i64)>,
}

/// The spouse and children, or `None` for a person without either all year.
fn surcharge_spouse(family: &SurchargeFamily) -> Result<Option<(Decimal, i64)>> {
    let children = dependent_children(family.dependent_children)?;
    let Some(spouse) = family.spouse_income else {
        // s 8C, a person with dependants but no spouse, has no published example.
        if children > 0 {
            return not_taken(
                "medicare-levy-surcharge",
                "dependent_children",
                "without spouse_income_for_surcharge_purposes",
            );
        }
        return Ok(None);
    };
    Ok(Some((money(spouse)?, children)))
}

/// The tier 1, 2 and 3 thresholds: the singles thresholds, or for a family
/// double them (Private Health Insurance Act 2007 s 22-40), raised for each
/// dependent child after the first.
fn surcharge_thresholds(year: &str, family: Option<i64>) -> Calc<Vec<Decimal>> {
    let singles = lookup(meta::SURCHARGE_THRESHOLDS, year);
    let raise = |threshold: &&str| match family {
        None => Ok(d(threshold)),
        Some(children) => {
            let extra = int((children - 1).max(0)).mul(&d(meta::SURCHARGE_CHILD_INCREASE))?;
            d(threshold).mul(&int(2))?.add(&extra)
        }
    };
    singles.iter().map(raise).collect()
}

type SurchargeWorking = (Vec<Step>, Vec<(String, String)>);

/// Medicare Levy Act ss 8B and 8D and the Fringe Benefits Act ss 12, 15 and
/// 16: the tier is set by income for surcharge purposes (combined with a
/// spouse's), and its rate applies to taxable income and reportable fringe
/// benefits for the days the section applies.
fn surcharge_working(facts: &SurchargeFacts, year: &str) -> Calc<SurchargeWorking> {
    let (income, thresholds) = match &facts.spouse {
        None => (facts.own.clone(), surcharge_thresholds(year, None)?),
        Some((spouse, children)) => (
            facts.own.add(spouse)?,
            surcharge_thresholds(year, Some(*children))?,
        ),
    };
    let mut tier = 0;
    for threshold in &thresholds {
        tier += usize::from(income.gt(threshold)?);
    }
    // s 8D(3)(c): a married person's own income must also exceed this amount,
    // the same figure as the s 3(1) threshold amount (c) in each year.
    let floor = facts
        .spouse
        .as_ref()
        .map(|_| medicare_thresholds(year, false).0);
    let liable = tier > 0 && floor.as_ref().map_or(Ok(true), |f| facts.own.gt(f))?;
    let percentage = if liable {
        d(meta::SURCHARGE_RATES[tier - 1])
    } else {
        int(0)
    };
    let base = facts.taxable.add(&facts.fringe)?;
    let surcharge = base.mul(&percentage)?.mul(&int(facts.days))?;
    let (days, days_in_year) = (int(facts.days), int(facts.days_in_year));
    let mut steps = vec![step(
        "income_for_surcharge_purposes",
        "Medicare Levy Act 1986 s 3(1), income for surcharge purposes: the person's, as established",
        "income_for_surcharge_purposes",
        &[&facts.own],
        facts.own.clone(),
    )];
    let mut rates = vec![rate("tier", ["base", "1", "2", "3"][tier])];
    if let Some((spouse, _)) = &facts.spouse {
        steps.push(step(
            "family_income_for_surcharge_purposes",
            "Medicare Levy Act 1986 s 8D(3)(b) and (5); Fringe Benefits Act s 15(1)(b): the person's and the spouse's income for surcharge purposes",
            "income_for_surcharge_purposes + spouse_income_for_surcharge_purposes",
            &[&facts.own, spouse],
            income,
        ));
    }
    steps.push(step(
        "surcharge_base",
        "Medicare Levy Act 1986 ss 8B(2) and 8D(3) and (4): taxable income; Fringe Benefits Act ss 12(2) and (3), 15(1) and 16(4): the reportable fringe benefits total",
        "taxable_income + reportable_fringe_benefits",
        &[&facts.taxable, &facts.fringe],
        base.clone(),
    ));
    steps.push(step(
        "medicare_levy_surcharge",
        surcharge_authority(facts.spouse.is_some(), tier, liable),
        "surcharge_base x surcharge_rate x days_without_cover / days_in_year",
        &[&base, &percentage, &days, &days_in_year],
        surcharge.div(&days_in_year)?,
    ));
    rates.push(rate("surcharge_rate", percentage));
    for (key, threshold) in ["tier_1_threshold", "tier_2_threshold", "tier_3_threshold"]
        .iter()
        .zip(&thresholds)
    {
        rates.push(rate(key, threshold));
    }
    if let Some(floor) = floor {
        rates.push(rate("own_income_threshold", floor));
    }
    rates.push(rate("days_without_cover", facts.days));
    rates.push(rate("days_in_year", facts.days_in_year));
    Ok((steps, rates))
}

/// The provisions that set the surcharge's rate for the days without cover,
/// from the Medicare Levy Act 1986 compilation 53 and A New Tax System
/// (Medicare Levy Surcharge—Fringe Benefits) Act 1999 (the Fringe Benefits Act)
/// compilation 42, both of 1 July 2026, read on 15 November 2026. A tier 2 or 3
/// earner is one under s 3A.
fn surcharge_authority(married: bool, tier: usize, liable: bool) -> &'static str {
    match (married, tier, liable) {
        (false, 0, _) => {
            "Medicare Levy Act 1986 s 8B(2); Fringe Benefits Act s 12(1)(a): income for surcharge purposes does not exceed the singles tier 1 threshold, so nil"
        }
        (false, ..) => {
            "Medicare Levy Act 1986 s 8B(2) to (4) and s 3A; Fringe Benefits Act s 12(2) to (5): the tier's rate on taxable income and reportable fringe benefits, for the days without cover as a share of the year"
        }
        (true, 0, _) => {
            "Medicare Levy Act 1986 s 8D(3)(b) and (4)(a)(i); Fringe Benefits Act ss 15(1)(b) and 16(2)(b): family income for surcharge purposes does not exceed the family tier 1 threshold, so nil"
        }
        (true, _, false) => {
            "Medicare Levy Act 1986 s 8D(3)(c) and (4)(a)(ii); Fringe Benefits Act ss 15(1)(c) and 16(2)(c): the person's own income for surcharge purposes does not exceed the s 8D(3)(c) amount, so nil"
        }
        (true, _, true) => {
            "Medicare Levy Act 1986 s 8D(3), (4), (4A) and (4B) and s 3A; Fringe Benefits Act ss 15 and 16: the tier's rate on taxable income and reportable fringe benefits, for the days without cover as a share of the year"
        }
    }
}

/// The Medicare levy surcharge on one person's taxable income (Medicare Levy
/// Act 1986 ss 8B and 8D) and reportable fringe benefits (A New Tax System
/// (Medicare Levy Surcharge - Fringe Benefits) Act 1999 ss 12, 15 and 16).
pub fn medicare_levy_surcharge(
    taxable_income: &Arg,
    reportable_fringe_benefits: &Arg,
    income_for_surcharge_purposes: &Arg,
    days_without_cover: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    family: SurchargeFamily,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, SURCHARGE)?;
    let taxable = money(taxable_income)?;
    whole_dollars(&taxable, "taxable income")?;
    let days_in_year = income_year_days(&year);
    let facts = SurchargeFacts {
        taxable,
        fringe: money(reportable_fringe_benefits)?,
        own: money(income_for_surcharge_purposes)?,
        days: day_count(days_without_cover, days_in_year, "days_without_cover")?,
        days_in_year,
        spouse: surcharge_spouse(&family)?,
    };
    if !meta::SURCHARGE_THRESHOLDS.iter().any(|(y, _)| *y == year) {
        return surcharge_without_thresholds(&facts, &year);
    }
    let (steps, rates) = precise(|| Ok(surcharge_working(&facts, &year)?))?;
    derived_result(SURCHARGE, &year, steps, rates)
}

/// A year whose thresholds are not yet published: the surcharge is nil when
/// no day was without cover, since Medicare Levy Act 1986 ss 8B, 8C and 8D and
/// the Fringe Benefits Act ss 12 to 14 apply only to a period without private
/// patient hospital cover; any day without cover needs the thresholds.
fn surcharge_without_thresholds(facts: &SurchargeFacts, year: &str) -> Result<Value> {
    if facts.days > 0 {
        let why = texts::UNPUBLISHED_2027_28
            .iter()
            .find(|(kind, _)| *kind == SURCHARGE)
            .map_or("", |(_, why)| why);
        return refuse(
            "period.unsupported",
            format!(
                "{year} is supported only with days_without_cover 0, when no surcharge applies. {why}"
            ),
        );
    }
    let steps = vec![step(
        "medicare_levy_surcharge",
        "Medicare Levy Act 1986 ss 8B(1)(c), 8C(1)(c) and 8D(1)(b); Fringe Benefits Act ss 12(1)(b)(iii), 13(1)(b)(iii) and 14(1)(b): each applies only for a period without private patient hospital cover, so with no day without cover there is no surcharge, whatever the year's thresholds",
        "0",
        &[],
        int(0),
    )];
    let rates = vec![
        rate("days_without_cover", facts.days),
        rate("days_in_year", facts.days_in_year),
    ];
    derived_result(SURCHARGE, year, steps, rates)
}

pub fn low_income_tax_offset(
    taxable_income: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "low_income_tax_offset")?;
    let income = money(taxable_income)?;
    whole_dollars(&income, "taxable income")?;
    let rule = |key: &str| d(lookup(&meta::LOW_INCOME_TAX_OFFSET, key));
    // ITAA 1997 compilation 268 (1 October 2026), read on 15 November 2026.
    let (authority, formula, offset) = precise(|| {
        Ok(if income.le(&rule("first_step"))? {
            (
                "ITAA 1997 s 61-115(1), table item 1",
                "maximum",
                rule("maximum"),
            )
        } else if income.le(&rule("second_step"))? {
            (
                "ITAA 1997 s 61-115(1), table item 2: $700 less 5% of the excess over $37,500",
                "maximum - (taxable_income_used - first_step) x first_rate",
                rule("maximum").sub(&income.sub(&rule("first_step"))?.mul(&rule("first_rate"))?)?,
            )
        } else if income.le(&rule("cut_out"))? {
            // At the cut-out the table gives minus half a cent; the offset is nil.
            (
                "ITAA 1997 s 61-115(1), table item 3: $325 less 1.5% of the excess over $45,000, nil where that is negative",
                "max(0, second_amount - (taxable_income_used - second_step) x second_rate)",
                py_max(
                    int(0),
                    rule("second_amount").sub(
                        &income
                            .sub(&rule("second_step"))?
                            .mul(&rule("second_rate"))?,
                    )?,
                )?,
            )
        } else {
            (
                "ITAA 1997 s 61-110(1)(b): no offset where taxable income exceeds $66,667",
                "0",
                int(0),
            )
        })
    })?;
    let mut values = vec![("taxable_income_used", income.clone())];
    values.extend(meta::LOW_INCOME_TAX_OFFSET.iter().map(|(k, v)| (*k, d(v))));
    let steps = vec![
        step(
            "taxable_income_used",
            "ITAA 1997 s 61-115(1)(a): an individual's relevant income is their taxable income",
            "taxable_income",
            &[&income],
            income.clone(),
        ),
        step_by_name("low_income_tax_offset", authority, formula, &values, offset),
    ];
    derived_result(
        "low_income_tax_offset",
        &year,
        steps,
        meta::LOW_INCOME_TAX_OFFSET
            .iter()
            .map(|(k, v)| rate(k, v))
            .collect(),
    )
}

/// `division_293`'s amounts: (id, authority, formula). The lesser of the low tax
/// contributions and the excess is nil when they are, as s 293-20(2) requires.
const DIVISION_293_STEPS: [(&str, &str, &str); 4] = [
    (
        "combined_income_and_contributions",
        "ITAA 1997 s 293-20(1)(a) and (b)",
        "division_293_income + low_tax_contributions",
    ),
    (
        "excess_over_threshold",
        "ITAA 1997 s 293-20(1)",
        "max(0, combined_income_and_contributions - threshold)",
    ),
    (
        "taxable_contributions",
        "ITAA 1997 s 293-20(1) and (2)",
        "min(low_tax_contributions, excess_over_threshold)",
    ),
    (
        "division_293_tax",
        "Superannuation (Sustaining the Superannuation Contribution Concession) Imposition Act 2013 s 5",
        "taxable_contributions x rate",
    ),
];

pub fn division_293(
    division_293_income: &Arg,
    low_tax_contributions: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "division_293")?;
    let income = money(division_293_income)?;
    let contributions = money(low_tax_contributions)?;
    let threshold = d(lookup(&meta::DIVISION_293, "threshold"));
    let rate_ = d(lookup(&meta::DIVISION_293, "rate"));
    let steps = precise(|| {
        let combined = income.add(&contributions)?;
        let excess = py_max(int(0), combined.sub(&threshold)?)?;
        let taxable = py_min(contributions.clone(), excess.clone())?;
        let tax = taxable.mul(&rate_)?;
        Ok(steps_from(
            &DIVISION_293_STEPS,
            [
                (&[&income, &contributions], combined.clone()),
                (&[&combined, &threshold], excess.clone()),
                (&[&contributions, &excess], taxable.clone()),
                (&[&taxable, &rate_], tax),
            ],
        ))
    })?;
    derived_result(
        "division_293",
        &year,
        steps,
        vec![rate("threshold", &threshold), rate("rate", &rate_)],
    )
}

/// A finite Decimal of at most 2dp and magnitude 1e12 that may be negative:
/// total superannuation earnings can be (Division 296 regulations, reg 296-65.02 note).
pub(crate) fn signed_money(value: &Arg) -> Result<Decimal> {
    let bad = || {
        refuse(
            "input.signed_amount_invalid",
            "Amounts that may be negative must be finite Decimals, at most 2dp and 1e12 either side of nil.",
        )
    };
    let Arg::Decimal(v) = value else {
        return bad();
    };
    if !v.is_finite() || v.lt(&d("-1e12"))? || v.gt(&d("1e12"))? || v.exponent() < -2 {
        return bad();
    }
    Ok(v.clone())
}

/// The share of `balance` above `threshold` as a percentage rounded to 2 decimal
/// places, half up (ITAA 1997 ss 296-40(2) and (3), 296-45(2) and (3)); nil when
/// the balance is not greater than the threshold.
fn percentage_above(balance: &Decimal, threshold: &Decimal) -> Result<Decimal> {
    if !balance.gt(threshold)? {
        return Ok(d("0.00"));
    }
    precise(|| {
        let share = balance.sub(threshold)?.div(balance)?.mul(&int(100))?;
        Ok(share.quantize(-2, Rounding::HalfUp)?)
    })
}

/// Exceptions in order, each with the reason given when it applies.
const DIVISION_296_EXCEPTIONS: [(&str, &str); 3] = [
    (
        "child_recipient",
        "not liable: a child recipient of a superannuation income stream in the year (s 296-20)",
    ),
    (
        "structured_settlement_contribution",
        "not liable: a structured settlement contribution was made in respect of the individual in or before the year (s 296-25)",
    ),
    (
        "died_in_year",
        "not liable: died on or before the last day of the 2026-27 income year (Income Tax (Transitional Provisions) Act 1997 s 296-1(3))",
    ),
];

/// The reason of the first exception that applies, if any, and whether the
/// individual died by the end of the year (s 296-50), once every flag is valid.
fn division_296_exceptions(exceptions: [&Arg; 3]) -> Result<(Option<&'static str>, bool)> {
    let mut excepted = None;
    let mut died = false;
    for (value, (name, reason)) in exceptions.into_iter().zip(DIVISION_296_EXCEPTIONS) {
        let set = flag(value, name)?;
        if set && excepted.is_none() {
            excepted = Some(reason);
        }
        if name == "died_in_year" {
            died = set;
        }
    }
    Ok((excepted, died))
}

/// Whether s 296-15 makes the individual liable, with the reason given.
fn division_296_liability(
    excepted: Option<&'static str>,
    reference: &Decimal,
    large: &Decimal,
    earnings: &Decimal,
    percentage: &Decimal,
) -> Result<(bool, &'static str)> {
    Ok(if let Some(reason) = excepted {
        (false, reason)
    } else if !reference.gt(large)? {
        (
            false,
            "not liable: the total superannuation balance at the end of the year is not greater than the large superannuation balance threshold (s 296-40(1)(a); Income Tax (Transitional Provisions) Act 1997 s 296-1(2))",
        )
    } else if !earnings.gt(&int(0))? {
        (
            false,
            "not liable: total superannuation earnings for the year are not greater than nil (s 296-40(1)(b))",
        )
    } else if percentage.is_zero() {
        (
            true,
            "liable (s 296-15), but the s 296-40(3) percentage rounds to 0.00, so taxable superannuation earnings are nil",
        )
    } else {
        (true, "liable (s 296-15)")
    })
}

/// The authority for an earnings amount that is nil because the balance is not
/// above the threshold: after death it is taken to be nil (s 296-50). The
/// Division 296 steps cite ITAA 1997 compilation 268 (1 October 2026), the
/// Income Tax (Transitional Provisions) Act 1997 compilation 104 (1 October
/// 2026) and the 2026 Imposition Act as made, all read on 15 November 2026.
fn division_296_nil(died: bool, section: &'static str) -> &'static str {
    match (died, section) {
        (true, "296-40") => {
            "ITAA 1997 s 296-50 and s 296-40(1)(a): none, as the balance is taken to be nil after death"
        }
        (true, _) => {
            "ITAA 1997 s 296-50 and s 296-45(1): none, as the balance is taken to be nil after death"
        }
        (false, "296-40") => {
            "ITAA 1997 s 296-40(1)(a) and (3): none, as the balance is not greater than the large superannuation balance threshold or the percentage rounds to 0.00"
        }
        (false, _) => {
            "ITAA 1997 s 296-45(1) and (3): none, as the balance is not greater than the very large superannuation balance threshold or the percentage rounds to 0.00"
        }
    }
}

/// The earnings amounts ss 296-40 and 296-45 define, whether or not an exception
/// removes the liability; also whether the very large component is negative. A
/// share is nil when the balance is not above its threshold, so taxable
/// superannuation earnings arise only from a nonzero first share and earnings
/// greater than nil (s 296-40(1)), and s 296-45 has no 'greater than nil'
/// condition. Results carry no negative amounts, so a negative component is
/// reported as null, and a zero one, which earnings of -0.00 would sign, as 0.00.
fn division_296_earnings(
    died: bool,
    earnings: &Decimal,
    [percentage, very_large_percentage]: [&Decimal; 2],
) -> Calc<([Step; 2], bool)> {
    let share = |percentage: &Decimal| earnings.mul(percentage)?.div(&int(100));
    let taxable = if !earnings.gt(&int(0))? {
        let none = "ITAA 1997 s 296-40(1)(b): none, as total superannuation earnings are not greater than nil";
        step("taxable_superannuation_earnings", none, "0", &[], int(0))
    } else if percentage.is_zero() {
        let none = division_296_nil(died, "296-40");
        step("taxable_superannuation_earnings", none, "0", &[], int(0))
    } else {
        step(
            "taxable_superannuation_earnings",
            "ITAA 1997 s 296-40(1) to (3); Income Tax (Transitional Provisions) Act 1997 s 296-1(2)(a) and (b): the balance at the end of 2026-27",
            "total_super_earnings x taxable_percentage / 100",
            &[earnings, percentage],
            share(percentage)?,
        )
    };
    let id = "very_large_balance_earnings_component";
    let component = share(very_large_percentage)?;
    let negative = component.lt(&int(0))?;
    let very_large = if very_large_percentage.is_zero() {
        step(id, division_296_nil(died, "296-45"), "0", &[], int(0))
    } else if component.is_zero() {
        let none = "ITAA 1997 s 296-45(1): none, as total superannuation earnings are nil";
        step(id, none, "0", &[], int(0))
    } else {
        let authority = if negative {
            "ITAA 1997 s 296-45, which has no 'greater than nil' condition: negative, so the result shows null"
        } else {
            "ITAA 1997 s 296-45; Income Tax (Transitional Provisions) Act 1997 s 296-1(2)(c) and (d): the balance at the end of 2026-27"
        };
        let formula = "total_super_earnings x very_large_percentage / 100";
        step(
            id,
            authority,
            formula,
            &[earnings, very_large_percentage],
            component,
        )
    };
    Ok(([taxable, very_large], negative))
}

/// The 2 taxes and their total, which only a liability gives; `liability` is
/// the reason for none.
fn division_296_taxes(
    (liable, liability): (bool, &'static str),
    [taxable, component]: [&Decimal; 2],
    [rate_, very_large_rate]: [&Decimal; 2],
) -> Calc<[Step; 3]> {
    let act =
        "Superannuation (Building a Stronger and Fairer Super System) Imposition Act 2026 s 5";
    let (tax, further) = if liable {
        (
            step(
                "tax_on_taxable_superannuation_earnings",
                "Superannuation (Building a Stronger and Fairer Super System) Imposition Act 2026 s 5(a) and (b)(i)",
                "taxable_superannuation_earnings x rate",
                &[taxable, rate_],
                taxable.mul(rate_)?,
            ),
            step(
                "tax_on_very_large_balance_earnings_component",
                "Superannuation (Building a Stronger and Fairer Super System) Imposition Act 2026 s 5(b)(ii); nil without a component (s 5(a))",
                "very_large_balance_earnings_component x very_large_rate",
                &[component, very_large_rate],
                component.mul(very_large_rate)?,
            ),
        )
    } else {
        (
            step(
                "tax_on_taxable_superannuation_earnings",
                liability,
                "0",
                &[],
                int(0),
            ),
            step(
                "tax_on_very_large_balance_earnings_component",
                liability,
                "0",
                &[],
                int(0),
            ),
        )
    };
    let total = step(
        "division_296_tax",
        act,
        "tax_on_taxable_superannuation_earnings + tax_on_very_large_balance_earnings_component",
        &[&tax.value, &further.value],
        tax.value.add(&further.value)?,
    );
    Ok([tax, further, total])
}

pub fn division_296(
    total_super_balance_end_of_year: &Arg,
    total_super_earnings: &Arg,
    exceptions: [&Arg; 3],
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "division_296")?;
    let balance = money(total_super_balance_end_of_year)?;
    let earnings = signed_money(total_super_earnings)?;
    let (excepted, died) = division_296_exceptions(exceptions)?;
    let [large, very_large, rate_, very_large_rate] = [
        "large_threshold",
        "very_large_threshold",
        "rate",
        "very_large_rate",
    ]
    .map(|key| d(lookup(&meta::DIVISION_296, key)));
    // ITTPA s 296-1(2): for 2026-27 the reference amount is the balance at the end of the
    // year, which s 296-50 takes to be nil for someone who has died by then.
    let reference = if died { int(0) } else { balance };
    let percentage = percentage_above(&reference, &large)?;
    let very_large_percentage = percentage_above(&reference, &very_large)?;
    let liability = division_296_liability(excepted, &reference, &large, &earnings, &percentage)?;
    let (steps, negative_component) = precise(|| {
        let ([taxable, component], negative) =
            division_296_earnings(died, &earnings, [&percentage, &very_large_percentage])?;
        let taxes = division_296_taxes(
            liability,
            [&taxable.value, &component.value],
            [&rate_, &very_large_rate],
        )?;
        let mut steps = vec![taxable, component];
        steps.extend(taxes);
        Ok((steps, negative))
    })?;
    let mut out = derived_result(
        "division_296",
        &year,
        steps,
        vec![
            rate("large_superannuation_balance_threshold", &large),
            rate("very_large_superannuation_balance_threshold", &very_large),
            rate("rate", &rate_),
            rate("very_large_rate", &very_large_rate),
            rate("taxable_percentage", &percentage),
            rate("very_large_percentage", &very_large_percentage),
        ],
    )?;
    if negative_component {
        let id = "very_large_balance_earnings_component";
        out["amounts"][id] = Value::Null;
        STEPS.with_borrow_mut(|recorded| {
            if let Some(step) = recorded.iter_mut().flatten().find(|s| s["id"] == id) {
                step["value"] = Value::Null;
            }
        });
    }
    out["liability"] = json!(liability.1);
    Ok(out)
}

/// SISR reg 6.01(2): 55 for a person born before 1 July 1960, one more for
/// each later year of birth from 1 July, and 60 for one born after 30 June 1964.
fn preservation_age(born: (i64, i64, i64)) -> i64 {
    let (year, month, _) = born;
    let birth_year_from_july = if month >= 7 { year } else { year - 1 };
    (birth_year_from_july - 1960 + 56).clamp(55, 60)
}

/// Schedule 11 for one ETP part: the part within its cap, the part above it,
/// and the amount withheld from both, rounded to the dollar with 50 cents up.
fn etp_part(
    component: &Decimal,
    cap: &Decimal,
    concessional: &Decimal,
    top: &Decimal,
) -> Calc<[Decimal; 3]> {
    let within = py_min(component.clone(), cap.clone())?;
    let above = component.sub(&within)?;
    let withheld = within
        .mul(concessional)?
        .add(&above.mul(top)?)?
        .quantize(0, Rounding::HalfUp)?;
    Ok([within, above, withheld])
}

/// Schedule 11 for one ETP: the tax-free part of the redundancy payment, then
/// the excluded part (code R) first, under the ETP cap alone, and the
/// non-excluded part (code O), under the smaller of the ETP cap left and the
/// whole-of-income cap (s 82-10(4), (6) and (7); Schedule 11). `reached` is
/// whether the payee is their preservation age or older at the year's end,
/// which sets the rate within a cap. The s 83-170(3) formula is
/// 'Base amount + (Service amount × Years of service)' in the Act's text on
/// the ATO's legal database (PAC/19970038/83-170), read on 15 November 2026;
/// Schedule 11's note 1 and example 4 apply it.
fn etp_steps(
    redundancy: &Decimal,
    years: i64,
    excluded: &Decimal,
    non_excluded: &Decimal,
    other: &Decimal,
    (concessional, reached): (&Decimal, bool),
) -> Result<Vec<Step>> {
    let [etp_cap, whole_of_income, base, service, top] = [
        "etp_cap",
        "whole_of_income_cap",
        "base_amount",
        "service_amount",
        "above_cap_rate",
    ]
    .map(|key| d(lookup(&meta::TERMINATION_PAYMENT, key)));
    let withholding = if reached {
        "Schedule 11, Table A, preservation age or over, and 'Working out the withholding amount': to the nearest dollar, 50 cents up"
    } else {
        "Schedule 11, Table A, under preservation age, and 'Working out the withholding amount': to the nearest dollar, 50 cents up"
    };
    let above_cap = "Schedule 11, 'ETP caps': the amount over the cap, at the top rate";
    precise(|| {
        // s 83-170(3): base amount + service amount x years of service.
        let n = int(years);
        let limit = base.add(&service.mul(&n)?)?;
        let tax_free = py_min(redundancy.clone(), limit.clone())?;
        let excluded_part = redundancy.sub(&tax_free)?.add(excluded)?;
        let [r_within, r_above, r_withheld] =
            etp_part(&excluded_part, &etp_cap, concessional, &top)?;
        let remaining = etp_cap.sub(&r_within)?;
        let calculated = py_max(int(0), whole_of_income.sub(other)?)?;
        let non_excluded_cap = py_min(remaining.clone(), calculated.clone())?;
        let [o_within, o_above, o_withheld] =
            etp_part(non_excluded, &non_excluded_cap, concessional, &top)?;
        let total = r_withheld.add(&o_withheld)?;
        let parts = "dollars(excluded_within_cap x withholding_rate_within_cap \
                     + excluded_above_cap x withholding_rate_above_cap)";
        let other_parts = "dollars(non_excluded_within_cap x withholding_rate_within_cap \
                           + non_excluded_above_cap x withholding_rate_above_cap)";
        Ok(vec![
            step(
                "tax_free_limit",
                "ITAA 1997 s 83-170(3); Schedule 11, 'ETP caps', note 1",
                "redundancy_base_amount + redundancy_service_amount x completed_years_of_service",
                &[&base, &service, &n],
                limit.clone(),
            ),
            step(
                "tax_free_redundancy_amount",
                "ITAA 1997 s 83-170(2)",
                "min(redundancy_payment, tax_free_limit)",
                &[redundancy, &limit],
                tax_free.clone(),
            ),
            step(
                "excluded_taxable_component",
                "ITAA 1997 s 83-170(3) note and s 82-10(6): the redundancy payment over its tax-free part, with the other excluded payments",
                "redundancy_payment - tax_free_redundancy_amount + excluded_payment",
                &[redundancy, &tax_free, excluded],
                excluded_part.clone(),
            ),
            step(
                "excluded_within_cap",
                "ITAA 1997 s 82-10(4)(a), (6) and (7)(a)(i); Schedule 11, 'ETP caps', column 1: the ETP cap alone",
                "min(excluded_taxable_component, etp_cap)",
                &[&excluded_part, &etp_cap],
                r_within.clone(),
            ),
            step(
                "excluded_above_cap",
                above_cap,
                "excluded_taxable_component - excluded_within_cap",
                &[&excluded_part, &r_within],
                r_above.clone(),
            ),
            step(
                "excluded_withholding",
                withholding,
                parts,
                &[&r_within, concessional, &r_above, &top],
                r_withheld.clone(),
            ),
            step(
                "remaining_etp_cap",
                "ITAA 1997 s 82-10(4)(a) and (7)(a)(ii): the ETP cap less the excluded part within it",
                "etp_cap - excluded_within_cap",
                &[&etp_cap, &r_within],
                remaining.clone(),
            ),
            step(
                "whole_of_income_cap",
                "ITAA 1997 s 82-10(4)(c) and (5); Schedule 11, 'Steps to work out smallest cap', steps 1 to 3",
                "max(0, whole_of_income_cap_amount - other_taxable_payments)",
                &[&whole_of_income, other],
                calculated.clone(),
            ),
            step(
                "non_excluded_cap",
                "ITAA 1997 s 82-10(4); Schedule 11, 'Steps to work out smallest cap', steps 4 and 5",
                "min(remaining_etp_cap, whole_of_income_cap)",
                &[&remaining, &calculated],
                non_excluded_cap.clone(),
            ),
            step(
                "non_excluded_taxable_component",
                "ITAA 1997 s 82-10(7)(a)(ii): the part that is not an excluded payment",
                "non_excluded_payment",
                &[non_excluded],
                non_excluded.clone(),
            ),
            step(
                "non_excluded_within_cap",
                "ITAA 1997 s 82-10(4); Schedule 11, 'ETP caps', column 2",
                "min(non_excluded_taxable_component, non_excluded_cap)",
                &[non_excluded, &non_excluded_cap],
                o_within.clone(),
            ),
            step(
                "non_excluded_above_cap",
                above_cap,
                "non_excluded_taxable_component - non_excluded_within_cap",
                &[non_excluded, &o_within],
                o_above.clone(),
            ),
            step(
                "non_excluded_withholding",
                withholding,
                other_parts,
                &[&o_within, concessional, &o_above, &top],
                o_withheld.clone(),
            ),
            step(
                "total_withholding",
                "Schedule 11, example 4: each part's withholding",
                "excluded_withholding + non_excluded_withholding",
                &[&r_withheld, &o_withheld],
                total,
            ),
        ])
    })
}

/// The whole years of service, from 0 to 150, that a redundancy payment relates
/// to (s 83-170(3)).
fn years_of_service(value: &Arg) -> Result<i64> {
    let Some(Some(years)) = value
        .int()
        .filter(|n| n.is_some_and(|n| (0..=150).contains(&n)))
    else {
        return refuse(
            "input.years_of_service_invalid",
            "completed_years_of_service must be a whole number from 0 to 150.",
        );
    };
    Ok(years)
}

#[allow(clippy::too_many_arguments)]
pub fn termination_payment(
    redundancy_payment: &Arg,
    completed_years_of_service: &Arg,
    excluded_payment: &Arg,
    non_excluded_payment: &Arg,
    other_taxable_payments: &Arg,
    date_of_birth: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "termination_payment")?;
    let redundancy = money(redundancy_payment)?;
    let years = years_of_service(completed_years_of_service)?;
    let excluded = money(excluded_payment)?;
    let non_excluded = money(non_excluded_payment)?;
    let other = money(other_taxable_payments)?;
    let born = dates::to_civil(date_arg(date_of_birth, "date_of_birth")?);
    let figure = |key| lookup(&meta::TERMINATION_PAYMENT, key);
    // s 82-10(3)(a): preservation age or older on the last day of the income year.
    let age = preservation_age(born);
    let year_end: i64 = year[..4].parse::<i64>().expect("income year") + 1;
    let reached = (born.0 + age, born.1, born.2) <= (year_end, 6, 30);
    let within_rate = if reached {
        "preservation_age_rate"
    } else {
        "under_preservation_age_rate"
    };
    let concessional = d(figure(within_rate));
    let steps = etp_steps(
        &redundancy,
        years,
        &excluded,
        &non_excluded,
        &other,
        (&concessional, reached),
    )?;
    let rates = vec![
        rate("etp_cap", figure("etp_cap")),
        rate("whole_of_income_cap_amount", figure("whole_of_income_cap")),
        rate("redundancy_base_amount", figure("base_amount")),
        rate("redundancy_service_amount", figure("service_amount")),
        rate("preservation_age", age),
        rate("preservation_age_reached", reached),
        rate("withholding_rate_within_cap", &concessional),
        rate("withholding_rate_above_cap", figure("above_cap_rate")),
    ];
    derived_result("termination_payment", &year, steps, rates)
}

mod etp_tax_free;
pub use etp_tax_free::etp_tax_free_component;
mod unused_leave;
pub use unused_leave::{LeavePayee, LeaveSplits, unused_leave_payment};

pub fn study_loan_repayment(
    repayment_income: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "study_loan_repayment")?;
    let income = money(repayment_income)?;
    whole_dollars(&income, "repayment income")?;
    let thresholds = lookup(meta::STUDY_LOAN_REPAYMENT_THRESHOLDS, &year);
    let get = |key: &str| *lookup(thresholds, key);
    let (minimum, step_) = (d(get("minimum")), d(get("step")));
    let rates = |key: &str| d(lookup(&meta::STUDY_LOAN_REPAYMENT_RATES, key));
    let (lower, upper, flat) = (rates("lower"), rates("upper"), rates("flat"));
    let (steps, base, rate_applied) = precise(|| {
        let base = step_.sub(&minimum)?.mul(&lower)?;
        let marginal = py_max(
            int(0),
            py_min(income.clone(), step_.clone())?.sub(&minimum)?,
        )?
        .mul(&lower)?
        .add(&py_max(int(0), income.sub(&step_)?)?.mul(&upper)?)?;
        let whole = income.mul(&flat)?;
        let lesser = "HESA s 154-20(1)(a) and (b): the lesser of the 2; the repayable debt in (c) is outside the worksheet";
        let (rate_applied, repayment) = if income.le(&minimum)? {
            let none = "HESA s 154-1(1)(a): nothing unless repayment income exceeds the minimum repayment income";
            (int(0), step("compulsory_repayment", none, "0", &[], int(0)))
        } else if whole.le(&marginal)? {
            let formula = "ten_percent_of_income";
            (
                flat.clone(),
                step(
                    "compulsory_repayment",
                    lesser,
                    formula,
                    &[&whole],
                    whole.clone(),
                ),
            )
        } else {
            let rate_applied = if income.le(&step_)? { &lower } else { &upper };
            let formula = "marginal_amount";
            let repayment = step(
                "compulsory_repayment",
                lesser,
                formula,
                &[&marginal],
                marginal.clone(),
            );
            (rate_applied.clone(), repayment)
        };
        let steps = steps_from(
            &STUDY_LOAN_REPAYMENT_STEPS,
            [
                (&[&income], income.clone()),
                (
                    &[&income, &step_, &minimum, &lower, &income, &step_, &upper],
                    marginal,
                ),
                (&[&income, &flat], whole),
            ],
        );
        Ok((
            steps.into_iter().chain([repayment]).collect(),
            base,
            rate_applied,
        ))
    })?;
    derived_result(
        "study_loan_repayment",
        &year,
        steps,
        vec![
            rate("rate_applied", &rate_applied),
            rate("minimum_repayment_income", &minimum),
            rate("lower_rate", &lower),
            rate("lower_band_top", &step_),
            rate("upper_band_base", &base),
            rate("upper_band_base_as_printed", get("printed_base")),
            rate("upper_rate", &upper),
            rate("flat_rate", &flat),
            rate("flat_from_as_printed", get("flat_from")),
        ],
    )
}

/// `study_loan_repayment`'s first 3 amounts: (id, authority, formula). The
/// minimum repayment income and the top of the 15% band are the indexed
/// amounts of HESA s 154-10 and s 154-20(2) the ATO publishes for the year.
/// The steps cite HESA compilation 105 (17 October 2026), read on 15 November
/// 2026 with the ATO's 'Study and training loan repayment thresholds and
/// rates' page (last updated 30 June 2026).
const STUDY_LOAN_REPAYMENT_STEPS: [(&str, &str, &str); 3] = [
    (
        "repayment_income_used",
        "HESA s 154-5(1): the repayment income the operator established",
        "repayment_income",
    ),
    (
        "marginal_amount",
        "HESA s 154-20(2)(a) and (b), with s 154-10 and s 154-25: 15% of the part above the minimum repayment income to the top of the band, and 17% above it",
        "max(0, min(repayment_income_used, lower_band_top) - minimum_repayment_income) x lower_rate + max(0, repayment_income_used - lower_band_top) x upper_rate",
    ),
    (
        "ten_percent_of_income",
        "HESA s 154-20(1)(b)",
        "repayment_income_used x flat_rate",
    ),
];

#[allow(clippy::too_many_arguments)]
pub fn contribution_caps(
    total_super_balance: &Arg,
    concessional_contributions: &Arg,
    unused_concessional_cap: &Arg,
    non_concessional_contributions: &Arg,
    under_75_in_year: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    determination_date: Option<&Arg>,
    downsizer: Option<&DownsizerArgs>,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "contribution_caps")?;
    let balance = money(total_super_balance)?;
    let concessional = money(concessional_contributions)?;
    let unused = money(unused_concessional_cap)?;
    let non_concessional = money(non_concessional_contributions)?;
    let Arg::Bool(under_75) = under_75_in_year else {
        return refuse("input.flag_invalid", "under_75_in_year must be a boolean.");
    };
    let cap = lookup(meta::CONTRIBUTION_CAPS, &year).0;
    let steps = precise(|| {
        Ok(concessional_steps(
            &balance,
            &concessional,
            &unused,
            &d(cap),
        )?)
    })?;
    caps_result(
        &year,
        &balance,
        steps,
        (&non_concessional, *under_75),
        Vec::new(),
        (determination_date, downsizer),
    )
}

/// `contribution_caps`' result: the concessional `steps`, then the
/// non-concessional amounts, with the year's rates and then `more_rates`.
fn caps_result(
    year: &str,
    balance: &Decimal,
    mut steps: Vec<Step>,
    (non_concessional, under_75): (&Decimal, bool),
    more_rates: Vec<(String, String)>,
    (determination_date, downsizer): (Option<&Arg>, Option<&DownsizerArgs>),
) -> Result<Value> {
    let (cap, transfer) = lookup(meta::CONTRIBUTION_CAPS, year);
    let mut release_warning = None;
    let rates = precise(|| {
        let (cap, transfer) = (d(cap), d(transfer));
        let annual = cap.mul(&int(4))?;
        let multiple = bring_forward_multiple(balance, &transfer, &annual, under_75)?;
        let triggered = multiple > 1 && non_concessional.gt(&annual)?;
        steps.extend(non_concessional_steps(
            non_concessional,
            &annual,
            multiple,
            triggered,
        )?);
        let mut rates = vec![
            rate("concessional_cap", &cap),
            rate("non_concessional_cap", &annual),
            rate("general_transfer_balance_cap", &transfer),
            rate(
                "carry_forward_balance_limit",
                meta::CARRY_FORWARD_BALANCE_LIMIT,
            ),
            rate("non_concessional_cap_multiple", multiple),
            rate(
                "bring_forward_period_years",
                if triggered { multiple } else { 0 },
            ),
        ];
        rates.extend(more_rates);
        if let Some(date) = determination_date {
            release_warning = Some(add_release(&mut steps, &mut rates, year, date)?);
        }
        Ok(rates)
    })?;
    let tests = downsizer::extend(downsizer, year, &mut steps)?;
    let mut out = derived_result("contribution_caps", year, steps, rates)?;
    downsizer::record(&mut out, tests);
    if let Some(warning) = release_warning {
        ato_interest_penalties::warn(&mut out, warning);
    }
    Ok(out)
}

/// s 292-85: nil at or above the transfer balance cap; otherwise the balance
/// band sets how many annual caps the first year can bring forward.
fn bring_forward_multiple(
    balance: &Decimal,
    transfer: &Decimal,
    annual: &Decimal,
    under_75: bool,
) -> Result<i64> {
    Ok(if balance.compare(transfer)?.is_ge() {
        0
    } else if !under_75 || balance.compare(&transfer.sub(annual)?)?.is_ge() {
        1
    } else if balance
        .compare(&transfer.sub(&annual.mul(&int(2))?)?)?
        .is_ge()
    {
        2
    } else {
        3
    })
}

/// Appends the release of the excess non-concessional contributions (the
/// last step) to `determination_date`, returning the ATO-figures warning.
fn add_release(
    steps: &mut Vec<Step>,
    rates: &mut Vec<(String, String)>,
    year: &str,
    date: &Arg,
) -> Result<String> {
    let excess = steps.last().expect("the excess").value.clone();
    let (more, release_rates, warning) = excess_contributions::release_steps(&excess, year, date)?;
    steps.extend(more);
    rates.extend(release_rates);
    Ok(warning)
}

/// The concessional amounts. s 291-20(3)-(4): the unused cap raises this
/// year's cap only by the excess it covers, and only below the $500,000
/// balance; unapplied amounts remain subject to 5-year expiry. The
/// contribution caps steps cite ITAA 1997 compilation 268 (1 October 2026),
/// read on 15 November 2026.
fn concessional_steps(
    balance: &Decimal,
    concessional: &Decimal,
    unused: &Decimal,
    cap: &Decimal,
) -> Calc<Vec<Step>> {
    let excess = py_max(int(0), concessional.sub(cap)?)?;
    let (applied, available) = if balance.lt(&d(meta::CARRY_FORWARD_BALANCE_LIMIT))? {
        (
            step(
                "carry_forward_applied",
                "ITAA 1997 s 291-20(3) and (4): the unused cap applied, but not by more than the excess",
                "min(unused_concessional_cap, max(0, concessional_contributions - concessional_cap))",
                &[unused, concessional, cap],
                py_min(unused.clone(), excess)?,
            ),
            step(
                "concessional_available",
                "ITAA 1997 s 291-20(2) to (4): the cap with the unused cap that can increase it",
                "concessional_cap + unused_concessional_cap",
                &[cap, unused],
                cap.add(unused)?,
            ),
        )
    } else {
        let below = "ITAA 1997 s 291-20(3)(b): no carry forward unless the total superannuation balance is less than $500,000";
        (
            step("carry_forward_applied", below, "0", &[], int(0)),
            step(
                "concessional_available",
                "ITAA 1997 s 291-20(2)",
                "concessional_cap",
                &[cap],
                cap.add(&int(0))?,
            ),
        )
    };
    let available_value = available.value.clone();
    Ok(vec![
        applied,
        available,
        step(
            "concessional_remaining",
            "ITAA 1997 s 291-20(6): the shortfall below the cap",
            "max(0, concessional_available - concessional_contributions)",
            &[&available_value, concessional],
            py_max(int(0), available_value.sub(concessional)?)?,
        ),
        step(
            "excess_concessional",
            "ITAA 1997 s 291-20(1): the excess over the cap",
            "max(0, concessional_contributions - concessional_available)",
            &[concessional, &available_value],
            py_max(int(0), concessional.sub(&available_value)?)?,
        ),
    ])
}

/// The non-concessional amounts: the cap the balance band allows, as a
/// multiple of the general cap (s 292-85(2) to (5)). The authority names a
/// first year of bring forward only when the contributions are over the
/// general cap (s 292-85(3)(a)), `triggered`.
fn non_concessional_steps(
    non_concessional: &Decimal,
    annual: &Decimal,
    multiple: i64,
    triggered: bool,
) -> Calc<Vec<Step>> {
    let authority = match (multiple, triggered) {
        (0, _) => "ITAA 1997 s 292-85(2)(b): nil at or above the general transfer balance cap",
        (1, _) => {
            "ITAA 1997 s 292-85(2)(a): 4 times the s 291-20(2) cap; no bring forward under s 292-85(3)(c) or (e)"
        }
        (2, true) => {
            "ITAA 1997 s 292-85(3) and (5)(a): twice the general cap in a first year of bring forward"
        }
        (2, false) => {
            "ITAA 1997 s 292-85(3) and (5)(a): twice the general cap, should contributions over the general cap make this a first year of bring forward; these are not over it (s 292-85(3)(a)), so bring forward is not triggered"
        }
        (_, true) => {
            "ITAA 1997 s 292-85(3) and (5)(b): 3 times the general cap in a first year of bring forward"
        }
        (_, false) => {
            "ITAA 1997 s 292-85(3) and (5)(b): 3 times the general cap, should contributions over the general cap make this a first year of bring forward; these are not over it (s 292-85(3)(a)), so bring forward is not triggered"
        }
    };
    let times = int(multiple);
    let available = annual.mul(&times)?;
    Ok(vec![
        step(
            "non_concessional_available",
            authority,
            "non_concessional_cap x non_concessional_cap_multiple",
            &[annual, &times],
            available.clone(),
        ),
        step(
            "non_concessional_remaining",
            "ITAA 1997 s 292-85 and s 292-90: the cap less the non-concessional contributions",
            "max(0, non_concessional_available - non_concessional_contributions)",
            &[&available, non_concessional],
            py_max(int(0), available.sub(non_concessional)?)?,
        ),
        step(
            "excess_non_concessional",
            "ITAA 1997 s 292-85 and s 292-90: the non-concessional contributions over the cap",
            "max(0, non_concessional_contributions - non_concessional_available)",
            &[non_concessional, &available],
            py_max(int(0), non_concessional.sub(&available)?)?,
        ),
    ])
}

mod carry_forward;
pub use carry_forward::{CarryForward, contribution_caps_by_year};
mod downsizer;
pub use downsizer::{DOWNSIZER_INPUTS, DownsizerArgs};

fn is_leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub fn pension_minimum(
    account_balance: &Arg,
    age: &Arg,
    days: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    transition_to_retirement: Option<&Arg>,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "pension_minimum")?;
    let ttr = transition_to_retirement
        .map_or(Ok(false), |given| flag(given, "transition_to_retirement"))?;
    let balance = money(account_balance)?;
    let Some(Some(age)) = age
        .int()
        .filter(|n| n.is_some_and(|n| (0..=150).contains(&n)))
    else {
        return refuse(
            "input.age_invalid",
            "age must be an integer number of years from 0 to 150.",
        );
    };
    let (days, days_in_year) = held_days(days, &year)?;
    let factor = d(meta::PENSION_MINIMUM_FACTORS
        .iter()
        .find(|(lowest, _)| age >= *lowest)
        .expect("age 0 band")
        .1);
    let steps = minimum_steps(&balance, &factor, days, days_in_year)?;
    let steps = ttr_maximum(steps, &balance, ttr)?;
    let mut out = derived_result(
        "pension_minimum",
        &year,
        steps,
        vec![
            rate("percentage_factor", &factor),
            rate("days", days),
            rate("days_in_year", days_in_year),
        ],
    )?;
    if ttr {
        texts::add_note(
            &mut out,
            (texts::TTR_MAXIMUM_SOURCE, texts::TTR_MAXIMUM_WARNING),
            texts::TTR_MAXIMUM_SCOPE,
        );
    }
    Ok(out)
}

/// The days the pension is held in `year`, checked, and the days in it.
fn held_days(days: &Arg, year: &str) -> Result<(i64, i64)> {
    let start: i64 = year[..4].parse().expect("supported period");
    let days_in_year: i64 = if is_leap(start + 1) { 366 } else { 365 };
    let Some(Some(days)) = days
        .int()
        .filter(|n| n.is_some_and(|n| (1..=days_in_year).contains(&n)))
    else {
        return refuse(
            "input.day_count_invalid",
            format!("days must be an integer from 1 to {days_in_year} for {year}."),
        );
    };
    Ok((days, days_in_year))
}

/// SISR Schedule 7 cl 1(1), 3 and 4: the minimum for `days` of a
/// `days_in_year` year, before rounding.
fn minimum_before_rounding(
    balance: &Decimal,
    factor: &Decimal,
    days: i64,
    days_in_year: i64,
) -> Result<Step> {
    let (held, length) = (int(days), int(days_in_year));
    precise(|| {
        // cl 4: nothing is required for a pension commencing on or after 1 June,
        // which is within the last 30 days of the income year. Otherwise cl 3
        // pro-rates the factor over a first year that starts after 1 July.
        let (authority, formula, operands, exact): (_, _, &[&Decimal], _) = if days <= 30 {
            ("SISR Schedule 7 cl 4", "0", &[], int(0))
        } else if days < days_in_year {
            (
                "SISR Schedule 7 cl 1(1) and cl 3",
                "account_balance_used x percentage_factor x days / days_in_year",
                &[balance, factor, &held, &length],
                balance.mul(factor)?.mul(&held)?.div(&length)?,
            )
        } else {
            (
                "SISR Schedule 7 cl 1(1)",
                "account_balance_used x percentage_factor",
                &[balance, factor],
                balance.mul(factor)?,
            )
        };
        let id = "minimum_before_rounding";
        Ok(step(id, authority, formula, operands, exact))
    })
}

/// SISR Schedule 7: the minimum for `days` of a `days_in_year` year, before
/// and after rounding.
fn minimum_steps(
    balance: &Decimal,
    factor: &Decimal,
    days: i64,
    days_in_year: i64,
) -> Result<Vec<Step>> {
    let before = minimum_before_rounding(balance, factor, days, days_in_year)?;
    let exact = before.value.clone();
    let minimum = precise(|| {
        Ok(exact
            .div(&int(10))?
            .quantize(0, Rounding::HalfUp)?
            .mul(&int(10))?)
    })?;
    Ok(vec![
        step(
            "account_balance_used",
            "SISR Schedule 7 cl 1(2)",
            "account_balance",
            &[balance],
            balance.clone(),
        ),
        before,
        step(
            "minimum_payment",
            "SISR Schedule 7 cl 5",
            "dollars(minimum_before_rounding / 10) x 10",
            &[&exact],
            minimum,
        ),
    ])
}

/// SISR reg 6.01(2) 'transition to retirement income stream': the maximum
/// payment, 10% of the balance the minimum uses, down to the cent so that
/// what is paid stays within it, after the minimum's steps.
fn ttr_maximum(mut steps: Vec<Step>, balance: &Decimal, ttr: bool) -> Result<Vec<Step>> {
    if ttr {
        steps.push(step(
            "maximum_payment",
            "SISR reg 6.01(2) 'transition to retirement income stream' paras (a)(ii) and (b)(ii): no more than 10% of the account balance on 1 July, or on the commencement day in the first year, not reduced for a part year",
            "cents_down(account_balance_used x 0.10)",
            &[balance],
            precise(|| Ok(balance.mul(&d("0.10"))?.quantize(-2, Rounding::Down)?))?,
        ));
    }
    Ok(steps)
}

/// A period of days, both ends included, and for income use the part of the
/// dwelling's floor area used.
struct Span {
    start: i64,
    end: i64,
    part: Option<Decimal>,
}

fn date_arg(value: &Arg, name: &str) -> Result<i64> {
    match value.text().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => refuse(
            "input.date_invalid",
            format!("{name} must be a YYYY-MM-DD date from 1900 to 2100."),
        ),
    }
}

/// A list of periods inside `first..=last`, sorted, refused if two overlap.
fn spans(value: &Arg, name: &str, with_part: bool, first: i64, last: i64) -> Result<Vec<Span>> {
    let bad = |code, why: &str| refuse(code, format!("{name}: {why}"));
    let Arg::Json(Value::Array(items)) = value else {
        return bad(
            "main_residence.periods_not_a_list",
            "must be a list of periods.",
        );
    };
    if items.len() > 1000 {
        return bad("main_residence.periods_too_many", "at most 1000 periods.");
    }
    let keys: &[&str] = if with_part {
        &["start", "end", "floor_area"]
    } else {
        &["start", "end"]
    };
    let mut out = Vec::new();
    for item in items {
        let fields = match item {
            Value::Object(f)
                if f.len() == keys.len() && keys.iter().all(|k| f.contains_key(*k)) =>
            {
                f
            }
            _ => {
                return bad(
                    "main_residence.period_shape_invalid",
                    &format!(
                        "each period must be an object with exactly {}.",
                        keys.join(", ")
                    ),
                );
            }
        };
        let date = |key: &str| fields[key].as_str().and_then(dates::parse);
        let (Some(start), Some(end)) = (date("start"), date("end")) else {
            return bad(
                "main_residence.period_date_invalid",
                "start and end must be YYYY-MM-DD dates from 1900 to 2100.",
            );
        };
        if start > end || start < first || end > last {
            return bad(
                "main_residence.period_outside_ownership",
                "each period must run forwards inside the ownership period.",
            );
        }
        let part = if with_part {
            let fraction = fields["floor_area"]
                .as_str()
                .and_then(|t| Decimal::parse_input(t).ok());
            match fraction {
                Some(f)
                    if f.is_finite()
                        && f.exponent() >= -4
                        && f.gt(&int(0))?
                        && f.le(&int(1))? =>
                {
                    Some(f)
                }
                _ => {
                    return bad(
                        "main_residence.floor_area_invalid",
                        "floor_area must be a decimal string above 0 and at most 1, to 4 places.",
                    );
                }
            }
        } else {
            None
        };
        out.push(Span { start, end, part });
    }
    out.sort_by_key(|s| s.start);
    if out.windows(2).any(|w| w[1].start <= w[0].end) {
        return bad(
            "main_residence.periods_overlap",
            "periods must not overlap.",
        );
    }
    Ok(out)
}

/// ITAA 1997 Subdiv 118-B: the part of a dwelling's capital gain or loss that
/// is not exempt, from a taxable weight for each day of the ownership period.
#[allow(clippy::too_many_arguments)]
pub fn main_residence(
    capital_gain: &Arg,
    capital_loss: &Arg,
    market_value_basis: &Arg,
    ownership_start: &Arg,
    ownership_end: &Arg,
    lived_periods: &Arg,
    income_use_periods: &Arg,
    absence_choice_periods: &Arg,
    new_home_acquired: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let kind = "main_residence";
    let year = scope(scope_confirmed, year, kind)?;
    let gain = money(capital_gain)?;
    let loss = money(capital_loss)?;
    if !gain.is_zero() && !loss.is_zero() {
        return refuse(
            "main_residence.gain_and_loss_both_supplied",
            "Supply a capital gain or a capital loss, not both.",
        );
    }
    let market_value = flag(market_value_basis, "market_value_basis")?;
    let first = date_arg(ownership_start, "ownership_start")?;
    let last = date_arg(ownership_end, "ownership_end")?;
    if last < first {
        return refuse(
            "main_residence.ownership_dates_reversed",
            "ownership_end must not be before ownership_start.",
        );
    }
    let year_start = dates::parse(&format!("{}-07-01", &year[..4])).expect("income year");
    if last < year_start {
        return refuse(
            "main_residence.ownership_ends_before_year",
            format!(
                "ownership_end must not be before the {year} income year began: the ownership period ends on or after the CGT event."
            ),
        );
    }
    let year_end = dates::add_months(year_start, 12) - 1;
    if first > year_end {
        return refuse(
            "main_residence.ownership_starts_after_year",
            format!(
                "ownership_start must not be after the {year} income year ended: the dwelling is acquired before the CGT event."
            ),
        );
    }
    if first < dates::parse("1985-09-20").expect("date") {
        return refuse(
            "main_residence.pre_cgt_dwelling",
            "A dwelling acquired before 20 September 1985 is out of scope.",
        );
    }
    let lived = spans(lived_periods, "lived_periods", false, first, last)?;
    let income = spans(income_use_periods, "income_use_periods", true, first, last)?;
    let chosen = spans(
        absence_choice_periods,
        "absence_choice_periods",
        false,
        first,
        last,
    )?;
    let new_home = match new_home_acquired {
        Arg::Json(Value::Null) => None,
        Arg::Json(Value::String(t)) => match dates::parse(t) {
            Some(day) if day <= last => Some(day),
            _ => {
                return refuse(
                    "main_residence.new_home_date_invalid",
                    "new_home_acquired must be null or a YYYY-MM-DD date no later than ownership_end.",
                );
            }
        },
        _ => {
            return refuse(
                "main_residence.new_home_date_invalid",
                "new_home_acquired must be null or a YYYY-MM-DD date no later than ownership_end.",
            );
        }
    };

    let days = (last - first + 1) as usize;
    let at = |day: i64| (day - first) as usize;
    let mut is_lived = vec![false; days];
    let mut part: Vec<Option<Decimal>> = vec![None; days];
    let mut is_chosen = vec![false; days];
    for s in &lived {
        is_lived[at(s.start)..=at(s.end)].fill(true);
    }
    for s in &income {
        part[at(s.start)..=at(s.end)].fill(s.part.clone());
    }
    for s in &chosen {
        if s.start == first
            || !is_lived[at(s.start - 1)]
            || is_lived[at(s.start)..=at(s.end)].contains(&true)
        {
            return refuse(
                "main_residence.absence_choice_period_invalid",
                "absence_choice_periods: each period must start the day after a lived period ends and contain no lived day.",
            );
        }
        is_chosen[at(s.start)..=at(s.end)].fill(true);
    }

    // s 118-145: a part used to produce income before the move stays taxable
    // throughout the absence (ATO QC66030, Helen); income use is covered for
    // 6 years per absence, to the sixth anniversary of its first day inclusive
    // (ATO QC66030, Roya), counted across breaks in the same absence. A
    // covered day's weight is that part; a day past the 6 years is taxable.
    let mut chosen_weight: Vec<Option<Decimal>> = vec![None; days];
    let mut past_cap = vec![false; days];
    for s in &chosen {
        let mut run = at(s.start - 1);
        while run > 0 && is_lived[run - 1] {
            run -= 1;
        }
        let before = &part[run..at(s.start)];
        let kept = before.last().cloned().flatten();
        // Once part use begins it must continue, at one floor area, to the move.
        let mut consistent = true;
        let mut began = false;
        for p in before {
            match (p, &kept) {
                (Some(p), Some(k)) => {
                    began = true;
                    consistent &= k.equals(p)?;
                }
                (Some(_), None) => consistent = false,
                (None, _) => consistent &= !began,
            }
        }
        if !consistent {
            return refuse(
                "main_residence.income_use_changed_before_absence",
                "Income use of part of the dwelling changed or stopped before the move that started an absence choice period: out of scope.",
            );
        }
        let kept = kept.unwrap_or_else(|| int(0));
        let mut cap: Option<i64> = None;
        let mut used = 0;
        for day in s.start..=s.end {
            let i = at(day);
            chosen_weight[i] = Some(kept.clone());
            if part[i].is_some() {
                let limit = *cap.get_or_insert_with(|| dates::add_months(day, 72) - day + 1);
                used += 1;
                if used > limit {
                    chosen_weight[i] = Some(int(1));
                    past_cap[i] = true;
                }
            }
        }
    }

    // s 118-140: both homes are main residences for up to 6 months ending
    // when ownership ends, from the 6-month date itself (Library ¶2-075).
    let overlap = match new_home {
        None => Err("not claimed"),
        Some(acquired) => {
            // The 12 months ending on the last day start the day after the
            // same date a year earlier.
            let window = (dates::add_months(last, -12) + 1).max(first);
            let mut three_months = false;
            let mut run_start = None;
            for day in window..=last {
                if is_lived[at(day)] {
                    let a = *run_start.get_or_insert(day);
                    three_months |= day >= dates::add_months(a, 3) - 1;
                } else {
                    run_start = None;
                }
            }
            // Not the main residence: neither lived in nor covered by a choice.
            let income_while_not_home = (window..=last).any(|d| {
                let i = at(d);
                part[i].is_some() && !is_lived[i] && (!is_chosen[i] || past_cap[i])
            });
            if !three_months {
                Err(
                    "not applied: no continuous 3 months as main residence in the 12 months before ownership ended",
                )
            } else if income_while_not_home {
                Err(
                    "not applied: used to produce income in those 12 months while not the main residence",
                )
            } else {
                Ok(acquired.max(dates::months_before(last, 6)).max(first))
            }
        }
    };

    // Each day's taxable weight: 0 exempt, 1 taxable, or the part of the
    // floor area used to produce income.
    let mut weight = vec![int(0); days];
    for (i, w) in weight.iter_mut().enumerate() {
        let day = first + i as i64;
        *w = if is_lived[i] {
            part[i].clone().unwrap_or_else(|| int(0))
        } else if let Some(chosen) = &chosen_weight[i] {
            chosen.clone()
        } else if matches!(overlap, Ok(from) if day >= from) {
            int(0)
        } else {
            int(1)
        };
    }

    // s 118-192: first used to produce income after 20 August 1996 when a
    // full exemption would have applied just before, and only partly exempt
    // because of that use: acquired at market value on that day.
    let first_income = income.first().map(|s| s.start);
    let cutoff = dates::parse("1996-08-20").expect("date");
    let mut from = first;
    if let Some(start) = first_income.filter(|d| *d > first) {
        let exempt_before = weight[..at(start)]
            .iter()
            .try_fold(true, |ok, w| Ok::<bool, Signal>(ok && w.is_zero()))?;
        let taxable_after = weight[at(start)..].iter().any(|w| !w.is_zero());
        if exempt_before && taxable_after && start >= cutoff {
            if start == cutoff {
                return refuse(
                    "main_residence.income_use_started_20_august_1996",
                    "Income use first started on 20 August 1996, when the market value rule's 7.30 pm start applies: out of scope.",
                );
            }
            let other_cause = (at(start)..days)
                .any(|i| !weight[i].is_zero() && part[i].is_none() && !is_chosen[i]);
            if other_cause {
                return refuse(
                    "main_residence.partial_exemption_other_cause",
                    "The dwelling is only partly exempt for a reason besides income use as well as that use: out of scope.",
                );
            }
            from = start;
        }
    }
    let applies = from != first;
    if applies && !market_value {
        return refuse(
            "main_residence.first_income_use_rule_applies",
            format!(
                "The home first used to produce income rule applies (s 118-192): work out the gain or loss from the dwelling's market value on {}, then set market_value_basis.",
                dates::format(from)
            ),
        );
    }
    if !applies && market_value {
        return refuse(
            "main_residence.first_income_use_rule_does_not_apply",
            "The home first used to produce income rule does not apply: work out the gain from the actual cost base, or the loss from the actual reduced cost base, and set market_value_basis to false.",
        );
    }

    let period = &weight[at(from)..];
    let counted = period.len() as i64;
    let (taxable_days, steps) = precise(|| {
        let mut sum = int(0);
        for w in period {
            sum = sum.add(w)?;
        }
        Ok((
            sum.clone(),
            main_residence_steps(&gain, &loss, &sum, counted, applies)?,
        ))
    })?;
    derived_result(
        kind,
        &year,
        steps,
        vec![
            rate("ownership_period_from", dates::format(from)),
            rate("ownership_period_to", dates::format(last)),
            rate("ownership_period_days", counted),
            rate("non_main_residence_days", taxable_days),
            rate(
                "home_first_used_to_produce_income",
                if applies { "applied" } else { "not applied" },
            ),
            rate(
                "changing_main_residence_overlap",
                match overlap {
                    Ok(day) => format!("from {}", dates::format(day)),
                    Err(why) => why.to_string(),
                },
            ),
        ],
    )
}

/// The assessable and exempt parts of the gain and of the loss: the taxable
/// share is rounded to cents first so the exempt share adds back to it. Each
/// day's weight is 1 when the dwelling was not the main residence (s 118-185)
/// and the floor area used when part of it produced income (s 118-190).
/// The steps cite ITAA 1997 compilation 268 (1 October 2026), read on 15
/// November 2026.
fn main_residence_steps(
    gain: &Decimal,
    loss: &Decimal,
    taxable_days: &Decimal,
    counted: i64,
    applies: bool,
) -> Calc<Vec<Step>> {
    let assessable = if applies {
        "ITAA 1997 s 118-185(2) and s 118-190(2), over the ownership period from the first income use (s 118-192(2))"
    } else {
        "ITAA 1997 s 118-185(2) and s 118-190(2)"
    };
    let exempt =
        "ITAA 1997 s 118-110(1) and s 118-185(2): the part the main residence exemption leaves out";
    let days = int(counted);
    let mut steps = Vec::new();
    for (amount, name, [assessable_id, exempt_id]) in [
        (
            gain,
            "capital_gain",
            ["assessable_capital_gain", "exempt_capital_gain"],
        ),
        (
            loss,
            "capital_loss",
            ["assessable_capital_loss", "exempt_capital_loss"],
        ),
    ] {
        let part = cents(amount.mul(taxable_days)?.div(&days)?)?;
        let rest = amount.sub(&part)?;
        let share = format!("cents({name} x non_main_residence_days / ownership_period_days)");
        steps.push(step(
            assessable_id,
            assessable,
            &share,
            &[amount, taxable_days, &days],
            part.clone(),
        ));
        let formula = format!("{name} - {assessable_id}");
        steps.push(step(exempt_id, exempt, &formula, &[amount, &part], rest));
    }
    Ok(steps)
}

/// The dates that fix the period of a loan under s 25-25(5): the first day
/// it was borrowed, the end of its original contract and the day it was
/// repaid.
pub struct LoanDates<'a> {
    pub start: &'a Arg,
    pub term_end: &'a Arg,
    pub repaid_on: &'a Arg,
}

/// A loan's validated dates as day numbers.
struct Loan {
    start: i64,
    term_end: Option<i64>,
    repaid: Option<i64>,
    /// The income year the loan began in, by the calendar year it starts in.
    first_year: i64,
}

impl LoanDates<'_> {
    /// Each date in turn.
    fn parse(&self) -> Result<Loan> {
        let start = date_arg(self.start, "loan_start")?;
        let (year, month, _) = dates::to_civil(start);
        let later = |value: &Arg, name: &str| match value {
            Arg::Json(Value::Null) => Ok(None),
            Arg::Json(Value::String(t)) if dates::parse(t).is_some_and(|day| day >= start) => {
                Ok(dates::parse(t))
            }
            _ => refuse(
                "borrowing.loan_date_invalid",
                format!("{name} must be null or a YYYY-MM-DD date no earlier than loan_start."),
            ),
        };
        let term_end = later(self.term_end, "loan_term_end")?;
        let repaid = later(self.repaid_on, "repaid_on")?;
        let first_year = if month >= 7 { year } else { year - 1 };
        Ok(Loan {
            start,
            term_end,
            repaid,
            first_year,
        })
    }
}

/// The first day of the income year that starts in July of `first`.
fn income_year_start(first: i64) -> i64 {
    dates::parse(&format!("{first}-07-01")).expect("income year")
}

/// s 25-25(5)(c): the last day of the 5 years starting on `start`, that day
/// counted (Acts Interpretation Act 1901 s 36(1) item 2). Read as 60 months
/// under s 2G(2) of that Act, they end immediately before the same day of the
/// month 60 months on or, where that month has no such day, at its end: a
/// loan first drawn on 29 February 2024 has its 5 years end on 28 February
/// 2029, as they would by the anniversary.
fn five_years_end(start: i64) -> i64 {
    dates::months_end(start, 60)
}

/// One income year of the spread under s 25-25(4).
struct SpreadYear {
    /// The income year, by the calendar year it starts in.
    first: i64,
    /// The last day of the period of the loan, as determined at the end of
    /// the income year.
    end: i64,
    remaining: Decimal,
    /// The days left in the period of the loan from the year's first day in it.
    left: i64,
    /// Those days in the income year.
    days: i64,
    maximum: Decimal,
}

impl Loan {
    /// s 25-25(5): the shortest of the contract's period, the period to the
    /// day the loan is repaid and 5 years from the first day it was borrowed,
    /// each day counted (the ATO's 3 July to 2 July 5 years later), as
    /// determined at the end of the income year ending on `last`.
    fn end_at(&self, last: i64) -> i64 {
        let five_years = five_years_end(self.start);
        let planned = self.term_end.map_or(five_years, |end| end.min(five_years));
        // Determined at the end of each income year: a repayment counts from then.
        match self.repaid {
            Some(day) if day <= last => planned.min(day),
            _ => planned,
        }
    }

    /// The paragraph of s 25-25(5) that ends the period of the loan on `end`.
    fn basis(&self, end: i64) -> &'static str {
        if self.term_end == Some(end) {
            "s 25-25(5)(a): the original loan contract"
        } else if self.repaid == Some(end) {
            "s 25-25(5)(b): to the day the loan was repaid"
        } else {
            "s 25-25(5)(c): 5 years from the first borrowing"
        }
    }

    /// s 25-25(4), year by year from the income year the loan began to the
    /// one its period ends in: the remaining expenditure over the days left,
    /// times those days in the year. Each maximum is rounded to cents and
    /// carried forward at that amount, as the ATO's examples are, so the year
    /// the period ends takes what remains.
    fn years(&self, expenditure: &Decimal) -> Result<Vec<SpreadYear>> {
        precise(|| {
            let mut remaining = expenditure.clone();
            let mut rows = Vec::new();
            let mut first = self.first_year;
            loop {
                let from = income_year_start(first);
                let last = income_year_start(first + 1) - 1;
                let end = self.end_at(last);
                let begin = self.start.max(from);
                let (left, days) = (end - begin + 1, end.min(last) - begin + 1);
                let maximum = cents(remaining.mul(&int(days))?.div(&int(left))?)?;
                let after = remaining.sub(&maximum)?;
                rows.push(SpreadYear {
                    first,
                    end,
                    remaining,
                    left,
                    days,
                    maximum,
                });
                if end <= last {
                    return Ok(rows);
                }
                remaining = after;
                first += 1;
            }
        })
    }
}

/// The borrowing expenses steps cite ITAA 1997 s 25-25 in the ATO's legal
/// database and the ATO's 'Borrowing expenses' page (updated 22 May 2026),
/// both read on 15 and 19 November 2026.
const BORROWING_EXPENSES: &str =
    "ITAA 1997 s 25-25(1): the expenditure incurred for borrowing the money";

/// `not_spread`'s amounts: (id, authority, formula).
const NOT_SPREAD_STEPS: [(&str, &str, &str); 5] = [
    (
        "borrowing_expenses",
        BORROWING_EXPENSES,
        "borrowing_expenses",
    ),
    (
        "remaining_expenditure",
        "ITAA 1997 s 25-25(6): not spread, so all of it",
        "borrowing_expenses",
    ),
    (
        "maximum_deduction",
        "ITAA 1997 s 25-25(6): deducted for the income year it is incurred",
        "borrowing_expenses",
    ),
    (
        "deduction",
        "ITAA 1997 s 25-25(6)(c) and (d): the proportion for the income-producing use, to the cent",
        "cents(maximum_deduction x income_use)",
    ),
    (
        "remaining_expenditure_after_year",
        "ITAA 1997 s 25-25(6): nothing is left to spread",
        "0",
    ),
];

/// `2026-27` for 2026.
fn income_year(first: i64) -> String {
    format!("{first}-{:02}", (first + 1) % 100)
}

/// s 25-25(6): the deduction, not spread, in the income year the expenditure
/// was incurred.
fn not_spread(year: &str, loan: &Loan, expenditure: &Decimal, share: &Decimal) -> Result<Value> {
    let incurred_in = income_year(loan.first_year);
    if incurred_in != year {
        return refuse(
            "borrowing.s25_25_6_not_spread",
            format!(
                "s 25-25(6) applied in the {incurred_in} income year the expenditure was incurred, so its deduction is not spread to {year}."
            ),
        );
    }
    let steps = precise(|| {
        let deduction = cents(expenditure.mul(share)?)?;
        Ok(steps_from(
            &NOT_SPREAD_STEPS,
            [
                (&[expenditure], expenditure.clone()),
                (&[expenditure], expenditure.clone()),
                (&[expenditure], expenditure.clone()),
                (&[expenditure, share], deduction),
                (&[], int(0)),
            ],
        ))
    })?;
    derived_result(
        "borrowing_expenses",
        year,
        steps,
        vec![
            rate("spread", "not spread: $100 or less (s 25-25(6))"),
            rate("income_use", share),
        ],
    )
}

/// s 25-25(4): the expenditure spread by days over the period of the loan.
fn spread(year: &str, loan: &Loan, expenditure: &Decimal, share: &Decimal) -> Result<Value> {
    let claimed: i64 = year[..4].parse().expect("income year");
    let rows = loan.years(expenditure)?;
    let Some(at) = rows.iter().position(|row| row.first == claimed) else {
        let end = rows.last().expect("the year the loan began").end;
        return refuse(
            "borrowing.loan_period_ended",
            format!(
                "The period of the loan ended on {}, before the {year} income year began: nothing remains to deduct.",
                dates::format(end)
            ),
        );
    };
    let row = &rows[at];
    // Each earlier year's maximum written in, worked on the figures that year used.
    let mut earlier = "borrowing_expenses".to_string();
    for before in &rows[..at] {
        let (remaining, days, left) = (&before.remaining, before.days, before.left);
        earlier.push_str(&format!(" - cents({remaining} x {days} / {left})"));
    }
    let steps = precise(|| {
        let deduction = cents(row.maximum.mul(share)?)?;
        let after = row.remaining.sub(&row.maximum)?;
        let (days, left) = (int(row.days), int(row.left));
        Ok(vec![
            step(
                "borrowing_expenses",
                BORROWING_EXPENSES,
                "borrowing_expenses",
                &[expenditure],
                expenditure.clone(),
            ),
            step(
                "remaining_expenditure",
                "ITAA 1997 s 25-25(4) step 1: the expenditure less the maximum deduction for each earlier income year",
                &earlier,
                &[expenditure],
                row.remaining.clone(),
            ),
            step(
                "maximum_deduction",
                "ITAA 1997 s 25-25(4) steps 2 to 4, rounded to the cent and carried forward as the ATO's 'Borrowing expenses' examples are",
                "cents(remaining_expenditure x days_in_year / remaining_loan_period_days)",
                &[&row.remaining, &days, &left],
                row.maximum.clone(),
            ),
            step(
                "deduction",
                "ITAA 1997 s 25-25(2) and (3): the proportion for the income-producing use, to the cent",
                "cents(maximum_deduction x income_use)",
                &[&row.maximum, share],
                deduction,
            ),
            step(
                "remaining_expenditure_after_year",
                "ITAA 1997 s 25-25(4) step 1: what remains for a later income year",
                "remaining_expenditure - maximum_deduction",
                &[&row.remaining, &row.maximum],
                after,
            ),
        ])
    })?;
    derived_result(
        "borrowing_expenses",
        year,
        steps,
        vec![
            rate("spread", "spread over the period of the loan (s 25-25(4))"),
            rate("income_use", share),
            rate("period_of_the_loan_from", dates::format(loan.start)),
            rate("period_of_the_loan_to", dates::format(row.end)),
            rate("period_of_the_loan", loan.basis(row.end)),
            rate("remaining_loan_period_days", row.left),
            rate("days_in_year", row.days),
        ],
    )
}

/// Identifies limits that can affect the s 25-25 amount for the income year.
fn borrowing_limit_warnings(out: &mut Value, first: i64) {
    let warnings = out["warnings"].as_array_mut().expect("warnings");
    if first >= 2019 {
        warnings.push(json!(texts::BORROWING_S26_102));
    }
    if first >= 2027 {
        warnings.push(json!(texts::BORROWING_S26_155));
    }
}

/// ITAA 1997 s 25-25: one income year's deduction for one loan's borrowing
/// expenses. Under s 25-25(6) a total of $100 or less is deducted when
/// incurred; otherwise s 25-25(4) spreads it by days over the period of the
/// loan, as determined at the end of each income year, so the year the loan
/// is repaid takes what remains.
pub fn borrowing_expenses(
    borrowing_expenses: &Arg,
    other_borrowing_expenses_in_year: &Arg,
    income_use: &Arg,
    loan: LoanDates<'_>,
    section_26_50_applies: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, "borrowing_expenses")?;
    let expenditure = money(borrowing_expenses)?;
    let other = money(other_borrowing_expenses_in_year)?;
    let Some(share) = finite_between(income_use, "0", "1")? else {
        return refuse(
            "input.fraction_invalid",
            "income_use must be a finite Decimal fraction from 0 to 1.",
        );
    };
    if flag(section_26_50_applies, "section_26_50_applies")? {
        return refuse("borrowing.s26_50_denied", texts::BORROWING_S26_50);
    }
    let loan = loan.parse()?;
    let claimed: i64 = year[..4].parse().expect("income year");
    if loan.first_year > claimed {
        return refuse(
            "borrowing.loan_starts_after_year",
            format!("loan_start must not be after the {year} income year ended."),
        );
    }

    // s 25-25(6): this loan's income-producing share plus the taxpayer's
    // other borrowing expenditure that year, at $100 or less, is not spread.
    // Money not used to produce income at all is outside s 25-25(6)(a) and
    // (b), so it is spread and nothing is deducted (s 25-25(3), note).
    let total = precise(|| Ok(expenditure.mul(&share)?.add(&other)?))?;
    let mut out = if !share.is_zero() && total.le(&int(100))? {
        not_spread(&year, &loan, &expenditure, &share)?
    } else {
        spread(&year, &loan, &expenditure, &share)?
    };
    borrowing_limit_warnings(&mut out, claimed);
    Ok(out)
}

mod borrowing_schedule;
pub use borrowing_schedule::borrowing_expenses_schedule;

mod hire_purchase;
pub use hire_purchase::{
    ChattelMortgageArgs, HirePurchaseArgs, LoanTransactionsArgs, chattel_mortgage_schedule,
    hire_purchase_schedule, loan_transactions,
};

mod business_capital;
pub use business_capital::business_capital_expenditure;

mod balancing;
pub use balancing::balancing_adjustment;
mod franking_account;
pub use franking_account::{FrankingArgs, franking_year};
mod trade_in;
pub use trade_in::{TradeInArgs, trade_in};
mod fbt_ev_exemption;
mod fbt_instalment;
pub use fbt_ev_exemption::{EvExemptionArgs, fbt_ev_exemption_check};
pub use fbt_instalment::{FBT_INSTALMENT_OPTIONAL, FbtInstalmentArgs, fbt_instalment};
mod foreign_purchaser_surcharge;
pub use foreign_purchaser_surcharge::{ForeignPurchaserSurchargeArgs, foreign_purchaser_surcharge};
mod dasp_tax;
pub use dasp_tax::{DaspTaxArgs, dasp_tax};
mod super_benefit_tax;
pub use super_benefit_tax::{
    STREAM_OPTIONAL, SUPER_BENEFIT_OPTIONAL, SuperBenefitTaxArgs, SuperWithholdingArgs,
    super_benefit_tax, super_withholding,
};
mod transfer_duty;
pub use transfer_duty::{TransferDutyArgs, transfer_duty};
mod source_quotes;
pub(crate) use source_quotes::attach as attach_source_quotes;
mod payroll_tax;
pub use payroll_tax::{
    PayrollTaxArgs, PayrollTaxGroupArgs, PayrollTaxMonthlyArgs, payroll_tax, payroll_tax_group,
    payroll_tax_monthly,
};
mod land_tax;
pub use land_tax::{LandTaxArgs, land_tax};
mod workers_compensation_premium;
pub use workers_compensation_premium::{WorkersCompensationArgs, workers_compensation_premium};
mod motor_vehicle_duty;
pub use motor_vehicle_duty::{MotorVehicleDutyArgs, motor_vehicle_duty};
mod insurance_duty;
pub use insurance_duty::{InsuranceDutyArgs, insurance_duty};
mod portable_lsl_levy;
pub use portable_lsl_levy::{PortableLslLevyArgs, portable_lsl_levy};
mod luxury_car_tax;
mod stated;
pub use luxury_car_tax::{LctArgs, luxury_car_tax};
mod cgt_parcels;
pub use cgt_parcels::{CgtCostBaseArgs, CgtParcelsArgs, cgt_cost_base_adjustments, cgt_parcels};

/// One pay that includes an additional payment, as Schedule 5 (NAT 3348) reads it.
struct AdditionalPay {
    kind: &'static str,
    year: String,
    earnings: Decimal,
    period: String,
    scale: i64,
    study_loan: bool,
    /// The pay date, given only with a study loan, for its Schedule 8 component.
    pay_date: Option<Arg>,
    /// Pay periods in a financial year: 52, 26 or 12, with no adjustment for a
    /// year of 53 weekly or 27 fortnightly pays.
    per_year: i64,
    /// The most pays a financial year can hold: 53, 27 or 12.
    most_pays: i64,
}

/// A study loan component needs a year with Schedule 8 coefficients and the
/// pay date `study_loan_withholding` requires for it: 2025-26's apply to
/// payments made from 24 September 2025. Method B(i) recalculates earlier
/// periods, and Schedule 5 does not say with which coefficients when they
/// change during the year, so it takes only a year whose apply from 1 July.
fn study_loan_year(kind: &str, year: &str, pay_date: Option<&Arg>) -> Result<()> {
    let from = |year: &str| *lookup(meta::STUDY_LOAN_WITHHOLDING_FROM, year);
    let b_i = kind == "schedule_5_method_b_i";
    let mut years = meta::supported_periods("study_loan_withholding");
    if !years.contains(&year) {
        years.retain(|y| !b_i || from(y).ends_with("-07-01"));
        return refuse(
            "period.unsupported",
            format!(
                "study_loan is supported for {} only: this crate does not hold the Schedule 8 study and training support loan component for {year}.",
                years.join(", ")
            ),
        );
    }
    if b_i && !from(year).ends_with("-07-01") {
        return refuse(
            "period.unsupported",
            format!(
                "Method B(i) does not take study_loan for {year}: its Schedule 8 coefficients changed on {}, and Schedule 5 does not say which recalculate the earlier periods. Method A takes it with the pay date.",
                from(year)
            ),
        );
    }
    schedule_8_pay_date(pay_date, year)
}

/// The study loan flag and the pay date, which is given only with it.
pub type StudyLoan<'a> = (&'a Arg, Option<&'a Arg>);

/// The study loan flag, checked with its pay date for the year.
fn study_loan_flag(kind: &str, year: &str, (study_loan, pay_date): StudyLoan) -> Result<bool> {
    let study_loan = flag(study_loan, "study_loan")?;
    if study_loan {
        study_loan_year(kind, year, pay_date)?;
    } else if pay_date.is_some() {
        return refuse(
            "payg.pay_date_without_study_loan",
            "pay_date selects the Schedule 8 study and training support loan component; give it only with study_loan true.",
        );
    }
    Ok(study_loan)
}

fn additional_pay(
    kind: &'static str,
    earnings: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    (study_loan, pay_date): StudyLoan,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<AdditionalPay> {
    let (year, earnings, period, scale) =
        payg_checks(kind, earnings, pay_period, scale, year, scope_confirmed)?;
    let study_loan = study_loan_flag(kind, &year, (study_loan, pay_date))?;
    let (per_year, most_pays) = match period.as_str() {
        "weekly" => (52, 53),
        "fortnightly" => (26, 27),
        _ => (12, 12),
    };
    Ok(AdditionalPay {
        kind,
        year,
        earnings,
        period,
        scale,
        study_loan,
        pay_date: pay_date.cloned(),
        per_year,
        most_pays,
    })
}

impl AdditionalPay {
    /// What the relevant tax table withholds from `earnings` for one pay: the
    /// Schedule 1 amount, plus the Schedule 8 component for a study and
    /// training support loan, which Schedule 5 allows in place of separate
    /// calculations.
    fn table(&self, earnings: &Decimal) -> Result<Decimal> {
        let args = [
            Arg::Decimal(earnings.clone()),
            Arg::Text(self.period.clone()),
            Arg::Int(self.scale),
            Arg::Text(self.year.clone()),
            Arg::Bool(true),
        ];
        let [e, p, s, y, c] = &args;
        let out = if self.study_loan {
            study_loan_withholding(e, p, s, y, c, self.pay_date.as_ref())?
        } else {
            payg_withholding(e, p, s, y, c, &PaygOptions::default())?
        };
        Ok(d(out["amounts"]["withholding"]
            .as_str()
            .expect("withholding amount")))
    }

    /// How [`Self::table`] works out its amount on `earnings`: Schedule 1's
    /// formula, plus Schedule 8's component for a study loan, on `e`, the
    /// formula for those earnings, with the coefficients of the rows the
    /// earnings fall in written in. The amount is taken in cents, as the
    /// table's result gives it. Also the number of times `e` appears.
    fn table_formula(&self, earnings: &Decimal, e: &str) -> Calc<(String, usize)> {
        let (x, x_formula, _) = weekly_earnings_working(earnings, &self.period, e)?;
        let x_formula = group(&x_formula);
        let mut tables = vec![payg_rows(&self.year, self.scale).expect("checked scale")];
        if self.study_loan {
            tables.push(study_loan_rows(&self.year, self.scale));
        }
        let mut terms = Vec::new();
        for rows in &tables {
            let (a, b, weekly) = weekly_formula(&x, rows)?;
            let weekly_formula = format!("max(0, dollars({a} x {x_formula} - {b}))");
            terms.push(for_pay_period_working(&weekly, &self.period, &weekly_formula)?.1);
        }
        Ok((format!("cents({})", terms.join(" + ")), tables.len()))
    }

    /// The step for `amount`, [`Self::table`]'s result on `earnings`, whose
    /// formula is `e` on `operands`.
    fn table_step(
        &self,
        id: &'static str,
        authority: &'static str,
        (earnings, e, operands): (&Decimal, &str, &[&Decimal]),
        amount: Decimal,
    ) -> Calc<Step> {
        let (formula, times) = self.table_formula(earnings, e)?;
        let operands: Vec<&Decimal> = (0..times).flat_map(|_| operands.iter().copied()).collect();
        Ok(step(id, authority, &formula, &operands, amount))
    }

    fn finish(&self, steps: Vec<Step>, extra: Vec<(String, String)>) -> Result<Value> {
        let mut rates = vec![
            rate("scale", self.scale),
            rate("study_loan", self.study_loan),
        ];
        rates.extend(extra);
        let period = format!("{} {}", self.year, self.period);
        derived_result(self.kind, &period, steps, rates)
    }
}

fn pay_count(value: &Arg, name: &str, low: i64, high: i64) -> Result<i64> {
    match value.int() {
        Some(Some(n)) if (low..=high).contains(&n) => Ok(n),
        _ => refuse(
            "input.pay_periods_invalid",
            format!("{name} must be an integer from {low} to {high}."),
        ),
    }
}

/// Methods A and B(ii): 47% of the additional payment, and the lesser of that
/// and `extra` as the withholding on it, cents ignored and a negative result nil.
fn within_limit(extra: Decimal, additional: &Decimal) -> Result<(Decimal, Decimal)> {
    let limit = additional.mul(&d(meta::SCHEDULE_5_LIMIT_RATE))?;
    let lesser = py_min(extra, limit.clone())?;
    let on_additional = py_max(int(0), lesser)?.quantize(0, Rounding::Down)?;
    Ok((limit, on_additional))
}

/// Method A and B(ii) steps 3 to 7: the additional payments spread over
/// `periods` pay periods, cents ignored, added to `base`, and the extra the tax
/// table then withholds for one pay, times `periods`.
fn spread_withholding(
    pay: &AdditionalPay,
    base: &Decimal,
    base_withholding: &Decimal,
    additional: &Decimal,
    periods: i64,
) -> Result<(Decimal, Decimal, Decimal)> {
    let share = additional.div(&int(periods))?.quantize(0, Rounding::Down)?;
    let with_share = pay.table(&base.add(&share)?)?;
    let extra = with_share.sub(base_withholding)?.mul(&int(periods))?;
    Ok((share, with_share, extra))
}

/// Method A steps 1 to 10.
fn method_a_steps(pay: &AdditionalPay, additional: &Decimal, periods: i64) -> Result<Vec<Step>> {
    precise(|| {
        let earnings = pay.earnings.quantize(0, Rounding::Down)?;
        let on_earnings = pay.table(&earnings)?;
        let (share, with_share, extra) =
            spread_withholding(pay, &earnings, &on_earnings, additional, periods)?;
        let (limit, on_additional) = within_limit(extra.clone(), additional)?;
        let total = on_earnings.add(&on_additional)?;
        let (n, limit_rate) = (int(periods), d(meta::SCHEDULE_5_LIMIT_RATE));
        let with_share_earnings = earnings.add(&share)?;
        Ok(vec![
            step(
                "earnings_used",
                "Schedule 5, Method A step 1",
                "floor(earnings)",
                &[&pay.earnings],
                earnings.clone(),
            ),
            pay.table_step(
                "withholding_on_earnings",
                "Schedule 5, Method A step 2; Schedule 1, 'Using a formula', with Schedule 8 for a study loan",
                (&earnings, "earnings_used", &[&earnings]),
                on_earnings.clone(),
            )?,
            step(
                "additional_payment_per_period",
                "Schedule 5, Method A step 3",
                "floor(additional_payment / pay_periods)",
                &[additional, &n],
                share.clone(),
            ),
            pay.table_step(
                "withholding_on_earnings_and_share",
                "Schedule 5, Method A steps 4 and 5; Schedule 1, 'Using a formula', with Schedule 8 for a study loan",
                (
                    &with_share_earnings,
                    "earnings_used + additional_payment_per_period",
                    &[&earnings, &share],
                ),
                with_share.clone(),
            )?,
            step(
                "withholding_before_limit",
                "Schedule 5, Method A steps 6 and 7",
                "(withholding_on_earnings_and_share - withholding_on_earnings) x pay_periods",
                &[&with_share, &on_earnings, &n],
                extra.clone(),
            ),
            step(
                "withholding_limit",
                "Schedule 5, Method A step 8 and 'Withholding limit'",
                "additional_payment x limit_rate",
                &[additional, &limit_rate],
                limit.clone(),
            ),
            step(
                "withholding_on_additional_payment",
                "Schedule 5, Method A step 9; a negative result is nil ('Available methods')",
                "floor(max(0, min(withholding_before_limit, withholding_limit)))",
                &[&extra, &limit],
                on_additional.clone(),
            ),
            step(
                "withholding",
                "Schedule 5, Method A step 10",
                "withholding_on_earnings + withholding_on_additional_payment",
                &[&on_earnings, &on_additional],
                total,
            ),
        ])
    })
}

/// Schedule 5 Method A: withholding from a pay that includes back payments,
/// commissions, bonuses or similar payments, spread over the pay periods.
#[allow(clippy::too_many_arguments)]
pub fn schedule_5_method_a(
    earnings: &Arg,
    additional_payment: &Arg,
    pay_periods: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    study_loan: StudyLoan,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let pay = additional_pay(
        "schedule_5_method_a",
        earnings,
        pay_period,
        scale,
        study_loan,
        year,
        scope_confirmed,
    )?;
    let additional = money(additional_payment)?;
    // A payment for a single pay period is added to that pay under Schedule 1.
    let periods = pay_count(pay_periods, "pay_periods", 2, pay.per_year)?;
    let steps = method_a_steps(&pay, &additional, periods)?;
    pay.finish(
        steps,
        vec![
            rate("pay_periods", periods),
            rate("limit_rate", meta::SCHEDULE_5_LIMIT_RATE),
        ],
    )
}

/// An earlier pay period a Method B(i) back payment applies to.
struct BackPaidPeriod {
    earnings: Decimal,
    back_payment: Decimal,
    withheld: Decimal,
}

fn back_payment_periods_invalid<T>(most: i64) -> Result<T> {
    refuse(
        "input.back_payment_periods_invalid",
        format!(
            "back_payment_periods must be a list of 1 to {most} objects, each with exactly earnings, back_payment and withheld as decimal strings."
        ),
    )
}

fn back_payment_periods(value: &Arg, most: i64) -> Result<Vec<BackPaidPeriod>> {
    let Arg::Json(Value::Array(items)) = value else {
        return back_payment_periods_invalid(most);
    };
    if items.is_empty() || items.len() as i64 > most {
        return back_payment_periods_invalid(most);
    }
    items
        .iter()
        .map(|item| {
            let Value::Object(fields) = item else {
                return back_payment_periods_invalid(most);
            };
            if fields.len() != 3 {
                return back_payment_periods_invalid(most);
            }
            let amount = |key: &str| match fields.get(key).and_then(Value::as_str) {
                Some(text) => match Decimal::parse_input(text) {
                    Ok(v) => money(&Arg::Decimal(v)),
                    Err(_) => back_payment_periods_invalid(most),
                },
                None => back_payment_periods_invalid(most),
            };
            Ok(BackPaidPeriod {
                earnings: amount("earnings")?,
                back_payment: amount("back_payment")?,
                withheld: amount("withheld")?,
            })
        })
        .collect()
}

/// Method B(i) steps 1 to 7: each earlier period recalculated, less what was
/// withheld from it, the total nil if negative, with no 47% limit. The
/// periods are a list, so their figures are written into the working.
fn method_b_i_steps(pay: &AdditionalPay, periods: &[BackPaidPeriod]) -> Result<Vec<Step>> {
    precise(|| {
        let mut back_payment = int(0);
        let mut difference = int(0);
        let (mut paid, mut differences) = (Vec::new(), Vec::new());
        for period in periods {
            let total = period.earnings.add(&period.back_payment)?;
            let recalculated = pay.table(&total)?;
            difference = difference.add(&recalculated.sub(&period.withheld)?)?;
            back_payment = back_payment.add(&period.back_payment)?;
            let e = format!("{} + {}", period.earnings, period.back_payment);
            let (table, _) = pay.table_formula(&total, &e)?;
            differences.push(format!("{table} - {}", period.withheld));
            // Written as the total adds it, to 0, so that the working sums to
            // the total: 5E+1 as 50, and -0.00 as 0.00.
            paid.push(int(0).add(&period.back_payment)?.to_string());
        }
        let on_back_payment = py_max(int(0), difference)?;
        let on_earnings = pay.table(&pay.earnings)?;
        let total = on_earnings.add(&on_back_payment)?;
        Ok(vec![
            step(
                "back_payment",
                "Schedule 5, Method B(i) step 1: the back payment for each earlier pay period",
                &paid.join(" + "),
                &[],
                back_payment,
            ),
            step(
                "withholding_on_back_payment",
                "Schedule 5, Method B(i) steps 2 to 5; Schedule 1, 'Using a formula', with Schedule 8 for a study loan; a negative total is nil ('Available methods')",
                &format!("max(0, {})", differences.join(" + ")),
                &[],
                on_back_payment.clone(),
            ),
            pay.table_step(
                "withholding_on_earnings",
                "Schedule 5, Method B(i) step 6; Schedule 1, 'Using a formula', with Schedule 8 for a study loan",
                (&pay.earnings, "earnings", &[&pay.earnings]),
                on_earnings.clone(),
            )?,
            step(
                "withholding",
                "Schedule 5, Method B(i) step 7",
                "withholding_on_earnings + withholding_on_back_payment",
                &[&on_earnings, &on_back_payment],
                total,
            ),
        ])
    })
}

/// Schedule 5 Method B(i): withholding from a pay that includes a back payment
/// for earlier pay periods in the same financial year, recalculated period by period.
pub fn schedule_5_method_b_i(
    earnings: &Arg,
    back_payments: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    study_loan: StudyLoan,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let pay = additional_pay(
        "schedule_5_method_b_i",
        earnings,
        pay_period,
        scale,
        study_loan,
        year,
        scope_confirmed,
    )?;
    let periods = back_payment_periods(back_payments, pay.most_pays - 1)?;
    let steps = method_b_i_steps(&pay, &periods)?;
    pay.finish(steps, vec![rate("back_payment_periods", periods.len())])
}

/// The year to date a Method B(ii) payment is averaged against.
struct YearToDate {
    earnings: Decimal,
    pays: i64,
    /// Earlier additional payments in the year worked by Method B(ii), and
    /// what was withheld from them.
    earlier: Decimal,
    earlier_withheld: Decimal,
}

/// Method B(ii) steps 1 to 12.
fn method_b_ii_steps(
    pay: &AdditionalPay,
    additional: &Decimal,
    to_date: &YearToDate,
) -> Result<Vec<Step>> {
    precise(|| {
        let average = to_date.earnings.div(&int(to_date.pays))?;
        let average = average.quantize(0, Rounding::Down)?;
        let on_average = pay.table(&average)?;
        let all_additional = to_date.earlier.add(additional)?;
        let (share, with_share, extra) =
            spread_withholding(pay, &average, &on_average, &all_additional, pay.per_year)?;
        // Step 8 subtracts what earlier Method B(ii) payments had withheld, and
        // can be negative; results carry no negative amounts, so it is not shown.
        let less_earlier = extra.sub(&to_date.earlier_withheld)?;
        let (limit, on_additional) = within_limit(less_earlier, additional)?;
        let on_earnings = pay.table(&pay.earnings)?;
        let total = on_earnings.add(&on_additional)?;
        let (pays, per_year) = (int(to_date.pays), int(pay.per_year));
        let limit_rate = d(meta::SCHEDULE_5_LIMIT_RATE);
        let with_share_earnings = average.add(&share)?;
        Ok(vec![
            step(
                "average_total_earnings",
                "Schedule 5, Method B(ii) step 1 and 'Average total earnings'",
                "floor(year_to_date_earnings / pay_periods_to_date)",
                &[&to_date.earnings, &pays],
                average.clone(),
            ),
            pay.table_step(
                "withholding_on_average_earnings",
                "Schedule 5, Method B(ii) step 2; Schedule 1, 'Using a formula', with Schedule 8 for a study loan",
                (&average, "average_total_earnings", &[&average]),
                on_average.clone(),
            )?,
            step(
                "additional_payments_per_period",
                "Schedule 5, Method B(ii) step 3 and 'Pay periods per financial year'",
                "floor((earlier_additional_payments + additional_payment) / pay_periods)",
                &[&to_date.earlier, additional, &per_year],
                share.clone(),
            ),
            pay.table_step(
                "withholding_on_average_and_share",
                "Schedule 5, Method B(ii) steps 4 and 5; Schedule 1, 'Using a formula', with Schedule 8 for a study loan",
                (
                    &with_share_earnings,
                    "average_total_earnings + additional_payments_per_period",
                    &[&average, &share],
                ),
                with_share.clone(),
            )?,
            step(
                "withholding_for_the_year",
                "Schedule 5, Method B(ii) steps 6 and 7",
                "(withholding_on_average_and_share - withholding_on_average_earnings) x pay_periods",
                &[&with_share, &on_average, &per_year],
                extra.clone(),
            ),
            step(
                "withholding_limit",
                "Schedule 5, Method B(ii) step 9 and 'Withholding limit'",
                "additional_payment x limit_rate",
                &[additional, &limit_rate],
                limit.clone(),
            ),
            step(
                "withholding_on_additional_payment",
                "Schedule 5, Method B(ii) steps 8 and 10; a negative result is nil ('Available methods')",
                "floor(max(0, min(withholding_for_the_year - earlier_additional_withholding, withholding_limit)))",
                &[&extra, &to_date.earlier_withheld, &limit],
                on_additional.clone(),
            ),
            pay.table_step(
                "withholding_on_earnings",
                "Schedule 5, Method B(ii) step 11; Schedule 1, 'Using a formula', with Schedule 8 for a study loan",
                (&pay.earnings, "earnings", &[&pay.earnings]),
                on_earnings.clone(),
            )?,
            step(
                "withholding",
                "Schedule 5, Method B(ii) step 12",
                "withholding_on_earnings + withholding_on_additional_payment",
                &[&on_earnings, &on_additional],
                total,
            ),
        ])
    })
}

/// Schedule 5 Method B(ii): withholding from a pay that includes an additional
/// payment for a prior financial year or for no single pay period, averaged
/// over the financial year. `to_date` is the year to date's earnings, pay
/// periods, earlier Method B(ii) additional payments and their withholding.
#[allow(clippy::too_many_arguments)]
pub fn schedule_5_method_b_ii(
    earnings: &Arg,
    additional_payment: &Arg,
    to_date: [&Arg; 4],
    pay_period: &Arg,
    scale: &Arg,
    study_loan: StudyLoan,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let pay = additional_pay(
        "schedule_5_method_b_ii",
        earnings,
        pay_period,
        scale,
        study_loan,
        year,
        scope_confirmed,
    )?;
    let additional = money(additional_payment)?;
    let [to_date_earnings, pays_to_date, earlier, earlier_withheld] = to_date;
    let to_date = YearToDate {
        earnings: money(to_date_earnings)?,
        pays: pay_count(pays_to_date, "pay_periods_to_date", 1, pay.most_pays)?,
        earlier: money(earlier)?,
        earlier_withheld: money(earlier_withheld)?,
    };
    let steps = method_b_ii_steps(&pay, &additional, &to_date)?;
    pay.finish(
        steps,
        vec![
            rate("pay_periods_to_date", to_date.pays),
            rate("pay_periods", pay.per_year),
            rate("limit_rate", meta::SCHEDULE_5_LIMIT_RATE),
        ],
    )
}

/// Schedule 15's lookup tool column for a registered employer's payment to a
/// working holiday maker who gave a TFN, and its Table A rate, from the
/// payments made before it in the income year. No column covers payments
/// before of $135,000.01 to $135,000.99, so they are refused rather than
/// placed in column 3 or 4.
fn working_holiday_maker_rate(before: &Decimal) -> Result<(usize, &'static str)> {
    for (index, (low, high, rate_)) in meta::WORKING_HOLIDAY_MAKER_COLUMNS.iter().enumerate() {
        let above = low.map_or(Ok(true), |low| d(low).le(before))?;
        let below = high.map_or(Ok(true), |high| before.le(&d(high)))?;
        if above && below {
            return Ok((index + 2, *rate_));
        }
    }
    refuse(
        "working_holiday_maker.band_unsettled",
        "payments_before of $135,000.01 to $135,000.99 is in none of Schedule 15's columns: column 3 (30%) ends at $135,000 and column 4 (37%) starts at $135,001. Table A's rate column gives 37% on each $1 over $135,000, but its income bands leave the same gap, so the worksheet does not choose; confirm the rate with the ATO before withholding.",
    )
}

const WORKING_HOLIDAY_MAKER: &str = "working_holiday_maker_withholding";

/// Schedule 15 for an unregistered employer whose payee gave a TFN, who
/// withholds at the foreign resident rates: the Schedule 1 scale 3 amount for
/// the pay period.
fn unregistered_working_holiday_maker(
    payment: Decimal,
    period: String,
    year: String,
) -> Result<Value> {
    let label = format!("{year} {period}");
    let rows = payg_rows(&year, 3).expect("scale 3 in every year");
    let authority = if period == "weekly" {
        "Schedule 15, 'Unregistered employer'; Schedule 1, scale 3 (foreign residents), 'Working out the weekly earnings', 'Using a formula' and 'Rounding of withholding amounts'"
    } else {
        "Schedule 15, 'Unregistered employer'; Schedule 1, scale 3 (foreign residents), 'Using a formula' and 'Calculating withholding fortnightly, monthly or quarterly amounts'"
    };
    // Schedule 1 on scale 3, with the row's coefficients written in.
    let steps = precise(|| {
        let (x, x_formula, _) = weekly_earnings_working(&payment, &period, "payment_used")?;
        let (a, b, weekly) = weekly_formula(&x, rows)?;
        let weekly_formula = format!("max(0, dollars({a} x {} - {b}))", group(&x_formula));
        let (withholding, formula) = for_pay_period_working(&weekly, &period, &weekly_formula)?;
        Ok(vec![
            step(
                "payment_used",
                "Schedule 15, 'Unregistered employer': the payment, withheld at foreign resident rates",
                "payment",
                &[&payment],
                payment.clone(),
            ),
            step("withholding", authority, &formula, &[&payment], withholding),
        ])
    })?;
    let mut out = derived_result(WORKING_HOLIDAY_MAKER, &label, steps, vec![rate("scale", 3)])?;
    out["withholding_basis"] = json!(
        "unregistered employer: foreign resident rates, the Schedule 1 scale 3 amount for the pay period"
    );
    Ok(out)
}

/// Schedule 15 at a flat rate: a registered employer's Table A rate for the
/// column, or 45% for any employer without a TFN (for an unregistered one, the
/// Schedule 1 scale 4 foreign resident rate), times the payment with cents ignored.
fn flat_rate_working_holiday_maker(
    payment: &Decimal,
    before: &Decimal,
    registered: bool,
    tfn: bool,
    label: &str,
) -> Result<Value> {
    let (column, rate_) = if tfn {
        working_holiday_maker_rate(before)?
    } else {
        (6, meta::WORKING_HOLIDAY_MAKER_NO_TFN_RATE)
    };
    // Cents are ignored in the result at the top rate (column 5) and without a
    // TFN (column 6); otherwise it is rounded straight to the dollar, 50 cents up.
    let (rounding, formula, authority) = match (column, registered) {
        (2..=4, _) => (
            Rounding::HalfUp,
            "dollars(rate x payment_used)",
            "Schedule 15, 'Registered employer', 'Using a formula' and Table A, and 'Rounding of withholding amounts': to the nearest dollar, 50 cents up",
        ),
        (5, _) => (
            Rounding::Down,
            "floor(rate x payment_used)",
            "Schedule 15, 'Registered employer', 'Using a formula' and Table A, and 'Rounding of withholding amounts': at the top rate, cents in the result are ignored",
        ),
        (_, true) => (
            Rounding::Down,
            "floor(rate x payment_used)",
            "Schedule 15, 'If a TFN has not been provided' and 'Rounding of withholding amounts': 45%, cents in the result ignored",
        ),
        (_, false) => (
            Rounding::Down,
            "floor(rate x payment_used)",
            "Schedule 15, 'Unregistered employer'; Schedule 1, scale 4 and 'Rounding of withholding amounts': the foreign resident rate without a TFN, cents ignored",
        ),
    };
    let used = if registered {
        "Schedule 15, 'Using a formula': the total payment for the pay period, ignoring any cents"
    } else {
        "Schedule 1, scale 4 and 'Rounding of withholding amounts': cents in earnings are ignored"
    };
    let a = d(rate_);
    let steps = precise(|| {
        let x = payment.quantize(0, Rounding::Down)?;
        let y = a.mul(&x)?.quantize(0, rounding)?;
        Ok(vec![
            step(
                "payment_used",
                used,
                "floor(payment)",
                &[payment],
                x.clone(),
            ),
            step("withholding", authority, formula, &[&a, &x], y),
        ])
    })?;
    let mut out = derived_result(WORKING_HOLIDAY_MAKER, label, steps, vec![rate("rate", &a)])?;
    out["withholding_basis"] = json!(if registered {
        format!(
            "registered employer, {}: column {column}, rate {a} on the payment with cents ignored",
            if tfn { "TFN quoted" } else { "no TFN" }
        )
    } else {
        format!(
            "unregistered employer, no TFN: the foreign resident rate without a TFN (Schedule 1 scale 4), {a} on the payment with cents ignored"
        )
    });
    Ok(out)
}

/// Schedule 15: withholding from one payment to a working holiday maker. The
/// steps cite Schedules 1 and 15 of the Withholding Schedules Instruments 2024
/// (F2024L00664), (No. 2) 2025 (F2025L01016) and 2026 (F2026L00716), whose
/// headings they share, read on 15 November 2026.
pub fn working_holiday_maker_withholding(
    payment: &Arg,
    payments_before: &Arg,
    employer_registered: &Arg,
    tfn_quoted: &Arg,
    pay_period: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, WORKING_HOLIDAY_MAKER)?;
    let payment = money(payment)?;
    let before = money(payments_before)?;
    let registered = flag(employer_registered, "employer_registered")?;
    let tfn = flag(tfn_quoted, "tfn_quoted")?;
    let period = weekly_fortnightly_or_monthly(pay_period)?;
    if !registered && tfn {
        return unregistered_working_holiday_maker(payment, period, year);
    }
    let label = format!("{year} {period}");
    flat_rate_working_holiday_maker(&payment, &before, registered, tfn, &label)
}

/// A pay period of a schedule with no quarterly rule.
fn weekly_fortnightly_or_monthly(pay_period: &Arg) -> Result<String> {
    match pay_period.text() {
        Some(p @ ("weekly" | "fortnightly" | "monthly")) => Ok(p.to_string()),
        _ => refuse(
            "input.pay_period_invalid",
            "pay_period must be weekly, fortnightly or monthly.",
        ),
    }
}

const SENIORS_PENSIONERS: &str = "seniors_pensioners_withholding";

/// The index of `value` in `names`, when it is one of them.
fn position(value: &Arg, names: [&str; 3]) -> Option<usize> {
    value
        .text()
        .and_then(|text| names.iter().position(|name| *name == text))
}

/// A senior's or pensioner's status, as Schedule 9 and the offset take it: 0
/// single, 1 a member of an illness-separated couple, 2 a member of a couple.
fn seniors_status(status: &Arg) -> Result<usize> {
    match position(status, ["single", "illness_separated", "couple"]) {
        Some(index) => Ok(index),
        None => refuse(
            "input.status_invalid",
            "status must be single, illness_separated or couple.",
        ),
    }
}

/// Schedule 9's scale for the payee's status and Medicare levy exemption:
/// 1 to 3 (single, illness-separated couple, couple) with the levy, 3 more
/// with a full exemption and 6 more with a half exemption.
fn schedule_9_scale(status: &Arg, exemption: &Arg) -> Result<usize> {
    let status = seniors_status(status)?;
    let Some(exemption) = position(exemption, ["none", "full", "half"]) else {
        return refuse(
            "input.medicare_levy_exemption_invalid",
            "medicare_levy_exemption must be none, full or half.",
        );
    };
    Ok(exemption * 3 + status + 1)
}

/// Schedule 8's component rates for a Schedule 9 payee with a study loan, who
/// has claimed the tax-free threshold at question 5 of either Withholding
/// declaration Schedule 9 names.
fn schedule_9_study_loan_rows(
    year: &str,
    study_loan: StudyLoan,
) -> Result<Option<&'static [CoefficientRow]>> {
    let study_loan = study_loan_flag(SENIORS_PENSIONERS, year, study_loan)?;
    Ok(study_loan.then(|| {
        *lookup(
            lookup(meta::STUDY_LOAN_WITHHOLDING_COEFFICIENTS, year),
            "threshold_or_foreign",
        )
    }))
}

/// Schedule 9: withholding from one payment to a senior or pensioner, by
/// Schedule 1's weekly earnings, formula and pay period conversion on
/// Schedule 9's coefficients. The steps cite Schedule 9 of the Withholding
/// Schedules Instruments 2024 (F2024L00664), (No. 2) 2025 (F2025L01016) and
/// 2026 (F2026L00716), whose headings they share, read on 15 November 2026.
/// With a study loan, the ATO's Schedule 9 pages refer the payer to the study
/// and training support loans weekly tax table (NAT 2173), which adds
/// Schedule 8's component on the same weekly earnings, converted to the pay
/// period as Schedule 8 says; those pages were read the same day.
pub fn seniors_pensioners_withholding(
    earnings: &Arg,
    pay_period: &Arg,
    status: &Arg,
    medicare_levy_exemption: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    study_loan: StudyLoan,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, SENIORS_PENSIONERS)?;
    let earnings = money(earnings)?;
    let period = weekly_fortnightly_or_monthly(pay_period)?;
    let scale = schedule_9_scale(status, medicare_levy_exemption)?;
    let rows = lookup(meta::SENIORS_PENSIONERS_COEFFICIENTS, &year)[scale - 1];
    let loan_rows = schedule_9_study_loan_rows(&year, study_loan)?;
    let (x_authority, authority) = if period == "weekly" {
        (
            "Schedule 9, 'Using a formula': the weekly earnings in whole dollars plus 99 cents",
            "Schedule 9, 'Working out the withholding amount'",
        )
    } else {
        let other = "Schedule 9, 'Calculating fortnightly or monthly withholding amounts'";
        (other, other)
    };
    let (steps, rates) = precise(|| {
        let (x, x_formula, _) = weekly_earnings_working(&earnings, &period, "earnings")?;
        let (a, b, weekly) = weekly_formula(&x, rows)?;
        let (withholding, formula) =
            for_pay_period_working(&weekly, &period, "weekly_withholding")?;
        let mut steps = vec![
            step(
                "weekly_earnings_used",
                x_authority,
                &x_formula,
                &[&earnings],
                x.clone(),
            ),
            step(
                "weekly_withholding",
                "Schedule 9, 'Using a formula', the scale's coefficients, and 'Rounding of withholding amounts'",
                "max(0, dollars(a x weekly_earnings_used - b))",
                &[&a, &x, &b],
                weekly.clone(),
            ),
        ];
        let mut rates = vec![rate("scale", scale), rate("a", a), rate("b", b)];
        let Some(loan_rows) = loan_rows else {
            steps.push(step(
                "withholding",
                authority,
                &formula,
                &[&weekly],
                withholding,
            ));
            return Ok((steps, rates));
        };
        let (component, component_rates) = study_loan_component(&x, &period, loan_rows)?;
        let total = step(
            "withholding",
            "Schedule 9, 'When your payee has a study and training support loan debt', and the study and training support loans weekly tax table it refers to, 'Working out the withholding amount': the component for a payee claiming the tax-free threshold added to the withholding amount",
            "schedule_9_withholding + study_loan_component",
            &[&withholding, &component.value],
            withholding.add(&component.value)?,
        );
        steps.extend([
            step(
                "schedule_9_withholding",
                authority,
                &formula,
                &[&weekly],
                withholding,
            ),
            component,
            total,
        ]);
        rates.extend(component_rates);
        Ok((steps, rates))
    })?;
    derived_result(
        SENIORS_PENSIONERS,
        &format!("{year} {period}"),
        steps,
        rates,
    )
}

const SAPTO: &str = "seniors_pensioners_tax_offset";

/// The optional `seniors-pensioners-tax-offset` inputs about the spouse, each
/// `None` when not given.
pub struct SaptoSpouse<'a> {
    pub rebate_income: Option<&'a Arg>,
    pub taxable_income: Option<&'a Arg>,
}

fn whole_dollar_money(value: &Arg, what: &str) -> Result<Decimal> {
    let amount = money(value)?;
    whole_dollars(&amount, what)?;
    Ok(amount)
}

/// The spouse's rebate income, which s 160AAAA(4) needs for a spouse and which
/// a single person cannot have, and the spouse's taxable income for a transfer.
fn sapto_spouse(spouse: &SaptoSpouse, status: usize) -> Result<(Option<Decimal>, Option<Decimal>)> {
    let given = [
        ("spouse_rebate_income", spouse.rebate_income),
        ("spouse_taxable_income", spouse.taxable_income),
    ];
    if status == 0 {
        return match given.iter().find(|(_, value)| value.is_some()) {
            Some((name, _)) => refuse(
                "input.field_unknown",
                format!(
                    "{name} is not an input of seniors-pensioners-tax-offset for a single person"
                ),
            ),
            None => Ok((None, None)),
        };
    }
    let Some(rebate_income) = spouse.rebate_income else {
        return refuse(
            "input.missing_field",
            "spouse_rebate_income is required unless status is single",
        );
    };
    let rebate_income = whole_dollar_money(rebate_income, "spouse rebate income")?;
    let taxable_income = spouse
        .taxable_income
        .map(|value| whole_dollar_money(value, "spouse taxable income"))
        .transpose()?;
    Ok((Some(rebate_income), taxable_income))
}

fn sapto_rule(key: &str) -> Decimal {
    d(lookup(&meta::SAPTO_RULES, key))
}

/// A non-negative amount rounded up to the whole dollar.
fn dollars_up(value: &Decimal) -> Calc<Decimal> {
    let down = value.quantize(0, Rounding::Down)?;
    if down.lt(value)? {
        down.add(&int(1))
    } else {
        Ok(down)
    }
}

/// The year's tax-free threshold and lowest and second lowest marginal tax
/// rates, from Income Tax Rates Act 1986 Sch 7 Part I (regulations s 4).
fn rates_act_terms(year: &str) -> (Decimal, Decimal, Decimal) {
    let scale = lookup(meta::RESIDENT_TAX_SCALES, year);
    (int(scale[0].0), d(scale[0].2), d(scale[1].2))
}

/// `$rest` of the regulations that set the offset for `$year`: the Income Tax
/// Assessment (1936 Act) Regulations 2025 (F2025L01060, as made) for a year
/// starting from 1 October 2025 (their s 23), otherwise the Income Tax
/// Assessment (1936 Act) Regulation 2015 (F2015L01415, compilation 8), which
/// number ss 9 to 12 the same. Both read on 15 November 2026, with ITAA 1936
/// compilation 193 (1 October 2026).
macro_rules! sapto_regulations {
    ($year:expr, $rest:literal) => {
        if $year >= "2026-27" {
            concat!("Income Tax Assessment (1936 Act) Regulations 2025 ", $rest)
        } else {
            concat!("Income Tax Assessment (1936 Act) Regulation 2015 ", $rest)
        }
    };
}

/// Regulations s 9(4) to (8): the rebate threshold for a base rebate amount,
/// written `base`, its formula and the subsections it applies.
fn rebate_threshold(base: &Decimal, b: &str, year: &str) -> Calc<(Decimal, String, &'static str)> {
    let (free, lowest, second) = rates_act_terms(year);
    let maximum = sapto_rule("rebate_maximum_amount");
    let reduction_rate = sapto_rule("rebate_reduction_rate");
    let reduction_threshold = sapto_rule("rebate_reduction_threshold");
    let amount = maximum.add(base)?.div(&lowest)?.add(&free)?;
    let (amount, formula, authority) = if amount.gt(&reduction_threshold)? {
        // s 9(6), with s 9(7) as its last term.
        let rates = reduction_rate.add(&second)?;
        let last = reduction_threshold.mul(&rates.sub(&lowest)?)?;
        let amount = lowest
            .mul(&free)?
            .add(&maximum)?
            .add(base)?
            .add(&last)?
            .div(&rates)?;
        let formula = format!(
            "(lowest_marginal_tax_rate x tax_free_threshold + rebate_maximum_amount + {b} \
             + rebate_reduction_threshold x (rebate_reduction_rate + second_lowest_marginal_tax_rate \
             - lowest_marginal_tax_rate)) / (rebate_reduction_rate + second_lowest_marginal_tax_rate)"
        );
        (
            amount,
            formula,
            sapto_regulations!(year, "s 9(4)(b), (6), (7) and (8)"),
        )
    } else {
        let formula = format!(
            "(rebate_maximum_amount + {b}) / lowest_marginal_tax_rate + tax_free_threshold"
        );
        (
            amount,
            formula,
            sapto_regulations!(year, "s 9(4)(a), (5) and (8)"),
        )
    };
    Ok((
        dollars_up(&amount)?,
        format!("dollars_up({formula})"),
        authority,
    ))
}

/// Regulations s 11(2): the base amount, written `b`, less 12.5 cents for each
/// dollar of rebate income above the threshold, and nil once that reaches the
/// base.
fn rebate_amount(
    base: &Decimal,
    b: &str,
    threshold: &Decimal,
    income: &Decimal,
) -> Calc<(Decimal, String)> {
    let excess = py_max(int(0), income.sub(threshold)?)?;
    let amount = py_max(
        int(0),
        base.sub(&excess.mul(&sapto_rule("shade_out_rate"))?)?,
    )?;
    let formula =
        format!("max(0, {b} - max(0, rebate_income_used - rebate_threshold) x shade_out_rate)");
    Ok((amount, formula))
}

/// Regulations s 12(3)(b): the spouse's unused base rebate amount, its formula
/// and the provision. At or below $6,000 of taxable income, below the tax-free
/// threshold, the resident spouse pays no tax and the whole base amount is
/// unused (s 12(3)(b)(i)); above it s 12(3)(b)(ii) takes 15% of the excess.
fn unused_from_spouse(
    base: &Decimal,
    spouse_taxable_income: &Decimal,
    year: &str,
) -> Calc<(Decimal, &'static str, &'static str)> {
    let floor = sapto_rule("transfer_income_floor");
    if spouse_taxable_income.le(&floor)? {
        let authority = sapto_regulations!(
            year,
            "s 12(1) and (3)(b)(i): the spouse's taxable income is at most $6,000, so no tax and the whole base amount is unused"
        );
        return Ok((base.clone(), "base_rebate_amount", authority));
    }
    let reduction = spouse_taxable_income
        .sub(&floor)?
        .mul(&sapto_rule("transfer_rate"))?;
    Ok((
        py_max(int(0), base.sub(&reduction)?)?,
        "max(0, base_rebate_amount - (spouse_taxable_income - transfer_income_floor) x transfer_rate)",
        sapto_regulations!(year, "s 12(1), (3)(b)(ii) and (4)"),
    ))
}

/// The steps and rates for one person's offset.
type SaptoWorking = (Vec<Step>, Vec<(String, String)>);

fn sapto_working(
    income: &Decimal,
    year: &str,
    (base, status): (&Decimal, usize),
    spouse_rebate_income: Option<&Decimal>,
    spouse_taxable_income: Option<&Decimal>,
) -> Calc<SaptoWorking> {
    let rates = sapto_rates(year);
    let mut values: Vec<(&str, Decimal)> = rates.iter().map(|(k, v)| (k.as_str(), d(v))).collect();
    values.extend([
        ("rebate_income", income.clone()),
        ("rebate_income_used", income.clone()),
        ("base_rebate_amount", base.clone()),
    ]);
    let (threshold, threshold_formula, threshold_authority) =
        rebate_threshold(base, "base_rebate_amount", year)?;
    let cut_out = dollars_up(&base.div(&sapto_rule("shade_out_rate"))?.add(&threshold)?)?;
    // s 160AAAA(4): a spouse is tested on half the couple's rebate income.
    let (tested, tested_formula, tested_authority) = match spouse_rebate_income {
        Some(spouse) => {
            values.push(("spouse_rebate_income", spouse.clone()));
            (
                income.add(spouse)?.div(&int(2))?,
                "(rebate_income_used + spouse_rebate_income) / 2",
                "ITAA 1936 s 160AAAA(4): half the sum of the taxpayer's and the spouse's rebate income",
            )
        }
        None => (
            income.clone(),
            "rebate_income_used",
            "ITAA 1936 s 160AAAA(3): the taxpayer's rebate income",
        ),
    };
    let entitled = tested.lt(&cut_out)?;
    let mut steps = vec![
        step(
            "rebate_income_used",
            "ITAA 1936 s 160AAAA(3): the taxpayer's rebate income, as established",
            "rebate_income",
            &[income],
            income.clone(),
        ),
        step_by_name(
            "rebate_income_tested",
            tested_authority,
            tested_formula,
            &values,
            tested.clone(),
        ),
        step(
            "base_rebate_amount",
            [
                sapto_regulations!(year, "s 9(2), table item 1: not a spouse"),
                sapto_regulations!(
                    year,
                    "s 9(2), table item 3: a member of an illness separated couple"
                ),
                sapto_regulations!(year, "s 9(2), table item 2: a spouse"),
            ][status],
            &base.to_string(),
            &[],
            base.clone(),
        ),
    ];
    let (offset_before, offset_formula) =
        rebate_amount(base, "base_rebate_amount", &threshold, income)?;
    let mut working = (
        threshold.clone(),
        threshold_formula.clone(),
        threshold_authority,
    );
    let mut offset = (offset_before.clone(), offset_formula);
    let mut raised_by_transfer = false;
    if let Some(spouse) = spouse_taxable_income {
        values.push(("spouse_taxable_income", spouse.clone()));
        // s 12(1)(d) and (e): only an entitled person whose offset is below
        // its base amount takes the spouse's unused amount.
        let (unused, formula, authority) = if entitled && offset_before.lt(base)? {
            unused_from_spouse(base, spouse, year)?
        } else {
            let authority = sapto_regulations!(
                year,
                "s 12(1)(d) and (e): only a person entitled to the rebate, and whose rebate is less than its base amount, takes a spouse's unused amount; not here, so nil"
            );
            (int(0), "0", authority)
        };
        values.push(("spouse_unused_amount_transferred", unused.clone()));
        steps.push(step_by_name(
            "spouse_unused_amount_transferred",
            authority,
            formula,
            &values,
            unused.clone(),
        ));
        if unused.gt(&int(0))? {
            // s 12(3) adds the unused amount to the base for s 11, not s 10.
            let raised = base.add(&unused)?;
            let b = "base_rebate_amount + spouse_unused_amount_transferred";
            let (raised_threshold, formula, _) = rebate_threshold(&raised, b, year)?;
            working = (
                raised_threshold.clone(),
                formula,
                sapto_regulations!(
                    year,
                    "s 9(4) to (8) and s 12(3): the base amount increased by the spouse's unused amount"
                ),
            );
            offset = rebate_amount(&raised, b, &raised_threshold, income)?;
            raised_by_transfer = true;
        }
    }
    let (threshold_used, formula, authority) = working;
    values.push(("rebate_threshold", threshold_used.clone()));
    steps.push(step_by_name(
        "rebate_threshold",
        authority,
        &formula,
        &values,
        threshold_used,
    ));
    // s 10 uses the threshold before any s 12 transfer.
    let before = if raised_by_transfer {
        threshold_formula
    } else {
        "rebate_threshold".to_string()
    };
    values.push(("cut_out_threshold", cut_out.clone()));
    steps.push(step_by_name(
        "cut_out_threshold",
        sapto_regulations!(
            year,
            "s 10: the base rebate amount over 0.125, plus the rebate threshold before any s 12 transfer, rounded up to the dollar"
        ),
        &format!("dollars_up(base_rebate_amount / shade_out_rate + {before})"),
        &values,
        cut_out,
    ));
    // The ATO rounds the offset up to the whole dollar.
    let step = if entitled {
        step_by_name(
            "seniors_pensioners_tax_offset",
            sapto_regulations!(
                year,
                "s 11(2), as s 12 affects it, rounded up to the whole dollar as the ATO does"
            ),
            &format!("dollars_up({})", offset.1),
            &values,
            dollars_up(&offset.0)?,
        )
    } else {
        step(
            "seniors_pensioners_tax_offset",
            sapto_regulations!(
                year,
                "s 10 and ITAA 1936 s 160AAAA(3) and (4): the rebate income tested is not less than the cut-out threshold, so no rebate"
            ),
            "0",
            &[],
            int(0),
        )
    };
    steps.push(step);
    Ok((steps, rates))
}

/// The year's Schedule 7 terms and the regulations' constants.
fn sapto_rates(year: &str) -> Vec<(String, String)> {
    let (free, lowest, second) = rates_act_terms(year);
    let mut rates = vec![
        rate("tax_free_threshold", free),
        rate("lowest_marginal_tax_rate", lowest),
        rate("second_lowest_marginal_tax_rate", second),
    ];
    rates.extend(meta::SAPTO_RULES.iter().map(|(k, v)| rate(k, v)));
    rates
}

/// The seniors and pensioners tax offset (ITAA 1936 s 160AAAA and the
/// regulations' ss 9 to 12) for one person, before it is applied against tax.
pub fn seniors_pensioners_tax_offset(
    rebate_income: &Arg,
    status: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    spouse: SaptoSpouse,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, SAPTO)?;
    let income = whole_dollar_money(rebate_income, "rebate income")?;
    let status = seniors_status(status)?;
    let (spouse_rebate_income, spouse_taxable_income) = sapto_spouse(&spouse, status)?;
    let base = d(meta::SAPTO_BASE_REBATE_AMOUNTS[status]);
    let (steps, rates) = precise(|| {
        Ok(sapto_working(
            &income,
            &year,
            (&base, status),
            spouse_rebate_income.as_ref(),
            spouse_taxable_income.as_ref(),
        )?)
    })?;
    let mut out = derived_result(SAPTO, &year, steps, rates)?;
    year_warning(&mut out, texts::SAPTO_YEAR_NOTES, &year);
    Ok(out)
}

mod lump_sum_arrears;
pub use lump_sum_arrears::lump_sum_arrears_tax_offset;
mod income_tests;
pub use income_tests::income_tests;
mod government_super_amounts;
pub use government_super_amounts::{SPOUSE_INPUTS, government_super_amounts};
mod wine_equalisation_tax;
pub use wine_equalisation_tax::{WetArgs, wine_equalisation_tax};
mod vic_property_taxes;
pub use vic_property_taxes::{VicPropertyTaxesArgs, vic_property_taxes};
mod alcohol_excise;
pub use alcohol_excise::{AlcoholExciseArgs, alcohol_excise};
mod asic_fees;
pub use asic_fees::{AsicFeesArgs, asic_fees};
mod fbt_minor_benefit;
pub use fbt_minor_benefit::{MinorBenefitArgs, fbt_minor_benefit_check};
mod gst_margin_scheme;
pub use gst_margin_scheme::{MarginSchemeArgs, gst_margin_scheme};
mod fhss;
pub use fhss::{FhssArgs, fhss};
mod aml_cdd_review;
pub use aml_cdd_review::aml_cdd_review;
mod ppsr_timing;
pub use ppsr_timing::ppsr_timing;
mod individual_offsets;
pub use individual_offsets::{
    InvalidCarerArgs, MedicalArgs, ZoneArgs, disability_expenses_offset, invalid_carer_tax_offset,
    rebatable_benefit_offset, zone_tax_offset,
};
mod sb_cgt_concessions;
pub use sb_cgt_concessions::{SB_CGT_OPTIONAL, SbCgtConcessionsArgs, sb_cgt_concessions};
mod work_expenses;
pub use work_expenses::{WORK_EXPENSES_OPTIONAL, WORK_EXPENSES_RECORDS, work_expenses};
mod notice_period;
pub use notice_period::{NoticePeriodArgs, notice_period};
mod ato_interest_penalties;
pub use ato_interest_penalties::ato_interest_penalties;
mod excess_contributions;
pub use excess_contributions::excess_contributions;
mod transfer_balance;
pub use transfer_balance::transfer_balance;

mod private_health_insurance_rebate;
pub use private_health_insurance_rebate::{PhiRebateArgs, private_health_insurance_rebate};

mod depreciation_pools;
pub use depreciation_pools::{
    capital_works, depreciation_low_value_pool, depreciation_small_business_pool,
};
mod instant_asset_write_off;
pub use instant_asset_write_off::instant_asset_write_off;
mod small_business_income_offset;
pub use small_business_income_offset::{
    SBITO_OPTIONAL, SmallBusinessIncomeOffsetArgs, small_business_income_offset,
};
mod trading_stock;
pub use trading_stock::{TradingStockArgs, trading_stock};
mod prepayments;
pub use prepayments::{PrepaymentsArgs, prepayments};
mod non_commercial_losses;
pub use non_commercial_losses::{NCL_OPTIONAL, NonCommercialLossesArgs, non_commercial_losses};
mod psi_test;
pub use psi_test::{PSI_OPTIONAL, PsiTestArgs, psi_test};
mod first_home_owner_grant;
pub use first_home_owner_grant::{FHOG_OPTIONAL, FirstHomeOwnerGrantArgs, first_home_owner_grant};
mod payg_instalment;
pub use payg_instalment::{PAYG_INSTALMENT_OPTIONAL, PaygInstalmentArgs, payg_instalment};
mod pcg_2021_4;
pub use pcg_2021_4::{PCG_OPTIONAL, PcgArgs, pcg_2021_4};
mod rental_property;
pub use rental_property::{RentalPropertyArgs, rental_property};
mod foreign_resident_withholding;
pub use foreign_resident_withholding::{
    ForeignResidentWithholdingArgs, foreign_resident_withholding,
};
mod non_resident_withholding;
pub use non_resident_withholding::{
    NON_RESIDENT_WITHHOLDING_OPTIONAL, NonResidentWithholdingArgs, non_resident_withholding,
};
mod residential_vacancy_fee;
pub use residential_vacancy_fee::{
    RESIDENTIAL_VACANCY_FEE_OPTIONAL, ResidentialVacancyFeeArgs, residential_vacancy_fee,
};
mod withholding_schedules;
pub use withholding_schedules::{
    daily_casual_withholding, horticulture_shearing_withholding, no_abn_withholding,
    upward_variation_withholding,
};
mod payment_withholding;
pub use payment_withholding::{
    annuity_withholding, entertainer_withholding, return_to_work_withholding,
    voluntary_agreement_withholding,
};
mod residency_tests;
pub use residency_tests::{ResidencyTestsArgs, residency_tests};
mod lodgment_required;
pub use lodgment_required::{LodgmentRequiredArgs, lodgment_required};
mod rd_tax_offset;
pub use rd_tax_offset::{RdTaxOffsetArgs, rd_tax_offset};
mod esic;
pub use esic::{EsicArgs, esic};
mod gst_adjustments;
pub use gst_adjustments::{
    AnnualApportionmentArgs, BadDebtArgs, CreditablePurposeArgs, FinancialAcquisitionsArgs,
    GoingConcernArgs, SecondHandGoodsArgs, gst_annual_apportionment, gst_bad_debt,
    gst_creditable_purpose_change, gst_financial_acquisitions_threshold, gst_going_concern,
    gst_second_hand_goods,
};
mod gst_food_retailer_sam;
pub use gst_food_retailer_sam::{
    GST_FOOD_RETAILER_SAM_OPTIONAL, GstFoodRetailerSamArgs, gst_food_retailer_sam,
};
mod gst_corrections;
pub use gst_corrections::{
    CreditTimeLimitArgs, ErrorCorrectionArgs, gst_credit_time_limit, gst_error_correction,
};
mod tourist_refund;
pub use tourist_refund::{TouristRefundArgs, tourist_refund};
mod deferred_tax;
pub use deferred_tax::{DeferredTaxArgs, deferred_tax};
mod deferred_revenue;
pub use deferred_revenue::{DeferredRevenueArgs, deferred_revenue};
mod impairment_test;
pub use impairment_test::{ImpairmentTestArgs, impairment_test};
mod leave_provisions;
mod wip_lockup;
pub use leave_provisions::{LeaveProvisionsArgs, leave_provisions};
pub use wip_lockup::{WipLockupArgs, wip_lockup};
mod long_service_leave;
pub use long_service_leave::{LongServiceLeaveArgs, long_service_leave};
mod foreign_income_conversion;
pub use foreign_income_conversion::foreign_income_conversion;
mod foreign_income_tax_offset;
pub use foreign_income_tax_offset::{
    FITO_OPTIONAL, ForeignIncomeTaxOffsetArgs, foreign_income_tax_offset,
};
mod minors_income;
pub use minors_income::{MINORS_OPTIONAL, MinorsIncomeArgs, minors_income};
mod smsf_tax;
pub use smsf_tax::smsf_tax;
mod ess_discount;
pub use ess_discount::{EssDiscountArgs, ess_discount};
mod help_indexation;
pub use help_indexation::{HELP_INDEXATION_OPTIONAL, HelpIndexationArgs, help_indexation};
mod defined_benefit_income_cap;
pub use defined_benefit_income_cap::{DefinedBenefitIncomeCapArgs, defined_benefit_income_cap};
mod foreign_pension_upp;
pub use foreign_pension_upp::{
    FOREIGN_PENSION_UPP_OPTIONAL, ForeignPensionUppArgs, foreign_pension_upp,
};
mod scholarship_exemption;
pub use scholarship_exemption::{ScholarshipExemptionArgs, scholarship_exemption};
mod super_crystallisation;
pub use super_crystallisation::{SuperCrystallisationArgs, super_crystallisation};
mod endorsement_review;
pub use endorsement_review::endorsement_review;

const KINDS: [&str; 149] = [
    "gst",
    "gst_margin_scheme",
    "gst_food_retailer_sam",
    "fuel_tax_credits",
    "resident_tax",
    "foreign_resident_tax",
    "capital_gains",
    "main_residence",
    "cgt_parcels",
    "borrowing_expenses",
    "borrowing_expenses_schedule",
    "hire_purchase_schedule",
    "chattel_mortgage_schedule",
    "loan_transactions",
    "business_capital_expenditure",
    "balancing_adjustment",
    "trade_in",
    "franking_account",
    "foreign_purchaser_surcharge",
    "dasp_tax",
    "super_benefit_tax",
    "transfer_duty",
    "super_withholding",
    "payroll_tax",
    "payroll_tax_group",
    "payroll_tax_monthly",
    "land_tax",
    "workers_compensation_premium",
    "motor_vehicle_duty",
    "insurance_duty",
    "portable_lsl_levy",
    "fbt",
    "fbt_car_statutory",
    "fbt_car_operating_cost",
    "fbt_ev_exemption_check",
    "fbt_instalment",
    "fbt_loan",
    "fbt_meal_entertainment",
    "fbt_meal_entertainment_register",
    "fbt_car_parking",
    "fbt_debt_waiver",
    "fbt_expense_payment",
    "fbt_property",
    "depreciation",
    "quarterly_sg",
    "payg_withholding",
    "payg_gross_up",
    "contribution_caps",
    "pension_minimum",
    "study_loan_repayment",
    "study_loan_withholding",
    "medicare_levy",
    "medicare_levy_surcharge",
    "low_income_tax_offset",
    "division_293",
    "division_296",
    "termination_payment",
    "etp_tax_free_component",
    "unused_leave_payment",
    "schedule_5_method_a",
    "schedule_5_method_b_i",
    "schedule_5_method_b_ii",
    "working_holiday_maker_withholding",
    "seniors_pensioners_withholding",
    "seniors_pensioners_tax_offset",
    "lump_sum_arrears_tax_offset",
    "beneficiary_tax_offset",
    "invalid_carer_tax_offset",
    "zone_tax_offset",
    "net_medical_expenses_tax_offset",
    "luxury_car_tax",
    "fbt_residual",
    "fbt_housing",
    "fbt_lafha",
    "fbt_board",
    "fbt_tax_exempt_body_entertainment",
    "fbt_entertainment_classifier",
    "fbt_minor_benefit_check",
    "fhss",
    "income_tests",
    "aml_cdd_review",
    "ppsr_timing",
    "wine_equalisation_tax",
    "vic_property_taxes",
    "alcohol_excise",
    "asic_fees",
    "sb_cgt_concessions",
    "work_expenses",
    "notice_period",
    "ato_interest_penalties",
    "transfer_balance",
    "excess_contributions",
    "private_health_insurance_rebate",
    "depreciation_low_value_pool",
    "depreciation_small_business_pool",
    "capital_works",
    "instant_asset_write_off",
    "small_business_income_offset",
    "non_commercial_losses",
    "psi_test",
    "pcg_2021_4",
    "government_super_amounts",
    "rental_property",
    "foreign_resident_withholding",
    "non_resident_withholding",
    "residential_vacancy_fee",
    "horticulture_shearing_withholding",
    "upward_variation_withholding",
    "daily_casual_withholding",
    "no_abn_withholding",
    "entertainer_withholding",
    "return_to_work_withholding",
    "annuity_withholding",
    "voluntary_agreement_withholding",
    "residency_tests",
    "lodgment_required",
    "rd_tax_offset",
    "esic",
    "gst_creditable_purpose_change",
    "gst_annual_apportionment",
    "gst_financial_acquisitions_threshold",
    "gst_going_concern",
    "gst_second_hand_goods",
    "gst_bad_debt",
    "gst_error_correction",
    "gst_credit_time_limit",
    "tourist_refund",
    "deferred_tax",
    "cgt_cost_base_adjustments",
    "payg_instalment",
    "leave_provisions",
    "deferred_revenue",
    "wip_lockup",
    "impairment_test",
    "long_service_leave",
    "foreign_income_conversion",
    "foreign_income_tax_offset",
    "trading_stock",
    "prepayments",
    "minors_income",
    "smsf_tax",
    "ess_discount",
    "help_indexation",
    "defined_benefit_income_cap",
    "foreign_pension_upp",
    "scholarship_exemption",
    "super_crystallisation",
    "endorsement_review",
    "first_home_owner_grant",
];

/// `periods(kind)`: accepted period arguments and inclusive calendar boundaries.
pub fn periods(kind: &str) -> Value {
    let mut out = Vec::new();
    for year in meta::supported_periods(kind) {
        if kind.starts_with("fbt") {
            let ended: i64 = year.parse().expect("FBT year");
            out.push(json!({
                "id": year,
                "basis": "FBT year",
                "start": format!("{}-04-01", ended - 1),
                "end": format!("{year}-03-31"),
                "arguments": { "year_ended": ended },
            }));
            continue;
        }
        let start: i64 = year[..4].parse().expect("income year");
        // Overwriting a key keeps its position (`preserve_order`).
        let mut period = json!({
            "id": year,
            "basis": "income year",
            "start": format!("{start}-07-01"),
            "end": format!("{}-06-30", start + 1),
            "arguments": { "year": year },
        });
        if kind == "study_loan_withholding" {
            period["start"] = json!(lookup(meta::STUDY_LOAN_WITHHOLDING_FROM, year));
        }
        if kind == "quarterly_sg" {
            period["basis"] = json!("complete quarter within income year");
            let next = start + 1;
            period["quarters"] = json!([
                {"quarter": 1, "start": format!("{start}-07-01"), "end": format!("{start}-09-30")},
                {"quarter": 2, "start": format!("{start}-10-01"), "end": format!("{start}-12-31")},
                {"quarter": 3, "start": format!("{next}-01-01"), "end": format!("{next}-03-31")},
                {"quarter": 4, "start": format!("{next}-04-01"), "end": format!("{next}-06-30")},
            ]);
        }
        out.push(period);
    }
    Value::Array(out)
}

fn example_value(value: Example) -> Value {
    match value {
        Example::Text(s) => json!(s),
        Example::Flag(b) => json!(b),
        Example::Whole(n) => json!(n),
        Example::Json(text) => serde_json::from_str(text).expect("JSON example"),
    }
}

/// The anchor of `kind`'s heading in docs/calculation-evidence.md, where
/// worksheets that share their evidence share a heading.
fn evidence_heading(kind: &str) -> String {
    match kind {
        "fbt_car_statutory" | "fbt_car_operating_cost" => "fbt-car-benefits".into(),
        "schedule_5_method_a" | "schedule_5_method_b_i" | "schedule_5_method_b_ii" => {
            "schedule-5".into()
        }
        "working_holiday_maker_withholding" => "working-holiday-makers".into(),
        "seniors_pensioners_withholding" => "seniors-and-pensioners".into(),
        "seniors_pensioners_tax_offset" => "seniors-and-pensioners-tax-offset".into(),
        "sb_cgt_concessions" => "small-business-cgt-concessions".into(),
        "ato_interest_penalties" => "ato-penalties-and-interest".into(),
        "pcg_2021_4" => "pcg-20214".into(),
        "esic" => "early-stage-investor-tax-offset".into(),
        "wip_lockup" => "wip-lock-up".into(),
        _ => kind.replace('_', "-"),
    }
}

/// `worksheet_catalogue()`: JSON-ready discovery data for every worksheet.
pub fn worksheet_catalogue() -> Value {
    let mut out = Map::new();
    for kind in KINDS {
        let supported = periods(kind);
        let fbt = kind.starts_with("fbt");
        let (checked, passage) = meta::source_review(kind);
        let mut evidence: Map<String, Value> = meta::library_evidence(kind)
            .iter()
            .map(|(k, v)| (k.to_string(), json!(v)))
            .collect();
        evidence.insert(
            "note".into(),
            json!(format!(
                "docs/calculation-evidence.md#{}",
                evidence_heading(kind)
            )),
        );
        let mut inputs = Map::new();
        inputs.insert("kind".into(), json!(kind));
        inputs.insert(
            "scope_confirmed".into(),
            json!("boolean true; establish every scope condition"),
        );
        inputs.insert(
            if fbt { "year_ended" } else { "year" }.into(),
            json!(if fbt { "integer" } else { "income-year string" }),
        );
        for (k, v) in meta::input_units(kind) {
            inputs.insert(k.into(), json!(v));
        }
        let mut facts = Map::new();
        facts.insert("kind".into(), json!(kind));
        facts.insert("scope_confirmed".into(), json!(true));
        if let Some(Value::Object(arguments)) = supported[0].get("arguments") {
            for (k, v) in arguments {
                facts.insert(k.clone(), v.clone());
            }
        }
        for (k, v) in meta::examples(kind) {
            facts.insert(k.into(), example_value(v));
        }
        let mut entry = Map::new();
        entry.insert("scope".into(), json!(lookup(texts::SCOPES, kind)));
        entry.insert("source".into(), json!(lookup(texts::SOURCES, kind)));
        entry.insert("sources".into(), json!(sources(kind)));
        entry.insert("source_checked".into(), json!(checked));
        entry.insert("source_passage".into(), json!(passage));
        if let Some(quotes) = source_quotes::catalogue(kind) {
            entry.insert("source_quotes".into(), quotes);
        }
        entry.insert("example_evidence".into(), Value::Object(evidence));
        entry.insert("supported_periods".into(), supported.clone());
        entry.insert("required_inputs".into(), Value::Object(inputs));
        let money_format = "Non-negative finite AUD decimal strings, at most 2 decimal places and 1000000000000.00. Resident taxable income must be whole dollars.";
        entry.insert(
            "money_format".into(),
            json!(if kind == "division_296" {
                format!("{money_format} Total superannuation earnings may be negative, down to -1000000000000.00.")
            } else {
                money_format.to_string()
            }),
        );
        entry.insert(
            "methods".into(),
            if kind == "depreciation" {
                json!(["prime_cost", "diminishing_value"])
            } else if kind == "work_expenses" {
                json!(work_expenses::METHODS)
            } else {
                json!(fbt_benefits::methods(kind))
            },
        );
        entry.insert(
            "example".into(),
            json!({ "synthetic": true, "facts": Value::Object(facts) }),
        );
        let optional: Map<String, Value> = meta::optional_inputs(kind)
            .into_iter()
            .map(|(k, v)| (k.into(), json!(v)))
            .collect();
        if !optional.is_empty() {
            entry.insert("optional_inputs".into(), Value::Object(optional));
        }
        out.insert(kind.into(), Value::Object(entry));
    }
    Value::Object(out)
}
