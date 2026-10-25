//! `trials/__init__.py`: comparison trials, this repository's engines
//! against the LodgeiT calculators.
//!
//! A trial is not a test of either engine. Neither is the oracle. Each case
//! carries its own expected arithmetic, derived by hand from the statute or
//! the standard, and a disagreement is traced back to that rather than to
//! whichever engine is more convenient to believe.
//!
//! The evaluation outcomes are deliberately not the accounting-review states
//! used elsewhere in this repository. A `MATCH` here says two
//! implementations agreed on a number. It says nothing about whether a loan
//! complies, whether a benefit is reportable, or whether a reviewer should
//! sign anything.
pub mod depreciation;
pub mod div7a;
pub mod fano;
pub mod fbt;

use crate::decimal::Decimal;
use crate::p;
use crate::pyjson::Json;
use crate::pyops::{R, S, decimal_signal, json_str, pts};

/// A `dict[str, Decimal]`, in insertion order with unique keys.
pub type Money = Vec<(S, Decimal)>;

/// `values[name]`.
pub(crate) fn get<'a>(values: &'a Money, name: &[u32]) -> Option<&'a Decimal> {
    values.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

/// `values[name] = value`.
pub(crate) fn put(values: &mut Money, name: &[u32], value: Decimal) {
    match values.iter_mut().find(|(k, _)| k == name) {
        Some(entry) => entry.1 = value,
        None => values.push((name.to_vec(), value)),
    }
}

/// `sorted(values)`.
fn keys(values: &Money) -> Vec<&S> {
    let mut names: Vec<&S> = values.iter().map(|(k, _)| k).collect();
    names.sort();
    names
}

/// The outcome of comparing one case. Not an accounting verdict.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Evaluation {
    Match,
    NumericDifference,
    ScopeMismatch,
    UnsupportedPeriod,
    ContractFailure,
    UpstreamUnavailable,
    LocalRefused,
    #[default]
    NotRun,
}

impl Evaluation {
    pub fn as_str(self) -> &'static str {
        match self {
            Evaluation::Match => "MATCH",
            Evaluation::NumericDifference => "NUMERIC_DIFFERENCE",
            Evaluation::ScopeMismatch => "SCOPE_MISMATCH",
            Evaluation::UnsupportedPeriod => "UNSUPPORTED_PERIOD",
            Evaluation::ContractFailure => "CONTRACT_FAILURE",
            Evaluation::UpstreamUnavailable => "UPSTREAM_UNAVAILABLE",
            Evaluation::LocalRefused => "LOCAL_REFUSED",
            Evaluation::NotRun => "NOT_RUN",
        }
    }
}

/// One case, both sides, and what the difference means.
#[derive(Clone, Debug, Default)]
pub struct Comparison {
    pub case_id: S,
    pub evaluation: Evaluation,
    pub expected: Money,
    pub local: Money,
    pub upstream: Money,
    pub differences: Money,
    pub reasons: Vec<S>,
    pub local_verdict: Option<S>,
    pub upstream_status: Option<S>,
}

impl Comparison {
    /// `to_json_dict()`.
    pub fn to_json(&self) -> Json {
        let money = |values: &Money| {
            let mut pairs: Vec<&(S, Decimal)> = values.iter().collect();
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            Json::Object(
                pairs
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            crate::pyjson::PyStr::from_code_points(k.clone()),
                            Json::str(&v.fixed_format()),
                        )
                    })
                    .collect(),
            )
        };
        let optional = |value: &Option<S>| value.clone().map_or(Json::Null, json_str);
        Json::obj([
            ("case_id", json_str(self.case_id.clone())),
            ("evaluation", Json::str(self.evaluation.as_str())),
            ("expected", money(&self.expected)),
            ("local", money(&self.local)),
            ("upstream", money(&self.upstream)),
            ("differences", money(&self.differences)),
            (
                "reasons",
                Json::Array(self.reasons.iter().cloned().map(json_str).collect()),
            ),
            ("local_verdict", optional(&self.local_verdict)),
            ("upstream_status", optional(&self.upstream_status)),
            (
                "boundary",
                Json::str(
                    "An evaluation outcome compares arithmetic. It is not a readiness, close or \
                     ledger-review state and it approves nothing.",
                ),
            ),
        ])
    }
}

/// `str(value)`.
pub(crate) fn shown(value: &Decimal) -> S {
    pts(&value.to_sci_string())
}

/// `compare_values(expected, local, upstream, tolerance=...)`: compare the
/// fields present on both sides against the expected figures.
///
/// A field missing from either side is a reason, not a silent pass. The
/// tolerance is for the last cent only: anything wider hides a method
/// difference, which is the thing a trial is for.
pub fn compare_values(
    expected: &Money,
    local: &Money,
    upstream: &Money,
    tolerance: &Decimal,
) -> R<(Evaluation, Money, Vec<S>)> {
    let mut reasons: Vec<S> = Vec::new();
    let mut differences = Money::new();
    for name in keys(expected) {
        let want = get(expected, name).expect("a key of expected");
        match get(local, name) {
            None => reasons.push(p!(name, ": the local engine produced no figure")),
            Some(have) if !have.equals(want).map_err(decimal_signal)? => reasons.push(p!(
                name,
                ": local ",
                shown(have),
                " differs from the independently derived ",
                shown(want)
            )),
            Some(_) => {}
        }
        if get(upstream, name).is_none() {
            reasons.push(p!(name, ": the provider returned no figure"));
        }
    }
    // A field one side produced and the other did not is a coverage
    // difference and is named, whether or not the case expected it. Leaving
    // it out let a trial report MATCH on a field nobody compared.
    for name in keys(local) {
        if get(upstream, name).is_none() {
            reasons.push(p!(
                name,
                ": the local engine produced a figure and the provider did not"
            ));
        }
    }
    for name in keys(upstream) {
        if get(local, name).is_none() {
            reasons.push(p!(
                name,
                ": the provider returned a figure and the local engine did not"
            ));
        }
    }
    let shared: Vec<&S> = keys(local)
        .into_iter()
        .filter(|name| get(upstream, name).is_some())
        .collect();
    if shared.is_empty() {
        reasons.push(pts(
            "no field is present on both sides, so nothing was compared",
        ));
        return Ok((Evaluation::ScopeMismatch, differences, reasons));
    }
    for name in shared {
        let (mine, theirs) = (get(local, name).unwrap(), get(upstream, name).unwrap());
        let difference = mine.sub(theirs).map_err(decimal_signal)?;
        if difference
            .copy_abs()
            .gt(tolerance)
            .map_err(decimal_signal)?
        {
            put(&mut differences, name, difference);
            reasons.push(p!(
                name,
                ": local ",
                shown(mine),
                " against provider ",
                shown(theirs)
            ));
        }
    }
    if !differences.is_empty() {
        return Ok((Evaluation::NumericDifference, differences, reasons));
    }
    if !reasons.is_empty() {
        // Every figure the two sides both produced agreed. What is left is a
        // field one side did not produce, which is a difference in what was
        // covered, not in what was calculated.
        return Ok((Evaluation::ScopeMismatch, differences, reasons));
    }
    Ok((Evaluation::Match, differences, Vec::new()))
}

/// `Decimal("0.01")`, the last-cent tolerance.
pub(crate) fn cent() -> Decimal {
    Decimal::parse_exact("0.01").expect("a literal")
}
