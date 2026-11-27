//! The Division 7A trial's origination year and the computed-answer half of
//! `evaluate_outcome`. Expected values are the retired Python adapter's
//! `_origination_year_label` and `evaluate_outcome` under CPython 3.15.0
//! (win32; no platform-specific code runs in these paths). The local side
//! runs the in-tree engine, as `trials_div7a.rs` does.
use lodgeit_calculator_adapter::client::{Outcome, Status};
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::pyjson::Json;
use lodgeit_calculator_adapter::pyvalue::points_lossy;
use lodgeit_calculator_adapter::trials::div7a::{self as trial, Case};
use lodgeit_calculator_adapter::trials::{Evaluation, Money};

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn case(id: &str) -> Case {
    trial::load_cases(None)
        .expect("the packaged cases load")
        .into_iter()
        .find(|c| c.case_id == id)
        .expect("a case")
}

fn made_on(date: &str) -> Case {
    Case {
        loan_origination_date: date.to_owned(),
        ..case("D7A-1-met-first-real-year")
    }
}

/// The local side's verdict, figures and reasons, as text.
fn local(case: &Case) -> (String, Vec<(String, String)>, Vec<String>) {
    let (verdict, values, reasons) = trial::run_local(case).expect("the local side runs");
    (
        verdict,
        values
            .iter()
            .map(|(name, value)| (points_lossy(name), value.to_sci_string()))
            .collect(),
        reasons.iter().map(|r| points_lossy(r)).collect(),
    )
}

fn computed(result: Json) -> Outcome {
    Outcome {
        status: Status::Computed,
        calculator: None,
        period: None,
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: None,
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

fn one_figure() -> Money {
    vec![(
        pts("statutory_myr"),
        Decimal::parse_exact("1.00").expect("a literal"),
    )]
}

#[test]
fn a_loan_made_before_july_belongs_to_the_income_year_that_began_the_july_before() {
    // Python labels 2024-09-01, 2025-03-01 and 2025-06-30 "2024-25" and
    // 2025-07-01 "2025-26".
    let september = local(&made_on("2024-09-01"));
    assert_eq!(september.0, "MYR_MET");
    for date in ["2025-03-01", "2025-06-30"] {
        assert_eq!(local(&made_on(date)), september, "{date}");
    }
    assert_ne!(local(&made_on("2025-07-01")), september);
}

#[test]
fn a_computed_answer_with_no_result_compares_nothing_and_says_why() {
    // `outcome.result or {}`: a null and an empty result read the same.
    let case = case("D7A-1-met-first-real-year");
    let expected = [
        "local reason",
        "benchmark_rate: the local engine produced no figure",
        "benchmark_rate: the provider returned no figure",
        "shortfall: the local engine produced no figure",
        "shortfall: the provider returned no figure",
        "statutory_myr: local 1.00 differs from the independently derived 19450.97",
        "statutory_myr: the provider returned no figure",
        "total_repayments: the local engine produced no figure",
        "total_repayments: the provider returned no figure",
        "statutory_myr: the local engine produced a figure and the provider did not",
        "no field is present on both sides, so nothing was compared",
    ];
    for result in [Json::Null, Json::Object(Vec::new())] {
        let comparison = trial::evaluate_outcome(
            &case,
            "MYR_MET",
            &one_figure(),
            &[pts("local reason")],
            &computed(result.clone()),
        )
        .expect("a comparison");
        assert_eq!(
            comparison.evaluation,
            Evaluation::ScopeMismatch,
            "{result:?}"
        );
        assert!(comparison.upstream.is_empty(), "{result:?}");
        assert!(comparison.differences.is_empty(), "{result:?}");
        let reasons: Vec<String> = comparison.reasons.iter().map(|r| points_lossy(r)).collect();
        assert_eq!(reasons, expected, "{result:?}");
    }
}

#[test]
fn a_signalling_nan_among_the_expected_figures_raises_invalid_operation() {
    // `local[name] != expected[name]` with a signalling NaN.
    let case = Case {
        expected: vec![(
            pts("statutory_myr"),
            Decimal::parse_exact("sNaN").expect("a literal"),
        )],
        ..case("D7A-1-met-first-real-year")
    };
    let error =
        trial::evaluate_outcome(&case, "MYR_MET", &one_figure(), &[], &computed(Json::Null))
            .expect_err("the comparison raises");
    assert_eq!(error.class, "decimal.InvalidOperation");
    assert_eq!(
        points_lossy(&error.text),
        "[<class 'decimal.InvalidOperation'>]"
    );
}
