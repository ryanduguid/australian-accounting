// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `lump_sum_arrears_tax_offset`: the tax offset for lump sum payments in
//! arrears (ITAA 1936 Part III Division 17 Subdivision AB, ss 159ZR to
//! 159ZRD), from the year of receipt's taxable income and the arrears by the
//! earlier income year in which they accrued.
//!
//! The steps cite ITAA 1936 compilation 193 (1 October 2026), read on
//! 16 November 2026. Each year's tax is worked at the resident rates the ATO
//! publishes ([`meta::PRIOR_RESIDENT_TAX_SCALES`] and
//! [`meta::RESIDENT_TAX_SCALES`]), written into the working band by band.

use serde_json::{Map, Value};

use super::{
    Arg, Result, Step, d, derived_result, int, py_max, py_min, rate, refuse, scope, step,
    step_by_name, whole_dollar_money,
};
use crate::decimal::{Calc, Decimal, Rounding};
use crate::metadata as meta;

const KIND: &str = "lump_sum_arrears_tax_offset";

/// The 10% of s 159ZRA(1)(b).
const THRESHOLD_RATE: &str = "0.10";

/// An accrual year's optional facts, in the order of [`AccrualYear::facts`].
const FACTS: [&str; 4] = [
    "taxable_income",
    "net_capital_gain",
    "rebate_year_arrears",
    "earlier_rebate_arrears",
];

/// The rates applying to a recent accrual year's tax, for the authorities.
macro_rules! at_resident_rates {
    ($text:literal) => {
        concat!(
            $text,
            ", at the year's resident rates as the ATO publishes them (Income Tax Rates Act 1986 Schedule 7 Part I), with the temporary budget repair levy of 2% above $180,000 in 2014-15 to 2016-17 (Income Tax (Transitional Provisions) Act 1997 s 4-11)"
        )
    };
}

/// The recent accrual years' steps, most recent first: s 159ZRC's notional
/// tax amount and s 159ZRD's increase in normal tax, each (id, authority).
const RECENT: [[(&str, &str); 2]; 2] = [
    [
        (
            "notional_tax_most_recent_year",
            at_resident_rates!(
                "ITAA 1936 s 159ZRC: the notional tax amount for the most recent accrual year, the rebated tax on its taxable income with its annual arrears amount added less the rebated tax on its taxable income, each adjusted by paragraphs (b) to (e) for a rebate year"
            ),
        ),
        (
            "normal_tax_on_arrears_most_recent_year",
            at_resident_rates!(
                "ITAA 1936 s 159ZRD(1): increased normal tax less normal tax for the most recent accrual year, the gross tax on its normal taxable income with its annual arrears amount added less the gross tax on its normal taxable income, each adjusted by paragraphs (b) to (e) for a rebate year"
            ),
        ),
    ],
    [
        (
            "notional_tax_second_most_recent_year",
            at_resident_rates!(
                "ITAA 1936 s 159ZRC: the notional tax amount for the second most recent accrual year, the rebated tax on its taxable income with its annual arrears amount added less the rebated tax on its taxable income, each adjusted by paragraphs (b) to (e) for a rebate year"
            ),
        ),
        (
            "normal_tax_on_arrears_second_most_recent_year",
            at_resident_rates!(
                "ITAA 1936 s 159ZRD(1): increased normal tax less normal tax for the second most recent accrual year, the gross tax on its normal taxable income with its annual arrears amount added less the gross tax on its normal taxable income, each adjusted by paragraphs (b) to (e) for a rebate year"
            ),
        ),
    ],
];

/// One earlier income year in which part of the arrears accrued.
struct AccrualYear {
    label: String,
    start: i64,
    arrears: Decimal,
    /// [`FACTS`] as given.
    facts: [Option<Decimal>; 4],
}

fn invalid<T>(why: impl std::fmt::Display) -> Result<T> {
    refuse(
        "lump_sum_arrears.accrual_years_invalid",
        format!("accrual_years: {why}."),
    )
}

/// The first calendar year of an income year written like "2023-24".
fn income_year(text: &str) -> Option<i64> {
    let (start, end) = text.split_once('-')?;
    let digits = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(start, 4) || !digits(end, 2) {
        return None;
    }
    let start: i64 = start.parse().ok()?;
    (end.parse::<i64>().ok()? == (start + 1) % 100).then_some(start)
}

/// An accrual year's whole-dollar amount `key`, if given.
fn amount(fields: &Map<String, Value>, key: &str, label: &str) -> Result<Option<Decimal>> {
    let Some(given) = fields.get(key) else {
        return Ok(None);
    };
    let what = format!("{key} of {label}");
    match given.as_str().map(Decimal::parse) {
        Some(Ok(value)) => whole_dollar_money(&Arg::Decimal(value), &what).map(Some),
        _ => invalid(format!("{what} is not a decimal string")),
    }
}

fn accrual_year(item: &Value, receipt: i64) -> Result<AccrualYear> {
    let Value::Object(fields) = item else {
        return invalid("each accrual year is an object");
    };
    let year = fields.get("year").and_then(Value::as_str);
    let Some((label, start)) = year.and_then(|y| Some((y, income_year(y)?))) else {
        return invalid("each accrual year needs year, an income year such as \"2023-24\"");
    };
    let taken = |key: &str| key == "year" || key == "arrears" || FACTS.contains(&key);
    if let Some(key) = fields.keys().find(|key| !taken(key)) {
        return invalid(format!(
            "{key} is not a field of an accrual year, which takes year, arrears, {}",
            FACTS.join(", ")
        ));
    }
    if start >= receipt {
        return invalid(format!(
            "{label} is not before the year of receipt: the total arrears amount is what accrued in earlier years (ITAA 1936 s 159ZR(1))"
        ));
    }
    let Some(arrears) = amount(fields, "arrears", label)? else {
        return invalid(format!("arrears of {label} is required"));
    };
    if arrears.is_zero() {
        return invalid(format!(
            "arrears of {label} is nil: give only the years in which part of the arrears accrued"
        ));
    }
    let mut facts = [None, None, None, None];
    for (fact, key) in facts.iter_mut().zip(FACTS) {
        *fact = amount(fields, key, label)?;
    }
    Ok(AccrualYear {
        label: label.to_string(),
        start,
        arrears,
        facts,
    })
}

/// The accrual years, most recent first, each with the facts its kind takes:
/// s 159ZR(1) makes the 2 latest recent when there are 3 or more, and every
/// one recent otherwise.
fn accrual_years(value: &Arg, receipt: &str) -> Result<Vec<AccrualYear>> {
    let Arg::Json(Value::Array(items)) = value else {
        return invalid("give a list with an object for each accrual year");
    };
    if items.is_empty() {
        return invalid("give a list with an object for each accrual year");
    }
    let receipt = income_year(receipt).expect("a supported income year");
    let mut years = items
        .iter()
        .map(|item| accrual_year(item, receipt))
        .collect::<Result<Vec<_>>>()?;
    years.sort_by_key(|year| std::cmp::Reverse(year.start));
    if let Some(pair) = years.windows(2).find(|pair| pair[0].start == pair[1].start) {
        return invalid(format!(
            "{} is given more than once: add every payer's arrears for a year together",
            pair[0].label
        ));
    }
    let distant = years.len() > 2;
    for (index, year) in years.iter().enumerate() {
        let label = &year.label;
        let given = FACTS
            .iter()
            .zip(&year.facts)
            .find(|(_, fact)| fact.is_some());
        if index >= 2 {
            if let Some((key, _)) = given {
                return invalid(format!(
                    "{key} of {label} is not used: {label} is a distant accrual year, whose notional tax ITAA 1936 s 159ZRD works out from the recent accrual years' rates, so it takes year and arrears only"
                ));
            }
        } else if year.facts[0].is_none() {
            return invalid(format!(
                "taxable_income of {label} is required: {label} is a recent accrual year (ITAA 1936 s 159ZRC)"
            ));
        } else if !distant && year.facts[1].is_some() {
            return invalid(format!(
                "net_capital_gain of {label} is not used: with fewer than 3 accrual years every one is recent, and only ITAA 1936 s 159ZRD, for distant accrual years, uses normal taxable income"
            ));
        }
    }
    Ok(years)
}

/// A year's resident rates: each row's lower and upper bound and rate.
type Scale = &'static [(i64, Option<i64>, &'static str)];

/// A year's resident rates, if the worksheet holds them.
fn scale(year: &str) -> Option<Scale> {
    meta::PRIOR_RESIDENT_TAX_SCALES
        .iter()
        .chain(meta::RESIDENT_TAX_SCALES)
        .find(|(y, _)| *y == year)
        .map(|(_, scale)| *scale)
}

/// The values added in order, as a working adds them.
fn sum(values: impl IntoIterator<Item = Decimal>) -> Calc<Decimal> {
    let mut values = values.into_iter();
    let first = values.next().expect("a value");
    values.try_fold(first, |total, value| total.add(&value))
}

/// The value of the step named `id`.
fn named(values: &[(&str, Decimal)], id: &str) -> Decimal {
    let (_, value) = values.iter().find(|(name, _)| *name == id).expect("a step");
    value.clone()
}

/// The tax on `income`, written `x`, at `scale`: each row's part of the
/// income at its rate.
fn scale_tax(
    scale: &[(i64, Option<i64>, &str)],
    income: &Decimal,
    x: &str,
) -> Calc<(Decimal, String)> {
    let mut parts = Vec::new();
    let mut terms = Vec::new();
    for (low, high, rate) in scale {
        let (ceiling, top) = match high {
            None => (income.clone(), x.to_string()),
            Some(high) => (
                py_min(income.clone(), int(*high))?,
                format!("min({x}, {high})"),
            ),
        };
        parts.push(py_max(int(0), ceiling.sub(&int(*low))?)?.mul(&d(rate))?);
        terms.push(format!("max(0, {top} - {low}) x {rate}"));
    }
    Ok((sum(parts)?, terms.join(" + ")))
}

/// The tax at `scale` on `base` with `arrears` added, less the tax on `base`,
/// where `base` is written `b`.
fn tax_increase(
    scale: &[(i64, Option<i64>, &str)],
    (base, b): (&Decimal, &str),
    arrears: &Decimal,
) -> Calc<(Decimal, String)> {
    let (with, with_formula) = scale_tax(scale, &base.add(arrears)?, &format!("{b} + {arrears}"))?;
    let (without, without_formula) = scale_tax(scale, base, b)?;
    Ok((
        with.sub(&without)?,
        format!("{with_formula} - ({without_formula})"),
    ))
}

/// A recent accrual year's taxable income, or normal taxable income, adjusted
/// by s 159ZRC(b) and (c), or s 159ZRD(1)(b) and (c): less its own total
/// arrears amount as a rebate year, plus what accrued in it of an earlier
/// rebate year's.
fn adjusted(year: &AccrualYear, (income, written): (Decimal, String)) -> Calc<(Decimal, String)> {
    let (mut income, mut written) = (income, written);
    if let Some(own) = &year.facts[2] {
        income = income.sub(own)?;
        written = format!("{written} - {own}");
    }
    if let Some(earlier) = &year.facts[3] {
        income = income.add(earlier)?;
        written = format!("{written} + {earlier}");
    }
    Ok((income, written))
}

/// s 159ZRD(2) and (3): a recent accrual year's rate, the increase in normal
/// tax over its arrears to 3 decimal places, increased by 0.001 when the
/// fourth decimal place would be more than 4; and its formula on the step
/// named `increase`.
fn recent_rate(increase: (&Decimal, &str), arrears: &Decimal) -> Calc<(Decimal, String)> {
    let ten_thousandths = increase
        .0
        .div(arrears)?
        .mul(&int(10000))?
        .quantize(0, Rounding::Down)?;
    let thousandths = ten_thousandths
        .div(&int(10))?
        .add(&d("0.5"))?
        .quantize(0, Rounding::Down)?;
    Ok((
        thousandths.div(&int(1000))?,
        format!(
            "floor(floor({} / {arrears} x 10000) / 10 + 0.5) / 1000",
            increase.1
        ),
    ))
}

/// The values the steps name, in the order they were worked out.
type Values = Vec<(&'static str, Decimal)>;

/// The steps and rates once the 10% test is met.
type OffsetWorking = (Vec<Step>, Vec<(String, String)>);

/// The recent accrual years' rates, refusing a year the worksheet holds none
/// for.
fn recent_scales(recent: &[AccrualYear]) -> Result<Vec<Scale>> {
    recent
        .iter()
        .map(|year| match scale(&year.label) {
            Some(scale) => Ok(scale),
            None => refuse(
                "lump_sum_arrears.recent_year_unsupported",
                format!(
                    "{} is a recent accrual year (ITAA 1936 s 159ZR(1)), whose notional tax s 159ZRC works out at that year's rates, and this worksheet holds resident rates from {} only",
                    year.label,
                    meta::PRIOR_RESIDENT_TAX_SCALES[0].0
                ),
            ),
        })
        .collect()
}

/// s 159ZRD(1): each recent accrual year's increase in normal tax, as a step,
/// and its rate with the rate's formula.
fn recent_rates(
    recent: &[AccrualYear],
    scales: &[Scale],
    steps: &mut Vec<Step>,
    values: &mut Values,
) -> Calc<Vec<(Decimal, String)>> {
    let mut recent_rates = Vec::new();
    for ((year, scale), [_, (id, authority)]) in recent.iter().zip(scales).zip(&RECENT) {
        let income = year.facts[0].clone().expect("validated");
        let normal = match &year.facts[1] {
            Some(gain) => (
                py_max(int(0), income.sub(gain)?)?,
                format!("max(0, {income} - {gain})"),
            ),
            None => (income.clone(), income.to_string()),
        };
        let base = adjusted(year, normal)?;
        let (increase, formula) = tax_increase(scale, (&base.0, &base.1), &year.arrears)?;
        steps.push(step(id, authority, &formula, &[], increase.clone()));
        values.push((id, increase.clone()));
        recent_rates.push(recent_rate((&increase, id), &year.arrears)?);
    }
    Ok(recent_rates)
}

/// s 159ZRD for the distant accrual years: each recent accrual year's
/// increase in normal tax and its rate, then the distant years' notional tax
/// at the average rate; and the rates.
fn distant_steps(
    recent: &[AccrualYear],
    scales: &[Scale],
    distant: &[AccrualYear],
    steps: &mut Vec<Step>,
    values: &mut Values,
) -> Calc<Vec<(String, String)>> {
    let recent_rates = recent_rates(recent, scales, steps, values)?;
    let average = recent_rates[0].0.add(&recent_rates[1].0)?;
    let arrears: Vec<String> = distant.iter().map(|y| y.arrears.to_string()).collect();
    let amount = sum(distant.iter().map(|y| y.arrears.clone()))?
        .mul(&average)?
        .div(&int(2))?;
    let id = "notional_tax_distant_accrual_years";
    steps.push(step_by_name(
        id,
        "ITAA 1936 s 159ZRD: the notional tax amounts for the distant accrual years, their annual arrears amounts at the average tax rate on recent arrears, the average of the recent accrual years' rates, each their increased normal tax less normal tax over their annual arrears amount, to 3 decimal places (s 159ZRD(2)) and increased by 0.001 when the fourth decimal place would be more than 4 (s 159ZRD(3))",
        &format!(
            "({}) x ({} + {}) / 2",
            arrears.join(" + "),
            recent_rates[0].1,
            recent_rates[1].1
        ),
        values,
        amount.clone(),
    ));
    values.push((id, amount));
    Ok(vec![
        rate("tax_rate_on_arrears_most_recent_year", &recent_rates[0].0),
        rate(
            "tax_rate_on_arrears_second_most_recent_year",
            &recent_rates[1].0,
        ),
        rate("average_tax_rate_on_recent_arrears", average.div(&int(2))?),
    ])
}

/// s 159ZRB's tax on arrears in the year of receipt, as a step.
fn tax_on_arrears(income: &Decimal, receipt: &str, values: &mut Values) -> Calc<(Step, Decimal)> {
    let total = named(values, "total_arrears_amount");
    let receipt_scale = scale(receipt).expect("a supported year of receipt");
    let (with, with_formula) = scale_tax(receipt_scale, income, "taxable_income")?;
    let reduced = "taxable_income - total_arrears_amount";
    let (without, without_formula) = scale_tax(receipt_scale, &income.sub(&total)?, reduced)?;
    let tax_on_arrears = with.sub(&without)?;
    let step = step_by_name(
        "tax_on_arrears",
        at_resident_rates!(
            "ITAA 1936 s 159ZRB, tax on arrears: the rebated tax on the taxable income of the year of receipt less the rebated tax on that taxable income reduced by the total arrears amount"
        ),
        &format!("{with_formula} - ({without_formula})"),
        values,
        tax_on_arrears.clone(),
    );
    values.push(("tax_on_arrears", tax_on_arrears.clone()));
    Ok((step, tax_on_arrears))
}

/// The steps once the 10% test is met: the tax on the arrears, each recent
/// accrual year's notional tax, the distant accrual years', their total and
/// the offset.
fn offset_steps(
    income: &Decimal,
    receipt: &str,
    years: &[AccrualYear],
    values: &mut Values,
) -> Result<OffsetWorking> {
    let (first, tax_on_arrears) = self::tax_on_arrears(income, receipt, values)?;
    let mut steps = vec![first];
    let (recent, distant) = years.split_at(years.len().min(2));
    let scales = recent_scales(recent)?;
    let mut notional = Vec::new();
    for ((year, scale), [(id, authority), _]) in recent.iter().zip(&scales).zip(&RECENT) {
        let income = year.facts[0].clone().expect("validated");
        let base = adjusted(year, (income.clone(), income.to_string()))?;
        let (amount, formula) = tax_increase(scale, (&base.0, &base.1), &year.arrears)?;
        steps.push(step(id, authority, &formula, &[], amount.clone()));
        values.push((id, amount));
        notional.push(*id);
    }
    let rates = if distant.is_empty() {
        Vec::new()
    } else {
        notional.push("notional_tax_distant_accrual_years");
        distant_steps(recent, &scales, distant, &mut steps, values)?
    };
    let notional_tax = sum(notional.iter().map(|id| named(values, id)))?;
    steps.push(step_by_name(
        "notional_tax_on_arrears",
        "ITAA 1936 s 159ZRB, notional tax on arrears: the total of the notional tax amounts for the accrual years",
        &notional.join(" + "),
        values,
        notional_tax.clone(),
    ));
    values.push(("notional_tax_on_arrears", notional_tax.clone()));
    steps.push(step_by_name(
        "lump_sum_arrears_tax_offset",
        "ITAA 1936 ss 159ZRA(1) and 159ZRB: tax on arrears less notional tax on arrears, the rebate being the amount, if any, so calculated",
        "max(0, tax_on_arrears - notional_tax_on_arrears)",
        values,
        py_max(int(0), tax_on_arrears.sub(&notional_tax)?)?,
    ));
    Ok((steps, rates))
}

/// The total arrears amount's step (id, authority).
const TOTAL: (&str, &str) = (
    "total_arrears_amount",
    "ITAA 1936 s 159ZR(1): the total arrears amount, the eligible lump sums to the extent they accrued in earlier years, the total of the annual arrears amounts",
);

/// The steps of s 159ZRA(1)(b)'s 10% test, from the year of receipt's taxable
/// income and net capital gain and the accrual years, and the values they
/// name.
fn threshold_steps(
    income: &Decimal,
    gain: &Decimal,
    years: &[AccrualYear],
) -> Calc<(Vec<Step>, Values)> {
    let total = sum(years.iter().map(|y| y.arrears.clone()))?;
    let normal = py_max(int(0), income.sub(gain)?)?;
    let remaining = py_max(int(0), normal.sub(&total)?)?;
    let threshold = remaining.mul(&d(THRESHOLD_RATE))?;
    let arrears: Vec<String> = years.iter().map(|y| y.arrears.to_string()).collect();
    let mut values = vec![
        ("taxable_income", income.clone()),
        ("net_capital_gain", gain.clone()),
        ("threshold_rate", d(THRESHOLD_RATE)),
        ("total_arrears_amount", total.clone()),
        ("normal_taxable_income", normal.clone()),
        ("threshold_test_income", remaining.clone()),
    ];
    let steps = vec![
        step(TOTAL.0, TOTAL.1, &arrears.join(" + "), &[], total),
        step_by_name(
            "normal_taxable_income",
            "ITAA 1936 s 159ZR(1), normal taxable income: the taxable income as if no net capital gain were included in assessable income under ITAA 1997 s 102-5",
            "max(0, taxable_income - net_capital_gain)",
            &values,
            normal,
        ),
        step_by_name(
            "threshold_test_income",
            "ITAA 1936 s 159ZRA(1)(b): the amount, if any, remaining after deducting the total arrears amount from normal taxable income",
            "max(0, normal_taxable_income - total_arrears_amount)",
            &values,
            remaining,
        ),
        step_by_name(
            "ten_percent_of_threshold_test_income",
            "ITAA 1936 s 159ZRA(1)(b): the total arrears amount must be not less than 10% of that remaining amount",
            "threshold_test_income x threshold_rate",
            &values,
            threshold.clone(),
        ),
    ];
    values.push(("ten_percent_of_threshold_test_income", threshold));
    Ok((steps, values))
}

/// The lump sum payment in arrears tax offset (ITAA 1936 ss 159ZR to 159ZRD)
/// for an individual's year of receipt.
pub fn lump_sum_arrears_tax_offset(
    taxable_income: &Arg,
    net_capital_gain: &Arg,
    accrual_years: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let income = whole_dollar_money(taxable_income, "taxable income")?;
    let gain = whole_dollar_money(net_capital_gain, "net capital gain")?;
    let years = self::accrual_years(accrual_years, &year)?;
    let (steps, rates) = super::precise(|| {
        let (mut steps, mut values) = threshold_steps(&income, &gain, &years)?;
        let mut rates = vec![rate("threshold_rate", THRESHOLD_RATE)];
        let total = named(&values, "total_arrears_amount");
        if total.lt(&named(&values, "ten_percent_of_threshold_test_income"))? {
            steps.push(step(
                "lump_sum_arrears_tax_offset",
                "ITAA 1936 s 159ZRA(1)(b): the total arrears amount is less than 10% of normal taxable income less that amount, so no rebate",
                "0",
                &[],
                int(0),
            ));
            return Ok((steps, rates));
        }
        let (more, recent_rates) = offset_steps(&income, &year, &years, &mut values)?;
        steps.extend(more);
        rates.extend(recent_rates);
        Ok((steps, rates))
    })?;
    derived_result(KIND, &year, steps, rates)
}
