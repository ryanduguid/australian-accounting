//! `tests/test_client_outcomes.py`: every failure keeps its own name, and
//! none of them becomes a figure.
//!
//! Python's `stub` fixture is a loopback `http.server`; here it is a
//! loopback `TcpListener` the test starts itself, reached through the real
//! dialler (`dial::Net`) with `allow_loopback` set.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use lodgeit_calculator_adapter::client::{Client, Outcome, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::errors::{DISALLOWED_TARGET, TRANSPORT};
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout, PyStr};
use lodgeit_calculator_adapter::transport::{self, Call};
use num_bigint::BigInt;

const CALC: &str = "urn:sbrm:calculator:div7a:at";
const PERIOD: &str = "urn:sbrm:period:div7a:fy2026";

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn good() -> Json {
    Json::obj([
        ("statutory_myr", Json::str("21874.92")),
        ("shortfall", Json::str("0.00")),
        ("total_repayments", Json::str("22000.00")),
        ("benchmark_rate", Json::str("0.0837")),
        (
            "manifest",
            Json::obj([
                ("calculator", Json::str(CALC)),
                ("period", Json::str(PERIOD)),
                ("rate_table_uris", Json::Array(Vec::new())),
                ("engine", Json::str("x")),
                ("citation", Json::str("s 109E")),
            ]),
        ),
        (
            "advisory",
            Json::obj([
                ("figure_type", Json::str("div7a")),
                ("notes", Json::Array(vec![Json::str("not advice")])),
            ]),
        ),
    ])
}

/// `{key: value for key, value in payload.items() if key != name}`.
fn without(payload: Json, name: &str) -> Json {
    match payload {
        Json::Object(mut pairs) => {
            pairs.retain(|(key, _)| key.as_str() != Some(name));
            Json::Object(pairs)
        }
        other => other,
    }
}

/// `{**payload, name: value}`.
fn with(payload: Json, name: &str, value: Json) -> Json {
    match payload {
        Json::Object(mut pairs) => {
            match pairs.iter_mut().find(|(key, _)| key.as_str() == Some(name)) {
                Some(slot) => slot.1 = value,
                None => pairs.push((PyStr::Text(name.into()), value)),
            }
            Json::Object(pairs)
        }
        other => other,
    }
}

/// What the stub saw of one request.
struct Seen {
    method: String,
    path: String,
}

/// What the stub should do next, and what it saw: conftest's `StubState`.
struct State {
    status: u16,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    requests: Vec<Seen>,
    delay: Duration,
}

/// conftest's `stub` fixture: a loopback HTTP server on an ephemeral port.
struct Stub {
    base_url: String,
    state: Arc<Mutex<State>>,
}

impl Stub {
    fn start() -> Stub {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let state = Arc::new(Mutex::new(State {
            status: 200,
            body: b"{}".to_vec(),
            headers: vec![("Content-Type".into(), "application/json".into())],
            requests: Vec::new(),
            delay: Duration::ZERO,
        }));
        let shared = Arc::clone(&state);
        // The thread ends with the test process; each connection gets its
        // own thread so a delayed reply never holds up the next request.
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let state = Arc::clone(&shared);
                thread::spawn(move || serve(stream, &state));
            }
        });
        Stub { base_url, state }
    }

    /// `state.respond(payload, status)`.
    fn respond(&self, payload: &Json, status: u16) {
        self.respond_raw(pyjson::dumps(payload, Layout::Canonical).as_bytes(), status);
    }

    /// `state.respond(None, status, raw=raw)`.
    fn respond_raw(&self, raw: &[u8], status: u16) {
        let mut state = self.state.lock().unwrap();
        state.status = status;
        state.body = raw.to_vec();
    }

    fn requests(&self) -> usize {
        self.state.lock().unwrap().requests.len()
    }

    /// conftest's `stub_config`.
    fn config(&self) -> AdapterConfig {
        config(&self.base_url, |_| {})
    }
}

fn serve(stream: TcpStream, state: &Mutex<State>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return;
    }
    let mut words = line.split_whitespace();
    let method = words.next().unwrap_or("").to_owned();
    let path = words.next().unwrap_or("").to_owned();
    let mut length = 0;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).unwrap_or(0) == 0 {
            return;
        }
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0; length];
    if reader.read_exact(&mut body).is_err() {
        return;
    }
    let (status, headers, reply, delay) = {
        let mut state = state.lock().unwrap();
        state.requests.push(Seen { method, path });
        (
            state.status,
            state.headers.clone(),
            state.body.clone(),
            state.delay,
        )
    };
    thread::sleep(delay);
    let mut head = format!("HTTP/1.0 {status} Stub\r\n");
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n\r\n", reply.len()));
    let mut stream = stream;
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&reply);
}

/// `AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True,
/// max_attempts=1, ...)`, with `change` applying the test's own fields.
fn config(base_url: &str, change: impl FnOnce(&mut AdapterConfig)) -> AdapterConfig {
    let mut config = AdapterConfig {
        enabled: true,
        base_url: Some(base_url.into()),
        allow_loopback: true,
        max_attempts: BigInt::from(1),
        ..AdapterConfig::default()
    };
    change(&mut config);
    config.checked().unwrap()
}

fn calculators() -> Contract {
    contract::load("lodgeit-calculators", None).unwrap()
}

fn client<'a>(
    config: &'a AdapterConfig,
    contract: &'a Contract,
    net: &'a mut Net,
    sleep: &'a mut fn(f64) -> Result<(), lodgeit_calculator_adapter::pyops::Exc>,
) -> Client<'a> {
    Client {
        config,
        contract,
        dial: net,
        sleep,
    }
}

fn invoke(
    config: &AdapterConfig,
    contract: &Contract,
    calc: &str,
    period: &str,
    body: &Json,
) -> Outcome {
    let mut sleep: fn(f64) -> _ = dial::sleep;
    client(config, contract, &mut Net, &mut sleep)
        .invoke(&pts(calc), &pts(period), body)
        .unwrap()
}

/// `call(client)`: the div7a calculator for FY2026 with a Decimal body.
fn call(config: &AdapterConfig, contract: &Contract) -> Outcome {
    let body = Json::obj([(
        "amalgamated_base",
        Json::Dec(Decimal::parse_exact("100000.00").unwrap()),
    )]);
    invoke(config, contract, CALC, PERIOD, &body)
}

fn discover(config: &AdapterConfig, contract: &Contract) -> Outcome {
    let mut sleep: fn(f64) -> _ = dial::sleep;
    client(config, contract, &mut Net, &mut sleep)
        .discover()
        .unwrap()
}

fn value(outcome: &Outcome, name: &str) -> Option<String> {
    outcome
        .values
        .iter()
        .find(|(key, _)| text(key) == name)
        .map(|(_, decimal)| decimal.to_sci_string())
}

fn any_finding(findings: &[Vec<u32>], needle: &str) -> bool {
    findings
        .iter()
        .any(|finding| text(finding).contains(needle))
}

#[test]
fn a_complete_200_is_the_only_computed_outcome() {
    let stub = Stub::start();
    stub.respond(&good(), 200);
    let outcome = call(&stub.config(), &calculators());
    assert_eq!(outcome.status, Status::Computed);
    assert_eq!(
        value(&outcome, "statutory_myr").as_deref(),
        Some("21874.92")
    );
    assert!(outcome.manifest.truthy() && outcome.advisory.truthy());
}

#[test]
fn each_provider_status_keeps_its_own_outcome() {
    let stub = Stub::start();
    let contract = calculators();
    for (status, expected) in [
        (400, Status::UpstreamRefused),
        (422, Status::UpstreamRejected),
        (404, Status::UpstreamNotFound),
        (429, Status::UpstreamThrottled),
        (500, Status::UpstreamUnavailable),
        (503, Status::UpstreamUnavailable),
        (418, Status::ContractFailure),
    ] {
        let refusal = Json::obj([
            ("refusal_class", Json::str("out_of_scope")),
            ("detail", Json::str("because")),
        ]);
        stub.respond(&refusal, status);
        let outcome = call(&stub.config(), &contract);
        assert_eq!(outcome.status, expected, "{status}");
        assert!(outcome.values.is_empty(), "a non-200 must carry no figure");
        assert_eq!(outcome.http_status, Some(status));
    }
}

#[test]
fn a_provider_refusal_keeps_the_providers_own_class_and_body() {
    let stub = Stub::start();
    let refusal = Json::obj([
        ("refusal_class", Json::str("pool_asset_out_of_t6_scope")),
        ("reason", Json::str("pooled")),
    ]);
    stub.respond(&refusal, 400);
    let outcome = call(&stub.config(), &calculators());
    assert_eq!(
        outcome.upstream_refusal_class,
        Json::str("pool_asset_out_of_t6_scope")
    );
    assert_eq!(
        outcome.raw_response.get("reason"),
        Some(&Json::str("pooled"))
    );
}

#[test]
fn a_200_without_an_advisory_is_a_contract_failure_not_a_result() {
    let stub = Stub::start();
    stub.respond(&without(good(), "advisory"), 200);
    let outcome = call(&stub.config(), &calculators());
    assert_eq!(outcome.status, Status::ContractFailure);
    assert!(outcome.values.is_empty());
    assert!(any_finding(&outcome.findings, "advisory"));
}

#[test]
fn a_200_without_a_manifest_is_a_contract_failure() {
    let stub = Stub::start();
    stub.respond(&without(good(), "manifest"), 200);
    let outcome = call(&stub.config(), &calculators());
    assert_eq!(outcome.status, Status::ContractFailure);
    assert!(any_finding(&outcome.findings, "manifest"));
}

#[test]
fn a_200_missing_an_expected_decimal_field_is_a_contract_failure() {
    let stub = Stub::start();
    stub.respond(&without(good(), "statutory_myr"), 200);
    let outcome = call(&stub.config(), &calculators());
    assert_eq!(outcome.status, Status::ContractFailure);
    assert!(any_finding(&outcome.findings, "statutory_myr"));
}

#[test]
fn an_unparseable_body_is_a_contract_failure() {
    let stub = Stub::start();
    stub.respond_raw(b"<html>not json</html>", 200);
    let outcome = call(&stub.config(), &calculators());
    assert_eq!(outcome.status, Status::ContractFailure);
    assert!(outcome.values.is_empty());
}

#[test]
fn a_failure_status_keeps_its_outcome_when_its_body_is_not_json() {
    // A complete 503 with an HTML body came back as CONTRACT_FAILURE because
    // the body was decoded before the status was read.
    let stub = Stub::start();
    let contract = calculators();
    for (status, expected) in [
        (503, Status::UpstreamUnavailable),
        (500, Status::UpstreamUnavailable),
        (429, Status::UpstreamThrottled),
        (404, Status::UpstreamNotFound),
        (422, Status::UpstreamRejected),
        (400, Status::UpstreamRefused),
        (403, Status::UpstreamRefused),
        (418, Status::ContractFailure),
    ] {
        for body in [&b"<html>Service down</html>"[..], b"\xff\xfe busy"] {
            stub.respond_raw(body, status);
            let outcome = call(&stub.config(), &contract);
            assert_eq!(outcome.status, expected, "{status} {body:?}");
            assert_eq!(outcome.http_status, Some(status));
            assert!(outcome.values.is_empty() && outcome.result == Json::Null);
            assert_eq!(outcome.upstream_refusal_class, Json::Null);
            assert_eq!(
                outcome.raw_response,
                Json::str(&String::from_utf8_lossy(body))
            );
        }
    }
}

#[test]
fn a_large_json_failure_body_is_kept_whole_with_its_outcome() {
    // The evidence record keeps the provider's JSON unaltered, and the
    // transport's response ceiling is what bounds it. Only a body that is
    // not JSON is cut to a 2,048-byte excerpt.
    let stub = Stub::start();
    let contract = calculators();
    for (status, expected, refusal) in [
        (503, Status::UpstreamUnavailable, None),
        (400, Status::UpstreamRefused, Some("out_of_scope")),
    ] {
        let mut payload = Json::obj([("detail", Json::str(&"x".repeat(3000)))]);
        if let Some(refusal) = refusal {
            payload = with(payload, "refusal_class", Json::str(refusal));
        }
        stub.respond(&payload, status);
        let outcome = call(&stub.config(), &contract);
        assert_eq!(outcome.status, expected);
        assert_eq!(
            outcome.upstream_refusal_class,
            refusal.map_or(Json::Null, Json::str)
        );
        assert_eq!(outcome.raw_response, payload);
        assert!(outcome.values.is_empty() && outcome.result == Json::Null);
    }
}

#[test]
fn a_period_the_snapshot_does_not_record_never_reaches_the_network() {
    let stub = Stub::start();
    stub.respond(&good(), 200);
    let outcome = invoke(
        &stub.config(),
        &calculators(),
        CALC,
        "urn:sbrm:period:div7a:fy2024",
        &Json::Object(Vec::new()),
    );
    assert_eq!(outcome.status, Status::ContractFailure);
    assert_eq!(stub.requests(), 0, "nothing was sent");
}

#[test]
fn a_calculator_the_snapshot_does_not_record_is_refused_before_sending() {
    let stub = Stub::start();
    let outcome = invoke(
        &stub.config(),
        &calculators(),
        "urn:sbrm:calculator:fbt:board",
        "urn:sbrm:period:fbt:fy2026",
        &Json::Object(Vec::new()),
    );
    assert_eq!(outcome.status, Status::ContractFailure);
    assert!(any_finding(&outcome.findings, "reviewed snapshot"));
    assert_eq!(
        stub.requests(),
        0,
        "an unreviewed calculator is never called"
    );
}

#[test]
fn an_optional_field_may_be_absent_but_a_required_one_may_not() {
    let stub = Stub::start();
    let contract = calculators();
    stub.respond(&good(), 200); // carries no deemed_dividend or interest_accrued
    let outcome = call(&stub.config(), &contract);
    assert_eq!(outcome.status, Status::Computed);
    assert_eq!(value(&outcome, "deemed_dividend"), None);

    stub.respond(&with(good(), "deemed_dividend", Json::str("123.45")), 200);
    let outcome = call(&stub.config(), &contract);
    assert_eq!(
        value(&outcome, "deemed_dividend").as_deref(),
        Some("123.45")
    );

    stub.respond(&with(good(), "shortfall", Json::str("not a number")), 200);
    let outcome = call(&stub.config(), &contract);
    assert_eq!(outcome.status, Status::ContractFailure);
}

#[test]
fn a_redirect_is_refused_rather_than_followed() {
    let stub = Stub::start();
    {
        let mut state = stub.state.lock().unwrap();
        state.status = 302;
        state.headers = vec![
            (
                "Location".into(),
                "https://evil.example/v1/calculators".into(),
            ),
            ("Content-Type".into(), "application/json".into()),
        ];
        state.body = b"{}".to_vec();
    }
    let outcome = call(&stub.config(), &calculators());
    assert_eq!(outcome.status, Status::RefusedToSend);
    assert!(any_finding(&outcome.findings, "redirect"));
}

#[test]
fn an_oversized_response_is_abandoned() {
    let stub = Stub::start();
    stub.respond_raw(&[b'x'; 5000], 200);
    let config = config(&stub.base_url, |c| {
        c.max_response_bytes = BigInt::from(1000);
    });
    let outcome = call(&config, &calculators());
    assert_eq!(outcome.status, Status::UpstreamUnavailable);
    assert!(any_finding(&outcome.findings, "ceiling"));
}

#[test]
fn a_timeout_is_never_a_number() {
    let stub = Stub::start();
    stub.respond(&good(), 200);
    stub.state.lock().unwrap().delay = Duration::from_millis(600);
    let config = config(&stub.base_url, |c| c.read_timeout = 0.05);
    let outcome = call(&config, &calculators());
    assert_eq!(outcome.status, Status::UpstreamUnavailable);
    assert!(outcome.result == Json::Null && outcome.values.is_empty());
}

#[test]
fn a_5xx_is_retried_and_a_4xx_is_not() {
    let stub = Stub::start();
    let contract = calculators();
    stub.respond(&Json::obj([("detail", Json::str("server"))]), 503);
    let config = config(&stub.base_url, |c| {
        c.max_attempts = BigInt::from(3);
        c.retry_backoff_seconds = 0.01;
    });
    call(&config, &contract);
    assert_eq!(
        stub.requests(),
        3,
        "a 5xx is retried to the configured limit"
    );

    stub.state.lock().unwrap().requests.clear();
    stub.respond(&Json::obj([("refusal_class", Json::str("x"))]), 400);
    call(&config, &contract);
    assert_eq!(stub.requests(), 1, "a 4xx is never retried");
}

/// Set in the child process this test starts, so the discovery runs there
/// and the parent reads what it wrote to stdout and stderr (`capsys`).
const LOG_CHILD: &str = "LODGEIT_CLIENT_OUTCOMES_LOG_CHILD";

#[test]
fn a_get_is_retry_safe_and_the_body_is_never_logged() {
    let name = "a_get_is_retry_safe_and_the_body_is_never_logged";
    if std::env::var_os(LOG_CHILD).is_some() {
        let stub = Stub::start();
        let listing = Json::Array(vec![Json::obj([
            ("calc_uri", Json::str(CALC)),
            ("supported_periods", Json::Array(vec![Json::str(PERIOD)])),
        ])]);
        stub.respond(&listing, 200);
        let outcome = discover(&stub.config(), &calculators());
        assert!(outcome.computed());
        assert_eq!(stub.state.lock().unwrap().requests[0].method, "GET");
        assert_eq!(
            stub.state.lock().unwrap().requests[0].path,
            "/v1/calculators"
        );
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([name, "--exact", "--nocapture", "--test-threads=1"])
        .env(LOG_CHILD, "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("1 passed"), "{stdout}");
    assert!(!stdout.contains("calc_uri") && !stderr.contains("calc_uri"));
}

#[test]
fn the_transport_refuses_a_target_off_the_allowlist_directly() {
    let stub = Stub::start();
    let call = Call {
        method: "GET",
        url: "https://evil.example/v1/calculators",
        body: None,
        content_type: None,
        retry_safe: false,
        bearer_token: None,
    };
    let error = transport::request(&stub.config(), &mut Net, &call, &mut dial::sleep).unwrap_err();
    assert_eq!(error.class, DISALLOWED_TARGET);
}

#[test]
fn a_closed_port_is_a_transport_error_not_a_result() {
    // Port 1 on loopback with nothing listening: a connection failure,
    // which is a TransportError and not an answer.
    let config = config("http://127.0.0.1:1", |c| c.read_timeout = 1.0);
    let call = Call {
        method: "GET",
        url: "http://127.0.0.1:1/v1/calculators",
        body: None,
        content_type: None,
        retry_safe: false,
        bearer_token: None,
    };
    let error = transport::request(&config, &mut Net, &call, &mut dial::sleep).unwrap_err();
    assert_eq!(error.class, TRANSPORT);
    let outcome = discover(&config, &calculators());
    assert_eq!(outcome.status, Status::UpstreamUnavailable);
}

#[test]
fn drift_reports_differences_and_updates_nothing() {
    let stub = Stub::start();
    let contract = calculators();
    stub.respond(
        &Json::Array(vec![
            Json::obj([
                ("calc_uri", Json::str(CALC)),
                (
                    "supported_periods",
                    Json::Array(vec![
                        Json::str(PERIOD),
                        Json::str("urn:sbrm:period:div7a:fy2027"),
                    ]),
                ),
            ]),
            Json::obj([
                ("calc_uri", Json::str("urn:sbrm:calculator:fbt:board")),
                (
                    "supported_periods",
                    Json::Array(vec![Json::str("urn:sbrm:period:fbt:fy2026")]),
                ),
            ]),
        ]),
        200,
    );
    let before = std::fs::read(contract.path.fs()).unwrap();
    let config = stub.config();
    let mut sleep: fn(f64) -> _ = dial::sleep;
    let (outcome, findings) = client(&config, &contract, &mut Net, &mut sleep)
        .drift()
        .unwrap();
    assert!(outcome.computed());
    assert!(any_finding(&findings, "fy2027"));
    assert!(any_finding(&findings, "fbt:board"));
    assert_eq!(
        std::fs::read(contract.path.fs()).unwrap(),
        before,
        "a drift report never rewrites the snapshot"
    );
}

#[test]
fn an_unrecorded_response_field_is_noted_but_not_fatal() {
    let stub = Stub::start();
    stub.respond(&with(good(), "brand_new_field", Json::str("surprise")), 200);
    let outcome = call(&stub.config(), &calculators());
    assert_eq!(outcome.status, Status::Computed);
    assert!(any_finding(&outcome.findings, "brand_new_field"));
    // The provider's body is preserved whole.
    assert_eq!(
        outcome.raw_response.get("brand_new_field"),
        Some(&Json::str("surprise"))
    );
    assert!(!pyjson::dumps(&outcome.raw_response, Layout::Canonical).is_empty());
}

/// The live div7a route uses `disclaimer`, not the `notes` the standard
/// shows. Read from a live response on 18 September 2026 and recorded in
/// the snapshot's `advisory_any_of`. A response with neither key is still a
/// contract failure.
#[test]
fn an_advisory_carried_under_disclaimer_is_accepted() {
    let stub = Stub::start();
    let contract = calculators();
    let live_shape = with(
        without(good(), "advisory"),
        "advisory",
        Json::obj([
            (
                "disclaimer",
                Json::str("This is calculator output, not advice."),
            ),
            ("registered_agent_required", Json::Bool(true)),
        ]),
    );
    stub.respond(&live_shape, 200);
    let outcome = call(&stub.config(), &contract);
    assert_eq!(outcome.status, Status::Computed);
    assert!(outcome.advisory.get("disclaimer").is_some_and(Json::truthy));

    let live_shape = with(
        live_shape,
        "advisory",
        Json::obj([("registered_agent_required", Json::Bool(true))]),
    );
    stub.respond(&live_shape, 200);
    let outcome = call(&stub.config(), &contract);
    assert_eq!(outcome.status, Status::ContractFailure);
    assert!(any_finding(&outcome.findings, "notes, disclaimer"));
}

/// Fields a live div7a 200 carries, so none of them reads as unrecorded.
#[test]
fn the_live_response_fields_are_recorded_in_the_snapshot() {
    let contract = calculators();
    let Some(Json::Array(known)) = contract.response_contract.get("known_top_level_fields") else {
        panic!("the snapshot records no known_top_level_fields list");
    };
    for field in [
        "amalgamated_base",
        "period_uri",
        "rate_uris_consumed",
        "statutory_myr",
        "is_complying",
        "interest_accrued",
    ] {
        assert!(known.contains(&Json::str(field)), "{field}");
    }
}
