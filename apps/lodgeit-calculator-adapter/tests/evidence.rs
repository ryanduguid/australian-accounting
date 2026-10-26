//! `tests/test_evidence.py`: evidence, repeatable where it must be and
//! honest about the rest. The calls go to a loopback stub this file starts.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use num_bigint::BigInt;

use lodgeit_calculator_adapter::client::{Client, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::Net;
use lodgeit_calculator_adapter::evidence;
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout, PyStr};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::pypath::PyPath;

const CALC: &str = "urn:sbrm:calculator:div7a:at";
const PERIOD: &str = "urn:sbrm:period:div7a:fy2026";

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn json(text: &str) -> Json {
    pyjson::loads_standard(text).expect("valid JSON")
}

fn good() -> Json {
    json(&format!(
        r#"{{"statutory_myr": "21874.92", "shortfall": "0.00", "total_repayments": "22000.00",
            "benchmark_rate": "0.0837",
            "manifest": {{"calculator": "{CALC}", "period": "{PERIOD}",
                          "rate_table_uris": [{{"uri": "urn:sbrm:rate:div7a:fy2026:benchmark-interest",
                                                "sha256": "{}"}}],
                          "engine": "div7a/1.0.0", "citation": "ITAA 1936 s 109E"}},
            "advisory": {{"figure_type": "div7a_myr", "notes": ["Not advice."]}}}}"#,
        "a".repeat(64)
    ))
}

fn body() -> Json {
    Json::obj([
        (
            "amalgamated_base",
            Json::Dec(Decimal::parse_exact("100000.00").expect("a decimal literal")),
        ),
        ("loan_term_years", Json::Int(BigInt::from(7))),
    ])
}

fn the_contract() -> Contract {
    contract::load("lodgeit-calculators", None).expect("the reviewed snapshot loads")
}

fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lodgeit-evidence-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    dir
}

// -- JSON editing helpers ---------------------------------------------------

fn member<'a>(value: &'a Json, key: &str) -> &'a Json {
    value.get(key).unwrap_or_else(|| panic!("no member {key}"))
}

fn member_mut<'a>(value: &'a mut Json, key: &str) -> &'a mut Json {
    let Json::Object(pairs) = value else {
        panic!("not an object");
    };
    &mut pairs
        .iter_mut()
        .find(|(k, _)| k.as_str() == Some(key))
        .unwrap_or_else(|| panic!("no member {key}"))
        .1
}

fn set(value: &mut Json, key: &str, item: Json) {
    let Json::Object(pairs) = value else {
        panic!("not an object");
    };
    match pairs.iter_mut().find(|(k, _)| k.as_str() == Some(key)) {
        Some(pair) => pair.1 = item,
        None => pairs.push((PyStr::Text(key.to_owned()), item)),
    }
}

fn redigest(record: &mut Json) {
    let digest = evidence::digest_of(record).expect("a digest");
    set(record, "calculation_sha256", Json::str(&digest));
}

fn findings(record: &Json) -> Vec<String> {
    evidence::verify(record)
        .expect("verify runs")
        .iter()
        .map(|f| text(f))
        .collect()
}

// -- the loopback stub: conftest.py's `stub` and `stub_config` --------------

struct StubState {
    status: u16,
    body: Vec<u8>,
}

struct Stub {
    base_url: String,
    state: Arc<Mutex<StubState>>,
}

impl Stub {
    fn start() -> Stub {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a loopback port");
        let port = listener.local_addr().expect("a local address").port();
        let state = Arc::new(Mutex::new(StubState {
            status: 200,
            body: b"{}".to_vec(),
        }));
        let shared = Arc::clone(&state);
        std::thread::spawn(move || {
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
        let mut state = self.state.lock().expect("stub state");
        state.status = status;
        state.body = pyjson::dumps(payload, Layout::Canonical).into_bytes();
    }

    /// `AdapterConfig(enabled=True, base_url=..., allow_loopback=True, max_attempts=1)`.
    fn config(&self) -> AdapterConfig {
        AdapterConfig {
            enabled: true,
            base_url: Some(self.base_url.clone()),
            allow_loopback: true,
            max_attempts: BigInt::from(1),
            ..AdapterConfig::default()
        }
        .checked()
        .expect("the stub config is valid")
    }
}

fn serve(stream: TcpStream, state: &Mutex<StubState>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut length = 0;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() || header == "\r\n" || header.is_empty() {
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
    let (status, reply) = {
        let state = state.lock().expect("stub state");
        (state.status, state.body.clone())
    };
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        _ => "Status",
    };
    let head = format!(
        "HTTP/1.0 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        reply.len()
    );
    let _ = writer.write_all(head.as_bytes());
    let _ = writer.write_all(&reply);
}

/// `LodgeitClient(stub_config, contract).invoke(CALC, PERIOD, BODY)`, its
/// outcome built into a record with the given label.
fn invoke_and_build(stub: &Stub, contract: &Contract, label: &str) -> Json {
    let config = stub.config();
    let mut sleep = |_: f64| -> Result<(), Exc> { Ok(()) };
    let mut client = Client {
        config: &config,
        contract,
        dial: &mut Net,
        sleep: &mut sleep,
    };
    let outcome = client
        .invoke(&pts(CALC), &pts(PERIOD), &body())
        .expect("invoke returns an outcome");
    evidence::build(&outcome, label, true, &Json::Null).expect("a record builds")
}

/// `capture(stub, stub_config, contract, payload, label)`.
fn capture(stub: &Stub, contract: &Contract, payload: Option<Json>, label: &str) -> Json {
    stub.respond(&payload.unwrap_or_else(good), 200);
    invoke_and_build(stub, contract, label)
}

fn captured() -> (Json, Contract) {
    let contract = the_contract();
    let stub = Stub::start();
    (capture(&stub, &contract, None, "trial"), contract)
}

#[test]
fn an_evidence_record_verifies_against_its_own_digest() {
    let (record, _) = captured();
    assert!(findings(&record).is_empty());
    assert_eq!(
        member(&record, "calculation_sha256").as_str(),
        Some(evidence::digest_of(&record).expect("a digest").as_str())
    );
}

#[test]
fn the_digest_covers_the_calculation_and_not_the_observation() {
    // Two identical calls produce the same digest and different observations.
    let contract = the_contract();
    let stub = Stub::start();
    let first = capture(&stub, &contract, None, "trial");
    let mut second = capture(&stub, &contract, None, "trial");
    assert_eq!(
        member(&first, "calculation_sha256"),
        member(&second, "calculation_sha256")
    );
    let observation = member_mut(&mut second, "observation");
    set(observation, "elapsed_ms", Json::Int(BigInt::from(99999)));
    set(
        observation,
        "captured_at",
        Json::str("2099-01-01T00:00:00+00:00"),
    );
    assert!(
        findings(&second).is_empty(),
        "observation metadata is outside the digest"
    );
}

#[test]
fn changing_a_figure_breaks_the_digest() {
    let (mut record, _) = captured();
    let response = member_mut(
        member_mut(member_mut(&mut record, "calculation"), "upstream"),
        "response",
    );
    set(response, "statutory_myr", Json::str("1.00"));
    let found = findings(&record);
    assert!(
        found.iter().any(|f| f.contains("does not match")),
        "{found:?}"
    );
}

#[test]
fn the_upstream_response_is_preserved_and_normalisation_sits_beside_it() {
    let (record, _) = captured();
    let calculation = member(&record, "calculation");
    let upstream = member(calculation, "upstream");
    assert_eq!(
        member(member(upstream, "response"), "statutory_myr").as_str(),
        Some("21874.92")
    );
    assert_eq!(
        member(member(upstream, "manifest"), "citation").as_str(),
        Some("ITAA 1936 s 109E")
    );
    assert_eq!(
        member(member(upstream, "advisory"), "notes"),
        &Json::Array(vec![Json::str("Not advice.")])
    );
    let normalised = member(calculation, "normalised");
    assert_eq!(
        member(member(normalised, "values"), "statutory_myr").as_str(),
        Some("21874.92")
    );
    assert!(
        member(normalised, "note")
            .as_str()
            .is_some_and(|note| note.contains("unaltered"))
    );
}

#[test]
fn the_input_is_recorded_with_its_exact_wire_form() {
    let (record, _) = captured();
    let calculation = member(&record, "calculation");
    assert!(
        member(calculation, "input_json")
            .as_str()
            .is_some_and(|sent| sent.contains("100000.00"))
    );
    assert_eq!(member(calculation, "synthetic_input"), &Json::Bool(true));
}

#[test]
fn a_refusal_is_recorded_as_evidence_too() {
    let contract = the_contract();
    let stub = Stub::start();
    let refusal = json(r#"{"refusal_class": "insufficient_facts"}"#);
    let _ = capture(&stub, &contract, Some(refusal.clone()), "trial");
    // The stub answers 200 by default, so force the refusal path explicitly.
    stub.respond(&refusal, 400);
    let record = invoke_and_build(&stub, &contract, "refusal");
    assert!(findings(&record).is_empty());
    let calculation = member(&record, "calculation");
    assert_eq!(
        member(member(calculation, "call"), "status").as_str(),
        Some(Status::UpstreamRefused.as_str())
    );
    assert_eq!(
        member(member(calculation, "upstream"), "refusal_class").as_str(),
        Some("insufficient_facts")
    );
    assert_eq!(
        member(member(calculation, "normalised"), "values"),
        &Json::Object(Vec::new())
    );
    assert_eq!(
        member(member(calculation, "validation"), "accepted"),
        &Json::Bool(false)
    );
}

#[test]
fn a_computed_record_without_a_manifest_is_reported() {
    let (mut record, _) = captured();
    let upstream = member_mut(member_mut(&mut record, "calculation"), "upstream");
    set(upstream, "manifest", Json::Null);
    redigest(&mut record);
    let found = findings(&record);
    assert!(
        found.iter().any(|f| f.contains("no upstream manifest")),
        "{found:?}"
    );
}

#[test]
fn an_unknown_schema_is_refused() {
    let (mut record, _) = captured();
    set(&mut record, "schema", Json::str("something-else/9"));
    let found = findings(&record);
    assert!(
        found.iter().any(|f| f.contains("unknown evidence schema")),
        "{found:?}"
    );
}

#[test]
fn a_record_round_trips_through_a_file() {
    let (record, _) = captured();
    let dir = tmp_dir("round-trip");
    let path = PyPath::new(dir.join("evidence.json").to_str().expect("a UTF-8 path"));
    evidence::write(&record, &path).expect("the record is written");
    let reloaded = evidence::read(&path).unwrap_or_else(|_| panic!("the record reads back"));
    assert_eq!(reloaded, record);
    assert!(findings(&reloaded).is_empty());
    let written = std::fs::read_to_string(path.fs()).expect("the file reads");
    assert!(written.ends_with('\n'));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_record_carries_the_contract_snapshot_it_was_checked_against() {
    let (record, contract) = captured();
    let provider = member(member(&record, "calculation"), "provider");
    assert_eq!(
        member(provider, "contract_snapshot").as_str(),
        Some("lodgeit-calculators-2026-09-18")
    );
    assert_eq!(
        member(provider, "contract_sha256").as_str(),
        Some(contract.sha256.as_str())
    );
}

#[test]
fn no_boundary_claim_is_overstated() {
    let (record, _) = captured();
    let Json::Array(lines) = member(member(&record, "calculation"), "boundary") else {
        panic!("boundary is a list");
    };
    let boundary = lines
        .iter()
        .map(|line| line.as_str().expect("a boundary line"))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(boundary.contains("Not advice"));
    assert!(boundary.contains("proves nothing about whether the figure is right"));
    let serialised = pyjson::dumps(&record, Layout::Canonical).to_lowercase();
    for claim in ["approved", "certified", "attested", "conformant"] {
        assert!(!serialised.contains(claim), "{claim}");
    }
}
