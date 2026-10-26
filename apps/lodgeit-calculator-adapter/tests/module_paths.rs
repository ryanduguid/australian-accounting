//! `tests/test_module_paths.py`: refusals and edge paths in the
//! configuration, contract, Core, wire and evidence modules. Everything here
//! is offline; the client tests use a loopback stub this file starts.
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use num_bigint::BigInt;

use lodgeit_calculator_adapter::client::{self, Client, Outcome, Status};
use lodgeit_calculator_adapter::config::{self, AdapterConfig, KNOWN_HOSTS};
use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::core::{self, CoreConfig};
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::Net;
use lodgeit_calculator_adapter::errors::{CONFIGURATION, CONTRACT, DISALLOWED_TARGET, NOT_ENABLED};
use lodgeit_calculator_adapter::evidence;
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout, PyStr};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::pypath::PyPath;
use lodgeit_calculator_adapter::transport::RequestConfig;
use lodgeit_calculator_adapter::wire;

const CALC: &str = "urn:sbrm:calculator:div7a:at";
const PERIOD: &str = "urn:sbrm:period:div7a:fy2026";
const HOST: &str = KNOWN_HOSTS[0];

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
                          "rate_table_uris": [], "engine": "div7a/1.0.0",
                          "citation": "ITAA 1936 s 109E"}},
            "advisory": {{"figure_type": "div7a_myr", "notes": ["Not advice."]}}}}"#
    ))
}

fn the_contract() -> Contract {
    contract::load("lodgeit-calculators", None).expect("the reviewed snapshot loads")
}

/// A fresh, empty folder for one test.
fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lodgeit-module-paths-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    dir
}

fn py_path(path: &std::path::Path) -> PyPath {
    PyPath::new(path.to_str().expect("a UTF-8 temp path"))
}

fn refusal<T: std::fmt::Debug>(result: Result<T, Exc>) -> (&'static str, String) {
    let error = result.expect_err("a refusal");
    (error.class, text(&error.text))
}

// -- JSON editing helpers ---------------------------------------------------

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

fn remove(value: &mut Json, key: &str) {
    let Json::Object(pairs) = value else {
        panic!("not an object");
    };
    pairs.retain(|(k, _)| k.as_str() != Some(key));
}

// -- the loopback stub: conftest.py's `stub` and `stub_config` --------------

struct StubState {
    status: u16,
    body: Vec<u8>,
    requests: usize,
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
            requests: 0,
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

    fn requests(&self) -> usize {
        self.state.lock().expect("stub state").requests
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
        let mut state = state.lock().expect("stub state");
        state.requests += 1;
        (state.status, state.body.clone())
    };
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        503 => "Service Unavailable",
        _ => "Status",
    };
    let head = format!(
        "HTTP/1.0 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        reply.len()
    );
    let _ = writer.write_all(head.as_bytes());
    let _ = writer.write_all(&reply);
}

/// `LodgeitClient(config, contract).invoke(CALC, PERIOD, body)`.
fn invoke(config: &AdapterConfig, contract: &Contract, body: &Json) -> Outcome {
    let mut sleep = |_: f64| -> Result<(), Exc> { Ok(()) };
    let mut client = Client {
        config,
        contract,
        dial: &mut Net,
        sleep: &mut sleep,
    };
    client
        .invoke(&pts(CALC), &pts(PERIOD), body)
        .expect("invoke returns an outcome")
}

// -- configuration ----------------------------------------------------------

#[test]
fn a_limit_outside_its_range_is_refused() {
    // Python's `retry_backoff_seconds="1"` is a str the dataclass refuses;
    // the Rust field is an f64, so a non-positive value stands in for it.
    let cases: [(AdapterConfig, &str); 5] = [
        (
            AdapterConfig {
                read_timeout: 0.0,
                ..AdapterConfig::default()
            },
            "read_timeout must be a positive number",
        ),
        (
            AdapterConfig {
                retry_backoff_seconds: -1.0,
                ..AdapterConfig::default()
            },
            "retry_backoff_seconds must be a positive number",
        ),
        (
            AdapterConfig {
                max_response_bytes: BigInt::from(0),
                ..AdapterConfig::default()
            },
            "max_response_bytes must be positive",
        ),
        (
            AdapterConfig {
                max_attempts: BigInt::from(0),
                ..AdapterConfig::default()
            },
            "max_attempts must be between 1 and 5",
        ),
        (
            AdapterConfig {
                max_attempts: BigInt::from(6),
                ..AdapterConfig::default()
            },
            "max_attempts must be between 1 and 5",
        ),
    ];
    for (options, message) in cases {
        assert_eq!(
            refusal(options.checked()),
            (CONFIGURATION, message.to_owned())
        );
    }
}

#[test]
fn a_loopback_url_with_another_scheme_is_refused() {
    let config = AdapterConfig {
        allow_loopback: true,
        ..AdapterConfig::default()
    };
    let (class, message) = refusal(config.check_url("ftp://127.0.0.1/v1/calculators", true));
    assert_eq!(class, DISALLOWED_TARGET);
    assert!(message.contains("scheme 'ftp' is refused"), "{message}");
}

#[test]
fn a_path_that_is_not_in_its_decoded_form_is_refused() {
    let url = format!("https://{HOST}/v1/calculators/%61");
    let (class, message) = refusal(AdapterConfig::default().check_url(&url, true));
    assert_eq!(class, DISALLOWED_TARGET);
    assert!(
        message.contains("the path is not in its decoded form"),
        "{message}"
    );
}

#[test]
fn the_environment_can_name_the_allowed_hosts_and_the_limits() {
    let environ: HashMap<&str, String> = HashMap::from([
        (
            "LODGEIT_ADAPTER_ALLOWED_HOSTS",
            format!(" {HOST} , calc.example.test ,"),
        ),
        ("LODGEIT_ADAPTER_READ_TIMEOUT", "5".to_owned()),
        ("LODGEIT_ADAPTER_MAX_ATTEMPTS", "3".to_owned()),
    ]);
    let config = config::from_environment(|name| environ.get(name).cloned())
        .expect("the environment is valid");
    assert_eq!(config.allowed_hosts, [HOST, "calc.example.test"]);
    assert_eq!(config.read_timeout, 5.0);
    assert_eq!(config.max_attempts, BigInt::from(3));
}

#[test]
fn an_environment_limit_that_is_not_a_number_is_refused() {
    let result = config::from_environment(|name| {
        (name == "LODGEIT_ADAPTER_READ_TIMEOUT").then(|| "soon".to_owned())
    });
    assert_eq!(
        refusal(result),
        (
            CONFIGURATION,
            "LODGEIT_ADAPTER_READ_TIMEOUT='soon' is not a number".to_owned()
        )
    );
}

// -- the contract snapshot --------------------------------------------------

#[test]
fn the_snapshot_folder_falls_back_to_the_first_candidate() {
    // Python patches CONTRACTS_DIRS; the Rust candidates are fixed (the
    // source checkout, then `contracts` beside the binary), so this checks
    // their order and that the first existing one is chosen.
    let dirs = contract::contracts_dirs();
    assert_eq!(dirs.len(), 2);
    assert_eq!(
        dirs[0].display(),
        PyPath::new(env!("CARGO_MANIFEST_DIR"))
            .join("contracts")
            .display()
    );
    let exe = std::env::current_exe().expect("the test binary");
    let beside = py_path(exe.parent().expect("a parent folder")).join("contracts");
    assert_eq!(dirs[1].display(), beside.display());
    let chosen = dirs
        .iter()
        .find(|dir| dir.fs().is_dir())
        .unwrap_or(&dirs[0]);
    assert_eq!(contract::contracts_dir().display(), chosen.display());
    assert_eq!(contract::contracts_dir().display(), dirs[0].display());
}

#[test]
fn a_snapshot_that_is_not_there_is_refused() {
    let dir = tmp_dir("absent");
    let (class, message) = refusal(contract::load("absent", Some(&py_path(&dir))));
    assert_eq!(class, CONTRACT);
    assert!(message.contains("could not be read"), "{message}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_snapshot_that_is_not_json_is_refused() {
    for (index, payload) in [&b"{not json"[..], &b"\xff\xfe"[..]]
        .into_iter()
        .enumerate()
    {
        let dir = tmp_dir(&format!("broken-{index}"));
        std::fs::write(dir.join("broken.json"), payload).expect("write the snapshot");
        let (class, message) = refusal(contract::load("broken", Some(&py_path(&dir))));
        assert_eq!(class, CONTRACT);
        assert!(message.contains("is not valid JSON"), "{message}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn a_snapshot_missing_a_section_names_it() {
    let dir = tmp_dir("thin");
    std::fs::write(dir.join("thin.json"), r#"{"provider": "x"}"#).expect("write the snapshot");
    let (class, message) = refusal(contract::load("thin", Some(&py_path(&dir))));
    assert_eq!(class, CONTRACT);
    assert!(
        message.contains("is missing calculators, read_at, read_by"),
        "{message}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn live_from(contract: &Contract) -> Vec<Json> {
    let Json::Object(calculators) = &contract.calculators else {
        panic!("calculators is an object");
    };
    calculators
        .iter()
        .map(|(uri, spec)| {
            Json::obj([
                ("calc_uri", Json::Str(uri.clone())),
                (
                    "supported_periods",
                    spec.get("supported_periods")
                        .cloned()
                        .unwrap_or(Json::Array(Vec::new())),
                ),
                (
                    "input_schema_ref",
                    spec.get("input_schema_ref").cloned().unwrap_or(Json::Null),
                ),
            ])
        })
        .collect()
}

fn live_entry<'a>(live: &'a mut [Json], uri: &str) -> &'a mut Json {
    live.iter_mut()
        .find(|entry| entry.get("calc_uri").and_then(Json::as_str) == Some(uri))
        .expect("the calculator is listed")
}

#[test]
fn a_calculator_with_no_recorded_schema_ref_is_not_compared_on_it() {
    let contract = the_contract();
    let mut unreferenced = contract.clone();
    remove(
        member_mut(&mut unreferenced.calculators, CALC),
        "input_schema_ref",
    );
    let mut live = live_from(&contract);
    set(
        live_entry(&mut live, CALC),
        "input_schema_ref",
        Json::str("#/moved"),
    );
    let findings = contract::compare(&unreferenced, &live).expect("compare runs");
    assert!(findings.is_empty());
}

#[test]
fn a_moved_schema_ref_is_drift() {
    let contract = the_contract();
    let mut live = live_from(&contract);
    let entry = live_entry(&mut live, CALC);
    let recorded = entry
        .get("input_schema_ref")
        .and_then(Json::as_str)
        .expect("a recorded schema ref")
        .to_owned();
    set(
        entry,
        "input_schema_ref",
        Json::str("#/components/schemas/Moved"),
    );
    let findings: Vec<String> = contract::compare(&contract, &live)
        .expect("compare runs")
        .iter()
        .map(|f| text(f))
        .collect();
    assert_eq!(
        findings,
        [format!(
            "{CALC}: input schema ref moved from {recorded} to #/components/schemas/Moved"
        )]
    );
}

// -- the calculators client -------------------------------------------------

#[test]
fn drift_without_a_live_listing_compares_nothing() {
    let contract = the_contract();
    let stub = Stub::start();
    stub.respond(&json(r#"{"detail": "down"}"#), 503);
    let config = stub.config();
    let mut sleep = |_: f64| -> Result<(), Exc> { Ok(()) };
    let mut client = Client {
        config: &config,
        contract: &contract,
        dial: &mut Net,
        sleep: &mut sleep,
    };
    let (outcome, findings) = client.drift().expect("drift returns an outcome");
    assert_eq!(outcome.status, Status::UpstreamUnavailable);
    assert!(findings.is_empty());
}

#[test]
fn a_calculator_with_no_recorded_route_is_never_called() {
    let mut contract = the_contract();
    let stub = Stub::start();
    remove(member_mut(&mut contract.calculators, CALC), "route");
    let body = Json::obj([(
        "amalgamated_base",
        Json::Dec(Decimal::parse_exact("1.00").expect("a decimal literal")),
    )]);
    let outcome = invoke(&stub.config(), &contract, &body);
    assert_eq!(outcome.status, Status::ContractFailure);
    let findings: Vec<String> = outcome.findings.iter().map(|f| text(f)).collect();
    assert_eq!(findings, [format!("snapshot records no route for {CALC}")]);
    assert_eq!(stub.requests(), 0);
}

#[test]
fn a_snapshot_that_refuses_unknown_fields_fails_a_response_carrying_one() {
    let mut strict = the_contract();
    let stub = Stub::start();
    let mut payload = good();
    set(&mut payload, "surprise", Json::str("1"));
    stub.respond(&payload, 200);
    set(
        &mut strict.response_contract,
        "refuse_unknown_fields",
        Json::Bool(true),
    );
    let body = Json::obj([("amalgamated_base", Json::Int(BigInt::from(1)))]);
    let outcome = invoke(&stub.config(), &strict, &body);
    assert_eq!(outcome.status, Status::ContractFailure);
    let findings: Vec<String> = outcome.findings.iter().map(|f| text(f)).collect();
    assert!(
        findings
            .iter()
            .any(|f| f == "response carries fields the snapshot does not record: surprise"),
        "{findings:?}"
    );
}

// -- Core -------------------------------------------------------------------

#[test]
fn core_makes_one_attempt_with_no_extra_headers() {
    let config = CoreConfig::default();
    assert_eq!(config.max_attempts(), 1);
    assert_eq!(config.retry_backoff_seconds(), 1.0);
    assert!(config.extra_headers().is_empty());
}

#[test]
fn core_access_is_disabled_unless_switched_on() {
    assert_eq!(
        refusal(CoreConfig::default().require_enabled()),
        (NOT_ENABLED, "Core access is disabled.".to_owned())
    );
}

#[test]
fn a_core_page_of_the_wrong_shape_is_no_page() {
    let bodies: [&[u8]; 2] = [
        b"[]",
        b"{\"items\": [{\"id\": \"6f1c1d2e-8f5a-4b8e-9c1d-2e3f4a5b6c7d\", \"code\": 5, \
          \"archived\": false, \"private\": false, \"profile\": null}]}",
    ];
    for body in bodies {
        assert!(core::parse_page(body, 5).is_none());
    }
}

// -- exact decimals on the wire ---------------------------------------------

#[test]
fn an_integer_is_read_exactly() {
    let parsed = wire::parse_decimal(&Json::Int(BigInt::from(7)), &pts("loan_term_years"))
        .expect("an integer is exact");
    assert_eq!(parsed, Decimal::from_int(7));
}

#[test]
fn a_non_finite_decimal_string_is_refused() {
    for value in ["NaN", "Infinity", "-Infinity"] {
        let error = wire::parse_decimal(&Json::str(value), &pts("rate")).expect_err("refused");
        let message = text(&error.0);
        assert!(
            message.contains(&format!("rate: '{value}' is not finite")),
            "{message}"
        );
    }
}

#[test]
fn strings_are_escaped_and_null_is_written_as_null() {
    let note = "a\"b\\c\nd\re\tf\u{1}g";
    let value = Json::obj([("note", Json::str(note)), ("missing", Json::Null)]);
    let written = wire::dumps(&value, &pts("body")).expect("serialisable");
    assert_eq!(
        text(&written),
        r#"{"note":"a\"b\\c\nd\re\tf\u0001g","missing":null}"#
    );
}

#[test]
fn a_value_with_no_exact_form_is_refused() {
    // Rust's `Json` cannot hold a set; a float is the one value it can hold
    // that has no exact wire form, and it is refused at its dotted field.
    let value = Json::obj([("when", Json::Float(2026.0))]);
    let error = wire::dumps(&value, &pts("body")).expect_err("refused");
    let message = text(&error.0);
    assert!(
        message.starts_with("body.when: a float reached the serialiser."),
        "{message}"
    );
}

#[test]
fn an_upstream_error_keeps_the_providers_own_status_and_body() {
    // Rust has no `UpstreamError`; a provider 400's outcome carries the
    // same status, body and refusal class.
    let contract = the_contract();
    let stub = Stub::start();
    let payload = json(r#"{"refusal_class": "scope"}"#);
    stub.respond(&payload, 400);
    let body = Json::obj([("amalgamated_base", Json::Int(BigInt::from(1)))]);
    let outcome = invoke(&stub.config(), &contract, &body);
    assert_eq!(outcome.status, Status::UpstreamRefused);
    assert_eq!(
        (
            text(&outcome.findings[0]),
            outcome.http_status,
            outcome.raw_response,
            outcome.upstream_refusal_class
        ),
        (
            "provider refused: scope".to_owned(),
            Some(400),
            payload,
            Json::str("scope")
        )
    );
}

// -- evidence ---------------------------------------------------------------

fn computed() -> Outcome {
    Outcome {
        status: Status::Computed,
        calculator: Some(pts(CALC)),
        period: Some(pts(PERIOD)),
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: None,
        result: Json::Null,
        values: Vec::new(),
        manifest: Json::obj([("calculator", Json::str(CALC))]),
        advisory: Json::obj([(
            "notes",
            Json::Array(vec![Json::str("Fabricated evidence test.")]),
        )]),
        upstream_refusal_class: Json::Null,
        findings: Vec::new(),
        observed_at: client::now(),
        elapsed_ms: None,
        contract_snapshot: Json::Null,
        contract_sha256: String::new(),
        base_url: None,
    }
}

fn redigest(record: &mut Json) {
    let digest = evidence::digest_of(record).expect("a digest");
    set(record, "calculation_sha256", Json::str(&digest));
}

fn record_with(changes: &[(&str, Json)]) -> Json {
    let mut record =
        evidence::build(&computed(), "fabricated", true, &Json::Null).expect("a record builds");
    for (key, value) in changes {
        set(member_mut(&mut record, "calculation"), key, value.clone());
    }
    redigest(&mut record);
    record
}

fn findings(record: &Json) -> Vec<String> {
    evidence::verify(record)
        .expect("verify runs")
        .iter()
        .map(|f| text(f))
        .collect()
}

#[test]
fn a_label_must_be_a_string() {
    // `build` takes a `&str`; the label check it runs is the one that
    // refuses a non-string.
    assert_eq!(
        evidence::check_label(Some(&Json::Int(BigInt::from(5)))).map_err(|m| text(&m)),
        Err("label must be a string, got int".to_owned())
    );
}

#[test]
fn a_record_without_a_calculation_block_is_reported() {
    let mut record = record_with(&[]);
    remove(&mut record, "calculation");
    assert_eq!(findings(&record), ["no calculation block"]);
}

#[test]
fn a_calculation_that_cannot_be_canonicalised_is_reported() {
    let mut record = record_with(&[]);
    let upstream = member_mut(member_mut(&mut record, "calculation"), "upstream");
    set(upstream, "figure", Json::Float(f64::NAN));
    let found = findings(&record);
    assert_eq!(found.len(), 1);
    assert!(
        found[0].starts_with("calculation block cannot be canonicalised: "),
        "{found:?}"
    );
}

#[test]
fn a_calculation_naming_another_schema_is_reported() {
    assert_eq!(
        findings(&record_with(&[("schema", Json::str("other/1"))])),
        ["the calculation block names a different schema from the record"]
    );
}

#[test]
fn a_calculation_whose_label_is_not_a_slug_is_reported() {
    let found = findings(&record_with(&[("label", Json::str("Not A Slug"))]));
    assert_eq!(found.len(), 1);
    assert!(
        found[0].starts_with("label 'Not A Slug' is not a slug."),
        "{found:?}"
    );
}

#[test]
fn a_computed_record_without_an_advisory_is_reported() {
    let mut record = record_with(&[]);
    let upstream = member_mut(member_mut(&mut record, "calculation"), "upstream");
    set(upstream, "advisory", Json::Null);
    redigest(&mut record);
    assert_eq!(
        findings(&record),
        ["a computed result with no upstream advisory"]
    );
}

#[test]
fn a_block_that_is_not_an_object_is_reported() {
    let found = findings(&record_with(&[("call", Json::str("COMPUTED"))]));
    assert!(
        found.iter().any(|f| f == "call is str, not an object"),
        "{found:?}"
    );
}
