//! `tests/test_review_regressions.py`: regressions for defects an independent
//! review pass demonstrated, one test per defect, named for the behaviour it
//! pins. Everything here runs offline against a loopback stub this file
//! starts itself.

use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use lodgeit_calculator_adapter::cli::{self, Io};
use lodgeit_calculator_adapter::client::{Client, Outcome, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::errors::{CONTRACT, DISALLOWED_TARGET};
use lodgeit_calculator_adapter::evidence;
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout, Numbers, PyStr};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::trials::{Evaluation, Money, compare_values, div7a, fano, fbt};

const HOST: &str = "https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app";
const DIV7A: &str = "urn:sbrm:calculator:div7a:at";
const FY2026: &str = "urn:sbrm:period:div7a:fy2026";
const DIV7A_RESPONSE: &str = r#"{"statutory_myr": "21874.92", "total_repayments": "22000.00", "shortfall": "0.00", "benchmark_rate": "0.0837", "manifest": {"rate_uris_consumed": ["urn:sbrm:rate:div7a:fy2026"]}, "advisory": {"disclaimer": "Calculated on supplied facts."}}"#;

// -- the loopback stub (conftest.py's `stub`) -------------------------------

#[derive(Clone, Debug)]
struct Seen {
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

    /// `state.respond(payload, status=status)`, with the payload as JSON text.
    fn respond_status(&self, body: &str, status: u16) {
        let mut shared = self.shared.lock().expect("the stub state");
        shared.status = status;
        shared.body = body.as_bytes().to_vec();
    }

    fn respond(&self, body: &str) {
        self.respond_status(body, 200);
    }

    fn requests(&self) -> Vec<Seen> {
        self.shared.lock().expect("the stub state").requests.clone()
    }

    /// The last request body the stub received, as text.
    fn sent(&self) -> String {
        let requests = self.requests();
        let last = requests.last().expect("the stub received a request");
        String::from_utf8(last.body.clone()).expect("the body is UTF-8")
    }

    /// `AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1)`.
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
    let mut length = 0;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).unwrap_or(0) == 0 {
            break;
        }
        let header = header.trim_end_matches(['\r', '\n']);
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
    let (status, reply) = {
        let mut state = shared.lock().expect("the stub state");
        state.requests.push(Seen { body });
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

fn texts(points: &[Vec<u32>]) -> Vec<String> {
    points.iter().map(|p| text(p)).collect()
}

fn any_contains(findings: &[Vec<u32>], needle: &str) -> bool {
    findings
        .iter()
        .any(|finding| text(finding).contains(needle))
}

fn dec(literal: &str) -> Decimal {
    Decimal::parse_exact(literal).expect("a decimal literal")
}

fn money(pairs: &[(&str, &str)]) -> Money {
    pairs
        .iter()
        .map(|(name, value)| (pts(name), dec(value)))
        .collect()
}

fn cent() -> Decimal {
    dec("0.01")
}

fn contract() -> Contract {
    contract::load("lodgeit-calculators", None).expect("the reviewed snapshot loads")
}

fn fano_contract() -> Contract {
    contract::load("fano-classifier", None).expect("the Fano snapshot loads")
}

fn enabled() -> AdapterConfig {
    AdapterConfig {
        enabled: true,
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a valid configuration")
}

/// `LodgeitClient(config, contract)`, over real loopback sockets, given to `f`.
fn with_client<T>(
    config: &AdapterConfig,
    contract: &Contract,
    f: impl FnOnce(&mut Client<'_>) -> T,
) -> T {
    let mut sleep = dial::sleep;
    let mut client = Client {
        config,
        contract,
        dial: &mut Net,
        sleep: &mut sleep,
    };
    f(&mut client)
}

fn invoke_div7a(stub: &Stub, contract: &Contract, body: &Json) -> Outcome {
    with_client(&stub.config(), contract, |client| {
        client
            .invoke(&pts(DIV7A), &pts(FY2026), body)
            .expect("invoke returns an outcome")
    })
}

fn base_body() -> Json {
    Json::obj([("amalgamated_base", Json::Dec(dec("100000.00")))])
}

fn drift(stub: &Stub, contract: &Contract) -> (Outcome, Vec<Vec<u32>>) {
    with_client(&stub.config(), contract, |client| {
        client.drift().expect("drift returns findings")
    })
}

fn discover(stub: &Stub, contract: &Contract) -> Outcome {
    with_client(&stub.config(), contract, |client| {
        client.discover().expect("discover returns an outcome")
    })
}

/// `record[key]`, for assignment.
fn member_mut<'a>(json: &'a mut Json, key: &str) -> &'a mut Json {
    let Json::Object(pairs) = json else {
        panic!("{key}: not an object")
    };
    &mut pairs
        .iter_mut()
        .find(|(name, _)| name.as_str() == Some(key))
        .unwrap_or_else(|| panic!("{key} is absent"))
        .1
}

fn redigest(record: &mut Json) {
    let digest = evidence::digest_of(record).expect("the record canonicalises");
    *member_mut(record, "calculation_sha256") = Json::str(&digest);
}

/// `main(argv)` with an empty environment, real sockets and `time.sleep`.
fn run_cli(args: &[&str]) -> (i32, String, String) {
    let env = |_: &str| -> Option<OsString> { None };
    let mut sleep = dial::sleep;
    let mut io = Io {
        env: &env,
        dial: &mut Net,
        sleep: &mut sleep,
    };
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = cli::run_io(&args, &mut stdout, &mut stderr, &mut io);
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// pytest's `tmp_path`: a fresh directory, removed when dropped.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> TmpPath {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "lodgeit-review-regressions-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create a temporary directory");
        TmpPath(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn path_str(path: &std::path::Path) -> &str {
    path.to_str().expect("a Unicode temporary path")
}

// -- the route allowlist ----------------------------------------------------

#[test]
fn a_path_that_is_not_a_route_is_refused_whatever_it_is_dressed_as() {
    let config = enabled();
    for path in [
        // A dot segment lands wherever the first thing that normalises decides.
        "/v1/calculators/../../admin",
        "/v1/calculators/./../admin",
        // Percent-encoded, so a check that never decodes sees no dot segment.
        "/v1/calculators/%2e%2e/%2e%2e/admin",
        // An encoded slash: one segment before decoding, three after it.
        "/v1/calculators/..%2f..%2fadmin",
        // A backslash is a path delimiter to a browser and to some servers.
        "/v1/calculators/..%5c..%5cadmin",
        // Prefix extension: `/v1/ratesheet` starts with `/v1/rates` and is not it.
        "/v1/ratesheet-evil",
        "/v1/calculatorsX",
    ] {
        let error = config
            .check_url(&format!("{HOST}{path}"), true)
            .unwrap_err();
        assert_eq!(error.class, DISALLOWED_TARGET, "{path}");
    }
}

#[test]
fn the_routes_this_adapter_actually_calls_still_pass() {
    let config = enabled();
    for path in [
        "/v1/calculators",
        "/v1/calculators/div7a/at/urn:sbrm:period:div7a:fy2026",
        "/v1/rates/urn:sbrm:period:div7a:fy2026",
        "/openapi.json",
        "/healthz",
    ] {
        assert!(
            config.check_url(&format!("{HOST}{path}"), true).is_ok(),
            "{path}"
        );
    }
}

#[test]
fn a_port_off_loopback_is_part_of_the_destination() {
    let error = enabled()
        .check_url(&format!("{HOST}:9999/v1/calculators"), true)
        .unwrap_err();
    assert_eq!(error.class, DISALLOWED_TARGET);
    assert!(text(&error.text).contains("port 9999"));
}

#[test]
fn a_base_url_carrying_a_query_is_refused_at_construction() {
    // `base + path` put the whole route inside the query string, so every
    // invocation reached the base URL's own route and nothing said so.
    let error = AdapterConfig {
        enabled: true,
        base_url: Some(format!("{HOST}?trace=1")),
        ..AdapterConfig::default()
    }
    .checked()
    .unwrap_err();
    assert_eq!(error.class, DISALLOWED_TARGET);
    assert!(text(&error.text).contains("query string"));
}

#[test]
fn an_ephemeral_loopback_port_is_still_reachable_for_a_stub() {
    let stub = Stub::start();
    let config = stub.config();
    assert!(
        config
            .check_url(&format!("{}/v1/calculators", stub.base_url), true)
            .is_ok()
    );
}

// -- what a refused response may hand back ----------------------------------

#[test]
fn a_contract_failure_hands_back_no_result() {
    let stub = Stub::start();
    // No manifest and no advisory: the response is refused.
    stub.respond(r#"{"statutory_myr": "21874.92", "total_repayments": "22000.00", "shortfall": "0.00", "benchmark_rate": "0.0837"}"#);
    let outcome = invoke_div7a(&stub, &contract(), &base_body());
    assert_eq!(outcome.status, Status::ContractFailure);
    // The money fields of a refused body must not travel as a result. The
    // body itself is preserved for audit in raw_response.
    assert_eq!(outcome.result, Json::Null);
    assert!(outcome.values.is_empty());
    assert_eq!(
        outcome.raw_response.get("statutory_myr"),
        Some(&Json::str("21874.92"))
    );
}

#[test]
fn an_empty_manifest_is_not_a_manifest() {
    let stub = Stub::start();
    stub.respond(r#"{"statutory_myr": "21874.92", "total_repayments": "22000.00", "shortfall": "0.00", "benchmark_rate": "0.0837", "manifest": {}, "advisory": {"disclaimer": "Calculated on supplied facts."}}"#);
    let outcome = invoke_div7a(&stub, &contract(), &base_body());
    assert_eq!(outcome.status, Status::ContractFailure);
    assert!(any_contains(
        &outcome.findings,
        "does not name what it consumed"
    ));
}

#[test]
fn a_provider_that_declines_to_answer_is_a_refusal_not_a_contract_failure() {
    let contract = contract();
    for status in [401, 403] {
        let stub = Stub::start();
        stub.respond_status(r#"{"detail": "Not authenticated"}"#, status);
        let outcome = invoke_div7a(&stub, &contract, &base_body());
        assert_eq!(outcome.status, Status::UpstreamRefused, "{status}");
        assert!(
            any_contains(&outcome.findings, "declined to answer"),
            "{status}"
        );
    }
}

#[test]
fn a_body_that_cannot_be_encoded_is_a_refusal_not_a_traceback() {
    let stub = Stub::start();
    // A lone surrogate survives the serialiser and fails the UTF-8 encode.
    let body = Json::obj([
        ("amalgamated_base", Json::Dec(dec("100000.00"))),
        ("note", Json::Str(PyStr::from_code_points(vec![0xD800]))),
    ]);
    let outcome = invoke_div7a(&stub, &contract(), &body);
    assert_eq!(outcome.status, Status::ContractFailure);
    assert!(any_contains(&outcome.findings, "could not be serialised"));
}

// -- reading a request body from a file -------------------------------------

fn fields(names: &[&str]) -> Json {
    Json::Array(names.iter().map(|name| Json::str(name)).collect())
}

#[test]
fn an_identifier_is_not_a_number_just_because_it_parses_as_one() {
    let body = Json::obj([
        ("amalgamated_base", Json::str("100000.00")),
        ("reference", Json::str("0012")),
        ("abn", Json::str("51824753556")),
        ("loan_origination_date", Json::str("2024-09-01")),
        (
            "repayments",
            Json::Array(vec![Json::obj([
                ("amount", Json::str("11000.00")),
                ("date", Json::str("2025-12-15")),
            ])]),
        ),
    ]);
    let converted = cli::decimalise(
        body.clone(),
        &fields(&["amalgamated_base", "repayments[].amount"]),
    )
    .expect("no uncaught error")
    .expect("the body converts");
    assert_eq!(
        converted.get("amalgamated_base"),
        Some(&Json::Dec(dec("100000.00")))
    );
    let Some(Json::Array(repayments)) = converted.get("repayments") else {
        panic!("repayments is a list")
    };
    assert_eq!(
        repayments[0].get("amount"),
        Some(&Json::Dec(dec("11000.00")))
    );
    // These are strings in the provider's schema and their digits are an
    // identity, not a quantity. A leading zero dropped is a different reference.
    assert_eq!(converted.get("reference"), Some(&Json::str("0012")));
    assert_eq!(converted.get("abn"), Some(&Json::str("51824753556")));
    assert_eq!(
        converted.get("loan_origination_date"),
        Some(&Json::str("2024-09-01"))
    );
    // The caller's value is untouched.
    assert_eq!(body.get("amalgamated_base"), Some(&Json::str("100000.00")));
}

#[test]
fn a_named_number_field_that_is_not_a_number_is_a_usage_error() {
    let message = cli::decimalise(
        Json::obj([("amalgamated_base", Json::str("not a number"))]),
        &fields(&["amalgamated_base"]),
    )
    .expect("no uncaught error")
    .unwrap_err();
    assert!(
        text(&message).contains("amalgamated_base"),
        "{}",
        text(&message)
    );
}

#[test]
fn an_absent_optional_number_field_is_not_an_error() {
    let converted = cli::decimalise(
        Json::Object(Vec::new()),
        &fields(&["amalgamated_base", "asset.cost"]),
    )
    .expect("no uncaught error")
    .expect("the body converts");
    assert_eq!(converted, Json::Object(Vec::new()));
}

#[test]
fn the_snapshot_names_the_repayment_amounts_as_numbers() {
    // The trial sends them as numbers, so a body read from a file must too.
    let contract = contract();
    let recorded = contract.calculators.get(DIV7A).expect("div7a is recorded");
    let Some(Json::Array(numbers)) = recorded.get("request_number_fields") else {
        panic!("request_number_fields is a list")
    };
    assert!(numbers.contains(&Json::str("repayments[].amount")));
}

// -- comparing two engines --------------------------------------------------

#[test]
fn exact_agreement_with_an_uncompared_field_is_scope_not_arithmetic() {
    let expected = money(&[("statutory_myr", "100.00"), ("deemed_dividend", "0.00")]);
    let local = money(&[("statutory_myr", "100.00")]);
    let upstream = money(&[("statutory_myr", "100.00")]);
    let (evaluation, differences, reasons) =
        compare_values(&expected, &local, &upstream, &cent()).unwrap();
    assert_eq!(evaluation, Evaluation::ScopeMismatch);
    assert!(differences.is_empty());
    assert!(!reasons.is_empty());
}

#[test]
fn a_figure_only_one_side_produced_is_named() {
    let local = money(&[("statutory_myr", "100.00"), ("interest_accrued", "7.00")]);
    let upstream = money(&[("statutory_myr", "100.00")]);
    let (evaluation, _differences, reasons) =
        compare_values(&Money::new(), &local, &upstream, &cent()).unwrap();
    assert_eq!(evaluation, Evaluation::ScopeMismatch);
    assert!(any_contains(&reasons, "interest_accrued"));
}

#[test]
fn full_agreement_on_every_field_is_still_a_match() {
    let values = money(&[("statutory_myr", "100.00")]);
    let (evaluation, differences, reasons) =
        compare_values(&values, &values.clone(), &values.clone(), &cent()).unwrap();
    assert_eq!(evaluation, Evaluation::Match);
    assert!(differences.is_empty() && reasons.is_empty());
}

// -- the FBT join -----------------------------------------------------------

#[test]
fn a_grossed_up_value_is_refused_by_its_value_not_only_by_its_name() {
    let error: Exc = fbt::prepare_aggregate_input(
        &dec("15094.40"),
        "Type 2",
        "taxable_value",
        &money(&[
            ("taxable_value", "8000.00"),
            ("grossed_up_taxable_value", "15094.40"),
        ]),
    )
    .unwrap_err();
    assert_eq!(error.class, CONTRACT);
    assert!(text(&error.text).contains("already grossed up"));
}

#[test]
fn the_category_value_itself_still_passes_that_check() {
    let (_type_one, type_two) = fbt::prepare_aggregate_input(
        &dec("8000.00"),
        "Type 2",
        "taxable_value",
        &money(&[
            ("taxable_value", "8000.00"),
            ("grossed_up_taxable_value", "15094.40"),
            ("fbt_payable", "7094.37"),
        ]),
    )
    .unwrap();
    assert_eq!(type_two, dec("8000.00"));
}

/// `_fbt_response(**overrides)`, as JSON text; `None` removes a field.
fn fbt_response(overrides: &[(&str, Option<&str>)]) -> String {
    let mut pairs: Vec<(&str, Json)> = vec![
        ("taxable_value", Json::str("8000.00")),
        ("grossed_up_taxable_value", Json::str("15094.40")),
        ("fbt_payable", Json::str("7094.37")),
        (
            "manifest",
            Json::obj([(
                "rate_uris_consumed",
                Json::Array(vec![Json::str("urn:sbrm:rate:fbt:type2:fy2026")]),
            )]),
        ),
        (
            "advisory",
            Json::obj([("disclaimer", Json::str("Calculated on supplied facts."))]),
        ),
    ];
    for (name, value) in overrides {
        match value {
            Some(value) => {
                pairs
                    .iter_mut()
                    .find(|(key, _)| key == name)
                    .expect("an override of a known field")
                    .1 = Json::str(value);
            }
            None => pairs.retain(|(key, _)| key != name),
        }
    }
    let object = Json::Object(
        pairs
            .into_iter()
            .map(|(k, v)| (PyStr::Text(k.to_owned()), v))
            .collect(),
    );
    pyjson::dumps(&object, Layout::Canonical)
}

fn benefit() -> fbt::CarBenefit {
    fbt::CarBenefit {
        case_id: "FBT-REG-1".into(),
        period_uri: fbt::PERIOD_URI.into(),
        base_value: dec("40000.00"),
        days_available: 365,
        accessories: dec("0.00"),
        employee_contribution: dec("0.00"),
        fbt_type: "Type 2".into(),
    }
}

fn fbt_evaluate(stub: &Stub) -> lodgeit_calculator_adapter::trials::Comparison {
    with_client(&stub.config(), &contract(), |client| {
        fbt::evaluate(&benefit(), Some(client)).expect("the trial evaluates")
    })
}

#[test]
fn the_fbt_trial_compares_the_payable_before_it_says_match() {
    let stub = Stub::start();
    stub.respond(&fbt_response(&[]));
    assert_eq!(fbt_evaluate(&stub).evaluation, Evaluation::Match);
}

#[test]
fn an_fbt_payable_that_differs_is_a_numeric_difference() {
    let stub = Stub::start();
    // The gross-up agrees to the cent; only the payable moves.
    stub.respond(&fbt_response(&[("fbt_payable", Some("6000.00"))]));
    let comparison = fbt_evaluate(&stub);
    assert_eq!(comparison.evaluation, Evaluation::NumericDifference);
    assert!(
        comparison
            .differences
            .iter()
            .any(|(name, _)| *name == pts("fbt_payable"))
    );
}

#[test]
fn an_fbt_response_with_nothing_to_compare_is_not_a_match() {
    let stub = Stub::start();
    stub.respond(&fbt_response(&[
        ("grossed_up_taxable_value", None),
        ("fbt_payable", None),
    ]));
    assert_eq!(fbt_evaluate(&stub).evaluation, Evaluation::ScopeMismatch);
}

#[test]
fn an_fbt_response_missing_only_the_payable_is_not_a_match() {
    let stub = Stub::start();
    stub.respond(&fbt_response(&[("fbt_payable", None)]));
    assert_eq!(fbt_evaluate(&stub).evaluation, Evaluation::ScopeMismatch);
}

// -- evidence records -------------------------------------------------------

/// `_record(stub, contract)`.
fn record(stub: &Stub, contract: &Contract) -> Json {
    stub.respond(DIV7A_RESPONSE);
    let outcome = invoke_div7a(stub, contract, &base_body());
    evidence::build(
        &outcome,
        "div7a-minimum-yearly-repayment",
        true,
        &Json::Null,
    )
    .expect("the record builds")
}

#[test]
fn a_digest_mismatch_names_the_digest_it_actually_computed() {
    let stub = Stub::start();
    let mut record = record(&stub, &contract());
    *member_mut(member_mut(&mut record, "calculation"), "label") = Json::str("tampered-label");
    let findings = evidence::verify(&record).unwrap();
    assert!(!findings.is_empty());
    // The message used to print the placeholder instead of the value.
    let first = text(&findings[0]);
    assert!(!first.contains("{actual}"), "{first}");
    assert!(
        first.contains(&evidence::digest_of(&record).unwrap()),
        "{first}"
    );
}

#[test]
fn verify_reports_a_null_block_rather_than_crashing() {
    let stub = Stub::start();
    let mut record = record(&stub, &contract());
    *member_mut(member_mut(&mut record, "calculation"), "call") = Json::Null;
    redigest(&mut record);
    let findings = evidence::verify(&record).unwrap();
    assert!(
        any_contains(&findings, "no call block"),
        "{:?}",
        texts(&findings)
    );
}

#[test]
fn the_command_line_verify_reports_a_null_block() {
    let tmp = TmpPath::new();
    let stub = Stub::start();
    let mut record = record(&stub, &contract());
    *member_mut(member_mut(&mut record, "calculation"), "call") = Json::Null;
    redigest(&mut record);
    let path = tmp.join("evidence.json");
    std::fs::write(&path, pyjson::dumps(&record, Layout::Canonical)).unwrap();
    let (code, _, _) = run_cli(&["verify", "--evidence", path_str(&path)]);
    assert_eq!(code, 1);
}

#[test]
fn a_label_the_consumer_would_refuse_is_refused_here() {
    // The monthly-close control plane keys a source digest on this label
    // and requires a slug.
    let contract = contract();
    let long = "d".repeat(121);
    for label in [
        "Div7A MYR",   // spaces and capitals
        "div7a_myr",   // an underscore
        "div7a--myr",  // a double hyphen
        "-div7a",      // a leading hyphen
        "div7a\0myr",  // a control character
        long.as_str(), // longer than the consumer accepts
    ] {
        let stub = Stub::start();
        stub.respond(r#"{"manifest": {}, "advisory": {}}"#);
        let outcome = discover(&stub, &contract);
        let error = evidence::build(&outcome, label, true, &Json::Null).unwrap_err();
        assert_eq!(error.class, "ValueError", "{label:?}");
        assert!(text(&error.text).contains("label"), "{label:?}");
    }
}

// -- what the fixtures claim ------------------------------------------------

#[test]
fn every_case_records_the_verdict_the_local_engine_actually_reaches() {
    for case in div7a::load_cases(None).unwrap() {
        let (verdict, _values, _reasons) = div7a::run_local(&case).unwrap();
        assert_eq!(verdict, case.expect_local, "{}", case.case_id);
    }
}

#[test]
fn the_command_line_writes_no_evidence_under_a_label_the_consumer_refuses() {
    let tmp = TmpPath::new();
    let stub = Stub::start();
    stub.respond(r#"[{"calculator_uri": "urn:sbrm:calculator:div7a:at"}]"#);
    let out = tmp.join("evidence.json");
    let (code, _, _) = run_cli(&[
        "invoke",
        "--enable-network",
        "--allow-loopback",
        "--base-url",
        &stub.base_url,
        "--calculator",
        DIV7A,
        "--period",
        FY2026,
        "--body",
        "fixtures/example-div7a-request.json",
        "--label",
        "Manual Invocation",
        "--evidence-out",
        path_str(&out),
    ]);
    assert_eq!(code, 2);
    assert!(!out.exists());
}

#[test]
fn the_default_label_is_one_the_consumer_accepts() {
    let tmp = TmpPath::new();
    let stub = Stub::start();
    stub.respond(DIV7A_RESPONSE);
    let out = tmp.join("evidence.json");
    let (code, _, stderr) = run_cli(&[
        "invoke",
        "--enable-network",
        "--allow-loopback",
        "--base-url",
        &stub.base_url,
        "--calculator",
        DIV7A,
        "--period",
        FY2026,
        "--body",
        "fixtures/example-div7a-request.json",
        "--evidence-out",
        path_str(&out),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let written = std::fs::read_to_string(&out).unwrap();
    let record = pyjson::loads_standard(&written).unwrap();
    assert_eq!(
        record.at("calculation").get("label"),
        Some(&Json::str("manual-invocation"))
    );
}

#[test]
fn a_repayment_amount_read_from_a_file_goes_on_the_wire_as_a_number() {
    let stub = Stub::start();
    stub.respond(DIV7A_RESPONSE);
    let (code, _, stderr) = run_cli(&[
        "invoke",
        "--enable-network",
        "--allow-loopback",
        "--base-url",
        &stub.base_url,
        "--calculator",
        DIV7A,
        "--period",
        FY2026,
        "--body",
        "fixtures/example-div7a-request.json",
    ]);
    assert_eq!(code, 0, "{stderr}");
    let sent = stub.sent();
    assert!(sent.contains(r#""amalgamated_base":100000.00"#), "{sent}");
    assert!(sent.contains(r#""amount":11000.00"#), "{sent}");
    // The date beside it is a string and stays one.
    assert!(sent.contains(r#""date":"2025-12-15""#), "{sent}");
}

// -- a second review pass ---------------------------------------------------

#[test]
fn a_json_number_keeps_its_own_digits_on_the_way_in() {
    let tmp = TmpPath::new();
    let stub = Stub::start();
    stub.respond(DIV7A_RESPONSE);
    let body = tmp.join("body.json");
    // 12345678901.234567 is inside the serialiser's scale limit and comes
    // back from a float as ...568.
    std::fs::write(
        &body,
        r#"{"amalgamated_base": 12345678901.234567, "loan_term_years": 7, "loan_origination_date": "2024-09-01", "income_year_start_date": "2025-07-01", "is_first_real_myr_year": true, "repayments": []}"#,
    )
    .unwrap();
    let (code, _, stderr) = run_cli(&[
        "invoke",
        "--enable-network",
        "--allow-loopback",
        "--base-url",
        &stub.base_url,
        "--calculator",
        DIV7A,
        "--period",
        FY2026,
        "--body",
        path_str(&body),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let sent = stub.sent();
    assert!(
        sent.contains(r#""amalgamated_base":12345678901.234567"#),
        "{sent}"
    );
    assert!(!sent.contains("12345678901.234568"), "{sent}");
    // An integer stays an integer, not a decimal-pointed float.
    assert!(sent.contains(r#""loan_term_years":7"#), "{sent}");
}

#[test]
fn a_number_too_precise_to_send_is_refused_rather_than_rounded() {
    let tmp = TmpPath::new();
    let stub = Stub::start();
    let body = tmp.join("toofine.json");
    std::fs::write(
        &body,
        r#"{"amalgamated_base": 0.10000000000000001, "loan_term_years": 7, "loan_origination_date": "2024-09-01", "income_year_start_date": "2025-07-01", "is_first_real_myr_year": true, "repayments": []}"#,
    )
    .unwrap();
    let (code, _, _) = run_cli(&[
        "invoke",
        "--enable-network",
        "--allow-loopback",
        "--base-url",
        &stub.base_url,
        "--calculator",
        DIV7A,
        "--period",
        FY2026,
        "--body",
        path_str(&body),
    ]);
    assert_eq!(code, 1);
    assert!(stub.requests().is_empty());
}

/// Python's `cli.load_body` is `json.loads(text, parse_float=Decimal)`; the
/// Rust CLI reads the body with `loads_numbers(text, Numbers::DecimalFloats)`.
#[test]
fn the_body_reader_keeps_an_integer_an_integer() {
    let parsed = pyjson::loads_numbers(
        r#"{"a": 7, "b": 1.50, "c": "0012"}"#,
        Numbers::DecimalFloats,
    )
    .unwrap();
    assert_eq!(parsed.get("a"), Some(&Json::Int(7.into())));
    let Some(Json::Dec(b)) = parsed.get("b") else {
        panic!("b is a Decimal")
    };
    assert_eq!(b.to_sci_string(), "1.50");
    assert_eq!(parsed.get("c"), Some(&Json::str("0012")));
}

fn line(description: &str, topology: &str, amount: &str) -> fano::Line {
    fano::Line {
        description: description.into(),
        source_topology: topology.into(),
        amount: dec(amount),
        predicted_code: "sbrm_0000".into(),
        confidence: dec("0"),
    }
}

#[test]
fn a_result_row_that_is_not_an_object_is_a_contract_failure() {
    let stub = Stub::start();
    let lines = [
        line("Bank account", "current_assets", "15000.00"),
        line("Sales", "revenue", "-15000.00"),
    ];
    stub.respond(r#"{"status": "COMPLETE", "equilibrium_valid": true, "results": [{"description": "Bank account", "predicted_code": "sbrm_1234"}, "not an object"]}"#);
    let (status, suggestions, notes) = with_client(&stub.config(), &fano_contract(), |client| {
        fano::classify("company", &lines, Some(client)).expect("classify returns")
    });
    assert_eq!(status, "CONTRACT_FAILURE");
    assert!(suggestions.is_empty());
    assert!(any_contains(&notes, "position 1"), "{:?}", texts(&notes));
}

// -- the final review pass --------------------------------------------------

#[test]
fn a_discovery_body_that_is_not_a_list_is_not_agreement() {
    let stub = Stub::start();
    stub.respond(r#"{"calculators": [{"calc_uri": "brand-new"}]}"#);
    let (outcome, findings) = drift(&stub, &contract());
    assert!(outcome.computed());
    assert!(!findings.is_empty());
    assert!(text(&findings[0]).contains("nothing was compared"));
}

#[test]
fn a_listing_entry_that_is_not_an_object_is_reported_not_a_crash() {
    let stub = Stub::start();
    stub.respond(
        r#"["urn:sbrm:calculator:div7a:at", {"calc_uri": "urn:sbrm:calculator:div7a:at"}]"#,
    );
    let (_outcome, findings) = drift(&stub, &contract());
    assert!(
        any_contains(&findings, "entry 0 is str"),
        "{:?}",
        texts(&findings)
    );
}

#[test]
fn verify_reports_an_evidence_file_that_is_not_an_object() {
    assert_eq!(
        texts(&evidence::verify(&Json::Array(Vec::new())).unwrap()),
        ["the evidence file is list, not a JSON object"]
    );
    assert_eq!(
        texts(&evidence::verify(&Json::Null).unwrap()),
        ["the evidence file is NoneType, not a JSON object"]
    );
}

#[test]
fn verify_accepts_a_discovery_record_the_tool_wrote() {
    // A discovery listing has no manifest and no advisory, and no contract
    // asks for one.
    let stub = Stub::start();
    stub.respond(r#"[{"calc_uri": "urn:sbrm:calculator:div7a:at"}]"#);
    let contract = contract();
    let outcome = discover(&stub, &contract);
    let record = evidence::build(&outcome, "discover", true, &contract.response_contract).unwrap();
    assert!(evidence::verify(&record).unwrap().is_empty());
}

#[test]
fn verify_honours_a_contract_that_requires_no_manifest() {
    let stub = Stub::start();
    stub.respond(r#"{"status": "COMPLETE", "equilibrium_valid": true, "results": [{"description": "Bank account", "predicted_code": "sbrm_1234"}]}"#);
    let contract = fano_contract();
    let outcome = with_client(&stub.config(), &contract, |client| {
        client
            .invoke(
                &pts(fano::ROUTE_CALC_URI),
                &pts(fano::PERIOD_URI),
                &Json::obj([
                    ("entity_structure", Json::str("company")),
                    ("lines", Json::Array(Vec::new())),
                ]),
            )
            .unwrap()
    });
    assert!(outcome.computed(), "{:?}", texts(&outcome.findings));
    let record =
        evidence::build(&outcome, "fano-trial", true, &contract.response_contract).unwrap();
    assert!(evidence::verify(&record).unwrap().is_empty());
}

#[test]
fn verify_still_requires_a_manifest_where_the_contract_does() {
    let stub = Stub::start();
    stub.respond(DIV7A_RESPONSE);
    let contract = contract();
    let outcome = invoke_div7a(&stub, &contract, &base_body());
    let mut record = evidence::build(&outcome, "div7a", true, &contract.response_contract).unwrap();
    assert!(evidence::verify(&record).unwrap().is_empty());
    *member_mut(
        member_mut(member_mut(&mut record, "calculation"), "upstream"),
        "manifest",
    ) = Json::Null;
    redigest(&mut record);
    let findings = evidence::verify(&record).unwrap();
    assert!(
        any_contains(&findings, "no upstream manifest"),
        "{:?}",
        texts(&findings)
    );
}

#[test]
fn a_port_that_is_not_a_number_is_a_refusal_not_a_traceback() {
    let error = enabled()
        .check_url(&format!("{HOST}:notaport/v1/calculators"), true)
        .unwrap_err();
    assert_eq!(error.class, DISALLOWED_TARGET);
    assert!(text(&error.text).contains("port is not a number"));
}

#[test]
fn a_listing_entry_without_a_calc_uri_is_a_finding_not_agreement() {
    // Every recorded calculator present, plus one malformed object.
    let stub = Stub::start();
    let contract = contract();
    let Json::Object(calculators) = &contract.calculators else {
        panic!("calculators is an object")
    };
    let mut listing: Vec<Json> = calculators
        .iter()
        .map(|(uri, recorded)| {
            Json::obj([
                ("calc_uri", Json::Str(uri.clone())),
                (
                    "supported_periods",
                    recorded
                        .get("supported_periods")
                        .cloned()
                        .unwrap_or(Json::Array(Vec::new())),
                ),
                (
                    "input_schema_ref",
                    recorded
                        .get("input_schema_ref")
                        .cloned()
                        .unwrap_or(Json::Null),
                ),
            ])
        })
        .collect();
    listing.push(Json::obj([("label", Json::str("no uri here"))]));
    stub.respond(&pyjson::dumps(&Json::Array(listing), Layout::Canonical));
    let (_outcome, findings) = drift(&stub, &contract);
    assert!(
        any_contains(&findings, "carries no calc_uri string"),
        "{:?}",
        texts(&findings)
    );
}

#[test]
fn malformed_supported_periods_are_a_finding_not_a_crash() {
    // A JSON integer arrives as a string, because the client reads numbers
    // as text, so the non-string case here is an object entry.
    let contract = contract();
    for periods in [
        "null",
        r#""fy2026""#,
        r#"[["nested"]]"#,
        r#"[{"period": 1}]"#,
    ] {
        let stub = Stub::start();
        stub.respond(&format!(
            r#"[{{"calc_uri": "urn:sbrm:calculator:div7a:at", "supported_periods": {periods}}}]"#
        ));
        let (_outcome, findings) = drift(&stub, &contract);
        let wanted = if periods == "null" {
            "no longer accepts"
        } else {
            "not a list of strings"
        };
        assert!(
            any_contains(&findings, wanted),
            "{periods}: {:?}",
            texts(&findings)
        );
    }
}

#[test]
fn a_figure_only_the_provider_returned_is_a_scope_difference() {
    let (evaluation, differences, reasons) = compare_values(
        &Money::new(),
        &Money::new(),
        &money(&[("shortfall", "1.00")]),
        &cent(),
    )
    .unwrap();
    assert_eq!(evaluation, Evaluation::ScopeMismatch);
    assert!(differences.is_empty());
    assert_eq!(
        texts(&reasons),
        [
            "shortfall: the provider returned a figure and the local engine did not",
            "no field is present on both sides, so nothing was compared",
        ]
    );
}
