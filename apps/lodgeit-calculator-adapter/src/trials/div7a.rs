//! `trials/div7a.py`: Division 7A, the local engine against the provider's
//! overlapping scope.
//!
//! The 6 conventions `div7a.py` lists (period, rate unit, balance, remaining
//! term, repayments, rounding) are lined up the same way here, and the
//! provider's `is_complying` and `deemed_dividend` are never mapped into the
//! local engine's verdict. The local engine is the in-tree Rust
//! `div7a-loan-review`, in place of the Python release the trial pinned.
use div7a_loan_review::Failure;
use div7a_loan_review::decimal::Signal as EngineSignal;
use div7a_loan_review::myr::{GateSummary, MyrFacts, minimum_yearly_repayment};
use div7a_loan_review::raw::Raw;
use div7a_loan_review::years::parse_year;

use super::{Comparison, Evaluation, Money, compare_values, put};
use crate::client::{Client, Outcome, Status};
use crate::decimal::{Decimal, Signal};
use crate::p;
use crate::pyjson::{self, Json};
use crate::pyops::{Exc, R, S, conversion, decimal_signal, getitem, pts};
use crate::pyvalue::str_points;

pub const CALC_URI: &str = "urn:sbrm:calculator:div7a:at";
const CASES: &str = include_str!("../../fixtures/div7a-cases.json");

/// The provider's period URN for an income year ending 30 June NNNN, against
/// the local engine's label for the same year.
pub const PERIOD_MAP: [(&str, &str); 2] = [
    ("urn:sbrm:period:div7a:fy2025", "2024-25"),
    ("urn:sbrm:period:div7a:fy2026", "2025-26"),
];

#[derive(Clone, Debug)]
pub struct Repayment {
    pub amount: Decimal,
    pub date: String,
}

#[derive(Clone, Debug)]
pub struct Case {
    pub case_id: String,
    pub period_uri: String,
    pub amalgamated_base: Decimal,
    pub loan_term_years: i64,
    pub loan_origination_date: String,
    pub income_year_start_date: String,
    pub is_first_real_myr_year: bool,
    pub repayments: Vec<Repayment>,
    pub remaining_term_years: i64,
    pub expected: Money,
    /// The verdict `run_local` reaches for this case.
    pub expect_local: String,
    /// The evaluation the trial reported against the provider on the probe
    /// date recorded in `docs/live-probe.md`.
    pub expect_evaluation: String,
    pub note: String,
}

impl Case {
    /// The local income-year label, or `None` when the period is unmapped:
    /// one side covers a year the other does not, which the trial reports
    /// rather than guessing a mapping for.
    pub fn local_year(&self) -> Option<&'static str> {
        PERIOD_MAP
            .iter()
            .find(|(uri, _)| *uri == self.period_uri)
            .map(|(_, year)| *year)
    }

    pub fn repayments_total(&self) -> R<Decimal> {
        self.repayments
            .iter()
            .try_fold(Decimal::from_int(0), |total, item| total.add(&item.amount))
            .map_err(decimal_signal)
    }
}

/// `Decimal(value)` for a str, an int or a float.
fn decimal(value: &Json) -> R<Decimal> {
    match value {
        Json::Int(i) => Ok(Decimal::from_bigint(i)),
        Json::Bool(b) => Ok(Decimal::from_int(i64::from(*b))),
        Json::Float(f) => Ok(Decimal::from_float(*f)),
        Json::Str(s) => Decimal::parse_points(&s.code_points()).map_err(conversion),
        other => Err(Exc::msg(
            "TypeError",
            &format!(
                "conversion from {} to Decimal is not supported",
                crate::pyvalue::type_name(other)
            ),
        )),
    }
}

fn text(value: &Json) -> R<String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Exc::msg("TypeError", "a fixture field is not a str"))
}

fn int(value: &Json) -> R<i64> {
    match value {
        Json::Int(i) => i
            .try_into()
            .map_err(|_| Exc::msg("OverflowError", "a fixture int is out of range")),
        _ => Err(Exc::msg("TypeError", "a fixture field is not an int")),
    }
}

/// `load_cases(path)`: the cases in `text`, the packaged case file when
/// `None`.
pub fn load_cases(text_of_file: Option<&str>) -> R<Vec<Case>> {
    let payload = pyjson::loads_standard(text_of_file.unwrap_or(CASES))
        .map_err(|_| Exc::msg("json.decoder.JSONDecodeError", "the case file is not JSON"))?;
    let Json::Array(raws) = getitem(&payload, "cases")? else {
        return Err(Exc::msg("TypeError", "cases is not a list"));
    };
    raws.iter().map(load_case).collect()
}

/// Each field in the order Python's constructor call reads it, so the first
/// failure is the same one. Repayment amounts are read here rather than at
/// first use: the typed case holds Decimals.
fn load_case(raw: &Json) -> R<Case> {
    let field = |name: &str| getitem(raw, name);
    Ok(Case {
        case_id: text(field("case_id")?)?,
        period_uri: text(field("period_uri")?)?,
        amalgamated_base: decimal(field("amalgamated_base")?)?,
        loan_term_years: int(field("loan_term_years")?)?,
        loan_origination_date: text(field("loan_origination_date")?)?,
        income_year_start_date: text(field("income_year_start_date")?)?,
        is_first_real_myr_year: match field("is_first_real_myr_year")? {
            Json::Bool(value) => *value,
            _ => {
                return Err(Exc::msg(
                    "TypeError",
                    "is_first_real_myr_year is not a bool",
                ));
            }
        },
        repayments: match field("repayments")? {
            Json::Array(items) => items
                .iter()
                .map(|item| {
                    Ok(Repayment {
                        amount: decimal(getitem(item, "amount")?)?,
                        date: text(getitem(item, "date")?)?,
                    })
                })
                .collect::<R<Vec<_>>>()?,
            _ => return Err(Exc::msg("TypeError", "repayments is not a list")),
        },
        remaining_term_years: int(field("remaining_term_years")?)?,
        expected: match field("expected")? {
            Json::Object(pairs) => {
                let mut expected = Money::new();
                for (key, value) in pairs {
                    put(&mut expected, &key.code_points(), decimal(value)?);
                }
                expected
            }
            _ => return Err(Exc::msg("AttributeError", "expected has no items")),
        },
        expect_local: text(field("expect_local")?)?,
        expect_evaluation: text(field("expect_evaluation")?)?,
        note: raw.get("note").map_or(Ok(String::new()), text)?,
    })
}

/// `build_request(case)`: the provider request body. Money stays Decimal to
/// the serialiser.
pub fn build_request(case: &Case) -> Json {
    let repayments = case
        .repayments
        .iter()
        .map(|item| {
            Json::obj([
                ("amount", Json::Dec(item.amount.clone())),
                ("date", Json::str(&item.date)),
            ])
        })
        .collect();
    Json::obj([
        ("amalgamated_base", Json::Dec(case.amalgamated_base.clone())),
        ("loan_term_years", Json::Int(case.loan_term_years.into())),
        (
            "loan_origination_date",
            Json::str(&case.loan_origination_date),
        ),
        (
            "income_year_start_date",
            Json::str(&case.income_year_start_date),
        ),
        (
            "is_first_real_myr_year",
            Json::Bool(case.is_first_real_myr_year),
        ),
        ("repayments", Json::Array(repayments)),
    ])
}

/// The engine's Decimal as the adapter's, digit for digit.
fn local(value: &div7a_loan_review::decimal::Decimal) -> Decimal {
    Decimal::parse_exact(&value.to_sci_string()).expect("an engine Decimal reads back")
}

fn engine_error(failure: Failure) -> Exc {
    match failure {
        Failure::Value(message) => Exc::msg("ValueError", &message),
        Failure::Signal(signal) => decimal_signal(match signal {
            EngineSignal::InvalidOperation => Signal::InvalidOperation,
            EngineSignal::DivisionByZero => Signal::DivisionByZero,
            EngineSignal::Overflow => Signal::Overflow,
        }),
        // The engine keeps only the last traceback line, class included, so
        // the class here is generic and the text carries the original.
        Failure::Uncaught(message) => Exc::msg("Exception", &message),
    }
}

/// `_origination_year_label(case)`: the income year the loan was made in,
/// as the local engine labels it.
fn origination_year_label(case: &Case) -> R<String> {
    let part = |from: usize, to: usize| -> R<num_bigint::BigInt> {
        let piece: String = case
            .loan_origination_date
            .chars()
            .skip(from)
            .take(to - from)
            .collect();
        crate::py::parse_int(&piece, 10).map_err(|message| Exc::msg("ValueError", &message))
    };
    let year = part(0, 4)?;
    let start = if part(5, 7)? >= 7u8.into() {
        year
    } else {
        year - 1u8
    };
    let next: String = (&start + 1u8).to_string().chars().skip(2).collect();
    Ok(format!("{start}-{next}"))
}

/// `run_local(case)`: the local engine's verdict, figures and reasons.
pub fn run_local(case: &Case) -> R<(String, Money, Vec<S>)> {
    let Some(label) = case.local_year() else {
        return Ok((
            "UNMAPPED_PERIOD".into(),
            Money::new(),
            vec![p!(
                case.period_uri,
                " has no entry in PERIOD_MAP, so no local income year is established for it. \
                 A mapping is a reviewed fact, not a guess from the label."
            )],
        ));
    };
    let year_of = |text: &str| {
        parse_year(&Raw::Str(text.to_owned()), "case")
            .map_err(|m| Exc::msg("div7aloan.years.YearError", &m))
    };
    let year = year_of(label)?;
    let made = year_of(&origination_year_label(case)?)?;
    // The s 109N gate is supplied as COMPLYING on the case's own asserted
    // facts: the scope alignment, never a conclusion the provider handed over.
    let gate = GateSummary {
        verdict: "COMPLYING".into(),
        benchmark_year_used: made.label(),
        reasons: Vec::new(),
        caveats: Vec::new(),
    };
    let engine = |value: &Decimal| {
        div7a_loan_review::decimal::Decimal::parse(&value.to_sci_string())
            .expect("an adapter Decimal reads back")
    };
    let facts = MyrFacts {
        loan_id: case.case_id.clone(),
        year_of_income: Some(year),
        amalgamated_loan_unpaid_at_end_of_previous_year: Some(engine(&case.amalgamated_base)),
        remaining_term_years: Some(engine(&Decimal::from_int(case.remaining_term_years))),
        payments_applied_during_the_year: Some(engine(&case.repayments_total()?)),
        gate_result: Some(gate),
        year_loan_made: Some(made),
    };
    let result = minimum_yearly_repayment(&facts, None, None).map_err(engine_error)?;
    let mut values = Money::new();
    let rate = result.benchmark.as_ref().and_then(|b| b.rate.as_ref());
    for (name, value) in [
        ("statutory_myr", result.myr_required.as_ref()),
        ("total_repayments", result.payments_applied.as_ref()),
        ("shortfall", result.shortfall.as_ref()),
        ("benchmark_rate", rate),
    ] {
        if let Some(value) = value {
            put(&mut values, &pts(name), local(value));
        }
    }
    let mut reasons = vec![pts(
        "The s 109N gate was supplied to this trial as an asserted fact, so the repayment \
         arithmetic could be compared. It was not derived from the provider and it is not a \
         conclusion about the loan.",
    )];
    reasons.extend(result.reasons.iter().map(|r| pts(r)));
    Ok((result.verdict.to_owned(), values, reasons))
}

/// `extract_upstream(result)`: the provider's figures, as Decimals, for the
/// fields both sides have.
pub fn extract_upstream(result: &Json) -> R<Money> {
    let mut values = Money::new();
    for name in [
        "statutory_myr",
        "total_repayments",
        "shortfall",
        "benchmark_rate",
    ] {
        if let Some(value) = result.get(name) {
            let parsed = crate::wire::parse_decimal(value, &pts(name))
                .map_err(|e| Exc::new("lodgeitadapter.decimals.DecimalWireError", e.0))?;
            put(&mut values, &pts(name), parsed);
        }
    }
    Ok(values)
}

/// `evaluate(case, client)`: one case on both sides, and what the
/// difference means.
pub fn evaluate(case: &Case, client: Option<&mut Client<'_>>) -> R<Comparison> {
    let (verdict, values, reasons) = run_local(case)?;
    let base = Comparison {
        case_id: pts(&case.case_id),
        expected: case.expected.clone(),
        local_verdict: Some(pts(&verdict)),
        reasons,
        ..Comparison::default()
    };
    if verdict == "UNMAPPED_PERIOD" {
        return Ok(Comparison {
            evaluation: Evaluation::UnsupportedPeriod,
            ..base
        });
    }
    if verdict == "REFUSED" || verdict == "UNKNOWN" {
        return Ok(Comparison {
            evaluation: Evaluation::LocalRefused,
            local: values,
            ..base
        });
    }
    let Some(client) = client else {
        return Ok(Comparison {
            evaluation: Evaluation::NotRun,
            local: values,
            reasons: vec![pts(
                "No client supplied: the local side ran and the provider was not called.",
            )],
            ..base
        });
    };
    let outcome = client.invoke(&pts(CALC_URI), &pts(&case.period_uri), &build_request(case))?;
    evaluate_outcome(case, &verdict, &values, &base.reasons, &outcome)
}

/// `evaluate_outcome(...)`: the evaluation half, separated so it can be
/// driven from a fixture.
pub fn evaluate_outcome(
    case: &Case,
    local_verdict: &str,
    local_values: &Money,
    local_reasons: &[S],
    outcome: &Outcome,
) -> R<Comparison> {
    let common = Comparison {
        case_id: pts(&case.case_id),
        expected: case.expected.clone(),
        local: local_values.clone(),
        local_verdict: Some(pts(local_verdict)),
        upstream_status: Some(pts(outcome.status.as_str())),
        ..Comparison::default()
    };
    let findings = || outcome.findings.clone();
    let with = |evaluation, reasons: Vec<S>| Comparison {
        evaluation,
        reasons,
        ..common.clone()
    };
    match outcome.status {
        Status::UpstreamNotFound => {
            let mut reasons = vec![p!("the provider does not accept ", case.period_uri)];
            reasons.extend(findings());
            return Ok(with(Evaluation::UnsupportedPeriod, reasons));
        }
        Status::UpstreamUnavailable | Status::UpstreamThrottled | Status::RefusedToSend => {
            return Ok(with(Evaluation::UpstreamUnavailable, findings()));
        }
        Status::UpstreamRefused => {
            let class = match &outcome.upstream_refusal_class {
                value if value.truthy() => str_points(value),
                _ => pts("no class given"),
            };
            let mut reasons = vec![p!("the provider refused this case: ", class)];
            reasons.extend(findings());
            return Ok(with(Evaluation::ScopeMismatch, reasons));
        }
        Status::Computed => {}
        _ => return Ok(with(Evaluation::ContractFailure, findings())),
    }
    let result = if outcome.result.truthy() {
        &outcome.result
    } else {
        &Json::Null
    };
    let upstream = extract_upstream(result)?;
    let (mut evaluation, differences, mut extra) =
        compare_values(&case.expected, local_values, &upstream, &super::cent())?;
    let recorded = result
        .get("remaining_term_years")
        .filter(|v| **v != Json::Null);
    if let Some(term) = recorded {
        let supplied = pts(&case.remaining_term_years.to_string());
        if str_points(term) != supplied {
            extra.push(p!(
                "remaining term: the provider used ",
                str_points(term),
                ", the case supplied ",
                supplied,
                ". A term difference is a convention difference, not an arithmetic one."
            ));
            evaluation = Evaluation::ScopeMismatch;
        }
    }
    if result.get("is_complying").is_some() {
        extra.push(pts(
            "the provider returned is_complying; it is its own label for its own arithmetic \
             and is not carried into any local verdict",
        ));
    }
    let mut reasons = local_reasons.to_vec();
    reasons.extend(extra);
    Ok(Comparison {
        evaluation,
        upstream,
        differences,
        reasons,
        ..common
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(id: &str) -> Case {
        let cases = load_cases(None).expect("the packaged cases load");
        cases.into_iter().find(|c| c.case_id == id).expect("a case")
    }

    fn dec(text: &str) -> Decimal {
        Decimal::parse_exact(text).expect("a literal")
    }

    /// `provider_body(case, **overrides)`'s figures, as a computed outcome.
    fn answered(case: &Case, overrides: &[(&str, Json)]) -> Outcome {
        let figure = |name: &str, fallback: &str| {
            let value = super::super::get(&case.expected, &pts(name));
            Json::str(&value.map_or(fallback.to_owned(), Decimal::to_sci_string))
        };
        let mut result = Json::obj([
            ("statutory_myr", figure("statutory_myr", "0.00")),
            ("total_repayments", figure("total_repayments", "0.00")),
            ("shortfall", figure("shortfall", "0.00")),
            ("benchmark_rate", figure("benchmark_rate", "0.0837")),
            (
                "remaining_term_years",
                Json::Int(case.remaining_term_years.into()),
            ),
            ("is_complying", Json::Bool(true)),
        ]);
        if let Json::Object(pairs) = &mut result {
            for (name, value) in overrides {
                let slot = pairs.iter_mut().find(|(k, _)| k.as_str() == Some(name));
                slot.expect("a field").1 = value.clone();
            }
        }
        Outcome {
            status: Status::Computed,
            calculator: None,
            period: None,
            request_body: None,
            request_json: None,
            raw_response: Json::Null,
            http_status: Some(200),
            result,
            values: Vec::new(),
            manifest: Json::Null,
            advisory: Json::Null,
            upstream_refusal_class: Json::Null,
            findings: Vec::new(),
            observed_at: String::new(),
            elapsed_ms: None,
            contract_snapshot: Json::Null,
            contract_sha256: String::new(),
            base_url: None,
        }
    }

    fn compared(case: &Case, outcome: &Outcome) -> Comparison {
        let (verdict, values, reasons) = run_local(case).expect("the local side runs");
        evaluate_outcome(case, &verdict, &values, &reasons, outcome).expect("an evaluation")
    }

    #[test]
    fn every_case_reaches_the_local_verdict_its_fixture_records() {
        for case in load_cases(None).unwrap() {
            let (verdict, _, _) = run_local(&case).unwrap();
            assert_eq!(verdict, case.expect_local, "{}", case.case_id);
        }
    }

    #[test]
    fn the_local_engine_reproduces_the_independently_derived_figures() {
        for id in [
            "D7A-1-met-first-real-year",
            "D7A-2-short-repayment",
            "D7A-3-excess-repayment",
            "D7A-5-earlier-year-rate",
            "D7A-6-term-of-one-year",
            "D7A-7-secured-25-year-term",
        ] {
            let case = case(id);
            let (_, values, _) = run_local(&case).unwrap();
            for (name, want) in &case.expected {
                let have = super::super::get(&values, name).expect("a figure");
                assert!(have.equals(want).unwrap(), "{id}");
            }
        }
    }

    #[test]
    fn a_nil_remaining_term_is_refused_locally_and_nothing_is_compared() {
        let case = case("D7A-8-nil-remaining-term-refused");
        let (verdict, values, reasons) = run_local(&case).unwrap();
        assert_eq!(verdict, "REFUSED");
        assert!(super::super::get(&values, &pts("statutory_myr")).is_none());
        let shown: Vec<String> = reasons
            .iter()
            .map(|r| crate::pyvalue::points_lossy(r))
            .collect();
        assert!(
            shown.iter().any(|r| r.contains("remaining term")),
            "{shown:?}"
        );
        let comparison = evaluate(&case, None).unwrap();
        assert_eq!(comparison.evaluation.as_str(), case.expect_evaluation);
    }

    #[test]
    fn a_match_a_cent_and_a_term_convention_are_told_apart() {
        let met = case("D7A-1-met-first-real-year");
        let comparison = compared(&met, &answered(&met, &[]));
        assert_eq!(comparison.evaluation, Evaluation::Match);
        let statutory = pts("statutory_myr");
        let local = super::super::get(&comparison.local, &statutory).unwrap();
        assert!(local.equals(&dec("21874.92")).unwrap());
        // The compliance label is recorded and never mapped.
        assert!(
            comparison
                .reasons
                .iter()
                .any(|r| { crate::pyvalue::points_lossy(r).contains("is_complying") })
        );

        let short = case("D7A-2-short-repayment");
        let cent = [
            ("statutory_myr", Json::str("21874.90")),
            ("shortfall", Json::str("6874.90")),
        ];
        let comparison = compared(&short, &answered(&short, &cent));
        assert_eq!(comparison.evaluation, Evaluation::NumericDifference);
        let difference = super::super::get(&comparison.differences, &statutory).unwrap();
        assert!(difference.equals(&dec("0.02")).unwrap());

        let term = [("remaining_term_years", Json::Int(5.into()))];
        let comparison = compared(&met, &answered(&met, &term));
        assert_eq!(comparison.evaluation, Evaluation::ScopeMismatch);
    }
}
