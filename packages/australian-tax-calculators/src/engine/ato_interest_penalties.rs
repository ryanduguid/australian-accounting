// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `ato_interest_penalties`: the general interest charge (TAA 1953 ss 8AAC
//! and 8AAD; ITAA 1997 s 5-15), the shortfall interest charge (TAA Sch 1
//! ss 280-100 and 280-105), the failure to lodge on time penalty (s 286-80),
//! the false or misleading statement and not reasonably arguable penalties
//! (ss 284-85, 284-90, 284-220, 284-224 and 284-225), and the interest the
//! Commissioner pays on early payments and overpayments (Taxation (Interest on
//! Overpayments and Early Payments) Act 1983 ss 8A to 8C, 9, 10, 12D and 12E),
//! each before any remission. Read on 22 November 2026: the sections at the
//! ATO's legal database; the Overpayments Act, compilation 58 (14 October
//! 2024), at the Federal Register of Legislation; Crimes Act 1914 s 4AA; the
//! ATO's GIC, SIC, credit interest and penalty unit pages; PS LA 2006/8,
//! 2011/19, 2011/23 and 2012/5; MT 2008/2 and MT 2012/3.

use num_bigint::BigUint;
use serde_json::{Map, Value, json};

mod penalties;
mod rates;

use super::{Arg, Result, d, rate, refuse, result, scope, stated};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "ato_interest_penalties";
const QUESTIONS: [&str; 6] = [
    "gic",
    "sic",
    "failure_to_lodge",
    "statement_penalty",
    "early_payment_interest",
    "overpayment_interest",
];

const GIC_RATE: &str = "TAA 1953 s 8AAD(1): the general interest charge rate for a day is the base interest rate for the day plus 7 percentage points, divided by the number of days in the calendar year; the annual rate for each quarter as the ATO publishes it ('General interest charge (GIC) rates', last updated 4 September 2026)";
const GIC_DAYS: &str = "ITAA 1997 s 5-15 and TAA 1953 s 8AAC(3): GIC for each day from the start of the due day to the end of the last day at whose end any of the amount or its GIC remains unpaid, on the unpaid amount plus unpaid GIC from previous days, so it compounds daily; a payment on a day is applied before that day's charge; worked exactly and rounded half up to the cent once";
const SIC_RATE: &str = "TAA 1953 Sch 1 s 280-105(2): the base interest rate plus 3 percentage points, divided by the number of days in the calendar year; the annual rate for each quarter as the ATO publishes it ('Shortfall interest charge (SIC) rates', last updated 4 September 2026)";
const SIC_DAYS: &str = "TAA 1953 Sch 1 ss 280-100(1) and (2) and 280-105(1): SIC on the additional income tax for each day from the start of the day the first assessment's tax was due (or the earlier amended assessment's, for a reinstated liability, s 280-100(3)) to the end of the day before notice of the amended assessment, on the additional tax plus SIC from previous days; paying early does not stop it (PS LA 2006/8 paras 75 to 78); worked exactly and rounded half up to the cent once";
const CREDIT_RATE: &str = "Taxation (Interest on Overpayments and Early Payments) Act 1983 ss 8C and 10(1)(b): the base interest rate (TAA 1953 s 8AAD(2)), which is the published GIC rate less 7 points (s 8AAD(1)); the ATO's 'Credit interest rates and calculation' (QC58284, last updated 19 June 2026) publishes the same rates from 1 July 2023 to 30 September 2026";
const CREDIT_SUM: &str = "The ATO's 'Credit interest rates and calculation': (A / B) x C x (D / 100), simple interest (PS LA 2011/23 para 4), applied day by day so each day takes its quarter's rate and its year's length; rounded to the nearest cent, half a cent up (s 12D); under 50 cents is not payable (s 12E)";
const EARLY: &str = "Overpayments Act s 8A(1): interest on a payment of income tax, SIC, a compulsory study loan repayment amount or an FS assessment debt made more than 14 days before the day it is due and payable; s 8B(1): from the start of the later of the payment day and the notice day (the payment day for a full self-assessment taxpayer) to the end of the due day";
const OVERPAID: &str = "Overpayments Act s 9(1): interest on relevant tax paid and overpaid as a result of a decision to which the Act applies, then refunded or applied; instalments are separate amounts and a partial refund is attributed to them in reverse order of payment (s 9(2) and (3)); s 10(1)(a): from the later of the day the notice of the assessment, determination or decision was issued and the day the amount was paid, to the day it was refunded or applied";
const REMISSION: &str = "Before remission: the Commissioner may remit GIC (TAA 1953 s 8AAG), SIC (TAA Sch 1 s 280-160) and administrative penalties (s 298-20), a discretion this command does not estimate";

/// The question's facts: every field beside `question`, `year` and
/// `scope_confirmed`.
pub(super) struct Facts<'a> {
    question: &'static str,
    map: &'a Map<String, Value>,
}

impl Facts<'_> {
    fn item(&self) -> String {
        format!("question {}", self.question)
    }

    fn only(&self, takes: &[&str]) -> Result<()> {
        stated::known_keys(self.map, &self.item(), self.question, takes)
    }

    fn given(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    fn value(&self, key: &str) -> Result<&Value> {
        stated::required(self.map, &self.item(), key)
    }

    fn cents(&self, key: &str) -> Result<u64> {
        to_cents(&stated::amount(self.map, &self.item(), key)?)
    }

    fn date(&self, key: &str) -> Result<i64> {
        date(self.value(key)?, &self.item(), key)
    }

    fn flag(&self, key: &str) -> Result<bool> {
        flag(self.value(key)?, &self.item(), key)
    }

    fn choice(&self, key: &str, choices: &[&'static str]) -> Result<&'static str> {
        choice(self.value(key)?, &self.item(), key, choices)
    }

    fn list(&self, key: &str, most: usize) -> Result<&Vec<Value>> {
        match self.value(key)? {
            Value::Array(items) if !items.is_empty() && items.len() <= most => Ok(items),
            _ => fact_invalid(format!(
                "{}: {key} must be a list of 1 to {most} objects.",
                self.item()
            )),
        }
    }
}

fn fact_invalid<T>(message: String) -> Result<T> {
    refuse("ato_interest_penalties.fact_invalid", message)
}

/// A whole number of cents from a checked amount (at most 2 decimal places).
fn to_cents(value: &Decimal) -> Result<u64> {
    let text = value.quantize(-2, Rounding::HalfUp)?.to_string();
    Ok(text
        .replace('.', "")
        .parse()
        .expect("cents of a checked amount"))
}

fn dollars(cents: u128) -> Decimal {
    d(&format!("{}.{:02}", cents / 100, cents % 100))
}

fn date(value: &Value, item: &str, key: &str) -> Result<i64> {
    match value.as_str().and_then(dates::parse) {
        Some(day) => Ok(day),
        None => fact_invalid(format!("{item}: {key} must be a date YYYY-MM-DD.")),
    }
}

fn flag(value: &Value, item: &str, key: &str) -> Result<bool> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => refuse(
            "input.flag_invalid",
            format!("{item}: {key} must be a boolean."),
        ),
    }
}

fn choice(value: &Value, item: &str, key: &str, choices: &[&'static str]) -> Result<&'static str> {
    match choices.iter().find(|c| value.as_str() == Some(**c)) {
        Some(found) => Ok(found),
        None => fact_invalid(format!(
            "{item}: {key} must be one of {}.",
            choices.join(", ")
        )),
    }
}

/// The financial year a day falls in, as `2026-27`.
fn financial_year(day: i64) -> String {
    let (y, m, _) = dates::to_civil(day);
    let start = if m >= 7 { y } else { y - 1 };
    format!("{start}-{:02}", (start + 1) % 100)
}

/// `year` must be the financial year of the question's end date `key`.
fn in_year(day: i64, period: &str, key: &str) -> Result<()> {
    let found = financial_year(day);
    if found == period {
        return Ok(());
    }
    refuse(
        "ato_interest_penalties.year_mismatch",
        format!(
            "{key} ({}) is in {found}; year must be the financial year of {key}.",
            dates::format(day)
        ),
    )
}

fn date_order<T>(message: String) -> Result<T> {
    refuse("ato_interest_penalties.date_order", message)
}

fn unpublished<T>(message: String) -> Result<T> {
    refuse("ato_interest_penalties.rate_unpublished", message)
}

/// Hundredths of 1% as a percentage string: 1151 is 11.51.
fn percent(hundredths: u32) -> String {
    format!("{}.{:02}", hundredths / 100, hundredths % 100)
}

fn year_length(day: i64) -> u32 {
    let (y, _, _) = dates::to_civil(day);
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    if leap { 366 } else { 365 }
}

/// The first day of `day`'s calendar quarter.
fn quarter_start(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    let first = (m - 1) / 3 * 3 + 1;
    dates::parse(&format!("{y:04}-{first:02}-01")).expect("quarter start")
}

/// Consecutive days at one annual rate within one calendar quarter.
struct Stretch {
    from: i64,
    to: i64,
    rate: u32,
    assumed: bool,
}

fn add_day(stretches: &mut Vec<Stretch>, day: i64, rate: u32, assumed: bool) {
    match stretches.last_mut() {
        Some(last)
            if last.to + 1 == day
                && last.rate == rate
                && last.assumed == assumed
                && quarter_start(last.from) == quarter_start(day) =>
        {
            last.to = day;
        }
        _ => stretches.push(Stretch {
            from: day,
            to: day,
            rate,
            assumed,
        }),
    }
}

fn stretches_json(stretches: &[Stretch]) -> Value {
    Value::Array(
        stretches
            .iter()
            .map(|s| {
                let mut out = json!({
                    "from": dates::format(s.from),
                    "to": dates::format(s.to),
                    "days": s.to - s.from + 1,
                    "annual_rate_percent": percent(s.rate),
                });
                if s.assumed {
                    out["assumed"] = json!(true);
                }
                out
            })
            .collect(),
    )
}

/// Half up on a non-negative exact ratio, in cents.
fn half_up(numerator: &BigUint, denominator: &BigUint) -> u128 {
    let two = BigUint::from(2_u8);
    ((&two * numerator + denominator) / (&two * denominator))
        .try_into()
        .expect("an amount within u128 cents")
}

/// A day's annual rate in hundredths of 1% and whether it was assumed.
type RateOf<'a> = &'a dyn Fn(i64) -> Result<(u32, bool)>;

/// The exact balance at the end of `last` of `amount` cents unpaid from the
/// start of `first`, charged daily at `rate_of`, with each payment applied on
/// its day before that day's charge, as a ratio and the stretches of days.
fn compound(
    rate_of: RateOf,
    amount: u64,
    first: i64,
    last: i64,
    payments: &[(i64, u64)],
) -> Result<(BigUint, BigUint, Vec<Stretch>)> {
    let (mut numerator, mut denominator) = (BigUint::from(amount), BigUint::from(1_u8));
    let mut stretches = Vec::new();
    for day in first..=last {
        for (_, paid) in payments.iter().filter(|(on, _)| *on == day) {
            let owed = BigUint::from(*paid) * &denominator;
            if owed > numerator {
                return refuse(
                    "ato_interest_penalties.overpaid",
                    format!(
                        "The payment of {} on {} is more than the amount owing that day.",
                        dollars(u128::from(*paid)),
                        dates::format(day)
                    ),
                );
            }
            numerator -= owed;
        }
        let (rate, assumed) = rate_of(day)?;
        let base = 10_000 * year_length(day);
        numerator *= base + rate;
        denominator *= base;
        add_day(&mut stretches, day, rate, assumed);
    }
    Ok((numerator, denominator, stretches))
}

/// Every day's denominator: 10,000 (hundredths of 1% as a fraction) x 365 x 366.
const SIMPLE_DENOMINATOR: u128 = 10_000 * 365 * 366;

/// Simple interest on `amount` cents from the start of `first` to the end of
/// `last` at the base interest rate, as a numerator over `SIMPLE_DENOMINATOR`
/// in cents, and the stretches of days.
fn simple(amount: u64, first: i64, last: i64) -> Result<(u128, Vec<Stretch>)> {
    let (mut sum, mut stretches) = (0_u128, Vec::new());
    for day in first..=last {
        let Some(rate) = rates::base(day) else {
            return unpublished(format!(
                "No base interest rate is published for {}: the rates run from 1 July 1999 to {}.",
                dates::format(day),
                rates::TABLE_END
            ));
        };
        // rate / (10,000 x days), over 10,000 x 365 x 366.
        let other = if year_length(day) == 365 { 366 } else { 365 };
        sum += u128::from(rate) * other;
        add_day(&mut stretches, day, rate, false);
    }
    Ok((u128::from(amount) * sum, stretches))
}

fn simple_cents(numerator: u128) -> u128 {
    (2 * numerator + SIMPLE_DENOMINATOR) / (2 * SIMPLE_DENOMINATOR)
}

fn remission_note(out: &mut Value) {
    out["remission"] = json!(REMISSION);
}

/// GIC payments: a list of `{date, amount}` from the due day to `to`.
fn payments(f: &Facts, due: i64, to: i64) -> Result<Vec<(i64, u64)>> {
    if !f.given("payments") {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for (i, item) in f.list("payments", 1000)?.iter().enumerate() {
        let name = format!("payment {}", i + 1);
        let Value::Object(fields) = item else {
            return fact_invalid(format!("{name} must be an object."));
        };
        stated::known_keys(fields, &name, "a payment", &["date", "amount"])?;
        let day = date(stated::required(fields, &name, "date")?, &name, "date")?;
        if day < due || day > to {
            return date_order(format!(
                "{name}: date must be from due_date to to: a payment after to does not change the GIC to then."
            ));
        }
        out.push((day, to_cents(&stated::amount(fields, &name, "amount")?)?));
    }
    Ok(out)
}

/// An optional assumed annual rate for days after the published quarters: a
/// percentage above 0 and at most 100 with at most 2 decimal places.
fn assumed_rate(f: &Facts) -> Result<Option<u32>> {
    if !f.given("assumed_annual_rate") {
        return Ok(None);
    }
    let cents = f.cents("assumed_annual_rate")?;
    match u32::try_from(cents) {
        Ok(rate) if rate > 0 && rate <= 10_000 => Ok(Some(rate)),
        _ => fact_invalid(format!(
            "{}: assumed_annual_rate must be a percentage above 0 and at most 100.",
            f.item()
        )),
    }
}

fn gic(f: &Facts, period: &str) -> Result<Value> {
    f.only(&[
        "amount",
        "due_date",
        "to",
        "payments",
        "assumed_annual_rate",
    ])?;
    let amount = f.cents("amount")?;
    let (due, to) = (f.date("due_date")?, f.date("to")?);
    in_year(to, period, "to")?;
    if to < due {
        return date_order(format!(
            "{}: to must be on or after due_date, the first day GIC accrues.",
            f.item()
        ));
    }
    if due < rates::first_day(&rates::GIC) {
        return unpublished("GIC rates are published from 1 July 1999.".into());
    }
    let assumed = assumed_rate(f)?;
    let paid = payments(f, due, to)?;
    let end = rates::table_end();
    let rate_of = |day: i64| match (rates::gic(day), assumed) {
        (Some(rate), _) => Ok((rate, false)),
        (None, Some(rate)) if day > end => Ok((rate, true)),
        _ => unpublished(format!(
            "The ATO has published GIC rates to {}; give assumed_annual_rate to project {} and later.",
            rates::TABLE_END,
            dates::format(day)
        )),
    };
    let (numerator, denominator, stretches) = compound(&rate_of, amount, due, to, &paid)?;
    let balance = half_up(&numerator, &denominator);
    let total_paid: u128 = paid.iter().map(|(_, cents)| u128::from(*cents)).sum();
    let charge = balance + total_paid - u128::from(amount);
    let assumed_days: i64 = stretches
        .iter()
        .filter(|s| s.assumed)
        .map(|s| s.to - s.from + 1)
        .sum();
    let mut out = result(
        KIND,
        period,
        vec![
            ("gic", dollars(charge)),
            ("paid", dollars(total_paid)),
            ("balance", dollars(balance)),
        ],
        assumed
            .map(|r| rate("assumed_annual_rate_percent", percent(r)))
            .into_iter()
            .collect(),
    )?;
    out["question"] = json!("gic");
    out["days"] = json!(to - due + 1);
    out["assumed_days"] = json!(assumed_days);
    out["stretches"] = stretches_json(&stretches);
    out["rate_authority"] = json!(GIC_RATE);
    out["gic_authority"] = json!(GIC_DAYS);
    remission_note(&mut out);
    if assumed_days > 0 {
        warn(
            &mut out,
            format!(
                "{assumed_days} days after {} use the assumed rate: the GIC for them is an estimate, not a published rate.",
                rates::TABLE_END
            ),
        );
    }
    Ok(out)
}

fn sic(f: &Facts, period: &str) -> Result<Value> {
    f.only(&["amount", "due_date", "notice_date"])?;
    let amount = f.cents("amount")?;
    let (due, notice) = (f.date("due_date")?, f.date("notice_date")?);
    in_year(notice, period, "notice_date")?;
    let first = rates::first_day(&rates::SIC);
    if due < first && notice > due {
        return unpublished("SIC rates are published from 1 April 2005.".into());
    }
    let rate_of = |day: i64| match rates::sic(day) {
        Some(rate) => Ok((rate, false)),
        None => unpublished(format!(
            "The ATO has published SIC rates to {}; {} is later.",
            rates::TABLE_END,
            dates::format(day)
        )),
    };
    let (numerator, denominator, stretches) = compound(&rate_of, amount, due, notice - 1, &[])?;
    let charge = half_up(&numerator, &denominator) - u128::from(amount);
    let mut out = result(KIND, period, vec![("sic", dollars(charge))], Vec::new())?;
    out["question"] = json!("sic");
    out["days"] = json!((notice - due).max(0));
    if notice > due {
        out["first_day"] = json!(dates::format(due));
        out["last_day"] = json!(dates::format(notice - 1));
    }
    out["stretches"] = stretches_json(&stretches);
    out["rate_authority"] = json!(SIC_RATE);
    out["sic_authority"] = json!(SIC_DAYS);
    remission_note(&mut out);
    Ok(out)
}

/// The liabilities s 8A(1)(a) lists that the command takes.
const LIABILITIES: [&str; 4] = [
    "income_tax",
    "shortfall_interest_charge",
    "compulsory_study_loan_repayment",
    "fs_assessment_debt",
];

fn early_payment_interest(f: &Facts, period: &str) -> Result<Value> {
    let fsa = f.flag("full_self_assessment")?;
    let mut takes = vec![
        "amount",
        "liability",
        "paid_on",
        "due_date",
        "full_self_assessment",
    ];
    if !fsa {
        takes.push("notice_issued");
    }
    f.only(&takes)?;
    let liability = f.value("liability")?;
    if !LIABILITIES.iter().any(|l| liability.as_str() == Some(*l)) {
        return refuse(
            "ato_interest_penalties.liability_not_listed",
            format!(
                "{}: liability must be one Overpayments Act s 8A(1)(a) lists: {}. Other amounts, such as GST or PAYG instalments, earn no early payment interest under s 8A.",
                f.item(),
                LIABILITIES.join(", ")
            ),
        );
    }
    let amount = f.cents("amount")?;
    let (paid, due) = (f.date("paid_on")?, f.date("due_date")?);
    in_year(due, period, "due_date")?;
    let start = if fsa {
        paid
    } else {
        paid.max(f.date("notice_issued")?)
    };
    let early = due - paid;
    let entitled = early > 14;
    let (numerator, stretches) = if entitled && start <= due {
        simple(amount, start, due)?
    } else {
        (0, Vec::new())
    };
    let interest = simple_cents(numerator);
    let payable = if interest >= 50 { interest } else { 0 };
    let mut out = result(
        KIND,
        period,
        vec![
            ("interest", dollars(interest)),
            ("interest_payable", dollars(payable)),
        ],
        Vec::new(),
    )?;
    out["question"] = json!("early_payment_interest");
    out["days_early"] = json!(early);
    out["entitled"] = json!(entitled);
    if entitled && start <= due {
        out["first_day"] = json!(dates::format(start));
        out["last_day"] = json!(dates::format(due));
        out["days"] = json!(due - start + 1);
    } else {
        out["days"] = json!(0);
    }
    out["stretches"] = stretches_json(&stretches);
    out["entitlement_authority"] = json!(EARLY);
    out["rate_authority"] = json!(CREDIT_RATE);
    out["interest_authority"] = json!(CREDIT_SUM);
    if !entitled {
        warn(
            &mut out,
            format!(
                "The payment was {early} days before the due day, not more than 14 (s 8A(1)(b)): no interest."
            ),
        );
    }
    Ok(out)
}

fn overpayment_interest(f: &Facts, period: &str) -> Result<Value> {
    f.only(&[
        "payments",
        "refunded",
        "refunded_on",
        "notice_issued",
        "relevant_tax",
        "decision_to_which_act_applies",
    ])?;
    if !f.flag("relevant_tax")? || !f.flag("decision_to_which_act_applies")? {
        return refuse(
            "ato_interest_penalties.not_entitled",
            format!(
                "{}: Part III interest is payable only on relevant tax (s 3C) overpaid as a result of a decision to which the Act applies (s 3(1)); this command does not work other credit interest.",
                f.item()
            ),
        );
    }
    let (refunded, refunded_on) = (f.cents("refunded")?, f.date("refunded_on")?);
    let notice = f.date("notice_issued")?;
    in_year(refunded_on, period, "refunded_on")?;
    let mut paid = Vec::new();
    for (i, item) in f.list("payments", 1000)?.iter().enumerate() {
        let name = format!("payment {}", i + 1);
        let Value::Object(fields) = item else {
            return fact_invalid(format!("{name} must be an object."));
        };
        stated::known_keys(fields, &name, "a payment", &["paid_on", "amount"])?;
        let day = date(
            stated::required(fields, &name, "paid_on")?,
            &name,
            "paid_on",
        )?;
        paid.push((day, to_cents(&stated::amount(fields, &name, "amount")?)?));
    }
    if refunded > paid.iter().map(|(_, c)| *c).sum::<u64>() {
        return refuse(
            "ato_interest_penalties.overpaid",
            format!("{}: refunded is more than the payments.", f.item()),
        );
    }
    // s 9(3): the refund goes to the instalments in reverse order of payment.
    paid.sort_by_key(|(day, _)| *day);
    let (mut left, mut total, mut parts) = (refunded, 0_u128, Vec::new());
    for (day, cents) in paid.iter().rev() {
        if left == 0 {
            break;
        }
        let share = left.min(*cents);
        left -= share;
        let from = notice.max(*day);
        if from > refunded_on {
            return date_order(format!(
                "{}: refunded_on is before a payment it refunds or the notice day.",
                f.item()
            ));
        }
        let (numerator, stretches) = simple(share, from, refunded_on)?;
        total += numerator;
        parts.push(json!({
            "paid_on": dates::format(*day),
            "attributed": dollars(u128::from(share)).to_string(),
            "first_day": dates::format(from),
            "last_day": dates::format(refunded_on),
            "days": refunded_on - from + 1,
            "stretches": stretches_json(&stretches),
        }));
    }
    let interest = simple_cents(total);
    let payable = if interest >= 50 { interest } else { 0 };
    let mut out = result(
        KIND,
        period,
        vec![
            ("refunded", dollars(u128::from(refunded))),
            ("interest", dollars(interest)),
            ("interest_payable", dollars(payable)),
        ],
        Vec::new(),
    )?;
    out["question"] = json!("overpayment_interest");
    out["attribution"] = Value::Array(parts);
    out["entitlement_authority"] = json!(OVERPAID);
    out["rate_authority"] = json!(CREDIT_RATE);
    out["interest_authority"] = json!(CREDIT_SUM);
    Ok(out)
}

pub(super) fn warn(out: &mut Value, text: String) {
    out["warnings"]
        .as_array_mut()
        .expect("warnings")
        .push(json!(text));
}

/// The command: `question` picks the charge, penalty or interest.
pub fn ato_interest_penalties(
    question: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    facts: &Map<String, Value>,
) -> Result<Value> {
    let period = scope(scope_confirmed, year, KIND)?;
    let question = match question.text() {
        Some("remission") => {
            return refuse(
                "ato_interest_penalties.remission_discretionary",
                "Remission of GIC (TAA 1953 s 8AAG), SIC (TAA Sch 1 s 280-160) and administrative penalties (s 298-20) is the Commissioner's discretion; every amount this command gives is before remission, and it does not estimate one.",
            );
        }
        Some("scheme_penalty") => {
            return refuse(
                "ato_interest_penalties.scheme_penalty",
                "Scheme penalties (TAA Sch 1 Subdiv 284-C) turn on the scheme's dominant purpose and whether the position was reasonably arguable; this command does not work them.",
            );
        }
        given => match QUESTIONS.iter().find(|q| given == Some(**q)) {
            Some(found) => *found,
            None => {
                return refuse(
                    "ato_interest_penalties.question_invalid",
                    format!("question must be one of {}.", QUESTIONS.join(", ")),
                );
            }
        },
    };
    let f = Facts {
        question,
        map: facts,
    };
    match question {
        "gic" => gic(&f, &period),
        "sic" => sic(&f, &period),
        "failure_to_lodge" => penalties::failure_to_lodge(&f, &period),
        "statement_penalty" => penalties::statement_penalty(&f, &period),
        "early_payment_interest" => early_payment_interest(&f, &period),
        _ => overpayment_interest(&f, &period),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ATO's Evan example ('Credit interest rates and calculation'):
    /// $1,926 for 19 days at 2% in a 365-day year is $2.01.
    #[test]
    fn the_credit_formula_gives_evans_interest() {
        let numerator = 192_600_u128 * 19 * 200 * 366;
        assert_eq!(simple_cents(numerator), 201);
    }

    #[test]
    fn a_day_is_in_its_financial_year() {
        let day = |s| dates::parse(s).unwrap();
        assert_eq!(financial_year(day("2026-06-30")), "2025-26");
        assert_eq!(financial_year(day("2026-07-01")), "2026-27");
        assert_eq!(financial_year(day("2099-07-01")), "2099-00");
    }
}
