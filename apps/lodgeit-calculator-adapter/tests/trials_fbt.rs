//! `tests/test_trials_fbt.py`: the FBT trial, one category value in, and
//! the two ways it could go wrong.
//!
//! The provider is a loopback HTTP stub started inside each test and reached
//! through an `allow_loopback` config, or a fabricated `Outcome` passed to
//! `evaluate_outcome`. The Rust trial links the in-tree
//! `australian-tax-calculators`, whose worksheet also returns
//! `return_item_14a`, `return_item_14b`, `return_item_15` and
//! `return_item_16` in `local`; only the fields the Python tests read are
//! asserted.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use lodgeit_calculator_adapter::client::{Client, Outcome, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract;
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::errors::CONTRACT;
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::pyvalue::points_lossy;
use lodgeit_calculator_adapter::trials::fbt::{self as trial, CarBenefit};
use lodgeit_calculator_adapter::trials::{Comparison, Evaluation, Money};
use num_bigint::BigInt;

const CLASSIFICATION_REQUIRED: &str = "lodgeitadapter.trials.fbt.ClassificationRequired";

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

fn same(value: &Decimal, want: &str) -> bool {
    value.equals(&dec(want)).expect("comparable")
}

fn reasons(comparison: &Comparison) -> Vec<String> {
    comparison.reasons.iter().map(|r| points_lossy(r)).collect()
}

fn mentions(comparison: &Comparison, needle: &str) -> bool {
    reasons(comparison).iter().any(|r| r.contains(needle))
}

/// `BENEFIT`.
fn benefit() -> CarBenefit {
    CarBenefit {
        case_id: "FBT-1-type-2-car".to_owned(),
        base_value: dec("40000.00"),
        days_available: 365,
        accessories: dec("0.00"),
        employee_contribution: dec("0.00"),
        fbt_type: "Type 2".to_owned(),
        period_uri: trial::PERIOD_URI.to_owned(),
    }
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

/// `provider_body(taxable_value="8000.00", **overrides)`.
fn provider_body(overrides: &[(&str, Json)]) -> Json {
    let mut body = Json::obj([
        ("taxable_value", Json::str("8000.00")),
        ("gross_up_factor", Json::str("1.8868")),
        ("grossed_up_taxable_value", Json::str("15094.40")),
        ("fbt_payable", Json::str("7094.37")),
        (
            "manifest",
            Json::obj([
                ("calculator", Json::str(trial::CALC_URI)),
                ("period", Json::str(trial::PERIOD_URI)),
                ("rate_table_uris", Json::Array(Vec::new())),
                ("engine", Json::str("fbt/1.0.0")),
                ("citation", Json::str("FBTAA s 9")),
            ]),
        ),
        (
            "advisory",
            Json::obj([
                ("figure_type", Json::str("fbt_taxable_value")),
                ("notes", Json::Array(vec![Json::str("Not advice.")])),
            ]),
        ),
    ]);
    for (name, value) in overrides {
        set(&mut body, name, value.clone());
    }
    body
}

/// `Outcome(status=status, findings=("fabricated finding",), values=...)`.
fn fabricated(status: Status, values: Money) -> Outcome {
    Outcome {
        status,
        calculator: None,
        period: None,
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: None,
        result: Json::Null,
        values,
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

// ---------------------------------------------------------------------------
// The tests, one per Python test function.

#[test]
fn only_the_category_taxable_value_crosses_into_the_worksheet() {
    let none = Money::new();
    let (one, two) =
        trial::prepare_aggregate_input(&dec("8000.00"), "Type 2", "taxable_value", &none)
            .expect("arguments");
    assert!(same(&one, "0.00") && same(&two, "8000.00"));
    let (one, two) =
        trial::prepare_aggregate_input(&dec("8000.00"), "Type 1", "taxable_value", &none)
            .expect("arguments");
    assert!(same(&one, "8000.00") && same(&two, "0.00"));
}

#[test]
fn an_already_grossed_up_figure_is_refused_at_the_door() {
    for field in ["grossed_up_taxable_value", "fbt_payable"] {
        let error =
            trial::prepare_aggregate_input(&dec("15094.40"), "Type 2", field, &Money::new())
                .expect_err("refused");
        assert_eq!(error.class, CONTRACT, "{field}");
        assert!(
            points_lossy(&error.text).contains("already grossed up"),
            "{field}"
        );
    }
}

#[test]
fn double_gross_up_is_what_the_refusal_prevents() {
    // The arithmetic the guard exists to stop, shown once so it is not
    // abstract.
    let none = Money::new();
    let arguments =
        trial::prepare_aggregate_input(&dec("8000.00"), "Type 2", "taxable_value", &none)
            .expect("arguments");
    let local = trial::run_local_aggregate(&arguments, 2026).expect("a worksheet");
    let correct = money(&local, "type_two_grossed_up")
        .expect("a gross-up")
        .clone();
    assert!(same(&correct, "15094.40"));
    let local =
        trial::run_local_aggregate(&(dec("0.00"), correct.clone()), 2026).expect("a worksheet");
    let doubled = money(&local, "type_two_grossed_up").expect("a gross-up");
    assert!(same(doubled, "28480.11"));
    let floor = correct.mul(&dec("1.8")).expect("a product");
    assert!(doubled.gt(&floor).expect("comparable"));
}

#[test]
fn an_unknown_classification_refuses_before_the_call() {
    let unknown = CarBenefit {
        fbt_type: "unknown".to_owned(),
        ..benefit()
    };
    let error = unknown.request().expect_err("refused");
    assert_eq!(error.class, CLASSIFICATION_REQUIRED);
    let error =
        trial::prepare_aggregate_input(&dec("8000.00"), "unknown", "taxable_value", &Money::new())
            .expect_err("refused");
    assert_eq!(error.class, CLASSIFICATION_REQUIRED);
    let comparison = trial::evaluate(&unknown, None).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::ScopeMismatch);
}

#[test]
fn a_period_with_no_local_counterpart_is_unsupported() {
    let other = CarBenefit {
        period_uri: "urn:sbrm:period:fbt:fy2025".to_owned(),
        ..benefit()
    };
    let comparison = trial::evaluate(&other, None).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::UnsupportedPeriod);
}

#[test]
fn the_join_carries_the_category_value_and_agrees_on_the_gross_up() {
    let stub = Stub::start();
    stub.respond(&provider_body(&[]), 200);
    let comparison = with_client(&stub, |client| trial::evaluate(&benefit(), Some(client)))
        .expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::Match);
    assert!(same(
        money(&comparison.upstream, "taxable_value").expect("a figure"),
        "8000.00"
    ));
    assert!(same(
        money(&comparison.local, "type_two_grossed_up").expect("a figure"),
        "15094.40"
    ));
    assert!(mentions(&comparison, "only taxable_value crossed"));
    // The stub saw one POST to the snapshot's route.
    assert_eq!(stub.request_count(), 1);
    let state = stub.state.lock().unwrap();
    assert_eq!(state.requests[0].method, "POST");
    assert!(!state.requests[0].path.is_empty());
}

#[test]
fn a_gross_up_rate_difference_shows_up_as_a_difference() {
    let stub = Stub::start();
    // The Type 1 factor.
    let overrides = [("grossed_up_taxable_value", Json::str("16641.60"))];
    stub.respond(&provider_body(&overrides), 200);
    let comparison = with_client(&stub, |client| trial::evaluate(&benefit(), Some(client)))
        .expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::NumericDifference);
    assert!(mentions(&comparison, "gross-up rate"));
}

#[test]
fn a_response_without_a_taxable_value_produces_nothing() {
    let stub = Stub::start();
    let mut body = provider_body(&[]);
    remove(&mut body, "taxable_value");
    stub.respond(&body, 200);
    let comparison = with_client(&stub, |client| trial::evaluate(&benefit(), Some(client)))
        .expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::ContractFailure);
    assert!(comparison.local.is_empty());
}

#[test]
fn a_provider_refusal_does_not_reach_the_worksheet() {
    let stub = Stub::start();
    stub.respond(
        &Json::obj([("refusal_class", Json::str("out_of_scope_benefit"))]),
        400,
    );
    let comparison = with_client(&stub, |client| trial::evaluate(&benefit(), Some(client)))
        .expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::ScopeMismatch);
    assert!(comparison.local.is_empty());
}

#[test]
fn without_a_client_the_provider_is_not_called() {
    let comparison = trial::evaluate(&benefit(), None).expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::NotRun);
    assert_eq!(
        reasons(&comparison),
        vec!["No client supplied; the provider was not called.".to_owned()]
    );
}

#[test]
fn a_call_that_produced_no_figure_is_named_for_why() {
    for (status, expected) in [
        (Status::UpstreamNotFound, Evaluation::UnsupportedPeriod),
        (Status::UpstreamUnavailable, Evaluation::UpstreamUnavailable),
        (Status::UpstreamThrottled, Evaluation::UpstreamUnavailable),
        (Status::RefusedToSend, Evaluation::UpstreamUnavailable),
        (Status::UpstreamRejected, Evaluation::ContractFailure),
    ] {
        let comparison = trial::evaluate_outcome(&benefit(), &fabricated(status, Money::new()))
            .expect("a comparison");
        assert_eq!(comparison.evaluation, expected, "{status:?}");
        assert_eq!(
            reasons(&comparison),
            vec!["fabricated finding".to_owned()],
            "{status:?}"
        );
    }
}

#[test]
fn a_computed_outcome_without_a_taxable_value_reaches_no_worksheet() {
    let comparison =
        trial::evaluate_outcome(&benefit(), &fabricated(Status::Computed, Money::new()))
            .expect("a comparison");
    assert_eq!(comparison.evaluation, Evaluation::ContractFailure);
    assert!(reasons(&comparison)[0].starts_with("the response carries no taxable_value"));
}

#[test]
fn a_figure_only_the_provider_returned_is_named() {
    // Python monkeypatches `run_local_aggregate` to return no figures, so the
    // provider's grossed-up value is the only side present. Rust has no seam
    // there and the real worksheet always returns both derived figures, so
    // the nearest contract is the same one-sided naming from the other
    // side: the provider omits fbt_payable, and the worksheet's fbt_estimate
    // is named as a figure only the local side produced.
    let values = vec![
        (pts("taxable_value"), dec("8000.00")),
        (pts("grossed_up_taxable_value"), dec("15094.40")),
    ];
    let comparison = trial::evaluate_outcome(&benefit(), &fabricated(Status::Computed, values))
        .expect("a comparison");
    let reasons = reasons(&comparison);
    assert!(
        reasons.contains(
            &"fbt_estimate: the local worksheet produced a figure and the provider returned no \
              fbt_payable"
                .to_owned()
        ),
        "{reasons:?}"
    );
    assert!(
        !reasons
            .iter()
            .any(|r| r.contains("grossed_up_taxable_value: the provider returned"))
    );
    assert_eq!(comparison.evaluation, Evaluation::ScopeMismatch);
}
