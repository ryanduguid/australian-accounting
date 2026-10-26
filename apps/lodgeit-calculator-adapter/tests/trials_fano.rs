//! `tests/test_trials_fano.py`: the Fano trial, where suggestions stay
//! suggestions.
//!
//! The classifier is a loopback HTTP stub started inside each test and
//! reached through an `allow_loopback` config with the `fano-classifier`
//! snapshot, as the Python `fano_client` fixture does.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use lodgeit_calculator_adapter::client::Client;
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract;
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::errors::CONTRACT;
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::pyvalue::points_lossy;
use lodgeit_calculator_adapter::trials::fano::{self as trial, Interpreted, Line};
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

/// The `fano_client` fixture: `LodgeitClient(config, fano_contract)` with
/// `enabled=True`, the stub's base URL, `allow_loopback=True` and
/// `max_attempts=1`, over the real dialler.
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
    let contract = contract::load("fano-classifier", None).expect("the snapshot loads");
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

fn dec(text: &str) -> Decimal {
    Decimal::parse_exact(text).expect("a literal")
}

/// `trial.Line(description, topology, amount)` with the defaults
/// `predicted_code="sbrm_0000"` and `confidence=Decimal("0")`.
fn line(description: &str, topology: &str, amount: &str) -> Line {
    Line {
        description: description.to_owned(),
        source_topology: topology.to_owned(),
        amount: dec(amount),
        predicted_code: "sbrm_0000".to_owned(),
        confidence: dec("0"),
    }
}

/// `LINES`.
fn lines() -> Vec<Line> {
    vec![
        line("Bank account, operating", "current_assets", "15000.00"),
        line("Trade debtors", "current_assets", "42000.00"),
        line("Sales, maintenance services", "revenue", "-57000.00"),
    ]
}

fn texts(notes: &[Vec<u32>]) -> Vec<String> {
    notes.iter().map(|n| points_lossy(n)).collect()
}

fn mentions(notes: &[Vec<u32>], needle: &str) -> bool {
    texts(notes).iter().any(|n| n.contains(needle))
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

/// `body["results"][index][key] = value`.
fn set_row(body: &mut Json, index: usize, key: &str, value: Json) {
    let Json::Object(pairs) = body else {
        panic!("an object")
    };
    let results = pairs
        .iter_mut()
        .find(|(k, _)| k.as_str() == Some("results"))
        .map(|(_, v)| v)
        .expect("results");
    let Json::Array(rows) = results else {
        panic!("a list")
    };
    set(&mut rows[index], key, value);
}

/// `response_for(lines, **overrides)`.
fn response_for(lines: &[Line], overrides: &[(&str, Json)]) -> Json {
    let rows = lines
        .iter()
        .map(|line| {
            Json::obj([
                ("description", Json::str(&line.description)),
                ("predicted_code", Json::str("sbrm_1234")),
                ("confidence", Json::str("0.91")),
                ("cascade_topology", Json::str(&line.source_topology)),
                ("model_architecture", Json::str("iter11.B_R3")),
                (
                    "operator_hint_predicted_code",
                    Json::str(&line.predicted_code),
                ),
                (
                    "operator_hint_source_topology",
                    Json::str(&line.source_topology),
                ),
                (
                    "operator_hint_confidence",
                    Json::str(&line.confidence.fixed_format()),
                ),
                ("fano_status", Json::str("accepted_fact")),
                ("quarantine_reason", Json::str("keyword_rule:sbrm_1234")),
            ])
        })
        .collect();
    let mut body = Json::obj([
        ("status", Json::str("COMPLETE")),
        ("equilibrium_valid", Json::Bool(true)),
        ("results", Json::Array(rows)),
    ]);
    for (name, value) in overrides {
        set(&mut body, name, value.clone());
    }
    body
}

/// `trial.classify("company", LINES, client)` against the stub's reply.
fn classify(body: &Json, status: u16) -> Interpreted {
    let stub = Stub::start();
    stub.respond(body, status);
    let interpreted = with_client(&stub, |client| {
        trial::classify("company", &lines(), Some(client))
    })
    .expect("an interpretation");
    // The stub saw one POST to the snapshot's route.
    assert_eq!(stub.request_count(), 1);
    let state = stub.state.lock().unwrap();
    assert_eq!(state.requests[0].method, "POST");
    assert!(!state.requests[0].path.is_empty());
    interpreted
}

// ---------------------------------------------------------------------------
// The tests, one per Python test function.

#[test]
fn a_trial_balance_that_does_not_balance_is_refused_locally() {
    let unbalanced = &lines()[..2];
    let error = trial::build_payload("company", unbalanced).expect_err("refused");
    assert_eq!(error.class, CONTRACT);
    assert!(points_lossy(&error.text).contains("net to"));
}

#[test]
fn the_provider_limits_are_checked_before_sending() {
    let many = vec![lines()[0].clone(); 501];
    for (entity, given, needle) in [
        ("company", many, "500"),
        ("llc", lines(), "entity_structure"),
        ("company", Vec::new(), "no lines"),
    ] {
        let error = trial::build_payload(entity, &given).expect_err("refused");
        assert_eq!(error.class, CONTRACT, "{needle}");
        assert!(points_lossy(&error.text).contains(needle), "{needle}");
    }
}

#[test]
fn every_line_keeps_its_original_alongside_the_suggestion() {
    let (status, suggestions, notes) = classify(&response_for(&lines(), &[]), 200);
    // This trial's own vocabulary, never the provider's string. Its status
    // is recorded as a note so a reader can see it without it being read as
    // ours.
    assert_eq!(status, "INTERPRETED");
    assert!(mentions(&notes, "'COMPLETE'"));
    assert_eq!(suggestions.len(), 3);
    let first = suggestions[0].to_json();
    assert_eq!(
        first.at("original").at("description"),
        &Json::str("Bank account, operating")
    );
    assert_eq!(
        first.at("original").at("topology"),
        &Json::str("current_assets")
    );
    assert_eq!(first.at("original").at("amount"), &Json::str("15000.00"));
    assert_eq!(
        first.at("suggestion").at("proposed_code"),
        &Json::str("sbrm_1234")
    );
    assert_eq!(
        first.at("suggestion").at("classifier"),
        &Json::str("iter11.B_R3")
    );
    assert_eq!(first.at("review").at("applied"), &Json::Bool(false));
    assert_eq!(first.at("review").at("approved_by"), &Json::Null);
}

#[test]
fn accepted_fact_is_not_an_approval() {
    let (_, suggestions, _) = classify(&response_for(&lines(), &[]), 200);
    for suggestion in &suggestions {
        assert_eq!(suggestion.fano_status, Json::str("accepted_fact"));
        assert!(suggestion.needs_review());
        let payload = suggestion.to_json();
        assert_eq!(payload.at("review").at("approved_by"), &Json::Null);
        assert!(
            payload
                .at("review")
                .at("boundary")
                .as_str()
                .expect("a str")
                .contains("not an approval")
        );
    }
}

#[test]
fn a_draft_fact_is_a_correct_outcome_and_is_carried_through() {
    let mut body = response_for(&lines(), &[]);
    set_row(&mut body, 1, "fano_status", Json::str("draft_fact"));
    set_row(
        &mut body,
        1,
        "quarantine_reason",
        Json::str("rule_model_disagreement (rule=sbrm_1200, model=sbrm_1234)"),
    );
    let (_, suggestions, _) = classify(&body, 200);
    assert_eq!(suggestions[1].fano_status, Json::str("draft_fact"));
    assert!(
        suggestions[1]
            .reason
            .as_str()
            .expect("a str")
            .contains("rule_model_disagreement")
    );
}

#[test]
fn a_topology_disagreement_is_flagged() {
    let mut body = response_for(&lines(), &[]);
    set_row(
        &mut body,
        0,
        "cascade_topology",
        Json::str("non_current_assets"),
    );
    let (_, suggestions, _) = classify(&body, 200);
    assert!(suggestions[0].topology_disagrees());
    assert_eq!(
        suggestions[0]
            .to_json()
            .at("review")
            .at("topology_disagreement"),
        &Json::Bool(true)
    );
}

#[test]
fn an_unknown_status_leaves_the_line_unreviewed() {
    let mut body = response_for(&lines(), &[]);
    set_row(&mut body, 2, "fano_status", Json::str("brand_new_state"));
    let (_, suggestions, _) = classify(&body, 200);
    assert!(mentions(&suggestions[2].findings, "unknown fano_status"));
}

#[test]
fn a_row_count_mismatch_proposes_nothing() {
    let mut body = response_for(&lines(), &[]);
    let Json::Array(rows) = body.at("results").clone() else {
        panic!("a list")
    };
    set(&mut body, "results", Json::Array(rows[..2].to_vec()));
    let (status, suggestions, _) = classify(&body, 200);
    assert_eq!(status, "CONTRACT_FAILURE");
    assert!(suggestions.is_empty());
}

#[test]
fn a_reported_imbalance_proposes_nothing() {
    let body = response_for(&lines(), &[("equilibrium_valid", Json::Bool(false))]);
    let (status, suggestions, _) = classify(&body, 200);
    assert_eq!(status, "CONTRACT_FAILURE");
    assert!(suggestions.is_empty());
}

#[test]
fn an_unavailable_service_proposes_nothing() {
    let (status, suggestions, _) = classify(&Json::obj([("detail", Json::str("down"))]), 503);
    assert_eq!(status, "UPSTREAM_UNAVAILABLE");
    assert!(suggestions.is_empty());
}

#[test]
fn an_authentication_refusal_is_reported_not_worked_around() {
    let body = Json::obj([("detail", Json::str("Not authenticated"))]);
    let (status, suggestions, notes) = classify(&body, 403);
    // The contract snapshot records that a 401 or 403 is the provider
    // refusing to answer, not a contract failure.
    assert_eq!(status, "UPSTREAM_REFUSED");
    assert!(suggestions.is_empty());
    assert!(mentions(&notes, "declined to answer"));
}

#[test]
fn a_code_needs_a_reviewed_crosswalk_before_it_means_anything() {
    let error = trial::require_crosswalk(None, "sbrm_1234").expect_err("refused");
    assert_eq!(error.class, CONTRACT);
    assert!(points_lossy(&error.text).contains("no reviewed crosswalk"));
    let other = [("sbrm_9999".to_owned(), "1-1100".to_owned())];
    let error = trial::require_crosswalk(Some(&other), "sbrm_1234").expect_err("refused");
    assert_eq!(error.class, CONTRACT);
    assert!(points_lossy(&error.text).contains("no entry in the reviewed crosswalk"));
    let crosswalk = [("sbrm_1234".to_owned(), "1-1100".to_owned())];
    assert_eq!(
        trial::require_crosswalk(Some(&crosswalk), "sbrm_1234").expect("a mapping"),
        "1-1100"
    );
}

#[test]
fn the_suggestion_record_never_carries_an_applied_mapping() {
    let (_, suggestions, notes) = classify(&response_for(&lines(), &[]), 200);
    let records = Json::Array(suggestions.iter().map(|s| s.to_json()).collect());
    // `json.dumps` separates a key from its value with ": ", as the
    // indented layout does.
    let serialised = pyjson::dumps(&records, Layout::Indented);
    assert!(serialised.contains("\"applied\": false"));
    assert!(!serialised.contains("\"applied\": true"));
    assert!(mentions(&notes, "awaiting review"));
}

#[test]
fn a_line_with_an_unknown_topology_is_refused_before_sending() {
    let error = line("Bank account", "assets", "1.00")
        .payload()
        .expect_err("refused");
    assert_eq!(error.class, CONTRACT);
    assert!(points_lossy(&error.text).contains("'assets' is not one of"));
}

#[test]
fn without_a_client_the_classifier_is_not_called() {
    let (status, suggestions, notes) =
        trial::classify("company", &lines(), None).expect("an interpretation");
    assert_eq!((status.as_str(), suggestions.len()), ("NOT_RUN", 0));
    assert_eq!(
        texts(&notes),
        vec!["No client supplied; the classifier was not called.".to_owned()]
    );
}

#[test]
fn a_response_without_a_results_array_proposes_nothing() {
    let body = response_for(
        &lines(),
        &[("results", Json::obj([("rows", Json::Array(Vec::new()))]))],
    );
    let (status, suggestions, notes) = classify(&body, 200);
    assert_eq!(
        (status.as_str(), suggestions.len()),
        ("CONTRACT_FAILURE", 0)
    );
    assert_eq!(
        texts(&notes),
        vec!["the response carries no results array".to_owned()]
    );
}

#[test]
fn an_echoed_description_that_differs_is_flagged() {
    let mut body = response_for(&lines(), &[]);
    set_row(&mut body, 0, "description", Json::str("Something else"));
    let (_, suggestions, _) = classify(&body, 200);
    assert!(mentions(
        &suggestions[0].findings,
        "rows may be out of order"
    ));
}
