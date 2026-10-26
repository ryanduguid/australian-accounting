//! `tests/test_no_network_by_default.py`: nothing leaves unless asked.
//!
//! Python sabotages `socket.create_connection` and `socket.socket.connect`.
//! The Rust seam is `transport::Dial`: every client here gets a dialler that
//! panics on any dial, so a request that should not happen fails loudly
//! rather than reaching a service. The MCP tests were retired with the MCP
//! surface.

use std::ffi::OsString;
use std::io;

use lodgeit_calculator_adapter::cli::{self, Io};
use lodgeit_calculator_adapter::client::{Client, Status};
use lodgeit_calculator_adapter::config::{self, AdapterConfig};
use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::errors::{DISALLOWED_TARGET, NOT_ENABLED};
use lodgeit_calculator_adapter::pyjson::{Json, PyStr};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::transport::{self, Call, Connection, Dial};
use lodgeit_calculator_adapter::trials::div7a;

const HOST: &str = "fbt-calculator-api-qkp3j5bjnq-ts.a.run.app";

/// The `no_sockets` fixture: any dial is a failure of the test.
struct NoSockets;

impl Dial for NoSockets {
    fn dial(&mut self, host: &str, port: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
        panic!("a socket was opened when none should have been ({host}:{port})");
    }
}

/// `time.sleep` must not be reached either: no request means no retry.
fn no_sleep(_: f64) -> Result<(), Exc> {
    panic!("a retry backoff was slept when no request should have been made");
}

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn contract() -> Contract {
    contract::load("lodgeit-calculators", None).expect("the reviewed snapshot loads")
}

fn enabled() -> AdapterConfig {
    AdapterConfig {
        enabled: true,
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a valid configuration")
}

fn has_key(mapping: &Json, key: &str) -> bool {
    matches!(mapping, Json::Object(pairs) if pairs.iter().any(|(k, _)| k.as_str() == Some(key)))
}

/// Python imports every module in a fresh interpreter with sockets
/// sabotaged. Rust has no import-time code, so the nearest contract is that
/// the crate's offline entry points (both snapshots, the configuration
/// constructors, a client, the packaged trial cases and the offline
/// `contract` command) run to completion with a dialler that panics.
#[test]
fn importing_every_module_opens_no_socket() {
    let calculators = contract();
    contract::load("fano-classifier", None).expect("the Fano snapshot loads");
    let _ = AdapterConfig::default()
        .checked()
        .expect("the defaults are valid");
    let _ = config::from_environment(|_| None).expect("an empty environment is valid");
    let _ = div7a::load_cases(None).expect("the packaged cases load");
    let config = AdapterConfig::default();
    let mut sleep = no_sleep;
    let _client = Client {
        config: &config,
        contract: &calculators,
        dial: &mut NoSockets,
        sleep: &mut sleep,
    };

    let env = |_: &str| -> Option<OsString> { None };
    let mut sleep = no_sleep;
    let mut io = Io {
        env: &env,
        dial: &mut NoSockets,
        sleep: &mut sleep,
    };
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = cli::run_io(&["contract".to_owned()], &mut stdout, &mut stderr, &mut io);
    assert_eq!(code, 0, "{}", String::from_utf8_lossy(&stderr));
    assert!(String::from_utf8_lossy(&stdout).contains("lodgeit-calculators-2026-09-18"));
}

#[test]
fn the_default_configuration_is_off() {
    let config = AdapterConfig::default().checked().unwrap();
    assert!(!config.enabled);
    assert_eq!(config.base_url, None);
    let error = config.require_enabled().unwrap_err();
    assert_eq!(error.class, NOT_ENABLED);
}

#[test]
fn an_environment_without_the_switch_is_still_off() {
    let base_url = format!("https://{HOST}");
    let config = config::from_environment(|name| {
        (name == "LODGEIT_ADAPTER_BASE_URL").then(|| base_url.clone())
    })
    .unwrap();
    assert!(!config.enabled);
}

#[test]
fn discovery_offline_refuses_to_send_rather_than_answering() {
    let contract = contract();
    let config = AdapterConfig::default();
    let mut sleep = no_sleep;
    let mut client = Client {
        config: &config,
        contract: &contract,
        dial: &mut NoSockets,
        sleep: &mut sleep,
    };
    let outcome = client.discover().unwrap();
    assert_eq!(outcome.status, Status::RefusedToSend);
    assert_eq!(outcome.result, Json::Null);
    assert!(
        outcome
            .findings
            .iter()
            .any(|finding| text(finding).contains("not enabled"))
    );
}

#[test]
fn invocation_offline_refuses_and_produces_no_figure() {
    let contract = contract();
    let config = AdapterConfig::default();
    let mut sleep = no_sleep;
    let mut client = Client {
        config: &config,
        contract: &contract,
        dial: &mut NoSockets,
        sleep: &mut sleep,
    };
    let outcome = client
        .invoke(
            &pts("urn:sbrm:calculator:div7a:at"),
            &pts("urn:sbrm:period:div7a:fy2026"),
            &Json::obj([("amalgamated_base", Json::Int(1.into()))]),
        )
        .unwrap();
    assert_eq!(outcome.status, Status::RefusedToSend);
    assert_eq!(outcome.result, Json::Null);
    assert!(outcome.values.is_empty());
}

#[test]
fn the_reviewed_snapshot_is_readable_offline() {
    let contract = contract();
    assert_eq!(
        contract.snapshot_id,
        Json::Str(PyStr::Text("lodgeit-calculators-2026-09-18".into()))
    );
    assert!(has_key(
        &contract.calculators,
        "urn:sbrm:calculator:div7a:at"
    ));
}

#[test]
fn enabled_without_a_base_url_still_sends_nothing() {
    let contract = contract();
    let config = enabled();
    let mut sleep = no_sleep;
    let mut client = Client {
        config: &config,
        contract: &contract,
        dial: &mut NoSockets,
        sleep: &mut sleep,
    };
    let outcome = client.discover().unwrap();
    assert_eq!(outcome.status, Status::RefusedToSend);
}

#[test]
fn a_host_off_the_allowlist_is_refused_before_a_socket_opens() {
    let config = AdapterConfig {
        enabled: true,
        allowed_hosts: vec![HOST.to_owned()],
        ..AdapterConfig::default()
    }
    .checked()
    .unwrap();
    let urls = [
        "https://evil.example/v1/calculators",
        "http://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/v1/calculators",
        "https://169.254.169.254/v1/calculators",
        "https://user:pass@fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/v1/calculators",
        "file:///etc/passwd",
        "https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/../../admin",
        "http://127.0.0.1:9000/v1/calculators",
    ];
    for url in urls {
        let error = config.check_url(url, true).unwrap_err();
        assert_eq!(error.class, DISALLOWED_TARGET, "{url}");
    }
    // The same refusal through the transport, with a base URL so the
    // enablement check passes: it comes before any dial.
    let sending = AdapterConfig {
        base_url: Some(format!("https://{HOST}")),
        ..config
    };
    for url in urls {
        let call = Call {
            method: "GET",
            url,
            body: None,
            content_type: None,
            retry_safe: true,
            bearer_token: None,
        };
        let mut sleep = no_sleep;
        let error = transport::request(&sending, &mut NoSockets, &call, &mut sleep).unwrap_err();
        assert_eq!(error.class, DISALLOWED_TARGET, "{url}");
    }
}

#[test]
fn a_route_the_adapter_does_not_call_is_refused() {
    let config = enabled();
    for url in [
        format!("https://{HOST}/admin/reset"),
        // A schema URL lifted out of a response body is the case this closes.
        format!("https://{HOST}/schemas/anything.json"),
    ] {
        let error = config.check_url(&url, true).unwrap_err();
        assert_eq!(error.class, DISALLOWED_TARGET, "{url}");
    }
}

#[test]
fn loopback_needs_its_own_switch() {
    let off = enabled();
    let error = off
        .check_url("http://127.0.0.1:8000/v1/calculators", true)
        .unwrap_err();
    assert_eq!(error.class, DISALLOWED_TARGET);
    let on = AdapterConfig {
        enabled: true,
        allow_loopback: true,
        ..AdapterConfig::default()
    }
    .checked()
    .unwrap();
    assert!(
        on.check_url("http://127.0.0.1:8000/v1/calculators", true)
            .is_ok()
    );
}
