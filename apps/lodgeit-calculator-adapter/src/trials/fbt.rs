//! `trials/fbt.py`: FBT car benefits, the provider's statutory formula
//! against the local aggregate worksheet.
//!
//! Only the provider's category taxable value crosses into the local
//! worksheet, which grosses up what it is given; the provider's grossed-up
//! figures are recorded and compared, never consumed. The local worksheet is
//! the in-tree Rust `australian-tax-calculators`, in place of the Python
//! release the trial pinned; it also returns the return items 14A, 14B, 15
//! and 16, which appear in `local` and are not compared.
use australian_tax_calculators::engine::{self, Arg};

use super::{Comparison, Evaluation, Money, dict, get, put, shown};
use crate::client::{Client, Outcome, Status};
use crate::decimal::{Decimal, Signal};
use crate::errors::CONTRACT;
use crate::p;
use crate::pyjson::Json;
use crate::pyops::{Exc, R, S, decimal_signal, pts};

pub const CALC_URI: &str = "urn:sbrm:calculator:fbt:car-statutory-formula";
pub const PERIOD_URI: &str = "urn:sbrm:period:fbt:fy2026";
const CLASSIFICATION_REQUIRED: &str = "lodgeitadapter.trials.fbt.ClassificationRequired";

/// The provider's FBT year against the local worksheet's `year_ended`: the
/// provider's fy2026 is the FBT year ending 31 March 2026.
pub const PERIOD_MAP: [(&str, i64); 1] = [(PERIOD_URI, 2026)];

/// Fields that are already grossed up. None may reach the local worksheet.
pub const ALREADY_GROSSED_UP: [&str; 2] = ["grossed_up_taxable_value", "fbt_payable"];

/// A fabricated car benefit, with its reviewed classification.
#[derive(Clone, Debug)]
pub struct CarBenefit {
    pub case_id: String,
    pub base_value: Decimal,
    pub days_available: i64,
    pub accessories: Decimal,
    pub employee_contribution: Decimal,
    /// "Type 1", "Type 2" or "unknown".
    pub fbt_type: String,
    pub period_uri: String,
}

impl CarBenefit {
    /// `request()`: the provider body, refused until the classification is
    /// established.
    pub fn request(&self) -> R<Json> {
        if self.fbt_type != "Type 1" && self.fbt_type != "Type 2" {
            return Err(Exc::new(
                CLASSIFICATION_REQUIRED,
                pts(&format!(
                    "{}: fbt_type is {}. Type 1 and type 2 turn on whether the employer was \
                     entitled to an input tax credit for the acquisition, which is a reviewed \
                     judgement. The provider defaults to Type 2 when the field is omitted; this \
                     trial will not inherit that default.",
                    self.case_id,
                    crate::py::repr_str(&self.fbt_type)
                )),
            ));
        }
        Ok(Json::obj([
            ("baseValue", Json::Dec(self.base_value.clone())),
            ("daysAvailable", Json::Int(self.days_available.into())),
            ("accessories", Json::Dec(self.accessories.clone())),
            (
                "employeeContribution",
                Json::Dec(self.employee_contribution.clone()),
            ),
            ("fbtType", Json::str(&self.fbt_type)),
        ]))
    }
}

/// `prepare_aggregate_input(taxable_value, fbt_type, field_name=...,
/// response_values=...)`: the type 1 and type 2 values, refusing anything
/// that is not a category taxable value, by name or by figure.
pub fn prepare_aggregate_input(
    taxable_value: &Decimal,
    fbt_type: &str,
    field_name: &str,
    response_values: &Money,
) -> R<(Decimal, Decimal)> {
    let response_values = &dict(response_values);
    let calc = |r: crate::decimal::Calc<bool>| r.map_err(decimal_signal);
    if ALREADY_GROSSED_UP.contains(&field_name) {
        return Err(Exc::msg(
            CONTRACT,
            &format!(
                "{field_name} is already grossed up. Passing it to the aggregate worksheet would \
                 gross it up a second time. Use the category taxable_value."
            ),
        ));
    }
    let zero = Decimal::from_int(0);
    for name in ALREADY_GROSSED_UP {
        if let Some(recorded) = get(response_values, &pts(name))
            && calc(recorded.equals(taxable_value))?
            && !calc(taxable_value.equals(&zero))?
        {
            return Err(Exc::msg(
                CONTRACT,
                &format!(
                    "the value supplied as {field_name} is the same figure the provider returned \
                     as {name}, which is already grossed up. Grossing it up again would overstate \
                     the liability. Use the category taxable_value."
                ),
            ));
        }
    }
    let none = Decimal::parse_exact("0.00").expect("a literal");
    match fbt_type {
        "Type 1" => Ok((taxable_value.clone(), none)),
        "Type 2" => Ok((none, taxable_value.clone())),
        _ => Err(Exc::msg(
            CLASSIFICATION_REQUIRED,
            &format!(
                "fbt_type {} is not an established classification; the aggregate worksheet \
                 cannot be called until it is.",
                crate::py::repr_str(fbt_type)
            ),
        )),
    }
}

/// `run_local_aggregate(arguments, year_ended)`: the local aggregate
/// worksheet on established taxable values.
pub fn run_local_aggregate(arguments: &(Decimal, Decimal), year_ended: i64) -> R<Money> {
    let engine_decimal = |value: &Decimal| {
        Arg::Decimal(
            australian_tax_calculators::decimal::Decimal::parse(&value.to_sci_string())
                .expect("an adapter Decimal reads back"),
        )
    };
    let result = engine::fbt(
        &engine_decimal(&arguments.0),
        &engine_decimal(&arguments.1),
        &Arg::Int(year_ended),
        &Arg::Bool(true),
    )
    .map_err(|error| match error {
        engine::Error::Value(message) => Exc::msg("ValueError", &message),
        engine::Error::Decimal(signal) => decimal_signal(match signal.name() {
            "DivisionByZero" => Signal::DivisionByZero,
            "Overflow" => Signal::Overflow,
            _ => Signal::InvalidOperation,
        }),
    })?;
    let mut amounts = Money::new();
    for (name, value) in result["amounts"]
        .as_object()
        .expect("the worksheet's amounts")
    {
        let text = value.as_str().expect("an amount is a str");
        put(
            &mut amounts,
            &pts(name),
            Decimal::parse_exact(text).expect("a worksheet amount reads"),
        );
    }
    Ok(amounts)
}

/// `evaluate(benefit, client)`: fetch one category taxable value and carry
/// it into the local worksheet.
pub fn evaluate(benefit: &CarBenefit, client: Option<&mut Client<'_>>) -> R<Comparison> {
    let common = Comparison {
        case_id: pts(&benefit.case_id),
        ..Comparison::default()
    };
    if !PERIOD_MAP.iter().any(|(uri, _)| *uri == benefit.period_uri) {
        return Ok(Comparison {
            evaluation: Evaluation::UnsupportedPeriod,
            reasons: vec![p!(
                benefit.period_uri,
                " has no local counterpart; the local worksheet supports only the FBT years in \
                 PERIOD_MAP"
            )],
            ..common
        });
    }
    let body = match benefit.request() {
        Ok(body) => body,
        Err(error) if error.class == CLASSIFICATION_REQUIRED => {
            return Ok(Comparison {
                evaluation: Evaluation::ScopeMismatch,
                reasons: vec![error.text],
                ..common
            });
        }
        Err(error) => return Err(error),
    };
    let Some(client) = client else {
        return Ok(Comparison {
            evaluation: Evaluation::NotRun,
            reasons: vec![pts("No client supplied; the provider was not called.")],
            ..common
        });
    };
    let outcome = client.invoke(&pts(CALC_URI), &pts(&benefit.period_uri), &body)?;
    evaluate_outcome(benefit, &outcome)
}

/// `evaluate_outcome(benefit, outcome)`.
pub fn evaluate_outcome(benefit: &CarBenefit, outcome: &Outcome) -> R<Comparison> {
    let common = Comparison {
        case_id: pts(&benefit.case_id),
        upstream_status: Some(pts(outcome.status.as_str())),
        ..Comparison::default()
    };
    let with = |evaluation, reasons: Vec<S>| Comparison {
        evaluation,
        reasons,
        ..common.clone()
    };
    match outcome.status {
        Status::UpstreamNotFound => {
            return Ok(with(
                Evaluation::UnsupportedPeriod,
                outcome.findings.clone(),
            ));
        }
        Status::UpstreamUnavailable | Status::UpstreamThrottled | Status::RefusedToSend => {
            return Ok(with(
                Evaluation::UpstreamUnavailable,
                outcome.findings.clone(),
            ));
        }
        Status::UpstreamRefused => {
            return Ok(with(Evaluation::ScopeMismatch, outcome.findings.clone()));
        }
        Status::Computed => {}
        _ => return Ok(with(Evaluation::ContractFailure, outcome.findings.clone())),
    }
    let values: Money = dict(&outcome.values);
    let taxable = pts("taxable_value");
    let Some(category_value) = get(&values, &taxable).cloned() else {
        let mut reasons = vec![pts(
            "the response carries no taxable_value, so there is nothing to carry into the \
             aggregate worksheet",
        )];
        reasons.extend(outcome.findings.iter().cloned());
        return Ok(with(Evaluation::ContractFailure, reasons));
    };
    let arguments =
        prepare_aggregate_input(&category_value, &benefit.fbt_type, "taxable_value", &values)?;
    // `PERIOD_MAP[benefit.period_uri]`: an outcome for a period `evaluate`
    // would have refused raises KeyError, as the subscript does.
    let Some((_, year_ended)) = PERIOD_MAP
        .iter()
        .find(|(uri, _)| *uri == benefit.period_uri)
    else {
        return Err(Exc::msg(
            "KeyError",
            &crate::py::repr_str(&benefit.period_uri),
        ));
    };
    let local = run_local_aggregate(&arguments, *year_ended)?;
    let mut reasons = outcome.findings.clone();

    let mut upstream = Money::new();
    put(&mut upstream, &taxable, category_value);
    for name in ALREADY_GROSSED_UP {
        if let Some(value) = get(&values, &pts(name)) {
            put(&mut upstream, &pts(name), value.clone());
        }
    }
    let local_grossed = if benefit.fbt_type == "Type 1" {
        "type_one_grossed_up"
    } else {
        "type_two_grossed_up"
    };
    let mut compared: Vec<(&str, Decimal, Decimal)> = Vec::new();
    for (upstream_name, local_name) in [
        ("grossed_up_taxable_value", local_grossed),
        ("fbt_payable", "fbt_estimate"),
    ] {
        match (
            get(&upstream, &pts(upstream_name)),
            get(&local, &pts(local_name)),
        ) {
            (Some(theirs), Some(mine)) => {
                compared.push((upstream_name, mine.clone(), theirs.clone()));
            }
            (Some(_), None) => reasons.push(pts(&format!(
                "{upstream_name}: the provider returned a figure and the local worksheet \
                 produced no {local_name}"
            ))),
            (None, Some(_)) => reasons.push(pts(&format!(
                "{local_name}: the local worksheet produced a figure and the provider returned \
                 no {upstream_name}"
            ))),
            (None, None) => {}
        }
    }
    compared.sort_by(|a, b| a.0.cmp(b.0));
    let tolerance = super::cent();
    let mut differences = Money::new();
    for (name, mine, theirs) in &compared {
        let difference = mine.sub(theirs).map_err(decimal_signal)?;
        if difference
            .copy_abs()
            .gt(&tolerance)
            .map_err(decimal_signal)?
        {
            put(&mut differences, &pts(name), difference);
            reasons.push(p!(
                *name,
                ": local ",
                shown(mine),
                " against provider ",
                shown(theirs),
                ". Check the gross-up rate and the FBT rate each side used."
            ));
        }
    }
    let evaluation = if !differences.is_empty() {
        Evaluation::NumericDifference
    } else if compared.is_empty() {
        reasons.push(pts(
            "the provider returned neither a grossed-up amount nor an FBT payable, so the only \
             figure that crossed was never checked against anything",
        ));
        Evaluation::ScopeMismatch
    } else if compared.len() < 2 {
        Evaluation::ScopeMismatch
    } else {
        reasons.push(p!(
            "only taxable_value crossed into the aggregate worksheet, as a ",
            benefit.fbt_type,
            " amount"
        ));
        Evaluation::Match
    };
    Ok(Comparison {
        evaluation,
        local,
        upstream,
        differences,
        reasons,
        ..common
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(text: &str) -> Decimal {
        Decimal::parse_exact(text).expect("a literal")
    }

    fn benefit(fbt_type: &str) -> CarBenefit {
        CarBenefit {
            case_id: "FBT-1".to_owned(),
            base_value: dec("60000.00"),
            days_available: 365,
            accessories: dec("0.00"),
            employee_contribution: dec("0.00"),
            fbt_type: fbt_type.to_owned(),
            period_uri: PERIOD_URI.to_owned(),
        }
    }

    fn computed(values: &[(&str, &str)]) -> Outcome {
        Outcome {
            status: Status::Computed,
            calculator: None,
            period: None,
            request_body: None,
            request_json: None,
            raw_response: Json::Null,
            http_status: Some(200),
            result: Json::Null,
            values: values.iter().map(|(k, v)| (pts(k), dec(v))).collect(),
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

    #[test]
    fn an_unestablished_classification_is_never_sent() {
        let error = benefit("unknown").request().expect_err("refused");
        assert_eq!(error.class, CLASSIFICATION_REQUIRED);
        let compared = evaluate(&benefit("unknown"), None).expect("a comparison");
        assert!(matches!(compared.evaluation, Evaluation::ScopeMismatch));
        assert!(matches!(
            evaluate(&benefit("Type 1"), None)
                .expect("a comparison")
                .evaluation,
            Evaluation::NotRun
        ));
    }

    #[test]
    fn a_grossed_up_figure_is_refused_by_name_and_by_value() {
        let none = Money::new();
        let by_name = prepare_aggregate_input(&dec("10.00"), "Type 1", "fbt_payable", &none);
        assert_eq!(by_name.expect_err("refused").class, CONTRACT);
        let recorded = vec![(pts("grossed_up_taxable_value"), dec("10.0"))];
        let by_value = prepare_aggregate_input(&dec("10.00"), "Type 1", "taxable_value", &recorded);
        assert_eq!(by_value.expect_err("refused").class, CONTRACT);
        let zero = vec![(pts("fbt_payable"), dec("0"))];
        let (one, two) = prepare_aggregate_input(&dec("0.00"), "Type 2", "taxable_value", &zero)
            .expect("a nil figure is not mistaken for a grossed-up one");
        assert_eq!(
            (one.to_sci_string(), two.to_sci_string()),
            ("0.00".into(), "0.00".into())
        );
    }

    #[test]
    fn the_local_worksheet_agrees_with_the_provider_within_a_cent() {
        let outcome = computed(&[
            ("taxable_value", "8500.42"),
            ("grossed_up_taxable_value", "16038.60"),
            ("fbt_payable", "7538.14"),
        ]);
        let compared = evaluate_outcome(&benefit("Type 2"), &outcome).expect("a comparison");
        assert!(matches!(compared.evaluation, Evaluation::Match));
        let local = |name: &str| get(&compared.local, &pts(name)).map(Decimal::to_sci_string);
        assert_eq!(local("type_two_grossed_up").as_deref(), Some("16038.59"));
        assert_eq!(local("fbt_estimate").as_deref(), Some("7538.14"));

        let off = computed(&[("taxable_value", "8500.42"), ("fbt_payable", "7538.16")]);
        let compared = evaluate_outcome(&benefit("Type 2"), &off).expect("a comparison");
        assert!(matches!(compared.evaluation, Evaluation::NumericDifference));
        assert_eq!(
            compared.differences,
            vec![(pts("fbt_payable"), dec("-0.02"))]
        );
    }

    #[test]
    fn an_outcome_for_an_unmapped_period_raises_key_error() {
        let mut other = benefit("Type 1");
        other.period_uri = "urn:sbrm:period:fbt:fy2025".to_owned();
        let outcome = computed(&[("taxable_value", "100.00")]);
        let error = evaluate_outcome(&other, &outcome).expect_err("no local year");
        assert_eq!(error.class, "KeyError");
        assert_eq!(error.text, pts("'urn:sbrm:period:fbt:fy2025'"));
    }

    /// Python's values are a dict: a repeated name holds its last figure, so
    /// the guard against a second gross-up sees the figure that stands.
    #[test]
    fn a_repeated_figure_is_read_as_its_last_value() {
        let refused = computed(&[
            ("taxable_value", "8500.42"),
            ("grossed_up_taxable_value", "16038.60"),
            ("fbt_payable", "7538.14"),
            ("grossed_up_taxable_value", "8500.42"),
        ]);
        let error = evaluate_outcome(&benefit("Type 2"), &refused).expect_err("refused");
        assert_eq!(error.class, CONTRACT);

        let agreed = computed(&[
            ("taxable_value", "8500.42"),
            ("grossed_up_taxable_value", "8500.42"),
            ("fbt_payable", "7538.14"),
            ("grossed_up_taxable_value", "16038.60"),
        ]);
        let compared = evaluate_outcome(&benefit("Type 2"), &agreed).expect("a comparison");
        assert_eq!(compared.evaluation, Evaluation::Match);
        assert_eq!(
            get(&compared.upstream, &pts("grossed_up_taxable_value")).map(Decimal::to_sci_string),
            Some("16038.60".into())
        );

        let recorded = vec![
            (pts("fbt_payable"), dec("1.00")),
            (pts("fbt_payable"), dec("10.00")),
        ];
        let by_value = prepare_aggregate_input(&dec("10.00"), "Type 1", "taxable_value", &recorded);
        assert_eq!(by_value.expect_err("refused").class, CONTRACT);
        let recorded = vec![
            (pts("fbt_payable"), dec("10.00")),
            (pts("fbt_payable"), dec("1.00")),
        ];
        assert!(
            prepare_aggregate_input(&dec("10.00"), "Type 1", "taxable_value", &recorded).is_ok()
        );
    }
}
