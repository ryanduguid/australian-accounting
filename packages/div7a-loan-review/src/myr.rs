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

const GENUINE_REPAYMENT_CAVEAT: &str = "s 109R: this engine did not decide whether the \
payments applied are genuine repayments. A payment made where the borrower intended to draw a \
similar or larger loan is left out of the reckoning by s 109R(2), and that turns on what a \
reasonable person would conclude. The figure used is the operator's assertion.";

const NOT_A_DETERMINATION_CAVEAT: &str = "Any shortfall shown is an experimental review aid, \
not an ATO assessment and not a s 109E(1) determination. s 109E(2) makes the dividend the \
shortfall subject to s 109Y, which caps Division 7A dividends at the company's distributable \
surplus, and s 109E(1)(d) removes the dividend where s 109Q applies. Neither is modelled \
here, and neither is s 109RB.";

/// Work out the s 109E minimum yearly repayment and any shortfall. REFUSED
/// where the question is outside this engine, UNKNOWN where a fact is missing.
pub fn minimum_yearly_repayment(
    facts: &MyrFacts,
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
    result.verdict = if met { "MYR_MET" } else { "MYR_SHORT" };
    result.benchmark = Some(rate);
    result.myr_required = Some(required);
    result.payments_applied = Some(applied);
    result.experimental_deemed_dividend_exposure = (!met).then(|| shortfall.clone());
    result.shortfall = Some(shortfall);
    result.statutory_trace = trace;
    Ok(result)
}
