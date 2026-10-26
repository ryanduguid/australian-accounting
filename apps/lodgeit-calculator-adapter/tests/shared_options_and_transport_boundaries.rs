//! Port of `tests/test_shared_options_and_transport_boundaries.py`:
//! regressions for the shared command-line options and the transport
//! boundary.
//!
//! The transport tests use a scripted `Dial` whose replies time out reading
//! the body; the proxy test dials a loopback server the test starts itself.

use std::io::{self, Read, Write};
use std::net::TcpListener;

use lodgeit_calculator_adapter::argparse::{Get, Namespace, Value};
use lodgeit_calculator_adapter::cli::build_parser;
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract::{self, Contract, compare};
use lodgeit_calculator_adapter::dial::Net;
use lodgeit_calculator_adapter::errors::TRANSPORT;
use lodgeit_calculator_adapter::pyjson::{Json, PyStr};
use lodgeit_calculator_adapter::transport::{Call, Connection, Dial, request};

const SHARED: [&str; 9] = [
    "--contract",
    "custom",
    "--enable-network",
    "--base-url",
    "http://127.0.0.1:9",
    "--allow-loopback",
    "--evidence-out",
    "out.json",
    "--not-synthetic",
];
const INVOKE: [&str; 7] = [
    "invoke",
    "--calculator",
    "div7a",
    "--period",
    "fy2026",
    "--body",
    "body.json",
];

fn shared() -> Vec<&'static str> {
    SHARED.to_vec()
}

fn parse(words: &[&str]) -> Namespace {
    let args: Vec<String> = words.iter().map(|w| (*w).to_owned()).collect();
    build_parser()
        .parse(&args)
        .unwrap_or_else(|exit| panic!("{words:?} did not parse: {exit:?}"))
}

/// `args.evidence_out == Path("out.json")`: Rust keeps the string the CLI
/// later makes a path of.
fn assert_shared(args: &Namespace) {
    assert_eq!(args.str("contract").as_deref(), Some("custom"));
    assert!(args.flag("enable_network"));
    assert_eq!(args.str("base_url").as_deref(), Some("http://127.0.0.1:9"));
    assert!(args.flag("allow_loopback"));
    assert_eq!(args.str("evidence_out").as_deref(), Some("out.json"));
    assert!(args.flag("not_synthetic"));
    assert_eq!(args.str("command").as_deref(), Some("invoke"));
    assert_eq!(args.str("calculator").as_deref(), Some("div7a"));
}

#[test]
fn shared_options_before_the_command_word_survive_the_subcommand() {
    // `--not-synthetic invoke ...` used to record synthetic=True.
    let commands: [&[&str]; 4] = [&["contract"], &["discover"], &["drift"], &INVOKE];
    for words in commands {
        let args = parse(&[shared(), words.to_vec()].concat());
        assert!(args.flag("not_synthetic"), "{words:?}");
        assert_eq!(args.str("evidence_out").as_deref(), Some("out.json"));
        assert!(args.flag("enable_network"));
        assert_eq!(args.str("contract").as_deref(), Some("custom"));
    }
}

#[test]
fn shared_options_after_the_command_word_still_work() {
    assert_shared(&parse(&[INVOKE.to_vec(), shared()].concat()));
}

#[test]
fn shared_options_split_around_the_command_word_are_all_kept() {
    let all = shared();
    let (before, after) = all.split_at(5);
    assert_shared(&parse(&[before, &INVOKE, after].concat()));
}

#[test]
fn absent_shared_options_keep_their_defaults() {
    let args = parse(&INVOKE);
    assert_eq!(args.str("contract").as_deref(), Some("lodgeit-calculators"));
    assert_eq!(args.get("enable_network"), Some(&Value::Bool(false)));
    assert_eq!(args.get("base_url"), Some(&Value::None));
    assert_eq!(args.get("allow_loopback"), Some(&Value::Bool(false)));
    assert_eq!(args.get("evidence_out"), Some(&Value::None));
    assert_eq!(args.get("not_synthetic"), Some(&Value::Bool(false)));
}

#[test]
fn an_option_typed_in_both_positions_takes_the_later_value() {
    let args = parse(
        &[
            &["--contract", "first"],
            &INVOKE[..],
            &["--contract", "second"],
        ]
        .concat(),
    );
    assert_eq!(args.str("contract").as_deref(), Some("second"));
}

fn snapshot() -> Contract {
    contract::load("lodgeit-calculators", None).expect("the snapshot loads")
}

fn json_str(text: &str) -> Json {
    Json::Str(PyStr::Text(text.to_owned()))
}

fn set(entry: &mut Json, key: &str, value: Json) {
    let Json::Object(pairs) = entry else {
        panic!("an entry is an object");
    };
    match pairs.iter_mut().find(|(k, _)| k.as_str() == Some(key)) {
        Some(pair) => pair.1 = value,
        None => pairs.push((PyStr::Text(key.to_owned()), value)),
    }
}

/// `_live_from(contract)`: a live listing that agrees with the snapshot.
fn live_from(contract: &Contract) -> Vec<Json> {
    let Json::Object(calculators) = &contract.calculators else {
        panic!("the snapshot's calculators are an object");
    };
    calculators
        .iter()
        .map(|(uri, spec)| {
            let periods = match spec.get("supported_periods") {
                Some(Json::Array(items)) => Json::Array(items.clone()),
                _ => Json::Array(Vec::new()),
            };
            Json::Object(vec![
                (PyStr::Text("calc_uri".into()), Json::Str(uri.clone())),
                (PyStr::Text("supported_periods".into()), periods),
                (
                    PyStr::Text("input_schema_ref".into()),
                    spec.get("input_schema_ref").cloned().unwrap_or(Json::Null),
                ),
            ])
        })
        .collect()
}

fn findings(contract: &Contract, live: &[Json]) -> Vec<String> {
    compare(contract, live)
        .expect("compare returns findings")
        .iter()
        .map(|f| f.iter().filter_map(|&cp| char::from_u32(cp)).collect())
        .collect()
}

#[test]
fn a_matching_live_listing_still_agrees() {
    let contract = snapshot();
    assert_eq!(
        findings(&contract, &live_from(&contract)),
        Vec::<String>::new()
    );
}

#[test]
fn a_repeated_calc_uri_is_drift_in_either_order() {
    // A conflicting duplicate ahead of a matching one used to print agreement.
    let contract = snapshot();
    for conflict_first in [true, false] {
        let live = live_from(&contract);
        let Some(Json::Str(uri)) = live[0].get("calc_uri") else {
            panic!("an entry carries its calc_uri");
        };
        let uri = uri.as_str().expect("a text URI").to_owned();
        let mut conflicting = live[0].clone();
        set(
            &mut conflicting,
            "supported_periods",
            Json::Array(vec![json_str("urn:fabricated:period")]),
        );
        let duplicate = if conflict_first {
            vec![conflicting, live[0].clone()]
        } else {
            vec![live[0].clone(), conflicting]
        };
        let found = findings(&contract, &[duplicate, live[1..].to_vec()].concat());
        let expected = format!("live lists {uri} more than once");
        assert!(found.iter().any(|f| f.contains(&expected)), "{found:?}");
        let identical = [vec![live[0].clone(), live[0].clone()], live[1..].to_vec()].concat();
        assert!(
            !findings(&contract, &identical).is_empty(),
            "an identical repeat is still drift"
        );
    }
}

#[test]
fn a_lost_schema_reference_is_drift() {
    let contract = snapshot();
    for current_ref in [Json::Null, json_str(""), Json::Int(7.into())] {
        let mut live = live_from(&contract);
        let with_ref = live
            .iter_mut()
            .find(|entry| entry.get("input_schema_ref").is_some_and(Json::truthy))
            .expect("one calculator states an input schema ref");
        set(with_ref, "input_schema_ref", current_ref);
        let found = findings(&contract, &live);
        assert!(
            found
                .iter()
                .any(|f| f.contains("no longer states input schema ref")),
            "{found:?}"
        );
    }
}

#[test]
fn the_opener_ignores_environment_proxies() {
    // Python checks that its opener carries no environment-reading
    // ProxyHandler, against a default opener that does. Rust has no proxy
    // support to switch off, so the nearest contract is observable: with the
    // proxy variables naming a port nothing listens on, a request reaches the
    // loopback server directly, in origin form.
    for name in ["https_proxy", "HTTPS_PROXY", "http_proxy", "HTTP_PROXY"] {
        // SAFETY: no other test in this binary reads or writes the
        // environment, and none runs a dial concurrently with this one.
        unsafe { std::env::set_var(name, "http://127.0.0.1:9") };
    }
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port = listener.local_addr().expect("its address").port();
    let server = std::thread::spawn(move || {
        let (mut peer, _) = listener.accept().expect("the request arrives");
        let mut seen = Vec::new();
        while !seen.ends_with(b"\r\n\r\n") {
            let mut byte = [0u8; 1];
            peer.read_exact(&mut byte).expect("the request head");
            seen.push(byte[0]);
        }
        peer.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n[]")
            .expect("the reply");
        seen
    });
    let base = format!("http://127.0.0.1:{port}");
    let url = format!("{base}/v1/calculators");
    let config = AdapterConfig {
        enabled: true,
        base_url: Some(base),
        allow_loopback: true,
        max_attempts: 1.into(),
        read_timeout: 5.0,
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a valid configuration");
    let call = Call {
        method: "GET",
        url: &url,
        body: None,
        content_type: None,
        retry_safe: false,
        bearer_token: None,
    };
    let raw = request(&config, &mut Net, &call, &mut |_| Ok(())).expect("the server answers");
    assert_eq!(raw.status, 200);
    assert_eq!(raw.body, b"[]");
    let seen = server.join().expect("the server thread");
    assert!(seen.starts_with(b"GET /v1/calculators HTTP/1.1\r\n"));
    let host = format!("\r\nHost: 127.0.0.1:{port}\r\n");
    assert!(seen.windows(host.len()).any(|w| w == host.as_bytes()));
}

/// A reply whose status line and headers arrive, then a body read that times
/// out: `_FakeOpener`'s HTTPError over `_TimingOutBody`.
struct TimingOut(io::Cursor<Vec<u8>>);

impl Read for TimingOut {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.0.read(buf)? {
            0 => Err(io::Error::new(io::ErrorKind::TimedOut, "timed out")),
            n => Ok(n),
        }
    }
}

impl Write for TimingOut {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct FakeOpener {
    status: u16,
    opens: usize,
}

impl Dial for FakeOpener {
    fn dial(&mut self, _: &str, _: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
        self.opens += 1;
        let head = format!(
            "HTTP/1.1 {} fabricated\r\nContent-Length: 2\r\n\r\n",
            self.status
        );
        Ok(Box::new(TimingOut(io::Cursor::new(head.into_bytes()))))
    }
}

fn request_with(status: u16, attempts: u32) -> FakeOpener {
    let mut opener = FakeOpener { status, opens: 0 };
    let config = AdapterConfig {
        enabled: true,
        base_url: Some("http://127.0.0.1:9".into()),
        allow_loopback: true,
        max_attempts: attempts.into(),
        retry_backoff_seconds: 0.01,
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a valid configuration");
    let call = Call {
        method: "GET",
        url: "http://127.0.0.1:9/v1/calculators",
        body: None,
        content_type: None,
        retry_safe: true,
        bearer_token: None,
    };
    let error = request(&config, &mut opener, &call, &mut |_| Ok(())).unwrap_err();
    let message: String = error
        .text
        .iter()
        .filter_map(|&cp| char::from_u32(cp))
        .collect();
    assert_eq!(error.class, TRANSPORT);
    assert!(
        message.contains(&format!("HTTP {status} body could not be read")),
        "{message:?}"
    );
    opener
}

#[test]
fn a_timed_out_5xx_body_is_retried_then_normalised() {
    // A raw TimeoutError used to escape after one attempt despite max_attempts=2.
    assert_eq!(request_with(503, 2).opens, 2);
}

#[test]
fn a_timed_out_4xx_body_is_not_retried() {
    assert_eq!(request_with(400, 3).opens, 1);
}
