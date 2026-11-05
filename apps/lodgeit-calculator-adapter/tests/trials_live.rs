// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The manual pass over the trial cases, against a loopback stub: one
//! request a case, a pause between requests, and a stop at the first answer
//! that is not `COMPUTED`. Nothing here leaves the loopback interface.
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use lodgeit_calculator_adapter::client::Client;
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::trials::{Evaluation, div7a, fbt, live};
use lodgeit_calculator_adapter::wire;
use num_bigint::BigInt;

const COMMITTED: &str = include_str!("../docs/live-trials-2026-11-06.json");

/// What the stub saw: each request's target and body in arrival order, and, if
/// a test named a file to watch, what that file held when the last one arrived.
#[derive(Default)]
struct Seen {
    requests: Mutex<Vec<(String, Vec<u8>)>>,
    watch: Mutex<Option<PathBuf>>,
    on_disk: Mutex<Option<String>>,
}

struct Stub {
    base_url: String,
    seen: Arc<Seen>,
}

impl Stub {
    fn requests(&self) -> usize {
        self.seen.requests.lock().unwrap().len()
    }
}

/// Answers every request with `status` and `body`, counting them.
fn stub(status: u16, body: &Json) -> Stub {
    start(
        status,
        Some(pyjson::dumps(body, Layout::Canonical).into_bytes()),
    )
}

/// As `stub`, with raw bytes, or closing the connection unanswered for `None`.
fn start(status: u16, reply: Option<Vec<u8>>) -> Stub {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port = listener.local_addr().expect("a bound address").port();
    let seen = Arc::new(Seen::default());
    let shared = Arc::clone(&seen);
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            serve(stream, status, reply.as_deref(), &shared);
        }
    });
    Stub {
        base_url: format!("http://127.0.0.1:{port}"),
        seen,
    }
}

fn serve(stream: TcpStream, status: u16, reply: Option<&[u8]>, seen: &Seen) {
    let mut reader = BufReader::new(stream.try_clone().expect("a second handle"));
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
        return;
    }
    let target = request_line.split_whitespace().nth(1).unwrap_or_default();
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
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
    seen.requests
        .lock()
        .unwrap()
        .push((target.to_owned(), body));
    if let Some(path) = seen.watch.lock().unwrap().as_ref() {
        *seen.on_disk.lock().unwrap() = fs::read_to_string(path).ok();
    }
    let Some(reply) = reply else {
        return;
    };
    let mut stream = stream;
    let head = format!(
        "HTTP/1.0 {status} Stub\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        reply.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(reply);
    let _ = stream.flush();
}

fn answer() -> Json {
    Json::obj([
        ("benchmark_rate", Json::str("0.0837")),
        ("statutory_myr", Json::str("19450.97")),
        ("total_repayments", Json::str("22000.00")),
        ("shortfall", Json::str("0.00")),
        ("taxable_value", Json::str("8000.00")),
        ("gross_up_factor", Json::str("1.8868")),
        ("grossed_up_taxable_value", Json::str("15094.40")),
        ("fbt_payable", Json::str("7094.37")),
        (
            "manifest",
            Json::obj([
                ("calculator", Json::str("stub")),
                ("period", Json::str("stub")),
                ("rate_table_uris", Json::Array(Vec::new())),
                ("engine", Json::str("stub")),
                ("citation", Json::str("stub")),
            ]),
        ),
        (
            "advisory",
            Json::obj([
                ("figure_type", Json::str("stub")),
                ("notes", Json::Array(vec![Json::str("Not advice.")])),
            ]),
        ),
    ])
}

/// The pass against `stub`, with how many requests it had seen at each pause.
fn pass(stub: &Stub) -> (live::Pass, Vec<usize>) {
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
    let mut pauses = Vec::new();
    let mut result = live::Pass::default();
    let mut pause = || {
        pauses.push(stub.requests());
        Ok(())
    };
    live::run(&mut client, &mut pause, &mut result, &mut |_| Ok(())).expect("a pass");
    (result, pauses)
}

/// A path for a record, with nothing at it.
fn record_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lodgeit-live-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a temporary directory");
    dir.join("record.json")
}

/// The real path's settings, except that they may reach the stub.
fn to_stub(stub: &Stub) -> AdapterConfig {
    AdapterConfig {
        allow_loopback: true,
        ..live::pass_config(&stub.base_url)
    }
}

fn written(path: &PathBuf) -> Json {
    pyjson::loads_standard(&fs::read_to_string(path).expect("a record")).expect("JSON")
}

/// `records/0/request`, through objects and lists.
fn at<'a>(json: &'a Json, path: &str) -> &'a Json {
    path.split('/')
        .fold(json, |node, step| match (node, step.parse::<usize>()) {
            (Json::Array(items), Ok(index)) => &items[index],
            _ => node
                .get(step)
                .unwrap_or_else(|| panic!("{path}: no {step}")),
        })
}

fn text<'a>(json: &'a Json, path: &str) -> &'a str {
    at(json, path)
        .as_str()
        .unwrap_or_else(|| panic!("{path} is not a string"))
}

fn len(json: &Json) -> usize {
    let Json::Array(items) = json else {
        panic!("a list")
    };
    items.len()
}

fn kind(json: &Json) -> &'static str {
    match json {
        Json::Null => "null",
        Json::Bool(_) => "bool",
        Json::Int(_) => "int",
        Json::Float(_) | Json::Dec(_) => "number",
        Json::Str(_) => "str",
        Json::Array(_) => "list",
        Json::Object(_) => "object",
    }
}

fn shaped(json: &Json, fields: &[(&str, &str)]) {
    for (path, want) in fields {
        assert_eq!(kind(at(json, path)), *want, "{path}");
    }
}

fn keys(json: &Json) -> Vec<String> {
    let Json::Object(pairs) = json else {
        panic!("an object")
    };
    let mut keys: Vec<String> = pairs
        .iter()
        .map(|(key, _)| key.as_str().expect("a key").to_owned())
        .collect();
    keys.sort();
    keys
}

/// The target and exact wire body of each sendable case, in case file order:
/// Division 7A, then FBT car, without the 2 cases the pass must never send.
fn sendable_requests() -> Vec<(String, Vec<u8>)> {
    const UNSENT: [&str; 2] = [
        "D7A-8-nil-remaining-term-refused",
        "D7A-9-unsupported-period",
    ];
    let field: Vec<u32> = "body".chars().map(u32::from).collect();
    let on_wire = |body: Json| {
        let points = wire::dumps(&body, &field).expect("exact");
        let text: String = points
            .iter()
            .map(|&point| char::from_u32(point).expect("a scalar"))
            .collect();
        text.into_bytes()
    };
    let div7a_cases = div7a::load_cases(None).expect("the packaged cases load");
    let div7a_requests = div7a_cases
        .iter()
        .filter(|case| !UNSENT.contains(&case.case_id.as_str()))
        .map(|case| {
            (
                format!("/v1/calculators/div7a/at/{}", case.period_uri),
                on_wire(div7a::build_request(case)),
            )
        });
    let fbt_cases = fbt::load_cases().expect("the packaged cases load");
    let fbt_requests = fbt_cases.iter().map(|case| {
        (
            format!(
                "/v1/calculators/{}/{}",
                fbt::CALC_URI,
                case.benefit.period_uri
            ),
            on_wire(case.benefit.request().expect("a sendable case")),
        )
    });
    div7a_requests.chain(fbt_requests).collect()
}

#[test]
fn a_clean_pass_sends_each_sendable_case_once_and_pauses_after_each() {
    let stub = stub(200, &answer());
    let (result, pauses) = pass(&stub);
    assert_eq!(result.stopped, None);
    // 7 Division 7A and 5 FBT car requests; D7A-8 and D7A-9 are never sent.
    assert_eq!(stub.requests(), 12);
    // A pause follows each request before the next goes out.
    assert_eq!(pauses, (1..=12).collect::<Vec<usize>>());
    assert_eq!(result.records.len(), 14);
    let want = sendable_requests();
    let distinct: HashSet<_> = want.iter().collect();
    assert_eq!(distinct.len(), want.len(), "two cases share a request");
    assert_eq!(*stub.seen.requests.lock().unwrap(), want);
}

#[test]
fn the_record_of_a_pass_is_written_with_its_money_as_exact_digits() {
    let stub = stub(200, &answer());
    let (result, _) = pass(&stub);
    let written = live::record(&result, "https://example.invalid", &Json::str("snapshot"));
    assert!(
        written.contains("\"amalgamated_base\": \"100000.00\""),
        "{written}"
    );
    let parsed = pyjson::loads_standard(&written).expect("the record is JSON");
    assert_eq!(len(at(&parsed, "records")), 14);
    let local = "comparisons/0/result/local";
    assert_eq!(
        text(&parsed, &format!("records/0/{local}/statutory_myr")),
        "19450.97"
    );
    // Records 7 and 8 are the 2 unsent cases, so 9 is FBT-1.
    assert_eq!(
        text(&parsed, &format!("records/9/{local}/taxable_value")),
        "8000.00"
    );
    assert_eq!(
        text(&parsed, "records/9/comparisons/1/result/local/fbt_estimate"),
        "7094.37"
    );
    assert_eq!(
        text(&parsed, "records/9/provider_result/fbt_payable"),
        "7094.37"
    );
}

#[test]
fn the_first_answer_that_is_not_computed_stops_the_pass() {
    let stub = stub(503, &Json::obj([("error", Json::str("unavailable"))]));
    let (result, pauses) = pass(&stub);
    assert_eq!(result.stopped.as_deref(), Some("D7A-1-met-first-real-year"));
    assert_eq!(stub.requests(), 1);
    assert!(pauses.is_empty());
    assert_eq!(result.records.len(), 1);
}

#[test]
fn a_refusal_stops_the_pass_too() {
    let body = Json::obj([("refusal_class", Json::str("out_of_scope"))]);
    let stub = stub(400, &body);
    let (result, _) = pass(&stub);
    assert_eq!(result.stopped.as_deref(), Some("D7A-1-met-first-real-year"));
    assert_eq!(stub.requests(), 1);
}

#[test]
fn without_enable_network_nothing_is_sent_and_no_file_is_made() {
    let stub = stub(200, &answer());
    let out = record_path("no-flag");
    let path = out.display().to_string();
    let args = |flags: &[&str]| -> Vec<String> {
        let tail = ["--base-url", &stub.base_url, "--out", &path];
        flags.iter().chain(&tail).map(ToString::to_string).collect()
    };
    assert_eq!(live::cli(&args(&[])), 2);
    assert!(!out.exists());
    // The flag is the difference: with it the pass starts, and the real
    // settings refuse the loopback stub before any request.
    assert_eq!(live::cli(&args(&["--enable-network"])), 1);
    assert!(out.exists());
    assert_eq!(stub.requests(), 0);
}

#[test]
fn the_real_path_makes_one_attempt_where_the_default_settings_would_retry() {
    assert!(AdapterConfig::default().max_attempts > BigInt::from(1));
    let stub = stub(503, &Json::obj([("error", Json::str("unavailable"))]));
    let out = record_path("one-attempt");
    let stopped = live::run_to_file(to_stub(&stub), &out, &mut || Ok(()));
    assert_eq!(
        stopped.expect("a pass").as_deref(),
        Some("D7A-1-met-first-real-year")
    );
    assert_eq!(stub.requests(), 1);
    assert_eq!(len(at(&written(&out), "records")), 1);
}

#[test]
fn an_error_part_way_through_keeps_what_was_sent() {
    let stub = stub(200, &answer());
    let out = record_path("error");
    let mut pause = || {
        Err(Exc {
            class: "OSError",
            text: "stub failure".chars().map(u32::from).collect(),
            notes: Vec::new(),
        })
    };
    let result = live::run_to_file(to_stub(&stub), &out, &mut pause);
    assert_eq!(result, Err("OSError: stub failure".to_string()));
    assert_eq!(stub.requests(), 1);
    let record = written(&out);
    assert_eq!(len(at(&record, "records")), 1);
    assert_eq!(text(&record, "error"), "OSError: stub failure");
}

#[test]
fn the_attempt_is_on_disk_before_the_request_arrives() {
    let stub = start(200, None);
    let out = record_path("attempt");
    *stub.seen.watch.lock().unwrap() = Some(out.clone());
    let _ = live::run_to_file(to_stub(&stub), &out, &mut || Ok(()));
    let on_disk = stub.seen.on_disk.lock().unwrap().clone().expect("a copy");
    let record = pyjson::loads_standard(&on_disk).expect("JSON");
    assert_eq!(at(&record, "records/0/attempted"), &Json::Bool(true));
    assert_eq!(
        text(&record, "records/0/request/amalgamated_base"),
        "100000.00"
    );
}

/// `invoke` itself returns an error here: the first answer is a 503, and the
/// back-off before the retry fails. `Client` takes its `sleep`, so no seam is
/// needed. `run_to_file` turns any error from `run` into the record's `error`,
/// as `an_error_part_way_through_keeps_what_was_sent` shows.
#[test]
fn an_error_from_invoke_leaves_the_saved_attempt() {
    let stub = stub(503, &Json::obj([("error", Json::str("unavailable"))]));
    let config = AdapterConfig {
        max_attempts: BigInt::from(2),
        ..to_stub(&stub)
    }
    .checked()
    .expect("a loopback config");
    let contract = contract::load("lodgeit-calculators", None).expect("the snapshot loads");
    let mut net = Net;
    let mut sleep = |_: f64| {
        Err(Exc {
            class: "OSError",
            text: "no back-off".chars().map(u32::from).collect(),
            notes: Vec::new(),
        })
    };
    let mut client = Client {
        config: &config,
        contract: &contract,
        dial: &mut net,
        sleep: &mut sleep,
    };
    let mut result = live::Pass::default();
    let mut saved = Vec::new();
    let mut save = |pass: &live::Pass| {
        saved.push(live::record(pass, "https://example.invalid", &Json::Null));
        Ok(())
    };
    let error =
        live::run(&mut client, &mut || Ok(()), &mut result, &mut save).expect_err("invoke's error");
    assert_eq!(error.class, "OSError");
    assert_eq!(stub.requests(), 1);
    assert_eq!(saved.len(), 1);
    let record = pyjson::loads_standard(&saved[0]).expect("JSON");
    assert_eq!(len(at(&record, "records")), 1);
    assert_eq!(at(&record, "records/0/attempted"), &Json::Bool(true));
    assert_eq!(
        text(&record, "records/0/request/amalgamated_base"),
        "100000.00"
    );
    assert_eq!(result.records.len(), 1);
}

#[test]
fn a_lost_connection_keeps_the_request_and_stops() {
    let stub = start(200, None);
    let out = record_path("lost");
    let result = live::run_to_file(to_stub(&stub), &out, &mut || Ok(()));
    assert_eq!(stub.requests(), 1);
    let record = written(&out);
    assert_eq!(len(at(&record, "records")), 1);
    assert_eq!(
        text(&record, "records/0/request/amalgamated_base"),
        "100000.00"
    );
    assert_eq!(
        result.expect("an outcome").as_deref(),
        Some("D7A-1-met-first-real-year")
    );
    assert_eq!(text(&record, "stopped_at"), "D7A-1-met-first-real-year");
}

#[test]
fn a_malformed_response_keeps_the_request_and_stops() {
    for reply in [
        b"not JSON".to_vec(),
        br#"{"statutory_myr": "not a number"}"#.to_vec(),
    ] {
        let stub = start(200, Some(reply));
        let out = record_path("malformed");
        let result = live::run_to_file(to_stub(&stub), &out, &mut || Ok(()));
        assert_eq!(stub.requests(), 1);
        let record = written(&out);
        assert_eq!(len(at(&record, "records")), 1);
        assert_eq!(
            text(&record, "records/0/request/amalgamated_base"),
            "100000.00"
        );
        assert_eq!(
            result.expect("an outcome").as_deref(),
            Some("D7A-1-met-first-real-year")
        );
    }
}

#[test]
fn each_completed_response_is_on_disk_at_the_next_pause() {
    let stub = stub(200, &answer());
    let out = record_path("incremental");
    let mut counts = Vec::new();
    let mut pause = || {
        let record = written(&out);
        counts.push(len(at(&record, "records")));
        assert_eq!(text(&record, "records/0/comparisons/0/trial"), "div7a");
        if counts.len() == 2 {
            return Err(Exc {
                class: "OSError",
                text: "enough".chars().map(u32::from).collect(),
                notes: Vec::new(),
            });
        }
        Ok(())
    };
    assert!(live::run_to_file(to_stub(&stub), &out, &mut pause).is_err());
    assert_eq!(counts, [1, 2]);
}

#[test]
fn the_committed_record_keeps_its_schema_and_says_how_it_differs_from_the_writer() {
    let committed = pyjson::loads_standard(COMMITTED).expect("the record is JSON");
    let mut writer: Vec<String> = keys(
        &pyjson::loads_standard(&live::record(
            &live::Pass::default(),
            "https://example.invalid",
            &Json::Null,
        ))
        .expect("JSON"),
    );
    writer.retain(|key| key != "error");
    writer.push("edits_after_the_run".to_owned());
    writer.sort();
    assert_eq!(keys(&committed), writer);

    let edits = pyjson::dumps(at(&committed, "edits_after_the_run"), Layout::Canonical);
    for named in ["independently derived", "`error`"] {
        assert!(edits.contains(named), "{edits}");
    }

    let Json::Array(records) = at(&committed, "records") else {
        panic!("a list")
    };
    assert_eq!(records.len(), 14);
    let sent: Vec<&Json> = records
        .iter()
        .filter(|record| record.get("sent") == Some(&Json::Bool(true)))
        .collect();
    assert_eq!(sent.len(), 12);
    for record in sent {
        shaped(
            record,
            &[
                ("http_status", "int"),
                ("observed_at", "str"),
                ("findings", "list"),
                ("comparisons", "list"),
                ("comparisons/0/trial", "str"),
                ("comparisons/0/result", "object"),
                ("comparisons/0/result/evaluation", "str"),
                ("request", "object"),
                ("provider_result", "object"),
                ("provider_result/advisory", "object"),
                ("provider_result/manifest", "object"),
                ("provider_result/manifest/rate_table_uris", "list"),
                ("provider_result/rate_uris_consumed", "list"),
                ("provider_result/rounding_policy", "str"),
            ],
        );
        assert_eq!(at(record, "http_status"), &Json::Int(200.into()));
        if at(record, "request").get("amalgamated_base").is_some() {
            shaped(
                record,
                &[
                    ("request/amalgamated_base", "str"),
                    ("request/income_year_start_date", "str"),
                    ("request/is_first_real_myr_year", "bool"),
                    ("request/loan_origination_date", "str"),
                    ("request/loan_term_years", "int"),
                    ("request/repayments", "list"),
                    ("provider_result/benchmark_rate", "str"),
                    ("provider_result/deemed_dividend", "str"),
                    ("provider_result/is_complying", "bool"),
                    ("provider_result/remaining_term_years", "str"),
                    ("provider_result/shortfall", "str"),
                    ("provider_result/statutory_myr", "str"),
                    ("provider_result/total_repayments", "str"),
                ],
            );
        } else {
            shaped(
                record,
                &[
                    ("request/accessories", "str"),
                    ("request/baseValue", "str"),
                    ("request/daysAvailable", "int"),
                    ("request/employeeContribution", "str"),
                    ("request/fbtType", "str"),
                    ("comparisons/1/result", "object"),
                    ("provider_result/fbt_payable", "str"),
                    ("provider_result/gross_up_factor", "str"),
                    ("provider_result/grossed_up_taxable_value", "str"),
                    ("provider_result/taxable_value", "str"),
                ],
            );
        }
    }
    let reasons = pyjson::dumps(at(&records[3], "comparisons"), Layout::Canonical);
    assert!(reasons.contains("D7A-4-later-year-not-first"));
    assert!(
        reasons
            .contains("differs from the intentionally inconsistent fixture expectation 29507.73"),
        "{reasons}"
    );
    assert!(!COMMITTED.contains("from the independently derived 29507.73\","));
}

#[test]
fn a_pass_that_could_not_keep_its_record_never_starts() {
    let stub = stub(200, &answer());
    let taken = record_path("taken");
    fs::write(&taken, "an earlier record").expect("a file");
    let missing = record_path("missing").with_file_name("no-such-folder/record.json");
    for out in [&taken, &missing] {
        let result = live::run_to_file(to_stub(&stub), out, &mut || Ok(()));
        assert!(result.is_err(), "{}", out.display());
    }
    assert_eq!(stub.requests(), 0);
    assert_eq!(fs::read_to_string(&taken).unwrap(), "an earlier record");
}

#[test]
fn the_local_worksheet_reproduces_each_cases_derived_taxable_value() {
    for case in fbt::load_cases().expect("the packaged cases load") {
        let local = fbt::run_local_statutory(&case.benefit).expect("a worksheet result");
        assert_eq!(
            local[0].1.to_sci_string(),
            case.expected[0].1.to_sci_string(),
            "{}",
            case.benefit.case_id
        );
    }
}

#[test]
fn a_formula_difference_past_the_last_cent_is_named() {
    let case = fbt::load_cases()
        .expect("the packaged cases load")
        .remove(1);
    let mut outcome = lodgeit_calculator_adapter::client::Outcome {
        status: lodgeit_calculator_adapter::client::Status::Computed,
        calculator: None,
        period: None,
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: Some(200),
        result: Json::Null,
        values: Vec::new(),
        manifest: Json::Null,
        advisory: Json::Null,
        upstream_refusal_class: Json::Null,
        findings: Vec::new(),
        observed_at: String::new(),
        elapsed_ms: None,
        contract_snapshot: Json::Null,
        contract_sha256: String::new(),
        base_url: None,
    };
    let name: Vec<u32> = "taxable_value".chars().map(u32::from).collect();
    let figure = |text: &str| {
        lodgeit_calculator_adapter::decimal::Decimal::parse_exact(text).expect("a literal")
    };
    outcome.values = vec![(name.clone(), figure("4804.38"))];
    let agreed = fbt::compare_formula(&case, &outcome).expect("a comparison");
    assert_eq!(agreed.evaluation, Evaluation::Match);
    // 1 cent is the deliberate last-cent tolerance; 2 cents is a difference.
    outcome.values = vec![(name.clone(), figure("4804.39"))];
    let tolerated = fbt::compare_formula(&case, &outcome).expect("a comparison");
    assert_eq!(tolerated.evaluation, Evaluation::Match);
    outcome.values = vec![(name, figure("4804.40"))];
    let differed = fbt::compare_formula(&case, &outcome).expect("a comparison");
    assert_eq!(differed.evaluation, Evaluation::NumericDifference);
}
