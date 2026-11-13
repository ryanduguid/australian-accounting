// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The minimum yearly repayment: ITAA 1936 s 109E(5) and (6).
//!
//! The operator supplies the remaining term as (a) minus (b); the engine
//! applies the statutory rounding up. It refuses the year the loan was made,
//! a loan not on s 109N terms, and any question of which payments count
//! (s 109R). A shortfall is a review aid, not the dividend.

use serde_json::{Value, json};

use crate::Failure;
use crate::decimal::{Calc, Decimal, Rounding, with_precision};
use crate::facts::{optional_money, optional_year_of_income, optional_years, text_cell};
use crate::gate::GateResult;
use crate::money::{FORMULA_PRECISION, ROUNDING, cents_str, d, to_cents};
use crate::rates::{BenchmarkTable, RateOverride, RateResult, benchmark_rate};
use crate::raw::Raw;
use crate::years::YearOfIncome;

/// The parts of a gate result the repayment question reads.
#[derive(Clone, Debug)]
pub struct GateSummary {
    pub verdict: String,
    pub benchmark_year_used: String,
    pub reasons: Vec<String>,
    pub caveats: Vec<String>,
}

impl From<&GateResult> for GateSummary {
    fn from(gate: &GateResult) -> GateSummary {
        GateSummary {
            verdict: gate.verdict.to_string(),
            benchmark_year_used: gate.benchmark_year_used.clone(),
            reasons: gate.reasons.clone(),
            caveats: gate.caveats.clone(),
        }
    }
}

/// Operator assertions for one amalgamated loan, one later year of income.
#[derive(Clone, Debug, Default)]
pub struct MyrFacts {
    pub year_of_income: Option<YearOfIncome>,
    pub amalgamated_loan_unpaid_at_end_of_previous_year: Option<Decimal>,
    pub remaining_term_years: Option<Decimal>,
    pub payments_applied_during_the_year: Option<Decimal>,
    pub gate_result: Option<GateSummary>,
    pub year_loan_made: Option<YearOfIncome>,
    pub loan_id: String,
}

impl MyrFacts {
    pub fn from_mapping(
        row: &Raw,
        year_of_income: YearOfIncome,
        gate_result: Option<GateSummary>,
        where_: &str,
    ) -> Result<MyrFacts, String> {
        let at = |key: &str| format!("{where_} {key}");
        Ok(MyrFacts {
            loan_id: text_cell(row.get("loan_id")),
            year_of_income: Some(year_of_income),
            amalgamated_loan_unpaid_at_end_of_previous_year: optional_money(
                row.get("amalgamated_loan_unpaid_at_end_of_previous_year"),
                &at("amalgamated_loan_unpaid_at_end_of_previous_year"),
            )?,
            remaining_term_years: optional_years(
                row.get("remaining_term_years"),
                &at("remaining_term_years"),
            )?,
            payments_applied_during_the_year: optional_money(
                row.get("payments_applied_during_the_year"),
                &at("payments_applied_during_the_year"),
            )?,
            gate_result,
            year_loan_made: optional_year_of_income(
                row.get("year_loan_made"),
                &at("year_loan_made"),
            )?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MyrResult {
    pub verdict: &'static str,
    pub loan_id: String,
    pub year_of_income: String,
    pub gate_verdict: Option<String>,
    pub benchmark: Option<RateResult>,
    pub amalgamated_loan_unpaid_at_end_of_previous_year: Option<Decimal>,
    pub remaining_term_years_used: Option<Decimal>,
    pub myr_required: Option<Decimal>,
    pub payments_applied: Option<Decimal>,
    pub shortfall: Option<Decimal>,
    pub experimental_deemed_dividend_exposure: Option<Decimal>,
    pub reasons: Vec<String>,
    pub reason_codes: Vec<String>,
    pub caveats: Vec<String>,
    pub statutory_trace: Vec<String>,
}

impl MyrResult {
    fn empty(verdict: &'static str, loan_id: &str) -> MyrResult {
        MyrResult {
            verdict,
            loan_id: loan_id.to_string(),
            year_of_income: String::new(),
            gate_verdict: None,
            benchmark: None,
            amalgamated_loan_unpaid_at_end_of_previous_year: None,
            remaining_term_years_used: None,
            myr_required: None,
            payments_applied: None,
            shortfall: None,
            experimental_deemed_dividend_exposure: None,
            reasons: vec![],
            reason_codes: vec![],
            caveats: vec![],
            statutory_trace: vec![],
        }
    }

    pub fn to_json(&self) -> Calc<Value> {
        let money = |v: &Option<Decimal>| -> Calc<Value> {
            Ok(match v {
                None => Value::Null,
                Some(v) => json!(cents_str(v)?),
            })
        };
        Ok(json!({
            "verdict": self.verdict,
            "loan_id": self.loan_id,
            "year_of_income": self.year_of_income,
            "gate_verdict": self.gate_verdict,
            "benchmark_rate": self.benchmark.as_ref().and_then(RateResult::rate_text),
            "benchmark_provenance": self.benchmark.as_ref().map(RateResult::provenance),
            "manifest": self.benchmark.as_ref().map_or(json!({"rate_table_uris": []}), RateResult::manifest),
            "amalgamated_loan_unpaid_at_end_of_previous_year": money(&self.amalgamated_loan_unpaid_at_end_of_previous_year)?,
            "remaining_term_years_used": self.remaining_term_years_used.as_ref().map(Decimal::to_sci_string),
            "myr_required": money(&self.myr_required)?,
            "payments_applied": money(&self.payments_applied)?,
            "shortfall": money(&self.shortfall)?,
            "experimental_deemed_dividend_exposure": money(&self.experimental_deemed_dividend_exposure)?,
            "rounding": ROUNDING,
            "reasons": self.reasons,
            "reason_codes": self.reason_codes,
            "caveats": self.caveats,
            "statutory_trace": self.statutory_trace,
        }))
    }
}

/// The s 109E(6) rounding rule: up to the next whole number.
pub fn statutory_remaining_term(supplied: &Decimal) -> Calc<Decimal> {
    supplied.to_integral_rounded(Rounding::Ceiling)
}

/// Why the bare formula cannot be evaluated.
#[derive(Debug, PartialEq)]
pub enum FormulaError {
    Value(&'static str),
    Signal(crate::decimal::Signal),
}

impl From<crate::decimal::Signal> for FormulaError {
    fn from(signal: crate::decimal::Signal) -> FormulaError {
        FormulaError::Signal(signal)
    }
}

/// The bare s 109E(6) formula at 60 digits, quantised once to cents.
pub fn minimum_yearly_repayment_amount(
    principal: &Decimal,
    rate: &Decimal,
    remaining_term: &Decimal,
) -> Result<Decimal, FormulaError> {
    let zero = d("0");
    if rate.le(&zero)? {
        return Err(FormulaError::Value(
            "the s 109E(6) formula is undefined at a nil benchmark rate",
        ));
    }
    if remaining_term.le(&zero)? {
        return Err(FormulaError::Value(
            "the s 109E(6) formula is undefined at a nil remaining term",
        ));
    }
    if !remaining_term.equals(&remaining_term.to_integral_value()?)? {
        return Err(FormulaError::Value(
            "remaining_term must be a whole number of years; use statutory_remaining_term first",
        ));
    }
    let raw = with_precision(FORMULA_PRECISION, || -> Calc<Decimal> {
        let one = d("1");
        let numerator = principal.mul(rate)?;
        let base = one.div(&one.add(rate)?)?;
        let denominator = one.sub(&base.pow_integral(remaining_term)?)?;
        numerator.div(&denominator)
    })?;
    Ok(to_cents(&raw)?)
}

pub(crate) const GENUINE_REPAYMENT_CAVEAT: &str = "s 109R: this engine did not decide whether the \
payments applied are genuine repayments. A payment made where the borrower intended to draw a \
similar or larger loan is left out of the reckoning by s 109R(2), and that turns on what a \
reasonable person would conclude. The figure used is the operator's assertion.";

const NOT_A_DETERMINATION_CAVEAT: &str = "Any shortfall shown is an experimental review aid, \
not an ATO assessment and not a s 109E(1) determination. s 109E(2) makes the dividend the \
shortfall subject to s 109Y, which caps Division 7A dividends at the company's distributable \
surplus, and s 109E(1)(d) removes the dividend where s 109Q applies. Neither is modelled \
here, and neither is s 109RB.";

const TERM_NOT_CHECKED_CAVEAT: &str = "maximum_term_years was not supplied, so this engine has \
not checked the remaining term against the loan's term or that term less the years since the \
loan year (s 109E(6)). A remaining term that is too long gives a lower minimum yearly repayment \
than the Act requires.";

/// Work out the s 109E minimum yearly repayment and any shortfall. REFUSED
/// where the question is outside this engine, UNKNOWN where a fact is missing.
/// It takes no loan term, so it checks no remaining-term limit, and a figure
/// it gives says so in a caveat.
pub fn minimum_yearly_repayment(
    facts: &MyrFacts,
    table: Option<&BenchmarkTable>,
    over: Option<&RateOverride>,
) -> Result<MyrResult, Failure> {
    repayment_within(facts, None, None, table, over)
}

/// `minimum_yearly_repayment`, also refusing a remaining term longer than
/// `maximum_term`, the s 109N(3) maximum term the same register row's security
/// facts establish, than `loan_term`, the row's own `maximum_term_years`, or
/// than that term less the years since the year the loan was made.
pub(crate) fn repayment_within(
    facts: &MyrFacts,
    maximum_term: Option<&Decimal>,
    loan_term: Option<&Decimal>,
    table: Option<&BenchmarkTable>,
    over: Option<&RateOverride>,
) -> Result<MyrResult, Failure> {
    let mut refusals: Vec<(&str, String)> = Vec::new();
    let mut unknowns: Vec<(&str, String)> = Vec::new();
    let mut caveats: Vec<String> = vec![
        GENUINE_REPAYMENT_CAVEAT.into(),
        NOT_A_DETERMINATION_CAVEAT.into(),
    ];
    let gate = facts.gate_result.as_ref();
    if let Some(gate) = gate {
        caveats.extend(gate.caveats.iter().cloned());
    }
    let Some(year) = facts.year_of_income else {
        let mut result = MyrResult::empty("UNKNOWN", &facts.loan_id);
        result.reasons = vec!["year_of_income was not supplied.".into()];
        result.reason_codes = vec!["YEAR_OF_INCOME_UNKNOWN".into()];
        result.caveats = caveats;
        return Ok(result);
    };
    let gate_verdict = gate.map(|g| g.verdict.clone());
    match gate {
        None => refusals.push((
            "REFUSED_GATE_RESULT_MISSING",
            "No s 109N gate result was supplied. s 109E works on an amalgamated loan, which \
             s 109E(3)(b) builds out of loans that would be dividends apart from s 109N. Run \
             the gate first."
                .into(),
        )),
        Some(gate) if gate.verdict == "NOT_COMPLYING" => refusals.push((
            "REFUSED_GATE_NOT_COMPLYING",
            format!(
                "The s 109N gate returned NOT_COMPLYING. A constituent loan of an amalgamated \
                 loan is one that would be a s 109D dividend apart from s 109N (s 109E(3)(b)), \
                 so a loan that fails s 109N is not one, and s 109E produces no minimum yearly \
                 repayment for it. Where no other Subdivision D provision applies, s 109D(1) \
                 instead treats it as a dividend in the year it was made, for the amount \
                 unpaid before the lodgment day (s 109D(1AA)). No MYR figure is emitted. \
                 Reasons: {}",
                gate.reasons.join("; ")
            ),
        )),
        Some(gate) if gate.verdict == "UNKNOWN" => refusals.push((
            "REFUSED_GATE_UNKNOWN",
            format!(
                "The s 109N gate returned UNKNOWN, so it is not established that this loan is \
                 an amalgamated loan on complying terms. Reasons: {}",
                gate.reasons.join("; ")
            ),
        )),
        Some(_) => {}
    }
    if let (Some(gate), Some(made)) = (gate, facts.year_loan_made)
        && gate.benchmark_year_used != made.label()
    {
        refusals.push((
            "REFUSED_GATE_BENCHMARK_YEAR_MISMATCH",
            "The gate benchmark year differs from year_loan_made. Run the original-year gate \
             first."
                .into(),
        ));
    }
    match facts.year_loan_made {
        Some(made) if made == year => refusals.push((
            "REFUSED_YEAR_IS_YEAR_OF_LOAN",
            format!(
                "The year of income requested ({}) is the year the loan was made. \
                 s 109E(1)(a) applies only where the amalgamated loan was made in an earlier \
                 year of income, and s 109P puts an amalgamated loan outside s 109D in the \
                 year it is made. There is no minimum yearly repayment for the year of the \
                 loan.",
                year.label()
            ),
        )),
        Some(made) if made > year => refusals.push((
            "REFUSED_YEAR_BEFORE_LOAN",
            format!(
                "The year of income requested ({}) is before the year the loan was made ({}). \
                 s 109E(1)(a) requires an earlier year of making.",
                year.label(),
                made.label()
            ),
        )),
        Some(_) => {}
        None => {
            unknowns.push((
                "YEAR_LOAN_MADE_UNKNOWN",
                "year_loan_made was not established.".into(),
            ));
            caveats.push(
                "year_loan_made was not supplied, so this engine has not established that the \
                 amalgamated loan was made in an earlier year of income as s 109E(1)(a) \
                 requires. The supplied balance unpaid at the end of the previous year implies \
                 it; nothing here verifies it."
                    .into(),
            );
        }
    }
    let rate = benchmark_rate(year, table, over);
    if !rate.known || rate.rate.is_none() {
        unknowns.push((
            "BENCHMARK_RATE_UNKNOWN",
            rate.reason
                .clone()
                .filter(|r| !r.is_empty())
                .unwrap_or_else(|| format!("No reviewed benchmark rate for {}.", year.label())),
        ));
    }
    let principal = facts
        .amalgamated_loan_unpaid_at_end_of_previous_year
        .clone();
    if principal.is_none() {
        unknowns.push((
            "UNPAID_BALANCE_UNKNOWN",
            "amalgamated_loan_unpaid_at_end_of_previous_year was not established. This engine \
             does not form the amalgamated loan from its constituent loans (s 109E(3)); the \
             operator supplies the balance."
                .into(),
        ));
    }
    let payments = facts.payments_applied_during_the_year.clone();
    if payments.is_none() {
        unknowns.push((
            "PAYMENTS_APPLIED_UNKNOWN",
            "payments_applied_during_the_year was not established. Bank credits are not a \
             substitute: s 109R decides which payments count, and this engine does not apply \
             it."
            .into(),
        ));
    }
    let zero = d("0");
    let mut term_used: Option<Decimal> = None;
    let complying = gate.is_some_and(|g| g.verdict == "COMPLYING");
    // s 109E(6)(b): the whole years from the end of the loan year to the end of
    // the year before this one, and the loan's term less them, rounded up. The
    // term is rounded up first: the same figure, as the years are whole, and
    // exact, where subtracting first would round a long term to the context
    // precision.
    let term_left = match (loan_term, facts.year_loan_made) {
        (Some(term), Some(made)) if complying && made < year => {
            let elapsed = year.start_year - made.start_year - 1;
            let left = statutory_remaining_term(term)?.sub(&d(&elapsed.to_string()))?;
            Some((term, made, elapsed, left))
        }
        _ => None,
    };
    match &facts.remaining_term_years {
        None => unknowns.push((
            "REMAINING_TERM_UNKNOWN",
            "remaining_term_years was not established.".into(),
        )),
        Some(supplied) => {
            let used = statutory_remaining_term(supplied)?;
            if used.le(&zero)? {
                refusals.push((
                    "REFUSED_REMAINING_TERM_NOT_POSITIVE",
                    format!(
                        "remaining_term_years is {supplied}, which leaves a remaining term of \
                         {used} under s 109E(6). The formula divides by a term that is nil at \
                         that point, and the loan should have been repaid in full by the end \
                         of the previous year of income. No MYR figure is emitted."
                    ),
                ));
            } else if !used.equals(supplied)? {
                caveats.push(format!(
                    "remaining_term_years {supplied} was rounded up to {used} under the closing \
                     words of s 109E(6)."
                ));
            }
            if let Some(maximum) = maximum_term
                && complying
                && used.gt(maximum)?
            {
                refusals.push((
                    "REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM",
                    format!(
                        "remaining_term_years is {supplied}, a remaining term of {used} under \
                         s 109E(6), longer than the {maximum}-year maximum term this row's \
                         security facts give under s 109N(3) before any s 109N(3A) to (3D) \
                         reduction, which can only shorten it, yet the gate found the loan on \
                         s 109N terms. Both cannot hold: each constituent loan is one s 109N \
                         saves (s 109E(3)(b)), so its term is within its maximum term \
                         (s 109N(1)(c)); the constituent loans share one maximum term \
                         (s 109E(3)(c)); and the remaining term is the longest constituent \
                         term less the years since the loan year (s 109E(6)). Correct the \
                         remaining term or the security facts. Enter a loan secured and \
                         extended under s 109E(3A) as the new amalgamated loan s 109E(3B) \
                         creates, with its 25-year security facts and, in maximum_term_years, \
                         25 years less the time the loan had already run before the \
                         extension, which is how the ATO works the term of such a loan. No \
                         MYR figure is emitted."
                    ),
                ));
            } else if let Some(term) = loan_term
                && complying
                && used.gt(&statutory_remaining_term(term)?)?
            {
                refusals.push((
                    "REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM",
                    format!(
                        "remaining_term_years is {supplied}, a remaining term of {used} under \
                         s 109E(6), longer than the {term}-year term of the loan in \
                         maximum_term_years. The remaining term is the longest constituent-loan \
                         term less the years from the end of the loan year to the end of the \
                         previous year of income (s 109E(6)), so it cannot be longer than that \
                         term, and a longer one gives a lower minimum yearly repayment than the \
                         Act requires. Correct remaining_term_years or, where a constituent loan \
                         of this amalgamated loan has a longer term, enter that longest term in \
                         maximum_term_years. No MYR figure is emitted."
                    ),
                ));
            } else if let Some((term, made, elapsed, left)) = &term_left
                && used.gt(&zero)?
                && used.gt(left)?
            {
                let previous = YearOfIncome {
                    start_year: year.start_year - 1,
                };
                let years = if *elapsed == 1 { "year" } else { "years" };
                let limit = if left.gt(&zero)? {
                    format!("so the remaining term is at most {left}")
                } else {
                    format!(
                        "which leaves a remaining term of {left}, so the loan should have \
                         been repaid in full by the end of the previous year of income"
                    )
                };
                refusals.push((
                    "REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED",
                    format!(
                        "remaining_term_years is {supplied}, a remaining term of {used} under \
                         s 109E(6), longer than the {term}-year term in maximum_term_years less \
                         the years since the loan year. s 109E(6) makes the remaining term the \
                         difference between (a) 'the number of years in the longest term of \
                         any of the constituent loans that the amalgamated loan takes account \
                         of' and (b) 'the number of years between the end of the private \
                         company's year of income in which the loan was made and the end of \
                         the private company's year of income before the year of income for \
                         which the minimum yearly repayment is being worked out', rounded up \
                         if not already a whole number. From the end of {} to the end of {} \
                         is {elapsed} {years}, {limit}; a longer remaining term gives a lower \
                         minimum yearly repayment than the Act requires. Correct \
                         remaining_term_years, year_loan_made or maximum_term_years. A loan \
                         secured and extended under s 109E(3A) is the new amalgamated loan \
                         s 109E(3B) treats as made just before the start of the later year of \
                         income, so its year_loan_made is the year before that later year, and \
                         its maximum_term_years, as the ATO works it, is 25 years less the \
                         time the loan had already run before the extension. No MYR figure is \
                         emitted.",
                        made.label(),
                        previous.label()
                    ),
                ));
            }
            term_used = Some(used);
        }
    }
    if let Some(r) = &rate.rate
        && r.le(&zero)?
    {
        refusals.push((
            "REFUSED_BENCHMARK_RATE_NOT_POSITIVE",
            format!(
                "The benchmark rate for {} is {}. The s 109E(6) formula divides by 1 minus a \
                 power of 1/(1+rate), which is nil at a nil rate. No MYR figure is emitted.",
                year.label(),
                rate.rate_text().unwrap_or_default()
            ),
        ));
    }
    let mut trace = vec![
        "ITAA 1936 s 109E(1): a private company is taken to pay a dividend at the end of the \
         current year where an amalgamated loan made in an earlier year is not repaid and \
         payments fall short of the minimum yearly repayment."
            .to_string(),
        "s 109E(5): the minimum yearly repayment is worked out using the formula in s 109E(6)."
            .to_string(),
    ];
    let mut result = MyrResult::empty("REFUSED", &facts.loan_id);
    result.year_of_income = year.label();
    result.gate_verdict = gate_verdict;
    result.amalgamated_loan_unpaid_at_end_of_previous_year = principal.clone();
    result.remaining_term_years_used = term_used.clone();
    result.payments_applied = payments.clone();
    result.caveats = caveats;
    if !refusals.is_empty() || !unknowns.is_empty() {
        let (verdict, items) = if refusals.is_empty() {
            ("UNKNOWN", unknowns)
        } else {
            ("REFUSED", refusals)
        };
        result.verdict = verdict;
        result.benchmark = Some(rate);
        result.reasons = items.iter().map(|(_, text)| text.clone()).collect();
        result.reason_codes = items.iter().map(|(code, _)| code.to_string()).collect();
        result.statutory_trace = trace;
        return Ok(result);
    }
    let (principal, payments, term, benchmark) = (
        principal.expect("established"),
        payments.expect("established"),
        term_used.expect("established"),
        rate.rate.clone().expect("known"),
    );
    let required = match minimum_yearly_repayment_amount(&principal, &benchmark, &term) {
        Ok(required) => required,
        Err(FormulaError::Signal(signal)) => return Err(Failure::Signal(signal)),
        Err(FormulaError::Value(message)) => return Err(Failure::Value(message.into())),
    };
    let applied = to_cents(&payments)?;
    let mut shortfall = required.sub(&applied)?;
    if shortfall.lt(&zero)? {
        shortfall = d("0.00");
    }
    let shortfall = to_cents(&shortfall)?;
    let met = shortfall.equals(&d("0.00"))?;
    let rate_text = rate.rate_text().unwrap_or_default();
    let (p, r, a, s) = (
        cents_str(&principal)?,
        cents_str(&required)?,
        cents_str(&applied)?,
        cents_str(&shortfall)?,
    );
    trace.extend([
        "s 109E(6) formula: MYR = (amount of the loan not repaid by the end of the previous \
         year of income x current year's benchmark interest rate) / (1 - (1 / (1 + current \
         year's benchmark interest rate)) ^ remaining term)."
            .to_string(),
        format!("Amount of the loan not repaid by the end of the previous year of income = {p}."),
        format!(
            "Current year's benchmark interest rate ({}, s 109N(2)) = {rate_text}, from RBA \
             table {} series {}, {} figure.",
            year.label(),
            rate.entry.as_ref().map_or("", |e| e.rba_table.as_str()),
            rate.entry.as_ref().map_or("", |e| e.rba_series.as_str()),
            rate.entry.as_ref().map_or("", |e| e.rba_month.as_str()),
        ),
        format!("Remaining term (s 109E(6), rounded up to a whole number) = {term}."),
        format!(
            "MYR = ({p} x {rate_text}) / (1 - (1 / (1 + {rate_text})) ^ {term}) = {r} \
             (quantised to cents, {ROUNDING}; the Act prescribes no rounding)."
        ),
        format!("Payments applied during the year (operator assertion, s 109R not applied) = {a}."),
        format!("Shortfall (s 109E(1)(c)) = max(0, {r} - {a}) = {s}."),
    ]);
    if loan_term.is_none() {
        result.caveats.push(TERM_NOT_CHECKED_CAVEAT.into());
    }
    result.verdict = if met { "MYR_MET" } else { "MYR_SHORT" };
    result.benchmark = Some(rate);
    result.myr_required = Some(required);
    result.payments_applied = Some(applied);
    result.experimental_deemed_dividend_exposure = (!met).then(|| shortfall.clone());
    result.shortfall = Some(shortfall);
    result.statutory_trace = trace;
    Ok(result)
}
