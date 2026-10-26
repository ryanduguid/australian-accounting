//! `tests/test_trials_div7a.py`: the Division 7A trial, the local side
//! against independently derived arithmetic.
//!
//! The local engine runs for real. The provider is a loopback HTTP stub
//! started inside each test and reached through an `allow_loopback` config,
//! or a fabricated `Outcome` passed to `evaluate_outcome`, as the Python
//! tests do. The Rust trial links the in-tree `div7a-loan-review`, where the
//! Python tests ran against the pinned `div7aloan`.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use div7a_loan_review::myr::{GateSummary, MyrFacts, minimum_yearly_repayment};
use div7a_loan_review::rates::frozen_table;
use div7a_loan_review::raw::Raw;
use div7a_loan_review::years::parse_year;
use lodgeit_calculator_adapter::client::{Client, Outcome, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract;
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::pyvalue::points_lossy;
use lodgeit_calculator_adapter::trials::div7a::{self as trial, Case};
use lodgeit_calculator_adapter::trials::{Comparison, Evaluation, Money};
use num_bigint::BigInt;

// ---------------------------------------------------------------------------
// The loopback stub: `conftest.py`'s `stub` and `stub_config` fixtures.

struct Seen {
    method: String,
    path: String,
}

struct StubState {
    status: u16,
    body: Vec<u8>,
    requests: Vec<Seen>,
}

struct Stub {
    base_url: String,
    state: Arc<Mutex<StubState>>,
}

impl Stub {
    /// An HTTP server on 127.0.0.1, answering every request with the reply
    /// set by `respond` and recording what it saw. Its thread ends with the
    /// test process.
    fn start() -> Stub {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().expect("a bound address").port();
        let state = Arc::new(Mutex::new(StubState {
            status: 200,
            body: b"{}".to_vec(),
            requests: Vec::new(),
        }));
        let shared = Arc::clone(&state);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                serve(stream, &shared);
            }
        });
        Stub {
            base_url: format!("http://127.0.0.1:{port}"),
            state,
        }
    }

    fn respond(&self, payload: &Json, status: u16) {
        let mut state = self.state.lock().unwrap();
        state.status = status;
        state.body = pyjson::dumps(payload, Layout::Canonical).into_bytes();
    }

    fn request_count(&self) -> usize {
        self.state.lock().unwrap().requests.len()
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}

fn serve(stream: TcpStream, state: &Mutex<StubState>) {
    let mut reader = BufReader::new(stream.try_clone().expect("a second handle"));
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
        return;
    }
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; length];
    if reader.read_exact(&mut body).is_err() {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let (status, reply) = {
        let mut state = state.lock().unwrap();
        state.requests.push(Seen {
            method: parts.next().unwrap_or_default().to_owned(),
            path: parts.next().unwrap_or_default().to_owned(),
        });
        (state.status, state.body.clone())
    };
    let mut stream = stream;
    let head = format!(
        "HTTP/1.0 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        reason_phrase(status),
        reply.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&reply);
    let _ = stream.flush();
}

/// `LodgeitClient(stub_config, contract)`: enabled, the stub's base URL,
/// `allow_loopback=True`, `max_attempts=1`, the real dialler.
fn with_client<T>(stub: &Stub, run: impl FnOnce(&mut Client<'_>) -> T) -> T {
    let config = AdapterConfig {
        enabled: true,
        base_url: Some(stub.base_url.clone()),
        allow_loopback: true,
        max_attempts: BigInt::from(1),
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a loopback config");
    let contract = contract::load("lodgeit-calculators", None).expect("the snapshot loads");
    let mut net = Net;
    let mut sleep = dial::sleep;
    let mut client = Client {
        config: &config,
        contract: &contract,
        dial: &mut net,
        sleep: &mut sleep,
    };
    run(&mut client)
}

// ---------------------------------------------------------------------------
// Helpers.

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn dec(text: &str) -> Decimal {
    Decimal::parse_exact(text).expect("a literal")
}

fn money<'a>(values: &'a Money, name: &str) -> Option<&'a Decimal> {
    let name = pts(name);
    values.iter().find(|(k, _)| *k == name).map(|(_, v)| v)
}

fn equals(value: Option<&Decimal>, want: &str) -> bool {
    value.is_some_and(|v| v.equals(&dec(want)).expect("comparable"))
}

fn reasons(comparison: &Comparison) -> Vec<String> {
    comparison.reasons.iter().map(|r| points_lossy(r)).collect()
}

fn mentions(lines: &[Vec<u32>], needle: &str) -> bool {
    lines.iter().any(|line| points_lossy(line).contains(needle))
}

fn case(id: &str) -> Case {
    trial::load_cases(None)
        .expect("the packaged cases load")
        .into_iter()
        .find(|c| c.case_id == id)
        .expect("a case")
}

fn set(body: &mut Json, key: &str, value: Json) {
    let Json::Object(pairs) = body else {
        panic!("an object")
    };
    match pairs.iter_mut().find(|(k, _)| k.as_str() == Some(key)) {
        Some(slot) => slot.1 = value,
        None => {
            let Json::Str(name) = Json::str(key) else {
                unreachable!()
            };
            pairs.push((name, value));
        }
    }
}

fn remove(body: &mut Json, key: &str) {
    let Json::Object(pairs) = body else {
        panic!("an object")
    };
    pairs.retain(|(k, _)| k.as_str() != Some(key));
}

/// `provider_body(case, **overrides)`: a provider response shaped like the
/// contract, from the case's own expectations.
fn provider_body(case: &Case, overrides: &[(&str, Json)]) -> Json {
    let figure = |name: &str, fallback: &str| {
        Json::str(&money(&case.expected, name).map_or(fallback.to_owned(), Decimal::to_sci_string))
    };
    let mut body = Json::obj([
        ("statutory_myr", figure("statutory_myr", "0.00")),
        ("total_repayments", figure("total_repayments", "0.00")),
        ("shortfall", figure("shortfall", "0.00")),
        ("benchmark_rate", figure("benchmark_rate", "0.0837")),
        (
            "remaining_term_years",
            Json::Int(case.remaining_term_years.into()),
        ),
        ("is_complying", Json::Bool(true)),
        (
            "manifest",
            Json::obj([
                ("calculator", Json::str(trial::CALC_URI)),
                ("period", Json::str(&case.period_uri)),
                (
                    "rate_table_uris",
                    Json::Array(vec![Json::obj([
                        (
                            "uri",
                            Json::str("urn:sbrm:rate:div7a:fy2026:benchmark-interest"),
                        ),
                        ("sha256", Json::str(&"0".repeat(64))),
                    ])]),
                ),
                ("engine", Json::str("div7a/1.0.0")),
                ("citation", Json::str("ITAA 1936 s 109E")),
            ]),
        ),
        (
            "advisory",
            Json::obj([
                ("figure_type", Json::str("div7a_myr")),
                ("notes", Json::Array(vec![Json::str("Tax agent framing.")])),
            ]),
        ),
    ]);
    for (name, value) in overrides {
        set(&mut body, name, value.clone());
    }
    body
}

/// `Outcome(status=status, findings=("fabricated finding",))`.
fn fabricated(status: Status) -> Outcome {
    Outcome {
        status,
        calculator: None,
        period: None,
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: None,
        result: Json::Null,
        values: Vec::new(),
        manifest: Json::Null,
        advisory: Json::Null,
        upstream_refusal_class: Json::Null,
        findings: vec![pts("fabricated finding")],
        observed_at: String::new(),
        elapsed_ms: None,
        contract_snapshot: Json::Null,
        contract_sha256: String::new(),
        base_url: None,
    }
}

fn fixture_path() -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/div7a-cases.json").to_owned()
}

// ---------------------------------------------------------------------------
// The tests, one per Python test function.

#[test]
fn the_case_file_is_synthetic_and_carries_a_derivation_for_every_case() {
    // `trial.cases_path()` is the source checkout's fixture, which the Rust
    // trial embeds at build time.
    let text = std::fs::read_to_string(fixture_path()).expect("the case file reads");
    let payload = pyjson::loads_standard(&text).expect("the case file is JSON");
    assert_eq!(payload.at("synthetic"), &Json::Bool(true));
    let Json::Array(cases) = payload.at("cases") else {
        panic!("cases is a list")
    };
    for case in cases {
        let derivation = case.at("derivation").as_str().unwrap_or_default();
        assert!(
            !derivation.trim().is_empty(),
            "{:?}",
            case.at("case_id").as_str()
        );
    }
}

#[test]
fn the_period_map_agrees_with_the_local_rate_table() {
    // Checked against the live rate documents on 18 September 2026: the
    // provider's fy2026 is 0.0837 and fy2025 is 0.0877.
    let table = frozen_table();
    let expected = [
        ("urn:sbrm:period:div7a:fy2025", "0.0877"),
        ("urn:sbrm:period:div7a:fy2026", "0.0837"),
    ];
    for (period_uri, local_year) in trial::PERIOD_MAP {
        let year = parse_year(&Raw::Str(local_year.to_owned()), "test").expect("a year label");
        let rate = table.lookup(year).rate.expect("a reviewed rate");
        let want = expected
            .iter()
            .find(|(uri, _)| *uri == period_uri)
            .map(|(_, rate)| *rate)
            .expect("an expected rate");
        let want = div7a_loan_review::decimal::Decimal::parse(want).expect("a literal");
        assert!(
            rate.equals(&want).expect("comparable"),
            "{period_uri} -> {local_year}"
        );
    }
}

#[test]
fn the_local_engine_reproduces_the_independently_derived_figure() {
    for case_id in [
        "D7A-1-met-first-real-year",
        "D7A-2-short-repayment",
        "D7A-3-excess-repayment",
        "D7A-5-earlier-year-rate",
        "D7A-6-term-of-one-year",
        "D7A-7-secured-25-year-term",
    ] {
        let case = case(case_id);
        let (verdict, values, _) = trial::run_local(&case).expect("the local side runs");
        assert_eq!(verdict, case.expect_local, "{case_id}");
        for (field, expected) in &case.expected {
            let have = values
                .iter()
                .find(|(k, _)| k == field)
                .map(|(_, v)| v)
                .expect("a figure");
            assert!(
                have.equals(expected).expect("comparable"),
                "{case_id}.{}",
                points_lossy(field)
            );
        }
    }
}

#[test]
fn a_nil_remaining_term_is_refused_by_the_local_engine_not_divided() {
    let stub = Stub::start();
    let case = case("D7A-8-nil-remaining-term-refused");
    let (verdict, values, local_reasons) = trial::run_local(&case).expect("the local side runs");
    assert_eq!(verdict, "REFUSED");
    assert!(money(&values, "statutory_myr").is_none());
    assert!(mentions(&local_reasons, "remaining term"));
    // A local refusal ends the case. Nothing is sent, so the fixture's
    // recorded evaluation is decidable offline.
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert_eq!(comparison.evaluation.as_str(), case.expect_evaluation);
    assert_eq!(stub.request_count(), 0);
}

#[test]
fn a_matching_provider_answer_is_a_match() {
    let stub = Stub::start();
    let case = case("D7A-1-met-first-real-year");
    stub.respond(&provider_body(&case, &[]), 200);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::Match);
    assert!(equals(
        money(&comparison.local, "statutory_myr"),
        "21874.92"
    ));
    assert!(equals(
        money(&comparison.upstream, "statutory_myr"),
        "21874.92"
    ));
    // The stub saw one POST to the snapshot's route.
    let state = stub.state.lock().unwrap();
    assert_eq!(state.requests.len(), 1);
    assert_eq!(state.requests[0].method, "POST");
    assert!(!state.requests[0].path.is_empty());
}

#[test]
fn a_cent_of_disagreement_is_a_numeric_difference_not_a_match() {
    let stub = Stub::start();
    let case = case("D7A-2-short-repayment");
    let overrides = [
        ("statutory_myr", Json::str("21874.90")),
        ("shortfall", Json::str("6874.90")),
    ];
    stub.respond(&provider_body(&case, &overrides), 200);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::NumericDifference);
    assert!(equals(
        money(&comparison.differences, "statutory_myr"),
        "0.02"
    ));
}

#[test]
fn a_term_convention_difference_is_scope_not_arithmetic() {
    let stub = Stub::start();
    let case = case("D7A-1-met-first-real-year");
    let overrides = [("remaining_term_years", Json::Int(5.into()))];
    stub.respond(&provider_body(&case, &overrides), 200);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::ScopeMismatch);
    assert!(mentions(&comparison.reasons, "remaining term"));
}

#[test]
fn the_rate_trap_case_reports_a_difference_and_names_the_rate() {
    // D7A-4 expects the previous year's rate; the engine must use the current
    // one. The provider echoes the term the case supplied, isolating the rate.
    let stub = Stub::start();
    let case = case("D7A-4-later-year-not-first");
    stub.respond(&provider_body(&case, &[]), 200);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::NumericDifference);
    assert!(equals(
        money(&comparison.local, "statutory_myr"),
        "29297.01"
    ));
    assert!(equals(
        money(&comparison.expected, "statutory_myr"),
        "29507.73"
    ));
    assert!(mentions(&comparison.reasons, "29297.01"));
}

#[test]
fn the_rate_trap_case_reaches_the_outcome_its_fixture_records() {
    // On 18 September 2026 the provider derived a remaining term of 4 where
    // the case supplied 3, so the term convention outranks the rate.
    let stub = Stub::start();
    let case = case("D7A-4-later-year-not-first");
    let overrides = [("remaining_term_years", Json::Int(4.into()))];
    stub.respond(&provider_body(&case, &overrides), 200);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert_eq!(comparison.evaluation.as_str(), case.expect_evaluation);
    assert_eq!(comparison.evaluation, Evaluation::ScopeMismatch);
    assert!(mentions(&comparison.reasons, "remaining term"));
}

#[test]
fn an_unsupported_period_is_its_own_outcome() {
    let stub = Stub::start();
    let case = case("D7A-9-unsupported-period");
    stub.respond(&Json::obj([("error", Json::str("not found"))]), 404);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    // Neither side covers fy2024, so nothing is sent.
    assert_eq!(comparison.evaluation, Evaluation::UnsupportedPeriod);
    assert_eq!(comparison.evaluation.as_str(), case.expect_evaluation);
    assert!(comparison.upstream.is_empty() && comparison.local.is_empty());
    assert_eq!(stub.request_count(), 0);
}

#[test]
fn a_provider_outage_never_becomes_agreement() {
    let stub = Stub::start();
    let case = case("D7A-1-met-first-real-year");
    stub.respond(&Json::obj([("detail", Json::str("down"))]), 503);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::UpstreamUnavailable);
    assert!(comparison.upstream.is_empty());
    assert!(
        equals(money(&comparison.local, "statutory_myr"), "21874.92"),
        "the local side still answered"
    );
}

#[test]
fn the_providers_compliance_label_is_recorded_and_never_mapped() {
    let stub = Stub::start();
    let case = case("D7A-1-met-first-real-year");
    let overrides = [
        ("is_complying", Json::Bool(false)),
        ("deemed_dividend", Json::str("12345.67")),
    ];
    stub.respond(&provider_body(&case, &overrides), 200);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert!(mentions(&comparison.reasons, "is_complying"));
    assert_eq!(
        comparison.local_verdict.as_deref().map(points_lossy),
        Some("MYR_MET".to_owned()),
        "the local verdict is the local engine's own"
    );
    let serialised = pyjson::dumps(&comparison.to_json(), Layout::Canonical);
    assert!(!serialised.replace("MYR_MET", "").contains("COMPLYING"));
}

#[test]
fn the_local_side_runs_without_a_client_and_says_so() {
    let comparison =
        trial::evaluate(&case("D7A-1-met-first-real-year"), None).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::NotRun);
    assert!(equals(
        money(&comparison.local, "statutory_myr"),
        "21874.92"
    ));
    assert!(comparison.upstream.is_empty());
}

#[test]
fn an_evaluation_outcome_is_not_an_accounting_state() {
    let comparison =
        trial::evaluate(&case("D7A-1-met-first-real-year"), None).expect("a comparison");
    let payload = comparison.to_json();
    assert_eq!(payload.at("evaluation"), &Json::str("NOT_RUN"));
    assert!(
        payload
            .at("boundary")
            .as_str()
            .expect("a str")
            .contains("approves nothing")
    );
    let evaluation = payload.at("evaluation").as_str().expect("a str");
    assert!(!["PASS", "REVIEW", "BLOCKED", "READY", "NOT_READY"].contains(&evaluation));
}

#[test]
fn the_case_file_falls_back_to_the_source_checkout_path() {
    // Python searches `_FIXTURE_DIRS` (the source checkout, then the wheel)
    // at run time. The Rust trial embeds the source checkout's case file at
    // build time, so the nearest contract is that the packaged cases are
    // that file's cases, field for field.
    let text = std::fs::read_to_string(fixture_path()).expect("the case file reads");
    let from_file = trial::load_cases(Some(&text)).expect("the file's cases load");
    let packaged = trial::load_cases(None).expect("the packaged cases load");
    assert_eq!(packaged.len(), from_file.len());
    for (a, b) in packaged.iter().zip(&from_file) {
        assert_eq!(a.case_id, b.case_id);
        assert_eq!(a.period_uri, b.period_uri);
        assert_eq!(a.expect_local, b.expect_local);
        assert_eq!(a.expect_evaluation, b.expect_evaluation);
        assert_eq!(a.expected, b.expected);
    }
}

#[test]
fn the_local_side_carries_only_the_figures_the_engine_produced() {
    // Python monkeypatches `minimum_yearly_repayment` to return a figureless
    // UNKNOWN result. Rust has no seam there, so the nearest contract uses a
    // real figureless engine result (D7A-8, refused on a nil term): the
    // trial's verdict, figures and last reason are exactly the engine's.
    let case = case("D7A-8-nil-remaining-term-refused");
    let engine_decimal = |value: &Decimal| {
        div7a_loan_review::decimal::Decimal::parse(&value.to_sci_string()).expect("reads back")
    };
    let year_of = |label: &str| parse_year(&Raw::Str(label.to_owned()), "case").expect("a year");
    let made = year_of("2018-19");
    let total = case
        .repayments
        .iter()
        .try_fold(Decimal::from_int(0), |sum, item| sum.add(&item.amount))
        .expect("a sum");
    let facts = MyrFacts {
        loan_id: case.case_id.clone(),
        year_of_income: Some(year_of(case.local_year().expect("a mapped year"))),
        amalgamated_loan_unpaid_at_end_of_previous_year: Some(engine_decimal(
            &case.amalgamated_base,
        )),
        remaining_term_years: Some(engine_decimal(&Decimal::from_int(
            case.remaining_term_years,
        ))),
        payments_applied_during_the_year: Some(engine_decimal(&total)),
        gate_result: Some(GateSummary {
            verdict: "COMPLYING".into(),
            benchmark_year_used: made.label(),
            reasons: Vec::new(),
            caveats: Vec::new(),
        }),
        year_loan_made: Some(made),
    };
    let engine = minimum_yearly_repayment(&facts, None, None).expect("the engine answers");

    let (verdict, values, local_reasons) = trial::run_local(&case).expect("the local side runs");
    assert_eq!(verdict, engine.verdict);
    let produced: Vec<&str> = [
        ("statutory_myr", engine.myr_required.is_some()),
        ("total_repayments", engine.payments_applied.is_some()),
        ("shortfall", engine.shortfall.is_some()),
        (
            "benchmark_rate",
            engine.benchmark.as_ref().is_some_and(|b| b.rate.is_some()),
        ),
    ]
    .into_iter()
    .filter(|(_, present)| *present)
    .map(|(name, _)| name)
    .collect();
    let carried: Vec<String> = values.iter().map(|(k, _)| points_lossy(k)).collect();
    assert_eq!(carried, produced);
    assert!(money(&values, "statutory_myr").is_none());
    assert_eq!(
        local_reasons.last().map(|r| points_lossy(r)),
        engine.reasons.last().cloned()
    );
}

#[test]
fn only_the_figures_the_provider_returned_are_extracted() {
    let result = Json::obj([
        ("statutory_myr", Json::str("1.00")),
        ("is_complying", Json::Bool(true)),
    ]);
    let values = trial::extract_upstream(&result).expect("figures");
    assert_eq!(values, vec![(pts("statutory_myr"), dec("1.00"))]);
}

#[test]
fn a_provider_answer_without_a_figure_is_named_for_why() {
    let case = case("D7A-1-met-first-real-year");
    let (verdict, values, local_reasons) = trial::run_local(&case).expect("the local side runs");
    for (status, expected) in [
        (Status::UpstreamNotFound, Evaluation::UnsupportedPeriod),
        (Status::UpstreamRefused, Evaluation::ScopeMismatch),
        (Status::UpstreamRejected, Evaluation::ContractFailure),
    ] {
        let comparison = trial::evaluate_outcome(
            &case,
            &verdict,
            &values,
            &local_reasons,
            &fabricated(status),
        )
        .expect("a comparison");
        assert_eq!(comparison.evaluation, expected, "{status:?}");
        assert!(
            reasons(&comparison).contains(&"fabricated finding".to_owned()),
            "{status:?}"
        );
        assert!(comparison.upstream.is_empty(), "{status:?}");
    }
}

#[test]
fn a_refusal_without_a_class_says_so() {
    let case = case("D7A-1-met-first-real-year");
    let comparison = trial::evaluate_outcome(
        &case,
        "MYR_MET",
        &Money::new(),
        &[],
        &fabricated(Status::UpstreamRefused),
    )
    .expect("a comparison");
    assert_eq!(
        reasons(&comparison)[0],
        "the provider refused this case: no class given"
    );
}

#[test]
fn a_computed_answer_without_a_compliance_label_records_none() {
    let stub = Stub::start();
    let case = case("D7A-1-met-first-real-year");
    let mut body = provider_body(&case, &[]);
    remove(&mut body, "is_complying");
    stub.respond(&body, 200);
    let comparison =
        with_client(&stub, |client| trial::evaluate(&case, Some(client))).expect("a comparison");
    assert!(!mentions(&comparison.reasons, "is_complying"));
}
