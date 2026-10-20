//! The complying-loan gate: ITAA 1936 s 109N(1). Every limb must be
//! established; a failed limb gives NOT_COMPLYING and an unestablished one
//! UNKNOWN. The lodgment day is the operator's assertion, not computed here.

use serde_json::{Value, json};

use crate::Failure;
use crate::decimal::{Calc, Decimal};
use crate::facts::{
    optional_rate, optional_year_of_income, optional_years, parse_tristate, text_cell,
};
use crate::money::d;
use crate::rates::{BenchmarkTable, RateOverride, RateResult, benchmark_rate};
use crate::raw::Raw;
use crate::years::YearOfIncome;

/// s 109N(3)(a): 25 years for a loan secured by a registered mortgage over real property.
pub const SECURED_MAXIMUM_TERM: &str = "25";
/// s 109N(3)(b): 7 years for any other loan.
pub const DEFAULT_MAXIMUM_TERM: &str = "7";
/// s 109N(3)(a)(ii): security of at least 110% of the loan.
pub const REQUIRED_SECURITY_COVERAGE: &str = "1.10";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimbState {
    Pass,
    Fail,
    Unknown,
}

impl LimbState {
    pub fn name(self) -> &'static str {
        match self {
            LimbState::Pass => "PASS",
            LimbState::Fail => "FAIL",
            LimbState::Unknown => "UNKNOWN",
        }
    }
}

/// One requirement of s 109N(1), and what the supplied facts did to it.
#[derive(Clone, Debug)]
pub struct Limb {
    pub cite: &'static str,
    pub requirement: &'static str,
    pub state: LimbState,
    pub finding: String,
}

impl Limb {
    fn new(
        cite: &'static str,
        requirement: &'static str,
        state: LimbState,
        finding: impl Into<String>,
    ) -> Limb {
        Limb {
            cite,
            requirement,
            state,
            finding: finding.into(),
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "cite": self.cite,
            "requirement": self.requirement,
            "state": self.state.name(),
            "finding": self.finding,
        })
    }
}

/// Operator assertions about one loan. Nothing here is inferred.
#[derive(Clone, Debug, Default)]
pub struct GateFacts {
    pub maximum_term_years: Option<Decimal>,
    pub written_agreement: Option<bool>,
    pub terms_in_place_before_lodgment_day: Option<bool>,
    pub secured_by_registered_mortgage_over_real_property: Option<bool>,
    pub security_coverage_at_first_made: Option<Decimal>,
    pub interest_rate_for_years_after_year_loan_made: Option<Decimal>,
    pub year_of_income_being_tested: Option<YearOfIncome>,
    pub year_loan_made: Option<YearOfIncome>,
    pub loan_id: String,
}

impl GateFacts {
    /// `GateFacts.from_mapping(row, where)`: a CSV row or MCP mapping.
    pub fn from_mapping(row: &Raw, where_: &str) -> Result<GateFacts, String> {
        let at = |key: &str| format!("{where_} {key}");
        Ok(GateFacts {
            loan_id: text_cell(row.get("loan_id")),
            written_agreement: parse_tristate(
                row.get("written_agreement"),
                &at("written_agreement"),
            )?,
            terms_in_place_before_lodgment_day: parse_tristate(
                row.get("terms_in_place_before_lodgment_day"),
                &at("terms_in_place_before_lodgment_day"),
            )?,
            maximum_term_years: optional_years(
                row.get("maximum_term_years"),
                &at("maximum_term_years"),
            )?,
            secured_by_registered_mortgage_over_real_property: parse_tristate(
                row.get("secured_by_registered_mortgage_over_real_property"),
                &at("secured_by_registered_mortgage_over_real_property"),
            )?,
            security_coverage_at_first_made: optional_years(
                row.get("security_coverage_at_first_made"),
                &at("security_coverage_at_first_made"),
            )?,
            interest_rate_for_years_after_year_loan_made: optional_rate(
                row.get("interest_rate_for_years_after_year_loan_made"),
                &at("interest_rate_for_years_after_year_loan_made"),
            )?,
            year_of_income_being_tested: optional_year_of_income(
                row.get("year_of_income_being_tested"),
                &at("year_of_income_being_tested"),
            )?,
            year_loan_made: optional_year_of_income(
                row.get("year_loan_made"),
                &at("year_loan_made"),
            )?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GateResult {
    pub verdict: &'static str,
    pub loan_id: String,
    pub benchmark_year_used: String,
    pub benchmark: Option<RateResult>,
    pub maximum_term_years_allowed: Option<Decimal>,
    pub limbs: Vec<Limb>,
    pub reasons: Vec<String>,
    pub reason_codes: Vec<String>,
    pub caveats: Vec<String>,
    pub statutory_trace: Vec<String>,
}

impl GateResult {
    pub fn to_json(&self) -> Value {
        json!({
            "verdict": self.verdict,
            "loan_id": self.loan_id,
            "benchmark_year_used": self.benchmark_year_used,
            "benchmark_rate": self.benchmark.as_ref().and_then(RateResult::rate_text),
            "benchmark_provenance": self.benchmark.as_ref().map(RateResult::provenance),
            "manifest": self.benchmark.as_ref().map_or(json!({"rate_table_uris": []}), RateResult::manifest),
            "maximum_term_years_allowed": self.maximum_term_years_allowed.as_ref().map(Decimal::to_sci_string),
            "limbs": self.limbs.iter().map(Limb::to_json).collect::<Vec<_>>(),
            "reasons": self.reasons,
            "reason_codes": self.reason_codes,
            "caveats": self.caveats,
            "statutory_trace": self.statutory_trace,
        })
    }
}

fn written_agreement_limb(facts: &GateFacts) -> Limb {
    let (cite, requirement) = (
        "s 109N(1)(a)",
        "the agreement that the loan was made under is in writing",
    );
    match facts.written_agreement {
        None => Limb::new(
            cite,
            requirement,
            LimbState::Unknown,
            "written_agreement was not established. This engine will not read the absence \
             of an assertion as an absence of an agreement.",
        ),
        Some(true) => Limb::new(
            cite,
            requirement,
            LimbState::Pass,
            "Operator asserts a written agreement.",
        ),
        Some(false) => Limb::new(
            cite,
            requirement,
            LimbState::Fail,
            "Operator asserts there is no written agreement.",
        ),
    }
}

fn lodgment_day_limb(facts: &GateFacts) -> Limb {
    let (cite, requirement) = (
        "s 109N(1) chapeau, with s 109D(6)",
        "the criteria were met before the lodgment day for the year of income",
    );
    match facts.terms_in_place_before_lodgment_day {
        None => Limb::new(
            cite,
            requirement,
            LimbState::Unknown,
            "terms_in_place_before_lodgment_day was not established. The lodgment day under \
             s 109D(6) is the earlier of the due date for the company's return and the day \
             it was lodged; this engine does not compute either.",
        ),
        Some(true) => Limb::new(
            cite,
            requirement,
            LimbState::Pass,
            "Operator asserts the terms were in place before the lodgment day.",
        ),
        Some(false) => Limb::new(
            cite,
            requirement,
            LimbState::Fail,
            "Operator asserts the terms were not in place before the lodgment day.",
        ),
    }
}

fn interest_limb(facts: &GateFacts, rate: &RateResult) -> Calc<Limb> {
    let (cite, requirement) = (
        "s 109N(1)(b)",
        "the rate of interest payable for years of income after the year the loan was made \
         equals or exceeds the benchmark interest rate",
    );
    let Some(supplied) = &facts.interest_rate_for_years_after_year_loan_made else {
        return Ok(Limb::new(
            cite,
            requirement,
            LimbState::Unknown,
            "interest_rate_for_years_after_year_loan_made was not established.",
        ));
    };
    let (Some(benchmark), true) = (&rate.rate, rate.known) else {
        return Ok(Limb::new(
            cite,
            requirement,
            LimbState::Unknown,
            format!(
                "No reviewed benchmark rate for {}, so the supplied rate {supplied} cannot be \
                 tested against a floor.",
                rate.year_of_income
            ),
        ));
    };
    let text = rate.rate_text().unwrap_or_default();
    Ok(if !supplied.lt(benchmark)? {
        Limb::new(
            cite,
            requirement,
            LimbState::Pass,
            format!(
                "Supplied rate {supplied} is at or above the {} benchmark {text}.",
                rate.year_of_income
            ),
        )
    } else {
        Limb::new(
            cite,
            requirement,
            LimbState::Fail,
            format!(
                "Supplied rate {supplied} is below the {} benchmark {text}.",
                rate.year_of_income
            ),
        )
    })
}

/// The s 109N(3) maximum term, its basis and caveats; None where not established.
fn maximum_term(facts: &GateFacts) -> Calc<(Option<Decimal>, String, Vec<String>)> {
    let coverage = &facts.security_coverage_at_first_made;
    match facts.secured_by_registered_mortgage_over_real_property {
        Some(false) => Ok((
            Some(d(DEFAULT_MAXIMUM_TERM)),
            "s 109N(3)(b): 7 years for a loan that is not secured".into(),
            vec![],
        )),
        Some(true) => {
            let Some(coverage) = coverage else {
                return Ok((
                    None,
                    "s 109N(3)(a)(ii): security coverage at the time the loan was first made \
                     was not established"
                        .into(),
                    vec![],
                ));
            };
            if !coverage.lt(&d(REQUIRED_SECURITY_COVERAGE))? {
                return Ok((
                    Some(d(SECURED_MAXIMUM_TERM)),
                    format!(
                        "s 109N(3)(a): 25 years; registered mortgage over real property and \
                         coverage {coverage} is at least {REQUIRED_SECURITY_COVERAGE}"
                    ),
                    vec![
                        "s 109N(3A) to (3D) reduce the maximum term where the loan refinances \
                         an earlier loan. Refinancing is not modelled; if this loan refinanced \
                         another, the 25 years may be shorter."
                            .into(),
                    ],
                ));
            }
            Ok((
                Some(d(DEFAULT_MAXIMUM_TERM)),
                format!(
                    "s 109N(3)(b): 7 years; coverage {coverage} is below the \
                     {REQUIRED_SECURITY_COVERAGE} that s 109N(3)(a)(ii) requires, so the \
                     25-year limb is not available"
                ),
                vec![],
            ))
        }
        None => Ok((
            None,
            "s 109N(3): whether the loan is secured by a registered mortgage over real \
             property was not established"
                .into(),
            vec![],
        )),
    }
}

fn term_limb(facts: &GateFacts) -> Calc<(Limb, Option<Decimal>, Vec<String>)> {
    let (cite, requirement) = (
        "s 109N(1)(c)",
        "the term of the loan does not exceed the maximum term under s 109N(3)",
    );
    let (allowed, basis, caveats) = maximum_term(facts)?;
    let Some(term) = &facts.maximum_term_years else {
        return Ok((
            Limb::new(
                cite,
                requirement,
                LimbState::Unknown,
                "maximum_term_years was not established.",
            ),
            allowed,
            caveats,
        ));
    };
    let Some(allowed) = allowed else {
        // The security facts only matter above 7 years.
        if term.le(&d(DEFAULT_MAXIMUM_TERM))? {
            return Ok((
                Limb::new(
                    cite,
                    requirement,
                    LimbState::Pass,
                    format!(
                        "Term {term} years is within the 7 years s 109N(3)(b) allows for any \
                         loan, so the unestablished security facts cannot change this limb."
                    ),
                ),
                Some(d(DEFAULT_MAXIMUM_TERM)),
                caveats,
            ));
        }
        return Ok((
            Limb::new(
                cite,
                requirement,
                LimbState::Unknown,
                format!(
                    "{basis}, and the term {term} years exceeds the {DEFAULT_MAXIMUM_TERM} \
                     years available without it."
                ),
            ),
            None,
            caveats,
        ));
    };
    if term.le(&allowed)? {
        return Ok((
            Limb::new(
                cite,
                requirement,
                LimbState::Pass,
                format!("Term {term} years is within {allowed} ({basis})."),
            ),
            Some(allowed),
            caveats,
        ));
    }
    Ok((
        Limb::new(
            cite,
            requirement,
            LimbState::Fail,
            format!("Term {term} years exceeds {allowed} ({basis})."),
        ),
        Some(allowed),
        caveats,
    ))
}

const INTEREST_FLOOR_CAVEAT: &str = "Interest-floor interpretation unresolved: ATO guidance \
requires the benchmark interest rate for each year of the loan. This model compares the \
supplied rate with one selected year's benchmark and defaults to year_loan_made. COMPLYING \
does not establish that the agreement meets the annual interest requirement. Review the \
agreement and relevant years before relying on this result. See \
docs/primary-source-review-2026-08-31.md.";

const NOT_A_FINDING_CAVEAT: &str = "COMPLYING here means the four model comparisons passed \
on the facts supplied, subject to the interest-floor limitation. It is not a finding that no \
dividend arises: Subdivision D holds other exclusions, s 109T to s 109X reach interposed \
entities, and s 109XA reaches unpaid present entitlements. None of those is modelled.";

/// Test one loan against s 109N(1) on the operator's own facts.
pub fn complying_loan_gate(
    facts: &GateFacts,
    table: Option<&BenchmarkTable>,
    over: Option<&RateOverride>,
) -> Result<GateResult, Failure> {
    let floor_year = facts.year_of_income_being_tested.or(facts.year_loan_made);
    let Some(floor_year) = floor_year else {
        return Ok(GateResult {
            verdict: "UNKNOWN",
            loan_id: facts.loan_id.clone(),
            benchmark_year_used: String::new(),
            benchmark: None,
            maximum_term_years_allowed: None,
            limbs: vec![],
            reasons: vec![
                "Neither year_of_income_being_tested nor year_loan_made was supplied, so there \
                 is no year to read a benchmark interest rate for and s 109N(1)(b) cannot be \
                 tested."
                    .into(),
            ],
            reason_codes: vec!["GATE_NO_BENCHMARK_YEAR".into()],
            caveats: vec![],
            statutory_trace: vec![
                "ITAA 1936 s 109N(1): every limb must be met before the lodgment day.".into(),
            ],
        });
    };
    let rate = benchmark_rate(floor_year, table, over);
    let mut caveats = vec![INTEREST_FLOOR_CAVEAT.to_string()];
    if let (Some(made), Some(tested)) = (facts.year_loan_made, facts.year_of_income_being_tested)
        && tested != made
    {
        caveats.push(format!(
            "This s 109N(1)(b) model uses the operator's nominated benchmark year {}, rather \
             than its default year_loan_made ({}). The annual-interest interpretation remains \
             unresolved. See evaluation/div7a_myr/README.md.",
            floor_year.label(),
            made.label()
        ));
    }
    let (term, allowed, term_caveats) = term_limb(facts)?;
    let named = [
        ("WRITTEN_AGREEMENT", written_agreement_limb(facts)),
        ("LODGMENT_DAY", lodgment_day_limb(facts)),
        ("INTEREST", interest_limb(facts, &rate)?),
        ("TERM", term),
    ];
    caveats.extend(term_caveats);
    let states: Vec<LimbState> = named.iter().map(|(_, l)| l.state).collect();
    let verdict = if states.contains(&LimbState::Fail) {
        "NOT_COMPLYING"
    } else if states.contains(&LimbState::Unknown) {
        "UNKNOWN"
    } else {
        "COMPLYING"
    };
    let mut reasons = Vec::new();
    let mut reason_codes = Vec::new();
    for (name, limb) in &named {
        if limb.state != LimbState::Pass {
            reasons.push(format!("{}: {}", limb.cite, limb.finding));
            reason_codes.push(format!("GATE_{name}_{}", limb.state.name()));
        }
    }
    caveats.push(NOT_A_FINDING_CAVEAT.into());
    let trace = vec![
        "ITAA 1936 s 109N(1): a loan is not taken under s 109D to be a dividend if, before \
         the lodgment day for the year of income, the agreement is in writing, the interest \
         rate meets the benchmark, and the term is within the s 109N(3) maximum."
            .to_string(),
        format!(
            "Benchmark interest rate read for {} (s 109N(2)): {}.",
            rate.year_of_income,
            rate.rate_text().unwrap_or_else(|| "UNKNOWN".into())
        ),
        "s 109D(6): lodgment day is the earlier of the due date for the company's return and \
         the day it was lodged; asserted by the operator, not computed here."
            .to_string(),
    ];
    Ok(GateResult {
        verdict,
        loan_id: facts.loan_id.clone(),
        benchmark_year_used: rate.year_of_income.clone(),
        benchmark: Some(rate),
        maximum_term_years_allowed: allowed,
        limbs: named.into_iter().map(|(_, l)| l).collect(),
        reasons,
        reason_codes,
        caveats,
        statutory_trace: trace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapping(key: &str, value: &str) -> Raw {
        Raw::Dict(vec![(key.into(), Raw::text(value))])
    }

    #[test]
    fn mapped_facts_keep_unknown_apart_from_false() {
        let blank = GateFacts::from_mapping(&mapping("written_agreement", ""), "row").unwrap();
        assert_eq!(blank.written_agreement, None);
        let typo = GateFacts::from_mapping(&mapping("written_agreement", "ture"), "row");
        assert!(typo.unwrap_err().contains("true, false, or unknown"));
        let rate = "interest_rate_for_years_after_year_loan_made";
        assert!(GateFacts::from_mapping(&mapping(rate, "-0.05"), "row").is_err());
        let nan = GateFacts::from_mapping(&mapping(rate, "nan"), "row").unwrap_err();
        assert!(nan.contains("nan"), "{nan}");
    }
}
