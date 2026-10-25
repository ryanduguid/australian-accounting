//! `tests/test_cli_paths.py`: command-line paths the regression suites do
//! not reach. All offline; the network commands talk to a loopback stub
//! this file starts. The MCP tests wait for gate D.
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lodgeit_calculator_adapter::cli::{self, Io};
use lodgeit_calculator_adapter::client::{self, Outcome, Status};
use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::Net;
use lodgeit_calculator_adapter::evidence;
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::pypath::PyPath;

const CALC: &str = "urn:sbrm:calculator:div7a:at";
const PERIOD: &str = "urn:sbrm:period:div7a:fy2026";

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn json(text: &str) -> Json {
    pyjson::loads_standard(text).expect("valid JSON")
}

fn the_contract() -> Contract {
    contract::load("lodgeit-calculators", None).expect("the reviewed snapshot loads")
}

/// A fresh, empty folder for one test.
fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lodgeit-cli-paths-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    dir
}

fn path_text(path: &std::path::Path) -> String {
    path.to_str().expect("a UTF-8 temp path").to_owned()
}

/// `cli.main(argv)` with captured streams and no `LODGEIT_ADAPTER_*`
/// variables, as the autouse fixture leaves the environment. Retry backoff
/// does not sleep. Captured output reads `\n` for the `\r\n` Windows
/// text-mode stdout writes, as `capsys` does.
fn run(args: &[&str]) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let env = |_: &str| -> Option<OsString> { None };
    let mut sleep = |_: f64| -> Result<(), Exc> { Ok(()) };
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = cli::run_io(
        &args,
        &mut stdout,
        &mut stderr,
        &mut Io {
            env: &env,
            dial: &mut Net,
            sleep: &mut sleep,
        },
    );
    let decode = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .expect("UTF-8 output")
            .replace("\r\n", "\n")
    };
    (code, decode(stdout), decode(stderr))
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

fn network(base_url: &str) -> [&str; 4] {
    [
        "--enable-network",
        "--allow-loopback",
        "--base-url",
        base_url,
    ]
}

// -- the loopback stub: conftest.py's `stub` fixture ------------------------

struct Seen {
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
    /// An HTTP server on 127.0.0.1 that answers every request with the
    /// configured status and body and records what it saw.
    fn start() -> Stub {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a loopback port");
        let port = listener.local_addr().expect("a local address").port();
        let state = Arc::new(Mutex::new(StubState {
            status: 200,
            body: b"{}".to_vec(),
            requests: Vec::new(),
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

    fn paths(&self) -> Vec<String> {
        let state = self.state.lock().expect("stub state");
        state.requests.iter().map(|r| r.path.clone()).collect()
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
    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned();
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
        state.requests.push(Seen { path });
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

// -- the request body -------------------------------------------------------

fn decimalised(body: Json, fields: &str) -> Result<Json, String> {
    cli::decimalise(body, &json(fields))
        .expect("no exception")
        .map_err(|message| text(&message))
}

#[test]
fn a_body_that_is_not_an_object_is_refused() {
    assert_eq!(
        decimalised(json("[1]"), r#"["amalgamated_base"]"#),
        Err("a request body must be a JSON object".to_owned())
    );
}

#[test]
fn a_list_path_over_something_that_is_not_a_list_is_left_alone() {
    let body = json(r#"{"amalgamated_base": "1.00", "repayments": "none"}"#);
    assert_eq!(
        decimalised(body.clone(), r#"["repayments[].amount"]"#),
        Ok(body)
    );
}

#[test]
fn a_dotted_path_reaches_a_nested_field() {
    let converted = decimalised(
        json(r#"{"loan": {"amount": "1.50"}}"#),
        r#"["loan.amount"]"#,
    );
    let expected = Json::obj([(
        "loan",
        Json::obj([(
            "amount",
            Json::Dec(Decimal::parse_exact("1.50").expect("a decimal literal")),
        )]),
    )]);
    assert_eq!(converted, Ok(expected));
}

#[test]
fn a_boolean_or_null_where_a_number_belongs_is_refused() {
    for (value, shown) in [("true", "True"), ("null", "None")] {
        let body = json(&format!(r#"{{"amalgamated_base": {value}}}"#));
        assert_eq!(
            decimalised(body, r#"["amalgamated_base"]"#),
            Err(format!("amalgamated_base: {shown} is not a number"))
        );
    }
}

// -- the offline commands ---------------------------------------------------

#[test]
fn an_unknown_contract_name_is_a_refusal() {
    let (code, _, stderr) = run(&["--contract", "no-such-snapshot", "contract"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("lodgeit-adapter: contract snapshot "),
        "{stderr}"
    );
}

#[test]
fn the_contract_command_prints_the_reviewed_snapshot() {
    let contract = the_contract();
    let (code, stdout, _) = run(&["contract"]);
    assert_eq!(code, 0);
    let printed = json(&stdout);
    assert_eq!(printed.get("snapshot_id"), Some(&contract.snapshot_id));
    let Json::Object(calculators) = &contract.calculators else {
        panic!("calculators is an object");
    };
    let mut names: Vec<String> = calculators
        .iter()
        .map(|(k, _)| text(&k.code_points()))
        .collect();
    names.sort();
    let printed_names: Vec<String> = match printed.get("calculators") {
        Some(Json::Array(items)) => items
            .iter()
            .map(|item| item.as_str().expect("a calculator name").to_owned())
            .collect(),
        other => panic!("calculators is a list, not {other:?}"),
    };
    assert_eq!(printed_names, names);
}

#[test]
fn verify_names_an_evidence_file_it_cannot_read() {
    let dir = tmp_dir("verify-missing");
    let missing = path_text(&dir.join("missing.json"));
    let (code, _, stderr) = run(&["verify", "--evidence", &missing]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with(&format!("lodgeit-adapter: {missing} could not be read: ")),
        "{stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn computed_outcome() -> Outcome {
    Outcome {
        status: Status::Computed,
        calculator: None,
        period: None,
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

#[test]
fn verify_accepts_a_record_that_hangs_together() {
    let dir = tmp_dir("verify-good");
    let record = evidence::build(&computed_outcome(), "fabricated", true, &Json::Null)
        .expect("a record builds");
    let path = path_text(&dir.join("evidence.json"));
    evidence::write(&record, &PyPath::new(&path)).expect("the record is written");
    let (code, stdout, _) = run(&["verify", "--evidence", &path]);
    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        format!("evidence: {path} verifies against its own digest\n")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// -- the network commands, against the loopback stub ------------------------

#[test]
fn discover_prints_the_live_listing() {
    let contract = the_contract();
    let stub = Stub::start();
    stub.respond(&Json::Array(live_from(&contract)), 200);
    let mut args = vec!["discover"];
    args.extend(network(&stub.base_url));
    let (code, stdout, _) = run(&args);
    assert_eq!(code, 0);
    let printed = json(&stdout);
    assert_eq!(
        printed.get("status").and_then(Json::as_str),
        Some(Status::Computed.as_str())
    );
    assert_eq!(
        stub.paths().last().map(String::as_str),
        Some("/v1/calculators")
    );
}

#[test]
fn drift_that_agrees_says_so_and_exits_zero() {
    let contract = the_contract();
    let stub = Stub::start();
    stub.respond(&Json::Array(live_from(&contract)), 200);
    let mut args = vec!["drift"];
    args.extend(network(&stub.base_url));
    let (code, stdout, _) = run(&args);
    assert_eq!(code, 0);
    let expected = format!(
        "drift: live discovery agrees with snapshot {}",
        text(&contract.snapshot())
    );
    assert!(stdout.contains(&expected), "{stdout}");
    assert!(
        stdout.contains("Nothing here updates a snapshot."),
        "{stdout}"
    );
}

#[test]
fn drift_that_differs_lists_each_finding_and_exits_one() {
    let contract = the_contract();
    let stub = Stub::start();
    stub.respond(&Json::Array(live_from(&contract)[1..].to_vec()), 200);
    let mut args = vec!["drift"];
    args.extend(network(&stub.base_url));
    let (code, stdout, _) = run(&args);
    assert_eq!(code, 1);
    assert!(stdout.contains("which live no longer lists"), "{stdout}");
}

#[test]
fn drift_that_cannot_reach_the_service_reports_the_outcome() {
    let stub = Stub::start();
    stub.respond(&json(r#"{"detail": "down"}"#), 503);
    let mut args = vec!["drift"];
    args.extend(network(&stub.base_url));
    let (code, stdout, _) = run(&args);
    assert_eq!(code, 1);
    let printed = json(&stdout);
    assert_eq!(
        printed.get("status").and_then(Json::as_str),
        Some(Status::UpstreamUnavailable.as_str())
    );
}

#[test]
fn invoke_names_a_body_file_it_cannot_read() {
    let stub = Stub::start();
    let dir = tmp_dir("invoke-missing");
    let missing = path_text(&dir.join("missing.json"));
    let mut args = vec!["invoke"];
    args.extend(network(&stub.base_url));
    args.extend(["--calculator", CALC, "--period", PERIOD, "--body", &missing]);
    let (code, _, stderr) = run(&args);
    assert_eq!(code, 2);
    assert!(
        stderr.starts_with(&format!("lodgeit-adapter: {missing} could not be read: ")),
        "{stderr}"
    );
    assert!(stub.paths().is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
