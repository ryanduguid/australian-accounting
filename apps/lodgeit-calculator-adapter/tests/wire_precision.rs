//! `tests/test_wire_precision.py`: precision on the wire, checked as bytes.
//!
//! The assertions read the request body the loopback stub received, not a
//! value that has been decoded again, because a round trip through floats
//! would hide exactly the defect these tests exist to catch.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use lodgeit_calculator_adapter::client::{Client, Outcome, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::wire::{dumps, format_number, parse_decimal};

// -- the loopback stub (conftest.py's `stub` and `stub_config`) -------------

#[derive(Clone, Debug)]
struct Seen {
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

struct Shared {
    status: u16,
    body: Vec<u8>,
    requests: Vec<Seen>,
}

/// A loopback HTTP stub on an ephemeral port, serving one request per
/// connection with HTTP/1.0 as `http.server` does.
struct Stub {
    base_url: String,
    address: SocketAddr,
    shared: Arc<Mutex<Shared>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Stub {
    fn start() -> Stub {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a loopback port");
        let address = listener.local_addr().expect("the bound address");
        let shared = Arc::new(Mutex::new(Shared {
            status: 200,
            body: b"{}".to_vec(),
            requests: Vec::new(),
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let (shared, stop) = (Arc::clone(&shared), Arc::clone(&stop));
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(stream) = stream {
                        serve(stream, &shared);
                    }
                }
            })
        };
        Stub {
            base_url: format!("http://127.0.0.1:{}", address.port()),
            address,
            shared,
            stop,
            thread: Some(thread),
        }
    }

    /// `state.respond(None, raw=...)`.
    fn respond_raw(&self, body: &[u8]) {
        let mut shared = self.shared.lock().expect("the stub state");
        shared.status = 200;
        shared.body = body.to_vec();
    }

    fn requests(&self) -> Vec<Seen> {
        self.shared.lock().expect("the stub state").requests.clone()
    }

    /// `stub_config`.
    fn config(&self) -> AdapterConfig {
        AdapterConfig {
            enabled: true,
            base_url: Some(self.base_url.clone()),
            allow_loopback: true,
            max_attempts: 1.into(),
            ..AdapterConfig::default()
        }
        .checked()
        .expect("the stub configuration is valid")
    }
}

impl Drop for Stub {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.address);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn serve(stream: TcpStream, shared: &Mutex<Shared>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return;
    }
    let mut headers = Vec::new();
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).unwrap_or(0) == 0 {
            break;
        }
        let header = header.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            headers.push((name.to_owned(), value.trim().to_owned()));
        }
    }
    let length = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0; length];
    if reader.read_exact(&mut body).is_err() {
        return;
    }
    let (status, reply) = {
        let mut state = shared.lock().expect("the stub state");
        state.requests.push(Seen { headers, body });
        (state.status, state.body.clone())
    };
    let mut stream = reader.into_inner();
    let head = format!(
        "HTTP/1.0 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        reply.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&reply);
    let _ = stream.flush();
}

// -- helpers ----------------------------------------------------------------

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn dec(literal: &str) -> Decimal {
    Decimal::parse_exact(literal).expect("a decimal literal")
}

fn contract() -> Contract {
    contract::load("lodgeit-calculators", None).expect("the reviewed snapshot loads")
}

/// `LodgeitClient(config, contract).invoke(...)` over real loopback sockets.
fn invoke(config: &AdapterConfig, contract: &Contract, body: &Json) -> Outcome {
    let mut sleep = dial::sleep;
    let mut client = Client {
        config,
        contract,
        dial: &mut Net,
        sleep: &mut sleep,
    };
    client
        .invoke(
            &pts("urn:sbrm:calculator:div7a:at"),
            &pts("urn:sbrm:period:div7a:fy2026"),
            body,
        )
        .expect("invoke returns an outcome")
}

fn value<'a>(outcome: &'a Outcome, name: &str) -> &'a Decimal {
    let name = pts(name);
    &outcome
        .values
        .iter()
        .find(|(key, _)| *key == name)
        .expect("the value is present")
        .1
}

// -- the tests --------------------------------------------------------------

#[test]
fn a_decimal_is_emitted_as_its_own_digits() {
    let wire = |name: &str, literal: &str| {
        text(&dumps(&Json::obj([(name, Json::Dec(dec(literal)))]), &pts("body")).unwrap())
    };
    assert_eq!(
        wire("amalgamated_base", "55000.10"),
        r#"{"amalgamated_base":55000.10}"#
    );
    assert_eq!(wire("rate", "0.0837"), r#"{"rate":0.0837}"#);
    assert_eq!(wire("x", "0.1"), r#"{"x":0.1}"#);
    // The number a float serialiser would have produced, for contrast.
    assert_eq!(
        pyjson::dumps(&Json::Float(55000.10), Layout::Canonical),
        "55000.1"
    );
}

#[test]
fn a_float_never_reaches_the_wire() {
    assert!(
        dumps(
            &Json::obj([("amalgamated_base", Json::Float(55000.10))]),
            &pts("body")
        )
        .is_err()
    );
    assert!(dumps(&Json::Array(vec![Json::Float(1.5)]), &pts("body")).is_err());
}

#[test]
fn trailing_zeros_and_scale_survive() {
    assert_eq!(format_number(&dec("100.00")), "100.00");
    assert_eq!(format_number(&dec("1E+3")), "1000");
    assert_eq!(format_number(&dec("-0")), "0");
}

#[test]
fn unrepresentable_values_are_refused() {
    for literal in ["NaN", "Infinity", "1E+20", "0.00000000001"] {
        let result = dumps(&Json::obj([("x", Json::Dec(dec(literal)))]), &pts("body"));
        assert!(result.is_err(), "{literal} was written");
    }
}

#[test]
fn reading_refuses_a_json_float_and_accepts_a_decimal_string() {
    let myr = parse_decimal(&Json::str("21874.92"), &pts("myr")).unwrap();
    assert_eq!(myr, dec("21874.92"));
    let rate = parse_decimal(&Json::str("0.0837"), &pts("rate")).unwrap();
    assert_eq!(rate, dec("0.0837"));
    for bad in [
        Json::Float(21874.92),
        Json::Bool(true),
        Json::Null,
        Json::str(""),
        Json::str("not a number"),
    ] {
        assert!(
            parse_decimal(&bad, &pts("myr")).is_err(),
            "{bad:?} was read as a decimal"
        );
    }
}

#[test]
fn the_bytes_the_stub_receives_carry_the_exact_digits() {
    let stub = Stub::start();
    stub.respond_raw(
        br#"{"statutory_myr": "21874.92", "manifest": {"calculator": "x"}, "advisory": {"figure_type": "y", "notes": ["z"]}}"#,
    );
    let contract = contract();
    invoke(
        &stub.config(),
        &contract,
        &Json::obj([
            ("amalgamated_base", Json::Dec(dec("55000.10"))),
            ("loan_term_years", Json::Int(7.into())),
        ]),
    );
    let requests = stub.requests();
    let last = requests.last().expect("the stub received the request");
    let sent = String::from_utf8(last.body.clone()).expect("the body is UTF-8");
    assert!(sent.contains("55000.10"), "{sent}");
    assert!(
        !sent.contains("55000.1,") && !sent.contains("55000.099"),
        "{sent}"
    );
    let content_type = last
        .headers
        .iter()
        .find(|(name, _)| name == "Content-Type")
        .map(|(_, value)| value.as_str());
    assert_eq!(content_type, Some("application/json"));
}

#[test]
fn a_provider_json_number_keeps_its_own_digits_on_the_way_in() {
    let stub = Stub::start();
    stub.respond_raw(
        b"{\"benchmark_rate\": 0.0837, \"statutory_myr\": 21874.92, \
          \"total_repayments\": 22000.00, \"shortfall\": 0.00, \
          \"manifest\": {\"calculator\": \"x\"}, \
          \"advisory\": {\"figure_type\": \"y\", \"notes\": [\"z\"]}}",
    );
    let contract = contract();
    let outcome = invoke(&stub.config(), &contract, &Json::Object(Vec::new()));
    assert!(outcome.computed(), "{:?}", outcome.findings);
    assert_eq!(*value(&outcome, "benchmark_rate"), dec("0.0837"));
    assert_eq!(*value(&outcome, "statutory_myr"), dec("21874.92"));
    assert_eq!(value(&outcome, "benchmark_rate").to_sci_string(), "0.0837");
}

#[test]
fn a_nan_in_a_response_is_a_contract_failure() {
    let stub = Stub::start();
    stub.respond_raw(br#"{"statutory_myr": NaN, "manifest": {}, "advisory": {"notes": ["z"]}}"#);
    let contract = contract();
    let outcome = invoke(&stub.config(), &contract, &Json::Object(Vec::new()));
    assert!(!outcome.computed());
    assert_ne!(outcome.status, Status::Computed);
    assert!(
        outcome
            .findings
            .iter()
            .any(|finding| text(finding).contains("NaN")),
        "{:?}",
        outcome.findings.iter().map(|f| text(f)).collect::<Vec<_>>()
    );
}
