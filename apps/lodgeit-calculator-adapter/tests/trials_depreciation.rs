//! `tests/test_trials_depreciation.py`: the accounting depreciation trial,
//! where the movement closes or it is refused.
//!
//! The provider is a loopback HTTP stub started inside each test and reached
//! through an `allow_loopback` config, or a fabricated `Outcome` passed to
//! `evaluate_outcome`.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use lodgeit_calculator_adapter::client::{Client, Outcome, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract;
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::pyvalue::points_lossy;
use lodgeit_calculator_adapter::trials::depreciation::{self as trial, Asset};
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

fn same(value: &Decimal, want: &str) -> bool {
    value.equals(&dec(want)).expect("comparable")
}

fn reasons(comparison: &Comparison) -> Vec<String> {
    comparison.reasons.iter().map(|r| points_lossy(r)).collect()
}

fn mentions(comparison: &Comparison, needle: &str) -> bool {
    reasons(comparison).iter().any(|r| r.contains(needle))
}

fn mentioned(lines: &[Vec<u32>], needle: &str) -> bool {
    lines.iter().any(|line| points_lossy(line).contains(needle))
}

/// `ASSET`.
fn asset() -> Asset {
    Asset {
        case_id: "DEP-1-plant".to_owned(),
        cost: dec("120000.00"),
        acquisition_date: "2024-07-01".to_owned(),
        useful_life_years: 5,
        method: "prime_cost".to_owned(),
    }
}

/// `provider_range(opening=..., closing=..., depreciation=...)`.
fn provider_range(opening: &str, closing: &str, depreciation: &str) -> Json {
    Json::obj([
        ("opening_wdv", Json::str(opening)),
        ("closing_wdv", Json::str(closing)),
        ("range_dep", Json::str(depreciation)),
        ("days_in_range", Json::Int(365.into())),
        ("truncated", Json::Bool(false)),
        ("day_count", Json::str("actual/actual")),
        (
            "manifest",
            Json::obj([
                ("calculator", Json::str(trial::RANGE_CALC_URI)),
                ("period", Json::str(trial::PERIOD_URI)),
                ("rate_table_uris", Json::Array(Vec::new())),
                ("engine", Json::str("dep/1.0.0")),
                ("citation", Json::str("AASB 116")),
            ]),
        ),
        (
            "advisory",
            Json::obj([
                ("figure_type", Json::str("accounting_carrying_amount")),
                (
                    "notes",
                    Json::Array(vec![Json::str(
                        "Not a Division 40 deduction. Do not carry into a tax return.",
                    )]),
                ),
            ]),
        ),
    ])
}

fn values(pairs: &[(&str, &str)]) -> Money {
    pairs.iter().map(|(k, v)| (pts(k), dec(v))).collect()
}

/// `trial.evaluate_range(ASSET, from_date, to_date, "actual/actual", client,
/// supplied_additions=...)` against the stub.
fn evaluate_range(
    stub: &Stub,
    from_date: &str,
    to_date: &str,
    supplied_additions: Option<&Decimal>,
) -> Comparison {
    with_client(stub, |client| {
        trial::evaluate_range(
            &asset(),
            from_date,
            to_date,
            "actual/actual",
            Some(client),
            supplied_additions,
            None,
        )
    })
    .expect("a comparison")
}

// ---------------------------------------------------------------------------
// The tests, one per Python test function.

#[test]
fn the_request_requires_a_day_count_and_rejects_an_unknown_one() {
    let body = trial::range_request(&asset(), "2024-07-01", "2025-06-30", "actual/actual")
        .expect("a body");
    assert_eq!(body.at("day_count"), &Json::str("actual/actual"));
    assert_eq!(body.at("basis"), &Json::str("accounting"));
    let Json::Dec(cost) = body.at("asset").at("cost") else {
        panic!("the cost stays a Decimal")
    };
    assert!(same(cost, "120000.00"));
    let error = trial::range_request(&asset(), "2024-07-01", "2025-06-30", "actual/360")
        .expect_err("refused");
    assert_eq!(error.class, "ValueError");
}

#[test]
fn a_closing_movement_is_a_match() {
    let stub = Stub::start();
    stub.respond(&provider_range("120000.00", "96000.00", "24000.00"), 200);
    let comparison = evaluate_range(&stub, "2024-07-01", "2025-06-30", None);
    assert_eq!(comparison.evaluation, Evaluation::Match);
    assert!(mentions(&comparison, "Division 40"));
    // The stub saw one POST to the snapshot's route.
    assert_eq!(stub.request_count(), 1);
    let state = stub.state.lock().unwrap();
    assert_eq!(state.requests[0].method, "POST");
    assert!(!state.requests[0].path.is_empty());
}

#[test]
fn a_movement_that_does_not_close_is_refused_and_no_addition_is_invented() {
    // The provider's published pre-acquisition case: the cost enters inside
    // the window, no cost_additions field is emitted, and the three figures
    // do not reconcile.
    let stub = Stub::start();
    stub.respond(&provider_range("0.00", "108000.00", "12000.00"), 200);
    let comparison = evaluate_range(&stub, "2024-01-01", "2024-12-31", None);
    assert_eq!(comparison.evaluation, Evaluation::ContractFailure);
    assert!(mentions(&comparison, "does not close"));
    assert!(mentions(&comparison, "pre-acquisition limitation"));
    assert!(money(&comparison.upstream, "cost_additions").is_none());
}

#[test]
fn a_separately_evidenced_addition_closes_it_and_is_named() {
    let stub = Stub::start();
    stub.respond(&provider_range("0.00", "108000.00", "12000.00"), 200);
    let supplied = dec("120000.00");
    let comparison = evaluate_range(&stub, "2024-01-01", "2024-12-31", Some(&supplied));
    assert_eq!(comparison.evaluation, Evaluation::Match);
    assert!(mentions(&comparison, "supplied by the caller"));
}

#[test]
fn reconcile_refuses_rather_than_deriving_the_gap() {
    let figures = values(&[
        ("opening_wdv", "0.00"),
        ("closing_wdv", "108000.00"),
        ("range_dep", "12000.00"),
    ]);
    let tolerance = dec("0.01");
    let (closes, found) = trial::reconcile(&figures, None, &tolerance).expect("a reconciliation");
    assert!(!closes);
    assert!(mentioned(&found, "No addition has been invented"));
    let supplied = dec("120000.00");
    let (closes, _) =
        trial::reconcile(&figures, Some(&supplied), &tolerance).expect("a reconciliation");
    assert!(closes);
}

#[test]
fn a_missing_figure_cannot_be_reconciled_around() {
    let (closes, found) = trial::reconcile(&values(&[("opening_wdv", "1.00")]), None, &dec("0.01"))
        .expect("a reconciliation");
    assert!(!closes);
    assert!(mentioned(&found, "missing"));
}

#[test]
fn the_pre_acquisition_window_is_recognised_before_the_call() {
    assert!(trial::spans_acquisition(&asset(), "2024-01-01"));
    assert!(!trial::spans_acquisition(&asset(), "2024-07-01"));
    assert!(!trial::spans_acquisition(&asset(), "2025-01-01"));
}

#[test]
fn consecutive_windows_telescope_and_cover_every_day() {
    let pairs = |list: &[(&str, &str)]| -> Vec<(String, String)> {
        list.iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect()
    };
    let windows =
        trial::consecutive_windows("2024-07-01", "2025-06-30", &["2024-12-31".to_owned()])
            .expect("windows");
    assert_eq!(
        windows,
        pairs(&[("2024-07-01", "2024-12-31"), ("2025-01-01", "2025-06-30")])
    );
    // A leap-year boundary: 29 February exists and the split is still
    // contiguous.
    let leap = trial::consecutive_windows("2024-01-01", "2024-12-31", &["2024-02-29".to_owned()])
        .expect("windows");
    assert_eq!(
        leap,
        pairs(&[("2024-01-01", "2024-02-29"), ("2024-03-01", "2024-12-31")])
    );
}

#[test]
fn consecutive_ranges_sum_to_the_whole() {
    // Two halves against the year: the closing of one is the opening of the
    // next.
    let stub = Stub::start();
    let first = provider_range("120000.00", "107934.43", "12065.57");
    let second = provider_range("107934.43", "96000.00", "11934.43");
    let windows =
        trial::consecutive_windows("2024-07-01", "2025-06-30", &["2024-12-31".to_owned()])
            .expect("windows");
    let results: Vec<Comparison> = with_client(&stub, |client| {
        [first, second]
            .iter()
            .zip(&windows)
            .map(|(payload, (start, end))| {
                stub.respond(payload, 200);
                trial::evaluate_range(
                    &asset(),
                    start,
                    end,
                    "actual/actual",
                    Some(&mut *client),
                    None,
                    None,
                )
                .expect("a comparison")
            })
            .collect()
    });
    assert!(
        results
            .iter()
            .all(|item| item.evaluation == Evaluation::Match)
    );
    let total = results
        .iter()
        .map(|item| money(&item.upstream, "range_dep").expect("a figure"))
        .try_fold(Decimal::from_int(0), |sum, value| sum.add(value))
        .expect("a sum");
    assert!(same(&total, "24000.00"));
    let closing = money(&results[0].upstream, "closing_wdv").expect("a figure");
    let opening = money(&results[1].upstream, "opening_wdv").expect("a figure");
    assert!(closing.equals(opening).expect("comparable"));
}

#[test]
fn life_end_leaves_nothing() {
    let stub = Stub::start();
    stub.respond(&provider_range("24000.00", "0.00", "24000.00"), 200);
    let comparison = evaluate_range(&stub, "2028-07-01", "2029-06-30", None);
    assert_eq!(comparison.evaluation, Evaluation::Match);
    assert!(same(
        money(&comparison.upstream, "closing_wdv").expect("a figure"),
        "0.00"
    ));
}

#[test]
fn a_pooled_asset_refusal_stays_a_scope_mismatch() {
    let stub = Stub::start();
    stub.respond(
        &Json::obj([
            ("refusal_class", Json::str("pool_asset_out_of_t6_scope")),
            ("reason", Json::str("pooled")),
        ]),
        400,
    );
    let comparison = evaluate_range(&stub, "2024-07-01", "2025-06-30", None);
    assert_eq!(comparison.evaluation, Evaluation::ScopeMismatch);
    assert!(comparison.upstream.is_empty());
}

#[test]
fn without_a_client_the_provider_is_not_called() {
    let comparison = trial::evaluate_range(
        &asset(),
        "2024-07-01",
        "2025-06-30",
        "actual/actual",
        None,
        None,
        None,
    )
    .expect("a comparison");
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
        (Status::ContractFailure, Evaluation::ContractFailure),
    ] {
        // `Outcome(status=status, findings=("fabricated finding",))`.
        let outcome = Outcome {
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
        };
        let comparison =
            trial::evaluate_outcome(&asset(), "2024-07-01", pts("DEP-1"), &outcome, None)
                .expect("a comparison");
        assert_eq!(comparison.evaluation, expected, "{status:?}");
        assert_eq!(
            reasons(&comparison),
            vec!["fabricated finding".to_owned()],
            "{status:?}"
        );
        assert_eq!(
            comparison.upstream_status,
            Some(pts(status.as_str())),
            "{status:?}"
        );
    }
}

#[test]
fn a_movement_after_acquisition_that_does_not_close_names_no_limitation() {
    let stub = Stub::start();
    stub.respond(&provider_range("120000.00", "90000.00", "24000.00"), 200);
    let comparison = evaluate_range(&stub, "2024-07-01", "2025-06-30", None);
    assert_eq!(comparison.evaluation, Evaluation::ContractFailure);
    assert!(mentions(&comparison, "does not close"));
    assert!(!mentions(&comparison, "pre-acquisition"));
}
